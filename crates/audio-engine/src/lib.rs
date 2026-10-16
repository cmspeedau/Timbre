//! The audio engine: decoding, processing, rendering, and the signal path.
//!
//! Phase 0 skeleton. Decoding and the processing pipeline land with the
//! [Audio Engine Roadmap](../../Audio%20Engine%20Roadmap.md).

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::AudioEngine;
