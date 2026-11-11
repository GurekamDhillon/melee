//! envoy_sim: an Envoy run (docs/rust-port/notes/envoy-1p.md, envoy-drives.md). The sim half: run state, the plan
//! of the next fight, tags, mid-fight stage switches, drives and keystones (envoy-core-v2.md, milestone V2), local
//! co-op and the run's end. Everything is this module's linear memory (rolled back with the game) or engine statics
//! (melee_modes::tu::{envoy, tag_swap, petey}); the presentation (envoy_ui) only reads the published view.
//!
//! * run: the Short run, 5 fights: a duel, an elite (act boss 1), a pack, an elite (act boss 2), the final boss (a
//!   level-9 Ganondorf, or Petey with MELEE_ENVOY_BOSS=petey). Each player's pair (main + partner) shares one stock
//!   pool: 4 solo, 5 in co-op (MELEE_ENVOY_COOP=1: a second human in slot 1 with its own pair).
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown (Lock: 10 s), 60-frame frozen cinematic.
//! * stage switches: from fight 2 on, every 30 s of play, the next of BF / YS / DL / FoD / FD.
//! * drives (envoy_data, 20): each fighter wears up to 3, 4 after the first boss (`[2 players][2 fighters][4]` of id
//!   and level 1-3; a second copy levels it up). The fighter in play charges its drives from hits landed (aerial
//!   hits, hits on a foe at 100%+), hits taken, landings, jumps, time, tag-in, KOs, stage shifts and other fires; a
//!   full drive fires on the next free frame, at most one fire per set every 20 frames (Relay's volley: 6 frames
//!   apart). Nova, Comet and Spire launch through `gd_envoy_blast` (a hitbox on the engine's hit path); every other
//!   effect is percent, a status (Burning, Chilled, Marked, Haste), an orb, a heal or an air jump. Fires go to the
//!   12-entry fire log the presentation draws and plays from. Drive effects only ever reach the other side.
//! * keystones (6): one each player at the run's start and after each of the first two bosses, each with its price.
//! * foes: elites and bosses carry 1-2 drives that charge and fire at the players (Chilled and Marked act at half
//!   strength on a boss).
//! * earning: KOs drop drives on floor points, one per player in the owner's colour (an elite: two); fights 2-5
//!   open with a pick of three drives for each player, then the fighter that wears it.
//! * MELEE_ENVOY_SHARED_DRIVES=1: one set shared by each pair (Q1's alternative), fixed per run.
//! * determinism: the whole state is one `#[repr(C)]` struct of i32 words; every frame an FNV-1a over all of it, the
//!   plan and the pool digest goes into `gd_hash_word`, so a divergence in any charge shows on the frame it happens.
#![no_std]
#![allow(static_mut_refs)]

use envoy_data::view::*;
use envoy_data::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_fighter_from_slot(slot: i32) -> u32;
    fn gd_fighter_slot(h: u32) -> i32;
    fn gd_fighter_get_input(h: u32, out: *mut u32);
    fn gd_fighter_kind(h: u32) -> i32;
    fn gd_fighter_get_pos_x(h: u32) -> f32;
    fn gd_fighter_get_pos_y(h: u32) -> f32;
    fn gd_fighter_get_percent(h: u32) -> f32;
    fn gd_fighter_get_action_state(h: u32) -> i32;
    fn gd_player_present(slot: i32) -> i32;
    fn gd_player_stocks(slot: i32) -> i32;
    fn gd_tag_partner(slot: i32) -> i32;
    fn gd_tag_begin_ex(slot: i32, len: i32, flags: i32) -> i32;
    fn gd_tag_swap(skip: i32) -> i32;
    fn gd_transition_phase() -> i32;
    fn gd_transition_kind() -> i32;
    fn gd_transition_frame() -> i32;
    fn gd_transition_slot() -> i32;
    fn gd_stage_grkind() -> i32;
    fn gd_stage_transition(grkind: i32, len: i32) -> i32;
    fn gd_hold_begin(len: i32) -> i32;
    fn gd_hold_release() -> i32;
    fn gd_pad_pressed(port: i32) -> u32;
    fn gd_envoy_plan(ptr: *const i32, n: u32) -> i32;
    fn gd_envoy_publish(ptr: *const i32, n: u32) -> i32;
    fn gd_envoy_seq() -> i32;
    fn gd_envoy_start_fight() -> i32;
    fn gd_envoy_roster(i: i32) -> i32;
    fn gd_envoy_apply(slot: i32, which: i32, v: i32) -> i32;
    fn gd_envoy_percent(slot: i32, bench: i32, delta_milli: i32) -> i32;
    fn gd_envoy_test_drives(out: *mut u8, cap: u32) -> i32;
    fn gd_envoy_shared_drives() -> i32;
    fn gd_envoy_coop() -> i32;
    fn gd_envoy_speed(slot: i32, active: i32, bench: i32) -> i32;
    fn gd_envoy_absorb(slot: i32) -> i32;
    fn gd_envoy_attacker(slot: i32) -> i32;
    fn gd_envoy_jumps(slot: i32) -> i32;
    fn gd_envoy_air_jump(slot: i32) -> i32;
    fn gd_envoy_blast(slot: i32, x: f32, y: f32, r: f32, dmg: i32, angle: i32, kbg: i32, bkb: i32) -> i32;
    fn gd_envoy_boss(out: *mut u32) -> i32;
    fn gd_envoy_armed() -> i32;
    fn gd_envoy_record(out: *mut i32, cap: u32) -> i32;
    fn gd_hash_word(w: u32);
    fn gd_log(msg: *const u8, value: i32);
}

const FIGHTS: usize = 5;
const START_POOL: [i32; 2] = [4, 5];
const TAG_LEN: i32 = 60;
const TAG_COOLDOWN: i32 = 300;
const TAG_SKIP_AFTER: i32 = 6;
const STAGE_LEN: i32 = 90;
const STAGE_PERIOD: i32 = 1800;
const REWARD_TIME: i32 = 1200;
/// Frames into a fight before its reward screen opens (after the fighters are placed).
const REWARD_DELAY: i32 = 2;
/// After the last KO, the fight waits up to this long while a drop is still on the floor (the collect hold), then
/// the drops are swept up.
const COLLECT_HOLD: i32 = 600;

const DPAD_LEFT: u32 = 0x0001;
const DPAD_RIGHT: u32 = 0x0002;
const DPAD_DOWN: u32 = 0x0004;
const BTN_A: u32 = 0x0100;
const BTN_B: u32 = 0x0200;
const STICK_LEFT: u32 = 0x0004_0000;
const STICK_RIGHT: u32 = 0x0008_0000;

const KIND_TAG: i32 = 1;
const KIND_HOLD: i32 = 3;

// gd_envoy_apply `which`
const AP_ATTACK: i32 = 0;
const AP_DEFENSE: i32 = 1;
const AP_STOCKS: i32 = 2;

/// Gr_Kind: Battlefield, Yoshi's Story, Dream Land, Fountain of Dreams, Final Destination.
const STAGES: [i32; 5] = [36, 10, 28, 12, 37];
/// CPU characters (CharacterKind, the CSS ids): the retail cast minus Ice Climbers / Zelda / Sheik / Ganondorf (boss).
const CPU_POOL: [i32; 22] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 20, 21, 22, 23, 24];
const CK_GANONDORF: i32 = 0x19;

/// Action states: JumpF, JumpB, JumpAerialF, JumpAerialB (a jump); AttackAirN..AttackAirLw (an aerial).
const AS_JUMP: (i32, i32) = (25, 28);
const AS_AERIAL: (i32, i32) = (65, 69);
/// Landing, LandingFallSpecial, LandingAirN..LandingAirLw (a landing; polled: the aerial landings have no hook row).
const AS_LANDING: [(i32, i32); 2] = [(42, 43), (70, 74)];

struct Fight {
    kind: i32,
    ncpu: i32,
    level: [i32; 3],
    stocks: [i32; 3],
    /// Drives the foe carries (elites and bosses) and their level.
    foe_drives: i32,
    foe_level: i32,
    scale: i32,
}
const SCHEDULE: [Fight; FIGHTS] = [
    Fight { kind: FK_DUEL, ncpu: 1, level: [3, 0, 0], stocks: [1, 0, 0], foe_drives: 0, foe_level: 0, scale: 1000 },
    Fight { kind: FK_ELITE, ncpu: 1, level: [6, 0, 0], stocks: [2, 0, 0], foe_drives: 1, foe_level: 1, scale: 1150 },
    Fight { kind: FK_PACK, ncpu: 2, level: [5, 6, 0], stocks: [1, 1, 0], foe_drives: 0, foe_level: 0, scale: 1000 },
    Fight { kind: FK_ELITE, ncpu: 1, level: [8, 0, 0], stocks: [2, 0, 0], foe_drives: 2, foe_level: 2, scale: 1200 },
    Fight { kind: FK_BOSS, ncpu: 1, level: [9, 0, 0], stocks: [3, 0, 0], foe_drives: 2, foe_level: 2, scale: 1250 },
];
/// The drives a foe may carry: the ones that read as an attack on the players.
const FOE_POOL: [&str; 8] = ["ember", "nova", "bolt", "frost", "rush", "bulwark", "hex", "comet"];

const RUN_NONE: i32 = 0;
const RUN_LIVE: i32 = 1;
const RUN_CLEAR: i32 = 2;
const RUN_OVER: i32 = 3;

/// The reward screens: a pick of three drives, a keystone pick.
const RW_DRIVES: i32 = 1;
const RW_KEYSTONES: i32 = 2;
const NP: usize = 2;
const SETS: usize = 4;
const OWNERS: usize = 8;
const PEND: usize = 8;
/// Echo Oath's repeat: frames after the fire; Feather Storm's hits, frames apart.
const ECHO_DELAY: i32 = 12;
const FEATHER_GAP: i32 = 8;
/// Effect reach in world units (fighters are about 10-15 tall).
const BOLT_R: f32 = 90.0;
const SPLASH_R: f32 = 26.0;
const NOVA_R: f32 = 38.0;
const SHOCK_R: f32 = 32.0;
/// Thorns' Marked (frames).
const THORNS_MARK: i32 = 300;
/// A floor drop is taken when its owner's fighter in play stands this close (x, and y above the drop).
const PICK_DX: f32 = 11.0;
const PICK_DY: f32 = 22.0;
/// Floor points (x100) a drop lands on: the main floor of every stage in the rotation is at y 0.
const DROP_X: [i32; 4] = [-3500, -1200, 1200, 3500];

// pending kinds
const PK_ECHO: i32 = 1;
const PK_PULSE: i32 = 2;
// a gain's source
const G_DROP: i32 = 1;
const G_PICK: i32 = 2;
const G_KEYSTONE: i32 = 3;
const G_SWEPT: i32 = 4;

