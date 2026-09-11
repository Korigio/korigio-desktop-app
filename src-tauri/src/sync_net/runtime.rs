//! SyncRuntime: UDP discovery + encrypted TCP mesh.
//! The DB mutex is never held across network IO.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::db::repository::now_utc_rfc3339;
use crate::domain::identity;
use crate::domain::sync::apply::ApplyOutcome;
use crate::domain::sync::hlc::Hlc;
use crate::domain::sync::record::{changes_after, get_peer_cursor, set_peer_cursor, SyncChange};
use crate::domain::sync::snapshot::{dump_snapshot, list_blob_hashes};
use crate::domain::sync::{
    apply_snapshot_row, idle_sync_status, PresenceItem, PresenceListResult, PresenceStaff,
    SyncStatus,
};
use crate::domain::team::constants::NEARBY_TTL_SECS;
use crate::domain::team::invite::hash_invite_code;
use crate::domain::team::pin::hash_team_pin;
use crate::domain::team::{JoinGrant, NearbyTeam};
use crate::error::AppError;
use crate::sync_net::crypto::{auth_mac, join_key_from_pin, team_key_from_psk_hex, PROTO};
use crate::sync_net::frame::{decode_frame, encode_frame};
use crate::sync_net::protocol::{
    ChangeWire, SyncAppliedPayload, TcpMessage, UdpBeacon, BLOB_CHUNK_MAX, BLOB_WANT_MAX,
    CHANGE_PUSH_MAX, HEARTBEAT_SECS, ONLINE_WINDOW_SECS, SCHEMA_VERSION, SYNC_APPLIED_EVENT,
    TCP_PORT, UDP_MAX_BYTES, UDP_PORT,
};

const DIAL_TIMEOUT: Duration = Duration::from_secs(3);
const CONNECT_FAIL_MSG: &str = "Could not reach the other computer on the network.";

mod network;

use network::lan_loop;

