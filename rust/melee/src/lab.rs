//! The Lab's sim side (gd-lab-v1, docs/rust-port/notes/wasm-ui-lab-api.md part B and its 2-player amendment): what the
//! `gd_lab` commands do to the match. The presentation side (melee_modhost::lab) only queues commands and reads what
//! this publishes.
//!
//! A [`LabRun`] owns the match engine for the lab match (as the synctest's `ModesGame` does: the snapshot base is taken
//! after the scene's first frames). Once per command-clock frame ([`LabRun::tick`], the lab frame offline, the lockstep
//! frame online) it:
//! 1. applies that frame's commands in order (online: the host's, then the guest's, on both peers);
//! 2. unless paused (or with a step pending), picks each port's pad: the logged pad while replaying after a rewind,
//!    else a playback (B2), else the dummy synthesiser (B3), else the pad the input boundary produced; records it;
//! 3. steps the engine, captures the sparse rewind ring (one snapshot every [`RING_EVERY`] frames), and publishes the
//!    fighters, boxes, events and frame-meter cells for the queries.
//!
//! Everything here is a function of the commands, the pads and the sim, so two peers with the same command stream
//! stay identical: random choices draw from the lab RNG (GAP-7), a xorshift seeded from the agreed seed or an
//! `OP_RNG_SEED` command, never from wall time and never from the game RNG. The run is a field of the play flow, on
//! the system heap (never in the arena): a savestate load or rewind rewinds the match, not the Lab.

use crate::play_net::{ModesGame, ModesSnap};
use melee_modhost::lab::*;
use melee_net::session::RbGame;
use melee_types::PADStatus;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

/// The ring holds one snapshot every RING_EVERY frames; a rewind re-sims the rest from the logged pads (at most 7
/// frames, ~2 ms: the depth stays the full ring, the capture cost per frame halves).
pub const RING_EVERY: i32 = 8;

const BTN_A: u16 = 0x100;
const BTN_B: u16 = 0x200;
const BTN_X: u16 = 0x400;
const BTN_L: u16 = 0x40;
const BTN_R: u16 = 0x20;
const BTN_START: u16 = 0x1000;

fn is_idle(m: i32) -> bool {
    matches!(m, 14..=23 | 25..=34 | 39..=42 | 179)
}
fn is_stun_motion(m: i32) -> bool {
    matches!(m, 75..=91 | 181 | 183..=198 | 205 | 239..=243)
}
fn is_aerial(m: i32) -> bool {
    (65..=69).contains(&m)
}
fn is_landing_air(m: i32) -> bool {
    (70..=74).contains(&m)
}
fn is_cliff(m: i32) -> bool {
    (252..=263).contains(&m)
}

unsafe fn fighter(slot: i32) -> Option<&'static mut melee_types::Fighter> {
    let g = melee_fighter::player::get_entity(slot);
    if g.is_null() || (*g).user_data.is_null() {
        return None;
    }
    Some(&mut *((*g).user_data as *mut melee_types::Fighter))
}

#[derive(Clone)]
pub struct Rec {
    pub mask: u8,
    pub frames: Vec<[PADStatus; 4]>,
    pub start: Option<(i32, ModesSnap)>,
    pub open: bool,
}

#[derive(Clone, Default)]
struct PlaySet {
    ids: Vec<u8>,
    w: Vec<u8>,
    order: u32,
    cur: usize,
}

#[derive(Clone)]
struct Play {
    rec: u8,
    flags: u32,
    at: usize,
    set: Option<PlaySet>,
    /// A reaction's one-shot playback (ends instead of looping).
    once: bool,
}

#[derive(Clone, Copy, Default)]
struct React {
    option: i32,
    p: ReactParam,
    weights: u32,
}

#[derive(Clone, Copy, Default)]
struct DummyState {
    since_l: i32,
    oos: Option<(i32, i32)>,
    di: i32,
    sdi: i32,
    tech: i32,
    block: bool,
    opp_motion: i32,
    wait_since: i32,
    hit_at: i32,
    nair: bool,
}

#[derive(Clone, Copy, Default)]
struct Dummy {
    beh: i32,
    param: i32,
    react: [React; RK_COUNT],
    st: DummyState,
}

/// Per-port observation for the events, the frame meter and the observed frame data.
#[derive(Clone, Default)]
struct Obs {
    present: bool,
    motion: i32,
    motion_frames: i32,
    had_active: bool,
    shield_frames: i32,
    js_end: Option<i32>,
    airdodge: Option<(i32, i32, i32)>, // (frame, angle, after jumpsquat)
    left_ledge: Option<i32>,
    ledge_dodged: bool,
    pos: (f32, f32),
    hitlag: f32,
    percent: f32,
    iasa: bool,
    airborne: bool,
    ground_y: f32,
    actionable: bool,
    act_at: i32,
    timeline: Vec<u8>,
    tl_first: i32,
    tl_last: i32,
    tl_iasa: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Perf {
    pub ring_saves: u64,
    pub ring_ns: u64,
    pub slot_saves: u64,
    pub slot_ns: u64,
    pub loads: u64,
    pub load_ns: u64,
    pub rewinds: u64,
    pub rewind_ns: u64,
    pub resim_frames: u64,
    pub steps: u64,
    pub step_ns: u64,
    pub ring_bytes: usize,
    pub ring_peak_bytes: usize,
    /// Bytes newly copied by ring captures (the rest is shared with the previous capture).
    pub ring_fresh: u64,
}

pub struct LabRun {
    pub g: ModesGame,
    pub sh: Rc<RefCell<LabShared>>,
    /// Match frames since the lab took the engine (0 = the state it took).
    pub f: i32,
    slots: Vec<Option<(i32, ModesSnap)>>,
    ring_frames: u32,
    ring: VecDeque<(i32, ModesSnap)>,
    log: VecDeque<[PADStatus; 4]>,
    /// Frame of log[0].
    log_f0: i32,
    replay_until: Option<i32>,
    pub paused: bool,
    steps: u32,
    recs: BTreeMap<u8, Rec>,
    play: [Option<Play>; 4],
    dummy: [Dummy; 4],
    rng: u32,
    obs: [Obs; 4],
    exchange: Option<i32>,
    events: Vec<LabEvent>,
    pub perf: Perf,
    /// The last frame's pads as stepped (the input display's source and the tests').
    pub last_pads: [PADStatus; 4],
    pub drew: bool,
}

fn xorshift(s: &mut u32) -> u32 {
    let mut x = *s;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *s = if x == 0 { 0x9E37_79B9 } else { x };
    *s
}

fn mirror(p: &mut PADStatus) {
    p.stickX = p.stickX.saturating_neg();
    p.substickX = p.substickX.saturating_neg();
}

impl LabRun {
    pub fn new(g: ModesGame, sh: Rc<RefCell<LabShared>>, seed: u32) -> LabRun {
        let mut r = LabRun {
            g, sh, f: 0, slots: vec![None; SLOTS], ring_frames: 0, ring: VecDeque::new(), log: VecDeque::new(), log_f0: 0, replay_until: None,
            paused: false, steps: 0, recs: BTreeMap::new(), play: Default::default(), dummy: [Dummy::default(); 4], rng: mix32(seed) | 1, obs: Default::default(),
            exchange: None, events: Vec::new(), perf: Perf::default(), last_pads: [PADStatus::zeroed(); 4], drew: false,
        };
        r.sh.borrow_mut().active = true;
        // the match's timer (as ifTime_CreateTimers decides whether to show it); read once, the rules are fixed
        r.sh.borrow_mut().timed = unsafe {
            let rules = melee_world::gm::gm_GetStartMeleeRules();
            !rules.is_null() && ((*rules).timer_enabled() != 0 || (*rules).x1_0() != 0)
        };
        // the snapshot base now (as the rollback session takes it at its first frame): a base taken lazily at a later
        // savestate would be captured inside the frame's own allocations
        let t = std::time::Instant::now();
        drop(r.g.save());
        eprintln!("lab: snapshot base captured in {:.1} ms", t.elapsed().as_secs_f64() * 1e3);
        r.observe(false);
        r.publish();
        r
    }

