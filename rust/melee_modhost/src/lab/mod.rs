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
/// Drills (lab-theorycraft.md 8 C / 4b): `port` = the dummy, `a` = drill index (DRILL_STOP stops), `b` = the recording
/// handles of Record slots 1-4 (a byte each; a drill may pick among them). The attempt loop, the picks (lab RNG) and the
/// weight updates then run sim-side on both peers from the same events: no further commands.
pub const OP_DRILL: u8 = 20;
/// Adaptive trainer mode for the running drill: `a` = ADAPT_*, `b` = rate (weight added per failure).
pub const OP_ADAPT: u8 = 21;
/// Mirror and jitter on load (8 K): `port` = the varied fighter, `a` = VARY_* flags, `b` = percent range, `c` = x range.
pub const OP_LOAD_VARY: u8 = 22;
/// TM-CE dummy details (8 J): `a` = DO_*, `b` = value.
pub const OP_DUMMY_OPT: u8 = 23;

pub const DRILL_STOP: u16 = 0xFFFF;
/// Drills compiled in (the same on every peer; the only ones the online LAB accepts).
pub const DRILL_BUILTIN: usize = 5;
pub const ADAPT_OFF: u16 = 0;
/// Weights move toward the options the user fails against ("weak spots").
pub const ADAPT_WEAK: u16 = 1;
pub const VARY_MIRROR: u16 = 1;
pub const VARY_MIRROR_RANDOM: u16 = 2;
/// Frames after the counter action starts before the last slot reloads (0 off).
pub const DO_AUTO_RESTORE: u16 = 1;
/// Chance (percent) the dummy presses tech early and is locked out (a missed tech by the game's own 40-frame rule).
pub const DO_TECH_LOCKOUT: u16 = 2;
/// SDI pushes per hit (0 = one every interval for the whole hitlag).
pub const DO_SDI_COUNT: u16 = 3;
/// Drill status (i32): active, drill, phase (0 waiting for the trigger, 1 in the window, 2 showing the result), attempts,
/// successes, result sequence, last verdict (DV_*), last frames late, last option, options, adapt mode, streak, best
/// streak, late sum, late count, error (DE_*), then weights[8], attempts per option[8], successes per option[8].
pub const DRILL_I32: usize = 40;
pub const DV_NONE: i32 = 0;
pub const DV_SUCCESS: i32 = 1;
pub const DV_MISSED: i32 = 2;
pub const DV_PUNISHED: i32 = 3;
pub const DE_OK: i32 = 0;
pub const DE_NEEDS_REC: i32 = 1;
pub const DE_NO_DRILL: i32 = 2;
// Geno (gd-lab-v1.1, docs/rust-port/notes/geno-wasm-plan.md step 4): offline only, as the C's gs_require_offline. The
// presentation side never queues them in LabNetplay and the play flow ignores them there, so a v1 peer never meets
// one and LAB_CMD_VERSION stays 1.
/// port, a = mode (0 off, 1 on, 2 place): `GenoFly_Set`
pub const OP_GENO_FLY: u8 = 24;
/// port, b = x bits, c = y bits: `GenoFly_Teleport`
pub const OP_GENO_TELEPORT: u8 = 25;
/// port, a = flags (bit 0 on, bit 1 every frame) | damage << 2 | angle << 7, b = radius bits, c = active | gap << 16,
/// d = kb | kbg << 16: `GenoFly_AttackConfig` (on: the fighter is put in flight first, as gs_fly_attack_cfg)
pub const OP_GENO_FLY_ATTACK: u8 = 26;
/// port: `GenoFly_Clear` (the cursor's target and attack)
pub const OP_GENO_FLY_CLEAR: u8 = 27;
/// port, b = ki (the meter's hundredths, as `GenoGame_ComboSetKi`)
pub const OP_GENO_KI: u8 = 28;
/// `gw_Geno_Reload` (then `GenoGame_LabReload`, or the restart when the layout changed); the text is published
pub const OP_GENO_RELOAD: u8 = 29;
/// The last opcode `LabCmd::from_bytes` accepts.
pub const OP_LAST: u8 = OP_GENO_RELOAD;

pub fn is_geno_op(op: u8) -> bool {
    (OP_GENO_FLY..=OP_GENO_RELOAD).contains(&op)
}

