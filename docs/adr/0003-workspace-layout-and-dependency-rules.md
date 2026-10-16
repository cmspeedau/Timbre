# ADR-0003: Workspace layout, naming, and dependency rules

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The modular monolith in [ADR-0002](0002-rust-only-modular-monolith.md) only stays
modular if the crate boundaries are real and the dependency direction is fixed.
Without a stated rule, subsystems drift into mutual dependencies and the
"monolith" loses its structure.

The workspace also needs a naming convention so package names are consistent with
the project name, Timbre.

## Decision

### Layout

One Cargo workspace. Members are `crates/*` and `apps/*`.

| Crate | Owns |
| --- | --- |
| `timbre-shared-types` | Identifiers, error codes, job status, subsystem names. |
| `timbre-api-contracts` | Versioned models that travel over the API. |
| `timbre-server` | API, configuration, orchestration, authentication. |
| `timbre-library` | Scanning, metadata, database, artwork, indexing. |
| `timbre-audio-engine` | Decoding, processing, rendering, signal path. |
| `timbre-devices` | Output discovery, zones, endpoints, synchronization. |
| `timbre-discovery` | Metadata enrichment, search, relationships, recommendations. |
| `timbre-remote-access` | Pairing, secure connectivity, remote sessions. |
| `timbre-integrations` | Streaming providers and external protocol adapters. |
| `timbre-ui` | Shared user interface components for the clients. |
| `timbre-testing` | Cross-system contract, regression, performance, audio tests. |
| `timbre-desktop`, `timbre-mobile` | Client application binaries. |

Directory names omit the prefix (`crates/server/`); package names carry it
(`timbre-server`). `deploy/` is not a crate.

### Naming

- Package names use the `timbre-` prefix.
- Internal dependencies are declared once in `[workspace.dependencies]` and
  referenced as `dep = { workspace = true }`.

### Dependency rules

1. `timbre-shared-types` depends on no other internal crate.
2. `timbre-api-contracts` depends only on `timbre-shared-types`.
3. Subsystem crates depend only **downward**: on `timbre-shared-types` and
   `timbre-api-contracts`, and on other subsystem crates only where the Master
   Roadmap defines the edge (for example `timbre-server` coordinating the
   library and playback subsystems).
4. Subsystem crates must never depend on `timbre-server` or on a client crate.
5. `timbre-testing` and `apps/*` may depend on any crate; nothing may depend on
   them.
6. The dependency graph is acyclic. A new edge that would create a cycle
   requires an ADR.
7. Third-party dependency versions are agreed in `[workspace.dependencies]`, not
   chosen per crate.

Ownership follows the crate boundary and is recorded in `CODEOWNERS`.

## Consequences

The build order and the direction of knowledge flow are explicit, and a violation
fails to compile rather than being caught in review. Adding an edge is a visible,
reviewable act.

The cost is that shared needs must be pushed into `timbre-shared-types` or
`timbre-api-contracts` rather than reached for sideways. That is the intended
discipline; if it becomes painful, the boundary is wrong and should be changed
with an ADR.
