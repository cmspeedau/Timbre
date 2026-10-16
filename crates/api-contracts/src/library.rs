//! Library browse contracts: artists, albums, recordings, and credits.

use serde::{Deserialize, Serialize};
use timbre_shared_types::{AlbumId, ArtistId, RecordingId};

/// An artist or performer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artist {
    /// Identity.
    pub id: ArtistId,
    /// Display name.
    pub name: String,
}

/// A released collection of recordings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    /// Identity.
    pub id: AlbumId,
    /// Album title.
    pub title: String,
    /// The album artist, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<ArtistId>,
    /// Release year, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
}

/// A single recording: one track of an album, independent of its files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recording {
    /// Identity.
    pub id: RecordingId,
    /// Track title.
    pub title: String,
    /// The album it belongs to, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<AlbumId>,
    /// Track number within its album, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_number: Option<u16>,
    /// Duration in milliseconds, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// The role a credit names on a work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditRole {
    /// A performer.
    Performer,
    /// The composer of the work.
    Composer,
    /// A producer.
    Producer,
    /// A recording or mixing engineer.
    Engineer,
}

/// A credit linking an artist to a work in a role.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credit {
    /// The credited artist.
    pub artist: ArtistId,
    /// The role the artist is credited with.
    pub role: CreditRole,
}
