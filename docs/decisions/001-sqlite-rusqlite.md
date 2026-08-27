# ADR 001 — SQLite via rusqlite

## Status

Accepted

## Context

The app must be fully local, offline, lightweight, and installable without a separate database server.

## Decision

Use SQLite with the Rust `rusqlite` crate and bundled SQLite.

## Consequences

- No PostgreSQL/MySQL/Docker for customers.
- Single-file DB simplifies backup.
- WAL + foreign keys configured in our connection layer.
- Schema evolves only via migrations.
