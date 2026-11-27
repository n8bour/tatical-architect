//! Tactical Bible Bridge — FM27 live bridge protocol client (`127.0.0.1:27029`).
//!
//! Faithful 1:1 Rust port of `src/ui/fmbridge.js`.
//! ALL FM27-specific numeric IDs live in [`FmbIds`] below: after an FM27 patch
//! this is the only block to re-check. Capture new IDs with the plugin's
//! `EVENTS` / `RECORD` commands (do the action by hand in FM, then read what FM sent).

use crate::types::{
    Attr, AttrLookup, BridgeContext, BridgeError, BridgeFocusItem, BridgeFocusRequest,
    BridgeNextOpponent, BridgeRecording, BridgeRoutine, BridgeRoutineOption, BridgeRoutineSlot,
    BridgeShortlistResult, BridgeSlot, BridgeSquadResult, BridgeStatus, BridgeTacticSlotInfo,
    BridgeTacticState, BridgeTransport, BridgeXiResult, Foot, Lang, Player, PlayerMeta, PosGroup,
    SetPiecePlan, ShortlistEntry, SlotAssignment, Tactic,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::time::{Duration, Instant};

// ============================================================================
// FM27 ID Specifications (ZERO FM IDs hardcoded outside `FmbIds`)
// ============================================================================

/// Mapping of an app role ID to its 64-bit FM27 role bitmask and English name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticRoleSpec {
    pub id: &'static str,
    pub bits: u64,
    pub en_name: &'static str,
}

/// Mapping of an app team-instruction key to its FM27 instruction ID, option count,
/// app-index -> FM-index translation table, app default index, and display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticTiSpec {
    pub key: &'static str,
    pub name: &'static str,
    pub fm_id: i64,
    pub fm_count: usize,
    pub default_opt: usize,
    pub map: &'static [Option<usize>],
}

/// Mapping of an app player-instruction name to FM27 `(in_possession, category_id, prev_presses)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticPiSpec {
    pub name: &'static str,
    pub in_possession: bool,
    pub category: i64,
    pub prev_presses: usize,
}

/// Centralized FM27 bridge protocol IDs (`FMB_IDS`).
///
/// Verified against FM27 27.1.0.2505117 with Tactical Bible Bridge 0.4.2.
pub struct FmbIds;

impl FmbIds {
    pub const BUILT_FOR: &'static str = "FM27 27.1 (verified with Tactical Bible Bridge 0.4)";
    pub const DEFAULT_HOST: &'static str = "127.0.0.1";
    pub const DEFAULT_PORT: u16 = 27029;

    // ---- Common reference pair keys & type tags ----
    pub const PAIR_TYPE_KEY: i64 = 825_307_168;
    pub const PAIR_SESSION_KEY: i64 = 1_198_877_028;
    pub const PAIR_SESSION_DEFAULT: i64 = 1;
    pub const PAIR_TACTIC_FLAG_KEY: i64 = 1_651_271_022;
    pub const PAIR_TACTIC_FLAG_VAL: i64 = 1;
    pub const PAIR_ACTION_KEY: i64 = 1_633_904_996;
    pub const PAIR_FOCUS_TEAM_KEY: i64 = 1_769_566_810;
    pub const PAIR_SEARCH_FLAG1_KEY: i64 = 1_719_300_978;
    pub const PAIR_SEARCH_FLAG1_VAL: i64 = 1;
    pub const PAIR_SEARCH_FLAG2_KEY: i64 = 1_954_115_685;
    pub const PAIR_SEARCH_FLAG2_VAL: i64 = 0;

    pub const PAIR_TYPE_ACTION: i64 = 1;
    pub const PAIR_TYPE_PERSON_OR_TEAM: i64 = 28;
    pub const PAIR_TYPE_FOCUS: i64 = 85;
    pub const PAIR_TYPE_SET_PIECE: i64 = 93;
    pub const PAIR_TYPE_SCOUTING_INFO: i64 = 97;
    pub const PAIR_TYPE_SEARCH: i64 = 98;
    pub const PAIR_TYPE_TACTIC: i64 = 126;
    pub const PAIR_TYPE_TRAINING: i64 = 128;

    // ---- Top-level properties & events ----
    pub const PROP_NAME: i64 = 1_851_878_757;
    pub const PROP_SLOTS: i64 = 1_919_628_150;
    pub const PROP_SELECTION: i64 = 1_098_208_873;
    pub const PROP_TRAINING_PLAYERS: i64 = 1_432_578_628;

    pub const EV_PLACE: i64 = 826_434_931;

    // ---- Common record keys ----
    pub const KEY_SLOT: i64 = 1_414_752_632;
    pub const KEY_POS: i64 = 1_835_167_823;
    pub const KEY_PERSON: i64 = 1_886_157_170;
    pub const KEY_SEL: i64 = 1_883_858_041;
    pub const KEY_PERSON_ID: i64 = 1_685_218_662;
    pub const KEY_NUMBER: i64 = 1_886_680_684;
    pub const KEY_PAYLOAD: i64 = 1_903_577_654;
    pub const KEY_SEL_IDX: i64 = 1_635_149_634;

    // ---- Reference types & context keys ----
    pub const REF_TEAM_TRAINING: &'static str = "FM.UI.TeamTrainingReference";
    pub const REF_TACTIC: &'static str = "FM.UI.TeamTacticReference";
    pub const REF_CALENDAR: &'static str = "FM.UI.CalendarReference";
    pub const REF_PERSON: &'static str = "FM.UI.PersonReference";

    pub const REF_TEAM_TRAINING_MANAGER: i64 = 1_348_620_377;
    pub const REF_TEAM_TRAINING_TEAM: i64 = 1_416_914_273;
    pub const REF_CALENDAR_MANAGER: i64 = 1_214_805_357;
    pub const REF_CALENDAR_TEAM: i64 = 1_415_930_221;

    // ---- Set pieces ----
    pub const SP_EV_REMOVE: i64 = 910_380_099;
    pub const SP_EV_ADD_LEFT: i64 = 1_213_227_378;
    pub const SP_EV_ADD_RIGHT: i64 = 1_416_066_355;
    pub const SP_EV_TECHNIQUE: i64 = 927_877_996;
    pub const SP_EV_TARGET: i64 = 1_249_537_108;

    pub const SP_PROP_TYPE: i64 = 1_668_375_608;
    pub const SP_KEY_KIND: i64 = 1_398_883_404;
    pub const SP_KEY_LIST: i64 = 961_376_122;

    pub const SP_KIND_CORNER: i32 = 5;
    pub const SP_KIND_FREE_KICK: i32 = 6;
    pub const SP_KIND_ATT_CORNER: i32 = 5;
    pub const SP_KIND_DEF_CORNER: i32 = 0;
    pub const SP_KIND_ATT_FK: i32 = 6;
    pub const SP_KIND_DEF_FK: i32 = 1;
    pub const SP_KIND_THROW_IN: i32 = 9;

    pub const SP_TECH_INSWING: i32 = 0;
    pub const SP_TECH_OUTSWING: i32 = 1;

    pub const SP_TARGET_MIXED: i32 = 0;
    pub const SP_TARGET_SHORT: i32 = 1;
    pub const SP_TARGET_NEAR_POST: i32 = 2;
    pub const SP_TARGET_FAR_POST: i32 = 3;
    pub const SP_TARGET_EDGE: i32 = 4;
    pub const SP_TARGET_CENTRE: i32 = 5;

    pub const SP_TARGET_LABELS: [&'static str; 6] = [
        "Mixed",
        "Short",
        "Near Post",
        "Far Post",
        "Edge of Area",
        "Centre",
    ];

    pub const SP_ROUTINE_PROP: i64 = 1_867_869_768;
    pub const SP_ROUTINE_KIND: i64 = 1_398_883_404;
    pub const SP_ROUTINE_SIDE: i64 = 1_919_054_668;
    pub const SP_ROUTINE_IN_BOX: i64 = 1_750_096_198;
    pub const SP_ROUTINE_OUT_BOX: i64 = 945_304_884;
    pub const SP_ROUTINE_GROUP: i64 = 1_114_920_816;
    pub const SP_ROUTINE_SLOTS: i64 = 1_752_528_242;
    pub const SP_ROUTINE_SLOT: i64 = 1_883_858_041;
    pub const SP_ROUTINE_NAME: i64 = 1_851_878_757;
    pub const SP_ROUTINE_PERSON: i64 = 1_886_157_170;
    pub const SP_ROUTINE_CURRENT: i64 = 1_750_627_616;
    pub const SP_ROUTINE_OPTIONS: i64 = 1_833_723_970;
    pub const SP_ROUTINE_OPT_ITEM: i64 = 1_383_036_740;
    pub const SP_ROUTINE_OPTION: i64 = 1_750_627_616;
    pub const SP_ROUTINE_EV: i64 = 1_633_039_705;
    pub const SP_ROUTINE_FLAG: i64 = 1_279_880_290;

