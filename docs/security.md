# Security

## Principles

- Validate all command inputs in Rust.
- Parameterized SQL only (no string-built queries with user data).
- Least-privilege Tauri capabilities; no shell access from the UI.
- Content Security Policy for the WebView.
- Never commit secrets.
- Never log customer PII (names, phones, emails, addresses, serials, free-text notes).

## Data location

All mutable data lives under the OS application data directory (AppData). Install directory is read-only from the app’s perspective.

## Authentication

Single-workstation tool for v1: **no required login**. Optional unlock later only with an ADR.

## Encryption (Phase 12)

**v1 decision:** no at-rest SQLite encryption. Rely on OS account protection (and BitLocker when the shop enables it). See [ADR 004](decisions/004-encryption-deferred.md).

Backups stay portable/unencrypted until a future ADR. Do not invent cryptography.

## Capabilities audit (Phase 12)

[`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json) grants only:

| Permission | Why |
| --- | --- |
| `core:default` | Required Tauri window/event IPC |
| `core:path:default` | Path helpers for AppData / `convertFileSrc` |
| `dialog:default` / `allow-open` / `allow-save` | File pickers for images + backup create/restore |

No shell, HTTP, or broad filesystem plugin. Asset protocol scope is limited to `$APPDATA/images/**` and `$APPDATA/thumbs/**` in `tauri.conf.json`.

## CSP

`default-src 'self'`; images allow `asset:` / localhost asset hosts / `data:` / `blob:` for thumbs; scripts `'self'` only; styles `'self' 'unsafe-inline'` (tokenized Tailwind / component styles).

## Installer (Phase 14)

Offline WebView2 bootstrapper so corporate PCs without CDN access can install.
