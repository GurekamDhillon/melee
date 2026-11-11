//! Petey Piranha boss for Envoy fight 5 (notes/petey-boss.md, notes/petey-behavior.md).
//!
//! Opt-in: the Envoy plan marks the boss fight (`envoy::P_BOSS`) and `MELEE_ENVOY_BOSS=petey` is set; otherwise the
//! level-9 Ganondorf stand-in plays as before and nothing here runs.
//!
//! Clean room: the behaviour follows a functional spec written by a separate reader (prose and tables only) plus the
//! data in the owner's files read as formats (petey-behavior.md). Section numbers `§n` below refer to that spec.
//!
//! Shape:
//! * the match's one CPU is a *carrier*: invisible, intangible, standing still. Petey's hitboxes are the carrier's
//!   capsule slots (the engine's normal hit path); when Petey is beaten the carrier takes the bottom-blast-zone KO so
//!   Envoy's win check runs.
//! * Petey is plain statics (rollback snapshot covered), our own LCG, no wall clock, no host input. The statuses,
//!   rules, timings and hitboxes are §4 / §8 / §9; the cages are §6.
//! * Hurt and hit shapes follow the animated bones when the owner's local model is present (`tracks`, built at load
//!   from the files, kept in memory only); without it a stand-in set of shapes is used. The two can give different
//!   sims, so a replay recorded with the model present must be played with it present.
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
/// Round half away from zero through the explicit truncation.
fn round_i(v: f32) -> i32 {
    fctiwz_f32(if v < 0.0 { v - 0.5 } else { v + 0.5 })
}

// ---------------------------------------------------------------- spec numbers

/// Cage HP and max (§5, §6.1).
pub const CAGE_HP: i32 = 999;
/// Death threshold in cage-HP percent (param 3057).
#[allow(dead_code)] // the fallback is petey_file::FALLBACK
const DEATH_PCT: f32 = 10.0;
/// Cage proc thresholds (§6.2).
#[allow(dead_code)] // the fallback is petey_file::FALLBACK
const PHASE1_PCT: f32 = 50.0;
#[allow(dead_code)] // the fallback is petey_file::FALLBACK
const PHASE2_PCT: f32 = 25.0;
// Param 3056 (0.75) is not a speed scale (spec §2 / §7, Q5): it is the fallback bounce factor applied when a running
// movement touches a wall or floor. Petey's dive and landing never bounce and he is never launched, so it is unused;
// the movement kinds run at their raw values (measured dive at L 2.5: 0.2225 first step, +0.0225 / f, max 1.475).
/// The difficulty index the fight uses (0-14): clamp(3 tier + 2 - min(retries, 5)) in Brawl (spec §3); Envoy has no
/// SSE tier or continues, so a fresh Normal game: 5.
const DIFFICULTY: usize = 5;
/// Strength by phase (param 24001 / 24002; phase 0 is the placement's 1).
#[allow(dead_code)] // the fallback is petey_file::FALLBACK
const STRENGTH: [usize; 3] = [1, 2, 3];
/// L table, default mode (§3).
const L_TABLE: [[f32; 15]; 5] = [
    [0.0, 0.3, 0.5, 1.7, 2.1, 2.5, 3.2, 3.6, 4.0, 5.5, 6.0, 6.5, 8.0, 8.5, 9.0],
    [0.6, 0.9, 1.1, 2.5, 2.9, 3.3, 3.8, 4.2, 4.8, 6.1, 6.6, 7.1, 8.5, 9.0, 9.5],
    [1.2, 1.5, 1.7, 3.3, 3.7, 4.1, 4.7, 5.1, 5.5, 6.7, 7.2, 7.7, 9.0, 9.5, 10.0],
    [1.8, 2.1, 2.3, 3.9, 4.3, 4.7, 5.5, 5.9, 6.3, 7.3, 7.8, 8.3, 9.5, 10.0, 10.5],
    [2.4, 2.7, 2.9, 4.5, 5.0, 5.5, 6.2, 6.6, 7.0, 8.0, 8.5, 9.0, 10.0, 10.5, 11.0],
];
/// Petey's x range on FD (FD's edges; the jump goals are +-68, spec §7.2).
const ARENA: f32 = 85.0;
/// The jump's absolute world goal x (params 44006 / 44007).
#[allow(dead_code)] // the fallback is petey_file::FALLBACK
const JUMP_GOAL: f32 = 68.0;

// motions (§1.2): index = the spec's motion number
pub const M_ANGRY: usize = 0;
pub const M_BASKET_L: usize = 1;
pub const M_BASKET_R: usize = 2;
pub const M_DAMAGE_L: usize = 3;
pub const M_DAMAGE_R: usize = 4;
pub const M_DOWN: usize = 5;
pub const M_JUMP_ATTACK: usize = 6;
pub const M_JUMP_LANDING: usize = 7;
pub const M_JUMP_LOOP: usize = 8;
pub const M_JUMP_START: usize = 9;
pub const M_WAIT1: usize = 10;
pub const M_WAIT2: usize = 11;
pub const MOTIONS: [&str; 12] = [
    "Angry", "BasketAttackL", "BasketAttackR", "DamageLeft", "DamageRight", "Down", "JumpAttack", "JumpAttackLanding",
    "JumpLoop", "JumpStart", "Wait1", "Wait2",
];
/// Frame counts (the clips' own lengths; the same for body and both cages).
pub const MOT_LEN: [i32; 12] = [70, 160, 160, 30, 30, 721, 90, 71, 31, 20, 271, 160];

// statuses (§4.3)
pub const ST_OFF: i32 = -1;
pub const ST_DOWN: i32 = 1;
pub const ST_WAIT_A: i32 = 2;
pub const ST_WAIT_B: i32 = 3;
pub const ST_CHOOSE_BASKET: i32 = 4;
pub const ST_MENU: i32 = 5;
pub const ST_ANGRY: i32 = 6;
pub const ST_JUMP_START: i32 = 7;
pub const ST_JUMP_LOOP: i32 = 8;
pub const ST_JUMP_ATTACK: i32 = 9;
pub const ST_JUMP_LANDING: i32 = 10;
pub const ST_BASKET_L: i32 = 11;
pub const ST_BASKET_R: i32 = 12;
pub const ST_DAMAGE_L: i32 = 13;
pub const ST_DAMAGE_R: i32 = 14;
pub const ST_DYING: i32 = 15;

#[derive(Clone, Copy)]
pub struct Boss {
    pub st: i32,
    pub prev_st: i32,
    /// frames since entering the status (0, 1, 2, ...)
    pub sf: i32,
    /// status timeout T
    pub t_out: f32,
    pub mot: i32,
    /// motion frame
    pub mf: i32,
    pub px: f32,
    pub vx: f32,
    pub cage: [i32; 2],
    pub phase: i32,
    /// cage work counter (0 / 1 / 2) and the phase-up rules the cages enabled
    pub cage_ctr: i32,
    pub rule28: bool,
    pub rule29: bool,
    pub jump_count: i32,
    pub jumps_wanted: i32,
    pub side: i32,
    /// which status the flinch returns to (rule 19 -> 4, rule 20 -> 5)
    pub flinch_to: i32,
    /// dive direction (+1 / -1)
    pub aim: f32,
    pub cage_broken: bool,
    /// 0 none, 1 death by the cage threshold, 2 cage broken
    pub cause: i32,
    /// the freed side (0 left cage / Peach, 1 right cage / Zelda)
    pub freed: i32,
    pub ko_done: bool,
    pub down_t: i32,
    pub rng: u32,
    pub carrier_slot: i32,
    pub last_frame: u32,
    /// bumps on every status entry and motion start: hit groups key on it
    pub inst: u32,
    pub flash: [i32; 3],
    /// last player slot targeted (repeat weighting)
    pub last_slot: i32,
    /// the locked target slot (-1 none): rule 13 locks once per jump; landing and the menu clear it
    pub locked: i32,
    /// the last attack-menu pick (`menuPick`, for the 0.3 repeat cut; the file's init leaves it 0)
    pub menu_pick: i32,
    /// absolute world goal x set by rule 13 (spec §7.2); `has_goal` false = unset
    pub has_goal: bool,
    pub goal_x: f32,
    /// Envoy endless loops: each cage's full HP this fight (CAGE_HP x the plan's P_BOSS_HP, set as the fight begins)
    pub cage_max: i32,
    /// the carrier's hitlag last frame (a rise = one of Petey's boxes connected this frame)
    pub lag_prev: f32,
    /// the adventure follow camera (spec §2.3 / §10, Q6): started, centre x / y and eye distance
    pub cam_on: bool,
    pub cam: [f32; 3],
}

