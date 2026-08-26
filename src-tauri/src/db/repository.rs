//! Shared helpers for domain repositories (Phase 3+).

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::AppError;

#[allow(dead_code)] // Phase 3+ repositories
pub fn now_utc_rfc3339() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| AppError::Internal {
            message: format!("timestamp format failed: {err}"),
        })
}
