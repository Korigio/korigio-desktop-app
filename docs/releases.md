# Releases and public downloads

Servioo **source stays private** (`M-WRI/cp-reparaciones`). Visitors download installers from a **public** GitHub repo and Pages site so they never need access to this repository.

| Piece | Where |
| --- | --- |
| Source, CI, version files | this private repo |
| Public installers + download page | [M-WRI/servioo](https://github.com/M-WRI/servioo) |
| Public page | [https://m-wri.github.io/servioo/](https://m-wri.github.io/servioo/) |

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
4. If secret `SERVIOO_RELEASES_TOKEN` is set, the same files are published to **public** `M-WRI/servioo` (that is what the download page reads)

Create a classic PAT (or fine-grained token) with `contents: write` on `M-WRI/servioo` only, then add it as repo secret `SERVIOO_RELEASES_TOKEN` on **this** private repo.

## Metadata

Publisher / author: **Moritz Alexander Wright** (`bundle.publisher`, Cargo `authors`, npm `author`). Identifier stays `com.servioo.desktop` (do not change after shops install).
