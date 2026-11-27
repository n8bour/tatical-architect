//! Tactic › Analyse: tactic header (name, IP/OOP formations, mentality), interactive pitch with
//! slot editor (roles, player, PIs), audit findings, role synergy across every phase, the team
//! instructions editor and the FM bridge actions (apply / create new tactic in FM).

use crate::state::{NavSection, TacticTab, ToastKind};
use crate::views::fm_bridge_ui::{fm_button, spawn_bridge_tactic};
use crate::views::tactic_sheet::spawn_print_sheet;
use crate::views::team::{apply_tactic_to_app, sync_animated_pitch};
use crate::widgets::common::{
    attr_color, badge_pill, score_color, score_ring, suitability_bar, with_alpha, BadgeTone,
};
use crate::widgets::pitch_interactive::{interactive_pitch, PitchPhaseMode};
use crate::widgets::synergy_map::dual_synergy_maps;
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt, Theme};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use ta_backend::{pick_open_file, pick_save_file};
use ta_core::data::{
    attr_name, group_name, group_of, is_attr_verified, l, mentality_info, mentality_name, pi_name,
    role_by_id, role_name, roles_for, slot_info, t as tr, t_name, ti_by_key, ti_name, ti_opt_name,
    ti_val, FM_SECTIONS, FORMATIONS, PLAYER_INSTRUCTIONS, TEMPLATES,
};
use ta_core::engine::{
    audit_tactic, auto_pick_xi, best_roles_for_player, new_tactic, recommend_pis, retarget_tactic,
    role_score, slot_score, synergy_part_name, synergy_phase_name, synergy_report,
    tactic_from_template,
};
use ta_core::types::{
    AppBackup, Attr, Lang, Mentality, Phase, Player, Role, Severity, SlotMeta,
    SynergyPhase, Tactic,
};

/// Analyse-tab UI state owned by the root app (`app.analyse_ui`).
#[derive(Default)]
pub struct AnalyseViewState {
    /// Tactic name input (created lazily on first render, needs a `Window`).
    name_input: Option<Entity<InputState>>,
    /// Last tactic name written into / read from the input.
    name_synced: String,
    _subs: Vec<Subscription>,
    /// Steps reported by the last successful "Apply in FM" / "Create new in FM".
    pub(crate) tac_done: Option<Vec<String>>,
    /// A bridge tactic write is running.
    pub(crate) bridge_running: bool,
    /// Team instructions whose explanation is expanded.
    ti_open: HashSet<&'static str>,
    /// "All player instructions" list expanded in the slot editor.
    show_all_pis: bool,
    /// "Main links" list expanded in the synergy panel.
    show_links: bool,
}

const GOOD: u32 = 0x16a34a;
const WARN: u32 = 0xd97706;
const CRIT: u32 = 0xdc2626;
const INFO: u32 = 0x2563eb;

// ============================================================================
// Mutation helpers
// ============================================================================

/// Persists the mutated `state.tactic`, re-syncs the animated pitch and re-renders.
fn commit_tactic(this: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    this.state.persist();
    sync_animated_pitch(this, cx);
    cx.notify();
}

fn set_ti(this: &mut ArchitectApp, key: &'static str, v: usize, cx: &mut Context<ArchitectApp>) {
    let Some(d) = ti_by_key(key) else { return };
    if v == d.def {
        this.state.tactic.ti.remove(key);
    } else {
        this.state.tactic.ti.insert(key.to_string(), v);
    }
    commit_tactic(this, cx);
}

fn set_player(this: &mut ArchitectApp, i: usize, pid: Option<String>, cx: &mut Context<ArchitectApp>) {
    let slots = &mut this.state.tactic.slots;
    if i >= slots.len() {
        return;
    }
    if pid.is_some() {
        if let Some(other) = slots
            .iter()
            .enumerate()
            .position(|(j, s)| j != i && s.pid == pid)
        {
            // swap: the player's previous slot takes this slot's player
            slots[other].pid = slots[i].pid.clone();
        }
    }
    slots[i].pid = pid;
    commit_tactic(this, cx);
}

/// Current UTC time as ISO-8601 (`2026-11-27T10:00:00Z`).
fn iso_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        sod / 3600,
        sod % 3600 / 60,
        sod % 60
    )
}

fn fmt_num(v: f32) -> String {
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

fn file_stem_for(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    if s.trim_matches('_').is_empty() {
        "tatica".to_string()
    } else {
        s
    }
}

// ============================================================================
// Small view helpers
// ============================================================================

fn panel(theme: &Theme) -> Div {
    div()
        .v_flex()
        .w_full()
        .gap_3()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
}

fn heading(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .text_sm()
        .font_semibold()
        .text_color(theme.foreground)
        .child(text.into())
}

fn muted(text: impl Into<SharedString>, theme: &Theme) -> Div {
    div()
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(text.into())
}

fn field(label: impl Into<SharedString>, control: impl IntoElement, theme: &Theme) -> Div {
    div()
        .v_flex()
        .flex_1()
        .min_w_0()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(label.into()),
        )
        .child(control)
}

/// Button with a dropdown menu of options; `on_pick(index)` runs on the root app.
fn select_menu(
    id: impl Into<ElementId>,
    current: impl Into<SharedString>,
    items: Vec<(SharedString, bool)>,
    entity: &Entity<ArchitectApp>,
    on_pick: impl Fn(usize, &mut ArchitectApp, &mut Context<ArchitectApp>) + 'static,
) -> impl IntoElement {
    let items = Rc::new(items);
    let on_pick = Rc::new(on_pick);
    let entity = entity.clone();
    Button::new(id)
        .outline()
        .small()
        .w_full()
        .justify_between()
        .dropdown_caret(true)
        .label(current)
        .dropdown_menu(move |mut menu, _, _| {
            menu = menu.scrollable(true).max_h(px(420.0)).min_w(px(220.0));
            for (ix, (label, checked)) in items.iter().enumerate() {
                let entity = entity.clone();
                let on_pick = on_pick.clone();
                menu = menu.item(PopupMenuItem::new(label.clone()).checked(*checked).on_click(
                    move |_, _, cx| {
                        let on_pick = on_pick.clone();
                        entity.update(cx, |this, cx| on_pick(ix, this, cx));
                    },
                ));
            }
            menu
        })
}

