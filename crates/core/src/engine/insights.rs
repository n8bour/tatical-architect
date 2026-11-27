//! Insights: recommended player instructions, bench & youth successors,
//! opponent scouting report, set-piece ratings, and set-piece planner.

use crate::data::roles::attr_name;
use crate::data::tactics_data::{
    has_player_instruction, slot_info, template_by_id, ti_val, TEMPLATES,
};
use crate::engine::hungarian::{auto_pick_xi, tactic_from_template};
use crate::engine::parsers::{majority_club, surname};
use crate::engine::scoring::slot_score;
use crate::types::{
    Attr, AttrLookup, BenchCandidate, CornerTaker, Foot, Lang, OpponentDanger, OpponentPlanItem,
    OpponentReport, OpponentSector, PiRecommendation, Player, PosGroup, Role, RoleTag,
    SetPiecePlan, SetPieceRole, SetPieceTaker, SetPieceTip, SlotBench, Tactic, XiMember,
};
use std::collections::{HashMap, HashSet};

fn fmt_val(v: f32) -> String {
    if (v - v.round()).abs() < 1e-4 {
        format!("{}", v.round() as i32)
    } else {
        format!("{v:.1}")
    }
}

fn avg_of(p: &Player, keys: &[Attr]) -> Option<f32> {
    let mut sum = 0.0_f32;
    let mut count = 0_usize;
    for &k in keys {
        if let Some(v) = p.attr(k) {
            sum += v;
            count += 1;
        }
    }
    (count > 0).then(|| sum / count as f32)
}

fn avg_players(players: &[&Player], keys: &[Attr]) -> Option<f32> {
    let mut sum = 0.0_f32;
    let mut count = 0_usize;
    for &p in players {
        if let Some(v) = avg_of(p, keys) {
            sum += v;
            count += 1;
        }
    }
    (count > 0).then(|| sum / count as f32)
}

/// Recommend up to 3 FM27 player instructions for a player in a given role and position group.
pub fn recommend_pis(
    p: &Player,
    ip_role: Option<&Role>,
    _oop_role: Option<&Role>,
    group: PosGroup,
    lang: Lang,
) -> Vec<PiRecommendation> {
    if group == PosGroup::Gk {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut add = |pi: &'static str, why: String| {
        if has_player_instruction(pi) {
            out.push(PiRecommendation { pi, why });
        }
    };

    let a = |k: Attr| p.attr(k).unwrap_or(0.0);
    let has_tag = |tag: RoleTag| ip_role.is_some_and(|r| r.has_tag(tag));
    let attacker = matches!(
        group,
        PosGroup::St | PosGroup::Amc | PosGroup::AmLr | PosGroup::MLr
    );

    if attacker && (a(Attr::Pac) >= 15.0 || a(Attr::Acc) >= 15.0) && a(Attr::Otb) >= 13.0 {
        let spd = fmt_val(a(Attr::Pac).max(a(Attr::Acc)));
        let otb = fmt_val(a(Attr::Otb));
        add(
            "Attack the Defensive Line",
            match lang {
                Lang::Pt => format!("Velocidade {spd} e Sem Bola {otb}: ganha nas costas da defesa."),
                Lang::En => format!("Pace {spd} and Off the Ball {otb}: gets in behind the defence."),
            },
        );
    }

    if group == PosGroup::St
        && a(Attr::Pac) >= 14.0
        && a(Attr::Otb) >= 12.0
        && !has_tag(RoleTag::Drop)
    {
        add(
            "Move into channels",
            lang.pick(
                "Rápido e com bom movimento: arrasta os centrais para os corredores.",
                "Quick with good movement: drags centre-backs into the channels.",
            )
            .to_string(),
        );
    }

    if matches!(group, PosGroup::Dm | PosGroup::Mc | PosGroup::Amc)
        && a(Attr::Otb) >= 13.0
        && (has_tag(RoleTag::Adv) || has_tag(RoleTag::Box) || has_tag(RoleTag::Run))
    {
        let otb = fmt_val(a(Attr::Otb));
        add(
            "Arrive Late",
            match lang {
                Lang::Pt => format!(
                    "Sem Bola {otb}: entra na área em segunda vaga, quando a defesa já está presa."
                ),
                Lang::En => format!(
                    "Off the Ball {otb}: arrives in the box as a second wave, once the defence is pinned."
                ),
            },
        );
    }

    if a(Attr::Lon) >= 15.0 && a(Attr::Tec) >= 13.0 {
        let lon = fmt_val(a(Attr::Lon));
        add(
            "Shoot more",
            match lang {
                Lang::Pt => format!("Remates de Longe {lon}: vale a pena arriscar de fora da área."),
                Lang::En => format!("Long Shots {lon}: worth trying from outside the box."),
            },
        );
    } else if attacker && a(Attr::Fin) > 0.0 && a(Attr::Fin) <= 10.0 && a(Attr::Lon) <= 11.0 {
        let fin = fmt_val(a(Attr::Fin));
        let lon = fmt_val(a(Attr::Lon));
        add(
            "Shoot less",
            match lang {
                Lang::Pt => {
                    format!("Finalização {fin} e Remates de Longe {lon}: melhor procurar o passe.")
                }
                Lang::En => {
                    format!("Finishing {fin} and Long Shots {lon}: better to look for the pass.")
                }
            },
        );
    }

    if a(Attr::Dri) >= 15.0 && a(Attr::Agi) >= 13.0 {
        let dri = fmt_val(a(Attr::Dri));
        let agi = fmt_val(a(Attr::Agi));
        add(
            "Dribble more",
            match lang {
                Lang::Pt => format!("Drible {dri}, Agilidade {agi}: ganha duelos individuais."),
                Lang::En => format!("Dribbling {dri}, Agility {agi}: wins one-on-one duels."),
            },
        );
    } else if a(Attr::Dri) > 0.0 && a(Attr::Dri) <= 9.0 && attacker {
        let dri = fmt_val(a(Attr::Dri));
        add(
            "Dribble less",
            match lang {
                Lang::Pt => format!("Drible {dri}: perde muitas bolas a conduzir."),
                Lang::En => format!("Dribbling {dri}: loses the ball often when carrying it."),
            },
        );
    }

    if a(Attr::Pas) >= 15.0 && a(Attr::Vis) >= 15.0 {
        let pas = fmt_val(a(Attr::Pas));
        let vis = fmt_val(a(Attr::Vis));
        add(
            "Take more risks",
            match lang {
                Lang::Pt => format!("Passe {pas} e Visão {vis}: pode tentar o passe de ruptura."),
                Lang::En => {
                    format!("Passing {pas} and Vision {vis}: can try the defence-splitting pass.")
                }
            },
        );
    } else if a(Attr::Pas) > 0.0
        && a(Attr::Pas) <= 10.0
        && (has_tag(RoleTag::Cr) || matches!(group, PosGroup::Dc | PosGroup::Dm))
    {
        let pas = fmt_val(a(Attr::Pas));
        add(
            "Take fewer risks",
            match lang {
                Lang::Pt => format!("Passe {pas}: passes simples evitam perdas perigosas."),
                Lang::En => format!("Passing {pas}: simple passes avoid dangerous turnovers."),
            },
        );
    }

    if group.is_wide_group() && a(Attr::Cro) >= 15.0 {
        let cro = fmt_val(a(Attr::Cro));
        add(
            if has_tag(RoleTag::In) {
                "Cross from deep"
            } else {
                "Cross from byline"
            },
            match lang {
                Lang::Pt => format!("Cruzamentos {cro}: é uma arma, use-a."),
                Lang::En => format!("Crossing {cro}: it is a weapon, use it."),
            },
        );
    }

    if a(Attr::Sta) > 0.0 && a(Attr::Sta) <= 10.0 {
        let sta = fmt_val(a(Attr::Sta));
        add(
            "Press less",
            match lang {
                Lang::Pt => format!("Resistência {sta}: guarda energia para os 90 minutos."),
                Lang::En => format!("Stamina {sta}: save energy for the full 90 minutes."),
            },
        );
    } else if a(Attr::Wor) >= 15.0 && a(Attr::Sta) >= 14.0 && attacker {
        let wor = fmt_val(a(Attr::Wor));
        let sta = fmt_val(a(Attr::Sta));
        add(
            "Press more",
            match lang {
                Lang::Pt => format!(
                    "Índice de Trabalho {wor} e Resistência {sta}: aguenta pressionar mais."
                ),
                Lang::En => format!("Work Rate {wor} and Stamina {sta}: can press harder."),
            },
        );
    }

    if matches!(
        group,
        PosGroup::Dc | PosGroup::DLr | PosGroup::WbLr | PosGroup::Dm
    ) {
        if a(Attr::Tck) >= 15.0 && a(Attr::Agg) >= 13.0 {
            let tck = fmt_val(a(Attr::Tck));
            let agg = fmt_val(a(Attr::Agg));
            add(
                "Tackle harder",
                match lang {
                    Lang::Pt => format!("Desarme {tck}, Agressividade {agg}: ganha duelos."),
                    Lang::En => format!("Tackling {tck}, Aggression {agg}: wins duels."),
                },
            );
        } else if a(Attr::Tck) > 0.0 && a(Attr::Tck) <= 10.0 {
            let tck = fmt_val(a(Attr::Tck));
            add(
                "Ease off tackles",
                match lang {
                    Lang::Pt => format!("Desarme {tck}: evita faltas e cartões."),
                    Lang::En => format!("Tackling {tck}: avoids fouls and cards."),
                },
            );
        }
        if a(Attr::Mar) >= 15.0 {
            let mar = fmt_val(a(Attr::Mar));
            add(
                "Mark tighter",
                match lang {
                    Lang::Pt => format!("Marcação {mar}: anula o adversário directo."),
                    Lang::En => format!("Marking {mar}: shuts down his direct opponent."),
                },
            );
        }
    }

    if group == PosGroup::St {
        if has_tag(RoleTag::Air) || (a(Attr::Hea) >= 15.0 && a(Attr::Jum) >= 14.0) {
            let hea = fmt_val(a(Attr::Hea));
            add(
                "Target Centre",
                match lang {
                    Lang::Pt => format!("Cabeceamento {hea}: ataca o centro da área nos cruzamentos."),
                    Lang::En => {
                        format!("Heading {hea}: attacks the centre of the box on crosses.")
                    }
                },
            );
        } else if ip_role.is_some_and(|r| r.id == "poa") {
            add(
                "Target Near Post",
                lang.pick(
                    "Poacher: o primeiro poste é onde aparece primeiro.",
                    "Poacher: the near post is where he gets there first.",
                )
                .to_string(),
            );
        }
    }

    if ip_role.is_some_and(|r| r.id == "wf_ip") {
        add(
            "Target Far Post",
            lang.pick(
                "Wide Forward: ataca o segundo poste enquanto o avançado prende os centrais.",
                "Wide Forward: attacks the far post while the striker pins the centre-backs.",
            )
            .to_string(),
        );
    }

    if matches!(group, PosGroup::Dm | PosGroup::Mc)
        && has_tag(RoleTag::Rd)
        && a(Attr::Dec) >= 12.0
    {
        add(
            "Hold position",
            lang.pick(
                "Garante a rest-defence enquanto os outros sobem.",
                "Secures the rest-defence while the others push on.",
            )
            .to_string(),
        );
    }

    out.truncate(3);
    out
}

