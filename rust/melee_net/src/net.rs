//! The session object: a line-by-line port of gw_net.c's `struct gw_net` and its functions.
//!
//! Frames on the wire are INDEXES from `first_frame` (u32); the API speaks the session's signed
//! frame numbers. All timing is u32 milliseconds compared wrap-safely (`reached`). Floats (the
//! frame-advantage estimate) follow the shipping build: native shims are compiled by clang for
//! i686 (SSE2, no FMA), so `a + (b - a) * k` is a separate f32 multiply and add, as written here.

use crate::transport::{Addr, Transport};
use crate::wire::*;
use crate::*;

const HELLO_INTERVAL_MS: u32 = 250;
const READY_INTERVAL_MS: u32 = 150;
const START_INTERVAL_MS: u32 = 100;
const HANDSHAKE_TIMEOUT_MS: u32 = 10000;
const MIN_SEND_GAP_MS: u32 = 4;
const MAX_CHECKSUMS: usize = 8;
const ADV_EMA: f32 = 0.125;
const MAX_PULL_PER_POLL: usize = 64;
const LOBBY_QUEUE: usize = 32;

/// `gw_net_event`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Event {
    Accepted = 1,
    Starting = 2,
    Started = 3,
    Refused = 4,
    Interrupted = 5,
    Resumed = 6,
    Disconnected = 7,
}

/// `gw_net_state` (the ordering matters: the silence check uses `>= Accepted && < Dead`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum State {
    Idle = 0,
    Connecting,
    Listening,
    Accepted,
    Starting,
    Running,
    Refused,
    Dead,
}

/// The session side (`gw_net_callbacks`, plus `cfg.now_ms`). Every method has the default of the
/// C's NULL callback; the `has_*` methods say whether the PULL callbacks / lobby are present.
pub trait Callbacks {
    /// A remote payload, exactly once, strictly increasing frame order (all slots of frame f in
    /// ascending slot order before any of f+1). `payload.len()` = payload bytes.
    fn remote_input(&mut self, _frame: i32, _slot: usize, _payload: &[u8]) {}
    /// PULL model: is `local_input` provided? (C: `cb.local_input != NULL`.)
    fn has_local_input(&self) -> bool {
        false
    }
    /// PULL: write the local input for (frame, slot) into `out`; false = not yet available.
    fn local_input(&mut self, _frame: i32, _slot: usize, _out: &mut [u8]) -> bool {
        false
    }
    /// PULL: is `checksum` provided?
    fn has_checksum(&self) -> bool {
        false
    }
    /// PULL: the checksum of the state at the START of `frame`, if known.
    fn checksum(&mut self, _frame: i32) -> Option<u32> {
        None
    }
    fn event(&mut self, _ev: Event, _msg: Option<&str>) {}
    fn desync(&mut self, _frame: i32, _local: u32, _remote: u32) {}
    /// Host: a guest's HELLO was accepted; may rewrite `blob[..*blob_len]` (cap = blob.len()).
    /// Setting `*blob_len` over the cap refuses this guest.
    fn guest_hello(&mut self, _info: &[u8], _blob: &mut [u8], _blob_len: &mut u16) {}
    /// LOBBY: is `lobby_msg` provided? (C: `cb.lobby_msg != NULL`; it also enables heartbeats.)
    fn has_lobby(&self) -> bool {
        false
    }
    fn lobby_msg(&mut self, _data: &[u8]) {}
    /// The clock in ms; `None` = the wall clock (C: `cfg.now_ms == NULL`).
    fn now_ms(&self) -> Option<u32> {
        None
    }
}

/// A callbacks object that does nothing (push model, wall clock).
pub struct NoCallbacks;
impl Callbacks for NoCallbacks {}

/// `gw_net_config` minus the callbacks (passed separately).
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub exe_hash: u64,
    pub iso_hash: u64,
    pub mods_hash: u64,
    pub mods_desc: Option<String>,
    pub mods_soft: bool,
    pub first_frame: i32,
    /// 0 = DEFAULT_PAYLOAD; at most MAX_PAYLOAD.
    pub payload_bytes: u8,
    pub seed: u32,
    pub input_delay: u8,
    pub host_slots: u8,
    pub guest_slots: u8,
    pub rules: u32,
    pub rules_expect: bool,
    pub envoy: u32,
    pub envoy_expect: bool,
    pub match_blob: Vec<u8>,
    pub guest_info: Vec<u8>,
    pub hold_start: bool,
    pub disconnect_timeout_ms: u32,
    pub handshake_timeout_ms: u32,
    pub notify_timeout_ms: u32,
    pub frame_us: u32,
    /// Capability bits sent as a trailing byte of HELLO / ACCEPT (0 = not sent, the C build's wire;
    /// `early::CAP_EARLY`). Older peers (and the C build) ignore trailing bytes.
    pub caps: u8,
}

/// What `gw_net_remote_config` fills.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteConfig {
    pub seed: u32,
    pub input_delay: u8,
    pub host_slots: u8,
    pub guest_slots: u8,
    pub first_frame: i32,
    pub payload_bytes: u8,
    pub rules: u32,
    pub envoy: u32,
    pub match_blob: Vec<u8>,
}

/// `gw_net_stats`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub packets_sent: u32,
    pub packets_received: u32,
    pub duplicates: u32,
    pub out_of_order: u32,
    pub bad_packets: u32,
    pub inputs_delivered: u32,
    pub resent_frames: u32,
}

#[derive(Clone, Copy)]
struct LobbyMsg {
    seq: u16,
    len: u8,
    data: [u8; LOBBY_MAX],
}

type FrameBuf = [[u8; MAX_PAYLOAD]; MAX_SLOTS];

fn wall_now_ms() -> u32 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static T0: OnceLock<Instant> = OnceLock::new();
    let t0 = *T0.get_or_init(Instant::now);
    t0.elapsed().as_millis() as u64 as u32
}

#[inline]
fn reached(t: u32, at: u32) -> bool {
    (t.wrapping_sub(at) as i32) >= 0
}

/// One netplay connection (`gw_net`). Dropping it sends QUIT (x3) if still connected.
pub struct Net {
    cfg: Config,
    cb: Box<dyn Callbacks>,
    tp: Box<dyn Transport>,
    is_host: bool,
    state: State,
    peer: Addr,
    peer_locked: bool,
    session: u32,
    salt: u32,
    reason: String,

