//! Full-save scouting database parser, derived heuristics, 30+ filter predicates,
//! 11 presets, and squad enrichment.
//! Ported 1:1 from `src/core/scouting.js` with a compact memory layout for 100k+ players.

use crate::data::{attr_from_norm_header, l, norm_header, role_by_id, roles_for, slot_info};
use crate::engine::scoring::{pos_factor, role_score, slot_score};
use crate::types::{
    Attr, AttrLookup, BestGain, Foot, JoinLikelihood, Lang, LoanFilter, Phase, Player, PosGroup,
    PosLevel, Role, ScoutContext, ScoutDb, ScoutFilter, ScoutHid, ScoutMatch, ScoutPers,
    ScoutPlayer, ScoutPosKey, ScoutPreset, ScoutSort, SortDir, Tactic,
};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// All 97 columns requested from `fmsave export <save> players --all --columns ...`.
pub const SCOUT_COLUMNS: [&str; 97] = [
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
    "positions_gk",
    "positions_dl",
    "positions_dc",
    "positions_dr",
    "positions_dm",
    "positions_ml",
    "positions_mc",
    "positions_mr",
    "positions_aml",
    "positions_amc",
    "positions_amr",
    "positions_stc",
    "positions_wbl",
    "positions_wbr",
    "personality_ambition",
    "personality_professionalism",
    "personality_pressure",
    "personality_loyalty",
    "personality_temperament",
    "personality_adaptability",
    "personality_sportsmanship",
    "personality_controversy",
    "attributes_crossing",
    "attributes_dribbling",
    "attributes_finishing",
    "attributes_heading",
    "attributes_long_shots",
    "attributes_marking",
    "attributes_off_the_ball",
    "attributes_passing",
    "attributes_penalty_taking",
    "attributes_tackling",
    "attributes_vision",
    "attributes_handling",
    "attributes_aerial_reach",
    "attributes_command_of_area",
    "attributes_communication",
    "attributes_kicking",
    "attributes_throwing",
    "attributes_anticipation",
    "attributes_decisions",
    "attributes_one_on_ones",
    "attributes_positioning",
    "attributes_reflexes",
    "attributes_first_touch",
    "attributes_technique",
    "attributes_flair",
    "attributes_corners",
    "attributes_teamwork",
    "attributes_work_rate",
    "attributes_long_throws",
    "attributes_eccentricity",
    "attributes_rushing_out",
    "attributes_punching",
    "attributes_acceleration",
    "attributes_free_kick_taking",
    "attributes_strength",
    "attributes_stamina",
    "attributes_pace",
    "attributes_jumping_reach",
    "attributes_leadership",
    "attributes_dirtiness",
    "attributes_balance",
    "attributes_bravery",
    "attributes_consistency",
    "attributes_aggression",
    "attributes_agility",
    "attributes_important_matches",
    "attributes_injury_proneness",
    "attributes_versatility",
    "attributes_natural_fitness",
    "attributes_determination",
    "attributes_composure",
    "attributes_concentration",
];

/// Negative FM potential-range code -> `[min, max]`.
pub const fn pa_range(code: i32) -> Option<[u16; 2]> {
    match code {
        -1 => Some([1, 20]),
        -2 => Some([10, 40]),
        -25 => Some([20, 50]),
        -3 => Some([30, 60]),
        -35 => Some([40, 70]),
        -4 => Some([50, 80]),
        -45 => Some([60, 90]),
        -5 => Some([70, 100]),
        -55 => Some([80, 110]),
        -6 => Some([90, 120]),
        -65 => Some([100, 130]),
        -7 => Some([110, 140]),
        -75 => Some([120, 150]),
        -8 => Some([130, 160]),
        -85 => Some([140, 170]),
        -9 => Some([150, 180]),
        -95 => Some([160, 190]),
        -10 => Some([170, 200]),
        _ => None,
    }
}

