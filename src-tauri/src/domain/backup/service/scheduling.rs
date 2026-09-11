use std::fs;
use std::path::{Path, PathBuf};

use time::{Date, OffsetDateTime};

use crate::db::Db;
use crate::domain::backup::constants::{
    BACKUP_EXTENSION, BACKUP_FILE_PREFIX, BACKUP_FILE_PREFIX_LEGACY,
};
use crate::domain::backup::types::{AutoBackupResult, AutoBackupSkipReason};
use crate::domain::settings::{self, AutoBackupInterval};
use crate::error::AppError;

use super::package::create_auto_backup;

pub fn run_auto_backup_if_due(db: &Db) -> Result<AutoBackupResult, AppError> {
    let settings = settings::get_auto_backup_settings(db.conn())?;
    if settings.interval == AutoBackupInterval::Never {
        return Ok(skipped_auto_backup(AutoBackupSkipReason::Disabled));
    }
    let Some(folder) = settings.folder_path.as_deref().filter(|s| !s.is_empty()) else {
        return Ok(skipped_auto_backup(AutoBackupSkipReason::NoFolder));
    };
    let dest_dir = PathBuf::from(folder);
    if !dest_dir.is_absolute() || fs::create_dir_all(&dest_dir).is_err() || !dest_dir.is_dir() {
        return Ok(skipped_auto_backup(AutoBackupSkipReason::NoFolder));
    }
    if scheduled_backup_covers_period(&dest_dir, settings.interval)? {
        return Ok(skipped_auto_backup(AutoBackupSkipReason::NotDue));
    }
    Ok(AutoBackupResult {
        ran: true,
        backup: Some(create_auto_backup(db, &dest_dir)?),
        pruned_count: 0,
        skipped_reason: None,
    })
}

fn skipped_auto_backup(reason: AutoBackupSkipReason) -> AutoBackupResult {
    AutoBackupResult {
        ran: false,
        backup: None,
        pruned_count: 0,
        skipped_reason: Some(reason),
    }
}

pub fn filename_covers_period(name: &str, interval: AutoBackupInterval, today: Date) -> bool {
    let Some(date) = parse_scheduled_backup_date(name) else {
        return false;
    };
    match interval {
        AutoBackupInterval::Never => false,
        AutoBackupInterval::Day => date == today,
        AutoBackupInterval::Week => {
            let (year, week, _) = date.to_iso_week_date();
            let (today_year, today_week, _) = today.to_iso_week_date();
            year == today_year && week == today_week
        }
        AutoBackupInterval::Month => date.year() == today.year() && date.month() == today.month(),
        AutoBackupInterval::Year => date.year() == today.year(),
    }
}

fn parse_scheduled_backup_date(name: &str) -> Option<Date> {
    let stem = name.strip_suffix(&format!(".{BACKUP_EXTENSION}"))?;
    let rest = scheduled_backup_date_stem(stem)?;
    let parts: Vec<&str> = rest.split('-').collect();
    if parts.len() != 4 {
        return None;
    }
    let (year, month, day, time) = (parts[0], parts[1], parts[2], parts[3]);
    if year.len() != 4
        || month.len() != 2
        || day.len() != 2
        || time.len() != 4
        || !year.chars().all(|c| c.is_ascii_digit())
        || !month.chars().all(|c| c.is_ascii_digit())
        || !day.chars().all(|c| c.is_ascii_digit())
        || !time.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    Date::from_calendar_date(
        year.parse().ok()?,
        time::Month::try_from(month.parse::<u8>().ok()?).ok()?,
        day.parse().ok()?,
    )
    .ok()
}

fn scheduled_backup_date_stem(stem: &str) -> Option<&str> {
    for prefix in [BACKUP_FILE_PREFIX, BACKUP_FILE_PREFIX_LEGACY] {
        if stem.starts_with(&format!("{prefix}-safety-")) {
            return None;
        }
        if let Some(rest) = stem.strip_prefix(&format!("{prefix}-")) {
            return Some(rest);
        }
    }
    None
}

fn scheduled_backup_covers_period(
    dest_dir: &Path,
    interval: AutoBackupInterval,
) -> Result<bool, AppError> {
    let today = OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date();
    if !dest_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(dest_dir)? {
        let entry = entry?;
        if entry.path().is_file()
            && filename_covers_period(&entry.file_name().to_string_lossy(), interval, today)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
