#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! FM27 Tactical Architect — native desktop application built with GPUI + `gpui-kit`.
//!
//! - Central persisted `AppState` (`ta-backend::StateStore`) + async background task runners
//! - Global file drag-and-drop (`.json` backup, `.fmf`, squad/opponent CSV/HTML, full-save scouting CSV)
//!   & keyboard shortcuts (`Cmd/Ctrl+B`, `Cmd/Ctrl+Shift+L`)
//! - Screens in `views/`: Home, My Team (Squad, Best XI, Generator, Opponent), Scouting,
//!   Tactic (Analyse, Animated Pitch, Game States, Set Pieces, FM27 Sheet, Import), Training, Settings

mod state;
mod views;
mod widgets;

use crate::state::{
    bind_global_keys, non_empty, AppState, BackendBridgeTransport, NavSection, ReadSaveNow,
    TacticTab, TeamTab, ToastKind, ToggleCompactNav,
};
use crate::views::fm_bridge_ui::spawn_refresh_bridge_status;
use crate::views::tactic_sheet::SheetViewState;
use crate::views::generator::GeneratorViewState;
use crate::views::home::render_home;
use crate::views::scouting::{render_scouting, ScoutingViewState};
use crate::views::settings::{render_settings, SettingsInputs};
use crate::views::tactic_gameplans::{render_gameplans, GamePlansViewState};
use crate::views::team::{apply_tactic_to_app, ingest_squad_text, render_team_tab, TeamViewState};
use crate::views::tactic_analyse::{render_tactic_analyse, AnalyseViewState};
use crate::views::tactic_import::{render_import, spawn_inspect_fmf, ImportViewState};
use crate::views::tactic_setpieces::render_setpieces;
use crate::views::tactic_sheet::render_sheet;
use crate::views::training::render_training;
use crate::views::scouting::{is_full_save_export, spawn_ingest_scout_path};
use crate::widgets::common::{
    busy_floating_card, busy_top_bar, head_font, pal, toast_banner, with_alpha, RADIUS,
};
use crate::widgets::pitch_animated::AnimatedPitchView;
use crate::widgets::pitch_interactive::PitchPhaseMode;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::path::PathBuf;
use ta_backend::{export_save, latest_save, plugin_install, plugin_status};
use ta_core::live::sim_model::sim_model;
use ta_core::live::FmbClient;
use ta_core::types::Lang;
use ta_core::APP_VERSION;

/// Root GPUI application view.
pub struct ArchitectApp {
    pub state: AppState,
    pub section: NavSection,
    pub team_tab: TeamTab,
    pub tactic_tab: TacticTab,
    pub pitch_mode: PitchPhaseMode,
    pub selected_slot: Option<usize>,
    pub settings_inputs: SettingsInputs,
    pub animated_pitch: Entity<AnimatedPitchView>,
    pub team_ui: TeamViewState,
    pub scout_ui: ScoutingViewState,
    pub gen_ui: GeneratorViewState,
    pub gameplans_ui: GamePlansViewState,
    pub analyse_ui: AnalyseViewState,
    pub import_ui: ImportViewState,
    pub sheet_ui: SheetViewState,
    focus_handle: FocusHandle,
}

