# AGENTS.md — Repair Manager

AI agents: read this and `docs/` before changing code.

## Product

Offline Windows desktop repair-shop app (Tauri 2 + React/TS + Rust + SQLite). Working name: Repair Manager.

## Source of truth

- [docs/architecture.md](docs/architecture.md)
- [docs/coding-standards.md](docs/coding-standards.md)
- [docs/roadmap.md](docs/roadmap.md)
- [docs/decisions/](docs/decisions/)
- `.cursor/rules/` (project, frontend, backend)

## Hard rules

1. Only implement the **currently approved phase**. Stop when the phase ends.
2. No unsolicited features — propose first.
3. UI ↔ Rust only via Tauri commands; no localhost HTTP API.
4. Business rules and trusted validation in Rust.
5. Atomic Design + feature modules; hooks for state; utils for non-state. Pages stay thin: compose named `ui` / feature components — no copy-pasted headers, filters, pagination, or status chrome.
6. Tailwind v4 tokens; Radix only inside `src/ui`.
7. React Router 7 Framework Mode (`src/routes.ts` with `index`/`route`/`layout`/`prefix`); TanStack Form; TanStack Table.
8. i18n for all user-facing strings (`en`, `es`, `de`).
9. Performance over visual effects; Windows 10/11 x64 + 4 GB RAM target.
10. Never commit secrets; never log customer PII.
