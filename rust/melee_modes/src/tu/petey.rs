//! Petey Piranha boss for Envoy fight 5 (notes/petey-boss.md; plan notes/brawl-boss-petey.md).
//!
//! Opt-in: the Envoy plan marks the boss fight (`envoy::P_BOSS`) and `MELEE_ENVOY_BOSS=petey` is set; otherwise the
//! level-9 Ganondorf stand-in plays as before and nothing here runs.
//!
//! Shape of the slice:
//! * the match's one CPU is a *carrier*: invisible, intangible, standing still. Petey's attacks are hitboxes in the
//!   carrier's own capsule slots (the engine's normal hit path: knockback, hitlag, shields); when Petey is beaten the
//!   carrier takes the normal bottom-blast-zone KO, so Envoy's "every CPU out" win check ends the fight.
//! * Petey himself (HP, two cages with their own HP, the attack state machine) is plain statics: the rollback
//!   snapshot covers them, the RNG is our own LCG in the statics, no wall clock, no host input.
//! * players hit Petey with their hitboxes (swept capsule vs body / head / cage); his HP is separate from %.
//! * the cages hold the run's partner (P1: party rescue): tags are refused while caged; breaking either cage frees
//!   both (P2) and wins; draining HP wins too. HP scales with the player count (P3), phase 2 at 50% HP (P4).
//! * looks: original placeholder capsules (body, head, cage bars, the partner) drawn in the carrier's draw pass.
//!   A real model would come only from the owner's own extracted files at runtime (not implemented).
//! Numbers are our own stand-ins, marked [?] in the notes until the owner reads the real ones.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_fighter::tu::player as pl;
use melee_types::*;
use melee_core::ppc::{fctiwz_f32, Ppc};

