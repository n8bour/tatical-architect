//! Shared GPUI widgets, score ring `canvas()`, attribute chips, collapsible cards,
//! KPI cards, busy progress indicators, toast banner, and 2D vector canvas helpers.

use crate::state::{BusyState, ToastKind, ToastState};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::f32::consts::{FRAC_PI_2, TAU};

/// Semantic tone for pills and badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    Good,
    Warn,
    Crit,
    Info,
    Muted,
}

/// Maps a 0–100 tactical/suitability score to a clear semantic color.
pub fn score_color(score: u8) -> Hsla {
    if score >= 75 {
        rgb(0x16a34a).into()
    } else if score >= 55 {
        rgb(0xd97706).into()
    } else {
        rgb(0xdc2626).into()
    }
}

/// Maps an FM 1–20 attribute value to a classic FM color tier.
pub fn attr_color(val: f32) -> Hsla {
    if val >= 16.0 {
        rgb(0x10b981).into()
    } else if val >= 13.0 {
        rgb(0x22c55e).into()
    } else if val >= 10.0 {
        rgb(0xf59e0b).into()
    } else {
        rgb(0xef4444).into()
    }
}

/// Parses a `#rgb`, `#rrggbb`, or CSS named/rgba hint into `Hsla`.
pub fn parse_css_color(s: &str) -> Hsla {
    let t = s.trim();
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() == 6 {
            if let Ok(v) = u32::from_str_radix(hex, 16) {
                return rgb(v).into();
            }
        } else if hex.len() == 3 {
            let cs: Vec<char> = hex.chars().collect();
            let expanded = format!(
                "{0}{0}{1}{1}{2}{2}",
                cs[0], cs[1], cs[2]
            );
            if let Ok(v) = u32::from_str_radix(&expanded, 16) {
                return rgb(v).into();
            }
        }
    }
    if let Some(inner) = t.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<f32>().unwrap_or(255.0) / 255.0;
            let g = parts[1].trim().parse::<f32>().unwrap_or(255.0) / 255.0;
            let b = parts[2].trim().parse::<f32>().unwrap_or(255.0) / 255.0;
            let a = parts[3].trim().parse::<f32>().unwrap_or(1.0);
            return rgba_f32(r, g, b, a);
        }
    }
    rgb(0x94a3b8).into()
}

