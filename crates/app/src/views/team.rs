//! My Team screens:
//! - Squad (`render_team_squad`): CSV/HTML/JSON import, fmsave read, FM Bridge live refresh,
//!   search & position filter, 3 sortable table modes (Roles, Ability & Contract, Personality
//!   & Hidden), and clickable Player Profile panel (attributes, best IP/OOP roles, XI fit).
//! - Best XI (`render_team_best_xi`): slot suitability, better alt role, clickable recommended
//!   PIs, backup & U21 successor, and all systems ranked for the squad.
//! - Opponent (`render_team_opponent`): CSV/HTML import or FM Bridge next opponent, 9-sector
//!   comparison vs own XI, top 3 dangerous players + OIs, and 1-click game plan apply.

use crate::state::{BackendBridgeTransport, NavSection, TacticTab, TeamTab, ToastKind};
use crate::widgets::common::{
    attr_chip, attr_color, badge_pill, collapsible_card, empty_state_card, score_color, score_ring,
    suitability_bar, with_alpha, BadgeTone,
};
use crate::widgets::pitch_interactive::{interactive_pitch, PitchPhaseMode};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use ta_backend::pick_open_file;
use ta_core::data::{
    attr_name, l, mentality_name, pi_name, pt_terms, role_by_id, role_name, roles_for,
    slot_info, t, t_name, template_by_id, ti_by_key, ti_name, ti_opt_name, ti_val, TEMPLATES,
};
use ta_core::engine::hungarian::{auto_pick_xi, tactic_from_template};
use ta_core::engine::insights::{bench_and_successors, opponent_report, recommend_pis};
use ta_core::engine::parsers::{majority_club, parse_positions, parse_squad};
use ta_core::engine::scoring::{best_roles_for_player, role_score, slot_score, suggest_tactics};
use ta_core::live::fmbridge::FmbClient;
use ta_core::live::sim_model::sim_model;
use ta_core::types::{Attr, AttrCat, Lang, Phase, Player, PosGroup, Tactic};

// ============================================================================
// View-Local State (`TeamViewState`) — embed as `pub team_ui: TeamViewState`
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SquadTableMode {
    #[default]
    Roles,
    Ability,
    Personality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SquadSortCol {
    #[default]
    None,
    Name,
    Age,
    Ca,
    Pa,
    Value,
    Wage,
    ContractEnd,
    Pers(&'static str),
    Hid(&'static str),
}

/// View-local UI state for My Team screens (Squad table mode/sort/filter/profile modal).
/// Implements `Default` for zero-friction embedding on `ArchitectApp`.
#[derive(Default)]
pub struct TeamViewState {
    pub squad_mode: SquadTableMode,
    pub squad_sort: SquadSortCol,
    pub squad_sort_asc: bool,
    pub pos_filter: Option<PosGroup>,
    pub search_query: String,
    pub search_input: Option<Entity<InputState>>,
    pub selected_player_id: Option<String>,
    _subs: Vec<Subscription>,
}

impl TeamViewState {
    /// Initializes `TeamViewState` with a live GPUI search `InputState` bound to `search_query`.
    pub fn with_search_input(window: &mut Window, cx: &mut Context<ArchitectApp>) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Filter by player name or position…")
        });
        let sub = cx.subscribe(&search_input, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let val = entity.read(cx).value().to_string();
                if this.team_ui.search_query != val {
                    this.team_ui.search_query = val;
                    cx.notify();
                }
            }
        });
        Self {
            search_input: Some(search_input),
            _subs: vec![sub],
            ..Self::default()
        }
    }

    pub fn toggle_sort(&mut self, col: SquadSortCol) {
        if self.squad_sort == col {
            self.squad_sort_asc = !self.squad_sort_asc;
        } else {
            self.squad_sort = col;
            self.squad_sort_asc = matches!(
                col,
                SquadSortCol::Name
                    | SquadSortCol::Age
                    | SquadSortCol::Value
                    | SquadSortCol::Wage
                    | SquadSortCol::ContractEnd
            );
        }
    }
}

// ============================================================================
// Shared Squad/Tactic Mutation Helpers
// ============================================================================

/// Syncs `animated_pitch` with the current `state.tactic`, `state.squad`, and `state.lang`.
pub(crate) fn sync_animated_pitch(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let model = sim_model(&app.state.tactic, lang);
    let squad = app.state.squad.clone();
    app.animated_pitch.update(cx, |v, cx| {
        v.lang = lang;
        v.sync_squad(&squad);
        v.set_model(Some(model));
        cx.notify();
    });
}

/// Loads a tactic into `app.state.tactic`, persists, syncs the animated pitch, and optionally
/// switches to `Tactic › Analyse`.
pub(crate) fn apply_tactic_to_app(
    app: &mut ArchitectApp,
    tactic: Tactic,
    open_analyser: bool,
    cx: &mut Context<ArchitectApp>,
) {
    app.state.tactic = tactic;
    app.selected_slot = Some(0);
    if open_analyser {
        app.section = NavSection::Tactic;
        app.tactic_tab = TacticTab::Analyse;
    }
    sync_animated_pitch(app, cx);
    app.state.persist();
    cx.notify();
}

/// Parses CSV/HTML/TSV/pipe text into either `state.squad` or `state.opponent`.
pub(crate) fn ingest_squad_text(
    app: &mut ArchitectApp,
    text: &str,
    source_name: &str,
    force_opp: Option<bool>,
    cx: &mut Context<ArchitectApp>,
) {
    let lang = app.state.lang;
    let clean = text.trim_start_matches('\u{feff}');
    let mut parsed = parse_squad(clean, lang);
    if parsed.players.is_empty() {
        app.state.set_toast(
            format!(
                "{}{}",
                l(lang, "Não encontrei jogadores em ", "No players found in "),
                source_name
            ),
            ToastKind::Warn,
        );
        cx.notify();
        return;
    }

    for p in &mut parsed.players {
        if p.ca.is_none() {
            p.ca = p
                .meta
                .ca
                .as_deref()
                .and_then(|s| s.trim().parse::<f32>().ok());
        }
        if p.pa.is_none() {
            p.pa = p
                .meta
                .pa
                .as_deref()
                .and_then(|s| s.trim().parse::<f32>().ok());
        }
    }

    let mc = majority_club(&parsed.players);
    if force_opp.is_none() && mc.club.is_some() && mc.share < 0.6 {
        app.state.set_toast(
            l(
                lang,
                "Exportação com vários clubes ignorada para o plantel.",
                "Multi-club export ignored for squad.",
            ),
            ToastKind::Warn,
        );
        cx.notify();
        return;
    }

    let to_opp = match force_opp {
        Some(v) => v,
        None => {
            !app.state.my_club.is_empty()
                && mc
                    .club
                    .as_deref()
                    .map_or(false, |c| c != app.state.my_club.as_str())
        }
    };

    let count = parsed.players.len();
    if to_opp {
        app.state.opponent = parsed.players;
        app.state.opponent_source = source_name.to_string();
        let club_str = mc
            .club
            .as_deref()
            .map(|c| format!(" {}", c))
            .unwrap_or_default();
        app.state.set_toast(
            match lang {
                Lang::Pt => format!("Adversário{}: {} jogadores — relatório pronto.", club_str, count),
                Lang::En => format!("Opponent{}: {} players — report ready.", club_str, count),
            },
            ToastKind::Success,
        );
    } else {
        let ids: HashSet<String> = parsed.players.iter().map(|p| p.id.clone()).collect();
        app.state.squad = parsed.players;
        app.state.squad_source = source_name.to_string();
        if let Some(club) = mc.club.clone() {
            app.state.my_club = club;
        }
        for s in &mut app.state.tactic.slots {
            if let Some(pid) = &s.pid {
                if !ids.contains(pid) {
                    s.pid = None;
                }
            }
        }
        if app.state.tactic.slots.iter().all(|s| s.pid.is_none()) && app.state.squad.len() >= 11 {
            auto_pick_xi(&mut app.state.tactic, &app.state.squad);
        }
        sync_animated_pitch(app, cx);
        let club_str = mc
            .club
            .as_deref()
            .map(|c| format!(" — {}", c))
            .unwrap_or_default();
        app.state.set_toast(
            match lang {
                Lang::Pt => format!("Plantel{}: {} jogadores.", club_str, count),
                Lang::En => format!("Squad{}: {} players.", club_str, count),
            },
            ToastKind::Success,
        );
    }
    app.state.persist();
    cx.notify();
}

/// Opens a native OS file picker and imports a CSV/HTML/TSV/TXT/JSON squad or opponent file.
pub(crate) fn import_squad_from_picker(
    app: &mut ArchitectApp,
    force_opp: bool,
    cx: &mut Context<ArchitectApp>,
) {
    let lang = app.state.lang;
    let title = if force_opp {
        l(
            lang,
            "Escolher ficheiro do plantel adversário",
            "Choose opponent squad file",
        )
    } else {
        l(lang, "Escolher ficheiro do plantel", "Choose squad file")
    };
    let filters = &[
        ("Squad exports", &["csv", "html", "htm", "tsv", "txt", "json"][..]),
    ];
    let Some(path) = pick_open_file(title, filters) else {
        return;
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    if ext == "json" && !force_opp {
        match app.state.import_backup_path(&path) {
            Ok(msg) => {
                sync_animated_pitch(app, cx);
                app.state.set_toast(msg, ToastKind::Success);
                cx.notify();
                return;
            }
            Err(_) => {
                // Fall through: might be a flat player array JSON (e.g. FMLibreEditor)
            }
        }
    }

    match std::fs::read_to_string(&path) {
        Ok(raw) => {
            if ext == "json" {
                if let Some(tsv) = flatten_json_players_to_tsv(&raw) {
                    ingest_squad_text(app, &tsv, &name, Some(force_opp), cx);
                } else {
                    app.state.set_toast(
                        l(lang, "Ficheiro JSON não reconhecido.", "Unrecognised JSON file."),
                        ToastKind::Error,
                    );
                    cx.notify();
                }
            } else {
                ingest_squad_text(app, &raw, &name, Some(force_opp), cx);
            }
        }
        Err(err) => {
            app.state
                .set_toast(format!("{}: {}", name, err), ToastKind::Error);
            cx.notify();
        }
    }
}

fn flatten_json_players_to_tsv(raw: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(raw).ok()?;
    let arr = val
        .as_array()
        .or_else(|| val.get("players").and_then(|v| v.as_array()))?;
    if arr.is_empty() {
        return None;
    }

    fn flatten_obj(
        v: &serde_json::Value,
        prefix: &str,
        out: &mut BTreeMap<String, String>,
    ) {
        if let Some(map) = v.as_object() {
            for (k, child) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}_{}", prefix, k)
                };
                if child.is_object() {
                    flatten_obj(child, &key, out);
                } else if let Some(a) = child.as_array() {
                    let joined: Vec<String> = a
                        .iter()
                        .map(|x| x.as_str().map(str::to_string).unwrap_or_else(|| x.to_string()))
                        .collect();
                    out.insert(key, joined.join(", "));
                } else if let Some(s) = child.as_str() {
                    out.insert(key, s.replace('\t', " "));
                } else if !child.is_null() {
                    out.insert(key, child.to_string());
                }
            }
        }
    }

    let mut rows = Vec::with_capacity(arr.len());
    let mut keys = Vec::new();
    let mut key_set = HashSet::new();
    for item in arr {
        let mut flat = BTreeMap::new();
        flatten_obj(item, "", &mut flat);
        for k in flat.keys() {
            if key_set.insert(k.clone()) {
                keys.push(k.clone());
            }
        }
        rows.push(flat);
    }
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(keys.join("\t"));
    for r in rows {
        let cols: Vec<String> = keys
            .iter()
            .map(|k| r.get(k).cloned().unwrap_or_default())
            .collect();
        lines.push(cols.join("\t"));
    }
    Some(lines.join("\n"))
}

