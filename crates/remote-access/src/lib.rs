//! Remote access: pairing, secure connectivity, and remote sessions.
//!
//! Phase 0 skeleton. Pairing and authenticated connectivity land with the
//! [Remote Access Roadmap](../../Remote%20Access%20Roadmap.md). Remote access is
//! disabled unless it is configured.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::RemoteAccess;

/// The API contract version exposed over a remote session.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;
