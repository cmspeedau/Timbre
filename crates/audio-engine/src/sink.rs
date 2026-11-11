//! Audio output abstractions and deterministic buffered sinks.

use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicU64, Ordering},
};

use crate::{AudioError, AudioFormat, AudioResult, AudioRingBuffer};

/// Remap interleaved PCM to an output rate/channel layout using linear interpolation.
/// Mono expands to all channels; multichannel-to-mono averages channels; other layouts
/// preserve matching channel indices and fill added channels with a stereo average.
pub fn adapt_pcm(input: &[i16], source: AudioFormat, target: AudioFormat) -> AudioResult<Vec<i16>> {
    if source.sample_rate == 0
        || target.sample_rate == 0
        || source.channels == 0
        || target.channels == 0
    {
        return Err(AudioError::MissingAudioFormat);
    }
    let src_ch = source.channels as usize;
    let dst_ch = target.channels as usize;
    if input.len() % src_ch != 0 {
        return Err(AudioError::Sink(
            "PCM sample count is not frame-aligned".into(),
        ));
    }
    let frames = input.len() / src_ch;
    if frames == 0 {
        return Ok(Vec::new());
    }
    let out_frames = ((frames as u128 * target.sample_rate as u128
        + source.sample_rate as u128 / 2)
        / source.sample_rate as u128)
        .min(usize::MAX as u128) as usize;
    let mut output = Vec::with_capacity(out_frames.saturating_mul(dst_ch));
    for out_frame in 0..out_frames {
        let pos = out_frame as f64 * source.sample_rate as f64 / target.sample_rate as f64;
        let left = (pos.floor() as usize).min(frames - 1);
        let right = (left + 1).min(frames - 1);
        let frac = (pos - left as f64).clamp(0.0, 1.0) as f32;
        let mut frame = Vec::with_capacity(src_ch);
        for ch in 0..src_ch {
            let a = input[left * src_ch + ch] as f32;
            let b = input[right * src_ch + ch] as f32;
            frame.push((a + (b - a) * frac).clamp(i16::MIN as f32, i16::MAX as f32));
        }
        for ch in 0..dst_ch {
            let value = if dst_ch == 1 {
                frame.iter().sum::<f32>() / src_ch as f32
            } else if src_ch == 1 {
                frame[0]
            } else if ch < src_ch {
                frame[ch]
            } else {
                (frame[0] + frame[1]) * 0.5
            };
            output.push(value.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16);
        }
    }
    Ok(output)
}

/// Descriptive output device capabilities exposed by the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioOutputDevice {
    /// Stable identifier used to request this device.
    pub id: String,
    /// Human-readable output device name.
    pub name: String,
    /// Whether the system currently marks this device as default.
    pub is_default: bool,
    /// Advertised sample rates in hertz.
    pub sample_rates: Vec<u32>,
    /// Advertised channel counts.
    pub channels: Vec<u16>,
}

/// Enumerate available output devices (or a deterministic memory sink without CPAL).
pub fn output_devices() -> Vec<AudioOutputDevice> {
    #[cfg(feature = "cpal")]
    {
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        let default_name = host.default_output_device().and_then(|d| d.name().ok());
        let mut devices = Vec::new();
        if let Ok(iter) = host.output_devices() {
            for device in iter {
                if let Ok(name) = device.name() {
                    let mut rates = Vec::new();
                    let mut channels = Vec::new();
                    if let Ok(configs) = device.supported_output_configs() {
                        for c in configs {
                            rates.extend([c.min_sample_rate().0, c.max_sample_rate().0]);
                            channels.push(c.channels());
                        }
                    }
                    rates.sort_unstable();
                    rates.dedup();
                    channels.sort_unstable();
                    channels.dedup();
                    devices.push(AudioOutputDevice {
                        id: format!("cpal:{name}"),
                        is_default: default_name.as_deref() == Some(name.as_str()),
                        name,
                        sample_rates: rates,
                        channels,
                    });
                }
            }
        }
        if !devices.is_empty() {
            return devices;
        }
    }
    vec![AudioOutputDevice {
        id: "memory".into(),
        name: "Memory / headless output".into(),
        is_default: true,
        sample_rates: vec![],
        channels: vec![],
    }]
}

/// Snapshot counters and occupancy reported by an audio sink. Counts are interleaved samples.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AudioSinkMetrics {
    /// Maximum interleaved samples the sink can buffer.
    pub capacity_samples: u64,
    /// Interleaved samples currently buffered.
    pub buffered_samples: u64,
    /// Number of output callback underruns.
    pub underrun_count: u64,
    /// Frames successfully submitted to the sink.
    pub frames_written: u64,
    /// Frames dropped because the sink buffer was full.
    pub frames_dropped: u64,
}