/// The whole sim state: i32 words only (no padding), so the FNV digest over its bytes is well defined.
#[repr(C)]
struct St {
    // ---- the run (persisted in the record)
    state: i32,
    runs: i32,
    seed: i32,
    fight: i32,
    pool: i32,
    shared: i32,
    /// Humans: 1 solo, 2 co-op.
    nh: i32,
    bosses: i32,
    /// The pairs: [player][main, partner] (character kinds).
    pair: [[i32; 2]; NP],
    picks: i32,
    /// Worn drives per set (player * 2 + fighter; shared: fighter 0 only): pool index + 1 (0 empty), level 1-3.
    drives: [[i32; SLOTS]; SETS],
    levels: [[i32; SLOTS]; SETS],
    /// Keystones per player: pool index + 1 (0 empty).
    keys: [[i32; KEYSTONES]; NP],
    arm_seen: i32,
    // ---- the fight
    seq: i32,
    live: i32,
    frames: i32,
    /// Unfrozen live frames (the clock every window and charge uses: freezes do not eat them).
    play: i32,
    attrs_done: i32,
    reward_due: i32,
    outcome: i32, // 0 running, 1 won, 2 lost
    stage_clock: i32,
    stage_switches: i32,
    tags: i32,
    last_stocks: i32,
    collect: i32,
    won_pool: i32, // the pool when the last foe fell (the collect hold may outlive the players' stock reads)
    tag_cd: [i32; NP],
    active: [i32; NP],
    kind: [[i32; 2]; NP],
    tag_slot: i32,
    prev_phase: i32,
    prev_kind: i32,
    my_stocks: [i32; NP],
    foe_stocks: i32,
    stocks_each: [i32; 4],
    prev_as: [i32; 4],
    boss_slot: i32,
    boss_hits: [i32; 4],
    // charges per set and slot (time: frames; events: counts), the slot the last charging event struck, cooldowns
    charge: [[i32; SLOTS]; SETS],
    target: [[i32; SLOTS]; SETS],
    cd_until: [[i32; SLOTS]; SETS],
    /// Per owner (sets 0-3, foe slots 4-7): the last fire frame (the 20-frame rule).
    last_fire_at: [i32; OWNERS],
    relay_mask: [i32; SETS],
    relay_next: [i32; SETS],
    /// The last fire of each set (Echo repeats it): id + 1, level, target.
    last_fired: [[i32; 3]; SETS],
    /// Pending repeats and pulses: (frame, kind, id + 1, owner, level, frac, target, flags).
    pend: [[i32; 8]; PEND],
    /// Bulwark's orb per owner; Haste per owner (until); Updraft's banked jump per player.
    shield: [i32; OWNERS],
    haste_until: [i32; OWNERS],
    banked_jump: [i32; NP],
    // statuses per match slot
    burn_until: [i32; 4],
    burn_next: [i32; 4],
    burn_stacks: [i32; 4],
    chill_until: [i32; 4],
    mark_until: [i32; 4],
    /// Foe drives per match slot (elites and bosses): id + 1, level, charge, target.
    fdrives: [[i32; 2]; 4],
    flevels: [[i32; 2]; 4],
    fcharge: [[i32; 2]; 4],
    ftarget: [[i32; 2]; 4],
    atk_set: [i32; NP],
    def_set: [i32; NP],
    fires: i32,
    hits: i32,
    taken: i32,
    landings: i32,
    jumps: i32,
    blasts: i32,
    // floor drops per player: (state 0 none / 1 on the floor / 2 taken, id + 1, x100, y100)
    drop: [[[i32; 4]; DROPS]; NP],
    drops_due: i32,
    drops_made: i32,
    /// The last gain per player (presentation keys its sound on the count): frame, id + 1, fighter, level, source, n.
    gain: [[i32; 6]; NP],
    /// The fire log ring.
    log: [[i32; LOG_W]; LOG],
    log_n: i32,
    // ---- the reward screens (one per player, one shared timer)
    rw_active: i32,
    timer: i32,
    rw_done: [i32; NP],
    offers: [[i32; 3]; NP],
    cursor: [i32; NP],
    step: [i32; NP],
    fcursor: [i32; NP],
    last_pick: [i32; NP],
    last_wearer: [i32; NP],
    since_pick: i32,
}

static mut S: St = unsafe { core::mem::zeroed() };
static mut PLAN: [i32; 32] = [0; 32];
static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];

fn fnv(mut h: u32, b: &[u8]) -> u32 {
    for &x in b {
        h ^= x as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

fn rec_digest(r: &[i32; REC_WORDS]) -> i32 {
    let mut h: u32 = 0x811C_9DC5;
    for &w in &r[..REC_WORDS - 1] {
        h = fnv(h, &w.to_le_bytes());
    }
    h as i32
}

unsafe fn record() -> [i32; REC_WORDS] {
    let mut r = [0i32; REC_WORDS];
    r[0] = REC_MAGIC;
    r[1] = REC_VERSION;
    r[R_SEED] = S.seed;
    r[R_RUNS] = S.runs;
    r[R_FIGHT] = S.fight;
    r[R_POOL] = S.pool;
    r[R_SHARED] = S.shared;
    r[R_COOP] = (S.nh == 2) as i32;
    r[R_BOSSES] = S.bosses;
    for p in 0..NP {
        for k in 0..KEYSTONES {
            r[R_KEYS + p * KEYSTONES + k] = S.keys[p][k];
        }
        r[R_PAIRS + p * 2] = S.pair[p][0];
        r[R_PAIRS + p * 2 + 1] = S.pair[p][1];
    }
    r[R_PICKS] = S.picks;
    for s in 0..SETS {
        for k in 0..SLOTS {
            r[R_DRIVES + s * SLOTS + k] = S.drives[s][k] << 2 | S.levels[s][k];
        }
    }
    r[R_POOL_DIGEST] = POOL_DIGEST as i32;
    r[REC_WORDS - 1] = rec_digest(&r);
    r
}

fn is_kind(id: i32, kind: u8) -> bool {
    (1..=POOL_LEN as i32).contains(&id) && POOL[id as usize - 1].kind == kind
}

/// The run on arm: the host's saved record (Continue) or a fresh one (New run: the host forgot the record).
unsafe fn take_arm() {
    let mut r = [0i32; REC_WORDS];
    let n = gd_envoy_record(r.as_mut_ptr(), REC_WORDS as u32);
    let ok = n == REC_WORDS as i32 && r[0] == REC_MAGIC && r[1] == REC_VERSION && r[REC_WORDS - 1] == rec_digest(&r) && (0..FIGHTS as i32).contains(&r[R_FIGHT]);
    if ok && r[R_POOL_DIGEST] != POOL_DIGEST as i32 {
        log(b"envoy: saved run is from another drive pool, dropped\0", r[R_POOL_DIGEST]);
        S.state = RUN_NONE;
    } else if ok {
        S.state = RUN_LIVE;
        S.seed = r[R_SEED];
        S.runs = r[R_RUNS];
        S.fight = r[R_FIGHT];
        S.pool = r[R_POOL].max(1);
        S.shared = (r[R_SHARED] != 0) as i32;
        S.nh = if r[R_COOP] != 0 { 2 } else { 1 };
        S.bosses = r[R_BOSSES].clamp(0, 2);
        for p in 0..NP {
            for k in 0..KEYSTONES {
                let id = r[R_KEYS + p * KEYSTONES + k];
                S.keys[p][k] = if is_kind(id, K_KEYSTONE) { id } else { 0 };
            }
            S.pair[p] = [r[R_PAIRS + p * 2], r[R_PAIRS + p * 2 + 1]];
        }
        S.picks = r[R_PICKS];
        for s in 0..SETS {
            for k in 0..SLOTS {
                let w = r[R_DRIVES + s * SLOTS + k];
                let (id, lv) = (w >> 2, w & 3);
                let ok = is_kind(id, K_DRIVE) && (1..=MAX_LEVEL).contains(&lv);
                S.drives[s][k] = if ok { id } else { 0 };
                S.levels[s][k] = if ok { lv } else { 0 };
            }
        }
        log(b"envoy: armed, run continued at fight\0", S.fight + 1);
    } else {
        S.state = RUN_NONE;
        log(b"envoy: armed, new run\0", 0);
    }
    S.outcome = 0;
    S.rw_active = 0;
    S.last_pick = [-1; NP];
}

fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^ (x >> 16)
}

fn log(s: &[u8], v: i32) {
    unsafe { gd_log(s.as_ptr(), v) }
}

/// `envoy: <what> <drive key>` (value: as given).
fn log_drive(what: &[u8], id: usize, v: i32) {
    let mut b = [0u8; 80];
    let mut n = 0;
    for &x in b"envoy: ".iter().chain(what.iter()).chain(b" ".iter()).chain(POOL[id].key.as_bytes().iter()) {
        if n < b.len() - 1 {
            b[n] = x;
            n += 1;
        }
    }
    unsafe { gd_log(b.as_ptr(), v) }
}

fn sched() -> &'static Fight {
    unsafe { &SCHEDULE[S.fight.clamp(0, FIGHTS as i32 - 1) as usize] }
}

fn nh() -> i32 {
    unsafe { S.nh.clamp(1, 2) }
}

/// CPUs this fight: the schedule's, one more in co-op for a duel or a pack (the CPUs have slots 2-3 then).
unsafe fn ncpu() -> i32 {
    let f = sched();
    let extra = (nh() == 2 && (f.kind == FK_DUEL || f.kind == FK_PACK)) as i32;
    (f.ncpu + extra).min(4 - nh())
}

/// Is match slot `s` a foe (a CPU of this fight)?
unsafe fn is_foe(s: i32) -> bool {
    s >= nh() && s < nh() + ncpu()
}

unsafe fn is_boss(s: i32) -> bool {
    (is_foe(s) && (sched().kind == FK_BOSS || sched().kind == FK_ELITE)) || (s >= 0 && s == S.boss_slot)
}

// ---- drives: who wears what

/// The set fighter `f` of player `p` uses (the pair's one set when drives are shared).
unsafe fn set_of(p: usize, f: usize) -> usize {
    p * 2 + if S.shared != 0 { 0 } else { f }
}

/// Drive slots each fighter has open: 3, then 4 after the first boss (envoy-core-v2.md 2.3).
unsafe fn open_slots() -> usize {
    3 + (S.bosses >= 1) as usize
}

unsafe fn slot_of(set: usize, id: usize) -> Option<usize> {
    S.drives[set].iter().position(|&d| d == id as i32 + 1)
}

/// Whether fighter `f` of player `p` can take drive `id`: a level-up below 3, or a free slot.
unsafe fn can_take(p: usize, f: usize, id: usize) -> bool {
    let set = set_of(p, f);
    match slot_of(set, id) {
        Some(k) => S.levels[set][k] < MAX_LEVEL,
        None => S.drives[set][..open_slots()].contains(&0),
    }
}

