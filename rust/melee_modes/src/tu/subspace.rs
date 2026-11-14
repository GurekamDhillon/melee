//! Subspace enemies for Envoy rooms (notes/envoy-subspace.md): Brawl's Subspace Emissary common enemies as plain
//! sim entities beside the fighters.
//!
//! Same shape as the Petey boss (`petey.rs`), generalised to a pool:
//! * every enemy is a record in a static pool (rollback snapshot covered), driven by our own LCG, no wall clock, no
//!   host input. Nothing here runs unless the Envoy sim mod spawns an enemy in an armed run (`gd_envoy_enemy_spawn`
//!   with `which >= 100`), so retail play and the goldens never reach it.
//! * the numbers (HP, damage, angles, knockback, attack frames) are hand-written from reading the enemy parameter
//!   files on the owner's own disc; each scaled field is `base + slope x L`, L being Brawl's difficulty scalar,
//!   which Envoy raises with the loop (`L_BY_TIER`). Speeds, sight ranges and hitbox placement are our own tuning.
//! * the look is presentation only (`subspace_model.rs`): the sim never reads a model, so a run is identical with or
//!   without the owner's files and whether or not a model has finished loading.
//! * player attacks hit an enemy by the same test Petey uses (live fighter and item hitboxes against the enemy's hurt
//!   capsule, latched per hitbox lifetime); an enemy's attack lands on a player as an environment hit through the
//!   engine's own damage log (`ftcoll_8007a06c`), so percent, knockback, hitlag and the damage states are Melee's.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::tu::subspace_model::Source;
use melee_core::ppc::{fctiwz_f32, Ppc};
use melee_fighter::tu::player as pl;
use melee_types::*;

// console-arithmetic helpers (notes/semantics.md)
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn fmax(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
fn fclamp(v: f32, lo: f32, hi: f32) -> f32 {
    fmax(fmin(v, hi), lo)
}
fn fabs(a: f32) -> f32 {
    if a < 0.0 { -a } else { a }
}
fn sgn1(a: f32) -> f32 {
    if a < 0.0 { -1.0 } else { 1.0 }
}

/// Pool size (also the width of the per-hitbox latch masks).
pub const MAX: usize = 8;
pub const KINDS: usize = 7;
pub const K_GOOMBA: usize = 0;
pub const K_PRIMID: usize = 1;
pub const K_BILL: usize = 2;
pub const K_GIANT: usize = 3;
pub const K_KOOPA: usize = 4;
pub const K_HAMMER: usize = 5;
pub const K_JYK: usize = 6;

/// `gd_envoy_enemy_spawn`'s `which` for a Subspace enemy: `SPAWN_BASE + kind + 32 x tier`.
pub const SPAWN_BASE: i32 = 100;
/// Brawl's difficulty scalar L per Envoy tier (tier = loops cleared): Normal, Hard, Very Hard, then on to Intense.
pub const L_BY_TIER: [f32; 5] = [2.5, 4.0, 5.5, 7.0, 9.0];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// Goomba family: patrols, charges a player it sees, hurts only while charging.
    Walker,
    /// Primid: closes in and uses a small set of melee attacks.
    Brawler,
    /// Bullet Bill: flies at a player and explodes.
    Missile,
    /// Jyk: hangs in the air near its spawn point, drifts toward a player it sees, hurts on contact.
    Floater,
}

/// A scaled number: `base + slope x L`.
type Sc = (f32, f32);

/// One attack box (a circle `r` at `fx` in front of the enemy and `y` above its origin).
#[derive(Clone, Copy)]
pub struct Hit {
    pub fx: f32,
    pub y: f32,
    pub r: f32,
    pub dmg: Sc,
    pub angle: i32,
    pub kbg: Sc,
    pub wkb: f32,
    pub bkb: Sc,
}

/// An attack: animation phases played in order, the frames the box is live (counted over the whole attack), the
/// distance at which it is chosen and its weight in the pick.
pub struct Attack {
    pub phases: &'static [(usize, i32)],
    pub active: (i32, i32),
    pub hit: Hit,
    pub reach: f32,
    pub weight: u32,
}

/// (name when facing right, name when facing left, frames, loops)
type AnimDef = (&'static str, &'static str, i32, bool);

pub struct Def {
    pub name: &'static str,
    pub src: Source,
    pub arch: Arch,
    pub hp: Sc,
    pub weight: f32,
    pub walk: f32,
    pub run: f32,
    pub gravity: f32,
    pub fall_max: f32,
    /// hurt capsule: ax, ay, bx, by (x mirrored by facing), radius
    pub hurt: [f32; 5],
    pub scale: f32,
    /// the model turns with the facing (else the clips come in L / R pairs)
    pub yaw_face: bool,
    pub anims: &'static [AnimDef],
    pub attacks: &'static [Attack],
    /// how far it sees a player (x, y)
    pub sight: [f32; 2],
    /// landing on its head: 0 nothing, 1 squashes it, 2 hurts it (and the player bounces either way)
    pub stomp: i32,
}

// animation slots every kind fills (an index into `Def::anims`)
const A_IDLE: usize = 0;
const A_WALK: usize = 1;
const A_RUN: usize = 2;
const A_HOLD: usize = 3;
const A_BRAKE: usize = 4;
const A_HURT: usize = 5;
const A_FALL: usize = 6;
const A_LAND: usize = 7;
const A_DEAD: usize = 8;
const A_X0: usize = 9;

const COMMON: &[&str] = &["stage", "adventure", "adventure_common.arc"];