impl ArchitectApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::with_state(AppState::load_default(), window, cx)
    }

    pub fn with_state(state: AppState, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings_inputs = SettingsInputs::new(
            &state.custom_save_dir,
            &state.custom_fm_dir,
            &state.my_club,
            window,
            cx,
        );
        let lang = state.lang;
        let initial_sim = sim_model(&state.tactic, lang);
        let initial_squad = state.squad.clone();
        let animated_pitch = cx.new(|_| {
            let mut v = AnimatedPitchView::new(lang);
            v.sync_squad(&initial_squad);
            v.set_model(Some(initial_sim));
            v
        });
        let focus_handle = cx.focus_handle();
        let team_ui = TeamViewState::with_search_input(window, cx);

        Self {
            state,
            section: NavSection::Home,
            team_tab: TeamTab::Squad,
            tactic_tab: TacticTab::Analyse,
            pitch_mode: PitchPhaseMode::Ip,
            selected_slot: Some(0),
            settings_inputs,
            animated_pitch,
            team_ui,
            scout_ui: ScoutingViewState::default(),
            gen_ui: GeneratorViewState::default(),
            gameplans_ui: GamePlansViewState::default(),
            analyse_ui: AnalyseViewState::default(),
            import_ui: ImportViewState::default(),
            sheet_ui: SheetViewState::default(),
            focus_handle,
        }
    }

    /// Makes `tactic` the active tactic (persisted, animated pitch re-synced) without changing screen.
    pub fn apply_active_tactic(&mut self, tactic: ta_core::types::Tactic, cx: &mut Context<Self>) {
        apply_tactic_to_app(self, tactic, false, cx);
    }

    pub fn set_lang(&mut self, lang: Lang, cx: &mut Context<Self>) {
        if self.state.lang != lang {
            self.state.lang = lang;
            self.state.persist();
            let updated_sim = sim_model(&self.state.tactic, lang);
            self.animated_pitch.update(cx, |v, cx| {
                v.lang = lang;
                v.set_model(Some(updated_sim));
                cx.notify();
            });
            cx.notify();
        }
    }

    pub fn set_dark_mode(&mut self, dark: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.state.dark_mode = dark;
        self.state.apply_theme(Some(window), cx);
        self.state.persist();
        cx.notify();
    }

    /// Scans for the newest `.fm` save on a background thread.
    pub fn spawn_scan_latest_save(&mut self, cx: &mut Context<Self>) {
        let custom = non_empty(&self.state.custom_save_dir).map(str::to_string);
        cx.spawn(async move |this, cx| {
            let found = cx
                .background_executor()
                .spawn(async move { latest_save(custom.as_deref()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.state.last_save_info = found;
                cx.notify();
            });
        })
        .detach();
    }

    /// Runs `latest_save` + `export_save` (`fmsave`) on a background thread without blocking UI.
    pub fn spawn_read_save_now(&mut self, cx: &mut Context<Self>) {
        let lang = self.state.lang;
        let custom = non_empty(&self.state.custom_save_dir).map(str::to_string);
        self.state.set_busy(
            lang.pick("A ler save do FM27 (fmsave)…", "Reading FM27 save (fmsave)…"),
            Some(0.3),
        );
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let info = latest_save(custom.as_deref())?;
                    let export = export_save(&info.path);
                    Some((info, export))
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                this.state.clear_busy();
                match result {
                    None => {
                        let msg = this.state.lang.pick(
                            "Nenhum save .fm encontrado nas pastas habituais.",
                            "No .fm save found in standard folders.",
                        );
                        this.state.last_save_info = None;
                        this.state.last_save_err = Some(msg.to_string());
                        this.state.set_toast(msg, ToastKind::Warn);
                    }
                    Some((info, Ok(exp))) => {
                        let lines = exp.content.lines().count().saturating_sub(1);
                        let summary = format!("{} ({} linhas via {})", exp.save, lines, exp.runner);
                        this.state.last_save_info = Some(info);
                        this.state.last_save_mtime = exp.modified;
                        this.state.last_save_read = Some(summary.clone());
                        this.state.last_save_err = None;
                        this.state.persist();
                        this.state.set_toast(
                            format!(
                                "{}: {}",
                                this.state.lang.pick("Save lido", "Save read"),
                                summary
                            ),
                            ToastKind::Success,
                        );
                    }
                    Some((info, Err(err))) => {
                        this.state.last_save_info = Some(info);
                        this.state.last_save_err = Some(err.clone());
                        this.state.persist();
                        this.state.set_toast(err, ToastKind::Error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Refreshes `plugin_status` off-thread.
    pub fn spawn_refresh_plugin_status(&mut self, cx: &mut Context<Self>) {
        let custom_fm = non_empty(&self.state.custom_fm_dir).map(str::to_string);
        cx.spawn(async move |this, cx| {
            let status = cx
                .background_executor()
                .spawn(async move { plugin_status(custom_fm.as_deref()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.state.plugin_info = status;
                cx.notify();
            });
        })
        .detach();
    }

    /// Installs or updates the embedded `TacticalBibleBridge.dll` off-thread.
    pub fn spawn_install_plugin(&mut self, cx: &mut Context<Self>) {
        let lang = self.state.lang;
        let custom_fm = non_empty(&self.state.custom_fm_dir).map(str::to_string);
        self.state.set_busy(
            lang.pick("A instalar plugin no FM27…", "Installing FM27 plugin…"),
            Some(0.5),
        );
        cx.notify();

        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { plugin_install(custom_fm.as_deref()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.state.clear_busy();
                match res {
                    Ok(status) => {
                        this.state.plugin_info = status;
                        this.state.set_toast(
                            this.state.lang.pick(
                                "Plugin instalado em BepInEx/plugins/TacticalBibleBridge.",
                                "Plugin installed in BepInEx/plugins/TacticalBibleBridge.",
                            ),
                            ToastKind::Success,
                        );
                    }
                    Err(err) => {
                        this.state.set_toast(err, ToastKind::Error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Settings › "check connection": PING/VERSION over the bridge (forced), with a toast.
    pub fn spawn_ping_bridge(&mut self, cx: &mut Context<Self>) {
        let lang = self.state.lang;
        self.state.set_busy(
            lang.pick("A verificar ligação ao Bridge…", "Checking FM27 Bridge connection…"),
            Some(0.5),
        );
        cx.notify();

        cx.spawn(async move |this, cx| {
            let status = cx
                .background_executor()
                .spawn(async move { FmbClient::new(BackendBridgeTransport, lang).status(true) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.state.clear_busy();
                let lang = this.state.lang;
                if status.ok && status.ready {
                    this.state.set_toast(lang.pick("Bridge FM27 ligado.", "FM27 Bridge connected."), ToastKind::Success);
                } else if status.ok {
                    this.state.set_toast(
                        lang.pick(
                            "O FM está aberto mas ainda não carregou um jogo.",
                            "FM is open but has not loaded a game yet.",
                        ),
                        ToastKind::Warn,
                    );
                } else {
                    this.state.set_toast(
                        status.err.clone().unwrap_or_else(|| "no reply".into()),
                        ToastKind::Warn,
                    );
                }
                this.state.bridge_status = status;
                cx.notify();
            });
        })
        .detach();
    }

    /// Keeps `state.bridge_status` fresh (as the browser build's `fmbRefresh`, 15 s throttle),
    /// so the "… in FM" buttons reflect whether FM and the plugin are up.
    pub fn start_bridge_status_poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            if this.update(cx, |this, cx| spawn_refresh_bridge_status(this, cx)).is_err() {
                break;
            }
            cx.background_executor().timer(std::time::Duration::from_secs(15)).await;
        })
        .detach();
    }

    fn handle_dropped_paths(&mut self, paths: &[PathBuf], window: &mut Window, cx: &mut Context<Self>) {
        if paths.is_empty() {
            return;
        }
        self.state.last_dropped_files = paths.to_vec();

        for path in paths {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            if ext == "json" {
                match self.state.import_backup_path(path) {
                    Ok(msg) => {
                        self.state.apply_theme(Some(window), cx);
                        self.settings_inputs.sync_from_state(self, window, cx);
                        self.state.set_toast(msg, ToastKind::Success);
                    }
                    Err(err) => self.state.set_toast(err, ToastKind::Error),
                }
            } else if ext == "fmf" {
                self.section = NavSection::Tactic;
                self.tactic_tab = TacticTab::Import;
                spawn_inspect_fmf(self, path.clone(), cx);
            } else {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                match std::fs::read(path) {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        if is_full_save_export(&text) {
                            spawn_ingest_scout_path(self, path.clone(), cx);
                            self.section = NavSection::Scouting;
                        } else {
                            ingest_squad_text(self, &text, &name, None, cx);
                            self.section = NavSection::Team;
                            self.team_tab = if self.state.opponent_source == name {
                                TeamTab::Opponent
                            } else {
                                TeamTab::Squad
                            };
                        }
                    }
                    Err(err) => self.state.set_toast(format!("{name}: {err}"), ToastKind::Error),
                }
            }
        }
        cx.notify();
    }

    /// Red club-colored top bar (browser build `#nav .tb-row`): FM27 badge, main sections in
    /// condensed capitals, status lights, read-save, EN|PT and the Settings gear. Narrow windows
    /// drop the labels first (status, brand, button text), as the browser build's media queries.
    fn render_topbar(&self, width: f32, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.state.lang;
        let ink: Hsla = rgb(pal::BAR_INK).into();
        let ink_soft = with_alpha(ink, 0.86);
        let shade = |a: f32| -> Hsla { with_alpha(rgb(0x000000).into(), a) };
        let compact = self.state.nav_compact || width < 760.0;
        let show_status_labels = width >= 1560.0;
        let show_brand_label = width >= 1366.0;
        let tight = width < 1180.0;

        let mut nav = div().h_flex().h_full().min_w_0().overflow_hidden();
        for section in NavSection::MAIN {
            let active = section == self.section;
            nav = nav.child(
                div()
                    .id(("nav", section as usize))
                    .relative()
                    .h_full()
                    .flex()
                    .flex_none()
                    .items_center()
                    .px(px(if tight { 9.0 } else { 13.0 }))
                    .cursor_pointer()
                    .font_family(head_font())
                    .text_size(px(if tight { 15.0 } else { 16.0 }))
                    .font_semibold()
                    .text_color(if active { ink } else { ink_soft })
                    .when(active, |d| d.bg(shade(0.24)))
                    .hover(|s| s.bg(shade(0.14)).text_color(ink))
                    .child(if compact {
                        section.icon().to_string()
                    } else {
                        section.label(lang).to_uppercase()
                    })
                    .when(active, |d| {
                        d.child(div().absolute().left(px(10.0)).right(px(10.0)).bottom_0().h(px(3.0)).bg(ink))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.section = section;
                        cx.notify();
                    })),
            );
        }

        let dot = |on: bool| {
            div()
                .flex_none()
                .size(px(8.0))
                .rounded_full()
                .bg(if on { rgb(pal::DOT_OK).into() } else { with_alpha(ink, 0.35) })
        };
        let stat = |id: &'static str, on: bool, label: String, tip: String| {
            div()
                .id(id)
                .h_flex()
                .items_center()
                .gap(px(6.0))
                .text_xs()
                .text_color(ink)
                .child(dot(on))
                .when(show_status_labels, |d| d.child(label))
                .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
        };
        let squad_n = self.state.squad.len();
        let squad_label = if squad_n > 0 {
            let mut s = format!("{squad_n} {}", lang.pick("jogadores", "players"));
            if !self.state.my_club.is_empty() {
                s.push_str(" · ");
                s.push_str(&self.state.my_club);
            }
            s
        } else {
            lang.pick("Sem plantel", "No squad").to_string()
        };
        let save_tip = match (&self.state.last_save_err, &self.state.last_save_read) {
            (Some(_), _) => lang.pick("Save: erro (ver Definições)", "Save: error (see Settings)").to_string(),
            (None, Some(msg)) => format!("Save: {msg}"),
            (None, None) => lang.pick("Save: ainda não lido", "Save: not read yet").to_string(),
        };
        let has_opp = !self.state.opponent.is_empty();
        let bst = &self.state.bridge_status;
        let fm_tip = if bst.ok && bst.ready {
            lang.pick("FM ligado (Tactical Bible Bridge)", "FM connected (Tactical Bible Bridge)")
        } else if bst.ok {
            lang.pick("FM aberto, à espera do jogo", "FM open, waiting for the game")
        } else {
            lang.pick(
                "FM não ligado (plugin opcional, ver Definições)",
                "FM not connected (optional plugin, see Settings)",
            )
        };

        let bar_btn = |id: &'static str, label: String, on: bool| {
            div()
                .id(id)
                .h(px(30.0))
                .px(px(10.0))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(6.0))
                .rounded(px(RADIUS))
                .border_1()
                .cursor_pointer()
                .text_sm()
                .when(on, |d| d.bg(ink).border_color(ink).text_color(rgb(pal::BAR)))
                .when(!on, |d| {
                    d.bg(shade(0.2))
                        .border_color(with_alpha(ink, 0.28))
                        .text_color(ink)
                        .hover(|s| s.border_color(ink))
                })
                .child(label)
        };
        let read_label = if show_brand_label {
            format!("📂  {}", lang.pick("Ler save agora", "Read save now"))
        } else {
            "📂".to_string()
        };
        let lang_seg = div()
            .h_flex()
            .flex_none()
            .rounded(px(RADIUS))
            .border_1()
            .border_color(with_alpha(ink, 0.3))
            .overflow_hidden()
            .children([(Lang::En, "EN"), (Lang::Pt, "PT")].map(|(l, n)| {
                let on = lang == l;
                div()
                    .id(n)
                    .h(px(28.0))
                    .px(px(12.0))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .text_sm()
                    .when(on, |d| d.bg(ink).text_color(rgb(pal::BAR)))
                    .when(!on, |d| d.text_color(ink).hover(|s| s.bg(shade(0.14))))
                    .child(n)
                    .on_click(cx.listener(move |this, _, _, cx| this.set_lang(l, cx)))
            }));
        let settings_on = self.section == NavSection::Settings;

        div()
            .h_flex()
            .flex_none()
            .w_full()
            .h(px(48.0))
            .pl(px(6.0))
            .pr(px(10.0))
            .gap_1()
            .items_center()
            .bg(rgb(pal::BAR))
            .text_color(ink)
            .shadow_md()
            .child(
                div()
                    .id("brand")
                    .h_flex()
                    .flex_none()
                    .h_full()
                    .items_center()
                    .gap(px(9.0))
                    .pl(px(6.0))
                    .pr(px(10.0))
                    .cursor_pointer()
                    .child(
                        div()
                            .size(px(34.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(RADIUS))
                            .bg(ink)
                            .text_color(rgb(pal::BAR))
                            .font_family(head_font())
                            .text_size(px(12.0))
                            .font_bold()
                            .child("FM27"),
                    )
                    .when(show_brand_label, |d| {
                        d.child(
                            div()
                                .font_family(head_font())
                                .text_size(px(17.0))
                                .font_bold()
                                .child("TACTICAL ARCHITECT"),
                        )
                    })
                    .tooltip(|window, cx| {
                        Tooltip::new(format!("FM27 Tactical Architect v{APP_VERSION}")).build(window, cx)
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Home;
                        cx.notify();
                    })),
            )
            .child(nav)
            .child(
                div()
                    .ml_auto()
                    .h_flex()
                    .flex_none()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap(px(if show_status_labels { 12.0 } else { 8.0 }))
                            .child(stat("st-squad", squad_n > 0, squad_label, save_tip))
                            .child(stat(
                                "st-opp",
                                has_opp,
                                lang.pick(
                                    if has_opp { "Adversário carregado" } else { "Sem adversário" },
                                    if has_opp { "Opponent loaded" } else { "No opponent" },
                                )
                                .to_string(),
                                lang.pick("Adversário", "Opponent").to_string(),
                            ))
                            .child(stat("st-fm", bst.ok && bst.ready, "FM".into(), fm_tip.to_string())),
                    )
                    .child(
                        bar_btn("read-save-now", read_label, false)
                            .tooltip(move |window, cx| {
                                Tooltip::new(lang.pick(
                                    "Ler o save mais recente do FM27 (Ctrl+Shift+L)",
                                    "Read the latest FM27 save (Ctrl+Shift+L)",
                                ))
                                .build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, _, cx| this.spawn_read_save_now(cx))),
                    )
                    .child(lang_seg)
                    .child(
                        div()
                            .id("nav-settings")
                            .size(px(34.0))
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .rounded(px(RADIUS))
                            .cursor_pointer()
                            .text_size(px(18.0))
                            .text_color(ink)
                            .when(settings_on, |d| d.bg(shade(0.24)))
                            .hover(|s| s.bg(shade(0.24)))
                            .child("⚙")
                            .tooltip(move |window, cx| {
                                Tooltip::new(NavSection::Settings.label(lang)).build(window, cx)
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.section = NavSection::Settings;
                                cx.notify();
                            })),
                    ),
            )
    }

    /// Screen tabs or breadcrumb under the top bar (browser build `.tb-sub`).
    fn render_subbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let lang = self.state.lang;
        let bar = div()
            .h_flex()
            .flex_none()
            .w_full()
            .h(px(36.0))
            .px_4()
            .gap(px(2.0))
            .items_stretch()
            .bg(theme.tab_bar)
            .border_b_1()
            .border_color(theme.border);
        let tab = |id: (&'static str, usize), label: &'static str, on: bool| {
            div()
                .id(id)
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .px(px(12.0))
                .cursor_pointer()
                .text_sm()
                .when(on, |d| {
                    d.text_color(theme.foreground)
                        .font_semibold()
                        .child(div().absolute().left_0().right_0().bottom_0().h(px(2.0)).bg(rgb(pal::BAR)))
                })
                .when(!on, |d| d.text_color(theme.muted_foreground).hover(|s| s.text_color(theme.foreground)))
                .child(label)
        };
        match self.section {
            NavSection::Team => bar.children(TeamTab::ALL.map(|t| {
                tab(("team-tab", t.index()), t.label(lang), t == self.team_tab).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.team_tab = t;
                        cx.notify();
                    },
                ))
            })),
            NavSection::Tactic => bar.children(TacticTab::ALL.map(|t| {
                tab(("tactic-tab", t.index()), t.label(lang), t == self.tactic_tab).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.tactic_tab = t;
                        cx.notify();
                    },
                ))
            })),
            section => bar.child(
                div()
                    .flex()
                    .items_center()
                    .font_family(head_font())
                    .text_size(px(14.0))
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .child(section.label(lang).to_uppercase()),
            ),
        }
    }

    fn render_content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.section {
            NavSection::Home => render_home(self, cx).into_any_element(),
            NavSection::Team => render_team_tab(self, window, cx),
            NavSection::Scouting => render_scouting(self, window, cx).into_any_element(),
            NavSection::Tactic => match self.tactic_tab {
                TacticTab::Analyse => render_tactic_analyse(self, window, cx).into_any_element(),
                TacticTab::AnimatedPitch => self.animated_pitch.clone().into_any_element(),
                TacticTab::GameStates => render_gameplans(self, cx).into_any_element(),
                TacticTab::SetPieces => render_setpieces(self, cx).into_any_element(),
                TacticTab::Fm27Sheet => render_sheet(self, cx).into_any_element(),
                TacticTab::Import => render_import(self, cx).into_any_element(),
            },
            NavSection::Training => render_training(self, cx).into_any_element(),
            NavSection::Settings => render_settings(self, cx).into_any_element(),
        }
    }
}

