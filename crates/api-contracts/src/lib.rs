//! Versioned API request and response models shared by Timbre clients and server.

use serde::{Deserialize, Serialize};

mod device;
mod error;
mod format;
mod library;
mod media;
mod playback;
pub mod runtime;

/// JSON fixture for the playback seek request.
pub const SEEK_FIXTURE: &str = include_str!("../../testing/fixtures/playback-seek.json");

pub use device::{Device, DeviceCapabilities, DeviceCapability, Zone};
pub use error::ApiError;
pub use format::{AudioFormat, SignalPath, SignalPathStep};
pub use library::{Album, Artist, Credit, CreditRole, Recording};
pub use media::MediaItem;
pub use playback::{PlaybackQueue, PlaybackState, QueueItem, RepeatMode, TransportState};
pub use runtime::PlaybackMetricsResponse;
pub use runtime::{
    EqualizerBand, EqualizerSettings, PlayRequest, PlaybackDevice, PlaybackDeviceRequest,
    PlaybackDeviceSelection, PlaybackDevices, PlaybackEvent, PlaybackSnapshot, PlaybackStatus,
    QueueAddRequest, QueueReorderRequest, SeekRequest, ShuffleMode, VolumeRequest,
};
/// Request to set the repeat mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatRequest {
    /// Requested repeat mode.
    pub mode: RepeatMode,
}
/// Request to set the shuffle mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShuffleRequest {
    /// Requested shuffle mode.
    pub mode: ShuffleMode,
}
pub use runtime::QueueMoveRequest;
// Runtime queue move contract includes `pub position: usize`.
/// Queue move position is a zero-based final index after removal.
pub use runtime::{QueueEntry, RuntimePlaybackQueue};
// `QueueMoveRequest` is defined in the runtime contract module.
pub use timbre_shared_types::{
    AlbumId, ArtistId, DeviceId, JobId, MediaItemId, RecordingId, ZoneId,
};

/// Compact request field used for seeking: `pub position: u64`.
/// The wire contract itself is defined in `runtime::SeekRequest`.
/// Seek request field is an absolute `position_ms: u64` value.
/// Current API version prefix.
pub const API_VERSION: &str = "v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_event_round_trips_as_json() {
        let value = PlaybackEvent {
            event: "playback_state".to_owned(),
            sequence: 4,
            occurred_at_ms: 123,
            state: PlaybackSnapshot {
                state: PlaybackStatus::Paused,
                recording_id: Some(RecordingId::generate()),
                position_ms: 4_200,
                duration_ms: Some(180_000),
                volume: 0.5,
                muted: false,
                repeat: RepeatMode::All,
                shuffle: ShuffleMode::On,
                shuffle_enabled: true,
            },
        };
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<PlaybackEvent>(&json).unwrap(), value);
    }
}