/// Fast RFC-4180 single-line CSV splitter.
pub fn split_csv_line(line: &str, delim: char) -> Vec<String> {
    if !line.contains('"') {
        return line.split(delim).map(|s| s.trim().to_string()).collect();
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            if in_quotes && chars.peek() == Some(&'"') {
                cur.push('"');
                chars.next();
            } else {
                in_quotes = !in_quotes;
            }
        } else if c == delim && !in_quotes {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn hidden_attr_from_suffix(suffix: &str) -> Option<ScoutHid> {
    match suffix {
        "consistency" => Some(ScoutHid::Cons),
        "important_matches" => Some(ScoutHid::Imp),
        "injury_proneness" => Some(ScoutHid::Inj),
        "versatility" => Some(ScoutHid::Vers),
        "dirtiness" => Some(ScoutHid::Dirt),
        _ => None,
    }
}

struct RowReader {
    attr_cols: Vec<(usize, Attr)>,
    hid_cols: Vec<(usize, ScoutHid)>,
    pos_cols: Vec<(usize, ScoutPosKey)>,
    pers_cols: Vec<(usize, ScoutPers)>,
    uid: Option<usize>,
    unique_id: Option<usize>,
    name: usize,
    age: Option<usize>,
    nation_id: Option<usize>,
    height_cm: Option<usize>,
    ca: usize,
    pa: Option<usize>,
    pa_code: Option<usize>,
    rep: Option<usize>,
    wrep: Option<usize>,
    club_uid: Option<usize>,
    club_name: Option<usize>,
    club_nation_id: Option<usize>,
    club_rep: Option<usize>,
    on_loan: Option<usize>,
    loan_parent: Option<usize>,
    value: Option<usize>,
    wage: Option<usize>,
    contract_end: Option<usize>,
    squad_status: Option<usize>,
    left_foot: Option<usize>,
    right_foot: Option<usize>,
    rows: usize,
}

impl RowReader {
    fn new(head: &[String], lang: Lang) -> Result<Self, String> {
        let mut ix: HashMap<&str, usize> = HashMap::with_capacity(head.len());
        for (i, h) in head.iter().enumerate() {
            ix.insert(h.trim(), i);
        }
        let col = |n: &str| ix.get(n).copied();
        let (Some(name), Some(ca)) = (col("name"), col("ability_current")) else {
            return Err(l(
                lang,
                "Este ficheiro não parece uma exportação de jogadores do fmsave (faltam as colunas name/ability_current).",
                "This file does not look like an fmsave players export (the name/ability_current columns are missing).",
            )
            .to_string());
        };

        let mut attr_cols = Vec::with_capacity(47);
        let mut hid_cols = Vec::with_capacity(5);
        for (i, h) in head.iter().enumerate() {
            if let Some(suf) = h.trim().strip_prefix("attributes_") {
                if let Some(hk) = hidden_attr_from_suffix(suf) {
                    hid_cols.push((i, hk));
                } else if let Some(a) = attr_from_norm_header(&norm_header(h)) {
                    attr_cols.push((i, a));
                }
            }
        }

        let pos_keys = [
            ScoutPosKey::Gk,
            ScoutPosKey::Dl,
            ScoutPosKey::Dc,
            ScoutPosKey::Dr,
            ScoutPosKey::Dm,
            ScoutPosKey::Ml,
            ScoutPosKey::Mc,
            ScoutPosKey::Mr,
            ScoutPosKey::Aml,
            ScoutPosKey::Amc,
            ScoutPosKey::Amr,
            ScoutPosKey::Stc,
            ScoutPosKey::Wbl,
            ScoutPosKey::Wbr,
        ];
        let mut pos_cols = Vec::with_capacity(14);
        for pk in pos_keys {
            let cname = format!("positions_{}", ScoutPosKey::NAMES[pk as usize]);
            if let Some(ci) = col(&cname) {
                pos_cols.push((ci, pk));
            }
        }

        let pers_keys = [
            ScoutPers::Ambition,
            ScoutPers::Professionalism,
            ScoutPers::Pressure,
            ScoutPers::Loyalty,
            ScoutPers::Temperament,
            ScoutPers::Adaptability,
            ScoutPers::Sportsmanship,
            ScoutPers::Controversy,
        ];
        let mut pers_cols = Vec::with_capacity(8);
        for pk in pers_keys {
            let cname = format!("personality_{}", ScoutPers::NAMES[pk as usize]);
            if let Some(ci) = col(&cname) {
                pers_cols.push((ci, pk));
            }
        }

        Ok(Self {
            attr_cols,
            hid_cols,
            pos_cols,
            pers_cols,
            uid: col("uid"),
            unique_id: col("unique_id"),
            name,
            age: col("age"),
            nation_id: col("nation_id"),
            height_cm: col("height_cm"),
            ca,
            pa: col("ability_potential"),
            pa_code: col("ability_potential_range_code"),
            rep: col("reputation_current"),
            wrep: col("reputation_world"),
            club_uid: col("club_uid"),
            club_name: col("club_name"),
            club_nation_id: col("club_nation_id"),
            club_rep: col("club_reputation"),
            on_loan: col("on_loan"),
            loan_parent: col("loan_parent_club_name"),
            value: col("transfer_value"),
            wage: col("contract_wage"),
            contract_end: col("contract_end"),
            squad_status: col("contract_squad_status"),
            left_foot: col("left_foot"),
            right_foot: col("right_foot"),
            rows: 0,
        })
    }

    #[inline]
    fn parse_f64(s: Option<&str>) -> Option<f64> {
        let s = s?.trim();
        if s.is_empty() {
            return None;
        }
        let v = s.parse::<f64>().ok()?;
        v.is_finite().then_some(v)
    }

    #[inline]
    fn parse_u8(s: Option<&str>) -> Option<u8> {
        Self::parse_f64(s).map(|v| v.round().clamp(0.0, 255.0) as u8)
    }

    #[inline]
    fn parse_u16(s: Option<&str>) -> Option<u16> {
        Self::parse_f64(s).map(|v| v.round().clamp(0.0, u16::MAX as f64) as u16)
    }

    #[inline]
    fn parse_i32(s: Option<&str>) -> Option<i32> {
        Self::parse_f64(s).map(|v| v.round() as i32)
    }

    #[inline]
    fn parse_i64(s: Option<&str>) -> Option<i64> {
        Self::parse_f64(s).map(|v| v.round() as i64)
    }

    fn read_line(&mut self, line: &str, fields_buf: &mut Vec<String>) -> Option<ScoutPlayer> {
        if line.is_empty() {
            return None;
        }
        // Fast path: zero quotes -> borrow slices directly without allocating per field!
        if !line.contains('"') {
            let parts: Vec<&str> = line.split(',').collect();
            return self.build_player(|idx| parts.get(idx).copied());
        }
        fields_buf.clear();
        let mut cur = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '"' {
                if in_quotes && chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            } else if c == ',' && !in_quotes {
                fields_buf.push(cur.trim().to_string());
                cur.clear();
            } else {
                cur.push(c);
            }
        }
        fields_buf.push(cur.trim().to_string());
        self.build_player(|idx| fields_buf.get(idx).map(|s| s.as_str()))
    }

    fn build_player<'a>(&mut self, get: impl Fn(usize) -> Option<&'a str>) -> Option<ScoutPlayer> {
        let name = get(self.name)?.trim();
        if name.is_empty() {
            return None;
        }
        let mut attrs = [0u8; Attr::COUNT];
        for &(ci, a) in &self.attr_cols {
            if let Some(v) = Self::parse_u8(get(ci)) {
                attrs[a.index()] = v;
            }
        }
        let mut hid = [0u8; ScoutHid::COUNT];
        for &(ci, h) in &self.hid_cols {
            if let Some(v) = Self::parse_u8(get(ci)) {
                hid[h as usize] = v;
            }
        }
        let mut pers = [0u8; ScoutPers::COUNT];
        for &(ci, p) in &self.pers_cols {
            if let Some(v) = Self::parse_u8(get(ci)) {
                pers[p as usize] = v;
            }
        }
        let mut pos_r = [0u8; ScoutPosKey::COUNT];
        let mut pos_mask = 0u32;
        let mut nat_pos_mask = 0u16;
        let mut good_pos: Vec<(u8, &'static str)> = Vec::new();
        for &(ci, pk) in &self.pos_cols {
            if let Some(v) = Self::parse_u8(get(ci)) {
                pos_r[pk as usize] = v;
                let (grp, side) = pk.group_and_side();
                if v >= 15 {
                    pos_mask |= grp.bit();
                    if let Some(s) = side {
                        pos_mask |= grp.side_bit(s);
                    }
                    good_pos.push((v, ScoutPosKey::NAMES[pk as usize]));
                }
                if v >= 18 {
                    nat_pos_mask |= grp.bit() as u16;
                }
            }
        }
        good_pos.sort_by_key(|a| std::cmp::Reverse(a.0));
        let pos_raw = good_pos
            .iter()
            .map(|(_, k)| k.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join(", ");

        let pa_direct = self.pa.and_then(|c| Self::parse_u16(get(c)));
        let pa_code = self.pa_code.and_then(|c| Self::parse_i32(get(c)));
        let rng = pa_code.and_then(pa_range);
        let (pa, pa_range_val) = match (pa_direct, rng) {
            (Some(v), _) => (Some(v), None),
            (None, Some(r)) => (
                Some(((r[0] as f32 + r[1] as f32) / 2.0).round() as u16),
                Some(r),
            ),
            (None, None) => (None, None),
        };

        let lf = self.left_foot.and_then(|c| Self::parse_u8(get(c)));
        let rf = self.right_foot.and_then(|c| Self::parse_u8(get(c)));
        let foot = match (lf, rf) {
            (Some(l), Some(r)) => Some(if l >= 15 && r >= 15 {
                Foot::B
            } else if l > r {
                Foot::L
            } else {
                Foot::R
            }),
            _ => None,
        };

        let uid_s = self
            .uid
            .and_then(&get)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let uniq_s = self
            .unique_id
            .and_then(&get)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let id = format!(
            "db{}",
            uid_s
                .as_deref()
                .map(str::to_string)
                .unwrap_or_else(|| self.rows.to_string())
        );

        let club = self
            .club_name
            .and_then(&get)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let club_lower = club.to_lowercase();
        let end_raw = self
            .contract_end
            .and_then(&get)
            .map(str::trim)
            .unwrap_or("");
        let end = if is_iso_date_prefix(end_raw) {
            end_raw[..10].to_string()
        } else {
            String::new()
        };
        let loan = self
            .on_loan
            .and_then(&get)
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("true"));

        self.rows += 1;
        Some(ScoutPlayer {
            id,
            uid: uid_s,
            uniq: uniq_s,
            name: name.to_string(),
            name_lower: name.to_lowercase(),
            age: self.age.and_then(|c| Self::parse_u16(get(c))),
            nat: self.nation_id.and_then(|c| Self::parse_i32(get(c))),
            height: self.height_cm.and_then(|c| Self::parse_u16(get(c))),
            ca: Self::parse_u16(get(self.ca)),
            pa,
            pa_range: pa_range_val,
            rep: self.rep.and_then(|c| Self::parse_i32(get(c))),
            wrep: self.wrep.and_then(|c| Self::parse_i32(get(c))),
            club,
            club_lower,
            club_uid: self
                .club_uid
                .and_then(&get)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            club_nat: self.club_nation_id.and_then(|c| Self::parse_i32(get(c))),
            club_rep: self.club_rep.and_then(|c| Self::parse_i32(get(c))),
            loan,
            loan_from: self
                .loan_parent
                .and_then(&get)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            value: self.value.and_then(|c| Self::parse_i64(get(c))),
            wage: self.wage.and_then(|c| Self::parse_i64(get(c))),
            end,
            status: self
                .squad_status
                .and_then(get)
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
            foot,
            lf,
            rf,
            attrs,
            pos_r,
            pos_mask,
            nat_pos_mask,
            pos_raw,
            pers,
            hid,
        })
    }
}

fn is_iso_date_prefix(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 10
        && b[0..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
}

/// Parse an `fmsave export players --all` CSV into a compact `ScoutDb`.
pub fn parse_scout_db(
    text: &str,
    lang: Lang,
    mut on_progress: impl FnMut(f32, usize),
) -> Result<ScoutDb, String> {
    let t0 = Instant::now();
    let src = text.strip_prefix('\u{feff}').unwrap_or(text);
    let total = src.len().max(1);
    let mut lines = src.lines();
    let head_line = lines.next().unwrap_or("");
    let head = split_csv_line(head_line.trim_end_matches('\r'), ',');
    let mut reader = RowReader::new(&head, lang)?;
    let attr_cols = reader.attr_cols.len();
    let est_rows = (src.len() / 240).max(64);
    let mut players = Vec::with_capacity(est_rows);
    let mut fields_buf = Vec::with_capacity(head.len());
    let mut consumed = head_line.len() + 1;
    let mut counter = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end_matches('\r');
        consumed += raw_line.len() + 1;
        if let Some(p) = reader.read_line(line, &mut fields_buf) {
            players.push(p);
        }
        counter += 1;
        if counter & 2047 == 0 {
            on_progress((consumed as f32 / total as f32).min(1.0), players.len());
        }
    }
    on_progress(1.0, players.len());

    Ok(ScoutDb {
        players,
        nations: HashMap::new(),
        date: None,
        date_guess: false,
        save: String::new(),
        cached: false,
        parse_ms: t0.elapsed().as_millis() as u64,
        columns: head.len(),
        attr_cols,
    })
}

/// Synchronous parse for small fmsave exports (squad enrichment).
pub fn parse_scout_sync(text: &str, lang: Lang) -> Result<Vec<ScoutPlayer>, String> {
    Ok(parse_scout_db(text, lang, |_, _| {})?.players)
}

/// Parse `fmsave export nations --all` CSV into `nation_id -> name`.
pub fn parse_nations(text: &str) -> HashMap<i32, String> {
    let mut map = HashMap::new();
    let src = text.strip_prefix('\u{feff}').unwrap_or(text).trim();
    if src.is_empty() {
        return map;
    }
    let mut lines = src.lines().filter(|l| !l.trim().is_empty());
    let Some(first) = lines.next() else {
        return map;
    };
    let delim = if first.split('\t').count() > first.split(',').count() {
        '\t'
    } else if first.split(';').count() > first.split(',').count() {
        ';'
    } else {
        ','
    };
    let head: Vec<String> = split_csv_line(first.trim_end_matches('\r'), delim)
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect();
    let id_col = ["uid", "nation_id", "id"]
        .iter()
        .find_map(|k| head.iter().position(|h| h == k));
    let nm_col = ["name", "nation_name", "short_name"]
        .iter()
        .find_map(|k| head.iter().position(|h| h == k));
    let (Some(ic), Some(nc)) = (id_col, nm_col) else {
        return map;
    };
    for line in lines {
        let row = split_csv_line(line.trim_end_matches('\r'), delim);
        if let (Some(id_s), Some(nm)) = (row.get(ic), row.get(nc)) {
            if !nm.is_empty() {
                if let Ok(id) = id_s.trim().parse::<i32>() {
                    map.insert(id, nm.clone());
                }
            }
        }
    }
    map
}

/// Extract game date (`YYYY-MM-DD`) from `fmsave info --json`.
pub fn game_date_from_info(json: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(json).ok()?;
    fn walk(v: &serde_json::Value, key: &str) -> Option<String> {
        match v {
            serde_json::Value::String(s) => {
                let low = key.to_lowercase();
                if is_iso_date_prefix(s)
                    && (low.contains("date")
                        || low.contains("day")
                        || low.contains("clock")
                        || low.contains("current"))
                {
                    return Some(s[..10].to_string());
                }
                None
            }
            serde_json::Value::Object(map) => {
                for (k, child) in map {
                    if let Some(hit) = walk(child, k) {
                        return Some(hit);
                    }
                }
                None
            }
            serde_json::Value::Array(arr) => {
                for child in arr {
                    if let Some(hit) = walk(child, key) {
                        return Some(hit);
                    }
                }
                None
            }
            _ => None,
        }
    }
    walk(&val, "")
}

/// Infer game date when `fmsave info` is absent (most common earliest contract-end year -> Jan 15).
pub fn infer_game_date(players: &[ScoutPlayer]) -> Option<String> {
    let mut years: HashMap<&str, usize> = HashMap::new();
    let mut total = 0usize;
    for p in players {
        if p.end.len() >= 4 {
            *years.entry(&p.end[..4]).or_insert(0) += 1;
            total += 1;
        }
    }
    if total == 0 {
        return None;
    }
    let mut ys: Vec<&str> = years.keys().copied().collect();
    ys.sort_unstable();
    let thresh = (total as f64) * 0.04;
    let first = ys
        .iter()
        .copied()
        .find(|y| (years[y] as f64) >= thresh)
        .unwrap_or(ys[0]);
    Some(format!("{first}-01-15"))
}

/// Convert `YYYY-MM-DD` to days since Unix epoch (Howard Hinnant's algorithm).
fn days_from_iso(s: &str) -> Option<i64> {
    if !is_iso_date_prefix(s) {
        return None;
    }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y_adj = y - if m <= 2 { 1 } else { 0 };
    let era = y_adj.div_euclid(400);
    let yoe = y_adj.rem_euclid(400) as u64;
    let m_adj = (m + if m > 2 { -3 } else { 9 }) as u64;
    let doy = (153 * m_adj + 2) / 5 + (d as u64) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + (doe as i64) - 719_468)
}

