# ADR 005 — Three locales from day one

## Status

Accepted

## Context

The product needs Spanish, German, and English. Hard-coded UI strings block localization and create inconsistent AI edits.

## Decision

- Maintain `en`, `es`, and `de` message catalogs under `src/i18n/`.
- No user-facing hard-coded strings in components.
- Status and similar codes stay English identifiers in the DB; labels come from i18n.
- Default UI locale: **Spanish** until product owner changes it.

## Consequences

- Slightly more setup in Phase 1.
- Every new UI string must be added to all three catalogs.
- Prepares for switching default language without refactors.
