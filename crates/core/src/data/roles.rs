//! Static FM26/FM27 attributes, position groups, and 77 IP/OOP roles.
//! Ported 1:1 from `src/core/roles.js`.

use crate::types::{Attr, AttrCat, AttrMeta, Lang, Phase, PosGroup, Role, RoleTag, SlotConstraint};

/// Metadata for all 47 FM player attributes (ordered identically to `Attr::ALL`).
pub const ATTR_META: [AttrMeta; Attr::COUNT] = [
    AttrMeta { attr: Attr::Cor, pt: "Cantos", en: "Corners", cat: AttrCat::Technical, aliases: &["cor", "corners"] },
    AttrMeta { attr: Attr::Cro, pt: "Cruzamentos", en: "Crossing", cat: AttrCat::Technical, aliases: &["cro", "crossing", "crossing"] },
    AttrMeta { attr: Attr::Dri, pt: "Drible", en: "Dribbling", cat: AttrCat::Technical, aliases: &["dri", "dribbling"] },
    AttrMeta { attr: Attr::Fin, pt: "Finalização", en: "Finishing", cat: AttrCat::Technical, aliases: &["fin", "finishing"] },
    AttrMeta { attr: Attr::Fir, pt: "Primeiro Toque", en: "First Touch", cat: AttrCat::Technical, aliases: &["fir", "first touch", "firsttouch"] },
    AttrMeta { attr: Attr::Fre, pt: "Livres", en: "Free Kick Taking", cat: AttrCat::Technical, aliases: &["fre", "free kick taking", "free kicks"] },
    AttrMeta { attr: Attr::Hea, pt: "Cabeceamento", en: "Heading", cat: AttrCat::Technical, aliases: &["hea", "heading"] },
    AttrMeta { attr: Attr::Lon, pt: "Remates de Longe", en: "Long Shots", cat: AttrCat::Technical, aliases: &["lon", "long shots"] },
    AttrMeta { attr: Attr::Lth, pt: "Lançamentos Longos", en: "Long Throws", cat: AttrCat::Technical, aliases: &["l th", "lth", "long throws"] },
    AttrMeta { attr: Attr::Mar, pt: "Marcação", en: "Marking", cat: AttrCat::Technical, aliases: &["mar", "marking"] },
    AttrMeta { attr: Attr::Pas, pt: "Passe", en: "Passing", cat: AttrCat::Technical, aliases: &["pas", "passing"] },
    AttrMeta { attr: Attr::Pen, pt: "Penáltis", en: "Penalty Taking", cat: AttrCat::Technical, aliases: &["pen", "penalty taking"] },
    AttrMeta { attr: Attr::Tck, pt: "Desarme", en: "Tackling", cat: AttrCat::Technical, aliases: &["tck", "tackling"] },
    AttrMeta { attr: Attr::Tec, pt: "Técnica", en: "Technique", cat: AttrCat::Technical, aliases: &["tec", "technique"] },
    AttrMeta { attr: Attr::Agg, pt: "Agressividade", en: "Aggression", cat: AttrCat::Mental, aliases: &["agg", "aggression"] },
    AttrMeta { attr: Attr::Ant, pt: "Antecipação", en: "Anticipation", cat: AttrCat::Mental, aliases: &["ant", "anticipation"] },
    AttrMeta { attr: Attr::Bra, pt: "Bravura", en: "Bravery", cat: AttrCat::Mental, aliases: &["bra", "bravery"] },
    AttrMeta { attr: Attr::Cmp, pt: "Compostura", en: "Composure", cat: AttrCat::Mental, aliases: &["cmp", "composure"] },
    AttrMeta { attr: Attr::Cnt, pt: "Concentração", en: "Concentration", cat: AttrCat::Mental, aliases: &["cnt", "concentration"] },
    AttrMeta { attr: Attr::Dec, pt: "Decisões", en: "Decisions", cat: AttrCat::Mental, aliases: &["dec", "decisions"] },
    AttrMeta { attr: Attr::Det, pt: "Determinação", en: "Determination", cat: AttrCat::Mental, aliases: &["det", "determination"] },
    AttrMeta { attr: Attr::Fla, pt: "Talento", en: "Flair", cat: AttrCat::Mental, aliases: &["fla", "flair"] },
    AttrMeta { attr: Attr::Ldr, pt: "Liderança", en: "Leadership", cat: AttrCat::Mental, aliases: &["ldr", "leadership"] },
    AttrMeta { attr: Attr::Otb, pt: "Sem Bola", en: "Off the Ball", cat: AttrCat::Mental, aliases: &["otb", "off the ball"] },
    AttrMeta { attr: Attr::Pos, pt: "Posicionamento", en: "Positioning", cat: AttrCat::Mental, aliases: &["pos", "positioning"] },
    AttrMeta { attr: Attr::Tea, pt: "Trabalho de Equipa", en: "Teamwork", cat: AttrCat::Mental, aliases: &["tea", "teamwork", "team work"] },
    AttrMeta { attr: Attr::Vis, pt: "Visão", en: "Vision", cat: AttrCat::Mental, aliases: &["vis", "vision"] },
    AttrMeta { attr: Attr::Wor, pt: "Índice de Trabalho", en: "Work Rate", cat: AttrCat::Mental, aliases: &["wor", "work rate", "workrate"] },
    AttrMeta { attr: Attr::Acc, pt: "Aceleração", en: "Acceleration", cat: AttrCat::Physical, aliases: &["acc", "acceleration"] },
    AttrMeta { attr: Attr::Agi, pt: "Agilidade", en: "Agility", cat: AttrCat::Physical, aliases: &["agi", "agility"] },
    AttrMeta { attr: Attr::Bal, pt: "Equilíbrio", en: "Balance", cat: AttrCat::Physical, aliases: &["bal", "balance"] },
    AttrMeta { attr: Attr::Jum, pt: "Impulsão", en: "Jumping Reach", cat: AttrCat::Physical, aliases: &["jum", "jumping reach", "jumping"] },
    AttrMeta { attr: Attr::Nat, pt: "Aptidão Natural", en: "Natural Fitness", cat: AttrCat::Physical, aliases: &["nat", "natural fitness"] },
    AttrMeta { attr: Attr::Pac, pt: "Velocidade", en: "Pace", cat: AttrCat::Physical, aliases: &["pac", "pace"] },
    AttrMeta { attr: Attr::Sta, pt: "Resistência", en: "Stamina", cat: AttrCat::Physical, aliases: &["sta", "stamina"] },
    AttrMeta { attr: Attr::Str, pt: "Força", en: "Strength", cat: AttrCat::Physical, aliases: &["str", "strength"] },
    AttrMeta { attr: Attr::Aer, pt: "Alcance Aéreo", en: "Aerial Reach", cat: AttrCat::Goalkeeping, aliases: &["aer", "aerial reach", "aerial ability"] },
    AttrMeta { attr: Attr::Cmd, pt: "Domínio da Área", en: "Command of Area", cat: AttrCat::Goalkeeping, aliases: &["cmd", "command of area"] },
    AttrMeta { attr: Attr::Com, pt: "Comunicação", en: "Communication", cat: AttrCat::Goalkeeping, aliases: &["com", "communication"] },
    AttrMeta { attr: Attr::Ecc, pt: "Excentricidade", en: "Eccentricity", cat: AttrCat::Goalkeeping, aliases: &["ecc", "eccentricity"] },
    AttrMeta { attr: Attr::Han, pt: "Jogo de Mãos", en: "Handling", cat: AttrCat::Goalkeeping, aliases: &["han", "handling"] },
    AttrMeta { attr: Attr::Kic, pt: "Pontapé", en: "Kicking", cat: AttrCat::Goalkeeping, aliases: &["kic", "kicking"] },
    AttrMeta { attr: Attr::Ovo, pt: "Um para Um", en: "One on Ones", cat: AttrCat::Goalkeeping, aliases: &["1v1", "ovo", "one on ones"] },
    AttrMeta { attr: Attr::Pun, pt: "Socos", en: "Punching (Tendency)", cat: AttrCat::Goalkeeping, aliases: &["pun", "punching (tendency)", "punching"] },
    AttrMeta { attr: Attr::Ref, pt: "Reflexos", en: "Reflexes", cat: AttrCat::Goalkeeping, aliases: &["ref", "reflexes"] },
    AttrMeta { attr: Attr::Tro, pt: "Saídas", en: "Rushing Out (Tendency)", cat: AttrCat::Goalkeeping, aliases: &["tro", "rushing out (tendency)", "rushing out"] },
    AttrMeta { attr: Attr::Thr, pt: "Reposição à Mão", en: "Throwing", cat: AttrCat::Goalkeeping, aliases: &["thr", "throwing"] },
];

