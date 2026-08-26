# ADR 004 — Defer database encryption to Phase 12

## Status

Accepted (temporary)

## Context

Customer PII deserves protection, but SQLCipher / DPAPI key UX affects portability, installer size, performance, and backup restore story.

## Decision

Ship early phases without at-rest SQLite encryption. Keep validation, CSP, capability lockdown, AppData separation, and clean logging from day one. Revisit encryption in Phase 12 with measured trade-offs.

Candidates then: optional app unlock password; SQLCipher; DPAPI-wrapped key material. Never invent crypto.

## Consequences

- Disk access under an unlocked Windows profile can read the DB file until encryption lands.
- Backups stay simple and portable in the meantime.
- Architecture must not paint us into a corner (paths, backup manifest remain encryption-ready).
