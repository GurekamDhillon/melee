//! gd-lab-v1, the Lab layer (docs/rust-port/notes/wasm-ui-lab-api.md part B with the 2-player amendment; the
//! lab-ui-spec.md gaps GAP-1, GAP-5, GAP-7, GAP-8, GAP-11, GAP-12 and GAP-13).
//!
//! Split (the sim is never touched from here):
//! * This module is the presentation side: the `gd_lab` imports a Lab mod calls from its presentation store, linked on
//!   the same `Linker<PresData>` as `gd_ui` ([`LabExt`] is a [`PresExtension`]). Every call that changes the sim
//!   (save, load, rewind, pause, step, record, play, CPU / reaction changes, percent and position writes, the lab RNG
//!   seed) is not applied here: it becomes a [`LabCmd`], a fixed 16-byte record queued in [`LabShared::outbox`] with
//!   the frame it will apply on, which the call returns (GAP-13).
//! * The play flow (`melee/src/lab.rs`, which owns the engine) drains the outbox at the next input boundary. Offline
//!   the commands apply on the next lab frame; in `LabNetplay` they ride the lockstep stream attached to this peer's
//!   pad of that frame (np_lockstep.rs `MLS2`), so both peers apply the same commands on the same frame, in the same
//!   order (host's first). Recordings, savestate slots and the rewind ring are host objects there, referred to by
//!   handle; the mod's memory never holds them.
//! * After each frame the play flow publishes what the queries read ([`LabShared::fighters`], boxes, events, the
//!   frame meter cells): every query here is a copy out of that, so a query never writes and costs no sim access.

use crate::pres::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasmi::{Caller, Linker};

pub const LAB_CMD_VERSION: u8 = 1;
pub const LAB_CMD_BYTES: usize = 16;
/// Commands one peer may attach to one lockstep frame (the rest wait for the next frame).
pub const LAB_CMDS_PER_FRAME: usize = 8;

// ---- command opcodes (u8 op, u8 port, u16 a, u32 b, u32 c, u32 d; little-endian)
pub const OP_SAVE: u8 = 1;
pub const OP_LOAD: u8 = 2;
pub const OP_RING: u8 = 3;
pub const OP_REWIND: u8 = 4;
pub const OP_PAUSE: u8 = 5;
pub const OP_STEP: u8 = 6;
pub const OP_GO_LIVE: u8 = 7;
pub const OP_REC_START: u8 = 8;
pub const OP_REC_STOP: u8 = 9;
pub const OP_PLAY: u8 = 10;
pub const OP_PLAY_STOP: u8 = 11;
pub const OP_REC_FREE: u8 = 12;
pub const OP_CPU: u8 = 13;
pub const OP_REACTION: u8 = 14;
pub const OP_PERCENT: u8 = 15;
pub const OP_POS: u8 = 16;
pub const OP_PLAY_SET: u8 = 17;
pub const OP_RNG_SEED: u8 = 18;
pub const OP_REACTION_WEIGHTS: u8 = 19;

// ---- B1..B3 constants (docs/specs/gd-ui-lab-v1.h)
pub const SLOTS: usize = 16;
pub const RING_MAX: u32 = 600;
pub const PLAY_LOOP: u32 = 1;
pub const PLAY_MIRROR: u32 = 2;
pub const PLAY_FROM_SLOT: u32 = 4;
pub const ORDER_ORDERED: u32 = 0;
pub const ORDER_RANDOM: u32 = 1;
pub const ORDER_WEIGHTED: u32 = 2;

pub const CPU_NONE: i32 = 0;
pub const CPU_STAND: i32 = 1;
pub const CPU_CROUCH: i32 = 2;
pub const CPU_JUMP: i32 = 3;
pub const CPU_SHIELD: i32 = 4;
pub const CPU_LEVEL: i32 = 5;
pub const CPU_RECORDING: i32 = 6;

pub const RK_DI: i32 = 0;
pub const RK_SDI: i32 = 1;
pub const RK_TECH: i32 = 2;
pub const RK_GETUP: i32 = 3;
pub const RK_LEDGE: i32 = 4;
pub const RK_SHIELD: i32 = 5;
pub const RK_OOS: i32 = 6;
pub const RK_COUNT: usize = 7;