    /// Give the engine back (the match ended or the session closes).
    pub fn finish(self) -> ModesGame {
        let mut s = self.sh.borrow_mut();
        s.active = false;
        s.fighters = [None; 4];
        drop(s);
        self.g
    }

    fn rnd(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { xorshift(&mut self.rng) % n }
    }

    /// A weighted draw over options 1..=8 (4-bit weights); uniform over `uniform` when the table is all zero.
    fn draw(&mut self, weights: u32, uniform: &[i32]) -> i32 {
        let tot: u32 = (0..8).map(|i| weights >> (i * 4) & 0xF).sum();
        if tot == 0 {
            return uniform[self.rnd(uniform.len() as u32) as usize];
        }
        let mut k = self.rnd(tot);
        for i in 0..8 {
            let w = weights >> (i * 4) & 0xF;
            if k < w {
                return i as i32 + 1;
            }
            k -= w;
        }
        uniform[0]
    }

    fn save_state(&self) -> ModesSnap {
        self.g.save()
    }

    fn load_state(&mut self, s: &ModesSnap) {
        self.g.load(s);
    }

    /// One engine frame with these pads (a resimulated one is not drawn).
    fn step_engine(&mut self, pads: [PADStatus; 4], resim: bool) {
        melee_engine::gxbridge::shim().set_suppress_draws(resim);
        unsafe { melee_runtime::tu::pobj::ext::gx_suppress_draws = resim as i32 };
        let t = std::time::Instant::now();
        self.g.step(pads);
        if !resim {
            self.perf.steps += 1;
            self.perf.step_ns += t.elapsed().as_nanos() as u64;
        }
        melee_engine::gxbridge::shim().set_suppress_draws(false);
        unsafe { melee_runtime::tu::pobj::ext::gx_suppress_draws = 0 };
    }

    // ------------------------------------------------------------------------------------------------ commands

    pub fn apply(&mut self, c: &LabCmd) {
        let port = (c.port & 3) as usize;
        match c.op {
            OP_SAVE => {
                let t = std::time::Instant::now();
                let s = self.save_state();
                self.perf.slot_saves += 1;
                self.perf.slot_ns += t.elapsed().as_nanos() as u64;
                if let Some(slot) = self.slots.get_mut(c.a as usize) {
                    *slot = Some((self.f, s));
                }
                self.note(format!("lab: frame {}: saved slot {}", self.f, c.a));
            }
            OP_LOAD => {
                if let Some(Some((fr, s))) = self.slots.get(c.a as usize).cloned() {
                    let t = std::time::Instant::now();
                    self.load_state(&s);
                    self.perf.loads += 1;
                    self.perf.load_ns += t.elapsed().as_nanos() as u64;
                    // a load forks the timeline: the forward history goes
                    self.f = fr;
                    self.ring.clear();
                    self.log.clear();
                    self.log_f0 = fr;
                    self.replay_until = None;
                    self.exchange = None;
                    self.obs = Default::default();
                    self.observe(false);
                    self.note(format!("lab: loaded slot {} (frame {fr})", c.a));
                }
            }
            OP_RING => {
                self.ring_frames = (c.a as u32).min(RING_MAX);
                if self.ring_frames == 0 {
                    self.ring.clear();
                    self.log.clear();
                    self.log_f0 = self.f;
                } else {
                    self.trim_ring();
                }
            }
            OP_REWIND => {
                let f = self.f;
                self.rewind(c.b as i32);
                self.note(format!("lab: frame {f}: rewound {} frames to {}", f - self.f, self.f));
            }
            OP_PAUSE => {
                self.paused = c.a != 0;
                self.steps = 0;
            }
            OP_STEP => self.steps = self.steps.saturating_add(c.b),
            OP_GO_LIVE => {
                self.note(format!("lab: frame {}: live", self.f));
                // take over: the logged future is dropped, live input from here
                if self.replay_until.take().is_some() {
                    let keep = (self.f - self.log_f0).max(0) as usize;
                    self.log.truncate(keep);
                    let f = self.f;
                    self.ring.retain(|r| r.0 <= f);
                }
            }
            OP_REC_START => {
                let st = (self.f, self.save_state());
                self.recs.insert(c.b as u8, Rec { mask: (c.a & 0xF) as u8, frames: Vec::new(), start: Some(st), open: true });
                self.note(format!("lab: frame {}: recording {:#x} of ports {:#x}", self.f, c.b, c.a));
            }
            OP_REC_STOP => {
                if let Some(r) = self.recs.get_mut(&(c.b as u8)) {
                    r.open = false;
                    eprintln!("lab: frame {}: recording {:#x} stopped, {} frames", self.f, c.b, r.frames.len());
                }
            }
            OP_REC_FREE => {
                self.recs.remove(&(c.b as u8));
                for p in self.play.iter_mut() {
                    if p.as_ref().is_some_and(|p| p.rec == c.b as u8) {
                        *p = None;
                    }
                }
            }
            OP_PLAY => {
                let rec = c.b as u8;
                if self.recs.contains_key(&rec) {
                    self.note(format!("lab: frame {}: playing {rec:#x} on ports {:#x} (flags {:#x})", self.f, c.a, c.c));
                    if c.c & PLAY_FROM_SLOT != 0 {
                        self.load_rec_start(rec);
                    }
                    for p in 0..4 {
                        if c.a >> p & 1 != 0 {
                            self.play[p] = Some(Play { rec, flags: c.c, at: 0, set: None, once: false });
                        }
                    }
                }
            }
            OP_PLAY_STOP => {
                for p in 0..4 {
                    if c.a >> p & 1 != 0 {
                        self.play[p] = None;
                    }
                }
            }
            OP_PLAY_SET => {
                let mut ids = Vec::new();
                let mut w = Vec::new();
                for (i, b) in c.b.to_le_bytes().into_iter().chain(c.c.to_le_bytes()).enumerate() {
                    if b != 0xFF && self.recs.contains_key(&b) {
                        ids.push(b);
                        w.push((c.d >> (i * 4) & 0xF) as u8);
                    }
                }
                if !ids.is_empty() {
                    let mut set = PlaySet { ids, w, order: (c.a & 0xFF) as u32, cur: 0 };
                    set.cur = self.pick(&set, None);
                    let flags = (c.a >> 8) as u32 | PLAY_LOOP;
                    let rec = set.ids[set.cur];
                    if flags & PLAY_FROM_SLOT != 0 {
                        self.load_rec_start(rec);
                    }
                    self.play[port] = Some(Play { rec, flags, at: 0, set: Some(set), once: false });
                }
            }
            OP_CPU => {
                self.note(format!("lab: frame {}: P{} control {} param {}", self.f, port + 1, c.a, c.b));
                let d = &mut self.dummy[port];
                d.beh = c.a as i32;
                d.param = c.b as i32;
                d.st = DummyState::default();
                if d.beh == CPU_LEVEL {
                    // the retail CPU level on the player (the port's slot type must be CPU for the AI to drive it)
                    melee_fighter::tu::player::Player_SetPlayerAndEntityCpuLevel(port as i32, d.param.clamp(1, 9));
                }
                if d.beh == CPU_RECORDING {
                    let rec = c.b as u8;
                    self.play[port] = self.recs.contains_key(&rec).then(|| Play { rec, flags: PLAY_LOOP, at: 0, set: None, once: false });
                }
            }
            OP_REACTION => {
                if let Some(r) = self.dummy[port].react.get_mut(c.a as usize) {
                    r.option = c.b as i32;
                    r.p = ReactParam::from_u32(c.c);
                    self.note(format!("lab: frame {}: P{} reaction {} option {} param {:#x}", self.f, port + 1, c.a, c.b, c.c));
                }
            }
            OP_REACTION_WEIGHTS => {
                if let Some(r) = self.dummy[port].react.get_mut(c.a as usize) {
                    r.weights = c.b;
                }
            }
            OP_PERCENT => {
                let v = f32::from_bits(c.b);
                melee_fighter::tu::player::Player_SetHUDDamage(port as i32, v as i32);
                if let Some(fp) = unsafe { fighter(port as i32) } {
                    fp.dmg.x1830_percent = v;
                }
                self.note(format!("lab: frame {}: P{} percent {v} (forks the timeline)", self.f, port + 1));
            }
            OP_POS => {
                let (x, y) = (f32::from_bits(c.b), f32::from_bits(c.c));
                unsafe {
                    let g = melee_fighter::player::get_entity(port as i32);
                    if let Some(fp) = fighter(port as i32) {
                        for v in [&mut fp.cur_pos, &mut fp.prev_pos, &mut fp.coll_data.cur_pos, &mut fp.coll_data.prev_pos, &mut fp.coll_data.last_pos] {
                            v.x = x;
                            v.y = y;
                        }
                        fp.self_vel = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        fp.x8c_kb_vel = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                        melee_fighter::ftcommon2::ftcommon_8007d5d4(fp);
                        melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(g as _);
                    }
                }
                self.note(format!("lab: frame {}: P{} position {x},{y} (forks the timeline)", self.f, port + 1));
            }
            OP_RNG_SEED => self.rng = mix32(c.b) | 1,
            _ => {}
        }
    }

