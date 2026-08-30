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

#[derive(Clone)]
struct NetIdentity {
    device_id: String,
    device_name: String,
    team_id: Option<String>,
    team_name: Option<String>,
    team_psk: Option<String>,
    team_pin: Option<String>,
    staff_id: Option<String>,
}

fn snapshot_identity(db: &Arc<Mutex<crate::db::Db>>) -> Option<NetIdentity> {
    let guard = db.lock().ok()?;
    let identity = identity::get_local_identity(guard.conn()).ok().flatten()?;
    let team_name = identity.team_id.as_ref().and_then(|id| {
        crate::domain::team::repository::get_team_row(guard.conn(), id)
            .ok()
            .flatten()
            .map(|(_, name, _, _)| name)
    });
    Some(NetIdentity {
        device_id: identity.device_id,
        device_name: identity.device_name,
        team_id: identity.team_id,
        team_name,
        team_psk: identity.team_psk,
        team_pin: identity.team_pin,
        staff_id: identity.current_staff_id,
    })
}

async fn lan_loop(runtime: SyncRuntime, db: Arc<Mutex<crate::db::Db>>) {
    let udp = match UdpSocket::bind(("0.0.0.0", UDP_PORT)).await {
        Ok(s) => s,
        Err(_) => {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.error_message = Some("Could not bind the team discovery port.".into());
                inner.state = "error".into();
            }
            idle_poll(runtime, db).await;
            return;
        }
    };
    let _ = udp.set_broadcast(true);
    let tcp = match TcpListener::bind(("0.0.0.0", TCP_PORT)).await {
        Ok(s) => s,
        Err(_) => {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.error_message = Some("Could not bind the team sync port.".into());
                inner.state = "error".into();
            }
            idle_poll(runtime, db).await;
            return;
        }
    };

    let mut buf = [0u8; UDP_MAX_BYTES];
    let sleep = tokio::time::sleep(Duration::ZERO);
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => {
                expire_online(&runtime);
                broadcast_beacon(&udp, &runtime, &db).await;
                sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
            }
            _ = runtime.notify.notified() => {
                expire_online(&runtime);
                broadcast_beacon(&udp, &runtime, &db).await;
                sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
            }
            Ok((len, src)) = udp.recv_from(&mut buf) => {
                handle_udp(&runtime, &db, &buf[..len], src);
            }
            Ok((stream, _)) = tcp.accept() => {
                let runtime = runtime.clone();
                let db = db.clone();
                tokio::spawn(async move {
                    let _ = handle_incoming(runtime, db, stream).await;
                });
            }
        }
    }
}

async fn idle_poll(runtime: SyncRuntime, db: Arc<Mutex<crate::db::Db>>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(runtime.interval()) => {}
            _ = runtime.notify.notified() => {}
        }
        let snapshot = snapshot_identity(&db);
        if snapshot.and_then(|s| s.team_id).is_none() {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.state = "idle".into();
                inner.online.clear();
            }
        }
    }
}