    last_rx_ms: u32,
    last_rx_tsend: u32,
    last_rx_at: u32,
    have_echo: bool,
    last_send_ms: u32,
    rtt_ms: u32,
    /// Display only: how long the packet being handled waited between landing (transport stamp) and this poll, the
    /// same for the last packet we echo (sent to the peer in flags bits 1-7), and the smoothed network round trip.
    rx_wait: u32,
    last_rx_wait: u32,
    net_rtt: f32,
    interrupted: bool,

    t_begin: u32,
    next_hello: u32,
    next_ready: u32,
    next_start: u32,
    start_time: u32,
    start_time_valid: bool,
    start_acked: bool,
    started_fired: bool,
    host_slots: u8,
    guest_slots: u8,
    seed: u32,
    input_delay: u8,
    blob: Box<[u8; MAX_BLOB]>,
    blob_len: u16,
    host_rules: u32,
    host_envoy: u32,
    peer_mods: u64,
    have_remote_cfg: bool,
    held: bool,
    lq: Vec<LobbyMsg>,
    lseq_next: u16,
    lrecv_next: u16,
    next_lobby: u32,
    peer_ready: bool,

    first: i32,
    pb: usize,
    lslots: Vec<usize>,
    rslots: Vec<usize>,
    max_per_packet: i32,

    local: Vec<FrameBuf>,
    local_next: u32,
    peer_ack: u32,

    remote: Vec<FrameBuf>,
    rtag: Vec<u32>,
    remote_next: u32,

    local_frame: u32,
    adv_local: f32,
    adv_remote: f32,
    have_adv: bool,
    cooldown: i32,
    frame_ms: f32,

    lchk: Vec<u32>,
    lchk_tag: Vec<u32>,
    rchk: Vec<u32>,
    rchk_tag: Vec<u32>,
    recent: [u32; MAX_CHECKSUMS],
    nrecent: usize,
    next_chk: u32,
    desync_frame: i32,
    desync_fired: bool,

    st: Stats,
    peer_caps: u8,
}

impl Net {
    fn make(cfg: &Config, cb: Box<dyn Callbacks>, tp: Box<dyn Transport>, is_host: bool) -> Option<Net> {
        let pb = if cfg.payload_bytes != 0 { cfg.payload_bytes as usize } else { DEFAULT_PAYLOAD };
        if pb > MAX_PAYLOAD {
            return None;
        }
        let mut cfg = cfg.clone();
        if cfg.disconnect_timeout_ms == 0 {
            cfg.disconnect_timeout_ms = 5000;
        }
        if cfg.notify_timeout_ms == 0 {
            cfg.notify_timeout_ms = 1000;
        }
        cfg.guest_info.truncate(MAX_GUEST_INFO);
        let frame_ms = (if cfg.frame_us != 0 { cfg.frame_us } else { 16667 }) as f32 / 1000.0f32;
        let ring = RING as usize;
        let mut n = Net {
            first: cfg.first_frame,
            held: cfg.hold_start,
            cfg,
            cb,
            tp,
            is_host,
            state: State::Idle,
            peer: Addr::default(),
            peer_locked: false,
            session: 0,
            salt: 0,
            reason: String::new(),
            last_rx_ms: 0,
            last_rx_tsend: 0,
            last_rx_at: 0,
            have_echo: false,
            last_send_ms: 0,
            rtt_ms: 0,
            rx_wait: 0,
            last_rx_wait: 0,
            net_rtt: 0.0,
            interrupted: false,
            t_begin: 0,
            next_hello: 0,
            next_ready: 0,
            next_start: 0,
            start_time: 0,
            start_time_valid: false,
            start_acked: false,
            started_fired: false,
            host_slots: 0,
            guest_slots: 0,
            seed: 0,
            input_delay: 0,
            blob: Box::new([0u8; MAX_BLOB]),
            blob_len: 0,
            host_rules: 0,
            host_envoy: 0,
            peer_mods: 0,
            have_remote_cfg: false,
            lq: Vec::with_capacity(LOBBY_QUEUE),
            lseq_next: 0,
            lrecv_next: 0,
            next_lobby: 0,
            peer_ready: false,
            pb,
            lslots: Vec::new(),
            rslots: Vec::new(),
            max_per_packet: 0,
            local: vec![[[0u8; MAX_PAYLOAD]; MAX_SLOTS]; ring],
            local_next: 0,
            peer_ack: 0,
            remote: vec![[[0u8; MAX_PAYLOAD]; MAX_SLOTS]; ring],
            rtag: vec![0; ring],
            remote_next: 0,
            local_frame: 0,
            adv_local: 0.0,
            adv_remote: 0.0,
            have_adv: false,
            cooldown: 0,
            frame_ms,
            lchk: vec![0; ring],
            lchk_tag: vec![0; ring],
            rchk: vec![0; ring],
            rchk_tag: vec![0; ring],
            recent: [0; MAX_CHECKSUMS],
            nrecent: 0,
            next_chk: 0,
            desync_frame: NO_FRAME,
            desync_fired: false,
            st: Stats::default(),
            peer_caps: 0,
        };
        // the C mixes the object's address into the session id; any per-instance value will do
        n.salt = (n.remote.as_ptr() as usize as u64 ^ ((n.remote.as_ptr() as usize as u64) >> 32)) as u32;
        n.t_begin = n.now();
        n.last_rx_ms = n.t_begin;
        Some(n)
    }

    /// Host: wait for one guest (`gw_net_host`). `None` on bad arguments.
    pub fn host(cfg: &Config, cb: Box<dyn Callbacks>, tp: Box<dyn Transport>) -> Option<Net> {
        if (cfg.host_slots & cfg.guest_slots) != 0
            || cfg.host_slots == 0
            || cfg.guest_slots == 0
            || cfg.match_blob.len() > MAX_BLOB
        {
            return None;
        }
        let mut n = Net::make(cfg, cb, tp, true)?;
        n.state = State::Listening;
        n.host_slots = cfg.host_slots;
        n.guest_slots = cfg.guest_slots;
        n.seed = cfg.seed;
        n.input_delay = cfg.input_delay;
        n.blob_len = cfg.match_blob.len() as u16;
        n.blob[..cfg.match_blob.len()].copy_from_slice(&cfg.match_blob);
        n.have_remote_cfg = true;
        n.setup_slots();
        Some(n)
    }

