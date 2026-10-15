Devices Roadmap
1. Purpose and scope

Discover, describe, and manage audio output destinations. Provide a unified abstraction for local outputs, network endpoints, and grouped playback zones.

Device management coordinates endpoints but delegates decoding and DSP to audio-engine/.

2. Architecture

Modules:

discovery/ — enumeration and endpoint discovery.
registry/ — stable device identity and capability records.
adapters/ — platform and network protocol integrations.
zones/ — named destinations and groups.
sessions/ — endpoint lifecycle and playback assignment.
sync/ — clock, buffering, and timing coordination.

Treat protocol capabilities as explicit, discoverable data rather than assuming every device supports the same features.

Stack: mdns-sd for discovery, tokio for asynchronous I/O, and rustls for transport security. RAAT is proprietary and unavailable; multi-room synchronization must be designed independently and measured.

3. Implementation phases
Phase 0 — Device model
- [ ] Define device, capability, zone, and endpoint-session schemas.
- [ ] Define device lifecycle and connection states.
- [ ] Separate physical device identity from network address.
- [ ] Define error and reconnect behavior.
- [ ] Exit gate: Device, capability, zone, and session schemas are defined with lifecycle and reconnect behavior.
Phase 1 — Local output
- [ ] Enumerate local audio devices.
- [ ] Expose supported sample rates, channels, and formats.
- [ ] Allow selecting a default output.
- [ ] Detect disconnection and reconnection.
- [ ] Display device status in the client.
- [ ] Exit gate: Local outputs are enumerated, selectable, and recover from disconnection.
Phase 2 — Independent zones
- [ ] Create named zones.
- [ ] Bind zones to supported output endpoints.
- [ ] Track per-zone playback and volume state.
- [ ] Allow switching the active output.
- [ ] Prevent incompatible device configurations.
- [ ] Recover gracefully from unavailable devices.
- [ ] Exit gate: Named zones hold independent playback and volume state and reject incompatible configurations.
Phase 3 — Network endpoints
- [ ] Evaluate supported standards and libraries.
- [ ] Implement selected protocol adapters.
- [ ] Add endpoint discovery and pairing where applicable.
- [ ] Validate network endpoint capabilities.
- [ ] Add latency and buffering measurements.
- [ ] Document unsupported protocol features.
- [ ] Exit gate: Selected network protocols are implemented with measured latency and documented limitations.
Phase 4 — Synchronized groups
- [ ] Select a synchronization protocol or design.
- [ ] Establish clock-offset and drift measurements.
- [ ] Coordinate buffering and start times.
- [ ] Implement group membership and master selection.
- [ ] Recover from packet loss and endpoint reconnection.
- [ ] Validate synchronization across heterogeneous devices.
- [ ] Exit gate: Grouped playback has measured synchronization, clock-offset handling, and recovery from endpoint loss.
4. Task checklist
- [ ] Devices have stable identities.
- [ ] Capability negotiation occurs before playback.
- [ ] Offline endpoints do not crash the server.
- [ ] Zone state is separate from physical device state.
- [ ] Network discovery does not expose unsafe control surfaces.
- [ ] Sync quality is measured rather than assumed.
- [ ] Grouping restrictions are documented.
5. API and contracts

Provide interfaces for:

Discovering and listing devices.
Reading capabilities and connection status.
Creating, updating, and deleting zones.
Assigning an endpoint to a zone.
Starting, stopping, and moving zone playback.
Reading timing and synchronization metrics.

Coordinate with the audio engine through a stable output-adapter contract.

6. Testing strategy
Test device enumeration and identity stability.
Simulate endpoint loss and reconnects.
Test incompatible sample-rate and channel configurations.
Test zone reassignment during playback.
Measure synchronized start offsets and drift.
Test network interruptions and congested connections.
Validate supported hardware on each target operating system.
7. Acceptance criteria
Local outputs can be selected and used reliably.
Each independent zone has coherent playback state.
Unsupported configurations are rejected clearly.
Disconnected devices recover without server failure.
Group playback has measurable synchronization behavior.
Protocol limitations are visible to users.
8. Dependencies and blockers
Depends on audio-engine/ output interfaces.
Coordinates zone commands with server/.
Supplies device information to client/ and mobile/.
Multi-room work depends on validated timing measurements and compatible endpoints.
Uses testing/ for hardware and network simulations.
9. Definition of done

A device adapter must document supported capabilities, failure modes, discovery behavior, security assumptions, tests, and compatibility limitations.
