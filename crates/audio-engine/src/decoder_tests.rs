use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use super::SymphoniaDecoder;

struct WavFixture(std::path::PathBuf);

impl Drop for WavFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn wav_fixture() -> WavFixture {
    let mut bytes = Vec::new();
    let samples: [i16; 4] = [-12_000, -1, 1, 12_000];
    let data_len = (samples.len() * 2) as u32;
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8_000u32.to_le_bytes());
    bytes.extend_from_slice(&16_000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("timbre-audio-{nonce}.wav"));
    fs::write(&path, bytes).unwrap();
    WavFixture(path)
}

#[test]
fn decodes_wav_and_reaches_end_of_stream() {
    let path = wav_fixture();
    let mut decoder = SymphoniaDecoder::open(&path.0).unwrap();
    assert_eq!(decoder.duration_seconds(), Some(0.0005));
    assert_eq!(decoder.audio_format().sample_rate, 8_000);
    assert_eq!(decoder.audio_format().channels, 1);
    let mut all = Vec::new();
    while let Some(chunk) = decoder.next_chunk().unwrap() {
        all.extend(chunk.samples);
    }
    assert_eq!(all, [-12_000, -1, 1, 12_000]);
    assert!(decoder.next_chunk().unwrap().is_none());
}

#[test]
fn rejects_invalid_seek_positions() {
    let path = wav_fixture();
    let mut decoder = SymphoniaDecoder::open(&path.0).unwrap();
    assert!(decoder.seek_seconds(-1.0).is_err());
    assert!(decoder.seek_seconds(f64::NAN).is_err());
}

#[test]
fn playback_engine_applies_gain_and_mute_to_decoded_pcm() {
    use crate::{MemoryAudioSink, PlaybackEngine};
    use std::sync::Arc;

    let path = wav_fixture();
    let sink = Arc::new(MemoryAudioSink::new());
    let mut engine = PlaybackEngine::new(sink.clone());
    engine.load(&path.0).unwrap();
    engine.set_volume(0.5);
    engine.play();
    engine.pump(8).unwrap();
    let gained = sink.samples();
    assert_eq!(gained, [-6000, -1, 1, 6000]);

    sink.clear();
    engine.seek(0.0).unwrap();
    engine.set_muted(true);
    engine.play();
    engine.pump(8).unwrap();
    assert_eq!(sink.samples(), [0, 0, 0, 0]);
    assert_eq!(engine.volume().level(), 0.5, "mute preserves selected gain");
}

#[test]
fn playback_engine_seek_updates_position_and_rejects_past_duration() {
    use crate::{MemoryAudioSink, PlaybackEngine};
    use std::sync::Arc;
    let path = wav_fixture();
    let mut engine = PlaybackEngine::new(Arc::new(MemoryAudioSink::new()));
    engine.load(&path.0).unwrap();
    assert_eq!(engine.duration_seconds(), Some(0.0005));
    assert!(engine.seek(0.0006).is_err());
    engine.seek(0.00025).unwrap();
    assert!((engine.position_seconds() - 0.00025).abs() < 1e-9);
}
