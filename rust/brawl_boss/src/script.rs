//! Scripts (anim-command lists): the 8-byte command container and the typed command set.
//!
//! A script is an array of 8-byte commands ending with an all-zero command. The command word is
//! `[31:24] module, [23:16] code, [15:8] argument count, [7:0] 0`; the second word points at `count` 8-byte arguments
//! `{u32 type, u32 value}` (format spec 3.1). Every command keeps its raw word and its arguments, so a command whose
//! id or argument shape is not one the runner knows is preserved losslessly as [`Op::Unknown`].

use crate::fx::Fx;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// A variable reference: `[31:28]` memory, `[27:24]` kind, `[23:0]` id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var {
    pub mem: u8,
    pub kind: u8,
    pub id: u32,
}
impl Var {
    pub const fn from_raw(v: u32) -> Var {
        Var { mem: (v >> 28) as u8, kind: ((v >> 24) & 0xf) as u8, id: v & 0xff_ffff }
    }
    pub const fn raw(self) -> u32 {
        (self.mem as u32) << 28 | (self.kind as u32) << 24 | self.id
    }
    pub const fn new(mem: u8, kind: u8, id: u32) -> Var {
        Var { mem, kind, id }
    }
}

/// Memory 0: engine values and parameters (read through the accessor).
pub const MEM_PARAM: u8 = 0;
/// Memory 1: the object's work variables.
pub const MEM_WORK: u8 = 1;
/// Memory 2: temporaries (rule engine).
pub const MEM_TEMP: u8 = 2;
pub const KIND_INT: u8 = 0;
pub const KIND_FLOAT: u8 = 1;
pub const KIND_FLAG: u8 = 2;

/// Where a script pointer leads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// Null (an un-relocated zero).
    None,
    /// A script in this file (data-section offset).
    Local(u32),
    /// An imported common script (index into the file's import list).
    Import(u16),
    /// A non-zero value that is neither relocated nor an import field. Kept, never followed.
    Unresolved(u32),
}

impl Default for Target {
    fn default() -> Target {
        Target::None
    }
}

/// One argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg {
    Int(i32),
    /// raw / 60000.
    Scalar(Fx),
    Ptr(Target),
    Bool(bool),
    /// Type 4: an offset that is not relocated (only command 1B02).
    Offset(u32),
    Var(Var),
    /// Type 6: requirement id; `not` is bit 31.
    Req { not: bool, id: u32 },
    /// Any other type tag, kept as is.
    Other { ty: u32, raw: u32 },
}

impl Arg {
    /// Decode from the stored (type, value) pair; pointers are resolved by the caller.
    pub fn from_pair(ty: u32, v: u32) -> Arg {
        match ty {
            0 => Arg::Int(v as i32),
            1 => Arg::Scalar(Fx(v as i32 as i64)),
            2 => Arg::Ptr(if v == 0 { Target::None } else { Target::Unresolved(v) }),
            3 => Arg::Bool(v != 0),
            4 => Arg::Offset(v),
            5 => Arg::Var(Var::from_raw(v)),
            6 => Arg::Req { not: v >> 31 != 0, id: v & 0x7fff_ffff },
            _ => Arg::Other { ty, raw: v },
        }
    }
    /// The stored (type, value) pair; pointers give the target offset (Local) or 0.
    pub fn to_pair(self) -> (u32, u32) {
        match self {
            Arg::Int(i) => (0, i as u32),
            Arg::Scalar(f) => (1, f.0 as i32 as u32),
            Arg::Ptr(t) => (
                2,
                match t {
                    Target::Local(o) | Target::Unresolved(o) => o,
                    _ => 0,
                },
            ),
            Arg::Bool(b) => (3, b as u32),
            Arg::Offset(o) => (4, o),
            Arg::Var(v) => (5, v.raw()),
            Arg::Req { not, id } => (6, (not as u32) << 31 | id),
            Arg::Other { ty, raw } => (ty, raw),
        }
    }
    fn int(&self) -> Option<i32> {
        if let Arg::Int(i) = *self {
            Some(i)
        } else {
            None
        }
    }
    fn var(&self) -> Option<Var> {
        if let Arg::Var(v) = *self {
            Some(v)
        } else {
            None
        }
    }
    fn ptr(&self) -> Option<Target> {
        if let Arg::Ptr(t) = *self {
            Some(t)
        } else {
            None
        }
    }
    /// A number-like argument: int, scalar, bool or a variable (read at run time).
    fn value(&self) -> Option<Value> {
        match *self {
            Arg::Int(i) => Some(Value::Num(Fx::from_int(i as i64))),
            Arg::Scalar(f) => Some(Value::Num(f)),
            Arg::Bool(b) => Some(Value::Num(Fx::from_int(b as i64))),
            Arg::Var(v) => Some(Value::Var(v)),
            _ => None,
        }
    }
}

