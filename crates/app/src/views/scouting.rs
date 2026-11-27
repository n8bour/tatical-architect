//! Scouting view: 100k+ player database loaded via `export_save_db` or CSV picker,
//! 11 preset pills, 5 collapsible filter groups (incl. dynamic `+ Attribute` rows),
//! Results vs Shortlist tabs, sortable paginated table, CSV export, Player Profile
//! overlay modal, Shortlist -> FM push, and Scout in FM Recruitment Focus modal.

use crate::state::{non_empty, BackendBridgeTransport, ToastKind};
use crate::widgets::common::{
    attr_chip, badge_pill, collapsible_card, empty_state_card, head_font, kpi_card, page_title,
    panel, score_color, BadgeTone, RADIUS,
};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme, Disableable, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use ta_backend::{export_save_db, latest_save, pick_open_file, pick_save_file};
use ta_core::data::{attr_name, group_name, role_by_id, role_name, ROLES};
use ta_core::live::fmbridge::FmbClient;
use ta_core::planning::{
    best_gain, best_roles_any, game_date_from_info, infer_game_date, join_likelihood,
    parse_nations, parse_scout_db, reach_likelihood, scout_filter, scout_presets,
};
use ta_core::types::{
    Attr, AttrCat, AttrLookup, BridgeFocusArea, BridgeFocusRequest, Foot, Lang, LoanFilter, Phase,
    PosGroup, PosLevel, ScoutAttrFilter, ScoutContext, ScoutDb, ScoutFilter, ScoutHid, ScoutMatch,
    ScoutPers, ScoutPlayer, ScoutSort, ShortlistEntry, SortDir,
};

const PAGE_SIZE: usize = 50;

/// Modal state for creating an in-game FM27 Recruitment Focus from current search results.
#[derive(Debug, Clone)]
pub struct FocusModalState {
    pub name: String,
    pub age_min: u16,
    pub age_max: u16,
    pub ca_stars: f32,
    pub pa_stars: f32,
    pub selected_nations: HashSet<i32>,
}

/// Persistent `InputState` handles for Scouting search text fields.
pub struct ScoutingInputs {
    pub search_q: Entity<InputState>,
    pub club_q: Entity<InputState>,
    pub focus_name: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl ScoutingInputs {
    pub fn new(window: &mut Window, cx: &mut Context<ArchitectApp>) -> Self {
        let search_q = cx.new(|cx| InputState::new(window, cx).placeholder("Search player name…"));
        let club_q = cx.new(|cx| InputState::new(window, cx).placeholder("Club name…"));
        let focus_name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tactical Architect")
                .default_value("Tactical Architect")
        });

        let mut subs = Vec::new();
        subs.push(cx.subscribe(&search_q, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let v = entity.read(cx).value().to_string();
                if this.scout_ui.filter.q != v {
                    this.scout_ui.filter.q = v;
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                }
            }
        }));
        subs.push(cx.subscribe(&club_q, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let v = entity.read(cx).value().to_string();
                if this.scout_ui.filter.club != v {
                    this.scout_ui.filter.club = v;
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                }
            }
        }));
        subs.push(cx.subscribe(&focus_name, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                if let Some(m) = &mut this.scout_ui.focus_modal {
                    m.name = entity.read(cx).value().chars().take(25).collect();
                }
            }
        }));

        Self {
            search_q,
            club_q,
            focus_name,
            _subs: subs,
        }
    }

    pub fn sync_filter(&self, f: &ScoutFilter, window: &mut Window, cx: &mut App) {
        let q = f.q.clone();
        let club = f.club.clone();
        self.search_q.update(cx, |st, cx| {
            if st.value().as_ref() != q.as_str() {
                st.set_value(q, window, cx);
            }
        });
        self.club_q.update(cx, |st, cx| {
            if st.value().as_ref() != club.as_str() {
                st.set_value(club, window, cx);
            }
        });
    }
}

/// View-local UI state for Scouting.
pub struct ScoutingViewState {
    pub filter: ScoutFilter,
    pub preset: String,
    pub page: usize,
    pub show_shortlist: bool,
    pub open_player_id: Option<String>,
    pub focus_modal: Option<FocusModalState>,
    pub last_err: Option<String>,
    pub inputs: Option<ScoutingInputs>,
    pub cached_matches: Vec<ScoutMatch>,
    pub cache_dirty: bool,
}

impl Default for ScoutingViewState {
    fn default() -> Self {
        Self {
            filter: ScoutFilter::default(),
            preset: String::new(),
            page: 0,
            show_shortlist: false,
            open_player_id: None,
            focus_modal: None,
            last_err: None,
            inputs: None,
            cached_matches: Vec::new(),
            cache_dirty: true,
        }
    }
}

impl ScoutingViewState {
    pub fn invalidate_cache(&mut self) {
        self.cache_dirty = true;
    }
}

fn percentile_i64(mut vals: Vec<i64>, q: f32) -> Option<i64> {
    if vals.is_empty() {
        return None;
    }
    vals.sort_unstable();
    let idx = ((vals.len() as f32 * q) as usize).min(vals.len() - 1);
    Some(vals[idx])
}

/// Builds a `ScoutContext` from current app state and loaded `ScoutDb`.
pub fn build_scout_ctx<'a>(app: &'a ArchitectApp) -> ScoutContext<'a> {
    let squad = &app.state.squad;
    let pm: HashMap<&str, &ta_core::types::Player> =
        squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let xi_cas: Vec<f32> = app
        .state
        .tactic
        .slots
        .iter()
        .filter_map(|s| s.pid.as_deref().and_then(|id| pm.get(id)))
        .filter_map(|p| p.ca)
        .collect();
    let xi_ca = if xi_cas.is_empty() {
        120
    } else {
        (xi_cas.iter().sum::<f32>() / xi_cas.len() as f32).round() as u16
    };

    let (date, cheap_value, bargain_value, my_rep) = if let Some(db) = &app.state.scout_db {
        let cheap = percentile_i64(
            db.players
                .iter()
                .filter(|p| p.age.unwrap_or(99) <= 19 && p.pa.unwrap_or(0) >= 150)
                .filter_map(|p| p.value)
                .collect(),
            0.35,
        )
        .or(Some(3_000_000));

        let bargain = percentile_i64(
            db.players
                .iter()
                .filter(|p| {
                    let age = p.age.unwrap_or(0);
                    age >= 21 && age <= 29 && p.ca.unwrap_or(0) + 5 >= xi_ca
                })
                .filter_map(|p| p.value)
                .collect(),
            0.30,
        );

        let my_club_low = app.state.my_club.trim().to_lowercase();
        let rep = if my_club_low.is_empty() {
            None
        } else {
            let mut reps: Vec<i32> = db
                .players
                .iter()
                .filter(|p| p.club_lower == my_club_low)
                .filter_map(|p| p.club_rep)
                .collect();
            if reps.is_empty() {
                None
            } else {
                reps.sort_unstable();
                Some(reps[reps.len() / 2])
            }
        };
        (db.date.as_deref(), cheap, bargain, rep)
    } else {
        (None, Some(3_000_000), None, None)
    };

    ScoutContext {
        date,
        my_club: non_empty(&app.state.my_club),
        tactic: Some(&app.state.tactic),
        squad,
        xi_ca,
        cheap_value,
        bargain_value,
        my_rep,
    }
}

