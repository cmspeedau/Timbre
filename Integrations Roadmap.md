Integrations Roadmap
1. Purpose and scope

Connect the platform to external metadata sources, streaming catalogs, and supported playback protocols through isolated, replaceable adapters.

Integrations must not become mandatory dependencies for local-library operation.

2. Architecture
providers/metadata/ — metadata provider adapters.
providers/streaming/ — streaming catalog and playback adapters.
protocols/ — supported device and network protocol adapters.
auth/ — provider-specific authentication.
capabilities/ — supported operations and limitations.
sync/ — permitted playlist and library synchronization.
diagnostics/ — provider status, errors, and rate limits.

Every adapter implements a shared interface while retaining provider-specific capabilities.

Stack: reqwest, oauth2, and librespot for Spotify Connect.

3. Implementation phases
Phase 0 — Integration framework
- [ ] Define adapter interfaces and capability declarations.
- [ ] Define common error categories and retry policies.
- [ ] Establish secret storage and credential redaction.
- [ ] Document provider licensing and API review requirements.
- [ ] Build mock adapters for integration testing.
- [ ] Exit gate: Adapter interfaces, capabilities, error categories, and mock adapters are defined.
Phase 1 — Metadata providers
- [ ] Integrate the chosen metadata services.
- [ ] Implement rate limits and response caching.
- [ ] Record provenance and attribution.
- [ ] Handle timeouts, provider errors, and schema changes.
- [ ] Ensure provider failure does not interrupt playback.
- [ ] Exit gate: Metadata providers work with rate limits, caching, provenance, and failure isolation.
Phase 2 — Playback protocols
- [ ] Evaluate supported protocols and available libraries.
- [ ] Implement selected protocols through devices/ adapters.
- [ ] Expose capability limitations accurately.
- [ ] Add interoperability tests with compatible hardware.
- [ ] Document pairing and network-security requirements.
- [ ] Exit gate: Selected playback protocols interoperate with compatible hardware and document their limits.
Phase 3 — Streaming providers
- [ ] Identify target services with supported APIs.
- [ ] Confirm catalog, playback, and caching permissions.
- [ ] Implement supported authentication flows.
- [ ] Represent external catalog entries separately from local recordings.
- [ ] Support permitted playlist synchronization.
- [ ] Handle entitlement changes and expired credentials.
- [ ] Exit gate: Supported streaming providers authenticate, keep external IDs separate, and degrade on entitlement changes.
Phase 4 — Long-term maintenance
- [ ] Monitor API deprecations and provider policy changes.
- [ ] Add contract tests and mock fixtures.
- [ ] Version adapters independently where necessary.
- [ ] Provide user-facing connection diagnostics.
- [ ] Define graceful degradation when a provider is discontinued.
- [ ] Exit gate: Deprecation monitoring, contract tests, and user-facing diagnostics are in place.
4. Task checklist
- [ ] Each integration has documented capabilities and limitations.
- [ ] Credentials are encrypted or stored in an appropriate secure facility.
- [ ] Rate limits and retry behavior are bounded.
- [ ] External errors are isolated from core playback.
- [ ] Provider data retains its provenance.
- [ ] Unsupported operations are not advertised as functional.
- [ ] Integration tests can run without live provider accounts.
5. API and contracts

Each adapter should expose a standard lifecycle, capability description, health state, and typed operations.

Use provider-specific models internally and translate to canonical application entities at the boundary.

Never merge external catalog identifiers into local primary keys. Preserve the mapping between provider IDs and canonical entities.

6. Testing strategy
Test adapters against mocks and recorded fixtures.
Test expired credentials and rate limits.
Test malformed provider responses and API timeouts.
Verify local playback during provider outages.
Test protocol interoperability on compatible devices.
Review licensing and policy constraints before release.
7. Acceptance criteria
An adapter can be disabled without breaking unrelated subsystems.
Provider capabilities and limitations are explicit.
Authentication failures are recoverable.
External IDs do not corrupt local catalog identity.
Every shipped integration has documented legal and technical constraints.
8. Dependencies and blockers
Uses server/ for configuration, jobs, and authentication coordination.
Enriches library/ through its metadata interfaces.
Uses devices/ for playback protocol integration.
May supply catalog items to discovery/ and client/.
Requires deployment support for provider configuration and secrets.
9. Definition of done

An integration requires documented permissions, capability mapping, secure credentials, automated tests, failure isolation, and a maintenance plan.
