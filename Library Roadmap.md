Library Roadmap
1. Purpose and scope

Maintain an accurate, searchable catalog of the user's music files, separating physical files from logical recordings, releases, artists, and user-curated metadata.

The library must remain useful when external metadata providers are unavailable.

2. Architecture

Modules:

scanner/ — directory traversal and incremental scans.
extractors/ — tags, duration, codec, and technical properties.
catalog/ — tracks, releases, artists, and credits.
storage/ — file references and storage locations.
artwork/ — embedded artwork, caching, resizing.
matching/ — duplicate detection and candidate identification.
metadata/ — provenance and user overrides.
events/ — library-change notifications.

Use SQLite in WAL mode through sqlx as the default store, as defined in the Master Roadmap. Use stable internal IDs and explicit relationships between media files and catalog entities.

Stack: lofty for tags and technical properties, walkdir for traversal, blake3 for file fingerprints, image for artwork decoding, sqlx for persistence.

3. Implementation phases
Phase 0 — Data model
- [x] Define initial artist, album, recording, and scanned-track models at the library/server boundary.
- [x] Define in-memory configured library roots and scan-job results.
- [ ] Define durable media-file/catalog schemas, artwork references, provenance, and user overrides.
- [ ] Separate source tags, matched metadata, and user overrides.
- [ ] Add library schema migrations and indexes (the current SQLite migrations persist playback state, not the catalog).
- [ ] Exit gate: The catalog schema, migrations, and indexes exist and separate files, recordings, and user overrides.
Phase 1 — File scanning
- [x] Recursively scan explicitly configured and canonicalized roots.
- [x] Support extension-filtered audio discovery with Lofty metadata extraction and WAV property fallback.
- [x] Extract title/artist/album/track and available duration, channel count, sample rate, and bit depth.
- [x] Keep file paths and metadata in scan results and the current in-memory catalog.
- [x] Isolate malformed-file errors so other files continue scanning.
- [ ] Track changed, moved, and missing files durably across repeat scans.
- [ ] Persist roots, catalog, and scan jobs across restarts.
- [ ] Report incremental progress and support cancellation.
- [ ] Exit gate: A configured folder scans without manual database setup and reports progress and cancellation.
Phase 2 — Catalog normalization
- [x] Create initial artist and album relationships from extracted tags and stable in-memory recording IDs.
- [x] Handle missing titles/artists with filename and “Unknown Artist” fallbacks.
- [ ] Support multi-disc albums and compilations.
- [ ] Preserve original tag values separately from normalized/user-edited values.
- [ ] Detect likely duplicates using identifiers and file fingerprints.
- [x] Keep scanning non-destructive; no source files are modified or removed.
- [ ] Exit gate: Repeated scans are idempotent and produce normalized, non-destructive catalog records.
Phase 3 — Artwork and enrichment
- [ ] Extract embedded artwork.
- [ ] Cache thumbnails and larger images.
- [ ] Add external metadata matching through a provider interface.
- [ ] Record provider, timestamp, and confidence for matched fields.
- [ ] Add manual correction and override support.
- [ ] Publish incremental library-change events.
- [ ] Exit gate: Artwork is cached, external matches carry provenance, and library-change events publish incrementally.
Phase 4 — Scale and recovery
- [ ] Optimize incremental scans.
- [ ] Add resumable jobs.
- [ ] Test very large directories and deep folder structures.
- [ ] Handle unavailable network mounts.
- [ ] Add catalog consistency checks and recovery tooling.
- [ ] Exit gate: Incremental scans are resumable and survive large libraries and unavailable mounts.
4. Task checklist
- [ ] Repeated scans are idempotent.
- [ ] File identity is separate from track identity.
- [ ] Missing files do not immediately erase catalog history.
- [ ] User edits survive scans and metadata refreshes.
- [ ] Artwork caching does not duplicate large binary data unnecessarily.
- [ ] Scanning does not block playback.
- [ ] File paths are validated and constrained to configured locations.
5. API and contracts

Provide internal interfaces for:

ScanLibrary(locationID)
GetTrack(trackID)
ListAlbums(filter, pagination)
ListArtists(filter, pagination)
SearchCatalog(query, filters)
GetArtwork(artworkID, size)
UpdateUserMetadata(entityID, patch)
SubscribeLibraryChanges()

Define catalog entities separately from media-file records. Expose normalized data to the server rather than making clients query the database directly.

6. Testing strategy
Test tags with real-world fixture files.
Test malformed and partially tagged media.
Test duplicate and conflicting identifiers.
Test repeated scans and file changes.
Test database migrations against existing catalogs.
Test user overrides across rescans.
Test inaccessible folders and interrupted scans.
Test artwork caching and invalid images.
7. Acceptance criteria
A configured library can be indexed without manual database setup.
Scans can be repeated without duplicating catalog records.
Albums and artists are browsable even with incomplete tags.
Metadata provenance is visible and corrections persist.
Missing or unavailable files are represented accurately.
External provider outages do not break local catalog browsing.
8. Dependencies and blockers
Depends on server configuration and persistence conventions.
Supplies catalog APIs to server/ and client/.
Supplies media references to audio-engine/.
Supplies normalized entities to discovery/.
Requires representative audio fixtures from testing/.
9. Definition of done

A library feature is complete when its data model, migration, scanning behavior, failure handling, tests, and API contract are documented and verified against both clean and existing libraries.
