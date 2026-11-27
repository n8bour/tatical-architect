//! Complete live Settings screen wired to `ta-backend`:
//! - `fmsave` save folder picker, "Read save now", auto-poll switch, `uv`/fmsave `open_url` links
//! - FM27 Bridge (`TacticalBibleBridge.dll`) status, FM27 folder picker, install/update, live TCP ping
//! - My Club input, Dark/Light theme toggle, EN/PT language switcher
//! - Export JSON backup, Import JSON / legacy `.fm27tb.json` backup, Wipe local data with confirmation dialog

use crate::state::{non_empty, ToastKind};
use crate::widgets::common::{badge_pill, collapsible_card, BadgeTone};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt, WindowExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use ta_backend::{open_url, pick_folder, pick_open_file, pick_save_file, PLUGIN_VERSION};
use ta_core::types::Lang;
use ta_core::APP_VERSION;

pub const URL_FMSAVE: &str = "https://github.com/rhiever/fmsave";
pub const URL_FMSAVE_WEB: &str = "https://rhiever.github.io/fmsave/app/";
pub const URL_UV: &str = "https://docs.astral.sh/uv/getting-started/installation/";
pub const URL_BRIDGE_RELEASES: &str = "https://github.com/n8bour/tatical-architect/releases";
pub const URL_BEPINEX: &str = "https://builds.bepinex.dev/projects/bepinex_be";

/// Persistent `InputState` handles owned by the root app for the Settings screen.
pub struct SettingsInputs {
    pub save_dir: Entity<InputState>,
    pub fm_dir: Entity<InputState>,
    pub my_club: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl SettingsInputs {
    pub fn new(
        initial_save_dir: &str,
        initial_fm_dir: &str,
        initial_club: &str,
        window: &mut Window,
        cx: &mut Context<ArchitectApp>,
    ) -> Self {
        let save_val = initial_save_dir.to_string();
        let fm_val = initial_fm_dir.to_string();
        let club_val = initial_club.to_string();

        let save_dir = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Documents\\Sports Interactive\\Football Manager 27\\games")
                .default_value(save_val)
        });
        let fm_dir = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("D:\\SteamLibrary\\steamapps\\common\\Football Manager 27")
                .default_value(fm_val)
        });
        let my_club = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("SL Benfica")
                .default_value(club_val)
        });

        let mut subs = Vec::new();
        subs.push(cx.subscribe(&save_dir, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let val = entity.read(cx).value().to_string();
                if this.state.custom_save_dir != val {
                    this.state.custom_save_dir = val;
                    this.state.persist();
                    cx.notify();
                }
            }
        }));
        subs.push(cx.subscribe(&fm_dir, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let val = entity.read(cx).value().to_string();
                if this.state.custom_fm_dir != val {
                    this.state.custom_fm_dir = val;
                    this.state.persist();
                    this.spawn_refresh_plugin_status(cx);
                }
            }
        }));
        subs.push(cx.subscribe(&my_club, |this, entity, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change | InputEvent::Blur) {
                let val = entity.read(cx).value().to_string();
                if this.state.my_club != val {
                    this.state.my_club = val;
                    this.state.persist();
                    cx.notify();
                }
            }
        }));

        Self {
            save_dir,
            fm_dir,
            my_club,
            _subs: subs,
        }
    }

    pub fn sync_from_state(&self, app: &ArchitectApp, window: &mut Window, cx: &mut App) {
        let s_dir = app.state.custom_save_dir.clone();
        let f_dir = app.state.custom_fm_dir.clone();
        let club = app.state.my_club.clone();
        self.save_dir.update(cx, |st, cx| {
            if st.value().as_ref() != s_dir.as_str() {
                st.set_value(s_dir, window, cx);
            }
        });
        self.fm_dir.update(cx, |st, cx| {
            if st.value().as_ref() != f_dir.as_str() {
                st.set_value(f_dir, window, cx);
            }
        });
        self.my_club.update(cx, |st, cx| {
            if st.value().as_ref() != club.as_str() {
                st.set_value(club, window, cx);
            }
        });
    }
}

fn open_link_with_toast(app: &mut ArchitectApp, url: &'static str, cx: &mut Context<ArchitectApp>) {
    if let Err(err) = open_url(url) {
        app.state.set_toast(
            format!(
                "{}: {}",
                app.state
                    .lang
                    .pick("Não consegui abrir o link", "Could not open link"),
                err
            ),
            ToastKind::Error,
        );
        cx.notify();
    }
}