/// Converts normalized `r, g, b, a` (`0.0..=1.0`) into `Hsla`.
pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Hsla {
    let r8 = (r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g8 = (g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b8 = (b.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a8 = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
    rgba((r8 << 24) | (g8 << 16) | (b8 << 8) | a8).into()
}

/// Returns a copy of `color` with alpha scaled by `factor`.
pub fn with_alpha(mut color: Hsla, alpha: f32) -> Hsla {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

// ============================================================================
// Low-Level GPUI Canvas 2D Drawing Helpers
// ============================================================================

/// Paints a filled circle (approximated with a smooth polygon) into `window`.
pub fn paint_circle_fill(
    window: &mut Window,
    center: Point<Pixels>,
    radius: f32,
    color: Hsla,
    segments: usize,
) {
    if radius <= 0.1 {
        return;
    }
    let n = segments.max(12);
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let a = (i as f32 / n as f32) * TAU;
        pts.push(point(
            center.x + px(a.cos() * radius),
            center.y + px(a.sin() * radius),
        ));
    }
    let mut pb = PathBuilder::fill();
    pb.add_polygon(&pts, true);
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paints a filled ellipse (used for player & ball ground shadows).
pub fn paint_ellipse_fill(
    window: &mut Window,
    center: Point<Pixels>,
    rx: f32,
    ry: f32,
    color: Hsla,
    segments: usize,
) {
    if rx <= 0.1 || ry <= 0.1 {
        return;
    }
    let n = segments.max(12);
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let a = (i as f32 / n as f32) * TAU;
        pts.push(point(
            center.x + px(a.cos() * rx),
            center.y + px(a.sin() * ry),
        ));
    }
    let mut pb = PathBuilder::fill();
    pb.add_polygon(&pts, true);
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paints a stroked circular arc (`start_rad..=end_rad`, clockwise in screen coords).
pub fn paint_arc_stroke(
    window: &mut Window,
    center: Point<Pixels>,
    radius: f32,
    start_rad: f32,
    end_rad: f32,
    width: f32,
    color: Hsla,
    dash: Option<&[Pixels]>,
    segments: usize,
) {
    if radius <= 0.1 || width <= 0.05 {
        return;
    }
    let steps = segments.max(8);
    let mut pb = PathBuilder::stroke(px(width));
    if let Some(d) = dash {
        pb = pb.dash_array(d);
    }
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = start_rad + (end_rad - start_rad) * t;
        let p = point(
            center.x + px(a.cos() * radius),
            center.y + px(a.sin() * radius),
        );
        if i == 0 {
            pb.move_to(p);
        } else {
            pb.line_to(p);
        }
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paints a stroked rectangle outline.
pub fn paint_rect_stroke(
    window: &mut Window,
    origin: Point<Pixels>,
    w: f32,
    h: f32,
    stroke_w: f32,
    color: Hsla,
    dash: Option<&[Pixels]>,
) {
    let mut pb = PathBuilder::stroke(px(stroke_w));
    if let Some(d) = dash {
        pb = pb.dash_array(d);
    }
    pb.move_to(origin);
    pb.line_to(point(origin.x + px(w), origin.y));
    pb.line_to(point(origin.x + px(w), origin.y + px(h)));
    pb.line_to(point(origin.x, origin.y + px(h)));
    pb.close();
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }
}

/// Paints a straight or quadratic-curved arrow with a solid triangular arrowhead.
pub fn paint_arrow(
    window: &mut Window,
    a: Point<Pixels>,
    b: Point<Pixels>,
    color: Hsla,
    width: f32,
    dash: Option<&[Pixels]>,
    bend_px: f32,
) {
    let x1: f32 = a.x.into();
    let y1: f32 = a.y.into();
    let x2: f32 = b.x.into();
    let y2: f32 = b.y.into();
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = dx.hypot(dy);
    if len < 3.0 {
        return;
    }

    let mut cx = (x1 + x2) * 0.5;
    let mut cy = (y1 + y2) * 0.5;
    if bend_px.abs() > 0.05 {
        cx -= (dy / len) * bend_px;
        cy += (dx / len) * bend_px;
    }

    let mut pb = PathBuilder::stroke(px(width));
    if let Some(d) = dash {
        pb = pb.dash_array(d);
    }
    pb.move_to(a);
    if bend_px.abs() > 0.05 {
        pb.curve_to(b, point(px(cx), px(cy)));
    } else {
        pb.line_to(b);
    }
    if let Ok(path) = pb.build() {
        window.paint_path(path, color);
    }

    // Arrowhead
    let ang = if bend_px.abs() > 0.05 {
        (y2 - cy).atan2(x2 - cx)
    } else {
        dy.atan2(dx)
    };
    let hs = (width * 3.1).max(6.0);
    let p_left = point(
        px(x2 - hs * (ang - 0.48).cos()),
        px(y2 - hs * (ang - 0.48).sin()),
    );
    let p_right = point(
        px(x2 - hs * (ang + 0.48).cos()),
        px(y2 - hs * (ang + 0.48).sin()),
    );
    let mut head = PathBuilder::fill();
    head.add_polygon(&[b, p_left, p_right], true);
    if let Ok(path) = head.build() {
        window.paint_path(path, color);
    }
}

/// Paints a single-line centered text label directly onto the GPUI window canvas.
pub fn paint_centered_text(
    window: &mut Window,
    cx: &mut App,
    text: &str,
    center: Point<Pixels>,
    font_size: f32,
    color: Hsla,
    bold: bool,
) {
    if text.is_empty() {
        return;
    }
    let s: SharedString = text.to_string().into();
    let mut font = window.text_style().font();
    font.weight = if bold {
        FontWeight::BOLD
    } else {
        FontWeight::SEMIBOLD
    };
    let run = TextRun {
        len: s.len(),
        font,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let fs = px(font_size.max(7.0));
    let line = window.text_system().shape_line(s, fs, &[run], None);
    let origin = point(
        center.x - line.width * 0.5,
        center.y - fs * 0.62,
    );
    let _ = line.paint(origin, fs * 1.25, TextAlign::Left, None, window, cx);
}

/// Paints a rounded dark pill badge with centered text inside a canvas (clamped to `bounds`).
pub fn paint_canvas_pill(
    window: &mut Window,
    cx: &mut App,
    bounds: Bounds<Pixels>,
    text: &str,
    center: Point<Pixels>,
    fg: Hsla,
    bg: Hsla,
    font_size: f32,
) {
    if text.is_empty() {
        return;
    }
    let s: SharedString = text.to_string().into();
    let mut font = window.text_style().font();
    font.weight = FontWeight::BOLD;
    let run = TextRun {
        len: s.len(),
        font,
        color: fg,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let fs = px(font_size.max(8.0));
    let line = window.text_system().shape_line(s, fs, &[run], None);
    let pad_x = px(7.0);
    let pill_w = line.width + pad_x * 2.0;
    let pill_h = fs + px(6.0);

    let min_x = bounds.origin.x + px(4.0);
    let max_x = (bounds.origin.x + bounds.size.width - pill_w - px(4.0)).max(min_x);
    let min_y = bounds.origin.y + px(4.0);
    let max_y = (bounds.origin.y + bounds.size.height - pill_h - px(4.0)).max(min_y);

    let rx = (center.x - pill_w * 0.5).clamp(min_x, max_x);
    let ry = (center.y - pill_h * 0.5).clamp(min_y, max_y);

    window.paint_quad(PaintQuad {
        bounds: Bounds::new(point(rx, ry), size(pill_w, pill_h)),
        corner_radii: Corners::all(px(5.0)),
        background: bg.into(),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::Solid,
    });

    let text_origin = point(rx + pad_x, ry + px(2.5));
    let _ = line.paint(text_origin, fs * 1.2, TextAlign::Left, None, window, cx);
}

// ============================================================================
// Reusable UI Components
// ============================================================================

/// Circular 0–100 score ring rendered via GPUI `canvas()`.
pub fn score_ring(score: u8, size_px: f32, cx: &App) -> impl IntoElement {
    let clamped = score.min(100);
    let fg = score_color(clamped);
    let track = cx.theme().border;
    let text_col = cx.theme().foreground;

    div()
        .w(px(size_px))
        .h(px(size_px))
        .flex_shrink_0()
        .child(
            canvas(
                move |_, _, _| {},
                move |bounds, _, window, cx| {
                    let cx_p = bounds.origin.x + bounds.size.width * 0.5;
                    let cy_p = bounds.origin.y + bounds.size.height * 0.5;
                    let center = point(cx_p, cy_p);
                    let stroke = (size_px * 0.11).max(4.0);
                    let r = (size_px * 0.5 - stroke * 0.7).max(6.0);

                    paint_arc_stroke(window, center, r, 0.0, TAU, stroke, track, None, 40);
                    if clamped > 0 {
                        let sweep = TAU * (clamped as f32 / 100.0);
                        paint_arc_stroke(
                            window,
                            center,
                            r,
                            -FRAC_PI_2,
                            -FRAC_PI_2 + sweep,
                            stroke,
                            fg,
                            None,
                            40,
                        );
                    }
                    paint_centered_text(
                        window,
                        cx,
                        &clamped.to_string(),
                        center,
                        size_px * 0.28,
                        text_col,
                        true,
                    );
                },
            )
            .size_full(),
        )
}

/// Horizontal progress / suitability bar with label and numeric value.
pub fn suitability_bar(
    label: impl Into<SharedString>,
    value: u8,
    max: u8,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let denom = max.max(1) as f32;
    let pct = (value as f32 / denom).clamp(0.0, 1.0);
    let bar_col = score_color(((pct * 100.0).round() as u8).min(100));

    div()
        .h_flex()
        .w_full()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(120.0))
                .text_xs()
                .text_color(theme.muted_foreground)
                .truncate()
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .h(px(7.0))
                .rounded_full()
                .bg(theme.border)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(relative(pct))
                        .rounded_full()
                        .bg(bar_col),
                ),
        )
        .child(
            div()
                .w(px(28.0))
                .text_xs()
                .font_semibold()
                .text_right()
                .text_color(theme.foreground)
                .child(value.to_string()),
        )
}

/// Color-coded FM attribute chip (e.g. `PAS 16`).
pub fn attr_chip(code: &str, value: f32, cx: &App) -> impl IntoElement {
    let col = attr_color(value);
    let theme = cx.theme();
    div()
        .h_flex()
        .items_center()
        .gap_1()
        .px_2()
        .py(px(2.0))
        .rounded_sm()
        .border_1()
        .border_color(with_alpha(col, 0.45))
        .bg(with_alpha(col, 0.14))
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(code.to_uppercase()),
        )
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(col)
                .child(format!("{}", value.round() as i32)),
        )
}

/// Semantic pill badge (`Good`, `Warn`, `Crit`, `Info`, `Muted`).
pub fn badge_pill(text: impl Into<SharedString>, tone: BadgeTone, cx: &App) -> Div {
    let theme = cx.theme();
    let (bg, fg, border) = match tone {
        BadgeTone::Good => (
            with_alpha(rgb(0x16a34a).into(), 0.18),
            rgb(0x22c55e).into(),
            with_alpha(rgb(0x16a34a).into(), 0.45),
        ),
        BadgeTone::Warn => (
            with_alpha(rgb(0xd97706).into(), 0.18),
            rgb(0xf59e0b).into(),
            with_alpha(rgb(0xd97706).into(), 0.45),
        ),
        BadgeTone::Crit => (
            with_alpha(rgb(0xdc2626).into(), 0.18),
            rgb(0xef4444).into(),
            with_alpha(rgb(0xdc2626).into(), 0.45),
        ),
        BadgeTone::Info => (
            with_alpha(rgb(0x2563eb).into(), 0.18),
            rgb(0x60a5fa).into(),
            with_alpha(rgb(0x2563eb).into(), 0.45),
        ),
        BadgeTone::Muted => (theme.secondary, theme.muted_foreground, theme.border),
    };

    div()
        .px_2()
        .py(px(2.0))
        .rounded_full()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_xs()
        .font_semibold()
        .text_color(fg)
        .child(text.into())
}

/// Compact KPI summary card.
pub fn kpi_card(
    title: impl Into<SharedString>,
    value: impl Into<SharedString>,
    hint: Option<SharedString>,
    accent: Option<Hsla>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    let val_color = accent.unwrap_or(theme.foreground);

    div()
        .v_flex()
        .flex_1()
        .min_w(px(140.0))
        .gap_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(title.into()),
        )
        .child(
            div()
                .text_xl()
                .font_bold()
                .text_color(val_color)
                .child(value.into()),
        )
        .when_some(hint, |el, h| {
            el.child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(h),
            )
        })
}

