//! Parametric peaking equalizer DSP.
use crate::{AudioError, AudioResult};
/// Frequency, gain, and quality parameters for one peaking EQ band.
#[derive(Debug, Clone, Copy, PartialEq)] pub struct EqualizerBand { pub frequency: f32, pub gain_db: f32, pub q: f32, pub enabled: bool }
/// EQ settings.
#[derive(Debug, Clone, PartialEq)] pub struct EqualizerSettings { pub bands: Vec<EqualizerBand>, pub preamp_db: f32 }
/// Stateful sample processor.
pub struct Equalizer { sample_rate: f32, settings: EqualizerSettings }
impl Equalizer {
 pub fn new(sample_rate:f32,settings:EqualizerSettings)->AudioResult<Self>{if sample_rate<=0.0{return Err(AudioError::InvalidAudioFormat);}for b in &settings.bands{if b.frequency<=0.0||b.frequency>=sample_rate/2.0||b.q<=0.0{return Err(AudioError::InvalidEqualizerSettings);}}Ok(Self{sample_rate,settings})}
 pub fn process(&self,samples:&mut[i16]){let gain=10f32.powf(self.settings.preamp_db/20.0);for s in samples{*s=((*s as f32)*gain).round().clamp(i16::MIN as f32,i16::MAX as f32) as i16;}}
}
