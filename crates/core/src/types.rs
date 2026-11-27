//! Shared domain, analysis, simulation, scouting, bridge, and persistence types
//! for `ta-core`.
//!
//! Designed to be:
//! - Zero-UI, pure `std` + `serde` + `serde_json` + `thiserror`.
//! - Serde-compatible with legacy `fm27tb.v1` localStorage and `.fm27tb.json` exports.
//! - Compact for 100k+ scouting databases (`ScoutPlayer` uses fixed arrays & bitmasks).

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

// ============================================================================
// Core Enums
// ============================================================================

/// Active UI / report language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Pt,
}

impl Lang {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Pt => "pt",
        }
    }

    pub const fn locale(self) -> &'static str {
        match self {
            Self::En => "en-GB",
            Self::Pt => "pt-PT",
        }
    }

    pub const fn toggle(self) -> Self {
        match self {
            Self::Pt => Self::En,
            Self::En => Self::Pt,
        }
    }

    #[inline]
    pub const fn pick<'a>(self, pt: &'a str, en: &'a str) -> &'a str {
        match self {
            Self::Pt => pt,
            Self::En => en,
        }
    }
}

/// Attribute group in FM (Technical, Mental, Physical, Goalkeeping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttrCat {
    #[serde(rename = "T")]
    Technical,
    #[serde(rename = "M")]
    Mental,
    #[serde(rename = "F")]
    Physical,
    #[serde(rename = "G")]
    Goalkeeping,
}

/// All 47 visible FM player attributes (14 technical, 14 mental, 8 physical, 11 GK).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Attr {
    // Technical (0..14)
    Cor,
    Cro,
    Dri,
    Fin,
    Fir,
    Fre,
    Hea,
    Lon,
    Lth,
    Mar,
    Pas,
    Pen,
    Tck,
    Tec,
    // Mental (14..28)
    Agg,
    Ant,
    Bra,
    Cmp,
    Cnt,
    Dec,
    Det,
    Fla,
    Ldr,
    Otb,
    Pos,
    Tea,
    Vis,
    Wor,
    // Physical (28..36)
    Acc,
    Agi,
    Bal,
    Jum,
    Nat,
    Pac,
    Sta,
    Str,
    // Goalkeeping (36..47)
    Aer,
    Cmd,
    Com,
    Ecc,
    Han,
    Kic,
    Ovo,
    Pun,
    Ref,
    Tro,
    Thr,
}

impl Attr {
    pub const COUNT: usize = 47;

    pub const ALL: [Attr; Self::COUNT] = [
        // Technical
        Self::Cor,
        Self::Cro,
        Self::Dri,
        Self::Fin,
        Self::Fir,
        Self::Fre,
        Self::Hea,
        Self::Lon,
        Self::Lth,
        Self::Mar,
        Self::Pas,
        Self::Pen,
        Self::Tck,
        Self::Tec,
        // Mental
        Self::Agg,
        Self::Ant,
        Self::Bra,
        Self::Cmp,
        Self::Cnt,
        Self::Dec,
        Self::Det,
        Self::Fla,
        Self::Ldr,
        Self::Otb,
        Self::Pos,
        Self::Tea,
        Self::Vis,
        Self::Wor,
        // Physical
        Self::Acc,
        Self::Agi,
        Self::Bal,
        Self::Jum,
        Self::Nat,
        Self::Pac,
        Self::Sta,
        Self::Str,
        // Goalkeeping
        Self::Aer,
        Self::Cmd,
        Self::Com,
        Self::Ecc,
        Self::Han,
        Self::Kic,
        Self::Ovo,
        Self::Pun,
        Self::Ref,
        Self::Tro,
        Self::Thr,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_index(idx: usize) -> Option<Self> {
        if idx < Self::COUNT {
            Some(Self::ALL[idx])
        } else {
            None
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Cor => "cor",
            Self::Cro => "cro",
            Self::Dri => "dri",
            Self::Fin => "fin",
            Self::Fir => "fir",
            Self::Fre => "fre",
            Self::Hea => "hea",
            Self::Lon => "lon",
            Self::Lth => "lth",
            Self::Mar => "mar",
            Self::Pas => "pas",
            Self::Pen => "pen",
            Self::Tck => "tck",
            Self::Tec => "tec",
            Self::Agg => "agg",
            Self::Ant => "ant",
            Self::Bra => "bra",
            Self::Cmp => "cmp",
            Self::Cnt => "cnt",
            Self::Dec => "dec",
            Self::Det => "det",
            Self::Fla => "fla",
            Self::Ldr => "ldr",
            Self::Otb => "otb",
            Self::Pos => "pos",
            Self::Tea => "tea",
            Self::Vis => "vis",
            Self::Wor => "wor",
            Self::Acc => "acc",
            Self::Agi => "agi",
            Self::Bal => "bal",
            Self::Jum => "jum",
            Self::Nat => "nat",
            Self::Pac => "pac",
            Self::Sta => "sta",
            Self::Str => "str",
            Self::Aer => "aer",
            Self::Cmd => "cmd",
            Self::Com => "com",
            Self::Ecc => "ecc",
            Self::Han => "han",
            Self::Kic => "kic",
            Self::Ovo => "ovo",
            Self::Pun => "pun",
            Self::Ref => "ref",
            Self::Tro => "tro",
            Self::Thr => "thr",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "cor" => Some(Self::Cor),
            "cro" => Some(Self::Cro),
            "dri" => Some(Self::Dri),
            "fin" => Some(Self::Fin),
            "fir" => Some(Self::Fir),
            "fre" => Some(Self::Fre),
            "hea" => Some(Self::Hea),
            "lon" => Some(Self::Lon),
            "lth" => Some(Self::Lth),
            "mar" => Some(Self::Mar),
            "pas" => Some(Self::Pas),
            "pen" => Some(Self::Pen),
            "tck" => Some(Self::Tck),
            "tec" => Some(Self::Tec),
            "agg" => Some(Self::Agg),
            "ant" => Some(Self::Ant),
            "bra" => Some(Self::Bra),
            "cmp" => Some(Self::Cmp),
            "cnt" => Some(Self::Cnt),
            "dec" => Some(Self::Dec),
            "det" => Some(Self::Det),
            "fla" => Some(Self::Fla),
            "ldr" => Some(Self::Ldr),
            "otb" => Some(Self::Otb),
            "pos" => Some(Self::Pos),
            "tea" => Some(Self::Tea),
            "vis" => Some(Self::Vis),
            "wor" => Some(Self::Wor),
            "acc" => Some(Self::Acc),
            "agi" => Some(Self::Agi),
            "bal" => Some(Self::Bal),
            "jum" => Some(Self::Jum),
            "nat" => Some(Self::Nat),
            "pac" => Some(Self::Pac),
            "sta" => Some(Self::Sta),
            "str" => Some(Self::Str),
            "aer" => Some(Self::Aer),
            "cmd" => Some(Self::Cmd),
            "com" => Some(Self::Com),
            "ecc" => Some(Self::Ecc),
            "han" => Some(Self::Han),
            "kic" => Some(Self::Kic),
            "ovo" => Some(Self::Ovo),
            "pun" => Some(Self::Pun),
            "ref" => Some(Self::Ref),
            "tro" => Some(Self::Tro),
            "thr" => Some(Self::Thr),
            _ => None,
        }
    }

    pub const fn category(self) -> AttrCat {
        match self.index() {
            0..=13 => AttrCat::Technical,
            14..=27 => AttrCat::Mental,
            28..=35 => AttrCat::Physical,
            _ => AttrCat::Goalkeeping,
        }
    }
}

/// 10 position groups matching `GROUPS` in `src/core/roles.js`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PosGroup {
    #[serde(rename = "GK")]
    Gk,
    #[serde(rename = "DC")]
    Dc,
    #[serde(rename = "D_LR")]
    DLr,
    #[serde(rename = "WB_LR")]
    WbLr,
    #[serde(rename = "DM")]
    Dm,
    #[serde(rename = "MC")]
    Mc,
    #[serde(rename = "M_LR")]
    MLr,
    #[serde(rename = "AMC")]
    Amc,
    #[serde(rename = "AM_LR")]
    AmLr,
    #[serde(rename = "ST")]
    St,
}

