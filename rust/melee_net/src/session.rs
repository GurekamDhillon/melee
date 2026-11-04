//! The rollback SESSION: a native port of `pc/platform/gw_rollback.c` (input delay, prediction,
//! rollback, stalls, time-sync waits, the per-frame curated checksum ring, the fake network and
//! the live-pad latch), plus the resimulation driver that `gmscene.c`'s scene loop is in the C.
//!
//! What the C reaches through externs is a trait here, [`RbGame`]: the frame counter
//! (`gw_Replay_Frame`), the recorded / scripted inputs (`gw_Replay_PeekInput`), the savestates
//! (`gw_snap_save` / `gw_snap_load`: here a snapshot of the NATIVE game state, whatever the game's
//! `Snapshot` type holds, kept in a ring of `maxb + 3` slots), the whole-state hash
//! (`gw_Snap_Checksum`, SyncTest's compare) and the curated gameplay hash
//! (`gw_RB_GameHashForFrame`, the neutral hash: what two machines must agree on).
//!
//! Flow per render tick ([`run_tick`], the C's gmscene gm_801A4D34 loop):
//! `iterations()` (deliver, plan the rollback / stall / wait, return k + 1), then per iteration
//! `iter_start()` (load the snapshot on a rollback's first iteration, fix the frame's inputs, hash,
//! save), the game's frame (`RbGame::run_frame`, reading the inputs `input_any` / `input_for`
//! return), and `iter_end()`.
//!
//! Ported line for line: rb_init's configuration (from [`RbConfig`] or the same environment
//! variables), gw_RB_SceneBegin (epochs), every network-facing function (`_e` epochs, the pre-open
//! early buffer, refusal beyond the ring), rb_same, rb_neutral_raw, rb_hash, rb_gen (padgen),
//! rb_conf, rb_hash_final, the fake network (rb_arrival, rb_fake_deliver, live samples in flight),
//! gw_RB_PadLatch / rb_live_sample, gw_RB_Iterations (open, hash log, network clock, plan, stall,
//! wait), rb_prepare, gw_RB_IterStart, gw_RB_InputFor / InputAny / PadGoverned / PadField.
//! Not ported (game/app couplings that live in other modules): Slippi fixture mode
//! (gw_rb_slippi_*), Envoy build apply, Lua fault undo, the Lab's visualiser, the profiler and
//! wall-clock statistics, MELEE_RB_PERTURB, rb_hash_detail, sound suppression during
//! resimulation (`RbGame::run_frame` gets `resim` instead).

use crate::pad::RbInput;
use crate::rollback::RollbackSession;

/// `GW_RB_SLOTS`: 4 ports x (leader, follower).
pub const SLOTS: usize = 8;
/// `RB_RING`: frames of history per slot.
pub const RING: usize = 64;
/// `RB_FIRST`: Slippi's first frame.
pub const FIRST: i32 = -123;
const NONE: i32 = i32::MAX;

/// What the session needs from the game (the C's externs). The frame counter is part of the
/// snapshot: a load rewinds it.
pub trait RbGame {
    type Snapshot: Clone;
    /// `gw_Replay_Frame`: the newest simulated frame (`FIRST - 1` once the match is armed).
    fn frame(&self) -> i32;
    /// `gw_Replay_LastFrame` (the replay's last frame; `i32::MAX` when open-ended).
    fn last_frame(&self) -> i32 {
        i32::MAX
    }
    /// `gw_Replay_Live` / `gw_Replay_PortHuman`: live pads decide which ports play.
    fn live(&self) -> bool {
        false
    }
    fn port_human(&self, _port: usize) -> bool {
        false
    }
    /// `gw_Replay_PeekInput`: the recorded input for (slot, frame), the fake network's "player".
    fn peek_input(&self, slot: usize, frame: i32) -> Option<RbInput>;
    /// `gw_snap_save` / `gw_snap_load`: the whole native game state.
    fn save(&self) -> Self::Snapshot;
    fn load(&mut self, s: &Self::Snapshot);
    /// `gw_Snap_Checksum`: a whole-state hash (valid within one process only).
    fn state_hash(&self) -> u32;
    /// `gw_RB_GameHashForFrame`: the curated gameplay hash of the current state.
    fn game_hash(&self) -> u32;
    /// One logic frame: simulates `frame() + 1` with the session's inputs. `resim` is
    /// `gw_Snap_SessionResim` (side effects such as sound must be gated while it is set).
    fn run_frame(&mut self, inputs: &[Option<RbInput>; SLOTS], resim: bool);
    /// The game could not complete its last frame (native port only: a code path that is not ported
    /// yet was reached, typically on a mispredicted input). The frame counter did not move and the
    /// state is half-stepped, so the session must not snapshot or hash it; it waits for the
    /// confirmed inputs and rolls back.
    fn halted(&self) -> bool {
        false
    }
}

/// `MELEE_RB_INPUT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSource {
    /// The recorded PROCESSED inputs (`peek_input`).
    Replay = 0,
    /// The deterministic raw pad generator (`rb_gen`).
    PadGen = 1,
    /// Live pads through [`RbSession::pad_latch`].
    Live = 2,
}

/// rb_init's configuration (the environment in the C).
#[derive(Clone, Debug)]
pub struct RbConfig {
    pub fake_lat: i32,
    pub fake_jit: i32,
    pub fake_loss: i32,
    pub delay: i32,
    pub maxb: i32,
    pub seed: u32,
    pub remote_mask: u32,
    pub src: InputSource,
    pub gen_seed: u32,
    /// Netplay: a real peer delivers the remote port (gw_netplay.c); `run_tick`'s `net_tick`.
    pub net: bool,
    pub local_port: usize,
    /// MELEE_RB_HASHLOG: record every final curated hash in [`RbSession::hash_log`].
    pub hash_log: bool,
}

impl Default for RbConfig {
    fn default() -> Self {
        RbConfig {
            fake_lat: 0,
            fake_jit: 0,
            fake_loss: 0,
            delay: 2,
            maxb: 7,
            seed: 12345,
            remote_mask: 0x2,
            src: InputSource::Replay,
            gen_seed: 777,
            net: false,
            local_port: 0,
            hash_log: false,
        }
    }
}

fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (neg, d) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let mut v: i32 = 0;
    for c in d.bytes().take_while(|c| c.is_ascii_digit()) {
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i32);
    }
    if neg {
        v.wrapping_neg()
    } else {
        v
    }
}

impl RbConfig {
    /// rb_init from the environment: `None` when MELEE_RB_FAKE is unset or empty (session off).
    pub fn from_env() -> Option<RbConfig> {
        let get = |k: &str| std::env::var(k).ok();
        let v = get("MELEE_RB_FAKE").filter(|v| !v.is_empty())?;
        let mut c = RbConfig { fake_lat: atoi(&v), ..RbConfig::default() };
        if let Some(i) = v.find(',') {
            let j = &v[i + 1..];
            c.fake_jit = atoi(j);
            if let Some(k) = j.find(',') {
                c.fake_loss = atoi(&j[k + 1..]);
            }
        }
        if let Some(d) = get("MELEE_RB_DELAY") {
            c.delay = atoi(&d);
        }
        if let Some(m) = get("MELEE_RB_MAX") {
            c.maxb = atoi(&m);
        }
        if let Some(s) = get("MELEE_RB_SEED") {
            c.seed = atoi(&s) as u32;
        }
        if let Some(r) = get("MELEE_RB_REMOTE") {
            let t = r.trim().trim_start_matches("0x").trim_start_matches("0X");
            let n: String = t.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            c.remote_mask = u32::from_str_radix(&n, 16).unwrap_or(0);
        }
        c.src = match get("MELEE_RB_INPUT").and_then(|s| s.bytes().next()) {
            Some(b'p' | b'P') => InputSource::PadGen,
            Some(b'l' | b'L') => InputSource::Live,
            _ => InputSource::Replay,
        };
        if let Some(g) = get("MELEE_RB_PADSEED") {
            c.gen_seed = atoi(&g) as u32;
        }
        c.hash_log = get("MELEE_RB_HASHLOG").is_some();
        Some(c)
    }

    /// rb_init's clamps.
    fn clamped(mut self) -> Self {
        self.delay = self.delay.clamp(0, 8);
        self.maxb = self.maxb.clamp(1, 12);
        if self.net {
            self.src = InputSource::Live;
            self.remote_mask = 1 << (1 - self.local_port.min(1)) as u32;
            self.fake_lat = 0;
            self.fake_jit = 0;
            self.fake_loss = 0;
        }
        self
    }
}

#[derive(Clone, Copy, Default)]
struct Entry {
    frame: i32,
    input: RbInput,
}
const EMPTY: Entry = Entry {
    frame: i32::MIN,
    input: RbInput {
        lx: 0.0, ly: 0.0, cx: 0.0, cy: 0.0, trigger: 0.0, buttons: 0, seed: 0, raw: [0; 4],
        present: 0, confirmed: 0, is_raw: 0, pad_l: 0, pad_r: 0, pad_a: 0, pad_b: 0, pad_err: 0,
        physical_buttons: 0, physical_l: 0.0, physical_r: 0.0, physical_complete: 0,
    },
};

fn rix(frame: i32) -> usize {
    (frame as u32 % RING as u32) as usize // (unsigned) frame % RB_RING, as the C
}

/// rb_at with create = 0.
fn at(ring: &[Entry; RING], frame: i32) -> Option<&Entry> {
    let e = &ring[rix(frame)];
    (e.frame == frame).then_some(e)
}
fn at_mut(ring: &mut [Entry; RING], frame: i32) -> Option<&mut Entry> {
    let e = &mut ring[rix(frame)];
    (e.frame == frame).then_some(e)
}
/// rb_at with create = 1 (a new entry starts zeroed).
fn at_create(ring: &mut [Entry; RING], frame: i32) -> &mut Entry {
    let e = &mut ring[rix(frame)];
    if e.frame != frame {
        e.frame = frame;
        e.input = RbInput::default();
    }
    e
}

/// `rb_same`: the fields that make two inputs play differently (floats compared bitwise, as memcmp).
pub fn rb_same(a: &RbInput, b: &RbInput) -> bool {
    let fb = |i: &RbInput| [i.lx.to_bits(), i.ly.to_bits(), i.cx.to_bits(), i.cy.to_bits(), i.trigger.to_bits()];
    a.present == b.present
        && a.is_raw == b.is_raw
        && fb(a) == fb(b)
        && a.buttons == b.buttons
        && a.raw == b.raw
        && a.pad_l == b.pad_l
        && a.pad_r == b.pad_r
        && a.pad_a == b.pad_a
        && a.pad_b == b.pad_b
        && a.pad_err == b.pad_err
}

/// `rb_neutral_raw`.
pub fn rb_neutral_raw() -> RbInput {
    RbInput { present: 1, is_raw: 1, confirmed: 1, ..RbInput::default() }
}

/// `rb_hash`.
pub fn rb_hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(2654435761)
        ^ b.wrapping_add(0x9E3779B9).wrapping_mul(2246822519)
        ^ c.wrapping_add(0x85EBCA6B).wrapping_mul(3266489917);
    h ^= h >> 15;
    h = h.wrapping_mul(2246822519);
    h ^= h >> 13;
    h
}

