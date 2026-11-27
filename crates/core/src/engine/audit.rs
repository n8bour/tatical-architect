//! 12-category tactical audit engine (IP, OOP, Coherence, Complexity, Squad fit).

use crate::data::roles::{role_by_id, valid_role};
use crate::data::tactics_data::{slot_info, ti_label, ti_val};
use crate::engine::scoring::{
    avg_attr, best_roles_for_player, complexity_of, pos_factor, role_score, slot_score,
};
use crate::types::{
    Attr, AttrLookup, AuditCat, AuditFinding, AuditReport, Lang, Phase, Player, PosGroup, Role,
    RoleTag, Severity, Side, SlotAssignment, SlotMeta, Tactic,
};
use std::collections::{BTreeMap, HashMap};

struct AuditSlot<'a> {
    s: &'a SlotAssignment,
    a: SlotMeta,
    b: SlotMeta,
    ip: Option<&'static Role>,
    oop: Option<&'static Role>,
    p: Option<&'a Player>,
}

/// Run the full 12-rule tactical audit on `t` with optional `squad` players.
pub fn audit_tactic(t: &Tactic, squad: &[Player], lang: Lang) -> AuditReport {
    let mut findings: Vec<AuditFinding> = Vec::new();
    let mut add = |cat: AuditCat, sev: Severity, title: String, text: String, fix: Option<String>| {
        findings.push(AuditFinding {
            cat,
            sev,
            title,
            text,
            fix,
        });
    };

    let pmap: HashMap<&str, &Player> = squad.iter().map(|p| (p.id.as_str(), p)).collect();
    let slots: Vec<AuditSlot<'_>> = t
        .slots
        .iter()
        .filter_map(|s| {
            let a = slot_info(&t.ip_form, &s.ip_code)?;
            let b = slot_info(&t.oop_form, &s.oop_code)?;
            Some(AuditSlot {
                s,
                a,
                b,
                ip: role_by_id(&s.ip),
                oop: role_by_id(&s.oop),
                p: s.pid.as_deref().and_then(|id| pmap.get(id).copied()),
            })
        })
        .collect();

    let ip_tag = |tag: RoleTag| -> Vec<&AuditSlot<'_>> {
        slots
            .iter()
            .filter(|x| x.ip.is_some_and(|r| r.has_tag(tag)))
            .collect()
    };
    let oop_tag = |tag: RoleTag| -> Vec<&AuditSlot<'_>> {
        slots
            .iter()
            .filter(|x| x.oop.is_some_and(|r| r.has_tag(tag)))
            .collect()
    };
    let mi = t.ment.index();
    let outfield: Vec<&AuditSlot<'_>> = slots
        .iter()
        .filter(|x| x.a.group != PosGroup::Gk)
        .collect();

    // 1. Missing / invalid roles
    for x in &slots {
        if x.ip.is_none()
            || !valid_role(&x.s.ip, x.a.group, Phase::Ip, Some(&x.a.code), x.a.wide_cb)
        {
            add(
                AuditCat::Ip,
                Severity::Crit,
                match lang {
                    Lang::Pt => format!("Papel IP inválido em {}", x.s.ip_code),
                    Lang::En => format!("Invalid in-possession role at {}", x.s.ip_code),
                },
                lang.pick(
                    "O papel não existe para esta posição.",
                    "That role does not exist for this position.",
                )
                .to_string(),
                Some(
                    lang.pick("Escolha um papel IP válido.", "Pick a valid in-possession role.")
                        .to_string(),
                ),
            );
        }
        if x.oop.is_none()
            || !valid_role(
                &x.s.oop,
                x.b.group,
                Phase::Oop,
                Some(&x.b.code),
                x.b.wide_cb,
            )
        {
            add(
                AuditCat::Oop,
                Severity::Crit,
                match lang {
                    Lang::Pt => format!("Papel OOP inválido em {}", x.s.oop_code),
                    Lang::En => format!("Invalid out-of-possession role at {}", x.s.oop_code),
                },
                lang.pick(
                    "O papel não existe para esta posição.",
                    "That role does not exist for this position.",
                )
                .to_string(),
                Some(
                    lang.pick(
                        "Escolha um papel OOP válido.",
                        "Pick a valid out-of-possession role.",
                    )
                    .to_string(),
                ),
            );
        }
    }

    // 2. Width per flank
    let wide_side = |x: &AuditSlot<'_>| -> Side {
        if x.a.side != Side::C {
            x.a.side
        } else if x.a.x < 50.0 {
            Side::L
        } else {
            Side::R
        }
    };
    for side in [Side::L, Side::R] {
        let nm_pt = if side == Side::L { "esquerdo" } else { "direito" };
        let nm_en_low = if side == Side::L { "left" } else { "right" };
        let nm_en_cap = if side == Side::L { "Left" } else { "Right" };

        let wide_count = ip_tag(RoleTag::W)
            .into_iter()
            .filter(|x| wide_side(x) == side)
            .count();
        if wide_count == 0 {
            add(
                AuditCat::Ip,
                Severity::Warn,
                match lang {
                    Lang::Pt => format!("Sem largura no flanco {nm_pt}"),
                    Lang::En => format!("No width on the {nm_en_low} flank"),
                },
                lang.pick(
                    "Nenhum papel com bola fica aberto deste lado. O adversário fecha o meio sem custo e os ataques ficam previsíveis.",
                    "No in-possession role stays wide on this side. The opponent can close the middle for free and your attacks become predictable.",
                )
                .to_string(),
                Some(
                    lang.pick(
                        "Use um Winger/Wide Forward, um Wing-Back/Advanced Wing-Back, ou um Wide Central Midfielder deste lado.",
                        "Use a Winger/Wide Forward, a Wing-Back/Advanced Wing-Back, or a Wide Central Midfielder on this side.",
                    )
                    .to_string(),
                ),
            );
        }

        let flankers: Vec<&AuditSlot<'_>> = outfield
            .iter()
            .copied()
            .filter(|x| {
                (x.a.side == side && x.a.group.is_wide_group())
                    || (x.a.wide_cb && x.a.side == side)
            })
            .collect();
        let fb = flankers
            .iter()
            .copied()
            .find(|x| matches!(x.a.group, PosGroup::DLr | PosGroup::WbLr));
        let att = flankers
            .iter()
            .copied()
            .find(|x| matches!(x.a.group, PosGroup::MLr | PosGroup::AmLr));
        if let (Some(fb), Some(att)) = (fb, att) {
            if let (Some(fb_ip), Some(att_ip)) = (fb.ip, att.ip) {
                let fb_in = fb_ip.has_tag(RoleTag::In);
                let att_in = att_ip.has_tag(RoleTag::In);
                let fb_w = fb_ip.has_tag(RoleTag::W);
                let att_w = att_ip.has_tag(RoleTag::W);
                if att_in && fb_w {
                    add(
                        AuditCat::Ip,
                        Severity::Good,
                        match lang {
                            Lang::Pt => format!("Flanco {nm_pt} complementar"),
                            Lang::En => format!("{nm_en_cap} flank is complementary"),
                        },
                        match lang {
                            Lang::Pt => format!(
                                "{} por dentro + {} por fora: largura e ameaça interior sem conflito.",
                                att_ip.name, fb_ip.name
                            ),
                            Lang::En => format!(
                                "{} inside + {} outside: width and an inside threat without clashing.",
                                att_ip.name, fb_ip.name
                            ),
                        },
                        None,
                    );
                } else if fb_in && att_w {
                    add(
                        AuditCat::Ip,
                        Severity::Good,
                        match lang {
                            Lang::Pt => format!("Flanco {nm_pt} complementar"),
                            Lang::En => format!("{nm_en_cap} flank is complementary"),
                        },
                        match lang {
                            Lang::Pt => format!(
                                "{} fecha por dentro, {} dá largura — boa rest-defence neste lado.",
                                fb_ip.name, att_ip.name
                            ),
                            Lang::En => format!(
                                "{} tucks inside, {} gives width — good rest-defence on this side.",
                                fb_ip.name, att_ip.name
                            ),
                        },
                        None,
                    );
                } else if fb_w && att_w && fb_ip.has_tag(RoleTag::Adv) && att_ip.has_tag(RoleTag::Run)
                {
                    add(
                        AuditCat::Ip,
                        Severity::Info,
                        match lang {
                            Lang::Pt => format!("Corredor {nm_pt} pode ficar congestionado"),
                            Lang::En => format!("The {nm_en_low} flank may get crowded"),
                        },
                        match lang {
                            Lang::Pt => {
                                format!("{} e {} querem ambos o corredor.", fb_ip.name, att_ip.name)
                            }
                            Lang::En => format!(
                                "{} and {} both want the same channel.",
                                fb_ip.name, att_ip.name
                            ),
                        },
                        Some(
                            lang.pick(
                                "Ponha Supporting Runs em Overlap e/ou troque o extremo por Inside Forward/Inside Winger.",
                                "Set Supporting Runs to Overlap and/or switch the winger to an Inside Forward/Inside Winger.",
                            )
                            .to_string(),
                        ),
                    );
                }
            }
        }
    }

    // 3. Rest-defence
    let rd = outfield
        .iter()
        .filter(|x| {
            x.ip.is_some_and(|r| {
                r.has_tag(RoleTag::Rd)
                    || r.has_tag(RoleTag::ToCb)
                    || (x.s.pis.iter().any(|pi| pi == "Hold position") && !r.has_tag(RoleTag::Adv))
            })
        })
        .count();
    if rd < 3 {
        add(
            AuditCat::Ip,
            if mi >= 4 { Severity::Crit } else { Severity::Warn },
            match lang {
                Lang::Pt => format!(
                    "Rest-defence frágil ({rd} jogador{})",
                    if rd == 1 { "" } else { "es" }
                ),
                Lang::En => format!(
                    "Fragile rest-defence ({rd} player{})",
                    if rd == 1 { "" } else { "s" }
                ),
            },
            lang.pick(
                "Poucos jogadores ficam atrás da linha da bola. Cada perda pode virar contra-ataque com superioridade adversária.",
                "Too few players stay behind the ball. Every turnover can become a counter-attack with the opponent outnumbering you.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Garanta 3–4: dois centrais + DM/Half Back seguros, ou um lateral como Inside Full-Back/Inside Wing-Back.",
                    "Keep 3–4 back: two centre-backs + a holding DM/Half Back, or a full-back as Inside Full-Back/Inside Wing-Back.",
                )
                .to_string(),
            ),
        );
    } else if rd >= 6 {
        add(
            AuditCat::Ip,
            Severity::Info,
            match lang {
                Lang::Pt => format!("Muitos jogadores atrás ({rd})"),
                Lang::En => format!("Many players held back ({rd})"),
            },
            lang.pick(
                "Estrutura muito conservadora com bola; pode faltar gente na frente.",
                "A very conservative shape in possession; you may lack bodies up front.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Liberte um lateral (Wing-Back) ou um médio (Box-To-Box).",
                    "Free up a full-back (Wing-Back) or a midfielder (Box-To-Box).",
                )
                .to_string(),
            ),
        );
    } else {
        add(
            AuditCat::Ip,
            Severity::Good,
            match lang {
                Lang::Pt => format!("Rest-defence equilibrada ({rd})"),
                Lang::En => format!("Balanced rest-defence ({rd})"),
            },
            lang.pick(
                "Número adequado de jogadores a proteger a transição.",
                "A suitable number of players protecting the transition.",
            )
            .to_string(),
            None,
        );
    }

    // 4. Box presence
    let box_n = ip_tag(RoleTag::Box).len();
    if box_n == 0 {
        add(
            AuditCat::Ip,
            Severity::Crit,
            lang.pick("Ninguém ataca a área", "Nobody attacks the box")
                .to_string(),
            lang.pick(
                "Nenhum papel com ameaça de golo na área.",
                "No role carries a goal threat in the box.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Avançado Centre Forward/Poacher, extremo Inside Forward, médio Attacking Midfielder/Box-To-Box.",
                    "A Centre Forward/Poacher up front, an Inside Forward out wide, an Attacking Midfielder/Box-To-Box in midfield.",
                )
                .to_string(),
            ),
        );
    } else if box_n == 1 {
        add(
            AuditCat::Ip,
            Severity::Warn,
            lang.pick(
                "Pouca presença na área (1)",
                "Little presence in the box (1)",
            )
            .to_string(),
            lang.pick(
                "Um só jogador a atacar a área é fácil de neutralizar.",
                "A single player attacking the box is easy to neutralise.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Adicione um Inside Forward, Attacking Midfielder ou Box-To-Box.",
                    "Add an Inside Forward, Attacking Midfielder or Box-To-Box.",
                )
                .to_string(),
            ),
        );
    } else {
        add(
            AuditCat::Ip,
            Severity::Good,
            match lang {
                Lang::Pt => format!("Presença na área ({box_n})"),
                Lang::En => format!("Presence in the box ({box_n})"),
            },
            lang.pick(
                "Vários jogadores a atacar a área.",
                "Several players attack the box.",
            )
            .to_string(),
            None,
        );
    }

    // 5. Depth & runs in behind
    let st: Vec<&AuditSlot<'_>> = slots.iter().filter(|x| x.a.group == PosGroup::St).collect();
    let runners = ip_tag(RoleTag::Run)
        .into_iter()
        .filter(|x| {
            x.a.group != PosGroup::St || !x.ip.is_some_and(|r| r.has_tag(RoleTag::Drop))
        })
        .count();
    if !st.is_empty()
        && st
            .iter()
            .all(|x| x.ip.is_some_and(|r| r.has_tag(RoleTag::Drop)))
    {
        let others = ip_tag(RoleTag::Run)
            .into_iter()
            .filter(|x| x.a.group != PosGroup::St)
            .count()
            + ip_tag(RoleTag::Box)
                .into_iter()
                .filter(|x| x.a.group != PosGroup::St)
                .count();
        if others < 2 {
            add(
                AuditCat::Ip,
                Severity::Warn,
                lang.pick(
                    "Avançado(s) baixam e ninguém ataca o espaço",
                    "The striker(s) drop and nobody attacks the space",
                )
                .to_string(),
                lang.pick(
                    "False Nine/Deep-Lying Forward arrasta centrais mas o espaço criado fica vazio.",
                    "The False Nine/Deep-Lying Forward drags centre-backs out, but the space created stays empty.",
                )
                .to_string(),
                Some(
                    lang.pick(
                        "Use Inside Forward, Channel Midfielder ou Second Striker para atacar a profundidade.",
                        "Use an Inside Forward, Channel Midfielder or Second Striker to attack depth.",
                    )
                    .to_string(),
                ),
            );
        }
    }
    if runners == 0 {
        add(
            AuditCat::Ip,
            Severity::Warn,
            lang.pick("Sem ataque à profundidade", "No runs in behind")
                .to_string(),
            lang.pick(
                "Nenhum papel ataca as costas da defesa — fácil para o adversário subir a linha.",
                "No role attacks the space behind the defence — easy for the opponent to push up.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Channel Forward, Poacher, Inside Forward, Wide Forward ou Advanced Wing-Back.",
                    "Channel Forward, Poacher, Inside Forward, Wide Forward or Advanced Wing-Back.",
                )
                .to_string(),
            ),
        );
    }
    if st.len() == 2 {
        if let (Some(r0), Some(r1)) = (st[0].ip, st[1].ip) {
            if r0.id == r1.id {
                add(
                    AuditCat::Ip,
                    Severity::Info,
                    lang.pick("Dupla de avançados igual", "Identical strike partnership")
                        .to_string(),
                    match lang {
                        Lang::Pt => format!("Dois {}: tendem a ocupar o mesmo espaço.", r0.name),
                        Lang::En => {
                            format!("Two {}s: they tend to occupy the same space.", r0.name)
                        }
                    },
                    Some(
                        lang.pick(
                            "Combine perfis: um que baixa (DLF/TF) e um que ataca o espaço (Channel Forward/Poacher).",
                            "Mix profiles: one who drops (DLF/TF) and one who runs in behind (Channel Forward/Poacher).",
                        )
                        .to_string(),
                    ),
                );
            }
        }
    }

    // 6. Creativity
    let cr: Vec<&AuditSlot<'_>> = ip_tag(RoleTag::Cr)
        .into_iter()
        .filter(|x| !matches!(x.a.group, PosGroup::Gk | PosGroup::Dc))
        .collect();
    if cr.is_empty() {
        add(
            AuditCat::Ip,
            Severity::Warn,
            lang.pick("Sem criador designado", "No designated creator")
                .to_string(),
            lang.pick(
                "Nenhum papel expressivo/organizador — a progressão depende só das instruções.",
                "No expressive/playmaking role — progression relies on team instructions alone.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Deep-Lying Playmaker, Midfield Playmaker, Advanced Playmaker ou Playmaking Winger.",
                    "Deep-Lying Playmaker, Midfield Playmaker, Advanced Playmaker or Playmaking Winger.",
                )
                .to_string(),
            ),
        );
    } else if cr.len() >= 4 {
        let abs = cr
            .iter()
            .filter_map(|x| x.ip.map(|r| r.ab))
            .collect::<Vec<_>>()
            .join(", ");
        add(
            AuditCat::Ip,
            Severity::Warn,
            match lang {
                Lang::Pt => format!("Demasiados criadores ({})", cr.len()),
                Lang::En => format!("Too many creators ({})", cr.len()),
            },
            match lang {
                Lang::Pt => format!("{abs} pedem todos a bola."),
                Lang::En => format!("{abs} all want the ball."),
            },
            Some(
                lang.pick(
                    "Mantenha 2–3 e troque os restantes por papéis de movimento (corredores/finalizadores).",
                    "Keep 2–3 and switch the rest to movement roles (runners/finishers).",
                )
                .to_string(),
            ),
        );
    }

    // 7. Full-backs & ACB cover
    let fbs: Vec<&AuditSlot<'_>> = slots
        .iter()
        .filter(|x| matches!(x.a.group, PosGroup::DLr | PosGroup::WbLr))
        .collect();
    let n_dc = slots.iter().filter(|x| x.a.group == PosGroup::Dc).count();
    let has_fb_cover = outfield.iter().any(|x| {
        x.ip.is_some_and(|r| {
            r.has_tag(RoleTag::ToCb) || (x.a.group == PosGroup::Dm && r.has_tag(RoleTag::Rd))
        })
    });
    if fbs.len() == 2
        && fbs
            .iter()
            .all(|x| x.ip.is_some_and(|r| r.has_tag(RoleTag::Adv)))
        && !has_fb_cover
        && n_dc < 3
    {
        add(
            AuditCat::Ip,
            Severity::Warn,
            lang.pick(
                "Os dois laterais sobem com linha de 2",
                "Both full-backs push on with only two centre-backs",
            )
            .to_string(),
            lang.pick(
                "Com dois alas ofensivos e só 2 centrais, os flancos ficam expostos em transição.",
                "With two attacking wide defenders and only 2 centre-backs, the flanks are exposed in transition.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Half Back no DM, um Inside Full-Back, ou um papel OOP Wide Covering no médio.",
                    "A Half Back at DM, an Inside Full-Back, or a Wide Covering out-of-possession role in midfield.",
                )
                .to_string(),
            ),
        );
    }
    let has_acb = outfield
        .iter()
        .any(|x| x.ip.is_some_and(|r| r.id == "acb"));
    let other_dcs = outfield
        .iter()
        .filter(|x| x.a.group == PosGroup::Dc && x.ip.is_some_and(|r| r.id != "acb"))
        .count();
    let has_tocb = outfield
        .iter()
        .any(|x| x.ip.is_some_and(|r| r.has_tag(RoleTag::ToCb)));
    if has_acb && other_dcs < 2 && !has_tocb {
        add(
            AuditCat::Ip,
            Severity::Warn,
            lang.pick(
                "Advanced Centre-Back sem cobertura",
                "Advanced Centre-Back without cover",
            )
            .to_string(),
            lang.pick(
                "O ACB sobe para o meio-campo e fica só um central atrás.",
                "The ACB steps into midfield and leaves only one centre-back behind.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Junte um Half Back ou Inside Full-Back, ou use linha de 3.",
                    "Add a Half Back or Inside Full-Back, or use a back three.",
                )
                .to_string(),
            ),
        );
    }

    // 8. OOP: outlets & pressing
    let outl = oop_tag(RoleTag::Out).len();
    if outl >= 3 {
        add(
            AuditCat::Oop,
            Severity::Crit,
            match lang {
                Lang::Pt => format!("Outlets a mais ({outl})"),
                Lang::En => format!("Too many outlets ({outl})"),
            },
            lang.pick(
                "Três ou mais jogadores não defendem: o bloco fica em inferioridade.",
                "Three or more players do not defend: the block is outnumbered.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Máximo 1–2 outlets; troque por Tracking/padrão.",
                    "At most 1–2 outlets; switch the others to Tracking/standard roles.",
                )
                .to_string(),
            ),
        );
    }
    let loe_raw = ti_val(t, "loe");
    let loe = (loe_raw * 2) as i32;
    let dl = ti_val(t, "dline") as i32;
    let trig = ti_val(t, "trigger");
    let dtr = ti_val(t, "def_trans");
    let atr = ti_val(t, "att_trans");
    let hp = oop_tag(RoleTag::Hp).len();

    if loe >= 3 && hp <= 1 {
        add(
            AuditCat::Oop,
            Severity::Warn,
            lang.pick("Pressão alta sem pressionadores", "High press without pressers")
                .to_string(),
            lang.pick(
                "Linha de pressão alta mas quase nenhum papel OOP assertivo.",
                "A high line of engagement but almost no aggressive out-of-possession roles.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Pressing CM/DM/FB, Tracking Centre Forward, Stopping CB.",
                    "Pressing CM/DM/FB, Tracking Centre Forward, Stopping CB.",
                )
                .to_string(),
            ),
        );
    }
    if loe <= 1 && hp >= 4 {
        add(
            AuditCat::Oop,
            Severity::Warn,
            lang.pick(
                "Papéis de pressão num bloco baixo",
                "Pressing roles in a low block",
            )
            .to_string(),
            lang.pick(
                "Vários papéis assertivos com linha de pressão baixa: desorganiza o bloco.",
                "Several aggressive roles with a low line of engagement: it disorganises the block.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Use Screening/Covering ou suba a Line of Engagement.",
                    "Use Screening/Covering roles or raise the Line of Engagement.",
                )
                .to_string(),
            ),
        );
    }
    if loe - dl >= 2 {
        let loe_lbl = ti_label("loe", loe_raw).unwrap_or("");
        let dl_lbl = ti_label("dline", dl as usize).unwrap_or("");
        add(
            AuditCat::Oop,
            Severity::Warn,
            lang.pick(
                "Distância entre linhas (pressão alta, defesa baixa)",
                "Gap between the lines (high press, deep defence)",
            )
            .to_string(),
            match lang {
                Lang::Pt => format!(
                    "LoE {loe_lbl} com linha {dl_lbl}: espaço enorme entre médios e defesa."
                ),
                Lang::En => format!(
                    "LoE {loe_lbl} with a {dl_lbl} line: a huge gap between midfield and defence."
                ),
            },
            Some(
                lang.pick(
                    "Aproxime as linhas (suba a defesa ou baixe a pressão).",
                    "Bring the lines closer (push the defence up or drop the press).",
                )
                .to_string(),
            ),
        );
    }
    if dl - loe >= 2 {
        let loe_lbl = ti_label("loe", loe_raw).unwrap_or("");
        let dl_lbl = ti_label("dline", dl as usize).unwrap_or("");
        add(
            AuditCat::Oop,
            Severity::Warn,
            lang.pick(
                "Linha alta sem pressão sobre a bola",
                "High line without pressure on the ball",
            )
            .to_string(),
            match lang {
                Lang::Pt => format!(
                    "Linha {dl_lbl} com LoE {loe_lbl}: o portador tem tempo para lançar nas costas."
                ),
                Lang::En => format!(
                    "A {dl_lbl} line with LoE {loe_lbl}: the ball carrier has time to play it over the top."
                ),
            },
            Some(
                lang.pick(
                    "Suba a Line of Engagement ou baixe a linha defensiva.",
                    "Raise the Line of Engagement or drop the defensive line.",
                )
                .to_string(),
            ),
        );
    }

    let gk = slots.iter().find(|x| x.a.group == PosGroup::Gk);
    if dl >= 3 && gk.is_some_and(|g| g.oop.is_some_and(|r| r.id != "swk")) {
        add(
            AuditCat::Oop,
            Severity::Warn,
            lang.pick(
                "Linha alta sem Sweeper Keeper",
                "High line without a Sweeper Keeper",
            )
            .to_string(),
            lang.pick(
                "Bolas nas costas da defesa ficam sem cobertura.",
                "Balls in behind the defence are left uncovered.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Mude o papel OOP do GR para Sweeper Keeper.",
                    "Change the goalkeeper’s out-of-possession role to Sweeper Keeper.",
                )
                .to_string(),
            ),
        );
    }
    if dl <= 1 && gk.is_some_and(|g| g.oop.is_some_and(|r| r.id == "swk")) {
        add(
            AuditCat::Oop,
            Severity::Info,
            lang.pick(
                "Sweeper Keeper com linha baixa",
                "Sweeper Keeper with a low line",
            )
            .to_string(),
            lang.pick(
                "Pouco espaço para varrer; o GR sai sem necessidade.",
                "Little space to sweep; the keeper rushes out needlessly.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Considere Goalkeeper ou Line-Holding Keeper.",
                    "Consider Goalkeeper or Line-Holding Keeper.",
                )
                .to_string(),
            ),
        );
    }
    if atr == 2 && outl == 0 && ip_tag(RoleTag::Run).is_empty() {
        add(
            AuditCat::Coherence,
            Severity::Warn,
            lang.pick("Counter-Attack sem alvos", "Counter-Attack without targets")
                .to_string(),
            lang.pick(
                "Transição de contra-ataque mas sem outlets nem corredores.",
                "A counter-attacking transition but no outlets or runners.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Adicione 1 Outlet OOP ou papéis de profundidade.",
                    "Add 1 out-of-possession outlet or runners in behind.",
                )
                .to_string(),
            ),
        );
    }
    if dtr == 2 && mi <= 1 {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Counter-Press com mentalidade defensiva",
                "Counter-Press with a defensive mentality",
            )
            .to_string(),
            lang.pick(
                "Instrução e mentalidade puxam em sentidos opostos.",
                "The instruction and the mentality pull in opposite directions.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Escolha: Regroup para bloco baixo, ou suba a mentalidade.",
                    "Choose: Regroup for a low block, or raise the mentality.",
                )
                .to_string(),
            ),
        );
    }
    if dtr == 2 && ti_val(t, "width") >= 3 {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Counter-Press com equipa muito larga",
                "Counter-Press with a very wide team",
            )
            .to_string(),
            lang.pick(
                "Jogadores longe uns dos outros na perda chegam tarde à pressão.",
                "Players spread far apart when the ball is lost arrive late to the press.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Balanced/Narrower na largura ou Regroup.",
                    "Balanced/Narrower width, or Regroup.",
                )
                .to_string(),
            ),
        );
    }
    let trap = ti_val(t, "press_trap");
    if trap == 2
        && !slots.iter().any(|x| {
            x.oop
                .is_some_and(|r| matches!(r.id, "pfb" | "prwb" | "tw" | "twm" | "swcb"))
        })
    {
        add(
            AuditCat::Oop,
            Severity::Info,
            lang.pick(
                "Pressing Trap Outside sem quem feche o corredor",
                "Pressing Trap Outside with nobody closing the channel",
            )
            .to_string(),
            lang.pick(
                "Força para fora mas ninguém está instruído a apertar na ala.",
                "It forces play wide but nobody is set to squeeze the flank.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Pressing Full-Back/Wing-Back ou Tracking Winger.",
                    "Pressing Full-Back/Wing-Back or Tracking Winger.",
                )
                .to_string(),
            ),
        );
    }

    // 9. Build-up coherence
    let bu = ti_val(t, "buildup");
    if bu == 0 {
        if gk.is_some_and(|g| g.ip.is_some_and(|r| r.id == "nngk")) {
            add(
                AuditCat::Coherence,
                Severity::Warn,
                lang.pick(
                    "Play Through Press com No-Nonsense GK",
                    "Play Through Press with a No-Nonsense GK",
                )
                .to_string(),
                lang.pick(
                    "O GR vai bater longo contra a intenção da equipa.",
                    "The keeper will go long, against the team’s intention.",
                )
                .to_string(),
                Some(
                    lang.pick(
                        "Ball-Playing Goalkeeper ou Goalkeeper.",
                        "Ball-Playing Goalkeeper or Goalkeeper.",
                    )
                    .to_string(),
                ),
            );
        }
        if slots
            .iter()
            .any(|x| x.ip.is_some_and(|r| r.id == "nncb"))
        {
            add(
                AuditCat::Coherence,
                Severity::Warn,
                lang.pick(
                    "Play Through Press com No-Nonsense CB",
                    "Play Through Press with a No-Nonsense CB",
                )
                .to_string(),
                lang.pick(
                    "Um central que alivia sempre quebra a construção curta.",
                    "A centre-back who always clears breaks the short build-up.",
                )
                .to_string(),
                Some(
                    lang.pick(
                        "Centre-Back ou Ball-Playing Centre-Back.",
                        "Centre-Back or Ball-Playing Centre-Back.",
                    )
                    .to_string(),
                ),
            );
        }
    }
    if bu == 2
        && slots
            .iter()
            .any(|x| x.ip.is_some_and(|r| matches!(r.id, "bpgk" | "dlp")))
    {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Bypass Press desperdiça construtores",
                "Bypass Press wastes your ball-players",
            )
            .to_string(),
            lang.pick(
                "BPGK/DLP têm pouco impacto se a bola salta o meio-campo.",
                "A BPGK/DLP has little impact if the ball skips midfield.",
            )
            .to_string(),
            Some(
                lang.pick("Balanced ou troque o papel.", "Balanced, or change the role.")
                    .to_string(),
            ),
        );
    }
    if ti_val(t, "goal_kicks") == 0 && gk.is_some_and(|g| g.ip.is_some_and(|r| r.id == "nngk")) {
        add(
            AuditCat::Coherence,
            Severity::Warn,
            lang.pick(
                "Goal Kicks curtos com No-Nonsense GK",
                "Short goal kicks with a No-Nonsense GK",
            )
            .to_string(),
            lang.pick("Contradição directa.", "A direct contradiction.")
                .to_string(),
            Some(
                lang.pick("Mixed/Long ou mude o papel.", "Mixed/Long, or change the role.")
                    .to_string(),
            ),
        );
    }
    let cs = ti_val(t, "crossing");
    if cs == 3 && ip_tag(RoleTag::Air).is_empty() {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Cruzamentos altos sem referência aérea",
                "High crosses without an aerial target",
            )
            .to_string(),
            lang.pick(
                "Floated crosses sem Target Forward.",
                "Floated crosses without a Target Forward.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Low Crosses, ou um Target Forward/Centre Forward forte no ar.",
                    "Low Crosses, or a Target Forward/Centre Forward who is strong in the air.",
                )
                .to_string(),
            ),
        );
    }
    if ti_val(t, "reception") == 2 && ti_val(t, "tempo") >= 4 {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Passe ao pé com ritmo muito alto",
                "Passes to feet at a very high tempo",
            )
            .to_string(),
            lang.pick(
                "Exige Primeiro Toque e Decisões muito altos.",
                "It demands very high First Touch and Decisions.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Baixe o tempo para Higher ou Balanced.",
                    "Lower the tempo to Higher or Balanced.",
                )
                .to_string(),
            ),
        );
    }
    if mi >= 5 && dl <= 1 && loe <= 1 {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Mentalidade atacante com bloco baixo",
                "Attacking mentality with a low block",
            )
            .to_string(),
            lang.pick(
                "Ataca com muitos mas defende muito atrás: equipa partida.",
                "It attacks with many but defends very deep: the team splits in two.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Suba linhas ou baixe a mentalidade.",
                    "Push the lines up or lower the mentality.",
                )
                .to_string(),
            ),
        );
    }
    if mi <= 1 && loe >= 3 {
        add(
            AuditCat::Coherence,
            Severity::Info,
            lang.pick(
                "Mentalidade defensiva com pressão alta",
                "Defensive mentality with a high press",
            )
            .to_string(),
            lang.pick("Mensagem contraditória.", "A contradictory message.")
                .to_string(),
            Some(
                lang.pick("Ajuste uma das duas.", "Adjust one of the two.")
                    .to_string(),
            ),
        );
    }

    // 10. IP <-> OOP transition distance
    for x in &slots {
        let d = (x.a.x - x.b.x).hypot(x.a.y - x.b.y);
        if d > 38.0 {
            let dr = d.round() as i32;
            add(
                AuditCat::Coherence,
                Severity::Warn,
                match lang {
                    Lang::Pt => format!("{} → {}: transição longa", x.s.ip_code, x.s.oop_code),
                    Lang::En => format!("{} → {}: long transition", x.s.ip_code, x.s.oop_code),
                },
                match lang {
                    Lang::Pt => format!(
                        "O jogador percorre uma grande distância entre a posição com e sem bola ({dr})."
                    ),
                    Lang::En => format!(
                        "The player covers a long distance between his in- and out-of-possession positions ({dr})."
                    ),
                },
                Some(
                    lang.pick(
                        "Escolha uma formação OOP mais próxima ou troque posições.",
                        "Choose a closer out-of-possession formation or swap positions.",
                    )
                    .to_string(),
                ),
            );
        }
    }

    // 11. Complexity
    let c = complexity_of(t, lang);
    let xi_players: Vec<&Player> = slots.iter().filter_map(|x| x.p).collect();
    let mut cap = 12.0_f32;
    if xi_players.len() >= 8 {
        let iq = avg_attr(
            &xi_players,
            &[
                Attr::Dec,
                Attr::Ant,
                Attr::Tea,
                Attr::Pos,
                Attr::Otb,
                Attr::Cnt,
            ],
        );
        cap = match iq {
            Some(v) if v < 10.0 => 6.0,
            Some(v) if v < 12.0 => 9.0,
            Some(v) if v < 14.0 => 12.0,
            Some(_) => 16.0,
            None => 6.0,
        };
        if let Some(iq_val) = iq.filter(|_| c.cx > cap) {
            add(
                AuditCat::Complexity,
                Severity::Warn,
                match lang {
                    Lang::Pt => {
                        format!("Tática complexa para este plantel ({} > {cap})", c.cx)
                    }
                    Lang::En => format!("Tactic too complex for this squad ({} > {cap})", c.cx),
                },
                match lang {
                    Lang::Pt => format!(
                        "Média tática do XI (Decisões/Antecipação/Trabalho de Equipa/…) ≈ {iq_val:.1}. No FM27 isto alonga a adaptação."
                    ),
                    Lang::En => format!(
                        "The XI’s tactical average (Decisions/Anticipation/Teamwork/…) ≈ {iq_val:.1}. In FM27 this lengthens adaptation."
                    ),
                },
                Some(
                    lang.pick(
                        "Remova instruções que não resolvem um problema concreto.",
                        "Remove instructions that do not solve a specific problem.",
                    )
                    .to_string(),
                ),
            );
        } else {
            add(
                AuditCat::Complexity,
                Severity::Good,
                match lang {
                    Lang::Pt => format!("Complexidade adequada ({} ≤ {cap})", c.cx),
                    Lang::En => format!("Suitable complexity ({} ≤ {cap})", c.cx),
                },
                lang.pick(
                    "O plantel deve conseguir absorver a tática.",
                    "The squad should be able to absorb the tactic.",
                )
                .to_string(),
                None,
            );
        }
    } else if c.cx > 12.0 {
        add(
            AuditCat::Complexity,
            Severity::Warn,
            match lang {
                Lang::Pt => format!("Complexidade alta ({})", c.cx),
                Lang::En => format!("High complexity ({})", c.cx),
            },
            lang.pick(
                "Muitas instruções fora do default: mais tempo de treino e jogadores mais capazes.",
                "Many non-default instructions: more training time and more capable players needed.",
            )
            .to_string(),
            Some(
                lang.pick(
                    "Comece simples; adicione instruções quando resolverem um problema identificado.",
                    "Start simple; add instructions when they solve an identified problem.",
                )
                .to_string(),
            ),
        );
    } else {
        add(
            AuditCat::Complexity,
            Severity::Good,
            match lang {
                Lang::Pt => format!("Complexidade {} ({})", c.level.to_lowercase(), c.cx),
                Lang::En => format!("{} complexity ({})", c.level, c.cx),
            },
            lang.pick("Fácil de assimilar.", "Easy to learn.")
                .to_string(),
            None,
        );
    }

    // 12. Squad checks
    if !xi_players.is_empty() {
        for x in &slots {
            let Some(p) = x.p else { continue };
            let sc = slot_score(p, t, x.s);
            let ri = x.ip.and_then(|r| role_score(p, r));
            let ro = x.oop.and_then(|r| role_score(p, r));
            if sc < 50 {
                let alt = best_roles_for_player(
                    p,
                    x.a.group,
                    Phase::Ip,
                    Some(&x.a.code),
                    x.a.wide_cb,
                    1,
                )
                .into_iter()
                .next();
                let ip_name = x.ip.map(|r| r.name).unwrap_or(&x.s.ip);
                let oop_name = x.oop.map(|r| r.name).unwrap_or(&x.s.oop);
                let ri_s = ri
                    .map(|r| r.score.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let ro_s = ro
                    .map(|r| r.score.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let out_of_pos =
                    pos_factor(p, x.a.group, x.a.side).min(pos_factor(p, x.b.group, x.b.side))
                        < 1.0;
                let suffix = if out_of_pos {
                    lang.pick(
                        " Penalizado por jogar fora da posição natural (com ou sem bola).",
                        " Penalised for playing out of his natural position (in or out of possession).",
                    )
                } else {
                    ""
                };
                let fix = match alt {
                    Some(cand) if x.ip.is_none_or(|r| cand.role_id != r.id) => {
                        let alt_name = role_by_id(cand.role_id)
                            .map(|r| r.name)
                            .unwrap_or(cand.role_id);
                        match lang {
                            Lang::Pt => {
                                format!("Melhor papel IP para ele aqui: {alt_name} ({}).", cand.score)
                            }
                            Lang::En => {
                                format!("His best in-possession role here: {alt_name} ({}).", cand.score)
                            }
                        }
                    }
                    _ => lang
                        .pick("Considere outro jogador.", "Consider another player.")
                        .to_string(),
                };
                add(
                    AuditCat::Squad,
                    Severity::Warn,
                    match lang {
                        Lang::Pt => {
                            format!("{} pouco adequado em {} ({sc})", p.name, x.s.ip_code)
                        }
                        Lang::En => {
                            format!("{} is a poor fit at {} ({sc})", p.name, x.s.ip_code)
                        }
                    },
                    format!("{ip_name}: {ri_s} · {oop_name}: {ro_s}.{suffix}"),
                    Some(fix),
                );
            }
        }

        let dcs: Vec<&Player> = slots
            .iter()
            .filter(|x| x.a.group == PosGroup::Dc)
            .filter_map(|x| x.p)
            .collect();
        if let Some(sp) = avg_attr(&dcs, &[Attr::Pac, Attr::Acc]) {
            if dl >= 3 && sp < 12.0 {
                add(
                    AuditCat::Squad,
                    Severity::Crit,
                    lang.pick(
                        "Linha alta com centrais lentos",
                        "High line with slow centre-backs",
                    )
                    .to_string(),
                    match lang {
                        Lang::Pt => format!("Velocidade/Aceleração média dos centrais ≈ {sp:.1}."),
                        Lang::En => format!("Centre-backs’ average Pace/Acceleration ≈ {sp:.1}."),
                    },
                    Some(
                        lang.pick(
                            "Baixe a linha para Standard/Lower ou mude os centrais.",
                            "Drop the line to Standard/Lower or change the centre-backs.",
                        )
                        .to_string(),
                    ),
                );
            }
        }

        if trig >= 3 || loe >= 3 || dtr == 2 {
            if let Some(ws) = avg_attr(&xi_players, &[Attr::Wor, Attr::Sta]) {
                if ws < 12.0 {
                    add(
                        AuditCat::Squad,
                        Severity::Warn,
                        lang.pick(
                            "Pressão exigente para o físico do XI",
                            "Pressing too demanding for the XI’s physique",
                        )
                        .to_string(),
                        match lang {
                            Lang::Pt => {
                                format!("Índice de Trabalho/Resistência médio ≈ {ws:.1}.")
                            }
                            Lang::En => format!("Average Work Rate/Stamina ≈ {ws:.1}."),
                        },
                        Some(
                            lang.pick(
                                "Reduza Trigger Press / LoE ou use Press Less nos Tactical Tweaks.",
                                "Reduce Trigger Press / LoE or use Press Less in Tactical Tweaks.",
                            )
                            .to_string(),
                        ),
                    );
                }
            }
        }

        if bu == 0 {
            if let Some(gk_p) = gk.and_then(|g| g.p) {
                if let Some(g) = avg_attr(&[gk_p], &[Attr::Pas, Attr::Fir, Attr::Cmp, Attr::Kic]) {
                    if g < 11.0 {
                        add(
                            AuditCat::Squad,
                            Severity::Warn,
                            lang.pick(
                                "GR fraco com os pés para construção curta",
                                "Keeper weak with his feet for a short build-up",
                            )
                            .to_string(),
                            match lang {
                                Lang::Pt => {
                                    format!("Passe/Primeiro Toque/Compostura/Pontapé ≈ {g:.1}.")
                                }
                                Lang::En => {
                                    format!("Passing/First Touch/Composure/Kicking ≈ {g:.1}.")
                                }
                            },
                            Some(
                                lang.pick(
                                    "Balanced/Bypass Press ou outro GR.",
                                    "Balanced/Bypass Press or another keeper.",
                                )
                                .to_string(),
                            ),
                        );
                    }
                }
            }
        }

        if ti_val(t, "cross_eng") == 0 {
            if let Some(h) = avg_attr(&dcs, &[Attr::Hea, Attr::Jum, Attr::Str]) {
                if h < 13.0 {
                    add(
                        AuditCat::Squad,
                        Severity::Warn,
                        lang.pick(
                            "Convida cruzamentos sem centrais fortes no ar",
                            "Inviting crosses without centre-backs strong in the air",
                        )
                        .to_string(),
                        match lang {
                            Lang::Pt => {
                                format!("Cabeceamento/Impulsão/Força dos centrais ≈ {h:.1}.")
                            }
                            Lang::En => {
                                format!("Centre-backs’ Heading/Jumping Reach/Strength ≈ {h:.1}.")
                            }
                        },
                        Some(
                            lang.pick("Balanced ou Stop Crosses.", "Balanced or Stop Crosses.")
                                .to_string(),
                        ),
                    );
                }
            }
        }

        if atr == 2 {
            let fast = xi_players
                .iter()
                .filter(|p| {
                    p.attr(Attr::Pac).unwrap_or(0.0) >= 14.0
                        || p.attr(Attr::Acc).unwrap_or(0.0) >= 14.0
                })
                .count();
            if fast < 2 {
                add(
                    AuditCat::Squad,
                    Severity::Warn,
                    lang.pick("Contra-ataque sem velocidade", "Counter-attack without pace")
                        .to_string(),
                    match lang {
                        Lang::Pt => format!("Só {fast} jogador(es) com Velocidade/Aceleração ≥ 14."),
                        Lang::En => {
                            format!("Only {fast} player(s) with Pace/Acceleration ≥ 14.")
                        }
                    },
                    Some(
                        lang.pick(
                            "Hold Shape/Balanced, ou jogadores rápidos nos outlets.",
                            "Hold Shape/Balanced, or quick players as outlets.",
                        )
                        .to_string(),
                    ),
                );
            }
        }

        if cs == 3 {
            let air = xi_players
                .iter()
                .filter(|p| {
                    p.attr(Attr::Hea).unwrap_or(0.0) >= 14.0
                        && p.attr(Attr::Jum).unwrap_or(0.0) >= 13.0
                })
                .count();
            if air < 2 {
                add(
                    AuditCat::Squad,
                    Severity::Info,
                    lang.pick(
                        "Cruzamentos altos com poucos jogadores aéreos",
                        "High crosses with few aerial players",
                    )
                    .to_string(),
                    match lang {
                        Lang::Pt => {
                            format!("{air} jogador(es) com Cabeceamento ≥ 14 e Impulsão ≥ 13.")
                        }
                        Lang::En => {
                            format!("{air} player(s) with Heading ≥ 14 and Jumping Reach ≥ 13.")
                        }
                    },
                    Some("Low Crosses.".to_string()),
                );
            }
        }
    }

    // Category scores & overall
    let mut cats = vec![
        AuditCat::Ip,
        AuditCat::Oop,
        AuditCat::Coherence,
        AuditCat::Complexity,
    ];
    if !xi_players.is_empty() {
        cats.push(AuditCat::Squad);
    }

    let penalty = |sev: Severity| -> u8 {
        match sev {
            Severity::Crit => 25,
            Severity::Warn => 10,
            Severity::Info => 3,
            Severity::Good => 0,
        }
    };

    let mut scores = BTreeMap::new();
    for &cat in &cats {
        let pen_sum: u16 = findings
            .iter()
            .filter(|f| f.cat == cat)
            .map(|f| penalty(f.sev) as u16)
            .sum();
        scores.insert(cat, 100u16.saturating_sub(pen_sum) as u8);
    }

    let overall = (cats.iter().map(|c| scores[c] as f32).sum::<f32>() / cats.len() as f32)
        .round()
        .clamp(0.0, 100.0) as u8;

    findings.sort_by_key(|f| f.sev);

    AuditReport {
        findings,
        scores,
        overall,
        complexity: c,
        cap,
    }
}
