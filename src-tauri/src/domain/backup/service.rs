use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, DatabaseName};
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::db::repository::now_utc_rfc3339;
use crate::db::Db;
use crate::domain::backup::constants::{
    APP_VERSION, AUTO_RETENTION, BACKUP_EXTENSION, DATABASE_ENTRY, MANIFEST_NAME,
};
use crate::domain::backup::types::{
    AutoBackupResult, BackupInfo, BackupKind, BackupManifest, BackupManifestFile,
    BackupValidationResult, CreateBackupInput, LocalBackupListResult, RestoreBackupResult,
};
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn create_backup(db: &Db, input: CreateBackupInput) -> Result<BackupInfo, AppError> {
    let dest = resolve_manual_destination(db.paths(), input.destination_path.as_deref())?;
    write_backup_package(db, &dest, BackupKind::Manual)
}

pub fn create_safety_backup(db: &Db) -> Result<BackupInfo, AppError> {
    let file_name = format!("Servioo-safety-{}.{}", local_stamp()?, BACKUP_EXTENSION);
    let dest = db.paths().backups.join(&file_name);
    write_backup_package(db, &dest, BackupKind::Safety)
}

pub fn create_auto_backup(db: &Db) -> Result<BackupInfo, AppError> {
    let file_name = format!("Servioo-{}.{}", local_stamp()?, BACKUP_EXTENSION);
    let dest = db.paths().backups_auto().join(&file_name);
    write_backup_package(db, &dest, BackupKind::Auto)
}

pub fn validate_backup(path: &Path) -> Result<BackupValidationResult, AppError> {
    let mut errors = Vec::new();
    if !path.is_file() {
        return Ok(BackupValidationResult {
            valid: false,
            app_version: None,
            created_at: None,
            errors: vec!["Backup file was not found.".into()],
        });
    }

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| AppError::Validation {
        field: Some("path".into()),
        message: format!("Backup archive could not be opened: {err}"),
    })?;

    let manifest = match read_manifest(&mut archive) {
        Ok(m) => m,
        Err(err) => {
            return Ok(BackupValidationResult {
                valid: false,
                app_version: None,
                created_at: None,
                errors: vec![err],
            });
        }
    };

    if manifest.app_version.trim().is_empty() {
        errors.push("Manifest is missing appVersion.".into());
    }
    if manifest.created_at.trim().is_empty() {
        errors.push("Manifest is missing createdAt.".into());
    }

    let mut has_db = false;
    for entry in &manifest.files {
        if entry.path == DATABASE_ENTRY {
            has_db = true;
        }
        match archive.by_name(&entry.path) {
            Ok(mut zf) => {
                let digest = hash_reader(&mut zf).unwrap_or_default();
                if digest != entry.sha256 {
                    errors.push(format!("Checksum mismatch for {}.", entry.path));
                }
            }
            Err(_) => errors.push(format!("Missing file in backup: {}.", entry.path)),
        }
    }

    if !has_db {
        errors.push("Backup does not include database.sqlite.".into());
    }

    // Quick SQLite open check when checksums look good so far.
    if errors.is_empty() {
        if let Ok(mut zf) = archive.by_name(DATABASE_ENTRY) {
            let tmp = tempfile::Builder::new()
                .suffix(".sqlite")
                .tempfile()
                .map_err(|e| AppError::Io { source: e })?;
            {
                let mut out = File::create(tmp.path())?;
                std::io::copy(&mut zf, &mut out)?;
            }
            match Connection::open(tmp.path()) {
                Ok(conn) => {
                    let check: String = conn
                        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                        .unwrap_or_else(|err| format!("error:{err}"));
                    if check != "ok" {
                        errors.push(format!("Restored database failed integrity check: {check}"));
                    }
                }
                Err(err) => {
                    errors.push(format!("database.sqlite is not a valid SQLite file: {err}"))
                }
            }
        }
    }

    Ok(BackupValidationResult {
        valid: errors.is_empty(),
        app_version: Some(manifest.app_version),
        created_at: Some(manifest.created_at),
        errors,
    })
}