/// Approximate months between `from_iso` and `date_iso`.
pub fn months_until(date_iso: &str, from_iso: &str) -> Option<f64> {
    let a = days_from_iso(from_iso)?;
    let b = days_from_iso(date_iso)?;
    Some((b - a) as f64 / 30.44)
}

// ============================================================================
// Derived Heuristics
// ============================================================================

/// Estimated likelihood (0..=100) that a young player reaches their potential.
pub fn reach_likelihood(p: &ScoutPlayer) -> Option<u8> {
    let (pa, ca) = (p.pa?, p.ca?);
    let prof = p.pers_val(ScoutPers::Professionalism).unwrap_or(10) as f64;
    let amb = p.pers_val(ScoutPers::Ambition).unwrap_or(10) as f64;
    let inj = p.hid_val(ScoutHid::Inj).unwrap_or(10) as f64;
    let cons = p.hid_val(ScoutHid::Cons).unwrap_or(10) as f64;
    let det = p.attr(Attr::Det).unwrap_or(10.0) as f64;
    reach_likelihood_core(
        pa as f64,
        ca as f64,
        p.age.unwrap_or(20) as f64,
        prof,
        amb,
        det,
        inj,
        cons,
        p.club_rep.map(|v| v as f64),
    )
}

/// Same PA-reach heuristic for `Player` (Squad table).
pub fn reach_likelihood_player(p: &Player) -> Option<u8> {
    let (pa, ca) = (p.pa?, p.ca?);
    let prof = p.pers.get("professionalism").copied().unwrap_or(10.0) as f64;
    let amb = p.pers.get("ambition").copied().unwrap_or(10.0) as f64;
    let inj = p.hid.get("inj").copied().unwrap_or(10.0) as f64;
    let cons = p.hid.get("cons").copied().unwrap_or(10.0) as f64;
    let det = p.attr(Attr::Det).unwrap_or(10.0) as f64;
    reach_likelihood_core(
        pa as f64,
        ca as f64,
        p.age.unwrap_or(20.0) as f64,
        prof,
        amb,
        det,
        inj,
        cons,
        p.club_rep.map(|v| v as f64),
    )
}

