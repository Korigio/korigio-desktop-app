//! Application errors: safe messages for the UI, details for logs.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error")]
    Database {
        #[source]
        source: rusqlite::Error,
    },

    #[error("Database migration failed")]
    Migration { message: String },

    #[error("Application data path is unavailable")]
    #[allow(dead_code)] // reserved for explicit path errors
    DataPath { message: String },

    #[error("I/O error")]
    Io {
        #[source]
        source: std::io::Error,
    },

    #[error("Internal error")]
    Internal { message: String },
}

impl AppError {
    /// Stable machine-readable code for the frontend.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database { .. } => "database",
            Self::Migration { .. } => "migration",
            Self::DataPath { .. } => "data_path",
            Self::Io { .. } => "io",
            Self::Internal { .. } => "internal",
        }
    }

    /// User-facing message (no internal SQL / path details).
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Database { .. } => {
                "The database operation could not be completed. Your existing data has not been changed unexpectedly. Please try again."
            }
            Self::Migration { .. } => {
                "The application database could not be updated. Please restart the application. If the problem continues, restore a backup."
            }
            Self::DataPath { .. } | Self::Io { .. } => {
                "The application could not access its data folder. Check disk permissions and try again."
            }
            Self::Internal { .. } => {
                "Something went wrong. Please try again."
            }
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(source: rusqlite::Error) -> Self {
        Self::Database { source }
    }
}

impl From<std::io::Error> for AppError {
    fn from(source: std::io::Error) -> Self {
        Self::Io { source }
    }
}

/// Payload returned to the frontend via Tauri commands.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        // Log technical detail without PII (errors here are structural).
        eprintln!(
            "app_error code={} detail={value:?}",
            value.code()
        );
        Self {
            code: value.code().to_string(),
            message: value.user_message().to_string(),
        }
    }
}
