//! Player suitability scoring, tactic complexity, template suggestions, and training plans.

use crate::data::roles::{role_by_id, roles_for};
use crate::data::tactics_data::{
    slot_info, style_profile, ti_val, TEAM_INSTRUCTIONS, TEMPLATES,
};
use crate::engine::audit::audit_tactic;
use crate::engine::hungarian::{auto_pick_xi, tactic_from_template};
use crate::types::{
    Attr, AttrLookup, AuditCat, AuditReport, ComplexityReport, Lang, Phase, Player, PosGroup,
    Role, RoleCandidate, RoleFit, RoleTag, Severity, Side, SlotAssignment, SlotMeta, Tactic,
    TacticSuggestion, TacticSuggestionSlot, TrainingAreaPriority, TrainingIndiv, TrainingPlan,
};
use std::collections::{HashMap, HashSet};

/// Weighted role suitability score (0..=100) and attribute coverage (0.0..=1.0).
/// Key attributes carry weight 2; preferred attributes carry weight 1.
pub fn role_score(p: &impl AttrLookup, role: &Role) -> Option<RoleFit> {
    let mut sum = 0.0_f32;
    let mut weight = 0.0_f32;
    let mut have = 0_usize;
    let mut tot = 0_usize;

    for &k in role.key {
        tot += 1;
        if let Some(v) = p.attr(k) {
            sum += v * 2.0;
            weight += 2.0;
            have += 1;
        }
    }
    for &k in role.pref {
        tot += 1;
        if let Some(v) = p.attr(k) {
            sum += v;
            weight += 1.0;
            have += 1;
        }
    }
    if weight == 0.0 {
        return None;
    }
    let avg = sum / weight;
    let score = (((avg - 1.0) / 19.0) * 100.0).clamp(0.0, 100.0).round() as u8;
    let coverage = if tot > 0 {
        have as f32 / tot as f32
    } else {
        0.0
    };
    Some(RoleFit { score, coverage })
}

fn adjacent_groups(group: PosGroup) -> &'static [PosGroup] {
    match group {
        PosGroup::Dm => &[PosGroup::Mc, PosGroup::Dc],
        PosGroup::Mc => &[PosGroup::Dm, PosGroup::Amc],
        PosGroup::Amc => &[PosGroup::Mc, PosGroup::St],
        PosGroup::St => &[PosGroup::Amc],
        PosGroup::MLr => &[PosGroup::AmLr, PosGroup::WbLr],
        PosGroup::AmLr => &[PosGroup::MLr, PosGroup::St],
        PosGroup::DLr => &[PosGroup::WbLr, PosGroup::Dc],
        PosGroup::WbLr => &[PosGroup::DLr, PosGroup::MLr],
        PosGroup::Dc => &[PosGroup::Dm, PosGroup::DLr],
        PosGroup::Gk => &[],
    }
}

/// Positional familiarity multiplier (1.0 natural, 0.9 wrong flank, 0.82 adjacent, 0.6 out of position, 0.1 GK mismatch).
pub fn pos_factor(p: &impl AttrLookup, group: PosGroup, side: Side) -> f32 {
    if !p.has_any_pos() {
        return 1.0;
    }
    let is_gk = p.has_pos_group(PosGroup::Gk);
    if group == PosGroup::Gk {
        return if is_gk { 1.0 } else { 0.1 };
    }
    if p.is_only_gk() {
        return 0.1;
    }
    if p.has_pos_group(group) {
        if matches!(side, Side::L | Side::R) && group.is_wide_group() {
            return if p.has_pos_side(group, side) {
                1.0
            } else {
                0.9
            };
        }
        return 1.0;
    }
    if adjacent_groups(group)
        .iter()
        .any(|&g| p.has_pos_group(g))
    {
        return 0.82;
    }
    0.6
}

/// Preferred-foot multiplier for wide slots (inverted roles prefer opposite foot, wide roles prefer same foot).
pub fn foot_factor(p: &impl AttrLookup, slot: &SlotMeta, role: &Role) -> f32 {
    if !slot.group.is_wide_group() || !matches!(slot.side, Side::L | Side::R) {
        return 1.0;
    }
    let Some(foot) = p.foot_side() else {
        return 1.0;
    };
    let Some(foot_side) = foot.as_side() else {
        return 1.0;
    };
    let inside = role.has_tag(RoleTag::In);
    let wide = role.has_tag(RoleTag::W);
    if inside && foot_side == slot.side {
        return 0.92;
    }
    if wide && !inside && foot_side != slot.side {
        return 0.95;
    }
    1.0
}