/// Player `p`'s fighter `f` gains drive `id`: a level-up, or a new drive in the first free slot. The new level (0: no).
unsafe fn gain(p: usize, f: usize, id: usize, how: i32) -> i32 {
    let set = set_of(p, f);
    let lv = match slot_of(set, id) {
        Some(k) => {
            if S.levels[set][k] >= MAX_LEVEL {
                return 0;
            }
            S.levels[set][k] += 1;
            S.levels[set][k]
        }
        None => {
            let Some(k) = S.drives[set][..open_slots()].iter().position(|&d| d == 0) else { return 0 };
            S.drives[set][k] = id as i32 + 1;
            S.levels[set][k] = 1;
            S.charge[set][k] = 0;
            1
        }
    };
    let n = S.gain[p][5] + 1;
    S.gain[p] = [S.play, id as i32 + 1, f as i32, lv, how, n];
    log_drive(if lv > 1 { b"drive level up" } else { b"drive gained" }, id, p as i32 * 100 + f as i32 * 10 + lv);
    lv
}

unsafe fn has_key(p: usize, effect: u8) -> bool {
    S.keys[p].iter().any(|&k| k > 0 && POOL[k as usize - 1].effect == effect)
}

unsafe fn has_price(p: usize, price: u8) -> bool {
    S.keys[p].iter().any(|&k| k > 0 && POOL[k as usize - 1].price == price)
}

unsafe fn key_count(p: usize) -> i32 {
    S.keys[p].iter().filter(|&&k| k > 0).count() as i32
}

/// Keystones each player should have by now: one at the start, one after each of the first two bosses.
unsafe fn keys_earned() -> i32 {
    (1 + S.bosses).min(KEYSTONES as i32)
}

unsafe fn take_keystone(p: usize, id: usize) {
    if let Some(k) = S.keys[p].iter().position(|&k| k == 0) {
        S.keys[p][k] = id as i32 + 1;
        let n = S.gain[p][5] + 1;
        S.gain[p] = [S.play, id as i32 + 1, S.active[p], 1, G_KEYSTONE, n];
        log_drive(b"keystone taken", id, p as i32 * 10 + k as i32);
        S.atk_set[p] = -1;
        S.def_set[p] = -1;
    }
}

/// MELEE_ENVOY_TEST_DRIVES="key[:level],key/key|keystone,keystone" (main / partner | keystones), and in co-op a
/// second player's after `;`: worn from the run's start (tests, demos). Keystones given here skip the keystone pick.
unsafe fn test_drives() {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    for (p, player) in s.split(';').take(NP).enumerate() {
        let (drv, ks) = player.split_once('|').unwrap_or((player, ""));
        for (f, side) in drv.split('/').take(2).enumerate() {
            for item in side.split(',') {
                let (key, lv) = item.split_once(':').unwrap_or((item, "1"));
                let lv = lv.trim().parse::<i32>().unwrap_or(1).clamp(1, MAX_LEVEL);
                if let Some(id) = by_key(key.trim()).filter(|&i| POOL[i].kind == K_DRIVE) {
                    for _ in 0..lv {
                        gain(p, f, id, G_PICK);
                    }
                }
            }
        }
        for key in ks.split(',') {
            if let Some(id) = by_key(key.trim()).filter(|&i| POOL[i].kind == K_KEYSTONE) {
                take_keystone(p, id);
            }
        }
    }
    S.gain = [[0; 6]; NP];
}

/// The plan of the match the run wants next: the live run's current fight, or a new run's first.
unsafe fn make_plan() {
    if S.arm_seen == 0 {
        PLAN = [0; 32];
        gd_envoy_plan(PLAN.as_ptr(), 32);
        return;
    }
    let fresh = S.state != RUN_LIVE;
    let coop = if fresh { gd_envoy_coop() != 0 } else { S.nh == 2 };
    let (fight, pool, seed) = if fresh {
        (gd_envoy_start_fight() as usize, START_POOL[coop as usize], mix((S.runs as u32).wrapping_add(1).wrapping_mul(0x9E37_79B9)))
    } else {
        (S.fight as usize, S.pool, S.seed as u32)
    };
    let f = &SCHEDULE[fight];
    let n = (f.ncpu + (coop && (f.kind == FK_DUEL || f.kind == FK_PACK)) as i32).min(if coop { 2 } else { 3 });
    let p = &mut PLAN;
    *p = [0; 32];
    p[0] = 1;
    p[1] = STAGES[fight];
    p[2] = if fresh { -1 } else { S.pair[0][0] };
    p[3] = if fresh { -1 } else { S.pair[0][1] };
    p[4] = pool.max(1);
    p[5] = 1000; // the ratios are the keystones' (set every frame for the fighter in play)
    p[6] = 1000;
    p[7] = n;
    for i in 0..3 {
        let j = i.min(f.ncpu as usize - 1);
        p[8 + i] = CPU_POOL[(mix(seed ^ ((fight as u32) << 8) ^ i as u32) % CPU_POOL.len() as u32) as usize];
        p[11 + i] = f.level[j].max(1);
        p[14 + i] = f.stocks[j].max(1) + (coop && f.kind >= FK_ELITE) as i32;
        p[17 + i] = 1000;
        p[20 + i] = 1000;
        p[23 + i] = f.scale;
    }
    if f.kind >= FK_ELITE {
        // an elite or a boss: bigger, hits harder, takes less
        p[17] = f.scale;
        p[20] = 2000 - f.scale;
    }
    if f.kind == FK_BOSS {
        p[8] = CK_GANONDORF;
        p[26] = 1; // the boss fight (the engine swaps in Petey when MELEE_ENVOY_BOSS=petey)
    }
    p[27] = coop as i32;
    p[28] = if fresh || !coop { -1 } else { S.pair[1][0] };
    p[29] = if fresh || !coop { -1 } else { S.pair[1][1] };
    gd_envoy_plan(p.as_ptr(), 32);
}

/// A rewritten match began (gd_envoy_seq moved).
unsafe fn on_new_match(seq: i32) {
    if S.state != RUN_LIVE {
        S.runs = S.runs.wrapping_add(1);
        S.state = RUN_LIVE;
        S.seed = mix((S.runs as u32).wrapping_mul(0x9E37_79B9)) as i32;
        S.fight = gd_envoy_start_fight();
        S.nh = if gd_envoy_coop() != 0 { 2 } else { 1 };
        S.pool = START_POOL[(S.nh == 2) as usize];
        S.bosses = 0;
        S.drives = [[0; SLOTS]; SETS];
        S.levels = [[0; SLOTS]; SETS];
        S.keys = [[0; KEYSTONES]; NP];
        S.picks = 0;
        S.shared = (gd_envoy_shared_drives() != 0) as i32;
        S.pair = [[gd_envoy_roster(0), gd_envoy_roster(1)], [gd_envoy_roster(2), gd_envoy_roster(3)]];
        S.last_pick = [-1; NP];
        S.attrs_done = 0;
        log(b"envoy: run start, players\0", S.nh);
        if S.shared != 0 {
            log(b"envoy: drives shared by the pair\0", 1);
        }
        test_drives();
    }
    S.seq = seq;
    S.live = 0;
    S.frames = 0;
    S.play = 0;
    S.attrs_done = 0;
    let keys_short = (0..nh() as usize).any(|p| key_count(p) < keys_earned());
    S.reward_due = if keys_short { RW_KEYSTONES } else if S.fight > 0 { RW_DRIVES } else { 0 };
    S.outcome = 0;
    S.stage_clock = 0;
    S.stage_switches = 0;
    S.tags = 0;
    S.last_stocks = -1;
    S.collect = 0;
    S.won_pool = 0;
    S.tag_cd = [0; NP];
    S.active = [0; NP];
    S.kind = [[-1; 2]; NP];
    S.tag_slot = -1;
    S.prev_phase = 0;
    S.prev_kind = 0;
    S.prev_as = [-1; 4];
    S.boss_slot = -1;
    S.boss_hits = [0; 4];
    S.charge = [[0; SLOTS]; SETS];
    S.target = [[0; SLOTS]; SETS];
    S.cd_until = [[0; SLOTS]; SETS];
    S.last_fire_at = [-FIRE_GAP; OWNERS];
    S.relay_mask = [0; SETS];
    S.relay_next = [0; SETS];
    S.last_fired = [[0; 3]; SETS];
    S.pend = [[0; 8]; PEND];
    S.shield = [0; OWNERS];
    S.haste_until = [0; OWNERS];
    S.banked_jump = [0; NP];
    S.burn_until = [0; 4];
    S.burn_next = [0; 4];
    S.burn_stacks = [0; 4];
    S.chill_until = [0; 4];
    S.mark_until = [0; 4];
    S.fdrives = [[0; 2]; 4];
    S.flevels = [[0; 2]; 4];
    S.fcharge = [[0; 2]; 4];
    S.ftarget = [[0; 2]; 4];
    S.atk_set = [-1; NP];
    S.def_set = [-1; NP];
    S.drop = [[[0; 4]; DROPS]; NP];
    let f = sched();
    S.drops_due = match f.kind {
        FK_ELITE | FK_BOSS => 2,
        FK_PACK => (ncpu() / 2).max(1),
        _ => 1,
    };
    S.drops_made = 0;
    S.rw_active = 0;
    foe_drives();
    log(b"envoy: fight\0", S.fight + 1);
}

/// Elites and bosses carry drives (envoy-core-v2.md Q7), rolled from the run seed: the first CPU only.
unsafe fn foe_drives() {
    let f = sched();
    if f.foe_drives <= 0 {
        return;
    }
    let s = nh() as usize;
    let mut h = mix(S.seed as u32 ^ 0xF0E5_0000 ^ (S.fight as u32) << 4);
    let mut n = 0;
    let mut tries = 0;
    while n < f.foe_drives as usize && tries < 32 {
        tries += 1;
        let key = FOE_POOL[(h % FOE_POOL.len() as u32) as usize];
        h = mix(h.wrapping_add(0x6D2B_79F5));
        let Some(id) = by_key(key) else { continue };
        if S.fdrives[s][..n].contains(&(id as i32 + 1)) {
            continue;
        }
        S.fdrives[s][n] = id as i32 + 1;
        S.flevels[s][n] = f.foe_level.clamp(1, MAX_LEVEL);
        log_drive(b"foe carries", id, s as i32);
        n += 1;
    }
}

// ---- the reward screens

/// Words player `p`'s pair already gives or reads (bit per word), for the "one card connects" rule.
unsafe fn pair_words(p: usize) -> u32 {
    let mut m = 0u32;
    for set in [p * 2, p * 2 + 1] {
        for &d in &S.drives[set] {
            if d > 0 {
                let c = &POOL[d as usize - 1];
                m |= (1 << c.gives) | (1 << c.reads);
            }
        }
    }
    for &k in &S.keys[p] {
        if k > 0 {
            let c = &POOL[k as usize - 1];
            m |= (1 << c.gives) | (1 << c.reads);
        }
    }
    m & !1
}

