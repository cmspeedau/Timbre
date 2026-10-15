# Timbre

A self-hosted, Roon-inspired music platform: local library management, playback, metadata, audio processing, multi-room devices, discovery, remote access, and deployment.

**The name.** Timbre is the character that makes a sound what it is — what a listener recognizes beyond pitch and loudness. The project deliberately copies no proprietary implementation details, branding, or protocols.

> **Status: planning.** There is no code yet. The roadmaps in this repository are the specification, and they are the project's source of truth.

## Contents

- [What this is](#what-this-is)
- [Source of truth](#source-of-truth)
- [Technology stack](#technology-stack)
- [Repository layout](#repository-layout)
- [Roadmap](#roadmap)
- [Subsystem roadmaps](#subsystem-roadmaps)
- [Development](#development)
- [Non-goals](#non-goals)

## What this is

A modular monolith, written entirely in Rust, that indexes a local music library, plays it to one or more output devices, enriches it with metadata, and exposes it to desktop and mobile clients over versioned APIs.

Goals:

- Deliver a reliable local library and playback MVP.
- Establish stable contracts between independently developed subsystems.
- Add music intelligence and advanced audio capabilities incrementally.
- Support multiple clients and output devices.
- Maintain security, testability, and operational reliability.

## Source of truth

The roadmap set is the single source of truth for this project.

1. **Every decision lives in a roadmap.** A decision that exists only in code or conversation is not decided.
2. **Code derives from the roadmap.** When implementation and a roadmap disagree, the roadmap is the reference and must be updated deliberately, not drifted from silently.
3. **Shared decisions are defined once.** Contracts, testing strategy, and stack choices live in the [Master Roadmap](Master%20Roadmap.md) and are referenced, not restated, elsewhere.

Phases, tasks, and exit gates are tracked as checkboxes in the roadmaps. Tick items as work lands.

## Technology stack

Rust only — no non-Rust runtime, interpreter, or web engine is required to function.

| Area | Crates |
| --- | --- |
| Server / API | `axum`, `tokio`, `tower`, `tower-http`, `serde`, `sqlx`, `tracing` |
| Library | `lofty`, `walkdir`, `blake3`, `image`, `sqlx` |
| Audio engine | `symphonia`, `cpal`, `rubato`, `rustfft`, `realfft`, `ebur128` |
| Devices | `mdns-sd`, `tokio`, `rustls`, `tokio-rustls` |
| Discovery | `reqwest`, `serde`, `tantivy` |
| Integrations | `reqwest`, `oauth2`, `librespot` |
| Remote access | `rustls`, `tokio-rustls`, `axum` |
| Client (desktop + mobile) | `Slint`, with `jni`/`ndk` (Android) and `objc2` (iOS) |
| Testing | `cargo test`, `criterion`, `proptest`, `insta`, `wiremock`, `hound` |
| Deployment | `cargo build --release`, musl static target, distroless images |

**Persistence:** SQLite in WAL mode via `sqlx` is the default store for a single-node deployment. Any other database (for example PostgreSQL) is an optional adapter behind the storage layer.

**Bit-perfect output** is a first-class goal and is implemented per operating system: WASAPI exclusive (Windows), ALSA `hw` (Linux), and CoreAudio hog mode (macOS).

## Repository layout

The intended Cargo workspace mirrors the subsystems:

```
crates/api-contracts/   shared, versioned API models and schemas
crates/shared-types/    shared domain types: identifiers, errors, job status
crates/server/          API, configuration, orchestration, authentication
crates/library/         scanning, metadata, database, artwork, indexing
crates/audio-engine/    decoding, processing, rendering, signal path
crates/devices/         output discovery, zones, endpoints, synchronization
crates/discovery/       metadata enrichment, search, relationships, recommendations
crates/remote-access/   pairing, secure connectivity, remote sessions
crates/integrations/    streaming providers and external protocol adapters
crates/ui/              shared Slint components for desktop and mobile
apps/desktop/           desktop client binary
apps/mobile/            mobile client binary
crates/testing/         cross-system quality, regression, performance, audio validation
deploy/                 packaging, installation, upgrades, backups, observability
```

## Roadmap

Development proceeds in stages. A stage is complete only when all its tasks are ticked and its exit gate is demonstrated. See the [Master Roadmap](Master%20Roadmap.md) for the tracked checklist.

| Phase | Focus |
| --- | --- |
| 0 | Project foundation — workspace, contracts, conventions |
| 1 | Local-library MVP — scan, browse, queue, play one output |
| 2 | Metadata, audio quality, and bit-perfect output |
| 3 | Multiple outputs and discovery |
| 4 | Remote and mobile access |
| 5 | Advanced capabilities — grouped playback, streaming, packaging |
| 6 | Future backlog (not scheduled) — Focus, classical works, profiles, extension API, internet radio |

## Subsystem roadmaps

| Subsystem | Roadmap |
| --- | --- |
| Master | [Master Roadmap.md](Master%20Roadmap.md) |
| Server | [Server Roadmap.md](Server%20Roadmap.md) |
| Library | [Library Roadmap.md](Library%20Roadmap.md) |
| Audio engine | [Audio Engine Roadmap.md](Audio%20Engine%20Roadmap.md) |
| Devices | [Devices Roadmap.md](Devices%20Roadmap.md) |
| Client | [Client Roadmap.md](Client%20Roadmap.md) |
| Mobile | [Mobile Roadmap.md](Mobile%20Roadmap.md) |
| Discovery | [Discovery Roadmap.md](Discovery%20Roadmap.md) |
| Integrations | [Integrations Roadmap.md](Integrations%20Roadmap.md) |
| Remote access | [Remote Access Roadmap.md](Remote%20Access%20Roadmap.md) |
| Testing | [Testing Roadmap.md](Testing%20Roadmap.md) |
| Deployment | [Deployment Roadmap.md](Deployment%20Roadmap.md) |

## Development

Once the workspace exists, the standard commands apply:

```sh
cargo build              # build the workspace
cargo test               # run unit and integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

The first milestone is Phase 0: create the Cargo workspace, define shared contracts, and set up CI. Until then, work happens in the roadmaps.

## Non-goals

- Copying proprietary implementation details or branding.
- Assuming access to proprietary Roon protocols, including RAAT.
- Depending on cloud services for basic local playback.
- Building a distributed microservice architecture before it is necessary.
- Depending on a non-Rust language, runtime, web engine, or interpreter for any component.