impl PosGroup {
    pub const COUNT: usize = 10;

    pub const ALL: [PosGroup; Self::COUNT] = [
        Self::Gk,
        Self::Dc,
        Self::DLr,
        Self::WbLr,
        Self::Dm,
        Self::Mc,
        Self::MLr,
        Self::Amc,
        Self::AmLr,
        Self::St,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Bit for the base position group (bits 0..10).
    #[inline]
    pub const fn bit(self) -> u32 {
        1u32 << (self as u32)
    }

    /// Bit for a sided flank tag (`D_LR:L` -> bit 10, `D_LR:R` -> bit 11, etc.).
    #[inline]
    pub const fn side_bit(self, side: Side) -> u32 {
        let off = match self {
            Self::DLr => 10,
            Self::WbLr => 12,
            Self::MLr => 14,
            Self::AmLr => 16,
            _ => return 0,
        };
        match side {
            Side::L => 1u32 << off,
            Side::R => 1u32 << (off + 1),
            Side::C => 0,
        }
    }

    pub const fn is_wide_group(self) -> bool {
        matches!(self, Self::DLr | Self::WbLr | Self::MLr | Self::AmLr)
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Gk => "GK",
            Self::Dc => "DC",
            Self::DLr => "D_LR",
            Self::WbLr => "WB_LR",
            Self::Dm => "DM",
            Self::Mc => "MC",
            Self::MLr => "M_LR",
            Self::Amc => "AMC",
            Self::AmLr => "AM_LR",
            Self::St => "ST",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "GK" => Some(Self::Gk),
            "DC" => Some(Self::Dc),
            "D_LR" => Some(Self::DLr),
            "WB_LR" => Some(Self::WbLr),
            "DM" => Some(Self::Dm),
            "MC" => Some(Self::Mc),
            "M_LR" => Some(Self::MLr),
            "AMC" => Some(Self::Amc),
            "AM_LR" => Some(Self::AmLr),
            "ST" => Some(Self::St),
            _ => None,
        }
    }
}

/// Lateral side of a pitch slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    L,
    C,
    R,
}

impl Side {
    pub const fn code(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::C => "C",
            Self::R => "R",
        }
    }
}

/// Preferred foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Foot {
    L,
    R,
    B,
}

impl Foot {
    pub fn parse_label(s: &str) -> Option<Self> {
        let low = s.to_lowercase();
        if low.contains("either") || low.contains("ambos") || low.contains("both") || low == "b" {
            Some(Self::B)
        } else if low.contains("left") || low.contains("esq") || low == "l" {
            Some(Self::L)
        } else if low.contains("right") || low.contains("dir") || low == "r" {
            Some(Self::R)
        } else {
            None
        }
    }

    pub const fn as_side(self) -> Option<Side> {
        match self {
            Self::L => Some(Side::L),
            Self::R => Some(Side::R),
            Self::B => None,
        }
    }

    pub const fn en_label(self) -> &'static str {
        match self {
            Self::L => "Left",
            Self::R => "Right",
            Self::B => "Either",
        }
    }
}

/// Possession phase (`IP` or `OOP`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    #[serde(rename = "IP")]
    Ip,
    #[serde(rename = "OOP")]
    Oop,
}

/// Exact-slot restriction for FM26/27 roles (e.g. `b2b` only in off-centre DM slots `DML`/`DMR`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotConstraint {
    #[serde(rename = "C")]
    Central,
    #[serde(rename = "S")]
    Side,
}

/// Behavioral tags attached to roles (used by audit, synergy, gameplans, sim).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleTag {
    // IP
    W,
    In,
    Rd,
    Box,
    Cr,
    Link,
    Run,
    Drop,
    Air,
    Adv,
    ToCb,
    ToDm,
    // OOP
    Hp,
    Low,
    Out,
    Cw,
    Trk,
}

/// Team mentality (7 ordered levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum Mentality {
    #[serde(rename = "Very Defensive")]
    VeryDefensive,
    #[serde(rename = "Defensive")]
    Defensive,
    #[serde(rename = "Cautious")]
    Cautious,
    #[default]
    #[serde(rename = "Balanced")]
    Balanced,
    #[serde(rename = "Positive")]
    Positive,
    #[serde(rename = "Attacking")]
    Attacking,
    #[serde(rename = "Very Attacking")]
    VeryAttacking,
}

