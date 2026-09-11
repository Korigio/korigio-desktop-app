use std::fs;
use std::path::Path;

use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};

use crate::db::repository::now_utc_rfc3339;
use crate::domain::backup::constants::BACKUP_EXTENSION;
use crate::domain::backup::types::{BackupInfo, BackupKind, LocalBackupListResult};
use crate::error::AppError;
use crate::paths::AppPaths;

pub fn list_local_backups(paths: &AppPaths) -> Result<LocalBackupListResult, AppError> {
    let mut items = Vec::new();
    collect_backups_in_dir(&paths.backups, BackupKind::Manual, &mut items)?;
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
        let name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) if name.ends_with(&format!(".{BACKUP_EXTENSION}")) => name.to_string(),
            _ => continue,
        };
        if dir.ends_with("backups") && path.parent().is_some_and(|parent| parent.ends_with("auto"))
        {
            continue;
        }
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

pub(super) fn backup_info_for_path(path: &Path, kind: BackupKind) -> Result<BackupInfo, AppError> {
    let meta = fs::metadata(path)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();
    let created_at = parse_backup_date_from_name(&file_name)
        .and_then(|date| {
            date.with_hms(0, 0, 0)
                .ok()
                .map(|datetime| datetime.assume_utc())
                .and_then(|datetime| datetime.format(&Rfc3339).ok())
        })
        .or_else(|| {
            meta.modified()
                .ok()
                .and_then(|time| {
                    let secs = time.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64;
                    OffsetDateTime::from_unix_timestamp(secs).ok()
                })
                .and_then(|datetime| datetime.format(&Rfc3339).ok())
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

fn parse_backup_date_from_name(name: &str) -> Option<Date> {
    let stem = name.strip_suffix(&format!(".{BACKUP_EXTENSION}"))?;
    let parts: Vec<&str> = stem.split('-').collect();
    let (year, month, day) = if parts.len() >= 5 && parts[1] == "safety" {
        (parts[2], parts[3], parts[4])
    } else if parts.len() >= 4 {
        (parts[1], parts[2], parts[3])
    } else {
        return None;
    };
    Date::from_calendar_date(
        year.parse().ok()?,
        time::Month::try_from(month.parse::<u8>().ok()?).ok()?,
        day.parse().ok()?,
    )
    .ok()
}
