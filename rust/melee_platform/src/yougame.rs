//! The YouGame desktop app's stdio bridge (docs/rust-port/notes/yougame.md), standalone mode first.
//!
//! On only when the app started us: `YOUGAME_STDIO=1`. Then stdout carries bridge lines and nothing else (the
//! process's stdout handle is pointed at stderr, the original handle is kept for the bridge), stdin carries the
//! app's lines, read on a thread of its own. Off, nothing here runs and nothing changes.
//!
//! Lines written (one write per line, `\n`): `yg {"caps":[...]}` first, `yg {"ready":true}` after the first
//! presented frame, `yg {"key":"escape"}` for Shift+Esc, `yg {"overlay":"down"|"up"|"cancel"}` for Shift+Tab when
//! embedded, `yg {"type":"exitReason",...,"id":N}` requests. Lines read: `embed <cmd>` (the player box: `quit`,
//! `rect`, `show`/`hide`, `focus`/`blur`, `volume`), a bare `quit`, and `yg {...}` answers (`re`) and pushes
//! (`push`). EOF on stdin = the app is gone: quit.
//!
//! Written from the protocol as observed (Hero88go's melee-unlocked v0.9.107 and Crunch's 2026-11-08 handoff);
//! none of his code is used.
use std::collections::VecDeque;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// Seconds between a quit request and the hard exit if the main loop has not ended by then (the app kills at 5 s).
pub const QUIT_GRACE: Duration = Duration::from_millis(4000);

/// Caps when the game has its own top-level window: no pads / rumble / view / volume / fullscreen (the app must
/// not attach its pad relay; the game reads its controllers itself). No `overlay` (Shift+Tab opens our own menu),
/// no `join` / `notice` (not handled yet).
pub const CAPS_STANDALONE: &str = r#"yg {"caps":["ready"]}"#;
/// Caps in the player box (YOUGAME_EMBED=1): Shift+Tab goes to the page as overlay down/up. No `pads` (the pad
/// mirror is not implemented), no `view` / `volume` (not implemented).
pub const CAPS_EMBEDDED: &str = r#"yg {"caps":["ready","overlay"]}"#;
pub const READY: &str = r#"yg {"ready":true}"#;

struct State {
    embedded: bool,
    data: Option<String>,
    out: Mutex<Option<Box<dyn Write + Send>>>,
    ready: AtomicBool,
    quit: AtomicBool,
    menu: AtomicBool,
    tab_down: AtomicBool,
    next_id: AtomicI64,
    /// Player-box commands for the window thread (rect / show / hide / focus / blur), oldest first.
    box_cmds: Mutex<VecDeque<String>>,
    answers: Mutex<Vec<(i64, String)>>,
}

static STATE: OnceLock<State> = OnceLock::new();

fn st() -> Option<&'static State> {
    STATE.get()
}

/// The bridge is on (the YouGame app started this process with YOUGAME_STDIO=1).
pub fn active() -> bool {
    st().is_some()
}
/// The app asked for the player box (YOUGAME_EMBED=1).
pub fn embedded() -> bool {
    st().is_some_and(|s| s.embedded)
}
/// YOUGAME_DATA: the per-game writable folder the app gives us.
pub fn data_dir() -> Option<&'static str> {
    st().and_then(|s| s.data.as_deref())
}

