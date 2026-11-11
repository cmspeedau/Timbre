//! Playback transport controls and a server-sent event stream.
use crate::state::{AppState, PlaybackStore};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures_util::stream::Stream;
use std::time::{SystemTime, UNIX_EPOCH};
use timbre_api_contracts::{
    ApiError, EqualizerBand, EqualizerSettings, PlayRequest, PlaybackDevice, PlaybackDeviceRequest,
    PlaybackDeviceSelection, PlaybackDevices, PlaybackEvent, PlaybackMetricsResponse,
    PlaybackSnapshot, QueueAddRequest, QueueMoveRequest, QueueReorderRequest, RepeatMode,
    RepeatRequest, RuntimePlaybackQueue as PlaybackQueue, SeekRequest, ShuffleMode, ShuffleRequest,
    VolumeRequest,
};
use timbre_shared_types::ErrorCode;

#[derive(Debug)]
pub(super) struct HttpError(StatusCode, ApiError);
impl HttpError {
    fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self(
            status,
            ApiError {
                code,
                message: message.into(),
                context: None,
            },
        )
    }
}
impl axum::response::IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}
/// Routes under both the versioned and compatibility API prefixes.
pub(crate) fn routes(prefix: &str) -> Router<AppState> {
    Router::new()
        .route(&format!("{prefix}/playback/play"), post(play))
        .route(&format!("{prefix}/playback/pause"), post(pause))
        .route(&format!("{prefix}/playback/stop"), post(stop))
        .route(&format!("{prefix}/playback/next"), post(next))
        .route(&format!("{prefix}/playback/previous"), post(previous))
        .route(&format!("{prefix}/playback/pump"), post(pump))
        .route(&format!("{prefix}/playback/seek"), post(seek))
        .route(&format!("{prefix}/playback/volume"), post(volume))
        .route(
            &format!("{prefix}/playback/equalizer"),
            get(equalizer).post(set_equalizer),
        )
        .route(&format!("{prefix}/playback/metrics"), get(metrics))
        .route(&format!("{prefix}/playback/repeat"), post(repeat))
        .route(&format!("{prefix}/playback/shuffle"), post(shuffle))
        .route(&format!("{prefix}/playback/devices"), get(devices))
        .route(
            &format!("{prefix}/playback/device"),
            get(selected_device).post(select_device),
        )
        .route(&format!("{prefix}/playback/state"), get(state))
        .route(&format!("{prefix}/playback/events"), get(events))
        .route(
            &format!("{prefix}/playback/queue"),
            get(get_queue).post(add_queue_entry).delete(clear_queue),
        )
        .route(
            &format!("{prefix}/playback/queue/{{entry_id}}"),
            axum::routing::delete(remove_queue_entry),
        )
        .route(
            &format!("{prefix}/playback/queue/reorder"),
            post(reorder_queue),
        )
        .route(
            &format!("{prefix}/playback/queue/move"),
            post(move_queue_entry),
        )
}
fn event(store: &mut PlaybackStore, name: &str) -> PlaybackEvent {
    store.sequence = store.sequence.wrapping_add(1);
    PlaybackEvent {
        event: name.to_owned(),
        sequence: store.sequence,
        occurred_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        state: store.snapshot(),
    }
}
fn publish(state: &AppState, ev: PlaybackEvent) {
    let _ = state.playback_events.send(ev);
}
fn error(status: StatusCode, code: ErrorCode, text: &str) -> HttpError {
    HttpError::new(status, code, text)
}

