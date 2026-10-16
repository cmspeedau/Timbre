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
