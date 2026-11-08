//! `melee slippi-netplay`: one peer of a Slippi-protocol direct 1v1 (ENet + Slippi's online messages,
//! `melee_net::slippi`) on the native engine, with Slippi online mode on (InitOnlinePlay's seed and SyncRNG, the online
//! desync checksum). The pads are the scenario script split by port (the decider plays port 1, the other port 2).
//! Every finalized checksum we send is compared with the one the peer sends for the same frame, as Slippi's ASM does.
//!
//! ```text
//! melee slippi-netplay --host --port 51600 --peer 127.0.0.1:51601 [--delay 2] [--lag-ms N] [--loss-pct P] [--frames N] [--out F.csv]
//! melee slippi-netplay --join --port 51601 --peer 127.0.0.1:51600 [...]
//! ```
//! Direct and local only: no matchmaking, STUN or Slippi service is contacted; the peer address is given.

use crate::netmatch::engine_game::EngineGame;
use crate::netmatch::{script_input, NetGame};
use crate::netplay::Degraded;
use melee_core::slippi_online::{compare, Compare};
use melee_net::pad::RbInput;
use melee_net::session::{run_tick, InputSource, RbConfig, RbGame, RbSession, FIRST};
use melee_net::slippi::{pad_bytes, pad_input, Client, Note, Selections, Status, PAD_WIRE};
use melee_net::transport::{addr_parse, Transport, UdpTransport};
use std::collections::{BTreeMap, VecDeque};
use std::process::ExitCode;
use std::time::{Duration, Instant};

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// Session frame of online frame `f` (online frame 1 = session `FIRST`).
fn sess(f: i32) -> i32 {
    f + FIRST - 1
}
fn online(s: i32) -> i32 {
    s - FIRST + 1
}

/// The engine's side of Dolphin's `online_match_block` (`CEXISlippi::prepareOnlineMatchState`) for a 1v1: stock mode,
/// 4 stocks, the 8:00 count-down timer, items off, each port's character and color from its MATCH_SELECTIONS, the stage
/// of the first selection in port order that has one (Battlefield if none). The pause and score-UI bits are not
/// simulation. Returns the scenario string and the stage.
/// Slippi online's gameplay codes are on (`Slippi_Codes() == 2`, the online codeset) in every place that asks: the
/// offscreen-damage rule (`BrawlOffscreenDamage`: offscreen = outside the stage camera bounds, not the HUD magnifier,
/// which follows the live camera and so differed by a few frames from Slippi Dolphin: port/slippi4's 60 ms / 10 %
/// divergence), and the online code paths of ftLib and the FD background. A replay of an online game gets the same
/// value from its Game Start block (`install_replay`).
/// The online codeset's gameplay codes we have as `melee_core::codes` bits: NeutralSpawn, FreezeGlitchFix,
/// PreventWobbling, the ledge-grab limit (45), UCF 0.84 (setting the word turns off the default UCF otherwise) and, unless the match's `alt_stage_mode` asks for the moving stage,
/// frozen Pokemon Stadium (slippi-online-patches.md section 3).
pub fn online_codes(alt_stage_mode: u8) {
    use melee_core::codes as c;
    let frozen = if alt_stage_mode == 0 { c::FROZEN_PS } else { 0 };
    let w = std::env::var("MELEE_SLIPPI_ONLINE_BITS").ok().and_then(|v| u32::from_str_radix(&v, 16).ok());
    c::set_ps_camera_independent_monitor(true);
    c::set(w.unwrap_or(c::UCF | c::NEUTRAL_SPAWN | c::FREEZE_FIX | c::NO_WOBBLE | c::LGL | frozen), 45);
    unsafe {
        melee_fighter::platform::hook::Slippi_Codes = Some(|| 2);
        melee_fighter::hooks::SLIPPI_CODES = Some(|| 2);
        melee_world::tu::grlast::ext::SLIPPI_CODES = Some(|| 2);
    }
}

pub const LEGAL_STAGES: [u16; 6] = [0x02, 0x03, 0x08, 0x1C, 0x1F, 0x20];

