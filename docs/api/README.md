# API contracts

The contracts are Rust types, defined once in `crates/api-contracts` and
`crates/shared-types`. This directory holds the prose that cannot live in a type:
versioning policy, compatibility rules, and the endpoint groups as they land.

The Master Roadmap requires every inter-subsystem contract to be documented here
or in `crates/api-contracts/`. Subsystem roadmaps reference these types rather
than restating them.

## Required shared models

The Master Roadmap names the models that must exist as shared contracts. All of
them are defined in `timbre-api-contracts` or `timbre-shared-types`.

| Model | Defined in |
| --- | --- |
| Media item and file identity | `api-contracts::media`, `shared_types::MediaItemId` |
| Artist, album, recording, credit | `api-contracts::library` |
| Playback queue and playback state | `api-contracts::playback` |
| Output device, capabilities, zone | `api-contracts::device` |
| Audio format and processing pipeline | `api-contracts::format` |
| Job status and progress | `shared_types::job` |
| Error code and diagnostic context | `shared_types::error`, `api-contracts::error` |

## Versioning policy

- The contract version is `timbre_api_contracts::API_VERSION` and appears in the
  request path: `/api/v1/...`.
- Additive changes — a new optional field, a new endpoint, a new enum variant a
  client can ignore — do not change the version.
- A breaking change ships as a new version, with the previous version served
  alongside it until consumers have migrated.
- Enum variants are additive but not free: a client must handle a variant it has
  never seen, because a server may be newer than it is.

## Changing a contract

Per the Master Roadmap, a change to a shared contract requires all of:

1. The updated schema or interface definition.
2. A compatibility assessment, stated in the change description.
3. Updated consumer tests.
4. Migration notes where necessary.

Serde is the wire format. Optional fields use `#[serde(skip_serializing_if =
"Option::is_none")]` so an absent field stays absent, and error codes are
`snake_case` strings that clients branch on.

The contract tests in `crates/testing/tests/contracts.rs` round-trip every shared
model and assert that the server and the clients agree on the version.

## Library API (Milestone 3)

The server currently mounts the library routes under both `/api/v1/library`
(the versioned contract path) and `/api/library` (a compatibility alias).
Browse collection responses use `{ "items": [], "offset": 0, "limit": 100,
"total": 0 }`; offset defaults to 0 and limit defaults to 100 and is capped at
500. Results are ordered by display name/title and then stable identity.

| Method and path | Purpose |
| --- | --- |
| `GET /api/v1/health` | Health response including served API version. |
| `POST /api/v1/library/roots` | Configure a local directory (`{"path":"..."}`); returns the canonical root and ID. |
| `GET /api/v1/library/roots` | List configured roots. |
| `POST /api/v1/library/scan` | Queue a scan (`{"root_id":"..."}`); returns `202` with a `JobStatus`. |
| `GET /api/v1/jobs/{id}` | Read a queued scan's job status. |
| `GET /api/v1/library/artists` | Browse artists. |
| `GET /api/v1/library/albums` | Browse albums; optional `artist_id` filter. |
| `GET /api/v1/library/tracks` | Browse tracks; optional `artist_id` and `album_id` filters. |
| `GET /api/v1/library/artists/{id}` | Read an artist. |
| `GET /api/v1/library/albums/{id}` | Read an album. |
| `GET /api/v1/library/tracks/{id}` | Read a track by recording ID. |

Errors use the shared `ApiError` envelope. Invalid roots are rejected before
being stored, and scans run on a blocking worker rather than in the HTTP request
handler. This milestone uses an in-memory catalog and job store: it is a
functional server adapter, not yet persistent across process restarts. The
unversioned alias is temporary compatibility and should be removed once clients
use the versioned path.


## Playback API (Milestone 5; volume and mute delivered in Phase 2 Milestone 4)

Playback controls are in-memory and shared across cloned server state. Responses return the `PlaybackSnapshot` (`state`, current recording ID, position in milliseconds, optional duration, linear volume, and mute flag). A recording can be played once it has been indexed by the library scanner. Volume must be a finite number in `[0,1]`; seeking is an absolute non-negative millisecond offset. When decoder metadata provides a duration, seeking past that duration is rejected; exact-end seeks are allowed. Seeking emits `playback.seeked` with the updated snapshot. Duration and position reflect decoder metadata and the decoded stream, respectively.

