ALTER TABLE playback_queue_state ADD COLUMN volume REAL NOT NULL DEFAULT 1.0 CHECK (volume >= 0.0 AND volume <= 1.0);
ALTER TABLE playback_queue_state ADD COLUMN muted INTEGER NOT NULL DEFAULT 0 CHECK (muted IN (0,1));
