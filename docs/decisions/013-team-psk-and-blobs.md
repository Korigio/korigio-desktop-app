# ADR 013 — Team PSK, invite codes, and content-hash blobs

## Status

Accepted

## Context

LAN frames must not be readable by a neighbor shop on the same Wi‑Fi. Repair photos and documents live on disk, not in SQLite, so row gossip alone would leave broken images on other PCs. At-rest SQLite encryption stays deferred ([ADR 004](004-encryption-deferred.md)).

## Decision

The team shares one 32-byte **PSK**, stored only on `local_identity` of members (never returned to the UI, never put on UDP). TCP frames use HKDF-SHA256 then **XChaCha20-Poly1305**. Invite codes (`XXXX-XXXX`) are hashed in SQLite and used only as a **join bootstrap** secret (separate HKDF) until the member grants the team PSK.

Files sync by **SHA-256 content hash**. Canonical bytes live under `blobs/{hash[0:2]}/{hash}`. Display paths use the repair or company UUID (`images/{repairUuid}/…`, thumbs, documents, company logos). Entity rows store both `content_hash` and the relative display path.

## Consequences

- Join still needs one online member to complete the snapshot + PSK grant.
- Missing blobs are requested with `BlobWant` after rows apply.
- No at-rest DB encryption; an unlocked Windows profile can still read local files.

## Date

2026-08-29
