# Production QA checklist (Phase 16)

Use this before handing Korigio to a customer. Check each item on a **Windows 10/11 x64** machine (preferably ≤ 4 GB RAM class hardware).

## Install & first run

- [ ] Install from NSIS `Korigio_*_x64-setup.exe` **with network disconnected** (offline WebView2)
- [ ] App starts; window title **Korigio**; no crash on cold start
- [ ] Cold start feels acceptable (target ≤ ~3 s on mid-range PC)
- [ ] Language follows OS / Settings preference (`es` / `de` / `en`)

## Core workflows

- [ ] Create customer → edit → archive → unarchive
- [ ] Create device linked to customer → search by serial
- [ ] Intake (`Ctrl+Shift+N` / Intake nav): Save & next creates repair with `YYYY-NNNNNN`
- [ ] Change repair status from list and from detail
- [ ] Global search finds by phone, serial, repair number, problem text
- [ ] Diagnosis: create template → apply on repair → save checklist → reopen persists
- [ ] Images: attach JPEG/PNG → thumbs load → caption → delete
- [ ] Print: open repair report → OS print dialog → A4 layout readable
- [ ] Backup: create `.backup` → restore (safety backup created) → data intact
- [ ] Auto backup: after first day / force via restart, `backups/auto/` has an entry

## Security & data

- [ ] AppData under OS Application Support / AppData for `com.servioo.desktop` (not Program Files)
- [ ] Uninstall app → **AppData still present** (DB/images/backups)
- [ ] Reinstall → existing data still opens
- [ ] No customer PII in console/logs during normal use
- [ ] Capabilities: no shell; images only via scoped asset paths

## Performance spot-check (optional load)

- [ ] `/search` remains responsive with a large local dataset (≪ 500 ms feel)
- [ ] Idle CPU near 0%; gallery does not freeze UI when attaching several photos

## Regression / packaging

- [ ] `npm run typecheck` and `npm run lint` clean on release commit
- [ ] `cargo test --lib` green
- [ ] CI workflow `Release` produced Windows / macOS / Linux installers for a `v*` tag (and a GitHub Release)
- [ ] Public download page [korigio.github.io/korigio-downloads](https://korigio.github.io/korigio-downloads/) shows the latest public release when `SERVIOO_RELEASES_TOKEN` is configured

## Sign-off

| Role | Name | Date | Notes |
| --- | --- | --- | --- |
| Builder | | | |
| Reviewer | | | |

Related: [installer.md](installer.md), [security.md](security.md), [performance-results.md](performance-results.md), [roadmap.md](roadmap.md).
