//! Background job status, shared by the server, the library, and clients.

use serde::{Deserialize, Serialize};

use crate::{Error, JobId};

/// The lifecycle state of a background job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    /// Accepted but not yet started.
    Queued,
    /// Currently running.
    Running,
    /// Finished successfully.
    Succeeded,
    /// Finished with an error.
    Failed,
    /// Cancelled before completion.
    Cancelled,
}

impl JobState {
    /// Returns `true` when the job has reached a terminal state.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// Progress of a running job.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobProgress {
    /// Items completed so far.
    pub completed: u64,
    /// Total items expected, when the job can estimate it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

/// The observable status of a background job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobStatus {
    /// Identifies the job.
    pub id: JobId,
    /// A short, stable job kind, for example `library_scan`.
    pub kind: String,
    /// The current lifecycle state.
    pub state: JobState,
    /// Progress within the job.
    pub progress: JobProgress,
    /// Optional human-readable detail, for example a failure reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The error that ended the job, when it failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}
