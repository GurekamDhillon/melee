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
pub fn online_scene(sel: &[Selections; 2]) -> Result<(String, u16), String> {
    let stage = sel.iter().find(|s| s.is_stage_selected).map_or(0x1F, |s| s.stage_id);
    // the engine boots FD and Battlefield from these ids (scenario `stage=ext:N`); other stages are not wired
    if stage != 0x20 && stage != 0x1F {
        return Err(format!("stage {stage:#x} is not wired on the engine"));
    }
    let mut t = String::new();
    for (i, s) in sel.iter().enumerate() {
        t += &format!("p{}=ck:{}/human/c{};", i + 1, s.character_id, s.character_color);
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
    let (decider, local_port, remote_port, peer) = if let Some(url) = arg(args, "--mm") {
        let code = arg(args, "--code").unwrap_or_else(|| "RUST#001".into());
        let req = melee_net::slippi_mm::JoinRequest {
            uid: format!("local-test-{}", code.replace('#', "-")),
            display_name: arg(args, "--name").unwrap_or_else(|| "Rust".into()),
            connect_code: code,
            mode: 2,
            search_code: arg(args, "--search").unwrap_or_default(),
            app_version: arg(args, "--app-version").unwrap_or_else(|| "4.0.0-mainline-beta.20".into()),
            port: num("--port", 0) as u16,
        };
        if req.port == 0 {
            eprintln!("melee: --mm needs --port (the netplay port the server is told)");
            return ExitCode::from(2);
        }
        let shown = |t: &str| eprintln!("melee: mm: {}", t.chars().take(160).collect::<String>());
        let m = match melee_net::slippi_mm::join(&url, &req, Duration::from_secs(max_s), shown) {
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
        eprintln!("melee: matched ({}): local player {li}, remote {ri} at {ra}, decider {}, stages {:?}", m.match_id, m.is_host, m.stages);
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
    let udp = match UdpTransport::open(0x7F000001, num("--port", 0) as u16) {
        Ok(u) => u,
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
    let mut game = match EngineGame::open_scene(args, &scene, rng_offset) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    for (i, s) in by_port.iter().enumerate() {
        game.slippi_chars[i] = s.character_id as u32;
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
    let shift = num("--script-shift", 0) as i32;
    // --padgen SEED: the session's deterministic pad generator instead of the (1800-frame) scenario script, for long matches
    let padgen = arg(args, "--padgen").and_then(|v| v.parse::<u32>().ok());
    loop {
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        }
        next_tick += tick;
        ticks += 1;
        if t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: timeout at session frame {}", s.current_frame());
            break;
        }
        cl.poll();
        while let Some(n) = cl.next_note() {
            match n {
                Note::Pad { frame, pad, .. } => {
                    game.slippi_bits.entry(sess(frame)).or_default()[remote_port as usize & 3] = pad[8];
                    s.submit_remote_input(remote_port as usize * 2, sess(frame), &pad_input(&pad));
                }
                Note::Disconnected { reason } => {
                    eprintln!("melee: {tag}: peer disconnected (reason {reason})");
                    ended = true;
                }
                _ => {}
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
            eprintln!("melee: {tag}: the match ended at online frame {}", online(game.frame()));
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
            eprintln!("melee: {tag}: frame {} confirmed {} compared {compared} ping {} us offset {} us rollbacks {}", online(s.current_frame()), online(conf), cl.ping_us(), cl.time_offset_us(), s.stats.rollbacks);
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
    cl.disconnect(0);
    let last = finals.keys().next_back().copied().unwrap_or(0);
    eprintln!(
        "melee: {tag}: {} final checksums (online frames 1..{last}), {compared} compared with the peer: {soft} soft, {hard} HARD desyncs; rollbacks {} (max depth {}), resim {}, stalls {}, waits {}; ping {} us; pads sent {} received {}; enet resent {}",
        finals.len(), st.rollbacks, st.depth_max, st.resim, st.stall_ticks, st.waits, cl.ping_us(), cl.stats.pads_sent, cl.stats.pads_received,
        cl.host.peers.iter().map(|p| p.stats_resent).sum::<u64>()
    );
    if hard == 0 && soft == 0 && compared > 0 && (last >= frames || ended) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
