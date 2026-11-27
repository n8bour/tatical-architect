//! 4-phase tactical synergy analysis: build-up, attack, defence, and transitions.

use crate::data::roles::role_by_id;
use crate::data::tactics_data::{slot_info, ti_val};
use crate::engine::parsers::surname;
use crate::types::{
    Attr, AttrLookup, Lang, Player, PosGroup, Role, RoleTag, Side, SlotMeta, SynergyIssue,
    SynergyLink, SynergyLinkType, SynergyPhase, SynergyPhaseScore, SynergyReport, Tactic,
};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Attacking phase synergy breakdown returned by `synergy_attack`.
#[derive(Debug, Clone, PartialEq)]
pub struct SynergyAttack {
    pub score: u8,
    pub parts: BTreeMap<String, u8>,
    pub links: Vec<SynergyLink>,
    pub issues: Vec<SynergyIssue>,
    pub creators: usize,
    pub box_n: usize,
}

struct SynSlot<'a> {
    idx: usize,
    role: &'static Role,
    si: SlotMeta,
    p: Option<&'a Player>,
    code: &'a str,
}

impl SynSlot<'_> {
    #[inline]
    fn tag(&self, t: RoleTag) -> bool {
        self.role.has_tag(t)
    }

    fn label(&self) -> String {
        match self.p {
            Some(pl) => format!("{} {}", self.role.ab, surname(&pl.name)),
            None => format!("{} {}", self.role.ab, self.code),
        }
    }
}

#[inline]
fn attr_or_10(p: Option<&Player>, k: Attr) -> f32 {
    p.and_then(|pl| pl.attr(k)).unwrap_or(10.0)
}

fn q(p: Option<&Player>, keys: &[Attr]) -> f32 {
    let sum: f32 = keys.iter().map(|&k| attr_or_10(p, k)).sum();
    sum / keys.len() as f32 / 20.0
}

#[inline]
fn pct(v: f32) -> u8 {
    (v * 100.0).round().clamp(0.0, 100.0) as u8
}

#[inline]
fn lane(x: &SynSlot<'_>) -> Side {
    if x.si.x < 38.0 {
        Side::L
    } else if x.si.x > 62.0 {
        Side::R
    } else {
        Side::C
    }
}

#[inline]
fn same_lane(c: &SynSlot<'_>, r: &SynSlot<'_>) -> bool {
    lane(c) == Side::C || lane(r) == Side::C || lane(c) == lane(r)
}

fn mk_link(
    link_type: SynergyLinkType,
    phase: SynergyPhase,
    from: &SynSlot<'_>,
    to: &SynSlot<'_>,
    value: f32,
    why: String,
) -> SynergyLink {
    SynergyLink {
        link_type,
        phase,
        from_code: from.code.to_string(),
        to_code: to.code.to_string(),
        from_label: from.label(),
        to_label: to.label(),
        value,
        why,
    }
}

