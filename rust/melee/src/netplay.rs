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

use crate::netmatch::{golden_hashes, open_match, NetMatch};
use crate::vs_match::rollback::script_input;
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
    inner: UdpTransport,
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

type Sess = RbSession<NetMatch>;

pub fn netplay(args: &[String]) -> std::process::ExitCode {
    use std::process::ExitCode;
    let host = args.iter().any(|a| a == "--host");
    let join = arg(args, "--join");
    if host == join.is_some() {
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
    let mut game = match open_match(args) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::FAILURE;
        }
    };
    game.live = true;

    let rcfg = RbConfig { net: true, local_port, delay, src: InputSource::Live, ..RbConfig::default() };
    let sess: Rc<RefCell<Sess>> = Rc::new(RefCell::new(Sess::new(rcfg)));
    sess.borrow_mut().scene_begin(2);
    let log = Rc::new(RefCell::new(NetLog::default()));
    let cb = Box::new(NetAdapter::new(sess.clone(), log.clone(), None));
    let ncfg = Config {
        first_frame: FIRST,
        payload_bytes: NP_PAYLOAD as u8,
        input_delay: delay as u8,
        host_slots: 0x01,
        guest_slots: 0x04,
        seed: 12345,
        frame_us: 16667,
        ..Config::default()
    };
    let lag = Duration::from_millis(num("--lag-ms", 0) as u64);
    let mk = |bind_port: u16| -> std::io::Result<Box<dyn Transport>> {
        let inner = UdpTransport::open(0x7F000001, bind_port)?;
        Ok(Box::new(Degraded { inner, lag, loss_pct: num("--loss-pct", 0) as u32, rng: 0x1234_5678 ^ local_port as u32, q: VecDeque::new() }))
    };
    let net = if host {
        let p = num("--port", 51500) as u16;
        match mk(p) {
            Ok(tp) => {
                eprintln!("melee: {tag} listening on 127.0.0.1:{p}");
                Net::host(&ncfg, cb, tp)
            }
            Err(e) => {
                eprintln!("melee: bind: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        let Some(addr) = addr_parse(join.as_deref().unwrap(), 51500) else {
            eprintln!("melee: bad --join address");
            return ExitCode::from(2);
        };
        match mk(0) {
            Ok(tp) => Net::join(&ncfg, cb, tp, addr),
            Err(e) => {
                eprintln!("melee: bind: {e}");
                return ExitCode::FAILURE;
            }
        }
    };
    let Some(mut net) = net else {
        eprintln!("melee: could not open the session");
        return ExitCode::FAILURE;
    };

    let last_final = FIRST + golden_frames + 1; // the checksum at the START of this frame = golden frame `golden_frames`
    let mut finals: Vec<(i32, u32)> = Vec::new();
    let t0 = Instant::now();
    let tick_len = if fps > 0 { Some(Duration::from_micros(1_000_000 / fps as u64)) } else { None };
    let mut next_tick = Instant::now();
    let mut adopted = false;
    let mut done_at: Option<Instant> = None;
    let mut ticks = 0u64;
    let mut stop_logged = false;
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
            let i = script_input(local_port, ff - FIRST);
            s.submit_local_input(local_port * 2, ff, &i);
        }
        let n_run = run_tick(&mut s, &mut game, None);
        if game.vs.stopped.is_some() != stop_logged {
            stop_logged = game.vs.stopped.is_some();
            if stop_logged {
                let fr = game.vs.frame + FIRST + 1;
                for p in 0..2usize {
                    let u = s.input_any(p, fr).map(|i| (i.buttons, i.lx, i.ly, i.confirmed));
                    let w = script_input(p, fr - FIRST);
                    eprintln!("melee: {tag}:   frame {fr} port {p} used {u:?} script {:?}", (w.buttons, w.lx, w.ly));
                }
            }
            eprintln!("melee: {tag}: tick {ticks} sim {} at session frame {} (game frame {}, conf {}, remote_conf {}): {:?}", if stop_logged { "STOPPED" } else { "resumed" }, s.current_frame(), game.vs.frame, s.confirmed_frame(), net.remote_confirmed_frame(), game.vs.stopped);
        }
        if trace && ticks % 120 == 0 {
            eprintln!("melee: {tag}: tick {ticks} frame {} ran {n_run} confirmed {} adv {} remote_conf {} unacked {} rtt {} silent {} state {:?}",
                s.current_frame(), s.confirmed_frame(), s.frame_advantage(), net.remote_confirmed_frame(), net.unacked(), net.rtt_ms(), net.silent_ms(), net.state());
            eprintln!("melee: {tag}:   {} stopped {:?} game frame {}", s.dbg_line(), game.vs.stopped.is_some(), game.vs.frame);
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
    eprintln!("melee: {tag}: {} final hashes (through golden frame {}), {} ticks; rollbacks {} (max depth {}), resim {}, stalls {}, waits {}, session desyncs {}, transport desyncs {}; golden {}/{} equal; net {:?}",
        finals.len(), finals.last().map_or(-99, |x| x.0 - 1 - FIRST), ticks, st.rollbacks, st.depth_max, st.resim, st.stall_ticks, st.waits, st.desyncs, lg.desyncs.len(), n - bad, n, net.stats());
    if reached && bad == 0 && st.desyncs == 0 && lg.desyncs.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
