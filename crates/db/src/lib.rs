//! SQLite persistence for playback queue state.
use rusqlite::{Connection, Transaction, params};
use std::path::Path;
use timbre_api_contracts::{
    QueueEntry, RecordingId, RepeatMode, RuntimePlaybackQueue as PlaybackQueue, ShuffleMode,
};

/// A persisted queue plus its active queue cursor.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedQueue {
    /// Queue occurrences in playback order.
    pub queue: PlaybackQueue,
    /// Zero-based active queue index, if one is selected.
    pub current_cursor: Option<usize>,
    /// Repeat mode persisted across restarts.
    pub repeat: RepeatMode,
    /// Shuffle mode persisted across restarts.
    pub shuffle: ShuffleMode,
    /// Queue index traversal order while shuffle is enabled.
    pub shuffle_order: Vec<usize>,
    /// Previously visited queue indices, oldest first.
    pub history: Vec<usize>,
    /// Persisted linear output gain.
    pub volume: f32,
    /// Persisted mute state.
    pub muted: bool,
}

/// Thread-safe callers should put this database handle behind a mutex.
pub struct QueueDatabase {
    connection: Connection,
}

impl QueueDatabase {
    /// Open or create a database at a filesystem path and apply migrations.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    /// Create an ephemeral in-memory database for tests or non-persistent use.
    pub fn in_memory() -> rusqlite::Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> rusqlite::Result<Self> {
        connection.execute_batch(include_str!("migrations/0001_playback_queue.sql"))?;
        connection.execute_batch("CREATE TABLE IF NOT EXISTS timbre_schema_migrations(version INTEGER PRIMARY KEY NOT NULL);")?;
        let applied: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM timbre_schema_migrations WHERE version=2)",
            [],
            |row| row.get(0),
        )?;
        if !applied {
            connection.execute_batch(include_str!("migrations/0002_playback_modes.sql"))?;
            connection.execute(
                "INSERT INTO timbre_schema_migrations(version) VALUES (2)",
                [],
            )?;
        }
        let applied: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM timbre_schema_migrations WHERE version=3)",
            [],
            |row| row.get(0),
        )?;
        if !applied {
            connection.execute_batch(include_str!("migrations/0003_playback_volume.sql"))?;
            connection.execute(
                "INSERT INTO timbre_schema_migrations(version) VALUES (3)",
                [],
            )?;
        }
        let applied: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM timbre_schema_migrations WHERE version=4)",
            [],
            |row| row.get(0),
        )?;
        if !applied {
            connection.execute_batch(include_str!("migrations/0004_audio_devices.sql"))?;
            connection.execute(
                "INSERT INTO timbre_schema_migrations(version) VALUES (4)",
                [],
            )?;
        }
        let applied: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM timbre_schema_migrations WHERE version=5)",
            [],
            |row| row.get(0),
        )?;
        if !applied {
            connection.execute_batch(include_str!("migrations/0005_playback_equalizer.sql"))?;
            connection.execute(
                "INSERT INTO timbre_schema_migrations(version) VALUES (5)",
                [],
            )?;
        }
        Ok(Self { connection })
    }

    /// Read persisted EQ settings JSON.
    pub fn equalizer_json(&self) -> rusqlite::Result<String> {
        self.connection.query_row(
            "SELECT settings_json FROM playback_equalizer WHERE singleton=1",
            [],
            |r| r.get(0),
        )
    }
    /// Persist EQ settings JSON.
    pub fn set_equalizer_json(&mut self, value: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE playback_equalizer SET settings_json=?1 WHERE singleton=1",
            [value],
        )?;
        Ok(())
    }

    /// Get persisted selected audio output device ID.
    pub fn selected_audio_device(&self) -> rusqlite::Result<Option<String>> {
        self.connection.query_row(
            "SELECT device_id FROM playback_audio_device WHERE singleton=1",
            [],
            |row| row.get(0),
        )
    }
    /// Persist selected output device ID (NULL denotes system/default).
    pub fn set_selected_audio_device(&mut self, device_id: Option<&str>) -> rusqlite::Result<()> {
        self.connection.execute(
            "UPDATE playback_audio_device SET device_id=?1 WHERE singleton=1",
            [device_id],
        )?;
        Ok(())
    }
    /// Load queue items, revision, and cursor in stable ordering.
    pub fn load(&self) -> rusqlite::Result<PersistedQueue> {
        let (revision, cursor, repeat, shuffle, order_json, history_json, volume, muted): (i64, Option<i64>, String, String, String, String, f64, bool) = self.connection.query_row(
            "SELECT revision, current_cursor, repeat_mode, shuffle_mode, shuffle_order, queue_history, volume, muted FROM playback_queue_state WHERE singleton = 1",
            [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
        )?;
        let mut statement = self.connection.prepare(
            "SELECT entry_id, recording_id FROM playback_queue_items ORDER BY ordering_index",
        )?;
        let rows = statement.query_map([], |row| {
            let entry = uuid::Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            let recording = row
                .get::<_, String>(1)?
                .parse::<RecordingId>()
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })?;
            Ok(QueueEntry {
                id: entry,
                recording_id: recording,
            })
        })?;
        let entries = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        let current_cursor = cursor
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| *index < entries.len());
        let repeat = match repeat.as_str() {
            "all" => RepeatMode::All,
            "one" => RepeatMode::One,
            _ => RepeatMode::Off,
        };
        let shuffle = if shuffle == "on" {
            ShuffleMode::On
        } else {
            ShuffleMode::Off
        };
        let shuffle_order = serde_json::from_str::<Vec<usize>>(&order_json)
            .unwrap_or_default()
            .into_iter()
            .filter(|i| *i < entries.len())
            .collect();
        let history = serde_json::from_str::<Vec<usize>>(&history_json)
            .unwrap_or_default()
            .into_iter()
            .filter(|i| *i < entries.len())
            .collect();
        Ok(PersistedQueue {
            queue: PlaybackQueue {
                entries,
                revision: u64::try_from(revision).unwrap_or(0),
                current_cursor,
            },
            current_cursor,
            repeat,
            shuffle,
            shuffle_order,
            history,
            volume: volume as f32,
            muted,
        })
    }

    /// Atomically replace persisted ordered occurrences and cursor.
    pub fn save(
        &mut self,
        queue: &PlaybackQueue,
        current_cursor: Option<usize>,
    ) -> rusqlite::Result<()> {
        let saved = self.load()?;
        self.save_playback(&PersistedQueue {
            queue: queue.clone(),
            current_cursor,
            ..saved
        })
    }

    /// Atomically persist queue, playback modes, shuffled traversal, and history.
    pub fn save_modes(
        &mut self,
        queue: &PlaybackQueue,
        current_cursor: Option<usize>,
        repeat: RepeatMode,
        shuffle: ShuffleMode,
        order: &[usize],
        history: &[usize],
    ) -> rusqlite::Result<()> {
        let (volume, muted): (f64, bool) = self.connection.query_row(
            "SELECT volume, muted FROM playback_queue_state WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        self.save_playback(&PersistedQueue {
            queue: queue.clone(),
            current_cursor,
            repeat,
            shuffle,
            shuffle_order: order.to_vec(),
            history: history.to_vec(),
            volume: volume as f32,
            muted,
        })
    }

    /// Persist queue, traversal, modes, gain, and mute atomically.
    pub fn save_playback(&mut self, playback: &PersistedQueue) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        save_in_transaction(&tx, &playback.queue, playback.current_cursor)?;
        tx.execute("UPDATE playback_queue_state SET repeat_mode=?1, shuffle_mode=?2, shuffle_order=?3, queue_history=?4, volume=?5, muted=?6 WHERE singleton=1", params![match playback.repeat { RepeatMode::Off => "off", RepeatMode::All => "all", RepeatMode::One => "one" }, match playback.shuffle { ShuffleMode::Off => "off", ShuffleMode::On => "on" }, serde_json::to_string(&playback.shuffle_order).unwrap_or_else(|_| "[]".into()), serde_json::to_string(&playback.history).unwrap_or_else(|_| "[]".into()), playback.volume.clamp(0.0, 1.0), playback.muted])?;
        tx.commit()
    }
}

