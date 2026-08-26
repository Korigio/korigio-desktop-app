# ADR 007 — React Router 7 + TanStack Form + TanStack Table

## Status

Accepted

## Context

Routing, forms, and tables must follow one standard so AI and humans do not invent parallel patterns.

## Decision

| Concern | Standard |
| --- | --- |
| Routing | React Router 7 — route tree in `src/app/routes.tsx` |
| Forms | `@tanstack/react-form` for all create/edit forms |
| Tables | `@tanstack/react-table` for all data tables |

- Do not use React Hook Form, Formik, or ad-hoc primary navigation switches.
- TanStack libs are headless; chrome lives in `ui/`; domain column/field defs live in features.
- Prefer SQL/server-side pagination; TanStack owns UI table state.
- Client validators are UX-only; Rust remains authoritative.

## Consequences

- Phase 1 wires Router with a placeholder route.
- Phase 3 establishes canonical Form + Table examples to copy afterward.
- Extra dependencies are bundled into the app at build time (no customer internet required).
