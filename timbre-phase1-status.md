# Timbre — Phase 1 status and implementation plan

**Review date:** 2026-11-09
**Repository:** `cmspeedau/Timbre`, branch `main`, commit `057caba` (`docs(roadmap): mark Phase 0 complete`)

## Executive status

**Phase 1 has not started in implementation.** The checkout is a clean Phase 0 scaffold: workspace, shared Rust types/API models, ADRs, migrations convention, and CI exist, but user-facing subsystem code is still placeholder-level. The Master Roadmap leaves every Phase 1 task unchecked; README also describes the project as having no user-facing functionality. No repository issues or pull requests were found in the GitHub searches performed. Local uploads and prior outputs were empty.

Phase 1 is **feasible**, but not as a single milestone: server APIs, persistent library scanning, client browsing/queue, and audio output span several crate boundaries and require a working end-to-end test harness. The practical route is staged vertical slices with schema and contracts first. Do not claim the Phase 1 exit gate until index → browse albums → play album is demonstrated reliably.

## What exists vs. what remains

| Phase 1 capability | Existing evidence | Assessment |
|---|---|---|
| Server health + library APIs | `crates/server/src/lib.rs` only declares subsystem and API version; no HTTP dependencies, router, configuration, or endpoints. Server Roadmap proposes `/api/v1/health` and core APIs, explicitly noting proposed routes are not implemented. | Not built. Health/readiness is an uncompleted server-foundation prerequisite. |
| Local scan + persisted metadata | `crates/library/src/lib.rs` is a 9-line Phase 0 skeleton. No scanner, catalog storage, SQL dependencies, or SQL migration files. `migrations/README.md` confirms there are no migrations yet. | Not built. Requires schema, safe configured-path handling, extraction, persistence, progress/cancellation, and scan tests. |
| Browse, search, queue interfaces | `crates/api-contracts` already has serializable Artist/Album/Recording and playback queue/state models; current tests cover JSON round trips. No endpoint implementations, search contract, persistent queue, or functional UI. `crates/ui` and both apps remain skeletons. | Models are useful groundwork; no user workflow exists. |
| Local playback | `crates/audio-engine` and `crates/devices` contain only subsystem constants. No decoder/output dependencies or device selection. | Not built. A meaningful local-output path is a separate high-risk integration milestone. |
| Playlists, favorites, persistent playback | No associated schema, service, routes, or client workflows found. | Not built. Plan after scan/browse and a working playback transport; persist via migrations. |
| Tests | Current integration coverage is Phase 0 contract serialization/identity checks. No scan-to-play, database, API, queue, decoder, or hardware tests/fixtures. CI is configured for fmt, clippy, tests, build. | Foundation exists; Phase 1 correctness coverage must be added. |

The Master Roadmap marks Phase 0 complete, while the subsystem roadmaps still show their own Phase 0 items unchecked. In particular, the Server Roadmap’s server entry/configuration/health/database/shutdown items are prerequisites that should be explicitly reconciled with the Master Roadmap before coding, rather than silently treating them as delivered.

## Feasibility and key risks

- **Feasible with incremental delivery.** The project has the right Rust workspace, shared IDs, library/playback contracts, architectural and migration conventions, and CI entry points.
- **Main integration risk:** the first persistent data model and internal library/server interfaces are not defined yet. Agree them before adding routes or client-specific types.
- **Audio/device risk:** no prototype, supported codec list, target-OS commitment, output backend, or hardware test fixtures are present. The Audio Engine Roadmap calls for an end-to-end output prototype; prove decode-to-one-output early, before building higher-level controls around it.
- **Metadata/scanning scope risk:** library Phase 1 includes incremental change/missing-file tracking, malformed-file resilience, job progress, and cancellation. Keep automated metadata matching, artwork, DSP, exclusive/bit-perfect paths, multiroom, and remote access out of this MVP.
- **Contract risk:** playback queue contracts exist, but the roadmap also requires versioned error/API conventions, pagination, playlists/favorites, and a machine-readable API spec. Extend shared contracts deliberately and add compatibility tests.
- **Safety risk:** scan only explicit configured roots; canonicalize and constrain paths, avoid arbitrary filesystem access, and make missing files non-destructive. Keep scanning off request/audio threads.