fn save_in_transaction(
    tx: &Transaction<'_>,
    queue: &PlaybackQueue,
    current_cursor: Option<usize>,
) -> rusqlite::Result<()> {
    tx.execute("DELETE FROM playback_queue_items", [])?;
    {
        let mut statement = tx.prepare("INSERT INTO playback_queue_items(entry_id, recording_id, ordering_index) VALUES (?1, ?2, ?3)")?;
        for (index, item) in queue.entries.iter().enumerate() {
            statement.execute(params![
                item.id.to_string(),
                item.recording_id.to_string(),
                i64::try_from(index).unwrap_or(i64::MAX)
            ])?;
        }
    }
    let cursor = current_cursor
        .filter(|index| *index < queue.entries.len())
        .and_then(|index| i64::try_from(index).ok());
    tx.execute(
        "UPDATE playback_queue_state SET revision = ?1, current_cursor = ?2 WHERE singleton = 1",
        params![i64::try_from(queue.revision).unwrap_or(i64::MAX), cursor],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_round_trips_with_cursor_and_revision() {
        let mut db = QueueDatabase::in_memory().unwrap();
        let id = RecordingId::generate();
        let queue = PlaybackQueue {
            entries: vec![QueueEntry {
                id: uuid::Uuid::new_v4(),
                recording_id: id,
            }],
            revision: 9,
            current_cursor: Some(0),
        };
        db.save(&queue, Some(0)).unwrap();
        assert_eq!(
            db.load().unwrap(),
            PersistedQueue {
                queue: queue.clone(),
                current_cursor: Some(0),
                repeat: RepeatMode::Off,
                shuffle: ShuffleMode::Off,
                shuffle_order: vec![],
                history: vec![],
                volume: 1.0,
                muted: false
            }
        );
        db.save_playback(&PersistedQueue {
            queue: queue.clone(),
            current_cursor: Some(0),
            repeat: RepeatMode::All,
            shuffle: ShuffleMode::On,
            shuffle_order: vec![0],
            history: vec![0],
            volume: 0.42,
            muted: true,
        })
        .unwrap();
        let loaded = db.load().unwrap();
        assert_eq!(loaded.repeat, RepeatMode::All);
        assert_eq!(loaded.shuffle, ShuffleMode::On);
        assert_eq!(loaded.shuffle_order, vec![0]);
        assert_eq!(loaded.history, vec![0]);
        assert_eq!(loaded.volume, 0.42);
        assert!(loaded.muted);
    }
}
