Deployment Roadmap
1. Purpose and scope

Make the platform straightforward to install, configure, update, back up, diagnose, and uninstall across supported environments.

The first deployment target should be a single server with persistent storage, with desktop packaging and optional endpoint agents added as needed.

2. Architecture
containers/ — development and server container definitions.
installers/ — platform-specific installation packages.
config/ — configuration templates and validation.
migrations/ — database upgrade execution.
backup/ — database, configuration, and metadata backup.
monitoring/ — health checks, logs, metrics.
release/ — versioning and artifact generation.
docs/ — installation, upgrade, recovery, and troubleshooting.

Build: cargo build --release, a musl static target for portable binaries, and distroless container images. Containers are for development and packaging, not a runtime requirement.
3. Implementation phases
Phase 0 — Developer deployment
- [ ] Create Docker Compose configuration for development.
- [ ] Define persistent database and cache volumes.
- [ ] Add environment variable and configuration documentation.
- [ ] Add health checks and startup ordering.
- [ ] Document local development and test commands.
- [ ] Exit gate: Docker Compose, persistent volumes, health checks, and configuration docs enable development.
Phase 1 — Self-hosted MVP
- [ ] Package the server with a documented configuration file.
- [ ] Support configurable music-library mount paths.
- [ ] Persist the database and artwork cache.
- [ ] Document filesystem permissions.
- [ ] Add health and readiness reporting.
- [ ] Provide a secure default network configuration.
- [ ] Exit gate: A packaged server runs from a documented config with persistent data and secure defaults.
Phase 2 — Backup and upgrades
- [ ] Implement consistent database backups.
- [ ] Document media-file backup responsibilities separately.
- [ ] Add restore verification.
- [ ] Run migrations with explicit failure handling.
- [ ] Preserve configuration across upgrades.
- [ ] Document rollback and recovery limitations.
- [ ] Exit gate: Backups restore on a clean install and migrations run with explicit failure handling.
Phase 3 — User-facing installation
- [ ] Build desktop installers for supported operating systems.
- [ ] Automate release artifact generation.
- [ ] Add version reporting and update guidance.
- [ ] Sign artifacts where supported and practical.
- [ ] Document clean installation and uninstallation.
- [ ] Add migration notes for breaking changes.
- [ ] Exit gate: Desktop installers build and launch on each supported OS with reproducible artifacts.
Phase 4 — Production operations
- [ ] Add structured log collection.
- [ ] Add CPU, memory, storage, and playback metrics.
- [ ] Add diagnostic export with sensitive data redaction.
- [ ] Establish release channels and rollback procedures.
- [ ] Test restart and recovery scenarios.
- [ ] Publish supported configuration and upgrade matrices.
- [ ] Exit gate: Logs, metrics, diagnostics, and rollback procedures support production operations.
4. Task checklist
- [ ] Configuration is validated before startup.
- [ ] Persistent data is not stored only inside disposable containers.
- [ ] Backups can be restored on a clean installation.
- [ ] Database migrations are tested against previous releases.
- [ ] Secrets are excluded from logs and diagnostic bundles.
- [ ] Installation does not require unsafe default network exposure.
- [ ] Upgrade failures produce actionable messages.
- [ ] Release artifacts are reproducible and versioned.
5. API and contracts

Deployment relies on stable health, readiness, diagnostics, configuration, and version interfaces from server/.

Define explicit compatibility requirements for:

Server and database versions.
Client and server API versions.
Configuration schema versions.
Database migrations.
Backup formats.
Optional endpoint-agent versions.
6. Testing strategy
Test clean installation and first startup.
Test upgrades from supported previous versions.
Test database backup and restore.
Test missing configuration and invalid permissions.
Test container restarts and persistent storage.
Test installer launch and uninstall behavior.
Test release artifact integrity and supported OS compatibility.
7. Acceptance criteria
A user can follow the installation guide successfully.
Data survives expected restarts and upgrades.
Backups restore into a documented supported version.
Configuration and network requirements are explicit.
Diagnostic reports help identify common failures without leaking secrets.
Each release has a clear version, changelog, and recovery procedure.
8. Dependencies and blockers
Packages server/ and its runtime dependencies.
Depends on migrations and storage requirements from library/.
Packages client/ and mobile/ through their respective distribution workflows.
Uses testing/ release gates.
Depends on remote-access/ for secure network configuration guidance.
9. Definition of done

A deployment feature requires reproducible build or installation steps, documented compatibility, automated validation, recovery instructions, and a tested failure path.