fn fmt_money(v: Option<i64>) -> String {
    let Some(n) = v else {
        return "—".into();
    };
    let abs = n.abs() as f64;
    if abs >= 1_000_000.0 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn to_shortlist_entry(p: &ScoutPlayer) -> ShortlistEntry {
    let mut pos = Vec::new();
    let mut nat_pos = Vec::new();
    for g in PosGroup::ALL {
        if p.has_pos_group(g) {
            pos.push(g.code().to_string());
        }
        if p.has_nat_group(g) {
            nat_pos.push(g.code().to_string());
        }
    }
    ShortlistEntry {
        id: p.id.clone(),
        uniq: p.uniq.clone().or_else(|| p.uid.clone()),
        nat: p.nat,
        name: p.name.clone(),
        age: p.age,
        club: p.club.clone(),
        pos_raw: p.pos_raw.clone(),
        ca: p.ca,
        pa: p.pa,
        value: p.value,
        wage: p.wage,
        end: p.end.clone(),
        pos,
        nat_pos,
    }
}

/// Loads the entire save DB via `ta_backend::export_save_db` + `parse_scout_db` off-thread.
pub fn spawn_load_scout_db(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let custom = non_empty(&app.state.custom_save_dir).map(str::to_string);
    app.state.set_busy(
        lang.pick(
            "A ler todos os jogadores do save (fmsave)…",
            "Reading all players from save (fmsave)…",
        ),
        Some(0.25),
    );
    app.scout_ui.last_err = None;
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res: Result<ScoutDb, String> = cx
            .background_executor()
            .spawn(async move {
                let info = latest_save(custom.as_deref()).ok_or_else(|| {
                    lang.pick(
                        "Nenhum save .fm encontrado nas pastas habituais.",
                        "No .fm save found in standard folders.",
                    )
                    .to_string()
                })?;
                let raw = export_save_db(&info.path)?;
                let mut db = parse_scout_db(&raw.players, lang, |_, _| {})?;
                db.nations = parse_nations(&raw.nations);
                if let Some(d) = game_date_from_info(&raw.info) {
                    db.date = Some(d);
                    db.date_guess = false;
                } else {
                    db.date = infer_game_date(&db.players);
                    db.date_guess = db.date.is_some();
                }
                db.save = raw.save;
                db.cached = raw.cached;
                Ok(db)
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            match res {
                Ok(db) => {
                    let count = db.players.len();
                    this.state.scout_db = Some(db);
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    this.state.set_toast(
                        format!(
                            "{} {}",
                            count,
                            this.state
                                .lang
                                .pick("jogadores carregados.", "players loaded.")
                        ),
                        ToastKind::Success,
                    );
                }
                Err(err) => {
                    this.scout_ui.last_err = Some(err.clone());
                    this.state.set_toast(err, ToastKind::Error);
                }
            }
            cx.notify();
        });
    })
    .detach();
}

/// fmsave export with every player in the save (thousands of rows): belongs to Scouting, not the squad.
pub fn is_full_save_export(text: &str) -> bool {
    let head_end = text.char_indices().nth(5000).map_or(text.len(), |(i, _)| i);
    text.len() > 100_000
        && text[..head_end].contains("ability_current")
        && text.bytes().filter(|&b| b == b'\n').count() > 400
}

/// Imports a scouting CSV file via native file picker off-thread.
pub fn spawn_import_scout_csv(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let Some(path) = pick_open_file(
        lang.pick("Importar CSV de prospeção", "Import scouting CSV"),
        &[("CSV", &["csv", "txt"])],
    ) else {
        return;
    };
    spawn_ingest_scout_path(app, path, cx);
}

/// Parses a scouting CSV (fmsave full export) off-thread into `state.scout_db`.
pub fn spawn_ingest_scout_path(
    app: &mut ArchitectApp,
    path: std::path::PathBuf,
    cx: &mut Context<ArchitectApp>,
) {
    let lang = app.state.lang;
    app.state.set_busy(
        lang.pick("A analisar CSV de prospeção…", "Parsing scouting CSV…"),
        Some(0.4),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res: Result<ScoutDb, String> = cx
            .background_executor()
            .spawn(async move {
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let mut db = parse_scout_db(&text, lang, |_, _| {})?;
                db.date = infer_game_date(&db.players);
                db.date_guess = db.date.is_some();
                db.save = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                Ok(db)
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            match res {
                Ok(db) => {
                    let count = db.players.len();
                    this.state.scout_db = Some(db);
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    this.state.set_toast(
                        format!(
                            "{} {}",
                            count,
                            this.state
                                .lang
                                .pick("jogadores importados.", "players imported.")
                        ),
                        ToastKind::Success,
                    );
                }
                Err(err) => {
                    this.scout_ui.last_err = Some(err.clone());
                    this.state.set_toast(err, ToastKind::Error);
                }
            }
            cx.notify();
        });
    })
    .detach();
}

fn spawn_push_shortlist_to_fm(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let list = app.state.shortlist_entries.clone();
    if list.is_empty() {
        return;
    }
    app.state.set_busy(
        lang.pick("A enviar shortlist para o FM27…", "Pushing shortlist to FM27…"),
        Some(0.35),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.push_shortlist(&list, |_, _| {})
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            match res {
                Ok(r) => this.state.set_toast(
                    format!(
                        "{}: {} · {}: {}",
                        this.state.lang.pick("Adicionados no FM", "Added in FM"),
                        r.added.len(),
                        this.state.lang.pick("não encontrados", "missing"),
                        r.missing.len()
                    ),
                    ToastKind::Success,
                ),
                Err(e) => this.state.set_toast(e.to_string(), ToastKind::Error),
            }
            cx.notify();
        });
    })
    .detach();
}

fn spawn_create_focus_in_fm(
    app: &mut ArchitectApp,
    req: BridgeFocusRequest,
    cx: &mut Context<ArchitectApp>,
) {
    let lang = app.state.lang;
    app.state.set_busy(
        lang.pick(
            "A criar Recruitment Focus no FM27…",
            "Creating Recruitment Focus in FM27…",
        ),
        Some(0.45),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.create_focus(&req, |_| {})
            })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            match res {
                Ok(item) => {
                    this.scout_ui.focus_modal = None;
                    this.state.set_toast(
                        format!(
                            "{}: {}",
                            this.state
                                .lang
                                .pick("Recruitment Focus criado no FM27", "Recruitment Focus created in FM27"),
                            item.name
                        ),
                        ToastKind::Success,
                    );
                }
                Err(e) => this.state.set_toast(e.to_string(), ToastKind::Error),
            }
            cx.notify();
        });
    })
    .detach();
}

fn export_matches_csv(db: &ScoutDb, matches: &[ScoutMatch]) -> String {
    let mut lines = Vec::with_capacity(matches.len() + 1);
    lines.push("name,age,pos,club,ca,pa,value,wage,contract_end".to_string());
    for m in matches {
        if let Some(p) = db.players.get(m.index) {
            let esc = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
            lines.push(format!(
                "{},{},{},{},{},{},{},{},{}",
                esc(&p.name),
                p.age.map(|v| v.to_string()).unwrap_or_default(),
                esc(&p.pos_raw),
                esc(&p.club),
                p.ca.map(|v| v.to_string()).unwrap_or_default(),
                p.pa.map(|v| v.to_string()).unwrap_or_default(),
                p.value.map(|v| v.to_string()).unwrap_or_default(),
                p.wage.map(|v| v.to_string()).unwrap_or_default(),
                esc(&p.end),
            ));
        }
    }
    lines.join("\n")
}

/// Ensures cached scouting matches are up to date (call before rendering).
pub fn refresh_scout_cache_if_needed(app: &mut ArchitectApp) {
    if !app.scout_ui.cache_dirty {
        return;
    }
    if let Some(db) = &app.state.scout_db {
        let ctx = build_scout_ctx(app);
        let matches = scout_filter(db, &app.scout_ui.filter, &ctx, app.state.lang);
        app.scout_ui.cached_matches = matches;
    } else {
        app.scout_ui.cached_matches.clear();
    }
    app.scout_ui.cache_dirty = false;
}

