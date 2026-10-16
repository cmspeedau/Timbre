//! Phase 0 exit gate: the shared contracts serialize, the API version is owned
//! in one place, and every subsystem identifies itself through the agreed
//! interface.

use std::collections::HashSet;

use timbre_api_contracts::{
    API_VERSION, Album, Artist, AudioFormat, Credit, CreditRole, Device, DeviceCapabilities,
    DeviceCapability, MediaItem, PlaybackQueue, PlaybackState, QueueItem, Recording, RepeatMode,
    SignalPath, SignalPathStep, TransportState, Zone,
};
use timbre_shared_types::{
    AlbumId, ArtistId, DeviceId, JobId, JobProgress, JobState, JobStatus, MediaItemId, RecordingId,
    Subsystem, ZoneId,
};
use timbre_testing::roundtrip;

#[test]
fn api_version_is_v1() {
    assert_eq!(API_VERSION, "v1");
}

#[test]
fn server_and_clients_agree_on_the_contract_version() {
    assert_eq!(timbre_server::API_VERSION, API_VERSION);
    assert_eq!(timbre_ui::API_VERSION, API_VERSION);
    assert_eq!(timbre_remote_access::API_VERSION, API_VERSION);
}

#[test]
fn every_subsystem_identifies_itself() {
    let declared: [Subsystem; 8] = [
        timbre_server::SUBSYSTEM,
        timbre_library::SUBSYSTEM,
        timbre_audio_engine::SUBSYSTEM,
        timbre_devices::SUBSYSTEM,
        timbre_discovery::SUBSYSTEM,
        timbre_remote_access::SUBSYSTEM,
        timbre_integrations::SUBSYSTEM,
        timbre_ui::SUBSYSTEM,
    ];

    let mut seen = HashSet::new();
    for subsystem in declared {
        assert!(seen.insert(subsystem), "duplicate subsystem {subsystem:?}");
    }
    assert_eq!(seen.len(), 8, "every subsystem reports exactly once");
}

#[test]
fn library_models_round_trip() {
    let artist = Artist {
        id: ArtistId::generate(),
        name: "Hildegard von Bingen".to_owned(),
    };
    roundtrip(&artist);

    let album = Album {
        id: AlbumId::generate(),
        title: "Ordo Virtutum".to_owned(),
        artist: Some(artist.id),
        year: Some(1151),
    };
    roundtrip(&album);

    let recording = Recording {
        id: RecordingId::generate(),
        title: "Prologue".to_owned(),
        album: Some(album.id),
        track_number: Some(1),
        duration_ms: Some(272_000),
    };
    roundtrip(&recording);

    let credit = Credit {
        artist: artist.id,
        role: CreditRole::Composer,
    };
    roundtrip(&credit);
}

#[test]
fn media_and_device_models_round_trip() {
    let media = MediaItem {
        id: MediaItemId::generate(),
        album: Some(AlbumId::generate()),
        format: AudioFormat {
            sample_rate_hz: 44_100,
            bit_depth: 16,
            channels: 2,
        },
        size_bytes: 34_000_000,
    };
    roundtrip(&media);

    let device = Device {
        id: DeviceId::generate(),
        name: "Living Room DAC".to_owned(),
        capabilities: DeviceCapabilities {
            capabilities: vec![
                DeviceCapability::BitPerfectExclusive,
                DeviceCapability::Synchronizable,
            ],
            supported_formats: vec![AudioFormat {
                sample_rate_hz: 192_000,
                bit_depth: 24,
                channels: 2,
            }],
        },
        online: true,
    };
    roundtrip(&device);

    let zone = Zone {
        id: ZoneId::generate(),
        name: "Living Room".to_owned(),
        devices: vec![device.id],
        grouped: false,
    };
    roundtrip(&zone);
}

#[test]
fn playback_state_round_trips_with_a_signal_path() {
    let state = PlaybackState {
        zone: ZoneId::generate(),
        transport: TransportState::Playing,
        queue: PlaybackQueue {
            items: vec![QueueItem {
                recording: RecordingId::generate(),
                media_item: Some(MediaItemId::generate()),
            }],
            current_index: Some(0),
            repeat: RepeatMode::Off,
            shuffle: false,
        },
        position_ms: 1_000,
        signal_path: Some(SignalPath {
            steps: vec![SignalPathStep {
                kind: "decode".to_owned(),
                description: "FLAC to 16-bit PCM".to_owned(),
                input: AudioFormat {
                    sample_rate_hz: 44_100,
                    bit_depth: 16,
                    channels: 2,
                },
                output: AudioFormat {
                    sample_rate_hz: 44_100,
                    bit_depth: 16,
                    channels: 2,
                },
            }],
        }),
    };
    roundtrip(&state);
}

#[test]
fn an_empty_signal_path_is_bit_perfect() {
    assert!(SignalPath::default().is_bit_perfect());
}

#[test]
fn job_status_round_trips_and_reports_terminal_state() {
    let job = JobStatus {
        id: JobId::generate(),
        kind: "library_scan".to_owned(),
        state: JobState::Succeeded,
        progress: JobProgress {
            completed: 1_204,
            total: Some(1_204),
        },
        message: Some("scan complete".to_owned()),
        error: None,
    };
    let decoded = roundtrip(&job);
    assert!(decoded.state.is_terminal());
    assert_eq!(decoded.progress.completed, 1_204);

    assert!(!JobState::Running.is_terminal());
    assert!(!JobState::Queued.is_terminal());
}
