use rusqlite::Connection;

use crate::domain::settings::repository;
use crate::domain::settings::types::{
    LocalePreference, LocaleSettings, ThemePreference, ThemeSettings,
};
use crate::error::AppError;

/// Best-effort OS locale tag (e.g. `de-DE`). Falls back to `es` when unknown.
pub fn detect_system_locale_tag() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "es".into())
}

pub fn map_tag_to_catalog(tag: &str) -> &'static str {
    let lower = tag.trim().to_ascii_lowercase();
    if lower.starts_with("de") {
        "de"
    } else if lower.starts_with("en") {
        "en"
    } else if lower.starts_with("es") {
        "es"
    } else {
        "es"
    }
}

pub fn resolve_catalog(preference: LocalePreference, system_tag: &str) -> &'static str {
    match preference {
        LocalePreference::System => map_tag_to_catalog(system_tag),
        LocalePreference::En => "en",
        LocalePreference::Es => "es",
        LocalePreference::De => "de",
    }
}

pub fn get_locale_settings(conn: &Connection) -> Result<LocaleSettings, AppError> {
    let system_locale = detect_system_locale_tag();
    let preference = match repository::get_setting(conn, LocalePreference::STORAGE_KEY)? {
        Some(raw) => LocalePreference::parse(&raw).unwrap_or(LocalePreference::System),
        None => LocalePreference::System,
    };
    let resolved_locale = resolve_catalog(preference, &system_locale).to_string();
    Ok(LocaleSettings {
        preference,
        system_locale,
        resolved_locale,
    })
}

pub fn set_locale_preference(
    conn: &Connection,
    preference: LocalePreference,
) -> Result<LocaleSettings, AppError> {
    repository::upsert_setting(
        conn,
        LocalePreference::STORAGE_KEY,
        preference.as_storage_value(),
    )?;
    get_locale_settings(conn)
}

pub fn get_theme_settings(conn: &Connection) -> Result<ThemeSettings, AppError> {
    let preference = match repository::get_setting(conn, ThemePreference::STORAGE_KEY)? {
        Some(raw) => ThemePreference::parse(&raw).unwrap_or(ThemePreference::System),
        None => ThemePreference::System,
    };
    Ok(ThemeSettings { preference })
}

pub fn set_theme_preference(
    conn: &Connection,
    preference: ThemePreference,
) -> Result<ThemeSettings, AppError> {
    repository::upsert_setting(
        conn,
        ThemePreference::STORAGE_KEY,
        preference.as_storage_value(),
    )?;
    get_theme_settings(conn)
}
