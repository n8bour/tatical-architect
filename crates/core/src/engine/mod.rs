//! Core tactical analysis engine (Module B): Hungarian matching, parsers, scoring,
//! 12-category audit, insights, set pieces, and 4-phase synergy.

pub mod audit;
pub mod hungarian;
pub mod insights;
pub mod parsers;
pub mod scoring;
pub mod synergy;

pub use audit::audit_tactic;
pub use hungarian::{
    auto_pick_xi, hungarian, match_formations, new_tactic, retarget_tactic, tactic_from_template,
};
pub use insights::{
    bench_and_successors, opponent_report, recommend_pis, set_piece_plan, sp_aerial, sp_captain,
    sp_corner, sp_cover, sp_edge, sp_fk_cross, sp_fk_direct, sp_marker, sp_outlet, sp_penalty,
    sp_zone,
};
pub use parsers::{
    inspect_fmf, majority_club, parse_attr_value, parse_fmf_container, parse_positions,
    parse_squad, surname,
};
pub use scoring::{
    avg_attr, best_roles_for_player, complexity_of, foot_factor, pos_factor, role_score,
    slot_score, style_fit, suggest_tactics, training_plan,
};
pub use synergy::{
    synergy_attack, synergy_part_name, synergy_phase_name, synergy_report, SynergyAttack,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::roles::{role_by_id, roles_for, valid_role, ROLES};
    use crate::data::tactics_data::{
        has_player_instruction, slot_info, ti_by_key, FORMATIONS, TEMPLATES,
    };
    use crate::types::{Attr, Lang, Phase, Player, PosGroup, SynergyPhase, Tactic};
    use std::collections::HashSet;
    use std::path::PathBuf;

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

    // #3: All 17x17 formation pairs match 11 distinct OOP slots with GK -> GK
    #[test]
    fn parity_03_all_formation_pairs_match_11_distinct_and_gk() {
        assert_eq!(FORMATIONS.len(), 17);
        for a in &FORMATIONS {
            assert_eq!(a.slots.len(), 11, "11 slots in {}", a.name);
            for b in &FORMATIONS {
                let pairs = match_formations(a.name, b.name);
                assert_eq!(pairs.len(), 11);
                let oop_set: HashSet<&str> = pairs.iter().map(|(_, o)| o.as_str()).collect();
                assert_eq!(oop_set.len(), 11, "match {} -> {}", a.name, b.name);
                let gk = pairs.iter().find(|(ip, _)| ip == "GK").unwrap();
                assert_eq!(gk.1, "GK", "GK mismatch {} -> {}", a.name, b.name);
            }
        }
    }

    // #4: All 8 templates apply exact IP/OOP roles, pass audit cleanly, and have valid TIs
    #[test]
    fn parity_04_templates_applied_and_valid() {
        assert_eq!(TEMPLATES.len(), 8);
        for tp in &TEMPLATES {
            let t = tactic_from_template(tp);
            assert_eq!(t.slots.len(), 11);
            for s in &t.slots {
                let a = slot_info(tp.ip, &s.ip_code).unwrap();
                let b = slot_info(tp.oop, &s.oop_code).unwrap();
                assert!(
                    valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb),
                    "{} IP {} {}",
                    tp.id,
                    s.ip_code,
                    s.ip
                );
                assert!(
                    valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb),
                    "{} OOP {} {}",
                    tp.id,
                    s.oop_code,
                    s.oop
                );
                let &(_, want_ip, want_oop) = tp
                    .roles
                    .iter()
                    .find(|&&(code, _, _)| code == s.ip_code)
                    .unwrap_or_else(|| panic!("{} missing role for {}", tp.id, s.ip_code));
                assert_eq!(s.ip, want_ip, "{} {} IP", tp.id, s.ip_code);
                assert_eq!(s.oop, want_oop, "{} {} OOP", tp.id, s.ip_code);
            }
            let au = audit_tactic(&t, &[], Lang::En);
            assert!(au.overall <= 100);
            for &(k, _) in tp.ti {
                assert!(ti_by_key(k).is_some(), "unknown ti {k}");
            }
        }
    }

    // #5: parse_positions edge cases + short codes
    #[test]
    fn parity_05_parse_positions_edge_cases() {
        let p1 = parse_positions("D (RL), WB (R)");
        assert!(p1.contains(&"D_LR".to_string()));
        assert!(p1.contains(&"D_LR:R".to_string()));
        assert!(p1.contains(&"D_LR:L".to_string()));
        assert!(p1.contains(&"WB_LR".to_string()));
        assert!(p1.contains(&"WB_LR:R".to_string()));
        assert!(!p1.contains(&"WB_LR:L".to_string()));

        let p2 = parse_positions("AM (RLC), ST (C)");
        assert!(p2.contains(&"AMC".to_string()));
        assert!(p2.contains(&"AM_LR".to_string()));
        assert!(p2.contains(&"AM_LR:R".to_string()));
        assert!(p2.contains(&"AM_LR:L".to_string()));
        assert!(p2.contains(&"ST".to_string()));

        assert_eq!(parse_positions("DM, M (C)"), vec!["DM", "MC"]);
        assert_eq!(parse_positions("M/AM (R)"), vec!["M_LR", "M_LR:R", "AM_LR", "AM_LR:R"]);
        assert_eq!(parse_positions("GK"), vec!["GK"]);
        assert_eq!(parse_positions("DC, DM"), vec!["DC", "DM"]);
        assert_eq!(parse_positions("AMR"), vec!["AM_LR", "AM_LR:R"]);
    }

    // #6: FM pipe table + range average + CSV + HTML table
    #[test]
    fn parity_06_parse_squad_pipe_csv_html() {
        let pipe = "| Name | Position | Age | Acc | Pac | Fin | Fir | Tec | Cmp | OtB | Str | Dri | Pas | Hea | Ant | Dec | Wor | Sta | Tea | Pos | Mar | Tck | Cnt |
| ------------------------------------- |
| Rui Matos | ST (C) | 24 | 14 | 13 | 15 | 14 | 13 | 14 | 15 | 14 | 12 | 11 | 14 | 14 | 12 | 12 | 13 | 12 | 6 | 5 | 6 | 11 |
| Tiago Lobo | D (C) | 29 | 10 | 11 | 5 | 10 | 9 | 13 | 6 | 15 | 7 | 11 | 15 | 13 | 13 | 12 | 13 | 13 | 14 | 15 | 14-16 | 14 |";

        let r = parse_squad(pipe, Lang::Pt);
        assert_eq!(r.players.len(), 2);
        assert_eq!(r.players[1].a.get(&Attr::Tck), Some(&15.0));
        assert!(r.players[0].pos.contains(&"ST".to_string()));
        assert_eq!(r.players[1].a.get(&Attr::Pos), Some(&14.0));

        let csv = "Nome;Posição;Idade;Fin;Pas\nA;GK;20;3;10\nB;ST (C);22;15;11";
        let r2 = parse_squad(csv, Lang::Pt);
        assert_eq!(r2.players.len(), 0); // < 3 attrs
        assert!(!r2.warnings.is_empty());

        let html = "<html><body><table><tr><th>Name</th><th>Position</th><th>Finishing</th><th>Passing</th><th>Tackling</th></tr><tr><td><span>João &amp; Silva</span></td><td>ST (C)</td><td>15</td><td>13</td><td>6-8</td></tr></table></body></html>";
        let r3 = parse_squad(html, Lang::En);
        assert_eq!(r3.players.len(), 1);
        assert_eq!(r3.players[0].name, "João & Silva");
        assert_eq!(r3.players[0].a.get(&Attr::Tck), Some(&7.0));
    }

    // #7: retarget_tactic preserves roles when valid
    #[test]
    fn parity_07_retarget_preserves_roles() {
        let t = tactic_from_template(&TEMPLATES[0]);
        let t2 = retarget_tactic(&t, "4-2-3-1 Wide", "4-4-1-1", Lang::En);
        assert_eq!(t2.slots.len(), 11);
        assert_eq!(t2.name, t.name);
        assert_eq!(t2.ment, t.ment);
        assert_eq!(t2.ti, t.ti);
        for s in &t2.slots {
            let a = slot_info(&t2.ip_form, &s.ip_code).unwrap();
            let b = slot_info(&t2.oop_form, &s.oop_code).unwrap();
            assert!(valid_role(&s.ip, a.group, Phase::Ip, Some(&a.code), a.wide_cb));
            assert!(valid_role(&s.oop, b.group, Phase::Oop, Some(&b.code), b.wide_cb));
        }
    }

    // #8: complexity_of gegenpress template
    #[test]
    fn parity_08_complexity_gegenpress() {
        let t = tactic_from_template(&TEMPLATES[2]); // gegen4222
        let cx = complexity_of(&t, Lang::En);
        assert!(cx.cx > 6.0);
        assert!(!cx.parts.is_empty());
        assert_eq!(cx.level, "Medium");
    }

    // #9: real FM27 .fmf container fixture -> recognized, 3 encrypted blocks, largest cipher 707 B
    #[test]
    fn parity_09_fmf_container_and_inspect() {
        let bytes = read_fixture_bytes("cenas_v2.fmf");
        let c = parse_fmf_container(&bytes);
        assert!(c.recognized);
        assert_eq!(c.blocks.len(), 3);
        assert_eq!(c.largest.map(|b| b.cipher), Some(707));

        let rep = inspect_fmf("cenas_v2.fmf", &bytes);
        assert_eq!(rep.size, bytes.len());
        assert!(rep.container.recognized);
        assert_eq!(rep.container.blocks.len(), 3);
        assert_eq!(rep.container.largest.map(|b| b.cipher), Some(707));
        assert!(rep.magic.iter().any(|m| m.starts_with("zstd @")));
        assert!(rep.entropy > 5.0);
    }

    // #10: FM View Exporter CSV fixtures (EN and German semicolon/decimal)
    #[test]
    fn parity_10_exporter_en_and_de_fixtures() {
        for f in ["squad_en.csv", "squad_de.csv"] {
            let r = parse_squad(&read_fixture_text(f), Lang::En);
            assert_eq!(r.players.len(), 16, "{f} player count");
            let p0 = &r.players[0];
            let p3 = &r.players[3];
            let p5 = &r.players[5];
            assert_eq!(p0.a.len(), 47, "{f} attr count");
            assert_eq!(p3.a.get(&Attr::Tck), Some(&13.5), "{f} tck range");
            assert!(p5.pos.contains(&"WB_LR:R".to_string()), "{f} WB_LR:R");
            assert!(p0.meta.uid.is_some(), "{f} uid");
            assert_eq!(p0.height, Some(180.0), "{f} parsed height");
            assert_eq!(r.warnings.len(), 0, "{f} warnings");
        }
    }

    // #12: Insights — recommended PIs, opponent report, bench & successors, majority club
    #[test]
    fn parity_12_insights() {
        let me = parse_squad(&read_fixture_text("squad_en.csv"), Lang::En).players;
        let them = parse_squad(&read_fixture_text("squad_de.csv"), Lang::En).players;
        let mut t = tactic_from_template(&TEMPLATES[0]);
        auto_pick_xi(&mut t, &me);

        for s in &t.slots {
            let p = me.iter().find(|q| Some(q.id.as_str()) == s.pid.as_deref()).unwrap();
            let grp = slot_info(&t.ip_form, &s.ip_code).unwrap().group;
            let recs = recommend_pis(p, role_by_id(&s.ip), role_by_id(&s.oop), grp, Lang::En);
            assert!(recs.len() <= 3);
            for rec in recs {
                assert!(has_player_instruction(rec.pi), "invalid PI {}", rec.pi);
                assert!(rec.why.len() > 5, "empty why for {}", rec.pi);
            }
        }

        // Fast ST must get "Attack the Defensive Line"
        let mut fast = Player {
            id: "x".to_string(),
            name: "Rápido".to_string(),
            ..Player::default()
        };
        for (a, v) in [
            (Attr::Pac, 17.0),
            (Attr::Acc, 17.0),
            (Attr::Otb, 15.0),
            (Attr::Fin, 14.0),
            (Attr::Sta, 14.0),
            (Attr::Wor, 12.0),
            (Attr::Dri, 12.0),
            (Attr::Agi, 12.0),
        ] {
            fast.a.insert(a, v);
        }
        let fast_recs = recommend_pis(
            &fast,
            role_by_id("poa"),
            role_by_id("cf_oop"),
            PosGroup::St,
            Lang::En,
        );
        assert!(fast_recs.iter().any(|r| r.pi == "Attack the Defensive Line"));

        let rep = opponent_report(&t, &me, &them, Lang::En);
        assert_eq!(rep.them.len(), 11);
        assert_eq!(rep.sectors.len(), 9);
        assert_eq!(rep.dangers.len(), 3);
        for item in &rep.plan {
            let def = ti_by_key(item.ti).unwrap();
            assert!(item.value < def.opts.len());
        }

        // Empty own squad does not panic
        let rep2 = opponent_report(&t, &[], &them, Lang::En);
        assert!(rep2.sectors.iter().all(|s| s.us_val.is_none()));

        let bs = bench_and_successors(&t, &me);
        assert_eq!(bs.len(), 11);

        let mc = majority_club(&me);
        assert_eq!(mc.club.as_deref(), Some("Clube"));
        assert!((mc.share - 1.0).abs() < 1e-6);

        assert_eq!(surname("Kevin De Bruyne"), "De Bruyne");
        assert_eq!(surname("Virgil van Dijk"), "van Dijk");
        assert_eq!(surname("Neymar da Silva Júnior"), "da Silva Júnior");
    }

    // #13: fmsave CSV format
    #[test]
    fn parity_13_fmsave_csv() {
        let r = parse_squad(&read_fixture_text("fmsave_players.csv"), Lang::Pt);
        assert_eq!(r.players.len(), 16);
        let p0 = &r.players[0];
        let p5 = &r.players[5];
        let p11 = &r.players[11];
        assert_eq!(p0.a.len(), 47);
        assert!(p5.pos.contains(&"D_LR:R".to_string()));
        assert!(p5.pos.contains(&"WB_LR".to_string()));
        assert_eq!(p0.meta.club.as_deref(), Some("Patro Eisden"));
        assert!(p11.pos.contains(&"ST".to_string()));
        assert_eq!(r.warnings.len(), 0);
    }

    // #14: Set pieces across 3 squads x 8 templates
    #[test]
    fn parity_14_set_pieces_all_squads_and_templates() {
        let cases: [(&str, Option<&str>); 3] = [
            ("fmsave_players.csv", Some("squad_en.csv")),
            ("squad_en.csv", None),
            ("squad_de.csv", Some("fmsave_players.csv")),
        ];
        for (mine_f, opp_f) in cases {
            let me = parse_squad(&read_fixture_text(mine_f), Lang::En).players;
            let op = opp_f
                .map(|f| parse_squad(&read_fixture_text(f), Lang::En).players)
                .unwrap_or_default();
            for tp in &TEMPLATES {
                let t = tactic_from_template(tp);
                let plan = set_piece_plan(&t, &me, &op, Lang::En)
                    .unwrap_or_else(|| panic!("no set-piece plan for {mine_f} / {}", tp.id));

                let a_ids: Vec<&str> = plan.attack.iter().map(|r| r.player_id.as_str()).collect();
                let d_ids: Vec<&str> = plan.defend.iter().map(|r| r.player_id.as_str()).collect();
                assert_eq!(
                    a_ids.iter().copied().collect::<HashSet<_>>().len(),
                    a_ids.len(),
                    "duplicate attacker in {}",
                    tp.id
                );
                assert_eq!(
                    d_ids.iter().copied().collect::<HashSet<_>>().len(),
                    d_ids.len(),
                    "duplicate defender in {}",
                    tp.id
                );
                let back_n = plan.attack.iter().filter(|r| r.key.starts_with("back")).count();
                assert!(back_n >= 2, "<2 back in {}", tp.id);
                let gk_ids: HashSet<&str> = plan
                    .xi
                    .iter()
                    .filter(|m| m.group == PosGroup::Gk)
                    .map(|m| m.player_id.as_str())
                    .collect();
                assert!(
                    !plan
                        .attack
                        .iter()
                        .chain(plan.defend.iter())
                        .any(|r| gk_ids.contains(r.player_id.as_str())),
                    "GK assigned outfield in {}",
                    tp.id
                );
                assert_eq!(plan.pens.len(), 3);
                assert_eq!(plan.caps.len(), 2);
            }
        }
    }

    // #17: Exact-slot availability for all 77 roles and confirmed FM27 rules
    #[test]
    fn parity_17_exact_slot_rules_and_77_roles() {
        for r in &ROLES {
            let found = FORMATIONS.iter().find_map(|f| {
                f.slots.iter().find_map(|z| {
                    let a = slot_info(f.name, z.code)?;
                    valid_role(r.id, a.group, r.phase, Some(&a.code), a.wide_cb)
                        .then_some((f.name, a))
                })
            });
            assert!(found.is_some(), "role {} has no valid slot", r.id);
        }

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
    }

    // #19: Defensive synergy textbook cases
    #[test]
    fn parity_19_defensive_synergy_textbook_cases() {
        let mk = |roles: &[(&str, &str)]| -> Tactic {
            let mut t = new_tactic("4-3-3 DM Wide", "4-1-4-1 DM Wide", Lang::Pt);
            for s in &mut t.slots {
                if let Some(&(_, r)) = roles.iter().find(|&&(code, _)| code == s.oop_code) {
                    s.oop = r.to_string();
                }
            }
            t
        };

        let stop_cover = synergy_report(&mk(&[("DCL", "scb"), ("DCR", "ccb")]), &[], Lang::Pt);
        let two_stop = synergy_report(&mk(&[("DCL", "scb"), ("DCR", "scb")]), &[], Lang::Pt);
        assert_eq!(stop_cover.phases[&SynergyPhase::Def].parts["cb"], 100);
        assert!(two_stop.phases[&SynergyPhase::Def].parts["cb"] < 50);
        assert!(two_stop.issues.iter().any(|i| i.key == "cb"));
        assert!(
            !stop_cover
                .issues
                .iter()
                .any(|i| i.key == "press" && (i.text.contains("scb") || i.text.contains("SCD")))
        );

        let no_screen = synergy_report(
            &mk(&[("DM", "pdm"), ("MCL", "pcm"), ("MCR", "pcm")]),
            &[],
            Lang::Pt,
        );
        assert!(no_screen.phases[&SynergyPhase::Def].parts["screen"] < 50);
        assert!(no_screen.issues.iter().any(|i| i.key == "screen"));
    }

    // #23: Partial 4-attribute CSV never panics in audit, suggest_tactics, or training_plan
    #[test]
    fn parity_23_partial_4_attr_csv_never_panics() {
        let positions = [
            "GK", "DC", "DC", "D (L)", "D (R)", "DM", "M (C)", "M (C)", "AM (L)", "AM (R)", "ST",
            "ST", "DC", "M (C)", "ST", "GK",
        ];
        let mut csv = String::from("Name,Club,Position,Age,Finishing,Passing,Tackling,Pace\n");
        for (i, pos) in positions.iter().enumerate() {
            csv.push_str(&format!(
                "P{i},Club 5,{pos},24,{},12,11,13\n",
                10 + (i % 8)
            ));
        }
        let ps = parse_squad(&csv, Lang::En).players;
        assert_eq!(ps.len(), 16);
        for tp in &TEMPLATES {
            let mut t = tactic_from_template(tp);
            auto_pick_xi(&mut t, &ps);
            let au = audit_tactic(&t, &ps, Lang::En);
            let plan = training_plan(&t, &ps, &au, Lang::En);
            assert!(!plan.areas.is_empty());
        }
        let sugg = suggest_tactics(&ps, Lang::En);
        assert_eq!(sugg.len(), 8);
    }

    #[test]
    fn scoring_foot_factor_pos_factor_and_55_45_blend() {
        let mut p = Player {
            id: "w1".to_string(),
            name: "Left Winger".to_string(),
            pos: vec!["AM_LR".to_string(), "AM_LR:L".to_string()],
            foot: Some(crate::types::Foot::L),
            ..Player::default()
        };
        for attr in Attr::ALL {
            p.a.insert(attr, 15.0);
        }
        let aml = slot_info("4-3-3 DM Wide", "AML").unwrap();
        let amr = slot_info("4-3-3 DM Wide", "AMR").unwrap();
        let if_role = role_by_id("if_ip").unwrap();
        let w_role = role_by_id("w_ip").unwrap();

        // Inside Forward left-footed on left -> 0.92 penalty; on right -> 1.0
        assert!((foot_factor(&p, &aml, if_role) - 0.92).abs() < 1e-6);
        assert!((foot_factor(&p, &amr, if_role) - 1.0).abs() < 1e-6);
        // Winger left-footed on left -> 1.0; on right -> 0.95 penalty
        assert!((foot_factor(&p, &aml, w_role) - 1.0).abs() < 1e-6);
        assert!((foot_factor(&p, &amr, w_role) - 0.95).abs() < 1e-6);
        // Natural flank 1.0 vs opposite flank 0.9 vs adjacent (M_LR) 0.82 vs non-adj (ST) 0.6 vs GK 0.1
        assert!((pos_factor(&p, PosGroup::AmLr, crate::types::Side::L) - 1.0).abs() < 1e-6);
        assert!((pos_factor(&p, PosGroup::AmLr, crate::types::Side::R) - 0.9).abs() < 1e-6);
        assert!((pos_factor(&p, PosGroup::MLr, crate::types::Side::L) - 0.82).abs() < 1e-6);
        assert!((pos_factor(&p, PosGroup::St, crate::types::Side::C) - 0.6).abs() < 1e-6);
        assert!((pos_factor(&p, PosGroup::Gk, crate::types::Side::C) - 0.1).abs() < 1e-6);

        // 55/45 IP/OOP blend check
        let t = new_tactic("4-3-3 DM Wide", "4-3-3 DM Wide", Lang::En);
        let aml_slot = t.slots.iter().find(|s| s.ip_code == "AML").unwrap();
        let sc = slot_score(&p, &t, aml_slot);
        let expected = ((15.0_f32 - 1.0) / 19.0 * 100.0).round() as u8; // all attrs = 15, natural AML, left-footed Winger -> 1.0 * 1.0
        assert_eq!(sc, expected);
    }

    #[test]
    fn inspect_fmf_inflates_embedded_zlib_and_matches_terms() {
        use flate2::write::ZlibEncoder;
        use flate2::Compression;
        use std::io::Write;

        let payload = b"Inside Forward 4-3-3 Passing Directness Positive Counter-Press";
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(payload).unwrap();
        let mut blob = vec![0x00, 0x01, 0x02, 0x03];
        blob.extend_from_slice(&enc.finish().unwrap());
        blob.extend_from_slice(&[0xff, 0xfe, 0xfd, 0xfc]);

        let rep = inspect_fmf("synthetic.fmf", &blob);
        assert_eq!(rep.streams.len(), 1);
        assert_eq!(rep.streams[0].fmt, "deflate");
        assert!(rep.decoded);
        assert!(rep.matches.roles.contains(&"Inside Forward".to_string()));
        assert!(rep.matches.formations.contains(&"4-3-3".to_string()));
        assert!(rep.matches.instructions.contains(&"Passing Directness".to_string()));
        assert!(rep.matches.mentality.contains(&"Positive".to_string()));
    }
}
