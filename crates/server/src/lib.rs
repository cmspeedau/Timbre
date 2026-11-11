//! The application server: versioned HTTP API and library orchestration.
//!
//! The router exposes the core library endpoints and can be mounted by an
//! application with [`router`]. The current catalog adapter is in-memory;
//! persistent storage and server startup configuration are separate milestones.

pub mod api;
pub mod state;

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Server;

/// The API contract version this server implements.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;

/// Builds the server router with the supplied application state.
pub fn router(state: state::AppState) -> axum::Router {
    api::router(state)
}
