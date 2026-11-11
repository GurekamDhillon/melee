//! The script runner: threads for the per-motion timelines, the status scripts and the cages' linked scripts; the
//! control commands; the rule engine; the 16 built-ins. Integer / fixed-point only.
//!
//! One frame ([`Runner::step`]):
//! 1. the frame input is written into the engine-maintained variables (HP now / before, hit by a player);
//! 2. the rule engine runs (format spec 4.3) and may change status (the enter script runs at once);
//! 3. the status loop thread runs;
//! 4. the boss's nine timeline threads run (game 0-2, effect 0-2, sound 0-2), then the two cages' linked threads;
//! 5. the frame counter advances (motion frames follow it: 1 animation frame per game frame).
//!
//! The world does movement before calling `step` (the spec puts the rules "after movement").

use crate::file::{BossFile, COMMON_SLOT, STATUS_SLOTS};
use crate::fx::{Fx, ONE};
use crate::level::level;
use crate::rng::{below, RandomSource};
use crate::script::{
    req, Command, Cond, Op, Value, Var, KIND_FLAG, KIND_FLOAT, KIND_INT, MEM_PARAM, MEM_TEMP, MEM_WORK,
};
use crate::script::Target;
use crate::world::{Builtin, DamageCheck, Event, Obj, ResolvedHitbox, Stamp, World};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

/// How 0002 (wait until motion frame f) behaves inside a loop (format spec 3.12 / 9 item 8, uncertain).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncWaitInLoop {
    /// Default: always the absolute motion frame (a frame already passed does not wait). This is the only reading
    /// that keeps the basket's cage hitboxes on frames 70-114 as observed (spec.md 8): with the relative reading the
    /// basket's `20 x {shake, wait-until 6}` would run to frame 190 and push the frame-115 clear past the motion's end.
    Absolute,
    /// The format spec's "+6 frames" reading: relative to the start of the innermost loop's current iteration.
    IterationRelative,
}

/// When the weighted switch truncates weights to integers (format spec 4.3 step 5, uncertain [L]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightRounding {
    /// Default: no truncation; weights are exact fixed-point (equivalent to float weights).
    Exact,
    /// base + slope * L is truncated before the entry's condition (the 0.3 cut) runs.
    TruncateBeforeCut,
    /// Truncated after the condition.
    TruncateAfterCut,
}

/// How the rule engine breaks ties (format spec 4.3 steps 3-4, uncertain [M]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleTieBreak {
    /// Default: among the lowest-priority true (rule, slot) pairs, if the pairs with a slot span two or more slots,
    /// draw one slot (one RNG draw, tuning weights, repeat cut on the last used slot); keep slotless pairs and pairs
    /// with the drawn slot; take the first in table order.
    SlotDrawThenTableOrder,
    /// No slot draw: the first pair in table order (then slot order) wins.
    TableOrderOnly,
}

/// Runner options. Each uncertain reading of the spec is a named switch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Difficulty index 0-14 (a fresh Normal game is 5).
    pub difficulty: i32,
    /// Strength at placement (1).
    pub strength: i32,
    /// The status entered after the init script. The spec does not say which status the fight opens in; the flow it
    /// describes starts at Wait A (2).
    pub initial_status: i32,
    pub async_wait_in_loop: AsyncWaitInLoop,
    pub weight_rounding: WeightRounding,
    pub tie_break: RuleTieBreak,
    /// A status table slot that is null falls back to the common slot 16 (enter / loop / exit).
    pub common_slot_fallback: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            difficulty: 5,
            strength: 1,
            initial_status: 2,
            async_wait_in_loop: AsyncWaitInLoop::Absolute,
            weight_rounding: WeightRounding::Exact,
            tie_break: RuleTieBreak::SlotDrawThenTableOrder,
            common_slot_fallback: true,
        }
    }
}

/// What the game tells the runner at the start of each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameInput {
    /// Body HP now (float 1) and at the start of the frame (float 14).
    pub hp_now: Fx,
    pub hp_before: Fx,
    /// A player's attack damaged the boss this frame (flag 18).
    pub hit_by_player: bool,
}

/// Cage to boss events (format spec 6), handled natively.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CageEvent {
    /// Event 1: the cage was damaged this frame.
    Damaged,
    /// 0x1400: HP% <= the phase-1 threshold with counter 0.
    PhaseOne,
    /// 0x1401: HP% <= the phase-2 threshold with counter 1.
    PhaseTwo,
    /// 0x13FF: HP reached 0.
    Broken,
    /// 0x13F7: resync the body's motion frame to the cage's.
    Resync,
}

/// Work variables of one object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vars {
    pub ints: BTreeMap<u32, i32>,
    pub floats: BTreeMap<u32, Fx>,
    pub flags: BTreeSet<u32>,
}
impl Vars {
    pub fn int(&self, id: u32) -> i32 {
        self.ints.get(&id).copied().unwrap_or(0)
    }
    pub fn float(&self, id: u32) -> Fx {
        self.floats.get(&id).copied().unwrap_or(Fx::ZERO)
    }
    pub fn flag(&self, id: u32) -> bool {
        self.flags.contains(&id)
    }
    fn read(&self, kind: u8, id: u32) -> Fx {
        match kind {
            KIND_INT => Fx::from_int(self.int(id) as i64),
            KIND_FLOAT => self.float(id),
            _ => Fx::from_int(self.flag(id) as i64),
        }
    }
    fn write(&mut self, kind: u8, id: u32, v: Fx) {
        match kind {
            KIND_INT => {
                self.ints.insert(id, v.trunc() as i32);
            }
            KIND_FLOAT => {
                self.floats.insert(id, v);
            }
            _ => {
                if v != Fx::ZERO {
                    self.flags.insert(id);
                } else {
                    self.flags.remove(&id);
                }
            }
        }
    }
}

/// Named boss work variables (format spec 3.2).
pub mod var {
    pub const INT_SIDE: u32 = 1;
    pub const INT_JUMP_COUNT: u32 = 5;
    pub const INT_MENU_PICK: u32 = 16;
    pub const INT_JUMPS_WANTED: u32 = 17;
    pub const INT_PHASE: u32 = 18;
    pub const FLOAT_HP: u32 = 1;
    pub const FLOAT_TIMEOUT: u32 = 2;
    pub const FLOAT_AIM: u32 = 4;
    pub const FLOAT_DMG_OUT: u32 = 7;
    pub const FLOAT_MIN_CAGE: u32 = 8;
    pub const FLOAT_TARGET_X: u32 = 10;
    pub const FLOAT_LEVEL: u32 = 12;
    pub const FLOAT_HP_BEFORE: u32 = 14;
    /// On a cage: its own L.
    pub const CAGE_FLOAT_LEVEL: u32 = 0;
    pub const FLAG_CAGE_BROKEN: u32 = 6;
    pub const FLAG_LOWER_IS_LEFT: u32 = 7;
    pub const FLAG_HIT_BY_PLAYER: u32 = 18;
    pub const TEMP_INT_SLOT: u32 = 0;
    pub const TEMP_INT_ENTRY: u32 = 6;
    pub const TEMP_FLOAT_WEIGHT: u32 = 0;
    pub const TEMP_FLAG_TRUE: u32 = 4;
}

