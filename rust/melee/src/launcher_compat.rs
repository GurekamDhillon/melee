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
//! * `--replay-dir D`: every match is exported there as a Slippi `.slp` (`--slp auto`, `MELEE_SLP_DIR`);
//! * `MELEE_NO_GC_ADAPTER` (the launcher's test runs) means `MELEE_PAD_IGNORE_ADAPTER=1`.
//!
//! Options with no Rust counterpart that change nothing for an offline game (`--threaded-renderer`, `--sys-dir`,
//! `--user-dir`, `--sysgen-data-dir`, the vanilla flags) are accepted and logged. Ones that would
//! change the game (mods, peer-to-peer and Slippi lobby matches) are refused with exit code 3 and a line in the
//! log, rather than starting a game other than the one asked for. Anything else is passed to `melee play` as given.
use std::path::{Path, PathBuf};

/// Launcher options that take a value and are accepted without effect.
const IGNORED_VALUE: &[&str] = &["--sys-dir", "--sysgen-data-dir", "--slippi-menus", "--online-delay", "--language", "--backend", "--fps", "--aspect", "--dlss", "--dlss-mode", "--ssao", "--frame-mode"];
/// Launcher switches accepted without effect.
const IGNORED_SWITCH: &[&str] = &["--as-experienced", "--threaded-renderer", "--vanilla-only", "--vanilla-game", "--no-mod-autodetect", "--quiet"];
/// Options that would start a different game than an offline vanilla one; refused.
const REFUSED: &[&str] = &["--mod-gci", "--mod-dir", "--mod-profile", "--local-peer"];
/// The flags only the launcher's command line starts with (a `melee` subcommand never begins with `--`).
const LAUNCHER_FIRST: &[&str] = &["--iso", "--settings-path", "--card-dir", "--sys-dir", "--lobby-status-file", "--threaded-renderer", "--settings-window"];

/// The subcommand `plan` returns for `--settings-window` (the launcher's Settings button / standby panel).
pub const SETTINGS_WINDOW: &str = "launcher-settings-window";

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
    // Hero's "Music" slider (pc_settings `music`, 0..100). His `audio_device` is a WASAPI endpoint id, not a name
    // our output can open, so it is not mapped: the device is picked in our own settings overlay.
    if let Some(v) = get("music").and_then(|s| s.trim().parse::<i32>().ok()) {
        out.push(("MELEE_MUSIC".into(), v.clamp(0, 100).to_string()));
    }
    // the adapter keeps reading with the window in the background (melee_platform PadSystem::background_input)
    if let Some(v) = get("backgroundinput") {
        out.push(("MELEE_BACKGROUND_INPUT".into(), (on(v) as u8).to_string()));
    }
    out
}

fn push_env(env: &mut Vec<(String, String)>, k: &str, v: String) {
    env.retain(|(a, _)| a != k);
    env.push((k.to_string(), v));
}

