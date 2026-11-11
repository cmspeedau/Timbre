# Timbre audio engine

This crate provides local file decoding, PCM buffering, and playback controls. It
is designed to be driven by a dedicated playback worker, not by an HTTP handler
or a real-time audio callback.

## Basic integration

```rust,ignore
use std::sync::Arc;
use timbre_audio_engine::{BufferedPcmSink, PlaybackEngine};

let sink = Arc::new(BufferedPcmSink::new(48_000 * 2)); // 1 second of stereo samples
let mut engine = PlaybackEngine::new(sink.clone());
engine.load("music.flac")?;
engine.play();
engine.pump(8)?; // invoke repeatedly on the playback worker
```

A device adapter can consume `BufferedPcmSink::read` on its own output thread.
Writes block when the bounded FIFO is full, providing backpressure instead of
silently dropping samples. `MemoryAudioSink` is provided for deterministic,
headless tests and simple adapters. No CPAL implementation is included; this
keeps device selection and callback lifecycle in the devices/output layer.

## Supported containers/codecs

The default Symphonia registration enables FLAC, MP3, WAV/PCM, AAC in ISO-MP4
(M4A), and OGG/Vorbis. The decoder produces interleaved `i16` PCM chunks. Runtime
support remains dependent on Symphonia codec/container support and media files.

Playback controls expose `Play`, `Pause`, `Stop`, `Seek`, track-ended state, and
linear volume/mute. `pump` applies the current linear gain to decoded PCM before
submitting it to the sink (rounding and saturating to `i16`); mute substitutes
zero samples without changing the remembered gain. A gain/mute change affects
chunks decoded after the change, not samples already queued by an output sink.
`pump` decodes up to a bounded number of chunks and returns at EOF with
`TrackEnded` state. Seeking updates the decoder and observable position but does not itself resume
playback. Decoder frame-count metadata supplies an optional duration and rejects
seeks beyond known track length.


With the optional CPAL feature, `CpalAudioSink` converts interleaved `i16` PCM
to the default output device format before enqueueing it. `adapt_pcm` provides
linear sample-rate conversion and basic mono/stereo/multichannel remixing. The adapter rejects partial interleaved frames. The
callback remains a non-blocking consumer; device-open failures are handled by
the server's memory-sink fallback. This linear adapter is intentionally basic
and is not a substitute for a quality polyphase/sinc resampler.

## Output-device enumeration and decoder preparation (Milestone 6)

With `cpal`, `output_devices()` reports host output devices and supported rate/channel bounds; without CPAL or when no device is enumerable it returns a headless memory device. `CpalAudioSink::open_device` opens a selected `cpal:<name>` ID or the default. The engine supports `prepare(path)` to open a decoder/reader ahead of transition and reuses it when that path is loaded. This reduces file-open/decoder setup latency but is not full audio-buffer gapless playback.