// console-arithmetic helpers (notes/semantics.md): plain comparisons, explicit conversions
fn fmin(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}
fn fmax(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
fn fclamp(v: f32, lo: f32, hi: f32) -> f32 {
    fmax(fmin(v, hi), lo)
}
/// Round half away from zero (f32::round) through the explicit truncation.
fn round_i(v: f32) -> i32 {
    fctiwz_f32(if v < 0.0 { v - 0.5 } else { v + 0.5 })
}

// --- tunables (stand-ins, [?]) ---
pub const HP_BASE: i32 = 300;
pub const CAGE_HP: i32 = 70;
const HP_SCALE_PER_PLAYER: i32 = 600; // x1000: HP x (1 + 0.6 (players - 1))
const STUN_DMG: f32 = 12.0;
const ARENA: f32 = 55.0; // Petey's x range on FD
const CAGE_DX: f32 = 24.0;
const CAGE_Y: f32 = 22.0;
const CAGE_R: f32 = 9.0;
const BODY_A: f32 = 10.0;
const BODY_B: f32 = 26.0;
const BODY_R: f32 = 11.0;
const HEAD_Y: f32 = 34.0;
const HEAD_R: f32 = 9.0;

#[derive(Clone, Copy)]
struct Atk {
    tele: i32,
    active: i32,
    rec: i32,
    dmg: u32,
    angle: i32,
    kbg: u32,
    bkb: u32,
}
const SLAM: usize = 0;
const SWEEP: usize = 1;
const FLOP: usize = 2;
const SPIN: usize = 3;
const BITE: usize = 4;
const N_ATK: usize = 5;
/// cage slam, cage sweep, belly flop (+ shockwave), spin, bite. Stun is the 6th (reactive).
const ATTACKS: [Atk; N_ATK] = [
    Atk { tele: 40, active: 10, rec: 40, dmg: 16, angle: 80, kbg: 70, bkb: 40 },
    Atk { tele: 30, active: 24, rec: 30, dmg: 12, angle: 45, kbg: 80, bkb: 30 },
    Atk { tele: 30, active: 46, rec: 50, dmg: 14, angle: 70, kbg: 70, bkb: 40 },
    Atk { tele: 25, active: 100, rec: 40, dmg: 6, angle: 60, kbg: 50, bkb: 30 },
    Atk { tele: 30, active: 12, rec: 35, dmg: 18, angle: 40, kbg: 85, bkb: 50 },
];
const STUN_LEN: i32 = 120;

pub const ST_OFF: i32 = 0;
pub const ST_INTRO: i32 = 1;
pub const ST_IDLE: i32 = 2;
pub const ST_TELE: i32 = 3;
pub const ST_ACTIVE: i32 = 4;
pub const ST_REC: i32 = 5;
pub const ST_STUN: i32 = 6;
pub const ST_DOWN: i32 = 7;

#[derive(Clone, Copy)]
pub struct Boss {
    pub st: i32,
    pub atk: i32,
    pub t: i32,
    pub dur: i32,
    pub px: f32,
    pub py: f32,
    pub face: f32,
    pub tx: f32,
    pub hp: i32,
    pub hp_max: i32,
    pub cage: [i32; 2],
    pub cage_max: i32,
    pub phase2: bool,
    pub rng: u32,
    pub last_atk: i32,
    pub stun_cd: i32,
    pub inst: u32,
    pub flash: [i32; 3],
    /// 1 HP drained, 2 cage broken
    pub cause: i32,
    pub ko_done: bool,
    pub carrier_slot: i32,
    pub last_frame: u32,
    pub dmg_hp: [f32; 1],
}

const OFF: Boss = Boss {
    st: ST_OFF, atk: -1, t: 0, dur: 0, px: 0.0, py: 0.0, face: -1.0, tx: 0.0, hp: 0, hp_max: 1, cage: [0; 2], cage_max: 1,
    phase2: false, rng: 1, last_atk: -1, stun_cd: 0, inst: 0, flash: [0; 3], cause: 0, ko_done: false, carrier_slot: -1,
    last_frame: u32::MAX, dmg_hp: [0.0],
};
pub static mut B: Boss = OFF;
/// The match about to start is a Petey fight (set by `envoy::match_begin`, consumed at the first tick).
pub static mut ARMED: bool = false;
/// Players at match start (HP scaling).
static mut PLAYERS: i32 = 1;
/// Per player slot, entity, hitbox: targets already hit by that live hitbox (bit 0 body/head, 1 cage L, 2 cage R).
static mut LATCH: [[[u8; 4]; 2]; 4] = [[[0; 4]; 2]; 4];
/// Our copies of the carrier's capsule slots (put back if the carrier's own state change cleared them) and the
/// attack instance each slot belongs to.
static mut SAVED: [HitCapsule; 4] = unsafe { core::mem::zeroed() };
static mut SLOT_INST: [u32; 4] = [0; 4];

/// Is Petey selected for boss fights (`MELEE_ENVOY_BOSS=petey`; fixed for the process).
pub fn selected() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_ENVOY_BOSS").map(|v| v.eq_ignore_ascii_case("petey")).unwrap_or(false))
}

pub fn active() -> bool {
    unsafe { B.st != ST_OFF || ARMED }
}

/// The partner is in a cage: no tags.
pub fn partner_caged() -> bool {
    unsafe { active() && B.st != ST_DOWN }
}

/// Called by `envoy::match_begin` for the boss fight: `humans` player slots fight the carrier in `carrier_slot`.
pub unsafe fn arm(humans: i32, carrier_slot: i32, seed: u32) {
    B = OFF;
    B.carrier_slot = carrier_slot;
    B.rng = seed | 1;
    PLAYERS = humans.max(1);
    ARMED = true;
    LATCH = [[[0; 4]; 2]; 4];
    SLOT_INST = [0; 4];
    let _ = crate::tu::petey_model::get(); // presentation only: load (or miss) the local model before frame 0
}

/// Not a Petey match: everything off.
pub unsafe fn disarm() {
    B = OFF;
    ARMED = false;
}

fn rnd(n: u32) -> u32 {
    unsafe {
        B.rng = B.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (B.rng >> 16).prem(n.max(1))
    }
}

unsafe fn fighter_of(slot: i32, i: usize) -> *mut Fighter {
    if !(0..4).contains(&slot) {
        return core::ptr::null_mut();
    }
    let p = pl::Player_GetPtrForSlot(slot);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    let g = (*p).player_entity[i];
    if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter }
}

fn scaled(base: i32) -> i32 {
    let n = unsafe { PLAYERS };
    base * (1000 + HP_SCALE_PER_PLAYER * (n - 1)) / 1000
}