    /// Guest: connect to `host` (`gw_net_join`).
    pub fn join(cfg: &Config, cb: Box<dyn Callbacks>, tp: Box<dyn Transport>, host: Addr) -> Option<Net> {
        let mut n = Net::make(cfg, cb, tp, false)?;
        n.state = State::Connecting;
        n.peer = host;
        n.peer_locked = true;
        n.next_hello = n.t_begin;
        Some(n)
    }

    #[inline]
    fn now(&self) -> u32 {
        self.cb.now_ms().unwrap_or_else(wall_now_ms)
    }
    #[inline]
    fn to_frame(&self, idx: u32) -> i32 {
        self.first.wrapping_add(idx as i32)
    }

    fn set_reason(&mut self, msg: &str) {
        // char reason[96]: at most 95 bytes
        let b = msg.as_bytes();
        let n = b.len().min(95);
        self.reason = String::from_utf8_lossy(&b[..n]).into_owned();
    }

    fn fire(&mut self, ev: Event, msg: Option<&str>) {
        self.cb.event(ev, msg);
    }

    // ---- packet building / sending ----

    fn begin_packet(&self, w: &mut Writer, ty: u8) {
        let t = self.now();
        w.u16(MAGIC);
        w.u8(ty);
        // bit0: t_echo valid; bits 1-7 (display only, ignored by older builds and the C): how long the echoed packet
        // sat before our poll read it, ms, saturating at 127
        w.u8(if self.have_echo { 1 | (self.last_rx_wait.min(127) as u8) << 1 } else { 0 });
        w.u32(self.session);
        w.u32(t);
        w.u32(if self.have_echo { self.last_rx_tsend } else { 0 });
        w.u16(if self.have_echo { t.wrapping_sub(self.last_rx_at) & 0xFFFF } else { 0 });
    }

    fn send_packet(&mut self, buf: &[u8]) {
        if self.peer_locked {
            let peer = self.peer;
            self.tp.send(&peer, buf);
            self.st.packets_sent += 1;
            self.last_send_ms = self.now();
        }
    }

