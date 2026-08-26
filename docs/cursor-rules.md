# Cursor rules (install as `.mdc`)

Plan mode blocked writing `.mdc` binaries/extensions. Copy these into `.cursor/rules/` as `.mdc` files when Agent mode is available, or approve a short Agent-mode pass to create them.

---

## project.mdc

```yaml
---
description: Repair Manager project-wide stack, phases, and hard constraints
alwaysApply: true
---
```

See body in repository root guidance: [AGENTS.md](../AGENTS.md) and below.

### Body

- Stack locked: Tauri 2, React, TS, Vite, Rust, SQLite/rusqlite, Tailwind v4, Radix in `src/ui` only, React Router 7, TanStack Form, TanStack Table.
- Forbidden without ADR: Electron, Next.js, MUI, Ant Design, Chakra, localhost HTTP API, React Hook Form, Formik, Redux/MobX.
- Only implement the approved phase; 12-point report; STOP.
- UI never touches SQLite/FS; Tauri commands only.
- One domain = `src/features/<domain>` + `src-tauri/src/domain/<domain>`.
- Offline customer install (`offlineInstaller`); Win10/11 x64 only.
- Propose features; do not implement without permission.
- Prefer performance over visual effects; document new dependencies in phase reports.

---

## react-frontend.mdc

```yaml
---
description: React frontend — Atomic Design, Tailwind tokens, Router, TanStack Form/Table
globs: src/**/*.{ts,tsx,css}
alwaysApply: false
---
```

- Features under `src/features/<domain>/{components,hooks,utils,types,constants,api,pages}`.
- UI library: `src/ui/{atoms,molecules,organisms,templates}` — no domain knowledge.
- Atomic dependency: pages → templates → organisms → molecules → atoms (never upward).
- Hooks for state; utils pure; pages do not `invoke` directly.
- Mandatory: React Router 7, TanStack Form, TanStack Table, Tailwind tokens.
- i18n only for user-facing strings (`en`/`es`/`de`).
- Features import `@/ui`, never `@radix-ui/*`.

---

## rust-backend.mdc

```yaml
---
description: Rust/Tauri backend — commands, services, repositories, errors, SQL
globs: src-tauri/**/*
alwaysApply: false
---
```

- `commands/` thin; `domain/*/service.rs` for rules; `repository.rs` for SQL.
- No `unwrap`/`expect` on recoverable paths; no PII in logs.
- Parameterized SQL; FKs on; repair numbers allocated in transactions.
- Soft archive; least-privilege capabilities; tests use temp DBs only.
