mod appearance;
mod auto_backup;
mod shop_sync;

use crate::domain::settings::repository;

pub use appearance::{
    detect_system_locale_tag, get_locale_settings, get_theme_settings, map_tag_to_catalog,
    resolve_catalog, set_locale_preference, set_theme_preference,
};
pub use auto_backup::{get_auto_backup_settings, set_auto_backup_settings};
pub use shop_sync::{get_shop_settings, get_sync_interval, set_shop_settings, set_sync_interval};

#[cfg(test)]
mod tests;
