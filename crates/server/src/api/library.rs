//! Configured-root, scan-job, and catalog browse endpoints.

use std::path::{Path, PathBuf};

use axum::extract::{Path as PathParam, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use timbre_api_contracts::{Album, ApiError, Artist, Recording};
use timbre_shared_types::{AlbumId, ArtistId, ErrorCode, JobId, JobState, JobStatus};

use crate::state::{AppState, queued_scan, snapshot};

/// A configured, canonicalized library scan root.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryRoot {
    /// Opaque root identifier.
    pub id: String,
    /// Canonical filesystem directory approved for scanning.
    pub path: String,
}

/// Request to add a local library root.
#[derive(Clone, Debug, Deserialize)]
pub struct AddRootRequest {
    /// Directory path to configure. It must exist and be a directory.
    pub path: String,
}

/// Request to start a background scan of a configured root.
#[derive(Clone, Debug, Deserialize)]
pub struct ScanRequest {
    /// Identifier returned by `POST /library/roots`.
    pub root_id: String,
}

/// Paginated collection envelope returned by catalog endpoints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    /// Results for this page.
    pub items: Vec<T>,
    /// Zero-based item offset.
    pub offset: usize,
    /// Maximum number of items requested.
    pub limit: usize,
    /// Total number of matching items.
    pub total: usize,
}

/// Track browse row linking its shared recording and performer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackView {
    /// Recording metadata.
    pub recording: Recording,
    /// Primary artist identity.
    pub artist: ArtistId,
}

/// Pagination and optional catalog filters.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct BrowseQuery {
    /// Zero-based result offset (defaults to 0).
    pub offset: Option<usize>,
    /// Page size (defaults to 100; maximum 500).
    pub limit: Option<usize>,
    /// Optional artist filter for albums and tracks.
    pub artist_id: Option<ArtistId>,
    /// Optional album filter for tracks.
    pub album_id: Option<AlbumId>,
}

#[derive(Serialize)]
struct ScanAccepted {
    job: JobStatus,
}

#[derive(Debug)]
struct HttpError {
    status: StatusCode,
    error: ApiError,
}

impl HttpError {
    fn new(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            status,
            error: ApiError {
                code,
                message: message.into(),
                context: None,
            },
        }
    }

    fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::Internal,
            "library state is unavailable",
        )
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.error)).into_response()
    }
}

/// Builds library endpoints mounted beneath the supplied API prefix.
pub(crate) fn routes(prefix: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{prefix}/library/roots"),
            post(add_root).get(list_roots),
        )
        .route(&format!("{prefix}/library/scan"), post(start_scan))
        .route(&format!("{prefix}/library/artists"), get(list_artists))
        .route(&format!("{prefix}/library/albums"), get(list_albums))
        .route(&format!("{prefix}/library/tracks"), get(list_tracks))
        .route(&format!("{prefix}/library/tracks/{{id}}"), get(get_track))
        .route(&format!("{prefix}/library/albums/{{id}}"), get(get_album))
        .route(&format!("{prefix}/library/artists/{{id}}"), get(get_artist))
        .route(&format!("{prefix}/jobs/{{id}}"), get(get_job))
}

async fn add_root(
    State(state): State<AppState>,
    Json(request): Json<AddRootRequest>,
) -> Result<impl IntoResponse, HttpError> {
    if request.path.trim().is_empty() {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "path must not be empty",
        ));
    }
    let canonical = std::fs::canonicalize(Path::new(&request.path)).map_err(|_| {
        HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "path must be an accessible directory",
        )
    })?;
    if !canonical.is_dir() {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "path must be a directory",
        ));
    }
    let path = canonical.to_string_lossy().into_owned();
    let id = uuid::Uuid::new_v4().to_string();
    let root = LibraryRoot {
        id: id.clone(),
        path,
    };
    let mut store = state.library.write().map_err(|_| HttpError::internal())?;
    if store
        .roots
        .values()
        .any(|existing| existing.path == root.path)
    {
        return Err(HttpError::new(
            StatusCode::CONFLICT,
            ErrorCode::Conflict,
            "library root is already configured",
        ));
    }
    store.roots.insert(id, root.clone());
    Ok((StatusCode::CREATED, Json(root)))
}

async fn list_roots(State(state): State<AppState>) -> Result<Json<Vec<LibraryRoot>>, HttpError> {
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    Ok(Json(data.roots))
}

async fn start_scan(
    State(state): State<AppState>,
    Json(request): Json<ScanRequest>,
) -> Result<impl IntoResponse, HttpError> {
    let root = {
        let mut store = state.library.write().map_err(|_| HttpError::internal())?;
        let root = store.roots.get(&request.root_id).cloned().ok_or_else(|| {
            HttpError::new(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "configured library root was not found",
            )
        })?;
        let job = queued_scan();
        let id = job.id;
        store.jobs.insert(id, job.clone());
        (root, job)
    };
    let (root, job) = root;
    let job_id = job.id;
    let shared = state.clone();
    tokio::task::spawn_blocking(move || run_scan(shared, PathBuf::from(root.path), job_id));
    Ok((StatusCode::ACCEPTED, Json(ScanAccepted { job })))
}