/// Best bench cover and best U21 successor per slot in `t`.
pub fn bench_and_successors(t: &Tactic, players: &[Player]) -> Vec<SlotBench> {
    let used: HashSet<&str> = t.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
    let rest: Vec<&Player> = players
        .iter()
        .filter(|p| !used.contains(p.id.as_str()))
        .collect();

    t.slots
        .iter()
        .map(|s| {
            let mut ranked: Vec<(&Player, u8)> =
                rest.iter().map(|&p| (p, slot_score(p, t, s))).collect();
            ranked.sort_by_key(|a| std::cmp::Reverse(a.1));

            let bench = ranked
                .first()
                .filter(|&&(_, sc)| sc >= 30)
                .map(|&(p, score)| BenchCandidate {
                    player_id: p.id.clone(),
                    player_name: p.name.clone(),
                    age: p.age,
                    score,
                });

            let succ = ranked
                .iter()
                .find(|&&(p, sc)| p.age.is_some_and(|age| age <= 21.0) && sc >= 40)
                .map(|&(p, score)| BenchCandidate {
                    player_id: p.id.clone(),
                    player_name: p.name.clone(),
                    age: p.age,
                    score,
                });

            SlotBench { bench, succ }
        })
        .collect()
}

struct OppSectorDef {
    key: &'static str,
    name_pt: &'static str,
    them: &'static [PosGroup],
    a: &'static [Attr],
    us: &'static [PosGroup],
    ua: &'static [Attr],
    dir: &'static str,
}

