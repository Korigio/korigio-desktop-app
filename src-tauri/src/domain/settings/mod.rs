//! App settings (key/value in SQLite `settings` table).

pub mod repository;
pub mod service;
pub mod types;
pub mod validation;

pub use service::{
    get_locale_settings, get_shop_settings, set_locale_preference, set_shop_settings,
};
pub use types::{LocalePreference, LocaleSettings, ShopSettings, ShopSettingsInput};
pub use validation::parse_tax_rate_percent;