pub static DEFS: [Def; KINDS] = [
    // Goomba (Brawl: Kuribo). HP 4 + 0.8 L. Its one attack is the charge: 6 + 2.5 L, angle 63, growth 16 + L,
    // base 110. A walking Goomba does not hurt. Landing on its head squashes it.
    Def {
        name: "Goomba",
        src: Source { rel: &["stage", "adventure", "060001.pac"], model: "EnmKuribo", prefix: "EnmKuribo", idle: "WalkR", extra: &[] },
        arch: Arch::Walker,
        hp: (4.0, 0.8),
        weight: 80.0,
        walk: 0.28,
        run: 0.95,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 3.5, 0.0, 6.5, 5.0],
        scale: 1.2,
        yaw_face: false,
        anims: &[
            ("WalkR", "WalkL", 31, true),
            ("WalkR", "WalkL", 31, true),
            ("RunR", "RunL", 21, true),
            ("RunHoldR", "RunHoldL", 17, false),
            ("RunBrakeR", "RunBrakeL", 30, false),
            ("Damage", "Damage", 20, false),
            ("Fall", "Fall", 31, true),
            ("Landing", "Landing", 45, false),
            ("Collapses", "Collapses", 35, false),
        ],
        attacks: &[Attack {
            phases: &[(A_RUN, 21)],
            active: (0, 9999),
            hit: Hit { fx: 3.0, y: 5.0, r: 4.0, dmg: (6.0, 2.5), angle: 63, kbg: (16.0, 1.0), wkb: 0.0, bkb: (110.0, 0.0) },
            reach: 0.0,
            weight: 0,
        }],
        sight: [55.0, 14.0],
        stomp: 1,
    },
    // Primid. HP 10 + 1.4 L. Punch (live 10-16 of 25: 4 + 2 L, angle 30, set knockback 24), kick (10-21 of 34:
    // 5 + 2.1 L, angle 65, set knockback 33), hard punch (wind-up 8 + 41, then live 8-11 of 33: 12 + 2.5 L, angle
    // 40, growth 25 + 0.5 L, base 120).
    Def {
        name: "Primid",
        src: Source { rel: COMMON, model: "EnmPrim", prefix: "EnmPrim", idle: "Wait1", extra: &[] },
        arch: Arch::Brawler,
        hp: (10.0, 1.4),
        weight: 95.0,
        walk: 0.38,
        run: 0.9,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 3.5, 0.0, 9.5, 4.0],
        scale: 1.4,
        yaw_face: true,
        anims: &[
            ("Wait1", "Wait1", 101, true),
            ("Walk", "Walk", 51, true),
            ("Run", "Run", 31, true),
            ("Turn", "Turn", 20, false),
            ("RunBrake", "RunBrake", 25, false),
            ("Damage1", "Damage1", 17, false),
            ("Fall", "Fall", 15, true),
            ("Landing", "Landing", 21, false),
            ("Down", "Down", 121, false),
            ("Punch", "Punch", 31, false),
            ("Kick", "Kick", 42, false),
            ("HardPunchStart", "HardPunchStart", 8, false),
            ("HardPunchHold", "HardPunchHold", 41, true),
            ("HardPunch", "HardPunch", 55, false),
            ("JumpSquat", "JumpSquat", 5, false),
            ("Jump", "Jump", 46, false),
        ],
        attacks: &[
            Attack {
                phases: &[(A_X0, 25)],
                active: (10, 16),
                hit: Hit { fx: 7.0, y: 7.5, r: 5.5, dmg: (4.0, 2.0), angle: 30, kbg: (100.0, 0.0), wkb: 24.0, bkb: (0.0, 0.0) },
                reach: 11.0,
                weight: 45,
            },
            Attack {
                phases: &[(A_X0 + 1, 34)],
                active: (10, 21),
                hit: Hit { fx: 8.0, y: 6.0, r: 6.0, dmg: (5.0, 2.1), angle: 65, kbg: (100.0, 0.0), wkb: 33.0, bkb: (0.0, 0.0) },
                reach: 12.0,
                weight: 35,
            },
            Attack {
                phases: &[(A_X0 + 2, 8), (A_X0 + 3, 41), (A_X0 + 4, 33)],
                active: (57, 60),
                hit: Hit { fx: 9.0, y: 7.5, r: 8.0, dmg: (12.0, 2.5), angle: 40, kbg: (25.0, 0.5), wkb: 0.0, bkb: (120.0, 0.0) },
                reach: 13.0,
                weight: 20,
            },
        ],
        sight: [140.0, 60.0],
        stomp: 0,
    },
    // Bullet Bill (Brawl: Killer). HP 4 + 0.7 L. Contact in flight: 3 + 2 L, angle 70, growth 50, base 106; it then
    // explodes: 6 + 3.2 L, angle 90, growth 35, base 120 - 5 L, radius 14, live for 15 frames.
    Def {
        name: "Bullet Bill",
        src: Source { rel: &["stage", "adventure", "060002.pac"], model: "EnmKiller", prefix: "EnmKiller", idle: "Flight", extra: &[] },
        arch: Arch::Missile,
        hp: (4.0, 0.7),
        weight: 90.0,
        walk: 0.9,
        run: 1.25,
        gravity: 0.07,
        fall_max: 2.4,
        hurt: [-3.5, 0.0, 3.5, 0.0, 6.5],
        scale: 1.2,
        yaw_face: true,
        anims: &[
            ("Flight", "Flight", 43, true),
            ("Flight", "Flight", 43, true),
            ("FlightHoming", "FlightHoming", 61, true),
            ("Flight", "Flight", 43, true),
            ("Flight", "Flight", 43, true),
            ("Damage", "Damage", 19, false),
            ("DamageReturn", "DamageReturn", 30, true),
            ("Flight", "Flight", 43, true),
            ("Damage", "Damage", 19, false),
        ],
        attacks: &[
            Attack {
                phases: &[(A_RUN, 61)],
                active: (0, 9999),
                hit: Hit { fx: 3.5, y: 0.0, r: 5.0, dmg: (3.0, 2.0), angle: 70, kbg: (50.0, 0.0), wkb: 0.0, bkb: (106.0, 0.0) },
                reach: 0.0,
                weight: 0,
            },
            Attack {
                phases: &[(A_HURT, 15)],
                active: (0, 15),
                hit: Hit { fx: 0.0, y: 0.0, r: 14.0, dmg: (6.0, 3.2), angle: 90, kbg: (35.0, 0.0), wkb: 0.0, bkb: (120.0, -5.0) },
                reach: 0.0,
                weight: 0,
            },
        ],
        sight: [400.0, 400.0],
        stomp: 0,
    },
    // Giant Goomba (Brawl: Dekakuribo). HP 18 + 6 L. Charge: 14 + 2.4 L, angle 80, growth 28 + 0.5 L, base
    // 113 - 1.6 L. Its tantrum stomp (live from frame 39 of 121): 12 + 1.5 L, angle 70, growth 30, base 117 - 0.5 L.
    // Landing on its head hurts it and bounces the player; it is not squashed.
    Def {
        name: "Giant Goomba",
        src: Source { rel: &["stage", "adventure", "060004.pac"], model: "EnmDekakuribo", prefix: "EnmDekakuribo", idle: "WalkR", extra: &[] },
        arch: Arch::Walker,
        hp: (18.0, 6.0),
        weight: 200.0,
        walk: 0.22,
        run: 0.8,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 8.0, 0.0, 17.0, 12.0],
        scale: 1.0,
        yaw_face: false,
        anims: &[
            ("WalkR", "WalkL", 61, true),
            ("WalkR", "WalkL", 61, true),
            ("RunR", "RunL", 31, true),
            ("AttackRunStart", "AttackRunStart", 41, false),
            ("RunBrakeR", "RunBrakeL", 30, false),
            ("Damage", "Damage", 19, false),
            ("Fall", "Fall", 31, true),
            ("Landing", "Landing", 45, false),
            ("Collapses", "Collapses", 50, false),
            ("AngryR", "AngryL", 121, false),
            ("Stepped", "Stepped", 24, false),
        ],
        attacks: &[
            Attack {
                phases: &[(A_RUN, 31)],
                active: (0, 9999),
                hit: Hit { fx: 7.0, y: 10.0, r: 9.5, dmg: (14.0, 2.4), angle: 80, kbg: (28.0, 0.5), wkb: 0.0, bkb: (113.0, -1.6) },
                reach: 0.0,
                weight: 0,
            },
            Attack {
                phases: &[(A_X0, 121)],
                active: (39, 62),
                hit: Hit { fx: 0.0, y: 5.0, r: 15.0, dmg: (12.0, 1.5), angle: 70, kbg: (30.0, 0.0), wkb: 0.0, bkb: (117.0, -0.5) },
                reach: 20.0,
                weight: 1,
            },
        ],
        sight: [70.0, 20.0],
        stomp: 2,
    },
    // Koopa Troopa (Brawl: Nokonoko, drawn with the Paratroopa rig EnmPatapata, which carries the walking and shell
    // clips). Approximation (no parameter file read): walks its ledge, and when it sees a player tucks into its shell
    // and slides at it. HP 6 + 1 L. Shell slide: 7 + 2 L, angle 50, growth 30, base 50. Landing on it squashes it.
    Def {
        name: "Koopa",
        src: Source { rel: &["stage", "adventure", "060001.pac"], model: "EnmPatapata", prefix: "EnmPatapata", idle: "Walk", extra: &[] },
        arch: Arch::Walker,
        hp: (6.0, 1.0),
        weight: 85.0,
        walk: 0.32,
        run: 1.6,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 4.0, 0.0, 8.0, 6.0],
        scale: 1.2,
        yaw_face: true,
        anims: &[
            ("Walk", "Walk", 60, true),
            ("Walk", "Walk", 60, true),
            ("ShellMode", "ShellMode", 30, true),
            ("ShellMode", "ShellMode", 12, false),
            ("ShellReturn", "ShellReturn", 20, false),
            ("Damage", "Damage", 20, false),
            ("Fall", "Fall", 30, true),
            ("Landing", "Landing", 20, false),
            ("Damage", "Damage", 30, false),
        ],
        attacks: &[Attack {
            phases: &[(A_RUN, 30)],
            active: (0, 9999),
            hit: Hit { fx: 2.0, y: 4.0, r: 5.0, dmg: (7.0, 2.0), angle: 50, kbg: (30.0, 0.0), wkb: 0.0, bkb: (50.0, 0.0) },
            reach: 0.0,
            weight: 0,
        }],
        sight: [70.0, 14.0],
        stomp: 1,
    },
    // Hammer Bro (Brawl: Hammerbros). Approximation: keeps its distance and throws (the hammer's arc is a box in
    // front of it while the throw is live), or swings on a jump when a player is close. HP 12 + 2 L. Throw: 5 + 1.5 L,
    // angle 50, growth 40, base 40. Jump swing: 8 + 1.5 L, angle 60, growth 45, base 60.
    Def {
        name: "Hammer Bro",
        src: Source { rel: &["stage", "adventure", "060002.pac"], model: "EnmHammerbros", prefix: "EnmHammerbros", idle: "Wait", extra: &[] },
        arch: Arch::Brawler,
        hp: (12.0, 2.0),
        weight: 100.0,
        walk: 0.3,
        run: 0.6,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 3.5, 0.0, 10.0, 4.5],
        scale: 1.3,
        yaw_face: true,
        anims: &[
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Turn", "Turn", 10, false),
            ("Wait", "Wait", 20, false),
            ("Damage", "Damage", 20, false),
            ("Jump", "Jump", 30, true),
            ("Landing", "Landing", 15, false),
            ("Damage", "Damage", 30, false),
            ("Hammer", "Hammer", 40, false),
            ("JumpHammer", "JumpHammer", 40, false),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("JumpSquat", "JumpSquat", 5, false),
            ("Jump", "Jump", 30, false),
        ],
        attacks: &[
            Attack {
                phases: &[(A_X0, 40)],
                active: (14, 34),
                hit: Hit { fx: 22.0, y: 12.0, r: 8.0, dmg: (5.0, 1.5), angle: 50, kbg: (40.0, 0.0), wkb: 0.0, bkb: (40.0, 0.0) },
                reach: 34.0,
                weight: 60,
            },
            Attack {
                phases: &[(A_X0 + 1, 40)],
                active: (12, 22),
                hit: Hit { fx: 8.0, y: 9.0, r: 7.0, dmg: (8.0, 1.5), angle: 60, kbg: (45.0, 0.0), wkb: 0.0, bkb: (60.0, 0.0) },
                reach: 14.0,
                weight: 40,
            },
        ],
        sight: [160.0, 60.0],
        stomp: 2,
    },
    // Jyk (Brawl: Jyakeel). Approximation: a spiked ball that hangs near its spawn point, drifts toward a player it
    // sees (within 40 of home sideways and up, 60 down) and hurts on touch. HP 8 + 1.5 L. Touch: 6 + 1.5 L, angle 45, growth 50,
    // base 40.
    Def {
        name: "Jyk",
        src: Source { rel: &["stage", "adventure", "060004.pac"], model: "EnmJyakeel", prefix: "EnmJyakeel", idle: "Wait", extra: &[] },
        arch: Arch::Floater,
        hp: (8.0, 1.5),
        weight: 70.0,
        walk: 0.25,
        run: 0.25,
        gravity: 0.09,
        fall_max: 2.2,
        hurt: [0.0, 0.0, 0.0, 0.0, 9.0],
        scale: 1.0,
        yaw_face: true,
        anims: &[
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 20, false),
            ("Wait", "Wait", 60, true),
            ("Wait", "Wait", 20, false),
            ("Wait", "Wait", 30, false),
        ],
        attacks: &[Attack {
            phases: &[(A_IDLE, 60)],
            active: (0, 9999),
            hit: Hit { fx: 0.0, y: 0.0, r: 6.5, dmg: (6.0, 1.5), angle: 45, kbg: (50.0, 0.0), wkb: 0.0, bkb: (40.0, 0.0) },
            reach: 0.0,
            weight: 0,
        }],
        sight: [90.0, 60.0],
        stomp: 0,
    },
];