/// Call first thing in main, before anything is printed. `None` (and no effect) when YOUGAME_STDIO is not 1.
/// Returns the environment defaults it set (for the log).
pub fn init() -> Option<Vec<String>> {
    if std::env::var("YOUGAME_STDIO").ok().as_deref() != Some("1") {
        return None;
    }
    let embedded = std::env::var("YOUGAME_EMBED").ok().as_deref() == Some("1");
    let data = std::env::var("YOUGAME_DATA").ok().filter(|d| !d.is_empty());
    // Children we start must not think the app started them.
    for k in ["YOUGAME_STDIO", "YOUGAME_EMBED", "YOUGAME_BRIDGE", "YOUGAME_TOKEN"] {
        std::env::remove_var(k);
    }
    let notes = data.as_deref().map(apply_data_dir).unwrap_or_default();
    let out = take_stdout();
    let s = State {
        embedded,
        data,
        out: Mutex::new(out),
        ready: AtomicBool::new(false),
        quit: AtomicBool::new(false),
        menu: AtomicBool::new(false),
        tab_down: AtomicBool::new(false),
        next_id: AtomicI64::new(1),
        box_cmds: Mutex::new(VecDeque::new()),
        answers: Mutex::new(Vec::new()),
    };
    if STATE.set(s).is_err() {
        return Some(notes);
    }
    write_line(if embedded { CAPS_EMBEDDED } else { CAPS_STANDALONE });
    std::thread::Builder::new()
        .name("yougame-stdin".into())
        .spawn(|| read_stdin(std::io::stdin().lock()))
        .ok();
    Some(notes)
}

/// YOUGAME_DATA: saves, settings and replays go under it unless already chosen (`MELEE_*` set). The log stays on
/// stderr (the app reads its last line for the "stopped" notice); MELEE_LOG_FILE still redirects it.
fn apply_data_dir(d: &str) -> Vec<String> {
    let base = std::path::Path::new(d);
    let mut notes = Vec::new();
    for (key, sub, dir) in [
        ("MELEE_CARD_PATH", "card", true),
        ("MELEE_SETTINGS_CFG", "settings.cfg", false),
        ("MELEE_SLP_DIR", "replays", true),
    ] {
        if std::env::var_os(key).is_some() {
            continue;
        }
        let p = base.join(sub);
        if dir {
            let _ = std::fs::create_dir_all(&p);
        } else {
            let _ = std::fs::create_dir_all(base);
        }
        std::env::set_var(key, &p);
        notes.push(format!("{key}=<YOUGAME_DATA>\\{sub}"));
    }
    notes
}

