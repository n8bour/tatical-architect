//! Live 11v11 animated pitch tactical model (6 match phases: Build-up,
//! Progression, Final third, Ball lost, Out-of-possession block, Ball won).
//!
//! Coordinates:
//! - `x`: width `0.0` (left touchline) -> `100.0` (right touchline)
//! - `y`: depth `0.0` (own goal line) -> `100.0` (opponent goal line)
//!
//! Pure non-Bible live match phase model ported faithfully from
//! `src/core/sim_model.js` (L1-572) and `simBallState` from `src/ui/sim_view.js`.

use std::collections::HashSet;

use crate::data::i18n::pt_terms;
use crate::data::roles::role_by_id;
use crate::data::tactics_data::{group_of, slot_info, ti_label, ti_val};
use crate::types::{
    BallKind, BallPoint, BallSample, Lang, PhaseCaption, PosGroup, Role, RoleTag, Side, SimArrow,
    SimArrowKind, SimLine, SimModel, SimOverlay, SimOverlayKind, SimPhase, SimPlayer, SimPoint,
    SimZone, SimZoneKind, SlotAssignment, SlotMeta, Tactic,
};

// ============================================================================
// Phase Metadata, Opponent Shape, Math Helpers
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseInfo {
    pub phase: SimPhase,
    pub key: &'static str,
    pub name_pt: &'static str,
    pub name_en: &'static str,
    pub tag: &'static str,
}

pub const PHASES: [PhaseInfo; SimPhase::COUNT] = [
    PhaseInfo {
        phase: SimPhase::Build,
        key: "build",
        name_pt: "Construção",
        name_en: "Build-up",
        tag: "IP",
    },
    PhaseInfo {
        phase: SimPhase::Prog,
        key: "prog",
        name_pt: "Progressão",
        name_en: "Progression",
        tag: "IP",
    },
    PhaseInfo {
        phase: SimPhase::Final,
        key: "final",
        name_pt: "Último terço",
        name_en: "Final third",
        tag: "IP",
    },
    PhaseInfo {
        phase: SimPhase::Loss,
        key: "loss",
        name_pt: "Perda da bola",
        name_en: "Ball lost",
        tag: "T",
    },
    PhaseInfo {
        phase: SimPhase::Oop,
        key: "oop",
        name_pt: "Bloco sem bola",
        name_en: "Out-of-possession block",
        tag: "OOP",
    },
    PhaseInfo {
        phase: SimPhase::Win,
        key: "win",
        name_pt: "Recuperação",
        name_en: "Ball won",
        tag: "T",
    },
];

pub const OPP_SHAPE: [[f32; 2]; 11] = [
    [50.0, 4.0],
    [12.0, 22.0],
    [36.0, 18.0],
    [64.0, 18.0],
    [88.0, 22.0],
    [40.0, 36.0],
    [60.0, 36.0],
    [14.0, 62.0],
    [50.0, 60.0],
    [86.0, 62.0],
    [50.0, 78.0],
];

/// Matches JS `Math.max(a, Math.min(b, v))`.
#[inline]
pub fn clamp(v: f32, a: f32, b: f32) -> f32 {
    v.min(b).max(a)
}

/// Quadratic ease-in-out curve matching JS `ease(u)`.
#[inline]
pub fn ease(u: f32) -> f32 {
    if u < 0.5 {
        2.0 * u * u
    } else {
        1.0 - (-2.0 * u + 2.0).powi(2) / 2.0
    }
}

// ============================================================================
// Base In-Possession Role Movement Table (`MOVES`)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XKey {
    Wide,
    Wide2,
    Wide3,
    Half,
    In,
    In2,
    Ch,
    Cbw,
    C,
    Ball,
    BallHalf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveStep {
    pub y: Option<f32>,
    pub xs: Option<f32>,
    pub x: Option<XKey>,
}

impl MoveStep {
    const fn y(y: f32) -> Self {
        Self {
            y: Some(y),
            xs: None,
            x: None,
        }
    }

    const fn y_xs(y: f32, xs: f32) -> Self {
        Self {
            y: Some(y),
            xs: Some(xs),
            x: None,
        }
    }

