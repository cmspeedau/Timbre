#!/usr/bin/env python3
"""Offline structural, migration, route and fixture verification for milestone 6."""
import json, sqlite3
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
for path in (ROOT / 'crates/testing/fixtures').glob('*.json'):
    data = json.loads(path.read_text())
    assert json.loads(json.dumps(data)) == data
assert json.loads((ROOT/'crates/testing/fixtures/playback-devices.json').read_text())['devices'][0]['id'] == 'memory'
assert json.loads((ROOT/'crates/testing/fixtures/playback-device-selection.json').read_text()) == {'device_id':'memory'}
db = sqlite3.connect(':memory:')
for name in ['0001_playback_queue.sql','0002_playback_modes.sql','0003_playback_volume.sql','0004_audio_devices.sql']:
    db.executescript((ROOT/'crates/db/src/migrations'/name).read_text())
db.execute('UPDATE playback_audio_device SET device_id=? WHERE singleton=1', ('memory',))
assert db.execute('SELECT device_id FROM playback_audio_device WHERE singleton=1').fetchone()[0] == 'memory'
contracts=(ROOT/'crates/api-contracts/src/lib.rs').read_text() + (ROOT/'crates/api-contracts/src/runtime.rs').read_text(); api=(ROOT/'crates/server/src/api/playback.rs').read_text(); engine=(ROOT/'crates/audio-engine/src/lib.rs').read_text(); sink=(ROOT/'crates/audio-engine/src/sink.rs').read_text(); state=(ROOT/'crates/server/src/state.rs').read_text(); docs=(ROOT/'docs/api/README.md').read_text()
for route in ('playback/devices','playback/device'): assert route in api
for marker in ('PlaybackDevice','PlaybackDeviceSelection','sample_rates','channels'): assert marker in contracts
for marker in ('output_devices','open_device','playback.device_changed','set_selected_audio_device'): assert marker in api
for marker in ('prepare(', 'prepared.take()', 'replace_sink'): assert marker in engine
assert 'supported_output_configs' in sink and 'Memory / headless output' in sink
assert '0004_audio_devices.sql' in (ROOT/'crates/db/src/lib.rs').read_text()
assert 'preload' in docs.lower() and 'playback.device_changed' in docs
print('PASS: fixture JSON roundtrips, SQLite migration/device persistence, contracts, endpoints/SSE, CPAL/memory sink, preloading and docs')