#[allow(clippy::too_many_arguments)]
fn reach_likelihood_core(
    pa: f64,
    ca: f64,
    age: f64,
    prof: f64,
    amb: f64,
    det: f64,
    inj: f64,
    cons: f64,
    club_rep: Option<f64>,
) -> Option<u8> {
    let age_f = if age <= 17.0 {
        1.0
    } else if age <= 18.0 {
        0.92
    } else if age <= 19.0 {
        0.82
    } else if age <= 20.0 {
        0.70
    } else if age <= 21.0 {
        0.56
    } else if age <= 23.0 {
        0.38
    } else {
        0.20
    };
    let gap = (pa - ca).max(0.0);
    let gap_f = if gap == 0.0 {
        1.0
    } else {
        (1.0 - gap / 120.0).max(0.0)
    };
    let env = club_rep
        .map(|r| (0.45 + r / 10000.0 * 0.7).min(1.0))
        .unwrap_or(0.7);
    let s = prof / 20.0 * 0.30
        + det / 20.0 * 0.18
        + amb / 20.0 * 0.10
        + (21.0 - inj) / 20.0 * 0.10
        + cons / 20.0 * 0.04
        + age_f * 0.16
        + gap_f * 0.06
        + env * 0.06;
    Some((s * 100.0).min(100.0).round() as u8)
}

fn is_fringe_status(status: &str) -> bool {
    let low = status.to_lowercase();
    [
        "backup",
        "not needed",
        "surplus",
        "rotation",
        "youngster",
        "emergency",
        "reserve",
        "prospect",
    ]
    .iter()
    .any(|kw| low.contains(kw))
}

