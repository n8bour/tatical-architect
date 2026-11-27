//! Training screen: FM27 Tactical Understanding priorities, suggested 5-day
//! week, XI individual Role Training + weak attributes, and mentoring groups.

use crate::state::{NavSection, TeamTab, ToastKind};
use crate::views::team::sync_animated_pitch;
use crate::widgets::common::{
    attr_chip, badge_pill, collapsible_card, score_color, with_alpha, BadgeTone,
};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::HashSet;
use ta_core::data::{attr_name, l, pt_terms, role_by_id, role_name, t_name, training_area};
use ta_core::engine::audit::audit_tactic;
use ta_core::engine::hungarian::auto_pick_xi;
use ta_core::engine::scoring::training_plan;
use ta_core::types::Lang;

fn area_display(key: &str, lang: Lang) -> String {
    let raw = training_area(key).map(|a| a.name).unwrap_or(key);
    match lang {
        Lang::Pt => pt_terms(raw),
        Lang::En => raw.to_string(),
    }
}

fn fit_pill(score: u8) -> Div {
    let col = score_color(score);
    div()
        .px_2()
        .py(px(1.0))
        .rounded_sm()
        .bg(col)
        .text_xs()
        .font_bold()
        .text_color(rgb(0xffffff))
        .child(score.to_string())
}

