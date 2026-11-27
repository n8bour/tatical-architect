//! Static formations, mentalities, team/player instructions, tactical tweaks,
//! training areas, templates, style profiles, and FM27 sheet sections.
//! Ported 1:1 from non-Bible `src/core/tactics_data.js`.

use crate::types::{
    Attr, FmSection, Formation, FormationSlot, Lang, Mentality, Phase, PlayerInstructionDef,
    PosGroup, Side, SlotMeta, StyleProfile, Tactic, TacticalTweakDef, TeamInstructionDef,
    Template, TiNeed, TrainingAreaDef,
};

/// Role IDs whose attribute weights were verified against public FM26 sources.
pub const ATTR_VERIFIED: &[&str] = &["dlf", "poa", "f9", "iw_ip", "scm", "pfb", "lhk"];

pub fn is_attr_verified(role_id: &str) -> bool {
    ATTR_VERIFIED.contains(&role_id)
}

/// All 17 formations with normalized coordinates: x 0(left)..100(right), y 0(own goal)..100(opp goal).
pub const FORMATIONS: [Formation; 17] = [
    Formation {
        name: "4-3-3 DM Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DM", x: 50.0, y: 38.0 },
            FormationSlot { code: "MCL", x: 32.0, y: 52.0 },
            FormationSlot { code: "MCR", x: 68.0, y: 52.0 },
            FormationSlot { code: "AML", x: 12.0, y: 72.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
            FormationSlot { code: "AMR", x: 88.0, y: 72.0 },
        ],
    },
    Formation {
        name: "4-2-3-1 DM Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DML", x: 38.0, y: 38.0 },
            FormationSlot { code: "DMR", x: 62.0, y: 38.0 },
            FormationSlot { code: "AML", x: 12.0, y: 68.0 },
            FormationSlot { code: "AMC", x: 50.0, y: 66.0 },
            FormationSlot { code: "AMR", x: 88.0, y: 68.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
        ],
    },
    Formation {
        name: "4-2-3-1 Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 46.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 46.0 },
            FormationSlot { code: "AML", x: 12.0, y: 68.0 },
            FormationSlot { code: "AMC", x: 50.0, y: 66.0 },
            FormationSlot { code: "AMR", x: 88.0, y: 68.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
        ],
    },
    Formation {
        name: "4-1-4-1 DM Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DM", x: 50.0, y: 36.0 },
            FormationSlot { code: "ML", x: 10.0, y: 52.0 },
            FormationSlot { code: "MCL", x: 36.0, y: 52.0 },
            FormationSlot { code: "MCR", x: 64.0, y: 52.0 },
            FormationSlot { code: "MR", x: 90.0, y: 52.0 },
            FormationSlot { code: "ST", x: 50.0, y: 80.0 },
        ],
    },
    Formation {
        name: "4-4-2",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "ML", x: 10.0, y: 50.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 48.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 48.0 },
            FormationSlot { code: "MR", x: 90.0, y: 50.0 },
            FormationSlot { code: "STL", x: 40.0, y: 80.0 },
            FormationSlot { code: "STR", x: 60.0, y: 80.0 },
        ],
    },
    Formation {
        name: "4-4-1-1",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "ML", x: 10.0, y: 50.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 48.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 48.0 },
            FormationSlot { code: "MR", x: 90.0, y: 50.0 },
            FormationSlot { code: "AMC", x: 50.0, y: 68.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
        ],
    },
    Formation {
        name: "4-4-2 Diamond",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DM", x: 50.0, y: 36.0 },
            FormationSlot { code: "MCL", x: 30.0, y: 50.0 },
            FormationSlot { code: "MCR", x: 70.0, y: 50.0 },
            FormationSlot { code: "AMC", x: 50.0, y: 64.0 },
            FormationSlot { code: "STL", x: 40.0, y: 82.0 },
            FormationSlot { code: "STR", x: 60.0, y: 82.0 },
        ],
    },
    Formation {
        name: "4-2-2-2",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DML", x: 38.0, y: 38.0 },
            FormationSlot { code: "DMR", x: 62.0, y: 38.0 },
            FormationSlot { code: "AMCL", x: 30.0, y: 66.0 },
            FormationSlot { code: "AMCR", x: 70.0, y: 66.0 },
            FormationSlot { code: "STL", x: 40.0, y: 84.0 },
            FormationSlot { code: "STR", x: 60.0, y: 84.0 },
        ],
    },
    Formation {
        name: "4-3-3 Flat",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "MCL", x: 28.0, y: 48.0 },
            FormationSlot { code: "MC", x: 50.0, y: 46.0 },
            FormationSlot { code: "MCR", x: 72.0, y: 48.0 },
            FormationSlot { code: "AML", x: 12.0, y: 72.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
            FormationSlot { code: "AMR", x: 88.0, y: 72.0 },
        ],
    },
    Formation {
        name: "3-4-2-1",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DCL", x: 28.0, y: 20.0 },
            FormationSlot { code: "DC", x: 50.0, y: 18.0 },
            FormationSlot { code: "DCR", x: 72.0, y: 20.0 },
            FormationSlot { code: "WBL", x: 8.0, y: 42.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 46.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 46.0 },
            FormationSlot { code: "WBR", x: 92.0, y: 42.0 },
            FormationSlot { code: "AMCL", x: 32.0, y: 68.0 },
            FormationSlot { code: "AMCR", x: 68.0, y: 68.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
        ],
    },
    Formation {
        name: "3-4-3",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DCL", x: 28.0, y: 20.0 },
            FormationSlot { code: "DC", x: 50.0, y: 18.0 },
            FormationSlot { code: "DCR", x: 72.0, y: 20.0 },
            FormationSlot { code: "WBL", x: 8.0, y: 44.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 46.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 46.0 },
            FormationSlot { code: "WBR", x: 92.0, y: 44.0 },
            FormationSlot { code: "AML", x: 14.0, y: 72.0 },
            FormationSlot { code: "ST", x: 50.0, y: 84.0 },
            FormationSlot { code: "AMR", x: 86.0, y: 72.0 },
        ],
    },
    Formation {
        name: "3-5-2",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DCL", x: 28.0, y: 20.0 },
            FormationSlot { code: "DC", x: 50.0, y: 18.0 },
            FormationSlot { code: "DCR", x: 72.0, y: 20.0 },
            FormationSlot { code: "WBL", x: 8.0, y: 44.0 },
            FormationSlot { code: "DM", x: 50.0, y: 36.0 },
            FormationSlot { code: "MCL", x: 34.0, y: 52.0 },
            FormationSlot { code: "MCR", x: 66.0, y: 52.0 },
            FormationSlot { code: "WBR", x: 92.0, y: 44.0 },
            FormationSlot { code: "STL", x: 40.0, y: 82.0 },
            FormationSlot { code: "STR", x: 60.0, y: 82.0 },
        ],
    },
    Formation {
        name: "5-3-2 WB",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "WBL", x: 8.0, y: 28.0 },
            FormationSlot { code: "DCL", x: 30.0, y: 18.0 },
            FormationSlot { code: "DC", x: 50.0, y: 16.0 },
            FormationSlot { code: "DCR", x: 70.0, y: 18.0 },
            FormationSlot { code: "WBR", x: 92.0, y: 28.0 },
            FormationSlot { code: "MCL", x: 30.0, y: 46.0 },
            FormationSlot { code: "MC", x: 50.0, y: 44.0 },
            FormationSlot { code: "MCR", x: 70.0, y: 46.0 },
            FormationSlot { code: "STL", x: 40.0, y: 76.0 },
            FormationSlot { code: "STR", x: 60.0, y: 76.0 },
        ],
    },
    Formation {
        name: "5-4-1",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "WBL", x: 8.0, y: 26.0 },
            FormationSlot { code: "DCL", x: 30.0, y: 18.0 },
            FormationSlot { code: "DC", x: 50.0, y: 16.0 },
            FormationSlot { code: "DCR", x: 70.0, y: 18.0 },
            FormationSlot { code: "WBR", x: 92.0, y: 26.0 },
            FormationSlot { code: "ML", x: 12.0, y: 46.0 },
            FormationSlot { code: "MCL", x: 38.0, y: 44.0 },
            FormationSlot { code: "MCR", x: 62.0, y: 44.0 },
            FormationSlot { code: "MR", x: 88.0, y: 46.0 },
            FormationSlot { code: "ST", x: 50.0, y: 74.0 },
        ],
    },
    Formation {
        name: "4-2-4 DM Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 24.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 20.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 20.0 },
            FormationSlot { code: "DR", x: 90.0, y: 24.0 },
            FormationSlot { code: "DML", x: 38.0, y: 38.0 },
            FormationSlot { code: "DMR", x: 62.0, y: 38.0 },
            FormationSlot { code: "AML", x: 10.0, y: 66.0 },
            FormationSlot { code: "AMR", x: 90.0, y: 66.0 },
            FormationSlot { code: "STL", x: 40.0, y: 84.0 },
            FormationSlot { code: "STR", x: 60.0, y: 84.0 },
        ],
    },
    Formation {
        name: "4-4-2 DM Wide",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 22.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 18.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 18.0 },
            FormationSlot { code: "DR", x: 90.0, y: 22.0 },
            FormationSlot { code: "DML", x: 38.0, y: 34.0 },
            FormationSlot { code: "DMR", x: 62.0, y: 34.0 },
            FormationSlot { code: "ML", x: 10.0, y: 48.0 },
            FormationSlot { code: "MR", x: 90.0, y: 48.0 },
            FormationSlot { code: "STL", x: 42.0, y: 72.0 },
            FormationSlot { code: "STR", x: 58.0, y: 72.0 },
        ],
    },
    Formation {
        name: "4-5-1",
        slots: [
            FormationSlot { code: "GK", x: 50.0, y: 4.0 },
            FormationSlot { code: "DL", x: 10.0, y: 22.0 },
            FormationSlot { code: "DCL", x: 36.0, y: 18.0 },
            FormationSlot { code: "DCR", x: 64.0, y: 18.0 },
            FormationSlot { code: "DR", x: 90.0, y: 22.0 },
            FormationSlot { code: "ML", x: 10.0, y: 44.0 },
            FormationSlot { code: "MCL", x: 32.0, y: 44.0 },
            FormationSlot { code: "MC", x: 50.0, y: 42.0 },
            FormationSlot { code: "MCR", x: 68.0, y: 44.0 },
            FormationSlot { code: "MR", x: 90.0, y: 44.0 },
            FormationSlot { code: "ST", x: 50.0, y: 72.0 },
        ],
    },
];

