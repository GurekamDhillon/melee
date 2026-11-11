//! What the runner asks of, and tells, the game. Phase 2 implements [`World`] on top of the game; [`Recorder`] is the
//! recording implementation used by the tests.

use crate::fx::Fx;
use crate::script::{EffectArgs, EffectKind, SoundKind, Target};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Which object an event or a script belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Obj {
    Boss,
    /// 0 = left (Peach), 1 = right (Zelda).
    Cage(u8),
}

/// When an event happened: the runner's frame counter and the emitting object's motion and motion frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub tick: u32,
    pub motion: i32,
    pub frame: i32,
}

/// The shared damage checks behind the imported `CommonAICheck*` conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DamageCheck {
    Air,
    Ground,
    Turn,
    NoReaction,
    ReactionEnd,
}

/// A hitbox with the level slopes applied (base + slope * L; integer fields truncated toward zero).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedHitbox {
    pub id: i32,
    pub part: i32,
    pub bone: i32,
    pub damage: i32,
    /// As stored (the landing centre is -90: straight down; Melee would use 270).
    pub angle: i32,
    pub kbg: i32,
    pub wkb: i32,
    pub bkb: i32,
    pub size: Fx,
    pub offset: [Fx; 3],
    pub element: i32,
    pub trip: Fx,
    pub hitlag: Fx,
    pub sdi: Fx,
    /// Arguments 21-45 as ints (bools as 0 / 1, scalars truncated).
    pub flags: [i32; 25],
    /// The L the box was scaled with.
    pub level: Fx,
}
impl ResolvedHitbox {
    /// a23, best reading [L].
    pub fn shield_damage(&self) -> i32 {
        self.flags[2]
    }
    /// a29 [L].
    pub fn hit_sound(&self) -> i32 {
        self.flags[8]
    }
    /// a31 [L].
    pub fn target_mask(&self) -> i32 {
        self.flags[10]
    }
    /// a36 [L].
    pub fn rehit(&self) -> i32 {
        self.flags[15]
    }
}

/// The 16 imported common scripts, implemented natively (format spec 4.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Builtin {
    StandBy,
    StandByInit,
    StandByExit,
    CheckDamageAir,
    CheckDamageGround,
    CheckDamageTurn,
    CheckDamageNoReaction,
    CheckReactionEnd,
    CheckMotionEnd,
    CheckDeathFigure,
    NoneFlyDamageProc,
    ExecDamageTurnNoRot,
    DyingFigureProcInit,
    DyingFigureProc,
    DyingNoneEffProc,
    BossDown,
}

impl Builtin {
    pub const ALL: [(&'static str, Builtin); 16] = [
        ("statusAnimCmd_CommonStandBy", Builtin::StandBy),
        ("statusAnimCmdInit_CommonStandBy", Builtin::StandByInit),
        ("statusAnimCmdExit_CommonStandBy", Builtin::StandByExit),
        ("statusAnimCmd_CommonAICheckDamageAir", Builtin::CheckDamageAir),
        ("statusAnimCmd_CommonAICheckDamageGround", Builtin::CheckDamageGround),
        ("statusAnimCmd_CommonAICheckDamageTurn", Builtin::CheckDamageTurn),
        ("statusAnimCmd_CommonAICheckDamageNoReaction", Builtin::CheckDamageNoReaction),
        ("statusAnimCmd_CommonAICheckReactionEnd", Builtin::CheckReactionEnd),
        ("statusAnimCmd_CommonAICheckMotionEnd", Builtin::CheckMotionEnd),
        ("statusAnimCmd_CommonAICheckDeathFigure", Builtin::CheckDeathFigure),
        ("statusAnimCmd_CommonNoneFlyDamageProc", Builtin::NoneFlyDamageProc),
        ("statusAnimCmd_CommonAIExecDamageTurnNoRot", Builtin::ExecDamageTurnNoRot),
        ("statusAnimCmd_CommonDyingFigureProcInit", Builtin::DyingFigureProcInit),
        ("statusAnimCmd_CommonDyingFigureProc", Builtin::DyingFigureProc),
        ("statusAnimCmd_CommonDyingNoneEffProc", Builtin::DyingNoneEffProc),
        ("gameAnimCmd_CommonBossDown", Builtin::BossDown),
    ];
    pub fn from_name(n: &str) -> Option<Builtin> {
        Self::ALL.iter().find(|x| x.0 == n).map(|x| x.1)
    }
}

/// Everything the runner does to the world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// 0409 (boss) or link event 5104 (cage).
    Motion(i32),
    Op0410(i32, i32),
    Hitbox(ResolvedHitbox),
    ClearHitboxes,
    /// 0605 (2 = intangible).
    HurtState(i32),
    /// 0808.
    GroundReset,
    Op0900(i32),
    Sound(SoundKind, i32),
    Effect(EffectKind, EffectArgs),
    Shake(i32),
    StageEvent(i32),
    /// 650F: spawn a cage from the record `param` (side = the record's word 1).
    SpawnCage { param: u32, side: i32, arg: i32 },
    GoalX(Fx),
    CommonEffect(i32),
    /// 6523 / spawn: strength and the resulting L.
    Strength { strength: i32, level: Fx },
    StartMove { slot: i32, kind: i32, angle: Option<Fx> },
    StopMove(i32),
    MoveKind { slot: i32, kind: i32 },
    MoveDir { slot: i32, angle: Fx },
    TargetLocked(i32),
    TargetCleared,
    RulesReset,
    RuleEnabled(i32),
    /// A rule fired (index, slot).
    RuleFired { rule: usize, slot: Option<i32> },
    Status { from: i32, to: i32 },
    /// 6B03 fired: HP set to 0.
    Dead,
    LinkInt { link: i32, event: i32, args: Vec<i32> },
    LinkFloat { link: i32, event: i32, int: i32, value: Fx },
    LinkRun { link: i32, script: Target },
    /// A built-in with a world-side effect ran (damage reactions, dying sequences).
    Builtin(Builtin),
    /// A command the runner keeps but does not execute (raw command word).
    Unknown(u32),
    /// A call to a script that cannot be resolved.
    BadCall(Target),
    /// A thread ran 100 000 commands in one frame without waiting and was stopped.
    Runaway,
}

