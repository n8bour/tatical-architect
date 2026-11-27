//! Parsers for FM squad tables (CSV/TSV/semicolon/pipe/HTML), position strings,
//! names/clubs, and FM27 `.fmf` binary container inspection.

use crate::data::roles::{ATTR_META, ROLES};
use crate::data::tactics_data::{FORMATIONS, TEAM_INSTRUCTIONS};
use crate::types::{
    Attr, FmfBlock, FmfContainer, FmfInspectReport, FmfMatches, FmfSection, FmfStream, Lang,
    MajorityClub, MappedColumn, Mentality, ParsedSquad, Player, PlayerMeta,
};
use flate2::read::{GzDecoder, ZlibDecoder};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;

const POS_ALIASES: &[&str] = &[
    "position",
    "positions",
    "posição",
    "posicao",
    "posições",
    "pos.",
];
const NAME_ALIASES: &[&str] = &["name", "nome", "player", "jogador"];
const AGE_ALIASES: &[&str] = &["age", "idade"];

const NAME_PARTICLES: &[&str] = &[
    "de", "da", "das", "do", "dos", "di", "del", "della", "van", "von", "der", "den", "ten",
    "ter", "le", "la", "du", "mac", "al", "el", "bin", "st.",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum HeaderKey {
    Name,
    Position,
    Age,
    Meta(&'static str),
    PosRating(&'static str),
    Attr(Attr),
}

impl HeaderKey {
    fn as_mapped_string(&self) -> String {
        match self {
            Self::Name => "name".to_string(),
            Self::Position => "position".to_string(),
            Self::Age => "age".to_string(),
            Self::Meta(k) => format!("meta:{k}"),
            Self::PosRating(k) => format!("posr:{k}"),
            Self::Attr(a) => a.code().to_string(),
        }
    }
}

fn meta_alias(n: &str) -> Option<&'static str> {
    match n {
        "best pos" | "best position" => Some("bestPos"),
        "best role" => Some("bestRole"),
        "ability" => Some("ability"),
        "potential" => Some("potential"),
        "ability current" | "current ability" | "ca" => Some("ca"),
        "ability potential" | "potential ability" | "pa" => Some("pa"),
        "unique id" | "uid" => Some("uid"),
        "personality" => Some("personality"),
        "club" | "club name" => Some("club"),
        "preferred foot" => Some("foot"),
        "height" => Some("height"),
        _ => None,
    }
}

fn pos_rating_tokens(code: &str) -> &'static [&'static str] {
    match code {
        "gk" => &["GK"],
        "dl" => &["D_LR", "D_LR:L"],
        "dc" => &["DC"],
        "dr" => &["D_LR", "D_LR:R"],
        "wbl" => &["WB_LR", "WB_LR:L"],
        "wbr" => &["WB_LR", "WB_LR:R"],
        "dm" | "dmc" => &["DM"],
        "ml" => &["M_LR", "M_LR:L"],
        "mc" => &["MC"],
        "mr" => &["M_LR", "M_LR:R"],
        "aml" => &["AM_LR", "AM_LR:L"],
        "amc" => &["AMC"],
        "amr" => &["AM_LR", "AM_LR:R"],
        "st" | "sc" | "stc" => &["ST"],
        _ => &[],
    }
}

fn norm_header(h: &str) -> String {
    let lower = h.trim().to_lowercase();
    let replaced: String = lower
        .chars()
        .map(|c| if c == '_' || c == '-' { ' ' } else { c })
        .collect();
    let collapsed = replaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let stripped = collapsed.trim_end_matches(['.', ':']);

    const PREFIXES: &[&str] = &[
        "gk attributes ",
        "goalkeeping ",
        "goalkeeper ",
        "attributes ",
        "attribute ",
        "technical ",
        "mental ",
        "physical ",
    ];
    for pref in PREFIXES {
        if let Some(rest) = stripped.strip_prefix(pref) {
            return rest.to_string();
        }
    }
    stripped.to_string()
}

fn match_pos_rating(n: &str) -> Option<&'static str> {
    const PREFIXES: &[&str] = &[
        "position rating ",
        "proficiencies ",
        "proficiency ",
        "positions ",
        "position ",
        "pos ",
    ];
    let rest = PREFIXES.iter().find_map(|p| n.strip_prefix(p))?;
    match rest {
        "gk" => Some("gk"),
        "dl" => Some("dl"),
        "dc" => Some("dc"),
        "dr" => Some("dr"),
        "wbl" => Some("wbl"),
        "wbr" => Some("wbr"),
        "dm" => Some("dm"),
        "dmc" => Some("dmc"),
        "ml" => Some("ml"),
        "mc" => Some("mc"),
        "mr" => Some("mr"),
        "aml" => Some("aml"),
        "amc" => Some("amc"),
        "amr" => Some("amr"),
        "stc" => Some("stc"),
        "st" => Some("st"),
        "sc" => Some("sc"),
        _ => None,
    }
}

