use rusqlite::Connection;

use crate::domain::settings::repository;
use crate::domain::settings::types::{
    CURRENCY_KEY, DEFAULT_CURRENCY, DEFAULT_TAX_RATE_PERCENT, LocalePreference, LocaleSettings,
    ShopSettings, ShopSettingsInput, TAX_RATE_PERCENT_KEY,
};
use crate::domain::settings::validation::{normalize_currency, parse_tax_rate_percent};
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

pub fn get_shop_settings(conn: &Connection) -> Result<ShopSettings, AppError> {
    let tax_rate_percent = match repository::get_setting(conn, TAX_RATE_PERCENT_KEY)? {
        Some(raw) => match parse_tax_rate_percent(&raw) {
            Ok((normalized, _)) => normalized,
            Err(_) => DEFAULT_TAX_RATE_PERCENT.to_string(),
        },
        None => DEFAULT_TAX_RATE_PERCENT.to_string(),
    };
    let currency = match repository::get_setting(conn, CURRENCY_KEY)? {
        Some(raw) => match normalize_currency(&raw) {
            Ok(code) => code,
            Err(_) => DEFAULT_CURRENCY.to_string(),
        },
        None => DEFAULT_CURRENCY.to_string(),
    };
    Ok(ShopSettings {
        tax_rate_percent,
        currency,
    })
}

pub fn set_shop_settings(
    conn: &Connection,
    input: ShopSettingsInput,
) -> Result<ShopSettings, AppError> {
    if input.tax_rate_percent.is_none() && input.currency.is_none() {
        return get_shop_settings(conn);
    }

    if let Some(raw) = &input.tax_rate_percent {
        let (normalized, _) = parse_tax_rate_percent(raw)?;
        repository::upsert_setting(conn, TAX_RATE_PERCENT_KEY, &normalized)?;
    }
    if let Some(raw) = &input.currency {
        let code = normalize_currency(raw)?;
        repository::upsert_setting(conn, CURRENCY_KEY, &code)?;
    }
    get_shop_settings(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

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

    #[test]
    fn shop_settings_defaults() {
        let db = Db::open_in_memory().expect("db");
        let settings = get_shop_settings(db.conn()).expect("get");
        assert_eq!(settings.tax_rate_percent, "19");
        assert_eq!(settings.currency, "EUR");
    }

    #[test]
    fn set_shop_settings_roundtrip_and_validation() {
        let db = Db::open_in_memory().expect("db");
        let updated = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: Some("19.5".into()),
                currency: Some("usd".into()),
            },
        )
        .expect("set");
        assert_eq!(updated.tax_rate_percent, "19.5");
        assert_eq!(updated.currency, "USD");

        let err = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: Some("101".into()),
                currency: None,
            },
        )
        .expect_err("invalid tax");
        assert!(matches!(err, AppError::Validation { .. }));

        let err = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: None,
                currency: Some("EURO".into()),
            },
        )
        .expect_err("invalid currency");
        assert!(matches!(err, AppError::Validation { .. }));
    }
}
