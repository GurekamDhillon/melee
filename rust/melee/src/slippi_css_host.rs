//! The host half of the Slippi online select (`melee_modes::tu::slippi_css`): the session that searches while the
//! screen is up, holds the link to the opponent, sends this side's lock-in and reads theirs (Slippi's B5 / match
//! selections), as Hero's host does behind `mu_slippi_cmd`. When both are locked in, the session leaves the arguments of
//! the match (`play --slippi-peer ... --slippi-link`) and keeps its link for the match process to take over
//! (slippi_link.rs); a Direct set's next game comes back here on the same link (`start_rematch`).

use melee_modes::tu::slippi_css as sc;
use melee_net::slippi::{Client, Note, Selections, Status};
use melee_net::transport::UdpTransport;
use melee_platform::yougame_mm as ym;
use std::time::{Duration, Instant};

/// The match's arguments, once both are locked in.
/// (Host state: on the system heap, never rewound by a rollback load; see `melee_engine::arena::SysStatic`.)
static HANDOFF: melee_engine::arena::SysStatic<Option<Vec<String>>> = melee_engine::arena::SysStatic::new(|| None);
/// Both sides locked in: from then on this process is handing over to the match.
static LOCKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The game of the set the handoff's match is (0: the first), as it was at the lock-in.
static HANDOFF_GAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub fn handoff_game() -> u32 {
    HANDOFF_GAME.load(std::sync::atomic::Ordering::Relaxed)
}

/// Back on the select after a match played in this process (`slippi_link::inprocess`): a new search or lock-in may
/// start again (the relaunched process started with these cleared).
pub fn reset_for_select() {
    LOCKED.store(false, std::sync::atomic::Ordering::SeqCst);
    HANDOFF.with(|h| *h = None);
}

pub fn take_handoff() -> Option<Vec<String>> {
    HANDOFF.with(|h| h.take())
}

fn arg(a: &[String], k: &str) -> Option<String> {
    a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned()
}

/// The last search's arguments, for a search again from the idle select (after a hold-Z disconnect).
/// (Host state: on the system heap, never rewound by a rollback load.)
static LAST: melee_engine::arena::SysStatic<Option<(Vec<String>, i32)>> = melee_engine::arena::SysStatic::new(|| None);

/// Once per tick: the idle select asked to search again (Start after a hold-Z disconnect).
pub fn poll() {
    if sc::take_search_again() {
        let last = LAST.with(|l| l.clone());
        match last {
            Some((args, mode)) => {
                eprintln!("online css: session: Start with a fighter chosen: searching");
                search(args, mode);
            }
            None => eprintln!("online css: session: no search to repeat"),
        }
    }
}

/// The online select opens: idle until Start with a fighter chosen, which searches (Hero's mu_slippi_css begins
/// matchmaking only then; B before it leaves with no search). `args` are `play`'s `--slippi-*` options; `mode` the
/// online menu's mode. The screen reads the state through `slippi_css::with`.
pub fn start(args: Vec<String>, mode: i32) {
    if LOCKED.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("online css: session: the match is on its way; no new search");
        return;
    }
    LAST.with(|l| *l = Some((args.clone(), mode)));
    set_user_from(&args);
    sc::wait_start(mode, &arg(&args, "--slippi-search").unwrap_or_default());
    HANDOFF.with(|h| *h = None);
}

fn set_user_from(args: &[String]) {
    // the select names the signed-in player (Hero FN_UserTextUpdate): the login's name and connect code, read here
    // and shown on the screen only, never logged
    let (name, code) = match arg(args, "--slippi-user-json") {
        Some(path) => {
            let acct = std::fs::read_to_string(&path).ok().and_then(|t| melee_net::slippi_mm::Json::parse(&t));
            let field = |k: &str| acct.as_ref().and_then(|j| j.get(k)).and_then(|v| v.str()).map(str::to_string).unwrap_or_default();
            (field("displayName"), field("connectCode"))
        }
        None => (arg(args, "--slippi-name").unwrap_or_default(), arg(args, "--slippi-code").unwrap_or_default()),
    };
    sc::set_user(&name, &code);
}