// states
const ST_FALL: i32 = 0;
const ST_IDLE: i32 = 1;
const ST_WALK: i32 = 2;
const ST_HOLD: i32 = 3;
const ST_RUN: i32 = 4;
const ST_BRAKE: i32 = 5;
const ST_ATTACK: i32 = 6;
const ST_HURT: i32 = 7;
const ST_LAUNCH: i32 = 8;
const ST_DEAD: i32 = 9;
const ST_FLY: i32 = 10;
const ST_BOOM: i32 = 11;
const ST_LAND: i32 = 12;
const ST_JUMP: i32 = 13;

/// `kind`: -1 free, -2 a defeated enemy's record kept for the sim's last read, else the kind.
#[derive(Clone, Copy)]
pub struct Enemy {
    pub kind: i32,
    pub handle: i32,
    pub tier: i32,
    pub st: i32,
    pub sf: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub face: f32,
    pub hp: f32,
    /// damage taken so far, hits received, the last attacker's slot + 1 (the sim's records)
    pub taken: f32,
    pub rcv: i32,
    pub last_by: i32,
    pub ground: bool,
    pub lag: i32,
    pub flash: i32,
    pub stun: i32,
    pub an: i32,
    pub af: i32,
    pub atk: i32,
    pub cool: i32,
    /// player slots this attack instance has hit
    pub hit_mask: u32,
    pub dead: bool,
    pub life: i32,
    /// order of spawning (the oldest defeated record is reused first)
    pub born: u32,
    /// a floater's spawn point (it never strays far from it)
    pub home: [f32; 2],
}

const FREE: Enemy = Enemy {
    kind: -1, handle: 0, tier: 0, st: ST_FALL, sf: 0, x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, face: 1.0, hp: 0.0, taken: 0.0, rcv: 0,
    last_by: 0, ground: false, lag: 0, flash: 0, stun: 0, an: 0, af: 0, atk: -1, cool: 0, hit_mask: 0, dead: false, life: 0, born: 0, home: [0.0; 2],
};

pub static mut E: [Enemy; MAX] = [FREE; MAX];
static mut RNG: u32 = 1;
static mut BORN: u32 = 0;
static mut LAST_FRAME: u32 = u32::MAX;
/// Any record in use this match (the tick returns at once otherwise).
static mut ON: bool = false;
/// Per player slot, entity, hitbox: bit n set = enemy n already hit by that live hitbox.
static mut LATCH: [[[u8; 4]; 2]; 4] = [[[0; 4]; 2]; 4];
/// Item hitbox latches: (item gobj, per hitbox enemy bits).
static mut LATCH_IT: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];
/// Engine-side test spawns (`MELEE_SUBSPACE_TEST`) done for this match.
static mut TEST_DONE: bool = false;
/// Match frames ticked since `begin_match`.
static mut CLOCK: i32 = 0;

fn rnd(n: u32) -> u32 {
    unsafe {
        RNG = RNG.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (RNG >> 8).prem(n.max(1))
    }
}

fn level(tier: i32) -> f32 {
    L_BY_TIER[tier.clamp(0, L_BY_TIER.len() as i32 - 1) as usize]
}
fn sc(v: Sc, tier: i32) -> f32 {
    v.0 + v.1 * level(tier)
}

fn logging() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_SUBSPACE_LOG").is_some())
}

/// A new match: the pool is empty. Called from `envoy::match_begin` for every match (armed or not).
pub unsafe fn begin_match(seed: u32) {
    E = [FREE; MAX];
    RNG = seed | 1;
    BORN = 0;
    LAST_FRAME = u32::MAX;
    ON = false;
    LATCH = [[[0; 4]; 2]; 4];
    LATCH_IT = [(0, [0; 4]); 16];
    TEST_DONE = false;
    CLOCK = 0;
}

/// Start loading the looks of the kinds in `mask` (bit = kind) off the game thread; never blocks.
pub fn preload(mask: u32) {
    for k in 0..KINDS {
        if mask & 1u32.pshl(k as i64) != 0 {
            crate::tu::subspace_model::request(k);
        }
    }
}

/// The sim spawns an enemy: `which` = `SPAWN_BASE + kind + 32 x tier`, at world (x, y), facing +1 / -1, under the
/// sim's own `handle` (> 0). Returns the handle, or -1 when refused.
pub unsafe fn sub_spawn(which: i32, x: f32, y: f32, facing: i32, handle: i32) -> i32 {
    let w = which - SPAWN_BASE;
    if w < 0 || handle <= 0 {
        return -1;
    }
    let (kind, tier) = ((w & 31) as usize, w >> 5);
    if kind >= KINDS || E.iter().any(|e| e.kind != -1 && e.handle == handle) {
        return -1;
    }
    // a free record, else the oldest defeated one
    let slot = match E.iter().position(|e| e.kind == -1) {
        Some(i) => i,
        None => {
            let mut best: Option<usize> = None;
            for (i, e) in E.iter().enumerate() {
                if e.kind == -2 && best.map_or(true, |b| e.born < E[b].born) {
                    best = Some(i);
                }
            }
            match best {
                Some(i) => i,
                None => return -1,
            }
        }
    };
    let d = &DEFS[kind];
    BORN = BORN.wrapping_add(1);
    let mut e = FREE;
    e.kind = kind as i32;
    e.handle = handle;
    e.tier = tier.clamp(0, 15);
    e.x = x;
    e.y = y;
    e.face = if facing < 0 { -1.0 } else { 1.0 };
    e.hp = sc(d.hp, e.tier);
    e.born = BORN;
    e.cool = 30;
    if d.arch == Arch::Missile {
        // a missile comes in from the side it flies away from, at a height a player can meet (on Final
        // Destination; a map room's spawn point is the cannon's own place)
        if handle < ROOM_HANDLE_BASE {
            if fabs(e.x) < 140.0 {
                e.x = -e.face * 150.0;
            }
            e.y = fclamp(e.y, 8.0, 40.0);
        }
        e.st = ST_FLY;
        e.an = A_RUN as i32;
        e.life = 480;
        e.vx = e.face * d.run;
        e.atk = 0;
    } else if d.arch == Arch::Floater {
        // on Final Destination it hangs at a height a player can meet; a map room's spawn point is its own place
        if handle < ROOM_HANDLE_BASE {
            e.y = fclamp(e.y, 10.0, 35.0);
        }
        e.home = [e.x, e.y];
        e.st = ST_FLY;
        e.an = A_IDLE as i32;
        e.atk = 0;
    } else {
        e.st = ST_FALL;
        e.an = A_FALL as i32;
    }
    E[slot] = e;
    // nothing the newcomer has not been hit by yet
    for s in LATCH.iter_mut() {
        for en in s.iter_mut() {
            for l in en.iter_mut() {
                *l &= !1u8.pshl(slot as i64);
            }
        }
    }
    for l in LATCH_IT.iter_mut() {
        for b in l.1.iter_mut() {
            *b &= !1u8.pshl(slot as i64);
        }
    }
    ON = true;
    crate::tu::subspace_model::request(kind);
    melee_fighter::tu::ftdrawcommon::BOSS_DRAW = Some(sub_draw);
    eprintln!("subspace: {} spawned (tier {}, hp {:.1}) handle {handle}", d.name, e.tier, e.hp);
    handle
}