async fn equalizer(State(state): State<AppState>) -> Result<Json<EqualizerSettings>, HttpError> {
    let p = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state unavailable",
        )
    })?;
    Ok(Json(EqualizerSettings {
        bands: p
            .engine
            .equalizer()
            .bands
            .iter()
            .map(|b| EqualizerBand {
                frequency: b.frequency,
                gain_db: b.gain_db,
                q: b.q,
                enabled: b.enabled,
            })
            .collect(),
        preamp_db: p.engine.equalizer().preamp_db,
    }))
}
async fn set_equalizer(
    State(state): State<AppState>,
    Json(settings): Json<EqualizerSettings>,
) -> Result<Json<EqualizerSettings>, HttpError> {
    if !settings.preamp_db.is_finite()
        || settings.preamp_db.abs() > 24.0
        || settings.bands.len() > 64
        || settings.bands.iter().any(|b| {
            !b.frequency.is_finite()
                || b.frequency <= 0.0
                || !b.q.is_finite()
                || b.q <= 0.0
                || !b.gain_db.is_finite()
                || b.gain_db.abs() > 24.0
        })
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "invalid equalizer settings",
        ));
    }
    let internal = timbre_audio_engine::EqSettings {
        bands: settings
            .bands
            .iter()
            .map(|b| timbre_audio_engine::EqBand {
                frequency: b.frequency,
                gain_db: b.gain_db,
                q: b.q,
                enabled: b.enabled,
            })
            .collect(),
        preamp_db: settings.preamp_db,
    };
    let json = serde_json::to_string(&settings).map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "EQ serialization failed",
        )
    })?;
    let mut p = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state unavailable",
        )
    })?;
    p.engine.set_equalizer(internal).map_err(|e| {
        error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            &e.to_string(),
        )
    })?;
    p.db_handle
        .lock()
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "database unavailable",
            )
        })?
        .set_equalizer_json(&json)
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "EQ persistence failed",
            )
        })?;
    let ev = event(&mut p, "playback.equalizer_changed");
    drop(p);
    publish(&state, ev);
    Ok(Json(settings))
}
async fn metrics(
    State(state): State<AppState>,
) -> Result<Json<PlaybackMetricsResponse>, HttpError> {
    let playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state unavailable",
        )
    })?;
    let format = playback.engine.audio_format();
    let sink = playback.engine.sink_metrics();
    let (rate, channels) = format
        .map(|f| (f.sample_rate, f.channels))
        .unwrap_or((0, 0));
    let channel_count = u64::from(channels.max(1));
    let capacity_frames = sink.capacity_samples / channel_count;
    let buffered_frames = sink.buffered_samples / channel_count;
    let buffered_duration_ms = if rate > 0 {
        buffered_frames.saturating_mul(1000) / u64::from(rate)
    } else {
        0
    };
    let completed = sink.frames_written.saturating_add(sink.frames_dropped);
    let drop_rate = if completed == 0 {
        0.0
    } else {
        sink.frames_dropped as f32 / completed as f32
    };
    Ok(Json(PlaybackMetricsResponse {
        underrun_count: sink.underrun_count,
        buffer_capacity_frames: capacity_frames,
        buffered_frames,
        buffered_duration_ms,
        output_sample_rate: rate,
        channel_count: channels,
        active_sink_id: Some(
            playback
                .selected_device_id
                .clone()
                .unwrap_or_else(|| "memory".into()),
        ),
        drop_rate,
        latency_estimate_ms: buffered_duration_ms,
    }))
}