/// Search now (Start on the idle select).
fn search(args: Vec<String>, mode: i32) {
    // both are locked in and the match is on its way: the select was left for the match, and a menu re-entered
    // meanwhile must not start a second search (it would queue again in YouGame's pool)
    if LOCKED.load(std::sync::atomic::Ordering::SeqCst) {
        eprintln!("online css: session: the match is on its way; no new search");
        return;
    }
    LAST.with(|l| *l = Some((args.clone(), mode)));
    std::env::remove_var("MELEE_SLIPPI_MM_NAMES");
    let search = arg(&args, "--slippi-search").unwrap_or_default();
    // begin() clears the screen state: the chosen flag is kept (the fighter stays picked)
    let (chosen, locks) = sc::with(|o| (o.chosen, o.start_locks));
    sc::begin(mode, &search);
    sc::with(|o| {
        o.chosen = chosen;
        o.start_locks = locks;
    });
    HANDOFF.with(|h| *h = None);
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
/// `rematch_env`): `peer;port;local_port;decider;mode;opp_code;winner;game;link`. `link` is the last match's link,
/// kept for this select (`slippi_link::hand_back`): taken over and locked in on, with no new connection (Slippi Dolphin
/// keeps one client through the set and takes no second); without it the select connects again. Returns the mode when
/// it was there.
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
    let kept = f.get(8).filter(|l| !l.is_empty()).map(|l| l.to_string());
    sc::begin(mode, "");
    sc::with(|o| {
        o.opp_code = opp.clone();
        o.winner = winner;
        o.game = game;
        o.conn = sc::CONN_OPPONENT_CONNECTING;
    });
    HANDOFF.with(|h| *h = None);
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
            match &kept {
                Some(spec) => {
                    sc::with(|o| o.opp_code = p.opp_code.clone());
                    let t = crate::slippi_link::take(spec, 0, port, |udp| Box::new(udp))?;
                    eprintln!("online css: rematch: the set's link taken over ({:?}), no new connection", t.cl.status);
                    lock_in(t.cl, &p, port, max_s).map(|_| ())
                }
                None => link(&p, port, max_s).map(|_| ()),
            }
        });
        if let Err(e) = r {
            eprintln!("online css: session: {e}");
            if mode != melee_modes::tu::slippi_online::MODE_DIRECT && e.contains("disconnected") {
                // Slippi: the opponent left (real Slippi Dolphin drops the link after every Unranked game); this side is
                // told and back on the idle select with the fighter kept. No search until the player presses Start.
                eprintln!("online css: rematch: the opponent left; back on the idle select, no search");
                let chosen = sc::with(|o| o.chosen);
                sc::go_idle();
                sc::with(|o| {
                    o.error = "Your opponent disconnected".into();
                    o.chosen = chosen;
                });
                return;
            }
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
/// `link`: the match's link kept for the select (`slippi_link::hand_back`).
pub fn rematch_env(match_args: &[String], winner_port: i32, link: &str) -> Option<String> {
    let set = std::env::var("MELEE_SLIPPI_SET").ok().filter(|v| !v.is_empty())?;
    let f: Vec<&str> = set.split(';').collect();
    let (mode, opp, game) = (f.first()?, f.get(1).copied().unwrap_or(""), f.get(2).and_then(|g| g.parse::<u32>().ok()).unwrap_or(0));
    let peer = arg(match_args, "--slippi-peer")?;
    let port = arg(match_args, "--slippi-port").unwrap_or_else(|| "0".into());
    let lp: u8 = arg(match_args, "--slippi-local-port").and_then(|v| v.parse().ok()).unwrap_or(0);
    let decider = match_args.iter().any(|a| a == "--slippi-host");
    let winner = if winner_port < 0 { sc::WINNER_NULL } else if winner_port as u8 == lp { sc::WINNER_WON } else { sc::WINNER_LOST };
    Some(format!("{peer};{port};{lp};{};{mode};{opp};{winner};{};{link}", decider as u8, game + 1))
}

/// In the match process: this match is a game of an online-select set and the set goes on after it (its link is
/// kept for the next game, not closed). Tests: `MELEE_SLIPPI_SET_GAMES` ends the set after that many games.
pub fn set_goes_on() -> bool {
    let Some(set) = std::env::var("MELEE_SLIPPI_SET").ok().filter(|v| !v.is_empty()) else { return false };
    let played = set.split(';').nth(2).and_then(|g| g.parse::<u32>().ok()).unwrap_or(0) + 1;
    !std::env::var("MELEE_SLIPPI_SET_GAMES").ok().and_then(|v| v.parse::<u32>().ok()).is_some_and(|n| played >= n)
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
    // the search's STUN socket on this port (Hero's): closed now, its NAT mapping stays for the link
    crate::slippi_np::release_port();
    // the port can still be held a moment by the process before (the last game's match window): retry the bind
    let mut udp = UdpTransport::open(p.bind_ip, port);
    let bind_t = Instant::now();
    while udp.is_err() && port != 0 && bind_t.elapsed() < Duration::from_secs(8) {
        std::thread::sleep(Duration::from_millis(100));
        udp = UdpTransport::open(p.bind_ip, port);
    }
    let udp = udp.map_err(|e| format!("bind: {e}"))?;
    let mut cl = Client::new(Box::new(udp), p.local_port, p.peer, p.remote_port);
    cl.set_fallback(p.fallback);
    cl.connect_timeout_ms = 30000;
    lock_in(cl, p, port, max_s)
}

/// The select on the link `cl` (new, or a set's kept from its last game): both lock in, then the link is kept for
/// the match process. Ok(true) when both locked in (the handoff is set).
fn lock_in(mut cl: Client, p: &crate::slippi_np::Paired, port: u16, max_s: u64) -> Result<bool, String> {
    sc::with(|o| o.local_index = p.local_port);
    let t0 = Instant::now();
    let (mut sent, mut theirs) = (None::<(u8, u8)>, None::<Selections>);
    let mut mine_sel = Selections::default();
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
            let z = sc::with(|o| o.disconnect);
            eprintln!("online css: session: {}, link closed", if z { "disconnected (hold Z)" } else { "cancelled" });
            if z {
                // Hero: after the cleanup the select stays up, idle; Start there searches again
                sc::go_idle();
            }
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
            // this side's rng offset goes with the lock-in (Slippi's selections carry it; the decider's is the match's):
            // the match goes on on this link, nothing is sent again
            let rng_offset = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1) ^ 0x9E37_79B9) % 0xFFFF;
            mine_sel = Selections {
                character_id: ck,
                character_color: color,
                is_character_selected: true,
                player_idx: p.local_port,
                stage_id: stage.unwrap_or(0),
                is_stage_selected: stage.is_some(),
                rng_offset,
                ..Default::default()
            };
            cl.set_match_selections(mine_sel);
            sent = Some((ck, color));
            // the set's state as it was at the lock-in (the menus can be re-entered before the handoff)
            lock_at = sc::with(|o| Some((o.mode, o.lock_stage, o.game)));
            sc::with(|o| o.local_ready = true);
        }
        if sent.is_some() && theirs.is_some() {
            LOCKED.store(true, std::sync::atomic::Ordering::SeqCst);
            break;
        }
        if t0.elapsed().as_secs() > max_s {
            return Err("timed out".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
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
    let theirs = theirs.unwrap();
    // Direct: the loser's stage, or none from the winner (the loser's stands); Unranked keeps the match's default
    let (mode, stage, game) = lock_at.unwrap();
    let opp = p.opp_code.clone();
    // the match's name tags (Slippi's online names over the fighters): matchmaking's names, else this login's name or
    // code and the opponent's code (`play_net::apply_name_tags`; never logged)
    let mm: Vec<String> = std::env::var("MELEE_SLIPPI_MM_NAMES").unwrap_or_default().split(char::from(9)).map(str::to_string).collect();
    let mut tags: [String; 4] = Default::default();
    for (i, t) in tags.iter_mut().enumerate() {
        *t = mm.get(i).cloned().unwrap_or_default();
    }
    let (un, uc) = sc::user();
    let lp = p.local_port as usize;
    let rp = p.remote_port as usize;
    if lp < 4 && tags[lp].trim().is_empty() {
        tags[lp] = if un.trim().is_empty() { uc } else { un };
    }
    if rp < 4 && tags[rp].trim().is_empty() {
        tags[rp] = opp.clone();
    }
    std::env::set_var("MELEE_SLIPPI_TAGS", tags.join("\t"));
    HANDOFF_GAME.store(game, std::sync::atomic::Ordering::Relaxed);
    // Slippi's match block: pausing only in Direct (Unranked / Ranked set disable_pausing)
    if mode == melee_modes::tu::slippi_online::MODE_DIRECT {
        m.extend(["--slippi-pause".into(), "1".into()]);
    }
    if mode == melee_modes::tu::slippi_online::MODE_DIRECT {
        match stage {
            Some(s) => m.extend(["--slippi-stage".into(), s.to_string()]),
            None => m.push("--slippi-stage-unset".into()),
        }
    }
    // the match process comes back to this select after its game (rematch_env / start_rematch)
    // (Direct only: Unranked's link ends with its game, as Slippi's does)
    // Unranked too (Slippi / Hero): after a game both are back on the select, still linked; the next game is played on
    // the same link once both lock in, and B on the select ends it for both
    std::env::set_var("MELEE_SLIPPI_SET", format!("{mode};{opp};{game}"));
    // Unranked (Slippi and Hero): the results, then back to the online select, idle; Start searches again
    // (the match process relaunches the menus with MELEE_SLIPPI_BACK, `slippi_after_back`)
    if mode == melee_modes::tu::slippi_online::MODE_DIRECT {
        std::env::remove_var("MELEE_SLIPPI_AFTER");
    } else {
        std::env::set_var("MELEE_SLIPPI_AFTER", mode.to_string());
    }
    eprintln!("online css: session: both locked in, the match next");
    // one link into the match (Slippi Dolphin takes no second connection): kept polled on this thread until the match
    // process takes it over (slippi_link.rs); the handoff is set once the keeper listens
    crate::slippi_link::keep(cl, mine_sel, theirs, "match process", |link| {
        m.extend(["--slippi-link".into(), link]);
        HANDOFF.with(|h| *h = Some(m));
    })?;
    Ok(true)
}