/// Handles `spawn` hands out start here, clear of the sim's own small handles.
pub const ROOM_HANDLE_BASE: i32 = 1 << 20;

/// The kind number for a name ("goomba", "primid", "bill" / "bullet_bill", "giant" / "giant_goomba"), or None.
pub fn kind_by_name(name: &str) -> Option<usize> {
    match name.to_ascii_lowercase().as_str() {
        "goomba" => Some(K_GOOMBA),
        "primid" => Some(K_PRIMID),
        "bill" | "bullet_bill" | "bulletbill" => Some(K_BILL),
        "giant" | "giant_goomba" | "giantgoomba" => Some(K_GIANT),
        "koopa" => Some(K_KOOPA),
        "hammer" | "hammer_bro" | "hammerbro" => Some(K_HAMMER),
        "jyk" => Some(K_JYK),
        _ => None,
    }
}

/// A kind's display name (logs).
pub fn kind_name(kind: usize) -> &'static str {
    DEFS.get(kind).map(|d| d.name).unwrap_or("?")
}

/// The room-side spawn (the Subspace map lane): `kind` (K_*), `tier` (Envoy loops cleared, 0..), world (x, y),
/// facing +1 / -1. Picks its own handle (>= ROOM_HANDLE_BASE) and returns it, or -1 when the pool is full.
/// Read it back with `sub_status` / `sub_enemy_i`; `clear_all` removes everything at the room's end.
pub unsafe fn spawn(kind: usize, tier: i32, x: f32, y: f32, facing: i32) -> i32 {
    if kind >= KINDS {
        return -1;
    }
    let mut h = ROOM_HANDLE_BASE;
    while E.iter().any(|e| e.kind != -1 && e.handle == h) {
        h += 1;
    }
    sub_spawn(SPAWN_BASE + kind as i32 + 32 * tier.clamp(0, 15), x, y, facing, h)
}

fn by_handle(handle: i32) -> Option<&'static Enemy> {
    unsafe { E.iter().find(|e| e.kind != -1 && e.handle == handle && handle > 0) }
}

/// Whether `handle` is one of ours (alive or defeated).
pub fn owns(handle: i32) -> bool {
    by_handle(handle).is_some()
}

/// The sim's status read: 1 alive, 2 defeated, 0 unknown.
pub fn sub_status(handle: i32) -> i32 {
    match by_handle(handle) {
        Some(e) if e.kind >= 0 => 1,
        Some(_) => 2,
        None => 0,
    }
}

/// The sim's field read (the script-enemy field numbers it already uses): 0 kind, 2 damage taken, 6 hits received,
/// 7 the last attacker's slot + 1; 8 / 9 position x / y (rounded).
pub fn sub_enemy_i(handle: i32, field: i32) -> i32 {
    let Some(e) = by_handle(handle) else { return -1 };
    match field {
        0 => e.kind,
        2 => fctiwz_f32(e.taken + 0.5),
        6 => e.rcv,
        7 => e.last_by,
        8 => fctiwz_f32(e.x),
        9 => fctiwz_f32(e.y),
        _ => -1,
    }
}

/// Live enemies (the checks and the HUD).
pub fn alive() -> i32 {
    unsafe { E.iter().filter(|e| e.kind >= 0).count() as i32 }
}

/// Remove every enemy (the room is won: the stragglers go with it).
pub unsafe fn clear_all() {
    for e in E.iter_mut() {
        if e.kind >= 0 {
            e.kind = -2;
        }
    }
}

/// Remove the map room's own enemies (handles from `spawn`), leaving the Envoy sim's.
pub unsafe fn clear_room() {
    for e in E.iter_mut() {
        if e.kind >= 0 && e.handle >= ROOM_HANDLE_BASE {
            e.kind = -2;
        }
    }
}

// ---------------------------------------------------------------- players

unsafe fn humans() -> i32 {
    if crate::tu::envoy::PLAN[crate::tu::envoy::P_COOP] == 1 { 2 } else { 1 }
}

