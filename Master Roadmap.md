Master Roadmap
1. Purpose and scope

Coordinate development of a self-hosted, Roon-inspired music platform covering library management, playback, metadata, audio processing, device management, discovery, remote access, and deployment.

This roadmap set is the project's source of truth. This file tracks cross-subsystem milestones, the agreed technology stack, and integration readiness. Detailed implementation tasks belong in each subsystem's own roadmap.md.

Goals
Deliver a reliable local music library and playback MVP.
Establish stable contracts between independently developed subsystems.
Add music intelligence and advanced audio capabilities incrementally.
Support multiple clients and output devices.
Maintain security, testability, and operational reliability.
Non-goals
Copying proprietary implementation details or branding.
Assuming access to proprietary Roon protocols, including RAAT.
Depending on cloud services for basic local playback.
Building a distributed microservice architecture before it is necessary.
Depending on a non-Rust language, runtime, web engine, or interpreter for any component.

2. Source of truth

The roadmap set in this repository is the single source of truth for the project.

Rules:
Every architectural and stack decision is recorded in a roadmap. A decision that exists only in code or in conversation is not decided.
Code, tests, and generated artifacts derive from the roadmaps. When implementation and a roadmap disagree, the roadmap is the reference and must be updated deliberately, with a stated reason, rather than drifting silently.
Shared decisions are defined once, in this file, and referenced elsewhere. Subsystem roadmaps must not restate contracts, testing strategy, or stack choices that belong to another roadmap.

3. Architecture and stack

The initial product is a modular monolith in a single Rust workspace, with separate client applications and optional playback endpoint processes. Every component is written in Rust.

Workspace:
crates/api-contracts/ — shared, versioned API models and schemas.
crates/shared-types/ — shared domain types: identifiers, errors, job status.
crates/server/ — API, configuration, orchestration, authentication.
crates/library/ — scanning, metadata, database, artwork, indexing.
crates/audio-engine/ — decoding, processing, rendering, signal path.
crates/devices/ — output discovery, zones, endpoints, synchronization.
crates/discovery/ — metadata enrichment, search, relationships, recommendations.
crates/remote-access/ — pairing, secure connectivity, remote sessions.
crates/integrations/ — streaming providers and external protocol adapters.
crates/ui/ — shared Slint components used by the desktop and mobile clients.
apps/desktop/ — desktop client binary.
apps/mobile/ — mobile client binary.
crates/testing/ — cross-system quality, regression, performance, audio validation.
deploy/ — packaging, installation, upgrades, backups, observability.

Workspace package names take the `timbre-` prefix (for example `timbre-server` and `timbre-audio-engine`); the directory names omit it.

Stack decisions:
server/ — axum, tokio, tower, tower-http, serde, sqlx, tracing, tracing-subscriber.
library/ — lofty, walkdir, blake3, image, sqlx.
audio-engine/ — symphonia, cpal, rubato, rustfft, realfft, ebur128.
devices/ — mdns-sd, tokio, rustls, tokio-rustls.
discovery/ — reqwest, serde, tantivy.
integrations/ — reqwest, oauth2, librespot.
remote-access/ — rustls, tokio-rustls, axum.
client/ — Slint, one cross-platform codebase shared by desktop and mobile.
mobile/ — Slint, with platform integration through jni/ndk (Android) and objc2 (iOS).
testing/ — cargo test, criterion, proptest, insta, wiremock, hound.
deployment/ — cargo build --release, musl static target, distroless container.

Persistence: SQLite in WAL mode, accessed through sqlx, is the default store for a single-node deployment. Any other database, such as PostgreSQL, is an optional adapter behind the storage layer rather than a core dependency.

Architectural rules
The server owns authoritative library and playback-session state.
The library owns media indexing and metadata persistence.
The audio engine owns decoding and audio processing, not UI state.
Devices expose capabilities through a documented interface.
Clients communicate through versioned API contracts.
External services are accessed through replaceable adapters.
Local playback must remain functional if external providers are unavailable.
No component may require a non-Rust runtime, interpreter, or web engine to function.

4. Implementation phases

Each task below is a checkbox. A stage is complete only when every checkbox in it is ticked and its exit gate is demonstrated; tick items as work lands so this file remains the source of truth for progress.

Phase 0 — Project foundation
- [ ] Establish the Cargo workspace, coding conventions, CI, and development environment.
- [ ] Define shared API contracts and architectural decision records.
- [ ] Create database migration and configuration conventions.
- [ ] Define crate ownership and dependency rules.
- [ ] Exit gate: All crates build, test, and communicate through agreed interfaces.

Phase 1 — Local-library MVP
- [ ] Implement server health and library APIs.
- [ ] Scan local music folders and persist metadata.
- [ ] Build browsing, search, and queue interfaces.
- [ ] Play supported files through one local audio output.
- [ ] Add playlists, favorites, and persistent playback state.
- [ ] Exit gate: A user can index a folder, browse albums, and play an album reliably.

Phase 2 — Metadata, audio quality, and bit-perfect output
- [ ] Add metadata matching and relationship navigation.
- [ ] Implement signal-path reporting.
- [ ] Add basic DSP and output capability negotiation.
- [ ] Implement bit-perfect exclusive output per operating system: WASAPI exclusive (Windows), ALSA hw (Linux), and CoreAudio hog mode (macOS).
- [ ] Add explicit sample-rate conversion and channel mapping where the exclusive path requires it.
- [ ] Improve error reporting and library recovery.
- [ ] Exit gate: Metadata edits are reliable, audio behavior is observable and testable, and bit-perfect output is demonstrated on each supported operating system.