/// Fetches the live managed-club squad from FM27 via `FmbClient::read_squad` on a background thread.
pub(crate) fn spawn_refresh_squad_from_fm(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    app.state.set_busy(
        l(lang, "A ler o plantel do FM…", "Reading squad from FM…"),
        Some(0.4),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.read_squad(|_| {})
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            let lang = this.state.lang;
            match res {
                Ok(mut r) if !r.players.is_empty() => {
                    // Preserve fmsave-only extras by unique_id from previous squad / scout DB
                    let mut prev_by_uid: HashMap<String, Player> = HashMap::new();
                    if let Some(db) = &this.state.scout_db {
                        for sp in &db.players {
                            if let Some(u) = &sp.uniq {
                                prev_by_uid.insert(u.clone(), sp.to_player());
                            }
                        }
                    }
                    for p in &this.state.squad {
                        if let Some(u) = p.uniq.as_ref().or(p.meta.uid.as_ref()) {
                            prev_by_uid.insert(u.clone(), p.clone());
                        }
                    }
                    for p in &mut r.players {
                        if let Some(old) = p.uniq.as_ref().and_then(|u| prev_by_uid.get(u)) {
                            if p.ca.is_none() { p.ca = old.ca; }
                            if p.pa.is_none() { p.pa = old.pa; }
                            if p.pa_range.is_none() { p.pa_range = old.pa_range; }
                            if p.hid.is_empty() { p.hid = old.hid.clone(); }
                            if p.pers.is_empty() { p.pers = old.pers.clone(); }
                            if p.value.is_none() { p.value = old.value; }
                            if p.wage.is_none() { p.wage = old.wage; }
                            if p.end.is_none() { p.end = old.end.clone(); }
                            if p.status.is_none() { p.status = old.status.clone(); }
                            if p.rep.is_none() { p.rep = old.rep; }
                            if p.wrep.is_none() { p.wrep = old.wrep; }
                            if p.nat.is_none() { p.nat = old.nat; }
                            if p.loan.is_none() { p.loan = old.loan; }
                            if p.loan_from.is_none() { p.loan_from = old.loan_from.clone(); }
                        }
                    }
                    let count = r.players.len();
                    let club = r.club.clone();
                    let ids: HashSet<String> = r.players.iter().map(|p| p.id.clone()).collect();
                    this.state.squad = r.players;
                    this.state.squad_source = l(lang, "FM ao vivo (ponte)", "FM live (bridge)").to_string();
                    if !club.is_empty() {
                        this.state.my_club = club.clone();
                    }
                    for s in &mut this.state.tactic.slots {
                        if let Some(pid) = &s.pid {
                            if !ids.contains(pid) {
                                s.pid = None;
                            }
                        }
                    }
                    if this.state.tactic.slots.iter().all(|s| s.pid.is_none()) && count >= 11 {
                        auto_pick_xi(&mut this.state.tactic, &this.state.squad);
                    }
                    sync_animated_pitch(this, cx);
                    this.state.persist();
                    let suffix = if club.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", club)
                    };
                    this.state.set_toast(
                        match lang {
                            Lang::Pt => format!("Plantel atualizado do FM{}: {} jogadores.", suffix, count),
                            Lang::En => format!("Squad refreshed from FM{}: {} players.", suffix, count),
                        },
                        ToastKind::Success,
                    );
                }
                Ok(_) => {
                    this.state.set_toast(
                        l(
                            lang,
                            "Nenhum jogador com atributos foi lido do FM.",
                            "No players with attributes were read from FM.",
                        ),
                        ToastKind::Warn,
                    );
                }
                Err(err) => {
                    this.state.set_toast(err.to_string(), ToastKind::Error);
                }
            }
            cx.notify();
        });
    })
    .detach();
}

/// Fetches the next scheduled opponent from FM27 via `FmbClient::next_opponent` and matches
/// player `unique_id`s against the loaded Scouting DB.
pub(crate) fn spawn_next_opponent_from_fm(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    app.state.set_busy(
        l(
            lang,
            "A procurar o próximo adversário no FM…",
            "Finding the next opponent in FM…",
        ),
        Some(0.4),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.next_opponent()
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            let lang = this.state.lang;
            match res {
                Ok(None) => {
                    this.state.set_toast(
                        l(
                            lang,
                            "O FM não tem um próximo jogo marcado.",
                            "FM has no next match scheduled.",
                        ),
                        ToastKind::Warn,
                    );
                }
                Ok(Some(nx)) => {
                    let Some(db) = &this.state.scout_db else {
                        this.state.set_toast(
                            format!(
                                "{}: {} — {}",
                                l(lang, "Próximo adversário detetado", "Next opponent detected"),
                                nx.name,
                                l(
                                    lang,
                                    "carregue a base de dados em Prospeção primeiro (ou importe o CSV deles).",
                                    "load the Scouting database first (or import their CSV)."
                                )
                            ),
                            ToastKind::Warn,
                        );
                        cx.notify();
                        return;
                    };
                    let by_uid: HashMap<&str, usize> = db
                        .players
                        .iter()
                        .enumerate()
                        .filter_map(|(i, p)| p.uniq.as_deref().map(|u| (u, i)))
                        .collect();
                    let matched: Vec<Player> = nx
                        .uniqs
                        .iter()
                        .filter_map(|u| by_uid.get(u.as_str()).copied())
                        .map(|idx| {
                            let mut p = db.players[idx].to_player();
                            p.club = Some(nx.name.clone());
                            p.meta.club = Some(nx.name.clone());
                            p
                        })
                        .collect();
                    if matched.len() < 11 {
                        this.state.set_toast(
                            match lang {
                                Lang::Pt => format!(
                                    "Só encontrei {} jogadores do {} na base de dados do save.",
                                    matched.len(),
                                    nx.name
                                ),
                                Lang::En => format!(
                                    "Only found {} {} players in the save database.",
                                    matched.len(),
                                    nx.name
                                ),
                            },
                            ToastKind::Warn,
                        );
                    } else {
                        let count = matched.len();
                        this.state.opponent = matched;
                        this.state.opponent_source = format!(
                            "{}{}",
                            nx.name,
                            l(lang, " (próximo jogo, do FM)", " (next match, from FM)")
                        );
                        this.state.persist();
                        this.state.set_toast(
                            match lang {
                                Lang::Pt => format!("Próximo adversário: {} ({} jogadores).", nx.name, count),
                                Lang::En => format!("Next opponent: {} ({} players).", nx.name, count),
                            },
                            ToastKind::Success,
                        );
                    }
                }
                Err(err) => {
                    this.state.set_toast(err.to_string(), ToastKind::Error);
                }
            }
            cx.notify();
        });
    })
    .detach();
}

/// Loads the deterministic 22-player sample squad into `app.state.squad` and auto-picks the XI.
pub(crate) fn load_sample_squad(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let ps = make_sample_squad();
    app.state.squad = ps;
    app.state.squad_source = l(lang, "exemplo (fictício)", "sample (fictional)").to_string();
    app.state.my_club.clear();
    auto_pick_xi(&mut app.state.tactic, &app.state.squad);
    sync_animated_pitch(app, cx);
    app.state.persist();
    app.state.set_toast(
        l(
            lang,
            "Plantel de exemplo carregado (22 jogadores fictícios).",
            "Sample squad loaded (22 fictional players).",
        ),
        ToastKind::Success,
    );
    cx.notify();
}