/// A constant or a variable read at run time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Num(Fx),
    Var(Var),
}

/// A requirement with its arguments (the `if` family and status-change requests).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cond {
    pub not: bool,
    pub req: u32,
    pub args: Vec<Value>,
}

pub mod req {
    /// The current motion has ended.
    pub const MOTION_END: u32 = 2;
    /// Compare a op b (op 0 `<`, 1 `<=`, 2 `==`, 3 `!=`, 4 `>=`, 5 `>`).
    pub const COMPARE: u32 = 7;
    /// A flag is set.
    pub const FLAG: u32 = 8;
    /// Always true.
    pub const ALWAYS: u32 = 15;
    /// A target candidate exists in the slot (args 0, slot).
    pub const SLOT_PRESENT: u32 = 18;
    /// The locked target is on side `arg1` (side 1 = target x < boss x).
    pub const TARGET_SIDE: u32 = 10008;
    /// A target is locked and valid.
    pub const TARGET_LOCKED: u32 = 10010;
}

/// The 47 arguments of command 0619 (level-scaled hitbox), typed. Scaled field = base + slope * L.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HitboxArgs {
    pub id: Value,
    pub part: Value,
    pub bone: Value,
    pub damage: (Value, Value),
    pub angle: Value,
    pub kbg: (Value, Value),
    pub wkb: (Value, Value),
    pub bkb: (Value, Value),
    pub size: (Value, Value),
    pub offset: [Value; 3],
    pub element: Value,
    pub trip: Value,
    pub hitlag: Value,
    pub sdi: Value,
    /// Arguments 21..=45 as stored (meanings are [L] in the spec; see [`crate::runner::ResolvedHitbox`]).
    pub flags: [Arg; 25],
    /// Argument 46: the level variable (body: float 12, cage: its float 0).
    pub level: Value,
}

/// A particle effect (1100: 16 arguments; 1101 / 1102: 10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectArgs {
    /// bank << 16 | index.
    pub id: i32,
    pub bone: i32,
    pub offset: [Fx; 3],
    pub rotation: [Fx; 3],
    pub scale: Fx,
    /// Only 1100.
    pub random_offset: [Fx; 3],
    /// Only 1100.
    pub random_rotation: [Fx; 3],
    pub flag: bool,
}

/// Which effect command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    /// 1100: at a bone.
    AtBone,
    /// 1101: follows the bone [M].
    Follow,
    /// 1102: not attached / ends with the motion [L].
    Detached,
}

/// Which sound command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundKind {
    /// 0A00.
    Play,
    /// 0A02: stopped on status change [L].
    StatusBound,
    /// 0A05: voice / defeat [L].
    Voice,
}