/// Engine value ids (memory 0) the runner answers itself.
pub mod value {
    pub const STATUS: u32 = 20001;
    pub const PREV_STATUS: u32 = 20003;
    pub const STATUS_FRAME: u32 = 21002;
    pub const TARGET_HANDLE: u32 = 21004;
    pub const BOUNCE: u32 = 3056;
    pub const DEATH_THRESHOLD: u32 = 3057;
}

/// Link events (format spec 3.8).
pub mod link {
    pub const SET_PROC: i32 = 5103;
    pub const MOTION: i32 = 5104;
    pub const DAMAGE: i32 = 5105;
    pub const COUNTER: i32 = 5106;
    pub const LEVEL: i32 = 5112;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Code {
    File(u32),
    BossDown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LoopSt {
    body: usize,
    left: i32,
    iter_start: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    code: Code,
    pc: usize,
    loops: Vec<LoopSt>,
}

/// A running script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thread {
    obj: Obj,
    frames: Vec<Frame>,
    /// Resume when tick * ONE >= resume.
    resume: i64,
}
impl Thread {
    fn new(obj: Obj, code: Code) -> Thread {
        Thread { obj, frames: vec![Frame { code, pc: 0, loops: Vec::new() }], resume: i64::MIN }
    }
    pub fn done(&self) -> bool {
        self.frames.is_empty()
    }
}

/// Per-cage state the runner keeps (the rest of the cage is native and lives in the game).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cage {
    pub spawned: bool,
    pub proc_: i32,
    pub counter: i32,
    pub motion: i32,
    motion_t0: u32,
    pub vars: Vars,
    thread: Option<Thread>,
}

const BUDGET: u32 = 100_000;
const MAX_STATUS_CHAIN: u32 = 8;

/// The boss.
#[derive(Clone, Debug)]
pub struct Runner<'f> {
    pub file: &'f BossFile,
    pub config: Config,
    builtins: Vec<Option<Builtin>>,
    tick: u32,
    pub status: i32,
    pub prev_status: i32,
    status_t0: u32,
    status_gen: u32,
    pub motion: i32,
    motion_t0: u32,
    motion_gen: u32,
    pub strength: i32,
    pub vars: Vars,
    pub temps: Vars,
    pub cages: [Cage; 2],
    rules_on: Vec<bool>,
    /// The locked target slot.
    pub lock: Option<i32>,
    /// The slot the rule engine last used (for the repeat cut).
    pub last_slot: Option<i32>,
    /// The slot being evaluated (temp int 0 when it was set by the rule engine).
    eval_slot: Option<i32>,
    pending_status: Option<i32>,
    pub dead: bool,
    broken_seen: [bool; 2],
    loop_thread: Option<Thread>,
    timelines: [Option<Thread>; 9],
    link_depth: u32,
}

struct Io<'a> {
    w: &'a mut dyn World,
    r: &'a mut dyn RandomSource,
}

