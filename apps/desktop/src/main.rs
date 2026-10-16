//! The Timbre desktop client.
//!
//! Phase 0 skeleton: it reports the contract version it will render. The window
//! and Slint interface land with the [Client Roadmap](../../Client%20Roadmap.md).

fn main() {
    let name = env!("CARGO_PKG_NAME");
    let subsystem = timbre_ui::SUBSYSTEM;
    let api = timbre_api_contracts::API_VERSION;
    println!("{name} (subsystem {subsystem:?}) — API {api}");
}
