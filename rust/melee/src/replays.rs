//! `melee replays [DIR] [--json] [--play N [play options]]`: the replay browser (docs/rust-port/notes/spectate-replay.md).
//!
//! Lists the `.slp` (Slippi, read with melee_data::slp) and `.mrpl` (this engine's replay, play_net::Replay) files of a
//! folder (default `MELEE_SLP_DIR`, else `Replays`), newest first, with characters, stage, date and duration. `--play N`
//! plays entry N: a `.mrpl` through `melee play --replay` (pause / step / seek / speed / camera: replay_ctl.rs), a `.slp`
//! through the sim's Slippi player (`melee golden --slp`, headless, hash per frame; it plays only what the sim covers).
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CHARS: [&str; 26] = [
    "Falcon", "DK", "Fox", "G&W", "Kirby", "Bowser", "Link", "Luigi", "Mario", "Marth", "Mewtwo", "Ness", "Peach", "Pikachu", "ICs", "Puff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco", "YLink", "Doc", "Roy", "Pichu", "Ganon",
];

pub struct Entry {
    pub path: PathBuf,
    pub kind: &'static str,
    pub chars: Vec<String>,
    pub stage: String,
    pub date: String,
    pub secs: f64,
    pub note: String,
    mtime: u64,
}

fn stage_name(ext: i32) -> String {
    crate::play::STAGES.iter().find(|s| s.0 == ext).map_or(format!("stage {ext}"), |s| s.1.to_string())
}

fn char_name(ck: i32) -> String {
    CHARS.get(ck as usize).map_or(format!("ck{ck}"), |s| s.to_string())
}

/// Seconds since 1970 as `YYYY-MM-DD HH:MM` (UTC).
fn date_of(t: u64) -> String {
    let days = (t / 86400) as i64;
    let (h, m) = ((t % 86400) / 3600, (t % 3600) / 60);
    // civil from days (H. Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (mo <= 2) as i64;
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}")
}

/// The UBJSON metadata's `startAt` string (`S U <len> text`), if present.
fn slp_start_at(d: &[u8]) -> Option<String> {
    let k = d.windows(7).rposition(|w| w == b"startAt")?;
    let r = d.get(k + 7..)?;
    if r.first() != Some(&b'S') || r.get(1) != Some(&b'U') {
        return None;
    }
    let n = *r.get(2)? as usize;
    let s = std::str::from_utf8(r.get(3..3 + n)?).ok()?;
    Some(s.replace('T', " ").chars().take(16).collect())
}

pub fn read_entry(path: &Path) -> Option<Entry> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let mtime = std::fs::metadata(path).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let d = std::fs::read(path).ok()?;
    let mut e = Entry { path: path.to_path_buf(), kind: "", chars: Vec::new(), stage: "?".into(), date: date_of(mtime), secs: 0.0, note: String::new(), mtime };
    match ext.as_str() {
        "slp" => {
            e.kind = "slp";
            match melee_data::slp::parse(&d, false) {
                Ok(s) => {
                    if let Some(g) = &s.info {
                        e.stage = stage_name(g.stage() as i32);
                        for i in 0..4 {
                            let p = g.player(i);
                            if p.ptype < 3 {
                                e.chars.push(format!("P{} {}{}", i + 1, char_name(p.ckind as i32), if p.ptype == 1 { " (CPU)" } else { "" }));
                            }
                        }
                    }
                    e.secs = (s.last_frame - s.first_frame + 1).max(0) as f64 / 60.0;
                    if let Some(t) = slp_start_at(&d) {
                        e.date = t;
                    }
                    e.note = format!("Slippi {}{}", s.version_str(), if s.truncated { ", cut short" } else { "" });
                }
                Err(er) => e.note = format!("unreadable: {er}"),
            }
        }
        "mrpl" => {
            e.kind = "mrpl";
            match crate::play_net::Replay::parse(&d) {
                Ok(r) => {
                    let frames = r.ticks.iter().filter(|t| t.1 != 0).count();
                    e.secs = frames as f64 / 60.0;
                    if let Some(sc) = r.meta("np_scene").and_then(melee_net::app::scene::parse_scene) {
                        use melee_net::app::room::RoomEnv;
                        let env = crate::play_net::PlayEnv::default();
                        e.chars = vec![format!("P1 {}", char_name(env.ck_for_token(&sc.host_tok))), format!("P2 {}", char_name(env.ck_for_token(&sc.guest_tok)))];
                        e.stage = stage_name(melee_net::app::lobby::LobbyEnv::ext_for_token(&env, &sc.stage_tok));
                    } else if let Some(i) = r.args.iter().position(|a| a == "--match") {
                        let spec = r.args.get(i + 1).cloned().unwrap_or_default();
                        for (k, part) in spec.split(',').enumerate() {
                            match crate::play::STAGES.iter().find(|s| s.0.to_string() == part || s.1.eq_ignore_ascii_case(part)) {
                                Some(s) => e.stage = s.1.to_string(),
                                None => e.chars.push(format!("P{} {part}", k + 1)),
                            }
                        }
                    } else if r.meta("np_css").is_some() {
                        e.stage = "picked on the CSS".into();
                    } else {
                        e.stage = "menus".into();
                    }
                    if let Some(t) = path.file_stem().and_then(|s| s.to_str()).and_then(|s| s.strip_prefix("melee-")).and_then(|s| s.parse::<u64>().ok()) {
                        e.date = date_of(t);
                    }
                    e.note = format!("{} ticks{}{}", r.ticks.len(), r.meta("np_role").map_or(String::new(), |x| format!(", netplay {x}")), if r.flags & 1 != 0 { ", seekable" } else { "" });
                }
                Err(er) => e.note = format!("unreadable: {er}"),
            }
        }
        _ => return None,
    }
    Some(e)
}