#[inline]
pub fn attr_meta(attr: Attr) -> &'static AttrMeta {
    &ATTR_META[attr.index()]
}

#[inline]
pub fn attr_name(attr: Attr, lang: Lang) -> &'static str {
    let m = attr_meta(attr);
    match lang {
        Lang::En => m.en,
        Lang::Pt => m.pt,
    }
}

/// Normalize a CSV/table column header identically to JS `normHeader`.
pub fn norm_header(h: &str) -> String {
    let mut out = String::with_capacity(h.len());
    let mut prev_space = true;
    for ch in h.trim().to_lowercase().chars() {
        let c = if ch == '_' || ch == '-' || ch.is_whitespace() { ' ' } else { ch };
        if c == ' ' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    let trimmed = out.trim().trim_end_matches(['.', ':']);
    for prefix in [
        "gk attributes ",
        "goalkeeping ",
        "goalkeeper ",
        "attributes ",
        "attribute ",
        "technical ",
        "mental ",
        "physical ",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    trimmed.to_string()
}

/// Look up an `Attr` by its normalized CSV/FM alias or normalized PT name.
pub fn attr_from_norm_header(norm: &str) -> Option<Attr> {
    for m in &ATTR_META {
        if m.aliases.iter().any(|&a| a == norm) || norm_header(m.pt) == norm {
            return Some(m.attr);
        }
    }
    None
}

/// Portuguese labels for the 10 position groups.
pub const fn group_name_pt(group: PosGroup) -> &'static str {
    match group {
        PosGroup::Gk => "Guarda-redes",
        PosGroup::Dc => "Defesa central",
        PosGroup::DLr => "Lateral (D E/D)",
        PosGroup::WbLr => "Ala (WB E/D)",
        PosGroup::Dm => "Médio defensivo",
        PosGroup::Mc => "Médio centro",
        PosGroup::MLr => "Médio ala (M E/D)",
        PosGroup::Amc => "Médio ofensivo centro",
        PosGroup::AmLr => "Extremo (AM E/D)",
        PosGroup::St => "Avançado",
    }
}

/// Localized label for a position group.
pub fn group_name(group: PosGroup, lang: Lang) -> String {
    match lang {
        Lang::Pt => group_name_pt(group).to_string(),
        Lang::En => crate::data::i18n::t(Lang::En, group_name_pt(group)),
    }
}

/// All 77 FM26/FM27 IP & OOP roles.
pub const ROLES: [Role; 77] = [
    Role {
        id: "gk_ip", ab: "GK", name: "Goalkeeper", phase: Phase::Ip,
        groups: &[PosGroup::Gk], tags: &[RoleTag::Rd],
        key: &[Attr::Han, Attr::Ref, Attr::Pos, Attr::Cnt, Attr::Ovo, Attr::Agi, Attr::Aer], pref: &[Attr::Kic, Attr::Thr, Attr::Com, Attr::Dec, Attr::Ant, Attr::Cmd],
        desc_pt: "Guarda-redes equilibrado na posse: distribui de forma segura conforme as instruções de equipa.", tip_pt: "Base sólida quando o GR não é bom com os pés; combina com Goal Kicks \"Mixed\".",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "bpgk", ab: "BPG", name: "Ball-Playing Goalkeeper", phase: Phase::Ip,
        groups: &[PosGroup::Gk], tags: &[RoleTag::Rd, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Kic, Attr::Thr, Attr::Cmp, Attr::Dec, Attr::Vis], pref: &[Attr::Han, Attr::Ref, Attr::Tec, Attr::Ant, Attr::Cnt, Attr::Pos],
        desc_pt: "Participa activamente na construção, assume riscos no passe e pode sair da baliza para servir de apoio.", tip_pt: "Essencial com \"Play Through Press\". Sem Passe/Compostura ≥ 12 é uma fonte de perdas de bola perigosas.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "nngk", ab: "NNG", name: "No-Nonsense Goalkeeper", phase: Phase::Ip,
        groups: &[PosGroup::Gk], tags: &[RoleTag::Rd],
        key: &[Attr::Kic, Attr::Han, Attr::Ref, Attr::Aer, Attr::Pos, Attr::Cnt], pref: &[Attr::Com, Attr::Cmd, Attr::Str],
        desc_pt: "Joga pelo seguro: alivia, bate longo, evita riscos na construção.", tip_pt: "Bom par para \"Bypass Press\" e Goal Kicks \"Long\" com um Target Forward.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "gk_oop", ab: "GK", name: "Goalkeeper", phase: Phase::Oop,
        groups: &[PosGroup::Gk], tags: &[],
        key: &[Attr::Han, Attr::Ref, Attr::Pos, Attr::Cnt, Attr::Ovo, Attr::Agi], pref: &[Attr::Aer, Attr::Cmd, Attr::Com, Attr::Ant, Attr::Dec],
        desc_pt: "Comportamento defensivo padrão: alterna entre ficar na linha e sair consoante a situação.", tip_pt: "Escolha segura com linha defensiva média.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "lhk", ab: "LHK", name: "Line-Holding Keeper", phase: Phase::Oop,
        groups: &[PosGroup::Gk], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Cnt, Attr::Ref, Attr::Han, Attr::Agi], pref: &[Attr::Ovo, Attr::Dec],
        desc_pt: "Fica na linha de golo e reage. Bom para equipas que defendem a área em bloco baixo.", tip_pt: "Funciona melhor com linha defensiva baixa; com linha alta deixa espaço enorme nas costas.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "swk", ab: "SK", name: "Sweeper Keeper", phase: Phase::Oop,
        groups: &[PosGroup::Gk], tags: &[RoleTag::Hp],
        key: &[Attr::Cmd, Attr::Ovo, Attr::Tro, Attr::Ref, Attr::Ant, Attr::Dec, Attr::Cmp, Attr::Acc], pref: &[Attr::Aer, Attr::Han, Attr::Pas, Attr::Fir, Attr::Kic, Attr::Pos, Attr::Agi],
        desc_pt: "Sai da baliza para varrer bolas nas costas da defesa.", tip_pt: "Obrigatório com linha defensiva alta. Exige Antecipação, Decisões e Saídas.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "cb_ip", ab: "CB", name: "Centre-Back", phase: Phase::Ip,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Rd],
        key: &[Attr::Hea, Attr::Mar, Attr::Tck, Attr::Pos, Attr::Str, Attr::Jum], pref: &[Attr::Cmp, Attr::Dec, Attr::Cnt, Attr::Ant, Attr::Pac],
        desc_pt: "Mantém a posição na construção, passes simples para o médio ou lateral.", tip_pt: "O \"default\" que nunca compromete a rest-defence.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "bpcb", ab: "BPD", name: "Ball-Playing Centre-Back", phase: Phase::Ip,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Rd, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Cmp, Attr::Dec, Attr::Hea, Attr::Mar, Attr::Tck, Attr::Pos, Attr::Str], pref: &[Attr::Vis, Attr::Fir, Attr::Tec, Attr::Ant, Attr::Cnt, Attr::Jum],
        desc_pt: "Mantém posição mas é expressivo com bola: procura passes progressivos e mudanças de flanco.", tip_pt: "Um BPD + um DC simples é a dupla mais estável. Dois BPD só com médio defensivo seguro.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "nncb", ab: "NCB", name: "No-Nonsense Centre-Back", phase: Phase::Ip,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Rd],
        key: &[Attr::Hea, Attr::Mar, Attr::Tck, Attr::Pos, Attr::Str, Attr::Jum, Attr::Bra], pref: &[Attr::Agg, Attr::Cnt, Attr::Ant],
        desc_pt: "Cuidadoso: alivia sem hesitar, não arrisca na construção.", tip_pt: "Contradiz \"Play Through Press\". Use em bloco baixo / jogo directo.",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "acb", ab: "ACB", name: "Advanced Centre-Back", phase: Phase::Ip,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Adv, RoleTag::ToDm, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Dec, Attr::Vis, Attr::Pos, Attr::Tck], pref: &[Attr::Ant, Attr::Tea, Attr::Sta, Attr::Dri],
        desc_pt: "Sobe para o meio-campo (zona de médio defensivo) na posse, criando superioridade central.", tip_pt: "Garanta cobertura: com ACB, deixe 2 centrais/IFB atrás ou um Half Back.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "ocb", ab: "OCB", name: "Overlapping Centre-Back", phase: Phase::Ip,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Adv, RoleTag::W],
        key: &[Attr::Cro, Attr::Dri, Attr::Tck, Attr::Mar, Attr::Pos, Attr::Wor, Attr::Sta, Attr::Pac], pref: &[Attr::Otb, Attr::Tea, Attr::Acc, Attr::Hea],
        desc_pt: "Central do lado numa linha de 3 que sobrepõe pelo corredor no último terço.", tip_pt: "Só faz sentido em linha de 3. Precisa de resistência e um ala que entre por dentro para não se atropelarem.",
        verified: true, wide_cb: true, slot: None,
    },
    Role {
        id: "cb_oop", ab: "CB", name: "Centre-Back", phase: Phase::Oop,
        groups: &[PosGroup::Dc], tags: &[],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Hea, Attr::Str, Attr::Cnt], pref: &[Attr::Ant, Attr::Dec, Attr::Jum, Attr::Cmp],
        desc_pt: "Defende na linha com comportamento equilibrado: sai ao portador quando faz sentido.", tip_pt: "Escolha base; ajuste com o papel do parceiro (um sai, outro cobre).",
        verified: false, wide_cb: false, slot: None,
    },
    Role {
        id: "ccb", ab: "CCD", name: "Covering Centre-Back", phase: Phase::Oop,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Ant, Attr::Cnt, Attr::Dec, Attr::Pac], pref: &[Attr::Mar, Attr::Tck, Attr::Acc, Attr::Cmp],
        desc_pt: "Cauteloso, mantém-se ligeiramente atrás da linha para cobrir a profundidade.", tip_pt: "Clássico par de um Stopping Centre-Back. Ideal com linha alta e central rápido.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "scb", ab: "SCD", name: "Stopping Centre-Back", phase: Phase::Oop,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Hp],
        key: &[Attr::Tck, Attr::Agg, Attr::Bra, Attr::Ant, Attr::Dec, Attr::Str], pref: &[Attr::Mar, Attr::Hea, Attr::Acc, Attr::Jum],
        desc_pt: "Assertivo: sai da linha para pressionar o jogador entre linhas.", tip_pt: "Excelente contra um médio ofensivo/falso 9. Precisa de um Covering ao lado.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cwcb", ab: "CWD", name: "Covering Wide Centre-Back", phase: Phase::Oop,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Low, RoleTag::Cw],
        key: &[Attr::Pos, Attr::Ant, Attr::Cnt, Attr::Pac, Attr::Mar], pref: &[Attr::Tck, Attr::Dec],
        desc_pt: "Central do lado (linha de 3) que defende reactivamente e mais fundo, cobrindo o canal.", tip_pt: "Proteja as costas de um ala muito ofensivo.",
        verified: true, wide_cb: true, slot: None,
    },
    Role {
        id: "swcb", ab: "SWD", name: "Stopping Wide Centre-Back", phase: Phase::Oop,
        groups: &[PosGroup::Dc], tags: &[RoleTag::Hp],
        key: &[Attr::Tck, Attr::Agg, Attr::Ant, Attr::Acc, Attr::Wor, Attr::Mar], pref: &[Attr::Str, Attr::Bra, Attr::Dec],
        desc_pt: "Central do lado (linha de 3) que sai agressivamente para pressionar nos meios-espaços e corredor.", tip_pt: "Muito bom contra AMCL/AMCR adversários. Mais liberdade que numa linha de 4.",
        verified: true, wide_cb: true, slot: None,
    },
    Role {
        id: "fb_ip", ab: "FB", name: "Full-Back", phase: Phase::Ip,
        groups: &[PosGroup::DLr], tags: &[RoleTag::W, RoleTag::Rd],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Cnt, Attr::Ant, Attr::Tea], pref: &[Attr::Cro, Attr::Pas, Attr::Dec, Attr::Wor, Attr::Sta],
        desc_pt: "Mantém posição e largura; apoia sem se lançar constantemente.", tip_pt: "Seguro. Combine com um extremo que vá por dentro para manter a largura.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "ifb", ab: "IFB", name: "Inside Full-Back", phase: Phase::Ip,
        groups: &[PosGroup::DLr], tags: &[RoleTag::Rd, RoleTag::In, RoleTag::ToCb],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Hea, Attr::Str, Attr::Cmp], pref: &[Attr::Pas, Attr::Dec, Attr::Cnt, Attr::Ant, Attr::Jum],
        desc_pt: "Fecha por dentro e torna-se terceiro central na posse (linha de 3 na construção).", tip_pt: "Óptimo para rest-defence. O outro lado pode ter um lateral ofensivo.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wb_ip", ab: "WB", name: "Wing-Back", phase: Phase::Ip,
        groups: &[PosGroup::DLr, PosGroup::WbLr], tags: &[RoleTag::W, RoleTag::Adv],
        key: &[Attr::Cro, Attr::Dri, Attr::Tck, Attr::Mar, Attr::Tea, Attr::Wor, Attr::Sta, Attr::Acc], pref: &[Attr::Otb, Attr::Ant, Attr::Pos, Attr::Pas, Attr::Pac, Attr::Agi],
        desc_pt: "Sobe para dar largura e cruzar; aparece bastante no último terço.", tip_pt: "Precisa de Resistência alta. Combine com Inside Forward/Inside Winger à frente.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "iwb", ab: "IWB", name: "Inside Wing-Back", phase: Phase::Ip,
        groups: &[PosGroup::DLr, PosGroup::WbLr], tags: &[RoleTag::In, RoleTag::Rd, RoleTag::ToDm],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Dec, Attr::Tea, Attr::Pos], pref: &[Attr::Tck, Attr::Vis, Attr::Ant, Attr::Mar],
        desc_pt: "Mantém posição e entra para o meio (zona de médio defensivo) na posse.", tip_pt: "Cria um 3-2 na construção. Exige um extremo que dê largura no mesmo lado.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "pwb", ab: "PWB", name: "Playmaking Wing-Back", phase: Phase::Ip,
        groups: &[PosGroup::DLr, PosGroup::WbLr], tags: &[RoleTag::In, RoleTag::ToDm, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Vis, Attr::Cmp, Attr::Dec, Attr::Tea], pref: &[Attr::Dri, Attr::Ant, Attr::Otb, Attr::Fla, Attr::Wor],
        desc_pt: "Entra para o meio e assume a criação a partir de trás. Expressivo.", tip_pt: "Máximo um por lado/tática. Tire responsabilidade criativa ao DLP se usar ambos.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "awb", ab: "CWB", name: "Advanced Wing-Back", phase: Phase::Ip,
        groups: &[PosGroup::WbLr], tags: &[RoleTag::W, RoleTag::Adv, RoleTag::Run],
        key: &[Attr::Cro, Attr::Dri, Attr::Tec, Attr::Otb, Attr::Tea, Attr::Wor, Attr::Acc, Attr::Pac, Attr::Sta], pref: &[Attr::Fir, Attr::Pas, Attr::Fla, Attr::Ant, Attr::Dec, Attr::Agi],
        desc_pt: "Vai ainda mais à frente, quase como extremo. Considerado dos papéis mais fortes no FM26.", tip_pt: "Deixa o flanco exposto: compense com Covering Wide CB / Wide Covering CM.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "fb_oop", ab: "FB", name: "Full-Back", phase: Phase::Oop,
        groups: &[PosGroup::DLr], tags: &[],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Cnt, Attr::Ant], pref: &[Attr::Dec, Attr::Tea, Attr::Acc],
        desc_pt: "Defesa padrão do flanco, mantém a linha.", tip_pt: "Base equilibrada.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "hfb", ab: "HFB", name: "Holding Full-Back", phase: Phase::Oop,
        groups: &[PosGroup::DLr], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Cnt, Attr::Mar, Attr::Dec, Attr::Str], pref: &[Attr::Tck, Attr::Ant, Attr::Hea],
        desc_pt: "Fica mais fundo e estreito para proteger o central contra ataques directos; envolve-se tarde.", tip_pt: "Bom com o lado oposto a pressionar alto (estrutura assimétrica).",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "pfb", ab: "PFB", name: "Pressing Full-Back", phase: Phase::Oop,
        groups: &[PosGroup::DLr], tags: &[RoleTag::Hp],
        key: &[Attr::Wor, Attr::Ant, Attr::Agg, Attr::Acc, Attr::Tck], pref: &[Attr::Sta, Attr::Tea, Attr::Dec],
        desc_pt: "Sai agressivamente aos extremos adversários alto no campo.", tip_pt: "Combina com Pressing Trap \"Outside\". Deixa espaço nas costas — precisa de Aceleração.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wb_oop", ab: "WB", name: "Wing-Back", phase: Phase::Oop,
        groups: &[PosGroup::WbLr], tags: &[RoleTag::Trk],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Wor, Attr::Sta, Attr::Acc], pref: &[Attr::Ant, Attr::Tea],
        desc_pt: "Recua para formar linha de 5 quando necessário.", tip_pt: "Base para 3 atrás.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "hwb", ab: "HWB", name: "Holding Wing-Back", phase: Phase::Oop,
        groups: &[PosGroup::WbLr], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Cnt, Attr::Mar, Attr::Dec], pref: &[Attr::Tck, Attr::Ant, Attr::Str],
        desc_pt: "Cauteloso, mantém posição numa linha de 5.", tip_pt: "Bloco baixo 5-4-1 / 5-3-2.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "prwb", ab: "PWB", name: "Pressing Wing-Back", phase: Phase::Oop,
        groups: &[PosGroup::WbLr], tags: &[RoleTag::Hp],
        key: &[Attr::Wor, Attr::Agg, Attr::Ant, Attr::Acc, Attr::Sta], pref: &[Attr::Tck, Attr::Tea],
        desc_pt: "Sobe para pressionar o lateral/extremo adversário.", tip_pt: "Ideal num 3-4-3 de pressão.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "dm_ip", ab: "DM", name: "Defensive Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Rd],
        key: &[Attr::Tck, Attr::Pos, Attr::Ant, Attr::Cnt, Attr::Dec, Attr::Tea, Attr::Mar], pref: &[Attr::Pas, Attr::Cmp, Attr::Str, Attr::Wor, Attr::Sta],
        desc_pt: "Mantém posição à frente da defesa; circula a bola com cuidado.", tip_pt: "A âncora da rest-defence. Em quase todas as táticas pelo menos um destes \"seguros\".",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "dlp", ab: "DLP", name: "Deep-Lying Playmaker", phase: Phase::Ip,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Rd, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Dec, Attr::Vis, Attr::Tea], pref: &[Attr::Ant, Attr::Pos, Attr::Bal],
        desc_pt: "Mantém posição e é o principal organizador a partir de trás. Expressivo.", tip_pt: "Precisa de linhas de passe: combine com IWB ou BPD. Evite com \"Bypass Press\".",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "hb", ab: "HB", name: "Half Back", phase: Phase::Ip,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Rd, RoleTag::Drop, RoleTag::ToCb],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Ant, Attr::Cmp, Attr::Cnt, Attr::Dec, Attr::Tea], pref: &[Attr::Pas, Attr::Str, Attr::Jum, Attr::Fir],
        desc_pt: "Baixa para entre os centrais na construção, formando linha de 3.", tip_pt: "Permite aos laterais subirem os dois. Redundante com IFB do mesmo lado.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "b2b", ab: "BBM", name: "Box-To-Box Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Adv, RoleTag::Box],
        key: &[Attr::Pas, Attr::Tck, Attr::Otb, Attr::Tea, Attr::Wor, Attr::Sta], pref: &[Attr::Dri, Attr::Fin, Attr::Fir, Attr::Lon, Attr::Ant, Attr::Cmp, Attr::Dec, Attr::Pos, Attr::Acc, Attr::Str, Attr::Bal],
        desc_pt: "Sobe até à zona de médio ofensivo e chega à área.", tip_pt: "Resistência e Índice de Trabalho são inegociáveis.",
        verified: true, wide_cb: false, slot: Some(SlotConstraint::Side),
    },
    Role {
        id: "b2bp", ab: "BBP", name: "Box-To-Box Playmaker", phase: Phase::Ip,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Adv, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Vis, Attr::Dec, Attr::Tea, Attr::Wor, Attr::Sta], pref: &[Attr::Dri, Attr::Lon, Attr::Otb, Attr::Ant, Attr::Cmp],
        desc_pt: "Sobe à zona de médio ofensivo como criativo.", tip_pt: "Bom num duplo pivot com um DM seguro.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "dm_oop", ab: "DM", name: "Defensive Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Dm], tags: &[],
        key: &[Attr::Tck, Attr::Pos, Attr::Ant, Attr::Cnt, Attr::Dec, Attr::Mar], pref: &[Attr::Tea, Attr::Wor, Attr::Str, Attr::Sta],
        desc_pt: "Comportamento defensivo padrão à frente da defesa.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "ddm", ab: "DDM", name: "Dropping Defensive Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Low, RoleTag::Drop],
        key: &[Attr::Mar, Attr::Tck, Attr::Pos, Attr::Hea, Attr::Cnt, Attr::Str], pref: &[Attr::Ant, Attr::Dec, Attr::Jum],
        desc_pt: "Baixa para a linha de centrais sem bola (linha de 5).", tip_pt: "Bloco baixo contra 2 avançados.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "sdm", ab: "SDM", name: "Screening Defensive Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Cnt, Attr::Dec, Attr::Ant], pref: &[Attr::Mar, Attr::Tck, Attr::Tea],
        desc_pt: "Cauteloso, fecha linhas de passe centrais e não se deixa arrastar.", tip_pt: "Protecção da zona 14.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wcdm", ab: "WCDM", name: "Wide Covering Defensive Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Cw],
        key: &[Attr::Wor, Attr::Sta, Attr::Pos, Attr::Tea, Attr::Ant, Attr::Tck], pref: &[Attr::Acc, Attr::Dec],
        desc_pt: "Desloca-se para cobrir o flanco quando o lateral sobe.", tip_pt: "Par natural de um ala muito ofensivo.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "pdm", ab: "PDM", name: "Pressing Defensive Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Dm], tags: &[RoleTag::Hp],
        key: &[Attr::Wor, Attr::Agg, Attr::Ant, Attr::Sta, Attr::Tck, Attr::Acc], pref: &[Attr::Tea, Attr::Dec, Attr::Bra],
        desc_pt: "Sai para pressionar alto a partir da posição de médio defensivo.", tip_pt: "Gegenpress. Deixe outro médio a fechar.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cm_ip", ab: "CM", name: "Central Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Mc], tags: &[RoleTag::Link],
        key: &[Attr::Pas, Attr::Tck, Attr::Dec, Attr::Tea, Attr::Fir], pref: &[Attr::Ant, Attr::Cmp, Attr::Cnt, Attr::Vis, Attr::Wor, Attr::Sta, Attr::Tec, Attr::Otb],
        desc_pt: "Equilibrado: apoia a construção e chega ao ataque com moderação.", tip_pt: "Cola que equilibra o meio-campo.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "mp", ab: "MP", name: "Midfield Playmaker", phase: Phase::Ip,
        groups: &[PosGroup::Mc], tags: &[RoleTag::Cr, RoleTag::Link],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Dec, Attr::Vis, Attr::Tea], pref: &[Attr::Ant, Attr::Otb, Attr::Fla, Attr::Bal, Attr::Agi],
        desc_pt: "Organizador do meio-campo, expressivo, pede a bola.", tip_pt: "Um só criador central por tática costuma bastar.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wcm", ab: "WCM", name: "Wide Central Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Mc], tags: &[RoleTag::W],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Dec, Attr::Otb, Attr::Tea, Attr::Wor, Attr::Sta], pref: &[Attr::Cro, Attr::Dri, Attr::Vis, Attr::Acc],
        desc_pt: "Abre para o corredor (ex-Mezzala/Carrilero na posse) dando largura a partir do meio.", tip_pt: "Perfeito com um extremo que entre por dentro ou um IWB.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "chm", ab: "CHM", name: "Channel Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Mc, PosGroup::Amc], tags: &[RoleTag::Run, RoleTag::Box],
        key: &[Attr::Otb, Attr::Dri, Attr::Fir, Attr::Pas, Attr::Tec, Attr::Wor, Attr::Sta, Attr::Acc], pref: &[Attr::Fin, Attr::Lon, Attr::Dec, Attr::Ant, Attr::Fla],
        desc_pt: "Ataca os canais entre central e lateral — \"game-breaker\" discreto.", tip_pt: "Combina com um falso 9 que liberte espaço.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "ap", ab: "AP", name: "Advanced Playmaker", phase: Phase::Ip,
        groups: &[PosGroup::Mc, PosGroup::Amc], tags: &[RoleTag::Cr, RoleTag::Link],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Vis, Attr::Cmp, Attr::Dec, Attr::Otb, Attr::Tea], pref: &[Attr::Dri, Attr::Ant, Attr::Fla, Attr::Agi],
        desc_pt: "Procura espaço entre linhas para criar.", tip_pt: "Precisa de corredores de corrida à frente (IF, CF, Channel Fwd).",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "am_ip", ab: "AM", name: "Attacking Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::Mc, PosGroup::Amc], tags: &[RoleTag::Box, RoleTag::Link],
        key: &[Attr::Fir, Attr::Lon, Attr::Pas, Attr::Tec, Attr::Ant, Attr::Dec, Attr::Fla, Attr::Otb], pref: &[Attr::Dri, Attr::Fin, Attr::Cmp, Attr::Vis, Attr::Agi],
        desc_pt: "Ocupa o espaço entre linhas e ameaça o golo (o antigo CM-Ataque num MC).", tip_pt: "Ótimo para chegar à área em segunda vaga.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cm_oop", ab: "CM", name: "Central Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Mc], tags: &[],
        key: &[Attr::Tck, Attr::Pos, Attr::Dec, Attr::Tea, Attr::Wor], pref: &[Attr::Ant, Attr::Cnt, Attr::Sta, Attr::Mar],
        desc_pt: "Defesa padrão do meio-campo.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "pcm", ab: "PCM", name: "Pressing Central Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Mc], tags: &[RoleTag::Hp],
        key: &[Attr::Wor, Attr::Agg, Attr::Ant, Attr::Sta, Attr::Acc, Attr::Tck], pref: &[Attr::Tea, Attr::Dec, Attr::Bra],
        desc_pt: "Sobe para pressionar (pode formar a primeira linha com o avançado).", tip_pt: "Gegenpress / bloco alto.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "scm", ab: "SCM", name: "Screening Central Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Mc], tags: &[RoleTag::Low],
        key: &[Attr::Pos, Attr::Cnt, Attr::Dec], pref: &[Attr::Mar, Attr::Tck, Attr::Ant, Attr::Tea],
        desc_pt: "Mantém posição, bloqueia linhas de passe centrais, pressiona menos.", tip_pt: "Bloco médio/baixo compacto.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wccm", ab: "WCCM", name: "Wide Covering Central Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Mc], tags: &[RoleTag::Cw],
        key: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Pos, Attr::Tck], pref: &[Attr::Acc, Attr::Ant, Attr::Dec, Attr::Pac],
        desc_pt: "Cobre o flanco do seu lado (ex-Carrilero) quando o lateral/ala subiu.", tip_pt: "Par do Advanced Wing-Back.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wm_ip", ab: "WM", name: "Wide Midfielder", phase: Phase::Ip,
        groups: &[PosGroup::MLr], tags: &[RoleTag::W],
        key: &[Attr::Pas, Attr::Cro, Attr::Tea, Attr::Wor, Attr::Dec, Attr::Sta], pref: &[Attr::Fir, Attr::Tec, Attr::Otb, Attr::Tck, Attr::Pos, Attr::Acc],
        desc_pt: "Mantém posição e largura, papel de equilíbrio.", tip_pt: "4-4-2 clássico.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "w_ip", ab: "W", name: "Winger", phase: Phase::Ip,
        groups: &[PosGroup::MLr, PosGroup::AmLr], tags: &[RoleTag::W, RoleTag::Run],
        key: &[Attr::Cro, Attr::Dri, Attr::Tec, Attr::Acc, Attr::Agi, Attr::Pac], pref: &[Attr::Fir, Attr::Pas, Attr::Otb, Attr::Fla, Attr::Bal],
        desc_pt: "Fica aberto, vai à linha e cruza.", tip_pt: "Combine com referências na área (CF, TF, Poacher).",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "iw_ip", ab: "IW", name: "Inside Winger", phase: Phase::Ip,
        groups: &[PosGroup::MLr, PosGroup::AmLr], tags: &[RoleTag::In, RoleTag::Cr],
        key: &[Attr::Dri, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Tea, Attr::Acc, Attr::Agi], pref: &[Attr::Pas, Attr::Vis, Attr::Otb, Attr::Dec, Attr::Fla],
        desc_pt: "Entra por dentro como ameaça criativa.", tip_pt: "Lateral do mesmo lado deve dar largura (WB/AWB).",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "pw", ab: "PW", name: "Playmaking Winger", phase: Phase::Ip,
        groups: &[PosGroup::MLr, PosGroup::AmLr], tags: &[RoleTag::In, RoleTag::Cr],
        key: &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Vis, Attr::Dec, Attr::Cmp, Attr::Fla], pref: &[Attr::Dri, Attr::Otb, Attr::Ant, Attr::Agi],
        desc_pt: "Parte da ala, entra por dentro, muito expressivo.", tip_pt: "Um criador por flanco no máximo.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wm_oop", ab: "WM", name: "Wide Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::MLr], tags: &[RoleTag::Trk],
        key: &[Attr::Wor, Attr::Tea, Attr::Pos, Attr::Sta], pref: &[Attr::Tck, Attr::Mar, Attr::Dec],
        desc_pt: "Recua para a linha de 4/5 do meio-campo.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "twm", ab: "TWM", name: "Tracking Wide Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::MLr], tags: &[RoleTag::Trk, RoleTag::Low],
        key: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Tck, Attr::Mar, Attr::Pos], pref: &[Attr::Acc, Attr::Ant, Attr::Agg],
        desc_pt: "Acompanha o recuo e defende o flanco intensamente.", tip_pt: "Bloco baixo.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wowm", ab: "WOWM", name: "Wide Outlet Wide Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::MLr], tags: &[RoleTag::Out],
        key: &[Attr::Acc, Attr::Pac, Attr::Otb], pref: &[Attr::Fir, Attr::Dri, Attr::Ant, Attr::Sta],
        desc_pt: "Fica alto e aberto para lançar contra-ataques.", tip_pt: "Máx. 2 outlets por tática.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "fr", ab: "FR", name: "Free Role", phase: Phase::Ip,
        groups: &[PosGroup::Amc], tags: &[RoleTag::Cr, RoleTag::Link],
        key: &[Attr::Fir, Attr::Pas, Attr::Tec, Attr::Dri, Attr::Cmp, Attr::Dec, Attr::Vis, Attr::Fla, Attr::Otb], pref: &[Attr::Ant, Attr::Agi, Attr::Bal, Attr::Lon, Attr::Fin],
        desc_pt: "Liberdade total para vaguear e criar (substitui Trequartista/Enganche).", tip_pt: "Só para o melhor jogador técnico; dá estrutura à volta.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "ss", ab: "SS", name: "Second Striker", phase: Phase::Ip,
        groups: &[PosGroup::Amc], tags: &[RoleTag::Box, RoleTag::Run],
        key: &[Attr::Fin, Attr::Fir, Attr::Otb, Attr::Cmp, Attr::Ant, Attr::Acc], pref: &[Attr::Dri, Attr::Tec, Attr::Dec, Attr::Pas, Attr::Lon, Attr::Agi],
        desc_pt: "Sobe para junto do avançado como ameaça de golo.", tip_pt: "Combine com DLF/TF/F9 que baixe.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "am_oop", ab: "AM", name: "Attacking Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Amc], tags: &[],
        key: &[Attr::Wor, Attr::Tea, Attr::Ant, Attr::Dec], pref: &[Attr::Tck, Attr::Sta, Attr::Pos],
        desc_pt: "Contribui defensivamente no meio-campo adversário.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "coam", ab: "COAM", name: "Central Outlet Attacking Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Amc], tags: &[RoleTag::Out],
        key: &[Attr::Otb, Attr::Ant, Attr::Acc, Attr::Fir], pref: &[Attr::Cmp, Attr::Dec, Attr::Pac],
        desc_pt: "Fica alto e central como alvo para contra-ataque, sem defender.", tip_pt: "Útil em bloco baixo com Counter-Attack.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "soam", ab: "SOAM", name: "Splitting Outlet Attacking Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Amc], tags: &[RoleTag::Out],
        key: &[Attr::Otb, Attr::Acc, Attr::Pac, Attr::Ant], pref: &[Attr::Dri, Attr::Fir, Attr::Sta],
        desc_pt: "Fica alto e desvia-se para os canais para iniciar transições rápidas.", tip_pt: "Contra laterais ofensivos.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "tam", ab: "TAM", name: "Tracking Attacking Midfielder", phase: Phase::Oop,
        groups: &[PosGroup::Amc], tags: &[RoleTag::Trk],
        key: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Tck, Attr::Pos], pref: &[Attr::Ant, Attr::Agg, Attr::Mar, Attr::Dec],
        desc_pt: "Recua para defender activamente, alta responsabilidade defensiva.", tip_pt: "Bloco baixo 4-5-1.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "if_ip", ab: "IF", name: "Inside Forward", phase: Phase::Ip,
        groups: &[PosGroup::AmLr], tags: &[RoleTag::In, RoleTag::Box, RoleTag::Run],
        key: &[Attr::Dri, Attr::Fin, Attr::Fir, Attr::Tec, Attr::Otb, Attr::Cmp, Attr::Acc, Attr::Agi], pref: &[Attr::Pas, Attr::Lon, Attr::Ant, Attr::Fla, Attr::Dec, Attr::Bal, Attr::Pac],
        desc_pt: "Corta para dentro em direcção à baliza — ameaça de golo.", tip_pt: "Lateral/ala do mesmo lado deve dar largura.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wf_ip", ab: "WF", name: "Wide Forward", phase: Phase::Ip,
        groups: &[PosGroup::AmLr], tags: &[RoleTag::W, RoleTag::Box, RoleTag::Run],
        key: &[Attr::Fin, Attr::Dri, Attr::Fir, Attr::Otb, Attr::Acc, Attr::Pac, Attr::Tec], pref: &[Attr::Cro, Attr::Cmp, Attr::Ant, Attr::Agi, Attr::Sta],
        desc_pt: "Fica aberto mas ataca a baliza (diagonais curtas, segundo poste).", tip_pt: "Com IWB/IFB atrás o flanco fica bem equilibrado.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "w_oop", ab: "W", name: "Winger", phase: Phase::Oop,
        groups: &[PosGroup::AmLr], tags: &[],
        key: &[Attr::Wor, Attr::Tea, Attr::Sta], pref: &[Attr::Tck, Attr::Pos, Attr::Ant],
        desc_pt: "Defesa padrão do extremo.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "tw", ab: "TW", name: "Tracking Winger", phase: Phase::Oop,
        groups: &[PosGroup::AmLr], tags: &[RoleTag::Trk],
        key: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Tck, Attr::Mar, Attr::Pos], pref: &[Attr::Acc, Attr::Ant, Attr::Agg, Attr::Dec],
        desc_pt: "Acompanha o lateral adversário até atrás.", tip_pt: "Bloco médio/baixo com 4-5-1.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "wow", ab: "WOW", name: "Wide Outlet Winger", phase: Phase::Oop,
        groups: &[PosGroup::AmLr], tags: &[RoleTag::Out],
        key: &[Attr::Acc, Attr::Pac, Attr::Otb], pref: &[Attr::Fir, Attr::Dri, Attr::Ant],
        desc_pt: "Fica alto e aberto para o contra-ataque.", tip_pt: "Máx. 2 outlets.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "iow", ab: "IOW", name: "Inside Outlet Winger", phase: Phase::Oop,
        groups: &[PosGroup::AmLr], tags: &[RoleTag::Out],
        key: &[Attr::Otb, Attr::Acc, Attr::Pac, Attr::Ant], pref: &[Attr::Fin, Attr::Fir, Attr::Dri, Attr::Cmp],
        desc_pt: "Fica alto e por dentro, pronto para atacar o espaço central em transição.", tip_pt: "Combina com Counter-Attack.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cf_ip", ab: "CF", name: "Centre Forward", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Box, RoleTag::Link],
        key: &[Attr::Fin, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Otb, Attr::Acc, Attr::Str], pref: &[Attr::Hea, Attr::Dri, Attr::Pas, Attr::Ant, Attr::Dec, Attr::Pac],
        desc_pt: "Avançado completo: ameaça de golo que também participa.", tip_pt: "Escolha segura em quase qualquer sistema.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "chf", ab: "CHF", name: "Channel Forward", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Run, RoleTag::Box],
        key: &[Attr::Fin, Attr::Dri, Attr::Otb, Attr::Acc, Attr::Pac, Attr::Wor], pref: &[Attr::Fir, Attr::Tec, Attr::Ant, Attr::Cmp, Attr::Sta],
        desc_pt: "Ataca os canais entre central e lateral.", tip_pt: "Abre espaço central para AM/SS.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "dlf", ab: "DLF", name: "Deep-Lying Forward", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Drop, RoleTag::Link],
        key: &[Attr::Fin, Attr::Fir, Attr::Tec, Attr::Cmp, Attr::Otb, Attr::Str], pref: &[Attr::Dri, Attr::Pas, Attr::Ant, Attr::Dec, Attr::Tea, Attr::Vis, Attr::Bal],
        desc_pt: "Baixa para o espaço entre linhas e liga o jogo.", tip_pt: "Precisa de alguém a atacar a profundidade (IF, CHM, SS).",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "f9", ab: "F9", name: "False Nine", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Drop, RoleTag::Link, RoleTag::Cr],
        key: &[Attr::Dri, Attr::Fir, Attr::Pas, Attr::Tec, Attr::Cmp, Attr::Dec, Attr::Otb, Attr::Tea, Attr::Vis, Attr::Acc], pref: &[Attr::Fin, Attr::Ant, Attr::Fla, Attr::Agi, Attr::Bal],
        desc_pt: "Recua até à zona de médio ofensivo, arrasta centrais e cria.", tip_pt: "Sem corredores (IF/CHM/SS) a área fica vazia.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "poa", ab: "P", name: "Poacher", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Box, RoleTag::Run],
        key: &[Attr::Fin, Attr::Hea, Attr::Ant, Attr::Cmp, Attr::Cnt, Attr::Otb, Attr::Acc], pref: &[Attr::Fir, Attr::Tec, Attr::Dec, Attr::Bal],
        desc_pt: "Joga na última linha, vive para finalizar.", tip_pt: "Precisa de servidores — criadores/cruzadores.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "tf", ab: "TF", name: "Target Forward", phase: Phase::Ip,
        groups: &[PosGroup::St], tags: &[RoleTag::Air, RoleTag::Link, RoleTag::Box],
        key: &[Attr::Hea, Attr::Str, Attr::Jum, Attr::Bra, Attr::Fin, Attr::Cmp, Attr::Bal], pref: &[Attr::Fir, Attr::Otb, Attr::Tea, Attr::Agg, Attr::Ant, Attr::Dec],
        desc_pt: "Referência na última linha: ganha duelos aéreos e segura a bola.", tip_pt: "Combine com cruzamentos e \"Bypass Press\".",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cf_oop", ab: "CF", name: "Centre Forward", phase: Phase::Oop,
        groups: &[PosGroup::St], tags: &[],
        key: &[Attr::Wor, Attr::Tea, Attr::Ant], pref: &[Attr::Sta, Attr::Agg, Attr::Acc, Attr::Dec],
        desc_pt: "Comportamento defensivo padrão do avançado.", tip_pt: "Base.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "cocf", ab: "COCF", name: "Central Outlet Centre Forward", phase: Phase::Oop,
        groups: &[PosGroup::St], tags: &[RoleTag::Out],
        key: &[Attr::Otb, Attr::Ant, Attr::Cmp, Attr::Str], pref: &[Attr::Acc, Attr::Fir],
        desc_pt: "Fica alto e central como válvula de escape.", tip_pt: "Bloco baixo com Counter-Attack.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "socf", ab: "SOCF", name: "Splitting Outlet Centre Forward", phase: Phase::Oop,
        groups: &[PosGroup::St], tags: &[RoleTag::Out],
        key: &[Attr::Otb, Attr::Acc, Attr::Pac, Attr::Ant], pref: &[Attr::Dri, Attr::Fir, Attr::Sta],
        desc_pt: "Fica alto e desvia-se para os canais à espera da bola longa.", tip_pt: "Contra defesas com laterais muito subidos.",
        verified: true, wide_cb: false, slot: None,
    },
    Role {
        id: "tcf", ab: "TCF", name: "Tracking Centre Forward", phase: Phase::Oop,
        groups: &[PosGroup::St], tags: &[RoleTag::Trk, RoleTag::Hp],
        key: &[Attr::Wor, Attr::Sta, Attr::Tea, Attr::Agg, Attr::Ant], pref: &[Attr::Tck, Attr::Acc, Attr::Bra, Attr::Dec],
        desc_pt: "Recua para apoiar a defesa / é o primeiro a pressionar.", tip_pt: "Gegenpress ou bloco compacto.",
        verified: true, wide_cb: false, slot: None,
    },
];