/// `rb_gen`: the pad generator, a pure function of (seed, port, frame).
pub fn rb_gen(gen_seed: u32, slot: usize, frame: i32) -> RbInput {
    let port = (slot >> 1) as u32;
    let (mut sx, mut sy, mut cx, mut cy, l, mut r) = (0i32, 0i32, 0i32, 0i32, 0i32, 0i32);
    let mut btn: u32 = 0;
    let blk = (frame.wrapping_sub(FIRST) >> 3) as u32;
    let h = rb_hash(gen_seed.wrapping_add(port.wrapping_mul(7919)), blk, 0x51);
    let j = rb_hash(gen_seed.wrapping_add(port.wrapping_mul(104729)), frame as u32, 0x77);
    let mag = 40 + ((h >> 8) & 0x2F) as i32;
    let mut o = rb_neutral_raw();
    if frame < FIRST + 24 {
        return o;
    }
    match h & 7 {
        0 => {}
        1 => sx = -mag - 30,
        2 => sx = mag + 30,
        3 => {
            btn = 0x0400;
            sx = if (h >> 4) & 1 != 0 { mag } else { -mag };
        }
        4 => {
            btn = 0x0100;
            sx = if (h >> 4) & 1 != 0 { 30 } else { -30 };
        }
        5 => {
            btn = 0x0200;
            sy = if (h >> 4) & 1 != 0 { 70 } else { -70 };
            sx = if (h >> 5) & 1 != 0 { 60 } else { -60 };
        }
        6 => {
            btn = 0x0020;
            r = 200;
            sx = ((h >> 4) & 0x3F) as i32 - 32;
        }
        _ => {
            cx = if (h >> 4) & 1 != 0 { 85 } else { -85 };
            cy = ((h >> 5) & 0x1F) as i32 - 16;
        }
    }
    if (h >> 12) & 3 == 0 {
        btn |= 0x0100;
    }
    sx += (j & 7) as i32 - 3;
    sy += ((j >> 3) & 7) as i32 - 3;
    o.buttons = btn;
    o.raw = [sx as i8, sy as i8, cx as i8, cy as i8];
    o.pad_l = l as u8;
    o.pad_r = r as u8;
    o
}

/// `rb_arrival`: the fake network tick at which (slot, frame)'s remote input reaches us.
pub fn rb_arrival(c: &RbConfig, slot: usize, frame: i32) -> i64 {
    let mut sent = frame as i64 - FIRST as i64 - c.delay as i64;
    if sent < 0 {
        sent = 0;
    }
    let mut j = 0i64;
    if c.fake_jit > 0 {
        j = (rb_hash(c.seed, frame as u32, slot as u32) % (2 * c.fake_jit + 1) as u32) as i64 - c.fake_jit as i64;
    }
    let mut a = sent + c.fake_lat as i64 + j;
    if c.fake_loss > 0 && ((rb_hash(c.seed ^ 0xA5A5, frame as u32, slot as u32) % 100) as i32) < c.fake_loss {
        a += 2 * c.fake_lat as i64 + 3;
    }
    a.max(sent)
}

#[derive(Clone, Copy, Default)]
struct Latch {
    have: bool,
    bacc: u16,
    bnew: u16,
    sx: i8,
    sy: i8,
    cx: i8,
    cy: i8,
    err: i8,
    l: u8,
    r: u8,
    a: u8,
    b: u8,
    lpk: u8,
    rpk: u8,
    apk: u8,
    bpk: u8,
}

#[derive(Clone, Copy, Default, Debug)]
struct Plan {
    rollback: bool,
    first: i32,
    n: i32,
    i: i32,
    k: i32,
    new_frame: bool,
    active: bool,
}

/// Statistics since the session began (the counting half of the C's).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RbStats {
    pub rollbacks: i32,
    pub resim: i32,
    pub depth_sum: i32,
    pub depth_max: i32,
    pub stall_ticks: i32,
    pub desyncs: i32,
    pub late_deliveries: i32,
    pub ticks: i32,
    pub new: i32,
    pub held: i32,
    pub waits: i32,
}

/// The snapshot ring (gw_snap.c's session slots): `k + 2` slots, k = maxb + 1.
struct Snaps<T> {
    slots: Vec<(i32, Option<T>, u32)>,
}
impl<T: Clone> Snaps<T> {
    fn open(n: usize) -> Self {
        Snaps { slots: (0..n).map(|_| (i32::MIN, None, 0)).collect() }
    }
    fn ix(&self, frame: i32) -> usize {
        frame.rem_euclid(self.slots.len() as i32) as usize
    }
    fn save(&mut self, frame: i32, s: T, h: u32) {
        let i = self.ix(frame);
        self.slots[i] = (frame, Some(s), h);
    }
    fn get(&self, frame: i32) -> Option<&(i32, Option<T>, u32)> {
        if self.slots.is_empty() {
            return None;
        }
        let e = &self.slots[self.ix(frame)];
        (e.0 == frame).then_some(e)
    }
}

/// The session (the C's file-static `rb`), generic over the game's snapshot.
pub struct RbSession<G: RbGame> {
    pub cfg: RbConfig,
    pub on: bool,
    in_match: bool,
    epoch: i32,
    wait_ticks: i32,
    hring: [(i32, u32); RING],
    hlog_next: i32,
    /// The final curated hashes, in order (MELEE_RB_HASHLOG's `frame,hash` rows).
    pub hash_log: Vec<(i32, u32)>,
    early: Vec<(i32, usize, i32, RbInput)>,
    /// rb_early: remote inputs that reached the transport before the open.
    pre_open: Vec<(usize, i32, RbInput)>,
    latch: [Latch; 4],
    pend: Vec<(usize, i32, i64, RbInput)>,
    opened: bool,
    started: bool,
    tk: i64,
    newest: i32,
    prev_idle: bool,
    last: i32,
    slot_present: [bool; SLOTS],
    slot_remote: [bool; SLOTS],
    conf_slot: [i32; SLOTS],
    used: [[Entry; RING]; SLOTS],
    truth: [[Entry; RING]; SLOTS],
    first_wrong: i32,
    first_wrong_slot: usize,
    plan: Plan,
    /// The game's frame as of the last call that saw the game (gw_Replay_Frame for the
    /// network-facing functions, which run without the game).
    frame: i32,
    resim: bool,
    snaps: Snaps<G::Snapshot>,
    pub stats: RbStats,
    /// Early inputs (early.rs): use the peer's early pads as predictions.
    pub hints_on: bool,
    hint: [crate::early::Buffer; SLOTS],
    /// per slot and frame: the frame whose used input came from an early pad, and what repeat-last would have used
    hint_used: Vec<[(i32, bool, RbInput); RING]>,
    pub hint_trust: [crate::early::Trust; SLOTS],
    pub hint_stats: HintStats,
}

