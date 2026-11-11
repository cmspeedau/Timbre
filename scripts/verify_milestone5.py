#!/usr/bin/env python3
"""Offline checks for Milestone 5 contracts, routes, fixtures, and audio adapters."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
fixtures = ROOT / 'crates/testing/fixtures'
for path in fixtures.glob('*.json'):
    json.loads(path.read_text())
seek = json.loads((fixtures / 'playback-seek.json').read_text())
assert seek == {'position_ms': 1234}
contracts = (ROOT / 'crates/api-contracts/src/lib.rs').read_text() + (ROOT / 'crates/api-contracts/src/runtime.rs').read_text()
api = (ROOT / 'crates/server/src/api/playback.rs').read_text()
state = (ROOT / 'crates/server/src/state.rs').read_text()
decoder = (ROOT / 'crates/audio-engine/src/decoder.rs').read_text()
engine = (ROOT / 'crates/audio-engine/src/lib.rs').read_text()
sink = (ROOT / 'crates/audio-engine/src/sink.rs').read_text()
docs = (ROOT / 'docs/api/README.md').read_text()
assert 'pub struct SeekRequest' in contracts and 'pub position_ms: u64' in contracts
assert '"../../testing/fixtures/playback-seek.json"' in contracts
assert 'playback/seek' in api and 'playback.seeked' in api
assert 'target_seconds > duration' in api
assert 'duration_ms: if self.recording_id.is_some()' in state
assert 'pub fn seek_seconds' in decoder and 'SeekTo::Time' in decoder
assert 'duration_seconds\n            .is_some_and' in engine
assert 'pub fn adapt_pcm' in sink and 'target.sample_rate' in sink
assert 'output_format' in sink and 'adapt_pcm(samples, source, self.output_format)' in sink
assert 'resamples_duration_and_interpolates' in sink
assert 'remixes_mono_stereo_and_multichannel' in sink
assert 'playback.seeked' in docs and 'playback/seek` (also `/api/playback/seek`)' in docs
print('PASS: JSON fixtures, seek contract and route/event/bounds, snapshot duration, decoder seek metadata, PCM rate/channel adapter, tests/docs markers')
