//! `melee netplay`: one peer of a rollback match over UDP (the session half of gw_netplay.c, without
//! the lobby), driving the native VS match. The pads are the scenario script split by port (the host
//! plays port 1, the guest port 2), injected as the local controller of each machine; every other
//! input comes from the peer. The final per-frame neutral hashes of both peers must equal each
//! other and the C build's golden CSV.
//!
//! ```text
//! melee netplay --host --port 51500 [--delay 2] [--lag-ms N] [--loss-pct P] [--out F.csv] [--golden G.csv]
//! melee netplay --join 127.0.0.1:51500 [...]
//! ```
//! `--lag-ms` / `--loss-pct` degrade this peer's INBOUND datagrams (a deterministic LCG), so the
//! predictions miss and the session rolls back.

use crate::netmatch::{golden_hashes, NetGame};
use crate::np_env::{ck_name, lobby_bot, NativeEnv};
use melee_net::app::room::{Phase, Room, RoomEnv, RoomSettings};
use melee_net::app::scene::parse_scene;
use crate::netmatch::script_input;
use melee_net::net::{Config, Net};
use melee_net::rollback::{NetAdapter, NetLog};
use melee_net::session::{run_tick, InputSource, RbConfig, RbSession, FIRST};
use melee_net::transport::{addr_parse, Addr, Transport, UdpTransport};
use melee_net::NP_PAYLOAD;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// Inbound lag / loss on a real socket.
struct Degraded {
    inner: Box<dyn Transport>,
    lag: Duration,
    loss_pct: u32,
    rng: u32,
    q: VecDeque<(Instant, Addr, Vec<u8>)>,
}

impl Transport for Degraded {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        self.inner.send(to, data)
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        let mut tmp = [0u8; 2048];
        loop {
            let mut a = Addr::default();
            let n = self.inner.recv(&mut a, &mut tmp);
            if n <= 0 {
                break;
            }
            self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
            if self.loss_pct > 0 && (self.rng >> 16) % 100 < self.loss_pct {
                continue;
            }
            self.q.push_back((Instant::now() + self.lag, a, tmp[..n as usize].to_vec()));
        }
        if let Some(f) = self.q.front() {
            if f.0 <= Instant::now() {
                let (_, a, d) = self.q.pop_front().unwrap();
                *from = a;
                let n = d.len().min(buf.len());
                buf[..n].copy_from_slice(&d[..n]);
                return n as i32;
            }
        }
        0
    }
    fn local_port(&self) -> u16 {
        self.inner.local_port()
    }
}

/// `melee netplay`: one peer of a rollback match on the consolidated engine.
pub fn netplay(args: &[String]) -> std::process::ExitCode {
    netplay_with::<crate::netmatch::engine_game::EngineGame>(args)
}

