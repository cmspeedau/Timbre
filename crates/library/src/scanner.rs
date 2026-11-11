//! Recursive discovery and per-file ingestion for supported audio formats.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::parser::{AudioMetadata, parse_audio_file};

/// Extensions whose files are handed to the audio metadata parser.
pub const SUPPORTED_EXTENSIONS: &[&str] = &["flac", "mp3", "m4a", "aac", "ogg", "wav", "alac"];

/// A recoverable file or directory issue encountered during a scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanIssue {
    /// File or directory that caused the issue, when known.
    pub path: Option<PathBuf>,
    /// Human-readable error detail.
    pub message: String,
}

/// One discovered audio file and its best-effort metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScannedAudioFile {
    /// Filesystem path reported by `walkdir` for this entry.
    pub path: PathBuf,
    /// Parsed tags and audio properties.
    pub metadata: AudioMetadata,
}

/// The successful files and recoverable issues from a directory traversal.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// Files parsed successfully.
    pub files: Vec<ScannedAudioFile>,
    /// Entries skipped because traversal or parsing failed.
    pub issues: Vec<ScanIssue>,
}

/// Minimal write boundary for the library's catalog/storage implementation.
///
/// Milestone 1 does not currently provide a repository implementation in this
/// checkout. Storage backends can implement this trait without coupling the
/// scanner to a database or API contract.
pub trait AudioMetadataRepository {
    /// Inserts or updates metadata associated with a physical file path.
    fn upsert_audio_file(&mut self, path: &Path, metadata: &AudioMetadata) -> Result<(), String>;
}

/// Scans one explicitly supplied directory recursively without following symlinks.
///
/// Files are returned in the traversal order produced by `walkdir`; callers that
/// expose a stable API should sort by the returned path before serializing results.
///
/// A missing or non-directory root is a configuration error and returns `Err`;
/// individual unreadable directories, corrupt audio files, and tag parse failures
/// are collected in `ScanReport::issues` and do not stop other files from scanning.
pub fn scan_directory(root: &Path) -> Result<ScanReport, std::io::Error> {
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "scan root is not an accessible directory: {}",
                root.display()
            ),
        ));
    }

    let mut report = ScanReport::default();
    for entry in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                report.issues.push(ScanIssue {
                    path: error.path().map(Path::to_path_buf),
                    message: error.to_string(),
                });
                continue;
            }
        };
        if !entry.file_type().is_file() || !has_supported_extension(entry.path()) {
            continue;
        }

        match parse_audio_file(entry.path()) {
            Ok(metadata) => report.files.push(ScannedAudioFile {
                path: entry.into_path(),
                metadata,
            }),
            Err(error) => report.issues.push(ScanIssue {
                path: Some(entry.into_path()),
                message: error.to_string(),
            }),
        }
    }
    Ok(report)
}

/// Scans a directory and passes each successfully parsed file to a repository.
///
/// A repository error is recorded for that file while later files continue to
/// be ingested. The returned count includes only successful repository writes.
pub fn scan_into_repository<R: AudioMetadataRepository>(
    root: &Path,
    repository: &mut R,
) -> Result<(ScanReport, usize), std::io::Error> {
    let mut report = scan_directory(root)?;
    let mut stored = 0;
    for file in &report.files {
        match repository.upsert_audio_file(&file.path, &file.metadata) {
            Ok(()) => stored += 1,
            Err(message) => report.issues.push(ScanIssue {
                path: Some(file.path.clone()),
                message: format!("repository upsert failed: {message}"),
            }),
        }
    }
    Ok((report, stored))
}

/// Returns whether a path has one of the extensions recognized by the scanner.
pub fn has_supported_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("timbre-scan-{unique}"));
            fs::create_dir_all(&path).expect("fixture directory should be created");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn minimal_pcm_wave() -> Vec<u8> {
        Vec::from(
            &b"RIFF\x28\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x44\xac\x00\x00\x10\xb1\x02\x00\x04\x00\x10\x00data\x04\x00\x00\x00\x00\x00\x00\x00"[..],
        )
    }

    #[test]
    fn scans_recursively_skips_unsupported_and_isolates_corrupt_audio() {
        let root = TempDir::new();
        let nested = root.0.join("disc 1");
        fs::create_dir(&nested).expect("nested directory should be created");
        fs::write(nested.join("quiet.WAV"), minimal_pcm_wave()).expect("WAV should be written");
        fs::write(nested.join("broken.flac"), b"bad audio")
            .expect("corrupt audio should be written");
        fs::write(root.0.join("cover.jpg"), b"image").expect("non-audio file should be written");

        let report = scan_directory(&root.0).expect("root should scan");
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].metadata.title, "quiet");
        assert_eq!(report.issues.len(), 1);
        assert!(
            report.issues[0]
                .path
                .as_deref()
                .is_some_and(|path| path.ends_with("broken.flac"))
        );
    }

    #[test]
    fn supports_requested_extensions_case_insensitively() {
        for extension in SUPPORTED_EXTENSIONS {
            assert!(has_supported_extension(Path::new(&format!(
                "track.{extension}"
            ))));
            assert!(has_supported_extension(Path::new(&format!(
                "track.{}",
                extension.to_uppercase()
            ))));
        }
        assert!(!has_supported_extension(Path::new("cover.jpg")));
    }

    #[test]
    fn rejects_missing_scan_roots() {
        let missing = std::env::temp_dir().join("timbre-definitely-missing-root");
        assert!(scan_directory(&missing).is_err());
    }

    #[derive(Default)]
    struct MemoryRepository(Vec<PathBuf>);

    impl AudioMetadataRepository for MemoryRepository {
        fn upsert_audio_file(
            &mut self,
            path: &Path,
            metadata: &AudioMetadata,
        ) -> Result<(), String> {
            assert!(!metadata.title.is_empty());
            self.0.push(path.to_path_buf());
            Ok(())
        }
    }

    #[test]
    fn parsed_files_are_forwarded_to_the_storage_boundary() {
        let root = TempDir::new();
        fs::write(root.0.join("album.wav"), minimal_pcm_wave()).expect("WAV should be written");
        let mut repository = MemoryRepository::default();
        let (report, stored) =
            scan_into_repository(&root.0, &mut repository).expect("scan should succeed");
        assert_eq!(stored, 1);
        assert_eq!(report.files.len(), 1);
        assert_eq!(repository.0, vec![root.0.join("album.wav")]);
    }
}
