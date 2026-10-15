Remote Access Roadmap
1. Purpose and scope

Allow authorized clients to reach a music server outside the local network while protecting the user's library, credentials, and home network.

Remote access is optional. Local playback must work without it.

2. Architecture
pairing/ — explicit client enrollment and revocation.
identity/ — server and client identities.
sessions/ — access tokens and renewal.
transport/ — secure direct or relay connectivity.
authorization/ — permission checks.
diagnostics/ — connectivity and session health.

Stack: rustls and tokio-rustls for transport, with axum for authenticated gateway endpoints.

Prefer a VPN or an authenticated, encrypted gateway for initial releases. Evaluate relays only when their operational and privacy implications are understood.

3. Implementation phases
Phase 0 — Threat model
- [ ] Define attacker scenarios and trust boundaries.
- [ ] Identify data exposed by each remote API.
- [ ] Establish encryption and credential-storage requirements.
- [ ] Define pairing, revocation, and recovery procedures.
- [ ] Exit gate: A threat model defines trust boundaries, exposed data, and pairing and revocation procedures.
Phase 1 — Secure local sessions
- [ ] Implement client registration and pairing.
- [ ] Issue scoped session credentials.
- [ ] Add session expiry and revocation.
- [ ] Enforce authorization on protected APIs.
- [ ] Protect secrets and sensitive diagnostics.
- [ ] Exit gate: Clients pair, receive scoped sessions, and are authorized and revocable.
Phase 2 — Remote connectivity
- [ ] Document a secure VPN deployment path.
- [ ] Add connection diagnostics.
- [ ] Implement authenticated remote gateway support if needed.
- [ ] Validate TLS and certificate handling.
- [ ] Prevent unauthenticated public exposure.
- [ ] Add rate limiting and abuse monitoring.
- [ ] Exit gate: Authenticated encrypted remote connectivity works with diagnostics and no unauthenticated exposure.
Phase 3 — Remote media access
- [ ] Authorize library and artwork requests.
- [ ] Support remote playback commands.
- [ ] Handle bandwidth limits and interrupted streams.
- [ ] Add optional secure relay support only if justified.
- [ ] Implement client revocation and suspicious-session handling.
- [ ] Exit gate: Remote library, artwork, and playback requests are authorized with bandwidth and interruption handling.
Phase 4 — Hardening
- [ ] Test expired and stolen credentials.
- [ ] Audit access-control decisions.
- [ ] Add session and connection diagnostics.
- [ ] Document secure upgrades and key rotation.
- [ ] Review privacy implications of any relay infrastructure.
- [ ] Exit gate: Security tests cover expired credentials, revocation, and rate limits, with documented key rotation.
4. Task checklist
- [ ] Remote access is opt-in.
- [ ] All remote commands are authenticated and authorized.
- [ ] Sessions can be revoked.
- [ ] Credentials are never logged.
- [ ] Public endpoints have abuse controls.
- [ ] Filesystem paths cannot be selected arbitrarily by remote clients.
- [ ] Connection failures do not affect local playback.
5. API and contracts

Define contracts for pairing, session creation and renewal, revocation, connection status, and remote playback authorization.

The transport layer must not bypass server authorization. Device identity, session identity, and user identity should remain distinct.

6. Testing strategy
Test pairing and revocation.
Test expired and invalid tokens.
Test unauthorized file and playback requests.
Run network interruption and reconnect tests.
Test TLS configuration and certificate failures.
Perform security reviews of public-facing endpoints.
Test rate limits and repeated failed authentication attempts.
7. Acceptance criteria
Users can explicitly authorize and revoke clients.
Remote traffic uses an authenticated encrypted transport.
Unauthenticated requests cannot access protected content.
A failed remote connection does not disable local functionality.
Session and connection status are diagnosable.
8. Dependencies and blockers
Depends on server/ authentication and authorization.
Serves mobile/ and remotely connected client/ instances.
Uses playback and catalog APIs without bypassing their contracts.
Requires security review and deployment support from testing/ and deployment/.
9. Definition of done

Remote-access features require a documented threat model, security tests, revocation support, operational guidance, and explicit privacy limitations.