const OFF: Boss = Boss {
    st: ST_OFF, prev_st: ST_OFF, sf: 0, t_out: 0.0, mot: M_WAIT1 as i32, mf: 0, px: 0.0, vx: 0.0, cage: [CAGE_HP; 2],
    phase: 0, cage_ctr: 0, rule28: false, rule29: false, jump_count: 1, jumps_wanted: 1, side: 0, flinch_to: ST_MENU,
    aim: 1.0, cage_broken: false, cause: 0, freed: -1, ko_done: false, down_t: 0, rng: 1, carrier_slot: -1,
    last_frame: u32::MAX, inst: 1, flash: [0; 3], last_slot: -1, locked: -1, menu_pick: 0, has_goal: false, goal_x: 0.0, cage_max: CAGE_HP, lag_prev: 0.0,
    cam_on: false, cam: [0.0; 3],
};

/// Each cage's full HP this fight (CAGE_HP with no Envoy loop scaling).
fn cage_max() -> i32 {
    let m = unsafe { B.cage_max };
    if m > 0 { m } else { tun().cage_hp }
}
pub static mut B: Boss = OFF;
/// The match about to start is a Petey fight (set by `envoy::match_begin`, consumed at the first tick).
pub static mut ARMED: bool = false;
/// Per player slot, entity, hitbox: targets already hit by that live hitbox (bit 0 body, 1 cage L, 2 cage R).
static mut LATCH: [[[u8; 4]; 2]; 4] = [[[0; 4]; 2]; 4];
/// Our copies of the carrier's capsule slots (put back if the carrier's own state change cleared them) and the hit
/// group each slot belongs to.
static mut SAVED: [HitCapsule; 4] = unsafe { core::mem::zeroed() };
static mut SLOT_INST: [u32; 4] = [0; 4];
/// Frames left of the knockback log after a hit (MELEE_PETEY_LOG only).
static mut KB_LOG: i32 = 0;
/// Item hitbox latches: (item gobj, per hitbox target bits).
static mut LATCH_IT: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];

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
    unsafe { active() && B.cause == 0 }
}

/// Called by `envoy::match_begin` for the boss fight. The player count does not scale anything (Brawl has no such
/// rule); it is kept in the signature for the caller.
pub unsafe fn arm(_humans: i32, carrier_slot: i32, seed: u32) {
    B = OFF;
    B.carrier_slot = carrier_slot;
    HITS = [0; 4];
    DMG = [0; 4];
    B.rng = seed | 1;
    ARMED = true;
    LATCH = [[[0; 4]; 2]; 4];
    LATCH_IT = [(0, [0; 4]); 16];
    SLOT_INST = [0; 4];
    let _ = tracks(); // load (or miss) the local model before frame 0
    crate::tu::petey_file::load(); // and his parameter file (helper thread, outside the arena)
}

/// Hits each player slot landed on Petey or a cage (Envoy's drives charge on them: his hits never reach the
/// take-damage hook, the carrier being intangible).
pub static mut HITS: [u32; 4] = [0; 4];
/// Envoy V3 (`gd_envoy_boss_dmg`): the damage each player slot dealt Petey or a cage, x1000, a running total that
/// wraps (the same `dmg` the hit test charges the cages with).
pub static mut DMG: [u32; 4] = [0; 4];

/// The carrier's slot while a Petey fight runs (-1 otherwise): Envoy's drives treat it as the boss.
pub fn carrier() -> i32 {
    unsafe { if B.st != ST_OFF { B.carrier_slot } else { -1 } }
}

/// Petey's position for Envoy's drives (the body centre).
pub fn pos() -> (f32, f32) {
    unsafe { (B.px, 18.0) }
}

/// `damage(amount)` (envoy-core-v2.md 2.6): a drive's burst or Burning tick on the boss, whose HP is separate from
/// percent. Body damage, as a player hit: both cages lose it, then the cage and death rules run as on a hit frame.
pub unsafe fn damage(amount: f32) {
    if B.st == ST_OFF || B.cause != 0 || B.ko_done || amount <= 0.0 {
        return;
    }
    let d = round_i(amount).max(1);
    B.cage[0] -= d;
    B.cage[1] -= d;
    B.flash[0] = 8;
    cage_events(true, [false; 2]);
}

/// Not a Petey match: everything off.
pub unsafe fn disarm() {
    B = OFF;
    ARMED = false;
}

fn rnd(n: u32) -> u32 {
    unsafe {
        B.rng = B.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (B.rng >> 8).prem(n.max(1))
    }
}
/// Uniform in [0, 1).
fn rnd01() -> f32 {
    rnd(1 << 16) as f32 / 65536.0
}
/// The decoded boss parameters (petey_file.rs), or the built-in values.
fn tun() -> &'static crate::tu::petey_file::Tuning {
    crate::tu::petey_file::get()
}

/// A weighted list's weights x1000 (base + slope L), with the 0.3 repeat cut on the previous pick `prev`
/// (format spec 4.3: the conditions of 44008 / 44009). Exact to 1/1000.
fn weights<const N: usize>(list: &[(i32, f32); N], prev: Option<i32>) -> [i32; N] {
    let l = level();
    let mut w = [0i32; N];
    for i in 0..N {
        let x = fctiwz_f32((list[i].0 as f32 + list[i].1 * l) * 1000.0);
        w[i] = if prev == Some(i as i32) { x * tun().repeat_pm / 1000 } else { x };
    }
    w
}

/// Weighted pick (§4.7) over integer weights (<= 0 skipped).
fn pick(w: &[i32]) -> usize {
    let sum: i32 = w.iter().map(|&x| x.max(0)).sum();
    if sum <= 0 {
        return 0;
    }
    let r = rnd(sum as u32) as i32;
    let mut acc = 0;
    for (i, &x) in w.iter().enumerate() {
        if x <= 0 {
            continue;
        }
        acc += x;
        if acc > r {
            return i;
        }
    }
    w.len() - 1
}
/// A range parameter (§2.1): uniform in [lo, hi].
fn range(min: f32, min_s: f32, max: f32, max_s: f32) -> f32 {
    let l = level();
    let lo = min + min_s * l;
    let hi = fmax(max + max_s * l, lo);
    lo + (hi - lo) * rnd01()
}

/// The difficulty scalar L for the current phase (§3).
pub fn level() -> f32 {
    let s = tun().strength[unsafe { B.phase }.clamp(0, 2) as usize];
    L_TABLE[s - 1][DIFFICULTY]
}

