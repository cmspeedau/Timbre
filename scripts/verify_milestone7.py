#!/usr/bin/env python3
"""Offline structural, migration, route, and serialization checks for milestone 7."""
import json, sqlite3, subprocess, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
for path in (ROOT/'crates/testing/fixtures').glob('*.json'):
    obj=json.loads(path.read_text()); assert json.loads(json.dumps(obj))==obj
fixtures=ROOT/'crates/testing/fixtures'
eq=json.loads((fixtures/'equalizer_settings.json').read_text()); assert all({'frequency','gain_db','q','enabled'}<=b.keys() for b in eq['bands']) and 'preamp_db' in eq
m=json.loads((fixtures/'playback_metrics.json').read_text()); assert all(k in m for k in ('underrun_count','buffer_capacity_frames','buffered_frames','buffered_duration_ms','output_sample_rate','channel_count','active_sink_id','drop_rate','latency_estimate_ms'))
db=sqlite3.connect(':memory:')
for i in range(1,6): db.executescript((ROOT/'crates/db/src/migrations'/f"{i:04}_{ {1:'playback_queue',2:'playback_modes',3:'playback_volume',4:'audio_devices',5:'playback_equalizer'}[i] }.sql").read_text())
assert db.execute('select settings_json from playback_equalizer').fetchone()[0]=='{"bands":[],"preamp_db":0.0}'
db.execute('update playback_equalizer set settings_json=?', (json.dumps(eq),)); assert json.loads(db.execute('select settings_json from playback_equalizer').fetchone()[0])==eq
contracts=(ROOT/'crates/api-contracts/src/lib.rs').read_text(); api=(ROOT/'crates/server/src/api/playback.rs').read_text(); audio=(ROOT/'crates/audio-engine/src/lib.rs').read_text(); dsp=(ROOT/'crates/audio-engine/src/equalizer.rs').read_text(); dbsrc=(ROOT/'crates/db/src/lib.rs').read_text()
for marker in ('EqualizerBand','EqualizerSettings','PlaybackMetricsResponse'): assert marker in contracts
for marker in ('playback/equalizer','playback/metrics','playback.equalizer_changed'): assert marker in api
for marker in ('self.equalizer.process','set_equalizer','volume.level'): assert marker in audio
assert '0005_playback_equalizer.sql' in dbsrc and 'biquad' in dsp.lower()
for n in (3,4,5,6): subprocess.run([sys.executable,str(ROOT/f'scripts/verify_milestone{n}.py')],check=True)
print('PASS: migrations 0001-0005, EQ/metrics JSON fixtures, contracts/routes/DSP, and verifiers 3-6')
