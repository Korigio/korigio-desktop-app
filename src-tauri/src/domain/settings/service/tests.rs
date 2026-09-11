use super::*;
use crate::domain::settings::types::{LocalePreference, ShopSettingsInput, ThemePreference};
use crate::error::AppError;

mod tests {
    use super::*;
    use crate::db::Db;
    use crate::domain::settings::types::{
        AutoBackupInterval, SetAutoBackupSettingsInput, AUTO_BACKUP_FOLDER_KEY, SYNC_INTERVAL_KEY,
    };

    #[test]
    fn maps_german_tags() {
        assert_eq!(map_tag_to_catalog("de-DE"), "de");
        assert_eq!(map_tag_to_catalog("de"), "de");
    }

    #[test]
    fn unknown_falls_back_to_spanish() {
        assert_eq!(map_tag_to_catalog("fr-FR"), "es");
    }

    #[test]
    fn system_preference_uses_os() {
        assert_eq!(resolve_catalog(LocalePreference::System, "de-CH"), "de");
        assert_eq!(resolve_catalog(LocalePreference::En, "de-CH"), "en");
    }

    #[test]
    fn shop_settings_defaults() {
        let db = Db::open_in_memory().expect("db");
        let settings = get_shop_settings(db.conn()).expect("get");
        assert_eq!(settings.tax_rate_percent, "19");
        assert_eq!(settings.currency, "EUR");
    }

