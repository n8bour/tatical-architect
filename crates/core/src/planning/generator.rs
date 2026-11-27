//! Tactic generator: evaluates all valid IP×OOP formation pairs, runs 2-pass
//! coordinate descent alternating Hungarian XI assignment with greedy per-slot
//! IP/OOP role search (55% suitability / 30% synergy / 15% structural balance),
//! generates reasoned Team Instructions, depth chart, and weak-spot report.
//! Ported 1:1 from `src/core/generator.js`.

use crate::data::{l, role_by_id, roles_for, slot_info, ti_by_key, FORMATIONS};
use crate::engine::audit::audit_tactic;
use crate::engine::hungarian::{hungarian, match_formations, new_tactic};
use crate::engine::insights::sp_aerial;
use crate::engine::scoring::{foot_factor, pos_factor, role_score, slot_score};
use crate::engine::synergy::synergy_report;
use crate::types::{
    Attr, AttrLookup, AuditCat, BenchCandidate, DepthSlot, Expectation, GenProgress,
    GeneratedCandidate, GeneratorConfig, GeneratorResult, Lang, Mentality, Phase, Player, PosGroup,
    Role, Side, SlotMeta, Tactic, WeakSpot,
};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct FormationPair {
    pub ip: &'static str,
    pub oop: &'static str,
    pub cost: f64,
}

/// Sum of Euclidean IP→OOP transition distances across all 11 matched slots.
pub fn pair_cost(ip: &str, oop: &str) -> f64 {
    match_formations(ip, oop)
        .into_iter()
        .map(|(ip_code, oop_code)| {
            let a = slot_info(ip, &ip_code);
            let b = slot_info(oop, &oop_code);
            match (a, b) {
                (Some(a), Some(b)) => ((a.x - b.x) as f64).hypot((a.y - b.y) as f64),
                _ => 0.0,
            }
        })
        .sum()
}

/// Average outfield y-coordinate of a formation (IP shape must not sit deeper than OOP shape).
pub fn form_depth(form: &str) -> f64 {
    let Some(slots) = crate::data::formation_slots(form) else {
        return 0.0;
    };
    let (mut sum, mut cnt) = (0.0f64, 0usize);
    for s in slots {
        if s.code != "GK" {
            sum += s.y as f64;
            cnt += 1;
        }
    }
    if cnt == 0 {
        0.0
    } else {
        sum / cnt as f64
    }
}

/// All valid `(ip, oop)` formation pairs: for each IP formation, pick the `per_ip` closest
/// OOP formations with `form_depth(ip) >= form_depth(oop) - 0.5`.
pub fn formation_pairs(per_ip: usize) -> Vec<FormationPair> {
    let mut out = Vec::new();
    for ip_f in &FORMATIONS {
        let ip_d = form_depth(ip_f.name);
        let mut cands: Vec<FormationPair> = FORMATIONS
            .iter()
            .filter(|oop_f| ip_d >= form_depth(oop_f.name) - 0.5)
            .map(|oop_f| FormationPair {
                ip: ip_f.name,
                oop: oop_f.name,
                cost: pair_cost(ip_f.name, oop_f.name),
            })
            .collect();
        cands.sort_by(|a, b| a.cost.total_cmp(&b.cost));
        cands.truncate(per_ip);
        out.extend(cands);
    }
    out
}

/// Memoized player × role scorer for fast inner-loop coordinate descent.
struct Scorer<'a> {
    players: &'a [Player],
    cache: HashMap<(usize, &'static str), f64>,
}

impl<'a> Scorer<'a> {
    fn new(players: &'a [Player]) -> Self {
        Self {
            players,
            cache: HashMap::with_capacity(players.len() * 32),
        }
    }

    fn rs(&mut self, pi: usize, role: &'static Role) -> f64 {
        if let Some(&v) = self.cache.get(&(pi, role.id)) {
            return v;
        }
        let v = role_score(&self.players[pi], role)
            .map(|rf| rf.score as f64)
            .unwrap_or(0.0);
        self.cache.insert((pi, role.id), v);
        v
    }

    fn fit(
        &mut self,
        pi: usize,
        a: &SlotMeta,
        b: &SlotMeta,
        ri: &'static Role,
        ro: &'static Role,
    ) -> f64 {
        let p = &self.players[pi];
        let base = self.rs(pi, ri) * 0.55 + self.rs(pi, ro) * 0.45;
        let pf = pos_factor(p, a.group, a.side).min(pos_factor(p, b.group, b.side)) as f64;
        let ff = foot_factor(p, a, ri) as f64;
        base * pf * ff
    }
}

