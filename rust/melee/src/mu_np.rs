//! P2P Direct (Hero's launcher lobby, his mu_net protocol): the game side. The match is agreed in the launcher's lobby
//! and arrives as `--p2p-*` options; this module parses them as his game does, loads the lobby's identity file, builds
//! the session descriptor, and connects (`melee_net::mu_net`). The match itself is a Slippi-online-rules 1v1 on the
//! engine, the same simulation as our Slippi-wire direct match; only the wire differs.
//! Spec and milestones: docs/rust-port/notes/p2p-direct-plan.md.
//!
//! ```text
//! melee mu-netplay --p2p-port P --p2p-peer A.B.C.D:P [--p2p-peer ..] --p2p-slot 0|1 --p2p-chars C/c:C/c
//!     --p2p-stage S --p2p-seed HEX --p2p-delay D [--p2p-identity F] [--p2p-expect HEX64] [--p2p-names A:B]
//!     [--p2p-connect-seconds N] [--p2p-games N] [--p2p-result F] [--frames N] [--out CSV] [--padgen SEED]
//! ```
//! (`mu-netplay`: headless, scripted pads, for the loopback tests; `play --p2p-*` is the windowed match.)

use crate::netmatch::engine_game::EngineGame;
use crate::netmatch::{script_input, NetGame};
use melee_core::slippi_online::{compare, Compare};
use melee_net::mu_net::conn::{Connector, DEFAULT_LIMIT};
use melee_net::mu_net::crypto::{hex, random, unhex, Bytes32, Identity};
use melee_net::mu_net::desc::{Descriptor, Slot, MAX_DELAY, MAX_GAMES, MAX_NAME, MIN_DELAY, STAGE_POOL};
use melee_net::mu_net::link::Link;
use melee_net::session::{run_tick, InputSource, RbConfig, RbGame, RbSession, FIRST};
use melee_net::slippi::{pad_bytes, pad_input};
use melee_net::transport::{Addr, Transport, UdpTransport};
use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::{Duration, Instant};

/// Exit code of a P2P run that played no game (his `session_done`: 2 = no match was played, 0 after one).
pub const EXIT_NO_GAME: u8 = 2;

/// Rules profile of his game library built with the Slippi layer (profile 1 is the one without it): the online
/// rule codes our match applies are the Slippi online codeset, so we name that profile. Engines are kept apart by
/// the build id.
pub const RULES_PROFILE: u32 = 2;

/// The `--p2p-*` options, checked as his game checks them.
#[derive(Clone, Debug, PartialEq)]
pub struct P2pOptions {
    pub port: u16,
    pub peers: Vec<Addr>,
    pub slot: usize,
    pub chars: [(u8, u8); 2],
    pub stage: u16,
    pub seed: u32,
    pub delay: u8,
    pub identity: String,
    pub expect: Option<Bytes32>,
    pub result: Option<String>,
    pub names: [String; 2],
    pub connect_seconds: Option<u32>,
    pub games: u32,
}

fn num(text: &str, low: i64, high: i64) -> Option<i64> {
    // base 10, or 0x-prefixed hex for the stage (his strtol base 0 also takes octal; no launcher sends that)
    let v = if let Some(h) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) { i64::from_str_radix(h, 16).ok()? } else { text.parse().ok()? };
    (low..=high).contains(&v).then_some(v)
}

fn fighter(t: &str) -> Option<(u8, u8)> {
    let (c, k) = t.split_once('/').unwrap_or((t, "0"));
    Some((num(c, 0, 255)? as u8, num(k, 0, 255)? as u8))
}