/// Thread-safe destination for interleaved signed 16-bit PCM.
pub trait AudioSink: Send + Sync {
    /// Configure sample rate and channel count before playback.
    fn configure(&self, format: AudioFormat) -> AudioResult<()>;
    /// Write a frame-aligned PCM chunk to the consumer.
    fn write(&self, interleaved_samples: &[i16]) -> AudioResult<()>;
    /// Return best-effort sink counters; unsupported values are zero.
    fn metrics(&self) -> AudioSinkMetrics {
        AudioSinkMetrics::default()
    }
}

#[derive(Debug, Default)]
struct MemoryState {
    format: Option<AudioFormat>,
    samples: Vec<i16>,
}

/// Thread-safe sink useful for headless playback and deterministic tests.
#[derive(Debug, Clone, Default)]
pub struct MemoryAudioSink(Arc<Mutex<MemoryState>>);

impl MemoryAudioSink {
    /// Create an empty in-memory sink.
    pub fn new() -> Self {
        Self::default()
    }
    /// Return the configured sample format, if any.
    pub fn format(&self) -> Option<AudioFormat> {
        self.0.lock().ok().and_then(|s| s.format)
    }
    /// Clone the samples written so far.
    pub fn samples(&self) -> Vec<i16> {
        self.0.lock().map(|s| s.samples.clone()).unwrap_or_default()
    }
    /// Remove all recorded samples.
    pub fn clear(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.samples.clear();
        }
    }
}

impl AudioSink for MemoryAudioSink {
    fn configure(&self, format: AudioFormat) -> AudioResult<()> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        state.format = Some(format);
        Ok(())
    }
    fn write(&self, samples: &[i16]) -> AudioResult<()> {
        let mut state = self
            .0
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        state.samples.extend_from_slice(samples);
        Ok(())
    }
}

#[derive(Debug)]
struct BufferedState {
    format: Option<AudioFormat>,
    queue: AudioRingBuffer<i16>,
}

/// Bounded thread-safe PCM FIFO. Writers apply backpressure rather than dropping
/// samples; a device or other consumer drains samples with [`Self::read`].
/// This makes a robust headless/adapter output without committing to a device API.
#[derive(Debug)]
pub struct BufferedPcmSink {
    state: Mutex<BufferedState>,
    underrun_count: AtomicU64,
    frames_written: AtomicU64,
    frames_consumed: AtomicU64,
    readable: Condvar,
    writable: Condvar,
}

impl BufferedPcmSink {
    /// Construct a FIFO with capacity measured in interleaved samples.
    pub fn new(capacity_samples: usize) -> Self {
        Self {
            state: Mutex::new(BufferedState {
                format: None,
                queue: AudioRingBuffer::new(capacity_samples),
            }),
            underrun_count: AtomicU64::new(0),
            frames_written: AtomicU64::new(0),
            frames_consumed: AtomicU64::new(0),
            readable: Condvar::new(),
            writable: Condvar::new(),
        }
    }
    /// Return the configured output format.
    pub fn format(&self) -> Option<AudioFormat> {
        self.state.lock().ok().and_then(|s| s.format)
    }
    /// Read up to `output.len()` samples, blocking until at least one is available.
    pub fn read(&self, output: &mut [i16]) -> AudioResult<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        while state.queue.is_empty() {
            state = self
                .readable
                .wait(state)
                .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        }
        let count = state.queue.pop(output);
        self.frames_consumed.fetch_add(
            (count / state.format.map_or(1, |f| f.channels.max(1) as usize)) as u64,
            Ordering::Relaxed,
        );
        self.writable.notify_all();
        Ok(count)
    }
    /// Read available samples immediately without blocking.
    pub fn try_read(&self, output: &mut [i16]) -> AudioResult<usize> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        let count = state.queue.pop(output);
        if count > 0 {
            self.frames_consumed.fetch_add(
                (count / state.format.map_or(1, |f| f.channels.max(1) as usize)) as u64,
                Ordering::Relaxed,
            );
            self.writable.notify_all();
        } else if !output.is_empty() {
            self.underrun_count.fetch_add(1, Ordering::Relaxed);
        }
        Ok(count)
    }
    /// Number of samples queued for the consumer.
    pub fn queued_samples(&self) -> AudioResult<usize> {
        self.state
            .lock()
            .map(|s| s.queue.len())
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))
    }
}

