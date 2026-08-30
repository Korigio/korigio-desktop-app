# Backup and restore

Critical feature (implement in Phase 10; design locked here).

## Manual backup

Settings → Backup → Create backup.

Produces a single portable package, e.g.:

```text
Servioo-YYYY-MM-DD-HHmm.backup
```

(ZIP with custom extension or documented ZIP layout.)

Contents:

- SQLite database
- `images/` and `thumbs/`
- Relevant settings
- Manifest (app version, created_at, checksums)

## Restore

Settings → Restore backup.

1. Validate package + SQLite integrity.
2. Create a **safety backup** of current data.
3. Replace data only after validation succeeds.
4. Verify the app can open the restored database.

Never overwrite a working DB without a safety backup.

**Phase 17:** refuse to restore a package whose `schema_migrations` max version is below **10**. Those backups used integer primary keys and cannot be upgraded in place. Tell the user to start from an empty shop or keep using the old app with that backup.

## Automatic backups

Daily copies under `backups/auto/`.

**Default retention:** 14 daily backups + keep the latest pre-restore safety backup.

Adjustable later via settings; do not keep unlimited backups by default.

## Uninstall

Uninstalling the application must **not** delete AppData (customer data) unless the user explicitly opts in (Phase 14).
