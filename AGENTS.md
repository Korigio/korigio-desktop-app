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
11. **Always** split non-trivial work across `/phase-orchestrator` → `/backend` → `/frontend` → `/i18n` → `/verifier`. The main chat must not implement full-stack in one pass (exception: tiny typo/docs or explicit user override).

## Specialized agents (mandatory split)

**Always** split feature / phase work across project subagents under `.cursor/agents/`.  
The main chat **must not** implement backend + frontend + i18n in one pass.

| Invoke | Owns |
| --- | --- |
| `/phase-orchestrator` | Phase/side-quest plan, IPC contract, task split, 12-point report (readonly) |
| `/backend` | `src-tauri/**` only |
| `/frontend` | `src/**` UI only (no Rust) |
| `/i18n` | Locale catalogs `en` / `es` / `de` |
| `/verifier` | Run checks; report pass/fail (readonly) |

**Required order for every non-trivial change:**

1. `/phase-orchestrator` — lock scope + IPC contract + who does what  
2. `/backend` — Rust commands, domain, tests  
3. `/frontend` — pages/components/hooks against that contract  
4. `/i18n` — all new user-facing strings in `en`/`es`/`de`  
5. `/verifier` — typecheck / lint / `cargo test` (and build if UI changed)

Exceptions (main chat may do alone): tiny typo fixes, pure docs, or an explicit user “do it in this chat” override.
