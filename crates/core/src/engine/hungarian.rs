//! Hungarian assignment solver and tactic/formation matching helpers.

use crate::data::roles::{default_role, valid_role};
use crate::data::tactics_data::{formation_slots, slot_info};
use crate::engine::scoring::slot_score;
use crate::types::{Lang, Mentality, Phase, Player, SlotAssignment, Tactic, Template};
use std::collections::BTreeMap;

/// O(n^2 * m) Hungarian algorithm for minimum-cost rectangular assignment (`n <= m`).
///
/// Returns a vector of length `n` where `res[i]` is the column assigned to row `i`.
pub fn hungarian(cost: &[Vec<f64>]) -> Vec<usize> {
    let n = cost.len();
    if n == 0 {
        return Vec::new();
    }
    let m = cost[0].len();
    if m == 0 {
        return Vec::new();
    }
    debug_assert!(n <= m, "hungarian requires rows ({n}) <= cols ({m})");

    const INF: f64 = 1e18;
    let mut u = vec![0.0_f64; n + 1];
    let mut v = vec![0.0_f64; m + 1];
    let mut p = vec![0_usize; m + 1];
    let mut way = vec![0_usize; m + 1];

    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0_usize;
        let mut minv = vec![INF; m + 1];
        let mut used = vec![false; m + 1];

        loop {
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = INF;
            let mut j1 = 0_usize;

            for j in 1..=m {
                if !used[j] {
                    let cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
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

    let mut res = vec![0_usize; n];
    for j in 1..=m {
        if p[j] != 0 {
            res[p[j] - 1] = j - 1;
        }
    }
    res
}

/// Pair each IP slot with its nearest OOP slot (minimum total weighted distance, GK locked to GK).
pub fn match_formations(ip_form: &str, oop_form: &str) -> Vec<(String, String)> {
    let (Some(a_slots), Some(b_slots)) = (formation_slots(ip_form), formation_slots(oop_form))
    else {
        return Vec::new();
    };
    let cost: Vec<Vec<f64>> = a_slots
        .iter()
        .map(|a| {
            b_slots
                .iter()
                .map(|b| {
                    if (a.code == "GK") != (b.code == "GK") {
                        1e6
                    } else {
                        ((a.x as f64) - (b.x as f64)).hypot(((a.y as f64) - (b.y as f64)) * 1.1)
                    }
                })
                .collect()
        })
        .collect();
    let asg = hungarian(&cost);
    a_slots
        .iter()
        .enumerate()
        .map(|(i, a)| (a.code.to_string(), b_slots[asg[i]].code.to_string()))
        .collect()
}

/// Create a fresh default tactic for a pair of IP/OOP formations.
pub fn new_tactic(ip_form: &str, oop_form: &str, lang: Lang) -> Tactic {
    let pairs = match_formations(ip_form, oop_form);
    let slots = pairs
        .into_iter()
        .filter_map(|(ip_code, oop_code)| {
            let a = slot_info(ip_form, &ip_code)?;
            let b = slot_info(oop_form, &oop_code)?;
            Some(SlotAssignment {
                ip_code,
                oop_code,
                ip: default_role(a.group, Phase::Ip).to_string(),
                oop: default_role(b.group, Phase::Oop).to_string(),
                pid: None,
                pis: Vec::new(),
            })
        })
        .collect();

    Tactic {
        name: lang.pick("Nova tática", "New tactic").to_string(),
        ment: Mentality::Balanced,
        ip_form: ip_form.to_string(),
        oop_form: oop_form.to_string(),
        slots,
        ti: BTreeMap::new(),
        ois: 0,
        source: "manual".to_string(),
        saved_at: None,
    }
}

/// Change a tactic's IP and/or OOP formations while preserving valid roles, players, and PIs.
pub fn retarget_tactic(t: &Tactic, ip_form: &str, oop_form: &str, lang: Lang) -> Tactic {
    let mut nt = new_tactic(ip_form, oop_form, lang);
    nt.name = t.name.clone();
    nt.ment = t.ment;
    nt.ti = t.ti.clone();
    nt.ois = t.ois;
    nt.source = t.source.clone();

    let (Some(old_ip), Some(new_ip)) = (formation_slots(&t.ip_form), formation_slots(ip_form))
    else {
        return nt;
    };

    let cost: Vec<Vec<f64>> = new_ip
        .iter()
        .map(|a| {
            old_ip
                .iter()
                .map(|b| {
                    if (a.code == "GK") != (b.code == "GK") {
                        1e6
                    } else {
                        ((a.x as f64) - (b.x as f64)).hypot((a.y as f64) - (b.y as f64))
                    }
                })
                .collect()
        })
        .collect();
    let asg = hungarian(&cost);

    for (i, s) in nt.slots.iter_mut().enumerate() {
        let Some(&old_idx) = asg.get(i) else {
            continue;
        };
        let Some(old_slot_def) = old_ip.get(old_idx) else {
            continue;
        };
        let Some(old) = t.slots.iter().find(|o| o.ip_code == old_slot_def.code) else {
            continue;
        };
        let (Some(a), Some(b)) = (
            slot_info(ip_form, &s.ip_code),
            slot_info(oop_form, &s.oop_code),
        ) else {
            continue;
        };
        if valid_role(&old.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb) {
            s.ip = old.ip.clone();
        }
        if valid_role(&old.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb) {
            s.oop = old.oop.clone();
        }
        s.pid = old.pid.clone();
        s.pis = old.pis.clone();
    }

    nt
}

/// Instantiate a `Tactic` from one of the built-in templates.
pub fn tactic_from_template(tp: &Template) -> Tactic {
    let mut t = new_tactic(tp.ip, tp.oop, Lang::Pt);
    t.name = tp.name_pt.to_string();
    t.ment = tp.ment;
    t.ti = tp.ti.iter().map(|&(k, v)| (k.to_string(), v)).collect();
    t.source = "template".to_string();

    for s in &mut t.slots {
        let Some(&(_, ip_r, oop_r)) = tp.roles.iter().find(|&&(code, _, _)| code == s.ip_code)
        else {
            continue;
        };
        let (Some(a), Some(b)) = (slot_info(tp.ip, &s.ip_code), slot_info(tp.oop, &s.oop_code))
        else {
            continue;
        };
        if valid_role(ip_r, a.group, Phase::Ip, Some(&a.code), a.wide_cb) {
            s.ip = ip_r.to_string();
        }
        if valid_role(oop_r, b.group, Phase::Oop, Some(&b.code), b.wide_cb) {
            s.oop = oop_r.to_string();
        }
    }

    t
}

/// Assign the best-fitting distinct starting XI from `players` using the Hungarian solver.
pub fn auto_pick_xi(t: &mut Tactic, players: &[Player]) {
    if players.is_empty() || t.slots.is_empty() {
        return;
    }
    let n_slots = t.slots.len();
    let n_cols = players.len().max(n_slots);
    let mut cost: Vec<Vec<f64>> = t
        .slots
        .iter()
        .map(|s| {
            let mut row: Vec<f64> = players
                .iter()
                .map(|p| (100 - slot_score(p, t, s) as i32) as f64)
                .collect();
            row.resize(n_cols, 1000.0);
            row
        })
        .collect();

    // Ensure rectangular dimensions (n_slots <= n_cols)
    let _ = &mut cost;
    let asg = hungarian(&cost);
    for (i, s) in t.slots.iter_mut().enumerate() {
        s.pid = players.get(asg[i]).map(|p| p.id.clone());
    }
}