/// Deterministic 22-player fictional squad ported from `makeSample()` in `src/ui/state.js`.
pub fn make_sample_squad() -> Vec<Player> {
    let mut seed: u64 = 7;
    let mut rnd = || -> f32 {
        seed = (seed * 9301 + 49297) % 233280;
        seed as f32 / 233280.0
    };
    let first = [
        "Rui", "Tiago", "Bruno", "Diogo", "Nuno", "André", "Pedro", "João", "Gonçalo", "Miguel",
        "Rafael", "Hugo", "Luís", "Vasco", "Duarte", "Tomás", "Martim", "Simão", "Afonso",
        "Henrique", "Leandro", "Fábio", "Ivo", "Caio",
    ];
    let last = [
        "Matos", "Ferraz", "Lobo", "Teixeira", "Pinheiro", "Carvalho", "Sardinha", "Valente",
        "Brito", "Quaresma", "Antunes", "Seixas", "Cordeiro", "Maia", "Rebelo", "Viana", "Barros",
        "Neves", "Pacheco", "Leal", "Morais", "Bastos", "Paiva", "Rocha",
    ];
    let profiles: &[(&str, &[(Attr, f32)])] = &[
        ("GK", &[(Attr::Han,14.0),(Attr::Ref,15.0),(Attr::Ovo,13.0),(Attr::Pos,13.0),(Attr::Cnt,13.0),(Attr::Agi,13.0),(Attr::Aer,13.0),(Attr::Cmd,12.0),(Attr::Com,12.0),(Attr::Kic,12.0),(Attr::Pas,11.0),(Attr::Fir,10.0),(Attr::Cmp,12.0),(Attr::Dec,12.0),(Attr::Tro,13.0),(Attr::Ant,12.0),(Attr::Acc,10.0)]),
        ("GK", &[(Attr::Han,12.0),(Attr::Ref,13.0),(Attr::Ovo,11.0),(Attr::Pos,12.0),(Attr::Cnt,11.0),(Attr::Agi,12.0),(Attr::Aer,12.0),(Attr::Cmd,11.0),(Attr::Com,11.0),(Attr::Kic,11.0),(Attr::Pas,9.0),(Attr::Fir,9.0),(Attr::Cmp,10.0),(Attr::Dec,10.0),(Attr::Tro,10.0),(Attr::Ant,10.0)]),
        ("D (C)", &[(Attr::Hea,15.0),(Attr::Mar,14.0),(Attr::Tck,14.0),(Attr::Pos,14.0),(Attr::Str,15.0),(Attr::Jum,15.0),(Attr::Cmp,12.0),(Attr::Dec,12.0),(Attr::Cnt,13.0),(Attr::Ant,13.0),(Attr::Pac,11.0),(Attr::Acc,10.0),(Attr::Pas,11.0),(Attr::Bra,14.0),(Attr::Agg,12.0)]),
        ("D (C)", &[(Attr::Hea,13.0),(Attr::Mar,13.0),(Attr::Tck,14.0),(Attr::Pos,13.0),(Attr::Str,12.0),(Attr::Jum,12.0),(Attr::Cmp,14.0),(Attr::Dec,13.0),(Attr::Cnt,12.0),(Attr::Ant,13.0),(Attr::Pac,14.0),(Attr::Acc,14.0),(Attr::Pas,14.0),(Attr::Vis,12.0),(Attr::Fir,12.0),(Attr::Tec,12.0)]),
        ("D (C)", &[(Attr::Hea,14.0),(Attr::Mar,12.0),(Attr::Tck,13.0),(Attr::Pos,12.0),(Attr::Str,14.0),(Attr::Jum,14.0),(Attr::Cmp,10.0),(Attr::Dec,11.0),(Attr::Cnt,11.0),(Attr::Ant,11.0),(Attr::Pac,12.0),(Attr::Acc,11.0),(Attr::Pas,9.0),(Attr::Bra,13.0)]),
        ("D (RL), WB (R)", &[(Attr::Tck,13.0),(Attr::Mar,12.0),(Attr::Pos,12.0),(Attr::Cro,13.0),(Attr::Dri,12.0),(Attr::Wor,15.0),(Attr::Sta,15.0),(Attr::Acc,15.0),(Attr::Pac,15.0),(Attr::Tea,13.0),(Attr::Otb,12.0),(Attr::Ant,12.0),(Attr::Pas,12.0),(Attr::Cnt,11.0),(Attr::Dec,11.0)]),
        ("D (L), WB (L)", &[(Attr::Tck,12.0),(Attr::Mar,12.0),(Attr::Pos,11.0),(Attr::Cro,14.0),(Attr::Dri,13.0),(Attr::Wor,14.0),(Attr::Sta,15.0),(Attr::Acc,14.0),(Attr::Pac,14.0),(Attr::Tea,13.0),(Attr::Otb,13.0),(Attr::Tec,12.0),(Attr::Pas,12.0),(Attr::Fir,12.0),(Attr::Dec,11.0)]),
        ("D (RL)", &[(Attr::Tck,14.0),(Attr::Mar,14.0),(Attr::Pos,14.0),(Attr::Cnt,13.0),(Attr::Ant,13.0),(Attr::Tea,13.0),(Attr::Cro,10.0),(Attr::Pas,12.0),(Attr::Cmp,13.0),(Attr::Dec,13.0),(Attr::Str,13.0),(Attr::Hea,13.0),(Attr::Wor,12.0),(Attr::Sta,13.0),(Attr::Acc,12.0)]),
        ("DM, M (C)", &[(Attr::Tck,15.0),(Attr::Pos,15.0),(Attr::Ant,14.0),(Attr::Cnt,14.0),(Attr::Dec,14.0),(Attr::Tea,15.0),(Attr::Mar,13.0),(Attr::Pas,13.0),(Attr::Cmp,13.0),(Attr::Str,14.0),(Attr::Wor,15.0),(Attr::Sta,14.0),(Attr::Fir,12.0),(Attr::Vis,11.0)]),
        ("DM, M (C)", &[(Attr::Pas,16.0),(Attr::Fir,15.0),(Attr::Tec,15.0),(Attr::Cmp,15.0),(Attr::Dec,15.0),(Attr::Vis,16.0),(Attr::Tea,14.0),(Attr::Ant,13.0),(Attr::Pos,12.0),(Attr::Tck,10.0),(Attr::Wor,12.0),(Attr::Sta,12.0),(Attr::Bal,13.0)]),
        ("M (C)", &[(Attr::Pas,13.0),(Attr::Tck,12.0),(Attr::Otb,14.0),(Attr::Tea,14.0),(Attr::Wor,16.0),(Attr::Sta,16.0),(Attr::Dri,12.0),(Attr::Fin,12.0),(Attr::Fir,13.0),(Attr::Lon,13.0),(Attr::Ant,13.0),(Attr::Cmp,12.0),(Attr::Dec,12.0),(Attr::Acc,13.0),(Attr::Str,13.0)]),
        ("M (C), AM (C)", &[(Attr::Pas,15.0),(Attr::Fir,15.0),(Attr::Tec,16.0),(Attr::Vis,15.0),(Attr::Cmp,14.0),(Attr::Dec,14.0),(Attr::Otb,14.0),(Attr::Tea,12.0),(Attr::Dri,14.0),(Attr::Fla,15.0),(Attr::Ant,13.0),(Attr::Lon,14.0),(Attr::Agi,14.0)]),
        ("M (C)", &[(Attr::Pas,12.0),(Attr::Tck,13.0),(Attr::Dec,12.0),(Attr::Tea,13.0),(Attr::Fir,12.0),(Attr::Wor,14.0),(Attr::Sta,14.0),(Attr::Pos,12.0),(Attr::Ant,12.0),(Attr::Cmp,11.0)]),
        ("AM (RL), ST (C)", &[(Attr::Dri,15.0),(Attr::Fin,14.0),(Attr::Fir,14.0),(Attr::Tec,15.0),(Attr::Otb,14.0),(Attr::Cmp,13.0),(Attr::Acc,16.0),(Attr::Agi,15.0),(Attr::Pac,15.0),(Attr::Fla,14.0),(Attr::Pas,12.0),(Attr::Lon,12.0),(Attr::Ant,12.0),(Attr::Dec,12.0),(Attr::Wor,12.0),(Attr::Sta,13.0)]),
        ("AM (R), M (R)", &[(Attr::Cro,15.0),(Attr::Dri,15.0),(Attr::Tec,14.0),(Attr::Acc,16.0),(Attr::Agi,14.0),(Attr::Pac,16.0),(Attr::Fir,13.0),(Attr::Pas,12.0),(Attr::Otb,13.0),(Attr::Fla,13.0),(Attr::Wor,12.0),(Attr::Sta,13.0),(Attr::Tea,11.0)]),
        ("AM (L)", &[(Attr::Dri,14.0),(Attr::Fin,12.0),(Attr::Fir,13.0),(Attr::Tec,14.0),(Attr::Otb,12.0),(Attr::Cmp,12.0),(Attr::Acc,14.0),(Attr::Agi,14.0),(Attr::Pac,13.0),(Attr::Pas,13.0),(Attr::Vis,13.0),(Attr::Fla,13.0),(Attr::Wor,11.0),(Attr::Sta,12.0)]),
        ("ST (C)", &[(Attr::Fin,15.0),(Attr::Fir,14.0),(Attr::Hea,14.0),(Attr::Tec,13.0),(Attr::Cmp,14.0),(Attr::Otb,15.0),(Attr::Acc,14.0),(Attr::Str,14.0),(Attr::Ant,14.0),(Attr::Dri,12.0),(Attr::Pac,13.0),(Attr::Cnt,12.0),(Attr::Wor,12.0),(Attr::Tea,12.0),(Attr::Dec,12.0)]),
        ("ST (C)", &[(Attr::Hea,16.0),(Attr::Str,16.0),(Attr::Jum,15.0),(Attr::Bra,14.0),(Attr::Fin,13.0),(Attr::Cmp,12.0),(Attr::Bal,14.0),(Attr::Fir,12.0),(Attr::Otb,12.0),(Attr::Tea,13.0),(Attr::Agg,13.0),(Attr::Ant,12.0),(Attr::Pac,10.0),(Attr::Acc,10.0)]),
        ("AM (C), ST (C)", &[(Attr::Fin,13.0),(Attr::Fir,13.0),(Attr::Otb,14.0),(Attr::Cmp,12.0),(Attr::Ant,13.0),(Attr::Acc,14.0),(Attr::Dri,13.0),(Attr::Tec,13.0),(Attr::Dec,12.0),(Attr::Pas,12.0),(Attr::Lon,12.0),(Attr::Agi,13.0),(Attr::Wor,13.0)]),
        ("D (C), DM", &[(Attr::Mar,13.0),(Attr::Tck,13.0),(Attr::Pos,13.0),(Attr::Ant,13.0),(Attr::Cmp,13.0),(Attr::Cnt,12.0),(Attr::Dec,13.0),(Attr::Tea,13.0),(Attr::Pas,12.0),(Attr::Str,13.0),(Attr::Jum,13.0),(Attr::Hea,13.0),(Attr::Fir,11.0)]),
        ("WB (L), M (L)", &[(Attr::Cro,13.0),(Attr::Dri,13.0),(Attr::Tec,12.0),(Attr::Otb,13.0),(Attr::Tea,12.0),(Attr::Wor,14.0),(Attr::Acc,15.0),(Attr::Pac,15.0),(Attr::Sta,14.0),(Attr::Fir,12.0),(Attr::Pas,11.0),(Attr::Tck,10.0),(Attr::Mar,9.0)]),
        ("M (R), AM (R)", &[(Attr::Pas,13.0),(Attr::Cro,13.0),(Attr::Tea,14.0),(Attr::Wor,15.0),(Attr::Dec,12.0),(Attr::Sta,15.0),(Attr::Fir,12.0),(Attr::Tec,12.0),(Attr::Otb,12.0),(Attr::Tck,11.0),(Attr::Pos,11.0),(Attr::Acc,13.0)]),
    ];
    let gk_attrs = [
        Attr::Aer, Attr::Cmd, Attr::Com, Attr::Ecc, Attr::Han, Attr::Kic, Attr::Ovo, Attr::Pun,
        Attr::Ref, Attr::Tro, Attr::Thr,
    ];

    profiles
        .iter()
        .enumerate()
        .map(|(i, &(pos_raw, overrides))| {
            let mut a = BTreeMap::new();
            for &attr in &Attr::ALL {
                a.insert(attr, (6.0 + (rnd() * 5.0).round()).clamp(1.0, 20.0));
            }
            if !pos_raw.starts_with("GK") {
                for &gka in &gk_attrs {
                    a.insert(gka, 1.0 + (rnd() * 3.0).round());
                }
            }
            for &(attr, base) in overrides {
                let jitter = (rnd() * 2.0 - 1.0).round();
                a.insert(attr, (base + jitter).clamp(1.0, 20.0));
            }
            a.insert(Attr::Det, 8.0 + (rnd() * 10.0).round());
            let age = 18.0 + (rnd() * 15.0).round();
            Player {
                id: format!("sample{}", i),
                name: format!("{} {}", first[i % first.len()], last[(i * 7) % last.len()]),
                pos_raw: pos_raw.to_string(),
                pos: parse_positions(pos_raw),
                age: Some(age),
                a,
                ..Player::default()
            }
        })
        .collect()
}