/// Evaluate attacking-phase synergy (chains, crosses, front line, box presence, flanks, creative load).
pub fn synergy_attack(t: &Tactic, players: &[Player], lang: Lang) -> SynergyAttack {
    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    let q_create = |p: Option<&Player>| q(p, &[Attr::Pas, Attr::Vis, Attr::Tec, Attr::Dec, Attr::Fir]);
    let q_run = |p: Option<&Player>| q(p, &[Attr::Otb, Attr::Acc, Attr::Pac, Attr::Ant]);
    let q_finish = |p: Option<&Player>| q(p, &[Attr::Fin, Attr::Cmp, Attr::Otb, Attr::Ant]);
    let q_air = |p: Option<&Player>| q(p, &[Attr::Hea, Attr::Jum, Attr::Str, Attr::Bra]);
    let q_cross = |p: Option<&Player>| q(p, &[Attr::Cro, Attr::Tec, Attr::Dec]);
    let q_dribble = |p: Option<&Player>| q(p, &[Attr::Dri, Attr::Agi, Attr::Acc, Attr::Fla]);

    let x_slots: Vec<SynSlot<'_>> = t
        .slots
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| {
            let role = role_by_id(&s.ip)?;
            let si = slot_info(&t.ip_form, &s.ip_code)?;
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied());
            Some(SynSlot {
                idx,
                role,
                si,
                p,
                code: s.ip_code.as_str(),
            })
        })
        .collect();

    let out: Vec<&SynSlot<'_>> = x_slots
        .iter()
        .filter(|x| x.si.group != PosGroup::Gk)
        .collect();

    let mut links: Vec<SynergyLink> = Vec::new();
    let mut issues: Vec<SynergyIssue> = Vec::new();

    // 1. Creator -> finisher/runner
    let creators: Vec<&SynSlot<'_>> = out
        .iter()
        .copied()
        .filter(|x| x.tag(RoleTag::Cr) && !matches!(x.si.group, PosGroup::Dc | PosGroup::Gk))
        .collect();
    let receivers: Vec<&SynSlot<'_>> = out
        .iter()
        .copied()
        .filter(|x| x.tag(RoleTag::Run) || x.tag(RoleTag::Box))
        .collect();

    let mut chains: Vec<f32> = Vec::new();
    let mut chain_targets: Vec<usize> = Vec::new();
    let mut first_target_label: Option<String> = None;

    for &c in &creators {
        let best = receivers
            .iter()
            .copied()
            .filter(|r| r.idx != c.idx && r.si.y >= c.si.y - 4.0 && same_lane(c, r))
            .map(|r| {
                let v = (q_create(c.p) * q_run(r.p).max(q_finish(r.p))).sqrt();
                (r, v)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));

        if let Some((r, v)) = best {
            chains.push(v);
            chain_targets.push(r.idx);
            if first_target_label.is_none() {
                first_target_label = Some(r.label());
            }
            let cl = c.label();
            let rl = r.label();
            links.push(mk_link(
                SynergyLinkType::Chain,
                SynergyPhase::Att,
                c,
                r,
                v,
                match lang {
                    Lang::Pt => format!("{cl} serve {rl}"),
                    Lang::En => format!("{cl} feeds {rl}"),
                },
            ));
        } else {
            let cl = c.label();
            issues.push(SynergyIssue {
                key: "chain".to_string(),
                phase: SynergyPhase::Att,
                text: match lang {
                    Lang::Pt => format!(
                        "{cl} não tem ninguém à frente a atacar o espaço no seu corredor."
                    ),
                    Lang::En => format!("{cl} has nobody ahead attacking the space in his channel."),
                },
                fix: lang
                    .pick(
                        "Ponha um Avançado Interior, Channel Midfielder ou Segundo Avançado à frente dele.",
                        "Put an Inside Forward, Channel Midfielder or Second Striker ahead of him.",
                    )
                    .to_string(),
            });
        }
    }

    chains.sort_by(|a, b| b.total_cmp(a));
    let mut chain = if chains.is_empty() {
        0.0
    } else {
        let take = chains.len().min(3);
        chains[..take].iter().sum::<f32>() / take as f32
    };
    if chains.len() == 1 {
        chain *= 0.8;
    }

    let distinct_targets: HashSet<usize> = chain_targets.iter().copied().collect();
    if chain_targets.len() >= 2 && distinct_targets.len() == 1 {
        chain *= 0.8;
        let tgt = first_target_label.unwrap_or_default();
        issues.push(SynergyIssue {
            key: "chain".to_string(),
            phase: SynergyPhase::Att,
            text: match lang {
                Lang::Pt => format!(
                    "Todos os criativos servem o mesmo jogador ({tgt}): previsível e fácil de anular."
                ),
                Lang::En => format!(
                    "Every creator feeds the same player ({tgt}): predictable and easy to shut down."
                ),
            },
            fix: lang
                .pick(
                    "Dê um segundo alvo em movimento: Avançado Interior no outro flanco, Channel Midfielder ou Segundo Avançado.",
                    "Give them a second moving target: an Inside Forward on the other flank, a Channel Midfielder or a Second Striker.",
                )
                .to_string(),
        });
    }
    if creators.is_empty() {
        issues.push(SynergyIssue {
            key: "chain".to_string(),
            phase: SynergyPhase::Att,
            text: lang
                .pick(
                    "Ninguém cria no meio-campo ou no ataque.",
                    "Nobody creates in midfield or attack.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Um Construtor de Jogo (DLP, AP, MP) ou um Extremo Construtor dá a primeira peça da ligação.",
                    "A playmaker (DLP, AP, MP) or a Playmaking Winger provides the first link.",
                )
                .to_string(),
        });
    }
    if receivers.is_empty() {
        issues.push(SynergyIssue {
            key: "chain".to_string(),
            phase: SynergyPhase::Att,
            text: lang
                .pick(
                    "Ninguém ataca o espaço nem a área.",
                    "Nobody attacks the space or the box.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Avançado Interior, Avançado de Corredor, Ponta-de-Lança Fixo ou Médio Área-a-Área.",
                    "Inside Forward, Channel Forward, Poacher or Box-to-Box Midfielder.",
                )
                .to_string(),
        });
    }

    // 2. Crosses -> targets
    let wides: Vec<&SynSlot<'_>> = out
        .iter()
        .copied()
        .filter(|x| x.tag(RoleTag::W) && x.si.side != Side::C && x.si.group != PosGroup::Dc)
        .collect();
    let targets: Vec<&SynSlot<'_>> = out
        .iter()
        .copied()
        .filter(|x| {
            x.tag(RoleTag::Box)
                && matches!(x.si.group, PosGroup::St | PosGroup::Amc | PosGroup::Mc)
        })
        .collect();
    let best_target = targets
        .iter()
        .copied()
        .map(|x| (x, q_air(x.p).max(q_finish(x.p) * 0.9)))
        .max_by(|a, b| a.1.total_cmp(&b.1));

    let mut cross = 0.0_f32;
    if let (false, Some((bt, bt_v))) = (wides.is_empty(), best_target) {
        let (bw, bw_v) = wides
            .iter()
            .copied()
            .map(|x| (x, q_cross(x.p)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        cross = (bw_v * bt_v).sqrt() * (0.55 + 0.25 * targets.len() as f32).min(1.0);
        let bwl = bw.label();
        let btl = bt.label();
        links.push(mk_link(
            SynergyLinkType::Cross,
            SynergyPhase::Att,
            bw,
            bt,
            cross,
            match lang {
                Lang::Pt => format!("{bwl} cruza para {btl}"),
                Lang::En => format!("{bwl} crosses for {btl}"),
            },
        ));
        if bw_v < 0.55 {
            issues.push(SynergyIssue {
                key: "cross".to_string(),
                phase: SynergyPhase::Att,
                text: match lang {
                    Lang::Pt => format!("Quem dá largura cruza mal ({bwl})."),
                    Lang::En => format!("The width providers cross poorly ({bwl})."),
                },
                fix: lang
                    .pick(
                        "Low Crosses, ou um ala/extremo com melhores Cruzamentos.",
                        "Low Crosses, or a wide player with better Crossing.",
                    )
                    .to_string(),
            });
        }
    } else if !wides.is_empty() && best_target.is_none() {
        issues.push(SynergyIssue {
            key: "cross".to_string(),
            phase: SynergyPhase::Att,
            text: lang
                .pick(
                    "Há largura mas ninguém ataca a área para os cruzamentos.",
                    "There is width but nobody attacks the box for crosses.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Ponta-de-Lança, Avançado de Referência ou um médio que chegue à área.",
                    "A Centre Forward, Target Forward or a midfielder who arrives in the box.",
                )
                .to_string(),
        });
    }

    // 3. Front line partnership
    let sts: Vec<&SynSlot<'_>> = out
        .iter()
        .copied()
        .filter(|x| x.si.group == PosGroup::St)
        .collect();
    let front: f32 = if sts.len() >= 2 {
        let drop_st: Vec<&SynSlot<'_>> = sts
            .iter()
            .copied()
            .filter(|x| x.tag(RoleTag::Drop) || x.tag(RoleTag::Air))
            .collect();
        let run_st: Vec<&SynSlot<'_>> = sts
            .iter()
            .copied()
            .filter(|x| x.tag(RoleTag::Run) && !x.tag(RoleTag::Drop))
            .collect();
        let complete_st: Vec<&SynSlot<'_>> = sts
            .iter()
            .copied()
            .filter(|x| {
                x.tag(RoleTag::Link)
                    && x.tag(RoleTag::Box)
                    && !x.tag(RoleTag::Drop)
                    && !x.tag(RoleTag::Air)
            })
            .collect();

        if drop_st.is_empty()
            && !complete_st.is_empty()
            && !run_st.is_empty()
            && complete_st[0].idx != run_st[0].idx
        {
            let v = 0.82;
            let cl = complete_st[0].label();
            let rl = run_st[0].label();
            links.push(mk_link(
                SynergyLinkType::Front,
                SynergyPhase::Att,
                complete_st[0],
                run_st[0],
                v,
                match lang {
                    Lang::Pt => format!("{cl} segura e {rl} ataca a profundidade"),
                    Lang::En => format!("{cl} holds it up and {rl} runs in behind"),
                },
            ));
            v
        } else if !drop_st.is_empty() && !run_st.is_empty() && drop_st[0].idx != run_st[0].idx {
            let v = 0.9 + 0.1 * q_run(run_st[0].p).min(1.0);
            let dl = drop_st[0].label();
            let rl = run_st[0].label();
            links.push(mk_link(
                SynergyLinkType::Front,
                SynergyPhase::Att,
                drop_st[0],
                run_st[0],
                v,
                match lang {
                    Lang::Pt => format!("{dl} baixa e abre espaço para {rl}"),
                    Lang::En => format!("{dl} drops and opens space for {rl}"),
                },
            ));
            v
        } else {
            issues.push(SynergyIssue {
                key: "front".to_string(),
                phase: SynergyPhase::Att,
                text: lang
                    .pick(
                        "Os dois avançados têm o mesmo perfil e ocupam o mesmo espaço.",
                        "Both strikers have the same profile and occupy the same space.",
                    )
                    .to_string(),
                fix: lang
                    .pick(
                        "Um que baixe (Avançado Recuado, Avançado de Referência) e outro que ataque o espaço (Avançado de Corredor, Ponta-de-Lança Fixo).",
                        "One who drops (Deep-Lying Forward, Target Forward) and one who runs in behind (Channel Forward, Poacher).",
                    )
                    .to_string(),
            });
            0.45
        }
    } else if sts.len() == 1 {
        let st = sts[0];
        if st.tag(RoleTag::Drop) {
            let beyond: Vec<&SynSlot<'_>> = out
                .iter()
                .copied()
                .filter(|x| {
                    x.idx != st.idx
                        && (x.tag(RoleTag::Run)
                            || (x.tag(RoleTag::Box) && x.si.group != PosGroup::St))
                        && x.si.y >= 55.0
                })
                .collect();
            let v = (0.4 + 0.25 * beyond.len() as f32).min(1.0);
            if beyond.len() < 2 {
                let stl = st.label();
                issues.push(SynergyIssue {
                    key: "front".to_string(),
                    phase: SynergyPhase::Att,
                    text: match lang {
                        Lang::Pt => {
                            format!("{stl} baixa, mas quase ninguém ataca o espaço que ele abre.")
                        }
                        Lang::En => {
                            format!("{stl} drops, but hardly anyone attacks the space he opens.")
                        }
                    },
                    fix: lang
                        .pick(
                            "Dois corredores por trás dele: Avançado Interior, Channel Midfielder ou Segundo Avançado.",
                            "Two runners behind him: Inside Forward, Channel Midfielder or Second Striker.",
                        )
                        .to_string(),
                });
            } else {
                let stl = st.label();
                let bl = beyond[0].label();
                links.push(mk_link(
                    SynergyLinkType::Front,
                    SynergyPhase::Att,
                    st,
                    beyond[0],
                    v,
                    match lang {
                        Lang::Pt => format!("{stl} baixa e {bl} ataca o espaço"),
                        Lang::En => format!("{stl} drops and {bl} attacks the space"),
                    },
                ));
            }
            v
        } else {
            0.55 + 0.35 * q_finish(st.p).max(q_run(st.p))
        }
    } else {
        0.35
    };

    // 4. Box presence (3–4 ideal)
    let box_n = out
        .iter()
        .filter(|x| x.tag(RoleTag::Box) || (x.tag(RoleTag::In) && x.tag(RoleTag::Run)))
        .count();
    let box_v: f32 = match box_n {
        0 => 0.0,
        1 => 0.15,
        2 => 0.45,
        3 => 0.85,
        4 => 1.0,
        5 => 0.85,
        _ => 0.6,
    };
    if box_n <= 2 {
        issues.push(SynergyIssue {
            key: "box".to_string(),
            phase: SynergyPhase::Att,
            text: match lang {
                Lang::Pt => format!("Só {box_n} jogador(es) chegam à área: poucas finalizações."),
                Lang::En => format!("Only {box_n} player(s) arrive in the box: few shots."),
            },
            fix: lang
                .pick(
                    "Troque um papel de apoio por Avançado Interior, Channel Midfielder ou Médio Área-a-Área.",
                    "Swap a support role for an Inside Forward, Channel Midfielder or Box-to-Box Midfielder.",
                )
                .to_string(),
        });
    }

    // 5. Flank combinations
    let mut flank_sum = 0.0_f32;
    let mut fl_n = 0_usize;
    for side in [Side::L, Side::R] {
        let att: Vec<&SynSlot<'_>> = out
            .iter()
            .copied()
            .filter(|x| {
                x.si.side == side && matches!(x.si.group, PosGroup::AmLr | PosGroup::MLr)
            })
            .collect();
        let back: Vec<&SynSlot<'_>> = out
            .iter()
            .copied()
            .filter(|x| {
                x.si.side == side && matches!(x.si.group, PosGroup::DLr | PosGroup::WbLr)
            })
            .collect();
        if att.is_empty() && back.is_empty() {
            continue;
        }
        fl_n += 1;
        let a_in = att.iter().any(|x| x.tag(RoleTag::In));
        let a_w = att.iter().any(|x| x.tag(RoleTag::W));
        let b_w = back
            .iter()
            .any(|x| x.tag(RoleTag::W) || x.tag(RoleTag::Adv));
        let b_in = back
            .iter()
            .any(|x| x.tag(RoleTag::In) || x.tag(RoleTag::Rd));
        let nm_pt = if side == Side::L { "esquerdo" } else { "direito" };
        let nm_en = if side == Side::L { "Left" } else { "Right" };

        if a_in && b_w {
            flank_sum += 1.0;
            if let (Some(&f), Some(&g)) = (
                back.iter()
                    .find(|x| x.tag(RoleTag::W) || x.tag(RoleTag::Adv)),
                att.iter().find(|x| x.tag(RoleTag::In)),
            ) {
                let fl = f.label();
                let gl = g.label();
                links.push(mk_link(
                    SynergyLinkType::Flank,
                    SynergyPhase::Att,
                    f,
                    g,
                    1.0,
                    match lang {
                        Lang::Pt => format!("Flanco {nm_pt}: {fl} por fora, {gl} por dentro"),
                        Lang::En => format!("{nm_en} flank: {fl} outside, {gl} inside"),
                    },
                ));
            }
        } else if a_w && b_in {
            flank_sum += 0.9;
        } else if a_w && b_w {
            flank_sum += 0.45;
            issues.push(SynergyIssue {
                key: "flank".to_string(),
                phase: SynergyPhase::Att,
                text: match lang {
                    Lang::Pt => format!("Flanco {nm_pt}: dois jogadores abertos no mesmo corredor."),
                    Lang::En => format!("{nm_en} flank: two wide players in the same channel."),
                },
                fix: lang
                    .pick(
                        "Um deles por dentro (Extremo de Apoio Interior / Ala Invertido) ou Supporting Runs em Underlap.",
                        "One of them inside (Inside Winger / Inside Wing-Back) or Supporting Runs set to Underlap.",
                    )
                    .to_string(),
            });
        } else if a_in && b_in {
            flank_sum += 0.35;
            issues.push(SynergyIssue {
                key: "flank".to_string(),
                phase: SynergyPhase::Att,
                text: match lang {
                    Lang::Pt => format!("Flanco {nm_pt}: ninguém dá largura."),
                    Lang::En => format!("{nm_en} flank: nobody provides width."),
                },
                fix: lang
                    .pick(
                        "Lateral/ala por fora (Wing-Back, Ala Projetado) ou um Extremo aberto.",
                        "A full-back/wing-back outside (Wing-Back, Advanced Wing-Back) or a wide Winger.",
                    )
                    .to_string(),
            });
        } else {
            flank_sum += 0.7;
        }
    }
    let flank = if fl_n > 0 {
        flank_sum / fl_n as f32
    } else {
        0.6
    };

    // 6. Creative load
    let n_cr = creators.len();
    let load: f32 = match n_cr {
        0 => 0.0,
        1 => 0.65,
        2 => 1.0,
        3 => 0.9,
        _ => 0.5,
    };
    if n_cr == 1 {
        let cl = creators[0].label();
        issues.push(SynergyIssue {
            key: "load".to_string(),
            phase: SynergyPhase::Att,
            text: match lang {
                Lang::Pt => format!("Só {cl} cria: se for anulado, a equipa deixa de criar."),
                Lang::En => format!("Only {cl} creates: if he is shut down, the team stops creating."),
            },
            fix: lang
                .pick(
                    "Um segundo criativo noutra zona (Construtor de Jogo Recuado atrás, ou Extremo Construtor numa ala).",
                    "A second creator in another zone (a Deep-Lying Playmaker behind, or a Playmaking Winger out wide).",
                )
                .to_string(),
        });
    }
    if n_cr > 3 {
        issues.push(SynergyIssue {
            key: "load".to_string(),
            phase: SynergyPhase::Att,
            text: match lang {
                Lang::Pt => {
                    format!("{n_cr} criativos pedem todos a bola; falta quem a receba em movimento.")
                }
                Lang::En => {
                    format!("{n_cr} creators all want the ball; nobody receives it on the move.")
                }
            },
            fix: lang
                .pick(
                    "Troque um criativo por um papel de movimento (corredor/finalizador).",
                    "Swap one creator for a movement role (runner/finisher).",
                )
                .to_string(),
        });
    }

    // Individual dribble bonus (5%)
    let drib = out
        .iter()
        .filter(|x| {
            matches!(
                x.si.group,
                PosGroup::AmLr | PosGroup::Amc | PosGroup::St | PosGroup::MLr
            )
        })
        .map(|x| q_dribble(x.p))
        .fold(0.0_f32, f32::max);

    let mut parts = BTreeMap::new();
    let p_chain = pct(chain);
    let p_cross = pct(cross);
    let p_front = pct(front);
    let p_box = pct(box_v);
    let p_flank = pct(flank);
    let p_load = pct(load);
    parts.insert("chain".to_string(), p_chain);
    parts.insert("cross".to_string(), p_cross);
    parts.insert("front".to_string(), p_front);
    parts.insert("box".to_string(), p_box);
    parts.insert("flank".to_string(), p_flank);
    parts.insert("load".to_string(), p_load);

    let weighted = p_chain as f32 * 0.30
        + p_cross as f32 * 0.15
        + p_front as f32 * 0.12
        + p_box as f32 * 0.15
        + p_flank as f32 * 0.16
        + p_load as f32 * 0.12;
    let score = (weighted * 0.95 + drib * 100.0 * 0.05)
        .min(100.0)
        .round()
        .clamp(0.0, 100.0) as u8;

    links.sort_by(|a, b| b.value.total_cmp(&a.value));
    links.truncate(8);

    SynergyAttack {
        score,
        parts,
        links,
        issues,
        creators: n_cr,
        box_n,
    }
}

/// Full 4-phase synergy report: build-up (15%), attack (40%), defence (30%), transitions (15%).
pub fn synergy_report(t: &Tactic, players: &[Player], lang: Lang) -> SynergyReport {
    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();

    let ipx: Vec<SynSlot<'_>> = t
        .slots
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| {
            let role = role_by_id(&s.ip)?;
            let si = slot_info(&t.ip_form, &s.ip_code)?;
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied());
            Some(SynSlot {
                idx,
                role,
                si,
                p,
                code: s.ip_code.as_str(),
            })
        })
        .collect();

    let oox: Vec<SynSlot<'_>> = t
        .slots
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| {
            let role = role_by_id(&s.oop)?;
            let si = slot_info(&t.oop_form, &s.oop_code)?;
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied());
            Some(SynSlot {
                idx,
                role,
                si,
                p,
                code: s.oop_code.as_str(),
            })
        })
        .collect();

    // ---------------- ATTACK
    let att = synergy_attack(t, players, lang);

    // ---------------- BUILD-UP
    let mut b_iss: Vec<SynergyIssue> = Vec::new();
    let mut b_links: Vec<SynergyLink> = Vec::new();

    let back: Vec<&SynSlot<'_>> = ipx
        .iter()
        .filter(|x| matches!(x.si.group, PosGroup::Gk | PosGroup::Dc) || x.tag(RoleTag::ToCb))
        .collect();
    let recv: Vec<&SynSlot<'_>> = ipx
        .iter()
        .filter(|x| {
            matches!(x.si.group, PosGroup::Dm | PosGroup::Mc)
                || x.tag(RoleTag::ToDm)
                || (x.tag(RoleTag::In) && matches!(x.si.group, PosGroup::DLr | PosGroup::WbLr))
        })
        .collect();

    let pass_q = |p: Option<&Player>| q(p, &[Attr::Pas, Attr::Fir, Attr::Cmp, Attr::Dec]);
    let best_back = back
        .iter()
        .copied()
        .map(|x| (x, pass_q(x.p) * if x.tag(RoleTag::Cr) { 1.05 } else { 1.0 }))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let best_recv = recv
        .iter()
        .copied()
        .map(|x| (x, q(x.p, &[Attr::Fir, Attr::Pas, Attr::Cmp, Attr::Ant, Attr::Dec])))
        .max_by(|a, b| a.1.total_cmp(&b.1));

    let b_short = match (best_back, best_recv) {
        (Some((bb, bb_v)), Some((br, br_v))) => {
            let v = (bb_v * br_v).sqrt();
            let bbl = bb.label();
            let brl = br.label();
            b_links.push(mk_link(
                SynergyLinkType::Build,
                SynergyPhase::Build,
                bb,
                br,
                v,
                match lang {
                    Lang::Pt => format!("{bbl} sai a jogar para {brl}"),
                    Lang::En => format!("{bbl} plays out to {brl}"),
                },
            ));
            v
        }
        _ => 0.3,
    };

    let support_n = ipx
        .iter()
        .filter(|x| {
            x.tag(RoleTag::ToDm)
                || x.tag(RoleTag::ToCb)
                || (x.tag(RoleTag::Drop) && x.si.group == PosGroup::Dm)
                || x.si.group == PosGroup::Dm
        })
        .count();
    let b_support: f32 = match support_n {
        0 => 0.3,
        1 => 0.75,
        2 | 3 => 1.0,
        _ => 0.85,
    };
    if support_n == 0 {
        b_iss.push(SynergyIssue {
            key: "build".to_string(),
            phase: SynergyPhase::Build,
            text: lang
                .pick(
                    "Ninguém se oferece entre linhas na saída de bola: os centrais ficam sem linha de passe.",
                    "Nobody offers himself between the lines in the build-up: the centre-backs have no passing option.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Um Médio Defensivo, Construtor de Jogo Recuado ou um lateral que feche por dentro (Lateral Invertido / Ala Invertido).",
                    "A Defensive Midfielder, Deep-Lying Playmaker or a full-back who tucks inside (Inside Full-Back / Inside Wing-Back).",
                )
                .to_string(),
        });
    }

    let first_line: Vec<&SynSlot<'_>> = ipx
        .iter()
        .filter(|x| matches!(x.si.group, PosGroup::Gk | PosGroup::Dc | PosGroup::Dm))
        .collect();
    let resist = if first_line.is_empty() {
        0.5
    } else {
        first_line
            .iter()
            .map(|x| q(x.p, &[Attr::Fir, Attr::Cmp, Attr::Pas, Attr::Dec]))
            .sum::<f32>()
            / first_line.len() as f32
    };

    let through = ti_val(t, "buildup") == 0;
    let bypass = ti_val(t, "buildup") == 2 || ti_val(t, "goal_kicks") == 2;
    let mut b_long = 0.7_f32;
    if bypass {
        let tgt = ipx
            .iter()
            .filter(|x| x.si.group == PosGroup::St)
            .map(|x| {
                (
                    x,
                    q(x.p, &[Attr::Hea, Attr::Jum, Attr::Str, Attr::Bra])
                        * if x.tag(RoleTag::Air) { 1.08 } else { 1.0 },
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));
        b_long = tgt.map(|(_, v)| v.min(1.0)).unwrap_or(0.2);
        if tgt.is_none_or(|(_, v)| v < 0.6) {
            b_iss.push(SynergyIssue {
                key: "build".to_string(),
                phase: SynergyPhase::Build,
                text: lang
                    .pick(
                        "Saltar a pressão com bola longa, mas sem uma referência forte no ar.",
                        "Bypassing the press with long balls, but without a strong aerial target.",
                    )
                    .to_string(),
                fix: lang
                    .pick(
                        "Avançado de Referência na frente, ou sair a jogar curto.",
                        "A Target Forward up front, or play out short.",
                    )
                    .to_string(),
            });
        }
    }
    if through && resist < 0.62 {
        let r_disp = (resist * 20.0 * 10.0).round() / 10.0;
        b_iss.push(SynergyIssue {
            key: "build".to_string(),
            phase: SynergyPhase::Build,
            text: match lang {
                Lang::Pt => format!(
                    "Sair a jogar contra a pressão com uma primeira linha pouco resistente ({r_disp})."
                ),
                Lang::En => format!(
                    "Playing through the press with a first line that is not press-resistant ({r_disp})."
                ),
            },
            fix: lang
                .pick(
                    "Balanced ou Contornar a Pressão, ou centrais/GR melhores com bola.",
                    "Balanced or Bypass Press, or centre-backs/keeper better on the ball.",
                )
                .to_string(),
        });
    }

    let mut b_parts = BTreeMap::new();
    let pb_short = pct(b_short);
    let pb_support = pct(b_support);
    let pb_resist = pct(if through { resist * 1.05 } else { resist.max(0.6) });
    let pb_long = pct(b_long);
    b_parts.insert("short".to_string(), pb_short);
    b_parts.insert("support".to_string(), pb_support);
    b_parts.insert("resist".to_string(), pb_resist);
    b_parts.insert("long".to_string(), pb_long);
    let build_score = (pb_short as f32 * 0.35
        + pb_support as f32 * 0.25
        + pb_resist as f32 * 0.25
        + pb_long as f32 * 0.15)
        .round()
        .clamp(0.0, 100.0) as u8;
    let build = SynergyPhaseScore {
        score: build_score,
        parts: b_parts,
    };

    // ---------------- DEFENCE (OOP shape)
    let mut d_iss: Vec<SynergyIssue> = Vec::new();
    let mut d_links: Vec<SynergyLink> = Vec::new();
    let outf: Vec<&SynSlot<'_>> = oox
        .iter()
        .filter(|x| x.si.group != PosGroup::Gk)
        .collect();
    let press_q = |p: Option<&Player>| q(p, &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg, Attr::Ant]);
    let cover_q = |p: Option<&Player>| q(p, &[Attr::Pos, Attr::Ant, Attr::Pac, Attr::Cnt]);

    // 1. Pressing with cover behind
    let pressers: Vec<&SynSlot<'_>> = outf
        .iter()
        .copied()
        .filter(|x| x.tag(RoleTag::Hp))
        .collect();
    let p_cover: f32 = if !pressers.is_empty() {
        let mut sum = 0.0_f32;
        for &pr in &pressers {
            let cov = outf
                .iter()
                .copied()
                .filter(|c| {
                    c.idx != pr.idx
                        && !c.tag(RoleTag::Hp)
                        && !c.tag(RoleTag::Out)
                        && ((pr.si.group == PosGroup::Dc
                            && c.si.group == PosGroup::Dc
                            && (c.si.y - pr.si.y).abs() <= 6.0)
                            || (c.si.y < pr.si.y
                                && (lane(c) == lane(pr) || lane(c) == Side::C || lane(pr) == Side::C)
                                && pr.si.y - c.si.y <= 30.0))
                })
                .map(|c| {
                    let mult = if c.tag(RoleTag::Low) || c.tag(RoleTag::Cw) || c.tag(RoleTag::Drop) {
                        1.08
                    } else {
                        1.0
                    };
                    (c, cover_q(c.p) * mult)
                })
                .max_by(|a, b| a.1.total_cmp(&b.1));

            if let Some((cov_slot, cov_v)) = cov {
                let v = (press_q(pr.p) * cov_v.min(1.0)).sqrt();
                let prl = pr.label();
                let cl = cov_slot.label();
                d_links.push(mk_link(
                    SynergyLinkType::Press,
                    SynergyPhase::Def,
                    pr,
                    cov_slot,
                    v,
                    match lang {
                        Lang::Pt => format!("{prl} sai a pressionar, {cl} cobre"),
                        Lang::En => format!("{prl} steps out to press, {cl} covers"),
                    },
                ));
                sum += v;
            } else {
                let prl = pr.label();
                d_iss.push(SynergyIssue {
                    key: "press".to_string(),
                    phase: SynergyPhase::Def,
                    text: match lang {
                        Lang::Pt => {
                            format!("{prl} sai a pressionar sem ninguém a cobrir atrás dele.")
                        }
                        Lang::En => {
                            format!("{prl} steps out to press with nobody covering behind him.")
                        }
                    },
                    fix: lang
                        .pick(
                            "Um papel de cobertura atrás (Central de Cobertura, Médio Defensivo de Proteção, Médio Centro de Cobertura) ou troque-o por um papel normal.",
                            "A covering role behind (Covering Centre-Back, Screening Defensive Midfielder, Wide Covering Central Midfielder) or switch him to a standard role.",
                        )
                        .to_string(),
                });
                sum += 0.25;
            }
        }
        sum / pressers.len() as f32
    } else {
        if ti_val(t, "loe") == 2 {
            d_iss.push(SynergyIssue {
                key: "press".to_string(),
                phase: SynergyPhase::Def,
                text: lang
                    .pick(
                        "Linha de pressão alta sem nenhum papel que saia a pressionar.",
                        "A high line of engagement with no role that steps out to press.",
                    )
                    .to_string(),
                fix: lang
                    .pick(
                        "Médio Trabalhador, Lateral Trabalhador ou Avançado Esforçado.",
                        "Pressing Central Midfielder, Pressing Full-Back or Tracking Centre Forward.",
                    )
                    .to_string(),
            });
        }
        0.65
    };

    // 2. Centre-back pair: stopper + cover
    let cbs: Vec<&SynSlot<'_>> = outf
        .iter()
        .copied()
        .filter(|x| x.si.group == PosGroup::Dc)
        .collect();
    let mut cb_pair = 0.8_f32;
    if cbs.len() >= 2 {
        let st_n = cbs.iter().filter(|x| x.tag(RoleTag::Hp)).count();
        let cv_n = cbs
            .iter()
            .filter(|x| x.tag(RoleTag::Low) || x.tag(RoleTag::Cw))
            .count();
        if st_n >= 1 && cv_n >= 1 {
            cb_pair = 1.0;
        } else if st_n >= 2 {
            cb_pair = 0.35;
            d_iss.push(SynergyIssue {
                key: "cb".to_string(),
                phase: SynergyPhase::Def,
                text: lang
                    .pick(
                        "Todos os centrais saem da linha a pressionar: o espaço nas costas fica aberto.",
                        "Every centre-back steps out of the line: the space behind is left open.",
                    )
                    .to_string(),
                fix: lang
                    .pick(
                        "Combine um Central de Antecipação com um Central de Cobertura.",
                        "Pair a Stopping Centre-Back with a Covering Centre-Back.",
                    )
                    .to_string(),
            });
        }
    }

    // 3. Screen in front of centre-backs
    let screens: Vec<&SynSlot<'_>> = outf
        .iter()
        .copied()
        .filter(|x| {
            matches!(x.si.group, PosGroup::Dm | PosGroup::Mc)
                && !x.tag(RoleTag::Hp)
                && !x.tag(RoleTag::Out)
        })
        .collect();
    let screen = if screens.is_empty() {
        d_iss.push(SynergyIssue {
            key: "screen".to_string(),
            phase: SynergyPhase::Def,
            text: lang
                .pick(
                    "Ninguém protege a zona à frente dos centrais: os médios saem todos a pressionar.",
                    "Nobody protects the zone in front of the centre-backs: every midfielder steps out to press.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Um Médio Defensivo de Proteção ou um Médio Centro de Proteção.",
                    "A Screening Defensive Midfielder or a Screening Central Midfielder.",
                )
                .to_string(),
        });
        0.3
    } else {
        let best_scr = screens
            .iter()
            .map(|x| q(x.p, &[Attr::Pos, Attr::Ant, Attr::Tck, Attr::Cnt, Attr::Tea]))
            .fold(0.0_f32, f32::max);
        best_scr
            * if screens.iter().any(|x| x.tag(RoleTag::Low)) {
                1.06
            } else {
                1.0
            }
    };

    // 4. Flank defence
    let mut flk = 0.0_f32;
    let mut nf = 0_usize;
    for side in [Side::L, Side::R] {
        let wa: Vec<&SynSlot<'_>> = outf
            .iter()
            .copied()
            .filter(|x| {
                x.si.side == side && matches!(x.si.group, PosGroup::AmLr | PosGroup::MLr)
            })
            .collect();
        let fb: Vec<&SynSlot<'_>> = outf
            .iter()
            .copied()
            .filter(|x| {
                x.si.side == side && matches!(x.si.group, PosGroup::DLr | PosGroup::WbLr)
            })
            .collect();
        if wa.is_empty() && fb.is_empty() {
            continue;
        }
        nf += 1;
        let cw_help = outf
            .iter()
            .any(|x| x.tag(RoleTag::Cw) && (x.si.side == side || lane(x) == Side::C));
        let nm_pt = if side == Side::L { "esquerdo" } else { "direito" };
        let nm_en = if side == Side::L { "Left" } else { "Right" };

        if wa.iter().any(|x| x.tag(RoleTag::Out)) {
            let fb_hold = fb.iter().any(|x| x.tag(RoleTag::Low));
            let fb_press = fb.iter().any(|x| x.tag(RoleTag::Hp));
            if fb_press && !cw_help {
                flk += 0.25;
                d_iss.push(SynergyIssue {
                    key: "flank".to_string(),
                    phase: SynergyPhase::Def,
                    text: match lang {
                        Lang::Pt => format!(
                            "Flanco {nm_pt}: o extremo fica na frente e o lateral sai a pressionar — o corredor fica vazio."
                        ),
                        Lang::En => format!(
                            "{nm_en} flank: the winger stays forward and the full-back steps out to press — the channel is left empty."
                        ),
                    },
                    fix: lang
                        .pick(
                            "Lateral Fixo nesse lado, ou um Médio de Cobertura que feche o corredor.",
                            "A Holding Full-Back on that side, or a Wide Covering midfielder to close the channel.",
                        )
                        .to_string(),
                });
            } else if fb_hold || cw_help {
                flk += 0.8;
                let o = wa.iter().copied().find(|x| x.tag(RoleTag::Out));
                let h = fb
                    .iter()
                    .copied()
                    .find(|x| x.tag(RoleTag::Low))
                    .or_else(|| outf.iter().copied().find(|x| x.tag(RoleTag::Cw)));
                if let (Some(o), Some(h)) = (o, h) {
                    let ol = o.label();
                    let hl = h.label();
                    d_links.push(mk_link(
                        SynergyLinkType::FlankDef,
                        SynergyPhase::Def,
                        o,
                        h,
                        0.8,
                        match lang {
                            Lang::Pt => {
                                format!("Flanco {nm_pt}: {ol} fica na frente, {hl} segura o corredor")
                            }
                            Lang::En => {
                                format!("{nm_en} flank: {ol} stays forward, {hl} holds the channel")
                            }
                        },
                    ));
                }
            } else {
                flk += 0.5;
                d_iss.push(SynergyIssue {
                    key: "flank".to_string(),
                    phase: SynergyPhase::Def,
                    text: match lang {
                        Lang::Pt => format!(
                            "Flanco {nm_pt}: o extremo fica na frente e o lateral fica sozinho contra dois."
                        ),
                        Lang::En => format!(
                            "{nm_en} flank: the winger stays forward and the full-back is left alone against two."
                        ),
                    },
                    fix: lang
                        .pick(
                            "Lateral Fixo, ou um Médio de Cobertura que dobre nesse lado.",
                            "A Holding Full-Back, or a Wide Covering midfielder doubling up on that side.",
                        )
                        .to_string(),
                });
            }
        } else if let Some(&tw) = wa.iter().find(|x| x.tag(RoleTag::Trk)) {
            flk += 1.0;
            if let Some(&f) = fb.first() {
                let twl = tw.label();
                let fl = f.label();
                d_links.push(mk_link(
                    SynergyLinkType::FlankDef,
                    SynergyPhase::Def,
                    tw,
                    f,
                    1.0,
                    match lang {
                        Lang::Pt => format!("Flanco {nm_pt}: {twl} acompanha e dobra com {fl}"),
                        Lang::En => format!("{nm_en} flank: {twl} tracks back and doubles up with {fl}"),
                    },
                ));
            }
        } else {
            flk += 0.8;
        }
    }
    let flank_def = if nf > 0 { flk / nf as f32 } else { 0.7 };

    // 5. Defensive line vs CB pace & Sweeper Keeper
    let dl = ti_val(t, "dline");
    let gk_sk = oox
        .iter()
        .any(|x| x.si.group == PosGroup::Gk && x.role.id == "swk");
    let cb_pace = if cbs.is_empty() {
        0.6
    } else {
        cbs.iter()
            .map(|x| q(x.p, &[Attr::Pac, Attr::Acc]))
            .sum::<f32>()
            / cbs.len() as f32
    };
    let line = if dl >= 3 {
        (cb_pace / 0.72).min(1.0) * if gk_sk { 1.0 } else { 0.72 }
    } else if dl <= 1 {
        0.9
    } else {
        (0.75 + cb_pace * 0.3).min(1.0)
    };
    if dl >= 3 && (cb_pace < 0.62 || !gk_sk) {
        d_iss.push(SynergyIssue {
            key: "line".to_string(),
            phase: SynergyPhase::Def,
            text: lang
                .pick(
                    "Linha alta sem os perfis para a suportar (centrais rápidos e Guarda-Redes Líbero).",
                    "A high line without the profiles to support it (quick centre-backs and a Sweeper Keeper).",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Baixe a linha, ou Guarda-Redes Líbero e centrais com Velocidade/Aceleração.",
                    "Drop the line, or a Sweeper Keeper and centre-backs with Pace/Acceleration.",
                )
                .to_string(),
        });
    }

    // 6. Aerial defence
    let air_d = if cbs.is_empty() {
        0.5
    } else {
        let mut vals: Vec<f32> = cbs
            .iter()
            .map(|x| q(x.p, &[Attr::Hea, Attr::Jum, Attr::Str, Attr::Pos]))
            .collect();
        vals.sort_by(|a, b| b.total_cmp(a));
        vals.truncate(2);
        vals.iter().sum::<f32>() / vals.len() as f32
    };
    if air_d < 0.6 && ti_val(t, "cross_eng") == 0 {
        d_iss.push(SynergyIssue {
            key: "air".to_string(),
            phase: SynergyPhase::Def,
            text: lang
                .pick(
                    "Convida cruzamentos com centrais fracos no ar.",
                    "Inviting crosses with centre-backs weak in the air.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Impedir Cruzamentos, ou centrais mais fortes no ar.",
                    "Stop Crosses, or centre-backs stronger in the air.",
                )
                .to_string(),
        });
    }

    let out_n = outf.iter().filter(|x| x.tag(RoleTag::Out)).count();
    let mut d_parts = BTreeMap::new();
    let pd_press = pct(p_cover);
    let pd_cb = pct(cb_pair);
    let pd_screen = pct(screen.min(1.0));
    let pd_flankd = pct(flank_def);
    let pd_line = pct(line);
    let pd_air = pct(air_d);
    d_parts.insert("press".to_string(), pd_press);
    d_parts.insert("cb".to_string(), pd_cb);
    d_parts.insert("screen".to_string(), pd_screen);
    d_parts.insert("flankd".to_string(), pd_flankd);
    d_parts.insert("line".to_string(), pd_line);
    d_parts.insert("air".to_string(), pd_air);
    let def_score = (pd_press as f32 * 0.22
        + pd_cb as f32 * 0.12
        + pd_screen as f32 * 0.20
        + pd_flankd as f32 * 0.22
        + pd_line as f32 * 0.14
        + pd_air as f32 * 0.10)
        .round()
        .clamp(0.0, 100.0) as u8;
    let def = SynergyPhaseScore {
        score: def_score,
        parts: d_parts,
    };

    // ---------------- TRANSITIONS
    let mut t_iss: Vec<SynergyIssue> = Vec::new();
    let mut t_links: Vec<SynergyLink> = Vec::new();

    let rd: Vec<&SynSlot<'_>> = ipx
        .iter()
        .filter(|x| x.tag(RoleTag::Rd) && x.si.group != PosGroup::Gk)
        .collect();
    let rd_q = if rd.is_empty() {
        0.4
    } else {
        rd.iter()
            .map(|x| q(x.p, &[Attr::Pac, Attr::Acc, Attr::Pos, Attr::Ant]))
            .sum::<f32>()
            / rd.len() as f32
    };
    let rd_n = rd.len();
    let rd_count: f32 = match rd_n {
        0 => 0.1,
        1 => 0.3,
        2 => 0.65,
        3 | 4 => 1.0,
        5 => 0.8,
        _ => 0.7,
    };
    let loss = rd_count * (0.55 + 0.45 * rd_q);
    if rd_n >= 3 && rd_q < 0.6 {
        t_iss.push(SynergyIssue {
            key: "loss".to_string(),
            phase: SynergyPhase::Trans,
            text: lang
                .pick(
                    "Quem fica atrás é lento: os contra-ataques adversários chegam em vantagem.",
                    "Those who stay back are slow: opposition counter-attacks arrive with the advantage.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Um Médio Defensivo / lateral mais rápido na rest-defence, ou linha mais baixa.",
                    "A quicker Defensive Midfielder / full-back in the rest-defence, or a deeper line.",
                )
                .to_string(),
        });
    }

    let mut breakers: Vec<&SynSlot<'_>> = oox
        .iter()
        .filter(|x| x.tag(RoleTag::Out))
        .collect();
    breakers.extend(
        ipx.iter()
            .filter(|x| x.tag(RoleTag::Run) && matches!(x.si.group, PosGroup::St | PosGroup::AmLr)),
    );
    let best_run = breakers
        .iter()
        .copied()
        .map(|x| (x, q(x.p, &[Attr::Pac, Attr::Acc, Attr::Otb, Attr::Dri])))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let releaser = ipx
        .iter()
        .filter(|x| {
            matches!(x.si.group, PosGroup::Dm | PosGroup::Mc | PosGroup::Amc)
                || x.tag(RoleTag::Cr)
        })
        .map(|x| (x, q(x.p, &[Attr::Vis, Attr::Pas, Attr::Dec, Attr::Tec])))
        .max_by(|a, b| a.1.total_cmp(&b.1));

    let win = match (best_run, releaser) {
        (Some((br, br_v)), Some((rel, rel_v))) => {
            let v = (br_v * rel_v).sqrt();
            if br.idx != rel.idx {
                let rl = rel.label();
                let bl = br.label();
                t_links.push(mk_link(
                    SynergyLinkType::Counter,
                    SynergyPhase::Trans,
                    rel,
                    br,
                    v,
                    match lang {
                        Lang::Pt => format!("{rl} lança {bl} no contra-ataque"),
                        Lang::En => format!("{rl} releases {bl} on the counter"),
                    },
                ));
            }
            v
        }
        _ => 0.35,
    };
    if ti_val(t, "att_trans") == 2 && win < 0.62 {
        t_iss.push(SynergyIssue {
            key: "win".to_string(),
            phase: SynergyPhase::Trans,
            text: lang
                .pick(
                    "Contra-Ataque escolhido, mas sem velocidade na frente ou sem quem lance a bola.",
                    "Counter-Attack chosen, but without pace up front or anyone to release the ball.",
                )
                .to_string(),
            fix: lang
                .pick(
                    "Um Avançado de Desmarcação / Extremo de Transição rápido e um médio com Visão.",
                    "A quick Splitting Outlet / Wide Outlet and a midfielder with Vision.",
                )
                .to_string(),
        });
    }
    if out_n >= 3 {
        t_iss.push(SynergyIssue {
            key: "out".to_string(),
            phase: SynergyPhase::Trans,
            text: match lang {
                Lang::Pt => format!(
                    "{out_n} jogadores ficam na frente sem defender: o bloco fica em inferioridade."
                ),
                Lang::En => format!(
                    "{out_n} players stay forward without defending: the block is outnumbered."
                ),
            },
            fix: lang
                .pick("No máximo 1–2 papéis de saída.", "At most 1–2 outlet roles.")
                .to_string(),
        });
    }
    let out_bal = if out_n == 0 {
        0.7
    } else if out_n <= 2 {
        1.0
    } else {
        0.4
    };

    let mut t_parts = BTreeMap::new();
    let pt_loss = pct(loss);
    let pt_win = pct(win);
    let pt_out = pct(out_bal);
    t_parts.insert("loss".to_string(), pt_loss);
    t_parts.insert("win".to_string(), pt_win);
    t_parts.insert("out".to_string(), pt_out);
    let trans_score = (pt_loss as f32 * 0.50 + pt_win as f32 * 0.35 + pt_out as f32 * 0.15)
        .round()
        .clamp(0.0, 100.0) as u8;
    let trans = SynergyPhaseScore {
        score: trans_score,
        parts: t_parts,
    };

    let mut phases = BTreeMap::new();
    phases.insert(SynergyPhase::Build, build);
    phases.insert(
        SynergyPhase::Att,
        SynergyPhaseScore {
            score: att.score,
            parts: att.parts.clone(),
        },
    );
    phases.insert(SynergyPhase::Def, def);
    phases.insert(SynergyPhase::Trans, trans);

    let score = (phases[&SynergyPhase::Build].score as f32 * 0.15
        + phases[&SynergyPhase::Att].score as f32 * 0.40
        + phases[&SynergyPhase::Def].score as f32 * 0.30
        + phases[&SynergyPhase::Trans].score as f32 * 0.15)
        .round()
        .clamp(0.0, 100.0) as u8;

    d_links.sort_by(|a, b| b.value.total_cmp(&a.value));
    d_links.truncate(5);

    let mut links = Vec::new();
    links.extend(b_links);
    links.extend(att.links);
    links.extend(d_links);
    links.extend(t_links);

    let mut issues = Vec::new();
    issues.extend(b_iss);
    issues.extend(att.issues);
    issues.extend(d_iss);
    issues.extend(t_iss);

    SynergyReport {
        score,
        phases,
        parts: att.parts,
        att_score: att.score,
        links,
        issues,
        creators: att.creators,
        box_n: att.box_n,
        out_n,
        rd_n,
    }
}