impl Render for ArchitectApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = f32::from(window.viewport_size().width);
        let content = self.render_content(window, cx);
        let topbar = self.render_topbar(width, cx).into_any_element();
        let subbar = self.render_subbar(cx).into_any_element();
        let theme = cx.theme();

        div()
            .id("architect-root")
            .key_context("ArchitectApp")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &ToggleCompactNav, _, cx| {
                this.state.nav_compact = !this.state.nav_compact;
                this.state.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ReadSaveNow, _, cx| {
                this.spawn_read_save_now(cx);
            }))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.handle_dropped_paths(paths.paths(), window, cx);
            }))
            .relative()
            .v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(topbar)
            .child(subbar)
            .when_some(self.state.busy.as_ref(), |d, busy| d.child(busy_top_bar(busy, cx)))
            .child(
                div()
                    .id("main-scroll-area")
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px(px(22.0))
                    .pt(px(18.0))
                    .pb(px(60.0))
                    .child(div().w_full().min_w_0().child(content)),
            )
            .when_some(self.state.busy.as_ref(), |d, busy| {
                d.child(busy_floating_card(busy, cx))
            })
            .when_some(self.state.toast.as_ref(), |d, toast| {
                d.child(toast_banner(
                    toast,
                    cx.listener(|this, _, _, cx| {
                        this.state.clear_toast();
                        cx.notify();
                    }),
                    cx,
                ))
            })
    }
}

