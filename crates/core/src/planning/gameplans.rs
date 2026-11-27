//! Match-state tactical plans (Plan B / Plan C): 5 scenarios built from the user's
//! active tactic and squad, with synergy-guarded role swaps, TI/PI/Tweak changes,
//! bench substitution proposals, opposition instructions, and before/after impact diffs.
//! Ported 1:1 from `src/core/gameplans.js`.

use crate::data::{
    has_player_instruction, has_tactical_tweak, l, role_by_id, role_name, slot_info, t_name,
    ti_by_key, ti_val, valid_role,
};
use crate::engine::audit::audit_tactic;
use crate::engine::insights::opponent_report;
use crate::engine::scoring::{pos_factor, role_score};
use crate::engine::synergy::synergy_report;
use crate::types::{
    Attr, AttrLookup, GameIntent, GamePlan, GamePlanChange, GamePlanChangeKind, GamePlanOi,
    GamePlanSub, GameStateKind, Lang, Mentality, Phase, Player, PosGroup, RoleTag, SynergyPhase,
    Tactic,
};
use std::collections::{HashMap, HashSet};

pub fn scenario_meta(kind: GameStateKind, lang: Lang) -> (&'static str, &'static str) {
    match kind {
        GameStateKind::Chase => (
            l(lang, "Precisamos de golos", "We need goals"),
            l(
                lang,
                "A perder ou empatado quando só a vitória serve: mais gente na frente, mais ritmo, mais risco.",
                "Losing, or drawing when only a win will do: more bodies forward, more tempo, more risk.",
            ),
        ),
        GameStateKind::Protect => (
            l(lang, "Segurar o resultado", "Protect the lead"),
            l(
                lang,
                "A ganhar na parte final: menos risco, bloco compacto, gerir o tempo.",
                "Winning late on: less risk, a compact block, manage the clock.",
            ),
        ),
        GameStateKind::AwayStrong => (
            l(lang, "Fora contra mais fortes", "Away to a stronger side"),
            l(
                lang,
                "Prudência primeiro: bloco baixo, transições rápidas, anular os perigosos deles.",
                "Caution first: a low block, quick transitions, shut down their dangerous players.",
            ),
        ),
        GameStateKind::AwayEqual => (
            l(
                lang,
                "Fora contra equipa do mesmo nível",
                "Away to an evenly matched side",
            ),
            l(
                lang,
                "Equilíbrio: bloco médio, não dar espaço nas costas, aproveitar as transições.",
                "Balance: a mid block, no space in behind, make the most of transitions.",
            ),
        ),
        GameStateKind::HomeWeak => (
            l(
                lang,
                "Em casa contra mais fracos (bloco baixo)",
                "At home to a weaker side (low block)",
            ),
            l(
                lang,
                "O adversário fecha-se: paciência, largura, gente na área e pressão logo após a perda.",
                "The opponent sits deep: patience, width, bodies in the box and pressure straight after losing the ball.",
            ),
        ),
    }
}

/// Intensity metrics complementary to synergy: attacking commitment vs defensive security (0..=100).
pub fn game_intent(t: &Tactic) -> GameIntent {
    let mi = t.ment.index() as f64;
    let ti = |k: &str| ti_val(t, k) as f64;

    let mut box_n = 0usize;
    let mut adv_fb = 0usize;
    let mut rd = 0usize;
    let mut hold = 0usize;
    let mut outs = 0usize;

    for s in &t.slots {
        let (Some(r), Some(a)) = (role_by_id(&s.ip), slot_info(&t.ip_form, &s.ip_code)) else {
            continue;
        };
        let o = role_by_id(&s.oop);
        if r.has_tag(RoleTag::Box) || (r.has_tag(RoleTag::In) && r.has_tag(RoleTag::Run)) {
            box_n += 1;
        }
        if matches!(a.group, PosGroup::DLr | PosGroup::WbLr)
            && (r.has_tag(RoleTag::Adv)
                || r.has_tag(RoleTag::W)
                || s.pis.iter().any(|p| p == "Get further forward"))
        {
            adv_fb += 1;
        }
        if r.has_tag(RoleTag::Rd) && a.group != PosGroup::Gk {
            rd += 1;
        }
        if let Some(or) = o {
            if or.has_tag(RoleTag::Low) || or.has_tag(RoleTag::Cw) || or.has_tag(RoleTag::Trk) {
                hold += 1;
            }
            if or.has_tag(RoleTag::Out) {
                outs += 1;
            }
        }
    }

    let att = mi / 6.0 * 28.0
        + (box_n.min(5) as f64) / 5.0 * 20.0
        + (adv_fb as f64) / 2.0 * 12.0
        + ti("tempo") / 4.0 * 10.0
        + ti("width") / 4.0 * 6.0
        + ti("loe") / 2.0 * 8.0
        + ti("dline") / 4.0 * 8.0
        + if ti_val(t, "def_trans") == 2 {
            4.0
        } else {
            0.0
        }
        + if ti_val(t, "timewaste") == 0 {
            4.0
        } else {
            0.0
        };

    let def_trans_pts = match ti_val(t, "def_trans") {
        0 => 8.0,
        1 => 4.0,
        _ => 0.0,
    };
    let out_pts = match outs {
        0 => 8.0,
        1 => 6.0,
        2 => 3.0,
        _ => 0.0,
    };
    let sec = (rd.min(4) as f64) / 4.0 * 26.0
        + (1.0 - ti("dline") / 4.0) * 14.0
        + (2.0 - ti("loe")) / 2.0 * 10.0
        + def_trans_pts
        + (hold.min(5) as f64) / 5.0 * 16.0
        + out_pts
        + (6.0 - mi) / 6.0 * 12.0
        + if ti_val(t, "tackling") == 0 { 3.0 } else { 0.0 }
        + if ti_val(t, "timewaste") == 2 {
            3.0
        } else {
            0.0
        };

    GameIntent {
        att: att.min(100.0).round() as u8,
        sec: sec.min(100.0).round() as u8,
    }
}

fn attr_or_10(p: Option<&Player>, k: Attr) -> f64 {
    p.and_then(|x| x.attr(k)).map(f64::from).unwrap_or(10.0)
}

fn q_avg(p: Option<&Player>, ks: &[Attr]) -> f64 {
    ks.iter().map(|&k| attr_or_10(p, k)).sum::<f64>() / ks.len() as f64
}

fn opposite_pi(pi: &str) -> Option<&'static str> {
    match pi {
        "Hold position" => Some("Get further forward"),
        "Get further forward" => Some("Hold position"),
        "Take more risks" => Some("Take fewer risks"),
        "Take fewer risks" => Some("Take more risks"),
        "Shoot more" => Some("Shoot less"),
        "Press more" => Some("Press less"),
        "Press less" => Some("Press more"),
        "Dribble more" => Some("Dribble less"),
        _ => None,
    }
}

