//! Discovery: metadata enrichment, search, relationships, and recommendations.
//!
//! Phase 0 skeleton. Enrichment and search land with the
//! [Discovery Roadmap](../../Discovery%20Roadmap.md). Enrichment must never
//! block local playback.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Discovery;
