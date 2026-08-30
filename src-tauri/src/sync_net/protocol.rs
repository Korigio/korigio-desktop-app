use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: i64 = crate::domain::sync::snapshot::SNAPSHOT_SCHEMA_VERSION;
pub const UDP_PORT: u16 = 47821;
pub const TCP_PORT: u16 = 47822;
pub const UDP_MAX_BYTES: usize = 512;
pub const HEARTBEAT_SECS: u64 = 5;
pub const ONLINE_WINDOW_SECS: u64 = 15;
pub const CHANGE_PUSH_MAX: usize = 100;
pub const BLOB_WANT_MAX: usize = 32;
pub const BLOB_CHUNK_MAX: usize = 256 * 1024;
pub const SYNC_APPLIED_EVENT: &str = "sync-applied";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncAppliedPayload {
    pub at: String,
    pub applied: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UdpBeacon {
    pub v: u8,
    pub kind: String,
    pub team_id: String,
    #[serde(default)]
    pub team_name: String,
    pub device_id: String,
    pub tcp_port: u16,
    pub pin_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TcpMessage {
    Hello {
        proto: String,
        device_id: String,
        nonce: String,
    },
    Auth {
        mac: String,
    },
    Grant {
        team_id: String,
        team_psk_hex: String,
        device_code: String,
    },
    Heartbeat {
        staff_id: Option<String>,
        device_name: String,
        hlc_max: crate::domain::sync::Hlc,
    },
    ChangePull {
        after: crate::domain::sync::Hlc,
    },
    ChangePush {
        changes: Vec<ChangeWire>,
    },
    ChangeAck {
        applied_ids: Vec<String>,
    },
    SnapshotBegin {
        schema_version: i64,
        counts: i64,
    },
    SnapshotRow {
        table: String,
        payload: serde_json::Value,
    },
    SnapshotBlobList {
        hashes: Vec<String>,
    },
    SnapshotEnd,
    BlobHave {
        hashes: Vec<String>,
    },
    BlobWant {
        hashes: Vec<String>,
    },
    BlobChunk {
        hash: String,
        offset: u64,
        data: String,
        eof: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeWire {
    pub id: String,
    pub entity_table: String,
    pub entity_id: String,
    pub op: String,
    pub payload_json: String,
    pub hlc: crate::domain::sync::Hlc,
    pub created_at: String,
}
