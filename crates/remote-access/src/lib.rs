//! Pairing, secure connectivity, and remote sessions.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::RemoteAccess;

/// API contract version used by remote clients.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;