/// Early-input prediction statistics (this match).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct HintStats {
    /// early pads taken into the buffer
    pub received: u32,
    /// frame predictions made from an early pad (resimulations included)
    pub predicted: u32,
    /// judged on the confirmed input: right, and of those where repeat-last would have been wrong (helped)
    pub right: u32,
    pub wrong: u32,
    pub helped: u32,
    /// wrong where repeat-last would have been right
    pub hurt: u32,
    /// repeat-last predictions judged on the confirmed input (the baseline)
    pub plain_right: u32,
    pub plain_wrong: u32,
}

impl<G: RbGame> RbSession<G> {
    /// rb_init with `cfg` (the session is ON).
    pub fn new(cfg: RbConfig) -> Self {
        RbSession {
            cfg: cfg.clamped(),
            on: true,
            in_match: false,
            epoch: 0,
            wait_ticks: 0,
            hring: [(i32::MIN, 0); RING],
            hlog_next: FIRST,
            hash_log: Vec::new(),
            early: Vec::new(),
            pre_open: Vec::new(),
            latch: [Latch::default(); 4],
            pend: Vec::new(),
            opened: false,
            started: false,
            tk: 0,
            newest: i32::MIN,
            prev_idle: false,
            last: i32::MAX,
            slot_present: [false; SLOTS],
            slot_remote: [false; SLOTS],
            conf_slot: [FIRST - 1; SLOTS],
            used: [[EMPTY; RING]; SLOTS],
            truth: [[EMPTY; RING]; SLOTS],
            first_wrong: NONE,
            first_wrong_slot: 0,
            plan: Plan::default(),
            frame: FIRST - 2,
            resim: false,
            snaps: Snaps { slots: Vec::new() },
            stats: RbStats::default(),
            hints_on: false,
            hint: Default::default(),
            hint_used: vec![[(i32::MIN, false, RbInput::default()); RING]; SLOTS],
            hint_trust: [crate::early::Trust::default(); SLOTS],
            hint_stats: HintStats::default(),
        }
    }

    /// The local port is known only once the room's roles are (a random match decides them late).
    pub fn set_local_port(&mut self, port: usize) {
        self.cfg.local_port = port;
        self.cfg.remote_mask = 1 << (1 - port.min(1)) as u32;
    }

    /// gw_RB_SetDelay (netplay: the guest adopts the host's delay before frame -123).
    pub fn set_delay(&mut self, d: i32) {
        self.cfg.delay = d.clamp(0, 8);
    }

    /// gw_RB_SceneBegin: a session governs VS matches only (scene kind 2). A new match starts a
    /// fresh session state.
    pub fn scene_begin(&mut self, scene_kind: i32) {
        if self.on && self.cfg.net && scene_kind != 2 && self.in_match {
            self.on = false; // gw_Netplay_MatchOver: the app layer's
            self.in_match = false;
            self.pre_open.clear();
            return;
        }
        if !self.on {
            return;
        }
        self.epoch += 1;
        self.wait_ticks = 0;
        self.in_match = scene_kind == 2;
        if self.in_match {
            self.opened = false;
            self.started = false;
            self.tk = 0;
            self.newest = i32::MIN;
            self.prev_idle = false;
            self.first_wrong = NONE;
            self.plan = Plan::default();
            self.pend.clear();
            self.latch = [Latch::default(); 4];
            self.conf_slot = [FIRST - 1; SLOTS];
            self.hring = [(i32::MIN, 0); RING];
            self.hlog_next = FIRST;
        }
    }

    /// gw_rb_active.
    /// Debug line: planner state.
    pub fn dbg_line(&self) -> String {
        format!("first_wrong {} newest {} frame {} conf {} wait {} plan(rb {} first {} k {} new {})", self.first_wrong, self.newest, self.frame, self.conf(), self.wait_ticks, self.plan.rollback, self.plan.first, self.plan.k, self.plan.new_frame)
    }

    pub fn active(&self) -> bool {
        self.on && self.in_match && self.frame >= FIRST - 1
    }
    pub fn epoch(&self) -> i32 {
        self.epoch
    }
    pub fn rollbacks(&self) -> i32 {
        self.stats.rollbacks
    }
    pub fn desyncs(&self) -> i32 {
        self.stats.desyncs
    }

    fn src_peek(&self, g: &G, slot: usize, frame: i32) -> Option<RbInput> {
        if self.cfg.src == InputSource::PadGen {
            if slot & 1 != 0 || !self.slot_present[slot] {
                return None;
            }
            return Some(rb_gen(self.cfg.gen_seed, slot, frame));
        }
        g.peek_input(slot, frame)
    }

    /// rb_conf: the newest frame with every present remote slot confirmed; i32::MAX if none.
    fn conf(&self) -> i32 {
        let mut m = i32::MAX;
        for s in 0..SLOTS {
            if self.slot_present[s] && self.slot_remote[s] && self.conf_slot[s] < m {
                m = self.conf_slot[s];
            }
        }
        m
    }

    // ---- the network-facing interface -------------------------------------------------------

    /// gw_rb_submit_remote_input.
    pub fn submit_remote_input(&mut self, slot: usize, frame: i32, input: &RbInput) {
        if (!self.on || !self.opened) && slot < SLOTS && self.pre_open.len() < 256 {
            self.pre_open.push((slot, frame, *input));
            return;
        }
        if !self.on || slot >= SLOTS {
            return;
        }
        if self.opened && frame > self.frame + RING as i32 - 8 {
            return; // refused: further ahead than the ring holds
        }
        let t = at_create(&mut self.truth[slot], frame);
        t.input = *input;
        t.input.confirmed = 1;
        let tin = t.input;
        while at(&self.truth[slot], self.conf_slot[slot] + 1).is_some() {
            self.conf_slot[slot] += 1;
        }
        self.hint[slot].drop_through(self.conf_slot[slot]);
        if let Some(u) = at(&self.used[slot], frame) {
            let hu = &mut self.hint_used[slot][rix(frame)];
            if hu.0 == frame && hu.1 {
                hu.0 = i32::MIN;
                let right = rb_same(&u.input, &tin);
                let plain = rb_same(&hu.2, &tin);
                let st = &mut self.hint_stats;
                if right {
                    st.right += 1;
                    st.helped += !plain as u32;
                } else {
                    st.wrong += 1;
                    st.hurt += plain as u32;
                }
                if self.hint_trust[slot].judge(right) {
                    self.hint[slot].clear();
                }
            } else if hu.0 == frame {
                hu.0 = i32::MIN;
                if rb_same(&u.input, &tin) {
                    self.hint_stats.plain_right += 1;
                } else {
                    self.hint_stats.plain_wrong += 1;
                }
            }
        }
        if let Some(u) = at_mut(&mut self.used[slot], frame) {
            if !rb_same(&u.input, &tin) && frame < self.first_wrong {
                self.first_wrong = frame;
                self.first_wrong_slot = slot;
            }
            u.input.confirmed = 1;
        }
    }

