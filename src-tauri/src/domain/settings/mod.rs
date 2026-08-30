//! App settings (key/value in SQLite `settings` table).

pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

pub use service::{
    get_auto_backup_settings, get_locale_settings, get_shop_settings, get_sync_interval,
    get_theme_settings, set_auto_backup_settings, set_locale_preference, set_shop_settings,
    set_sync_interval, set_theme_preference,
};
pub use types::{
    AutoBackupInterval, AutoBackupSettings, LocalePreference, LocaleSettings,
    SetAutoBackupSettingsInput, ShopSettings, ShopSettingsInput, SyncIntervalSettings,
    ThemePreference, ThemeSettings,
};
pub use validation::parse_tax_rate_percent;
