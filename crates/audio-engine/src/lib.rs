//! Local streaming playback primitives for Timbre.
//!
//! The engine uses a pull/pump model: an integration layer calls [`PlaybackEngine::pump`]
//! from a worker thread to decode and submit bounded PCM chunks. Never call `pump`
//! from an interactive request or real-time device callback.

pub mod buffer;
pub mod decoder;
pub mod equalizer;
pub mod sink;

use std::{path::Path, sync::Arc};

use timbre_shared_types::Subsystem;

use thiserror::Error;

pub use buffer::AudioRingBuffer;
pub use decoder::{PcmChunk, SymphoniaDecoder};
pub use equalizer::{EqBand, EqSettings, Equalizer};
#[cfg(feature = "cpal")]
pub use sink::CpalAudioSink;
pub use sink::{
    AudioOutputDevice, AudioSink, AudioSinkMetrics, BufferedPcmSink, MemoryAudioSink, adapt_pcm,
    output_devices,
};

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::AudioEngine;

/// Result alias for audio operations.
pub type AudioResult<T> = Result<T, AudioError>;

/// Error returned by decoding, playback, or sink operations.
#[derive(Debug, Error)]
pub enum AudioError {
    /// Local file access failed.
    #[error("audio file I/O failed: {0}")]
    Io(String),
    /// Symphonia rejected a format, packet, or seek request.
    #[error("audio decode failed: {0}")]
    Decode(String),
    /// No audio track was present.
    #[error("no audio track found")]
    NoAudioTrack,
    /// Codec metadata did not provide a sample rate or channel count.
    #[error("audio format metadata is incomplete")]
    MissingAudioFormat,
    /// Seek target was negative or not finite.
    #[error("seek position must be a finite non-negative number")]
    InvalidSeek,
    /// Output sink failed.
    #[error("audio sink failed: {0}")]
    Sink(String),
}

/// Sample rate and number of interleaved PCM channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    /// Samples per second.
    pub sample_rate: u32,
    /// Number of interleaved channels.
    pub channels: u16,
}

/// Observable playback lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// No track is actively playing.
    Stopped,
    /// Audio is being rendered.
    Playing,
    /// A loaded track is temporarily paused.
    Paused,
    /// The current track reached its end.
    TrackEnded,
}

/// Configurable output gain. Muting preserves the selected volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Volume {
    level: f32,
    muted: bool,
}