fn tele_mul() -> f32 {
    if unsafe { B.phase2 } { 0.65 } else { 1.0 }
}

/// Per logic frame, after the post chain (next to `tag_swap::tick`).
pub unsafe fn tick() {
    if !active() {
        return;
    }
    let carrier = fighter_of(B.carrier_slot, 0);
    if carrier.is_null() {
        return; // not in a match yet (or it ended)
    }
    let frame = crate::tu_ext::gm_GetFrameCount(); // the match logic frame (gm_8016AEEC is the timer seconds)
    if frame == B.last_frame {
        return; // frozen / paused: Petey waits too
    }
    B.last_frame = frame;
    if ARMED {
        ARMED = false;
        B.hp_max = scaled(HP_BASE);
        B.hp = B.hp_max;
        B.cage_max = scaled(CAGE_HP);
        B.cage = [B.cage_max; 2];
        B.px = 30.0;
        B.face = -1.0;
        B.st = ST_INTRO;
        B.t = 0;
        B.dur = 60;
        eprintln!("petey: fight begins, hp {} cages {} players {}", B.hp, B.cage_max, PLAYERS);
        // the carrier is not a fighter on screen: no damage / stock HUD slot
        melee_world::tu::ifstatus::ifStatus_802F6788(B.carrier_slot as u8);
    }
    hide_marker(B.carrier_slot);
    let cf = &mut *carrier;
    if frame % 300 == 0 && std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{frame} carrier pos ({:.2}, {:.2}) petey x {:.1} st {}", cf.cur_pos.x, cf.cur_pos.y, B.px, B.st);
    }
    if !B.ko_done {
        cf.set_invisible(1);
        cf.x1988 = 2; // intangible
    }
    for f in B.flash.iter_mut() {
        *f = (*f - 1).max(0);
    }
    B.stun_cd = (B.stun_cd - 1).max(0);
    if B.st != ST_DOWN {
        player_hits(B.carrier_slot);
        if B.cage[0] <= 0 || B.cage[1] <= 0 {
            down(2);
        } else if B.hp <= 0 {
            down(1);
        } else if !B.phase2 && B.hp * 2 <= B.hp_max {
            B.phase2 = true;
            eprintln!("petey: phase 2");
        }
    }
    step();
    hitboxes(cf);
    if B.st == ST_DOWN && !B.ko_done && B.t >= 60 {
        B.ko_done = true;
        disable_all(cf);
        cf.x1988 = 0;
        melee_fighter::tu::ft_0d31::ftCo_800D3BC8(cf.gobj);
        eprintln!("petey: defeated (cause {}), carrier KO", B.cause);
    }
}

/// Hide the carrier's "CP" pointer: its proc shows it again each frame, this runs after the procs, before the draw.
unsafe fn hide_marker(slot: i32) {
    let g = melee_world::tu::ifnametag::un_804A1EE0[slot as usize];
    if g.is_null() {
        return;
    }
    let j = (*g).hsd_obj as *mut HSD_JObj;
    if !j.is_null() && !(*j).child.is_null() {
        melee_runtime::tu::jobj_world::HSD_JObjSetFlags((*j).child as _, JOBJ_HIDDEN as u32);
    }
}

unsafe fn down(cause: i32) {
    B.st = ST_DOWN;
    B.t = 0;
    B.cause = cause;
    B.atk = -1;
    eprintln!("petey: down, cause {cause} ({})", if cause == 2 { "cage broken: partner freed" } else { "HP drained" });
}

unsafe fn target_x() -> f32 {
    let mut best = B.px;
    let mut bd = f32::MAX;
    for s in 0..4 {
        if s == B.carrier_slot {
            continue;
        }
        let f = fighter_of(s, 0);
        if f.is_null() {
            continue;
        }
        let d = ((*f).cur_pos.x - B.px).abs();
        if d < bd {
            bd = d;
            best = (*f).cur_pos.x;
        }
    }
    best
}

unsafe fn enter(st: i32, dur: i32) {
    B.st = st;
    B.t = 0;
    B.dur = dur.max(1);
}

