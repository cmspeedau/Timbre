//! The Timbre mobile client.
//!
//! Phase 0 skeleton. The real mobile application is cross-compiled for Android
//! and iOS and lands with the [Mobile Roadmap](../../Mobile%20Roadmap.md). This
//! host-buildable binary exists so the workspace builds on every platform and
//! the contract dependency is wired from the start.

fn main() {
    let name = env!("CARGO_PKG_NAME");
    let subsystem = timbre_ui::SUBSYSTEM;
    let api = timbre_api_contracts::API_VERSION;
    println!("{name} (subsystem {subsystem:?}) — API {api}");
}
