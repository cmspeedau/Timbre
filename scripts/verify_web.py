#!/usr/bin/env python3
"""Offline structural checks for the Phase 2 web frontend integration."""
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
web=ROOT/'crates/server/web'
html=(web/'index.html').read_text()
css=(web/'timbre.css').read_text()
js=(web/'timbre.js').read_text()
api=(ROOT/'crates/server/src/api/mod.rs').read_text()
playback=(ROOT/'crates/server/src/api/playback.rs').read_text()
for feature in ('playPause','previous','next','stop','seek','volume','mute','queueList','repeat','shuffle','deviceSelect','eqBands','eqEnabled','eqReset','eqSave','metricBuffer','metricLatency','metricUnderruns'):
    assert f'id="{feature}"' in html, f'missing UI element: {feature}'
for path in ('/playback/play','/playback/pause','/playback/stop','/playback/next','/playback/previous','/playback/seek','/playback/volume','/playback/queue/move','/playback/repeat','/playback/shuffle','/playback/devices','/playback/device','/playback/equalizer','/playback/metrics','/playback/events'):
    assert path in js, f'missing API integration: {path}'
for event in ('playback_state','playback.state_changed','playback.seeked','playback.device_changed','playback.equalizer_changed','playback.mode_changed','queue.updated'):
    assert event in js, f'missing SSE subscription: {event}'
assert '@media(max-width:760px)' in css and '@media(max-width:430px)' in css
assert 'include_str!("../../web/index.html")' in api and 'include_str!("../../web/timbre.js")' in api
assert playback.count('"queue.updated"') >= 5, 'queue mutations should publish updates'
subprocess.run(['node','--check',str(web/'timbre.js')],check=True)
print('Web UI verification: PASS (controls, endpoints, SSE, responsive CSS, static routes, JS syntax)')
