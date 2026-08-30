//! LAN mesh: UDP discovery + PSK-encrypted TCP gossip.
//!
//! Unit tests cover HLC apply (domain/sync) and frame encode/decode.
//! Full UDP/TCP bind + join snapshot is a manual smoke test (ports 47821/47822).

pub mod crypto;
pub mod frame;
pub mod protocol;
pub mod runtime;

pub use runtime::SyncRuntime;