/// A card player `p` may be offered (2.2): a keystone not held; a drive either fighter can take (a full fighter is
/// only offered copies of what it wears, which `can_take` allows; a level 3 drive is never offered).
unsafe fn offer_ok(p: usize, c: usize, keys: bool) -> bool {
    if keys {
        POOL[c].kind == K_KEYSTONE && !S.keys[p].contains(&(c as i32 + 1))
    } else {
        POOL[c].kind == K_DRIVE && (can_take(p, 0, c) || (S.shared == 0 && can_take(p, 1, c)))
    }
}

unsafe fn connects(c: i32, words: u32) -> bool {
    c != OFFER_STOCK && words & ((1 << POOL[c as usize].gives) | (1 << POOL[c as usize].reads)) & !1 != 0
}

unsafe fn roll_offers(p: usize) {
    let mut h = mix(S.seed as u32 ^ 0xA5A5_0000 ^ (S.fight as u32) << 4 ^ (S.picks as u32) << 12 ^ (p as u32) << 24);
    S.offers[p] = [OFFER_STOCK; 3];
    let mut n = 0;
    let keys = S.rw_active == RW_KEYSTONES;
    let mut tries = 0;
    while n < 3 && tries < 400 {
        tries += 1;
        let c = (h % POOL_LEN as u32) as usize;
        h = mix(h.wrapping_add(0x6D2B_79F5));
        if !offer_ok(p, c, keys) || S.offers[p][..n].contains(&(c as i32)) {
            continue;
        }
        S.offers[p][n] = c as i32;
        n += 1;
    }
    if !keys {
        // at least one card connects to a word the pair already gives or reads (when one can)
        let words = pair_words(p);
        if words != 0 && n == 3 && !S.offers[p].iter().any(|&c| connects(c, words)) {
            for k in 0..POOL_LEN {
                let c = (k + (h as usize % POOL_LEN)) % POOL_LEN;
                if offer_ok(p, c, false) && !S.offers[p].contains(&(c as i32)) && connects(c as i32, words) {
                    S.offers[p][2] = c as i32;
                    break;
                }
            }
        }
        if gd_player_stocks(0) <= 1 {
            S.offers[p][2] = OFFER_STOCK;
        }
    }
}

unsafe fn open_reward(kind: i32) {
    S.rw_active = kind;
    S.timer = REWARD_TIME;
    for p in 0..NP {
        S.rw_done[p] = (p as i32 >= nh() || (kind == RW_KEYSTONES && key_count(p) >= keys_earned())) as i32;
        S.cursor[p] = 0;
        S.step[p] = 0;
        S.fcursor[p] = 0;
        if S.rw_done[p] == 0 {
            roll_offers(p);
        }
    }
    log(if kind == RW_KEYSTONES { b"envoy: keystone offer, fight\0" } else { b"envoy: drive offer, fight\0" }, S.fight + 1);
}

/// The default fighter for a drive card: the one in play when it can take it, else the other.
unsafe fn default_fighter(p: usize, id: i32) -> i32 {
    if id == OFFER_STOCK || S.shared != 0 || !(0..POOL_LEN as i32).contains(&id) {
        return 0;
    }
    let a = S.active[p] as usize;
    if can_take(p, a, id as usize) { a as i32 } else { 1 - a as i32 }
}

unsafe fn finish_pick(p: usize, id: i32, f: i32) {
    if id == OFFER_STOCK {
        S.pool += 1;
        for h in 0..nh() {
            gd_envoy_apply(h, AP_STOCKS, gd_player_stocks(h) + 1);
        }
        log(b"envoy: took +1 stock, player\0", p as i32);
    } else if (0..POOL_LEN as i32).contains(&id) {
        if POOL[id as usize].kind == K_KEYSTONE {
            take_keystone(p, id as usize);
        } else {
            gain(p, f.clamp(0, 1) as usize, id as usize, G_PICK);
        }
    }
    log(b"envoy: card taken, player\0", p as i32);
    S.picks += 1;
    S.last_pick[p] = id;
    S.last_wearer[p] = f;
    S.since_pick = 0;
    S.rw_done[p] = 1;
}

/// Every player has picked (or the timer ran out): the next screen (a keystone pick at a later fight's start is
/// followed by that fight's drive pick), or the fight.
unsafe fn reward_close() {
    if S.rw_active == RW_KEYSTONES && S.fight > 0 {
        open_reward(RW_DRIVES);
        return;
    }
    S.rw_active = 0;
    gd_hold_release();
}

unsafe fn reward_frame() {
    S.timer -= 1;
    for p in 0..nh() as usize {
        if S.rw_done[p] != 0 {
            continue;
        }
        let pad = gd_pad_pressed(p as i32);
        let left = pad & (DPAD_LEFT | STICK_LEFT) != 0;
        let right = pad & (DPAD_RIGHT | STICK_RIGHT) != 0;
        if S.step[p] == 0 {
            if left {
                S.cursor[p] = (S.cursor[p] + 2) % 3;
            }
            if right {
                S.cursor[p] = (S.cursor[p] + 1) % 3;
            }
            let id = S.offers[p][S.cursor[p].clamp(0, 2) as usize];
            let one_fighter = id == OFFER_STOCK || S.rw_active == RW_KEYSTONES || S.shared != 0 || gd_tag_partner(p as i32) < 0;
            if pad & BTN_A != 0 {
                if one_fighter {
                    finish_pick(p, id, default_fighter(p, id));
                } else {
                    S.step[p] = 1;
                    S.fcursor[p] = default_fighter(p, id);
                }
            } else if S.timer <= 0 {
                let id = S.offers[p][0];
                finish_pick(p, id, default_fighter(p, id));
            }
            continue;
        }
        // step 1: which fighter wears it
        let id = S.offers[p][S.cursor[p].clamp(0, 2) as usize];
        if left || right {
            S.fcursor[p] = 1 - S.fcursor[p];
        }
        if pad & BTN_B != 0 {
            S.step[p] = 0;
        } else if pad & BTN_A != 0 && can_take(p, S.fcursor[p] as usize, id as usize) {
            finish_pick(p, id, S.fcursor[p]);
        } else if S.timer <= 0 {
            finish_pick(p, id, default_fighter(p, id));
        }
    }
    if (0..nh() as usize).all(|p| S.rw_done[p] != 0) {
        reward_close();
    }
}

// ---- positions

#[derive(Clone, Copy)]
struct P {
    x: f32,
    y: f32,
}

unsafe fn pos(slot: i32) -> Option<P> {
    if !(0..4).contains(&slot) {
        return None;
    }
    if slot == S.boss_slot {
        let mut w = [0u32; 6];
        gd_envoy_boss(w.as_mut_ptr());
        return Some(P { x: f32::from_bits(w[0]), y: f32::from_bits(w[1]) - 8.0 });
    }
    let h = gd_fighter_from_slot(slot);
    if h == 0 {
        return None;
    }
    Some(P { x: gd_fighter_get_pos_x(h), y: gd_fighter_get_pos_y(h) })
}

unsafe fn alive(s: i32) -> bool {
    (0..4).contains(&s) && gd_player_present(s) != 0 && gd_player_stocks(s) > 0
}

fn d2(a: P, b: P) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    dx * dx + dy * dy
}

/// The owner's match slot: a player's set -> the player's slot; a foe owner -> its slot.
fn owner_slot(owner: usize) -> i32 {
    if owner < SETS { (owner / 2) as i32 } else { owner as i32 - OWNER_FOE }
}

/// Is slot `s` on the other side from `owner` (players vs foes)? Drive effects only ever reach the other side.
unsafe fn enemy_of(owner: usize, s: i32) -> bool {
    alive(s) && if owner < SETS { is_foe(s) } else { (0..nh()).contains(&s) }
}

/// The live enemy slot of `owner` nearest `at` within `reach`, other than `skip` (-1: none).
unsafe fn nearest(owner: usize, at: P, skip: i32, reach: f32) -> i32 {
    let mut best = -1;
    let mut bd = reach * reach;
    for s in 0..4 {
        if s == skip || !enemy_of(owner, s) {
            continue;
        }
        if let Some(p) = pos(s) {
            let d = d2(p, at);
            if d < bd {
                bd = d;
                best = s;
            }
        }
    }
    best
}

fn x100(v: f32) -> i32 {
    (v * 100.0) as i32
}

// ---- statuses (every match slot; on a boss, Chilled and Marked act at half strength)

unsafe fn status_len(s: i32, frames: i32) -> i32 {
    // Vulnerable (a price): statuses on that player last twice as long
    if (0..nh()).contains(&s) && has_price(s as usize, P_VULNERABLE) { frames * 2 } else { frames }
}

unsafe fn burn(s: i32, stacks_to: i32, add: bool) {
    if !(0..4).contains(&s) {
        return;
    }
    let i = s as usize;
    if S.burn_until[i] <= S.play {
        S.burn_next[i] = S.play + BURN_TICK;
        S.burn_stacks[i] = 0;
    }
    S.burn_stacks[i] = if add { (S.burn_stacks[i] + 1).min(stacks_to) } else { S.burn_stacks[i].max(1) };
    S.burn_until[i] = S.burn_until[i].max(S.play + status_len(s, BURN_FRAMES));
}

unsafe fn chill(s: i32, frames: i32) {
    if (0..4).contains(&s) {
        S.chill_until[s as usize] = S.chill_until[s as usize].max(S.play + status_len(s, frames));
    }
}

unsafe fn mark(s: i32, frames: i32) {
    if (0..4).contains(&s) {
        S.mark_until[s as usize] = S.mark_until[s as usize].max(S.play + status_len(s, frames));
    }
}

unsafe fn chilled(s: i32) -> bool {
    (0..4).contains(&s) && S.chill_until[s as usize] > S.play
}

unsafe fn marked(s: i32) -> bool {
    (0..4).contains(&s) && S.mark_until[s as usize] > S.play
}

unsafe fn burning(s: i32) -> bool {
    (0..4).contains(&s) && S.burn_until[s as usize] > S.play && S.burn_stacks[s as usize] > 0
}

unsafe fn clear_status(s: usize) {
    S.burn_until[s] = 0;
    S.burn_stacks[s] = 0;
    S.chill_until[s] = 0;
    S.mark_until[s] = 0;
}

/// Haste on an owner (a player's set, or a foe slot + OWNER_FOE).
unsafe fn hasted(owner: usize) -> bool {
    S.haste_until[owner] > S.play
}

/// Percent on a slot (`milli` x1000; Petey's HP through `damage(amount)` in the engine).
unsafe fn hurt(s: i32, milli: i32) {
    if milli > 0 && alive(s) {
        gd_envoy_percent(s, 0, milli);
    }
}

// ---- firing