    fn note(&self, s: String) {
        eprintln!("{s}");
    }

    fn load_rec_start(&mut self, rec: u8) {
        if let Some((fr, s)) = self.recs.get(&rec).and_then(|r| r.start.clone()) {
            self.load_state(&s);
            self.f = fr;
            self.ring.clear();
            self.log.clear();
            self.log_f0 = fr;
            self.replay_until = None;
            self.exchange = None;
            self.obs = Default::default();
            self.observe(false);
        }
    }

    fn pick(&mut self, set: &PlaySet, prev: Option<usize>) -> usize {
        let n = set.ids.len();
        match set.order {
            ORDER_RANDOM => self.rnd(n as u32) as usize,
            ORDER_WEIGHTED => {
                let tot: u32 = set.w.iter().map(|&w| w as u32).sum();
                if tot == 0 {
                    return self.rnd(n as u32) as usize;
                }
                let mut k = self.rnd(tot);
                for (i, &w) in set.w.iter().enumerate() {
                    if k < w as u32 {
                        return i;
                    }
                    k -= w as u32;
                }
                0
            }
            _ => prev.map_or(0, |p| (p + 1) % n),
        }
    }

    fn trim_ring(&mut self) {
        let keep = self.ring_frames as i32;
        let oldest = self.f - keep;
        while self.ring.len() > 1 && self.ring.get(1).is_some_and(|r| r.0 <= oldest) {
            self.ring.pop_front();
        }
        let f0 = self.ring.front().map_or(self.f, |r| r.0).min(self.replay_until.map_or(i32::MAX, |_| self.log_f0.max(0)));
        while self.log_f0 < f0 && !self.log.is_empty() {
            self.log.pop_front();
            self.log_f0 += 1;
        }
        // distinct bytes (consecutive ring snapshots share their unchanged pages), sampled every 60 captures
        if self.perf.ring_saves % 60 == 0 {
            self.perf.ring_bytes = ModesGame::distinct_bytes(self.ring.iter().map(|r| &r.1));
            self.perf.ring_peak_bytes = self.perf.ring_peak_bytes.max(self.perf.ring_bytes);
        }
    }

    /// Back n frames: load the newest ring snapshot before the target and re-sim to it with the logged pads (as the
    /// rollback loop does); the logged future plays back until `go_live`.
    fn rewind(&mut self, n: i32) {
        let target = (self.f - n).max(self.ring.front().map_or(self.f, |r| r.0));
        if target >= self.f || self.ring.is_empty() {
            return;
        }
        let t = std::time::Instant::now();
        // the snapshot one frame before the target at least, so the target frame itself is stepped and drawn
        let Some((k, s)) = self.ring.iter().rev().find(|r| r.0 < target).or_else(|| self.ring.front()).cloned() else { return };
        self.load_state(&s);
        let mut fr = k;
        while fr < target {
            let i = (fr - self.log_f0) as usize;
            let Some(&p) = self.log.get(i) else { break };
            self.step_engine(p, fr + 1 < target);
            self.perf.resim_frames += 1;
            fr += 1;
        }
        self.f = fr;
        self.replay_until = Some(self.log_f0 + self.log.len() as i32);
        self.perf.rewinds += 1;
        self.perf.rewind_ns += t.elapsed().as_nanos() as u64;
        self.observe(false);
        self.drew = true;
    }

    // ------------------------------------------------------------------------------------------------ input

    /// The pad of a playback on `port` for this frame (advancing it), or None.
    fn playback(&mut self, port: usize) -> Option<PADStatus> {
        let mut pl = self.play[port].take()?;
        let len = self.recs.get(&pl.rec)?.frames.len();
        if pl.at >= len {
            if pl.once || len == 0 {
                return None;
            }
            if let Some(mut set) = pl.set.take() {
                set.cur = self.pick(&set, Some(set.cur));
                pl.rec = set.ids[set.cur];
                pl.set = Some(set);
            } else if pl.flags & PLAY_LOOP == 0 {
                return None;
            }
            pl.at = 0;
            if pl.flags & PLAY_FROM_SLOT != 0 {
                let r = pl.rec;
                self.load_rec_start(r);
            }
        }
        let rec = self.recs.get(&pl.rec)?;
        let fr = rec.frames.get(pl.at)?;
        let src = if rec.mask >> port & 1 != 0 { port } else { rec.mask.trailing_zeros().min(3) as usize };
        let mut p = fr[src];
        if pl.flags & PLAY_MIRROR != 0 {
            mirror(&mut p);
        }
        pl.at += 1;
        self.play[port] = Some(pl);
        Some(p)
    }