impl P2pOptions {
    /// None when no `--p2p-*` option is given; an error for a bad or half request.
    pub fn parse(args: &[String]) -> Result<Option<P2pOptions>, String> {
        let mut o = P2pOptions {
            port: 0,
            peers: Vec::new(),
            slot: 0,
            chars: [(2, 0), (2, 0)],
            stage: 0x1F,
            seed: 0,
            delay: 2,
            identity: "p2p_identity.json".into(),
            expect: None,
            result: None,
            names: ["P1".into(), "P2".into()],
            connect_seconds: None,
            games: 1,
        };
        let mut port = None;
        let mut any = false;
        let mut i = 0;
        while i < args.len() {
            let k = args[i].as_str();
            if !k.starts_with("--p2p-") {
                i += 1;
                continue;
            }
            any = true;
            let v = args.get(i + 1).map(String::as_str).ok_or_else(|| format!("{k} needs a value"))?;
            i += 2;
            let bad = |want: &str| format!("{k} {want}");
            match k {
                "--p2p-port" => port = Some(num(v, 0, 65535).ok_or_else(|| bad("<local UDP port, 0 to 65535>"))? as u16),
                "--p2p-peer" => {
                    let (ip, p) = v.rsplit_once(':').ok_or_else(|| bad("<dotted IPv4 address>:<port>"))?;
                    let ip: std::net::Ipv4Addr = ip.parse().map_err(|_| bad("<dotted IPv4 address>:<port>"))?;
                    let p = num(p, 1, 65535).ok_or_else(|| bad("<dotted IPv4 address>:<port>"))?;
                    if o.peers.len() >= melee_net::mu_net::conn::MAX_CANDIDATES {
                        return Err(bad("was given too many times"));
                    }
                    o.peers.push(Addr::new(u32::from(ip), p as u16));
                }
                "--p2p-slot" => o.slot = num(v, 0, 1).ok_or_else(|| bad("<0|1>"))? as usize,
                "--p2p-chars" => {
                    let e = || bad("<character id>[/<color>]:<character id>[/<color>]");
                    let (a, b) = v.split_once(':').ok_or_else(e)?;
                    o.chars = [fighter(a).ok_or_else(e)?, fighter(b).ok_or_else(e)?];
                }
                "--p2p-stage" => o.stage = num(v, 0, 0xFFFF).ok_or_else(|| bad("<stage id>"))? as u16,
                "--p2p-seed" => {
                    let h = v.trim_start_matches("0x");
                    if v.is_empty() || v.len() > 10 || h.is_empty() || h.len() > 8 {
                        return Err(bad("<hex, up to 8 digits>"));
                    }
                    o.seed = u32::from_str_radix(h, 16).map_err(|_| bad("<hex, up to 8 digits>"))?;
                }
                "--p2p-delay" => o.delay = num(v, MIN_DELAY as i64, MAX_DELAY as i64).ok_or_else(|| bad("<1 to 15 frames>"))? as u8,
                "--p2p-identity" if !v.is_empty() => o.identity = v.to_string(),
                "--p2p-expect" => {
                    let mut k = [0u8; 32];
                    if !unhex(v, &mut k) {
                        return Err(bad("<64 hex digits: the other player's identity key>"));
                    }
                    o.expect = Some(k);
                }
                "--p2p-result" if !v.is_empty() => o.result = Some(v.to_string()),
                "--p2p-names" => {
                    let e = || bad("<slot 0 name>:<slot 1 name> (at most 31 bytes each)");
                    let (a, b) = v.split_once(':').ok_or_else(e)?;
                    if a.is_empty() || b.is_empty() || a.len() > MAX_NAME || b.len() > MAX_NAME {
                        return Err(e());
                    }
                    o.names = [a.to_string(), b.to_string()];
                }
                "--p2p-connect-seconds" => o.connect_seconds = Some(num(v, 1, 600).ok_or_else(|| bad("<1 to 600>"))? as u32),
                "--p2p-games" => o.games = num(v, 0, MAX_GAMES as i64).ok_or_else(|| bad("<1 to 127 games, or 0 for no limit>"))? as u32,
                _ => return Err(format!("unknown option {k}")),
            }
        }
        if !any {
            return Ok(None);
        }
        match (port, o.peers.is_empty()) {
            (Some(p), false) => {
                o.port = p;
                Ok(Some(o))
            }
            _ => Err("a P2P match needs both --p2p-port and --p2p-peer".into()),
        }
    }
}