// ---- Geno queries (published by the play flow: GenoView)
/// GENO_COMBO_MAX_ROUTES (melee_script::geno::consts).
pub const GENO_ROUTES_MAX: usize = 128;
/// `gd_geno_combo_info` what codes: 0..15 as `GenoGame_ComboInfo`, then 16 + route for the route states.
pub const GENO_COMBO_WHATS: usize = 16 + GENO_ROUTES_MAX;
/// `GenoFly_State` fields 0..13.
pub const GENO_FLY_FIELDS: usize = 14;
/// `gd_geno_result` header bytes: u32 seq, i32 op, i32 port, i32 rc.
pub const GENO_RESULT_HEAD: usize = 16;
/// The fly cursor's coordinate bound (the C's fly_target check).
pub const GENO_FLY_COORD_MAX: f32 = 49000.0;

// ---- B1..B3 constants (docs/specs/gd-ui-lab-v1.h)
pub const SLOTS: usize = 16;

// ---- lookahead (docs/rust-port/notes/lab-theorycraft.md 3b / 4c): save, tick under the speculative flag until each
// fighter can act (cap LOOK_CAP frames), load. Local only: a request is never a command, the result never touches the sim.
/// `gd_lab_lookahead(kind, port)`: `port` is the defender (the one hit / shielding); the attacker is the other of P1 / P2.
/// Hit: the no-DI run (frame advantage on hit) and the 8-direction DI fan (KO / survive and advantage per direction).
pub const LOOK_HIT: u32 = 0;
/// Shield: the defender holds shield, the attacker is neutral (frame advantage on shield).
pub const LOOK_SHIELD: u32 = 1;
/// Both neutral from now (frame advantage of the current exchange, whatever it is).
pub const LOOK_NEUTRAL: u32 = 2;
/// RNG sweep: the neutral run across LOOK_RUNS game seeds (the outcome spread of the random draws from here).
pub const LOOK_RNG: u32 = 3;
/// `gd_lab_lookahead_cfg` flags: run automatically on hit / shield events; run while playing (else only when frozen).
pub const LOOK_AUTO: u32 = 1;
pub const LOOK_LIVE: u32 = 2;
pub const LOOK_CAP: i32 = 300;
pub const LOOK_RUNS: usize = 9;
/// Result header (i32): version, state (LS_*), kind, anchor match frame, generation, attacker, defender, advantage
/// (attacker relative to defender, from the no-DI / neutral / shield run; i32::MIN when not known), runs, runs done,
/// ticks, cost us, slices, longest slice us, snapshot-verify mismatches, 0.
pub const LOOK_HEAD: usize = 16;
/// Per run (i32): tag (DI as a numpad direction 1-9, 5 = none; RNG: the seed index), outcome (LO_*), defender frame
/// (first actionable, or the KO frame; frames after the anchor), attacker frame (first actionable, -1 none).
pub const LOOK_RUN_I32: usize = 4;
pub const LOOK_I32: usize = LOOK_HEAD + LOOK_RUNS * LOOK_RUN_I32;
pub const LS_NONE: i32 = 0;
pub const LS_PENDING: i32 = 1;
pub const LS_RUNNING: i32 = 2;
pub const LS_DONE: i32 = 3;
pub const LO_NOT_RUN: i32 = 0;
pub const LO_ACTS: i32 = 1;
pub const LO_KO: i32 = 2;
pub const LO_CAP: i32 = 3;
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
/// TM-CE counter action: once the dummy can act again after a hit (or shield stun).
pub const RK_COUNTER: i32 = 7;
pub const RK_COUNT: usize = 8;

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
// COUNTER
pub const RO_COUNTER_JUMP: i32 = 1;
pub const RO_COUNTER_ATTACK: i32 = 2;
pub const RO_COUNTER_SHIELD: i32 = 3;
pub const RO_COUNTER_SPOTDODGE: i32 = 4;
pub const RO_COUNTER_GRAB: i32 = 5;
/// Random out of shield (RO_RANDOM is 5, which is spot-dodge here).
pub const RO_OOS_RANDOM: i32 = 8;
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
        if b.len() < LAB_CMD_BYTES || b[0] == 0 || b[0] > OP_LAST {
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

/// The result of the last applied Geno command (`gd_geno_result`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenoResult {
    /// Applied Geno commands so far (0: none yet).
    pub seq: u32,
    pub op: u8,
    pub port: u8,
    /// The game function's code (reload: `gw_Geno_Reload`'s, <0 refused, 0 layout changed, >0 re-applied).
    pub rc: i32,
    /// Why, as the C LAB says it (reload: the hot-reload line; fly: gs_fly_err); empty on plain success.
    pub text: String,
}

