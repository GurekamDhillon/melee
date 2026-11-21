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

pub fn take_handoff() -> Option<Vec<String>> {
    HANDOFF.lock().ok()?.take()
}

fn arg(a: &[String], k: &str) -> Option<String> {
    a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned()
}

/// Start searching. `args` are `play`'s `--slippi-*` options (the same the match used to take); `mode` the online
/// menu's mode. The screen reads the state through `slippi_css::with`.
pub fn start(args: Vec<String>, mode: i32) {
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
    // the room stays open while the select is up (dropped with the session: the match runs in the relaunched window)
    let _room = found.room;
    let p = found.paired;
    if sc::with(|o| o.cancel) {
        eprintln!("online css: session: cancelled after matchmaking");
        return Ok(());
    }
    eprintln!("online css: session: paired with {}, local player {}, decider {}", if p.opp_code.is_empty() { "?" } else { &p.opp_code }, p.local_port, p.decider);
    sc::with(|o| {
        o.conn = sc::CONN_OPPONENT_CONNECTING;
        o.opp_code = p.opp_code.clone();
    });
    let udp = UdpTransport::open(p.bind_ip, port).map_err(|e| format!("bind: {e}"))?;
    let mut cl = Client::new(Box::new(udp), p.local_port, p.peer, p.remote_port);
    cl.connect_timeout_ms = 30000;
    let t0 = Instant::now();
    let (mut sent, mut theirs) = (None::<(u8, u8)>, None::<Selections>);
    let mut both_at = None::<Instant>;
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
            eprintln!("online css: session: cancelled, link closed");
            return Ok(());
        }
        match cl.status {
            Status::Connected => {
                if sc::with(|o| o.conn) != sc::CONN_SUCCESS {
                    eprintln!("online css: session: connected after {:.2}s", t0.elapsed().as_secs_f32());
                    sc::with(|o| o.conn = sc::CONN_SUCCESS);
                }
            }
            Status::Failed | Status::Disconnected => return Err(format!("the opponent's link {:?}", cl.status)),
            _ => {}
        }
        if let (Some((ck, color)), None, Status::Connected) = (lock, sent, cl.status) {
            cl.set_match_selections(Selections {
                character_id: ck,
                character_color: color,
                is_character_selected: true,
                player_idx: p.local_port,
                ..Default::default()
            });
            sent = Some((ck, color));
            sc::with(|o| o.local_ready = true);
        }
        if sent.is_some() && theirs.is_some() {
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
    eprintln!("online css: session: both locked in, the match next");
    *HANDOFF.lock().unwrap() = Some(m);
    Ok(())
}
