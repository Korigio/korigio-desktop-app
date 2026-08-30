pub mod apply;
pub mod blobs;
pub mod hlc;
pub mod record;
pub mod snapshot;

#[cfg(test)]
mod tests;

pub use apply::{apply_remote_change, apply_snapshot_row};
pub use hlc::{hlc_equal, hlc_greater, tick_hlc, tick_hlc_value, Hlc};
pub use record::{begin_write, record_delete, record_upsert, SyncChange, WriteContext};

use std::sync::OnceLock;

use rusqlite::Connection;
use tokio::sync::Notify;

use crate::error::AppError;

fn local_change() -> &'static Notify {
    static NOTIFY: OnceLock<Notify> = OnceLock::new();
    NOTIFY.get_or_init(Notify::new)
}

/// Wake gossip sessions after a successful local replicated write.
pub fn notify_local_change() {
    local_change().notify_waiters();
}

/// Completes when [`notify_local_change`] runs. Recreate after each wait.
pub fn local_change_notified() -> impl std::future::Future<Output = ()> {
    local_change().notified()
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PresenceItem {
    pub device_id: String,
    pub device_name: String,
    pub device_code: String,
    pub is_this_device: bool,
    pub online: bool,
    pub last_seen_at: Option<String>,
    pub staff: Option<PresenceStaff>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PresenceStaff {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PresenceListResult {
    pub items: Vec<PresenceItem>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub team_id: Option<String>,
    pub state: String,
    pub last_synced_at: Option<String>,
    pub peers_online: i64,
    pub pending_outgoing: i64,
    pub pending_incoming: i64,
    pub error_message: Option<String>,
}

pub fn idle_sync_status() -> SyncStatus {
    SyncStatus {
        team_id: None,
        state: "idle".into(),
        last_synced_at: None,
        peers_online: 0,
        pending_outgoing: 0,
        pending_incoming: 0,
        error_message: None,
    }
}

pub fn count_pending_outgoing(conn: &Connection) -> Result<i64, AppError> {
    let count = conn.query_row("SELECT COUNT(*) FROM sync_changes", [], |row| row.get(0))?;
    Ok(count)
}