impl AudioSink for BufferedPcmSink {
    fn configure(&self, format: AudioFormat) -> AudioResult<()> {
        if format.sample_rate == 0 || format.channels == 0 {
            return Err(AudioError::MissingAudioFormat);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        state.format = Some(format);
        Ok(())
    }
    fn write(&self, mut samples: &[i16]) -> AudioResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
        let submitted_samples = samples.len();
        let channels = state.format.map_or(1, |f| f.channels.max(1) as usize);
        while !samples.is_empty() {
            while state.queue.available() == 0 {
                state = self
                    .writable
                    .wait(state)
                    .map_err(|_| AudioError::Sink("sink lock poisoned".into()))?;
            }
            let accepted = state.queue.push(samples);
            samples = &samples[accepted..];
            self.readable.notify_all();
        }
        self.frames_written
            .fetch_add((submitted_samples / channels) as u64, Ordering::Relaxed);
        Ok(())
    }
    fn metrics(&self) -> AudioSinkMetrics {
        let Ok(state) = self.state.lock() else {
            return AudioSinkMetrics::default();
        };
        AudioSinkMetrics {
            capacity_samples: state.queue.capacity() as u64,
            buffered_samples: state.queue.len() as u64,
            underrun_count: self.underrun_count.load(Ordering::Relaxed),
            frames_written: self.frames_written.load(Ordering::Relaxed),
            frames_dropped: 0,
        }
    }
}

/// CPAL hardware sink adapter, enabled with the `cpal` feature.
#[cfg(feature = "cpal")]
pub struct CpalAudioSink {
    format: Mutex<Option<AudioFormat>>,
    output_format: AudioFormat,
    samples: Arc<Mutex<std::collections::VecDeque<i16>>>,
    _worker_lifetime: Mutex<Option<std::sync::mpsc::Sender<()>>>,
}

#[cfg(feature = "cpal")]
impl CpalAudioSink {
    /// Open the system's default output device; returns an error if unavailable.
    pub fn open_default() -> AudioResult<Self> {
        Self::open_device(None)
    }
    /// Open a CPAL device by its `cpal:<name>` ID, or the system default when `None`.
    pub fn open_device(id: Option<&str>) -> AudioResult<Self> {
        let id = id.map(str::to_owned);
        let samples = Arc::new(Mutex::new(std::collections::VecDeque::<i16>::new()));
        let callback_samples = Arc::clone(&samples);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let (lifetime_tx, lifetime_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("timbre-cpal-output".into())
            .spawn(move || {
                let stream = match build_cpal_stream(id.as_deref(), callback_samples) {
                    Ok((format, stream)) => {
                        if ready_tx.send(Ok(format)).is_err() {
                            return;
                        }
                        stream
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error.to_string()));
                        return;
                    }
                };
                let _stream = stream;
                loop {
                    match lifetime_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    }
                }
            })
            .map_err(|e| AudioError::Sink(e.to_string()))?;
        let output_format = ready_rx
            .recv()
            .map_err(|e| AudioError::Sink(e.to_string()))?
            .map_err(AudioError::Sink)?;
        Ok(Self {
            format: Mutex::new(None),
            output_format,
            samples,
            _worker_lifetime: Mutex::new(Some(lifetime_tx)),
        })
    }
}

#[cfg(feature = "cpal")]
fn build_cpal_stream(
    id: Option<&str>,
    samples: Arc<Mutex<std::collections::VecDeque<i16>>>,
) -> AudioResult<(AudioFormat, cpal::Stream)> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let host = cpal::default_host();
    let device = if let Some(id) = id {
        let requested = id
            .strip_prefix("cpal:")
            .ok_or_else(|| AudioError::Sink("invalid CPAL device ID".into()))?;
        host.output_devices()
            .map_err(|e| AudioError::Sink(e.to_string()))?
            .find(|d| d.name().ok().as_deref() == Some(requested))
            .ok_or_else(|| AudioError::Sink(format!("output device not found: {requested}")))?
    } else {
        host.default_output_device()
            .ok_or_else(|| AudioError::Sink("no default output device".into()))?
    };
    let config = device
        .default_output_config()
        .map_err(|e| AudioError::Sink(e.to_string()))?;
    let sample_format = config.sample_format();
    let output_format = AudioFormat {
        sample_rate: config.sample_rate().0,
        channels: config.channels(),
    };
    let stream_config: cpal::StreamConfig = config.into();
    let error_callback = |e| eprintln!("audio output stream error: {e}");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &stream_config,
            move |out: &mut [f32], _| fill_f32(out, &samples),
            error_callback,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &stream_config,
            move |out: &mut [i16], _| fill_i16(out, &samples),
            error_callback,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_output_stream(
            &stream_config,
            move |out: &mut [u16], _| fill_u16(out, &samples),
            error_callback,
            None,
        ),
        format => {
            return Err(AudioError::Sink(format!(
                "unsupported CPAL sample format: {format:?}"
            )));
        }
    }
    .map_err(|e| AudioError::Sink(e.to_string()))?;
    stream.play().map_err(|e| AudioError::Sink(e.to_string()))?;
    Ok((output_format, stream))
}