async fn devices(State(state): State<AppState>) -> Json<PlaybackDevices> {
    let mut list: Vec<PlaybackDevice> = timbre_audio_engine::output_devices()
        .into_iter()
        .map(|d| PlaybackDevice {
            id: d.id,
            name: d.name,
            is_default: d.is_default,
            sample_rates: d.sample_rates,
            channels: d.channels,
        })
        .collect();
    if !state.hardware_audio {
        list = vec![PlaybackDevice {
            id: "memory".into(),
            name: "Memory / headless output".into(),
            is_default: true,
            sample_rates: vec![],
            channels: vec![],
        }];
    }
    Json(PlaybackDevices { devices: list })
}
async fn selected_device(
    State(state): State<AppState>,
) -> Result<Json<PlaybackDeviceSelection>, HttpError> {
    let playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    Ok(Json(PlaybackDeviceSelection {
        device_id: playback.selected_device_id.clone(),
    }))
}
async fn select_device(
    State(state): State<AppState>,
    Json(request): Json<PlaybackDeviceRequest>,
) -> Result<Json<PlaybackDeviceSelection>, HttpError> {
    let listed = timbre_audio_engine::output_devices();
    if let Some(id) = request.device_id.as_deref() {
        if !listed.iter().any(|d| d.id == id) || (!state.hardware_audio && id != "memory") {
            return Err(error(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                "unknown output device ID",
            ));
        }
    }
    let sink: std::sync::Arc<dyn timbre_audio_engine::AudioSink> =
        match request.device_id.as_deref() {
            Some("memory") => std::sync::Arc::new(timbre_audio_engine::MemoryAudioSink::new()),
            None if !state.hardware_audio => {
                std::sync::Arc::new(timbre_audio_engine::MemoryAudioSink::new())
            }
            Some(id) if state.hardware_audio => {
                #[cfg(feature = "hardware-audio")]
                {
                    std::sync::Arc::new(
                        timbre_audio_engine::CpalAudioSink::open_device(Some(id)).map_err(|e| {
                            error(
                                StatusCode::UNPROCESSABLE_ENTITY,
                                ErrorCode::Unavailable,
                                &format!("unable to open output device: {e}"),
                            )
                        })?,
                    )
                }
                #[cfg(not(feature = "hardware-audio"))]
                {
                    let _ = id;
                    return Err(error(
                        StatusCode::NOT_IMPLEMENTED,
                        ErrorCode::Unavailable,
                        "hardware audio support is disabled",
                    ));
                }
            }
            None => {
                #[cfg(feature = "hardware-audio")]
                {
                    std::sync::Arc::new(
                        timbre_audio_engine::CpalAudioSink::open_default().map_err(|e| {
                            error(
                                StatusCode::UNPROCESSABLE_ENTITY,
                                ErrorCode::Unavailable,
                                &format!("unable to open default output device: {e}"),
                            )
                        })?,
                    )
                }
                #[cfg(not(feature = "hardware-audio"))]
                {
                    std::sync::Arc::new(timbre_audio_engine::MemoryAudioSink::new())
                }
            }
            _ => {
                return Err(error(
                    StatusCode::BAD_REQUEST,
                    ErrorCode::InvalidRequest,
                    "unknown output device ID",
                ));
            }
        };
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.replace_sink(sink);
    playback.selected_device_id = request.device_id.clone();
    let mut db = playback.db_handle.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "database is unavailable",
        )
    })?;
    db.set_selected_audio_device(request.device_id.as_deref())
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "unable to persist output device",
            )
        })?;
    drop(db);
    let response = PlaybackDeviceSelection {
        device_id: playback.selected_device_id.clone(),
    };
    let ev = event(&mut playback, "playback.device_changed");
    drop(playback);
    publish(&state, ev);
    Ok(Json(response))
}

async fn play(
    State(state): State<AppState>,
    Json(request): Json<PlayRequest>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let path = {
        let store = state.library.read().map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "library state is unavailable",
            )
        })?;
        store
            .tracks
            .iter()
            .find(|(_, track)| track.recording.id == request.recording_id)
            .map(|(path, _)| path.clone())
            .ok_or_else(|| {
                error(
                    StatusCode::NOT_FOUND,
                    ErrorCode::NotFound,
                    "recording was not found",
                )
            })?
    };
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.load(path).map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::Unavailable,
            &format!("unable to load recording: {e}"),
        )
    })?;
    playback.recording_id = Some(request.recording_id);
    if let Some(index) = playback
        .queue
        .iter()
        .position(|entry| entry.recording_id == request.recording_id)
    {
        if playback.current_cursor != Some(index) {
            playback.set_queue_position(index);
        }
    }
    playback.engine.play();
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback_state");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn next(State(state): State<AppState>) -> Result<Json<PlaybackSnapshot>, HttpError> {
    transport_queue(&state, true).await
}
async fn previous(State(state): State<AppState>) -> Result<Json<PlaybackSnapshot>, HttpError> {
    transport_queue(&state, false).await
}