/// Renders the Training view wired directly to `audit_tactic` & `training_plan`.
pub fn render_training(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let tactic = &app.state.tactic;
    let squad = &app.state.squad;

    let audit = audit_tactic(tactic, squad, lang);
    let plan = training_plan(tactic, squad, &audit, lang);

    // ------------------------------------------------------------------------
    // Header
    // ------------------------------------------------------------------------
    let header = div()
        .v_flex()
        .gap_1()
        .child(
            div()
                .text_xl()
                .font_bold()
                .text_color(theme.foreground)
                .child(format!(
                    "{} “{}”",
                    l(lang, "Plano de treino para", "Training plan for"),
                    t_name(tactic, lang)
                )),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(l(
                    lang,
                    "Ligado às áreas de Compreensão Tática do FM27: treine primeiro o que a sua tática mais exige.",
                    "Linked to FM27 Tactical Understanding areas: train first what your tactic demands most.",
                )),
        );

    // ------------------------------------------------------------------------
    // Left panel: Tactical Understanding priorities
    // ------------------------------------------------------------------------
    let mut prio_body = div().v_flex().gap_2();
    for (idx, area) in plan.areas.iter().enumerate() {
        let mut seen = HashSet::new();
        let reasons: Vec<&str> = area
            .why
            .iter()
            .filter(|w| seen.insert(w.as_str()))
            .map(String::as_str)
            .collect();
        let tone = if idx < 2 {
            BadgeTone::Warn
        } else {
            BadgeTone::Info
        };
        prio_body = prio_body.child(
            div()
                .h_flex()
                .items_start()
                .gap_2()
                .p_2()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(badge_pill(format!("{}", idx + 1), tone, cx))
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
                                .child(area_display(area.key, lang)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(reasons.join(" · ")),
                        ),
                ),
        );
    }

    if plan.cohesion {
        prio_body = prio_body.child(
            div()
                .p_2()
                .rounded_sm()
                .border_1()
                .border_color(rgb(0xd97706))
                .bg(with_alpha(rgb(0xd97706).into(), 0.14))
                .text_xs()
                .text_color(theme.foreground)
                .child(l(
                    lang,
                    "Complexidade acima do que o XI absorve: inclua Coesão da Equipa e reduza instruções.",
                    "Complexity exceeds what the XI can absorb: include Team Cohesion and reduce instructions.",
                )),
        );
    }

    let prio_card = collapsible_card(
        "training-card-prio",
        l(
            lang,
            "Prioridades de Compreensão Tática",
            "Tactical Understanding priorities",
        ),
        None,
        app.state.is_panel_collapsed("training-prio"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("training-prio");
            cx.notify();
        }),
        prio_body,
        cx,
    );

    // ------------------------------------------------------------------------
    // Right panel: Suggested 5-day typical week
    // ------------------------------------------------------------------------
    let a0 = plan.areas.first().map(|a| a.key).unwrap_or("ip_shape");
    let a1 = plan.areas.get(1).map(|a| a.key).unwrap_or("oop_shape");
    let a2 = plan.areas.get(2).map(|a| a.key).unwrap_or("buildup");

    let week_rows = [
        (
            l(lang, "Dia 1 (pós-jogo)", "Day 1 (post-match)"),
            l(lang, "Recuperação", "Recovery").to_string(),
        ),
        (l(lang, "Dia 2", "Day 2"), area_display(a0, lang)),
        (
            l(lang, "Dia 3", "Day 3"),
            format!(
                "{} + {}",
                area_display(a1, lang),
                l(lang, "Individual", "Individual")
            ),
        ),
        (l(lang, "Dia 4", "Day 4"), area_display(a2, lang)),
        (
            l(lang, "Dia 5 (véspera)", "Day 5 (match eve)"),
            l(
                lang,
                "Preparação do Jogo (próximo adversário) + bolas paradas",
                "Match Preparation (next opponent) + set pieces",
            )
            .to_string(),
        ),
    ];

    let mut week_body = div().v_flex().gap_1();
    for (day, session) in week_rows {
        week_body = week_body.child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_3()
                .px_3()
                .py_2()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(
                    div()
                        .w(px(140.0))
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(day),
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(session),
                ),
        );
    }
    week_body = week_body.child(
        div()
            .pt_2()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(l(
                lang,
                "Semanas com dois jogos: só Recuperação + Preparação do Jogo.",
                "Two-match weeks: Recovery + Match Preparation only.",
            )),
    );

    let week_card = collapsible_card(
        "training-card-week",
        l(lang, "Semana-tipo sugerida", "Suggested typical week"),
        None,
        app.state.is_panel_collapsed("training-week"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("training-week");
            cx.notify();
        }),
        week_body,
        cx,
    );

    let top_row = div()
        .h_flex()
        .flex_wrap()
        .items_start()
        .gap_4()
        .child(div().flex_1().min_w(px(300.0)).child(prio_card))
        .child(div().flex_1().min_w(px(300.0)).child(week_card));

    // ------------------------------------------------------------------------
    // Bottom panel: XI individual Role Training + weak attributes + mentoring
    // ------------------------------------------------------------------------
    let mut indiv_body = div().v_flex().gap_2();
    if plan.indiv.is_empty() {
        indiv_body = indiv_body
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(l(
                        lang,
                        "Atribua jogadores ao XI (Plantel → Analisador → Auto XI) para recomendações individuais.",
                        "Assign players to the XI (Squad → Analyser → Auto XI) for individual recommendations.",
                    )),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .pt_1()
                    .when(squad.len() >= 11, |d| {
                        d.child(
                            Button::new("training-auto-xi")
                                .primary()
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
                    })
                    .when(squad.len() < 11, |d| {
                        d.child(
                            Button::new("training-go-squad")
                                .outline()
                                .small()
                                .label(l(lang, "Ir para Plantel", "Go to Squad"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.section = NavSection::Team;
                                    this.team_tab = TeamTab::Squad;
                                    cx.notify();
                                })),
                        )
                    }),
            );
    } else {
        // Table header
        indiv_body = indiv_body.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .border_b_1()
                .border_color(theme.border)
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .child(div().w(px(170.0)).child(l(lang, "Jogador", "Player")))
                .child(div().w(px(48.0)).child(l(lang, "Pos", "Pos")))
                .child(div().w(px(46.0)).child("IP"))
                .child(div().w(px(46.0)).child("OOP"))
                .child(
                    div()
                        .w(px(190.0))
                        .child(l(lang, "Treino de Papel", "Role Training")),
                )
                .child(
                    div()
                        .flex_1()
                        .child(l(lang, "Foco adicional", "Additional focus")),
                ),
        );

        for x in &plan.indiv {
            let role_lbl = role_by_id(x.role_id)
                .map(|r| role_name(r, lang))
                .unwrap_or_else(|| x.role_id.to_string());
            let mut weak_row = div().h_flex().flex_wrap().flex_1().gap_1();
            if x.weak.is_empty() {
                weak_row = weak_row.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("—"),
                );
            } else {
                for &(attr, val) in &x.weak {
                    weak_row = weak_row.child(attr_chip(attr_name(attr, lang), val as f32, cx));
                }
            }

            indiv_body = indiv_body.child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded_sm()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .w(px(170.0))
                            .text_xs()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .truncate()
                            .child(x.player_name.clone()),
                    )
                    .child(
                        div()
                            .w(px(48.0))
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(x.slot.clone()),
                    )
                    .child(div().w(px(46.0)).child(fit_pill(x.si)))
                    .child(div().w(px(46.0)).child(fit_pill(x.so)))
                    .child(
                        div()
                            .w(px(190.0))
                            .text_xs()
                            .text_color(theme.foreground)
                            .truncate()
                            .child(role_lbl),
                    )
                    .child(weak_row),
            );
        }
    }

    if !plan.mentors.is_empty() && !plan.youth.is_empty() {
        let mentors_str: Vec<&str> = plan.mentors.iter().take(4).map(String::as_str).collect();
        let youth_str: Vec<&str> = plan.youth.iter().take(8).map(String::as_str).collect();
        indiv_body = indiv_body.child(
            div()
                .v_flex()
                .gap_1()
                .mt_3()
                .pt_3()
                .border_t_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(l(lang, "Mentoria", "Mentoring")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "{}: {}. {}: {}.",
                            l(
                                lang,
                                "Mentores (Determinação ≥ 15, ≥ 26 anos)",
                                "Mentors (Determination ≥ 15, aged 26+)"
                            ),
                            mentors_str.join(", "),
                            l(lang, "Jovens ≤ 21", "Youngsters ≤ 21"),
                            youth_str.join(", ")
                        )),
                ),
        );
    }

    let indiv_card = collapsible_card(
        "training-card-indiv",
        l(
            lang,
            "Desenvolvimento individual do XI",
            "Individual XI development",
        ),
        None,
        app.state.is_panel_collapsed("training-indiv"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("training-indiv");
            cx.notify();
        }),
        indiv_body,
        cx,
    );

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .child(top_row)
        .child(indiv_card)
}
