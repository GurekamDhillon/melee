//! The Slippi spectator feed served by our game, and a spectator that watches one (docs/rust-port/notes/net.md,
//! "Slippi spectating"). The protocol and its state are `melee_net::slippi_spectate`; this file owns the sockets.
//!
//! Broadcast (Hero's `slippi::spectate`, Slippi Dolphin's spectator server): while a match is exported as `.slp`
//! (`--slp`, which the launcher's `--replay-dir` turns on), `slp_export` hands every event group it writes to [`write`]
//! here, and a thread serves them on 127.0.0.1 (loopback only, so no firewall prompt) over ENet to the Slippi
//! Launcher, stream overlays, or our own spectator. Settings: `slippi_spectator` (default on; off under YouGame, as
//! Hero's), `slippi_spectator_port` (default 51441). The feed is fed only frames already final (a rollback run's
//! confirmed frames), and only reads: it never touches the game, its inputs or its timing (a queue push per group).
//!
//! Watch (`melee slp-spectate --connect 127.0.0.1:PORT --out FILE.slp`): connects, asks from the current game's start,
//! and writes the stream as a `.slp` while it arrives (raw length 0 while in progress, as Dolphin writes one), the
//! finished file at the game's end. `melee play --replay FILE.slp` plays such a file while it is still being written
//! (slp_play.rs live mode), which is what Hero's launcher "Watch live" does with the Slippi Launcher's Spectate folder.
//! The spectator only reads: it sends nothing but its connect request.

use melee_engine::arena::SysStatic;
use melee_net::slippi_spectate::{connect_request, Feed, Input, Seen, Watcher, CHANNELS, DEFAULT_PORT, MAX_CLIENTS};
use melee_net::{Addr, UdpTransport};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::{Duration, Instant};

static QUEUE: SysStatic<Vec<Input>> = SysStatic::new(Vec::new);
static THREAD: SysStatic<Option<std::thread::JoinHandle<()>>> = SysStatic::new(|| None);
static RUNNING: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);
static PORT: AtomicU16 = AtomicU16::new(0);

fn log(m: &str) {
    eprintln!("spectator: {m}");
}

fn push(i: Input) {
    if RUNNING.load(Ordering::Acquire) {
        QUEUE.with(|q| q.push(i));
    }
}

/// One group of recording events (slp_export: Event Payloads + Game Start, a Splitter, a frame, Game End).
pub fn write(bytes: &[u8]) {
    if RUNNING.load(Ordering::Relaxed) {
        push(Input::Events(bytes.to_vec()));
    }
}
pub fn start_game() {
    push(Input::StartGame);
}
pub fn end_game(closed: bool) {
    push(Input::EndGame { closed });
}
pub fn running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}
pub fn port() -> u16 {
    PORT.load(Ordering::Relaxed)
}

/// Start the server when the setting allows (once per run). Call before the engine boots: the statics are made then,
/// before any rollback snapshot (a load from before their first use would reset them).
pub fn start() {
    if RUNNING.load(Ordering::Acquire) {
        return;
    }
    if !crate::cfg::flag("slippi_spectator", true) {
        log("off in settings");
        return;
    }
    if melee_platform::yougame::active() {
        log("off under YouGame");
        return;
    }
    let port = crate::cfg::open().get_int("slippi_spectator_port", DEFAULT_PORT as i32);
    let Ok(port) = u16::try_from(port).map(|p| p.max(1)) else {
        log(&format!("port {port} is not valid; no spectator server"));
        return;
    };
    QUEUE.with(|q| q.clear());
    STOP.store(false, Ordering::Release);
    RUNNING.store(true, Ordering::Release);
    let h = std::thread::Builder::new().name("slippi spectator".into()).spawn(move || serve(port));
    match h {
        Ok(h) => THREAD.with(|t| *t = Some(h)),
        Err(e) => {
            RUNNING.store(false, Ordering::Release);
            log(&format!("no thread: {e}"));
        }
    }
}

/// The run ends: `end_game` with `dolphin_closed`, the last messages out, the thread joined.
pub fn stop() {
    if !RUNNING.load(Ordering::Acquire) {
        return;
    }
    end_game(true);
    STOP.store(true, Ordering::Release);
    if let Some(h) = THREAD.with(|t| t.take()) {
        let _ = h.join();
    }
    RUNNING.store(false, Ordering::Release);
}

