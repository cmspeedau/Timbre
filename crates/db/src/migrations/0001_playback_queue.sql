CREATE TABLE IF NOT EXISTS playback_queue_state (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    revision INTEGER NOT NULL DEFAULT 0,
    current_cursor INTEGER NULL
);
INSERT OR IGNORE INTO playback_queue_state(singleton, revision, current_cursor) VALUES (1, 0, NULL);
CREATE TABLE IF NOT EXISTS playback_queue_items (
    entry_id TEXT PRIMARY KEY NOT NULL,
    recording_id TEXT NOT NULL,
    ordering_index INTEGER NOT NULL UNIQUE CHECK (ordering_index >= 0)
);
