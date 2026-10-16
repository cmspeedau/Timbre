//! Shared domain types for Timbre.
//!
//! This crate is the innermost layer of the workspace: it depends on nothing
//! else in the project, and every other crate may depend on it. It holds the
//! vocabulary that subsystems share — identifiers, error codes, and job
//! status — but no behavior that belongs to a single subsystem.
//!
//! Contracts that travel over the API live in `timbre-api-contracts` instead.
//! See [ADR-0003](../../docs/adr/0003-workspace-layout-and-dependency-rules.md).

mod error;
mod id;
mod job;
mod subsystem;

pub use error::{DiagnosticContext, Error, ErrorCode};
pub use id::{AlbumId, ArtistId, DeviceId, JobId, MediaItemId, RecordingId, SessionId, ZoneId};
pub use job::{JobProgress, JobState, JobStatus};
pub use subsystem::Subsystem;