/// Generate an optimal tactic + up to 3 distinct-formation alternatives for `players`.
pub fn generate_tactic(
    players: &[Player],
    cfg: &GeneratorConfig,
    lang: Lang,
    mut on_progress: impl FnMut(GenProgress),
) -> Option<GeneratorResult> {
    if players.len() < 11 {
        return None;
    }
    let exp = cfg.expect;
    let (w_syn, w_fit, w_bal) = if cfg.use_synergy {
        (0.30f64, 0.55f64, 0.15f64)
    } else {
        (0.0f64, 0.72f64, 0.28f64)
    };
    let must: HashSet<&str> = cfg.must.iter().map(String::as_str).collect();
    let youth: HashSet<&str> = cfg.youth.iter().map(String::as_str).collect();
    let star = cfg.star.as_deref().filter(|s| !s.is_empty());
    let depth = if cfg.depth == 0 { 10 } else { cfg.depth };

    on_progress(GenProgress {
        done: 0,
        total: 1,
        label: l(
            lang,
            "A avaliar todos os pares de formações",
            "Evaluating every pair of formations",
        )
        .to_string(),
    });

    let mut scorer = Scorer::new(players);
    let bonus = |p: &Player| -> f64 {
        if youth.contains(p.id.as_str()) {
            6.0
        } else {
            0.0
        }
    };
    let weight = |p: &Player| -> f64 {
        if star == Some(p.id.as_str()) {
            1.6
        } else {
            1.0
        }
    };
    let id_to_idx: HashMap<&str, usize> = players
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id.as_str(), i))
        .collect();

    // ---------- 1. Fast pre-selection across all formation pairs
    let pairs_all = formation_pairs(3);
    let considered = pairs_all.len();
    let mut quick: Vec<(FormationPair, f64)> = Vec::with_capacity(considered);
    for pr in pairs_all {
        let matched = match_formations(pr.ip, pr.oop);
        let metas: Vec<(SlotMeta, SlotMeta)> = matched
            .iter()
            .filter_map(|(ic, oc)| Some((slot_info(pr.ip, ic)?, slot_info(pr.oop, oc)?)))
            .collect();
        let n = metas.len();
        let m = players.len().max(n);
        let mut cost = vec![vec![1000.0f64; m]; n];
        let mut best_table = vec![vec![0.0f64; players.len()]; n];
        for (si, (a, b)) in metas.iter().enumerate() {
            let ip_roles = roles_for(a.group, Phase::Ip, Some(&a.code), a.wide_cb);
            let op_roles = roles_for(b.group, Phase::Oop, Some(&b.code), b.wide_cb);
            for (pi, p) in players.iter().enumerate() {
                let bi = ip_roles
                    .iter()
                    .map(|r| scorer.rs(pi, r))
                    .fold(0.0f64, f64::max);
                let bo = op_roles
                    .iter()
                    .map(|r| scorer.rs(pi, r))
                    .fold(0.0f64, f64::max);
                let pf = pos_factor(p, a.group, a.side).min(pos_factor(p, b.group, b.side)) as f64;
                let bp = (bi * 0.55 + bo * 0.45) * pf;
                best_table[si][pi] = bp;
                let must_pen = if must.contains(p.id.as_str()) {
                    1000.0
                } else {
                    0.0
                };
                cost[si][pi] = -(bp + bonus(p)) * weight(p) - must_pen;
            }
        }
        let asg = hungarian(&cost);
        let mut sum = 0.0f64;
        for (si, &pi) in asg.iter().enumerate() {
            if pi < players.len() {
                sum += best_table[si][pi];
            }
        }
        quick.push((pr, sum / 11.0));
    }
    quick.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut shortlist: Vec<FormationPair> = Vec::new();
    let mut per_ip_count: HashMap<&'static str, usize> = HashMap::new();
    for (pr, _) in quick {
        let c = per_ip_count.entry(pr.ip).or_insert(0);
        if *c >= 2 {
            continue;
        }
        *c += 1;
        shortlist.push(pr);
        if shortlist.len() >= depth {
            break;
        }
    }

    // ---------- 2. Joint optimization: roles × XI
    let assign = |t: &mut Tactic, scorer: &mut Scorer<'_>| {
        let n = t.slots.len();
        let m = players.len().max(n);
        let mut cost = vec![vec![1000.0f64; m]; n];
        for (si, s) in t.slots.iter().enumerate() {
            let (Some(a), Some(b), Some(ri), Some(ro)) = (
                slot_info(&t.ip_form, &s.ip_code),
                slot_info(&t.oop_form, &s.oop_code),
                role_by_id(&s.ip),
                role_by_id(&s.oop),
            ) else {
                continue;
            };
            for (pi, p) in players.iter().enumerate() {
                let must_pen = if must.contains(p.id.as_str()) {
                    1000.0
                } else {
                    0.0
                };
                cost[si][pi] = -(scorer.fit(pi, &a, &b, ri, ro) + bonus(p)) * weight(p) - must_pen;
            }
        }
        let asg = hungarian(&cost);
        for (si, s) in t.slots.iter_mut().enumerate() {
            s.pid = asg
                .get(si)
                .and_then(|&pi| players.get(pi))
                .map(|p| p.id.clone());
        }
    };

    let objective = |t: &Tactic, scorer: &mut Scorer<'_>| -> (f64, f64, f64, f64) {
        let mut sum = 0.0f64;
        let mut wsum = 0.0f64;
        for s in &t.slots {
            let Some(pid) = s.pid.as_deref() else {
                continue;
            };
            let Some(&pi) = id_to_idx.get(pid) else {
                continue;
            };
            let (Some(a), Some(b), Some(ri), Some(ro)) = (
                slot_info(&t.ip_form, &s.ip_code),
                slot_info(&t.oop_form, &s.oop_code),
                role_by_id(&s.ip),
                role_by_id(&s.oop),
            ) else {
                continue;
            };
            let p = &players[pi];
            let w = weight(p);
            sum += (scorer.fit(pi, &a, &b, ri, ro) + bonus(p)) * w;
            wsum += w;
        }
        let fit_avg = if wsum > 0.0 { sum / wsum } else { 0.0 };
        let au = audit_tactic(t, &[], lang);
        let ip_s = *au.scores.get(&AuditCat::Ip).unwrap_or(&0) as f64;
        let oop_s = *au.scores.get(&AuditCat::Oop).unwrap_or(&0) as f64;
        let bal = (ip_s + oop_s) / 2.0;
        let syn = if w_syn > 0.0 {
            synergy_report(t, players, lang).score as f64
        } else {
            0.0
        };
        let v = fit_avg * w_fit + syn * w_syn + bal * w_bal;
        (v, fit_avg, bal, syn)
    };

    let optimised = shortlist.len();
    let mut results: Vec<GeneratedCandidate> = Vec::with_capacity(optimised);
    for (k, pr) in shortlist.iter().enumerate() {
        on_progress(GenProgress {
            done: k,
            total: optimised,
            label: format!(
                "{} {} → {}",
                l(lang, "A optimizar", "Optimising"),
                pr.ip,
                pr.oop
            ),
        });
        let mut t = new_tactic(pr.ip, pr.oop, lang);
        t.ment = exp.mentality();
        t.name = format!("{} ({})", pr.ip, l(lang, "gerada", "generated"));
        assign(&mut t, &mut scorer);
        for _pass in 0..2 {
            for si in 0..t.slots.len() {
                if t.slots[si].pid.is_none() {
                    continue;
                }
                let ip_code = t.slots[si].ip_code.clone();
                let oop_code = t.slots[si].oop_code.clone();
                let (Some(a), Some(b)) = (
                    slot_info(&t.ip_form, &ip_code),
                    slot_info(&t.oop_form, &oop_code),
                ) else {
                    continue;
                };
                // Optimize IP role
                let mut best_ip = t.slots[si].ip.clone();
                let mut best_v = -1.0f64;
                for r in roles_for(a.group, Phase::Ip, Some(&a.code), a.wide_cb) {
                    t.slots[si].ip = r.id.to_string();
                    let (v, _, _, _) = objective(&t, &mut scorer);
                    if v > best_v + 1e-9 {
                        best_v = v;
                        best_ip = r.id.to_string();
                    }
                }
                t.slots[si].ip = best_ip;

                // Optimize OOP role
                let mut best_oop = t.slots[si].oop.clone();
                let mut best_v = -1.0f64;
                for r in roles_for(b.group, Phase::Oop, Some(&b.code), b.wide_cb) {
                    t.slots[si].oop = r.id.to_string();
                    let (v, _, _, _) = objective(&t, &mut scorer);
                    if v > best_v + 1e-9 {
                        best_v = v;
                        best_oop = r.id.to_string();
                    }
                }
                t.slots[si].oop = best_oop;
            }
            assign(&mut t, &mut scorer);
        }
        let (score_v, fit_avg, bal, syn) = objective(&t, &mut scorer);
        results.push(GeneratedCandidate {
            ip_form: pr.ip.to_string(),
            oop_form: pr.oop.to_string(),
            tactic: t,
            fit_avg: fit_avg.round() as u8,
            balance: bal.round() as u8,
            syn: syn.round() as u8,
            total: 0,
            score_v,
            reasons: BTreeMap::new(),
            ment_why: String::new(),
            audit: audit_tactic(&new_tactic(pr.ip, pr.oop, lang), &[], lang),
            synergy: synergy_report(&new_tactic(pr.ip, pr.oop, lang), &[], lang),
        });
    }
    results.sort_by(|a, b| b.score_v.total_cmp(&a.score_v));

    on_progress(GenProgress {
        done: optimised,
        total: optimised,
        label: l(
            lang,
            "A gerar instruções, bolas paradas e alternativas",
            "Building instructions, set pieces and alternatives",
        )
        .to_string(),
    });

    // ---------- 3. Reasoned Team Instructions + final scoring
    for r in &mut results {
        let g = generate_tis(&r.tactic, players, exp, lang);
        r.tactic.ti = g.ti;
        r.tactic.ment = g.ment;
        r.reasons = g.reasons;
        r.ment_why = g.ment_why;
        r.audit = audit_tactic(&r.tactic, players, lang);
        r.synergy = synergy_report(&r.tactic, players, lang);
        r.syn = r.synergy.score;
        r.total = (r.fit_avg as f64 * 0.45
            + r.synergy.score as f64 * 0.30
            + r.audit.overall as f64 * 0.25)
            .round() as u8;
    }
    results.sort_by(|a, b| {
        b.total
            .cmp(&a.total)
            .then_with(|| b.score_v.total_cmp(&a.score_v))
    });

    let mut iter = results.into_iter();
    let main = iter.next()?;
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(main.tactic.ip_form.clone());
    let mut alts = Vec::new();
    for r in iter {
        if seen.contains(&r.tactic.ip_form) {
            continue;
        }
        seen.insert(r.tactic.ip_form.clone());
        alts.push(r);
        if alts.len() == 3 {
            break;
        }
    }

    Some(GeneratorResult {
        main,
        alts,
        considered,
        optimised,
        expect: exp,
    })
}