    /// App attacking-corner plan role -> preferred FM option IDs.
    pub const SP_ATT_ROLES: &'static [(&'static str, &'static [i32])] = &[
        ("far", &[6, 9, 4]),
        ("near", &[5, 8, 4]),
        ("centre", &[4]),
        ("spot", &[4, 11]),
        ("edge", &[7, 10]),
        ("short", &[12]),
        ("back", &[2, 3]),
    ];

    /// App defending-corner plan role -> preferred FM option IDs.
    pub const SP_DEF_ROLES: &'static [(&'static str, &'static [i32])] = &[
        ("zn", &[7, 3]),
        ("zc", &[9, 8, 10]),
        ("zf", &[11, 4]),
        ("m", &[6]),
        ("de", &[12]),
        ("o0", &[2, 13, 14]),
        ("o1", &[14, 13, 2]),
    ];

    /// Decoded set-piece events not yet wired in the app.
    pub const SP_TODO_PENALTY_MOVE: i64 = 1_649_488_964;
    pub const SP_TODO_LIST_MOVE: i64 = 1_144_009_829;
    pub const SP_TODO_SLOT_MOVE: i64 = 1_449_477_207;

    // ---- Tactic ----
    pub const TACTIC_SLOTS: i64 = 1_919_628_150;
    pub const TACTIC_SLOT_IDX: i64 = 1_414_752_632;
    pub const TACTIC_IP_POS: i64 = 1_835_167_823;
    pub const TACTIC_OOP_POS: i64 = 927_294_289;
    pub const TACTIC_IP_ROLES: i64 = 930_166_885;
    pub const TACTIC_IP_ROLE: i64 = 1_698_852_728;
    pub const TACTIC_OOP_ROLES: i64 = 1_798_391_651;
    pub const TACTIC_OOP_ROLE: i64 = 1_349_481_322;
    /// OOP role field carries side flags in bits 21..=23 (bit 20 is Central Midfielder).
    pub const TACTIC_OOP_ROLE_SIDE_MASK: u64 = 2_097_152 | 4_194_304 | 8_388_608;

    pub const TACTIC_MOVE: i64 = 21;
    pub const TACTIC_MOVE_FROM: i64 = 929_121_399;
    pub const TACTIC_MOVE_TO: i64 = 944_851_554;
    pub const TACTIC_MOVE_IP: i64 = 1_769_491_558;
    pub const TACTIC_MOVE_SLOT: i64 = 857_762_170;

    pub const TACTIC_SET_ROLE: i64 = 1_467_503_190;
    pub const TACTIC_ROLE_REC: i64 = 1_349_481_322;
    pub const TACTIC_ROLE_NAME: i64 = 909_713_440;
    pub const TACTIC_ROLE_IP: i64 = 1_265_792_339;
    pub const TACTIC_ROLE_RATING_REC: i64 = 1_464_367_445;
    pub const TACTIC_ROLE_RATING_CUR: i64 = 1_298_233_430;
    pub const TACTIC_ROLE_RATING_MAX: i64 = 1_298_755_158;
    pub const TACTIC_ROLE_RATING_FLAG: i64 = 1_397_715_049;
    pub const TACTIC_ROLE_LOCK_FLAG: i64 = 1_936_487_524;

    pub const TACTIC_MENTALITY: i64 = 1_162_170_689;
    pub const TACTIC_MENU: i64 = 1_949_648_203;
    pub const TACTIC_MENU_ITEM: i64 = 1_383_036_740;
    pub const TACTIC_MENU_COPY: i32 = 0;
    pub const TACTIC_MENU_SUB: i64 = 876_950_356;
    pub const TACTIC_MAX_TACTICS: usize = 3;

    pub const TACTIC_WIZARD_OPEN: i64 = 1_412_787_285;
    pub const TACTIC_WIZARD_NEXT: i64 = 1_515_149_153;
    pub const TACTIC_WIZARD_FINISH: i64 = 879_515_246;
    pub const TACTIC_WIZARD_STYLE: i64 = 1_381_059_181;
    pub const TACTIC_WIZARD_STYLE_BASE: i64 = 10; // Clean Slate
    pub const TACTIC_NAME: i64 = 1_851_878_757;

    pub const TACTIC_PI_ENABLED: bool = true;
    pub const TACTIC_PI_PREV: i64 = 1_668_445_234;
    pub const TACTIC_PI_CAT: i64 = 1_449_616_730;
    pub const TACTIC_PI_REVERT: i64 = 1_984_657_952;
    pub const TACTIC_PI_REVERT_REC: i64 = 1_936_487_524;

    pub const TACTIC_TI_PREV: i64 = 1_249_276_755;
    pub const TACTIC_TI_NEXT: i64 = 862_931_569;

    /// Left/right flags added to central positions (`DCL = DC + TACTIC_LEFT`).
    pub const TACTIC_LEFT: u64 = 1_048_576;
    pub const TACTIC_RIGHT: u64 = 2_097_152;

    /// Spare position bits used to break OOP move cycles.
    pub const TACTIC_OOP_SPARE_POSITIONS: [u64; 10] = [
        256, 512, 32, 64, 2048, 4096, 8192, 16384, 1024, 128,
    ];

    /// All 20 mapped FM27 player instructions (`‹` presses from Standard after Revert).
    pub const TACTIC_PIS: &'static [TacticPiSpec] = &[
        TacticPiSpec { name: "Sit narrower", in_possession: true, category: 0, prev_presses: 1 },
        TacticPiSpec { name: "Stay wider", in_possession: true, category: 0, prev_presses: 2 },
        TacticPiSpec { name: "Dribble less", in_possession: true, category: 2, prev_presses: 1 },
        TacticPiSpec { name: "Dribble more", in_possession: true, category: 2, prev_presses: 2 },
        TacticPiSpec { name: "Get further forward", in_possession: true, category: 3, prev_presses: 1 },
        TacticPiSpec { name: "Roam from position", in_possession: true, category: 4, prev_presses: 1 },
        TacticPiSpec { name: "Shoot less", in_possession: true, category: 6, prev_presses: 1 },
        TacticPiSpec { name: "Shoot more", in_possession: true, category: 6, prev_presses: 2 },
        TacticPiSpec { name: "Take fewer risks", in_possession: true, category: 8, prev_presses: 1 },
        TacticPiSpec { name: "Take more risks", in_possession: true, category: 8, prev_presses: 2 },
        TacticPiSpec { name: "Target Near Post", in_possession: true, category: 9, prev_presses: 2 },
        TacticPiSpec { name: "Target Centre", in_possession: true, category: 9, prev_presses: 3 },
        TacticPiSpec { name: "Target Far Post", in_possession: true, category: 9, prev_presses: 4 },
        TacticPiSpec { name: "Cross from byline", in_possession: true, category: 10, prev_presses: 1 },
        TacticPiSpec { name: "Cross from deep", in_possession: true, category: 10, prev_presses: 2 },
        TacticPiSpec { name: "Press less", in_possession: false, category: 11, prev_presses: 1 },
        TacticPiSpec { name: "Press more", in_possession: false, category: 11, prev_presses: 2 },
        TacticPiSpec { name: "Mark tighter", in_possession: false, category: 12, prev_presses: 1 },
        TacticPiSpec { name: "Ease off tackles", in_possession: false, category: 13, prev_presses: 1 },
        TacticPiSpec { name: "Tackle harder", in_possession: false, category: 13, prev_presses: 2 },
    ];

    /// All 69 verified FM27 64-bit role bitmasks (exact copy of `FMB_IDS.tactic.roles`).
    pub const TACTIC_ROLES: &'static [TacticRoleSpec] = &[
        TacticRoleSpec { id: "acb", bits: 16_384, en_name: "Advanced Centre-Back" },
        TacticRoleSpec { id: "am_ip", bits: 512, en_name: "Attacking Midfielder" },
        TacticRoleSpec { id: "am_oop", bits: 1_073_741_824, en_name: "Attacking Midfielder" },
        TacticRoleSpec { id: "ap", bits: 131_072, en_name: "Advanced Playmaker" },
        TacticRoleSpec { id: "awb", bits: 274_877_906_944, en_name: "Advanced Wing-Back" },
        TacticRoleSpec { id: "b2b", bits: 65_536, en_name: "Box-To-Box Midfielder" },
        TacticRoleSpec { id: "b2bp", bits: 70_368_744_177_664, en_name: "Box-To-Box Playmaker" },
        TacticRoleSpec { id: "bpcb", bits: 16_777_216, en_name: "Ball-Playing Centre-Back" },
        TacticRoleSpec { id: "bpgk", bits: 4_096, en_name: "Ball-Playing Goalkeeper" },
        TacticRoleSpec { id: "cb_ip", bits: 2, en_name: "Centre-Back" },
        TacticRoleSpec { id: "cb_oop", bits: 8, en_name: "Centre-Back" },
        TacticRoleSpec { id: "ccb", bits: 32, en_name: "Covering Centre-Back" },
        TacticRoleSpec { id: "cf_ip", bits: 2_048, en_name: "Centre Forward" },
        TacticRoleSpec { id: "cf_oop", bits: 4_398_046_511_104, en_name: "Centre Forward" },
        TacticRoleSpec { id: "chf", bits: 2_147_483_648, en_name: "Channel Forward" },
        TacticRoleSpec { id: "chm", bits: 72_057_594_037_927_936, en_name: "Channel Midfielder" },
        TacticRoleSpec { id: "cm_ip", bits: 32, en_name: "Central Midfielder" },
        TacticRoleSpec { id: "cm_oop", bits: 1_048_576, en_name: "Central Midfielder" },
        TacticRoleSpec { id: "coam", bits: 4_294_967_296, en_name: "Central Outlet Attacking Midfielder" },
        TacticRoleSpec { id: "cocf", bits: 17_592_186_044_416, en_name: "Central Outlet Centre Forward" },
        TacticRoleSpec { id: "ddm", bits: 65_536, en_name: "Dropping Defensive Midfielder" },
        TacticRoleSpec { id: "dlf", bits: 1_024, en_name: "Deep-Lying Forward" },
        TacticRoleSpec { id: "dlp", bits: 32_768, en_name: "Deep-Lying Playmaker" },
        TacticRoleSpec { id: "dm_ip", bits: 16, en_name: "Defensive Midfielder" },
        TacticRoleSpec { id: "dm_oop", bits: 32_768, en_name: "Defensive Midfielder" },
        TacticRoleSpec { id: "f9", bits: 1_099_511_627_776, en_name: "False Nine" },
        TacticRoleSpec { id: "fb_ip", bits: 4, en_name: "Full-Back" },
        TacticRoleSpec { id: "fb_oop", bits: 512, en_name: "Full-Back" },
        TacticRoleSpec { id: "gk_ip", bits: 1, en_name: "Goalkeeper" },
        TacticRoleSpec { id: "gk_oop", bits: 1, en_name: "Goalkeeper" },
        TacticRoleSpec { id: "hb", bits: 34_359_738_368, en_name: "Half Back" },
        TacticRoleSpec { id: "hfb", bits: 2_048, en_name: "Holding Full-Back" },
        TacticRoleSpec { id: "if_ip", bits: 134_217_728, en_name: "Inside Forward" },
        TacticRoleSpec { id: "ifb", bits: 4_503_599_627_370_496, en_name: "Inside Full-Back" },
        TacticRoleSpec { id: "iow", bits: 1_099_511_627_776, en_name: "Inside Outlet Winger" },
        TacticRoleSpec { id: "iw_ip", bits: 562_949_953_421_312, en_name: "Inside Winger" },
        TacticRoleSpec { id: "iwb", bits: 17_592_186_044_416, en_name: "Inside Wing-Back" },
        TacticRoleSpec { id: "lhk", bits: 4, en_name: "Line-Holding Keeper" },
        TacticRoleSpec { id: "mp", bits: 144_115_188_075_855_872, en_name: "Midfield Playmaker" },
        TacticRoleSpec { id: "nncb", bits: 536_870_912, en_name: "No-Nonsense Centre-Back" },
        TacticRoleSpec { id: "nngk", bits: 9_007_199_254_740_992, en_name: "No-Nonsense Goalkeeper" },
        TacticRoleSpec { id: "pcm", bits: 134_217_728, en_name: "Pressing Central Midfielder" },
        TacticRoleSpec { id: "pdm", bits: 131_072, en_name: "Pressing Defensive Midfielder" },
        TacticRoleSpec { id: "pfb", bits: 1_024, en_name: "Pressing Full-Back" },
        TacticRoleSpec { id: "poa", bits: 524_288, en_name: "Poacher" },
        TacticRoleSpec { id: "pw", bits: 8_796_093_022_208, en_name: "Playmaking Winger" },
        TacticRoleSpec { id: "pwb", bits: 36_028_797_018_963_968, en_name: "Playmaking Wing-Back" },
        TacticRoleSpec { id: "scb", bits: 16, en_name: "Stopping Centre-Back" },
        TacticRoleSpec { id: "scm", bits: 268_435_456, en_name: "Screening Central Midfielder" },
        TacticRoleSpec { id: "sdm", bits: 262_144, en_name: "Screening Defensive Midfielder" },
        TacticRoleSpec { id: "socf", bits: 35_184_372_088_832, en_name: "Splitting Outlet Centre Forward" },
        TacticRoleSpec { id: "swk", bits: 2, en_name: "Sweeper Keeper" },
        TacticRoleSpec { id: "tam", bits: 2_147_483_648, en_name: "Tracking Attacking Midfielder" },
        TacticRoleSpec { id: "tcf", bits: 8_796_093_022_208, en_name: "Tracking Centre Forward" },
        TacticRoleSpec { id: "tf", bits: 262_144, en_name: "Target Forward" },
        TacticRoleSpec { id: "tw", bits: 549_755_813_888, en_name: "Tracking Winger" },
        TacticRoleSpec { id: "twm", bits: 68_719_476_736, en_name: "Tracking Wide Midfielder" },
        TacticRoleSpec { id: "w_ip", bits: 128, en_name: "Winger" },
        TacticRoleSpec { id: "w_oop", bits: 274_877_906_944, en_name: "Winger" },
        TacticRoleSpec { id: "wb_ip", bits: 8, en_name: "Wing-Back" },
        TacticRoleSpec { id: "wb_oop", bits: 8, en_name: "Wing-Back" },
        TacticRoleSpec { id: "wccm", bits: 536_870_912, en_name: "Wide Covering Central Midfielder" },
        TacticRoleSpec { id: "wcdm", bits: 524_288, en_name: "Wide Covering Defensive Midfielder" },
        TacticRoleSpec { id: "wcm", bits: 281_474_976_710_656, en_name: "Wide Central Midfielder" },
        TacticRoleSpec { id: "wf_ip", bits: 35_184_372_088_832, en_name: "Wide Forward" },
        TacticRoleSpec { id: "wm_ip", bits: 64, en_name: "Wide Midfielder" },
        TacticRoleSpec { id: "wm_oop", bits: 34_359_738_368, en_name: "Wide Midfielder" },
        TacticRoleSpec { id: "wow", bits: 2_199_023_255_552, en_name: "Wide Outlet Winger" },
        TacticRoleSpec { id: "wowm", bits: 137_438_953_472, en_name: "Wide Outlet Wide Midfielder" },
    ];

    /// All 27 FM27 team instruction mappings (order matches `FMB_IDS.tactic.ti`).
    pub const TACTIC_TIS: &'static [TacticTiSpec] = &[
        TacticTiSpec { key: "passing", name: "Passing Directness", fm_id: 1, fm_count: 5, default_opt: 2, map: &[Some(0), Some(1), Some(2), Some(3), Some(4)] },
        TacticTiSpec { key: "tempo", name: "Tempo", fm_id: 21, fm_count: 5, default_opt: 2, map: &[Some(0), Some(1), Some(2), Some(3), Some(4)] },
        TacticTiSpec { key: "width", name: "Attacking Width", fm_id: 4, fm_count: 5, default_opt: 2, map: &[Some(0), Some(1), Some(2), Some(3), Some(4)] },
        TacticTiSpec { key: "creative", name: "Creative Freedom", fm_id: 5, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "att_trans", name: "Attacking Transition", fm_id: 3, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "set_pieces", name: "Play for Set Pieces", fm_id: 6, fm_count: 2, default_opt: 1, map: &[Some(0), None, Some(1)] },
        TacticTiSpec { key: "timewaste", name: "Time Wasting", fm_id: 7, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "reception", name: "Pass Reception", fm_id: 20, fm_count: 3, default_opt: 1, map: &[Some(2), Some(1), Some(0)] },
        TacticTiSpec { key: "buildup", name: "Build-up Strategy", fm_id: 24, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "goal_kicks", name: "Goal Kicks", fm_id: 29, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "gk_speed", name: "GK Distribution Speed", fm_id: 17, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "gk_target", name: "GK Distribution", fm_id: 16, fm_count: 7, default_opt: 2, map: &[Some(1), Some(2), Some(0), Some(3), Some(5)] },
        TacticTiSpec { key: "support_runs", name: "Supporting Runs", fm_id: 28, fm_count: 4, default_opt: 1, map: &[None, Some(0), Some(3)] },
        TacticTiSpec { key: "dribbling", name: "Dribbling", fm_id: 19, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "progress", name: "Progress Through", fm_id: 30, fm_count: 5, default_opt: 1, map: &[Some(2), Some(0), Some(1), Some(3)] },
        TacticTiSpec { key: "patience", name: "Final Third Patience", fm_id: 18, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "long_shots", name: "Shots from Distance", fm_id: 27, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "crossing", name: "Crossing Style", fm_id: 26, fm_count: 4, default_opt: 1, map: &[Some(3), Some(0), Some(2), Some(1)] },
        TacticTiSpec { key: "loe", name: "Line of Engagement", fm_id: 8, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "dline", name: "Defensive Line", fm_id: 9, fm_count: 5, default_opt: 2, map: &[Some(0), Some(1), Some(2), Some(3), Some(4)] },
        TacticTiSpec { key: "trigger", name: "Trigger Press", fm_id: 10, fm_count: 5, default_opt: 2, map: &[Some(0), Some(1), Some(2), Some(3), Some(4)] },
        TacticTiSpec { key: "def_trans", name: "Defensive Transition", fm_id: 11, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "tackling", name: "Tackling", fm_id: 12, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "press_trap", name: "Pressing Trap", fm_id: 25, fm_count: 3, default_opt: 1, map: &[Some(0), Some(1), Some(2)] },
        TacticTiSpec { key: "cross_eng", name: "Cross Engagement", fm_id: 22, fm_count: 3, default_opt: 1, map: &[Some(2), Some(1), Some(0)] },
        TacticTiSpec { key: "short_gk", name: "Short Goalkeeper Distribution", fm_id: 23, fm_count: 2, default_opt: 0, map: &[Some(0), Some(1)] },
        TacticTiSpec { key: "dl_behaviour", name: "Defensive Line Behaviour", fm_id: 15, fm_count: 3, default_opt: 1, map: &[Some(2), Some(1), Some(0)] },
    ];

    // ---- Team & Fixture ----
    pub const TEAM_NEXT_FIXTURE: i64 = 1_852_130_921;
    pub const TEAM_PLAYERS: i64 = 926_359_584;
    pub const TEAM_NAME: i64 = 1_851_878_757;

    pub const FIXTURE_OPP: i64 = 1_717_663_572;
    pub const FIXTURE_US: i64 = 1_718_119_764;
    pub const FIXTURE_DATE: i64 = 1_717_859_429;

    // ---- Person / Live Squad ----
    pub const PERSON_NAME: i64 = 1_851_878_757;
    pub const PERSON_UNIQUE_ID: i64 = 1_970_170_212;
    pub const PERSON_AGE: i64 = 825_565_216;
    pub const PERSON_HEIGHT: i64 = 825_761_824;
    pub const PERSON_FOOT: i64 = 1_244_885_353;
    pub const PERSON_FOOT_LEFT: i64 = 842_276_896;
    pub const PERSON_FOOT_RIGHT: i64 = 842_211_360;
    pub const PERSON_POSITIONS: i64 = 1_785_291_618;
    pub const PERSON_POS_BIT: i64 = 1_349_481_321;

    pub const PERSON_ATTR_LISTS: [i64; 5] = [
        1_095_920_457, // physical
        1_198_541_656, // technical
        1_667_594_052, // set pieces
        2_018_531_687, // mental
        1_515_086_679, // goalkeeping
    ];
    pub const PERSON_ATTR_VAL_BOX: i64 = 1_112_556_614;
    pub const PERSON_ATTR_VAL: i64 = 829_967_672;
    pub const PERSON_ATTR_INFO_BOX: i64 = 1_095_398_731;
    pub const PERSON_ATTR_ID: i64 = 1_768_169_504;

    /// All 47 language-independent FM27 attribute IDs (0..=46).
    pub const PERSON_ATTR_KEYS: [Attr; Attr::COUNT] = [
        Attr::Aer, // 0
        Attr::Cmd, // 1
        Attr::Com, // 2
        Attr::Ecc, // 3
        Attr::Han, // 4
        Attr::Kic, // 5
        Attr::Ref, // 6
        Attr::Tro, // 7
        Attr::Pun, // 8
        Attr::Thr, // 9
        Attr::Ovo, // 10
        Attr::Cro, // 11
        Attr::Dri, // 12
        Attr::Fin, // 13
        Attr::Hea, // 14
        Attr::Lon, // 15
        Attr::Lth, // 16
        Attr::Mar, // 17
        Attr::Pas, // 18
        Attr::Pen, // 19
        Attr::Fre, // 20
        Attr::Tck, // 21
        Attr::Tec, // 22
        Attr::Fir, // 23
        Attr::Cor, // 24
        Attr::Agg, // 25
        Attr::Ant, // 26
        Attr::Bra, // 27
        Attr::Vis, // 28
        Attr::Dec, // 29
        Attr::Det, // 30
        Attr::Fla, // 31
        Attr::Ldr, // 32
        Attr::Otb, // 33
        Attr::Pos, // 34
        Attr::Tea, // 35
        Attr::Wor, // 36
        Attr::Cmp, // 37
        Attr::Cnt, // 38
        Attr::Acc, // 39
        Attr::Agi, // 40
        Attr::Bal, // 41
        Attr::Pac, // 42
        Attr::Sta, // 43
        Attr::Str, // 44
        Attr::Jum, // 45
        Attr::Nat, // 46
    ];

    /// FM27 person position bits -> app `pos_r` key + expanded `pos` tags.
    pub const PERSON_POS_KEYS: &'static [(u64, &'static str, &'static [&'static str])] = &[
        (1, "gk", &["GK"]),
        (4, "dr", &["D_LR", "D_LR:R"]),
        (8, "dl", &["D_LR", "D_LR:L"]),
        (16, "dc", &["DC"]),
        (32, "wbr", &["WB_LR", "WB_LR:R"]),
        (64, "wbl", &["WB_LR", "WB_LR:L"]),
        (128, "dm", &["DM"]),
        (256, "mr", &["M_LR", "M_LR:R"]),
        (512, "ml", &["M_LR", "M_LR:L"]),
        (1_024, "mc", &["MC"]),
        (2_048, "amr", &["AM_LR", "AM_LR:R"]),
        (4_096, "aml", &["AM_LR", "AM_LR:L"]),
        (8_192, "amc", &["AMC"]),
        (16_384, "stc", &["ST"]),
    ];

    // ---- Recruitment: search, shortlist action, recruitment focus ----
    pub const SEARCH_TEXT: i64 = 1_851_878_757;
    pub const SEARCH_PEOPLE: i64 = 1_886_547_059;
    pub const SEARCH_UNIQUE_ID: i64 = 1_970_170_212;

    pub const ACTION_EV: i64 = 44;
    pub const ACTION_SHORTLIST_ADD: i64 = 1_097_488_485;
    pub const ACTION_SHORTLIST_REMOVE: i64 = 1_500_280_650;
    pub const ACTION_KEY_FLAG1: i64 = 1_853_189_474;
    pub const ACTION_KEY_STR_LIST: i64 = 2_004_054_900;
    pub const ACTION_KEY_MANAGER: i64 = 1_214_805_357;
    pub const ACTION_KEY_PERSON_LIST: i64 = 1_685_218_636;
    pub const ACTION_KEY_EMPTY_STR: i64 = 1_936_223_853;
    pub const ACTION_KEY_ZERO1: i64 = 1_954_115_685;
    pub const ACTION_KEY_ZERO2: i64 = 1_818_719_336;

    pub const FOCUS_NAME: i64 = 1_851_878_757;
    pub const FOCUS_AGE_MIN: i64 = 1_394_625_906;
    pub const FOCUS_AGE_MAX: i64 = 929_909_872;
    pub const FOCUS_CA: i64 = 1_346_584_898;
    pub const FOCUS_PA: i64 = 1_347_436_866;
    pub const FOCUS_LEVEL: i64 = 1_298_233_430;
    pub const FOCUS_CA_STEP: i64 = 1_517_250_124;
    pub const FOCUS_PA_STEP: i64 = 1_431_517_038;
    pub const FOCUS_AREA: i64 = 1_817_143_629;
    pub const FOCUS_ADD_AREA: i64 = 0;
    pub const FOCUS_AREAS: i64 = 1_986_161_206;
    pub const FOCUS_CONFIRM: i64 = 55;
    pub const FOCUS_LIST: i64 = 1_953_983_593;
    pub const FOCUS_REMOVE: i64 = 53;
    pub const FOCUS_ID: i64 = 1_919_314_276;
    /// FM NationReference id = fmsave `nation_id + 0x0A00_0000`.
    pub const FOCUS_NATION_BASE: i64 = 167_772_160;

    /// FM position bits (low 20 bits) -> app slot base.
    pub const POS_BITS: &'static [(u64, &'static str)] = &[
        (1, "GK"),
        (2, "SW"),
        (4, "DR"),
        (8, "DL"),
        (16, "DC"),
        (32, "WBR"),
        (64, "WBL"),
        (128, "DM"),
        (256, "MR"),
        (512, "ML"),
        (1_024, "MC"),
        (2_048, "AMR"),
        (4_096, "AML"),
        (8_192, "AMC"),
        (16_384, "ST"),
    ];

    // ---- Pair-string builders (no raw IDs outside `FmbIds`) ----

    pub fn tactic_pairs(c: BridgeContext) -> String {
        format!(
            "{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_TACTIC,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::PAIR_TACTIC_FLAG_KEY,
            Self::PAIR_TACTIC_FLAG_VAL,
            Self::KEY_PERSON_ID,
            c.team
        )
    }

    pub fn training_pairs(c: BridgeContext) -> String {
        format!(
            "{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_TRAINING,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::REF_TEAM_TRAINING_MANAGER,
            c.manager,
            Self::REF_TEAM_TRAINING_TEAM,
            c.team
        )
    }

    pub fn sp_pairs(c: BridgeContext) -> String {
        format!(
            "{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_SET_PIECE,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::REF_TEAM_TRAINING_MANAGER,
            c.manager,
            Self::KEY_PERSON_ID,
            c.team
        )
    }

    pub fn team_pairs(id: i64) -> String {
        format!(
            "{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_PERSON_OR_TEAM,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::KEY_PERSON_ID,
            id
        )
    }

    pub fn person_pairs(id: i64) -> String {
        Self::team_pairs(id)
    }

    pub fn search_pairs() -> String {
        format!(
            "{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_SEARCH,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::PAIR_SEARCH_FLAG1_KEY,
            Self::PAIR_SEARCH_FLAG1_VAL,
            Self::PAIR_SEARCH_FLAG2_KEY,
            Self::PAIR_SEARCH_FLAG2_VAL
        )
    }

    pub fn action_pairs(action_id: i64) -> String {
        format!(
            "{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_ACTION,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::PAIR_ACTION_KEY,
            action_id
        )
    }

    pub fn focus_pairs(c: BridgeContext, focus_id: i64) -> String {
        format!(
            "{}={};{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_FOCUS,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::REF_TEAM_TRAINING_MANAGER,
            c.manager,
            Self::PAIR_FOCUS_TEAM_KEY,
            c.team,
            Self::FOCUS_ID,
            focus_id
        )
    }

    pub fn focus_info_pairs(c: BridgeContext) -> String {
        format!(
            "{}={};{}={};{}={};{}={}",
            Self::PAIR_TYPE_KEY,
            Self::PAIR_TYPE_SCOUTING_INFO,
            Self::PAIR_SESSION_KEY,
            Self::PAIR_SESSION_DEFAULT,
            Self::REF_TEAM_TRAINING_MANAGER,
            c.manager,
            Self::PAIR_FOCUS_TEAM_KEY,
            c.team
        )
    }

    pub const fn focus_nation(nat_id: i64) -> i64 {
        Self::FOCUS_NATION_BASE + nat_id
    }

    // ---- Table lookups ----

    pub fn role_spec(role_id: &str) -> Option<&'static TacticRoleSpec> {
        Self::TACTIC_ROLES.iter().find(|r| r.id == role_id)
    }

    pub fn role_bits(role_id: &str) -> Option<u64> {
        Self::role_spec(role_id).map(|r| r.bits)
    }

    pub fn ti_spec(key: &str) -> Option<&'static TacticTiSpec> {
        Self::TACTIC_TIS.iter().find(|t| t.key == key)
    }

    pub fn pi_spec(name: &str) -> Option<&'static TacticPiSpec> {
        Self::TACTIC_PIS.iter().find(|p| p.name == name)
    }

    pub const fn attr_by_fm_id(id: i64) -> Option<Attr> {
        if id >= 0 && (id as usize) < Attr::COUNT {
            Some(Self::PERSON_ATTR_KEYS[id as usize])
        } else {
            None
        }
    }

    pub fn person_pos_by_bit(bit: u64) -> Option<(&'static str, &'static [&'static str])> {
        Self::PERSON_POS_KEYS
            .iter()
            .find(|(b, _, _)| *b == bit)
            .map(|&(_, k, tags)| (k, tags))
    }

    pub fn sp_role_prefs(attacking: bool, role_key: &str) -> Option<&'static [i32]> {
        let table = if attacking {
            Self::SP_ATT_ROLES
        } else {
            Self::SP_DEF_ROLES
        };
        table.iter().find(|(k, _)| *k == role_key).map(|&(_, ids)| ids)
    }

    pub fn sp_target_label(target: i32) -> &'static str {
        Self::SP_TARGET_LABELS
            .get(usize::try_from(target).unwrap_or(0))
            .copied()
            .unwrap_or("Mixed")
    }

    /// FM in-possession / out-of-possession position bits for an app slot code
    /// (`DCL = DC + TACTIC_LEFT`, `DCR = DC + TACTIC_RIGHT`, etc.).
    pub fn ip_bits(code: &str) -> Option<u64> {
        let (base, side_flag) = split_central_side(code);
        let base_bit = match base {
            "GK" => 1,
            "DR" => 4,
            "DL" => 8,
            "DC" => 16,
            "WBR" => 32,
            "WBL" => 64,
            "DM" => 128,
            "MR" => 256,
            "ML" => 512,
            "MC" => 1_024,
            "AMR" => 2_048,
            "AML" => 4_096,
            "AMC" => 8_192,
            "ST" => 16_384,
            _ => return None,
        };
        Some(base_bit + side_flag)
    }

    /// Map low 20 FM position bits to an app slot base code.
    pub fn position(bits: u64) -> &'static str {
        let low = bits & 0xF_FFFF;
        Self::POS_BITS
            .iter()
            .find(|(bit, _)| (low & bit) != 0)
            .map(|&(_, code)| code)
            .unwrap_or("MC")
    }
}

/// Options for [`FmbClient::apply_tactic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ApplyTacticOptions {
    /// When `true`, creates a new tactic slot in FM27 first (refusing at 3 tactics).
    pub create: bool,
}

// ============================================================================
// Bridge Protocol Client
// ============================================================================

/// High-level client for `TacticalBibleBridge` (`127.0.0.1:27029` line protocol).
pub struct FmbClient<T: BridgeTransport> {
    transport: T,
    lang: Lang,
    state: Option<BridgeStatus>,
    checked_at: Option<Instant>,
    status_throttle: Duration,
    routine_timeout: Duration,
    routine_max_polls: Option<usize>,
    recording: BridgeRecording,
}

impl<T: BridgeTransport> FmbClient<T> {
    pub const STATUS_THROTTLE_MS: u64 = 15_000;
    pub const ROUTINE_TIMEOUT_MS: u64 = 120_000;
    pub const DEFAULT_TIMEOUT_MS: u64 = 15_000;
    pub const PING_TIMEOUT_MS: u64 = 2_500;
    pub const READ_TIMEOUT_MS: u64 = 20_000;
    pub const READ_CHUNK_SIZE: usize = 60;

    pub fn new(transport: T, lang: Lang) -> Self {
        Self {
            transport,
            lang,
            state: None,
            checked_at: None,
            status_throttle: Duration::from_millis(Self::STATUS_THROTTLE_MS),
            routine_timeout: Duration::from_millis(Self::ROUTINE_TIMEOUT_MS),
            routine_max_polls: None,
            recording: BridgeRecording::default(),
        }
    }

    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    pub fn into_transport(self) -> T {
        self.transport
    }

    pub fn state(&self) -> Option<&BridgeStatus> {
        self.state.as_ref()
    }