const OPP_SECTORS: &[OppSectorDef] = &[
    OppSectorDef {
        key: "cb_pace",
        name_pt: "Centrais: velocidade",
        them: &[PosGroup::Dc],
        a: &[Attr::Pac, Attr::Acc],
        us: &[PosGroup::St, PosGroup::AmLr, PosGroup::Amc],
        ua: &[Attr::Pac, Attr::Acc],
        dir: "attack",
    },
    OppSectorDef {
        key: "cb_air",
        name_pt: "Centrais: jogo aéreo",
        them: &[PosGroup::Dc],
        a: &[Attr::Hea, Attr::Jum, Attr::Str],
        us: &[PosGroup::St],
        ua: &[Attr::Hea, Attr::Jum, Attr::Str],
        dir: "attack",
    },
    OppSectorDef {
        key: "fb_def",
        name_pt: "Laterais a defender",
        them: &[PosGroup::DLr, PosGroup::WbLr],
        a: &[Attr::Tck, Attr::Mar, Attr::Pos, Attr::Acc],
        us: &[PosGroup::AmLr, PosGroup::MLr],
        ua: &[Attr::Dri, Attr::Acc, Attr::Agi, Attr::Tec],
        dir: "attack",
    },
    OppSectorDef {
        key: "mid_create",
        name_pt: "Criação no meio-campo",
        them: &[PosGroup::Dm, PosGroup::Mc, PosGroup::Amc],
        a: &[Attr::Pas, Attr::Vis, Attr::Dec, Attr::Tec, Attr::Fir],
        us: &[PosGroup::Dm, PosGroup::Mc],
        ua: &[Attr::Pos, Attr::Tck, Attr::Ant, Attr::Dec],
        dir: "defend",
    },
    OppSectorDef {
        key: "press",
        name_pt: "Pressão",
        them: &[
            PosGroup::Dm,
            PosGroup::Mc,
            PosGroup::Amc,
            PosGroup::AmLr,
            PosGroup::St,
        ],
        a: &[Attr::Wor, Attr::Sta, Attr::Agg, Attr::Acc],
        us: &[PosGroup::Dc, PosGroup::Dm, PosGroup::Gk],
        ua: &[Attr::Pas, Attr::Cmp, Attr::Fir, Attr::Dec],
        dir: "defend",
    },
    OppSectorDef {
        key: "wing",
        name_pt: "Extremos: perigo",
        them: &[PosGroup::AmLr, PosGroup::MLr],
        a: &[Attr::Dri, Attr::Acc, Attr::Pac, Attr::Cro],
        us: &[PosGroup::DLr, PosGroup::WbLr],
        ua: &[Attr::Tck, Attr::Mar, Attr::Acc, Attr::Pos],
        dir: "defend",
    },
    OppSectorDef {
        key: "fin",
        name_pt: "Finalização",
        them: &[PosGroup::St, PosGroup::Amc],
        a: &[Attr::Fin, Attr::Cmp, Attr::Otb],
        us: &[PosGroup::Dc],
        ua: &[Attr::Pos, Attr::Ant, Attr::Cnt, Attr::Mar],
        dir: "defend",
    },
    OppSectorDef {
        key: "fw_pace",
        name_pt: "Avançados: velocidade",
        them: &[PosGroup::St, PosGroup::AmLr],
        a: &[Attr::Pac, Attr::Acc],
        us: &[PosGroup::Dc],
        ua: &[Attr::Pac, Attr::Acc],
        dir: "defend",
    },
    OppSectorDef {
        key: "air_att",
        name_pt: "Ataque aéreo",
        them: &[PosGroup::St],
        a: &[Attr::Hea, Attr::Jum, Attr::Str],
        us: &[PosGroup::Dc],
        ua: &[Attr::Hea, Attr::Jum, Attr::Str],
        dir: "defend",
    },
];

struct XiSlotRef<'a> {
    p: &'a Player,
    code: String,
    group: PosGroup,
    side: crate::types::Side,
}

impl XiSlotRef<'_> {
    fn to_member(&self) -> XiMember {
        XiMember {
            player_id: self.p.id.clone(),
            player_name: self.p.name.clone(),
            code: self.code.clone(),
            group: self.group,
            side: self.side,
        }
    }
}

fn opponent_xi_refs(opp_players: &[Player]) -> Vec<XiSlotRef<'_>> {
    let tp = template_by_id("balanced4231").unwrap_or(&TEMPLATES[0]);
    let mut t = tactic_from_template(tp);
    auto_pick_xi(&mut t, opp_players);
    let pm: HashMap<&str, &Player> = opp_players.iter().map(|p| (p.id.as_str(), p)).collect();
    t.slots
        .iter()
        .filter_map(|s| {
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied())?;
            let si = slot_info(&t.ip_form, &s.ip_code)?;
            Some(XiSlotRef {
                p,
                code: s.ip_code.clone(),
                group: si.group,
                side: si.side,
            })
        })
        .collect()
}

fn my_xi_refs<'a>(t: &Tactic, players: &'a [Player]) -> Vec<XiSlotRef<'a>> {
    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    let assigned = t
        .slots
        .iter()
        .filter(|s| s.pid.as_deref().is_some_and(|id| pm.contains_key(id)))
        .count();
    let owned: Option<Tactic> = if assigned < 9 && players.len() >= 11 {
        let mut tt = t.clone();
        auto_pick_xi(&mut tt, players);
        Some(tt)
    } else {
        None
    };
    let active = owned.as_ref().unwrap_or(t);
    active
        .slots
        .iter()
        .filter_map(|s| {
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied())?;
            let si = slot_info(&active.ip_form, &s.ip_code)?;
            Some(XiSlotRef {
                p,
                code: s.ip_code.clone(),
                group: si.group,
                side: si.side,
            })
        })
        .collect()
}

