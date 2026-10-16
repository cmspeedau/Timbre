//! Audio format and signal-path contracts.

use serde::{Deserialize, Serialize};

/// A PCM audio format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFormat {
    /// Sample rate in hertz, for example `44100`.
    pub sample_rate_hz: u32,
    /// Bits per sample, for example `16` or `24`.
    pub bit_depth: u16,
    /// Number of channels, for example `2` for stereo.
    pub channels: u16,
}

/// One step in the audio signal path, from source to output.
///
/// The signal path is the ordered list of transformations a stream passes
/// through. It is a first-class contract because clients display it and because
/// bit-perfect output means an empty list between decode and the device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalPathStep {
    /// A short, stable step type, for example `decode` or `resample`.
    pub kind: String,
    /// A human-readable description for the client's signal-path display.
    pub description: String,
    /// Format entering the step.
    pub input: AudioFormat,
    /// Format leaving the step.
    pub output: AudioFormat,
}

/// The ordered processing applied to one stream before it reaches output.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalPath {
    /// Steps in order, from decode to output.
    pub steps: Vec<SignalPathStep>,
}

impl SignalPath {
    /// Returns `true` when no transformation occurs between decode and output.
    #[must_use]
    pub fn is_bit_perfect(&self) -> bool {
        self.steps.is_empty()
    }
}
