Mobile Roadmap
1. Purpose and scope

Provide a mobile interface for browsing, searching, controlling playback, and listening to music. Support remote control of the server and optional local playback or offline downloads where permitted.

2. Architecture
Slint for the UI, sharing components in crates/ui/ and types in crates/api-contracts/ with the desktop client.
Platform integration written in Rust: jni/ndk on Android and objc2 on iOS for audio sessions, background playback, and media notifications.
Platform-specific secure credential storage.
Local cache for artwork and permitted offline media.
A connection manager for server discovery, pairing, and reconnection.

Do not assume web audio behavior is equivalent to native mobile playback.

3. Implementation phases
Phase 0 — Mobile foundation
- [ ] Configure iOS and Android builds.
- [ ] Set up navigation and shared UI components.
- [ ] Implement connection settings and server pairing.
- [ ] Add secure credential storage.
- [ ] Implement network and permission error states.
- [ ] Exit gate: iOS and Android builds run with navigation, pairing, and secure credential storage.
Phase 1 — Browse and control
- [ ] Add album, artist, and track browsing.
- [ ] Add search and playlists.
- [ ] Implement queue inspection and editing.
- [ ] Add remote playback controls.
- [ ] Display current zone and playback status.
- [ ] Support realtime updates and reconnects.
- [ ] Exit gate: Users can browse, search, and control playback with realtime updates and reconnects.
Phase 2 — Native playback
- [ ] Integrate native playback facilities where required.
- [ ] Handle audio interruptions and route changes.
- [ ] Support background playback.
- [ ] Implement media notifications and system controls where available.
- [ ] Test Bluetooth, wired outputs, and supported external audio devices.
- [ ] Exit gate: Native playback handles interruptions, route changes, background playback, and system controls.
Phase 3 — Offline and remote use
- [ ] Add optional offline downloads.
- [ ] Track download progress and storage usage.
- [ ] Support resumable downloads.
- [ ] Define cache expiry and content-removal behavior.
- [ ] Add remote access through remote-access/.
- [ ] Synchronize playlists, favorites, and history.
- [ ] Exit gate: Offline downloads are cancellable and resumable, and remote access works through remote-access/.
Phase 4 — Mobile polish
- [ ] Optimize startup and scrolling.
- [ ] Add accessibility and dynamic text sizing.
- [ ] Test low-memory and low-connectivity behavior.
- [ ] Improve battery use and background-task handling.
- [ ] Exit gate: Startup, scrolling, accessibility, battery, and low-connectivity behavior are within target.
4. Task checklist
- [ ] Secrets use platform-appropriate secure storage.
- [ ] Background playback follows OS requirements.
- [ ] Reconnection does not create duplicate playback commands.
- [ ] Downloads are cancellable and resumable.
- [ ] Offline content is clearly distinguished from unavailable content.
- [ ] Network failures do not freeze the interface.
- [ ] Permissions are requested only when needed.
5. API and contracts

Consume shared APIs for catalog browsing, playback, queue management, zones, authentication, and downloads.

Define a mobile-specific contract for:

Server pairing.
Session renewal and revocation.
Download authorization and status.
Playback notifications and system transport commands.
Network reconnect and stale-state recovery.
6. Testing strategy
Test on physical iOS and Android devices.
Test backgrounding, lock screen, and audio interruptions.
Test Wi-Fi to cellular transitions.
Test expired sessions and server restarts.
Test offline downloads and interrupted transfers.
Test Bluetooth and external audio routing.
Automate core flows with mobile UI tests.
7. Acceptance criteria
Users can browse and control a paired server.
Playback state remains coherent after reconnects.
Background playback works within OS constraints.
Offline downloads recover from interrupted transfers.
Credentials and session tokens are protected.
Both platforms have documented support requirements.
8. Dependencies and blockers
Depends on server/ APIs and authentication.
Uses catalog data from library/ and discovery/.
Uses zone state from devices/.
Depends on remote-access/ for secure remote connectivity.
Shares contracts with client/.
9. Definition of done

Mobile features must be tested on supported platforms, handle lifecycle and network interruptions, protect credentials, and document OS-specific limitations.