    #[test]
    fn set_shop_settings_roundtrip_and_validation() {
        let db = Db::open_in_memory().expect("db");
        let updated = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: Some("19.5".into()),
                currency: Some("usd".into()),
            },
        )
        .expect("set");
        assert_eq!(updated.tax_rate_percent, "19.5");
        assert_eq!(updated.currency, "USD");

        let err = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: Some("101".into()),
                currency: None,
            },
        )
        .expect_err("invalid tax");
        assert!(matches!(err, AppError::Validation { .. }));

        let err = set_shop_settings(
            db.conn(),
            ShopSettingsInput {
                tax_rate_percent: None,
                currency: Some("EURO".into()),
            },
        )
        .expect_err("invalid currency");
        assert!(matches!(err, AppError::Validation { .. }));
    }

    #[test]
    fn sync_interval_defaults_to_five() {
        let db = Db::open_in_memory().expect("db");
        let settings = get_sync_interval(db.conn()).expect("get");
        assert_eq!(settings.interval_seconds, 5);
    }

    #[test]
    fn sync_interval_rejects_out_of_range() {
        let db = Db::open_in_memory().expect("db");
        for bad in [0u64, 1, 61, 120] {
            let err = set_sync_interval(db.conn(), bad).expect_err("invalid");
            match err {
                AppError::Validation { field, .. } => {
                    assert_eq!(field.as_deref(), Some("intervalSeconds"));
                }
                other => panic!("expected validation, got {other:?}"),
            }
        }
    }

    #[test]
    fn sync_interval_round_trip_bounds() {
        let db = Db::open_in_memory().expect("db");
        for secs in [2u64, 5, 60] {
            let set = set_sync_interval(db.conn(), secs).expect("set");
            assert_eq!(set.interval_seconds, secs);
            let got = get_sync_interval(db.conn()).expect("get");
            assert_eq!(got.interval_seconds, secs);
        }
    }

    #[test]
    fn sync_interval_corrupt_key_falls_back_to_default() {
        let db = Db::open_in_memory().expect("db");
        super::repository::upsert_setting(db.conn(), SYNC_INTERVAL_KEY, "not-a-number")
            .expect("corrupt");
        assert_eq!(
            get_sync_interval(db.conn()).expect("get").interval_seconds,
            5
        );
        super::repository::upsert_setting(db.conn(), SYNC_INTERVAL_KEY, "1").expect("too small");
        assert_eq!(
            get_sync_interval(db.conn()).expect("get").interval_seconds,
            5
        );
        super::repository::upsert_setting(db.conn(), SYNC_INTERVAL_KEY, "").expect("empty");
        assert_eq!(
            get_sync_interval(db.conn()).expect("get").interval_seconds,
            5
        );
    }

    #[test]
    fn sync_interval_is_local_only() {
        let db = Db::open_in_memory().expect("db");
        set_sync_interval(db.conn(), 30).expect("set");

        let sync_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM sync_changes", [], |row| row.get(0))
            .expect("count");
        assert_eq!(sync_count, 0);

        let shop_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sync_changes WHERE entity_table = 'shop_settings'",
                [],
                |row| row.get(0),
            )
            .expect("shop count");
        assert_eq!(shop_count, 0);

        let stored = super::repository::get_setting(db.conn(), SYNC_INTERVAL_KEY)
            .expect("read")
            .expect("present");
        assert_eq!(stored, "30");

        let (_version, rows) =
            crate::domain::sync::snapshot::dump_snapshot(db.conn()).expect("snapshot");
        for row in rows {
            if row.table == "shop_settings" {
                assert!(row.payload.get("syncIntervalSecs").is_none());
                assert!(row.payload.get("sync_interval_secs").is_none());
                assert_eq!(
                    row.payload.get("taxRatePercent").and_then(|v| v.as_str()),
                    Some("19")
                );
            }
        }
    }

    #[test]
    fn theme_defaults_to_system() {
        let db = Db::open_in_memory().expect("db");
        let settings = get_theme_settings(db.conn()).expect("get");
        assert_eq!(settings.preference, ThemePreference::System);
    }

    #[test]
    fn theme_round_trip_values() {
        let db = Db::open_in_memory().expect("db");
        for preference in [
            ThemePreference::Light,
            ThemePreference::Dark,
            ThemePreference::System,
        ] {
            let set = set_theme_preference(db.conn(), preference).expect("set");
            assert_eq!(set.preference, preference);
            let got = get_theme_settings(db.conn()).expect("get");
            assert_eq!(got.preference, preference);
        }
    }

    #[test]
    fn theme_corrupt_key_falls_back_to_system() {
        let db = Db::open_in_memory().expect("db");
        super::repository::upsert_setting(db.conn(), ThemePreference::STORAGE_KEY, "sepia")
            .expect("corrupt");
        assert_eq!(
            get_theme_settings(db.conn()).expect("get").preference,
            ThemePreference::System
        );
        super::repository::upsert_setting(db.conn(), ThemePreference::STORAGE_KEY, "LIGHT")
            .expect("uppercase");
        assert_eq!(
            get_theme_settings(db.conn()).expect("get").preference,
            ThemePreference::Light
        );
        super::repository::upsert_setting(db.conn(), ThemePreference::STORAGE_KEY, "")
            .expect("empty");
        assert_eq!(
            get_theme_settings(db.conn()).expect("get").preference,
            ThemePreference::System
        );
    }

    #[test]
    fn theme_preference_is_local_only() {
        let db = Db::open_in_memory().expect("db");
        set_theme_preference(db.conn(), ThemePreference::Dark).expect("set");

        let sync_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM sync_changes", [], |row| row.get(0))
            .expect("count");
        assert_eq!(sync_count, 0);

        let shop_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sync_changes WHERE entity_table = 'shop_settings'",
                [],
                |row| row.get(0),
            )
            .expect("shop count");
        assert_eq!(shop_count, 0);

        let stored = super::repository::get_setting(db.conn(), ThemePreference::STORAGE_KEY)
            .expect("read")
            .expect("present");
        assert_eq!(stored, "dark");

        let (_version, rows) =
            crate::domain::sync::snapshot::dump_snapshot(db.conn()).expect("snapshot");
        for row in rows {
            if row.table == "shop_settings" {
                assert!(row.payload.get("themePreference").is_none());
                assert!(row.payload.get("theme_preference").is_none());
                assert_eq!(
                    row.payload.get("taxRatePercent").and_then(|v| v.as_str()),
                    Some("19")
                );
            }
        }
    }

    fn auto_backup_input(
        interval: AutoBackupInterval,
        folder_path: Option<&str>,
    ) -> SetAutoBackupSettingsInput {
        SetAutoBackupSettingsInput {
            interval,
            folder_path: folder_path.map(|s| s.to_string()),
        }
    }

    fn assert_folder_path_error(err: AppError) {
        match err {
            AppError::Validation { field, .. } => {
                assert_eq!(field.as_deref(), Some("folderPath"));
            }
            other => panic!("expected folderPath validation, got {other:?}"),
        }
    }

    #[test]
    fn auto_backup_defaults_to_never_without_folder() {
        let db = Db::open_in_memory().expect("db");
        let settings = get_auto_backup_settings(db.conn()).expect("get");
        assert_eq!(settings.interval, AutoBackupInterval::Never);
        assert_eq!(settings.folder_path, None);
    }

    #[test]
    fn auto_backup_cannot_enable_without_folder() {
        let db = Db::open_in_memory().expect("db");
        for folder in [None, Some(""), Some("   ")] {
            let err = set_auto_backup_settings(
                db.conn(),
                auto_backup_input(AutoBackupInterval::Day, folder),
            )
            .expect_err("folder required");
            assert_folder_path_error(err);
        }
        let settings = get_auto_backup_settings(db.conn()).expect("get");
        assert_eq!(settings.interval, AutoBackupInterval::Never);
        assert_eq!(settings.folder_path, None);
    }

    #[test]
    fn auto_backup_never_allowed_without_folder() {
        let db = Db::open_in_memory().expect("db");
        let settings = set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Never, None),
        )
        .expect("set");
        assert_eq!(settings.interval, AutoBackupInterval::Never);
        assert_eq!(settings.folder_path, None);
    }

    #[test]
    fn auto_backup_keeps_folder_when_setting_never() {
        let db = Db::open_in_memory().expect("db");
        let dest = tempfile::tempdir().expect("dest");
        let path = dest.path().to_string_lossy().into_owned();

        set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Day, Some(&path)),
        )
        .expect("enable");

        let disabled = set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Never, Some(&path)),
        )
        .expect("never");
        assert_eq!(disabled.interval, AutoBackupInterval::Never);
        assert_eq!(disabled.folder_path.as_deref(), Some(path.as_str()));
    }

    #[test]
    fn auto_backup_rejects_clear_folder_while_enabled() {
        let db = Db::open_in_memory().expect("db");
        let dest = tempfile::tempdir().expect("dest");
        let path = dest.path().to_string_lossy().into_owned();
        set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Week, Some(&path)),
        )
        .expect("enable");

        for folder in [None, Some(""), Some("  ")] {
            let err = set_auto_backup_settings(
                db.conn(),
                auto_backup_input(AutoBackupInterval::Week, folder),
            )
            .expect_err("clear rejected");
            assert_folder_path_error(err);
        }
        let settings = get_auto_backup_settings(db.conn()).expect("get");
        assert_eq!(settings.interval, AutoBackupInterval::Week);
        assert_eq!(settings.folder_path.as_deref(), Some(path.as_str()));
    }

    #[test]
    fn auto_backup_rejects_relative_or_file_folder() {
        let db = Db::open_in_memory().expect("db");
        let err = set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Day, Some("relative/backups")),
        )
        .expect_err("relative");
        assert_folder_path_error(err);

        let dir = tempfile::tempdir().expect("dir");
        let file_path = dir.path().join("not-a-dir.txt");
        std::fs::write(&file_path, b"nope").expect("file");
        let err = set_auto_backup_settings(
            db.conn(),
            auto_backup_input(
                AutoBackupInterval::Month,
                Some(&file_path.to_string_lossy()),
            ),
        )
        .expect_err("file");
        assert_folder_path_error(err);
    }

    #[test]
    fn auto_backup_creates_missing_folder_on_enable() {
        let db = Db::open_in_memory().expect("db");
        let parent = tempfile::tempdir().expect("parent");
        let dest = parent.path().join("scheduled");
        assert!(!dest.exists());
        let path = dest.to_string_lossy().into_owned();

        let settings = set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Year, Some(&path)),
        )
        .expect("set");
        assert_eq!(settings.interval, AutoBackupInterval::Year);
        assert_eq!(settings.folder_path.as_deref(), Some(path.as_str()));
        assert!(dest.is_dir());
    }

    #[test]
    fn auto_backup_corrupt_interval_falls_back_to_never() {
        let db = Db::open_in_memory().expect("db");
        super::repository::upsert_setting(db.conn(), AutoBackupInterval::STORAGE_KEY, "daily")
            .expect("corrupt");
        assert_eq!(
            get_auto_backup_settings(db.conn()).expect("get").interval,
            AutoBackupInterval::Never
        );
        super::repository::upsert_setting(db.conn(), AutoBackupInterval::STORAGE_KEY, "")
            .expect("empty");
        assert_eq!(
            get_auto_backup_settings(db.conn()).expect("get").interval,
            AutoBackupInterval::Never
        );
    }

    #[test]
    fn auto_backup_unknown_interval_parse_is_none() {
        assert_eq!(AutoBackupInterval::parse("daily"), None);
        assert_eq!(AutoBackupInterval::parse("hourly"), None);
        assert_eq!(
            AutoBackupInterval::parse("DAY"),
            Some(AutoBackupInterval::Day)
        );
    }

    #[test]
    fn auto_backup_settings_are_local_only() {
        let db = Db::open_in_memory().expect("db");
        let dest = tempfile::tempdir().expect("dest");
        let path = dest.path().to_string_lossy().into_owned();
        set_auto_backup_settings(
            db.conn(),
            auto_backup_input(AutoBackupInterval::Day, Some(&path)),
        )
        .expect("set");

        let sync_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM sync_changes", [], |row| row.get(0))
            .expect("count");
        assert_eq!(sync_count, 0);

        let stored_interval =
            super::repository::get_setting(db.conn(), AutoBackupInterval::STORAGE_KEY)
                .expect("read")
                .expect("present");
        assert_eq!(stored_interval, "day");
        let stored_folder = super::repository::get_setting(db.conn(), AUTO_BACKUP_FOLDER_KEY)
            .expect("read")
            .expect("present");
        assert_eq!(stored_folder, path);

        let (_version, rows) =
            crate::domain::sync::snapshot::dump_snapshot(db.conn()).expect("snapshot");
        for row in rows {
            if row.table == "shop_settings" {
                assert!(row.payload.get("autoBackupInterval").is_none());
                assert!(row.payload.get("autoBackupFolder").is_none());
                assert!(row.payload.get("auto_backup_interval").is_none());
                assert!(row.payload.get("auto_backup_folder").is_none());
                assert_eq!(
                    row.payload.get("taxRatePercent").and_then(|v| v.as_str()),
                    Some("19")
                );
            }
        }
    }
}
