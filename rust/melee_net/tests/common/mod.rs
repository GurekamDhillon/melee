//! The test peers and scenarios of gw_net_tests.c / oracle/oracle.c, over the Rust port.
//! `run_oracle_scenarios` produces the same transcript text the C oracle prints.
#![allow(dead_code)]

use melee_net::net::{Callbacks, Config, Event, Net, State};
use melee_net::sim::{SimEndpoint, SimNet, SimRef};
use melee_net::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const FIRST: i32 = -123;
pub const HOST_SLOTS: u8 = 0x13;
pub const GUEST_SLOTS: u8 = 0x44;

pub struct PState {
    pub sim: SimRef,
    pub id: usize,
    pub sender_id: usize,
    pub offset: u32,
    pub frame: i32,
    pub limit: i32,
    pub speed: f32,
    pub acc: f32,
    pub stall: i32,
    pub use_sync: bool,
    pub stalls_taken: i32,
    pub diverge_at: i32,
    pub report_checksums: bool,
    pub pull: bool,
    pub rslots: Vec<usize>,
    pub exp_frame: i32,
    pub delivered: u32,
    pub exp_idx: usize,
    pub bad: i32,
    pub bad_msg: String,
    pub desync_calls: i32,
    pub desync_frame: i32,
    pub lobby_rx: i32,
    pub pb: usize,
    pub events: Vec<(Event, String)>,
    /// print events / desyncs / lobby messages into the sim log (oracle transcript)
    pub trace: bool,
}

pub fn mk_payload(pb: usize, frame: i32, slot: usize, who: usize, out: &mut [u8]) {
    let mut h = (frame as u32).wrapping_mul(2654435761)
        ^ (slot as u32).wrapping_mul(40503)
        ^ (who as u32 + 1).wrapping_mul(0x9E3779B1);
    for (i, o) in out.iter_mut().enumerate().take(pb) {
        h ^= h >> 15;
        h = h.wrapping_mul(2246822519);
        h ^= h >> 13;
        *o = (h >> (i & 7)) as u8;
    }
}
pub fn chk_of(f: i32) -> u32 {
    (f as u32).wrapping_mul(2654435761) ^ 0x5BD1E995
}

fn log(sim: &SimRef, line: String) {
    if let Some(l) = sim.borrow_mut().log.as_mut() {
        l.push(line);
    }
}

pub struct PeerCb {
    pub st: Rc<RefCell<PState>>,
    pub pull: bool,
    pub lobby: bool,
    pub real_clock: bool,
}

impl Callbacks for PeerCb {
    fn remote_input(&mut self, frame: i32, slot: usize, payload: &[u8]) {
        let mut p = self.st.borrow_mut();
        if p.bad != 0 {
            return;
        }
        if frame != p.exp_frame || slot != p.rslots[p.exp_idx] {
            p.bad_msg = format!(
                "delivered frame {} slot {}, expected frame {} slot {}",
                frame, slot, p.exp_frame, p.rslots[p.exp_idx]
            );
            p.bad = 1;
            return;
        }
        let mut want = [0u8; MAX_PAYLOAD];
        mk_payload(p.pb, frame, slot, p.sender_id, &mut want);
        if payload[..p.pb] != want[..p.pb] {
            p.bad_msg = format!("frame {} slot {}: wrong payload contents", frame, slot);
            p.bad = 2;
            return;
        }
        p.exp_idx += 1;
        if p.exp_idx == p.rslots.len() {
            p.exp_idx = 0;
            p.exp_frame += 1;
            p.delivered += 1;
        }
    }
    fn has_local_input(&self) -> bool {
        self.pull
    }
    fn local_input(&mut self, frame: i32, slot: usize, out: &mut [u8]) -> bool {
        let p = self.st.borrow();
        if frame >= p.frame {
            return false;
        }
        mk_payload(p.pb, frame, slot, p.id, out);
        true
    }
    fn has_checksum(&self) -> bool {
        self.pull
    }
    fn checksum(&mut self, frame: i32) -> Option<u32> {
        let p = self.st.borrow();
        if frame >= p.frame {
            return None;
        }
        Some(chk_of(frame) ^ if frame >= p.diverge_at { 0xDEADBEEF } else { 0 })
    }
    fn event(&mut self, ev: Event, msg: Option<&str>) {
        let mut p = self.st.borrow_mut();
        let m = msg.unwrap_or("").to_string();
        if p.trace {
            let now = p.sim.borrow().now;
            log(&p.sim, format!("E {} {} {} {}", now, p.id, ev as i32, m));
        }
        p.events.push((ev, m));
    }
    fn desync(&mut self, frame: i32, local: u32, remote: u32) {
        let mut p = self.st.borrow_mut();
        p.desync_calls += 1;
        p.desync_frame = frame;
        if p.trace {
            let now = p.sim.borrow().now;
            log(&p.sim, format!("D {} {} {} {:08x} {:08x}", now, p.id, frame, local, remote));
        }
    }
    fn has_lobby(&self) -> bool {
        self.lobby
    }
    fn lobby_msg(&mut self, data: &[u8]) {
        let mut p = self.st.borrow_mut();
        if p.trace {
            let now = p.sim.borrow().now;
            let hex: String = data.iter().map(|b| format!("{:02x}", b)).collect();
            log(&p.sim, format!("L {} {} {}", now, p.id, hex));
        }
        p.lobby_rx += 1;
    }
    fn now_ms(&self) -> Option<u32> {
        if self.real_clock {
            return None;
        }
        let p = self.st.borrow();
        let now = p.sim.borrow().now;
        Some(now.wrapping_add(p.offset))
    }
}

