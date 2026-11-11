//! Stateful per-channel parametric biquad equalizer.
use crate::{AudioError, AudioFormat, AudioResult};
/// One parametric peak-filter band.
#[derive(Debug, Clone, PartialEq)]
pub struct EqBand {
    /// Center frequency in hertz.
    pub frequency: f32,
    /// Band gain in decibels.
    pub gain_db: f32,
    /// Dimensionless filter Q.
    pub q: f32,
    /// Whether the band is active.
    pub enabled: bool,
}
/// Equalizer configuration applied to decoded PCM.
#[derive(Debug, Clone, PartialEq)]
pub struct EqSettings {
    /// Ordered parametric bands.
    pub bands: Vec<EqBand>,
    /// Overall gain before applying the bands, in decibels.
    pub preamp_db: f32,
}
impl Default for EqSettings {
    fn default() -> Self {
        Self {
            bands: vec![],
            preamp_db: 0.0,
        }
    }
}
#[derive(Clone, Copy, Default)]
struct State {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}
#[derive(Clone, Copy)]
struct Coeff {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
}
/// Stateful per-channel digital equalizer.
pub struct Equalizer {
    settings: EqSettings,
    states: Vec<Vec<State>>,
    rate: u32,
    channels: u16,
}
impl Default for Equalizer {
    fn default() -> Self {
        Self::new(EqSettings::default())
    }
}
impl Equalizer {
    /// Create an equalizer with the provided settings.
    pub fn new(settings: EqSettings) -> Self {
        Self {
            settings,
            states: vec![],
            rate: 0,
            channels: 0,
        }
    }
    /// Return the currently configured equalizer settings.
    pub fn settings(&self) -> &EqSettings {
        &self.settings
    }
    /// Validate and install settings, clearing filter history.
    pub fn set_settings(&mut self, s: EqSettings) -> AudioResult<()> {
        validate(&s, self.rate.max(48000))?;
        self.settings = s;
        self.reset();
        Ok(())
    }
    fn reset(&mut self) {
        self.states =
            vec![vec![State::default(); self.settings.bands.len()]; self.channels as usize];
    }
    /// Apply preamp and enabled filter bands to interleaved signed 16-bit PCM.
    pub fn process(&mut self, samples: &mut [i16], format: AudioFormat) -> AudioResult<()> {
        validate(&self.settings, format.sample_rate)?;
        if format.channels == 0 || samples.len() % format.channels as usize != 0 {
            return Err(AudioError::MissingAudioFormat);
        }
        if self.rate != format.sample_rate || self.channels != format.channels {
            self.rate = format.sample_rate;
            self.channels = format.channels;
            self.reset()
        }
        let coeffs: Vec<_> = self
            .settings
            .bands
            .iter()
            .map(|b| coeff(b.clone(), format.sample_rate))
            .collect();
        let pre = 10f64.powf(self.settings.preamp_db as f64 / 20.0);
        for (i, sample) in samples.iter_mut().enumerate() {
            let ch = i % format.channels as usize;
            let mut x = *sample as f64 * pre;
            for (j, b) in self.settings.bands.iter().enumerate() {
                if !b.enabled || b.gain_db.abs() < 1e-6 {
                    continue;
                }
                let c = coeffs[j];
                let st = &mut self.states[ch][j];
                let y = c.b0 * x + c.b1 * st.x1 + c.b2 * st.x2 - c.a1 * st.y1 - c.a2 * st.y2;
                st.x2 = st.x1;
                st.x1 = x;
                st.y2 = st.y1;
                st.y1 = y;
                x = y;
            }
            *sample = x.round().clamp(i16::MIN as f64, i16::MAX as f64) as i16;
        }
        Ok(())
    }
}
fn validate(s: &EqSettings, rate: u32) -> AudioResult<()> {
    if !s.preamp_db.is_finite()
        || s.preamp_db.abs() > 24.0
        || s.bands.iter().any(|b| {
            !b.frequency.is_finite()
                || b.frequency <= 0.0
                || b.frequency >= rate as f32 / 2.0
                || !b.q.is_finite()
                || b.q <= 0.0
                || b.gain_db.abs() > 24.0
                || !b.gain_db.is_finite()
        })
    {
        return Err(AudioError::Sink("invalid equalizer settings".into()));
    }
    Ok(())
}
fn coeff(b: EqBand, rate: u32) -> Coeff {
    let a = 10f64.powf(b.gain_db as f64 / 40.0);
    let w = 2.0 * std::f64::consts::PI * b.frequency as f64 / rate as f64;
    let alpha = w.sin() / (2.0 * b.q as f64);
    let c = w.cos();
    let a0 = 1.0 + alpha / a;
    let (b0, b1, b2) = (1.0 + alpha * a, -2.0 * c, 1.0 - alpha * a);
    let (a1, a2) = (-2.0 * c, 1.0 - alpha / a);
    Coeff {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}
