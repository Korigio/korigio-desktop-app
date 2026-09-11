# Windows installer (Phase 14)

Korigio ships as a **Windows 10/11 x64** NSIS setup executable with an **embedded offline WebView2 installer** so corporate PCs without internet can install.

## Configuration

Locked in [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json):

| Setting                             | Value                    | Why                                                                      |
| ----------------------------------- | ------------------------ | ------------------------------------------------------------------------ |
| `bundle.targets`                    | `["app", "dmg", "nsis"]` | Customer ship = NSIS on Windows; macOS `.app` + `.dmg` for local testing |
| `windows.webviewInstallMode.type`   | `offlineInstaller`       | No CDN required at install time (~+127 MB)                               |
| `windows.webviewInstallMode.silent` | `true`                   | Quiet WebView2 bootstrap when missing                                    |
| `windows.nsis.installMode`          | `currentUser`            | Typical shop PC without admin elevation                                  |

**Uninstall must not delete AppData** (`%AppData%\com.servioo.desktop` / Tauri app data). Tauri’s default NSIS script leaves application data in place so customer DB, images, and backups survive uninstall/reinstall. Do not add hooks that wipe AppData unless the user explicitly opts in (future ADR).

## Local build on macOS (dev / testing)

```bash
npm ci
npm run tauri build
```

macOS app bundle:

```text
src-tauri/target/release/bundle/macos/Korigio.app
```

macOS disk image (opens the familiar drag-to-Applications window):

```text
src-tauri/target/release/bundle/dmg/Korigio_*.dmg
```

Double-click the `.dmg`, then drag **Korigio** into Applications. Customer delivery remains Windows NSIS only.

## Local / CI build on Windows

```bash
npm ci
npm run tauri build
```

Artifact path (x64):

```text
src-tauri/target/release/bundle/nsis/Korigio_*_x64-setup.exe
```

(Exact filename includes version from `tauri.conf.json`. Cross-compiled paths may use `x86_64-pc-windows-msvc`.)

## Code signing

Authenticode is optional and lives in the **build/release** layer. See [WINDOWS_CODE_SIGNING.md](WINDOWS_CODE_SIGNING.md).

- `npm run tauri dev` never requires a certificate.
- `npm run tauri build` stays unsigned unless a thumbprint is supplied (so macOS and cert-less Windows keep working).
- On Windows, `npm run build:windows` signs via Tauri when `WINDOWS_CERTIFICATE_THUMBPRINT` or `src-tauri/tauri.windows-signing.json` is present, post-signs the exe + NSIS installer (Tauri’s NSIS patch can otherwise leave the exe unsigned), then verifies both.
- GitHub Actions imports a PFX from secrets `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` when those secrets exist.

A **self-signed** cert names the publisher (**Moritz Alexander Wright** / **Korigio**) in the signature. It does **not** clear SmartScreen on customer PCs. Linux AppImage stays unsigned convenience — no install-time publisher dialog.

## Related

- Architecture OS target: [architecture.md](architecture.md)
- CI release: [`.github/workflows/release.yml`](../.github/workflows/release.yml)
- Windows Authenticode: [WINDOWS_CODE_SIGNING.md](WINDOWS_CODE_SIGNING.md)
- Version tags and public downloads: [releases.md](releases.md)
- Backup AppData layout: [backup.md](backup.md) / [database.md](database.md)