/// Renders the Scouting screen.
pub fn render_scouting(
    app: &mut ArchitectApp,
    window: &mut Window,
    cx: &mut Context<ArchitectApp>,
) -> impl IntoElement {
    if app.scout_ui.inputs.is_none() {
        app.scout_ui.inputs = Some(ScoutingInputs::new(window, cx));
    }
    refresh_scout_cache_if_needed(app);

    let lang = app.state.lang;
    let theme = cx.theme().clone();

    let Some(db) = app.state.scout_db.as_ref() else {
        let mut actions = div()
            .h_flex()
            .flex_wrap()
            .justify_center()
            .gap_2()
            .child(
                Button::new("scout-load-save")
                    .primary()
                    .small()
                    .label(lang.pick(
                        "Ler todos os jogadores do save",
                        "Read every player in save",
                    ))
                    .on_click(cx.listener(|this, _, _, cx| {
                        spawn_load_scout_db(this, cx);
                    })),
            )
            .child(
                Button::new("scout-import-csv")
                    .outline()
                    .small()
                    .label(lang.pick("Importar CSV…", "Import CSV…"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        spawn_import_scout_csv(this, cx);
                    })),
            );
        if let Some(err) = &app.scout_ui.last_err {
            actions = actions.child(
                div()
                    .w_full()
                    .pt_2()
                    .text_xs()
                    .text_color(theme.warning)
                    .text_center()
                    .child(err.clone()),
            );
        }
        return div()
            .v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(page_title(lang.pick("Prospeção", "Scouting"), cx))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(lang.pick(
                                "Todos os jogadores do save, com atributos, CA/PA, personalidade, contrato e valor.",
                                "Every player in your save, with attributes, CA/PA, personality, contract and value.",
                            )),
                    ),
            )
            .child(empty_state_card(
                lang.pick("Carregar base de dados de prospeção", "Load scouting database"),
                lang.pick(
                    "Lê todos os jogadores do save (100k+) via fmsave com cache automática, ou importe um CSV de jogadores.",
                    "Reads every player from your FM27 save (100k+) via cached fmsave, or import a players CSV directly.",
                ),
                Some(actions.into_any_element()),
                cx,
            ));
    };

    let ctx = build_scout_ctx(app);
    let presets = scout_presets(&ctx, lang);
    let matches = &app.scout_ui.cached_matches;
    let total_matches = matches.len();
    let total_pages = (total_matches + PAGE_SIZE - 1).div_ceil(PAGE_SIZE).max(1);
    let page = app.scout_ui.page.min(total_pages.saturating_sub(1));
    let shortlist_ids: HashSet<&str> = app
        .state
        .shortlist_entries
        .iter()
        .map(|s| s.id.as_str())
        .collect();

    // Top bar
    let meta_str = format!(
        "{} {} · {} · {} {} ({})",
        db.players.len(),
        lang.pick("jogadores", "players"),
        db.save,
        lang.pick("data", "date"),
        db.date.as_deref().unwrap_or("—"),
        if db.cached {
            lang.pick("cache", "cached").to_string()
        } else {
            format!("{:.1}s", db.parse_ms as f64 / 1000.0)
        }
    );

    let top_bar = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_2()
        .child(
            div()
                .v_flex()
                .min_w_0()
                .gap(px(2.0))
                .child(page_title(lang.pick("Prospeção", "Scouting"), cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(meta_str),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("scout-refresh")
                        .outline()
                        .small()
                        .label(lang.pick("Atualizar do save", "Refresh from save"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_load_scout_db(this, cx);
                        })),
                )
                .child(
                    Button::new("scout-csv-in")
                        .outline()
                        .small()
                        .label(lang.pick("Importar CSV", "Import CSV"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_import_scout_csv(this, cx);
                        })),
                )
                .child(
                    Button::new("scout-csv-out")
                        .outline()
                        .small()
                        .label(lang.pick("Exportar CSV", "Export CSV"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            let Some(db) = &this.state.scout_db else {
                                return;
                            };
                            let title = this
                                .state
                                .lang
                                .pick("Exportar resultados CSV", "Export results CSV");
                            if let Some(path) =
                                pick_save_file(title, "scouting_results.csv", &[("CSV", &["csv"])])
                            {
                                let csv = export_matches_csv(db, &this.scout_ui.cached_matches);
                                match std::fs::write(&path, csv) {
                                    Ok(()) => this.state.set_toast(
                                        format!(
                                            "{}: {}",
                                            this.state.lang.pick("CSV exportado", "CSV exported"),
                                            path.display()
                                        ),
                                        ToastKind::Success,
                                    ),
                                    Err(e) => {
                                        this.state.set_toast(e.to_string(), ToastKind::Error);
                                    }
                                }
                                cx.notify();
                            }
                        })),
                )
                .when(!app.state.shortlist_entries.is_empty(), |d| {
                    d.child(
                        Button::new("scout-push-shortlist")
                            .primary()
                            .small()
                            .label(lang.pick("Shortlist → FM", "Shortlist → FM"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                spawn_push_shortlist_to_fm(this, cx);
                            })),
                    )
                })
                .child(
                    Button::new("scout-open-focus")
                        .outline()
                        .small()
                        .label(lang.pick("Observar no FM (Focus)", "Scout in FM (Focus)"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            let mut counts: HashMap<i32, usize> = HashMap::new();
                            if let Some(db) = &this.state.scout_db {
                                for m in &this.scout_ui.cached_matches {
                                    if let Some(nat) = db.players.get(m.index).and_then(|p| p.nat) {
                                        if db.nations.contains_key(&nat) {
                                            *counts.entry(nat).or_insert(0) += 1;
                                        }
                                    }
                                }
                            }
                            let mut top: Vec<(i32, usize)> = counts.into_iter().collect();
                            top.sort_by(|a, b| b.1.cmp(&a.1));
                            let selected_nations: HashSet<i32> =
                                top.into_iter().take(3).map(|(n, _)| n).collect();
                            let f = &this.scout_ui.filter;
                            let age_min = f.age_min.unwrap_or(15).max(15);
                            let age_max = f.age_max.unwrap_or(28).max(age_min);
                            let name = if this.scout_ui.preset.is_empty() {
                                "Tactical Architect".to_string()
                            } else {
                                this.scout_ui.preset.chars().take(25).collect()
                            };
                            if let Some(inp) = &this.scout_ui.inputs {
                                let nm = name.clone();
                                inp.focus_name.update(cx, |s, cx| s.set_value(nm, window, cx));
                            }
                            this.scout_ui.focus_modal = Some(FocusModalState {
                                name,
                                age_min,
                                age_max,
                                ca_stars: 1.0,
                                pa_stars: if f.pa_min.unwrap_or(0) >= 140 { 4.0 } else { 3.0 },
                                selected_nations,
                            });
                            cx.notify();
                        })),
                ),
        );

    // 11 Preset pills
    let mut preset_row = div().h_flex().flex_wrap().gap_1();
    for (i, pre) in presets.iter().enumerate() {
        let key = pre.key.to_string();
        let filt = pre.filter.clone();
        let active = app.scout_ui.preset == pre.key;
        let disabled = (pre.need_squad && app.state.squad.is_empty())
            || (pre.need_my_club && ctx.my_rep.is_none());
        preset_row = preset_row.child(
            Button::new(("scout-preset", i))
                .xsmall()
                .label(format!("{} {}", pre.icon, pre.name))
                .disabled(disabled)
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.scout_ui.preset = key.clone();
                    this.scout_ui.filter = filt.clone();
                    this.scout_ui.page = 0;
                    this.scout_ui.show_shortlist = false;
                    if let Some(inp) = &this.scout_ui.inputs {
                        inp.sync_filter(&this.scout_ui.filter, window, cx);
                    }
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                })),
        );
    }

    // Left Filters Panel (5 collapsible groups)
    let f = &app.scout_ui.filter;
    let inputs = app.scout_ui.inputs.as_ref().unwrap();

    // Group 1: Player
    let mut pos_chips = div().h_flex().flex_wrap().gap_1().child(
        Button::new("sf-pos-all")
            .xsmall()
            .label(lang.pick("Todas", "All"))
            .when(f.pos.is_none(), |b| b.primary())
            .when(f.pos.is_some(), |b| b.outline())
            .on_click(cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.pos = None;
                this.scout_ui.page = 0;
                this.scout_ui.invalidate_cache();
                cx.notify();
            })),
    );
    for g in PosGroup::ALL {
        let active = f.pos == Some(g);
        pos_chips = pos_chips.child(
            Button::new(("sf-pos", g.index()))
                .xsmall()
                .label(g.code())
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.scout_ui.filter.pos = Some(g);
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                })),
        );
    }

    let player_group_body = div()
        .v_flex()
        .gap_2()
        .child(Input::new(&inputs.search_q))
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_1()
                .child(step_filter_btn(
                    "sf-age-max",
                    &format!("{} ≤ {}", lang.pick("Idade", "Age"), f.age_max.map(|v| v.to_string()).unwrap_or_else(|| "∞".into())),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.age_max = match this.scout_ui.filter.age_max {
                            None => Some(23),
                            Some(23) => Some(21),
                            Some(21) => Some(19),
                            Some(19) => Some(28),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-ca-min",
                    &format!("CA ≥ {}", f.ca_min.map(|v| v.to_string()).unwrap_or_else(|| "0".into())),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.ca_min = match this.scout_ui.filter.ca_min {
                            None => Some(110),
                            Some(110) => Some(125),
                            Some(125) => Some(140),
                            Some(140) => Some(155),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-pa-min",
                    &format!("PA ≥ {}", f.pa_min.map(|v| v.to_string()).unwrap_or_else(|| "0".into())),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.pa_min = match this.scout_ui.filter.pa_min {
                            None => Some(130),
                            Some(130) => Some(145),
                            Some(145) => Some(160),
                            Some(160) => Some(175),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                )),
        )
        .child(pos_chips)
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_1()
                .child(step_filter_btn(
                    "sf-pos-lvl",
                    match f.pos_level {
                        PosLevel::Acc => lang.pick("Pos: Comp/Nat (≥15)", "Pos: Acc/Nat (≥15)"),
                        PosLevel::Nat => lang.pick("Pos: Só Natural (≥18)", "Pos: Natural (≥18)"),
                    },
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.pos_level = match this.scout_ui.filter.pos_level {
                            PosLevel::Acc => PosLevel::Nat,
                            PosLevel::Nat => PosLevel::Acc,
                        };
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-foot",
                    &format!(
                        "{}: {}",
                        lang.pick("Pé", "Foot"),
                        match f.foot {
                            None => lang.pick("Qualquer", "Any"),
                            Some(Foot::L) => lang.pick("Esq", "L"),
                            Some(Foot::R) => lang.pick("Dir", "R"),
                            Some(Foot::B) => lang.pick("Ambos", "Both"),
                        }
                    ),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.foot = match this.scout_ui.filter.foot {
                            None => Some(Foot::L),
                            Some(Foot::L) => Some(Foot::R),
                            Some(Foot::R) => Some(Foot::B),
                            Some(Foot::B) => None,
                        };
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                )),
        );

    let g1 = collapsible_card(
        "sf-g1",
        lang.pick("1. Jogador", "1. Player"),
        None,
        app.state.is_panel_collapsed("scout-g1"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("scout-g1");
            cx.notify();
        }),
        player_group_body,
        cx,
    );

    // Group 2: Club, contract & value
    let club_group_body = div()
        .v_flex()
        .gap_2()
        .child(Input::new(&inputs.club_q))
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_1()
                .child(step_filter_btn(
                    "sf-val-max",
                    &format!("{} ≤ {}", lang.pick("Valor", "Value"), fmt_money(f.value_max)),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.value_max = match this.scout_ui.filter.value_max {
                            None => Some(2_000_000),
                            Some(2_000_000) => Some(5_000_000),
                            Some(5_000_000) => Some(15_000_000),
                            Some(15_000_000) => Some(40_000_000),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-end-months",
                    &format!(
                        "{} ≤ {}m",
                        lang.pick("Contrato", "Contract"),
                        f.end_months.map(|m| m.to_string()).unwrap_or_else(|| "∞".into())
                    ),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.end_months = match this.scout_ui.filter.end_months {
                            None => Some(6),
                            Some(6) => Some(12),
                            Some(12) => Some(18),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-join-min",
                    &format!(
                        "{} ≥ {}",
                        lang.pick("Vir?", "Join"),
                        f.join_min.map(|v| v.to_string()).unwrap_or_else(|| "0".into())
                    ),
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.join_min = match this.scout_ui.filter.join_min {
                            None => Some(55),
                            Some(55) => Some(70),
                            Some(70) => Some(85),
                            _ => None,
                        };
                        this.scout_ui.page = 0;
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                ))
                .child(step_filter_btn(
                    "sf-loan",
                    match f.loan {
                        LoanFilter::Any => lang.pick("Emprést.: Todos", "Loans: Any"),
                        LoanFilter::Exclude => lang.pick("Emprést.: Excluir", "Loans: Exclude"),
                        LoanFilter::Only => lang.pick("Emprést.: Só", "Loans: Only"),
                    },
                    cx.listener(|this, _, _, cx| {
                        this.scout_ui.filter.loan = match this.scout_ui.filter.loan {
                            LoanFilter::Any => LoanFilter::Exclude,
                            LoanFilter::Exclude => LoanFilter::Only,
                            LoanFilter::Only => LoanFilter::Any,
                        };
                        this.scout_ui.invalidate_cache();
                        cx.notify();
                    }),
                )),
        )
        .child(
            Switch::new("sf-free")
                .checked(f.free)
                .label(lang.pick("Só jogadores livres", "Free agents only"))
                .on_change(cx.listener(|this, v: &bool, _, cx| {
                    this.scout_ui.filter.free = *v;
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                })),
        )
        .child(
            Switch::new("sf-excl-mine")
                .checked(f.exclude_mine)
                .label(lang.pick("Excluir o meu clube", "Exclude my club"))
                .on_change(cx.listener(|this, v: &bool, _, cx| {
                    this.scout_ui.filter.exclude_mine = *v;
                    this.scout_ui.page = 0;
                    this.scout_ui.invalidate_cache();
                    cx.notify();
                })),
        );

    let g2 = collapsible_card(
        "sf-g2",
        lang.pick("2. Clube, contrato e valor", "2. Club, contract & value"),
        None,
        app.state.is_panel_collapsed("scout-g2"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("scout-g2");
            cx.notify();
        }),
        club_group_body,
        cx,
    );

    // Group 3: Personality & hidden
    let pers_group_body = div()
        .h_flex()
        .flex_wrap()
        .gap_1()
        .child(step_filter_btn(
            "sf-prof",
            &format!("Prof ≥ {}", f.prof_min.unwrap_or(0)),
            cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.prof_min = cycle_opt_u8(this.scout_ui.filter.prof_min, &[13, 15, 17]);
                this.scout_ui.invalidate_cache();
                cx.notify();
            }),
        ))
        .child(step_filter_btn(
            "sf-det",
            &format!("Det ≥ {}", f.det_min.unwrap_or(0)),
            cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.det_min = cycle_opt_u8(this.scout_ui.filter.det_min, &[13, 15, 17]);
                this.scout_ui.invalidate_cache();
                cx.notify();
            }),
        ))
        .child(step_filter_btn(
            "sf-cons",
            &format!("Cons ≥ {}", f.cons_min.unwrap_or(0)),
            cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.cons_min = cycle_opt_u8(this.scout_ui.filter.cons_min, &[12, 14, 16]);
                this.scout_ui.invalidate_cache();
                cx.notify();
            }),
        ))
        .child(step_filter_btn(
            "sf-inj",
            &format!("Inj ≤ {}", f.inj_max.map(|v| v.to_string()).unwrap_or_else(|| "20".into())),
            cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.inj_max = cycle_opt_u8(this.scout_ui.filter.inj_max, &[10, 8, 6]);
                this.scout_ui.invalidate_cache();
                cx.notify();
            }),
        ))
        .child(step_filter_btn(
            "sf-likely",
            &format!("PA% ≥ {}", f.likely_min.unwrap_or(0)),
            cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.likely_min = cycle_opt_u8(this.scout_ui.filter.likely_min, &[60, 75, 85]);
                this.scout_ui.invalidate_cache();
                cx.notify();
            }),
        ));

    let g3 = collapsible_card(
        "sf-g3",
        lang.pick("3. Personalidade e ocultos", "3. Personality & hidden"),
        None,
        app.state.is_panel_collapsed("scout-g3"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("scout-g3");
            cx.notify();
        }),
        pers_group_body,
        cx,
    );

    // Group 4: Dynamic minimum attribute rows
    let mut attr_rows = div().v_flex().gap_1();
    for (i, row) in f.attrs.iter().enumerate() {
        let cur_attr = row.k.unwrap_or(Attr::Pac);
        let cur_min = row.min.unwrap_or(14);
        attr_rows = attr_rows.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(
                    Button::new(("sf-attr-k", i))
                        .outline()
                        .xsmall()
                        .flex_1()
                        .label(attr_name(cur_attr, lang))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(a) = this.scout_ui.filter.attrs.get_mut(i) {
                                let next_idx = (a.k.unwrap_or(Attr::Pac).index() + 1) % Attr::COUNT;
                                a.k = Attr::from_index(next_idx);
                                this.scout_ui.page = 0;
                                this.scout_ui.invalidate_cache();
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Button::new(("sf-attr-v", i))
                        .outline()
                        .xsmall()
                        .label(format!("≥ {}", cur_min))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(a) = this.scout_ui.filter.attrs.get_mut(i) {
                                let next = match a.min.unwrap_or(14) {
                                    10 => 12,
                                    12 => 14,
                                    14 => 15,
                                    15 => 16,
                                    16 => 18,
                                    _ => 10,
                                };
                                a.min = Some(next);
                                this.scout_ui.page = 0;
                                this.scout_ui.invalidate_cache();
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Button::new(("sf-attr-del", i))
                        .ghost()
                        .xsmall()
                        .label("×")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if i < this.scout_ui.filter.attrs.len() {
                                this.scout_ui.filter.attrs.remove(i);
                                this.scout_ui.page = 0;
                                this.scout_ui.invalidate_cache();
                                cx.notify();
                            }
                        })),
                ),
        );
    }
    attr_rows = attr_rows.child(
        Button::new("sf-attr-add")
            .outline()
            .xsmall()
            .label(lang.pick("+ Atributo", "+ Attribute"))
            .on_click(cx.listener(|this, _, _, cx| {
                this.scout_ui.filter.attrs.push(ScoutAttrFilter {
                    k: Some(Attr::Pac),
                    min: Some(14),
                });
                this.scout_ui.page = 0;
                this.scout_ui.invalidate_cache();
                cx.notify();
            })),
    );
    let g4 = collapsible_card(
        "sf-g4",
        format!(
            "4. {} ({})",
            lang.pick("Atributos mínimos", "Minimum attributes"),
            f.attrs.len()
        ),
        None,
        app.state.is_panel_collapsed("scout-g4"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("scout-g4");
            cx.notify();
        }),
        attr_rows,
        cx,
    );

    // Group 5: Role fit
    let ip_roles: Vec<&'static str> = ROLES
        .iter()
        .filter(|r| r.phase == Phase::Ip)
        .map(|r| r.id)
        .collect();
    let cur_role_lbl = f
        .role
        .as_deref()
        .and_then(role_by_id)
        .map(|r| format!("{} ({})", role_name(r, lang), r.ab))
        .unwrap_or_else(|| lang.pick("— nenhum —", "— none —").to_string());
    let ip_roles_clone = ip_roles.clone();
    let role_group_body = div()
        .v_flex()
        .gap_2()
        .child(
            div()
                .h_flex()
                .gap_1()
                .child(
                    Button::new("sf-role-cycle")
                        .outline()
                        .xsmall()
                        .flex_1()
                        .label(cur_role_lbl)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            let next = match this.scout_ui.filter.role.as_deref() {
                                None => ip_roles_clone.first().map(|s| s.to_string()),
                                Some(cur) => {
                                    let idx = ip_roles_clone.iter().position(|&r| r == cur);
                                    match idx {
                                        Some(i) if i + 1 < ip_roles_clone.len() => {
                                            Some(ip_roles_clone[i + 1].to_string())
                                        }
                                        _ => None,
                                    }
                                }
                            };
                            this.scout_ui.filter.role = next;
                            this.scout_ui.page = 0;
                            this.scout_ui.invalidate_cache();
                            cx.notify();
                        })),
                )
                .when(f.role.is_some(), |d| {
                    d.child(
                        Button::new("sf-role-min")
                            .outline()
                            .xsmall()
                            .label(format!("≥ {}", f.role_min.unwrap_or(65)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scout_ui.filter.role_min =
                                    cycle_opt_u8(this.scout_ui.filter.role_min, &[55, 65, 75, 82]);
                                this.scout_ui.invalidate_cache();
                                cx.notify();
                            })),
                    )
                }),
        );
    let g5 = collapsible_card(
        "sf-g5",
        lang.pick("5. Papel tático", "5. Tactical role"),
        None,
        app.state.is_panel_collapsed("scout-g5"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("scout-g5");
            cx.notify();
        }),
        role_group_body,
        cx,
    );

    let filters_col = div()
        .v_flex()
        .w(px(260.0))
        .flex_shrink_0()
        .gap_2()
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_family(head_font())
                        .text_size(px(16.0))
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("FILTROS", "FILTERS")),
                )
                .child(
                    Button::new("sf-reset")
                        .ghost()
                        .xsmall()
                        .label(lang.pick("Limpar", "Clear"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.scout_ui.filter = ScoutFilter::default();
                            this.scout_ui.preset.clear();
                            this.scout_ui.page = 0;
                            if let Some(inp) = &this.scout_ui.inputs {
                                inp.sync_filter(&this.scout_ui.filter, window, cx);
                            }
                            this.scout_ui.invalidate_cache();
                            cx.notify();
                        })),
                ),
        )
        .child(g1)
        .child(g2)
        .child(g3)
        .child(g4)
        .child(g5);

    // Right Table Panel (Results vs Shortlist)
    let sort_btn = |id: &'static str, sort: ScoutSort, lbl: &'static str, cur_sort: ScoutSort, cur_dir: SortDir, cx: &mut Context<ArchitectApp>| {
        let active = cur_sort == sort;
        let text = if active {
            format!("{} {}", lbl, if cur_dir == SortDir::Asc { "▲" } else { "▼" })
        } else {
            lbl.to_string()
        };
        Button::new(id)
            .ghost()
            .xsmall()
            .label(text)
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.scout_ui.filter.sort == sort {
                    this.scout_ui.filter.dir = match this.scout_ui.filter.dir {
                        SortDir::Asc => SortDir::Desc,
                        SortDir::Desc => SortDir::Asc,
                    };
                } else {
                    this.scout_ui.filter.sort = sort;
                    this.scout_ui.filter.dir = SortDir::Desc;
                }
                this.scout_ui.page = 0;
                this.scout_ui.invalidate_cache();
                cx.notify();
            }))
    };

    let cur_sort = f.sort;
    let cur_dir = f.dir;
    let table_head = div()
        .h_flex()
        .items_center()
        .gap(px(6.0))
        .pb_1()
        .border_b_1()
        .border_color(theme.border)
        .child(div().w(px(COL_STAR)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).child("☆"))
        .child(div().h_flex().flex_1().min_w(px(110.0)).overflow_hidden().child(sort_btn("th-name", ScoutSort::Name, lang.pick("Nome", "Name"), cur_sort, cur_dir, cx)))
        .child(div().h_flex().w(px(COL_AGE)).flex_shrink_0().overflow_hidden().child(sort_btn("th-age", ScoutSort::Age, lang.pick("Id.", "Age"), cur_sort, cur_dir, cx)))
        .child(div().w(px(COL_POS)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).child("Pos"))
        .child(div().flex_1().min_w_0().text_xs().text_color(theme.muted_foreground).truncate().child(lang.pick("Clube", "Club")))
        .child(div().h_flex().w(px(COL_NUM)).flex_shrink_0().overflow_hidden().child(sort_btn("th-ca", ScoutSort::Ca, "CA", cur_sort, cur_dir, cx)))
        .child(div().h_flex().w(px(COL_NUM)).flex_shrink_0().overflow_hidden().child(sort_btn("th-pa", ScoutSort::Pa, "PA", cur_sort, cur_dir, cx)))
        .child(div().h_flex().w(px(COL_MONEY)).flex_shrink_0().overflow_hidden().child(sort_btn("th-val", ScoutSort::Value, lang.pick("Valor", "Value"), cur_sort, cur_dir, cx)))
        .child(div().h_flex().w(px(COL_MONEY)).flex_shrink_0().overflow_hidden().child(sort_btn("th-wage", ScoutSort::Wage, lang.pick("Sal.", "Wage"), cur_sort, cur_dir, cx)))
        .child(div().h_flex().w(px(COL_END)).flex_shrink_0().overflow_hidden().child(sort_btn("th-end", ScoutSort::End, lang.pick("Contr.", "End"), cur_sort, cur_dir, cx)));

    let mut rows_box = div().v_flex();
    if app.scout_ui.show_shortlist {
        for (row_i, entry) in app.state.shortlist_entries.iter().enumerate() {
            let pid = entry.id.clone();
            let pid_star = entry.id.clone();
            rows_box = rows_box.child(
                div()
                    .id(("sl-row", row_i))
                    .h_flex()
                    .items_center()
                    .gap(px(6.0))
                    .py_1()
                    .border_b_1()
                    .border_color(theme.border)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.scout_ui.open_player_id = Some(pid.clone());
                        cx.notify();
                    }))
                    .child(
                        div().w(px(COL_STAR)).flex_shrink_0().child(
                            Button::new(("sl-star", row_i))
                                .ghost()
                                .xsmall()
                                .label("★")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.state.shortlist_entries.retain(|x| x.id != pid_star);
                                    if let Ok(uid) = pid_star.parse::<i64>() {
                                        this.state.shortlist.remove(&uid);
                                    }
                                    this.state.persist();
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(div().flex_1().min_w(px(110.0)).text_xs().font_semibold().truncate().child(entry.name.clone()))
                    .child(div().w(px(COL_AGE)).flex_shrink_0().text_xs().child(entry.age.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_POS)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).truncate().child(entry.pos_raw.clone()))
                    .child(div().flex_1().min_w_0().text_xs().text_color(theme.muted_foreground).truncate().child(if entry.club.is_empty() { lang.pick("livre", "free").to_string() } else { entry.club.clone() }))
                    .child(div().w(px(COL_NUM)).flex_shrink_0().text_xs().font_semibold().child(entry.ca.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_NUM)).flex_shrink_0().text_xs().child(entry.pa.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_MONEY)).flex_shrink_0().text_xs().child(fmt_money(entry.value)))
                    .child(div().w(px(COL_MONEY)).flex_shrink_0().text_xs().child(fmt_money(entry.wage)))
                    .child(div().w(px(COL_END)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).truncate().child(entry.end.clone())),
            );
        }
    } else {
        let start = page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(total_matches);
        for (offset, m) in matches[start..end].iter().enumerate() {
            let Some(p) = db.players.get(m.index) else {
                continue;
            };
            let starred = shortlist_ids.contains(p.id.as_str());
            let pid_open = p.id.clone();
            let entry_star = to_shortlist_entry(p);
            let pid_star = p.id.clone();

            rows_box = rows_box.child(
                div()
                    .id(("sc-row", offset))
                    .h_flex()
                    .items_center()
                    .gap(px(6.0))
                    .py_1()
                    .border_b_1()
                    .border_color(theme.border)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.scout_ui.open_player_id = Some(pid_open.clone());
                        cx.notify();
                    }))
                    .child(
                        div().w(px(COL_STAR)).flex_shrink_0().child(
                            Button::new(("sc-star", offset))
                                .ghost()
                                .xsmall()
                                .label(if starred { "★" } else { "☆" })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Some(pos) = this
                                        .state
                                        .shortlist_entries
                                        .iter()
                                        .position(|x| x.id == pid_star)
                                    {
                                        this.state.shortlist_entries.remove(pos);
                                        if let Ok(uid) = pid_star.parse::<i64>() {
                                            this.state.shortlist.remove(&uid);
                                        }
                                    } else {
                                        if let Ok(uid) = entry_star
                                            .uniq
                                            .as_deref()
                                            .unwrap_or(&entry_star.id)
                                            .parse::<i64>()
                                        {
                                            this.state.shortlist.insert(uid);
                                        }
                                        this.state.shortlist_entries.push(entry_star.clone());
                                    }
                                    this.state.persist();
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(div().flex_1().min_w(px(110.0)).text_xs().font_semibold().truncate().child(p.name.clone()))
                    .child(div().w(px(COL_AGE)).flex_shrink_0().text_xs().child(p.age.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_POS)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).truncate().child(p.pos_raw.clone()))
                    .child(div().flex_1().min_w_0().text_xs().text_color(theme.muted_foreground).truncate().child(if p.club.is_empty() { lang.pick("livre", "free").to_string() } else { p.club.clone() }))
                    .child(div().w(px(COL_NUM)).flex_shrink_0().text_xs().font_semibold().child(p.ca.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_NUM)).flex_shrink_0().text_xs().child(p.pa.map(|v| v.to_string()).unwrap_or_else(|| "—".into())))
                    .child(div().w(px(COL_MONEY)).flex_shrink_0().text_xs().child(fmt_money(p.value)))
                    .child(div().w(px(COL_MONEY)).flex_shrink_0().text_xs().child(fmt_money(p.wage)))
                    .child(div().w(px(COL_END)).flex_shrink_0().text_xs().text_color(theme.muted_foreground).truncate().child(p.end.clone())),
            );
        }
    }

    let table_panel = panel(cx)
        .flex_1()
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .h_flex()
                        .gap_1()
                        .child(
                            Button::new("sc-tab-res")
                                .xsmall()
                                .label(format!("{} ({})", lang.pick("Resultados", "Results"), total_matches))
                                .when(!app.scout_ui.show_shortlist, |b| b.primary())
                                .when(app.scout_ui.show_shortlist, |b| b.outline())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scout_ui.show_shortlist = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("sc-tab-sl")
                                .xsmall()
                                .label(format!(
                                    "★ {} ({})",
                                    lang.pick("Lista de observação", "Shortlist"),
                                    app.state.shortlist_entries.len()
                                ))
                                .when(app.scout_ui.show_shortlist, |b| b.primary())
                                .when(!app.scout_ui.show_shortlist, |b| b.outline())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.scout_ui.show_shortlist = true;
                                    cx.notify();
                                })),
                        ),
                )
                .when(!app.scout_ui.show_shortlist && total_pages > 1, |d| {
                    d.child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("sc-prev")
                                    .outline()
                                    .xsmall()
                                    .disabled(page == 0)
                                    .label("‹")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.scout_ui.page = this.scout_ui.page.saturating_sub(1);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{} / {}", page + 1, total_pages)),
                            )
                            .child(
                                Button::new("sc-next")
                                    .outline()
                                    .xsmall()
                                    .disabled(page + 1 >= total_pages)
                                    .label("›")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.scout_ui.page += 1;
                                        cx.notify();
                                    })),
                            ),
                    )
                }),
        )
        .child(table_head)
        .child(rows_box);

    // Optional Player Profile Modal
    let open_player = app
        .scout_ui
        .open_player_id
        .as_deref()
        .and_then(|id| db.players.iter().find(|p| p.id == id));
    let profile_modal = open_player.map(|p| render_player_profile_modal(p, db, &ctx, lang, cx));

    // Optional Scout in FM (Recruitment Focus) Modal
    let focus_modal_el = app
        .scout_ui
        .focus_modal
        .as_ref()
        .map(|fm| render_focus_modal(fm, db, matches, inputs, lang, cx));

    div()
        .relative()
        .v_flex()
        .w_full()
        .gap_3()
        .child(top_bar)
        .child(preset_row)
        .child(
            div()
                .h_flex()
                .w_full()
                .items_start()
                .gap_4()
                .child(filters_col)
                .child(table_panel),
        )
        .when_some(profile_modal, |d, m| d.child(m))
        .when_some(focus_modal_el, |d, m| d.child(m))
}