/// Combined IP (55%) + OOP (45%) suitability score for a player in a specific tactical slot (0..=100).
pub fn slot_score(p: &impl AttrLookup, t: &Tactic, slot: &SlotAssignment) -> u8 {
    let (Some(a), Some(b)) = (
        slot_info(&t.ip_form, &slot.ip_code),
        slot_info(&t.oop_form, &slot.oop_code),
    ) else {
        return 0;
    };
    let ip_role = role_by_id(&slot.ip);
    let oop_role = role_by_id(&slot.oop);
    let ri = ip_role.and_then(|r| role_score(p, r));
    let ro = oop_role.and_then(|r| role_score(p, r));
    let base = match (ri, ro) {
        (Some(i), Some(o)) => i.score as f32 * 0.55 + o.score as f32 * 0.45,
        (Some(i), None) => i.score as f32,
        (None, Some(o)) => o.score as f32,
        (None, None) => return 0,
    };
    let pf = pos_factor(p, a.group, a.side).min(pos_factor(p, b.group, b.side));
    let ff = ip_role.map(|r| foot_factor(p, &a, r)).unwrap_or(1.0);
    (base * pf * ff).round().clamp(0.0, 100.0) as u8
}

/// Rank the top `n` roles in `group`/`phase` for a player.
pub fn best_roles_for_player(
    p: &impl AttrLookup,
    group: PosGroup,
    phase: Phase,
    code: Option<&str>,
    wide_cb: bool,
    n: usize,
) -> Vec<RoleCandidate> {
    let mut ranked: Vec<RoleCandidate> = roles_for(group, phase, code, wide_cb)
        .into_iter()
        .map(|r| RoleCandidate {
            role_id: r.id,
            score: role_score(p, r).map(|rf| rf.score).unwrap_or(0),
        })
        .collect();
    ranked.sort_by_key(|a| std::cmp::Reverse(a.score));
    ranked.truncate(n);
    ranked
}

/// Mean value of `keys` across all provided players (ignoring missing attributes).
pub fn avg_attr(players: &[&Player], keys: &[Attr]) -> Option<f32> {
    let mut sum = 0.0_f32;
    let mut count = 0_usize;
    for p in players {
        for &k in keys {
            if let Some(v) = p.attr(k) {
                sum += v;
                count += 1;
            }
        }
    }
    (count > 0).then(|| sum / count as f32)
}

/// Compute tactical complexity score, level label, and contributing parts.
pub fn complexity_of(t: &Tactic, lang: Lang) -> ComplexityReport {
    let mut cx = 0.0_f32;
    let mut parts: Vec<(String, f32)> = Vec::new();

    for d in &TEAM_INSTRUCTIONS {
        let v = ti_val(t, d.key);
        if v != d.def {
            let steps = v.abs_diff(d.def) as f32;
            let c = steps * d.cx;
            cx += c;
            let opt = d.opts.get(v).copied().unwrap_or("");
            parts.push((format!("{}: {opt}", d.name), c));
        }
    }

    for s in &t.slots {
        if let Some(r) = role_by_id(&s.ip) {
            if matches!(r.id, "fr" | "ocb" | "acb" | "f9") {
                cx += 1.0;
                parts.push((
                    format!("{}{}", lang.pick("Papel exigente: ", "Demanding role: "), r.name),
                    1.0,
                ));
            }
        }
        if !s.pis.is_empty() {
            let c = s.pis.len() as f32 * 0.5;
            cx += c;
            let label = match lang {
                Lang::Pt => format!("{} instrução(ões) individuais ({})", s.pis.len(), s.ip_code),
                Lang::En => format!("{} player instruction(s) ({})", s.pis.len(), s.ip_code),
            };
            parts.push((label, c));
        }
    }

    if t.ois > 0 {
        let c = t.ois as f32 * 0.5;
        cx += c;
        let label = match lang {
            Lang::Pt => format!("{} instruções de adversário", t.ois),
            Lang::En => format!("{} opposition instructions", t.ois),
        };
        parts.push((label, c));
    }

    let level = if cx <= 6.0 {
        lang.pick("Baixa", "Low")
    } else if cx <= 12.0 {
        lang.pick("Média", "Medium")
    } else {
        lang.pick("Alta", "High")
    }
    .to_string();

    ComplexityReport {
        cx: (cx * 10.0).round() / 10.0,
        level,
        parts,
    }
}

