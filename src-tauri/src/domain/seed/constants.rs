//! Synthetic seed limits.
//!
//! Command defaults are intentionally **small** (safe for accidental clicks).
//! Full performance measurement should pass explicit counts, e.g.:
//! `customers: 10000`, `devices: 20000`, `repairs: 50000`.

/// Default customer count when omitted from input.
pub const DEFAULT_CUSTOMERS: u32 = 10;
/// Default device count when omitted from input.
pub const DEFAULT_DEVICES: u32 = 20;
/// Default repair count when omitted from input.
pub const DEFAULT_REPAIRS: u32 = 50;

/// Hard cap — prevents accidental disk fill.
pub const MAX_CUSTOMERS: u32 = 20_000;
/// Hard cap — prevents accidental disk fill.
pub const MAX_DEVICES: u32 = 40_000;
/// Hard cap — prevents accidental disk fill.
pub const MAX_REPAIRS: u32 = 100_000;