unsafe fn step() {
    B.t += 1;
    match B.st {
        ST_INTRO => {
            if B.t >= B.dur {
                enter(ST_IDLE, 40);
            }
        }
        ST_IDLE => {
            let tx = target_x();
            B.face = if tx >= B.px { 1.0 } else { -1.0 };
            let want = tx - B.face * 35.0;
            let sp = if B.phase2 { 0.9 } else { 0.6 };
            B.px += fclamp(want - B.px, -sp, sp);
            B.px = fclamp(B.px, -ARENA, ARENA);
            if B.t >= B.dur {
                let mut a = rnd(N_ATK as u32) as i32;
                if a == B.last_atk {
                    a = (a + 1 + rnd(N_ATK as u32 - 1) as i32) % N_ATK as i32;
                }
                B.atk = a;
                B.last_atk = a;
                B.tx = fclamp(tx, -ARENA, ARENA);
                enter(ST_TELE, fctiwz_f32(ATTACKS[a as usize].tele as f32 * tele_mul()));
            }
        }
        ST_TELE => {
            if B.t >= B.dur {
                B.inst = B.inst.wrapping_add(1);
                enter(ST_ACTIVE, ATTACKS[B.atk as usize].active);
            }
        }
        ST_ACTIVE => {
            active_motion();
            if B.t >= B.dur {
                enter(ST_REC, fctiwz_f32(ATTACKS[B.atk as usize].rec as f32 * tele_mul()));
            }
        }
        ST_REC => {
            if B.t >= B.dur {
                B.py = 0.0;
                let chain = B.phase2 && rnd(2) == 0;
                enter(ST_IDLE, if chain { 8 } else if B.phase2 { 20 + rnd(20) as i32 } else { 40 + rnd(30) as i32 });
            }
        }
        ST_STUN => {
            if B.t >= B.dur {
                B.py = 0.0;
                enter(ST_IDLE, 30);
            }
        }
        _ => {}
    }
}

unsafe fn active_motion() {
    let t = B.t;
    match B.atk as usize {
        FLOP => {
            if t <= 20 {
                B.py = 45.0 * (t as f32 / 20.0);
                B.px += (B.tx - B.px) * 0.12;
            } else if t <= 30 {
                B.py = 45.0 * (1.0 - (t - 20) as f32 / 10.0);
            } else {
                B.py = 0.0;
            }
        }
        SPIN => {
            let sp = if B.phase2 { 1.6 } else { 1.2 };
            B.px += B.face * sp;
            if B.px.abs() > ARENA {
                B.px = fclamp(B.px, -ARENA, ARENA);
                B.face = -B.face;
            }
            if t % 12 == 0 {
                B.inst = B.inst.wrapping_add(1); // re-hit window
            }
        }
        BITE => {
            B.px = fclamp(B.px + B.face * 1.5, -ARENA, ARENA);
        }
        _ => {}
    }
}

#[derive(Clone, Copy, Default)]
pub struct Pose {
    pub body: [f32; 4], // ax ay bx by
    pub head: [f32; 2],
    pub cage: [[f32; 2]; 2],
}

