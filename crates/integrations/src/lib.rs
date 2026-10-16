//! Integrations: streaming providers and external protocol adapters.
//!
//! Phase 0 skeleton. Providers land with the
//! [Integrations Roadmap](../../Integrations%20Roadmap.md). Every external
//! service sits behind a replaceable adapter, and local playback must keep
//! working when a provider is unavailable.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Integrations;
