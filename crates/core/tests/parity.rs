//! Comprehensive Golden Parity & i18n Audit Suite for `ta-core`.
//!
//! Covers all 25 suites from `tests/engine.test.cjs`, live simulator continuity
//! checks from `tests/sim_canvas.test.cjs`, end-to-end `fmbridge` integration,
//! and bidirectional EN/PT zero-leak & `pt_terms` audits replacing
//! `tests/ui_i18n.py`, `tests/ui_pt_terms.py`, and `tests/ui_lang_audit.py`.

use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use ta_core::data::i18n::{
    mentality_name, pi_name, pt_terms, role_name, t, t_name, ti_name, ti_opt_name,
    EN_DICT_ENTRIES, PT_TERMS,
};
use ta_core::data::roles::{attr_meta, attr_name, group_name, role_by_id, roles_for, valid_role, ROLES};
use ta_core::data::tactics_data::{
    formation_slots, has_player_instruction, mentality_info, slot_info, ti_by_key, FORMATIONS,
    PLAYER_INSTRUCTIONS, STYLE_PROFILES, TACTICAL_TWEAKS, TEAM_INSTRUCTIONS, TEMPLATES,
    TRAINING_AREAS,
};
use ta_core::engine::audit::audit_tactic;
use ta_core::engine::hungarian::{
    auto_pick_xi, match_formations, new_tactic, retarget_tactic, tactic_from_template,
};
use ta_core::engine::insights::{
    bench_and_successors, opponent_report, recommend_pis, set_piece_plan,
};
use ta_core::engine::parsers::{
    inspect_fmf, majority_club, parse_fmf_container, parse_positions, parse_squad, surname,
};
use ta_core::engine::scoring::{complexity_of, suggest_tactics, training_plan};
use ta_core::engine::synergy::synergy_report;
use ta_core::live::fmbridge::{ApplyTacticOptions, FmbClient, FmbIds};
use ta_core::live::sim_model::{phase_text, sim_ball_state, sim_model, PHASES};
use ta_core::planning::gameplans::{game_intent, game_plan};
use ta_core::planning::generator::{depth_chart, form_depth, generate_tactic, weak_spots};
use ta_core::planning::scouting::{
    best_gain, enrich_from_scout, infer_game_date, join_likelihood, months_until, parse_scout_db,
    scout_filter, scout_presets, SCOUT_COLUMNS,
};
use ta_core::types::{
    Attr, BallKind, BallPoint, BridgeError, BridgeTransport, Expectation, GameStateKind,
    GeneratorConfig, Lang, Mentality, Phase, Player, PosGroup, PosLevel, Role, ScoutAttrFilter,
    ScoutContext, ScoutFilter, ScoutHid, ScoutPers, ScoutPlayer, ScoutPosKey, ScoutSort, Severity,
    SimPhase, SynergyPhase, Tactic,
};

// ============================================================================
// Shared Fixture & Deterministic LCG Helpers
// ============================================================================

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn read_fixture_text(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
}

fn read_fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name))
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
}

fn test_tactic_for_role(r: &Role) -> (Tactic, Option<usize>) {
    let fits = |form: &str, code: &str| {
        slot_info(form, code)
            .is_some_and(|a| valid_role(r.id, a.group, r.phase, Some(&a.code), a.wide_cb))
    };
    let Some(form) = FORMATIONS
        .iter()
        .find(|f| f.slots.iter().any(|z| fits(f.name, z.code)))
    else {
        return (new_tactic("4-3-3 DM Wide", "4-1-4-1 DM Wide", Lang::En), None);
    };
    let mut tac = new_tactic(form.name, form.name, Lang::En);
    let idx = tac.slots.iter().position(|s| fits(form.name, &s.ip_code));
    if let Some(i) = idx {
        match r.phase {
            Phase::Ip => tac.slots[i].ip = r.id.to_string(),
            Phase::Oop => tac.slots[i].oop = r.id.to_string(),
        }
    }
    (tac, idx)
}

const GEN_ATTR_NAMES: [&str; 52] = [
    "crossing",
    "dribbling",
    "finishing",
    "heading",
    "long_shots",
    "marking",
    "off_the_ball",
    "passing",
    "penalty_taking",
    "tackling",
    "vision",
    "handling",
    "aerial_reach",
    "command_of_area",
    "communication",
    "kicking",
    "throwing",
    "anticipation",
    "decisions",
    "one_on_ones",
    "positioning",
    "reflexes",
    "first_touch",
    "technique",
    "flair",
    "corners",
    "teamwork",
    "work_rate",
    "long_throws",
    "eccentricity",
    "rushing_out",
    "punching",
    "acceleration",
    "free_kick_taking",
    "strength",
    "stamina",
    "pace",
    "jumping_reach",
    "leadership",
    "dirtiness",
    "balance",
    "bravery",
    "consistency",
    "aggression",
    "agility",
    "important_matches",
    "injury_proneness",
    "versatility",
    "natural_fitness",
    "determination",
    "composure",
    "concentration",
];