/// The cages' L: the boss's L truncated to an integer (spec §3, Q4).
pub fn cage_level() -> f32 {
    fctiwz_f32(level()) as f32
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

fn cage_pct(i: usize) -> f32 {
    100.0 * unsafe { B.cage[i] }.max(0) as f32 / cage_max() as f32
}

/// Bumped by every [`tick`] of a live Petey match (presentation: the host plays Brawl's boss music while it moves).
pub static HEARTBEAT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// The Brawl stream sound Petey's fight plays (Brawl's own boss battle theme), by its name in the sound archive.
pub const BOSS_MUSIC: &str = "snd_bgm_X18_BOSSBATTLE";

/// Per logic frame, after the post chain (next to `tag_swap::tick`).
pub unsafe fn tick() {
    if !active() {
        return;
    }
    HEARTBEAT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let carrier = fighter_of(B.carrier_slot, 0);
    if carrier.is_null() {
        return;
    }
    let frame = crate::tu_ext::gm_GetFrameCount();
    if frame == B.last_frame {
        return; // frozen / paused: Petey waits too
    }
    B.last_frame = frame;
    if ARMED {
        ARMED = false;
        B.px = 0.0;
        // Envoy endless loops: the plan's cage HP x1000 (0 / no plan: retail's)
        let hp = crate::tu::envoy::PLAN[crate::tu::envoy::P_BOSS_HP];
        let base = tun().cage_hp;
        B.cage_max = if hp > 0 { (base as i64 * hp.clamp(1000, 20_000) as i64 / 1000) as i32 } else { base };
        // the file's init: jumpCount 0, jumpsWanted 1, phase 0, side 0, menuPick 0 (format spec 3.2, corrects spec 6.1)
        B.jump_count = 0;
        B.jumps_wanted = 1;
        B.cage = [B.cage_max; 2];
        // §6.1 init: side 0, jumpCount 1, phase 0; the fight opens in Wait A
        enter(ST_WAIT_A);
        eprintln!("petey: fight begins, cages {} L {:.1}", B.cage_max, level());
        melee_world::tu::ifstatus::ifStatus_802F6788(B.carrier_slot as u8);
    }
    hide_marker(B.carrier_slot);
    let cf = &mut *carrier;
    if !B.ko_done {
        cf.set_invisible(1);
        cf.x1988 = 2; // intangible
        // the CPUs (a CPU ally, the level-9 test players) aim at the carrier's position: keep it on the floor under
        // Petey's body, not at its parked spawn at the stage's edge (the camera already frames Petey: camera_box).
        // Game state in the arena (rolled back with the fighter); Petey's own targeting skips the carrier.
        if cf.ground_or_air == GA_Ground {
            cf.cur_pos.x = B.px;
        }
    }
    for f in B.flash.iter_mut() {
        *f = (*f - 1).max(0);
    }
    // hits on Petey this frame (body damage forwards to both cages; a cage hit to that cage)
    let (hit_any, cage_hit) = if B.cause == 0 { player_hits(B.carrier_slot) } else { (false, [false; 2]) };
    if B.cause == 0 {
        cage_events(hit_any, cage_hit);
    }
    // hitlag: when one of Petey's boxes connects the carrier (the attacker) gets Melee's hitlag; Petey's motion and
    // movement freeze with it, as in Brawl (measured: basket 9 frozen frames, landing 4). The landing boxes carry a
    // hitlag multiplier of 0.5 (spec §8), applied to the attacker and the victims on the frame the hit lands.
    let lag = cf.dmg.x195c_hitlag_frames;
    if lag > B.lag_prev && std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{} hit lands: motion {} frame {} attacker hitlag {}", B.last_frame, B.mot, B.mf, lag);
    }
    if lag > B.lag_prev && B.mot == M_JUMP_LANDING as i32 {
        let full = lag;
        cf.dmg.x195c_hitlag_frames = fctiwz_f32(full * 0.5) as f32;
        for s in 0..4 {
            if s == B.carrier_slot {
                continue;
            }
            let v = fighter_of(s, 0);
            if !v.is_null() && (*v).dmg.x195c_hitlag_frames == full {
                (*v).dmg.x195c_hitlag_frames = fctiwz_f32(full * 0.5) as f32;
            }
        }
    }
    if std::env::var_os("MELEE_PETEY_LOG").is_some() && (B.lag_prev > 0.0 || KB_LOG > 0) {
        // the victims' first frames of flight (launch angle / speed), for the Brawl comparison
        KB_LOG = if B.lag_prev > 0.0 && cf.dmg.x195c_hitlag_frames <= 0.0 { 4 } else { KB_LOG - 1 };
        if KB_LOG > 0 {
            for s in 0..4 {
                let v = fighter_of(s, 0);
                if s != B.carrier_slot && !v.is_null() {
                    eprintln!("petey: f{} victim {} pos {:.2} {:.2} pct {}", B.last_frame, s, (*v).cur_pos.x, (*v).cur_pos.y, (*v).dmg.x1830_percent);
                }
            }
        }
    }
    B.lag_prev = cf.dmg.x195c_hitlag_frames;
    if cf.dmg.x195c_hitlag_frames <= 0.0 || B.cause != 0 {
        rules();
        advance();
    }
    hitboxes(cf);
    follow_camera();
    if B.cause != 0 {
        B.down_t += 1;
        if !B.ko_done && B.down_t >= 90 {
            B.ko_done = true;
            disable_all(cf);
            cf.x1988 = 0;
            melee_fighter::tu::ft_0d31::ftCo_800D3BC8(cf.gobj);
            eprintln!("petey: defeated (cause {}, freed side {}), carrier KO", B.cause, B.freed);
        }
    }
}

/// Brawl's stage camera for this fight (spec §2.3 / §10, Q6): the shared adventure follow camera. It follows the
/// player only (Petey's range box and clip sphere never widen it), at a fixed eye distance of 260 that it approaches
/// by 5% of the remaining gap per frame, tilted -1 degree (eye slightly above, looking down), and keeps the visible
/// rectangle plus a 10-unit margin inside the camera limit box x -180..180, y -70..120. Brawl's FOV is unknown;
/// Melee's own camera FOV, 30 degrees, gives 2 * 260 * tan 15 = 139.3 visible units of height, the reader's figure.
/// The centre also eases in by the same 5%. Width uses the camera's authored aspect (not the window), so the logic
/// is the same on every machine. Driven through Melee's scripted (boss intro) mode with fixed interest and eye, whose
/// proc then copies our targets as they are. State is in `B` and `game_camera` (statics): rollback-safe.
const CAM_DIST: f32 = 260.0;
const CAM_RATE: f32 = 0.05;
const CAM_TILT_DEG: f32 = -1.0;
const CAM_FOV: f32 = 30.0;
const CAM_LIMIT: [f32; 4] = [-180.0, 180.0, -70.0, 120.0];
const CAM_MARGIN: f32 = 10.0;

