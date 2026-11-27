//! FM27 `.fm` save discovery and `fmsave` export (managed squad + cached scouting DB).
//!
//! Always operates on a temporary copy of the `.fm` save — never writes to or modifies
//! the user's save files.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Metadata about a discovered `.fm` save file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveInfo {
    pub path: String,
    pub name: String,
    pub modified: u64,
    pub size: u64,
}

/// Output of exporting the managed club's squad CSV from a `.fm` save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveExport {
    pub save: String,
    pub modified: u64,
    pub content: String,
    pub runner: String,
}

/// Output of exporting the entire save player database + nations + save info JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDb {
    pub save: String,
    pub modified: u64,
    pub players: String,
    pub nations: String,
    pub info: String,
    pub runner: String,
    pub cached: bool,
}

/// Ordered fallback runners for invoking `fmsave`.
/// 1st: latest PyPI via `uvx`, 2nd: cached `uvx`, then direct CLI / Python module installs.
pub const FMSAVE_RUNNERS: [(&str, &[&str]); 5] = [
    ("uvx", &["fmsave@latest"]),
    ("uvx", &["fmsave"]),
    ("fmsave", &[]),
    ("py", &["-m", "fmsave"]),
    ("python", &["-m", "fmsave"]),
];

/// Matches `SCOUT_COLUMNS` in the scouting engine. If `fmsave` rejects any column name,
/// `export_save_db` automatically falls back to exporting all columns.
pub const DB_COLUMNS: &str = "uid,unique_id,name,age,nation_id,height_cm,ability_current,ability_potential,ability_potential_range_code,reputation_current,reputation_world,club_uid,club_name,club_nation_id,club_reputation,on_loan,loan_parent_club_name,transfer_value,contract_wage,contract_end,contract_squad_status,left_foot,right_foot,positions_gk,positions_dl,positions_dc,positions_dr,positions_dm,positions_ml,positions_mc,positions_mr,positions_aml,positions_amc,positions_amr,positions_stc,positions_wbl,positions_wbr,personality_ambition,personality_professionalism,personality_pressure,personality_loyalty,personality_temperament,personality_adaptability,personality_sportsmanship,personality_controversy,attributes_crossing,attributes_dribbling,attributes_finishing,attributes_heading,attributes_long_shots,attributes_marking,attributes_off_the_ball,attributes_passing,attributes_penalty_taking,attributes_tackling,attributes_vision,attributes_handling,attributes_aerial_reach,attributes_command_of_area,attributes_communication,attributes_kicking,attributes_throwing,attributes_anticipation,attributes_decisions,attributes_one_on_ones,attributes_positioning,attributes_reflexes,attributes_first_touch,attributes_technique,attributes_flair,attributes_corners,attributes_teamwork,attributes_work_rate,attributes_long_throws,attributes_eccentricity,attributes_rushing_out,attributes_punching,attributes_acceleration,attributes_free_kick_taking,attributes_strength,attributes_stamina,attributes_pace,attributes_jumping_reach,attributes_leadership,attributes_dirtiness,attributes_balance,attributes_bravery,attributes_consistency,attributes_aggression,attributes_agility,attributes_important_matches,attributes_injury_proneness,attributes_versatility,attributes_natural_fitness,attributes_determination,attributes_composure,attributes_concentration";

