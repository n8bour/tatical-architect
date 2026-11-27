//! Central persisted & runtime application state for FM27 Tactical Architect.

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;
use ta_backend::{
    bridge_call, import_legacy_file, latest_save, plugin_status, LegacyBackup, PluginStatus,
    SaveInfo, StateStore,
};
use ta_core::data::{default_role, formation_slots, slot_info, template_by_id, TEMPLATES};
use ta_core::types::{
    AppBackup, AutoSavePrefs, BridgeError, BridgeStatus, BridgeTransport, Lang, PersistedState,
    Phase, Player, ScoutDb, ShortlistEntry, SlotAssignment, SquadPack, Tactic,
};

/// `FmbClient` transport over the backend's TCP bridge (`127.0.0.1:27029`).
pub struct BackendBridgeTransport;

impl BridgeTransport for BackendBridgeTransport {
    fn call(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, BridgeError> {
        bridge_call(cmd, Some(timeout_ms)).map_err(BridgeError::Transport)
    }
}

// Global keyboard actions bound at app startup.
gpui_kit::actions!(ta_app, [ToggleSidebar, ReadSaveNow]);

/// Active primary sidebar section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavSection {
    Home,
    Team,
    Scouting,
    Tactic,
    Training,
    Settings,
}

impl NavSection {
    pub const ALL: [NavSection; 6] = [
        NavSection::Home,
        NavSection::Team,
        NavSection::Scouting,
        NavSection::Tactic,
        NavSection::Training,
        NavSection::Settings,
    ];

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            NavSection::Home => lang.pick("Início", "Home"),
            NavSection::Team => lang.pick("O Meu Plantel", "My Team"),
            NavSection::Scouting => lang.pick("Prospeção", "Scouting"),
            NavSection::Tactic => lang.pick("Tática", "Tactic"),
            NavSection::Training => lang.pick("Treino", "Training"),
            NavSection::Settings => lang.pick("Definições", "Settings"),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            NavSection::Home => "⌂",
            NavSection::Team => "◎",
            NavSection::Scouting => "⌕",
            NavSection::Tactic => "◈",
            NavSection::Training => "⚡",
            NavSection::Settings => "⚙",
        }
    }
}

/// Sub-tabs under My Team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamTab {
    Squad,
    BestXi,
    Generator,
    Opponent,
}

impl TeamTab {
    pub const ALL: [TeamTab; 4] = [
        TeamTab::Squad,
        TeamTab::BestXi,
        TeamTab::Generator,
        TeamTab::Opponent,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    pub fn from_index(ix: usize) -> Self {
        Self::ALL.get(ix).copied().unwrap_or(TeamTab::Squad)
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            TeamTab::Squad => lang.pick("Plantel", "Squad"),
            TeamTab::BestXi => lang.pick("Melhor XI", "Best XI"),
            TeamTab::Generator => lang.pick("Gerador", "Generator"),
            TeamTab::Opponent => lang.pick("Adversário", "Opponent"),
        }
    }
}

/// Sub-tabs under Tactic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TacticTab {
    Analyse,
    AnimatedPitch,
    GameStates,
    SetPieces,
    Fm27Sheet,
    Import,
}

impl TacticTab {
    pub const ALL: [TacticTab; 6] = [
        TacticTab::Analyse,
        TacticTab::AnimatedPitch,
        TacticTab::GameStates,
        TacticTab::SetPieces,
        TacticTab::Fm27Sheet,
        TacticTab::Import,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    pub fn from_index(ix: usize) -> Self {
        Self::ALL.get(ix).copied().unwrap_or(TacticTab::Analyse)
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            TacticTab::Analyse => lang.pick("Analisar", "Analyse"),
            TacticTab::AnimatedPitch => lang.pick("Campo Animado", "Animated Pitch"),
            TacticTab::GameStates => lang.pick("Situações de Jogo", "Game States"),
            TacticTab::SetPieces => lang.pick("Bolas Paradas", "Set Pieces"),
            TacticTab::Fm27Sheet => lang.pick("Ficha FM27", "FM27 Sheet"),
            TacticTab::Import => lang.pick("Importar", "Import"),
        }
    }
}

/// Active background operation shown in the non-obscuring top bar & activity card.
#[derive(Debug, Clone)]
pub struct BusyState {
    pub label: String,
    pub started_at: Instant,
    pub pct: Option<f32>,
}

