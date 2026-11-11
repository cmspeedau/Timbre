# Timbre Phase 2 Milestone 7 — Status

## Implemented
- Added parametric equalizer settings and stateful per-channel biquad DSP, with finite/range validation, preamp, band enable controls, and EQ-before-volume processing.
- Added SQLite migration `0005_playback_equalizer.sql`, persisted EQ API settings, event notification, API contracts, and JSON fixtures.
- Added `GET /api/v1/playback/metrics` and compatibility route backed by sink counters instead of fixed placeholders. The bounded PCM sink measures capacity/occupancy in interleaved samples, output frames written, and empty nonblocking read underruns; the API converts sample occupancy to frames and estimates buffered duration/latency from sample rate.
- Fixed malformed `PlaybackEngine` fields/initializer, removed duplicate pre-EQ gain processing, repaired malformed contract-test nesting, and added sink telemetry assertions.
- Carried forward milestones 3–6 features and offline verification scripts.

## Frontend / Web UI Integration
- Added a responsive single-page listening room at `crates/server/web/` and served it from `/` with embedded-at-build static HTML, CSS, and JavaScript asset routes (`/assets/timbre.css`, `/assets/timbre.js`). No JavaScript framework or extra runtime dependency is required.
- Integrated play/pause/stop/next/previous, seek and elapsed/total time, volume/mute, library search and add-to-queue, queue current-track indication, move up/down and remove/clear operations, repeat cycling and shuffle, and output-device discovery/selection.
- Added five-band EQ gain and preamp controls, EQ enable/reset/save, plus a live playback metrics panel for buffer occupancy, estimated latency, underruns, format, and active sink.
- Connected playback SSE at `/api/v1/playback/events`, handling transport, seek, device, equalizer, mode, transition, and queue updates; state polling/reconnect support refreshes missed changes.
- Queue create/remove/reorder/move/clear server actions now publish `queue.updated` events, matching the web client's live subscription.

## Verification
- `python3 scripts/verify_milestone3.py` through `verify_milestone7.py`: PASS (existing backend checks).
- `python3 scripts/verify_web.py`, Node.js `--check` of the frontend script, and Python byte-compilation of the frontend verifier: PASS.
- Python byte-compilation of all five verifiers and JSON parsing of all fixtures: PASS.
- SQLite migration and EQ persistence checks: PASS through the offline Milestone 7 verifier.
- Rust compilation/tests: NOT RUN; Rust toolchain (`cargo`/`rustc`) is not installed in this sandbox.

## Caveats
- Measured occupancy and underruns are reported by `BufferedPcmSink`; sinks that do not implement counters return defaults, so memory/CPAL-backed paths may report zero buffer/counter values.
- `frames_dropped` is zero for the bounded sink (writes apply backpressure rather than dropping samples). `drop_rate` is accordingly zero. Duration/latency are buffer-based estimates, not end-to-end hardware latency.
- Hardware audio/device switching was not exercised in this headless sandbox.