/// Compare our XI against an opponent squad across 9 tactical sectors and produce TI/OI advice.
pub fn opponent_report(
    t: &Tactic,
    my_players: &[Player],
    opp_players: &[Player],
    lang: Lang,
) -> OpponentReport {
    let them = opponent_xi_refs(opp_players);
    let us = if my_players.is_empty() {
        Vec::new()
    } else {
        my_xi_refs(t, my_players)
    };

    fn pick<'a>(xi: &[XiSlotRef<'a>], gs: &[PosGroup]) -> Vec<&'a Player> {
        xi.iter()
            .filter(|x| gs.contains(&x.group))
            .map(|x| x.p)
            .collect()
    }

    let sectors: Vec<OpponentSector> = OPP_SECTORS
        .iter()
        .map(|s| {
            let tv = avg_players(&pick(&them, s.them), s.a);
            let uv = if us.is_empty() {
                None
            } else {
                avg_players(&pick(&us, s.us), s.ua)
            };
            let diff = match (tv, uv) {
                (Some(tval), Some(uval)) => Some(uval - tval),
                _ => None,
            };
            OpponentSector {
                key: s.key,
                name_pt: s.name_pt,
                dir: s.dir,
                them_val: tv,
                us_val: uv,
                diff,
            }
        })
        .collect();

    let by_key: HashMap<&str, &OpponentSector> = sectors.iter().map(|s| (s.key, s)).collect();
    let d = |k: &str| by_key.get(k).and_then(|s| s.diff);

    let mut plan: Vec<OpponentPlanItem> = Vec::new();
    let mut push = |ti: &'static str, value: usize, why: String| {
        plan.push(OpponentPlanItem { ti, value, why });
    };

    // Attack adjustments
    if d("cb_pace").is_some_and(|v| v >= 1.5) {
        let s = by_key["cb_pace"];
        let uv = s.us_val.unwrap_or(0.0);
        let tv = s.them_val.unwrap_or(0.0);
        push(
            "reception",
            0,
            match lang {
                Lang::Pt => format!(
                    "Os teus atacantes são mais rápidos do que os centrais deles ({uv:.1} vs {tv:.1}): joga para o espaço nas costas."
                ),
                Lang::En => format!(
                    "Your attackers are quicker than their centre-backs ({uv:.1} vs {tv:.1}): play into the space behind."
                ),
            },
        );
    }
    if d("cb_pace").is_some_and(|v| v <= -1.5) {
        push(
            "reception",
            2,
            lang.pick(
                "Os centrais deles são mais rápidos do que os teus atacantes: passe ao pé, sem bolas nas costas.",
                "Their centre-backs are quicker than your attackers: pass to feet, no balls in behind.",
            )
            .to_string(),
        );
    }
    if d("cb_air").is_some_and(|v| v >= 1.5) {
        let s = by_key["cb_air"];
        let uv = s.us_val.unwrap_or(0.0);
        let tv = s.them_val.unwrap_or(0.0);
        push(
            "crossing",
            3,
            match lang {
                Lang::Pt => format!(
                    "Os teus avançados ganham no ar aos centrais deles ({uv:.1} vs {tv:.1}): cruzamentos altos."
                ),
                Lang::En => format!(
                    "Your strikers beat their centre-backs in the air ({uv:.1} vs {tv:.1}): high crosses."
                ),
            },
        );
        push(
            "set_pieces",
            2,
            lang.pick(
                "Vantagem aérea: procura cantos e livres.",
                "Aerial advantage: go for corners and free kicks.",
            )
            .to_string(),
        );
    }
    if d("cb_air").is_some_and(|v| v <= -1.5) {
        let tv = by_key["cb_air"].them_val.unwrap_or(0.0);
        push(
            "crossing",
            0,
            match lang {
                Lang::Pt => format!("Os centrais deles dominam no ar ({tv:.1}): só cruzamentos rasteiros."),
                Lang::En => format!("Their centre-backs dominate in the air ({tv:.1}): low crosses only."),
            },
        );
    }

    if by_key.get("fb_def").and_then(|s| s.them_val).is_some() {
        let dl = them.iter().find(|x| x.code == "DL");
        let dr = them.iter().find(|x| x.code == "DR");
        let fb_attrs = OPP_SECTORS[2].a;
        let vl = dl.and_then(|x| avg_of(x.p, fb_attrs));
        let vr = dr.and_then(|x| avg_of(x.p, fb_attrs));
        if let (Some(vl), Some(vr), Some(dl_ref), Some(dr_ref)) = (vl, vr, dl, dr) {
            if (vl - vr).abs() >= 1.2 {
                let weak_left = vl < vr;
                let weak_name = if weak_left {
                    &dl_ref.p.name
                } else {
                    &dr_ref.p.name
                };
                push(
                    "progress",
                    if weak_left { 3 } else { 0 },
                    match lang {
                        Lang::Pt => format!(
                            "O lateral {} deles ({weak_name}) é o elo fraco: ataca pelo teu flanco {}.",
                            if weak_left { "esquerdo" } else { "direito" },
                            if weak_left { "direito" } else { "esquerdo" }
                        ),
                        Lang::En => format!(
                            "Their {} full-back ({weak_name}) is the weak link: attack down your {} flank.",
                            if weak_left { "left" } else { "right" },
                            if weak_left { "right" } else { "left" }
                        ),
                    },
                );
            }
        }
    }

    // Defence adjustments
    if let Some(pr) = by_key.get("press").and_then(|s| s.them_val) {
        if pr >= 13.5 {
            push(
                "buildup",
                2,
                match lang {
                    Lang::Pt => format!(
                        "Pressionam muito ({pr:.1}): salta a pressão com bola longa em vez de sair a jogar."
                    ),
                    Lang::En => format!(
                        "They press a lot ({pr:.1}): bypass the press with long balls instead of playing out."
                    ),
                },
            );
        }
        if pr <= 10.5 {
            push(
                "buildup",
                0,
                match lang {
                    Lang::Pt => format!("Pressionam pouco ({pr:.1}): sai a jogar curto com calma."),
                    Lang::En => format!("They press little ({pr:.1}): play out short and calmly."),
                },
            );
        }
    }

    if d("fw_pace").is_some_and(|v| v <= -1.5) {
        let s = by_key["fw_pace"];
        let tv = s.them_val.unwrap_or(0.0);
        let uv = s.us_val.unwrap_or(0.0);
        push(
            "dline",
            1,
            match lang {
                Lang::Pt => format!(
                    "Os avançados deles são mais rápidos do que os teus centrais ({tv:.1} vs {uv:.1}): baixa a linha."
                ),
                Lang::En => format!(
                    "Their forwards are quicker than your centre-backs ({tv:.1} vs {uv:.1}): drop the line."
                ),
            },
        );
    }
    if d("fw_pace").is_some_and(|v| v >= 1.5) {
        push(
            "dline",
            3,
            lang.pick(
                "Os teus centrais são mais rápidos do que os avançados deles: podes subir a linha.",
                "Your centre-backs are quicker than their forwards: you can push the line up.",
            )
            .to_string(),
        );
    }
    if d("air_att").is_some_and(|v| v <= -1.5) {
        let tv = by_key["air_att"].them_val.unwrap_or(0.0);
        push(
            "cross_eng",
            2,
            match lang {
                Lang::Pt => format!(
                    "Os avançados deles são fortes no ar ({tv:.1}): impede os cruzamentos."
                ),
                Lang::En => format!(
                    "Their forwards are strong in the air ({tv:.1}): stop the crosses."
                ),
            },
        );
    }
    if let Some(mc) = by_key.get("mid_create").and_then(|s| s.them_val) {
        if mc >= 14.0 {
            push(
                "press_trap",
                2,
                match lang {
                    Lang::Pt => format!(
                        "O meio-campo deles cria muito ({mc:.1}): empurra o jogo para as alas."
                    ),
                    Lang::En => format!(
                        "Their midfield creates a lot ({mc:.1}): push play out wide."
                    ),
                },
            );
        }
    }
    if by_key
        .get("wing")
        .and_then(|s| s.them_val)
        .is_some_and(|w| w >= 14.5)
        && d("wing").is_some_and(|v| v <= -1.0)
    {
        push(
            "def_trans",
            0,
            lang.pick(
                "Os extremos deles são perigosos em transição: reagrupa antes de pressionar.",
                "Their wingers are dangerous in transition: regroup before pressing.",
            )
            .to_string(),
        );
    }

    let strengths: Vec<OpponentSector> = sectors
        .iter()
        .filter(|s| s.them_val.is_some_and(|v| v >= 14.0))
        .cloned()
        .collect();
    let weaknesses: Vec<OpponentSector> = sectors
        .iter()
        .filter(|s| s.them_val.is_some_and(|v| v <= 10.5))
        .cloned()
        .collect();

    let threat = |x: &XiSlotRef<'_>| -> f32 {
        let base: &[Attr] = match x.group {
            PosGroup::St | PosGroup::Amc => &[Attr::Fin, Attr::Otb, Attr::Cmp, Attr::Acc, Attr::Dri],
            PosGroup::AmLr | PosGroup::MLr => &[Attr::Dri, Attr::Acc, Attr::Pac, Attr::Cro, Attr::Fin],
            _ => &[Attr::Pas, Attr::Vis, Attr::Dec, Attr::Tec, Attr::Lon],
        };
        avg_of(x.p, base).unwrap_or(0.0)
    };

    let mut dangers: Vec<(&XiSlotRef<'_>, f32)> = them
        .iter()
        .filter(|x| {
            !matches!(
                x.group,
                PosGroup::Gk | PosGroup::Dc | PosGroup::DLr | PosGroup::WbLr
            )
        })
        .map(|x| (x, threat(x)))
        .collect();
    dangers.sort_by(|a, b| b.1.total_cmp(&a.1));
    dangers.truncate(3);

    let dangers_out: Vec<OpponentDanger> = dangers
        .into_iter()
        .map(|(x, th)| {
            let ois = match x.group {
                PosGroup::AmLr | PosGroup::MLr => vec!["Show onto weaker foot", "Tight marking"],
                PosGroup::St => vec!["Tight marking", "Tackle harder"],
                _ => vec!["Close down more", "Tight marking"],
            };
            OpponentDanger {
                player_id: x.p.id.clone(),
                player_name: x.p.name.clone(),
                code: x.code.clone(),
                threat: th,
                ois,
            }
        })
        .collect();

    let mut seen_ti = HashSet::new();
    let uplan: Vec<OpponentPlanItem> = plan
        .into_iter()
        .filter(|x| seen_ti.insert(x.ti))
        .collect();

    OpponentReport {
        them: them.iter().map(XiSlotRef::to_member).collect(),
        us: us.iter().map(XiSlotRef::to_member).collect(),
        sectors,
        plan: uplan,
        strengths,
        weaknesses,
        dangers: dangers_out,
        club: majority_club(opp_players).club,
    }
}

