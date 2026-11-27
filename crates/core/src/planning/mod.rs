//! Planning engine (Module C): Tactic Generator, Match-State Game Plans, and Scouting DB.

pub mod gameplans;
pub mod generator;
pub mod scouting;

pub use gameplans::{game_intent, game_plan, scenario_meta};
pub use generator::{
    depth_chart, form_depth, formation_pairs, generate_tactic, generate_tis, pair_cost, weak_spots,
    FormationPair, GeneratedTis,
};
pub use scouting::{
    best_gain, best_roles_any, enrich_from_scout, game_date_from_info, infer_game_date,
    join_likelihood, months_until, pa_range, parse_nations, parse_scout_db, parse_scout_sync,
    reach_likelihood, reach_likelihood_player, scout_filter, scout_presets, split_csv_line,
    SCOUT_COLUMNS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{slot_info, valid_role, TEAM_INSTRUCTIONS, TEMPLATES};
    use crate::engine::hungarian::{auto_pick_xi, tactic_from_template};
    use crate::engine::parsers::parse_squad;
    use crate::engine::synergy::synergy_report;
    use crate::types::{
        Attr, AttrLookup, Expectation, GameStateKind, GeneratorConfig, Lang, Mentality, Phase,
        PosGroup, PosLevel, ScoutAttrFilter, ScoutContext, ScoutFilter, ScoutHid, ScoutPers,
        ScoutPlayer, ScoutPosKey, ScoutSort, Severity, SynergyPhase,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::time::Instant;

    fn fixture(name: &str) -> String {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name);
        std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("missing fixture {}: {e}", p.display()))
    }

    /// Deterministic LCG CSV generator matching `tests/gen_scout_db.cjs` (same seed & formulas).
    fn gen_scout_db_csv(n: usize, seed: u64) -> String {
        const ATTR_NAMES: [&str; 52] = [
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

        let mut header: Vec<String> = vec![
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
            header.push(format!("positions_{p}"));
        }
        for p in ScoutPers::NAMES {
            header.push(format!("personality_{p}"));
        }
        for a in ATTR_NAMES {
            header.push(format!("attributes_{a}"));
        }
        header.push("transfer_value_state".into());
        header.push("transfer_value_state_code".into());

        let mut out = String::with_capacity(n * 280 + 2048);
        out.push_str(&header.join(","));

        let csv_esc = |s: &str| -> String {
            if s.contains('"') || s.contains(',') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };

        for i in 0..n {
            out.push('\n');
            let age = rng.ri(15, 38);
            let ca = ((60.0 + rng.next_r() * 110.0 - if age < 19 { 40.0 } else { 0.0 }).round()
                as i64)
                .clamp(30, 195);
            let pa = (ca + if age < 21 { rng.ri(0, 70) } else { rng.ri(0, 10) }).min(200);
            let free = rng.next_r() < 0.04;
            let c = if free {
                None
            } else {
                Some(&clubs[rng.ri(0, (clubs.len() - 1) as i64) as usize])
            };
            let main_pos = rng.ri(0, (ScoutPosKey::COUNT - 1) as i64) as usize;
            let lvl = ((ca as f64 / 10.0).round() as i64).clamp(1, 20);
            let mut atts = [0i64; 52];
            for slot in &mut atts {
                *slot = (lvl + rng.ri(-5, 5)).clamp(1, 20);
            }
            let mut posv = [0i64; ScoutPosKey::COUNT];
            for (j, slot) in posv.iter_mut().enumerate() {
                *slot = if j == main_pos {
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
            let rep_cur = rng.ri(100, 9000);
            let rep_w = rng.ri(100, 9000);
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
            let end_s = if free {
                String::new()
            } else {
                format!("{end_y}-06-30")
            };

            use std::fmt::Write;
            let _ = write!(
                out,
                "{},{},Player {},{},{},{},{},{},,{},{},{},{},{},{},{},,{},{},{},Rotation,{},{}",
                100_000 + i,
                200_000 + i,
                i,
                age,
                1 + (i % 30),
                height,
                ca,
                pa,
                rep_cur,
                rep_w,
                c.map(|x| x.uid.to_string()).unwrap_or_default(),
                c.map(|x| csv_esc(&x.name)).unwrap_or_default(),
                c.map(|x| x.nat.to_string()).unwrap_or_default(),
                c.map(|x| x.rep.to_string()).unwrap_or_default(),
                on_loan,
                val,
                wage,
                end_s,
                lf,
                rf,
            );
            for v in posv {
                let _ = write!(out, ",{v}");
            }
            for _ in 0..ScoutPers::COUNT {
                let _ = write!(out, ",{}", rng.ri(1, 20));
            }
            for v in atts {
                let _ = write!(out, ",{v}");
            }
            out.push_str(",ok,0");
        }
        out
    }

    // Parity #15: generator produces valid, coherent tactics with reasons across 3 squads × 3 expectations
    #[test]
    fn parity_15_generator_3_squads_3_expectations() {
        for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
            let ps = parse_squad(&fixture(f), Lang::Pt).players;
            let must = vec![ps[3].id.clone()];
            let star = ps[14].id.clone();
            for exp in [Expectation::Title, Expectation::Mid, Expectation::Low] {
                let cfg = GeneratorConfig {
                    expect: exp,
                    must: must.clone(),
                    star: Some(star.clone()),
                    ..GeneratorConfig::default()
                };
                let r = generate_tactic(&ps, &cfg, Lang::Pt, |_| {}).expect("generator result");
                for x in std::iter::once(&r.main).chain(r.alts.iter()) {
                    let t = &x.tactic;
                    let pids: HashSet<&str> =
                        t.slots.iter().filter_map(|s| s.pid.as_deref()).collect();
                    assert_eq!(pids.len(), 11, "11 distinct XI in {f}");
                    for s in &t.slots {
                        let a = slot_info(&t.ip_form, &s.ip_code).unwrap();
                        let b = slot_info(&t.oop_form, &s.oop_code).unwrap();
                        assert!(valid_role(
                            &s.ip,
                            a.group,
                            Phase::Ip,
                            Some(&a.code),
                            a.wide_cb
                        ));
                        assert!(valid_role(
                            &s.oop,
                            b.group,
                            Phase::Oop,
                            Some(&b.code),
                            b.wide_cb
                        ));
                    }
                    let crits: Vec<_> = x
                        .audit
                        .findings
                        .iter()
                        .filter(|q| q.sev == Severity::Crit)
                        .collect();
                    assert!(crits.is_empty(), "crit finding in {f} {exp:?}: {crits:?}");
                    assert!(form_depth(&t.ip_form) >= form_depth(&t.oop_form) - 0.5);
                    assert!(x.reasons.len() >= TEAM_INSTRUCTIONS.len() - 2);
                }
                assert!(r
                    .main
                    .tactic
                    .slots
                    .iter()
                    .any(|s| s.pid.as_deref() == Some(must[0].as_str())));
                assert!(r
                    .main
                    .tactic
                    .slots
                    .iter()
                    .any(|s| s.pid.as_deref() == Some(star.as_str())));
                assert!(r.alts.len() >= 2);
                if exp == Expectation::Low {
                    assert_eq!(r.main.tactic.ment, Mentality::Cautious);
                }
            }
        }
    }

    // Parity #18: generator with synergy beats without synergy without sacrificing defence or fit
    #[test]
    fn parity_18_generator_synergy_improvement() {
        for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
            let ps = parse_squad(&fixture(f), Lang::Pt).players;
            for exp in [Expectation::Title, Expectation::Low] {
                let a = generate_tactic(
                    &ps,
                    &GeneratorConfig {
                        expect: exp,
                        use_synergy: false,
                        ..GeneratorConfig::default()
                    },
                    Lang::Pt,
                    |_| {},
                )
                .unwrap()
                .main;
                let b = generate_tactic(
                    &ps,
                    &GeneratorConfig {
                        expect: exp,
                        use_synergy: true,
                        ..GeneratorConfig::default()
                    },
                    Lang::Pt,
                    |_| {},
                )
                .unwrap()
                .main;
                let sa = synergy_report(&a.tactic, &ps, Lang::Pt);
                let sb = synergy_report(&b.tactic, &ps, Lang::Pt);
                assert!(
                    sb.score >= sa.score,
                    "synergy regressed {f} {exp:?}: {} -> {}",
                    sa.score,
                    sb.score
                );
                let da = sa.phases[&SynergyPhase::Def].score as i16;
                let db = sb.phases[&SynergyPhase::Def].score as i16;
                assert!(db >= da - 4, "defence dropped >4 {f} {exp:?}: {da} -> {db}");
                assert!(
                    b.fit_avg as i16 >= a.fit_avg as i16 - 3,
                    "fit dropped >3 {f} {exp:?}"
                );
                assert!(sb.box_n >= 3, "box_n >= 3 in {f} {exp:?}, got {}", sb.box_n);
                assert!(
                    (1..=3).contains(&sb.creators),
                    "creators 1..=3 in {f} {exp:?}, got {}",
                    sb.creators
                );
            }
        }
    }

    // Parity #20: game plans across 3 squads × 5 scenarios
    #[test]
    fn parity_20_gameplans_3_squads_5_scenarios() {
        let opp = parse_squad(&fixture("squad_de.csv"), Lang::Pt).players;
        for f in ["fmsave_players.csv", "squad_en.csv", "squad_de.csv"] {
            let ps = parse_squad(&fixture(f), Lang::Pt).players;
            let base = generate_tactic(&ps, &GeneratorConfig::default(), Lang::Pt, |_| {})
                .unwrap()
                .main
                .tactic;
            for sc in GameStateKind::ALL {
                let r = game_plan(&base, &ps, sc, &opp, Lang::Pt);
                for x in &r.tactic.slots {
                    let a = slot_info(&r.tactic.ip_form, &x.ip_code).unwrap();
                    let b = slot_info(&r.tactic.oop_form, &x.oop_code).unwrap();
                    assert!(valid_role(
                        &x.ip,
                        a.group,
                        Phase::Ip,
                        Some(&a.code),
                        a.wide_cb
                    ));
                    assert!(valid_role(
                        &x.oop,
                        b.group,
                        Phase::Oop,
                        Some(&b.code),
                        b.wide_cb
                    ));
                    assert!(x.pis.len() <= 3);
                }
                let in_xi: HashSet<&str> = r
                    .tactic
                    .slots
                    .iter()
                    .filter_map(|s| s.pid.as_deref())
                    .collect();
                let mut seen_subs = HashSet::new();
                for sub in &r.subs {
                    let p = ps
                        .iter()
                        .find(|q| q.name == sub.in_name)
                        .expect("sub exists");
                    assert!(!in_xi.contains(p.id.as_str()), "sub must come from bench");
                    assert!(seen_subs.insert(sub.in_name.as_str()), "duplicate sub");
                }
                if matches!(sc, GameStateKind::Chase | GameStateKind::HomeWeak) {
                    assert!(
                        r.intent_after.att > r.intent_before.att,
                        "att must rise for {f} {sc:?}"
                    );
                    assert!(
                        r.after.phases[&SynergyPhase::Att].score
                            >= r.before.phases[&SynergyPhase::Att].score
                    );
                }
                if matches!(sc, GameStateKind::Protect | GameStateKind::AwayStrong) {
                    assert!(
                        r.intent_after.sec > r.intent_before.sec,
                        "sec must rise for {f} {sc:?}"
                    );
                    assert!(r.intent_after.att <= r.intent_before.att);
                    assert!(
                        r.after.phases[&SynergyPhase::Def].score as i16
                            >= r.before.phases[&SynergyPhase::Def].score as i16 - 2
                    );
                }
                if sc == GameStateKind::AwayStrong {
                    assert_eq!(r.ois.len(), 3);
                }
            }
        }
    }

    // Parity #21 & #24: scouting 5k + 100k benchmark, presets, combined filters, unknown value exclusion
    #[test]
    fn parity_21_and_24_scouting_presets_and_100k_perf() {
        assert_eq!(SCOUT_COLUMNS.len(), 97);
        let csv5k = gen_scout_db_csv(5_000, 42);
        let mut db = parse_scout_db(&csv5k, Lang::En, |_, _| {}).unwrap();
        assert_eq!(db.players.len(), 5_000);
        assert!(db.attr_cols >= 45);
        let p0 = &db.players[0];
        assert!(p0.ca.unwrap_or(0) > 0 && p0.pa.unwrap_or(0) >= p0.ca.unwrap_or(0));
        assert!(p0.pers_val(ScoutPers::Professionalism).unwrap_or(0) > 0);
        assert!(p0.hid_val(ScoutHid::Inj).unwrap_or(0) > 0);
        assert!(db.players.iter().any(|x| x.has_pos_group(PosGroup::St)));
        assert!(db.players.iter().any(|x| x.club.contains("\"B\"")));

        let date = infer_game_date(&db.players).expect("date inferred");
        assert_eq!(date.len(), 10);

        let squad = parse_squad(&fixture("fmsave_players.csv"), Lang::En).players;
        let mut t = tactic_from_template(&TEMPLATES[0]);
        auto_pick_xi(&mut t, &squad);
        let ctx = ScoutContext {
            date: Some(&date),
            my_club: None,
            tactic: Some(&t),
            squad: &squad,
            xi_ca: 120,
            cheap_value: None,
            bargain_value: None,
            my_rep: None,
        };
        for pr in scout_presets(&ctx, Lang::En) {
            let res = scout_filter(&db, &pr.filter, &ctx, Lang::En);
            match pr.key {
                "cheap_wonder" => {
                    assert!(!res.is_empty());
                    assert!(res.iter().all(|m| {
                        let x = &db.players[m.index];
                        x.age.unwrap() <= 19
                            && x.pa.unwrap() >= 150
                            && x.value.is_none_or(|v| v <= pr.filter.value_max.unwrap())
                    }));
                }
                "reach" => {
                    assert!(res
                        .iter()
                        .all(|m| m.likely.unwrap_or(0) >= 70
                            && db.players[m.index].age.unwrap() <= 20))
                }
                "expiring" => {
                    assert!(!res.is_empty());
                    assert!(res
                        .iter()
                        .all(|m| months_until(&db.players[m.index].end, &date)
                            .is_some_and(|v| v <= 6.0)));
                }
                "free" => {
                    assert!(!res.is_empty());
                    assert!(res.iter().all(|m| db.players[m.index].club.is_empty()));
                }
                "upgrade" => {
                    assert!(!res.is_empty());
                    assert!(res[0].gain >= res[res.len() - 1].gain);
                }
                "hidden" => assert!(res
                    .iter()
                    .all(|m| db.players[m.index].club_rep.is_none_or(|r| r <= 4000))),
                _ => {}
            }
        }

        // Combined filters
        let f_comb = ScoutFilter {
            pos: Some(PosGroup::Dc),
            pos_level: PosLevel::Nat,
            attrs: vec![
                ScoutAttrFilter {
                    k: Some(Attr::Hea),
                    min: Some(12),
                },
                ScoutAttrFilter {
                    k: Some(Attr::Pac),
                    min: Some(12),
                },
            ],
            height_min: Some(185),
            sort: ScoutSort::Ca,
            ..ScoutFilter::default()
        };
        let r2 = scout_filter(&db, &f_comb, &ctx, Lang::En);
        assert!(r2.iter().all(|m| {
            let x = &db.players[m.index];
            x.has_nat_group(PosGroup::Dc)
                && x.attrs[Attr::Hea.index()] >= 12
                && x.attrs[Attr::Pac.index()] >= 12
                && x.height.unwrap_or(0) >= 185
        }));

        let star_p = db.players.iter().find(|x| x.ca.unwrap_or(0) > 150).unwrap();
        assert!(best_gain(star_p, &t, &squad).is_some());

        // Parity #24: unknown value stars excluded from bargain/cheap_wonder/hidden (free agents still pass)
        let mut star_ids = HashSet::new();
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
            star_ids.insert(p.id.clone());
        }
        let ctx24 = ScoutContext {
            date: Some("2027-01-15"),
            my_rep: Some(6000),
            xi_ca: 120,
            ..ScoutContext::default()
        };
        for k in ["bargain", "cheap_wonder", "hidden"] {
            let pr = scout_presets(&ctx24, Lang::En)
                .into_iter()
                .find(|p| p.key == k)
                .unwrap();
            let res = scout_filter(&db, &pr.filter, &ctx24, Lang::En);
            assert!(res
                .iter()
                .all(|m| !star_ids.contains(&db.players[m.index].id)));
            assert!(res.iter().all(|m| {
                let p = &db.players[m.index];
                p.club.is_empty() || p.value.is_some_and(|v| v <= pr.filter.value_max.unwrap())
            }));
        }

        // 100k-player benchmark (fast even in unoptimized debug builds!)
        let big = gen_scout_db_csv(100_000, 42);
        let t0 = Instant::now();
        let db_big = parse_scout_db(&big, Lang::En, |_, _| {}).unwrap();
        let parse_ms = t0.elapsed().as_millis();
        let t1 = Instant::now();
        let _filtered = scout_filter(
            &db_big,
            &ScoutFilter {
                age_max: Some(21),
                pa_min: Some(150),
                ..ScoutFilter::default()
            },
            &ctx,
            Lang::En,
        );
        let filter_ms = t1.elapsed().as_millis();
        assert_eq!(db_big.players.len(), 100_000);
        assert!(parse_ms < 10_000, "100k parse took {parse_ms} ms");
        assert!(filter_ms < 500, "100k filter took {filter_ms} ms");
    }

    // Parity #22: join likelihood monotonicity & "interested" preset
    #[test]
    fn parity_22_join_likelihood() {
        let csv3k = gen_scout_db_csv(3_000, 42);
        let db = parse_scout_db(&csv3k, Lang::En, |_, _| {}).unwrap();
        let base = ScoutPlayer {
            club: "Other".into(),
            club_lower: "other".into(),
            club_rep: Some(5000),
            rep: Some(4000),
            end: "2027-06-30".into(),
            status: "Rotation".into(),
            pers: {
                let mut arr = [0u8; ScoutPers::COUNT];
                arr[ScoutPers::Ambition as usize] = 12;
                arr[ScoutPers::Loyalty as usize] = 10;
                arr
            },
            ..db.players[0].clone()
        };
        let mk_ctx = |my_rep: Option<i32>| ScoutContext {
            date: Some("2027-01-15"),
            my_rep,
            ..ScoutContext::default()
        };
        assert!(join_likelihood(&base, &mk_ctx(None), Lang::En).is_none());
        assert!(
            join_likelihood(&base, &mk_ctx(Some(8000)), Lang::En)
                .unwrap()
                .score
                > join_likelihood(&base, &mk_ctx(Some(3000)), Lang::En)
                    .unwrap()
                    .score
        );
        let mut free_p = base.clone();
        free_p.club.clear();
        free_p.club_rep = None;
        assert!(
            join_likelihood(&free_p, &mk_ctx(Some(4000)), Lang::En)
                .unwrap()
                .score
                > join_likelihood(&base, &mk_ctx(Some(4000)), Lang::En)
                    .unwrap()
                    .score
        );
        let mut high_rep = base.clone();
        high_rep.rep = Some(9000);
        assert!(
            join_likelihood(&high_rep, &mk_ctx(Some(5000)), Lang::En)
                .unwrap()
                .score
                < join_likelihood(&base, &mk_ctx(Some(5000)), Lang::En)
                    .unwrap()
                    .score
        );
        let mut loyal = base.clone();
        loyal.pers[ScoutPers::Loyalty as usize] = 19;
        let mut disloyal = base.clone();
        disloyal.pers[ScoutPers::Loyalty as usize] = 5;
        assert!(
            join_likelihood(&loyal, &mk_ctx(Some(5000)), Lang::En)
                .unwrap()
                .score
                < join_likelihood(&disloyal, &mk_ctx(Some(5000)), Lang::En)
                    .unwrap()
                    .score
        );

        let c = ScoutContext {
            date: Some("2027-01-15"),
            my_rep: Some(6000),
            xi_ca: 110,
            ..ScoutContext::default()
        };
        let pr = scout_presets(&c, Lang::En)
            .into_iter()
            .find(|p| p.key == "interested")
            .unwrap();
        let res = scout_filter(&db, &pr.filter, &c, Lang::En);
        assert!(!res.is_empty());
        assert!(res.iter().all(|m| m.join.unwrap_or(0) >= 70));
        assert!(res.windows(2).all(|w| w[0].join >= w[1].join));
    }

    // Parity #25: enrich_from_scout enriches fmsave squads and leaves F9 CSVs untouched
    #[test]
    fn parity_25_enrich_from_scout() {
        let csv = gen_scout_db_csv(25, 11);
        let mut ps = parse_squad(&csv, Lang::En).players;
        let n = enrich_from_scout(&mut ps, &csv);
        assert_eq!(n, 25);
        assert!(ps
            .iter()
            .all(|p| p.ca.unwrap_or(0.0) > 0.0 && p.pa.unwrap_or(0.0) >= p.ca.unwrap_or(0.0)));
        assert!(ps.iter().all(|p| p.pers.len() == 8 && p.hid.len() == 5));
        assert!(ps.iter().any(|p| p.has_pos_group(PosGroup::St)));

        let f9 = "Name,Position,Age,Finishing,Passing,Tackling\nA,ST,22,14,10,8\nB,DC,25,8,11,15";
        let mut f9_ps = parse_squad(f9, Lang::En).players;
        assert_eq!(enrich_from_scout(&mut f9_ps, f9), 0);
    }
}