## Recommended implementation sequence

1. **Agree the first vertical slice and interfaces.** Reconcile the Server Roadmap Phase 0 prerequisite checkboxes with the Master Roadmap; document supported audio formats/platform for the prototype; define library location, media-file, scan job, and catalog persistence boundaries using existing shared IDs/contracts. Record any roadmap changes before code changes.
2. **Persistence + scanner foundation.** Add the first forward-only SQLite migration(s) for configured library roots, media-file identity/metadata, catalog artist/album/recording records, scan jobs, and indexes. Implement a repeatable recursive scan of a configured root, metadata extraction for explicitly supported formats, malformed-file isolation, file stat/reference capture, and added/changed/missing reporting. Add cancellation/progress without doing long work in HTTP handlers. Test clean and repeated scans, partial tags, malformed files, inaccessible roots, and path traversal.
3. **Server core API.** Add config, DB startup/migrations, health/readiness, consistent error envelope, versioned `/api/v1` routes, and library album/artist/track browse (pagination and deterministic ordering). Connect routes to library service abstractions, not directly coupled client types. Add API contract/integration tests and publish an OpenAPI/schema artifact as required by Server Roadmap.
4. **Audio proof and queue transport.** In parallel or immediately after the library schema stabilizes, implement a minimal file-backed decoder-to-one-local-output prototype on a declared initial platform; then define lifecycle/error behavior and play/pause/stop/seek/end-of-track. Add an in-memory queue first only if its persistence boundary is explicit; persist queue and playback state in a subsequent migration. Test decoder behavior without hardware and make hardware smoke tests clearly marked.
5. **Client browsing and scan-to-play.** Implement loading/empty/error states, album and artist lists/details, queue and basic transport over the versioned API; then playlists/favorites and state restoration. Add an automated scan → browse album → enqueue/play journey. Only then assess the Master Phase 1 exit gate.

### First PR-sized milestone

Start with a **library persistence + deterministic scan vertical slice**, not UI polish or broad API surface: one configured folder, a constrained recursive scan, metadata for a small declared codec set, SQLite persistence, progress/result reporting, and tests proving repeat scans do not duplicate records. This establishes the stable data consumed by browse endpoints and is feasible without claiming audio playback is ready. In parallel, prototype one supported decoder/output early enough to expose hardware or platform blockers before the client depends on it.

### Definition of done for the first slice

- A fresh database can migrate without manual setup.
- Configured-root validation prevents scanning outside the approved root.
- Fixture files produce deterministic media/album/artist records, including incomplete tags.
- A malformed or unreadable file is reported and does not abort the rest of the scan.
- A second scan is idempotent; changed and missing files are represented non-destructively.
- Progress and cancellation behavior are tested.
- `cargo fmt`, `cargo clippy`, and relevant `cargo test` suites pass in CI.

## Validation performed

- Cloned the public repository to `/home/user/Timbre`; working tree was clean on `main` at `057caba`.
- Reviewed Master, Server, Library, Audio Engine, Devices, Client, and Testing roadmaps; README; CI; core crate manifests/sources; API contracts/tests; and migration notes.
- GitHub searches returned **zero issues** and **zero pull requests** for `cmspeedau/Timbre`.
- Attempted `cargo check --workspace` and `cargo test --workspace`. The repository pins Rust `1.99.0`, so that toolchain was installed. Both commands are **blocked before project compilation**: this sandbox has no system linker/`cc` (`linker cc not found` while compiling `proc-macro2`). No project compiler/test result can be asserted from this environment. CI specifies the intended checks and runs them on Ubuntu; rerun the commands in a Rust environment with a C linker installed.

## Immediate actions