pub struct Peer {
    pub st: Rc<RefCell<PState>>,
    pub net: Option<Net>,
}

impl Peer {
    pub fn new(sim: &SimRef, id: usize, offset: u32, pb: usize, trace: bool) -> Peer {
        let rmask = if id == 0 { GUEST_SLOTS } else { HOST_SLOTS };
        let st = PState {
            sim: sim.clone(),
            id,
            sender_id: if id == 0 { 1 } else { 0 },
            offset,
            frame: FIRST,
            limit: i32::MAX,
            speed: 1.0,
            acc: 0.0,
            stall: 0,
            use_sync: false,
            stalls_taken: 0,
            diverge_at: i32::MAX,
            report_checksums: true,
            pull: false,
            rslots: (0..MAX_SLOTS).filter(|i| rmask & (1 << i) != 0).collect(),
            exp_frame: FIRST,
            delivered: 0,
            exp_idx: 0,
            bad: 0,
            bad_msg: String::new(),
            desync_calls: 0,
            desync_frame: 0,
            lobby_rx: 0,
            pb,
            events: Vec::new(),
            trace,
        };
        Peer { st: Rc::new(RefCell::new(st)), net: None }
    }
    pub fn cb(&self, pull: bool, lobby: bool) -> Box<PeerCb> {
        Box::new(PeerCb { st: self.st.clone(), pull, lobby, real_clock: false })
    }
    pub fn net(&mut self) -> &mut Net {
        self.net.as_mut().unwrap()
    }
    pub fn n(&self) -> &Net {
        self.net.as_ref().unwrap()
    }
    pub fn has_event(&self, ev: Event) -> bool {
        self.st.borrow().events.iter().any(|e| e.0 == ev)
    }
    pub fn event_msg(&self, ev: Event) -> String {
        self.st.borrow().events.iter().find(|e| e.0 == ev).map(|e| e.1.clone()).unwrap_or_default()
    }
    pub fn delivered(&self) -> u32 {
        self.st.borrow().delivered
    }
}

pub fn blob() -> Vec<u8> {
    (0..300).map(|i| (i * 7 + 3) as u8).collect()
}

pub fn peer_cfg(pb: usize, exe: u64, iso: u64, mods: u64) -> Config {
    Config {
        exe_hash: exe,
        iso_hash: iso,
        mods_hash: mods,
        first_frame: FIRST,
        payload_bytes: pb as u8,
        seed: 0xC0FFEE11,
        input_delay: 2,
        host_slots: HOST_SLOTS,
        guest_slots: GUEST_SLOTS,
        match_blob: blob(),
        ..Config::default()
    }
}

pub fn make_pair_x(sim: &SimRef, guest_offset: u32, pull: bool, lobby: bool, pb: usize, trace: bool) -> (Peer, Peer) {
    let mut h = Peer::new(sim, 0, 0, pb, trace);
    let mut g = Peer::new(sim, 1, guest_offset, pb, trace);
    h.st.borrow_mut().pull = pull;
    g.st.borrow_mut().pull = pull;
    let mut hc = peer_cfg(pb, 0x1111, 0x2222, 0x3333);
    let mut gc = peer_cfg(pb, 0x1111, 0x2222, 0x3333);
    if lobby {
        hc.hold_start = true;
        gc.hold_start = true;
    }
    h.net = Net::host(&hc, h.cb(pull, lobby), SimEndpoint::new(sim, 0));
    g.net = Net::join(&gc, g.cb(pull, lobby), SimEndpoint::new(sim, 1), SimNet::addr(0));
    (h, g)
}
pub fn make_pair(sim: &SimRef, guest_offset: u32, pull: bool, pb: usize, trace: bool) -> (Peer, Peer) {
    make_pair_x(sim, guest_offset, pull, false, pb, trace)
}