pub fn list(dir: &Path) -> Vec<Entry> {
    let mut v: Vec<Entry> = std::fs::read_dir(dir).into_iter().flatten().filter_map(|x| x.ok()).filter_map(|x| read_entry(&x.path())).collect();
    v.sort_by(|a, b| b.mtime.cmp(&a.mtime).then(a.path.cmp(&b.path)));
    v
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

pub fn replays(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let dir = args.get(1).filter(|a| !a.starts_with("--")).cloned().or_else(|| std::env::var("MELEE_SLP_DIR").ok()).unwrap_or_else(|| "Replays".into());
    let v = list(Path::new(&dir));
    if let Some(n) = get("--play") {
        let Some(e) = n.parse::<usize>().ok().and_then(|i| v.get(i.wrapping_sub(1))) else {
            eprintln!("replays: --play {n}: {} entries in {dir}", v.len());
            return ExitCode::from(2);
        };
        let pos = args.iter().position(|a| a == "--play").unwrap();
        let rest: Vec<String> = args[pos + 2..].to_vec();
        let exe = std::env::current_exe().unwrap_or_else(|_| "melee".into());
        let p = e.path.to_string_lossy().to_string();
        let mut cmd = std::process::Command::new(exe);
        if e.kind == "mrpl" {
            cmd.args(["play", "--replay", &p]).args(&rest);
        } else {
            eprintln!("replays: {p}: Slippi playback through the sim's player (headless, `golden --slp`)");
            cmd.args(["golden", "--slp", &p]).args(&rest);
        }
        return match cmd.status() {
            Ok(s) if s.success() => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        };
    }
    if args.iter().any(|a| a == "--json") {
        let items: Vec<String> = v
            .iter()
            .map(|e| {
                format!(
                    "{{\"path\":{},\"kind\":\"{}\",\"chars\":[{}],\"stage\":{},\"date\":{},\"seconds\":{:.2},\"note\":{}}}",
                    json_str(&e.path.to_string_lossy()),
                    e.kind,
                    e.chars.iter().map(|c| json_str(c)).collect::<Vec<_>>().join(","),
                    json_str(&e.stage),
                    json_str(&e.date),
                    e.secs,
                    json_str(&e.note)
                )
            })
            .collect();
        println!("[{}]", items.join(",\n "));
        return ExitCode::SUCCESS;
    }
    println!("{} replays in {dir}", v.len());
    for (i, e) in v.iter().enumerate() {
        let (m, s) = ((e.secs as u64) / 60, (e.secs as u64) % 60);
        println!(
            "{:3}  {:4}  {}  {:>2}:{:02}  {:<22}  {:<28}  {}  ({})",
            i + 1,
            e.kind,
            e.date,
            m,
            s,
            e.stage,
            e.chars.join(" vs "),
            e.path.file_name().map_or(String::new(), |f| f.to_string_lossy().to_string()),
            e.note
        );
    }
    ExitCode::SUCCESS
}
