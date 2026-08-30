# ADR 010 — Hybrid logical clocks and last-write-wins

## Status

Accepted

## Context

Two people may edit the same customer or repair while offline, then both return to Wi‑Fi. Wall clocks on shop PCs drift. A conflict screen is extra UX on low-end hardware. The product choice is automatic last save wins.

## Decision

Concurrent edits use a **hybrid logical clock** `(hlc_wall_ms, hlc_counter, origin_device_id)`. Compare: higher wall wins; if equal, higher counter; if equal, lexicographically greater `device_id`. The later HLC replaces the row. There is **no conflict UI**.

On local write: `wall = max(local_hlc.wall, now_ms)`; if `wall` equals the previous local wall then `counter += 1`, else `counter = 0`. Stamp `origin_device_id` as this PC.

Equal triples are treated as the same write (idempotent). Older remote changes are ignored but acknowledged.

## Consequences

- An offline edit of the **same field** can be discarded if another PC saved later.
- Causality is preserved better than raw timestamps when clocks are wrong.
- Apply logic must be identical on every peer.

## Date

2026-08-29