#[cfg(feature = "cpal")]
fn fill_f32(output: &mut [f32], samples: &Arc<Mutex<std::collections::VecDeque<i16>>>) {
    if let Ok(mut queue) = samples.try_lock() {
        for value in output {
            *value = queue.pop_front().unwrap_or(0) as f32 / 32768.0;
        }
    } else {
        output.fill(0.0);
    }
}
#[cfg(feature = "cpal")]
fn fill_i16(output: &mut [i16], samples: &Arc<Mutex<std::collections::VecDeque<i16>>>) {
    if let Ok(mut queue) = samples.try_lock() {
        for value in output {
            *value = queue.pop_front().unwrap_or(0);
        }
    } else {
        output.fill(0);
    }
}
#[cfg(feature = "cpal")]
fn fill_u16(output: &mut [u16], samples: &Arc<Mutex<std::collections::VecDeque<i16>>>) {
    if let Ok(mut queue) = samples.try_lock() {
        for value in output {
            *value = (queue.pop_front().unwrap_or(0) as i32 + 32768) as u16;
        }
    } else {
        output.fill(32768);
    }
}

#[cfg(feature = "cpal")]
impl AudioSink for CpalAudioSink {
    fn configure(&self, format: AudioFormat) -> AudioResult<()> {
        *self
            .format
            .lock()
            .map_err(|_| AudioError::Sink("CPAL format lock poisoned".into()))? = Some(format);
        Ok(())
    }
    fn write(&self, samples: &[i16]) -> AudioResult<()> {
        const MAX_BUFFERED_SAMPLES: usize = 192_000;
        let source = self
            .format
            .lock()
            .map_err(|_| AudioError::Sink("CPAL format lock poisoned".into()))?
            .ok_or(AudioError::MissingAudioFormat)?;
        let converted = adapt_pcm(samples, source, self.output_format)?;
        let mut queue = self
            .samples
            .lock()
            .map_err(|_| AudioError::Sink("CPAL buffer lock poisoned".into()))?;
        if queue.len().saturating_add(converted.len()) > MAX_BUFFERED_SAMPLES {
            return Err(AudioError::Sink("CPAL output buffer is full".into()));
        }
        queue.extend(converted);
        Ok(())
    }
}

#[cfg(test)]
mod adaptation_tests {
    use super::adapt_pcm;
    use crate::AudioFormat;
    #[test]
    fn resamples_duration_and_interpolates() {
        let out = adapt_pcm(
            &[0, 1000, 2000],
            AudioFormat {
                sample_rate: 2,
                channels: 1,
            },
            AudioFormat {
                sample_rate: 4,
                channels: 1,
            },
        )
        .unwrap();
        assert_eq!(out, vec![0, 500, 1000, 1500, 2000, 2000]);
    }
    #[test]
    fn remixes_mono_stereo_and_multichannel() {
        let stereo = adapt_pcm(
            &[100, -100],
            AudioFormat {
                sample_rate: 48_000,
                channels: 1,
            },
            AudioFormat {
                sample_rate: 48_000,
                channels: 2,
            },
        )
        .unwrap();
        assert_eq!(stereo, vec![100, 100, -100, -100]);
        let mono = adapt_pcm(
            &[100, -100],
            AudioFormat {
                sample_rate: 48_000,
                channels: 2,
            },
            AudioFormat {
                sample_rate: 48_000,
                channels: 1,
            },
        )
        .unwrap();
        assert_eq!(mono, vec![0]);
        let multi = adapt_pcm(
            &[100, -100],
            AudioFormat {
                sample_rate: 48_000,
                channels: 2,
            },
            AudioFormat {
                sample_rate: 48_000,
                channels: 4,
            },
        )
        .unwrap();
        assert_eq!(multi, vec![100, -100, 0, 0]);
    }
}