/// The bridge's own stdout; the process's stdout is pointed at stderr so a stray print cannot corrupt the bridge.
#[cfg(windows)]
fn take_stdout() -> Option<Box<dyn Write + Send>> {
    use std::os::windows::io::FromRawHandle;
    use windows_sys::Win32::Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Console::{GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    unsafe {
        let out = GetStdHandle(STD_OUTPUT_HANDLE);
        if out.is_null() || out == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut dup: HANDLE = std::ptr::null_mut();
        if DuplicateHandle(GetCurrentProcess(), out, GetCurrentProcess(), &mut dup, 0, 0, DUPLICATE_SAME_ACCESS) == 0 {
            return None;
        }
        let err = GetStdHandle(STD_ERROR_HANDLE);
        SetStdHandle(STD_OUTPUT_HANDLE, err);
        Some(Box::new(std::fs::File::from_raw_handle(dup as _)))
    }
}
#[cfg(unix)]
fn take_stdout() -> Option<Box<dyn Write + Send>> {
    use std::os::fd::FromRawFd;
    unsafe {
        let dup = libc_dup(1);
        if dup < 0 {
            return None;
        }
        libc_dup2(2, 1);
        Some(Box::new(std::fs::File::from_raw_fd(dup)))
    }
}
#[cfg(unix)]
extern "C" {
    #[link_name = "dup"]
    fn libc_dup(fd: i32) -> i32;
    #[link_name = "dup2"]
    fn libc_dup2(a: i32, b: i32) -> i32;
}

/// One bridge line (a single write, so lines never interleave).
pub fn write_line(line: &str) {
    let Some(s) = st() else { return };
    let mut g = s.out.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(o) = g.as_mut() {
        let mut b = Vec::with_capacity(line.len() + 1);
        b.extend_from_slice(line.as_bytes());
        b.push(b'\n');
        if o.write_all(&b).and_then(|_| o.flush()).is_err() {
            *g = None; // the app closed its end
        }
    }
}

/// After every successful present of a game frame: the first one sends `ready` (the app shows the game then).
pub fn presented() {
    if let Some(s) = st() {
        if !s.ready.swap(true, Ordering::SeqCst) {
            write_line(READY);
            crate::plog!("yougame: ready (first frame presented)");
        }
    }
}

/// The app asked us to stop (`embed quit`, `quit`) or closed stdin. Checked by the main loop every wake.
pub fn quit_requested() -> bool {
    st().is_some_and(|s| s.quit.load(Ordering::SeqCst))
}

/// Request a graceful quit; a watchdog ends the process after `QUIT_GRACE` if the loop has not.
pub fn request_quit(why: &str) {
    let Some(s) = st() else { return };
    if s.quit.swap(true, Ordering::SeqCst) {
        return;
    }
    crate::plog!("yougame: quit ({why})");
    if std::env::var_os("MELEE_YOUGAME_NO_WATCHDOG").is_none() {
        std::thread::spawn(|| {
            std::thread::sleep(QUIT_GRACE);
            crate::plog!("yougame: the main loop did not end within {:?} of the quit; exiting", QUIT_GRACE);
            std::process::exit(0);
        });
    }
}

/// Shift+Esc: the app brings its own window forward.
pub fn relay_escape() {
    write_line(r#"yg {"key":"escape"}"#);
}

/// Shift+Tab (window thread). Embedded: the page's overlay (down now, up on release). Standalone: our in-game menu
/// (taken once by the game as an F1 press). Returns true: the key is swallowed.
pub fn shift_tab(down: bool, repeat: bool) -> bool {
    let Some(s) = st() else { return false };
    if s.embedded {
        if down && !repeat && !s.tab_down.swap(true, Ordering::SeqCst) {
            write_line(r#"yg {"overlay":"down"}"#);
        } else if !down && s.tab_down.swap(false, Ordering::SeqCst) {
            write_line(r#"yg {"overlay":"up"}"#);
        }
    } else if down && !repeat {
        s.menu.store(true, Ordering::SeqCst);
    }
    true
}
/// Focus lost while the overlay key is held: the page cancels it.
pub fn focus_lost() {
    if let Some(s) = st() {
        if s.tab_down.swap(false, Ordering::SeqCst) {
            write_line(r#"yg {"overlay":"cancel"}"#);
        }
    }
}

/// The in-game menu was asked for (Shift+Tab, standalone), taken once.
pub fn take_menu_request() -> bool {
    st().is_some_and(|s| s.menu.swap(false, Ordering::SeqCst))
}

/// Player-box commands for the window thread, oldest first.
pub fn take_box_commands() -> Vec<String> {
    st().map(|s| s.box_cmds.lock().unwrap().drain(..).collect()).unwrap_or_default()
}

/// Tell the app why we are leaving (`{"type":"exitReason","reason":R,...}`), waiting up to 500 ms for its answer.
/// Only `mod_outdated` is a reason seen in use (with optional `mod`, `modVersion`).
pub fn exit_reason(reason: &str, extra: &[(&str, &str)]) {
    let Some(s) = st() else { return };
    let id = s.next_id.fetch_add(1, Ordering::SeqCst);
    let mut line = format!(r#"yg {{"type":"exitReason","reason":{}"#, json_str(reason));
    for (k, v) in extra {
        line.push_str(&format!(",{}:{}", json_str(k), json_str(&clean_field(v))));
    }
    line.push_str(&format!(r#","id":{id}}}"#));
    write_line(&line);
    let until = std::time::Instant::now() + Duration::from_millis(500);
    while std::time::Instant::now() < until {
        if s.answers.lock().unwrap().iter().any(|(i, _)| *i == id) || s.quit.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A crash's one stderr line for the app's "stopped" notice: `crash: <reason>`, first clause only, no paths.
pub fn crash_line(reason: &str) -> String {
    let r = reason.split([';', '\r', '\n']).next().unwrap_or("").trim_end();
    let r = if r.is_empty() || r.contains(['\\', '/']) { "fatal error" } else { r };
    format!("crash: {}", r.chars().take(160).collect::<String>())
}

/// Printable ASCII, no links, at most 60 characters, trimmed.
fn clean_field(v: &str) -> String {
    let c: String = v.chars().filter(|c| (' '..='~').contains(c)).collect();
    let l = c.to_ascii_lowercase();
    if l.contains("://") || l.contains("www.") {
        return String::new();
    }
    c.chars().take(60).collect::<String>().trim().to_string()
}

fn json_str(v: &str) -> String {
    let mut o = String::from("\"");
    for ch in v.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// What one line from the app means.
#[derive(Debug, PartialEq)]
pub enum AppLine {
    Quit,
    Box(String),
    Answer(i64),
    Push,
    PadRelay,
    Ignored,
}

pub fn parse_line(line: &str) -> AppLine {
    let line = line.strip_suffix('\r').unwrap_or(line);
    if line == "quit" || line == "embed quit" {
        return AppLine::Quit;
    }
    if let Some(c) = line.strip_prefix("embed ") {
        return if c.len() <= 160 { AppLine::Box(c.to_string()) } else { AppLine::Ignored };
    }
    if line.starts_with("yg-pads ") {
        return AppLine::PadRelay; // the app's pad relay: only when caps say `pads`, which ours never do
    }
    if let Some(j) = line.strip_prefix("yg ") {
        // A minimal look at the object: an answer carries "re":<id>; anything else with "push" is a push.
        if let Some(i) = j.find("\"re\"") {
            let rest = j[i + 4..].trim_start().strip_prefix(':').unwrap_or("").trim_start();
            let n: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '-').collect();
            if let Ok(id) = n.parse() {
                return AppLine::Answer(id);
            }
        }
        if j.contains("\"push\"") {
            return AppLine::Push;
        }
    }
    AppLine::Ignored
}

fn read_stdin<R: BufRead>(mut r: R) {
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match r.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        if buf.len() > 1 << 20 {
            continue;
        }
        let line = String::from_utf8_lossy(&buf);
        let line = line.trim_end_matches('\n');
        let Some(s) = st() else { return };
        match parse_line(line) {
            AppLine::Quit => request_quit("the app asked"),
            AppLine::Box(c) => {
                let mut q = s.box_cmds.lock().unwrap();
                if q.len() < 256 {
                    q.push_back(c);
                }
            }
            AppLine::Answer(id) => s.answers.lock().unwrap().push((id, line.to_string())),
            AppLine::Push => crate::plog!("yougame: push from the app (not handled yet): {}", line.chars().take(200).collect::<String>()),
            AppLine::PadRelay | AppLine::Ignored => {}
        }
    }
    request_quit("the app closed stdin");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lines() {
        assert_eq!(parse_line("embed quit"), AppLine::Quit);
        assert_eq!(parse_line("quit\r"), AppLine::Quit);
        assert_eq!(parse_line("embed rect 1 2 3 4"), AppLine::Box("rect 1 2 3 4".into()));
        assert_eq!(parse_line(r#"yg {"re": 7,"ok":true}"#), AppLine::Answer(7));
        assert_eq!(parse_line(r#"yg {"push":{"type":"notice"}}"#), AppLine::Push);
        assert_eq!(parse_line(r#"yg-pads {"seq":1}"#), AppLine::PadRelay);
        assert_eq!(parse_line("hello"), AppLine::Ignored);
    }
    #[test]
    fn crash_and_fields() {
        assert_eq!(crash_line("GPU lost; more"), "crash: GPU lost");
        assert_eq!(crash_line(r"cannot open C:\x\y.iso"), "crash: fatal error");
        assert_eq!(clean_field("  Mod\u{7} 1.2 "), "Mod 1.2");
        assert_eq!(clean_field("see https://x"), "");
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
    }
    #[test]
    fn caps_standalone_has_no_pads() {
        for k in ["pads", "rumble", "view", "volume", "fullscreen"] {
            assert!(!CAPS_STANDALONE.contains(k));
        }
        assert!(CAPS_STANDALONE.contains("ready"));
    }
}