    /// The dummy synthesiser (B3 + GAP-5): a pad from the dummy's settings and the sim state before this frame.
    fn synth(&mut self, port: usize, live: PADStatus) -> PADStatus {
        let d = self.dummy[port];
        if d.beh == CPU_NONE && d.react.iter().all(|r| r.option == RO_NONE) {
            return live;
        }
        let Some(fp) = (unsafe { fighter(port as i32) }) else { return live };
        // no base behaviour (CPU_NONE): the reactions act on top of the port's own pad (a human, or nobody), so setting
        // a tech or DI option never takes the stick away between hits; a behaviour replaces the pad
        let mut p = if d.beh == CPU_NONE { live } else { PADStatus::zeroed() };
        // a dummy on a port without a controller (a human slot nobody holds): the synthesised pad is connected, or
        // the game would read it as unplugged and the dummy would never act
        p.err = 0;
        let f = self.f;
        let m = fp.motion_id;
        let (x, y) = (fp.cur_pos.x, fp.cur_pos.y);
        let toward_centre: i8 = if x > 0.0 { -127 } else { 127 };
        let opp = (0..4).filter(|&q| q != port).find_map(|q| unsafe { fighter(q as i32) }.map(|o| (q, o.cur_pos.x, o.cur_pos.y, o.motion_id, o.x914.iter().any(|h| h.state != 0))));
        let away_from_opp: i8 = match opp { Some((_, ox, ..)) if ox > x => -127, Some(_) => 127, None => -toward_centre };
        let hitlag = fp.dmg.x195c_hitlag_frames;
        let mut st = d.st;
        // base behaviour
        match d.beh {
            CPU_CROUCH => p.stickY = -127,
            CPU_JUMP => {
                if fp.ground_or_air == melee_types::GA_Ground && is_idle(m) && f % 2 == 0 {
                    p.button |= BTN_X;
                }
            }
            CPU_SHIELD => {
                p.button |= BTN_R;
                p.triggerRight = 255;
            }
            CPU_LEVEL => return live,
            _ => {}
        }
        // a fresh hit: the per-hit random draws
        if hitlag > 0.0 && self.obs[port].hitlag <= 0.0 {
            st.hit_at = f;
            let r = d.react[RK_DI as usize];
            st.di = if r.option == RO_RANDOM { self.draw(r.weights, &[RO_IN, RO_OUT, RO_AWAY, RO_TOWARD, RO_UP, RO_DOWN]) } else { r.option };
            let r = d.react[RK_SDI as usize];
            st.sdi = if r.option == RO_RANDOM { self.draw(r.weights, &[RO_IN, RO_AWAY, RO_UP, RO_DOWN]) } else { r.option };
            let r = d.react[RK_TECH as usize];
            st.tech = if r.option == RO_RANDOM { self.draw(r.weights, &[RO_TECH_IN_PLACE, RO_TECH_ROLL_IN, RO_TECH_ROLL_AWAY, RO_TECH_MISS]) } else { r.option };
        }
        let dir = |o: i32| -> (i8, i8) {
            match o {
                RO_IN => (toward_centre, 0),
                RO_OUT => (-toward_centre, 0),
                RO_AWAY => (away_from_opp, 0),
                RO_TOWARD => (-away_from_opp, 0),
                RO_UP => (0, 127),
                RO_DOWN => (0, -127),
                _ => (0, 0),
            }
        };
        let scale = |v: i8, s: u8| -> i8 { if s == 0 { v } else { (v as i32 * s.min(100) as i32 / 100) as i8 } };
        if hitlag > 0.0 {
            if hitlag <= 1.0 {
                // DI: the stick on the last hitlag frame
                let (sx, sy) = dir(st.di);
                let s = d.react[RK_DI as usize].p.strength;
                p.stickX = scale(sx, s);
                p.stickY = scale(sy, s);
            } else if st.sdi != RO_NONE {
                // SDI: pulses every `delay` frames (each a fresh push out of the dead zone)
                let every = d.react[RK_SDI as usize].p.delay.max(1) as i32;
                if (f - st.hit_at) % (every * 2) < every {
                    let (sx, sy) = dir(st.sdi);
                    let s = d.react[RK_SDI as usize].p.strength;
                    p.stickX = scale(sx, s);
                    p.stickY = scale(sy, s);
                }
            }
        }
        // tech: one press about 6 frames before the predicted landing, past the 40-frame lockout
        st.since_l += 1;
        if fp.ground_or_air == melee_types::GA_Air && matches!(m, 75..=91) && hitlag <= 0.0 && st.tech != RO_NONE && st.tech != RO_TECH_MISS {
            let vy = fp.self_vel.y + fp.x8c_kb_vel.y;
            if vy < 0.0 && y + vy * 6.0 <= self.obs[port].ground_y + 2.0 && st.since_l > 41 {
                p.button |= BTN_L;
                p.triggerLeft = 255;
                st.since_l = 0;
                match st.tech {
                    RO_TECH_ROLL_IN => p.stickX = toward_centre,
                    RO_TECH_ROLL_AWAY => p.stickX = -toward_centre,
                    _ => {}
                }
            }
        }
        // getup and ledge options after `delay` frames in the wait state
        if matches!(m, 184 | 192 | 253) {
            if self.obs[port].motion != m {
                st.wait_since = f;
            }
            let k = if m == 253 { RK_LEDGE } else { RK_GETUP } as usize;
            let r = d.react[k];
            if r.option != RO_NONE && f - st.wait_since >= r.p.delay as i32 {
                let o = if r.option == RO_RANDOM { self.draw(r.weights, &[1, 2, 3, 4]) } else { r.option };
                match o {
                    RO_GETUP_STAND => if m == 253 { p.stickX = toward_centre } else { p.stickY = 127 },
                    RO_GETUP_ROLL_IN => if m == 253 { p.button |= BTN_R; p.triggerRight = 255 } else { p.stickX = toward_centre },
                    RO_GETUP_ROLL_AWAY => if m == 253 { p.button |= BTN_R; p.triggerRight = 255 } else { p.stickX = -toward_centre },
                    RO_GETUP_ATTACK => p.button |= BTN_A,
                    RO_LEDGE_JUMP => p.button |= BTN_X,
                    RO_LEDGE_DROP => p.stickX = -toward_centre,
                    RO_RECORDING => self.react_rec(port, r.p.rec),
                    _ => {}
                }
            }
        }
        // block policy (GAP-5): shield while the opponent attacks
        let r = d.react[RK_SHIELD as usize];
        if let Some((_, ox, _, om, oactive)) = opp {
            if om != st.opp_motion {
                st.opp_motion = om;
                st.block = match r.option {
                    RO_BLOCK_ALL => true,
                    RO_BLOCK_AFTER_FIRST => f - st.hit_at < 120 && st.hit_at > 0,
                    RO_BLOCK_RANDOM => self.rnd(100) < if r.p.chance == 0 { 50 } else { r.p.chance as u32 },
                    _ => false,
                };
            }
            let threat = oactive || (!is_idle(om) && !is_stun_motion(om));
            if st.block && threat && (ox - x).abs() < 40.0 && fp.ground_or_air == melee_types::GA_Ground && hitlag <= 0.0 {
                p.button |= BTN_R;
                p.triggerRight = 255;
            }
        }
        // out of shield: when shield stun ends, the option after `delay` frames (the shield held meanwhile)
        let r = d.react[RK_OOS as usize];
        if r.option != RO_NONE {
            if self.obs[port].motion == 181 && m != 181 {
                st.oos = Some((f, if r.option == RO_OOS_RANDOM { self.draw(r.weights, &[1, 2, 3, 4, 5, 6]) } else { r.option }));
            }
            if let Some((at, o)) = st.oos {
                if f - at < r.p.delay as i32 {
                    p.button |= BTN_R;
                    p.triggerRight = 255;
                } else {
                    match o {
                        RO_OOS_JUMP => p.button |= BTN_X,
                        RO_OOS_NAIR => {
                            p.button |= BTN_X;
                            st.nair = true;
                        }
                        RO_OOS_GRAB => {
                            p.button |= BTN_R | BTN_A;
                            p.triggerRight = 255;
                        }
                        RO_OOS_UPB => {
                            p.stickY = 127;
                            p.button |= BTN_B;
                        }
                        RO_OOS_SPOTDODGE => {
                            p.button |= BTN_R;
                            p.triggerRight = 255;
                            p.stickY = -127;
                        }
                        RO_OOS_ROLL => {
                            p.button |= BTN_R;
                            p.triggerRight = 255;
                            p.stickX = away_from_opp;
                        }
                        RO_RECORDING => self.react_rec(port, r.p.rec),
                        _ => {}
                    }
                    st.oos = None;
                }
            }
        }
        if st.nair && fp.ground_or_air == melee_types::GA_Air && m != 24 {
            p.button |= BTN_A;
            st.nair = false;
        }
        self.dummy[port].st = st;
        p
    }

