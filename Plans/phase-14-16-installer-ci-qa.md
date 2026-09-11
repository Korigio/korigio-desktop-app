# Phases 14–16 — Installer + CI + Production QA

**Context:** Phases 0–13 done. Combined delivery per explicit approval.

**Branch:** `phase-14-16` from `phase-11-13` (PR #13).

## Locked defaults

| Phase  | Decision                                                                                                             |
| ------ | -------------------------------------------------------------------------------------------------------------------- |
| **14** | NSIS `.exe` only (Windows x64); `webviewInstallMode: offlineInstaller`; do **not** wipe AppData on uninstall         |
| **14** | `installMode: currentUser` (no admin required for typical shop PC); document per-machine option                      |
| **15** | GitHub Actions: build Windows NSIS on `windows-latest` for tags `v*` and manual `workflow_dispatch`; upload artifact |
| **16** | Production QA checklist + security/perf audit doc under `docs/` — no new product features                            |

## Out of scope

Code signing certificates (shop can add later) · auto-update · macOS/Linux installers · cloud release to stores

## Deliverables

1. `tauri.conf.json` Windows NSIS + offline WebView2
2. `docs/installer.md` — how to build/ship locally
3. `.github/workflows/windows-release.yml`
4. `docs/qa-checklist.md` — Phase 16 production QA
5. Roadmap 14–16 **Done**