// ============================================================================
// Reasoned Team Instructions (`generateTIs`)
// ============================================================================

pub struct GeneratedTis {
    pub ti: BTreeMap<String, usize>,
    pub reasons: BTreeMap<String, String>,
    pub ment: Mentality,
    pub ment_why: String,
}

fn avg_of_player(p: &Player, attrs: &[Attr]) -> Option<f64> {
    let (mut s, mut n) = (0.0f64, 0usize);
    for &k in attrs {
        if let Some(v) = p.attr(k) {
            s += v as f64;
            n += 1;
        }
    }
    (n > 0).then(|| s / n as f64)
}

fn avg_players(ps: &[&Player], attrs: &[Attr]) -> Option<f64> {
    let (mut s, mut n) = (0.0f64, 0usize);
    for p in ps {
        if let Some(v) = avg_of_player(p, attrs) {
            s += v;
            n += 1;
        }
    }
    (n > 0).then(|| s / n as f64)
}

fn f1(x: Option<f64>) -> String {
    x.map(|v| format!("{v:.1}")).unwrap_or_else(|| "—".into())
}

pub fn generate_tis(t: &Tactic, players: &[Player], exp: Expectation, lang: Lang) -> GeneratedTis {
    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    struct Xi<'a> {
        p: &'a Player,
        a: SlotMeta,
        ip: Option<&'static Role>,
    }
    let xi: Vec<Xi<'_>> = t
        .slots
        .iter()
        .filter_map(|s| {
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied())?;
            let a = slot_info(&t.ip_form, &s.ip_code)?;
            Some(Xi {
                p,
                a,
                ip: role_by_id(&s.ip),
            })
        })
        .collect();
    let grp = |gs: &[PosGroup]| -> Vec<&Player> {
        xi.iter()
            .filter(|x| gs.contains(&x.a.group))
            .map(|x| x.p)
            .collect()
    };

    let mut top: Vec<&Player> = players.iter().collect();
    top.sort_by(|a, b| {
        let va = avg_of_player(a, &Attr::ALL).unwrap_or(0.0);
        let vb = avg_of_player(b, &Attr::ALL).unwrap_or(0.0);
        vb.total_cmp(&va)
    });
    top.truncate(18);
    let top_outfield: Vec<&Player> = top.into_iter().filter(|p| !p.is_only_gk()).collect();
    let sq = |ks: &[Attr]| avg_players(&top_outfield, ks).unwrap_or(10.0);

    let ti = std::cell::RefCell::new(BTreeMap::new());
    let reasons = std::cell::RefCell::new(BTreeMap::new());
    let set = |k: &str, val: usize, why: String| {
        if let Some(d) = ti_by_key(k) {
            if val != d.def {
                ti.borrow_mut().insert(k.to_string(), val);
            }
        }
        reasons.borrow_mut().insert(k.to_string(), why);
    };
    let keep = |k: &str, why: String| {
        reasons.borrow_mut().insert(k.to_string(), why);
    };

    let out: Vec<&Player> = xi
        .iter()
        .filter(|x| x.a.group != PosGroup::Gk)
        .map(|x| x.p)
        .collect();
    let dc = grp(&[PosGroup::Dc]);
    let fb = grp(&[PosGroup::DLr, PosGroup::WbLr]);
    let mid = grp(&[PosGroup::Dm, PosGroup::Mc, PosGroup::Amc]);
    let wide = grp(&[PosGroup::MLr, PosGroup::AmLr]);
    let st = grp(&[PosGroup::St]);
    let att = grp(&[PosGroup::St, PosGroup::AmLr, PosGroup::Amc, PosGroup::MLr]);
    let gk = grp(&[PosGroup::Gk]);

    let dc_mid_fb: Vec<&Player> = dc
        .iter()
        .chain(mid.iter())
        .chain(fb.iter())
        .copied()
        .collect();
    let tech = avg_players(&dc_mid_fb, &[Attr::Pas, Attr::Tec, Attr::Fir, Attr::Cmp]);
    let phys = avg_players(&out, &[Attr::Pac, Attr::Acc, Attr::Str]);
    let press = avg_players(&out, &[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg]);
    let sq_press = sq(&[Attr::Wor, Attr::Sta, Attr::Acc, Attr::Agg]);
    let att_pace = avg_players(&att, &[Attr::Pac, Attr::Acc]);
    let sq_pace = sq(&[Attr::Pac, Attr::Acc]);
    let cb_pace = avg_players(&dc, &[Attr::Pac, Attr::Acc]);
    let st_air = avg_players(&st, &[Attr::Hea, Attr::Jum, Attr::Str]);
    let cb_air = avg_players(&dc, &[Attr::Hea, Attr::Jum, Attr::Str]);
    let gk_dc: Vec<&Player> = gk.iter().chain(dc.iter()).copied().collect();
    let build_q = avg_players(&gk_dc, &[Attr::Pas, Attr::Cmp, Attr::Fir]);
    let flair = avg_players(&att, &[Attr::Fla, Attr::Dec, Attr::Vis]);
    let dri = avg_players(&att, &[Attr::Dri, Attr::Agi]);
    let lon = {
        let mut ls: Vec<f64> = out
            .iter()
            .map(|p| p.attr(Attr::Lon).unwrap_or(0.0) as f64)
            .collect();
        ls.sort_by(|a, b| b.total_cmp(a));
        ls.truncate(3);
        if ls.is_empty() {
            0.0
        } else {
            ls.iter().sum::<f64>() / ls.len() as f64
        }
    };
    let wide_fb: Vec<&Player> = wide.iter().chain(fb.iter()).copied().collect();
    let wide_q = avg_players(&wide_fb, &[Attr::Cro, Attr::Dri, Attr::Acc]);
    let centre_q = avg_players(&mid, &[Attr::Pas, Attr::Vis, Attr::Dec]);
    let dm = grp(&[PosGroup::Dm]);
    let def_grp: Vec<&Player> = dc
        .iter()
        .chain(fb.iter())
        .chain(dm.iter())
        .copied()
        .collect();
    let def_t = avg_players(&def_grp, &[Attr::Tck, Attr::Agg]);
    let inside = xi
        .iter()
        .filter(|x| {
            matches!(x.a.group, PosGroup::MLr | PosGroup::AmLr)
                && x.ip.is_some_and(|r| r.has_tag(crate::types::RoleTag::In))
        })
        .count();
    let wide_w = xi
        .iter()
        .filter(|x| {
            matches!(x.a.group, PosGroup::MLr | PosGroup::AmLr)
                && x.ip.is_some_and(|r| r.has_tag(crate::types::RoleTag::W))
        })
        .count();
    let side_q = |side: Side| -> Option<f64> {
        let ps: Vec<&Player> = xi
            .iter()
            .filter(|x| x.a.side == side && x.a.group != PosGroup::Gk && x.a.group != PosGroup::Dc)
            .map(|x| x.p)
            .collect();
        avg_players(
            &ps,
            &[Attr::Dri, Attr::Cro, Attr::Pas, Attr::Acc, Attr::Otb],
        )
    };

    // Mentality
    let ment = exp.mentality();
    let mut ment_why = match lang {
        Lang::Pt => format!(
            "{}: {} dá iniciativa sem expor a equipa.",
            exp.label(Lang::Pt),
            ment.as_str()
        ),
        Lang::En => format!(
            "{}: {} takes the initiative without exposing the team.",
            exp.label(Lang::En),
            ment.as_str()
        ),
    };
    if exp.press_bias() < 0.0 {
        ment_why.push_str(l(
            lang,
            " Contra equipas mais fortes, prudência primeiro.",
            " Against stronger sides, caution first.",
        ));
    }

    // ---- In Possession
    match (tech, phys) {
        (Some(te), Some(ph)) if te - ph >= 1.0 => set(
            "passing",
            1,
            match lang {
                Lang::Pt => format!(
                    "Os teus defesas e médios são mais técnicos ({}) do que físicos ({}): passe curto.",
                    f1(tech),
                    f1(phys)
                ),
                Lang::En => format!(
                    "Your defenders and midfielders are more technical ({}) than physical ({}): short passing.",
                    f1(tech),
                    f1(phys)
                ),
            },
        ),
        (Some(te), Some(ph)) if ph - te >= 1.2 && st_air.is_some_and(|a| a >= 13.0) => set(
            "passing",
            3,
            match lang {
                Lang::Pt => format!(
                    "Equipa mais física ({}) do que técnica ({}) e avançados fortes no ar: jogo mais directo.",
                    f1(phys),
                    f1(tech)
                ),
                Lang::En => format!(
                    "A more physical ({}) than technical ({}) side with strikers strong in the air: play more directly.",
                    f1(phys),
                    f1(tech)
                ),
            },
        ),
        _ => keep(
            "passing",
            match lang {
                Lang::Pt => format!("Técnica ({}) e físico ({}) equilibrados.", f1(tech), f1(phys)),
                Lang::En => format!("Technique ({}) and physique ({}) are balanced.", f1(tech), f1(phys)),
            },
        ),
    }

    let mid_dec = avg_players(&mid, &[Attr::Dec, Attr::Tec]);
    if mid_dec.is_some_and(|v| v >= sq(&[Attr::Dec, Attr::Tec]) + 1.0) {
        set(
            "tempo",
            3,
            match lang {
                Lang::Pt => format!(
                    "O meio-campo decide e executa bem ({}): podes acelerar.",
                    f1(mid_dec)
                ),
                Lang::En => format!(
                    "Your midfield decides and executes well ({}): you can speed up.",
                    f1(mid_dec)
                ),
            },
        );
    } else if exp.press_bias() < 0.0 {
        set(
            "tempo",
            1,
            l(
                lang,
                "Como underdog, ritmo mais baixo controla o jogo e poupa energia.",
                "As the underdog, a lower tempo controls the game and saves energy.",
            )
            .to_string(),
        );
    } else {
        keep(
            "tempo",
            l(
                lang,
                "Ritmo normal para o nível do meio-campo.",
                "Normal tempo for your midfield’s level.",
            )
            .to_string(),
        );
    }

    match (wide_q, centre_q) {
        (Some(w), Some(c)) if w - c >= 1.2 => set(
            "width",
            3,
            match lang {
                Lang::Pt => format!(
                    "Corredores mais fortes ({}) do que o meio ({}): abre o jogo.",
                    f1(wide_q),
                    f1(centre_q)
                ),
                Lang::En => format!(
                    "Your flanks are stronger ({}) than the centre ({}): stretch the game.",
                    f1(wide_q),
                    f1(centre_q)
                ),
            },
        ),
        (Some(w), Some(c)) if c - w >= 1.2 => set(
            "width",
            1,
            match lang {
                Lang::Pt => format!(
                    "Meio-campo mais forte ({}) do que os corredores ({}): joga mais por dentro.",
                    f1(centre_q),
                    f1(wide_q)
                ),
                Lang::En => format!(
                    "Your midfield is stronger ({}) than the flanks ({}): play more through the middle.",
                    f1(centre_q),
                    f1(wide_q)
                ),
            },
        ),
        _ => keep(
            "width",
            l(
                lang,
                "Corredores e meio-campo ao mesmo nível.",
                "Flanks and midfield at the same level.",
            )
            .to_string(),
        ),
    }

    if flair.is_some_and(|v| v >= 14.0) {
        set(
            "creative",
            2,
            match lang {
                Lang::Pt => format!(
                    "Atacantes criativos (Talento/Decisões/Visão {}): mais liberdade.",
                    f1(flair)
                ),
                Lang::En => format!(
                    "Creative attackers (Flair/Decisions/Vision {}): more freedom.",
                    f1(flair)
                ),
            },
        );
    } else if flair.is_some_and(|v| v <= 10.5) {
        set(
            "creative",
            0,
            match lang {
                Lang::Pt => format!(
                    "Talento/Decisões baixos na frente ({}): mais disciplina.",
                    f1(flair)
                ),
                Lang::En => format!(
                    "Low Flair/Decisions up front ({}): more discipline.",
                    f1(flair)
                ),
            },
        );
    } else {
        keep(
            "creative",
            l(
                lang,
                "Criatividade média: equilíbrio.",
                "Average creativity: balance.",
            )
            .to_string(),
        );
    }

    if att_pace.is_some_and(|v| v >= 14.0f64.max(sq_pace + 1.0)) || exp.press_bias() < 0.0 {
        let why = if att_pace.is_some_and(|v| v >= 14.0) {
            match lang {
                Lang::Pt => format!(
                    "Atacantes rápidos ({}): ataca a desorganização adversária.",
                    f1(att_pace)
                ),
                Lang::En => format!(
                    "Quick attackers ({}): exploit the opponent’s disorganisation.",
                    f1(att_pace)
                ),
            }
        } else {
            l(
                lang,
                "Como underdog, o contra-ataque é a melhor arma.",
                "As the underdog, the counter-attack is your best weapon.",
            )
            .to_string()
        };
        set("att_trans", 2, why);
    } else if exp.press_bias() > 0.0 && tech.is_some_and(|v| v >= 13.0) {
        set(
            "att_trans",
            0,
            l(
                lang,
                "Favorito e técnico: depois de recuperar, assume a posse.",
                "Favourites and technical: after winning the ball, take control of possession.",
            )
            .to_string(),
        );
    } else {
        keep(
            "att_trans",
            l(lang, "Transição equilibrada.", "A balanced transition.").to_string(),
        );
    }

    if exp.press_bias() > 0.0 {
        set(
            "timewaste",
            0,
            l(
                lang,
                "Favorito: queres ritmo, não queimar tempo.",
                "Favourites: you want tempo, not to run down the clock.",
            )
            .to_string(),
        );
    } else {
        keep(
            "timewaste",
            l(
                lang,
                "Ajusta durante o jogo conforme o resultado.",
                "Adjust during the match depending on the score.",
            )
            .to_string(),
        );
    }

    let mut air_vals: Vec<f64> = out.iter().map(|p| sp_aerial(*p) as f64).collect();
    air_vals.sort_by(|a, b| b.total_cmp(a));
    air_vals.truncate(4);
    let air_top = if air_vals.is_empty() {
        0.0
    } else {
        air_vals.iter().sum::<f64>() / air_vals.len() as f64
    };
    if air_top >= 14.0 {
        set(
            "set_pieces",
            2,
            match lang {
                Lang::Pt => format!("Quatro jogadores fortes no ar (média {air_top:.1}): procura cantos e livres."),
                Lang::En => format!("Four players strong in the air (average {air_top:.1}): go for corners and free kicks."),
            },
        );
    } else if air_top <= 11.0 {
        set(
            "set_pieces",
            0,
            match lang {
                Lang::Pt => format!("Pouca presença aérea ({air_top:.1}): mantém a bola em jogo."),
                Lang::En => {
                    format!("Little aerial presence ({air_top:.1}): keep the ball in play.")
                }
            },
        );
    } else {
        keep(
            "set_pieces",
            l(lang, "Presença aérea média.", "Average aerial presence.").to_string(),
        );
    }

    let att_otb = avg_players(&att, &[Attr::Otb]);
    let att_tech = avg_players(&att, &[Attr::Fir, Attr::Tec]);
    if att_pace.is_some_and(|v| v >= 14.0) && att_otb.is_some_and(|v| v >= 12.5) {
        set(
            "reception",
            0,
            match lang {
                Lang::Pt => format!(
                    "Avançados rápidos e com bom movimento ({} / {}): passe para o espaço.",
                    f1(att_pace),
                    f1(att_otb)
                ),
                Lang::En => format!(
                    "Quick forwards with good movement ({} / {}): pass into space.",
                    f1(att_pace),
                    f1(att_otb)
                ),
            },
        );
    } else if att_tech.is_some_and(|v| v >= att_pace.unwrap_or(0.0) + 1.5) {
        set(
            "reception",
            2,
            l(
                lang,
                "Avançados mais técnicos do que rápidos: passe ao pé.",
                "Forwards more technical than quick: pass to feet.",
            )
            .to_string(),
        );
    } else {
        keep(
            "reception",
            l(
                lang,
                "Mistura de passe ao pé e para o espaço.",
                "A mix of passes to feet and into space.",
            )
            .to_string(),
        );
    }

    if build_q.is_some_and(|v| v >= 13.5) {
        set(
            "buildup",
            0,
            match lang {
                Lang::Pt => format!(
                    "GR e centrais bons com bola ({}): sai a jogar contra a pressão.",
                    f1(build_q)
                ),
                Lang::En => format!(
                    "Keeper and centre-backs good on the ball ({}): play through the press.",
                    f1(build_q)
                ),
            },
        );
    } else if build_q.is_some_and(|v| v <= 10.5) {
        set(
            "buildup",
            2,
            match lang {
                Lang::Pt => format!(
                    "GR e centrais fracos com bola ({}): salta a pressão.",
                    f1(build_q)
                ),
                Lang::En => format!(
                    "Keeper and centre-backs weak on the ball ({}): bypass the press.",
                    f1(build_q)
                ),
            },
        );
    } else {
        keep(
            "buildup",
            match lang {
                Lang::Pt => format!("Qualidade de construção média ({}).", f1(build_q)),
                Lang::En => format!("Average build-up quality ({}).", f1(build_q)),
            },
        );
    }

    if build_q.is_some_and(|v| v >= 13.5) {
        set(
            "goal_kicks",
            0,
            l(
                lang,
                "Construção curta a partir do GR.",
                "Short build-up from the keeper.",
            )
            .to_string(),
        );
    } else if build_q.is_some_and(|v| v <= 10.5) && st_air.is_some_and(|v| v >= 13.5) {
        set(
            "goal_kicks",
            2,
            match lang {
                Lang::Pt => format!(
                    "Pontapés longos para o teu avançado forte no ar ({}).",
                    f1(st_air)
                ),
                Lang::En => format!(
                    "Long kicks to your striker who is strong in the air ({}).",
                    f1(st_air)
                ),
            },
        );
    } else {
        keep(
            "goal_kicks",
            l(lang, "Mistura curto e longo.", "A mix of short and long.").to_string(),
        );
    }

    if ti.borrow().get("att_trans") == Some(&2) {
        set(
            "gk_speed",
            2,
            l(
                lang,
                "Contra-ataque: o GR repõe rápido.",
                "Counter-attack: the keeper distributes quickly.",
            )
            .to_string(),
        );
    } else {
        keep(
            "gk_speed",
            l(
                lang,
                "Ritmo normal de reposição.",
                "Normal distribution speed.",
            )
            .to_string(),
        );
    }

    let buildup_val = ti.borrow().get("buildup").copied();
    if buildup_val == Some(2) && st_air.is_some_and(|v| v >= 14.0) {
        set(
            "gk_target",
            4,
            l(
                lang,
                "Bola longa para a referência aérea.",
                "Long ball to the aerial target.",
            )
            .to_string(),
        );
    } else if buildup_val == Some(0)
        && avg_players(&fb, &[Attr::Pas, Attr::Fir]).is_some_and(|v| v >= 13.0)
    {
        set(
            "gk_target",
            1,
            l(
                lang,
                "Laterais bons com bola: começa por eles.",
                "Full-backs good on the ball: start through them.",
            )
            .to_string(),
        );
    } else {
        keep(
            "gk_target",
            l(lang, "Distribuição variada.", "Varied distribution.").to_string(),
        );
    }

    if inside >= 1 && wide_w == 0 {
        set(
            "support_runs",
            2,
            l(
                lang,
                "Os teus alas entram por dentro: os laterais sobrepõem por fora.",
                "Your wide players come inside: the full-backs overlap outside.",
            )
            .to_string(),
        );
    } else if wide_w >= 2 {
        set(
            "support_runs",
            0,
            l(
                lang,
                "Alas abertos: os laterais apoiam por dentro.",
                "Wide wingers: the full-backs support inside.",
            )
            .to_string(),
        );
    } else {
        keep(
            "support_runs",
            l(
                lang,
                "Apoios mistos conforme o flanco.",
                "Mixed support depending on the flank.",
            )
            .to_string(),
        );
    }

    if dri.is_some_and(|v| v >= 14.0) {
        set(
            "dribbling",
            2,
            match lang {
                Lang::Pt => format!("Atacantes com Drible/Agilidade altos ({}).", f1(dri)),
                Lang::En => format!("Attackers with high Dribbling/Agility ({}).", f1(dri)),
            },
        );
    } else if dri.is_some_and(|v| v <= 10.0) {
        set(
            "dribbling",
            0,
            match lang {
                Lang::Pt => format!("Drible fraco na frente ({}): solta a bola.", f1(dri)),
                Lang::En => format!(
                    "Weak dribbling up front ({}): release the ball early.",
                    f1(dri)
                ),
            },
        );
    } else {
        keep(
            "dribbling",
            l(lang, "Drible na média.", "Average dribbling.").to_string(),
        );
    }

    match (side_q(Side::L), side_q(Side::R)) {
        (Some(ql), Some(qr)) if (ql - qr).abs() >= 1.3 => {
            let left_stronger = ql > qr;
            set(
                "progress",
                if left_stronger { 0 } else { 3 },
                match lang {
                    Lang::Pt => format!(
                        "O teu flanco {} é mais forte ({:.1} vs {:.1}).",
                        if left_stronger { "esquerdo" } else { "direito" },
                        ql.max(qr),
                        ql.min(qr)
                    ),
                    Lang::En => format!(
                        "Your {} flank is stronger ({:.1} vs {:.1}).",
                        if left_stronger { "left" } else { "right" },
                        ql.max(qr),
                        ql.min(qr)
                    ),
                },
            );
        }
        _ => keep(
            "progress",
            l(lang, "Flancos equilibrados.", "Balanced flanks.").to_string(),
        ),
    }

    if exp.press_bias() > 0.0 && tech.is_some_and(|v| v >= 13.0) {
        set(
            "patience",
            2,
            l(
                lang,
                "Favorito e técnico: trabalha a bola até à área.",
                "Favourites and technical: work the ball into the box.",
            )
            .to_string(),
        );
    } else if exp.press_bias() < 0.0 {
        set(
            "patience",
            0,
            l(
                lang,
                "Como underdog, cruzar cedo antes da defesa adversária se organizar.",
                "As the underdog, hit early crosses before the opposition sets.",
            )
            .to_string(),
        );
    } else {
        keep(
            "patience",
            l(lang, "Paciência normal.", "Normal patience.").to_string(),
        );
    }

    if lon >= 14.5 {
        set(
            "long_shots",
            2,
            match lang {
                Lang::Pt => format!("Há quem remate bem de longe (top 3: {lon:.1})."),
                Lang::En => format!("You have good long-range shooters (top 3: {lon:.1})."),
            },
        );
    } else if lon <= 10.0 {
        set(
            "long_shots",
            0,
            match lang {
                Lang::Pt => format!("Remates de longe fracos ({lon:.1})."),
                Lang::En => format!("Weak long-range shooting ({lon:.1})."),
            },
        );
    } else {
        keep(
            "long_shots",
            l(
                lang,
                "Remates de longe na média.",
                "Average long-range shooting.",
            )
            .to_string(),
        );
    }

    if st_air.is_some_and(|v| v >= 15.0) {
        set(
            "crossing",
            3,
            match lang {
                Lang::Pt => format!(
                    "Avançados dominantes no ar ({}): cruzamentos altos.",
                    f1(st_air)
                ),
                Lang::En => format!(
                    "Strikers dominant in the air ({}): high crosses.",
                    f1(st_air)
                ),
            },
        );
    } else if st_air.is_some_and(|v| v <= 11.0) && att_pace.is_some_and(|v| v >= 13.0) {
        set(
            "crossing",
            0,
            l(
                lang,
                "Avançados baixos mas rápidos: cruzamentos rasteiros.",
                "Short but quick strikers: low crosses.",
            )
            .to_string(),
        );
    } else {
        keep(
            "crossing",
            l(lang, "Cruzamentos variados.", "Varied crosses.").to_string(),
        );
    }

    // ---- Out of Possession
    let press_rel = press.map(|v| v - sq_press).unwrap_or(0.0);
    if press.is_some_and(|v| v >= 13.5 || press_rel >= 1.0) && exp.press_bias() >= 0.0 {
        set(
            "loe",
            2,
            match lang {
                Lang::Pt => format!(
                    "Índice de Trabalho/Resistência/Aceleração do XI {}: aguenta pressão alta.",
                    f1(press)
                ),
                Lang::En => format!(
                    "The XI’s Work Rate/Stamina/Acceleration {}: it can sustain a high press.",
                    f1(press)
                ),
            },
        );
    } else if press.is_some_and(|v| v <= 10.5) || exp.press_bias() < 0.0 {
        let why = if press.is_some_and(|v| v <= 10.5) {
            match lang {
                Lang::Pt => format!("Pouca capacidade de pressão ({}): bloco baixo.", f1(press)),
                Lang::En => format!("Little pressing capacity ({}): low block.", f1(press)),
            }
        } else {
            l(
                lang,
                "Como underdog, bloco baixo e compacto.",
                "As the underdog, a low and compact block.",
            )
            .to_string()
        };
        set("loe", 0, why);
    } else {
        keep("loe", l(lang, "Bloco médio.", "Mid block.").to_string());
    }

    let loe_v = ti.borrow().get("loe").copied().unwrap_or(1);
    let cb_rel = cb_pace.map(|v| v - sq_pace).unwrap_or(0.0);
    if cb_pace.is_some_and(|v| cb_rel <= -1.5 || v <= 10.5) {
        set(
            "dline",
            (loe_v * 2).saturating_sub(1).max(1),
            match lang {
                Lang::Pt => format!("Centrais mais lentos do que o resto da equipa ({} vs {sq_pace:.1}): linha mais baixa.", f1(cb_pace)),
                Lang::En => format!("Centre-backs slower than the rest of the team ({} vs {sq_pace:.1}): a deeper line.", f1(cb_pace)),
            },
        );
    } else if cb_pace.is_some_and(|v| v >= 13.5) && loe_v == 2 {
        set(
            "dline",
            3,
            match lang {
                Lang::Pt => format!(
                    "Centrais rápidos ({}) e pressão alta: linha subida.",
                    f1(cb_pace)
                ),
                Lang::En => format!(
                    "Quick centre-backs ({}) and a high press: a high line.",
                    f1(cb_pace)
                ),
            },
        );
    } else {
        set(
            "dline",
            [1, 2, 3][loe_v.min(2)],
            l(
                lang,
                "Linha alinhada com a altura da pressão.",
                "The line matches the height of the press.",
            )
            .to_string(),
        );
    }

    if press.is_some_and(|v| v >= 14.0) {
        set(
            "trigger",
            3,
            match lang {
                Lang::Pt => format!("Muita energia no XI ({}).", f1(press)),
                Lang::En => format!("Plenty of energy in the XI ({}).", f1(press)),
            },
        );
    } else if press.is_some_and(|v| v <= 10.5) {
        set(
            "trigger",
            1,
            match lang {
                Lang::Pt => format!("Pouca energia ({}): pressiona menos.", f1(press)),
                Lang::En => format!("Little energy ({}): press less.", f1(press)),
            },
        );
    } else {
        keep(
            "trigger",
            l(lang, "Pressão normal.", "Normal pressing.").to_string(),
        );
    }

    if press.is_some_and(|v| v >= 13.0) && exp.press_bias() >= 0.0 {
        set(
            "def_trans",
            2,
            l(
                lang,
                "Recupera a bola logo após a perda.",
                "Win the ball back straight after losing it.",
            )
            .to_string(),
        );
    } else if exp.press_bias() < 0.0 || press.is_some_and(|v| v <= 10.5) {
        set(
            "def_trans",
            0,
            l(
                lang,
                "Reagrupa primeiro: protege a organização.",
                "Regroup first: protect the shape.",
            )
            .to_string(),
        );
    } else {
        keep(
            "def_trans",
            l(
                lang,
                "Reacção equilibrada à perda.",
                "A balanced reaction to losing the ball.",
            )
            .to_string(),
        );
    }

    if def_t.is_some_and(|v| v >= 14.0) {
        set(
            "tackling",
            2,
            match lang {
                Lang::Pt => format!("Defesas fortes no desarme ({}).", f1(def_t)),
                Lang::En => format!("Defenders strong in the tackle ({}).", f1(def_t)),
            },
        );
    } else if def_t.is_some_and(|v| v <= 10.5) {
        set(
            "tackling",
            0,
            match lang {
                Lang::Pt => format!("Desarme fraco ({}): evita faltas.", f1(def_t)),
                Lang::En => format!("Weak tackling ({}): avoid fouls.", f1(def_t)),
            },
        );
    } else {
        keep(
            "tackling",
            l(lang, "Desarme normal.", "Normal tackling.").to_string(),
        );
    }

    let fb_agg = avg_players(&fb, &[Attr::Acc, Attr::Tck, Attr::Agg]);
    let dm_mc = grp(&[PosGroup::Dm, PosGroup::Mc]);
    let md_def = avg_players(&dm_mc, &[Attr::Tck, Attr::Pos, Attr::Ant]);
    match (fb_agg, md_def) {
        (Some(fa), Some(md)) if fa - md >= 1.0 => set(
            "press_trap",
            2,
            match lang {
                Lang::Pt => format!(
                    "Laterais agressivos ({}): força o jogo para fora.",
                    f1(fb_agg)
                ),
                Lang::En => format!("Aggressive full-backs ({}): force play wide.", f1(fb_agg)),
            },
        ),
        (Some(fa), Some(md)) if md - fa >= 1.0 => set(
            "press_trap",
            0,
            match lang {
                Lang::Pt => format!(
                    "Meio-campo forte a defender ({}): força o jogo para dentro.",
                    f1(md_def)
                ),
                Lang::En => format!(
                    "A midfield strong defensively ({}): force play inside.",
                    f1(md_def)
                ),
            },
        ),
        _ => keep(
            "press_trap",
            l(lang, "Sem armadilha específica.", "No specific trap.").to_string(),
        ),
    }

    if loe_v == 2 {
        set(
            "short_gk",
            1,
            l(
                lang,
                "Pressão alta: não deixes o GR adversário sair curto.",
                "High press: do not let the opposing keeper play short.",
            )
            .to_string(),
        );
    } else {
        keep(
            "short_gk",
            l(
                lang,
                "Deixa o adversário sair curto.",
                "Let the opponent play out short.",
            )
            .to_string(),
        );
    }

    keep(
        "dl_behaviour",
        l(
            lang,
            "Comportamento normal da linha.",
            "Normal line behaviour.",
        )
        .to_string(),
    );

    if cb_air.is_some_and(|v| v >= 14.0) {
        set(
            "cross_eng",
            0,
            match lang {
                Lang::Pt => format!(
                    "Centrais dominantes no ar ({}): deixa-os cruzar.",
                    f1(cb_air)
                ),
                Lang::En => format!(
                    "Centre-backs dominant in the air ({}): let them cross.",
                    f1(cb_air)
                ),
            },
        );
    } else if cb_air.is_some_and(|v| v <= 11.5) {
        set(
            "cross_eng",
            2,
            match lang {
                Lang::Pt => format!(
                    "Centrais fracos no ar ({}): impede os cruzamentos.",
                    f1(cb_air)
                ),
                Lang::En => format!(
                    "Centre-backs weak in the air ({}): stop the crosses.",
                    f1(cb_air)
                ),
            },
        );
    } else {
        keep(
            "cross_eng",
            l(
                lang,
                "Cruzamentos adversários sem tratamento especial.",
                "No special treatment for opposition crosses.",
            )
            .to_string(),
        );
    }

    GeneratedTis {
        ti: ti.into_inner(),
        reasons: reasons.into_inner(),
        ment,
        ment_why,
    }
}