fn gen_scout_columns() -> Vec<String> {
    let mut cols: Vec<String> = vec![
        "uid",
        "unique_id",
        "name",
        "age",
        "nation_id",
        "height_cm",
        "ability_current",
        "ability_potential",
        "ability_potential_range_code",
        "reputation_current",
        "reputation_world",
        "club_uid",
        "club_name",
        "club_nation_id",
        "club_reputation",
        "on_loan",
        "loan_parent_club_name",
        "transfer_value",
        "contract_wage",
        "contract_end",
        "contract_squad_status",
        "left_foot",
        "right_foot",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    for p in ScoutPosKey::NAMES {
        cols.push(format!("positions_{p}"));
    }
    for p in ScoutPers::NAMES {
        cols.push(format!("personality_{p}"));
    }
    for a in GEN_ATTR_NAMES {
        cols.push(format!("attributes_{a}"));
    }
    cols.push("transfer_value_state".to_string());
    cols.push("transfer_value_state_code".to_string());
    cols
}

/// Deterministic LCG CSV generator matching `tests/gen_scout_db.cjs` (same seed & formulas).
fn gen_scout_db_csv(n: usize, seed: u64) -> String {
    struct Rng(u64);
    impl Rng {
        fn next_r(&mut self) -> f64 {
            self.0 = (self.0.wrapping_mul(1_103_515_245).wrapping_add(12_345)) % 2_147_483_648;
            self.0 as f64 / 2_147_483_648.0
        }
        fn ri(&mut self, a: i64, b: i64) -> i64 {
            a + (self.next_r() * ((b - a + 1) as f64)).floor() as i64
        }
    }
    let mut rng = Rng(seed);

    struct Club {
        uid: i64,
        name: String,
        nat: i64,
        rep: i64,
    }
    let clubs: Vec<Club> = (0..400)
        .map(|i| Club {
            uid: 1000 + i,
            name: format!("Club {}{}", i, if i % 7 == 0 { " \"B\"" } else { "" }),
            nat: 1 + (i % 20),
            rep: rng.ri(500, 9500),
        })
        .collect();

    let cols = gen_scout_columns();
    let mut out = String::with_capacity(n * 240 + 1500);
    out.push_str(&cols.join(","));
    out.push('\n');

    let push_csv_field = |buf: &mut String, s: &str| {
        if s.contains('"') || s.contains(',') {
            buf.push('"');
            buf.push_str(&s.replace('"', "\"\""));
            buf.push('"');
        } else {
            buf.push_str(s);
        }
    };

    for i in 0..n {
        let age = rng.ri(15, 38);
        let ca = ((60.0 + rng.next_r() * 110.0 - if age < 19 { 40.0 } else { 0.0 }).round() as i64)
            .clamp(30, 195);
        let pa = (ca + if age < 21 { rng.ri(0, 70) } else { rng.ri(0, 10) }).min(200);
        let free = rng.next_r() < 0.04;
        let club = if free {
            None
        } else {
            Some(&clubs[rng.ri(0, (clubs.len() - 1) as i64) as usize])
        };
        let main = rng.ri(0, (ScoutPosKey::COUNT - 1) as i64) as usize;
        let lvl = ((ca as f64 / 10.0).round() as i64).clamp(1, 20);
        let mut att = [0i64; 52];
        for slot in &mut att {
            *slot = (lvl + rng.ri(-5, 5)).clamp(1, 20);
        }
        let mut posv = [0i64; ScoutPosKey::COUNT];
        for (j, slot) in posv.iter_mut().enumerate() {
            *slot = if j == main {
                20
            } else if rng.next_r() < 0.15 {
                rng.ri(10, 18)
            } else {
                rng.ri(1, 6)
            };
        }
        let end_y = 2027 + rng.ri(0, 4);
        let lf = rng.ri(3, 20);
        let rf = rng.ri(3, 20);
        let height = rng.ri(165, 198);
        let rep = rng.ri(100, 9000);
        let wrep = rng.ri(100, 9000);
        let on_loan = if rng.next_r() < 0.05 { "True" } else { "False" };
        let val = if free {
            String::new()
        } else {
            let mult = rng.ri(30, 120) * if age < 22 { 2 } else { 1 };
            (ca * ca * mult).to_string()
        };
        let wage = if free {
            String::new()
        } else {
            rng.ri(500, 150_000).to_string()
        };
        let end = if free {
            String::new()
        } else {
            format!("{end_y}-06-30")
        };

        use std::fmt::Write as _;
        let _ = write!(
            out,
            "{},{},Player {},{},{},{},{},{},,{},{},",
            100_000 + i,
            200_000 + i,
            i,
            age,
            1 + (i % 30),
            height,
            ca,
            pa,
            rep,
            wrep
        );
        if let Some(c) = club {
            let _ = write!(out, "{},", c.uid);
            push_csv_field(&mut out, &c.name);
            let _ = write!(out, ",{},{},", c.nat, c.rep);
        } else {
            out.push_str(",,,,");
        }
        let _ = write!(
            out,
            "{on_loan},,{val},{wage},{end},Rotation,{lf},{rf}"
        );
        for v in posv {
            let _ = write!(out, ",{v}");
        }
        for _ in 0..ScoutPers::COUNT {
            let _ = write!(out, ",{}", rng.ri(1, 20));
        }
        for v in att {
            let _ = write!(out, ",{v}");
        }
        out.push_str(",ok,0\n");
    }
    out
}

// ============================================================================
// Parity #1 - #10: Roles, Formations, Templates, Parsers, Retarget, FMF
// ============================================================================

#[test]
fn parity_01_02_roles_attrs_unique_ids_and_all_groups() {
    assert_eq!(ROLES.len(), 77);
    let mut ids = HashSet::new();
    for r in &ROLES {
        assert!(ids.insert(r.id), "duplicate role id: {}", r.id);
        for &a in r.key.iter().chain(r.pref.iter()) {
            assert_eq!(attr_meta(a).attr, a);
        }
    }
    for g in PosGroup::ALL {
        assert!(!roles_for(g, Phase::Ip, None, false).is_empty(), "IP {g:?}");
        assert!(!roles_for(g, Phase::Oop, None, false).is_empty(), "OOP {g:?}");
    }
}

#[test]
fn parity_03_all_17x17_formation_pairs_match_11_distinct_and_gk() {
    assert_eq!(FORMATIONS.len(), 17);
    for a in &FORMATIONS {
        assert_eq!(a.slots.len(), 11);
        for b in &FORMATIONS {
            let m = match_formations(a.name, b.name);
            assert_eq!(m.len(), 11);
            let distinct: HashSet<&str> = m.iter().map(|(_, o)| o.as_str()).collect();
            assert_eq!(distinct.len(), 11, "{} -> {}", a.name, b.name);
            let gk = m.iter().find(|(ip, _)| ip == "GK").unwrap();
            assert_eq!(gk.1, "GK", "GK mismatch {} -> {}", a.name, b.name);
        }
    }
}

#[test]
fn parity_04_templates_applied_valid_and_audited() {
    assert_eq!(TEMPLATES.len(), 8);
    for tp in &TEMPLATES {
        let tac = tactic_from_template(tp);
        assert_eq!(tac.slots.len(), 11);
        for s in &tac.slots {
            let a = slot_info(tp.ip, &s.ip_code).unwrap();
            let b = slot_info(tp.oop, &s.oop_code).unwrap();
            assert!(valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb));
            assert!(valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb));
            let &(_, want_ip, want_oop) = tp
                .roles
                .iter()
                .find(|&&(c, _, _)| c == s.ip_code)
                .unwrap();
            assert_eq!(s.ip, want_ip);
            assert_eq!(s.oop, want_oop);
        }
        for &(k, v) in tp.ti {
            let def = ti_by_key(k).unwrap();
            assert!(v < def.opts.len());
        }
        let au = audit_tactic(&tac, &[], Lang::En);
        assert!(au.overall <= 100);
    }
}

#[test]
fn parity_05_06_07_08_positions_squad_retarget_complexity() {
    // #5 positions
    let p1 = parse_positions("D (RL), WB (R)");
    assert!(p1.contains(&"D_LR".to_string()));
    assert!(p1.contains(&"D_LR:R".to_string()));
    assert!(p1.contains(&"D_LR:L".to_string()));
    assert!(p1.contains(&"WB_LR:R".to_string()));
    let p2 = parse_positions("AM (RLC), ST (C)");
    assert!(p2.contains(&"AMC".to_string()));
    assert!(p2.contains(&"AM_LR:L".to_string()));
    assert!(p2.contains(&"ST".to_string()));
    assert_eq!(parse_positions("DM, M (C)"), vec!["DM", "MC"]);
    assert_eq!(parse_positions("M/AM (R)"), vec!["M_LR", "M_LR:R", "AM_LR", "AM_LR:R"]);
    assert_eq!(parse_positions("GK"), vec!["GK"]);
    assert_eq!(parse_positions("DC, DM"), vec!["DC", "DM"]);
    assert_eq!(parse_positions("AMR"), vec!["AM_LR", "AM_LR:R"]);

    // #6 FM pipe table + range average
    let pipe = "| Name | Position | Age | Acc | Pac | Fin | Fir | Tec | Cmp | OtB | Str | Dri | Pas | Hea | Ant | Dec | Wor | Sta | Tea | Pos | Mar | Tck | Cnt |
| ------------------------------------- |
| Rui Matos | ST (C) | 24 | 14 | 13 | 15 | 14 | 13 | 14 | 15 | 14 | 12 | 11 | 14 | 14 | 12 | 12 | 13 | 12 | 6 | 5 | 6 | 11 |
| Tiago Lobo | D (C) | 29 | 10 | 11 | 5 | 10 | 9 | 13 | 6 | 15 | 7 | 11 | 15 | 13 | 13 | 12 | 13 | 13 | 14 | 15 | 14-16 | 14 |";
    let r = parse_squad(pipe, Lang::En);
    assert_eq!(r.players.len(), 2);
    assert_eq!(r.players[1].a.get(&Attr::Tck), Some(&15.0));
    assert!(r.players[0].pos.contains(&"ST".to_string()));
    assert_eq!(r.players[1].a.get(&Attr::Pos), Some(&14.0));

    // #7 retarget preserves roles
    let base = tactic_from_template(&TEMPLATES[0]);
    let ret = retarget_tactic(&base, "4-2-3-1 Wide", "4-4-1-1", Lang::En);
    assert_eq!(ret.slots.len(), 11);
    for s in &ret.slots {
        let a = slot_info(&ret.ip_form, &s.ip_code).unwrap();
        let b = slot_info(&ret.oop_form, &s.oop_code).unwrap();
        assert!(valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb));
        assert!(valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb));
    }

    // #8 complexity of gegenpress
    let cx = complexity_of(&tactic_from_template(&TEMPLATES[2]), Lang::En);
    assert!(cx.cx > 6.0);
    assert_eq!(cx.level, "Medium");
}