pub fn online_scene(sel: &[Selections; 2]) -> Result<(String, u16), String> {
    let stage = sel.iter().find(|s| s.is_stage_selected).map_or(0x1F, |s| s.stage_id);
    // internal stage ids (`stage=ext:N`): the legal set of Slippi's online modes (Fountain of Dreams, Pokemon Stadium,
    // Yoshi's Story, Dreamland, Battlefield, Final Destination); others are not tested on the engine
    if !LEGAL_STAGES.contains(&stage) {
        return Err(format!("stage {stage:#x} is not wired on the engine"));
    }
    let mut t = String::new();
    // a mirror match in the same costume: the second port gets shade 1 (PlayerInitData.sub_color), as Slippi Dolphin's
    // match block has it (read back from the Game Start block of its replay)
    let mirror = sel[0].character_id == sel[1].character_id && sel[0].character_color == sel[1].character_color;
    for (i, s) in sel.iter().enumerate() {
        let shade = if mirror && i == 1 { "/shade1" } else { "" };
        t += &format!("p{}=ck:{}/human/c{}{shade};", i + 1, s.character_id, s.character_color);
    }
    t += &format!("p3=none;p4=none;stage=ext:{stage};match=stock;stocks=4;minutes=8;items=off");
    Ok((t, stage))
}

