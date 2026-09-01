use crate::domain::updates::constants::{APP_VERSION, LATEST_JSON_URL, OPEN_URL_REJECTED};
use crate::domain::updates::service::{
    app_version, check_app_update_for, parse_latest_json, LatestFetcher,
};
use crate::domain::updates::types::{OsAsset, Semver, UpdateCheck};
use crate::domain::updates::validation::{
    is_allowed_latest_json_url, is_allowed_open_url, parse_semver, sanitize_download_url,
    validate_open_url,
};
use crate::error::AppError;

struct MockFetcher {
    body: Result<String, &'static str>,
}

impl LatestFetcher for MockFetcher {
    fn fetch_latest_json(&self) -> Result<String, AppError> {
        self.body
            .clone()
            .map_err(|reason| AppError::network(reason))
    }
}

fn semver(major: u64, minor: u64, patch: u64) -> Semver {
    Semver {
        major,
        minor,
        patch,
    }
}

#[test]
fn app_version_uses_cargo_pkg_version() {
    assert_eq!(app_version().version, env!("CARGO_PKG_VERSION"));
    assert_eq!(APP_VERSION, env!("CARGO_PKG_VERSION"));
}

#[test]
fn semver_compares_patch() {
    assert!(parse_semver("1.0.1").unwrap() < parse_semver("1.0.2").unwrap());
}

#[test]
fn semver_equal() {
    assert_eq!(
        parse_semver("1.0.2").unwrap(),
        parse_semver("1.0.2").unwrap()
    );
}

#[test]
fn semver_compares_minor_numerically() {
    assert!(parse_semver("1.9.0").unwrap() < parse_semver("1.10.0").unwrap());
}

#[test]
fn semver_compares_major() {
    assert!(parse_semver("2.0.0").unwrap() > parse_semver("1.9.9").unwrap());
}

#[test]
fn semver_tuple_matches_expected_components() {
    assert_eq!(parse_semver("1.10.0").unwrap(), semver(1, 10, 0));
}

#[test]
fn network_error_code_and_user_message() {
    let err = AppError::network("timeout");
    assert_eq!(err.code(), "network");
    assert_eq!(err.user_message(), "Could not check for updates.");
}

#[test]
fn parse_latest_json_allows_omitted_macos_and_linux() {
    let manifest = parse_latest_json(
        r#"{"version":"1.0.2","windows":"https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"}"#,
    )
    .expect("manifest");
    assert_eq!(manifest.version, "1.0.2");
    assert!(manifest.windows.is_some());
    assert!(manifest.macos.is_none());
    assert!(manifest.linux.is_none());
}

#[test]
fn parse_latest_json_rejects_bad_json() {
    let err = parse_latest_json("{not-json").expect_err("bad json");
    assert_eq!(err.code(), "network");
}

#[test]
fn parse_latest_json_rejects_missing_version() {
    let err =
        parse_latest_json(r#"{"windows":"https://github.com/Korigio/korigio-downloads/a.exe"}"#)
            .expect_err("missing version");
    assert_eq!(err.code(), "network");
}

#[test]
fn equal_remote_is_not_an_update() {
    let fetcher = MockFetcher {
        body: Ok(
            r#"{"version":"1.0.2","windows":"https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"}"#
                .into(),
        ),
    };
    let result = check_app_update_for(&fetcher, "1.0.2", OsAsset::Windows).expect("check");
    assert!(!result.update_available);
    assert_eq!(result.current_version, "1.0.2");
    assert_eq!(result.latest_version, "1.0.2");
}

#[test]
fn older_remote_is_not_an_update() {
    let fetcher = MockFetcher {
        body: Ok(r#"{"version":"1.0.1"}"#.into()),
    };
    let result = check_app_update_for(&fetcher, "1.0.2", OsAsset::Windows).expect("check");
    assert!(!result.update_available);
    assert_eq!(result.latest_version, "1.0.1");
}

#[test]
fn newer_remote_is_an_update() {
    let fetcher = MockFetcher {
        body: Ok(
            r#"{"version":"1.0.2","windows":"https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"}"#
                .into(),
        ),
    };
    let result = check_app_update_for(&fetcher, "1.0.1", OsAsset::Windows).expect("check");
    assert!(result.update_available);
    assert_eq!(
        result.download_url.as_deref(),
        Some("https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe")
    );
}

#[test]
fn disallowed_json_download_url_becomes_null() {
    let fetcher = MockFetcher {
        body: Ok(r#"{"version":"1.0.2","windows":"https://evil.com/setup.exe"}"#.into()),
    };
    let result = check_app_update_for(&fetcher, "1.0.1", OsAsset::Windows).expect("check");
    assert!(result.update_available);
    assert_eq!(result.download_url, None);
}

#[test]
fn fetcher_errors_are_network() {
    let fetcher = MockFetcher {
        body: Err("timeout"),
    };
    let err = check_app_update_for(&fetcher, "1.0.1", OsAsset::Windows).expect_err("network");
    assert_eq!(err.code(), "network");
    assert_eq!(err.user_message(), "Could not check for updates.");
}

#[test]
fn latest_json_url_is_allowlisted() {
    assert!(is_allowed_latest_json_url(LATEST_JSON_URL));
    assert!(!is_allowed_latest_json_url(
        "https://evil.com/korigio-downloads/latest.json"
    ));
    assert!(!is_allowed_latest_json_url(
        "http://korigio.github.io/korigio-downloads/latest.json"
    ));
    assert!(!is_allowed_latest_json_url(
        "https://korigio.github.io/korigio-downloads/other.json"
    ));
}

#[test]
fn allowlist_accepts_github_release_url() {
    assert!(is_allowed_open_url(
        "https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"
    ));
}

#[test]
fn allowlist_accepts_mailto_with_query() {
    assert!(is_allowed_open_url(
        "mailto:info@korigio.com?subject=Korigio%20feedback"
    ));
    validate_open_url("mailto:info@korigio.com?subject=Korigio%20feedback").expect("mailto");
}

#[test]
fn allowlist_rejects_evil_com() {
    assert!(!is_allowed_open_url("https://evil.com/setup.exe"));
    let err = validate_open_url("https://evil.com/setup.exe").expect_err("evil");
    assert!(matches!(
        err,
        AppError::Validation {
            field: None,
            ref message
        } if message == OPEN_URL_REJECTED
    ));
}

#[test]
fn allowlist_rejects_http() {
    assert!(!is_allowed_open_url(
        "http://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"
    ));
}

#[test]
fn allowlist_rejects_wrong_mailto() {
    assert!(!is_allowed_open_url("mailto:attacker@evil.com"));
    assert!(!is_allowed_open_url("mailto:info@korigio.com.evil.com"));
}

#[test]
fn sanitize_download_url_nulls_bad_links() {
    assert_eq!(
        sanitize_download_url(Some("https://evil.com/setup.exe")),
        None
    );
    assert!(sanitize_download_url(Some(
        "https://github.com/Korigio/korigio-downloads/releases/download/v1.0.2/setup.exe"
    ))
    .is_some());
}

#[test]
fn update_check_serializes_camel_case_and_null_url() {
    let dto = UpdateCheck {
        current_version: "1.0.1".into(),
        latest_version: "1.0.2".into(),
        update_available: true,
        download_url: None,
    };
    let value = serde_json::to_value(&dto).expect("json");
    assert_eq!(value["currentVersion"], "1.0.1");
    assert_eq!(value["latestVersion"], "1.0.2");
    assert_eq!(value["updateAvailable"], true);
    assert!(value["downloadUrl"].is_null());
}
