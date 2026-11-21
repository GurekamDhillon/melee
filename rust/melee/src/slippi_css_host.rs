//! The host half of the Slippi online select (`melee_modes::tu::slippi_css`): the session that searches while the
//! screen is up, holds the link to the opponent, sends this side's lock-in and reads theirs (Slippi's B5 / match
//! selections), as Hero's host does behind `mu_slippi_cmd`. When both are locked in, the session closes its link and
//! leaves the arguments of the match (`play --slippi-peer ...`), which connects to the same opponent again with the
//! fighters picked here.

use melee_modes::tu::slippi_css as sc;
use melee_net::slippi::{Client, Note, Selections, Status};
use melee_net::transport::UdpTransport;
use melee_platform::yougame_mm as ym;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The match's arguments, once both are locked in.
static HANDOFF: Mutex<Option<Vec<String>>> = Mutex::new(None);
/// Both sides locked in: from then on this process is handing over to the match.
static LOCKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The game of the set the handoff's match is (0: the first), as it was at the lock-in.
static HANDOFF_GAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub fn handoff_game() -> u32 {
    HANDOFF_GAME.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn take_handoff() -> Option<Vec<String>> {
    HANDOFF.lock().ok()?.take()
}

fn arg(a: &[String], k: &str) -> Option<String> {
    a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned()
}

/// Start searching. `args` are `play`'s `--slippi-*` options (the same the match used to take); `mode` the online
/// menu's mode. The screen reads the state through `slippi_css::with`.
pub fn start(args: Vec<String>, mode: i32) {
    // both are locked in and the match is on its way: the select was left for the match, and a menu re-entered
    // meanwhile must not start a second search (it would queue again in YouGame's pool)
    if LOCKED.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("online css: session: the match is on its way; no new search");
        return;
    }
    let search = arg(&args, "--slippi-search").unwrap_or_default();
    sc::begin(mode, &search);
    *HANDOFF.lock().unwrap() = None;
    std::thread::spawn(move || {
        if let Err(e) = run(&args) {
            eprintln!("online css: session: {e}");
            sc::with(|o| {
                o.conn = sc::CONN_ERROR;
                o.error = e;
            });
        }
    });
}