/// Translate the launcher's command line. `read` returns a file's text (the settings file).
pub fn plan(args: &[String], cwd: &Path, read: &dyn Fn(&Path) -> Option<String>) -> Plan {
    plan_env(args, cwd, read, &|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
}

/// Slippi's own matchmaking server: only with the player's login, never from an agent or test run.
pub const SLIPPI_MM_REAL: &str = "wss://matchmaking.slippi.gg/v1/queue";

/// Hero's main.cpp `--lobby-direct` check: `NAME#123`, upper-case letters / digits before the `#`, digits after, at
/// most 18 characters.
pub fn lobby_code_ok(code: &str) -> bool {
    let Some(hash) = code.find('#') else { return false };
    hash > 0
        && hash < code.len() - 1
        && code.len() <= 18
        && code.char_indices().all(|(i, c)| i == hash || c.is_ascii_digit() || (i < hash && c.is_ascii_uppercase()))
}

/// The lobby's Direct button (`--lobby-direct CODE --lobby-character N`, H:launcher.cpp:2905) as a Slippi Direct
/// match through our Slippi-wire path (`play --slippi-mm ...`, play_net::connect_slippi): matchmaking to CODE with
/// external character N, the match in this window, exit 0 after it and non-zero when it cannot connect (his
/// "could not connect" box, H:launcher.cpp:2758-2770). The login is the player's Slippi user.json
/// (`YOUGAME_SLIPPI_USER_JSON`, else `user.json` in `--user-dir`; only its path here, slippi_np reads it). A test or
/// agent run (`MELEE_SLIPPI_MM_URL`, a pad script, a hidden window or `MELEE_LAUNCHER_TEST`) never reaches Slippi's
/// servers: it needs the loopback mock and a test code (`MELEE_SLIPPI_TEST_CODE`).
fn lobby_direct_args(code: &str, character: u8, user_dir: Option<&str>, env: &dyn Fn(&str) -> Option<String>) -> Result<Vec<String>, String> {
    let port = env("MELEE_SLIPPI_NP_PORT").or_else(|| env("YOUGAME_SLIPPI_UDP_PORT")).unwrap_or_else(|| "51441".into());
    let mut out: Vec<String> = Vec::new();
    let agent = env("MELEE_PAD_SCRIPT").is_some() || env("MELEE_WINDOW_HIDDEN").is_some_and(|v| v == "1") || env("MELEE_LAUNCHER_TEST").is_some();
    if let Some(url) = env("MELEE_SLIPPI_MM_URL") {
        let host = url.split("://").nth(1).unwrap_or("").split(['/', ':']).next().unwrap_or("");
        if host != "127.0.0.1" && host != "localhost" {
            return Err("MELEE_SLIPPI_MM_URL must be a loopback mock".into());
        }
        out.extend(["--slippi-mm".to_string(), url, "--slippi-port".into(), port]);
        out.extend(["--slippi-code".to_string(), env("MELEE_SLIPPI_TEST_CODE").unwrap_or_else(|| "RUST#001".into())]);
    } else if agent {
        return Err("a test run needs the loopback mock (MELEE_SLIPPI_MM_URL), never Slippi's servers".into());
    } else {
        let login = env("YOUGAME_SLIPPI_USER_JSON").or_else(|| user_dir.map(|d| Path::new(d).join("user.json").display().to_string()));
        let Some(login) = login else { return Err("no Slippi login (--user-dir)".into()) };
        out.extend(["--slippi-mm", SLIPPI_MM_REAL, "--slippi-mm-real", "--slippi-user-json"].map(String::from));
        out.extend([login, "--slippi-port".into(), port]);
    }
    out.extend(["--slippi-mm-mode", "direct", "--slippi-search"].map(String::from));
    out.extend([code.to_string(), "--slippi-char".into(), character.to_string()]);
    if let Some(s) = env("MELEE_SLIPPI_NP_MAX_SECONDS") {
        out.extend(["--slippi-max-seconds".to_string(), s]);
    }
    Ok(out)
}

/// `plan` with the environment as a lookup (tests).
pub fn plan_env(args: &[String], cwd: &Path, read: &dyn Fn(&Path) -> Option<String>, env: &dyn Fn(&str) -> Option<String>) -> Plan {
    let mut p = Plan { play_args: vec!["play".into()], ..Plan::default() };
    let args: Vec<&String> = args.iter().skip(if args.first().is_some_and(|a| a == "play") { 1 } else { 0 }).collect();
    let mut cli_env: Vec<(String, String)> = Vec::new();
    let mut base_iso: Option<String> = None;
    let mut mod_iso: Option<String> = None;
    let (mut lobby_code, mut lobby_char, mut user_dir): (Option<String>, Option<u8>, Option<String>) = (None, None, None);
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
            // the lobby's Direct button (lobby_direct_args), with Hero's checks
            "--lobby-direct" => {
                let c = take(&mut i).unwrap_or_default();
                if lobby_code_ok(&c) {
                    lobby_code = Some(c);
                } else {
                    p.refused.push("--lobby-direct (invalid code)".into());
                }
            }
            "--lobby-character" => match take(&mut i).and_then(|v| v.parse::<u8>().ok()) {
                Some(n) => lobby_char = Some(n),
                None => p.refused.push("--lobby-character (invalid, 0..255)".into()),
            },
            "--user-dir" => {
                user_dir = take(&mut i);
                p.notes.push("--user-dir: the Slippi login's folder (lobby Direct only)".into());
            }
            "--iso" => {
                base_iso = take(&mut i);
            }
            // a mod disc: playable when it runs the retail game's code (a data-only mod), see mod_disc_ok
            "--mod-iso" => mod_iso = take(&mut i),
            "--mod-base-iso" => {
                // the Static Recomp form: --iso is the mod, this the retail disc
                if let Some(b) = take(&mut i) {
                    mod_iso = base_iso.replace(b);
                }
            }
            // the Build tab and the warm-up before PLAY: a hidden run of N frames
            "--hidden" => p.play_args.push("--headless".into()),
            "--frames" => {
                if let Some(v) = take(&mut i) {
                    p.play_args.extend(["--max-frames".to_string(), v]);
                }
            }
            "--settings-window" => p.play_args[0] = SETTINGS_WINDOW.into(),
            "--settings-standby" | "--settings-tab" => {
                let v = take(&mut i).unwrap_or_default();
                p.notes.push(format!("{flag} {v}: no settings panel in the Rust engine"));
            }
            "--replay-dir" => {
                // our own replays (.mrpl) in a folder of their own, so his .slp viewer never meets them
                if let Some(d) = take(&mut i) {
                    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
                    let f = Path::new(&d).join("RustPort").join(format!("melee-{t}.mrpl"));
                    p.play_args.extend(["--record".to_string(), f.display().to_string()]);
                    // and each match as a Slippi .slp in the folder itself, where his replay browser looks (slp_export.rs)
                    p.notes.push(format!("replays -> {d} (.slp)"));
                    push_env(&mut p.env, "MELEE_SLP_DIR", d);
                    p.play_args.extend(["--slp".to_string(), "auto".to_string()]);
                }
            }
            "--settings-path" => {
                if let Some(path) = take(&mut i) {
                    match read(Path::new(&path)) {
                        Some(text) => {
                            push_env(&mut p.env, "MELEE_PORT_SETTINGS", path.clone());
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
                    p.notes.push(crate::card_import::on_launch(&d, &base));
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
            // the lobby's status mailbox (lobby_status.rs): {"status":"In game"|"In match","stocks":[n]}
            "--lobby-status-file" => {
                if let Some(f) = take(&mut i) {
                    p.notes.push(format!("--lobby-status-file {f}: status written there"));
                    push_env(&mut p.env, "MELEE_LOBBY_STATUS_FILE", f);
                }
            }
            // the YouGame restart into a mode (his main.cpp: direct 2, ranked 1, else 0): MELEE_ONLINE_MODE for the
            // online browser; an unknown word is the plain online menu, as his port reads it
            "--online-mode" => {
                let m = take(&mut i).unwrap_or_default();
                let m = match m.as_str() { "direct" | "ranked" => m, _ => "unranked".to_string() };
                p.notes.push(format!("--online-mode {m}: MELEE_ONLINE_MODE"));
                push_env(&mut p.env, "MELEE_ONLINE_MODE", m);
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
    match (base_iso, mod_iso) {
        (Some(b), None) => p.play_args.splice(1..1, ["--iso".to_string(), b]).for_each(drop),
        (b, Some(m)) => match mod_disc_ok(Path::new(&m), b.as_deref().map(Path::new), read_bytes) {
            Ok(()) => {
                p.notes.push(format!("mod disc {m}: the retail game's code, its files played as a disc"));
                p.play_args.splice(1..1, ["--iso".to_string(), m]).for_each(drop);
            }
            Err(why) => p.refused.push(format!("--mod-iso {m} ({why})")),
        },
        (None, None) => {}
    }
    if let Some(code) = lobby_code {
        // Hero's default character is Fox (slippi_online.h: lobby_character = 2)
        let ch = lobby_char.unwrap_or(2);
        match lobby_direct_args(&code, ch, user_dir.as_deref(), env) {
            Ok(a) => {
                p.notes.push(format!("--lobby-direct {}: Slippi Direct, character {ch}", mask_code(&code)));
                p.play_args.extend(a);
            }
            Err(why) => p.refused.push(format!("--lobby-direct ({why})")),
        }
    } else if lobby_char.is_some() {
        p.notes.push("--lobby-character without --lobby-direct: ignored".into());
    }
    // the hidden warm-up runs (Build tab, warm cache) are not games anyone played: no replay
    if p.play_args.iter().any(|a| a == "--headless") {
        if let Some(i) = p.play_args.iter().position(|a| a == "--record") {
            p.play_args.drain(i..(i + 2).min(p.play_args.len()));
        }
    }
    // `--replay FILE.slp` plays a replay in the window (slp_play.rs): nothing to record or export
    if p.play_args.iter().any(|a| a == "--replay") {
        for flag in ["--record", "--slp"] {
            if let Some(i) = p.play_args.iter().position(|a| a == flag) {
                p.play_args.drain(i..(i + 2).min(p.play_args.len()));
            }
        }
    }
    if p.play_args[0] == SETTINGS_WINDOW {
        p.play_args.truncate(1);
    }
    for (k, v) in cli_env {
        push_env(&mut p.env, &k, v);
    }
    // the launcher's --log-file (the Build tab's launcher_build.log) names the log, else melee_port.log
    if !p.env.iter().any(|(k, _)| k == "MELEE_LOG_FILE") {
        push_env(&mut p.env, "MELEE_LOG_FILE", cwd.join("melee_port.log").display().to_string());
    } else if let Some(e) = p.env.iter_mut().find(|(k, _)| k == "MELEE_LOG_FILE") {
        if Path::new(&e.1).is_relative() {
            e.1 = cwd.join(&e.1).display().to_string();
        }
    }
    push_env(&mut p.env, "MELEE_CRASH_COPY", cwd.join("melee_port_crash.txt").display().to_string());
    if env("MELEE_NO_GC_ADAPTER").is_some() {
        push_env(&mut p.env, "MELEE_PAD_IGNORE_ADAPTER", "1".into());
    }
    p
}

/// A connect code for the log: the tag's first letter and the number's length only.
pub fn mask_code(code: &str) -> String {
    let (tag, num) = code.split_once('#').unwrap_or((code, ""));
    format!("{}***#{}", tag.chars().next().unwrap_or('?'), "*".repeat(num.len()))
}

fn read_bytes(path: &Path, off: u64, len: usize) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path).ok()?;
    f.seek(SeekFrom::Start(off)).ok()?;
    let mut v = vec![0; len];
    f.read_exact(&mut v).ok()?;
    Some(v)
}

/// A disc's main.dol (the header's 0x420 offset; the size from the DOL's 18 sections).
fn dol_of(path: &Path, read: fn(&Path, u64, usize) -> Option<Vec<u8>>) -> Option<Vec<u8>> {
    let be = |b: &[u8], o: usize| u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    let id = read(path, 0, 6)?;
    if &id[..] != b"GALE01" {
        return None;
    }
    let off = be(&read(path, 0x420, 4)?, 0) as u64;
    let h = read(path, off, 0x100)?;
    let end = (0..18).map(|k| be(&h, k * 4) as u64 + be(&h, 0x90 + k * 4) as u64).max()?.max(0x100);
    read(path, off, end as usize)
}

/// A mod disc the Rust engine can play: a Melee disc whose main.dol is the retail disc's byte for byte (the mod
/// changes files only: costumes, stages, music). A changed DOL (20XX, m-ex discs, Gecko-patched builds) is code the
/// native port does not run.
pub fn mod_disc_ok(m: &Path, base: Option<&Path>, read: fn(&Path, u64, usize) -> Option<Vec<u8>>) -> Result<(), String> {
    let base = base.ok_or("no retail --iso to compare it with")?;
    let md = dol_of(m, read).ok_or("not a readable Melee NTSC disc")?;
    let bd = dol_of(base, read).ok_or("the retail disc is not readable")?;
    if md != bd {
        return Err("its game code differs from the retail disc's".into());
    }
    Ok(())
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

/// Apply the launcher's controls (`MELEE_PORT_SETTINGS`, melee_platform::bindings) and `MELEE_BACKGROUND_INPUT`
/// to a freshly made pad system. Nothing changes without them.
pub fn apply_input(pads: &mut melee_platform::PadSystem) {
    if std::env::var("MELEE_BACKGROUND_INPUT").is_ok_and(|v| v == "1") {
        pads.background_input = true;
    }
    // controller profiles (melee_platform::profiles): --controllers / MELEE_CONTROLLERS, --pad-profile, --socd
    let args: Vec<String> = std::env::args().collect();
    match melee_platform::profiles::from_args(&args) {
        Ok(Some(set)) => {
            for w in &set.warnings {
                eprintln!("controllers: {w}");
            }
            eprintln!("controllers: {} profile(s), port overrides {:?}", set.profiles.len(), set.ports);
            pads.profiles = Some(set);
        }
        Ok(None) => {}
        Err(e) => eprintln!("controllers: {e} (profiles off)"),
    }
    let Some(text) = std::env::var_os("MELEE_PORT_SETTINGS").and_then(|p| std::fs::read_to_string(p).ok()) else { return };
    let b = melee_platform::bindings::parse(&text);
    if b.lines == 0 {
        return;
    }
    let pads_n = b.xinput.iter().filter(|m| m.is_some()).count();
    let gc_n = b.gc.iter().filter(|m| m.is_some()).count();
    eprintln!("launcher-compat: controls from port-settings.ini: keyboard {}, {pads_n} gamepad table(s), {gc_n} adapter remap(s)", if b.keyboard.is_some() { "remapped" } else { "default" });
    if let Some(kb) = b.keyboard {
        pads.keyboard[0] = kb;
    }
    pads.pad_maps = b.xinput;
    pads.gc_maps = b.gc;
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
    #[cfg_attr(not(windows), ignore = "the Windows launcher's command line: its paths only split on `\\` on Windows")]
    fn maps_the_launchers_command_line() {
        let ini = "fps 60\nscale 3\nfullscreen 1\nexclusivefullscreen 0\nvsync 0\nwidescreen 1\nwindow 1600x1200\nvolume 40\nmusic 25\nmsaa 4\n";
        let args = s(&[
            "--iso", "C:\\d\\melee.iso", "--threaded-renderer", "--settings-path", "C:\\g\\port-settings.ini", "--sys-dir", "C:\\g\\Sys",
            "--replay-dir", "C:\\g\\Replays", "--card-dir", "C:\\g\\User\\GC\\CardA", "--slippi-menus", "off", "--vanilla-only",
            "--lobby-status-file", "C:\\g\\lobby-game-status.json", "--quit-after-matches", "1",
        ]);
        let p = plan(&args, Path::new("C:\\g"), &|_| Some(ini.to_string()));
        assert!(p.refused.is_empty());
        assert_eq!(p.play_args[..3], s(&["play", "--iso", "C:\\d\\melee.iso"]));
        assert_eq!(p.play_args[3], "--record");
        assert!(p.play_args[4].starts_with("C:\\g\\Replays\\RustPort\\melee-") && p.play_args[4].ends_with(".mrpl"));
        assert_eq!(p.play_args[5..], s(&["--slp", "auto", "--quit-after-matches", "1"]));
        let env = |k: &str| p.env.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        assert_eq!(env("MELEE_FULLSCREEN"), Some("borderless"));
        assert_eq!(env("MELEE_WIDESCREEN"), Some("1"));
        assert_eq!(env("MELEE_WINDOW"), Some("1600x1200"));
        assert_eq!(env("MELEE_RENDER_SCALE"), Some("3"));
        assert_eq!(env("MELEE_MSAA"), Some("4"));
        assert_eq!(env("MELEE_VSYNC"), Some("off"));
        assert_eq!(env("MELEE_VOLUME"), Some("40"));
        assert_eq!(env("MELEE_MUSIC"), Some("25"));
        assert_eq!(env("MELEE_SLP_DIR"), Some(r"C:\g\Replays"));
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
        assert_eq!(p.refused.len(), 2);
        assert_eq!(p.refused[0], "--p2p-port 5000");
        assert!(p.refused[1].starts_with("--mod-iso m.iso ("), "{:?}", p.refused);
    }

    #[test]
    fn replay_passes_through_to_play() {
        let p = plan(&s(&["--iso", "d.iso", "--settings-path", "s", "--replay-dir", r"C:\g\R", "--replay", "x.slp", "--as-experienced"]), Path::new("."), &|_| None);
        assert!(p.refused.is_empty(), "{:?}", p.refused);
        assert_eq!(p.play_args, s(&["play", "--iso", "d.iso", "--replay", "x.slp"]));
    }

    #[test]
    fn build_flow_and_settings_window() {
        let p = plan(&s(&["--iso", "d.iso", "--settings-path", "s", "--hidden", "--frames", "30", "--volume", "0", "--log-file", "launcher_build.log"]), Path::new("."), &|_| Some("backgroundinput 1\n".into()));
        assert_eq!(p.play_args, s(&["play", "--iso", "d.iso", "--headless", "--max-frames", "30"]));
        let env = |k: &str| p.env.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
        assert_eq!(env("MELEE_PORT_SETTINGS").as_deref(), Some("s"));
        assert_eq!(env("MELEE_BACKGROUND_INPUT").as_deref(), Some("1"));
        assert!(env("MELEE_LOG_FILE").unwrap().ends_with("launcher_build.log"));
        assert!(detect(&s(&["--settings-window", "--settings-standby", "12", "--settings-path", "x"])));
        let p = plan(&s(&["--settings-window", "--settings-standby", "12", "--settings-path", "x"]), Path::new("."), &|_| None);
        assert_eq!(p.play_args, s(&[SETTINGS_WINDOW]));
    }

    #[test]
    fn launcher_05871a4a_arguments() {
        let p = plan(&s(&["--iso", "d.iso", "--sysgen-data-dir", "L/MeleeUnlocked-YouGame", "--lobby-status-file", "g.json", "--online-mode", "direct"]), Path::new("."), &|_| None);
        assert!(p.refused.is_empty());
        assert_eq!(p.play_args, s(&["play", "--iso", "d.iso"]));
        let env = |k: &str| p.env.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
        assert_eq!(env("MELEE_LOBBY_STATUS_FILE").as_deref(), Some("g.json"));
        assert_eq!(env("MELEE_ONLINE_MODE").as_deref(), Some("direct"));
        let p = plan(&s(&["--iso", "d", "--online-mode", "x"]), Path::new("."), &|_| None);
        assert_eq!(p.env.iter().find(|(a, _)| a == "MELEE_ONLINE_MODE").unwrap().1, "unranked");
    }

    fn envs(kv: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| kv.iter().find(|(a, _)| *a == k).map(|(_, v)| v.to_string())
    }

    #[test]
    fn lobby_direct_maps_to_slippi_direct() {
        let args = s(&["--iso", "d.iso", "--user-dir", "U", "--lobby-status-file", "g.json", "--lobby-direct", "BBBB#002", "--lobby-character", "9"]);
        // a test run: the loopback mock and the test code
        let e = envs(&[("MELEE_SLIPPI_MM_URL", "ws://127.0.0.1:43170/v1/queue"), ("MELEE_SLIPPI_TEST_CODE", "AAAA#001"), ("MELEE_SLIPPI_NP_PORT", "43270"), ("MELEE_PAD_SCRIPT", "x")]);
        let p = plan_env(&args, Path::new("."), &|_| None, &e);
        assert!(p.refused.is_empty(), "{:?}", p.refused);
        assert_eq!(
            p.play_args,
            s(&["play", "--iso", "d.iso", "--slippi-mm", "ws://127.0.0.1:43170/v1/queue", "--slippi-port", "43270", "--slippi-code", "AAAA#001", "--slippi-mm-mode", "direct", "--slippi-search", "BBBB#002", "--slippi-char", "9"])
        );
        assert!(p.notes.iter().any(|n| n.contains("B***#***") && !n.contains("BBBB")));
        assert_eq!(p.env.iter().find(|(k, _)| k == "MELEE_LOBBY_STATUS_FILE").unwrap().1, "g.json");
        // a player's run: Slippi's server with the login in --user-dir (only its path), default port, Fox by default
        let p = plan_env(&s(&["--iso", "d.iso", "--user-dir", "U", "--lobby-direct", "AB#1"]), Path::new("."), &|_| None, &envs(&[]));
        assert!(p.refused.is_empty(), "{:?}", p.refused);
        let user = Path::new("U").join("user.json").display().to_string();
        assert_eq!(p.play_args[3..], s(&["--slippi-mm", SLIPPI_MM_REAL, "--slippi-mm-real", "--slippi-user-json", &user, "--slippi-port", "51441", "--slippi-mm-mode", "direct", "--slippi-search", "AB#1", "--slippi-char", "2"]));
    }

    #[test]
    fn lobby_direct_never_reaches_slippi_from_a_test() {
        let args = s(&["--iso", "d.iso", "--user-dir", "U", "--lobby-direct", "AB#1", "--lobby-character", "2"]);
        for e in [&[("MELEE_LAUNCHER_TEST", "1")][..], &[("MELEE_WINDOW_HIDDEN", "1")], &[("MELEE_PAD_SCRIPT", "p.txt")], &[("MELEE_SLIPPI_MM_URL", "wss://matchmaking.slippi.gg/v1/queue")]] {
            let kv: &'static [(&str, &str)] = Box::leak(e.to_vec().into_boxed_slice());
            let p = plan_env(&args, Path::new("."), &|_| None, &envs(kv));
            assert_eq!(p.refused.len(), 1, "{e:?}");
            assert!(!p.play_args.iter().any(|a| a.contains("slippi.gg")), "{e:?}");
        }
    }

    #[test]
    fn lobby_direct_checks_like_hero() {
        for bad in ["", "AB", "#12", "AB#", "ab#1", "A-B#1", "AB#1C", "ABCDEFGHIJKLMNOP#12"] {
            assert!(!lobby_code_ok(bad), "{bad}");
        }
        for good in ["AB#1", "A1B#99", "ABCDEFGH#123456789"] {
            assert!(lobby_code_ok(good), "{good}");
        }
        let e = envs(&[("MELEE_SLIPPI_MM_URL", "ws://127.0.0.1:1/v1/queue")]);
        let p = plan_env(&s(&["--iso", "d", "--lobby-direct", "ab#1", "--lobby-character", "300"]), Path::new("."), &|_| None, &e);
        assert_eq!(p.refused.len(), 2, "{:?}", p.refused);
        // P2P Direct (Hero's own mu_net protocol) is still refused: exit 3, not a different game
        let p = plan_env(&s(&["--iso", "d", "--p2p-port", "5000", "--p2p-connect-seconds", "45", "--p2p-games", "1"]), Path::new("."), &|_| None, &e);
        assert_eq!(p.refused.len(), 3);
    }

    #[test]
    fn mod_disc_needs_the_retail_code() {
        // a fake disc: id, DOL at 0x440 with one 0x10-byte text section at file offset 0x100
        fn disc(code: u8) -> Vec<u8> {
            let mut d = vec![0u8; 0x440 + 0x110];
            d[..6].copy_from_slice(b"GALE01");
            d[0x420..0x424].copy_from_slice(&0x440u32.to_be_bytes());
            d[0x440..0x444].copy_from_slice(&0x100u32.to_be_bytes());
            d[0x440 + 0x90..0x440 + 0x94].copy_from_slice(&0x10u32.to_be_bytes());
            d[0x440 + 0x100] = code;
            d
        }
        fn rd(p: &Path, off: u64, len: usize) -> Option<Vec<u8>> {
            let d = disc(if p.to_str() == Some("other") { 2 } else { 1 });
            d.get(off as usize..off as usize + len).map(|x| x.to_vec())
        }
        assert!(mod_disc_ok(Path::new("same"), Some(Path::new("base")), rd).is_ok());
        assert!(mod_disc_ok(Path::new("other"), Some(Path::new("base")), rd).is_err());
        assert!(mod_disc_ok(Path::new("same"), None, rd).is_err());
    }
}