/// The fighter in play of `slot` (a tag pair's active one).
unsafe fn in_play(slot: i32) -> *mut Fighter {
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let p = pl::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[(*p).transformed[0] as usize & 1];
    if g.is_null() || (*g).user_data.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

unsafe fn entity(slot: i32, i: usize) -> *mut Fighter {
    let p = pl::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[i];
    if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

/// A player's fighter an enemy can see and hit: in play, not dead, asleep or on the revival platform.
unsafe fn target(slot: i32) -> *mut Fighter {
    let f = in_play(slot);
    if f.is_null() {
        return f;
    }
    let m = (*f).motion_id as i32;
    if m <= ftCo_MS_RebirthWait as i32 || m == ftCo_MS_Entry as i32 {
        return core::ptr::null_mut();
    }
    f
}

/// The nearest player to (x, y): (slot, dx, dy to the player's middle).
unsafe fn nearest(x: f32, y: f32) -> Option<(i32, f32, f32)> {
    let mut best: Option<(i32, f32, f32)> = None;
    for s in 0..humans() {
        let f = target(s);
        if f.is_null() {
            continue;
        }
        let (dx, dy) = ((*f).cur_pos.x - x, (*f).cur_pos.y + 5.0 - y);
        if best.map_or(true, |b| dx * dx + dy * dy < b.1 * b.1 + b.2 * b.2) {
            best = Some((s, dx, dy));
        }
    }
    best
}

// ---------------------------------------------------------------- ground

/// The floor under x between two heights (top first), if any.
unsafe fn floor_between(x: f32, top: f32, bottom: f32) -> Option<f32> {
    let mut hit = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let r = melee_world::tu::mplib::mpCheckFloor(
        x, top, x, bottom, 0.0, &mut hit, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), -1, -1, -1, None,
        core::ptr::null_mut(),
    );
    if r != 0 { Some(hit.y) } else { None }
}

/// Outside the stage's blast zone.
unsafe fn out_of_bounds(x: f32, y: f32) -> bool {
    use melee_world::tu::stage as st;
    x > st::Stage_GetBlastZoneRightOffset() || x < st::Stage_GetBlastZoneLeftOffset() || y > st::Stage_GetBlastZoneTopOffset() || y < st::Stage_GetBlastZoneBottomOffset()
}

// ---------------------------------------------------------------- geometry

/// 2D distance between segments pq and ab.
fn seg_dist(p: (f32, f32), q: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    fn pt(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = dx * dx + dy * dy;
        let t = if l > 0.0 { fclamp(((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l, 0.0, 1.0) } else { 0.0 };
        let (ex, ey) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
        melee_runtime::gekko::sqrtf(ex * ex + ey * ey)
    }
    fmin(fmin(pt(p, a, b), pt(q, a, b)), fmin(pt(a, p, q), pt(b, p, q)))
}

/// An enemy's hurt capsule in the world: (a, b, radius).
fn hurt_of(e: &Enemy) -> ((f32, f32), (f32, f32), f32) {
    let d = &DEFS[e.kind as usize];
    let h = d.hurt;
    ((e.x + h[0] * e.face, e.y + h[1]), (e.x + h[2] * e.face, e.y + h[3]), h[4])
}

/// Whether an enemy in this state can be hit.
fn hittable(e: &Enemy) -> bool {
    e.kind >= 0 && !e.dead && e.st != ST_DEAD && e.st != ST_BOOM
}

// ---------------------------------------------------------------- hits on enemies

/// One landed hit on enemy `i`: damage, the hit's knockback numbers, the attacker's x and slot.
unsafe fn receive(i: usize, dmg: f32, angle: i32, kbg: f32, wkb: f32, bkb: f32, from_x: f32, slot: i32) {
    let e = &mut E[i];
    let d = &DEFS[e.kind as usize];
    e.hp -= dmg;
    e.taken += dmg;
    e.rcv += 1;
    e.last_by = slot + 1;
    e.flash = 6;
    e.atk = -1;
    e.hit_mask = 0;
    // Melee's knockback formula with the damage taken so far as the percent
    let p = fmin(e.taken, 300.0);
    let base = if wkb > 0.0 { wkb * 10.0 / 20.0 + 1.0 } else { p / 10.0 + p * dmg / 20.0 };
    let mut kb = (base * (200.0 / (d.weight + 100.0)) * 1.4 + 18.0) * (kbg / 100.0) + bkb;
    let dir = if e.x >= from_x { 1.0 } else { -1.0 };
    if e.hp <= 0.0 {
        e.dead = true;
        kb = fmax(kb, 90.0);
    }
    e.lag = fctiwz_f32(dmg / 3.0 + 3.0).clamp(3, 12);
    if d.arch == Arch::Missile || d.arch == Arch::Floater {
        // a Bullet Bill (or a Jyk) that survives a hit flies on; a beaten one drops
        if e.dead {
            e.st = ST_LAUNCH;
            e.sf = 0;
            e.an = A_FALL as i32;
            e.af = 0;
            e.vx = dir * 0.8;
            e.vy = 0.6;
            e.stun = 40;
            e.ground = false;
        }
        if logging() {
            eprintln!("subspace: f{} {} #{i} hit by slot {slot} for {dmg:.1}, hp {:.1}, kb {kb:.0}{}", LAST_FRAME, d.name, e.hp, if e.dead { " (beaten)" } else { "" });
        }
        return;
    }
    if kb >= 60.0 || e.dead {
        let a = if angle == 361 { if kb < 32.1 { 0 } else { 44 } } else { angle };
        // a hit aimed into the floor pops a grounded enemy up instead
        let a = if e.ground && a > 180 { 360 - a } else { a };
        let rad = a as f32 * 0.017453292;
        let v = kb * 0.03;
        e.vx = v * melee_runtime::gekko::cosf(rad) * dir;
        e.vy = fmax(v * melee_runtime::gekko::sinf(rad), 0.8);
        e.stun = fctiwz_f32(kb * 0.4).clamp(12, 70);
        e.st = ST_LAUNCH;
        e.sf = 0;
        e.an = A_FALL as i32;
        e.af = 0;
        e.ground = false;
        e.face = -dir;
    } else {
        e.vx = dir * kb * 0.012;
        e.stun = fctiwz_f32(kb * 0.4).clamp(8, 28);
        e.st = ST_HURT;
        e.sf = 0;
        e.an = A_HURT as i32;
        e.af = 0;
    }
    if logging() {
        eprintln!("subspace: f{} {} #{i} hit by slot {slot} for {dmg:.1}, hp {:.1}, kb {kb:.0}{}", LAST_FRAME, d.name, e.hp, if e.dead { " (beaten)" } else { "" });
    }
}

/// One live hitbox (segment ab, radius r) against every enemy not yet latched by it. Returns whether it hit.
unsafe fn hit_enemies(a: (f32, f32), b: (f32, f32), r: f32, dmg: f32, h: *const HitCapsule, from_x: f32, slot: i32, latch: &mut u8) -> bool {
    let mut any = false;
    for i in 0..MAX {
        let bit = 1u8.pshl(i as i64);
        if *latch & bit != 0 || !hittable(&E[i]) {
            continue;
        }
        let (ha, hb, hr) = hurt_of(&E[i]);
        if seg_dist(a, b, ha, hb) <= r + hr {
            *latch |= bit;
            any = true;
            receive(i, dmg, (*h).kb_angle, (*h).x24 as f32, (*h).x28 as f32, (*h).x2C as f32, from_x, slot);
        }
    }
    any
}

/// The players' fighter hitboxes and their items' hitboxes against the pool.
unsafe fn player_hits() {
    for s in 0..humans() {
        for en in 0..2usize {
            let f = entity(s, en);
            if f.is_null() {
                continue;
            }
            let ratio = pl::Player_GetAttackRatio(s);
            for i in 0..4 {
                let h: *mut HitCapsule = core::ptr::addr_of_mut!((*f).x914[i]);
                let latch = &mut LATCH[s as usize][en][i];
                if (*h).state == HitCapsule_Disabled {
                    *latch = 0;
                    continue;
                }
                let r = if (*h).x43_b1() != 0 { (*h).scale } else { (*h).scale * (*f).x34_scale.y };
                let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
                let dmg = (*h).damage * ratio;
                if dmg <= 0.0 {
                    continue;
                }
                if hit_enemies(a, b, r, dmg, h, (*f).cur_pos.x, s, latch) {
                    // the attacker's side of a landed hit, as for hitting an item: hitlag next frame, the hit sound
                    let fp = &mut *f;
                    let d = fctiwz_f32(dmg + 0.5).max(1);
                    if (fp.dmg.x1914 as i32) < d {
                        fp.dmg.x1914 = d as _;
                    }
                    let snd = 0x7E + fp.player_idx as i32 * 2 + fp.is_sub_fighter() as i32;
                    melee_runtime::tu::lbcollision::lbColl_80005BB0(h, snd);
                }
            }
        }
    }
    // items and projectiles of the players (and ownerless ones)
    let mut next: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];
    let mut n = 0;
    let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !g.is_null() {
        let ip = (*g).user_data as *mut Item;
        g = (*g).next;
        if ip.is_null() {
            continue;
        }
        let it = &mut *ip;
        let mut slot = -1;
        let owner = it.owner;
        if !owner.is_null() && (*owner).classifier == HSD_GOBJ_CLASS_FIGHTER as _ {
            let of = (*owner).user_data as *mut Fighter;
            if !of.is_null() {
                slot = (*of).player_idx as i32;
                if slot >= humans() {
                    continue; // a foe's projectile passes through its own side
                }
            }
        }
        let key = it.entity as usize;
        let mut lat = LATCH_IT.iter().find(|(k, _)| *k == key).map(|(_, l)| *l).unwrap_or([0; 4]);
        let mut live = false;
        for i in 0..4 {
            let h: *mut HitCapsule = core::ptr::addr_of_mut!(it.x5D4_hitboxes[i].hit);
            if (*h).state == HitCapsule_Disabled {
                lat[i] = 0;
                continue;
            }
            live = true;
            let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
            if (*h).damage > 0.0 && hit_enemies(a, b, (*h).scale, (*h).damage, h, it.pos.x - it.x40_vel.x, slot, &mut lat[i]) {
                melee_runtime::tu::lbcollision::lbColl_80005BB0(h, 0x7E);
            }
        }
        if live && n < next.len() {
            next[n] = (key, lat);
            n += 1;
        }
    }
    LATCH_IT = next;
}

// ---------------------------------------------------------------- hits on players

/// Room for the environment record plus the hit record the damage log reads through the same pointer.
#[repr(C)]
struct EnvHit {
    env: lbColl_80008D30_arg1,
    pad: HitCapsule,
}

/// An enemy's attack lands on `f`: Melee's own damage path with an environment source, launched away from `from_x`.
unsafe fn strike(f: *mut Fighter, from_x: f32, dmg: i32, angle: i32, kbg: i32, wkb: i32, bkb: i32) -> bool {
    let fp = &mut *f;
    let m = fp.motion_id as i32;
    if (ftCo_MS_GuardOn as i32..=ftCo_MS_GuardSetOff as i32).contains(&m) {
        // a raised shield takes it (no knockback, no percent)
        fp.shield_health = fmax(fp.shield_health - dmg as f32, 0.0);
        return true;
    }
    let mut applied = dmg as f32;
    if !melee_fighter::tu::ftcoll::ftcoll_80076640(fp, &mut applied) {
        return true;
    }
    let mut eh: EnvHit = core::mem::zeroed();
    eh.env.damage = dmg.max(0) as u32;
    eh.env.kb_angle = angle;
    eh.env.unkC = kbg.max(0) as u32;
    eh.env.unk10 = wkb.max(0) as u32;
    eh.env.unk14 = bkb.max(0) as u32;
    eh.env.element = HitElement_Normal as u32;
    let mut entry: DmgLogEntry = core::mem::zeroed();
    entry.x0 = EntityKind_UNKNOWN;
    entry.kind = (-10i32) as FighterKind;
    entry.pos = fp.cur_pos;
    entry.x20 = applied;
    entry.size_of_xC = dmg.max(0) as usize;
    entry.xC.unk_anim0 = &mut eh.env;
    entry.x10.hurt1 = core::ptr::addr_of_mut!(fp.hurt_capsules[0]);
    let g = fp.gobj;
    let out = core::ptr::addr_of_mut!(fp.dmg.facing_dir_1) as *mut core::ffi::c_void;
    melee_fighter::tu::ftcoll::ftcoll_8007a06c(g, out, &mut entry, 1, 1);
    // launched away from the enemy (the environment path would use the victim's own facing)
    (*f).dmg.facing_dir_1 = if (*f).cur_pos.x > from_x { -1.0 } else { 1.0 };
    melee_fighter::tu::fighter::Fighter_procCollResolve(g);
    true
}

/// A player's body for an enemy's box: a vertical segment through the fighter and a radius.
unsafe fn body_of(f: *mut Fighter) -> ((f32, f32), (f32, f32), f32) {
    let s = (*f).x34_scale.y;
    let (x, y) = ((*f).cur_pos.x, (*f).cur_pos.y);
    ((x, y + 3.0 * s), (x, y + 9.0 * s), 3.5 * s)
}

/// Enemy `i`'s attack `k` this frame against the players it has not hit yet.
unsafe fn attack_players(i: usize, k: usize) {
    let e = E[i];
    let d = &DEFS[e.kind as usize];
    let h = d.attacks[k].hit;
    let c = (e.x + h.fx * e.face, e.y + h.y);
    for s in 0..humans() {
        if e.hit_mask & 1u32.pshl(s as i64) != 0 {
            continue;
        }
        let f = target(s);
        if f.is_null() || (*f).x1988 != 0 || (*f).x198C != 0 {
            continue;
        }
        let (a, b, r) = body_of(f);
        if seg_dist(c, c, a, b) > h.r + r {
            continue;
        }
        let dmg = fctiwz_f32(sc(h.dmg, e.tier)).max(1);
        let kbg = fctiwz_f32(sc(h.kbg, e.tier));
        let bkb = fctiwz_f32(sc(h.bkb, e.tier));
        strike(f, e.x, dmg, h.angle, kbg, fctiwz_f32(h.wkb), bkb);
        E[i].hit_mask |= 1u32.pshl(s as i64);
        E[i].lag = fctiwz_f32(dmg as f32 / 3.0 + 3.0).clamp(3, 10);
        if E[i].kind as usize == K_KOOPA {
            // a shell that connects stops and sits still for a while: the opening to hit it back
            E[i].vx = 0.0;
            E[i].lag = 45;
        }
        if logging() {
            eprintln!("subspace: f{} {} #{i} attack {k} hits slot {s} for {dmg} (angle {} kbg {kbg} bkb {bkb})", LAST_FRAME, d.name, h.angle);
        }
    }
}

/// A player landing on enemy `i`'s head: the enemy is squashed or hurt, the player bounces.
unsafe fn stomped(i: usize) -> bool {
    let e = E[i];
    let d = &DEFS[e.kind as usize];
    if d.stomp == 0 || !hittable(&e) || !e.ground {
        return false;
    }
    let head = e.y + d.hurt[3] + d.hurt[4] * 0.6;
    for s in 0..humans() {
        let f = target(s);
        if f.is_null() || (*f).ground_or_air == GA_Ground || (*f).self_vel.y >= -0.2 {
            continue;
        }
        let (px, py) = ((*f).cur_pos.x, (*f).cur_pos.y);
        if fabs(px - e.x) > d.hurt[4] + 2.5 || py > head + 4.0 || py < head - 5.0 {
            continue;
        }
        (*f).self_vel.y = 2.4;
        let en = &mut E[i];
        en.last_by = s + 1;
        en.rcv += 1;
        en.flash = 6;
        en.vx = 0.0;
        en.sf = 0;
        en.af = 0;
        en.atk = -1;
        if d.stomp == 1 {
            let left = fmax(en.hp, 1.0);
            en.taken += left;
            en.hp = 0.0;
            en.dead = true;
            en.st = ST_DEAD;
            en.an = A_DEAD as i32;
        } else {
            let dmg = 10.0;
            en.taken += dmg;
            en.hp -= dmg;
            en.st = ST_HURT;
            en.stun = 24;
            en.an = (A_X0 + 1) as i32; // "Stepped"
            if en.hp <= 0.0 {
                en.dead = true;
                en.st = ST_DEAD;
                en.an = A_DEAD as i32;
            }
        }
        if logging() {
            eprintln!("subspace: f{} {} #{i} stomped by slot {s}, hp {:.1}", LAST_FRAME, d.name, E[i].hp);
        }
        return true;
    }
    false
}

// ---------------------------------------------------------------- behaviour

unsafe fn set(e: &mut Enemy, st: i32, an: usize) {
    e.st = st;
    e.sf = 0;
    e.an = an as i32;
    e.af = 0;
}

/// The attack's frame count and the animation slot / frame it shows at attack frame `t`.
fn attack_len(a: &Attack) -> i32 {
    a.phases.iter().map(|p| p.1).sum()
}
fn attack_pose(a: &Attack, t: i32) -> (usize, i32) {
    let mut t0 = 0;
    for p in a.phases {
        if t < t0 + p.1 {
            return (p.0, t - t0);
        }
        t0 += p.1;
    }
    let l = a.phases[a.phases.len() - 1];
    (l.0, l.1 - 1)
}

/// Gravity, the fall and the landing; true when it stands on a floor after the step.
unsafe fn physics(e: &mut Enemy, d: &Def) {
    if e.ground {
        // follow the floor; a step off an edge starts a fall
        match floor_between(e.x + e.vx, e.y + 4.0, e.y - 4.0) {
            Some(fy) => {
                e.x += e.vx;
                e.y = fy;
                e.vy = 0.0;
            }
            None => {
                e.x += e.vx;
                e.ground = false;
                e.vy = 0.0;
            }
        }
    } else {
        e.vy = fmax(e.vy - d.gravity, -d.fall_max);
        let (nx, ny) = (e.x + e.vx, e.y + e.vy);
        if e.vy <= 0.0 {
            if let Some(fy) = floor_between(nx, e.y + 0.5, ny - 0.1) {
                e.x = nx;
                e.y = fy;
                e.vy = 0.0;
                e.ground = true;
                return;
            }
        }
        e.x = nx;
        e.y = ny;
    }
}

/// Whether the floor ends within `ahead` in front of a grounded enemy.
unsafe fn cliff_ahead(e: &Enemy, ahead: f32) -> bool {
    floor_between(e.x + e.face * ahead, e.y + 4.0, e.y - 6.0).is_none()
}

unsafe fn walker(i: usize) {
    let d = &DEFS[E[i].kind as usize];
    let near = nearest(E[i].x, E[i].y + d.hurt[1]);
    let e = &mut E[i];
    e.cool = (e.cool - 1).max(0);
    match e.st {
        ST_IDLE | ST_WALK => {
            if e.st == ST_IDLE {
                set(e, ST_WALK, A_WALK);
            }
            if cliff_ahead(e, d.hurt[4] + 2.0) {
                e.face = -e.face;
            }
            e.vx = e.face * d.walk;
            if let Some((_, dx, dy)) = near {
                if e.cool == 0 && fabs(dx) < d.sight[0] && fabs(dy) < d.sight[1] {
                    // the tantrum when a player is right on top of it, else the charge
                    let close = d.attacks.len() > 1 && fabs(dx) < d.attacks[1].reach;
                    e.face = sgn1(dx);
                    e.vx = 0.0;
                    if close {
                        e.atk = 1;
                        e.hit_mask = 0;
                        set(e, ST_ATTACK, d.attacks[1].phases[0].0);
                    } else {
                        set(e, ST_HOLD, A_HOLD);
                    }
                }
            }
        }
        ST_HOLD => {
            e.vx = 0.0;
            if e.sf >= d.anims[A_HOLD].2 {
                e.atk = 0;
                e.hit_mask = 0;
                set(e, ST_RUN, A_RUN);
            }
        }
        ST_RUN => {
            e.vx = e.face * d.run;
            let passed = near.map_or(true, |(_, dx, _)| dx * e.face < -18.0);
            if passed || e.sf > 110 || cliff_ahead(e, d.hurt[4] + 3.0) {
                e.atk = -1;
                set(e, ST_BRAKE, A_BRAKE);
            }
        }
        ST_BRAKE => {
            e.vx *= 0.85;
            if cliff_ahead(e, d.hurt[4] + 1.0) {
                e.vx = 0.0;
            }
            if e.sf >= d.anims[A_BRAKE].2 {
                e.cool = 45 + rnd(45) as i32;
                set(e, ST_WALK, A_WALK);
            }
        }
        ST_ATTACK => {
            e.vx = 0.0;
            let a = &d.attacks[e.atk.max(0) as usize];
            if e.sf >= attack_len(a) {
                e.atk = -1;
                e.cool = 60 + rnd(60) as i32;
                set(e, ST_WALK, A_WALK);
            }
        }
        _ => {}
    }
}

unsafe fn brawler(i: usize) {
    let d = &DEFS[E[i].kind as usize];
    let near = nearest(E[i].x, E[i].y + 5.0);
    let e = &mut E[i];
    e.cool = (e.cool - 1).max(0);
    match e.st {
        ST_IDLE => {
            e.vx = 0.0;
            if let Some((_, dx, dy)) = near {
                if fabs(dx) < d.sight[0] && fabs(dy) < d.sight[1] {
                    if fabs(dx) > 3.0 {
                        e.face = sgn1(dx);
                    }
                    let reach = d.attacks.iter().map(|a| a.reach).fold(0.0, fmax);
                    if fabs(dx) > reach - 2.0 || fabs(dy) > 9.0 {
                        if e.cool < 20 {
                            set(e, ST_WALK, A_WALK);
                        }
                    } else if e.cool == 0 {
                        // a weighted pick among the attacks that reach
                        let mut w = [0u32; 8];
                        let mut sum = 0;
                        for (k, a) in d.attacks.iter().enumerate().take(8) {
                            if fabs(dx) <= a.reach {
                                w[k] = a.weight;
                                sum += a.weight;
                            }
                        }
                        if sum > 0 {
                            let mut r = rnd(sum);
                            let mut pickd = 0;
                            for (k, x) in w.iter().enumerate() {
                                if *x == 0 {
                                    continue;
                                }
                                if r < *x {
                                    pickd = k;
                                    break;
                                }
                                r -= *x;
                            }
                            e.atk = pickd as i32;
                            e.hit_mask = 0;
                            set(e, ST_ATTACK, d.attacks[pickd].phases[0].0);
                        }
                    }
                }
            }
        }
        ST_WALK => {
            match near {
                Some((_, dx, dy)) => {
                    e.face = sgn1(dx);
                    e.vx = e.face * d.walk;
                    if cliff_ahead(e, 4.0) {
                        e.vx = 0.0;
                        // a player above or across a gap: jump for it now and then
                        if e.cool == 0 {
                            e.cool = 90;
                            e.vy = 2.9;
                            e.vx = e.face * d.run;
                            e.ground = false;
                            set(e, ST_JUMP, A_X0 + 6);
                        }
                    } else if fabs(dx) < 10.0 && fabs(dy) <= 9.0 {
                        set(e, ST_IDLE, A_IDLE);
                    } else if dy > 14.0 && fabs(dx) < 34.0 && e.cool == 0 {
                        e.cool = 100 + rnd(60) as i32;
                        e.vy = 2.9;
                        e.vx = e.face * d.walk;
                        e.ground = false;
                        set(e, ST_JUMP, A_X0 + 6);
                    }
                }
                None => set(e, ST_IDLE, A_IDLE),
            }
        }
        ST_ATTACK => {
            e.vx = 0.0;
            let a = &d.attacks[e.atk.max(0) as usize];
            if e.sf >= attack_len(a) {
                e.atk = -1;
                e.cool = 35 + rnd(55) as i32;
                set(e, ST_IDLE, A_IDLE);
            }
        }
        _ => {}
    }
}

unsafe fn missile(i: usize) {
    let d = &DEFS[E[i].kind as usize];
    let near = nearest(E[i].x, E[i].y);
    let e = &mut E[i];
    match e.st {
        ST_FLY => {
            e.life -= 1;
            // it drifts toward the player's height while it is still coming (Brawl's homing flight); never turns back
            if let Some((_, dx, dy)) = near {
                if dx * e.face > 8.0 {
                    e.vy = fclamp(dy * 0.02, -0.35, 0.35);
                } else {
                    e.vy *= 0.95;
                }
            }
            e.vx = e.face * d.run;
            let (x0, y0) = (e.x, e.y);
            e.x += e.vx;
            e.y += e.vy;
            // in a map room it also blows up on the terrain it flies into (Brawl's Killer explodes on any wall);
            // Final Destination's spawns never meet a wall, so they keep their old path
            let mut hit = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let wall = e.handle >= ROOM_HANDLE_BASE
                && melee_world::tu::mplib::mpCheckAll(&mut hit, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), -1, -1, x0, y0 + 4.0, e.x + e.face * 4.0, e.y + 4.0) != 0;
            if e.hit_mask != 0 || e.life <= 0 || wall {
                e.atk = 1;
                e.hit_mask = 0;
                e.vx = 0.0;
                e.vy = 0.0;
                set(e, ST_BOOM, A_HURT);
            }
        }
        ST_BOOM => {
            if e.sf >= attack_len(&d.attacks[1]) {
                e.dead = true;
                e.kind = -2;
            }
        }
        _ => {}
    }
}

/// A Jyk: drifts toward the nearest player in sight, held within 40 of its spawn point (60 below it), aiming at the player's middle; no gravity, the touch box is
/// always live (attack 0).
unsafe fn floater(i: usize) {
    let d = &DEFS[E[i].kind as usize];
    let near = nearest(E[i].x, E[i].y);
    let e = &mut E[i];
    let (mut tx, mut ty) = (e.home[0], e.home[1]);
    if let Some((_, dx, dy)) = near {
        if fabs(dx) < d.sight[0] && fabs(dy) < d.sight[1] {
            tx = fclamp(e.x + dx, e.home[0] - 40.0, e.home[0] + 40.0);
            ty = fclamp(e.y + dy + 6.0, e.home[1] - 60.0, e.home[1] + 40.0);
        }
    }
    e.vx = fclamp(tx - e.x, -d.walk, d.walk);
    e.vy = fclamp(ty - e.y, -d.walk, d.walk);
    if fabs(e.vx) > 0.01 {
        e.face = sgn1(e.vx);
    }
    e.x += e.vx;
    e.y += e.vy;
    e.atk = 0;
    // a touch hits each player once per pass; the mask clears once nobody is touching any more
    if e.sf >= 60 {
        e.sf = 0;
        e.hit_mask = 0;
    }
}

/// One enemy's frame.
unsafe fn advance(i: usize) {
    let kind = E[i].kind;
    if kind < 0 {
        return;
    }
    let d = &DEFS[kind as usize];
    E[i].flash = (E[i].flash - 1).max(0);
    if E[i].lag > 0 {
        E[i].lag -= 1;
        return;
    }
    if stomped(i) {
        return;
    }
    match E[i].st {
        ST_FALL => {
            let e = &mut E[i];
            e.vx *= 0.98;
            physics(e, d);
            if e.ground {
                e.vx = 0.0;
                set(e, ST_LAND, A_LAND);
            }
        }
        ST_JUMP => {
            let e = &mut E[i];
            physics(e, d);
            if e.ground {
                e.vx = 0.0;
                set(e, ST_LAND, A_LAND);
            }
        }
        ST_LAND => {
            let e = &mut E[i];
            physics(e, d);
            if !e.ground {
                set(e, ST_FALL, A_FALL);
            } else if e.sf >= d.anims[A_LAND].2.min(18) {
                set(e, ST_IDLE, A_IDLE);
            }
        }
        ST_HURT => {
            let e = &mut E[i];
            e.vx *= 0.85;
            physics(e, d);
            if !e.ground {
                set(e, ST_FALL, A_FALL);
            } else if e.sf >= e.stun {
                e.cool = e.cool.max(20);
                set(e, ST_IDLE, A_IDLE);
            }
        }
        ST_LAUNCH => {
            let e = &mut E[i];
            // knockback decays as a fighter's does
            let sp = melee_runtime::gekko::sqrtf(e.vx * e.vx + e.vy * e.vy);
            if sp > 0.051 && d.arch != Arch::Missile {
                let k = (sp - 0.051) / sp;
                e.vx *= k;
            }
            physics(e, d);
            if e.dead && (e.sf >= e.stun.min(45) || e.ground) {
                if d.arch == Arch::Missile && e.handle >= ROOM_HANDLE_BASE {
                    // a beaten map-room Bullet Bill blows up where it comes down
                    e.atk = 1;
                    e.hit_mask = 0;
                    e.vx = 0.0;
                    e.vy = 0.0;
                    set(e, ST_BOOM, A_HURT);
                } else if d.arch == Arch::Missile {
                    e.kind = -2;
                } else {
                    e.vx = 0.0;
                    set(e, ST_DEAD, A_DEAD);
                }
            } else if e.ground {
                e.vx = 0.0;
                set(e, ST_LAND, A_LAND);
            }
        }
        ST_DEAD => {
            let e = &mut E[i];
            e.vx = 0.0;
            if !e.ground {
                physics(e, d);
            }
            if e.sf >= DEAD_FRAMES {
                e.kind = -2;
            }
        }
        _ => match d.arch {
            Arch::Walker => {
                walker(i);
                physics(&mut E[i], d);
                if !E[i].ground && E[i].st != ST_FALL {
                    E[i].atk = -1;
                    set(&mut E[i], ST_FALL, A_FALL);
                }
            }
            Arch::Brawler => {
                brawler(i);
                physics(&mut E[i], d);
                if !E[i].ground && E[i].st != ST_FALL && E[i].st != ST_JUMP {
                    E[i].atk = -1;
                    set(&mut E[i], ST_FALL, A_FALL);
                }
            }
            Arch::Missile => missile(i),
            Arch::Floater => floater(i),
        },
    }
    let e = &mut E[i];
    if e.kind < 0 {
        return;
    }
    // the attack's box
    if e.atk >= 0 && (e.st == ST_ATTACK || e.st == ST_RUN || e.st == ST_FLY || e.st == ST_BOOM) {
        let a = &d.attacks[e.atk as usize];
        let t = e.sf;
        if e.st == ST_ATTACK {
            let (an, af) = attack_pose(a, t);
            e.an = an as i32;
            e.af = af;
        }
        if t >= a.active.0 && t <= a.active.1 {
            attack_players(i, E[i].atk as usize);
        }
    }
    let e = &mut E[i];
    // the clip's frame: loops wrap, the others hold their last frame
    if e.st != ST_ATTACK {
        let (_, _, len, lp) = d.anims[(e.an.max(0) as usize).min(d.anims.len() - 1)];
        e.af += 1;
        if e.af >= len {
            e.af = if lp { 0 } else { len - 1 };
        }
    }
    e.sf += 1;
    if out_of_bounds(e.x, e.y) {
        // knocked out of the arena: beaten (credited to the last attacker by the sim's read)
        if e.hp > 0.0 {
            e.taken += e.hp;
            e.hp = 0.0;
        }
        e.dead = true;
        e.kind = -2;
        if logging() {
            eprintln!("subspace: f{} {} #{i} left the arena", LAST_FRAME, d.name);
        }
    }
}

/// Frames a beaten enemy's last clip shows before it is gone.
const DEAD_FRAMES: i32 = 32;

/// `MELEE_SUBSPACE_TEST=kind[,kind...]`: spawn these (tier `MELEE_SUBSPACE_TIER`) 90 frames into every armed match,
/// without the sim (engine checks and screenshots; the sim does not count them as foes).
fn test_kinds() -> &'static [i32] {
    static V: std::sync::OnceLock<std::vec::Vec<i32>> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        std::env::var("MELEE_SUBSPACE_TEST")
            .map(|s| s.split(',').filter_map(|t| t.trim().parse::<i32>().ok()).filter(|k| (0..KINDS as i32).contains(k)).collect())
            .unwrap_or_default()
    })
}
fn test_tier() -> i32 {
    static V: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_SUBSPACE_TIER").ok().and_then(|s| s.parse().ok()).unwrap_or(0))
}