pub fn slippi_netplay(args: &[String]) -> ExitCode {
    let num = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
    let delay = num("--delay", 2) as i32;
    let frames = num("--frames", 3600) as i32;
    let max_s = num("--max-seconds", 600) as u64;
    let t0 = Instant::now();
    // --mm ws://127.0.0.1:P/v1/queue: be paired through a (local mock) matchmaking server, as Slippi Dolphin is;
    // otherwise direct: --host|--join with --peer
    // --mm-real: the only way to reach a matchmaking server that is not on loopback (Slippi's own, wss://). The account
    // comes from --user-json (a COPY of a Slippi user.json in a throwaway dir); none of its fields is ever printed.
    let real = args.iter().any(|a| a == "--mm-real");
    // --mm-lan: the matchmaking mock (and so the peer) is on another machine of a private network (a WSL guest and its
    // Windows host, two LAN boxes): ws:// to a private literal IPv4, sockets bound to every interface
    let lan = !real && args.iter().any(|a| a == "--mm-lan");
    let bind_ip: u32 = if real || lan { 0 } else { 0x7F000001 };
    let (decider, local_port, remote_port, peer) = if let Some(url) = arg(args, "--mm") {
        let mut code = arg(args, "--code").unwrap_or_else(|| "RUST#001".into());
        let (mut uid, mut play_key, mut display) = (format!("local-test-{}", code.replace('#', "-")), String::new(), arg(args, "--name").unwrap_or_else(|| "Rust".into()));
        let mut app_version = arg(args, "--app-version").unwrap_or_else(|| "4.0.0-mainline-beta.20".into());
        if real {
            let Some(path) = arg(args, "--user-json") else {
                eprintln!("melee: --mm-real needs --user-json (a copy of the account's Slippi user.json)");
                return ExitCode::from(2);
            };
            let acct = std::fs::read_to_string(&path).ok().and_then(|t| melee_net::slippi_mm::Json::parse(&t));
            let field = |k: &str| acct.as_ref().and_then(|j| j.get(k)).and_then(|v| v.str()).map(str::to_string);
            match (field("uid"), field("playKey"), field("connectCode")) {
                (Some(u), Some(k), Some(c)) => {
                    (uid, play_key, code) = (u, k, c);
                    display = field("displayName").unwrap_or(display);
                    // the server refuses an outdated client: the account's own latestVersion unless --app-version
                    if arg(args, "--app-version").is_none() {
                        app_version = field("latestVersion").unwrap_or(app_version);
                    }
                }
                _ => {
                    eprintln!("melee: --user-json: not a Slippi user.json (uid, playKey, connectCode)");
                    return ExitCode::from(2);
                }
            }
        }
        let mode = match arg(args, "--mm-mode").as_deref() {
            None | Some("direct") => 2,
            Some("unranked") if real => 1,
            Some(m) => {
                eprintln!("melee: --mm-mode {m}: only direct (or unranked with --mm-real, for a join-and-leave check); never ranked");
                return ExitCode::from(2);
            }
        };
        let req = melee_net::slippi_mm::JoinRequest {
            uid,
            play_key,
            lan_ip: arg(args, "--lan-ip").unwrap_or_default(),
            display_name: display,
            connect_code: code,
            mode,
            search_code: arg(args, "--search").unwrap_or_default(),
            app_version,
            port: num("--port", 0) as u16,
        };
        if req.port == 0 {
            eprintln!("melee: --mm needs --port (the netplay port the server is told)");
            return ExitCode::from(2);
        }
        // a real server's messages carry account data (uids, codes, addresses): only their type is shown
        let shown = |t: &str| {
            if real {
                let ty = melee_net::slippi_mm::Json::parse(t).and_then(|j| j.get("type").and_then(|v| v.str()).map(str::to_string));
                eprintln!("melee: mm: {} message", ty.as_deref().unwrap_or("?"));
            } else {
                eprintln!("melee: mm: {}", t.chars().take(160).collect::<String>());
            }
        };
        // --mm-leave-after-joined: close as soon as the server has accepted the join (the unranked check: never stays
        // queued long enough to be matched on purpose)
        let leave_early = args.iter().any(|a| a == "--mm-leave-after-joined");
        let leave = |t: &str| leave_early && melee_net::slippi_mm::Json::parse(t).is_some_and(|j| j.get("type").and_then(|v| v.str()) .is_some_and(|t| t == "joined" || t == "create-ticket-resp"));
        let target = if real {
            melee_net::slippi_mm::Target::Real
        } else if lan {
            melee_net::slippi_mm::Target::Lan
        } else {
            melee_net::slippi_mm::Target::Loopback
        };
        // enet://HOST[:PORT]: the ENet matchmaking service production Slippi Dolphin 3.x speaks (mm.slippi.gg:43113);
        // ws:// / wss://: the WebSocket service of the mainline client
        let res = if let Some(server) = url.strip_prefix("enet://") {
            melee_net::slippi_mm::join_enet(server.trim_end_matches('/'), bind_ip, &req, Duration::from_secs(max_s), target, shown, leave)
        } else {
            melee_net::slippi_mm::join_to(&url, &req, Duration::from_secs(max_s), target, shown, leave)
        };
        let m = match res {
            Err(e) if e == "left" => {
                eprintln!("melee: matchmaking: accepted (joined), left the queue at once");
                return ExitCode::SUCCESS;
            }
            Ok(m) => m,
            Err(e) => {
                eprintln!("melee: matchmaking: {e}");
                return ExitCode::FAILURE;
            }
        };
        let (Some(li), Some((ri, ra))) = (m.local_index(), m.remote_addresses().into_iter().next()) else {
            eprintln!("melee: matchmaking: the match has no local or remote player");
            return ExitCode::FAILURE;
        };
        let Some(peer) = addr_parse(&ra, 51441) else {
            eprintln!("melee: matchmaking: bad remote address {ra}");
            return ExitCode::FAILURE;
        };
        if real {
            eprintln!("melee: matched: local player {li}, remote {ri}, decider {}, stages {:?}", m.is_host, m.stages);
        } else {
            eprintln!("melee: matched ({}): local player {li}, remote {ri} at {ra}, decider {}, stages {:?}", m.match_id, m.is_host, m.stages);
        }
        (m.is_host, li, ri, peer)
    } else {
        let host = args.iter().any(|a| a == "--host");
        let Some(peer) = arg(args, "--peer").and_then(|p| addr_parse(&p, 51600)) else {
            eprintln!("usage: melee slippi-netplay (--host|--join) --port P --peer IP:PORT [--delay D] [--lag-ms N] [--loss-pct P] [--frames N] [--out CSV] [--max-seconds S]");
            eprintln!("       melee slippi-netplay --mm ws://127.0.0.1:P/v1/queue --port P [--code C] [--search C] [--char EXT] [--color N] [--stage EXT] [...]");
            return ExitCode::from(2);
        };
        let lp: u8 = if host { 0 } else { 1 };
        (host, lp, 1 - lp, peer)
    };
    let tag = if decider { "decider" } else { "guest" };
    let udp = match UdpTransport::open(bind_ip, num("--port", 0) as u16) {
        Ok(mut u) => {
            // arrival stamps for the displayed round trip (what each frame's poll sees is unchanged)
            if let Err(e) = u.stamp_arrivals() {
                eprintln!("melee: no arrival stamps ({e}): the ping shown includes frame-poll waits");
            }
            u
        }
        Err(e) => {
            eprintln!("melee: bind: {e}");
            return ExitCode::FAILURE;
        }
    };
    let tp: Box<dyn Transport> = Box::new(Degraded {
        inner: Box::new(udp),
        lag: Duration::from_millis(num("--lag-ms", 0) as u64),
        loss_pct: num("--loss-pct", 0) as u32,
        rng: 0x5150_0000 ^ local_port as u32,
        q: VecDeque::new(),
    });
    let mut cl = Client::new(tp, local_port, peer, remote_port);
    cl.connect_timeout_ms = 30000;
    // connect, then exchange MATCH_SELECTIONS (the decider's rng_offset is the match's)
    let rng_offset = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1) ^ 0x9E37_79B9) % 0xFFFF;
    // ours: --char (external id; default Falcon on port 1, Fox on port 2), --color, --stage (external id; FD)
    let mine = Selections {
        character_id: num("--char", if local_port == 0 { 0x00 } else { 0x02 }) as u8,
        character_color: num("--color", 0) as u8,
        is_character_selected: true,
        player_idx: local_port,
        stage_id: num("--stage", 0x20) as u16,
        is_stage_selected: true,
        rng_offset,
        ..Default::default()
    };
    let (mut sent_sel, mut remote_sel) = (false, None::<Selections>);
    while remote_sel.is_none() || !sent_sel {
        cl.poll();
        while let Some(n) = cl.next_note() {
            if let Note::Selections(s) = n {
                eprintln!("melee: {tag}: remote selections {s:?}");
                if s.is_character_selected {
                    remote_sel = Some(s);
                }
            }
        }
        if cl.status == Status::Connected && !sent_sel {
            sent_sel = true;
            cl.set_match_selections(mine);
            eprintln!("melee: {tag}: connected after {:.2}s", t0.elapsed().as_secs_f32());
        }
        if matches!(cl.status, Status::Failed | Status::Disconnected) || t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: connection failed ({:?})", cl.status);
            return ExitCode::FAILURE;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let theirs = remote_sel.unwrap();
    let mut by_port = [mine; 2];
    by_port[remote_port as usize & 1] = theirs;
    let rng_offset = if decider { mine.rng_offset } else { theirs.rng_offset };
    let (scene, stage) = match online_scene(&by_port) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("melee: {tag}: selections exchanged, rng_offset {rng_offset:#06x}, stage {stage:#x}, scene {scene}");
    melee_core::slippi_online::enable(rng_offset);
    if std::env::var_os("MELEE_SLIPPI_NO_ONLINE_CODES").is_none() {
        online_codes(by_port.iter().map(|s| s.alt_stage_mode).max().unwrap_or(0));
    }
    // the scene load blocks this thread: keep servicing the ENet host meanwhile (Slippi's netplay thread does), or a
    // boot longer than the peer's ENet timeout (30 s) loses the link, and the peer's early pads and pings go unacked
    let boot_t = Instant::now();
    let booted = {
        struct Pump(*mut Client);
        // SAFETY: the main thread does not touch `cl` until the pump thread is joined (end of the scope)
        unsafe impl Send for Pump {}
        let pump = Pump(&mut cl as *mut Client);
        let stop = std::sync::atomic::AtomicBool::new(false);
        std::thread::scope(|sc| {
            let stop = &stop;
            sc.spawn(move || {
                let pump = pump;
                while !stop.load(std::sync::atomic::Ordering::Acquire) {
                    unsafe { (*pump.0).poll() };
                    std::thread::sleep(Duration::from_millis(2));
                }
            });
            // test hook: MELEE_NP_BOOT_DELAY_S=N makes this side's boot N seconds slower (a slow machine's scene load)
            if let Some(d) = std::env::var("MELEE_NP_BOOT_DELAY_S").ok().and_then(|v| v.parse::<u64>().ok()) {
                eprintln!("melee: {tag}: test hook: boot delayed {d} s");
                std::thread::sleep(Duration::from_secs(d));
            }
            let g = EngineGame::open_scene(args, &scene, rng_offset);
            stop.store(true, std::sync::atomic::Ordering::Release);
            g
        })
    };
    let mut game = match booted {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!("melee: {tag}: booted in {:.2}s (link {:?})", boot_t.elapsed().as_secs_f32(), cl.status);
    if matches!(cl.status, Status::Failed | Status::Disconnected) {
        eprintln!("melee: {tag}: the peer link went down during the boot ({:?})", cl.status);
        return ExitCode::FAILURE;
    }
    for (i, s) in by_port.iter().enumerate() {
        game.slippi_chars[i] = s.character_id as u32;
    }
    if arg(args, "--trace").is_some() {
        game.slippi_trace = Some(Default::default());
    }
    game.slippi_pre_tick();
    game.set_live(None);
    let mut s: RbSession<EngineGame> = RbSession::new(RbConfig { net: true, local_port: local_port as usize, delay, src: InputSource::Live, ..RbConfig::default() });
    s.scene_begin(2);
    s.set_local_port(local_port as usize);
    s.set_delay(delay);
    cl.start_game();

    let mut rnd: u32 = rng_offset.wrapping_mul(2654435761) ^ (local_port as u32 + 1);
    let mut last_sent = FIRST - 1;
    let mut finals: BTreeMap<i32, u32> = BTreeMap::new();
    let (mut compared, mut soft, mut hard) = (0u32, 0u32, 0u32);
    let mut compared_upto = 0;
    let tick = Duration::from_micros(16683);
    let mut next_tick = Instant::now();
    let mut done_at: Option<Instant> = None;
    let mut ticks = 0u64;
    let mut ended = false;
    let mut skip_checked = 0;
    let mut forced_resims = 0u32;
    let shift = num("--script-shift", 0) as i32;
    // --padgen SEED: the session's deterministic pad generator instead of the (1800-frame) scenario script, for long matches
    let padgen = arg(args, "--padgen").and_then(|v| v.parse::<u32>().ok());
    // no remote pad for this long ends the run with a report instead of a silent stall: generous until the first pad
    // (the peer may still be loading), short after it
    let first_pad_s = num("--first-pad-seconds", 120) as u64;
    let silence_s = num("--silence-seconds", 10) as u64;
    let mut last_rx: Option<Instant> = None;
    let loop_t = Instant::now();
    let mut failed = false;
    let (mut late_sum, mut late_max) = (0u64, 0u64);
    let (mut work_t, mut work_sum, mut work_max) = (None::<Instant>, 0u64, 0u64);
    eprintln!("melee: {tag}: frame loop: frame {} link {:?} ping {} us (pad acks {} us); waiting for the peer's pads", online(s.current_frame()), cl.status, cl.net_ping_us(), cl.ping_us());
    loop {
        if let Some(w) = work_t {
            let us = w.elapsed().as_micros() as u64;
            work_sum += us;
            work_max = work_max.max(us);
        }
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        }
        // how late this tick woke (sleep overshoot or a slow previous tick)
        let late = Instant::now().saturating_duration_since(next_tick).as_micros() as u64;
        late_sum += late;
        late_max = late_max.max(late);
        next_tick += tick;
        ticks += 1;
        work_t = Some(Instant::now());
        if t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: timeout at session frame {}", s.current_frame());
            break;
        }
        cl.poll();
        while let Some(n) = cl.next_note() {
            match n {
                Note::Pad { frame, pad, .. } => {
                    if last_rx.is_none() {
                        eprintln!("melee: {tag}: first remote pad (online frame {frame}) after {:.2}s in the frame loop", loop_t.elapsed().as_secs_f32());
                    }
                    last_rx = Some(Instant::now());
                    let sf = sess(frame);
                    game.slippi_bits.entry(sf).or_default()[remote_port as usize & 3] = pad[8];
                    s.submit_remote_input(remote_port as usize * 2, sf, &pad_input(&pad));
                    // a frame that ran its seed-bit mix with this port's byte still unknown (a predicted pad counts 0)
                    if game.slippi_mixed.get(&sf).is_some_and(|b| b[remote_port as usize & 3] != pad[8]) {
                        forced_resims += 1;
                        s.force_resim_from(remote_port as usize * 2, sf);
                    }
                }
                Note::Disconnected { reason } => {
                    if reason == melee_net::slippi::REASON_PEER_RESTARTED {
                        eprintln!("melee: {tag}: the peer address started a new match (its process restarted): this match is over");
                        failed = true;
                    } else {
                        eprintln!("melee: {tag}: peer disconnected (reason {reason})");
                    }
                    ended = true;
                }
                _ => {}
            }
        }
        if !ended && matches!(cl.status, Status::Failed | Status::Disconnected) {
            eprintln!("melee: {tag}: link {:?} at online frame {}", cl.status, online(s.current_frame()));
            ended = true;
            failed = true;
        }
        if !ended {
            let (since, lim) = match last_rx {
                Some(t) => (t.elapsed(), silence_s),
                None => (loop_t.elapsed(), first_pad_s),
            };
            if since > Duration::from_secs(lim) && finals.keys().next_back().map_or(true, |&k| k < frames) {
                eprintln!(
                    "melee: {tag}: no pads from the peer for {} s ({}; link {:?}, ping {} us (pad acks {} us), pads received {}) at online frame {}: giving up",
                    since.as_secs(),
                    if last_rx.is_none() { "none ever" } else { "since the last" },
                    cl.status,
                    cl.net_ping_us(),
                    cl.ping_us(),
                    cl.stats.pads_received,
                    online(s.current_frame())
                );
                ended = true;
                failed = true;
            }
        }
        if ended {
            break;
        }
        // local controller: the script, known `delay` frames ahead
        let f = s.current_frame();
        for ff in f..=f + delay {
            let i = match padgen {
                Some(g) => melee_net::session::rb_gen(g, local_port as usize * 2, ff),
                None => script_input(local_port as usize, ff - FIRST - shift),
            };
            s.submit_local_input(local_port as usize * 2, ff, &i);
        }
        let (ckf, ck) = finals.iter().next_back().map_or((0, 0), |(a, b)| (*a, *b));
        for ff in (last_sent + 1)..=(f + delay) {
            let Some(i) = s.local_input_for_send(local_port as usize * 2, ff) else { break };
            let mut p = [0u8; PAD_WIRE];
            p[..8].copy_from_slice(&pad_bytes(&i));
            rnd ^= rnd << 13;
            rnd ^= rnd >> 17;
            rnd ^= rnd << 5;
            p[8] = rnd as u8;
            game.slippi_bits.entry(ff).or_default()[local_port as usize & 3] = p[8];
            cl.send_pad(online(ff), p, ckf, ck);
            last_sent = ff;
        }
        // Slippi's skip rule: well ahead of the peer, wait (only on frames that are multiples of 30)
        let of = online(s.current_frame());
        if of > 0 && of % 30 == 0 && of != skip_checked {
            skip_checked = of;
            let off = cl.time_offset_us();
            if off > 10000 {
                s.request_wait((((off - 10000) as i64 / 16683) + 1).min(5) as i32);
            }
        }
        run_tick(&mut s, &mut game, None);
        // newly finalized local checksums: the session's own finality rule (the state entering the frame is final once
        // the frame before it is confirmed and no rollback below it is pending), after this tick's resimulation
        let conf = s.confirmed_frame();
        let from = finals.keys().next_back().map_or(2, |k| k + 1); // online frame 1 is the state before any frame: never logged
        for of in from..=online(conf) + 1 {
            match game.slippi_ck.get(&of) {
                Some(&v) if s.checksum(sess(of)) != 0 => {
                    finals.insert(of, v);
                }
                _ => break,
            }
        }
        if game.halted() {
            eprintln!(
                "melee: {tag}: the match ended at online frame {} (stocks left: p1 {}, p2 {})",
                online(game.frame()),
                melee_fighter::stale::stocks(0),
                melee_fighter::stale::stocks(1)
            );
            ended = true;
        }
        // the ASM's compare: every frame both sides finalized
        // from online frame 2: frame 1 (the state before any frame) is never in `finals`, and Slippi Dolphin does send it
        for (&fr, &rv) in cl.remote_checksums.range((compared_upto + 1).max(2)..) {
            let Some(&lv) = finals.get(&fr) else { break };
            compared += 1;
            compared_upto = fr;
            match compare(lv, rv) {
                Compare::Same => {}
                Compare::Soft => soft += 1,
                Compare::Hard => {
                    hard += 1;
                    if hard < 4 {
                        eprintln!("melee: {tag}: DESYNC at online frame {fr}: local {lv:08x} remote {rv:08x}");
                    }
                }
            }
        }
        if ticks % 600 == 0 {
            eprintln!("melee: {tag}: frame {} confirmed {} compared {compared} ping {} us (pad acks {} us) offset {} us rollbacks {}", online(s.current_frame()), online(conf), cl.net_ping_us(), cl.ping_us(), cl.time_offset_us(), s.stats.rollbacks);
        }
        if finals.keys().next_back().map_or(false, |&k| k >= frames) || ended {
            let t = done_at.get_or_insert_with(Instant::now);
            if t.elapsed() > Duration::from_millis(1500) || ended {
                break;
            }
        }
    }
    let st = s.stats;
    if let Some(p) = arg(args, "--out") {
        let mut t = String::from("frame,checksum\n");
        for (f, v) in &finals {
            t += &format!("{f},{v:08X}\n");
        }
        let _ = std::fs::write(p, t);
    }
    if let (Some(p), Some(tr)) = (arg(args, "--trace"), game.slippi_trace.as_ref()) {
        // finalized frames only: pads applied in online frame F's tick, then the state after it (= Slippi's frame F checksum)
        let mut t = String::new();
        for (f, l) in tr.range(..=finals.keys().next_back().copied().unwrap_or(0)) {
            t += &format!("{f} | {l}
");
        }
        let _ = std::fs::write(p, t);
    }
    {
        // the seed-bit mix (TriggerSendInput section 11) on finalized frames: frames where a port rolled a random special
        let lastf = finals.keys().next_back().map_or(0, |&k| sess(k));
        let rolled: Vec<_> = game.slippi_mixed.range(..=lastf).collect();
        let forced = rolled.len();
        eprintln!("melee: {tag}: seed-bit rolls on {forced} finalized frames (first {:?}), forced resims {}", rolled.first().map(|(f, _)| online(**f)), forced_resims);
    }
    cl.disconnect(0);
    eprintln!("melee: {tag}: tick wake lateness mean {} us, max {} us over {ticks} ticks", late_sum / ticks.max(1) as u64, late_max);
    eprintln!("melee: {tag}: tick work (poll + sim + send) mean {} us, max {} us", work_sum / ticks.max(1), work_max);
    let last = finals.keys().next_back().copied().unwrap_or(0);
    eprintln!(
        "melee: {tag}: {} final checksums (online frames 1..{last}), {compared} compared with the peer: {soft} soft, {hard} HARD desyncs; rollbacks {} (max depth {}), resim {}, stalls {}, waits {}; ping {} us (pad acks {} us); pads sent {} received {}; enet resent {}",
        finals.len(), st.rollbacks, st.depth_max, st.resim, st.stall_ticks, st.waits, cl.net_ping_us(), cl.ping_us(), cl.stats.pads_sent, cl.stats.pads_received,
        cl.host.peers.iter().map(|p| p.stats_resent).sum::<u64>()
    );
    if !failed && hard == 0 && soft == 0 && compared > 0 && (last >= frames || ended) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// `melee slippi-replay --trace T.txt --log engine.log [--lag N] [--drop-seed S --drop-pct P] [--out CSV]`: a
/// `slippi-netplay --trace` run replayed offline and deterministically. The scene and rng_offset come from the run's
/// log, both ports' pads and seed bytes from the trace. Without `--lag` the frames run straight (no rollback: the sim on
/// the run's final inputs); with `--lag N` the other port's pads play the remote side through the rollback session:
/// each arrives N ticks late (plus a deterministic extra delay on `--drop-pct` of them), predicted until then, so the
/// session rolls back the way a lagged peer makes it. Writes the online checksums like the live run's `--out`.
/// `--slp FILE` (straight run only): the match as a Slippi replay (`slp_export`), the engine's own .slp of the game.
pub fn slippi_replay(args: &[String]) -> ExitCode {
    let num = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
    let (Some(tp), Some(lp)) = (arg(args, "--trace"), arg(args, "--log")) else {
        eprintln!("usage: melee slippi-replay --trace T --log L [--lag N] [--drop-seed S --drop-pct P] [--out CSV] [--slp FILE]");
        return ExitCode::from(2);
    };
    let log = std::fs::read_to_string(&lp).unwrap_or_default();
    let Some(line) = log.lines().find(|l| l.contains("selections exchanged, rng_offset ")) else {
        eprintln!("melee: no 'selections exchanged' line in {lp}");
        return ExitCode::from(2);
    };
    let decider = line.contains(" decider:");
    let rng_offset = u32::from_str_radix(line.split("rng_offset 0x").nth(1).unwrap_or("").split(',').next().unwrap_or(""), 16).unwrap_or(0);
    let scene = line.split("scene ").nth(1).unwrap_or("").trim().to_string();
    let chars: Vec<u32> = scene.split(';').take(2).filter_map(|t| t.split(':').nth(1)?.split('/').next()?.parse().ok()).collect();
    // trace lines: "F | btn sx sy cx cy l r bits | <port 2> | ck ..."
    let mut pads: BTreeMap<i32, [(RbInput, u8); 2]> = BTreeMap::new();
    for l in std::fs::read_to_string(&tp).unwrap_or_default().lines() {
        let f: Vec<&str> = l.split(" | ").collect();
        if f.len() < 3 {
            continue;
        }
        let Ok(fr) = f[0].parse::<i32>() else { continue };
        let pad = |t: &str| {
            let v: Vec<&str> = t.split_whitespace().collect();
            let n = |i: usize| v.get(i).and_then(|x| x.parse::<i32>().ok()).unwrap_or(0);
            let b = u32::from_str_radix(v.first().copied().unwrap_or("0"), 16).unwrap_or(0);
            let i = RbInput { present: 1, is_raw: 1, confirmed: 1, buttons: b, raw: [n(1) as i8, n(2) as i8, n(3) as i8, n(4) as i8], pad_l: n(5) as u8, pad_r: n(6) as u8, ..Default::default() };
            (i, u8::from_str_radix(v.get(7).copied().unwrap_or("0"), 16).unwrap_or(0))
        };
        pads.insert(fr, [pad(f[1]), pad(f[2])]);
    }
    let last = pads.keys().next_back().copied().unwrap_or(0);
    eprintln!("melee: slippi-replay: {} frames, rng_offset {rng_offset:#06x}, scene {scene}", pads.len());
    melee_core::slippi_online::enable(rng_offset);
    if std::env::var_os("MELEE_SLIPPI_NO_ONLINE_CODES").is_none() {
        online_codes(log.split("alt_stage_mode: ").nth(1).and_then(|t| t.split(|ch: char| !ch.is_ascii_digit()).next()?.parse().ok()).unwrap_or(0));
    }
    // `--slp FILE`: the straight run (no `--lag`) written as a Slippi replay, for slpcheck against Dolphin's
    let slp = arg(args, "--slp").filter(|_| arg(args, "--lag").is_none());
    if let Some(p) = &slp {
        crate::slp_export::install(p, "network", ["P1".into(), "P2".into(), "P3".into(), "P4".into()]);
    }
    let mut game = match EngineGame::open_scene(args, &scene, rng_offset) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    for (i, c) in chars.iter().enumerate() {
        game.slippi_chars[i] = *c;
    }
    if arg(args, "--trace-out").is_some() {
        game.slippi_trace = Some(Default::default());
    }
    let lag = arg(args, "--lag").and_then(|v| v.parse::<i32>().ok());
    let local = if decider { 0usize } else { 1 };
    let remote = 1 - local;
    for (&f, p) in &pads {
        // the remote byte is only known once its pad arrives (below); the straight run knows everything
        let rb = if lag.is_some() { 0 } else { p[remote].1 };
        let mut b = [0u8; 4];
        b[local] = p[local].1;
        b[remote] = rb;
        game.slippi_bits.insert(sess(f), b);
    }
    game.slippi_pre_tick();
    game.set_live(None);
    match lag {
        None => {
            for f in 1..=last {
                let mut inp = [None; melee_net::session::SLOTS];
                if let Some(p) = pads.get(&f) {
                    inp[0] = Some(p[0].0);
                    inp[2] = Some(p[1].0);
                }
                RbGame::run_frame(&mut game, &inp, false);
                if game.halted() {
                    break;
                }
            }
        }
        Some(lag) => {
            let delay = num("--delay", 2) as i32;
            let (dseed, dpct) = (num("--drop-seed", 1) as u32, num("--drop-pct", 0) as u32);
            // the arrival tick of each remote frame: lag ticks after it is due, more when "dropped" (resent later)
            let arrive = |f: i32| {
                let h = melee_net::session::rb_hash(dseed, f as u32, 77);
                f + lag + if h % 100 < dpct { 1 + (h >> 8) as i32 % 6 } else { 0 }
            };
            let mut by_tick: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
            for &f in pads.keys() {
                by_tick.entry(arrive(f)).or_default().push(f);
            }
            let mut s: RbSession<EngineGame> = RbSession::new(RbConfig { net: true, local_port: local, delay, src: InputSource::Live, ..RbConfig::default() });
            s.scene_begin(2);
            s.set_local_port(local);
            s.set_delay(delay);
            let mut tick = 0;
            while online(s.current_frame()) <= last && !game.halted() && tick < last * 4 {
                tick += 1;
                let f = s.current_frame();
                for ff in f..=f + delay {
                    if let Some(p) = pads.get(&online(ff)) {
                        s.submit_local_input(local * 2, ff, &p[local].0);
                    }
                }
                let now = tick; // arrivals by wall tick: a stalled session still receives
                let rest = by_tick.split_off(&(now + 1));
                for (_, fs) in std::mem::replace(&mut by_tick, rest) {
                    for rf in fs {
                        let (p, bits) = pads[&rf][remote];
                        game.slippi_bits.entry(sess(rf)).or_default()[remote] = bits;
                        s.submit_remote_input(remote * 2, sess(rf), &p);
                        if game.slippi_mixed.get(&sess(rf)).is_some_and(|b| b[remote] != bits) {
                            s.force_resim_from(remote * 2, sess(rf));
                        }
                    }
                }
                run_tick(&mut s, &mut game, None);
            }
            eprintln!("melee: slippi-replay: lag {lag} drop {dpct}%: rollbacks {} (max depth {}), resim {}", s.stats.rollbacks, s.stats.depth_max, s.stats.resim);
        }
    }
    if slp.is_some() {
        crate::slp_export::finish();
    }
    if let (Some(p), Some(tr)) = (arg(args, "--trace-out"), game.slippi_trace.as_ref()) {
        let t: String = tr.iter().map(|(f, l)| format!("{f} | {l}\n")).collect();
        let _ = std::fs::write(p, t);
    }
    if let Some(p) = arg(args, "--out") {
        let mut t = String::from("frame,checksum\n");
        for (f, v) in game.slippi_ck.range(2..=last + 1) {
            t += &format!("{f},{v:08X}\n");
        }
        let _ = std::fs::write(p, t);
    }
    ExitCode::SUCCESS
}