#[test]
fn parity_09_10_13_fmf_and_csv_fixtures() {
    // #9 cenas_v2.fmf
    let bytes = read_fixture_bytes("cenas_v2.fmf");
    let c = parse_fmf_container(&bytes);
    assert!(c.recognized);
    assert_eq!(c.blocks.len(), 3);
    assert_eq!(c.largest.map(|b| b.cipher), Some(707));

    let rep = inspect_fmf("cenas_v2.fmf", &bytes);
    assert!(rep.container.recognized);
    assert_eq!(rep.container.blocks.len(), 3);
    assert_eq!(rep.container.largest.map(|b| b.cipher), Some(707));

    // Embedded zlib stream inspection
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(b"Inside Forward 4-3-3 Passing Directness Positive Counter-Press")
        .unwrap();
    let mut zblob = vec![0u8; 4];
    zblob.extend_from_slice(&enc.finish().unwrap());
    let zrep = inspect_fmf("zlib_test.fmf", &zblob);
    assert_eq!(zrep.streams.len(), 1);
    assert!(zrep.decoded);
    assert!(zrep.matches.roles.contains(&"Inside Forward".to_string()));

    // #10 squad_en.csv & squad_de.csv
    for f in ["squad_en.csv", "squad_de.csv"] {
        let r = parse_squad(&read_fixture_text(f), Lang::En);
        assert_eq!(r.players.len(), 16, "{f}");
        assert_eq!(r.players[0].a.len(), 47, "{f}");
        assert_eq!(r.players[3].a.get(&Attr::Tck), Some(&13.5), "{f}");
        assert!(r.players[5].pos.contains(&"WB_LR:R".to_string()), "{f}");
        assert!(r.players[0].meta.uid.is_some(), "{f}");
        assert_eq!(r.warnings.len(), 0, "{f}");
    }

    // #13 fmsave_players.csv
    let fm = parse_squad(&read_fixture_text("fmsave_players.csv"), Lang::En);
    assert_eq!(fm.players.len(), 16);
    assert_eq!(fm.players[0].a.len(), 47);
    assert!(fm.players[5].pos.contains(&"D_LR:R".to_string()));
    assert!(fm.players[5].pos.contains(&"WB_LR".to_string()));
    assert_eq!(fm.players[0].meta.club.as_deref(), Some("Patro Eisden"));
    assert!(fm.players[11].pos.contains(&"ST".to_string()));
}

// ============================================================================
// Parity #11 + sim_canvas #1 & #4: 6-Phase Live Pitch Simulator
// ============================================================================

#[test]
fn parity_11_sim_model_all_templates_tis_roles_and_continuity() {
    // sim_canvas #1: lofted ball elevates at midpoint, ground pass stays at z == 0
    let loft = sim_ball_state(
        &[
            BallPoint { x: 20.0, y: 10.0, kind: BallKind::Loft },
            BallPoint { x: 80.0, y: 70.0, kind: BallKind::Loft },
        ],
        0.5,
    );
    let pass = sim_ball_state(
        &[
            BallPoint { x: 20.0, y: 10.0, kind: BallKind::Pass },
            BallPoint { x: 80.0, y: 70.0, kind: BallKind::Pass },
        ],
        0.5,
    );
    assert!(loft.z > 0.8, "loft z = {}", loft.z);
    assert_eq!(pass.z, 0.0);

    let check_finite = |tac: &Tactic, label: &str| {
        let m = sim_model(tac, Lang::En);
        assert_eq!(m.players.len(), 11, "{label}");
        for p in &m.players {
            for ph in SimPhase::ALL {
                let pt = p.pos[ph.index()];
                assert!(pt.x.is_finite() && pt.y.is_finite(), "NaN in {label} {ph:?}");
            }
        }
        for ph in SimPhase::ALL {
            let bp = &m.ball[ph.index()];
            assert!(bp.len() >= 2, "short ball path in {label} {ph:?}");
            for b in bp {
                assert!(b.x.is_finite() && b.y.is_finite(), "ball NaN in {label} {ph:?}");
            }
        }
        m
    };

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
        "dl_behaviour",
        "short_gk",
    ];

    for tp in &TEMPLATES {
        let base = tactic_from_template(tp);
        let m = check_finite(&base, tp.id);

        // Continuity across phase handoffs + no backward intra-phase jumps
        for (idx, pi) in PHASES.iter().enumerate() {
            if idx > 0 {
                let prev = PHASES[idx - 1].phase;
                let b_prev = &m.ball[prev.index()];
                let b_cur = &m.ball[pi.phase.index()];
                let last_a = b_prev.last().unwrap();
                let first_b = &b_cur[0];
                let dist = (last_a.x - first_b.x).hypot(last_a.y - first_b.y);
                if matches!(prev, SimPhase::Final | SimPhase::Loss | SimPhase::Oop) {
                    assert!(
                        dist <= 5.0,
                        "template {} {:?}->{:?} ball jump {dist:.1}",
                        tp.id,
                        prev,
                        pi.phase
                    );
                } else {
                    assert!(
                        first_b.y - last_a.y >= -10.0,
                        "template {} {:?}->{:?} backward jump dy={:.1}",
                        tp.id,
                        prev,
                        pi.phase,
                        first_b.y - last_a.y
                    );
                }
            }
            if matches!(pi.phase, SimPhase::Build | SimPhase::Prog | SimPhase::Final) {
                let arr = &m.ball[pi.phase.index()];
                for j in 1..arr.len() {
                    assert!(
                        arr[j].y >= arr[j - 1].y - 8.0,
                        "template {} {:?} intra-phase backward pass {:.1}->{:.1}",
                        tp.id,
                        pi.phase,
                        arr[j - 1].y,
                        arr[j].y
                    );
                }
            }
        }

        // Sweep all 12 animated TIs x all options
        for k in animated_tis {
            let def = ti_by_key(k).unwrap();
            for v in 0..def.opts.len() {
                let mut t2 = base.clone();
                t2.ti.insert(k.to_string(), v);
                check_finite(&t2, &format!("{}:{k}={v}", tp.id));
            }
        }
    }

    // All 77 roles
    for r in &ROLES {
        let (tac, idx) = test_tactic_for_role(r);
        let i = idx.unwrap_or_else(|| panic!("no slot for role {}", r.id));
        match r.phase {
            Phase::Ip => assert_eq!(tac.slots[i].ip, r.id),
            Phase::Oop => assert_eq!(tac.slots[i].oop, r.id),
        }
        check_finite(&tac, &format!("role:{}", r.id));
    }
}

// ============================================================================
// Parity #12, #14, #17, #19, #23: Insights, Set Pieces, Exact Slots, Synergy
// ============================================================================

