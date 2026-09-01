//! Update-check and external-link constants (not shared with backup).

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const LATEST_JSON_URL: &str = "https://korigio.github.io/korigio-downloads/latest.json";
pub const LATEST_JSON_HOST: &str = "korigio.github.io";
pub const LATEST_JSON_PATH: &str = "/korigio-downloads/latest.json";

pub const USER_AGENT_PRODUCT: &str = "Korigio";
pub const FETCH_TIMEOUT_SECS: u64 = 8;

pub const FEEDBACK_MAILTO_ADDRESS: &str = "info@korigio.com";
pub const ALLOWED_GITHUB_HOST: &str = "github.com";
pub const ALLOWED_GITHUB_PATH_PREFIX: &str = "/Korigio/korigio-downloads";
pub const ALLOWED_PAGES_HOST: &str = "korigio.github.io";
pub const ALLOWED_PAGES_PATH_PREFIX: &str = "/korigio-downloads";

pub const OPEN_URL_REJECTED: &str = "This link cannot be opened.";
