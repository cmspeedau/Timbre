//! The subsystems that make up the platform.

use serde::{Deserialize, Serialize};

/// A named part of the platform, aligned one-to-one with a workspace crate.
///
/// Used for diagnostics, health reporting, and ownership. See
/// [ADR-0003](../../docs/adr/0003-workspace-layout-and-dependency-rules.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subsystem {
    /// API, configuration, orchestration, and authentication.
    Server,
    /// Scanning, metadata, database, artwork, and indexing.
    Library,
    /// Decoding, processing, rendering, and the signal path.
    AudioEngine,
    /// Output discovery, zones, endpoints, and synchronization.
    Devices,
    /// Metadata enrichment, search, relationships, and recommendations.
    Discovery,
    /// Pairing, secure connectivity, and remote sessions.
    RemoteAccess,
    /// Streaming providers and external protocol adapters.
    Integrations,
    /// Shared client user interface.
    Ui,
    /// Cross-system quality and validation.
    Testing,
}