    fn react_rec(&mut self, port: usize, rec: u8) {
        if self.recs.contains_key(&rec) && self.play[port].is_none() {
            self.play[port] = Some(Play { rec, flags: 0, at: 0, set: None, once: true });
        }
    }

    // ------------------------------------------------------------------------------------------------ the frame

    /// One command-clock frame: the commands, then (unless frozen) one match frame. True when a frame ran.
    pub fn tick(&mut self, live: [PADStatus; 4], cmds: &[LabCmd]) -> bool {
        self.drew = false;
        for c in cmds {
            self.apply(c);
        }
        if self.g.e.ended {
            self.publish();
            // the match is over (results): no more lab commands (the results' START presses are not freezes)
            self.sh.borrow_mut().active = false;
            return false;
        }
        if self.paused && self.steps == 0 {
            self.events.clear();
            self.publish();
            return self.drew;
        }
        if self.paused {
            self.steps -= 1;
        }
        let f = self.f;
        let mut pads = live;
        // START belongs to the Lab (freeze, lab-ui-spec.md 4): the game never sees it, except with L+R+A held (the
        // retail pause, so L+R+A+START still quits the match)
        for p in pads.iter_mut() {
            if p.button & (BTN_L | BTN_R | BTN_A) != BTN_L | BTN_R | BTN_A {
                p.button &= !BTN_START;
            }
        }
        let replaying = self.replay_until.is_some_and(|e| f < e);
        if replaying {
            pads = self.log[(f - self.log_f0) as usize];
        } else {
            if self.replay_until.is_some() {
                self.replay_until = None;
            }
            for port in 0..4 {
                if let Some(p) = self.playback(port) {
                    let err = pads[port].err;
                    pads[port] = p;
                    pads[port].err = err;
                } else {
                    pads[port] = self.synth(port, pads[port]);
                }
            }
        }
        for r in self.recs.values_mut() {
            if r.open {
                r.frames.push(pads);
            }
        }
        if !replaying {
            if self.ring_frames > 0 {
                // the log covers the ring; a frame already logged (re-entered after a load) is replaced
                let i = (f - self.log_f0) as usize;
                self.log.truncate(i);
                self.log.push_back(pads);
            } else {
                self.log_f0 = f + 1;
            }
        }
        self.step_engine(pads, false);
        self.last_pads = pads;
        self.f += 1;
        self.drew = true;
        if self.ring_frames > 0 && self.f % RING_EVERY == 0 && !self.ring.iter().any(|r| r.0 == self.f) {
            let t = std::time::Instant::now();
            let (s, fresh) = self.g.save_after(self.ring.back().map(|r| &r.1));
            self.perf.ring_fresh += fresh as u64;
            self.perf.ring_saves += 1;
            self.perf.ring_ns += t.elapsed().as_nanos() as u64;
            let f = self.f;
            self.ring.retain(|r| r.0 < f);
            self.ring.push_back((f, s));
            self.trim_ring();
        }
        self.observe(true);
        self.publish();
        true
    }

