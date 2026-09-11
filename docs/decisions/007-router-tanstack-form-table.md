# ADR 007 — React Router 7 Framework Mode + TanStack Form + TanStack Table

## Status

Accepted (updated)

## Context

Routing, forms, and tables must follow one standard so AI and humans do not invent parallel patterns. React Router 7 Framework Mode documents routes via `RouteConfig` (`index`, `route`, `layout`, `prefix`) rather than hand-built `createBrowserRouter` / `RouterProvider` trees.

## Decision

| Concern     | Standard                                                                                                        |
| ----------- | --------------------------------------------------------------------------------------------------------------- |
| Routing     | **React Router 7 Framework Mode** — `src/routes.ts` with `RouteConfig` from `@react-router/dev/routes`          |
| SPA (Tauri) | `react-router.config.ts` with `ssr: false` and `routeDiscovery: { mode: "initial" }` (no runtime `/__manifest`) |
| Forms       | `@tanstack/react-form` for all create/edit forms                                                                |
| Tables      | `@tanstack/react-table` for all data tables                                                                     |

### Routing rules

- Define all app routes in [`src/routes.ts`](../../src/routes.ts) using `index`, `route`, `layout`, and `prefix`.
- Do **not** use `createBrowserRouter` / `createHashRouter` / `RouterProvider` for primary navigation.
- Do **not** use ad-hoc view switches (`if (tab === ...)`) for primary navigation.
- Keep **thin route modules** under `src/routes/` that render feature pages from `src/features/*/pages`.
- Prefer `clientLoader` / `clientAction` for route data in SPA mode (no non-root server `loader` unless we adopt pre-rendering later).
- Build/dev via `react-router dev` / `react-router build`; Tauri serves `build/client`.

### Forms / tables

- TanStack libs are headless; chrome lives in `ui/`; domain column/field defs live in features.
- Prefer SQL/server-side pagination; TanStack owns UI table state.
- Client validators are UX-only; Rust remains authoritative.
- Do not use React Hook Form, Formik, or custom mega-form frameworks.

## Consequences

- Slightly more tooling (`@react-router/dev`, typegen, `root.tsx` conventions).
- Route URLs and nesting stay declarative and reviewable in one file.
- Phase 3 establishes canonical Form + Table examples to copy afterward.