/// Renders the full Settings view.
pub fn render_settings(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();

    // ------------------------------------------------------------------------
    // 1. fmsave Card
    // ------------------------------------------------------------------------
    let save_badge = if app.state.auto_save_poll {
        badge_pill(lang.pick("Automático ligado", "Auto-poll on"), BadgeTone::Good, cx)
    } else {
        badge_pill(lang.pick("Manual", "Manual"), BadgeTone::Muted, cx)
    };
    let save_detected = match &app.state.last_save_info {
        Some(info) => format!(
            "{}: {} ({:.1} MB)",
            lang.pick("Último save detetado", "Latest detected save"),
            info.name,
            info.size as f64 / 1_048_576.0
        ),
        None => lang
            .pick(
                "Nenhum save .fm encontrado nas pastas habituais.",
                "No .fm save found in standard folders.",
            )
            .to_string(),
    };
    let save_read_line = app.state.last_save_read.as_ref().map(|r| {
        format!(
            "{}: {}",
            lang.pick("Última leitura", "Last read"),
            r
        )
    });
    let save_err = app.state.last_save_err.clone();

    let fmsave_body = div()
        .v_flex()
        .gap_3()
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(lang.pick(
                            "Lê o plantel diretamente do save (.fm), sempre sobre uma cópia temporária sem alterar o save.",
                            "Reads the squad directly from your .fm save, always using a temporary copy without modifying the save.",
                        )),
                )
                .child(save_badge),
        )
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("Pasta dos saves (opcional)", "Saves folder (optional)")),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(div().flex_1().child(Input::new(&app.settings_inputs.save_dir)))
                        .child(
                            Button::new("pick-save-dir")
                                .outline()
                                .small()
                                .label(lang.pick("Escolher…", "Browse…"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let title = this.state.lang.pick(
                                        "Escolher pasta de saves do FM27",
                                        "Choose FM27 saves folder",
                                    );
                                    if let Some(dir) = pick_folder(title) {
                                        this.state.custom_save_dir = dir.display().to_string();
                                        this.state.persist();
                                        this.settings_inputs.sync_from_state(this, window, cx);
                                        this.spawn_scan_latest_save(cx);
                                    }
                                })),
                        ),
                ),
        )
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.foreground)
                        .child(save_detected),
                )
                .when_some(save_read_line, |d, line| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(line),
                    )
                }),
        )
        .when_some(save_err, |d, err| {
            d.child(
                div()
                    .p_2()
                    .rounded_sm()
                    .border_1()
                    .border_color(rgb(0xd97706))
                    .bg(crate::widgets::common::with_alpha(rgb(0xd97706).into(), 0.14))
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(err),
            )
        })
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    Switch::new("auto-save-poll")
                        .checked(app.state.auto_save_poll)
                        .label(lang.pick(
                            "Ler automaticamente quando o FM grava",
                            "Auto-read whenever FM saves",
                        ))
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.state.auto_save_poll = *checked;
                            this.state.persist();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("settings-read-save")
                        .primary()
                        .small()
                        .label(lang.pick("Ler save agora", "Read save now"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.spawn_read_save_now(cx);
                        })),
                ),
        )
        .child(
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
                        .child(lang.pick(
                            "Requer uv (Windows: winget install --id=astral-sh.uv -e). O fmsave (MIT) é descarregado automaticamente na primeira leitura.",
                            "Requires uv (Windows: winget install --id=astral-sh.uv -e). fmsave (MIT) downloads automatically on the first read.",
                        )),
                )
                .child(
                    div()
                        .h_flex()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            Button::new("link-uv")
                                .outline()
                                .xsmall()
                                .label(lang.pick("Instalar uv ↗", "Install uv ↗"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    open_link_with_toast(this, URL_UV, cx);
                                })),
                        )
                        .child(
                            Button::new("link-fmsave")
                                .outline()
                                .xsmall()
                                .label("fmsave (GitHub) ↗")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    open_link_with_toast(this, URL_FMSAVE, cx);
                                })),
                        )
                        .child(
                            Button::new("link-fmsave-web")
                                .outline()
                                .xsmall()
                                .label(lang.pick("fmsave online ↗", "fmsave online ↗"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    open_link_with_toast(this, URL_FMSAVE_WEB, cx);
                                })),
                        ),
                ),
        );

    let fmsave_card = collapsible_card(
        "card-fmsave",
        lang.pick("Ler o save (.fm) — fmsave", "Read .fm save — fmsave"),
        Some(
            lang.pick(
                "Exportação automática do plantel e base de dados de prospeção",
                "Automatic squad & full scouting database export",
            )
            .into(),
        ),
        app.state.is_panel_collapsed("settings-fmsave"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("settings-fmsave");
            cx.notify();
        }),
        fmsave_body,
        cx,
    );

    // ------------------------------------------------------------------------
    // 2. FM27 Bridge Card (TacticalBibleBridge.dll)
    // ------------------------------------------------------------------------
    let pinfo = &app.state.plugin_info;
    let bstat = &app.state.bridge_status;

    let plugin_summary = if pinfo.fm_dir.is_none() {
        lang.pick(
            "FM27 não encontrado automaticamente nas pastas do Steam.",
            "FM27 not found automatically in Steam libraries.",
        )
        .to_string()
    } else if !pinfo.bepinex {
        lang.pick(
            "Falta o BepInEx 6 (IL2CPP) no FM27.",
            "BepInEx 6 (IL2CPP) is missing inside FM27.",
        )
        .to_string()
    } else if pinfo.up_to_date {
        format!(
            "{} v{} {}.",
            lang.pick("Plugin", "Plugin"),
            pinfo.version,
            lang.pick("atualizado no FM27", "up to date in FM27")
        )
    } else if pinfo.installed {
        format!(
            "{} (v{}).",
            lang.pick("Há uma versão mais recente do plugin", "A newer plugin version is bundled"),
            pinfo.version
        )
    } else {
        lang.pick(
            "O plugin ainda não está instalado no FM27.",
            "The plugin is not installed in FM27 yet.",
        )
        .to_string()
    };

    let can_install = pinfo.fm_dir.is_some() && pinfo.bepinex && !pinfo.up_to_date;
    let install_label = if pinfo.installed {
        lang.pick("Atualizar plugin no FM27", "Update plugin in FM27")
    } else {
        lang.pick("Instalar plugin no FM27", "Install plugin in FM27")
    };

    let bridge_pill = if bstat.ok && bstat.ready {
        badge_pill(
            format!(
                "{}{}",
                lang.pick("Ligado", "Connected"),
                if bstat.version.is_empty() {
                    String::new()
                } else {
                    format!(" v{}", bstat.version)
                }
            ),
            BadgeTone::Good,
            cx,
        )
    } else if bstat.ok {
        badge_pill(
            lang.pick("A aguardar FM27", "Waiting for FM27"),
            BadgeTone::Warn,
            cx,
        )
    } else {
        badge_pill(
            lang.pick("Desligado (127.0.0.1:27029)", "Offline (127.0.0.1:27029)"),
            BadgeTone::Muted,
            cx,
        )
    };

    let bridge_body = div()
        .v_flex()
        .gap_3()
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(plugin_summary),
                )
                .child(bridge_pill),
        )
        .when_some(pinfo.fm_dir.clone(), |d, dir| {
            d.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(format!("FM27: {}", dir)),
            )
        })
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("Pasta do FM27 (opcional)", "FM27 folder (optional)")),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(div().flex_1().child(Input::new(&app.settings_inputs.fm_dir)))
                        .child(
                            Button::new("pick-fm-dir")
                                .outline()
                                .small()
                                .label(lang.pick("Escolher…", "Browse…"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let title = this.state.lang.pick(
                                        "Escolher pasta de instalação do FM27",
                                        "Choose FM27 installation folder",
                                    );
                                    if let Some(dir) = pick_folder(title) {
                                        this.state.custom_fm_dir = dir.display().to_string();
                                        this.state.persist();
                                        this.settings_inputs.sync_from_state(this, window, cx);
                                        this.spawn_refresh_plugin_status(cx);
                                    }
                                })),
                        ),
                ),
        )
        .when_some(bstat.err.clone(), |d, err| {
            d.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(err),
            )
        })
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .when(can_install, |d| {
                    d.child(
                        Button::new("install-bridge-plugin")
                            .primary()
                            .small()
                            .label(install_label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.spawn_install_plugin(cx);
                            })),
                    )
                })
                .child(
                    Button::new("ping-bridge")
                        .outline()
                        .small()
                        .label(lang.pick("Verificar ligação (Ping)", "Check connection (Ping)"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.spawn_ping_bridge(cx);
                        })),
                )
                .child(
                    Button::new("refresh-plugin-status")
                        .ghost()
                        .small()
                        .label(lang.pick("Rever estado", "Re-check status"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.spawn_refresh_plugin_status(cx);
                        })),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .pt_2()
                .border_t_1()
                .border_color(theme.border)
                .child(
                    Button::new("link-bepinex")
                        .outline()
                        .xsmall()
                        .label("BepInEx 6 (IL2CPP) ↗")
                        .on_click(cx.listener(|this, _, _, cx| {
                            open_link_with_toast(this, URL_BEPINEX, cx);
                        })),
                )
                .child(
                    Button::new("link-releases")
                        .outline()
                        .xsmall()
                        .label(lang.pick("Releases & notas ↗", "Releases & notes ↗"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            open_link_with_toast(this, URL_BRIDGE_RELEASES, cx);
                        })),
                ),
        );

    let bridge_card = collapsible_card(
        "card-bridge",
        format!(
            "{} (TacticalBibleBridge.dll v{})",
            lang.pick("Ponte FM27 (BepInEx)", "FM27 Bridge (BepInEx)"),
            PLUGIN_VERSION
        ),
        Some(
            lang.pick(
                "Plugin Windows opcional em 127.0.0.1:27029 — aplica XI, tática, bolas paradas e shortlist",
                "Optional Windows plugin on 127.0.0.1:27029 — applies XI, tactic, set pieces & shortlist",
            )
            .into(),
        ),
        app.state.is_panel_collapsed("settings-bridge"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("settings-bridge");
            cx.notify();
        }),
        bridge_body,
        cx,
    );

    // ------------------------------------------------------------------------
    // 3. Club, Language & Appearance Card
    // ------------------------------------------------------------------------
    let club_body = div()
        .v_flex()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("O meu clube", "My club")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(lang.pick(
                            "Exportações de outros clubes entram automaticamente como adversário.",
                            "Exports from other clubs are automatically treated as opponent squads.",
                        )),
                )
                .child(Input::new(&app.settings_inputs.my_club)),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap_3()
                .pt_2()
                .border_t_1()
                .border_color(theme.border)
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(lang.pick("Idioma", "Language")),
                        )
                        .child(
                            div()
                                .h_flex()
                                .gap_1()
                                .child(
                                    Button::new("set-lang-pt")
                                        .small()
                                        .label("Português (PT)")
                                        .when(lang == Lang::Pt, |b| b.primary())
                                        .when(lang != Lang::Pt, |b| b.outline())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_lang(Lang::Pt, cx);
                                        })),
                                )
                                .child(
                                    Button::new("set-lang-en")
                                        .small()
                                        .label("English (EN)")
                                        .when(lang == Lang::En, |b| b.primary())
                                        .when(lang != Lang::En, |b| b.outline())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.set_lang(Lang::En, cx);
                                        })),
                                ),
                        ),
                )
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(lang.pick("Tema", "Theme")),
                        )
                        .child(
                            div()
                                .h_flex()
                                .gap_1()
                                .child(
                                    Button::new("set-theme-dark")
                                        .small()
                                        .label(lang.pick("Escuro", "Dark"))
                                        .when(app.state.dark_mode, |b| b.primary())
                                        .when(!app.state.dark_mode, |b| b.outline())
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.set_dark_mode(true, window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("set-theme-light")
                                        .small()
                                        .label(lang.pick("Claro", "Light"))
                                        .when(!app.state.dark_mode, |b| b.primary())
                                        .when(app.state.dark_mode, |b| b.outline())
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.set_dark_mode(false, window, cx);
                                        })),
                                ),
                        ),
                ),
        );

    let club_card = collapsible_card(
        "card-club-appearance",
        lang.pick("Clube, Idioma & Tema", "Club, Language & Theme"),
        None,
        app.state.is_panel_collapsed("settings-club"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("settings-club");
            cx.notify();
        }),
        club_body,
        cx,
    );

    // ------------------------------------------------------------------------
    // 4. Backup, Legacy Import & Wipe Card
    // ------------------------------------------------------------------------
    let store_path = app.state.store_path_display();
    let entity_handle = cx.entity().downgrade();

    let backup_body = div()
        .v_flex()
        .gap_3()
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!(
                    "{}: {}",
                    lang.pick("Ficheiro local de estado", "Local state file"),
                    store_path
                )),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("export-backup")
                        .outline()
                        .small()
                        .label(lang.pick("Exportar cópia JSON", "Export JSON backup"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            let title = this
                                .state
                                .lang
                                .pick("Guardar cópia de segurança", "Save backup");
                            if let Some(path) = pick_save_file(
                                title,
                                "fm27tb_backup.json",
                                &[("JSON", &["json"])],
                            ) {
                                match this.state.export_backup_json().and_then(|json| {
                                    std::fs::write(&path, json).map_err(|e| e.to_string())
                                }) {
                                    Ok(()) => this.state.set_toast(
                                        format!(
                                            "{}: {}",
                                            this.state.lang.pick("Cópia guardada", "Backup saved"),
                                            path.display()
                                        ),
                                        ToastKind::Success,
                                    ),
                                    Err(err) => this.state.set_toast(err, ToastKind::Error),
                                }
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Button::new("import-backup")
                        .outline()
                        .small()
                        .label(lang.pick(
                            "Importar JSON / .fm27tb.json",
                            "Import JSON / .fm27tb.json",
                        ))
                        .on_click(cx.listener(|this, _, window, cx| {
                            let title = this.state.lang.pick(
                                "Escolher cópia JSON ou .fm27tb.json",
                                "Choose JSON or .fm27tb.json backup",
                            );
                            if let Some(path) =
                                pick_open_file(title, &[("JSON", &["json"])])
                            {
                                match this.state.import_backup_path(&path) {
                                    Ok(msg) => {
                                        this.state.apply_theme(Some(window), cx);
                                        this.settings_inputs.sync_from_state(this, window, cx);
                                        this.state.set_toast(msg, ToastKind::Success);
                                    }
                                    Err(err) => this.state.set_toast(err, ToastKind::Error),
                                }
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Button::new("wipe-local-data")
                        .danger()
                        .small()
                        .label(lang.pick("Apagar dados locais", "Wipe local data"))
                        .on_click(move |_, window, cx| {
                            let weak = entity_handle.clone();
                            let title_str = lang.pick(
                                "Apagar todos os dados locais?",
                                "Wipe all local data?",
                            );
                            let desc_str = lang.pick(
                                "Isto remove o plantel, táticas guardadas, adversário, shortlist e definições deste computador. Não altera os teus saves do FM27.",
                                "This removes squad, saved tactics, opponent, shortlist, and preferences on this computer. Your FM27 saves are untouched.",
                            );
                            window.open_alert_dialog(cx, move |alert, _, _| {
                                let weak = weak.clone();
                                alert
                                    .confirm()
                                    .title(title_str)
                                    .description(desc_str)
                                    .show_cancel(true)
                                    .on_ok(move |_, window, cx| {
                                        let _ = weak.update(cx, |this, cx| {
                                            this.state.wipe_local_data();
                                            this.settings_inputs.sync_from_state(this, window, cx);
                                            cx.notify();
                                        });
                                        true
                                    })
                            });
                        }),
                ),
        )
        .child(
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
                        .child(lang.pick(
                            "Atalhos: Cmd/Ctrl+B recolher barra lateral · Cmd/Ctrl+Shift+L ler save agora · Arraste ficheiros (.csv/.html/.json/.fmf) para a janela.",
                            "Shortcuts: Cmd/Ctrl+B toggle sidebar · Cmd/Ctrl+Shift+L read save now · Drag files (.csv/.html/.json/.fmf) onto the window.",
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "FM27 Tactical Architect v{} · {}",
                            APP_VERSION,
                            lang.pick(
                                "não afiliado à Sports Interactive / SEGA.",
                                "not affiliated with Sports Interactive / SEGA.",
                            )
                        )),
                ),
        );

    let backup_card = collapsible_card(
        "card-backup",
        lang.pick("Cópia de segurança & Dados", "Backup & Local Data"),
        Some(
            lang.pick(
                "Compatível com cópias legadas fm27tb.v1 / .fm27tb.json",
                "Compatible with legacy fm27tb.v1 / .fm27tb.json backups",
            )
            .into(),
        ),
        app.state.is_panel_collapsed("settings-backup"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("settings-backup");
            cx.notify();
        }),
        backup_body,
        cx,
    );

    // Silence unused helper warning if compiler checks across cfg
    let _ = non_empty;

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(
            div()
                .h_flex()
                .w_full()
                .items_start()
                .gap_4()
                .child(div().v_flex().flex_1().gap_4().child(fmsave_card).child(club_card))
                .child(div().v_flex().flex_1().gap_4().child(bridge_card).child(backup_card)),
        )
}
