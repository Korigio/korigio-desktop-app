use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::StatusCode;

use crate::domain::updates::constants::{
    APP_VERSION, FETCH_TIMEOUT_SECS, LATEST_JSON_URL, USER_AGENT_PRODUCT,
};
use crate::domain::updates::types::{AppVersion, LatestManifest, OsAsset, UpdateCheck};
use crate::domain::updates::validation::{
    is_allowed_latest_json_url, parse_semver, sanitize_download_url,
};
use crate::error::AppError;

/// Injected so `cargo test` never performs a real HTTP request.
pub trait LatestFetcher {
    fn fetch_latest_json(&self) -> Result<String, AppError>;
}

/// Production fetcher: GET only the allowlisted Pages `latest.json`.
pub struct ReqwestLatestFetcher;

impl LatestFetcher for ReqwestLatestFetcher {
    fn fetch_latest_json(&self) -> Result<String, AppError> {
        fetch_latest_json()
    }
}

pub fn app_version() -> AppVersion {
    AppVersion {
        version: APP_VERSION.to_string(),
    }
}

pub fn check_app_update<F: LatestFetcher>(fetcher: &F) -> Result<UpdateCheck, AppError> {
    check_app_update_for(fetcher, APP_VERSION, OsAsset::current())
}

pub fn check_app_update_for<F: LatestFetcher>(
    fetcher: &F,
    current_version: &str,
    os: OsAsset,
) -> Result<UpdateCheck, AppError> {
    let body = fetcher.fetch_latest_json()?;
    let manifest = parse_latest_json(&body)?;
    let latest_version = manifest.version.trim().to_string();
    let remote =
        parse_semver(&latest_version).ok_or_else(|| AppError::network("missing_version"))?;
    let current =
        parse_semver(current_version).ok_or_else(|| AppError::network("missing_version"))?;

    Ok(UpdateCheck {
        current_version: current_version.to_string(),
        latest_version,
        update_available: remote > current,
        download_url: sanitize_download_url(os.pick(&manifest)),
    })
}

pub fn parse_latest_json(body: &str) -> Result<LatestManifest, AppError> {
    let manifest: LatestManifest =
        serde_json::from_str(body).map_err(|_| AppError::network("bad_json"))?;
    if manifest.version.trim().is_empty() {
        return Err(AppError::network("missing_version"));
    }
    Ok(manifest)
}

fn fetch_latest_json() -> Result<String, AppError> {
    if !is_allowed_latest_json_url(LATEST_JSON_URL) {
        return Err(AppError::network("url_not_allowed"));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .redirect(Policy::none())
        .user_agent(format!("{USER_AGENT_PRODUCT}/{APP_VERSION}"))
        .build()
        .map_err(|_| AppError::network("client"))?;

    let response = client
        .get(LATEST_JSON_URL)
        .send()
        .map_err(map_reqwest_error)?;

    if response.status() != StatusCode::OK {
        return Err(AppError::network("http_status"));
    }

    response.text().map_err(|_| AppError::network("body"))
}

fn map_reqwest_error(err: reqwest::Error) -> AppError {
    if err.is_timeout() {
        AppError::network("timeout")
    } else if err.is_connect() {
        AppError::network("connect")
    } else {
        AppError::network("transport")
    }
}