pub fn session_tick(p: &mut Peer) {
    let frame = p.st.borrow().frame;
    p.net().poll(frame);
    if !p.n().started() {
        return;
    }
    if p.st.borrow().use_sync {
        let stall = p.st.borrow().stall;
        if stall > 0 {
            p.st.borrow_mut().stall -= 1;
            return;
        }
        let w = p.net().recommend_wait();
        if w > 0 {
            let mut s = p.st.borrow_mut();
            s.stall = w - 1;
            s.stalls_taken += w;
            return;
        }
    }
    let speed = p.st.borrow().speed;
    p.st.borrow_mut().acc += speed;
    loop {
        if p.st.borrow().acc < 1.0 {
            break;
        }
        p.st.borrow_mut().acc -= 1.0;
        let (frame, limit, pull, pb, id, rc, div) = {
            let s = p.st.borrow();
            (s.frame, s.limit, s.pull, s.pb, s.id, s.report_checksums, s.diverge_at)
        };
        if frame >= limit || p.n().unacked() >= 60 {
            break;
        }
        if !pull {
            let mut inp = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
            let ls = p.n().local_slots();
            for (slot, b) in inp.iter_mut().enumerate() {
                if ls & (1 << slot) != 0 {
                    mk_payload(pb, frame, slot, id, b);
                }
            }
            if p.net().submit_local(frame, &inp) != 0 {
                break;
            }
            if rc {
                p.net().report_checksum(frame, chk_of(frame) ^ if frame >= div { 0xDEADBEEF } else { 0 });
            }
        }
        p.st.borrow_mut().frame += 1;
    }
}

pub fn pair_tick(sim: &SimRef, a: &mut Peer, b: &mut Peer) {
    sim.borrow_mut().step();
    session_tick(a);
    session_tick(b);
}

fn out(sim: &SimRef, s: String) {
    log(sim, s);
}

fn summary(sim: &SimRef, name: &str, h: &Peer, g: &Peer) {
    let (a, b) = (h.n().stats(), g.n().stats());
    let (hs, gs) = (h.st.borrow(), g.st.borrow());
    let line = format!(
        "R {} del={}/{} st={}/{} bad={}/{} dsf={}/{} rtt={}/{} adv={:08x}/{:08x} fr={}/{} stl={}/{} conf={}/{} unack={}/{} \
         h={},{},{},{},{},{},{} g={},{},{},{},{},{},{} why={}|{}",
        name,
        hs.delivered,
        gs.delivered,
        h.n().state() as i32,
        g.n().state() as i32,
        hs.bad,
        gs.bad,
        h.n().desync_frame(),
        g.n().desync_frame(),
        h.n().rtt_ms(),
        g.n().rtt_ms(),
        h.n().frame_advantage().to_bits(),
        g.n().frame_advantage().to_bits(),
        hs.frame,
        gs.frame,
        hs.stalls_taken,
        gs.stalls_taken,
        h.n().remote_confirmed_frame(),
        g.n().remote_confirmed_frame(),
        h.n().unacked(),
        g.n().unacked(),
        a.packets_sent,
        a.packets_received,
        a.duplicates,
        a.out_of_order,
        a.bad_packets,
        a.inputs_delivered,
        a.resent_frames,
        b.packets_sent,
        b.packets_received,
        b.duplicates,
        b.out_of_order,
        b.bad_packets,
        b.inputs_delivered,
        b.resent_frames,
        h.n().last_reason(),
        g.n().last_reason()
    );
    drop(hs);
    drop(gs);
    out(sim, line);
}

fn new_sim(seed: u64) -> SimRef {
    let s = SimNet::shared(seed);
    s.borrow_mut().log = Some(Vec::new());
    s
}

fn take_log(sim: &SimRef, all: &mut Vec<String>) {
    all.append(sim.borrow_mut().log.as_mut().unwrap());
}

fn finish(sim: &SimRef, all: &mut Vec<String>, h: Peer, g: Peer) {
    drop(h);
    drop(g);
    take_log(sim, all);
}

fn sc_handshake(all: &mut Vec<String>) {
    all.push("S handshake".into());
    let s = new_sim(1);
    let (mut h, mut g) = make_pair(&s, 0xFFFFF000, false, 9, true);
    for _ in 0..600 {
        pair_tick(&s, &mut h, &mut g);
    }
    out(&s, format!("T {} {}", h.n().start_time_ms(), g.n().start_time_ms()));
    summary(&s, "handshake", &h, &g);
    finish(&s, all, h, g);
}