/// The blast of a launching drive (Q6): radius, knockback angle, growth, base. False: no capsule free.
unsafe fn launch(owner: usize, at: P, effect: u8, milli: i32, level: i32) -> bool {
    let wide = if level >= MAX_LEVEL { 1.25 } else { 1.0 };
    let (r, angle, kbg, bkb) = match effect {
        E_NOVA => (16.0 * wide, 361, 70, 35),
        E_SHOCKWAVE => (13.0 * wide, 80, 55, 30),
        _ => (9.0 * wide, 90, 75, 40), // Spire: straight up
    };
    let rc = gd_envoy_blast(owner_slot(owner), at.x, at.y, r, (milli + 500) / 1000, angle, kbg, bkb);
    if rc == 0 {
        S.blasts += 1;
        log(b"envoy: blast armed, owner*100 + effect\0", owner as i32 * 100 + effect as i32);
    } else {
        log(b"envoy: blast refused (damage only), rc\0", rc);
    }
    rc == 0
}

/// Every enemy of `owner` within `r` of `at` takes `milli` (a blast's fallback). The first one struck, or -1.
unsafe fn hurt_around(owner: usize, at: P, r: f32, milli: i32) -> i32 {
    let mut first = -1;
    for s in 0..4 {
        if enemy_of(owner, s) && pos(s).is_some_and(|p| d2(p, at) < r * r) {
            hurt(s, milli);
            if first < 0 {
                first = s;
            }
        }
    }
    first
}

/// Fire drive `id` of `owner` at `level` (`frac` x1000 of its number; `target`: the slot its charge last struck).
unsafe fn fire(owner: usize, id: usize, level: i32, target: i32, frac: i32, flags: i32) {
    let c = &POOL[id];
    let n = scaled(c.number, level) * frac / 1000;
    let me_slot = owner_slot(owner);
    let me = pos(me_slot).unwrap_or(P { x: 0.0, y: 0.0 });
    let mut at = me;
    let mut tgt = -1;
    let mut fl = flags;
    let near = |s: i32| if enemy_of(owner, s) { s } else { nearest(owner, me, -1, 1.0e6) };
    match c.effect {
        E_BURST => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            burn(tgt, BURN_STACKS, false);
            if level >= MAX_LEVEL {
                for s in 0..4 {
                    if s != tgt && enemy_of(owner, s) && pos(s).is_some_and(|p| d2(p, at) < SPLASH_R * SPLASH_R) {
                        hurt(s, n);
                        burn(s, BURN_STACKS, false);
                    }
                }
            }
        }
        E_NOVA | E_SHOCKWAVE => {
            let r = if c.effect == E_NOVA { NOVA_R } else { SHOCK_R };
            tgt = nearest(owner, me, -1, r);
            let centre = if c.effect == E_NOVA { P { x: me.x, y: me.y + 8.0 } } else { P { x: me.x, y: me.y + 3.0 } };
            if tgt >= 0 && launch(owner, centre, c.effect, n, level) {
                fl |= FL_LAUNCH;
            } else {
                tgt = hurt_around(owner, me, r, n);
            }
        }
        E_SPIRE => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
                if launch(owner, P { x: p.x, y: p.y + 6.0 }, E_SPIRE, n, level) {
                    fl |= FL_LAUNCH;
                } else {
                    hurt(tgt, n);
                }
            }
        }
        E_BOLT => {
            tgt = nearest(owner, me, -1, BOLT_R * 4.0);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            if level >= MAX_LEVEL {
                hurt(nearest(owner, at, tgt, BOLT_R), n);
            }
        }
        E_HASTE => {
            S.haste_until[owner] = S.haste_until[owner].max(S.play + n * 60 / 1000);
        }
        E_SHIELD => S.shield[owner] = 1,
        E_CHILL => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            chill(tgt, n * 60 / 1000);
        }
        E_SHATTER => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            if tgt >= 0 {
                S.chill_until[tgt as usize] = 0;
            }
            fl |= FL_LINK;
        }
        E_MARK => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            mark(tgt, n * 60 / 1000);
        }
        E_THORNS => {
            // the attacker that charged it (its last hit), else the nearest enemy
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            mark(tgt, THORNS_MARK);
            fl |= FL_LINK;
        }
        E_HEAL => {
            gd_envoy_percent(me_slot, 0, -n);
        }
        E_AIRJUMP => {
            if owner < SETS && gd_envoy_air_jump(me_slot) != 0 {
                S.banked_jump[owner / 2] = 1; // no jump used yet: kept for when one is
            }
        }
        E_FEATHERS => {
            tgt = nearest(owner, me, -1, BOLT_R * 4.0);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n / 3);
            for k in 1..3 {
                pend(PK_PULSE, S.play + k * FEATHER_GAP, id, owner, level, n / 3, tgt, 0);
            }
        }
        E_WILDFIRE => {
            for s in 0..4 {
                if enemy_of(owner, s) {
                    hurt(s, n);
                    if tgt < 0 {
                        tgt = s;
                    }
                }
            }
            fl |= FL_LINK;
        }
        E_REPEAT => {
            // Echo: the set's last fire again, at half strength
            if owner < SETS && S.last_fired[owner][0] > 0 {
                let l = S.last_fired[owner];
                pend(PK_ECHO, S.play + 1, l[0] as usize - 1, owner, l[1], n.min(1000), l[2], FL_ECHO);
            }
        }
        E_SURGE => {
            if owner < SETS {
                for k in 0..SLOTS {
                    let d = S.drives[owner][k];
                    if d > 0 && POOL[d as usize - 1].effect != E_SURGE && POOL[d as usize - 1].charge != C_PASSIVE {
                        S.charge[owner][k] = need(&POOL[d as usize - 1]);
                    }
                }
            }
        }
        E_RELAY => {
            // every other drive of this set fires, 6 frames apart (exempt from the 20-frame rule)
            if owner < SETS {
                let mut m = 0;
                for k in 0..SLOTS {
                    let d = S.drives[owner][k];
                    if d > 0 && !matches!(POOL[d as usize - 1].effect, E_RELAY | E_TAILWIND) {
                        m |= 1 << k;
                    }
                }
                S.relay_mask[owner] = m;
                S.relay_next[owner] = S.play + RELAY_GAP;
            }
        }
        E_TAILWIND => {
            tgt = target;
            if let Some(p) = pos(tgt) {
                at = p;
            }
        }
        _ => {}
    }
    S.fires += 1;
    let e = &mut S.log[S.log_n as usize % LOG];
    *e = [S.play, id as i32 + 1, owner as i32, tgt, x100(at.x), x100(at.y), level, fl];
    S.log_n += 1;
    log_drive(if fl & FL_ECHO != 0 { b"drive echo" } else { b"drive fire" }, id, owner as i32 * 10 + level);
    if owner < SETS && fl & (FL_ECHO | FL_PASSIVE) == 0 && c.effect != E_REPEAT {
        S.last_fired[owner] = [id as i32 + 1, level, tgt];
        let p = owner / 2;
        if has_key(p, E_ECHO) && c.effect != E_RELAY {
            let half = by_key("echo_oath").map_or(500, |i| POOL[i].number);
            pend(PK_ECHO, S.play + ECHO_DELAY, id, owner, level, half, tgt, FL_ECHO);
        }
        if has_price(p, P_BLEED) {
            // Bleed: each fire costs the fighter in play 1%
            gd_envoy_percent(p as i32, 0, 1000);
        }
        // Echo (the drive) counts this set's other fires
        charge_set(owner, C_FIRE, -1);
    }
}

unsafe fn pend(kind: i32, at: i32, id: usize, owner: usize, level: i32, frac: i32, target: i32, flags: i32) {
    if let Some(q) = S.pend.iter_mut().find(|q| q[1] == 0) {
        *q = [at, kind, id as i32 + 1, owner as i32, level, frac, target, flags];
    }
}

unsafe fn fire_slot(set: usize, k: usize, flags: i32) {
    let d = S.drives[set][k];
    if d <= 0 {
        return;
    }
    let c = &POOL[d as usize - 1];
    S.charge[set][k] = 0;
    if c.cooldown > 0 {
        S.cd_until[set][k] = S.play + c.cooldown * 60 / 1000;
    }
    fire(set, d as usize - 1, S.levels[set][k], S.target[set][k], 1000, flags);
}

/// What a full charge is, in the drive's own units (time: frames).
fn need(c: &Drive) -> i32 {
    if c.charge == C_TIME { c.need * 60 / 1000 } else { c.need.max(1) }
}

/// The sets of player `p` that charge and fire now (bit per fighter): the fighter in play's, and the bench's with
/// Twin Soul.
unsafe fn live_sets(p: usize) -> u32 {
    if S.shared != 0 {
        return 1;
    }
    let a = S.active[p] as u32 & 1;
    if has_key(p, E_TWIN) { 3 } else { 1 << a }
}

/// Charge every drive of one set keyed on `src` (target: the slot the event struck).
unsafe fn charge_set(set: usize, src: u8, target: i32) {
    let m = if has_key(set / 2, E_OVERCLOCK) { 2 } else { 1 };
    for k in 0..SLOTS {
        let d = S.drives[set][k];
        if d <= 0 {
            continue;
        }
        let c = &POOL[d as usize - 1];
        if c.charge != src || S.play < S.cd_until[set][k] {
            continue;
        }
        if (c.reads == W_CHILLED && !chilled(target)) || (c.reads == W_BURNING && !burning(target)) {
            continue;
        }
        S.charge[set][k] = (S.charge[set][k] + m).min(need(c));
        if target >= 0 {
            S.target[set][k] = target;
        }
    }
}

/// Charge player `p`'s live sets on `src`.
unsafe fn charge(p: usize, src: u8, target: i32) {
    let live = live_sets(p);
    for f in 0..2 {
        if live >> f & 1 != 0 {
            charge_set(set_of(p, f), src, target);
        }
    }
}

/// A foe's drives charge from its own hits, hits taken, landings and time.
unsafe fn charge_foe(s: i32, src: u8, target: i32) {
    if !(0..4).contains(&s) {
        return;
    }
    let i = s as usize;
    for k in 0..2 {
        let d = S.fdrives[i][k];
        if d > 0 && POOL[d as usize - 1].charge == src {
            S.fcharge[i][k] = (S.fcharge[i][k] + 1).min(need(&POOL[d as usize - 1]));
            if target >= 0 {
                S.ftarget[i][k] = target;
            }
        }
    }
}

