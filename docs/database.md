# Database

## Engine

SQLite via **`rusqlite`** (bundled SQLite). See [ADR 001](decisions/001-sqlite-rusqlite.md).

## Location

Resolved through Tauri `app.path().app_data_dir()` (OS AppData / Application Support). Layout:

```text
{appDataDir}/
  database.sqlite
  images/
  thumbs/
  backups/
  logs/
```

The app data directory is already scoped to this application by Tauri (identifier `com.servioo.desktop`). Do not store mutable data under the install directory.

## Migrations

- Numbered SQL files under `src-tauri/migrations/`.
- Applied automatically on startup via a migrations table.
- Never hand-edit a customer database schema.

## Initial schema (Phase 2+)

Tables with a current purpose only:

| Table | Purpose |
| --- | --- |
| `customers` | People / companies bringing devices |
| `devices` | Hardware belonging to a customer |
| `repairs` | Core repair orders |
| `repair_images` | Image metadata (files on disk) |
| `diagnosis_templates` | Reusable diagnosis checklists |
| `repair_diagnosis` | Diagnosis result for a repair |
| `settings` | Key/value app settings |
| `schema_migrations` | Migration bookkeeping |

### Soft archive

Customers and devices use `archived_at` (nullable). Repairs use status `cancelled` and may use `archived_at`; no hard-delete of repair history in v1. See [ADR 002](decisions/002-soft-archive.md).

### Repair numbers

Format `YYYY-NNNNNN`, unique, allocated inside the insert transaction, sequence resets per calendar year. See [ADR 003](decisions/003-repair-numbering.md).

### Repair statuses (codes)

`received` · `diagnosis` · `waiting_customer` · `waiting_part` · `in_repair` · `ready` · `collected` · `cancelled`

UI labels come from i18n only.

## Query rules

- Parameterized SQL only.
- Explicit column lists (no unjustified `SELECT *`).
- Indexes match named search/list patterns (added when those queries exist).
- Prefer SQL pagination over loading full tables into the UI.
- Images: paths + metadata in DB; binary files under `images/` / `thumbs/`.

## Integrity

- Foreign keys ON.
- Transactions for multi-step writes.
- UNIQUE / NOT NULL where appropriate.
- `created_at` / `updated_at` on mutable entities.