impl<'f> Runner<'f> {
    pub fn new(file: &'f BossFile, config: Config) -> Runner<'f> {
        let builtins = file.imports.iter().map(|i| Builtin::from_name(&i.name)).collect();
        Runner {
            file,
            config,
            builtins,
            tick: 0,
            status: -1,
            prev_status: -1,
            status_t0: 0,
            status_gen: 0,
            motion: -1,
            motion_t0: 0,
            motion_gen: 0,
            strength: config.strength,
            vars: Vars::default(),
            temps: Vars::default(),
            cages: Default::default(),
            rules_on: file.rules.iter().map(|r| r.default_enabled).collect(),
            lock: None,
            last_slot: None,
            eval_slot: None,
            pending_status: None,
            dead: false,
            broken_seen: [false; 2],
            loop_thread: None,
            timelines: Default::default(),
            link_depth: 0,
        }
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }
    pub fn rule_enabled(&self, i: usize) -> bool {
        self.rules_on.get(i).copied().unwrap_or(false)
    }
    pub fn level(&self) -> Fx {
        self.vars.float(var::FLOAT_LEVEL)
    }
    /// Frames since the current status was entered.
    pub fn status_frame(&self) -> u32 {
        self.tick - self.status_t0
    }
    pub fn motion_elapsed(&self) -> u32 {
        self.tick - self.motion_t0
    }

    pub fn set_rule_enabled(&mut self, i: usize, on: bool) {
        if let Some(r) = self.rules_on.get_mut(i) {
            *r = on;
        }
    }

    /// Run the weighted switch over list `list` with `count` entries, as command 0010 would (one draw unless every
    /// weight is zero).
    pub fn weighted_pick(&mut self, w: &mut dyn World, r: &mut dyn RandomSource, list: Var, count: i32) -> Option<i32> {
        let mut io = Io { w, r };
        self.switch_pick(&mut io, Obj::Boss, list, count)
    }

    /// Set L directly (tests and tools; the game sets it through strength).
    pub fn set_level(&mut self, l: Fx) {
        self.vars.write(KIND_FLOAT, var::FLOAT_LEVEL, l);
    }

    /// Play a motion from frame 0 (restarting its timelines), as 0409 does.
    pub fn play_motion(&mut self, w: &mut dyn World, r: &mut dyn RandomSource, m: i32) {
        let mut io = Io { w, r };
        self.change_motion(&mut io, m);
    }

    /// Spawn: L from the placement strength, the init script, then the opening status.
    pub fn start(&mut self, w: &mut dyn World, r: &mut dyn RandomSource) {
        let mut io = Io { w, r };
        let l = level(self.strength, self.config.difficulty);
        self.vars.write(KIND_FLOAT, var::FLOAT_LEVEL, l);
        self.emit(&mut io, Obj::Boss, Event::Strength { strength: self.strength, level: l });
        let init = self.file.init;
        self.run_now(&mut io, Obj::Boss, init);
        let s = self.config.initial_status;
        self.change_status(&mut io, s, 0);
    }

    /// One frame.
    pub fn step(&mut self, w: &mut dyn World, r: &mut dyn RandomSource, input: &FrameInput) {
        let mut io = Io { w, r };
        self.vars.write(KIND_FLOAT, var::FLOAT_HP, input.hp_now);
        self.vars.write(KIND_FLOAT, var::FLOAT_HP_BEFORE, input.hp_before);
        self.vars.write(KIND_FLAG, var::FLAG_HIT_BY_PLAYER, Fx::from_int(input.hit_by_player as i64));
        self.evaluate_rules(&mut io, None);
        // status loop
        if let Some(mut t) = self.loop_thread.take() {
            let gen = self.status_gen;
            self.run(&mut io, &mut t);
            if self.status_gen == gen && self.loop_thread.is_none() && !t.done() {
                self.loop_thread = Some(t);
            }
            self.apply_pending(&mut io);
        }
        // timelines
        for i in 0..9 {
            if let Some(mut t) = self.timelines[i].take() {
                let gen = self.motion_gen;
                self.run(&mut io, &mut t);
                if self.motion_gen == gen && self.timelines[i].is_none() && !t.done() {
                    self.timelines[i] = Some(t);
                }
                self.apply_pending(&mut io);
            }
        }
        for c in 0..2 {
            if let Some(mut t) = self.cages[c].thread.take() {
                self.run(&mut io, &mut t);
                if self.cages[c].thread.is_none() && !t.done() {
                    self.cages[c].thread = Some(t);
                }
            }
        }
        self.tick = self.tick.wrapping_add(1);
    }

    /// A cage reports to the boss (format spec 6, 6.3).
    pub fn cage_event(&mut self, w: &mut dyn World, r: &mut dyn RandomSource, side: u8, ev: CageEvent) {
        let mut io = Io { w, r };
        let side = side.min(1);
        match ev {
            CageEvent::Damaged => {
                if self.status == 2 || self.status == 3 {
                    self.vars.write(KIND_INT, var::INT_SIDE, Fx::from_int(side as i64));
                    self.change_status(&mut io, 13 + side as i32, 0);
                }
                // the immediate priority-only check: -198 for the left cage (rule 6), -197 for the right (rule 7)
                self.evaluate_rules(&mut io, Some(-198 + side as i32));
            }
            CageEvent::PhaseOne => self.enable_rule(&mut io, 28),
            CageEvent::PhaseTwo => self.enable_rule(&mut io, 29),
            CageEvent::Broken => {
                if !self.broken_seen[0] && !self.broken_seen[1] {
                    self.emit(&mut io, Obj::Boss, Event::StageEvent(1000 + side as i32));
                    self.vars.write(KIND_FLAG, var::FLAG_CAGE_BROKEN, Fx::from_int(1));
                }
                self.broken_seen[side as usize] = true;
                self.evaluate_rules(&mut io, Some(-198 + side as i32));
            }
            CageEvent::Resync => {
                let f = self.cages[side as usize].motion_t0;
                self.motion_t0 = f;
                self.emit(&mut io, Obj::Boss, Event::Unknown(0x13F7));
            }
        }
    }

    // ------------------------------------------------------------------ emission and clocks

    fn obj_motion(&self, obj: Obj) -> (i32, u32) {
        match obj {
            Obj::Boss => (self.motion, self.motion_t0),
            Obj::Cage(c) => (self.cages[c as usize & 1].motion, self.cages[c as usize & 1].motion_t0),
        }
    }

    fn motion_frame(&self, io: &mut Io<'_>, obj: Obj) -> (i32, bool) {
        let (m, t0) = self.obj_motion(obj);
        let el = self.tick.wrapping_sub(t0) as i64;
        let len = io.w.motion_length(obj, m).max(1) as i64;
        let looping = self.file.motions.get(m as usize).is_some_and(|x| x.looping());
        if looping {
            let f = el % len;
            (f as i32, f == len - 1)
        } else {
            (el.min(len - 1) as i32, el >= len - 1)
        }
    }

    fn emit(&self, io: &mut Io<'_>, obj: Obj, ev: Event) {
        let (frame, _) = self.motion_frame(io, obj);
        let at = Stamp { tick: self.tick, motion: self.obj_motion(obj).0, frame };
        io.w.event(at, obj, ev);
    }

    // ------------------------------------------------------------------ variables and values

    fn vars_of(&mut self, obj: Obj) -> &mut Vars {
        match obj {
            Obj::Boss => &mut self.vars,
            Obj::Cage(c) => &mut self.cages[c as usize & 1].vars,
        }
    }

    fn param_value(&self, io: &mut Io<'_>, id: u32) -> Fx {
        match id {
            value::STATUS => Fx::from_int(self.status as i64),
            value::PREV_STATUS => Fx::from_int(self.prev_status as i64),
            value::STATUS_FRAME => Fx::from_int(self.status_frame() as i64),
            value::TARGET_HANDLE => Fx::from_int(self.target_handle() as i64),
            value::BOUNCE => self.file.bounce_factor(),
            value::DEATH_THRESHOLD => self.file.death_threshold(),
            _ => self.file.params.scalar(id).or_else(|| io.w.engine_value(id)).unwrap_or(Fx::ZERO),
        }
    }

    /// The target-slot handle (21004): the locked slot, else the slot being evaluated, else -1.
    fn target_handle(&self) -> i32 {
        self.lock.or(self.eval_slot).unwrap_or(-1)
    }

    fn read(&mut self, io: &mut Io<'_>, obj: Obj, v: Var) -> Fx {
        match v.mem {
            MEM_PARAM => self.param_value(io, v.id),
            MEM_WORK => self.vars_of(obj).read(v.kind, v.id),
            MEM_TEMP => self.temps.read(v.kind, v.id),
            _ => Fx::ZERO,
        }
    }

    fn write(&mut self, obj: Obj, v: Var, x: Fx) {
        match v.mem {
            MEM_WORK => self.vars_of(obj).write(v.kind, v.id, x),
            MEM_TEMP => self.temps.write(v.kind, v.id, x),
            _ => {}
        }
    }

    fn value(&mut self, io: &mut Io<'_>, obj: Obj, v: Value) -> Fx {
        match v {
            Value::Num(n) => n,
            Value::Var(x) => self.read(io, obj, x),
        }
    }

    // ------------------------------------------------------------------ requirements

    fn cond(&mut self, io: &mut Io<'_>, obj: Obj, c: &Cond) -> bool {
        let args: Vec<Fx> = c.args.iter().map(|&a| self.value(io, obj, a)).collect();
        let a = |i: usize| args.get(i).copied().unwrap_or(Fx::ZERO);
        let r = match c.req {
            req::MOTION_END => self.motion_frame(io, obj).1,
            req::COMPARE => {
                let (x, op, y) = (a(0), a(1).trunc(), a(2));
                match op {
                    0 => x < y,
                    1 => x <= y,
                    2 => x == y,
                    3 => x != y,
                    4 => x >= y,
                    5 => x > y,
                    _ => false,
                }
            }
            req::FLAG => a(0) != Fx::ZERO,
            req::ALWAYS => true,
            req::SLOT_PRESENT => io.w.slot_present(a(1).trunc() as i32),
            req::TARGET_SIDE => {
                let slot = a(0).trunc() as i32;
                match io.w.target_dx(slot) {
                    Some(dx) => (dx < Fx::ZERO) == (a(1).trunc() == 1),
                    None => false,
                }
            }
            req::TARGET_LOCKED => self.lock.is_some_and(|s| io.w.slot_present(s)),
            id => io.w.requirement(obj, id, &args),
        };
        r != c.not
    }

    // ------------------------------------------------------------------ rules

    fn enable_rule(&mut self, io: &mut Io<'_>, i: i32) {
        if let Some(r) = self.rules_on.get_mut(i as usize) {
            *r = true;
        }
        self.emit(io, Obj::Boss, Event::RuleEnabled(i));
    }

    fn check_rule(&mut self, io: &mut Io<'_>, i: usize, slot: Option<i32>) -> bool {
        let cond = self.file.rules[i].condition;
        if cond == Target::None {
            return true;
        }
        self.temps.write(KIND_FLAG, var::TEMP_FLAG_TRUE, Fx::ZERO);
        self.eval_slot = slot;
        if let Some(s) = slot {
            self.temps.write(KIND_INT, var::TEMP_INT_SLOT, Fx::from_int(s as i64));
        }
        self.run_now(io, Obj::Boss, cond);
        self.temps.flag(var::TEMP_FLAG_TRUE)
    }

    /// The decision procedure. `window = Some(p)` evaluates only rules of priority exactly `p` (the cage's immediate
    /// check).
    fn evaluate_rules(&mut self, io: &mut Io<'_>, window: Option<i32>) {
        let file = self.file;
        let rules = &file.rules;
        let mut hits: Vec<(usize, Option<i32>, i32)> = Vec::new();
        for i in 0..rules.len() {
            if !self.rules_on[i] {
                continue;
            }
            let r = &rules[i];
            if window.is_some_and(|p| p != r.priority) {
                continue;
            }
            if r.per_slot() {
                for s in 0..4 {
                    if r.target_mask & (1 << s) != 0 && io.w.slot_present(s) && self.check_rule(io, i, Some(s)) {
                        hits.push((i, Some(s), r.priority));
                    }
                }
            } else if self.check_rule(io, i, None) {
                hits.push((i, None, r.priority));
            }
        }
        self.eval_slot = None;
        let Some(best) = hits.iter().map(|h| h.2).min() else { return };
        hits.retain(|h| h.2 == best);
        let mut slots: Vec<i32> = hits.iter().filter_map(|h| h.1).collect();
        slots.sort_unstable();
        slots.dedup();
        let chosen = if slots.len() > 1 && self.config.tie_break == RuleTieBreak::SlotDrawThenTableOrder {
            Some(self.draw_slot(io, &slots))
        } else {
            None
        };
        let pick = hits.iter().find(|h| h.1.is_none() || chosen.is_none() || h.1 == chosen).copied();
        if let Some((i, slot, _)) = pick {
            self.fire_rule(io, i, slot);
        }
    }

    fn draw_slot(&mut self, io: &mut Io<'_>, slots: &[i32]) -> i32 {
        let base = self.file.slot_weights();
        let cut = self.file.slot_repeat_factor();
        let w: Vec<Fx> = slots
            .iter()
            .map(|&s| {
                let x = base[s as usize & 3];
                let x = if self.last_slot == Some(s) { x.mul(cut) } else { x };
                x.max(Fx::ZERO)
            })
            .collect();
        let total: i64 = w.iter().map(|x| x.raw()).sum();
        let r = below(io.r, total);
        let mut acc = 0i64;
        for (k, x) in w.iter().enumerate() {
            if x.raw() <= 0 {
                continue;
            }
            acc += x.raw();
            if acc > r {
                return slots[k];
            }
        }
        slots[0]
    }

    fn fire_rule(&mut self, io: &mut Io<'_>, i: usize, slot: Option<i32>) {
        let r = self.file.rules[i].clone();
        self.emit(io, Obj::Boss, Event::RuleFired { rule: i, slot });
        if let Some(s) = slot {
            self.temps.write(KIND_INT, var::TEMP_INT_SLOT, Fx::from_int(s as i64));
            self.last_slot = Some(s);
        }
        self.eval_slot = slot;
        self.pending_status = None;
        self.run_now(io, Obj::Boss, r.action);
        if r.lock {
            if let Some(s) = slot {
                if self.lock != Some(s) {
                    self.lock = Some(s);
                    self.emit(io, Obj::Boss, Event::TargetLocked(s));
                }
            }
        }
        self.eval_slot = None;
        if r.target_status != -1 {
            self.pending_status = Some(r.target_status);
        }
        self.apply_pending(io);
    }

    fn reset_rules(&mut self) {
        for (on, r) in self.rules_on.iter_mut().zip(&self.file.rules) {
            *on = r.default_enabled;
        }
    }

    // ------------------------------------------------------------------ status and motion

    fn status_slot(&self, table: &[Target; STATUS_SLOTS], s: i32) -> Target {
        let t = if (0..STATUS_SLOTS as i32).contains(&s) { table[s as usize] } else { Target::None };
        if t == Target::None && self.config.common_slot_fallback {
            table[COMMON_SLOT]
        } else {
            t
        }
    }

    fn apply_pending(&mut self, io: &mut Io<'_>) {
        if let Some(s) = self.pending_status.take() {
            self.change_status(io, s, 0);
        }
    }

    /// Change status: the old status's exit script, then the new one's enter script, then its loop thread.
    pub fn force_status(&mut self, w: &mut dyn World, r: &mut dyn RandomSource, s: i32) {
        let mut io = Io { w, r };
        self.change_status(&mut io, s, 0);
    }

    fn change_status(&mut self, io: &mut Io<'_>, s: i32, depth: u32) {
        if depth > MAX_STATUS_CHAIN {
            return;
        }
        self.emit(io, Obj::Boss, Event::Status { from: self.status, to: s });
        if self.status >= 0 {
            let exit = self.status_slot(&self.file.status_exit, self.status);
            self.run_now(io, Obj::Boss, exit);
        }
        self.prev_status = self.status;
        self.status = s;
        self.status_t0 = self.tick;
        self.status_gen = self.status_gen.wrapping_add(1);
        self.loop_thread = None;
        self.pending_status = None;
        let enter = self.status_slot(&self.file.status_enter, s);
        self.run_now(io, Obj::Boss, enter);
        let lp = self.status_slot(&self.file.status_loop, s);
        self.loop_thread = self.thread_for(io, Obj::Boss, lp);
        if let Some(p) = self.pending_status.take() {
            self.change_status(io, p, depth + 1);
        }
    }

    fn change_motion(&mut self, io: &mut Io<'_>, m: i32) {
        self.motion = m;
        self.motion_t0 = self.tick;
        self.motion_gen = self.motion_gen.wrapping_add(1);
        self.emit(io, Obj::Boss, Event::Motion(m));
        for c in 0..3 {
            for t in 0..3 {
                let tg = self.file.timelines[c][t].get(m as usize).copied().unwrap_or(Target::None);
                self.timelines[3 * c + t] = self.thread_for(io, Obj::Boss, tg);
            }
        }
    }

    // ------------------------------------------------------------------ threads

    /// A thread for a target, or None (null target; a built-in with no script body runs at once).
    fn thread_for(&mut self, io: &mut Io<'_>, obj: Obj, t: Target) -> Option<Thread> {
        match t {
            Target::Local(o) => Some(Thread::new(obj, Code::File(o))),
            Target::Import(i) => match self.builtins.get(i as usize).copied().flatten() {
                Some(Builtin::BossDown) => Some(Thread::new(obj, Code::BossDown)),
                Some(b) => {
                    self.builtin(io, obj, b);
                    None
                }
                None => {
                    self.emit(io, obj, Event::BadCall(t));
                    None
                }
            },
            Target::None => None,
            Target::Unresolved(_) => {
                self.emit(io, obj, Event::BadCall(t));
                None
            }
        }
    }

    /// Run a script immediately (conditions, actions, enter / exit, init). A wait ends it.
    fn run_now(&mut self, io: &mut Io<'_>, obj: Obj, t: Target) {
        if let Some(mut th) = self.thread_for(io, obj, t) {
            th.resume = i64::MIN;
            self.run_until_wait(io, &mut th, true);
        }
    }

    fn run(&mut self, io: &mut Io<'_>, t: &mut Thread) {
        self.run_until_wait(io, t, false);
    }

    fn code_cmd(&self, code: Code, pc: usize) -> Option<Command> {
        match code {
            Code::File(o) => self.file.script(o)?.cmds.get(pc).cloned(),
            Code::BossDown => boss_down(pc),
        }
    }
    fn code_op(&self, code: Code, pc: usize) -> Option<Op> {
        self.code_cmd(code, pc).map(|c| c.op)
    }

    fn now(&self) -> i64 {
        self.tick as i64 * ONE
    }

    fn run_until_wait(&mut self, io: &mut Io<'_>, t: &mut Thread, immediate: bool) {
        let obj = t.obj;
        let mut budget = BUDGET;
        loop {
            if t.frames.is_empty() || t.resume > self.now() {
                return;
            }
            budget -= 1;
            if budget == 0 {
                self.emit(io, obj, Event::Runaway);
                t.frames.clear();
                return;
            }
            let fi = t.frames.len() - 1;
            let (code, pc) = (t.frames[fi].code, t.frames[fi].pc);
            let Some(cmd) = self.code_cmd(code, pc) else {
                t.frames.pop();
                continue;
            };
            t.frames[fi].pc = pc + 1;
            match cmd.op {
                Op::Wait(v) => {
                    let n = self.value(io, obj, v);
                    if immediate && n.raw() > 0 {
                        t.frames.clear();
                        return;
                    }
                    t.resume = self.now() + n.raw();
                }
                Op::WaitUntil(v) => {
                    let f = self.value(io, obj, v);
                    let base = match (self.config.async_wait_in_loop, t.frames[fi].loops.last()) {
                        (AsyncWaitInLoop::IterationRelative, Some(l)) => l.iter_start,
                        _ => self.obj_motion(obj).1,
                    };
                    let at = base as i64 * ONE + f.raw();
                    if immediate && at > self.now() {
                        t.frames.clear();
                        return;
                    }
                    t.resume = at;
                }
                Op::LoopStart(n) => {
                    if n == 0 {
                        self.skip_loop(t, fi);
                    } else {
                        let tick = self.tick;
                        t.frames[fi].loops.push(LoopSt { body: pc + 1, left: n, iter_start: tick });
                    }
                }
                Op::LoopEnd => {
                    let tick = self.tick;
                    let f = &mut t.frames[fi];
                    if let Some(l) = f.loops.last_mut() {
                        if l.left < 0 || l.left > 1 {
                            if l.left > 0 {
                                l.left -= 1;
                            }
                            l.iter_start = tick;
                            f.pc = l.body;
                        } else {
                            f.loops.pop();
                        }
                    }
                }
                Op::Call(tg) => self.call(io, t, tg),
                Op::Return => {
                    t.frames.pop();
                }
                Op::If(c) => {
                    let v = self.cond_chain(io, t, fi, obj, &c);
                    if !v {
                        self.skip_branch(io, t, fi, obj);
                    }
                }
                Op::ElseIf(_) | Op::Else => self.skip_to_endif(t, fi),
                Op::And(_) | Op::Or(_) | Op::EndIf | Op::Case(_) | Op::Default | Op::EndSwitch => {}
                Op::Switch { list, count, .. } => {
                    let pick = self.switch_pick(io, obj, list, count);
                    self.jump_case(t, fi, pick);
                }
                Op::Break => self.do_break(t, fi),
                op => self.exec(io, obj, op, cmd.word),
            }
        }
    }

    fn call(&mut self, io: &mut Io<'_>, t: &mut Thread, tg: Target) {
        match tg {
            Target::Local(o) => t.frames.push(Frame { code: Code::File(o), pc: 0, loops: Vec::new() }),
            Target::Import(i) => match self.builtins.get(i as usize).copied().flatten() {
                Some(Builtin::BossDown) => t.frames.push(Frame { code: Code::BossDown, pc: 0, loops: Vec::new() }),
                Some(b) => self.builtin(io, t.obj, b),
                None => self.emit(io, t.obj, Event::BadCall(tg)),
            },
            _ => self.emit(io, t.obj, Event::BadCall(tg)),
        }
    }

    /// Evaluate a condition and fold the `and` / `or` commands that follow it (left to right).
    fn cond_chain(&mut self, io: &mut Io<'_>, t: &mut Thread, fi: usize, obj: Obj, c: &Cond) -> bool {
        let mut v = self.cond(io, obj, c);
        loop {
            let code = t.frames[fi].code;
            let pc = t.frames[fi].pc;
            match self.code_op(code, pc) {
                Some(Op::And(c2)) => {
                    t.frames[fi].pc += 1;
                    let x = self.cond(io, obj, &c2);
                    v = v && x;
                }
                Some(Op::Or(c2)) => {
                    t.frames[fi].pc += 1;
                    let x = self.cond(io, obj, &c2);
                    v = v || x;
                }
                _ => return v,
            }
        }
    }

    /// After a false `if` / `else if`: on to the next `else if` that holds, the `else`, or past the `end if`.
    fn skip_branch(&mut self, io: &mut Io<'_>, t: &mut Thread, fi: usize, obj: Obj) {
        let code = t.frames[fi].code;
        let mut depth = 0u32;
        loop {
            let pc = t.frames[fi].pc;
            let Some(op) = self.code_op(code, pc) else { return };
            t.frames[fi].pc = pc + 1;
            match op {
                Op::If(_) => depth += 1,
                Op::EndIf if depth == 0 => return,
                Op::EndIf => depth -= 1,
                Op::Else if depth == 0 => return,
                Op::ElseIf(c) if depth == 0 => {
                    if self.cond_chain(io, t, fi, obj, &c) {
                        return;
                    }
                }
                _ => {}
            }
        }
    }

    fn skip_to_endif(&self, t: &mut Thread, fi: usize) {
        let code = t.frames[fi].code;
        let mut depth = 0u32;
        loop {
            let pc = t.frames[fi].pc;
            let Some(op) = self.code_op(code, pc) else { return };
            t.frames[fi].pc = pc + 1;
            match op {
                Op::If(_) => depth += 1,
                Op::EndIf if depth == 0 => return,
                Op::EndIf => depth -= 1,
                _ => {}
            }
        }
    }

    fn skip_loop(&self, t: &mut Thread, fi: usize) {
        let code = t.frames[fi].code;
        let mut depth = 0u32;
        loop {
            let pc = t.frames[fi].pc;
            let Some(op) = self.code_op(code, pc) else { return };
            t.frames[fi].pc = pc + 1;
            match op {
                Op::LoopStart(_) => depth += 1,
                Op::LoopEnd if depth == 0 => return,
                Op::LoopEnd => depth -= 1,
                _ => {}
            }
        }
    }

    /// Jump to `case pick`, else `default`, else past `end switch` (nested switches skipped).
    fn jump_case(&self, t: &mut Thread, fi: usize, pick: Option<i32>) {
        let code = t.frames[fi].code;
        let start = t.frames[fi].pc;
        let (mut depth, mut pc) = (0u32, start);
        let (mut default, mut end) = (None, None);
        while let Some(op) = self.code_op(code, pc) {
            match op {
                Op::Switch { .. } => depth += 1,
                Op::EndSwitch if depth == 0 => {
                    end = Some(pc);
                    break;
                }
                Op::EndSwitch => depth -= 1,
                Op::Case(k) if depth == 0 && Some(k) == pick => {
                    t.frames[fi].pc = pc + 1;
                    return;
                }
                Op::Default if depth == 0 && default.is_none() => default = Some(pc),
                _ => {}
            }
            pc += 1;
        }
        t.frames[fi].pc = match (default, end) {
            (Some(d), _) => d + 1,
            (None, Some(e)) => e + 1,
            (None, None) => pc,
        };
    }

    /// `break`: past the enclosing `end switch`, or out of the enclosing loop, whichever closes first.
    fn do_break(&self, t: &mut Thread, fi: usize) {
        let code = t.frames[fi].code;
        let (mut sw, mut lp) = (0u32, 0u32);
        loop {
            let pc = t.frames[fi].pc;
            let Some(op) = self.code_op(code, pc) else { return };
            t.frames[fi].pc = pc + 1;
            match op {
                Op::Switch { .. } => sw += 1,
                Op::EndSwitch if sw == 0 => return,
                Op::EndSwitch => sw -= 1,
                Op::LoopStart(_) => lp += 1,
                Op::LoopEnd if lp == 0 => {
                    t.frames[fi].loops.pop();
                    return;
                }
                Op::LoopEnd => lp -= 1,
                _ => {}
            }
        }
    }

    /// The weighted switch (format spec 4.3). Returns the case index, or None when no entry has weight > 0 (no
    /// draw then).
    fn switch_pick(&mut self, io: &mut Io<'_>, obj: Obj, list: Var, count: i32) -> Option<i32> {
        let entries = self.file.params.weighted(list.id, count).ok()?;
        let l = self.level();
        let mut w: Vec<Fx> = Vec::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            let mut x = Fx::from_int(e.base as i64) + e.slope.mul(l);
            if self.config.weight_rounding == WeightRounding::TruncateBeforeCut {
                x = x.trunc_fx();
            }
            if e.condition != Target::None {
                self.temps.write(KIND_INT, var::TEMP_INT_ENTRY, Fx::from_int(i as i64));
                self.temps.write(KIND_FLOAT, var::TEMP_FLOAT_WEIGHT, x);
                self.temps.write(KIND_FLAG, var::TEMP_FLAG_TRUE, Fx::ZERO);
                self.run_now(io, obj, e.condition);
                if !self.temps.flag(var::TEMP_FLAG_TRUE) {
                    x = Fx::ZERO;
                } else {
                    x = self.temps.float(var::TEMP_FLOAT_WEIGHT);
                }
            }
            if self.config.weight_rounding == WeightRounding::TruncateAfterCut {
                x = x.trunc_fx();
            }
            w.push(x);
        }
        let total: i64 = w.iter().filter(|x| x.raw() > 0).map(|x| x.raw()).sum();
        if total <= 0 {
            return None;
        }
        let r = below(io.r, total);
        let mut acc = 0;
        for (i, x) in w.iter().enumerate() {
            if x.raw() <= 0 {
                continue;
            }
            acc += x.raw();
            if acc > r {
                return Some(i as i32);
            }
        }
        None
    }