// ============================================================================
// Formatting & Small UI Helpers
// ============================================================================

fn fmt_money(v: Option<f64>) -> String {
    let Some(val) = v else {
        return "—".to_string();
    };
    if val >= 1_000_000.0 {
        format!("€{:.1}M", val / 1_000_000.0)
    } else if val >= 1_000.0 {
        format!("€{:.0}K", val / 1_000.0)
    } else if val > 0.0 {
        format!("€{:.0}", val)
    } else {
        "—".to_string()
    }
}

fn fit_badge(score: u8) -> Div {
    div()
        .px_2()
        .py(px(1.0))
        .rounded_sm()
        .bg(score_color(score))
        .text_xs()
        .font_bold()
        .text_color(rgb(0xffffff))
        .child(score.to_string())
}

fn pers_val_badge(val: Option<f32>, inverted: bool, cx: &App) -> Div {
    let theme = cx.theme();
    let Some(v) = val else {
        return div()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child("—");
    };
    let eff = if inverted { 21.0 - v } else { v };
    let col = attr_color(eff);
    div()
        .px_2()
        .py(px(1.0))
        .rounded_sm()
        .bg(with_alpha(col, 0.16))
        .text_xs()
        .font_bold()
        .text_color(col)
        .child(format!("{}", v.round() as i32))
}