/// The typed command. 63 command ids occur in Petey's file (the spec says 64; see the notes' question list).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    // ---- control (module 00)
    /// 0001: wait n frames.
    Wait(Value),
    /// 0002: wait until motion frame f.
    WaitUntil(Value),
    /// 0004: loop n times (-1 = forever).
    LoopStart(i32),
    /// 0005.
    LoopEnd,
    /// 0007.
    Call(Target),
    /// 0008.
    Return,
    /// 000A.
    If(Cond),
    /// 000B: joins the previous if with AND.
    And(Cond),
    /// 000C: joins the previous if with OR.
    Or(Cond),
    /// 000D.
    ElseIf(Cond),
    /// 000E.
    Else,
    /// 000F.
    EndIf,
    /// 0010: weighted switch over the weighted-list parameter `list` with `count` entries; `unknown` is the
    /// unexplained first argument (1000 in every use).
    Switch { unknown: i32, list: Var, count: i32 },
    /// 0011.
    Case(i32),
    /// 0012.
    Default,
    /// 0013.
    EndSwitch,
    /// 0018.
    Break,
    // ---- status / motion
    /// 0202: request status `status` with priority `prio` when `cond` holds.
    ChangeStatus { prio: i32, status: i32, cond: Cond },
    /// 0409: change motion (from frame 0).
    ChangeMotion(i32),
    /// 0410: variant-only cage setup [L].
    Op0410(i32, i32),
    // ---- hitboxes (module 06)
    /// 0604.
    ClearHitboxes,
    /// 0605: hurtbox state (2 = intangible).
    HurtState(i32),
    /// 0619.
    Hitbox(Box<HitboxArgs>),
    /// 0808: landing / ground reset [M].
    GroundReset,
    /// 0900 n [M].
    Op0900(i32),
    // ---- sound, effect, shake
    Sound(SoundKind, i32),
    Effect(EffectKind, Box<EffectArgs>),
    /// 1A00: screen shake 0 / 1 / 2.
    Shake(i32),
    /// 1B02: closes a rule action; offset back into the rule; no runner effect [L].
    RuleClose(u32),
    // ---- variables (module 12)
    /// 1200: int var = value.
    SetInt(Value, Var),
    /// 1203.
    IncInt(Var),
    /// 1206: float var = value.
    SetFloat(Value, Var),
    /// 1208: float b -= a.
    SubFloat(Value, Var),
    /// 120A.
    SetFlag(Var),
    /// 120B.
    ClearFlag(Var),
    /// 120F: float b *= a.
    MulFloat(Value, Var),
    // ---- params, stage (module 65)
    /// 6505.
    StageEvent(i32),
    /// 6507: out = uniform random in the range record `param`.
    RandomRange { param: u32, out: Var },
    /// 6509: out = a + b * L.
    Linear { param: u32, out: Var },
    /// 650F: spawn a cage from a spawn record.
    SpawnCage { param: u32, arg: i32 },
    /// 6514: set the absolute goal X.
    SetGoalX(Value),
    /// 651D.
    CommonEffect(i32),
    /// 651F: out = the record's first float.
    FirstFloat { param: u32, out: Var },
    /// 6523: strength = param; recompute L.
    SetStrength { param: u32 },
    // ---- movement (module 66)
    /// 6600.
    StartMove { slot: i32, kind: i32, angle: Option<Var> },
    /// 6607 (-1 = all).
    StopMove(i32),
    /// 660B.
    SetMoveKind { slot: i32, kind: i32 },
    /// 660C.
    SetMoveDir { slot: i32, angle: Var },
    // ---- targets (module 69) and rules (6A, 6B)
    /// 6900.
    LockTarget(Var),
    /// 6901: angle (degrees) from the boss to the target into `out`.
    TargetAngle { slot: Var, a: Value, b: Value, out: Var, flag: bool },
    /// 6907 / 6908: target x relative to the boss into `out` [M].
    TargetRelX { code: u8, slot: Var, out: Var },
    /// 6A00.
    EnableRule(i32),
    /// 6A02.
    ResetRules,
    /// 6A03.
    ClearTarget,
    /// 6B03: death check, dead when value <= threshold.
    DeathCheck { threshold: Value, value: Value },
    // ---- links (module 6E)
    /// 6E00: int event to a cage (variables are truncated to int).
    LinkInt { link: i32, event: i32, args: Vec<Value> },
    /// 6E01: float event.
    LinkFloat { link: i32, event: i32, int: i32, value: Value },
    /// 6E02: make the cage run a script now.
    LinkRun { link: i32, script: Target },
    /// Anything else: kept losslessly in [`Command::args`].
    Unknown,
}