async fn transport_queue(
    state: &AppState,
    forward: bool,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    let index = if playback.current_cursor.is_none() && forward {
        if playback.shuffle == ShuffleMode::On {
            playback.ensure_shuffle_order();
            playback.shuffle_order.first().copied()
        } else {
            Some(0)
        }
    } else {
        playback.next_queue_index(forward)
    }
    .ok_or_else(|| {
        error(
            StatusCode::CONFLICT,
            ErrorCode::Conflict,
            "playback queue boundary reached",
        )
    })?;
    if forward {
        if let Some(current) = playback.current_cursor {
            playback.history.push(current);
        }
    }
    let recording_id = playback.queue[index].recording_id;
    let path = state
        .library
        .read()
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "library state is unavailable",
            )
        })?
        .tracks
        .iter()
        .find(|(_, t)| t.recording.id == recording_id)
        .map(|(p, _)| p.clone())
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "queued recording is not in the library",
            )
        })?;
    playback.engine.load(path).map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::Unavailable,
            &format!("unable to load recording: {e}"),
        )
    })?;
    playback.engine.play();
    playback.recording_id = Some(recording_id);
    playback.set_queue_position(index);
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "track_transition");
    drop(playback);
    publish(state, ev);
    Ok(Json(snapshot))
}

/// Drive one bounded decode step. Audio/device integrations should call this from their worker loop.
pub(super) async fn pump(
    State(state): State<AppState>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.pump(8).map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::Unavailable,
            &format!("playback pump failed: {e}"),
        )
    })?;
    if playback.engine.state() == timbre_audio_engine::PlaybackState::Playing {
        let next = if playback.repeat == RepeatMode::One {
            playback.current_cursor
        } else {
            playback.next_queue_index(true)
        };
        if let Some(index) = next {
            if let Some(entry) = playback.queue.get(index) {
                if let Some(path) = state.library.read().ok().and_then(|lib| {
                    lib.tracks
                        .iter()
                        .find(|(_, t)| t.recording.id == entry.recording_id)
                        .map(|(p, _)| p.clone())
                }) {
                    let _ = playback.engine.prepare(path);
                }
            }
        }
    }
    if playback.engine.state() == timbre_audio_engine::PlaybackState::TrackEnded {
        let next_index = if playback.repeat == RepeatMode::One {
            playback.current_cursor
        } else if playback.current_cursor.is_none() {
            None
        } else {
            playback.next_queue_index(true)
        };
        if playback.repeat == RepeatMode::One && next_index.is_none() {
            if let Some(recording_id) = playback.recording_id {
                let path = state
                    .library
                    .read()
                    .map_err(|_| {
                        error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            ErrorCode::Internal,
                            "library state is unavailable",
                        )
                    })?
                    .tracks
                    .iter()
                    .find(|(_, track)| track.recording.id == recording_id)
                    .map(|(path, _)| path.clone());
                if let Some(path) = path {
                    playback.engine.load(path).map_err(|e| {
                        error(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            ErrorCode::Unavailable,
                            &format!("unable to repeat recording: {e}"),
                        )
                    })?;
                    playback.engine.play();
                    let snapshot = playback.snapshot();
                    let ev = event(&mut playback, "queue_advanced");
                    drop(playback);
                    publish(&state, ev);
                    return Ok(Json(snapshot));
                }
            }
        }
        if let Some(index) = next_index {
            let recording_id = playback.queue[index].recording_id;
            let path = state
                .library
                .read()
                .map_err(|_| {
                    error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorCode::Internal,
                        "library state is unavailable",
                    )
                })?
                .tracks
                .iter()
                .find(|(_, track)| track.recording.id == recording_id)
                .map(|(path, _)| path.clone());
            if let Some(path) = path {
                if playback.current_cursor != Some(index) {
                    if let Some(old) = playback.current_cursor {
                        playback.history.push(old);
                    }
                }
                playback.engine.load(path).map_err(|e| {
                    error(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ErrorCode::Unavailable,
                        &format!("unable to load next recording: {e}"),
                    )
                })?;
                playback.engine.play();
                playback.recording_id = Some(recording_id);
                playback.set_queue_position(index);
                let snapshot = playback.snapshot();
                let ev = event(&mut playback, "queue_advanced");
                drop(playback);
                publish(&state, ev);
                return Ok(Json(snapshot));
            }
        }
    }
    Ok(Json(playback.snapshot()))
}

