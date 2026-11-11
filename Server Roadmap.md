Server Roadmap
1. Purpose and scope

Build the central application server responsible for API access, client sessions, configuration, playback coordination, job orchestration, and system health.

The server coordinates subsystems but does not reimplement audio decoding or music metadata extraction.

Non-goals
Direct ownership of UI presentation.
Audio processing inside API request handlers.
Provider-specific business logic in core server modules.
2. Architecture

Suggested modules:

api/ — HTTP routes, request validation, response serialization.
realtime/ — WebSocket events and client subscriptions.
auth/ — local users, sessions, tokens, permissions.
config/ — configuration loading and validation.
orchestration/ — coordination between library, playback, and devices.
jobs/ — background-job scheduling and cancellation.
events/ — typed internal domain events.
health/ — health, readiness, and diagnostics.
observability/ — structured logs and metrics.

Use a modular-monolith architecture in Rust: axum for HTTP, tokio for the runtime, tower and tower-http for middleware, serde for serialization, sqlx for persistence, and tracing for structured logging. SQLite in WAL mode is the default store, as defined in the Master Roadmap; any other database sits behind the storage layer as an optional adapter.

Stack: axum, tokio, tower, tower-http, serde, sqlx, tracing.

3. Implementation phases
Phase 0 — Server foundation
- [ ] Create a production server entry point and configuration loader (router/state are currently library APIs; a binary/config lifecycle is not yet present).
- [ ] Add structured logging and request IDs.
- [x] Add a basic health endpoint.
- [x] Set up SQLite queue-state/device/EQ persistence and migrations.
- [ ] Implement graceful shutdown.
- [ ] Add environment-specific configuration validation and a readiness endpoint.
- [ ] Exit gate: The server starts from documented configuration, exposes health and readiness, and shuts down gracefully.
Phase 1 — Core APIs
- [x] Define initial versioned `/api/v1` routes and a shared error envelope.
- [x] Implement library-root, scan submission, job-result, artist/album/track browse and detail endpoints.
- [ ] Implement a server-side search endpoint (the web UI currently filters its loaded tracks locally).
- [x] Implement playback transport, queue, seek, volume, repeat/shuffle, device-selection, EQ, metrics, state, and SSE event routes.
- [ ] Implement playlist and favorite endpoints.
- [x] Add offset/limit pagination and basic filters/sorting for library browse endpoints.
- [ ] Generate or publish a machine-readable API specification.
- [ ] Exit gate: The documented core APIs are implemented, versioned, and covered by contract tests.
Phase 2 — Realtime and jobs
- [x] Publish playback state and queue updates over Server-Sent Events.
- [ ] Publish incremental library scan progress.
- [x] Add asynchronous scan submission and job-result lookup.
- [ ] Add job cancellation and retry policies.
- [x] Support reconnect by allowing clients to fetch current playback state and refresh after SSE reconnection.
- [x] Keep event payloads typed and sequenced for playback updates.
- [ ] Exit gate: Realtime events and background jobs report status and recover after client reconnection.

Implementation evidence (2026-11-11): Backend unit/workspace tests, strict Clippy, formatting, milestone and web verifiers pass. Limitations remain: scan roots/catalog/jobs are in memory, only playback state is persisted; no production binary/configuration, auth, playlists/favorites, server search, OpenAPI document, scan progress/cancellation, or graceful shutdown is included.
Phase 3 — Security and resilience
- [ ] Add user and client authentication.
- [ ] Implement authorization for sensitive operations.
- [ ] Validate all request parameters.
- [ ] Apply API rate limits where appropriate.
- [ ] Prevent arbitrary filesystem access.
- [ ] Add database transaction boundaries.
- [ ] Test graceful handling of unavailable dependencies.
- [ ] Exit gate: Authentication and authorization are enforced and sensitive operations are protected.
Phase 4 — Production operations
- [ ] Add runtime metrics and diagnostic endpoints.
- [ ] Implement configurable log levels.
- [ ] Add database backup coordination hooks.
- [ ] Document upgrade and migration procedures.
- [ ] Add resource limits for background jobs.
- [ ] Exit gate: Runtime metrics, diagnostics, and documented upgrade procedures support production operation.
4. Task checklist
- [ ] Server startup and shutdown are deterministic.
- [ ] Configuration errors are actionable.
- [ ] All APIs use consistent error envelopes.
- [ ] Long-running work is not performed synchronously in HTTP handlers.
- [ ] Realtime events recover gracefully after reconnection.
- [ ] Background jobs expose status and progress.
- [ ] Sensitive credentials are never written to logs.
- [ ] API compatibility is covered by contract tests.
5. API and contracts

Initial endpoint groups:

GET /api/v1/health
GET /api/v1/library/albums
GET /api/v1/library/artists
GET /api/v1/library/tracks
GET /api/v1/search
GET /api/v1/playback/state
POST /api/v1/playback/commands
GET /api/v1/playback/queue
POST /api/v1/playback/queue/items
GET /api/v1/devices
GET /api/v1/zones
GET /api/v1/jobs/{id}

These are proposed contracts, not existing endpoints.

Use separate read and command models. Commands should return validation results and a traceable operation ID where asynchronous processing is involved.

6. Testing strategy
Unit-test configuration, validation, and orchestration logic.
Test database transactions and migrations.
Contract-test all public endpoints.
Test concurrent queue updates.
Test client disconnects and WebSocket reconnection.
Test authentication and authorization boundaries.
Test shutdown during active scans and playback.
7. Acceptance criteria
The server starts from documented configuration.
APIs remain responsive during library scans.
Invalid commands fail without corrupting playback state.
Client reconnects restore the current state.
Unauthorized clients cannot modify protected resources.
Server restart preserves persistent library and playlist data.
8. Dependencies and blockers
Depends on shared API contracts and database migrations.
Coordinates with library/ for scan and catalog operations.
Coordinates with audio-engine/ and devices/ for playback.
Serves client/, mobile/, and remote-access/.
Relies on testing/ for contract and integration validation.
9. Definition of done

Every server feature must include validation, appropriate logs, tests, API documentation, and defined failure behavior. Public API changes must be reviewed for compatibility before release.