/// The game, as seen by the runner.
pub trait World {
    /// Length in frames of a motion's animation (the CHR0's frame count; the file does not carry it).
    fn motion_length(&mut self, obj: Obj, motion: i32) -> i32;
    /// A player is present and selectable in slot 0-3.
    fn slot_present(&mut self, slot: i32) -> bool;
    /// Target x minus boss x for the player in `slot`.
    fn target_dx(&mut self, slot: i32) -> Option<Fx>;
    /// Angle in degrees (0 = +x, counter-clockwise) from the boss to the player in `slot`.
    fn target_angle(&mut self, slot: i32) -> Option<Fx>;
    /// Engine values read by id (memory 0) that are not in the file and not kept by the runner: 4001 / 4002 (cage HP
    /// percent).
    fn engine_value(&mut self, id: u32) -> Option<Fx>;
    /// The shared damage checks, for this frame.
    fn damage_check(&mut self, kind: DamageCheck) -> bool;
    /// A requirement id the runner does not know.
    fn requirement(&mut self, _obj: Obj, _id: u32, _args: &[Fx]) -> bool {
        false
    }
    fn event(&mut self, at: Stamp, obj: Obj, ev: Event);
}

/// A recording world for tests: fixed answers, every event logged.
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    /// Motion lengths by index (default 60).
    pub lengths: Vec<i32>,
    pub present: [bool; 4],
    pub dx: [Fx; 4],
    pub angle: [Fx; 4],
    pub values: BTreeMap<u32, Fx>,
    pub damage: Vec<DamageCheck>,
    pub log: Vec<(Stamp, Obj, Event)>,
}

impl World for Recorder {
    fn motion_length(&mut self, _obj: Obj, m: i32) -> i32 {
        self.lengths.get(m as usize).copied().unwrap_or(60)
    }
    fn slot_present(&mut self, slot: i32) -> bool {
        (0..4).contains(&slot) && self.present[slot as usize]
    }
    fn target_dx(&mut self, slot: i32) -> Option<Fx> {
        self.slot_present(slot).then(|| self.dx[slot as usize])
    }
    fn target_angle(&mut self, slot: i32) -> Option<Fx> {
        self.slot_present(slot).then(|| self.angle[slot as usize])
    }
    fn engine_value(&mut self, id: u32) -> Option<Fx> {
        self.values.get(&id).copied()
    }
    fn damage_check(&mut self, kind: DamageCheck) -> bool {
        self.damage.contains(&kind)
    }
    fn event(&mut self, at: Stamp, obj: Obj, ev: Event) {
        self.log.push((at, obj, ev));
    }
}