    /// Read the fighters after a frame: events, cells, observed frame data (reads only).
    fn observe(&mut self, events: bool) {
        self.events.clear();
        let f = self.f;
        for port in 0..4 {
            let Some(fp) = (unsafe { fighter(port as i32) }) else {
                self.obs[port] = Obs::default();
                continue;
            };
            let mut o = std::mem::take(&mut self.obs[port]);
            let m = fp.motion_id;
            let hitlag = fp.dmg.x195c_hitlag_frames;
            let pct = fp.dmg.x1830_percent;
            let iasa = fp.allow_interrupt() != 0;
            let air = fp.ground_or_air == melee_types::GA_Air;
            let pos = (fp.cur_pos.x, fp.cur_pos.y);
            let active = fp.x914.iter().any(|h| h.state != 0);
            let changed = !o.present || o.motion != m;
            let mut ev = |kind: u8, a: i32, b: i32| {
                if events {
                    self.events.push(LabEvent { kind, port: port as u8, frame: f, a, b });
                }
            };
            if o.present && events {
                if changed {
                    if is_landing_air(m) && is_aerial(o.motion) {
                        ev(EV_LCANCEL, (fp.x67F < 7) as i32, fp.x67F as i32);
                    }
                    match m {
                        199 => ev(EV_TECH, 1, 0),
                        200 => ev(EV_TECH, 2, 0),
                        201 => ev(EV_TECH, 3, 0),
                        183 | 191 if matches!(o.motion, 75..=91 | 38) => ev(EV_TECH, 0, 0),
                        182 => ev(EV_POWERSHIELD, o.shield_frames, 0),
                        181 if matches!(o.motion, 178 | 179) => ev(EV_SHIELD, fp.dmg.x18c4_source_ply, (fp.shield_health * 100.0) as i32),
                        _ => {}
                    }
                    if matches!(o.motion, 178 | 179 | 181 | 182) && !matches!(m, 178..=182) && !is_stun_motion(m) {
                        ev(EV_OOS, m, o.shield_frames);
                    }
                    if o.motion == 24 {
                        o.js_end = Some(f);
                    }
                    if m == 236 {
                        let lp = self.last_pads[port];
                        let ang = if lp.stickX == 0 && lp.stickY == 0 { 0 } else { ((lp.stickY as f32).atan2(lp.stickX as f32 * fp.facing_dir).to_degrees()).round() as i32 };
                        let after = o.js_end.filter(|&j| f - j <= 10).map_or(-1, |j| f - j);
                        o.airdodge = Some((f, ang, after));
                        if o.left_ledge.is_some() {
                            o.ledge_dodged = true;
                        }
                        ev(EV_AIRDODGE, ang, after);
                    }
                    if m == 43 && o.motion == 236 {
                        if let Some((_, ang, after)) = o.airdodge.filter(|a| a.2 >= 0) {
                            ev(EV_WAVEDASH, ang, after);
                        }
                    }
                    if is_cliff(o.motion) && !is_cliff(m) && air {
                        o.left_ledge = Some(f);
                        o.ledge_dodged = false;
                    }
                }
                if o.airborne && !air {
                    if let Some(l) = o.left_ledge.take() {
                        if f - l <= 60 && o.ledge_dodged {
                            ev(EV_LEDGEDASH, fp.x1990, 0);
                        }
                    }
                }
                if iasa && !o.iasa && !is_idle(m) {
                    ev(EV_IASA, m, o.motion_frames + 1);
                }
                if hitlag > 0.0 && o.hitlag <= 0.0 && pct > o.percent {
                    ev(EV_HIT, fp.dmg.x18c4_source_ply, ((pct - o.percent) * 100.0) as i32);
                    self.exchange = Some(f);
                }
                if m == 181 && o.motion != 181 {
                    self.exchange = Some(f);
                }
                if hitlag > 0.0 && o.hitlag > 0.0 {
                    let (dx, dy) = (pos.0 - o.pos.0, pos.1 - o.pos.1);
                    if dx.abs() + dy.abs() > 0.001 {
                        ev(EV_SDI, (dx * 100.0) as i32, (dy * 100.0) as i32);
                    }
                } else if hitlag <= 0.0 && o.hitlag > 0.0 {
                    let (dx, dy) = (pos.0 - o.pos.0 - fp.self_vel.x - fp.x8c_kb_vel.x, pos.1 - o.pos.1 - fp.self_vel.y - fp.x8c_kb_vel.y);
                    if dx.abs() + dy.abs() > 0.01 {
                        ev(EV_ASDI, (dx * 100.0) as i32, (dy * 100.0) as i32);
                    }
                }
            }
            // frame meter cell (GAP-8)
            if changed {
                // the finished action's observed timeline (gd_lab_framedata)
                if o.present && !o.timeline.is_empty() && !is_idle(o.motion) && !is_stun_motion(o.motion) {
                    let fd = FrameData { cells: std::mem::take(&mut o.timeline), first_active: o.tl_first, last_active: o.tl_last, iasa: o.tl_iasa };
                    self.sh.borrow_mut().framedata.insert((fp.kind as i32, o.motion), fd);
                }
                o.timeline.clear();
                o.tl_first = -1;
                o.tl_last = -1;
                o.tl_iasa = -1;
                o.motion_frames = 0;
                o.had_active = false;
            } else {
                o.motion_frames += 1;
            }
            o.had_active |= active;
            let cell = if hitlag > 0.0 || (is_stun_motion(m) && !iasa) {
                CELL_STUN
            } else if is_idle(m) {
                CELL_IDLE
            } else if active {
                CELL_ACTIVE
            } else if iasa {
                CELL_CAN_ACT
            } else if o.had_active {
                CELL_RECOVERY
            } else {
                CELL_STARTUP
            };
            if o.timeline.len() < METER_CELLS {
                let i = o.timeline.len() as i32;
                if cell == CELL_ACTIVE {
                    if o.tl_first < 0 {
                        o.tl_first = i;
                    }
                    o.tl_last = i;
                }
                if cell == CELL_CAN_ACT && o.tl_iasa < 0 {
                    o.tl_iasa = i;
                }
                o.timeline.push(cell);
            }
            let actionable = cell == CELL_IDLE || cell == CELL_CAN_ACT;
            if actionable && !o.actionable {
                o.act_at = f;
            }
            o.actionable = actionable;
            if events {
                let mut s = self.sh.borrow_mut();
                let q = &mut s.meter[port];
                q.push_back(cell);
                if q.len() > METER_CELLS {
                    q.pop_front();
                }
            }
            if matches!(m, 178 | 179 | 181 | 182) {
                o.shield_frames += 1;
            } else {
                o.shield_frames = 0;
            }
            if !air {
                o.ground_y = pos.1;
            }
            o.present = true;
            o.motion = m;
            o.hitlag = hitlag;
            o.percent = pct;
            o.iasa = iasa;
            o.airborne = air;
            o.pos = pos;
            self.obs[port] = o;
        }
        // frame advantage: after an exchange, once both are actionable again (P1 relative to P2)
        if let Some(x) = self.exchange {
            let (a, b) = (&self.obs[0], &self.obs[1]);
            if a.present && b.present && a.actionable && b.actionable && a.act_at > x && b.act_at > x {
                self.sh.borrow_mut().frame_adv = b.act_at - a.act_at;
                self.exchange = None;
            } else if f - x > 300 {
                self.exchange = None;
            }
        }
    }

