//! My Team › Generate Tactic view wired to `ta_core::planning::generator`.

use crate::state::{NavSection, TacticTab, ToastKind};
use crate::widgets::common::{
    badge_pill, empty_state_card, score_color, score_ring, suitability_bar, BadgeTone,
};
use crate::widgets::synergy_map::dual_synergy_maps;
use crate::views::tactic_sheet::sheet_text;
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use ta_core::data::{
    attr_name, mentality_name, pi_name, role_by_id, role_name, slot_info, ti_by_key, ti_name,
    ti_opt_name, ti_val, training_area, FM_SECTIONS,
};
use ta_core::engine::{recommend_pis, set_piece_plan, slot_score, training_plan};
use ta_core::planning::{depth_chart, generate_tactic, weak_spots};
use ta_core::types::{Expectation, GeneratorConfig, GeneratorResult, Player, SynergyPhase};

/// View-local UI state for the Tactic Generator.
#[derive(Debug, Clone)]
pub struct GeneratorViewState {
    pub config: GeneratorConfig,
    pub result: Option<GeneratorResult>,
    pub pick: usize,
    pub weak_foot_flanks: bool,
}

impl Default for GeneratorViewState {
    fn default() -> Self {
        Self {
            config: GeneratorConfig::default(),
            result: None,
            pick: 0,
            weak_foot_flanks: true,
        }
    }
}

/// Runs `generate_tactic` off-thread with busy indicator and stores the result.
pub fn spawn_generate_tactic(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let squad = app.state.squad.clone();
    if squad.len() < 11 {
        app.state.set_toast(
            lang.pick(
                "Carregue um plantel com pelo menos 11 jogadores primeiro.",
                "Load a squad with at least 11 players first.",
            ),
            ToastKind::Warn,
        );
        cx.notify();
        return;
    }

    let cfg = app.gen_ui.config.clone();
    app.state.set_busy(
        lang.pick(
            "A gerar tática ideal para o plantel…",
            "Generating optimal tactic for your squad…",
        ),
        Some(0.25),
    );
    cx.notify();

    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move { generate_tactic(&squad, &cfg, lang, |_| {}) })
            .await;

        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            match res {
                Some(gen) => {
                    let msg = format!(
                        "{} ({} {})",
                        this.state
                            .lang
                            .pick("Tática gerada", "Tactic generated"),
                        gen.considered,
                        this.state.lang.pick("pares avaliados", "pairs evaluated")
                    );
                    this.gen_ui.result = Some(gen);
                    this.gen_ui.pick = 0;
                    this.state.set_toast(msg, ToastKind::Success);
                }
                None => {
                    this.state.set_toast(
                        this.state.lang.pick(
                            "Não foi possível gerar uma tática com este plantel.",
                            "Could not generate a tactic with this squad.",
                        ),
                        ToastKind::Error,
                    );
                }
            }
            cx.notify();
        });
    })
    .detach();
}

