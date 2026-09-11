pub const REPORTED_PROBLEM_MAX_LEN: usize = 500;
pub const ACCESSORIES_RECEIVED_MAX_LEN: usize = 500;
pub const DEVICE_CONDITION_MAX_LEN: usize = 500;
pub const DIAGNOSIS_NOTES_MAX_LEN: usize = 2000;
pub const WORK_PERFORMED_MAX_LEN: usize = 2000;
pub const NOTES_MAX_LEN: usize = 2000;

pub const DEFAULT_PAGE_SIZE: u32 = 10;
pub const MAX_PAGE_SIZE: u32 = 100;

pub const DEFAULT_STATUS: &str = "received";

/// Inclusive max for `warranty_years` at summary handover.
pub const WARRANTY_YEARS_MAX: i64 = 10;

pub const ALLOWED_REPAIR_DOCUMENT_EXTENSIONS: &[&str] = &["pdf", "jpg", "jpeg", "png"];

/// Signed document uploads (PDF or image scan).
pub const MAX_REPAIR_DOCUMENT_BYTES: u64 = 20 * 1024 * 1024;

pub const REPAIR_STATUSES: &[&str] = &[
    "received",
    "diagnosis",
    "waiting_customer",
    "waiting_part",
    "in_repair",
    "ready",
    "awaiting_pickup",
    "collected",
    "cancelled",
];