/// Collapsible card panel with clickable header and chevron indicator.
pub fn collapsible_card<F>(
    id: impl Into<ElementId>,
    title: impl Into<SharedString>,
    subtitle: Option<SharedString>,
    collapsed: bool,
    on_toggle: F,
    body: impl IntoElement,
    cx: &App,
) -> impl IntoElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let theme = cx.theme();
    let chevron = if collapsed { "▸" } else { "▾" };

    div()
        .v_flex()
        .w_full()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .overflow_hidden()
        .child(
            div()
                .id(id)
                .h_flex()
                .w_full()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .cursor_pointer()
                .when(!collapsed, |d| d.border_b_1().border_color(theme.border))
                .on_click(on_toggle)
                .child(
                    div()
                        .v_flex()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_sm()
                                .font_semibold()
                                .text_color(theme.foreground)
                                .child(title.into()),
                        )
                        .when_some(subtitle, |d, sub| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(sub),
                            )
                        }),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(chevron),
                ),
        )
        .when(!collapsed, |d| d.child(div().p_4().child(body)))
}

/// Centered empty-state card with title, explanation, and optional action element.
pub fn empty_state_card(
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    action: Option<AnyElement>,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .v_flex()
        .w_full()
        .items_center()
        .justify_center()
        .gap_2()
        .p_6()
        .rounded_md()
        .border_1()
        .border_color(theme.border)
        .bg(theme.secondary)
        .child(
            div()
                .text_base()
                .font_semibold()
                .text_color(theme.foreground)
                .child(title.into()),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .max_w(px(520.0))
                .text_center()
                .child(body.into()),
        )
        .when_some(action, |d, act| d.child(div().pt_2().child(act)))
}