fn sev_icon(sev: Severity) -> impl IntoElement {
    let (sym, col) = match sev {
        Severity::Crit => ("!", CRIT),
        Severity::Warn => ("▲", WARN),
        Severity::Info => ("i", INFO),
        Severity::Good => ("✓", GOOD),
    };
    div()
        .flex_shrink_0()
        .size(px(22.0))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(col))
        .text_xs()
        .font_bold()
        .text_color(rgb(0xffffff))
        .child(sym)
}

/// Attribute chip coloured by the player's value (name only when there is no player).
fn attr_tag(attr: Attr, player: Option<&Player>, lang: Lang, theme: &Theme) -> Div {
    let name = attr_name(attr, lang);
    match player.and_then(|p| p.a.get(&attr).copied()) {
        Some(v) => {
            let col = attr_color(v);
            div()
                .px_2()
                .py(px(2.0))
                .rounded_sm()
                .border_1()
                .border_color(with_alpha(col, 0.45))
                .bg(with_alpha(col, 0.16))
                .text_xs()
                .text_color(theme.foreground)
                .child(format!("{name} {}", v.round() as i32))
        }
        None => div()
            .px_2()
            .py(px(2.0))
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(name),
    }
}

fn role_card(role: &Role, player: Option<&Player>, lang: Lang, theme: &Theme, cx: &App) -> Div {
    let phase_tone = if role.phase == Phase::Ip {
        BadgeTone::Good
    } else {
        BadgeTone::Info
    };
    let src = if is_attr_verified(role.id) {
        badge_pill("CONF", BadgeTone::Good, cx)
    } else {
        badge_pill("EST", BadgeTone::Muted, cx)
    };
    let mut key_row = div().h_flex().flex_wrap().items_center().gap_1().child(
        div()
            .text_xs()
            .font_semibold()
            .text_color(theme.foreground)
            .child(l(lang, "Chave:", "Key:")),
    );
    for &a in role.key {
        key_row = key_row.child(attr_tag(a, player, lang, theme));
    }
    let mut pref_row = div().h_flex().flex_wrap().items_center().gap_1().child(
        div()
            .text_xs()
            .font_semibold()
            .text_color(theme.foreground)
            .child(l(lang, "Preferidos:", "Preferred:")),
    );
    for &a in role.pref {
        pref_row = pref_row.child(attr_tag(a, player, lang, theme));
    }
    div()
        .v_flex()
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(role_name(role, lang)),
                )
                .child(badge_pill(
                    if role.phase == Phase::Ip { "IP" } else { "OOP" },
                    phase_tone,
                    cx,
                ))
                .child(src),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.foreground)
                .child(tr(lang, role.desc_pt)),
        )
        .child(muted(format!("→ {}", tr(lang, role.tip_pt)), theme))
        .child(key_row)
        .child(pref_row)
}

// ============================================================================
// Render
// ============================================================================