fn best_roles_across_positions(p: &Player, phase: Phase, limit: usize) -> Vec<(&'static str, u8)> {
    let groups: Vec<PosGroup> = if p.pos.is_empty() {
        PosGroup::ALL.to_vec()
    } else {
        let mut gs = Vec::new();
        for tok in &p.pos {
            if !tok.contains(':') {
                if let Some(g) = PosGroup::from_code(tok) {
                    if !gs.contains(&g) {
                        gs.push(g);
                    }
                }
            }
        }
        if gs.is_empty() {
            PosGroup::ALL.to_vec()
        } else {
            gs
        }
    };
    let mut seen = HashSet::new();
    let mut scored: Vec<(&'static str, u8)> = Vec::new();
    for g in groups {
        for r in roles_for(g, phase, None, g == PosGroup::Dc) {
            if seen.insert(r.id) {
                let sc = role_score(p, r).map_or(0, |rf| rf.score);
                scored.push((r.id, sc));
            }
        }
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(limit);
    scored
}

const PERS_COLS: &[(&str, &str, &str, bool, bool)] = &[
    // (key, pt, en, is_hid, inverted)
    ("professionalism", "Prof.", "Prof.", false, false),
    ("ambition", "Amb.", "Amb.", false, false),
    ("pressure", "Press.", "Press.", false, false),
    ("loyalty", "Leal.", "Loyal.", false, false),
    ("temperament", "Temp.", "Temp.", false, false),
    ("adaptability", "Adapt.", "Adapt.", false, false),
    ("cons", "Cons.", "Cons.", true, false),
    ("imp", "Imp.", "Imp.", true, false),
    ("inj", "Lesões", "Inj.", true, true),
    ("vers", "Vers.", "Vers.", true, false),
];

fn sort_indicator(active: bool, asc: bool) -> &'static str {
    if !active {
        ""
    } else if asc {
        " ▲"
    } else {
        " ▼"
    }
}

// ============================================================================
// 1. My Team › Squad (`render_team_squad`)
// ============================================================================

pub fn render_team_squad(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme().clone();
    let squad = &app.state.squad;
    let ui = &app.team_ui;

    let club_title = if app.state.my_club.is_empty() {
        l(lang, "Plantel", "Squad").to_string()
    } else {
        format!("{} — {}", l(lang, "Plantel", "Squad"), app.state.my_club)
    };
    let subtitle = if squad.is_empty() {
        l(lang, "Ainda sem plantel carregado.", "No squad loaded yet.").to_string()
    } else {
        match lang {
            Lang::Pt => format!("{} jogadores · {}", squad.len(), app.state.squad_source),
            Lang::En => format!("{} players · {}", squad.len(), app.state.squad_source),
        }
    };

    let header = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(theme.foreground)
                        .child(club_title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("sq-read-save")
                        .primary()
                        .small()
                        .label(l(lang, "Ler save agora", "Read save now"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.spawn_read_save_now(cx);
                        })),
                )
                .child(
                    Button::new("sq-refresh-fm")
                        .outline()
                        .small()
                        .label(l(lang, "Atualizar do FM", "Refresh from FM"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_refresh_squad_from_fm(this, cx);
                        })),
                )
                .child(
                    Button::new("sq-import-file")
                        .outline()
                        .small()
                        .label(l(lang, "Importar CSV/HTML/JSON", "Import CSV/HTML/JSON"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            import_squad_from_picker(this, false, cx);
                        })),
                )
                .child(
                    Button::new("sq-sample")
                        .ghost()
                        .small()
                        .label(l(lang, "Usar exemplo", "Use sample"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            load_sample_squad(this, cx);
                        })),
                )
                .when(!squad.is_empty(), |d| {
                    d.child(
                        Button::new("sq-clear")
                            .ghost()
                            .small()
                            .label(l(lang, "Limpar", "Clear"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.squad.clear();
                                this.state.squad_source.clear();
                                this.state.my_club.clear();
                                for s in &mut this.state.tactic.slots {
                                    s.pid = None;
                                }
                                this.team_ui.selected_player_id = None;
                                sync_animated_pitch(this, cx);
                                this.state.persist();
                                cx.notify();
                            })),
                    )
                }),
        );

    if squad.is_empty() {
        let empty = empty_state_card(
            l(lang, "Carregue o seu plantel do FM27", "Load your FM27 squad"),
            l(
                lang,
                "Leia diretamente do save (.fm) via fmsave, atualize ao vivo via Ponte FM27, importe um CSV/HTML/JSON ou carregue o plantel de exemplo.",
                "Read directly from your .fm save via fmsave, refresh live from FM27 Bridge, import a CSV/HTML/JSON file, or try the sample squad.",
            ),
            Some(
                div()
                    .h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("sq-empty-save")
                            .primary()
                            .small()
                            .label(l(lang, "Ler save agora", "Read save now"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.spawn_read_save_now(cx);
                            })),
                    )
                    .child(
                        Button::new("sq-empty-file")
                            .outline()
                            .small()
                            .label(l(lang, "Importar ficheiro", "Import file"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                import_squad_from_picker(this, false, cx);
                            })),
                    )
                    .child(
                        Button::new("sq-empty-sample")
                            .outline()
                            .small()
                            .label(l(lang, "Usar plantel de exemplo", "Use sample squad"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                load_sample_squad(this, cx);
                            })),
                    )
                    .into_any_element(),
            ),
            cx,
        );
        return div().v_flex().w_full().gap_4().child(header).child(empty);
    }

    // ------------------------------------------------------------------------
    // Filter & Mode Controls
    // ------------------------------------------------------------------------
    let mut pos_row = div().h_flex().flex_wrap().items_center().gap_1().child(
        Button::new("sq-pos-all")
            .xsmall()
            .when(ui.pos_filter.is_none(), |b| b.primary())
            .when(ui.pos_filter.is_some(), |b| b.outline())
            .label(l(lang, "Todas", "All"))
            .on_click(cx.listener(|this, _, _, cx| {
                this.team_ui.pos_filter = None;
                cx.notify();
            })),
    );
    for (idx, g) in PosGroup::ALL.iter().copied().enumerate() {
        let on = ui.pos_filter == Some(g);
        pos_row = pos_row.child(
            Button::new(("sq-pos", idx))
                .xsmall()
                .when(on, |b| b.primary())
                .when(!on, |b| b.outline())
                .label(g.code())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.team_ui.pos_filter = if this.team_ui.pos_filter == Some(g) {
                        None
                    } else {
                        Some(g)
                    };
                    cx.notify();
                })),
        );
    }

    let mode_row = div()
        .h_flex()
        .gap_1()
        .child(
            Button::new("sq-mode-roles")
                .small()
                .when(ui.squad_mode == SquadTableMode::Roles, |b| b.primary())
                .when(ui.squad_mode != SquadTableMode::Roles, |b| b.outline())
                .label(l(lang, "Papéis", "Roles"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.team_ui.squad_mode = SquadTableMode::Roles;
                    cx.notify();
                })),
        )
        .child(
            Button::new("sq-mode-ability")
                .small()
                .when(ui.squad_mode == SquadTableMode::Ability, |b| b.primary())
                .when(ui.squad_mode != SquadTableMode::Ability, |b| b.outline())
                .label(l(lang, "Habilidade e contrato", "Ability & contract"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.team_ui.squad_mode = SquadTableMode::Ability;
                    cx.notify();
                })),
        )
        .child(
            Button::new("sq-mode-pers")
                .small()
                .when(ui.squad_mode == SquadTableMode::Personality, |b| b.primary())
                .when(ui.squad_mode != SquadTableMode::Personality, |b| b.outline())
                .label(l(lang, "Personalidade e ocultos", "Personality & hidden"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.team_ui.squad_mode = SquadTableMode::Personality;
                    cx.notify();
                })),
        );

    let controls_bar = div()
        .v_flex()
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_3()
                .when_some(ui.search_input.as_ref(), |d, inp| {
                    d.child(div().w(px(260.0)).child(Input::new(inp)))
                })
                .child(mode_row),
        )
        .child(pos_row);

    // ------------------------------------------------------------------------
    // Filter + Sort Players
    // ------------------------------------------------------------------------
    let q = ui.search_query.trim().to_lowercase();
    let mut filtered: Vec<&Player> = squad
        .iter()
        .filter(|p| {
            if let Some(g) = ui.pos_filter {
                if !p.pos.iter().any(|tok| tok == g.code()) {
                    return false;
                }
            }
            if !q.is_empty() {
                let hay = format!("{} {}", p.name, p.pos_raw).to_lowercase();
                if !hay.contains(&q) {
                    return false;
                }
            }
            true
        })
        .collect();

    let sort_col = ui.squad_sort;
    let sort_asc = ui.squad_sort_asc;
    if sort_col != SquadSortCol::None {
        filtered.sort_by(|a, b| {
            let ord = match sort_col {
                SquadSortCol::None => std::cmp::Ordering::Equal,
                SquadSortCol::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SquadSortCol::Age => cmp_opt_f32(a.age, b.age),
                SquadSortCol::Ca => cmp_opt_f32(a.ca, b.ca),
                SquadSortCol::Pa => cmp_opt_f32(a.pa, b.pa),
                SquadSortCol::Value => cmp_opt_f64(a.value, b.value),
                SquadSortCol::Wage => cmp_opt_f64(a.wage, b.wage),
                SquadSortCol::ContractEnd => cmp_opt_str(a.end.as_deref(), b.end.as_deref()),
                SquadSortCol::Pers(k) => cmp_opt_f32(a.pers.get(k).copied(), b.pers.get(k).copied()),
                SquadSortCol::Hid(k) => cmp_opt_f32(a.hid.get(k).copied(), b.hid.get(k).copied()),
            };
            if sort_asc { ord } else { ord.reverse() }
        });
    }

    let has_ca = squad.iter().any(|p| p.ca.is_some());
    let has_pers = squad.iter().any(|p| !p.pers.is_empty());

    // ------------------------------------------------------------------------
    // Table Header
    // ------------------------------------------------------------------------
    let sort_btn = |id: (&'static str, usize), label: String, col: SquadSortCol, cx: &mut Context<ArchitectApp>| {
        let on = sort_col == col;
        Button::new(id)
            .ghost()
            .xsmall()
            .label(format!("{}{}", label, sort_indicator(on, sort_asc)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.team_ui.toggle_sort(col);
                cx.notify();
            }))
    };

    let mut thead = div()
        .h_flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(div().w(px(175.0)).child(sort_btn(
            ("sq-h-name", 0),
            l(lang, "Jogador", "Player").to_string(),
            SquadSortCol::Name,
            cx,
        )))
        .child(
            div()
                .w(px(105.0))
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(l(lang, "Pos", "Pos")),
        )
        .child(div().w(px(54.0)).child(sort_btn(
            ("sq-h-age", 1),
            l(lang, "Idade", "Age").to_string(),
            SquadSortCol::Age,
            cx,
        )));

    match ui.squad_mode {
        SquadTableMode::Roles => {
            if has_ca {
                thead = thead.child(div().w(px(56.0)).child(sort_btn(
                    ("sq-h-ca", 2),
                    "CA".to_string(),
                    SquadSortCol::Ca,
                    cx,
                )));
            }
            thead = thead
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(lang, "Melhores papéis IP", "Best IP roles")),
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(lang, "Melhores papéis OOP", "Best OOP roles")),
                );
        }
        SquadTableMode::Ability => {
            thead = thead
                .child(div().w(px(56.0)).child(sort_btn(("sq-h-ca2", 10), "CA".into(), SquadSortCol::Ca, cx)))
                .child(div().w(px(56.0)).child(sort_btn(("sq-h-pa", 11), "PA".into(), SquadSortCol::Pa, cx)))
                .child(div().w(px(86.0)).child(sort_btn(
                    ("sq-h-val", 12),
                    l(lang, "Valor", "Value").into(),
                    SquadSortCol::Value,
                    cx,
                )))
                .child(div().w(px(86.0)).child(sort_btn(
                    ("sq-h-wage", 13),
                    l(lang, "Salário", "Wage").into(),
                    SquadSortCol::Wage,
                    cx,
                )))
                .child(div().w(px(96.0)).child(sort_btn(
                    ("sq-h-end", 14),
                    l(lang, "Contrato", "Contract").into(),
                    SquadSortCol::ContractEnd,
                    cx,
                )))
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(lang, "Estatuto", "Status")),
                );
        }
        SquadTableMode::Personality => {
            for (ci, &(key, pt, en, is_hid, _)) in PERS_COLS.iter().enumerate() {
                let col = if is_hid {
                    SquadSortCol::Hid(key)
                } else {
                    SquadSortCol::Pers(key)
                };
                thead = thead.child(div().w(px(56.0)).child(sort_btn(
                    ("sq-h-pers", ci),
                    l(lang, pt, en).to_string(),
                    col,
                    cx,
                )));
            }
        }
    }

    // ------------------------------------------------------------------------
    // Table Rows
    // ------------------------------------------------------------------------
    let mut table_rows = div().v_flex();
    for (ri, p) in filtered.iter().enumerate() {
        let pid = p.id.clone();
        let selected = ui.selected_player_id.as_deref() == Some(p.id.as_str());
        let mut row = div()
            .id(("sq-row", ri))
            .h_flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.border)
            .cursor_pointer()
            .when(selected, |d| d.bg(with_alpha(theme.primary, 0.14)))
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.team_ui.selected_player_id.as_deref() == Some(pid.as_str()) {
                    this.team_ui.selected_player_id = None;
                } else {
                    this.team_ui.selected_player_id = Some(pid.clone());
                }
                cx.notify();
            }))
            .child(
                div()
                    .w(px(175.0))
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .truncate()
                    .child(p.name.clone()),
            )
            .child(
                div()
                    .w(px(105.0))
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .truncate()
                    .child(if p.pos_raw.is_empty() {
                        "—".to_string()
                    } else {
                        p.pos_raw.clone()
                    }),
            )
            .child(
                div()
                    .w(px(54.0))
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(
                        p.age
                            .map(|a| format!("{}", a.round() as i32))
                            .unwrap_or_else(|| "—".to_string()),
                    ),
            );

        match ui.squad_mode {
            SquadTableMode::Roles => {
                if has_ca {
                    row = row.child(
                        div()
                            .w(px(56.0))
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(
                                p.ca.map(|v| format!("{}", v.round() as i32))
                                    .unwrap_or_else(|| "—".to_string()),
                            ),
                    );
                }
                let ip_best = best_roles_across_positions(p, Phase::Ip, 3);
                let oop_best = best_roles_across_positions(p, Phase::Oop, 3);
                let render_role_list = |items: Vec<(&'static str, u8)>| {
                    let mut col = div().v_flex().flex_1().gap(px(2.0));
                    for (rid, sc) in items {
                        let rname = role_by_id(rid)
                            .map(|r| role_name(r, lang))
                            .unwrap_or_else(|| rid.to_string());
                        col = col.child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_1()
                                .child(fit_badge(sc))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.foreground)
                                        .truncate()
                                        .child(rname),
                                ),
                        );
                    }
                    col
                };
                row = row
                    .child(render_role_list(ip_best))
                    .child(render_role_list(oop_best));
            }
            SquadTableMode::Ability => {
                row = row
                    .child(
                        div()
                            .w(px(56.0))
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(
                                p.ca.map(|v| format!("{}", v.round() as i32))
                                    .unwrap_or_else(|| "—".to_string()),
                            ),
                    )
                    .child(
                        div()
                            .w(px(56.0))
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(
                                p.pa.map(|v| {
                                    format!(
                                        "{}{}",
                                        v.round() as i32,
                                        if p.pa_range.is_some() { "*" } else { "" }
                                    )
                                })
                                .unwrap_or_else(|| "—".to_string()),
                            ),
                    )
                    .child(
                        div()
                            .w(px(86.0))
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(fmt_money(p.value)),
                    )
                    .child(
                        div()
                            .w(px(86.0))
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(fmt_money(p.wage)),
                    )
                    .child(
                        div()
                            .w(px(96.0))
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(p.end.clone().unwrap_or_else(|| "—".to_string())),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .truncate()
                            .child(p.status.clone().unwrap_or_else(|| "—".to_string())),
                    );
            }
            SquadTableMode::Personality => {
                for &(key, _, _, is_hid, inv) in PERS_COLS {
                    let val = if is_hid {
                        p.hid.get(key).copied()
                    } else {
                        p.pers.get(key).copied()
                    };
                    row = row.child(div().w(px(56.0)).child(pers_val_badge(val, inv, cx)));
                }
            }
        }
        table_rows = table_rows.child(row);
    }

    let missing_rich_notice = if (ui.squad_mode == SquadTableMode::Ability && !has_ca)
        || (ui.squad_mode == SquadTableMode::Personality && !has_pers)
    {
        Some(
            div()
                .p_2()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(l(
                    lang,
                    "Estes dados vêm do save: leia o plantel com «Ler save agora» (fmsave) para ver CA/PA, personalidade, atributos ocultos, contrato e valor.",
                    "This data comes from your save: read the squad with \"Read save now\" (fmsave) to see CA/PA, personality, hidden attributes, contract, and value.",
                )),
        )
    } else {
        None
    };

    let table_card = div()
        .v_flex()
        .w_full()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .when_some(missing_rich_notice, |d, n| d.child(div().p_3().child(n)))
        .child(
            div()
                .px_3()
                .py_2()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(match lang {
                    Lang::Pt => format!(
                        "{} de {} jogadores · clique num jogador para abrir a ficha completa.",
                        filtered.len(),
                        squad.len()
                    ),
                    Lang::En => format!(
                        "{} of {} players · click any player row to open the full profile.",
                        filtered.len(),
                        squad.len()
                    ),
                }),
        )
        .child(thead)
        .child(table_rows);

    // ------------------------------------------------------------------------
    // Clickable Player Profile Panel (inline detail drawer)
    // ------------------------------------------------------------------------
    let selected_player = ui
        .selected_player_id
        .as_deref()
        .and_then(|id| squad.iter().find(|p| p.id == id));

    let profile_panel = selected_player.map(|p| render_player_profile(p, &app.state.tactic, lang, cx));

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .child(controls_bar)
        .when_some(profile_panel, |d, card| d.child(card))
        .child(table_card)
}