#[allow(clippy::too_many_arguments)]
fn sc_inputs(all: &mut Vec<String>, name: &str, loss: i32, jitter: u32, dup: i32, frames: i32, seed: u64, pb: usize, pull: bool) {
    all.push(format!("S {}", name));
    let s = new_sim(seed);
    let (mut h, mut g) = make_pair(&s, 0xFFFFF000, pull, pb, true);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = loss;
        sm.jitter_ms = jitter;
        sm.dup_pct = dup;
    }
    for p in [&h, &g] {
        let mut st = p.st.borrow_mut();
        st.limit = FIRST + frames;
        st.report_checksums = false;
    }
    for _ in 0..(frames as u32 + 4000) {
        pair_tick(&s, &mut h, &mut g);
        if h.delivered() >= frames as u32 && g.delivered() >= frames as u32 {
            break;
        }
    }
    summary(&s, name, &h, &g);
    finish(&s, all, h, g);
}

fn sc_sync(all: &mut Vec<String>, use_sync: bool) {
    let name = if use_sync { "sync1" } else { "sync0" };
    all.push(format!("S {}", name));
    let s = new_sim(5);
    let (mut h, mut g) = make_pair(&s, 0x1000, false, 9, true);
    {
        let mut sm = s.borrow_mut();
        sm.base_ms = 50;
        sm.jitter_ms = 10;
        sm.loss_pct = 5;
    }
    g.st.borrow_mut().speed = 0.85;
    for p in [&h, &g] {
        let mut st = p.st.borrow_mut();
        st.use_sync = use_sync;
        st.report_checksums = false;
    }
    for _ in 0..1500 {
        pair_tick(&s, &mut h, &mut g);
    }
    summary(&s, name, &h, &g);
    finish(&s, all, h, g);
}

fn sc_desync(all: &mut Vec<String>, pull: bool) {
    let name = if pull { "desync1" } else { "desync0" };
    all.push(format!("S {}", name));
    let s = new_sim(3);
    let (mut h, mut g) = make_pair(&s, 0, pull, 9, true);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = 20;
        sm.jitter_ms = 40;
    }
    g.st.borrow_mut().diverge_at = FIRST + 300;
    for _ in 0..700 {
        pair_tick(&s, &mut h, &mut g);
    }
    summary(&s, name, &h, &g);
    finish(&s, all, h, g);
}

pub fn refuse_cfgs(which: i32) -> (Config, Config) {
    let mut hc = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    hc.mods_desc = Some(if which == 9 {
        format!("{},bbbb", "a".repeat(250))
    } else {
        "plco#12ab,itco#34cd".to_string()
    });
    let mut gc = peer_cfg(
        9,
        if which == 0 { 0x9999 } else { 0x1111 },
        if which == 1 { 0x9999 } else { 0x2222 },
        if which == 2 || which == 9 { 0x9999 } else { 0x3333 },
    );
    if which == 3 {
        gc.payload_bytes = 12;
    }
    if which == 4 || which == 5 {
        hc.rules = 0x5FB;
        gc.rules = if which == 4 { 0 } else { 0x5FB ^ 1 };
        gc.rules_expect = true;
    }
    if which == 6 {
        hc.envoy = ENVOY_MODE_V1;
        gc.envoy = 0;
        gc.envoy_expect = true;
    }
    if which == 7 {
        hc.envoy = 0;
        gc.envoy = ENVOY_MODE_V1;
        gc.envoy_expect = true;
    }
    if which == 8 {
        hc.envoy = ENVOY_MODE_V1;
        gc.envoy = 0x1234;
    }
    if which == 10 {
        hc.rules = 0x77;
        gc.rules = 0x78;
    }
    (hc, gc)
}

fn sc_refuse(all: &mut Vec<String>, which: i32) {
    let name = format!("refuse{}", which);
    all.push(format!("S {}", name));
    let s = new_sim(7);
    let (hc, gc) = refuse_cfgs(which);
    let mut h = Peer::new(&s, 0, 0, 9, true);
    let mut g = Peer::new(&s, 1, 0, 9, true);
    h.net = Net::host(&hc, h.cb(false, false), SimEndpoint::new(&s, 0));
    g.net = Net::join(&gc, g.cb(false, false), SimEndpoint::new(&s, 1), SimNet::addr(0));
    let mut i = 0;
    while i < 100 && g.n().state() == State::Connecting {
        pair_tick(&s, &mut h, &mut g);
        i += 1;
    }
    for _ in 0..30 {
        pair_tick(&s, &mut h, &mut g);
    }
    let line = match g.n().remote_config() {
        Some(rc) => format!(
            "C 1 {:08x} {} {:02x} {:02x} {} {} {:08x} {:08x} {}",
            rc.seed,
            rc.input_delay,
            rc.host_slots,
            rc.guest_slots,
            rc.first_frame,
            rc.payload_bytes,
            rc.rules,
            rc.envoy,
            rc.match_blob.len()
        ),
        None => "C 0 00000000 0 00 00 0 0 00000000 00000000 0".to_string(),
    };
    out(&s, line);
    summary(&s, &name, &h, &g);
    finish(&s, all, h, g);
}

