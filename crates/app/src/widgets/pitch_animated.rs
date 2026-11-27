//! 60fps GPUI `canvas()` animated 11v11 pitch renderer driven by `ta_core::types::SimModel`.
//!
//! Renders grass stripes, pitch markings, OOP gap/behind overlays, tactical zones & lines,
//! movement trails, run/press/cover arrows, curved/dashed ball trajectories, 22 player tokens
//! with shadows/facing arrows/pressing rings, 3D-arcing ball with altitude shadow, click-to-focus
//! hit-testing, and a playback control bar (Play/Pause, 0.5x/1x/2x speed, 6-phase scrubber,
//! opponent & trails toggles).

use crate::widgets::common::{
    paint_arc_stroke, paint_arrow, paint_canvas_pill, paint_centered_text, paint_circle_fill,
    paint_ellipse_fill, paint_rect_stroke, parse_css_color, rgba_f32, with_alpha,
};
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, Sizable, StyledExt};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};
use std::rc::Rc;
use std::time::Instant;
use ta_core::data::role_by_id;
use ta_core::types::{
    BallKind, BallPoint, BallSample, Lang, Player, SimArrowKind, SimModel, SimOverlayKind,
    SimPhase, SimZoneKind,
};

#[derive(Debug, Clone, Copy)]
struct HitCircle {
    idx: usize,
    cx: f32,
    cy: f32,
    r: f32,
}

#[inline]
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) * 0.5
    }
}

/// Evaluates the 3D ball position `(x, y, z)` along a phase's `BallPoint` trajectory.
pub fn sample_ball(path: &[BallPoint], bu: f32) -> BallSample {
    if path.is_empty() {
        return BallSample {
            x: 50.0,
            y: 50.0,
            z: 0.0,
            kind: BallKind::Pass,
        };
    }
    if path.len() == 1 {
        return BallSample {
            x: path[0].x,
            y: path[0].y,
            z: 0.0,
            kind: path[0].kind,
        };
    }
    let seg = path.len() - 1;
    let f = bu.clamp(0.0, 1.0) * seg as f32;
    let j = (f.floor() as usize).min(seg - 1);
    let r = ease(f - j as f32);
    let a = path[j];
    let b = path[j + 1];
    let kind = b.kind;
    let bx = a.x + (b.x - a.x) * r;
    let by = a.y + (b.y - a.y) * r;
    let z = match kind {
        BallKind::Loft => 4.0 * r * (1.0 - r),
        BallKind::Curve => 2.2 * r * (1.0 - r),
        _ => 0.0,
    };
    if matches!(kind, BallKind::Loft | BallKind::Curve) {
        let dx = b.y - a.y;
        let dy = b.x - a.x;
        let len = dx.hypot(dy).max(1.0);
        let h = (if kind == BallKind::Loft { 7.0 } else { 4.5 }) * 4.0 * r * (1.0 - r);
        return BallSample {
            x: (bx + (dx / len) * h * 0.45).clamp(2.0, 98.0),
            y: (by - (dy / len) * h * 0.3).clamp(1.0, 99.0),
            z,
            kind,
        };
    }
    BallSample {
        x: bx,
        y: by,
        z,
        kind,
    }
}

fn surname(full: &str) -> String {
    let parts: Vec<&str> = full.split_whitespace().collect();
    let s = if parts.len() >= 2 {
        parts[parts.len() - 1]
    } else {
        full.trim()
    };
    s.chars().take(11).collect()
}

/// Interactive 60fps GPUI view managing playback state and canvas hit-testing for `SimModel`.
pub struct AnimatedPitchView {
    pub model: Option<SimModel>,
    pub lang: Lang,
    pub playing: bool,
    pub speed: f32,
    pub phase_idx: usize,
    pub u: f32,
    pub show_opp: bool,
    pub show_trails: bool,
    pub focus: Option<usize>,
    pub player_names: HashMap<String, String>,
    last_tick: Instant,
    hits: Rc<RefCell<Vec<HitCircle>>>,
}

