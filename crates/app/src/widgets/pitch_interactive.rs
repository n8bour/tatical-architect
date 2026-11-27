//! Interactive vertical pitch widget (`canvas()` + positioned clickable slot buttons)
//! for Tactic › Analyse (IP / OOP / BOTH toggle, role code, player surname, score badge).

use crate::widgets::common::{
    paint_arc_stroke, paint_arrow, paint_circle_fill, paint_rect_stroke, rgba_f32, score_color,
    RADIUS,
};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::rc::Rc;
use ta_core::data::{formation_slots, role_by_id};
use ta_core::types::{Lang, Player, Tactic};

/// Display phase mode on the interactive tactical pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PitchPhaseMode {
    #[default]
    Ip,
    Oop,
    Both,
}

impl PitchPhaseMode {
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Ip => lang.pick("Com bola (IP)", "In Possession (IP)"),
            Self::Oop => lang.pick("Sem bola (OOP)", "Out of Possession (OOP)"),
            Self::Both => lang.pick("IP + OOP", "IP + OOP"),
        }
    }
}

fn short_surname(full_name: &str) -> String {
    let trimmed = full_name.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let s = if parts.len() >= 2 {
        parts[parts.len() - 1]
    } else {
        parts[0]
    };
    s.chars().take(11).collect()
}