/// Estimated likelihood (0..=100) that a player would join the user's club.
pub fn join_likelihood(
    p: &ScoutPlayer,
    ctx: &ScoutContext<'_>,
    lang: Lang,
) -> Option<JoinLikelihood> {
    let my_rep = ctx.my_rep? as f64;
    let mut s = 50.0f64;
    let mut why = Vec::new();
    if p.club.is_empty() {
        s += 18.0;
        why.push(l(lang, "sem clube", "no club").to_string());
    } else if let Some(cr) = p.club_rep {
        let d = (my_rep - cr as f64) / 1000.0;
        s += (d * 9.0).clamp(-30.0, 22.0);
        if d >= 0.5 {
            why.push(
                l(
                    lang,
                    "o teu clube é maior que o dele",
                    "your club is bigger than his",
                )
                .to_string(),
            );
        } else if d <= -0.5 {
            why.push(
                l(
                    lang,
                    "o clube dele é maior que o teu",
                    "his club is bigger than yours",
                )
                .to_string(),
            );
        } else {
            why.push(l(lang, "clubes de dimensão parecida", "clubs of similar size").to_string());
        }
    }
    if let Some(rep) = p.rep {
        if (rep as f64) > my_rep {
            s -= ((rep as f64 - my_rep) / 1000.0 * 10.0).min(30.0);
            why.push(
                l(
                    lang,
                    "a reputação dele está acima da do teu clube",
                    "his reputation is above your club’s",
                )
                .to_string(),
            );
        }
    }
    if let Some(from) = ctx.date {
        if let Some(m) = months_until(&p.end, from) {
            if m <= 6.0 {
                s += 10.0;
                why.push(
                    l(
                        lang,
                        "contrato a acabar em 6 meses",
                        "contract ending within 6 months",
                    )
                    .to_string(),
                );
            } else if m <= 12.0 {
                s += 6.0;
                why.push(
                    l(
                        lang,
                        "contrato a acabar em 12 meses",
                        "contract ending within 12 months",
                    )
                    .to_string(),
                );
            }
        }
    }
    if !p.status.is_empty() && is_fringe_status(&p.status) {
        s += 10.0;
        why.push(
            l(
                lang,
                "não é titular no clube dele",
                "not a regular at his club",
            )
            .to_string(),
        );
    }
    let up = p.club.is_empty() || p.club_rep.is_none_or(|cr| my_rep >= cr as f64);
    if let Some(amb) = p.pers_val(ScoutPers::Ambition) {
        let a = amb as f64;
        s += (if up { 1.0 } else { -1.0 }) * (a - 10.0) * 1.2;
        if amb >= 15 {
            why.push(if up {
                l(lang, "ambicioso: quer subir", "ambitious: wants to step up").to_string()
            } else {
                l(
                    lang,
                    "ambicioso: não quer descer",
                    "ambitious: does not want a step down",
                )
                .to_string()
            });
        }
    }
    if let Some(loy) = p.pers_val(ScoutPers::Loyalty) {
        if !p.club.is_empty() {
            s -= (loy as f64 - 10.0) * 1.0;
            if loy >= 15 {
                why.push(
                    l(
                        lang,
                        "muito leal ao clube actual",
                        "very loyal to his current club",
                    )
                    .to_string(),
                );
            }
        }
    }
    Some(JoinLikelihood {
        score: s.clamp(0.0, 100.0).round() as u8,
        why,
    })
}

// ============================================================================
// XI Gain & Best Roles
// ============================================================================

/// Where a candidate player would fit in `tactic`'s XI and by how much they improve the starter.
pub fn best_gain(p: &impl AttrLookup, tactic: &Tactic, squad: &[Player]) -> Option<BestGain> {
    if tactic.slots.is_empty() {
        return None;
    }
    let pm: HashMap<&str, &Player> = squad.iter().map(|x| (x.id.as_str(), x)).collect();
    let mut best: Option<BestGain> = None;
    for s in &tactic.slots {
        let Some(a) = slot_info(&tactic.ip_form, &s.ip_code) else {
            continue;
        };
        if pos_factor(p, a.group, a.side) < 0.85 {
            continue;
        }
        let mine = s.pid.as_deref().and_then(|id| pm.get(id).copied());
        let cur = mine.map(|m| slot_score(m, tactic, s)).unwrap_or(0);
        let his = slot_score(p, tactic, s);
        let gain = his as i16 - cur as i16;
        if best.as_ref().is_none_or(|b| gain > b.gain) {
            best = Some(BestGain {
                code: s.ip_code.clone(),
                gain,
                his,
                cur,
                starter: mine.map(|m| m.name.clone()).unwrap_or_default(),
            });
        }
    }
    best
}

/// Top `n` IP roles across all position groups where the player is at least accomplished.
pub fn best_roles_any(p: &impl AttrLookup, n: usize) -> Vec<(PosGroup, &'static Role, u8)> {
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut out = Vec::new();
    for g in PosGroup::ALL {
        if !p.has_pos_group(g) {
            continue;
        }
        for r in roles_for(g, Phase::Ip, None, g == PosGroup::Dc) {
            if seen.insert(r.id) {
                let sc = role_score(p, r).map(|f| f.score).unwrap_or(0);
                out.push((g, r, sc));
            }
        }
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.2));
    out.truncate(n);
    out
}

// ============================================================================
// Filter & Sort Engine
// ============================================================================

