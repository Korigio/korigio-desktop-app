pub const REPORTED_PROBLEM_MAX_LEN: usize = 500;
pub const ACCESSORIES_RECEIVED_MAX_LEN: usize = 500;
pub const DEVICE_CONDITION_MAX_LEN: usize = 500;
pub const DIAGNOSIS_NOTES_MAX_LEN: usize = 2000;
pub const WORK_PERFORMED_MAX_LEN: usize = 2000;
pub const NOTES_MAX_LEN: usize = 2000;

pub const DEFAULT_PAGE_SIZE: u32 = 25;
pub const MAX_PAGE_SIZE: u32 = 100;

pub const DEFAULT_STATUS: &str = "received";

pub const REPAIR_STATUSES: &[&str] = &[
    "received",
    "diagnosis",
    "waiting_customer",
    "waiting_part",
    "in_repair",
    "ready",
    "collected",
    "cancelled",
];