pub fn formation_by_name(name: &str) -> Option<&'static Formation> {
    FORMATIONS.iter().find(|f| f.name == name)
}

pub fn formation_slots(name: &str) -> Option<&'static [FormationSlot; 11]> {
    formation_by_name(name).map(|f| &f.slots)
}

pub fn group_of(code: &str) -> PosGroup {
    if code == "GK" {
        PosGroup::Gk
    } else if code == "DL" || code == "DR" {
        PosGroup::DLr
    } else if code.starts_with("DC") {
        PosGroup::Dc
    } else if code.starts_with("WB") {
        PosGroup::WbLr
    } else if code.starts_with("DM") {
        PosGroup::Dm
    } else if code == "ML" || code == "MR" {
        PosGroup::MLr
    } else if code.starts_with("MC") {
        PosGroup::Mc
    } else if code == "AML" || code == "AMR" {
        PosGroup::AmLr
    } else if code.starts_with("AMC") {
        PosGroup::Amc
    } else if code.starts_with("ST") {
        PosGroup::St
    } else {
        PosGroup::Mc
    }
}

pub fn is_wide_cb(code: &str, formation: &str) -> bool {
    let Some(slots) = formation_slots(formation) else {
        return false;
    };
    let n_cb = slots.iter().filter(|s| s.code.starts_with("DC")).count();
    n_cb >= 3 && (code == "DCL" || code == "DCR")
}