    // ------------------------------------------------------------------ commands with effects

    fn exec(&mut self, io: &mut Io<'_>, obj: Obj, op: Op, word: u32) {
        match op {
            Op::ChangeStatus { status, cond, .. } => {
                if self.cond(io, obj, &cond) {
                    self.pending_status = Some(status);
                }
            }
            Op::ChangeMotion(m) => {
                if obj == Obj::Boss {
                    self.change_motion(io, m);
                } else {
                    self.cage_motion(io, obj, m);
                }
            }
            Op::Op0410(a, b) => self.emit(io, obj, Event::Op0410(a, b)),
            Op::ClearHitboxes => self.emit(io, obj, Event::ClearHitboxes),
            Op::HurtState(n) => self.emit(io, obj, Event::HurtState(n)),
            Op::Hitbox(h) => {
                let l = self.value(io, obj, h.level);
                let mut v = |x: Value| self.value(io, obj, x);
                let sc = |b: Fx, s: Fx| b + s.mul(l);
                let id = v(h.id).trunc() as i32;
                let part = v(h.part).trunc() as i32;
                let bone = v(h.bone).trunc() as i32;
                let damage = sc(v(h.damage.0), v(h.damage.1)).trunc() as i32;
                let angle = v(h.angle).trunc() as i32;
                let kbg = sc(v(h.kbg.0), v(h.kbg.1)).trunc() as i32;
                let wkb = sc(v(h.wkb.0), v(h.wkb.1)).trunc() as i32;
                let bkb = sc(v(h.bkb.0), v(h.bkb.1)).trunc() as i32;
                let size = sc(v(h.size.0), v(h.size.1));
                let offset = [v(h.offset[0]), v(h.offset[1]), v(h.offset[2])];
                let element = v(h.element).trunc() as i32;
                let trip = v(h.trip);
                let hitlag = v(h.hitlag);
                let sdi = v(h.sdi);
                let mut flags = [0i32; 25];
                for (k, a) in h.flags.iter().enumerate() {
                    flags[k] = match *a {
                        crate::script::Arg::Int(i) => i,
                        crate::script::Arg::Bool(b) => b as i32,
                        crate::script::Arg::Scalar(f) => f.trunc() as i32,
                        _ => a.to_pair().1 as i32,
                    };
                }
                let hb = ResolvedHitbox {
                    id,
                    part,
                    bone,
                    damage,
                    angle,
                    kbg,
                    wkb,
                    bkb,
                    size,
                    offset,
                    element,
                    trip,
                    hitlag,
                    sdi,
                    flags,
                    level: l,
                };
                self.emit(io, obj, Event::Hitbox(hb));
            }
            Op::GroundReset => self.emit(io, obj, Event::GroundReset),
            Op::Op0900(n) => self.emit(io, obj, Event::Op0900(n)),
            Op::Sound(k, id) => self.emit(io, obj, Event::Sound(k, id)),
            Op::Effect(k, a) => self.emit(io, obj, Event::Effect(k, *a)),
            Op::Shake(n) => self.emit(io, obj, Event::Shake(n)),
            Op::RuleClose(_) => {}
            Op::SetInt(v, x) | Op::SetFloat(v, x) => {
                let n = self.value(io, obj, v);
                self.write(obj, x, n);
            }
            Op::IncInt(x) => {
                let n = self.read(io, obj, x) + Fx::from_int(1);
                self.write(obj, x, n);
            }
            Op::SubFloat(a, b) => {
                let n = self.read(io, obj, b) - self.value(io, obj, a);
                self.write(obj, b, n);
            }
            Op::MulFloat(a, b) => {
                let n = self.read(io, obj, b).mul(self.value(io, obj, a));
                self.write(obj, b, n);
            }
            Op::SetFlag(x) => self.write(obj, x, Fx::from_int(1)),
            Op::ClearFlag(x) => self.write(obj, x, Fx::ZERO),
            Op::StageEvent(e) => self.emit(io, obj, Event::StageEvent(e)),
            Op::RandomRange { param, out } => {
                let l = self.level();
                let v = match self.file.params.range(param) {
                    Some([a, b, c, d]) => {
                        let lo = a + b.mul(l);
                        let hi = (c + d.mul(l)).max(lo);
                        lo + Fx(below(io.r, (hi - lo).raw()))
                    }
                    None => Fx::ZERO,
                };
                self.write(obj, out, v);
            }
            Op::Linear { param, out } => {
                let l = self.level();
                let f = self.file.params.floats(param).unwrap_or_default();
                let v = f.first().copied().unwrap_or(Fx::ZERO) + f.get(1).copied().unwrap_or(Fx::ZERO).mul(l);
                self.write(obj, out, v);
            }
            Op::FirstFloat { param, out } => {
                let f = self.file.params.floats(param).and_then(|f| f.first().copied()).unwrap_or(Fx::ZERO);
                self.write(obj, out, f);
            }
            Op::SpawnCage { param, arg } => {
                let side = self.file.params.raw(param).and_then(|r| r.get(1).copied()).unwrap_or(0) as i32;
                let c = &mut self.cages[side as usize & 1];
                c.spawned = true;
                // the cage starts with the boss's L truncated, as link 5112 would send it (assumption: the init
                // script never sends 5112, yet the spec's phase-0 basket damage uses the cage's truncated L)
                let l = Fx::from_int(self.vars.float(var::FLOAT_LEVEL).trunc());
                c.vars.write(KIND_FLOAT, var::CAGE_FLOAT_LEVEL, l);
                c.motion = self.motion;
                c.motion_t0 = self.motion_t0;
                self.emit(io, obj, Event::SpawnCage { param, side, arg });
            }
            Op::SetGoalX(v) => {
                let x = self.value(io, obj, v);
                self.emit(io, obj, Event::GoalX(x));
            }
            Op::CommonEffect(id) => self.emit(io, obj, Event::CommonEffect(id)),
            Op::SetStrength { param } => {
                let s = self.file.params.int(param).unwrap_or(1);
                self.strength = s;
                let l = level(s, self.config.difficulty);
                self.vars.write(KIND_FLOAT, var::FLOAT_LEVEL, l);
                self.emit(io, obj, Event::Strength { strength: s, level: l });
            }
            Op::StartMove { slot, kind, angle } => {
                let a = angle.map(|x| self.read(io, obj, x));
                self.emit(io, obj, Event::StartMove { slot, kind, angle: a });
            }
            Op::StopMove(s) => self.emit(io, obj, Event::StopMove(s)),
            Op::SetMoveKind { slot, kind } => self.emit(io, obj, Event::MoveKind { slot, kind }),
            Op::SetMoveDir { slot, angle } => {
                let a = self.read(io, obj, angle);
                self.emit(io, obj, Event::MoveDir { slot, angle: a });
            }
            Op::LockTarget(_) => {
                // locks the slot being evaluated (the handle the argument names)
                if let Some(s) = self.eval_slot.or(self.lock) {
                    self.lock = Some(s);
                    self.emit(io, obj, Event::TargetLocked(s));
                }
            }
            Op::TargetAngle { slot, out, .. } => {
                let s = self.read(io, obj, slot).trunc() as i32;
                if let Some(a) = io.w.target_angle(s) {
                    self.write(obj, out, a);
                }
            }
            Op::TargetRelX { slot, out, .. } => {
                let s = self.read(io, obj, slot).trunc() as i32;
                if let Some(dx) = io.w.target_dx(s) {
                    self.write(obj, out, dx);
                }
            }
            Op::EnableRule(i) => self.enable_rule(io, i),
            Op::ResetRules => {
                self.reset_rules();
                self.emit(io, obj, Event::RulesReset);
            }
            Op::ClearTarget => {
                self.lock = None;
                self.emit(io, obj, Event::TargetCleared);
            }
            Op::DeathCheck { threshold, value } => {
                let th = self.value(io, obj, threshold);
                let v = self.value(io, obj, value);
                if v <= th {
                    self.vars.write(KIND_FLOAT, var::FLOAT_HP, Fx::ZERO);
                    if !self.dead {
                        self.dead = true;
                        self.emit(io, obj, Event::Dead);
                        self.emit(io, obj, Event::StageEvent(1));
                    }
                    self.temps.write(KIND_FLAG, var::TEMP_FLAG_TRUE, Fx::from_int(1));
                }
            }
            Op::LinkInt { link: l, event, args } => {
                let ints: Vec<i32> = args.iter().map(|&a| self.value(io, obj, a).trunc() as i32).collect();
                self.link_int(io, l, event, &ints);
                self.emit(io, obj, Event::LinkInt { link: l, event, args: ints });
            }
            Op::LinkFloat { link: l, event, int, value } => {
                let v = self.value(io, obj, value);
                self.emit(io, obj, Event::LinkFloat { link: l, event, int, value: v });
            }
            Op::LinkRun { link: l, script } => {
                self.emit(io, obj, Event::LinkRun { link: l, script });
                let c = Obj::Cage((l & 1) as u8);
                if self.link_depth >= 4 {
                    return; // a linked script that links again: bounded
                }
                self.link_depth += 1;
                let th = self.thread_for(io, c, script);
                self.link_depth -= 1;
                if let Some(mut th) = th {
                    self.link_depth += 1;
                    th.resume = i64::MIN;
                    self.run(io, &mut th);
                    self.link_depth -= 1;
                    self.cages[(l & 1) as usize].thread = if th.done() { None } else { Some(th) };
                }
            }
            _ => self.emit(io, obj, Event::Unknown(word)),
        }
    }

