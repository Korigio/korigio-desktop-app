# ADR 005 — Three locales from day one

## Status

Accepted (amended: OS locale + Settings preference)

## Context

The product needs Spanish, German, and English. Hard-coded UI strings block localization and create inconsistent AI edits. The workshop may run Windows/macOS in German while the product still needs a Spanish fallback for unsupported OS languages.

## Decision

- Maintain `en`, `es`, and `de` message catalogs under `src/i18n/`.
- No user-facing hard-coded strings in components.
- Status and similar codes stay English identifiers in the DB; labels come from i18n.
- **Default preference:** follow the **OS language** (`locale_preference = system`).
- Map OS tags: `de*` → German, `en*` → English, `es*` → Spanish; anything else → **Spanish** fallback.
- Users can override via **Settings** (native menu `Settings…` / `⌘,` and in-app `/settings`), persisted in SQLite `settings`.

## Consequences

- Every new UI string must be added to all three catalogs.
- Locale resolution runs at startup through Tauri (`get_locale_settings`).
- Changing language updates the UI immediately without restart.
