//! Application state, catalog adapter, and persistent playback queue.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use timbre_api_contracts::{
    PlaybackEvent, PlaybackSnapshot, PlaybackStatus, QueueEntry, RepeatMode,
    RuntimePlaybackQueue as PlaybackQueue, ShuffleMode,
};
use timbre_audio_engine::{
    AudioSink, MemoryAudioSink, PlaybackEngine, PlaybackState as EngineState,
};
use tokio::sync::broadcast;

use timbre_api_contracts::{Album, Artist, Recording};
use timbre_shared_types::{AlbumId, ArtistId, JobId, JobProgress, JobState, JobStatus};

use crate::api::library::{LibraryRoot, TrackView};

/// Shared server state. Cloning this value shares the same catalog.
#[derive(Clone)]
pub struct AppState {
    pub(crate) library: Arc<RwLock<LibraryStore>>,
    pub(crate) playback: Arc<Mutex<PlaybackStore>>,
    pub(crate) playback_events: broadcast::Sender<PlaybackEvent>,
    pub(crate) hardware_audio: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let queue_db = Arc::new(Mutex::new(
            timbre_db::QueueDatabase::in_memory().expect("in-memory queue database"),
        ));
        let mut state = Self::with_queue_database(queue_db);
        state.hardware_audio = false;
        state
    }
}

impl AppState {
    /// Creates empty in-memory application state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open application state backed by a persistent SQLite database; persisted queue state is loaded immediately.
    pub fn open(database_path: impl AsRef<std::path::Path>) -> rusqlite::Result<Self> {
        let db = timbre_db::QueueDatabase::open(database_path)?;
        Ok(Self::with_queue_database(Arc::new(Mutex::new(db))))
    }

    /// Open with explicit output selection. Hardware selection falls back to memory when CPAL is disabled or no device is available.
    pub fn open_with_audio_backend(
        database_path: impl AsRef<std::path::Path>,
        hardware: bool,
    ) -> rusqlite::Result<Self> {
        let db = timbre_db::QueueDatabase::open(database_path)?;
        let selected = db.selected_audio_device()?;
        let sink: Arc<dyn AudioSink> = if hardware {
            #[cfg(feature = "hardware-audio")]
            {
                timbre_audio_engine::CpalAudioSink::open_device(selected.as_deref())
                    .map(|sink| Arc::new(sink) as Arc<dyn AudioSink>)
                    .unwrap_or_else(|_| Arc::new(MemoryAudioSink::new()))
            }
            #[cfg(not(feature = "hardware-audio"))]
            {
                Arc::new(MemoryAudioSink::new())
            }
        } else {
            Arc::new(MemoryAudioSink::new())
        };
        let db_handle = Arc::new(Mutex::new(db));
        let mut state = Self::with_queue_database_and_sink(db_handle, sink);
        state.hardware_audio = hardware;
        if let Ok(mut playback) = state.playback.lock() {
            playback.selected_device_id = selected;
        }
        Ok(state)
    }

    fn with_queue_database(db_handle: Arc<Mutex<timbre_db::QueueDatabase>>) -> Self {
        Self::with_queue_database_and_sink(db_handle, Arc::new(MemoryAudioSink::new()))
    }

    fn with_queue_database_and_sink(
        db_handle: Arc<Mutex<timbre_db::QueueDatabase>>,
        sink: Arc<dyn AudioSink>,
    ) -> Self {
        let (playback_events, _) = broadcast::channel(128);
        let selected_device_id = db_handle
            .lock()
            .ok()
            .and_then(|db| db.selected_audio_device().ok().flatten());
        let persisted = db_handle.lock().ok().and_then(|db| db.load().ok());
        let (
            queue,
            queue_revision,
            current_cursor,
            repeat,
            shuffle,
            shuffle_order,
            history,
            volume,
            muted,
        ) = persisted.map_or(
            (
                Vec::new(),
                0,
                None,
                RepeatMode::Off,
                ShuffleMode::Off,
                Vec::new(),
                Vec::new(),
                1.0,
                false,
            ),
            |saved| {
                (
                    saved.queue.entries,
                    saved.queue.revision,
                    saved.current_cursor,
                    saved.repeat,
                    saved.shuffle,
                    saved.shuffle_order,
                    saved.history,
                    saved.volume,
                    saved.muted,
                )
            },
        );
        let mut engine = PlaybackEngine::new(sink);
        engine.set_volume(volume);
        engine.set_muted(muted);
        Self {
            library: Arc::new(RwLock::new(LibraryStore::default())),
            playback: Arc::new(Mutex::new(PlaybackStore {
                engine,
                recording_id: None,
                sequence: 0,
                queue,
                queue_revision,
                current_cursor,
                repeat,
                shuffle,
                shuffle_order,
                history,
                db_handle,
                selected_device_id,
            })),
            playback_events,
            hardware_audio: false,
        }
    }
}