pub fn role_by_id(id: &str) -> Option<&'static Role> {
    ROLES.iter().find(|r| r.id == id)
}

/// Central (`C`) vs off-centre (`S`) slot within a horizontal line (DM/MC/AMC/ST/DC).
pub fn slot_pos(code: &str) -> Option<SlotConstraint> {
    match code {
        "DM" | "MC" | "AMC" | "ST" | "DC" => Some(SlotConstraint::Central),
        "DML" | "DMR" | "MCL" | "MCR" | "AMCL" | "AMCR" | "STL" | "STR" | "DCL" | "DCR" => {
            Some(SlotConstraint::Side)
        }
        _ => None,
    }
}

/// Whether `role` is valid for the given position group, phase, exact slot code, and wide-CB context.
pub fn role_fits(
    role: &Role,
    group: PosGroup,
    phase: Phase,
    code: Option<&str>,
    wide_cb: bool,
) -> bool {
    if role.phase != phase || !role.groups.contains(&group) {
        return false;
    }
    if role.wide_cb && !wide_cb {
        return false;
    }
    if let (Some(req_slot), Some(c)) = (role.slot, code) {
        if let Some(sp) = slot_pos(c) {
            if sp != req_slot {
                return false;
            }
        }
    }
    true
}

/// All roles valid for a slot context.
pub fn roles_for(
    group: PosGroup,
    phase: Phase,
    code: Option<&str>,
    wide_cb: bool,
) -> Vec<&'static Role> {
    ROLES
        .iter()
        .filter(|r| role_fits(r, group, phase, code, wide_cb))
        .collect()
}