fn modified_secs(p: &Path) -> u64 {
    fs::metadata(p)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Standard FM27 save directories across Windows (Documents / Documentos / OneDrive) and macOS.
pub fn save_dirs(custom: Option<&str>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if let Some(c) = custom {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            out.push(PathBuf::from(trimmed));
        }
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    for var in ["USERPROFILE", "HOME"] {
        if let Ok(h) = std::env::var(var) {
            for docs in ["Documents", "Documentos"] {
                roots.push(PathBuf::from(&h).join(docs));
                roots.push(PathBuf::from(&h).join("OneDrive").join(docs));
            }
            roots.push(PathBuf::from(&h).join("Library").join("Application Support"));
        }
    }
    if let Ok(od) = std::env::var("OneDrive") {
        for docs in ["Documents", "Documentos"] {
            roots.push(PathBuf::from(&od).join(docs));
        }
    }
    for r in roots {
        let p = r
            .join("Sports Interactive")
            .join("Football Manager 27")
            .join("games");
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// Returns the most recently modified `.fm` file across the given directories.
pub fn latest_save_in_dirs(dirs: &[PathBuf]) -> Option<SaveInfo> {
    let mut best: Option<(u64, PathBuf)> = None;
    for dir in dirs {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let is_fm = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("fm"))
                .unwrap_or(false);
            if !is_fm {
                continue;
            }
            let m = modified_secs(&path);
            let newer = match &best {
                Some((bm, _)) => m > *bm,
                None => true,
            };
            if newer {
                best = Some((m, path));
            }
        }
    }
    best.map(|(m, path)| SaveInfo {
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        size: fs::metadata(&path).map(|md| md.len()).unwrap_or(0),
        path: path.display().to_string(),
        modified: m,
    })
}

/// Returns the most recently modified FM27 `.fm` save in standard (or custom) folders.
pub fn latest_save(custom_dir: Option<&str>) -> Option<SaveInfo> {
    let dirs = save_dirs(custom_dir);
    latest_save_in_dirs(&dirs)
}

fn fmsave_exec(
    args: &[OsString],
    out: Option<&Path>,
    prefer: Option<usize>,
) -> Result<(usize, Vec<u8>), String> {
    let mut order: Vec<usize> = Vec::new();
    if let Some(i) = prefer {
        order.push(i);
    }
    for j in 0..FMSAVE_RUNNERS.len() {
        if Some(j) != prefer {
            order.push(j);
        }
    }
    let mut errors: Vec<String> = Vec::new();
    for i in order {
        let (exe, prefix) = FMSAVE_RUNNERS[i];
        if let Some(o) = out {
            let _ = fs::remove_file(o);
        }
        let mut cmd = Command::new(exe);
        cmd.args(prefix.iter()).args(args.iter());
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        match cmd.output() {
            Ok(o) if o.status.success() && out.map(|p| p.is_file()).unwrap_or(true) => {
                let data = match out {
                    Some(p) => fs::read(p).map_err(|e| e.to_string())?,
                    None => o.stdout,
                };
                return Ok((i, data));
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr).into_owned();
                let last = err
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .last()
                    .unwrap_or("falhou")
                    .to_string();
                errors.push(format!("{} {}: {}", exe, prefix.join(" "), last));
            }
            Err(e) => errors.push(format!("{} {}: {}", exe, prefix.join(" "), e)),
        }
    }
    Err(errors.join(" | "))
}

