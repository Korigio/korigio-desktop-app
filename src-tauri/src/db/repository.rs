//! Shared helpers for domain repositories (Phase 3+).

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::error::AppError;

pub fn now_utc_rfc3339() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| AppError::Internal {
            message: format!("timestamp format failed: {err}"),
        })
}

/// Trim, escape `\`, `%`, `_`, and wrap with `%` for SQL `LIKE … ESCAPE '\'`.
/// Returns `None` when the input is missing or whitespace-only.
pub fn like_pattern(search: Option<&str>) -> Option<String> {
    search.map(str::trim).filter(|s| !s.is_empty()).map(|s| {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        format!("%{escaped}%")
    })
}