/// Where everything is this frame (shared by the hit test, the hitboxes and the draw).
pub fn pose() -> Pose {
    let b = unsafe { B };
    let (x, y) = (b.px, b.py);
    let mut p = Pose {
        body: [x, y + BODY_A, x, y + BODY_B],
        head: [x + b.face * 6.0, y + HEAD_Y],
        cage: [[x - CAGE_DX, y + CAGE_Y], [x + CAGE_DX, y + CAGE_Y]],
    };
    let k = |t: i32, d: i32| fclamp(t as f32 / d.max(1) as f32, 0.0, 1.0);
    match b.st {
        ST_TELE => match b.atk as usize {
            SLAM => {
                for c in &mut p.cage {
                    c[1] += 30.0 * k(b.t, b.dur);
                }
            }
            SWEEP => {
                for (i, c) in p.cage.iter_mut().enumerate() {
                    c[0] = x - b.face * (30.0 + i as f32 * 4.0) * k(b.t, b.dur) + c[0] - x;
                    c[0] = fmax(fmin(c[0], x + 40.0), x - 40.0);
                }
            }
            FLOP => p.body[1] -= 3.0 * k(b.t, b.dur),
            BITE => p.head[0] = x - b.face * 4.0,
            _ => {}
        },
        ST_ACTIVE => match b.atk as usize {
            SLAM => {
                let f = k(b.t, 4);
                for c in &mut p.cage {
                    c[1] = y + CAGE_Y + 30.0 - (30.0 + CAGE_Y - CAGE_R) * f;
                }
            }
            SWEEP => {
                let f = k(b.t, b.dur);
                for (i, c) in p.cage.iter_mut().enumerate() {
                    c[0] = x + b.face * (-30.0 + 70.0 * f - i as f32 * 8.0);
                    c[1] = y + 12.0;
                }
            }
            SPIN => {
                let a = b.t as f32 * 0.25;
                for (i, c) in p.cage.iter_mut().enumerate() {
                    let s = if i == 0 { 1.0 } else { -1.0 };
                    c[0] = x + s * 26.0 * libm_cos(a);
                    c[1] = y + 20.0 + s * 6.0 * libm_sin(a);
                }
            }
            BITE => p.head = [x + b.face * (6.0 + 30.0 * k(b.t, b.dur)), y + 30.0],
            _ => {}
        },
        ST_REC if b.atk as usize == SLAM => {
            for c in &mut p.cage {
                c[1] = y + CAGE_R;
            }
        }
        ST_STUN | ST_DOWN => {
            p.head = [x + b.face * 12.0, y + 22.0];
            if b.st == ST_DOWN {
                let s = fmin(b.t as f32 * 0.5, 30.0);
                p.body[1] -= s;
                p.body[3] -= s;
                p.head[1] -= s;
            }
        }
        _ => {}
    }
    p
}

