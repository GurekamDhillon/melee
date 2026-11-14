//! Geno boss actors (Geno v5.2 boss block, docs/specs/geno-v5.2-boss.md).
//!
//! A boss is a *define* (data: parts, HP pools, statuses, prioritised rules, weighted tables, kinetics, timelines,
//! messages, phases, a level scalar L) run by this generic interpreter. Nothing here knows any particular boss: a
//! define comes from a mod or, for bosses built from a local boss IR, from `geno_boss_ir` at load time.
//!
//! Determinism / rollback: the define is immutable after load (a `OnceLock`, never written again). All mutable state
//! is the fixed-size [`State`] in one static (`S`), no heap, no wall clock, its own LCG, so the engine's memory
//! snapshots cover it and [`state_hash`] folds it into NHV1 while a boss is live. Cost: a handful of rules and ops per
//! frame, no allocation in [`tick`].
#![allow(static_mut_refs, clippy::missing_safety_doc)]

use melee_core::ppc::fctiwz_f32;

/// Engine limits (the state block is fixed size).
pub const MAX_PARTS: usize = 4;
pub const MAX_VARS: usize = 24;
pub const MAX_RULES: usize = 64;
pub const MAX_EVENTS: usize = 16;

// ------------------------------------------------------------------------------------------------- define

/// A number that may scale with the level: `base + per_level * L`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    K(f32),
    Lin(f32, f32),
    /// the value of a variable
    Var(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    fn test(self, a: f32, b: f32) -> bool {
        match self {
            Cmp::Eq => a == b,
            Cmp::Ne => a != b,
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
        }
    }
}

