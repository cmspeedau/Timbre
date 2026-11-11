//! Versioned API request and response models shared by Timbre clients and server.
use serde::{Deserialize, Serialize};
pub use timbre_shared_types::{AlbumId, ArtistId, RecordingId};
/// Current version prefix.
pub const API_VERSION: &str = "v1";
/// API error envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct ApiError { pub code: timbre_shared_types::ErrorCode, pub message: String, pub context: Option<serde_json::Value> }
/// Artist catalog entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct Artist { pub id: ArtistId, pub name: String }
/// Album catalog entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct Album { pub id: AlbumId, pub title: String, pub artist: Option<ArtistId>, pub year: Option<u16> }
/// Recording catalog entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct Recording { pub id: RecordingId, pub title: String, pub album: Option<AlbumId>, pub track_number: Option<u16>, pub duration_ms: Option<u64> }
/// Transport lifecycle in the playback API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] #[serde(rename_all="snake_case")] pub enum PlaybackStatus { Stopped, Playing, Paused, TrackEnded }
/// Request to load and start the specified catalog recording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct PlayRequest { pub recording_id: RecordingId }
/// Seek to an absolute track position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct SeekRequest { pub position_ms: u64 }
/// Available output device and advertised capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDevice { pub id: String, pub name: String, pub is_default: bool, #[serde(default)] pub sample_rates: Vec<u32>, #[serde(default)] pub channels: Vec<u16> }
/// List of output devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDevices { pub devices: Vec<PlaybackDevice> }
/// Current selected output device. `device_id: null` means system default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackDeviceSelection { pub device_id: Option<String> }
/// Request a selected output device by stable API ID (or null for default).
pub type PlaybackDeviceRequest = PlaybackDeviceSelection;
/// Set linear volume and optionally mute output.
/// One parametric EQ band. Frequency is in Hz, gain in dB, Q is dimensionless.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerBand { pub frequency: f32, pub gain_db: f32, pub q: f32, #[serde(default = "enabled_by_default")] pub enabled: bool }
fn enabled_by_default() -> bool { true }
/// Equalizer settings applied before output volume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqualizerSettings { pub bands: Vec<EqualizerBand>, #[serde(default)] pub preamp_db: f32 }
/// Output buffer, format, and health snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaybackMetricsResponse { pub underrun_count: u64, pub buffer_capacity_frames: u64, pub buffered_frames: u64, pub buffered_duration_ms: u64, pub output_sample_rate: u32, pub channel_count: u16, pub active_sink_id: Option<String>, pub drop_rate: f32, pub latency_estimate_ms: u64 }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] pub struct VolumeRequest { pub volume: f32, #[serde(default)] pub muted: Option<bool> }
/// Current transport snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] pub struct PlaybackSnapshot { pub state: PlaybackStatus, pub recording_id: Option<RecordingId>, pub position_ms: u64, pub duration_ms: Option<u64>, pub volume: f32, pub muted: bool, #[serde(default)] pub repeat: RepeatMode, #[serde(default)] pub shuffle: ShuffleMode, #[serde(default)] pub shuffle_enabled: bool }
/// Compatibility name for the current playback status response.
pub type PlaybackStatusResponse = PlaybackSnapshot;
/// Public playback state snapshot alias.
pub type PlaybackState = PlaybackSnapshot;
/// Event model delivered by the playback event stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] pub struct PlaybackEvent { pub event: String, pub sequence: u64, pub occurred_at_ms: u64, pub state: PlaybackSnapshot }
/// One occurrence of a recording in the server's ordered playback queue.
/// The entry ID distinguishes duplicate occurrences of the same recording.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueEntry { pub id: uuid::Uuid, pub recording_id: RecordingId }
/// Snapshot of the ordered playback queue and selected queue cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackQueue { pub entries: Vec<QueueEntry>, pub revision: u64, #[serde(default)] pub current_cursor: Option<usize> }
/// Request to reorder queue occurrences; `entry_ids` must list every current ID once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueReorderRequest { pub entry_ids: Vec<uuid::Uuid> }
/// Request to append one recording occurrence to the queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueAddRequest { pub recording_id: RecordingId }
/// Move one queue occurrence to its final zero-based position (after removal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueMoveRequest { pub entry_id: uuid::Uuid, pub position: usize }

#[cfg(test)]
mod playback_contract_tests { use super::*; #[test] fn smoke() { let _ = API_VERSION; } }