impl BusyState {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            started_at: Instant::now(),
            pct: None,
        }
    }

    pub fn with_pct(mut self, pct: f32) -> Self {
        self.pct = Some(pct.clamp(0.0, 1.0));
        self
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.started_at.elapsed().as_secs_f32()
    }
}

/// Visual tone for transient toast messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warn,
    Error,
}

/// Transient notification toast.
#[derive(Debug, Clone)]
pub struct ToastState {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
}

impl ToastState {
    pub fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            created_at: Instant::now(),
        }
    }
}

/// On-disk persistence envelope combining legacy-compatible `PersistedState` with desktop UI prefs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StoredAppData {
    pub persisted: PersistedState,
    pub dark_mode: bool,
    pub sidebar_collapsed: bool,
    pub collapsed_panels: Vec<String>,
    pub shortlist_uids: Vec<i64>,
}

/// Builds a default valid 11-slot `Tactic` from the first built-in template.
pub fn default_tactic() -> Tactic {
    let tp = template_by_id("pos433").unwrap_or(&TEMPLATES[0]);
    let ip_slots = formation_slots(tp.ip).map_or(&[][..], |v| v.as_slice());
    let oop_slots = formation_slots(tp.oop)
        .map(|v| v.as_slice())
        .unwrap_or(ip_slots);
    let mut slots = Vec::with_capacity(11);
    for (i, &(ip_code, ip_role, oop_role)) in tp.roles.iter().enumerate() {
        let oop_code = oop_slots
            .get(i)
            .map(|s| s.code)
            .unwrap_or(ip_code);
        slots.push(SlotAssignment {
            ip_code: ip_code.to_string(),
            oop_code: oop_code.to_string(),
            ip: ip_role.to_string(),
            oop: oop_role.to_string(),
            pid: None,
            pis: Vec::new(),
        });
    }
    if slots.is_empty() {
        for (i, s) in ip_slots.iter().enumerate() {
            let oop_code = oop_slots.get(i).map(|o| o.code).unwrap_or(s.code);
            let meta = slot_info(tp.ip, s.code);
            let ip_r = meta
                .as_ref()
                .map(|m| default_role(m.group, Phase::Ip))
                .unwrap_or("gk");
            let oop_r = meta
                .as_ref()
                .map(|m| default_role(m.group, Phase::Oop))
                .unwrap_or("gko");
            slots.push(SlotAssignment {
                ip_code: s.code.to_string(),
                oop_code: oop_code.to_string(),
                ip: ip_r.to_string(),
                oop: oop_r.to_string(),
                pid: None,
                pis: Vec::new(),
            });
        }
    }
    let mut ti = BTreeMap::new();
    for &(k, v) in tp.ti {
        ti.insert(k.to_string(), v);
    }
    Tactic {
        name: tp.name_pt.to_string(),
        ment: tp.ment,
        ip_form: tp.ip.to_string(),
        oop_form: tp.oop.to_string(),
        slots,
        ti,
        ois: 0,
        source: "template".to_string(),
        saved_at: None,
    }
}

/// Central mutable state owned by the root GPUI view.
pub struct AppState {
    store: StateStore,

    // Preferences & UI layout
    pub lang: Lang,
    pub dark_mode: bool,
    pub sidebar_collapsed: bool,
    pub collapsed_panels: HashSet<String>,

    // Domain data
    pub tactic: Tactic,
    pub saved_tactics: Vec<Tactic>,
    pub squad: Vec<Player>,
    pub squad_source: String,
    pub opponent: Vec<Player>,
    pub opponent_source: String,
    pub my_club: String,
    /// FM27 sheet in PT: `false` keeps the in-game (English) names, `true` translates them.
    pub sheet_names_pt: bool,

    // Scouting DB & shortlist
    pub scout_db: Option<ScoutDb>,
    pub shortlist: HashSet<i64>,
    pub shortlist_entries: Vec<ShortlistEntry>,

    // Save / Bridge / OS integration state
    pub auto_save_poll: bool,
    pub custom_save_dir: String,
    pub custom_fm_dir: String,
    pub last_save_read: Option<String>,
    pub last_save_mtime: u64,
    pub last_save_info: Option<SaveInfo>,
    pub last_save_err: Option<String>,
    pub plugin_info: PluginStatus,
    pub bridge_status: BridgeStatus,