#[test]
fn parity_12_14_17_19_23_engine_insights_synergy_and_partial_csv() {
    let me = parse_squad(&read_fixture_text("squad_en.csv"), Lang::En).players;
    let them = parse_squad(&read_fixture_text("squad_de.csv"), Lang::En).players;
    let fm = parse_squad(&read_fixture_text("fmsave_players.csv"), Lang::En).players;

    // #12 insights
    let mut tac = tactic_from_template(&TEMPLATES[0]);
    auto_pick_xi(&mut tac, &me);
    for s in &tac.slots {
        let p = me.iter().find(|q| Some(q.id.as_str()) == s.pid.as_deref()).unwrap();
        let grp = slot_info(&tac.ip_form, &s.ip_code).unwrap().group;
        for rec in recommend_pis(p, role_by_id(&s.ip), role_by_id(&s.oop), grp, Lang::En) {
            assert!(has_player_instruction(rec.pi));
            assert!(rec.why.len() > 5);
        }
    }
    let mut fast = Player { id: "x".into(), name: "Fast".into(), ..Player::default() };
    for (a, v) in [
        (Attr::Pac, 17.0), (Attr::Acc, 17.0), (Attr::Otb, 15.0), (Attr::Fin, 14.0),
        (Attr::Sta, 14.0), (Attr::Wor, 12.0), (Attr::Dri, 12.0), (Attr::Agi, 12.0),
    ] {
        fast.a.insert(a, v);
    }
    assert!(recommend_pis(&fast, role_by_id("poa"), role_by_id("cf_oop"), PosGroup::St, Lang::En)
        .iter()
        .any(|r| r.pi == "Attack the Defensive Line"));

    let rep = opponent_report(&tac, &me, &them, Lang::En);
    assert_eq!(rep.them.len(), 11);
    assert_eq!(rep.sectors.len(), 9);
    assert_eq!(rep.dangers.len(), 3);
    assert!(opponent_report(&tac, &[], &them, Lang::En)
        .sectors
        .iter()
        .all(|s| s.us_val.is_none()));
    assert_eq!(bench_and_successors(&tac, &me).len(), 11);
    assert_eq!(majority_club(&me).club.as_deref(), Some("Clube"));
    assert_eq!(surname("Kevin De Bruyne"), "De Bruyne");

    // #14 set-piece plan across 3 squads x 8 templates
    for (mine, opp) in [(&fm, Some(&me)), (&me, None), (&them, Some(&fm))] {
        let empty = Vec::new();
        let opp_slice = opp.map(Vec::as_slice).unwrap_or(&empty);
        for tp in &TEMPLATES {
            let t = tactic_from_template(tp);
            let plan = set_piece_plan(&t, mine, opp_slice, Lang::En).unwrap();
            let a_ids: HashSet<&str> = plan.attack.iter().map(|r| r.player_id.as_str()).collect();
            let d_ids: HashSet<&str> = plan.defend.iter().map(|r| r.player_id.as_str()).collect();
            assert_eq!(a_ids.len(), plan.attack.len());
            assert_eq!(d_ids.len(), plan.defend.len());
            assert!(plan.attack.iter().filter(|r| r.key.starts_with("back")).count() >= 2);
            let gks: HashSet<&str> = plan
                .xi
                .iter()
                .filter(|x| x.group == PosGroup::Gk)
                .map(|x| x.player_id.as_str())
                .collect();
            assert!(!plan
                .attack
                .iter()
                .chain(plan.defend.iter())
                .any(|r| gks.contains(r.player_id.as_str())));
            assert_eq!(plan.pens.len(), 3);
            assert_eq!(plan.caps.len(), 2);
        }
    }

    // #17 exact-slot rules
    let at = |f: &str, c: &str| -> Vec<&'static str> {
        let a = slot_info(f, c).unwrap();
        roles_for(a.group, Phase::Ip, Some(&a.code), a.wide_cb)
            .into_iter()
            .map(|r| r.id)
            .collect()
    };
    assert!(!at("4-3-3 DM Wide", "MCL").contains(&"b2bp"));
    assert!(!at("4-3-3 DM Wide", "MCL").contains(&"b2b"));
    assert!(!at("4-3-3 DM Wide", "DM").contains(&"b2b"));
    assert!(at("4-2-4 DM Wide", "DML").contains(&"b2b"));

    // #19 defensive synergy textbook cases
    let mk_oop = |roles: &[(&str, &str)]| {
        let mut t = new_tactic("4-3-3 DM Wide", "4-1-4-1 DM Wide", Lang::En);
        for s in &mut t.slots {
            if let Some(&(_, r)) = roles.iter().find(|&&(c, _)| c == s.oop_code) {
                s.oop = r.to_string();
            }
        }
        t
    };
    let sc = synergy_report(&mk_oop(&[("DCL", "scb"), ("DCR", "ccb")]), &[], Lang::En);
    let ts = synergy_report(&mk_oop(&[("DCL", "scb"), ("DCR", "scb")]), &[], Lang::En);
    let ns = synergy_report(
        &mk_oop(&[("DM", "pdm"), ("MCL", "pcm"), ("MCR", "pcm")]),
        &[],
        Lang::En,
    );
    assert_eq!(sc.phases[&SynergyPhase::Def].parts["cb"], 100);
    assert!(ts.phases[&SynergyPhase::Def].parts["cb"] < 50);
    assert!(ts.issues.iter().any(|i| i.key == "cb"));
    assert!(!sc.issues.iter().any(|i| i.key == "press" && (i.text.contains("scb") || i.text.contains("SCD"))));
    assert!(ns.phases[&SynergyPhase::Def].parts["screen"] < 50);
    assert!(ns.issues.iter().any(|i| i.key == "screen"));

    // #23 partial 4-attr CSV never panics
    let pos = [
        "GK", "DC", "DC", "D (L)", "D (R)", "DM", "M (C)", "M (C)", "AM (L)", "AM (R)", "ST",
        "ST", "DC", "M (C)", "ST", "GK",
    ];
    let mut csv = String::from("Name,Club,Position,Age,Finishing,Passing,Tackling,Pace\n");
    for (i, p) in pos.iter().enumerate() {
        csv.push_str(&format!("P{i},Club 5,{p},24,{},12,11,13\n", 10 + i % 8));
    }
    let ps = parse_squad(&csv, Lang::En).players;
    assert_eq!(ps.len(), 16);
    for tp in &TEMPLATES {
        let mut t = tactic_from_template(tp);
        auto_pick_xi(&mut t, &ps);
        let au = audit_tactic(&t, &ps, Lang::En);
        let _ = training_plan(&t, &ps, &au, Lang::En);
    }
    assert_eq!(suggest_tactics(&ps, Lang::En).len(), 8);
}

// ============================================================================
// Parity #15, #18, #20: Tactic Generator, Synergy Optimization, Game Plans
// ============================================================================

#[test]
fn parity_15_generator_3_squads_x_3_expectations() {
    for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
        let ps = parse_squad(&read_fixture_text(f), Lang::En).players;
        let must = vec![ps[3].id.clone()];
        let star = Some(ps[14].id.clone());
        for ex in [Expectation::Title, Expectation::Mid, Expectation::Low] {
            let cfg = GeneratorConfig {
                expect: ex,
                must: must.clone(),
                star: star.clone(),
                ..GeneratorConfig::default()
            };
            let res = generate_tactic(&ps, &cfg, Lang::En, |_| {}).unwrap();
            let all_cands: Vec<_> = std::iter::once(&res.main).chain(res.alts.iter()).collect();
            for cand in all_cands {
                let t = &cand.tactic;
                let pids: HashSet<&str> = t.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
                assert_eq!(pids.len(), 11, "11 distinct XI {f} {ex:?}");
                for s in &t.slots {
                    let a = slot_info(&t.ip_form, &s.ip_code).unwrap();
                    let b = slot_info(&t.oop_form, &s.oop_code).unwrap();
                    assert!(valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb));
                    assert!(valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb));
                }
                assert!(
                    !cand.audit.findings.iter().any(|q| q.sev == Severity::Crit),
                    "crit finding in generated {f} {ex:?}"
                );
                assert!(form_depth(&t.ip_form) >= form_depth(&t.oop_form) - 0.5);
                assert!(cand.reasons.len() >= TEAM_INSTRUCTIONS.len() - 2);
            }
            assert!(res.main.tactic.slots.iter().any(|s| s.pid.as_deref() == Some(must[0].as_str())));
            assert!(res.main.tactic.slots.iter().any(|s| s.pid.as_deref() == star.as_deref()));
            assert!(res.alts.len() >= 2);
            if ex == Expectation::Low {
                assert_eq!(res.main.tactic.ment, Mentality::Cautious);
            }
            assert_eq!(depth_chart(&res.main.tactic, &ps, 3).len(), 11);
            let _ = weak_spots(&res.main.tactic, &ps);
        }
    }
}

#[test]
fn parity_18_generator_synergy_on_beats_off() {
    for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
        let ps = parse_squad(&read_fixture_text(f), Lang::En).players;
        for ex in [Expectation::Title, Expectation::Low] {
            let off = generate_tactic(
                &ps,
                &GeneratorConfig { expect: ex, use_synergy: false, ..GeneratorConfig::default() },
                Lang::En,
                |_| {},
            )
            .unwrap()
            .main;
            let on = generate_tactic(
                &ps,
                &GeneratorConfig { expect: ex, use_synergy: true, ..GeneratorConfig::default() },
                Lang::En,
                |_| {},
            )
            .unwrap()
            .main;
            let sa = synergy_report(&off.tactic, &ps, Lang::En);
            let sb = synergy_report(&on.tactic, &ps, Lang::En);
            assert!(sb.score >= sa.score, "synergy regressed {f} {ex:?}: {} -> {}", sa.score, sb.score);
            assert!(
                sb.phases[&SynergyPhase::Def].score as i16 >= sa.phases[&SynergyPhase::Def].score as i16 - 4,
                "def dropped {f} {ex:?}"
            );
            assert!(on.fit_avg as i16 >= off.fit_avg as i16 - 3, "fit dropped {f} {ex:?}");
            assert!(sb.box_n >= 3, "box_n < 3 in {f} {ex:?}: {}", sb.box_n);
            assert!((1..=3).contains(&sb.creators), "creators out of 1..=3 in {f} {ex:?}");
        }
    }
}