#[inline]
pub const fn side_of(x: f32) -> Side {
    if x < 33.0 {
        Side::L
    } else if x > 67.0 {
        Side::R
    } else {
        Side::C
    }
}

pub fn slot_info(formation: &str, code: &str) -> Option<SlotMeta> {
    let s = formation_slots(formation)?.iter().find(|z| z.code == code)?;
    Some(SlotMeta {
        code: s.code.to_string(),
        x: s.x,
        y: s.y,
        group: group_of(code),
        side: side_of(s.x),
        wide_cb: is_wide_cb(code, formation),
    })
}

pub const fn mentality_info_pt(m: Mentality) -> &'static str {
    match m {
        Mentality::VeryDefensive => "Bloco muito baixo, risco mínimo. Para proteger vantagens curtas ou jogos de enorme inferioridade.",
        Mentality::Defensive => "Prioriza a solidez, transições cautelosas. Underdog fora de casa.",
        Mentality::Cautious => "Equilibrado mas com prudência; posse mais lenta e paciente.",
        Mentality::Balanced => "Ponto de partida neutro; as instruções e os papéis definem o carácter.",
        Mentality::Positive => "Proactivo sem extremos: avança linhas, mais risco controlado. Óptimo default para equipas favoritas.",
        Mentality::Attacking => "Mais largura, ritmo e risco. Atenção à rest-defence.",
        Mentality::VeryAttacking => "Tudo ao ataque. Use em fases finais quando precisa de golos; expõe muito as transições.",
    }
}

pub fn mentality_info(m: Mentality, lang: Lang) -> String {
    crate::data::i18n::t(lang, mentality_info_pt(m))
}