fn header_to_key(h: &str) -> Option<HeaderKey> {
    let n = norm_header(h);
    if n.is_empty() {
        return None;
    }
    if NAME_ALIASES.contains(&n.as_str()) {
        return Some(HeaderKey::Name);
    }
    if POS_ALIASES.contains(&n.as_str()) {
        return Some(HeaderKey::Position);
    }
    if AGE_ALIASES.contains(&n.as_str()) {
        return Some(HeaderKey::Age);
    }
    if let Some(meta) = meta_alias(&n) {
        return Some(HeaderKey::Meta(meta));
    }
    if n.ends_with(" code") {
        return None;
    }
    if let Some(pr) = match_pos_rating(&n) {
        return Some(HeaderKey::PosRating(pr));
    }
    for meta in ATTR_META.iter() {
        if meta.aliases.contains(&n.as_str()) || norm_header(meta.pt) == n {
            return Some(HeaderKey::Attr(meta.attr));
        }
    }
    None
}

/// Parse an FM attribute/numeric cell supporting European decimals (`13,5`) and ranges (`14-16` -> `15.0`).
pub fn parse_attr_value(v: &str) -> Option<f32> {
    let s = v.trim();
    if s.is_empty() || s == "-" || s == "–" {
        return None;
    }
    // Check range: 1-2 digits, optional spaces, '-' or '–', optional spaces, 1-2 digits
    if let Some((left, right)) = split_range(s) {
        if (1..=2).contains(&left.len())
            && (1..=2).contains(&right.len())
            && left.bytes().all(|b| b.is_ascii_digit())
            && right.bytes().all(|b| b.is_ascii_digit())
        {
            if let (Ok(a), Ok(b)) = (left.parse::<f32>(), right.parse::<f32>()) {
                return Some((a + b) / 2.0);
            }
        }
    }
    let normalized = s.replace(',', ".");
    parse_leading_f32(&normalized)
}

fn split_range(s: &str) -> Option<(&str, &str)> {
    for sep in ['-', '–'] {
        if let Some(idx) = s.find(sep) {
            let left = s[..idx].trim();
            let right = s[idx + sep.len_utf8()..].trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, right));
            }
        }
    }
    None
}

/// Mimic JS `parseFloat` (parses leading finite float prefix, e.g. `"180 cm"` -> `Some(180.0)`).
pub fn parse_leading_f32(s: &str) -> Option<f32> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }
    let mut has_digits = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        has_digits = true;
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            has_digits = true;
            i += 1;
        }
    }
    if !has_digits {
        return None;
    }
    // Optional exponent
    let exp_start = i;
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        let mut exp_digits = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp_digits = true;
            i += 1;
        }
        if !exp_digits {
            i = exp_start;
        }
    }
    trimmed[..i]
        .parse::<f32>()
        .ok()
        .filter(|n| n.is_finite())
}

fn push_unique(out: &mut Vec<String>, seen: &mut HashSet<String>, token: &str) {
    if seen.insert(token.to_string()) {
        out.push(token.to_string());
    }
}