// options (per kind; RANDOM draws from the lab RNG, a weighted table from gd_lab_reaction_weights)
pub const RO_NONE: i32 = 0;
pub const RO_IN: i32 = 1;
pub const RO_OUT: i32 = 2;
pub const RO_AWAY: i32 = 3;
pub const RO_TOWARD: i32 = 4;
pub const RO_RANDOM: i32 = 5;
pub const RO_UP: i32 = 6;
pub const RO_DOWN: i32 = 7;
// TECH
pub const RO_TECH_IN_PLACE: i32 = 1;
pub const RO_TECH_ROLL_IN: i32 = 2;
pub const RO_TECH_ROLL_AWAY: i32 = 3;
pub const RO_TECH_MISS: i32 = 4;
// GETUP / LEDGE
pub const RO_GETUP_STAND: i32 = 1;
pub const RO_GETUP_ROLL_IN: i32 = 2;
pub const RO_GETUP_ROLL_AWAY: i32 = 3;
pub const RO_GETUP_ATTACK: i32 = 4;
pub const RO_LEDGE_JUMP: i32 = 6;
pub const RO_LEDGE_DROP: i32 = 7;
// SHIELD (block policy, GAP-5)
pub const RO_BLOCK_ALL: i32 = 1;
pub const RO_BLOCK_AFTER_FIRST: i32 = 2;
pub const RO_BLOCK_RANDOM: i32 = 3;
// OOS
pub const RO_OOS_JUMP: i32 = 1;
pub const RO_OOS_NAIR: i32 = 2;
pub const RO_OOS_GRAB: i32 = 3;
pub const RO_OOS_UPB: i32 = 4;
pub const RO_OOS_SPOTDODGE: i32 = 5;
pub const RO_OOS_ROLL: i32 = 6;
/// Any kind: play recording `param.rec` as the reaction (GAP-5).
pub const RO_RECORDING: i32 = 15;

/// GAP-5 reaction parameter: `delay | strength << 8 | chance << 16 | rec << 24`.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ReactParam {
    /// Frames to wait before acting (GETUP, LEDGE, OOS, SDI interval).
    pub delay: u8,
    /// DI / SDI stick strength, 0..100 (0 reads as 100).
    pub strength: u8,
    /// RANDOM block / random reactions: chance in percent (0 reads as 50).
    pub chance: u8,
    /// RO_RECORDING: the recording handle.
    pub rec: u8,
}
impl ReactParam {
    pub fn from_u32(v: u32) -> Self {
        ReactParam { delay: v as u8, strength: (v >> 8) as u8, chance: (v >> 16) as u8, rec: (v >> 24) as u8 }
    }
    pub fn to_u32(self) -> u32 {
        self.delay as u32 | (self.strength as u32) << 8 | (self.chance as u32) << 16 | (self.rec as u32) << 24
    }
}

// ---- B4 overlay bits
pub const OV_HITBOX: u32 = 1 << 0;
pub const OV_HURTBOX: u32 = 1 << 1;
pub const OV_ECB: u32 = 1 << 2;
pub const OV_LEDGE_GRAB: u32 = 1 << 3;
pub const OV_COLLISION: u32 = 1 << 4;
pub const OV_BLASTZONE: u32 = 1 << 5;
pub const OV_CAMERA: u32 = 1 << 6;
pub const OV_LABELS: u32 = 1 << 7;
pub const OV_SWEPT: u32 = 1 << 8;
pub const OV_SHIELD: u32 = 1 << 9;
pub const OV_GRAB: u32 = 1 << 10;
pub const OV_IASA: u32 = 1 << 11;
pub const OV_INVINC: u32 = 1 << 12;
pub const OV_DI_LINE: u32 = 1 << 13;
pub const OV_KB_ARC: u32 = 1 << 14;
pub const OV_STYLE_KINDS: usize = 8;

// ---- B5 events (16 bytes: u8 kind, u8 port, u16 0, i32 frame, i32 a, i32 b)
pub const EV_LCANCEL: u8 = 1; // a = success, b = frames from the press to the landing (255 none)
pub const EV_TECH: u8 = 2; // a = 0 miss, 1 in place, 2 roll forward, 3 roll back
pub const EV_OOS: u8 = 3; // a = action left shield into, b = frames spent in shield states
pub const EV_IASA: u8 = 4; // a = action, b = frame of the action the window opened on
pub const EV_HIT: u8 = 5; // a = attacker port (-1 unknown), b = damage x100
pub const EV_SHIELD: u8 = 6; // a = attacker port, b = shield HP x100 after the hit
pub const EV_POWERSHIELD: u8 = 7; // a = frames into the shield when hit
pub const EV_WAVEDASH: u8 = 8; // a = air dodge angle (degrees, 0 = forward, -90 = down), b = frames after jumpsquat
pub const EV_LEDGEDASH: u8 = 9; // a = GALINT (intangible frames left on landing)
pub const EV_SDI: u8 = 10; // a = dx x100, b = dy x100
pub const EV_ASDI: u8 = 11; // a = dx x100, b = dy x100
pub const EV_AIRDODGE: u8 = 12; // a = angle, b = frames after jumpsquat (-1 none)
pub const EVENT_BYTES: usize = 16;
/// Frames of event history kept for gd_lab_events_range (GAP-1).
pub const EVENT_HISTORY: i32 = 600;