impl Mentality {
    pub const ALL: [Mentality; 7] = [
        Self::VeryDefensive,
        Self::Defensive,
        Self::Cautious,
        Self::Balanced,
        Self::Positive,
        Self::Attacking,
        Self::VeryAttacking,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_index(idx: usize) -> Option<Self> {
        if idx < 7 {
            Some(Self::ALL[idx])
        } else {
            None
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VeryDefensive => "Very Defensive",
            Self::Defensive => "Defensive",
            Self::Cautious => "Cautious",
            Self::Balanced => "Balanced",
            Self::Positive => "Positive",
            Self::Attacking => "Attacking",
            Self::VeryAttacking => "Very Attacking",
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|m| m.as_str() == s)
    }
}

// ============================================================================
// Static Knowledge-Base Types (populated by Module A: `data`)
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct AttrMeta {
    pub attr: Attr,
    pub pt: &'static str,
    pub en: &'static str,
    pub cat: AttrCat,
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct Role {
    pub id: &'static str,
    pub ab: &'static str,
    pub name: &'static str,
    pub phase: Phase,
    pub groups: &'static [PosGroup],
    pub tags: &'static [RoleTag],
    pub key: &'static [Attr],
    pub pref: &'static [Attr],
    pub desc_pt: &'static str,
    pub tip_pt: &'static str,
    pub verified: bool,
    pub wide_cb: bool,
    pub slot: Option<SlotConstraint>,
}

impl Role {
    #[inline]
    pub fn has_tag(&self, tag: RoleTag) -> bool {
        self.tags.contains(&tag)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FormationSlot {
    pub code: &'static str,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Formation {
    pub name: &'static str,
    pub slots: [FormationSlot; 11],
}

/// Resolved metadata for one slot in a specific formation.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotMeta {
    pub code: String,
    pub x: f32,
    pub y: f32,
    pub group: PosGroup,
    pub side: Side,
    pub wide_cb: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TiNeed {
    pub low: &'static [Attr],
    pub high: &'static [Attr],
}

#[derive(Debug, Clone, Copy)]
pub struct TeamInstructionDef {
    pub key: &'static str,
    pub name: &'static str,
    pub phase: Phase,
    pub area_pt: &'static str,
    pub opts: &'static [&'static str],
    pub def: usize,
    pub cx: f32,
    pub fm27: bool,
    pub unconf: bool,
    pub desc_pt: &'static str,
    pub pro_pt: &'static str,
    pub con_pt: &'static str,
    pub need: Option<TiNeed>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerInstructionDef {
    pub name: &'static str,
    pub group_pt: &'static str,
    pub desc_pt: &'static str,
    pub tip_pt: &'static str,
    pub fm27: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TacticalTweakDef {
    pub name: &'static str,
    pub desc_pt: &'static str,
    pub when_pt: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Template {
    pub id: &'static str,
    pub name_pt: &'static str,
    pub ip: &'static str,
    pub oop: &'static str,
    pub ment: Mentality,
    pub style: &'static str,
    pub desc_pt: &'static str,
    /// `(ip_code, ip_role_id, oop_role_id)`
    pub roles: &'static [(&'static str, &'static str, &'static str)],
    /// `(ti_key, option_index)`
    pub ti: &'static [(&'static str, usize)],
}

#[derive(Debug, Clone, Copy)]
pub struct StyleProfile {
    pub id: &'static str,
    pub name_pt: &'static str,
    pub attrs: &'static [Attr],
}

#[derive(Debug, Clone, Copy)]
pub struct TrainingAreaDef {
    pub key: &'static str,
    pub name: &'static str,
    pub session_pt: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct FmSection {
    pub title: &'static str,
    pub keys: &'static [&'static str],
}

// ============================================================================
// Tactic, Player, and Squad Types (Serde-compatible with `fm27tb.v1`)
// ============================================================================

/// Flexible deserializer accepting either JSON string or number as `Option<String>`.
fn de_opt_string_or_num<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(match val {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => {
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(serde_json::Value::Bool(b)) => Some(b.to_string()),
        Some(other) => Some(other.to_string()),
    })
}

/// Flexible deserializer accepting JSON number or numeric string as `Option<f32>`.
fn de_opt_f32_or_str<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(match val {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|v| v as f32),
        Some(serde_json::Value::String(s)) => s.trim().replace(',', ".").parse::<f32>().ok(),
        _ => None,
    })
}

/// Flexible deserializer accepting JSON number or numeric string as `Option<f64>`.
fn de_opt_f64_or_str<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(match val {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        Some(serde_json::Value::String(s)) => s.trim().replace(',', ".").parse::<f64>().ok(),
        _ => None,
    })
}

/// One of the 11 tactical slots linking an IP position+role, OOP position+role, player, and PIs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SlotAssignment {
    pub ip_code: String,
    pub oop_code: String,
    pub ip: String,
    pub oop: String,
    pub pid: Option<String>,
    pub pis: Vec<String>,
}

/// Complete tactic definition (compatible with JS `ST.tactic` and `.fm27tb.json`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Tactic {
    pub name: String,
    pub ment: Mentality,
    pub ip_form: String,
    pub oop_form: String,
    pub slots: Vec<SlotAssignment>,
    pub ti: BTreeMap<String, usize>,
    pub ois: usize,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_at: Option<String>,
}

/// Extra metadata columns imported from FM View Exporter / fmsave.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerMeta {
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub best_pos: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub best_role: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub ability: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub potential: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub ca: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub pa: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub uid: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub personality: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub club: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub foot: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub height: Option<String>,
}

/// Squad / opponent player with full attributes, position proficiencies, and optional fmsave fields.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Player {
    pub id: String,
    pub name: String,
    /// Position tokens such as `"DC"`, `"D_LR"`, `"D_LR:L"`, `"ST"`.
    pub pos: Vec<String>,
    pub pos_raw: String,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub age: Option<f32>,
    pub a: BTreeMap<Attr, f32>,
    pub meta: PlayerMeta,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub pos_r: BTreeMap<String, f32>,
    // Optional rich fields from fmsave / live bridge
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub ca: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub pa: Option<f32>,
    pub pa_range: Option<[u16; 2]>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub pers: BTreeMap<String, f32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hid: BTreeMap<String, f32>,
    #[serde(default, deserialize_with = "de_opt_f64_or_str")]
    pub value: Option<f64>,
    #[serde(default, deserialize_with = "de_opt_f64_or_str")]
    pub wage: Option<f64>,
    pub end: Option<String>,
    pub status: Option<String>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub rep: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub wrep: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub club_rep: Option<f32>,
    pub club_nat: Option<i32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub height: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub lf: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_f32_or_str")]
    pub rf: Option<f32>,
    pub foot: Option<Foot>,
    pub loan: Option<bool>,
    pub loan_from: Option<String>,
    pub nat: Option<i32>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub uid: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string_or_num")]
    pub uniq: Option<String>,
    pub club: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nat_pos: Vec<String>,
}

/// Shared read-only view over player attributes/positions/foot so scoring formulas work
/// identically on `Player` and compact `ScoutPlayer` with zero allocations.
pub trait AttrLookup {
    fn attr(&self, attr: Attr) -> Option<f32>;
    fn has_pos_group(&self, group: PosGroup) -> bool;
    fn has_pos_side(&self, group: PosGroup, side: Side) -> bool;
    fn has_any_pos(&self) -> bool;
    fn is_only_gk(&self) -> bool;
    fn foot_side(&self) -> Option<Foot>;
    fn height_cm(&self) -> Option<f32>;
    fn age_years(&self) -> Option<f32>;
}

impl AttrLookup for Player {
    #[inline]
    fn attr(&self, attr: Attr) -> Option<f32> {
        self.a.get(&attr).copied()
    }

    #[inline]
    fn has_pos_group(&self, group: PosGroup) -> bool {
        let code = group.code();
        self.pos.iter().any(|p| p == code)
    }

    #[inline]
    fn has_pos_side(&self, group: PosGroup, side: Side) -> bool {
        match side {
            Side::C => self.has_pos_group(group),
            Side::L | Side::R => {
                let g = group.code();
                let s = side.code();
                self.pos.iter().any(|p| {
                    p.len() == g.len() + 2
                        && p.starts_with(g)
                        && p.as_bytes()[g.len()] == b':'
                        && &p[g.len() + 1..] == s
                })
            }
        }
    }

    #[inline]
    fn has_any_pos(&self) -> bool {
        !self.pos.is_empty()
    }

    #[inline]
    fn is_only_gk(&self) -> bool {
        self.pos.len() == 1 && self.pos[0] == "GK"
    }

    fn foot_side(&self) -> Option<Foot> {
        if let Some(f) = self.foot {
            return Some(f);
        }
        self.meta.foot.as_deref().and_then(Foot::parse_label)
    }

    fn height_cm(&self) -> Option<f32> {
        if self.height.is_some() {
            return self.height;
        }
        self.meta
            .height
            .as_deref()
            .and_then(|s| s.trim().replace(',', ".").parse::<f32>().ok())
    }

