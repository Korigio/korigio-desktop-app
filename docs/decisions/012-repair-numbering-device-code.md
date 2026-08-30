# ADR 012 — Repair numbering `YYYY-DX-NNNNNN`

## Status

Accepted (supersedes the **format** in [ADR 003](003-repair-numbering.md); transactional allocation and UNIQUE remain).

## Context

ADR 003 allocated `YYYY-NNNNNN` from a per-year sequence in one database. Two PCs creating repairs offline would both mint `2026-000042`.

## Decision

- Format: `YYYY-DX-NNNNNN` (example `2026-AA-000001`).
- `DX` is a two-letter per-PC code `[A-Z]{2}` assigned at team create or join (`AA`…`ZZ`). Solo / first device uses `AA`.
- Allocate the next number **inside the same DB transaction** as the repair insert, keyed by `(year, device_code)`.
- Sequence resets each calendar year (local timezone) **per device code**.
- Enforce UNIQUE on `repair_number`.
- Frontend never invents repair numbers.

## Consequences

- Numbers stay unique without a central allocator.
- Display and search treat the full string as the public identifier.
- Maximum 676 devices per team (`AA`–`ZZ`).
- Old numbers `YYYY-NNNNNN` will not appear in new databases.

## Date

2026-08-29