/// Renders the interactive vertical pitch with phase toggle buttons and 11 clickable slots.
#[allow(clippy::too_many_arguments)]
pub fn interactive_pitch(
    tactic: &Tactic,
    squad: &[Player],
    slot_scores: Option<&[u8]>,
    mode: PitchPhaseMode,
    selected_slot: Option<usize>,
    lang: Lang,
    on_mode: impl Fn(&PitchPhaseMode, &mut Window, &mut App) + 'static,
    on_select: impl Fn(&usize, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let on_mode = Rc::new(on_mode);
    let on_select = Rc::new(on_select);

    let ip_form_slots = formation_slots(&tactic.ip_form).map_or(&[][..], |v| v.as_slice());
    let oop_form_slots = formation_slots(&tactic.oop_form)
        .map(|v| v.as_slice())
        .unwrap_or(ip_form_slots);

    let ip_map: HashMap<&str, (f32, f32)> = ip_form_slots
        .iter()
        .map(|s| (s.code, (s.x, s.y)))
        .collect();
    let oop_map: HashMap<&str, (f32, f32)> = oop_form_slots
        .iter()
        .map(|s| (s.code, (s.x, s.y)))
        .collect();
    let player_map: HashMap<&str, &Player> = squad.iter().map(|p| (p.id.as_str(), p)).collect();

    // Precompute IP -> OOP shift arrows for `PitchPhaseMode::Both`.
    let shift_arrows: Vec<((f32, f32), (f32, f32))> = if mode == PitchPhaseMode::Both {
        tactic
            .slots
            .iter()
            .filter_map(|s| {
                let a = *ip_map.get(s.ip_code.as_str())?;
                let b = *oop_map.get(s.oop_code.as_str())?;
                if (a.0 - b.0).hypot(a.1 - b.1) > 3.0 {
                    Some((a, b))
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let pitch_w = 340.0_f32;
    let pitch_h = 480.0_f32;

    // Map FM normalized coords (x: 0..100 left-to-right, y: 0..100 own-goal-to-opp-goal)
    // to vertical pitch pixels (own goal at bottom, attack toward top).
    let to_norm_px = |x: f32, y: f32| -> (f32, f32) {
        let px_x = (0.06 + (x / 100.0) * 0.88) * pitch_w;
        let px_y = (0.94 - (y / 100.0) * 0.88) * pitch_h;
        (px_x, px_y)
    };

    let mut slot_overlay = div().relative().w(px(pitch_w)).h(px(pitch_h)).child(
        canvas(
            move |_, _, _| {},
            move |bounds, _, window, _cx| {
                let ox: f32 = bounds.origin.x.into();
                let oy: f32 = bounds.origin.y.into();
                let bw: f32 = bounds.size.width.into();
                let bh: f32 = bounds.size.height.into();

                let grass1: Hsla = rgb(0x2c5a36).into();
                let grass2: Hsla = rgb(0x275131).into();
                let line_col = rgba_f32(1.0, 1.0, 1.0, 0.6);

                // Base grass + horizontal stripes
                window.paint_quad(fill(bounds, grass1));
                let stripe_h = bh / 10.0;
                for i in (1..10).step_by(2) {
                    window.paint_quad(fill(
                        Bounds::new(
                            point(px(ox), px(oy + stripe_h * i as f32)),
                            size(px(bw), px(stripe_h)),
                        ),
                        grass2,
                    ));
                }

                // Pitch markings
                let pad_x = bw * 0.04;
                let pad_y = bh * 0.03;
                let fw = bw - pad_x * 2.0;
                let fh = bh - pad_y * 2.0;

                paint_rect_stroke(
                    window,
                    point(px(ox + pad_x), px(oy + pad_y)),
                    fw,
                    fh,
                    1.4,
                    line_col,
                    None,
                );

                // Halfway line
                let mut mid = PathBuilder::stroke(px(1.3));
                mid.move_to(point(px(ox + pad_x), px(oy + pad_y + fh * 0.5)));
                mid.line_to(point(px(ox + pad_x + fw), px(oy + pad_y + fh * 0.5)));
                if let Ok(p) = mid.build() {
                    window.paint_path(p, line_col);
                }

                // Center circle & spot
                let center = point(px(ox + bw * 0.5), px(oy + bh * 0.5));
                paint_arc_stroke(window, center, fw * 0.145, 0.0, TAU, 1.3, line_col, None, 36);
                paint_circle_fill(window, center, 2.6, line_col, 14);

                // Top & bottom penalty boxes + 6-yard boxes
                let pen_w = fw * 0.58;
                let pen_h = fh * 0.155;
                let six_w = fw * 0.27;
                let six_h = fh * 0.055;
                let pen_x = ox + (bw - pen_w) * 0.5;
                let six_x = ox + (bw - six_w) * 0.5;

                // Top box (attacking goal)
                paint_rect_stroke(
                    window,
                    point(px(pen_x), px(oy + pad_y)),
                    pen_w,
                    pen_h,
                    1.3,
                    line_col,
                    None,
                );
                paint_rect_stroke(
                    window,
                    point(px(six_x), px(oy + pad_y)),
                    six_w,
                    six_h,
                    1.3,
                    line_col,
                    None,
                );
                paint_arc_stroke(
                    window,
                    point(px(ox + bw * 0.5), px(oy + pad_y + fh * 0.11)),
                    fw * 0.125,
                    0.52,
                    PI - 0.52,
                    1.2,
                    line_col,
                    None,
                    20,
                );

                // Bottom box (own goal)
                paint_rect_stroke(
                    window,
                    point(px(pen_x), px(oy + pad_y + fh - pen_h)),
                    pen_w,
                    pen_h,
                    1.3,
                    line_col,
                    None,
                );
                paint_rect_stroke(
                    window,
                    point(px(six_x), px(oy + pad_y + fh - six_h)),
                    six_w,
                    six_h,
                    1.3,
                    line_col,
                    None,
                );
                paint_arc_stroke(
                    window,
                    point(px(ox + bw * 0.5), px(oy + pad_y + fh * 0.89)),
                    fw * 0.125,
                    PI + 0.52,
                    TAU - 0.52,
                    1.2,
                    line_col,
                    None,
                    20,
                );

                // IP -> OOP shift arrows in BOTH mode
                let dash = [px(4.0), px(3.0)];
                for ((ax, ay), (bx, by)) in &shift_arrows {
                    let p1 = point(
                        px(ox + (0.06 + (ax / 100.0) * 0.88) * bw),
                        px(oy + (0.94 - (ay / 100.0) * 0.88) * bh),
                    );
                    let p2 = point(
                        px(ox + (0.06 + (bx / 100.0) * 0.88) * bw),
                        px(oy + (0.94 - (by / 100.0) * 0.88) * bh),
                    );
                    paint_arrow(
                        window,
                        p1,
                        p2,
                        rgba_f32(0.49, 0.83, 0.99, 0.85),
                        1.8,
                        Some(&dash),
                        0.0,
                    );
                }
            },
        )
        .size_full()
        .rounded(px(RADIUS))
        .overflow_hidden(),
    );

    // Layer clickable slot tokens over the canvas
    for (idx, slot) in tactic.slots.iter().enumerate() {
        let (x, y) = match mode {
            PitchPhaseMode::Oop => oop_map
                .get(slot.oop_code.as_str())
                .or_else(|| ip_map.get(slot.ip_code.as_str()))
                .copied()
                .unwrap_or((50.0, 50.0)),
            PitchPhaseMode::Ip | PitchPhaseMode::Both => ip_map
                .get(slot.ip_code.as_str())
                .copied()
                .unwrap_or((50.0, 50.0)),
        };
        let (cx_px, cy_px) = to_norm_px(x, y);
        let is_selected = selected_slot == Some(idx);

        let ip_ab = role_by_id(&slot.ip).map(|r| r.ab).unwrap_or(&slot.ip);
        let oop_ab = role_by_id(&slot.oop).map(|r| r.ab).unwrap_or(&slot.oop);
        let role_badge = match mode {
            PitchPhaseMode::Ip => ip_ab.to_string(),
            PitchPhaseMode::Oop => oop_ab.to_string(),
            PitchPhaseMode::Both => format!("{}/{}", ip_ab, oop_ab),
        };
        let pos_code = match mode {
            PitchPhaseMode::Oop => slot.oop_code.as_str(),
            PitchPhaseMode::Ip | PitchPhaseMode::Both => slot.ip_code.as_str(),
        };
        let surname = slot
            .pid
            .as_deref()
            .and_then(|id| player_map.get(id))
            .map(|p| short_surname(&p.name))
            .unwrap_or_default();
        let score = slot_scores.and_then(|s| s.get(idx).copied());

        // Disc like the browser build: fit colour when a player is assigned, dark otherwise.
        let token_bg: Hsla = match score {
            Some(sc) if !surname.is_empty() => score_color(sc),
            _ if mode == PitchPhaseMode::Oop => rgb(0x3d4a5a).into(),
            _ => rgba_f32(0.0, 0.0, 0.0, 0.42),
        };
        let border_col: Hsla = if is_selected {
            rgb(0xffe680).into()
        } else {
            rgba_f32(1.0, 1.0, 1.0, 0.85)
        };

        let on_sel = on_select.clone();
        let token_w = 72.0_f32;
        let token_h = 50.0_f32;
        let disc = if is_selected { 36.0 } else { 32.0 };

        slot_overlay = slot_overlay.child(
            div()
                .id(("pitch-slot", idx))
                .absolute()
                .left(px((cx_px - token_w * 0.5).clamp(2.0, pitch_w - token_w - 2.0)))
                .top(px((cy_px - token_h * 0.5).clamp(2.0, pitch_h - token_h - 2.0)))
                .w(px(token_w))
                .cursor_pointer()
                .v_flex()
                .items_center()
                .gap(px(2.0))
                .on_click(move |_, window, cx| on_sel(&idx, window, cx))
                .child(
                    div()
                        .relative()
                        .size(px(disc))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(token_bg)
                        .border_color(border_col)
                        .when(is_selected, |d| d.border_2())
                        .when(!is_selected, |d| d.border_1())
                        .shadow_sm()
                        .child(
                            div()
                                .text_size(px(if role_badge.len() > 4 { 9.0 } else { 11.0 }))
                                .font_bold()
                                .text_color(rgb(0xffffff))
                                .child(role_badge),
                        )
                        .when_some(score.filter(|_| !surname.is_empty()), |el, sc| {
                            el.child(
                                div()
                                    .absolute()
                                    .top(px(-6.0))
                                    .left(px(disc - 2.0))
                                    .text_size(px(10.0))
                                    .font_semibold()
                                    .text_color(rgb(0xffffff))
                                    .child(sc.to_string()),
                            )
                        }),
                )
                .child(
                    div()
                        .max_w(px(token_w))
                        .px(px(3.0))
                        .rounded_sm()
                        .bg(rgba_f32(0.0, 0.0, 0.0, 0.35))
                        .text_size(px(11.0))
                        .font_semibold()
                        .text_color(rgb(0xffffff))
                        .truncate()
                        .child(if surname.is_empty() {
                            pos_code.to_string()
                        } else {
                            surname
                        }),
                ),
        );
    }

    let mut phase_bar = div().h_flex().items_center().gap_1();
    for m in [PitchPhaseMode::Ip, PitchPhaseMode::Oop, PitchPhaseMode::Both] {
        let on_m = on_mode.clone();
        let active = m == mode;
        phase_bar = phase_bar.child(
            Button::new(("pitch-phase", m as usize))
                .xsmall()
                .label(m.label(lang))
                .when(active, |b| b.primary())
                .when(!active, |b| b.outline())
                .on_click(move |_, window, cx| on_m(&m, window, cx)),
        );
    }

    div()
        .v_flex()
        .items_center()
        .gap_2()
        .child(phase_bar)
        .child(slot_overlay)
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("IP: {} · OOP: {}", tactic.ip_form, tactic.oop_form)),
        )
}