// Scouting table column widths (sum fits next to the 260px filter column at 1000px).
const COL_STAR: f32 = 24.0;
const COL_AGE: f32 = 38.0;
const COL_POS: f32 = 56.0;
const COL_NUM: f32 = 46.0;
const COL_MONEY: f32 = 58.0;
const COL_END: f32 = 72.0;

fn step_filter_btn(
    id: &'static str,
    label: &str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .outline()
        .xsmall()
        .label(label.to_string())
        .on_click(on_click)
}

fn cycle_opt_u8(cur: Option<u8>, steps: &[u8]) -> Option<u8> {
    match cur {
        None => steps.first().copied(),
        Some(v) => {
            let idx = steps.iter().position(|&x| x == v);
            match idx {
                Some(i) if i + 1 < steps.len() => Some(steps[i + 1]),
                _ => None,
            }
        }
    }
}

fn render_player_profile_modal(
    p: &ScoutPlayer,
    db: &ScoutDb,
    ctx: &ScoutContext<'_>,
    lang: Lang,
    cx: &mut Context<ArchitectApp>,
) -> impl IntoElement {
    let theme = cx.theme();
    let jn = join_likelihood(p, ctx, lang);
    let gain = if !ctx.squad.is_empty() {
        ctx.tactic.and_then(|t| best_gain(p, t, ctx.squad))
    } else {
        None
    };
    let roles = best_roles_any(p, 5);
    let lk = reach_likelihood(p);
    let nat_name = p
        .nat
        .and_then(|id| db.nations.get(&id))
        .map(String::as_str)
        .unwrap_or("—");

    let is_gk = p.is_only_gk() || p.pos_r[0] >= 15;
    let cats = if is_gk {
        [AttrCat::Goalkeeping, AttrCat::Mental, AttrCat::Physical]
    } else {
        [AttrCat::Technical, AttrCat::Mental, AttrCat::Physical]
    };

    let mut attr_cols = div().h_flex().flex_wrap().items_start().gap_3();
    for cat in cats {
        let mut col = div().v_flex().flex_1().min_w(px(200.0)).gap_1().child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(theme.foreground)
                .child(match cat {
                    AttrCat::Technical => lang.pick("Técnicos", "Technical"),
                    AttrCat::Mental => lang.pick("Mentais", "Mental"),
                    AttrCat::Physical => lang.pick("Físicos", "Physical"),
                    AttrCat::Goalkeeping => lang.pick("Guarda-redes", "Goalkeeping"),
                }),
        );
        let mut wrap = div().h_flex().flex_wrap().gap_1();
        for a in Attr::ALL.iter().copied().filter(|a| a.category() == cat) {
            if let Some(v) = p.attr(a) {
                wrap = wrap.child(attr_chip(a.code(), v, cx));
            }
        }
        col = col.child(wrap);
        attr_cols = attr_cols.child(col);
    }

    let mut roles_box = div().v_flex().flex_1().min_w(px(240.0)).gap_1().child(
        div()
            .text_xs()
            .font_semibold()
            .text_color(theme.foreground)
            .child(lang.pick("Melhores papéis", "Best roles")),
    );
    for (g, r, sc) in roles {
        roles_box = roles_box.child(
            div()
                .h_flex()
                .justify_between()
                .text_xs()
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(format!("{} ({})", role_name(r, lang), group_name(g, lang))),
                )
                .child(
                    div()
                        .font_bold()
                        .text_color(score_color(sc))
                        .child(sc.to_string()),
                ),
        );
    }

    let mut pers_box = div().v_flex().flex_1().min_w(px(240.0)).gap_1().child(
        div()
            .text_xs()
            .font_semibold()
            .text_color(theme.foreground)
            .child(lang.pick("Personalidade e ocultos", "Personality & hidden")),
    );
    for (ps, lbl) in [
        (ScoutPers::Professionalism, lang.pick("Profissionalismo", "Professionalism")),
        (ScoutPers::Ambition, lang.pick("Ambição", "Ambition")),
        (ScoutPers::Pressure, lang.pick("Pressão", "Pressure")),
        (ScoutPers::Loyalty, lang.pick("Lealdade", "Loyalty")),
    ] {
        if let Some(v) = p.pers_val(ps) {
            pers_box = pers_box.child(
                div()
                    .h_flex()
                    .justify_between()
                    .text_xs()
                    .child(div().text_color(theme.muted_foreground).child(lbl))
                    .child(div().font_semibold().child(v.to_string())),
            );
        }
    }
    for (hd, lbl) in [
        (ScoutHid::Cons, lang.pick("Consistência", "Consistency")),
        (ScoutHid::Imp, lang.pick("Jogos importantes", "Important matches")),
        (ScoutHid::Inj, lang.pick("Propensão a lesões", "Injury proneness")),
    ] {
        if let Some(v) = p.hid_val(hd) {
            pers_box = pers_box.child(
                div()
                    .h_flex()
                    .justify_between()
                    .text_xs()
                    .child(div().text_color(theme.muted_foreground).child(lbl))
                    .child(div().font_semibold().child(v.to_string())),
            );
        }
    }

    div()
        .v_flex()
        .gap_3()
        .p_5()
        .rounded(px(RADIUS))
        .border_1()
        .border_color(theme.primary)
        .bg(theme.secondary)
        .shadow_lg()
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .v_flex()
                        .min_w_0()
                        .gap(px(2.0))
                        .child(
                            div()
                                .font_family(head_font())
                                .text_size(px(22.0))
                                .font_bold()
                                .text_color(theme.foreground)
                                .child(p.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{} · {} · {} · {}",
                                    p.age.map(|a| format!("{} {}", a, lang.pick("anos", "y"))).unwrap_or_else(|| "—".into()),
                                    if p.club.is_empty() { lang.pick("Livre", "Free") } else { &p.club },
                                    nat_name,
                                    p.pos_raw
                                )),
                        ),
                )
                .child(
                    Button::new("close-scout-profile")
                        .outline()
                        .small()
                        .label(lang.pick("Fechar ✕", "Close ✕"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.scout_ui.open_player_id = None;
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(kpi_card("CA", p.ca.map(|v| v.to_string()).unwrap_or_else(|| "—".into()), None, None, cx))
                .child(kpi_card("PA", p.pa.map(|v| v.to_string()).unwrap_or_else(|| "—".into()), lk.map(|v| format!("{}% PA", v).into()), None, cx))
                .child(kpi_card(lang.pick("Valor", "Value"), fmt_money(p.value), None, None, cx))
                .child(kpi_card(lang.pick("Salário", "Wage"), fmt_money(p.wage), Some(p.end.clone().into()), None, cx)),
        )
        .when_some(jn, |d, j| {
            d.child(badge_pill(
                format!(
                    "{}: {}/100 — {}",
                    lang.pick("Prob. querer vir", "Join chance"),
                    j.score,
                    j.why.join(" · ")
                ),
                if j.score >= 70 { BadgeTone::Good } else { BadgeTone::Info },
                cx,
            ))
        })
        .when_some(gain, |d, g| {
            d.child(
                div()
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(format!(
                        "{}: {} ({} vs {} · {:+})",
                        lang.pick("Melhor casa no teu XI", "Best slot in your XI"),
                        g.code,
                        g.his,
                        g.cur,
                        g.gain
                    )),
            )
        })
        .child(attr_cols)
        .child(div().h_flex().flex_wrap().items_start().gap_4().child(roles_box).child(pers_box))
}

fn render_focus_modal(
    fm: &FocusModalState,
    db: &ScoutDb,
    matches: &[ScoutMatch],
    inputs: &ScoutingInputs,
    lang: Lang,
    cx: &mut Context<ArchitectApp>,
) -> impl IntoElement {
    let theme = cx.theme();
    let mut counts: HashMap<i32, usize> = HashMap::new();
    for m in matches {
        if let Some(nat) = db.players.get(m.index).and_then(|p| p.nat) {
            if db.nations.contains_key(&nat) {
                *counts.entry(nat).or_insert(0) += 1;
            }
        }
    }
    let mut top: Vec<(i32, String, usize)> = counts
        .into_iter()
        .filter_map(|(k, c)| db.nations.get(&k).map(|n| (k, n.clone(), c)))
        .collect();
    top.sort_by(|a, b| b.2.cmp(&a.2));
    top.truncate(10);

    let mut nat_chips = div().h_flex().flex_wrap().gap_1();
    for (i, (nat_id, nat_name, cnt)) in top.iter().enumerate() {
        let nid = *nat_id;
        let active = fm.selected_nations.contains(&nid);
        nat_chips = nat_chips.child(
            Button::new(("fm-nat", i))
                .xsmall()
                .label(format!("{} · {}", nat_name, cnt))
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(m) = &mut this.scout_ui.focus_modal {
                        if !m.selected_nations.remove(&nid) {
                            m.selected_nations.insert(nid);
                        }
                        cx.notify();
                    }
                })),
        );
    }

    let areas_req: Vec<BridgeFocusArea> = fm
        .selected_nations
        .iter()
        .filter_map(|nid| {
            db.nations.get(nid).map(|lbl| BridgeFocusArea {
                id: *nid as i64,
                label: lbl.clone(),
            })
        })
        .collect();
    let req = BridgeFocusRequest {
        name: fm.name.clone(),
        age_min: fm.age_min,
        age_max: fm.age_max,
        ca_stars: fm.ca_stars,
        pa_stars: fm.pa_stars,
        areas: areas_req,
    };
    let can_submit = !req.areas.is_empty();

    div()
        .v_flex()
        .gap_3()
        .p_5()
        .rounded(px(RADIUS))
        .border_1()
        .border_color(theme.primary)
        .bg(theme.secondary)
        .shadow_lg()
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .font_family(head_font())
                        .text_size(px(18.0))
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick(
                            "Criar Recruitment Focus no FM27",
                            "Create Recruitment Focus in FM27",
                        )),
                )
                .child(
                    Button::new("fm-focus-close")
                        .ghost()
                        .xsmall()
                        .label("✕")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.scout_ui.focus_modal = None;
                            cx.notify();
                        })),
                ),
        )
        .child(Input::new(&inputs.focus_name))
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(step_filter_btn(
                    "fm-age-min",
                    &format!("{} ≥ {}", lang.pick("Idade", "Age"), fm.age_min),
                    cx.listener(|this, _, _, cx| {
                        if let Some(m) = &mut this.scout_ui.focus_modal {
                            m.age_min = match m.age_min {
                                15 => 17,
                                17 => 19,
                                19 => 21,
                                _ => 15,
                            };
                            if m.age_max < m.age_min {
                                m.age_max = m.age_min;
                            }
                            cx.notify();
                        }
                    }),
                ))
                .child(step_filter_btn(
                    "fm-age-max",
                    &format!("{} ≤ {}", lang.pick("Idade", "Age"), fm.age_max),
                    cx.listener(|this, _, _, cx| {
                        if let Some(m) = &mut this.scout_ui.focus_modal {
                            m.age_max = match m.age_max {
                                19 => 21,
                                21 => 23,
                                23 => 28,
                                28 => 32,
                                _ => 19,
                            }
                            .max(m.age_min);
                            cx.notify();
                        }
                    }),
                ))
                .child(step_filter_btn(
                    "fm-ca-st",
                    &format!("CA ★ {:.1}", fm.ca_stars),
                    cx.listener(|this, _, _, cx| {
                        if let Some(m) = &mut this.scout_ui.focus_modal {
                            m.ca_stars = if m.ca_stars >= 4.5 { 0.5 } else { m.ca_stars + 0.5 };
                            cx.notify();
                        }
                    }),
                ))
                .child(step_filter_btn(
                    "fm-pa-st",
                    &format!("PA ★ {:.1}", fm.pa_stars),
                    cx.listener(|this, _, _, cx| {
                        if let Some(m) = &mut this.scout_ui.focus_modal {
                            m.pa_stars = if m.pa_stars >= 5.0 { 1.5 } else { m.pa_stars + 0.5 };
                            cx.notify();
                        }
                    }),
                )),
        )
        .child(nat_chips)
        .child(
            div()
                .h_flex()
                .justify_end()
                .gap_2()
                .child(
                    Button::new("fm-focus-cancel")
                        .outline()
                        .small()
                        .label(lang.pick("Cancelar", "Cancel"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.scout_ui.focus_modal = None;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("fm-focus-create")
                        .primary()
                        .small()
                        .disabled(!can_submit)
                        .label(lang.pick("Criar no FM27", "Create in FM27"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            spawn_create_focus_in_fm(this, req.clone(), cx);
                        })),
                ),
        )
}