fn expire_online(runtime: &SyncRuntime) {
    let cutoff = match now_utc_rfc3339() {
        Ok(now) => now,
        Err(_) => return,
    };
    if let Ok(mut inner) = runtime.inner.lock() {
        let stale: Vec<String> = inner
            .last_seen
            .iter()
            .filter(|(_, seen)| {
                match time::OffsetDateTime::parse(
                    seen,
                    &time::format_description::well_known::Rfc3339,
                ) {
                    Ok(ts) => {
                        (time::OffsetDateTime::now_utc() - ts).whole_seconds()
                            > runtime.online_window_secs() as i64
                    }
                    Err(_) => true,
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in stale {
            inner.online.remove(&id);
            inner.last_seen.remove(&id);
        }
        let _ = cutoff;
    }
}

async fn broadcast_beacon(udp: &UdpSocket, runtime: &SyncRuntime, db: &Arc<Mutex<crate::db::Db>>) {
    let identity = snapshot_identity(db);
    let (join_team_id, join_pin) = {
        let Ok(inner) = runtime.inner.lock() else {
            return;
        };
        (inner.join_team_id.clone(), inner.join_pin.clone())
    };

    let beacon = if let (Some(team_id), Some(pin)) = (join_team_id, join_pin) {
        let Some(identity) = identity else { return };
        UdpBeacon {
            v: 2,
            kind: "join".into(),
            team_id,
            team_name: String::new(),
            device_id: identity.device_id,
            tcp_port: TCP_PORT,
            pin_hash: Some(hash_team_pin(&pin)),
        }
    } else if let Some(identity) = identity {
        let Some(team_id) = identity.team_id else {
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.state = "idle".into();
                inner.online.clear();
            }
            return;
        };
        if let Ok(mut inner) = runtime.inner.lock() {
            if inner.state == "idle" {
                inner.state = "offline".into();
            }
        }
        UdpBeacon {
            v: 2,
            kind: "hello".into(),
            team_id,
            team_name: identity.team_name.unwrap_or_default(),
            device_id: identity.device_id,
            tcp_port: TCP_PORT,
            pin_hash: None,
        }
    } else {
        return;
    };

    let Ok(bytes) = serde_json::to_vec(&beacon) else {
        return;
    };
    if bytes.len() > UDP_MAX_BYTES {
        return;
    }
    let _ = udp.send_to(&bytes, ("255.255.255.255", UDP_PORT)).await;
    let peer_addrs: Vec<SocketAddr> = {
        let Ok(inner) = runtime.inner.lock() else {
            return;
        };
        inner.peer_addrs.values().copied().collect()
    };
    for addr in peer_addrs {
        let dest = SocketAddr::new(addr.ip(), UDP_PORT);
        let _ = udp.send_to(&bytes, dest).await;
    }
}

fn handle_udp(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    bytes: &[u8],
    src: SocketAddr,
) {
    let Ok(beacon) = serde_json::from_slice::<UdpBeacon>(bytes) else {
        return;
    };
    if beacon.v != 2 {
        return;
    }
    let Some(me) = snapshot_identity(db) else {
        return;
    };
    if beacon.device_id == me.device_id {
        return;
    }
    if device_is_removed(db, &beacon.device_id) {
        return;
    }
    remember_peer_addr(runtime, &beacon.device_id, src);

    if beacon.kind == "hello" && me.team_id.is_none() {
        runtime.remember_nearby(beacon.team_id, beacon.team_name);
        return;
    }

    if beacon.kind == "join" {
        let Some(hash) = beacon.pin_hash.clone() else {
            return;
        };
        let Some(my_team) = me.team_id.as_deref() else {
            return;
        };
        if beacon.team_id != my_team {
            return;
        }
        let Some(pin) = me.team_pin.clone() else {
            return;
        };
        if hash_team_pin(&pin) != hash {
            return;
        }
        spawn_dial(
            runtime.clone(),
            db.clone(),
            src.ip().to_string(),
            beacon.tcp_port,
            beacon.device_id,
            Some(pin),
        );
        return;
    }

    let Some(team_id) = me.team_id.as_deref() else {
        return;
    };
    if beacon.kind != "hello" || beacon.team_id != team_id {
        return;
    }
    if should_dial_on_hello(&me.device_id, &beacon.device_id) {
        spawn_dial(
            runtime.clone(),
            db.clone(),
            src.ip().to_string(),
            beacon.tcp_port,
            beacon.device_id,
            None,
        );
    }
}

fn device_is_removed(db: &Arc<Mutex<crate::db::Db>>, device_id: &str) -> bool {
    let Ok(guard) = db.lock() else {
        return false;
    };
    guard
        .conn()
        .query_row(
            "SELECT removed_at FROM team_devices WHERE id = ?1",
            [device_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .is_some()
}

fn spawn_dial(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    ip: String,
    port: u16,
    peer_id: String,
    join_pin: Option<String>,
) {
    {
        let Ok(mut inner) = runtime.inner.lock() else {
            return;
        };
        if !inner.connected.insert(peer_id.clone()) {
            return;
        }
    }
    tokio::spawn(async move {
        let addr = format!("{ip}:{port}");
        match tokio::time::timeout(DIAL_TIMEOUT, TcpStream::connect(&addr)).await {
            Ok(Ok(stream)) => {
                let _ = drive_session(runtime.clone(), db, stream, Some(peer_id.clone()), join_pin)
                    .await;
            }
            Ok(Err(_)) | Err(_) => {
                if let Ok(mut inner) = runtime.inner.lock() {
                    if !inner.online.contains(&peer_id) {
                        inner.error_message = Some(CONNECT_FAIL_MSG.into());
                    }
                }
            }
        }
        if let Ok(mut inner) = runtime.inner.lock() {
            inner.connected.remove(&peer_id);
        }
    });
}

async fn handle_incoming(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    stream: TcpStream,
) -> Result<(), AppError> {
    drive_session(runtime, db, stream, None, None).await
}

async fn drive_session(
    runtime: SyncRuntime,
    db: Arc<Mutex<crate::db::Db>>,
    mut stream: TcpStream,
    expected_peer: Option<String>,
    join_pin: Option<String>,
) -> Result<(), AppError> {
    let me = snapshot_identity(&db).ok_or_else(|| AppError::sync_err("Identity is missing."))?;
    let pending_join = runtime
        .inner
        .lock()
        .ok()
        .and_then(|inner| inner.join_pin.clone());
    let is_join = use_join_key(
        join_pin.as_deref(),
        pending_join.as_deref(),
        me.team_id.as_deref(),
    );
    let (key, psk_bytes) = if is_join {
        let pin = join_pin
            .clone()
            .or(pending_join.clone())
            .ok_or_else(|| AppError::sync_err("Join PIN is missing."))?;
        (join_key_from_pin(&pin)?, pin.as_bytes().to_vec())
    } else {
        let psk_hex = me
            .team_psk
            .clone()
            .ok_or_else(|| AppError::sync_err("Team key is missing."))?;
        let raw = hex::decode(&psk_hex).map_err(|_| AppError::sync_err("Team key is invalid."))?;
        (team_key_from_psk_hex(&psk_hex)?, raw)
    };

    let mut my_nonce = [0u8; 32];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut my_nonce);
    write_msg(
        &mut stream,
        &key,
        &TcpMessage::Hello {
            proto: PROTO.into(),
            device_id: me.device_id.clone(),
            nonce: hex::encode(my_nonce),
        },
    )
    .await?;
    let hello = read_msg(&mut stream, &key).await?;
    let (peer_id, peer_nonce_hex) = match hello {
        TcpMessage::Hello {
            proto,
            device_id,
            nonce,
        } if proto == PROTO => (device_id, nonce),
        _ => return Err(AppError::sync_err("Peer handshake was rejected.")),
    };
    if let Some(expected) = expected_peer {
        if expected != peer_id {
            return Err(AppError::sync_err("Peer identity did not match."));
        }
    }
    if device_is_removed(&db, &peer_id) {
        return Err(AppError::sync_err("This device was removed from the team."));
    }
    let peer_nonce = hex::decode(&peer_nonce_hex)
        .map_err(|_| AppError::sync_err("Peer handshake was rejected."))?;
    write_msg(
        &mut stream,
        &key,
        &TcpMessage::Auth {
            mac: auth_mac(&psk_bytes, &peer_nonce, &me.device_id),
        },
    )
    .await?;
    let auth = read_msg(&mut stream, &key).await?;
    let TcpMessage::Auth { mac } = auth else {
        return Err(AppError::sync_err("Peer handshake was rejected."));
    };
    if mac != auth_mac(&psk_bytes, &my_nonce, &peer_id) {
        return Err(AppError::sync_err("Peer handshake was rejected."));
    }

    runtime.mark_online(&peer_id);
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.connected.insert(peer_id.clone());
        inner.error_message = None;
    }

    let result = if is_join {
        if me.team_id.is_some() {
            if let Some(pin) = join_pin.or(pending_join) {
                let grant = {
                    let guard = db.lock().map_err(|_| AppError::Internal {
                        message: "database lock poisoned".into(),
                    })?;
                    crate::domain::team::issue_join_grant(guard.conn(), &pin, &peer_id, "PC")?
                };
                write_msg(
                    &mut stream,
                    &key,
                    &TcpMessage::Grant {
                        team_id: grant.team_id.clone(),
                        team_psk_hex: grant.team_psk_hex.clone(),
                        device_code: grant.device_code.clone(),
                    },
                )
                .await?;
                send_join_snapshot(&mut stream, &key, &db).await?;
                wait_for_join_blob_want(&mut stream, &key, &db).await?;
            }
        } else {
            receive_join_snapshot(&runtime, &db, &mut stream, &key, &peer_id).await?;
        }
        Ok(())
    } else {
        gossip_loop(&runtime, &db, stream, &key, &me, &peer_id).await
    };

    if let Ok(mut inner) = runtime.inner.lock() {
        inner.connected.remove(&peer_id);
        inner.snapshot_rows.remove(&peer_id);
    }
    result
}

/// Use the join/PIN key only for a real join session — never leftover PIN after this PC is in a team.
fn use_join_key(
    join_pin: Option<&str>,
    pending_join: Option<&str>,
    team_id: Option<&str>,
) -> bool {
    if join_pin.is_some() {
        return true;
    }
    pending_join.is_some() && team_id.is_none()
}

/// Both sides always dial on team hello so a one-way firewall cannot stall gossip.
fn should_dial_on_hello(_local_device_id: &str, _peer_device_id: &str) -> bool {
    true
}

fn is_default_empty_cursor(hlc: &Hlc) -> bool {
    hlc.wall == 0 && hlc.counter == 0
}

fn remember_peer_addr(runtime: &SyncRuntime, device_id: &str, src: SocketAddr) {
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.peer_addrs.insert(device_id.to_string(), src);
    }
}

async fn maybe_send_catchup_snapshot(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    rx: &mut tokio::sync::mpsc::Receiver<Result<TcpMessage, AppError>>,
) -> Result<(), AppError> {
    let empty = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        is_default_empty_cursor(&get_peer_cursor(guard.conn(), peer_id)?)
    };
    if !empty {
        return Ok(());
    }
    if let Ok(mut inner) = runtime.inner.lock() {
        inner.catching_up = true;
    }
    let (rows, hashes) = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        let (_schema_version, rows) = dump_snapshot(guard.conn())?;
        let hashes = list_blob_hashes(guard.conn())?;
        (rows, hashes)
    };
    drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
    write_locked(
        writer,
        key,
        &TcpMessage::SnapshotBegin {
            schema_version: SCHEMA_VERSION,
            counts: rows.len() as i64,
        },
    )
    .await?;
    for row in rows {
        drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
        write_locked(
            writer,
            key,
            &TcpMessage::SnapshotRow {
                table: row.table,
                payload: row.payload,
            },
        )
        .await?;
    }
    if !hashes.is_empty() {
        drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
        write_locked(writer, key, &TcpMessage::SnapshotBlobList { hashes }).await?;
    }
    drain_gossip_inbox(runtime, db, writer, key, peer_id, rx).await?;
    write_locked(writer, key, &TcpMessage::SnapshotEnd).await
}