fn render_player_profile(
    p: &Player,
    tactic: &Tactic,
    lang: Lang,
    cx: &mut Context<ArchitectApp>,
) -> impl IntoElement {
    let theme = cx.theme();
    let best_ip = best_roles_across_positions(p, Phase::Ip, 5);
    let best_oop = best_roles_across_positions(p, Phase::Oop, 5);

    let mut slot_fits: Vec<(&str, u8)> = tactic
        .slots
        .iter()
        .map(|s| (s.ip_code.as_str(), slot_score(p, tactic, s)))
        .collect();
    slot_fits.sort_by(|a, b| b.1.cmp(&a.1));

    let attr_group_col = |cat: AttrCat, title: &'static str| {
        let mut col = div().v_flex().flex_1().min_w(px(160.0)).gap_1().child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(theme.foreground)
                .child(title),
        );
        let mut chips = div().h_flex().flex_wrap().gap_1();
        for &a in &Attr::ALL {
            if a.category() == cat {
                if let Some(val) = p.a.get(&a).copied() {
                    chips = chips.child(attr_chip(attr_name(a, lang), val, cx));
                }
            }
        }
        col = col.child(chips);
        col
    };

    let mut roles_ip_row = div().h_flex().flex_wrap().gap_2();
    for (rid, sc) in best_ip {
        let rn = role_by_id(rid)
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| rid.to_string());
        roles_ip_row = roles_ip_row.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(fit_badge(sc))
                .child(div().text_xs().text_color(theme.foreground).child(rn)),
        );
    }

    let mut roles_oop_row = div().h_flex().flex_wrap().gap_2();
    for (rid, sc) in best_oop {
        let rn = role_by_id(rid)
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| rid.to_string());
        roles_oop_row = roles_oop_row.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(fit_badge(sc))
                .child(div().text_xs().text_color(theme.foreground).child(rn)),
        );
    }

    let mut xi_fit_row = div().h_flex().flex_wrap().gap_2();
    for (code, sc) in slot_fits.into_iter().take(6) {
        xi_fit_row = xi_fit_row.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .px_2()
                .py(px(2.0))
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(code.to_string()),
                )
                .child(fit_badge(sc)),
        );
    }

    let meta_summary = [
        (!p.pos_raw.is_empty()).then(|| p.pos_raw.clone()),
        p.age.map(|a| format!("{} {}", a.round() as i32, l(lang, "anos", "yrs"))),
        p.ca.map(|v| format!("CA {}", v.round() as i32)),
        p.pa.map(|v| format!("PA {}", v.round() as i32)),
        p.height.map(|h| format!("{} cm", h.round() as i32)),
        p.value.map(|v| fmt_money(Some(v))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");

    let is_gk = p.pos.iter().any(|x| x == "GK");

    div()
        .v_flex()
        .w_full()
        .gap_3()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(theme.primary)
        .bg(theme.secondary)
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .v_flex()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_base()
                                .font_bold()
                                .text_color(theme.foreground)
                                .child(p.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(meta_summary),
                        ),
                )
                .child(
                    Button::new("sq-close-profile")
                        .ghost()
                        .small()
                        .label(l(lang, "✕ Fechar ficha", "✕ Close profile"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.team_ui.selected_player_id = None;
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(lang, "Melhores papéis IP", "Best IP roles")),
                )
                .child(roles_ip_row)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(lang, "Melhores papéis OOP", "Best OOP roles")),
                )
                .child(roles_oop_row)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.muted_foreground)
                        .child(l(
                            lang,
                            "Encaixe na tática atual (XI)",
                            "Fit in current tactic (XI)",
                        )),
                )
                .child(xi_fit_row),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_start()
                .gap_4()
                .pt_2()
                .border_t_1()
                .border_color(theme.border)
                .when(!is_gk, |d| {
                    d.child(attr_group_col(
                        AttrCat::Technical,
                        l(lang, "Técnicos", "Technical"),
                    ))
                })
                .when(is_gk, |d| {
                    d.child(attr_group_col(
                        AttrCat::Goalkeeping,
                        l(lang, "Guarda-redes", "Goalkeeping"),
                    ))
                })
                .child(attr_group_col(AttrCat::Mental, l(lang, "Mentais", "Mental")))
                .child(attr_group_col(
                    AttrCat::Physical,
                    l(lang, "Físicos", "Physical"),
                )),
        )
}

