//! Media item identity contract.

use serde::{Deserialize, Serialize};
use timbre_shared_types::{AlbumId, MediaItemId};

use crate::format::AudioFormat;

/// A media file known to the library.
///
/// The identity is the item's assigned [`MediaItemId`]. The content fingerprint
/// (a BLAKE3 hash, per the Library Roadmap) is owned by the library subsystem
/// and is deliberately not part of this contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaItem {
    /// Identity.
    pub id: MediaItemId,
    /// The album the file belongs to, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumId>,
    /// The audio format as stored on disk.
    pub format: AudioFormat,
    /// File size in bytes.
    pub size_bytes: u64,
}
