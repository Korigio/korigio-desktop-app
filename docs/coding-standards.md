# Coding standards

These rules apply to humans and AI agents. Cursor rules under `.cursor/rules/` enforce the same.

## Phases

- Implement only the current approved phase.
- After each phase: 12-point report + stop until explicit continue.
- No unsolicited features; propose first.
- No unrelated refactors in a phase diff.

## Frontend layout

```text
src/
  root.tsx               # React Router root (Layout, Outlet, ErrorBoundary)
  routes.ts              # RouteConfig (index / route / layout / prefix)
  routes/                # thin route modules only
  app/providers/         # app-wide providers (i18n, …)
  features/<domain>/     # domain module
  ui/                    # Atomic Design library (no domain)
  shared/                # invoke, cn, shared types
  i18n/                  # en, es, de
  styles/                # tokens + Tailwind globals
```

### Routing (mandatory)

- Use React Router 7 **Framework Mode** with `src/routes.ts` + `@react-router/dev/routes`.
- Pattern: `index(...)`, `route(...)`, `layout(...)`, `prefix(...)` as in the [official routing docs](https://reactrouter.com/start/framework/routing).
- Route modules in `src/routes/` stay thin; feature UI lives in `features/*/pages`.
- Do not use `RouterProvider` / `createBrowserRouter` / `createHashRouter` for primary app routing.
- SPA mode: `react-router.config.ts` → `ssr: false`. Use `clientLoader` / `clientAction` for route data.

### Feature module shape

```text
features/<domain>/
  components/    # domain organisms
  hooks/         # state and side effects
  utils/         # pure non-state helpers
  types/
  constants/
  api/           # thin wrappers around shared invoke
  pages/         # route targets
  index.ts
```

### Atomic Design

| Level | Path | Domain? |
| --- | --- | --- |
| Atoms | `ui/atoms/` | No |
| Molecules | `ui/molecules/` | No |
| Organisms (generic) | `ui/organisms/` | No |
| Organisms (domain) | `features/*/components/` | Yes |
| Templates | `ui/templates/` | No |
| Pages | `features/*/pages/` | Yes |

Dependency direction: `pages → templates → organisms → molecules → atoms` (never upward).  
`ui` never imports `features`. Features never import `@radix-ui/*` (only `@/ui`).

### Hooks and utils

- Components stay thin.
- Hooks own state / effects / form instances / table instances.
- Utils are pure (no React, no `invoke`).

### Mandatory libraries

| Concern | Must use | Must not use instead |
| --- | --- | --- |
| Routing | React Router 7 Framework Mode (`src/routes.ts`) | `RouterProvider` / ad-hoc primary nav switches |
| Forms | TanStack Form | React Hook Form, Formik, sprawling useState forms |
| Tables | TanStack Table | Hand-rolled grids for data tables |
| Styling | Tailwind v4 + tokens | MUI, Ant Design, Chakra as app kit |

Client-side form checks are UX only; Rust always re-validates.

### i18n

No hard-coded user-facing strings in components. Use `en` / `es` / `de` catalogs. Default locale: Spanish until changed.

### TypeScript

Strict mode. No `any`. DTOs align with Rust command payloads.

## Backend layout

```text
src-tauri/src/
  commands/           # thin IPC (= routes/controllers)
  domain/<name>/      # constants, types, validation, service, repository
  db/                 # connection, migrate
  error.rs
  paths.rs
  shared/
```

- Commands: deserialize → call service → map errors.
- Services: business rules and transactions.
- Repositories: parameterized SQL only.
- Domains call other domains via **services**, not foreign repositories.
- No `unwrap()` / `expect()` on recoverable paths.
- User-facing errors are safe; technical detail goes to logs (without PII).

## Dependencies

Before adding any dependency, document in the phase report: purpose, approximate impact, why native/existing stack was insufficient. Prefer not inventing crypto or DB engines.

## Commits (when requested)

Prefer small commits with `feat:`, `fix:`, `refactor:`, `test:`, `build:`, `docs:` prefixes.
