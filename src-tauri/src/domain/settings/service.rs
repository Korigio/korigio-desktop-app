use rusqlite::Connection;

use crate::domain::settings::repository;
use crate::domain::settings::types::{LocalePreference, LocaleSettings};
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
        // Unsupported OS languages → Spanish product default
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_german_tags() {
        assert_eq!(map_tag_to_catalog("de-DE"), "de");
        assert_eq!(map_tag_to_catalog("de"), "de");
    }

    #[test]
    fn unknown_falls_back_to_spanish() {
        assert_eq!(map_tag_to_catalog("fr-FR"), "es");
    }

    #[test]
    fn system_preference_uses_os() {
        assert_eq!(
            resolve_catalog(LocalePreference::System, "de-CH"),
            "de"
        );
        assert_eq!(resolve_catalog(LocalePreference::En, "de-CH"), "en");
    }
}