pub fn render_tactic_analyse(
    app: &mut ArchitectApp,
    window: &mut Window,
    cx: &mut Context<ArchitectApp>,
) -> impl IntoElement {
    // Tactic name input: created on first render, kept in sync with `state.tactic.name`.
    let cur_name = app.state.tactic.name.clone();
    match app.analyse_ui.name_input.clone() {
        None => {
            let init = cur_name.clone();
            let input = cx.new(|cx| InputState::new(window, cx).default_value(init));
            let sub = cx.subscribe(&input, |this, entity, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    let val = entity.read(cx).value().to_string();
                    this.analyse_ui.name_synced = val.clone();
                    if this.state.tactic.name != val {
                        this.state.tactic.name = val;
                        commit_tactic(this, cx);
                    }
                }
            });
            app.analyse_ui._subs.push(sub);
            app.analyse_ui.name_input = Some(input);
            app.analyse_ui.name_synced = cur_name;
        }
        Some(input) => {
            if app.analyse_ui.name_synced != cur_name {
                let v = cur_name.clone();
                input.update(cx, |st, cx| st.set_value(v, window, cx));
                app.analyse_ui.name_synced = cur_name;
            }
        }
    }
    let app: &ArchitectApp = app;

    let theme = cx.theme().clone();
    let entity = cx.entity();
    let lang = app.state.lang;
    let t = &app.state.tactic;
    let squad = &app.state.squad;
    let pmap: HashMap<&str, &Player> = squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let audit = audit_tactic(t, squad, lang);
    let syn = synergy_report(t, squad, lang);

    // ------------------------------------------------------------------ header
    let running = app.analyse_ui.bridge_running;
    let actions = div()
        .h_flex()
        .flex_wrap()
        .gap_2()
        .child(fm_button(
            app,
            "an-fm-new",
            l(lang, "Criar nova no FM", "Create new in FM"),
            l(
                lang,
                "Cria uma tática nova no FM (as outras ficam como estão) e aplica esta nela. O FM só aceita 3 táticas.",
                "Creates a new tactic in FM (the others stay as they are) and applies this one to it. FM only holds 3 tactics.",
            ),
            running,
            |this, cx| spawn_bridge_tactic(this, true, cx),
            cx,
        ))
        .child(fm_button(
            app,
            "an-fm-apply",
            l(lang, "Aplicar no FM", "Apply in FM"),
            l(
                lang,
                "Escreve esta tática na tática activa do FM: formação com bola, XI, papéis, mentalidade e instruções de equipa.",
                "Writes this tactic into FM’s active tactic: in-possession formation, XI, roles, mentality and team instructions.",
            ),
            running,
            |this, cx| spawn_bridge_tactic(this, false, cx),
            cx,
        ))
        .child(
            Button::new("an-print")
                .outline()
                .small()
                .label(l(lang, "Imprimir/PDF", "Print/PDF"))
                .tooltip(l(
                    lang,
                    "Abre a ficha e a análise no navegador, pronta a imprimir ou guardar em PDF.",
                    "Opens the sheet and the analysis in your browser, ready to print or save as PDF.",
                ))
                .on_click(cx.listener(|this, _, _, cx| spawn_print_sheet(this, true, cx))),
        )
        .child(
            Button::new("an-sim")
                .primary()
                .small()
                .label(l(lang, "▶ Ver em campo", "▶ View on pitch"))
                .on_click(cx.listener(|this, _, _, cx| {
                    sync_animated_pitch(this, cx);
                    this.section = NavSection::Tactic;
                    this.tactic_tab = TacticTab::AnimatedPitch;
                    cx.notify();
                })),
        )
        .child(
            Button::new("an-new")
                .outline()
                .small()
                .label(l(lang, "Nova", "New"))
                .on_click(cx.listener(|this, _, _, cx| {
                    let lang = this.state.lang;
                    let nt = new_tactic(&this.state.tactic.ip_form, &this.state.tactic.oop_form, lang);
                    apply_tactic_to_app(this, nt, false, cx);
                    this.selected_slot = None;
                })),
        )
        .child(
            Button::new("an-save")
                .outline()
                .small()
                .label(l(lang, "Guardar", "Save"))
                .on_click(cx.listener(|this, _, _, cx| {
                    let mut c = this.state.tactic.clone();
                    c.saved_at = Some(iso_now());
                    let name = c.name.clone();
                    this.state.saved_tactics.retain(|x| x.name != name);
                    this.state.saved_tactics.insert(0, c);
                    this.state.persist();
                    this.state.set_toast(
                        format!("{}{}", l(this.state.lang, "Tática guardada: ", "Tactic saved: "), name),
                        ToastKind::Success,
                    );
                    cx.notify();
                })),
        )
        .child(
            Button::new("an-export")
                .outline()
                .small()
                .label(l(lang, "Exportar JSON", "Export JSON"))
                .on_click(cx.listener(|this, _, _, cx| {
                    let lang = this.state.lang;
                    let default_name = format!("{}.fm27tb.json", file_stem_for(&this.state.tactic.name));
                    let Some(path) = pick_save_file(
                        l(lang, "Exportar tática", "Export tactic"),
                        &default_name,
                        &[("JSON", &["json"])],
                    ) else {
                        return;
                    };
                    let backup = AppBackup::Tactic { v: 1, tactic: this.state.tactic.clone() };
                    match serde_json::to_string_pretty(&backup)
                        .map_err(|e| e.to_string())
                        .and_then(|json| std::fs::write(&path, json).map_err(|e| e.to_string()))
                    {
                        Ok(()) => this.state.set_toast(
                            format!("{}: {}", l(lang, "Tática exportada", "Tactic exported"), path.display()),
                            ToastKind::Success,
                        ),
                        Err(err) => this.state.set_toast(err, ToastKind::Error),
                    }
                    cx.notify();
                })),
        )
        .child(
            Button::new("an-import")
                .outline()
                .small()
                .label(l(lang, "Importar JSON", "Import JSON"))
                .on_click(cx.listener(|this, _, window, cx| {
                    let lang = this.state.lang;
                    let Some(path) = pick_open_file(
                        l(lang, "Importar tática JSON", "Import tactic JSON"),
                        &[("JSON", &["json"])],
                    ) else {
                        return;
                    };
                    match this.state.import_backup_path(&path) {
                        Ok(msg) => {
                            this.state.apply_theme(Some(window), cx);
                            this.settings_inputs.sync_from_state(this, window, cx);
                            this.selected_slot = None;
                            sync_animated_pitch(this, cx);
                            this.state.set_toast(msg, ToastKind::Success);
                        }
                        Err(_) => this.state.set_toast(
                            l(lang, "JSON inválido", "Invalid JSON"),
                            ToastKind::Error,
                        ),
                    }
                    cx.notify();
                })),
        );

    let header = div()
        .h_flex()
        .flex_wrap()
        .w_full()
        .items_start()
        .justify_between()
        .gap_3()
        .child(
            div()
                .v_flex()
                .gap_1()
                .min_w_0()
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(theme.foreground)
                        .child(t_name(t, lang)),
                )
                .child(muted(
                    l(
                        lang,
                        "Clique num jogador do campo para mudar papéis, jogador e instruções individuais.",
                        "Click a player on the pitch to change roles, player and player instructions.",
                    ),
                    &theme,
                )),
        )
        .child(actions);

    let done_notice = app.analyse_ui.tac_done.as_ref().map(|done| {
        div()
            .h_flex()
            .w_full()
            .items_start()
            .justify_between()
            .gap_3()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(with_alpha(rgb(GOOD).into(), 0.45))
            .bg(with_alpha(rgb(GOOD).into(), 0.10))
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    .children(done.iter().map(|s| {
                        div().text_xs().text_color(theme.foreground).child(s.clone())
                    })),
            )
            .child(
                Button::new("an-done-close")
                    .ghost()
                    .xsmall()
                    .label("×")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.analyse_ui.tac_done = None;
                        cx.notify();
                    })),
            )
    });

    // ------------------------------------------------------------ tactic form
    let form_items: Vec<(SharedString, bool)> = FORMATIONS
        .iter()
        .map(|f| (SharedString::from(f.name), false))
        .collect();
    let ip_items: Vec<(SharedString, bool)> = form_items
        .iter()
        .map(|(n, _)| (n.clone(), n.as_ref() == t.ip_form))
        .collect();
    let oop_items: Vec<(SharedString, bool)> = form_items
        .iter()
        .map(|(n, _)| (n.clone(), n.as_ref() == t.oop_form))
        .collect();
    let ment_items: Vec<(SharedString, bool)> = Mentality::ALL
        .iter()
        .map(|&m| (SharedString::from(mentality_name(m, lang)), m == t.ment))
        .collect();
    let tpl_items: Vec<(SharedString, bool)> = TEMPLATES
        .iter()
        .map(|tp| {
            (
                SharedString::from(format!("{} · {}", tr(lang, tp.name_pt), tp.ip)),
                false,
            )
        })
        .collect();

    let mut form_panel = panel(&theme).child(field(
        l(lang, "Nome", "Name"),
        app.analyse_ui
            .name_input
            .as_ref()
            .map(|inp| Input::new(inp).into_any_element())
            .unwrap_or_else(|| div().into_any_element()),
        &theme,
    ));
    form_panel = form_panel
        .child(
            div()
                .h_flex()
                .gap_3()
                .child(field(
                    l(lang, "Formação com bola (IP)", "In-possession formation (IP)"),
                    select_menu("an-ip-form", t.ip_form.clone(), ip_items, &entity, |ix, this, cx| {
                        let f = FORMATIONS[ix].name;
                        if this.state.tactic.ip_form == f {
                            return;
                        }
                        let lang = this.state.lang;
                        let oop = this.state.tactic.oop_form.clone();
                        let nt = retarget_tactic(&this.state.tactic, f, &oop, lang);
                        apply_tactic_to_app(this, nt, false, cx);
                        this.selected_slot = None;
                    }),
                    &theme,
                ))
                .child(field(
                    l(lang, "Formação sem bola (OOP)", "Out-of-possession formation (OOP)"),
                    select_menu("an-oop-form", t.oop_form.clone(), oop_items, &entity, |ix, this, cx| {
                        let f = FORMATIONS[ix].name;
                        if this.state.tactic.oop_form == f {
                            return;
                        }
                        let lang = this.state.lang;
                        let ip = this.state.tactic.ip_form.clone();
                        let nt = retarget_tactic(&this.state.tactic, &ip, f, lang);
                        apply_tactic_to_app(this, nt, false, cx);
                        this.selected_slot = None;
                    }),
                    &theme,
                )),
        )
        .child(
            div()
                .h_flex()
                .gap_3()
                .child(field(
                    l(lang, "Mentalidade", "Mentality"),
                    select_menu("an-ment", mentality_name(t.ment, lang), ment_items, &entity, |ix, this, cx| {
                        this.state.tactic.ment = Mentality::ALL[ix];
                        commit_tactic(this, cx);
                    }),
                    &theme,
                ))
                .child(field(
                    l(lang, "Começar de um modelo", "Start from a template"),
                    select_menu(
                        "an-template",
                        l(lang, "Escolher modelo…", "Choose template…"),
                        tpl_items,
                        &entity,
                        |ix, this, cx| {
                            let tp = &TEMPLATES[ix];
                            let lang = this.state.lang;
                            let mut nt = tactic_from_template(tp);
                            nt.name = tr(lang, tp.name_pt);
                            if this.state.squad.len() >= 11 {
                                auto_pick_xi(&mut nt, &this.state.squad);
                            }
                            apply_tactic_to_app(this, nt, false, cx);
                            this.selected_slot = None;
                        },
                    ),
                    &theme,
                )),
        )
        .child(muted(mentality_info(t.ment, lang), &theme));

    // ------------------------------------------------------------------ pitch
    let slot_scores: Vec<u8> = t
        .slots
        .iter()
        .map(|s| {
            s.pid
                .as_deref()
                .and_then(|pid| pmap.get(pid).copied())
                .map_or(0, |p| slot_score(p, t, s))
        })
        .collect();
    let pitch = interactive_pitch(
        t,
        squad,
        (!squad.is_empty()).then_some(slot_scores.as_slice()),
        app.pitch_mode,
        app.selected_slot,
        lang,
        cx.listener(|this, mode: &PitchPhaseMode, _, cx| {
            this.pitch_mode = *mode;
            cx.notify();
        }),
        cx.listener(|this, idx: &usize, _, cx| {
            this.selected_slot = if this.selected_slot == Some(*idx) { None } else { Some(*idx) };
            cx.notify();
        }),
        cx,
    );
    let xi_tools = if squad.is_empty() {
        div().child(muted(
            l(
                lang,
                "Carregue o plantel para atribuir jogadores",
                "Load the squad to assign players",
            ),
            &theme,
        ))
    } else {
        div()
            .h_flex()
            .gap_2()
            .child(
                Button::new("an-auto-xi")
                    .outline()
                    .xsmall()
                    .label("Auto XI")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let lang = this.state.lang;
                        if this.state.squad.len() < 11 {
                            this.state.set_toast(
                                l(
                                    lang,
                                    "Precisa de pelo menos 11 jogadores no plantel",
                                    "You need at least 11 players in the squad",
                                ),
                                ToastKind::Warn,
                            );
                            cx.notify();
                            return;
                        }
                        let squad = std::mem::take(&mut this.state.squad);
                        auto_pick_xi(&mut this.state.tactic, &squad);
                        this.state.squad = squad;
                        this.state.set_toast(
                            l(lang, "XI escolhido pela melhor adequação", "XI picked by best fit"),
                            ToastKind::Success,
                        );
                        commit_tactic(this, cx);
                    })),
            )
            .child(
                Button::new("an-clear-xi")
                    .outline()
                    .xsmall()
                    .label(l(lang, "Limpar XI", "Clear XI"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        for s in &mut this.state.tactic.slots {
                            s.pid = None;
                        }
                        commit_tactic(this, cx);
                    })),
            )
    };
    let pitch_hint = format!(
        "{}{}",
        l(
            lang,
            "Clique num jogador para editar papéis e instruções.",
            "Click a player to edit roles and instructions.",
        ),
        if app.pitch_mode == PitchPhaseMode::Both {
            l(
                lang,
                " As setas mostram a deslocação para a posição sem bola.",
                " Arrows show the shift to the out-of-possession position.",
            )
        } else {
            ""
        }
    );
    let pitch_panel = panel(&theme)
        .items_center()
        .child(div().h_flex().w_full().justify_end().child(xi_tools))
        .child(pitch)
        .child(muted(pitch_hint, &theme));

    // ------------------------------------------------------------ slot editor
    let slot_editor = app
        .selected_slot
        .filter(|&i| i < t.slots.len())
        .map(|i| render_slot_editor(app, i, &entity, &theme, cx));

    let left_col = div()
        .v_flex()
        .w(px(440.0))
        .flex_shrink_0()
        .gap_3()
        .child(form_panel)
        .child(pitch_panel)
        .children(slot_editor);

    // ------------------------------------------------------------------ audit
    let mut bars = div().v_flex().flex_1().min_w_0().gap_1();
    for (cat, v) in &audit.scores {
        bars = bars.child(suitability_bar(cat.label(lang), *v, 100, cx));
    }
    let cxr = &audit.complexity;
    let mut cx_line = format!(
        "{} {} ({})",
        l(lang, "Complexidade:", "Complexity:"),
        cxr.level,
        fmt_num(cxr.cx)
    );
    if !cxr.parts.is_empty() {
        let parts: Vec<&str> = cxr.parts.iter().take(4).map(|p| p.0.as_str()).collect();
        cx_line.push_str(" — ");
        cx_line.push_str(&parts.join("; "));
        if cxr.parts.len() > 4 {
            cx_line.push('…');
        }
    }
    let audit_panel = panel(&theme)
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_4()
                .child(score_ring(audit.overall, 88.0, cx))
                .child(bars),
        )
        .child(muted(cx_line, &theme));

    let mut findings_panel = panel(&theme).child(heading(l(lang, "Diagnóstico", "Diagnosis"), &theme));
    for f in &audit.findings {
        findings_panel = findings_panel.child(
            div()
                .h_flex()
                .items_start()
                .gap_3()
                .child(sev_icon(f.sev))
                .child(
                    div()
                        .v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(
                            div()
                                .h_flex()
                                .flex_wrap()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(theme.foreground)
                                        .child(f.title.clone()),
                                )
                                .child(badge_pill(f.cat.label(lang), BadgeTone::Muted, cx)),
                        )
                        .child(div().text_xs().text_color(theme.foreground).child(f.text.clone()))
                        .when_some(f.fix.clone(), |d, fix| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.primary)
                                    .child(format!("→ {fix}")),
                            )
                        }),
                ),
        );
    }

    // ---------------------------------------------------------------- synergy
    let synergy_panel = render_synergy(app, t, &syn, squad.is_empty(), &theme, cx);

    let right_col = div()
        .v_flex()
        .flex_1()
        .min_w(px(420.0))
        .gap_3()
        .child(audit_panel)
        .child(findings_panel)
        .child(synergy_panel);

    // ----------------------------------------------------- team instructions
    let ti_panel = render_ti_editor(app, t, &entity, &theme, cx);

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .children(done_notice)
        .child(
            div()
                .h_flex()
                .w_full()
                .items_start()
                .gap_4()
                .child(left_col)
                .child(right_col),
        )
        .child(ti_panel)
}

