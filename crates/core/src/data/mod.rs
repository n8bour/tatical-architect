//! Static knowledge base & bilingual EN/PT translation engine (Module A).

pub mod i18n;
pub mod roles;
pub mod tactics_data;

pub use i18n::{
    l, mentality_name, pi_name, pt_terms, role_name, t, t_name, ti_name, ti_opt_name,
    EN_DICT_ENTRIES, PT_EXACT_ONLY, PT_TERMS,
};
pub use roles::{
    attr_from_norm_header, attr_meta, attr_name, default_role, group_name, group_name_pt,
    norm_header, role_by_id, role_fits, roles_for, slot_pos, valid_role, ATTR_META, ROLES,
};
pub use tactics_data::{
    formation_by_name, formation_slots, group_of, has_player_instruction, has_tactical_tweak,
    is_attr_verified, is_wide_cb, mentality_info, mentality_info_pt, side_of, slot_info,
    style_profile, template_by_id, ti_by_key, ti_label, ti_val, training_area, ATTR_VERIFIED,
    FM_SECTIONS, FORMATIONS, PLAYER_INSTRUCTIONS, STYLE_PROFILES, TACTICAL_TWEAKS,
    TEAM_INSTRUCTIONS, TEMPLATES, TRAINING_AREAS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Attr, Lang, Mentality, Phase, PosGroup, Tactic};
    use std::collections::HashSet;

    #[test]
    fn roles_77_unique_ids_and_valid_attrs() {
        assert_eq!(ROLES.len(), 77);
        let mut ids = HashSet::new();
        for r in &ROLES {
            assert!(ids.insert(r.id), "duplicate role id: {}", r.id);
            assert!(!r.key.is_empty(), "role {} has no key attrs", r.id);
            for &a in r.key.iter().chain(r.pref.iter()) {
                assert_eq!(attr_meta(a).attr, a);
            }
        }
    }

    #[test]
    fn every_pos_group_has_ip_and_oop_roles() {
        for g in PosGroup::ALL {
            assert!(
                !roles_for(g, Phase::Ip, None, false).is_empty(),
                "missing IP roles for {:?}",
                g
            );
            assert!(
                !roles_for(g, Phase::Oop, None, false).is_empty(),
                "missing OOP roles for {:?}",
                g
            );
        }
    }

    #[test]
    fn formations_17_by_11_slots_with_gk() {
        assert_eq!(FORMATIONS.len(), 17);
        for f in &FORMATIONS {
            assert_eq!(f.slots.len(), 11, "formation {} must have 11 slots", f.name);
            let codes: HashSet<&str> = f.slots.iter().map(|s| s.code).collect();
            assert_eq!(codes.len(), 11, "duplicate codes in {}", f.name);
            assert!(codes.contains("GK"), "missing GK in {}", f.name);
        }
    }

    #[test]
    fn templates_8_have_valid_formations_roles_and_tis() {
        assert_eq!(TEMPLATES.len(), 8);
        for tp in &TEMPLATES {
            let ip_slots = formation_slots(tp.ip).expect("template IP formation exists");
            assert!(formation_slots(tp.oop).is_some(), "template OOP formation exists");
            assert_eq!(tp.roles.len(), 11, "template {} must define 11 role pairs", tp.id);
            for &(code, ip_id, _oop_id) in tp.roles {
                assert!(
                    ip_slots.iter().any(|s| s.code == code),
                    "template {} unknown IP slot {}",
                    tp.id,
                    code
                );
                let meta = slot_info(tp.ip, code).unwrap();
                assert!(
                    valid_role(ip_id, meta.group, Phase::Ip, Some(&meta.code), meta.wide_cb),
                    "template {} invalid IP role {} at {}",
                    tp.id,
                    ip_id,
                    code
                );
            }
            for &(k, v) in tp.ti {
                let d = ti_by_key(k).unwrap_or_else(|| panic!("unknown TI {} in {}", k, tp.id));
                assert!(v < d.opts.len(), "TI {} value {} out of bounds", k, v);
            }
        }
    }

    #[test]
    fn exact_slot_rules_b2b_and_b2bp() {
        let at = |form: &str, code: &str| -> Vec<&'static str> {
            let m = slot_info(form, code).unwrap();
            roles_for(m.group, Phase::Ip, Some(&m.code), m.wide_cb)
                .into_iter()
                .map(|r| r.id)
                .collect()
        };
        assert!(!at("4-3-3 DM Wide", "MCL").contains(&"b2bp"));
        assert!(!at("4-3-3 DM Wide", "MCL").contains(&"b2b"));
        assert!(!at("4-3-3 DM Wide", "DM").contains(&"b2b"));
        assert!(at("4-2-4 DM Wide", "DML").contains(&"b2b"));
        assert!(at("4-2-4 DM Wide", "DML").contains(&"b2bp"));
        assert!(at("3-4-2-1", "DCL").contains(&"ocb"));
        assert!(!at("4-3-3 DM Wide", "DCL").contains(&"ocb"));
    }

    #[test]
    fn i18n_en_dict_and_pt_terms_coverage() {
        assert_eq!(TEAM_INSTRUCTIONS.len(), 27);
        assert_eq!(PLAYER_INSTRUCTIONS.len(), 24);
        assert_eq!(TACTICAL_TWEAKS.len(), 5);
        assert_eq!(TRAINING_AREAS.len(), 8);
        assert_eq!(STYLE_PROFILES.len(), 7);

        // EN dictionary exact + numeric placeholder + prefix + native error rx
        assert_eq!(t(Lang::En, "Prospeção"), "Scouting");
        assert_eq!(t(Lang::En, "Situações de jogo"), "Game states");
        assert_eq!(t(Lang::En, "→ Ler o save agora"), "→ Read the save now");
        assert_eq!(t(Lang::En, "Tática gerada em 2.4 s"), "Tactic generated in 2.4 s");
        assert_eq!(
            t(Lang::En, "Não consegui correr o fmsave. uvx: not found"),
            "Could not run fmsave. uvx: not found"
        );

        // PT terms translate all 77 roles, 27 TIs, 24 PIs, 7 mentalities
        for r in &ROLES {
            let pt = role_name(r, Lang::Pt);
            assert!(!pt.is_empty());
            assert_ne!(pt, r.name, "role {} ({}) untranslated in PT", r.id, r.name);
        }
        assert_eq!(role_name(role_by_id("am_oop").unwrap(), Lang::Pt), "Médio Ofensivo");
        assert_eq!(role_name(role_by_id("am_ip").unwrap(), Lang::Pt), "Médio Entrelinhas");

        for ti in &TEAM_INSTRUCTIONS {
            let pt = ti_name(ti, Lang::Pt);
            assert_ne!(pt, ti.name, "TI {} untranslated in PT", ti.name);
        }
        for pi in &PLAYER_INSTRUCTIONS {
            let pt = pi_name(pi.name, Lang::Pt);
            assert_ne!(pt, pi.name, "PI {} untranslated in PT", pi.name);
        }
        for m in Mentality::ALL {
            let pt = mentality_name(m, Lang::Pt);
            assert_ne!(pt, m.as_str(), "Mentality {} untranslated in PT", m.as_str());
        }
        assert_eq!(pt_terms("Yes"), "Sim");
        assert_eq!(pt_terms("No"), "Não");
        // Must not corrupt Portuguese words containing "No" or "Yes"
        assert_eq!(pt_terms("Nova tática"), "Nova tática");

        let mut tac = Tactic::default();
        tac.name = "Nova tática".into();
        assert_eq!(t_name(&tac, Lang::En), "New tactic");
        assert_eq!(t_name(&tac, Lang::Pt), "Nova tática");

        // Header normalization & alias lookup
        assert_eq!(attr_from_norm_header(&norm_header("attributes_first_touch")), Some(Attr::Fir));
        assert_eq!(attr_from_norm_header(&norm_header("1v1")), Some(Attr::Ovo));
        assert_eq!(attr_from_norm_header(&norm_header("Finalização")), Some(Attr::Fin));
        assert_eq!(attr_name(Attr::Otb, Lang::En), "Off the Ball");
        assert_eq!(attr_name(Attr::Otb, Lang::Pt), "Sem Bola");
        assert!(!mentality_info(Mentality::Positive, Lang::En).is_empty());
        assert_eq!(group_name(PosGroup::Gk, Lang::Pt), "Guarda-redes");
    }
}