impl AnimatedPitchView {
    pub fn new(lang: Lang) -> Self {
        Self {
            model: None,
            lang,
            playing: true,
            speed: 1.0,
            phase_idx: 0,
            u: 0.0,
            show_opp: true,
            show_trails: false,
            focus: None,
            player_names: HashMap::new(),
            last_tick: Instant::now(),
            hits: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn set_model(&mut self, model: Option<SimModel>) {
        self.model = model;
        self.phase_idx = 0;
        self.u = 0.0;
        self.last_tick = Instant::now();
    }

    pub fn sync_squad(&mut self, squad: &[Player]) {
        self.player_names = squad
            .iter()
            .map(|p| (p.id.clone(), surname(&p.name)))
            .collect();
    }

    fn advance(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.1);
        self.last_tick = now;
        if !self.playing || self.model.is_none() {
            return;
        }
        // ~2.6 seconds per phase at 1.0x speed
        self.u += (dt * self.speed) / 2.6;
        if self.u >= 1.0 {
            self.u -= 1.0;
            self.phase_idx = (self.phase_idx + 1) % SimPhase::COUNT;
        }
    }

    fn handle_canvas_click(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        let mx: f32 = pos.x.into();
        let my: f32 = pos.y.into();
        let hit = self
            .hits
            .borrow()
            .iter()
            .find(|h| (mx - h.cx).hypot(my - h.cy) <= h.r)
            .map(|h| h.idx);
        if let Some(i) = hit {
            self.focus = if self.focus == Some(i) { None } else { Some(i) };
            cx.notify();
        }
    }

    fn render_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let lang = self.lang;
        let theme = cx.theme();

        let mut phase_btns = div().h_flex().flex_wrap().items_center().gap_1();
        for ph in SimPhase::ALL {
            let idx = ph.index();
            let active = self.phase_idx == idx;
            phase_btns = phase_btns.child(
                Button::new(("sim-phase", idx))
                    .xsmall()
                    .label(ph.label(lang))
                    .when(active, |b| b.primary())
                    .when(!active, |b| b.outline())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.phase_idx = idx;
                        this.u = 0.0;
                        this.last_tick = Instant::now();
                        cx.notify();
                    })),
            );
        }

        let mut speed_btns = div().h_flex().items_center().gap_1();
        for (ix, (sp, lbl)) in [(0.5_f32, "0.5×"), (1.0_f32, "1×"), (2.0_f32, "2×")]
            .into_iter()
            .enumerate()
        {
            let active = (self.speed - sp).abs() < 0.05;
            speed_btns = speed_btns.child(
                Button::new(("sim-speed", ix))
                    .xsmall()
                    .label(lbl)
                    .when(active, |b| b.primary())
                    .when(!active, |b| b.outline())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.speed = sp;
                        cx.notify();
                    })),
            );
        }

        div()
            .v_flex()
            .gap_2()
            .p_3()
            .rounded(px(crate::widgets::common::RADIUS))
            .border_1()
            .border_color(theme.border)
            .bg(theme.secondary)
            .shadow_md()
            .child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("sim-play")
                                    .primary()
                                    .xsmall()
                                    .label(if self.playing {
                                        lang.pick("⏸ Pausa", "⏸ Pause")
                                    } else {
                                        lang.pick("▶ Reproduzir", "▶ Play")
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.playing = !this.playing;
                                        this.last_tick = Instant::now();
                                        cx.notify();
                                    })),
                            )
                            .child(speed_btns),
                    )
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_1()
                            .child(
                                Button::new("sim-opp")
                                    .xsmall()
                                    .label(lang.pick("Adversário", "Opponents"))
                                    .when(self.show_opp, |b| b.primary())
                                    .when(!self.show_opp, |b| b.outline())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_opp = !this.show_opp;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("sim-trails")
                                    .xsmall()
                                    .label(lang.pick("Rastos", "Trails"))
                                    .when(self.show_trails, |b| b.primary())
                                    .when(!self.show_trails, |b| b.outline())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_trails = !this.show_trails;
                                        cx.notify();
                                    })),
                            )
                            .when(self.focus.is_some(), |d| {
                                d.child(
                                    Button::new("sim-unfocus")
                                        .ghost()
                                        .xsmall()
                                        .label(lang.pick("Limpar foco", "Clear focus"))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.focus = None;
                                            cx.notify();
                                        })),
                                )
                            }),
                    ),
            )
            .child(phase_btns)
    }
}