/// The set goes on: the match process came back to the online select with the same opponent (Hero: after a game the
/// select comes back, connected, the loser picking the next stage). `MELEE_SLIPPI_REMATCH` (set by the match process,
/// `rematch_env`): `peer;port;local_port;decider;mode;opp_code;winner;game`. Returns the mode when it was there.
pub fn start_rematch() -> Option<i32> {
    let v = std::env::var("MELEE_SLIPPI_REMATCH").ok().filter(|v| !v.is_empty())?;
    std::env::remove_var("MELEE_SLIPPI_REMATCH");
    let f: Vec<&str> = v.split(';').collect();
    if f.len() < 8 {
        eprintln!("online css: rematch: malformed {v}");
        return None;
    }
    let (peer, port, local_port, decider) = (f[0].to_string(), f[1].parse::<u16>().unwrap_or(0), f[2].parse::<u8>().unwrap_or(0), f[3] == "1");
    let (mode, opp, winner, game) = (f[4].parse::<i32>().unwrap_or(0), f[5].to_string(), f[6].parse::<u8>().unwrap_or(sc::WINNER_NULL), f[7].parse::<u32>().unwrap_or(1));
    sc::begin(mode, "");
    sc::with(|o| {
        o.opp_code = opp.clone();
        o.winner = winner;
        o.game = game;
        o.conn = sc::CONN_OPPONENT_CONNECTING;
    });
    *HANDOFF.lock().unwrap() = None;
    let max_s: u64 = std::env::var("MELEE_SLIPPI_NP_MAX_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    eprintln!("online css: rematch: game {} against {}, last game {}", game + 1, if opp.is_empty() { "?" } else { &opp }, ["none", "won", "lost"][winner.min(2) as usize]);
    std::thread::spawn(move || {
        let mut a: Vec<String> = vec!["--peer".into(), peer, "--local-port".into(), local_port.to_string()];
        if decider {
            a.push("--host".into());
        }
        let r = crate::slippi_np::pair(&a, max_s).map_err(|(_, m)| m).and_then(|mut p| {
            p.decider = decider;
            p.opp_code = opp;
            link(&p, port, max_s).map(|_| ())
        });
        if let Err(e) = r {
            eprintln!("online css: session: {e}");
            sc::with(|o| {
                o.conn = sc::CONN_ERROR;
                o.error = e;
            });
        }
    });
    Some(mode)
}

/// The environment the match process takes back to the online select after its game (`start_rematch`): `None` when
/// this match is not part of an online-select set (`MELEE_SLIPPI_SET`, set at the handoff: `mode;opp_code;game`).
pub fn rematch_env(match_args: &[String], winner_port: i32) -> Option<String> {
    let set = std::env::var("MELEE_SLIPPI_SET").ok().filter(|v| !v.is_empty())?;
    let f: Vec<&str> = set.split(';').collect();
    let (mode, opp, game) = (f.first()?, f.get(1).copied().unwrap_or(""), f.get(2).and_then(|g| g.parse::<u32>().ok()).unwrap_or(0));
    let peer = arg(match_args, "--slippi-peer")?;
    let port = arg(match_args, "--slippi-port").unwrap_or_else(|| "0".into());
    let lp: u8 = arg(match_args, "--slippi-local-port").and_then(|v| v.parse().ok()).unwrap_or(0);
    let decider = match_args.iter().any(|a| a == "--slippi-host");
    let winner = if winner_port < 0 { sc::WINNER_NULL } else if winner_port as u8 == lp { sc::WINNER_WON } else { sc::WINNER_LOST };
    Some(format!("{peer};{port};{lp};{};{mode};{opp};{winner};{}", decider as u8, game + 1))
}

fn run(args: &[String]) -> Result<(), String> {
    // `--slippi-x` -> `--x`, as `NetDriver::connect_slippi` maps them
    let mut sub: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--slippi-route" {
            i += 2;
            continue;
        }
        if let Some(k) = args[i].strip_prefix("--slippi-") {
            sub.push(format!("--{k}"));
            if k != "mm-real" && k != "mm-lan" {
                if let Some(v) = args.get(i + 1) {
                    sub.push(v.clone());
                    i += 1;
                }
            }
        }
        i += 1;
    }
    let max_s: u64 = arg(&sub, "--max-seconds").and_then(|v| v.parse().ok()).unwrap_or(600);
    let port: u16 = arg(&sub, "--port").and_then(|v| v.parse().ok()).unwrap_or(0);
    sc::with(|o| o.conn = sc::CONN_MATCHMAKING);
    // --slippi-route yougame|blend (from the online menu in YouGame mode): YouGame's rooms, or both pools
    let route = match arg(args, "--slippi-route").as_deref() {
        Some("yougame") => ym::Route::YouGame,
        Some("blend") => ym::Route::Blend,
        _ => ym::Route::Slippi,
    };
    let agent = std::env::var_os("MELEE_PAD_SCRIPT").is_some() || std::env::var("MELEE_WINDOW_HIDDEN").is_ok_and(|v| v == "1");
    let mode = sc::with(|o| o.mode);
    let found = crate::yougame_search::pair(route, mode, &sub, max_s, agent, &|| sc::with(|o| o.cancel), &|s| eprintln!("online css: session: {s}"));
    let found = match found {
        Err(e) if e == "Cancelled" => {
            eprintln!("online css: session: search cancelled");
            return Ok(());
        }
        r => r?,
    };
    eprintln!("online css: session: the opponent came from {}", found.side.name());
    // the room stays open from here through the match (crate::room_keeper: this process keeps it while the match
    // runs in the relaunched window); a cancel or an error before the match leaves it
    if let Some((j, opener)) = found.room {
        crate::room_keeper::start(j, opener, mode, found.relay);
    }
    let p = found.paired;
    let r = link(&p, port, max_s);
    if !matches!(r, Ok(true)) {
        crate::room_keeper::close();
    }
    r.map(|_| ())
}