| Method and path | Request | Purpose |
| --- | --- | --- |
| `POST /api/v1/playback/play` | `{"recording_id":"<uuid>"}` | Load an indexed recording and start/resume it. |
| `POST /api/v1/playback/pause` | — | Pause the loaded recording. |
| `POST /api/v1/playback/stop` | — | Stop and reset playback, clearing the active recording. |
| `POST /api/v1/playback/next` / `previous` | — | Navigate according to repeat and shuffle modes; boundary attempts return `409`. |
| `POST /api/v1/playback/pump` | — | Run one bounded decode step (primarily for integrations/tests). |
| `POST /api/v1/playback/seek` (also `/api/playback/seek`) | `{"position_ms":1234}` | Seek within the loaded recording; emits `playback.seeked`. |
| `POST /api/v1/playback/volume` | `{"volume":0.75,"muted":false}` | Set gain and optional mute. |
| `POST /api/v1/playback/repeat` | `{"mode":"off"}` (`off`, `all`, `one`) | Select repeat policy. |
| `POST /api/v1/playback/shuffle` | `{"mode":"on"}` (`off`, `on`) | Enable/disable persistent shuffled traversal. |
| `GET /api/v1/playback/state` | — | Return the current playback snapshot. |
| `GET /api/v1/playback/events` | — | Subscribe to Server-Sent Events with serialized `PlaybackEvent` data. |

Event stream events carry a monotonic process-local sequence number and timestamp; playback changes are `playback_state`, manual transitions are `track_transition`, EOF advancement is `queue_advanced`, and mode changes are `playback.mode_changed`. Slow subscribers may miss buffered events; refresh `/playback/state` after reconnect. Router startup drives bounded decode pumping in the background; actual audible output is opt-in via the audio crate `cpal` feature and server `hardware-audio` feature; when unavailable, server output falls back to memory. `AppState::open_with_audio_backend(path, true)` requests hardware output.

## Playback queue API (Phase 2 Milestone 2)

This phase couples ordered queue management to transport controls and persists queue state to SQLite. A queue is an ordered list of **occurrences**;
adding the same recording more than once is valid. Each occurrence has a unique
entry UUID so clients can remove one duplicate without ambiguity. The `revision` starts at zero and increments on queue changes and cursor movement. A nullable `current_cursor` identifies the active queue index.
Clients may use it to detect that a previously fetched view is stale; it is not
an optimistic-concurrency precondition.

| Method and path | Request | Purpose |
| --- | --- | --- |
| `GET /api/v1/playback/queue` | — | Read `{ "entries": [], "revision": 0 }`. |
| `POST /api/v1/playback/queue` | `{"recording_id":"<uuid>"}` | Append a catalog recording; returns `201` and the full queue snapshot. |
| `DELETE /api/v1/playback/queue/{entry_id}` | — | Remove one occurrence; returns the updated snapshot. |
| `DELETE /api/v1/playback/queue` | — | Clear all occurrences; returns the updated snapshot. |
| `POST /api/v1/playback/queue/reorder` | `{"entry_ids":["<uuid>"]}` | Reorder all queue occurrences; every ID must occur exactly once. |

The SQLite migration `crates/db/src/migrations/0001_playback_queue.sql` stores entries, ordering, revision, and cursor. Use `AppState::open(path)` for persistence and startup restoration; `AppState::new()` uses an in-memory SQLite database. The router starts a lightweight bounded decode worker that advances to the next queue occurrence at EOF.

Adding a recording not present in the current in-memory catalog returns the
shared `not_found` error envelope. Queue state is shared among clones of one `AppState`, persisted to the configured database, automatically advanced at EOF, and announced through SSE. It is not user-scoped. These endpoints are available under `/api` as well
as the versioned prefix, matching the existing compatibility policy. See
`crates/testing/fixtures/playback-queue.json` for a wire fixture.


## Playback modes and audio output (Phase 2 Milestone 3)

`PlaybackSnapshot` includes `repeat` (`off`, `all`, `one`), `shuffle` (`off`, `on`), and `shuffle_enabled`. Mode changes are persisted with the ordered queue, selected cursor, shuffled traversal order, and navigation history. `repeat: one` repeats the active track at EOF; `all` wraps the queue; shuffle traverses a persisted shuffled occurrence order. `POST /api/v1/playback/repeat` accepts `{"mode":"all"}` and `POST /api/v1/playback/shuffle` accepts `{"mode":"on"}`. The SQLite upgrade is `0002_playback_modes.sql`.

