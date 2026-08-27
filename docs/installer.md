# Windows installer (Phase 14)

Servioo ships as a **Windows 10/11 x64** NSIS setup executable with an **embedded offline WebView2 installer** so corporate PCs without internet can install.

## Configuration

Locked in [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json):

| Setting | Value | Why |
| --- | --- | --- |
| `bundle.targets` | `["app", "dmg", "nsis"]` | Customer ship = NSIS on Windows; macOS `.app` + `.dmg` for local testing |
| `windows.webviewInstallMode.type` | `offlineInstaller` | No CDN required at install time (~+127 MB) |
| `windows.webviewInstallMode.silent` | `true` | Quiet WebView2 bootstrap when missing |
| `windows.nsis.installMode` | `currentUser` | Typical shop PC without admin elevation |

**Uninstall must not delete AppData** (`%AppData%\com.servioo.desktop` / Tauri app data). Tauri’s default NSIS script leaves application data in place so customer DB, images, and backups survive uninstall/reinstall. Do not add hooks that wipe AppData unless the user explicitly opts in (future ADR).

## Local build on macOS (dev / testing)

```bash
npm ci
npm run tauri build
```

macOS app bundle:

```text
src-tauri/target/release/bundle/macos/Servioo.app
```

macOS disk image (opens the familiar drag-to-Applications window):

```text
src-tauri/target/release/bundle/dmg/Servioo_*.dmg
```

Double-click the `.dmg`, then drag **Servioo** into Applications. Customer delivery remains Windows NSIS only.

## Local / CI build on Windows

```bash
npm ci
npm run tauri build
```

Artifact path (x64):

```text
src-tauri/target/release/bundle/nsis/Servioo_*_x64-setup.exe
```

(Exact filename includes version from `tauri.conf.json`. Cross-compiled paths may use `x86_64-pc-windows-msvc`.)

## Code signing

Not configured in v1 (no certificate in repo). Shops that need SmartScreen reputation should sign the NSIS exe with their Authenticode cert via CI secrets later.

## Related

- Architecture OS target: [architecture.md](architecture.md)
- CI release: [`.github/workflows/windows-release.yml`](../.github/workflows/windows-release.yml)
- Backup AppData layout: [backup.md](backup.md) / [database.md](database.md)
