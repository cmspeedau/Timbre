//! Versioned API contracts shared by the server, clients, and device adapters.
//!
//! Every type here is part of a published contract. Changing one follows the
//! compatibility process in the [Master Roadmap](../../Master%20Roadmap.md):
//! update the schema, assess compatibility, update the consumer tests, and add
//! migration notes where needed.
//!
//! Types are defined once, here, and referenced by subsystem crates rather than
//! restated. See [ADR-0003](../../docs/adr/0003-workspace-layout-and-dependency-rules.md)
//! for how the crates fit together.

pub mod device;
pub mod error;
pub mod format;
pub mod library;
pub mod media;
pub mod playback;

pub use device::{Device, DeviceCapabilities, DeviceCapability, Zone};
pub use error::ApiError;
pub use format::{AudioFormat, SignalPath, SignalPathStep};
pub use library::{Album, Artist, Credit, CreditRole, Recording};
pub use media::MediaItem;
pub use playback::{PlaybackQueue, PlaybackState, QueueItem, RepeatMode, TransportState};

/// The version of the API contract implemented by this crate.
///
/// The value appears in request paths (`/api/v1/...`). It changes only when a
/// breaking revision of the contract ships, never for additive changes.
pub const API_VERSION: &str = "v1";
