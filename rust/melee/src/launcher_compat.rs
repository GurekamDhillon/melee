//! The Melee Unlocked launcher seam: run `melee.exe` as a third engine beside the launcher's Source Port and Static
//! Recomp (docs/rust-port/notes/launcher-seam.md).
//!
//! The launcher starts a game engine as `<exe> --iso <disc> --threaded-renderer --settings-path <port-settings.ini>
//! [--sys-dir D --user-dir D --replay-dir D --card-dir D] [--slippi-menus off --vanilla-only [--vanilla-game]]
//! --lobby-status-file <json>`, in the game folder, with no subcommand. This module recognises that command line
//! (the first argument is a `--` flag that only the launcher sends, or `melee play --launcher-compat`) and turns it
//! into a `melee play` run:
//!
//! * the disc is `--iso`;
//! * `port-settings.ini` (`key value` lines) gives the display (`fullscreen`, `exclusivefullscreen`, `widescreen`,
//!   `truewidescreen`, `window`, `scale`, `msaa`, `vsync`) and the master `volume`; each lands in the `MELEE_<KEY>`
//!   variable the display and audio layers read, unless that variable is already set (the environment still wins, then
//!   the launcher's settings, then settings.cfg, then the default). `--volume N` and the display flags of the Source
//!   Port's command line override the file;
//! * `--card-dir <..\User\GC\CardA>` puts our card beside it, in `<..\User\GC>\RustPort` (`MELEE_CARD_PATH`): our
//!   card is a `USA\Card A` tree of .gci files, the launcher's folder holds the Source Port's own files, so the two
//!   never write each other's saves;
//! * the log is `melee_port.log` in the working directory (`MELEE_LOG_FILE`), and a crash also writes
//!   `melee_port_crash.txt` there (`MELEE_CRASH_COPY`, crash.rs): the two files the launcher's crash flow reads;
//! * `MELEE_NO_GC_ADAPTER` (the launcher's test runs) means `MELEE_PAD_IGNORE_ADAPTER=1`.
//!
//! Options with no Rust counterpart that change nothing for an offline game (`--threaded-renderer`, `--sys-dir`,
//! `--user-dir`, `--replay-dir`, `--lobby-status-file`, the vanilla flags) are accepted and logged. Ones that would
//! change the game (mods, peer-to-peer and Slippi lobby matches, replays) are refused with exit code 3 and a line in the
//! log, rather than starting a game other than the one asked for. Anything else is passed to `melee play` as given.
use std::path::{Path, PathBuf};

/// Launcher options that take a value and are accepted without effect.
const IGNORED_VALUE: &[&str] = &["--sys-dir", "--user-dir", "--replay-dir", "--lobby-status-file", "--slippi-menus", "--online-delay", "--language", "--backend", "--fps", "--aspect", "--dlss", "--dlss-mode", "--ssao", "--frame-mode"];
/// Launcher switches accepted without effect.
const IGNORED_SWITCH: &[&str] = &["--threaded-renderer", "--vanilla-only", "--vanilla-game", "--no-mod-autodetect", "--quiet"];
/// Options that would start a different game than an offline vanilla one; refused.
const REFUSED: &[&str] = &["--mod-iso", "--mod-gci", "--mod-dir", "--mod-profile", "--mod-base-iso", "--replay", "--lobby-direct", "--lobby-character", "--local-peer"];
/// The flags only the launcher's command line starts with (a `melee` subcommand never begins with `--`).
const LAUNCHER_FIRST: &[&str] = &["--iso", "--settings-path", "--card-dir", "--sys-dir", "--lobby-status-file", "--threaded-renderer"];

/// Exit code for a launcher request the Rust engine cannot honour.
pub const EXIT_UNSUPPORTED: u8 = 3;

/// True when `args` (argv without the program) is the launcher's command line.
pub fn detect(args: &[String]) -> bool {
    match args.first().map(String::as_str) {
        Some("play") => args.iter().any(|a| a == "--launcher-compat"),
        Some(a) => LAUNCHER_FIRST.contains(&a),
        None => false,
    }
}

/// The translated run: `melee play` arguments plus environment variables to set (only where unset).
#[derive(Debug, Default, PartialEq)]
pub struct Plan {
    pub play_args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub notes: Vec<String>,
    pub refused: Vec<String>,
}

/// `port-settings.ini`: `key value` per line.
pub fn parse_settings(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            let (k, v) = l.split_once(char::is_whitespace)?;
            Some((k.to_string(), v.trim().to_string()))
        })
        .collect()
}

fn on(v: &str) -> bool {
    v.trim().parse::<i64>().map_or(false, |n| n != 0)
}