    fn cage_motion(&mut self, io: &mut Io<'_>, obj: Obj, m: i32) {
        if let Obj::Cage(c) = obj {
            let cg = &mut self.cages[c as usize & 1];
            cg.motion = m;
            cg.motion_t0 = self.tick;
            self.emit(io, obj, Event::Motion(m));
        }
    }

    /// The link events the runner itself keeps track of (the cage's motion, L, proc and counter).
    fn link_int(&mut self, io: &mut Io<'_>, l: i32, event: i32, a: &[i32]) {
        let c = (l & 1) as usize;
        let first = a.first().copied().unwrap_or(0);
        match event {
            link::SET_PROC => self.cages[c].proc_ = first,
            link::MOTION => self.cage_motion(io, Obj::Cage(c as u8), first),
            // 5106 carries (0, n): the counter is the last argument (spec 3.8 / 9 item 9: second argument unverified)
            link::COUNTER => self.cages[c].counter = a.last().copied().unwrap_or(0),
            // the float L travels as an int: this is where the cage's truncated L comes from
            link::LEVEL => self.cages[c].vars.write(KIND_FLOAT, var::CAGE_FLOAT_LEVEL, Fx::from_int(first as i64)),
            _ => {}
        }
    }

    // ------------------------------------------------------------------ built-ins

