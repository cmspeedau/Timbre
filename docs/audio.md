# Audio processing

Playback applies the stateful parametric peaking biquad EQ and preamp to decoded PCM before linear volume/mute gain and sink submission. EQ supports up to 64 bands, frequency below Nyquist, Q > 0, gain/preamp within ±24 dB. Settings persist in SQLite migration 0005. Metrics currently report decoded format and selected sink; sinks do not yet expose queue depth or underrun counters, so unavailable buffering/drop estimates are zero.
