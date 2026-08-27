# ADR 003 — Repair numbering `YYYY-NNNNNN`

## Status

Accepted

## Context

Repairs need human-readable unique numbers. Collisions and year boundaries must be defined.

## Decision

- Format: `YYYY-NNNNNN` (example `2026-000001`).
- Allocate the next number **inside the same DB transaction** as the repair insert.
- Sequence resets each calendar year (local timezone).
- Enforce UNIQUE on `repair_number`.
- Frontend never invents repair numbers.

## Consequences

- Year change is safe and predictable.
- Concurrent creates cannot share a number.
- Display and search treat the full string as the public identifier.