/// Evaluate how well the squad's top 16 players match a named style profile (0..=100).
pub fn style_fit(players: &[Player], style_id: &str) -> u8 {
    let Some(profile) = style_profile(style_id) else {
        return 50;
    };
    let mut ranked: Vec<&Player> = players.iter().collect();
    ranked.sort_by(|a, b| {
        let va = avg_attr(&[*a], &Attr::ALL).unwrap_or(0.0);
        let vb = avg_attr(&[*b], &Attr::ALL).unwrap_or(0.0);
        vb.total_cmp(&va)
    });
    ranked.truncate(16);
    match avg_attr(&ranked, profile.attrs) {
        Some(v) => (((v - 1.0) / 19.0) * 100.0).clamp(0.0, 100.0).round() as u8,
        None => 50,
    }
}

/// Rank all 8 built-in templates for a given squad (`>= 11` players).
pub fn suggest_tactics(players: &[Player], lang: Lang) -> Vec<TacticSuggestion> {
    if players.len() < 11 {
        return Vec::new();
    }
    let pmap: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();

    let mut out: Vec<TacticSuggestion> = TEMPLATES
        .iter()
        .map(|tp| {
            let mut t = tactic_from_template(tp);
            auto_pick_xi(&mut t, players);

            let xi: Vec<TacticSuggestionSlot> = t
                .slots
                .iter()
                .map(|s| {
                    let p = s.pid.as_deref().and_then(|id| pmap.get(id).copied());
                    let sc = p.map(|pl| slot_score(pl, &t, s)).unwrap_or(0);
                    TacticSuggestionSlot {
                        ip_code: s.ip_code.clone(),
                        ip_role: s.ip.clone(),
                        player_id: p.map(|pl| pl.id.clone()),
                        player_name: p.map(|pl| pl.name.clone()),
                        score: sc,
                    }
                })
                .collect();

            let xi_avg = if xi.is_empty() {
                0
            } else {
                (xi.iter().map(|x| x.score as f32).sum::<f32>() / xi.len() as f32).round() as u8
            };

            let used: HashSet<&str> = t.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
            let bench: Vec<&Player> = players
                .iter()
                .filter(|p| !used.contains(p.id.as_str()))
                .collect();
            let depth = if t.slots.is_empty() {
                0
            } else {
                let sum: f32 = t
                    .slots
                    .iter()
                    .map(|s| {
                        bench
                            .iter()
                            .map(|p| slot_score(*p, &t, s))
                            .max()
                            .unwrap_or(0) as f32
                    })
                    .sum();
                (sum / t.slots.len() as f32).round() as u8
            };

            let fit = style_fit(players, tp.style);
            let audit = audit_tactic(&t, players, lang);
            let total = (xi_avg as f32 * 0.55
                + fit as f32 * 0.20
                + depth as f32 * 0.15
                + audit.overall as f32 * 0.10)
                .round()
                .clamp(0.0, 100.0) as u8;

            let mut weakest = xi.clone();
            weakest.sort_by_key(|x| x.score);
            weakest.truncate(2);

            let mut strongest = xi;
            strongest.sort_by_key(|a| std::cmp::Reverse(a.score));
            strongest.truncate(3);

            TacticSuggestion {
                template_id: tp.id,
                tactic: t,
                xi_avg,
                depth,
                fit,
                audit,
                total,
                weakest,
                strongest,
            }
        })
        .collect();

    out.sort_by_key(|a| std::cmp::Reverse(a.total));
    out
}