// ============================================================================
// Depth Chart & Weak Spots
// ============================================================================

/// Top `n` candidates in the squad for each tactical slot.
pub fn depth_chart(t: &Tactic, players: &[Player], n: usize) -> Vec<DepthSlot> {
    t.slots
        .iter()
        .map(|s| {
            let mut list: Vec<BenchCandidate> = players
                .iter()
                .map(|p| BenchCandidate {
                    player_id: p.id.clone(),
                    player_name: p.name.clone(),
                    age: p.age,
                    score: slot_score(p, t, s),
                })
                .collect();
            list.sort_by_key(|a| std::cmp::Reverse(a.score));
            list.truncate(n);
            DepthSlot {
                code: s.ip_code.clone(),
                list,
            }
        })
        .collect()
}

/// Urgent recruitment/training spots where starter `< 55` or best alternative `< 45`.
pub fn weak_spots(t: &Tactic, players: &[Player]) -> Vec<WeakSpot> {
    let pm: HashMap<&str, &Player> = players.iter().map(|p| (p.id.as_str(), p)).collect();
    let mut out: Vec<WeakSpot> = t
        .slots
        .iter()
        .filter_map(|s| {
            let ri = role_by_id(&s.ip)?;
            let p = s.pid.as_deref().and_then(|id| pm.get(id).copied());
            let sc = p.map(|x| slot_score(x, t, s)).unwrap_or(0);
            let alt = players
                .iter()
                .filter(|q| s.pid.as_deref() != Some(q.id.as_str()))
                .map(|q| slot_score(q, t, s))
                .max()
                .unwrap_or(0);
            let mut lack: Vec<(Attr, f32)> = p
                .map(|x| {
                    ri.key
                        .iter()
                        .filter_map(|&k| Some((k, x.attr(k)?)))
                        .collect()
                })
                .unwrap_or_default();
            lack.sort_by(|a, b| a.1.total_cmp(&b.1));
            let lack_attrs: Vec<Attr> = lack.into_iter().take(3).map(|(k, _)| k).collect();
            let urgent = sc < 55 || alt < 45;
            urgent.then(|| WeakSpot {
                code: s.ip_code.clone(),
                role_id: ri.id,
                player_id: p.map(|x| x.id.clone()),
                player_name: p.map(|x| x.name.clone()),
                score: sc,
                alt,
                lack: lack_attrs,
                urgent,
            })
        })
        .collect();
    out.sort_by_key(|x| x.score);
    out
}
