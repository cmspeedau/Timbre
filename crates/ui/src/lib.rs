//! Shared client user interface components for the desktop and mobile apps.
//!
//! Phase 0 skeleton. The Slint components land with the
//! [Client Roadmap](../../Client%20Roadmap.md); this crate declares the contract
//! version the interface renders so the dependency is explicit from the start.

use timbre_shared_types::Subsystem;

/// The subsystem this crate implements.
pub const SUBSYSTEM: Subsystem = Subsystem::Ui;

/// The API contract version the interface renders.
pub const API_VERSION: &str = timbre_api_contracts::API_VERSION;