/// Parse FM position strings such as `"D (RL), WB (R)"`, `"AM (RLC), ST (C)"`, `"M/AM (R)"`, `"GK"`.
pub fn parse_positions(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    let s = input
        .to_uppercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Split by top-level commas (outside parentheses)
    let mut parts = Vec::new();
    let mut depth = 0_i32;
    let mut cur = String::new();
    for ch in s.chars() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        }
        if ch == ',' && depth == 0 {
            parts.push(cur);
            cur = String::new();
        } else {
            cur.push(ch);
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }

    for part in parts {
        let code = part.trim();
        let short: Option<&[&str]> = match code {
            "GK" => Some(&["GK"]),
            "DC" | "DCL" | "DCR" => Some(&["DC"]),
            "DL" => Some(&["D_LR", "D_LR:L"]),
            "DR" => Some(&["D_LR", "D_LR:R"]),
            "WBL" => Some(&["WB_LR", "WB_LR:L"]),
            "WBR" => Some(&["WB_LR", "WB_LR:R"]),
            "DM" => Some(&["DM"]),
            "MC" => Some(&["MC"]),
            "ML" => Some(&["M_LR", "M_LR:L"]),
            "MR" => Some(&["M_LR", "M_LR:R"]),
            "AMC" => Some(&["AMC"]),
            "AML" => Some(&["AM_LR", "AM_LR:L"]),
            "AMR" => Some(&["AM_LR", "AM_LR:R"]),
            "ST" | "SC" => Some(&["ST"]),
            _ => None,
        };
        if let Some(tokens) = short {
            for &tok in tokens {
                push_unique(&mut out, &mut seen, tok);
            }
            continue;
        }

        // Match ^([A-Z/ ]+?)\s*(?:\(([RLC]+)\))?$
        let (pref_str, sides_str) = if let Some(stripped) = code.strip_suffix(')') {
            let Some(open) = stripped.rfind('(') else {
                continue;
            };
            let sides = &stripped[open + 1..];
            if sides.is_empty() || !sides.chars().all(|c| matches!(c, 'R' | 'L' | 'C')) {
                continue;
            }
            (stripped[..open].trim(), sides)
        } else {
            (code, "")
        };

        if pref_str.is_empty()
            || !pref_str
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == '/' || c == ' ')
        {
            continue;
        }

        let sides: Vec<char> = sides_str.chars().collect();
        for p in pref_str.split('/').map(str::trim).filter(|x| !x.is_empty()) {
            match p {
                "GK" => push_unique(&mut out, &mut seen, "GK"),
                "DM" => push_unique(&mut out, &mut seen, "DM"),
                "ST" | "S" => push_unique(&mut out, &mut seen, "ST"),
                "D" => {
                    if sides.is_empty() || sides.contains(&'C') {
                        push_unique(&mut out, &mut seen, "DC");
                    }
                    for &side in &sides {
                        if side != 'C' {
                            push_unique(&mut out, &mut seen, "D_LR");
                            push_unique(&mut out, &mut seen, &format!("D_LR:{side}"));
                        }
                    }
                }
                "WB" => {
                    let eff: &[char] = if sides.is_empty() {
                        &['L', 'R']
                    } else {
                        &sides
                    };
                    for &side in eff {
                        if side != 'C' {
                            push_unique(&mut out, &mut seen, "WB_LR");
                            push_unique(&mut out, &mut seen, &format!("WB_LR:{side}"));
                        }
                    }
                }
                "M" => {
                    if sides.is_empty() || sides.contains(&'C') {
                        push_unique(&mut out, &mut seen, "MC");
                    }
                    for &side in &sides {
                        if side != 'C' {
                            push_unique(&mut out, &mut seen, "M_LR");
                            push_unique(&mut out, &mut seen, &format!("M_LR:{side}"));
                        }
                    }
                }
                "AM" => {
                    if sides.is_empty() || sides.contains(&'C') {
                        push_unique(&mut out, &mut seen, "AMC");
                    }
                    for &side in &sides {
                        if side != 'C' {
                            push_unique(&mut out, &mut seen, "AM_LR");
                            push_unique(&mut out, &mut seen, &format!("AM_LR:{side}"));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    out
}

fn split_csv(line: &str, delim: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                cur.push('"');
                i += 1;
            } else {
                in_quotes = !in_quotes;
            }
        } else if c == delim && !in_quotes {
            out.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(c);
        }
        i += 1;
    }
    out.push(cur.trim().to_string());
    out
}

fn decode_html_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some(semi) = s[i..].find(';') {
                let entity = &s[i + 1..i + semi];
                let decoded = match entity {
                    "amp" => Some("&".to_string()),
                    "lt" => Some("<".to_string()),
                    "gt" => Some(">".to_string()),
                    "quot" => Some("\"".to_string()),
                    "apos" | "#39" => Some("'".to_string()),
                    "nbsp" => Some(" ".to_string()),
                    "ndash" => Some("–".to_string()),
                    "mdash" => Some("—".to_string()),
                    _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                        u32::from_str_radix(&entity[2..], 16)
                            .ok()
                            .and_then(char::from_u32)
                            .map(|c| c.to_string())
                    }
                    _ if entity.starts_with('#') => entity[1..]
                        .parse::<u32>()
                        .ok()
                        .and_then(char::from_u32)
                        .map(|c| c.to_string()),
                    _ => None,
                };
                if let Some(rep) = decoded {
                    out.push_str(&rep);
                    i += semi + 1;
                    continue;
                }
            }
        }
        let ch = s[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    decode_html_entities(&out).trim().to_string()
}

