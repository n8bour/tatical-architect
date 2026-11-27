//! Dual horizontal GPUI `canvas()` synergy arrow maps (IP & OOP) rendering
//! `ta_core::types::SynergyLink`s with color-coded lines, arrowheads, and role nodes.

use crate::widgets::common::{
    paint_arc_stroke, paint_arrow, paint_centered_text, paint_circle_fill, paint_rect_stroke,
    rgba_f32,
};
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::*;
use std::collections::HashMap;
use std::f32::consts::TAU;
use ta_core::data::{formation_slots, role_by_id};
use ta_core::types::{Lang, SynergyLink, SynergyLinkType, SynergyPhase, SynergyReport, Tactic};

/// Returns the canonical visual color for each synergy link type.
pub fn synergy_link_color(link_type: SynergyLinkType) -> Hsla {
    match link_type {
        SynergyLinkType::Build => rgb(0x14b8a6).into(),
        SynergyLinkType::Chain => rgb(0x22c55e).into(),
        SynergyLinkType::Cross => rgb(0x3b82f6).into(),
        SynergyLinkType::Front => rgb(0xd97706).into(),
        SynergyLinkType::Flank => rgb(0xa855f7).into(),
        SynergyLinkType::Press => rgb(0xef4444).into(),
        SynergyLinkType::FlankDef => rgb(0xa855f7).into(),
        SynergyLinkType::Counter => rgb(0xf59e0b).into(),
    }
}

/// Renders one horizontal synergy arrow map (`attack → right`) onto a GPUI `canvas()`.
pub fn synergy_map_canvas(
    tactic: &Tactic,
    links: &[SynergyLink],
    formation_name: &str,
    is_oop: bool,
) -> impl IntoElement {
    let slots = formation_slots(formation_name).map_or(&[][..], |v| v.as_slice());
    let pos_map: HashMap<String, (f32, f32)> = slots
        .iter()
        .map(|s| (s.code.to_string(), (s.x, s.y)))
        .collect();

    // Virtual 100x64 coordinate mapping identical to `src/ui/views_synergy.js`.
    let node_xy = |code: &str| -> (f32, f32) {
        pos_map
            .get(code)
            .map(|&(x, y)| (6.0 + y * 0.88, 4.0 + x * 0.56))
            .unwrap_or((50.0, 32.0))
    };

    let arrow_data: Vec<((f32, f32), (f32, f32), Hsla, f32, bool)> = links
        .iter()
        .map(|l| {
            let (x1, y1) = node_xy(&l.from_code);
            let (x2, y2) = node_xy(&l.to_code);
            let dx = x2 - x1;
            let dy = y2 - y1;
            let d = dx.hypot(dy).max(1.0);
            let k = 3.2 / d;
            let dashed = matches!(l.link_type, SynergyLinkType::Flank | SynergyLinkType::FlankDef);
            (
                (x1 + dx * k, y1 + dy * k),
                (x2 - dx * k, y2 - dy * k),
                synergy_link_color(l.link_type),
                0.7 + l.value.clamp(0.0, 1.0) * 1.1,
                dashed,
            )
        })
        .collect();

    let node_data: Vec<(f32, f32, String)> = tactic
        .slots
        .iter()
        .map(|s| {
            let code = if is_oop { &s.oop_code } else { &s.ip_code };
            let role_id = if is_oop { &s.oop } else { &s.ip };
            let ab = role_by_id(role_id).map(|r| r.ab).unwrap_or("?").to_string();
            let (x, y) = node_xy(code);
            (x, y, ab)
        })
        .collect();

    div()
        .w_full()
        .h(px(210.0))
        .rounded_md()
        .overflow_hidden()
        .child(
            canvas(
                move |_, _, _| {},
                move |bounds, _, window, cx| {
                    let ox: f32 = bounds.origin.x.into();
                    let oy: f32 = bounds.origin.y.into();
                    let bw: f32 = bounds.size.width.into();
                    let bh: f32 = bounds.size.height.into();
                    let sx = bw / 100.0;
                    let sy = bh / 64.0;
                    let scale = sx.min(sy);
                    let pt = |vx: f32, vy: f32| point(px(ox + vx * sx), px(oy + vy * sy));

                    // Pitch surface
                    window.paint_quad(fill(bounds, rgb(0x285532)));
                    let line_col = rgba_f32(1.0, 1.0, 1.0, 0.55);

                    paint_rect_stroke(window, pt(2.0, 2.0), 96.0 * sx, 60.0 * sy, 1.1, line_col, None);
                    let mut mid = PathBuilder::stroke(px(1.1));
                    mid.move_to(pt(50.0, 2.0));
                    mid.line_to(pt(50.0, 62.0));
                    if let Ok(p) = mid.build() {
                        window.paint_path(p, line_col);
                    }
                    paint_arc_stroke(window, pt(50.0, 32.0), 7.0 * scale, 0.0, TAU, 1.1, line_col, None, 28);
                    paint_rect_stroke(window, pt(2.0, 17.0), 15.0 * sx, 30.0 * sy, 1.1, line_col, None);
                    paint_rect_stroke(window, pt(83.0, 17.0), 15.0 * sx, 30.0 * sy, 1.1, line_col, None);

                    // Synergy arrows
                    let dash = [px(5.0), px(3.0)];
                    for ((x1, y1), (x2, y2), col, w, dashed) in &arrow_data {
                        paint_arrow(
                            window,
                            pt(*x1, *y1),
                            pt(*x2, *y2),
                            *col,
                            (*w * (scale * 0.45)).max(1.5),
                            if *dashed { Some(&dash) } else { None },
                            0.0,
                        );
                    }

                    // Role nodes
                    let node_fill: Hsla = if is_oop {
                        rgb(0x1e3a5f).into()
                    } else {
                        rgb(0x0b5d3a).into()
                    };
                    let r = (3.2 * scale).clamp(8.5, 14.0);
                    for (x, y, ab) in &node_data {
                        let c = pt(*x, *y);
                        paint_circle_fill(window, c, r, node_fill, 20);
                        paint_arc_stroke(
                            window,
                            c,
                            r,
                            0.0,
                            TAU,
                            1.2,
                            rgba_f32(1.0, 1.0, 1.0, 0.92),
                            None,
                            20,
                        );
                        paint_centered_text(window, cx, ab, c, (r * 0.78).max(8.0), rgb(0xffffff).into(), true);
                    }
                },
            )
            .size_full(),
        )
}