// deterministic polynomial sin/cos (no libm dependency differences between platforms)
fn libm_sin(a: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let mut x = a - tau * (a / tau).floor();
    if x > core::f32::consts::PI {
        x -= tau;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}
fn libm_cos(a: f32) -> f32 {
    libm_sin(a + core::f32::consts::FRAC_PI_2)
}

/// Active hit spheres: (x, y, radius, damage, angle, kbg, bkb, grounded_only).
fn hit_list() -> ([(f32, f32, f32, u32, i32, u32, u32, bool); 4], usize) {
    let b = unsafe { B };
    let mut v = [(0.0, 0.0, 0.0, 0, 0, 0, 0, false); 4];
    let mut n = 0;
    if b.st != ST_ACTIVE || b.atk < 0 {
        return (v, 0);
    }
    let a = ATTACKS[b.atk as usize];
    let p = pose();
    let mut push = |x: f32, y: f32, r: f32, d: u32, g: bool| {
        if n < 4 {
            v[n] = (x, y, r, d, a.angle, a.kbg, a.bkb, g);
            n += 1;
        }
    };
    match b.atk as usize {
        SLAM => {
            if b.t >= 2 {
                for c in p.cage {
                    push(c[0], c[1], CAGE_R + 1.0, a.dmg, false);
                }
            }
        }
        SWEEP | SPIN => {
            for c in p.cage {
                push(c[0], c[1], CAGE_R + 1.0, a.dmg, false);
            }
            if b.atk as usize == SPIN {
                push(p.body[0], (p.body[1] + p.body[3]) * 0.5, BODY_R + 2.0, a.dmg, false);
            }
        }
        FLOP => {
            if (28..=33).contains(&b.t) {
                push(b.px, b.py + 12.0, 16.0, a.dmg, false);
            }
            if (30..=45).contains(&b.t) {
                let w = 16.0 + 3.0 * (b.t - 30) as f32;
                push(b.px - w, 3.0, 6.0, 9, true);
                push(b.px + w, 3.0, 6.0, 9, true);
            }
        }
        BITE => push(p.head[0], p.head[1], HEAD_R + 1.0, a.dmg, false),
        _ => {}
    }
    (v, n)
}

unsafe fn disable_all(cf: &mut Fighter) {
    for i in 0..4 {
        if SLOT_INST[i] != 0 {
            cf.x914[i].state = HitCapsule_Disabled;
            SLOT_INST[i] = 0;
        }
    }
}

/// Mirror the hit list into the carrier's capsule slots (offsets local to its root joint).
unsafe fn hitboxes(cf: &mut Fighter) {
    let (list, n) = hit_list();
    if n == 0 || B.ko_done {
        disable_all(cf);
        return;
    }
    let root = (*cf.parts).joint;
    if root.is_null() {
        return;
    }
    let inv = melee_runtime::hsd_mtx::inverse(&(*root).mtx);
    for i in 0..4 {
        if i >= n {
            if SLOT_INST[i] != 0 {
                cf.x914[i].state = HitCapsule_Disabled;
                SLOT_INST[i] = 0;
            }
            continue;
        }
        let (x, y, r, d, ang, kbg, bkb, grounded) = list[i];
        let cfp: *mut Fighter = cf;
        let h = &mut *core::ptr::addr_of_mut!((*cfp).x914[i]);
        if SLOT_INST[i] == B.inst && h.state == HitCapsule_Disabled {
            *h = SAVED[i]; // cleared by the carrier's own state change: same attack, same victims
        }
        if SLOT_INST[i] != B.inst || h.state == HitCapsule_Disabled {
            *h = core::mem::zeroed();
            h.x4 = B.inst; // own hit group per instance
            h.state = HitCapsule_Enabled;
            melee_fighter::tu::ftcoll::ftcoll_800768a0(cfp, h);
            h.jobj = root;
            h.damage = d as f32;
            h.unk_count = d;
            h.kb_angle = ang;
            h.x24 = kbg;
            h.x28 = 0;
            h.x2C = bkb;
            h.element = 0;
            h.x34 = 0;
            h.sfx_severity = 1;
            h.set_x40_b2(if grounded { 0 } else { 1 });
            h.set_x40_b3(1);
            h.set_x42_b5(1);
            h.set_x42_b7(1);
            h.set_x42_b1(1);
            (*cfp).set_x2219_b3(1);
            SLOT_INST[i] = B.inst;
        }
        let m = &inv;
        h.b_offset = Vec3 {
            x: m[0][0] * x + m[0][1] * y + m[0][3],
            y: m[1][0] * x + m[1][1] * y + m[1][3],
            z: m[2][0] * x + m[2][1] * y + m[2][3],
        };
        h.scale = r;
        h.set_x43_b1(1);
        SAVED[i] = *h;
    }
    cf.facing_dir = B.face;
}

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

unsafe fn player_hits(carrier: i32) {
    let p = pose();
    for s in 0..4i32 {
        if s == carrier {
            continue;
        }
        for e in 0..2usize {
            let f = fighter_of(s, e);
            if f.is_null() {
                continue;
            }
            let f = &*f;
            let ratio = pl::Player_GetAttackRatio(s);
            for i in 0..4 {
                let h = &f.x914[i];
                let latch = &mut LATCH[s as usize][e][i];
                if h.state == HitCapsule_Disabled {
                    *latch = 0;
                    continue;
                }
                let r = if h.x43_b1() != 0 { h.scale } else { h.scale * f.x34_scale.y };
                let (a, b) = ((h.x58.x, h.x58.y), (h.x4C.x, h.x4C.y));
                let dmg = h.damage * ratio;
                if *latch & 1 == 0 {
                    let body = seg_dist(a, b, (p.body[0], p.body[1]), (p.body[2], p.body[3])) <= r + BODY_R;
                    let head = seg_dist(a, b, (p.head[0], p.head[1]), (p.head[0], p.head[1])) <= r + HEAD_R;
                    if body || head {
                        *latch |= 1;
                        B.hp -= round_i(dmg);
                        B.flash[0] = 8;
                        if head && dmg >= STUN_DMG && B.stun_cd == 0 && B.st != ST_STUN {
                            B.atk = -1;
                            B.stun_cd = 600;
                            enter(ST_STUN, STUN_LEN);
                        }
                    }
                }
                for c in 0..2 {
                    let bit = 2u32.pshl(c as i64) as u8;
                    if *latch & bit == 0 {
                        let q = (p.cage[c][0], p.cage[c][1]);
                        if seg_dist(a, b, q, q) <= r + CAGE_R {
                            *latch |= bit;
                            B.cage[c] -= round_i(dmg);
                            B.flash[1 + c] = 8;
                        }
                    }
                }
            }
        }
    }
}

/// Words for the presentation (`gd_ui_petey`): active, hp, hp max, cage L, cage R, cage max, state, phase2, cause,
/// attack, partner kind (CharacterKind of the run's partner).
pub fn view() -> [i32; 12] {
    let b = unsafe { B };
    [
        (b.st != ST_OFF) as i32, b.hp.max(0), b.hp_max, b.cage[0].max(0), b.cage[1].max(0), b.cage_max, b.st,
        b.phase2 as i32, b.cause, b.atk, unsafe { crate::tu::envoy::ROSTER[1] },
        crate::tu::petey_model::loaded() as i32,
    ]
}

/// Placeholder look: plain capsules in the carrier's draw pass (opaque pass only).
pub unsafe fn draw(g: *mut HSD_GObj, pass: i32) {
    if pass != 0 || B.st == ST_OFF || B.carrier_slot < 0 {
        return;
    }
    let cf = fighter_of(B.carrier_slot, 0);
    if cf.is_null() || (*cf).gobj != g {
        return;
    }
    let p = pose();
    if let Some(m) = crate::tu::petey_model::get() {
        // the owner's own extracted model (presentation only): body fitted to the placeholder's height
        let h = fmax(m.body.max[1] - m.body.min[1], 1.0);
        let sc = (HEAD_Y + HEAD_R) / h;
        let sink = p.body[1] - (B.py + BODY_A);
        let yaw = B.face * 0.35;
        crate::tu::petey_model::draw_mesh(&m.body, [B.px, B.py + sink, 0.0], sc, yaw, B.flash[0] > 0);
        for (i, arm) in [&m.arm_l, &m.arm_r].into_iter().enumerate() {
            if let Some(a) = arm {
                // the arm models are authored around Petey's root: move them rigidly by the cage's offset from rest
                let c = p.cage[i];
                let rest = [B.px + if i == 0 { -CAGE_DX } else { CAGE_DX }, B.py + CAGE_Y];
                let at = [B.px + c[0] - rest[0], B.py + sink + c[1] - rest[1], 0.0];
                crate::tu::petey_model::draw_mesh_rel(a, &m.body, at, sc, yaw, B.flash[1 + i] > 0);
            }
        }
        return;
    }
    let z = 0.0;
    let col = |r: u8, gg: u8, b: u8| GXColor { r, g: gg, b, a: 0xFF };
    let cap = |ax: f32, ay: f32, bx: f32, by: f32, r: f32, c: GXColor| {
        let d = col(c.r / 2, c.g / 2, c.b / 2);
        melee_runtime::tu::lbcollision::lbColl_80008FC8(Vec3 { x: ax, y: ay, z }, Vec3 { x: bx, y: by, z }, &c, &d, r);
    };
    let hit = |i: usize, c: GXColor| if B.flash[i] > 0 { col(0xFF, 0xFF, 0xFF) } else { c };
    let body = hit(0, if B.phase2 { col(0x8A, 0x4A, 0x9A) } else { col(0x4A, 0x7A, 0x9A) });
    cap(p.body[0], p.body[1], p.body[2], p.body[3], BODY_R, body);
    cap(p.head[0], p.head[1], p.head[0], p.head[1], HEAD_R, hit(0, col(0xC8, 0xB0, 0x60)));
    for (i, c) in p.cage.iter().enumerate() {
        let bar = hit(1 + i, col(0xB0, 0xB0, 0xB8));
        let (cx, cy, hw, hh) = (c[0], c[1], 7.0, 8.5);
        for k in 0..4 {
            let x = cx - hw + k as f32 * (2.0 * hw / 3.0);
            cap(x, cy - hh, x, cy + hh, 0.8, bar);
        }
        cap(cx - hw, cy + hh, cx + hw, cy + hh, 1.2, bar);
        cap(cx - hw, cy - hh, cx + hw, cy - hh, 1.2, bar);
        if B.st != ST_DOWN || B.cause != 2 {
            cap(cx, cy - 5.0, cx, cy + 1.0, 2.6, col(0xE0, 0x70, 0x40)); // the caged partner
        }
        if c[0] != p.body[0] {
            cap(c[0], c[1] + hh, p.body[0], p.body[1] + 10.0, 1.6, col(0x50, 0x60, 0x50)); // arm
        }
    }
}