    /// gw_rb_submit_local_input.
    pub fn submit_local_input(&mut self, slot: usize, frame: i32, input: &RbInput) {
        if !self.on || slot >= SLOTS {
            return;
        }
        let t = at_create(&mut self.truth[slot], frame);
        t.input = *input;
        t.input.confirmed = 1;
    }

    /// An early pad from the peer for remote `slot`, first appearing at `target` (early.rs). Ignored while off, before
    /// the open, for a local slot, for an already confirmed frame, or from an untrusted peer.
    pub fn submit_hint(&mut self, slot: usize, target: i32, pad: &crate::early::Pad) {
        if !self.hints_on || !self.opened || slot >= SLOTS || !self.slot_remote[slot] || self.hint_trust[slot].distrusted {
            return;
        }
        if target <= self.conf_slot[slot] || target > self.frame + RING as i32 - 8 {
            return;
        }
        self.hint_stats.received += 1;
        self.hint[slot].add(target, pad);
    }

    /// gw_rb_local_input_for_send.
    pub fn local_input_for_send(&self, slot: usize, frame: i32) -> Option<RbInput> {
        if !self.on || slot >= SLOTS {
            return None;
        }
        at(&self.truth[slot], frame).map(|e| e.input)
    }

    /// gw_rb_confirmed_frame.
    pub fn confirmed_frame(&self) -> i32 {
        let c = self.conf();
        if c == i32::MAX {
            self.frame
        } else {
            c
        }
    }
    /// gw_rb_current_frame.
    pub fn current_frame(&self) -> i32 {
        self.frame + 1
    }
    /// gw_rb_frame_advantage.
    pub fn frame_advantage(&self) -> i32 {
        self.frame - self.confirmed_frame()
    }

    fn hash_final(&self, frame: i32) -> bool {
        frame - 1 <= self.confirmed_frame()
            && frame <= self.frame
            && !(self.first_wrong != NONE && self.first_wrong < frame)
    }

    /// gw_rb_checksum: the curated hash at the START of `frame`, once final; else 0.
    pub fn checksum(&self, frame: i32) -> u32 {
        let e = self.hring[rix(frame)];
        if !self.on || !self.hash_final(frame) || e.0 != frame {
            return 0;
        }
        e.1
    }

    /// gw_rb_checksum_full: the whole-state hash of the snapshot of `frame`.
    pub fn checksum_full(&self, frame: i32) -> u32 {
        if frame > self.confirmed_frame() {
            return 0;
        }
        self.snaps.get(frame).map_or(0, |e| e.2)
    }

    /// gw_rb_checksum_e.
    pub fn checksum_e(&self, epoch: i32, frame: i32) -> u32 {
        if epoch == self.epoch {
            self.checksum(frame)
        } else {
            0
        }
    }

    /// gw_rb_submit_remote_input_e.
    pub fn submit_remote_input_e(&mut self, epoch: i32, slot: usize, frame: i32, input: &RbInput) {
        if epoch == self.epoch {
            self.submit_remote_input(slot, frame, input);
        } else if epoch > self.epoch && self.early.len() < 256 {
            self.early.push((epoch, slot, frame, *input));
        }
    }

    /// gw_rb_request_wait.
    pub fn request_wait(&mut self, frames: i32) {
        if frames > 0 {
            self.wait_ticks += frames;
        }
    }

    /// gw_RB_InputFor: the PROCESSED entry (replay-style) for (port, follower) at `frame`.
    pub fn input_for(&self, port: usize, follower: bool, frame: i32) -> Option<&RbInput> {
        if !self.on || port > 3 {
            return None;
        }
        at(&self.used[port * 2 + follower as usize], frame)
            .map(|e| &e.input)
            .filter(|i| i.present != 0 && i.is_raw == 0)
    }

    /// gw_RB_InputAny: the leader's entry, raw-controller ones included.
    pub fn input_any(&self, port: usize, frame: i32) -> Option<&RbInput> {
        if !self.on || port > 3 {
            return None;
        }
        at(&self.used[port * 2], frame).map(|e| &e.input).filter(|i| i.present != 0)
    }

    /// gw_RB_PadGoverned.
    pub fn pad_governed(&self) -> bool {
        self.on && self.in_match && self.opened && self.cfg.src != InputSource::Replay && self.frame >= FIRST - 1
    }

    /// gw_RB_PadField (which: 0 button, 1-4 sticks, 5-6 triggers, 7-8 analog A/B, else err).
    pub fn pad_field(&self, port: usize, which: i32) -> i32 {
        if port > 3 {
            return 0;
        }
        let Some(e) = at(&self.used[port * 2], self.frame + 1).filter(|e| e.input.present != 0) else {
            return 0;
        };
        let i = &e.input;
        match which {
            0 => (i.buttons & 0xFFFF) as i32,
            1..=4 => i.raw[(which - 1) as usize] as i32,
            5 => i.pad_l as i32,
            6 => i.pad_r as i32,
            7 => i.pad_a as i32,
            8 => i.pad_b as i32,
            _ => i.pad_err as i32,
        }
    }

    // ---- live pads ---------------------------------------------------------------------------