Hardware output is an optional CPAL backend (`timbre-audio-engine/cpal`, or server feature `hardware-audio`). `AppState::open_with_audio_backend(path, true)` opens the default system output device when compiled with that feature, otherwise or on device failure it uses `MemoryAudioSink`. The device callback consumes queued PCM non-blockingly; headless deployments require no audio device.

## Phase 2 Milestone 4 — output level and queue movement

`PlaybackSnapshot.volume` is a linear gain in `[0.0,1.0]`; `muted` is an
independent output gate. `POST /api/v1/playback/volume` accepts a finite gain
and optional `muted` boolean; omitted `muted` preserves its current value. The
engine applies gain to every decoded PCM sample before writing to the sink,
rounding and saturating to signed 16-bit range. Mute outputs silence but keeps
the chosen gain. Changes apply to subsequently decoded buffers; data already in
a hardware queue may still play. Gain and mute persist in SQLite and are
restored when the server opens the database. The additive migration is
`crates/db/src/migrations/0003_playback_volume.sql`; old databases default to
full volume and unmuted. Existing v1 requests/responses remain wire-compatible.

Queue entries can be moved without replacing the rest of the queue:

| Method and path | Request | Purpose |
| --- | --- | --- |
| `POST /api/v1/playback/queue/move` | `{"entry_id":"<uuid>","position":2}` | Move the identified occurrence to a zero-based final index (the index after removal). |

The target must be in `[0, queue length)` and the entry must exist. Invalid
requests return the shared `invalid_request` envelope. Moving preserves the
active occurrence by identity, clears stale shuffle traversal/history, and
increments the queue revision; a no-op move leaves the revision unchanged.
This endpoint is additive and does not alter existing full-order `reorder`.
Wire examples: `crates/testing/fixtures/playback-volume.json` and
`crates/testing/fixtures/playback-queue-move.json`.


## Phase 2 Milestone 5 — seeking and output-format adaptation

The engine seeks the active Symphonia reader/decoder to an absolute position.
`PlaybackSnapshot.duration_ms` is populated when codec/container frame-count
metadata is available; bounds are validated against that duration. Without
known duration, only finite non-negative positions can be checked. Successful
seek preserves playing/paused state (an ended track becomes paused) and reports
the selected position immediately.

The optional CPAL sink adapts decoded signed-16-bit PCM into the device's
negotiated rate and channel count. It uses frame-wise linear interpolation for
rate conversion, duplicates mono into output channels, averages multichannel
to mono, preserves common channel indices, and fills additional channels from
a stereo average. The conversion happens before samples enter the bounded
non-blocking callback queue; memory and generic sinks retain their configured
input format. This is a basic adapter, not a high-quality anti-aliasing
resampler. It requires complete interleaved input frames and does not yet
carry interpolation history between packet boundaries.

## Phase 2 Milestone 6 — output-device selection and preloading

`GET /api/v1/playback/devices` (compatibility `/api/playback/devices`) lists output device IDs, names, default status, and advertised supported sample rates/channel counts where the CPAL host reports them. Headless/disabled hardware exposes a deterministic `memory` device. `GET /api/v1/playback/device` returns `{"device_id":null}` for system default or the selected ID; `POST` accepts `{"device_id":"cpal:<name>"}`, `{"device_id":"memory"}`, or `{"device_id":null}`. A successful change opens/replaces the sink, persists the selected ID in SQLite migration `0004_audio_devices.sql`, and emits `playback.device_changed` on the playback SSE stream. The CPAL stable ID uses the device name and is host-local; if a device is absent or cannot open, selection returns an error and the current sink remains active. Existing running playback state and decoder are retained across sink replacement.

The bounded background pump also prepares the next queued track's Symphonia decoder while the current track is playing; repeat-one prepares the current track. At EOF, `load` consumes the matching prepared decoder rather than opening it then. This is decoder/reader preloading groundwork, not sample-buffer crossfade or strict hardware gapless playback. The existing queue-advance event is emitted when transition is applied. See `playback-devices.json` and `playback-device-selection.json` fixtures.


## Equalizer and metrics (Milestone 7)
`GET`/`POST /api/v1/playback/equalizer` (also `/api/playback/equalizer`) reads/sets `{ "bands": [{"frequency": 1000, "gain_db": 2, "q": 1, "enabled": true}], "preamp_db": 0 }`. Changes emit `playback.equalizer_changed`. `GET /api/v1/playback/metrics` (also `/api/playback/metrics`) returns buffer health, format, sink, drop-rate, and latency estimate fields.