async fn pause(State(state): State<AppState>) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.pause();
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback_state");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn stop(State(state): State<AppState>) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.stop().map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::Unavailable,
            &format!("stop failed: {e}"),
        )
    })?;
    playback.recording_id = None;
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback_state");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn seek(
    State(state): State<AppState>,
    Json(request): Json<SeekRequest>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    let target_seconds = request.position_ms as f64 / 1000.0;
    if playback
        .engine
        .duration_seconds()
        .is_some_and(|duration| target_seconds > duration)
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::InvalidRequest,
            "seek position exceeds track duration",
        ));
    }
    playback.engine.seek(target_seconds).map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::InvalidRequest,
            &format!("seek failed: {e}"),
        )
    })?;
    playback.persist_queue();
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback.seeked");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn repeat(
    State(state): State<AppState>,
    Json(request): Json<RepeatRequest>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.set_repeat(request.mode);
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback.mode_changed");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn shuffle(
    State(state): State<AppState>,
    Json(request): Json<ShuffleRequest>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.set_shuffle(request.mode);
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback.mode_changed");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}

async fn volume(
    State(state): State<AppState>,
    Json(request): Json<VolumeRequest>,
) -> Result<Json<PlaybackSnapshot>, HttpError> {
    if !request.volume.is_finite() || !(0.0..=1.0).contains(&request.volume) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "volume must be finite and between 0 and 1",
        ));
    }
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    playback.engine.set_volume(request.volume);
    if let Some(muted) = request.muted {
        playback.engine.set_muted(muted);
    }
    playback.persist_queue();
    let snapshot = playback.snapshot();
    let ev = event(&mut playback, "playback_state");
    drop(playback);
    publish(&state, ev);
    Ok(Json(snapshot))
}
async fn state(State(state): State<AppState>) -> Result<Json<PlaybackSnapshot>, HttpError> {
    let playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    Ok(Json(playback.snapshot()))
}
/// Read the current ordered process-local playback queue.
async fn get_queue(State(state): State<AppState>) -> Result<Json<PlaybackQueue>, HttpError> {
    let playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    Ok(Json(playback.queue_snapshot()))
}

/// Append a recording occurrence; duplicate recordings are allowed.
async fn add_queue_entry(
    State(state): State<AppState>,
    request: Result<Json<QueueAddRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<PlaybackQueue>), HttpError> {
    let Json(request) = request.map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "queue request must contain a valid recording_id UUID",
        )
    })?;
    let exists = state
        .library
        .read()
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::Internal,
                "library state is unavailable",
            )
        })?
        .tracks
        .values()
        .any(|track| track.recording.id == request.recording_id);
    if !exists {
        return Err(error(
            StatusCode::NOT_FOUND,
            ErrorCode::NotFound,
            "recording was not found",
        ));
    }
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    let queue = playback.append_queue(request.recording_id);
    let ev = event(&mut playback, "queue.updated");
    drop(playback);
    publish(&state, ev);
    Ok((StatusCode::CREATED, Json(queue)))
}