/// The identity at `path`: the lobby's JSON (`"ed_seed": "<64 hex>"`) or a raw 32-byte seed. A missing file is
/// created in the lobby's JSON form (written beside and renamed into place); an existing file is never rewritten.
pub fn load_identity(path: &str) -> Result<(Identity, bool), String> {
    let p = std::path::Path::new(path);
    if p.exists() {
        let d = std::fs::read(p).map_err(|e| format!("the identity file cannot be read: {e}"))?;
        if d.len() > 4 << 20 {
            return Err("the identity file is too large to be one".into());
        }
        let mut seed = [0u8; 32];
        if d.len() == 32 {
            seed.copy_from_slice(&d);
        } else if !seed_from_json(&String::from_utf8_lossy(&d), &mut seed) {
            return Err("the identity file holds no usable key".into());
        }
        let id = Identity::from_seed(&seed);
        seed.fill(0);
        return Ok((id, false));
    }
    let (mut seed, mut x) = ([0u8; 32], [0u8; 32]);
    if !random(&mut seed) || !random(&mut x) {
        return Err("no random numbers for a new identity".into());
    }
    if let Some(dir) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(dir);
    }
    let tmp = p.with_extension("json.new");
    let text = format!("{{\n  \"ed_seed\": \"{}\",\n  \"x_secret\": \"{}\"\n}}\n", hex(&seed), hex(&x));
    std::fs::write(&tmp, text).map_err(|e| format!("the identity file cannot be written: {e}"))?;
    std::fs::rename(&tmp, p).map_err(|e| format!("the identity file cannot be put in place: {e}"))?;
    let id = Identity::from_seed(&seed);
    seed.fill(0);
    x.fill(0);
    Ok((id, true))
}

/// `"ed_seed"` `:` `"<64 hex>"` anywhere in the text (the rest of the lobby's file is not ours to read).
fn seed_from_json(text: &str, seed: &mut [u8; 32]) -> bool {
    let Some(at) = text.find("\"ed_seed\"") else { return false };
    let rest = text[at + 9..].trim_start();
    let Some(rest) = rest.strip_prefix(':') else { return false };
    let Some(rest) = rest.trim_start().strip_prefix('"') else { return false };
    rest.len() > 64 && rest.as_bytes()[64] == b'"' && unhex(&rest[..64], seed)
}

/// Our build id: only peers running the same engine build pair (the descriptor digest covers it).
pub fn build_id() -> String {
    if let Ok(v) = std::env::var("MELEE_P2P_TEST_BUILD_ID") {
        return v; // test hook: pretend to be another build
    }
    format!("melee-rust {} {:015x}", melee_core::VERSION, crate::play_net::build_hash())
}

/// The descriptor both players build from the same options (his `build_descriptor`, in our words): the two slots,
/// stage, seed and delay; content `retail-1.02`, our rules profile and build id; a match id made from the options.
pub fn build_descriptor(o: &P2pOptions) -> Descriptor {
    let mut d = Descriptor { build_id: build_id(), rules_profile: RULES_PROFILE, stage: o.stage, rng_seed: o.seed, input_delay: o.delay, ..Default::default() };
    d.content_hash[..11].copy_from_slice(b"retail-1.02");
    for (i, p) in d.players.iter_mut().take(2).enumerate() {
        *p = Slot { present: true, character: o.chars[i].0, color: o.chars[i].1, name: o.names[i].clone(), ..Default::default() };
    }
    let s = o.seed.to_be_bytes();
    let st = o.stage.to_be_bytes();
    d.match_id = [b'c', b'l', b'i', 0, s[0], s[1], s[2], s[3], o.chars[0].0, o.chars[0].1, o.chars[1].0, o.chars[1].1, st[0], st[1], o.delay, 0];
    d
}

/// The match scenario of a descriptor (as `slippi_np::online_scene` makes one from Slippi's selections).
pub fn scene_of(d: &Descriptor) -> Result<String, String> {
    if !STAGE_POOL.contains(&d.stage) {
        return Err(format!("stage {:#x} is not one of the legal six", d.stage));
    }
    let p = &d.players;
    let mirror = p[0].character == p[1].character && p[0].color == p[1].color;
    let mut t = String::new();
    for (i, s) in p.iter().take(2).enumerate() {
        let shade = if mirror && i == 1 { "/shade1" } else { "" };
        t += &format!("p{}=ck:{}/human/c{}{shade};", i + 1, s.character, s.color);
    }
    // pause as in a Slippi Direct match (the descriptor's pause bit is his game's to set; Q in the plan)
    t += &format!("p3=none;p4=none;stage=ext:{};match=stock;stocks={};minutes={};items=off;pause=1", d.stage, d.stocks, d.timer_seconds / 60);
    Ok(t)
}

