//! Tactic › Game States (Match Situations) view wired to `ta_core::planning::gameplans`.

use crate::state::{NavSection, TacticTab, ToastKind};
use crate::widgets::common::{badge_pill, BadgeTone};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use ta_core::data::t_name;
use ta_core::planning::{game_plan, scenario_meta};
use ta_core::types::{GamePlan, GamePlanChangeKind, GameStateKind, Lang, Phase, SynergyPhase};

/// View-local UI state for Tactic › Game States.
#[derive(Debug, Clone, Default)]
pub struct GamePlansViewState {
    pub selected: GameStateKind,
}

fn delta_badge(before: u8, after: u8, cx: &App) -> impl IntoElement {
    let diff = after as i16 - before as i16;
    let col: Hsla = if diff > 0 {
        rgb(0x22c55e).into()
    } else if diff < 0 {
        rgb(0xef4444).into()
    } else {
        cx.theme().muted_foreground
    };
    let txt = if diff != 0 {
        format!("{} → {} ({:+})", before, after, diff)
    } else {
        format!("{} → {}", before, after)
    };
    div().text_xs().font_semibold().text_color(col).child(txt)
}

fn format_plan_clipboard(plan: &GamePlan, lang: Lang) -> String {
    let mut out = Vec::new();
    out.push(format!("{} — {}", plan.title, plan.desc));
    out.push(String::new());
    for c in &plan.changes {
        let prefix = match c.kind {
            GamePlanChangeKind::Ment => lang.pick("Mentalidade", "Mentality"),
            GamePlanChangeKind::Ti => "TI",
            GamePlanChangeKind::Role => lang.pick("Papel", "Role"),
            GamePlanChangeKind::Pi => "PI",
        };
        let slot = c.code.as_deref().unwrap_or("");
        if c.removed {
            out.push(format!("[{}] {} {}: -{} ({})", prefix, slot, c.label, c.to, c.why));
        } else {
            out.push(format!(
                "[{}] {} {}: {} -> {} ({})",
                prefix, slot, c.label, c.from, c.to, c.why
            ));
        }
    }
    if !plan.subs.is_empty() {
        out.push(String::new());
        out.push(lang.pick("Substituições:", "Substitutions:").to_string());
        for s in &plan.subs {
            out.push(format!("  {} · {} -> {} ({})", s.code, s.out_name, s.in_name, s.why));
        }
    }
    if !plan.ois.is_empty() {
        out.push(String::new());
        out.push(
            lang.pick("Instruções de adversário:", "Opposition instructions:")
                .to_string(),
        );
        for o in &plan.ois {
            out.push(format!("  {} ({}): {}", o.player, o.code, o.ois.join(", ")));
        }
    }
    out.join("\n")
}