#[test]
fn parity_20_game_plans_3_squads_x_5_scenarios() {
    let opp = parse_squad(&read_fixture_text("squad_de.csv"), Lang::En).players;
    for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
        let ps = parse_squad(&read_fixture_text(f), Lang::En).players;
        let base = generate_tactic(
            &ps,
            &GeneratorConfig { expect: Expectation::Mid, ..GeneratorConfig::default() },
            Lang::En,
            |_| {},
        )
        .unwrap()
        .main
        .tactic;

        for kind in GameStateKind::ALL {
            let plan = game_plan(&base, &ps, kind, &opp, Lang::En);
            for s in &plan.tactic.slots {
                let a = slot_info(&plan.tactic.ip_form, &s.ip_code).unwrap();
                let b = slot_info(&plan.tactic.oop_form, &s.oop_code).unwrap();
                assert!(valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb));
                assert!(valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb));
                assert!(s.pis.len() <= 3);
            }
            let in_xi: HashSet<&str> = plan.tactic.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
            let mut sub_names = HashSet::new();
            for sub in &plan.subs {
                let pl = ps.iter().find(|q| q.name == sub.in_name).unwrap();
                assert!(!in_xi.contains(pl.id.as_str()), "sub must come from bench");
                assert!(sub_names.insert(sub.in_name.as_str()), "duplicate sub");
            }
            let ib = game_intent(&base);
            let ia = game_intent(&plan.tactic);
            assert_eq!(ib, plan.intent_before);
            assert_eq!(ia, plan.intent_after);
            match kind {
                GameStateKind::Chase | GameStateKind::HomeWeak => {
                    assert!(ia.att > ib.att, "att must rise for {f} {kind:?}");
                    assert!(
                        plan.after.phases[&SynergyPhase::Att].score
                            >= plan.before.phases[&SynergyPhase::Att].score,
                        "att synergy must not drop for {f} {kind:?}"
                    );
                }
                GameStateKind::Protect | GameStateKind::AwayStrong => {
                    assert!(ia.sec > ib.sec, "sec must rise for {f} {kind:?}");
                    assert!(ia.att <= ib.att, "att must not rise for {f} {kind:?}");
                    assert!(
                        plan.after.phases[&SynergyPhase::Def].score as i16
                            >= plan.before.phases[&SynergyPhase::Def].score as i16 - 2,
                        "def synergy must not drop for {f} {kind:?}"
                    );
                }
                GameStateKind::AwayEqual => {}
            }
            if kind == GameStateKind::AwayStrong {
                assert_eq!(plan.ois.len(), 3);
            }
        }
    }
}

// ============================================================================
// Parity #21, #22, #24, #25: Scouting DB (5k + 100k Benchmark), Presets, Enrich
// ============================================================================

#[test]
fn parity_21_scouting_columns_5k_presets_filters_and_100k_benchmark() {
    let cols = gen_scout_columns();
    let col_set: HashSet<&str> = cols.iter().map(String::as_str).collect();
    for &c in &SCOUT_COLUMNS {
        assert!(col_set.contains(c), "missing SCOUT_COLUMN {c}");
    }

    let db = parse_scout_db(&gen_scout_db_csv(5_000, 42), Lang::En, |_, _| {}).unwrap();
    assert_eq!(db.players.len(), 5_000);
    assert!(db.attr_cols >= 45);
    let p0 = &db.players[0];
    assert!(p0.ca.unwrap_or(0) > 0);
    assert!(p0.pa.unwrap_or(0) >= p0.ca.unwrap_or(0));
    assert!(p0.pers_val(ScoutPers::Professionalism).unwrap_or(0) > 0);
    assert!(p0.hid_val(ScoutHid::Inj).unwrap_or(0) > 0);
    assert!(db.players.iter().any(|x| (x.pos_mask & PosGroup::St.bit()) != 0));
    assert!(db.players.iter().any(|x| x.club.contains("\"B\"")));

    let date = infer_game_date(&db.players).unwrap();
    assert_eq!(date.len(), 10);

    let squad = parse_squad(&read_fixture_text("fmsave_players.csv"), Lang::En).players;
    let mut tac = tactic_from_template(&TEMPLATES[0]);
    auto_pick_xi(&mut tac, &squad);
    let ctx = ScoutContext {
        date: Some(&date),
        my_club: Some(""),
        tactic: Some(&tac),
        squad: &squad,
        xi_ca: 120,
        ..ScoutContext::default()
    };

    let presets = scout_presets(&ctx, Lang::En);
    assert!(presets.len() >= 10);
    for pr in &presets {
        let res = scout_filter(&db, &pr.filter, &ctx, Lang::En);
        match pr.key {
            "cheap_wonder" => {
                assert!(!res.is_empty());
                assert!(res.iter().all(|m| {
                    let x = &db.players[m.index];
                    x.age.unwrap_or(99) <= 19
                        && x.pa.unwrap_or(0) >= 150
                        && x.value.is_none_or(|v| v <= pr.filter.value_max.unwrap())
                }));
            }
            "reach" => {
                assert!(res.iter().all(|m| {
                    m.likely.unwrap_or(0) >= 70 && db.players[m.index].age.unwrap_or(99) <= 20
                }));
            }
            "expiring" => {
                assert!(!res.is_empty());
                assert!(res.iter().all(|m| {
                    months_until(&db.players[m.index].end, &date).is_some_and(|mo| mo <= 6.0)
                }));
            }
            "free" => {
                assert!(!res.is_empty());
                assert!(res.iter().all(|m| db.players[m.index].club.is_empty()));
            }
            "upgrade" => {
                assert!(!res.is_empty());
                assert!(res.first().unwrap().gain >= res.last().unwrap().gain);
            }
            "hidden" => {
                assert!(res.iter().all(|m| {
                    db.players[m.index].club_rep.is_none_or(|r| r <= 4000)
                }));
            }
            _ => {}
        }
    }

    // Combined filter
    let filt = ScoutFilter {
        pos: Some(PosGroup::Dc),
        pos_level: PosLevel::Nat,
        attrs: vec![
            ScoutAttrFilter { k: Some(Attr::Hea), min: Some(12) },
            ScoutAttrFilter { k: Some(Attr::Pac), min: Some(12) },
        ],
        height_min: Some(185),
        sort: ScoutSort::Ca,
        ..ScoutFilter::default()
    };
    let r2 = scout_filter(&db, &filt, &ctx, Lang::En);
    assert!(r2.iter().all(|m| {
        let x = &db.players[m.index];
        x.has_nat_group(PosGroup::Dc)
            && x.attrs[Attr::Hea.index()] >= 12
            && x.attrs[Attr::Pac.index()] >= 12
            && x.height.unwrap_or(0) >= 185
    }));

    let star_p = db.players.iter().find(|x| x.ca.unwrap_or(0) > 150).unwrap();
    let gain = best_gain(star_p, &tac, &squad).unwrap();
    assert!(!gain.code.is_empty());

    // 100k-player benchmark
    let big_csv = gen_scout_db_csv(100_000, 42);
    let t0 = Instant::now();
    let db2 = parse_scout_db(&big_csv, Lang::En, |_, _| {}).unwrap();
    let parse_ms = t0.elapsed().as_millis();
    let t1 = Instant::now();
    let _ = scout_filter(
        &db2,
        &ScoutFilter { age_max: Some(21), pa_min: Some(150), ..ScoutFilter::default() },
        &ctx,
        Lang::En,
    );
    let filter_ms = t1.elapsed().as_millis();
    assert_eq!(db2.players.len(), 100_000);
    assert!(parse_ms < 15_000, "100k parse too slow: {parse_ms} ms");
    assert!(filter_ms < 500, "100k filter too slow: {filter_ms} ms");
}