Phase 3 — Multiple outputs and discovery
- [ ] Add device discovery and named zones.
- [ ] Add advanced search and related-music navigation.
- [ ] Introduce radio and explainable recommendations.
- [ ] Validate network-output compatibility.
- [ ] Exit gate: Multiple independent playback destinations operate without destabilizing the server.

Phase 4 — Remote and mobile access
- [ ] Add device pairing and authenticated remote access.
- [ ] Deliver mobile browsing and playback control.
- [ ] Add background playback and optional offline downloads.
- [ ] Exit gate: Authorized users can securely access their library remotely.

Note: mobile is the long pole of a Rust-only stack. Slint provides the interface, but background playback, lock-screen controls, and audio-session and route handling require direct platform APIs. This phase's exit gate must not assume a web runtime or a platform-native UI toolkit.

Phase 5 — Advanced capabilities
- [ ] Add synchronized playback groups.
- [ ] Introduce eligible streaming-service integrations.
- [ ] Improve recommendations and library-scale performance.
- [ ] Complete production packaging and recovery workflows.
- [ ] Exit gate: Release-quality installation, recovery, security, and regression tests pass.

Phase 6 — Future backlog (not scheduled)

The following capabilities are recognized as central to the product but are deliberately unscheduled. Each is a candidate milestone once Phases 0 through 5 are complete. Ownership belongs to the subsystem roadmap named beside it.

Focus and Bookmarks (library and discovery)
- [ ] Faceted filtering across the library by genre, label, year, performer, format, bit depth, sample rate, and channel count.
- [ ] Saved Focus queries as named bookmarks, with sharing between profiles where permitted.
- [ ] Focus reachable from any browsing context, not only a dedicated screen.

Classical works and movements (library and discovery)
- [ ] Represent works, movements, opus and catalogue numbers, and part structure.
- [ ] Group recordings under a work and link performances to the same composition.
- [ ] Preserve original classical tags alongside normalized values.

Profiles (server, client, and mobile)
- [ ] Multiple user profiles with separate playlists, favorites, tags, and listening history.
- [ ] Per-profile zones, settings, and default behaviors.
- [ ] Profile-scoped library access and privacy controls.

Extension API and home automation (server and integrations)
- [ ] A public, versioned API for third-party extensions.
- [ ] Event notifications for playback, zone, and library changes.
- [ ] Documented home-automation integrations (for example webhooks, Home Assistant, and control systems) with permission and security review.

Internet radio (integrations and client)
- [ ] A directory of internet radio stations plus user-added custom streams.
- [ ] Station playback through the existing zone and device model.
- [ ] Handling of unstable streams, redirects, and metadata for live audio.

5. Cross-project task checklist
- [ ] Every crate has an owner and an up-to-date roadmap.
- [ ] Shared contracts are versioned and reviewed.
- [ ] Database migrations are backward-compatible where required.
- [ ] Playback behavior is tested independently of the UI.
- [ ] Long-running jobs cannot block audio rendering.
- [ ] External integrations fail independently of local playback.
- [ ] Security and privacy requirements are documented.
- [ ] Release artifacts can be built reproducibly.
- [ ] No hard dependency on a non-Rust language, runtime, or web engine is introduced.

6. Shared API and contract policy

All inter-subsystem contracts must be documented in docs/api/ or crates/api-contracts/.

Required shared models:
Media item and file identity.
Artist, album, recording, and credit.
Playback queue and playback state.
Output device, capabilities, and zone.
Audio format and processing pipeline.
Job status and progress.
Error code and diagnostic context.

Contracts are defined once as Rust types in crates/api-contracts/ and crates/shared-types/. Subsystem roadmaps reference these types rather than restating them.

Changes to shared contracts require:
Updated schema or interface definition.
Compatibility assessment.
Updated consumer tests.
Migration notes where necessary.

7. Testing strategy
Unit tests within every crate.
Contract tests between clients, server, and device adapters.
End-to-end tests covering scan, browse, queue, and playback.
Audio fixtures for decoding, conversion, and DSP.
Load tests for large libraries and concurrent clients.
Security tests for authentication, file access, and remote sessions.
Recovery tests for restart, missing files, and disconnected devices.

The testing toolchain is cargo test, criterion, proptest, insta, wiremock, and hound. Per-subsystem test detail belongs in each subsystem's roadmap.

8. Master acceptance criteria
A clean installation can index and play a representative local library.
Clients do not depend on internal database implementation details.
Playback continues independently of UI rendering and metadata enrichment.
Supported audio formats and output capabilities are reported accurately.
Bit-perfect output is verifiable on each supported operating system.
Library and playback state recover predictably after restart.
Remote access is authenticated and disabled unless configured.
The platform builds and ships as Rust binaries with no non-Rust runtime dependency.
Every release has documented limitations and reproducible test results.

9. Dependencies and blockers
Audio engine requires a stable media-file access interface.
Client requires versioned library and playback APIs.
Device management requires audio-engine capability and lifecycle interfaces.
Discovery requires library identifiers and metadata provenance.
Mobile remote playback depends on server authentication and session APIs.
Multi-room playback depends on endpoint timing measurements and a synchronization design, because the RAAT protocol is unavailable.
Bit-perfect output depends on per-operating-system exclusive-mode backends.
Streaming integrations depend on provider API access and licensing.

10. Definition of done

A master milestone is complete when its subsystem tasks are finished, integration tests pass, acceptance criteria are demonstrated, documentation is updated, and known limitations are recorded.

Do not mark a milestone complete merely because code has been merged.