impl Render for AnimatedPitchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.advance();
        if self.playing && self.model.is_some() {
            window.request_animation_frame();
        }

        let lang = self.lang;
        let model = self.model.clone();
        let to_ph = SimPhase::from_index(self.phase_idx);
        let from_ph = SimPhase::from_index((self.phase_idx + SimPhase::COUNT - 1) % SimPhase::COUNT);
        let um = ease((self.u * 1.45).clamp(0.0, 1.0));
        let bu = ((self.u - 0.12) / 0.88).clamp(0.0, 1.0);
        let pulse = (self.u * 2.0).fract();
        let show_opp = self.show_opp;
        let show_trails = self.show_trails;
        let focus = self.focus;
        let names = self.player_names.clone();
        let hits = self.hits.clone();
        let oopish = matches!(to_ph, SimPhase::Oop | SimPhase::Loss);
        let phase_label = format!("{} · {}", to_ph.tag(), to_ph.label(lang));

        div()
            .v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .id("animated-pitch-surface")
                    .w_full()
                    .h(px(440.0))
                    .rounded(px(crate::widgets::common::RADIUS))
                    .shadow_md()
                    .overflow_hidden()
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                            this.handle_canvas_click(ev.position, cx);
                        }),
                    )
                    .child(
                        canvas(
                            move |_, _, _| {},
                            move |bounds, _, window, cx| {
                                paint_animated_frame(
                                    bounds,
                                    window,
                                    cx,
                                    model.as_ref(),
                                    from_ph,
                                    to_ph,
                                    um,
                                    bu,
                                    pulse,
                                    oopish,
                                    show_opp,
                                    show_trails,
                                    focus,
                                    &names,
                                    &phase_label,
                                    lang,
                                    &hits,
                                );
                            },
                        )
                        .size_full(),
                    ),
            )
            .child(self.render_controls(cx))
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_animated_frame(
    bounds: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
    model: Option<&SimModel>,
    from_ph: SimPhase,
    to_ph: SimPhase,
    move_u: f32,
    bu: f32,
    pulse: f32,
    oopish: bool,
    show_opp: bool,
    show_trails: bool,
    focus: Option<usize>,
    names: &HashMap<String, String>,
    phase_label: &str,
    lang: Lang,
    hits: &Rc<RefCell<Vec<HitCircle>>>,
) {
    let ox: f32 = bounds.origin.x.into();
    let oy: f32 = bounds.origin.y.into();
    let bw: f32 = bounds.size.width.into();
    let bh: f32 = bounds.size.height.into();
    // Maintain 110:72 virtual coordinate space identical to `src/ui/sim_view.js`.
    let sx = bw / 110.0;
    let sy = bh / 72.0;
    let sc = sx.min(sy);
    let vx = |y_fm: f32| 3.0 + y_fm * 1.04;
    let vy = |x_fm: f32| 2.0 + x_fm * 0.68;
    let pt = |x110: f32, y72: f32| point(px(ox + x110 * sx), px(oy + y72 * sy));

    // Grass + vertical stripes
    window.paint_quad(fill(bounds, rgb(0x2c5a36)));
    for k in (1..10).step_by(2) {
        let x0 = 3.0 + (k as f32) * 10.4;
        window.paint_quad(fill(
            Bounds::new(pt(x0, 2.0), size(px(10.4 * sx), px(68.0 * sy))),
            rgb(0x275131),
        ));
    }

    // Pitch lines, boxes, circles, arcs, goals
    let pline = rgba_f32(1.0, 1.0, 1.0, 0.6);
    paint_rect_stroke(window, pt(3.0, 2.0), 104.0 * sx, 68.0 * sy, (0.36 * sc).max(1.0), pline, None);
    let mut mid = PathBuilder::stroke(px((0.36 * sc).max(1.0)));
    mid.move_to(pt(55.0, 2.0));
    mid.line_to(pt(55.0, 70.0));
    if let Ok(p) = mid.build() {
        window.paint_path(p, pline);
    }
    paint_arc_stroke(window, pt(55.0, 36.0), 9.2 * sc, 0.0, TAU, (0.36 * sc).max(1.0), pline, None, 36);
    paint_circle_fill(window, pt(55.0, 36.0), (0.55 * sc).max(2.0), pline, 12);

    paint_rect_stroke(window, pt(3.0, 15.8), 16.6 * sx, 40.4 * sy, (0.36 * sc).max(1.0), pline, None);
    paint_rect_stroke(window, pt(3.0, 26.8), 5.6 * sx, 18.4 * sy, (0.36 * sc).max(1.0), pline, None);
    paint_rect_stroke(window, pt(90.4, 15.8), 16.6 * sx, 40.4 * sy, (0.36 * sc).max(1.0), pline, None);
    paint_rect_stroke(window, pt(101.4, 26.8), 5.6 * sx, 18.4 * sy, (0.36 * sc).max(1.0), pline, None);
    paint_circle_fill(window, pt(14.5, 36.0), (0.45 * sc).max(1.8), pline, 10);
    paint_circle_fill(window, pt(95.5, 36.0), (0.45 * sc).max(1.8), pline, 10);
    paint_arc_stroke(window, pt(14.5, 36.0), 8.2 * sc, -0.65, 0.65, (0.36 * sc).max(1.0), pline, None, 16);
    paint_arc_stroke(
        window,
        pt(95.5, 36.0),
        8.2 * sc,
        PI - 0.65,
        PI + 0.65,
        (0.36 * sc).max(1.0),
        pline,
        None,
        16,
    );

    // Goal frames
    for gx in [1.3_f32, 107.0_f32] {
        window.paint_quad(fill(
            Bounds::new(pt(gx, 32.4), size(px(1.7 * sx), px(7.2 * sy))),
            rgba_f32(1.0, 1.0, 1.0, 0.14),
        ));
        paint_rect_stroke(window, pt(gx, 32.4), 1.7 * sx, 7.2 * sy, (0.65 * sc).max(1.2), pline, None);
    }

    // Subtle pitch direction labels
    paint_centered_text(
        window,
        cx,
        lang.pick("a tua baliza", "your goal"),
        pt(11.0, 68.2),
        (2.0 * sc).clamp(9.0, 12.0),
        rgba_f32(1.0, 1.0, 1.0, 0.55),
        false,
    );
    paint_centered_text(
        window,
        cx,
        lang.pick("ataque →", "attack →"),
        pt(99.0, 68.2),
        (2.0 * sc).clamp(9.0, 12.0),
        rgba_f32(1.0, 1.0, 1.0, 0.55),
        false,
    );

    let Some(m) = model else {
        hits.borrow_mut().clear();
        paint_canvas_pill(
            window,
            cx,
            bounds,
            lang.pick(
                "Simulador 11v11 pronto — aguarda ta-core::live::sim_model",
                "11v11 canvas renderer ready — awaiting ta-core::live::sim_model",
            ),
            pt(55.0, 36.0),
            rgb(0xffffff).into(),
            rgba_f32(0.08, 0.08, 0.1, 0.8),
            12.0,
        );
        return;
    };

    let from_i = from_ph.index();
    let to_i = to_ph.index();

    // 1. OOP overlays (gap / space behind)
    if to_ph == SimPhase::Oop {
        let dash = [px(5.0), px(3.5)];
        for o in &m.overlays {
            let x1 = vx(o.y1);
            let x2 = vx(o.y2);
            if x2 <= x1 {
                continue;
            }
            let (fill_c, stroke_c) = match o.kind {
                SimOverlayKind::Gap => (
                    rgba_f32(1.0, 0.42, 0.35, 0.18),
                    rgba_f32(1.0, 0.42, 0.35, 0.75),
                ),
                SimOverlayKind::Behind => (
                    rgba_f32(1.0, 0.78, 0.31, 0.16),
                    rgba_f32(1.0, 0.78, 0.31, 0.75),
                ),
            };
            window.paint_quad(fill(
                Bounds::new(pt(x1, 2.0), size(px((x2 - x1) * sx), px(68.0 * sy))),
                fill_c,
            ));
            paint_rect_stroke(
                window,
                pt(x1, 2.0),
                (x2 - x1) * sx,
                68.0 * sy,
                (0.35 * sc).max(1.0),
                stroke_c,
                Some(&dash),
            );
            paint_canvas_pill(
                window,
                cx,
                bounds,
                &o.label,
                pt((x1 + x2) * 0.5, 5.4),
                rgb(0xffffff).into(),
                rgba_f32(0.08, 0.07, 0.06, 0.74),
                10.0,
            );
        }
    }

    // 2. Tactical zones
    let zone_dash = [px(5.5), px(3.5)];
    for z in &m.zones[to_i] {
        let fill_c = parse_css_color(z.color);
        let stroke_c = parse_css_color(z.stroke);
        match z.kind {
            SimZoneKind::Circle => {
                let c = pt(vx(z.cy), vy(z.cx));
                let r = (z.r.max(4.0) * 0.72) * sc;
                paint_circle_fill(window, c, r, with_alpha(fill_c, 0.24), 28);
                paint_arc_stroke(
                    window,
                    c,
                    r,
                    0.0,
                    TAU,
                    (0.4 * sc).max(1.1),
                    stroke_c,
                    Some(&zone_dash),
                    28,
                );
                if !z.label.is_empty() {
                    paint_canvas_pill(
                        window,
                        cx,
                        bounds,
                        &z.label,
                        point(c.x, c.y - px(r + 10.0)),
                        rgb(0xffffff).into(),
                        rgba_f32(0.08, 0.07, 0.06, 0.75),
                        10.0,
                    );
                }
            }
            SimZoneKind::Trap | SimZoneKind::Lane => {
                let x1 = vx(z.y1);
                let x2 = vx(z.y2);
                let y1 = vy(z.x1);
                let y2 = vy(z.x2);
                let w = ((x2 - x1).abs().max(1.0)) * sx;
                let h = ((y2 - y1).abs().max(1.0)) * sy;
                let orig = pt(x1.min(x2), y1.min(y2));
                window.paint_quad(fill(
                    Bounds::new(orig, size(px(w), px(h))),
                    with_alpha(fill_c, 0.22),
                ));
                paint_rect_stroke(window, orig, w, h, (0.38 * sc).max(1.0), stroke_c, Some(&zone_dash));
                if !z.label.is_empty() {
                    paint_canvas_pill(
                        window,
                        cx,
                        bounds,
                        &z.label,
                        pt((x1 + x2) * 0.5, y1.min(y2) + 2.8),
                        rgb(0xffffff).into(),
                        rgba_f32(0.08, 0.07, 0.06, 0.75),
                        10.0,
                    );
                }
            }
        }
    }

    // 3. Tactical lines
    let line_dash = [px(7.0), px(4.5)];
    for ln in &m.lines[to_i] {
        let x = vx(ln.y);
        let col = parse_css_color(ln.color);
        let mut pb = PathBuilder::stroke(px((0.48 * sc).max(1.3))).dash_array(&line_dash);
        pb.move_to(pt(x, 2.0));
        pb.line_to(pt(x, 70.0));
        if let Ok(p) = pb.build() {
            window.paint_path(p, col);
        }
        if !ln.label.is_empty() {
            paint_canvas_pill(
                window,
                cx,
                bounds,
                &ln.label,
                pt(x, 5.2),
                col,
                rgba_f32(0.08, 0.07, 0.06, 0.78),
                10.0,
            );
        }
    }

    // 4. Player movement trails across phases
    if show_trails || focus.is_some() {
        let trail_dash = [px(4.5), px(3.0)];
        for p in &m.players {
            if !show_trails && focus != Some(p.idx) {
                continue;
            }
            let is_f = focus == Some(p.idx);
            let mut pb = PathBuilder::stroke(px(if is_f { 2.0 } else { 1.1 })).dash_array(&trail_dash);
            for (k, pos) in p.pos.iter().enumerate() {
                let q = pt(vx(pos.y), vy(pos.x));
                if k == 0 {
                    pb.move_to(q);
                } else {
                    pb.line_to(q);
                }
            }
            pb.close();
            if let Ok(path) = pb.build() {
                let col = if is_f {
                    rgba_f32(1.0, 0.89, 0.48, 0.75)
                } else {
                    rgba_f32(1.0, 1.0, 1.0, 0.28)
                };
                window.paint_path(path, col);
            }
        }
    }

    // 5. Role / tactical arrows
    let cover_dash = [px(5.0), px(3.0)];
    for ar in &m.arrows[to_i] {
        let col = with_alpha(parse_css_color(ar.color), if ar.dim { 0.35 } else { 0.88 });
        paint_arrow(
            window,
            pt(vx(ar.from.y), vy(ar.from.x)),
            pt(vx(ar.to.y), vy(ar.to.x)),
            col,
            (0.56 * sc).max(1.8),
            if ar.kind == SimArrowKind::Cover {
                Some(&cover_dash)
            } else {
                None
            },
            0.0,
        );
    }

    // 6. Ball trajectory arrows
    let bp = &m.ball[to_i];
    let loft_dash = [px(7.0), px(4.5)];
    let drib_dash = [px(3.0), px(3.0)];
    let opp_dash = [px(5.0), px(3.5)];
    for seg in bp.windows(2) {
        let a = seg[0];
        let b = seg[1];
        let kind = b.kind;
        let col = match kind {
            BallKind::Shot => rgb(0xff9b42).into(),
            BallKind::Dribble => rgb(0xffe27a).into(),
            BallKind::Opp => rgba_f32(0.92, 0.92, 0.92, 0.7),
            _ => rgba_f32(1.0, 1.0, 1.0, 0.9),
        };
        let dash = match kind {
            BallKind::Loft => Some(&loft_dash[..]),
            BallKind::Dribble => Some(&drib_dash[..]),
            BallKind::Opp => Some(&opp_dash[..]),
            _ => None,
        };
        let bend = match kind {
            BallKind::Loft => -7.0 * sc,
            BallKind::Curve => -4.5 * sc,
            _ => 0.0,
        };
        let w = match kind {
            BallKind::Shot => (0.72 * sc).max(2.2),
            BallKind::Opp => (0.44 * sc).max(1.4),
            _ => (0.52 * sc).max(1.7),
        };
        paint_arrow(window, pt(vx(a.y), vy(a.x)), pt(vx(b.y), vy(b.x)), col, w, dash, bend);
    }

    let ball_now = sample_ball(bp, bu);
    let bx_v = vx(ball_now.y);
    let by_v = vy(ball_now.x);
    let ball_ground = pt(bx_v, by_v);

    // 7. Opponent 11 tokens with ground shadows
    if show_opp {
        let opp_from = &m.opp[from_i];
        let opp_to = &m.opp[to_i];
        let opp_r = (1.5 * sc).clamp(6.0, 10.5);
        for (a, b) in opp_from.iter().zip(opp_to.iter()) {
            let ox_v = vx(a.y + (b.y - a.y) * move_u);
            let oy_v = vy(a.x + (b.x - a.x) * move_u);
            let c = pt(ox_v, oy_v);
            paint_ellipse_fill(
                window,
                point(c.x + px(1.5), c.y + px(3.5)),
                opp_r * 0.95,
                opp_r * 0.6,
                rgba_f32(0.0, 0.0, 0.0, 0.28),
                16,
            );
            paint_circle_fill(window, c, opp_r, rgba_f32(0.89, 0.91, 0.94, 0.82), 18);
            paint_arc_stroke(
                window,
                c,
                opp_r,
                0.0,
                TAU,
                1.1,
                rgba_f32(0.06, 0.09, 0.16, 0.6),
                None,
                18,
            );
        }
    }

    // 8. Managed team 11 tokens (shadows, facing arrows, pressing pulse rings, focus halo, role ab, surname)
    let mut new_hits = Vec::with_capacity(m.players.len());
    for p in &m.players {
        let a = p.pos[from_i];
        let b = p.pos[to_i];
        let x_fm = a.x + (b.x - a.x) * move_u;
        let y_fm = a.y + (b.y - a.y) * move_u;
        let c = pt(vx(y_fm), vy(x_fm));
        let is_f = focus == Some(p.idx);
        let is_gk = p.slot.ip_code == "GK";
        let r = ((if is_f { 2.75 } else { 2.3 }) * sc).clamp(9.0, 17.5);
        let cx_f: f32 = c.x.into();
        let cy_f: f32 = c.y.into();
        new_hits.push(HitCircle {
            idx: p.idx,
            cx: cx_f,
            cy: cy_f,
            r: r + 6.0,
        });

        // Pressing pulse ring
        if p.pressing && oopish {
            let pr = r + (1.1 + 1.1 * pulse) * sc;
            paint_arc_stroke(
                window,
                c,
                pr,
                0.0,
                TAU,
                (0.55 * sc).max(1.8),
                rgba_f32(1.0, 0.42, 0.35, (0.95 - pulse * 0.6).clamp(0.1, 1.0)),
                None,
                24,
            );
        }

        // Ground shadow
        paint_ellipse_fill(
            window,
            point(c.x + px(2.0), c.y + px(4.0)),
            r * 0.96,
            r * 0.62,
            rgba_f32(0.0, 0.0, 0.0, 0.36),
            18,
        );

        // Facing arrow pointing along movement vector or toward ball
        let mdx = (vx(b.y) - vx(a.y)) * sx;
        let mdy = (vy(b.x) - vy(a.x)) * sy;
        let bgx: f32 = ball_ground.x.into();
        let bgy: f32 = ball_ground.y.into();
        let ang = if mdx.hypot(mdy) > 4.0 {
            mdy.atan2(mdx)
        } else {
            (bgy - cy_f).atan2(bgx - cx_f)
        };
        let tip = point(
            c.x + px(ang.cos() * (r + 4.5)),
            c.y + px(ang.sin() * (r + 4.5)),
        );
        let fl = point(
            c.x + px((ang + 0.5).cos() * (r - 0.5)),
            c.y + px((ang + 0.5).sin() * (r - 0.5)),
        );
        let fr = point(
            c.x + px((ang - 0.5).cos() * (r - 0.5)),
            c.y + px((ang - 0.5).sin() * (r - 0.5)),
        );
        let mut facing = PathBuilder::fill();
        facing.add_polygon(&[tip, fl, fr], true);
        if let Ok(path) = facing.build() {
            let fc: Hsla = if is_f {
                rgb(0xffe27a).into()
            } else {
                rgba_f32(1.0, 1.0, 1.0, 0.85)
            };
            window.paint_path(path, fc);
        }

        // Focus ring
        if is_f {
            paint_arc_stroke(window, c, r + 4.0, 0.0, TAU, 2.2, rgb(0xffe27a).into(), None, 24);
        }

        // Token body & border
        let body_col: Hsla = if is_gk {
            rgb(0x464a52).into()
        } else if oopish {
            if is_f {
                rgb(0x2563eb).into()
            } else {
                rgb(0x1d416b).into()
            }
        } else if is_f {
            rgb(0x0f8a52).into()
        } else {
            rgb(0x135737).into()
        };
        let rim_col: Hsla = if is_gk {
            rgb(0xfacc15).into()
        } else if is_f {
            rgb(0xffe27a).into()
        } else {
            rgb(0xffffff).into()
        };
        paint_circle_fill(window, c, r, body_col, 24);
        paint_arc_stroke(window, c, r, 0.0, TAU, if is_f { 2.0 } else { 1.4 }, rim_col, None, 24);

        let role_id = if oopish { p.oop_role } else { p.ip_role };
        let ab = role_by_id(role_id).map(|rr| rr.ab).unwrap_or("?");
        paint_centered_text(window, cx, ab, c, (r * 0.72).max(8.5), rgb(0xffffff).into(), true);

        if let Some(pid) = p.slot.pid.as_deref() {
            if let Some(nm) = names.get(pid) {
                let lbl = point(c.x, c.y + px(r + 9.0));
                paint_centered_text(
                    window,
                    cx,
                    nm,
                    point(lbl.x + px(0.9), lbl.y + px(0.9)),
                    9.5,
                    rgba_f32(0.0, 0.0, 0.0, 0.82),
                    false,
                );
                paint_centered_text(window, cx, nm, lbl, 9.5, rgb(0xffffff).into(), false);
            }
        }
    }
    *hits.borrow_mut() = new_hits;

    // 9. 3D-arcing ball with altitude shadow & short motion trail
    if bp.len() >= 2 && bu > 0.01 {
        let prev_b = sample_ball(bp, (bu - 0.12).max(0.0));
        let p1 = pt(vx(prev_b.y), vy(prev_b.x) - prev_b.z * 3.1);
        let p2 = pt(bx_v, by_v - ball_now.z * 3.1);
        let mut trail = PathBuilder::stroke(px((1.0 * sc).max(2.5)));
        trail.move_to(p1);
        trail.line_to(p2);
        if let Ok(path) = trail.build() {
            let tc = if ball_now.kind == BallKind::Shot {
                rgba_f32(1.0, 0.61, 0.26, 0.65)
            } else {
                rgba_f32(1.0, 1.0, 1.0, 0.45)
            };
            window.paint_path(path, tc);
        }
    }

    let br = ((1.05 + ball_now.z * 0.45) * sc).clamp(4.5, 9.0);
    paint_ellipse_fill(
        window,
        pt(bx_v + 0.35 + ball_now.z * 0.7, by_v + 0.55 + ball_now.z * 0.45),
        br * 1.05,
        br * 0.62,
        rgba_f32(0.0, 0.0, 0.0, (0.38 - ball_now.z * 0.18).clamp(0.1, 0.4)),
        14,
    );
    let ball_air = pt(bx_v, by_v - ball_now.z * 3.1);
    paint_circle_fill(window, ball_air, br, rgb(0xffffff).into(), 16);
    paint_arc_stroke(window, ball_air, br, 0.0, TAU, 1.1, rgb(0x111111).into(), None, 16);
    paint_circle_fill(
        window,
        point(ball_air.x + px(br * 0.22), ball_air.y - px(br * 0.18)),
        br * 0.32,
        rgb(0x1f2937).into(),
        10,
    );

    // 10. Bottom-left phase label pill
    paint_canvas_pill(
        window,
        cx,
        bounds,
        phase_label,
        pt(18.0, 67.2),
        rgb(0xffffff).into(),
        rgba_f32(0.08, 0.07, 0.06, 0.78),
        11.0,
    );
}

#[cfg(test)]
mod tests {
    use super::sample_ball;
    use ta_core::types::{BallKind, BallPoint};

    #[test]
    fn sample_ball_loft_has_altitude_and_pass_is_flat() {
        let loft = [
            BallPoint { x: 20.0, y: 20.0, kind: BallKind::Loft },
            BallPoint { x: 80.0, y: 70.0, kind: BallKind::Loft },
        ];
        let mid_loft = sample_ball(&loft, 0.5);
        assert!(mid_loft.z > 0.8);

        let pass = [
            BallPoint { x: 20.0, y: 20.0, kind: BallKind::Pass },
            BallPoint { x: 80.0, y: 70.0, kind: BallKind::Pass },
        ];
        let mid_pass = sample_ball(&pass, 0.5);
        assert_eq!(mid_pass.z, 0.0);
    }
}
