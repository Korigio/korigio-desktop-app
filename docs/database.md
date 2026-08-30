# Database

## Engine

SQLite via **`rusqlite`** (bundled SQLite). See [ADR 001](decisions/001-sqlite-rusqlite.md).

## Location

Resolved through Tauri `app.path().app_data_dir()` (OS AppData / Application Support). Layout:

```text
{appDataDir}/
  database.sqlite
  images/
    {repairUuid}/
      {contentHash}.{ext}
    companies/{companyUuid}/
      {contentHash}.{ext}
  thumbs/
    {repairUuid}/
      {contentHash}.jpg
  documents/
    repairs/{repairUuid}/
      {contentHash}.{ext}
  blobs/
    {hash[0:2]}/
      {hash}
  backups/
    auto/
    Servioo-*.backup
    Servioo-safety-*.backup
  logs/
```

Image binaries live on disk; `repair_images` stores **relative** display paths under the AppData root plus `content_hash`. Thumbs are ~320px JPEG. Caps: JPEG/PNG, 10 MB/file, 30 images/repair. Sync copies missing blobs by hash ([ADR 013](decisions/013-team-psk-and-blobs.md)).

The app data directory is already scoped to this application by Tauri (identifier `com.servioo.desktop`). Do not store mutable data under the install directory.

## Migrations

- Numbered SQL files under `src-tauri/migrations/`.
- Applied automatically on startup via a migrations table.
- Never hand-edit a customer database schema.
- **Migration 010** (Phase 17) DROP-recreates domain tables with UUID v7 `TEXT` primary keys. Existing rows are discarded. Pre-010 backups cannot be restored.

## Schema (Phase 17+)

Domain entity PKs are UUID v7 `TEXT`. See [ADR 008](decisions/008-uuid-v7-primary-keys.md).

| Table | Purpose |
| --- | --- |
| `customers` | People / companies bringing devices |
| `devices` | Hardware belonging to a customer |
| `repairs` | Core repair orders (`assigned_to_staff_id`) |
| `repair_images` | Image metadata + `content_hash` |
| `diagnosis_templates` | Reusable diagnosis checklists |
| `repair_diagnosis` | Diagnosis result for a repair |
| `companies` | Shop / billing companies |
| `repair_documents` | Repair file metadata + `content_hash` |
| `staff` | Named operators; `role` admin \| staff |
| `teams` | Shop team (replicated) |
| `team_devices` | PCs in the team |
| `team_invites` | Invite code hashes |
| `content_blobs` | Content-addressed file metadata |
| `sync_changes` | Gossip log |
| `settings` | Local key/value (not replicated except shop tax/currency via sync). Includes `locale_preference`, `sync_interval_secs`, `theme_preference`, `auto_backup_interval`, and `auto_backup_folder` (local, not replicated). |
| `local_identity` | This PC only (device id, PSK, session) |
| `presence` | Last-seen peers (local) |
| `repair_number_sequences` | `(year, device_code)` allocator |
| `schema_migrations` | Migration bookkeeping |

Replicated rows carry HLC columns: `hlc_wall_ms`, `hlc_counter`, `origin_device_id`, `updated_by_staff_id`, optional `deleted_at` tombstone. See [ADR 010](decisions/010-hlc-last-write-wins.md).

### Soft archive

Customers and devices use `archived_at` (nullable). Repairs use status `cancelled` and may use `archived_at`; no hard-delete of repair history in v1. See [ADR 002](decisions/002-soft-archive.md).

### Repair numbers

Format `YYYY-DX-NNNNNN` (example `2026-AA-000001`), unique, allocated inside the insert transaction, sequence per `(year, device_code)`. See [ADR 012](decisions/012-repair-numbering-device-code.md).

### Repair statuses (codes)

`received` · `diagnosis` · `waiting_customer` · `waiting_part` · `in_repair` · `ready` · `awaiting_pickup` · `collected` · `cancelled`

UI labels come from i18n only.

## Query rules

- Parameterized SQL only.
- Explicit column lists (no unjustified `SELECT *`).
- Indexes match named search/list patterns (added when those queries exist).
- Prefer SQL pagination over loading full tables into the UI.
- Images: paths + `content_hash` in DB; binaries under `images/` / `thumbs/` / `blobs/`.

## Integrity

- Foreign keys ON.
- Transactions for multi-step writes.
- UNIQUE / NOT NULL where appropriate.
- `created_at` / `updated_at` on mutable entities.