pub fn restore_backup(db: &mut Db, path: &Path) -> Result<RestoreBackupResult, AppError> {
    let validation = validate_backup(path)?;
    if !validation.valid {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: if validation.errors.is_empty() {
                "Backup is invalid.".into()
            } else {
                validation.errors.join(" ")
            },
        });
    }

    let safety = create_safety_backup(db)?;
    let paths = db.paths().clone();

    let staging = tempfile::tempdir()?;
    extract_backup_to(path, staging.path())?;

    let staged_db = staging.path().join(DATABASE_ENTRY);
    if !staged_db.is_file() {
        return Err(AppError::Validation {
            field: Some("path".into()),
            message: "Backup is missing database.sqlite.".into(),
        });
    }

    let max_version: i64 = {
        let conn = Connection::open(&staged_db)?;
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
    };
    if max_version < 10 {
        return Err(AppError::conflict(
            "This backup is from an older Servioo version and cannot be restored.",
        ));
    }

    db.checkpoint_wal()?;
    db.close_connection_for_restore()?;

    // Replace database file and media trees.
    replace_file(&staged_db, &paths.database)?;
    replace_tree(&staging.path().join("images"), &paths.images)?;
    replace_tree(&staging.path().join("thumbs"), &paths.thumbs)?;

    // Remove leftover WAL/SHM from previous connection if present.
    let _ = fs::remove_file(format!("{}-wal", paths.database.display()));
    let _ = fs::remove_file(format!("{}-shm", paths.database.display()));

    db.reopen_after_restore()?;

    Ok(RestoreBackupResult {
        restored_from: path.to_string_lossy().into_owned(),
        safety_backup_path: safety.path,
    })
}

pub fn list_local_backups(paths: &AppPaths) -> Result<LocalBackupListResult, AppError> {
    let mut items = Vec::new();
    collect_backups_in_dir(&paths.backups, BackupKind::Manual, &mut items)?;
    // Safety files also live under backups/ — reclassify by filename.
    for item in &mut items {
        if item.file_name.contains("-safety-") {
            item.kind = BackupKind::Safety;
        }
    }
    collect_backups_in_dir(&paths.backups_auto(), BackupKind::Auto, &mut items)?;
    items.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then(b.file_name.cmp(&a.file_name))
    });
    Ok(LocalBackupListResult { items })
}

pub fn run_auto_backup_if_due(db: &Db) -> Result<AutoBackupResult, AppError> {
    let auto_dir = db.paths().backups_auto();
    fs::create_dir_all(&auto_dir)?;

    if auto_backup_exists_today(&auto_dir)? {
        return Ok(AutoBackupResult {
            ran: false,
            backup: None,
            pruned_count: 0,
        });
    }

    let backup = create_auto_backup(db)?;
    let pruned_count = prune_auto_backups(&auto_dir)?;
    Ok(AutoBackupResult {
        ran: true,
        backup: Some(backup),
        pruned_count,
    })
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

    let _ = kind; // encoded in destination path / filename
    backup_info_for_path(dest, kind_from_path(dest))
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

fn hash_reader<R: Read>(reader: &mut R) -> Result<String, AppError> {
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

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<BackupManifest, String> {
    let mut file = archive
        .by_name(MANIFEST_NAME)
        .map_err(|_| "Backup is missing manifest.json.".to_string())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|err| format!("Could not read manifest.json: {err}"))?;
    serde_json::from_str(&buf).map_err(|err| format!("manifest.json is invalid: {err}"))
}

fn extract_backup_to(path: &Path, dest: &Path) -> Result<(), AppError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| AppError::Validation {
        field: Some("path".into()),
        message: format!("Backup archive could not be opened: {err}"),
    })?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|err| AppError::Internal {
            message: format!("failed to read zip entry: {err}"),
        })?;
        let name = entry.name().to_string();
        if name == MANIFEST_NAME {
            continue;
        }
        if has_unsafe_zip_path(&name) {
            return Err(AppError::Validation {
                field: Some("path".into()),
                message: "Backup contains an unsafe path.".into(),
            });
        }
        let out_path = dest.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}

fn has_unsafe_zip_path(name: &str) -> bool {
    let path = Path::new(name);
    path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir))
}

