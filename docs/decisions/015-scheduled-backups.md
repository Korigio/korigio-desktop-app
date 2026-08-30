# ADR 015 — Opt-in scheduled security copies

## Status

Accepted

## Context

Phase 10 wrote a daily package under AppData `backups/auto/` on every startup (`run_auto_backup_if_due`) and pruned that folder to 14 files. That ran without a shop choosing a destination or interval, filled AppData by default, and treated retention as an implicit product rule.

## Decision

- Default interval is **never**. Missing or corrupt `auto_backup_interval` is treated as never. No scheduled copy until the user enables one.
- Destination is a user-chosen folder (`auto_backup_folder`). AppData `backups/auto/` is no longer the default write target.
- Persist both keys in the local SQLite `settings` table. Do **not** replicate via LAN sync (`sync_changes` / shop snapshot).
- Do **not** prune the user folder. `prunedCount` on `run_auto_backup_if_due` is always 0.
- Packages stay unencrypted (same as manual backups; [ADR 004](004-encryption-deferred.md)).
- Due is the filename calendar period only (`Servioo-YYYY-MM-DD-HHmm.backup`): same local day, ISO 8601 week-date `(iso_year, iso_week)`, calendar month, or calendar year. No time-of-day.
- Startup still calls `run_auto_backup_if_due`. The command name is unchanged; skip reasons are `disabled`, `noFolder`, and `notDue`.

## Consequences

- Existing installs stop writing automatic copies until a folder and interval are configured.
- Leftover files under AppData `backups/auto/` remain on disk.
- `list_local_backups` does not scan the user folder (AppData only).
- Rust `std::fs` write outside AppData is the same exception as manual create.

## Date

2026-08-30