/// What the play flow publishes for the `gd_geno_*` queries: the per-port part after every frame, the registry tables
/// (names, counts) when the lab run starts and after a reload. The tables are registry data, not sim state, but the
/// presentation side still reads only what is published here (the file's rule; melee_modhost does not link the
/// registry).
#[derive(Clone, Debug)]
pub struct GenoView {
    /// Bumped every time the tables are republished.
    pub tables_gen: u32,
    /// `gw_Geno_ProfileForKind(fp->kind)` per port (-1 none or no fighter).
    pub profile: [i32; 4],
    /// `GenoGame_ComboInfo(port, what)` per port and what code (-1 where it answers -1).
    pub combo: [[i32; GENO_COMBO_WHATS]; 4],
    /// `GenoFly_Get` per port (-1 no fighter).
    pub fly_mode: [i32; 4],
    /// `GenoFly_State(port, field)`.
    pub fly: [[i32; GENO_FLY_FIELDS]; 4],
    /// State names by profile (`gw_Geno_StateName`; the count is the length).
    pub states: Vec<Vec<String>>,
    /// Route names by profile (`gw_Geno_ComboName`, the registry's route order).
    pub routes: Vec<Vec<String>>,
    /// Define display names by CSS kind (`gw_Geno_DefineName`), None where it answers 0.
    pub defines: Vec<Option<String>>,
    pub result: GenoResult,
}

impl Default for GenoView {
    fn default() -> Self {
        GenoView {
            tables_gen: 0,
            profile: [-1; 4],
            combo: [[-1; GENO_COMBO_WHATS]; 4],
            fly_mode: [-1; 4],
            fly: [[-1; GENO_FLY_FIELDS]; 4],
            states: Vec::new(),
            routes: Vec::new(),
            defines: Vec::new(),
            result: GenoResult::default(),
        }
    }
}

