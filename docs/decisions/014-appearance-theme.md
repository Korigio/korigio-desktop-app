# ADR 014 — Appearance (system / light / dark)

## Status

Accepted

## Context

Servioo previously had a single light token set. The workshop needs a ClickUp-inspired look that follows the computer’s light/dark appearance, with an explicit override like language. Theme is a per-PC preference, not shop data.

## Decision

- Persist `theme_preference` in the local SQLite `settings` table: `system` | `light` | `dark`. Default `system`.
- Do **not** replicate appearance via LAN sync (`sync_changes`).
- Resolve in the UI: `class="dark"` on `<html>` plus Tailwind v4 `@custom-variant dark`. When preference is `system`, follow `prefers-color-scheme` and listen for OS changes.
- Do not add a Rust crate for OS theme detection.
- Print views stay forced light (existing print CSS).
- Tokens stay in `src/styles/tokens.css` (ADR 006). Quiet visuals: no card drop-shadow, no header backdrop blur.

## Consequences

- Settings → General exposes Appearance next to language.
- Changing appearance updates the UI immediately without restart.
- Team PCs can disagree on light/dark; that is intended.