    pub fn recording(&self) -> &BridgeRecording {
        &self.recording
    }

    pub fn with_routine_timeout_ms(mut self, ms: u64) -> Self {
        self.routine_timeout = Duration::from_millis(ms);
        self
    }

    pub fn with_routine_max_polls(mut self, max_polls: usize) -> Self {
        self.routine_max_polls = Some(max_polls);
        self
    }

    pub fn with_status_throttle_ms(mut self, ms: u64) -> Self {
        self.status_throttle = Duration::from_millis(ms);
        self
    }

    #[inline]
    fn l<'a>(&self, pt: &'a str, en: &'a str) -> &'a str {
        self.lang.pick(pt, en)
    }

    // ---- Low-level line protocol ----

    pub fn call(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, BridgeError> {
        self.transport.call(cmd, timeout_ms)
    }

    pub fn ok(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, BridgeError> {
        let r = self.call(cmd, timeout_ms)?;
        if !r.starts_with("OK") {
            return Err(BridgeError::Protocol(if r.is_empty() {
                "no reply".to_string()
            } else {
                r
            }));
        }
        Ok(if r.len() > 3 {
            r[3..].to_string()
        } else {
            String::new()
        })
    }

    /// Check bridge status (`PING` + `VERSION`), throttled to 15s unless `force` is true.
    pub fn status(&mut self, force: bool) -> BridgeStatus {
        if !force {
            if let (Some(ts), Some(st)) = (self.checked_at, self.state.as_ref()) {
                if ts.elapsed() < self.status_throttle {
                    return st.clone();
                }
            }
        }
        let st = self.fetch_status();
        self.state = Some(st.clone());
        self.checked_at = Some(Instant::now());
        st
    }

    fn fetch_status(&mut self) -> BridgeStatus {
        match self.call("PING", Self::PING_TIMEOUT_MS) {
            Ok(p) if p.starts_with("PONG") => {
                let mut version = String::new();
                let mut fm = String::new();
                if let Ok(v) = self.ok("VERSION", Self::PING_TIMEOUT_MS) {
                    let mut parts = v.split(' ');
                    if let Some(a) = parts.next() {
                        version = a.to_string();
                    }
                    if let Some(b) = parts.next() {
                        fm = b.to_string();
                    }
                }
                BridgeStatus {
                    ok: true,
                    ready: p.contains("ready"),
                    version,
                    fm,
                    err: None,
                }
            }
            Ok(other) => BridgeStatus {
                ok: false,
                ready: false,
                version: String::new(),
                fm: String::new(),
                err: Some(if other.is_empty() {
                    "no reply".to_string()
                } else {
                    other
                }),
            },
            Err(e) => BridgeStatus {
                ok: false,
                ready: false,
                version: String::new(),
                fm: String::new(),
                err: Some(e.to_string()),
            },
        }
    }

    /// Batched `READ` of `(ref_type_short, pairs, prop_id)` tuples (chunks of 60).
    pub fn read(&mut self, reqs: &[(&str, &str, i64)]) -> Result<Vec<Option<Value>>, BridgeError> {
        self.read_at(reqs, None)
    }

    /// Batched `READ` or `READAT <at>` (use `Some("auto")` for screen-scoped properties
    /// such as FM27 set-piece routines on plugin >= 0.3).
    pub fn read_at(
        &mut self,
        reqs: &[(&str, &str, i64)],
        at: Option<&str>,
    ) -> Result<Vec<Option<Value>>, BridgeError> {
        let mut out = Vec::with_capacity(reqs.len());
        for chunk in reqs.chunks(Self::READ_CHUNK_SIZE) {
            let prefix = match at {
                Some(a) => format!("READAT {a} "),
                None => "READ ".to_string(),
            };
            let body = chunk
                .iter()
                .map(|(ty, pairs, prop)| format!("FM.UI.{ty}|{pairs}|{prop}"))
                .collect::<Vec<_>>()
                .join(";;");
            let r = self.ok(&format!("{prefix}{body}"), Self::READ_TIMEOUT_MS)?;
            let parts: Vec<&str> = r.split('\x1e').collect();
            for j in 0..chunk.len() {
                let p = parts.get(j).copied().unwrap_or("ERR");
                if p.starts_with("ERR") {
                    out.push(None);
                    continue;
                }
                let cleaned: String = p
                    .chars()
                    .map(|ch| if (ch as u32) <= 0x1F { ' ' } else { ch })
                    .collect();
                match serde_json::from_str::<Value>(&cleaned) {
                    Ok(v) => out.push(Some(v)),
                    Err(_) => out.push(None),
                }
            }
        }
        Ok(out)
    }

    /// `WRITE FM.UI.<ref_type>|<pairs>|<prop> <json>`.
    pub fn write(
        &mut self,
        ref_type: &str,
        pairs: &str,
        prop: i64,
        val: &Value,
    ) -> Result<String, BridgeError> {
        let encoded =
            serde_json::to_string(val).map_err(|e| BridgeError::Parse(e.to_string()))?;
        self.ok(
            &format!("WRITE FM.UI.{ref_type}|{pairs}|{prop} {encoded}"),
            Self::DEFAULT_TIMEOUT_MS,
        )
    }

    /// `SENDJ <ev> FM.UI.<ref_type> <pairs> <json>`.
    pub fn send(
        &mut self,
        ev: i64,
        ref_type: &str,
        pairs: &str,
        rec: &Value,
    ) -> Result<String, BridgeError> {
        let encoded =
            serde_json::to_string(rec).map_err(|e| BridgeError::Parse(e.to_string()))?;
        self.ok(
            &format!("SENDJ {ev} FM.UI.{ref_type} {pairs} {encoded}"),
            Self::DEFAULT_TIMEOUT_MS,
        )
    }

    // ---- Live event recorder ----

    /// Start recording FM27 UI events (`RECORD 1` + flush `EVENTS`).
    pub fn rec_start(&mut self) -> Result<(), BridgeError> {
        self.ok("RECORD 1", Self::DEFAULT_TIMEOUT_MS)?;
        self.ok("EVENTS", Self::DEFAULT_TIMEOUT_MS)?;
        self.recording = BridgeRecording {
            on: true,
            out: String::new(),
        };
        Ok(())
    }

    /// Collect recorded events via `EVENTS`, store formatted JSON in `self.recording`,
    /// and return the captured event count.
    pub fn rec_stop(&mut self) -> Result<usize, BridgeError> {
        let r = self.ok("EVENTS", Self::DEFAULT_TIMEOUT_MS)?;
        let evs: Vec<Value> = if r.is_empty() {
            Vec::new()
        } else {
            r.split('\x1e')
                .filter(|s| !s.is_empty())
                .map(|x| serde_json::from_str(x).unwrap_or_else(|_| Value::String(x.to_string())))
                .collect()
        };
        let count = evs.len();
        let payload = json!({
            "fm": self.state.as_ref().map(|s| s.fm.as_str()),
            "plugin": self.state.as_ref().map(|s| s.version.as_str()),
            "events": evs,
        });
        let out = serde_json::to_string_pretty(&payload)
            .map_err(|e| BridgeError::Parse(e.to_string()))?;
        self.recording = BridgeRecording { on: false, out };
        Ok(count)
    }

    // ---- JSON helpers over plugin format ----

    pub fn field(rec: Option<&Value>, key: i64) -> Option<&Value> {
        let entries = rec?.get("entries")?.as_array()?;
        entries
            .iter()
            .find(|x| x.as_array().and_then(|a| a.first()).and_then(Value::as_i64) == Some(key))
            .and_then(|x| x.as_array())
            .and_then(|a| a.get(2))
    }

    pub fn num_i64(val: Option<&Value>) -> Option<i64> {
        let v = val?;
        if let Some(i) = v.as_i64() {
            return Some(i);
        }
        if let Some(u) = v.as_u64() {
            return i64::try_from(u).ok();
        }
        if let Some(f) = v.as_f64() {
            return f.is_finite().then_some(f.round() as i64);
        }
        if let Some(s) = v.as_str() {
            let t = s.trim();
            if let Ok(i) = t.parse::<i64>() {
                return Some(i);
            }
            if let Ok(f) = t.parse::<f64>() {
                if f.is_finite() {
                    return Some(f.round() as i64);
                }
            }
        }
        None
    }

    pub fn num_i32(val: Option<&Value>, default: i32) -> i32 {
        Self::num_i64(val)
            .and_then(|n| i32::try_from(n).ok())
            .unwrap_or(default)
    }

    pub fn num_u64(val: Option<&Value>) -> Option<u64> {
        let v = val?;
        if let Some(u) = v.as_u64() {
            return Some(u);
        }
        if let Some(i) = v.as_i64() {
            return u64::try_from(i).ok();
        }
        if let Some(s) = v.as_str() {
            let t = s.trim();
            if let Ok(u) = t.parse::<u64>() {
                return Some(u);
            }
            if let Ok(f) = t.parse::<f64>() {
                if f.is_finite() && f >= 0.0 {
                    return Some(f.round() as u64);
                }
            }
        }
        if let Some(f) = v.as_f64() {
            if f.is_finite() && f >= 0.0 {
                return Some(f.round() as u64);
            }
        }
        None
    }

    pub fn pair_opt(ref_val: Option<&Value>, key: i64) -> Option<i64> {
        let pairs = ref_val?.get("pairs")?.as_array()?;
        pairs
            .iter()
            .find(|p| p.as_array().and_then(|a| a.first()).and_then(Value::as_i64) == Some(key))
            .and_then(|p| p.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| Self::num_i64(Some(v)))
    }

    pub fn pair(ref_val: Option<&Value>, key: i64) -> i64 {
        Self::pair_opt(ref_val, key).unwrap_or(0)
    }

    pub fn items(list: Option<&Value>) -> Vec<&Value> {
        list.and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| {
                        let pair = x.as_array()?;
                        (pair.len() == 2).then_some(&pair[1])
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// FM display strings come as `"\u{0001}<formatting>\u{0002}<text>\u{0001}"`: keep the text.
    pub fn text(val: Option<&Value>) -> String {
        val.and_then(Value::as_str)
            .map(|s| {
                s.rsplit('\x02')
                    .next()
                    .unwrap_or("")
                    .replace('\x01', "")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default()
    }

    // ---- Context & Active Tactic ----

    /// Discover `(manager, team)` IDs from `REFS` (`TeamTrainingReference` or `CalendarReference`).
    pub fn context(&mut self) -> Result<BridgeContext, BridgeError> {
        let refs = self.ok("REFS", Self::DEFAULT_TIMEOUT_MS)?;
        let mut map: HashMap<String, HashMap<i64, i64>> = HashMap::new();
        for raw_part in refs.split(" | ") {
            let mut part = raw_part.trim();
            if part.starts_with("session=") {
                if let Some(sp) = part.find(' ') {
                    part = part[sp + 1..].trim();
                }
            }
            let Some(eq) = part.find('=') else {
                continue;
            };
            let ty = part[..eq].trim();
            let mut kv_map = HashMap::new();
            for kv in part[eq + 1..].split(';') {
                if let Some((k, v)) = kv.split_once('=') {
                    if let (Ok(ki), Ok(vi)) = (k.trim().parse::<i64>(), v.trim().parse::<i64>()) {
                        kv_map.insert(ki, vi);
                    }
                }
            }
            map.insert(ty.to_string(), kv_map);
        }
        if let Some(tt) = map.get(FmbIds::REF_TEAM_TRAINING) {
            if let Some(&team) = tt.get(&FmbIds::REF_TEAM_TRAINING_TEAM) {
                if team != 0 {
                    let manager = tt.get(&FmbIds::REF_TEAM_TRAINING_MANAGER).copied().unwrap_or(0);
                    return Ok(BridgeContext { manager, team });
                }
            }
        }
        if let Some(cal) = map.get(FmbIds::REF_CALENDAR) {
            if let Some(&team) = cal.get(&FmbIds::REF_CALENDAR_TEAM) {
                if team != 0 {
                    let manager = cal.get(&FmbIds::REF_CALENDAR_MANAGER).copied().unwrap_or(0);
                    return Ok(BridgeContext { manager, team });
                }
            }
        }
        Err(BridgeError::State(
            self.l(
                "Jogo ainda não reconhecido: abra uma vez o ecrã Treino no FM.",
                "Game not recognised yet: open the Training screen in FM once.",
            )
            .to_string(),
        ))
    }

    /// Read active tactic in FM27 (name, 11 slots + persons, selection order index).
    pub fn read_tactic(&mut self, c: BridgeContext) -> Result<BridgeTacticState, BridgeError> {
        let pairs = FmbIds::tactic_pairs(c);
        let res = self.read(&[
            ("TeamTacticReference", &pairs, FmbIds::PROP_NAME),
            ("TeamTacticReference", &pairs, FmbIds::PROP_SLOTS),
            ("TeamTacticReference", &pairs, FmbIds::PROP_SELECTION),
        ])?;
        let name = res
            .first()
            .and_then(|v| v.as_ref())
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let mut slots = Vec::new();
        for r in Self::items(res.get(1).and_then(|v| v.as_ref())) {
            let index = Self::num_i32(Self::field(Some(r), FmbIds::KEY_SLOT), -1);
            if index < 0 {
                continue;
            }
            let pos_bits = Self::num_u64(Self::field(Some(r), FmbIds::KEY_POS)).unwrap_or(0);
            let person = Self::pair(Self::field(Some(r), FmbIds::KEY_PERSON), FmbIds::KEY_PERSON_ID);
            slots.push(BridgeSlot {
                index,
                pos: FmbIds::position(pos_bits).to_string(),
                person,
            });
        }
        let mut sel_index = HashMap::new();
        for r in Self::items(res.get(2).and_then(|v| v.as_ref())) {
            let person = Self::pair(Self::field(Some(r), FmbIds::KEY_PERSON), FmbIds::KEY_PERSON_ID);
            let s = Self::num_i32(Self::field(Some(r), FmbIds::KEY_SEL), -1);
            if person != 0 && s >= 0 {
                sel_index.insert(person, s);
            }
        }
        Ok(BridgeTacticState {
            name,
            slots,
            sel_index,
        })
    }

    /// Map `fmsave unique_id (string)` -> FM runtime `person_id`.
    pub fn persons_of(&mut self, c: BridgeContext) -> Result<HashMap<String, i64>, BridgeError> {
        let tp = FmbIds::training_pairs(c);
        let list = self.read(&[(
            "TeamTrainingReference",
            &tp,
            FmbIds::PROP_TRAINING_PLAYERS,
        )])?;
        let mut seen = HashSet::new();
        let mut persons = Vec::new();
        for x in Self::items(list.first().and_then(|v| v.as_ref())) {
            let pid = Self::pair(Some(x), FmbIds::KEY_PERSON_ID);
            if pid != 0 && seen.insert(pid) {
                persons.push(pid);
            }
        }
        if persons.is_empty() {
            return Ok(HashMap::new());
        }
        let pair_strs: Vec<String> = persons.iter().map(|&p| FmbIds::person_pairs(p)).collect();
        let reqs: Vec<(&str, &str, i64)> = pair_strs
            .iter()
            .map(|ps| ("PersonReference", ps.as_str(), FmbIds::PERSON_UNIQUE_ID))
            .collect();
        let vals = self.read(&reqs)?;
        let mut map = HashMap::new();
        for (&p, val) in persons.iter().zip(vals.iter()) {
            let uid = Self::num_i64(val.as_ref()).unwrap_or(0);
            if uid > 0 {
                map.insert(uid.to_string(), p);
            }
        }
        Ok(map)
    }

    // ---- Place XI ----

    /// Place the app's XI into FM's active tactic, matching by position.
    pub fn apply_xi(
        &mut self,
        tactic: &Tactic,
        squad: &[Player],
    ) -> Result<BridgeXiResult, BridgeError> {
        let c = self.context()?;
        let pairs = FmbIds::tactic_pairs(c);
        let person_of = self.persons_of(c)?;
        let st = self.read_tactic(c)?;

        let mut by_base: BTreeMap<String, Vec<&SlotAssignment>> = BTreeMap::new();
        for s in &tactic.slots {
            if s.pid.is_none() {
                continue;
            }
            by_base
                .entry(slot_base_code(&s.ip_code).to_string())
                .or_default()
                .push(s);
        }
        let mut fm_by_base: BTreeMap<String, Vec<&BridgeSlot>> = BTreeMap::new();
        for s in &st.slots {
            fm_by_base.entry(s.pos.clone()).or_default().push(s);
        }

        let mut wanted: BTreeMap<i32, i64> = BTreeMap::new();
        let mut skipped: Vec<String> = Vec::new();
        for (b, mut mine) in by_base {
            mine.sort_by_key(|s| slot_lateral_order(&s.ip_code));
            let mut theirs = fm_by_base.remove(&b).unwrap_or_default();
            theirs.sort_by_key(|s| s.index);
            for (i, s) in mine.into_iter().enumerate() {
                let pid = s.pid.as_deref().unwrap_or("");
                let player = squad.iter().find(|x| x.id == pid);
                let label = player.map(|p| p.name.as_str()).unwrap_or(pid);
                let Some(fm_slot) = theirs.get(i) else {
                    skipped.push(format!(
                        "{label}: {}",
                        self.lang.pick(
                            &format!("sem casa {b} na tática do FM"),
                            &format!("no {b} slot in the FM tactic"),
                        )
                    ));
                    continue;
                };
                let person = player
                    .and_then(|p| p.meta.uid.as_deref())
                    .and_then(|uid| person_of.get(uid).copied());
                let Some(person) = person else {
                    skipped.push(format!(
                        "{label}: {}",
                        self.l("não encontrado no FM (uid)", "not found in FM (uid)")
                    ));
                    continue;
                };
                wanted.insert(fm_slot.index, person);
            }
        }

        let mut placed = 0usize;
        for (idx, person) in wanted {
            let now = if placed == 0 {
                st.clone()
            } else {
                self.read_tactic(c)?
            };
            let Some(slot) = now.slots.iter().find(|x| x.index == idx) else {
                continue;
            };
            if slot.person == person {
                continue;
            }
            let sel = now.sel_index.get(&person).copied().unwrap_or(-1);
            let rec = dyn_ref(vec![
                dyn_num(FmbIds::KEY_NUMBER, 2),
                dyn_entry(
                    FmbIds::KEY_PAYLOAD,
                    "SI.Bindable.DynamicReference",
                    dyn_ref(vec![
                        dyn_entry(FmbIds::KEY_PERSON, FmbIds::REF_PERSON, person_ref(person)),
                        dyn_num(FmbIds::KEY_SEL_IDX, sel),
                        dyn_num(FmbIds::KEY_SEL, person & 0xFF_FFFF),
                        dyn_num(FmbIds::KEY_SLOT, idx),
                    ]),
                ),
            ]);
            match self.send(FmbIds::EV_PLACE, "TeamTacticReference", &pairs, &rec) {
                Ok(_) => {
                    placed += 1;
                    self.transport.sleep_ms(250);
                }
                Err(e) => skipped.push(format!("#{idx}: {e}")),
            }
        }
        Ok(BridgeXiResult { placed, skipped })
    }

    // ---- Tactic slots, tactic list, create tactic, apply full tactic ----

    /// Read IP/OOP positions and 64-bit IP/OOP role masks for all 11 FM slots.
    pub fn read_tactic_slots(
        &mut self,
        c: BridgeContext,
    ) -> Result<Vec<BridgeTacticSlotInfo>, BridgeError> {
        let p = FmbIds::tactic_pairs(c);
        let res = self.read(&[
            ("TeamTacticReference", &p, FmbIds::TACTIC_SLOTS),
            ("TeamTacticReference", &p, FmbIds::TACTIC_IP_ROLES),
            ("TeamTacticReference", &p, FmbIds::TACTIC_OOP_ROLES),
        ])?;
        let mut ip_r: HashMap<i32, u64> = HashMap::new();
        let mut oop_r: HashMap<i32, u64> = HashMap::new();
        for r in Self::items(res.get(1).and_then(|v| v.as_ref())) {
            let slot = Self::num_i32(Self::field(Some(r), FmbIds::TACTIC_SLOT_IDX), -1);
            if let Some(bits) = Self::num_u64(Self::field(Some(r), FmbIds::TACTIC_IP_ROLE)) {
                ip_r.insert(slot, bits);
            }
        }
        for r in Self::items(res.get(2).and_then(|v| v.as_ref())) {
            let slot = Self::num_i32(Self::field(Some(r), FmbIds::TACTIC_SLOT_IDX), -1);
            if let Some(raw) = Self::num_u64(Self::field(Some(r), FmbIds::TACTIC_OOP_ROLE)) {
                oop_r.insert(slot, raw & !FmbIds::TACTIC_OOP_ROLE_SIDE_MASK);
            }
        }
        let mut out = Vec::new();
        for r in Self::items(res.first().and_then(|v| v.as_ref())) {
            let slot = Self::num_i32(Self::field(Some(r), FmbIds::TACTIC_SLOT_IDX), -1);
            let ip = Self::num_u64(Self::field(Some(r), FmbIds::TACTIC_IP_POS)).unwrap_or(0);
            let oop = Self::num_u64(Self::field(Some(r), FmbIds::TACTIC_OOP_POS)).unwrap_or(0);
            out.push(BridgeTacticSlotInfo {
                slot,
                ip,
                oop,
                ip_role: ip_r.get(&slot).copied(),
                oop_role: oop_r.get(&slot).copied(),
            });
        }
        Ok(out)
    }

    /// Read FM's live tactic names from the tactic menu's "Copy Tactic" submenu.
    pub fn tactic_list(&mut self, c: BridgeContext) -> Result<Vec<String>, BridgeError> {
        let p = FmbIds::tactic_pairs(c);
        let res = self.read(&[("TeamTacticReference", &p, FmbIds::TACTIC_MENU)])?;
        let menu = res.first().and_then(|v| v.as_ref());
        let copy = Self::items(menu).into_iter().find(|m| {
            let inner = Self::field(Some(m), FmbIds::TACTIC_MENU_ITEM);
            Self::num_i32(Self::field(inner, FmbIds::KEY_NUMBER), -1) == FmbIds::TACTIC_MENU_COPY
        });
        let sub = Self::field(copy, FmbIds::TACTIC_MENU_SUB);
        Ok(Self::items(sub)
            .into_iter()
            .map(|x| {
                Self::field(Some(x), FmbIds::KEY_PAYLOAD)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            })
            .collect())
    }

    /// Create a NEW tactic in FM27 by replaying the Clean Slate wizard (no event 90!).
    /// Refuses when FM already has 3 tactics (`FmbIds::TACTIC_MAX_TACTICS`).
    pub fn create_tactic(&mut self, mut say: impl FnMut(&str)) -> Result<String, BridgeError> {
        let c = self.context()?;
        let p = FmbIds::tactic_pairs(c);
        let have = self.tactic_list(c)?;
        if have.len() >= FmbIds::TACTIC_MAX_TACTICS {
            return Err(BridgeError::State(
                self.lang
                    .pick(
                        &format!(
                            "O FM já tem {} táticas (o máximo): apague uma no FM primeiro, ou use “Aplicar no FM”, que escreve na tática activa.",
                            have.len()
                        ),
                        &format!(
                            "FM already has {} tactics (the maximum): delete one in FM first, or use “Apply in FM”, which writes into the active tactic.",
                            have.len()
                        ),
                    )
                    .to_string(),
            ));
        }
        say(self.l("A criar uma tática nova", "Creating a new tactic"));
        self.send(FmbIds::TACTIC_WIZARD_OPEN, "TeamTacticReference", &p, &Value::Null)?;
        self.transport.sleep_ms(600);
        let style_val = json!(["SI.Core.DynamicNumber", FmbIds::TACTIC_WIZARD_STYLE_BASE.to_string()]);
        self.write("TeamTacticReference", &p, FmbIds::TACTIC_WIZARD_STYLE, &style_val)?;
        self.transport.sleep_ms(3_500);
        self.send(FmbIds::TACTIC_WIZARD_NEXT, "TeamTacticReference", &p, &Value::Null)?;
        self.transport.sleep_ms(400);
        self.send(FmbIds::TACTIC_WIZARD_NEXT, "TeamTacticReference", &p, &Value::Null)?;
        self.transport.sleep_ms(400);
        self.send(FmbIds::TACTIC_WIZARD_FINISH, "TeamTacticReference", &p, &Value::Null)?;
        self.transport.sleep_ms(1_500);
        let now = self.tactic_list(c)?;
        if now.len() != have.len() + 1 {
            return Err(BridgeError::State(
                self.l(
                    "O FM não criou a tática nova.",
                    "FM did not create the new tactic.",
                )
                .to_string(),
            ));
        }
        let nm = self.read(&[("TeamTacticReference", &p, FmbIds::TACTIC_NAME)])?;
        Ok(nm
            .first()
            .and_then(|v| v.as_ref())
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string())
    }

    /// Apply a full tactic to FM27's active tactic (or a newly created one if `opts.create`):
    /// IP formation, OOP formation (with cycle breaking), XI, 64-bit IP/OOP roles,
    /// player instructions, mentality, and 27 team instructions.
    pub fn apply_tactic(
        &mut self,
        t: &Tactic,
        squad: &[Player],
        opts: ApplyTacticOptions,
        mut say: impl FnMut(&str),
    ) -> Result<Vec<String>, BridgeError> {
        let st = self.status(true);
        if !version_at_least(&st.version, 0, 4, 2) {
            return Err(BridgeError::State(
                self.l(
                    "Aplicar a tática precisa do Tactical Bible Bridge 0.4.2 ou mais recente (Definições › Integrações).",
                    "Applying the tactic needs Tactical Bible Bridge 0.4.2 or newer (Settings › Integrations).",
                )
                .to_string(),
            ));
        }
        let c = self.context()?;
        let p = FmbIds::tactic_pairs(c);
        let mut done: Vec<String> = Vec::new();
        if opts.create {
            let nm = self.create_tactic(&mut say)?;
            done.push(
                self.lang
                    .pick(
                        &format!("Tática nova no FM: {nm}"),
                        &format!("New tactic in FM: {nm}"),
                    )
                    .to_string(),
            );
        }

        // 1. In-possession formation
        say(self.l("Formação", "Formation"));
        let mut want: Vec<u64> = Vec::with_capacity(t.slots.len());
        for s in &t.slots {
            match FmbIds::ip_bits(&s.ip_code) {
                Some(b) => want.push(b),
                None => {
                    return Err(BridgeError::State(
                        self.l(
                            "Posição da formação sem equivalente no FM.",
                            "Formation position with no FM equivalent.",
                        )
                        .to_string(),
                    ));
                }
            }
        }
        let cur = self.read_tactic_slots(c)?;
        let mut free = want;
        let mut movers: Vec<BridgeTacticSlotInfo> = Vec::new();
        for s in cur {
            if let Some(pos_idx) = free.iter().position(|&b| b == s.ip) {
                free.remove(pos_idx);
            } else {
                movers.push(s);
            }
        }
        for (i, s) in movers.iter().enumerate() {
            let Some(&to) = free.get(i) else {
                break;
            };
            let rec = move_record(s.slot, s.ip, to, true);
            self.send(FmbIds::TACTIC_MOVE, "TeamTacticReference", &p, &rec)?;
            self.transport.sleep_ms(150);
        }
        done.push(
            self.lang
                .pick(
                    &format!(
                        "Formação com bola: {} ({} posições mudadas)",
                        t.ip_form,
                        movers.len()
                    ),
                    &format!(
                        "In-possession formation: {} ({} positions moved)",
                        t.ip_form,
                        movers.len()
                    ),
                )
                .to_string(),
        );

        // 1b. Out-of-possession formation (with cycle-breaking through spare slots)
        let cur = self.read_tactic_slots(c)?;
        let mut want_oop: BTreeMap<i32, u64> = BTreeMap::new();
        let mut used_s: HashSet<i32> = HashSet::new();
        for s in &t.slots {
            let Some(ip_b) = FmbIds::ip_bits(&s.ip_code) else {
                continue;
            };
            if let Some(fm) = cur.iter().find(|x| x.ip == ip_b && !used_s.contains(&x.slot)) {
                used_s.insert(fm.slot);
                let Some(oop_b) = FmbIds::ip_bits(&s.oop_code) else {
                    return Err(BridgeError::State(
                        self.l(
                            "Posição sem bola sem equivalente no FM.",
                            "Out-of-possession position with no FM equivalent.",
                        )
                        .to_string(),
                    ));
                };
                want_oop.insert(fm.slot, oop_b);
            }
        }
        let mut pos: BTreeMap<i32, u64> = cur.iter().map(|x| (x.slot, x.oop)).collect();
        let spare: Vec<u64> = FmbIds::TACTIC_OOP_SPARE_POSITIONS
            .iter()
            .copied()
            .filter(|b| !want_oop.values().any(|&w| w == *b))
            .collect();
        let mut moved_oop = 0usize;
        for _guard in 0..40 {
            let todo: Vec<i32> = want_oop
                .iter()
                .filter_map(|(&slot, &target)| {
                    (pos.get(&slot).copied() != Some(target)).then_some(slot)
                })
                .collect();
            if todo.is_empty() {
                break;
            }
            if let Some(&s) = todo.iter().find(|&&slot| {
                let target = want_oop[&slot];
                !pos.values().any(|&cur_p| cur_p == target)
            }) {
                let from = pos[&s];
                let to = want_oop[&s];
                let rec = move_record(s, from, to, false);
                self.send(FmbIds::TACTIC_MOVE, "TeamTacticReference", &p, &rec)?;
                pos.insert(s, to);
                moved_oop += 1;
            } else {
                let x = todo[0];
                let Some(&tmp) = spare
                    .iter()
                    .find(|&&b| !pos.values().any(|&cur_p| cur_p == b))
                else {
                    break;
                };
                let from = pos[&x];
                let rec = move_record(x, from, tmp, false);
                self.send(FmbIds::TACTIC_MOVE, "TeamTacticReference", &p, &rec)?;
                pos.insert(x, tmp);
            }
            self.transport.sleep_ms(150);
        }
        let after_oop = self.read_tactic_slots(c)?;
        let ok_oop_pos = after_oop
            .iter()
            .filter(|x| want_oop.get(&x.slot).copied() == Some(x.oop))
            .count();
        done.push(
            self.lang
                .pick(
                    &format!(
                        "Formação sem bola: {} ({moved_oop} mudadas, {ok_oop_pos}/11 confirmadas)",
                        t.oop_form
                    ),
                    &format!(
                        "Out-of-possession formation: {} ({moved_oop} moved, {ok_oop_pos}/11 confirmed)",
                        t.oop_form
                    ),
                )
                .to_string(),
        );

        // 2. XI
        say("XI");
        let xi = self.apply_xi(t, squad)?;
        let mut xi_line = self
            .lang
            .pick(
                &format!("XI: {} trocas", xi.placed),
                &format!("XI: {} swaps", xi.placed),
            )
            .to_string();
        if !xi.skipped.is_empty() {
            let preview: Vec<&str> = xi.skipped.iter().take(3).map(String::as_str).collect();
            xi_line.push_str(" — ");
            xi_line.push_str(&preview.join("; "));
        }
        done.push(xi_line);

        // 3. Roles (64-bit IP & OOP bitmasks)
        say(self.l("Papéis", "Roles"));
        let cur = self.read_tactic_slots(c)?;
        let mut used: HashSet<i32> = HashSet::new();
        let mut missing: Vec<String> = Vec::new();
        let mut fm_slot_of: Vec<(&SlotAssignment, i32)> = Vec::new();
        for s in &t.slots {
            let Some(bits) = FmbIds::ip_bits(&s.ip_code) else {
                continue;
            };
            let Some(fm) = cur.iter().find(|x| x.ip == bits && !used.contains(&x.slot)) else {
                continue;
            };
            used.insert(fm.slot);
            fm_slot_of.push((s, fm.slot));
            for (rid, ip) in [(&s.ip, true), (&s.oop, false)] {
                if rid.is_empty() {
                    continue;
                }
                let Some(spec) = FmbIds::role_spec(rid) else {
                    if !missing.iter().any(|m| m == rid) {
                        missing.push(rid.clone());
                    }
                    continue;
                };
                let rec = set_role_record(fm.slot, spec.bits, spec.en_name, ip);
                self.send(FmbIds::TACTIC_SET_ROLE, "TeamTacticReference", &p, &rec)?;
                self.transport.sleep_ms(80);
            }
        }
        let after = self.read_tactic_slots(c)?;
        let mut ok_ip = 0usize;
        let mut ok_oop = 0usize;
        let mut n_slots = 0usize;
        for s in &t.slots {
            let Some(bits) = FmbIds::ip_bits(&s.ip_code) else {
                continue;
            };
            let Some(fm) = after.iter().find(|x| x.ip == bits) else {
                continue;
            };
            n_slots += 1;
            if fm.ip_role.is_some() && fm.ip_role == FmbIds::role_bits(&s.ip) {
                ok_ip += 1;
            }
            if fm.oop_role.is_some() && fm.oop_role == FmbIds::role_bits(&s.oop) {
                ok_oop += 1;
            }
        }
        let mut role_line = self
            .lang
            .pick(
                &format!("Papéis: {ok_ip}/{n_slots} com bola e {ok_oop}/{n_slots} sem bola confirmados no FM"),
                &format!("Roles: {ok_ip}/{n_slots} in possession and {ok_oop}/{n_slots} out of possession confirmed in FM"),
            )
            .to_string();
        if !missing.is_empty() {
            role_line.push_str(self.l(" · sem equivalente: ", " · no FM equivalent: "));
            role_line.push_str(&missing.join(", "));
        }
        done.push(role_line);

        // 3b. Player instructions (revert to role defaults, then ‹ presses from Standard)
        say(self.l("Instruções individuais", "Player instructions"));
        let mut n_pi = 0usize;
        let mut no_pi: BTreeSet<String> = BTreeSet::new();
        if FmbIds::TACTIC_PI_ENABLED {
            for &(s, slot) in &fm_slot_of {
                if s.pis.is_empty() {
                    continue;
                }
                let rev_ip = pi_revert_record(slot, true);
                self.send(FmbIds::TACTIC_PI_REVERT, "TeamTacticReference", &p, &rev_ip)?;
                self.transport.sleep_ms(150);
                let rev_oop = pi_revert_record(slot, false);
                self.send(FmbIds::TACTIC_PI_REVERT, "TeamTacticReference", &p, &rev_oop)?;
                self.transport.sleep_ms(250);
                for pi_name in &s.pis {
                    let Some(spec) = FmbIds::pi_spec(pi_name) else {
                        no_pi.insert(pi_name.clone());
                        continue;
                    };
                    let press_rec = pi_press_record(slot, spec.category, spec.in_possession);
                    for _ in 0..spec.prev_presses {
                        self.send(FmbIds::TACTIC_PI_PREV, "TeamTacticReference", &p, &press_rec)?;
                        self.transport.sleep_ms(150);
                    }
                    n_pi += 1;
                }
            }
            let mut pi_line = self
                .lang
                .pick(
                    &format!("Instruções individuais: {n_pi} aplicadas (o FM ignora as que o papel bloqueia)"),
                    &format!("Player instructions: {n_pi} applied (FM ignores the ones a role locks)"),
                )
                .to_string();
            if !no_pi.is_empty() {
                pi_line.push_str(self.l(" · sem equivalente no FM: ", " · no FM equivalent: "));
                let list: Vec<&str> = no_pi.iter().map(String::as_str).collect();
                pi_line.push_str(&list.join(", "));
            }
            done.push(pi_line);
        }

        // 4. Mentality
        let mi = t.ment.index();
        let ment_val = dyn_ref(vec![
            dyn_num(FmbIds::KEY_NUMBER, mi + 1),
            dyn_str(FmbIds::KEY_PAYLOAD, t.ment.as_str()),
        ]);
        self.write("TeamTacticReference", &p, FmbIds::TACTIC_MENTALITY, &ment_val)?;
        done.push(format!(
            "{}{}",
            self.l("Mentalidade: ", "Mentality: "),
            t.ment.as_str()
        ));

        // 5. Team instructions (‹ to first option, then › to target option, 150ms spacing)
        say(self.l("Instruções de equipa", "Team instructions"));
        let mut n_ti = 0usize;
        let mut no_ti: Vec<&'static str> = Vec::new();
        for spec in FmbIds::TACTIC_TIS {
            let v = t.ti.get(spec.key).copied().unwrap_or(spec.default_opt);
            let target = spec.map.get(v).copied().flatten();
            let Some(target_idx) = target else {
                no_ti.push(spec.name);
                continue;
            };
            let id_val = json!(["SI.Core.DynamicNumber", spec.fm_id.to_string()]);
            for _ in 0..spec.fm_count {
                self.send(FmbIds::TACTIC_TI_PREV, "TeamTacticReference", &p, &id_val)?;
                self.transport.sleep_ms(150);
            }
            for _ in 0..target_idx {
                self.send(FmbIds::TACTIC_TI_NEXT, "TeamTacticReference", &p, &id_val)?;
                self.transport.sleep_ms(150);
            }
            n_ti += 1;
        }
        let mut ti_line = self
            .lang
            .pick(
                &format!("Instruções de equipa: {n_ti} aplicadas"),
                &format!("Team instructions: {n_ti} applied"),
            )
            .to_string();
        if !no_ti.is_empty() {
            ti_line.push_str(self.l(" · sem opção igual no FM: ", " · no matching FM option: "));
            ti_line.push_str(&no_ti.join(", "));
        }
        done.push(ti_line);

        Ok(done)
    }

    // ---- Set Pieces ----

    /// Replace FM's left-foot and right-foot taker lists for one set-piece kind.
    pub fn set_takers(
        &mut self,
        c: BridgeContext,
        kind: i32,
        left: &[i64],
        right: &[i64],
    ) -> Result<(), BridgeError> {
        let pairs = FmbIds::sp_pairs(c);
        let any = left.first().or_else(|| right.first()).copied().unwrap_or(0);
        let entry = |list: i32, index: i64, person: i64| {
            dyn_ref(vec![
                dyn_num(FmbIds::SP_KEY_KIND, kind),
                dyn_num(FmbIds::SP_KEY_LIST, list),
                dyn_num(FmbIds::KEY_SEL, index),
                dyn_entry(FmbIds::KEY_PERSON, FmbIds::REF_PERSON, person_ref(person)),
            ])
        };
        for list in 0..2 {
            for _ in 0..8 {
                self.send(
                    FmbIds::SP_EV_REMOVE,
                    "SetPieceReference",
                    &pairs,
                    &entry(list, 0, any),
                )?;
            }
        }
        let as_list = |ps: &[i64]| -> Value {
            Value::Array(
                ps.iter()
                    .map(|&x| json!(["SI.Bindable.DynamicReference", entry(2, x & 0xFF_FFFF, x)]))
                    .collect(),
            )
        };
        if !left.is_empty() {
            self.send(FmbIds::SP_EV_ADD_LEFT, "SetPieceReference", &pairs, &as_list(left))?;
        }
        if !right.is_empty() {
            self.send(FmbIds::SP_EV_ADD_RIGHT, "SetPieceReference", &pairs, &as_list(right))?;
        }
        Ok(())
    }

    /// Apply corner & direct free-kick takers per foot, corner technique (inswing/outswing),
    /// and corner aim target (far post / short).
    pub fn apply_takers(
        &mut self,
        c: BridgeContext,
        plan: &SetPiecePlan,
        squad: &[Player],
        person_of: &HashMap<String, i64>,
    ) -> Result<Vec<String>, BridgeError> {
        let mut field: Vec<(&Player, i64)> = Vec::new();
        for x in &plan.xi {
            if x.group == PosGroup::Gk {
                continue;
            }
            let Some(p) = squad.iter().find(|p| p.id == x.player_id) else {
                continue;
            };
            let Some(uid) = p.meta.uid.as_deref() else {
                continue;
            };
            let Some(&person) = person_of.get(uid) else {
                continue;
            };
            field.push((p, person));
        }
        if field.is_empty() {
            return Err(BridgeError::State(
                self.l(
                    "Nenhum jogador do XI encontrado no FM: o plantel tem de vir do save do clube que treina.",
                    "No XI player found in FM: the squad must come from the save of the club you manage.",
                )
                .to_string(),
            ));
        }

        let pick = |score: fn(&Player) -> f32, foot: Foot| -> Vec<(&Player, i64)> {
            let mut sorted = field.clone();
            sorted.sort_by(|a, b| {
                score(b.0)
                    .partial_cmp(&score(a.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let with_foot: Vec<(&Player, i64)> = sorted
                .iter()
                .copied()
                .filter(|(p, _)| matches!(p.foot_side(), Some(f) if f == foot || f == Foot::B))
                .collect();
            let chosen = if with_foot.is_empty() {
                sorted
            } else {
                with_foot
            };
            chosen.into_iter().take(2).collect()
        };

        type TakerScoreFn = fn(&Player) -> f32;
        let mut done = Vec::new();
        let kinds: [(i32, TakerScoreFn, &str); 2] = [
            (
                FmbIds::SP_KIND_CORNER,
                sp_corner_score,
                self.l("Cantos", "Corners"),
            ),
            (
                FmbIds::SP_KIND_FREE_KICK,
                sp_fk_direct_score,
                self.l("Livres directos", "Direct free kicks"),
            ),
        ];
        for (kind, score, label) in kinds {
            let l = pick(score, Foot::L);
            let r = pick(score, Foot::R);
            let l_ids: Vec<i64> = l.iter().map(|&(_, id)| id).collect();
            let r_ids: Vec<i64> = r.iter().map(|&(_, id)| id).collect();
            self.set_takers(c, kind, &l_ids, &r_ids)?;
            let l_names = l
                .iter()
                .map(|(p, _)| short_name(&p.name))
                .collect::<Vec<_>>()
                .join(", ");
            let r_names = r
                .iter()
                .map(|(p, _)| short_name(&p.name))
                .collect::<Vec<_>>()
                .join(", ");
            done.push(format!(
                "{label}: {l_names} ({}) · {r_names} ({})",
                self.l("pé esquerdo", "left foot"),
                self.l("pé direito", "right foot"),
            ));
        }

        let pairs = FmbIds::sp_pairs(c);
        let air = plan.air_top >= 14.0;
        let tab_rec = dyn_ref(vec![
            dyn_num(FmbIds::KEY_NUMBER, 0),
            dyn_str(FmbIds::KEY_PAYLOAD, "Corners"),
        ]);
        self.write("SetPieceReference", &pairs, FmbIds::SP_PROP_TYPE, &tab_rec)?;
        self.transport.sleep_ms(500);
        let tech = if air {
            FmbIds::SP_TECH_INSWING
        } else {
            FmbIds::SP_TECH_OUTSWING
        };
        self.send(
            FmbIds::SP_EV_TECHNIQUE,
            "SetPieceReference",
            &pairs,
            &json!(["SI.Core.DynamicNumber", tech.to_string()]),
        )?;
        done.push(
            if air {
                self.l("Cantos: em curva para dentro", "Corners: inswinging")
            } else {
                self.l("Cantos: em curva para fora", "Corners: outswinging")
            }
            .to_string(),
        );

        let tz = if air {
            FmbIds::SP_TARGET_FAR_POST
        } else {
            FmbIds::SP_TARGET_SHORT
        };
        let target_rec = dyn_ref(vec![
            dyn_str(FmbIds::KEY_PAYLOAD, FmbIds::sp_target_label(tz)),
            dyn_num(FmbIds::KEY_NUMBER, tz),
        ]);
        self.send(FmbIds::SP_EV_TARGET, "SetPieceReference", &pairs, &target_rec)?;
        done.push(
            if air {
                self.l("Cantos: procurar o 2.º poste", "Corners: aim for the far post")
            } else {
                self.l("Cantos: curtos", "Corners: short")
            }
            .to_string(),
        );
        Ok(done)
    }

    /// Routines need `READAT` (plugin >= 0.3).
    pub fn can_routines(&self) -> bool {
        self.state
            .as_ref()
            .map(|st| version_at_least(&st.version, 0, 3, 0))
            .unwrap_or(false)
    }

    /// Read the set-piece routine currently shown on FM's Set Pieces screen (`READAT auto`).
    /// Returns `Ok(None)` when the screen is not open (`kind < 0`).
    pub fn read_routine(&mut self, c: BridgeContext) -> Result<Option<BridgeRoutine>, BridgeError> {
        let pairs = FmbIds::sp_pairs(c);
        let res = self.read_at(
            &[("SetPieceReference", &pairs, FmbIds::SP_ROUTINE_PROP)],
            Some("auto"),
        )?;
        let Some(r) = res.first().and_then(|v| v.as_ref()) else {
            return Ok(None);
        };
        let kind = Self::num_i32(Self::field(Some(r), FmbIds::SP_ROUTINE_KIND), -1);
        if kind < 0 {
            return Ok(None);
        }
        let opt_of = |rec: Option<&Value>| -> BridgeRoutineOption {
            let inner = Self::field(rec, FmbIds::KEY_NUMBER);
            BridgeRoutineOption {
                id: Self::num_i32(Self::field(inner, FmbIds::SP_ROUTINE_OPTION), -1),
                label: Self::field(rec, FmbIds::KEY_PAYLOAD)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            }
        };
        let mut slots = Vec::new();
        for box_key in [FmbIds::SP_ROUTINE_IN_BOX, FmbIds::SP_ROUTINE_OUT_BOX] {
            for g in Self::items(Self::field(Some(r), box_key)) {
                let group = Self::num_i32(Self::field(Some(g), FmbIds::SP_ROUTINE_GROUP), -1);
                for s in Self::items(Self::field(Some(g), FmbIds::SP_ROUTINE_SLOTS)) {
                    let cur = opt_of(Self::field(Some(s), FmbIds::SP_ROUTINE_CURRENT));
                    let options = Self::items(Self::field(Some(s), FmbIds::SP_ROUTINE_OPTIONS))
                        .into_iter()
                        .map(|o| opt_of(Self::field(Some(o), FmbIds::SP_ROUTINE_OPT_ITEM)))
                        .filter(|o| o.id >= 0)
                        .collect();
                    slots.push(BridgeRoutineSlot {
                        group,
                        slot: Self::num_i32(Self::field(Some(s), FmbIds::SP_ROUTINE_SLOT), -1),
                        name: Self::field(Some(s), FmbIds::SP_ROUTINE_NAME)
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        person: Self::pair(
                            Self::field(Some(s), FmbIds::SP_ROUTINE_PERSON),
                            FmbIds::KEY_PERSON_ID,
                        ),
                        option: cur.id,
                        label: cur.label,
                        options,
                    });
                }
            }
        }
        let side = Self::num_i32(Self::field(Some(r), FmbIds::SP_ROUTINE_SIDE), 0);
        Ok(Some(BridgeRoutine { kind, side, slots }))
    }

    /// Set a single set-piece routine slot's instruction.
    pub fn set_instruction(
        &mut self,
        c: BridgeContext,
        kind: i32,
        slot: &BridgeRoutineSlot,
        opt: &BridgeRoutineOption,
    ) -> Result<String, BridgeError> {
        let pairs = FmbIds::sp_pairs(c);
        let rec = dyn_ref(vec![
            dyn_str(FmbIds::KEY_PAYLOAD, &opt.label),
            dyn_entry(
                FmbIds::KEY_NUMBER,
                "SI.Bindable.DynamicReference",
                dyn_ref(vec![
                    dyn_num(FmbIds::SP_ROUTINE_GROUP, slot.group),
                    dyn_num(FmbIds::SP_ROUTINE_SLOT, slot.slot),
                    dyn_num(FmbIds::SP_ROUTINE_OPTION, opt.id),
                    dyn_num(FmbIds::SP_ROUTINE_KIND, kind),
                    dyn_num(FmbIds::SP_ROUTINE_FLAG, 0),
                ]),
            ),
        ]);
        self.send(FmbIds::SP_ROUTINE_EV, "SetPieceReference", &pairs, &rec)
    }

    /// Apply the app's corner plan to one FM routine (attacking or defending), slot by slot.
    pub fn apply_routine(
        &mut self,
        c: BridgeContext,
        plan: &SetPiecePlan,
        routine: &BridgeRoutine,
        squad: &[Player],
        person_of: &HashMap<String, i64>,
    ) -> Result<Vec<String>, BridgeError> {
        let att = routine.kind == FmbIds::SP_KIND_ATT_CORNER;
        let rows = if att { &plan.attack } else { &plan.defend };
        // Preserve insertion order for missing-player reporting.
        let mut role_of: HashMap<i64, (&[i32], &str, &str)> = HashMap::new();
        let mut role_order: Vec<i64> = Vec::new();
        for r in rows {
            if r.key == "taker" {
                continue;
            }
            let Some(p) = squad.iter().find(|p| p.id == r.player_id) else {
                continue;
            };
            let Some(uid) = p.meta.uid.as_deref() else {
                continue;
            };
            let Some(&person) = person_of.get(uid) else {
                continue;
            };
            let norm_key = normalize_sp_role_key(&r.key);
            if let Some(prefs) = FmbIds::sp_role_prefs(att, norm_key) {
                if !role_of.contains_key(&person) {
                    role_order.push(person);
                }
                role_of.insert(person, (prefs, p.name.as_str(), r.label.as_str()));
            }
        }

        let mut done: Vec<String> = Vec::new();
        let mut kept: Vec<String> = Vec::new();
        for s in &routine.slots {
            let Some((prefs, player_name, role_label)) = role_of.remove(&s.person) else {
                continue;
            };
            let opt = prefs
                .iter()
                .find_map(|&id| s.options.iter().find(|o| o.id == id));
            let who = short_name(player_name);
            let Some(opt) = opt else {
                kept.push(format!("{who} ({role_label})"));
                continue;
            };
            if opt.id != s.option {
                self.set_instruction(c, routine.kind, s, opt)?;
            }
            done.push(format!("{who}: {}", opt.label));
        }

        let title = if att {
            self.l("Canto ofensivo", "Attacking corner")
        } else {
            self.l("Canto defensivo", "Defending corner")
        };
        let summary = if done.is_empty() {
            self.l("nada a mudar", "nothing to change").to_string()
        } else {
            done.join(" · ")
        };
        let mut out = vec![format!("{title}: {summary}")];
        if !kept.is_empty() {
            out.push(format!(
                "{}{}",
                self.lang.pick(
                    &format!("{title}: sem instrução compatível na categoria do FM para "),
                    &format!("{title}: no matching instruction in FM's category for "),
                ),
                kept.join(", ")
            ));
        }
        let missing: Vec<String> = role_order
            .into_iter()
            .filter_map(|pid| role_of.get(&pid).map(|&(_, name, _)| short_name(name)))
            .collect();
        if !missing.is_empty() {
            out.push(format!(
                "{}{}{}",
                self.lang.pick(
                    &format!("{title}: o FM não escalou "),
                    &format!("{title}: FM didn't put "),
                ),
                missing.join(", "),
                self.l(
                    " nesta rotina (escolhe pelos seus rankings)",
                    " in this routine (it picks from its own rankings)",
                )
            ));
        }
        Ok(out)
    }

    /// Apply full set-piece plan: takers + technique + aim, then poll FM's Set Pieces screen
    /// for both attacking and defending corner routines.
    pub fn apply_set_pieces(
        &mut self,
        plan: &SetPiecePlan,
        squad: &[Player],
        mut say: impl FnMut(&str),
    ) -> Result<Vec<String>, BridgeError> {
        self.status(true);
        let c = self.context()?;
        let person_of = self.persons_of(c)?;
        say(self.l("A aplicar os marcadores", "Applying the takers"));
        let mut done = self.apply_takers(c, plan, squad, &person_of)?;
        if !self.can_routines() {
            done.push(
                self.l(
                    "Rotinas dos cantos: precisam do Tactical Bible Bridge 0.3 ou mais recente.",
                    "Corner routines: need Tactical Bible Bridge 0.3 or newer.",
                )
                .to_string(),
            );
            return Ok(done);
        }
        let mut need: BTreeSet<i32> = [FmbIds::SP_KIND_ATT_CORNER, FmbIds::SP_KIND_DEF_CORNER]
            .into_iter()
            .collect();
        let t0 = Instant::now();
        let mut polls = 0usize;
        while !need.is_empty() && t0.elapsed() < self.routine_timeout {
            if let Some(max_p) = self.routine_max_polls {
                if polls >= max_p {
                    break;
                }
            }
            polls += 1;
            let r = self.read_routine(c)?;
            if let Some(routine) = r.as_ref() {
                if need.contains(&routine.kind) {
                    say(if routine.kind == FmbIds::SP_KIND_ATT_CORNER {
                        self.l("A aplicar o canto ofensivo", "Applying the attacking corner")
                    } else {
                        self.l("A aplicar o canto defensivo", "Applying the defending corner")
                    });
                    let lines = self.apply_routine(c, plan, routine, squad, &person_of)?;
                    done.extend(lines);
                    need.remove(&routine.kind);
                    continue;
                }
            }
            let prompt = match r {
                None => self.l(
                    "No FM, abra Táticas › Bolas paradas › Cantos",
                    "In FM, open Tactics › Set Pieces › Corners",
                ),
                Some(ref rt)
                    if need.contains(&FmbIds::SP_KIND_DEF_CORNER)
                        && rt.kind == FmbIds::SP_KIND_ATT_CORNER =>
                {
                    self.l(
                        "No FM, carregue em Defender (cantos)",
                        "In FM, click Defending (corners)",
                    )
                }
                Some(ref rt)
                    if need.contains(&FmbIds::SP_KIND_ATT_CORNER)
                        && rt.kind == FmbIds::SP_KIND_DEF_CORNER =>
                {
                    self.l(
                        "No FM, carregue em Atacar (cantos)",
                        "In FM, click Attacking (corners)",
                    )
                }
                Some(_) => self.l(
                    "No FM, abra o separador Cantos",
                    "In FM, open the Corners tab",
                ),
            };
            say(prompt);
            self.transport.sleep_ms(1_200);
        }
        if !need.is_empty() {
            let missing_names: Vec<&str> = need
                .iter()
                .map(|&k| {
                    if k == FmbIds::SP_KIND_ATT_CORNER {
                        self.l("o canto ofensivo", "the attacking corner")
                    } else {
                        self.l("o canto defensivo", "the defending corner")
                    }
                })
                .collect();
            done.push(format!(
                "{}{}{}",
                self.l(
                    "Rotinas: o tempo acabou antes de abrir ",
                    "Routines: timed out before opening ",
                ),
                missing_names.join(self.l(" e ", " and ")),
                self.l(" no FM.", " in FM."),
            ));
        }
        Ok(done)
    }

    // ---- Live Squad & Next Opponent ----

    /// Read the managed team's live squad from the running FM27 instance
    /// (visible attributes, positions, age, preferred foot, height).
    pub fn read_squad(
        &mut self,
        mut say: impl FnMut(&str),
    ) -> Result<BridgeSquadResult, BridgeError> {
        self.status(true);
        let c = self.context()?;
        let team_pairs = FmbIds::team_pairs(c.team);
        say(self.l("A ler a lista de jogadores", "Reading squad list"));
        let res = self.read(&[
            ("TeamReference", &team_pairs, FmbIds::TEAM_NAME),
            ("TeamReference", &team_pairs, FmbIds::TEAM_PLAYERS),
        ])?;
        let club = Self::text(res.first().and_then(|v| v.as_ref()));
        let mut seen = HashSet::new();
        let mut persons = Vec::new();
        for x in Self::items(res.get(1).and_then(|v| v.as_ref())) {
            let pid = Self::pair(Some(x), FmbIds::KEY_PERSON_ID);
            if pid != 0 && seen.insert(pid) {
                persons.push(pid);
            }
        }
        if persons.is_empty() {
            return Err(BridgeError::State(
                self.l(
                    "Não encontrei jogadores na equipa do FM.",
                    "No players found in the FM team.",
                )
                .to_string(),
            ));
        }
        let props: [i64; 11] = [
            FmbIds::PERSON_NAME,
            FmbIds::PERSON_UNIQUE_ID,
            FmbIds::PERSON_AGE,
            FmbIds::PERSON_HEIGHT,
            FmbIds::PERSON_FOOT,
            FmbIds::PERSON_POSITIONS,
            FmbIds::PERSON_ATTR_LISTS[0],
            FmbIds::PERSON_ATTR_LISTS[1],
            FmbIds::PERSON_ATTR_LISTS[2],
            FmbIds::PERSON_ATTR_LISTS[3],
            FmbIds::PERSON_ATTR_LISTS[4],
        ];
        let pair_strs: Vec<String> = persons.iter().map(|&id| FmbIds::person_pairs(id)).collect();
        let mut reqs: Vec<(&str, &str, i64)> = Vec::with_capacity(persons.len() * props.len());
        for ps in &pair_strs {
            for &pr in &props {
                reqs.push(("PersonReference", ps.as_str(), pr));
            }
        }
        say(self.lang.pick(
            &format!("A ler {} jogadores do FM", persons.len()),
            &format!("Reading {} players from FM", persons.len()),
        ));
        let vals = self.read(&reqs)?;

        let mut players = Vec::new();
        for (pi, &person_id) in persons.iter().enumerate() {
            let b = pi * props.len();
            let name = Self::text(vals.get(b).and_then(|v| v.as_ref()));
            if name.is_empty() {
                continue;
            }
            let uid_num = Self::num_i64(vals.get(b + 1).and_then(|v| v.as_ref())).unwrap_or(0);
            let uid = (uid_num > 0).then(|| uid_num.to_string());
            let age = Self::num_i64(vals.get(b + 2).and_then(|v| v.as_ref())).map(|n| n as f32);
            let height = Self::num_i64(vals.get(b + 3).and_then(|v| v.as_ref())).map(|n| n as f32);

            let foot_rec = vals.get(b + 4).and_then(|v| v.as_ref());
            let lf = Self::num_i64(Self::field(foot_rec, FmbIds::PERSON_FOOT_LEFT)).map(|n| n as f32);
            let rf =
                Self::num_i64(Self::field(foot_rec, FmbIds::PERSON_FOOT_RIGHT)).map(|n| n as f32);
            let foot = match (lf, rf) {
                (Some(l), Some(r)) => Some(if l >= 15.0 && r >= 15.0 {
                    Foot::B
                } else if l > r {
                    Foot::L
                } else {
                    Foot::R
                }),
                _ => None,
            };

            let mut pos_r: BTreeMap<String, f32> = BTreeMap::new();
            let mut nat_set: Vec<String> = Vec::new();
            let mut pos_entries: Vec<(&'static str, i64, &'static [&'static str])> = Vec::new();
            for item in Self::items(vals.get(b + 5).and_then(|v| v.as_ref())) {
                let bit = Self::num_u64(Self::field(Some(item), FmbIds::PERSON_POS_BIT)).unwrap_or(0);
                let Some((k, tags)) = FmbIds::person_pos_by_bit(bit) else {
                    continue;
                };
                let r = Self::num_i64(Self::field(Some(item), FmbIds::KEY_NUMBER)).unwrap_or(0);
                if r > 0 {
                    pos_r.insert(k.to_string(), r as f32);
                    pos_entries.push((k, r, tags));
                    if r >= 18 {
                        if let Some(&primary) = tags.first() {
                            if !nat_set.iter().any(|x| x == primary) {
                                nat_set.push(primary.to_string());
                            }
                        }
                    }
                }
            }
            let top = pos_entries.iter().map(|&(_, r, _)| r).max().unwrap_or(0);
            let good: Vec<(&'static str, &'static [&'static str])> = pos_entries
                .iter()
                .filter(|&&(_, r, _)| r >= 15 || (top < 15 && r == top))
                .map(|&(k, _, tags)| (k, tags))
                .collect();
            let mut pos: Vec<String> = Vec::new();
            for &(_, tags) in &good {
                for &tag in tags {
                    if !pos.iter().any(|x| x == tag) {
                        pos.push(tag.to_string());
                    }
                }
            }
            let pos_raw = good
                .iter()
                .map(|&(k, _)| k.to_uppercase())
                .collect::<Vec<_>>()
                .join(", ");

            let mut a: BTreeMap<Attr, f32> = BTreeMap::new();
            for li in 0..FmbIds::PERSON_ATTR_LISTS.len() {
                for item in Self::items(vals.get(b + 6 + li).and_then(|v| v.as_ref())) {
                    let info = Self::field(Some(item), FmbIds::PERSON_ATTR_INFO_BOX);
                    let val_box = Self::field(Some(item), FmbIds::PERSON_ATTR_VAL_BOX);
                    let aid = Self::num_i64(Self::field(info, FmbIds::PERSON_ATTR_ID)).unwrap_or(-1);
                    let v = Self::num_i64(Self::field(val_box, FmbIds::PERSON_ATTR_VAL));
                    if let (Some(attr), Some(val)) = (FmbIds::attr_by_fm_id(aid), v) {
                        a.insert(attr, val as f32);
                    }
                }
            }
            if a.len() < 3 {
                continue;
            }

            let id = match uid.as_deref() {
                Some(u) => format!("u{u}"),
                None => format!("fm{person_id}"),
            };
            let foot_label = foot.map(|f| f.en_label().to_string());
            let club_opt = (!club.is_empty()).then(|| club.clone());
            players.push(Player {
                id,
                name,
                pos,
                pos_raw,
                age,
                a,
                meta: PlayerMeta {
                    uid: uid.clone(),
                    club: club_opt.clone(),
                    foot: foot_label,
                    height: height.map(|h| (h.round() as i64).to_string()),
                    ..PlayerMeta::default()
                },
                pos_r,
                height,
                lf,
                rf,
                foot,
                uniq: uid,
                club: club_opt,
                nat_pos: nat_set,
                ..Player::default()
            });
        }

        Ok(BridgeSquadResult {
            team: c.team,
            club,
            players,
        })
    }

    /// Read the managed team's next scheduled opponent (`team_id`, `name`, `unique_id` list).
    pub fn next_opponent(&mut self) -> Result<Option<BridgeNextOpponent>, BridgeError> {
        let c = self.context()?;
        let my_pairs = FmbIds::team_pairs(c.team);
        let fx_res = self.read(&[("TeamReference", &my_pairs, FmbIds::TEAM_NEXT_FIXTURE)])?;
        let fx = fx_res.first().and_then(|v| v.as_ref());
        let p = Self::pair(fx, FmbIds::FIXTURE_OPP);
        let q = Self::pair(fx, FmbIds::FIXTURE_US);
        let opp = if p != 0 && p != c.team {
            p
        } else if q != 0 && q != c.team {
            q
        } else {
            0
        };
        if opp == 0 {
            return Ok(None);
        }
        let opp_pairs = FmbIds::team_pairs(opp);
        let res = self.read(&[
            ("TeamReference", &opp_pairs, FmbIds::TEAM_NAME),
            ("TeamReference", &opp_pairs, FmbIds::TEAM_PLAYERS),
        ])?;
        let name = Self::text(res.first().and_then(|v| v.as_ref()));
        let persons: Vec<i64> = Self::items(res.get(1).and_then(|v| v.as_ref()))
            .into_iter()
            .map(|x| Self::pair(Some(x), FmbIds::KEY_PERSON_ID))
            .filter(|&id| id != 0)
            .collect();
        let pair_strs: Vec<String> = persons.iter().map(|&id| FmbIds::person_pairs(id)).collect();
        let reqs: Vec<(&str, &str, i64)> = pair_strs
            .iter()
            .map(|ps| ("PersonReference", ps.as_str(), FmbIds::SEARCH_UNIQUE_ID))
            .collect();
        let uq = self.read(&reqs)?;
        let uniqs = uq
            .into_iter()
            .filter_map(|v| value_to_id_string(v.as_ref()))
            .filter(|s| !s.is_empty())
            .collect();
        Ok(Some(BridgeNextOpponent {
            team: opp,
            name,
            uniqs,
        }))
    }

    // ---- Recruitment: Search, Shortlist, Recruitment Focus ----

    /// Look up an FM runtime `person_id` by name using FM's own search, optionally confirming
    /// against `fmsave`'s `unique_id`.
    pub fn find_person(
        &mut self,
        name: &str,
        uniq: Option<&str>,
    ) -> Result<Option<i64>, BridgeError> {
        let sp = FmbIds::search_pairs();
        self.write(
            "SearchReference",
            &sp,
            FmbIds::SEARCH_TEXT,
            &json!(["System.String", name]),
        )?;
        for _ in 0..10 {
            self.transport.sleep_ms(350);
            let res = self.read(&[("SearchReference", &sp, FmbIds::SEARCH_PEOPLE)])?;
            let ids: Vec<i64> = Self::items(res.first().and_then(|v| v.as_ref()))
                .into_iter()
                .map(|x| Self::pair(Some(x), FmbIds::KEY_PERSON_ID))
                .filter(|&id| id != 0)
                .collect();
            if ids.is_empty() {
                continue;
            }
            let Some(want_uniq) = uniq.filter(|s| !s.is_empty()) else {
                return Ok(if ids.len() == 1 { Some(ids[0]) } else { None });
            };
            let pair_strs: Vec<String> = ids.iter().map(|&id| FmbIds::person_pairs(id)).collect();
            let reqs: Vec<(&str, &str, i64)> = pair_strs
                .iter()
                .map(|ps| ("PersonReference", ps.as_str(), FmbIds::SEARCH_UNIQUE_ID))
                .collect();
            let uq = self.read(&reqs)?;
            let matched = uq
                .iter()
                .position(|v| value_to_id_string(v.as_ref()).as_deref() == Some(want_uniq));
            return Ok(matched.map(|k| ids[k]));
        }
        Ok(None)
    }

    /// Add one FM `person_id` to FM's shortlist.
    pub fn shortlist_add(
        &mut self,
        c: BridgeContext,
        person: i64,
    ) -> Result<String, BridgeError> {
        let p_rec = |id: i64| dyn_ref(vec![dyn_num(FmbIds::KEY_PERSON_ID, id)]);
        let pairs = FmbIds::action_pairs(FmbIds::ACTION_SHORTLIST_ADD);
        let payload = dyn_ref(vec![
            dyn_num(FmbIds::ACTION_KEY_FLAG1, 1),
            dyn_entry(
                FmbIds::ACTION_KEY_STR_LIST,
                "System.Collections.Generic.List`1",
                json!([["System.String", ""]]),
            ),
            dyn_entry(
                FmbIds::ACTION_KEY_MANAGER,
                "SI.Bindable.DynamicReference",
                p_rec(c.manager),
            ),
            dyn_entry(
                FmbIds::KEY_PERSON_ID,
                "SI.Bindable.DynamicReference",
                p_rec(person),
            ),
            dyn_entry(
                FmbIds::ACTION_KEY_PERSON_LIST,
                "System.Collections.Generic.List`1",
                json!([["SI.Bindable.DynamicReference", p_rec(person)]]),
            ),
            dyn_str(FmbIds::ACTION_KEY_EMPTY_STR, ""),
            dyn_num(FmbIds::ACTION_KEY_ZERO1, 0),
            dyn_num(FmbIds::ACTION_KEY_ZERO2, 0),
        ]);
        self.send(FmbIds::ACTION_EV, "ActionReference", &pairs, &payload)
    }

    /// Push the app's shortlist to FM27's shortlist.
    pub fn push_shortlist(
        &mut self,
        list: &[ShortlistEntry],
        mut say: impl FnMut(&str, f32),
    ) -> Result<BridgeShortlistResult, BridgeError> {
        let c = self.context()?;
        let mut out = BridgeShortlistResult::default();
        let total = list.len();
        for (i, p) in list.iter().enumerate() {
            let frac = if total == 0 {
                0.0
            } else {
                i as f32 / total as f32
            };
            say(&format!("{}/{} · {}", i + 1, total, p.name), frac);
            let person = self.find_person(&p.name, p.uniq.as_deref())?;
            match person {
                Some(pid) => {
                    self.shortlist_add(c, pid)?;
                    out.added.push(p.name.clone());
                }
                None => out.missing.push(p.name.clone()),
            }
        }
        Ok(out)
    }

    /// Read all existing recruitment focuses in FM27.
    pub fn focus_list(&mut self, c: BridgeContext) -> Result<Vec<BridgeFocusItem>, BridgeError> {
        let info_pairs = FmbIds::focus_info_pairs(c);
        let res = self.read(&[("ScoutingInfoReference", &info_pairs, FmbIds::FOCUS_LIST)])?;
        let ids: Vec<i64> = Self::items(res.first().and_then(|v| v.as_ref()))
            .into_iter()
            .filter_map(|x| Self::pair_opt(Some(x), FmbIds::FOCUS_ID))
            .collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let pair_strs: Vec<String> = ids.iter().map(|&id| FmbIds::focus_pairs(c, id)).collect();
        let reqs: Vec<(&str, &str, i64)> = pair_strs
            .iter()
            .map(|ps| ("RecruitmentFocusItemReference", ps.as_str(), FmbIds::FOCUS_NAME))
            .collect();
        let names = self.read(&reqs)?;
        Ok(ids
            .into_iter()
            .zip(names)
            .map(|(id, nm)| BridgeFocusItem {
                id,
                name: nm
                    .as_ref()
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            })
            .collect())
    }

    /// Create a recruitment focus (scout assignment) in FM27 and return the created item.
    pub fn create_focus(
        &mut self,
        req: &BridgeFocusRequest,
        mut say: impl FnMut(&str),
    ) -> Result<BridgeFocusItem, BridgeError> {
        let c = self.context()?;
        let p = FmbIds::focus_pairs(c, -1);
        let areas_res = self.read(&[("RecruitmentFocusItemReference", &p, FmbIds::FOCUS_AREAS)])?;
        if !Self::items(areas_res.first().and_then(|v| v.as_ref())).is_empty() {
            return Err(BridgeError::State(
                self.l(
                    "Há um Recruitment Focus a meio no FM (com áreas escolhidas): termine-o ou cancele-o primeiro.",
                    "There is a Recruitment Focus half-created in FM (with areas chosen): finish or cancel it first.",
                )
                .to_string(),
            ));
        }
        let before: HashSet<i64> = self.focus_list(c)?.into_iter().map(|f| f.id).collect();
        say(self.l("Idade e capacidade", "Age and ability"));
        let num_rec = |n: u16| {
            dyn_ref(vec![
                dyn_num(FmbIds::KEY_NUMBER, n),
                dyn_str(FmbIds::KEY_PAYLOAD, &n.to_string()),
            ])
        };
        self.write(
            "RecruitmentFocusItemReference",
            &p,
            FmbIds::FOCUS_AGE_MIN,
            &num_rec(req.age_min),
        )?;
        self.write(
            "RecruitmentFocusItemReference",
            &p,
            FmbIds::FOCUS_AGE_MAX,
            &num_rec(req.age_max),
        )?;

        self.step_focus_stars(&p, FmbIds::FOCUS_CA, FmbIds::FOCUS_CA_STEP, req.ca_stars)?;
        self.step_focus_stars(&p, FmbIds::FOCUS_PA, FmbIds::FOCUS_PA_STEP, req.pa_stars)?;

        say(self.l("Áreas", "Areas"));
        for a in &req.areas {
            let area_rec = dyn_ref(vec![
                dyn_str(FmbIds::KEY_PAYLOAD, &a.label),
                dyn_entry(
                    FmbIds::KEY_NUMBER,
                    "SI.Bindable.DynamicReference",
                    dyn_ref(vec![dyn_num(FmbIds::KEY_PERSON_ID, a.id)]),
                ),
            ]);
            self.write("RecruitmentFocusItemReference", &p, FmbIds::FOCUS_AREA, &area_rec)?;
            self.send(
                FmbIds::FOCUS_ADD_AREA,
                "RecruitmentFocusItemReference",
                &p,
                &Value::Null,
            )?;
            self.transport.sleep_ms(250);
        }

        let short_focus_name: String = req.name.chars().take(25).collect();
        self.write(
            "RecruitmentFocusItemReference",
            &p,
            FmbIds::FOCUS_NAME,
            &json!(["System.String", short_focus_name]),
        )?;
        self.transport.sleep_ms(300);

        say(self.l("A confirmar", "Confirming"));
        self.send(
            FmbIds::FOCUS_CONFIRM,
            "RecruitmentFocusItemReference",
            &p,
            &Value::Null,
        )?;
        self.transport.sleep_ms(900);

        let after = self.focus_list(c)?;
        after
            .into_iter()
            .find(|f| !before.contains(&f.id))
            .ok_or_else(|| {
                BridgeError::State(
                    self.l(
                        "O FM não criou o Recruitment Focus.",
                        "FM did not create the Recruitment Focus.",
                    )
                    .to_string(),
                )
            })
    }

    fn step_focus_stars(
        &mut self,
        pairs: &str,
        prop: i64,
        step_ev: i64,
        stars: f32,
    ) -> Result<(), BridgeError> {
        let target = (10.0 + 2.0 * stars).round() as i64;
        for _ in 0..14 {
            let res = self.read(&[("RecruitmentFocusItemReference", pairs, prop)])?;
            let r = res.first().and_then(|v| v.as_ref());
            let v = Self::num_i64(Self::field(r, FmbIds::FOCUS_LEVEL)).unwrap_or(target);
            if v == target {
                return Ok(());
            }
            let dir = if v < target { "1" } else { "0" };
            self.send(
                step_ev,
                "RecruitmentFocusItemReference",
                pairs,
                &json!(["SI.Core.DynamicNumber", dir]),
            )?;
            self.transport.sleep_ms(200);
        }
        Ok(())
    }
}

// ============================================================================
// Pure Protocol & Helper Functions
// ============================================================================

fn dyn_entry(key: i64, ty: &str, val: Value) -> Value {
    json!([key, ty, val])
}

fn dyn_num(key: i64, val: impl ToString) -> Value {
    json!([key, "SI.Core.DynamicNumber", val.to_string()])
}

fn dyn_bool(key: i64, val: bool) -> Value {
    json!([key, "System.Boolean", if val { "True" } else { "False" }])
}

fn dyn_str(key: i64, val: &str) -> Value {
    json!([key, "System.String", val])
}

fn dyn_ref(entries: Vec<Value>) -> Value {
    json!({
        "type": "SI.Bindable.DynamicReference",
        "entries": entries,
    })
}

fn core_record(entries: Vec<Value>) -> Value {
    json!({
        "type": "SI.Core.Record",
        "entries": entries,
    })
}

fn person_ref(person_id: i64) -> Value {
    json!({
        "type": FmbIds::REF_PERSON,
        "pairs": [
            [FmbIds::PAIR_TYPE_KEY, FmbIds::PAIR_TYPE_PERSON_OR_TEAM],
            [FmbIds::PAIR_SESSION_KEY, FmbIds::PAIR_SESSION_DEFAULT],
            [FmbIds::KEY_PERSON_ID, person_id],
        ]
    })
}

fn move_record(slot: i32, from: u64, to: u64, in_possession: bool) -> Value {
    core_record(vec![
        dyn_entry(
            FmbIds::TACTIC_MOVE_FROM,
            "SI.Core.Record",
            core_record(vec![
                dyn_num(FmbIds::TACTIC_MOVE_FROM, from),
                dyn_bool(FmbIds::TACTIC_MOVE_IP, in_possession),
                dyn_num(FmbIds::TACTIC_MOVE_SLOT, slot),
            ]),
        ),
        dyn_entry(
            FmbIds::TACTIC_MOVE_TO,
            "SI.Core.Record",
            core_record(vec![
                dyn_num(FmbIds::TACTIC_MOVE_TO, to),
                dyn_bool(FmbIds::TACTIC_MOVE_IP, in_possession),
            ]),
        ),
    ])
}

fn set_role_record(slot: i32, bits: u64, name: &str, in_possession: bool) -> Value {
    core_record(vec![
        dyn_num(FmbIds::TACTIC_SLOT_IDX, slot),
        dyn_entry(
            FmbIds::TACTIC_ROLE_REC,
            "SI.Bindable.DynamicReference",
            dyn_ref(vec![
                dyn_num(FmbIds::KEY_NUMBER, 1),
                dyn_entry(
                    FmbIds::KEY_PAYLOAD,
                    "SI.Bindable.DynamicReference",
                    dyn_ref(vec![
                        dyn_num(FmbIds::TACTIC_ROLE_REC, bits),
                        dyn_entry(
                            FmbIds::TACTIC_ROLE_RATING_REC,
                            "SI.Bindable.DynamicReference",
                            dyn_ref(vec![
                                dyn_num(FmbIds::TACTIC_ROLE_RATING_CUR, 14),
                                dyn_num(FmbIds::TACTIC_ROLE_RATING_MAX, 14),
                                dyn_bool(FmbIds::TACTIC_ROLE_RATING_FLAG, false),
                            ]),
                        ),
                        dyn_bool(FmbIds::TACTIC_ROLE_LOCK_FLAG, false),
                        dyn_str(FmbIds::TACTIC_ROLE_NAME, name),
                    ]),
                ),
            ]),
        ),
        dyn_bool(FmbIds::TACTIC_ROLE_IP, in_possession),
    ])
}

fn pi_revert_record(slot: i32, in_possession: bool) -> Value {
    core_record(vec![
        dyn_num(FmbIds::TACTIC_SLOT_IDX, slot),
        dyn_bool(FmbIds::TACTIC_ROLE_IP, in_possession),
        dyn_entry(
            FmbIds::TACTIC_PI_REVERT_REC,
            "SI.Bindable.DynamicReference",
            dyn_ref(vec![
                dyn_num(FmbIds::KEY_NUMBER, 0),
                dyn_str(FmbIds::KEY_PAYLOAD, "Revert to Role"),
            ]),
        ),
    ])
}

fn pi_press_record(slot: i32, cat: i64, in_possession: bool) -> Value {
    dyn_ref(vec![
        dyn_num(FmbIds::TACTIC_SLOT_IDX, slot),
        dyn_num(FmbIds::TACTIC_PI_CAT, cat),
        dyn_bool(FmbIds::TACTIC_ROLE_IP, in_possession),
    ])
}

fn split_central_side(code: &str) -> (&str, u64) {
    for prefix in ["DC", "DM", "MC", "AMC", "ST"] {
        if let Some(rest) = code.strip_prefix(prefix) {
            if rest == "L" {
                return (prefix, FmbIds::TACTIC_LEFT);
            }
            if rest == "R" {
                return (prefix, FmbIds::TACTIC_RIGHT);
            }
        }
    }
    (code, 0)
}

fn slot_base_code(code: &str) -> &str {
    split_central_side(code).0
}

fn slot_lateral_order(code: &str) -> u8 {
    match split_central_side(code).1 {
        FmbIds::TACTIC_LEFT => 0,
        0 => 1,
        _ => 2,
    }
}

fn normalize_sp_role_key(key: &str) -> &str {
    if let Some(rest) = key.strip_prefix("back") {
        if rest.len() == 1 && rest.as_bytes()[0].is_ascii_digit() {
            return "back";
        }
    }
    if let Some(rest) = key.strip_prefix('m') {
        if rest.len() == 1 && rest.as_bytes()[0].is_ascii_digit() {
            return "m";
        }
    }
    key
}

fn version_at_least(version: &str, min_major: u32, min_minor: u32, min_patch: u32) -> bool {
    let mut parts = version.trim().split('.').map(|s| s.parse::<u32>().unwrap_or(0));
    let maj = parts.next().unwrap_or(0);
    let min = parts.next().unwrap_or(0);
    let pat = parts.next().unwrap_or(0);
    (maj, min, pat) >= (min_major, min_minor, min_patch)
}

fn value_to_id_string(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        other => Some(other.to_string()),
    }
}

fn sp_corner_score(p: &Player) -> f32 {
    let a = |attr: Attr| p.attr(attr).unwrap_or(8.0);
    (a(Attr::Cor) * 1.6 + a(Attr::Cro) + a(Attr::Tec) * 0.5) / 3.1
}

fn sp_fk_direct_score(p: &Player) -> f32 {
    let a = |attr: Attr| p.attr(attr).unwrap_or(8.0);
    (a(Attr::Fre) * 1.7 + a(Attr::Tec) + a(Attr::Lon) * 0.6 + a(Attr::Cmp) * 0.3) / 3.6
}

const NAME_PARTICLES: &[&str] = &[
    "de", "da", "das", "do", "dos", "di", "del", "della", "van", "von", "der", "den", "ten", "ter",
    "le", "la", "du", "mac", "al", "el", "bin", "st.",
];

fn is_name_suffix(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "junior" | "júnior" | "jr" | "jr." | "filho" | "neto" | "sobrinho" | "ii" | "iii"
    )
}

fn short_name(name: &str) -> String {
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
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        BridgeFocusArea, CornerTaker, Mentality, SetPieceRole, XiMember,
    };
    use std::collections::VecDeque;

    type MockHandler = Box<dyn FnMut(&str, u64) -> Result<String, BridgeError>>;

    struct MockTransport {
        handler: MockHandler,
        sent: Vec<String>,
    }

    impl MockTransport {
        fn new<F>(f: F) -> Self
        where
            F: FnMut(&str, u64) -> Result<String, BridgeError> + 'static,
        {
            Self {
                handler: Box::new(f),
                sent: Vec::new(),
            }
        }

        fn from_replies(replies: Vec<&str>) -> Self {
            let mut q: VecDeque<String> = replies.into_iter().map(String::from).collect();
            Self::new(move |cmd, _| {
                q.pop_front()
                    .ok_or_else(|| BridgeError::Transport(format!("unexpected cmd: {cmd}")))
            })
        }
    }

    impl BridgeTransport for MockTransport {
        fn call(&mut self, cmd: &str, timeout_ms: u64) -> Result<String, BridgeError> {
            self.sent.push(cmd.to_string());
            (self.handler)(cmd, timeout_ms)
        }

        fn sleep_ms(&self, _ms: u64) {}
    }

    fn make_player(id: &str, uid: &str, name: &str, foot: Foot, attrs: &[(Attr, f32)]) -> Player {
        let mut a = BTreeMap::new();
        for &(k, v) in attrs {
            a.insert(k, v);
        }
        Player {
            id: id.to_string(),
            name: name.to_string(),
            foot: Some(foot),
            a,
            meta: PlayerMeta {
                uid: Some(uid.to_string()),
                foot: Some(foot.en_label().to_string()),
                ..PlayerMeta::default()
            },
            ..Player::default()
        }
    }

    fn slot_rec(slot: i32, ip_pos: u64, oop_pos: u64, person: i64) -> Value {
        json!([
            "SI.Bindable.DynamicReference",
            {
                "type": "SI.Bindable.DynamicReference",
                "entries": [
                    [FmbIds::KEY_SLOT, "SI.Core.DynamicNumber", slot.to_string()],
                    [FmbIds::TACTIC_IP_POS, "SI.Core.DynamicNumber", ip_pos.to_string()],
                    [FmbIds::TACTIC_OOP_POS, "SI.Core.DynamicNumber", oop_pos.to_string()],
                    [FmbIds::KEY_PERSON, "FM.UI.PersonReference", person_ref(person)],
                ]
            }
        ])
    }

    fn role_rec(slot: i32, key: i64, bits: u64) -> Value {
        json!([
            "SI.Bindable.DynamicReference",
            {
                "type": "SI.Bindable.DynamicReference",
                "entries": [
                    [FmbIds::TACTIC_SLOT_IDX, "SI.Core.DynamicNumber", slot.to_string()],
                    [key, "SI.Core.DynamicNumber", bits.to_string()],
                ]
            }
        ])
    }

    fn sample_tactic_slots(ip_codes: &[&str], oop_codes: &[&str]) -> Vec<SlotAssignment> {
        ip_codes
            .iter()
            .zip(oop_codes.iter())
            .enumerate()
            .map(|(i, (&ip_c, &oop_c))| SlotAssignment {
                ip_code: ip_c.to_string(),
                oop_code: oop_c.to_string(),
                ip: if ip_c == "GK" {
                    "bpgk"
                } else if ip_c == "MC" || ip_c == "MCL" || ip_c == "MCR" {
                    "mp"
                } else if ip_c == "ST" {
                    "f9"
                } else {
                    "cb_ip"
                }
                .to_string(),
                oop: if oop_c == "GK" {
                    "swk"
                } else if oop_c == "MC" || oop_c == "MCL" || oop_c == "MCR" {
                    "cm_oop"
                } else if oop_c == "ST" {
                    "cf_oop"
                } else {
                    "cb_oop"
                }
                .to_string(),
                pid: Some(format!("p{i}")),
                pis: if i == 1 {
                    vec!["Stay wider".to_string(), "Press more".to_string(), "Hold position".to_string()]
                } else {
                    vec![]
                },
            })
            .collect()
    }

    #[test]
    fn ids_tables_are_complete_and_unique() {
        assert_eq!(FmbIds::TACTIC_ROLES.len(), 69);
        assert_eq!(FmbIds::TACTIC_TIS.len(), 27);
        assert_eq!(FmbIds::TACTIC_PIS.len(), 20);
        assert_eq!(FmbIds::PERSON_ATTR_KEYS.len(), 47);

        let role_ids: HashSet<&str> = FmbIds::TACTIC_ROLES.iter().map(|r| r.id).collect();
        assert_eq!(role_ids.len(), FmbIds::TACTIC_ROLES.len());

        let ti_keys: HashSet<&str> = FmbIds::TACTIC_TIS.iter().map(|t| t.key).collect();
        assert_eq!(ti_keys.len(), 27);

        let pi_names: HashSet<&str> = FmbIds::TACTIC_PIS.iter().map(|p| p.name).collect();
        assert_eq!(pi_names.len(), 20);

        let attrs: HashSet<Attr> = FmbIds::PERSON_ATTR_KEYS.iter().copied().collect();
        assert_eq!(attrs.len(), 47);

        // Verify 64-bit role masks exceeding 2^53 (JS Number.MAX_SAFE_INTEGER) remain exact.
        assert_eq!(FmbIds::role_bits("mp"), Some(144_115_188_075_855_872));
        assert_eq!(FmbIds::role_bits("chm"), Some(72_057_594_037_927_936));
        assert_eq!(FmbIds::role_bits("pwb"), Some(36_028_797_018_963_968));
        assert_eq!(FmbIds::role_bits("nngk"), Some(9_007_199_254_740_992));

        // Verify ip_bits left/right central flags.
        assert_eq!(FmbIds::ip_bits("DCL"), Some(16 + FmbIds::TACTIC_LEFT));
        assert_eq!(FmbIds::ip_bits("DCR"), Some(16 + FmbIds::TACTIC_RIGHT));
        assert_eq!(FmbIds::ip_bits("ST"), Some(16_384));
        assert_eq!(FmbIds::ip_bits("INVALID"), None);
        assert_eq!(FmbIds::position(16 + FmbIds::TACTIC_LEFT), "DC");
        assert_eq!(FmbIds::focus_nation(42), 167_772_202);
    }

    #[test]
    fn status_and_recorder_helpers() {
        let mock = MockTransport::from_replies(vec![
            "PONG ready",
            "OK 0.4.2 27.1.0",
            "OK",
            "OK",
            "OK {\"ev\":21}\x1e{\"ev\":1467503190}",
        ]);
        let mut client = FmbClient::new(mock, Lang::En);
        let st = client.status(true);
        assert!(st.ok && st.ready);
        assert_eq!(st.version, "0.4.2");
        assert_eq!(st.fm, "27.1.0");
        assert!(client.can_routines());

        // Throttled second call shouldn't hit transport.
        let st2 = client.status(false);
        assert_eq!(st2, st);
        assert_eq!(client.transport().sent.len(), 2);

        client.rec_start().unwrap();
        assert!(client.recording().on);
        let count = client.rec_stop().unwrap();
        assert_eq!(count, 2);
        assert!(!client.recording().on);
        assert!(client.recording().out.contains("\"0.4.2\""));
        assert!(client.recording().out.contains("1467503190"));
    }

    #[test]
    fn context_parses_training_and_calendar_refs() {
        let mock = MockTransport::from_replies(vec![
            "OK session=1 FM.UI.TeamTrainingReference=1348620377=9001;1416914273=5002 | FM.UI.CalendarReference=1214805357=111;1415930221=222",
            "OK session=1 FM.UI.CalendarReference=1214805357=777;1415930221=888",
            "OK session=1 FM.UI.TeamTacticReference=1685218662=999",
        ]);
        let mut client = FmbClient::new(mock, Lang::En);
        assert_eq!(
            client.context().unwrap(),
            BridgeContext {
                manager: 9001,
                team: 5002
            }
        );
        assert_eq!(
            client.context().unwrap(),
            BridgeContext {
                manager: 777,
                team: 888
            }
        );
        let err = client.context().unwrap_err();
        assert!(err.to_string().contains("Training screen"));
    }

    #[test]
    fn read_tactic_and_apply_xi_matches_and_swaps() {
        let mut slot_persons: HashMap<i32, i64> = HashMap::from([
            (0, 100), // GK already correct
            (1, 999), // DCL wrong -> needs swap to 201
            (2, 998), // DCR wrong -> needs swap to 202
        ]);
        let mock = MockTransport::new(move |cmd, _| {
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.starts_with("READ ") && cmd.contains(&FmbIds::PROP_TRAINING_PLAYERS.to_string()) {
                let list = json!([
                    ["FM.UI.PersonReference", person_ref(100)],
                    ["FM.UI.PersonReference", person_ref(201)],
                    ["FM.UI.PersonReference", person_ref(202)],
                ]);
                return Ok(format!("OK {list}"));
            }
            if cmd.starts_with("READ ") && cmd.contains(&FmbIds::PERSON_UNIQUE_ID.to_string()) {
                return Ok("OK 1000\x1e2001\x1e2002".into());
            }
            if cmd.starts_with("READ ") && cmd.contains(&FmbIds::PROP_SELECTION.to_string()) {
                let slots = json!([
                    slot_rec(0, 1, 1, slot_persons[&0]),
                    slot_rec(1, 16 + FmbIds::TACTIC_LEFT, 16 + FmbIds::TACTIC_LEFT, slot_persons[&1]),
                    slot_rec(2, 16 + FmbIds::TACTIC_RIGHT, 16 + FmbIds::TACTIC_RIGHT, slot_persons[&2]),
                ]);
                let sel = json!([
                    ["SI.Bindable.DynamicReference", {
                        "type": "SI.Bindable.DynamicReference",
                        "entries": [
                            [FmbIds::KEY_PERSON, "FM.UI.PersonReference", person_ref(201)],
                            [FmbIds::KEY_SEL, "SI.Core.DynamicNumber", "12"]
                        ]
                    }],
                    ["SI.Bindable.DynamicReference", {
                        "type": "SI.Bindable.DynamicReference",
                        "entries": [
                            [FmbIds::KEY_PERSON, "FM.UI.PersonReference", person_ref(202)],
                            [FmbIds::KEY_SEL, "SI.Core.DynamicNumber", "13"]
                        ]
                    }],
                ]);
                return Ok(format!("OK \"My 4-3-3\"\x1e{slots}\x1e{sel}"));
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::EV_PLACE)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let payload = FmbClient::<MockTransport>::field(Some(&v), FmbIds::KEY_PAYLOAD);
                let idx = FmbClient::<MockTransport>::num_i32(
                    FmbClient::<MockTransport>::field(payload, FmbIds::KEY_SLOT),
                    -1,
                );
                let person = FmbClient::<MockTransport>::pair(
                    FmbClient::<MockTransport>::field(payload, FmbIds::KEY_PERSON),
                    FmbIds::KEY_PERSON_ID,
                );
                slot_persons.insert(idx, person);
                return Ok("OK".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut client = FmbClient::new(mock, Lang::En);
        let c = client.context().unwrap();
        let st = client.read_tactic(c).unwrap();
        assert_eq!(st.name, "My 4-3-3");
        assert_eq!(st.slots.len(), 3);
        assert_eq!(st.sel_index.get(&201), Some(&12));

        let tactic = Tactic {
            name: "Test".into(),
            ment: Mentality::Positive,
            ip_form: "4-3-3 DM Wide".into(),
            oop_form: "4-3-3 DM Wide".into(),
            slots: vec![
                SlotAssignment { ip_code: "GK".into(), pid: Some("p0".into()), ..Default::default() },
                SlotAssignment { ip_code: "DCR".into(), pid: Some("p2".into()), ..Default::default() },
                SlotAssignment { ip_code: "DCL".into(), pid: Some("p1".into()), ..Default::default() },
                SlotAssignment { ip_code: "ST".into(), pid: Some("p3".into()), ..Default::default() },
            ],
            ..Default::default()
        };
        let squad = vec![
            make_player("p0", "1000", "Keeper", Foot::R, &[]),
            make_player("p1", "2001", "Left CB", Foot::L, &[]),
            make_player("p2", "2002", "Right CB", Foot::R, &[]),
            make_player("p3", "9999", "Missing ST", Foot::R, &[]),
        ];

        let res = client.apply_xi(&tactic, &squad).unwrap();
        assert_eq!(res.placed, 2);
        assert_eq!(res.skipped.len(), 1);
        assert!(res.skipped[0].contains("no ST slot"));
    }

    #[test]
    fn create_tactic_refuses_at_three_and_succeeds_at_two() {
        let mut tactics = vec!["T1".to_string(), "T2".to_string(), "T3".to_string()];
        let mock = MockTransport::new(move |cmd, _| {
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.contains(&FmbIds::TACTIC_MENU.to_string()) {
                let sub: Vec<Value> = tactics
                    .iter()
                    .map(|nm| {
                        json!([
                            "SI.Bindable.DynamicReference",
                            {
                                "type": "SI.Bindable.DynamicReference",
                                "entries": [[FmbIds::KEY_PAYLOAD, "System.String", nm]]
                            }
                        ])
                    })
                    .collect();
                let menu = json!([[
                    "SI.Bindable.DynamicReference",
                    {
                        "type": "SI.Bindable.DynamicReference",
                        "entries": [
                            [FmbIds::TACTIC_MENU_ITEM, "SI.Bindable.DynamicReference", {
                                "type": "SI.Bindable.DynamicReference",
                                "entries": [[FmbIds::KEY_NUMBER, "SI.Core.DynamicNumber", "0"]]
                            }],
                            [FmbIds::TACTIC_MENU_SUB, "System.Collections.Generic.List`1", sub],
                        ]
                    }
                ]]);
                return Ok(format!("OK {menu}"));
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_WIZARD_FINISH)) {
                tactics.push("Clean Slate".into());
                return Ok("OK".into());
            }
            if cmd.starts_with("SENDJ ") || cmd.starts_with("WRITE ") {
                return Ok("OK".into());
            }
            if cmd.contains(&FmbIds::TACTIC_NAME.to_string()) {
                return Ok("OK \"Clean Slate\"".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut client = FmbClient::new(mock, Lang::En);
        let err = client.create_tactic(|_| {}).unwrap_err();
        assert!(err.to_string().contains("3 tactics"));

        // Now test with 2 tactics -> creates 3rd via Clean Slate wizard (and never sends event 90!).
        let mut tactics2 = vec!["T1".to_string(), "T2".to_string()];
        let mock2 = MockTransport::new(move |cmd, _| {
            assert!(!cmd.starts_with("SENDJ 90 "), "must never replay event 90");
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.contains(&FmbIds::TACTIC_MENU.to_string()) {
                let sub: Vec<Value> = tactics2
                    .iter()
                    .map(|nm| {
                        json!([
                            "SI.Bindable.DynamicReference",
                            {
                                "type": "SI.Bindable.DynamicReference",
                                "entries": [[FmbIds::KEY_PAYLOAD, "System.String", nm]]
                            }
                        ])
                    })
                    .collect();
                let menu = json!([[
                    "SI.Bindable.DynamicReference",
                    {
                        "type": "SI.Bindable.DynamicReference",
                        "entries": [
                            [FmbIds::TACTIC_MENU_ITEM, "SI.Bindable.DynamicReference", {
                                "type": "SI.Bindable.DynamicReference",
                                "entries": [[FmbIds::KEY_NUMBER, "SI.Core.DynamicNumber", "0"]]
                            }],
                            [FmbIds::TACTIC_MENU_SUB, "System.Collections.Generic.List`1", sub],
                        ]
                    }
                ]]);
                return Ok(format!("OK {menu}"));
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_WIZARD_FINISH)) {
                tactics2.push("Clean Slate".into());
                return Ok("OK".into());
            }
            if cmd.starts_with("SENDJ ") || cmd.starts_with("WRITE ") {
                return Ok("OK".into());
            }
            if cmd.contains(&FmbIds::TACTIC_NAME.to_string()) {
                return Ok("OK \"Clean Slate\"".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut client2 = FmbClient::new(mock2, Lang::En);
        let nm = client2.create_tactic(|_| {}).unwrap();
        assert_eq!(nm, "Clean Slate");
    }

    #[test]
    fn apply_tactic_handles_oop_cycles_64bit_roles_pis_and_tis() {
        // 3 slots with a 2-slot OOP swap cycle (slot 1 and slot 2 swap OOP positions DM <-> MC!)
        // and a 64-bit IP role (mp = 144115188075855872) + OOP role with side flags (cm_oop | side).
        #[derive(Clone)]
        struct FmSlotState {
            slot: i32,
            ip: u64,
            oop: u64,
            ip_role: u64,
            oop_role_raw: u64,
            person: i64,
        }
        let initial_state = vec![
            FmSlotState { slot: 0, ip: 1, oop: 1, ip_role: 1, oop_role_raw: 1, person: 100 },
            FmSlotState { slot: 1, ip: 1024, oop: 128, ip_role: 32, oop_role_raw: 32_768, person: 101 },
            FmSlotState { slot: 2, ip: 16_384, oop: 1024, ip_role: 2_048, oop_role_raw: 1_048_576, person: 102 },
        ];

        let state_ref = std::rc::Rc::new(std::cell::RefCell::new((
            initial_state,
            Vec::<(i32, u64, u64)>::new(),
            Vec::<(i32, i64, bool)>::new(),
            HashMap::<i64, usize>::new(),
            HashMap::<i64, usize>::new(),
        )));
        let st_handle = state_ref.clone();

        let mock = MockTransport::new(move |cmd, _| {
            if cmd == "PING" {
                return Ok("PONG ready".into());
            }
            if cmd == "VERSION" {
                return Ok("OK 0.4.2 27.1.0".into());
            }
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.contains(&FmbIds::PROP_TRAINING_PLAYERS.to_string()) {
                return Ok(format!(
                    "OK {}",
                    json!([
                        ["FM.UI.PersonReference", person_ref(100)],
                        ["FM.UI.PersonReference", person_ref(101)],
                        ["FM.UI.PersonReference", person_ref(102)],
                    ])
                ));
            }
            if cmd.contains(&FmbIds::PERSON_UNIQUE_ID.to_string()) {
                return Ok("OK 1000\x1e1001\x1e1002".into());
            }
            if cmd.contains(&FmbIds::TACTIC_IP_ROLES.to_string()) {
                let borrow = st_handle.borrow();
                let slots_j: Value = Value::Array(
                    borrow.0.iter().map(|s| slot_rec(s.slot, s.ip, s.oop, s.person)).collect(),
                );
                let ip_j: Value = Value::Array(
                    borrow.0.iter().map(|s| role_rec(s.slot, FmbIds::TACTIC_IP_ROLE, s.ip_role)).collect(),
                );
                let oop_j: Value = Value::Array(
                    borrow.0.iter().map(|s| role_rec(s.slot, FmbIds::TACTIC_OOP_ROLE, s.oop_role_raw)).collect(),
                );
                return Ok(format!("OK {slots_j}\x1e{ip_j}\x1e{oop_j}"));
            }
            if cmd.contains(&FmbIds::PROP_SELECTION.to_string()) {
                let borrow = st_handle.borrow();
                let slots_j: Value = Value::Array(
                    borrow.0.iter().map(|s| slot_rec(s.slot, s.ip, s.oop, s.person)).collect(),
                );
                return Ok(format!("OK \"Tac\"\x1e{slots_j}\x1e[]"));
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_MOVE)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let from_r = FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_MOVE_FROM);
                let to_r = FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_MOVE_TO);
                let slot = FmbClient::<MockTransport>::num_i32(
                    FmbClient::<MockTransport>::field(from_r, FmbIds::TACTIC_MOVE_SLOT),
                    -1,
                );
                let from_b = FmbClient::<MockTransport>::num_u64(
                    FmbClient::<MockTransport>::field(from_r, FmbIds::TACTIC_MOVE_FROM),
                )
                .unwrap();
                let to_b = FmbClient::<MockTransport>::num_u64(
                    FmbClient::<MockTransport>::field(to_r, FmbIds::TACTIC_MOVE_TO),
                )
                .unwrap();
                let is_ip = FmbClient::<MockTransport>::field(from_r, FmbIds::TACTIC_MOVE_IP)
                    .and_then(Value::as_str)
                    == Some("True");
                let mut borrow = st_handle.borrow_mut();
                if let Some(fs) = borrow.0.iter_mut().find(|s| s.slot == slot) {
                    if is_ip {
                        fs.ip = to_b;
                    } else {
                        fs.oop = to_b;
                    }
                }
                if !is_ip {
                    borrow.1.push((slot, from_b, to_b));
                }
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_SET_ROLE)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let slot = FmbClient::<MockTransport>::num_i32(
                    FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_SLOT_IDX),
                    -1,
                );
                let is_ip = FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_ROLE_IP)
                    .and_then(Value::as_str)
                    == Some("True");
                let r1 = FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_ROLE_REC);
                let r2 = FmbClient::<MockTransport>::field(r1, FmbIds::KEY_PAYLOAD);
                let bits = FmbClient::<MockTransport>::num_u64(
                    FmbClient::<MockTransport>::field(r2, FmbIds::TACTIC_ROLE_REC),
                )
                .unwrap();
                let mut borrow = st_handle.borrow_mut();
                if let Some(fs) = borrow.0.iter_mut().find(|s| s.slot == slot) {
                    if is_ip {
                        fs.ip_role = bits;
                    } else {
                        // Simulate FM adding side flag bit 2^22 on OOP role readback!
                        fs.oop_role_raw = bits | 4_194_304;
                    }
                }
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_PI_PREV)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let slot = FmbClient::<MockTransport>::num_i32(
                    FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_SLOT_IDX),
                    -1,
                );
                let cat = FmbClient::<MockTransport>::num_i64(
                    FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_PI_CAT),
                )
                .unwrap();
                let is_ip = FmbClient::<MockTransport>::field(Some(&v), FmbIds::TACTIC_ROLE_IP)
                    .and_then(Value::as_str)
                    == Some("True");
                st_handle.borrow_mut().2.push((slot, cat, is_ip));
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_TI_PREV)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let id: i64 = v[1].as_str().unwrap().parse().unwrap();
                *st_handle.borrow_mut().4.entry(id).or_insert(0) += 1;
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::TACTIC_TI_NEXT)) {
                let json_str = cmd.splitn(5, ' ').nth(4).unwrap();
                let v: Value = serde_json::from_str(json_str).unwrap();
                let id: i64 = v[1].as_str().unwrap().parse().unwrap();
                *st_handle.borrow_mut().3.entry(id).or_insert(0) += 1;
                return Ok("OK".into());
            }
            if cmd.starts_with("SENDJ ") || cmd.starts_with("WRITE ") {
                return Ok("OK".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut ti = BTreeMap::new();
        ti.insert("passing".to_string(), 4); // FM id 1, count 5, target 4
        ti.insert("reception".to_string(), 0); // FM id 20, count 3, map [2,1,0] -> target 2
        ti.insert("set_pieces".to_string(), 1); // FM id 6, map [Some(0), None, Some(1)] -> skipped!

        let tactic = Tactic {
            name: "Cycle Test".into(),
            ment: Mentality::Attacking,
            ip_form: "Custom".into(),
            oop_form: "Custom".into(),
            // Slot 1 (IP=MC) wants OOP=MC (currently held by slot 2!)
            // Slot 2 (IP=ST) wants OOP=DM (currently held by slot 1!) -> 2-cycle!
            slots: sample_tactic_slots(&["GK", "MC", "ST"], &["GK", "MC", "DM"]),
            ti,
            ..Default::default()
        };
        let squad = vec![
            make_player("p0", "1000", "GK", Foot::R, &[]),
            make_player("p1", "1001", "Mid", Foot::R, &[]),
            make_player("p2", "1002", "Fwd", Foot::L, &[]),
        ];

        let mut client = FmbClient::new(mock, Lang::En);
        let lines = client
            .apply_tactic(&tactic, &squad, ApplyTacticOptions::default(), |_| {})
            .unwrap();

        let borrow = state_ref.borrow();
        let state = &borrow.0;
        let oop_moves = &borrow.1;
        let pi_presses = &borrow.2;
        let ti_next_counts = &borrow.3;
        let ti_prev_counts = &borrow.4;

        // Cycle was broken via a spare slot (3 total OOP moves: 1->spare, 2->128, 1->1024).
        assert_eq!(oop_moves.len(), 3);
        assert_eq!(state[1].oop, 1024);
        assert_eq!(state[2].oop, 128);

        // 64-bit role masks applied & verified (including stripping OOP side flag 2^22).
        assert_eq!(state[1].ip_role, 144_115_188_075_855_872); // mp
        assert_eq!(state[2].ip_role, 1_099_511_627_776); // f9
        assert!(lines.iter().any(|l| l.contains("Roles: 3/3 in possession and 3/3 out of possession")));

        // Player instructions on slot 1: Stay wider (2x cat 0 IP) + Press more (2x cat 11 OOP); Hold position reported missing.
        assert_eq!(
            *pi_presses,
            vec![(1, 0, true), (1, 0, true), (1, 11, false), (1, 11, false)]
        );
        assert!(lines.iter().any(|l| l.contains("2 applied") && l.contains("Hold position")));

        // Team instructions: passing (id 1) -> 5 prev, 4 next; reception (id 20) -> 3 prev, 2 next; set_pieces (id 6) -> skipped.
        assert_eq!(ti_prev_counts.get(&1), Some(&5));
        assert_eq!(ti_next_counts.get(&1), Some(&4));
        assert_eq!(ti_prev_counts.get(&20), Some(&3));
        assert_eq!(ti_next_counts.get(&20), Some(&2));
        assert_eq!(ti_prev_counts.get(&6), None);
        assert!(lines.iter().any(|l| l.contains("26 applied") && l.contains("Play for Set Pieces")));
    }

    #[test]
    fn apply_tactic_rejects_old_plugin_version() {
        let mock = MockTransport::from_replies(vec!["PONG ready", "OK 0.4.1 27.1.0"]);
        let mut client = FmbClient::new(mock, Lang::En);
        let err = client
            .apply_tactic(&Tactic::default(), &[], ApplyTacticOptions::default(), |_| {})
            .unwrap_err();
        assert!(err.to_string().contains("0.4.2"));
    }

    #[test]
    fn apply_set_pieces_sets_takers_delivery_and_both_routines() {
        let mut routine_poll = 0usize;
        let routine_json = |kind: i32, side: i32, person: i64, cur_opt: i32, opts: &[(i32, &str)]| {
            let opt_arr: Vec<Value> = opts
                .iter()
                .map(|&(id, lbl)| {
                    json!([
                        "SI.Bindable.DynamicReference",
                        {
                            "type": "SI.Bindable.DynamicReference",
                            "entries": [[
                                FmbIds::SP_ROUTINE_OPT_ITEM,
                                "SI.Bindable.DynamicReference",
                                {
                                    "type": "SI.Bindable.DynamicReference",
                                    "entries": [
                                        [FmbIds::KEY_PAYLOAD, "System.String", lbl],
                                        [FmbIds::KEY_NUMBER, "SI.Bindable.DynamicReference", {
                                            "type": "SI.Bindable.DynamicReference",
                                            "entries": [[FmbIds::SP_ROUTINE_OPTION, "SI.Core.DynamicNumber", id.to_string()]]
                                        }]
                                    ]
                                }
                            ]]
                        }
                    ])
                })
                .collect();
            json!({
                "type": "SI.Bindable.DynamicReference",
                "entries": [
                    [FmbIds::SP_ROUTINE_KIND, "SI.Core.DynamicNumber", kind.to_string()],
                    [FmbIds::SP_ROUTINE_SIDE, "SI.Core.DynamicNumber", side.to_string()],
                    [FmbIds::SP_ROUTINE_IN_BOX, "System.Collections.Generic.List`1", [[
                        "SI.Bindable.DynamicReference",
                        {
                            "type": "SI.Bindable.DynamicReference",
                            "entries": [
                                [FmbIds::SP_ROUTINE_GROUP, "SI.Core.DynamicNumber", "1"],
                                [FmbIds::SP_ROUTINE_SLOTS, "System.Collections.Generic.List`1", [[
                                    "SI.Bindable.DynamicReference",
                                    {
                                        "type": "SI.Bindable.DynamicReference",
                                        "entries": [
                                            [FmbIds::SP_ROUTINE_SLOT, "SI.Core.DynamicNumber", "0"],
                                            [FmbIds::SP_ROUTINE_NAME, "System.String", "a1"],
                                            [FmbIds::SP_ROUTINE_PERSON, "FM.UI.PersonReference", person_ref(person)],
                                            [FmbIds::SP_ROUTINE_CURRENT, "SI.Bindable.DynamicReference", {
                                                "type": "SI.Bindable.DynamicReference",
                                                "entries": [
                                                    [FmbIds::KEY_PAYLOAD, "System.String", "Go Forward"],
                                                    [FmbIds::KEY_NUMBER, "SI.Bindable.DynamicReference", {
                                                        "type": "SI.Bindable.DynamicReference",
                                                        "entries": [[FmbIds::SP_ROUTINE_OPTION, "SI.Core.DynamicNumber", cur_opt.to_string()]]
                                                    }]
                                                ]
                                            }],
                                            [FmbIds::SP_ROUTINE_OPTIONS, "System.Collections.Generic.List`1", opt_arr],
                                        ]
                                    }
                                ]]]
                            ]
                        }
                    ]]],
                    [FmbIds::SP_ROUTINE_OUT_BOX, "System.Collections.Generic.List`1", []],
                ]
            })
        };

        let mock = MockTransport::new(move |cmd, _| {
            if cmd == "PING" {
                return Ok("PONG ready".into());
            }
            if cmd == "VERSION" {
                return Ok("OK 0.4.2 27.1.0".into());
            }
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.contains(&FmbIds::PROP_TRAINING_PLAYERS.to_string()) {
                return Ok(format!(
                    "OK {}",
                    json!([
                        ["FM.UI.PersonReference", person_ref(501)],
                        ["FM.UI.PersonReference", person_ref(502)],
                    ])
                ));
            }
            if cmd.contains(&FmbIds::PERSON_UNIQUE_ID.to_string()) {
                return Ok("OK 3001\x1e3002".into());
            }
            if cmd.starts_with("READAT auto ") {
                routine_poll += 1;
                return match routine_poll {
                    1 => Ok(format!(
                        "OK {}",
                        routine_json(
                            FmbIds::SP_KIND_ATT_CORNER,
                            1,
                            501,
                            4,
                            &[(4, "Go Forward"), (6, "Attack Far Post")],
                        )
                    )),
                    _ => Ok(format!(
                        "OK {}",
                        routine_json(
                            FmbIds::SP_KIND_DEF_CORNER,
                            2,
                            502,
                            1,
                            &[(1, "Go Back"), (7, "Zonally Mark Near Post")],
                        )
                    )),
                };
            }
            if cmd.starts_with("SENDJ ") || cmd.starts_with("WRITE ") {
                return Ok("OK".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let squad = vec![
            make_player(
                "p1",
                "3001",
                "Diogo Costa Silva",
                Foot::L,
                &[(Attr::Cor, 16.0), (Attr::Cro, 15.0), (Attr::Fre, 15.0)],
            ),
            make_player(
                "p2",
                "3002",
                "Virgil van Dijk",
                Foot::R,
                &[(Attr::Cor, 12.0), (Attr::Hea, 18.0), (Attr::Fre, 11.0)],
            ),
        ];
        let plan = SetPiecePlan {
            xi: vec![
                XiMember { player_id: "p1".into(), player_name: "Diogo Costa Silva".into(), code: "DL".into(), group: PosGroup::DLr, side: crate::types::Side::L },
                XiMember { player_id: "p2".into(), player_name: "Virgil van Dijk".into(), code: "DC".into(), group: PosGroup::Dc, side: crate::types::Side::C },
            ],
            corner_right: CornerTaker { player_id: "p1".into(), player_name: "Diogo Costa Silva".into(), code: "DL".into(), swing: None },
            corner_left: CornerTaker { player_id: "p2".into(), player_name: "Virgil van Dijk".into(), code: "DC".into(), swing: None },
            fk_direct: None,
            fk_cross: None,
            pens: vec![],
            long_throw: None,
            caps: vec![],
            attack: vec![SetPieceRole { key: "far".into(), label: "Far post".into(), player_id: "p1".into(), player_name: "Diogo Costa Silva".into(), code: "DL".into(), why: String::new() }],
            defend: vec![SetPieceRole { key: "zn".into(), label: "Zone near".into(), player_id: "p2".into(), player_name: "Virgil van Dijk".into(), code: "DC".into(), why: String::new() }],
            system: "Zonal".into(),
            their_air: vec![],
            tips: vec![],
            air_top: 15.5,
        };

        let mut client = FmbClient::new(mock, Lang::En);
        let lines = client.apply_set_pieces(&plan, &squad, |_| {}).unwrap();
        assert!(lines.iter().any(|l| l.contains("Corners: inswinging")));
        assert!(lines.iter().any(|l| l.contains("aim for the far post")));
        assert!(lines.iter().any(|l| l.contains("Attacking corner: Silva: Attack Far Post")));
        assert!(lines.iter().any(|l| l.contains("Defending corner: van Dijk: Zonally Mark Near Post")));
        assert!(
            client
                .transport()
                .sent
                .iter()
                .any(|c| c.starts_with(&format!("SENDJ {} ", FmbIds::SP_ROUTINE_EV)))
        );
    }

    #[test]
    fn read_squad_parses_live_players_attributes_foot_and_positions() {
        let attr_item = |fm_id: i64, val: i64| {
            json!([
                "SI.Bindable.DynamicReference",
                {
                    "type": "SI.Bindable.DynamicReference",
                    "entries": [
                        [FmbIds::PERSON_ATTR_INFO_BOX, "SI.Bindable.DynamicReference", {
                            "type": "SI.Bindable.DynamicReference",
                            "entries": [[FmbIds::PERSON_ATTR_ID, "SI.Core.DynamicNumber", fm_id.to_string()]]
                        }],
                        [FmbIds::PERSON_ATTR_VAL_BOX, "SI.Bindable.DynamicReference", {
                            "type": "SI.Bindable.DynamicReference",
                            "entries": [[FmbIds::PERSON_ATTR_VAL, "SI.Core.DynamicNumber", val.to_string()]]
                        }],
                    ]
                }
            ])
        };
        let pos_item = |bit: u64, rating: i64| {
            json!([
                "SI.Bindable.DynamicReference",
                {
                    "type": "SI.Bindable.DynamicReference",
                    "entries": [
                        [FmbIds::PERSON_POS_BIT, "SI.Core.DynamicNumber", bit.to_string()],
                        [FmbIds::KEY_NUMBER, "SI.Core.DynamicNumber", rating.to_string()],
                    ]
                }
            ])
        };

        let foot_rec = json!({
            "type": "SI.Bindable.DynamicReference",
            "entries": [
                [FmbIds::PERSON_FOOT_LEFT, "SI.Core.DynamicNumber", "18"],
                [FmbIds::PERSON_FOOT_RIGHT, "SI.Core.DynamicNumber", "16"],
            ]
        });
        let positions = json!([pos_item(16_384, 20), pos_item(4_096, 16)]);
        let phys = json!([attr_item(42, 17), attr_item(39, 16)]); // pac=17, acc=16
        let tech = json!([attr_item(13, 18), attr_item(18, 15)]); // fin=18, pas=15

        let mock = MockTransport::from_replies(vec![
            "PONG ready",
            "OK 0.4.2 27.1.0",
            "OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20",
            &format!(
                "OK \"\\u0001fmt\\u0002Sporting CP\\u0001\"\x1e{}",
                json!([["FM.UI.PersonReference", person_ref(700)]])
            ),
            &format!(
                "OK \"\\u0001c\\u0002Viktor Gyokeres\\u0001\"\x1e987654\x1e26\x1e187\x1e{foot_rec}\x1e{positions}\x1e{phys}\x1e{tech}\x1e[]\x1e[]\x1e[]"
            ),
        ]);

        let mut client = FmbClient::new(mock, Lang::En);
        let res = client.read_squad(|_| {}).unwrap();
        assert_eq!(res.team, 20);
        assert_eq!(res.club, "Sporting CP");
        assert_eq!(res.players.len(), 1);
        let p = &res.players[0];
        assert_eq!(p.id, "u987654");
        assert_eq!(p.name, "Viktor Gyokeres");
        assert_eq!(p.age, Some(26.0));
        assert_eq!(p.height, Some(187.0));
        assert_eq!(p.foot, Some(Foot::B));
        assert_eq!(p.attr(Attr::Fin), Some(18.0));
        assert_eq!(p.attr(Attr::Pac), Some(17.0));
        assert!(p.pos.contains(&"ST".to_string()));
        assert!(p.pos.contains(&"AM_LR:L".to_string()));
        assert_eq!(p.nat_pos, vec!["ST".to_string()]);
    }

    #[test]
    fn next_opponent_and_push_shortlist_work() {
        let fixture = json!({
            "type": "FM.UI.FixtureReference",
            "pairs": [
                [FmbIds::FIXTURE_US, 20],
                [FmbIds::FIXTURE_OPP, 99],
            ]
        });
        let opp_players = json!([
            ["FM.UI.PersonReference", person_ref(801)],
            ["FM.UI.PersonReference", person_ref(802)],
        ]);

        let mock = MockTransport::from_replies(vec![
            "OK session=1 FM.UI.CalendarReference=1214805357=10;1415930221=20",
            &format!("OK {fixture}"),
            &format!("OK \"\\u0001x\\u0002RB Salzburg\\u0001\"\x1e{opp_players}"),
            "OK 55501\x1e55502",
        ]);
        let mut client = FmbClient::new(mock, Lang::En);
        let opp = client.next_opponent().unwrap().unwrap();
        assert_eq!(opp.team, 99);
        assert_eq!(opp.name, "RB Salzburg");
        assert_eq!(opp.uniqs, vec!["55501", "55502"]);

        // Push shortlist: one found by unique_id, one missing.
        let search_hits = json!([
            ["FM.UI.SearchResultReference", person_ref(910)],
            ["FM.UI.SearchResultReference", person_ref(911)],
        ]);
        let mut search_calls = 0usize;
        let mock_sl = MockTransport::new(move |cmd, _| {
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.starts_with("WRITE FM.UI.SearchReference") {
                search_calls += 1;
                return Ok("OK".into());
            }
            if cmd.starts_with("READ FM.UI.SearchReference") {
                if search_calls == 1 {
                    return Ok(format!("OK {search_hits}"));
                }
                return Ok("OK []".into());
            }
            if cmd.contains(&FmbIds::SEARCH_UNIQUE_ID.to_string()) {
                return Ok("OK 1111\x1e2222".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::ACTION_EV)) {
                return Ok("OK".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut client_sl = FmbClient::new(mock_sl, Lang::En);
        let list = vec![
            ShortlistEntry { name: "Wonderkid".into(), uniq: Some("2222".into()), ..Default::default() },
            ShortlistEntry { name: "Ghost".into(), uniq: Some("9999".into()), ..Default::default() },
        ];
        let res = client_sl.push_shortlist(&list, |_, _| {}).unwrap();
        assert_eq!(res.added, vec!["Wonderkid"]);
        assert_eq!(res.missing, vec!["Ghost"]);
    }

    #[test]
    fn create_focus_steps_stars_adds_areas_and_confirms() {
        let mut ca_level = 14i64; // starts at 2.0 stars -> target 3.0 stars = 16 (2 up steps)
        let mut pa_level = 18i64; // starts at 4.0 stars -> target 3.5 stars = 17 (1 down step)
        let mut confirmed = false;

        let mock = MockTransport::new(move |cmd, _| {
            if cmd == "REFS" {
                return Ok("OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20".into());
            }
            if cmd.contains(&FmbIds::FOCUS_AREAS.to_string()) {
                return Ok("OK []".into());
            }
            if cmd.contains(&FmbIds::FOCUS_LIST.to_string()) {
                let list = if confirmed {
                    json!([
                        ["FM.UI.RecruitmentFocusItemReference", { "pairs": [[FmbIds::FOCUS_ID, 1]] }],
                        ["FM.UI.RecruitmentFocusItemReference", { "pairs": [[FmbIds::FOCUS_ID, 2]] }],
                    ])
                } else {
                    json!([
                        ["FM.UI.RecruitmentFocusItemReference", { "pairs": [[FmbIds::FOCUS_ID, 1]] }],
                    ])
                };
                return Ok(format!("OK {list}"));
            }
            if cmd.contains(&format!("|{}", FmbIds::FOCUS_NAME)) && cmd.starts_with("READ ") {
                return Ok(if confirmed {
                    "OK \"Existing\"\x1e\"U21 South America\"".into()
                } else {
                    "OK \"Existing\"".into()
                });
            }
            if cmd.contains(&format!("|{}", FmbIds::FOCUS_CA)) {
                let rec = json!({ "entries": [[FmbIds::FOCUS_LEVEL, "SI.Core.DynamicNumber", ca_level.to_string()]] });
                return Ok(format!("OK {rec}"));
            }
            if cmd.contains(&format!("|{}", FmbIds::FOCUS_PA)) {
                let rec = json!({ "entries": [[FmbIds::FOCUS_LEVEL, "SI.Core.DynamicNumber", pa_level.to_string()]] });
                return Ok(format!("OK {rec}"));
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::FOCUS_CA_STEP)) {
                if cmd.ends_with("\"1\"]") { ca_level += 1; } else { ca_level -= 1; }
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::FOCUS_PA_STEP)) {
                if cmd.ends_with("\"1\"]") { pa_level += 1; } else { pa_level -= 1; }
                return Ok("OK".into());
            }
            if cmd.starts_with(&format!("SENDJ {} ", FmbIds::FOCUS_CONFIRM)) {
                confirmed = true;
                return Ok("OK".into());
            }
            if cmd.starts_with("WRITE ") || cmd.starts_with("SENDJ ") {
                return Ok("OK".into());
            }
            Err(BridgeError::Protocol(format!("unhandled: {cmd}")))
        });

        let mut client = FmbClient::new(mock, Lang::En);
        let req = BridgeFocusRequest {
            name: "U21 South America Wonderkids Extra Long Title".into(),
            age_min: 16,
            age_max: 21,
            ca_stars: 3.0,
            pa_stars: 3.5,
            areas: vec![BridgeFocusArea {
                id: FmbIds::focus_nation(1651),
                label: "Brazil".into(),
            }],
        };
        let made = client.create_focus(&req, |_| {}).unwrap();
        assert_eq!(made.id, 2);
        assert_eq!(made.name, "U21 South America");
        // Verify name was truncated to 25 chars when written.
        assert!(client
            .transport()
            .sent
            .iter()
            .any(|c| c.contains("\"U21 South America Wonderk\"")));
    }

    #[test]
    fn create_focus_refuses_when_half_created_areas_exist() {
        let mock = MockTransport::from_replies(vec![
            "OK session=1 FM.UI.TeamTrainingReference=1348620377=10;1416914273=20",
            "OK [[\"SI.Bindable.DynamicReference\",{}]]",
        ]);
        let mut client = FmbClient::new(mock, Lang::En);
        let req = BridgeFocusRequest {
            name: "Test".into(),
            age_min: 18,
            age_max: 23,
            ca_stars: 2.5,
            pa_stars: 4.0,
            areas: vec![],
        };
        let err = client.create_focus(&req, |_| {}).unwrap_err();
        assert!(err.to_string().contains("half-created"));
    }

    #[test]
    fn read_chunks_over_60_and_handles_err_entries() {
        let mut chunk_sizes = Vec::new();
        let sizes_ref = std::rc::Rc::new(std::cell::RefCell::new(Vec::<usize>::new()));
        let sizes_handle = sizes_ref.clone();
        let mock = MockTransport::new(move |cmd, _| {
            assert!(cmd.starts_with("READ "));
            let count = cmd["READ ".len()..].split(";;").count();
            sizes_handle.borrow_mut().push(count);
            let reply = (0..count)
                .map(|i| if i == 1 { "ERR missing".to_string() } else { format!("{i}") })
                .collect::<Vec<_>>()
                .join("\x1e");
            Ok(format!("OK {reply}"))
        });
        let mut client = FmbClient::new(mock, Lang::En);
        let pairs = FmbIds::person_pairs(1);
        let reqs: Vec<(&str, &str, i64)> = (0..65)
            .map(|_| ("PersonReference", pairs.as_str(), FmbIds::PERSON_UNIQUE_ID))
            .collect();
        let vals = client.read(&reqs).unwrap();
        chunk_sizes.extend(sizes_ref.borrow().iter().copied());
        assert_eq!(chunk_sizes, vec![60, 5]);
        assert_eq!(vals.len(), 65);
        assert_eq!(vals[0], Some(json!(0)));
        assert_eq!(vals[1], None);
        assert_eq!(vals[60], Some(json!(0)));
    }
}
