use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use rusqlite::{Connection, DatabaseName};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::backup::constants::{
    APP_VERSION, BACKUP_EXTENSION, BACKUP_FILE_PREFIX, DATABASE_ENTRY, MANIFEST_NAME,
};
use crate::domain::backup::types::{
    BackupInfo, BackupKind, BackupManifest, BackupManifestFile, CreateBackupInput,
};
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn create_backup(db: &Db, input: CreateBackupInput) -> Result<BackupInfo, AppError> {
    let dest = resolve_manual_destination(db.paths(), input.destination_path.as_deref())?;
    write_backup_package(db, &dest, BackupKind::Manual)
}

pub(super) fn create_safety_backup(db: &Db) -> Result<BackupInfo, AppError> {
    let file_name = backup_file_name(&local_stamp()?, true);
    let dest = db.paths().backups.join(&file_name);
    write_backup_package(db, &dest, BackupKind::Safety)
}

pub(super) fn create_auto_backup(db: &Db, dest_dir: &Path) -> Result<BackupInfo, AppError> {
    let file_name = backup_file_name(&local_stamp()?, false);
    let dest = dest_dir.join(&file_name);
    write_backup_package(db, &dest, BackupKind::Auto)
}

fn write_backup_package(db: &Db, dest: &Path, kind: BackupKind) -> Result<BackupInfo, AppError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_dir = tempfile::tempdir()?;
    let db_copy = tmp_dir.path().join(DATABASE_ENTRY);
    online_backup_database(db.conn(), &db_copy)?;

    let created_at = now_utc_rfc3339()?;
    let mut files_meta = Vec::new();

    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    add_file_to_zip(&mut zip, options, DATABASE_ENTRY, &db_copy, &mut files_meta)?;
    add_tree_to_zip(
        &mut zip,
        options,
        &db.paths().images,
        "images",
        &mut files_meta,
    )?;
    add_tree_to_zip(
        &mut zip,
        options,
        &db.paths().thumbs,
        "thumbs",
        &mut files_meta,
    )?;

    let manifest = BackupManifest {
        app_version: APP_VERSION.to_string(),
        created_at: created_at.clone(),
        files: files_meta,
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|err| AppError::Internal {
        message: format!("failed to serialize backup manifest: {err}"),
    })?;
    zip.start_file(MANIFEST_NAME, options)
        .map_err(|err| AppError::Internal {
            message: format!("failed to write manifest entry: {err}"),
        })?;
    zip.write_all(&manifest_json)?;
    zip.finish().map_err(|err| AppError::Internal {
        message: format!("failed to finalize backup archive: {err}"),
    })?;

    super::listing::backup_info_for_path(dest, kind)
}

fn online_backup_database(src: &Connection, dest: &Path) -> Result<(), AppError> {
    src.backup(DatabaseName::Main, dest, None)?;
    Ok(())
}

fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    options: SimpleFileOptions,
    entry_name: &str,
    path: &Path,
    files_meta: &mut Vec<BackupManifestFile>,
) -> Result<(), AppError> {
    let mut input = File::open(path)?;
    let sha256 = hash_reader(&mut input)?;
    input = File::open(path)?;
    zip.start_file(entry_name, options)
        .map_err(|err| AppError::Internal {
            message: format!("failed to start zip entry {entry_name}: {err}"),
        })?;
    std::io::copy(&mut input, zip)?;
    files_meta.push(BackupManifestFile {
        path: entry_name.to_string(),
        sha256,
    });
    Ok(())
}

fn add_tree_to_zip(
    zip: &mut ZipWriter<File>,
    options: SimpleFileOptions,
    root: &Path,
    prefix: &str,
    files_meta: &mut Vec<BackupManifestFile>,
) -> Result<(), AppError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in walkdir(root)? {
        let rel = entry.strip_prefix(root).map_err(|err| AppError::Internal {
            message: format!("path strip failed: {err}"),
        })?;
        let entry_name = format!("{prefix}/{}", path_to_unix(rel));
        add_file_to_zip(zip, options, &entry_name, &entry, files_meta)?;
    }
    Ok(())
}

fn walkdir(root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut out = Vec::new();
    fn rec(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), AppError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                rec(&path, out)?;
            } else if path.is_file() {
                out.push(path);
            }
        }
        Ok(())
    }
    rec(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn path_to_unix(path: &Path) -> String {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn hash_reader<R: Read>(reader: &mut R) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn resolve_manual_destination(
    paths: &AppPaths,
    destination: Option<&str>,
) -> Result<PathBuf, AppError> {
    match destination {
        Some(raw) if !raw.trim().is_empty() => {
            let path = PathBuf::from(raw.trim());
            if path.is_dir() {
                Ok(path.join(backup_file_name(&local_stamp()?, false)))
            } else {
                Ok(path)
            }
        }
        _ => Ok(paths.backups.join(backup_file_name(&local_stamp()?, false))),
    }
}

fn local_stamp() -> Result<String, AppError> {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    // YYYY-MM-DD-HHmm in local time
    Ok(format!(
        "{:04}-{:02}-{:02}-{:02}{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute()
    ))
}

fn backup_file_name(stamp: &str, safety: bool) -> String {
    if safety {
        format!("{BACKUP_FILE_PREFIX}-safety-{stamp}.{BACKUP_EXTENSION}")
    } else {
        format!("{BACKUP_FILE_PREFIX}-{stamp}.{BACKUP_EXTENSION}")
    }
}