    /// gw_RB_PadLatch: one poll of one port (MELEE_RB_INPUT=live only).
    #[allow(clippy::too_many_arguments)]
    pub fn pad_latch(&mut self, mut port: usize, button: i32, sx: i32, sy: i32, cx: i32, cy: i32, l: i32, r: i32, a: i32, b: i32, err: i32) {
        if !self.on || self.cfg.src != InputSource::Live || port > 3 {
            return;
        }
        if self.cfg.net {
            if port != 0 {
                return;
            }
            port = self.cfg.local_port;
        }
        let lt = &mut self.latch[port];
        let (l, r, a, b) = (l as u8, r as u8, a as u8, b as u8);
        if !lt.have {
            lt.bacc = button as u16;
            lt.lpk = l;
            lt.rpk = r;
            lt.apk = a;
            lt.bpk = b;
        } else {
            lt.bacc |= button as u16;
            lt.lpk = lt.lpk.max(l);
            lt.rpk = lt.rpk.max(r);
            lt.apk = lt.apk.max(a);
            lt.bpk = lt.bpk.max(b);
        }
        lt.have = true;
        lt.bnew = button as u16;
        lt.sx = sx as i8;
        lt.sy = sy as i8;
        lt.cx = cx as i8;
        lt.cy = cy as i8;
        lt.l = l;
        lt.r = r;
        lt.a = a;
        lt.b = b;
        lt.err = err as i8;
    }

    /// rb_live_sample: this frame's live sample, for `next + D`.
    fn live_sample(&mut self, next: i32) {
        let ff = next + self.cfg.delay;
        for s in (0..SLOTS).step_by(2) {
            let p = s >> 1;
            if !self.slot_present[s] || at(&self.truth[s], ff).is_some() {
                continue;
            }
            if self.cfg.net && self.slot_remote[s] {
                continue;
            }
            if self.pend.iter().any(|e| e.0 == s && e.1 == ff) {
                continue;
            }
            let mut inp = rb_neutral_raw();
            let lt = &mut self.latch[p];
            if lt.have {
                inp.buttons = lt.bacc as u32;
                inp.raw = [lt.sx, lt.sy, lt.cx, lt.cy];
                inp.pad_l = lt.lpk;
                inp.pad_r = lt.rpk;
                inp.pad_a = lt.apk;
                inp.pad_b = lt.bpk;
                inp.pad_err = lt.err;
                lt.bacc = lt.bnew;
                lt.lpk = lt.l;
                lt.rpk = lt.r;
                lt.apk = lt.a;
                lt.bpk = lt.b;
            }
            if !self.slot_remote[s] {
                at_create(&mut self.truth[s], ff).input = inp;
            } else if self.pend.len() < 128 {
                let c = &self.cfg;
                let mut j = 0i64;
                if c.fake_jit > 0 {
                    j = (rb_hash(c.seed, ff as u32, s as u32) % (2 * c.fake_jit + 1) as u32) as i64 - c.fake_jit as i64;
                }
                let mut arr = self.tk + c.fake_lat as i64 + j;
                if c.fake_loss > 0 && ((rb_hash(c.seed ^ 0xA5A5, ff as u32, s as u32) % 100) as i32) < c.fake_loss {
                    arr += 2 * c.fake_lat as i64 + 3;
                }
                if arr < self.tk {
                    arr = self.tk;
                }
                self.pend.push((s, ff, arr, inp));
            }
        }
    }

    // ---- the fake network --------------------------------------------------------------------

    fn arrival(&self, slot: usize, frame: i32) -> i64 {
        rb_arrival(&self.cfg, slot, frame)
    }

    /// rb_fake_deliver.
    fn fake_deliver(&mut self, g: &G) {
        if self.cfg.src == InputSource::Live {
            // swap-remove order, as the C
            let mut i = 0;
            while i < self.pend.len() {
                if self.pend[i].2 <= self.tk {
                    let (sl, fr, _, inp) = self.pend.swap_remove(i);
                    self.submit_remote_input(sl, fr, &inp);
                } else {
                    i += 1;
                }
            }
            return;
        }
        for s in 0..SLOTS {
            if !self.slot_present[s] || !self.slot_remote[s] {
                continue;
            }
            let mut top = (self.tk + FIRST as i64 + self.cfg.delay as i64).min(i32::MAX as i64) as i32;
            if top > self.last {
                top = self.last;
            }
            let cap = self.frame + 1 + self.cfg.maxb + self.cfg.delay;
            if top > cap {
                top = cap;
            }
            let mut f = self.conf_slot[s] + 1;
            while f <= top {
                if at(&self.truth[s], f).is_none() && self.arrival(s, f) <= self.tk {
                    let inp = self.src_peek(g, s, f).unwrap_or_default();
                    if f < self.frame + 1 && self.started {
                        self.stats.late_deliveries += 1;
                    }
                    self.submit_remote_input(s, f, &inp);
                }
                f += 1;
            }
        }
    }

    // ---- the per-tick plan --------------------------------------------------------------------

