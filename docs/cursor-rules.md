# Cursor rules mirror

Canonical rules live in `.cursor/rules/*.mdc`. This file is a readable mirror.

## Specialized subagents

Project agents live in `.cursor/agents/`:

| File | Invoke | Role |
| --- | --- | --- |
| `frontend.md` | `/frontend` | React/TS under `src/` only |
| `backend.md` | `/backend` | Rust/SQLite under `src-tauri/` only |
| `i18n.md` | `/i18n` | `en` / `es` / `de` catalogs |
| `verifier.md` | `/verifier` | typecheck / lint / `cargo test` report |
| `phase-orchestrator.md` | `/phase-orchestrator` | Phase split + 12-point report |

Nested notes: [`src/AGENTS.md`](../src/AGENTS.md), [`src-tauri/AGENTS.md`](../src-tauri/AGENTS.md).

## project.mdc

- Stack: Tauri 2, React, TS, Vite, Rust, SQLite/rusqlite, Tailwind v4, Radix in `src/ui` only, React Router 7 **Framework Mode**, TanStack Form, TanStack Table.
- Forbidden without ADR: Electron, Next.js, MUI, Ant Design, Chakra, localhost HTTP API, React Hook Form, Formik, Redux/MobX, `RouterProvider` / `createBrowserRouter` for primary routing.
- Phase stop gates; AppData-only mutable data; offline installer; Win10/11 x64.

## react-frontend.mdc

- Features under `src/features/<domain>/…`
- Routes: `src/routes.ts` with `index` / `route` / `layout` / `prefix`; thin modules in `src/routes/`
- Atomic Design; hooks for state; utils pure
- TanStack Form/Table; Tailwind tokens; i18n `en`/`es`/`de`

## rust-backend.mdc

- commands → services → repositories
- No recoverable `unwrap`; no PII logs; parameterized SQL