/// Check if a role id exists and is valid for the slot context.
pub fn valid_role(
    id: &str,
    group: PosGroup,
    phase: Phase,
    code: Option<&str>,
    wide_cb: bool,
) -> bool {
    role_by_id(id).is_some_and(|r| role_fits(r, group, phase, code, wide_cb))
}

/// Default fallback role ID for a position group and phase.
pub const fn default_role(group: PosGroup, phase: Phase) -> &'static str {
    match (phase, group) {
        (Phase::Ip, PosGroup::Gk) => "gk_ip",
        (Phase::Ip, PosGroup::Dc) => "cb_ip",
        (Phase::Ip, PosGroup::DLr) => "fb_ip",
        (Phase::Ip, PosGroup::WbLr) => "wb_ip",
        (Phase::Ip, PosGroup::Dm) => "dm_ip",
        (Phase::Ip, PosGroup::Mc) => "cm_ip",
        (Phase::Ip, PosGroup::MLr) => "wm_ip",
        (Phase::Ip, PosGroup::Amc) => "am_ip",
        (Phase::Ip, PosGroup::AmLr) => "w_ip",
        (Phase::Ip, PosGroup::St) => "cf_ip",
        (Phase::Oop, PosGroup::Gk) => "gk_oop",
        (Phase::Oop, PosGroup::Dc) => "cb_oop",
        (Phase::Oop, PosGroup::DLr) => "fb_oop",
        (Phase::Oop, PosGroup::WbLr) => "wb_oop",
        (Phase::Oop, PosGroup::Dm) => "dm_oop",
        (Phase::Oop, PosGroup::Mc) => "cm_oop",
        (Phase::Oop, PosGroup::MLr) => "wm_oop",
        (Phase::Oop, PosGroup::Amc) => "am_oop",
        (Phase::Oop, PosGroup::AmLr) => "w_oop",
        (Phase::Oop, PosGroup::St) => "cf_oop",
    }
}
