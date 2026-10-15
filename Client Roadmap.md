Client Roadmap
1. Purpose and scope

Build the primary browsing and playback-control experience. The client should make large music collections intuitive to explore while maintaining a persistent, responsive player.

The client does not own the authoritative library, playback queue, or audio pipeline.

2. Architecture
Slint for the UI: one cross-platform Rust codebase shared with the mobile client.
Shared Slint components live in crates/ui/ and shared types in crates/api-contracts/.
No web engine, JavaScript runtime, or browser is required at runtime.
Feature modules for browsing, search, queue, player, playlists, and settings.
A dedicated playback-state store and a separate transient UI-state store.
3. Implementation phases
Phase 0 — Application foundation
- [ ] Set up the application shell, routing, and design tokens.
- [ ] Configure linting, type checking, and component tests.
- [ ] Add server connection configuration.
- [ ] Implement loading, error, and disconnected states.
- [ ] Establish keyboard navigation and accessibility conventions.
- [ ] Exit gate: The application shell, routing, design tokens, and connection and error states are in place.
Phase 1 — Library browsing
- [ ] Build album and artist grids.
- [ ] Build track and release tables.
- [ ] Implement album and artist detail views.
- [ ] Add pagination or virtualized lists for large catalogs.
- [ ] Add artwork loading and fallback images.
- [ ] Support sorting and basic filtering.
- [ ] Exit gate: Users can browse albums and artists and open detail views over a large catalog.
Phase 2 — Player and queue
- [ ] Implement a persistent now-playing panel.
- [ ] Add play, pause, seek, previous, and next controls.
- [ ] Build queue inspection, reordering, and removal.
- [ ] Add shuffle and repeat controls.
- [ ] Implement playlists and favorites.
- [ ] Restore the last selected view and appropriate playback state.
- [ ] Exit gate: A persistent player and queue work with playlists, favorites, and restored state.
Phase 3 — Search and discovery
- [ ] Add global search.
- [ ] Add search suggestions and keyboard shortcuts.
- [ ] Add filters for artist, album, genre, and release date.
- [ ] Link artists, contributors, and related releases.
- [ ] Add recently played and recently added views.
- [ ] Exit gate: Global search, filters, and related-entity navigation work over the library.
Phase 4 — Desktop polish
- [ ] Add native application packaging.
- [ ] Add system media controls where supported.
- [ ] Implement window and playback preferences.
- [ ] Improve accessibility, localization readiness, and high-DPI behavior.
- [ ] Add client diagnostics and an error-reporting workflow.
- [ ] Exit gate: The desktop build installs, launches, and supports accessibility and system media controls on each OS.
4. Task checklist
- [ ] Playback controls remain available during navigation.
- [ ] API errors have useful user-facing messages.
- [ ] Search and browsing do not block the UI thread.
- [ ] Loading and empty states are consistent.
- [ ] Queue state updates correctly when changed by another client.
- [ ] Keyboard and screen-reader navigation are supported.
- [ ] UI components are reusable rather than duplicated across screens.
5. API and contracts

Consume versioned APIs for catalog browsing, search, playback commands, queue management, playlists, device selection, and realtime updates.

Rules:

Never access the server database directly.
Treat realtime events as updates to server-owned state.
Handle stale events and reconnects.
Keep UI-only state out of persistent playback records.
Use shared typed models instead of recreating API types manually.
6. Testing strategy
Unit-test state transitions and formatting utilities.
Component-test queue, search, and player controls.
Use Playwright for core user journeys.
Test API errors and network interruption.
Test large-library rendering performance.
Test keyboard accessibility and responsive layouts.
Verify synchronization between multiple connected clients.
7. Acceptance criteria
A new user can find an album and start playback without documentation.
Playback controls remain responsive while browsing.
Queue changes are reflected consistently.
Search and browsing support large libraries.
Reconnection restores accurate server state.
The desktop build installs and launches on each supported operating system.
8. Dependencies and blockers
Depends on server/ API contracts.
Depends on library/ catalog models.
Uses playback and signal-path data from audio-engine/.
Uses zone and device data from devices/.
Shares components and types with mobile/ where appropriate.
9. Definition of done

Every UI feature must have loading, empty, success, and error states where applicable; accessible controls; automated tests; and documented API dependencies.