fn main() {
    // gpui-kit's bundled icons (select chevrons, checkbox ticks, …) render only with its assets.
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        bind_global_keys(cx);

        let initial_state = AppState::load_default();
        initial_state.apply_theme(None, cx);

        let bounds = Bounds::centered(None, size(px(1440.0), px(920.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("FM27 Tactical Architect".into()),
                ..Default::default()
            }),
            window_min_size: Some(size(px(1000.0), px(680.0))),
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| {
                let mut app = ArchitectApp::new(window, cx);
                app.start_bridge_status_poll(cx);
                app
            })
        })
        .expect("failed to open FM27 Tactical Architect window");
    });
}

#[cfg(test)]
mod tests {
    use super::state::{AppState, NavSection, TacticTab, TeamTab};
    use super::views::team::load_sample_squad;
    use super::ArchitectApp;
    use gpui_kit::{AppContext, TestAppContext, WindowBounds, WindowOptions};
    use ta_backend::StateStore;
    use ta_core::Lang;

    /// Every screen renders a full frame in both languages with a loaded squad (no panics
    /// from indexing, empty states or engine calls inside render paths).
    #[gpui_kit::test]
    fn every_screen_renders_in_both_languages(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let dir = std::env::temp_dir().join(format!("ta-app-render-{}", std::process::id()));
        for lang in [Lang::En, Lang::Pt] {
            let mut state = AppState::load_from(StateStore::new(dir.join("state.json")));
            state.lang = lang;
            let (handle, app) = cx.update(|cx| {
                gpui_kit::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds::centered(
                            None,
                            gpui_kit::size(gpui_kit::px(1440.0), gpui_kit::px(920.0)),
                            cx,
                        ))),
                        ..Default::default()
                    },
                    cx,
                    |window, cx| cx.new(|cx| ArchitectApp::with_state(state, window, cx)),
                )
                .expect("open test window")
            });
            let mut screens = vec![(NavSection::Home, None, None)];
            screens.extend(TeamTab::ALL.iter().map(|t| (NavSection::Team, Some(*t), None)));
            screens.push((NavSection::Scouting, None, None));
            screens.extend(TacticTab::ALL.iter().map(|t| (NavSection::Tactic, None, Some(*t))));
            screens.push((NavSection::Training, None, None));
            screens.push((NavSection::Settings, None, None));
            assert_eq!(screens.len(), 14);
            for squad in [false, true] {
                for &(section, team_tab, tactic_tab) in &screens {
                    cx.update_window(handle.into(), |_, window, cx| {
                        app.update(cx, |app, cx| {
                            if squad && app.state.squad.is_empty() {
                                load_sample_squad(app, cx);
                            }
                            app.section = section;
                            if let Some(t) = team_tab {
                                app.team_tab = t;
                            }
                            if let Some(t) = tactic_tab {
                                app.tactic_tab = t;
                            }
                        });
                        window.draw(cx).clear(cx);
                    })
                    .unwrap_or_else(|e| panic!("{lang:?} {section:?} {team_tab:?} {tactic_tab:?}: {e}"));
                }
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Visual review: renders every screen with the real Metal renderer into
    /// `target/screens/<w>x<h>-<lang>-<screen>.png` (dark theme, sample squad).
    /// Run with `cargo test -p ta-app screenshots -- --ignored`.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore]
    fn screenshots() {
        use gpui_kit::{px, size, HeadlessAppContext};
        let text = std::sync::Arc::new(gpui_apple::AppleTextSystem::new());
        let mut cx = HeadlessAppContext::with_platform(
            text,
            std::sync::Arc::new(gpui_kit::assets::Assets),
            gpui_kit::platform::current_headless_renderer,
        );
        cx.update(gpui_kit::init);
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
        std::fs::create_dir_all(&out).unwrap();
        let dir = std::env::temp_dir().join(format!("ta-app-shots-{}", std::process::id()));
        let only = std::env::var("TA_SHOTS").ok();
        for (w, h) in [(1000.0, 680.0), (1000.0, 2400.0), (1440.0, 2400.0)] {
            for lang in [Lang::En, Lang::Pt] {
                let mut state = AppState::load_from(StateStore::new(dir.join("state.json")));
                state.lang = lang;
                state.dark_mode = true;
                cx.update(|cx| state.apply_theme(None, cx));
                let mut app = None;
                let win = cx
                    .open_window(size(px(w), px(h)), |window, cx| {
                        let v = cx.new(|cx| ArchitectApp::with_state(state, window, cx));
                        app = Some(v.clone());
                        cx.new(|cx| gpui_kit::base::Root::new(v, window, cx))
                    })
                    .unwrap();
                let app = app.unwrap();
                for (name, section, team_tab, tactic_tab) in screen_list() {
                    if only.as_deref().is_some_and(|o| !name.contains(o)) {
                        continue;
                    }
                    cx.update_window(win.into(), |_, window, cx| {
                        app.update(cx, |app, cx| {
                            if app.state.squad.is_empty() {
                                load_sample_squad(app, cx);
                            }
                            app.state.toast = None;
                            app.section = section;
                            if let Some(t) = team_tab {
                                app.team_tab = t;
                            }
                            if let Some(t) = tactic_tab {
                                app.tactic_tab = t;
                            }
                            app.team_ui.squad_mode = if name == "team-Squad-personality" {
                                crate::views::team::SquadTableMode::Personality
                            } else {
                                crate::views::team::SquadTableMode::Roles
                            };
                            app.gen_ui.result = (name == "team-Generator-result")
                                .then(|| {
                                    ta_core::planning::generate_tactic(
                                        &app.state.squad,
                                        &app.gen_ui.config,
                                        app.state.lang,
                                        |_| {},
                                    )
                                })
                                .flatten();
                        });
                        window.refresh();
                        window.draw(cx).clear(cx);
                    })
                    .unwrap();
                    cx.run_until_parked();
                    cx.update_window(win.into(), |_, window, cx| {
                        window.refresh();
                        window.draw(cx).clear(cx);
                    })
                    .unwrap();
                    let img = cx.capture_screenshot(win.into()).unwrap();
                    img.save(out.join(format!("{w}x{h}-{lang:?}-{name}.png"))).unwrap();
                }
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    fn screen_list() -> Vec<(String, NavSection, Option<TeamTab>, Option<TacticTab>)> {
        let mut v = vec![("home".to_string(), NavSection::Home, None, None)];
        v.extend(TeamTab::ALL.iter().map(|t| (format!("team-{t:?}"), NavSection::Team, Some(*t), None)));
        v.push(("team-Generator-result".into(), NavSection::Team, Some(TeamTab::Generator), None));
        v.push(("team-Squad-personality".into(), NavSection::Team, Some(TeamTab::Squad), None));
        v.push(("scouting".into(), NavSection::Scouting, None, None));
        v.extend(TacticTab::ALL.iter().map(|t| (format!("tactic-{t:?}"), NavSection::Tactic, None, Some(*t))));
        v.push(("training".into(), NavSection::Training, None, None));
        v.push(("settings".into(), NavSection::Settings, None, None));
        v
    }

    #[test]
    fn navigation_tabs_roundtrip_and_localize() {
        for (i, tab) in TeamTab::ALL.iter().enumerate() {
            assert_eq!(tab.index(), i);
            assert_eq!(TeamTab::from_index(i), *tab);
            assert!(!tab.label(Lang::Pt).is_empty());
            assert!(!tab.label(Lang::En).is_empty());
        }
        for (i, tab) in TacticTab::ALL.iter().enumerate() {
            assert_eq!(tab.index(), i);
            assert_eq!(TacticTab::from_index(i), *tab);
            assert!(!tab.label(Lang::Pt).is_empty());
            assert!(!tab.label(Lang::En).is_empty());
        }
        assert_eq!(NavSection::ALL.len(), 6);
        assert_ne!(
            NavSection::Scouting.label(Lang::Pt),
            NavSection::Scouting.label(Lang::En)
        );
    }
}
