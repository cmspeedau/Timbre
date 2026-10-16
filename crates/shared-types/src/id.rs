//! Opaque, strongly typed identifiers for the shared domain model.
//!
//! Each identifier wraps a [`Uuid`] so the compiler prevents one kind of
//! identity being used where another is expected. The internal representation
//! is deliberately opaque; subsystems treat identifiers as values to store and
//! compare, not to inspect.

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Declares a strongly typed identifier backed by a [`Uuid`].
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new, random identifier.
            #[must_use]
            pub fn generate() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wraps an existing UUID, for example one read from the database.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self)
            }
        }
    };
}

define_id! {
    /// Identifies one physical media file known to the library.
    MediaItemId
}

define_id! {
    /// Identifies an artist or performer.
    ArtistId
}

define_id! {
    /// Identifies a released collection of recordings.
    AlbumId
}

define_id! {
    /// Identifies a single recording, independent of the files that hold it.
    RecordingId
}

define_id! {
    /// Identifies an output device.
    DeviceId
}

define_id! {
    /// Identifies a named playback destination.
    ZoneId
}

define_id! {
    /// Identifies a background job.
    JobId
}

define_id! {
    /// Identifies an authenticated client session.
    SessionId
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_distinct_ids() {
        assert_ne!(MediaItemId::generate(), MediaItemId::generate());
    }

    #[test]
    fn roundtrips_through_text() {
        let id = ArtistId::generate();
        let parsed: ArtistId = id.to_string().parse().expect("id text should parse");
        assert_eq!(id, parsed);
    }

    #[test]
    fn distinguishes_id_kinds_at_the_type_level() {
        // Compile-time property: a `DeviceId` is not an `AlbumId`. This test
        // exists to show the intent; the assertion is that the code compiles.
        let device = DeviceId::from_uuid(uuid::Uuid::nil());
        assert_eq!(device.as_uuid(), uuid::Uuid::nil());
    }
}
