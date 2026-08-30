# ADR 008 — UUID v7 primary keys

## Status

Accepted

## Context

Servioo is a single-SQLite, integer-AUTOINCREMENT app. Local Wi‑Fi team sync requires two PCs to insert rows while offline without colliding on `id`. Integer keys cannot merge. A dual integer+uuid period would complicate every repository and the UI.

Existing AppData is disposable for this change (clean slate). There is no row migration from AUTOINCREMENT.

## Decision

All domain entity primary keys are **UUID v7** stored as `TEXT` (`8-4-4-4-12`, lowercase). Foreign keys are the same strings. IDs are generated in Rust services before INSERT (`uuid::Uuid::now_v7()`). The UI never invents IDs.

Tables that stay **without** a UUID PK: `schema_migrations`, `settings` (local key/value), `repair_number_sequences` (keyed by year + device code), `content_blobs` (content hash), `local_identity` (this PC only).

## Consequences

- IPC and React routes use string IDs (`/repairs/:id`). Integer `Number(params.id)` is a bug.
- Pre-Phase-17 backups cannot be restored.
- Image/document paths use the entity UUID, not a numeric folder.

## Date

2026-08-29
