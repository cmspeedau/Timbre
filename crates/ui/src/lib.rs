//! Shared client user interface contracts.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Ui;

/// API contract version used by clients.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;
