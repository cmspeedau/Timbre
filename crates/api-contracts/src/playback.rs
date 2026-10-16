//! Playback state and queue contracts.

use serde::{Deserialize, Serialize};
use timbre_shared_types::{MediaItemId, RecordingId, ZoneId};

use crate::format::SignalPath;

/// The transport state of a zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportState {
    /// Stopped.
    Stopped,
    /// Playing.
    Playing,
    /// Paused.
    Paused,
}

/// How the queue repeats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatMode {
    /// Stop at the end of the queue.
    #[default]
    Off,
    /// Repeat the whole queue.
    All,
    /// Repeat the current item.
    One,
}

/// One entry in a playback queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItem {
    /// The recording to play.
    pub recording: RecordingId,
    /// The specific media file to play, once a choice has been made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_item: Option<MediaItemId>,
}

/// The playback queue of one zone.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackQueue {
    /// Items in order.
    pub items: Vec<QueueItem>,
    /// Index of the current item, when the queue is not empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_index: Option<usize>,
    /// Repeat behavior.
    pub repeat: RepeatMode,
    /// Whether the queue plays in random order.
    pub shuffle: bool,
}

/// The observable playback state of one zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackState {
    /// The zone this state describes.
    pub zone: ZoneId,
    /// Transport state.
    pub transport: TransportState,
    /// The queue.
    pub queue: PlaybackQueue,
    /// Position within the current item, in milliseconds.
    pub position_ms: u64,
    /// The current signal path, when a stream is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_path: Option<SignalPath>,
}
