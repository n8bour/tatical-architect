//! Attacking & defending half-pitch corner diagrams rendered via GPUI `canvas()`.
//!
//! Coordinates mirror `SP_POS` in `src/ui/views_setpieces.js` (top-down penalty box:
//! x in 0..68 pitch width, y = 0 at goal line).

use crate::widgets::common::{
    paint_arc_stroke, paint_centered_text, paint_circle_fill, paint_rect_stroke, rgba_f32,
};
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use ta_core::types::{Lang, SetPiecePlan, SetPieceRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPieceDiagramKind {
    Attack,
    Defend,
}

fn slot_coords(kind: SetPieceDiagramKind, key: &str) -> Option<(f32, f32)> {
    match kind {
        SetPieceDiagramKind::Attack => match key {
            "taker" => Some((2.0, 1.5)),
            "near" => Some((26.0, 4.5)),
            "centre" => Some((34.0, 6.5)),
            "far" => Some((43.0, 5.0)),
            "spot" => Some((34.0, 13.5)),
            "edge" => Some((34.0, 21.5)),
            "short" => Some((9.0, 9.0)),
            "back0" => Some((20.0, 35.0)),
            "back1" => Some((48.0, 35.0)),
            "back2" => Some((34.0, 37.5)),
            _ => None,
        },
        SetPieceDiagramKind::Defend => match key {
            "zn" => Some((26.5, 3.2)),
            "zc" => Some((34.0, 7.0)),
            "zf" => Some((41.5, 3.2)),
            "m0" => Some((27.5, 12.0)),
            "m1" => Some((40.5, 12.0)),
            "m2" => Some((34.0, 16.0)),
            "m3" => Some((21.0, 14.0)),
            "de" => Some((34.0, 23.0)),
            "o0" => Some((25.0, 36.0)),
            "o1" => Some((43.0, 36.0)),
            _ => None,
        },
    }
}

fn short_name(full: &str) -> String {
    let parts: Vec<&str> = full.split_whitespace().collect();
    let s = if parts.len() >= 2 {
        parts[parts.len() - 1]
    } else {
        full.trim()
    };
    s.chars().take(11).collect()
}

/// Renders one attacking or defending corner half-pitch diagram via GPUI `canvas()`.
pub fn setpiece_half_pitch(
    rows: &[SetPieceRole],
    kind: SetPieceDiagramKind,
    caption: impl Into<SharedString>,
    show_gk: bool,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let caption = caption.into();

    let dots: Vec<(f32, f32, Hsla, String, String)> = rows
        .iter()
        .filter_map(|r| {
            let (x, y) = slot_coords(kind, &r.key)?;
            let col: Hsla = if r.key.starts_with("back") || r.key.starts_with('o') {
                rgb(0x2d5f8f).into()
            } else if r.key == "taker" {
                rgb(0xb7791f).into()
            } else {
                rgb(0x0b5d3a).into()
            };
            let code = r
                .code
                .trim_end_matches(['L', 'R'])
                .chars()
                .take(3)
                .collect::<String>();
            let name = short_name(&r.player_name);
            Some((x, y, col, code, name))
        })
        .collect();

    div()
        .v_flex()
        .gap_1()
        .child(
            div()
                .w_full()
                .h(px(230.0))
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
                            // Virtual viewBox: x in 0..68, y in -1..41 (height 42)
                            let sx = bw / 68.0;
                            let sy = bh / 42.0;
                            let scale = sx.min(sy);
                            let pt = |vx: f32, vy: f32| {
                                point(px(ox + vx * sx), px(oy + (vy + 1.0) * sy))
                            };

                            window.paint_quad(fill(bounds, rgb(0x285532)));
                            let line_col = rgba_f32(1.0, 1.0, 1.0, 0.62);

                            // Goal line
                            let mut gl = PathBuilder::stroke(px(1.2));
                            gl.move_to(pt(0.0, 0.0));
                            gl.line_to(pt(68.0, 0.0));
                            if let Ok(p) = gl.build() {
                                window.paint_path(p, line_col);
                            }

                            // Penalty box & 6-yard box
                            paint_rect_stroke(
                                window,
                                pt(13.85, 0.0),
                                40.3 * sx,
                                16.5 * sy,
                                1.2,
                                line_col,
                                None,
                            );
                            paint_rect_stroke(
                                window,
                                pt(24.85, 0.0),
                                18.3 * sx,
                                5.5 * sy,
                                1.2,
                                line_col,
                                None,
                            );

                            // Penalty spot & D-arc
                            paint_circle_fill(window, pt(34.0, 11.0), 2.2, line_col, 12);
                            paint_arc_stroke(
                                window,
                                pt(34.0, 11.0),
                                9.15 * scale,
                                0.64,
                                PI - 0.64,
                                1.2,
                                line_col,
                                None,
                                24,
                            );

                            // Corner arc (top-left)
                            paint_arc_stroke(
                                window,
                                pt(0.0, 0.0),
                                2.0 * scale,
                                0.0,
                                FRAC_PI_2,
                                1.2,
                                line_col,
                                None,
                                12,
                            );

                            // Goal bar
                            window.paint_quad(fill(
                                Bounds::new(pt(30.34, -1.0), size(px(7.32 * sx), px(1.0 * sy))),
                                rgba_f32(1.0, 1.0, 1.0, 0.85),
                            ));

                            // Optional defending GK
                            if show_gk {
                                let gk_c = pt(34.0, 1.6);
                                let gk_r = (1.8 * scale).clamp(7.0, 11.0);
                                paint_circle_fill(window, gk_c, gk_r, rgb(0x3c3f46).into(), 18);
                                paint_arc_stroke(
                                    window,
                                    gk_c,
                                    gk_r,
                                    0.0,
                                    TAU,
                                    1.1,
                                    rgb(0xffffff).into(),
                                    None,
                                    18,
                                );
                                paint_centered_text(
                                    window,
                                    cx,
                                    "GK",
                                    gk_c,
                                    (gk_r * 0.8).max(7.5),
                                    rgb(0xffffff).into(),
                                    true,
                                );
                            }

                            // Player tokens
                            let r = (2.1 * scale).clamp(8.5, 13.0);
                            for (x, y, col, code, name) in &dots {
                                let c = pt(*x, *y);
                                paint_circle_fill(window, c, r, *col, 20);
                                paint_arc_stroke(
                                    window,
                                    c,
                                    r,
                                    0.0,
                                    TAU,
                                    1.2,
                                    rgb(0xffffff).into(),
                                    None,
                                    20,
                                );
                                paint_centered_text(
                                    window,
                                    cx,
                                    code,
                                    c,
                                    (r * 0.72).max(7.5),
                                    rgb(0xffffff).into(),
                                    true,
                                );
                                if !name.is_empty() {
                                    // Subtle dark backdrop + crisp label below token
                                    let label_pt = point(c.x, c.y + px(r + 7.5));
                                    paint_centered_text(
                                        window,
                                        cx,
                                        name,
                                        point(label_pt.x + px(0.8), label_pt.y + px(0.8)),
                                        9.5,
                                        rgba_f32(0.0, 0.0, 0.0, 0.8),
                                        false,
                                    );
                                    paint_centered_text(
                                        window,
                                        cx,
                                        name,
                                        label_pt,
                                        9.5,
                                        rgb(0xffffff).into(),
                                        false,
                                    );
                                }
                            }
                        },
                    )
                    .size_full(),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(caption),
        )
}

/// Side-by-side attacking and defending corner diagrams for a `SetPiecePlan`.
pub fn setpieces_diagrams_pair(plan: &SetPiecePlan, lang: Lang, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
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
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(lang.pick("Canto ofensivo", "Attacking corner")),
                )
                .child(setpiece_half_pitch(
                    &plan.attack,
                    SetPieceDiagramKind::Attack,
                    lang.pick(
                        "Canto do lado esquerdo; do outro lado é o espelho.",
                        "Left-side corner; right side is mirrored.",
                    ),
                    false,
                    cx,
                )),
        )
        .child(
            div()
                .v_flex()
                .flex_1()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(format!(
                            "{} — {}",
                            lang.pick("Canto defensivo", "Defending corner"),
                            plan.system
                        )),
                )
                .child(setpiece_half_pitch(
                    &plan.defend,
                    SetPieceDiagramKind::Defend,
                    lang.pick(
                        "Posições sem bola no canto contra.",
                        "Out-of-possession positions on corners against.",
                    ),
                    true,
                    cx,
                )),
        )
}