// ============================================================================
// Set-piece ratings (1–20 scale) & planner
// ============================================================================

#[inline]
fn sp_a(p: &impl AttrLookup, k: Attr) -> f32 {
    p.attr(k).unwrap_or(8.0)
}

pub fn sp_aerial(p: &impl AttrLookup) -> f32 {
    let h = p
        .height_cm()
        .filter(|&cm| cm > 150.0)
        .map(|cm| (cm - 183.0) / 5.0 * 0.6)
        .unwrap_or(0.0);
    (sp_a(p, Attr::Hea) * 1.3
        + sp_a(p, Attr::Jum) * 1.2
        + sp_a(p, Attr::Str) * 0.6
        + sp_a(p, Attr::Bra) * 0.5
        + sp_a(p, Attr::Ant) * 0.4)
        / 4.0
        + h
}

pub fn sp_corner(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Cor) * 1.6 + sp_a(p, Attr::Cro) + sp_a(p, Attr::Tec) * 0.5) / 3.1
}

pub fn sp_fk_direct(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Fre) * 1.7
        + sp_a(p, Attr::Tec)
        + sp_a(p, Attr::Lon) * 0.6
        + sp_a(p, Attr::Cmp) * 0.3)
        / 3.6
}

pub fn sp_fk_cross(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Fre) + sp_a(p, Attr::Cro) + sp_a(p, Attr::Tec) * 0.5) / 2.5
}

pub fn sp_penalty(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Pen) * 2.0 + sp_a(p, Attr::Cmp) + sp_a(p, Attr::Tec) * 0.3) / 3.3
}

pub fn sp_marker(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Mar) * 1.2
        + sp_a(p, Attr::Hea)
        + sp_a(p, Attr::Str) * 0.6
        + sp_a(p, Attr::Pos) * 0.6
        + sp_a(p, Attr::Jum) * 0.6
        + sp_a(p, Attr::Cnt) * 0.4)
        / 4.4
}

pub fn sp_zone(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Pos) * 1.2
        + sp_a(p, Attr::Hea)
        + sp_a(p, Attr::Jum) * 0.8
        + sp_a(p, Attr::Ant) * 0.6
        + sp_a(p, Attr::Cnt) * 0.4)
        / 4.0
}

pub fn sp_cover(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Pac)
        + sp_a(p, Attr::Acc)
        + sp_a(p, Attr::Pos) * 0.6
        + sp_a(p, Attr::Tck) * 0.5
        + sp_a(p, Attr::Ant) * 0.4)
        / 3.5
}

pub fn sp_outlet(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Pac)
        + sp_a(p, Attr::Acc)
        + sp_a(p, Attr::Otb) * 0.6
        + sp_a(p, Attr::Fir) * 0.3)
        / 2.9
}

pub fn sp_edge(p: &impl AttrLookup) -> f32 {
    (sp_a(p, Attr::Lon) * 1.2
        + sp_a(p, Attr::Tec)
        + sp_a(p, Attr::Fir) * 0.5
        + sp_a(p, Attr::Cmp) * 0.3)
        / 3.0
}

pub fn sp_captain(p: &impl AttrLookup) -> f32 {
    let age_bonus = ((p.age_years().unwrap_or(24.0) - 22.0) * 0.4).clamp(0.0, 4.0);
    (sp_a(p, Attr::Ldr) * 1.6
        + sp_a(p, Attr::Det)
        + sp_a(p, Attr::Tea) * 0.4
        + sp_a(p, Attr::Cmp) * 0.3)
        / 3.3
        + age_bonus
}

#[derive(Clone, Copy)]
enum SpKey {
    Aerial,
    Corner,
    FkDirect,
    FkCross,
    Penalty,
    Marker,
    Zone,
    Cover,
    Outlet,
    Edge,
}

struct SpEntry<'a> {
    p: &'a Player,
    code: String,
    group: PosGroup,
    aerial: f32,
    corner: f32,
    fk_direct: f32,
    fk_cross: f32,
    penalty: f32,
    marker: f32,
    zone: f32,
    cover: f32,
    outlet: f32,
    edge: f32,
}

