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
    TacticTab, TeamTab, ToastKind, ToggleSidebar,
};
use crate::views::fm_bridge_ui::spawn_refresh_bridge_status;
use crate::views::tactic_sheet::SheetViewState;
use crate::views::generator::{render_generator, GeneratorViewState};
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
use crate::widgets::common::{busy_floating_card, busy_top_bar, toast_banner};
use crate::widgets::pitch_animated::AnimatedPitchView;
use crate::widgets::pitch_interactive::PitchPhaseMode;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::path::PathBuf;
use ta_backend::{export_save, latest_save, plugin_install, plugin_status, PLUGIN_VERSION};
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

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let lang = self.state.lang;
        let current = self.section;
        let collapsed = self.state.sidebar_collapsed;

        let mut nav_list = div().v_flex().gap_1().flex_1();
        for section in NavSection::ALL {
            let active = section == current;
            let label = if collapsed {
                section.icon().to_string()
            } else {
                format!("{}  {}", section.icon(), section.label(lang))
            };
            let btn = Button::new(("nav", section as usize))
                .w_full()
                .when(!collapsed, |b| b.justify_start())
                .label(label)
                .when(active, |b| b.primary())
                .when(!active, |b| b.ghost())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.section = section;
                    cx.notify();
                }));
            nav_list = nav_list.child(btn);
        }

        div()
            .v_flex()
            .w(px(if collapsed { 64.0 } else { 232.0 }))
            .h_full()
            .p_3()
            .gap_3()
            .bg(theme.secondary)
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .pb_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .when(!collapsed, |d| {
                        d.child(
                            div()
                                .v_flex()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_base()
                                        .font_semibold()
                                        .text_color(theme.foreground)
                                        .child("FM27 Tactical Architect"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(format!("v{} · GPUI Native", APP_VERSION)),
                                ),
                        )
                    })
                    .child(
                        Button::new("collapse-sidebar")
                            .ghost()
                            .xsmall()
                            .label(if collapsed { "»" } else { "«" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.sidebar_collapsed = !this.state.sidebar_collapsed;
                                this.state.persist();
                                cx.notify();
                            })),
                    ),
            )
            .child(nav_list)
            .when(!collapsed, |d| {
                d.child(
                    div()
                        .v_flex()
                        .gap_1()
                        .pt_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!("Bridge DLL v{}", PLUGIN_VERSION)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(if self.state.plugin_info.installed {
                                    lang.pick("Plugin: instalado", "Plugin: installed")
                                } else {
                                    lang.pick("Plugin: não detetado", "Plugin: not detected")
                                }),
                        ),
                )
            })
    }

    fn render_topbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let lang = self.state.lang;

        let save_summary = match (&self.state.last_save_read, &self.state.last_save_info) {
            (Some(msg), _) => msg.clone(),
            (None, Some(info)) => format!("{} ({:.1} MB)", info.name, info.size as f64 / 1_048_576.0),
            (None, None) => lang
                .pick("Sem save carregado", "No save loaded")
                .to_string(),
        };

        let mut bar = div()
            .v_flex()
            .w_full()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .when_some(self.state.busy.as_ref(), |d, busy| {
                d.child(busy_top_bar(busy, cx))
            })
            .child(
                div()
                    .h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .gap_3()
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_semibold()
                                    .text_color(theme.foreground)
                                    .child(self.section.label(lang)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(save_summary),
                            ),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("read-save-now")
                                    .primary()
                                    .small()
                                    .label(lang.pick("Ler save agora", "Read save now"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.spawn_read_save_now(cx);
                                    })),
                            )
                            .child(
                                Button::new("toggle-lang")
                                    .outline()
                                    .small()
                                    .label(match lang {
                                        Lang::Pt => "PT → EN",
                                        Lang::En => "EN → PT",
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let next = this.state.lang.toggle();
                                        this.set_lang(next, cx);
                                    })),
                            )
                            .child(
                                Button::new("toggle-theme")
                                    .outline()
                                    .small()
                                    .label(if self.state.dark_mode {
                                        lang.pick("Claro", "Light")
                                    } else {
                                        lang.pick("Escuro", "Dark")
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.set_dark_mode(!this.state.dark_mode, window, cx);
                                    })),
                            ),
                    ),
            );

        match self.section {
            NavSection::Team => {
                let mut tabs = TabBar::new("team-tabs")
                    .selected_index(self.team_tab.index())
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.team_tab = TeamTab::from_index(*ix);
                        cx.notify();
                    }));
                for tab in TeamTab::ALL {
                    tabs = tabs.child(Tab::new().label(tab.label(lang)));
                }
                bar = bar.child(div().px_4().pb_1().child(tabs));
            }
            NavSection::Tactic => {
                let mut tabs = TabBar::new("tactic-tabs")
                    .selected_index(self.tactic_tab.index())
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.tactic_tab = TacticTab::from_index(*ix);
                        cx.notify();
                    }));
                for tab in TacticTab::ALL {
                    tabs = tabs.child(Tab::new().label(tab.label(lang)));
                }
                bar = bar.child(div().px_4().pb_1().child(tabs));
            }
            _ => {}
        }

        bar
    }

    fn render_content(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.section {
            NavSection::Home => render_home(self, cx).into_any_element(),
            NavSection::Team if self.team_tab == TeamTab::Generator => {
                render_generator(self, cx).into_any_element()
            }
            NavSection::Team => render_team_tab(self, cx),
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
        let content = self.render_content(window, cx);
        let theme = cx.theme();

        div()
            .id("architect-root")
            .key_context("ArchitectApp")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.state.sidebar_collapsed = !this.state.sidebar_collapsed;
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
            .h_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(self.render_topbar(cx))
                    .child(
                        div()
                            .id("main-scroll-area")
                            .flex_1()
                            .size_full()
                            .overflow_y_scroll()
                            .p_5()
                            .child(content),
                    ),
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
    gpui_kit::application().run(|cx| {
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