/// Localized phase label for synergy reports.
pub const fn synergy_phase_name(phase: SynergyPhase, lang: Lang) -> &'static str {
    match (phase, lang) {
        (SynergyPhase::Build, Lang::Pt) => "Construção",
        (SynergyPhase::Build, Lang::En) => "Build-up",
        (SynergyPhase::Att, Lang::Pt) => "Ataque",
        (SynergyPhase::Att, Lang::En) => "Attack",
        (SynergyPhase::Def, Lang::Pt) => "Defesa",
        (SynergyPhase::Def, Lang::En) => "Defence",
        (SynergyPhase::Trans, Lang::Pt) => "Transições",
        (SynergyPhase::Trans, Lang::En) => "Transitions",
    }
}

/// Localized sub-component label for synergy parts.
pub fn synergy_part_name(key: &str, lang: Lang) -> &'static str {
    match key {
        "chain" => lang.pick("Criador → finalizador", "Creator → finisher"),
        "cross" => lang.pick("Cruzamentos → alvos", "Crosses → targets"),
        "front" => lang.pick("Dupla / referência da frente", "Front line partnership"),
        "box" => lang.pick("Presença na área", "Box presence"),
        "flank" => lang.pick("Combinações de flanco", "Flank combinations"),
        "load" => lang.pick("Carga criativa", "Creative load"),
        "short" => lang.pick(
            "Saída curta: quem passa → quem recebe",
            "Short build-up: passer → receiver",
        ),
        "support" => lang.pick("Apoios entre linhas", "Support between the lines"),
        "resist" => lang.pick("Resistência à pressão", "Press resistance"),
        "long" => lang.pick("Bola longa → referência", "Long ball → target"),
        "press" => lang.pick("Pressão com cobertura", "Pressing with cover"),
        "cb" => lang.pick(
            "Dupla de centrais (sai + cobre)",
            "Centre-back pair (step + cover)",
        ),
        "screen" => lang.pick(
            "Proteção à frente dos centrais",
            "Screen in front of the centre-backs",
        ),
        "flankd" => lang.pick("Defesa dos flancos", "Flank defence"),
        "line" => lang.pick("Linha vs. perfis", "Line vs. profiles"),
        "air" => lang.pick("Jogo aéreo defensivo", "Aerial defence"),
        "loss" => lang.pick(
            "Reação à perda (rest-defence)",
            "Reaction to losing the ball (rest-defence)",
        ),
        "win" => lang.pick(
            "Contra-ataque: quem lança → quem arranca",
            "Counter: releaser → runner",
        ),
        "out" => lang.pick("Saídas vs. bloco", "Outlets vs. block"),
        _ => "",
    }
}
