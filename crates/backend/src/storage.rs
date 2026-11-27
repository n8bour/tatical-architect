//! Persistent JSON state store in the OS application data directory, plus
//! importer for legacy `.fm27tb.json` / `fm27tb.v1` JSON backups.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the OS-appropriate persistent config/state directory for FM27 Tactical Architect.
pub fn app_data_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return PathBuf::from(appdata).join("FM27TacticalArchitect");
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("FM27TacticalArchitect");
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                return PathBuf::from(xdg).join("fm27-tactical-architect");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return PathBuf::from(home)
                    .join(".config")
                    .join("fm27-tactical-architect");
            }
        }
    }
    std::env::temp_dir().join("fm27-tactical-architect")
}

/// Atomic JSON file store for persisted application state.
#[derive(Debug, Clone)]
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_store() -> Self {
        Self::new(app_data_dir().join("state.json"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_raw(&self) -> Result<Option<Value>, String> {
        if !self.path.is_file() {
            return Ok(None);
        }
        let text = fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        let val: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        Ok(Some(val))
    }

    pub fn save_raw(&self, val: &Value) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(val).map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &self.path).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e.to_string()
        })
    }

    pub fn load<T: DeserializeOwned>(&self) -> Result<Option<T>, String> {
        match self.load_raw()? {
            Some(val) => serde_json::from_value(val)
                .map(Some)
                .map_err(|e| e.to_string()),
            None => Ok(None),
        }
    }

    pub fn save<T: Serialize>(&self, data: &T) -> Result<(), String> {
        let val = serde_json::to_value(data).map_err(|e| e.to_string())?;
        self.save_raw(&val)
    }
}

/// Parsed representation of legacy `fm27tb.v1` / `.fm27tb.json` exports.
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyBackup {
    /// Full state backup (`kind: "fm27tb-all"` or raw `localStorage["fm27tb.v1"]` object).
    FullState(Value),
    /// Single exported tactic (`kind: "fm27tb-tactic"`).
    SingleTactic(Value),
}

/// Parses a legacy `.fm27tb.json` backup or raw `fm27tb.v1` JSON snapshot.
pub fn parse_legacy_backup(json_text: &str) -> Result<LegacyBackup, String> {
    let val: Value = serde_json::from_str(json_text).map_err(|e| format!("JSON inválido: {}", e))?;
    let obj = val
        .as_object()
        .ok_or_else(|| "Ficheiro de cópia deve ser um objeto JSON".to_string())?;

    if let Some(kind) = obj.get("kind").and_then(Value::as_str) {
        match kind {
            "fm27tb-tactic" => {
                let tactic = obj.get("tactic").cloned().unwrap_or_else(|| val.clone());
                return Ok(LegacyBackup::SingleTactic(tactic));
            }
            "fm27tb-all" => {
                let state = obj.get("state").cloned().unwrap_or_else(|| val.clone());
                return Ok(LegacyBackup::FullState(state));
            }
            other => {
                return Err(format!("Formato de cópia não suportado: {}", other));
            }
        }
    }

    // Raw localStorage `fm27tb.v1` blob heuristic: has at least one known top-level key.
    if obj.contains_key("tactic")
        || obj.contains_key("squad")
        || obj.contains_key("saved")
        || obj.contains_key("lang")
    {
        return Ok(LegacyBackup::FullState(val));
    }

    Err("JSON não reconhecido como cópia FM27 Tactical Architect / fm27tb.v1".into())
}

/// Reads and parses a legacy `.fm27tb.json` / `fm27tb.v1` file from disk.
pub fn import_legacy_file(path: &Path) -> Result<LegacyBackup, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse_legacy_backup(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SampleState {
        lang: String,
        dark: bool,
        saved_count: usize,
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("fm27_storage_{}_{}_{}", label, std::process::id(), nanos))
            .join("state.json")
    }

    #[test]
    fn json_storage_roundtrip() {
        let path = unique_temp_path("rt");
        let store = StateStore::new(path.clone());
        assert_eq!(store.load::<SampleState>().unwrap(), None);

        let initial = SampleState {
            lang: "pt".into(),
            dark: true,
            saved_count: 3,
        };
        store.save(&initial).unwrap();
        let loaded = store.load::<SampleState>().unwrap();
        assert_eq!(loaded, Some(initial));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn legacy_backup_parses_all_three_variants() {
        let all = r#"{"kind":"fm27tb-all","state":{"lang":"en","tactic":{"name":"4-3-3"}}}"#;
        match parse_legacy_backup(all).unwrap() {
            LegacyBackup::FullState(v) => assert_eq!(v["lang"], "en"),
            _ => panic!("expected FullState"),
        }

        let single = r#"{"kind":"fm27tb-tactic","tactic":{"name":"Gegenpress"}}"#;
        match parse_legacy_backup(single).unwrap() {
            LegacyBackup::SingleTactic(v) => assert_eq!(v["name"], "Gegenpress"),
            _ => panic!("expected SingleTactic"),
        }

        let raw_v1 = r#"{"lang":"pt","squad":[],"tactic":{"name":"4-2-3-1"}}"#;
        match parse_legacy_backup(raw_v1).unwrap() {
            LegacyBackup::FullState(v) => assert_eq!(v["tactic"]["name"], "4-2-3-1"),
            _ => panic!("expected FullState"),
        }

        assert!(parse_legacy_backup(r#"{"unrelated":1}"#).is_err());
    }
}