fn netplay_with<G: NetGame + 'static>(args: &[String]) -> std::process::ExitCode {
    type Sess<G> = RbSession<G>;
    use std::process::ExitCode;
    let random = args.iter().any(|a| a == "--random");
    let room_code = arg(args, "--room");
    let host = args.iter().any(|a| a == "--host");
    let join = arg(args, "--join");
    if !random && (host == (join.is_some() || room_code.is_some())) {
        eprintln!("usage: melee netplay (--host --port P | --join IP:PORT) [--delay D] [--lag-ms N] [--loss-pct P] [--fps 60] [--out CSV] [--golden CSV] [--frames N] [--max-seconds S]");
        return ExitCode::from(2);
    }
    let num = |k: &str, d: i64| arg(args, k).and_then(|s| s.parse::<i64>().ok()).unwrap_or(d);
    let delay = num("--delay", 2) as i32;
    let fps = num("--fps", 60);
    let golden_frames = num("--frames", 1800) as i32; // last golden frame whose hash must be final
    let max_s = num("--max-seconds", 180) as u64;
    let local_port = if host { 0 } else { 1 };
    let tag = if host { "host" } else { "guest" };
    let golden = match arg(args, "--golden").map(|p| golden_hashes(&p)).transpose() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    };
    let use_lobby = args.iter().any(|a| a == "--lobby");
    let ck = num("--ck", if host { 0 } else { 2 }) as i32;
    let stocks = num("--stocks", 3) as i32;
    let minutes = num("--minutes", 0) as i32;
    let lag = Duration::from_millis(num("--lag-ms", 0) as u64);
    let loss = num("--loss-pct", 0) as u32;
    let rcfg = RbConfig { net: true, local_port, delay, src: InputSource::Live, ..RbConfig::default() };
    let sess: Rc<RefCell<Sess<G>>> = Rc::new(RefCell::new(Sess::<G>::new(rcfg)));
    sess.borrow_mut().scene_begin(2);
    let log = Rc::new(RefCell::new(NetLog::default()));
    let inner = Box::new(NetAdapter::new(sess.clone(), log.clone(), None));
    let server = arg(args, "--server");
    let bind_port = if host || random { num("--port", if random { 0 } else { 51500 }) as u16 } else { 0 };
    let udp = match UdpTransport::open(0x7F000001, bind_port) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("melee: bind: {e}");
            return ExitCode::FAILURE;
        }
    };
    let host_port = udp.local_port();
    let (rdv, base): (Option<melee_net::app::rdv::RdvRef>, Box<dyn Transport>) = match &server {
        Some(sv) => match melee_net::app::rdv::Rdv::new(sv, Box::new(udp)) {
            Ok(r) => {
                r.borrow_mut().lan_override = Some(0x7F000001);
                (Some(r.clone()), Box::new(melee_net::app::rdv::RdvTransport(r)))
            }
            Err(e) => {
                eprintln!("melee: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => (None, Box::new(udp)),
    };
    let tp: Box<dyn Transport> = Box::new(Degraded { inner: base, lag, loss_pct: loss, rng: 0x1234_5678 ^ local_port as u32, q: VecDeque::new() });
    let settings = RoomSettings {
        host,
        ck,
        stocks,
        minutes,
        delay,
        stage_ext: num("--stage", 32) as i32,
        use_lobby: use_lobby || random || room_code.is_some() || (host && server.is_some()),
        peer: join.as_deref().and_then(|j| addr_parse(j, 51500)),
        name: arg(args, "--name").unwrap_or_default(),
        exe_hash: u64::from_str_radix(arg(args, "--exe-hash").as_deref().unwrap_or("0"), 16).unwrap_or(0),
        seed: arg(args, "--seed").and_then(|s| s.parse().ok()),
        ..RoomSettings::default()
    };
    let mut room = match Room::begin(settings, Box::new(NativeEnv), inner, tp, rdv, room_code, random, host_port) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    let t0 = Instant::now();
    if host {
        eprintln!("melee: {tag} listening on 127.0.0.1:{host_port}");
    }
    let mut shown_code = String::new();
    // connect (and, with --lobby, play the pick/ban lobby with a bot) until the match is agreed
    loop {
        if t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: timeout before the match was agreed: {}", room.sh.borrow().status);
            return ExitCode::FAILURE;
        }
        match room.phase() {
            Phase::Working => {
                room.poll();
                let c = room.code();
                if !c.is_empty() && c != shown_code {
                    eprintln!("melee: {tag}: room code {c}");
                    shown_code = c;
                }
            }
            Phase::Lobby => {
                room.lobby_tick();
                let h = room.sh.borrow().s.host;
                lobby_bot(&mut room, if random { if h { 0 } else { 2 } } else { ck }, h);
            }
            Phase::Connected => break,
            _ => {
                eprintln!("melee: {tag}: {}", room.sh.borrow().status);
                return ExitCode::FAILURE;
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let (scene_str, agreed_seed, agreed_delay) = {
        let b = room.sh.borrow();
        (b.scene.clone(), b.seed, b.s.delay)
    };
    eprintln!("melee: {tag}: match agreed after {:.2}s - \"{scene_str}\" (seed {agreed_seed}, delay {agreed_delay})", t0.elapsed().as_secs_f32());
    let Some(ps) = parse_scene(&scene_str) else {
        eprintln!("melee: {tag}: the agreed scene does not parse");
        return ExitCode::FAILURE;
    };
    let names: Vec<&str> = [&ps.host_tok, &ps.guest_tok].iter().map(|t| ck_name(NativeEnv.ck_for_token(t)).unwrap_or("falcon")).collect();
    let mut game = match G::open(args, "GrNLa.dat", &names, agreed_seed, ps.stocks) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    game.set_live(args.iter().any(|a| a == "--live-rng").then_some(agreed_seed));
    let host = room.sh.borrow().s.host;
    let tag = if host { "host" } else { "guest" };
    let local_port = if host { 0 } else { 1 };
    sess.borrow_mut().set_local_port(local_port);
    sess.borrow_mut().set_delay(agreed_delay);
    room.release(); // gw_Netplay_Handshake: loaded, ready to start
    let mut net = room.net.take().expect("room has a connection");
    let _ = (&log, minutes);

    let last_final = FIRST + golden_frames + 1; // the checksum at the START of this frame = golden frame `golden_frames`
    let mut finals: Vec<(i32, u32)> = Vec::new();
    let t0 = Instant::now();
    let tick_len = if fps > 0 { Some(Duration::from_micros(1_000_000 / fps as u64)) } else { None };
    let mut next_tick = Instant::now();
    let mut adopted = false;
    let mut done_at: Option<Instant> = None;
    let mut ticks = 0u64;
    let mut stop_logged = false;
    let shift = num("--script-shift", 0) as i32;
    let trace = args.iter().any(|a| a == "--trace");
    loop {
        if let Some(l) = tick_len {
            let now = Instant::now();
            if next_tick > now {
                std::thread::sleep(next_tick - now);
            }
            next_tick += l;
        }
        ticks += 1;
        if t0.elapsed().as_secs() > max_s {
            eprintln!("melee: {tag}: timeout; state {:?} ({})", net.state(), net.last_reason());
            break;
        }
        NetAdapter::tick(&mut net, &sess);
        if matches!(net.state(), melee_net::net::State::Refused | melee_net::net::State::Dead) {
            eprintln!("melee: {tag}: session ended: {}", net.last_reason());
            break;
        }
        if !net.started() {
            continue;
        }
        let mut s = sess.borrow_mut();
        if !adopted {
            adopted = true;
            if !host {
                if let Some(rc) = net.remote_config() {
                    s.set_delay(rc.input_delay as i32);
                }
            }
            eprintln!("melee: {tag}: started after {:.2}s", t0.elapsed().as_secs_f32());
        }
        // The local controller: the script's sample for every frame up to the delayed one.
        let f = s.current_frame();
        let d = s.cfg.delay;
        for ff in f..=f + d {
            let i = script_input(local_port, ff - FIRST - shift);
            s.submit_local_input(local_port * 2, ff, &i);
        }
        let n_run = run_tick(&mut s, &mut game, None);
        if game.halted() != stop_logged {
            stop_logged = game.halted();
            if stop_logged {
                let fr = game.frame() + 1;
                for p in 0..2usize {
                    let u = s.input_any(p, fr).map(|i| (i.buttons, i.lx, i.ly, i.confirmed));
                    let w = script_input(p, fr - FIRST);
                    eprintln!("melee: {tag}:   frame {fr} port {p} used {u:?} script {:?}", (w.buttons, w.lx, w.ly));
                }
            }
            eprintln!("melee: {tag}: tick {ticks} sim {} at session frame {} (game frame {}, conf {}, remote_conf {})", if stop_logged { "STOPPED" } else { "resumed" }, s.current_frame(), game.frame() - FIRST, s.confirmed_frame(), net.remote_confirmed_frame());
        }
        if trace && ticks % 120 == 0 {
            eprintln!("melee: {tag}: tick {ticks} frame {} ran {n_run} confirmed {} adv {} remote_conf {} unacked {} rtt {} silent {} state {:?}",
                s.current_frame(), s.confirmed_frame(), s.frame_advantage(), net.remote_confirmed_frame(), net.unacked(), net.rtt_ms(), net.silent_ms(), net.state());
            eprintln!("melee: {tag}:   {} halted {:?} game frame {}", s.dbg_line(), game.halted(), game.frame() - FIRST);
        }
        let next = finals.last().map_or(FIRST, |x| x.0 + 1);
        for fr in next..=s.current_frame() {
            let h = s.checksum(fr);
            if h == 0 {
                break;
            }
            finals.push((fr, h));
        }
        drop(s);
        if finals.last().map_or(false, |x| x.0 >= last_final) {
            let t = done_at.get_or_insert_with(Instant::now);
            // keep polling a moment so the peer gets our inputs and checksums
            if t.elapsed() > Duration::from_millis(1500) {
                break;
            }
        }
    }

    let s = sess.borrow();
    let st = s.stats;
    let lg = log.borrow();
    let reached = finals.last().map_or(false, |x| x.0 >= last_final);
    // golden frame of "start of session frame f" = f - 1 - FIRST
    let (mut bad, mut n) = (0, 0);
    if let Some(gm) = &golden {
        for &(f, h) in &finals {
            if let Some(&w) = gm.get(&(f - 1 - FIRST)) {
                n += 1;
                if h != w && h != (w | 1) {
                    bad += 1;
                    if bad < 4 {
                        eprintln!("melee: {tag}: golden mismatch at golden frame {} ({h:08x} vs {w:08x})", f - 1 - FIRST);
                    }
                }
            }
        }
    }
    if let Some(p) = arg(args, "--out") {
        let mut t = String::from("frame,hash\n");
        for &(f, h) in &finals {
            t += &format!("{},{:08X}\n", f - 1 - FIRST, h);
        }
        let _ = std::fs::write(p, t);
    }
    if let Some(p) = arg(args, "--raw-out") {
        let mut t = String::from("frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11\n");
        for (f, h) in &game.raw_log() {
            t += &format!("{},{}\n", f, h);
        }
        let _ = std::fs::write(p, t);
    }
    eprintln!("melee: {tag}: {} final hashes (through golden frame {}), {} ticks; rollbacks {} (max depth {}), resim {}, stalls {}, waits {}, session desyncs {}, transport desyncs {}; golden {}/{} equal; net {:?}",
        finals.len(), finals.last().map_or(-99, |x| x.0 - 1 - FIRST), ticks, st.rollbacks, st.depth_max, st.resim, st.stall_ticks, st.waits, st.desyncs, lg.desyncs.len(), n - bad, n, net.stats());
    if reached && bad == 0 && st.desyncs == 0 && lg.desyncs.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