fn read_lossy(p: &Path) -> String {
    fs::read(p)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

fn cache_temp_dir() -> PathBuf {
    std::env::temp_dir().join("fm27-tactical-bible")
}

/// Runs `fmsave` on a temporary copy of `path` and returns the managed club squad CSV.
pub fn export_save(path: &str) -> Result<SaveExport, String> {
    let src = PathBuf::from(path);
    if !src.is_file() {
        return Err(format!("Save não encontrado: {}", path));
    }
    let tmp = cache_temp_dir();
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let copy = tmp.join("save_copia.fm");
    fs::copy(&src, &copy).map_err(|e| format!("Não consegui copiar o save: {}", e))?;
    let out = tmp.join("plantel.csv");

    let args: Vec<OsString> = vec![
        "export".into(),
        copy.clone().into_os_string(),
        "players".into(),
        "--managed-club".into(),
        "-o".into(),
        out.clone().into_os_string(),
    ];

    let res = fmsave_exec(&args, Some(out.as_path()), None);
    let _ = fs::remove_file(&copy);

    let (ri, bytes) = res.map_err(|e| format!("Não consegui correr o fmsave. {}", e))?;
    let (exe, prefix) = FMSAVE_RUNNERS[ri];
    Ok(SaveExport {
        save: src
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        modified: modified_secs(&src),
        content: String::from_utf8_lossy(&bytes).into_owned(),
        runner: format!("{} {}", exe, prefix.join(" ")).trim().to_string(),
    })
}

/// Runs `fmsave` on a temporary copy of `path` to export the full scouting DB (players + nations + info).
/// Cached on disk by `path|mtime` until the save file changes.
pub fn export_save_db(path: &str) -> Result<SaveDb, String> {
    export_save_db_in_cache(path, &cache_temp_dir())
}

pub(crate) fn export_save_db_in_cache(path: &str, tmp: &Path) -> Result<SaveDb, String> {
    let src = PathBuf::from(path);
    if !src.is_file() {
        return Err(format!("Save não encontrado: {}", path));
    }
    let save_name = src
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let modified = modified_secs(&src);
    fs::create_dir_all(tmp).map_err(|e| e.to_string())?;
    let meta = tmp.join("db.meta");
    let dbf = tmp.join("db_players.csv");
    let natf = tmp.join("db_nations.csv");
    let infof = tmp.join("db_info.json");
    let key = format!("{}|{}", path, modified);

    if fs::read_to_string(&meta).map(|m| m == key).unwrap_or(false) && dbf.is_file() {
        return Ok(SaveDb {
            save: save_name,
            modified,
            players: read_lossy(&dbf),
            nations: read_lossy(&natf),
            info: read_lossy(&infof),
            runner: "cache".to_string(),
            cached: true,
        });
    }

    let copy = tmp.join("save_db_copia.fm");
    fs::copy(&src, &copy).map_err(|e| format!("Não consegui copiar o save: {}", e))?;

    let base: Vec<OsString> = vec![
        "export".into(),
        copy.clone().into_os_string(),
        "players".into(),
        "--all".into(),
    ];
    let mut with_cols = base.clone();
    with_cols.push("--columns".into());
    with_cols.push(DB_COLUMNS.into());
    with_cols.push("-o".into());
    with_cols.push(dbf.clone().into_os_string());

    let res = fmsave_exec(&with_cols, Some(dbf.as_path()), None).or_else(|_| {
        let mut all = base.clone();
        all.push("-o".into());
        all.push(dbf.clone().into_os_string());
        fmsave_exec(&all, Some(dbf.as_path()), None)
    });

    let (ri, data) = match res {
        Ok(x) => x,
        Err(e) => {
            let _ = fs::remove_file(&copy);
            return Err(format!("Não consegui correr o fmsave. {}", e));
        }
    };

    let players = String::from_utf8_lossy(&data).into_owned();
    let nat_args: Vec<OsString> = vec![
        "export".into(),
        copy.clone().into_os_string(),
        "nations".into(),
        "--all".into(),
        "-o".into(),
        natf.clone().into_os_string(),
    ];
    let nations = fmsave_exec(&nat_args, Some(natf.as_path()), Some(ri))
        .map(|(_, d)| String::from_utf8_lossy(&d).into_owned())
        .unwrap_or_default();

    let info_args: Vec<OsString> = vec![
        "info".into(),
        copy.clone().into_os_string(),
        "--json".into(),
    ];
    let info = fmsave_exec(&info_args, None, Some(ri))
        .map(|(_, d)| String::from_utf8_lossy(&d).into_owned())
        .unwrap_or_default();

    let _ = fs::write(&infof, &info);
    let _ = fs::write(&meta, &key);
    let _ = fs::remove_file(&copy);

    let (exe, prefix) = FMSAVE_RUNNERS[ri];
    Ok(SaveDb {
        save: save_name,
        modified,
        players,
        nations,
        info,
        runner: format!("{} {}", exe, prefix.join(" ")).trim().to_string(),
        cached: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, SystemTime};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "fm27_backend_save_test_{}_{}_{}",
            label,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_dirs_puts_custom_first_and_trims() {
        let dirs = save_dirs(Some("  /tmp/my_saves  "));
        assert_eq!(dirs[0], PathBuf::from("/tmp/my_saves"));
        assert!(dirs.len() >= 1);
    }

    #[test]
    fn latest_save_in_dirs_picks_newest_fm_file_only() {
        let dir = unique_temp_dir("latest");
        let ignored = dir.join("notes.txt");
        let older = dir.join("career_old.fm");
        let newer = dir.join("career_new.FM");

        fs::write(&ignored, b"ignore me").unwrap();
        fs::write(&older, b"old save bytes").unwrap();
        // Ensure distinct mtime on filesystems with 1-second granularity.
        thread::sleep(Duration::from_millis(1100));
        fs::write(&newer, b"newer save bytes!").unwrap();

        let found = latest_save_in_dirs(&[dir.clone()]).expect("should find newest .fm");
        assert_eq!(found.name, "career_new.FM");
        assert_eq!(found.size, 17);
        assert!(found.path.ends_with("career_new.FM"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_save_db_uses_cache_when_meta_key_matches() {
        let dir = unique_temp_dir("cache");
        let save_file = dir.join("test.fm");
        fs::write(&save_file, b"dummy fm save").unwrap();

        let cache = dir.join("cache");
        fs::create_dir_all(&cache).unwrap();
        let mtime = modified_secs(&save_file);
        let save_str = save_file.to_string_lossy().into_owned();

        fs::write(cache.join("db.meta"), format!("{}|{}", save_str, mtime)).unwrap();
        fs::write(cache.join("db_players.csv"), "uid,name\n1,Player One\n").unwrap();
        fs::write(cache.join("db_nations.csv"), "id,name\n1,Portugal\n").unwrap();
        fs::write(cache.join("db_info.json"), r#"{"date":"2027-08-01"}"#).unwrap();

        let db = export_save_db_in_cache(&save_str, &cache).expect("cache hit should succeed");
        assert!(db.cached);
        assert_eq!(db.runner, "cache");
        assert_eq!(db.save, "test.fm");
        assert!(db.players.contains("Player One"));
        assert!(db.nations.contains("Portugal"));
        assert!(db.info.contains("2027-08-01"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_save_errors_on_missing_file() {
        let err = export_save("/nonexistent/path/to/missing.fm").unwrap_err();
        assert!(err.contains("Save não encontrado"));
    }
}