/// Remove a queue occurrence by its unique entry ID.
async fn remove_queue_entry(
    State(state): State<AppState>,
    axum::extract::Path(entry_id): axum::extract::Path<String>,
) -> Result<Json<PlaybackQueue>, HttpError> {
    let entry_id = uuid::Uuid::parse_str(&entry_id).map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "queue entry ID is invalid",
        )
    })?;
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    if !playback.remove_queue_entry(entry_id) {
        return Err(error(
            StatusCode::NOT_FOUND,
            ErrorCode::NotFound,
            "queue entry was not found",
        ));
    }
    let queue = playback.queue_snapshot();
    let ev = event(&mut playback, "queue.updated");
    drop(playback);
    publish(&state, ev);
    Ok(Json(queue))
}

/// Remove all queued occurrences.
async fn reorder_queue(
    State(state): State<AppState>,
    Json(request): Json<QueueReorderRequest>,
) -> Result<Json<PlaybackQueue>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    if !playback.reorder_queue(&request.entry_ids) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "entry_ids must contain each queue entry exactly once",
        ));
    }
    let queue = playback.queue_snapshot();
    let ev = event(&mut playback, "queue.updated");
    drop(playback);
    publish(&state, ev);
    Ok(Json(queue))
}

/// Move an occurrence to its final zero-based queue position.
async fn move_queue_entry(
    State(state): State<AppState>,
    Json(request): Json<QueueMoveRequest>,
) -> Result<Json<PlaybackQueue>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    if request.position >= playback.queue.len()
        || !playback.move_queue_entry(request.entry_id, request.position)
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "entry_id must exist and position must be within the queue",
        ));
    }
    let queue = playback.queue_snapshot();
    let ev = event(&mut playback, "queue.updated");
    drop(playback);
    publish(&state, ev);
    Ok(Json(queue))
}

async fn clear_queue(State(state): State<AppState>) -> Result<Json<PlaybackQueue>, HttpError> {
    let mut playback = state.playback.lock().map_err(|_| {
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "playback state is unavailable",
        )
    })?;
    let queue = playback.clear_queue();
    let ev = event(&mut playback, "queue.updated");
    drop(playback);
    publish(&state, ev);
    Ok(Json(queue))
}

