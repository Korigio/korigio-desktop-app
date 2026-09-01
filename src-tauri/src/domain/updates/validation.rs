use reqwest::Url;

use crate::domain::updates::constants::{
    ALLOWED_GITHUB_HOST, ALLOWED_GITHUB_PATH_PREFIX, ALLOWED_PAGES_HOST, ALLOWED_PAGES_PATH_PREFIX,
    FEEDBACK_MAILTO_ADDRESS, LATEST_JSON_HOST, LATEST_JSON_PATH, OPEN_URL_REJECTED,
};
use crate::domain::updates::types::Semver;
use crate::error::AppError;

pub fn parse_semver(raw: &str) -> Option<Semver> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('.');
    let major = parse_numeric_component(parts.next()?)?;
    let minor = parse_numeric_component(parts.next()?)?;
    let patch = parse_numeric_component(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some(Semver {
        major,
        minor,
        patch,
    })
}

fn parse_numeric_component(raw: &str) -> Option<u64> {
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok()
}

pub fn is_allowed_latest_json_url(raw: &str) -> bool {
    let Ok(parsed) = Url::parse(raw) else {
        return false;
    };
    if !has_no_userinfo(&parsed) || parsed.port().is_some() || parsed.query().is_some() {
        return false;
    }
    parsed.scheme() == "https"
        && parsed.host_str() == Some(LATEST_JSON_HOST)
        && parsed.path() == LATEST_JSON_PATH
}

pub fn is_allowed_open_url(raw: &str) -> bool {
    let Ok(parsed) = Url::parse(raw.trim()) else {
        return false;
    };
    if !has_no_userinfo(&parsed) {
        return false;
    }
    match parsed.scheme() {
        "mailto" => parsed.path() == FEEDBACK_MAILTO_ADDRESS,
        "https" => is_allowed_https_open(&parsed),
        _ => false,
    }
}

pub fn validate_open_url(raw: &str) -> Result<(), AppError> {
    if is_allowed_open_url(raw) {
        Ok(())
    } else {
        eprintln!("app_error code=validation reason=url_not_allowed");
        Err(AppError::Validation {
            field: None,
            message: OPEN_URL_REJECTED.into(),
        })
    }
}

pub fn sanitize_download_url(raw: Option<&str>) -> Option<String> {
    raw.filter(|candidate| is_allowed_open_url(candidate))
        .map(str::to_string)
}

fn is_allowed_https_open(parsed: &Url) -> bool {
    if parsed.port().is_some() {
        return false;
    }
    match parsed.host_str() {
        Some(host) if host == ALLOWED_GITHUB_HOST => {
            path_allowed(parsed.path(), ALLOWED_GITHUB_PATH_PREFIX)
        }
        Some(host) if host == ALLOWED_PAGES_HOST => {
            path_allowed(parsed.path(), ALLOWED_PAGES_PATH_PREFIX)
        }
        _ => false,
    }
}

fn path_allowed(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

fn has_no_userinfo(parsed: &Url) -> bool {
    parsed.username().is_empty() && parsed.password().is_none()
}
