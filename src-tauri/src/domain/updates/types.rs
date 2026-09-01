use serde::{Deserialize, Serialize};

/// Desktop app version for Settings (`get_app_version`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppVersion {
    pub version: String,
}

/// Result of comparing the running build to `latest.json`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
}

/// Public download manifest baked next to the Pages `index.html`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LatestManifest {
    pub version: String,
    #[serde(default)]
    pub windows: Option<String>,
    #[serde(default)]
    pub macos: Option<String>,
    #[serde(default)]
    pub linux: Option<String>,
}

/// Numeric `major.minor.patch` used for update comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semver {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// Which `latest.json` asset key to expose to the running OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsAsset {
    Windows,
    Macos,
    Linux,
}

impl OsAsset {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Linux
        }
    }

    pub fn pick<'a>(self, manifest: &'a LatestManifest) -> Option<&'a str> {
        match self {
            Self::Windows => manifest.windows.as_deref(),
            Self::Macos => manifest.macos.as_deref(),
            Self::Linux => manifest.linux.as_deref(),
        }
    }
}
