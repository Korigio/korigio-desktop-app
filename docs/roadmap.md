# Development roadmap

Work in **strict phases**. After each phase: deliver the 12-point report, then **STOP** until explicit approval.

Do not combine phases without permission. Do not implement unsolicited features.

| Phase | Name | Outcome |
| --- | --- | --- |
| 0 | Requirements and architecture | Docs, ADRs, Cursor rules (this phase) |
| 1 | Minimal skeleton | **Done** — Tauri + React + Vite + Tailwind tokens + Router placeholder |
| 2 | Database foundation | **Done** — SQLite, migrations, AppData, connection, tests |
| 3 | Customer domain | **Done** — CRUD, search, archive, Form + Table examples |
| 4 | Device domain | **Done** — devices linked to customers, serial search, archive |
| 5 | Repair order domain | **Done** — numbers, status, core fields |
| 6 | Repair workflow UI | **Done** — keyboard intake flow, search comboboxes, Save & next |
| 7 | Search | **Done** — global search, multi-field repairs list, indexes |
| 8 | Diagnosis templates | **Done** — reusable checklists, apply on repair |
| 9 | Images | **Done** — FS storage, thumbs, lazy load |
| 10 | Backup and restore | **Done** — Manual + validation + safety backup |
| 11 | Printing | **Done** — first A4 report (core + diagnosis) |
| 12 | Security hardening | **Done** — CSP/capabilities audit; no at-rest encryption for v1 |
| 13 | Performance hardening | **Done** — synthetic seed + measured baseline |
| 14 | Windows installer | **Done** — NSIS offline WebView2 setup.exe |
| 15 | GitHub Actions | **Done** — Windows release artifact workflow |
| 16 | Production QA | **Done** — Checklist and audit |
| 17 | Local WiFi team sync | **Done** — UUID PKs, staff/roles, LAN mesh gossip, no cloud |

## Required output after every phase

1. What was implemented  
2. Architecture decisions  
3. Files created  
4. Files changed  
5. Code areas to review carefully  
6. How to test  
7. Expected behavior  
8. Known limitations  
9. Performance impact  
10. Dependency changes  
11. Security considerations  
12. `PHASE COMPLETE.` stop line  
