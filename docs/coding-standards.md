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
- SPA mode: `react-router.config.ts` → `ssr: false` and `routeDiscovery: { mode: "initial" }` (no runtime `/__manifest`). Use `clientLoader` / `clientAction` for route data.

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

| Level               | Path                     | Domain? |
| ------------------- | ------------------------ | ------- |
| Atoms               | `ui/atoms/`              | No      |
| Molecules           | `ui/molecules/`          | No      |
| Organisms (generic) | `ui/organisms/`          | No      |
| Organisms (domain)  | `features/*/components/` | Yes     |
| Templates           | `ui/templates/`          | No      |
| Pages               | `features/*/pages/`      | Yes     |

Dependency direction: `pages → templates → organisms → molecules → atoms` (never upward).  
`ui` never imports `features`. Features never import `@radix-ui/*` (only `@/ui`).

### Thin pages (mandatory)

`features/*/pages` are composition layers. A page should read like a storyboard of **named** parts, not raw layout markup.

| Extract to                                  | When                                                                                           |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `ui/atoms\|molecules\|organisms\|templates` | Generic chrome reused across domains (`Page`, `PageHeader`, `SearchField`, `PaginationBar`, …) |
| `features/<domain>/components`              | Domain meaning (`CustomerListFilters`, `CustomerDetailFields`)                                 |

**Forbidden in pages:** copy-pasted title/subtitle blocks, search toolbars, pagination bars, definition lists, button-styled `<Link>` className strings, repeated loading/error paragraphs.

**Required:** if markup repeats (or will repeat on the next domain), extract immediately with a meaningful name.

### Hooks and utils

- Components stay thin.
- Hooks own state / effects / form instances / table instances.
- Utils are pure (no React, no `invoke`).

### Module cohesion and extraction

- A page, hook, or service façade may coordinate a workflow and preserve its public contract; independently testable responsibilities belong in focused modules behind it.
- Extract state/effects and lifecycle orchestration to focused hooks or services; pure validation, mapping, formatting, transitions, and derivation to `utils`; shared DTOs and contracts to `types`; reusable markup to named components.
- Review the boundary before adding a third distinct concern, when a file approaches roughly **400 lines**, or when a change grows it by more than **100 lines**. These are advisory review triggers, not pass/fail limits: cohesion and dependency direction decide whether to extract.
- Prefer a small number of substantial responsibility modules. Do not create one-helper-per-file trees or move code solely to reduce a line count.
- Keep the existing façade/import path when consumers rely on it. Extractions must not silently change IPC payloads, hook return shapes, routes, persistence, errors, timing, or user-facing copy.
- Move existing tests with the responsibility they cover. Add focused tests for newly exposed pure seams and run the façade's regression/integration tests to prove the public contract is unchanged.

### DRY and reuse (mandatory)

Agents must deduplicate before marking work complete:

1. **Pure helpers** used in more than one file → `utils/` (domain) or `shared/`.
2. **Repeated JSX** across sibling components → shared component (e.g. `features/print/components/shared/`).
3. **Repeated async modal state** (`busy`, `uploading`, `error`, confirm handler) → shared hook (`useModalAsyncAction`, domain-specific upload hooks).
4. **Same modal flow, different copy/API** → one parameterized component; thin named exports for i18n labels only.

Do not leave duplicate `dash`-style helpers, print headers, or copy-pasted confirm/upload handlers in multiple files.

### Mandatory libraries

| Concern | Must use                                        | Must not use instead                              |
| ------- | ----------------------------------------------- | ------------------------------------------------- |
| Routing | React Router 7 Framework Mode (`src/routes.ts`) | `RouterProvider` / ad-hoc primary nav switches    |
| Forms   | TanStack Form                                   | React Hook Form, Formik, sprawling useState forms |
| Tables  | TanStack Table                                  | Hand-rolled grids for data tables                 |
| Styling | Tailwind v4 + tokens                            | MUI, Ant Design, Chakra as app kit                |

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
- Large services remain thin façades over cohesive internal modules (for example validation, persistence, scheduling, transport, or restore orchestration); internal modules stay private unless a public contract requires otherwise.
- Domains call other domains via **services**, not foreign repositories.
- No `unwrap()` / `expect()` on recoverable paths.
- User-facing errors are safe; technical detail goes to logs (without PII).

## Dependencies

Before adding any dependency, document in the phase report: purpose, approximate impact, why native/existing stack was insufficient. Prefer not inventing crypto or DB engines.

## Commits (when requested)

Prefer small commits with `feat:`, `fix:`, `refactor:`, `test:`, `build:`, `docs:` prefixes.