    fn send_simple(&mut self, ty: u8) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, ty);
        self.send_packet(&w.buf);
    }

    fn send_lobby(&mut self) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, T_LOBBY);
        w.u16(self.lrecv_next as u32);
        for m in &self.lq {
            if w.len() + 3 + m.len as usize > MAX_PACKET {
                break;
            }
            w.u16(m.seq as u32);
            w.u8(m.len);
            w.bytes(&m.data[..m.len as usize]);
        }
        self.send_packet(&w.buf);
    }

    fn on_lobby(&mut self, r: &mut Reader) {
        if r.left() < 2 {
            return;
        }
        let ack = r.u16() as u16;
        self.lq.retain(|m| (m.seq.wrapping_sub(ack) as i16) >= 0);
        while r.left() >= 3 {
            let seq = r.u16() as u16;
            let len = r.u8() as usize;
            if len > r.left() {
                break;
            }
            let data = r.take(len);
            if seq == self.lrecv_next {
                self.lrecv_next = self.lrecv_next.wrapping_add(1);
                if self.cb.has_lobby() {
                    self.cb.lobby_msg(data);
                }
            }
        }
    }

    /// LOBBY: queue a small reliable, ordered message (`gw_net_lobby_send`). 0, or < 0 if the
    /// queue is full or the message too long.
    pub fn lobby_send(&mut self, data: &[u8]) -> i32 {
        if data.len() > LOBBY_MAX || self.lq.len() >= LOBBY_QUEUE {
            return -1;
        }
        let mut m = LobbyMsg { seq: self.lseq_next, len: data.len() as u8, data: [0; LOBBY_MAX] };
        m.data[..data.len()].copy_from_slice(data);
        self.lseq_next = self.lseq_next.wrapping_add(1);
        self.lq.push(m);
        self.next_lobby = self.now();
        0
    }

    fn send_refuse_to(&mut self, to: &Addr, code: u8, msg: &str) {
        let mut w = Writer::new();
        let save = self.session;
        self.session = 0;
        self.begin_packet(&mut w, T_REFUSE);
        self.session = save;
        let b = msg.as_bytes();
        let len = b.len().min(230);
        w.u8(code);
        w.u8(len as u8);
        w.bytes(&b[..len]);
        self.tp.send(to, &w.buf);
        self.st.packets_sent += 1;
    }

    fn send_hello(&mut self) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, T_HELLO);
        w.u16(PROTOCOL_VERSION);
        w.u64(self.cfg.exe_hash);
        w.u64(self.cfg.iso_hash);
        w.u64(self.cfg.mods_hash);
        w.u8(self.pb as u8);
        w.u32(self.first as u32);
        w.u32(self.cfg.rules);
        w.u8(self.cfg.rules_expect as u8);
        w.u32(self.cfg.envoy);
        w.u8(self.cfg.envoy_expect as u8);
        w.u8(self.cfg.guest_info.len() as u8);
        w.bytes(&self.cfg.guest_info);
        if self.cfg.caps != 0 {
            w.u8(self.cfg.caps);
        }
        self.send_packet(&w.buf);
    }

    fn begin_start(&mut self) {
        let mut delay = 2u32.wrapping_mul(self.rtt_ms).wrapping_add(150);
        if delay < 300 {
            delay = 300;
        }
        self.start_time = self.now().wrapping_add(delay);
        self.start_time_valid = true;
        self.state = State::Starting;
        self.next_start = self.now();
        self.fire(Event::Starting, None);
    }

    fn send_accept(&mut self) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, T_ACCEPT);
        w.u16(PROTOCOL_VERSION);
        w.u32(self.seed);
        w.u8(self.input_delay);
        w.u8(self.host_slots);
        w.u8(self.guest_slots);
        w.u32(self.cfg.rules);
        w.u32(self.cfg.envoy);
        w.u16(self.blob_len as u32);
        w.bytes(&self.blob[..self.blob_len as usize]);
        if self.cfg.caps != 0 {
            w.u8(self.cfg.caps);
        }
        self.send_packet(&w.buf);
    }

    fn send_start(&mut self) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, T_START);
        let t = self.now();
        let remaining = self.start_time.wrapping_sub(t) as i32;
        w.u32(if remaining > 0 { remaining as u32 } else { 0 });
        self.send_packet(&w.buf);
    }

    /// INPUT body: u32 local_frame idx | s8 adv | u32 ack_next | u32 first idx | u8 count |
    /// count x (nl x payload bytes) | u8 nchk | nchk x (u32 frame idx, u32 hash)
    fn send_inputs(&mut self) {
        let mut w = Writer::new();
        self.begin_packet(&mut w, T_INPUT);
        let first = self.peer_ack;
        let avail = self.local_next.wrapping_sub(self.peer_ack);
        let count = if avail > self.max_per_packet as u32 { self.max_per_packet as u32 } else { avail };
        let mut adv = (self.adv_local + if self.adv_local >= 0.0 { 0.5f32 } else { -0.5f32 }) as i32;
        adv = adv.clamp(-127, 127);
        w.u32(self.local_frame);
        w.u8(adv as i8 as u8);
        w.u32(self.remote_next);
        w.u32(first);
        w.u8(count as u8);
        for i in 0..count {
            let fb = &self.local[(first.wrapping_add(i) % RING) as usize];
            for &s in &self.lslots {
                w.bytes(&fb[s][..self.pb]);
            }
        }
        w.u8(self.nrecent as u8);
        for j in 0..self.nrecent {
            let f = self.recent[j];
            w.u32(f);
            w.u32(self.lchk[(f % RING) as usize]);
        }
        self.st.resent_frames = self.st.resent_frames.wrapping_add(if count > 1 { count - 1 } else { 0 });
        self.send_packet(&w.buf);
    }

    // ---- slot lists ----

    fn setup_slots(&mut self) {
        let lm = if self.is_host { self.host_slots } else { self.guest_slots };
        let rm = if self.is_host { self.guest_slots } else { self.host_slots };
        self.lslots = (0..MAX_SLOTS).filter(|i| lm & (1u8 << i) != 0).collect();
        self.rslots = (0..MAX_SLOTS).filter(|i| rm & (1u8 << i) != 0).collect();
        let nl = self.lslots.len() as i32;
        self.max_per_packet = if nl > 0 { 1000 / (self.pb as i32 * nl) } else { 1 };
        if self.max_per_packet > MAX_INPUTS_PER_PACKET {
            self.max_per_packet = MAX_INPUTS_PER_PACKET;
        }
        if self.max_per_packet < 1 {
            self.max_per_packet = 1;
        }
    }

    /// cfg.hold_start: this side is ready; agree the start time once the peer is too.
    pub fn release(&mut self) {
        if !self.held {
            return;
        }
        self.held = false;
        if self.is_host {
            if self.state == State::Accepted && self.peer_ready {
                self.begin_start();
            }
        } else {
            self.next_ready = self.now();
        }
    }

    // ---- receiving ----

    fn rtt_sample(&mut self, t_echo: u32, echo_delay: u32) {
        let t = self.now();
        let mut s = t.wrapping_sub(t_echo).wrapping_sub(echo_delay) as i32;
        if !(0..=10000).contains(&s) {
            return;
        }
        if s < 1 {
            s = 1;
        }
        self.rtt_ms = if self.rtt_ms == 0 {
            s as u32
        } else {
            (self.rtt_ms.wrapping_mul(7).wrapping_add(s as u32).wrapping_add(4)) / 8
        };
    }

    /// The displayed round trip: like `rtt_sample` (which game timing keeps using unchanged) but measured from when
    /// the packet landed, minus the peer's whole hold (its echo delay plus its own wait before reading ours).
    fn net_rtt_sample(&mut self, t_echo: u32, echo_delay: u32, peer_wait: u8) {
        if let Some(s) = net_rtt_ms_sample(self.now(), self.rx_wait, t_echo, echo_delay, peer_wait as u32) {
            self.net_rtt = if self.net_rtt == 0.0 { s as f32 } else { self.net_rtt * 0.875 + s as f32 * 0.125 };
        }
    }

    fn check_checksum_pair(&mut self, idx: u32) {
        let slot = (idx % RING) as usize;
        let frame = self.to_frame(idx);
        let tag = idx.wrapping_add(1);
        if self.lchk_tag[slot] != tag || self.rchk_tag[slot] != tag {
            return;
        }
        if self.lchk[slot] == self.rchk[slot] {
            return;
        }
        if self.desync_frame == NO_FRAME || frame < self.desync_frame {
            self.desync_frame = frame;
        }
        if !self.desync_fired {
            self.desync_fired = true;
            let (l, r) = (self.lchk[slot], self.rchk[slot]);
            self.cb.desync(frame, l, r);
        }
    }

    fn go_dead(&mut self, msg: &str) {
        if self.state == State::Dead {
            return;
        }
        self.state = State::Dead;
        self.set_reason(msg);
        self.fire(Event::Disconnected, Some(msg));
    }

    fn deliver_remote(&mut self) {
        loop {
            let slot = (self.remote_next % RING) as usize;
            if self.rtag[slot] != self.remote_next.wrapping_add(1) {
                break;
            }
            let frame = self.to_frame(self.remote_next);
            for &s in &self.rslots {
                self.cb.remote_input(frame, s, &self.remote[slot][s][..self.pb]);
            }
            self.rtag[slot] = 0;
            self.remote_next = self.remote_next.wrapping_add(1);
            self.st.inputs_delivered = self.st.inputs_delivered.wrapping_add(1);
        }
    }

    fn on_input(&mut self, r: &mut Reader) {
        let per_frame = self.rslots.len() * self.pb;
        if self.state != State::Starting && self.state != State::Running {
            return;
        }
        if r.left() < 4 + 1 + 4 + 4 + 1 {
            self.st.bad_packets += 1;
            return;
        }
        let lf = r.u32();
        let adv_remote = r.u8() as i8 as i32;
        let ack_next = r.u32();
        let first = r.u32();
        let count = r.u8() as u32;
        if r.left() < count as usize * per_frame + 1 {
            self.st.bad_packets += 1;
            return;
        }
        self.start_acked = true;

        if ack_next > self.peer_ack && ack_next <= self.local_next {
            self.peer_ack = ack_next;
        }

        let mut gap = false;
        for i in 0..count {
            let frame = first.wrapping_add(i);
            let slot = (frame % RING) as usize;
            if frame < self.remote_next {
                r.skip(per_frame);
                self.st.duplicates += 1;
                continue;
            }
            if frame >= self.remote_next.wrapping_add(RING) {
                r.skip(per_frame);
                continue;
            }
            if frame > self.remote_next {
                gap = true;
            }
            if self.rtag[slot] == frame.wrapping_add(1) {
                r.skip(per_frame);
                self.st.duplicates += 1;
                continue;
            }
            for j in 0..self.rslots.len() {
                let s = self.rslots[j];
                let src = r.take(self.pb);
                self.remote[slot][s][..self.pb].copy_from_slice(src);
            }
            self.rtag[slot] = frame.wrapping_add(1);
        }
        if gap {
            self.st.out_of_order += 1;
        }
        self.deliver_remote();

        let nchk = r.u8() as usize;
        if r.left() >= nchk * 8 {
            for _ in 0..nchk {
                let f = r.u32();
                let h = r.u32();
                let slot = (f % RING) as usize;
                self.rchk[slot] = h;
                self.rchk_tag[slot] = f.wrapping_add(1);
                self.check_checksum_pair(f);
            }
        }

        // frame advantage: where the peer is NOW, given the frame it stamped and half the RTT
        let remote_now = lf as f32 + (self.rtt_ms as f32 / 2.0f32) / self.frame_ms;
        let adv = self.local_frame as f32 - remote_now;
        if !self.have_adv {
            self.adv_local = adv;
            self.adv_remote = adv_remote as f32;
            self.have_adv = true;
        } else {
            // fmuladd on i686 (no FMA): separate f32 multiply and add
            self.adv_local += (adv - self.adv_local) * ADV_EMA;
            self.adv_remote += (adv_remote as f32 - self.adv_remote) * ADV_EMA;
        }
    }

    /// The C's refuse_check: `Some(reason)` if this HELLO must be refused.
    #[allow(clippy::too_many_arguments)]
    fn refuse_check(
        &self,
        ver: u32,
        exe: u64,
        iso: u64,
        mods: u64,
        pb: usize,
        first: i32,
        rules: u32,
        rules_expect: bool,
        envoy: u32,
        envoy_expect: bool,
    ) -> Option<String> {
        if ver != PROTOCOL_VERSION {
            return Some(format!("protocol version mismatch (host {}, you {})", PROTOCOL_VERSION, ver));
        }
        if exe != self.cfg.exe_hash {
            return Some(format!(
                "different game build (host {:08x}, you {:08x}): the sources or the floating-point results differ",
                self.cfg.exe_hash as u32, exe as u32
            ));
        }
        if iso != self.cfg.iso_hash {
            return Some(format!("different disc image (host {:08x}, you {:08x})", self.cfg.iso_hash as u32, iso as u32));
        }
        if mods != self.cfg.mods_hash && !self.cfg.mods_soft {
            let d = match &self.cfg.mods_desc {
                Some(s) if !s.is_empty() => s.as_str(),
                _ => "none",
            };
            // snprintf into 231 bytes: at most 230 characters; a cut list ends "..."
            let full = format!("different global game data; host has: {}", d);
            let b = full.as_bytes();
            if b.len() >= 231 {
                let mut cut = b[..227].to_vec();
                cut.extend_from_slice(b"...");
                return Some(String::from_utf8_lossy(&cut).into_owned());
            }
            return Some(full);
        }
        if pb != self.pb || first != self.first {
            return Some(format!(
                "different netplay settings (input size {} vs {}, first frame {} vs {})",
                self.pb, pb, self.first, first
            ));
        }
        if rules_expect && rules != self.cfg.rules {
            return Some(format!("different match rules (host {:08x}, you {:08x})", self.cfg.rules, rules));
        }
        if envoy != self.cfg.envoy && (envoy_expect || !envoy_mode_valid(envoy)) {
            return Some(format!("different Envoy rules (host {:08x}, you {:08x})", self.cfg.envoy, envoy));
        }
        None
    }

    fn refuse_self(&mut self, why: &str) {
        self.state = State::Refused;
        self.set_reason(why);
        let r = self.reason.clone();
        self.fire(Event::Refused, Some(&r));
    }

    fn on_packet(&mut self, from: &Addr, buf: &[u8]) {
        if buf.len() < HDR_LEN {
            self.st.bad_packets += 1;
            return;
        }
        let mut r = Reader::new(buf);
        if r.u16() != MAGIC {
            self.st.bad_packets += 1;
            return;
        }
        let ty = r.u8();
        let flags = r.u8();
        let session = r.u32();
        let t_send = r.u32();
        let t_echo = r.u32();
        let echo_delay = r.u16();

        // Host, not yet locked to a guest: only a HELLO means anything.
        if self.is_host && !self.peer_locked {
            if ty != T_HELLO || r.left() < 2 + 24 + 1 + 4 {
                return;
            }
            let ver = r.u16();
            let exe = r.u64();
            let iso = r.u64();
            let mods = r.u64();
            let pb = r.u8() as usize;
            let first = r.u32() as i32;
            let cur = ver == PROTOCOL_VERSION;
            if cur && r.left() < 5 + 5 {
                return;
            }
            let rules = if cur { r.u32() } else { 0 };
            let rules_expect = if cur { r.u8() != 0 } else { false };
            let envoy = if cur { r.u32() } else { 0 };
            let envoy_expect = if cur { r.u8() != 0 } else { false };
            if let Some(why) = self.refuse_check(ver, exe, iso, mods, pb, first, rules, rules_expect, envoy, envoy_expect) {
                self.send_refuse_to(from, 1, &why);
                return; // stay listening: a refused guest does not consume the slot
            }
            self.peer_mods = mods;
            {
                let mut il = if r.left() >= 1 { r.u8() as usize } else { 0 };
                if il > r.left() {
                    il = r.left();
                }
                if il > MAX_GUEST_INFO {
                    il = MAX_GUEST_INFO;
                }
                let info = &buf[r.pos..r.pos + il];
                self.cb.guest_hello(info, &mut self.blob[..], &mut self.blob_len);
                // the trailing caps byte, right after the declared guest info
                let declared = buf.get(r.pos - 1).copied().unwrap_or(0) as usize;
                self.peer_caps = if r.left() > declared { buf[r.pos + declared] } else { 0 };
            }
            if self.blob_len as usize > MAX_BLOB {
                self.blob_len = 0;
                self.send_refuse_to(from, 1, "match setup exceeds transport limit");
                return;
            }
            self.peer = *from;
            self.peer_locked = true;
            self.session = self.now().wrapping_mul(2654435761)
                ^ from.ip.wrapping_mul(40503)
                ^ ((from.port as u32) << 16)
                ^ self.salt;
            if self.session == 0 {
                self.session = 1;
            }
            self.state = State::Accepted;
            self.last_rx_ms = self.now();
            self.last_rx_tsend = t_send;
            self.last_rx_at = self.now();
            self.last_rx_wait = self.rx_wait;
            self.have_echo = true;
            self.send_accept();
            return;
        }

        if *from != self.peer {
            if self.is_host && ty == T_HELLO {
                self.send_refuse_to(from, 2, "session full");
            }
            return;
        }

        if ty == T_HELLO {
            if self.is_host && self.state == State::Accepted {
                self.send_accept();
            }
            return;
        }

        if !self.is_host && self.state == State::Connecting {
            if ty == T_REFUSE && r.left() >= 2 {
                r.u8(); // code
                let mut l = r.u8() as usize;
                if l > r.left() {
                    l = r.left();
                }
                if l > 230 {
                    l = 230;
                }
                let why = String::from_utf8_lossy(r.take(l)).into_owned();
                self.state = State::Refused;
                self.set_reason(&why);
                self.fire(Event::Refused, Some(&why));
                return;
            }
            if ty == T_ACCEPT {
                if r.left() < 2 {
                    self.st.bad_packets += 1;
                    return;
                }
                let ver = r.u16();
                if ver != PROTOCOL_VERSION {
                    self.refuse_self("host speaks a different protocol version");
                    return;
                }
                if r.left() < 4 + 3 + 4 + 4 + 2 {
                    self.st.bad_packets += 1;
                    return;
                }
                self.seed = r.u32();
                self.input_delay = r.u8();
                self.host_slots = r.u8();
                self.guest_slots = r.u8();
                self.host_rules = r.u32();
                self.host_envoy = r.u32();
                if self.cfg.rules_expect && self.host_rules != self.cfg.rules {
                    let why = format!("different match rules (host {:08x}, you {:08x})", self.host_rules, self.cfg.rules);
                    self.refuse_self(&why);
                    return;
                }
                if !envoy_mode_valid(self.host_envoy) || (self.cfg.envoy_expect && self.host_envoy != self.cfg.envoy) {
                    let why = format!("different Envoy rules (host {:08x}, you {:08x})", self.host_envoy, self.cfg.envoy);
                    self.refuse_self(&why);
                    return;
                }
                let bl = r.u16() as usize;
                if bl > MAX_BLOB || r.left() < bl {
                    self.st.bad_packets += 1;
                    return;
                }
                self.blob_len = bl as u16;
                self.blob[..bl].copy_from_slice(r.take(bl));
                self.peer_caps = if r.left() >= 1 { r.u8() } else { 0 };
                self.session = session;
                self.have_remote_cfg = true;
                self.setup_slots();
                self.state = State::Accepted;
                self.last_rx_ms = self.now();
                self.last_rx_tsend = t_send;
                self.last_rx_at = self.now();
                self.last_rx_wait = self.rx_wait;
                self.have_echo = true;
                if flags & 1 != 0 {
                    self.rtt_sample(t_echo, echo_delay);
                    self.net_rtt_sample(t_echo, echo_delay, flags >> 1);
                }
                self.next_ready = self.now();
                self.fire(Event::Accepted, None);
            }
            return;
        }

        if session != self.session {
            return; // a stray from an old session
        }

        self.last_rx_ms = self.now();
        self.last_rx_tsend = t_send;
        self.last_rx_at = self.now();
        self.last_rx_wait = self.rx_wait;
        self.have_echo = true;
        self.st.packets_received += 1;
        if flags & 1 != 0 {
            self.rtt_sample(t_echo, echo_delay);
            self.net_rtt_sample(t_echo, echo_delay, flags >> 1);
        }
        if self.interrupted {
            self.interrupted = false;
            self.fire(Event::Resumed, None);
        }

        match ty {
            T_ACCEPT => {}
            T_READY => {
                if self.is_host && self.state == State::Accepted {
                    if self.held {
                        self.peer_ready = true;
                    } else {
                        self.begin_start();
                    }
                }
            }
            T_START => {
                if !self.is_host && self.state == State::Accepted && r.left() >= 4 {
                    let delay = r.u32();
                    let half = self.rtt_ms / 2;
                    self.start_time = self.now().wrapping_add(if delay > half { delay - half } else { 0 });
                    self.start_time_valid = true;
                    self.state = State::Starting;
                    self.fire(Event::Starting, None);
                }
                if !self.is_host && (self.state == State::Starting || self.state == State::Running) {
                    self.send_simple(T_START_ACK);
                }
            }
            T_START_ACK => self.start_acked = true,
            T_INPUT => self.on_input(&mut r),
            T_LOBBY => self.on_lobby(&mut r),
            T_QUIT => self.go_dead("peer quit"),
            _ => self.st.bad_packets += 1,
        }
    }

    // ---- pulling from the session ----

    fn accept_local(&mut self, idx: u32, src: &FrameBuf) -> i32 {
        if idx != self.local_next || self.local_next.wrapping_sub(self.peer_ack) >= RING - 8 {
            return -1;
        }
        let dst = &mut self.local[(idx % RING) as usize];
        for &s in &self.lslots {
            dst[s][..self.pb].copy_from_slice(&src[s][..self.pb]);
        }
        self.local_next = self.local_next.wrapping_add(1);
        0
    }

    fn pull_local(&mut self) {
        if !self.cb.has_local_input() || !self.have_remote_cfg || self.lslots.is_empty() {
            return;
        }
        for _ in 0..MAX_PULL_PER_POLL {
            let mut tmp: FrameBuf = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
            if self.local_next.wrapping_sub(self.peer_ack) >= RING - 8 {
                break;
            }
            let frame = self.to_frame(self.local_next);
            let mut ok = true;
            for j in 0..self.lslots.len() {
                let s = self.lslots[j];
                ok = self.cb.local_input(frame, s, &mut tmp[s][..self.pb]);
                if !ok {
                    break;
                }
            }
            if !ok {
                break;
            }
            let idx = self.local_next;
            self.accept_local(idx, &tmp);
        }
    }

    fn report_idx(&mut self, idx: u32, hash: u32) {
        let slot = (idx % RING) as usize;
        self.lchk[slot] = hash;
        self.lchk_tag[slot] = idx.wrapping_add(1);
        if self.nrecent == MAX_CHECKSUMS {
            self.recent.copy_within(1.., 0);
            self.nrecent -= 1;
        }
        self.recent[self.nrecent] = idx;
        self.nrecent += 1;
        self.check_checksum_pair(idx);
    }

    /// The state at the START of frame f depends on inputs < f, so f is final once
    /// f <= remote_next.
    fn pull_checksums(&mut self) {
        if !self.cb.has_checksum() {
            return;
        }
        let mut k = 0;
        while k < 8 && self.next_chk <= self.remote_next {
            let frame = self.to_frame(self.next_chk);
            if let Some(h) = self.cb.checksum(frame) {
                let idx = self.next_chk;
                self.report_idx(idx, h);
                self.next_chk = self.next_chk.wrapping_add(1);
            } else if self.remote_next - self.next_chk > 8 {
                self.next_chk = self.next_chk.wrapping_add(1);
            } else {
                break;
            }
            k += 1;
        }
    }

    // ---- the frame poll ----

    /// Once per logic-frame iteration (`gw_net_poll`). `local_frame` = the frame the session will
    /// simulate next.
    pub fn poll(&mut self, local_frame: i32) {
        if self.state == State::Dead || self.state == State::Refused {
            return;
        }
        self.local_frame = if local_frame > self.first { local_frame.wrapping_sub(self.first) as u32 } else { 0 };
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }

        let mut buf = [0u8; MAX_PACKET + 64];
        for _ in 0..64 {
            let mut from = Addr::default();
            crate::transport::set_rx_stamp(None);
            let r = self.tp.recv(&mut from, &mut buf);
            self.rx_wait = if self.cb.now_ms().is_none() { crate::transport::rx_wait_ms() } else { 0 };
            if r < 0 {
                self.go_dead("network error");
                return;
            }
            if r == 0 {
                break;
            }
            let len = (r as usize).min(buf.len());
            self.on_packet(&from, &buf[..len]);
            if self.state == State::Dead || self.state == State::Refused {
                return;
            }
        }
        let t = self.now();

        match self.state {
            State::Connecting => {
                let to = if self.cfg.handshake_timeout_ms != 0 { self.cfg.handshake_timeout_ms } else { HANDSHAKE_TIMEOUT_MS };
                if reached(t, self.t_begin.wrapping_add(to)) {
                    self.go_dead("handshake timed out (no host answered)");
                    return;
                }
                if reached(t, self.next_hello) {
                    self.send_hello();
                    self.next_hello = t.wrapping_add(HELLO_INTERVAL_MS);
                }
            }
            State::Accepted => {
                if !self.is_host && !self.held && reached(t, self.next_ready) {
                    self.send_simple(T_READY);
                    self.next_ready = t.wrapping_add(READY_INTERVAL_MS);
                }
            }
            State::Starting => {
                if self.is_host && !self.start_acked && reached(t, self.next_start) {
                    self.send_start();
                    self.next_start = t.wrapping_add(START_INTERVAL_MS);
                }
            }
            _ => {}
        }

        if self.state == State::Starting && self.start_time_valid && reached(t, self.start_time) {
            self.state = State::Running;
            if !self.started_fired {
                self.started_fired = true;
                self.fire(Event::Started, None);
            }
        }

        if self.peer_locked && self.state >= State::Accepted && self.state < State::Dead {
            let silent = t.wrapping_sub(self.last_rx_ms);
            let limit = if self.cfg.hold_start && self.state < State::Starting {
                HOLD_TIMEOUT_MS
            } else {
                self.cfg.disconnect_timeout_ms
            };
            if silent >= limit {
                self.go_dead("peer timed out");
                return;
            }
            if silent >= self.cfg.notify_timeout_ms && !self.interrupted {
                self.interrupted = true;
                self.fire(Event::Interrupted, None);
            }
        }

        if (self.state == State::Accepted
            || self.state == State::Starting
            || (self.state == State::Running && !self.lq.is_empty()))
            && self.session != 0
            && (self.cb.has_lobby() || !self.lq.is_empty())
            && reached(t, self.next_lobby)
        {
            self.send_lobby();
            self.next_lobby = t.wrapping_add(if !self.lq.is_empty() { 100 } else { 500 });
        }

        if self.state == State::Running {
            self.pull_local();
            self.pull_checksums();
            if reached(t, self.last_send_ms.wrapping_add(MIN_SEND_GAP_MS)) {
                self.send_inputs();
            }
        }
    }

    /// Right after the session ran this tick's frames: take the local inputs it now has and send them at once,
    /// instead of at the next tick's `poll` (~one render tick later). Same INPUT packet; nothing is read.
    pub fn send_local_now(&mut self, local_frame: i32) {
        if self.state != State::Running {
            return;
        }
        let before = self.local_next;
        self.local_frame = if local_frame > self.first { local_frame.wrapping_sub(self.first) as u32 } else { 0 };
        self.pull_local();
        if self.local_next != before {
            self.send_inputs();
        }
    }

    // ---- session-facing API ----

    /// PUSH model: this peer's inputs for `frame`, indexed by slot (`gw_net_submit_local`).
    /// 0, -1 (no config / no local slots), -2 (out of sequence), -3 (send window full).
    pub fn submit_local(&mut self, frame: i32, inputs: &[Input; MAX_SLOTS]) -> i32 {
        if !self.have_remote_cfg || self.lslots.is_empty() {
            return -1;
        }
        if frame < self.first || frame.wrapping_sub(self.first) as u32 != self.local_next {
            return -2;
        }
        let mut tmp: FrameBuf = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
        for &s in &self.lslots {
            tmp[s] = inputs[s];
        }
        let idx = self.local_next;
        if self.accept_local(idx, &tmp) == 0 {
            0
        } else {
            -3
        }
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn started(&self) -> bool {
        self.state == State::Running
    }
    pub fn start_time_ms(&self) -> u32 {
        self.start_time
    }
    pub fn local_slots(&self) -> u8 {
        if self.is_host {
            self.host_slots
        } else {
            self.guest_slots
        }
    }
    pub fn remote_slots(&self) -> u8 {
        if self.is_host {
            self.guest_slots
        } else {
            self.host_slots
        }
    }
    pub fn last_reason(&self) -> &str {
        &self.reason
    }
    pub fn is_host(&self) -> bool {
        self.is_host
    }
    pub fn payload_bytes(&self) -> usize {
        self.pb
    }
    pub fn first_frame(&self) -> i32 {
        self.first
    }

    /// The host's setup as this peer has it (`gw_net_remote_config`); `None` before ACCEPTED.
    pub fn remote_config(&self) -> Option<RemoteConfig> {
        if !self.have_remote_cfg {
            return None;
        }
        Some(RemoteConfig {
            seed: self.seed,
            input_delay: self.input_delay,
            host_slots: self.host_slots,
            guest_slots: self.guest_slots,
            first_frame: self.first,
            payload_bytes: self.pb as u8,
            rules: if self.is_host { self.cfg.rules } else { self.host_rules },
            envoy: if self.is_host { self.cfg.envoy } else { self.host_envoy },
            match_blob: self.blob[..self.blob_len as usize].to_vec(),
        })
    }

    /// Newest frame whose remote inputs are all delivered; first_frame - 1 if none.
    pub fn remote_confirmed_frame(&self) -> i32 {
        self.first.wrapping_add(self.remote_next as i32).wrapping_sub(1)
    }
    pub fn unacked(&self) -> u32 {
        self.local_next.wrapping_sub(self.peer_ack)
    }
    /// The peer's capability bits (0: an older build or the C build).
    pub fn peer_caps(&self) -> u8 {
        self.peer_caps
    }
    /// Where the peer is (valid once connected).
    pub fn peer_addr(&self) -> Addr {
        self.peer
    }
    /// The round trip game timing uses (frame advantage, start time, `--np-delay auto`): it includes each side's
    /// wait until its frame poll reads a packet.
    pub fn rtt_ms(&self) -> u32 {
        self.rtt_ms
    }
    /// The network round trip, for display and logs (0 until measured; falls back to `rtt_ms` against a peer
    /// that sends no wait bits only in that its wait reads 0).
    pub fn net_rtt_ms(&self) -> u32 {
        self.net_rtt.round() as u32
    }
    pub fn silent_ms(&self) -> u32 {
        self.now().wrapping_sub(self.last_rx_ms)
    }
    pub fn frame_advantage(&self) -> f32 {
        self.adv_local
    }
    pub fn peer_mods_hash(&self) -> u64 {
        self.peer_mods
    }

    /// GGPO's rule: the peer that is ahead waits half the gap (`gw_net_recommend_wait`).
    /// Consumes the recommendation; recommendations are spaced by a cooldown.
    pub fn recommend_wait(&mut self) -> i32 {
        if !self.have_adv || self.cooldown > 0 || self.state != State::Running {
            return 0;
        }
        let mut w = ((self.adv_local - self.adv_remote) / 2.0f32) as i32;
        if w < 1 {
            return 0;
        }
        if w > 8 {
            w = 8;
        }
        self.cooldown = 2 * w + 6;
        w
    }

    /// PUSH model: the checksum of the state at the start of `frame`.
    pub fn report_checksum(&mut self, frame: i32, hash: u32) {
        if frame < self.first {
            return;
        }
        self.report_idx(frame.wrapping_sub(self.first) as u32, hash);
    }
    /// First mismatching frame, NO_FRAME if none.
    pub fn desync_frame(&self) -> i32 {
        self.desync_frame
    }
    pub fn stats(&self) -> Stats {
        self.st
    }

    /// Access to the callbacks object (e.g. to read a test session's state).
    pub fn callbacks(&self) -> &dyn Callbacks {
        &*self.cb
    }
    pub fn callbacks_mut(&mut self) -> &mut dyn Callbacks {
        &mut *self.cb
    }
}