    // Activity & transient feedback
    pub busy: Option<BusyState>,
    pub toast: Option<ToastState>,
    pub last_dropped_files: Vec<PathBuf>,
}

impl AppState {
    /// Loads state from the default OS `StateStore`, falling back to clean defaults.
    pub fn load_default() -> Self {
        Self::load_from(StateStore::default_store())
    }

    /// Loads state from a specific `StateStore` (used in production and unit tests).
    pub fn load_from(store: StateStore) -> Self {
        let mut state = Self::fresh(store);
        if let Ok(Some(stored)) = state.store.load::<StoredAppData>() {
            state.apply_stored(stored);
        }
        // Quick local filesystem probe for initial status display (never mutates saves).
        let custom_save = non_empty(&state.custom_save_dir).map(str::to_string);
        let custom_fm = non_empty(&state.custom_fm_dir).map(str::to_string);
        state.last_save_info = latest_save(custom_save.as_deref());
        state.plugin_info = plugin_status(custom_fm.as_deref());
        state
    }

    fn fresh(store: StateStore) -> Self {
        Self {
            store,
            lang: Lang::Pt,
            dark_mode: true,
            sidebar_collapsed: false,
            collapsed_panels: HashSet::new(),
            tactic: default_tactic(),
            saved_tactics: Vec::new(),
            squad: Vec::new(),
            squad_source: String::new(),
            opponent: Vec::new(),
            opponent_source: String::new(),
            my_club: String::new(),
            sheet_names_pt: false,
            scout_db: None,
            shortlist: HashSet::new(),
            shortlist_entries: Vec::new(),
            auto_save_poll: false,
            custom_save_dir: String::new(),
            custom_fm_dir: String::new(),
            last_save_read: None,
            last_save_mtime: 0,
            last_save_info: None,
            last_save_err: None,
            plugin_info: PluginStatus {
                fm_dir: None,
                bepinex: false,
                installed: false,
                up_to_date: false,
                version: ta_backend::PLUGIN_VERSION.to_string(),
            },
            bridge_status: BridgeStatus::default(),
            busy: None,
            toast: None,
            last_dropped_files: Vec::new(),
        }
    }

    fn apply_stored(&mut self, stored: StoredAppData) {
        let p = stored.persisted;
        self.lang = p.lang;
        self.dark_mode = if p.theme == "light" {
            false
        } else if p.theme == "dark" {
            true
        } else {
            stored.dark_mode
        };
        self.sidebar_collapsed = stored.sidebar_collapsed;
        self.collapsed_panels = stored.collapsed_panels.into_iter().collect();

        if !p.tactic.slots.is_empty() {
            self.tactic = p.tactic;
        }
        self.saved_tactics = p.saved;
        self.squad = p.squad.players;
        self.squad_source = p.squad.source;
        self.opponent = p.opp.players;
        self.opponent_source = p.opp.source;
        self.my_club = p.my_club.unwrap_or_default();
        self.sheet_names_pt = p.sheet_names.as_deref() == Some("pt");
        self.shortlist_entries = p.shortlist;
        self.shortlist = stored.shortlist_uids.into_iter().collect();
        for entry in &self.shortlist_entries {
            if let Some(uid) = entry
                .uniq
                .as_deref()
                .or(Some(entry.id.as_str()))
                .and_then(|s| s.parse::<i64>().ok())
            {
                self.shortlist.insert(uid);
            }
        }

        self.auto_save_poll = p.auto.save;
        self.custom_save_dir = p.auto.save_dir;
        self.last_save_mtime = p.auto.save_mtime;
        self.last_save_read = non_empty(&p.auto.save_last).map(str::to_string);
        self.last_save_err = non_empty(&p.auto.save_err).map(str::to_string);
        self.custom_fm_dir = p.fm_dir.unwrap_or_default();
    }