    const fn y_x(y: f32, x: XKey) -> Self {
        Self {
            y: Some(y),
            xs: None,
            x: Some(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoleMove {
    pub id: &'static str,
    pub build: MoveStep,
    pub prog: MoveStep,
    pub final_third: MoveStep,
}

impl RoleMove {
    pub const fn for_phase(&self, ph: SimPhase) -> Option<MoveStep> {
        match ph {
            SimPhase::Build => Some(self.build),
            SimPhase::Prog => Some(self.prog),
            SimPhase::Final => Some(self.final_third),
            _ => None,
        }
    }
}

pub const MOVES: &[RoleMove] = &[
    RoleMove { id: "gk_ip", build: MoveStep::y(5.0), prog: MoveStep::y(11.0), final_third: MoveStep::y(16.0) },
    RoleMove { id: "bpgk", build: MoveStep::y(11.0), prog: MoveStep::y(18.0), final_third: MoveStep::y(24.0) },
    RoleMove { id: "nngk", build: MoveStep::y(4.0), prog: MoveStep::y(8.0), final_third: MoveStep::y(12.0) },
    RoleMove { id: "cb_ip", build: MoveStep::y_xs(13.0, 1.4), prog: MoveStep::y(26.0), final_third: MoveStep::y(38.0) },
    RoleMove { id: "bpcb", build: MoveStep::y_xs(14.0, 1.45), prog: MoveStep::y(30.0), final_third: MoveStep::y(42.0) },
    RoleMove { id: "nncb", build: MoveStep::y_xs(11.0, 1.22), prog: MoveStep::y(21.0), final_third: MoveStep::y(32.0) },
    RoleMove { id: "acb", build: MoveStep::y(16.0), prog: MoveStep::y_x(42.0, XKey::In), final_third: MoveStep::y_x(55.0, XKey::In) },
    RoleMove { id: "ocb", build: MoveStep::y_xs(15.0, 1.55), prog: MoveStep::y_x(48.0, XKey::Wide), final_third: MoveStep::y_x(78.0, XKey::Wide) },
    RoleMove { id: "fb_ip", build: MoveStep::y_x(18.0, XKey::Wide), prog: MoveStep::y_x(33.0, XKey::Wide), final_third: MoveStep::y_x(48.0, XKey::Wide) },
    RoleMove { id: "ifb", build: MoveStep::y_x(13.0, XKey::Cbw), prog: MoveStep::y_x(24.0, XKey::Cbw), final_third: MoveStep::y_x(36.0, XKey::Cbw) },
    RoleMove { id: "wb_ip", build: MoveStep::y_x(24.0, XKey::Wide), prog: MoveStep::y_x(52.0, XKey::Wide), final_third: MoveStep::y_x(80.0, XKey::Wide) },
    RoleMove { id: "iwb", build: MoveStep::y_x(27.0, XKey::Half), prog: MoveStep::y_x(42.0, XKey::Half), final_third: MoveStep::y_x(54.0, XKey::Half) },
    RoleMove { id: "pwb", build: MoveStep::y_x(29.0, XKey::Half), prog: MoveStep::y_x(48.0, XKey::Half), final_third: MoveStep::y_x(64.0, XKey::Half) },
    RoleMove { id: "awb", build: MoveStep::y_x(29.0, XKey::Wide), prog: MoveStep::y_x(64.0, XKey::Wide), final_third: MoveStep::y_x(88.0, XKey::Wide) },
    RoleMove { id: "dm_ip", build: MoveStep::y(27.0), prog: MoveStep::y(38.0), final_third: MoveStep::y(46.0) },
    RoleMove { id: "dlp", build: MoveStep::y(24.0), prog: MoveStep::y(39.0), final_third: MoveStep::y(53.0) },
    RoleMove { id: "hb", build: MoveStep::y_x(12.0, XKey::C), prog: MoveStep::y_x(22.0, XKey::C), final_third: MoveStep::y_x(35.0, XKey::C) },
    RoleMove { id: "b2b", build: MoveStep::y(33.0), prog: MoveStep::y(56.0), final_third: MoveStep::y_x(85.0, XKey::In) },
    RoleMove { id: "b2bp", build: MoveStep::y(34.0), prog: MoveStep::y(54.0), final_third: MoveStep::y_x(74.0, XKey::Half) },
    RoleMove { id: "cm_ip", build: MoveStep::y(39.0), prog: MoveStep::y(52.0), final_third: MoveStep::y(66.0) },
    RoleMove { id: "mp", build: MoveStep::y(37.0), prog: MoveStep::y_x(50.0, XKey::Half), final_third: MoveStep::y_x(64.0, XKey::Half) },
    RoleMove { id: "wcm", build: MoveStep::y(41.0), prog: MoveStep::y_x(58.0, XKey::Wide2), final_third: MoveStep::y_x(73.0, XKey::Wide2) },
    RoleMove { id: "chm", build: MoveStep::y(42.0), prog: MoveStep::y_x(60.0, XKey::Ch), final_third: MoveStep::y_x(87.0, XKey::Ch) },
    RoleMove { id: "ap", build: MoveStep::y(47.0), prog: MoveStep::y_x(63.0, XKey::Half), final_third: MoveStep::y_x(76.0, XKey::Half) },
    RoleMove { id: "am_ip", build: MoveStep::y(49.0), prog: MoveStep::y(66.0), final_third: MoveStep::y_x(84.0, XKey::In) },
    RoleMove { id: "wm_ip", build: MoveStep::y_x(44.0, XKey::Wide), prog: MoveStep::y_x(60.0, XKey::Wide), final_third: MoveStep::y_x(73.0, XKey::Wide) },
    RoleMove { id: "w_ip", build: MoveStep::y_x(50.0, XKey::Wide), prog: MoveStep::y_x(71.0, XKey::Wide), final_third: MoveStep::y_x(92.0, XKey::Wide) },
    RoleMove { id: "iw_ip", build: MoveStep::y_x(50.0, XKey::Wide2), prog: MoveStep::y_x(66.0, XKey::Half), final_third: MoveStep::y_x(78.0, XKey::Half) },
    RoleMove { id: "pw", build: MoveStep::y_x(48.0, XKey::Wide2), prog: MoveStep::y_x(62.0, XKey::Half), final_third: MoveStep::y_x(73.0, XKey::Half) },
    RoleMove { id: "if_ip", build: MoveStep::y_x(52.0, XKey::Wide2), prog: MoveStep::y_x(71.0, XKey::Half), final_third: MoveStep::y_x(90.0, XKey::In2) },
    RoleMove { id: "wf_ip", build: MoveStep::y_x(52.0, XKey::Wide), prog: MoveStep::y_x(72.0, XKey::Wide2), final_third: MoveStep::y_x(90.0, XKey::Wide3) },
    RoleMove { id: "fr", build: MoveStep::y(50.0), prog: MoveStep::y_x(64.0, XKey::Ball), final_third: MoveStep::y_x(77.0, XKey::BallHalf) },
    RoleMove { id: "ss", build: MoveStep::y(56.0), prog: MoveStep::y(75.0), final_third: MoveStep::y_x(90.0, XKey::In) },
    RoleMove { id: "cf_ip", build: MoveStep::y(64.0), prog: MoveStep::y(78.0), final_third: MoveStep::y(90.0) },
    RoleMove { id: "chf", build: MoveStep::y(65.0), prog: MoveStep::y_x(80.0, XKey::Ch), final_third: MoveStep::y_x(92.0, XKey::Ch) },
    RoleMove { id: "dlf", build: MoveStep::y(64.0), prog: MoveStep::y(59.0), final_third: MoveStep::y(74.0) },
    RoleMove { id: "f9", build: MoveStep::y_x(63.0, XKey::C), prog: MoveStep::y_x(54.0, XKey::C), final_third: MoveStep::y_x(67.0, XKey::C) },
    RoleMove { id: "poa", build: MoveStep::y(68.0), prog: MoveStep::y(85.0), final_third: MoveStep::y(95.0) },
    RoleMove { id: "tf", build: MoveStep::y(65.0), prog: MoveStep::y_x(81.0, XKey::C), final_third: MoveStep::y_x(91.0, XKey::C) },
];

pub fn role_move(id: &str) -> Option<&'static RoleMove> {
    MOVES.iter().find(|m| m.id == id)
}

// ============================================================================
// Per-Role Phase Caption Notes (`MOVE_NOTES`)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveNote {
    pub id: &'static str,
    pub phase: SimPhase,
    pub pt: &'static str,
    pub en: &'static str,
}

pub const MOVE_NOTES: &[MoveNote] = &[
    MoveNote { id: "bpgk", phase: SimPhase::Build, pt: "sai da baliza para servir de apoio na construção", en: "comes off his line to support the build-up" },
    MoveNote { id: "acb", phase: SimPhase::Prog, pt: "sobe para o meio-campo", en: "steps into midfield" },
    MoveNote { id: "ocb", phase: SimPhase::Final, pt: "sobrepõe pelo corredor como um ala", en: "overlaps down the flank like a wing-back" },
    MoveNote { id: "ifb", phase: SimPhase::Build, pt: "fecha como terceiro central", en: "tucks in as a third centre-back" },
    MoveNote { id: "wb_ip", phase: SimPhase::Final, pt: "sobe pelo corredor e dá a largura", en: "pushes up the flank and gives width" },
    MoveNote { id: "iwb", phase: SimPhase::Build, pt: "entra para o meio, ao lado do médio defensivo", en: "moves inside, alongside the defensive midfielder" },
    MoveNote { id: "pwb", phase: SimPhase::Prog, pt: "entra para o meio e organiza", en: "moves inside and organises" },
    MoveNote { id: "awb", phase: SimPhase::Final, pt: "vai até à linha de fundo, quase como extremo", en: "goes all the way to the byline, almost like a winger" },
    MoveNote { id: "dlp", phase: SimPhase::Prog, pt: "fica atrás a distribuir o jogo", en: "stays back to distribute" },
    MoveNote { id: "hb", phase: SimPhase::Build, pt: "baixa entre os centrais (linha de 3)", en: "drops between the centre-backs (back 3)" },
    MoveNote { id: "b2b", phase: SimPhase::Final, pt: "chega à área em segunda vaga", en: "arrives in the box as a second wave" },
    MoveNote { id: "b2bp", phase: SimPhase::Final, pt: "sobe para criar entre linhas", en: "pushes up to create between the lines" },
    MoveNote { id: "wcm", phase: SimPhase::Prog, pt: "abre para o corredor a partir do meio", en: "moves out to the flank from midfield" },
    MoveNote { id: "chm", phase: SimPhase::Final, pt: "ataca o canal entre central e lateral", en: "attacks the channel between centre-back and full-back" },
    MoveNote { id: "ap", phase: SimPhase::Prog, pt: "procura espaço entre linhas", en: "looks for space between the lines" },
    MoveNote { id: "am_ip", phase: SimPhase::Final, pt: "ocupa a zona 14 e ameaça de fora", en: "occupies zone 14 and threatens from distance" },
    MoveNote { id: "w_ip", phase: SimPhase::Final, pt: "vai à linha de fundo para cruzar", en: "goes to the byline to cross" },
    MoveNote { id: "iw_ip", phase: SimPhase::Final, pt: "entra pelo meio-espaço para criar", en: "moves into the half-space to create" },
    MoveNote { id: "pw", phase: SimPhase::Prog, pt: "parte da ala e entra para organizar", en: "starts wide and comes inside to organise" },
    MoveNote { id: "if_ip", phase: SimPhase::Final, pt: "corta para dentro em direcção à baliza", en: "cuts inside towards goal" },
    MoveNote { id: "wf_ip", phase: SimPhase::Final, pt: "fica aberto e ataca o segundo poste", en: "stays wide and attacks the far post" },
    MoveNote { id: "fr", phase: SimPhase::Prog, pt: "vagueia para o lado da bola", en: "drifts to the ball side" },
    MoveNote { id: "ss", phase: SimPhase::Final, pt: "junta-se ao avançado na área", en: "joins the striker in the box" },
    MoveNote { id: "cf_ip", phase: SimPhase::Final, pt: "fixa os centrais e finaliza", en: "pins the centre-backs and finishes" },
    MoveNote { id: "chf", phase: SimPhase::Final, pt: "ataca o canal", en: "attacks the channel" },
    MoveNote { id: "dlf", phase: SimPhase::Final, pt: "baixa para ligar o jogo", en: "drops to link play" },
    MoveNote { id: "f9", phase: SimPhase::Prog, pt: "baixa até ao meio-campo e arrasta centrais", en: "drops into midfield and drags centre-backs out" },
    MoveNote { id: "poa", phase: SimPhase::Final, pt: "vive na última linha à espera da bola", en: "lives on the last line waiting for the ball" },
    MoveNote { id: "tf", phase: SimPhase::Final, pt: "referência aérea na área", en: "aerial target in the box" },
    MoveNote { id: "swk", phase: SimPhase::Oop, pt: "posiciona-se alto para varrer bolas nas costas", en: "positions himself high to sweep balls in behind" },
    MoveNote { id: "lhk", phase: SimPhase::Oop, pt: "fica na linha de golo", en: "stays on the goal line" },
    MoveNote { id: "ccb", phase: SimPhase::Oop, pt: "fica atrás da linha a cobrir a profundidade", en: "stays behind the line covering depth" },
    MoveNote { id: "scb", phase: SimPhase::Oop, pt: "sai da linha para apertar entre linhas", en: "steps out of the line to squeeze between the lines" },
    MoveNote { id: "cwcb", phase: SimPhase::Oop, pt: "cobre o canal, mais fundo", en: "covers the channel, deeper" },
    MoveNote { id: "swcb", phase: SimPhase::Oop, pt: "sai agressivamente ao meio-espaço", en: "steps out aggressively into the half-space" },
    MoveNote { id: "hfb", phase: SimPhase::Oop, pt: "fica fundo e estreito a proteger o central", en: "stays deep and narrow protecting the centre-back" },
    MoveNote { id: "pfb", phase: SimPhase::Oop, pt: "sobe a pressionar o extremo", en: "pushes up to press the winger" },
    MoveNote { id: "prwb", phase: SimPhase::Oop, pt: "sobe a pressionar", en: "pushes up to press" },
    MoveNote { id: "hwb", phase: SimPhase::Oop, pt: "mantém a linha de 5", en: "holds the back 5" },
    MoveNote { id: "ddm", phase: SimPhase::Oop, pt: "baixa para a linha de defesas", en: "drops into the defensive line" },
    MoveNote { id: "sdm", phase: SimPhase::Oop, pt: "fecha linhas de passe centrais", en: "closes central passing lanes" },
    MoveNote { id: "wcdm", phase: SimPhase::Oop, pt: "desloca-se para cobrir o flanco", en: "moves across to cover the flank" },
    MoveNote { id: "pdm", phase: SimPhase::Oop, pt: "sai a pressionar do médio defensivo", en: "steps out to press from defensive midfield" },
    MoveNote { id: "pcm", phase: SimPhase::Oop, pt: "sobe a pressionar", en: "pushes up to press" },
    MoveNote { id: "scm", phase: SimPhase::Oop, pt: "protege a zona central", en: "protects the central zone" },
    MoveNote { id: "wccm", phase: SimPhase::Oop, pt: "cobre o flanco do lateral", en: "covers the full-back’s flank" },
    MoveNote { id: "twm", phase: SimPhase::Oop, pt: "acompanha o lateral adversário até atrás", en: "tracks the opposing full-back all the way back" },
    MoveNote { id: "wowm", phase: SimPhase::Oop, pt: "fica alto e aberto para a saída", en: "stays high and wide for the outlet" },
    MoveNote { id: "tam", phase: SimPhase::Oop, pt: "recua para defender", en: "drops back to defend" },
    MoveNote { id: "coam", phase: SimPhase::Oop, pt: "fica alto e central para a saída", en: "stays high and central for the outlet" },
    MoveNote { id: "soam", phase: SimPhase::Oop, pt: "fica alto, desviado para o canal", en: "stays high, drifting into the channel" },
    MoveNote { id: "tw", phase: SimPhase::Oop, pt: "acompanha o recuo", en: "tracks back" },
    MoveNote { id: "wow", phase: SimPhase::Oop, pt: "fica aberto à espera do contra-ataque", en: "stays wide waiting for the counter" },
    MoveNote { id: "iow", phase: SimPhase::Oop, pt: "fica alto e por dentro", en: "stays high and inside" },
    MoveNote { id: "cocf", phase: SimPhase::Oop, pt: "fica como válvula de escape", en: "acts as an outlet" },
    MoveNote { id: "socf", phase: SimPhase::Oop, pt: "fica alto, desviado para o canal", en: "stays high, drifting into the channel" },
    MoveNote { id: "tcf", phase: SimPhase::Oop, pt: "recua e pressiona o médio adversário", en: "drops and presses the opposing midfielder" },
];

pub fn move_note(id: &str) -> Option<&'static MoveNote> {
    MOVE_NOTES.iter().find(|n| n.id == id)
}

// ============================================================================
// Ball Trajectory Interpolation (`simBallState`)
// ============================================================================

/// Interpolates ball position and elevation along a phase path for `bu` in `[0, 1]`.
pub fn sim_ball_state(path: &[BallPoint], bu: f32) -> BallSample {
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
    let seg = (path.len() - 1) as f32;
    let f = clamp(bu, 0.0, 1.0) * seg;
    let j = (f.floor() as usize).min(path.len() - 2);
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
        let raw_len = dx.hypot(dy);
        let len = if raw_len == 0.0 { 1.0 } else { raw_len };
        let h = (if kind == BallKind::Loft { 7.0 } else { 4.5 }) * 4.0 * r * (1.0 - r);
        return BallSample {
            x: clamp(bx + (dx / len) * h * 0.45, 2.0, 98.0),
            y: clamp(by - (dy / len) * h * 0.3, 1.0, 99.0),
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

// ============================================================================
// Internal Working Types & Helpers for `sim_model`
// ============================================================================

struct WorkPlayer {
    idx: usize,
    slot: SlotAssignment,
    a: SlotMeta,
    b: SlotMeta,
    ip: Option<&'static Role>,
    oop: Option<&'static Role>,
    sg: i8,
    sgb: i8,
    pressing: bool,
    pos: [SimPoint; SimPhase::COUNT],
}

impl WorkPlayer {
    #[inline]
    fn ip_id(&self) -> Option<&'static str> {
        self.ip.map(|r| r.id)
    }

    #[inline]
    fn oop_id(&self) -> Option<&'static str> {
        self.oop.map(|r| r.id)
    }

    #[inline]
    fn ip_has(&self, tag: RoleTag) -> bool {
        self.ip.map_or(false, |r| r.has_tag(tag))
    }

    #[inline]
    fn oop_has(&self, tag: RoleTag) -> bool {
        self.oop.map_or(false, |r| r.has_tag(tag))
    }

    #[inline]
    fn pos_at(&self, ph: SimPhase) -> SimPoint {
        self.pos[ph.index()]
    }

    #[inline]
    fn set_pos(&mut self, ph: SimPhase, pt: SimPoint) {
        self.pos[ph.index()] = pt;
    }
}

fn fallback_slot_meta(code: &str) -> SlotMeta {
    SlotMeta {
        code: code.to_string(),
        x: 50.0,
        y: 50.0,
        group: group_of(code),
        side: Side::C,
        wide_cb: false,
    }
}

#[inline]
fn side_or(p: &WorkPlayer, alt: i8) -> i8 {
    if p.sg != 0 {
        p.sg
    } else if p.a.x < 50.0 {
        -1
    } else if p.a.x > 50.0 {
        1
    } else {
        alt
    }
}

#[inline]
fn x_key(p: &WorkPlayer, key: XKey, w: f32, bs: i8, flank_side: i8) -> f32 {
    let s = side_or(p, flank_side) as f32;
    match key {
        XKey::Wide => clamp(50.0 + s * 44.0 * w, 5.0, 95.0),
        XKey::Wide2 => clamp(50.0 + s * 36.0 * w, 7.0, 93.0),
        XKey::Wide3 => clamp(50.0 + s * 29.0 * w, 9.0, 91.0),
        XKey::Half => clamp(50.0 + s * 18.0 * w.sqrt(), 14.0, 86.0),
        XKey::In => 50.0 + s * 9.0,
        XKey::In2 => 50.0 + s * 13.0,
        XKey::Ch => 50.0 + s * 23.0,
        XKey::Cbw => 50.0 + s * 29.0,
        XKey::C => 50.0,
        XKey::Ball => 50.0 + (bs as f32) * 15.0,
        XKey::BallHalf => 50.0 + (bs as f32) * 20.0,
    }
}

#[inline]
fn bp_at(
    players: &[WorkPlayer],
    idx: Option<usize>,
    ph: SimPhase,
    dx: f32,
    dy: f32,
    kind: BallKind,
) -> BallPoint {
    if let Some(i) = idx {
        let p = players[i].pos_at(ph);
        BallPoint {
            x: clamp(p.x + dx, 4.0, 96.0),
            y: clamp(p.y + dy, 2.0, 99.0),
            kind,
        }
    } else {
        BallPoint {
            x: 50.0,
            y: 50.0,
            kind,
        }
    }
}

fn dedup_ball(pts: Vec<BallPoint>) -> Vec<BallPoint> {
    let mut out = Vec::with_capacity(pts.len());
    for (idx, pt) in pts.iter().enumerate() {
        if idx == 0 {
            out.push(*pt);
        } else {
            let prev = pts[idx - 1];
            if (pt.x - prev.x).hypot(pt.y - prev.y) > 1.5 {
                out.push(*pt);
            }
        }
    }
    out
}

fn fwd_pts(pts: Vec<BallPoint>) -> Vec<BallPoint> {
    let mut acc: Vec<BallPoint> = Vec::with_capacity(pts.len());
    for pt in pts {
        if acc.last().map_or(true, |last| pt.y >= last.y - 6.0) {
            acc.push(pt);
        }
    }
    dedup_ball(acc)
}

#[inline]
fn arrow(from: SimPoint, to: SimPoint, color: &'static str, kind: SimArrowKind) -> SimArrow {
    SimArrow {
        from,
        to,
        color,
        kind,
        dim: false,
    }
}

#[inline]
fn zone_rect(
    kind: SimZoneKind,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    color: &'static str,
    stroke: &'static str,
    label: &str,
) -> SimZone {
    SimZone {
        kind,
        x1,
        x2,
        y1,
        y2,
        cx: 0.0,
        cy: 0.0,
        r: 0.0,
        color,
        stroke,
        label: label.to_string(),
    }
}

#[inline]
fn zone_circle(
    cx: f32,
    cy: f32,
    r: f32,
    color: &'static str,
    stroke: &'static str,
    label: &str,
) -> SimZone {
    SimZone {
        kind: SimZoneKind::Circle,
        x1: 0.0,
        x2: 0.0,
        y1: 0.0,
        y2: 0.0,
        cx,
        cy,
        r,
        color,
        stroke,
        label: label.to_string(),
    }
}

#[inline]
fn opp_at(k: usize, d: f32, f: f32) -> SimPoint {
    let [ox, oy] = OPP_SHAPE[k];
    let tt = clamp((oy - 18.0) / 60.0, 0.0, 1.0);
    if k == 0 {
        SimPoint::new(100.0 - ox, (d + 10.0).min(97.0))
    } else {
        SimPoint::new(100.0 - ox, d + (f - d) * tt)
    }
}

#[inline]
fn in_groups(g: PosGroup, gs: &[PosGroup]) -> bool {
    gs.contains(&g)
}

// ============================================================================
// Live 6-Phase Tactical Simulation (`sim_model`)
// ============================================================================

pub fn sim_model(t: &Tactic, lang: Lang) -> SimModel {
    let mi = t.ment.index() as f32;
    let ms = (mi - 3.0) * 1.8;

    let v = |k: &str| ti_val(t, k);
    let width_v = v("width").min(4);
    let w = [0.68_f32, 0.84, 1.0, 1.12, 1.24][width_v];
    let prog = v("progress"); // 0:Left, 1:Balanced, 2:Centre, 3:Right
    let bs: i8 = if prog == 3 {
        1
    } else if prog == 2 {
        0
    } else {
        -1
    };
    let flank_side: i8 = if bs != 0 { bs } else { -1 };
    let pass_v = v("passing");
    let build_v = v("buildup");
    let gk_kick = v("goal_kicks");
    let gk_tgt = v("gk_target");
    let rec_v = v("reception");
    let sup_v = v("support_runs");
    let drib_v = v("dribbling");
    let pat_v = v("patience");
    let ls_v = v("long_shots");
    let cross_v = v("crossing");
    let cre_v = v("creative");

    let mut players: Vec<WorkPlayer> = t
        .slots
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let a = slot_info(&t.ip_form, &s.ip_code)
                .unwrap_or_else(|| fallback_slot_meta(&s.ip_code));
            let b = slot_info(&t.oop_form, &s.oop_code)
                .unwrap_or_else(|| fallback_slot_meta(&s.oop_code));
            let sg: i8 = if a.x < 45.0 {
                -1
            } else if a.x > 55.0 {
                1
            } else {
                0
            };
            let sgb: i8 = if b.x < 45.0 {
                -1
            } else if b.x > 55.0 {
                1
            } else {
                0
            };
            WorkPlayer {
                idx,
                slot: s.clone(),
                a,
                b,
                ip: role_by_id(&s.ip),
                oop: role_by_id(&s.oop),
                sg,
                sgb,
                pressing: false,
                pos: [SimPoint::default(); SimPhase::COUNT],
            }
        })
        .collect();

    let has_hb = players.iter().any(|p| p.ip_id() == Some("hb"));
    let has_ddm = players.iter().any(|p| p.oop_id() == Some("ddm"));

    let mut arrows: [Vec<SimArrow>; SimPhase::COUNT] = Default::default();
    let mut zones: [Vec<SimZone>; SimPhase::COUNT] = Default::default();
    let mut lines: [Vec<SimLine>; SimPhase::COUNT] = Default::default();
    let mut ball: [Vec<BallPoint>; SimPhase::COUNT] = Default::default();

    // ---- In-possession phases (Build, Prog, Final)
    for ph in [SimPhase::Build, SimPhase::Prog, SimPhase::Final] {
        for p in &mut players {
            let mut x = 50.0 + (p.a.x - 50.0) * w;
            let mut y = match ph {
                SimPhase::Build => p.a.y * 0.6 + 3.0,
                SimPhase::Prog => p.a.y * 0.85 + 8.0,
                SimPhase::Final => (p.a.y * 0.75 + 25.0).min(96.0),
                _ => p.a.y,
            };

            if let Some(m) = p.ip_id().and_then(role_move).and_then(|rm| rm.for_phase(ph)) {
                if let Some(my) = m.y {
                    y = my;
                }
                if let Some(xs) = m.xs {
                    x = 50.0 + (p.a.x - 50.0) * xs * (if has_hb { 1.25 } else { 1.0 });
                }
                if let Some(xk) = m.x {
                    x = x_key(p, xk, w, bs, flank_side);
                }
            }

            // Build-up & Goal Kicks
            if ph == SimPhase::Build {
                if build_v == 0 {
                    // Play Through Press
                    if p.a.group == PosGroup::Gk {
                        y += 6.0;
                    }
                    if p.a.group == PosGroup::Dc {
                        x = 50.0 + (x - 50.0) * 1.24;
                    }
                    if p.a.group == PosGroup::Dm {
                        y -= 5.0;
                    }
                } else if build_v == 2 || gk_kick == 2 {
                    // Bypass Press / Long Goal Kicks
                    if p.a.group == PosGroup::Dc {
                        x = 50.0 + (x - 50.0) * 0.85;
                    }
                    if in_groups(p.a.group, &[PosGroup::Dm, PosGroup::Mc, PosGroup::MLr, PosGroup::Amc]) {
                        y += 9.0;
                    }
                    if p.a.group == PosGroup::St {
                        y += 7.0;
                    }
                }
                if gk_kick == 0 && p.a.group == PosGroup::Dc {
                    y -= 4.0;
                    x = 50.0 + (x - 50.0) * 1.15;
                }
            }

            // Passing directness & Pass reception
            if ph == SimPhase::Prog {
                if pass_v <= 1 && p.a.group != PosGroup::Gk {
                    let pull = if pass_v == 0 { 0.22 } else { 0.12 };
                    x += (50.0 + (bs as f32) * 18.0 - x) * pull;
                    if in_groups(p.a.group, &[PosGroup::St, PosGroup::Amc]) {
                        y -= if pass_v == 0 { 6.0 } else { 3.0 };
                    }
                }
                if pass_v >= 3 && p.a.group != PosGroup::Gk {
                    if in_groups(p.a.group, &[PosGroup::St, PosGroup::AmLr]) {
                        y += if pass_v == 4 { 9.0 } else { 5.0 };
                    }
                    if p.a.group == PosGroup::Dc {
                        y -= 3.0;
                    }
                }
                if rec_v == 0
                    && in_groups(
                        p.a.group,
                        &[PosGroup::St, PosGroup::AmLr, PosGroup::Amc, PosGroup::WbLr, PosGroup::DLr],
                    )
                {
                    y += 9.0; // Pass into Space
                }
                if rec_v == 2
                    && in_groups(
                        p.a.group,
                        &[
                            PosGroup::St,
                            PosGroup::AmLr,
                            PosGroup::Amc,
                            PosGroup::Mc,
                            PosGroup::WbLr,
                            PosGroup::DLr,
                        ],
                    )
                {
                    y -= 6.0; // Pass to Feet
                }
                if drib_v == 2
                    && in_groups(
                        p.a.group,
                        &[PosGroup::AmLr, PosGroup::MLr, PosGroup::WbLr, PosGroup::DLr],
                    )
                    && side_or(p, flank_side) == flank_side
                {
                    y += 11.0;
                }
                if prog == 2
                    && in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc, PosGroup::AmLr, PosGroup::MLr])
                {
                    x = 50.0 + (x - 50.0) * 0.72; // Focus Centre
                }
                if cre_v == 2
                    && (in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc, PosGroup::AmLr])
                        || p.ip_has(RoleTag::Cr))
                {
                    x += (50.0 + (flank_side as f32) * 16.0 - x) * 0.42;
                    y += 6.0;
                }
                if cre_v == 0 && in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc, PosGroup::AmLr]) {
                    x = p.a.x; // Disciplined: rigid lanes
                }
            }

            // Supporting runs (Overlap vs Underlap) in progression & final third
            if ph == SimPhase::Final || ph == SimPhase::Prog {
                let is_fb = in_groups(p.a.group, &[PosGroup::DLr, PosGroup::WbLr])
                    && !matches!(p.ip_id(), Some("ifb" | "iwb" | "pwb"));
                let is_wide_att = in_groups(p.a.group, &[PosGroup::AmLr, PosGroup::MLr]);
                if sup_v == 2 {
                    // Overlap: FB goes wide & high, winger tucks into half-space
                    if is_fb {
                        x = x_key(p, XKey::Wide, w, bs, flank_side);
                        y += if ph == SimPhase::Final { 11.0 } else { 7.0 };
                    }
                    if is_wide_att && ph == SimPhase::Final {
                        x = x_key(p, XKey::Half, w, bs, flank_side);
                    }
                } else if sup_v == 0 {
                    // Underlap: winger stays glued wide, FB cuts into half-space
                    if is_fb {
                        x = x_key(p, XKey::Half, w, bs, flank_side);
                        y += if ph == SimPhase::Final { 8.0 } else { 5.0 };
                    }
                    if is_wide_att {
                        x = x_key(p, XKey::Wide, w, bs, flank_side);
                    }
                }
            }

            // Final third patience / long shots
            if ph == SimPhase::Final {
                if pat_v == 0 && in_groups(p.a.group, &[PosGroup::DLr, PosGroup::WbLr]) {
                    y = clamp(y - 18.0, 54.0, 64.0);
                }
                if pat_v == 2 && in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc, PosGroup::AmLr]) {
                    y = if p.a.group == PosGroup::AmLr {
                        clamp(y - 4.0, 78.0, 86.0)
                    } else {
                        clamp(y + 4.0, 70.0, 84.0)
                    };
                    x = 50.0 + (x - 50.0) * 0.55;
                }
                if ls_v == 2 && in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc]) {
                    y = clamp(y, 68.0, 76.0);
                }
            }

            if p.a.group != PosGroup::Gk {
                y += ms;
            }
            p.set_pos(ph, SimPoint::new(clamp(x, 5.0, 95.0), clamp(y, 2.0, 96.0)));
        }
    }

    // ---- Ball paths & arrows in possession
    let by_group = |gs: &[PosGroup]| -> Vec<usize> {
        players
            .iter()
            .enumerate()
            .filter(|(_, p)| in_groups(p.a.group, gs))
            .map(|(i, _)| i)
            .collect()
    };

    let gk = players.iter().position(|p| p.a.group == PosGroup::Gk);

    let mut cbs = by_group(&[PosGroup::Dc]);
    cbs.sort_by(|&pi, &qi| {
        let target_x = 50.0 + (flank_side as f32) * 30.0;
        (players[pi].pos_at(SimPhase::Build).x - target_x)
            .abs()
            .total_cmp(&(players[qi].pos_at(SimPhase::Build).x - target_x).abs())
    });

    let mut fbs = by_group(&[PosGroup::DLr, PosGroup::WbLr]);
    fbs.sort_by_key(|&i| if side_or(&players[i], flank_side) == flank_side { 0 } else { 1 });

    let mut mids = by_group(&[PosGroup::Dm, PosGroup::Mc]);
    mids.sort_by(|&pi, &qi| {
        let pcr = i32::from(players[pi].ip_has(RoleTag::Cr));
        let qcr = i32::from(players[qi].ip_has(RoleTag::Cr));
        qcr.cmp(&pcr).then_with(|| {
            players[pi]
                .pos_at(SimPhase::Build)
                .y
                .total_cmp(&players[qi].pos_at(SimPhase::Build).y)
        })
    });

    let mut strikers = by_group(&[PosGroup::St, PosGroup::Amc]);
    strikers.sort_by(|&pi, &qi| {
        let pair = i32::from(players[pi].ip_has(RoleTag::Air));
        let qair = i32::from(players[qi].ip_has(RoleTag::Air));
        qair.cmp(&pair).then_with(|| {
            players[qi]
                .pos_at(SimPhase::Final)
                .y
                .total_cmp(&players[pi].pos_at(SimPhase::Final).y)
        })
    });

    let direct_build = build_v == 2 || gk_kick == 2 || gk_tgt == 4 || pass_v >= 4;
    let direct_prog = pass_v >= 3 || build_v == 2;
    let holder = mids.first().copied().or_else(|| cbs.first().copied()).or(gk);
    let mid2 = mids
        .get(1)
        .copied()
        .or_else(|| {
            by_group(&[PosGroup::Amc, PosGroup::Mc, PosGroup::Dm])
                .into_iter()
                .find(|&i| Some(i) != holder)
        })
        .or(holder);

    let mut side_wide: Vec<usize> = players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.a.group != PosGroup::Gk && side_or(p, flank_side) == flank_side)
        .map(|(i, _)| i)
        .collect();
    side_wide.sort_by(|&pi, &qi| {
        let pf = players[pi].pos_at(SimPhase::Final);
        let qf = players[qi].pos_at(SimPhase::Final);
        let p_wide = i32::from((pf.x - 50.0).abs() > 28.0 && pf.y > 58.0);
        let q_wide = i32::from((qf.x - 50.0).abs() > 28.0 && qf.y > 58.0);
        q_wide.cmp(&p_wide).then_with(|| qf.y.total_cmp(&pf.y))
    });
    let wide_prov = side_wide.first().copied();

    let mut box_cands: Vec<usize> = players
        .iter()
        .enumerate()
        .filter(|&(i, p)| {
            Some(i) != wide_prov
                && p.a.group != PosGroup::Gk
                && (p.ip_has(RoleTag::Box) || p.ip_has(RoleTag::Air) || p.a.group == PosGroup::St)
        })
        .map(|(i, _)| i)
        .collect();
    box_cands.sort_by(|&pi, &qi| {
        let pst = i32::from(players[pi].a.group == PosGroup::St);
        let qst = i32::from(players[qi].a.group == PosGroup::St);
        qst.cmp(&pst).then_with(|| {
            players[qi]
                .pos_at(SimPhase::Final)
                .y
                .total_cmp(&players[pi].pos_at(SimPhase::Final).y)
        })
    });
    let box_target = box_cands
        .first()
        .copied()
        .or_else(|| strikers.first().copied())
        .or(holder);

    let mut creator_cands: Vec<usize> = players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.ip_has(RoleTag::Cr) && p.a.group != PosGroup::Gk)
        .map(|(i, _)| i)
        .collect();
    creator_cands.sort_by(|&pi, &qi| {
        players[qi]
            .pos_at(SimPhase::Final)
            .y
            .total_cmp(&players[pi].pos_at(SimPhase::Final).y)
    });
    let creator = creator_cands.first().copied().or(mid2);

    let mut runner_cands: Vec<usize> = players
        .iter()
        .enumerate()
        .filter(|&(i, p)| Some(i) != wide_prov && p.ip_has(RoleTag::Run))
        .map(|(i, _)| i)
        .collect();
    runner_cands.sort_by(|&pi, &qi| {
        players[qi]
            .pos_at(SimPhase::Final)
            .y
            .total_cmp(&players[pi].pos_at(SimPhase::Final).y)
    });
    let runner = runner_cands.first().copied().or(box_target);

    let mut shooter_cands = by_group(&[PosGroup::Amc, PosGroup::Mc]);
    shooter_cands.sort_by(|&pi, &qi| {
        players[qi]
            .pos_at(SimPhase::Final)
            .y
            .total_cmp(&players[pi].pos_at(SimPhase::Final).y)
    });
    let shooter = shooter_cands.first().copied().or(creator);
    let early_crosser = fbs.first().copied().or(wide_prov);
    let cb0 = cbs.first().copied();
    let fb0 = fbs.first().copied();
    let st0 = strikers.first().copied();

    // Build-up ball path + track who ends with the ball (`build_rec`)
    let build_rec: Option<usize>;
    if build_v == 2 && gk_kick != 2 && gk_tgt != 4 {
        build_rec = wide_prov.or(st0).or(runner);
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Pass),
            bp_at(&players, cb0.or(holder), SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Loft),
        ]);
    } else if direct_build {
        build_rec = st0.or(runner);
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Loft),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Loft),
        ]);
    } else if gk_tgt == 1 && fb0.is_some() {
        build_rec = mid2;
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Pass),
            bp_at(&players, fb0, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
        ]);
    } else if gk_tgt == 3 && wide_prov.is_some() {
        build_rec = wide_prov;
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Loft),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Loft),
        ]);
    } else if build_v == 0 {
        build_rec = mid2.or(holder);
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Pass),
            bp_at(&players, cb0.or(holder), SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, holder, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
        ]);
    } else if pass_v <= 1 || gk_kick == 0 {
        build_rec = fb0.or(mid2);
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Pass),
            bp_at(&players, cb0.or(holder), SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
        ]);
    } else {
        build_rec = holder;
        ball[SimPhase::Build.index()] = dedup_ball(vec![
            bp_at(&players, gk, SimPhase::Build, 0.0, 1.5, BallKind::Pass),
            bp_at(&players, cb0.or(holder), SimPhase::Build, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, build_rec, SimPhase::Build, 0.0, 0.0, BallKind::Pass),
        ]);
    }
    if ball[SimPhase::Build.index()].len() < 2 {
        ball[SimPhase::Build.index()].push(bp_at(
            &players,
            holder,
            SimPhase::Build,
            2.0,
            2.0,
            BallKind::Pass,
        ));
    }

    // Progression ball path
    let mut prog_center_cands = by_group(&[PosGroup::Amc, PosGroup::Mc]);
    prog_center_cands.sort_by(|&pi, &qi| {
        let pp = players[pi].pos_at(SimPhase::Prog);
        let qp = players[qi].pos_at(SimPhase::Prog);
        (pp.x - 50.0)
            .abs()
            .total_cmp(&(qp.x - 50.0).abs())
            .then_with(|| qp.y.total_cmp(&pp.y))
    });
    let prog_center = prog_center_cands.first().copied().or(creator);

    let flank_mid = by_group(&[PosGroup::MLr, PosGroup::AmLr])
        .into_iter()
        .find(|&i| side_or(&players[i], flank_side) == flank_side);

    let mut pat_hub_cands = by_group(&[PosGroup::Amc, PosGroup::Mc]);
    pat_hub_cands.sort_by(|&pi, &qi| {
        let pf = players[pi].pos_at(SimPhase::Final);
        let qf = players[qi].pos_at(SimPhase::Final);
        (pf.x - 50.0)
            .abs()
            .total_cmp(&(qf.x - 50.0).abs())
            .then_with(|| qf.y.total_cmp(&pf.y))
    });
    let pat_hub = pat_hub_cands.first().copied().or(creator);

    let prog_target = if ls_v == 2 {
        shooter
    } else if pat_v == 0 {
        early_crosser
    } else if pat_v == 2 {
        pat_hub
    } else if prog == 2 {
        prog_center
    } else if sup_v != 1
        && flank_mid.is_some()
        && fb0.is_some()
        && players[fb0.unwrap()].pos_at(SimPhase::Final).y
            >= players[flank_mid.unwrap()].pos_at(SimPhase::Final).y - 6.0
    {
        flank_mid
    } else {
        wide_prov.or(prog_center).or(st0)
    };

    let prog_rec: Option<usize>;
    if direct_build && build_rec.is_some() && build_rec == st0.or(runner) {
        let br_idx = build_rec.unwrap();
        let br_y = players[br_idx].pos_at(SimPhase::Prog).y;
        let mut flicks: Vec<usize> = players
            .iter()
            .enumerate()
            .filter(|&(i, p)| {
                i != br_idx && p.a.group != PosGroup::Gk && p.pos_at(SimPhase::Prog).y >= br_y - 12.0
            })
            .map(|(i, _)| i)
            .collect();
        flicks.sort_by(|&pi, &qi| {
            players[qi]
                .pos_at(SimPhase::Prog)
                .y
                .total_cmp(&players[pi].pos_at(SimPhase::Prog).y)
        });
        if let Some(flick) = flicks.first().copied() {
            prog_rec = Some(flick);
            ball[SimPhase::Prog.index()] = dedup_ball(vec![
                bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass),
                bp_at(
                    &players,
                    prog_rec,
                    SimPhase::Prog,
                    0.0,
                    if rec_v == 0 { 5.0 } else { 0.0 },
                    BallKind::Pass,
                ),
            ]);
        } else {
            prog_rec = build_rec;
            ball[SimPhase::Prog.index()] = dedup_ball(vec![
                bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Dribble),
                bp_at(&players, build_rec, SimPhase::Prog, 0.0, 6.0, BallKind::Dribble),
            ]);
        }
    } else if direct_prog {
        prog_rec = if rec_v == 0 { runner } else { st0.or(runner) };
        let tgt = if rec_v == 0 {
            bp_at(&players, prog_rec, SimPhase::Prog, 0.0, 6.0, BallKind::Loft)
        } else {
            bp_at(&players, prog_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Loft)
        };
        ball[SimPhase::Prog.index()] = dedup_ball(vec![
            bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Loft),
            tgt,
        ]);
    } else if drib_v == 2 && prog_target.is_some() {
        prog_rec = prog_target;
        let pt_idx = prog_target.unwrap();
        let mid_pt = bp_at(&players, prog_target, SimPhase::Prog, 0.0, -14.0, BallKind::Dribble);
        let br_y = build_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
        let mut pts = vec![bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass)];
        if mid_pt.y >= br_y - 4.0 {
            pts.push(mid_pt);
        }
        pts.push(bp_at(
            &players,
            prog_target,
            SimPhase::Prog,
            0.0,
            2.0,
            BallKind::Dribble,
        ));
        ball[SimPhase::Prog.index()] = dedup_ball(pts);
        let ppos = players[pt_idx].pos_at(SimPhase::Prog);
        arrows[SimPhase::Prog.index()].push(arrow(
            SimPoint::new(ppos.x, (ppos.y - 14.0).max(20.0)),
            ppos,
            "#ffe27a",
            SimArrowKind::Run,
        ));
    } else if cre_v == 2 && creator.is_some() && runner.is_some() && creator != build_rec {
        prog_rec = runner;
        let br_y = build_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
        let cr_y = players[creator.unwrap()].pos_at(SimPhase::Prog).y;
        let mut pts = vec![bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass)];
        if cr_y >= br_y - 2.0 {
            pts.push(bp_at(&players, creator, SimPhase::Prog, 0.0, 0.0, BallKind::Pass));
        }
        pts.push(bp_at(&players, runner, SimPhase::Prog, 0.0, 4.0, BallKind::Curve));
        ball[SimPhase::Prog.index()] = dedup_ball(pts);
    } else if drib_v == 0 && mid2.is_some() && mid2 != build_rec && mid2 != prog_target {
        prog_rec = prog_target.or(mid2);
        let mut pts = vec![bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass)];
        let m2_y = players[mid2.unwrap()].pos_at(SimPhase::Prog).y;
        let br_y = build_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
        let pr_y = prog_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
        if m2_y >= br_y - 2.0 && m2_y <= pr_y + 2.0 {
            pts.push(bp_at(&players, mid2, SimPhase::Prog, 0.0, 0.0, BallKind::Pass));
        }
        pts.push(bp_at(&players, prog_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass));
        ball[SimPhase::Prog.index()] = dedup_ball(pts);
    } else if pass_v <= 1 {
        prog_rec = prog_target.or(mid2);
        let mut pts = vec![bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass)];
        if let Some(m2) = mid2 {
            if Some(m2) != build_rec && Some(m2) != prog_rec {
                let m2_y = players[m2].pos_at(SimPhase::Prog).y;
                let br_y = build_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
                let pr_y = prog_rec.map_or(50.0, |i| players[i].pos_at(SimPhase::Prog).y);
                if m2_y >= br_y - 2.0 && m2_y <= pr_y + 2.0 {
                    pts.push(bp_at(&players, Some(m2), SimPhase::Prog, 0.0, 0.0, BallKind::Pass));
                }
            }
        }
        pts.push(bp_at(
            &players,
            prog_rec,
            SimPhase::Prog,
            0.0,
            if rec_v == 0 { 10.0 } else { 0.0 },
            BallKind::Pass,
        ));
        ball[SimPhase::Prog.index()] = dedup_ball(pts);
    } else {
        prog_rec = prog_target.or(st0).or(holder);
        ball[SimPhase::Prog.index()] = dedup_ball(vec![
            bp_at(&players, build_rec, SimPhase::Prog, 0.0, 0.0, BallKind::Pass),
            bp_at(
                &players,
                prog_rec,
                SimPhase::Prog,
                0.0,
                if rec_v == 0 { 10.0 } else { 0.0 },
                BallKind::Pass,
            ),
        ]);
    }
    if ball[SimPhase::Prog.index()].len() < 2 {
        ball[SimPhase::Prog.index()].push(bp_at(
            &players,
            prog_rec,
            SimPhase::Prog,
            0.0,
            6.0,
            BallKind::Pass,
        ));
    }

    // Final third ball path: strictly forward from `prog_rec`
    let f_start = bp_at(&players, prog_rec, SimPhase::Final, 0.0, 0.0, BallKind::Pass);
    let shot_goal = BallPoint {
        x: 50.0,
        y: 98.0,
        kind: BallKind::Shot,
    };
    if ls_v == 2 {
        ball[SimPhase::Final.index()] = fwd_pts(vec![
            f_start,
            bp_at(&players, shooter, SimPhase::Final, 0.0, 0.0, BallKind::Shot),
            shot_goal,
        ]);
    } else if pat_v == 0 {
        let cross_from = bp_at(&players, early_crosser, SimPhase::Final, 0.0, 0.0, BallKind::Loft);
        let first = if prog_rec == early_crosser { cross_from } else { f_start };
        ball[SimPhase::Final.index()] = fwd_pts(vec![
            first,
            cross_from,
            bp_at(&players, box_target, SimPhase::Final, 0.0, 2.0, BallKind::Loft),
            shot_goal,
        ]);
    } else if pat_v == 2 {
        let mut central_links: Vec<usize> = players
            .iter()
            .enumerate()
            .filter(|&(i, p)| {
                let pf = p.pos_at(SimPhase::Final);
                Some(i) != prog_rec
                    && Some(i) != box_target
                    && p.a.group != PosGroup::Gk
                    && (pf.x - 50.0).abs() <= 30.0
                    && pf.y >= f_start.y + 2.0
                    && pf.y <= 94.0
            })
            .map(|(i, _)| i)
            .collect();
        central_links.sort_by(|&pi, &qi| {
            players[pi]
                .pos_at(SimPhase::Final)
                .y
                .total_cmp(&players[qi].pos_at(SimPhase::Final).y)
        });
        let l1 = central_links.first().copied();
        let l2 = central_links.iter().copied().find(|&i| {
            Some(i) != l1
                && players[i].pos_at(SimPhase::Final).y
                    >= l1.map_or(f_start.y, |j| players[j].pos_at(SimPhase::Final).y + 2.0)
        });
        let mut pts = vec![f_start];
        if let Some(i1) = l1 {
            pts.push(bp_at(&players, Some(i1), SimPhase::Final, 0.0, 0.0, BallKind::Pass));
        }
        if let Some(i2) = l2 {
            pts.push(bp_at(&players, Some(i2), SimPhase::Final, 0.0, 0.0, BallKind::Pass));
        }
        pts.push(bp_at(&players, box_target, SimPhase::Final, 0.0, 1.0, BallKind::Pass));
        pts.push(shot_goal);
        ball[SimPhase::Final.index()] = fwd_pts(pts);
    } else if prog == 2 {
        let c1 = creator.or(holder);
        let c2 = box_target.or(runner);
        ball[SimPhase::Final.index()] = fwd_pts(vec![
            f_start,
            bp_at(&players, c1, SimPhase::Final, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, c2, SimPhase::Final, 0.0, 2.0, BallKind::Shot),
            shot_goal,
        ]);
    } else if wide_prov.is_some() {
        let cross_kind = if cross_v == 3 {
            BallKind::Loft
        } else if cross_v == 2 {
            BallKind::Curve
        } else {
            BallKind::Pass
        };
        let box_pt = if cross_v == 0 {
            bp_at(
                &players,
                box_target,
                SimPhase::Final,
                -(flank_side as f32) * 4.0,
                -4.0,
                BallKind::Pass,
            )
        } else if cross_v == 3 {
            bp_at(
                &players,
                box_target,
                SimPhase::Final,
                -(flank_side as f32) * 9.0,
                2.0,
                BallKind::Loft,
            )
        } else {
            bp_at(&players, box_target, SimPhase::Final, 0.0, 1.0, cross_kind)
        };
        let crosser = if sup_v != 1 && fb0.is_some() && prog_rec == flank_mid {
            fb0
        } else {
            wide_prov
        };
        ball[SimPhase::Final.index()] = fwd_pts(vec![
            f_start,
            bp_at(&players, crosser, SimPhase::Final, 0.0, 0.0, cross_kind),
            box_pt,
        ]);
    } else {
        ball[SimPhase::Final.index()] = fwd_pts(vec![
            f_start,
            bp_at(&players, creator.or(holder), SimPhase::Final, 0.0, 0.0, BallKind::Pass),
            bp_at(&players, runner, SimPhase::Final, 0.0, 2.0, BallKind::Shot),
            shot_goal,
        ]);
    }
    if ball[SimPhase::Final.index()].len() < 2 {
        ball[SimPhase::Final.index()].push(shot_goal);
    }
    let last_final = *ball[SimPhase::Final.index()].last().unwrap();
    let lost = SimPoint::new(clamp(last_final.x, 8.0, 92.0), clamp(last_final.y, 8.0, 95.0));

    // IP tactical arrows & zones
    for p in &players {
        let id = p.ip_id();
        if id == Some("hb") {
            arrows[SimPhase::Build.index()].push(arrow(
                SimPoint::new(50.0, 28.0),
                p.pos_at(SimPhase::Build),
                "#7dd3fc",
                SimArrowKind::Cover,
            ));
        }
        if id == Some("ifb") {
            arrows[SimPhase::Build.index()].push(arrow(
                SimPoint::new(50.0 + (p.sg as f32) * 40.0, 18.0),
                p.pos_at(SimPhase::Build),
                "#7dd3fc",
                SimArrowKind::Cover,
            ));
        }
        if matches!(id, Some("iwb" | "pwb")) {
            arrows[SimPhase::Build.index()].push(arrow(
                SimPoint::new(50.0 + (p.sg as f32) * 40.0, 22.0),
                p.pos_at(SimPhase::Build),
                "#ffe27a",
                SimArrowKind::Run,
            ));
        }
        if matches!(id, Some("f9" | "dlf")) {
            arrows[SimPhase::Prog.index()].push(arrow(
                p.pos_at(SimPhase::Build),
                p.pos_at(SimPhase::Prog),
                "#ffe27a",
                SimArrowKind::Run,
            ));
        }
        if matches!(
            id,
            Some("acb" | "ocb" | "wcm" | "chm" | "b2b" | "if_ip" | "awb" | "ss" | "chf")
        ) {
            arrows[SimPhase::Final.index()].push(arrow(
                p.pos_at(SimPhase::Prog),
                p.pos_at(SimPhase::Final),
                "#ffe27a",
                SimArrowKind::Run,
            ));
        }
        if (sup_v == 2 || sup_v == 0)
            && in_groups(p.a.group, &[PosGroup::DLr, PosGroup::WbLr])
            && !matches!(id, Some("ifb" | "iwb" | "pwb"))
        {
            arrows[SimPhase::Final.index()].push(arrow(
                p.pos_at(SimPhase::Prog),
                p.pos_at(SimPhase::Final),
                "#ffe27a",
                SimArrowKind::Run,
            ));
        }
    }

    if build_v == 0 {
        zones[SimPhase::Build.index()].push(zone_rect(
            SimZoneKind::Trap,
            22.0,
            78.0,
            10.0,
            34.0,
            "rgba(125,211,252,.14)",
            "rgba(125,211,252,.65)",
            lang.pick("Sair pelo meio", "Play through press"),
        ));
    }
    if build_v == 2 {
        zones[SimPhase::Build.index()].push(zone_rect(
            SimZoneKind::Lane,
            20.0,
            80.0,
            30.0,
            64.0,
            "rgba(255,226,122,.12)",
            "rgba(255,226,122,.6)",
            lang.pick("Saltar a pressão", "Bypass press"),
        ));
    }
    if rec_v == 0 {
        if let Some(pr) = prog_rec {
            let sp = bp_at(&players, Some(pr), SimPhase::Prog, 0.0, 10.0, BallKind::Pass);
            arrows[SimPhase::Prog.index()].push(arrow(
                players[pr].pos_at(SimPhase::Prog),
                SimPoint::new(sp.x, sp.y),
                "#ffe27a",
                SimArrowKind::Run,
            ));
            zones[SimPhase::Prog.index()].push(zone_circle(
                sp.x,
                sp.y,
                9.0,
                "rgba(255,226,122,.2)",
                "rgba(255,226,122,.75)",
                lang.pick("Espaço livre", "Into space"),
            ));
        }
    }
    if rec_v == 2 {
        if let Some(pr) = prog_rec {
            let pp = players[pr].pos_at(SimPhase::Prog);
            zones[SimPhase::Prog.index()].push(zone_circle(
                pp.x,
                pp.y,
                8.0,
                "rgba(125,211,252,.2)",
                "rgba(125,211,252,.75)",
                lang.pick("Passe ao pé", "To feet"),
            ));
        }
    }
    if drib_v == 2 {
        if let Some(pt) = prog_target {
            let pp = players[pt].pos_at(SimPhase::Prog);
            zones[SimPhase::Prog.index()].push(zone_rect(
                SimZoneKind::Lane,
                clamp(pp.x - 10.0, 2.0, 80.0),
                clamp(pp.x + 10.0, 20.0, 98.0),
                (pp.y - 16.0).max(24.0),
                (pp.y + 4.0).min(92.0),
                "rgba(255,226,122,.15)",
                "rgba(255,226,122,.7)",
                lang.pick("Conduzir 1×1", "Run at defence"),
            ));
        }
    }
    if cre_v == 2 {
        for p in &players {
            if in_groups(p.a.group, &[PosGroup::Mc, PosGroup::Amc, PosGroup::AmLr]) {
                arrows[SimPhase::Prog.index()].push(arrow(
                    p.pos_at(SimPhase::Build),
                    p.pos_at(SimPhase::Prog),
                    "#ffe27a",
                    SimArrowKind::Run,
                ));
            }
        }
        zones[SimPhase::Prog.index()].push(zone_rect(
            SimZoneKind::Trap,
            24.0,
            76.0,
            42.0,
            78.0,
            "rgba(255,226,122,.12)",
            "rgba(255,226,122,.65)",
            lang.pick("Mobilidade livre", "Roam & create"),
        ));
    }
    if cre_v == 0 {
        zones[SimPhase::Prog.index()].push(zone_rect(
            SimZoneKind::Lane,
            14.0,
            86.0,
            38.0,
            72.0,
            "rgba(125,211,252,.1)",
            "rgba(125,211,252,.55)",
            lang.pick("Posições rígidas", "Hold lanes"),
        ));
    }
    if prog == 0 {
        zones[SimPhase::Prog.index()].push(zone_rect(
            SimZoneKind::Lane,
            3.0,
            32.0,
            28.0,
            88.0,
            "rgba(255,226,122,.13)",
            "rgba(255,226,122,.45)",
            lang.pick("Corredor esq.", "Left flank"),
        ));
    }
    if prog == 3 {
        zones[SimPhase::Prog.index()].push(zone_rect(
            SimZoneKind::Lane,
            68.0,
            97.0,
            28.0,
            88.0,
            "rgba(255,226,122,.13)",
            "rgba(255,226,122,.45)",
            lang.pick("Corredor dir.", "Right flank"),
        ));
    }
    if prog == 2 {
        zones[SimPhase::Prog.index()].push(zone_rect(
            SimZoneKind::Lane,
            31.0,
            69.0,
            28.0,
            88.0,
            "rgba(255,226,122,.13)",
            "rgba(255,226,122,.45)",
            lang.pick("Zona central", "Centre"),
        ));
    }
    if width_v <= 1 {
        zones[SimPhase::Final.index()].push(zone_rect(
            SimZoneKind::Lane,
            22.0,
            78.0,
            50.0,
            95.0,
            "rgba(125,211,252,.08)",
            "rgba(125,211,252,.35)",
            lang.pick("Largura estreita", "Narrow width"),
        ));
    }
    if width_v >= 3 {
        zones[SimPhase::Final.index()].push(zone_rect(
            SimZoneKind::Lane,
            4.0,
            96.0,
            50.0,
            95.0,
            "rgba(255,226,122,.08)",
            "rgba(255,226,122,.35)",
            lang.pick("Largura máxima", "Wide stretch"),
        ));
    }
    if pat_v == 0 {
        let (x1, x2) = if flank_side < 0 { (2.0, 20.0) } else { (80.0, 98.0) };
        zones[SimPhase::Final.index()].push(zone_rect(
            SimZoneKind::Lane,
            x1,
            x2,
            52.0,
            68.0,
            "rgba(255,226,122,.16)",
            "rgba(255,226,122,.72)",
            lang.pick("Cruzamento cedo", "Early cross"),
        ));
    }
    if pat_v == 2 {
        zones[SimPhase::Final.index()].push(zone_rect(
            SimZoneKind::Trap,
            26.0,
            74.0,
            76.0,
            96.0,
            "rgba(125,211,252,.15)",
            "rgba(125,211,252,.7)",
            lang.pick("Entrar na área", "Work into box"),
        ));
    }

    // ---- Out-of-possession block (`SimPhase::Oop`)
    let dl = v("dline").min(4);
    let loe = v("loe").min(2);
    let trig = v("trigger").min(4);
    let dlb = v("dl_behaviour");
    let cross_eng = v("cross_eng");
    let short_gk = v("short_gk");
    let d_line = clamp(
        [14.0_f32, 20.0, 27.0, 34.0, 41.0][dl] + ms * 0.55 + ((dlb as f32) - 1.0) * 4.5,
        10.0,
        48.0,
    );
    let front = clamp(
        [42.0_f32, 57.0, 72.0][loe] + ms * 0.55,
        d_line + 18.0,
        82.0,
    );
    let trap = v("press_trap"); // 0: Inside, 1: Balanced, 2: Outside
    let opp_front = d_line + 8.0;
    let opp_back = (front + 14.0).max(82.0);
    let opp_gk = opp_at(0, opp_back, opp_front);
    let opp_start = if short_gk == 1 {
        opp_gk
    } else {
        opp_at(2, opp_back, opp_front)
    };
    let ball_oop_end = if short_gk == 1 {
        BallPoint {
            x: 50.0 + (flank_side as f32) * 18.0,
            y: clamp(d_line + 24.0, 38.0, 56.0),
            kind: BallKind::Loft,
        }
    } else if trap == 2 {
        BallPoint {
            x: 50.0 + (flank_side as f32) * 40.0,
            y: front + 2.0,
            kind: BallKind::Opp,
        }
    } else if trap == 0 {
        BallPoint {
            x: 50.0,
            y: front + 1.0,
            kind: BallKind::Opp,
        }
    } else if cross_eng == 2 {
        BallPoint {
            x: 50.0 + (flank_side as f32) * 42.0,
            y: d_line + 12.0,
            kind: BallKind::Opp,
        }
    } else {
        BallPoint {
            x: 50.0 + (flank_side as f32) * 22.0,
            y: front + 3.0,
            kind: BallKind::Opp,
        }
    };
    let bxo = ball_oop_end.x;

    for p in &mut players {
        let b = &p.b;
        let mut x = 50.0 + (b.x - 50.0) * 0.8 + (bxo - 50.0) * 0.22;
        let mut y = if b.group == PosGroup::Gk {
            5.0 + if dl >= 3 { 4.0 } else { 0.0 }
        } else {
            let tt = clamp((b.y - 18.0) / 62.0, 0.0, 1.0);
            d_line + (front - d_line) * tt
        };

        let id = p.oop_id();
        let sgb_or = |fallback: i8| -> f32 {
            if p.sgb != 0 {
                p.sgb as f32
            } else {
                fallback as f32
            }
        };

        if id == Some("swk") {
            y = (d_line - 11.0).max(9.0);
        }
        if id == Some("lhk") {
            y = 3.0;
        }
        if has_ddm && b.group == PosGroup::Dc {
            let dir = if p.sgb != 0 {
                p.sgb as f32
            } else if b.x < 50.0 {
                -1.0
            } else {
                1.0
            };
            x = 50.0 + dir * 22.0;
        }
        if p.oop_has(RoleTag::Hp) {
            y += 5.0;
        }
        if p.oop_has(RoleTag::Low) {
            y -= 4.0;
        }
        if p.oop_has(RoleTag::Trk) {
            y -= 6.0;
        }
        if matches!(id, Some("pfb" | "prwb")) {
            y += 7.0;
        }
        if id == Some("hfb") {
            x = 50.0 + sgb_or(flank_side) * 26.0;
        }
        if id == Some("ddm") {
            y = d_line;
            x = 50.0;
        }
        if matches!(id, Some("wcdm" | "wccm")) {
            x = 50.0 + sgb_or(-flank_side) * 32.0;
        }
        if matches!(id, Some("wowm" | "wow")) {
            y = y.max(front + 6.0);
            x = 50.0 + sgb_or(flank_side) * 42.0;
        }
        if id == Some("iow") {
            y = y.max(front + 6.0);
            x = 50.0 + sgb_or(flank_side) * 20.0;
        }
        if matches!(id, Some("coam" | "cocf")) {
            y = y.max(front + 8.0);
            x = 50.0;
        }
        if matches!(id, Some("soam" | "socf")) {
            y = y.max(front + 9.0);
            x = 50.0 + sgb_or(flank_side) * 26.0;
        }
        if id == Some("tcf") {
            y = front - 2.0;
        }
        if short_gk == 1 && in_groups(b.group, &[PosGroup::St, PosGroup::AmLr, PosGroup::Amc]) {
            y = y.max(if b.group == PosGroup::St { 84.0 } else { 76.0 });
        }
        if trap == 0 && in_groups(b.group, &[PosGroup::AmLr, PosGroup::MLr]) {
            x = 50.0 + (x - 50.0) * 1.15;
        }
        if trap == 2 && in_groups(b.group, &[PosGroup::St, PosGroup::Amc]) {
            x = 50.0 - (flank_side as f32) * 6.0;
        }
        if cross_eng == 0 && b.group == PosGroup::DLr {
            x = 50.0 + sgb_or(flank_side) * 25.0;
        }
        if cross_eng == 2 && b.group == PosGroup::DLr && p.sgb == flank_side {
            x = 50.0 + (flank_side as f32) * 43.0;
            y += 7.0;
        }
        if b.group != PosGroup::Gk {
            let min_off = match b.group {
                PosGroup::Dc | PosGroup::DLr | PosGroup::WbLr => 0.0,
                PosGroup::Dm => 7.0,
                PosGroup::Mc | PosGroup::MLr => 10.0,
                PosGroup::Amc => 15.0,
                PosGroup::AmLr => 14.0,
                PosGroup::St => 19.0,
                PosGroup::Gk => 0.0,
            };
            y = y.max(d_line + min_off - if p.oop_has(RoleTag::Trk) { 4.0 } else { 0.0 });
            if b.group == PosGroup::MLr || b.group == PosGroup::AmLr {
                x = 50.0 + (x - 50.0) * 0.94;
            }
        }
        p.set_pos(
            SimPhase::Oop,
            SimPoint::new(clamp(x, 5.0, 95.0), clamp(y, 2.0, 96.0)),
        );
    }

    // Pressing: nearest players (or HP roles within 32) step out toward the ball carrier
    let n_press = [0_usize, 1, 2, 3, 4][trig];
    let mut cand: Vec<(usize, f32)> = players
        .iter()
        .filter(|p| p.b.group != PosGroup::Gk && !p.oop_has(RoleTag::Out))
        .map(|p| {
            let po = p.pos_at(SimPhase::Oop);
            (p.idx, (po.x - bxo).hypot(po.y - ball_oop_end.y))
        })
        .collect();
    cand.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut pressers: HashSet<usize> = cand
        .into_iter()
        .filter(|&(i, _)| players[i].pos_at(SimPhase::Oop).y > d_line + 5.0)
        .take(n_press)
        .map(|(i, _)| i)
        .collect();
    for p in &players {
        if p.oop_has(RoleTag::Hp) {
            let po = p.pos_at(SimPhase::Oop);
            if (po.x - bxo).hypot(po.y - ball_oop_end.y) < 32.0 {
                pressers.insert(p.idx);
            }
        }
    }
    for p in &mut players {
        if pressers.contains(&p.idx) {
            let o = p.pos_at(SimPhase::Oop);
            let to = SimPoint::new(
                clamp(o.x + (bxo - o.x) * 0.56, 5.0, 95.0),
                clamp(o.y + (ball_oop_end.y - 3.0 - o.y) * 0.56, 4.0, 95.0),
            );
            arrows[SimPhase::Oop.index()].push(arrow(o, to, "#ff6b5a", SimArrowKind::Press));
            p.set_pos(SimPhase::Oop, to);
            p.pressing = true;
        }
    }

    // OOP tactical lines & zones
    lines[SimPhase::Oop.index()].push(SimLine {
        y: d_line,
        color: "rgba(125,211,252,.72)",
        label: lang.pick("Linha def.", "Def. line").to_string(),
    });
    lines[SimPhase::Oop.index()].push(SimLine {
        y: front,
        color: "rgba(255,145,90,.72)",
        label: lang.pick("Pressão", "Engagement").to_string(),
    });
    if dlb == 2 {
        for p in &players {
            if p.b.group == PosGroup::Dc || p.b.group == PosGroup::DLr {
                let po = p.pos_at(SimPhase::Oop);
                arrows[SimPhase::Oop.index()].push(arrow(
                    SimPoint::new(po.x, po.y - 7.0),
                    po,
                    "#ff6b5a",
                    SimArrowKind::Press,
                ));
            }
        }
        zones[SimPhase::Oop.index()].push(zone_rect(
            SimZoneKind::Lane,
            8.0,
            92.0,
            d_line - 2.0,
            d_line + 5.0,
            "rgba(255,107,90,.16)",
            "rgba(255,107,90,.72)",
            lang.pick("Subir / fora-de-jogo", "Step up / offside"),
        ));
    }
    if dlb == 0 {
        for p in &players {
            if p.b.group == PosGroup::Dc || p.b.group == PosGroup::DLr {
                let po = p.pos_at(SimPhase::Oop);
                arrows[SimPhase::Oop.index()].push(arrow(
                    SimPoint::new(po.x, po.y + 7.0),
                    po,
                    "#7dd3fc",
                    SimArrowKind::Cover,
                ));
            }
        }
        zones[SimPhase::Oop.index()].push(zone_rect(
            SimZoneKind::Lane,
            8.0,
            92.0,
            (d_line - 10.0).max(6.0),
            d_line,
            "rgba(125,211,252,.16)",
            "rgba(125,211,252,.72)",
            lang.pick("Proteger costas", "Protect behind"),
        ));
    }
    if trap == 0 {
        zones[SimPhase::Oop.index()].push(zone_rect(
            SimZoneKind::Trap,
            32.0,
            68.0,
            (front - 12.0).max(d_line + 6.0),
            front + 8.0,
            "rgba(255,107,90,.18)",
            "rgba(255,107,90,.68)",
            lang.pick("Armadilha interior", "Trap inside"),
        ));
    }
    if trap == 2 {
        let (x1, x2) = if flank_side < 0 { (3.0, 28.0) } else { (72.0, 97.0) };
        zones[SimPhase::Oop.index()].push(zone_rect(
            SimZoneKind::Trap,
            x1,
            x2,
            (front - 10.0).max(d_line + 6.0),
            front + 10.0,
            "rgba(255,107,90,.18)",
            "rgba(255,107,90,.68)",
            lang.pick("Armadilha fora", "Trap outside"),
        ));
    }
    if short_gk == 1 {
        for p in &players {
            if in_groups(p.b.group, &[PosGroup::St, PosGroup::AmLr, PosGroup::Amc]) {
                let po = p.pos_at(SimPhase::Oop);
                arrows[SimPhase::Oop.index()].push(arrow(
                    SimPoint::new(po.x, po.y - 9.0),
                    po,
                    "#ff6b5a",
                    SimArrowKind::Press,
                ));
            }
        }
        zones[SimPhase::Oop.index()].push(zone_rect(
            SimZoneKind::Trap,
            16.0,
            84.0,
            74.0,
            96.0,
            "rgba(255,107,90,.16)",
            "rgba(255,107,90,.72)",
            lang.pick("Forçar pontapé longo", "Force long kick"),
        ));
    }

    // ---- Ball lost (`SimPhase::Loss`)
    let dtr = v("def_trans");
    for p in &mut players {
        let f = p.pos_at(SimPhase::Final);
        let o = p.pos_at(SimPhase::Oop);
        let d = (f.x - lost.x).hypot(f.y - lost.y);
        let pt = if dtr == 2 && d < 30.0 && p.a.group != PosGroup::Gk {
            let pt = SimPoint::new(
                f.x + (lost.x - f.x) * 0.72,
                f.y + (lost.y - 4.0 - f.y) * 0.72,
            );
            arrows[SimPhase::Loss.index()].push(arrow(f, pt, "#ff6b5a", SimArrowKind::Press));
            pt
        } else if dtr == 2 {
            SimPoint::new(f.x, clamp(f.y - 4.0, 4.0, 95.0))
        } else if dtr == 0 {
            let pt = SimPoint::new(f.x + (o.x - f.x) * 0.8, f.y + (o.y - f.y) * 0.8);
            if p.a.group != PosGroup::Gk {
                arrows[SimPhase::Loss.index()].push(arrow(f, pt, "#7dd3fc", SimArrowKind::Cover));
            }
            pt
        } else if d < 18.0 && p.a.group != PosGroup::Gk {
            SimPoint::new(f.x + (lost.x - f.x) * 0.6, f.y + (lost.y - f.y) * 0.6)
        } else {
            SimPoint::new(f.x + (o.x - f.x) * 0.45, f.y + (o.y - f.y) * 0.45)
        };
        p.set_pos(
            SimPhase::Loss,
            SimPoint::new(clamp(pt.x, 5.0, 95.0), clamp(pt.y, 2.0, 96.0)),
        );
    }
    if dtr == 2 {
        zones[SimPhase::Loss.index()].push(zone_circle(
            lost.x,
            lost.y,
            16.0,
            "rgba(255,107,90,.16)",
            "rgba(255,107,90,.7)",
            lang.pick("Contra-pressão", "Counter-press"),
        ));
    }
    let loss_end = BallPoint {
        x: clamp(lost.x - (flank_side as f32) * 7.0, 8.0, 92.0),
        y: clamp(
            lost.y - if dtr == 0 { 22.0 } else { 9.0 },
            (front + 4.0).max(24.0),
            92.0,
        ),
        kind: BallKind::Opp,
    };
    ball[SimPhase::Loss.index()] = vec![
        BallPoint {
            x: lost.x,
            y: lost.y,
            kind: BallKind::Opp,
        },
        loss_end,
    ];
    // Chain loss -> oop smoothly instead of teleporting to an unrelated opponent CB
    let opp_start_bp = BallPoint {
        x: opp_start.x,
        y: opp_start.y,
        kind: BallKind::Opp,
    };
    let mut oop_ball = dedup_ball(vec![loss_end, opp_start_bp, ball_oop_end]);
    if oop_ball.len() < 2 {
        oop_ball = vec![opp_start_bp, ball_oop_end];
    }
    ball[SimPhase::Oop.index()] = oop_ball;

    // ---- Ball won (`SimPhase::Win`)
    let atr = v("att_trans");
    for p in &mut players {
        let o = p.pos_at(SimPhase::Oop);
        let runs = p.ip_has(RoleTag::Run)
            || p.ip_has(RoleTag::Box)
            || p.oop_has(RoleTag::Out)
            || p.a.group == PosGroup::St;
        let pt = if atr == 2 && runs {
            let f = p.pos_at(SimPhase::Final);
            let pt = SimPoint::new(o.x + (f.x - o.x) * 0.45, (o.y + 26.0).min(93.0));
            arrows[SimPhase::Win.index()].push(arrow(o, pt, "#ffe27a", SimArrowKind::Run));
            pt
        } else if atr == 2 {
            SimPoint::new(o.x, (o.y + 7.0).min(92.0))
        } else if atr == 0 {
            let b = p.pos_at(SimPhase::Build);
            let pt = SimPoint::new(o.x + (b.x - o.x) * 0.62, o.y + (b.y - o.y) * 0.62);
            if p.a.group != PosGroup::Gk {
                arrows[SimPhase::Win.index()].push(arrow(o, pt, "#7dd3fc", SimArrowKind::Cover));
            }
            pt
        } else if runs {
            SimPoint::new(o.x, (o.y + 13.0).min(90.0))
        } else {
            SimPoint::new(o.x, (o.y + 4.0).min(90.0))
        };
        p.set_pos(
            SimPhase::Win,
            SimPoint::new(clamp(pt.x, 5.0, 95.0), clamp(pt.y, 2.0, 96.0)),
        );
    }

    // Chain oop -> win right from where the opponent lost the ball (`ball_oop_end`)
    let win_target = if atr == 0 {
        cb0.or(holder)
    } else {
        players
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.pos_at(SimPhase::Win)
                    .y
                    .total_cmp(&b.pos_at(SimPhase::Win).y)
            })
            .map(|(i, _)| i)
    };
    let win_kind = if atr == 2 {
        BallKind::Loft
    } else {
        BallKind::Pass
    };
    let mut win_ball = dedup_ball(vec![
        BallPoint {
            x: ball_oop_end.x,
            y: ball_oop_end.y,
            kind: win_kind,
        },
        bp_at(
            &players,
            win_target,
            SimPhase::Win,
            0.0,
            if atr == 2 { 2.0 } else { 0.0 },
            win_kind,
        ),
    ]);
    if win_ball.len() < 2 {
        win_ball.push(bp_at(
            &players,
            win_target,
            SimPhase::Win,
            0.0,
            4.0,
            BallKind::Pass,
        ));
    }
    ball[SimPhase::Win.index()] = win_ball;

    // ---- Opponent 11 positions per phase
    let opp_cfg: [(SimPhase, f32, f32); SimPhase::COUNT] = [
        (SimPhase::Build, 64.0, 30.0),
        (SimPhase::Prog, 74.0, 44.0),
        (SimPhase::Final, 86.0, 60.0),
        (SimPhase::Loss, 80.0, 46.0),
        (SimPhase::Oop, opp_back, opp_front),
        (SimPhase::Win, 70.0, d_line + 4.0),
    ];
    let mut opp: [Vec<SimPoint>; SimPhase::COUNT] = Default::default();
    for (ph, d, f) in opp_cfg {
        opp[ph.index()] = (0..OPP_SHAPE.len()).map(|k| opp_at(k, d, f)).collect();
    }

    // ---- Visual warnings in OOP block
    let line_y = |gs: &[PosGroup]| -> Option<f32> {
        let mut sum = 0.0_f32;
        let mut cnt = 0usize;
        for p in &players {
            if in_groups(p.b.group, gs) && !p.pressing {
                sum += p.pos_at(SimPhase::Oop).y;
                cnt += 1;
            }
        }
        (cnt > 0).then(|| sum / cnt as f32)
    };
    let def_y = line_y(&[PosGroup::Dc]);
    let mid_y = line_y(&[PosGroup::Dm, PosGroup::Mc, PosGroup::MLr]);
    let mut overlays = Vec::new();
    if let (Some(dy), Some(my)) = (def_y, mid_y) {
        if my - dy > 21.0 {
            overlays.push(SimOverlay {
                kind: SimOverlayKind::Gap,
                y1: dy + 2.0,
                y2: my - 2.0,
                label: lang
                    .pick("espaço entre linhas", "gap between the lines")
                    .to_string(),
            });
        }
    }
    let gk_o = players.iter().find(|p| p.b.group == PosGroup::Gk);
    if dl >= 3 && loe < 2 {
        if let Some(gk_p) = gk_o {
            if gk_p.oop.is_some() && gk_p.oop_id() != Some("swk") {
                overlays.push(SimOverlay {
                    kind: SimOverlayKind::Behind,
                    y1: 8.0,
                    y2: def_y.unwrap_or(d_line) - 2.0,
                    label: lang
                        .pick("espaço nas costas sem cobertura", "uncovered space in behind")
                        .to_string(),
                });
            }
        }
    }

    let sim_players = players
        .into_iter()
        .map(|p| SimPlayer {
            idx: p.idx,
            slot: p.slot,
            ip_role: p.ip.map(|r| r.id).unwrap_or(""),
            oop_role: p.oop.map(|r| r.id).unwrap_or(""),
            sg: p.sg,
            sgb: p.sgb,
            pressing: p.pressing,
            pos: p.pos,
        })
        .collect();

    SimModel {
        players: sim_players,
        ball,
        opp,
        overlays,
        arrows,
        zones,
        lines,
        d_line,
        front,
        bs,
    }
}