fn render_slot_editor(
    app: &ArchitectApp,
    i: usize,
    entity: &Entity<ArchitectApp>,
    theme: &Theme,
    cx: &mut Context<ArchitectApp>,
) -> Div {
    let lang = app.state.lang;
    let t = &app.state.tactic;
    let squad = &app.state.squad;
    let s = &t.slots[i];
    let meta = |form: &str, code: &str| -> SlotMeta {
        slot_info(form, code).unwrap_or_else(|| SlotMeta {
            code: code.to_string(),
            x: 50.0,
            y: 50.0,
            group: group_of(code),
            side: ta_core::types::Side::C,
            wide_cb: false,
        })
    };
    let a = meta(&t.ip_form, &s.ip_code);
    let b = meta(&t.oop_form, &s.oop_code);
    let player = s
        .pid
        .as_deref()
        .and_then(|pid| squad.iter().find(|p| p.id == pid));
    let ip_roles = roles_for(a.group, Phase::Ip, Some(&a.code), a.wide_cb);
    let oop_roles = roles_for(b.group, Phase::Oop, Some(&b.code), b.wide_cb);
    let role_label = |r: &Role| -> String {
        match player.and_then(|p| role_score(p, r)) {
            Some(f) => format!("{} — {}", role_name(r, lang), f.score),
            None => role_name(r, lang),
        }
    };
    let ri = role_by_id(&s.ip);
    let ro = role_by_id(&s.oop);

    let mut ed = panel(theme).child(
        div()
            .h_flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(s.ip_code.clone()),
                    )
                    .child(muted(
                        format!("→ {} {}", l(lang, "sem bola", "out of possession"), s.oop_code),
                        theme,
                    )),
            )
            .child(
                Button::new("an-slot-close")
                    .ghost()
                    .xsmall()
                    .label(l(lang, "Fechar", "Close"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.selected_slot = None;
                        cx.notify();
                    })),
            ),
    );

    // Player picker (sorted by fit for this slot; picking an assigned player swaps them).
    if !squad.is_empty() {
        let mut plist: Vec<(&Player, u8)> = squad.iter().map(|q| (q, slot_score(q, t, s))).collect();
        plist.sort_by(|x, y| y.1.cmp(&x.1));
        let mut items: Vec<(SharedString, bool)> = vec![(
            SharedString::from(l(lang, "— vazio —", "— empty —")),
            s.pid.is_none(),
        )];
        let mut ids: Vec<Option<String>> = vec![None];
        for (q, sc) in &plist {
            let mut label = format!("{} ({})", q.name, sc);
            if !q.pos_raw.is_empty() {
                label.push_str(" · ");
                label.push_str(&q.pos_raw);
            }
            items.push((SharedString::from(label), s.pid.as_deref() == Some(q.id.as_str())));
            ids.push(Some(q.id.clone()));
        }
        let current = player
            .map(|p| p.name.clone())
            .unwrap_or_else(|| l(lang, "— vazio —", "— empty —").to_string());
        ed = ed.child(field(
            l(lang, "Jogador", "Player"),
            select_menu(("an-player", i), current, items, entity, move |ix, this, cx| {
                set_player(this, i, ids[ix].clone(), cx);
            }),
            theme,
        ));
    }

    let ip_items: Vec<(SharedString, bool)> = ip_roles
        .iter()
        .map(|r| (SharedString::from(role_label(r)), r.id == s.ip))
        .collect();
    let oop_items: Vec<(SharedString, bool)> = oop_roles
        .iter()
        .map(|r| (SharedString::from(role_label(r)), r.id == s.oop))
        .collect();
    let ip_ids: Vec<&'static str> = ip_roles.iter().map(|r| r.id).collect();
    let oop_ids: Vec<&'static str> = oop_roles.iter().map(|r| r.id).collect();
    ed = ed.child(
        div()
            .h_flex()
            .gap_3()
            .child(field(
                format!("{} ({})", l(lang, "Papel com bola", "In-possession role"), group_name(a.group, lang)),
                select_menu(
                    ("an-role-ip", i),
                    ri.map(&role_label).unwrap_or_else(|| s.ip.clone()),
                    ip_items,
                    entity,
                    move |ix, this, cx| {
                        if let Some(slot) = this.state.tactic.slots.get_mut(i) {
                            slot.ip = ip_ids[ix].to_string();
                            commit_tactic(this, cx);
                        }
                    },
                ),
                theme,
            ))
            .child(field(
                format!("{} ({})", l(lang, "Papel sem bola", "Out-of-possession role"), group_name(b.group, lang)),
                select_menu(
                    ("an-role-oop", i),
                    ro.map(&role_label).unwrap_or_else(|| s.oop.clone()),
                    oop_items,
                    entity,
                    move |ix, this, cx| {
                        if let Some(slot) = this.state.tactic.slots.get_mut(i) {
                            slot.oop = oop_ids[ix].to_string();
                            commit_tactic(this, cx);
                        }
                    },
                ),
                theme,
            )),
    );

    // Suitability and best IP roles for this player here.
    if let Some(p) = player {
        let sc = slot_score(p, t, s);
        let best = best_roles_for_player(p, a.group, Phase::Ip, Some(&a.code), a.wide_cb, 3);
        let best_txt: Vec<String> = best
            .iter()
            .filter_map(|c| role_by_id(c.role_id).map(|r| format!("{} {}", role_name(r, lang), c.score)))
            .collect();
        ed = ed.child(
            div()
                .h_flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(muted(l(lang, "Adequação:", "Suitability:"), theme))
                .child(
                    div()
                        .px_2()
                        .rounded_full()
                        .bg(score_color(sc))
                        .text_xs()
                        .font_bold()
                        .text_color(rgb(0xffffff))
                        .child(sc.to_string()),
                )
                .child(muted(
                    format!(
                        "· {} {}",
                        l(lang, "melhores papéis IP aqui:", "best IP roles here:"),
                        best_txt.join(" · ")
                    ),
                    theme,
                )),
        );
    }

    if let Some(r) = ri {
        ed = ed.child(role_card(r, player, lang, theme, cx));
    }
    if let Some(r) = ro {
        ed = ed.child(role_card(r, player, lang, theme, cx));
    }

    // Recommended player instructions.
    if let Some(p) = player {
        let recs = recommend_pis(p, ri, ro, a.group, lang);
        if !recs.is_empty() {
            let mut box_ = div()
                .v_flex()
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!("{} {}", l(lang, "Recomendadas para", "Recommended for"), p.name)),
                );
            for (k, r) in recs.iter().enumerate() {
                let applied = s.pis.iter().any(|x| x == r.pi);
                let pi: &'static str = r.pi;
                box_ = box_.child(
                    div()
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .text_color(theme.foreground)
                                .child(format!("{} — {}", pi_name(r.pi, lang), r.why)),
                        )
                        .child(if applied {
                            badge_pill(l(lang, "aplicada", "applied"), BadgeTone::Good, cx).into_any_element()
                        } else {
                            Button::new(("an-rec-pi", k))
                                .outline()
                                .xsmall()
                                .label(l(lang, "Aplicar", "Apply"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let Some(slot) = this.state.tactic.slots.get_mut(i) else { return };
                                    if !slot.pis.iter().any(|x| x == pi) {
                                        slot.pis.push(pi.to_string());
                                    }
                                    let lang = this.state.lang;
                                    this.state.set_toast(
                                        format!("{} {}", pi_name(pi, lang), l(lang, "aplicado", "applied")),
                                        ToastKind::Success,
                                    );
                                    commit_tactic(this, cx);
                                }))
                                .into_any_element()
                        }),
                );
            }
            ed = ed.child(box_);
        }
    }

    // All player instructions.
    let n_active = s.pis.len();
    let open = app.analyse_ui.show_all_pis;
    ed = ed.child(
        div()
            .id("an-all-pis-toggle")
            .cursor_pointer()
            .text_xs()
            .font_semibold()
            .text_color(theme.foreground)
            .on_click(cx.listener(|this, _, _, cx| {
                this.analyse_ui.show_all_pis = !this.analyse_ui.show_all_pis;
                cx.notify();
            }))
            .child(format!(
                "{} {}",
                if open { "▾" } else { "▸" },
                match lang {
                    Lang::Pt => format!(
                        "Todas as instruções individuais ({n_active} activas) — cada uma soma complexidade"
                    ),
                    Lang::En => format!(
                        "All player instructions ({n_active} active) — each one adds complexity"
                    ),
                }
            )),
    );
    if open {
        let mut list = div().v_flex().gap_1();
        for (j, def) in PLAYER_INSTRUCTIONS.iter().enumerate() {
            let on = s.pis.iter().any(|x| x == def.name);
            let name: &'static str = def.name;
            list = list.child(
                div()
                    .h_flex()
                    .items_start()
                    .gap_2()
                    .child(Checkbox::new(("an-pi", j)).checked(on).on_click(cx.listener(
                        move |this, _: &bool, _, cx| {
                            let Some(slot) = this.state.tactic.slots.get_mut(i) else { return };
                            if let Some(k) = slot.pis.iter().position(|x| x == name) {
                                slot.pis.remove(k);
                            } else {
                                slot.pis.push(name.to_string());
                            }
                            commit_tactic(this, cx);
                        },
                    )))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(format!("{} — {}", pi_name(def.name, lang), tr(lang, def.tip_pt))),
                    ),
            );
        }
        ed = ed.child(list);
    }
    ed
}

