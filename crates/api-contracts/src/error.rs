//! The API error envelope.

use serde::{Deserialize, Serialize};
use timbre_shared_types::{DiagnosticContext, Error, ErrorCode};

/// The body returned when an API request fails.
///
/// The shape is stable: clients branch on [`code`](Self::code) and never parse
/// [`message`](Self::message).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    /// The stable error code clients branch on.
    pub code: ErrorCode,
    /// A human-readable description.
    pub message: String,
    /// Trace context for support, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<DiagnosticContext>,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self {
            code: error.code,
            message: error.message,
            context: None,
        }
    }
}
