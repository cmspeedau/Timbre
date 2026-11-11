//! Symphonia-backed file decoder for common local audio formats.

use std::{fs::File, path::Path};

use symphonia::core::{
    audio::SampleBuffer,
    codecs::{Decoder, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, SeekMode, SeekTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

use crate::{AudioError, AudioFormat, AudioResult};

#[cfg(test)]
#[path = "decoder_tests.rs"]
mod tests;

/// A decoded chunk of interleaved signed 16-bit PCM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcmChunk {
    /// Sample rate and channel count for the chunk.
    pub format: AudioFormat,
    /// Interleaved signed 16-bit samples.
    pub samples: Vec<i16>,
}

/// Streaming decoder for FLAC, MP3, WAV/PCM, AAC/M4A, and OGG/Vorbis files.
pub struct SymphoniaDecoder {
    reader: Box<dyn symphonia::core::formats::FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    format: AudioFormat,
    duration_seconds: Option<f64>,
}

impl SymphoniaDecoder {
    /// Open a local audio file and select its default audio track.
    pub fn open(path: impl AsRef<Path>) -> AudioResult<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|e| AudioError::Io(e.to_string()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AudioError::Decode(e.to_string()))?;
        let reader = probed.format;
        let track = reader.default_track().ok_or(AudioError::NoAudioTrack)?;
        let track_id = track.id;
        let format = AudioFormat {
            sample_rate: track
                .codec_params
                .sample_rate
                .ok_or(AudioError::MissingAudioFormat)?,
            channels: track
                .codec_params
                .channels
                .map(|c| c.count() as u16)
                .ok_or(AudioError::MissingAudioFormat)?,
        };
        let duration_seconds = track
            .codec_params
            .n_frames
            .map(|frames| frames as f64 / format.sample_rate as f64);
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| AudioError::Decode(e.to_string()))?;
        Ok(Self {
            reader,
            decoder,
            track_id,
            format,
            duration_seconds,
        })
    }

    /// Current decoded PCM format.
    pub fn audio_format(&self) -> AudioFormat {
        self.format
    }

    /// Duration from stream metadata, if the container/codec declares a frame count.
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration_seconds
    }

    /// Decode the next packet belonging to the selected audio track.
    /// Returns `Ok(None)` at end-of-stream.
    pub fn next_chunk(&mut self) -> AudioResult<Option<PcmChunk>> {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(err))
                    if err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(None);
                }
                Err(SymphoniaError::ResetRequired) => {
                    return Err(AudioError::Decode("decoder reset required".into()));
                }
                Err(err) => return Err(AudioError::Decode(err.to_string())),
            };
            if packet.track_id() != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(err) => return Err(AudioError::Decode(err.to_string())),
            };
            let format = AudioFormat {
                sample_rate: decoded.spec().rate,
                channels: decoded.spec().channels.count() as u16,
            };
            self.format = format;
            let mut pcm = SampleBuffer::<i16>::new(decoded.capacity() as u64, *decoded.spec());
            pcm.copy_interleaved_ref(decoded);
            return Ok(Some(PcmChunk {
                format,
                samples: pcm.samples().to_vec(),
            }));
        }
    }

    /// Seek to the closest packet at or before the requested timestamp.
    pub fn seek_seconds(&mut self, seconds: f64) -> AudioResult<()> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(AudioError::InvalidSeek);
        }
        let time = symphonia::core::units::Time::from(seconds);
        self.reader
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| AudioError::Decode(e.to_string()))?;
        self.decoder.reset();
        Ok(())
    }
}