fn render_synergy(
    app: &ArchitectApp,
    t: &Tactic,
    rep: &ta_core::types::SynergyReport,
    no_squad: bool,
    theme: &Theme,
    cx: &mut Context<ArchitectApp>,
) -> Div {
    let lang = app.state.lang;
    let summary = match lang {
        Lang::Pt => format!(
            "Sinergia entre papéis em todas as fases: se os papéis se servem e se cobrem uns aos outros — não só se cada jogador está no melhor papel. {} chegam à área · {} criativo(s) · {} ficam atrás · {} saída(s).",
            rep.box_n, rep.creators, rep.rd_n, rep.out_n
        ),
        Lang::En => format!(
            "Role synergy across every phase: whether roles serve and cover each other — not just whether each player is in his best role. {} arrive in the box · {} creator(s) · {} stay back · {} outlet(s).",
            rep.box_n, rep.creators, rep.rd_n, rep.out_n
        ),
    };

    let mut p = panel(theme)
        .child(heading(
            l(lang, "Sinergia entre papéis — todas as fases", "Role synergy — every phase"),
            theme,
        ))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_3()
                .child(score_ring(rep.score, 72.0, cx))
                .child(
                    div()
                        .v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_1()
                        .child(div().text_xs().text_color(theme.foreground).child(summary))
                        .when(no_squad, |d| {
                            d.child(muted(
                                l(
                                    lang,
                                    "Sem plantel: avaliação só pela estrutura dos papéis.",
                                    "No squad: assessed on role structure only.",
                                ),
                                theme,
                            ))
                        }),
                ),
        );

    // Phase cards
    let mut cards = div().h_flex().flex_wrap().gap_3();
    for ph in SynergyPhase::ALL {
        let Some(ps) = rep.phases.get(&ph) else { continue };
        let mut card = div()
            .v_flex()
            .flex_1()
            .min_w(px(200.0))
            .gap_1()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(
                div()
                    .h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(synergy_phase_name(ph, lang)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_bold()
                            .text_color(score_color(ps.score))
                            .child(ps.score.to_string()),
                    ),
            );
        for (k, v) in &ps.parts {
            card = card.child(suitability_bar(synergy_part_name(k, lang), *v, 100, cx));
        }
        cards = cards.child(card);
    }
    p = p.child(cards).child(dual_synergy_maps(t, rep, lang, cx));

    // Main links
    if !rep.links.is_empty() {
        let open = app.analyse_ui.show_links;
        p = p.child(
            div()
                .id("an-syn-links-toggle")
                .cursor_pointer()
                .text_xs()
                .font_semibold()
                .text_color(theme.foreground)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.analyse_ui.show_links = !this.analyse_ui.show_links;
                    cx.notify();
                }))
                .child(format!(
                    "{} {} ({})",
                    if open { "▾" } else { "▸" },
                    l(lang, "Ligações principais", "Main links"),
                    rep.links.len()
                )),
        );
        if open {
            let mut list = div().v_flex().gap_1();
            for lk in &rep.links {
                list = list.child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(badge_pill(synergy_phase_name(lk.phase, lang), BadgeTone::Muted, cx))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_xs()
                                .text_color(theme.foreground)
                                .child(lk.why.clone()),
                        )
                        .child(muted(format!("({})", (lk.value * 100.0).round() as i32), theme)),
                );
            }
            p = p.child(list);
        }
    }

    // Issues by phase
    let by_phase: Vec<(SynergyPhase, Vec<&ta_core::types::SynergyIssue>)> = SynergyPhase::ALL
        .iter()
        .map(|&ph| (ph, rep.issues.iter().filter(|i| i.phase == ph).collect::<Vec<_>>()))
        .filter(|(_, v)| !v.is_empty())
        .collect();
    if by_phase.is_empty() {
        p = p.child(
            div()
                .text_xs()
                .text_color(theme.foreground)
                .child(l(
                    lang,
                    "Os papéis encaixam em todas as fases.",
                    "The roles fit together in every phase.",
                )),
        );
    } else {
        p = p.child(heading(
            l(lang, "O que está a falhar entre papéis", "What is not working between roles"),
            theme,
        ));
        for (ph, list) in by_phase {
            p = p.child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .child(synergy_phase_name(ph, lang).to_uppercase()),
            );
            for issue in list {
                p = p.child(
                    div()
                        .h_flex()
                        .items_start()
                        .gap_3()
                        .child(sev_icon(Severity::Warn))
                        .child(
                            div()
                                .v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(theme.foreground)
                                        .child(issue.text.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.primary)
                                        .child(format!("→ {}", issue.fix)),
                                ),
                        ),
                );
            }
        }
    }
    p
}