/// Frame meter cell kinds (GAP-8).
pub const CELL_IDLE: u8 = 0;
pub const CELL_STARTUP: u8 = 1;
pub const CELL_ACTIVE: u8 = 2;
pub const CELL_RECOVERY: u8 = 3;
pub const CELL_STUN: u8 = 4;
pub const CELL_CAN_ACT: u8 = 5;
pub const METER_CELLS: usize = 128;

pub const FIGHTER_V1_BYTES: usize = 128;
pub const HITBOX_BYTES: usize = 32;
pub const HURTBOX_BYTES: usize = 28;

/// One lab command: the unit of the lockstep stream (GAP-13). `frame` is not part of the 16 wire bytes: it is the
/// lockstep frame the command rides with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LabCmd {
    pub op: u8,
    pub port: u8,
    pub a: u16,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

impl LabCmd {
    pub fn new(op: u8, port: u8, a: u16, b: u32, c: u32, d: u32) -> Self {
        LabCmd { op, port, a, b, c, d }
    }
    pub fn to_bytes(&self) -> [u8; LAB_CMD_BYTES] {
        let mut o = [0u8; LAB_CMD_BYTES];
        o[0] = self.op;
        o[1] = self.port;
        o[2..4].copy_from_slice(&self.a.to_le_bytes());
        o[4..8].copy_from_slice(&self.b.to_le_bytes());
        o[8..12].copy_from_slice(&self.c.to_le_bytes());
        o[12..16].copy_from_slice(&self.d.to_le_bytes());
        o
    }
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() < LAB_CMD_BYTES || b[0] == 0 || b[0] > OP_REACTION_WEIGHTS {
            return None;
        }
        let u = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
        Some(LabCmd { op: b[0], port: b[1], a: u16::from_le_bytes([b[2], b[3]]), b: u(4), c: u(8), d: u(12) })
    }
}

/// A frame's event (B5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LabEvent {
    pub kind: u8,
    pub port: u8,
    pub frame: i32,
    pub a: i32,
    pub b: i32,
}
impl LabEvent {
    pub fn to_bytes(&self) -> [u8; EVENT_BYTES] {
        let mut o = [0u8; EVENT_BYTES];
        o[0] = self.kind;
        o[1] = self.port;
        o[4..8].copy_from_slice(&self.frame.to_le_bytes());
        o[8..12].copy_from_slice(&self.a.to_le_bytes());
        o[12..16].copy_from_slice(&self.b.to_le_bytes());
        o
    }
}

/// The observed timeline of one (character, action): cells of its last complete run (`gd_lab_framedata`).
#[derive(Clone, Default, Debug)]
pub struct FrameData {
    pub cells: Vec<u8>,
    pub first_active: i32,
    pub last_active: i32,
    pub iasa: i32,
}

/// What the presentation side and the play flow share (one per process, Rc on the system heap: never in the arena,
/// never snapshotted, so a savestate load or a rewind never rewinds the Lab itself).
pub struct LabShared {
    pub session: Session,
    /// A lab match is running (the play flow owns a lab run): commands are accepted.
    pub active: bool,
    /// The frame the next queued command applies on (the play flow sets it every tick).
    pub apply_frame: i32,
    /// Commands issued since the play flow last drained them.
    pub outbox: Vec<LabCmd>,
    /// The commands applied on the last command-clock frame with their issuer (0 offline / host, 1 guest), in apply
    /// order: `gd_lab_applied` (the other player's actions as toasts online).
    pub applied: Vec<(u8, LabCmd)>,
    /// Which side issues recording handles (0 offline / host, 1 guest): the top bits of a handle.
    pub issuer: u8,
    rec_next: u8,
    // ---- published by the play flow after each frame
    pub match_frame: i32,
    pub paused: bool,
    pub replaying: bool,
    pub ring_frames: u32,
    pub ring_len: u32,
    pub delay: u32,
    pub fighters: [Option<[u8; FIGHTER_V1_BYTES]>; 4],
    pub hitboxes: [Vec<[u8; HITBOX_BYTES]>; 4],
    pub hurtboxes: [Vec<[u8; HURTBOX_BYTES]>; 4],
    pub events: Vec<LabEvent>,
    /// Events of the last EVENT_HISTORY frames (GAP-1).
    pub history: std::collections::VecDeque<LabEvent>,
    pub slot_frame: [i32; SLOTS],
    /// Recording length in frames by handle (-1: no such recording); a fixed table so publishing never allocates.
    pub rec_len: [i32; 256],
    /// The lab match has a timer (the HUD moves the frame meter below it): `gd_lab_timed`.
    pub timed: bool,
    pub recording_mask: u32,
    pub meter: [std::collections::VecDeque<u8>; 4],
    pub frame_adv: i32,
    pub framedata: std::collections::BTreeMap<(i32, i32), FrameData>,
    pub attrs: std::collections::BTreeMap<i32, Vec<f32>>,
    /// The lab RNG's seed and the frame of the last published state (gd_lab_rng is a pure function of these).
    pub rng_seed: u32,
    rng_calls: u32,
    rng_frame: i32,
    // ---- B4 (presentation-only: never a command)
    pub overlay: [u32; 4],
    pub style: [u32; OV_STYLE_KINDS],
    pub log: Vec<String>,
}

