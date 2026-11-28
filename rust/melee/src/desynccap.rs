//! Desync capture for Slippi online matches: when the online checksum compare reports a Soft or a Hard desync, the
//! engine writes a report and the match's inputs as a .slp it can replay offline (`melee desync-replay DIR`).
//!
//! What is written, to `<root>/<timestamp>/` (root: `MELEE_DESYNC_DIR`, else `$YOUGAME_DATA/desyncs`, else `desyncs`):
//!
//! * `report.json`: the online frame, the Slippi frame number, both checksums, Soft / Hard, our build id, the Slippi app
//!   version we sent, the match block, the online codes word, the rollback stats and `state_dump()` at detection;
//! * `desync.slp`: the match from frame -123 through the last confirmed frame at detection. For a Soft, rewritten at the
//!   game's end with the whole match.
//!
//! The first Soft or Hard of a game gets a directory, and the first Hard gets one if the first was a Soft. No player
//! names, connect codes or account data are in either file.
//!
//! The capture costs nothing in a normal frame: the match is recorded in memory by the live exporter
//! (`slp_export::live_rollback`, the same events the .slp carries), and nothing touches a file on the game thread:
//! the bytes are encoded in memory at detection and a worker thread writes them. Off with `MELEE_DESYNC_CAPTURE=0`.
//!
//! State lives behind a pointer made once on the system heap, as the exporter's does: the rollback snapshot rewinds
//! the statics and the arena, and "the first desync of the game" must survive a rollback.

use melee_core::slippi_online::Compare;
use melee_net::session::RbStats;
use melee_net::slippi::Selections;
use std::sync::Mutex;

/// Whether desync capture is on (default on; `MELEE_DESYNC_CAPTURE=0` turns it off).
pub fn enabled() -> bool {
    std::env::var("MELEE_DESYNC_CAPTURE").map_or(true, |v| v != "0")
}

/// The match block of a Slippi link, as `slippi_np::Linked` holds it.
#[derive(Clone)]
pub struct MatchInfo {
    pub decider: bool,
    pub local_port: u8,
    pub remote_port: u8,
    pub by_port: [Selections; 2],
    pub rng_offset: u32,
    pub scene: String,
    pub stage: u16,
}

#[derive(Default)]
struct State {
    info: Option<MatchInfo>,
    /// the app version a matchmaking join sent (a direct link sends none)
    app_version: String,
    /// per game: what has been captured already
    seen_first: bool,
    seen_hard: bool,
    ended: bool,
    /// the first Soft's directory and its report, waiting for the game's end (to write the whole match)
    soft: Option<Event>,
    /// the remote player's Melee Unlocked build message (0xE0): (one-line text, `peer_build` JSON). None: none came
    peer_build: Option<(String, String)>,
    jobs: Vec<std::thread::JoinHandle<()>>,
    /// directory names claimed in this process but maybe not yet written by their worker. Kept here (system heap behind
    /// `ST`) and not in a static of its own: a rollback load rewinds every writable static, and a rewound Vec header is a
    /// double free (rbstatics_lint.py)
    claimed: Vec<std::path::PathBuf>,
}

struct Event {
    dir: std::path::PathBuf,
    report: String,
}

static ST: std::sync::atomic::AtomicPtr<Mutex<State>> = std::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

/// The state, made on the first call from a thread that is outside the game (before any snapshot is taken).
fn st() -> &'static Mutex<State> {
    let p = ST.load(std::sync::atomic::Ordering::Acquire);
    if !p.is_null() {
        return unsafe { &*p };
    }
    let _s = melee_engine::arena::SysScope::enter();
    let b = Box::into_raw(Box::new(Mutex::new(State::default())));
    match ST.compare_exchange(core::ptr::null_mut(), b, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire) {
        Ok(_) => unsafe { &*b },
        Err(p) => {
            drop(unsafe { Box::from_raw(b) });
            unsafe { &*p }
        }
    }
}

/// Make the netplay client report a peer's 0xE0 build message here (once per process, from `main`).
pub fn install_hook() {
    let _ = melee_net::slippi::BUILD_HOOK.set(note_peer_build);
}

