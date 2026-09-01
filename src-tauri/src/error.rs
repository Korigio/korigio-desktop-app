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
    #[allow(dead_code)]
    DataPath { message: String },

    #[error("I/O error")]
    Io {
        #[source]
        source: std::io::Error,
    },

    #[error("Internal error")]
    Internal { message: String },

    #[error("Validation error")]
    Validation {
        field: Option<String>,
        message: String,
    },

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized { message: String },

    #[error("Forbidden")]
    Forbidden { message: String },

    #[error("Conflict")]
    Conflict { message: String },

    #[error("Sync error")]
    Sync { message: String },

    #[error("Network error")]
    Network { message: String },
}

impl AppError {
    pub fn unauthorized() -> Self {
        Self::Unauthorized {
            message: "You need to sign in to continue.".into(),
        }
    }

    pub fn invalid_credentials() -> Self {
        Self::Unauthorized {
            message: "PIN or staff is not valid.".into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn sync_err(message: impl Into<String>) -> Self {
        Self::Sync {
            message: message.into(),
        }
    }

    pub fn network(reason: impl Into<String>) -> Self {
        Self::Network {
            message: reason.into(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Database { .. } => "database",
            Self::Migration { .. } => "migration",
            Self::DataPath { .. } => "data_path",
            Self::Io { .. } => "io",
            Self::Internal { .. } => "internal",
            Self::Validation { .. } => "validation",
            Self::NotFound => "not_found",
            Self::Unauthorized { .. } => "unauthorized",
            Self::Forbidden { .. } => "forbidden",
            Self::Conflict { .. } => "conflict",
            Self::Sync { .. } => "sync",
            Self::Network { .. } => "network",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::Database { .. } => {
                "The database operation could not be completed. Your existing data has not been changed unexpectedly. Please try again.".into()
            }
            Self::Migration { .. } => {
                "The application database could not be updated. Please restart the application. If the problem continues, restore a backup.".into()
            }
            Self::DataPath { .. } | Self::Io { .. } => {
                "The application could not access its data folder. Check disk permissions and try again.".into()
            }
            Self::Internal { .. } => "Something went wrong. Please try again.".into(),
            Self::Validation { message, .. } => message.clone(),
            Self::NotFound => "The requested record was not found.".into(),
            Self::Unauthorized { message } => message.clone(),
            Self::Forbidden { message } => message.clone(),
            Self::Conflict { message } => message.clone(),
            Self::Sync { message } => message.clone(),
            Self::Network { .. } => "Could not check for updates.".into(),
        }
    }

    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Validation { field, .. } => field.as_deref(),
            _ => None,
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

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        match &value {
            AppError::Network { message } => {
                eprintln!("app_error code=network reason={message}");
            }
            other => {
                eprintln!("app_error code={} detail={other:?}", other.code());
            }
        }
        Self {
            code: value.code().to_string(),
            field: value.field().map(str::to_string),
            message: value.user_message(),
        }
    }
}
