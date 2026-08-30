use serde::{Deserialize, Serialize};

/// Stored preference: follow OS or force a catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LocalePreference {
    System,
    En,
    Es,
    De,
}

impl LocalePreference {
    pub const STORAGE_KEY: &'static str = "locale_preference";

    pub fn as_storage_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::En => "en",
            Self::Es => "es",
            Self::De => "de",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "system" | "" => Some(Self::System),
            "en" | "en-us" | "en-gb" => Some(Self::En),
            "es" | "es-es" | "es-mx" => Some(Self::Es),
            "de" | "de-de" | "de-at" | "de-ch" => Some(Self::De),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocaleSettings {
    /// User preference (`system` or a fixed locale).
    pub preference: LocalePreference,
    /// OS language tag (best effort), e.g. `de-DE`.
    pub system_locale: String,
    /// Catalog actually used: `en` | `es` | `de`.
    pub resolved_locale: String,
}

pub const TAX_RATE_PERCENT_KEY: &str = "tax_rate_percent";
pub const CURRENCY_KEY: &str = "currency";
pub const DEFAULT_TAX_RATE_PERCENT: &str = "19";
pub const DEFAULT_CURRENCY: &str = "EUR";
pub const SHOP_SETTINGS_ID: &str = "shop";
pub const SYNC_INTERVAL_KEY: &str = "sync_interval_secs";
pub const DEFAULT_SYNC_INTERVAL_SECS: u64 = 5;
pub const MIN_SYNC_INTERVAL_SECS: u64 = 2;
pub const MAX_SYNC_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncIntervalSettings {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShopSettings {
    pub tax_rate_percent: String,
    pub currency: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopSettingsInput {
    pub tax_rate_percent: Option<String>,
    pub currency: Option<String>,
}