#[test]
fn parity_22_24_25_join_likelihood_unknown_value_and_enrich() {
    // #22 join_likelihood monotonicity
    let mk_sp = |club: &str, club_rep: Option<i32>, rep: Option<i32>, loyalty: u8| ScoutPlayer {
        id: "s1".into(),
        uid: None,
        uniq: None,
        name: "Test".into(),
        name_lower: "test".into(),
        age: Some(23),
        nat: Some(1),
        height: Some(180),
        ca: Some(130),
        pa: Some(145),
        pa_range: None,
        rep,
        wrep: None,
        club: club.into(),
        club_lower: club.to_lowercase(),
        club_uid: String::new(),
        club_nat: None,
        club_rep,
        loan: false,
        loan_from: String::new(),
        value: Some(1_000_000),
        wage: Some(10_000),
        end: "2027-06-30".into(),
        status: "Rotation".into(),
        foot: None,
        lf: None,
        rf: None,
        attrs: [10; Attr::COUNT],
        pos_r: [1; ScoutPosKey::COUNT],
        pos_mask: PosGroup::Mc.bit(),
        nat_pos_mask: PosGroup::Mc.bit() as u16,
        pos_raw: "MC".into(),
        pers: [12, 12, 12, loyalty, 12, 12, 12, 12],
        hid: [12; ScoutHid::COUNT],
    };
    let base = mk_sp("Other", Some(5000), Some(4000), 10);
    let ctx_rep = |r: i32| ScoutContext {
        my_rep: Some(r),
        date: Some("2027-01-15"),
        ..ScoutContext::default()
    };
    assert!(join_likelihood(&base, &ScoutContext { date: Some("x"), ..ScoutContext::default() }, Lang::En).is_none());
    assert!(
        join_likelihood(&base, &ctx_rep(8000), Lang::En).unwrap().score
            > join_likelihood(&base, &ctx_rep(3000), Lang::En).unwrap().score
    );
    let free = mk_sp("", None, Some(4000), 10);
    assert!(
        join_likelihood(&free, &ctx_rep(4000), Lang::En).unwrap().score
            > join_likelihood(&base, &ctx_rep(4000), Lang::En).unwrap().score
    );
    let high_rep = mk_sp("Other", Some(5000), Some(9000), 10);
    assert!(
        join_likelihood(&high_rep, &ctx_rep(5000), Lang::En).unwrap().score
            < join_likelihood(&base, &ctx_rep(5000), Lang::En).unwrap().score
    );
    let loyal = mk_sp("Other", Some(5000), Some(4000), 19);
    let disloyal = mk_sp("Other", Some(5000), Some(4000), 5);
    assert!(
        join_likelihood(&loyal, &ctx_rep(5000), Lang::En).unwrap().score
            < join_likelihood(&disloyal, &ctx_rep(5000), Lang::En).unwrap().score
    );
    let j = join_likelihood(&base, &ctx_rep(7000), Lang::En).unwrap();
    assert!(j.score <= 100 && !j.why.is_empty());

    // #22 interested preset + #24 unknown value excluded from bargains
    let mut db = parse_scout_db(&gen_scout_db_csv(4_000, 42), Lang::En, |_, _| {}).unwrap();
    let mut stars = HashSet::new();
    for p in db.players.iter_mut().take(60) {
        p.value = None;
        p.ca = Some(185);
        p.pa = Some(195);
        p.age = Some(22);
        p.club_rep = Some(9500);
        if p.club.is_empty() {
            p.club = "Big Club".into();
            p.club_lower = "big club".into();
        }
        stars.insert(p.id.clone());
    }
    let ctx = ScoutContext {
        date: Some("2027-01-15"),
        my_rep: Some(6000),
        my_club: Some(""),
        xi_ca: 120,
        ..ScoutContext::default()
    };
    let presets = scout_presets(&ctx, Lang::En);
    let interested = presets.iter().find(|p| p.key == "interested").unwrap();
    let ires = scout_filter(&db, &interested.filter, &ctx, Lang::En);
    assert!(!ires.is_empty());
    assert!(ires.iter().all(|m| m.join.unwrap_or(0) >= 70));
    assert!(ires.windows(2).all(|w| w[0].join >= w[1].join));

    for k in ["bargain", "cheap_wonder", "hidden"] {
        let pr = presets.iter().find(|p| p.key == k).unwrap();
        let res = scout_filter(&db, &pr.filter, &ctx, Lang::En);
        assert!(!res.iter().any(|m| stars.contains(&db.players[m.index].id)), "{k} leaked unknown value");
        assert!(res.iter().all(|m| {
            let x = &db.players[m.index];
            x.club.is_empty() || x.value.is_some_and(|v| v <= pr.filter.value_max.unwrap())
        }));
    }

    // #25 enrich_from_scout
    let csv25 = gen_scout_db_csv(25, 11);
    let mut ps = parse_squad(&csv25, Lang::En).players;
    assert_eq!(enrich_from_scout(&mut ps, &csv25), 25);
    assert!(ps.iter().all(|p| p.ca.unwrap_or(0.0) > 0.0 && p.pa >= p.ca));
    assert!(ps.iter().all(|p| p.pers.len() == 8 && p.hid.len() == 5));
    assert!(ps.iter().any(|p| p.pos.contains(&"ST".to_string())));

    let f9_csv = "Name,Position,Age,Finishing\nA,ST,22,14\nB,DC,25,8";
    let mut f9 = parse_squad(f9_csv, Lang::En).players;
    assert_eq!(enrich_from_scout(&mut f9, f9_csv), 0);
}

// ============================================================================
// End-to-End FM27 Bridge Integration Check
// ============================================================================

struct MockBridge {
    log: Vec<String>,
    tactics_count: usize,
}

impl BridgeTransport for MockBridge {
    fn call(&mut self, cmd: &str, _timeout_ms: u64) -> Result<String, BridgeError> {
        self.log.push(cmd.to_string());
        if cmd == "PING" {
            return Ok("PONG ready".to_string());
        }
        if cmd == "VERSION" {
            return Ok("OK 0.4.2 FM27".to_string());
        }
        if cmd == "REFS" {
            return Ok(format!(
                "OK session=1 {}={}={};{}={}",
                FmbIds::REF_TEAM_TRAINING,
                FmbIds::REF_TEAM_TRAINING_MANAGER,
                77,
                FmbIds::REF_TEAM_TRAINING_TEAM,
                555
            ));
        }
        if cmd.starts_with("READ ") {
            // Real wire shape: lists are `[[idx, rec], ...]`, records are `{"entries": [[key, type, val], ...]}`.
            let sub_items: Vec<serde_json::Value> = (0..self.tactics_count)
                .map(|i| {
                    serde_json::json!([i, {
                        "entries": [[FmbIds::KEY_PAYLOAD, "s", format!("Tactic {}", i + 1)]]
                    }])
                })
                .collect();
            let menu = serde_json::json!([[0, {
                "entries": [
                    [FmbIds::TACTIC_MENU_ITEM, "r", {
                        "entries": [[FmbIds::KEY_NUMBER, "n", FmbIds::TACTIC_MENU_COPY]]
                    }],
                    [FmbIds::TACTIC_MENU_SUB, "l", sub_items]
                ]
            }]]);
            return Ok(format!("OK {menu}"));
        }
        Ok("OK".to_string())
    }

    fn sleep_ms(&self, _ms: u64) {}
}

#[test]
fn integration_fmbridge_status_context_tactics_and_id_tables() {
    // Verify FmbIds maps all 27 TIs and core roles
    for ti in &TEAM_INSTRUCTIONS {
        assert!(FmbIds::ti_spec(ti.key).is_some(), "missing FmbIds TI {}", ti.key);
    }
    assert!(FmbIds::role_bits("bpgk").is_some());
    assert!(FmbIds::role_bits("if_ip").is_some());
    assert!(FmbIds::role_bits("swk").is_some());

    let mut client = FmbClient::new(
        MockBridge {
            log: Vec::new(),
            tactics_count: 3,
        },
        Lang::En,
    );
    let st = client.status(true);
    assert!(st.ok && st.ready);
    assert_eq!(st.version, "0.4.2");

    let ctx = client.context().unwrap();
    assert_eq!(ctx.manager, 77);
    assert_eq!(ctx.team, 555);

    // Refuses creating a 4th tactic when 3 already exist
    let err = client.create_tactic(|_| {}).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("3"), "expected 3-tactic limit message, got: {msg}");

    // Ensure ApplyTacticOptions compiles & defaults cleanly
    let _opts = ApplyTacticOptions::default();
}