    /// Copy what the queries read into the shared state.
    fn publish(&mut self) {
        let mut s = self.sh.borrow_mut();
        s.match_frame = self.f;
        s.paused = self.paused;
        s.replaying = self.replay_until.is_some_and(|e| self.f < e);
        s.ring_frames = self.ring_frames;
        s.ring_len = self.ring.back().map_or(0, |b| (b.0 - self.ring.front().unwrap().0) as u32);
        s.rng_seed = mix32(self.rng);
        for (i, sl) in self.slots.iter().enumerate() {
            s.slot_frame[i] = sl.as_ref().map_or(-1, |x| x.0);
        }
        s.rec_len.fill(-1);
        for (k, r) in &self.recs {
            s.rec_len[*k as usize] = r.frames.len() as i32;
        }
        s.recording_mask = self.recs.values().filter(|r| r.open).fold(0, |m, r| m | r.mask as u32);
        let ev = std::mem::take(&mut self.events);
        s.publish_events(&ev);
        self.events = ev;
        for port in 0..4 {
            let Some(fp) = (unsafe { fighter(port as i32) }) else {
                s.fighters[port] = None;
                s.hitboxes[port].clear();
                s.hurtboxes[port].clear();
                continue;
            };
            let o = &self.obs[port];
            let mut b = [0u8; FIGHTER_V1_BYTES];
            let mut w = 0usize;
            let mut put = |v: [u8; 4]| {
                b[w..w + 4].copy_from_slice(&v);
                w += 4;
            };
            let m = fp.motion_id;
            let hitstun = if matches!(m, 75..=91) { unsafe { *(core::ptr::addr_of!(fp.mv) as *const f32) } } else { 0.0 };
            put(1u32.to_le_bytes());
            put((FIGHTER_V1_BYTES as u32).to_le_bytes());
            put((fp.kind as i32).to_le_bytes());
            put(m.to_le_bytes());
            put(fp.cur_anim_frame.to_le_bytes());
            put(fp.anim_id.to_le_bytes());
            put(fp.dmg.x195c_hitlag_frames.to_le_bytes());
            put(hitstun.to_le_bytes());
            put((fp.allow_interrupt() as i32).to_le_bytes());
            put(fp.x1990.to_le_bytes());
            put(fp.x1994.to_le_bytes());
            put(fp.shield_health.to_le_bytes());
            put((fp.x1968_jumpsUsed as i32).to_le_bytes());
            put(((fp.ground_or_air == melee_types::GA_Air) as i32).to_le_bytes());
            for v in [fp.cur_pos.x, fp.cur_pos.y, fp.self_vel.x, fp.self_vel.y, fp.dmg.x1830_percent, fp.facing_dir] {
                put(v.to_le_bytes());
            }
            let e = &fp.coll_data.ecb;
            for v in [e.top.x, e.top.y, e.bottom.x, e.bottom.y, e.right.x, e.right.y, e.left.x, e.left.y] {
                put(v.to_le_bytes());
            }
            put(((!fp.victim_gobj.is_null()) as i32).to_le_bytes());
            put((s.meter[port].back().copied().unwrap_or(0) as i32).to_le_bytes());
            put(o.motion_frames.to_le_bytes());
            put(self.f.to_le_bytes());
            s.fighters[port] = Some(b);
            let hb = &mut s.hitboxes[port];
            hb.clear();
            for (i, h) in fp.x914.iter().enumerate() {
                if h.state == 0 {
                    continue;
                }
                let mut r = [0u8; HITBOX_BYTES];
                for (k, v) in [h.x4C.x, h.x4C.y, h.x58.x, h.x58.y, h.scale, h.damage].iter().enumerate() {
                    r[k * 4..k * 4 + 4].copy_from_slice(&v.to_le_bytes());
                }
                r[24..28].copy_from_slice(&h.kb_angle.to_le_bytes());
                r[28..32].copy_from_slice(&(i as i32).to_le_bytes());
                hb.push(r);
            }
            let hu = &mut s.hurtboxes[port];
            hu.clear();
            for h in fp.hurt_capsules.iter().take(fp.hurt_capsules_len as usize) {
                let c = &h.capsule;
                let mut r = [0u8; HURTBOX_BYTES];
                for (k, v) in [c.a_pos.x, c.a_pos.y, c.b_pos.x, c.b_pos.y, c.scale].iter().enumerate() {
                    r[k * 4..k * 4 + 4].copy_from_slice(&v.to_le_bytes());
                }
                r[20..24].copy_from_slice(&(c.state as i32).to_le_bytes());
                r[24..28].copy_from_slice(&c.bone_idx.to_le_bytes());
                hu.push(r);
            }
            let ch = fp.kind as i32;
            if !s.attrs.contains_key(&ch) {
                let n = core::mem::size_of::<melee_types::ftCo_DatAttrs>() / 4;
                let a = unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(fp.co_attrs) as *const f32, n) }.to_vec();
                s.attrs.insert(ch, a);
            }
        }
    }

    /// The frame's hash (the desync check's and the tests').
    pub fn frame_hash(&self) -> u32 {
        self.g.e.neutral_row()[0] | 1
    }

    pub fn perf_line(&self) -> String {
        let p = &self.perf;
        let us = |ns: u64, n: u64| if n == 0 { 0.0 } else { ns as f64 / n as f64 / 1e3 };
        format!(
            "lab perf: step {:.0} us/frame over {} frames; ring save {:.0} us each ({} saves, {:.0} us per frame amortised, {:.0} KB copied per save, ring {:.1} MB distinct, peak {:.1} MB); slot save {:.0} us; load {:.0} us; rewind {:.0} us each ({} rewinds, {} frames re-simmed)",
            us(p.step_ns, p.steps), p.steps, us(p.ring_ns, p.ring_saves), p.ring_saves, if p.steps == 0 { 0.0 } else { p.ring_ns as f64 / p.steps as f64 / 1e3 }, p.ring_fresh as f64 / 1024.0 / p.ring_saves.max(1) as f64,
            p.ring_bytes as f64 / 1e6, p.ring_peak_bytes as f64 / 1e6, us(p.slot_ns, p.slot_saves), us(p.load_ns, p.loads), us(p.rewind_ns, p.rewinds), p.rewinds, p.resim_frames
        )
    }

    // ------------------------------------------------------------------------------------------------ B4 overlay

    /// The overlay pass (B4): GX draws in 640x480 EFB space appended to the finished frame. Reads the fighters and the
    /// stage's collision lines only; never sets the game's debug-draw fields, so it never changes a hash.
    pub fn overlay(&self, ui: &mut crate::ui::Ui) -> bool {
        let s = self.sh.borrow();
        if s.overlay.iter().all(|&b| b == 0) {
            return false;
        }
        let Some(cam) = Camera::current() else { return false };
        let col = |k: usize| -> crate::ui::Rgba { let c = s.style[k]; [(c >> 24) as u8, (c >> 16) as u8, (c >> 8) as u8, c as u8] };
        let any = s.overlay.iter().fold(0, |a, &b| a | b);
        if any & (OV_COLLISION | OV_LEDGE_GRAB) != 0 {
            unsafe { stage_lines(ui, &cam, any, col(4), col(3)) };
        }
        for port in 0..4 {
            let bits = s.overlay[port];
            if bits == 0 {
                continue;
            }
            let Some(fp) = (unsafe { fighter(port as i32) }) else { continue };
            if bits & OV_HURTBOX != 0 {
                for h in fp.hurt_capsules.iter().take(fp.hurt_capsules_len as usize) {
                    let c = &h.capsule;
                    let k = if fp.x1990 > 0 || fp.x1994 > 0 { 7 } else { 1 };
                    capsule(ui, &cam, (c.a_pos.x, c.a_pos.y, c.a_pos.z), (c.b_pos.x, c.b_pos.y, c.b_pos.z), c.scale, col(k));
                }
            }
            if bits & OV_HITBOX != 0 {
                for h in fp.x914.iter().filter(|h| h.state != 0) {
                    let p = (h.x4C.x, h.x4C.y, h.x4C.z);
                    let q = if bits & OV_SWEPT != 0 { (h.x58.x, h.x58.y, h.x58.z) } else { p };
                    capsule(ui, &cam, p, q, h.scale, col(0));
                }
            }
            if bits & OV_ECB != 0 {
                let e = &fp.coll_data.ecb;
                let (x, y) = (fp.cur_pos.x, fp.cur_pos.y);
                let pts = [(e.top.x, e.top.y), (e.right.x, e.right.y), (e.bottom.x, e.bottom.y), (e.left.x, e.left.y)];
                for i in 0..4 {
                    let (a, b) = (pts[i], pts[(i + 1) % 4]);
                    if let (Some(p), Some(q)) = (cam.project(x + a.0, y + a.1, 0.0), cam.project(x + b.0, y + b.1, 0.0)) {
                        line(ui, p, q, 1.5, col(2));
                    }
                }
            }
            if bits & OV_DI_LINE != 0 {
                // the knockback the fighter is travelling on, and where held DI bends it (18 degrees at full deflection)
                let (vx, vy) = (fp.x8c_kb_vel.x, fp.x8c_kb_vel.y);
                if vx.abs() + vy.abs() > 0.01 {
                    let (x, y) = (fp.cur_pos.x, fp.cur_pos.y);
                    let len = 12.0;
                    if let (Some(p), Some(q)) = (cam.project(x, y, 0.0), cam.project(x + vx * len, y + vy * len, 0.0)) {
                        line(ui, p, q, 2.0, col(5));
                    }
                    let lp = self.last_pads[port];
                    let (sx, sy) = (lp.stickX as f32 / 80.0, lp.stickY as f32 / 80.0);
                    let ang = vy.atan2(vx);
                    let perp = (sx * -ang.sin() + sy * ang.cos()).clamp(-1.0, 1.0);
                    let a2 = ang + perp * 18f32.to_radians();
                    let m = (vx * vx + vy * vy).sqrt();
                    if let (Some(p), Some(q)) = (cam.project(x, y, 0.0), cam.project(x + a2.cos() * m * len, y + a2.sin() * m * len, 0.0)) {
                        line(ui, p, q, 1.0, col(0));
                    }
                }
            }
            if bits & (OV_LABELS | OV_IASA) != 0 {
                if let Some(p) = cam.project(fp.cur_pos.x, fp.cur_pos.y + 22.0, 0.0) {
                    let o = &self.obs[port];
                    let t = format!("P{} {} f{}", port + 1, fp.motion_id, o.motion_frames + 1);
                    let c = if bits & OV_IASA != 0 && fp.allow_interrupt() != 0 { [126, 224, 195, 255] } else { col(6) };
                    ui.text(p.0 - 24.0, p.1, 1.0, &t, c);
                }
            }
        }
        true
    }
}

