use rusqlite::Connection;

use crate::domain::settings::repository;
use crate::domain::settings::types::{
    ShopSettings, ShopSettingsInput, SyncIntervalSettings, CURRENCY_KEY, DEFAULT_CURRENCY,
    DEFAULT_SYNC_INTERVAL_SECS, DEFAULT_TAX_RATE_PERCENT, SHOP_SETTINGS_ID, SYNC_INTERVAL_KEY,
    TAX_RATE_PERCENT_KEY,
};
use crate::domain::settings::validation::{
    normalize_currency, parse_sync_interval_secs, parse_tax_rate_percent,
    validate_sync_interval_secs,
};
use crate::domain::sync::{self, begin_write};
use crate::error::AppError;

pub fn get_shop_settings(conn: &Connection) -> Result<ShopSettings, AppError> {
    let tax_rate_percent = match repository::get_setting(conn, TAX_RATE_PERCENT_KEY)? {
        Some(raw) => parse_tax_rate_percent(&raw)
            .map(|(normalized, _)| normalized)
            .unwrap_or_else(|_| DEFAULT_TAX_RATE_PERCENT.to_string()),
        None => DEFAULT_TAX_RATE_PERCENT.to_string(),
    };
    let currency = match repository::get_setting(conn, CURRENCY_KEY)? {
        Some(raw) => normalize_currency(&raw).unwrap_or_else(|_| DEFAULT_CURRENCY.to_string()),
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
        repository::upsert_setting(conn, CURRENCY_KEY, &normalize_currency(raw)?)?;
    }
    let settings = get_shop_settings(conn)?;
    let ctx = begin_write(conn)?;
    sync::record_upsert(
        conn,
        "shop_settings",
        SHOP_SETTINGS_ID,
        serde_json::json!({
            "id": SHOP_SETTINGS_ID,
            "taxRatePercent": settings.tax_rate_percent,
            "currency": settings.currency,
        }),
        &ctx,
    )?;
    Ok(settings)
}

pub fn get_sync_interval(conn: &Connection) -> Result<SyncIntervalSettings, AppError> {
    let interval_seconds = match repository::get_setting(conn, SYNC_INTERVAL_KEY)? {
        Some(raw) => parse_sync_interval_secs(&raw).unwrap_or(DEFAULT_SYNC_INTERVAL_SECS),
        None => DEFAULT_SYNC_INTERVAL_SECS,
    };
    Ok(SyncIntervalSettings { interval_seconds })
}

pub fn set_sync_interval(
    conn: &Connection,
    interval_seconds: u64,
) -> Result<SyncIntervalSettings, AppError> {
    let interval_seconds = validate_sync_interval_secs(interval_seconds)?;
    repository::upsert_setting(conn, SYNC_INTERVAL_KEY, &interval_seconds.to_string())?;
    get_sync_interval(conn)
}
