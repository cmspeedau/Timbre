# ADR-0004: Persistence and database migration conventions

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The Master Roadmap fixes the default store: SQLite in WAL mode, accessed through
`sqlx`, for a single-node deployment, with any other database as an optional
adapter behind the storage layer. It also requires that migrations be
backward-compatible where required.

That leaves the mechanics — where migrations live, how they are named, and who
applies them — to be decided once and followed everywhere.

## Decision

- **Default store:** SQLite in WAL mode via `sqlx`.
- **Location:** a single top-level `migrations/` directory, shared by the
  workspace, so schema history is linear regardless of which crate touches it.
- **Naming:** `YYYYMMDDHHMMSS_description.sql`, for example
  `20261016120000_create_media_items.sql`. Timestamps sort lexicographically, so
  the applied order is the filename order.
- **Forward only:** migrations are applied in order and are never edited after
  being committed. A correction is a new migration.
- **Applied at startup** by the server, before it serves traffic, so a failed
  migration stops the process rather than leaving a half-migrated schema.
- **Compatibility:** a migration that drops or rewrites data must state its
  migration notes in the accompanying change, per the Master Roadmap's contract
  change process.
- **Alternative databases** are adapters behind the storage layer and must not
  change the SQL in `migrations/` in ways that break SQLite.

## Consequences

There is one place to look for schema history, and the order is unambiguous. The
startup migration step makes a broken schema a startup failure, which is easier
to detect than a runtime error.

Because migrations are forward only, a bad migration must be corrected by a
follow-up rather than a rewrite — the cost of an append-only history, accepted
for its safety.