/// Binds the port (retrying a busy one for 2 s: the launcher's lobby closes it just before the game starts) and
/// connects until the session is ready or fails.
/// `args`' `--np-lag-ms / --np-jitter-ms / --np-loss-pct / --np-dup-pct / --np-reorder-pct` (tests) put a simulated
/// network under the socket (`play_net::Degraded`).
pub fn establish(o: &P2pOptions, tag: &str, args: &[String]) -> Result<Link, String> {
    let (identity, created) = load_identity(&o.identity)?;
    eprintln!("p2p: identity {} ({} {})", hex(&identity.public_key()), if created { "created" } else { "from" }, o.identity);
    let desc = build_descriptor(o);
    let mut tp = UdpTransport::open(0, o.port);
    for _ in 0..10 {
        if tp.is_ok() || o.port == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
        tp = UdpTransport::open(0, o.port);
    }
    let tp = tp.map_err(|e| format!("the UDP port {} could not be opened: {e}", o.port))?;
    let limit = o.connect_seconds.map_or(DEFAULT_LIMIT, |s| Duration::from_secs(s as u64));
    let peers: Vec<String> = o.peers.iter().map(|a| format!("{}:{}", std::net::Ipv4Addr::from(a.ip), a.port)).collect();
    eprintln!(
        "p2p: {tag}: slot {}, characters {}/{} and {}/{}, stage {}, seed {:08X}, delay {}, games {}, UDP port {}, peer {}",
        o.slot, o.chars[0].0, o.chars[0].1, o.chars[1].0, o.chars[1].1, o.stage, o.seed, o.delay, o.games, tp.local_port(), peers.join(", ")
    );
    let now = Instant::now();
    let n = crate::play_net::Netem::from_args(args);
    let tp: Box<dyn Transport> = if n.lag.is_zero() && n.jitter.is_zero() && n.loss == 0 && n.dup == 0 && n.reorder == 0 {
        Box::new(tp)
    } else {
        eprintln!("p2p: {tag}: simulated network {n:?}");
        Box::new(crate::play_net::Degraded::new(Box::new(tp), &n, 0x5eed ^ o.slot as u32))
    };
    let conn = Connector::start(tp, identity, desc, o.slot, &o.peers, o.expect, limit, now)?;
    let mut link = Link::new(conn, o.slot, o.games.min(u16::MAX as u32) as u16, now);
    loop {
        link.poll();
        if link.ready() {
            break;
        }
        if let Some(w) = link.down() {
            return Err(w);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    eprintln!(
        "p2p: {tag}: connected to {} ({}), descriptor {}",
        link.conn.peer_addr().map_or("?".into(), |a| format!("{}:{}", std::net::Ipv4Addr::from(a.ip), a.port)),
        if link.conn.decider() { "decider" } else { "follower" },
        hex(&link.conn.digest().unwrap_or_default()[..8])
    );
    Ok(link)
}

fn sess(f: i32) -> i32 {
    f + FIRST - 1
}
fn online(s: i32) -> i32 {
    s - FIRST + 1
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// `melee mu-netplay`: one headless peer of a P2P Direct match on loopback (scripted pads), for tests.
pub fn mu_netplay(args: &[String]) -> ExitCode {
    let o = match P2pOptions::parse(args) {
        Ok(Some(o)) => o,
        Ok(None) => {
            eprintln!("melee: mu-netplay needs --p2p-port and --p2p-peer (see mu_np.rs)");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    let n = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
    let frames = n("--frames", 3600) as i32;
    let tag = if o.slot == 0 { "slot0" } else { "slot1" };
    let t0 = Instant::now();
    let mut link = match establish(&o, tag, args) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("p2p: {tag}: failed: {e}");
            return ExitCode::from(EXIT_NO_GAME);
        }
    };
    eprintln!("p2p: {tag}: ready after {:.2}s", t0.elapsed().as_secs_f32());
    let d = link.conn.descriptor().clone();
    let scene = match scene_of(&d) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("p2p: {tag}: {e}");
            link.close();
            return ExitCode::from(EXIT_NO_GAME);
        }
    };
    melee_core::slippi_online::enable(d.rng_seed);
    if std::env::var_os("MELEE_SLIPPI_NO_ONLINE_CODES").is_none() {
        crate::slippi_np::online_codes(d.alt_stage);
    }
    // the scene load blocks this thread: keep the link serviced meanwhile (pings, the peer's early commits)
    let booted = {
        struct Pump(*mut Link);
        // SAFETY: the main thread does not touch `link` until the pump thread is joined (end of the scope)
        unsafe impl Send for Pump {}
        let pump = Pump(&mut link as *mut Link);
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
            let g = EngineGame::open_scene(args, &scene, d.rng_seed);
            stop.store(true, std::sync::atomic::Ordering::Release);
            g
        })
    };
    let mut game = match booted {
        Ok(g) => g,
        Err(e) => {
            eprintln!("p2p: {tag}: {e}");
            link.close();
            return ExitCode::from(EXIT_NO_GAME);
        }
    };
    if let Some(w) = link.down() {
        eprintln!("p2p: {tag}: the link went down during the boot: {w}");
        return ExitCode::from(EXIT_NO_GAME);
    }
    for (i, p) in d.players.iter().take(2).enumerate() {
        game.slippi_chars[i] = p.character as u32;
    }
    game.slippi_pre_tick();
    game.set_live(None);
    let local = o.slot;
    let remote = 1 - o.slot;
    let delay = d.input_delay as i32;
    let mut s: RbSession<EngineGame> = RbSession::new(RbConfig { net: true, local_port: local, delay, src: InputSource::Live, ..RbConfig::default() });
    s.scene_begin(2);
    s.set_local_port(local);
    s.set_delay(delay);
    let padgen = arg(args, "--padgen").and_then(|v| v.parse::<u32>().ok());
    let tick = Duration::from_micros(16683);
    let mut next_tick = Instant::now();
    let mut last_sent = FIRST - 1;
    let mut finals: BTreeMap<i32, u32> = BTreeMap::new();
    let (mut compared, mut compared_upto, mut soft, mut hard) = (0u32, 0i32, 0u32, 0u32);
    let mut skip_checked = 0;
    let mut ended = false;
    let mut failed = false;
    let mut done_at: Option<Instant> = None;
    let mut ticks = 0u64;
    let max_s = n("--max-seconds", 600) as u64;
    loop {
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        }
        next_tick += tick;
        ticks += 1;
        if t0.elapsed().as_secs() > max_s {
            eprintln!("p2p: {tag}: timeout at online frame {}", online(s.current_frame()));
            failed = true;
            break;
        }
        link.poll();
        for (f, pad) in link.input.take_fresh() {
            s.submit_remote_input(remote * 2, sess(f), &pad_input(&pad));
        }
        if let Some(w) = link.down() {
            eprintln!("p2p: {tag}: {w} at online frame {}", online(s.current_frame()));
            failed = !ended;
            break;
        }
        if link.input_timed_out() {
            eprintln!("p2p: {tag}: no input from the other player for 7 s at online frame {}: disconnected", online(s.current_frame()));
            failed = true;
            break;
        }
        // local pads, `delay` frames ahead; online frames 1..=delay carry the zero pad (the protocol's commit rule)
        let f = s.current_frame();
        for ff in f..=f + delay {
            let i = if online(ff) <= delay {
                pad_input(&[0; 8])
            } else {
                match padgen {
                    Some(g) => melee_net::session::rb_gen(g, local * 2, ff),
                    None => script_input(local, ff - FIRST),
                }
            };
            s.submit_local_input(local * 2, ff, &i);
        }
        for ff in (last_sent + 1)..=(f + delay) {
            let Some(i) = s.local_input_for_send(local * 2, ff) else { break };
            if online(ff) >= 1 {
                link.commit(online(ff), pad_bytes(&i));
            }
            last_sent = ff;
        }
        if let Some((&ckf, &ck)) = finals.iter().next_back() {
            link.input.set_checksum(ckf, ck);
        }
        link.send_commit();
        // the time-sync hold (plan 2.10): ahead by more than 10 ms, checked every 30 frames
        let of = online(s.current_frame());
        if of > 0 && of % 30 == 0 && of != skip_checked {
            skip_checked = of;
            let off = link.input.time_offset_us();
            if off > 10000 {
                s.request_wait((((off - 10000) as i64 / 16683) + 1).min(5) as i32);
            }
        }
        run_tick(&mut s, &mut game, None);
        let conf = s.confirmed_frame();
        link.input.set_floor(online(conf));
        link.advance_transcript();
        let from = finals.keys().next_back().map_or(2, |k| k + 1);
        for of in from..=online(conf) + 1 {
            match game.slippi_ck.get(&of) {
                Some(&v) if s.checksum(sess(of)) != 0 => {
                    finals.insert(of, v);
                }
                _ => break,
            }
        }
        if !ended && game.halted() {
            eprintln!("p2p: {tag}: the match ended at online frame {}", online(game.frame()));
            ended = true;
        }
        for (&fr, &rv) in link.remote_checksums.range((compared_upto + 1).max(2)..) {
            let Some(&lv) = finals.get(&fr) else { break };
            compared += 1;
            compared_upto = fr;
            match compare(lv, rv) {
                Compare::Same => {}
                Compare::Soft => soft += 1,
                Compare::Hard => {
                    hard += 1;
                    if hard < 4 {
                        eprintln!("p2p: {tag}: DESYNC at online frame {fr}: local {lv:08x} remote {rv:08x}");
                    }
                }
            }
        }
        if ticks % 600 == 0 {
            eprintln!(
                "p2p: {tag}: frame {} confirmed {} compared {compared} ping {} us offset {} us rollbacks {}",
                online(s.current_frame()),
                online(conf),
                link.input.ping_us().unwrap_or(0),
                link.input.time_offset_us(),
                s.stats.rollbacks
            );
        }
        if finals.keys().next_back().is_some_and(|&k| k >= frames) || ended {
            link.report_end();
            let t = done_at.get_or_insert_with(Instant::now);
            if (link.peer_agreement().is_some() && t.elapsed() > Duration::from_millis(300)) || t.elapsed() > Duration::from_millis(3000) {
                break;
            }
        }
    }
    link.report_end();
    let agree = link.peer_agreement();
    if let Some(p) = arg(args, "--out") {
        let mut t = String::from("frame,checksum\n");
        for (f, v) in &finals {
            t += &format!("{f},{v:08X}\n");
        }
        let _ = std::fs::write(p, t);
    }
    let tr = link.transcript.as_ref().map_or((0, String::new()), |t| (t.frames, hex(&t.digest()[..8])));
    let st = &link.input.stats;
    eprintln!(
        "p2p: {tag}: {} final checksums, {compared} compared: {soft} soft, {hard} HARD desyncs; transcript {} frames {} (peer {}); rollbacks {}, stalls {}, waits {}; ping {} us; commits sent {} received {}, gaps {}, conflicts {}",
        finals.len(),
        tr.0,
        tr.1,
        match agree {
            Some(true) => "agrees",
            Some(false) => "DISAGREES",
            None => "not compared",
        },
        s.stats.rollbacks,
        s.stats.stall_ticks,
        s.stats.waits,
        link.input.ping_us().unwrap_or(0),
        st.commits_sent,
        st.commits_received,
        st.gaps,
        st.conflicts
    );
    link.close();
    if !failed && hard == 0 && soft == 0 && compared > 0 && agree == Some(true) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_the_launchers_line() {
        let a = s(&[
            "--p2p-port", "51000", "--p2p-peer", "192.168.1.5:51001", "--p2p-peer", "1.2.3.4:51001", "--p2p-slot", "1", "--p2p-chars", "2/1:9/0",
            "--p2p-stage", "32", "--p2p-seed", "0A0B0C0D", "--p2p-delay", "3", "--p2p-identity", "C:\\L\\lobby-peer-identity.json", "--p2p-expect",
            &"ab".repeat(32), "--p2p-result", "C:\\L\\p2p-results\\x.json", "--p2p-names", "Alpha:Beta", "--p2p-connect-seconds", "45", "--p2p-games", "1",
        ]);
        let o = P2pOptions::parse(&a).unwrap().unwrap();
        assert_eq!((o.port, o.peers.len(), o.slot, o.chars, o.stage, o.seed, o.delay), (51000, 2, 1, [(2, 1), (9, 0)], 32, 0x0A0B_0C0D, 3));
        assert_eq!(o.peers[0], Addr::new(0xC0A8_0105, 51001));
        assert_eq!((o.connect_seconds, o.games, o.expect, o.names[1].as_str()), (Some(45), 1, Some([0xab; 32]), "Beta"));
        assert_eq!(P2pOptions::parse(&s(&["--iso", "x"])), Ok(None));
        assert!(P2pOptions::parse(&s(&["--p2p-port", "5"])).is_err(), "half a request");
        assert!(P2pOptions::parse(&s(&["--p2p-port", "5", "--p2p-peer", "1.2.3.4:5", "--p2p-delay", "16"])).is_err());
        assert!(P2pOptions::parse(&s(&["--p2p-port", "5", "--p2p-peer", "host:5"])).is_err());
        assert!(P2pOptions::parse(&s(&["--p2p-port", "5", "--p2p-peer", "1.2.3.4:5", "--p2p-expect", "abc"])).is_err());
        assert!(P2pOptions::parse(&s(&["--p2p-port", "5", "--p2p-peer", "1.2.3.4:5", "--p2p-bogus", "1"])).is_err());
    }

    #[test]
    fn both_players_build_the_same_descriptor() {
        let a = s(&["--p2p-port", "1", "--p2p-peer", "127.0.0.1:2", "--p2p-slot", "0", "--p2p-chars", "2/0:9/1", "--p2p-stage", "31", "--p2p-seed", "1234", "--p2p-names", "A:B"]);
        let mut b = a.clone();
        b[1] = "2".into();
        b[3] = "127.0.0.1:1".into();
        b[5] = "1".into();
        let (da, db) = (build_descriptor(&P2pOptions::parse(&a).unwrap().unwrap()), build_descriptor(&P2pOptions::parse(&b).unwrap().unwrap()));
        assert_eq!(da, db);
        assert_eq!(da.validate(), Ok(()));
        assert_eq!(&da.match_id[..8], &[b'c', b'l', b'i', 0, 0, 0, 0x12, 0x34]);
        assert_eq!(scene_of(&da).unwrap(), "p1=ck:2/human/c0;p2=ck:9/human/c1;p3=none;p4=none;stage=ext:31;match=stock;stocks=4;minutes=8;items=off;pause=1");
    }

    #[test]
    fn identity_file_forms() {
        let dir = std::env::temp_dir().join(format!("mu_np_id_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let p = dir.join("lobby-peer-identity.json");
        let ps = p.to_str().unwrap();
        let (a, created) = load_identity(ps).unwrap();
        assert!(created);
        let (b, created) = load_identity(ps).unwrap();
        assert!(!created);
        assert_eq!(a.public_key(), b.public_key());
        // the lobby's own layout (more keys, other spacing)
        let lobby = format!("{{\"profile\":{{}},\"ed_seed\" : \"{}\",\"friends\":[]}}", "42".repeat(32));
        std::fs::write(&p, lobby).unwrap();
        assert_eq!(load_identity(ps).unwrap().0.public_key(), Identity::from_seed(&[0x42; 32]).public_key());
        std::fs::write(&p, [7u8; 32]).unwrap();
        assert_eq!(load_identity(ps).unwrap().0.public_key(), Identity::from_seed(&[7; 32]).public_key());
        std::fs::write(&p, "{}").unwrap();
        assert!(load_identity(ps).is_err(), "an unusable file is an error, never replaced");
        assert_eq!(std::fs::read(&p).unwrap(), b"{}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