impl Default for LabShared {
    fn default() -> Self {
        LabShared {
            session: Session::Offline,
            active: false,
            apply_frame: 0,
            outbox: Vec::with_capacity(64),
            applied: Vec::with_capacity(2 * LAB_CMDS_PER_FRAME),
            issuer: 0,
            rec_next: 1,
            match_frame: 0,
            paused: false,
            replaying: false,
            ring_frames: 0,
            ring_len: 0,
            delay: 0,
            fighters: [None; 4],
            hitboxes: Default::default(),
            hurtboxes: Default::default(),
            events: Vec::new(),
            history: Default::default(),
            slot_frame: [-1; SLOTS],
            rec_len: [-1; 256],
            timed: false,
            recording_mask: 0,
            meter: Default::default(),
            frame_adv: 0,
            framedata: Default::default(),
            attrs: Default::default(),
            rng_seed: 0x1AB5_EED1,
            rng_calls: 0,
            rng_frame: -1,
            overlay: [0; 4],
            style: DEFAULT_STYLE,
            log: Vec::new(),
        }
    }
}

/// Overlay colours by kind: hitbox, hurtbox, ECB, ledge, collision, DI line, labels, invincible (RGBA8).
pub const DEFAULT_STYLE: [u32; OV_STYLE_KINDS] = [0xE0564EA0, 0xE0B84A60, 0xFF8C00D0, 0x7EE0C3E0, 0x8D9AB0C0, 0xFFFFFFE0, 0xE8EDF5FF, 0x4A7FE080];

/// SplitMix32-style mix: the lab RNG (GAP-7) and its uses in the play flow.
pub fn mix32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}

impl LabShared {
    pub fn new() -> Rc<RefCell<LabShared>> {
        Rc::new(RefCell::new(LabShared::default()))
    }

    /// Queue a command; returns its apply frame (or an error code).
    pub fn push(&mut self, c: LabCmd) -> i32 {
        if !self.session.lab_allowed() {
            return GD_EOFFLINE;
        }
        if !self.active {
            return GD_ESTATE;
        }
        if self.outbox.len() >= 64 {
            return GD_EFULL;
        }
        self.outbox.push(c);
        // more than a frame's worth waits for the next frames (the play flow attaches LAB_CMDS_PER_FRAME per frame)
        self.apply_frame + (self.outbox.len() as i32 - 1) / LAB_CMDS_PER_FRAME as i32
    }

    /// A fresh recording handle: issuer in the top 2 bits, so two peers never hand out the same one.
    pub fn new_rec(&mut self) -> u8 {
        let k = self.rec_next;
        self.rec_next = if k >= 63 { 1 } else { k + 1 };
        (self.issuer & 3) << 6 | k
    }

    /// The lab RNG as the presentation side sees it (GAP-7): a pure function of the synced seed, the last published
    /// frame and the call count within that frame. It reads, never advances, the sim's lab RNG: a choice a mod makes
    /// with it that matters to the sim goes out as a command, like every other lab action.
    pub fn rng(&mut self) -> u32 {
        if self.rng_frame != self.match_frame {
            self.rng_frame = self.match_frame;
            self.rng_calls = 0;
        }
        self.rng_calls += 1;
        mix32(self.rng_seed ^ mix32(self.match_frame as u32).wrapping_add(self.rng_calls.wrapping_mul(0x9E37_79B9)))
    }

    /// Session change: B1-B4 state that belongs to the old session is dropped.
    pub fn set_session(&mut self, s: Session) {
        if self.session != s {
            self.session = s;
            self.outbox.clear();
            self.overlay = [0; 4];
            self.issuer = 0;
        }
    }

    /// Publish a frame's events (keeps EVENT_HISTORY frames of history).
    pub fn publish_events(&mut self, ev: &[LabEvent]) {
        self.events.clear();
        self.events.extend_from_slice(ev);
        self.history.extend(ev.iter().copied());
        let keep = self.match_frame - EVENT_HISTORY;
        while self.history.front().is_some_and(|e| e.frame < keep) {
            self.history.pop_front();
        }
    }
}