#[derive(Debug, Default)]
struct Inner {
    online: HashSet<String>,
    last_seen: HashMap<String, String>,
    last_synced_at: Option<String>,
    state: String,
    error_message: Option<String>,
    catching_up: bool,
    invite_codes: HashMap<String, String>,
    join_team_id: Option<String>,
    join_pin: Option<String>,
    join_member_name: Option<String>,
    join_grant: Option<JoinGrant>,
    nearby: HashMap<String, NearbyEntry>,
    connected: HashSet<String>,
    peer_addrs: HashMap<String, SocketAddr>,
    snapshot_rows: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
struct NearbyEntry {
    name: String,
    seen: std::time::Instant,
}

#[derive(Clone)]
pub struct SyncRuntime {
    inner: Arc<Mutex<Inner>>,
    interval_secs: Arc<AtomicU64>,
    notify: Arc<tokio::sync::Notify>,
    app: Arc<Mutex<Option<AppHandle>>>,
}

impl SyncRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                state: "idle".into(),
                ..Inner::default()
            })),
            interval_secs: Arc::new(AtomicU64::new(HEARTBEAT_SECS)),
            notify: Arc::new(tokio::sync::Notify::new()),
            app: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, db: Arc<Mutex<crate::db::Db>>, handle: AppHandle) {
        if let Ok(guard) = db.lock() {
            if let Ok(settings) = crate::domain::settings::get_sync_interval(guard.conn()) {
                self.interval_secs
                    .store(settings.interval_seconds, Ordering::Relaxed);
            }
        }
        if let Ok(mut app) = self.app.lock() {
            *app = Some(handle);
        }
        let runtime = self.clone();
        std::thread::Builder::new()
            .name("servioo-lan".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(_) => return,
                };
                rt.block_on(lan_loop(runtime, db));
            })
            .ok();
    }

    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs())
    }

    pub fn interval_secs(&self) -> u64 {
        self.interval_secs.load(Ordering::Relaxed)
    }

    pub fn set_interval(&self, secs: u64) {
        self.interval_secs.store(secs, Ordering::Relaxed);
        self.nudge();
    }

    pub fn nudge(&self) {
        self.notify.notify_waiters();
    }

    fn online_window_secs(&self) -> u64 {
        ONLINE_WINDOW_SECS.max(self.interval_secs().saturating_mul(3))
    }

    fn nearby_ttl_secs(&self) -> u64 {
        NEARBY_TTL_SECS.max(self.interval_secs().saturating_mul(4))
    }

    fn emit_sync_applied(&self, applied: u64) {
        if applied == 0 {
            return;
        }
        let Ok(at) = now_utc_rfc3339() else {
            return;
        };
        let Ok(guard) = self.app.lock() else {
            return;
        };
        let Some(handle) = guard.as_ref() else {
            return;
        };
        let _ = handle.emit(SYNC_APPLIED_EVENT, SyncAppliedPayload { at, applied });
    }

    pub fn remember_invite(&self, code: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .invite_codes
                .insert(hash_invite_code(code), code.to_string());
        }
    }

    pub fn begin_join(&self, team_id: String, pin: String, member_name: String) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.join_team_id = Some(team_id);
            inner.join_pin = Some(pin);
            inner.join_member_name = Some(member_name);
            inner.join_grant = None;
        }
    }

    pub fn take_join_grant(&self) -> Option<JoinGrant> {
        self.inner.lock().ok()?.join_grant.take()
    }

    pub fn clear_join(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.join_team_id = None;
            inner.join_pin = None;
            inner.join_member_name = None;
        }
    }

    pub fn remember_nearby(&self, team_id: String, name: String) {
        if team_id.is_empty() {
            return;
        }
        if let Ok(mut inner) = self.inner.lock() {
            inner.nearby.insert(
                team_id,
                NearbyEntry {
                    name,
                    seen: std::time::Instant::now(),
                },
            );
        }
    }

    pub fn nearby_teams(&self) -> Vec<NearbyTeam> {
        let Ok(mut inner) = self.inner.lock() else {
            return Vec::new();
        };
        let cutoff = std::time::Duration::from_secs(self.nearby_ttl_secs());
        inner
            .nearby
            .retain(|_, entry| entry.seen.elapsed() <= cutoff);
        let mut items: Vec<NearbyTeam> = inner
            .nearby
            .iter()
            .map(|(team_id, entry)| NearbyTeam {
                team_id: team_id.clone(),
                name: entry.name.clone(),
            })
            .collect();
        items.sort_by(|a, b| a.name.cmp(&b.name).then(a.team_id.cmp(&b.team_id)));
        items
    }

    pub fn online_device_ids(&self) -> HashSet<String> {
        self.inner
            .lock()
            .map(|inner| inner.online.clone())
            .unwrap_or_default()
    }

    pub fn mark_online(&self, device_id: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.online.insert(device_id.to_string());
            if let Ok(now) = now_utc_rfc3339() {
                inner.last_seen.insert(device_id.to_string(), now);
            }
        }
    }

    pub fn list_presence(&self, conn: &Connection) -> Result<PresenceListResult, AppError> {
        let identity = identity::require_local_identity(conn)?;
        let Some(team_id) = identity.team_id.as_deref() else {
            return Ok(PresenceListResult { items: Vec::new() });
        };
        let devices = crate::domain::team::list_team_devices(conn)?;
        let inner = self.inner.lock().map_err(|_| AppError::Internal {
            message: "sync runtime lock poisoned".into(),
        })?;
        let mut items = Vec::new();
        for device in devices {
            if device.removed_at.is_some() {
                continue;
            }
            let online = device.is_this_device || inner.online.contains(&device.id);
            let last_seen = if device.is_this_device {
                now_utc_rfc3339().ok()
            } else {
                inner.last_seen.get(&device.id).cloned()
            };
            let staff = presence_staff_for(conn, &device.id, device.is_this_device)?;
            items.push(PresenceItem {
                device_id: device.id,
                device_name: device.device_name,
                device_code: device.device_code,
                is_this_device: device.is_this_device,
                online,
                last_seen_at: last_seen,
                staff,
            });
        }
        items.sort_by(|a, b| {
            b.is_this_device
                .cmp(&a.is_this_device)
                .then(a.device_name.cmp(&b.device_name))
        });
        let _ = team_id;
        Ok(PresenceListResult { items })
    }

    pub fn status(&self, conn: &Connection) -> Result<SyncStatus, AppError> {
        let identity = identity::require_local_identity(conn)?;
        if identity.team_id.is_none() {
            return Ok(idle_sync_status());
        }
        let inner = self.inner.lock().map_err(|_| AppError::Internal {
            message: "sync runtime lock poisoned".into(),
        })?;
        let peers_online = inner.online.len() as i64;
        let state = if inner.error_message.is_some() {
            "error"
        } else if inner.catching_up {
            "catchingUp"
        } else if peers_online == 0 {
            "offline"
        } else {
            "synced"
        };
        let pending_outgoing = crate::domain::sync::count_pending_outgoing(conn)?;
        Ok(SyncStatus {
            team_id: identity.team_id,
            state: state.into(),
            last_synced_at: inner.last_synced_at.clone(),
            peers_online,
            pending_outgoing,
            pending_incoming: 0,
            error_message: inner.error_message.clone(),
        })
    }
}

fn presence_staff_for(
    conn: &Connection,
    device_id: &str,
    is_this: bool,
) -> Result<Option<PresenceStaff>, AppError> {
    if is_this {
        if let Some(session) = crate::domain::staff::get_current_session(conn)? {
            return Ok(Some(PresenceStaff {
                id: session.staff.id,
                name: session.staff.name,
                role: session.staff.role.as_str().to_string(),
            }));
        }
        return Ok(None);
    }
    let mut stmt = conn.prepare(
        "SELECT staff.id, staff.name, staff.role
         FROM presence
         INNER JOIN staff ON staff.id = presence.staff_id
         WHERE presence.device_id = ?1",
    )?;
    let row = stmt
        .query_row([device_id], |row| {
            Ok(PresenceStaff {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
            })
        })
        .optional()?;
    Ok(row)
}

use rusqlite::OptionalExtension;