/// Renders the My Team › Generate Tactic view.
pub fn render_generator(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme();
    let squad = &app.state.squad;

    if squad.len() < 11 {
        return div().v_flex().w_full().child(empty_state_card(
            lang.pick("Carregue o plantel primeiro", "Load your squad first"),
            lang.pick(
                "O gerador monta uma tática completa à volta do teu plantel: formas IP/OOP, XI húngaro, 27 instruções com justificação, bolas paradas, profundidade, reforços e 3 alternativas.",
                "The generator builds a complete tactic around your squad: IP/OOP shapes, Hungarian XI, all 27 instructions with reasons, set pieces, depth, weak spots, and 3 alternatives.",
            ),
            Some(
                Button::new("gen-read-save")
                    .primary()
                    .small()
                    .label(lang.pick("Ler save agora", "Read save now"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.spawn_read_save_now(cx);
                    }))
                    .into_any_element(),
            ),
            cx,
        ));
    }

    let mut by_name: Vec<&Player> = squad.iter().collect();
    by_name.sort_by(|a, b| a.name.cmp(&b.name));
    let young: Vec<&Player> = by_name
        .iter()
        .copied()
        .filter(|p| p.age.map(|a| a <= 21.0).unwrap_or(false))
        .collect();

    let cfg = &app.gen_ui.config;
    let must_set: HashSet<&str> = cfg.must.iter().map(String::as_str).collect();
    let youth_set: HashSet<&str> = cfg.youth.iter().map(String::as_str).collect();

    // Context configuration panel
    let mut expect_row = div().h_flex().flex_wrap().gap_1();
    for exp in Expectation::ALL {
        let active = cfg.expect == exp;
        expect_row = expect_row.child(
            Button::new(("gen-exp", exp as usize))
                .xsmall()
                .label(exp.label(lang))
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.gen_ui.config.expect = exp;
                    cx.notify();
                })),
        );
    }

    let mut star_chips = div().h_flex().flex_wrap().gap_1().child(
        Button::new("gen-star-none")
            .xsmall()
            .label(lang.pick("Nenhum", "None"))
            .when(cfg.star.is_none(), |b| b.primary())
            .when(cfg.star.is_some(), |b| b.outline())
            .on_click(cx.listener(|this, _, _, cx| {
                this.gen_ui.config.star = None;
                cx.notify();
            })),
    );
    for (i, p) in by_name.iter().take(28).enumerate() {
        let pid = p.id.clone();
        let active = cfg.star.as_deref() == Some(p.id.as_str());
        star_chips = star_chips.child(
            Button::new(("gen-star", i))
                .xsmall()
                .label(p.name.clone())
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.gen_ui.config.star = if this.gen_ui.config.star.as_deref() == Some(pid.as_str()) {
                        None
                    } else {
                        Some(pid.clone())
                    };
                    cx.notify();
                })),
        );
    }

    let mut must_chips = div().h_flex().flex_wrap().gap_1();
    for (i, p) in by_name.iter().take(28).enumerate() {
        let pid = p.id.clone();
        let active = must_set.contains(p.id.as_str());
        must_chips = must_chips.child(
            Button::new(("gen-must", i))
                .xsmall()
                .label(p.name.clone())
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(pos) = this.gen_ui.config.must.iter().position(|x| x == &pid) {
                        this.gen_ui.config.must.remove(pos);
                    } else if this.gen_ui.config.must.len() < 11 {
                        this.gen_ui.config.must.push(pid.clone());
                    }
                    cx.notify();
                })),
        );
    }

    let mut youth_chips = div().h_flex().flex_wrap().gap_1();
    for (i, p) in young.iter().take(18).enumerate() {
        let pid = p.id.clone();
        let active = youth_set.contains(p.id.as_str());
        let lbl = format!("{} ({})", p.name, p.age.unwrap_or(18.0).round() as i32);
        youth_chips = youth_chips.child(
            Button::new(("gen-youth", i))
                .xsmall()
                .label(lbl)
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(pos) = this.gen_ui.config.youth.iter().position(|x| x == &pid) {
                        this.gen_ui.config.youth.remove(pos);
                    } else if this.gen_ui.config.youth.len() < 3 {
                        this.gen_ui.config.youth.push(pid.clone());
                    } else {
                        this.state.set_toast(
                            this.state.lang.pick(
                                "Máximo de 3 jovens prioritários.",
                                "Maximum of 3 priority youngsters.",
                            ),
                            ToastKind::Info,
                        );
                    }
                    cx.notify();
                })),
        );
    }

    let mut left_col = div()
        .v_flex()
        .w(px(340.0))
        .flex_shrink_0()
        .gap_3()
        .child(
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
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("O teu contexto", "Your context")),
                )
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.muted_foreground)
                                .child(lang.pick("Objetivo na liga", "League objective")),
                        )
                        .child(expect_row),
                )
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.muted_foreground)
                                .child(lang.pick(
                                    "Jogador-estrela (constrói à volta dele)",
                                    "Star player (build around him)",
                                )),
                        )
                        .child(star_chips),
                )
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{} ({})",
                                    lang.pick("Titulares obrigatórios", "Mandatory starters"),
                                    cfg.must.len()
                                )),
                        )
                        .child(must_chips),
                )
                .when(!young.is_empty(), |d| {
                    d.child(
                        div()
                            .v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(theme.muted_foreground)
                                    .child(format!(
                                        "{} ({}/3)",
                                        lang.pick("Jovens a desenvolver (≤ 21)", "Youngsters to develop (≤ 21)"),
                                        cfg.youth.len()
                                    )),
                            )
                            .child(youth_chips),
                    )
                })
                .child(
                    Switch::new("gen-use-synergy")
                        .checked(cfg.use_synergy)
                        .label(lang.pick(
                            "Otimizar sinergia entre papéis",
                            "Optimize role synergy",
                        ))
                        .on_change(cx.listener(|this, v: &bool, _, cx| {
                            this.gen_ui.config.use_synergy = *v;
                            cx.notify();
                        })),
                )
                .child(
                    Switch::new("gen-weak-foot")
                        .checked(app.gen_ui.weak_foot_flanks)
                        .label(lang.pick(
                            "Considerar pé contrário nos flancos",
                            "Consider inverted foot on flanks",
                        ))
                        .on_change(cx.listener(|this, v: &bool, _, cx| {
                            this.gen_ui.weak_foot_flanks = *v;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("gen-run")
                        .primary()
                        .w_full()
                        .label(if app.gen_ui.result.is_some() {
                            lang.pick("Gerar de novo", "Generate again")
                        } else {
                            lang.pick("Gerar tática", "Generate tactic")
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_generate_tactic(this, cx);
                        })),
                ),
        );

    let Some(res) = &app.gen_ui.result else {
        return div()
            .h_flex()
            .w_full()
            .items_start()
            .gap_4()
            .child(left_col)
            .child(div().flex_1().child(empty_state_card(
                lang.pick("Gerar tática à medida", "Generate a tailored tactic"),
                lang.pick(
                    "Escolha o objetivo, jogador-estrela, titulares obrigatórios ou jovens e clique em Gerar tática.",
                    "Pick your objective, star player, mandatory starters or youngsters and click Generate tactic.",
                ),
                None,
                cx,
            )));
    };

    // Candidate selector list on left column
    let mut all_candidates = Vec::with_capacity(1 + res.alts.len());
    all_candidates.push(&res.main);
    for alt in &res.alts {
        all_candidates.push(alt);
    }
    let picked_idx = app.gen_ui.pick.min(all_candidates.len().saturating_sub(1));
    let cand = all_candidates[picked_idx];
    let tac = &cand.tactic;

    let mut cand_list = div()
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
                .child(lang.pick("Opções geradas", "Generated options")),
        );

    for (i, c) in all_candidates.iter().enumerate() {
        let active = i == picked_idx;
        let heading = if i == 0 {
            lang.pick("Recomendada", "Recommended").to_string()
        } else {
            format!("{} {}", lang.pick("Alternativa", "Alternative"), i)
        };
        cand_list = cand_list.child(
            div()
                .id(("gen-pick", i))
                .h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .p_2()
                .rounded_sm()
                .cursor_pointer()
                .border_1()
                .border_color(if active { theme.primary } else { theme.border })
                .bg(if active { theme.accent } else { theme.background })
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.gen_ui.pick = i;
                    cx.notify();
                }))
                .child(
                    div()
                        .v_flex()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(heading),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.foreground)
                                .child(format!("{} → {}", c.ip_form, c.oop_form)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(format!(
                                    "{} {} · {} {} · {} {}",
                                    lang.pick("adeq.", "fit"),
                                    c.fit_avg,
                                    lang.pick("sin.", "syn"),
                                    c.syn,
                                    lang.pick("diag.", "diag"),
                                    c.audit.overall
                                )),
                        ),
                )
                .child(score_ring(c.total, 44.0, cx)),
        );
    }
    left_col = left_col.child(cand_list);

    // Precompute rich report sections
    let pm: HashMap<&str, &Player> = squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let sp = set_piece_plan(tac, squad, &app.state.opponent, lang);
    let tp = training_plan(tac, squad, &cand.audit, lang);
    let dc = depth_chart(tac, squad, 3);
    let weak = weak_spots(tac, squad);

    // Header actions
    let tac_for_use = tac.clone();
    let tac_for_pitch = tac.clone();
    let sheet_txt = sheet_text(tac, lang);

    let header = div()
        .h_flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_2()
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
                        .text_base()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!("{} → {}", tac.ip_form, tac.oop_form)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "{} {} · {} {}",
                            res.considered,
                            lang.pick("pares avaliados", "pairs evaluated"),
                            res.optimised,
                            lang.pick("otimizados a fundo", "deeply optimized")
                        )),
                ),
        )
        .child(
            div()
                .h_flex()
                .flex_wrap()
                .gap_2()
                .child(
                    Button::new("gen-use")
                        .primary()
                        .small()
                        .label(lang.pick("Usar esta tática", "Use this tactic"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.apply_active_tactic(tac_for_use.clone(), cx);
                            this.section = NavSection::Tactic;
                            this.tactic_tab = TacticTab::Analyse;
                            this.state.set_toast(
                                this.state
                                    .lang
                                    .pick("Tática gerada aplicada.", "Generated tactic applied."),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("gen-view-pitch")
                        .outline()
                        .small()
                        .label(lang.pick("▶ Ver em campo", "▶ View on pitch"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.apply_active_tactic(tac_for_pitch.clone(), cx);
                            this.section = NavSection::Tactic;
                            this.tactic_tab = TacticTab::AnimatedPitch;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("gen-copy-text")
                        .outline()
                        .small()
                        .label(lang.pick("Copiar como texto", "Copy as text"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(sheet_txt.clone()));
                            this.state.set_toast(
                                this.state.lang.pick(
                                    "Tática copiada para a área de transferência.",
                                    "Tactic copied to clipboard.",
                                ),
                                ToastKind::Success,
                            );
                            cx.notify();
                        })),
                ),
        );

    // Section 1: Mentality
    let sec1 = div()
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
                .child(format!(
                    "1. {} — {}",
                    lang.pick("Estilo e mentalidade", "Style & mentality"),
                    mentality_name(tac.ment, lang)
                )),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(cand.ment_why.clone()),
        );

    // Section 2: XI Table
    let mut xi_rows = div().v_flex().gap_1();
    for s in &tac.slots {
        let player = s.pid.as_deref().and_then(|id| pm.get(id)).copied();
        let sc = player.map(|p| slot_score(p, tac, s));
        let ip_r = role_by_id(&s.ip)
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| s.ip.clone());
        let oop_r = role_by_id(&s.oop)
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| s.oop.clone());
        let pos_lbl = if s.ip_code == s.oop_code {
            s.ip_code.clone()
        } else {
            format!("{} → {}", s.ip_code, s.oop_code)
        };
        let is_star = player
            .map(|p| cfg.star.as_deref() == Some(p.id.as_str()))
            .unwrap_or(false);
        let is_youth = player
            .map(|p| youth_set.contains(p.id.as_str()))
            .unwrap_or(false);

        xi_rows = xi_rows.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .py_1()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .w(px(90.0))
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(pos_lbl),
                )
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_1()
                        .w(px(180.0))
                        .text_xs()
                        .text_color(theme.foreground)
                        .child(player.map(|p| p.name.clone()).unwrap_or_else(|| "—".into()))
                        .when(is_star, |d| {
                            d.child(badge_pill(lang.pick("estrela", "star"), BadgeTone::Info, cx))
                        })
                        .when(is_youth, |d| {
                            d.child(badge_pill(lang.pick("jovem", "youth"), BadgeTone::Good, cx))
                        }),
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!("IP: {} · OOP: {}", ip_r, oop_r)),
                )
                .when_some(sc, |d, val| {
                    d.child(
                        div()
                            .w(px(34.0))
                            .text_xs()
                            .font_bold()
                            .text_right()
                            .text_color(score_color(val))
                            .child(val.to_string()),
                    )
                }),
        );
    }

    let sec2 = div()
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
                .child(format!("2. {}", lang.pick("Formas e XI titular", "Formations & Starting XI"))),
        )
        .child(xi_rows);

    // Section 3: Synergy (with dual canvas maps)
    let mut phase_bars = div().h_flex().flex_wrap().gap_3();
    for ph in SynergyPhase::ALL {
        let lbl = match ph {
            SynergyPhase::Build => lang.pick("Construção", "Build-up"),
            SynergyPhase::Att => lang.pick("Ataque", "Attack"),
            SynergyPhase::Def => lang.pick("Defesa", "Defense"),
            SynergyPhase::Trans => lang.pick("Transição", "Transition"),
        };
        let sc = cand
            .synergy
            .phases
            .get(&ph)
            .map(|p| p.score)
            .unwrap_or(0);
        phase_bars = phase_bars.child(div().flex_1().min_w(px(160.0)).child(suitability_bar(lbl, sc, 100, cx)));
    }
    let sec3 = div()
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
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!(
                            "3. {}",
                            lang.pick("Sinergia entre papéis", "Role synergy across phases")
                        )),
                )
                .child(score_ring(cand.synergy.score, 46.0, cx)),
        )
        .child(phase_bars)
        .child(dual_synergy_maps(tac, &cand.synergy, lang, cx));

    // Section 4: Team Instructions with reasons
    let mut ti_body = div().v_flex().gap_2();
    for fm_sec in &FM_SECTIONS {
        ti_body = ti_body.child(
            div()
                .text_xs()
                .font_semibold()
                .pt_1()
                .text_color(theme.foreground)
                .child(fm_sec.title),
        );
        for &k in fm_sec.keys {
            if let Some(d) = ti_by_key(k) {
                let v = ti_val(tac, k);
                let changed = v != d.def;
                let why = cand.reasons.get(k).cloned().unwrap_or_default();
                ti_body = ti_body.child(
                    div()
                        .h_flex()
                        .items_start()
                        .gap_2()
                        .text_xs()
                        .child(
                            div()
                                .w(px(170.0))
                                .text_color(if changed {
                                    theme.foreground
                                } else {
                                    theme.muted_foreground
                                })
                                .child(ti_name(d, lang)),
                        )
                        .child(
                            div()
                                .w(px(140.0))
                                .font_semibold()
                                .text_color(if changed {
                                    theme.primary
                                } else {
                                    theme.muted_foreground
                                })
                                .child(ti_opt_name(d.opts.get(v).copied().unwrap_or(""), lang)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(theme.muted_foreground)
                                .child(why),
                        ),
                );
            }
        }
    }
    let sec4 = div()
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
                .child(format!("4. {}", lang.pick("Instruções de equipa (27)", "Team instructions (27)"))),
        )
        .child(ti_body);

    // Section 5: Individual Instructions
    let mut pi_list = div().v_flex().gap_1();
    let mut pi_count = 0usize;
    for s in &tac.slots {
        if let (Some(p), Some(meta)) = (
            s.pid.as_deref().and_then(|id| pm.get(id)).copied(),
            slot_info(&tac.ip_form, &s.ip_code),
        ) {
            let recs = recommend_pis(p, role_by_id(&s.ip), role_by_id(&s.oop), meta.group, lang);
            if !recs.is_empty() {
                pi_count += 1;
                let pills: String = recs
                    .iter()
                    .take(2)
                    .map(|r| pi_name(r.pi, lang))
                    .collect::<Vec<_>>()
                    .join(" · ");
                pi_list = pi_list.child(
                    div()
                        .h_flex()
                        .gap_2()
                        .text_xs()
                        .child(
                            div()
                                .w(px(160.0))
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(format!("{} {}", s.ip_code, p.name)),
                        )
                        .child(
                            div()
                                .w(px(200.0))
                                .text_color(theme.primary)
                                .child(pills),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_color(theme.muted_foreground)
                                .child(recs[0].why.clone()),
                        ),
                );
            }
        }
    }
    let sec5 = div()
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
                .child(format!("5. {}", lang.pick("Instruções individuais", "Player instructions"))),
        )
        .child(if pi_count > 0 {
            pi_list.into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(lang.pick(
                    "Nenhum atributo exige instruções individuais extra.",
                    "No attributes require extra individual instructions.",
                ))
                .into_any_element()
        });

    // Section 6: Set Pieces & Captains
    let tac_for_sp = tac.clone();
    let sec6 = sp.map(|plan| {
        let cap1 = plan.caps.first().map(|c| c.player_name.as_str()).unwrap_or("—");
        let cap2 = plan.caps.get(1).map(|c| c.player_name.as_str()).unwrap_or("—");
        let pens = plan
            .pens
            .iter()
            .map(|p| p.player_name.as_str())
            .collect::<Vec<_>>()
            .join(" → ");
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
                    .h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(format!("6. {}", lang.pick("Bolas paradas e capitães", "Set pieces & captains"))),
                    )
                    .child(
                        Button::new("gen-open-sp")
                            .outline()
                            .xsmall()
                            .label(lang.pick("Ver diagramas", "View diagrams"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.apply_active_tactic(tac_for_sp.clone(), cx);
                                this.section = NavSection::Tactic;
                                this.tactic_tab = TacticTab::SetPieces;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(format!(
                        "{}: {} / {} · {}: {} / {}",
                        lang.pick("Capitães", "Captains"),
                        cap1,
                        cap2,
                        lang.pick("Cantos (dir./esq.)", "Corners (R/L)"),
                        plan.corner_right.player_name,
                        plan.corner_left.player_name
                    )),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(format!("{}: {}", lang.pick("Penáltis", "Penalties"), pens)),
            )
    });

    // Section 7: Depth chart
    let mut depth_grid = div().h_flex().flex_wrap().gap_3();
    for slot in &dc {
        let mut col = div()
            .v_flex()
            .w(px(170.0))
            .p_2()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .bg(theme.background)
            .gap(px(2.0))
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child(slot.code.clone()),
            );
        for (i, c) in slot.list.iter().enumerate() {
            col = col.child(
                div()
                    .h_flex()
                    .justify_between()
                    .text_xs()
                    .child(
                        div()
                            .text_color(theme.muted_foreground)
                            .truncate()
                            .child(format!("{}. {}", i + 1, c.player_name)),
                    )
                    .child(
                        div()
                            .font_semibold()
                            .text_color(score_color(c.score))
                            .child(c.score.to_string()),
                    ),
            );
        }
        depth_grid = depth_grid.child(col);
    }
    let sec7 = div()
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
                .child(format!("7. {}", lang.pick("Profundidade por posição", "Depth chart"))),
        )
        .child(depth_grid);

    // Section 8: Weak spots
    let mut weak_list = div().v_flex().gap_1();
    for w in &weak {
        let rname = role_by_id(w.role_id)
            .map(|r| role_name(r, lang))
            .unwrap_or_else(|| w.role_id.to_string());
        let lack = w
            .lack
            .iter()
            .map(|&a| attr_name(a, lang))
            .collect::<Vec<_>>()
            .join(", ");
        weak_list = weak_list.child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .text_xs()
                .child(badge_pill(
                    w.code.clone(),
                    if w.urgent { BadgeTone::Crit } else { BadgeTone::Warn },
                    cx,
                ))
                .child(
                    div()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(rname),
                )
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "{} {} · {} {}",
                            lang.pick("titular", "starter"),
                            w.score,
                            lang.pick("alt.", "backup"),
                            w.alt
                        )),
                )
                .when(!lack.is_empty(), |d| {
                    d.child(
                        div()
                            .flex_1()
                            .text_color(theme.muted_foreground)
                            .child(format!("{}: {}", lang.pick("procurar", "look for"), lack)),
                    )
                }),
        );
    }
    let sec8 = div()
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
                .child(format!("8. {}", lang.pick("Onde reforçar", "Weak spots to reinforce"))),
        )
        .child(if weak.is_empty() {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(lang.pick(
                    "Todas as posições têm titular ≥ 55 e alternativa ≥ 45.",
                    "Every slot has starter ≥ 55 and backup ≥ 45.",
                ))
                .into_any_element()
        } else {
            weak_list.into_any_element()
        });

    // Section 9: Training priorities
    let tac_for_train = tac.clone();
    let mut train_list = div().v_flex().gap_1();
    for area in tp.areas.iter().take(4) {
        let aname = training_area(area.key)
            .map(|a| a.name)
            .unwrap_or(area.key);
        let why = area.why.first().cloned().unwrap_or_default();
        train_list = train_list.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("• {} — {}", aname, why)),
        );
    }
    let sec9 = div()
        .v_flex()
        .gap_2()
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
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!("9. {}", lang.pick("Prioridades de treino", "Training priorities"))),
                )
                .child(
                    Button::new("gen-open-train")
                        .outline()
                        .xsmall()
                        .label(lang.pick("Plano completo", "Full training plan"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.apply_active_tactic(tac_for_train.clone(), cx);
                            this.section = NavSection::Training;
                            cx.notify();
                        })),
                ),
        )
        .child(train_list);

    let right_col = div()
        .v_flex()
        .flex_1()
        .min_w_0()
        .gap_3()
        .child(header)
        .child(sec1)
        .child(sec2)
        .child(sec3)
        .child(sec4)
        .child(sec5)
        .when_some(sec6, |d, s| d.child(s))
        .child(sec7)
        .child(sec8)
        .child(sec9);

    div()
        .h_flex()
        .w_full()
        .items_start()
        .gap_4()
        .child(left_col)
        .child(right_col)
}