// ============================================================================
// Phase Caption Text & Role Movement Notes (`phase_text`)
// ============================================================================

fn tactic_ti_label(t: &Tactic, key: &str) -> &'static str {
    ti_label(key, ti_val(t, key)).unwrap_or("")
}

pub fn phase_text(t: &Tactic, phase: SimPhase, model: &SimModel, lang: Lang) -> PhaseCaption {
    let lbl = |key: &str| tactic_ti_label(t, key);
    let prog_lbl = ti_val(t, "progress");
    let prog_txt = match prog_lbl {
        2 => lang.pick(
            "Foco pelo centro: o meio-campo junta-se para progredir pela zona 14.",
            "Focus through the centre: midfielders combine through Zone 14.",
        ),
        3 => lang.pick(
            "Progressão concentrada no corredor direito.",
            "Progression concentrated down the right flank.",
        ),
        0 => lang.pick(
            "Progressão concentrada no corredor esquerdo.",
            "Progression concentrated down the left flank.",
        ),
        _ => lang.pick(
            "Circula pelo meio e pelos corredores de forma equilibrada.",
            "Circulates through the middle and flanks evenly.",
        ),
    };

    let raw_text = match phase {
        SimPhase::Build => {
            let tail = if ti_val(t, "buildup") == 2
                || ti_val(t, "goal_kicks") == 2
                || ti_val(t, "passing") >= 4
            {
                lang.pick(
                    "A bola salta o meio-campo em passe longo directamente para a frente.",
                    "The ball skips midfield with a long pass straight forward.",
                )
            } else if ti_val(t, "buildup") == 0 {
                lang.pick(
                    "Saída curta através da pressão: GR sobe, centrais abrem e o médio baixa para triangulação.",
                    "Playing through the press: GK steps up, CBs split wide and DM drops for short triangles.",
                )
            } else {
                lang.pick(
                    "Saída equilibrada: centrais abrem e ligam com o médio defensivo.",
                    "Balanced build-up: CBs split and connect into midfield.",
                )
            };
            format!(
                "Build-up {}, Goal Kicks {}. {}",
                lbl("buildup"),
                lbl("goal_kicks"),
                tail
            )
        }
        SimPhase::Prog => {
            let vert = if ti_val(t, "passing") >= 3 {
                lang.pick(
                    "Jogo vertical: procura o avançado cedo. ",
                    "Vertical play: look for the striker early. ",
                )
            } else {
                ""
            };
            format!(
                "Passing {}, Tempo {}, Pass Reception {}. {}{}",
                lbl("passing"),
                lbl("tempo"),
                lbl("reception"),
                vert,
                prog_txt
            )
        }
        SimPhase::Final => {
            let tail = match ti_val(t, "support_runs") {
                2 => lang.pick(
                    "Os laterais sobrepõem por fora enquanto os extremos fecham por dentro.",
                    "Full-backs overlap outside while wingers tuck inside.",
                ),
                0 => lang.pick(
                    "Os extremos seguram a linha e os laterais atacam o meio-espaço (underlap).",
                    "Wingers hold the touchline while full-backs underlap into the half-space.",
                ),
                _ => lang.pick(
                    "Repara nas setas douradas (desmarcações) e em quem fica na rest-defence.",
                    "Watch the gold run arrows and who holds rest-defence.",
                ),
            };
            format!(
                "Attacking Width {}, Supporting Runs {}, Crossing {}. {}",
                lbl("width"),
                lbl("support_runs"),
                lbl("crossing"),
                tail
            )
        }
        SimPhase::Loss => {
            let tail = match ti_val(t, "def_trans") {
                2 => lang.pick(
                    "Contra-pressão: os mais próximos fecham de imediato sobre a bola (anel vermelho).",
                    "Counter-press: the nearest players swarm the ball immediately (red zone).",
                ),
                0 => lang.pick(
                    "Reagrupar: a equipa recua de imediato para o bloco (setas azuis).",
                    "Regroup: the team drops immediately into its block (blue arrows).",
                ),
                _ => lang.pick(
                    "Os mais próximos pressionam; os restantes recuam.",
                    "The nearest players press; the rest drop back.",
                ),
            };
            format!("Defensive Transition {}. {}", lbl("def_trans"), tail)
        }
        SimPhase::Oop => {
            let tail = lang.pick(
                "As linhas tracejadas mostram o bloco; anéis vermelhos saem a pressionar.",
                "Dashed lines mark the defensive block; red rings step out to press.",
            );
            format!(
                "Line of Engagement {}, Defensive Line {}, Trigger Press {}, Pressing Trap {}. {}",
                lbl("loe"),
                lbl("dline"),
                lbl("trigger"),
                lbl("press_trap"),
                tail
            )
        }
        SimPhase::Win => {
            let tail = match ti_val(t, "att_trans") {
                2 => lang.pick(
                    "Contra-ataque: corredores e outlets arrancam (setas douradas) e a bola vai nas costas.",
                    "Counter-attack: runners and outlets sprint forward (gold arrows) and the ball goes in behind.",
                ),
                0 => lang.pick(
                    "Hold Shape: passe seguro para trás e a equipa reorganiza a posse.",
                    "Hold Shape: safe pass back while the team settles into possession.",
                ),
                _ => lang.pick(
                    "Saída equilibrada: alguns arrancam, o resto acompanha.",
                    "A balanced break: some run, the rest follow.",
                ),
            };
            format!("Attacking Transition {}. {}", lbl("att_trans"), tail)
        }
    };

    let text = match lang {
        Lang::Pt => pt_terms(&raw_text),
        Lang::En => raw_text,
    };

    let mut notes = Vec::new();
    for p in &model.players {
        for rid in [p.ip_role, p.oop_role] {
            if rid.is_empty() {
                continue;
            }
            if let (Some(r), Some(n)) = (role_by_id(rid), move_note(rid)) {
                if n.phase == phase {
                    notes.push((
                        r.ab.to_string(),
                        p.slot.ip_code.clone(),
                        lang.pick(n.pt, n.en).to_string(),
                    ));
                }
            }
        }
    }

    PhaseCaption { text, notes }
}