/// Per unfrozen live frame: time charges, the fire queues (each set and each foe: one fire per 20 frames), Relay's
/// volleys, the pending repeats and pulses, Updraft's banked jump.
unsafe fn drives_frame() {
    for p in 0..nh() as usize {
        if !alive(p as i32) {
            continue;
        }
        charge(p, C_TIME, -1);
        let live = live_sets(p);
        for f in 0..2 {
            if live >> f & 1 == 0 || (S.shared != 0 && f == 1) {
                continue;
            }
            let set = set_of(p, f);
            if S.play - S.last_fire_at[set] >= FIRE_GAP {
                for k in 0..SLOTS {
                    let d = S.drives[set][k];
                    if d > 0 && POOL[d as usize - 1].charge != C_PASSIVE && S.charge[set][k] >= need(&POOL[d as usize - 1]) {
                        S.last_fire_at[set] = S.play;
                        fire_slot(set, k, 0);
                        break;
                    }
                }
            }
            if S.relay_mask[set] != 0 && S.play >= S.relay_next[set] {
                let k = S.relay_mask[set].trailing_zeros() as usize;
                S.relay_mask[set] &= !(1 << k);
                S.relay_next[set] = S.play + RELAY_GAP;
                fire_slot(set, k, FL_VOLLEY);
            }
        }
        if S.banked_jump[p] != 0 {
            let j = gd_envoy_jumps(p as i32);
            if j >= 0 && (j & 0xFF) >= (j >> 8) && gd_envoy_air_jump(p as i32) == 0 {
                S.banked_jump[p] = 0;
            }
        }
    }
    for s in 0..4 {
        if !is_foe(s) || !alive(s) {
            continue;
        }
        let i = s as usize;
        charge_foe(s, C_TIME, -1);
        let owner = OWNER_FOE as usize + i;
        if S.play - S.last_fire_at[owner] >= FIRE_GAP {
            for k in 0..2 {
                let d = S.fdrives[i][k];
                if d > 0 && S.fcharge[i][k] >= need(&POOL[d as usize - 1]) {
                    S.last_fire_at[owner] = S.play;
                    S.fcharge[i][k] = 0;
                    fire(owner, d as usize - 1, S.flevels[i][k], S.ftarget[i][k], 1000, 0);
                    break;
                }
            }
        }
    }
    for q in 0..PEND {
        let e = S.pend[q];
        if e[1] == 0 || S.play < e[0] {
            continue;
        }
        S.pend[q] = [0; 8];
        if e[1] == PK_ECHO {
            fire(e[3] as usize, e[2] as usize - 1, e[4], e[6], e[5], e[7]);
        } else if enemy_of(e[3] as usize, e[6]) {
            hurt(e[6], e[5]);
        }
    }
}

// ---- the take-damage and landing hooks (run inside the sim's frame: they re-run in a resimulation)

unsafe fn in_play() -> bool {
    S.live != 0 && S.state == RUN_LIVE && S.rw_active == 0 && S.outcome == 0 && S.attrs_done != 0
}

/// The owner index of the fighter in play at `slot` (a player's set, or a foe).
unsafe fn owner_at(slot: i32) -> usize {
    if (0..nh()).contains(&slot) { set_of(slot as usize, S.active[slot as usize] as usize) } else { OWNER_FOE as usize + slot as usize }
}

/// damage.fighter_take_damage pre: Bulwark's orb absorbs a hit on the fighter that armed it: the percent is skipped
/// and the knockback goes too (envoy-core-v2.md 2.4's "absorb the next hit"; V1 let the knockback through).
extern "C" fn on_damage_pre(h: u32, d: f32) -> i32 {
    unsafe {
        if !in_play() || d <= 0.0 {
            return 0;
        }
        let s = gd_fighter_slot(h);
        if !(0..4).contains(&s) || s == S.boss_slot {
            return 0;
        }
        let o = owner_at(s);
        if S.shield[o] != 0 {
            S.shield[o] = 0;
            gd_envoy_absorb(s);
            log(b"envoy: orb absorbed a hit (x100), slot\0", x100(d) * 10 + s);
            return 1; // GD_SKIP: no percent this hit
        }
    }
    0
}

/// damage.fighter_take_damage post: a hit landed (charges the attacker's drives) and taken (the victim's).
extern "C" fn on_damage(h: u32, d: f32) {
    unsafe {
        if !in_play() || d <= 0.0 {
            return;
        }
        let v = gd_fighter_slot(h);
        if !(0..4).contains(&v) {
            return;
        }
        let human_v = v < nh();
        let mut a = gd_envoy_attacker(v);
        if a < 0 || a == v {
            // no fighter source (a stage hazard): solo play has one player to credit
            a = if !human_v && nh() == 1 { 0 } else { -1 };
        }
        // Marked: the hit deals more (half on a boss)
        if marked(v) {
            let more = if is_boss(v) { BOSS_MARK_MORE } else { MARK_MORE };
            gd_envoy_percent(v, 0, (d * more as f32) as i32);
        }
        if human_v {
            S.taken += 1;
            charge(v as usize, C_TAKEN, a);
            if is_foe(a) {
                charge_foe(a, C_HIT, v);
            }
            return;
        }
        // a foe was hit
        charge_foe(v, C_TAKEN, a);
        if !(0..nh()).contains(&a) {
            return;
        }
        let p = a as usize;
        S.hits += 1;
        if has_key(p, E_PLAGUE) {
            let stacks = by_key("plague_bearer").map_or(BURN_STACKS, |i| POOL[i].number);
            burn(v, stacks, true);
        }
        if has_key(p, E_WINTER) {
            let ms = by_key("winter_oath").map_or(2000, |i| POOL[i].number);
            chill(v, ms * 60 / 1000);
        }
        // Tailwind: in Haste, the hit deals more (a passive fire for the presentation, at most one per 20 frames)
        let set = set_of(p, S.active[p] as usize);
        if hasted(set) {
            for k in 0..SLOTS {
                let id = S.drives[set][k];
                if id > 0 && POOL[id as usize - 1].effect == E_TAILWIND {
                    let more = scaled(POOL[id as usize - 1].number, S.levels[set][k]);
                    gd_envoy_percent(v, 0, (d * more as f32 / 100.0) as i32);
                    if S.play - S.cd_until[set][k] >= FIRE_GAP {
                        S.cd_until[set][k] = S.play;
                        fire(set, id as usize - 1, S.levels[set][k], v, 1000, FL_PASSIVE);
                    }
                }
            }
        }
        charge(p, C_HIT, v);
        let ha = gd_fighter_from_slot(a);
        if ha != 0 {
            let st = gd_fighter_get_action_state(ha);
            if (AS_AERIAL.0..=AS_AERIAL.1).contains(&st) {
                charge(p, C_AERIAL_HIT, v);
            }
        }
        let hv = gd_fighter_from_slot(v);
        if hv != 0 && gd_fighter_get_percent(hv) >= 100.0 {
            charge(p, C_HIT_100, v);
        }
    }
}

// ---- moments

unsafe fn on_tag(p: usize) {
    let out = S.active[p] as usize;
    let inn = 1 - out;
    S.active[p] = inn as i32;
    log(b"envoy: moment tag in, player*10 + fighter\0", p as i32 * 10 + inn as i32);
    charge(p, C_TAG_IN, -1);
}

/// Per unfrozen live frame: the stock edges (KOs dealt, stocks lost, the shared pool), jumps, Petey's hits, the
/// drops, Burning.
unsafe fn moments_frame() {
    let mut foes = 0;
    for s in 0..4 {
        let n = if (0..nh() + ncpu()).contains(&s) { gd_player_stocks(s).max(0) } else { 0 };
        let i = s as usize;
        if n < S.stocks_each[i] {
            if is_foe(s) {
                foe_ko(s);
            }
            clear_status(i);
        }
        if is_foe(s) {
            foes += n;
        }
        S.stocks_each[i] = n;
    }
    // the shared pool: a stock lost by one player is lost by both
    let low = (0..nh()).map(|p| gd_player_stocks(p)).min().unwrap_or(0);
    for p in 0..nh() as usize {
        let me = gd_player_stocks(p as i32);
        if low < S.my_stocks[p] && low > 0 {
            log(b"envoy: moment stock lost, player*100 + pool\0", p as i32 * 100 + low);
        }
        if me > low {
            gd_envoy_apply(p as i32, AP_STOCKS, low);
        }
        S.my_stocks[p] = low;
    }
    if foes < S.foe_stocks {
        log(b"envoy: moment KO dealt, foes left\0", foes);
    }
    S.foe_stocks = foes;
    // jumps and landings: entering a jump or a landing state (polled: aerial jumps and landings have no hook row)
    for s in 0..4 {
        if !(0..nh()).contains(&s) && !is_foe(s) {
            continue;
        }
        let h = gd_fighter_from_slot(s);
        let st = if h != 0 { gd_fighter_get_action_state(h) } else { -1 };
        let prev = S.prev_as[s as usize];
        S.prev_as[s as usize] = st;
        if st == prev {
            continue;
        }
        let land = AS_LANDING.iter().any(|r| (r.0..=r.1).contains(&st));
        if s < nh() {
            if (AS_JUMP.0..=AS_JUMP.1).contains(&st) {
                S.jumps += 1;
                charge(s as usize, C_JUMP, -1);
            }
            if land {
                S.landings += 1;
                charge(s as usize, C_LANDING, -1);
            }
        } else if land {
            charge_foe(s, C_LANDING, -1);
        }
    }
    // Petey's hits (the carrier never takes the take-damage path): each counts as a hit landed
    if S.boss_slot >= 0 {
        let mut w = [0u32; 6];
        gd_envoy_boss(w.as_mut_ptr());
        for p in 0..nh() as usize {
            let now = w[2 + p] as i32;
            let mut n = now.wrapping_sub(S.boss_hits[p]).clamp(0, 4);
            S.boss_hits[p] = now;
            while n > 0 {
                n -= 1;
                S.hits += 1;
                if has_key(p, E_PLAGUE) {
                    burn(S.boss_slot, BURN_STACKS, true);
                }
                charge(p, C_HIT, S.boss_slot);
            }
        }
    }
    drop_frame();
    // Burning: 2% a second per stack
    for i in 0..4 {
        if S.burn_stacks[i] > 0 && S.play >= S.burn_until[i] {
            S.burn_stacks[i] = 0;
        }
        if S.burn_stacks[i] > 0 && S.play >= S.burn_next[i] {
            S.burn_next[i] += BURN_TICK;
            if alive(i as i32) {
                gd_envoy_percent(i as i32, 0, BURN_TICK_PCT * S.burn_stacks[i]);
            }
        }
    }
}

/// A foe lost a stock: the KO charge of the player who dealt it (Wildfire reads a Burning foe), and drops.
unsafe fn foe_ko(s: i32) {
    let a = gd_envoy_attacker(s);
    let p = if (0..nh()).contains(&a) { a } else if nh() == 1 { 0 } else { -1 };
    if p >= 0 {
        let was_burning = burning(s);
        let live = live_sets(p as usize);
        for f in 0..2 {
            if live >> f & 1 == 0 {
                continue;
            }
            let set = set_of(p as usize, f);
            for k in 0..SLOTS {
                let d = S.drives[set][k];
                if d > 0 && POOL[d as usize - 1].charge == C_KO && (POOL[d as usize - 1].reads != W_BURNING || was_burning) {
                    S.charge[set][k] = need(&POOL[d as usize - 1]);
                    S.target[set][k] = s;
                }
            }
        }
    }
    if S.drops_made < S.drops_due {
        S.drops_made += 1;
        for q in 0..nh() as usize {
            roll_drop(q);
        }
    }
}