/// All 27 FM26/FM27 Team Instructions (18 IP + 9 OOP).
pub const TEAM_INSTRUCTIONS: [TeamInstructionDef; 27] = [
    TeamInstructionDef {
        key: "passing", name: "Passing Directness", phase: Phase::Ip, area_pt: "Geral",
        opts: &["Much Shorter", "Shorter", "Balanced", "More Direct", "Much More Direct"], def: 2, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Comprimento médio dos passes.", pro_pt: "Curto: retém a bola, cansa o adversário. Directo: chega rápido à frente, ignora a pressão.", con_pt: "Curto exige Passe/Técnica/Compostura em todos. Directo perde mais bolas se não houver referência/velocidade.",
        need: Some(TiNeed { low: &[Attr::Pas, Attr::Tec, Attr::Cmp, Attr::Fir], high: &[Attr::Hea, Attr::Str, Attr::Otb, Attr::Pac] }),
    },
    TeamInstructionDef {
        key: "tempo", name: "Tempo", phase: Phase::Ip, area_pt: "Geral",
        opts: &["Much Lower", "Lower", "Balanced", "Higher", "Much Higher"], def: 2, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Velocidade da tomada de decisão com bola.", pro_pt: "Alto: apanha o adversário desorganizado. Baixo: controlo e paciência.", con_pt: "Alto + passe curto exige Decisões/Técnica altas; cansa mais.",
        need: Some(TiNeed { low: &[], high: &[Attr::Dec, Attr::Tec, Attr::Fir, Attr::Sta] }),
    },
    TeamInstructionDef {
        key: "width", name: "Attacking Width", phase: Phase::Ip, area_pt: "Geral",
        opts: &["Much Narrower", "Narrower", "Balanced", "Wider", "Much Wider"], def: 2, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Quão aberta fica a equipa com bola.", pro_pt: "Largo: estica o bloco adversário, abre corredores interiores. Estreito: combinações centrais.", con_pt: "Largo dificulta a reacção à perda (counter-press). Estreito facilita o fecho do adversário ao meio.",
        need: None,
    },
    TeamInstructionDef {
        key: "creative", name: "Creative Freedom", phase: Phase::Ip, area_pt: "Geral",
        opts: &["More Disciplined", "Balanced", "More Expressive"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Liberdade para sair do plano.", pro_pt: "Expressivo: jogadores de topo encontram soluções. Disciplinado: execução previsível e segura.", con_pt: "Expressivo com jogadores fracos de Decisões/Talento = caos.",
        need: Some(TiNeed { low: &[], high: &[Attr::Dec, Attr::Fla, Attr::Vis] }),
    },
    TeamInstructionDef {
        key: "att_trans", name: "Attacking Transition", phase: Phase::Ip, area_pt: "Transição",
        opts: &["Hold Shape", "Balanced", "Counter-Attack"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "O que fazer ao recuperar a bola.", pro_pt: "Counter: explora defesa desorganizada. Hold Shape: reorganiza e assume posse.", con_pt: "Counter sem velocidade/outlets resulta em perdas rápidas.",
        need: Some(TiNeed { low: &[], high: &[Attr::Pac, Attr::Acc, Attr::Otb] }),
    },
    TeamInstructionDef {
        key: "set_pieces", name: "Play for Set Pieces", phase: Phase::Ip, area_pt: "Geral",
        opts: &["Keep Ball in Play", "Balanced", "Play for Set Pieces"], def: 1, cx: 0.5,
        fm27: true, unconf: true,
        desc_pt: "Procurar ou evitar ganhar cantos e livres (nova no FM27).", pro_pt: "Play for Set Pieces rende com bons cabeceadores e marcadores de bolas paradas.", con_pt: "Keep Ball in Play mantém o ritmo mas desperdiça uma arma se fores forte no ar.",
        need: None,
    },
    TeamInstructionDef {
        key: "timewaste", name: "Time Wasting", phase: Phase::Ip, area_pt: "Geral",
        opts: &["Less", "Balanced", "More"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Gestão do relógio.", pro_pt: "Útil a ganhar no fim.", con_pt: "Pode irritar adversários/árbitro; quebra o ritmo da própria equipa.",
        need: None,
    },
    TeamInstructionDef {
        key: "reception", name: "Pass Reception", phase: Phase::Ip, area_pt: "Progressão",
        opts: &["Pass into Space", "Balanced", "Pass to Feet"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Passar para o espaço ou para o pé (novo no FM26).", pro_pt: "Ao pé: controlo. Espaço: profundidade e velocidade.", con_pt: "Ao pé contra pressão alta exige Primeiro Toque/Compostura. Espaço exige Sem Bola/Aceleração.",
        need: Some(TiNeed { low: &[Attr::Otb, Attr::Acc], high: &[Attr::Fir, Attr::Cmp, Attr::Tec] }),
    },
    TeamInstructionDef {
        key: "buildup", name: "Build-up Strategy", phase: Phase::Ip, area_pt: "Construção",
        opts: &["Play Through Press", "Balanced", "Bypass Press"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Sair a jogar curto contra a pressão ou ultrapassá-la (novo no FM26).", pro_pt: "Through: atrai e quebra a pressão. Bypass: risco mínimo atrás.", con_pt: "Through com GR/centrais fracos tecnicamente = golos sofridos. Bypass perde posse.",
        need: Some(TiNeed { low: &[Attr::Pas, Attr::Cmp, Attr::Fir, Attr::Dec], high: &[] }),
    },
    TeamInstructionDef {
        key: "goal_kicks", name: "Goal Kicks", phase: Phase::Ip, area_pt: "Construção",
        opts: &["Short", "Mixed", "Long"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Tipo de pontapé de baliza.", pro_pt: "Curto: inicia a construção. Longo: salta a pressão para a referência.", con_pt: "Curto contra pressão alta e defesas fracos com bola é arriscado.",
        need: None,
    },
    TeamInstructionDef {
        key: "gk_speed", name: "GK Distribution Speed", phase: Phase::Ip, area_pt: "Construção",
        opts: &["Slow Pace Down", "Balanced", "Distribute Quickly"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Ritmo de reposição do guarda-redes.", pro_pt: "Rápido alimenta contra-ataques.", con_pt: "Rápido com a equipa desposicionada perde bolas.",
        need: None,
    },
    TeamInstructionDef {
        key: "gk_target", name: "GK Distribution", phase: Phase::Ip, area_pt: "Construção",
        opts: &["Centre-Backs", "Full-Backs", "Balanced", "Flanks", "Target Forward"], def: 2, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Para onde o GR distribui.", pro_pt: "Dirigir a bola ao seu ponto forte (central técnico, ala rápido, ponta-de-lança alto).", con_pt: "Previsível: o adversário pode montar armadilha.",
        need: None,
    },
    TeamInstructionDef {
        key: "support_runs", name: "Supporting Runs", phase: Phase::Ip, area_pt: "Progressão",
        opts: &["Underlap", "Balanced", "Overlap"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Laterais/alas apoiam por dentro (underlap) ou por fora (overlap).", pro_pt: "Overlap com extremos interiores; underlap com extremos abertos.", con_pt: "Contraria-se com o papel: um W com overlap gera aglomeração no corredor.",
        need: None,
    },
    TeamInstructionDef {
        key: "dribbling", name: "Dribbling", phase: Phase::Ip, area_pt: "Progressão",
        opts: &["Discourage", "Balanced", "Encourage"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Frequência de condução/drible.", pro_pt: "Encoraje com driblers de qualidade.", con_pt: "Encoraje com Drible fraco = perdas em zonas perigosas.",
        need: Some(TiNeed { low: &[], high: &[Attr::Dri, Attr::Agi, Attr::Fla] }),
    },
    TeamInstructionDef {
        key: "progress", name: "Progress Through", phase: Phase::Ip, area_pt: "Progressão",
        opts: &["Left", "Balanced", "Centre", "Right"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Zona preferida de progressão.", pro_pt: "Concentre no lado do seu melhor jogador ou do ponto fraco adversário.", con_pt: "Previsível; adversário pode sobrecarregar esse lado.",
        need: None,
    },
    TeamInstructionDef {
        key: "patience", name: "Final Third Patience", phase: Phase::Ip, area_pt: "Último terço",
        opts: &["Hit Early Crosses", "Balanced", "Work Ball Into Box"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Cruzar cedo para a área ou trabalhar a bola com paciência até à finalização.", pro_pt: "Work Ball Into Box: melhores oportunidades. Hit Early Crosses: apanha a defesa antes de se organizar.", con_pt: "Work Ball Into Box contra bloco baixo pode gerar posse estéril; Hit Early Crosses desperdiça bolas sem boas referências.",
        need: None,
    },
    TeamInstructionDef {
        key: "long_shots", name: "Shots from Distance", phase: Phase::Ip, area_pt: "Último terço",
        opts: &["Discourage", "Balanced", "Encourage"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Frequência de remates de fora da área.", pro_pt: "Encoraje com bons rematadores contra blocos baixos.", con_pt: "Remates fracos desperdiçam posses.",
        need: Some(TiNeed { low: &[], high: &[Attr::Lon, Attr::Tec] }),
    },
    TeamInstructionDef {
        key: "crossing", name: "Crossing Style", phase: Phase::Ip, area_pt: "Último terço",
        opts: &["Low Crosses", "Balanced", "Whipped Crosses", "Floated Crosses"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Tipo de cruzamento.", pro_pt: "Baixos para avançados rápidos; Floated para referências aéreas.", con_pt: "Floated sem jogadores fortes no ar é desperdício.",
        need: None,
    },
    TeamInstructionDef {
        key: "loe", name: "Line of Engagement", phase: Phase::Oop, area_pt: "Pressão",
        opts: &["Low Block", "Mid Block", "High Press"], def: 1, cx: 1.5,
        fm27: true, unconf: false,
        desc_pt: "Onde a equipa começa a pressionar.", pro_pt: "Alta: recupera bolas perto da baliza adversária. Baixa: bloco compacto, convida a sair.", con_pt: "Alta exige Resistência/Índice de Trabalho; desligada da linha defensiva abre espaço entre linhas.",
        need: Some(TiNeed { low: &[], high: &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg] }),
    },
    TeamInstructionDef {
        key: "dline", name: "Defensive Line", phase: Phase::Oop, area_pt: "Bloco",
        opts: &["Much Lower", "Lower", "Standard", "Higher", "Much Higher"], def: 2, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Altura da linha defensiva.", pro_pt: "Alta: equipa compacta, adversário longe da baliza. Baixa: protege a profundidade.", con_pt: "Alta exige centrais rápidos e Sweeper Keeper. Baixa convida pressão e remates.",
        need: Some(TiNeed { low: &[], high: &[Attr::Pac, Attr::Acc, Attr::Ant, Attr::Pos] }),
    },
    TeamInstructionDef {
        key: "trigger", name: "Trigger Press", phase: Phase::Oop, area_pt: "Pressão",
        opts: &["Much Less", "Less", "Balanced", "More", "Much More"], def: 2, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Frequência com que os jogadores disparam a pressão.", pro_pt: "Mais: recuperações altas, caos para o adversário.", con_pt: "Esgota fisicamente; pressão descoordenada é facilmente batida.",
        need: Some(TiNeed { low: &[], high: &[Attr::Wor, Attr::Sta, Attr::Agg, Attr::Ant, Attr::Tea] }),
    },
    TeamInstructionDef {
        key: "def_trans", name: "Defensive Transition", phase: Phase::Oop, area_pt: "Transição",
        opts: &["Regroup", "Balanced", "Counter-Press"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Reacção imediata à perda da bola.", pro_pt: "Counter-Press: recupera bolas antes do adversário se organizar. Regroup: protege a estrutura.", con_pt: "Counter-Press com equipa muito aberta/cansada é atravessado em transição.",
        need: Some(TiNeed { low: &[], high: &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Tea] }),
    },
    TeamInstructionDef {
        key: "tackling", name: "Tackling", phase: Phase::Oop, area_pt: "Duelos",
        opts: &["Stay On Feet", "Balanced", "Get Stuck In"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Agressividade nos desarmes.", pro_pt: "Get Stuck In ganha mais duelos.", con_pt: "Mais faltas e cartões; perigoso com defesas de baixo Desarme.",
        need: Some(TiNeed { low: &[], high: &[Attr::Tck, Attr::Agg, Attr::Bra] }),
    },
    TeamInstructionDef {
        key: "press_trap", name: "Pressing Trap", phase: Phase::Oop, area_pt: "Pressão",
        opts: &["Inside", "Balanced", "Outside"], def: 1, cx: 1.0,
        fm27: false, unconf: false,
        desc_pt: "Para onde se força o portador da bola (bloco médio/pressão alta).", pro_pt: "Outside: encurrala no corredor. Inside: força para o meio congestionado.", con_pt: "Outside sem laterais/extremos agressivos não fecha a armadilha; Inside exige meio-campo forte.",
        need: None,
    },
    TeamInstructionDef {
        key: "cross_eng", name: "Cross Engagement", phase: Phase::Oop, area_pt: "Bloco baixo",
        opts: &["Invite Crosses", "Balanced", "Stop Crosses"], def: 1, cx: 0.5,
        fm27: false, unconf: false,
        desc_pt: "Permitir ou impedir cruzamentos (bloco baixo).", pro_pt: "Invite: com centrais dominantes no ar. Stop: com defesas baixos.", con_pt: "Invite sem centrais fortes no ar = golos de cabeça.",
        need: None,
    },
    TeamInstructionDef {
        key: "short_gk", name: "Short Goalkeeper Distribution", phase: Phase::Oop, area_pt: "Pressão",
        opts: &["No", "Yes"], def: 0, cx: 0.5,
        fm27: true, unconf: false,
        desc_pt: "Pressionar a reposição curta do GR adversário (secção High Press).", pro_pt: "Yes força o adversário a bater longo.", con_pt: "Exige subir os avançados e médios — mais espaço atrás.",
        need: None,
    },
    TeamInstructionDef {
        key: "dl_behaviour", name: "Defensive Line Behaviour", phase: Phase::Oop, area_pt: "Bloco",
        opts: &["Drop Off", "Balanced", "Step Up"], def: 1, cx: 0.5,
        fm27: true, unconf: true,
        desc_pt: "Se a linha recua para proteger a profundidade ou sobe para apertar o espaço (nova no FM27).", pro_pt: "Step Up: equipa compacta, apanha foras-de-jogo. Drop Off: protege as costas.", con_pt: "Step Up com centrais lentos é perigoso; Drop Off afasta a defesa do meio-campo.",
        need: None,
    },
];

pub fn ti_by_key(key: &str) -> Option<&'static TeamInstructionDef> {
    TEAM_INSTRUCTIONS.iter().find(|t| t.key == key)
}

pub fn ti_val(tactic: &Tactic, key: &str) -> usize {
    tactic
        .ti
        .get(key)
        .copied()
        .unwrap_or_else(|| ti_by_key(key).map(|d| d.def).unwrap_or(0))
}

pub fn ti_label(key: &str, val: usize) -> Option<&'static str> {
    ti_by_key(key)?.opts.get(val).copied()
}

/// All 24 Player Instructions.
pub const PLAYER_INSTRUCTIONS: [PlayerInstructionDef; 24] = [
    PlayerInstructionDef { name: "Shoot more", group_pt: "Remate", desc_pt: "Remata mais vezes, incluindo de fora da área.", tip_pt: "Remates de Longe ≥ 15 e Técnica alta.", fm27: false },
    PlayerInstructionDef { name: "Shoot less", group_pt: "Remate", desc_pt: "Procura o passe em vez do remate.", tip_pt: "Quem desperdiça posses a rematar.", fm27: false },
    PlayerInstructionDef { name: "Dribble more", group_pt: "Drible", desc_pt: "Conduz e procura o um-para-um.", tip_pt: "Drible + Agilidade altos.", fm27: false },
    PlayerInstructionDef { name: "Dribble less", group_pt: "Drible", desc_pt: "Solta a bola mais cedo.", tip_pt: "Drible fraco em zonas perigosas.", fm27: false },
    PlayerInstructionDef { name: "Take more risks", group_pt: "Passe", desc_pt: "Tenta passes de ruptura.", tip_pt: "Passe + Visão ≥ 15.", fm27: false },
    PlayerInstructionDef { name: "Take fewer risks", group_pt: "Passe", desc_pt: "Passes seguros.", tip_pt: "Centrais/médios com Passe fraco.", fm27: false },
    PlayerInstructionDef { name: "Cross from byline", group_pt: "Cruzamento", desc_pt: "Vai à linha antes de cruzar.", tip_pt: "Wingers rápidos.", fm27: false },
    PlayerInstructionDef { name: "Cross from deep", group_pt: "Cruzamento", desc_pt: "Cruza mais cedo, de trás.", tip_pt: "Alas e extremos interiores com bom Cruzamento.", fm27: false },
    PlayerInstructionDef { name: "Roam from position", group_pt: "Movimento", desc_pt: "Liberdade para sair da posição.", tip_pt: "Só para jogadores com Decisões/Sem Bola altos; aumenta a complexidade.", fm27: false },
    PlayerInstructionDef { name: "Stay wider", group_pt: "Movimento", desc_pt: "Fica mais aberto.", tip_pt: "Equilibra flancos assimétricos.", fm27: false },
    PlayerInstructionDef { name: "Sit narrower", group_pt: "Movimento", desc_pt: "Fecha para dentro.", tip_pt: "Ajuda a superioridade central.", fm27: false },
    PlayerInstructionDef { name: "Hold position", group_pt: "Movimento", desc_pt: "Fica na sua zona em vez de acompanhar o ataque.", tip_pt: "Conta para a rest-defence.", fm27: false },
    PlayerInstructionDef { name: "Get further forward", group_pt: "Movimento", desc_pt: "Chega mais vezes ao último terço.", tip_pt: "Médios/laterais com Resistência e Sem Bola altos.", fm27: false },
    PlayerInstructionDef { name: "Move into channels", group_pt: "Movimento", desc_pt: "Procura o espaço entre central e lateral.", tip_pt: "Avançados rápidos contra linhas altas.", fm27: false },
    PlayerInstructionDef { name: "Press more", group_pt: "Pressão", desc_pt: "Pressiona mais do que o resto da equipa.", tip_pt: "Índice de Trabalho e Resistência altos.", fm27: false },
    PlayerInstructionDef { name: "Press less", group_pt: "Pressão", desc_pt: "Poupa energia na pressão.", tip_pt: "Resistência baixa.", fm27: false },
    PlayerInstructionDef { name: "Mark tighter", group_pt: "Defesa", desc_pt: "Marcação mais apertada.", tip_pt: "Marcação ≥ 15.", fm27: false },
    PlayerInstructionDef { name: "Tackle harder", group_pt: "Defesa", desc_pt: "Desarmes mais agressivos.", tip_pt: "Desarme e Agressividade altos.", fm27: false },
    PlayerInstructionDef { name: "Ease off tackles", group_pt: "Defesa", desc_pt: "Desarmes mais cautelosos.", tip_pt: "Desarme fraco ou jogador amarelado.", fm27: false },
    PlayerInstructionDef { name: "Arrive Late", group_pt: "Área (FM27)", desc_pt: "Chega à área em segunda vaga, atacando o espaço que se abre.", tip_pt: "Box-to-Box, Attacking Midfielder, Channel Midfielder.", fm27: true },
    PlayerInstructionDef { name: "Attack the Defensive Line", group_pt: "Área (FM27)", desc_pt: "Ataca a profundidade na última linha.", tip_pt: "Poacher, Channel Forward, Inside Forward rápidos.", fm27: true },
    PlayerInstructionDef { name: "Target Near Post", group_pt: "Área (FM27)", desc_pt: "Ataca o primeiro poste nos cruzamentos.", tip_pt: "Avançado rápido, Poacher.", fm27: true },
    PlayerInstructionDef { name: "Target Centre", group_pt: "Área (FM27)", desc_pt: "Ataca o centro da área nos cruzamentos.", tip_pt: "Referência aérea.", fm27: true },
    PlayerInstructionDef { name: "Target Far Post", group_pt: "Área (FM27)", desc_pt: "Ataca o segundo poste nos cruzamentos.", tip_pt: "Extremo do lado oposto, Wide Forward.", fm27: true },
];

pub fn has_player_instruction(name: &str) -> bool {
    PLAYER_INSTRUCTIONS.iter().any(|p| p.name == name)
}

/// 5 FM27 Tactical Tweaks.
pub const TACTICAL_TWEAKS: [TacticalTweakDef; 5] = [
    TacticalTweakDef { name: "More Urgency", desc_pt: "Aumenta a urgência: tipicamente sobe ritmo e verticalidade.", when_pt: "A perder ou empatado contra bloco baixo nos últimos 20–30 min." },
    TacticalTweakDef { name: "Play Longer", desc_pt: "Joga mais directo.", when_pt: "Quando a construção curta está a ser pressionada e perde bolas atrás, ou para explorar uma referência." },
    TacticalTweakDef { name: "Exploit Width", desc_pt: "Estica o jogo pelos corredores.", when_pt: "Adversário estreito (ex. 4-4-2 losango, 3-5-2 sem alas que recuem)." },
    TacticalTweakDef { name: "Press Less", desc_pt: "Baixa a intensidade da pressão.", when_pt: "Jogadores cansados, a ganhar, ou pressão a ser batida sistematicamente." },
    TacticalTweakDef { name: "Wingers Swap Sides", desc_pt: "Troca os extremos de flanco.", when_pt: "Para atacar o lateral mais fraco, ou para meter o extremo no pé forte/fraco conforme o objectivo." },
];

pub fn has_tactical_tweak(name: &str) -> bool {
    TACTICAL_TWEAKS.iter().any(|t| t.name == name)
}

/// 8 FM27 Tactical Understanding training areas.
pub const TRAINING_AREAS: [TrainingAreaDef; 8] = [
    TrainingAreaDef { key: "ip_shape", name: "In-Possession Shape", session_pt: "Tactical Understanding: posse/estrutura ofensiva" },
    TrainingAreaDef { key: "oop_shape", name: "Out-of-Possession Shape", session_pt: "Tactical Understanding: estrutura defensiva" },
    TrainingAreaDef { key: "passing", name: "Passing Style", session_pt: "Tactical Understanding: estilo de passe" },
    TrainingAreaDef { key: "buildup", name: "Build Up", session_pt: "Tactical Understanding: construção" },
    TrainingAreaDef { key: "chance", name: "Chance Creation", session_pt: "Tactical Understanding: criação no último terço" },
    TrainingAreaDef { key: "trigger", name: "Trigger Press", session_pt: "Tactical Understanding: pressing" },
    TrainingAreaDef { key: "trap", name: "Pressing Trap", session_pt: "Tactical Understanding: pressing" },
    TrainingAreaDef { key: "loe", name: "Line of Engagement", session_pt: "Tactical Understanding: pressing" },
];

pub fn training_area(key: &str) -> Option<&'static TrainingAreaDef> {
    TRAINING_AREAS.iter().find(|a| a.key == key)
}

/// 8 tactical templates used by Best XI & Systems suggestions.
pub const TEMPLATES: [Template; 8] = [
    Template {
        id: "possession433", name_pt: "4-3-3 Posse & Contra-pressão", ip: "4-3-3 DM Wide", oop: "4-1-4-1 DM Wide", ment: Mentality::Positive,
        style: "possession_press", desc_pt: "Domínio com bola, saída curta, recuperação imediata.",
        roles: &[("GK", "bpgk", "swk"), ("DL", "wb_ip", "fb_oop"), ("DCL", "bpcb", "ccb"), ("DCR", "cb_ip", "scb"), ("DR", "iwb", "pfb"), ("DM", "dlp", "sdm"), ("MCL", "chm", "pcm"), ("MCR", "mp", "cm_oop"), ("AML", "if_ip", "wm_oop"), ("AMR", "w_ip", "wm_oop"), ("ST", "f9", "tcf")],
        ti: &[("passing", 1), ("tempo", 3), ("reception", 2), ("buildup", 0), ("goal_kicks", 0), ("loe", 2), ("dline", 3), ("trigger", 3), ("def_trans", 2), ("press_trap", 2)],
    },
    Template {
        id: "balanced4231", name_pt: "4-2-3-1 Equilibrado", ip: "4-2-3-1 DM Wide", oop: "4-4-1-1", ment: Mentality::Balanced,
        style: "balanced", desc_pt: "Base sólida e versátil. Fácil de aprender (baixa complexidade).",
        roles: &[("GK", "gk_ip", "gk_oop"), ("DL", "fb_ip", "fb_oop"), ("DCL", "bpcb", "cb_oop"), ("DCR", "cb_ip", "cb_oop"), ("DR", "wb_ip", "fb_oop"), ("DML", "dm_ip", "scm"), ("DMR", "b2b", "cm_oop"), ("AML", "iw_ip", "wm_oop"), ("AMC", "am_ip", "am_oop"), ("AMR", "wf_ip", "wm_oop"), ("ST", "cf_ip", "cf_oop")],
        ti: &[],
    },
    Template {
        id: "gegen4222", name_pt: "4-2-2-2 Vertical Gegenpress", ip: "4-2-2-2", oop: "4-4-2", ment: Mentality::Attacking,
        style: "press_vertical", desc_pt: "Pressão alta, ataques rápidos pelo meio, muitos homens perto da área.",
        roles: &[("GK", "gk_ip", "swk"), ("DL", "wb_ip", "pfb"), ("DCL", "cb_ip", "ccb"), ("DCR", "bpcb", "scb"), ("DR", "wb_ip", "pfb"), ("DML", "dm_ip", "pcm"), ("DMR", "b2b", "pcm"), ("AMCL", "ap", "twm"), ("AMCR", "chm", "twm"), ("STL", "dlf", "tcf"), ("STR", "chf", "tcf")],
        ti: &[("passing", 3), ("tempo", 4), ("reception", 0), ("loe", 2), ("dline", 3), ("trigger", 4), ("def_trans", 2), ("width", 1), ("short_gk", 1)],
    },
    Template {
        id: "counter442", name_pt: "4-4-2 Bloco Médio & Contra-ataque", ip: "4-4-2", oop: "4-4-2", ment: Mentality::Cautious,
        style: "counter", desc_pt: "Compacto sem bola, transições rápidas para dois avançados.",
        roles: &[("GK", "gk_ip", "gk_oop"), ("DL", "fb_ip", "fb_oop"), ("DCL", "cb_ip", "cb_oop"), ("DCR", "cb_ip", "ccb"), ("DR", "fb_ip", "fb_oop"), ("ML", "w_ip", "twm"), ("MCL", "cm_ip", "scm"), ("MCR", "chm", "cm_oop"), ("MR", "iw_ip", "twm"), ("STL", "tf", "cf_oop"), ("STR", "chf", "socf")],
        ti: &[("passing", 3), ("att_trans", 2), ("loe", 1), ("dline", 1), ("reception", 0)],
    },
    Template {
        id: "wb3421", name_pt: "3-4-2-1 Alas & Meios-espaços", ip: "3-4-2-1", oop: "5-4-1", ment: Mentality::Positive,
        style: "possession", desc_pt: "Linha de 3 com alas a dar largura e dois 10 nos meios-espaços.",
        roles: &[("GK", "bpgk", "swk"), ("DCL", "cb_ip", "swcb"), ("DC", "bpcb", "ccb"), ("DCR", "cb_ip", "cwcb"), ("WBL", "awb", "wb_oop"), ("MCL", "mp", "scm"), ("MCR", "chm", "pcm"), ("WBR", "wb_ip", "wb_oop"), ("AMCL", "ap", "twm"), ("AMCR", "am_ip", "wm_oop"), ("ST", "cf_ip", "cf_oop")],
        ti: &[("passing", 1), ("buildup", 0), ("reception", 2), ("dline", 3), ("loe", 1)],
    },
    Template {
        id: "lowblock532", name_pt: "5-3-2 Bloco Baixo", ip: "3-5-2", oop: "5-3-2 WB", ment: Mentality::Defensive,
        style: "low_block", desc_pt: "Defende a área, sai rápido com dois avançados complementares.",
        roles: &[("GK", "nngk", "lhk"), ("DCL", "cb_ip", "cwcb"), ("DC", "nncb", "cb_oop"), ("DCR", "cb_ip", "cwcb"), ("WBL", "wb_ip", "hwb"), ("DM", "dlp", "scm"), ("MCL", "cm_ip", "wccm"), ("MCR", "chm", "wccm"), ("WBR", "wb_ip", "hwb"), ("STL", "tf", "cocf"), ("STR", "poa", "socf")],
        ti: &[("passing", 3), ("buildup", 2), ("goal_kicks", 2), ("att_trans", 2), ("loe", 0), ("dline", 1), ("cross_eng", 0), ("tackling", 2)],
    },
    Template {
        id: "diamond", name_pt: "4-4-2 Losango de Controlo", ip: "4-4-2 Diamond", oop: "4-4-2 Diamond", ment: Mentality::Positive,
        style: "possession", desc_pt: "Superioridade central, laterais dão toda a largura.",
        roles: &[("GK", "gk_ip", "gk_oop"), ("DL", "wb_ip", "fb_oop"), ("DCL", "bpcb", "cb_oop"), ("DCR", "cb_ip", "ccb"), ("DR", "wb_ip", "fb_oop"), ("DM", "dlp", "sdm"), ("MCL", "wcm", "wccm"), ("MCR", "chm", "pcm"), ("AMC", "ap", "am_oop"), ("STL", "dlf", "cf_oop"), ("STR", "chf", "cf_oop")],
        ti: &[("passing", 1), ("width", 3), ("support_runs", 2), ("reception", 2)],
    },
    Template {
        id: "wide4141", name_pt: "4-1-4-1 Pressão Equilibrada", ip: "4-1-4-1 DM Wide", oop: "4-1-4-1 DM Wide", ment: Mentality::Balanced,
        style: "press_balanced", desc_pt: "Bloco médio-alto, extremos abertos, um avançado de referência.",
        roles: &[("GK", "gk_ip", "swk"), ("DL", "ifb", "hfb"), ("DCL", "bpcb", "cb_oop"), ("DCR", "cb_ip", "cb_oop"), ("DR", "wb_ip", "pfb"), ("DM", "hb", "sdm"), ("ML", "w_ip", "wm_oop"), ("MCL", "am_ip", "pcm"), ("MCR", "mp", "cm_oop"), ("MR", "iw_ip", "wm_oop"), ("ST", "cf_ip", "tcf")],
        ti: &[("loe", 1), ("trigger", 3), ("press_trap", 2)],
    },
];

pub fn template_by_id(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.id == id)
}

/// 7 style attribute profiles used by `suggest_tactics`.
pub const STYLE_PROFILES: [StyleProfile; 7] = [
    StyleProfile { id: "possession_press", name_pt: "Posse + Contra-pressão", attrs: &[Attr::Pas, Attr::Tec, Attr::Fir, Attr::Cmp, Attr::Dec, Attr::Wor, Attr::Sta, Attr::Tea, Attr::Ant] },
    StyleProfile { id: "possession", name_pt: "Posse", attrs: &[Attr::Pas, Attr::Tec, Attr::Fir, Attr::Cmp, Attr::Dec, Attr::Vis, Attr::Tea] },
    StyleProfile { id: "balanced", name_pt: "Equilibrado", attrs: &[Attr::Dec, Attr::Tea, Attr::Pos, Attr::Pas, Attr::Wor] },
    StyleProfile { id: "press_vertical", name_pt: "Pressão + verticalidade", attrs: &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg, Attr::Otb, Attr::Fin, Attr::Dec] },
    StyleProfile { id: "press_balanced", name_pt: "Pressão moderada", attrs: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Ant, Attr::Pos, Attr::Pas] },
    StyleProfile { id: "counter", name_pt: "Contra-ataque", attrs: &[Attr::Pac, Attr::Acc, Attr::Otb, Attr::Fin, Attr::Pos, Attr::Tea, Attr::Str] },
    StyleProfile { id: "low_block", name_pt: "Bloco baixo", attrs: &[Attr::Pos, Attr::Cnt, Attr::Mar, Attr::Hea, Attr::Str, Attr::Tea, Attr::Cmp] },
];

pub fn style_profile(id: &str) -> Option<&'static StyleProfile> {
    STYLE_PROFILES.iter().find(|s| s.id == id)
}

/// FM27 Team Instructions screen sections (used by Generator report & FM27 Sheet).
pub const FM_SECTIONS: [FmSection; 6] = [
    FmSection { title: "Team Instructions › In Possession › Overview", keys: &["passing", "tempo", "width", "att_trans", "creative", "timewaste", "set_pieces"] },
    FmSection { title: "In Possession › Final Third", keys: &["patience", "long_shots", "crossing", "dribbling"] },
    FmSection { title: "In Possession › Progression", keys: &["support_runs", "progress", "reception"] },
    FmSection { title: "In Possession › Buildup", keys: &["buildup", "goal_kicks", "gk_speed", "gk_target"] },
    FmSection { title: "Out of Possession › Overview", keys: &["loe", "dline", "dl_behaviour", "trigger", "def_trans", "tackling"] },
    FmSection { title: "Out of Possession › High Press / Mid Block / Low Block", keys: &["short_gk", "press_trap", "cross_eng"] },
];