unsafe fn follow_camera() {
    use melee_world::tu::camera as cm;
    // the player: the first fighter that is not the carrier
    let mut tgt = None;
    for s in 0..4 {
        if s == B.carrier_slot {
            continue;
        }
        let f = fighter_of(s, 0);
        if !f.is_null() && !(*f).x890_cameraBox.is_null() {
            let p = (*(*f).x890_cameraBox).pos;
            tgt = Some((p.x, p.y));
            break;
        }
    }
    let gc = &mut *core::ptr::addr_of_mut!(cm::game_camera);
    if !B.cam_on {
        let (i, e) = (gc.transform.interest, gc.transform.position);
        let (dx, dy, dz) = (e.x - i.x, e.y - i.y, e.z - i.z);
        let d = melee_runtime::gekko::sqrtf(dx * dx + dy * dy + dz * dz);
        B.cam = [i.x, i.y, if d > 1.0 { d } else { CAM_DIST }];
        B.cam_on = true;
    }
    if gc.mode != CAMERA_BOSS_INTRO as u32 {
        cm::Camera_8002FE38();
    }
    let (tx, ty) = tgt.unwrap_or((B.cam[0], B.cam[1]));
    B.cam[2] += (CAM_DIST - B.cam[2]) * CAM_RATE;
    B.cam[0] += (tx - B.cam[0]) * CAM_RATE;
    B.cam[1] += (ty - B.cam[1]) * CAM_RATE;
    let hh = B.cam[2] * melee_runtime::gekko::tanf(0.5 * CAM_FOV * 0.017453292);
    let hw = hh * cm::cm_803BCB64.aspect;
    let clamp = |v: f32, lo: f32, hi: f32| if lo > hi { 0.5 * (lo + hi) } else { fmin(fmax(v, lo), hi) };
    B.cam[0] = clamp(B.cam[0], CAM_LIMIT[0] + CAM_MARGIN + hw, CAM_LIMIT[1] - CAM_MARGIN - hw);
    B.cam[1] = clamp(B.cam[1], CAM_LIMIT[2] + CAM_MARGIN + hh, CAM_LIMIT[3] - CAM_MARGIN - hh);
    let t = CAM_TILT_DEG * 0.017453292;
    let (ts, tc) = (melee_runtime::gekko::sinf(t), melee_runtime::gekko::cosf(t));
    let interest = Vec3 { x: B.cam[0], y: B.cam[1], z: 0.0 };
    let eye = Vec3 { x: B.cam[0], y: B.cam[1] - B.cam[2] * ts, z: B.cam[2] * tc };
    gc.set_x341_b1_b2(2);
    gc.set_x341_b3_b4(1);
    gc.set_x341_b5_b6(0);
    gc.x344.vec = interest;
    gc.x35C.vec = eye;
    gc.x350 = interest;
    gc.x368.vec = eye;
    gc.x374 = CAM_FOV;
    gc.transform.target_interest = interest;
    gc.transform.interest = interest;
    gc.transform.target_position = eye;
    gc.transform.position = eye;
    gc.transform.target_fov = CAM_FOV;
    gc.transform.fov = CAM_FOV;
    if B.last_frame % 120 == 0 && std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{} camera centre ({:.1}, {:.1}) dist {:.1} mode {} player {:?}", B.last_frame, B.cam[0], B.cam[1], B.cam[2], gc.mode, tgt);
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

/// §6.2 / §6.3 / §5: cage procs, the boss's reaction and the death check, in that order on the hit frame.
unsafe fn cage_events(hit_any: bool, cage_hit: [bool; 2]) {
    // a cage reaching 0: broken (0x13FF), the princess on that side is freed, boss to Down
    for c in 0..2 {
        if B.cage[c] <= 0 && !B.cage_broken {
            B.cage_broken = true;
            B.freed = c as i32;
            eprintln!("petey: cage {c} broken: partner freed");
        }
    }
    // death (rule 5): a frame with a player hit and the lower cage at or under 10%
    if hit_any && !B.cage_broken {
        let (l, r) = (cage_pct(0), cage_pct(1));
        if fmin(l, r) <= tun().death_pct {
            B.freed = if l < r { 0 } else { 1 }; // a tie counts as the right cage (the test is left < right)
            enter(ST_DYING);
            B.cause = 1;
            eprintln!("petey: dying (lower cage {} at {:.1}%)", B.freed, fmin(l, r));
            return;
        }
    }
    // event 1: a damaged cage makes the boss flinch, only from the waits
    for c in 0..2 {
        if cage_hit[c] && (B.st == ST_WAIT_A || B.st == ST_WAIT_B) && !B.cage_broken {
            B.flinch_to = if B.st == ST_WAIT_A { ST_CHOOSE_BASKET } else { ST_MENU };
            enter(if c == 0 { ST_DAMAGE_L } else { ST_DAMAGE_R });
            break;
        }
    }
    // 0x1400 / 0x1401: the phase-up rules
    let low = fmin(cage_pct(0), cage_pct(1));
    if B.cage_ctr == 0 && low <= tun().phase1_pct {
        B.rule28 = true;
    }
    if B.cage_ctr == 1 && low <= tun().phase2_pct {
        B.rule29 = true;
    }
}

/// Rule 10 is on in these statuses.
fn rule10_on(st: i32) -> bool {
    matches!(st, 2 | 3 | 6 | 7 | 8 | 10 | 11 | 12 | 13 | 14)
}

fn motion_end() -> bool {
    unsafe { B.mf + 1 >= MOT_LEN[B.mot as usize] }
}

/// The nearest present player's x (the "locked target" when one is needed), weighted per §4.1.
#[allow(dead_code)]
unsafe fn target_x() -> Option<f32> {
    let s = draw_slot()?;
    Some(slot_x(s))
}

/// The player's x in a slot.
unsafe fn slot_x(s: i32) -> f32 {
    let f = fighter_of(s, 0);
    if f.is_null() { B.px } else { (*f).cur_pos.x }
}

/// The rule engine's target-slot draw (format spec 4.3 step 3): tuning weights, the last used slot cut by the
/// tuning repeat factor (0.3), one draw. Weights x1000.
unsafe fn draw_slot() -> Option<i32> {
    let mut w = [0i32; 4];
    for s in 0..4i32 {
        if s == B.carrier_slot || fighter_of(s, 0).is_null() {
            continue;
        }
        let x = tun().slot_w[s as usize] * 1000;
        w[s as usize] = if B.last_slot == s { x * tun().slot_cut_pm / 1000 } else { x };
    }
    if w.iter().all(|&x| x <= 0) {
        return None;
    }
    let s = pick(&w) as i32;
    B.last_slot = s;
    Some(s)
}

#[allow(dead_code)]
unsafe fn target_x_old() -> Option<f32> {
    const W: [i32; 4] = [25, 10, 20, 8];
    let mut w = [0i32; 4];
    let mut xs = [0.0f32; 4];
    for s in 0..4i32 {
        if s == B.carrier_slot {
            continue;
        }
        let f = fighter_of(s, 0);
        if f.is_null() {
            continue;
        }
        w[s as usize] = W[s as usize];
        xs[s as usize] = (*f).cur_pos.x;
    }
    if w.iter().all(|&x| x == 0) {
        return None;
    }
    let s = pick(&w);
    B.last_slot = s as i32;
    Some(xs[s])
}

/// Side 1 = the target's world x is less than the boss's (screen-left; spec §4.4 rule 13, reader 2).
#[allow(dead_code)]
unsafe fn target_side() -> i32 {
    match target_x() {
        Some(x) if x < B.px => 1,
        Some(_) => 0,
        None => 0,
    }
}

/// One evaluation of the enabled rules (§4.4) by priority.
unsafe fn rules() {
    let st = B.st;
    if st == ST_DYING || st == ST_DOWN {
        return;
    }
    // -100: rule 10 (cage broken -> Down), rule 28 / 29 (phase-up from the waits)
    if B.cage_broken && rule10_on(st) {
        B.cause = 2;
        enter(ST_DOWN);
        return;
    }
    if st == ST_WAIT_A || st == ST_WAIT_B {
        if B.rule28 && B.phase == 0 {
            B.rule28 = false;
            B.phase = 1;
            B.cage_ctr = 1;
            eprintln!("petey: phase 1 (L {:.1})", level());
            enter(ST_ANGRY);
            return;
        }
        if B.rule29 && B.phase == 1 {
            B.rule29 = false;
            B.phase = 2;
            B.cage_ctr = 2;
            eprintln!("petey: phase 2 (L {:.1})", level());
        }
    }
    // 200: rule 13 in JumpStart / JumpLoop: lock the target, set the absolute goal x (-68 on side 1, else +68)
    // Once per jump (format spec 4.4 rule 13): only while no target is locked; landing and the menu clear the lock.
    if (st == ST_JUMP_START || st == ST_JUMP_LOOP) && B.locked < 0 {
        if let Some(s) = draw_slot() {
            B.locked = s;
            B.has_goal = true;
            B.goal_x = if slot_x(s) < B.px { tun().goal[0] } else { tun().goal[1] };
        }
    }
    // 200: the status's own rules
    let to = match st {
        ST_WAIT_A if B.sf as f32 >= B.t_out => Some(ST_CHOOSE_BASKET), // 16
        ST_WAIT_B if B.sf as f32 >= B.t_out => Some(ST_MENU),          // 17
        ST_CHOOSE_BASKET => Some(basket_status()),                      // 21 / 24
        ST_MENU => {
            // 22: (20, 20, 60 + 2L)
            // each entry cut to 0.3 when it repeats the last menu pick (format spec 4.4 rule 22)
            let p = pick(&weights(&tun().menu, Some(B.menu_pick)));
            B.menu_pick = p as i32;
            if p == 2 {
                B.jump_count = 0;
                B.jumps_wanted = if B.phase >= 1 { 2 } else { 1 };
                Some(ST_JUMP_START)
            } else if B.phase >= 1 {
                Some(aimed_basket())
            } else {
                B.side = p as i32;
                Some(if p == 0 { ST_BASKET_L } else { ST_BASKET_R })
            }
        }
        ST_ANGRY if motion_end() => Some(if pick(&weights(&tun().after_angry, None)) == 0 { ST_CHOOSE_BASKET } else { ST_MENU }), // 23
        ST_JUMP_START if motion_end() => Some(ST_JUMP_LOOP),                                              // 18
        ST_JUMP_LOOP if B.sf as f32 >= B.t_out => Some(ST_JUMP_ATTACK),                                   // 25
        ST_JUMP_ATTACK if motion_end() => Some(ST_JUMP_LANDING),                                          // 26
        ST_JUMP_LANDING if motion_end() => {
            // 27
            Some(if B.jump_count >= B.jumps_wanted { ST_WAIT_A } else { ST_JUMP_START })
        }
        ST_BASKET_L | ST_BASKET_R if motion_end() => Some(ST_WAIT_B), // 15
        ST_DAMAGE_L | ST_DAMAGE_R if motion_end() => Some(B.flinch_to), // 19 / 20
        _ => None,
    };
    if let Some(s) = to {
        enter(s); // statuses 4 and 5 resolve on the next frame's evaluation (§4.3)
    }
}

/// Status 4: phase 0 random side (rule 21), else aimed (rule 24).
unsafe fn basket_status() -> i32 {
    if B.phase >= 1 {
        aimed_basket()
    } else {
        B.side = pick(&weights(&tun().side, Some(B.side))) as i32; // 21: with the repeat cut on `side`
        if B.side == 0 { ST_BASKET_L } else { ST_BASKET_R }
    }
}

/// Rule 24: side 1 -> right basket, else left (no target -> left).
unsafe fn aimed_basket() -> i32 {
    // rule 24 locks the target it reads
    match draw_slot() {
        Some(s) => {
            B.locked = s;
            if slot_x(s) < B.px { ST_BASKET_R } else { ST_BASKET_L }
        }
        None => ST_BASKET_L,
    }
}

unsafe fn set_motion(m: usize) {
    B.mot = m as i32;
    B.mf = 0;
    B.inst = B.inst.wrapping_add(1);
}

unsafe fn pick_wait() {
    set_motion(if pick(&weights(&tun().wait_pick, None)) == 0 { M_WAIT1 } else { M_WAIT2 });
}

/// Enter a status (§4.3).
unsafe fn enter(st: i32) {
    if std::env::var_os("MELEE_PETEY_LOG").is_some() {
        eprintln!("petey: f{} status {} -> {} px {:.1} cages {} {}", B.last_frame, B.st, st, B.px, B.cage[0], B.cage[1]);
    }
    B.prev_st = B.st;
    B.st = st;
    B.sf = 0;
    B.inst = B.inst.wrapping_add(1);
    let l = level();
    match st {
        ST_DOWN => {
            set_motion(M_DOWN);
            B.vx = 0.0;
        }
        ST_WAIT_A => {
            pick_wait();
            let [a, b, c, d] = tun().wait_a;
            B.t_out = range(a, b, c, d);
        }
        ST_WAIT_B => {
            pick_wait();
            let [a, b, c, d] = tun().wait_b;
            B.t_out = range(a, b, c, d);
        }
        ST_ANGRY => set_motion(M_ANGRY),
        ST_JUMP_START => {
            set_motion(M_JUMP_START);
            B.t_out = tun().hover[0] + tun().hover[1] * l;
            B.jump_count += 1;
        }
        ST_JUMP_LOOP => set_motion(M_JUMP_LOOP),
        ST_JUMP_ATTACK => {
            set_motion(M_JUMP_ATTACK);
            // §7.2: the angle to the locked target, snapped to horizontal; the boss does not turn
            // the locked target (rule 13's), no new draw
            B.aim = if B.locked >= 0 && !fighter_of(B.locked, 0).is_null() && slot_x(B.locked) < B.px { -1.0 } else { 1.0 };
            B.vx = tun().dive[0] * B.aim;
        }
        ST_JUMP_LANDING => {
            B.locked = -1; // status 10: reset target
            set_motion(M_JUMP_LANDING);
        }
        ST_MENU => B.locked = -1, // status 5: reset target
        ST_BASKET_L => {
            B.side = 0;
            set_motion(M_BASKET_L);
        }
        ST_BASKET_R => {
            B.side = 1;
            set_motion(M_BASKET_R);
        }
        ST_DAMAGE_L => set_motion(M_DAMAGE_L),
        ST_DAMAGE_R => set_motion(M_DAMAGE_R),
        ST_DYING => {
            B.vx = 0.0;
        }
        _ => {} // 4, 5 keep the motion
    }
}

/// Frame advance: motion, wait-motion re-pick, movement (§7).
unsafe fn advance() {
    if B.st == ST_OFF {
        return;
    }
    let l = level();
    // movement: kind 16 during the dive, kind 14 / 15 (decay toward 0) otherwise
    if B.st == ST_JUMP_ATTACK {
        let k = tun().dive;
        let acc = k[1] + k[2] * l;
        let max = k[3] + k[4] * l;
        B.vx = fclamp(B.vx + acc * B.aim, -max, max);
    } else {
        let mut dec = tun().decay;
        if B.st == ST_JUMP_LANDING {
            dec += tun().land_decay_l * l;
        }
        B.vx = if B.vx > 0.0 { fmax(B.vx - dec, 0.0) } else { fmin(B.vx + dec, 0.0) };
    }
    B.px = fclamp(B.px + B.vx, -ARENA, ARENA);
    // §7.2: within reach of the goal x, snap to it and hold (every status, after the movement step)
    if B.has_goal && (B.goal_x - B.px).abs() <= 0.1 + B.vx.abs() {
        B.px = B.goal_x;
    }
    if B.px.abs() >= ARENA {
        B.vx = 0.0;
    }
    B.sf += 1;
    let len = MOT_LEN[B.mot as usize];
    if B.mf + 1 >= len {
        match B.st {
            ST_WAIT_A | ST_WAIT_B => pick_wait(),
            ST_JUMP_LOOP => {
                B.mf = 0; // the hover loops until its timeout
                B.inst = B.inst.wrapping_add(1);
            }
            _ => {} // hold the last frame (Down holds: spec §11 item 6)
        }
    } else {
        B.mf += 1;
    }
}

// ---------------------------------------------------------------- shapes

/// Bone-space points per motion frame (model space), in this order. Offsets are §5 / §8.
const P_BODY_A: usize = 0; // UppermouthN + (0,0,10): the body hurtbox (a sphere) and its stretch end
const P_BODY_B: usize = 1;
const P_CAGE_L_A: usize = 2; // left cage LIronNb, (0,0,0) -> (0,-20,0)
const P_CAGE_L_B: usize = 3;
const P_CAGE_R_A: usize = 4; // right cage RIronNb, (0,0,0) -> (0,20,0)
const P_CAGE_R_B: usize = 5;
const P_HIT_L_A: usize = 6; // left cage bone 9 + (0,5.5,0)
const P_HIT_L_B: usize = 7; // left cage bone 10 (princess hip) + (0,-3.8,0)
const P_HIT_R_A: usize = 8;
const P_HIT_R_B: usize = 9;
const P_CONTACT: usize = 10; // body bone 28 + (0,3,8)
const P_LAND_C: usize = 11; // TopN + (0,31,0)
const P_LAND_L: usize = 12; // TopN + (-29,0,0)
const P_LAND_R: usize = 13; // TopN + (29,0,0)
const NP: usize = 14;
const BODY_HURT_R: f32 = 39.0;
const CAGE_HURT_R: f32 = 11.0;

/// Model scale Brawl units -> ours (open question Q1).
pub const MODEL_SCALE: f32 = 1.0;

/// Per motion, per whole frame: the `P_*` points in model space. Built once from the owner's local files (derived
/// numbers, in memory only, never written).
pub struct Tracks {
    pub mots: std::vec::Vec<std::vec::Vec<[[f32; 3]; NP]>>,
    pub zc: f32,
}

pub fn tracks() -> Option<&'static Tracks> {
    static T: std::sync::OnceLock<Option<Tracks>> = std::sync::OnceLock::new();
    // built on a helper thread, as the model is: the first call comes inside a game frame, where allocations land in
    // the game arena that rollback snapshots rewrite (the tables were freed under the sim by the first load: arena
    // misuse, desyncs and a stalled synctest with the owner's files present)
    T.get_or_init(|| std::thread::spawn(build_tracks).join().ok().flatten()).as_ref()
}