1. Install/provision a Linux C toolchain/linker in the build environment, then run `cargo check --workspace`, `cargo test --workspace --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --all -- --check`.
2. Reconcile and check off only actually completed server prerequisites; preserve the Master Roadmap as source of truth.
3. Open implementation work in dependency order: initial schema + scan service/tests; health/config/database and browse APIs; decoder-to-one-output prototype; client browse/queue; playlists/favorites and persisted playback; scan-to-play E2E.
4. Update roadmaps and known limitations as each acceptance gate is demonstrated; keep Phase 1 unchecked until all checkboxes and its end-to-end exit gate pass.

## Milestone 3 — Server Library APIs (2026-11-09)

Implemented in the Timbre workspace and included in `timbre-milestone3.zip`:

- Axum router and clonable shared application state.
- `/api/v1` library roots, asynchronous scan submission, job lookup, and browse/detail endpoints for artists, albums, and tracks; `/api` compatibility alias retained for the requested paths.
- JSON request/response models, pagination (offset/limit), filtering, stable sorting, and common `ApiError` error envelopes.
- Canonicalized directory roots, duplicate-root conflict response, UUID job IDs, and scans dispatched with `spawn_blocking` through the existing Lofty/WalkDir library scanner.
- In-memory catalog upserts and focused tests for pagination bounds and stable recording identity on rescans.

**Limitations:** root/catalog/job data is in-memory only and is lost on restart. Search, playlists, playback, authentication, cancellation, live scan progress, and OpenAPI generation are not included. The scan job reports final successful-file counts and issue count, not incremental progress. Routes currently accept both `/api/v1` and `/api` prefixes.

**Validation:** `cargo fmt --all` and `cargo metadata --no-deps --format-version 1` succeeded. `cargo check -p timbre-server` resolved/downloaded dependencies but could not begin Rust compilation because this sandbox has no `cc` linker (`linker cc not found`); compilation and tests therefore remain unverified here.

## Milestone 5 — Playback Server APIs & SSE (2026-11-09)

Implemented against the available `timbre-milestone4.zip` workspace:

- Added typed playback transport contracts in `crates/api-contracts`: play and seek requests, volume request, status enum, snapshot, and sequenced/timestamped playback event models.
- Added server playback routes under `/api/v1/playback` plus the existing `/api` compatibility prefix: `POST play`, `pause`, `stop`, `seek`, `volume`; `GET state` and `GET events`.
- Connected `PlaybackEngine` to cloned `AppState` using a mutex-protected playback store and a Tokio broadcast channel. Control transitions publish `playback_state` events; SSE clients can recover from reconnect by fetching the current state.
- Play resolves the recording ID through the in-memory library catalog and loads the indexed file into the existing Symphonia engine. Invalid requests and load/seek failures return structured API error envelopes.
- Added unit coverage for snapshot initial state and serialization/delivery through broadcast; documented endpoint payloads and runtime limitations in `docs/api/README.md`.

**Phase 1 status: NOT COMPLETE.** Milestone 5 implements the API/control and event-stream slice but does not demonstrate the Phase 1 scan → browse → play workflow or hardware audio output. The existing `PlaybackEngine` is pump-driven; this server integration uses `MemoryAudioSink` and does not run a PCM pump/device worker, so actual audible output and advancing position are not delivered by HTTP controls alone. Catalog/playback state remains volatile. Phase 1 exit gates for persistence, client workflow, and end-to-end playback remain outstanding.

**Archive caveat:** the supplied milestone-4 ZIP omits numerous crates referenced by its workspace manifest and does not include their source or manifests (including `crates/shared-types` and `crates/api-contracts`). Minimal compatibility crates were restored for the source present, and placeholder manifests/targets were added for the other workspace members so the manifest graph can be inspected. Those placeholders do not restore omitted project implementations and must be reconciled with the canonical repository before integration. No external repository was accessed.

**Validation:** all 14 workspace `Cargo.toml` files parsed successfully with Python `tomllib`; `unzip -t` reported no archive errors. Route and model source was reviewed and contract/server unit tests were added, but not executed. `cargo` and `rustc` are not installed in this execution sandbox, so Rust formatting, Cargo manifest resolution, compilation, tests, clippy, and runtime/API balance checks could not be executed. Do not treat Rust syntax or test results as passed pending a Rust CI run.