    pub fn to_stored(&self) -> StoredAppData {
        let persisted = PersistedState {
            tactic: self.tactic.clone(),
            squad: SquadPack {
                players: self.squad.clone(),
                source: self.squad_source.clone(),
                ..SquadPack::default()
            },
            opp: SquadPack {
                players: self.opponent.clone(),
                source: self.opponent_source.clone(),
                ..SquadPack::default()
            },
            my_club: non_empty(&self.my_club).map(str::to_string),
            saved: self.saved_tactics.clone(),
            theme: if self.dark_mode {
                "dark".to_string()
            } else {
                "light".to_string()
            },
            auto: AutoSavePrefs {
                save: self.auto_save_poll,
                save_dir: self.custom_save_dir.clone(),
                save_mtime: self.last_save_mtime,
                save_err: self.last_save_err.clone().unwrap_or_default(),
                save_last: self.last_save_read.clone().unwrap_or_default(),
            },
            lang: self.lang,
            sheet_names: Some(if self.sheet_names_pt { "pt" } else { "en" }.to_string()),
            fm_dir: non_empty(&self.custom_fm_dir).map(str::to_string),
            plan_k: None,
            shortlist: self.shortlist_entries.clone(),
            sq_mode: None,
        };

        let mut collapsed_panels: Vec<String> = self.collapsed_panels.iter().cloned().collect();
        collapsed_panels.sort();
        let mut shortlist_uids: Vec<i64> = self.shortlist.iter().copied().collect();
        shortlist_uids.sort();

        StoredAppData {
            persisted,
            dark_mode: self.dark_mode,
            sidebar_collapsed: self.sidebar_collapsed,
            collapsed_panels,
            shortlist_uids,
        }
    }

    /// Persists the current state to disk. Logs failures into toast rather than panicking.
    pub fn persist(&mut self) {
        if let Err(err) = self.store.save(&self.to_stored()) {
            self.set_toast(
                format!(
                    "{}: {}",
                    self.lang
                        .pick("Erro ao guardar definições", "Failed to save state"),
                    err
                ),
                ToastKind::Error,
            );
        }
    }

    pub fn is_panel_collapsed(&self, id: &str) -> bool {
        self.collapsed_panels.contains(id)
    }

    pub fn toggle_panel(&mut self, id: &str) {
        if !self.collapsed_panels.remove(id) {
            self.collapsed_panels.insert(id.to_string());
        }
        self.persist();
    }

    pub fn set_busy(&mut self, label: impl Into<String>, pct: Option<f32>) {
        let mut b = BusyState::new(label);
        if let Some(p) = pct {
            b = b.with_pct(p);
        }
        self.busy = Some(b);
    }

    pub fn clear_busy(&mut self) {
        self.busy = None;
    }

