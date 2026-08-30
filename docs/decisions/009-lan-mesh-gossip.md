# ADR 009 — LAN mesh gossip (no cloud)

## Status

Accepted

## Context

Several shop PCs must share customers, repairs, and tasks on the shop Wi‑Fi only. A designated host PC would fail when that machine is off. Cloud or a hosted relay violates the offline product.

## Decision

Sync is a **LAN mesh**: UDP discovery plus PSK-encrypted TCP. There is no cloud and no star server. Any two online members exchange `sync_changes`. Join uses a shared invite that **any** online member can validate, so the team creator can be offline.

Rust owns sockets (ports UDP `47821`, TCP `47822`). The WebView never opens LAN HTTP and never talks to a localhost API. The UI only `invoke()`s Tauri commands (`list_presence`, `get_sync_status`, team/staff commands).

## Consequences

- Shop Wi‑Fi must allow PC-to-PC traffic. Guest AP isolation will show everyone offline.
- Windows Firewall may prompt on first bind.
- Join still needs at least one online member (the invite row and PSK live on every member after join).
- Compatible app/schema versions are required before applying remote changes.

## Date

2026-08-29
