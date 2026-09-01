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

Solo (no team): no required login. Optional named staff with PIN.

In a team: mutating commands require a signed-in staff session (PIN, 4–8 digits). Roles are `admin` and `staff` (honest-client; see [ADR 011](decisions/011-staff-roles-honest-client.md)). Team LAN traffic uses a PSK and XChaCha20-Poly1305 ([ADR 013](decisions/013-team-psk-and-blobs.md)). Never log PINs, PSK, or customer PII.

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
| `opener:allow-open-url` (scoped) | Open the in-app feedback `mailto:info@korigio.com*` and HTTPS download links under `github.com/Korigio/korigio-downloads/*` and `korigio.github.io/korigio-downloads/*`. No `opener:default` and no `open-path`. |

No shell, HTTP plugin, or broad filesystem plugin. Asset protocol scope is limited to `$APPDATA/images/**` and `$APPDATA/thumbs/**` in `tauri.conf.json`.

Update checks use **reqwest in Rust only** (GET `https://korigio.github.io/korigio-downloads/latest.json`). The WebView CSP is unchanged; there is no `connect-src` for that host.

## CSP

`default-src 'self'`; images allow `asset:` / localhost asset hosts / `data:` / `blob:` for thumbs; scripts `'self'` only; styles `'self' 'unsafe-inline'` (tokenized Tailwind / component styles).

## Installer (Phase 14)

Offline WebView2 bootstrapper so corporate PCs without CDN access can install. See [installer.md](installer.md). Uninstall must **not** delete AppData (customer data).