impl Drop for Net {
    /// `gw_net_free`: QUIT x3 if still connected; the transport is dropped with the object.
    fn drop(&mut self) {
        if self.peer_locked && matches!(self.state, State::Accepted | State::Starting | State::Running) {
            for _ in 0..3 {
                self.send_simple(T_QUIT);
            }
        }
    }
}

/// One displayed-round-trip sample (ms), NTP style: our arrival time (`now - rx_wait`) minus our send time
/// (`t_echo`) minus the peer's hold (`echo_delay`, its processing to send, plus `peer_wait`, its landing to
/// processing). None when out of range (clock wrap, a stale echo).
pub fn net_rtt_ms_sample(now: u32, rx_wait: u32, t_echo: u32, echo_delay: u32, peer_wait: u32) -> Option<u32> {
    let s = now.wrapping_sub(rx_wait).wrapping_sub(t_echo).wrapping_sub(echo_delay).wrapping_sub(peer_wait) as i32;
    (0..=10000).contains(&s).then_some((s as u32).max(1))
}

#[cfg(test)]
mod net_rtt_tests {
    use super::net_rtt_ms_sample;
    #[test]
    fn subtracts_both_sides_waits() {
        // sent at 1000; the peer's poll read it 9 ms after it landed and replied 7 ms later; the 2 ms + 2 ms network
        // trip puts the reply here at 1020, read by our poll at 1031 (wait 11)
        assert_eq!(net_rtt_ms_sample(1031, 11, 1000, 7, 9), Some(4));
        // without stamps (waits 0) it is the old frame-quantized figure
        assert_eq!(net_rtt_ms_sample(1031, 0, 1000, 7, 0), Some(24));
        // wraps across the u32 clock
        assert_eq!(net_rtt_ms_sample(5, 1, u32::MAX - 10, 2, 1), Some(12));
        // a hold longer than the trip (bad stamp): rejected
        assert_eq!(net_rtt_ms_sample(1000, 0, 999, 5, 0), None);
    }
}