/// The Lab as A's host sees it: links `gd_lab` and hands each store the shared state.
pub struct LabExt {
    pub shared: Rc<RefCell<LabShared>>,
}

impl LabExt {
    pub fn new() -> LabExt {
        LabExt { shared: LabShared::new() }
    }
}

impl Default for LabExt {
    fn default() -> Self {
        LabExt::new()
    }
}

fn lab(c: &Caller<'_, PresData>) -> Option<Rc<RefCell<LabShared>>> {
    c.data().ext.as_ref()?.downcast_ref::<Rc<RefCell<LabShared>>>().cloned()
}

fn write(c: &mut Caller<'_, PresData>, ptr: u32, b: &[u8]) -> bool {
    let Some(m) = c.data().mem else { return false };
    let d = m.data_mut(c);
    match d.get_mut(ptr as usize..(ptr as usize).saturating_add(b.len())) {
        Some(s) => {
            s.copy_from_slice(b);
            true
        }
        None => false,
    }
}

fn read(c: &Caller<'_, PresData>, ptr: u32, len: u32) -> Option<Vec<u8>> {
    let m = c.data().mem?;
    m.data(c).get(ptr as usize..(ptr as usize).checked_add(len as usize)?).map(|s| s.to_vec())
}

/// The feature gate of an import: LAB plus the sub-area, and an allowed session.
fn gate(c: &Caller<'_, PresData>, sub: u32) -> Result<Rc<RefCell<LabShared>>, i32> {
    let f = c.data().features;
    if f & FEATURE_LAB == 0 || (sub != 0 && f & sub == 0) {
        return Err(GD_ECAP);
    }
    let l = lab(c).ok_or(GD_ESTATE)?;
    if !l.borrow().session.lab_allowed() {
        return Err(GD_EOFFLINE);
    }
    Ok(l)
}

macro_rules! cmd {
    ($c:expr, $sub:expr, $cmd:expr) => {
        match gate(&$c, $sub) {
            Ok(l) => l.borrow_mut().push($cmd),
            Err(e) => e,
        }
    };
}

