ALTER TABLE playback_queue_state ADD COLUMN repeat_mode TEXT NOT NULL DEFAULT 'off' CHECK (repeat_mode IN ('off','all','one'));
ALTER TABLE playback_queue_state ADD COLUMN shuffle_mode TEXT NOT NULL DEFAULT 'off' CHECK (shuffle_mode IN ('off','on'));
ALTER TABLE playback_queue_state ADD COLUMN shuffle_order TEXT NOT NULL DEFAULT '[]';
ALTER TABLE playback_queue_state ADD COLUMN queue_history TEXT NOT NULL DEFAULT '[]';