fn build_tracks() -> Option<Tracks> {
    {
        let m = crate::tu::petey_model::get()?;
        let (al, ar) = (m.arm_l.as_ref()?, m.arm_r.as_ref()?);
        let b28 = m.body.bone_by_index(28)?;
        let b0 = m.body.bone_by_index(0)?;
        let (l9, l10, r9, r10) = (al.bone_by_index(9)?, al.bone_by_index(10)?, ar.bone_by_index(9)?, ar.bone_by_index(10)?);
        // the param file's hurtbox sets must agree with the spec's (a mismatch means the files are not the expected
        // ones: fall back to the stand-in)
        if let Some(p) = m.params.as_ref() {
            let ok = p.body.first().map(|h| h.flags >> 23 == 28 && h.r == BODY_HURT_R).unwrap_or(false)
                && p.arm_l.first().map(|h| h.flags >> 23 == 9 && h.r == CAGE_HURT_R).unwrap_or(false);
            if !ok {
                eprintln!("petey: param hurtboxes differ from the expected ones; stand-in shapes");
                return None;
            }
        }
        let mut mots = vec![];
        for name in MOTIONS {
            let n = fctiwz_f32(m.body.anim(name)?.frames).max(0) as usize;
            let (ab, aal, aar) = (m.body.anim(name), al.anim(name), ar.anim(name));
            let mut v = std::vec::Vec::with_capacity(n + 1);
            for f in 0..=n {
                let f = f as f32;
                let mut c = [[0.0f32; 3]; NP];
                c[P_BODY_A] = m.body.bone_point(ab, f, b28, [0.0, 0.0, 10.0]);
                c[P_BODY_B] = m.body.bone_point(ab, f, b28, [0.0, 0.0, 20.0]);
                c[P_CAGE_L_A] = al.bone_point(aal, f, l9, [0.0; 3]);
                c[P_CAGE_L_B] = al.bone_point(aal, f, l9, [0.0, -20.0, 0.0]);
                c[P_CAGE_R_A] = ar.bone_point(aar, f, r9, [0.0; 3]);
                c[P_CAGE_R_B] = ar.bone_point(aar, f, r9, [0.0, 20.0, 0.0]);
                c[P_HIT_L_A] = al.bone_point(aal, f, l9, [0.0, 5.5, 0.0]);
                c[P_HIT_L_B] = al.bone_point(aal, f, l10, [0.0, -3.8, 0.0]);
                c[P_HIT_R_A] = ar.bone_point(aar, f, r9, [0.0, 5.5, 0.0]);
                c[P_HIT_R_B] = ar.bone_point(aar, f, r10, [0.0, -3.8, 0.0]);
                c[P_CONTACT] = m.body.bone_point(ab, f, b28, [0.0, 3.0, 8.0]);
                c[P_LAND_C] = m.body.bone_point(ab, f, b0, [0.0, 31.0, 0.0]);
                c[P_LAND_L] = m.body.bone_point(ab, f, b0, [-29.0, 0.0, 0.0]);
                c[P_LAND_R] = m.body.bone_point(ab, f, b0, [29.0, 0.0, 0.0]);
                v.push(c);
            }
            mots.push(v);
        }
        let zc = (m.body.min[2] + m.body.max[2]) * 0.5;
        eprintln!("petey: bone-following hurt / hit shapes from the local data");
        Some(Tracks { mots, zc })
    }
}