    /// gw_RB_Iterations: how many logic iterations to run this render tick. `net_tick` is
    /// gw_Netplay_Tick (netplay mode: receive, deliver remote inputs, send ours).
    pub fn iterations(&mut self, g: &G, count: i32, net_tick: Option<&mut dyn FnMut(&mut Self)>) -> i32 {
        self.frame = g.frame();
        if !self.on || !self.in_match {
            return count;
        }
        let frame = self.frame;
        if frame < FIRST - 1 {
            return count;
        }
        if !self.opened {
            self.open(g);
        }
        if self.cfg.hash_log {
            let mut c = self.conf();
            if c == i32::MAX || c > frame {
                c = frame;
            }
            let mut f = self.hlog_next;
            while f <= c + 1 && f <= frame {
                if !self.hash_final(f) {
                    break;
                }
                let h = self.checksum(f);
                if h != 0 {
                    self.hash_log.push((f, h));
                }
                f += 1;
            }
            self.hlog_next = f;
        }
        if !self.started {
            if frame >= FIRST {
                self.started = true;
                self.tk = 1;
            }
        } else if frame > self.newest || self.prev_idle {
            self.tk += 1;
        } else {
            self.stats.held += 1;
        }
        if frame > self.newest {
            self.newest = frame;
        }
        if self.cfg.net {
            if let Some(t) = net_tick {
                t(self);
            }
        } else {
            self.fake_deliver(g);
        }

        let n = frame + 1;
        let mut conf = self.conf();
        if conf == i32::MAX {
            conf = frame;
        }
        self.plan = Plan { n, new_frame: true, ..Plan::default() };
        if self.first_wrong != NONE {
            let f = self.first_wrong;
            self.first_wrong = NONE;
            if f <= frame {
                if self.snaps.get(f).is_none() {
                    self.stats.desyncs += 1; // the correction's snapshot is gone
                } else {
                    self.plan.rollback = true;
                    self.plan.first = f;
                    self.plan.k = n - f;
                }
            }
        }
        if frame >= FIRST && (n - conf > self.cfg.maxb || (n > self.last && conf < self.last)) {
            self.plan.new_frame = false;
            self.stats.stall_ticks += 1;
        } else if self.wait_ticks > 0 && frame >= FIRST {
            self.wait_ticks -= 1;
            self.plan.new_frame = false;
            self.stats.waits += 1;
        }
        self.plan.active = self.plan.rollback || self.plan.new_frame;
        self.prev_idle = frame >= FIRST && !self.plan.new_frame;
        if self.plan.rollback {
            let k = self.plan.k;
            self.stats.rollbacks += 1;
            self.stats.resim += k;
            self.stats.depth_sum += k;
            self.stats.depth_max = self.stats.depth_max.max(k);
        }
        self.stats.ticks += 1;
        (if self.plan.rollback { self.plan.k } else { 0 }) + self.plan.new_frame as i32
    }

    /// The open (the first logic tick of the match): rings, slots, early inputs.
    fn open(&mut self, g: &G) {
        self.opened = true;
        for b in self.hint.iter_mut() {
            b.clear();
        }
        for u in self.hint_used.iter_mut() {
            *u = [(i32::MIN, false, RbInput::default()); RING];
        }
        self.last = g.last_frame();
        self.snaps = Snaps::open((self.cfg.maxb + 3) as usize);
        let src = self.cfg.src;
        for s in 0..SLOTS {
            let p = s >> 1;
            self.slot_present[s] = if g.live() {
                s & 1 == 0 && g.port_human(p)
            } else {
                g.peek_input(s, FIRST).is_some() && (src == InputSource::Replay || s & 1 == 0)
            };
            self.slot_remote[s] = (self.cfg.remote_mask >> p) & 1 != 0;
            self.conf_slot[s] = FIRST - 1;
            self.used[s] = [EMPTY; RING];
            self.truth[s] = [EMPTY; RING];
        }
        for (sl, fr, inp) in std::mem::take(&mut self.pre_open) {
            self.submit_remote_input(sl, fr, &inp);
        }
        if src == InputSource::Live && self.cfg.delay > 0 {
            for s in 0..SLOTS {
                if !self.slot_present[s] {
                    continue;
                }
                for f in FIRST..FIRST + self.cfg.delay {
                    at_create(&mut self.truth[s], f).input = rb_neutral_raw();
                }
                if self.slot_remote[s] {
                    self.conf_slot[s] = FIRST + self.cfg.delay - 1;
                }
            }
        }
        let mut q = 0;
        while q < self.early.len() {
            let (ep, sl, fr, inp) = self.early[q];
            if ep == self.epoch {
                self.early.swap_remove(q);
                self.submit_remote_input(sl, fr, &inp);
            } else if ep < self.epoch {
                self.early.swap_remove(q);
            } else {
                q += 1;
            }
        }
    }

    /// rb_prepare: fix the inputs frame `next` uses.
    fn prepare(&mut self, g: &G, next: i32) {
        for s in 0..SLOTS {
            if !self.slot_present[s] {
                continue;
            }
            let inp = if let Some(t) = at(&self.truth[s], next) {
                t.input
            } else if !self.slot_remote[s] {
                let mut i;
                match (self.cfg.src != InputSource::Live).then(|| self.src_peek(g, s, next)).flatten() {
                    Some(p) => {
                        i = p;
                        let nt = at_create(&mut self.truth[s], next);
                        nt.input = p;
                        nt.input.confirmed = 1;
                    }
                    None if self.cfg.src != InputSource::Replay => i = rb_neutral_raw(),
                    None => i = RbInput::default(),
                }
                i.confirmed = 1;
                i
            } else {
                let c = if self.conf_slot[s] >= FIRST { at(&self.truth[s], self.conf_slot[s]) } else { None };
                let mut i = match c {
                    Some(c) => c.input,
                    None => RbInput { present: 1, is_raw: (self.cfg.src != InputSource::Replay) as u8, ..RbInput::default() },
                };
                i.confirmed = 0;
                i.seed = 0;
                // early inputs: the newest early pad for a frame after the last confirmed one, up to this one
                let early = if self.hints_on && !self.hint_trust[s].distrusted { self.hint[s].best(next, self.conf_slot[s]).copied() } else { None };
                let k = rix(next);
                match early {
                    Some(p) => {
                        let plain = i;
                        let mut e = crate::pad::np_decode(&p);
                        e.confirmed = 0;
                        self.hint_used[s][k] = (next, true, plain);
                        self.hint_stats.predicted += 1;
                        i = e;
                    }
                    None => self.hint_used[s][k] = (next, false, i),
                }
                i
            };
            at_create(&mut self.used[s], next).input = inp;
        }
    }