fn legend_row(items: &[(SynergyLinkType, &str)], cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let mut row = div().h_flex().flex_wrap().gap_3();
    for &(lt, label) in items {
        row = row.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .w(px(10.0))
                        .h(px(4.0))
                        .rounded_full()
                        .bg(synergy_link_color(lt)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(label.to_string()),
                ),
        );
    }
    row
}

/// Side-by-side IP & OOP synergy maps with color legends.
pub fn dual_synergy_maps(tactic: &Tactic, report: &SynergyReport, lang: Lang, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let ip_links: Vec<SynergyLink> = report
        .links
        .iter()
        .filter(|l| l.phase != SynergyPhase::Def)
        .cloned()
        .collect();
    let oop_links: Vec<SynergyLink> = report
        .links
        .iter()
        .filter(|l| l.phase == SynergyPhase::Def)
        .cloned()
        .collect();

    let ip_legend = [
        (SynergyLinkType::Build, lang.pick("saída de bola", "build-up")),
        (SynergyLinkType::Chain, lang.pick("criador → finalizador", "creator → finisher")),
        (SynergyLinkType::Cross, lang.pick("cruzamento → alvo", "cross → target")),
        (SynergyLinkType::Front, lang.pick("dupla da frente", "front pair")),
        (SynergyLinkType::Flank, lang.pick("flanco", "flank")),
        (SynergyLinkType::Counter, lang.pick("contra-ataque", "counter")),
    ];
    let oop_legend = [
        (SynergyLinkType::Press, lang.pick("pressiona → cobre", "presses → covers")),
        (SynergyLinkType::FlankDef, lang.pick("flanco: dobra / segura", "flank: doubles / holds")),
    ];

    div()
        .h_flex()
        .w_full()
        .gap_4()
        .child(
            div()
                .v_flex()
                .flex_1()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!("{} · {}", lang.pick("Com bola", "In possession"), tactic.ip_form)),
                )
                .child(synergy_map_canvas(tactic, &ip_links, &tactic.ip_form, false))
                .child(legend_row(&ip_legend, cx)),
        )
        .child(
            div()
                .v_flex()
                .flex_1()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!("{} · {}", lang.pick("Sem bola", "Out of possession"), tactic.oop_form)),
                )
                .child(synergy_map_canvas(tactic, &oop_links, &tactic.oop_form, true))
                .child(legend_row(&oop_legend, cx)),
        )
}