fn note_peer_build(port: u8, pb: &melee_net::slippi::PeerBuild) {
    let _sys = melee_engine::arena::SysScope::enter();
    let text = pb.describe();
    eprintln!("slippi: opponent (port {port}): {text}");
    let caps = pb.capability_names().iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", ");
    let json = format!(
        "{{\"engine\": \"{}\", \"message_version\": {}, \"mod_view\": {}, \"mod\": {}, \"fingerprint_prefix\": {}, \"capabilities\": [{caps}], \"truncated\": {}, \"text\": \"{}\"}}",
        if pb.static_engine() { "static_recomp" } else { "source_port" },
        pb.version,
        pb.mod_view,
        if pb.mod_view { format!("\"{}\"", esc(&pb.name)) } else { "null".into() },
        if pb.mod_view { format!("\"{}\"", esc(&pb.fingerprint.chars().take(16).collect::<String>())) } else { "null".into() },
        pb.truncated,
        esc(&text)
    );
    if let Ok(mut s) = st().lock() {
        s.peer_build = Some((text, json));
    }
}

/// The peer's build as one line for the netplay summary: its 0xE0 message, or a note that none came.
pub fn peer_build_line() -> String {
    match st().lock().ok().and_then(|s| s.peer_build.clone()) {
        Some((t, _)) => t,
        None => "no Melee Unlocked build message (stock Slippi Dolphin or another client)".into(),
    }
}

/// The Slippi app version a matchmaking join sends (recorded for the report).
pub fn note_app_version(v: &str) {
    if let Ok(mut s) = st().lock() {
        s.app_version = v.to_string();
    }
}

/// A new game's link is up: remember its match block and forget what the last game captured.
pub fn set_match(info: MatchInfo) {
    if let Ok(mut s) = st().lock() {
        s.info = Some(info);
        s.seen_first = false;
        s.seen_hard = false;
        s.ended = false;
        s.soft = None;
    }
}

fn root() -> std::path::PathBuf {
    if let Some(d) = std::env::var_os("MELEE_DESYNC_DIR").filter(|d| !d.is_empty()) {
        return d.into();
    }
    match melee_platform::yougame::data_dir() {
        Some(d) => std::path::Path::new(d).join("desyncs"),
        None => "desyncs".into(),
    }
}

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => o += &format!("\\u{:04x}", c as u32),
            c => o.push(c),
        }
    }
    o
}

fn kind_name(k: Compare) -> &'static str {
    match k {
        Compare::Hard => "hard",
        _ => "soft",
    }
}

/// An online compare found a Soft or a Hard mismatch. Cheap unless it is the first of its kind in the game, which
/// builds the report and the .slp bytes here (in memory) and hands both to a worker thread.
///
/// `tag`: "decider" / "guest"; `fr`: the online frame compared; `cur`: the session's current online frame.
pub fn on_compare(kind: Compare, tag: &str, fr: i32, lv: u32, rv: u32, cur: i32, stats: &RbStats, bits: &dyn Fn() -> String) {
    if kind == Compare::Same || !enabled() {
        return;
    }
    let Ok(mut s) = st().lock() else { return };
    let hard = kind == Compare::Hard;
    if s.ended || (s.seen_first && !(hard && !s.seen_hard)) {
        return;
    }
    let _sys = melee_engine::arena::SysScope::enter();
    let first = !s.seen_first;
    s.seen_first = true;
    s.seen_hard |= hard;
    let dir = new_dir(&mut s.claimed);
    let report = build_report(&s, kind, tag, fr, lv, rv, cur, stats, hard);
    let slp = crate::slp_export::snapshot_bytes();
    eprintln!(
        "melee: {tag}: DESYNC CAPTURE ({}{}): online frame {fr}: local {lv:08x} remote {rv:08x}; report and {} to {}",
        kind_name(kind),
        if first { "" } else { ", first Hard" },
        if slp.is_some() { "the match so far" } else { "no .slp (the recording is off)" },
        dir.display()
    );
    if !hard {
        s.soft = Some(Event { dir: dir.clone(), report: report.clone() });
    }
    write_async(&mut s, dir, report, slp, bits());
}