    /// gw_RB_IterStart: at the top of every logic iteration. Returns whether this iteration is a
    /// resimulation (None: the session does not govern it).
    pub fn iter_start(&mut self, g: &mut G) -> Option<bool> {
        if !self.on || !self.in_match || !self.opened || !self.plan.active {
            return None;
        }
        let first_of_rollback = self.plan.i == 0 && self.plan.rollback;
        if first_of_rollback {
            let snap = self.snaps.get(self.plan.first).and_then(|e| e.1.clone()).expect("snapshot checked at planning");
            g.load(&snap);
        } else if self.plan.i == 0 && g.halted() {
            // The previous tick's attempt at the next frame died half-stepped (native port only):
            // restart it from the clean state saved before that attempt.
            let nxt = g.frame() + 1;
            if let Some(snap) = self.snaps.get(nxt).and_then(|e| e.1.clone()) {
                g.load(&snap);
            }
        }
        self.frame = g.frame();
        let next = self.frame + 1;
        let resim = self.plan.rollback && next < self.plan.n;
        if !resim && self.cfg.src == InputSource::Live {
            self.live_sample(next);
        }
        self.prepare(g, next);
        let halted = g.halted();
        if !halted {
            self.hring[rix(next)] = (next, g.game_hash() | 1);
        }
        if !first_of_rollback && !halted {
            let h = g.state_hash();
            self.snaps.save(next, g.save(), h);
            if !resim {
                self.stats.new += 1;
            }
        }
        self.resim = resim;
        self.plan.i += 1;
        Some(resim)
    }

    /// The inputs of the frame about to run, per slot (what the game's pad read sees).
    pub fn frame_inputs(&self) -> [Option<RbInput>; SLOTS] {
        let mut a = [None; SLOTS];
        for (s, o) in a.iter_mut().enumerate() {
            *o = at(&self.used[s], self.frame + 1).map(|e| e.input).filter(|i| i.present != 0);
        }
        a
    }

    /// After the game's frame: the frame counter moved.
    pub fn iter_end(&mut self, g: &G) {
        self.frame = g.frame();
    }

    pub fn is_resimulating(&self) -> bool {
        self.resim
    }
}

/// One render tick of the scene loop with the session (gmscene gm_801A4D34): plan, then run the
/// iterations. Returns the number of logic frames run (0 = stalled or waiting).
pub fn run_tick<G: RbGame>(s: &mut RbSession<G>, g: &mut G, net_tick: Option<&mut dyn FnMut(&mut RbSession<G>)>) -> i32 {
    let n = s.iterations(g, 1, net_tick);
    for _ in 0..n {
        let resim = s.iter_start(g).unwrap_or(false);
        let inputs = s.frame_inputs();
        g.run_frame(&inputs, resim);
        s.iter_end(g);
    }
    n
}

impl<G: RbGame> RollbackSession for RbSession<G> {
    fn submit_remote_input(&mut self, slot: usize, frame: i32, input: &RbInput) {
        RbSession::submit_remote_input(self, slot, frame, input)
    }
    fn local_input_for_send(&self, slot: usize, frame: i32) -> Option<RbInput> {
        RbSession::local_input_for_send(self, slot, frame)
    }
    fn checksum(&self, frame: i32) -> u32 {
        RbSession::checksum(self, frame)
    }
    fn current_frame(&self) -> i32 {
        RbSession::current_frame(self)
    }
    fn confirmed_frame(&self) -> i32 {
        RbSession::confirmed_frame(self)
    }
    fn request_wait(&mut self, frames: i32) {
        RbSession::request_wait(self, frames)
    }
}

// ---- SyncTest (gw_snap.c gw_SyncTest_Iterations / IterStart) ---------------------------------

/// GGPO's SyncTest over a game: every render tick saves S[F], loads S[F-k], resimulates F-k..F-1
/// with the same inputs and compares the whole-state hash at the start of every resimulated frame
/// with the first pass's. The curated hash is compared too.
pub struct SyncTest<G: RbGame> {
    pub k: i32,
    snaps: Snaps<G::Snapshot>,
    first_pass: Vec<(i32, u32, u32, [Option<RbInput>; SLOTS])>,
    pub passes: u32,
    pub checked: u32,
    /// (frame, first pass full hash, resimulated full hash, curated first, curated resim).
    pub mismatches: Vec<(i32, u32, u32, u32, u32)>,
}

impl<G: RbGame> SyncTest<G> {
    pub fn new(k: i32) -> Self {
        let k = k.max(1);
        SyncTest { k, snaps: Snaps::open(k as usize + 2), first_pass: Vec::new(), passes: 0, checked: 0, mismatches: Vec::new() }
    }

    fn record(&mut self, frame: i32, full: u32, cur: u32, inputs: [Option<RbInput>; SLOTS]) {
        self.first_pass.retain(|e| e.0 > frame - 2 * self.k - 4 && e.0 != frame);
        self.first_pass.push((frame, full, cur, inputs));
    }

    /// One tick: the new frame F (inputs from `input`), then roll back k and resimulate.
    pub fn tick(&mut self, g: &mut G, input: &mut dyn FnMut(i32) -> [Option<RbInput>; SLOTS]) {
        let f = g.frame() + 1;
        let full = g.state_hash();
        let cur = g.game_hash();
        let inputs = input(f);
        self.snaps.save(f, g.save(), full);
        self.record(f, full, cur, inputs);
        g.run_frame(&inputs, false);
        // roll back k from F+1 and resimulate up to it, comparing at the start of every frame
        let target = f + 1;
        let back = target - self.k;
        let Some(snap) = self.snaps.get(back).and_then(|e| e.1.clone()) else {
            return;
        };
        let (end_full, end_cur) = (g.state_hash(), g.game_hash());
        g.load(&snap);
        let mut fr = back;
        while fr < target {
            let (full1, cur1, inp) = {
                let e = self.first_pass.iter().find(|e| e.0 == fr).expect("first pass recorded");
                (e.1, e.2, e.3)
            };
            if fr > back {
                let (full2, cur2) = (g.state_hash(), g.game_hash());
                self.checked += 1;
                if full1 != full2 || cur1 != cur2 {
                    self.mismatches.push((fr, full1, full2, cur1, cur2));
                }
            }
            g.run_frame(&inp, true);
            fr += 1;
        }
        let (full2, cur2) = (g.state_hash(), g.game_hash());
        self.checked += 1;
        if end_full != full2 || end_cur != cur2 {
            self.mismatches.push((target, end_full, full2, end_cur, cur2));
        }
        self.passes += 1;
    }
}