/// One decoded command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    /// Data-section offset of the command.
    pub at: u32,
    /// The raw command word.
    pub word: u32,
    pub args: Vec<Arg>,
    pub op: Op,
}

impl Command {
    /// `module << 8 | code`.
    pub fn id(&self) -> u16 {
        (self.word >> 16) as u16
    }
    pub fn decode(at: u32, word: u32, args: Vec<Arg>) -> Command {
        let op = decode_op((word >> 16) as u16, &args).unwrap_or(Op::Unknown);
        Command { at, word, args, op }
    }
}

fn cond(a: &[Arg]) -> Option<Cond> {
    let Arg::Req { not, id } = *a.first()? else { return None };
    let args = a[1..].iter().map(|x| x.value()).collect::<Option<Vec<_>>>()?;
    Some(Cond { not, req: id, args })
}

fn vals<const N: usize>(a: &[Arg]) -> Option<[Value; N]> {
    let mut o = [Value::Num(Fx::ZERO); N];
    for i in 0..N {
        o[i] = a.get(i)?.value()?;
    }
    Some(o)
}

fn fx(a: &Arg) -> Option<Fx> {
    match *a {
        Arg::Scalar(f) => Some(f),
        Arg::Int(i) => Some(Fx::from_int(i as i64)),
        _ => None,
    }
}

fn effect(a: &[Arg], long: bool) -> Option<EffectArgs> {
    let n = if long { 16 } else { 10 };
    if a.len() != n {
        return None;
    }
    let f = |i: usize| fx(&a[i]);
    let flag = matches!(a[n - 1], Arg::Bool(true));
    if !matches!(a[n - 1], Arg::Bool(_)) {
        return None;
    }
    Some(EffectArgs {
        id: a[0].int()?,
        bone: a[1].int()?,
        offset: [f(2)?, f(3)?, f(4)?],
        rotation: [f(5)?, f(6)?, f(7)?],
        scale: f(8)?,
        random_offset: if long { [f(9)?, f(10)?, f(11)?] } else { [Fx::ZERO; 3] },
        random_rotation: if long { [f(12)?, f(13)?, f(14)?] } else { [Fx::ZERO; 3] },
        flag,
    })
}

fn hitbox(a: &[Arg]) -> Option<HitboxArgs> {
    if a.len() != 47 {
        return None;
    }
    let v = |i: usize| a[i].value();
    let mut flags = [Arg::Int(0); 25];
    flags.copy_from_slice(&a[21..46]);
    Some(HitboxArgs {
        id: v(0)?,
        part: v(1)?,
        bone: v(2)?,
        damage: (v(3)?, v(4)?),
        angle: v(5)?,
        kbg: (v(6)?, v(7)?),
        wkb: (v(8)?, v(9)?),
        bkb: (v(10)?, v(11)?),
        size: (v(12)?, v(13)?),
        offset: [v(14)?, v(15)?, v(16)?],
        element: v(17)?,
        trip: v(18)?,
        hitlag: v(19)?,
        sdi: v(20)?,
        flags,
        level: v(46)?,
    })
}

