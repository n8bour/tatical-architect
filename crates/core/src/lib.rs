//! Core domain logic for FM27 Tactical Architect (`ta-core`).
//!
//! Zero-UI, pure-Rust tactical analysis, simulation, scouting, i18n, and FM27
//! bridge protocol crate.

pub mod data;
pub mod engine;
pub mod live;
pub mod planning;
pub mod types;

pub use types::Lang;

/// Shared application version synced with workspace Cargo.toml.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_toggle_and_pick() {
        assert_eq!(Lang::Pt.code(), "pt");
        assert_eq!(Lang::Pt.toggle(), Lang::En);
        assert_eq!(Lang::En.toggle(), Lang::Pt);
        assert_eq!(Lang::Pt.pick("Plantel", "Squad"), "Plantel");
        assert_eq!(Lang::En.pick("Plantel", "Squad"), "Squad");
    }
}
