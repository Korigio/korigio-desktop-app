# ADR 002 — Soft archive instead of hard delete

## Status

Accepted

## Context

Deleting a customer or device that has repair history must not destroy workshop records.

## Decision

- Customers and devices: soft archive via nullable `archived_at`.
- Repairs: use status `cancelled`; may also set `archived_at`. No hard-delete of repair history in v1.
- Archived records are hidden from default lists but remain searchable/auditable when needed.

## Consequences

- UI needs archive / restore (or unarchive) actions.
- Queries default to `archived_at IS NULL`.
- Physical purge, if ever needed, is a separate explicit future feature.