pub fn link(l: &mut Linker<PresData>) -> Result<(), String> {
    macro_rules! f {
        ($name:literal, $f:expr) => {
            l.func_wrap("gd_lab", $name, $f).map_err(|e| format!("linker {}: {e}", $name))?;
        };
    }
    // ---- B1 savestates and rewind
    f!("gd_lab_save", |c: Caller<'_, PresData>, slot: u32| -> i32 {
        if slot as usize >= SLOTS { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_SAVE, 0, slot as u16, 0, 0, 0))
    });
    f!("gd_lab_load", |c: Caller<'_, PresData>, slot: u32| -> i32 {
        if slot as usize >= SLOTS { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_LOAD, 0, slot as u16, 0, 0, 0))
    });
    f!("gd_lab_slot_frame", |c: Caller<'_, PresData>, slot: u32| -> i32 {
        match gate(&c, FEATURE_LAB_STATE) {
            Ok(l) => l.borrow().slot_frame.get(slot as usize).copied().unwrap_or(GD_EINVAL),
            Err(e) => e,
        }
    });
    f!("gd_lab_ring", |c: Caller<'_, PresData>, frames: u32| -> i32 {
        if frames > RING_MAX { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_RING, 0, frames as u16, 0, 0, 0))
    });
    f!("gd_lab_rewind", |c: Caller<'_, PresData>, n: u32| -> i32 {
        if n > RING_MAX { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_REWIND, 0, 0, n, 0, 0))
    });
    f!("gd_lab_pause", |c: Caller<'_, PresData>, on: i32| -> i32 { cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_PAUSE, 0, (on != 0) as u16, 0, 0, 0)) });
    f!("gd_lab_step", |c: Caller<'_, PresData>, n: i32| -> i32 {
        if n.unsigned_abs() > RING_MAX { return GD_EINVAL }
        if n < 0 {
            cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_REWIND, 0, 0, n.unsigned_abs(), 0, 0))
        } else {
            cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_STEP, 0, 0, n as u32, 0, 0))
        }
    });
    f!("gd_lab_go_live", |c: Caller<'_, PresData>| -> i32 { cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_GO_LIVE, 0, 0, 0, 0, 0)) });
    // ---- B2 record and playback
    f!("gd_lab_rec_start", |c: Caller<'_, PresData>, mask: u32| -> u32 {
        let Ok(l) = gate(&c, FEATURE_LAB_INPUT) else { return 0 };
        let mut s = l.borrow_mut();
        let id = s.new_rec();
        if s.push(LabCmd::new(OP_REC_START, 0, (mask & 0xF) as u16, id as u32, 0, 0)) < 0 { 0 } else { id as u32 }
    });
    f!("gd_lab_rec_stop", |c: Caller<'_, PresData>, rec: u32| -> i32 { cmd!(c, FEATURE_LAB_INPUT, LabCmd::new(OP_REC_STOP, 0, 0, rec & 0xFF, 0, 0)) });
    f!("gd_lab_play", |c: Caller<'_, PresData>, rec: u32, mask: u32, flags: u32| -> i32 {
        cmd!(c, FEATURE_LAB_INPUT, LabCmd::new(OP_PLAY, 0, (mask & 0xF) as u16, rec & 0xFF, flags, 0))
    });
    f!("gd_lab_play_stop", |c: Caller<'_, PresData>| -> i32 { cmd!(c, FEATURE_LAB_INPUT, LabCmd::new(OP_PLAY_STOP, 0, 0xF, 0, 0, 0)) });
    f!("gd_lab_rec_len", |c: Caller<'_, PresData>, rec: u32| -> i32 {
        match gate(&c, FEATURE_LAB_INPUT) {
            Ok(l) => { let v = if rec < 256 { l.borrow().rec_len[rec as usize] } else { -1 }; if v < 0 { GD_EINVAL } else { v } },
            Err(e) => e,
        }
    });
    f!("gd_lab_rec_free", |c: Caller<'_, PresData>, rec: u32| -> i32 { cmd!(c, FEATURE_LAB_INPUT, LabCmd::new(OP_REC_FREE, 0, 0, rec & 0xFF, 0, 0)) });
    // ordered / random / weighted playback over up to 8 recordings: entries are (u32 rec, u32 weight 0..9)
    f!("gd_lab_play_set", |c: Caller<'_, PresData>, port: u32, ptr: u32, n: u32, order_flags: u32| -> i32 {
        if port > 3 || n == 0 || n > 8 { return GD_EINVAL }
        let Some(b) = read(&c, ptr, n * 8) else { return GD_EINVAL };
        let (mut ids, mut w) = ([0xFFu8; 8], 0u32);
        for i in 0..n as usize {
            ids[i] = b[i * 8];
            w |= (u32::from_le_bytes(b[i * 8 + 4..i * 8 + 8].try_into().unwrap()).min(9) & 0xF) << (i * 4);
        }
        let lo = u32::from_le_bytes(ids[0..4].try_into().unwrap());
        let hi = u32::from_le_bytes(ids[4..8].try_into().unwrap());
        cmd!(c, FEATURE_LAB_INPUT, LabCmd::new(OP_PLAY_SET, port as u8, (order_flags & 0xFFFF) as u16, lo, hi, w))
    });
    // ---- B3 dummy and CPU
    f!("gd_lab_cpu", |c: Caller<'_, PresData>, port: u32, beh: i32, param: i32| -> i32 {
        if port > 3 || !(CPU_NONE..=CPU_RECORDING).contains(&beh) { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_CPU, port as u8, beh as u16, param as u32, 0, 0))
    });
    f!("gd_lab_reaction", |c: Caller<'_, PresData>, port: u32, kind: i32, option: i32, param: u32| -> i32 {
        if port > 3 || !(0..RK_COUNT as i32).contains(&kind) || !(0..=15).contains(&option) { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_REACTION, port as u8, kind as u16, option as u32, param, 0))
    });
    // GAP-5 weighted option table: 8 x 4-bit weights for options 1..8 (RANDOM then draws by weight)
    f!("gd_lab_reaction_weights", |c: Caller<'_, PresData>, port: u32, kind: i32, weights: u32| -> i32 {
        if port > 3 || !(0..RK_COUNT as i32).contains(&kind) { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_REACTION_WEIGHTS, port as u8, kind as u16, weights, 0, 0))
    });
    f!("gd_lab_set_percent", |c: Caller<'_, PresData>, port: u32, pct: f32| -> i32 {
        if port > 3 || !(0.0..=999.0).contains(&pct) { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_PERCENT, port as u8, 0, pct.to_bits(), 0, 0))
    });
    f!("gd_lab_set_pos", |c: Caller<'_, PresData>, port: u32, x: f32, y: f32| -> i32 {
        if port > 3 || !x.is_finite() || !y.is_finite() { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_POS, port as u8, 0, x.to_bits(), y.to_bits(), 0))
    });
    // ---- GAP-7 lab RNG
    f!("gd_lab_rng", |c: Caller<'_, PresData>| -> u32 {
        match gate(&c, 0) {
            Ok(l) => l.borrow_mut().rng(),
            Err(_) => 0,
        }
    });
    f!("gd_lab_rng_seed", |c: Caller<'_, PresData>, seed: u32| -> i32 { cmd!(c, 0, LabCmd::new(OP_RNG_SEED, 0, 0, seed, 0, 0)) });
    // ---- B4 overlays (presentation only: local, never a command, never hashed)
    f!("gd_lab_overlay", |c: Caller<'_, PresData>, mask: u32, bits: u32| -> i32 {
        match gate(&c, FEATURE_LAB_OVERLAY) {
            Ok(l) => {
                let mut s = l.borrow_mut();
                for p in 0..4 {
                    if mask >> p & 1 != 0 {
                        s.overlay[p] = bits;
                    }
                }
                GD_OK
            }
            Err(e) => e,
        }
    });
    f!("gd_lab_overlay_style", |c: Caller<'_, PresData>, kind: u32, rgba: u32| -> i32 {
        match gate(&c, FEATURE_LAB_OVERLAY) {
            Ok(l) if (kind as usize) < OV_STYLE_KINDS => {
                l.borrow_mut().style[kind as usize] = rgba;
                GD_OK
            }
            Ok(_) => GD_EINVAL,
            Err(e) => e,
        }
    });
    // ---- B5 read-only queries
    f!("gd_lab_fighter", |mut c: Caller<'_, PresData>, port: u32, out: u32, len: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let Some(Some(f)) = l.borrow().fighters.get(port as usize).copied() else { return GD_ESTATE };
        let n = (len as usize).min(FIGHTER_V1_BYTES);
        if write(&mut c, out, &f[..n]) { n as i32 } else { GD_EINVAL }
    });
    f!("gd_lab_hitboxes", |mut c: Caller<'_, PresData>, port: u32, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 { return GD_EINVAL }
        let v: Vec<u8> = l.borrow().hitboxes[port as usize].iter().take(max as usize).flatten().copied().collect();
        if write(&mut c, out, &v) { (v.len() / HITBOX_BYTES) as i32 } else { GD_EINVAL }
    });
    f!("gd_lab_hurtboxes", |mut c: Caller<'_, PresData>, port: u32, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 { return GD_EINVAL }
        let v: Vec<u8> = l.borrow().hurtboxes[port as usize].iter().take(max as usize).flatten().copied().collect();
        if write(&mut c, out, &v) { (v.len() / HURTBOX_BYTES) as i32 } else { GD_EINVAL }
    });
    // the observed timeline (header: u32 n, i32 first active, i32 last active, i32 IASA; then n cell bytes)
    f!("gd_lab_framedata", |mut c: Caller<'_, PresData>, ch: i32, action: i32, out: u32, len: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let Some(fd) = l.borrow().framedata.get(&(ch, action)).cloned() else { return GD_ESTATE };
        let mut v = Vec::with_capacity(16 + fd.cells.len());
        v.extend_from_slice(&(fd.cells.len() as u32).to_le_bytes());
        v.extend_from_slice(&fd.first_active.to_le_bytes());
        v.extend_from_slice(&fd.last_active.to_le_bytes());
        v.extend_from_slice(&fd.iasa.to_le_bytes());
        v.extend_from_slice(&fd.cells);
        v.truncate(len as usize);
        if write(&mut c, out, &v) { v.len() as i32 } else { GD_EINVAL }
    });
    f!("gd_lab_attr", |c: Caller<'_, PresData>, ch: i32, idx: u32| -> f32 {
        let Ok(l) = gate(&c, 0) else { return f32::NAN };
        let v = l.borrow().attrs.get(&ch).and_then(|a| a.get(idx as usize).copied()).unwrap_or(f32::NAN);
        v
    });
    f!("gd_lab_frame_adv", |c: Caller<'_, PresData>, a: u32, b: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        // the published value is P1 (port 0) relative to P2 (port 1)
        let v = l.borrow().frame_adv;
        if (a, b) == (0, 1) { v } else if (a, b) == (1, 0) { -v } else { 0 }
    });
    f!("gd_lab_events", |mut c: Caller<'_, PresData>, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        // per-frame query: a stack buffer, no allocation (at most 64 events)
        let mut b = [0u8; 64 * EVENT_BYTES];
        let mut n = 0;
        for e in l.borrow().events.iter().take((max as usize).min(64)) {
            b[n * EVENT_BYTES..(n + 1) * EVENT_BYTES].copy_from_slice(&e.to_bytes());
            n += 1;
        }
        if write(&mut c, out, &b[..n * EVENT_BYTES]) { n as i32 } else { GD_EINVAL }
    });
    f!("gd_lab_events_range", |mut c: Caller<'_, PresData>, from: i32, to: i32, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let v: Vec<u8> = l.borrow().history.iter().filter(|e| e.frame >= from && e.frame <= to).take(max as usize).flat_map(|e| e.to_bytes()).collect();
        if write(&mut c, out, &v) { (v.len() / EVENT_BYTES) as i32 } else { GD_EINVAL }
    });
    // the frame meter's data source (GAP-8): the last n cells of a port, oldest first
    f!("gd_lab_meter", |mut c: Caller<'_, PresData>, port: u32, out: u32, n: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 { return GD_EINVAL }
        let mut b = [0u8; METER_CELLS];
        let k = {
            let s = l.borrow();
            let m = &s.meter[port as usize];
            let k = (n as usize).min(m.len()).min(METER_CELLS);
            for (d, v) in b.iter_mut().zip(m.iter().skip(m.len() - k)) {
                *d = *v;
            }
            k
        };
        if write(&mut c, out, &b[..k]) { k as i32 } else { GD_EINVAL }
    });
    // 1 when the lab match has a timer (the retail timer HUD is up), 0 when it is untimed
    f!("gd_lab_timed", |c: Caller<'_, PresData>| -> i32 {
        match gate(&c, 0) { Ok(l) => l.borrow().timed as i32, Err(e) => e }
    });
    // the last frame's applied commands: 20 bytes each = u8 issuer (0 offline / host, 1 guest), 3 zero, the 16-byte record
    f!("gd_lab_applied", |mut c: Caller<'_, PresData>, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let mut b = [0u8; 20 * 2 * LAB_CMDS_PER_FRAME];
        let n = {
            let s = l.borrow();
            let n = s.applied.len().min(max as usize).min(2 * LAB_CMDS_PER_FRAME);
            for (i, (side, cmd)) in s.applied[..n].iter().enumerate() {
                b[i * 20] = *side;
                b[i * 20 + 4..i * 20 + 20].copy_from_slice(&cmd.to_bytes());
            }
            n
        };
        if write(&mut c, out, &b[..n * 20]) { n as i32 } else { GD_EINVAL }
    });
    // status: i32 x 10 = active, match frame, paused, replaying, ring frames, ring length, command delay, recording mask,
    // issuer (0 offline / host, 1 guest), apply frame of a command issued now
    f!("gd_lab_status", |mut c: Caller<'_, PresData>, out: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let s = l.borrow();
        let v = [s.active as i32, s.match_frame, s.paused as i32, s.replaying as i32, s.ring_frames as i32, s.ring_len as i32, s.delay as i32, s.recording_mask as i32, s.issuer as i32, s.apply_frame];
        drop(s);
        let mut b = [0u8; 40];
        for (i, x) in v.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&x.to_le_bytes());
        }
        if write(&mut c, out, &b) { GD_OK } else { GD_EINVAL }
    });
    Ok(())
}