/// A KO's drop for player `p`: a drive its fighter in play can take (else its partner), on a floor point, in the
/// owner's colour; only that player can take it (co-op Q8).
unsafe fn roll_drop(p: usize) {
    let Some(slot) = S.drop[p].iter().position(|d| d[0] != 1) else { return };
    let mut h = mix(S.seed as u32 ^ 0xD20F_0000 ^ (S.fight as u32) << 8 ^ (S.drops_made as u32) << 16 ^ p as u32);
    let a = S.active[p] as usize;
    for pass in 0..2 {
        let f = if pass == 0 { a } else { 1 - a };
        for _ in 0..64 {
            let c = (h % POOL_LEN as u32) as usize;
            h = mix(h.wrapping_add(0x6D2B_79F5));
            if POOL[c].kind == K_DRIVE && can_take(p, f, c) && !S.drop[p].iter().any(|d| d[0] == 1 && d[1] == c as i32 + 1) {
                let x = DROP_X[(h >> 5) as usize % DROP_X.len()] + p as i32 * 600 + slot as i32 * 300;
                S.drop[p][slot] = [1, c as i32 + 1, x, 0];
                log_drive(b"drop", c, p as i32 * 1000 + x / 100);
                return;
            }
        }
    }
}

unsafe fn drop_frame() {
    for p in 0..nh() as usize {
        let Some(me) = pos(p as i32) else { continue };
        for k in 0..DROPS {
            let d = S.drop[p][k];
            if d[0] != 1 {
                continue;
            }
            let (dx, dy) = (me.x - d[2] as f32 / 100.0, me.y - d[3] as f32 / 100.0);
            if dx.abs() < PICK_DX && dy > -6.0 && dy < PICK_DY {
                take_drop(p, k, G_DROP);
            }
        }
    }
}

unsafe fn take_drop(p: usize, k: usize, how: i32) {
    let id = S.drop[p][k][1] as usize - 1;
    let a = S.active[p] as usize;
    let f = if can_take(p, a, id) { a } else { 1 - a };
    S.drop[p][k][0] = 2;
    if gain(p, f, id, how) == 0 {
        log_drive(b"drop lost (no room)", id, p as i32);
    }
}

unsafe fn drops_on_floor() -> bool {
    S.drop.iter().any(|q| q.iter().any(|d| d[0] == 1))
}

/// Per live frame: the ratios (keystone prices, Glass Core) and the speed of every fighter, written from a stored
/// base by the engine (`gd_envoy_speed`): nothing drifts, and a swapped-in or respawned fighter gets its own at once.
unsafe fn attrs_frame() {
    for p in 0..nh() as usize {
        let mut atk = 1000;
        let mut def = 1000;
        for &k in &S.keys[p] {
            if k <= 0 {
                continue;
            }
            let c = &POOL[k as usize - 1];
            match c.price {
                P_FRAGILE => def = def * 1250 / 1000,
                P_WEAK => atk = atk * 800 / 1000,
                _ => {}
            }
            if c.effect == E_GLASS {
                atk = atk * c.number / 1000;
                def = def * c.number / 1000;
            }
        }
        if atk != S.atk_set[p] {
            S.atk_set[p] = atk;
            gd_envoy_apply(p as i32, AP_ATTACK, atk);
        }
        if def != S.def_set[p] {
            S.def_set[p] = def;
            gd_envoy_apply(p as i32, AP_DEFENSE, def);
        }
    }
    for s in 0..4 {
        if gd_player_present(s) == 0 || s == S.boss_slot {
            continue;
        }
        let (act, bench) = if s < nh() {
            let p = s as usize;
            let slow = if has_price(p, P_SLOW) { SLOW_SPEED } else { 1000 };
            let a = S.active[p] as usize;
            let mut act = slow;
            if hasted(set_of(p, a)) {
                act = act * HASTE_SPEED / 1000;
            }
            if chilled(s) {
                act = act * CHILL_SPEED / 1000;
            }
            let bench = if hasted(set_of(p, 1 - a)) { slow * HASTE_SPEED / 1000 } else { slow };
            (act, bench)
        } else {
            let mut act = 1000;
            if hasted(OWNER_FOE as usize + s as usize) {
                act = act * HASTE_SPEED / 1000;
            }
            if chilled(s) {
                act = act * if is_boss(s) { BOSS_CHILL_SPEED } else { CHILL_SPEED } / 1000;
            }
            (act, 1000)
        };
        gd_envoy_speed(s, act, bench);
    }
}

/// The first live frame: the pairs' kinds, the stock baselines, the boss carrier.
unsafe fn fight_setup() {
    S.attrs_done = 1;
    for p in 0..nh() as usize {
        let h = gd_fighter_from_slot(p as i32);
        S.kind[p] = [if h != 0 { gd_fighter_kind(h) } else { -1 }, gd_tag_partner(p as i32)];
    }
    let mut w = [0u32; 6];
    S.boss_slot = gd_envoy_boss(w.as_mut_ptr());
    for p in 0..4 {
        S.boss_hits[p] = w[2 + p] as i32;
    }
    S.my_stocks = [0; NP];
    for p in 0..nh() as usize {
        S.my_stocks[p] = gd_player_stocks(p as i32);
    }
    S.foe_stocks = 0;
    for s in 0..4 {
        S.stocks_each[s as usize] = if (0..nh() + ncpu()).contains(&s) { gd_player_stocks(s).max(0) } else { 0 };
        if is_foe(s) {
            S.foe_stocks += S.stocks_each[s as usize];
        }
    }
}

/// Which fighter of player `p` is in: by kind when the pair differ (a respawn, a refused swap).
unsafe fn track_active(p: usize) {
    let k = S.kind[p];
    if k[0] < 0 || k[1] < 0 || k[0] == k[1] {
        return;
    }
    let h = gd_fighter_from_slot(p as i32);
    if h == 0 {
        return;
    }
    let now = gd_fighter_kind(h);
    if now == k[0] {
        S.active[p] = 0;
    } else if now == k[1] {
        S.active[p] = 1;
    }
}

unsafe fn tag_frame() {
    let phase = gd_transition_phase();
    if phase == 1 {
        if gd_transition_kind() == KIND_TAG {
            let s = gd_transition_slot();
            S.tag_slot = s;
            let f = gd_transition_frame();
            if f >= TAG_SKIP_AFTER && gd_pad_pressed(s.clamp(0, 3)) & DPAD_DOWN != 0 {
                gd_tag_swap(1);
            } else if f + 1 >= TAG_LEN {
                gd_tag_swap(0);
            }
        }
        return;
    }
    for p in 0..nh() as usize {
        if S.tag_cd[p] > 0 {
            S.tag_cd[p] -= 1;
            continue;
        }
        if gd_tag_partner(p as i32) < 0 {
            continue;
        }
        let h = gd_fighter_from_slot(p as i32);
        if h == 0 {
            continue;
        }
        let mut w = [0u32; 8];
        gd_fighter_get_input(h, w.as_mut_ptr());
        if w[6] & DPAD_DOWN == 0 {
            continue;
        }
        let rc = gd_tag_begin_ex(p as i32, TAG_LEN, 1);
        if rc != 0 {
            log(b"envoy: tag refused, rc*1000 - action state\0", rc * 1000 - gd_fighter_get_action_state(h));
        }
        if rc == 0 {
            S.tag_cd[p] = if has_price(p, P_LOCK) { LOCK_TAG_COOLDOWN } else { TAG_COOLDOWN };
            S.tags += 1;
            log(b"envoy: tag, player*100 + count\0", p as i32 * 100 + S.tags);
            return; // one transition at a time
        }
    }
}

unsafe fn stage_frame() {
    if S.fight < 1 || gd_transition_phase() != 0 || S.boss_slot >= 0 {
        return;
    }
    S.stage_clock += 1;
    if S.stage_clock < STAGE_PERIOD {
        return;
    }
    let cur = gd_stage_grkind();
    let next = match STAGES.iter().position(|&k| k == cur) {
        Some(i) => STAGES[(i + 1) % STAGES.len()],
        None => STAGES[0],
    };
    let rc = gd_stage_transition(next, STAGE_LEN);
    if rc == 0 {
        S.stage_clock = 0;
        S.stage_switches += 1;
        log(b"envoy: stage transition to\0", next);
        for p in 0..nh() as usize {
            charge(p, C_SHIFT, -1);
        }
    } else if rc != -9 {
        S.stage_clock = 0;
        log(b"envoy: stage transition refused\0", rc);
    }
}

unsafe fn outcome_frame() {
    if S.outcome != 0 {
        return;
    }
    let me = (0..nh()).map(|p| gd_player_stocks(p)).min().unwrap_or(0);
    let mut foes = 0;
    for s in 0..4 {
        if is_foe(s) {
            foes += gd_player_stocks(s).max(0);
        }
    }
    let key = me * 100 + foes;
    if key != S.last_stocks {
        S.last_stocks = key;
        log(b"envoy: stocks (pool*100 + foes)\0", key);
    }
    // the win latches when the last foe falls: the retail game end that follows tears the players down (their stock
    // reads go to 0), which must not read as a loss
    if S.collect == 0 && foes == 0 && me > 0 {
        S.collect = 1;
        S.won_pool = me;
    }
    if S.collect > 0 {
        if me > 0 {
            S.won_pool = me;
        }
        // the collect hold: up to 10 s to walk over a drop still on the floor, then it is swept up
        if me > 0 && drops_on_floor() && S.collect < COLLECT_HOLD {
            S.collect += 1;
            return;
        }
        fight_won();
    } else if me <= 0 {
        S.outcome = 2;
        S.pool = 0;
        S.state = RUN_OVER;
        log(b"envoy: RUN OVER at fight\0", S.fight + 1);
        make_plan();
    }
}

/// The fight is won (the last foe fell; `S.won_pool` is the pool left): sweep the drops, advance the run.
unsafe fn fight_won() {
    let me = S.won_pool.max(1);
    for p in 0..nh() as usize {
        for k in 0..DROPS {
            if S.drop[p][k][0] == 1 {
                take_drop(p, k, G_SWEPT);
            }
        }
    }
    S.outcome = 1;
    S.pool = me;
    if sched().kind >= FK_ELITE && S.bosses < 2 && (S.fight as usize) < FIGHTS - 1 {
        S.bosses += 1;
        log(b"envoy: boss beaten, keystone due; bosses ", S.bosses);
    }
    if S.fight as usize == FIGHTS - 1 {
        S.state = RUN_CLEAR;
        log(b"envoy: RUN CLEAR, pool left ", me);
    } else {
        S.fight += 1;
        log(b"envoy: fight won, next ", S.fight + 1);
    }
    make_plan();
}