async fn drain_gossip_inbox(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    rx: &mut tokio::sync::mpsc::Receiver<Result<TcpMessage, AppError>>,
) -> Result<(), AppError> {
    loop {
        match rx.try_recv() {
            Ok(msg) => {
                handle_tcp_message(runtime, db, writer, key, peer_id, msg?).await?;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return Ok(()),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                return Err(AppError::sync_err("Peer connection closed."));
            }
        }
    }
}

type TcpWriter = Arc<tokio::sync::Mutex<tokio::net::tcp::OwnedWriteHalf>>;

async fn gossip_loop(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    stream: TcpStream,
    key: &[u8; 32],
    me: &NetIdentity,
    peer_id: &str,
) -> Result<(), AppError> {
    let (read_half, write_half) = stream.into_split();
    let writer: TcpWriter = Arc::new(tokio::sync::Mutex::new(write_half));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<TcpMessage, AppError>>(16);
    let reader_key = *key;
    let reader = tokio::spawn(async move {
        let mut read_half = read_half;
        loop {
            match read_msg(&mut read_half, &reader_key).await {
                Ok(msg) => {
                    if tx.send(Ok(msg)).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(err)).await;
                    break;
                }
            }
        }
    });

    let result = async {
        send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
        maybe_send_catchup_snapshot(runtime, db, &writer, key, peer_id, &mut rx).await?;
        let sleep = tokio::time::sleep(runtime.interval());
        tokio::pin!(sleep);
        loop {
            tokio::select! {
                _ = &mut sleep => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                _ = runtime.notify.notified() => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                _ = crate::domain::sync::local_change_notified() => {
                    send_heartbeat_and_pull(db, &writer, key, me, peer_id).await?;
                    sleep.as_mut().reset(tokio::time::Instant::now() + runtime.interval());
                }
                msg = rx.recv() => {
                    let Some(msg) = msg else {
                        return Err(AppError::sync_err("Peer connection closed."));
                    };
                    handle_tcp_message(runtime, db, &writer, key, peer_id, msg?).await?;
                }
            }
        }
    }
    .await;
    reader.abort();
    result
}