impl GenoView {
    /// No fighter on a port (the lab run ended, or the port is empty).
    pub fn clear_port(&mut self, port: usize) {
        self.profile[port] = -1;
        self.combo[port] = [-1; GENO_COMBO_WHATS];
        self.fly_mode[port] = -1;
        self.fly[port] = [-1; GENO_FLY_FIELDS];
    }
    pub fn state_name(&self, p: i32, s: i32) -> Option<&str> {
        self.states.get(usize::try_from(p).ok()?)?.get(usize::try_from(s).ok()?).map(String::as_str)
    }
    pub fn route_name(&self, p: i32, r: i32) -> Option<&str> {
        self.routes.get(usize::try_from(p).ok()?)?.get(usize::try_from(r).ok()?).map(String::as_str)
    }
    pub fn define_name(&self, ck: i32) -> Option<&str> {
        self.defines.get(usize::try_from(ck).ok()?)?.as_deref()
    }
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
    // ---- lookahead (local, presentation-side: never a command)
    pub look_cfg: u32,
    pub look_budget_us: u32,
    /// A requested lookahead (kind, defender port), taken by the play flow's lab run.
    pub look_req: Option<(u32, u32)>,
    /// The latest result (LOOK_HEAD + runs).
    pub look: [i32; LOOK_I32],
    // ---- drills (published by the lab run; the names, labels and trend are presentation-side reads)
    pub drill_names: Vec<String>,
    /// The running drill's option labels.
    pub drill_opts: Vec<String>,
    pub drill: [i32; DRILL_I32],
    /// Earlier sessions of the running drill from the session log: (success per mille, attempts), oldest first.
    pub drill_trend: Vec<(i32, i32)>,
    // ---- Geno (published by the play flow)
    pub geno: GenoView,
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
            look_cfg: LOOK_AUTO,
            look_budget_us: 4000,
            look_req: None,
            look: [0; LOOK_I32],
            drill_names: Vec::new(),
            drill_opts: Vec::new(),
            drill: [0; DRILL_I32],
            drill_trend: Vec::new(),
            geno: GenoView::default(),
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
            // a lookahead belongs to the session it was asked in
            self.look_req = None;
            self.look = [0; LOOK_I32];
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

/// Copy a published string into the mod's memory, truncated to `cap` (no NUL, as gd_ui_list_get): the bytes written;
/// GD_EINVAL when there is no such string or the buffer is outside the memory.
fn put_str(c: &mut Caller<'_, PresData>, out: u32, cap: u32, s: Option<&str>) -> i32 {
    let Some(s) = s else { return GD_EINVAL };
    let b = &s.as_bytes()[..s.len().min(cap as usize)];
    if write(c, out, b) { b.len() as i32 } else { GD_EINVAL }
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

/// A Geno command: the gate, then offline only (the C refuses these online: gs_require_offline).
macro_rules! geno_cmd {
    ($c:expr, $sub:expr, $cmd:expr) => {
        match gate(&$c, $sub) {
            Ok(l) => {
                let mut s = l.borrow_mut();
                if s.session != Session::Offline { GD_EOFFLINE } else { s.push($cmd) }
            }
            Err(e) => e,
        }
    };
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
    // lookahead (local only: nothing goes to the command stream, so it is the same online and offline)
    f!("gd_lab_lookahead", |c: Caller<'_, PresData>, kind: u32, port: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if kind > LOOK_RNG || port > 1 { return GD_EINVAL }
        let mut s = l.borrow_mut();
        if !s.active { return GD_ESTATE }
        s.look_req = Some((kind, port));
        GD_OK
    });
    f!("gd_lab_lookahead_cfg", |c: Caller<'_, PresData>, flags: u32, budget_us: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let mut s = l.borrow_mut();
        s.look_cfg = flags & (LOOK_AUTO | LOOK_LIVE);
        s.look_budget_us = budget_us.clamp(500, 16_000);
        GD_OK
    });
    // the latest lookahead result: LOOK_I32 little-endian i32 (header, then LOOK_RUNS runs); bytes written
    f!("gd_lab_lookahead_result", |mut c: Caller<'_, PresData>, out: u32, len: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let mut b = [0u8; LOOK_I32 * 4];
        for (i, v) in l.borrow().look.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let n = (len as usize).min(b.len());
        if write(&mut c, out, &b[..n]) { n as i32 } else { GD_EINVAL }
    });
    // ---- drills, the adaptive trainer, mirror / jitter on load, TM-CE dummy details (lab-theorycraft.md 4b, 8 C J K)
    f!("gd_lab_drill", |c: Caller<'_, PresData>, index: u32, port: u32, recs: u32| -> i32 {
        if port > 3 { return GD_EINVAL }
        if index != DRILL_STOP as u32 {
            let l = match gate(&c, FEATURE_LAB_CPU) { Ok(l) => l, Err(e) => return e };
            let s = l.borrow();
            // online only the compiled-in drills: a drill file is local, the other peer may not have it
            if index as usize >= s.drill_names.len() || (s.session == Session::LabNetplay && index as usize >= DRILL_BUILTIN) { return GD_EINVAL }
        }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_DRILL, port as u8, index.min(0xFFFF) as u16, recs, 0, 0))
    });
    f!("gd_lab_adapt", |c: Caller<'_, PresData>, mode: u32, rate: u32| -> i32 {
        if mode > ADAPT_WEAK as u32 || rate > 15 { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_ADAPT, 0, mode as u16, rate, 0, 0))
    });
    f!("gd_lab_load_vary", |c: Caller<'_, PresData>, port: u32, flags: u32, pct: u32, x: u32| -> i32 {
        if port > 3 || flags > 3 || pct > 300 || x > 200 { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_LOAD_VARY, port as u8, flags as u16, pct, x, 0))
    });
    f!("gd_lab_dummy_opt", |c: Caller<'_, PresData>, port: u32, opt: u32, value: u32| -> i32 {
        if port > 3 || !(1..=DO_SDI_COUNT as u32).contains(&opt) || value > 600 { return GD_EINVAL }
        cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_DUMMY_OPT, port as u8, opt as u16, value, 0, 0))
    });
    f!("gd_lab_drill_count", |c: Caller<'_, PresData>| -> i32 {
        match gate(&c, 0) { Ok(l) => l.borrow().drill_names.len() as i32, Err(e) => e }
    });
    // text: kind 0 = drill name i, 1 = the running drill's option label i; bytes written (UTF-8)
    f!("gd_lab_drill_text", |mut c: Caller<'_, PresData>, kind: u32, i: u32, out: u32, len: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let b = {
            let s = l.borrow();
            let Some(t) = (match kind { 0 => s.drill_names.get(i as usize), 1 => s.drill_opts.get(i as usize), _ => None }) else { return GD_EINVAL };
            let mut n = t.len().min(len as usize);
            while !t.is_char_boundary(n) { n -= 1; }
            t.as_bytes()[..n].to_vec()
        };
        if write(&mut c, out, &b) { b.len() as i32 } else { GD_EINVAL }
    });
    f!("gd_lab_drill_status", |mut c: Caller<'_, PresData>, out: u32, len: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let mut b = [0u8; DRILL_I32 * 4];
        for (i, v) in l.borrow().drill.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        let n = (len as usize).min(b.len());
        if write(&mut c, out, &b[..n]) { n as i32 } else { GD_EINVAL }
    });
    // earlier sessions of the running drill: (i32 success per mille, i32 attempts) pairs, oldest first; pairs written
    f!("gd_lab_drill_trend", |mut c: Caller<'_, PresData>, out: u32, max: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let v: Vec<u8> = {
            let s = l.borrow();
            let k = s.drill_trend.len().saturating_sub(max.min(16) as usize);
            s.drill_trend[k..].iter().flat_map(|&(a, b)| a.to_le_bytes().into_iter().chain(b.to_le_bytes())).collect()
        };
        if write(&mut c, out, &v) { (v.len() / 8) as i32 } else { GD_EINVAL }
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
    // ---- Geno (gd-lab-v1.1): queries copy from LabShared::geno; writes are offline-only commands
    f!("gd_geno_profile", |c: Caller<'_, PresData>, port: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 { return GD_EINVAL }
        let v = l.borrow().geno.profile[port as usize];
        v
    });
    f!("gd_geno_state_count", |c: Caller<'_, PresData>, profile: i32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let v = usize::try_from(profile).ok().and_then(|p| l.borrow().geno.states.get(p).map(|v| v.len() as i32)).unwrap_or(0);
        v
    });
    f!("gd_geno_state_name", |mut c: Caller<'_, PresData>, profile: i32, s: i32, out: u32, cap: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let sh = l.borrow();
        put_str(&mut c, out, cap, sh.geno.state_name(profile, s))
    });
    f!("gd_geno_define_name", |mut c: Caller<'_, PresData>, ck: i32, out: u32, cap: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let sh = l.borrow();
        put_str(&mut c, out, cap, sh.geno.define_name(ck))
    });
    f!("gd_geno_combo_info", |c: Caller<'_, PresData>, port: u32, what: i32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 || !(0..GENO_COMBO_WHATS as i32).contains(&what) { return GD_EINVAL }
        let v = l.borrow().geno.combo[port as usize][what as usize];
        v
    });
    f!("gd_geno_combo_route_name", |mut c: Caller<'_, PresData>, profile: i32, route: i32, out: u32, cap: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let sh = l.borrow();
        put_str(&mut c, out, cap, sh.geno.route_name(profile, route))
    });
    f!("gd_geno_fly_get", |c: Caller<'_, PresData>, port: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 { return GD_EINVAL }
        let v = l.borrow().geno.fly_mode[port as usize];
        v
    });
    f!("gd_geno_fly_state", |c: Caller<'_, PresData>, port: u32, field: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        if port > 3 || field as usize >= GENO_FLY_FIELDS { return GD_EINVAL }
        let v = l.borrow().geno.fly[port as usize][field as usize];
        v
    });
    // the last applied Geno command: u32 seq, i32 op, i32 port, i32 rc, then its text; truncated to cap; bytes written
    f!("gd_geno_result", |mut c: Caller<'_, PresData>, out: u32, cap: u32| -> i32 {
        let l = match gate(&c, 0) { Ok(l) => l, Err(e) => return e };
        let b = {
            let sh = l.borrow();
            let r = &sh.geno.result;
            if r.seq == 0 { return GD_ESTATE }
            let mut b = Vec::with_capacity(GENO_RESULT_HEAD + r.text.len());
            b.extend_from_slice(&r.seq.to_le_bytes());
            b.extend_from_slice(&(r.op as i32).to_le_bytes());
            b.extend_from_slice(&(r.port as i32).to_le_bytes());
            b.extend_from_slice(&r.rc.to_le_bytes());
            b.extend_from_slice(r.text.as_bytes());
            b.truncate(cap as usize);
            b
        };
        if write(&mut c, out, &b) { b.len() as i32 } else { GD_EINVAL }
    });
    f!("gd_geno_fly", |c: Caller<'_, PresData>, port: u32, mode: i32| -> i32 {
        if port > 3 || !(0..=2).contains(&mode) { return GD_EINVAL }
        geno_cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_GENO_FLY, port as u8, mode as u16, 0, 0, 0))
    });
    f!("gd_geno_teleport", |c: Caller<'_, PresData>, port: u32, x: f32, y: f32| -> i32 {
        let ok = |v: f32| v.is_finite() && v.abs() <= GENO_FLY_COORD_MAX;
        if port > 3 || !ok(x) || !ok(y) { return GD_EINVAL }
        geno_cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_GENO_TELEPORT, port as u8, 0, x.to_bits(), y.to_bits(), 0))
    });
    // flags: 1 on (0 = attack off), 2 every frame; damage 1..30, radius 1..30, active 0..600, gap 0..3000, kb / kbg
    // 0..2000, angle 0..361 (0 = the game's default): the ranges GenoFly_AttackConfig accepts
    f!("gd_geno_fly_attack", |c: Caller<'_, PresData>, port: u32, flags: u32, damage: u32, radius: f32, active: u32, gap: u32, kb: u32, kbg: u32, angle: u32| -> i32 {
        match geno_attack_cmd(port, flags, damage, radius, active, gap, kb, kbg, angle) {
            Some(cmd) => geno_cmd!(c, FEATURE_LAB_CPU, cmd),
            None => GD_EINVAL,
        }
    });
    f!("gd_geno_fly_clear", |c: Caller<'_, PresData>, port: u32| -> i32 {
        if port > 3 { return GD_EINVAL }
        geno_cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_GENO_FLY_CLEAR, port as u8, 0, 0, 0, 0))
    });
    f!("gd_geno_combo_ki", |c: Caller<'_, PresData>, port: u32, ki: i32| -> i32 {
        if port > 3 || ki < 0 { return GD_EINVAL }
        geno_cmd!(c, FEATURE_LAB_CPU, LabCmd::new(OP_GENO_KI, port as u8, 0, ki as u32, 0, 0))
    });
    f!("gd_geno_reload", |c: Caller<'_, PresData>| -> i32 { geno_cmd!(c, FEATURE_LAB_STATE, LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0)) });
    Ok(())
}