/// The link to the opponent through the select: Ok(true) when both locked in (the handoff is set).
fn link(p: &crate::slippi_np::Paired, port: u16, max_s: u64) -> Result<bool, String> {
    if sc::with(|o| o.cancel) {
        eprintln!("online css: session: cancelled after matchmaking");
        return Ok(false);
    }
    eprintln!("online css: session: paired with {}, local player {}, decider {}", if p.opp_code.is_empty() { "?" } else { &p.opp_code }, p.local_port, p.decider);
    sc::with(|o| {
        o.conn = sc::CONN_OPPONENT_CONNECTING;
        o.opp_code = p.opp_code.clone();
    });
    // the port can still be held a moment by the process before (the last game's match window): retry the bind
    let mut udp = UdpTransport::open(p.bind_ip, port);
    let bind_t = Instant::now();
    while udp.is_err() && port != 0 && bind_t.elapsed() < Duration::from_secs(8) {
        std::thread::sleep(Duration::from_millis(100));
        udp = UdpTransport::open(p.bind_ip, port);
    }
    let udp = udp.map_err(|e| format!("bind: {e}"))?;
    sc::with(|o| o.local_index = p.local_port);
    let mut cl = Client::new(Box::new(udp), p.local_port, p.peer, p.remote_port);
    cl.connect_timeout_ms = 30000;
    let t0 = Instant::now();
    let (mut sent, mut theirs) = (None::<(u8, u8)>, None::<Selections>);
    let mut both_at = None::<Instant>;
    let mut lock_at: Option<(i32, Option<u16>, u32)> = None;
    loop {
        cl.poll();
        while let Some(n) = cl.next_note() {
            if let Note::Selections(s) = n {
                if s.is_character_selected && theirs.is_none() {
                    eprintln!("online css: session: the opponent locked in fighter {} colour {}", s.character_id, s.character_color);
                    theirs = Some(s);
                    sc::with(|o| o.remote_ready = true);
                }
            }
        }
        let (cancel, lock) = sc::with(|o| (o.cancel, o.lock));
        if cancel {
            // B, or hold Z (Hero's cleanup command): the opponent is told, the link closes
            cl.disconnect(0);
            for _ in 0..20 {
                cl.poll();
                std::thread::sleep(Duration::from_millis(2));
            }
            eprintln!("online css: session: {}, link closed", if sc::with(|o| o.disconnect) { "disconnected (hold Z)" } else { "cancelled" });
            return Ok(false);
        }
        match cl.status {
            Status::Connected => {
                if sc::with(|o| o.conn) != sc::CONN_SUCCESS {
                    eprintln!("online css: session: connected after {:.2}s", t0.elapsed().as_secs_f32());
                    sc::with(|o| o.conn = sc::CONN_SUCCESS);
                }
            }
            Status::Disconnected => return Err("the opponent disconnected".into()),
            Status::Failed => return Err(format!("the opponent's link {:?}", cl.status)),
            _ => {}
        }
        if let (Some((ck, color)), None, Status::Connected) = (lock, sent, cl.status) {
            let stage = sc::with(|o| o.lock_stage);
            cl.set_match_selections(Selections {
                character_id: ck,
                character_color: color,
                is_character_selected: true,
                player_idx: p.local_port,
                stage_id: stage.unwrap_or(0),
                is_stage_selected: stage.is_some(),
                ..Default::default()
            });
            sent = Some((ck, color));
            // the set's state as it was at the lock-in (the menus can be re-entered before the handoff)
            lock_at = sc::with(|o| Some((o.mode, o.lock_stage, o.game)));
            sc::with(|o| o.local_ready = true);
        }
        if sent.is_some() && theirs.is_some() {
            LOCKED.store(true, std::sync::atomic::Ordering::SeqCst);
            // a moment for the last acks, then the link closes before the match window binds the same port
            let at = *both_at.get_or_insert_with(Instant::now);
            if at.elapsed() > Duration::from_millis(700) {
                break;
            }
        }
        if t0.elapsed().as_secs() > max_s {
            return Err("timed out".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    drop(cl);
    let (ck, color) = sent.unwrap();
    let mut m: Vec<String> = vec![
        "--slippi-peer".into(),
        p.peer.to_string(),
        "--slippi-port".into(),
        port.to_string(),
        "--slippi-local-port".into(),
        p.local_port.to_string(),
        "--slippi-char".into(),
        ck.to_string(),
        "--slippi-color".into(),
        color.to_string(),
    ];
    if p.decider {
        m.push("--slippi-host".into());
    }
    // Direct: the loser's stage, or none from the winner (the loser's stands); Unranked keeps the match's default
    let (mode, stage, game) = lock_at.unwrap();
    let opp = p.opp_code.clone();
    HANDOFF_GAME.store(game, std::sync::atomic::Ordering::Relaxed);
    if mode == melee_modes::tu::slippi_online::MODE_DIRECT {
        match stage {
            Some(s) => m.extend(["--slippi-stage".into(), s.to_string()]),
            None => m.push("--slippi-stage-unset".into()),
        }
    }
    // the match process comes back to this select after its game (rematch_env / start_rematch)
    // (Direct only: Unranked's link ends with its game, as Slippi's does)
    if mode == melee_modes::tu::slippi_online::MODE_DIRECT {
        std::env::set_var("MELEE_SLIPPI_SET", format!("{mode};{opp};{game}"));
    } else {
        std::env::remove_var("MELEE_SLIPPI_SET");
    }
    eprintln!("online css: session: both locked in, the match next");
    *HANDOFF.lock().unwrap() = Some(m);
    Ok(true)
}
