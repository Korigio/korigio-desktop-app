# Architecture — Korigio

Product name: **Korigio** (npm package `servioo`; Tauri identifier `com.servioo.desktop`).

## Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri 2 (Windows NSIS installer) |
| UI | React + TypeScript + Vite |
| Styling | Tailwind CSS v4 + design tokens |
| UI primitives | Radix (wrapped in our `src/ui`), shadcn-style ownership |
| Routing | React Router 7 **Framework Mode** (`src/routes.ts` RouteConfig) |
| Forms / tables | TanStack Form + TanStack Table (mandatory) |
| Backend | Rust |
| Database | SQLite via `rusqlite` |
| IPC | Tauri commands only (no local HTTP server). Native → UI events: `open-settings`, `open-repair-intake`, `sync-applied`. |

## Layers

```text
React features / ui
        ↓ shared/api invoke
Tauri commands (thin)
        ↓
domain services (validation + business rules)
        ↓
repositories (SQL) / filesystem (images, backups)
        ↘
sync_net (UDP discovery + encrypted TCP mesh) — Rust only, no HTTP
        ↓
AppData: SQLite, images, thumbs, documents, blobs, backups, logs
```

## Non-negotiable rules

1. UI never accesses SQLite or the filesystem directly.
2. Authoritative validation and business rules live in Rust.
3. One domain = one `src/features/<domain>` + one `src-tauri/src/domain/<domain>`.
4. `src/ui` has no domain knowledge; features never import `@radix-ui/*` directly.
5. No Electron, Next.js, MUI, Ant Design, or localhost API.
6. Work in numbered phases; stop after each until explicitly approved.

## Target layout (Phase 1+)

```text
src/                     # React frontend (RR appDirectory)
  root.tsx
  routes.ts              # RouteConfig (index / route / layout / prefix)
  routes/                # thin route modules
  app/providers/
  features/<domain>/     # pages, components, hooks, utils, types, api, constants
  ui/                    # Atomic Design: atoms, molecules, organisms, templates
  shared/                # invoke helper, cn(), shared types
  i18n/                  # en, es, de
  styles/                # tokens.css, globals.css
src-tauri/               # Rust backend
  src/commands/          # IPC adapters (= routes)
  src/domain/<domain>/   # constants, types, validation, service, repository
  src/sync_net/          # LAN mesh (UDP/TCP); never a localhost HTTP API
  src/db/                # connection, migrations
  migrations/
docs/                    # architecture and ADRs
```

See [coding-standards.md](coding-standards.md) for file placement and dependency rules.

## OS and install

- Windows 10/11 x64 only (not XP/7).
- Customer install and runtime: **offline** (NSIS `webviewInstallMode: offlineInstaller`).
- All npm/Cargo dependencies are baked into the binary at build time.
- Mutable data only under OS AppData (never under Program Files).

## Related docs

- [database.md](database.md)
- [security.md](security.md)
- [backup.md](backup.md)
- [performance.md](performance.md)
- [installer.md](installer.md)
- [qa-checklist.md](qa-checklist.md)
- [roadmap.md](roadmap.md)
- [decisions/](decisions/)