/// The launcher's settings mapped onto our `MELEE_<KEY>` names (display.rs, melee_audio output.rs).
pub fn map_settings(kv: &[(String, String)]) -> Vec<(String, String)> {
    let get = |k: &str| kv.iter().rev().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
    let mut out = Vec::new();
    if let Some(fs) = get("fullscreen") {
        let mode = if !on(fs) { "off" } else if get("exclusivefullscreen").is_some_and(on) { "exclusive" } else { "borderless" };
        out.push(("MELEE_FULLSCREEN".into(), mode.into()));
    }
    if get("widescreen").is_some() || get("truewidescreen").is_some() {
        let ws = get("widescreen").is_some_and(on) || get("truewidescreen").is_some_and(on);
        out.push(("MELEE_WIDESCREEN".into(), (ws as u8).to_string()));
    }
    if let Some(w) = get("window").filter(|w| w.contains(['x', 'X'])) {
        out.push(("MELEE_WINDOW".into(), w.to_string()));
    }
    // the Source Port's internal resolution multiplier (efb_scale); 0 or below is its automatic choice
    if let Some(s) = get("scale").and_then(|s| s.trim().parse::<i64>().ok()) {
        out.push(("MELEE_RENDER_SCALE".into(), if s <= 0 { "auto".into() } else { s.min(8).to_string() }));
    }
    if let Some(m) = get("msaa").and_then(|s| s.trim().parse::<u32>().ok()) {
        let m = [8, 4, 2, 1].into_iter().find(|&n| m >= n).unwrap_or(1);
        out.push(("MELEE_MSAA".into(), m.to_string()));
    }
    if let Some(v) = get("vsync") {
        out.push(("MELEE_VSYNC".into(), if on(v) { "on" } else { "off" }.into()));
    }
    if let Some(v) = get("volume").and_then(|s| s.trim().parse::<i32>().ok()) {
        out.push(("MELEE_VOLUME".into(), v.clamp(0, 100).to_string()));
    }
    out
}

fn push_env(env: &mut Vec<(String, String)>, k: &str, v: String) {
    env.retain(|(a, _)| a != k);
    env.push((k.to_string(), v));
}

/// Translate the launcher's command line. `read` returns a file's text (the settings file).
pub fn plan(args: &[String], cwd: &Path, read: &dyn Fn(&Path) -> Option<String>) -> Plan {
    let mut p = Plan { play_args: vec!["play".into()], ..Plan::default() };
    let args: Vec<&String> = args.iter().skip(if args.first().is_some_and(|a| a == "play") { 1 } else { 0 }).collect();
    let mut cli_env: Vec<(String, String)> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let val = args.get(i + 1).map(|s| s.as_str());
        let (flag, inline) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v)),
            _ => (a, None),
        };
        let take = |i: &mut usize| -> Option<String> {
            if let Some(v) = inline {
                return Some(v.to_string());
            }
            *i += 1;
            val.map(str::to_string)
        };
        match flag {
            "--launcher-compat" => {}
            "--iso" => {
                if let Some(v) = take(&mut i) {
                    p.play_args.extend(["--iso".to_string(), v]);
                }
            }
            "--settings-path" => {
                if let Some(path) = take(&mut i) {
                    match read(Path::new(&path)) {
                        Some(text) => {
                            let m = map_settings(&parse_settings(&text));
                            p.notes.push(format!("settings {} -> {}", path, m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")));
                            for (k, v) in m {
                                push_env(&mut p.env, &k, v);
                            }
                        }
                        None => p.notes.push(format!("settings {path}: not there yet (the launcher writes it after its first change); defaults")),
                    }
                }
            }
            "--card-dir" => {
                if let Some(d) = take(&mut i) {
                    let d = PathBuf::from(d);
                    let base = d.parent().map(|q| q.join("RustPort")).unwrap_or_else(|| d.join("RustPort"));
                    p.notes.push(format!("card {} -> {}", d.display(), base.display()));
                    push_env(&mut p.env, "MELEE_CARD_PATH", base.display().to_string());
                }
            }
            "--volume" => {
                if let Some(v) = take(&mut i).and_then(|v| v.parse::<i32>().ok()) {
                    push_env(&mut cli_env, "MELEE_VOLUME", v.clamp(0, 100).to_string());
                }
            }
            "--music" | "--log-file" | "--scale" | "--frame-times" => {
                let v = take(&mut i).unwrap_or_default();
                match flag {
                    "--log-file" => push_env(&mut cli_env, "MELEE_LOG_FILE", v),
                    "--scale" => push_env(&mut cli_env, "MELEE_RENDER_SCALE", v),
                    _ => p.notes.push(format!("{flag} {v}: no Rust counterpart, ignored")),
                }
            }
            "--no-music" => p.notes.push("--no-music: no separate music volume, ignored".into()),
            "--fullscreen" => push_env(&mut cli_env, "MELEE_FULLSCREEN", "borderless".into()),
            "--widescreen" | "--true-widescreen" => push_env(&mut cli_env, "MELEE_WIDESCREEN", "1".into()),
            "--vsync" => push_env(&mut cli_env, "MELEE_VSYNC", "on".into()),
            "--window" => {
                if let Some(v) = take(&mut i) {
                    push_env(&mut cli_env, "MELEE_WINDOW", v);
                }
            }
            f if IGNORED_SWITCH.contains(&f) => p.notes.push(format!("{f}: accepted, nothing to do")),
            f if IGNORED_VALUE.contains(&f) => {
                let v = take(&mut i).unwrap_or_default();
                p.notes.push(format!("{f} {v}: accepted, nothing to do"));
            }
            f if REFUSED.contains(&f) || f.starts_with("--p2p-") => {
                let v = take(&mut i).unwrap_or_default();
                p.refused.push(format!("{f} {v}"));
            }
            _ => p.play_args.push(args[i].clone()),
        }
        i += 1;
    }
    for (k, v) in cli_env {
        push_env(&mut p.env, &k, v);
    }
    push_env(&mut p.env, "MELEE_LOG_FILE", cwd.join("melee_port.log").display().to_string());
    push_env(&mut p.env, "MELEE_CRASH_COPY", cwd.join("melee_port_crash.txt").display().to_string());
    if std::env::var_os("MELEE_NO_GC_ADAPTER").is_some() {
        push_env(&mut p.env, "MELEE_PAD_IGNORE_ADAPTER", "1".into());
    }
    p
}