impl Volume {
    /// Create volume at full level.
    pub const fn new() -> Self {
        Self {
            level: 1.0,
            muted: false,
        }
    }
    /// Linear gain in `[0.0, 1.0]`.
    pub const fn level(self) -> f32 {
        self.level
    }
    /// Whether output is muted.
    pub const fn is_muted(self) -> bool {
        self.muted
    }
    /// Set linear gain, clamped to `[0.0, 1.0]`.
    pub fn set_level(&mut self, level: f32) {
        self.level = if level.is_finite() {
            level.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    /// Set mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronous control surface for local playback; drive decoding via `pump`.
pub struct PlaybackEngine {
    sink: Arc<dyn AudioSink>,
    decoder: Option<SymphoniaDecoder>,
    prepared: Option<(std::path::PathBuf, SymphoniaDecoder)>,
    state: PlaybackState,
    volume: Volume,
    equalizer: Equalizer,
    position_seconds: f64,
    duration_seconds: Option<f64>,
}

impl PlaybackEngine {
    /// Create an engine around a thread-safe audio output sink.
    pub fn new(sink: Arc<dyn AudioSink>) -> Self {
        Self {
            sink,
            decoder: None,
            prepared: None,
            state: PlaybackState::Stopped,
            volume: Volume::new(),
            equalizer: Equalizer::default(),
            position_seconds: 0.0,
            duration_seconds: None,
        }
    }

    /// Open and load a track, configuring the sink from codec metadata.
    pub fn load(&mut self, path: impl AsRef<Path>) -> AudioResult<AudioFormat> {
        let path = path.as_ref();
        let decoder = if self
            .prepared
            .as_ref()
            .is_some_and(|(prepared_path, _)| prepared_path == path)
        {
            self.prepared.take().expect("checked prepared track").1
        } else {
            SymphoniaDecoder::open(path)?
        };
        let format = decoder.audio_format();
        let duration = decoder.duration_seconds();
        self.sink.configure(format)?;
        self.decoder = Some(decoder);
        self.state = PlaybackState::Stopped;
        self.position_seconds = 0.0;
        self.duration_seconds = duration;
        Ok(format)
    }

    /// Prepare a decoder in the background-facing control path for a later load.
    pub fn prepare(&mut self, path: impl AsRef<Path>) -> AudioResult<()> {
        let path = path.as_ref().to_path_buf();
        if self
            .prepared
            .as_ref()
            .is_some_and(|(cached, _)| cached == &path)
        {
            return Ok(());
        }
        self.prepared = Some((path.clone(), SymphoniaDecoder::open(path)?));
        Ok(())
    }
    /// Replace output sink (for device changes); active decoder and transport state are preserved.
    pub fn replace_sink(&mut self, sink: Arc<dyn AudioSink>) {
        self.sink = sink;
    }
    /// Begin or resume playback. A stopped/ended track starts from its current seek position.
    pub fn play(&mut self) {
        if self.decoder.is_some() {
            self.state = PlaybackState::Playing;
        }
    }
    /// Pause playback without discarding the decoder or position.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }
    /// Stop playback and reset the track to its beginning.
    pub fn stop(&mut self) -> AudioResult<()> {
        self.state = PlaybackState::Stopped;
        self.position_seconds = 0.0;
        if let Some(decoder) = &mut self.decoder {
            decoder.seek_seconds(0.0)?;
        }
        Ok(())
    }
    /// Seek to a position in seconds. Seeking does not itself start playback.
    pub fn seek(&mut self, seconds: f64) -> AudioResult<()> {
        let decoder = self
            .decoder
            .as_mut()
            .ok_or_else(|| AudioError::Sink("no track loaded".into()))?;
        if self
            .duration_seconds
            .is_some_and(|duration| seconds > duration)
        {
            return Err(AudioError::InvalidSeek);
        }
        decoder.seek_seconds(seconds)?;
        self.position_seconds = seconds;
        if self.state == PlaybackState::TrackEnded {
            self.state = PlaybackState::Paused;
        }
        Ok(())
    }
    /// Current decoded audio format, if a track is loaded.
    pub fn audio_format(&self) -> Option<AudioFormat> {
        self.decoder.as_ref().map(|d| d.audio_format())
    }
    /// Current lifecycle state.
    pub fn state(&self) -> PlaybackState {
        self.state
    }
    /// Approximate playback position, advanced by PCM frames submitted.
    pub fn position_seconds(&self) -> f64 {
        self.position_seconds
    }
    /// Duration when available from decoder metadata.
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration_seconds
    }
    /// Current volume control.
    pub fn volume(&self) -> Volume {
        self.volume
    }
    /// Update linear gain, clamped to `[0,1]`.
    pub fn set_volume(&mut self, level: f32) {
        self.volume.set_level(level);
    }
    /// Configure the DSP equalizer.
    pub fn set_equalizer(&mut self, settings: EqSettings) -> AudioResult<()> {
        self.equalizer.set_settings(settings)
    }
    /// Current equalizer settings.
    pub fn equalizer(&self) -> &EqSettings {
        self.equalizer.settings()
    }
    /// Current output sink counters and buffer occupancy.
    pub fn sink_metrics(&self) -> AudioSinkMetrics {
        self.sink.metrics()
    }
    /// Enable or disable mute without changing the selected volume.
    pub fn set_muted(&mut self, muted: bool) {
        self.volume.set_muted(muted);
    }

    /// Decode and submit at most `max_chunks` chunks. Returns number of chunks submitted.
    /// At EOF the state changes to [`PlaybackState::TrackEnded`].
    pub fn pump(&mut self, max_chunks: usize) -> AudioResult<usize> {
        if self.state != PlaybackState::Playing {
            return Ok(0);
        }
        let mut submitted = 0;
        for _ in 0..max_chunks {
            let chunk = match self
                .decoder
                .as_mut()
                .ok_or(AudioError::NoAudioTrack)?
                .next_chunk()?
            {
                Some(chunk) => chunk,
                None => {
                    self.state = PlaybackState::TrackEnded;
                    break;
                }
            };
            self.sink.configure(chunk.format)?;
            let mut processed = chunk.samples.clone();
            self.equalizer.process(&mut processed, chunk.format)?;
            let gain = if self.volume.muted {
                0.0
            } else {
                self.volume.level
            };
            let samples: Vec<i16> = processed
                .iter()
                .map(|sample| {
                    ((*sample as f32) * gain)
                        .round()
                        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
                })
                .collect();
            self.sink.write(&samples)?;
            if chunk.format.sample_rate > 0 && chunk.format.channels > 0 {
                self.position_seconds += samples.len() as f64
                    / (chunk.format.sample_rate as f64 * chunk.format.channels as f64);
            }
            submitted += 1;
        }
        Ok(submitted)
    }
}

#[cfg(test)]
mod tests {
    use super::{PlaybackEngine, PlaybackState, Volume};
    use crate::{AudioError, AudioFormat, AudioResult, AudioSink, BufferedPcmSink};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct TestSink(Mutex<Vec<i16>>);
    impl AudioSink for TestSink {
        fn configure(&self, _: AudioFormat) -> AudioResult<()> {
            Ok(())
        }
        fn write(&self, samples: &[i16]) -> AudioResult<()> {
            self.0
                .lock()
                .map_err(|_| AudioError::Sink("poisoned".into()))?
                .extend_from_slice(samples);
            Ok(())
        }
    }

    #[test]
    fn state_transitions_and_noop_controls_without_track() {
        let mut engine = PlaybackEngine::new(Arc::new(TestSink::default()));
        engine.play();
        assert_eq!(engine.state(), PlaybackState::Stopped);
        engine.pause();
        assert_eq!(engine.pump(1).unwrap(), 0);
    }

    #[test]
    fn buffered_sink_transfers_pcm_without_hardware() {
        let sink = BufferedPcmSink::new(4);
        sink.configure(AudioFormat {
            sample_rate: 48_000,
            channels: 2,
        })
        .unwrap();
        // Four samples exactly fit; the consumer can drain them immediately.
        sink.write(&[1, 2, 3, 4]).unwrap();
        let mut output = [0; 4];
        assert_eq!(sink.try_read(&mut output).unwrap(), 4);
        assert_eq!(output, [1, 2, 3, 4]);
        assert_eq!(sink.queued_samples().unwrap(), 0);
        let metrics = crate::AudioSink::metrics(&sink);
        assert_eq!(metrics.capacity_samples, 4);
        assert_eq!(metrics.buffered_samples, 0);
        assert_eq!(metrics.frames_written, 2);
        assert_eq!(metrics.underrun_count, 0);
        let mut empty = [0; 1];
        assert_eq!(sink.try_read(&mut empty).unwrap(), 0);
        assert_eq!(crate::AudioSink::metrics(&sink).underrun_count, 1);
    }

    #[test]
    fn volume_clamps_and_mute_preserves_level() {
        let mut volume = Volume::new();
        volume.set_level(1.5);
        volume.set_muted(true);
        assert_eq!(volume.level(), 1.0);
        assert!(volume.is_muted());
        volume.set_level(f32::NAN);
        assert_eq!(volume.level(), 0.0);
    }
}
