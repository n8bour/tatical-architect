//! Tactic › Set Pieces: takers, captains, corner routines (attack/defend diagrams),
//! one-click TI tips and FM Bridge "apply in FM" (explicit click, off-thread).

use crate::state::{BackendBridgeTransport, NavSection, TacticTab, ToastKind};
use crate::views::fm_bridge_ui::{fm_button, spawn_refresh_bridge_status};
use crate::views::team::sync_animated_pitch;
use crate::widgets::common::{empty_state_card, BadgeTone, badge_pill};
use crate::widgets::setpieces_diagram::setpieces_diagrams_pair;
use crate::ArchitectApp;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::*;
use std::collections::HashMap;
use ta_core::data::{attr_name, l, t_name, ti_by_key, ti_name, ti_opt_name, ti_val};
use ta_core::engine::set_piece_plan;
use ta_core::live::fmbridge::FmbClient;
use ta_core::types::{Attr, Lang, Player, SetPieceRole};

fn attrs_line(p: Option<&Player>, ks: &[Attr], lang: Lang) -> String {
    let Some(p) = p else { return String::new() };
    ks.iter()
        .map(|k| {
            format!(
                "{} {}",
                attr_name(*k, lang),
                p.a.get(k).copied().unwrap_or(0.0).round() as i32
            )
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn taker_row(
    label: String,
    who: Option<(&str, &str)>,
    extra: String,
    theme: &gpui_kit::component::theme::Theme,
) -> Div {
    div()
        .h_flex()
        .gap_3()
        .py_1()
        .border_b_1()
        .border_color(theme.border)
        .text_xs()
        .child(div().w(px(170.0)).text_color(theme.foreground).child(label))
        .child(
            div().w(px(190.0)).h_flex().gap_1().children(match who {
                Some((name, code)) => vec![
                    div().font_semibold().text_color(theme.foreground).child(name.to_string()),
                    div().text_color(theme.muted_foreground).child(code.to_string()),
                ],
                None => vec![div().text_color(theme.muted_foreground).child("—")],
            }),
        )
        .child(div().flex_1().text_color(theme.muted_foreground).child(extra))
}

fn routine_table(rows: &[SetPieceRole], theme: &gpui_kit::component::theme::Theme) -> Div {
    div().v_flex().children(rows.iter().map(|r| {
        div()
            .h_flex()
            .gap_3()
            .py_1()
            .border_b_1()
            .border_color(theme.border)
            .text_xs()
            .child(div().w(px(150.0)).text_color(theme.foreground).child(r.label.clone()))
            .child(div().w(px(160.0)).font_semibold().text_color(theme.foreground).child(r.player_name.clone()))
            .child(div().flex_1().text_color(theme.muted_foreground).child(r.why.clone()))
    }))
}

fn panel(title: impl Into<SharedString>, theme: &gpui_kit::component::theme::Theme) -> Div {
    div()
        .v_flex()
        .flex_1()
        .min_w(px(360.0))
        .gap_2()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(div().text_sm().font_semibold().text_color(theme.foreground).child(title.into()))
}

/// Applies the set-piece plan in FM27 via the bridge, off-thread.
fn spawn_apply_set_pieces(app: &mut ArchitectApp, cx: &mut Context<ArchitectApp>) {
    let lang = app.state.lang;
    let squad = app.state.squad.clone();
    let Some(plan) = set_piece_plan(&app.state.tactic, &squad, &app.state.opponent, lang) else {
        app.state.set_toast(
            l(lang, "Escolha o XI primeiro (Tática › Analisar).", "Pick the XI first (Tactic › Analyse)."),
            ToastKind::Warn,
        );
        cx.notify();
        return;
    };
    app.state.set_busy(
        l(
            lang,
            "A aplicar as bolas paradas no FM — no FM, abra Táticas › Bolas paradas",
            "Applying set pieces in FM — in FM, open Tactics › Set Pieces",
        ),
        None,
    );
    cx.notify();
    cx.spawn(async move |this, cx| {
        let res = cx
            .background_executor()
            .spawn(async move {
                let mut client = FmbClient::new(BackendBridgeTransport, lang);
                client.apply_set_pieces(&plan, &squad, |_| {})
            })
            .await;
        let _ = this.update(cx, |this, cx| {
            this.state.clear_busy();
            let lang = this.state.lang;
            match res {
                Ok(done) => {
                    let mut msg = l(lang, "Bolas paradas aplicadas no FM", "Set pieces applied in FM").to_string();
                    if !done.is_empty() {
                        msg.push_str(": ");
                        msg.push_str(&done.join(" · "));
                    }
                    msg.push_str(l(
                        lang,
                        ". Penáltis e lançamentos longos: siga a ficha.",
                        ". Penalties and long throws: follow the sheet.",
                    ));
                    this.state.set_toast(msg, ToastKind::Success);
                }
                Err(e) => this.state.set_toast(e.to_string(), ToastKind::Error),
            }
            spawn_refresh_bridge_status(this, cx);
            cx.notify();
        });
    })
    .detach();
}

/// Renders Tactic › Set Pieces.
pub fn render_setpieces(app: &ArchitectApp, cx: &mut Context<ArchitectApp>) -> impl IntoElement {
    let lang = app.state.lang;
    let theme = cx.theme().clone();
    let ps = &app.state.squad;

    if ps.len() < 11 {
        let go = Button::new("sp-go-squad")
            .primary()
            .small()
            .label(l(lang, "Ir para o plantel", "Go to squad"))
            .on_click(cx.listener(|this, _, _, cx| {
                this.section = NavSection::Team;
                cx.notify();
            }))
            .into_any_element();
        return div()
            .child(empty_state_card(
                l(lang, "Carregue o plantel primeiro", "Load your squad first"),
                l(
                    lang,
                    "As bolas paradas são escolhidas a partir dos atributos dos jogadores do teu XI.",
                    "Set-piece takers are picked from the attributes of the players in your XI.",
                ),
                Some(go),
                cx,
            ))
            .into_any_element();
    }

    let Some(r) = set_piece_plan(&app.state.tactic, ps, &app.state.opponent, lang) else {
        return div()
            .child(empty_state_card(
                l(lang, "Não consegui montar o XI", "Couldn't build the XI"),
                l(
                    lang,
                    "Atribua jogadores às posições em Tática › Analisar.",
                    "Assign players to the slots in Tactic › Analyse.",
                ),
                None,
                cx,
            ))
            .into_any_element();
    };

    let pm: HashMap<&str, &Player> = ps.iter().map(|p| (p.id.as_str(), p)).collect();
    let pl = |id: &str| pm.get(id).copied();
    let has_opp = !app.state.opponent.is_empty();

    // Header
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
                .child(div().text_base().font_semibold().text_color(theme.foreground).child(l(
                    lang,
                    "Bolas paradas & capitães",
                    "Set pieces & captains",
                )))
                .child(div().text_xs().text_color(theme.muted_foreground).child(format!(
                    "{} “{}”. {}",
                    l(lang, "Escolhidos entre o XI de", "Picked from the XI of"),
                    t_name(&app.state.tactic, lang),
                    if has_opp {
                        l(
                            lang,
                            "Marcações individuais contra os mais fortes no ar do adversário carregado.",
                            "Man-marking against the loaded opponent's strongest players in the air.",
                        )
                    } else {
                        l(
                            lang,
                            "Carregue o adversário para marcações individuais nominais.",
                            "Load the opponent for named man-marking assignments.",
                        )
                    }
                ))),
        )
        .child(
            div()
                .h_flex()
                .gap_2()
                .child(
                    Button::new("sp-to-sheet")
                        .outline()
                        .small()
                        .label(l(lang, "Ver na Ficha para o FM27", "View in the FM27 sheet"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.tactic_tab = TacticTab::Fm27Sheet;
                            cx.notify();
                        })),
                )
                .child(fm_button(
                    app,
                    "sp-bridge",
                    l(lang, "Aplicar no FM", "Apply in FM"),
                    l(
                        lang,
                        "Aplica os marcadores e as rotinas de bolas paradas no FM. No FM, abra Táticas › Bolas paradas.",
                        "Applies the takers and set-piece routines in FM. In FM, open Tactics › Set Pieces.",
                    ),
                    false,
                    spawn_apply_set_pieces,
                    cx,
                )),
        );

    // Takers table
    let curve = l(lang, "curva ", "swing ");
    let corner_extra = |c: &ta_core::types::CornerTaker| match &c.swing {
        Some(s) => format!("{curve}{s}"),
        None => attrs_line(pl(&c.player_id), &[Attr::Cor, Attr::Cro], lang),
    };
    let mut takers = panel(l(lang, "Marcadores", "Takers"), &theme)
        .child(taker_row(
            l(lang, "Cantos do lado direito", "Corners (right side)").into(),
            Some((&r.corner_right.player_name, &r.corner_right.code)),
            corner_extra(&r.corner_right),
            &theme,
        ))
        .child(taker_row(
            l(lang, "Cantos do lado esquerdo", "Corners (left side)").into(),
            Some((&r.corner_left.player_name, &r.corner_left.code)),
            corner_extra(&r.corner_left),
            &theme,
        ))
        .child(taker_row(
            l(lang, "Livres directos", "Direct free kicks").into(),
            r.fk_direct.as_ref().map(|x| (x.player_name.as_str(), x.code.as_str())),
            match &r.fk_direct {
                Some(x) => attrs_line(pl(&x.player_id), &[Attr::Fre, Attr::Tec, Attr::Lon], lang),
                None => l(lang, "ninguém com Livres ≥ 14 — cruzar", "nobody with Free Kicks ≥ 14 — cross it").into(),
            },
            &theme,
        ))
        .child(taker_row(
            l(lang, "Livres para cruzar", "Crossed free kicks").into(),
            r.fk_cross.as_ref().map(|x| (x.player_name.as_str(), x.code.as_str())),
            r.fk_cross
                .as_ref()
                .map(|x| attrs_line(pl(&x.player_id), &[Attr::Fre, Attr::Cro], lang))
                .unwrap_or_default(),
            &theme,
        ));
    for (i, x) in r.pens.iter().enumerate() {
        takers = takers.child(taker_row(
            format!("{} — {}", l(lang, "Penáltis", "Penalties"), match lang {
                Lang::Pt => format!("{}.º", i + 1),
                _ => format!("#{}", i + 1),
            }),
            Some((&x.player_name, &x.code)),
            attrs_line(pl(&x.player_id), &[Attr::Pen, Attr::Cmp], lang),
            &theme,
        ));
    }
    takers = takers
        .child(taker_row(
            l(lang, "Lançamentos longos", "Long throws").into(),
            r.long_throw.as_ref().map(|x| (x.player_name.as_str(), x.code.as_str())),
            match &r.long_throw {
                Some(x) => attrs_line(pl(&x.player_id), &[Attr::Lth], lang),
                None => l(lang, "ninguém com Lançamentos Longos ≥ 14", "nobody with Long Throws ≥ 14").into(),
            },
            &theme,
        ))
        .child(taker_row(
            l(lang, "Capitão", "Captain").into(),
            r.caps.first().map(|x| (x.player_name.as_str(), x.code.as_str())),
            r.caps
                .first()
                .map(|x| attrs_line(pl(&x.player_id), &[Attr::Ldr, Attr::Det], lang))
                .unwrap_or_default(),
            &theme,
        ))
        .child(taker_row(
            l(lang, "Vice-capitão", "Vice-captain").into(),
            r.caps.get(1).map(|x| (x.player_name.as_str(), x.code.as_str())),
            r.caps
                .get(1)
                .map(|x| attrs_line(pl(&x.player_id), &[Attr::Ldr, Attr::Det], lang))
                .unwrap_or_default(),
            &theme,
        ))
        .child(div().text_xs().text_color(theme.muted_foreground).child(l(
            lang,
            "Sem dados do pé preferido o efeito do canto não é indicado. O plantel lido do save (fmsave) tem o pé.",
            "Without preferred-foot data the corner swing isn't shown. The squad read from the save (fmsave) has it.",
        )));

    // Tips
    let mut tips = panel(l(lang, "Ideias para os cantos e livres", "Ideas for corners and free kicks"), &theme);
    if r.tips.is_empty() {
        tips = tips.child(div().text_xs().text_color(theme.muted_foreground).child(l(
            lang,
            "Nada a destacar.",
            "Nothing to highlight.",
        )));
    }
    for (i, tp) in r.tips.iter().enumerate() {
        let def = tp.ti.and_then(ti_by_key);
        let on = match (def, tp.value) {
            (Some(d), Some(v)) => ti_val(&app.state.tactic, d.key) == v,
            _ => false,
        };
        let mut body = div()
            .v_flex()
            .flex_1()
            .gap_1()
            .child(div().text_xs().text_color(theme.foreground).child(tp.text.clone()));
        if let (Some(d), Some(v), false) = (def, tp.value, on) {
            let key = d.key;
            let label = format!(
                "{} → {}",
                ti_name(d, lang),
                ti_opt_name(d.opts.get(v).copied().unwrap_or(""), lang)
            );
            body = body.child(
                div().child(
                    Button::new(("sp-tip", i))
                        .outline()
                        .xsmall()
                        .label(label.clone())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(d) = ti_by_key(key) {
                                if v == d.def {
                                    this.state.tactic.ti.remove(key);
                                } else {
                                    this.state.tactic.ti.insert(key.to_string(), v);
                                }
                            }
                            sync_animated_pitch(this, cx);
                            this.state.persist();
                            this.state.set_toast(label.clone(), ToastKind::Success);
                            cx.notify();
                        })),
                ),
            );
        }
        tips = tips.child(
            div()
                .h_flex()
                .items_start()
                .gap_2()
                .py_1()
                .child(badge_pill(
                    if on { "✓".to_string() } else { (i + 1).to_string() },
                    if on { BadgeTone::Good } else { BadgeTone::Info },
                    cx,
                ))
                .child(body),
        );
    }

    // Routines
    let attack = panel(l(lang, "Canto ofensivo", "Attacking corner"), &theme)
        .child(div().text_xs().text_color(theme.muted_foreground).child(l(
            lang,
            "Canto do lado esquerdo; do outro lado é o espelho.",
            "Corner from the left; the other side is the mirror image.",
        )))
        .child(routine_table(&r.attack, &theme));
    let defend = panel(
        format!("{} — {}", l(lang, "Canto defensivo", "Defending corner"), r.system),
        &theme,
    )
    .child(div().text_xs().text_color(theme.muted_foreground).child(l(
        lang,
        "Posições sem bola no canto contra.",
        "Off-ball positions when defending a corner.",
    )))
    .child(routine_table(&r.defend, &theme));

    div()
        .v_flex()
        .gap_4()
        .child(header)
        .child(div().h_flex().flex_wrap().items_start().gap_4().child(takers).child(tips))
        .child(setpieces_diagrams_pair(&r, lang, cx))
        .child(div().h_flex().flex_wrap().items_start().gap_4().child(attack).child(defend))
        .into_any_element()
}
