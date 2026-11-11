CREATE TABLE IF NOT EXISTS playback_audio_device (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    device_id TEXT
);
INSERT OR IGNORE INTO playback_audio_device(singleton, device_id) VALUES (1, NULL);