/// The game camera's view and perspective, read from the CObj (a copy: nothing is set up or written).
pub struct Camera {
    m: [[f32; 4]; 3],
    t: f32,
    aspect: f32,
    /// The renderer's widescreen x factor (melee_gx::interp::Widen; 1 at 4:3). The overlay is a 640x480 ortho layer
    /// the renderer does not widen, while the game's own perspective draws get x * factor: the overlay applies the
    /// same factor itself or it drifts outward and stretches 4:3 -> 16:9 (the "warped" hitboxes).
    wx: f32,
}

impl Camera {
    pub fn current() -> Option<Camera> {
        unsafe {
            let g = melee_world::tu::camera::game_camera.gobj;
            if g.is_null() {
                return None;
            }
            let c = (*g).hsd_obj as *const melee_types::HSD_CObj;
            if c.is_null() {
                return None;
            }
            let p = (*c).projection_param.perspective;
            if !(p.fov > 1.0 && p.fov < 170.0) {
                return None;
            }
            let wx = if melee_engine::view::camera_mode() { None } else { melee_gx::interp::Widen::for_aspect(melee_engine::view::aspect()) };
            Some(Camera { m: (*c).view_mtx, t: (p.fov.to_radians() * 0.5).tan(), aspect: if p.aspect > 0.1 { p.aspect } else { 4.0 / 3.0 }, wx: wx.map_or(1.0, |w| w.factor) })
        }
    }

    /// World point to 640x480 EFB pixels (None behind the camera).
    pub fn project(&self, x: f32, y: f32, z: f32) -> Option<(f32, f32)> {
        let m = &self.m;
        let cx = m[0][0] * x + m[0][1] * y + m[0][2] * z + m[0][3];
        let cy = m[1][0] * x + m[1][1] * y + m[1][2] * z + m[1][3];
        let cz = m[2][0] * x + m[2][1] * y + m[2][2] * z + m[2][3];
        if cz > -0.01 {
            return None;
        }
        let nx = cx / (-cz * self.t * self.aspect) * self.wx;
        let ny = cy / (-cz * self.t);
        Some((320.0 + nx * 320.0, 240.0 - ny * 240.0))
    }

    /// Overlay pixels per world unit at a point, in x and in y (they differ: 73:60 cameras on a 4:3 layer, widen).
    fn scale_xy(&self, x: f32, y: f32, z: f32) -> (f32, f32) {
        match (self.project(x, y, z), self.project(x + 1.0, y, z), self.project(x, y + 1.0, z)) {
            (Some(a), Some(b), Some(c)) => ((b.0 - a.0).abs(), (c.1 - a.1).abs()),
            _ => (0.0, 0.0),
        }
    }
}

fn line(ui: &mut crate::ui::Ui, a: (f32, f32), b: (f32, f32), w: f32, c: crate::ui::Rgba) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = (dx.abs().max(dy.abs()) / w.max(1.0)).ceil().clamp(1.0, 400.0) as i32;
    for i in 0..=n {
        let t = i as f32 / n as f32;
        ui.rect(a.0 + dx * t - w * 0.5, a.1 + dy * t - w * 0.5, w, w, c);
    }
}

/// An ellipse of radii `rx`, `ry` (a world circle on the overlay layer).
fn circle(ui: &mut crate::ui::Ui, p: (f32, f32), rx: f32, ry: f32, c: crate::ui::Rgba) {
    let n = ((rx.max(ry) * 0.8) as i32).clamp(8, 40);
    for i in 0..n {
        let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
        line(ui, (p.0 + rx * a0.cos(), p.1 + ry * a0.sin()), (p.0 + rx * a1.cos(), p.1 + ry * a1.sin()), 1.5, c);
    }
}

fn capsule(ui: &mut crate::ui::Ui, cam: &Camera, a: (f32, f32, f32), b: (f32, f32, f32), r: f32, c: crate::ui::Rgba) {
    let (Some(p), Some(q)) = (cam.project(a.0, a.1, a.2), cam.project(b.0, b.1, b.2)) else { return };
    let (sx, sy) = cam.scale_xy(a.0, a.1, a.2);
    let (rx, ry) = (r * sx, r * sy);
    if rx <= 0.0 || ry <= 0.0 || rx.max(ry) > 400.0 {
        return;
    }
    circle(ui, p, rx, ry, c);
    if (p.0 - q.0).abs() + (p.1 - q.1).abs() > 0.5 {
        circle(ui, q, rx, ry, c);
        // the tangent lines: the normal in the space where the ellipses are unit circles, mapped back
        let (dx, dy) = ((q.0 - p.0) / rx, (q.1 - p.1) / ry);
        let l = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = (-dy / l * rx, dx / l * ry);
        line(ui, (p.0 + nx, p.1 + ny), (q.0 + nx, q.1 + ny), 1.5, c);
        line(ui, (p.0 - nx, p.1 - ny), (q.0 - nx, q.1 - ny), 1.5, c);
    }
}

/// The stage's collision lines (COLLISION) and ledge corners (LEDGE_GRAB), straight from the collision data.
unsafe fn stage_lines(ui: &mut crate::ui::Ui, cam: &Camera, bits: u32, c_line: crate::ui::Rgba, c_ledge: crate::ui::Rgba) {
    use melee_world::tu::mplib::*;
    let cd = mpLib_804D64B4;
    if cd.is_null() || groundCollLine.is_null() || groundCollVtx.is_null() {
        return;
    }
    let n = (*cd).line_count.clamp(0, 4096);
    for i in 0..n {
        let l = &*groundCollLine.offset(i as isize);
        if l.x0.is_null() {
            continue;
        }
        let ml = &*l.x0;
        let v0 = &(*groundCollVtx.offset(ml.v0_idx as isize)).pos;
        let v1 = &(*groundCollVtx.offset(ml.v1_idx as isize)).pos;
        let (Some(p), Some(q)) = (cam.project(v0.x, v0.y, 0.0), cam.project(v1.x, v1.y, 0.0)) else { continue };
        if bits & OV_COLLISION != 0 {
            line(ui, p, q, 1.0, c_line);
        }
        if bits & OV_LEDGE_GRAB != 0 && l.flags & melee_types::LINE_FLAG_LEDGE as u32 != 0 {
            for e in [p, q] {
                ui.rect(e.0 - 3.0, e.1 - 3.0, 6.0, 6.0, c_ledge);
            }
        }
    }
}