/// Called first thing in `main` (before the log redirect): when `args` is the launcher's command line, set the
/// environment and return the `melee play` arguments to run instead.
pub fn prepare(args: &[String]) -> Option<Result<(Vec<String>, Vec<String>), Vec<String>>> {
    if !detect(args) {
        return None;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let p = plan(args, &cwd, &|f| std::fs::read_to_string(f).ok());
    for (k, v) in &p.env {
        if std::env::var_os(k).map_or(true, |x| x.is_empty()) {
            std::env::set_var(k, v);
        }
    }
    if !p.refused.is_empty() {
        return Some(Err(p.refused));
    }
    Some(Ok((p.play_args, p.notes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn detects_launcher_lines_only() {
        assert!(detect(&s(&["--iso", "x.iso", "--threaded-renderer"])));
        assert!(detect(&s(&["play", "--launcher-compat", "--iso", "x"])));
        assert!(!detect(&s(&["play", "--iso", "x"])));
        assert!(!detect(&s(&["golden"])));
        assert!(!detect(&[]));
    }

    #[test]
    fn maps_the_launchers_command_line() {
        let ini = "fps 60\nscale 3\nfullscreen 1\nexclusivefullscreen 0\nvsync 0\nwidescreen 1\nwindow 1600x1200\nvolume 40\nmsaa 4\n";
        let args = s(&[
            "--iso", "C:\\d\\melee.iso", "--threaded-renderer", "--settings-path", "C:\\g\\port-settings.ini", "--sys-dir", "C:\\g\\Sys",
            "--replay-dir", "C:\\g\\Replays", "--card-dir", "C:\\g\\User\\GC\\CardA", "--slippi-menus", "off", "--vanilla-only",
            "--lobby-status-file", "C:\\g\\lobby-game-status.json", "--quit-after-matches", "1",
        ]);
        let p = plan(&args, Path::new("C:\\g"), &|_| Some(ini.to_string()));
        assert!(p.refused.is_empty());
        assert_eq!(p.play_args, s(&["play", "--iso", "C:\\d\\melee.iso", "--quit-after-matches", "1"]));
        let env = |k: &str| p.env.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        assert_eq!(env("MELEE_FULLSCREEN"), Some("borderless"));
        assert_eq!(env("MELEE_WIDESCREEN"), Some("1"));
        assert_eq!(env("MELEE_WINDOW"), Some("1600x1200"));
        assert_eq!(env("MELEE_RENDER_SCALE"), Some("3"));
        assert_eq!(env("MELEE_MSAA"), Some("4"));
        assert_eq!(env("MELEE_VSYNC"), Some("off"));
        assert_eq!(env("MELEE_VOLUME"), Some("40"));
        assert_eq!(env("MELEE_CARD_PATH"), Some(Path::new("C:\\g\\User\\GC").join("RustPort").display().to_string().as_str()));
        assert!(env("MELEE_LOG_FILE").unwrap().ends_with("melee_port.log"));
        assert!(env("MELEE_CRASH_COPY").unwrap().ends_with("melee_port_crash.txt"));
    }

    #[test]
    fn command_line_beats_the_file_and_mods_are_refused() {
        let p = plan(&s(&["--iso", "a", "--settings-path", "s", "--volume", "7", "--mod-iso", "m.iso", "--p2p-port", "5000"]), Path::new("."), &|_| Some("volume 90\nwindow follow\nscale 0\n".into()));
        assert_eq!(p.env.iter().find(|(k, _)| k == "MELEE_VOLUME").unwrap().1, "7");
        assert_eq!(p.env.iter().find(|(k, _)| k == "MELEE_RENDER_SCALE").unwrap().1, "auto");
        assert!(!p.env.iter().any(|(k, _)| k == "MELEE_WINDOW"));
        assert_eq!(p.refused, s(&["--mod-iso m.iso", "--p2p-port 5000"]));
    }
}
