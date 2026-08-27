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
| 11 | Printing | First A4 report |
| 12 | Security hardening | Capabilities, CSP, encryption decision |
| 13 | Performance hardening | Synthetic data + measure + fix |
| 14 | Windows installer | NSIS offline WebView2 setup.exe |
| 15 | GitHub Actions | Windows release artifact |
| 16 | Production QA | Checklist and audit |

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