/// Which hits a damage condition matches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitKind {
    Any,
    Air,
    Ground,
    /// from behind (the actor's facing); an actor that never turns and has no facing never matches
    Behind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cond {
    Always,
    Never,
    /// the motion is on its last frame
    MotionEnd,
    /// status frame >= variable (a status timeout)
    TimerDone(u8),
    Var(u8, Cmp, f32),
    VarVar(u8, Cmp, u8),
    /// the current status is in the mask
    StatusIn(u64),
    /// the previous status is in the mask
    PrevStatusIn(u64),
    /// the core part took a player hit this frame
    Hit(HitKind),
    /// lowest weak-point HP percent <= x
    MinWeakPctLe(f32),
    /// one part's HP percent <= x
    PartPctLe(u8, f32),
    /// a selectable target exists (and, with `true`, none is locked)
    Target(bool),
    /// the goal x is set and reached
    AtGoal,
    /// the host reports the actor outside the area (blast zones)
    OutOfArea,
    And(Box<Cond>, Box<Cond>),
    Not(Box<Cond>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Nop,
    /// all rules back to their table defaults
    ResetRules,
    /// every rule off
    DisableRules,
    EnableRules(u64),
    ResetTarget,
    LockTarget,
    StopMove,
    /// kinetic slot: kinetic index, along the aim direction
    Kinetic(u8, bool),
    /// var = uniform in [lo, hi]
    SetTimer(u8, Num, Num),
    SetVar(u8, Num),
    AddVar(u8, f32),
    /// var = 1 when part a's percent < part b's, else 0
    SetLess(u8, u8, u8),
    Motion(u8),
    /// pick a motion from a table (entries' `motion`)
    MotionPick(u8),
    /// the same, only when the motion has ended (per-frame)
    MotionPickOnEnd(u8),
    /// weighted pick from a table, running the entry's ops; the store var gets the index (and is the repeat key)
    Pick(u8, Option<u8>),
    If(Cond, Vec<Op>, Vec<Op>),
    ChangeStatus(u8),
    /// change status by the locked (or a drawn) target's side: target x < actor x -> .0 else .1; none -> .1
    StatusBySide(u8, u8),
    /// absolute goal x by the target's side (x < actor x -> .0 else .1)
    GoalBySide(f32, f32),
    /// aim = sign(target x - x), snapped to the horizontal; kept when no target
    Aim,
    StageEvent(i32),
    /// stage event by a flag var: var != 0 -> .1 else .2
    StageEventIf(u8, i32, i32),
    /// stage event of the message's sending part (a table per part)
    StageEventOfSender([i32; MAX_PARTS]),
    /// change status per sending part
    StatusOfSender([u8; MAX_PARTS]),
    /// presentation sound (Brawl / game id; -1 = the host's own for this motion frame)
    Sound(i32),
    /// screen shake 0 small, 1 medium, 2 large
    Shake(u8),
    /// part intangibility (mask of parts)
    Intangible(u8, bool),
    /// forward this frame's core-part damage to the routing targets, unless the var is set
    Forward(Option<u8>),
    /// the level's strength input
    SetStrength(u8),
    /// set every weak point's phase counter
    SetCounter(i32),
    /// run one rule evaluation restricted to priority window (per sending part); stops the handler if one fired
    EvalWindowOfSender([i32; MAX_PARTS]),
    /// clear the part's (mask) hitboxes; mask 0 = all
    ClearHitboxes(u8),
    /// attack on: index into `attacks`, owning part
    Hitbox(u8, u8),
    /// the actor is gone after this status (host handles removal)
    Despawn,
}

#[derive(Clone, Debug)]
pub struct TableEntry {
    pub weight: Num,
    pub ops: Vec<Op>,
    pub motion: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub name: String,
    pub repeat: f32,
    pub entries: Vec<TableEntry>,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub index: u16,
    pub priority: i32,
    pub default_on: bool,
    pub cond: Cond,
    pub ops: Vec<Op>,
    pub to: Option<u8>,
    pub per_target: bool,
}

#[derive(Clone, Debug)]
pub struct Status {
    pub index: u8,
    pub name: String,
    pub enter: Vec<Op>,
    pub frame: Vec<Op>,
    /// frame ops run on the first frame only (a one-shot sequence such as a defeat)
    pub frame_once: bool,
    pub intangible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kinetic {
    /// speed decays toward 0 by `decay` per frame
    Decay(Num),
    /// starts at `start` along the aim, accelerates by `acc` up to `max`
    Accel { start: f32, acc: Num, max: Num },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Role {
    Core,
    WeakPoint,
    Limb,
}

/// Per-part procs (a small native state machine): on a damaged frame send `on_damaged`; otherwise, while the part's
/// counter equals `thresholds[i].0` and its percent <= `.1`, send `.2`; on reaching 0 HP send `on_zero` once.
#[derive(Clone, Debug)]
pub struct Part {
    pub name: String,
    pub role: Role,
    /// max HP at spawn: trunc(Num) at the spawn L
    pub hp: Num,
    pub on_damaged: Option<u32>,
    pub thresholds: Vec<(i32, f32, u32)>,
    pub on_zero: Option<u32>,
    pub intangible_in: u64,
    /// trunc(L) instead of L for this part's attacks
    pub int_level: bool,
}

#[derive(Clone, Debug)]
pub struct TimelineEvent {
    pub frame: i32,
    pub count: i32,
    pub every: i32,
    pub op: Op,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitboxDef {
    pub part: u8,
    pub joint: i32,
    pub offset: [f32; 3],
    pub frames: (i32, i32),
    pub damage: Num,
    pub angle: i32,
    pub kbg: Num,
    pub wkb: f32,
    pub bkb: Num,
    pub size: Num,
    pub group: u8,
    pub rehit: i32,
}

#[derive(Clone, Debug)]
pub struct Attack {
    pub name: String,
    pub boxes: Vec<HitboxDef>,
}

#[derive(Clone, Debug)]
pub struct Motion {
    pub name: String,
    pub len: i32,
    pub looping: bool,
    pub timeline: Vec<TimelineEvent>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Camera {
    pub distance: f32,
    pub ease: f32,
    pub tilt: f32,
    pub fov: f32,
    /// left, right, bottom, top
    pub limits: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Define {
    pub name: String,
    pub parts: Vec<Part>,
    pub vars: Vec<(String, f32)>,
    pub statuses: Vec<Status>,
    pub rules: Vec<Rule>,
    pub tables: Vec<Table>,
    pub kinetics: Vec<Kinetic>,
    pub motions: Vec<Motion>,
    pub attacks: Vec<Attack>,
    /// message id -> handler ops
    pub messages: Vec<(u32, Vec<Op>)>,
    /// level: rows by strength (1-based), columns by difficulty
    pub level_rows: Vec<Vec<f32>>,
    pub strength0: u8,
    pub start_status: u8,
    /// target slot weights and the repeat factor
    pub slot_w: [f32; 4],
    pub slot_repeat: f32,
    /// core-part damage routing targets (part mask) and factor
    pub route_mask: u8,
    pub route_factor: f32,
    /// statuses that count as defeated (cause 1 threshold, cause 2 part break)
    pub defeat_status: [Option<u8>; 2],
    pub goal_snap: bool,
    pub spawn_x: f32,
    pub camera: Option<Camera>,
    /// the default rule window
    pub window: (i32, i32),
    /// the evaluation passes per frame, in order: (lo, hi, only on a frame the core was hit). The enemy engine's are
    /// -100 first, the damage window -250..-150 during a hit reaction, then the normal window -50..250.
    pub passes: Vec<(i32, i32, bool)>,
}

impl Define {
    pub fn status(&self, ix: u8) -> Option<&Status> {
        self.statuses.iter().find(|s| s.index == ix)
    }
    pub fn motion_index(&self, name: &str) -> Option<u8> {
        self.motions.iter().position(|m| m.name == name).map(|i| i as u8)
    }
}

// ------------------------------------------------------------------------------------------------- state

/// Everything that changes during a fight. Plain data, fixed size: memory snapshots cover it.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct State {
    pub live: bool,
    pub status: i32,
    pub prev_status: i32,
    pub sf: i32,
    pub motion: i32,
    pub mf: i32,
    /// the last motion frame whose timeline events ran
    pub tl_mf: i32,
    pub x: f32,
    pub vx: f32,
    pub kinetic: i32,
    pub aim: f32,
    pub goal: f32,
    pub has_goal: bool,
    pub strength: i32,
    pub difficulty: i32,
    pub rules_on: u64,
    pub vars: [f32; MAX_VARS],
    pub hp: [i32; MAX_PARTS],
    pub hp_max: [i32; MAX_PARTS],
    pub counter: [i32; MAX_PARTS],
    pub zero_sent: u8,
    pub intangible: u8,
    /// this frame's input: core damage, per-part direct hits / damage, airborne
    pub hit: bool,
    pub hit_part: u8,
    pub dmg_core: i32,
    pub damaged: u8,
    /// target lock: slot (-1 none) and the last drawn slot (repeat cut)
    pub locked: i32,
    pub last_slot: i32,
    pub rng: u32,
    pub frame: u32,
    pub cause: i32,
    pub despawned: bool,
    /// per-frame presentation events (sound / shake / stage events), read by the host after `tick`
    pub ev_n: u32,
    pub ev: [(u8, i32); MAX_EVENTS],
    /// bumped on each motion start (hit groups key on it)
    pub inst: u32,
    /// live attack boxes: (attack index, part) per slot, 0xFF = none
    pub atk: [(u8, u8); 4],
}

pub const OFF: State = State {
    live: false, status: -1, prev_status: -1, sf: 0, motion: 0, mf: 0, tl_mf: -1, x: 0.0, vx: 0.0, kinetic: -1, aim: 1.0,
    goal: 0.0, has_goal: false, strength: 1, difficulty: 5, rules_on: 0, vars: [0.0; MAX_VARS], hp: [0; MAX_PARTS],
    hp_max: [1; MAX_PARTS], counter: [0; MAX_PARTS], zero_sent: 0, intangible: 0, hit: false, hit_part: 0, dmg_core: 0,
    damaged: 0, locked: -1, last_slot: -1, rng: 1, frame: 0, cause: 0, despawned: false, ev_n: 0,
    ev: [(0, 0); MAX_EVENTS], inst: 1, atk: [(0xFF, 0xFF); 4],
};

pub static mut S: State = OFF;

/// Event kinds in `State::ev`.
pub const EV_SOUND: u8 = 1;
pub const EV_SHAKE: u8 = 2;
pub const EV_STAGE: u8 = 3;
pub const EV_STATUS: u8 = 4;

/// The host's view of the world each frame: the target candidates' x (None = absent / not selectable) and whether
/// the actor is airborne.
#[derive(Clone, Copy, Default)]
pub struct World {
    pub slots: [Option<f32>; 4],
    pub airborne: bool,
    pub out_of_area: bool,
}

// ------------------------------------------------------------------------------------------------- runtime

pub struct Rt<'a> {
    pub d: &'a Define,
    pub w: World,
}

/// FNV-1a over the state words (folded into NHV1 while a boss is live).
pub fn state_hash() -> Option<u32> {
    let s = unsafe { &S };
    if !s.live {
        return None;
    }
    let mut h: u32 = 0x811C_9DC5;
    let mut mix = |v: u32| {
        for b in v.to_le_bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    };
    mix(s.status as u32);
    mix(s.sf as u32);
    mix(s.motion as u32);
    mix(s.mf as u32);
    mix(s.x.to_bits());
    mix(s.vx.to_bits());
    mix(s.goal.to_bits());
    mix(s.has_goal as u32);
    mix(s.strength as u32);
    mix(s.rules_on as u32);
    mix((s.rules_on >> 32) as u32);
    for v in s.vars {
        mix(v.to_bits());
    }
    for i in 0..MAX_PARTS {
        mix(s.hp[i] as u32);
        mix(s.counter[i] as u32);
    }
    mix(s.zero_sent as u32 | (s.intangible as u32) << 8);
    mix(s.locked as u32);
    mix(s.last_slot as u32);
    mix(s.rng);
    mix(s.cause as u32);
    Some(h)
}

fn log_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_GENO_BOSS_LOG").is_some() || std::env::var_os("MELEE_PETEY_LOG").is_some())
}

impl<'a> Rt<'a> {
    fn rnd(&self, n: u32) -> u32 {
        unsafe {
            S.rng = S.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
            (S.rng >> 8) % n.max(1)
        }
    }
    fn rnd01(&self) -> f32 {
        self.rnd(1 << 16) as f32 / 65536.0
    }

    /// L = row[strength - 1][difficulty]; 0 out of range.
    pub fn level(&self) -> f32 {
        let s = unsafe { &S };
        let r = (s.strength - 1) as usize;
        self.d.level_rows.get(r).and_then(|row| row.get(s.difficulty as usize)).copied().unwrap_or(0.0)
    }

    pub fn num(&self, n: Num) -> f32 {
        match n {
            Num::K(k) => k,
            Num::Lin(b, l) => b + l * self.level(),
            Num::Var(v) => unsafe { S.vars[v as usize] },
        }
    }

    /// A part's level (trunc L for parts that hold an integer level).
    pub fn part_level(&self, part: u8) -> f32 {
        let l = self.level();
        match self.d.parts.get(part as usize) {
            Some(p) if p.int_level => fctiwz_f32(l) as f32,
            _ => l,
        }
    }

    pub fn pct(&self, part: usize) -> f32 {
        let s = unsafe { &S };
        100.0 * s.hp[part].max(0) as f32 / s.hp_max[part].max(1) as f32
    }

    fn min_weak_pct(&self) -> f32 {
        let mut m = 100.0f32;
        for (i, p) in self.d.parts.iter().enumerate() {
            if p.role == Role::WeakPoint {
                let v = self.pct(i);
                if v < m {
                    m = v;
                }
            }
        }
        m
    }

    fn motion_len(&self) -> i32 {
        let s = unsafe { &S };
        self.d.motions.get(s.motion as usize).map(|m| m.len).unwrap_or(1)
    }

    fn cond(&self, c: &Cond) -> bool {
        let s = unsafe { &S };
        match c {
            Cond::Always => true,
            Cond::Never => false,
            Cond::MotionEnd => s.mf + 1 >= self.motion_len(),
            Cond::TimerDone(v) => s.sf as f32 >= s.vars[*v as usize],
            Cond::Var(v, op, k) => op.test(s.vars[*v as usize], *k),
            Cond::VarVar(a, op, b) => op.test(s.vars[*a as usize], s.vars[*b as usize]),
            Cond::StatusIn(m) => s.status >= 0 && s.status < 64 && m >> s.status & 1 != 0,
            Cond::PrevStatusIn(m) => s.prev_status >= 0 && s.prev_status < 64 && m >> s.prev_status & 1 != 0,
            Cond::Hit(k) => {
                s.hit
                    && match k {
                        HitKind::Any => true,
                        HitKind::Air => self.w.airborne,
                        HitKind::Ground => !self.w.airborne,
                        HitKind::Behind => false,
                    }
            }
            Cond::MinWeakPctLe(x) => self.min_weak_pct() <= *x,
            Cond::PartPctLe(p, x) => self.pct(*p as usize) <= *x,
            Cond::AtGoal => s.has_goal && s.x == s.goal,
            Cond::OutOfArea => self.w.out_of_area,
            Cond::Target(unlocked) => self.any_target() && (!*unlocked || s.locked < 0),
            Cond::And(a, b) => self.cond(a) && self.cond(b),
            Cond::Not(a) => !self.cond(a),
        }
    }

    fn any_target(&self) -> bool {
        self.w.slots.iter().zip(self.d.slot_w.iter()).any(|(s, w)| s.is_some() && *w > 0.0)
    }

    /// Weighted slot draw (repeat cut on the last drawn slot).
    fn draw_slot(&self) -> Option<i32> {
        let mut w = [0f32; 4];
        for i in 0..4 {
            if self.w.slots[i].is_some() {
                w[i] = self.d.slot_w[i];
                if unsafe { S.last_slot } == i as i32 {
                    w[i] *= self.d.slot_repeat;
                }
            }
        }
        let i = self.weighted(&w)?;
        unsafe { S.last_slot = i as i32 };
        Some(i as i32)
    }

    /// r = rand01 * sum; first running sum >= r (weights <= 0 skipped).
    fn weighted(&self, w: &[f32]) -> Option<usize> {
        let sum: f32 = w.iter().filter(|&&x| x > 0.0).sum();
        if sum <= 0.0 {
            return None;
        }
        let r = self.rnd01() * sum;
        let mut acc = 0.0;
        let mut last = None;
        for (i, &x) in w.iter().enumerate() {
            if x <= 0.0 {
                continue;
            }
            acc += x;
            last = Some(i);
            if acc >= r {
                return Some(i);
            }
        }
        last
    }

    fn table_pick(&self, t: u8, store: Option<u8>) -> Option<usize> {
        let t = &self.d.tables[t as usize];
        let prev = store.map(|v| unsafe { S.vars[v as usize] } as i32);
        let mut w = [0f32; 8];
        for (i, e) in t.entries.iter().enumerate().take(8) {
            let x = fctiwz_f32(self.num(e.weight)) as f32;
            w[i] = if prev == Some(i as i32) { x * t.repeat } else { x };
        }
        self.weighted(&w[..t.entries.len().min(8)])
    }

    fn target_x(&self) -> Option<f32> {
        let s = unsafe { &S };
        if s.locked >= 0 {
            return self.w.slots[s.locked as usize];
        }
        None
    }

    fn push_ev(&self, kind: u8, v: i32) {
        unsafe {
            if (S.ev_n as usize) < MAX_EVENTS {
                S.ev[S.ev_n as usize] = (kind, v);
                S.ev_n += 1;
            }
        }
    }

    fn set_motion(&self, m: u8) {
        unsafe {
            S.motion = m as i32;
            S.mf = 0;
            S.tl_mf = -1;
            S.inst = S.inst.wrapping_add(1);
            S.atk = [(0xFF, 0xFF); 4];
        }
    }

    pub fn enter(&self, st: u8) {
        let Some(sd) = self.d.status(st) else { return };
        unsafe {
            if log_on() {
                eprintln!("geno_boss: f{} status {} -> {} ({}) x {:.1} hp {:?}", S.frame, S.status, st, sd.name, S.x, &S.hp[..self.d.parts.len().min(MAX_PARTS)]);
            }
            S.prev_status = S.status;
            S.status = st as i32;
            S.sf = 0;
            S.inst = S.inst.wrapping_add(1);
            self.push_ev(EV_STATUS, st as i32);
            for (i, p) in self.d.parts.iter().enumerate() {
                if p.intangible_in >> st & 1 != 0 {
                    S.intangible |= 1 << i;
                } else if p.role == Role::Core {
                    S.intangible &= !(1 << i);
                }
            }
            if self.d.defeat_status[0] == Some(st) && S.cause == 0 {
                S.cause = 1;
            } else if self.d.defeat_status[1] == Some(st) && S.cause == 0 {
                S.cause = 2;
            }
        }
        let ops = &sd.enter;
        self.run(ops, 0);
    }

    /// Run ops; `sender` is the message's sending part. Returns true when a status change or a fired window ended it.
    fn run(&self, ops: &[Op], sender: usize) -> bool {
        for op in ops {
            if self.op(op, sender) {
                return true;
            }
        }
        false
    }

    fn op(&self, op: &Op, sender: usize) -> bool {
        let d = self.d;
        let s = unsafe { &mut S };
        match op {
            Op::Nop => {}
            Op::ResetRules => {
                s.rules_on = d.rules.iter().enumerate().filter(|(_, r)| r.default_on).fold(0, |m, (i, _)| m | 1u64 << i);
            }
            Op::DisableRules => s.rules_on = 0,
            Op::EnableRules(m) => s.rules_on |= *m,
            Op::ResetTarget => s.locked = -1,
            Op::LockTarget => {
                if s.locked < 0 {
                    s.locked = self.draw_slot().unwrap_or(-1);
                }
            }
            Op::StopMove => {
                s.vx = 0.0;
                s.kinetic = -1;
            }
            Op::Kinetic(k, along) => {
                s.kinetic = *k as i32;
                if let Some(Kinetic::Accel { start, .. }) = d.kinetics.get(*k as usize) {
                    s.vx = if *along { *start * s.aim } else { *start };
                }
            }
            Op::SetTimer(v, lo, hi) => {
                let a = self.num(*lo);
                let b = self.num(*hi);
                let b = if b < a { a } else { b };
                let r = self.rnd01();
                unsafe { S.vars[*v as usize] = a + (b - a) * r };
            }
            Op::SetVar(v, n) => {
                let x = self.num(*n);
                unsafe { S.vars[*v as usize] = x };
            }
            Op::AddVar(v, k) => s.vars[*v as usize] += *k,
            Op::SetLess(v, a, b) => s.vars[*v as usize] = (self.pct(*a as usize) < self.pct(*b as usize)) as i32 as f32,
            Op::Motion(m) => self.set_motion(*m),
            Op::MotionPick(t) | Op::MotionPickOnEnd(t) => {
                if matches!(op, Op::MotionPickOnEnd(_)) && unsafe { S.mf } + 1 < self.motion_len() {
                    return false;
                }
                if let Some(i) = self.table_pick(*t, None) {
                    if let Some(m) = d.tables[*t as usize].entries[i].motion {
                        self.set_motion(m);
                    }
                }
            }
            Op::Pick(t, store) => {
                if let Some(i) = self.table_pick(*t, *store) {
                    if let Some(v) = store {
                        unsafe { S.vars[*v as usize] = i as f32 };
                    }
                    return self.run(&d.tables[*t as usize].entries[i].ops, sender);
                }
            }
            Op::If(c, a, b) => return self.run(if self.cond(c) { a } else { b }, sender),
            Op::ChangeStatus(st) => {
                self.enter(*st);
                return true;
            }
            Op::StatusBySide(lt, ge) => {
                if s.locked < 0 {
                    s.locked = self.draw_slot().unwrap_or(-1);
                }
                let st = match self.target_x() {
                    Some(tx) if tx < unsafe { S.x } => *lt,
                    _ => *ge,
                };
                self.enter(st);
                return true;
            }
            Op::GoalBySide(lt, ge) => {
                if let Some(tx) = self.target_x() {
                    s.goal = if tx < s.x { *lt } else { *ge };
                    s.has_goal = d.goal_snap;
                }
            }
            Op::Aim => {
                if let Some(tx) = self.target_x() {
                    s.aim = if tx < s.x { -1.0 } else { 1.0 };
                }
            }
            Op::StageEvent(e) => {
                if log_on() {
                    eprintln!("geno_boss: f{} stage event {}", s.frame, e);
                }
                self.push_ev(EV_STAGE, *e)
            }
            Op::StageEventIf(v, a, b) => {
                let e = if s.vars[*v as usize] != 0.0 { *a } else { *b };
                return self.op(&Op::StageEvent(e), sender);
            }
            Op::StageEventOfSender(t) => return self.op(&Op::StageEvent(t[sender.min(MAX_PARTS - 1)]), sender),
            Op::StatusOfSender(t) => {
                self.enter(t[sender.min(MAX_PARTS - 1)]);
                return true;
            }
            Op::Sound(id) => self.push_ev(EV_SOUND, *id),
            Op::Shake(k) => self.push_ev(EV_SHAKE, *k as i32),
            Op::Intangible(mask, on) => {
                if *on {
                    s.intangible |= *mask
                } else {
                    s.intangible &= !*mask
                }
            }
            Op::Forward(unless) => {
                if unless.map_or(true, |v| s.vars[v as usize] == 0.0) && s.dmg_core > 0 {
                    let d_out = fctiwz_f32(s.dmg_core as f32 * d.route_factor);
                    for i in 0..d.parts.len().min(MAX_PARTS) {
                        if d.route_mask >> i & 1 != 0 && s.hp[i] > 0 {
                            s.hp[i] -= d_out;
                            s.damaged |= 1 << i;
                        }
                    }
                    s.dmg_core = 0;
                }
            }
            Op::SetStrength(n) => {
                s.strength = *n as i32;
                if log_on() {
                    eprintln!("geno_boss: f{} strength {} (L {:.1})", s.frame, n, self.level());
                }
            }
            Op::SetCounter(c) => s.counter = [*c; MAX_PARTS],
            Op::EvalWindowOfSender(t) => {
                let p = t[sender.min(MAX_PARTS - 1)];
                return self.evaluate((p, p));
            }
            Op::ClearHitboxes(mask) => {
                for a in s.atk.iter_mut() {
                    if *mask == 0 || (a.1 != 0xFF && *mask >> a.1 & 1 != 0) {
                        *a = (0xFF, 0xFF);
                    }
                }
            }
            Op::Hitbox(atk, part) => {
                if let Some(e) = s.atk.iter_mut().find(|a| a.0 == 0xFF || (a.0 == *atk && a.1 == *part)) {
                    *e = (*atk, *part);
                }
            }
            Op::Despawn => s.despawned = true,
        }
        false
    }

    /// One rule evaluation (rule_engine): enabled rules in the window whose condition holds, lowest priority first,
    /// table order on ties. Action-only rules (no target status) fire and the evaluation goes on (one per priority
    /// level); the first rule with a target status fires and ends it. Returns true if any rule fired.
    pub fn evaluate(&self, window: (i32, i32)) -> bool {
        let d = self.d;
        let mut order = [0u8; MAX_RULES];
        let mut n = 0;
        for (i, r) in d.rules.iter().enumerate().take(MAX_RULES) {
            if unsafe { S.rules_on } >> i & 1 != 0 && r.priority >= window.0 && r.priority <= window.1 {
                order[n] = i as u8;
                n += 1;
            }
        }
        // stable insertion sort by priority
        for i in 1..n {
            let mut j = i;
            while j > 0 && d.rules[order[j - 1] as usize].priority > d.rules[order[j] as usize].priority {
                order.swap(j - 1, j);
                j -= 1;
            }
        }
        let mut fired = false;
        let mut done_prio: Option<i32> = None;
        let st0 = unsafe { S.status };
        for &ri in &order[..n] {
            let r = &d.rules[ri as usize];
            if unsafe { S.rules_on } >> ri & 1 == 0 || unsafe { S.status } != st0 {
                continue;
            }
            if done_prio == Some(r.priority) {
                continue;
            }
            if !self.cond(&r.cond) {
                continue;
            }
            if r.per_target {
                unsafe {
                    if S.locked < 0 {
                        S.locked = self.draw_slot().unwrap_or(-1);
                    }
                }
            }
            if log_on() && r.to.is_some() {
                eprintln!("geno_boss: f{} rule {} fires", unsafe { S.frame }, r.index);
            }
            fired = true;
            done_prio = Some(r.priority);
            if self.run(&r.ops, 0) {
                return true;
            }
            if let Some(t) = r.to {
                self.enter(t);
                return true;
            }
        }
        fired
    }

    fn timeline(&self) {
        let s = unsafe { &mut S };
        let Some(m) = self.d.motions.get(s.motion as usize) else { return };
        let mut f = s.tl_mf + 1;
        while f <= s.mf {
            for e in &m.timeline {
                let mut hit = e.frame == f;
                if !hit && e.count > 1 && e.every > 0 && f > e.frame {
                    let k = f - e.frame;
                    hit = k % e.every == 0 && k / e.every < e.count;
                }
                if hit {
                    if log_on() {
                        match e.op {
                            Op::Shake(k) => eprintln!("geno_boss: f{} shake {} (motion {} frame {})", s.frame, k, s.motion, f),
                            Op::Sound(id) => eprintln!("geno_boss: f{} sound {} (motion {} frame {})", s.frame, id, s.motion, f),
                            _ => {}
                        }
                    }
                    self.op(&e.op, 0);
                }
            }
            f += 1;
        }
        if s.mf > s.tl_mf {
            s.tl_mf = s.mf;
        }
    }

    fn send(&self, msg: u32, part: usize) -> bool {
        if let Some((_, ops)) = self.d.messages.iter().find(|(id, _)| *id == msg) {
            if log_on() {
                eprintln!("geno_boss: f{} message {:#x} from part {}", unsafe { S.frame }, msg, part);
            }
            return self.run(ops, part);
        }
        false
    }

    fn part_procs(&self) {
        for (i, p) in self.d.parts.iter().enumerate().take(MAX_PARTS) {
            if p.role != Role::WeakPoint {
                continue;
            }
            let (hp, zs, dm, ctr) = unsafe { (S.hp[i], S.zero_sent, S.damaged, S.counter[i]) };
            if hp <= 0 {
                if zs >> i & 1 == 0 {
                    unsafe { S.zero_sent |= 1 << i };
                    if let Some(m) = p.on_zero {
                        self.send(m, i);
                    }
                }
                continue;
            }
            if dm >> i & 1 != 0 {
                if let Some(m) = p.on_damaged {
                    self.send(m, i);
                }
                continue;
            }
            let pct = self.pct(i);
            for &(c, x, m) in &p.thresholds {
                if ctr == c && pct <= x {
                    self.send(m, i);
                }
            }
        }
    }

    fn movement(&self) {
        let s = unsafe { &mut S };
        match self.d.kinetics.get(s.kinetic.max(0) as usize).filter(|_| s.kinetic >= 0) {
            Some(Kinetic::Accel { acc, max, .. }) => {
                let a = self.num(*acc);
                let m = self.num(*max);
                s.vx += a * s.aim;
                if s.vx > m {
                    s.vx = m
                }
                if s.vx < -m {
                    s.vx = -m
                }
            }
            Some(Kinetic::Decay(dc)) => {
                let dc = self.num(*dc);
                s.vx = if s.vx > 0.0 { (s.vx - dc).max(0.0) } else { (s.vx + dc).min(0.0) };
            }
            None => {}
        }
        s.x += s.vx;
        if s.has_goal && (s.goal - s.x).abs() <= 0.1 + s.vx.abs() {
            s.x = s.goal;
        }
    }

    fn advance(&self) {
        let s = unsafe { &mut S };
        s.sf += 1;
        let len = self.motion_len();
        if s.mf + 1 >= len {
            if self.d.motions.get(s.motion as usize).is_some_and(|m| m.looping) {
                s.mf = 0;
                s.tl_mf = -1;
                s.inst = s.inst.wrapping_add(1);
            }
        } else {
            s.mf += 1;
        }
    }
}

/// Start a fight: parts at full HP (`hp_scale_pm` per mille of each weak point's HP), the start status entered.
pub fn begin(d: &Define, w: World, difficulty: i32, hp_scale_pm: i32, seed: u32) {
    unsafe {
        S = OFF;
        S.live = true;
        S.rng = seed | 1;
        S.difficulty = difficulty.clamp(0, 14);
        S.strength = d.strength0 as i32;
        S.x = d.spawn_x;
        for (i, (_, v)) in d.vars.iter().enumerate().take(MAX_VARS) {
            S.vars[i] = *v;
        }
    }
    let rt0 = Rt { d, w };
    unsafe {
        for (i, p) in d.parts.iter().enumerate().take(MAX_PARTS) {
            let base = fctiwz_f32(rt0.num(p.hp));
            let hp = if p.role == Role::WeakPoint { (base as i64 * hp_scale_pm.max(1) as i64 / 1000) as i32 } else { base };
            S.hp[i] = hp;
            S.hp_max[i] = hp.max(1);
        }
    }
    let rt = Rt { d, w };
    rt.op(&Op::ResetRules, 0);
    rt.enter(d.start_status);
}

/// Player damage this frame (before [`tick`]): to the core part (routed by the rules) or straight to a part.
pub fn damage(d: &Define, part: usize, amount: i32) {
    if part >= MAX_PARTS || amount <= 0 {
        return;
    }
    unsafe {
        if S.intangible >> part & 1 != 0 {
            return;
        }
        // hitByPlayer is the core's own flag: a weak point is its own object, its hits reach the core only as messages
        let core = d.parts.get(part).is_some_and(|p| p.role == Role::Core);
        if core {
            S.hit = true;
            S.dmg_core += amount;
            S.hp[part] -= amount;
        } else {
            S.hp[part] -= amount;
            S.hit_part |= 1 << part;
            S.damaged |= 1 << part;
        }
    }
}

/// One frame, in the define's frame order: rules, the status's frame ops, timelines, part procs, movement, counters.
pub fn tick(d: &Define, w: World) {
    unsafe {
        if !S.live {
            return;
        }
        S.frame = S.frame.wrapping_add(1);
        S.ev_n = 0;
    }
    let rt = Rt { d, w };
    if unsafe { S.cause } == 0 || unsafe { S.cause } == 2 {
        let st0 = unsafe { S.status };
        for &(lo, hi, on_hit) in &d.passes {
            if on_hit && !unsafe { S.hit } {
                continue;
            }
            rt.evaluate((lo, hi));
            if unsafe { S.status } != st0 {
                break;
            }
        }
    }
    if let Some(sd) = d.status(unsafe { S.status } as u8) {
        if !sd.frame_once || unsafe { S.sf } == 0 {
            rt.run(&sd.frame, 0);
        }
    }
    rt.timeline();
    rt.part_procs();
    rt.movement();
    rt.advance();
    unsafe {
        S.hit = false;
        S.hit_part = 0;
        S.dmg_core = 0;
        S.damaged = 0;
    }
}

pub fn level(d: &Define) -> f32 {
    Rt { d, w: World::default() }.level()
}

pub fn part_level(d: &Define, part: u8) -> f32 {
    Rt { d, w: World::default() }.part_level(part)
}

pub fn num(d: &Define, n: Num) -> f32 {
    Rt { d, w: World::default() }.num(n)
}

pub fn pct(d: &Define, part: usize) -> f32 {
    Rt { d, w: World::default() }.pct(part)
}

pub fn stop() {
    unsafe { S = OFF };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Define {
        // two statuses: 0 waits T=3 frames then goes to 1; 1 waits for its motion to end and goes back
        Define {
            name: "tiny".into(),
            parts: vec![
                Part { name: "core".into(), role: Role::Core, hp: Num::K(100.0), on_damaged: None, thresholds: vec![], on_zero: None, intangible_in: 0, int_level: false },
                Part { name: "wp".into(), role: Role::WeakPoint, hp: Num::Lin(5.0, 2.0), on_damaged: Some(1), thresholds: vec![(0, 50.0, 2)], on_zero: None, intangible_in: 0, int_level: true },
            ],
            vars: vec![("T".into(), 0.0), ("hit".into(), 0.0)],
            statuses: vec![
                Status { index: 0, name: "A".into(), enter: vec![Op::ResetRules, Op::EnableRules(1 << 1), Op::SetTimer(0, Num::K(3.0), Num::K(3.0)), Op::Motion(0)], frame: vec![], frame_once: false, intangible: false },
                Status { index: 1, name: "B".into(), enter: vec![Op::ResetRules, Op::EnableRules(1 << 2), Op::Motion(1)], frame: vec![], frame_once: false, intangible: false },
            ],
            rules: vec![
                Rule { index: 0, priority: -200, default_on: true, cond: Cond::Hit(HitKind::Any), ops: vec![Op::Forward(None)], to: None, per_target: false },
                Rule { index: 1, priority: 200, default_on: false, cond: Cond::TimerDone(0), ops: vec![], to: Some(1), per_target: false },
                Rule { index: 2, priority: 200, default_on: false, cond: Cond::MotionEnd, ops: vec![], to: Some(0), per_target: false },
            ],
            tables: vec![],
            kinetics: vec![],
            motions: vec![
                Motion { name: "m0".into(), len: 100, looping: true, timeline: vec![] },
                Motion { name: "m1".into(), len: 5, looping: false, timeline: vec![TimelineEvent { frame: 1, count: 2, every: 2, op: Op::Shake(1) }] },
            ],
            attacks: vec![],
            messages: vec![(1, vec![Op::SetVar(1, Num::K(1.0))])],
            level_rows: vec![vec![2.5; 15]],
            strength0: 1,
            start_status: 0,
            slot_w: [1.0; 4],
            slot_repeat: 0.3,
            route_mask: 2,
            route_factor: 1.0,
            defeat_status: [None, None],
            goal_snap: true,
            spawn_x: 0.0,
            camera: None,
            window: (-50, 250),
            passes: vec![(-100, -100, false), (-250, -150, true), (-50, 250, false)],
        }
    }

    #[test]
    fn rules_timer_motion_and_routing() {
        let d = tiny();
        let w = World::default();
        begin(&d, w, 5, 1000, 7);
        let mut seen = vec![];
        for _ in 0..12 {
            tick(&d, w);
            seen.push(unsafe { S.status });
        }
        // A for frames 1..3, B entered at the evaluation where sf >= 3, back to A when B's 5-frame motion ends
        assert_eq!(&seen[..5], &[0, 0, 0, 1, 1]);
        assert!(seen[4..].contains(&0));
        // core damage is forwarded to the weak point by the -200 rule, the weak point's proc sends message 1
        damage(&d, 0, 4);
        tick(&d, w);
        unsafe {
            assert_eq!(S.hp[1], 6);
            assert_eq!(S.vars[1], 1.0);
        }
        assert_eq!(part_level(&d, 1), 2.0);
        assert!(state_hash().is_some());
    }
}