fn run_scan(state: AppState, root: PathBuf, job_id: JobId) {
    if let Ok(mut store) = state.library.write() {
        if let Some(job) = store.jobs.get_mut(&job_id) {
            job.state = JobState::Running;
        }
    }
    let result = timbre_library::scan_directory(&root);
    if let Ok(mut store) = state.library.write() {
        match result {
            Ok(mut report) => {
                report.files.sort_by(|a, b| a.path.cmp(&b.path));
                let completed = report.files.len() as u64;
                for file in report.files {
                    let artist = store.upsert_artist(&file.metadata.artist);
                    let album = file
                        .metadata
                        .album
                        .as_deref()
                        .map(|title| store.upsert_album(title, Some(artist)));
                    store.upsert_track(
                        file.path,
                        file.metadata.title,
                        album,
                        file.metadata.track_number,
                        file.metadata.duration_ms,
                        artist,
                    );
                }
                let issues = report.issues.len();
                if let Some(job) = store.jobs.get_mut(&job_id) {
                    job.state = JobState::Succeeded;
                    job.progress.completed = completed;
                    job.progress.total = Some(completed);
                    job.message = Some(format!("scan completed with {issues} issue(s)"));
                }
            }
            Err(error) => {
                if let Some(job) = store.jobs.get_mut(&job_id) {
                    job.state = JobState::Failed;
                    job.message = Some(error.to_string());
                    job.error = Some(timbre_shared_types::Error::new(
                        ErrorCode::Unavailable,
                        "library root could not be scanned",
                    ));
                }
            }
        }
    }
}

async fn get_track(
    State(state): State<AppState>,
    PathParam(id): PathParam<String>,
) -> Result<Json<TrackView>, HttpError> {
    let id = id
        .parse::<timbre_shared_types::RecordingId>()
        .map_err(|_| {
            HttpError::new(
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidRequest,
                "recording id is invalid",
            )
        })?;
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    data.tracks
        .into_iter()
        .find(|track| track.recording.id == id)
        .map(Json)
        .ok_or_else(|| {
            HttpError::new(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "track was not found",
            )
        })
}

async fn get_album(
    State(state): State<AppState>,
    PathParam(id): PathParam<String>,
) -> Result<Json<Album>, HttpError> {
    let id = id.parse::<AlbumId>().map_err(|_| {
        HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "album id is invalid",
        )
    })?;
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    data.albums
        .into_iter()
        .find(|album| album.id == id)
        .map(Json)
        .ok_or_else(|| {
            HttpError::new(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "album was not found",
            )
        })
}

async fn get_artist(
    State(state): State<AppState>,
    PathParam(id): PathParam<String>,
) -> Result<Json<Artist>, HttpError> {
    let id = id.parse::<ArtistId>().map_err(|_| {
        HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "artist id is invalid",
        )
    })?;
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    data.artists
        .into_iter()
        .find(|artist| artist.id == id)
        .map(Json)
        .ok_or_else(|| {
            HttpError::new(
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                "artist was not found",
            )
        })
}

async fn get_job(
    State(state): State<AppState>,
    PathParam(id): PathParam<String>,
) -> Result<Json<JobStatus>, HttpError> {
    let id = id.parse::<JobId>().map_err(|_| {
        HttpError::new(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "job id is invalid",
        )
    })?;
    let store = state.library.read().map_err(|_| HttpError::internal())?;
    store.jobs.get(&id).cloned().map(Json).ok_or_else(|| {
        HttpError::new(
            StatusCode::NOT_FOUND,
            ErrorCode::NotFound,
            "job was not found",
        )
    })
}

async fn list_artists(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<Json<Page<Artist>>, HttpError> {
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    let mut artists = data.artists;
    artists.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(Json(page(artists, &query, |_| true)))
}

async fn list_albums(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<Json<Page<Album>>, HttpError> {
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    let mut albums = data.albums;
    albums.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(Json(page(albums, &query, |album| {
        query
            .artist_id
            .is_none_or(|artist| album.artist == Some(artist))
    })))
}

async fn list_tracks(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<Json<Page<TrackView>>, HttpError> {
    let data = snapshot(&state).map_err(|()| HttpError::internal())?;
    let mut tracks = data.tracks;
    tracks.sort_by(|a, b| {
        a.recording
            .title
            .to_lowercase()
            .cmp(&b.recording.title.to_lowercase())
            .then_with(|| a.recording.id.cmp(&b.recording.id))
    });
    Ok(Json(page(tracks, &query, |track| {
        query.artist_id.is_none_or(|artist| track.artist == artist)
            && query
                .album_id
                .is_none_or(|album| track.recording.album == Some(album))
    })))
}

fn page<T>(mut items: Vec<T>, query: &BrowseQuery, filter: impl Fn(&T) -> bool) -> Page<T> {
    items.retain(filter);
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let total = items.len();
    Page {
        items: items.into_iter().skip(offset).take(limit).collect(),
        offset,
        limit,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_bounds_limit_and_reports_total() {
        let values = vec![1, 2, 3, 4];
        let result = page(
            values,
            &BrowseQuery {
                offset: Some(1),
                limit: Some(1000),
                artist_id: None,
                album_id: None,
            },
            |_| true,
        );
        assert_eq!(result.items, vec![2, 3, 4]);
        assert_eq!(result.offset, 1);
        assert_eq!(result.limit, 500);
        assert_eq!(result.total, 4);
    }

    #[test]
    fn repeated_file_upserts_keep_recording_identity() {
        let mut store = crate::state::LibraryStore::default();
        let artist = store.upsert_artist("Example Artist");
        let path = PathBuf::from("/library/track.wav");
        store.upsert_track(
            path.clone(),
            "First title".to_owned(),
            None,
            None,
            None,
            artist,
        );
        let first_id = store.tracks["/library/track.wav"].recording.id;
        store.upsert_track(path, "Updated title".to_owned(), None, None, None, artist);
        let updated = &store.tracks["/library/track.wav"].recording;
        assert_eq!(updated.id, first_id);
        assert_eq!(updated.title, "Updated title");
    }
}