fn cmp_opt_f32(a: Option<f32>, b: Option<f32>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.total_cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn cmp_opt_f64(a: Option<f64>, b: Option<f64>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.total_cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn cmp_opt_str(a: Option<&str>, b: Option<&str>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// ============================================================================
// 2. My Team › Best XI & Systems (`render_team_best_xi`)
// ============================================================================

pub fn render_team_best_xi(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let squad = &app.state.squad;
    let tactic = &app.state.tactic;

    if squad.len() < 11 {
        let mut tpl_grid = div().h_flex().flex_wrap().gap_3();
        for (ti, tp) in TEMPLATES.iter().enumerate() {
            let tpl_id = tp.id;
            tpl_grid = tpl_grid.child(
                div()
                    .v_flex()
                    .w(px(300.0))
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(t(lang, tp.name_pt)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(format!(
                                "{} → {} · {}",
                                tp.ip,
                                tp.oop,
                                mentality_name(tp.ment, lang)
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(t(lang, tp.desc_pt)),
                    )
                    .child(
                        Button::new(("xi-tpl", ti))
                            .outline()
                            .small()
                            .label(l(lang, "Abrir no analisador", "Open in analyser"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(tp) = template_by_id(tpl_id) {
                                    let mut tac = tactic_from_template(tp);
                                    if this.state.squad.len() >= 11 {
                                        auto_pick_xi(&mut tac, &this.state.squad);
                                    }
                                    apply_tactic_to_app(this, tac, true, cx);
                                }
                            })),
                    ),
            );
        }

        return div()
            .v_flex()
            .w_full()
            .gap_4()
            .child(empty_state_card(
                l(lang, "Carregue o plantel primeiro", "Load your squad first"),
                l(
                    lang,
                    "Com pelo menos 11 jogadores, a app escolhe o melhor XI, recomenda instruções individuais pelos atributos, mostra suplentes/sucessores e ordena os sistemas.",
                    "With at least 11 players, the app picks the best XI, recommends player instructions from attributes, shows backups/successors, and ranks tactical systems.",
                ),
                Some(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            Button::new("xi-empty-save")
                                .primary()
                                .small()
                                .label(l(lang, "Ler save agora", "Read save now"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.spawn_read_save_now(cx);
                                })),
                        )
                        .child(
                            Button::new("xi-empty-sample")
                                .outline()
                                .small()
                                .label(l(lang, "Usar exemplo", "Use sample"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    load_sample_squad(this, cx);
                                })),
                        )
                        .into_any_element(),
                ),
                cx,
            ))
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(l(lang, "Modelos disponíveis", "Available templates")),
                    )
                    .child(tpl_grid),
            );
    }

    let pmap: HashMap<&str, &Player> = squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let benches = bench_and_successors(tactic, squad);
    let slot_scores: Vec<u8> = tactic
        .slots
        .iter()
        .map(|s| {
            s.pid
                .as_deref()
                .and_then(|pid| pmap.get(pid).copied())
                .map_or(0, |p| slot_score(p, tactic, s))
        })
        .collect();

    let header = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(theme.foreground)
                        .child(format!(
                            "{} — {}",
                            l(lang, "Melhor XI", "Best XI"),
                            t_name(tactic, lang)
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(l(
                            lang,
                            "Clique numa instrução recomendada para a aplicar ou remover na tática e na Ficha FM27.",
                            "Click any recommended instruction to toggle it in your tactic and FM27 Sheet.",
                        )),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("xi-auto-pick")
                        .outline()
                        .small()
                        .label(l(lang, "Escolher XI automaticamente", "Auto-pick XI"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            auto_pick_xi(&mut this.state.tactic, &this.state.squad);
                            sync_animated_pitch(this, cx);
                            this.state.persist();
                            this.state.set_toast(
                                this.state.lang.pick(
                                    "XI escolhido pela melhor adequação.",
                                    "XI picked by best suitability.",
                                ),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("xi-apply-all-pis")
                        .primary()
                        .small()
                        .label(l(
                            lang,
                            "Aplicar a principal PI de cada jogador",
                            "Apply top PI per player",
                        ))
                        .on_click(cx.listener(|this, _, _, cx| {
                            let lang = this.state.lang;
                            let pm: HashMap<String, Player> = this
                                .state
                                .squad
                                .iter()
                                .map(|p| (p.id.clone(), p.clone()))
                                .collect();
                            let ip_form = this.state.tactic.ip_form.clone();
                            let mut added = 0usize;
                            for s in &mut this.state.tactic.slots {
                                let Some(pid) = &s.pid else { continue };
                                let Some(p) = pm.get(pid) else { continue };
                                let Some(a) = slot_info(&ip_form, &s.ip_code) else { continue };
                                let recs = recommend_pis(
                                    p,
                                    role_by_id(&s.ip),
                                    role_by_id(&s.oop),
                                    a.group,
                                    lang,
                                );
                                if let Some(first) = recs.first() {
                                    if !s.pis.iter().any(|x| x == first.pi) {
                                        s.pis.push(first.pi.to_string());
                                        added += 1;
                                    }
                                }
                            }
                            this.state.persist();
                            this.state.set_toast(
                                match lang {
                                    Lang::Pt => format!("{} instruções individuais aplicadas.", added),
                                    Lang::En => format!("{} player instructions applied.", added),
                                },
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                ),
        );

    let pitch = interactive_pitch(
        tactic,
        squad,
        Some(&slot_scores),
        PitchPhaseMode::Ip,
        app.selected_slot,
        lang,
        cx.listener(|this, mode: &PitchPhaseMode, _, cx| {
            this.pitch_mode = *mode;
            cx.notify();
        }),
        cx.listener(|this, idx: &usize, _, cx| {
            this.selected_slot = Some(*idx);
            cx.notify();
        }),
        cx,
    );

    let mut xi_table = div()
        .v_flex()
        .flex_1()
        .min_w(px(520.0))
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(theme.border)
                .bg(theme.background)
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(div().w(px(46.0)).child(l(lang, "Pos", "Pos")))
                .child(
                    div()
                        .w(px(210.0))
                        .child(l(lang, "Jogador & papéis", "Player & roles")),
                )
                .child(
                    div()
                        .flex_1()
                        .child(l(lang, "PIs recomendadas", "Recommended PIs")),
                )
                .child(div().w(px(140.0)).child(l(lang, "Suplente", "Backup")))
                .child(
                    div()
                        .w(px(145.0))
                        .child(l(lang, "Sucessor (≤ 21)", "Successor (≤ 21)")),
                ),
        );

    for (i, s) in tactic.slots.iter().enumerate() {
        let p = s.pid.as_deref().and_then(|pid| pmap.get(pid).copied());
        let ri = role_by_id(&s.ip);
        let ro = role_by_id(&s.oop);
        let a = slot_info(&tactic.ip_form, &s.ip_code);
        let sc = slot_scores.get(i).copied().unwrap_or(0);
        let alt = match (p, &a) {
            (Some(pl), Some(meta)) => {
                best_roles_for_player(pl, meta.group, Phase::Ip, Some(&meta.code), meta.wide_cb, 1)
                    .into_iter()
                    .next()
            }
            _ => None,
        };
        let cur_ip_sc = match (p, ri) {
            (Some(pl), Some(r)) => role_score(pl, r).map_or(0, |rf| rf.score),
            _ => 0,
        };
        let recs = match (p, &a) {
            (Some(pl), Some(meta)) => recommend_pis(pl, ri, ro, meta.group, lang),
            _ => Vec::new(),
        };
        let bench_info = benches.get(i);

        let ip_label = ri
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| s.ip.clone());
        let oop_label = ro
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| s.oop.clone());

        let alt_hint = alt.and_then(|cand| {
            if cand.role_id != s.ip && cand.score >= cur_ip_sc.saturating_add(5) {
                let alt_name = role_by_id(cand.role_id)
                    .map(|r| role_name(r, lang))
                    .unwrap_or_else(|| cand.role_id.to_string());
                Some(format!(
                    "{} {} ({})",
                    l(lang, "melhor como", "better as"),
                    alt_name,
                    cand.score
                ))
            } else {
                None
            }
        });

        let mut pi_col = div().h_flex().flex_wrap().flex_1().gap_1();
        if recs.is_empty() {
            pi_col = pi_col.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("—"),
            );
        } else {
            for (rj, rec) in recs.iter().enumerate() {
                let applied = s.pis.iter().any(|x| x == rec.pi);
                let pi_key = rec.pi.to_string();
                let label = format!(
                    "{} {}",
                    if applied { "✓" } else { "+" },
                    pi_name(rec.pi, lang)
                );
                pi_col = pi_col.child(
                    Button::new(("xi-pi", i * 16 + rj))
                        .xsmall()
                        .when(applied, |b| b.primary())
                        .when(!applied, |b| b.outline())
                        .label(label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(slot) = this.state.tactic.slots.get_mut(i) {
                                if let Some(pos) = slot.pis.iter().position(|x| x == &pi_key) {
                                    slot.pis.remove(pos);
                                } else {
                                    slot.pis.push(pi_key.clone());
                                }
                                this.state.persist();
                                cx.notify();
                            }
                        })),
                );
            }
        }

        let bench_cell = match bench_info.and_then(|b| b.bench.as_ref()) {
            Some(c) => div()
                .h_flex()
                .items_center()
                .gap_1()
                .w(px(140.0))
                .child(fit_badge(c.score))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.foreground)
                        .truncate()
                        .child(c.player_name.clone()),
                ),
            None => div()
                .w(px(140.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("—"),
        };

        let succ_cell = match bench_info.and_then(|b| b.succ.as_ref()) {
            Some(c) => div()
                .h_flex()
                .items_center()
                .gap_1()
                .w(px(145.0))
                .child(fit_badge(c.score))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.foreground)
                        .truncate()
                        .child(format!(
                            "{}{}",
                            c.player_name,
                            c.age
                                .map(|ag| format!(" ({})", ag.round() as i32))
                                .unwrap_or_default()
                        )),
                ),
            None => div()
                .w(px(145.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("—"),
        };

        let player_cell = div()
            .v_flex()
            .w(px(210.0))
            .gap(px(2.0))
            .child(match p {
                Some(pl) => div()
                    .h_flex()
                    .items_center()
                    .gap_1()
                    .child(fit_badge(sc))
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .truncate()
                            .child(pl.name.clone()),
                    )
                    .when_some(pl.age, |d, ag| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("{}a", ag.round() as i32)),
                        )
                    }),
                None => div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(l(lang, "vazio", "empty")),
            })
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .truncate()
                    .child(format!("{} / {}", ip_label, oop_label)),
            )
            .when_some(alt_hint, |d, hint| {
                d.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xf59e0b))
                        .truncate()
                        .child(hint),
                )
            });

        xi_table = xi_table.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .w(px(46.0))
                        .text_xs()
                        .font_bold()
                        .text_color(theme.foreground)
                        .child(s.ip_code.clone()),
                )
                .child(player_cell)
                .child(pi_col)
                .child(bench_cell)
                .child(succ_cell),
        );
    }

    // ------------------------------------------------------------------------
    // Systems Ranked for Your Squad (`suggest_tactics`)
    // ------------------------------------------------------------------------
    let suggestions = suggest_tactics(squad, lang);
    let mut sugg_list = div().v_flex().gap_3();
    for (idx, s) in suggestions.into_iter().enumerate() {
        let tp = template_by_id(s.template_id);
        let name = tp
            .map(|x| t(lang, x.name_pt))
            .unwrap_or_else(|| s.tactic.name.clone());
        let desc = tp.map(|x| t(lang, x.desc_pt)).unwrap_or_default();
        let meta_line = format!(
            "{} → {} · {}",
            s.tactic.ip_form,
            s.tactic.oop_form,
            mentality_name(s.tactic.ment, lang)
        );
        let strong_str: Vec<String> = s
            .strongest
            .iter()
            .map(|sl| {
                let rn = role_by_id(&sl.ip_role)
                    .map(|r| role_name(r, lang))
                    .unwrap_or_else(|| sl.ip_role.clone());
                format!(
                    "{} {} {} ({})",
                    sl.player_name.as_deref().unwrap_or("?"),
                    l(lang, "como", "as"),
                    rn,
                    sl.score
                )
            })
            .collect();
        let weak_str: Vec<String> = s
            .weakest
            .iter()
            .map(|sl| {
                format!(
                    "{} {} {} ({})",
                    sl.player_name.as_deref().unwrap_or("?"),
                    l(lang, "em", "at"),
                    sl.ip_code,
                    sl.score
                )
            })
            .collect();

        let tactic_clone = s.tactic.clone();
        sugg_list = sugg_list.child(
            div()
                .v_flex()
                .gap_3()
                .p_4()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .bg(theme.secondary)
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .child(
                            div()
                                .v_flex()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(badge_pill(
                                            format!("#{}", idx + 1),
                                            if idx == 0 {
                                                BadgeTone::Good
                                            } else {
                                                BadgeTone::Muted
                                            },
                                            cx,
                                        ))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_bold()
                                                .text_color(theme.foreground)
                                                .child(name),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(meta_line),
                                )
                                .when(!desc.is_empty(), |d| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child(desc),
                                    )
                                }),
                        )
                        .child(score_ring(s.total, 62.0, cx)),
                )
                .child(
                    div()
                        .v_flex()
                        .max_w(px(460.0))
                        .gap_1()
                        .child(suitability_bar("XI", s.xi_avg, 100, cx))
                        .child(suitability_bar(
                            l(lang, "Estilo", "Style"),
                            s.fit,
                            100,
                            cx,
                        ))
                        .child(suitability_bar(
                            l(lang, "Profundidade", "Depth"),
                            s.depth,
                            100,
                            cx,
                        ))
                        .child(suitability_bar(
                            l(lang, "Coerência", "Coherence"),
                            s.audit.overall,
                            100,
                            cx,
                        )),
                )
                .child(
                    div()
                        .v_flex()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.foreground)
                                .child(format!(
                                    "{}: {}",
                                    l(lang, "Pontos fortes", "Strongest"),
                                    strong_str.join(" · ")
                                )),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{}: {} {}",
                                    l(lang, "Pontos fracos", "Weakest"),
                                    weak_str.join(" · "),
                                    l(
                                        lang,
                                        "— alvo de recrutamento/treino.",
                                        "— recruitment/training target."
                                    )
                                )),
                        ),
                )
                .child(
                    div().h_flex().gap_2().child(
                        Button::new(("xi-use-sugg", idx))
                            .primary()
                            .small()
                            .label(l(lang, "Abrir no analisador", "Open in analyser"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                apply_tactic_to_app(this, tactic_clone.clone(), true, cx);
                            })),
                    ),
                ),
        );
    }

    let systems_section = collapsible_card(
        "xi-card-systems",
        l(
            lang,
            "Sistemas ordenados para o seu plantel",
            "Systems ranked for your squad",
        ),
        Some(
            l(
                lang,
                "Pontuação = adequação do XI (55%) + estilo (20%) + profundidade (15%) + coerência (10%)",
                "Score = XI suitability (55%) + style fit (20%) + depth (15%) + coherence (10%)",
            )
            .into(),
        ),
        app.state.is_panel_collapsed("xi-systems"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("xi-systems");
            cx.notify();
        }),
        sugg_list,
        cx,
    );

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_start()
                .gap_4()
                .child(pitch)
                .child(xi_table),
        )
        .child(systems_section)
}

// ============================================================================
// 3. My Team › Opponent (`render_team_opponent`)
// ============================================================================

