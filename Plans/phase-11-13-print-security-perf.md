# Phases 11–13 — Printing + Security + Performance

**Context:** Phases 0–10 done (images + backup on `phase-9-10`). Combined delivery per approval.

**Agent split:** `/phase-orchestrator` → `/backend` → `/frontend` → `/i18n` → `/verifier`

## Locked defaults

| Area          | Decision                                                    |
| ------------- | ----------------------------------------------------------- |
| Print content | Core repair + diagnosis checklist; **no** images            |
| Print tech    | `/repairs/:id/print` + A4 `@media print` + `window.print()` |
| Encryption    | **v1: none + OS account** (ADR 004 closed)                  |
| Perf          | Synthetic seeder + measured baseline; fix only clear misses |

## IPC

### `get_repair_print_report`

```ts
// invoke("get_repair_print_report", { repairId })
type RepairPrintReport = {
  repair: {
    id: number;
    repairNumber: string;
    status: string;
    receivedAt: string;
    reportedProblem: string | null;
    accessoriesReceived: string | null;
    deviceCondition: string | null;
    diagnosisNotes: string | null;
    workPerformed: string | null;
    notes: string | null;
    readyAt: string | null;
    collectedAt: string | null;
  };
  customer: {
    id: number;
    name: string;
    phone: string | null;
    email: string | null;
    address: string | null;
  };
  device: {
    id: number;
    deviceType: string;
    manufacturer: string | null;
    model: string | null;
    serialNumber: string | null;
  };
  diagnosis: {
    items: Array<{
      id: string;
      label: string;
      kind: string;
      value: boolean | string;
    }>;
  } | null;
};
```

### `seed_synthetic_data` (debug / explicit confirm)

```ts
// invoke("seed_synthetic_data", { input: { customers, devices, repairs, confirm: true } })
// Defaults for local measure: 10000 / 20000 / 50000; tests use small counts.
type SeedSyntheticDataInput = {
  customers?: number;
  devices?: number;
  repairs?: number;
  confirm: boolean; // must be true
};
type SeedSyntheticDataResult = {
  customers: number;
  devices: number;
  repairs: number;
  elapsedMs: number;
};
```

## Task split

1. **Backend:** `domain/print`, seed module/command, register commands
2. **Frontend:** print route (no AppShell), Print button, debug seed UI if needed
3. **Security docs:** ADR 004, security.md, capabilities/CSP audit notes
4. **Perf:** `docs/performance-results.md`
5. **i18n + verifier**

## Out of scope

PDF crate · photo thumbs on paper · SQLCipher · FTS5 unless measured miss · Phase 14/15