    fn builtin(&mut self, io: &mut Io<'_>, obj: Obj, b: Builtin) {
        let set_true = |s: &mut Self| s.temps.write(KIND_FLAG, var::TEMP_FLAG_TRUE, Fx::from_int(1));
        match b {
            Builtin::StandBy | Builtin::StandByInit | Builtin::StandByExit => {}
            Builtin::CheckDamageAir
            | Builtin::CheckDamageGround
            | Builtin::CheckDamageTurn
            | Builtin::CheckDamageNoReaction
            | Builtin::CheckReactionEnd => {
                let k = match b {
                    Builtin::CheckDamageAir => DamageCheck::Air,
                    Builtin::CheckDamageGround => DamageCheck::Ground,
                    Builtin::CheckDamageTurn => DamageCheck::Turn,
                    Builtin::CheckDamageNoReaction => DamageCheck::NoReaction,
                    _ => DamageCheck::ReactionEnd,
                };
                if io.w.damage_check(k) {
                    set_true(self);
                }
            }
            Builtin::CheckMotionEnd => {
                if self.motion_frame(io, obj).1 {
                    set_true(self);
                }
            }
            Builtin::CheckDeathFigure => {
                if self.dead {
                    set_true(self);
                }
            }
            Builtin::NoneFlyDamageProc | Builtin::ExecDamageTurnNoRot | Builtin::DyingFigureProc => {
                self.emit(io, obj, Event::Builtin(b))
            }
            Builtin::DyingFigureProcInit => {
                self.rules_on.iter_mut().for_each(|r| *r = false);
                self.lock = None;
                self.emit(io, obj, Event::HurtState(2));
                self.emit(io, obj, Event::Builtin(b));
            }
            Builtin::DyingNoneEffProc => {
                self.rules_on.iter_mut().for_each(|r| *r = false);
                self.lock = None;
                self.emit(io, obj, Event::TargetCleared);
                self.emit(io, obj, Event::StopMove(-1));
                self.emit(io, obj, Event::StageEvent(0));
                self.emit(io, obj, Event::HurtState(2));
                self.emit(io, obj, Event::Builtin(b));
            }
            Builtin::BossDown => {}
        }
    }
}

/// `gameAnimCmd_CommonBossDown` as a native script: stage notifications 5, 6, 7, 8 at frames 0, 1, 180, 420.
fn boss_down(pc: usize) -> Option<Command> {
    let wait = |n: i64| Command { at: 0, word: 0x0001_0100, args: Vec::new(), op: Op::Wait(Value::Num(Fx::from_int(n))) };
    let ev = |e: i32| Command { at: 0, word: 0x6505_0100, args: Vec::new(), op: Op::StageEvent(e) };
    Some(match pc {
        0 => ev(5),
        1 => wait(1),
        2 => ev(6),
        3 => wait(179),
        4 => ev(7),
        5 => wait(240),
        6 => ev(8),
        _ => return None,
    })
}
