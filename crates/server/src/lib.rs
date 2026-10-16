//! The application server: API, configuration, orchestration, and authentication.
//!
//! Phase 0 skeleton. The HTTP entry point, configuration loader, and health
//! endpoints land with the [Server Roadmap](../../Server%20Roadmap.md). This
//! crate declares the contract version it serves so the version is owned in one
//! place from the start.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Server;

/// The API contract version this server implements.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;
