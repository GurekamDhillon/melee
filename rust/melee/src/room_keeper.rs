//! Keeps a YouGame match room open from the online select through the match (Hero: the room "stays open afterwards
//! for results (game_started / game_ended) until end_session()", yougame_link.h).
//!
//! Hero runs the select and the match in one process. Ours relaunches the window for the match (the match needs the
//! rollback arena from boot), so the menu process stays: when the window relaunches, this process closes its window
//! and keeps only the room, the YouGame bridge (the app still sees the process it started) and the match process as
//! its child. The child writes what the room needs to a report file (`MELEE_ROOM_REPORT`): `start` when the match
//! starts, `end W` when it ends (W the 0-based winning port, -1 none). The keeper turns those into Hero's
//! `game_started` / `game_ended`: the host sends `begin`, everyone `finish` with the winner's participant id, or
//! `void` when the match ends with no winner or the child quits before an end. Then it leaves the room.
//!
//! In-process (the default with the one-window online loop, `slippi_link::inprocess`; `MELEE_YOUGAME_INPROCESS=0`
//! keeps the relaunch above): the match runs in this process and window. Its `start` / `end W` lines come to the
//! keeper in memory ([`report`]) instead of a file, and the match's end in this window ([`match_done`]) stands for
//! the child's exit. The keeper thread, the room relay and the punch's port are the same either way.

use melee_net::yougame_relay::{is_frame, Relay};
use melee_net::yougame_room as yr;
use melee_platform::yougame_mm as ym;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

enum Cmd {
    /// The match process: its handle and the file it reports to.
    Child(std::process::Child, PathBuf),
    /// The match runs in this process (`attach_inprocess`): reports come through [`report`].
    InProcess,
    /// No match follows (cancel, error): leave the room.
    Close,
    /// The process is ending: leave once the match in hand is reported (no next game).
    Finish,
}