pub fn render_team_opponent(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let opp = &app.state.opponent;
    let squad = &app.state.squad;
    let tactic = &app.state.tactic;

    if opp.is_empty() {
        let left_card = div()
            .v_flex()
            .flex_1()
            .min_w(px(300.0))
            .gap_3()
            .p_4()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary)
            .child(
                div()
                    .text_base()
                    .font_bold()
                    .text_color(theme.foreground)
                    .child(l(lang, "Relatório do adversário", "Opponent report")),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(l(
                        lang,
                        "Carregue o plantel do próximo adversário para comparar 9 setores com o seu XI, identificar os 3 mais perigosos e aplicar ajustes com 1 clique.",
                        "Load your next opponent's squad to compare 9 units against your XI, highlight their 3 biggest threats, and apply tactical adjustments in 1 click.",
                    )),
            )
            .child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("opp-empty-fm")
                            .primary()
                            .small()
                            .label(l(lang, "Próximo adversário do FM", "Next opponent from FM"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                spawn_next_opponent_from_fm(this, cx);
                            })),
                    )
                    .child(
                        Button::new("opp-empty-file")
                            .outline()
                            .small()
                            .label(l(
                                lang,
                                "Escolher ficheiro do adversário (CSV/HTML)",
                                "Import opponent file (CSV/HTML)",
                            ))
                            .on_click(cx.listener(|this, _, _, cx| {
                                import_squad_from_picker(this, true, cx);
                            })),
                    ),
            );

        let bullets = [
            l(
                lang,
                "• Velocidade e jogo aéreo dos centrais deles contra os seus avançados.",
                "• Their centre-backs' pace and aerial strength vs your forwards.",
            ),
            l(
                lang,
                "• Laterais, criação no meio-campo, pressão, extremos, finalização e ataque aéreo.",
                "• Full-backs, midfield creativity, pressing, wingers, finishing, and aerial attack.",
            ),
            l(
                lang,
                "• Os 3 jogadores mais perigosos e as instruções de adversário recomendadas.",
                "• Their 3 most dangerous players with recommended opposition instructions.",
            ),
            l(
                lang,
                "• Plano de ajustes aplicável à tática com um clique e visível no campo animado.",
                "• One-click tactical adjustment plan ready to preview on the animated pitch.",
            ),
        ];

        let mut right_card = div()
            .v_flex()
            .flex_1()
            .min_w(px(300.0))
            .gap_2()
            .p_4()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary)
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(l(lang, "O que vai ver", "What you'll see")),
            );
        for b in bullets {
            right_card = right_card.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(b),
            );
        }

        return div()
            .h_flex()
            .flex_wrap()
            .items_stretch()
            .w_full()
            .gap_4()
            .child(left_card)
            .child(right_card);
    }

    let rep = opponent_report(tactic, squad, opp, lang);
    let club_label = rep
        .club
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| app.state.opponent_source.clone());

    let header = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(theme.foreground)
                        .child(format!(
                            "{}: {}",
                            l(lang, "Relatório", "Report"),
                            club_label
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(match lang {
                            Lang::Pt => format!(
                                "{} jogadores · onze provável (4-2-3-1). {}",
                                opp.len(),
                                if squad.is_empty() {
                                    "Carregue o seu plantel para comparar."
                                } else {
                                    "Comparado com o seu XI."
                                }
                            ),
                            Lang::En => format!(
                                "{} players · probable XI (4-2-3-1). {}",
                                opp.len(),
                                if squad.is_empty() {
                                    "Load your squad to compare."
                                } else {
                                    "Compared against your XI."
                                }
                            ),
                        }),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("opp-refresh-fm")
                        .outline()
                        .small()
                        .label(l(lang, "Atualizar do FM", "Update from FM"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_next_opponent_from_fm(this, cx);
                        })),
                )
                .child(
                    Button::new("opp-import-other")
                        .outline()
                        .small()
                        .label(l(lang, "Importar outro", "Import another"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            import_squad_from_picker(this, true, cx);
                        })),
                )
                .child(
                    Button::new("opp-clear")
                        .ghost()
                        .small()
                        .label(l(lang, "Limpar adversário", "Clear opponent"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.opponent.clear();
                            this.state.opponent_source.clear();
                            this.state.persist();
                            cx.notify();
                        })),
                ),
        );

    // ------------------------------------------------------------------------
    // Left column: 9-unit sector comparison
    // ------------------------------------------------------------------------
    let mut sectors_body = div().v_flex().gap_2();
    for s in &rep.sectors {
        let tv = s.them_val.unwrap_or(0.0);
        let pct = ((tv - 1.0) / 19.0).clamp(0.0, 1.0);
        let bar_col: Hsla = if tv >= 14.0 {
            rgb(0xdc2626).into()
        } else if tv <= 10.5 {
            rgb(0x16a34a).into()
        } else {
            rgb(0xd97706).into()
        };
        let us_marker = s.us_val.map(|uv| ((uv - 1.0) / 19.0).clamp(0.0, 1.0));
        let cmp_txt = match (s.them_val, s.us_val) {
            (Some(t_v), Some(u_v)) => {
                let d = t_v - u_v;
                format!("{:.1} ({:+.1})", t_v, d)
            }
            (Some(t_v), None) => format!("{:.1}", t_v),
            _ => "—".to_string(),
        };

        sectors_body = sectors_body.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(165.0))
                        .text_xs()
                        .text_color(theme.foreground)
                        .truncate()
                        .child(t(lang, s.name_pt)),
                )
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .h(px(9.0))
                        .rounded_full()
                        .bg(theme.background)
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(relative(pct))
                                .rounded_full()
                                .bg(bar_col),
                        )
                        .when_some(us_marker, |d, u_pct| {
                            d.child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .left(relative(u_pct))
                                    .w(px(3.0))
                                    .bg(theme.foreground),
                            )
                        }),
                )
                .child(
                    div()
                        .w(px(74.0))
                        .text_xs()
                        .font_semibold()
                        .text_right()
                        .text_color(theme.foreground)
                        .child(cmp_txt),
                ),
        );
    }
    sectors_body = sectors_body.child(
        div()
            .pt_2()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(l(
                lang,
                "Vermelho = ponto forte deles · verde = ponto fraco · traço vertical = o seu XI.",
                "Red = their strength · green = their weakness · vertical marker = your XI.",
            )),
    );

    let sectors_card = collapsible_card(
        "opp-card-sectors",
        l(lang, "Setor a setor (9 unidades)", "Sector by sector (9 units)"),
        None,
        app.state.is_panel_collapsed("opp-sectors"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("opp-sectors");
            cx.notify();
        }),
        sectors_body,
        cx,
    );

    // ------------------------------------------------------------------------
    // Right column: Adjustment Plan + Top 3 Dangers
    // ------------------------------------------------------------------------
    let plan_items = rep.plan.clone();
    let mut plan_body = div().v_flex().gap_2();
    if !plan_items.is_empty() {
        let all_plan = plan_items.clone();
        plan_body = plan_body.child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .pb_1()
                .child(
                    Button::new("opp-apply-all")
                        .primary()
                        .small()
                        .label(l(lang, "Aplicar plano completo", "Apply full plan"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            for item in &all_plan {
                                if let Some(def) = ti_by_key(item.ti) {
                                    if item.value == def.def {
                                        this.state.tactic.ti.remove(item.ti);
                                    } else {
                                        this.state.tactic.ti.insert(item.ti.to_string(), item.value);
                                    }
                                }
                            }
                            sync_animated_pitch(this, cx);
                            this.state.persist();
                            this.state.set_toast(
                                this.state.lang.pick(
                                    "Plano de jogo aplicado à tática.",
                                    "Adjustment plan applied to tactic.",
                                ),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("opp-open-sim")
                        .outline()
                        .small()
                        .label(l(lang, "▶ Ver em campo", "▶ View on pitch"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.section = NavSection::Tactic;
                            this.tactic_tab = TacticTab::AnimatedPitch;
                            cx.notify();
                        })),
                ),
        );

        for (pi, item) in plan_items.iter().enumerate() {
            let applied = ti_val(tactic, item.ti) == item.value;
            let ti_title = ti_by_key(item.ti)
                .map(|def| {
                    let opt = def.opts.get(item.value).copied().unwrap_or("");
                    format!("{} → {}", ti_name(def, lang), ti_opt_name(opt, lang))
                })
                .unwrap_or_else(|| item.ti.to_string());
            let ti_key = item.ti;
            let ti_val_target = item.value;
            let toast_title = ti_title.clone();

            plan_body = plan_body.child(
                div()
                    .h_flex()
                    .items_start()
                    .gap_2()
                    .p_2()
                    .rounded_sm()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.background)
                    .child(badge_pill(
                        if applied {
                            "✓".to_string()
                        } else {
                            format!("{}", pi + 1)
                        },
                        if applied {
                            BadgeTone::Good
                        } else {
                            BadgeTone::Info
                        },
                        cx,
                    ))
                    .child(
                        div()
                            .v_flex()
                            .flex_1()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(theme.foreground)
                                    .child(ti_title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(item.why.clone()),
                            ),
                    )
                    .when(!applied, |d| {
                        d.child(
                            Button::new(("opp-apply-one", pi))
                                .outline()
                                .xsmall()
                                .label(l(lang, "Aplicar", "Apply"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Some(def) = ti_by_key(ti_key) {
                                        if ti_val_target == def.def {
                                            this.state.tactic.ti.remove(ti_key);
                                        } else {
                                            this.state
                                                .tactic
                                                .ti
                                                .insert(ti_key.to_string(), ti_val_target);
                                        }
                                    }
                                    sync_animated_pitch(this, cx);
                                    this.state.persist();
                                    this.state.set_toast(toast_title.clone(), ToastKind::Success);
                                    cx.notify();
                                })),
                        )
                    }),
            );
        }
    } else {
        plan_body = plan_body.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(l(
                    lang,
                    "Sem diferenças marcadas entre as equipas: mantenha o seu plano.",
                    "No major gaps between the two squads: stick to your base plan.",
                )),
        );
    }

    // Top 3 Dangerous Players + OIs
    let mut dangers_body = div().v_flex().gap_2();
    for d in &rep.dangers {
        let mut oi_row = div().h_flex().flex_wrap().gap_1();
        for oi in &d.ois {
            let lbl = match lang {
                Lang::Pt => pt_terms(oi),
                Lang::En => (*oi).to_string(),
            };
            oi_row = oi_row.child(badge_pill(lbl, BadgeTone::Warn, cx));
        }
        dangers_body = dangers_body.child(
            div()
                .h_flex()
                .items_start()
                .gap_2()
                .p_2()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(badge_pill("!", BadgeTone::Crit, cx))
                .child(
                    div()
                        .v_flex()
                        .flex_1()
                        .gap_1()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(theme.foreground)
                                        .child(d.player_name.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(d.code.clone()),
                                ),
                        )
                        .child(oi_row),
                ),
        );
    }

    let plan_card = collapsible_card(
        "opp-card-plan",
        l(lang, "Plano de jogo & ameaças", "Game plan & top threats"),
        None,
        app.state.is_panel_collapsed("opp-plan"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("opp-plan");
            cx.notify();
        }),
        div()
            .v_flex()
            .gap_3()
            .child(plan_body)
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .pt_2()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(l(
                                lang,
                                "Jogadores mais perigosos (OIs)",
                                "Most dangerous players (OIs)",
                            )),
                    )
                    .child(dangers_body),
            ),
        cx,
    );

    // Probable XI strip
    let mut xi_chips = div().h_flex().flex_wrap().gap_2();
    for m in &rep.them {
        xi_chips = xi_chips.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .px_2()
                .py(px(3.0))
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(
                    div()
                        .text_xs()
                        .font_bold()
                        .text_color(theme.primary)
                        .child(m.code.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.foreground)
                        .child(m.player_name.clone()),
                ),
        );
    }
    let xi_card = collapsible_card(
        "opp-card-xi",
        l(lang, "Onze provável do adversário", "Opponent probable XI"),
        None,
        app.state.is_panel_collapsed("opp-xi"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("opp-xi");
            cx.notify();
        }),
        xi_chips,
        cx,
    );

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_start()
                .gap_4()
                .child(div().flex_1().min_w(px(320.0)).child(sectors_card))
                .child(div().flex_1().min_w(px(320.0)).child(plan_card)),
        )
        .child(xi_card)
}

/// Convenience dispatcher for the 3 Team tabs owned by `team.rs` (`Squad`, `BestXi`, `Opponent`).
/// `TeamTab::Generator` is rendered by `views::generator`.
pub fn render_team_tab(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> AnyElement {
    match app.team_tab {
        TeamTab::Squad => render_team_squad(app, cx).into_any_element(),
        TeamTab::BestXi => render_team_best_xi(app, cx).into_any_element(),
        TeamTab::Opponent => render_team_opponent(app, cx).into_any_element(),
        TeamTab::Generator => empty_state_card(
            TeamTab::Generator.label(app.state.lang),
            l(
                app.state.lang,
                "Ligado via views::generator.",
                "Wired via views::generator.",
            ),
            None,
            cx,
        )
        .into_any_element(),
    }
}