/// Filter and sort `db` according to `f` and `ctx`, matching JS `scoutFilter`.
pub fn scout_filter(
    db: &ScoutDb,
    f: &ScoutFilter,
    ctx: &ScoutContext<'_>,
    lang: Lang,
) -> Vec<ScoutMatch> {
    let q = f.q.trim().to_lowercase();
    let club_q = f.club.trim().to_lowercase();
    let mine = ctx.my_club.map(|s| s.to_lowercase());
    let active_attrs: Vec<(Attr, u8)> = f
        .attrs
        .iter()
        .filter_map(|x| Some((x.k?, x.min?)))
        .collect();

    let mut matches: Vec<ScoutMatch> = Vec::new();
    for (idx, p) in db.players.iter().enumerate() {
        if !q.is_empty() && !p.name_lower.contains(&q) {
            continue;
        }
        if let Some(v) = f.age_min {
            if !p.age.is_some_and(|a| a >= v) {
                continue;
            }
        }
        if let Some(v) = f.age_max {
            if !p.age.is_some_and(|a| a <= v) {
                continue;
            }
        }
        if let Some(v) = f.ca_min {
            if !p.ca.is_some_and(|a| a >= v) {
                continue;
            }
        }
        if let Some(v) = f.ca_max {
            if !p.ca.is_some_and(|a| a <= v) {
                continue;
            }
        }
        if let Some(v) = f.pa_min {
            if !p.pa.is_some_and(|a| a >= v) {
                continue;
            }
        }
        if let Some(v) = f.pa_max {
            if !p.pa.is_some_and(|a| a <= v) {
                continue;
            }
        }
        if let Some(pos) = f.pos {
            let ok = match f.pos_level {
                PosLevel::Nat => p.has_nat_group(pos),
                PosLevel::Acc => p.has_pos_group(pos),
            };
            if !ok {
                continue;
            }
        }
        if let Some(ft) = f.foot {
            let ok = match ft {
                Foot::L => p.lf.is_some_and(|v| v >= 15),
                Foot::R => p.rf.is_some_and(|v| v >= 15),
                Foot::B => p.foot == Some(Foot::B),
            };
            if !ok {
                continue;
            }
        }
        if let Some(nat) = f.nation {
            if p.nat != Some(nat) {
                continue;
            }
        }
        if let Some(lg) = f.league {
            if p.club_nat != Some(lg) {
                continue;
            }
        }
        if !club_q.is_empty() && !p.club_lower.contains(&club_q) {
            continue;
        }
        if f.free && !p.club.is_empty() {
            continue;
        }
        if let Some(max_rep) = f.club_rep_max {
            if p.club_rep.is_some_and(|r| r > max_rep) {
                continue;
            }
        }
        if f.exclude_mine {
            if let Some(ref mc) = mine {
                if !mc.is_empty() && p.club_lower == *mc {
                    continue;
                }
            }
        }
        match f.loan {
            LoanFilter::Exclude if p.loan => continue,
            LoanFilter::Only if !p.loan => continue,
            _ => {}
        }
        // Unknown value/wage excluded when max is set — EXCEPT free agents (no club).
        if let Some(vmax) = f.value_max {
            match p.value {
                None => {
                    if !p.club.is_empty() {
                        continue;
                    }
                }
                Some(v) if v > vmax => continue,
                _ => {}
            }
        }
        if let Some(wmax) = f.wage_max {
            match p.wage {
                None => {
                    if !p.club.is_empty() {
                        continue;
                    }
                }
                Some(w) if w > wmax => continue,
                _ => {}
            }
        }
        if let Some(end_m) = f.end_months {
            let Some(from) = ctx.date else {
                continue;
            };
            let Some(m) = months_until(&p.end, from) else {
                continue;
            };
            if m > end_m as f64 || m < -1.0 {
                continue;
            }
        }
        if let Some(v) = f.prof_min {
            if p.pers_val(ScoutPers::Professionalism).unwrap_or(0) < v {
                continue;
            }
        }
        if let Some(v) = f.amb_min {
            if p.pers_val(ScoutPers::Ambition).unwrap_or(0) < v {
                continue;
            }
        }
        if let Some(v) = f.det_min {
            if p.attrs[Attr::Det.index()] < v {
                continue;
            }
        }
        if let Some(v) = f.cons_min {
            if p.hid_val(ScoutHid::Cons).unwrap_or(0) < v {
                continue;
            }
        }
        if let Some(v) = f.imp_min {
            if p.hid_val(ScoutHid::Imp).unwrap_or(0) < v {
                continue;
            }
        }
        if let Some(v) = f.inj_max {
            if p.hid_val(ScoutHid::Inj).unwrap_or(20) > v {
                continue;
            }
        }
        if let Some(v) = f.height_min {
            if p.height.unwrap_or(0) < v {
                continue;
            }
        }
        if active_attrs
            .iter()
            .any(|&(a, min)| p.attrs[a.index()] < min)
        {
            continue;
        }
        matches.push(ScoutMatch {
            index: idx,
            likely: None,
            join: None,
            role_score: None,
            gain: None,
            gain_slot: None,
        });
    }

    // Expensive metrics computed only on already-filtered candidates
    if f.likely_min.is_some() || f.sort == ScoutSort::Likely {
        for m in &mut matches {
            m.likely = reach_likelihood(&db.players[m.index]);
        }
        if let Some(min_lk) = f.likely_min {
            matches.retain(|m| m.likely.unwrap_or(0) >= min_lk);
        }
    }

    if (f.join_min.is_some() || f.sort == ScoutSort::Join) && ctx.my_rep.is_some() {
        for m in &mut matches {
            m.join = join_likelihood(&db.players[m.index], ctx, lang).map(|j| j.score);
        }
        if let Some(min_jn) = f.join_min {
            matches.retain(|m| m.join.unwrap_or(0) >= min_jn);
        }
    }

    if let Some(ref rid) = f.role {
        if let Some(role) = role_by_id(rid) {
            if matches.len() > 30_000 {
                matches.sort_by(|a, b| {
                    db.players[b.index]
                        .ca
                        .unwrap_or(0)
                        .cmp(&db.players[a.index].ca.unwrap_or(0))
                });
                matches.truncate(30_000);
            }
            for m in &mut matches {
                m.role_score = Some(
                    role_score(&db.players[m.index], role)
                        .map(|rf| rf.score)
                        .unwrap_or(0),
                );
            }
            if let Some(rm) = f.role_min {
                matches.retain(|m| m.role_score.unwrap_or(0) >= rm);
            }
        }
    }

    if f.sort == ScoutSort::Gain {
        if let Some(tactic) = ctx.tactic {
            if !ctx.squad.is_empty() {
                matches.sort_by(|a, b| {
                    db.players[b.index]
                        .ca
                        .unwrap_or(0)
                        .cmp(&db.players[a.index].ca.unwrap_or(0))
                });
                matches.truncate(3_000);
                for m in &mut matches {
                    if let Some(g) = best_gain(&db.players[m.index], tactic, ctx.squad) {
                        m.gain = Some(g.gain);
                        m.gain_slot = Some(g.code);
                    } else {
                        m.gain = Some(-99);
                        m.gain_slot = Some(String::new());
                    }
                }
            }
        }
    }

    let asc = f.dir == SortDir::Asc;
    matches.sort_by(|x, y| {
        let px = &db.players[x.index];
        let py = &db.players[y.index];
        let cmp_opt = |a: Option<i64>, b: Option<i64>| -> std::cmp::Ordering {
            match (a, b) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(_), None) => std::cmp::Ordering::Less,
                (Some(av), Some(bv)) => {
                    if asc {
                        av.cmp(&bv)
                    } else {
                        bv.cmp(&av)
                    }
                }
            }
        };
        let cmp_str = |a: &str, b: &str| -> std::cmp::Ordering {
            match (a.is_empty(), b.is_empty()) {
                (true, true) => std::cmp::Ordering::Equal,
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                (false, false) => {
                    if asc {
                        a.cmp(b)
                    } else {
                        b.cmp(a)
                    }
                }
            }
        };
        match f.sort {
            ScoutSort::Ca => cmp_opt(px.ca.map(i64::from), py.ca.map(i64::from)),
            ScoutSort::Pa => cmp_opt(px.pa.map(i64::from), py.pa.map(i64::from)),
            ScoutSort::Age => cmp_opt(px.age.map(i64::from), py.age.map(i64::from)),
            ScoutSort::Value => cmp_opt(px.value, py.value),
            ScoutSort::Wage => cmp_opt(px.wage, py.wage),
            ScoutSort::Likely => cmp_opt(x.likely.map(i64::from), y.likely.map(i64::from)),
            ScoutSort::Role => cmp_opt(x.role_score.map(i64::from), y.role_score.map(i64::from)),
            ScoutSort::Gain => cmp_opt(x.gain.map(i64::from), y.gain.map(i64::from)),
            ScoutSort::Join => cmp_opt(x.join.map(i64::from), y.join.map(i64::from)),
            ScoutSort::Name => cmp_str(&px.name, &py.name),
            ScoutSort::End => cmp_str(&px.end, &py.end),
        }
    });

    matches
}

