# Performance results (Phase 13)

Hardware context for these numbers: developer Mac (Apple Silicon) running `cargo test` against an in-memory SQLite DB. Treat as **relative baseline**, not the Windows 4 GB target machine. Re-run on Windows release builds before claiming installer targets.

Targets from [performance.md](performance.md): typical search ≪ **500 ms**; optimize only from measurements.

## Synthetic seed

| Scale                  | Customers | Devices | Repairs | Seed wall time                                           |
| ---------------------- | --------- | ------- | ------- | -------------------------------------------------------- |
| Medium (CI regression) | 1 000     | 2 000   | 5 000   | **~71 ms** (`search_timing_on_medium_seed`)              |
| Full measure (manual)  | 10 000    | 20 000  | 50 000  | `cargo test` seed helpers only — not exposed in Settings |

## Search

| Scenario                                               | Result                                       |
| ------------------------------------------------------ | -------------------------------------------- |
| `global_search` query `Customer 500` after medium seed | **~9 ms**; hits returned                     |
| vs target ≪ 500 ms                                     | **Pass** — no FTS5 or index rewrite required |

Regression guard: `domain::seed::tests::search_timing_on_medium_seed` fails if search ≥ 1000 ms on that seed.

## Cold start / RAM / DB size

| Metric          | Phase 13 note                                                                   |
| --------------- | ------------------------------------------------------------------------------- |
| Cold start      | Qualitative only this pass — confirm ≤ 3 s on Windows release in Phase 14/16 QA |
| Idle RAM        | Not instrumented in CI; prefer ≤ 150 MB on target hardware during QA            |
| On-disk DB size | Depends on AppData seed; measure after full seed on a real profile              |

## Fixes applied from measurements

**None.** Search was already well under target on a 5k-repair seed; no speculative FTS5.

## How to re-measure

```bash
cd src-tauri && cargo test --lib search_timing_on_medium_seed -- --nocapture
```

Then exercise `/search` on a large local database if you have one.
