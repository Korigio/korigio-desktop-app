# Releases and public downloads

Korigio **source stays private** (`Korigio/korigio-desktop-app`). Visitors download installers from a **public** GitHub repo and Pages site so they never need access to this repository.

| Piece | Where |
| --- | --- |
| Source, CI, version files | this private repo |
| Public installers + download page | [Korigio/korigio-downloads](https://github.com/Korigio/korigio-downloads) |
| Public page | [https://korigio.github.io/korigio-downloads/](https://korigio.github.io/korigio-downloads/) |

Windows 10/11 x64 NSIS remains the supported shop installer. macOS `.dmg` and Linux AppImage are unsigned convenience builds.

## Version bumper

One version in three files: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (and `Cargo.lock`).

```bash
npm run version:bump -- 1.0.0          # set exact
npm run version:bump -- patch          # 1.0.0 → 1.0.1
npm run version:bump -- minor          # 1.0.0 → 1.1.0
npm run version:bump -- major          # 1.0.0 → 2.0.0
npm run version:check                  # all files match
```

Then commit, tag, and push:

```bash
git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: release 1.0.1"
git tag v1.0.1
git push origin HEAD --tags
```

The tag **must** match the files (`v1.0.1` ↔ `1.0.1`) or the Release workflow fails.

## GitHub Actions

`.github/workflows/release.yml` on `v*` tags:

1. Typecheck, lint, `cargo test`, version check
2. Build NSIS / DMG / AppImage
3. GitHub Release on this private repo (for you)
4. Publish the same files to **public** `Korigio/korigio-downloads`
5. Bake [https://korigio.github.io/korigio-downloads/](https://korigio.github.io/korigio-downloads/) from those public releases: latest installers on top, older versions listed below. The same step writes `latest.json` beside `index.html` so the desktop app can check for updates.

Step 4 requires repo secret `SERVIOO_RELEASES_TOKEN` (classic PAT or fine-grained token with `contents: write` on `Korigio/korigio-downloads` only). The job **fails** if that secret is missing — otherwise the download page stays on an old release.

Windows NSIS signing (optional): repository secrets `WINDOWS_CERTIFICATE` (Base64 `.pfx`) and `WINDOWS_CERTIFICATE_PASSWORD`. If both are set, the Windows job Authenticode-signs the exe and NSIS installer, then verifies. If they are absent, the Windows artifact stays unsigned. Details: [WINDOWS_CODE_SIGNING.md](WINDOWS_CODE_SIGNING.md).

Preview the page locally after a public release exists:

```bash
npm run website:build
```

## Metadata

Publisher / author: **Moritz Alexander Wright** (`bundle.publisher`, Cargo `authors`, npm `author`). Identifier stays `com.servioo.desktop` (do not change after shops install).
