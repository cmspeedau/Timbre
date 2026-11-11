#!/usr/bin/env python3
"""Offline Milestone 4 verification for SQLite upgrades, fixtures, and route/pipeline structure."""
import json
import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
conn = sqlite3.connect(':memory:')
conn.executescript((ROOT / 'crates/db/src/migrations/0001_playback_queue.sql').read_text())
conn.executescript((ROOT / 'crates/db/src/migrations/0002_playback_modes.sql').read_text())
conn.executescript((ROOT / 'crates/db/src/migrations/0003_playback_volume.sql').read_text())
cols = {r[1] for r in conn.execute('PRAGMA table_info(playback_queue_state)')}
assert {'repeat_mode','shuffle_mode','shuffle_order','queue_history','volume','muted'} <= cols
initial = conn.execute('SELECT volume, muted FROM playback_queue_state WHERE singleton=1').fetchone()
assert initial == (1.0, 0)
conn.execute("UPDATE playback_queue_state SET repeat_mode='all', shuffle_mode='on', volume=0.375, muted=1 WHERE singleton=1")
restored = conn.execute('SELECT repeat_mode, shuffle_mode, volume, muted FROM playback_queue_state WHERE singleton=1').fetchone()
assert restored == ('all', 'on', 0.375, 1)
fixtures = ROOT / 'crates/testing/fixtures'
for fixture in fixtures.glob('*.json'):
    json.loads(fixture.read_text())
vol = json.loads((fixtures / 'playback-volume.json').read_text())
assert 0 <= vol['volume'] <= 1 and isinstance(vol['muted'], bool)
move = json.loads((fixtures / 'playback-queue-move.json').read_text())
assert move['position'] == 2 and len(move['entry_id']) == 36
contracts = (ROOT / 'crates/api-contracts/src/lib.rs').read_text()
api = (ROOT / 'crates/server/src/api/playback.rs').read_text()
state = (ROOT / 'crates/server/src/state.rs').read_text()
db = (ROOT / 'crates/db/src/lib.rs').read_text()
audio = (ROOT / 'crates/audio-engine/src/lib.rs').read_text()
assert 'QueueMoveRequest' in contracts and 'pub position: usize' in contracts
assert '/playback/queue/move' in api and 'move_queue_entry' in api
assert 'move_queue_entry' in state and 'save_playback' in state
assert 'volume, muted' in db and '0003_playback_volume.sql' in db
assert 'self.volume.muted' in audio and 'self.volume.level' in audio
assert 'playback_engine_applies_gain_and_mute_to_decoded_pcm' in (ROOT / 'crates/audio-engine/src/decoder_tests.rs').read_text()
print('PASS: SQLite 0001→0003 defaults/round-trip, all JSON fixtures, volume/mute pipeline markers, queue-move contract/route/persistence')
