pub const LEGAL_NAME_MAX_LEN: usize = 200;
pub const TRADE_NAME_MAX_LEN: usize = 200;
pub const TAX_ID_MAX_LEN: usize = 64;
pub const ADDRESS_MAX_LEN: usize = 500;
pub const PHONE_MAX_LEN: usize = 50;
pub const EMAIL_MAX_LEN: usize = 254;
pub const WEBSITE_MAX_LEN: usize = 500;

pub const DEFAULT_PAGE_SIZE: u32 = 10;
pub const MAX_PAGE_SIZE: u32 = 100;

/// Same limit as repair images (~10 MB).
pub const MAX_LOGO_BYTES: u64 = 10 * 1024 * 1024;
pub const ALLOWED_LOGO_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];