// ============================================================================
// Unit Tests (Parity with `tests/engine.test.cjs` #11 & `sim_canvas.test.cjs` #1/#4)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::roles::{default_role, valid_role, ROLES};
    use crate::data::tactics_data::{formation_slots, ti_by_key, FORMATIONS, TEMPLATES};
    use crate::types::{Mentality, Phase, Template};
    use std::collections::BTreeMap;

    /// Exact Hungarian 1:1 formation pairing for self-contained unit tests.
    fn test_match_slots(ip_form: &str, oop_form: &str) -> Vec<(String, String)> {
        let ip = formation_slots(ip_form).expect("ip formation");
        let oop = formation_slots(oop_form).expect("oop formation");
        let n = ip.len();
        let m = oop.len();
        let mut u = vec![0.0_f64; n + 1];
        let mut v = vec![0.0_f64; m + 1];
        let mut p = vec![0usize; m + 1];
        let mut way = vec![0usize; m + 1];
        for i in 1..=n {
            p[0] = i;
            let mut j0 = 0usize;
            let mut minv = vec![1e18_f64; m + 1];
            let mut used = vec![false; m + 1];
            loop {
                used[j0] = true;
                let i0 = p[j0];
                let mut delta = 1e18_f64;
                let mut j1 = 0usize;
                for j in 1..=m {
                    if !used[j] {
                        let a = &ip[i0 - 1];
                        let b = &oop[j - 1];
                        let c = if (a.code == "GK") != (b.code == "GK") {
                            1e6_f64
                        } else {
                            ((a.x - b.x) as f64).hypot(((a.y - b.y) * 1.1) as f64)
                        };
                        let cur = c - u[i0] - v[j];
                        if cur < minv[j] {
                            minv[j] = cur;
                            way[j] = j0;
                        }
                        if minv[j] < delta {
                            delta = minv[j];
                            j1 = j;
                        }
                    }
                }
                for j in 0..=m {
                    if used[j] {
                        u[p[j]] += delta;
                        v[j] -= delta;
                    } else {
                        minv[j] -= delta;
                    }
                }
                j0 = j1;
                if p[j0] == 0 {
                    break;
                }
            }
            loop {
                let j1 = way[j0];
                p[j0] = p[j1];
                j0 = j1;
                if j0 == 0 {
                    break;
                }
            }
        }
        let mut res = vec![0usize; n];
        for j in 1..=m {
            if p[j] != 0 {
                res[p[j] - 1] = j - 1;
            }
        }
        ip.iter()
            .enumerate()
            .map(|(i, a)| (a.code.to_string(), oop[res[i]].code.to_string()))
            .collect()
    }

    fn test_tactic_from_template(tp: &Template) -> Tactic {
        let pairs = test_match_slots(tp.ip, tp.oop);
        let slots = pairs
            .into_iter()
            .map(|(ip_code, oop_code)| {
                let a = slot_info(tp.ip, &ip_code).unwrap();
                let b = slot_info(tp.oop, &oop_code).unwrap();
                let mut ip = default_role(a.group, Phase::Ip).to_string();
                let mut oop = default_role(b.group, Phase::Oop).to_string();
                if let Some(&(_, r_ip, r_oop)) = tp.roles.iter().find(|&&(c, _, _)| c == ip_code) {
                    if valid_role(r_ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb) {
                        ip = r_ip.to_string();
                    }
                    if valid_role(r_oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb) {
                        oop = r_oop.to_string();
                    }
                }
                SlotAssignment {
                    ip_code,
                    oop_code,
                    ip,
                    oop,
                    pid: None,
                    pis: Vec::new(),
                }
            })
            .collect();
        let ti: BTreeMap<String, usize> =
            tp.ti.iter().map(|&(k, v)| (k.to_string(), v)).collect();
        Tactic {
            name: tp.name_pt.to_string(),
            ment: tp.ment,
            ip_form: tp.ip.to_string(),
            oop_form: tp.oop.to_string(),
            slots,
            ti,
            ois: 0,
            source: "template".to_string(),
            saved_at: None,
        }
    }

    fn test_tactic_for_role(r: &Role) -> (Tactic, usize) {
        let form = FORMATIONS
            .iter()
            .find(|f| {
                f.slots.iter().any(|s| {
                    slot_info(f.name, s.code).map_or(false, |a| {
                        valid_role(r.id, a.group, r.phase, Some(&a.code), a.wide_cb)
                    })
                })
            })
            .expect("every role must fit at least one formation");
        let mut slots: Vec<SlotAssignment> = form
            .slots
            .iter()
            .map(|s| {
                let a = slot_info(form.name, s.code).unwrap();
                SlotAssignment {
                    ip_code: s.code.to_string(),
                    oop_code: s.code.to_string(),
                    ip: default_role(a.group, Phase::Ip).to_string(),
                    oop: default_role(a.group, Phase::Oop).to_string(),
                    pid: None,
                    pis: Vec::new(),
                }
            })
            .collect();
        let idx = slots
            .iter()
            .position(|s| {
                let a = slot_info(form.name, &s.ip_code).unwrap();
                valid_role(r.id, a.group, r.phase, Some(&a.code), a.wide_cb)
            })
            .expect("slot for role");
        match r.phase {
            Phase::Ip => slots[idx].ip = r.id.to_string(),
            Phase::Oop => slots[idx].oop = r.id.to_string(),
        }
        let t = Tactic {
            name: format!("Test {}", r.id),
            ment: Mentality::Balanced,
            ip_form: form.name.to_string(),
            oop_form: form.name.to_string(),
            slots,
            ti: BTreeMap::new(),
            ois: 0,
            source: "test".to_string(),
            saved_at: None,
        };
        (t, idx)
    }

    fn assert_model_finite(m: &SimModel, label: &str) {
        assert_eq!(m.players.len(), 11, "{label}: expected 11 players");
        for p in &m.players {
            for ph in SimPhase::ALL {
                let pt = p.pos[ph.index()];
                assert!(
                    pt.x.is_finite() && pt.y.is_finite(),
                    "{label}: non-finite player pos at {} {:?}",
                    p.slot.ip_code,
                    ph
                );
            }
        }
        for ph in SimPhase::ALL {
            let b = &m.ball[ph.index()];
            assert!(b.len() >= 2, "{label}: ball path too short in {ph:?}");
            for pt in b {
                assert!(
                    pt.x.is_finite() && pt.y.is_finite(),
                    "{label}: non-finite ball pos in {ph:?}"
                );
            }
            assert_eq!(m.opp[ph.index()].len(), 11, "{label}: expected 11 opp in {ph:?}");
            for pt in &m.opp[ph.index()] {
                assert!(pt.x.is_finite() && pt.y.is_finite(), "{label}: non-finite opp in {ph:?}");
            }
        }
    }

    #[test]
    fn ball_elevation_loft_vs_ground_pass() {
        let loft_mid = sim_ball_state(
            &[
                BallPoint { x: 20.0, y: 10.0, kind: BallKind::Loft },
                BallPoint { x: 80.0, y: 70.0, kind: BallKind::Loft },
            ],
            0.5,
        );
        let pass_mid = sim_ball_state(
            &[
                BallPoint { x: 20.0, y: 10.0, kind: BallKind::Pass },
                BallPoint { x: 80.0, y: 70.0, kind: BallKind::Pass },
            ],
            0.5,
        );
        assert!(loft_mid.z > 0.8, "lofted ball should elevate at midpoint, got {}", loft_mid.z);
        assert_eq!(pass_mid.z, 0.0, "ground pass should stay at z=0");
    }

    #[test]
    fn templates_all_phases_and_animated_tis_finite_and_continuous() {
        let animated_tis = [
            "width",
            "passing",
            "loe",
            "dline",
            "trigger",
            "def_trans",
            "att_trans",
            "press_trap",
            "support_runs",
            "buildup",
            "short_gk",
            "dl_behaviour",
        ];

        for tp in &TEMPLATES {
            let base = test_tactic_from_template(tp);
            let m = sim_model(&base, Lang::En);
            assert_model_finite(&m, tp.id);

            // Continuous ball handoffs & forward progression (sim_canvas.test.cjs #4)
            for (idx, ph) in SimPhase::ALL.iter().copied().enumerate() {
                if idx > 0 {
                    let prev = SimPhase::ALL[idx - 1];
                    let b_a = &m.ball[prev.index()];
                    let b_b = &m.ball[ph.index()];
                    let last_a = b_a.last().unwrap();
                    let first_b = b_b.first().unwrap();
                    let dist = (last_a.x - first_b.x).hypot(last_a.y - first_b.y);
                    if matches!(prev, SimPhase::Final | SimPhase::Loss | SimPhase::Oop) {
                        assert!(
                            dist <= 5.0,
                            "template {} {:?}->{:?} ball jump {:.1}",
                            tp.id,
                            prev,
                            ph,
                            dist
                        );
                    } else {
                        assert!(
                            first_b.y - last_a.y >= -10.0,
                            "template {} {:?}->{:?} ball jumped backwards (dy={:.1})",
                            tp.id,
                            prev,
                            ph,
                            first_b.y - last_a.y
                        );
                    }
                }
                if matches!(ph, SimPhase::Build | SimPhase::Prog | SimPhase::Final) {
                    let arr = &m.ball[ph.index()];
                    for j in 1..arr.len() {
                        assert!(
                            arr[j].y >= arr[j - 1].y - 8.0,
                            "template {} {:?} intra-phase backward pass ({:.0}->{:.0})",
                            tp.id,
                            ph,
                            arr[j - 1].y,
                            arr[j].y
                        );
                    }
                }
            }

            // Vary all 12 animated TIs across every option (engine.test.cjs #11)
            for k in animated_tis {
                let def = ti_by_key(k).expect("ti def");
                for val in 0..def.opts.len() {
                    let mut t2 = base.clone();
                    t2.ti.insert(k.to_string(), val);
                    let m2 = sim_model(&t2, Lang::En);
                    assert_model_finite(&m2, &format!("{}:{}{}", tp.id, k, val));
                }
            }
        }
    }

    #[test]
    fn all_77_roles_produce_finite_sim_models() {
        for r in &ROLES {
            let (t, idx) = test_tactic_for_role(r);
            match r.phase {
                Phase::Ip => assert_eq!(t.slots[idx].ip, r.id),
                Phase::Oop => assert_eq!(t.slots[idx].oop, r.id),
            }
            let m = sim_model(&t, Lang::En);
            assert_model_finite(&m, &format!("role:{}", r.id));
        }
    }

    #[test]
    fn phase_text_renders_both_languages_and_role_notes() {
        let t = test_tactic_from_template(&TEMPLATES[0]);
        let m_en = sim_model(&t, Lang::En);
        let m_pt = sim_model(&t, Lang::Pt);
        for ph in SimPhase::ALL {
            let cap_en = phase_text(&t, ph, &m_en, Lang::En);
            let cap_pt = phase_text(&t, ph, &m_pt, Lang::Pt);
            assert!(!cap_en.text.is_empty(), "empty EN caption for {ph:?}");
            assert!(!cap_pt.text.is_empty(), "empty PT caption for {ph:?}");
        }
        // possession433 has bpgk (Build), dlp/f9 (Prog), if_ip/wb_ip (Final), swk/ccb/scb/sdm (Oop)
        let build_cap = phase_text(&t, SimPhase::Build, &m_en, Lang::En);
        assert!(build_cap.notes.iter().any(|(ab, _, _)| ab == "BPG"));
        let oop_cap = phase_text(&t, SimPhase::Oop, &m_en, Lang::En);
        assert!(oop_cap.notes.iter().any(|(ab, _, _)| ab == "SK"));
    }
}
