# ADR 004 — Database encryption (Phase 12 resolution)

## Status

**Accepted for v1** — no at-rest SQLite encryption.

## Context

Customer PII deserves protection. SQLCipher / DPAPI / unlock-password options affect portability, backup restore, installer size, and performance. Early phases deferred the choice (temporary ADR).

## Decision (Phase 12)

**Ship Servioo v1 without at-rest database encryption.** Rely on:

- Windows user-account / disk encryption (BitLocker) as the primary physical-access control
- AppData isolation (`com.servioo.desktop`)
- Least-privilege Tauri capabilities and WebView CSP
- Parameterized SQL + Rust validation on every command
- No PII in application logs

Backups (`.backup` ZIP) remain **portable and unencrypted** by design so shops can move data between PCs without a key ceremony. Encrypted-at-rest DB or password-locked backups require a **new ADR** if the threat model changes.

Candidates explicitly **not** chosen for v1: SQLCipher, DPAPI-wrapped keys, mandatory app unlock password.

## Consequences

- An unlocked Windows profile can read `database.sqlite` and backup packages on disk.
- Backup/restore stays simple (Phase 10).
- Paths and backup manifest remain compatible with a future encryption ADR.
- Phase 12 hardening focuses on CSP / capability audit, not crypto.

## Date

2026-08-27 (Phases 11–13 delivery)
