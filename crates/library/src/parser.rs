//! Audio tag and technical-property extraction using Lofty.

use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;

/// Metadata extracted from one supported audio file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioMetadata {
    /// Display title, falling back to the file name when tags omit it.
    pub title: String,
    /// Display artist, falling back to `Unknown Artist` when tags omit it.
    pub artist: String,
    /// Album title, if present and non-empty.
    pub album: Option<String>,
    /// One-based track number, if present and representable.
    pub track_number: Option<u16>,
    /// Duration in milliseconds, if known.
    pub duration_ms: Option<u64>,
    /// Sample rate in hertz, if known.
    pub sample_rate: Option<u32>,
    /// Bits per sample, if known.
    pub bit_depth: Option<u8>,
    /// Number of audio channels, if known.
    pub channels: Option<u8>,
    /// Format label inferred from the file extension (not codec probing).
    pub codec: String,
}

/// Reads supported audio tags and technical properties from a file.
///
/// An unsupported or malformed file returns an error for the scanner to record;
/// missing individual tags/properties are represented with filename fallbacks or
/// `None` rather than failing the entire scan.
pub fn parse_audio_file(path: &Path) -> Result<AudioMetadata, lofty::error::LoftyError> {
    if format_label(path) == "WAV" {
        return parse_wave_properties(path);
    }
    let tagged_file = lofty::read_from_path(path)?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let properties = tagged_file.properties();
    let title = tag
        .and_then(Accessor::title)
        .map(|value| value.into_owned())
        .filter(|value| !value.trim().is_empty())
        .or_else(|| file_title(path))
        .unwrap_or_else(|| "Unknown Track".to_owned());
    let artist = tag
        .and_then(Accessor::artist)
        .map(|value| value.into_owned())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Unknown Artist".to_owned());
    let album = tag
        .and_then(Accessor::album)
        .map(|value| value.into_owned())
        .filter(|value| !value.trim().is_empty());
    let track_number = tag
        .and_then(Accessor::track)
        .and_then(|number| u16::try_from(number).ok());
    let duration_ms = u64::try_from(properties.duration().as_millis()).ok();

    Ok(AudioMetadata {
        title,
        artist,
        album,
        track_number,
        duration_ms,
        sample_rate: properties.sample_rate(),
        bit_depth: properties.bit_depth(),
        channels: properties.channels(),
        codec: codec_label(path),
    })
}

fn parse_wave_properties(path: &Path) -> Result<AudioMetadata, lofty::error::LoftyError> {
    let bytes = std::fs::read(path)?;
    let invalid = || lofty::error::LoftyError::new(lofty::error::ErrorKind::UnknownFormat);
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(invalid());
    }
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes(bytes[24..28].try_into().map_err(|_| invalid())?);
    let byte_rate = u32::from_le_bytes(bytes[28..32].try_into().map_err(|_| invalid())?);
    let bit_depth = u16::from_le_bytes([bytes[34], bytes[35]]);
    let data_offset = bytes
        .windows(4)
        .position(|chunk| chunk == b"data")
        .ok_or_else(invalid)?;
    let data_length = u32::from_le_bytes(
        bytes
            .get(data_offset + 4..data_offset + 8)
            .ok_or_else(invalid)?
            .try_into()
            .map_err(|_| invalid())?,
    );
    if channels == 0 || sample_rate == 0 || byte_rate == 0 {
        return Err(invalid());
    }
    Ok(AudioMetadata {
        title: file_title(path).unwrap_or_else(|| "Unknown Track".to_owned()),
        artist: "Unknown Artist".to_owned(),
        album: None,
        track_number: None,
        duration_ms: Some(u64::from(data_length) * 1000 / u64::from(byte_rate)),
        sample_rate: Some(sample_rate),
        bit_depth: u8::try_from(bit_depth).ok(),
        channels: u8::try_from(channels).ok(),
        codec: codec_label(path),
    })
}

fn file_title(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().trim().to_owned())
        .filter(|stem| !stem.is_empty())
}

/// Returns a file-format label from its extension. A `.m4a` extension does not
/// distinguish AAC from ALAC, so it is intentionally reported as `M4A`.
pub fn format_label(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .map(|extension| match extension.as_str() {
            "flac" => "FLAC".to_owned(),
            "mp3" => "MP3".to_owned(),
            "m4a" => "M4A".to_owned(),
            "aac" => "AAC".to_owned(),
            "ogg" => "OGG".to_owned(),
            "wav" => "WAV".to_owned(),
            "alac" => "ALAC".to_owned(),
            _ => extension.to_ascii_uppercase(),
        })
        .unwrap_or_else(|| "Unknown".to_owned())
}

/// Describes the apparent codec/container using only the filename extension.
///
/// `.m4a` and `.ogg` are containers, not single codecs; the label intentionally
/// says `M4A container` / `OGG container` rather than guessing their audio codec.
fn codec_label(path: &Path) -> String {
    match format_label(path).as_str() {
        "M4A" => "M4A container".to_owned(),
        "OGG" => "OGG container".to_owned(),
        label => label.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempFile {
        path: std::path::PathBuf,
        directory: std::path::PathBuf,
    }

    impl TempFile {
        fn new(name: &str, content: &[u8]) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let directory = std::env::temp_dir().join(format!("timbre-{unique}"));
            fs::create_dir_all(&directory).expect("fixture directory should be created");
            let path = directory.join(name);
            fs::write(&path, content).expect("fixture should be written");
            Self { path, directory }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    fn minimal_pcm_wave() -> Vec<u8> {
        let mut bytes = Vec::new();
        let samples = [0_i16, 0_i16];
        let data_len = (samples.len() * 2) as u32;
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&44_100_u32.to_le_bytes());
        bytes.extend_from_slice(&(44_100_u32 * 4).to_le_bytes());
        bytes.extend_from_slice(&4_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn missing_tags_fall_back_to_filename_and_unknown_artist() {
        let fixture = TempFile::new("untagged.wav", &minimal_pcm_wave());
        let metadata = parse_audio_file(&fixture.path).expect("minimal WAV should parse");
        assert_eq!(metadata.title, "untagged");
        assert_eq!(metadata.artist, "Unknown Artist");
        assert_eq!(metadata.album, None);
        assert_eq!(metadata.codec, "WAV");
        assert_eq!(metadata.sample_rate, Some(44_100));
        assert_eq!(metadata.bit_depth, Some(16));
        assert_eq!(metadata.channels, Some(2));
    }

    #[test]
    fn malformed_audio_returns_an_error_instead_of_panicking() {
        let fixture = TempFile::new("broken.flac", b"not a FLAC stream");
        assert!(parse_audio_file(&fixture.path).is_err());
    }

    #[test]
    fn m4a_label_does_not_claim_a_specific_codec() {
        assert_eq!(format_label(Path::new("song.m4a")), "M4A");
        assert_eq!(codec_label(Path::new("song.m4a")), "M4A container");
        assert_eq!(format_label(Path::new("song.alac")), "ALAC");
    }
}
