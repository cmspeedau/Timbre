CREATE TABLE IF NOT EXISTS playback_equalizer (
 singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
 settings_json TEXT NOT NULL DEFAULT '{"bands":[],"preamp_db":0.0}'
);
INSERT OR IGNORE INTO playback_equalizer(singleton, settings_json) VALUES (1, '{"bands":[],"preamp_db":0.0}');
