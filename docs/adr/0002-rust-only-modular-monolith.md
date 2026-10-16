# ADR-0002: A Rust-only modular monolith

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The product is a Roon-inspired music platform: library management, playback,
metadata, multi-room devices, discovery, remote access, and deployment. The
Master Roadmap sets two hard constraints: every component is written in Rust, and
no component may require a non-Rust runtime, interpreter, or web engine. The
project must also avoid a distributed architecture before it is justified, and it
must not depend on proprietary protocols such as RAAT.

The forces in tension are speed of delivery, operational simplicity, and the
long-term need to run playback independently of the UI and of metadata work.

## Decision

Build a **modular monolith** in a single Cargo workspace.

- One server process owns authoritative library and playback-session state.
- Subsystems are separate crates with explicit, downward-only dependencies, not
  separate services. See [ADR-0003](0003-workspace-layout-and-dependency-rules.md).
- Inter-subsystem communication is through Rust types in `timbre-shared-types`
  and `timbre-api-contracts`, not through a network boundary.
- Clients and optional playback endpoints are separate binaries that talk to the
  server over a versioned API.
- No non-Rust runtime, interpreter, web engine, or UI toolkit is introduced.

## Consequences

Deployment is a single server binary plus clients, which keeps operations and
testing simple. Refactoring a boundary is a compile-time concern and cheap.

The trade-off is that subsystems share a process: a panic or blocking work in one
can affect the others. The Master Roadmap addresses this directly — long-running
jobs must never block audio rendering, and playback must continue independently
of UI rendering and metadata enrichment. Those constraints are enforced in the
subsystem designs, not by process isolation.

Choosing a Rust-only stack is also why mobile is the long pole: background
playback, lock-screen controls, and audio-session handling need direct platform
APIs rather than a web runtime.