// ============================================================================
// 11 Predefined Scouting Presets
// ============================================================================

pub fn scout_presets(ctx: &ScoutContext<'_>, lang: Lang) -> Vec<ScoutPreset> {
    let xi_ca = if ctx.xi_ca > 0 { ctx.xi_ca } else { 120 };
    let cheap = ctx.cheap_value.unwrap_or(3_000_000);
    let bargain = ctx.bargain_value.unwrap_or(5_000_000);
    let d = ScoutFilter::default;

    vec![
        ScoutPreset {
            key: "cheap_wonder",
            icon: "💎",
            name: l(lang, "Wonderkids baratos", "Cheap wonderkids").to_string(),
            desc: l(
                lang,
                "Até 19 anos, potencial ≥ 150 e valor baixo.",
                "Up to 19, potential ≥ 150 and a low value.",
            )
            .to_string(),
            filter: ScoutFilter {
                age_max: Some(19),
                pa_min: Some(150),
                value_max: Some(cheap),
                sort: ScoutSort::Pa,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "reach",
            icon: "📈",
            name: l(
                lang,
                "Wonderkids que vão lá chegar",
                "Wonderkids likely to reach potential",
            )
            .to_string(),
            desc: l(
                lang,
                "Até 20 anos, potencial ≥ 145 e alta probabilidade de o atingir (profissionalismo, determinação, ambição, lesões).",
                "Up to 20, potential ≥ 145 and a high chance of reaching it (professionalism, determination, ambition, injuries).",
            )
            .to_string(),
            filter: ScoutFilter {
                age_max: Some(20),
                pa_min: Some(145),
                likely_min: Some(70),
                sort: ScoutSort::Likely,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "expiring",
            icon: "⏳",
            name: l(lang, "Contrato a terminar", "Contract expiring").to_string(),
            desc: match lang {
                Lang::Pt => format!(
                    "Contrato acaba nos próximos 6 meses e CA perto do teu XI (≥ {}).",
                    xi_ca.saturating_sub(15).max(60)
                ),
                Lang::En => format!(
                    "Contract ends within 6 months and CA close to your XI (≥ {}).",
                    xi_ca.saturating_sub(15).max(60)
                ),
            },
            filter: ScoutFilter {
                end_months: Some(6),
                ca_min: Some(xi_ca.saturating_sub(15).max(60)),
                age_max: Some(32),
                sort: ScoutSort::Ca,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "interested",
            icon: "🤝",
            name: l(
                lang,
                "Interessados em vir (estimativa)",
                "Likely to join (estimate)",
            )
            .to_string(),
            desc: l(
                lang,
                "Alta probabilidade de aceitar vir para o teu clube, ao nível do teu XI. Estimativa: o fmsave não lê o “interessado” do jogo.",
                "High chance of agreeing to join your club, at your XI’s level. An estimate: fmsave does not read the game’s “interested” flag.",
            )
            .to_string(),
            filter: ScoutFilter {
                join_min: Some(70),
                ca_min: Some(xi_ca.saturating_sub(10).max(60)),
                sort: ScoutSort::Join,
                ..d()
            },
            need_squad: false,
            need_my_club: true,
            weak_spot: false,
        },
        ScoutPreset {
            key: "free",
            icon: "🆓",
            name: l(lang, "Jogadores livres", "Free agents").to_string(),
            desc: l(
                lang,
                "Sem clube: sem custo de transferência.",
                "Without a club: no transfer fee.",
            )
            .to_string(),
            filter: ScoutFilter {
                free: true,
                ca_min: Some(xi_ca.saturating_sub(30).max(50)),
                sort: ScoutSort::Ca,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "bargain",
            icon: "🏷",
            name: l(lang, "Pechinchas", "Bargains").to_string(),
            desc: l(
                lang,
                "Entre 21 e 29 anos, CA ao nível do teu XI e valor baixo.",
                "Between 21 and 29, CA at your XI’s level and a low value.",
            )
            .to_string(),
            filter: ScoutFilter {
                age_min: Some(21),
                age_max: Some(29),
                ca_min: Some(xi_ca.saturating_sub(5).max(60)),
                value_max: Some(bargain),
                sort: ScoutSort::Ca,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "upgrade",
            icon: "⬆",
            name: l(lang, "Reforços para a minha tática", "Upgrades for my tactic").to_string(),
            desc: l(
                lang,
                "Quem mais melhora o teu XI, posição a posição (adequação ao papel da tua tática).",
                "Who improves your XI the most, position by position (fit for your tactic’s role).",
            )
            .to_string(),
            filter: ScoutFilter {
                ca_min: Some(xi_ca.saturating_sub(10).max(60)),
                sort: ScoutSort::Gain,
                ..d()
            },
            need_squad: true,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "veteran",
            icon: "🎖",
            name: l(lang, "Veteranos a custo zero", "Veterans on a free").to_string(),
            desc: l(
                lang,
                "30 anos ou mais, qualidade alta e contrato a acabar em 12 meses (ou sem clube).",
                "30 or older, high quality and a contract ending within 12 months (or no club).",
            )
            .to_string(),
            filter: ScoutFilter {
                age_min: Some(30),
                ca_min: Some(xi_ca.max(60)),
                end_months: Some(12),
                sort: ScoutSort::Ca,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "hidden",
            icon: "🔍",
            name: l(lang, "Joias escondidas", "Hidden gems").to_string(),
            desc: l(
                lang,
                "Até 21 anos, potencial ≥ 140 em clubes pequenos (reputação baixa).",
                "Up to 21, potential ≥ 140 at small clubs (low reputation).",
            )
            .to_string(),
            filter: ScoutFilter {
                age_max: Some(21),
                pa_min: Some(140),
                value_max: Some(cheap),
                club_rep_max: Some(4000),
                sort: ScoutSort::Pa,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "elite_pers",
            icon: "🧠",
            name: l(lang, "Personalidade de elite", "Elite personality").to_string(),
            desc: l(
                lang,
                "Até 23 anos, Profissionalismo ≥ 17 e Determinação ≥ 15: bons para o balneário e para evoluir.",
                "Up to 23, Professionalism ≥ 17 and Determination ≥ 15: good for the dressing room and for development.",
            )
            .to_string(),
            filter: ScoutFilter {
                age_max: Some(23),
                prof_min: Some(17),
                det_min: Some(15),
                pa_min: Some(120),
                sort: ScoutSort::Pa,
                ..d()
            },
            need_squad: false,
            need_my_club: false,
            weak_spot: false,
        },
        ScoutPreset {
            key: "weak_spot",
            icon: "🎯",
            name: l(lang, "A minha posição mais fraca", "My weakest position").to_string(),
            desc: l(
                lang,
                "Jogadores naturais na posição onde o teu XI é mais fraco.",
                "Natural players for the position where your XI is weakest.",
            )
            .to_string(),
            filter: ScoutFilter {
                ca_min: Some(xi_ca.saturating_sub(10).max(60)),
                sort: ScoutSort::Gain,
                ..d()
            },
            need_squad: true,
            need_my_club: false,
            weak_spot: true,
        },
    ]
}

// ============================================================================
// Squad Enrichment from fmsave CSV
// ============================================================================

/// Enrich parsed squad players with fmsave-only fields (CA/PA, personality, hidden attrs, contract, value).
/// Returns number of enriched players. Leaves non-fmsave CSVs untouched (`0`).
pub fn enrich_from_scout(players: &mut [Player], text: &str) -> usize {
    let probe_end = text.len().min(6000);
    // Find a valid UTF-8 boundary at or before `probe_end`.
    let mut boundary = probe_end;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    if !text[..boundary].contains("ability_current") {
        return 0;
    }
    let Ok(rich) = parse_scout_sync(text, Lang::En) else {
        return 0;
    };
    let mut by_uid: HashMap<&str, usize> = HashMap::new();
    let mut by_name: HashMap<&str, usize> = HashMap::new();
    for (i, x) in rich.iter().enumerate() {
        if let Some(ref u) = x.uid {
            by_uid.insert(u.as_str(), i);
        }
        by_name.insert(x.name.as_str(), i);
    }

    let mut count = 0usize;
    for p in players.iter_mut() {
        let hit_idx = p
            .meta
            .uid
            .as_deref()
            .and_then(|u| by_uid.get(u).copied())
            .or_else(|| by_name.get(p.name.as_str()).copied());
        let Some(idx) = hit_idx else {
            continue;
        };
        let x = &rich[idx];
        if let Some(v) = x.ca {
            p.ca = Some(v as f32);
        }
        if let Some(v) = x.pa {
            p.pa = Some(v as f32);
        }
        if x.pa_range.is_some() {
            p.pa_range = x.pa_range;
        }
        for (i, &name) in ScoutPers::NAMES.iter().enumerate() {
            if x.pers[i] > 0 {
                p.pers.insert(name.to_string(), x.pers[i] as f32);
            }
        }
        for (i, &key) in ScoutHid::KEYS.iter().enumerate() {
            if x.hid[i] > 0 {
                p.hid.insert(key.to_string(), x.hid[i] as f32);
            }
        }
        if let Some(v) = x.value {
            p.value = Some(v as f64);
        }
        if let Some(w) = x.wage {
            p.wage = Some(w as f64);
        }
        if !x.end.is_empty() {
            p.end = Some(x.end.clone());
        }
        if !x.status.is_empty() {
            p.status = Some(x.status.clone());
        }
        if let Some(v) = x.rep {
            p.rep = Some(v as f32);
        }
        if let Some(v) = x.wrep {
            p.wrep = Some(v as f32);
        }
        if let Some(v) = x.club_rep {
            p.club_rep = Some(v as f32);
        }
        if x.club_nat.is_some() {
            p.club_nat = x.club_nat;
        }
        if let Some(v) = x.height {
            p.height = Some(v as f32);
        }
        if let Some(v) = x.lf {
            p.lf = Some(v as f32);
        }
        if let Some(v) = x.rf {
            p.rf = Some(v as f32);
        }
        if x.foot.is_some() {
            p.foot = x.foot;
        }
        p.loan = Some(x.loan);
        if x.nat.is_some() {
            p.nat = x.nat;
        }
        if x.uid.is_some() {
            p.uid = x.uid.clone();
        }
        if x.uniq.is_some() {
            p.uniq = x.uniq.clone();
        }
        if !x.club.is_empty() {
            p.club = Some(x.club.clone());
        }
        for (i, &name) in ScoutPosKey::NAMES.iter().enumerate() {
            if x.pos_r[i] > 0 {
                p.pos_r.insert(name.to_string(), x.pos_r[i] as f32);
            }
        }
        let mut nat_pos = Vec::new();
        for g in PosGroup::ALL {
            if x.has_nat_group(g) {
                nat_pos.push(g.code().to_string());
            }
        }
        p.nat_pos = nat_pos;
        count += 1;
    }
    count
}