/// Slim top progress strip shown while a background task is active.
pub fn busy_top_bar(busy: &BusyState, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let pct = busy.pct.unwrap_or(0.35).clamp(0.08, 1.0);
    div()
        .w_full()
        .h(px(3.0))
        .bg(theme.border)
        .overflow_hidden()
        .child(div().h_full().w(relative(pct)).bg(theme.primary))
}

/// Non-obscuring floating top-right activity card for active background jobs.
pub fn busy_floating_card(busy: &BusyState, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let meta = match busy.pct {
        Some(p) => format!("{}% · {:.1}s", (p * 100.0).round() as u8, busy.elapsed_secs()),
        None => format!("{:.1}s", busy.elapsed_secs()),
    };

    div()
        .absolute()
        .top(px(56.0))
        .right(px(16.0))
        .w(px(280.0))
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(theme.primary)
        .bg(theme.secondary)
        .shadow_md()
        .v_flex()
        .gap_1()
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
                        .truncate()
                        .child(busy.label.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(meta),
                ),
        )
        .child(busy_top_bar(busy, cx))
}

/// Floating bottom-right toast notification with dismiss button.
pub fn toast_banner<F>(toast: &ToastState, on_dismiss: F, cx: &App) -> impl IntoElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    let theme = cx.theme();
    let accent: Hsla = match toast.kind {
        ToastKind::Success => rgb(0x16a34a).into(),
        ToastKind::Warn => rgb(0xd97706).into(),
        ToastKind::Error => rgb(0xdc2626).into(),
        ToastKind::Info => theme.primary,
    };

    div()
        .absolute()
        .bottom(px(16.0))
        .right(px(16.0))
        .max_w(px(420.0))
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(accent)
        .bg(theme.secondary)
        .shadow_md()
        .h_flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w(px(8.0))
                .h(px(8.0))
                .rounded_full()
                .flex_shrink_0()
                .bg(accent),
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(theme.foreground)
                .child(toast.message.clone()),
        )
        .child(
            Button::new("dismiss-toast")
                .ghost()
                .xsmall()
                .label("✕")
                .on_click(on_dismiss),
        )
}

#[cfg(test)]
mod tests {
    use super::parse_css_color;

    #[test]
    fn css_color_parser_handles_hex_and_rgba() {
        let c1 = parse_css_color("#14b8a6");
        assert!((c1.a - 1.0).abs() < 0.01);
        let c2 = parse_css_color("#fff");
        assert!((c2.l - 1.0).abs() < 0.01);
        let c3 = parse_css_color("rgba(255,107,90,0.5)");
        assert!((c3.a - 0.5).abs() < 0.02);
    }
}