fn render_ti_editor(
    app: &ArchitectApp,
    t: &Tactic,
    entity: &Entity<ArchitectApp>,
    theme: &Theme,
    cx: &mut Context<ArchitectApp>,
) -> Div {
    let lang = app.state.lang;
    let ois = t.ois;

    let header = div()
        .h_flex()
        .flex_wrap()
        .w_full()
        .items_center()
        .justify_between()
        .gap_3()
        .child(heading(l(lang, "Instruções de equipa", "Team instructions"), theme))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(muted(
                    l(lang, "Instruções de adversário activas:", "Active opposition instructions:"),
                    theme,
                ))
                .child(
                    Button::new("an-ois-dec")
                        .outline()
                        .xsmall()
                        .label("−")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.state.tactic.ois > 0 {
                                this.state.tactic.ois -= 1;
                                commit_tactic(this, cx);
                            }
                        })),
                )
                .child(
                    div()
                        .w(px(28.0))
                        .text_center()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(ois.to_string()),
                )
                .child(
                    Button::new("an-ois-inc")
                        .outline()
                        .xsmall()
                        .label("+")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.state.tactic.ois < 20 {
                                this.state.tactic.ois += 1;
                                commit_tactic(this, cx);
                            }
                        })),
                )
                .child(
                    Button::new("an-ti-reset")
                        .outline()
                        .xsmall()
                        .label(l(lang, "Repor default", "Reset to default"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.tactic.ti.clear();
                            let lang = this.state.lang;
                            this.state.set_toast(
                                l(lang, "Instruções repostas", "Instructions reset"),
                                ToastKind::Info,
                            );
                            commit_tactic(this, cx);
                        })),
                ),
        );

    let mut p = panel(theme).child(header).child(muted(
        l(
            lang,
            "Nomes do FM26. A borda verde marca instruções fora do default (cada uma soma complexidade no FM27).",
            "FM26 names. The green border marks instructions changed from the default (each one adds complexity in FM27).",
        ),
        theme,
    ));

    let good: Hsla = rgb(GOOD).into();
    let mut ix_global = 0usize;
    for sec in FM_SECTIONS.iter() {
        p = p.child(
            div()
                .pt_2()
                .text_sm()
                .font_semibold()
                .text_color(theme.foreground)
                .child(tr(lang, sec.title)),
        );
        let mut grid = div().h_flex().flex_wrap().gap_3();
        for &key in sec.keys {
            let Some(d) = ti_by_key(key) else { continue };
            let k: &'static str = d.key;
            let v = ti_val(t, k);
            let changed = v != d.def;
            let default_tag = l(lang, " (padrão)", " (default)");
            let items: Vec<(SharedString, bool)> = d
                .opts
                .iter()
                .enumerate()
                .map(|(oi, o)| {
                    let mut s = ti_opt_name(o, lang);
                    if oi == d.def {
                        s.push_str(default_tag);
                    }
                    (SharedString::from(s), oi == v)
                })
                .collect();
            let current = d.opts.get(v).map(|o| ti_opt_name(o, lang)).unwrap_or_default();
            let open = app.analyse_ui.ti_open.contains(k);
            let need_txt = d.need.map(|n| {
                let mut seen: Vec<Attr> = Vec::new();
                for &a in n.low.iter().chain(n.high.iter()) {
                    if !seen.contains(&a) {
                        seen.push(a);
                    }
                }
                seen.iter().map(|&a| attr_name(a, lang)).collect::<Vec<_>>().join(", ")
            });

            let card = div()
                .v_flex()
                .w(px(268.0))
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(if changed { good } else { theme.border })
                .bg(if changed { with_alpha(good, 0.08) } else { theme.background })
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_1()
                                .min_w_0()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(theme.foreground)
                                        .truncate()
                                        .child(ti_name(d, lang)),
                                )
                                .when(d.fm27, |el| el.child(badge_pill("FM27", BadgeTone::Info, cx))),
                        )
                        .child(muted(tr(lang, d.area_pt), theme)),
                )
                .child(select_menu(("an-ti", ix_global), current, items, entity, move |oi, this, cx| {
                    set_ti(this, k, oi, cx);
                }))
                .child(
                    div()
                        .id(("an-ti-info", ix_global))
                        .cursor_pointer()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !this.analyse_ui.ti_open.remove(k) {
                                this.analyse_ui.ti_open.insert(k);
                            }
                            cx.notify();
                        }))
                        .child(format!("{} {}", if open { "▾" } else { "▸" }, tr(lang, d.desc_pt))),
                )
                .when(open, |el| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(format!("{} {}", l(lang, "Prós:", "Pros:"), tr(lang, d.pro_pt))),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.foreground)
                            .child(format!("{} {}", l(lang, "Contras:", "Cons:"), tr(lang, d.con_pt))),
                    )
                    .when_some(need_txt.clone(), |el, need| {
                        el.child(muted(
                            format!("{} {}", l(lang, "Atributos relevantes:", "Relevant attributes:"), need),
                            theme,
                        ))
                    })
                });
            grid = grid.child(card);
            ix_global += 1;
        }
        p = p.child(grid);
    }
    p
}