fn decode_op(id: u16, a: &[Arg]) -> Option<Op> {
    let n = a.len();
    let exact = |k: usize| if n == k { Some(()) } else { None };
    Some(match id {
        0x0001 => {
            exact(1)?;
            Op::Wait(a[0].value()?)
        }
        0x0002 => {
            exact(1)?;
            Op::WaitUntil(a[0].value()?)
        }
        0x0004 => {
            exact(1)?;
            Op::LoopStart(a[0].int()?)
        }
        0x0005 => {
            exact(0)?;
            Op::LoopEnd
        }
        0x0007 => {
            exact(1)?;
            Op::Call(a[0].ptr()?)
        }
        0x0008 => {
            exact(0)?;
            Op::Return
        }
        0x000A => Op::If(cond(a)?),
        0x000B => Op::And(cond(a)?),
        0x000C => Op::Or(cond(a)?),
        0x000D => Op::ElseIf(cond(a)?),
        0x000E => {
            exact(0)?;
            Op::Else
        }
        0x000F => {
            exact(0)?;
            Op::EndIf
        }
        0x0010 => {
            exact(3)?;
            Op::Switch { unknown: a[0].int()?, list: a[1].var()?, count: a[2].int()? }
        }
        0x0011 => {
            exact(1)?;
            Op::Case(a[0].int()?)
        }
        0x0012 => {
            exact(0)?;
            Op::Default
        }
        0x0013 => {
            exact(0)?;
            Op::EndSwitch
        }
        0x0018 => {
            exact(0)?;
            Op::Break
        }
        0x0202 => {
            exact(3)?;
            Op::ChangeStatus { prio: a[0].int()?, status: a[1].int()?, cond: cond(&a[2..])? }
        }
        0x0409 => {
            exact(1)?;
            Op::ChangeMotion(a[0].int()?)
        }
        0x0410 => {
            exact(2)?;
            Op::Op0410(a[0].int()?, a[1].int()?)
        }
        0x0604 => {
            exact(0)?;
            Op::ClearHitboxes
        }
        0x0605 => {
            exact(1)?;
            Op::HurtState(a[0].int()?)
        }
        0x0619 => Op::Hitbox(Box::new(hitbox(a)?)),
        0x0808 => {
            exact(0)?;
            Op::GroundReset
        }
        0x0900 => {
            exact(1)?;
            Op::Op0900(a[0].int()?)
        }
        0x0A00 | 0x0A02 | 0x0A05 => {
            exact(1)?;
            let k = match id {
                0x0A00 => SoundKind::Play,
                0x0A02 => SoundKind::StatusBound,
                _ => SoundKind::Voice,
            };
            Op::Sound(k, a[0].int()?)
        }
        0x1100 => Op::Effect(EffectKind::AtBone, Box::new(effect(a, true)?)),
        0x1101 => Op::Effect(EffectKind::Follow, Box::new(effect(a, false)?)),
        0x1102 => Op::Effect(EffectKind::Detached, Box::new(effect(a, false)?)),
        0x1A00 => {
            exact(1)?;
            Op::Shake(a[0].int()?)
        }
        0x1B02 => {
            exact(1)?;
            let Arg::Offset(o) = a[0] else { return None };
            Op::RuleClose(o)
        }
        0x1200 => {
            exact(2)?;
            Op::SetInt(a[0].value()?, a[1].var()?)
        }
        0x1203 => {
            exact(1)?;
            Op::IncInt(a[0].var()?)
        }
        0x1206 => {
            exact(2)?;
            Op::SetFloat(a[0].value()?, a[1].var()?)
        }
        0x1208 => {
            exact(2)?;
            Op::SubFloat(a[0].value()?, a[1].var()?)
        }
        0x120A => {
            exact(1)?;
            Op::SetFlag(a[0].var()?)
        }
        0x120B => {
            exact(1)?;
            Op::ClearFlag(a[0].var()?)
        }
        0x120F => {
            exact(2)?;
            Op::MulFloat(a[0].value()?, a[1].var()?)
        }
        0x6505 => {
            exact(1)?;
            Op::StageEvent(a[0].int()?)
        }
        0x6507 | 0x6509 | 0x651F => {
            exact(2)?;
            let p = a[0].var()?;
            if p.mem != MEM_PARAM {
                return None;
            }
            let out = a[1].var()?;
            match id {
                0x6507 => Op::RandomRange { param: p.id, out },
                0x6509 => Op::Linear { param: p.id, out },
                _ => Op::FirstFloat { param: p.id, out },
            }
        }
        0x650F => {
            exact(2)?;
            Op::SpawnCage { param: a[0].var()?.id, arg: a[1].int()? }
        }
        0x6514 => {
            exact(1)?;
            Op::SetGoalX(a[0].value()?)
        }
        0x651D => {
            exact(1)?;
            Op::CommonEffect(a[0].int()?)
        }
        0x6523 => {
            exact(1)?;
            Op::SetStrength { param: a[0].var()?.id }
        }
        0x6600 => {
            if n != 2 && n != 3 {
                return None;
            }
            Op::StartMove { slot: a[0].int()?, kind: a[1].int()?, angle: if n == 3 { Some(a[2].var()?) } else { None } }
        }
        0x6607 => {
            exact(1)?;
            Op::StopMove(a[0].int()?)
        }
        0x660B => {
            exact(2)?;
            Op::SetMoveKind { slot: a[0].int()?, kind: a[1].int()? }
        }
        0x660C => {
            exact(2)?;
            Op::SetMoveDir { slot: a[0].int()?, angle: a[1].var()? }
        }
        0x6900 => {
            exact(1)?;
            Op::LockTarget(a[0].var()?)
        }
        0x6901 => {
            exact(5)?;
            let Arg::Bool(flag) = a[4] else { return None };
            Op::TargetAngle { slot: a[0].var()?, a: a[1].value()?, b: a[2].value()?, out: a[3].var()?, flag }
        }
        0x6907 | 0x6908 => {
            exact(2)?;
            Op::TargetRelX { code: id as u8, slot: a[0].var()?, out: a[1].var()? }
        }
        0x6A00 => {
            exact(1)?;
            Op::EnableRule(a[0].int()?)
        }
        0x6A02 => {
            exact(0)?;
            Op::ResetRules
        }
        0x6A03 => {
            exact(0)?;
            Op::ClearTarget
        }
        0x6B03 => {
            let [t, v] = vals::<2>(a)?;
            exact(2)?;
            Op::DeathCheck { threshold: t, value: v }
        }
        0x6E00 => {
            if n < 2 {
                return None;
            }
            Op::LinkInt {
                link: a[0].int()?,
                event: a[1].int()?,
                args: a[2..].iter().map(|x| x.value()).collect::<Option<Vec<_>>>()?,
            }
        }
        0x6E01 => {
            exact(4)?;
            Op::LinkFloat { link: a[0].int()?, event: a[1].int()?, int: a[2].int()?, value: a[3].value()? }
        }
        0x6E02 => {
            exact(2)?;
            Op::LinkRun { link: a[0].int()?, script: a[1].ptr()? }
        }
        _ => return None,
    })
}

/// A decoded script: the commands up to (not including) the all-zero terminator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script {
    pub at: u32,
    pub cmds: Vec<Command>,
}

/// Every command id the typed decoder knows (`module << 8 | code`).
pub const KNOWN_IDS: [u16; 63] = [
    0x0001, 0x0002, 0x0004, 0x0005, 0x0007, 0x0008, 0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x000F, 0x0010, 0x0011,
    0x0012, 0x0013, 0x0018, 0x0202, 0x0409, 0x0410, 0x0604, 0x0605, 0x0619, 0x0808, 0x0900, 0x0A00, 0x0A02, 0x0A05,
    0x1100, 0x1101, 0x1102, 0x1200, 0x1203, 0x1206, 0x1208, 0x120A, 0x120B, 0x120F, 0x1A00, 0x1B02, 0x6505, 0x6507,
    0x6509, 0x650F, 0x6514, 0x651D, 0x651F, 0x6523, 0x6600, 0x6607, 0x660B, 0x660C, 0x6900, 0x6901, 0x6907, 0x6908,
    0x6A00, 0x6A02, 0x6A03, 0x6B03, 0x6E00, 0x6E01, 0x6E02,
];