/// Renders Tactic › Game States (5 scenarios, diffs, impact delta, subs, OIs, Save/Use/Copy).
pub fn render_gameplans(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let kind = app.gameplans_ui.selected;
    let plan = game_plan(
        &app.state.tactic,
        &app.state.squad,
        kind,
        &app.state.opponent,
        lang,
    );

    let ment_change = plan
        .changes
        .iter()
        .find(|c| c.kind == GamePlanChangeKind::Ment);
    let ti_changes: Vec<_> = plan
        .changes
        .iter()
        .filter(|c| c.kind == GamePlanChangeKind::Ti)
        .collect();
    let role_changes: Vec<_> = plan
        .changes
        .iter()
        .filter(|c| c.kind == GamePlanChangeKind::Role)
        .collect();
    let pi_changes: Vec<_> = plan
        .changes
        .iter()
        .filter(|c| c.kind == GamePlanChangeKind::Pi)
        .collect();

    let tac_save = plan.tactic.clone();
    let tac_use = plan.tactic.clone();
    let copy_txt = format_plan_clipboard(&plan, lang);

    // Header bar
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
                        .text_base()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("Situações de jogo", "Game states")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "{} “{}”",
                            lang.pick("Planos B/C a partir de", "Plans B/C built from"),
                            t_name(&app.state.tactic, lang)
                        )),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("gp-save")
                        .primary()
                        .small()
                        .label(lang.pick("Guardar como tática", "Save as tactic"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.state.saved_tactics.push(tac_save.clone());
                            this.state.persist();
                            this.state.set_toast(
                                this.state
                                    .lang
                                    .pick("Plano guardado nas táticas.", "Plan saved to tactics."),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("gp-use")
                        .outline()
                        .small()
                        .label(lang.pick("Usar agora", "Use now"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.apply_active_tactic(tac_use.clone(), cx);
                            this.section = NavSection::Tactic;
                            this.tactic_tab = TacticTab::Analyse;
                            this.state.set_toast(
                                this.state
                                    .lang
                                    .pick("Plano aplicado como tática ativa.", "Plan applied as active tactic."),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("gp-copy")
                        .outline()
                        .small()
                        .label(lang.pick("Copiar como texto", "Copy as text"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy_txt.clone()));
                            this.state.set_toast(
                                this.state.lang.pick("Plano copiado.", "Plan copied."),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                ),
        );

    // 5 scenario tabs
    let mut scenario_bar = div().h_flex().flex_wrap().gap_2();
    for sc in GameStateKind::ALL {
        let (title, _) = scenario_meta(sc, lang);
        let active = sc == kind;
        scenario_bar = scenario_bar.child(
            Button::new(("gp-sc", sc as usize))
                .small()
                .label(format!("{} {}", sc.icon(), title))
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.gameplans_ui.selected = sc;
                    cx.notify();
                })),
        );
    }

    // Left panel: diffs
    let mut left = div()
        .v_flex()
        .flex_1()
        .min_w_0()
        .gap_3()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(plan.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(plan.desc.clone()),
                ),
        );

    // Mentality diff
    left = left.child(
        div()
            .v_flex()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(lang.pick("Mentalidade", "Mentality")),
            )
            .child(match ment_change {
                Some(m) => div()
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(format!("{} → {} — {}", m.from, m.to, m.why)),
                None => div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(lang.pick("Mantém-se.", "Unchanged.")),
            }),
    );

    // Team instructions diff
    let mut ti_sec = div()
        .v_flex()
        .gap_1()
        .pt_2()
        .border_t_1()
        .border_color(theme.border)
        .child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(theme.foreground)
                .child(lang.pick("Instruções de equipa", "Team instructions")),
        );
    if ti_changes.is_empty() {
        ti_sec = ti_sec.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(lang.pick("Sem mudanças.", "No changes.")),
        );
    } else {
        for c in &ti_changes {
            ti_sec = ti_sec.child(
                div()
                    .h_flex()
                    .items_start()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .w(px(160.0))
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(c.label.clone()),
                    )
                    .child(
                        div()
                            .w(px(170.0))
                            .text_color(theme.primary)
                            .child(format!("{} → {}", c.from, c.to)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .child(c.why.clone()),
                    ),
            );
        }
    }
    left = left.child(ti_sec);

    // Role swaps diff
    let mut role_sec = div()
        .v_flex()
        .gap_1()
        .pt_2()
        .border_t_1()
        .border_color(theme.border)
        .child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(theme.foreground)
                .child(lang.pick("Papéis", "Roles")),
        );
    if role_changes.is_empty() {
        role_sec = role_sec.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(lang.pick(
                    "Os papéis atuais já servem este cenário (nenhuma troca melhorava a sinergia).",
                    "Current roles already fit this scenario (no swap improved synergy).",
                )),
        );
    } else {
        for c in &role_changes {
            let phase_lbl = match c.phase {
                Some(Phase::Oop) => "OOP",
                _ => "IP",
            };
            role_sec = role_sec.child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .child(badge_pill(
                        format!("{} {}", c.code.as_deref().unwrap_or(""), phase_lbl),
                        if phase_lbl == "IP" {
                            BadgeTone::Good
                        } else {
                            BadgeTone::Info
                        },
                        cx,
                    ))
                    .child(
                        div()
                            .w(px(120.0))
                            .text_color(theme.foreground)
                            .truncate()
                            .child(c.player.clone().unwrap_or_default()),
                    )
                    .child(
                        div()
                            .w(px(180.0))
                            .text_color(theme.primary)
                            .child(format!("{} → {}", c.from, c.to)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .child(c.why.clone()),
                    ),
            );
        }
    }
    left = left.child(role_sec);

    // PI diff
    if !pi_changes.is_empty() {
        let mut pi_sec = div()
            .v_flex()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(lang.pick("Instruções individuais", "Player instructions")),
            );
        for c in &pi_changes {
            pi_sec = pi_sec.child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .w(px(150.0))
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(format!(
                                "{} {}",
                                c.code.as_deref().unwrap_or(""),
                                c.player.as_deref().unwrap_or("")
                            )),
                    )
                    .child(badge_pill(
                        if c.removed {
                            format!("− {}", c.to)
                        } else {
                            format!("+ {}", c.to)
                        },
                        if c.removed {
                            BadgeTone::Crit
                        } else {
                            BadgeTone::Good
                        },
                        cx,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .child(c.why.clone()),
                    ),
            );
        }
        left = left.child(pi_sec);
    }

    // Tactical tweaks
    if !plan.tweaks.is_empty() {
        let mut tw = div()
            .v_flex()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child("Tactical Tweaks"),
            );
        for (name, why) in &plan.tweaks {
            tw = tw.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(format!("• {} — {}", name, why)),
            );
        }
        left = left.child(tw);
    }

    // Suggested subs
    if !plan.subs.is_empty() {
        let mut subs_sec = div()
            .v_flex()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(lang.pick("Substituições sugeridas", "Suggested substitutions")),
            );
        for s in &plan.subs {
            subs_sec = subs_sec.child(
                div()
                    .h_flex()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .w(px(55.0))
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(s.code.clone()),
                    )
                    .child(
                        div()
                            .w(px(210.0))
                            .text_color(theme.primary)
                            .child(format!("{} → {}", s.out_name, s.in_name)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .child(s.why.clone()),
                    ),
            );
        }
        left = left.child(subs_sec);
    }

    // OIs
    if !plan.ois.is_empty() {
        let mut oi_sec = div()
            .v_flex()
            .gap_1()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(lang.pick("Instruções de adversário", "Opposition instructions")),
            );
        for o in &plan.ois {
            oi_sec = oi_sec.child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .child(
                        div()
                            .w(px(170.0))
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(format!("{} ({})", o.player, o.code)),
                    )
                    .child(
                        div()
                            .text_color(theme.muted_foreground)
                            .child(o.ois.join(" · ")),
                    ),
            );
        }
        left = left.child(oi_sec);
    }

    // Notes
    for note in &plan.notes {
        left = left.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("💡 {}", note)),
        );
    }

    // Right column: Before/After Impact Delta + FM27 usage note
    let mut impact_rows = div()
        .v_flex()
        .gap_2()
        .child(
            div()
                .h_flex()
                .justify_between()
                .text_xs()
                .child(
                    div()
                        .text_color(theme.foreground)
                        .child(lang.pick("Compromisso ofensivo", "Attacking commitment")),
                )
                .child(delta_badge(plan.intent_before.att, plan.intent_after.att, cx)),
        )
        .child(
            div()
                .h_flex()
                .justify_between()
                .text_xs()
                .child(
                    div()
                        .text_color(theme.foreground)
                        .child(lang.pick("Segurança defensiva", "Defensive security")),
                )
                .child(delta_badge(plan.intent_before.sec, plan.intent_after.sec, cx)),
        )
        .child(
            div()
                .pt_2()
                .border_t_1()
                .border_color(theme.border)
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(lang.pick("Sinergia (coerência)", "Synergy (coherence)")),
        );

    for ph in SynergyPhase::ALL {
        let lbl = match ph {
            SynergyPhase::Build => lang.pick("Construção", "Build-up"),
            SynergyPhase::Att => lang.pick("Ataque", "Attack"),
            SynergyPhase::Def => lang.pick("Defesa", "Defense"),
            SynergyPhase::Trans => lang.pick("Transição", "Transition"),
        };
        let b = plan.before.phases.get(&ph).map(|p| p.score).unwrap_or(0);
        let a = plan.after.phases.get(&ph).map(|p| p.score).unwrap_or(0);
        impact_rows = impact_rows.child(
            div()
                .h_flex()
                .justify_between()
                .text_xs()
                .child(div().text_color(theme.muted_foreground).child(lbl))
                .child(delta_badge(b, a, cx)),
        );
    }
    impact_rows = impact_rows.child(
        div()
            .h_flex()
            .justify_between()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .text_xs()
            .child(
                div()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(lang.pick("Diagnóstico", "Diagnosis")),
            )
            .child(delta_badge(plan.audit_before.overall, plan.audit_after.overall, cx)),
    );

    let right = div()
        .v_flex()
        .w(px(300.0))
        .flex_shrink_0()
        .gap_3()
        .child(
            div()
                .v_flex()
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
                        .child(lang.pick("Impacto (Antes → Depois)", "Impact (Before → After)")),
                )
                .child(impact_rows),
        )
        .child(
            div()
                .v_flex()
                .gap_1()
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
                        .child(lang.pick("No jogo (FM27)", "In game (FM27)")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(lang.pick(
                            "Guarde o plano como tática B ou C (FM27 permite 3 táticas) ou use os Tactical Tweaks para ajustes rápidos durante o jogo.",
                            "Save this plan as tactic B or C (FM27 allows 3 tactics) or use the listed Tactical Tweaks for quick mid-match shifts.",
                        )),
                ),
        );

    div()
        .v_flex()
        .w_full()
        .gap_3()
        .child(header)
        .child(scenario_bar)
        .when(app.state.squad.is_empty(), |d| {
            d.child(
                div()
                    .p_2()
                    .rounded_sm()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.secondary)
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(lang.pick(
                        "Sem plantel carregado: os ajustes seguem apenas a estrutura da tática (substituições e PIs requerem plantel).",
                        "No squad loaded: adjustments follow tactic structure only (substitutions and PIs require a squad).",
                    )),
            )
        })
        .child(div().h_flex().w_full().items_start().gap_4().child(left).child(right))
}