#[derive(Default)]
pub(crate) struct LibraryStore {
    pub roots: BTreeMap<String, LibraryRoot>,
    pub artists: BTreeMap<ArtistId, Artist>,
    pub albums: BTreeMap<AlbumId, Album>,
    pub tracks: BTreeMap<String, TrackView>,
    pub jobs: BTreeMap<JobId, JobStatus>,
    artist_keys: BTreeMap<String, ArtistId>,
    album_keys: BTreeMap<String, AlbumId>,
}

impl LibraryStore {
    pub(crate) fn upsert_artist(&mut self, name: &str) -> ArtistId {
        let key = name.trim().to_lowercase();
        if let Some(id) = self.artist_keys.get(&key) {
            return *id;
        }
        let id = ArtistId::generate();
        self.artist_keys.insert(key, id);
        self.artists.insert(
            id,
            Artist {
                id,
                name: name.to_owned(),
            },
        );
        id
    }

    pub(crate) fn upsert_album(&mut self, title: &str, artist: Option<ArtistId>) -> AlbumId {
        let key = format!(
            "{}\0{}",
            artist.map_or_else(String::new, |id| id.to_string()),
            title.trim().to_lowercase()
        );
        if let Some(id) = self.album_keys.get(&key) {
            return *id;
        }
        let id = AlbumId::generate();
        self.album_keys.insert(key, id);
        self.albums.insert(
            id,
            Album {
                id,
                title: title.to_owned(),
                artist,
                year: None,
            },
        );
        id
    }

    pub(crate) fn upsert_track(
        &mut self,
        path: PathBuf,
        title: String,
        album: Option<AlbumId>,
        track_number: Option<u16>,
        duration_ms: Option<u64>,
        artist: ArtistId,
    ) {
        let key = path.to_string_lossy().into_owned();
        let recording_id = self
            .tracks
            .get(&key)
            .map(|existing| existing.recording.id)
            .unwrap_or_else(timbre_shared_types::RecordingId::generate);
        let recording = Recording {
            id: recording_id,
            title,
            album,
            track_number,
            duration_ms,
        };
        let view = TrackView { recording, artist };
        self.tracks.insert(key, view);
    }
}

/// A deterministic snapshot for a library browse query.
pub(crate) fn snapshot(state: &AppState) -> Result<LibrarySnapshot, ()> {
    let store = state.library.read().map_err(|_| ())?;
    Ok(LibrarySnapshot {
        roots: store.roots.values().cloned().collect(),
        artists: store.artists.values().cloned().collect(),
        albums: store.albums.values().cloned().collect(),
        tracks: store.tracks.values().cloned().collect(),
    })
}

#[derive(Clone)]
pub(crate) struct LibrarySnapshot {
    pub roots: Vec<LibraryRoot>,
    pub artists: Vec<Artist>,
    pub albums: Vec<Album>,
    pub tracks: Vec<TrackView>,
}

/// A queued background scan job status.
pub(crate) fn queued_scan() -> JobStatus {
    JobStatus {
        id: JobId::generate(),
        kind: "library_scan".to_owned(),
        state: JobState::Queued,
        progress: JobProgress {
            completed: 0,
            total: None,
        },
        message: None,
        error: None,
    }
}

/// Thread-safe controls and metadata associated with the audio engine.
pub(crate) struct PlaybackStore {
    pub engine: PlaybackEngine,
    pub recording_id: Option<timbre_shared_types::RecordingId>,
    pub sequence: u64,
    pub queue: Vec<QueueEntry>,
    pub queue_revision: u64,
    pub current_cursor: Option<usize>,
    pub repeat: RepeatMode,
    pub shuffle: ShuffleMode,
    pub shuffle_order: Vec<usize>,
    pub history: Vec<usize>,
    pub db_handle: Arc<Mutex<timbre_db::QueueDatabase>>,
    pub selected_device_id: Option<String>,
}

