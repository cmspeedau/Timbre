# Database migrations

The convention is defined by
[ADR-0004](../adr/0004-persistence-and-migrations.md). This summary is for people
adding a migration.

- **Store:** SQLite in WAL mode, accessed through `sqlx`.
- **One history:** every migration in the project lives in this directory, so the
  schema history is linear no matter which crate needs the change.
- **Naming:** `YYYYMMDDHHMMSS_description.sql`, for example
  `20261016120000_create_media_items.sql`. Applied in filename order.
- **Forward only:** never edit a committed migration. Correct it with a new one.
- **Applied at startup** by the server, before it serves traffic. A failed
  migration stops the process.
- **Destructive changes** — dropping or rewriting data — state their migration
  notes in the accompanying change, following the contract change process in the
  Master Roadmap.

There are no migrations yet. The first lands with Phase 1, when the library
schema is created.