fn replace_file(src: &Path, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        fs::remove_file(dest)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

fn replace_tree(src: &Path, dest: &Path) -> Result<(), AppError> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    if !src.exists() {
        return Ok(());
    }
    copy_dir_recursive(src, dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn resolve_manual_destination(
    paths: &AppPaths,
    destination: Option<&str>,
) -> Result<PathBuf, AppError> {
    match destination {
        Some(raw) if !raw.trim().is_empty() => {
            let path = PathBuf::from(raw.trim());
            if path.is_dir() {
                Ok(path.join(format!("Servioo-{}.{}", local_stamp()?, BACKUP_EXTENSION)))
            } else {
                Ok(path)
            }
        }
        _ => Ok(paths
            .backups
            .join(format!("Servioo-{}.{}", local_stamp()?, BACKUP_EXTENSION))),
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

fn local_today() -> Result<Date, AppError> {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    Ok(now.date())
}

fn auto_backup_exists_today(auto_dir: &Path) -> Result<bool, AppError> {
    let today = local_today()?;
    if !auto_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(auto_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(&format!(".{BACKUP_EXTENSION}")) {
            continue;
        }
        if let Some(date) = parse_backup_date_from_name(&name) {
            if date == today {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn parse_backup_date_from_name(name: &str) -> Option<Date> {
    // Servioo-YYYY-MM-DD-HHmm.backup or Servioo-safety-YYYY-MM-DD-HHmm.backup
    let stem = name.strip_suffix(&format!(".{BACKUP_EXTENSION}"))?;
    let parts: Vec<&str> = stem.split('-').collect();
    // Servioo YYYY MM DD HHmm  OR Servioo safety YYYY MM DD HHmm
    let (y, m, d) = if parts.len() >= 5 && parts[1] == "safety" {
        (parts[2], parts[3], parts[4])
    } else if parts.len() >= 4 {
        (parts[1], parts[2], parts[3])
    } else {
        return None;
    };
    let year: i32 = y.parse().ok()?;
    let month: u8 = m.parse().ok()?;
    let day: u8 = d.parse().ok()?;
    Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()
}

fn prune_auto_backups(auto_dir: &Path) -> Result<u32, AppError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(auto_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if !name.ends_with(&format!(".{BACKUP_EXTENSION}")) {
            continue;
        }
        // Never treat safety backups as auto (they should not live here).
        if name.contains("-safety-") {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .unwrap_or(Duration::ZERO);
        files.push((modified, path));
    }
    files.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    let mut pruned = 0_u32;
    for (_ts, path) in files.into_iter().skip(AUTO_RETENTION) {
        fs::remove_file(path)?;
        pruned += 1;
    }
    Ok(pruned)
}

fn collect_backups_in_dir(
    dir: &Path,
    default_kind: BackupKind,
    items: &mut Vec<BackupInfo>,
) -> Result<(), AppError> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.ends_with(&format!(".{BACKUP_EXTENSION}")) => n.to_string(),
            _ => continue,
        };
        // Skip nested auto dir when scanning backups/
        if dir.ends_with("backups") && path.parent().is_some_and(|p| p.ends_with("auto")) {
            continue;
        }
        // Only direct children of dir
        if path.parent() != Some(dir) {
            continue;
        }
        let kind = if name.contains("-safety-") {
            BackupKind::Safety
        } else {
            default_kind.clone()
        };
        items.push(backup_info_for_path(&path, kind)?);
    }
    Ok(())
}

fn kind_from_path(path: &Path) -> BackupKind {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if name.contains("-safety-") {
        BackupKind::Safety
    } else if path.parent().is_some_and(|p| p.ends_with("auto")) {
        BackupKind::Auto
    } else {
        BackupKind::Manual
    }
}

fn backup_info_for_path(path: &Path, kind: BackupKind) -> Result<BackupInfo, AppError> {
    let meta = fs::metadata(path)?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    let created_at = parse_backup_date_from_name(&file_name)
        .and_then(|date| {
            date.with_hms(0, 0, 0)
                .ok()
                .map(|dt| dt.assume_utc())
                .and_then(|dt| dt.format(&Rfc3339).ok())
        })
        .or_else(|| {
            meta.modified()
                .ok()
                .and_then(|t| {
                    let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64;
                    OffsetDateTime::from_unix_timestamp(secs).ok()
                })
                .and_then(|dt| dt.format(&Rfc3339).ok())
        })
        .unwrap_or_else(|| now_utc_rfc3339().unwrap_or_else(|_| "1970-01-01T00:00:00Z".into()));

    Ok(BackupInfo {
        path: path.to_string_lossy().into_owned(),
        file_name,
        kind,
        created_at,
        size_bytes: meta.len(),
    })
}