/// `gd_geno_fly_attack`'s command (None: out of the ranges GenoFly_AttackConfig accepts). With the attack off only
/// the flags travel.
#[allow(clippy::too_many_arguments)]
pub fn geno_attack_cmd(port: u32, flags: u32, damage: u32, radius: f32, active: u32, gap: u32, kb: u32, kbg: u32, angle: u32) -> Option<LabCmd> {
    if port > 3 || flags > 3 {
        return None;
    }
    if flags & 1 == 0 {
        return Some(LabCmd::new(OP_GENO_FLY_ATTACK, port as u8, flags as u16, 0, 0, 0));
    }
    if !(1..=30).contains(&damage) || !(1.0..=30.0).contains(&radius) || active > 600 || gap > 3000 || kb > 2000 || kbg > 2000 || angle > 361 {
        return None;
    }
    Some(LabCmd::new(OP_GENO_FLY_ATTACK, port as u8, (flags | damage << 2 | angle << 7) as u16, radius.to_bits(), active | gap << 16, kb | kbg << 16))
}

/// The arguments of `GenoFly_AttackConfig` an OP_GENO_FLY_ATTACK carries: (flags, damage, radius bits, active_gap,
/// kb_kbg, angle).
pub fn geno_attack_args(c: &LabCmd) -> (i32, i32, i32, i32, i32, i32) {
    let a = c.a as i32;
    (a & 3, a >> 2 & 0x1F, c.b as i32, c.c as i32, c.d as i32, a >> 7 & 0x1FF)
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