impl PresExtension for LabExt {
    fn feature_bits(&self) -> u32 {
        FEATURES_LAB
    }
    fn link(&self, l: &mut Linker<PresData>) -> Result<(), String> {
        link(l)
    }
    fn store_ext(&self) -> Option<Box<dyn std::any::Any>> {
        Some(Box::new(self.shared.clone()))
    }
    fn on_session(&mut self, s: Session) {
        self.shared.borrow_mut().set_session(s);
    }
    // the input boundary work (recording, playback, synthesised dummy input) needs the sim, so it runs in the play
    // flow's lab run (melee/src/lab.rs) on the pads this boundary produced; the overlay pass is drawn there too
    fn before_logic(&mut self, _f: &PresFrame, _io: &mut dyn InputBoundary) {}
    fn after_logic(&mut self, _f: &PresFrame) {}
    fn overlay(&mut self, _f: &PresFrame, _out: &mut Vec<melee_gx::draw::DrawCmd>) {}
}

/// True when a module imports from `gd_lab` (the presentation host then installs the Lab extension).
pub fn imports_lab(bytes: &[u8]) -> bool {
    let engine = wasmi::Engine::new(&crate::engine_config());
    wasmi::Module::new(&engine, bytes).is_ok_and(|m| m.imports().any(|i| i.module() == "gd_lab"))
}

#[cfg(test)]
mod tests;

/// Whether `MELEE_WASM_MODS` holds a Lab mod (the play flow then runs in the snapshot-capable allocator modes).
pub fn lab_mod_in_env() -> bool {
    let Some(dir) = crate::dir_from_env() else { return false };
    let Ok(rd) = std::fs::read_dir(&dir) else { return false };
    rd.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wasm")))
        .any(|p| std::fs::read(&p).is_ok_and(|b| imports_lab(&b)))
}