fn new_dir(claimed: &mut Vec<std::path::PathBuf>) -> std::path::PathBuf {
    let t = crate::slp_export::iso_now().replace([':', '-'], "");
    let t = t.trim_end_matches('Z').to_string();
    let r = root();
    let mut d = r.join(&t);
    let mut k = 2;
    while d.exists() {
        d = r.join(format!("{t}-{k}"));
        k += 1;
    }
    // a name claimed in this process but not yet written by its worker
    while claimed.contains(&d) {
        d = r.join(format!("{t}-{k}"));
        k += 1;
    }
    claimed.push(d.clone());
    d
}

#[allow(clippy::too_many_arguments)]
fn build_report(s: &State, kind: Compare, tag: &str, fr: i32, lv: u32, rv: u32, cur: i32, st: &RbStats, hard: bool) -> String {
    let mut j = String::from("{\n");
    let mut put = |k: &str, v: String| {
        j += &format!("  \"{k}\": {v},\n");
    };
    put("format", "1".into());
    put("kind", format!("\"{}\"", kind_name(kind)));
    put("tag", format!("\"{}\"", esc(tag)));
    // `online_frame`: the frame number the compare uses (Slippi's `frame` of the checksum packets): the state after the
    // game's (online_frame - 1)th tick. `slippi_frame`: the same state as the .slp's Slippi frame number (-123 is the
    // first tick), which `slpcheck --slippi-checksum-out` and `desync-replay` use.
    put("online_frame", fr.to_string());
    put("slippi_frame", slippi_frame(fr).to_string());
    put("session_online_frame_at_detection", cur.to_string());
    put("local_checksum", format!("\"{lv:08x}\""));
    put("remote_checksum", format!("\"{rv:08x}\""));
    put("build_id", format!("\"{}\"", esc(&format!("melee-rust {} {:015x}", melee_core::VERSION, crate::play_net::build_hash()))));
    put("engine_version", format!("\"{}\"", esc(melee_core::VERSION)));
    put("peer_build", s.peer_build.as_ref().map_or("null".into(), |p| p.1.clone()));
    put("slippi_app_version", if s.app_version.is_empty() { "null".into() } else { format!("\"{}\"", esc(&s.app_version)) });
    if let Some(m) = &s.info {
        let mut p = String::from("[");
        for (i, x) in m.by_port.iter().enumerate() {
            if i > 0 {
                p += ", ";
            }
            p += &format!("{{\"port\": {}, \"character_id\": {}, \"colour\": {}, \"team\": {}, \"local\": {}}}", i, x.character_id, x.character_color, x.team_id, i as u8 == m.local_port);
        }
        p += "]";
        put(
            "match",
            format!(
                "{{\"stage\": {}, \"players\": {p}, \"rng_offset\": {}, \"alt_stage_mode\": {}, \"decider\": {}, \"local_port\": {}, \"rules\": \"{}\", \"scene\": \"{}\"}}",
                m.stage,
                m.rng_offset,
                m.by_port.iter().map(|x| x.alt_stage_mode).max().unwrap_or(0),
                m.decider,
                m.local_port,
                esc(m.scene.split(';').filter(|t| !t.starts_with('p') && !t.starts_with("stage=")).collect::<Vec<_>>().join(";").as_str()),
                esc(&m.scene)
            ),
        );
    } else {
        put("match", "null".into());
    }
    let (cw, cl) = (melee_core::codes::word(), melee_core::codes::lgl_limit());
    put(
        "codes",
        format!(
            "{{\"word\": \"{:08x}\", \"lgl_limit\": {cl}, \"encoded\": \"{}\", \"slippi_codeset\": {}, \"names\": \"{}\"}}",
            cw,
            esc(&melee_core::codes::encode(cw, cl)),
            melee_core::codes::slippi_codeset(),
            esc(&melee_core::codes::describe(cw, cl))
        ),
    );
    put(
        "rollback",
        format!(
            "{{\"rollbacks\": {}, \"resim\": {}, \"depth_max\": {}, \"depth_sum\": {}, \"stall_ticks\": {}, \"waits\": {}, \"held\": {}, \"late_deliveries\": {}, \"ticks\": {}, \"start_hold\": {}}}",
            st.rollbacks, st.resim, st.depth_max, st.depth_sum, st.stall_ticks, st.waits, st.held, st.late_deliveries, st.ticks, st.start_hold
        ),
    );
    put("state_dump", format!("\"{}\"", esc(&melee_fighter::slippi_online::state_dump())));
    // what the .slp holds: from frame -123 through the last confirmed frame; a Soft's is rewritten with the whole match at
    // the game's end
    let rec = crate::slp_export::recorded_frames();
    put("slp", if rec.is_some() { "\"desync.slp\"".into() } else { "null".into() });
    put("slp_last_frame_at_detection", rec.map_or("null".into(), |r| r.1.to_string()));
    put("slp_through", if hard { "\"detection\"".into() } else { "\"game_end\"".into() });
    j.truncate(j.len() - 2);
    j + "\n}\n"
}

