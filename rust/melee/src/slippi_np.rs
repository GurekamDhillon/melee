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

pub fn slippi_netplay(args: &[String]) -> ExitCode {
    let host = args.iter().any(|a| a == "--host");
    let num = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
    let Some(peer) = arg(args, "--peer").and_then(|p| addr_parse(&p, 51600)) else {
        eprintln!("usage: melee slippi-netplay (--host|--join) --port P --peer IP:PORT [--delay D] [--lag-ms N] [--loss-pct P] [--frames N] [--out CSV] [--max-seconds S]");
        return ExitCode::from(2);
    };
    let delay = num("--delay", 2) as i32;
    let frames = num("--frames", 3600) as i32;
    let max_s = num("--max-seconds", 600) as u64;
    let tag = if host { "decider" } else { "guest" };
    let local_port: u8 = if host { 0 } else { 1 };
    let remote_port: u8 = 1 - local_port;
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
    let t0 = Instant::now();
    // connect, then exchange MATCH_SELECTIONS (the decider's rng_offset is the match's)
    let mut rng_offset = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1) ^ 0x9E37_79B9) % 0xFFFF;
    let (mut sent_sel, mut remote_sel) = (false, None::<Selections>);
    while remote_sel.is_none() || !sent_sel {
        cl.poll();
        while let Some(n) = cl.next_note() {
            if let Note::Selections(s) = n {
                remote_sel = Some(s);
            }
        }
        if cl.status == Status::Connected && !sent_sel {
            sent_sel = true;
            // Falcon (external 0x00) on port 1, Fox (0x02) on port 2, Final Destination (0x20)
            let ch = if host { 0x00 } else { 0x02 };
            cl.set_match_selections(Selections { character_id: ch, is_character_selected: true, stage_id: 0x20, is_stage_selected: true, rng_offset, ..Default::default() });
            eprintln!("melee: {tag}: connected after {:.2}s", t0.elapsed().as_secs_f32());
        }
        if matches!(cl.status, Status::Failed | Status::Disconnected) || t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: connection failed ({:?})", cl.status);
            return ExitCode::FAILURE;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    if !host {
        rng_offset = remote_sel.unwrap().rng_offset;
    }
    eprintln!("melee: {tag}: selections exchanged, rng_offset {rng_offset:#06x}");
    melee_core::slippi_online::enable(rng_offset);
    let mut game = match EngineGame::open(args, "GrNLa.dat", &["falcon", "fox"], rng_offset, 4) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
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
                Note::Pad { frame, pad, .. } => s.submit_remote_input(remote_port as usize * 2, sess(frame), &pad_input(&pad)),
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
        for (&fr, &rv) in cl.remote_checksums.range(compared_upto + 1..) {
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