    pub fn set_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toast = Some(ToastState::new(message, kind));
    }

    pub fn clear_toast(&mut self) {
        self.toast = None;
    }

    pub fn apply_theme(&self, window: Option<&mut Window>, cx: &mut App) {
        let mode = if self.dark_mode {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        Theme::change(mode, window, cx);
    }

    /// Builds a legacy-compatible `fm27tb-all` JSON backup envelope.
    pub fn export_backup_json(&self) -> Result<String, String> {
        let backup = AppBackup::All {
            v: 1,
            tactic: Some(self.tactic.clone()),
            squad: Some(SquadPack {
                players: self.squad.clone(),
                source: self.squad_source.clone(),
                ..SquadPack::default()
            }),
            opp: Some(SquadPack {
                players: self.opponent.clone(),
                source: self.opponent_source.clone(),
                ..SquadPack::default()
            }),
            saved: Some(self.saved_tactics.clone()),
            my_club: non_empty(&self.my_club).map(str::to_string),
        };
        serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())
    }

    /// Imports either our `StoredAppData`, an `AppBackup` (`fm27tb-all` / `fm27tb-tactic`),
    /// or a raw `fm27tb.v1` JSON snapshot from disk.
    pub fn import_backup_path(&mut self, path: &std::path::Path) -> Result<String, String> {
        let legacy = import_legacy_file(path)?;
        match legacy {
            LegacyBackup::SingleTactic(val) => {
                let tac: Tactic =
                    serde_json::from_value(val).map_err(|e| format!("Tática inválida: {}", e))?;
                let name = tac.name.clone();
                if !tac.slots.is_empty() {
                    self.tactic = tac;
                }
                self.persist();
                Ok(format!(
                    "{}: {}",
                    self.lang.pick("Tática importada", "Tactic imported"),
                    name
                ))
            }
            LegacyBackup::FullState(val) => {
                // Support both `AppBackup::All` and raw `PersistedState` (`fm27tb.v1`).
                if let Ok(AppBackup::All {
                    tactic,
                    squad,
                    opp,
                    saved,
                    my_club,
                    ..
                }) = serde_json::from_value::<AppBackup>(val.clone())
                {
                    if let Some(t) = tactic.filter(|t| !t.slots.is_empty()) {
                        self.tactic = t;
                    }
                    if let Some(s) = squad {
                        self.squad = s.players;
                        self.squad_source = s.source;
                    }
                    if let Some(o) = opp {
                        self.opponent = o.players;
                        self.opponent_source = o.source;
                    }
                    if let Some(sv) = saved {
                        self.saved_tactics = sv;
                    }
                    if let Some(c) = my_club {
                        self.my_club = c;
                    }
                    self.persist();
                    return Ok(self
                        .lang
                        .pick("Cópia completa restaurada.", "Full backup restored.")
                        .to_string());
                }

                let persisted: PersistedState = serde_json::from_value(val)
                    .map_err(|e| format!("Estado JSON inválido: {}", e))?;
                let dark = self.dark_mode;
                let sidebar = self.sidebar_collapsed;
                let collapsed: Vec<String> = self.collapsed_panels.iter().cloned().collect();
                let uids: Vec<i64> = self.shortlist.iter().copied().collect();
                self.apply_stored(StoredAppData {
                    persisted,
                    dark_mode: dark,
                    sidebar_collapsed: sidebar,
                    collapsed_panels: collapsed,
                    shortlist_uids: uids,
                });
                self.persist();
                Ok(self
                    .lang
                    .pick("Cópia fm27tb.v1 restaurada.", "Legacy fm27tb.v1 state restored.")
                    .to_string())
            }
        }
    }

    /// Wipes persisted data and resets state back to clean defaults (preserving current lang/theme).
    pub fn wipe_local_data(&mut self) {
        let lang = self.lang;
        let dark = self.dark_mode;
        let store = self.store.clone();
        let _ = std::fs::remove_file(store.path());
        *self = Self::fresh(store);
        self.lang = lang;
        self.dark_mode = dark;
        self.persist();
        self.set_toast(
            lang.pick("Dados locais apagados.", "Local data wiped."),
            ToastKind::Info,
        );
    }

    pub fn store_path_display(&self) -> String {
        self.store.path().display().to_string()
    }
}

pub fn non_empty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// Registers global keyboard shortcuts (`Cmd/Ctrl+B` collapse sidebar, `Cmd/Ctrl+Shift+L` read save).
pub fn bind_global_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("cmd-shift-l", ReadSaveNow, None),
        KeyBinding::new("ctrl-shift-l", ReadSaveNow, None),
    ]);
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use std::time::{SystemTime, UNIX_EPOCH};
    use ta_backend::StateStore;
    use ta_core::types::Lang;

    fn temp_store(label: &str) -> StateStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ta_app_state_{}_{}_{}",
            label,
            std::process::id(),
            nanos
        ));
        StateStore::new(dir.join("state.json"))
    }

    #[test]
    fn state_persists_and_imports_legacy_backup() {
        let store = temp_store("persist");
        let mut st = AppState::load_from(store.clone());
        assert_eq!(st.tactic.slots.len(), 11);

        st.lang = Lang::En;
        st.dark_mode = false;
        st.my_club = "SL Benfica".into();
        st.sidebar_collapsed = true;
        st.toggle_panel("settings-about");
        st.persist();

        let reloaded = AppState::load_from(store.clone());
        assert_eq!(reloaded.lang, Lang::En);
        assert!(!reloaded.dark_mode);
        assert_eq!(reloaded.my_club, "SL Benfica");
        assert!(reloaded.sidebar_collapsed);
        assert!(reloaded.is_panel_collapsed("settings-about"));

        // Export + import roundtrip
        let json = reloaded.export_backup_json().unwrap();
        let backup_path = store.path().with_file_name("backup.fm27tb.json");
        std::fs::write(&backup_path, json).unwrap();

        let mut st2 = AppState::load_from(temp_store("import"));
        let msg = st2.import_backup_path(&backup_path).unwrap();
        assert!(!msg.is_empty());
        assert_eq!(st2.my_club, "SL Benfica");

        let _ = std::fs::remove_dir_all(store.path().parent().unwrap());
    }
}
