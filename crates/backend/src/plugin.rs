//! Steam library lookup and embedded BepInEx plugin installer (`TacticalBibleBridge.dll`).
//!
//! Preserves exact compatibility with `bridge/dist/TacticalBibleBridge.dll`,
//! `BepInEx/plugins/TacticalBibleBridge/TacticalBibleBridge.dll`, and `PLUGIN_VERSION`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Embedded pre-compiled C# BepInEx plugin DLL (built on Windows + FM27 interop).
pub const PLUGIN_DLL: &[u8] = include_bytes!("../../../bridge/dist/TacticalBibleBridge.dll");

/// Must stay in sync with `Plugin.Version` in `bridge/TacticalBibleBridge/Plugin.cs`.
pub const PLUGIN_VERSION: &str = "0.4.2";

/// Installation status of the BepInEx bridge plugin inside FM27.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginStatus {
    pub fm_dir: Option<String>,
    pub bepinex: bool,
    pub installed: bool,
    pub up_to_date: bool,
    pub version: String,
}

/// Parses `"path"` entries from a Steam `steamapps/libraryfolders.vdf` file.
pub fn parse_vdf_library_paths(vdf_content: &str) -> Vec<PathBuf> {
    let mut libs = Vec::new();
    for line in vdf_content.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("\"path\"") {
            let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
            if !p.is_empty() {
                libs.push(PathBuf::from(p));
            }
        }
    }
    libs
}

/// Discovers Steam library folders from standard Windows/Steam locations and `libraryfolders.vdf`.
pub fn steam_libraries() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for v in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(p) = std::env::var(v) {
            roots.push(PathBuf::from(p).join("Steam"));
        }
    }
    roots.push(PathBuf::from("C:\\Steam"));

    let mut libs: Vec<PathBuf> = Vec::new();
    for r in roots {
        if let Ok(t) = fs::read_to_string(r.join("steamapps").join("libraryfolders.vdf")) {
            libs.extend(parse_vdf_library_paths(&t));
        }
        if r.is_dir() {
            libs.push(r);
        }
    }
    libs
}

/// Locates the Football Manager 27 installation directory (custom path or Steam libraries).
pub fn fm_dir(custom: Option<&str>) -> Option<PathBuf> {
    if let Some(c) = custom.map(str::trim).filter(|c| !c.is_empty()) {
        let p = PathBuf::from(c);
        if p.join("fm.exe").is_file() || p.join("BepInEx").is_dir() {
            return Some(p);
        }
    }
    steam_libraries()
        .into_iter()
        .map(|lib| {
            lib.join("steamapps")
                .join("common")
                .join("Football Manager 27")
        })
        .find(|p| p.is_dir())
}

/// Exact plugin destination path inside FM27's BepInEx tree.
/// Kept as `TacticalBibleBridge` to prevent duplicate DLL / port 27029 Harmony conflicts.
pub fn plugin_path(fm: &Path) -> PathBuf {
    fm.join("BepInEx")
        .join("plugins")
        .join("TacticalBibleBridge")
        .join("TacticalBibleBridge.dll")
}

/// Inspects BepInEx + installed DLL byte equality for a given FM27 directory.
pub fn plugin_status_of(fm: Option<PathBuf>) -> PluginStatus {
    let bepinex = fm
        .as_ref()
        .map(|d| d.join("BepInEx").join("core").is_dir())
        .unwrap_or(false);
    let current = fm.as_ref().and_then(|d| fs::read(plugin_path(d)).ok());
    PluginStatus {
        fm_dir: fm.as_ref().map(|d| d.to_string_lossy().into_owned()),
        bepinex,
        installed: current.is_some(),
        up_to_date: current.as_deref() == Some(PLUGIN_DLL),
        version: PLUGIN_VERSION.to_string(),
    }
}

/// Checks bridge plugin installation status (auto-detecting Steam FM27 unless `custom_dir` is given).
pub fn plugin_status(custom_dir: Option<&str>) -> PluginStatus {
    plugin_status_of(fm_dir(custom_dir))
}

/// Installs or updates the embedded `TacticalBibleBridge.dll` into `BepInEx/plugins/TacticalBibleBridge/`.
/// FM27 must be closed on Windows (otherwise the OS locks the DLL).
pub fn plugin_install(custom_dir: Option<&str>) -> Result<PluginStatus, String> {
    let fm = fm_dir(custom_dir).ok_or_else(|| "Não encontrei a pasta do FM27".to_string())?;
    if !fm.join("BepInEx").join("core").is_dir() {
        return Err("O BepInEx não está instalado no FM27".into());
    }
    let dest = plugin_path(&fm);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&dest, PLUGIN_DLL).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) {
            "Feche o FM27 e tente outra vez (o plugin está em uso)".to_string()
        } else {
            e.to_string()
        }
    })?;
    Ok(plugin_status_of(Some(fm)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "fm27_backend_plugin_test_{}_{}_{}",
            label,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_vdf_extracts_windows_and_unix_paths() {
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"      "C:\\Program Files (x86)\\Steam"
        "label"     ""
    }
    "1"
    {
        "path"      "D:\\SteamLibrary"
    }
}
"#;
        let paths = parse_vdf_library_paths(vdf);
        assert_eq!(
            paths,
            vec![
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"D:\SteamLibrary"),
            ]
        );
    }

    #[test]
    fn embedded_dll_is_present_and_installs_cleanly_into_temp_dir() {
        assert!(PLUGIN_DLL.len() > 1000, "embedded DLL should be non-empty");
        let fake_fm = unique_temp_dir("install");
        fs::create_dir_all(fake_fm.join("BepInEx").join("core")).unwrap();
        let dir_str = fake_fm.to_string_lossy().into_owned();

        let before = plugin_status(Some(&dir_str));
        assert!(before.bepinex);
        assert!(!before.installed);
        assert!(!before.up_to_date);
        assert_eq!(before.version, PLUGIN_VERSION);

        let after = plugin_install(Some(&dir_str)).expect("install should succeed");
        assert!(after.installed);
        assert!(after.up_to_date);
        assert!(plugin_path(&fake_fm).is_file());

        let _ = fs::remove_dir_all(&fake_fm);
    }
}
