# ADR-0006: Licensing

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A self-hosted music platform benefits from wide adoption: contributors, packagers,
and forks. The project copies no proprietary implementation details, branding, or
protocols, so there is no encumbrance preventing a permissive license. A license
must be chosen before outside contributions arrive, because the terms then cannot
be changed without every contributor's agreement.

## Decision

License the project under **MIT OR Apache-2.0**, the Rust ecosystem convention.

- `LICENSE-MIT` and `LICENSE-APACHE` hold the full texts.
- `Cargo.toml` declares `license = "MIT OR Apache-2.0"`.
- Contributors license their contributions under the same terms, which the
  repository states; no separate contributor agreement is required.

## Consequences

Users may choose either license, which is what most Rust users expect, and the
Apache-2.0 option supplies an explicit patent grant. Contributions remain
compatible with the surrounding Rust ecosystem.

The project gains no copyleft protection: derivatives, including closed ones, are
permitted. This was chosen deliberately for adoption over restriction.
