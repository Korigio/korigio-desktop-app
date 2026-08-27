//! Application data directories (never under the install folder).

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub database: PathBuf,
    pub images: PathBuf,
    pub thumbs: PathBuf,
    pub backups: PathBuf,
    pub logs: PathBuf,
}

impl AppPaths {
    pub fn from_app_data_dir(app_data_dir: PathBuf) -> Result<Self, AppError> {
        let root = app_data_dir;
        let paths = Self {
            database: root.join("database.sqlite"),
            images: root.join("images"),
            thumbs: root.join("thumbs"),
            backups: root.join("backups"),
            logs: root.join("logs"),
            root,
        };
        paths.ensure_directories()?;
        Ok(paths)
    }

    /// Test helper: use an arbitrary root directory.
    #[allow(dead_code)]
    pub fn from_root(root: impl Into<PathBuf>) -> Result<Self, AppError> {
        Self::from_app_data_dir(root.into())
    }

    pub fn ensure_directories(&self) -> Result<(), AppError> {
        for dir in [&self.root, &self.images, &self.thumbs, &self.backups, &self.logs] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