fn sc_old_heap(all: &mut Vec<String>, pass: i32) {
    let name = format!("oldheap{}", pass);
    all.push(format!("S {}", name));
    let s = new_sim(37 + pass as u64);
    s.borrow_mut().old_heap_peer = pass;
    let (mut h, mut g) = make_pair(&s, 0, false, 9, true);
    let mut i = 0;
    while i < 400 && g.n().state() != State::Refused {
        pair_tick(&s, &mut h, &mut g);
        i += 1;
    }
    summary(&s, &name, &h, &g);
    finish(&s, all, h, g);
}

fn sc_interrupt(all: &mut Vec<String>) {
    all.push("S interrupt".into());
    let s = new_sim(9);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, true);
    for t in 0..800 {
        if t == 300 {
            s.borrow_mut().blackhole = true;
        }
        if t == 390 {
            s.borrow_mut().blackhole = false;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    summary(&s, "interrupt", &h, &g);
    finish(&s, all, h, g);
}

fn sc_timeout(all: &mut Vec<String>) {
    all.push("S timeout".into());
    let s = new_sim(4);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, true);
    for t in 0..600 {
        if t == 200 {
            s.borrow_mut().blackhole = true;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    summary(&s, "timeout", &h, &g);
    finish(&s, all, h, g);
}

fn sc_quit(all: &mut Vec<String>) {
    all.push("S quit".into());
    let s = new_sim(6);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, true);
    for _ in 0..200 {
        pair_tick(&s, &mut h, &mut g);
    }
    g.net = None; // gw_net_free: QUIT x3
    for _ in 0..40 {
        s.borrow_mut().step();
        session_tick(&mut h);
    }
    out(&s, format!("Q {} {}", h.n().state() as i32, h.n().last_reason()));
    finish(&s, all, h, g);
}

fn sc_lobby(all: &mut Vec<String>) {
    all.push("S lobby".into());
    let s = new_sim(12);
    let (mut h, mut g) = make_pair_x(&s, 0x12345, false, true, 9, true);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = 25;
        sm.jitter_ms = 30;
    }
    for t in 0..700 {
        if t == 20 {
            for k in 0..5usize {
                let m: Vec<u8> = (0..10 + k * 7).map(|j| (k * 31 + j) as u8).collect();
                h.net().lobby_send(&m);
                g.net().lobby_send(&m[..3 + k]);
            }
        }
        if t == 150 {
            g.net().release();
        }
        if t == 260 {
            h.net().release();
        }
        if t == 400 {
            h.net().lobby_send(&[9, 8, 7]);
        }
        pair_tick(&s, &mut h, &mut g);
    }
    out(&s, format!("P {:x}", h.n().peer_mods_hash()));
    summary(&s, "lobby", &h, &g);
    finish(&s, all, h, g);
}

/// The oracle transcript, scenario for scenario as oracle.c's main().
pub fn run_oracle_scenarios() -> Vec<String> {
    let mut all = Vec::new();
    sc_handshake(&mut all);
    sc_inputs(&mut all, "clean", 0, 0, 0, 1500, 11, 9, false);
    sc_inputs(&mut all, "lossy", 20, 50, 10, 1500, 22, 28, false);
    sc_inputs(&mut all, "hostile", 50, 150, 10, 1500, 33, 9, false);
    sc_inputs(&mut all, "pull", 10, 30, 5, 1000, 44, 28, true);
    sc_sync(&mut all, true);
    sc_sync(&mut all, false);
    sc_desync(&mut all, false);
    sc_desync(&mut all, true);
    for w in 0..=10 {
        sc_refuse(&mut all, w);
    }
    sc_old_heap(&mut all, 1);
    sc_old_heap(&mut all, 2);
    sc_interrupt(&mut all);
    sc_timeout(&mut all);
    sc_quit(&mut all);
    sc_lobby(&mut all);
    all.push("END".into());
    all
}
