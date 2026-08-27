//! App settings (key/value in SQLite `settings` table).

pub mod repository;
pub mod service;
pub mod types;

pub use service::{get_locale_settings, set_locale_preference};
pub use types::{LocalePreference, LocaleSettings};