/// Build a training priority plan (team tactical understanding areas, individual focuses, mentoring, cohesion).
pub fn training_plan(
    t: &Tactic,
    players: &[Player],
    audit: &AuditReport,
    lang: Lang,
) -> TrainingPlan {
    let mut areas: Vec<TrainingAreaPriority> = Vec::new();
    let mut push = |key: &'static str, why: String, weight: usize| {
        if let Some(existing) = areas.iter_mut().find(|a| a.key == key) {
            existing.weight += weight;
            existing.why.push(why);
        } else {
            areas.push(TrainingAreaPriority {
                key,
                weight,
                why: vec![why],
            });
        }
    };

    push(
        "ip_shape",
        lang.pick(
            "Base de qualquer tática (formação IP).",
            "The base of any tactic (in-possession shape).",
        )
        .to_string(),
        2,
    );
    push(
        "oop_shape",
        lang.pick(
            "Base de qualquer tática (formação OOP).",
            "The base of any tactic (out-of-possession shape).",
        )
        .to_string(),
        2,
    );

    let v = |k: &str| ti_val(t, k);
    let d = |k: &str| {
        crate::data::tactics_data::ti_by_key(k)
            .map(|x| x.def)
            .unwrap_or(0)
    };

    if v("passing") != d("passing") || v("reception") != d("reception") || v("tempo") != d("tempo")
    {
        push(
            "passing",
            lang.pick(
                "Estilo de passe fora do default.",
                "Passing style set away from the default.",
            )
            .to_string(),
            2,
        );
    }
    if v("buildup") != d("buildup")
        || v("goal_kicks") != d("goal_kicks")
        || v("gk_target") != d("gk_target")
    {
        push(
            "buildup",
            lang.pick(
                "Instruções de construção definidas.",
                "Build-up instructions set.",
            )
            .to_string(),
            2,
        );
    }
    if v("patience") != d("patience")
        || v("crossing") != d("crossing")
        || v("long_shots") != d("long_shots")
        || v("dribbling") != d("dribbling")
    {
        push(
            "chance",
            lang.pick(
                "Instruções de último terço definidas.",
                "Final-third instructions set.",
            )
            .to_string(),
            2,
        );
    }
    if v("trigger") != d("trigger") {
        push(
            "trigger",
            lang.pick(
                "Trigger Press fora do default.",
                "Trigger Press set away from the default.",
            )
            .to_string(),
            2 + v("trigger").abs_diff(d("trigger")),
        );
    }
    if v("press_trap") != d("press_trap") {
        push(
            "trap",
            lang.pick("Pressing Trap definida.", "Pressing Trap set.")
                .to_string(),
            2,
        );
    }
    if v("loe") != d("loe") {
        push(
            "loe",
            lang.pick(
                "Line of Engagement fora do default.",
                "Line of Engagement set away from the default.",
            )
            .to_string(),
            1 + v("loe").abs_diff(d("loe")),
        );
    }

    for f in audit
        .findings
        .iter()
        .filter(|f| matches!(f.sev, Severity::Crit | Severity::Warn))
    {
        let msg = format!(
            "{}{}",
            lang.pick("Problema detectado: ", "Problem found: "),
            f.title
        );
        if f.cat == AuditCat::Oop {
            push("oop_shape", msg, 1);
        } else if f.cat == AuditCat::Ip {
            push("ip_shape", msg, 1);
        }
    }

    areas.sort_by_key(|a| std::cmp::Reverse(a.weight));

    let pmap: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    let mut indiv = Vec::new();
    for s in &t.slots {
        let Some(p) = s.pid.as_deref().and_then(|id| pmap.get(id).copied()) else {
            continue;
        };
        let (Some(ri), Some(ro)) = (role_by_id(&s.ip), role_by_id(&s.oop)) else {
            continue;
        };
        let si = role_score(p, ri).map(|rf| rf.score).unwrap_or(0);
        let so = role_score(p, ro).map(|rf| rf.score).unwrap_or(0);
        let focus_role = if si <= so { ri } else { ro };

        let mut seen_attr = HashSet::new();
        let mut keys: Vec<(Attr, f32)> = ri
            .key
            .iter()
            .chain(ro.key.iter())
            .copied()
            .filter(|k| seen_attr.insert(*k))
            .filter_map(|k| p.attr(k).map(|val| (k, val)))
            .collect();
        keys.sort_by(|a, b| a.1.total_cmp(&b.1));
        keys.truncate(2);
        let weak = keys
            .into_iter()
            .map(|(k, val)| (k, val.round().clamp(0.0, 20.0) as u8))
            .collect();

        indiv.push(TrainingIndiv {
            player_id: p.id.clone(),
            player_name: p.name.clone(),
            slot: s.ip_code.clone(),
            role_id: focus_role.id,
            si,
            so,
            weak,
        });
    }

    let mentors: Vec<String> = players
        .iter()
        .filter(|p| p.attr(Attr::Det).unwrap_or(0.0) >= 15.0 && p.age.unwrap_or(0.0) >= 26.0)
        .map(|p| p.name.clone())
        .collect();

    let youth: Vec<String> = players
        .iter()
        .filter(|p| p.age.is_some_and(|age| age <= 21.0))
        .map(|p| p.name.clone())
        .collect();

    let cap = if audit.cap > 0.0 { audit.cap } else { 12.0 };
    let cohesion = audit.complexity.cx > cap;

    TrainingPlan {
        areas,
        indiv,
        mentors,
        youth,
        cohesion,
    }
}
