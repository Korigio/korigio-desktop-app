# Performance

## Hardware baseline

Windows 10/11 x64 · 2 cores · **4 GB RAM** · integrated graphics · HDD or inexpensive SSD.

## Engineering targets

| Metric | Target |
| --- | --- |
| Cold start | ≤ 3 s |
| Idle RAM | preferably ≤ 150 MB |
| Idle CPU | ~0% |
| Typical search | well below 500 ms |
| UI interaction | no noticeable lag |
| Installer | offline-capable; size growth from embedded WebView2 accepted |

Treat as targets to measure, not marketing claims.

## Design rules that protect performance

- Do not load the full database into memory at startup.
- Paginate lists in SQL; TanStack Table drives UI page/sort state only.
- Search in SQLite with indexes — never filter large sets only in JS.
- Images: filesystem + thumbnails; lazy-load; do not embed full photos in list queries.
- Add Radix / UI packages only when a screen needs them.
- Prefer quiet visuals (limited blur, shadow, animation).
- Expensive work (backup, image processing) must not freeze the UI thread.

## Measurement

- Phase 1: note `tauri dev` / release start qualitatively.
- Phase 13: synthetic dataset + recorded baselines — see [performance-results.md](performance-results.md).
- Optimize only from measurements.
