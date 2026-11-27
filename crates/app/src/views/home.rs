//! Home screen: 3-step guided workflow (Squad → Tactic → Opponent), top audit
//! findings to fix first, best system suggestion, opponent summary, and saved tactics.

use crate::state::{NavSection, TacticTab, TeamTab, ToastKind};
use crate::views::team::{
    apply_tactic_to_app, import_squad_from_picker, load_sample_squad, spawn_next_opponent_from_fm,
};
use crate::widgets::common::{
    badge_pill, collapsible_card, page_title, pal, panel, score_ring, BadgeTone,
};
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use ta_core::data::{
    l, mentality_name, t, t_name, template_by_id, ti_by_key, ti_name, ti_opt_name,
};
use ta_core::engine::audit::audit_tactic;
use ta_core::engine::insights::opponent_report;
use ta_core::engine::scoring::suggest_tactics;
use ta_core::types::{Lang, Severity};

fn step_card(
    step_num: usize,
    done: bool,
    title: &'static str,
    body: impl IntoElement,
    actions: impl IntoElement,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let badge_bg: Hsla = if done {
        rgb(pal::GOOD).into()
    } else {
        theme.muted
    };
    let badge_fg: Hsla = if done {
        rgb(0xffffff).into()
    } else {
        theme.muted_foreground
    };

    panel(cx)
        .flex_1()
        .min_w(px(260.0))
        .gap_3()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(26.0))
                        .h(px(26.0))
                        .rounded_full()
                        .bg(badge_bg)
                        .text_color(badge_fg)
                        .text_xs()
                        .font_bold()
                        .h_flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(if done {
                            "✓".to_string()
                        } else {
                            step_num.to_string()
                        }),
                )
                .child(
                    div()
                        .min_w_0()
                        .text_base()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(title),
                ),
        )
        .child(div().flex_1().min_w_0().child(body))
        .child(div().h_flex().flex_wrap().gap_2().child(actions))
}

fn severity_badge(sev: Severity, cx: &App) -> Div {
    let (label, tone) = match sev {
        Severity::Crit => ("!", BadgeTone::Crit),
        Severity::Warn => ("▲", BadgeTone::Warn),
        Severity::Info => ("i", BadgeTone::Info),
        Severity::Good => ("✓", BadgeTone::Good),
    };
    badge_pill(label, tone, cx)
}

