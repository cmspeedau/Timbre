#!/usr/bin/env python3
"""Offline structural, SQLite migration, and wire-fixture verification."""
import json
import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
conn = sqlite3.connect(':memory:')
conn.executescript((ROOT / 'crates/db/src/migrations/0001_playback_queue.sql').read_text())
# Upgrade from the actual pre-milestone schema.
conn.executescript((ROOT / 'crates/db/src/migrations/0002_playback_modes.sql').read_text())
cols = {row[1] for row in conn.execute('PRAGMA table_info(playback_queue_state)')}
required = {'repeat_mode', 'shuffle_mode', 'shuffle_order', 'queue_history'}
assert required <= cols, required - cols
conn.execute("UPDATE playback_queue_state SET repeat_mode='all', shuffle_mode='on', shuffle_order='[2,0,1]', queue_history='[0,2]' WHERE singleton=1")
values = conn.execute('SELECT repeat_mode, shuffle_mode, shuffle_order, queue_history FROM playback_queue_state WHERE singleton=1').fetchone()
assert values == ('all', 'on', '[2,0,1]', '[0,2]')
for fixture in (ROOT / 'crates/testing/fixtures').glob('*.json'):
    json.loads(fixture.read_text())
mode = json.loads((ROOT / 'crates/testing/fixtures/playback-modes.json').read_text())
snapshot = json.loads((ROOT / 'crates/testing/fixtures/playback-snapshot-modes.json').read_text())
assert snapshot['repeat'] in {'off', 'all', 'one'} and snapshot['shuffle'] in {'off', 'on'}
assert snapshot['shuffle_enabled'] is (snapshot['shuffle'] == 'on')
assert mode == {'repeat': 'all', 'shuffle': 'on', 'shuffle_enabled': True}
contracts = (ROOT / 'crates/api-contracts/src/lib.rs').read_text()
playback_api = (ROOT / 'crates/server/src/api/playback.rs').read_text()
assert all(name in contracts for name in ('RepeatMode', 'ShuffleMode', 'shuffle_enabled'))
assert all(route in playback_api for route in ('/playback/repeat', '/playback/shuffle', 'playback.mode_changed'))
assert all(migration in (ROOT / 'crates/db/src/lib.rs').read_text() for migration in ('0001_playback_queue.sql', '0002_playback_modes.sql'))
print('PASS: SQLite 0001→0002 upgrade, mode/order/history persistence, JSON fixtures, and API structure')