/// This frame's points in world x y.
pub fn points() -> [(f32, f32); NP] {
    let b = unsafe { B };
    let mut out = [(0.0, 0.0); NP];
    if let Some(t) = tracks() {
        let v = &t.mots[b.mot.clamp(0, 11) as usize];
        let c = &v[(b.mf.max(0) as usize).min(v.len() - 1)];
        for (k, q) in c.iter().enumerate() {
            out[k] = (b.px + q[0] * MODEL_SCALE, q[1] * MODEL_SCALE);
        }
        return out;
    }
    stand_in_points(&b)
}

/// Without the local model: fixed shapes of the same sizes, the swinging cage moved across during its active
/// frames and the body raised in the jump, so the fight plays the same rules.
fn stand_in_points(b: &Boss) -> [(f32, f32); NP] {
    let x = b.px;
    let len = MOT_LEN[b.mot.clamp(0, 11) as usize] as f32;
    let k = b.mf as f32 / fmax(len - 1.0, 1.0);
    // the leap is in JumpAttack's clip (the model's root rises about 110 between its frames 5 and 85); JumpStart /
    // JumpLoop stay on the ground (a crouch held for the timeout)
    let _ = k;
    let lift = match b.mot as usize {
        M_JUMP_ATTACK => 110.0 * libm_sin(fclamp((b.mf as f32 - 5.0) / 80.0, 0.0, 1.0) * core::f32::consts::PI),
        _ => 0.0,
    };
    let mut cage = [(x + 36.0, 30.0 + lift), (x - 36.0, 30.0 + lift)];
    let swing = |m: usize, c: &mut (f32, f32)| {
        let f = fclamp((b.mf as f32 - 60.0) / 60.0, 0.0, 1.0);
        let dir = if m == M_BASKET_L { -1.0 } else { 1.0 };
        c.0 = x - dir * 36.0 + dir * 80.0 * f;
        c.1 = 30.0 - 22.0 * libm_sin(f * core::f32::consts::PI);
    };
    match b.mot as usize {
        M_BASKET_L => swing(M_BASKET_L, &mut cage[0]),
        M_BASKET_R => swing(M_BASKET_R, &mut cage[1]),
        _ => {}
    }
    let body = (x, 70.0 + lift);
    let mut o = [(0.0, 0.0); NP];
    o[P_BODY_A] = body;
    o[P_BODY_B] = (body.0, body.1);
    o[P_CAGE_L_A] = (cage[0].0, cage[0].1 + 10.0);
    o[P_CAGE_L_B] = (cage[0].0, cage[0].1 - 10.0);
    o[P_CAGE_R_A] = (cage[1].0, cage[1].1 + 10.0);
    o[P_CAGE_R_B] = (cage[1].0, cage[1].1 - 10.0);
    o[P_HIT_L_A] = (cage[0].0, cage[0].1 + 5.5);
    o[P_HIT_L_B] = (cage[0].0, cage[0].1 - 3.8);
    o[P_HIT_R_A] = (cage[1].0, cage[1].1 + 5.5);
    o[P_HIT_R_B] = (cage[1].0, cage[1].1 - 3.8);
    o[P_CONTACT] = (body.0, body.1 + 3.0);
    o[P_LAND_C] = (x, 31.0);
    o[P_LAND_L] = (x - 29.0, 0.0);
    o[P_LAND_R] = (x + 29.0, 0.0);
    o
}

/// Hurt capsules this frame: (ax, ay, bx, by, r) for body, cage L, cage R; `None` when intangible.
fn hurt() -> [Option<[f32; 5]>; 3] {
    let b = unsafe { B };
    let p = points();
    let cap = |a: usize, c: usize, r: f32| Some([p[a].0, p[a].1, p[c].0, p[c].1, r * MODEL_SCALE]);
    let body = if b.st == ST_DOWN || b.st == ST_DYING { None } else { cap(P_BODY_A, P_BODY_B, BODY_HURT_R) };
    let cages = !b.cage_broken && b.st != ST_DYING;
    [
        body,
        if cages && b.cage[0] > 0 { cap(P_CAGE_L_A, P_CAGE_L_B, CAGE_HURT_R) } else { None },
        if cages && b.cage[1] > 0 { cap(P_CAGE_R_A, P_CAGE_R_B, CAGE_HURT_R) } else { None },
    ]
}