// ============================================================================
// Static Bidirectional EN / PT i18n Zero-Leak & pt_terms Audit
// ============================================================================

const PT_LEAK_WORDS: &[&str] = &[
    "de", "da", "do", "das", "dos", "para", "com", "sem", "os", "ao", "aos", "um", "uma", "que",
    "por", "mais", "menos", "em", "na", "nos", "nas", "se", "ou", "já", "não", "ver", "ler",
    "abrir", "fechar", "guardar", "nova", "novo", "limpar", "usar", "gerar", "plantel", "jogador",
    "jogadores", "equipa", "tática", "táticas", "treino", "bíblia", "papel", "papéis",
    "instrução", "instruções", "posição", "posições", "ligar", "porta", "jogos", "golos",
    "ficheiro", "carregar", "escolher", "importar", "exportar", "apagar", "tudo", "todos", "cada",
    "quando", "onde", "como", "isso", "isto", "este", "esta", "aqui", "sobre", "entre", "até",
    "após", "antes", "depois", "ainda", "também", "só", "muito", "pouco", "bem", "melhor", "pior",
    "alto", "baixo", "médio", "média", "atrás", "frente", "meio", "lado", "flanco", "corredor",
    "área", "bola", "linha", "linhas", "pressão", "bloco", "defesa", "ataque", "avançado",
    "avançados", "centrais", "lateral", "laterais", "extremo", "extremos", "médios", "fica",
    "ficam", "faz", "pode", "deve", "vai", "sobe", "entra", "sai", "joga", "início", "definições",
    "dados", "tema", "sistema", "claro", "escuro", "idioma", "cópia", "segurança", "restaurar",
    "versão", "ligado", "desligado", "automático", "encontrado", "olá", "vamos", "próximo",
    "três", "passos", "computador", "corrigir", "primeiro", "completo", "analisador", "relatório",
    "adversário", "mentalidade", "formação", "estilo", "campo", "animado", "bolas", "paradas",
    "ficha", "construção", "progressão", "último", "terço", "perda", "recuperação", "nome",
    "objectivo", "estrela", "titulares", "jovens", "opções", "recomendada", "alternativa",
    "adequação", "equilíbrio", "diagnóstico", "marcadores", "cantos", "livres", "penáltis",
    "capitão", "ideias", "ofensivo", "defensivo", "misto", "experiências", "motor", "ligação",
    "procurar", "veredicto", "sementes", "referência", "vitórias", "pontos", "remates", "posse",
    "semana", "dia", "prioridades", "mentoria", "foco", "chave", "preferidos", "comportamento",
    "agora", "contrato", "terminar", "plano", "aplicado",
];

fn has_pt_diacritics_or_suffix(s: &str) -> bool {
    if s.chars().any(|c| {
        matches!(
            c,
            'ã' | 'õ' | 'ç' | 'â' | 'ê' | 'ô' | 'à' | 'Ã' | 'Õ' | 'Ç' | 'Â' | 'Ê' | 'Ô' | 'À'
        )
    }) {
        return true;
    }
    let low = s.to_lowercase();
    low.contains("ção") || low.contains("ões")
}