async fn send_heartbeat_and_pull(
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    me: &NetIdentity,
    peer_id: &str,
) -> Result<(), AppError> {
    let hlc_max = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        identity::require_local_identity(guard.conn())
            .map(|id| Hlc::new(id.hlc_wall_ms, id.hlc_counter, id.device_id))?
    };
    write_locked(
        writer,
        key,
        &TcpMessage::Heartbeat {
            staff_id: me.staff_id.clone(),
            device_name: me.device_name.clone(),
            hlc_max,
        },
    )
    .await?;
    let after = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        get_peer_cursor(guard.conn(), peer_id)?
    };
    write_locked(writer, key, &TcpMessage::ChangePull { after }).await?;
    Ok(())
}

async fn write_locked(
    writer: &TcpWriter,
    key: &[u8; 32],
    msg: &TcpMessage,
) -> Result<(), AppError> {
    let mut write_half = writer.lock().await;
    write_msg(&mut *write_half, key, msg).await
}

async fn handle_tcp_message(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    writer: &TcpWriter,
    key: &[u8; 32],
    peer_id: &str,
    msg: TcpMessage,
) -> Result<(), AppError> {
    match msg {
        TcpMessage::Heartbeat {
            staff_id,
            device_name: _,
            hlc_max: _,
        } => {
            runtime.mark_online(peer_id);
            let now = now_utc_rfc3339()?;
            let guard = db.lock().map_err(|_| AppError::Internal {
                message: "database lock poisoned".into(),
            })?;
            guard.conn().execute(
                "INSERT INTO presence (id, device_id, staff_id, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(device_id) DO UPDATE SET
                    staff_id = excluded.staff_id,
                    last_seen_at = excluded.last_seen_at",
                rusqlite::params![crate::domain::ids::new_entity_id(), peer_id, staff_id, now],
            )?;
        }
        TcpMessage::ChangePull { after } => {
            let changes = {
                let guard = db.lock().map_err(|_| AppError::Internal {
                    message: "database lock poisoned".into(),
                })?;
                changes_after(guard.conn(), &after, CHANGE_PUSH_MAX as i64)?
            };
            if !changes.is_empty() {
                write_locked(
                    writer,
                    key,
                    &TcpMessage::ChangePush {
                        changes: changes.into_iter().map(to_wire).collect(),
                    },
                )
                .await?;
            }
        }
        TcpMessage::ChangePush { changes } => {
            let (applied_ids, mutated) = apply_pushed(db, peer_id, changes)?;
            runtime.emit_sync_applied(mutated);
            write_locked(writer, key, &TcpMessage::ChangeAck { applied_ids }).await?;
            if let Ok(mut inner) = runtime.inner.lock() {
                inner.last_synced_at = now_utc_rfc3339().ok();
                inner.catching_up = false;
                inner.state = "synced".into();
            }
        }
        TcpMessage::SnapshotRow { table, payload } => {
            {
                let guard = db.lock().map_err(|_| AppError::Internal {
                    message: "database lock poisoned".into(),
                })?;
                apply_snapshot_row(guard.conn(), &table, payload)?;
            }
            if let Ok(mut inner) = runtime.inner.lock() {
                *inner.snapshot_rows.entry(peer_id.to_string()).or_insert(0) += 1;
            }
        }
        TcpMessage::SnapshotEnd => {
            let applied = if let Ok(mut inner) = runtime.inner.lock() {
                inner.last_synced_at = now_utc_rfc3339().ok();
                inner.catching_up = false;
                inner.state = "synced".into();
                inner.snapshot_rows.remove(peer_id).unwrap_or(1).max(1)
            } else {
                1
            };
            runtime.emit_sync_applied(applied);
        }
        TcpMessage::SnapshotBegin { .. } | TcpMessage::Grant { .. } => {}
        TcpMessage::BlobHave { hashes } | TcpMessage::SnapshotBlobList { hashes } => {
            let missing = missing_blobs(db, hashes)?;
            if !missing.is_empty() {
                write_locked(
                    writer,
                    key,
                    &TcpMessage::BlobWant {
                        hashes: missing.into_iter().take(BLOB_WANT_MAX).collect(),
                    },
                )
                .await?;
            }
        }
        TcpMessage::BlobWant { hashes } => {
            let mut write_half = writer.lock().await;
            send_blobs(db, &mut *write_half, key, hashes).await?;
        }
        TcpMessage::BlobChunk {
            hash,
            offset,
            data,
            eof,
        } => {
            if store_blob_chunk(db, &hash, offset, &data, eof)? {
                runtime.emit_sync_applied(1);
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_pushed(
    db: &Arc<Mutex<crate::db::Db>>,
    peer_id: &str,
    changes: Vec<ChangeWire>,
) -> Result<(Vec<String>, u64), AppError> {
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let mut applied = Vec::new();
    let mut mutated = 0u64;
    let mut max_hlc: Option<Hlc> = None;
    for wire in changes {
        let change = SyncChange {
            id: wire.id.clone(),
            entity_table: wire.entity_table,
            entity_id: wire.entity_id,
            op: wire.op,
            payload_json: wire.payload_json,
            hlc: wire.hlc.clone(),
            created_at: wire.created_at,
        };
        let outcome = crate::domain::sync::apply_remote_change(guard.conn(), &change)?;
        applied.push(wire.id);
        if outcome == ApplyOutcome::Applied {
            mutated += 1;
        }
        if max_hlc
            .as_ref()
            .map(|h| crate::domain::sync::hlc_greater(&wire.hlc, h))
            .unwrap_or(true)
        {
            max_hlc = Some(wire.hlc);
        }
    }
    if let Some(hlc) = max_hlc {
        set_peer_cursor(guard.conn(), peer_id, &hlc)?;
    }
    Ok((applied, mutated))
}

async fn send_join_snapshot<W>(
    stream: &mut W,
    key: &[u8; 32],
    db: &Arc<Mutex<crate::db::Db>>,
) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let (rows, hashes) = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        let (_schema_version, rows) = dump_snapshot(guard.conn())?;
        let hashes = list_blob_hashes(guard.conn())?;
        (rows, hashes)
    };
    write_msg(
        stream,
        key,
        &TcpMessage::SnapshotBegin {
            schema_version: SCHEMA_VERSION,
            counts: rows.len() as i64,
        },
    )
    .await?;
    for row in rows {
        write_msg(
            stream,
            key,
            &TcpMessage::SnapshotRow {
                table: row.table,
                payload: row.payload,
            },
        )
        .await?;
    }
    if !hashes.is_empty() {
        write_msg(stream, key, &TcpMessage::SnapshotBlobList { hashes }).await?;
    }
    write_msg(stream, key, &TcpMessage::SnapshotEnd).await?;
    Ok(())
}

async fn wait_for_join_blob_want(
    stream: &mut TcpStream,
    key: &[u8; 32],
    db: &Arc<Mutex<crate::db::Db>>,
) -> Result<(), AppError> {
    tokio::select! {
        msg = read_msg(stream, key) => {
            match msg? {
                TcpMessage::BlobWant { hashes } => {
                    send_blobs(db, stream, key, hashes).await?;
                }
                _ => {}
            }
        }
        _ = tokio::time::sleep(Duration::from_secs(10)) => {}
    }
    Ok(())
}

async fn receive_join_snapshot(
    runtime: &SyncRuntime,
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut TcpStream,
    key: &[u8; 32],
    peer_id: &str,
) -> Result<(), AppError> {
    let mut grant_fields: Option<(String, String, String)> = None;
    let mut team_name: Option<String> = None;
    let mut snapshot_applied = false;
    let mut snapshot_rows: u64 = 0;
    let mut listed_hashes: Vec<String> = Vec::new();
    let mut wanted: HashSet<String> = HashSet::new();
    let mut snapshot_done = false;

    loop {
        let msg = if snapshot_done {
            tokio::select! {
                msg = read_msg(stream, key) => msg?,
                _ = tokio::time::sleep(Duration::from_secs(15)) => break,
            }
        } else {
            read_msg(stream, key).await?
        };
        match msg {
            TcpMessage::Grant {
                team_id,
                team_psk_hex,
                device_code,
            } => {
                grant_fields = Some((team_id, team_psk_hex, device_code));
            }
            TcpMessage::SnapshotRow { table, payload } => {
                if table == "teams" {
                    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
                        if !name.is_empty() {
                            team_name = Some(name.to_string());
                        }
                    }
                }
                {
                    let guard = db.lock().map_err(|_| AppError::Internal {
                        message: "database lock poisoned".into(),
                    })?;
                    apply_snapshot_row(guard.conn(), &table, payload)?;
                }
                snapshot_rows += 1;
                snapshot_applied = true;
            }
            TcpMessage::ChangePush { changes } => {
                for wire in &changes {
                    if wire.entity_table == "teams" {
                        if let Ok(value) =
                            serde_json::from_str::<serde_json::Value>(&wire.payload_json)
                        {
                            if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
                                if !name.is_empty() {
                                    team_name = Some(name.to_string());
                                }
                            }
                        }
                    }
                }
                let (_applied_ids, mutated) = apply_pushed(db, peer_id, changes)?;
                runtime.emit_sync_applied(mutated);
                snapshot_applied = true;
            }
            TcpMessage::SnapshotBlobList { hashes } | TcpMessage::BlobHave { hashes } => {
                listed_hashes = hashes;
            }
            TcpMessage::SnapshotEnd => {
                snapshot_done = true;
                let mut candidates = listed_hashes.clone();
                if candidates.is_empty() {
                    let guard = db.lock().map_err(|_| AppError::Internal {
                        message: "database lock poisoned".into(),
                    })?;
                    candidates = list_blob_hashes(guard.conn())?;
                }
                let missing = missing_blobs(db, candidates)?;
                wanted.extend(missing.iter().cloned());
                write_msg(
                    stream,
                    key,
                    &TcpMessage::BlobWant {
                        hashes: missing.into_iter().take(BLOB_WANT_MAX).collect(),
                    },
                )
                .await?;
                if wanted.is_empty() {
                    break;
                }
            }
            TcpMessage::BlobChunk {
                hash,
                offset,
                data,
                eof,
            } => {
                if store_blob_chunk(db, &hash, offset, &data, eof)? {
                    runtime.emit_sync_applied(1);
                }
                if eof {
                    wanted.remove(&hash);
                    if snapshot_done && wanted.is_empty() {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    if let Some((team_id, team_psk_hex, device_code)) = grant_fields {
        if let Ok(mut inner) = runtime.inner.lock() {
            inner.join_grant = Some(JoinGrant {
                team_id,
                team_name: team_name.unwrap_or_default(),
                team_psk_hex,
                device_code,
                snapshot_applied,
            });
        }
    }
    runtime.emit_sync_applied(snapshot_rows);
    Ok(())
}

fn missing_blobs(
    db: &Arc<Mutex<crate::db::Db>>,
    hashes: Vec<String>,
) -> Result<Vec<String>, AppError> {
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let mut missing = Vec::new();
    for hash in hashes {
        if !crate::domain::sync::blobs::blob_exists(guard.conn(), &hash)? {
            missing.push(hash);
        }
    }
    Ok(missing)
}

async fn send_blobs<W>(
    db: &Arc<Mutex<crate::db::Db>>,
    stream: &mut W,
    key: &[u8; 32],
    hashes: Vec<String>,
) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let root = {
        let guard = db.lock().map_err(|_| AppError::Internal {
            message: "database lock poisoned".into(),
        })?;
        guard.paths().root.clone()
    };
    for hash in hashes.into_iter().take(BLOB_WANT_MAX) {
        let path = crate::domain::sync::blobs::blob_absolute(&root, &hash);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        for (i, chunk) in bytes.chunks(BLOB_CHUNK_MAX).enumerate() {
            let offset = (i * BLOB_CHUNK_MAX) as u64;
            let eof = offset as usize + chunk.len() >= bytes.len();
            write_msg(
                stream,
                key,
                &TcpMessage::BlobChunk {
                    hash: hash.clone(),
                    offset,
                    data: hex::encode(chunk),
                    eof,
                },
            )
            .await?;
        }
    }
    Ok(())
}

fn store_blob_chunk(
    db: &Arc<Mutex<crate::db::Db>>,
    hash: &str,
    offset: u64,
    data_hex: &str,
    eof: bool,
) -> Result<bool, AppError> {
    let chunk = hex::decode(data_hex).map_err(|_| AppError::sync_err("Blob data was invalid."))?;
    let guard = db.lock().map_err(|_| AppError::Internal {
        message: "database lock poisoned".into(),
    })?;
    let complete = crate::domain::sync::blobs::accept_blob_chunk(
        &guard.paths().root,
        hash,
        offset,
        &chunk,
        eof,
    )?;
    if let Some(bytes) = complete {
        crate::domain::sync::blobs::finalize_received_blob(
            &guard.paths().root,
            guard.conn(),
            hash,
            &bytes,
        )?;
        return Ok(true);
    }
    Ok(false)
}

fn to_wire(change: SyncChange) -> ChangeWire {
    ChangeWire {
        id: change.id,
        entity_table: change.entity_table,
        entity_id: change.entity_id,
        op: change.op,
        payload_json: change.payload_json,
        hlc: change.hlc,
        created_at: change.created_at,
    }
}

async fn write_msg<W>(stream: &mut W, key: &[u8; 32], msg: &TcpMessage) -> Result<(), AppError>
where
    W: AsyncWriteExt + Unpin,
{
    let json = serde_json::to_vec(msg).map_err(|err| AppError::Internal {
        message: format!("encode tcp message: {err}"),
    })?;
    let frame = encode_frame(key, &json)?;
    stream.write_all(&frame).await?;
    Ok(())
}

async fn read_msg<R>(stream: &mut R, key: &[u8; 32]) -> Result<TcpMessage, AppError>
where
    R: AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > crate::sync_net::frame::MAX_PLAINTEXT + 64 {
        return Err(AppError::sync_err("Frame is too large."));
    }
    let mut rest = vec![0u8; len];
    stream.read_exact(&mut rest).await?;
    let mut frame = Vec::with_capacity(4 + len);
    frame.extend_from_slice(&len_buf);
    frame.extend_from_slice(&rest);
    let plain = decode_frame(key, &frame)?;
    serde_json::from_slice(&plain).map_err(|_| AppError::sync_err("Peer sent an invalid message."))
}

#[cfg(test)]
mod tests {
    use super::{is_default_empty_cursor, should_dial_on_hello, use_join_key};
    use crate::domain::sync::hlc::Hlc;

    #[test]
    fn hello_always_dials_regardless_of_device_id_order() {
        assert!(should_dial_on_hello("aaa", "zzz"));
        assert!(should_dial_on_hello("zzz", "aaa"));
        assert!(should_dial_on_hello("same", "same"));
    }

    #[test]
    fn default_peer_cursor_is_empty_wall_and_counter() {
        assert!(is_default_empty_cursor(&Hlc::new(0, 0, String::new())));
        assert!(is_default_empty_cursor(&Hlc::new(0, 0, "device-1")));
        assert!(!is_default_empty_cursor(&Hlc::new(1, 0, String::new())));
        assert!(!is_default_empty_cursor(&Hlc::new(0, 1, String::new())));
    }

    #[test]
    fn joiner_not_yet_in_team_uses_pending_pin() {
        assert!(use_join_key(None, Some("123456"), None));
    }

    #[test]
    fn leftover_pending_after_join_uses_team_psk() {
        assert!(!use_join_key(None, Some("123456"), Some("team-1")));
    }

    #[test]
    fn host_session_join_pin_uses_join_key() {
        assert!(use_join_key(Some("123456"), None, Some("team-1")));
    }

    #[test]
    fn neither_pin_uses_team_psk() {
        assert!(!use_join_key(None, None, Some("team-1")));
        assert!(!use_join_key(None, None, None));
    }
}
