//! Runtime playback API request and response models.
#![allow(unreachable_pub)]
use serde::{Deserialize, Serialize};
use timbre_shared_types::RecordingId;

/// Shuffle mode for queue traversal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShuffleMode {
    #[default]
    /// Do not randomize queue traversal.
    Off,
    /// Randomize queue traversal.
    On,
}
/// Transport lifecycle in the playback API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackStatus {
    /// Playback is stopped.
    Stopped,
    /// Playback is active.
    Playing,
    /// Playback is paused.
    Paused,
    /// The current track reached its end.
    TrackEnded,
}
/// Request to load and start a catalog recording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayRequest {
    /// Recording to start.
    pub recording_id: RecordingId,
}
/// Absolute seek target in milliseconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeekRequest {
    /// Absolute position in milliseconds.
    pub position_ms: u64,
}
/// One output device and its advertised capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDevice {
    /// Stable device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Whether this is the system default device.
    pub is_default: bool,
    #[serde(default)]
    /// Supported sample rates in hertz.
    pub sample_rates: Vec<u32>,
    #[serde(default)]
    /// Supported channel counts.
    pub channels: Vec<u16>,
}
/// Available output devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDevices {
    /// Available output devices.
    pub devices: Vec<PlaybackDevice>,
}
/// Selected output device; `None` selects the system default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDeviceSelection {
    /// Selected device identifier; `None` chooses the system default.
    pub device_id: Option<String>,
}
/// Alias for selecting an output device.
pub type PlaybackDeviceRequest = PlaybackDeviceSelection;
/// Parametric EQ band.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand {
    /// Center frequency in hertz.
    pub frequency: f32,
    /// Gain in decibels.
    pub gain_db: f32,
    /// Dimensionless filter Q.
    pub q: f32,
    #[serde(default = "enabled_by_default")]
    /// Whether this band is applied.
    pub enabled: bool,
}
fn enabled_by_default() -> bool {
    true
}
/// EQ settings applied to playback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerSettings {
    /// Parametric equalizer bands.
    pub bands: Vec<EqualizerBand>,
    #[serde(default)]
    /// Gain applied before the equalizer bands.
    pub preamp_db: f32,
}
/// Playback output metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackMetricsResponse {
    /// Number of output buffer underruns.
    pub underrun_count: u64,
    /// Maximum output buffer size in frames.
    pub buffer_capacity_frames: u64,
    /// Frames currently buffered.
    pub buffered_frames: u64,
    /// Approximate buffered audio duration in milliseconds.
    pub buffered_duration_ms: u64,
    /// Output sample rate in hertz.
    pub output_sample_rate: u32,
    /// Number of output channels.
    pub channel_count: u16,
    /// Active sink identifier, if available.
    pub active_sink_id: Option<String>,
    /// Fraction of frames dropped by the output path.
    pub drop_rate: f32,
    /// Estimated output latency in milliseconds.
    pub latency_estimate_ms: u64,
}
/// Linear gain and optional mute update.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeRequest {
    /// Linear output gain in the inclusive range from zero to one.
    pub volume: f32,
    #[serde(default)]
    /// Optional mute-state update.
    pub muted: Option<bool>,
}
/// Playback state snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackSnapshot {
    /// Transport lifecycle state.
    pub state: PlaybackStatus,
    /// Recording loaded into the transport, when one is active.
    pub recording_id: Option<RecordingId>,
    /// Absolute position in milliseconds.
    pub position_ms: u64,
    /// Track duration in milliseconds, when known.
    pub duration_ms: Option<u64>,
    /// Linear output gain in the inclusive range from zero to one.
    pub volume: f32,
    /// Whether output is muted.
    pub muted: bool,
    #[serde(default)]
    /// Queue repeat mode.
    pub repeat: crate::RepeatMode,
    #[serde(default)]
    /// Queue shuffle mode.
    pub shuffle: crate::ShuffleMode,
    #[serde(default)]
    /// Whether shuffle traversal is enabled.
    pub shuffle_enabled: bool,
}
/// Compatibility alias for current playback state.
pub type PlaybackStatusResponse = PlaybackSnapshot;
/// Event streamed to playback clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackEvent {
    /// Event type name.
    pub event: String,
    /// Monotonically increasing event sequence.
    pub sequence: u64,
    /// Event creation timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Playback state associated with the event.
    pub state: PlaybackSnapshot,
}
/// A queue occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueEntry {
    /// Unique identifier for this queue occurrence.
    pub id: uuid::Uuid,
    /// Recording to start.
    pub recording_id: RecordingId,
}
/// Ordered queue snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePlaybackQueue {
    /// Queue occurrences in playback order.
    pub entries: Vec<QueueEntry>,
    /// Revision counter incremented after queue mutations.
    pub revision: u64,
    #[serde(default)]
    /// Index of the currently selected occurrence.
    pub current_cursor: Option<usize>,
}

/// Request to move a queue entry to a final position after removal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueMoveRequest {
    /// Identifier for the queue occurrence to move.
    pub entry_id: uuid::Uuid,
    /// Final zero-based position.
    pub position: usize,
}
/// Queue reorder request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueReorderRequest {
    /// Every current occurrence ID exactly once, in the requested order.
    pub entry_ids: Vec<uuid::Uuid>,
}
/// Add a recording occurrence to the queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueAddRequest {
    /// Recording to start.
    pub recording_id: RecordingId,
}