fn extract_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for ch in s.to_lowercase().chars() {
        if ch.is_alphabetic() {
            cur.push(ch);
        } else if !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

fn build_pt_only_vocab() -> HashSet<String> {
    let mut pt_w: HashSet<String> = PT_LEAK_WORDS.iter().map(|s| s.to_string()).collect();
    let mut en_w: HashSet<String> = HashSet::new();
    for &(pt, en) in EN_DICT_ENTRIES {
        for w in extract_words(pt) {
            if w.chars().count() >= 3 {
                pt_w.insert(w);
            }
        }
        for w in extract_words(en) {
            if w.chars().count() >= 3 {
                en_w.insert(w);
            }
        }
    }
    for &(en, _pt) in PT_TERMS {
        for w in extract_words(en) {
            en_w.insert(w);
        }
    }
    let allow: HashSet<&str> = [
        "fmsave", "steam", "tactics", "lab", "fm27", "fm26", "view", "exporter", "pypi",
        "github", "windows", "csv", "json", "html", "xml", "tsv", "zstd", "aes", "gcm", "nonce",
        "fmf", "uvx", "winget", "astral", "clean", "slate", "box", "ok", "pwb", "iwb",
        "architect", "wide", "flat", "diamond", "narrow", "dm", "wb", "deep", "player", "club",
        "country", "test", "tactic", "db", "total", "local", "normal", "geral", "zonal", "ideal",
        "final", "lateral", "central", "vertical", "mental", "regular", "natural", "original",
        "real", "zero", "extra", "contra", "duo", "duo", "ultra", "base", "clube", "patro",
        "eisden", "no", "in", "on", "to", "a", "or",
    ]
    .into_iter()
    .collect();
    pt_w.retain(|w| !en_w.contains(w) && !allow.contains(w.as_str()));
    pt_w
}

fn is_pt_leak(s: &str, ptw: &HashSet<String>) -> bool {
    let mut cleaned = s.to_string();
    for i in (0..30).rev() {
        cleaned = cleaned.replace(&format!("Jogador {i}"), "Player");
        cleaned = cleaned.replace(&format!("Atleta {i}"), "Player");
    }
    let trimmed = cleaned.trim();
    if matches!(
        trimmed,
        "o" | "os" | "as" | "do" | "da" | "dos" | "das" | "e" | "ou" | "na"
    ) {
        return true;
    }
    if has_pt_diacritics_or_suffix(trimmed) {
        return true;
    }
    let words = extract_words(trimmed);
    let hits = words.iter().filter(|w| ptw.contains(w.as_str())).count();
    let threshold = if words.len() <= 3 { 1 } else { 2 };
    hits >= threshold
}

#[test]
fn i18n_en_zero_portuguese_leaks_across_all_core_outputs() {
    let ptw = build_pt_only_vocab();

    // Detector self-test (must catch real PT leaks!)
    for probe in [
        "Ler o save agora",
        "Contrato a terminar",
        "Plano aplicado à tática",
        "Jogadores livres",
        "Misto",
    ] {
        assert!(is_pt_leak(probe, &ptw), "detector missed probe: {probe}");
    }

    let mut leaks: Vec<(String, String)> = Vec::new();
    let mut check = |origin: &str, text: &str| {
        if is_pt_leak(text, &ptw) {
            leaks.push((origin.to_string(), text.to_string()));
        }
    };

    // 1. Static knowledge base translated via t(Lang::En, ...)
    for r in &ROLES {
        check(&format!("role:{}:name", r.id), &role_name(r, Lang::En));
        check(&format!("role:{}:desc", r.id), &t(Lang::En, r.desc_pt));
        check(&format!("role:{}:tip", r.id), &t(Lang::En, r.tip_pt));
    }
    for ti in &TEAM_INSTRUCTIONS {
        check(&format!("ti:{}:name", ti.key), &ti_name(ti, Lang::En));
        check(&format!("ti:{}:area", ti.key), &t(Lang::En, ti.area_pt));
        check(&format!("ti:{}:desc", ti.key), &t(Lang::En, ti.desc_pt));
        check(&format!("ti:{}:pro", ti.key), &t(Lang::En, ti.pro_pt));
        check(&format!("ti:{}:con", ti.key), &t(Lang::En, ti.con_pt));
        for opt in ti.opts {
            check(&format!("ti:{}:opt", ti.key), &ti_opt_name(opt, Lang::En));
        }
    }
    for pi in &PLAYER_INSTRUCTIONS {
        check(&format!("pi:{}:name", pi.name), &pi_name(pi.name, Lang::En));
        check(&format!("pi:{}:group", pi.name), &t(Lang::En, pi.group_pt));
        check(&format!("pi:{}:desc", pi.name), &t(Lang::En, pi.desc_pt));
        check(&format!("pi:{}:tip", pi.name), &t(Lang::En, pi.tip_pt));
    }
    for tw in &TACTICAL_TWEAKS {
        check(&format!("tweak:{}:desc", tw.name), &t(Lang::En, tw.desc_pt));
        check(&format!("tweak:{}:when", tw.name), &t(Lang::En, tw.when_pt));
    }
    for tr in &TRAINING_AREAS {
        check(&format!("train:{}:session", tr.key), &t(Lang::En, tr.session_pt));
    }
    for m in Mentality::ALL {
        check(&format!("ment:{m:?}:name"), &mentality_name(m, Lang::En));
        check(&format!("ment:{m:?}:info"), &mentality_info(m, Lang::En));
    }
    for tp in &TEMPLATES {
        check(&format!("tpl:{}:name", tp.id), &t(Lang::En, tp.name_pt));
        check(&format!("tpl:{}:desc", tp.id), &t(Lang::En, tp.desc_pt));
    }
    for sp in &STYLE_PROFILES {
        check(&format!("style:{}:name", sp.id), &t(Lang::En, sp.name_pt));
    }
    for g in PosGroup::ALL {
        check(&format!("group:{g:?}"), &group_name(g, Lang::En));
    }
    for a in Attr::ALL {
        check(&format!("attr:{a:?}"), attr_name(a, Lang::En));
    }

    // 2. Dynamic engine, simulator, and planning outputs in Lang::En
    let me = parse_squad(&read_fixture_text("squad_en.csv"), Lang::En).players;
    let them = parse_squad(&read_fixture_text("squad_de.csv"), Lang::En).players;

    for tp in &TEMPLATES {
        let mut tac = tactic_from_template(tp);
        auto_pick_xi(&mut tac, &me);

        // Audit findings
        let au = audit_tactic(&tac, &me, Lang::En);
        check("audit:cx_level", &au.complexity.level);
        for (lbl, _) in &au.complexity.parts {
            check("audit:cx_part", lbl);
        }
        for f in &au.findings {
            check("audit:title", &f.title);
            check("audit:text", &f.text);
            if let Some(fix) = &f.fix {
                check("audit:fix", fix);
            }
        }

        // Training plan
        let tplan = training_plan(&tac, &me, &au, Lang::En);
        for a in &tplan.areas {
            for w in &a.why {
                check("train:why", w);
            }
        }

        // Synergy
        let syn = synergy_report(&tac, &me, Lang::En);
        for l in &syn.links {
            check("syn:link", &l.why);
        }
        for iss in &syn.issues {
            check("syn:issue:text", &iss.text);
            check("syn:issue:fix", &iss.fix);
        }

        // Simulator + Phase captions
        let sm = sim_model(&tac, Lang::En);
        for ov in &sm.overlays {
            check("sim:overlay", &ov.label);
        }
        for ph in SimPhase::ALL {
            let cap = phase_text(&tac, ph, &sm, Lang::En);
            check(&format!("sim:cap:{ph:?}"), &cap.text);
            for (_, _, note) in &cap.notes {
                check(&format!("sim:note:{ph:?}"), note);
            }
        }

        // Set pieces
        let sp = set_piece_plan(&tac, &me, &them, Lang::En).unwrap();
        check("sp:system", &t(Lang::En, &sp.system));
        for r in sp.attack.iter().chain(sp.defend.iter()) {
            check("sp:label", &r.label);
            check("sp:why", &r.why);
        }
        for tip in &sp.tips {
            check("sp:tip", &tip.text);
        }
    }

    // PIs & opponent report
    let mut tac0 = tactic_from_template(&TEMPLATES[0]);
    auto_pick_xi(&mut tac0, &me);
    for s in &tac0.slots {
        let p = me.iter().find(|q| Some(q.id.as_str()) == s.pid.as_deref()).unwrap();
        let grp = slot_info(&tac0.ip_form, &s.ip_code).unwrap().group;
        for rec in recommend_pis(p, role_by_id(&s.ip), role_by_id(&s.oop), grp, Lang::En) {
            check("pi:rec", &rec.why);
        }
    }
    let orep = opponent_report(&tac0, &me, &them, Lang::En);
    for s in &orep.sectors {
        check("opp:sector", &t(Lang::En, s.name_pt));
    }
    for item in &orep.plan {
        check("opp:plan", &item.why);
    }

    // Generator + Game Plans
    let gen_res = generate_tactic(
        &me,
        &GeneratorConfig { expect: Expectation::Mid, ..GeneratorConfig::default() },
        Lang::En,
        |_| {},
    )
    .unwrap();
    check("gen:ment_why", &gen_res.main.ment_why);
    for (k, why) in &gen_res.main.reasons {
        check(&format!("gen:ti:{k}"), why);
    }
    for kind in GameStateKind::ALL {
        let gp = game_plan(&gen_res.main.tactic, &me, kind, &them, Lang::En);
        check("gp:title", &gp.title);
        check("gp:desc", &gp.desc);
        for ch in &gp.changes {
            check("gp:change:label", &ch.label);
            check("gp:change:why", &ch.why);
        }
        for note in &gp.notes {
            check("gp:note", note);
        }
    }

    // Scouting presets + join likelihood
    let ctx = ScoutContext {
        date: Some("2027-01-15"),
        my_rep: Some(6500),
        tactic: Some(&tac0),
        squad: &me,
        xi_ca: 120,
        ..ScoutContext::default()
    };
    for pr in scout_presets(&ctx, Lang::En) {
        check(&format!("scout:preset:{}:name", pr.key), &pr.name);
        check(&format!("scout:preset:{}:desc", pr.key), &pr.desc);
    }

    assert!(
        leaks.is_empty(),
        "Found {} untranslated PT leak(s) in EN mode:\n{}",
        leaks.len(),
        leaks
            .iter()
            .take(25)
            .map(|(o, s)| format!("  [{o}] «{s}»"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn i18n_pt_terms_translates_all_roles_tis_pis_mentalities_and_preserves_sheet_en_override() {
    for r in &ROLES {
        let pt = role_name(r, Lang::Pt);
        assert!(!pt.is_empty());
        assert_ne!(pt, r.name, "role {} ({}) untranslated in PT", r.id, r.name);
        // Sheet EN-terms mode keeps raw FM English name
        assert_eq!(role_name(r, Lang::En), r.name);
    }
    assert_eq!(role_name(role_by_id("am_oop").unwrap(), Lang::Pt), "Médio Ofensivo");
    assert_eq!(role_name(role_by_id("am_ip").unwrap(), Lang::Pt), "Médio Entrelinhas");

    for ti in &TEAM_INSTRUCTIONS {
        let pt = ti_name(ti, Lang::Pt);
        assert_ne!(pt, ti.name, "TI {} untranslated in PT", ti.name);
        assert_eq!(ti_name(ti, Lang::En), ti.name);
    }
    for pi in &PLAYER_INSTRUCTIONS {
        let pt = pi_name(pi.name, Lang::Pt);
        assert_ne!(pt, pi.name, "PI {} untranslated in PT", pi.name);
        assert_eq!(pi_name(pi.name, Lang::En), pi.name);
    }
    for m in Mentality::ALL {
        let pt = mentality_name(m, Lang::Pt);
        assert_ne!(pt, m.as_str(), "Mentality {} untranslated in PT", m.as_str());
        assert_eq!(mentality_name(m, Lang::En), m.as_str());
    }

    // Boundary safety & default tactic name translation
    assert_eq!(pt_terms("Yes"), "Sim");
    assert_eq!(pt_terms("No"), "Não");
    assert_eq!(pt_terms("Nova tática"), "Nova tática");
    let tac = new_tactic("4-3-3 DM Wide", "4-1-4-1 DM Wide", Lang::Pt);
    assert_eq!(t_name(&tac, Lang::Pt), "Nova tática");
    assert_eq!(t_name(&tac, Lang::En), "New tactic");

    // Verify all 17 formations exist cleanly
    assert_eq!(
        FORMATIONS.iter().filter(|f| formation_slots(f.name).is_some()).count(),
        17
    );
}