fn find_tag_ranges(html: &str, tag: &str) -> Vec<(usize, usize)> {
    let lower = html.to_ascii_lowercase();
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let mut ranges = Vec::new();
    let mut pos = 0;

    while let Some(rel) = lower[pos..].find(&open_pat) {
        let start = pos + rel;
        let after_name = start + open_pat.len();
        if let Some(next_byte) = lower.as_bytes().get(after_name) {
            if next_byte.is_ascii_alphanumeric() {
                pos = after_name;
                continue;
            }
        }
        let Some(gt_rel) = lower[after_name..].find('>') else {
            break;
        };
        let content_start = after_name + gt_rel + 1;
        if let Some(close_rel) = lower[content_start..].find(&close_pat) {
            let content_end = content_start + close_rel;
            ranges.push((content_start, content_end));
            pos = content_end + close_pat.len();
        } else {
            ranges.push((content_start, html.len()));
            break;
        }
    }
    ranges
}

fn parse_html_table_rows(table_inner: &str) -> Vec<Vec<String>> {
    let lower = table_inner.to_ascii_lowercase();
    let mut rows = Vec::new();
    for (tr_start, tr_end) in find_tag_ranges(table_inner, "tr") {
        let tr = &table_inner[tr_start..tr_end];
        let tr_low = &lower[tr_start..tr_end];
        let mut cells = Vec::new();
        let mut p = 0;
        while p < tr_low.len() {
            let next_td = tr_low[p..].find("<td");
            let next_th = tr_low[p..].find("<th");
            let (rel, tag) = match (next_td, next_th) {
                (Some(a), Some(b)) => {
                    if a <= b {
                        (a, "td")
                    } else {
                        (b, "th")
                    }
                }
                (Some(a), None) => (a, "td"),
                (None, Some(b)) => (b, "th"),
                (None, None) => break,
            };
            let start = p + rel;
            let after = start + 3;
            if let Some(nb) = tr_low.as_bytes().get(after) {
                if nb.is_ascii_alphanumeric() {
                    p = after;
                    continue;
                }
            }
            let Some(gt_rel) = tr_low[after..].find('>') else {
                break;
            };
            let c_start = after + gt_rel + 1;
            let close_pat = format!("</{tag}>");
            if let Some(c_rel) = tr_low[c_start..].find(&close_pat) {
                let c_end = c_start + c_rel;
                cells.push(strip_html_tags(&tr[c_start..c_end]));
                p = c_end + close_pat.len();
            } else {
                cells.push(strip_html_tags(&tr[c_start..]));
                break;
            }
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    rows
}

fn table_from_text(text: &str) -> Vec<Vec<String>> {
    let cleaned = text.replace('\r', "");
    let t = cleaned.trim();
    if t.is_empty() {
        return Vec::new();
    }

    if t.to_ascii_lowercase().contains("<table") {
        let mut best_rows: Vec<Vec<String>> = Vec::new();
        for (start, end) in find_tag_ranges(t, "table") {
            let rows = parse_html_table_rows(&t[start..end]);
            if rows.len() > best_rows.len() {
                best_rows = rows;
            }
        }
        return best_rows;
    }

    let lines: Vec<&str> = t
        .split('\n')
        .filter(|l| {
            let tr = l.trim();
            !tr.is_empty()
                && !tr
                    .chars()
                    .all(|c| c.is_whitespace() || matches!(c, '|' | '-' | '+' | '=' | '_'))
        })
        .collect();
    if lines.is_empty() {
        return Vec::new();
    }

    if lines
        .iter()
        .any(|l| l.bytes().filter(|&b| b == b'|').count() >= 3)
    {
        return lines
            .into_iter()
            .map(|l| {
                let mut r: Vec<String> = l.split('|').map(|x| x.trim().to_string()).collect();
                if r.first().is_some_and(String::is_empty) {
                    r.remove(0);
                }
                if r.last().is_some_and(String::is_empty) {
                    r.pop();
                }
                r
            })
            .collect();
    }

    let head = lines[0];
    let mut delims = ['\t', ';', ','];
    delims.sort_by_key(|&d| std::cmp::Reverse(head.split(d).count()));
    let delim = delims[0];
    lines.into_iter().map(|l| split_csv(l, delim)).collect()
}

fn set_meta(meta: &mut PlayerMeta, key: &str, val: String) {
    match key {
        "bestPos" => meta.best_pos = Some(val),
        "bestRole" => meta.best_role = Some(val),
        "ability" => meta.ability = Some(val),
        "potential" => meta.potential = Some(val),
        "ca" => meta.ca = Some(val),
        "pa" => meta.pa = Some(val),
        "uid" => meta.uid = Some(val),
        "personality" => meta.personality = Some(val),
        "club" => meta.club = Some(val),
        "foot" => meta.foot = Some(val),
        "height" => meta.height = Some(val),
        _ => {}
    }
}

/// Parse an FM squad table (CSV, TSV, semicolon CSV, FM pipe table, or HTML `<table>`).
pub fn parse_squad(text: &str, lang: Lang) -> ParsedSquad {
    let rows = table_from_text(text);
    if rows.len() < 2 {
        return ParsedSquad {
            players: Vec::new(),
            warnings: vec![lang
                .pick(
                    "Não encontrei uma tabela com cabeçalho e linhas.",
                    "No table with a header and rows was found.",
                )
                .to_string()],
            mapped: Vec::new(),
        };
    }

    let mut hi = 0_usize;
    let mut best = 0_usize;
    let mut found_any = false;
    for (i, r) in rows.iter().take(6).enumerate() {
        let count = r.iter().filter_map(|h| header_to_key(h)).count();
        if !found_any || count > best {
            found_any = true;
            best = count;
            hi = i;
        }
    }

    let header: Vec<Option<HeaderKey>> = rows[hi].iter().map(|h| header_to_key(h)).collect();
    let mapped: Vec<MappedColumn> = rows[hi]
        .iter()
        .zip(header.iter())
        .map(|(h, k)| MappedColumn {
            h: h.clone(),
            k: k.as_ref().map(HeaderKey::as_mapped_string),
        })
        .collect();

    let mut warnings = Vec::new();
    if !header.iter().any(|k| matches!(k, Some(HeaderKey::Name))) {
        warnings.push(
            lang.pick(
                "Sem coluna de nome reconhecida (\"Name\"/\"Nome\"). Uso a primeira coluna.",
                "No recognised name column (\"Name\"/\"Nome\"). Using the first column.",
            )
            .to_string(),
        );
    }

    let attr_cols = header
        .iter()
        .filter(|k| matches!(k, Some(HeaderKey::Attr(_))))
        .count();
    if attr_cols < 10 {
        warnings.push(match lang {
            Lang::Pt => format!(
                "Só reconheci {attr_cols} atributos. Para análise completa exporte uma vista com todos os atributos."
            ),
            Lang::En => format!(
                "Only {attr_cols} attributes recognised. For a full analysis export a view with every attribute."
            ),
        });
    }

    let mut players = Vec::new();
    for (idx, r) in rows[hi + 1..].iter().enumerate() {
        let mut p = Player {
            id: format!("p{idx}"),
            ..Player::default()
        };
        let mut pos_r_order: Vec<(&'static str, f32)> = Vec::new();

        for (i, v) in r.iter().enumerate() {
            let Some(Some(key)) = header.get(i) else {
                continue;
            };
            match key {
                HeaderKey::Meta(mk) => {
                    if !v.is_empty() {
                        set_meta(&mut p.meta, mk, v.clone());
                    }
                }
                HeaderKey::PosRating(pk) => {
                    if let Some(n) = parse_attr_value(v) {
                        p.pos_r.insert((*pk).to_string(), n);
                        if let Some(entry) = pos_r_order.iter_mut().find(|(k, _)| k == pk) {
                            entry.1 = n;
                        } else {
                            pos_r_order.push((pk, n));
                        }
                    }
                }
                HeaderKey::Name => p.name = v.clone(),
                HeaderKey::Position => {
                    p.pos_raw = v.clone();
                    p.pos = parse_positions(v);
                }
                HeaderKey::Age => p.age = parse_attr_value(v),
                HeaderKey::Attr(attr) => {
                    if let Some(n) = parse_attr_value(v) {
                        p.a.insert(*attr, n);
                    }
                }
            }
        }

        if p.name.is_empty() {
            if let Some(first) = r.first() {
                p.name = first.clone();
            }
        }
        if let Some(ref h_str) = p.meta.height {
            p.height = parse_leading_f32(&h_str.replace(',', "."));
        }

        if p.pos.is_empty() && !pos_r_order.is_empty() {
            let top = pos_r_order
                .iter()
                .map(|(_, val)| *val)
                .fold(f32::NEG_INFINITY, f32::max);
            let good: Vec<&'static str> = pos_r_order
                .iter()
                .filter(|&&(_, val)| val >= 15.0 || (top < 15.0 && (val - top).abs() < f32::EPSILON))
                .map(|&(k, _)| k)
                .collect();
            let mut pos_seen = HashSet::new();
            for &k in &good {
                for &tok in pos_rating_tokens(k) {
                    push_unique(&mut p.pos, &mut pos_seen, tok);
                }
            }
            p.pos_raw = good
                .iter()
                .map(|k| k.to_uppercase())
                .collect::<Vec<_>>()
                .join(", ");
        }

        if p.pos.is_empty() {
            if let Some(bp) = p.meta.best_pos.clone() {
                p.pos_raw = bp.clone();
                p.pos = parse_positions(&bp);
            }
        }

        if let Some(ref uid) = p.meta.uid {
            p.id = format!("u{uid}");
        }

        if !p.name.is_empty() && p.a.len() >= 3 {
            players.push(p);
        }
    }

    let mut seen: HashMap<String, usize> = HashMap::new();
    for p in &mut players {
        if p.meta.uid.is_none() {
            let base = format!("n:{}", p.name.trim().to_lowercase());
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            p.id = if *count > 1 {
                format!("{base}#{count}")
            } else {
                base
            };
        }
    }

    if players.is_empty() {
        warnings.push(
            lang.pick(
                "Nenhum jogador com atributos foi lido.",
                "No players with attributes were read.",
            )
            .to_string(),
        );
    }

    ParsedSquad {
        players,
        warnings,
        mapped,
    }
}

/// Majority club name and share (0..=1) across an exported player list.
pub fn majority_club(players: &[Player]) -> MajorityClub {
    if players.is_empty() {
        return MajorityClub {
            club: None,
            share: 0.0,
        };
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for p in players {
        let club = p
            .meta
            .club
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| p.club.as_deref().filter(|s| !s.is_empty()));
        if let Some(c) = club {
            *counts.entry(c).or_insert(0) += 1;
        }
    }
    let best = counts
        .into_iter()
        .max_by(|(c1, n1), (c2, n2)| n1.cmp(n2).then_with(|| c2.cmp(c1)));
    match best {
        Some((club, n)) => MajorityClub {
            club: Some(club.to_string()),
            share: n as f32 / players.len() as f32,
        },
        None => MajorityClub {
            club: None,
            share: 0.0,
        },
    }
}

fn is_name_suffix(word: &str) -> bool {
    let lower = word.to_lowercase();
    matches!(
        lower.as_str(),
        "junior" | "júnior" | "jr" | "jr." | "filho" | "neto" | "sobrinho" | "ii" | "iii"
    )
}

/// Pitch-friendly surname preserving nobility/preposition particles (`De Bruyn`, `Van der Berg`, `Silva Júnior`).
pub fn surname(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.is_empty() {
        return "—".to_string();
    }
    if words.len() == 1 {
        return words[0].to_string();
    }
    let mut i = words.len() - 1;
    if i > 1 && is_name_suffix(words[i]) {
        i -= 1;
    }
    while i > 0 && NAME_PARTICLES.contains(&words[i - 1].to_lowercase().as_str()) {
        i -= 1;
    }
    words[i..].join(" ")
}

// ============================================================================
// FM27 .fmf binary container & inspection
// ============================================================================

fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let bytes: [u8; 4] = buf.get(off..off + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shannon_entropy(buf: &[u8]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    let mut counts = [0_u64; 256];
    for &b in buf {
        counts[b as usize] += 1;
    }
    let len = buf.len() as f64;
    let mut e = 0.0_f64;
    for &c in &counts {
        if c > 0 {
            let p = (c as f64) / len;
            e -= p * p.log2();
        }
    }
    e as f32
}

fn inflate_from(buf: &[u8], offset: usize, fmt: &str) -> Option<Vec<u8>> {
    const MAX_OUT: u64 = 64 * 1024 * 1024;
    let slice = buf.get(offset..)?;
    let mut out = Vec::new();
    let res = match fmt {
        "deflate" => ZlibDecoder::new(slice).take(MAX_OUT).read_to_end(&mut out),
        "gzip" => GzDecoder::new(slice).take(MAX_OUT).read_to_end(&mut out),
        _ => return None,
    };
    // Tolerate trailing bytes after the compressed stream (matching browser DecompressionStream).
    if res.is_ok() || !out.is_empty() {
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    } else {
        None
    }
}

fn extract_strings(buf: &[u8], min: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut add = |s: &str| {
        let trimmed = s.trim();
        if trimmed.chars().count() >= min && seen.insert(trimmed.to_string()) {
            out.push(trimmed.to_string());
        }
    };

    // ASCII
    let mut cur = String::new();
    for &b in buf {
        if (32..127).contains(&b) {
            cur.push(b as char);
        } else {
            if cur.len() >= min {
                add(&cur);
            }
            cur.clear();
        }
    }
    if cur.len() >= min {
        add(&cur);
    }

    // UTF-16LE ASCII subset
    cur.clear();
    let mut i = 0;
    while i + 1 < buf.len() {
        let b = buf[i];
        let z = buf[i + 1];
        if z == 0 && (32..127).contains(&b) {
            cur.push(b as char);
        } else {
            if cur.len() >= min {
                add(&cur);
            }
            cur.clear();
        }
        i += 2;
    }
    if cur.len() >= min {
        add(&cur);
    }

    // UTF-8 with accents: [\p{L}\p{N} '-]{5,}
    let lossy = String::from_utf8_lossy(buf);
    let mut ucur = String::new();
    let mut ulen = 0_usize;
    for ch in lossy.chars() {
        if ch.is_alphabetic() || ch.is_numeric() || matches!(ch, ' ' | '\'' | '-') {
            ucur.push(ch);
            ulen += 1;
        } else {
            if ulen >= 5 {
                add(&ucur);
            }
            ucur.clear();
            ulen = 0;
        }
    }
    if ulen >= 5 {
        add(&ucur);
    }

    out
}

/// Parse the FM27 `.fmf` container layout (`[02 01][4B tag][08 00 00]`, encrypted blocks, zstd index).
pub fn parse_fmf_container(buf: &[u8]) -> FmfContainer {
    let mut sections = Vec::new();
    let mut p = 0_usize;

    while p + 9 <= buf.len() && buf[p] == 0x02 && buf[p + 1] == 0x01 {
        let tag: String = buf[p + 2..p + 6].iter().map(|&b| b as char).collect();
        let name: String = tag.chars().rev().collect();
        let start = p + 9;
        if buf.get(start..start + 4) == Some(&[0x28, 0xb5, 0x2f, 0xfd]) {
            sections.push(FmfSection {
                tag,
                name,
                off: p,
                start,
                len: buf.len() - start,
                kind: "zstd".to_string(),
            });
            break;
        }
        let len = read_u32_le(buf, start).unwrap_or(0) as usize;
        if len == 0 || start.saturating_add(len) > buf.len() {
            break;
        }
        sections.push(FmfSection {
            tag,
            name,
            off: p,
            start,
            len,
            kind: "blob".to_string(),
        });
        p = start + len;
    }

    let mut blocks = Vec::new();
    for sec in sections.iter().filter(|s| s.kind == "blob") {
        let mut marks = Vec::new();
        let mut i = sec.start + 4;
        let sec_end = sec.start + sec.len;
        while i + 8 <= sec_end {
            if read_u32_le(buf, i) == Some(16) && read_u32_le(buf, i + 4) == Some(16) {
                marks.push(i);
                i += 8;
            } else {
                i += 1;
            }
        }
        for (k, &m) in marks.iter().enumerate() {
            let end = if k + 1 < marks.len() {
                marks[k + 1]
            } else {
                sec_end
            };
            let size = end.saturating_sub(m);
            blocks.push(FmfBlock {
                off: m,
                size,
                cipher: size.saturating_sub(40),
            });
        }
    }

    let recognized =
        sections.len() >= 2 && sections.iter().any(|s| s.kind == "zstd") && !blocks.is_empty();
    let largest = blocks.iter().copied().max_by_key(|b| b.size);

    FmfContainer {
        recognized,
        sections,
        blocks,
        largest,
    }
}

/// Inspect an `.fmf` file: entropy, embedded zlib/gzip/zstd/lz4 streams, strings, tactical matches, container blocks.
pub fn inspect_fmf(name: &str, buf: &[u8]) -> FmfInspectReport {
    let head_slice = &buf[..buf.len().min(32)];
    let head = hex_bytes(head_slice);
    let head_ascii: String = head_slice
        .iter()
        .map(|&b| if (32..127).contains(&b) { b as char } else { '.' })
        .collect();
    let entropy = (shannon_entropy(buf) * 100.0).round() / 100.0;

    let mut sigs: Vec<(usize, &'static str)> = Vec::new();
    let mut magic = Vec::new();
    if buf.len() > 4 {
        for i in 0..buf.len() - 4 {
            let a = buf[i];
            let b = buf[i + 1];
            if a == 0x78
                && matches!(b, 0x01 | 0x5e | 0x9c | 0xda)
                && (((a as u16) << 8) | (b as u16)).is_multiple_of(31)
            {
                sigs.push((i, "deflate"));
            } else if a == 0x1f && b == 0x8b && buf[i + 2] == 0x08 {
                sigs.push((i, "gzip"));
            } else if a == 0x28 && b == 0xb5 && buf[i + 2] == 0x2f && buf[i + 3] == 0xfd {
                magic.push(format!("zstd @{i}"));
            } else if a == 0x04 && b == 0x22 && buf[i + 2] == 0x4d && buf[i + 3] == 0x18 {
                magic.push(format!("lz4 @{i}"));
            }
            if sigs.len() > 40 {
                break;
            }
        }
    }

    let mut streams = Vec::new();
    let mut blobs: Vec<Vec<u8>> = Vec::new();
    for &(off, fmt) in sigs.iter().take(40) {
        if let Some(out) = inflate_from(buf, off, fmt) {
            if out.len() > 32 {
                streams.push(FmfStream {
                    off,
                    fmt,
                    len: out.len(),
                });
                blobs.push(out);
            }
        }
    }

    let mut strings = Vec::new();
    let mut str_seen = HashSet::new();
    for s in extract_strings(buf, 4) {
        if str_seen.insert(s.clone()) {
            strings.push(s);
        }
    }
    for blob in &blobs {
        for s in extract_strings(blob, 4) {
            if str_seen.insert(s.clone()) {
                strings.push(s);
            }
        }
    }

    let low: Vec<String> = strings.iter().map(|s| s.to_lowercase()).collect();
    let has = |term: &str| {
        let needle = term.to_lowercase();
        low.iter().any(|s| s.contains(&needle))
    };

    let mut role_seen = HashSet::new();
    let roles: Vec<String> = ROLES
        .iter()
        .filter_map(|r| {
            if role_seen.insert(r.name) && has(r.name) {
                Some(r.name.to_string())
            } else {
                None
            }
        })
        .collect();

    let mut form_seen = HashSet::new();
    let formations: Vec<String> = FORMATIONS
        .iter()
        .filter_map(|f| {
            let short = f.name.split(' ').next().unwrap_or(f.name);
            if form_seen.insert(short) && has(short) {
                Some(short.to_string())
            } else {
                None
            }
        })
        .collect();

    let instructions: Vec<String> = TEAM_INSTRUCTIONS
        .iter()
        .filter(|ti| has(ti.name))
        .map(|ti| ti.name.to_string())
        .collect();

    let mentality: Vec<String> = Mentality::ALL
        .iter()
        .filter(|m| has(m.as_str()))
        .map(|m| m.as_str().to_string())
        .collect();

    let decoded = !roles.is_empty() || !instructions.is_empty();
    let matches = FmfMatches {
        roles,
        formations,
        instructions,
        mentality,
    };
    let container = parse_fmf_container(buf);

    FmfInspectReport {
        name: name.to_string(),
        size: buf.len(),
        head,
        head_ascii,
        entropy,
        streams,
        magic,
        strings,
        matches,
        decoded,
        container,
    }
}
