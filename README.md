# Repair Manager

Offline Windows desktop app for small repair workshops (Tauri 2 + React + Rust + SQLite).

## Docs

- [Architecture](docs/architecture.md)
- [Coding standards](docs/coding-standards.md)
- [Roadmap](docs/roadmap.md)
- [AGENTS.md](AGENTS.md)

## Development (Phase 1+)

Prerequisites: Node.js 20+, Rust stable, platform WebView (macOS WKWebView / Windows WebView2).

```bash
npm install
npm run tauri dev
```

Other scripts:

```bash
npm run lint
npm run typecheck
npm run format:check
npm run build
```

## Status

Phase 2 — database foundation (SQLite in AppData). No customer UI yet.