/// FNV-1a over the whole state and the pool digest: this frame's word for the compared netplay / synctest hash.
unsafe fn digest() -> u32 {
    let b = core::slice::from_raw_parts(core::ptr::addr_of!(S) as *const u8, core::mem::size_of::<St>());
    let p = core::slice::from_raw_parts(PLAN.as_ptr() as *const u8, PLAN.len() * 4);
    fnv(fnv(fnv(0x811C_9DC5, &POOL_DIGEST.to_le_bytes()), b), p)
}

fn fbits(v: f32) -> i32 {
    v.to_bits() as i32
}

fn left(until: i32, now: i32) -> i32 {
    (until - now).max(0)
}

/// The view (envoy_data::view): the run, each player, every match slot, the fire log, the record.
unsafe fn publish() {
    let v = &mut VIEW;
    *v = [0; VIEW_WORDS];
    v[V_MAGIC] = VIEW_MAGIC;
    v[V_STATE] = S.state;
    v[V_FIGHT] = S.fight;
    v[V_FIGHTS] = FIGHTS as i32;
    v[V_POOL] = if S.live != 0 { (0..nh()).map(|p| gd_player_stocks(p)).min().unwrap_or(0) } else { S.pool };
    v[V_LIVE] = S.live;
    v[V_HUMANS] = nh();
    v[V_SHARED] = S.shared;
    v[V_RW] = S.rw_active;
    v[V_TIMER] = S.timer;
    v[V_RW_TIME] = REWARD_TIME;
    v[V_FROZEN] = (gd_transition_phase() != 0 || S.rw_active != 0) as i32;
    v[V_PLAY] = S.play;
    v[V_DIGEST] = POOL_DIGEST as i32;
    v[V_STAGE_IN] = if S.fight >= 1 && S.live != 0 && S.boss_slot < 0 { STAGE_PERIOD - S.stage_clock } else { -1 };
    v[V_FIRES] = S.fires;
    v[V_FIGHT_KIND] = sched().kind;
    v[V_OPEN] = open_slots() as i32;
    v[V_KEYS_EARNED] = keys_earned();
    v[V_SEQ] = gd_envoy_seq();
    v[V_ARMED] = S.arm_seen;
    v[V_NCPU] = ncpu();
    v[V_SINCE_PICK] = S.since_pick;
    v[V_BOSSES] = S.bosses;
    v[V_OUTCOME] = S.outcome;
    for p in 0..NP {
        let b = V_P0 + p * V_PW;
        v[b + P_PRESENT] = (p < nh() as usize) as i32;
        v[b + P_MAIN] = S.pair[p][0];
        v[b + P_PARTNER] = S.pair[p][1];
        if p >= nh() as usize {
            continue;
        }
        v[b + P_ACTIVE] = S.active[p];
        v[b + P_TAG_CD] = S.tag_cd[p];
        v[b + P_TAG_MAX] = if has_price(p, P_LOCK) { LOCK_TAG_COOLDOWN } else { TAG_COOLDOWN };
        v[b + P_PARTNER_KIND] = gd_tag_partner(p as i32);
        if let Some(q) = pos(p as i32) {
            v[b + P_X] = fbits(q.x);
            v[b + P_Y] = fbits(q.y);
            v[b + P_HAVE_POS] = alive(p as i32) as i32;
        }
        for k in 0..KEYSTONES {
            v[b + P_KEYS + k] = S.keys[p][k];
        }
        v[b + P_LAST_PICK] = S.last_pick[p];
        v[b + P_LAST_WEARER] = S.last_wearer[p];
        for f in 0..2 {
            let set = p * 2 + f;
            for k in 0..SLOTS {
                let d = S.drives[set][k];
                v[b + P_DRIVES + f * SLOTS + k] = d;
                v[b + P_LEVELS + f * SLOTS + k] = S.levels[set][k];
                v[b + P_CHARGE + f * SLOTS + k] = if d > 0 { S.charge[set][k] * 1000 / need(&POOL[d as usize - 1]).max(1) } else { 0 };
            }
            v[b + P_HASTE + f] = left(S.haste_until[set], S.play);
            v[b + P_SHIELD + f] = S.shield[set];
        }
        v[b + P_LIVE_SETS] = live_sets(p) as i32;
        v[b + P_RW_DONE] = S.rw_done[p];
        v[b + P_CURSOR] = S.cursor[p];
        v[b + P_STEP] = S.step[p];
        v[b + P_FCURSOR] = S.fcursor[p];
        let mut bits = 0;
        for i in 0..3 {
            let id = S.offers[p][i];
            v[b + P_OFFERS + i] = id;
            if (0..POOL_LEN as i32).contains(&id) && POOL[id as usize].kind == K_DRIVE {
                for f in 0..2 {
                    if can_take(p, f, id as usize) {
                        bits |= 1 << (i * 2 + f);
                    }
                }
            }
        }
        v[b + P_CAN_TAKE] = bits;
        for k in 0..DROPS {
            v[b + P_DROPS + k * 4..b + P_DROPS + k * 4 + 4].copy_from_slice(&S.drop[p][k]);
        }
        v[b + P_BANKED_JUMP] = S.banked_jump[p];
        v[b + P_GAIN..b + P_GAIN + 6].copy_from_slice(&S.gain[p]);
    }
    for s in 0..4 {
        let b = V_S0 + s as usize * V_SW;
        let i = s as usize;
        v[b + S_ALIVE] = alive(s) as i32;
        if let Some(q) = pos(s) {
            v[b + S_X] = fbits(q.x);
            v[b + S_Y] = fbits(q.y);
        }
        v[b + S_BURN] = if burning(s) { S.burn_stacks[i] } else { 0 };
        v[b + S_CHILL] = left(S.chill_until[i], S.play);
        v[b + S_MARK] = left(S.mark_until[i], S.play);
        v[b + S_FOE] = is_foe(s) as i32;
        v[b + S_BOSS] = is_boss(s) as i32;
        for k in 0..2 {
            let d = S.fdrives[i][k];
            v[b + S_DRIVES + k] = d;
            v[b + S_LEVELS + k] = S.flevels[i][k];
            v[b + S_CHARGE + k] = if d > 0 { S.fcharge[i][k] * 1000 / need(&POOL[d as usize - 1]).max(1) } else { 0 };
        }
        v[b + S_HASTE] = left(S.haste_until[OWNER_FOE as usize + i], S.play);
        v[b + S_SHIELD] = S.shield[OWNER_FOE as usize + i];
    }
    for e in 0..LOG {
        v[V_LOG + e * LOG_W..V_LOG + (e + 1) * LOG_W].copy_from_slice(&S.log[e]);
    }
    v[V_LOG_N] = S.log_n;
    let r = record();
    v[V_REC..V_REC + REC_WORDS].copy_from_slice(&r);
    gd_envoy_publish(v.as_ptr(), VIEW_WORDS as u32);
}

extern "C" fn on_frame_end() {
    unsafe {
        let armed = (gd_envoy_armed() != 0) as i32;
        if armed != S.arm_seen {
            S.arm_seen = armed;
            if armed != 0 {
                take_arm();
            } else {
                S.state = RUN_NONE;
                log(b"envoy: disarmed\0", 0);
            }
            make_plan();
        }
        let seq = gd_envoy_seq();
        if seq != S.seq {
            on_new_match(seq);
        }
        let present = gd_player_present(0) != 0;
        if present && S.seq != 0 && S.state == RUN_LIVE {
            S.live = 1;
            S.frames += 1;
            if S.since_pick < 9999 {
                S.since_pick += 1;
            }
            if S.attrs_done == 0 {
                fight_setup();
            }
            let phase = gd_transition_phase();
            let kind = gd_transition_kind();
            if S.prev_phase == 1 && phase == 0 && S.prev_kind == KIND_TAG && (0..nh()).contains(&S.tag_slot) {
                on_tag(S.tag_slot as usize);
            }
            S.prev_phase = phase;
            S.prev_kind = kind;
            for p in 0..nh() as usize {
                track_active(p);
            }
            if S.rw_active != 0 {
                reward_frame();
            } else if S.reward_due != 0 && S.frames >= REWARD_DELAY && phase == 0 && S.outcome == 0 {
                if gd_hold_begin(REWARD_TIME * 4 + 120) == 0 {
                    let due = S.reward_due;
                    S.reward_due = 0;
                    open_reward(due);
                    if (0..nh() as usize).all(|p| S.rw_done[p] != 0) {
                        reward_close();
                    }
                }
            } else if phase == 0 || kind != KIND_HOLD {
                tag_frame();
                stage_frame();
                if gd_transition_phase() == 0 && S.reward_due == 0 && S.outcome == 0 {
                    S.play += 1;
                    moments_frame();
                    drives_frame();
                }
                outcome_frame();
            }
            if S.outcome == 0 {
                attrs_frame();
            }
        } else if !present {
            S.live = 0;
            // the match ended inside the collect hold: the latched win stands
            if S.state == RUN_LIVE && S.outcome == 0 && S.collect > 0 {
                fight_won();
            }
        }
        gd_hash_word(digest());
        publish();
    }
}

#[no_mangle]
pub extern "C" fn gd_mod_loaded() {
    unsafe {
        S.seed = 1;
        S.nh = 1;
        S.pool = START_POOL[0];
        S.pair = [[-1; 2]; NP];
        S.last_pick = [-1; NP];
        S.last_wearer = [-1; NP];
        S.since_pick = 9999;
        S.last_stocks = -1;
        S.boss_slot = -1;
        S.offers = [[0, 1, 2]; NP];
        S.arm_seen = (gd_envoy_armed() != 0) as i32;
        make_plan();
        publish();
    }
}

#[repr(C)]
pub struct HookDecl { address: u32, mode: u32, priority: i32, flags: u32, f: *const () }
#[repr(C)]
pub struct Manifest {
    magic: u32, struct_size: u32, abi_min: u32, abi_max: u32, hook_table_version: u32, mod_version: u32,
    required_features: u32, optional_features: u32, state_bytes: u32, hook_count: u32, hooks: *const HookDecl,
    mod_id: [u8; 32], mod_name: [u8; 48],
}
unsafe impl Sync for Manifest {}
unsafe impl Sync for HookDecl {}

const fn text<const N: usize>(s: &[u8]) -> [u8; N] {
    let mut o = [0u8; N];
    let mut i = 0;
    while i < s.len() { o[i] = s[i]; i += 1; }
    o
}

const ABI: u32 = 1 << 16;
const PRE: u32 = 1;
const POST: u32 = 2;
static HOOKS: [HookDecl; 3] = [
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 40, flags: 0, f: on_frame_end as *const () },
    HookDecl { address: 0x8006_CC7C, mode: PRE, priority: 40, flags: 0, f: on_damage_pre as *const () },
    HookDecl { address: 0x8006_CC7C, mode: POST, priority: 40, flags: 0, f: on_damage as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0004_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 4096, hook_count: 3, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.envoy_sim"), mod_name: text(b"Envoy run"),
};