fn serve(port: u16) {
    let mut tp = None;
    for _ in 0..10 {
        match UdpTransport::open(0x7F000001, port) {
            Ok(t) => {
                tp = Some(t);
                break;
            }
            Err(_) if !STOP.load(Ordering::Acquire) => std::thread::sleep(Duration::from_millis(500)),
            Err(_) => break,
        }
    }
    let Some(tp) = tp else {
        log(&format!("could not open port {port} (is Slippi Dolphin or another copy running?)"));
        RUNNING.store(false, Ordering::Release);
        return;
    };
    let mut host = melee_net::enet::Host::new(Box::new(tp), MAX_CLIENTS, CHANNELS);
    let mut feed = Feed::new("Slippi Online", "3.18.0");
    PORT.store(port, Ordering::Relaxed);
    log(&format!("serving the game's Slippi feed on 127.0.0.1 port {port}"));
    let mut closing_at: Option<Instant> = None;
    loop {
        for i in QUEUE.with(std::mem::take) {
            feed.push(i);
        }
        while let Some(ev) = host.service() {
            match ev {
                melee_net::enet::Event::Connect { peer, .. } => {
                    log("a client connected");
                    feed.connect(peer);
                }
                melee_net::enet::Event::Disconnect { peer, .. } => {
                    log("a client disconnected");
                    feed.disconnect(peer);
                }
                melee_net::enet::Event::Receive { peer, data, .. } => {
                    if let Some(r) = feed.message(peer, &data) {
                        host.send(peer, 0, r.as_bytes(), true);
                    }
                }
            }
        }
        for (peer, m) in feed.drain() {
            if !host.send(peer, 0, m.as_bytes(), true) {
                log(&format!("a message of {} bytes could not be sent", m.len()));
            }
        }
        host.flush();
        if STOP.load(Ordering::Acquire) {
            // let the last messages (end_game) be delivered and acked, up to a second
            let t = *closing_at.get_or_insert_with(Instant::now);
            let owed = (0..host.peers.len()).any(|p| host.has_outgoing(p));
            if !owed || t.elapsed() > Duration::from_secs(1) {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    PORT.store(0, Ordering::Relaxed);
}

/// `melee slp-spectate --connect HOST:PORT --out FILE.slp [--games N] [--seconds S]`: watch a feed into a `.slp`.
/// Exit 0 when the games were watched to their end.
pub fn watch_cmd(args: &[String]) -> std::process::ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(to) = get("--connect").and_then(|s| s.parse::<std::net::SocketAddr>().ok()) else {
        eprintln!("usage: melee slp-spectate --connect 127.0.0.1:51441 --out FILE.slp [--games N] [--seconds S]");
        return std::process::ExitCode::from(2);
    };
    let Some(out) = get("--out") else {
        eprintln!("slp-spectate: --out FILE.slp");
        return std::process::ExitCode::from(2);
    };
    let games: u32 = get("--games").and_then(|s| s.parse().ok()).unwrap_or(1);
    let secs: u64 = get("--seconds").and_then(|s| s.parse().ok()).unwrap_or(600);
    match watch(to, &out, games, Duration::from_secs(secs)) {
        Ok(n) => {
            eprintln!("slp-spectate: {n} game(s) watched");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("slp-spectate: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// The spectator: connect, ask from the game's start, write each game to `out` (a second game gets `-2`, and so on)
/// as it arrives. Returns the games watched to their end.
pub fn watch(to: std::net::SocketAddr, out: &str, games: u32, limit: Duration) -> Result<u32, String> {
    let tp = UdpTransport::open(0x7F000001, 0).map_err(|e| format!("bind: {e}"))?;
    let mut host = melee_net::enet::Host::new(Box::new(tp), 1, CHANNELS);
    let peer = host.connect(Addr::from_socket(to), CHANNELS, 0).ok_or("no peer slot")?;
    let t0 = Instant::now();
    let mut w = Watcher::default();
    let mut connected = false;
    let mut last_write = Instant::now();
    let mut dirty = false;
    let path_for = |n: u32| -> String {
        if n == 0 {
            return out.to_string();
        }
        let p = std::path::Path::new(out);
        let stem = p.file_stem().map_or("game".into(), |s| s.to_string_lossy().to_string());
        p.with_file_name(format!("{stem}-{}.slp", n + 1)).to_string_lossy().to_string()
    };
    let mut events = 0u64;
    while w.games < games {
        if t0.elapsed() > limit {
            return Err(format!("timed out after {} s ({} games, {events} event groups)", limit.as_secs(), w.games));
        }
        while let Some(ev) = host.service() {
            match ev {
                melee_net::enet::Event::Connect { .. } => {
                    connected = true;
                    eprintln!("slp-spectate: connected; asking from cursor {}", w.next_cursor);
                    host.send(peer, 0, connect_request(w.next_cursor).as_bytes(), true);
                }
                melee_net::enet::Event::Disconnect { .. } => return Err(format!("the feed closed the link ({} games)", w.games)),
                melee_net::enet::Event::Receive { data, .. } => match w.message(&data) {
                    Seen::Reply { nick, version, cursor } => eprintln!("slp-spectate: feed \"{nick}\" {version}, from cursor {cursor}"),
                    Seen::GameStart => eprintln!("slp-spectate: game {} starts", w.games + 1),
                    Seen::Events(_) => {
                        events += 1;
                        dirty = true;
                    }
                    Seen::GameEnd { closed } => {
                        let p = path_for(w.games - 1);
                        std::fs::write(&p, w.slp(true)).map_err(|e| format!("{p}: {e}"))?;
                        eprintln!("slp-spectate: game {} ended{}: {} bytes of events to {p}", w.games, if closed { " (the game closed)" } else { "" }, w.raw.len());
                        dirty = false;
                    }
                    _ => {}
                },
            }
        }
        if dirty && w.in_game && last_write.elapsed() > Duration::from_millis(100) {
            let p = path_for(w.games);
            let tmp = format!("{p}.part");
            // whole events only, replaced atomically: a player reading it never sees half a write
            if std::fs::write(&tmp, w.slp(false)).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
            last_write = Instant::now();
            dirty = false;
        }
        if !connected && t0.elapsed() > Duration::from_secs(10) {
            return Err("no answer from the feed".into());
        }
        host.flush();
        std::thread::sleep(Duration::from_millis(1));
    }
    host.disconnect(peer, 0);
    host.flush();
    Ok(w.games)
}