/// Per logic frame, after the post chain (from `envoy::tick`).
pub unsafe fn sub_tick() {
    let tk = test_kinds();
    if !ON && (tk.is_empty() || TEST_DONE || crate::tu::envoy::armed() == 0) {
        return;
    }
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return;
    }
    let frame = crate::tu_ext::gm_GetFrameCount();
    if frame == LAST_FRAME {
        return; // frozen / paused: the enemies wait too
    }
    LAST_FRAME = frame;
    CLOCK += 1;
    if !tk.is_empty() && !TEST_DONE && CLOCK >= 90 && crate::tu::envoy::SEQ > 0 {
        TEST_DONE = true;
        let n = tk.len().min(MAX);
        for (i, k) in tk.iter().take(n).enumerate() {
            let x = if n == 1 { 35.0 } else { -50.0 + 100.0 * i as f32 / (n - 1) as f32 };
            let missile = DEFS[*k as usize].arch == Arch::Missile;
            let (x, y) = if missile { (if i % 2 == 0 { 150.0 } else { -150.0 }, 12.0 + 6.0 * i as f32) } else { (x, 45.0) };
            sub_spawn(SPAWN_BASE + *k + 32 * test_tier(), x, y, if x > 0.0 { -1 } else { 1 }, 0x7F00 + i as i32 + 1);
        }
    }
    if !ON {
        return;
    }
    player_hits();
    let mut any = false;
    for i in 0..MAX {
        advance(i);
        any |= E[i].kind >= 0;
    }
    if !any && E.iter().all(|e| e.kind == -1) {
        ON = false;
    }
}