async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.playback_events.subscribe();
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(message) => {
                    let event_name = message.event.clone();
                    let data = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_owned());
                    return Some((Ok(Event::default().event(event_name).data(data)), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broadcast_event_serializes_contract() {
        let state = AppState::new();
        let mut rx = state.playback_events.subscribe();
        let ev = {
            let mut p = state.playback.lock().unwrap();
            event(&mut p, "playback_state")
        };
        publish(&state, ev);
        let delivered = rx.try_recv().unwrap();
        assert_eq!(delivered.sequence, 1);
        assert_eq!(delivered.state.volume, 1.0);
        assert!(
            serde_json::to_string(&delivered)
                .unwrap()
                .contains("playback_state")
        );
    }
    #[test]
    fn tracks_empty_engine_as_stopped() {
        let state = AppState::new();
        assert_eq!(
            state.playback.lock().unwrap().snapshot().state,
            timbre_api_contracts::PlaybackStatus::Stopped
        );
    }

    #[tokio::test]
    async fn volume_handler_validates_and_persists_gain_and_mute() {
        let state = AppState::new();
        let result = volume(
            State(state.clone()),
            Json(VolumeRequest {
                volume: 0.4,
                muted: Some(true),
            }),
        )
        .await
        .unwrap();
        assert_eq!(result.0.volume, 0.4);
        assert!(result.0.muted);
        let persisted = state
            .playback
            .lock()
            .unwrap()
            .db_handle
            .lock()
            .unwrap()
            .load()
            .unwrap();
        assert_eq!(persisted.volume, 0.4);
        assert!(persisted.muted);
        let invalid = volume(
            State(state),
            Json(VolumeRequest {
                volume: f32::NAN,
                muted: None,
            }),
        )
        .await;
        assert!(invalid.is_err());
    }

    #[tokio::test]
    async fn move_queue_handler_moves_occurrence_and_rejects_bad_target() {
        let state = AppState::new();
        let ids = {
            let mut p = state.playback.lock().unwrap();
            let r = timbre_shared_types::RecordingId::generate();
            let q = p.append_queue(r);
            p.append_queue(r);
            (q.entries[0].id, p.queue[1].id)
        };
        let moved = move_queue_entry(
            State(state.clone()),
            Json(QueueMoveRequest {
                entry_id: ids.0,
                position: 1,
            }),
        )
        .await
        .unwrap();
        assert_eq!(moved.0.entries[1].id, ids.0);
        let invalid = move_queue_entry(
            State(state),
            Json(QueueMoveRequest {
                entry_id: ids.1,
                position: 2,
            }),
        )
        .await;
        assert!(invalid.is_err());
    }

    #[tokio::test]
    async fn queue_handlers_validate_recordings_and_manage_occurrences() {
        use std::path::PathBuf;
        use timbre_api_contracts::Artist;
        use timbre_shared_types::{ArtistId, RecordingId};

        let state = AppState::new();
        let artist_id = ArtistId::generate();
        let recording_id = RecordingId::generate();
        {
            let mut library = state.library.write().unwrap();
            library.artists.insert(
                artist_id,
                Artist {
                    id: artist_id,
                    name: "fixture".into(),
                },
            );
            library.upsert_track(
                PathBuf::from("fixture.wav"),
                "fixture".into(),
                None,
                None,
                None,
                artist_id,
            );
            let stored = library.tracks.get_mut("fixture.wav").unwrap();
            stored.recording.id = recording_id;
        }
        let added = add_queue_entry(
            State(state.clone()),
            Ok(Json(QueueAddRequest { recording_id })),
        )
        .await
        .unwrap();
        assert_eq!(added.0, StatusCode::CREATED);
        let queue = added.1.0;
        assert_eq!(queue.entries.len(), 1);
        let duplicate = add_queue_entry(
            State(state.clone()),
            Ok(Json(QueueAddRequest { recording_id })),
        )
        .await
        .unwrap();
        assert_eq!(duplicate.1.0.entries.len(), 2);
        assert_ne!(duplicate.1.0.entries[0].id, duplicate.1.0.entries[1].id);
        let missing = add_queue_entry(
            State(state.clone()),
            Ok(Json(QueueAddRequest {
                recording_id: RecordingId::generate(),
            })),
        )
        .await;
        assert!(missing.is_err());
        let removed = remove_queue_entry(
            State(state.clone()),
            axum::extract::Path(queue.entries[0].id.to_string()),
        )
        .await
        .unwrap();
        assert_eq!(removed.0.entries.len(), 1);
        assert_eq!(removed.0.entries[0].recording_id, recording_id);
        assert_eq!(removed.0.revision, 3);
        let added = add_queue_entry(
            State(state.clone()),
            Ok(Json(QueueAddRequest { recording_id })),
        )
        .await
        .unwrap();
        assert_eq!(
            clear_queue(State(state.clone()))
                .await
                .unwrap()
                .0
                .entries
                .len(),
            0
        );
        assert_eq!(
            clear_queue(State(state.clone())).await.unwrap().0.revision,
            5
        );
        assert_eq!(added.1.0.entries.len(), 2);
        let missing_entry = remove_queue_entry(
            State(state.clone()),
            axum::extract::Path(uuid::Uuid::new_v4().to_string()),
        )
        .await;
        assert!(missing_entry.is_err());
        assert_eq!(get_queue(State(state.clone())).await.unwrap().0.revision, 5);
        let invalid_entry =
            remove_queue_entry(State(state), axum::extract::Path("not-a-uuid".into())).await;
        assert!(invalid_entry.is_err());
    }
}