    #[inline]
    fn age_years(&self) -> Option<f32> {
        self.age
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MappedColumn {
    pub h: String,
    pub k: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SquadPack {
    pub players: Vec<Player>,
    pub source: String,
    pub warnings: Vec<String>,
    pub mapped: Vec<MappedColumn>,
    pub at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fm_team: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedSquad {
    pub players: Vec<Player>,
    pub warnings: Vec<String>,
    pub mapped: Vec<MappedColumn>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MajorityClub {
    pub club: Option<String>,
    pub share: f32,
}

// ============================================================================
// Engine: Scoring, Audit, Suggestions, Training, Insights, Synergy, FMF
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoleFit {
    pub score: u8,
    pub coverage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleCandidate {
    pub role_id: &'static str,
    pub score: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Crit,
    Warn,
    Info,
    Good,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AuditCat {
    #[serde(rename = "IP")]
    Ip,
    #[serde(rename = "OOP")]
    Oop,
    #[serde(rename = "Coerência")]
    Coherence,
    #[serde(rename = "Complexidade")]
    Complexity,
    #[serde(rename = "Plantel")]
    Squad,
}

impl AuditCat {
    pub const fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Ip, _) => "IP",
            (Self::Oop, _) => "OOP",
            (Self::Coherence, Lang::Pt) => "Coerência",
            (Self::Coherence, Lang::En) => "Coherence",
            (Self::Complexity, Lang::Pt) => "Complexidade",
            (Self::Complexity, Lang::En) => "Complexity",
            (Self::Squad, Lang::Pt) => "Plantel",
            (Self::Squad, Lang::En) => "Squad",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub cat: AuditCat,
    pub sev: Severity,
    pub title: String,
    pub text: String,
    pub fix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub cx: f32,
    pub level: String,
    pub parts: Vec<(String, f32)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
    pub scores: BTreeMap<AuditCat, u8>,
    pub overall: u8,
    pub complexity: ComplexityReport,
    pub cap: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TacticSuggestionSlot {
    pub ip_code: String,
    pub ip_role: String,
    pub player_id: Option<String>,
    pub player_name: Option<String>,
    pub score: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TacticSuggestion {
    pub template_id: &'static str,
    pub tactic: Tactic,
    pub xi_avg: u8,
    pub depth: u8,
    pub fit: u8,
    pub audit: AuditReport,
    pub total: u8,
    pub weakest: Vec<TacticSuggestionSlot>,
    pub strongest: Vec<TacticSuggestionSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingAreaPriority {
    pub key: &'static str,
    pub weight: usize,
    pub why: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingIndiv {
    pub player_id: String,
    pub player_name: String,
    pub slot: String,
    pub role_id: &'static str,
    pub si: u8,
    pub so: u8,
    pub weak: Vec<(Attr, u8)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingPlan {
    pub areas: Vec<TrainingAreaPriority>,
    pub indiv: Vec<TrainingIndiv>,
    pub mentors: Vec<String>,
    pub youth: Vec<String>,
    pub cohesion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiRecommendation {
    pub pi: &'static str,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchCandidate {
    pub player_id: String,
    pub player_name: String,
    pub age: Option<f32>,
    pub score: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotBench {
    pub bench: Option<BenchCandidate>,
    pub succ: Option<BenchCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XiMember {
    pub player_id: String,
    pub player_name: String,
    pub code: String,
    pub group: PosGroup,
    pub side: Side,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpponentSector {
    pub key: &'static str,
    pub name_pt: &'static str,
    pub dir: &'static str,
    pub them_val: Option<f32>,
    pub us_val: Option<f32>,
    pub diff: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpponentPlanItem {
    pub ti: &'static str,
    pub value: usize,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpponentDanger {
    pub player_id: String,
    pub player_name: String,
    pub code: String,
    pub threat: f32,
    pub ois: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpponentReport {
    pub them: Vec<XiMember>,
    pub us: Vec<XiMember>,
    pub sectors: Vec<OpponentSector>,
    pub plan: Vec<OpponentPlanItem>,
    pub strengths: Vec<OpponentSector>,
    pub weaknesses: Vec<OpponentSector>,
    pub dangers: Vec<OpponentDanger>,
    pub club: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CornerTaker {
    pub player_id: String,
    pub player_name: String,
    pub code: String,
    pub swing: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPieceTaker {
    pub player_id: String,
    pub player_name: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPieceRole {
    pub key: String,
    pub label: String,
    pub player_id: String,
    pub player_name: String,
    pub code: String,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPieceTip {
    pub ti: Option<&'static str>,
    pub value: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetPiecePlan {
    pub xi: Vec<XiMember>,
    pub corner_right: CornerTaker,
    pub corner_left: CornerTaker,
    pub fk_direct: Option<SetPieceTaker>,
    pub fk_cross: Option<SetPieceTaker>,
    pub pens: Vec<SetPieceTaker>,
    pub long_throw: Option<SetPieceTaker>,
    pub caps: Vec<SetPieceTaker>,
    pub attack: Vec<SetPieceRole>,
    pub defend: Vec<SetPieceRole>,
    pub system: String,
    pub their_air: Vec<(String, f32)>,
    pub tips: Vec<SetPieceTip>,
    pub air_top: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SynergyPhase {
    Build,
    Att,
    Def,
    Trans,
}

impl SynergyPhase {
    pub const ALL: [SynergyPhase; 4] = [Self::Build, Self::Att, Self::Def, Self::Trans];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SynergyLinkType {
    Build,
    Chain,
    Cross,
    Front,
    Flank,
    Press,
    FlankDef,
    Counter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynergyLink {
    pub link_type: SynergyLinkType,
    pub phase: SynergyPhase,
    pub from_code: String,
    pub to_code: String,
    pub from_label: String,
    pub to_label: String,
    pub value: f32,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyIssue {
    pub key: String,
    pub phase: SynergyPhase,
    pub text: String,
    pub fix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynergyPhaseScore {
    pub score: u8,
    pub parts: BTreeMap<String, u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynergyReport {
    pub score: u8,
    pub phases: BTreeMap<SynergyPhase, SynergyPhaseScore>,
    pub parts: BTreeMap<String, u8>,
    pub att_score: u8,
    pub links: Vec<SynergyLink>,
    pub issues: Vec<SynergyIssue>,
    pub creators: usize,
    pub box_n: usize,
    pub out_n: usize,
    pub rd_n: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FmfSection {
    pub tag: String,
    pub name: String,
    pub off: usize,
    pub start: usize,
    pub len: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FmfBlock {
    pub off: usize,
    pub size: usize,
    pub cipher: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FmfContainer {
    pub recognized: bool,
    pub sections: Vec<FmfSection>,
    pub blocks: Vec<FmfBlock>,
    pub largest: Option<FmfBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FmfStream {
    pub off: usize,
    pub fmt: &'static str,
    pub len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FmfMatches {
    pub roles: Vec<String>,
    pub formations: Vec<String>,
    pub instructions: Vec<String>,
    pub mentality: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FmfInspectReport {
    pub name: String,
    pub size: usize,
    pub head: String,
    pub head_ascii: String,
    pub entropy: f32,
    pub streams: Vec<FmfStream>,
    pub magic: Vec<String>,
    pub strings: Vec<String>,
    pub matches: FmfMatches,
    pub decoded: bool,
    pub container: FmfContainer,
}

// ============================================================================
// Planning: Tactic Generator, Game States, Scouting DB
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Expectation {
    Title,
    Top,
    #[default]
    Mid,
    Low,
}

impl Expectation {
    pub const ALL: [Expectation; 4] = [Self::Title, Self::Top, Self::Mid, Self::Low];

    pub const fn mentality(self) -> Mentality {
        match self {
            Self::Title | Self::Top => Mentality::Positive,
            Self::Mid => Mentality::Balanced,
            Self::Low => Mentality::Cautious,
        }
    }

    pub const fn press_bias(self) -> f32 {
        match self {
            Self::Title => 1.0,
            Self::Top => 0.5,
            Self::Mid => 0.0,
            Self::Low => -1.0,
        }
    }

    pub const fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Title, Lang::Pt) => "Candidato ao título",
            (Self::Title, Lang::En) => "Title contender",
            (Self::Top, Lang::Pt) => "Lugares de cima",
            (Self::Top, Lang::En) => "Top half / Europe",
            (Self::Mid, Lang::Pt) => "Meio da tabela",
            (Self::Mid, Lang::En) => "Mid-table",
            (Self::Low, Lang::Pt) => "Fuga à descida",
            (Self::Low, Lang::En) => "Relegation battle",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneratorConfig {
    pub expect: Expectation,
    pub must: Vec<String>,
    pub youth: Vec<String>,
    pub star: Option<String>,
    pub depth: usize,
    pub use_synergy: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            expect: Expectation::Mid,
            must: Vec::new(),
            youth: Vec::new(),
            star: None,
            depth: 10,
            use_synergy: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenProgress {
    pub done: usize,
    pub total: usize,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedCandidate {
    pub tactic: Tactic,
    pub ip_form: String,
    pub oop_form: String,
    pub fit_avg: u8,
    pub balance: u8,
    pub syn: u8,
    pub total: u8,
    pub score_v: f64,
    pub reasons: BTreeMap<String, String>,
    pub ment_why: String,
    pub audit: AuditReport,
    pub synergy: SynergyReport,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorResult {
    pub main: GeneratedCandidate,
    pub alts: Vec<GeneratedCandidate>,
    pub considered: usize,
    pub optimised: usize,
    pub expect: Expectation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthSlot {
    pub code: String,
    pub list: Vec<BenchCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeakSpot {
    pub code: String,
    pub role_id: &'static str,
    pub player_id: Option<String>,
    pub player_name: Option<String>,
    pub score: u8,
    pub alt: u8,
    pub lack: Vec<Attr>,
    pub urgent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStateKind {
    #[default]
    Chase,
    Protect,
    AwayStrong,
    AwayEqual,
    HomeWeak,
}

impl GameStateKind {
    pub const ALL: [GameStateKind; 5] = [
        Self::Chase,
        Self::Protect,
        Self::AwayStrong,
        Self::AwayEqual,
        Self::HomeWeak,
    ];

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Chase => "⚡",
            Self::Protect => "🛡",
            Self::AwayStrong => "⛰",
            Self::AwayEqual => "⚖",
            Self::HomeWeak => "🏟",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameIntent {
    pub att: u8,
    pub sec: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GamePlanChangeKind {
    Ment,
    Ti,
    Role,
    Pi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamePlanChange {
    pub kind: GamePlanChangeKind,
    pub key: Option<String>,
    pub phase: Option<Phase>,
    pub code: Option<String>,
    pub player: Option<String>,
    pub label: String,
    pub from: String,
    pub to: String,
    pub removed: bool,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GamePlanSub {
    pub out_name: String,
    pub in_name: String,
    pub code: String,
    pub role: String,
    pub why: String,
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamePlanOi {
    pub player: String,
    pub code: String,
    pub ois: Vec<String>,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GamePlan {
    pub kind: GameStateKind,
    pub title: String,
    pub desc: String,
    pub tactic: Tactic,
    pub changes: Vec<GamePlanChange>,
    pub tweaks: Vec<(String, String)>,
    pub subs: Vec<GamePlanSub>,
    pub ois: Vec<GamePlanOi>,
    pub notes: Vec<String>,
    pub before: SynergyReport,
    pub after: SynergyReport,
    pub intent_before: GameIntent,
    pub intent_after: GameIntent,
    pub audit_before: AuditReport,
    pub audit_after: AuditReport,
}

// ---------------- Scouting (100k+ players compact representation) ----------------

/// 8 personality columns in fmsave exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoutPers {
    Ambition = 0,
    Professionalism = 1,
    Pressure = 2,
    Loyalty = 3,
    Temperament = 4,
    Adaptability = 5,
    Sportsmanship = 6,
    Controversy = 7,
}

impl ScoutPers {
    pub const COUNT: usize = 8;
    pub const NAMES: [&'static str; Self::COUNT] = [
        "ambition",
        "professionalism",
        "pressure",
        "loyalty",
        "temperament",
        "adaptability",
        "sportsmanship",
        "controversy",
    ];
}

/// 5 hidden attributes in fmsave exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoutHid {
    Cons = 0,
    Imp = 1,
    Inj = 2,
    Vers = 3,
    Dirt = 4,
}

impl ScoutHid {
    pub const COUNT: usize = 5;
    pub const KEYS: [&'static str; Self::COUNT] = ["cons", "imp", "inj", "vers", "dirt"];
}

/// 14 FM position proficiency columns in fmsave exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoutPosKey {
    Gk = 0,
    Dl = 1,
    Dc = 2,
    Dr = 3,
    Dm = 4,
    Ml = 5,
    Mc = 6,
    Mr = 7,
    Aml = 8,
    Amc = 9,
    Amr = 10,
    Stc = 11,
    Wbl = 12,
    Wbr = 13,
}

impl ScoutPosKey {
    pub const COUNT: usize = 14;
    pub const NAMES: [&'static str; Self::COUNT] = [
        "gk", "dl", "dc", "dr", "dm", "ml", "mc", "mr", "aml", "amc", "amr", "stc", "wbl", "wbr",
    ];

    pub const fn group_and_side(self) -> (PosGroup, Option<Side>) {
        match self {
            Self::Gk => (PosGroup::Gk, None),
            Self::Dl => (PosGroup::DLr, Some(Side::L)),
            Self::Dc => (PosGroup::Dc, None),
            Self::Dr => (PosGroup::DLr, Some(Side::R)),
            Self::Dm => (PosGroup::Dm, None),
            Self::Ml => (PosGroup::MLr, Some(Side::L)),
            Self::Mc => (PosGroup::Mc, None),
            Self::Mr => (PosGroup::MLr, Some(Side::R)),
            Self::Aml => (PosGroup::AmLr, Some(Side::L)),
            Self::Amc => (PosGroup::Amc, None),
            Self::Amr => (PosGroup::AmLr, Some(Side::R)),
            Self::Stc => (PosGroup::St, None),
            Self::Wbl => (PosGroup::WbLr, Some(Side::L)),
            Self::Wbr => (PosGroup::WbLr, Some(Side::R)),
        }
    }
}

/// Compact player record for full-save scouting databases (100k+ rows).
#[derive(Debug, Clone, PartialEq)]
pub struct ScoutPlayer {
    pub id: String,
    pub uid: Option<String>,
    pub uniq: Option<String>,
    pub name: String,
    pub name_lower: String,
    pub age: Option<u16>,
    pub nat: Option<i32>,
    pub height: Option<u16>,
    pub ca: Option<u16>,
    pub pa: Option<u16>,
    pub pa_range: Option<[u16; 2]>,
    pub rep: Option<i32>,
    pub wrep: Option<i32>,
    pub club: String,
    pub club_lower: String,
    pub club_uid: String,
    pub club_nat: Option<i32>,
    pub club_rep: Option<i32>,
    pub loan: bool,
    pub loan_from: String,
    pub value: Option<i64>,
    pub wage: Option<i64>,
    pub end: String,
    pub status: String,
    pub foot: Option<Foot>,
    pub lf: Option<u8>,
    pub rf: Option<u8>,
    /// 0 means absent; 1..=20 is the FM attribute value.
    pub attrs: [u8; Attr::COUNT],
    /// 0 means absent; 1..=20 is the FM position proficiency.
    pub pos_r: [u8; ScoutPosKey::COUNT],
    /// Bitmask of accomplished/natural (`>= 15`) `PosGroup::bit()` and `PosGroup::side_bit()`.
    pub pos_mask: u32,
    /// Bitmask of natural (`>= 18`) `PosGroup::bit()`.
    pub nat_pos_mask: u16,
    pub pos_raw: String,
    /// 0 means absent; 1..=20 for the 8 personality dimensions.
    pub pers: [u8; ScoutPers::COUNT],
    /// 0 means absent; 1..=20 for the 5 hidden attributes.
    pub hid: [u8; ScoutHid::COUNT],
}

impl ScoutPlayer {
    #[inline]
    pub fn pers_val(&self, p: ScoutPers) -> Option<u8> {
        let v = self.pers[p as usize];
        (v > 0).then_some(v)
    }

    #[inline]
    pub fn hid_val(&self, h: ScoutHid) -> Option<u8> {
        let v = self.hid[h as usize];
        (v > 0).then_some(v)
    }

    #[inline]
    pub fn has_nat_group(&self, group: PosGroup) -> bool {
        (self.nat_pos_mask & (group.bit() as u16)) != 0
    }

    /// Convert into a full `Player` (e.g. for player card modal or opponent squad injection).
    pub fn to_player(&self) -> Player {
        let mut a = BTreeMap::new();
        for attr in Attr::ALL {
            let v = self.attrs[attr.index()];
            if v > 0 {
                a.insert(attr, v as f32);
            }
        }
        let mut pos_r = BTreeMap::new();
        for (i, &name) in ScoutPosKey::NAMES.iter().enumerate() {
            if self.pos_r[i] > 0 {
                pos_r.insert(name.to_string(), self.pos_r[i] as f32);
            }
        }
        let mut pos = Vec::new();
        let mut nat_pos = Vec::new();
        for g in PosGroup::ALL {
            if self.has_pos_group(g) {
                pos.push(g.code().to_string());
                if g.is_wide_group() {
                    if self.has_pos_side(g, Side::L) {
                        pos.push(format!("{}:L", g.code()));
                    }
                    if self.has_pos_side(g, Side::R) {
                        pos.push(format!("{}:R", g.code()));
                    }
                }
            }
            if self.has_nat_group(g) {
                nat_pos.push(g.code().to_string());
            }
        }
        let mut pers = BTreeMap::new();
        for (i, &name) in ScoutPers::NAMES.iter().enumerate() {
            if self.pers[i] > 0 {
                pers.insert(name.to_string(), self.pers[i] as f32);
            }
        }
        let mut hid = BTreeMap::new();
        for (i, &key) in ScoutHid::KEYS.iter().enumerate() {
            if self.hid[i] > 0 {
                hid.insert(key.to_string(), self.hid[i] as f32);
            }
        }
        Player {
            id: self.id.clone(),
            name: self.name.clone(),
            pos,
            pos_raw: self.pos_raw.clone(),
            age: self.age.map(|v| v as f32),
            a,
            meta: PlayerMeta {
                uid: self.uniq.clone().or_else(|| self.uid.clone()),
                club: (!self.club.is_empty()).then(|| self.club.clone()),
                foot: self.foot.map(|f| f.en_label().to_string()),
                height: self.height.map(|h| h.to_string()),
                ca: self.ca.map(|v| v.to_string()),
                pa: self.pa.map(|v| v.to_string()),
                ..PlayerMeta::default()
            },
            pos_r,
            ca: self.ca.map(|v| v as f32),
            pa: self.pa.map(|v| v as f32),
            pa_range: self.pa_range,
            pers,
            hid,
            value: self.value.map(|v| v as f64),
            wage: self.wage.map(|v| v as f64),
            end: (!self.end.is_empty()).then(|| self.end.clone()),
            status: (!self.status.is_empty()).then(|| self.status.clone()),
            rep: self.rep.map(|v| v as f32),
            wrep: self.wrep.map(|v| v as f32),
            club_rep: self.club_rep.map(|v| v as f32),
            club_nat: self.club_nat,
            height: self.height.map(|v| v as f32),
            lf: self.lf.map(|v| v as f32),
            rf: self.rf.map(|v| v as f32),
            foot: self.foot,
            loan: Some(self.loan),
            loan_from: (!self.loan_from.is_empty()).then(|| self.loan_from.clone()),
            nat: self.nat,
            uid: self.uid.clone(),
            uniq: self.uniq.clone(),
            club: (!self.club.is_empty()).then(|| self.club.clone()),
            nat_pos,
        }
    }
}

impl AttrLookup for ScoutPlayer {
    #[inline]
    fn attr(&self, attr: Attr) -> Option<f32> {
        let v = self.attrs[attr.index()];
        (v > 0).then_some(v as f32)
    }

    #[inline]
    fn has_pos_group(&self, group: PosGroup) -> bool {
        (self.pos_mask & group.bit()) != 0
    }

    #[inline]
    fn has_pos_side(&self, group: PosGroup, side: Side) -> bool {
        match side {
            Side::C => self.has_pos_group(group),
            Side::L | Side::R => (self.pos_mask & group.side_bit(side)) != 0,
        }
    }

    #[inline]
    fn has_any_pos(&self) -> bool {
        self.pos_mask != 0
    }

    #[inline]
    fn is_only_gk(&self) -> bool {
        self.pos_mask == PosGroup::Gk.bit()
    }

    #[inline]
    fn foot_side(&self) -> Option<Foot> {
        self.foot
    }

    #[inline]
    fn height_cm(&self) -> Option<f32> {
        self.height.map(|v| v as f32)
    }

    #[inline]
    fn age_years(&self) -> Option<f32> {
        self.age.map(|v| v as f32)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScoutDb {
    pub players: Vec<ScoutPlayer>,
    pub nations: HashMap<i32, String>,
    pub date: Option<String>,
    pub date_guess: bool,
    pub save: String,
    pub cached: bool,
    pub parse_ms: u64,
    pub columns: usize,
    pub attr_cols: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PosLevel {
    #[default]
    Acc,
    Nat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoanFilter {
    #[default]
    Any,
    Exclude,
    Only,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoutSort {
    #[default]
    Ca,
    Pa,
    Age,
    Value,
    Wage,
    Likely,
    Role,
    Gain,
    Join,
    Name,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDir {
    Asc,
    #[default]
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScoutAttrFilter {
    pub k: Option<Attr>,
    pub min: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScoutFilter {
    pub q: String,
    pub age_min: Option<u16>,
    pub age_max: Option<u16>,
    pub ca_min: Option<u16>,
    pub ca_max: Option<u16>,
    pub pa_min: Option<u16>,
    pub pa_max: Option<u16>,
    pub pos: Option<PosGroup>,
    pub pos_level: PosLevel,
    pub foot: Option<Foot>,
    pub nation: Option<i32>,
    pub club: String,
    pub league: Option<i32>,
    pub value_max: Option<i64>,
    pub wage_max: Option<i64>,
    pub end_months: Option<u16>,
    pub free: bool,
    pub exclude_mine: bool,
    pub loan: LoanFilter,
    pub prof_min: Option<u8>,
    pub amb_min: Option<u8>,
    pub det_min: Option<u8>,
    pub cons_min: Option<u8>,
    pub imp_min: Option<u8>,
    pub inj_max: Option<u8>,
    pub height_min: Option<u16>,
    pub attrs: Vec<ScoutAttrFilter>,
    pub role: Option<String>,
    pub role_min: Option<u8>,
    pub likely_min: Option<u8>,
    pub club_rep_max: Option<i32>,
    pub join_min: Option<u8>,
    pub sort: ScoutSort,
    pub dir: SortDir,
}

impl Default for ScoutFilter {
    fn default() -> Self {
        Self {
            q: String::new(),
            age_min: None,
            age_max: None,
            ca_min: None,
            ca_max: None,
            pa_min: None,
            pa_max: None,
            pos: None,
            pos_level: PosLevel::Acc,
            foot: None,
            nation: None,
            club: String::new(),
            league: None,
            value_max: None,
            wage_max: None,
            end_months: None,
            free: false,
            exclude_mine: true,
            loan: LoanFilter::Any,
            prof_min: None,
            amb_min: None,
            det_min: None,
            cons_min: None,
            imp_min: None,
            inj_max: None,
            height_min: None,
            attrs: Vec::new(),
            role: None,
            role_min: None,
            likely_min: None,
            club_rep_max: None,
            join_min: None,
            sort: ScoutSort::Ca,
            dir: SortDir::Desc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoutContext<'a> {
    pub date: Option<&'a str>,
    pub my_club: Option<&'a str>,
    pub tactic: Option<&'a Tactic>,
    pub squad: &'a [Player],
    pub xi_ca: u16,
    pub cheap_value: Option<i64>,
    pub bargain_value: Option<i64>,
    pub my_rep: Option<i32>,
}

impl Default for ScoutContext<'_> {
    fn default() -> Self {
        Self {
            date: None,
            my_club: None,
            tactic: None,
            squad: &[],
            xi_ca: 120,
            cheap_value: None,
            bargain_value: None,
            my_rep: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinLikelihood {
    pub score: u8,
    pub why: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestGain {
    pub code: String,
    pub gain: i16,
    pub his: u8,
    pub cur: u8,
    pub starter: String,
}

/// Filtered scouting match pointing into `ScoutDb::players[index]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoutMatch {
    pub index: usize,
    pub likely: Option<u8>,
    pub join: Option<u8>,
    pub role_score: Option<u8>,
    pub gain: Option<i16>,
    pub gain_slot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoutPreset {
    pub key: &'static str,
    pub icon: &'static str,
    pub name: String,
    pub desc: String,
    pub filter: ScoutFilter,
    pub need_squad: bool,
    pub need_my_club: bool,
    pub weak_spot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShortlistEntry {
    pub id: String,
    pub uniq: Option<String>,
    pub nat: Option<i32>,
    pub name: String,
    pub age: Option<u16>,
    pub club: String,
    pub pos_raw: String,
    pub ca: Option<u16>,
    pub pa: Option<u16>,
    pub value: Option<i64>,
    pub wage: Option<i64>,
    pub end: String,
    pub pos: Vec<String>,
    pub nat_pos: Vec<String>,
}

// ============================================================================
// Live Animated Pitch Simulation Types (Non-Bible 6-Phase Match Model)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimPhase {
    Build = 0,
    Prog = 1,
    Final = 2,
    Loss = 3,
    Oop = 4,
    Win = 5,
}

impl SimPhase {
    pub const COUNT: usize = 6;
    pub const ALL: [SimPhase; Self::COUNT] = [
        Self::Build,
        Self::Prog,
        Self::Final,
        Self::Loss,
        Self::Oop,
        Self::Win,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_index(idx: usize) -> Self {
        Self::ALL[idx % Self::COUNT]
    }

    pub const fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Self::Build, Lang::Pt) => "Construção",
            (Self::Build, Lang::En) => "Build-up",
            (Self::Prog, Lang::Pt) => "Progressão",
            (Self::Prog, Lang::En) => "Progression",
            (Self::Final, Lang::Pt) => "Último terço",
            (Self::Final, Lang::En) => "Final third",
            (Self::Loss, Lang::Pt) => "Perda da bola",
            (Self::Loss, Lang::En) => "Ball lost",
            (Self::Oop, Lang::Pt) => "Bloco sem bola",
            (Self::Oop, Lang::En) => "Out-of-possession block",
            (Self::Win, Lang::Pt) => "Recuperação",
            (Self::Win, Lang::En) => "Ball won",
        }
    }

    /// `"IP"`, `"OOP"`, or `"T"` (transition).
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Build | Self::Prog | Self::Final => "IP",
            Self::Oop => "OOP",
            Self::Loss | Self::Win => "T",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SimPoint {
    pub x: f32,
    pub y: f32,
}

impl SimPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BallKind {
    #[default]
    Pass,
    Loft,
    Curve,
    Dribble,
    Shot,
    Opp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallPoint {
    pub x: f32,
    pub y: f32,
    pub kind: BallKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallSample {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub kind: BallKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimArrowKind {
    Run,
    Press,
    Cover,
    Role,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimArrow {
    pub from: SimPoint,
    pub to: SimPoint,
    pub color: &'static str,
    pub kind: SimArrowKind,
    pub dim: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimZoneKind {
    Trap,
    Lane,
    Circle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimZone {
    pub kind: SimZoneKind,
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub color: &'static str,
    pub stroke: &'static str,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimLine {
    pub y: f32,
    pub color: &'static str,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimOverlayKind {
    Gap,
    Behind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimOverlay {
    pub kind: SimOverlayKind,
    pub y1: f32,
    pub y2: f32,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimPlayer {
    pub idx: usize,
    pub slot: SlotAssignment,
    pub ip_role: &'static str,
    pub oop_role: &'static str,
    pub sg: i8,
    pub sgb: i8,
    pub pressing: bool,
    /// Position for each of the 6 `SimPhase`s indexed by `SimPhase::index()`.
    pub pos: [SimPoint; SimPhase::COUNT],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimModel {
    pub players: Vec<SimPlayer>,
    pub ball: [Vec<BallPoint>; SimPhase::COUNT],
    pub opp: [Vec<SimPoint>; SimPhase::COUNT],
    pub overlays: Vec<SimOverlay>,
    pub arrows: [Vec<SimArrow>; SimPhase::COUNT],
    pub zones: [Vec<SimZone>; SimPhase::COUNT],
    pub lines: [Vec<SimLine>; SimPhase::COUNT],
    pub d_line: f32,
    pub front: f32,
    pub bs: i8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseCaption {
    pub text: String,
    /// `(role_ab, slot_code, note_text)`
    pub notes: Vec<(String, String, String)>,
}

// ============================================================================
// FM27 Bridge Protocol Types (Decoupled from TCP via `BridgeTransport`)
// ============================================================================

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("bridge rejected command: {0}")]
    Protocol(String),
    #[error("invalid response: {0}")]
    Parse(String),
    #[error("{0}")]
    State(String),
}

/// Abstract line-protocol transport to `TacticalBibleBridge` (`127.0.0.1:27029`).
/// Implemented by `ta-backend`'s TCP client in production and by mock transports in unit tests.
pub trait BridgeTransport {
    fn call(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, BridgeError>;

    fn sleep_ms(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub ok: bool,
    pub ready: bool,
    pub version: String,
    pub fm: String,
    pub err: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeContext {
    pub manager: i64,
    pub team: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSlot {
    pub index: i32,
    pub pos: String,
    pub person: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeTacticState {
    pub name: String,
    pub slots: Vec<BridgeSlot>,
    pub sel_index: HashMap<i64, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTacticSlotInfo {
    pub slot: i32,
    pub ip: u64,
    pub oop: u64,
    pub ip_role: Option<u64>,
    pub oop_role: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRoutineOption {
    pub id: i32,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRoutineSlot {
    pub group: i32,
    pub slot: i32,
    pub name: String,
    pub person: i64,
    pub option: i32,
    pub label: String,
    pub options: Vec<BridgeRoutineOption>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRoutine {
    pub kind: i32,
    pub side: i32,
    pub slots: Vec<BridgeRoutineSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeXiResult {
    pub placed: usize,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeSquadResult {
    pub team: i64,
    pub club: String,
    pub players: Vec<Player>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeNextOpponent {
    pub team: i64,
    pub name: String,
    pub uniqs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeShortlistResult {
    pub added: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeFocusItem {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeFocusArea {
    pub id: i64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeFocusRequest {
    pub name: String,
    pub age_min: u16,
    pub age_max: u16,
    pub ca_stars: f32,
    pub pa_stars: f32,
    pub areas: Vec<BridgeFocusArea>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BridgeRecording {
    pub on: bool,
    pub out: String,
}

// ============================================================================
// Persistence & Legacy Backup Compatibility (`fm27tb.v1` / `.fm27tb.json`)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AutoSavePrefs {
    pub save: bool,
    pub save_dir: String,
    pub save_mtime: u64,
    pub save_err: String,
    pub save_last: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PersistedState {
    pub tactic: Tactic,
    pub squad: SquadPack,
    pub opp: SquadPack,
    pub my_club: Option<String>,
    pub saved: Vec<Tactic>,
    pub theme: String,
    pub auto: AutoSavePrefs,
    pub lang: Lang,
    pub sheet_names: Option<String>,
    pub fm_dir: Option<String>,
    pub plan_k: Option<GameStateKind>,
    pub shortlist: Vec<ShortlistEntry>,
    pub sq_mode: Option<String>,
}

/// Legacy `.fm27tb.json` import/export envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AppBackup {
    #[serde(rename = "fm27tb-tactic")]
    Tactic {
        #[serde(default = "default_v1")]
        v: u32,
        tactic: Tactic,
    },
    #[serde(rename = "fm27tb-all")]
    All {
        #[serde(default = "default_v1")]
        v: u32,
        tactic: Option<Tactic>,
        squad: Option<SquadPack>,
        opp: Option<SquadPack>,
        saved: Option<Vec<Tactic>>,
        #[serde(rename = "myClub")]
        my_club: Option<String>,
    },
}

const fn default_v1() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_roundtrip_all_47() {
        assert_eq!(Attr::ALL.len(), 47);
        for (i, &a) in Attr::ALL.iter().enumerate() {
            assert_eq!(a.index(), i);
            assert_eq!(Attr::from_index(i), Some(a));
            assert_eq!(Attr::from_code(a.code()), Some(a));
        }
    }

    #[test]
    fn legacy_fm27tb_backup_deserializes() {
        let json = r#"{
            "kind": "fm27tb-tactic",
            "v": 1,
            "tactic": {
                "name": "4-3-3 Posse",
                "ment": "Positive",
                "ipForm": "4-3-3 DM Wide",
                "oopForm": "4-1-4-1 DM Wide",
                "slots": [{
                    "ipCode": "ST",
                    "oopCode": "ST",
                    "ip": "f9",
                    "oop": "tcf",
                    "pid": "u12345",
                    "pis": ["Move into channels"]
                }],
                "ti": { "passing": 1, "loe": 2 },
                "ois": 3,
                "source": "manual"
            }
        }"#;
        let parsed: AppBackup = serde_json::from_str(json).expect("legacy backup should parse");
        match parsed {
            AppBackup::Tactic { v, tactic } => {
                assert_eq!(v, 1);
                assert_eq!(tactic.ment, Mentality::Positive);
                assert_eq!(tactic.slots.len(), 1);
                assert_eq!(tactic.slots[0].ip_code, "ST");
                assert_eq!(tactic.ti.get("passing"), Some(&1));
            }
            _ => panic!("expected Tactic backup"),
        }
    }

    #[test]
    fn scout_player_attr_lookup_and_conversion() {
        let mut attrs = [0u8; Attr::COUNT];
        attrs[Attr::Fin.index()] = 16;
        attrs[Attr::Pac.index()] = 15;
        let sp = ScoutPlayer {
            id: "db1".into(),
            uid: Some("1".into()),
            uniq: Some("999".into()),
            name: "Test Striker".into(),
            name_lower: "test striker".into(),
            age: Some(21),
            nat: Some(1),
            height: Some(184),
            ca: Some(140),
            pa: Some(165),
            pa_range: None,
            rep: Some(6000),
            wrep: Some(5000),
            club: "Club A".into(),
            club_lower: "club a".into(),
            club_uid: "10".into(),
            club_nat: Some(1),
            club_rep: Some(6500),
            loan: false,
            loan_from: String::new(),
            value: Some(2_500_000),
            wage: Some(15_000),
            end: "2028-06-30".into(),
            status: "First Team".into(),
            foot: Some(Foot::R),
            lf: Some(10),
            rf: Some(20),
            attrs,
            pos_r: [0; ScoutPosKey::COUNT],
            pos_mask: PosGroup::St.bit() | PosGroup::AmLr.bit() | PosGroup::AmLr.side_bit(Side::L),
            nat_pos_mask: PosGroup::St.bit() as u16,
            pos_raw: "STC, AML".into(),
            pers: [0; ScoutPers::COUNT],
            hid: [0; ScoutHid::COUNT],
        };
        assert_eq!(sp.attr(Attr::Fin), Some(16.0));
        assert_eq!(sp.attr(Attr::Cor), None);
        assert!(sp.has_pos_group(PosGroup::St));
        assert!(sp.has_pos_side(PosGroup::AmLr, Side::L));
        assert!(!sp.has_pos_side(PosGroup::AmLr, Side::R));
        let p = sp.to_player();
        assert_eq!(p.attr(Attr::Fin), Some(16.0));
        assert!(p.has_pos_side(PosGroup::AmLr, Side::L));
        assert!(!p.has_pos_side(PosGroup::AmLr, Side::R));
    }
}

