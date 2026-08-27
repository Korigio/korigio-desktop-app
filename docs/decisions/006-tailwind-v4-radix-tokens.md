# ADR 006 — Tailwind v4 + Radix wrappers + design tokens

## Status

Accepted

## Context

We need reusable accessible components and a token-driven theme, while keeping ownership of the design system. The original prompt preferred plain CSS Modules and discouraged large UI kits and Tailwind; product owner explicitly chose Tailwind and wrap-own primitives.

## Decision

- Style with **Tailwind CSS v4** (`@theme` + CSS variables in `styles/tokens.css`).
- Behavior from **Radix** primitives, wrapped only inside `src/ui/**` (shadcn-style ownership).
- Features import `@/ui` only — never `@radix-ui/*`.
- Reject MUI / Ant Design / Chakra as the application UI kit.
- Icons via `lucide-react` as needed (tree-shake); `clsx` + `tailwind-merge` for class merging.
- Keep visuals quiet (limited blur/shadow/animation) for older PCs.
- Add Radix packages only when a screen needs them.

## Consequences

- Theme changes primarily edit tokens.
- Installer/runtime does not fetch CSS/JS from CDNs.
- Slightly more frontend weight than raw CSS Modules; acceptable under the 4 GB RAM target if disciplined.
