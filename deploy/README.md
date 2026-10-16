# Deployment

Packaging, installation, upgrades, backups, and observability. The
[Deployment Roadmap](../Deployment%20Roadmap.md) is the specification; this
directory holds the artifacts it produces.

Nothing is here yet. What lands, per the roadmap:

- A statically linked Linux binary built with the musl target, so installation
  does not depend on the host libc.
- A distroless container image.
- Installation, upgrade, and rollback procedures.
- Database backup coordination, matching the migration conventions in
  [`migrations/`](../migrations/README.md).
- Observability: structured logs and metrics exposed by the server.

The constraint that governs all of it: a release is Rust binaries with no
non-Rust runtime dependency, and release artifacts are built reproducibly.