/// Renders the live Home dashboard.
pub fn render_home(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let tactic = &app.state.tactic;
    let squad = &app.state.squad;
    let opp = &app.state.opponent;

    let audit = audit_tactic(tactic, squad, lang);
    let top_findings: Vec<_> = audit
        .findings
        .iter()
        .filter(|f| matches!(f.sev, Severity::Crit | Severity::Warn))
        .take(3)
        .cloned()
        .collect();

    let best_sugg = if squad.len() >= 11 {
        suggest_tactics(squad, lang).into_iter().next()
    } else {
        None
    };

    let opp_rep = if !opp.is_empty() {
        Some(opponent_report(tactic, squad, opp, lang))
    } else {
        None
    };

    // ------------------------------------------------------------------------
    // Header
    // ------------------------------------------------------------------------
    let header = div()
        .v_flex()
        .gap_1()
        .child(page_title(
            l(
                lang,
                "Olá! Vamos preparar o próximo jogo.",
                "Hello! Let's prepare for the next match.",
            ),
            cx,
        ))
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(l(
                    lang,
                    "Três passos. Tudo corre neste computador: o plantel vem do save e, com o FM aberto, o próximo adversário vem do jogo.",
                    "Three steps. Everything runs locally: your squad comes from your save and, with FM open, the next opponent comes from the game.",
                )),
        );

    // ------------------------------------------------------------------------
    // Step 1: Squad
    // ------------------------------------------------------------------------
    let squad_done = !squad.is_empty();
    let squad_body = if squad_done {
        let club_suffix = if app.state.my_club.is_empty() {
            String::new()
        } else {
            format!(" · {}", app.state.my_club)
        };
        let count_line = match lang {
            Lang::Pt => format!("{} jogadores{}", squad.len(), club_suffix),
            Lang::En => format!("{} players{}", squad.len(), club_suffix),
        };
        let src_line = if app.state.squad_source.is_empty() {
            l(lang, "Carregado localmente", "Loaded locally").to_string()
        } else {
            app.state.squad_source.clone()
        };
        div()
            .v_flex()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(count_line),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(src_line),
            )
    } else {
        let hint = if app.state.last_save_err.is_some() {
            l(
                lang,
                "Falta configurar o fmsave ou importe um CSV/HTML do plantel.",
                "Configure fmsave in Settings or import a squad CSV/HTML file.",
            )
        } else if app.state.auto_save_poll {
            l(
                lang,
                "Grave o jogo no FM27 ou clique em «Ler save agora».",
                "Save in FM27 or click \"Read save now\".",
            )
        } else {
            l(
                lang,
                "Clique em «Ler save agora» (fmsave), importe um CSV/HTML ou experimente o plantel de exemplo.",
                "Click \"Read save now\" (fmsave), import a CSV/HTML file, or try the sample squad.",
            )
        };
        div()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(hint)
    };

    let squad_actions = if squad_done {
        div()
            .h_flex()
            .flex_wrap()
            .gap_2()
            .child(
                Button::new("home-step1-xi")
                    .outline()
                    .small()
                    .label(l(lang, "Ver melhor XI", "View Best XI"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Team;
                        this.team_tab = TeamTab::BestXi;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("home-step1-squad")
                    .ghost()
                    .small()
                    .label(l(lang, "Abrir plantel", "Open squad"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Team;
                        this.team_tab = TeamTab::Squad;
                        cx.notify();
                    })),
            )
    } else {
        div()
            .h_flex()
            .flex_wrap()
            .gap_2()
            .child(
                Button::new("home-step1-read")
                    .primary()
                    .small()
                    .label(l(lang, "Ler save agora", "Read save now"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.spawn_read_save_now(cx);
                    })),
            )
            .child(
                Button::new("home-step1-import")
                    .outline()
                    .small()
                    .label(l(lang, "Importar CSV/HTML", "Import CSV/HTML"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        import_squad_from_picker(this, false, cx);
                    })),
            )
            .child(
                Button::new("home-step1-sample")
                    .ghost()
                    .small()
                    .label(l(lang, "Usar exemplo", "Use sample"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        load_sample_squad(this, cx);
                    })),
            )
    };

    // ------------------------------------------------------------------------
    // Step 2: Tactic
    // ------------------------------------------------------------------------
    let tactic_body = div()
        .h_flex()
        .items_center()
        .gap_3()
        .child(score_ring(audit.overall, 60.0, cx))
        .child(
            div()
                .v_flex()
                .gap(px(2.0))
                .min_w_0()
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .truncate()
                        .child(t_name(tactic, lang)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "{} → {} · {}",
                            tactic.ip_form,
                            tactic.oop_form,
                            mentality_name(tactic.ment, lang)
                        )),
                ),
        );

    let has_xi_squad = squad.len() >= 11;
    let tactic_actions = div()
        .h_flex()
        .flex_wrap()
        .gap_2()
        .when(has_xi_squad, |d| {
            d.child(
                Button::new("home-step2-gen")
                    .primary()
                    .small()
                    .label(l(lang, "Gerar tática", "Generate tactic"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Team;
                        this.team_tab = TeamTab::Generator;
                        cx.notify();
                    })),
            )
        })
        .child(
            Button::new("home-step2-analyse")
                .small()
                .when(!has_xi_squad, |b| b.primary())
                .when(has_xi_squad, |b| b.outline())
                .label(l(lang, "Analisar", "Analyse"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.section = NavSection::Tactic;
                    this.tactic_tab = TacticTab::Analyse;
                    cx.notify();
                })),
        )
        .child(
            Button::new("home-step2-sim")
                .outline()
                .small()
                .label(l(lang, "▶ Ver em campo", "▶ Animated pitch"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.section = NavSection::Tactic;
                    this.tactic_tab = TacticTab::AnimatedPitch;
                    cx.notify();
                })),
        );

    // ------------------------------------------------------------------------
    // Step 3: Opponent
    // ------------------------------------------------------------------------
    let opp_done = !opp.is_empty();
    let opp_body = if let Some(rep) = &opp_rep {
        let club_name = rep
            .club
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| app.state.opponent_source.clone());
        div()
            .v_flex()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(if club_name.is_empty() {
                        l(lang, "Adversário carregado", "Opponent loaded").to_string()
                    } else {
                        club_name
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(match lang {
                        Lang::Pt => format!("{} jogadores prontos para análise", opp.len()),
                        Lang::En => format!("{} players ready for scouting report", opp.len()),
                    }),
            )
    } else {
        div()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(l(
                lang,
                "Com o FM aberto, a app descobre o próximo jogo e lê o adversário; ou carregue um CSV/HTML.",
                "With FM open, fetch your next opponent automatically, or import their CSV/HTML squad.",
            ))
    };

    let opp_actions = div()
        .h_flex()
        .flex_wrap()
        .gap_2()
        .child(
            Button::new("home-step3-fm")
                .small()
                .when(!opp_done, |b| b.primary())
                .when(opp_done, |b| b.outline())
                .label(l(lang, "Próximo adversário do FM", "Next opponent from FM"))
                .on_click(cx.listener(|this, _, _, cx| {
                    spawn_next_opponent_from_fm(this, cx);
                })),
        )
        .child(
            Button::new("home-step3-view")
                .outline()
                .small()
                .label(if opp_done {
                    l(lang, "Ver relatório", "View report")
                } else {
                    l(lang, "Carregar adversário", "Load opponent")
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.section = NavSection::Team;
                    this.team_tab = TeamTab::Opponent;
                    cx.notify();
                })),
        );

    let steps_row = div()
        .h_flex()
        .flex_wrap()
        .items_stretch()
        .gap_4()
        .child(step_card(
            1,
            squad_done,
            l(lang, "Plantel", "Squad"),
            squad_body,
            squad_actions,
            cx,
        ))
        .child(step_card(
            2,
            true,
            l(lang, "Tática", "Tactic"),
            tactic_body,
            tactic_actions,
            cx,
        ))
        .child(step_card(
            3,
            opp_done,
            l(lang, "Adversário", "Opponent"),
            opp_body,
            opp_actions,
            cx,
        ));

    // ------------------------------------------------------------------------
    // Left column: What to fix first
    // ------------------------------------------------------------------------
    let mut fix_list = div().v_flex().gap_2();
    if top_findings.is_empty() {
        fix_list = fix_list.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(l(
                    lang,
                    "Nada grave na tática. 👍",
                    "Nothing critical in your tactic. 👍",
                )),
        );
    } else {
        for f in &top_findings {
            let mut item = div()
                .h_flex()
                .items_start()
                .gap_2()
                .p_2()
                .rounded(px(3.0))
                .border_1()
                .border_color(theme.border)
                .bg(theme.background)
                .child(severity_badge(f.sev, cx));
            let mut details = div()
                .v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(2.0))
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(f.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(f.text.clone()),
                );
            if let Some(fix) = &f.fix {
                details = details.child(
                    div()
                        .text_xs()
                        .text_color(theme.primary)
                        .child(format!("→ {}", fix)),
                );
            }
            item = item.child(details);
            fix_list = fix_list.child(item);
        }
    }
    fix_list = fix_list.child(
        div().pt_1().child(
            Button::new("home-full-diagnosis")
                .outline()
                .small()
                .label(l(lang, "Ver diagnóstico completo", "View full diagnosis"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.section = NavSection::Tactic;
                    this.tactic_tab = TacticTab::Analyse;
                    cx.notify();
                })),
        ),
    );

    let fix_card = collapsible_card(
        "home-card-fix",
        l(lang, "O que corrigir primeiro", "What to fix first"),
        Some(
            match lang {
                Lang::Pt => format!("Pontuação global: {}/100", audit.overall),
                Lang::En => format!("Overall score: {}/100", audit.overall),
            }
            .into(),
        ),
        app.state.is_panel_collapsed("home-fix"),
        cx.listener(|this, _, _, cx| {
            this.state.toggle_panel("home-fix");
            cx.notify();
        }),
        fix_list,
        cx,
    );

    // ------------------------------------------------------------------------
    // Right column: Best system + Opponent summary (or Animated Pitch shortcut)
    // ------------------------------------------------------------------------
    let mut right_col = div().v_flex().flex_1().min_w(px(300.0)).gap_4();

    if let Some(s0) = best_sugg {
        let tp_name = template_by_id(s0.template_id)
            .map(|tp| t(lang, tp.name_pt))
            .unwrap_or_else(|| s0.tactic.name.clone());
        let tactic_clone = s0.tactic.clone();
        let sugg_body = div()
            .v_flex()
            .gap_2()
            .child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_3()
                    .child(score_ring(s0.total, 56.0, cx))
                    .child(
                        div()
                            .v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(theme.foreground)
                                    .child(tp_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(match lang {
                                        Lang::Pt => format!(
                                            "XI {} · estilo {} · profundidade {}",
                                            s0.xi_avg, s0.fit, s0.depth
                                        ),
                                        Lang::En => format!(
                                            "XI {} · style {} · depth {}",
                                            s0.xi_avg, s0.fit, s0.depth
                                        ),
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("home-sugg-ranking")
                            .outline()
                            .small()
                            .label(l(lang, "Ver ranking completo", "View full ranking"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.section = NavSection::Team;
                                this.team_tab = TeamTab::BestXi;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("home-sugg-use")
                            .primary()
                            .small()
                            .label(l(lang, "Abrir no analisador", "Open in analyser"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                apply_tactic_to_app(this, tactic_clone.clone(), true, cx);
                            })),
                    ),
            );

        right_col = right_col.child(collapsible_card(
            "home-card-sugg",
            l(
                lang,
                "Sistema que melhor serve o plantel",
                "Best system for your squad",
            ),
            None,
            app.state.is_panel_collapsed("home-sugg"),
            cx.listener(|this, _, _, cx| {
                this.state.toggle_panel("home-sugg");
                cx.notify();
            }),
            sugg_body,
            cx,
        ));
    }

    if let Some(rep) = &opp_rep {
        let opp_title = format!(
            "{}: {}",
            l(lang, "Próximo adversário", "Next opponent"),
            rep.club.as_deref().unwrap_or("")
        );
        let mut opp_summary = div().v_flex().gap_2();
        if rep.plan.is_empty() {
            opp_summary = opp_summary.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(l(lang, "Sem ajustes necessários.", "No adjustments needed.")),
            );
        } else {
            for item in rep.plan.iter().take(2) {
                let ti_lbl = ti_by_key(item.ti)
                    .map(|def| {
                        let opt = def.opts.get(item.value).copied().unwrap_or("");
                        format!("{} → {}", ti_name(def, lang), ti_opt_name(opt, lang))
                    })
                    .unwrap_or_else(|| item.ti.to_string());
                opp_summary = opp_summary.child(
                    div()
                        .v_flex()
                        .gap(px(1.0))
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(format!("• {}", ti_lbl)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(item.why.clone()),
                        ),
                );
            }
        }
        opp_summary = opp_summary.child(
            Button::new("home-opp-open")
                .outline()
                .small()
                .label(l(lang, "Ver relatório", "View report"))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.section = NavSection::Team;
                    this.team_tab = TeamTab::Opponent;
                    cx.notify();
                })),
        );

        right_col = right_col.child(collapsible_card(
            "home-card-opp",
            opp_title,
            None,
            app.state.is_panel_collapsed("home-opp"),
            cx.listener(|this, _, _, cx| {
                this.state.toggle_panel("home-opp");
                cx.notify();
            }),
            opp_summary,
            cx,
        ));
    }

    if squad.len() < 11 && opp.is_empty() {
        let sim_card_body = div()
            .v_flex()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(l(
                        lang,
                        "Veja a sua tática em movimento nas 6 fases de jogo e teste instruções em tempo real.",
                        "Watch your tactic move across all 6 match phases and test instructions in real time.",
                    )),
            )
            .child(
                Button::new("home-sim-jump")
                    .primary()
                    .small()
                    .label(l(lang, "▶ Abrir campo animado", "▶ Open animated pitch"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.section = NavSection::Tactic;
                        this.tactic_tab = TacticTab::AnimatedPitch;
                        cx.notify();
                    })),
            );
        right_col = right_col.child(collapsible_card(
            "home-card-sim",
            l(lang, "Campo animado", "Animated pitch"),
            None,
            app.state.is_panel_collapsed("home-sim"),
            cx.listener(|this, _, _, cx| {
                this.state.toggle_panel("home-sim");
                cx.notify();
            }),
            sim_card_body,
            cx,
        ));
    }

    let main_grid = div()
        .h_flex()
        .flex_wrap()
        .items_start()
        .gap_4()
        .child(div().flex_1().min_w(px(300.0)).child(fix_card))
        .child(right_col);

    // ------------------------------------------------------------------------
    // Saved tactics bar
    // ------------------------------------------------------------------------
    let saved_section = if app.state.saved_tactics.is_empty() {
        None
    } else {
        let mut row = div().h_flex().flex_wrap().gap_2();
        for (idx, saved) in app.state.saved_tactics.iter().take(8).enumerate() {
            let label = saved.name.clone();
            let tac = saved.clone();
            row = row.child(
                Button::new(("home-saved", idx))
                    .outline()
                    .small()
                    .label(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let name = tac.name.clone();
                        apply_tactic_to_app(this, tac.clone(), true, cx);
                        this.state.set_toast(
                            format!(
                                "{}: {}",
                                this.state.lang.pick("Tática carregada", "Tactic loaded"),
                                name
                            ),
                            ToastKind::Info,
                        );
                        cx.notify();
                    })),
            );
        }
        Some(collapsible_card(
            "home-card-saved",
            l(lang, "Táticas guardadas", "Saved tactics"),
            None,
            app.state.is_panel_collapsed("home-saved"),
            cx.listener(|this, _, _, cx| {
                this.state.toggle_panel("home-saved");
                cx.notify();
            }),
            row,
            cx,
        ))
    };

    div()
        .v_flex()
        .w_full()
        .gap_4()
        .child(header)
        .child(steps_row)
        .child(main_grid)
        .when_some(saved_section, |d, card| d.child(card))
}