/// `bits.txt`: the seed-bit bytes of the frames where a port rolled a random special (TriggerSendInput section 11 read
/// the pads' 9th byte), one line per frame, `<tick> <b0> <b1> <b2> <b3>` with the tick counted from 0 at the match's
/// first tick (Slippi frame -123), bytes in hex. The .slp has no such byte; a replay needs them to draw the same seeds.
pub fn bits_text(mixed: impl Iterator<Item = (i32, [u8; 4])>) -> String {
    let mut t = String::new();
    for (sf, b) in mixed {
        t += &format!("{} {:02x} {:02x} {:02x} {:02x}\n", sf - melee_net::session::FIRST, b[0], b[1], b[2], b[3]);
    }
    t
}

/// The Slippi frame number whose post-frame state online frame `of`'s checksum covers: online frame 2 is the state
/// after the first tick (Slippi frame -123).
pub fn slippi_frame(of: i32) -> i32 {
    of - 125
}

fn write_async(s: &mut State, dir: std::path::PathBuf, report: String, slp: Option<Vec<u8>>, bits: String) {
    // the worker is outside the game: its allocations never go to the arena
    let h = std::thread::spawn(move || {
        melee_engine::arena::mark_foreign_thread();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("melee: desync capture: {}: {e}", dir.display());
            return;
        }
        if let Err(e) = std::fs::write(dir.join("report.json"), report) {
            eprintln!("melee: desync capture: report.json: {e}");
        }
        if let Err(e) = std::fs::write(dir.join("bits.txt"), bits) {
            eprintln!("melee: desync capture: bits.txt: {e}");
        }
        if let Some(b) = slp {
            if let Err(e) = std::fs::write(dir.join("desync.slp"), b) {
                eprintln!("melee: desync capture: desync.slp: {e}");
            }
        }
    });
    s.jobs.push(h);
}

/// The game is over (call before `slp_export::stop`): a Soft's .slp is rewritten with the whole match, and the
/// workers are waited for (the files are on disk when this returns). Safe to call twice.
pub fn end_game(bits: &dyn Fn() -> String) {
    let Ok(mut s) = st().lock() else { return };
    if s.ended {
        return;
    }
    s.ended = true;
    let _sys = melee_engine::arena::SysScope::enter();
    if let Some(ev) = s.soft.take() {
        if let Some(b) = crate::slp_export::snapshot_bytes() {
            let rec = crate::slp_export::recorded_frames();
            let report = ev.report.replace("\"slp_through\": \"game_end\"", &format!("\"slp_through\": \"game_end\",\n  \"slp_last_frame_at_end\": {}", rec.map_or(0, |r| r.1)));
            write_async(&mut s, ev.dir, report, Some(b), bits());
        }
    }
    let jobs = std::mem::take(&mut s.jobs);
    drop(s);
    for j in jobs {
        let _ = j.join();
    }
}