impl PlaybackStore {
    pub(crate) fn queue_snapshot(&self) -> PlaybackQueue {
        PlaybackQueue {
            entries: self.queue.clone(),
            revision: self.queue_revision,
            current_cursor: self.current_cursor,
        }
    }

    pub(crate) fn append_queue(
        &mut self,
        recording_id: timbre_shared_types::RecordingId,
    ) -> PlaybackQueue {
        self.queue.push(QueueEntry {
            id: uuid::Uuid::new_v4(),
            recording_id,
        });
        self.shuffle_order.clear();
        self.queue_revision = self.queue_revision.saturating_add(1);
        self.persist_queue();
        self.queue_snapshot()
    }

    pub(crate) fn remove_queue_entry(&mut self, entry_id: uuid::Uuid) -> bool {
        if let Some(index) = self.queue.iter().position(|entry| entry.id == entry_id) {
            self.queue.remove(index);
            self.shuffle_order.clear();
            self.history.clear();
            if let Some(cursor) = self.current_cursor {
                if index < cursor {
                    self.current_cursor = Some(cursor - 1);
                } else if index == cursor {
                    self.current_cursor = None;
                }
            }
            self.queue_revision = self.queue_revision.saturating_add(1);
            self.persist_queue();
            true
        } else {
            false
        }
    }

    pub(crate) fn clear_queue(&mut self) -> PlaybackQueue {
        if !self.queue.is_empty() {
            self.queue.clear();
            self.shuffle_order.clear();
            self.history.clear();
            self.current_cursor = None;
            self.queue_revision = self.queue_revision.saturating_add(1);
            self.persist_queue();
        }
        self.queue_snapshot()
    }

    pub(crate) fn persist_queue(&self) {
        // The database is authoritative on each mutation; lock poisoning is handled by startup/API.
        if let Ok(mut db) = self.db_handle.lock() {
            let _ = db.save_playback(&timbre_db::PersistedQueue {
                queue: self.queue_snapshot(),
                current_cursor: self.current_cursor,
                repeat: self.repeat,
                shuffle: self.shuffle,
                shuffle_order: self.shuffle_order.clone(),
                history: self.history.clone(),
                volume: self.engine.volume().level(),
                muted: self.engine.volume().is_muted(),
            });
        }
    }

    pub(crate) fn reorder_queue(&mut self, ids: &[uuid::Uuid]) -> bool {
        if ids.len() != self.queue.len() {
            return false;
        }
        let mut reordered = Vec::with_capacity(ids.len());
        for id in ids {
            let Some(entry) = self.queue.iter().find(|entry| entry.id == *id) else {
                return false;
            };
            if reordered
                .iter()
                .any(|prior: &QueueEntry| prior.id == entry.id)
            {
                return false;
            }
            reordered.push(entry.clone());
        }
        let active_id = self
            .current_cursor
            .and_then(|i| self.queue.get(i))
            .map(|entry| entry.id);
        self.queue = reordered;
        self.shuffle_order.clear();
        self.history.clear();
        self.current_cursor =
            active_id.and_then(|id| self.queue.iter().position(|entry| entry.id == id));
        self.queue_revision = self.queue_revision.saturating_add(1);
        self.persist_queue();
        true
    }

    /// Move one occurrence to its final position after removal; cursor follows active occurrence.
    pub(crate) fn move_queue_entry(&mut self, entry_id: uuid::Uuid, position: usize) -> bool {
        if position >= self.queue.len() {
            return false;
        }
        let Some(from) = self.queue.iter().position(|entry| entry.id == entry_id) else {
            return false;
        };
        if from == position {
            return true;
        }
        let active_id = self
            .current_cursor
            .and_then(|i| self.queue.get(i))
            .map(|entry| entry.id);
        let entry = self.queue.remove(from);
        self.queue.insert(position, entry);
        self.shuffle_order.clear();
        self.history.clear();
        self.current_cursor =
            active_id.and_then(|id| self.queue.iter().position(|entry| entry.id == id));
        self.queue_revision = self.queue_revision.saturating_add(1);
        self.persist_queue();
        true
    }