// ---------------------------------------------------------------- presentation

static mut DRAWER: usize = 0;
static mut SEEN: [usize; 8] = [0; 8];
static mut SEEN_N: usize = 0;

/// One fighter's pass 0 per render draws the pool (the first fighter drawn twice marks a new render).
unsafe fn is_drawer(g: usize) -> bool {
    if g == DRAWER {
        SEEN_N = 0;
        return true;
    }
    if SEEN[..SEEN_N].contains(&g) || SEEN_N >= SEEN.len() {
        DRAWER = g;
        SEEN_N = 0;
        return true;
    }
    SEEN[SEEN_N] = g;
    SEEN_N += 1;
    false
}

/// The fighter draw-pass hook (`ftdrawcommon::BOSS_DRAW`): the owner's models when loaded, else plain capsules.
pub unsafe fn sub_draw(g: *mut HSD_GObj, pass: i32) {
    if pass != 0 || !ON || g.is_null() || !is_drawer(g as usize) {
        return;
    }
    use crate::tu::petey_model as pm;
    let shapes = {
        static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *V.get_or_init(|| std::env::var_os("MELEE_SUBSPACE_SHAPES").is_some())
    };
    for i in 0..MAX {
        let e = E[i];
        if e.kind < 0 {
            continue;
        }
        let d = &DEFS[e.kind as usize];
        let (r, l, _, _) = d.anims[(e.an.max(0) as usize).min(d.anims.len() - 1)];
        let name = if e.face >= 0.0 { r } else { l };
        // a beaten enemy shrinks away over its last frames
        let fade = if e.st == ST_DEAD && e.sf > DEAD_FRAMES - 10 { fmax((DEAD_FRAMES - e.sf) as f32 / 10.0, 0.05) } else { 1.0 };
        let mut drawn = false;
        if let Some(m) = crate::tu::subspace_model::get(e.kind as usize) {
            if let Ok(mut bufs) = m.bufs.lock() {
                if let Some(b) = bufs.get_mut(i) {
                    let zc = (m.rig.min[2] + m.rig.max[2]) * 0.5;
                    m.rig.skin_with(m.rig.anim(name), e.af as f32, b);
                    let yaw = if d.yaw_face { e.face * core::f32::consts::FRAC_PI_2 } else { 0.0 };
                    pm::draw_tris(&m.rig, &b.tris, [0.0, zc], [e.x, e.y, 0.0], d.scale * fade, yaw, e.flash > 0);
                    drawn = true;
                }
            }
        }
        if drawn && !shapes {
            continue;
        }
        let col = |r: u8, gg: u8, b: u8| GXColor { r, g: gg, b, a: 0xFF };
        let cap = |a: (f32, f32), b: (f32, f32), r: f32, c: GXColor| {
            let dk = col(c.r / 2, c.g / 2, c.b / 2);
            melee_runtime::tu::lbcollision::lbColl_80008FC8(Vec3 { x: a.0, y: a.1, z: 0.0 }, Vec3 { x: b.0, y: b.1, z: 0.0 }, &c, &dk, r);
        };
        let (ha, hb, hr) = hurt_of(&e);
        let body = if e.flash > 0 { col(0xFF, 0xFF, 0xFF) } else { [col(0x9A, 0x6A, 0x3A), col(0x7A, 0x4A, 0x9A), col(0x30, 0x30, 0x38), col(0xA0, 0x60, 0x30)][e.kind as usize & 3] };
        cap(ha, hb, hr * fade, body);
        if e.atk >= 0 {
            let a = &d.attacks[e.atk as usize];
            if e.sf >= a.active.0 && e.sf <= a.active.1 {
                let c = (e.x + a.hit.fx * e.face, e.y + a.hit.y);
                cap(c, c, a.hit.r, col(0xE0, 0x30, 0x30));
            }
        }
    }
}
