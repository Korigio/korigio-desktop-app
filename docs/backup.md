# Backup and restore

Critical feature (implement in Phase 10; design locked here).

## Manual backup

Settings → Backup → Create backup.

Produces a single portable package, e.g.:

```text
Korigio-YYYY-MM-DD-HHmm.backup
```

Legacy files named `Servioo-YYYY-MM-DD-HHmm.backup` are still valid packages and still count for due-date checks and restore.

(ZIP with custom extension or documented ZIP layout.)

Contents:

- SQLite database
- `images/` and `thumbs/`
- Relevant settings
- Manifest (app version, created_at, checksums)

## Restore

Settings → Restore backup.

1. Validate package + SQLite integrity.
2. Create a **safety backup** of current data (`Korigio-safety-YYYY-MM-DD-HHmm.backup`; legacy `Servioo-safety-` files still count for restore).
3. Replace data only after validation succeeds.
4. Verify the app can open the restored database.

Never overwrite a working DB without a safety backup.

**Phase 17:** refuse to restore a package whose `schema_migrations` max version is below **10**. Those backups used integer primary keys and cannot be upgraded in place. Tell the user to start from an empty shop or keep using the old app with that backup.

## Scheduled security copies

Opt-in. Default interval is **never** — the app does not write a scheduled copy until the shop sets an interval (`day` / `week` / `month` / `year`) and a destination folder.

When enabled, startup still calls `run_auto_backup_if_due`. A new `Korigio-YYYY-MM-DD-HHmm.backup` is written into the **chosen folder** only if that folder has no scheduled file whose filename date covers the current local period (same calendar day, ISO 8601 week-date, calendar month, or calendar year). Existing `Servioo-` and `Servioo-safety-` files in that folder still count as covering the period and can still be restored.

- Destination is the configured folder, not AppData `backups/auto/`.
- The user folder is **not** pruned.
- `list_local_backups` still lists AppData only and does not scan the chosen folder.
- Leftover files under AppData `backups/auto/` from earlier daily copies remain on disk.
- Packages stay unencrypted (same as manual backups).

## Uninstall

Uninstalling the application must **not** delete AppData (customer data) unless the user explicitly opts in (Phase 14).