/// Build a match-state plan (`Chase`, `Protect`, `AwayStrong`, `AwayEqual`, `HomeWeak`) from `base`.
pub fn game_plan(
    base: &Tactic,
    players: &[Player],
    kind: GameStateKind,
    opp: &[Player],
    lang: Lang,
) -> GamePlan {
    let mut t = base.clone();
    let (title, desc) = scenario_meta(kind, lang);
    t.name = format!("{} — {title}", t_name(base, lang));
    t.source = "gameplan".to_string();

    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    let in_xi: HashSet<&str> = t.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
    let bench: Vec<&Player> = players
        .iter()
        .filter(|p| !in_xi.contains(p.id.as_str()))
        .collect();

    let mut changes: Vec<GamePlanChange> = Vec::new();
    let mut tweaks: Vec<(String, String)> = Vec::new();
    let mut subs: Vec<GamePlanSub> = Vec::new();
    let mut ois: Vec<GamePlanOi> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Snapshot initial slot metadata
    let slot_groups: Vec<PosGroup> = t
        .slots
        .iter()
        .map(|s| {
            slot_info(&t.ip_form, &s.ip_code)
                .map(|m| m.group)
                .unwrap_or(PosGroup::Mc)
        })
        .collect();
    let slot_players = |t: &Tactic, idx: usize| -> Option<&Player> {
        t.slots[idx]
            .pid
            .as_deref()
            .and_then(|id| pm.get(id).copied())
    };
    let grp_indices = |gs: &[PosGroup]| -> Vec<usize> {
        slot_groups
            .iter()
            .enumerate()
            .filter(|(_, g)| gs.contains(g))
            .map(|(i, _)| i)
            .collect()
    };
    let avg_p = |idxs: &[usize], ks: &[Attr]| -> f64 {
        if idxs.is_empty() {
            return 10.0;
        }
        idxs.iter()
            .map(|&i| q_avg(slot_players(&t, i), ks))
            .sum::<f64>()
            / idxs.len() as f64
    };

    // ---- On-pitch squad profile
    let sts = grp_indices(&[PosGroup::St]);
    let att_idx = grp_indices(&[PosGroup::St, PosGroup::Amc, PosGroup::AmLr, PosGroup::MLr]);
    let cbs = grp_indices(&[PosGroup::Dc]);
    let st_air = if sts.is_empty() {
        8.0
    } else {
        sts.iter()
            .map(|&i| q_avg(slot_players(&t, i), &[Attr::Hea, Attr::Jum, Attr::Str]))
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let att_pace = avg_p(&att_idx, &[Attr::Pac, Attr::Acc]);
    let cb_pace = avg_p(&cbs, &[Attr::Pac, Attr::Acc]);
    let cb_air = avg_p(&cbs, &[Attr::Hea, Attr::Jum, Attr::Str]);
    let tech_idx = grp_indices(&[
        PosGroup::Dc,
        PosGroup::Dm,
        PosGroup::Mc,
        PosGroup::DLr,
        PosGroup::WbLr,
    ]);
    let tech = avg_p(&tech_idx, &[Attr::Pas, Attr::Fir, Attr::Tec, Attr::Cmp]);
    let outfield_idx: Vec<usize> = slot_groups
        .iter()
        .enumerate()
        .filter(|(_, &g)| g != PosGroup::Gk)
        .map(|(i, _)| i)
        .collect();
    let press = avg_p(&outfield_idx, &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg]);
    let lon_top = {
        let mut ls: Vec<f64> = (0..t.slots.len())
            .map(|i| attr_or_10(slot_players(&t, i), Attr::Lon))
            .collect();
        ls.sort_by(|a, b| b.total_cmp(a));
        ls.iter().take(3).sum::<f64>() / 3.0
    };
    let air_top = {
        let mut av: Vec<f64> = outfield_idx
            .iter()
            .map(|&i| q_avg(slot_players(&t, i), &[Attr::Hea, Attr::Jum, Attr::Str]))
            .collect();
        av.sort_by(|a, b| b.total_cmp(a));
        av.iter().take(4).sum::<f64>() / 4.0
    };
    let wide_idx = grp_indices(&[PosGroup::DLr, PosGroup::WbLr, PosGroup::MLr, PosGroup::AmLr]);
    let best_cross = wide_idx
        .iter()
        .map(|&i| attr_or_10(slot_players(&t, i), Attr::Cro))
        .fold(0.0f64, f64::max);
    let first_line_idx = grp_indices(&[PosGroup::Gk, PosGroup::Dc]);
    let first_line = avg_p(&first_line_idx, &[Attr::Pas, Attr::Fir, Attr::Cmp]);

    // ---- Mutation helpers
    let guard_phase = match kind {
        GameStateKind::Chase | GameStateKind::HomeWeak => SynergyPhase::Att,
        GameStateKind::Protect | GameStateKind::AwayStrong | GameStateKind::AwayEqual => {
            SynergyPhase::Def
        }
    };

    let set_ti =
        |t: &mut Tactic, changes: &mut Vec<GamePlanChange>, key: &str, v: usize, why: String| {
            let Some(d) = ti_by_key(key) else {
                return;
            };
            let v = v.min(d.opts.len() - 1);
            let was = ti_val(t, key);
            if was == v {
                return;
            }
            if v == d.def {
                t.ti.remove(key);
            } else {
                t.ti.insert(key.to_string(), v);
            }
            changes.push(GamePlanChange {
                kind: GamePlanChangeKind::Ti,
                key: Some(key.to_string()),
                phase: None,
                code: None,
                player: None,
                label: d.name.to_string(),
                from: d.opts[was].to_string(),
                to: d.opts[v].to_string(),
                removed: false,
                why,
            });
        };

    let set_ment =
        |t: &mut Tactic, changes: &mut Vec<GamePlanChange>, m: Mentality, why: String| {
            if t.ment == m {
                return;
            }
            changes.push(GamePlanChange {
                kind: GamePlanChangeKind::Ment,
                key: None,
                phase: None,
                code: None,
                player: None,
                label: l(lang, "Mentalidade", "Mentality").to_string(),
                from: t.ment.as_str().to_string(),
                to: m.as_str().to_string(),
                removed: false,
                why,
            });
            t.ment = m;
        };

    let step_ment = |cur: Mentality, delta: isize, lo: Mentality, hi: Mentality| -> Mentality {
        let idx =
            (cur.index() as isize + delta).clamp(lo.index() as isize, hi.index() as isize) as usize;
        Mentality::from_index(idx).unwrap_or(cur)
    };

    // Synergy-guarded role swap: only applies if it improves guarded phase metric + 0.1 * role fit by >= 0.5
    let set_role = |t: &mut Tactic,
                    changes: &mut Vec<GamePlanChange>,
                    slot_idx: usize,
                    phase: Phase,
                    cands: &[&str],
                    why: String| {
        let code = match phase {
            Phase::Ip => t.slots[slot_idx].ip_code.clone(),
            Phase::Oop => t.slots[slot_idx].oop_code.clone(),
        };
        let form = match phase {
            Phase::Ip => t.ip_form.as_str(),
            Phase::Oop => t.oop_form.as_str(),
        };
        let Some(si) = slot_info(form, &code) else {
            return;
        };
        let ok: Vec<_> = cands
            .iter()
            .filter_map(|id| role_by_id(id))
            .filter(|r| valid_role(r.id, si.group, phase, Some(&si.code), si.wide_cb))
            .collect();
        if ok.is_empty() {
            return;
        }
        let orig = match phase {
            Phase::Ip => t.slots[slot_idx].ip.clone(),
            Phase::Oop => t.slots[slot_idx].oop.clone(),
        };
        let p = t.slots[slot_idx]
            .pid
            .as_deref()
            .and_then(|id| pm.get(id).copied());
        let fit_of = |rid: &str| -> f64 {
            match (p, role_by_id(rid)) {
                (Some(pl), Some(r)) => role_score(pl, r).map(|rf| rf.score as f64).unwrap_or(0.0),
                _ => 50.0,
            }
        };
        let metric = |tac: &Tactic| -> f64 {
            let rep = synergy_report(tac, players, lang);
            let base_sc = rep
                .phases
                .get(&guard_phase)
                .map(|x| x.score as f64)
                .unwrap_or(0.0);
            let box_bonus = if guard_phase == SynergyPhase::Att {
                rep.phases
                    .get(&SynergyPhase::Att)
                    .and_then(|x| x.parts.get("box"))
                    .copied()
                    .unwrap_or(0) as f64
                    * 0.05
            } else {
                0.0
            };
            base_sc + box_bonus
        };

        let m0 = metric(t) + fit_of(&orig) * 0.1;
        let mut best_role = None;
        let mut best_m = -1.0f64;
        for r in ok {
            match phase {
                Phase::Ip => t.slots[slot_idx].ip = r.id.to_string(),
                Phase::Oop => t.slots[slot_idx].oop = r.id.to_string(),
            }
            let m = metric(t) + fit_of(r.id) * 0.1;
            if m > best_m {
                best_m = m;
                best_role = Some(r);
            }
        }
        match phase {
            Phase::Ip => t.slots[slot_idx].ip = orig.clone(),
            Phase::Oop => t.slots[slot_idx].oop = orig.clone(),
        }
        let Some(best) = best_role else {
            return;
        };
        if best.id == orig || best_m < m0 + 0.5 {
            return;
        }
        let from_name = role_by_id(&orig)
            .map(|r| role_name(r, lang))
            .unwrap_or_default();
        changes.push(GamePlanChange {
            kind: GamePlanChangeKind::Role,
            key: None,
            phase: Some(phase),
            code: Some(code),
            player: Some(p.map(|x| x.name.clone()).unwrap_or_default()),
            label: String::new(),
            from: from_name,
            to: role_name(best, lang),
            removed: false,
            why,
        });
        match phase {
            Phase::Ip => t.slots[slot_idx].ip = best.id.to_string(),
            Phase::Oop => t.slots[slot_idx].oop = best.id.to_string(),
        }
    };

    let add_pi = |t: &mut Tactic,
                  changes: &mut Vec<GamePlanChange>,
                  slot_idx: usize,
                  pi: &str,
                  why: String| {
        let Some(p) = t.slots[slot_idx]
            .pid
            .as_deref()
            .and_then(|id| pm.get(id).copied())
        else {
            return;
        };
        if !has_player_instruction(pi) {
            return;
        }
        if let Some(opp_pi) = opposite_pi(pi) {
            t.slots[slot_idx].pis.retain(|z| z != opp_pi);
        }
        if t.slots[slot_idx].pis.iter().any(|z| z == pi) || t.slots[slot_idx].pis.len() >= 3 {
            return;
        }
        t.slots[slot_idx].pis.push(pi.to_string());
        changes.push(GamePlanChange {
            kind: GamePlanChangeKind::Pi,
            key: None,
            phase: None,
            code: Some(t.slots[slot_idx].ip_code.clone()),
            player: Some(p.name.clone()),
            label: String::new(),
            from: String::new(),
            to: pi.to_string(),
            removed: false,
            why,
        });
    };

    let drop_pi = |t: &mut Tactic, changes: &mut Vec<GamePlanChange>, slot_idx: usize, pi: &str| {
        if t.slots[slot_idx].pis.iter().any(|z| z == pi) {
            t.slots[slot_idx].pis.retain(|z| z != pi);
            let pname = t.slots[slot_idx]
                .pid
                .as_deref()
                .and_then(|id| pm.get(id))
                .map(|p| p.name.clone())
                .unwrap_or_default();
            changes.push(GamePlanChange {
                kind: GamePlanChangeKind::Pi,
                key: None,
                phase: None,
                code: Some(t.slots[slot_idx].ip_code.clone()),
                player: Some(pname),
                label: String::new(),
                from: String::new(),
                to: pi.to_string(),
                removed: true,
                why: l(
                    lang,
                    "Deixa de fazer sentido neste cenário.",
                    "No longer makes sense in this scenario.",
                )
                .to_string(),
            });
        }
    };

    let add_tweak = |tweaks: &mut Vec<(String, String)>, n: &str, why: String| {
        if has_tactical_tweak(n) {
            tweaks.push((n.to_string(), why));
        }
    };

    let attack_q = |p: &Player| {
        q_avg(
            Some(p),
            &[
                Attr::Fin,
                Attr::Otb,
                Attr::Acc,
                Attr::Pac,
                Attr::Dri,
                Attr::Cmp,
                Attr::Ant,
            ],
        )
    };
    let defend_q = |p: &Player| {
        q_avg(
            Some(p),
            &[
                Attr::Tck,
                Attr::Pos,
                Attr::Mar,
                Attr::Wor,
                Attr::Sta,
                Attr::Ant,
                Attr::Cnt,
            ],
        )
    };

    // Propose up to `max_n` distinct bench substitutions
    let propose_subs = |t: &Tactic,
                        subs: &mut Vec<GamePlanSub>,
                        score_fn: &dyn Fn(&Player, usize) -> f64,
                        ok_slot: &dyn Fn(PosGroup) -> bool,
                        max_n: usize,
                        why_fn: &dyn Fn(&Player) -> String| {
        if bench.is_empty() {
            return;
        }
        let mut opts: Vec<(usize, &Player, f64)> = Vec::new();
        for (i, s) in t.slots.iter().enumerate() {
            if !ok_slot(slot_groups[i]) {
                continue;
            }
            let Some(starter) = s.pid.as_deref().and_then(|id| pm.get(id).copied()) else {
                continue;
            };
            let Some(a) = slot_info(&t.ip_form, &s.ip_code) else {
                continue;
            };
            let starter_sc = score_fn(starter, i);
            let mut best_b: Option<(&Player, f64)> = None;
            for &b in &bench {
                let v = score_fn(b, i) * (pos_factor(b, a.group, a.side).min(1.0) as f64);
                if best_b.is_none_or(|(_, bv)| v > bv) {
                    best_b = Some((b, v));
                }
            }
            if let Some((b, bv)) = best_b {
                let gain = bv - starter_sc;
                if gain > 0.4 {
                    opts.push((i, b, gain));
                }
            }
        }
        opts.sort_by(|a, b| b.2.total_cmp(&a.2));
        let mut used: HashSet<&str> = HashSet::new();
        for (i, b, gain) in opts {
            if subs.len() >= max_n || !used.insert(b.id.as_str()) {
                continue;
            }
            let out_name = t.slots[i]
                .pid
                .as_deref()
                .and_then(|id| pm.get(id))
                .map(|p| p.name.clone())
                .unwrap_or_default();
            let role_lbl = role_by_id(&t.slots[i].ip)
                .map(|r| role_name(r, lang))
                .unwrap_or_default();
            subs.push(GamePlanSub {
                out_name,
                in_name: b.name.clone(),
                code: t.slots[i].ip_code.clone(),
                role: role_lbl,
                why: why_fn(b),
                gain: gain as f32,
            });
        }
    };

    // ---- Scenario branches
    match kind {
        GameStateKind::Chase => {
            let next_m = step_ment(t.ment, 1, Mentality::Balanced, Mentality::VeryAttacking);
            set_ment(
                &mut t,
                &mut changes,
                next_m,
                l(
                    lang,
                    "Mais jogadores entram no ataque e arriscam mais.",
                    "More players join the attack and take more risks.",
                )
                .to_string(),
            );
            let cur_tempo = ti_val(&t, "tempo");
            set_ti(
                &mut t,
                &mut changes,
                "tempo",
                cur_tempo.max(3),
                l(
                    lang,
                    "Menos tempo para o adversário se organizar.",
                    "Less time for the opponent to get organised.",
                )
                .to_string(),
            );
            let cur_width = ti_val(&t, "width");
            set_ti(
                &mut t,
                &mut changes,
                "width",
                cur_width.max(3),
                l(
                    lang,
                    "Esticar o bloco deles abre espaços por dentro.",
                    "Stretching their block opens space inside.",
                )
                .to_string(),
            );
            if st_air >= 14.0 {
                let cp = ti_val(&t, "passing");
                set_ti(
                    &mut t,
                    &mut changes,
                    "passing",
                    cp.max(3),
                    match lang {
                        Lang::Pt => format!("Referência forte no ar na frente ({st_air:.1}): a bola chega mais depressa à área."),
                        Lang::En => format!("A strong aerial target up front ({st_air:.1}): the ball reaches the box faster."),
                    },
                );
                set_ti(
                    &mut t,
                    &mut changes,
                    "crossing",
                    3,
                    match lang {
                        Lang::Pt => format!("Cruzamentos altos para a referência ({st_air:.1})."),
                        Lang::En => format!("High crosses for the target ({st_air:.1})."),
                    },
                );
                add_tweak(
                    &mut tweaks,
                    "Play Longer",
                    l(
                        lang,
                        "Bolas para a referência aérea.",
                        "Balls up to the aerial target.",
                    )
                    .to_string(),
                );
                set_ti(
                    &mut t,
                    &mut changes,
                    "goal_kicks",
                    2,
                    l(
                        lang,
                        "Pontapés de baliza para a referência aérea.",
                        "Goal kicks to the aerial target.",
                    )
                    .to_string(),
                );
            } else if att_pace >= 13.0 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "reception",
                    0,
                    match lang {
                        Lang::Pt => format!(
                            "Atacantes rápidos ({att_pace:.1}): bola para o espaço nas costas."
                        ),
                        Lang::En => {
                            format!("Quick attackers ({att_pace:.1}): balls into the space behind.")
                        }
                    },
                );
                set_ti(
                    &mut t,
                    &mut changes,
                    "crossing",
                    0,
                    l(
                        lang,
                        "Sem referência aérea: cruzamentos rasteiros.",
                        "No aerial target: low crosses.",
                    )
                    .to_string(),
                );
            }
            if lon_top >= 13.0 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "long_shots",
                    2,
                    match lang {
                        Lang::Pt => format!("Há quem remate bem de fora (top 3: {lon_top:.1})."),
                        Lang::En => {
                            format!("You have good long-range shooters (top 3: {lon_top:.1}).")
                        }
                    },
                );
            }
            if air_top >= 13.5 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "set_pieces",
                    2,
                    match lang {
                        Lang::Pt => format!("Vantagem nas bolas paradas ({air_top:.1})."),
                        Lang::En => format!("An edge at set pieces ({air_top:.1})."),
                    },
                );
            }
            set_ti(
                &mut t,
                &mut changes,
                "timewaste",
                0,
                l(lang, "Cada segundo conta.", "Every second counts.").to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "gk_speed",
                2,
                l(
                    lang,
                    "O GR repõe rápido.",
                    "The keeper distributes quickly.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "def_trans",
                2,
                l(
                    lang,
                    "Recuperar a bola logo após a perda, perto da área deles.",
                    "Win the ball back straight after losing it, near their box.",
                )
                .to_string(),
            );
            if press >= 12.0 {
                let cl = ti_val(&t, "loe");
                set_ti(
                    &mut t,
                    &mut changes,
                    "loe",
                    cl.max(2),
                    match lang {
                        Lang::Pt => format!("O XI aguenta pressão alta ({press:.1})."),
                        Lang::En => format!("The XI can sustain a high press ({press:.1})."),
                    },
                );
                let ct = ti_val(&t, "trigger");
                set_ti(
                    &mut t,
                    &mut changes,
                    "trigger",
                    ct.max(3),
                    l(
                        lang,
                        "Pressionar mais vezes para recuperar mais à frente.",
                        "Press more often to win the ball higher.",
                    )
                    .to_string(),
                );
            }
            let cd = ti_val(&t, "dline");
            set_ti(
                &mut t,
                &mut changes,
                "dline",
                cd.max(3),
                l(
                    lang,
                    "Linha subida para encurtar o campo.",
                    "A higher line to compress the pitch.",
                )
                .to_string(),
            );
            add_tweak(
                &mut tweaks,
                "More Urgency",
                l(
                    lang,
                    "Sobe ritmo e verticalidade de uma vez.",
                    "Raises tempo and verticality in one go.",
                )
                .to_string(),
            );
            if best_cross >= 14.0 {
                add_tweak(
                    &mut tweaks,
                    "Exploit Width",
                    match lang {
                        Lang::Pt => format!("Cruzadores fortes ({:.0}).", best_cross),
                        Lang::En => format!("Strong crossers ({:.0}).", best_cross),
                    },
                );
            }

            for i in grp_indices(&[PosGroup::DLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["wb_ip", "pwb", "iwb"],
                    l(
                        lang,
                        "Mais um no ataque: por fora se o extremo entra por dentro, por dentro se o extremo dá largura.",
                        "One more in attack: outside if the winger comes inside, inside if the winger gives width.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::WbLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["awb", "wb_ip"],
                    l(
                        lang,
                        "Ala projetado: quase um extremo.",
                        "Advanced wing-back: almost a winger.",
                    )
                    .to_string(),
                );
            }
            let mut mc_cands: Vec<usize> = grp_indices(&[PosGroup::Mc])
                .into_iter()
                .filter(|&i| {
                    role_by_id(&t.slots[i].ip)
                        .is_some_and(|r| !r.has_tag(RoleTag::Box) && !r.has_tag(RoleTag::Cr))
                })
                .collect();
            mc_cands.sort_by(|&a, &b| {
                let qa = slot_players(&t, a).map(attack_q).unwrap_or(10.0);
                let qb = slot_players(&t, b).map(attack_q).unwrap_or(10.0);
                qb.total_cmp(&qa)
            });
            if let Some(&mi) = mc_cands.first() {
                set_role(
                    &mut t,
                    &mut changes,
                    mi,
                    Phase::Ip,
                    &["chm", "am_ip"],
                    l(
                        lang,
                        "Mais um a chegar à área.",
                        "One more arriving in the box.",
                    )
                    .to_string(),
                );
            }
            if let Some(dmi) = grp_indices(&[PosGroup::Dm])
                .into_iter()
                .find(|&i| t.slots[i].ip == "dm_ip")
            {
                set_role(
                    &mut t,
                    &mut changes,
                    dmi,
                    Phase::Ip,
                    &["b2b", "b2bp", "dlp"],
                    l(
                        lang,
                        "O médio defensivo passa a ligar e a chegar à frente.",
                        "The defensive midfielder starts linking and getting forward.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::Amc]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["ss", "am_ip"],
                    l(
                        lang,
                        "Junta-se ao avançado na área.",
                        "Joins the striker in the box.",
                    )
                    .to_string(),
                );
            }
            for &i in &sts {
                if role_by_id(&t.slots[i].ip).is_some_and(|r| r.has_tag(RoleTag::Drop)) {
                    set_role(
                        &mut t,
                        &mut changes,
                        i,
                        Phase::Ip,
                        &["cf_ip", "poa", "chf", "tf"],
                        l(
                            lang,
                            "Precisamos dele na área, não a baixar.",
                            "We need him in the box, not dropping deep.",
                        )
                        .to_string(),
                    );
                }
            }
            let mut wingers = grp_indices(&[PosGroup::AmLr]);
            wingers.sort_by(|&a, &b| {
                q_avg(slot_players(&t, b), &[Attr::Pac, Attr::Acc])
                    .total_cmp(&q_avg(slot_players(&t, a), &[Attr::Pac, Attr::Acc]))
            });
            if let Some(&fw) = wingers.first() {
                set_role(
                    &mut t,
                    &mut changes,
                    fw,
                    Phase::Oop,
                    &["wow", "iow"],
                    l(
                        lang,
                        "Fica na frente para atacar logo que se recupera a bola.",
                        "Stays forward to attack as soon as the ball is won.",
                    )
                    .to_string(),
                );
            }
            for &i in &sts {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["cf_oop", "socf"],
                    l(
                        lang,
                        "Poupa-se a recuar: fica para finalizar.",
                        "Saves his legs: stays up to finish.",
                    )
                    .to_string(),
                );
            }
            for i in 0..t.slots.len() {
                let g = slot_groups[i];
                if g == PosGroup::Gk {
                    continue;
                }
                let Some(p) = slot_players(&t, i) else {
                    continue;
                };
                if matches!(g, PosGroup::DLr | PosGroup::WbLr | PosGroup::Dm) {
                    drop_pi(&mut t, &mut changes, i, "Hold position");
                }
                let lon = attr_or_10(Some(p), Attr::Lon);
                if lon >= 14.0 && g != PosGroup::Dc {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Shoot more",
                        match lang {
                            Lang::Pt => format!("Remates de Longe {:.0}: arriscar de fora.", lon),
                            Lang::En => format!("Long Shots {:.0}: try from distance.", lon),
                        },
                    );
                }
                let vis = attr_or_10(Some(p), Attr::Vis);
                if role_by_id(&t.slots[i].ip).is_some_and(|r| r.has_tag(RoleTag::Cr)) && vis >= 13.0
                {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Take more risks",
                        match lang {
                            Lang::Pt => format!("Visão {:.0}: tentar o passe de ruptura.", vis),
                            Lang::En => format!("Vision {:.0}: try the killer pass.", vis),
                        },
                    );
                }
                let dr = attr_or_10(Some(p), Attr::Dri);
                if matches!(g, PosGroup::AmLr | PosGroup::MLr) && dr >= 14.0 {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Dribble more",
                        match lang {
                            Lang::Pt => format!("Drible {:.0}: procurar o um-para-um.", dr),
                            Lang::En => format!("Dribbling {:.0}: take players on.", dr),
                        },
                    );
                }
                if matches!(g, PosGroup::St | PosGroup::Amc)
                    && q_avg(Some(p), &[Attr::Pac, Attr::Acc]) >= 14.0
                    && attr_or_10(Some(p), Attr::Otb) >= 13.0
                {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Attack the Defensive Line",
                        l(
                            lang,
                            "Rápido e com bom movimento: atacar a profundidade.",
                            "Quick with good movement: attack depth.",
                        )
                        .to_string(),
                    );
                }
            }
            propose_subs(
                &t,
                &mut subs,
                &|p, si| {
                    let rs = role_by_id(&t.slots[si].ip)
                        .and_then(|r| role_score(p, r))
                        .map(|rf| rf.score as f64)
                        .unwrap_or(0.0);
                    attack_q(p) + rs / 20.0
                },
                &|g| {
                    matches!(
                        g,
                        PosGroup::St
                            | PosGroup::Amc
                            | PosGroup::AmLr
                            | PosGroup::MLr
                            | PosGroup::Mc
                    )
                },
                2,
                &|b| {
                    match lang {
                    Lang::Pt => format!(
                        "{} é mais perigoso aqui (Finalização {:.0}, Sem Bola {:.0}, Aceleração {:.0}).",
                        b.name,
                        attr_or_10(Some(b), Attr::Fin),
                        attr_or_10(Some(b), Attr::Otb),
                        attr_or_10(Some(b), Attr::Acc)
                    ),
                    Lang::En => format!(
                        "{} is more dangerous here (Finishing {:.0}, Off the Ball {:.0}, Acceleration {:.0}).",
                        b.name,
                        attr_or_10(Some(b), Attr::Fin),
                        attr_or_10(Some(b), Attr::Otb),
                        attr_or_10(Some(b), Attr::Acc)
                    ),
                }
                },
            );
            if sts.len() == 1 {
                notes.push(
                    l(
                        lang,
                        "Nos últimos 10–15 minutos, se ainda faltar o golo: troque o médio mais defensivo por um segundo avançado (por exemplo, passar a duas pontas).",
                        "In the last 10–15 minutes, if the goal still has not come: swap your most defensive midfielder for a second striker (e.g. switch to two up front).",
                    )
                    .to_string(),
                );
            }
        }

        GameStateKind::Protect => {
            let next_m = step_ment(t.ment, -1, Mentality::Defensive, Mentality::Balanced);
            set_ment(
                &mut t,
                &mut changes,
                next_m,
                l(
                    lang,
                    "Menos gente no ataque, mais atrás da bola.",
                    "Fewer bodies in attack, more behind the ball.",
                )
                .to_string(),
            );
            let keep_ball = tech >= 13.0 && att_pace < 14.0;
            if keep_ball {
                let cp = ti_val(&t, "passing");
                set_ti(
                    &mut t,
                    &mut changes,
                    "passing",
                    cp.min(1),
                    match lang {
                        Lang::Pt => {
                            format!("Equipa técnica ({tech:.1}): guardar a bola é a melhor defesa.")
                        }
                        Lang::En => format!(
                            "A technical side ({tech:.1}): keeping the ball is the best defence."
                        ),
                    },
                );
                set_ti(
                    &mut t,
                    &mut changes,
                    "patience",
                    2,
                    l(
                        lang,
                        "Sem pressa: só finalizar com a jogada certa.",
                        "No rush: only shoot from the right move.",
                    )
                    .to_string(),
                );
                set_ti(
                    &mut t,
                    &mut changes,
                    "att_trans",
                    0,
                    l(
                        lang,
                        "Ao recuperar, reorganizar com a bola.",
                        "On winning the ball, reset in possession.",
                    )
                    .to_string(),
                );
            } else {
                set_ti(
                    &mut t,
                    &mut changes,
                    "att_trans",
                    if att_pace >= 13.0 { 2 } else { 0 },
                    if att_pace >= 13.0 {
                        match lang {
                            Lang::Pt => format!("Avançados rápidos ({att_pace:.1}): o golo da tranquilidade vem em contra-ataque."),
                            Lang::En => format!("Quick forwards ({att_pace:.1}): the clinching goal comes on the counter."),
                        }
                    } else {
                        l(
                            lang,
                            "Ao recuperar, reorganizar com a bola.",
                            "On winning the ball, reset in possession.",
                        )
                        .to_string()
                    },
                );
                if first_line < 11.5 {
                    set_ti(
                        &mut t,
                        &mut changes,
                        "buildup",
                        2,
                        match lang {
                            Lang::Pt => format!("Primeira linha fraca com bola ({first_line:.1}): não arriscar perto da baliza."),
                            Lang::En => format!("First line weak on the ball ({first_line:.1}): no risks near your goal."),
                        },
                    );
                }
            }
            let ct = ti_val(&t, "tempo");
            set_ti(
                &mut t,
                &mut changes,
                "tempo",
                ct.min(1),
                l(
                    lang,
                    "Ritmo baixo: o tempo corre a teu favor.",
                    "A lower tempo: the clock is on your side.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "timewaste",
                2,
                l(lang, "Gerir o relógio.", "Run down the clock.").to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "gk_speed",
                0,
                l(
                    lang,
                    "O GR abranda a reposição.",
                    "The keeper slows down distribution.",
                )
                .to_string(),
            );
            let cl = ti_val(&t, "loe");
            set_ti(
                &mut t,
                &mut changes,
                "loe",
                if cb_pace < 12.5 || press < 11.5 {
                    0
                } else {
                    cl.min(1)
                },
                l(
                    lang,
                    "Bloco compacto, sem espaço entre linhas.",
                    "A compact block, no space between the lines.",
                )
                .to_string(),
            );
            let cd = ti_val(&t, "dline");
            set_ti(
                &mut t,
                &mut changes,
                "dline",
                cd.min(if cb_pace < 12.5 { 1 } else { 2 }),
                if cb_pace < 12.5 {
                    match lang {
                        Lang::Pt => format!("Centrais pouco rápidos ({cb_pace:.1}): linha mais baixa, sem espaço nas costas."),
                        Lang::En => format!("Centre-backs short of pace ({cb_pace:.1}): a deeper line, no space in behind."),
                    }
                } else {
                    l(
                        lang,
                        "Linha normal: proteger a profundidade.",
                        "A standard line: protect depth.",
                    )
                    .to_string()
                },
            );
            let ctr = ti_val(&t, "trigger");
            set_ti(
                &mut t,
                &mut changes,
                "trigger",
                ctr.min(1),
                l(
                    lang,
                    "Pressionar menos poupa energia e evita deixar buracos.",
                    "Pressing less saves energy and avoids leaving gaps.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "def_trans",
                0,
                l(
                    lang,
                    "Ao perder, reagrupar primeiro.",
                    "On losing it, regroup first.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "tackling",
                0,
                l(
                    lang,
                    "Evitar faltas e penáltis no fim.",
                    "Avoid fouls and penalties late on.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "cross_eng",
                if cb_air >= 14.0 { 0 } else { 2 },
                if cb_air >= 14.0 {
                    match lang {
                        Lang::Pt => {
                            format!("Centrais dominantes no ar ({cb_air:.1}): deixa-os cruzar.")
                        }
                        Lang::En => format!(
                            "Centre-backs dominant in the air ({cb_air:.1}): let them cross."
                        ),
                    }
                } else {
                    match lang {
                        Lang::Pt => format!(
                            "Centrais menos fortes no ar ({cb_air:.1}): impedir os cruzamentos."
                        ),
                        Lang::En => format!(
                            "Centre-backs weaker in the air ({cb_air:.1}): stop the crosses."
                        ),
                    }
                },
            );
            set_ti(
                &mut t,
                &mut changes,
                "set_pieces",
                0,
                l(
                    lang,
                    "Manter a bola em jogo: menos bolas paradas a favor = menos contra-ataques contra.",
                    "Keep the ball in play: fewer set pieces for you = fewer counters against.",
                )
                .to_string(),
            );
            if ti_val(&t, "loe") >= 1 {
                add_tweak(
                    &mut tweaks,
                    "Press Less",
                    l(
                        lang,
                        "Baixa a intensidade da pressão de uma vez.",
                        "Lowers the pressing intensity in one go.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::DLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["hfb", "fb_oop"],
                    l(
                        lang,
                        "Segura a posição atrás.",
                        "Holds his position at the back.",
                    )
                    .to_string(),
                );
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["fb_ip", "ifb"],
                    l(
                        lang,
                        "Não sobe: fica a proteger o corredor.",
                        "Does not push on: stays to protect the flank.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::WbLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["hwb", "wb_oop"],
                    l(lang, "Fecha a linha de 5.", "Completes the back five.").to_string(),
                );
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["wb_ip", "iwb"],
                    l(lang, "Sobe menos.", "Pushes on less.").to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::Dm]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["sdm", "dm_oop"],
                    l(
                        lang,
                        "Protege a zona à frente dos centrais.",
                        "Protects the zone in front of the centre-backs.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::Mc]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["scm", "wccm", "cm_oop"],
                    l(
                        lang,
                        "Fecha os espaços centrais.",
                        "Closes the central spaces.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::AmLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["tw", "w_oop"],
                    l(
                        lang,
                        "Acompanha o lateral deles até atrás.",
                        "Tracks their full-back all the way back.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::MLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["twm", "wm_oop"],
                    l(lang, "Acompanha o recuo.", "Tracks back.").to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::Amc]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["tam", "am_oop"],
                    l(
                        lang,
                        "Recua para ajudar o meio-campo.",
                        "Drops back to help the midfield.",
                    )
                    .to_string(),
                );
            }
            let mut fast_s = sts.clone();
            fast_s.sort_by(|&a, &b| {
                q_avg(slot_players(&t, b), &[Attr::Pac, Attr::Acc])
                    .total_cmp(&q_avg(slot_players(&t, a), &[Attr::Pac, Attr::Acc]))
            });
            for (rank, &i) in fast_s.iter().enumerate() {
                let outlet = rank == 0 && att_pace >= 13.0 && !keep_ball;
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    if outlet {
                        &["socf", "cf_oop"]
                    } else {
                        &["tcf", "cf_oop"]
                    },
                    if outlet {
                        l(
                            lang,
                            "Fica como saída para o contra-ataque.",
                            "Stays up as the outlet for the counter.",
                        )
                        .to_string()
                    } else {
                        l(
                            lang,
                            "Pressiona a saída de bola deles.",
                            "Presses their build-up.",
                        )
                        .to_string()
                    },
                );
            }
            for (i, &g) in slot_groups.iter().enumerate().take(t.slots.len()) {
                let Some(p) = slot_players(&t, i) else {
                    continue;
                };
                for pi in [
                    "Get further forward",
                    "Take more risks",
                    "Shoot more",
                    "Dribble more",
                ] {
                    drop_pi(&mut t, &mut changes, i, pi);
                }
                if matches!(g, PosGroup::DLr | PosGroup::WbLr | PosGroup::Dm) {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Hold position",
                        l(
                            lang,
                            "Não deixar espaço nas costas.",
                            "Leave no space in behind.",
                        )
                        .to_string(),
                    );
                }
                let pas = attr_or_10(Some(p), Attr::Pas);
                if matches!(g, PosGroup::Dc | PosGroup::Dm) && pas <= 12.0 {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Take fewer risks",
                        match lang {
                            Lang::Pt => format!("Passe {:.0}: nada de perdas perto da área.", pas),
                            Lang::En => format!("Passing {:.0}: no turnovers near your box.", pas),
                        },
                    );
                }
            }
            propose_subs(
                &t,
                &mut subs,
                &|p, si| {
                    let rs = role_by_id(&t.slots[si].oop)
                        .and_then(|r| role_score(p, r))
                        .map(|rf| rf.score as f64)
                        .unwrap_or(0.0);
                    defend_q(p) + rs / 20.0
                },
                &|g| {
                    matches!(
                        g,
                        PosGroup::AmLr
                            | PosGroup::MLr
                            | PosGroup::Amc
                            | PosGroup::Mc
                            | PosGroup::St
                    )
                },
                2,
                &|b| {
                    match lang {
                    Lang::Pt => format!(
                        "{} defende melhor aqui (Desarme {:.0}, Posicionamento {:.0}, Índice de Trabalho {:.0}) e tem as pernas frescas.",
                        b.name,
                        attr_or_10(Some(b), Attr::Tck),
                        attr_or_10(Some(b), Attr::Pos),
                        attr_or_10(Some(b), Attr::Wor)
                    ),
                    Lang::En => format!(
                        "{} defends better here (Tackling {:.0}, Positioning {:.0}, Work Rate {:.0}) and has fresh legs.",
                        b.name,
                        attr_or_10(Some(b), Attr::Tck),
                        attr_or_10(Some(b), Attr::Pos),
                        attr_or_10(Some(b), Attr::Wor)
                    ),
                }
                },
            );
        }

        GameStateKind::AwayStrong => {
            let m = if t.ment > Mentality::Cautious {
                Mentality::Cautious
            } else {
                t.ment
            };
            set_ment(
                &mut t,
                &mut changes,
                m,
                l(
                    lang,
                    "Contra equipas mais fortes, primeiro não sofrer.",
                    "Against stronger sides, first do not concede.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "loe",
                if press >= 13.5 { 1 } else { 0 },
                if press >= 13.5 {
                    match lang {
                        Lang::Pt => format!("Bloco médio: o XI tem energia ({press:.1}) para fechar e sair."),
                        Lang::En => format!("Mid block: the XI has the energy ({press:.1}) to close down and break."),
                    }
                } else {
                    l(
                        lang,
                        "Bloco baixo: menos espaço para a qualidade deles.",
                        "Low block: less space for their quality.",
                    )
                    .to_string()
                },
            );
            set_ti(
                &mut t,
                &mut changes,
                "dline",
                if cb_pace >= 14.0 { 2 } else { 1 },
                if cb_pace >= 14.0 {
                    match lang {
                        Lang::Pt => format!("Centrais rápidos ({cb_pace:.1}): linha normal."),
                        Lang::En => format!("Quick centre-backs ({cb_pace:.1}): a standard line."),
                    }
                } else {
                    match lang {
                        Lang::Pt => format!("Linha mais baixa: os avançados deles não podem ter espaço nas costas (centrais {cb_pace:.1})."),
                        Lang::En => format!("A deeper line: their forwards must not get space in behind (centre-backs {cb_pace:.1})."),
                    }
                },
            );
            let ctr = ti_val(&t, "trigger");
            set_ti(
                &mut t,
                &mut changes,
                "trigger",
                ctr.min(1),
                l(
                    lang,
                    "Escolher os momentos de pressão.",
                    "Pick your pressing moments.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "def_trans",
                0,
                l(
                    lang,
                    "Ao perder, reagrupar: não deixar espaço para a transição deles.",
                    "On losing it, regroup: no space for their transition.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "att_trans",
                if att_pace >= 12.5 { 2 } else { 1 },
                if att_pace >= 12.5 {
                    match lang {
                        Lang::Pt => {
                            format!("Atacantes rápidos ({att_pace:.1}): o contra-ataque é a arma.")
                        }
                        Lang::En => {
                            format!("Quick attackers ({att_pace:.1}): the counter is your weapon.")
                        }
                    }
                } else {
                    l(
                        lang,
                        "Transição equilibrada: sem velocidade para contra-atacar sempre.",
                        "A balanced transition: not enough pace to always counter.",
                    )
                    .to_string()
                },
            );
            let cp = ti_val(&t, "passing");
            set_ti(
                &mut t,
                &mut changes,
                "passing",
                cp.max(3),
                l(
                    lang,
                    "Chegar à frente sem dar tempo para pressionarem.",
                    "Get forward without giving them time to press.",
                )
                .to_string(),
            );
            if att_pace >= 13.0 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "reception",
                    0,
                    l(
                        lang,
                        "Bola para o espaço nas costas deles.",
                        "Balls into the space behind them.",
                    )
                    .to_string(),
                );
            }
            if first_line < 12.5 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "buildup",
                    2,
                    match lang {
                        Lang::Pt => format!("Primeira linha pouco segura com bola ({first_line:.1}): saltar a pressão deles."),
                        Lang::En => format!("First line not secure on the ball ({first_line:.1}): bypass their press."),
                    },
                );
            }
            set_ti(
                &mut t,
                &mut changes,
                "patience",
                0,
                l(
                    lang,
                    "Cruzar cedo para aproveitar a defesa desorganizada.",
                    "Hit early crosses to catch the defence disorganised.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "cross_eng",
                if cb_air >= 14.0 { 0 } else { 2 },
                if cb_air >= 14.0 {
                    match lang {
                        Lang::Pt => format!("Centrais fortes no ar ({cb_air:.1})."),
                        Lang::En => format!("Centre-backs strong in the air ({cb_air:.1})."),
                    }
                } else {
                    match lang {
                        Lang::Pt => format!(
                            "Centrais menos fortes no ar ({cb_air:.1}): impedir cruzamentos."
                        ),
                        Lang::En => {
                            format!("Centre-backs weaker in the air ({cb_air:.1}): stop crosses.")
                        }
                    }
                },
            );
            let mut out_cands: Vec<usize> = sts
                .iter()
                .copied()
                .chain(grp_indices(&[PosGroup::AmLr]))
                .collect();
            out_cands.sort_by(|&a, &b| {
                q_avg(slot_players(&t, b), &[Attr::Pac, Attr::Acc, Attr::Otb]).total_cmp(&q_avg(
                    slot_players(&t, a),
                    &[Attr::Pac, Attr::Acc, Attr::Otb],
                ))
            });
            for (rank, &i) in out_cands.iter().enumerate() {
                let is_st = slot_groups[i] == PosGroup::St;
                if rank == 0 {
                    set_role(
                        &mut t,
                        &mut changes,
                        i,
                        Phase::Oop,
                        if is_st {
                            &["socf", "cocf"]
                        } else {
                            &["wow", "iow"]
                        },
                        l(
                            lang,
                            "A saída para o contra-ataque: o mais rápido fica na frente.",
                            "The counter-attack outlet: the quickest stays forward.",
                        )
                        .to_string(),
                    );
                } else {
                    set_role(
                        &mut t,
                        &mut changes,
                        i,
                        Phase::Oop,
                        if is_st {
                            &["tcf", "cf_oop"]
                        } else {
                            &["tw", "w_oop"]
                        },
                        l(lang, "Ajuda a defender.", "Helps to defend.").to_string(),
                    );
                }
            }
            for i in grp_indices(&[PosGroup::Dm]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["sdm", "dm_oop"],
                    l(
                        lang,
                        "Protege à frente dos centrais.",
                        "Screens in front of the centre-backs.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::DLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["hfb", "fb_oop"],
                    l(
                        lang,
                        "Segura o corredor atrás do extremo.",
                        "Holds the channel behind the winger.",
                    )
                    .to_string(),
                );
            }
            for i in grp_indices(&[PosGroup::MLr]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Oop,
                    &["twm", "wm_oop"],
                    l(lang, "Acompanha o recuo.", "Tracks back.").to_string(),
                );
            }
            if opp.len() >= 11 {
                let rep = opponent_report(&t, players, opp, lang);
                for d in rep.dangers {
                    ois.push(GamePlanOi {
                        player: d.player_name,
                        code: d.code,
                        ois: d.ois.into_iter().map(str::to_string).collect(),
                        why: l(
                            lang,
                            "Um dos três mais perigosos do adversário.",
                            "One of the opponent’s three most dangerous players.",
                        )
                        .to_string(),
                    });
                }
            } else {
                notes.push(
                    l(
                        lang,
                        "Carregue o plantel do adversário para saber a quem aplicar marcação apertada.",
                        "Load the opponent’s squad to know whom to tight-mark.",
                    )
                    .to_string(),
                );
            }
            notes.push(
                l(
                    lang,
                    "Se aos 60’ estiver empatado e precisar de ganhar, passe para “Precisamos de golos”.",
                    "If it is level at 60’ and you need the win, switch to “We need goals”.",
                )
                .to_string(),
            );
        }

        GameStateKind::AwayEqual => {
            let m = if t.ment > Mentality::Balanced {
                Mentality::Balanced
            } else if matches!(t.ment, Mentality::VeryDefensive | Mentality::Defensive) {
                Mentality::Cautious
            } else {
                t.ment
            };
            set_ment(
                &mut t,
                &mut changes,
                m,
                l(
                    lang,
                    "Fora e de igual para igual: equilíbrio.",
                    "Away and evenly matched: balance.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "loe",
                1,
                l(
                    lang,
                    "Bloco médio: nem convidar, nem expor.",
                    "Mid block: neither invite pressure nor get exposed.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "dline",
                if cb_pace >= 13.5 { 2 } else { 1 },
                l(
                    lang,
                    "Linha ajustada à velocidade dos centrais.",
                    "The line set to the centre-backs’ pace.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "def_trans",
                if press >= 13.0 { 2 } else { 1 },
                if press >= 13.0 {
                    match lang {
                        Lang::Pt => format!("Energia para contra-pressionar ({press:.1})."),
                        Lang::En => format!("Energy to counter-press ({press:.1})."),
                    }
                } else {
                    l(
                        lang,
                        "Reação equilibrada à perda.",
                        "A balanced reaction to losing the ball.",
                    )
                    .to_string()
                },
            );
            set_ti(
                &mut t,
                &mut changes,
                "att_trans",
                if att_pace >= 13.5 { 2 } else { 1 },
                if att_pace >= 13.5 {
                    match lang {
                        Lang::Pt => {
                            format!("Atacantes rápidos ({att_pace:.1}): aproveitar as transições.")
                        }
                        Lang::En => {
                            format!("Quick attackers ({att_pace:.1}): exploit transitions.")
                        }
                    }
                } else {
                    l(lang, "Transição equilibrada.", "A balanced transition.").to_string()
                },
            );
            set_ti(
                &mut t,
                &mut changes,
                "timewaste",
                1,
                l(lang, "Sem queimar tempo cedo.", "No time-wasting early on.").to_string(),
            );
            let mut fast_cands: Vec<usize> = grp_indices(&[PosGroup::AmLr])
                .into_iter()
                .chain(sts.iter().copied())
                .collect();
            fast_cands.sort_by(|&a, &b| {
                q_avg(slot_players(&t, b), &[Attr::Pac, Attr::Acc])
                    .total_cmp(&q_avg(slot_players(&t, a), &[Attr::Pac, Attr::Acc]))
            });
            if let Some(&fw) = fast_cands.first() {
                if att_pace >= 13.5 {
                    let is_st = slot_groups[fw] == PosGroup::St;
                    set_role(
                        &mut t,
                        &mut changes,
                        fw,
                        Phase::Oop,
                        if is_st {
                            &["socf", "cf_oop"]
                        } else {
                            &["wow", "w_oop"]
                        },
                        l(
                            lang,
                            "Uma saída rápida para a transição.",
                            "One quick outlet for transitions.",
                        )
                        .to_string(),
                    );
                }
            }
            notes.push(
                l(
                    lang,
                    "Mantenha o plano base e ajuste aos 60’ conforme o resultado: “Precisamos de golos” ou “Segurar o resultado”.",
                    "Keep the base plan and adjust at 60’ depending on the score: “We need goals” or “Protect the lead”.",
                )
                .to_string(),
            );
        }

        GameStateKind::HomeWeak => {
            let next_m = step_ment(t.ment, 1, Mentality::Positive, Mentality::Attacking);
            set_ment(
                &mut t,
                &mut changes,
                next_m,
                l(
                    lang,
                    "Assumir o jogo: o adversário vai fechar-se.",
                    "Take control: the opponent will sit deep.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "patience",
                2,
                l(
                    lang,
                    "Contra bloco baixo, trabalhar a bola até à melhor oportunidade.",
                    "Against a low block, work the ball until the best chance.",
                )
                .to_string(),
            );
            let cw = ti_val(&t, "width");
            set_ti(
                &mut t,
                &mut changes,
                "width",
                cw.max(3),
                l(
                    lang,
                    "Esticar o bloco para abrir espaços por dentro.",
                    "Stretch the block to open space inside.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "reception",
                2,
                l(
                    lang,
                    "Não há espaço nas costas: passe ao pé.",
                    "There is no space in behind: pass to feet.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "def_trans",
                2,
                l(
                    lang,
                    "Recuperar logo após a perda, antes que saiam.",
                    "Win it back straight after losing it, before they break.",
                )
                .to_string(),
            );
            if press >= 12.0 {
                let ctr = ti_val(&t, "trigger");
                set_ti(
                    &mut t,
                    &mut changes,
                    "trigger",
                    ctr.max(3),
                    match lang {
                        Lang::Pt => format!("Pressão constante ({press:.1})."),
                        Lang::En => format!("Constant pressure ({press:.1})."),
                    },
                );
            }
            let cd = ti_val(&t, "dline");
            set_ti(
                &mut t,
                &mut changes,
                "dline",
                cd.max(if cb_pace >= 13.0 { 3 } else { 2 }),
                l(
                    lang,
                    "Linha subida para empurrar o adversário para trás.",
                    "A higher line to push the opponent back.",
                )
                .to_string(),
            );
            set_ti(
                &mut t,
                &mut changes,
                "crossing",
                if st_air >= 14.0 { 3 } else { 0 },
                if st_air >= 14.0 {
                    match lang {
                        Lang::Pt => format!("Referência aérea ({st_air:.1}): cruzamentos altos."),
                        Lang::En => format!("Aerial target ({st_air:.1}): high crosses."),
                    }
                } else {
                    l(
                        lang,
                        "Cruzamentos rasteiros e atrasados para quem chega.",
                        "Low cut-backs for the late runners.",
                    )
                    .to_string()
                },
            );
            if lon_top >= 13.0 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "long_shots",
                    2,
                    match lang {
                        Lang::Pt => format!("Bloco baixo deixa a entrada da área livre (remate de longe top 3: {lon_top:.1})."),
                        Lang::En => format!("A low block leaves the edge of the box free (long shots top 3: {lon_top:.1})."),
                    },
                );
            }
            if air_top >= 13.5 {
                set_ti(
                    &mut t,
                    &mut changes,
                    "set_pieces",
                    2,
                    match lang {
                        Lang::Pt => {
                            format!("Bolas paradas desbloqueiam jogos fechados ({air_top:.1}).")
                        }
                        Lang::En => format!("Set pieces unlock tight games ({air_top:.1})."),
                    },
                );
            }
            set_ti(
                &mut t,
                &mut changes,
                "att_trans",
                1,
                l(
                    lang,
                    "Sem necessidade de contra-ataque: o jogo é teu.",
                    "No need to counter: the game is yours.",
                )
                .to_string(),
            );
            let mut fbs = grp_indices(&[PosGroup::DLr, PosGroup::WbLr]);
            if fbs.len() == 2 {
                fbs.sort_by(|&a, &b| {
                    q_avg(slot_players(&t, b), &[Attr::Pas, Attr::Fir, Attr::Dec]).total_cmp(
                        &q_avg(slot_players(&t, a), &[Attr::Pas, Attr::Fir, Attr::Dec]),
                    )
                });
                set_role(
                    &mut t,
                    &mut changes,
                    fbs[0],
                    Phase::Ip,
                    &["iwb", "ifb", "pwb"],
                    l(
                        lang,
                        "Entra por dentro para sobrecarregar o meio contra o bloco baixo.",
                        "Comes inside to overload the middle against the low block.",
                    )
                    .to_string(),
                );
                set_role(
                    &mut t,
                    &mut changes,
                    fbs[1],
                    Phase::Ip,
                    &["wb_ip", "awb"],
                    l(
                        lang,
                        "Dá toda a largura do outro lado.",
                        "Provides all the width on the other side.",
                    )
                    .to_string(),
                );
            }
            let has_amc = !grp_indices(&[PosGroup::Amc]).is_empty();
            for i in grp_indices(&[PosGroup::Amc]) {
                set_role(
                    &mut t,
                    &mut changes,
                    i,
                    Phase::Ip,
                    &["ap", "fr", "am_ip"],
                    l(
                        lang,
                        "Criatividade entre linhas para abrir o bloco.",
                        "Creativity between the lines to unlock the block.",
                    )
                    .to_string(),
                );
            }
            for &i in &sts {
                if role_by_id(&t.slots[i].ip).is_some_and(|r| r.has_tag(RoleTag::Drop)) && has_amc {
                    set_role(
                        &mut t,
                        &mut changes,
                        i,
                        Phase::Ip,
                        &["cf_ip", "poa"],
                        l(
                            lang,
                            "Fica na área: o espaço entre linhas já está ocupado.",
                            "Stays in the box: the space between the lines is already occupied.",
                        )
                        .to_string(),
                    );
                }
            }
            for (i, &g) in slot_groups.iter().enumerate().take(t.slots.len()) {
                let Some(p) = slot_players(&t, i) else {
                    continue;
                };
                if g == PosGroup::Gk {
                    continue;
                }
                let lon = attr_or_10(Some(p), Attr::Lon);
                if lon >= 15.0 && g != PosGroup::Dc {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Shoot more",
                        match lang {
                            Lang::Pt => format!("Remates de Longe {:.0}.", lon),
                            Lang::En => format!("Long Shots {:.0}.", lon),
                        },
                    );
                }
                let dr = attr_or_10(Some(p), Attr::Dri);
                if matches!(g, PosGroup::AmLr | PosGroup::MLr) && dr >= 14.0 {
                    add_pi(
                        &mut t,
                        &mut changes,
                        i,
                        "Dribble more",
                        match lang {
                            Lang::Pt => {
                                format!("Drible {:.0}: desequilibrar contra defesas fechadas.", dr)
                            }
                            Lang::En => format!("Dribbling {:.0}: unbalance a packed defence.", dr),
                        },
                    );
                }
            }
            add_tweak(
                &mut tweaks,
                "Exploit Width",
                l(
                    lang,
                    "Contra bloco baixo, a largura abre o meio.",
                    "Against a low block, width opens up the middle.",
                )
                .to_string(),
            );
        }
    }

    let before = synergy_report(base, players, lang);
    let after = synergy_report(&t, players, lang);
    let intent_before = game_intent(base);
    let intent_after = game_intent(&t);
    let audit_before = audit_tactic(base, players, lang);
    let audit_after = audit_tactic(&t, players, lang);

    GamePlan {
        kind,
        title: title.to_string(),
        desc: desc.to_string(),
        tactic: t,
        changes,
        tweaks,
        subs,
        ois,
        notes,
        before,
        after,
        intent_before,
        intent_after,
        audit_before,
        audit_after,
    }
}
