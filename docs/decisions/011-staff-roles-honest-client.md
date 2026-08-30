# ADR 011 — Staff identity and admin/staff roles (honest client)

## Status

Accepted

## Context

The shop needs named people (who changed a record, who took over a repair) and two roles: **admin** and **staff**. There is no cloud identity provider. Every PC has a full copy of the database.

## Decision

Staff are named, PIN-gated identities (4–8 digits) on a PC. Roles are `admin` and `staff`. Creating a team makes the signed-in person the first admin. Invited people join as staff. Only an admin may invite, kick a device, or change roles. Rust rejects removing or demoting the last active admin.

Team-mode writes require a signed-in session. Solo mode (no team) does not require a session.

This is **honest-client** security: a tampered binary on a staff PC could ignore checks locally. Honest peers still reject role-change `sync_changes` unless the actor was admin. This is not bank-grade auth, SSO, or a substitute for OS account protection.

## Consequences

- PIN hashes are replicated (salt + SHA-256). Never log PINs or return hashes to the UI.
- The always-visible roster shows PC name, signed-in staff, and role.
- Assign / take-over of repairs is allowed for any signed-in staff in a team.

## Date

2026-08-29