    pub(crate) fn set_queue_position(&mut self, index: usize) {
        self.current_cursor = Some(index);
        self.queue_revision = self.queue_revision.saturating_add(1);
        self.persist_queue();
    }

    pub(crate) fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
        self.persist_queue();
    }
    pub(crate) fn set_shuffle(&mut self, mode: ShuffleMode) {
        if mode != self.shuffle {
            self.shuffle_order.clear();
            self.history.clear();
        }
        self.shuffle = mode;
        if mode == ShuffleMode::On {
            self.ensure_shuffle_order();
        }
        self.persist_queue();
    }
    pub(crate) fn ensure_shuffle_order(&mut self) {
        if self.shuffle_order.len() == self.queue.len()
            && self.shuffle_order.iter().all(|i| *i < self.queue.len())
        {
            return;
        }
        self.shuffle_order = (0..self.queue.len()).collect();
        let mut seed = self
            .queue_revision
            .wrapping_add(self.queue.len() as u64)
            .max(1);
        for i in (1..self.shuffle_order.len()).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            self.shuffle_order.swap(i, (seed as usize) % (i + 1));
        }
    }
    pub(crate) fn next_queue_index(&mut self, forward: bool) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }
        let current = self.current_cursor?;
        if !forward {
            if let Some(previous) = self.history.pop() {
                self.persist_queue();
                return Some(previous);
            }
            if self.shuffle == ShuffleMode::On {
                self.ensure_shuffle_order();
                let pos = self
                    .shuffle_order
                    .iter()
                    .position(|index| *index == current)?;
                return pos
                    .checked_sub(1)
                    .and_then(|i| self.shuffle_order.get(i).copied())
                    .or_else(|| {
                        (self.repeat == RepeatMode::All)
                            .then(|| self.shuffle_order.last().copied())
                            .flatten()
                    });
            }
            return current
                .checked_sub(1)
                .or_else(|| (self.repeat == RepeatMode::All).then_some(self.queue.len() - 1));
        }
        if self.shuffle == ShuffleMode::On {
            self.ensure_shuffle_order();
            let order_pos = self
                .shuffle_order
                .iter()
                .position(|index| *index == current)?;
            return self.shuffle_order.get(order_pos + 1).copied().or_else(|| {
                (self.repeat == RepeatMode::All)
                    .then(|| self.shuffle_order.first().copied())
                    .flatten()
            });
        }
        if current + 1 < self.queue.len() {
            Some(current + 1)
        } else if self.repeat == RepeatMode::All {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn snapshot(&self) -> PlaybackSnapshot {
        let state = match self.engine.state() {
            EngineState::Playing => PlaybackStatus::Playing,
            EngineState::Paused => PlaybackStatus::Paused,
            EngineState::TrackEnded => PlaybackStatus::TrackEnded,
            EngineState::Stopped => PlaybackStatus::Stopped,
        };
        PlaybackSnapshot {
            state,
            recording_id: self.recording_id,
            position_ms: (self.engine.position_seconds().max(0.0) * 1000.0) as u64,
            duration_ms: if self.recording_id.is_some() {
                self.engine
                    .duration_seconds()
                    .map(|s| (s.max(0.0) * 1000.0) as u64)
            } else {
                None
            },
            volume: self.engine.volume().level(),
            muted: self.engine.volume().is_muted(),
            repeat: self.repeat,
            shuffle: self.shuffle,
            shuffle_enabled: self.shuffle == ShuffleMode::On,
        }
    }
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn volume_and_mute_are_restored_from_persistence() {
        let mut db = timbre_db::QueueDatabase::in_memory().unwrap();
        let queue = PlaybackQueue {
            entries: Vec::new(),
            revision: 0,
            current_cursor: None,
        };
        db.save_playback(&timbre_db::PersistedQueue {
            queue: queue.clone(),
            current_cursor: None,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
            shuffle_order: vec![],
            history: vec![],
            volume: 0.3,
            muted: true,
        })
        .unwrap();
        let handle = Arc::new(Mutex::new(db));
        let state = AppState::with_queue_database(handle);
        let playback = state.playback.lock().unwrap();
        assert_eq!(playback.engine.volume().level(), 0.3);
        assert!(playback.engine.volume().is_muted());
    }

    #[test]
    fn playback_modes_control_repeat_shuffle_and_persist() {
        let mut store = PlaybackStore {
            engine: PlaybackEngine::new(Arc::new(MemoryAudioSink::new())),
            recording_id: None,
            sequence: 0,
            queue: Vec::new(),
            queue_revision: 0,
            current_cursor: None,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
            shuffle_order: Vec::new(),
            history: Vec::new(),
            db_handle: Arc::new(Mutex::new(timbre_db::QueueDatabase::in_memory().unwrap())),
            selected_device_id: None,
        };
        let a = timbre_shared_types::RecordingId::generate();
        let b = timbre_shared_types::RecordingId::generate();
        store.append_queue(a);
        store.append_queue(b);
        store.set_queue_position(1);
        assert_eq!(store.next_queue_index(true), None);
        store.set_repeat(RepeatMode::All);
        assert_eq!(store.next_queue_index(true), Some(0));
        store.set_repeat(RepeatMode::Off);
        store.set_shuffle(ShuffleMode::On);
        assert_eq!(store.shuffle_order.len(), 2);
        assert!(store.snapshot().shuffle_enabled);
        let loaded = store.db_handle.lock().unwrap().load().unwrap();
        assert_eq!(loaded.shuffle, ShuffleMode::On);
        assert_eq!(loaded.shuffle_order.len(), 2);
    }

    #[test]
    fn moving_queue_occurrence_preserves_active_identity_and_persists() {
        let mut store = PlaybackStore {
            engine: PlaybackEngine::new(Arc::new(MemoryAudioSink::new())),
            recording_id: None,
            sequence: 0,
            queue: Vec::new(),
            queue_revision: 0,
            current_cursor: None,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
            shuffle_order: Vec::new(),
            history: Vec::new(),
            db_handle: Arc::new(Mutex::new(timbre_db::QueueDatabase::in_memory().unwrap())),
            selected_device_id: None,
        };
        let a = store.append_queue(timbre_shared_types::RecordingId::generate());
        let b = store.append_queue(timbre_shared_types::RecordingId::generate());
        let c = store.append_queue(timbre_shared_types::RecordingId::generate());
        store.set_queue_position(1);
        assert!(store.move_queue_entry(a.entries[0].id, 2));
        assert_eq!(store.current_cursor, Some(0));
        assert_eq!(store.queue[0].id, b.entries[1].id);
        assert_eq!(store.queue[2].id, a.entries[0].id);
        assert_eq!(store.queue_revision, 5);
        assert!(!store.move_queue_entry(c.entries[2].id, 3));
        assert_eq!(
            store
                .db_handle
                .lock()
                .unwrap()
                .load()
                .unwrap()
                .queue
                .entries,
            store.queue
        );
    }

    #[test]
    fn ordered_queue_preserves_duplicate_occurrences_and_revision() {
        let mut store = PlaybackStore {
            engine: PlaybackEngine::new(Arc::new(MemoryAudioSink::new())),
            recording_id: None,
            sequence: 0,
            queue: Vec::new(),
            queue_revision: 0,
            current_cursor: None,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
            shuffle_order: Vec::new(),
            history: Vec::new(),
            db_handle: Arc::new(Mutex::new(timbre_db::QueueDatabase::in_memory().unwrap())),
            selected_device_id: None,
        };
        let recording = timbre_shared_types::RecordingId::generate();
        let first = store.append_queue(recording);
        let second = store.append_queue(recording);
        assert_eq!(second.revision, 2);
        assert_eq!(second.entries.len(), 2);
        assert_ne!(second.entries[0].id, second.entries[1].id);
        assert!(store.remove_queue_entry(first.entries[0].id));
        assert_eq!(
            store.queue_snapshot().entries,
            vec![second.entries[1].clone()]
        );
        assert_eq!(store.queue_snapshot().revision, 3);
        assert!(!store.remove_queue_entry(first.entries[0].id));
        assert_eq!(store.clear_queue().revision, 4);
        assert_eq!(store.clear_queue().revision, 4);
    }
}
