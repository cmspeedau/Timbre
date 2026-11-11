Testing Roadmap
1. Purpose and scope

Provide shared testing infrastructure and quality gates for the music platform. Verify functional correctness, audio integrity, interoperability, security, performance, and recoverability.

Testing is a cross-cutting workstream. Each subsystem remains responsible for its own unit tests.

2. Architecture
unit/ — subsystem-level tests.
contracts/ — API and event compatibility.
integration/ — database, server, library, and playback flows.
e2e/ — complete user journeys.
audio/ — reference files, PCM validation, DSP tests.
performance/ — load, latency, and resource benchmarks.
security/ — authentication and authorization tests.
fixtures/ — curated music files and expected results.
reports/ — reproducible test outputs and release evidence.

Tooling: cargo test, criterion for benchmarks, proptest for property tests, insta for snapshots, wiremock for HTTP mocks, and hound for audio fixtures.
3. Implementation phases
Phase 0 — Testing foundation
- [x] Establish workspace-level Rust build, test, formatting, and strict Clippy checks in CI.
- [ ] Define fixture licensing and redistribution rules.
- [x] Add small generated WAV fixtures plus JSON contract fixtures.
- [x] Keep core tests deterministic and hardware independent through injected memory sinks and in-memory SQLite.
- [x] Add focused crate tests and offline milestone/web verification scripts.
- [ ] Exit gate: CI stages, fixtures, and deterministic test configuration are established.
Phase 1 — Core correctness
- [x] Test SQLite playback migrations and state persistence.
- [x] Test library scanning, metadata parsing, malformed-file isolation, and fallbacks.
- [x] Test API contract round trips and playback handlers.
- [x] Test queue persistence, playback modes, volume, and state transitions.
- [x] Test WAV decoder behavior, seek boundaries, and PCM output with a memory sink.
- [ ] Automate the complete configured-root scan → browse → audible-play end-to-end journey.
- [ ] Exit gate: Scan-to-play end-to-end tests and core correctness tests run automatically.
Phase 2 — Audio quality
- [ ] Add licensed reference PCM files and stable hashes.
- [x] Test WAV decoding and PCM sample-rate/channel adaptation numerically.
- [x] Test channel mapping and equalizer DSP behavior.
- [x] Test sink buffer counters/underrun behavior through deterministic sinks; hardware latency remains unmeasured.
- [ ] Test long playback sessions and track transitions.
- [x] Separate hardware-independent numerical tests from unverified hardware smoke tests.
- [ ] Exit gate: Decoding, conversion, channel mapping, and DSP have reference fixtures and numerical checks.
Phase 3 — Scale and resilience
- [ ] Generate large synthetic libraries.
- [ ] Test concurrent clients and queue updates.
- [ ] Test server restarts during active jobs.
- [ ] Test disconnected outputs and network interruptions.
- [ ] Benchmark search, scan time, memory, and CPU.
- [ ] Test recovery from database and cache failures.
- [ ] Exit gate: Scale, concurrency, restart, and recovery tests run with measured performance.
Phase 4 — Security and release
- [ ] Test authentication and authorization.
- [ ] Test path traversal and malformed inputs.
- [ ] Test dependency vulnerabilities.
- [ ] Add cross-platform release smoke tests.
- [ ] Establish regression gates and release reports.
- [ ] Exit gate: Security, dependency, and cross-platform smoke tests gate releases.
4. Task checklist
- [ ] Tests are repeatable and do not depend unnecessarily on live services.
- [ ] Fixtures have known formats and expected results.
- [ ] Flaky tests are tracked and corrected.
- [ ] Performance results include environment details.
- [ ] Hardware-dependent tests are clearly identified.
- [ ] Regressions are linked to reproducible cases.
- [ ] Critical failures block release.
5. API and contracts

Define a shared contract-test harness for API schemas, event payloads, error models, and compatibility rules.

Subsystems should expose testable interfaces and inject dependencies where appropriate. Do not require production credentials for routine CI.

6. Testing strategy

Use layered validation:

Fast unit and static-analysis checks on every change.
API and database integration tests on pull requests.
Audio regression and end-to-end tests on relevant changes.
Performance and security suites on scheduled runs and release candidates.
Hardware interoperability tests before declaring device support.
7. Acceptance criteria
Core user journeys run automatically.
Supported audio formats have regression fixtures.
Contract changes are detected before release.
Performance regressions can be measured.
Critical security and data-integrity tests pass.
Release artifacts pass documented smoke tests.
8. Dependencies and blockers
Receives fixtures and test hooks from every subsystem.
Coordinates API compatibility with server/ and client/.
Coordinates audio validation with audio-engine/.
Coordinates hardware testing with devices/.
Coordinates installation and upgrade tests with deployment/.
9. Definition of done

A subsystem release candidate must pass required automated tests, report known failures, include relevant performance results, and demonstrate the master roadmap's integration gates.