/// The in-process match's report lines, and whether it is over (`match_done`).
/// (A SysStatic: `report` runs inside the rollback session, whose loads rewind plain statics.)
static INPROC_LINES: melee_engine::arena::SysStatic<Vec<String>> = melee_engine::arena::SysStatic::new(Vec::new);
static INPROC_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static INPROC_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// The set goes on in this window (Hero: the room stays for the next game, `afterMatch: stay`): stay in the room.
static NEXT_GAME: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The in-process match's set goes on (browser.rs `slippi_set_back`): the room stays for the next game.
pub fn next_game() {
    if INPROC_ON.load(std::sync::atomic::Ordering::SeqCst) {
        NEXT_GAME.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// YouGame rooms play their matches in this process too (`MELEE_YOUGAME_INPROCESS=0`: the relaunch per match).
pub fn wants_inprocess() -> bool {
    let off = std::env::var("MELEE_YOUGAME_INPROCESS").is_ok_and(|v| v == "0");
    // the relaunch hands the Slippi link over loopback TCP (slippi_link::keep), which the YouGame sandbox denies
    if off && melee_platform::yougame::sandboxed() {
        eprintln!("melee: MELEE_YOUGAME_INPROCESS=0 is ignored under YouGame (its relaunch handoff needs loopback, which the sandbox denies)");
        return true;
    }
    !off
}

/// The kept room's match runs in this process from now on.
pub fn attach_inprocess() {
    INPROC_LINES.with(|v| v.clear());
    INPROC_DONE.store(false, std::sync::atomic::Ordering::SeqCst);
    if let Some(k) = KEEPER.lock().unwrap().as_ref() {
        INPROC_ON.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = k.tx.send(Cmd::InProcess);
    }
}

/// An in-process match is reporting to a kept room (play_net's `room_report` sends its lines here).
pub fn inprocess_attached() -> bool {
    INPROC_ON.load(std::sync::atomic::Ordering::SeqCst)
}

/// A line from the in-process match (`start`, `end W`).
pub fn report(line: &str) {
    INPROC_LINES.with(|v| v.push(line.to_string()));
}

/// The in-process match is over (back to the select, or it could not start): the child's exit, in-process.
pub fn match_done() {
    if INPROC_ON.swap(false, std::sync::atomic::Ordering::SeqCst) {
        INPROC_DONE.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

struct Keeper {
    tx: mpsc::Sender<Cmd>,
    thread: std::thread::JoinHandle<()>,
}

static KEEPER: Mutex<Option<Keeper>> = Mutex::new(None);

fn log(s: &str) {
    eprintln!("yougame room: {s}");
}

/// Take over the room the search joined; it is pumped on a thread of its own from now on.
pub fn start(j: yr::Joined, opener: yr::WsOpener, mode: i32, relay: Option<Relay>) {
    close(); // a room from an earlier search
    let (tx, rx) = mpsc::channel();
    let thread = std::thread::Builder::new().name("yougame-room".into()).spawn(move || run(j, opener, mode, relay, rx)).expect("room thread");
    *KEEPER.lock().unwrap() = Some(Keeper { tx, thread });
}

/// A room is being kept.
pub fn active() -> bool {
    KEEPER.lock().unwrap().as_ref().is_some_and(|k| !k.thread.is_finished())
}

/// The match runs in `child`, reporting to `report`.
pub fn attach(child: std::process::Child, report: PathBuf) {
    if let Some(k) = KEEPER.lock().unwrap().as_ref() {
        let _ = k.tx.send(Cmd::Child(child, report));
    }
}

/// Leave the room now (the select was cancelled or failed).
pub fn close() {
    if let Some(k) = KEEPER.lock().unwrap().take() {
        let _ = k.tx.send(Cmd::Close);
        let _ = k.thread.join();
    }
}

/// At the end of `play`: when a match process holds the room, wait for it (the window is gone by now); else leave.
/// Returns the match process's exit code when there was one.
pub fn finish() -> Option<i32> {
    // an in-process match still on when the window closed: it is over now, and no next game follows
    match_done();
    NEXT_GAME.store(false, std::sync::atomic::Ordering::SeqCst);
    let k = KEEPER.lock().unwrap().take()?;
    let _ = k.tx.send(Cmd::Finish);
    let _ = k.thread.join();
    CHILD_CODE.lock().unwrap().take()
}

static CHILD_CODE: Mutex<Option<i32>> = Mutex::new(None);

/// A report file for the match process.
pub fn report_path() -> PathBuf {
    let base = melee_platform::yougame::data_dir().map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join(format!("room-report-{}.txt", std::process::id()))
}

fn run(mut j: yr::Joined, mut opener: yr::WsOpener, mode: i32, mut relay: Option<Relay>, rx: mpsc::Receiver<Cmd>) {
    let no = || false;
    let mut claim = || true;
    let mut waiting = |_: i32| {};
    let mut renew = || ym::request_ticket(&mut ym::StdioBridge, mode, crate::yougame_search::compat(), None).ok().map(|t| t.ticket);
    let mut say = |s: &str| eprintln!("{s}");
    let mut h = yr::Hooks { cancelled: &no, claim: &mut claim, searching: &mut waiting, renew: &mut renew, now: &yr::now_ms, log: &mut say };
    let mut child: Option<(std::process::Child, PathBuf)> = None;
    let mut inproc = false;
    let mut games = 0u32;
    let mut finishing = false;
    let (mut started, mut ended) = (false, false);
    let mut seen = 0usize;
    let mut done_at: Option<Instant> = None;
    let mut last_ping = Instant::now();
    let mut last_report = Instant::now();
    if relay.is_some() {
        log("room relay on: the match's datagrams go through this room");
    }
    log(&format!("kept: room {}, seat {}, host {}", j.code, j.seat, j.room.is_host()));
    loop {
        match rx.try_recv() {
            Ok(Cmd::Child(c, p)) => {
                log("the match runs in the relaunched window; this process keeps the room");
                child = Some((c, p));
            }
            Ok(Cmd::InProcess) => {
                log(&format!("the match runs in this window (no relaunch); the room stays with it (game {})", games + 1));
                inproc = true;
            }
            Ok(Cmd::Finish) => {
                finishing = true;
                // between two games of a set in this window: nothing in hand, leave now
                if child.is_none() && !started {
                    break;
                }
            }
            Ok(Cmd::Close) => break,
            Err(mpsc::TryRecvError::Disconnected) if child.is_none() && !inproc => break,
            Err(_) => {}
        }
        // the room relay (Hero's yougame_room_relay.h): datagrams out every 16 ms, frames in as they come
        if let Some(r) = relay.as_mut() {
            for f in r.poll(yr::now_ms()) {
                j.socket.send(&f);
            }
        }
        let wait = Duration::from_millis(if relay.is_some() { 2 } else { 20 });
        for e in yr::pump(&mut j, &mut opener, &mut h, wait) {
            match e {
                yr::Event::Message { d, .. } if is_frame(&d) => {
                    if let Some(r) = relay.as_mut() {
                        r.receive(&d);
                    }
                }
                yr::Event::Started(id) => log(&format!("start {id}")),
                yr::Event::Result(id) => {
                    log(&format!("result {id}"));
                    if ended {
                        done_at.get_or_insert_with(Instant::now);
                    }
                }
                yr::Event::Refused { action, error } => log(&format!("{action} refused: {error}")),
                yr::Event::Ended(t) => {
                    log(&format!("the room ended ({t})"));
                    INPROC_ON.store(false, std::sync::atomic::Ordering::SeqCst);
                    if let Some((c, _)) = child.as_mut() {
                        *CHILD_CODE.lock().unwrap() = c.wait().ok().and_then(|s| s.code());
                    }
                    return;
                }
                _ => {}
            }
        }
        if last_ping.elapsed() > Duration::from_secs(5) {
            j.socket.send(&format!(r#"{{"t":"ping","at":{}}}"#, yr::now_ms()));
            last_ping = Instant::now();
        }
        // the app asked us to stop: the match is abandoned
        let quit = melee_platform::yougame::quit_requested();
        if last_report.elapsed() < Duration::from_millis(50) {
            continue;
        }
        last_report = Instant::now();
        let polled: Option<(Vec<String>, bool)> = if let Some((c, p)) = child.as_mut() {
            let text = std::fs::read_to_string(&*p).unwrap_or_default();
            let exited = match c.try_wait() {
                Ok(Some(s)) => {
                    *CHILD_CODE.lock().unwrap() = s.code();
                    true
                }
                Ok(None) => false,
                Err(_) => true,
            };
            if quit && !exited {
                let _ = c.kill();
            }
            Some((text.lines().map(str::to_string).collect(), exited))
        } else if inproc {
            Some((INPROC_LINES.with(|v| v.clone()), INPROC_DONE.load(std::sync::atomic::Ordering::SeqCst)))
        } else {
            None
        };
        if let Some((lines, exited)) = polled {
            for l in lines.iter().skip(seen) {
                if l == "start" && !started {
                    started = true;
                    j.room.game_started();
                    log("the match started: begin");
                } else if let Some(w) = l.strip_prefix("end ") {
                    if started && !ended {
                        ended = true;
                        let w: i32 = w.trim().parse().unwrap_or(-1);
                        j.room.game_ended(w);
                        log(&format!("the match ended: {}", if w >= 0 { format!("port {} won", w + 1) } else { "no winner (void)".into() }));
                    }
                }
            }
            seen = lines.len();
            if (exited || quit) && !ended {
                ended = true;
                if started {
                    j.room.game_ended(-1);
                    log(if inproc { "the match left this window before it ended: void" } else { "the match process ended before the match did: void" });
                }
                done_at.get_or_insert_with(Instant::now);
            }
            // in-process: the result can come while the window is already back on the select
            let exited = exited || (inproc && ended);
            if exited && ended {
                // the result (or 5 s for it), then leave
                let at = *done_at.get_or_insert_with(Instant::now);
                // no match ever started: nothing to report. Else finish went out (due() clears the match id).
                if !started || at.elapsed() > Duration::from_secs(5) || (j.room.match_id.is_empty() && at.elapsed() > Duration::from_millis(1500)) {
                    // a set in this window goes on: the same room for its next game (Hero's `afterMatch: stay`)
                    if inproc && started && !finishing && NEXT_GAME.swap(false, std::sync::atomic::Ordering::SeqCst) {
                        games += 1;
                        log(&format!("game {games} reported; the set goes on: the room stays for game {}", games + 1));
                        (started, ended, seen, done_at) = (false, false, 0, None);
                        INPROC_LINES.with(|v| v.clear());
                        INPROC_DONE.store(false, std::sync::atomic::Ordering::SeqCst);
                        continue;
                    }
                    break;
                }
            }
        } else if quit {
            break;
        }
    }
    if let Some(r) = &relay {
        log(&format!("room relay ended ({} sent, {} received, {} dropped)", r.sent, r.received, r.dropped));
    }
    j.socket.send(r#"{"t":"leave"}"#);
    log("left the room");
    INPROC_ON.store(false, std::sync::atomic::Ordering::SeqCst);
    if let Some((_, p)) = child {
        let _ = std::fs::remove_file(p);
    }
}
