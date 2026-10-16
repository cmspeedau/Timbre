//! Error codes and diagnostic context shared across subsystems.

use serde::{Deserialize, Serialize};

/// A stable, machine-readable error code.
///
/// Codes are part of the contract: clients may branch on them, so an existing
/// variant must not change meaning. Introduce a new variant instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The request was malformed or failed validation.
    InvalidRequest,
    /// The requested resource does not exist.
    NotFound,
    /// The request conflicts with current state.
    Conflict,
    /// Authentication is required, or it failed.
    Unauthorized,
    /// The caller is authenticated but not permitted.
    Forbidden,
    /// A dependency is temporarily unavailable.
    Unavailable,
    /// An unexpected internal failure occurred.
    Internal,
}

/// A domain error: a stable code plus human-readable detail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct Error {
    /// The stable error code.
    pub code: ErrorCode,
    /// A human-readable description, suitable for logs and diagnostics.
    pub message: String,
}

impl Error {
    /// Creates an error from a code and a message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Context attached to an error so a request or job can be traced end to end.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticContext {
    /// Correlates log lines for one request or job, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// The client-facing operation that produced the error, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}
