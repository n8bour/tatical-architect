//! Tauri-free native backend for FM27 Tactical Architect.
//!
//! Preserves all 7 native capabilities previously exposed via `src-tauri/src/main.rs`:
//! - FM27 `.fm` save discovery and `fmsave` export (managed squad + cached full scouting DB)
//! - Persistent TCP line-protocol client to the in-game BepInEx bridge (`127.0.0.1:27029`)
//! - Steam `libraryfolders.vdf` lookup & embedded `TacticalBibleBridge.dll` installer
//! - Persistent JSON state storage & legacy `fm27tb.v1` / `.fm27tb.json` backup import
//! - OS URL launcher (`https://` validated) and native file/folder dialogs

pub mod bridge;
pub mod os;
pub mod plugin;
pub mod save;
pub mod storage;

pub use bridge::{bridge_call, BridgeClient, BRIDGE_PORT};
pub use os::{open_path, open_url, pick_folder, pick_open_file, pick_save_file, validate_https_url};
pub use plugin::{
    fm_dir, parse_vdf_library_paths, plugin_install, plugin_path, plugin_status, PluginStatus,
    PLUGIN_VERSION,
};
pub use save::{export_save, export_save_db, latest_save, save_dirs, SaveDb, SaveExport, SaveInfo};
pub use storage::{
    app_data_dir, import_legacy_file, parse_legacy_backup, LegacyBackup, StateStore,
};