// deterministic polynomial sin (no libm differences between platforms)
fn libm_sin(a: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let mut x = a - tau * (a / tau).floor();
    if x > core::f32::consts::PI {
        x -= tau;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}

// ---------------------------------------------------------------- hitboxes (§8)

#[derive(Clone, Copy, Default)]
struct HitDef {
    x: f32,
    y: f32,
    r: f32,
    dmg: u32,
    angle: i32,
    kbg: u32,
    wkb: u32,
    bkb: u32,
    /// hit group (a new one re-arms the box: the rehit interval)
    grp: u32,
    /// knockback facing
    face: f32,
    /// the attack the box belongs to (0 body contact, 1 basket, 2 landing): one attack's boxes share their victims,
    /// so a victim is hit once per attack (Brawl, measured: one basket / landing hit, never two)
    atk: u8,
}

/// The boxes live this frame.
fn hit_list() -> ([HitDef; 4], usize) {
    let b = unsafe { B };
    let mut v = [HitDef::default(); 4];
    let mut n = 0;
    if b.cause != 0 || b.st == ST_OFF {
        return (v, 0);
    }
    let p = points();
    let l = level();
    let f = b.mf;
    let mut push = |h: HitDef| {
        if n < 4 {
            v[n] = h;
            n += 1;
        }
    };
    let grp = |id: u32, rehit: i32| b.inst.wrapping_mul(16).wrapping_add(id).wrapping_add(f.pdiv(rehit.max(1)) as u32 * 0x1000_0000);
    let mot = b.mot as usize;
    // body contact (id 4): every motion except Down and the landing; in the baskets until frame 68
    let basket = mot == M_BASKET_L || mot == M_BASKET_R;
    let contact = mot != M_DOWN && mot != M_JUMP_LANDING && (!basket || f < 69);
    if contact {
        let (x, y) = p[P_CONTACT];
        push(HitDef {
            x, y, r: 25.7 * MODEL_SCALE, dmg: 0, angle: 0, kbg: 100, wkb: if basket { 5 } else { 6 }, bkb: 0,
            grp: grp(4, 4), face: if x >= b.px { 1.0 } else { -1.0 }, atk: 0,
        });
    }
    // basket cage boxes: frames 70-114 on the swinging cage
    if basket && (70..=114).contains(&f) {
        let (a, c) = if mot == M_BASKET_L { (P_HIT_L_A, P_HIT_L_B) } else { (P_HIT_R_A, P_HIT_R_B) };
        let face = if mot == M_BASKET_L { 1.0 } else { -1.0 }; // the cage's own facing (proc 15: left +1, right -1)
        // the cage's own L: link 5112 hands it the boss's L read as an integer (truncated), spec §3 / §8 (Q4).
        // Normal phases 0/1/2: boss L 2.5/3.3/4.1 -> cage L 2/3/4 -> 15/18/20 damage. Body and landing keep the float L.
        let lc = cage_level();
        let dmg = fctiwz_f32(10.0 + 2.7 * lc).max(0) as u32;
        let bkb = fctiwz_f32(105.0 - 2.0 * lc).max(0) as u32;
        for (i, (pi, r)) in [(a, 8.0f32), (c, 10.0)].into_iter().enumerate() {
            let (x, y) = p[pi];
            push(HitDef { x, y, r: r * MODEL_SCALE, dmg, angle: 145, kbg: 44, wkb: 0, bkb, grp: grp(i as u32, 60), face, atk: 1 });
        }
    }
    // landing: centre frames 2-4, sides 3-4
    if mot == M_JUMP_LANDING {
        if (2..=4).contains(&f) {
            let (x, y) = p[P_LAND_C];
            push(HitDef {
                x, y, r: 28.0 * MODEL_SCALE, dmg: fctiwz_f32(12.0 + 3.0 * l) as u32, angle: 270,
                kbg: fctiwz_f32(45.0 + l) as u32, wkb: 0, bkb: fctiwz_f32(146.0 - 1.5 * l) as u32, grp: grp(0, 120), face: b.aim, atk: 2,
            });
        }
        if (3..=4).contains(&f) {
            for (i, pi) in [P_LAND_L, P_LAND_R].into_iter().enumerate() {
                let (x, y) = p[pi];
                push(HitDef {
                    x, y, r: (30.0 + 0.5 * l) * MODEL_SCALE, dmg: fctiwz_f32(8.0 + 2.0 * l) as u32, angle: 30, kbg: 48, wkb: 0,
                    bkb: fctiwz_f32(112.0 - 1.2 * l) as u32, grp: grp(1 + i as u32, 120), face: if i == 0 { -1.0 } else { 1.0 }, atk: 2,
                });
            }
        }
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
        let d = list[i];
        let cfp: *mut Fighter = cf;
        let h = &mut *core::ptr::addr_of_mut!((*cfp).x914[i]);
        if SLOT_INST[i] == d.grp && h.state == HitCapsule_Disabled {
            *h = SAVED[i]; // cleared by the carrier's own state change: same box, same victims
        }
        if SLOT_INST[i] != d.grp || h.state == HitCapsule_Disabled {
            *h = core::mem::zeroed();
            h.x4 = d.grp;
            h.state = HitCapsule_Enabled;
            melee_fighter::tu::ftcoll::ftcoll_800768a0(cfp, h);
            h.jobj = root;
            h.damage = d.dmg as f32;
            h.unk_count = d.dmg;
            h.kb_angle = d.angle;
            h.x24 = d.kbg;
            h.x28 = d.wkb;
            h.x2C = d.bkb;
            h.element = 0;
            h.x34 = 0;
            h.sfx_severity = 1;
            h.set_x40_b2(1);
            h.set_x40_b3(1);
            h.set_x42_b5(1);
            h.set_x42_b7(1);
            h.set_x42_b1(1);
            (*cfp).set_x2219_b3(1);
            SLOT_INST[i] = d.grp;
        }
        // knockback direction: Melee launches a fighter's victim relative to the attacker's (the invisible carrier's) x,
        // Brawl in the box owner's facing (cage L +1, cage R -1; landing sides outward). Mirror the angle so the launch
        // matches the facing for the player nearest the box (measured: basket L -> -x, basket R -> +x at 145).
        if d.angle < 180 && d.angle != 90 {
            let mut best: Option<f32> = None;
            for s in 0..4 {
                let v = fighter_of(s, 0);
                if s != B.carrier_slot && !v.is_null() {
                    let x = (*v).cur_pos.x;
                    if best.map_or(true, |b| (x - d.x).abs() < (b - d.x).abs()) {
                        best = Some(x);
                    }
                }
            }
            if let Some(vx) = best {
                let away = if vx > (*cfp).cur_pos.x { 1.0f32 } else { -1.0 };
                // launch x sign = away * sign(cos a): keep a when away == facing, else mirror it (180 - a)
                h.kb_angle = if away == d.face { d.angle } else { 180 - d.angle };
            }
        }
        let m = &inv;
        h.b_offset = Vec3 {
            x: m[0][0] * d.x + m[0][1] * d.y + m[0][3],
            y: m[1][0] * d.x + m[1][1] * d.y + m[1][3],
            z: m[2][0] * d.x + m[2][1] * d.y + m[2][3],
        };
        h.scale = d.r;
        h.set_x43_b1(1);
        SAVED[i] = *h;
        cf.facing_dir = d.face;
    }
    // one attack's boxes share their victims
    for i in 0..n {
        for j in 0..n {
            if i == j || list[i].atk == 0 || list[i].atk != list[j].atk {
                continue;
            }
            for k in 0..cf.x914[i].victims_1.len() {
                let v = cf.x914[i].victims_1[k];
                if v.victim.is_null() || cf.x914[j].victims_1.iter().any(|w| w.victim == v.victim) {
                    continue;
                }
                if let Some(e) = cf.x914[j].victims_1.iter_mut().find(|w| w.victim.is_null()) {
                    *e = v;
                }
            }
        }
    }
    for i in 0..n {
        SAVED[i] = cf.x914[i];
    }
}

// ---------------------------------------------------------------- hits on Petey

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

/// One live hitbox against the body and both cages (bits in `latch`, once per hitbox lifetime). Body damage is
/// forwarded to both cages (§5); a cage hit damages that cage. Returns (hit anything, cage bits hit).
unsafe fn hit_test(hs: &[Option<[f32; 5]>; 3], a: (f32, f32), b: (f32, f32), r: f32, dmg: f32, latch: &mut u8) -> (bool, u8) {
    let mut any = false;
    let mut cages = 0u8;
    let d = round_i(dmg);
    for (k, h) in hs.iter().enumerate() {
        let bit = 1u8.pshl(k as i64);
        let Some(c) = h else { continue };
        if *latch & bit != 0 {
            continue;
        }
        if seg_dist(a, b, (c[0], c[1]), (c[2], c[3])) <= r + c[4] {
            *latch |= bit;
            any = true;
            B.flash[k] = 8;
            if k == 0 {
                B.cage[0] -= d;
                B.cage[1] -= d;
            } else {
                B.cage[k - 1] -= d;
                cages |= 1u8.pshl(k as i64 - 1);
            }
        }
    }
    (any, cages)
}

unsafe fn player_hits(carrier: i32) -> (bool, [bool; 2]) {
    let hs = hurt();
    let mut any = false;
    let mut cages = 0u8;
    for s in 0..4i32 {
        if s == carrier {
            continue;
        }
        for e in 0..2usize {
            let f = fighter_of(s, e);
            if f.is_null() {
                continue;
            }
            let ratio = pl::Player_GetAttackRatio(s);
            for i in 0..4 {
                let h: *mut HitCapsule = core::ptr::addr_of_mut!((*f).x914[i]);
                let latch = &mut LATCH[s as usize][e][i];
                if (*h).state == HitCapsule_Disabled {
                    *latch = 0;
                    continue;
                }
                let r = if (*h).x43_b1() != 0 { (*h).scale } else { (*h).scale * (*f).x34_scale.y };
                let (a, b) = (((*h).x58.x, (*h).x58.y), ((*h).x4C.x, (*h).x4C.y));
                let dmg = (*h).damage * ratio;
                let (hit, cg) = hit_test(&hs, a, b, r, dmg, latch);
                if hit {
                    any = true;
                    cages |= cg;
                    HITS[s as usize] = HITS[s as usize].wrapping_add(1);
                    DMG[s as usize] = DMG[s as usize].wrapping_add(fctiwz_f32(dmg * 1000.0).max(0) as u32);
                    // the attacker's side of a landed hit, as for hitting an item: hitlag next frame, the hit sound
                    let fp = &mut *f;
                    let d = round_i(dmg).max(1);
                    if (fp.dmg.x1914 as i32) < d {
                        fp.dmg.x1914 = d as _;
                    }
                    let slot = 0x7E + fp.player_idx as i32 * 2 + fp.is_sub_fighter() as i32;
                    melee_runtime::tu::lbcollision::lbColl_80005BB0(h, slot);
                }
            }
        }
    }
    let (ia, ic) = item_hits(&hs, carrier);
    any |= ia;
    cages |= ic;
    (any, [cages & 1 != 0, cages & 2 != 0])
}

/// Items and projectiles hit Petey and the cages. Latched per item and hitbox until that hitbox goes off.
unsafe fn item_hits(hs: &[Option<[f32; 5]>; 3], carrier: i32) -> (bool, u8) {
    let mut next: [(usize, [u8; 4]); 16] = [(0, [0; 4]); 16];
    let mut n = 0;
    let mut any = false;
    let mut cages = 0u8;
    let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    while !g.is_null() {
        let ip = (*g).user_data as *mut Item;
        g = (*g).next;
        if ip.is_null() {
            continue;
        }
        let it = &mut *ip;
        let owner = it.owner;
        if !owner.is_null() {
            let of = (*owner).user_data as *mut Fighter;
            if (*owner).classifier == HSD_GOBJ_CLASS_FIGHTER as _ && !of.is_null() && (*of).player_idx as i32 == carrier {
                continue;
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
            let (hit, cg) = hit_test(hs, a, b, (*h).scale, (*h).damage, &mut lat[i]);
            if hit {
                any = true;
                cages |= cg;
                melee_runtime::tu::lbcollision::lbColl_80005BB0(h, 0x7E);
            }
        }
        if live && n < next.len() {
            next[n] = (key, lat);
            n += 1;
        }
    }
    LATCH_IT = next;
    (any, cages)
}

// ---------------------------------------------------------------- presentation

/// Words for `gd_ui_petey`: active, hp (cage HP left above the death line), hp max, cage at screen left, cage at
/// screen right, cage max, state (7 = beaten), phase >= 1, cause, status, partner kind, real model drawn.
pub fn view() -> [i32; 12] {
    let b = unsafe { B };
    let line = fctiwz_f32(cage_max() as f32 * tun().death_pct / 100.0);
    let low = b.cage[0].min(b.cage[1]).max(0);
    [
        (b.st != ST_OFF) as i32,
        (low - line).max(0),
        cage_max() - line,
        b.cage[1].max(0), // the right cage hangs at screen left (model -x)
        b.cage[0].max(0),
        cage_max(),
        if b.cause != 0 { 7 } else { 0 },
        (b.phase >= 1) as i32,
        b.cause,
        b.st,
        unsafe { crate::tu::envoy::ROSTER[1] },
        crate::tu::petey_model::loaded() as i32,
    ]
}

/// Draw: the owner's model when present, else plain capsules, in the carrier's draw pass (opaque pass only).
pub unsafe fn draw(g: *mut HSD_GObj, pass: i32) {
    if pass != 0 || B.st == ST_OFF || B.carrier_slot < 0 {
        return;
    }
    let cf = fighter_of(B.carrier_slot, 0);
    if cf.is_null() || (*cf).gobj != g {
        return;
    }
    if let Some(m) = crate::tu::petey_model::get() {
        draw_model(m);
        if std::env::var_os("MELEE_PETEY_SHAPES").is_none() {
            return;
        }
    }
    let z = 0.0;
    let col = |r: u8, gg: u8, b: u8| GXColor { r, g: gg, b, a: 0xFF };
    let cap = |a: [f32; 5], c: GXColor| {
        let d = col(c.r / 2, c.g / 2, c.b / 2);
        melee_runtime::tu::lbcollision::lbColl_80008FC8(Vec3 { x: a[0], y: a[1], z }, Vec3 { x: a[2], y: a[3], z }, &c, &d, a[4]);
    };
    let hit = |i: usize, c: GXColor| if B.flash[i] > 0 { col(0xFF, 0xFF, 0xFF) } else { c };
    let hs = hurt();
    let colors = [
        if B.phase >= 1 { col(0x8A, 0x4A, 0x9A) } else { col(0x4A, 0x7A, 0x9A) },
        col(0xE0, 0x90, 0xB0),
        col(0xB0, 0xB0, 0xE0),
    ];
    for (k, h) in hs.iter().enumerate() {
        if let Some(c) = h {
            cap(*c, hit(k, colors[k]));
        }
    }
    let (list, n) = hit_list();
    for d in list.iter().take(n) {
        if d.dmg > 0 {
            cap([d.x, d.y, d.x, d.y, d.r], col(0xE0, 0x30, 0x30));
        }
    }
}

/// The owner's model, posed by the sim's motion and motion frame (presentation only; a pure function of the
/// statics, so a replay or rollback redraws the same picture).
unsafe fn draw_model(m: &'static crate::tu::petey_model::Model) {
    use crate::tu::petey_model as pm;
    // one buffer set per part, kept until the next draw (the GX bridge may read the vertex data after the call
    // returns), made once on a helper thread: a draw inside the game scope must not allocate, or rollback rewrites
    // the arena under the buffers (write-after-free, desyncs, a stalled synctest with the owner's files present)
    static BUFS: std::sync::OnceLock<std::sync::Mutex<std::vec::Vec<pm::DrawBufs>>> = std::sync::OnceLock::new();
    let bufs = BUFS.get_or_init(|| {
        let v = std::thread::spawn(move || {
            let mut v = std::vec::Vec::with_capacity(3);
            v.push(m.body.draw_bufs());
            for r in [&m.arm_l, &m.arm_r] {
                if let Some(r) = r {
                    v.push(r.draw_bufs());
                }
            }
            std::sync::Mutex::new(v)
        });
        v.join().unwrap_or_else(|_| std::sync::Mutex::new(std::vec::Vec::new()))
    });
    let Ok(mut bufs) = bufs.lock() else { return };
    if bufs.is_empty() {
        return;
    }
    let name = MOTIONS[B.mot.clamp(0, 11) as usize];
    let fr = B.mf as f32;
    let zc = (m.body.min[2] + m.body.max[2]) * 0.5;
    // the clip's own root carries the jump height; y 0 is the stage
    let org = [0.0, zc];
    let at = [B.px, 0.0, 0.0];
    let a = m.body.anim(name);
    m.body.skin_with(a, fr, &mut bufs[0]);
    pm::draw_tris(&m.body, &bufs[0].tris, org, at, MODEL_SCALE, 0.0, B.flash[0] > 0);
    let mut k = 1;
    for (i, arm) in [&m.arm_l, &m.arm_r].into_iter().enumerate() {
        let Some(r) = arm else { continue };
        let slot = k;
        k += 1;
        if B.cage_broken && B.freed == i as i32 {
            continue; // the broken cage hides its model (§6.2 proc 17)
        }
        if let Some(b) = bufs.get_mut(slot) {
            r.skin_with(r.anim(name), fr, b);
            pm::draw_tris(r, &b.tris, org, at, MODEL_SCALE, 0.0, B.flash[1 + i] > 0);
        }
    }
}

#[cfg(test)]
mod probe {
    use super::*;
    /// Local-only (needs the owner's files): derived numbers for notes/petey-behavior.md.
    #[test]
    #[ignore]
    fn petey_tracks_probe() {
        let t = tracks().expect("local data");
        for (mi, v) in t.mots.iter().enumerate() {
            let ys: std::vec::Vec<f32> = v.iter().map(|c| c[P_BODY_A][1]).collect();
            let (lo, hi) = ys.iter().fold((f32::MAX, f32::MIN), |(a, b), &y| (a.min(y), b.max(y)));
            eprintln!("{:18} body-hurt y {:6.1}..{:6.1}  x {:6.1}..{:6.1}", MOTIONS[mi], lo, hi,
                v.iter().map(|c| c[P_BODY_A][0]).fold(f32::MAX, f32::min), v.iter().map(|c| c[P_BODY_A][0]).fold(f32::MIN, f32::max));
        }
        for (m, pa) in [(M_BASKET_L, P_HIT_L_A), (M_BASKET_R, P_HIT_R_A)] {
            for f in [0usize, 40, 69, 70, 80, 92, 104, 114, 130, 159] {
                let c = t.mots[m][f];
                eprintln!("{} f{f}: cage box ({:.1}, {:.1}) hurt L ({:.1},{:.1}) R ({:.1},{:.1})", MOTIONS[m], c[pa][0], c[pa][1],
                    c[P_CAGE_L_A][0], c[P_CAGE_L_A][1], c[P_CAGE_R_A][0], c[P_CAGE_R_A][1]);
            }
        }
        for m in [M_JUMP_START, M_JUMP_LOOP, M_JUMP_ATTACK, M_JUMP_LANDING] {
            let v = &t.mots[m];
            let s: std::vec::Vec<std::string::String> = (0..v.len()).step_by(10).map(|f| format!("{:.0}", v[f][P_CONTACT][1])).collect();
            eprintln!("{} contact y every 10f: {}", MOTIONS[m], s.join(" "));
        }
        let d = &t.mots[M_DOWN];
        eprintln!("Down body y f0 {:.1} f150 {:.1} f400 {:.1} f720 {:.1}", d[0][P_BODY_A][1], d[150][P_BODY_A][1], d[400][P_BODY_A][1], d[720][P_BODY_A][1]);
        let c = &t.mots[M_JUMP_LANDING][3];
        eprintln!("landing f3 centre ({:.1},{:.1}) L ({:.1},{:.1}) R ({:.1},{:.1})", c[P_LAND_C][0], c[P_LAND_C][1], c[P_LAND_L][0], c[P_LAND_L][1], c[P_LAND_R][0], c[P_LAND_R][1]);
    }
}