impl SpEntry<'_> {
    fn score(&self, key: SpKey) -> f32 {
        match key {
            SpKey::Aerial => self.aerial,
            SpKey::Corner => self.corner,
            SpKey::FkDirect => self.fk_direct,
            SpKey::FkCross => self.fk_cross,
            SpKey::Penalty => self.penalty,
            SpKey::Marker => self.marker,
            SpKey::Zone => self.zone,
            SpKey::Cover => self.cover,
            SpKey::Outlet => self.outlet,
            SpKey::Edge => self.edge,
        }
    }

    fn to_taker(&self) -> SetPieceTaker {
        SetPieceTaker {
            player_id: self.p.id.clone(),
            player_name: self.p.name.clone(),
            code: self.code.clone(),
        }
    }
}

fn why_attrs(p: &Player, keys: &[Attr], lang: Lang) -> String {
    keys.iter()
        .map(|&k| {
            format!(
                "{} {}",
                attr_name(k, lang),
                p.attr(k).unwrap_or(0.0).round() as i32
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Complete set-piece & captaincy plan for the active XI.
pub fn set_piece_plan<'a>(
    t: &Tactic,
    players: &'a [Player],
    opp_list: &[Player],
    lang: Lang,
) -> Option<SetPiecePlan> {
    let xi = if players.len() >= 11 {
        my_xi_refs(t, players)
    } else {
        Vec::new()
    };
    if xi.len() < 10 {
        return None;
    }

    let out: Vec<SpEntry<'_>> = xi
        .iter()
        .filter(|x| x.group != PosGroup::Gk)
        .map(|x| SpEntry {
            p: x.p,
            code: x.code.clone(),
            group: x.group,
            aerial: sp_aerial(x.p),
            corner: sp_corner(x.p),
            fk_direct: sp_fk_direct(x.p),
            fk_cross: sp_fk_cross(x.p),
            penalty: sp_penalty(x.p),
            marker: sp_marker(x.p),
            zone: sp_zone(x.p),
            cover: sp_cover(x.p),
            outlet: sp_outlet(x.p),
            edge: sp_edge(x.p),
        })
        .collect();
    if out.is_empty() {
        return None;
    }

    let rank = |key: SpKey, filter: &dyn Fn(&SpEntry<'a>) -> bool| -> Vec<usize> {
        let mut idxs: Vec<usize> = (0..out.len()).filter(|&i| filter(&out[i])).collect();
        idxs.sort_by(|&a, &b| out[b].score(key).total_cmp(&out[a].score(key)));
        idxs
    };

    // ---- Takers
    let corner_rank = rank(SpKey::Corner, &|_| true);
    let left_idx = corner_rank
        .iter()
        .copied()
        .find(|&i| out[i].p.foot_side() == Some(Foot::L));
    let right_idx = corner_rank
        .iter()
        .copied()
        .find(|&i| out[i].p.foot_side() == Some(Foot::R));
    let best_idx = corner_rank[0];

    let inswing = || lang.pick("para dentro (inswinger)", "inswinging").to_string();
    let outswing = || lang.pick("para fora (outswinger)", "outswinging").to_string();

    let (cr_idx, cr_swing) = match left_idx {
        Some(li) if out[li].corner >= out[best_idx].corner - 1.5 => (li, Some(inswing())),
        _ => {
            let sw = match out[best_idx].p.foot_side() {
                Some(Foot::R) => Some(outswing()),
                Some(Foot::L) => Some(inswing()),
                _ => None,
            };
            (best_idx, sw)
        }
    };
    let (cl_idx, cl_swing) = match right_idx {
        Some(ri) if out[ri].corner >= out[best_idx].corner - 1.5 => (ri, Some(inswing())),
        _ => {
            let sw = match out[best_idx].p.foot_side() {
                Some(Foot::L) => Some(outswing()),
                Some(Foot::R) => Some(inswing()),
                _ => None,
            };
            (best_idx, sw)
        }
    };

    let corner_right = CornerTaker {
        player_id: out[cr_idx].p.id.clone(),
        player_name: out[cr_idx].p.name.clone(),
        code: out[cr_idx].code.clone(),
        swing: cr_swing.clone(),
    };
    let corner_left = CornerTaker {
        player_id: out[cl_idx].p.id.clone(),
        player_name: out[cl_idx].p.name.clone(),
        code: out[cl_idx].code.clone(),
        swing: cl_swing,
    };

    let fk_d_idx = rank(SpKey::FkDirect, &|_| true).first().copied();
    let fk_c_idx = rank(SpKey::FkCross, &|_| true).first().copied();
    let direct_ok = fk_d_idx.is_some_and(|i| out[i].p.attr(Attr::Fre).unwrap_or(0.0) >= 14.0);
    let fk_direct = if direct_ok {
        fk_d_idx.map(|i| out[i].to_taker())
    } else {
        None
    };
    let fk_cross = fk_c_idx.map(|i| out[i].to_taker());

    let pens: Vec<SetPieceTaker> = rank(SpKey::Penalty, &|_| true)
        .into_iter()
        .take(3)
        .map(|i| out[i].to_taker())
        .collect();

    let mut lt_candidates = rank(SpKey::FkCross, &|x| {
        x.p.attr(Attr::Lth).unwrap_or(0.0) >= 14.0
    });
    lt_candidates.sort_by(|&a, &b| {
        out[b]
            .p
            .attr(Attr::Lth)
            .unwrap_or(0.0)
            .total_cmp(&out[a].p.attr(Attr::Lth).unwrap_or(0.0))
    });
    let lt_idx = lt_candidates.first().copied();
    let long_throw = lt_idx.map(|i| out[i].to_taker());

    let mut cap_ranked: Vec<(&XiSlotRef<'_>, f32)> =
        xi.iter().map(|x| (x, sp_captain(x.p))).collect();
    cap_ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    let caps: Vec<SetPieceTaker> = cap_ranked
        .into_iter()
        .take(2)
        .map(|(x, _)| SetPieceTaker {
            player_id: x.p.id.clone(),
            player_name: x.p.name.clone(),
            code: x.code.clone(),
        })
        .collect();

    // ---- Attacking corner
    let mut used: HashSet<&str> = HashSet::new();
    let take = |used: &mut HashSet<&'a str>,
                key: SpKey,
                n: usize,
                filter: &dyn Fn(&SpEntry<'a>) -> bool|
     -> Vec<usize> {
        let mut idxs: Vec<usize> = (0..out.len())
            .filter(|&i| !used.contains(out[i].p.id.as_str()) && filter(&out[i]))
            .collect();
        idxs.sort_by(|&a, &b| out[b].score(key).total_cmp(&out[a].score(key)));
        idxs.truncate(n);
        for &i in &idxs {
            used.insert(out[i].p.id.as_str());
        }
        idxs
    };

    let taker_idx = cr_idx;
    used.insert(out[taker_idx].p.id.as_str());

    let opp_xi = if opp_list.is_empty() {
        Vec::new()
    } else {
        opponent_xi_refs(opp_list)
    };
    let opp_fast = if opp_xi.is_empty() {
        None
    } else {
        let fwd: Vec<&Player> = opp_xi
            .iter()
            .filter(|x| matches!(x.group, PosGroup::St | PosGroup::AmLr))
            .map(|x| x.p)
            .collect();
        avg_players(&fwd, &[Attr::Pac, Attr::Acc])
    };
    let n_back = if opp_fast.is_some_and(|v| v >= 14.0) {
        3
    } else {
        2
    };

    let top_air_ids: HashSet<&str> = rank(SpKey::Aerial, &|_| true)
        .into_iter()
        .take(4)
        .map(|i| out[i].p.id.as_str())
        .collect();
    let back = take(&mut used, SpKey::Cover, n_back, &|x| {
        matches!(
            x.group,
            PosGroup::Dc | PosGroup::DLr | PosGroup::WbLr | PosGroup::Dm | PosGroup::Mc
        ) && !top_air_ids.contains(x.p.id.as_str())
    });
    let box_idxs = take(&mut used, SpKey::Aerial, 4, &|_| true);
    let mut rest_box: Vec<usize> = box_idxs.iter().skip(2).copied().collect();
    rest_box.sort_by(|&a, &b| {
        out[b]
            .p
            .attr(Attr::Acc)
            .unwrap_or(0.0)
            .total_cmp(&out[a].p.attr(Attr::Acc).unwrap_or(0.0))
    });
    let far_p = box_idxs.first().copied();
    let centre = box_idxs.get(1).copied();
    let near_p = rest_box.first().copied();
    let spot = rest_box.get(1).copied();
    let edge = take(&mut used, SpKey::Edge, 1, &|_| true).first().copied();
    let short_opt = take(&mut used, SpKey::FkCross, 1, &|x| {
        matches!(
            x.group,
            PosGroup::AmLr
                | PosGroup::MLr
                | PosGroup::DLr
                | PosGroup::WbLr
                | PosGroup::Mc
                | PosGroup::Amc
        )
    })
    .first()
    .copied();

    let mk_role = |key: String, label: String, idx: usize, why: String| SetPieceRole {
        key,
        label,
        player_id: out[idx].p.id.clone(),
        player_name: out[idx].p.name.clone(),
        code: out[idx].code.clone(),
        why,
    };

    let mut attack = Vec::new();
    let taker_why = match &cr_swing {
        Some(sw) => format!("{}{sw}{}", lang.pick("canto ", ""), lang.pick("", " corner")),
        None => why_attrs(out[taker_idx].p, &[Attr::Cor, Attr::Cro], lang),
    };
    attack.push(mk_role(
        "taker".to_string(),
        lang.pick("Marcador", "Taker").to_string(),
        taker_idx,
        taker_why,
    ));
    if let Some(i) = far_p {
        attack.push(mk_role(
            "far".to_string(),
            lang.pick("2.º poste", "Far post").to_string(),
            i,
            format!(
                "{}{}",
                why_attrs(out[i].p, &[Attr::Hea, Attr::Jum], lang),
                lang.pick(
                    " — o melhor no ar ataca a bola que passa todos",
                    " — the best in the air attacks the ball that clears everyone"
                )
            ),
        ));
    }
    if let Some(i) = centre {
        attack.push(mk_role(
            "centre".to_string(),
            lang.pick("Centro da pequena área", "Centre of the six-yard box")
                .to_string(),
            i,
            why_attrs(out[i].p, &[Attr::Hea, Attr::Jum, Attr::Str], lang),
        ));
    }
    if let Some(i) = near_p {
        attack.push(mk_role(
            "near".to_string(),
            lang.pick("1.º poste", "Near post").to_string(),
            i,
            format!(
                "{}{}",
                why_attrs(out[i].p, &[Attr::Hea, Attr::Acc], lang),
                lang.pick(
                    " — antecipa-se e desvia",
                    " — gets there first and flicks it on"
                )
            ),
        ));
    }
    if let Some(i) = spot {
        attack.push(mk_role(
            "spot".to_string(),
            lang.pick(
                "Marca de penálti (segunda bola)",
                "Penalty spot (second ball)",
            )
            .to_string(),
            i,
            why_attrs(out[i].p, &[Attr::Hea, Attr::Ant], lang),
        ));
    }
    if let Some(i) = edge {
        attack.push(mk_role(
            "edge".to_string(),
            lang.pick("Entrada da área", "Edge of the box").to_string(),
            i,
            format!(
                "{}{}",
                why_attrs(out[i].p, &[Attr::Lon, Attr::Tec], lang),
                lang.pick(" — remata as sobras", " — shoots from the loose balls")
            ),
        ));
    }
    if let Some(i) = short_opt {
        attack.push(mk_role(
            "short".to_string(),
            lang.pick("Opção curta", "Short option").to_string(),
            i,
            why_attrs(out[i].p, &[Attr::Pas, Attr::Cro], lang),
        ));
    }
    for (idx, &b) in back.iter().enumerate() {
        attack.push(mk_role(
            format!("back{idx}"),
            lang.pick("Fica atrás", "Stays back").to_string(),
            b,
            format!(
                "{}{}",
                why_attrs(out[b].p, &[Attr::Pac, Attr::Acc, Attr::Pos], lang),
                lang.pick(
                    " — protege o contra-ataque",
                    " — protects against the counter"
                )
            ),
        ));
    }

    // ---- Defending corner
    used.clear();
    let outlet_n = if ti_val(t, "att_trans") == 2 { 2 } else { 1 };
    let outlet = take(&mut used, SpKey::Outlet, outlet_n, &|x| {
        matches!(
            x.group,
            PosGroup::St | PosGroup::AmLr | PosGroup::Amc | PosGroup::MLr
        )
    });

    let mut their_air_refs: Vec<(&Player, f32)> = opp_xi
        .iter()
        .filter(|x| x.group != PosGroup::Gk)
        .map(|x| (x.p, sp_aerial(x.p)))
        .collect();
    their_air_refs.sort_by(|a, b| b.1.total_cmp(&a.1));
    their_air_refs.truncate(3);

    let out_players: Vec<&Player> = out.iter().map(|x| x.p).collect();
    let our_air_avg = avg_players(&out_players, &[Attr::Hea, Attr::Jum]);
    let system = if our_air_avg.is_some_and(|v| v >= 13.5) {
        "Zonal".to_string()
    } else if their_air_refs.first().is_some_and(|&(_, a)| a >= 15.0) {
        lang.pick(
            "Misto (zonal + marcação ao mais perigoso)",
            "Mixed (zonal + marking the most dangerous)",
        )
        .to_string()
    } else {
        "Misto".to_string()
    };

    let zones = take(&mut used, SpKey::Zone, 3, &|_| true);
    let markers = take(&mut used, SpKey::Marker, their_air_refs.len().max(2), &|_| {
        true
    });
    let d_edge = take(&mut used, SpKey::Cover, 1, &|x| {
        matches!(
            x.group,
            PosGroup::Dm | PosGroup::Mc | PosGroup::Amc | PosGroup::AmLr | PosGroup::MLr
        )
    })
    .first()
    .copied();

    let mut defend = Vec::new();
    if let Some(&zi) = zones.get(1).or(zones.first()) {
        defend.push(mk_role(
            "zn".to_string(),
            lang.pick("Zona 1.º poste", "Near-post zone").to_string(),
            zi,
            lang.pick("posicionamento e jogo aéreo", "positioning and aerial ability")
                .to_string(),
        ));
    }
    if let Some(&zi) = zones.first() {
        defend.push(mk_role(
            "zc".to_string(),
            lang.pick("Zona centro (pequena área)", "Central zone (six-yard box)")
                .to_string(),
            zi,
            why_attrs(out[zi].p, &[Attr::Pos, Attr::Hea, Attr::Jum], lang),
        ));
    }
    if let Some(&zi) = zones.get(2) {
        defend.push(mk_role(
            "zf".to_string(),
            lang.pick("Zona 2.º poste", "Far-post zone").to_string(),
            zi,
            lang.pick("posicionamento e jogo aéreo", "positioning and aerial ability")
                .to_string(),
        ));
    }
    for (idx, &m) in markers.iter().enumerate() {
        let (label, extra) = if let Some(&(opp_p, _)) = their_air_refs.get(idx) {
            let sn = surname(&opp_p.name);
            (
                match lang {
                    Lang::Pt => format!("Marca {sn}"),
                    Lang::En => format!("Marks {sn}"),
                },
                match lang {
                    Lang::Pt => format!(" · {} é dos melhores no ar deles", opp_p.name),
                    Lang::En => format!(" · {} is one of their best in the air", opp_p.name),
                },
            )
        } else {
            (
                lang.pick("Marcação individual", "Man-marking").to_string(),
                String::new(),
            )
        };
        defend.push(mk_role(
            format!("m{idx}"),
            label,
            m,
            format!(
                "{}{extra}",
                why_attrs(out[m].p, &[Attr::Mar, Attr::Hea, Attr::Str], lang)
            ),
        ));
    }
    if let Some(i) = d_edge {
        defend.push(mk_role(
            "de".to_string(),
            lang.pick(
                "Entrada da área (segundas bolas)",
                "Edge of the box (second balls)",
            )
            .to_string(),
            i,
            why_attrs(out[i].p, &[Attr::Acc, Attr::Ant], lang),
        ));
    }
    for (idx, &o) in outlet.iter().enumerate() {
        defend.push(mk_role(
            format!("o{idx}"),
            lang.pick(
                "Fica na frente (contra-ataque)",
                "Stays forward (counter-attack)",
            )
            .to_string(),
            o,
            why_attrs(out[o].p, &[Attr::Pac, Attr::Acc], lang),
        ));
    }

    // ---- General tips
    let mut tips = Vec::new();
    let our_air_top: Vec<f32> = rank(SpKey::Aerial, &|_| true)
        .into_iter()
        .take(4)
        .map(|i| out[i].aerial)
        .collect();
    let air_top =
        our_air_top.iter().sum::<f32>() / (our_air_top.len().max(1) as f32);

    if air_top >= 14.0 {
        tips.push(SetPieceTip {
            ti: Some("set_pieces"),
            value: Some(2),
            text: match lang {
                Lang::Pt => format!(
                    "Tens {} jogadores fortes no ar (média {air_top:.1}): procura cantos e livres.",
                    our_air_top.len()
                ),
                Lang::En => format!(
                    "You have {} players strong in the air (average {air_top:.1}): go for corners and free kicks.",
                    our_air_top.len()
                ),
            },
        });
    } else if air_top <= 11.0 {
        tips.push(SetPieceTip {
            ti: Some("set_pieces"),
            value: Some(0),
            text: match lang {
                Lang::Pt => format!(
                    "Pouca presença aérea (média {air_top:.1}): mais vale manter a bola em jogo; cantos curtos."
                ),
                Lang::En => format!(
                    "Little aerial presence (average {air_top:.1}): better to keep the ball in play; take short corners."
                ),
            },
        });
    }
    if !direct_ok {
        let best_fk = match fk_d_idx {
            Some(i) => format!(
                "{} {}",
                out[i].p.name,
                out[i].p.attr(Attr::Fre).unwrap_or(0.0).round() as i32
            ),
            None => "—".to_string(),
        };
        tips.push(SetPieceTip {
            ti: None,
            value: None,
            text: match lang {
                Lang::Pt => format!(
                    "Ninguém com Livres ≥ 14 (melhor: {best_fk}): nos livres frontais, cruza em vez de rematar."
                ),
                Lang::En => format!(
                    "Nobody with Free Kicks ≥ 14 (best: {best_fk}): from central free kicks, cross instead of shooting."
                ),
            },
        });
    }
    if let Some(i) = lt_idx {
        let lth = out[i].p.attr(Attr::Lth).unwrap_or(0.0).round() as i32;
        tips.push(SetPieceTip {
            ti: None,
            value: None,
            text: match lang {
                Lang::Pt => format!(
                    "{} tem Lançamentos Longos {lth}: use lançamentos longos para a área no último terço.",
                    out[i].p.name
                ),
                Lang::En => format!(
                    "{} has Long Throws {lth}: use long throws into the box in the final third.",
                    out[i].p.name
                ),
            },
        });
    }
    if n_back == 3 {
        tips.push(SetPieceTip {
            ti: None,
            value: None,
            text: lang
                .pick(
                    "Os avançados deles são rápidos: deixa 3 atrás nos cantos ofensivos.",
                    "Their forwards are quick: leave 3 back on your attacking corners.",
                )
                .to_string(),
        });
    }

    let their_air: Vec<(String, f32)> = their_air_refs
        .into_iter()
        .map(|(p, a)| (p.name.clone(), a))
        .collect();

    Some(SetPiecePlan {
        xi: xi.iter().map(XiSlotRef::to_member).collect(),
        corner_right,
        corner_left,
        fk_direct,
        fk_cross,
        pens,
        long_throw,
        caps,
        attack,
        defend,
        system,
        their_air,
        tips,
        air_top,
    })
}
