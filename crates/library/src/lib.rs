//! The library: scanning, metadata, database, artwork, and indexing.
//!
//! Provides recursive audio discovery, metadata extraction, and the repository
//! write boundary for the [Library Roadmap](../../Library%20Roadmap.md).

/// Audio metadata parsing through Lofty.
pub mod parser;
/// Recursive filesystem traversal and ingestion.
pub mod scanner;

pub use parser::{AudioMetadata, format_label, parse_audio_file};
pub use scanner::{
    AudioMetadataRepository, SUPPORTED_EXTENSIONS, ScanIssue, ScanReport, ScannedAudioFile,
    has_supported_extension, scan_directory, scan_into_repository,
};

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Library;
