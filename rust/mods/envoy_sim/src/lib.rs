//! envoy_sim: a single-player Envoy run (vertical slice, docs/rust-port/notes/envoy-1p.md). The sim half: run state,
//! the plan of the next fight, tags, mid-fight stage switches, the reward pick and the run's end. Everything is this
//! module's linear memory (rolled back with the game) or engine statics (melee_modes::tu::{envoy, tag_swap}); the
//! presentation (envoy_ui) only reads the published view.
//!
//! * run: 5 fights against CPUs that ramp up (level and count); fight 5 is the boss stand-in (a level-9 Ganondorf with
//!   damage, defense and size buffs). The player's pair (P1 = main, P2 = partner on the retail CSS) shares one stock
//!   pool across the run; a fight is won when every CPU is out of stocks, the run is over when the pool is empty.
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown, 60-frame frozen cinematic; each fighter keeps its %.
//! * stage switches: from fight 2 on, every 30 s of play, the next of BF / YS / DL / FoD / FD, under a 90-frame
//!   frozen wipe-and-flash.
//! * rewards: at the start of fights 2-5 the sim freezes and offers 3 upgrades (D-pad / stick left-right, A picks;
//!   20 s, then offer 0).
#![no_std]
#![allow(static_mut_refs)]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_fighter_from_slot(slot: i32) -> u32;
    fn gd_fighter_get_input(h: u32, out: *mut u32);
    fn gd_player_present(slot: i32) -> i32;
    fn gd_player_stocks(slot: i32) -> i32;
    fn gd_tag_partner(slot: i32) -> i32;
    fn gd_tag_begin_ex(slot: i32, len: i32, flags: i32) -> i32;
    fn gd_tag_swap(skip: i32) -> i32;
    fn gd_transition_phase() -> i32;
    fn gd_transition_kind() -> i32;
    fn gd_transition_frame() -> i32;
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
    fn gd_hash_word(w: u32);
    fn gd_log(msg: *const u8, value: i32);
}

const FIGHTS: usize = 5;
const START_POOL: i32 = 4;
const TAG_LEN: i32 = 60;
const TAG_COOLDOWN: i32 = 300;
const TAG_SKIP_AFTER: i32 = 6;
const STAGE_LEN: i32 = 90;
const STAGE_PERIOD: i32 = 1800;
const REWARD_TIME: i32 = 1200;
/// Frames into a fight before its reward screen opens (after the fighters are placed).
const REWARD_DELAY: i32 = 2;

const DPAD_LEFT: u32 = 0x0001;
const DPAD_RIGHT: u32 = 0x0002;
const DPAD_DOWN: u32 = 0x0004;
const BTN_A: u32 = 0x0100;
const STICK_LEFT: u32 = 0x0004_0000;
const STICK_RIGHT: u32 = 0x0008_0000;

const KIND_TAG: i32 = 1;
const KIND_HOLD: i32 = 3;

/// Gr_Kind: Battlefield, Yoshi's Story, Dream Land, Fountain of Dreams, Final Destination.
const STAGES: [i32; 5] = [36, 10, 28, 12, 37];
/// CPU characters (CharacterKind, the CSS ids): the retail cast minus Ice Climbers / Zelda / Sheik / Ganondorf (boss).
const CPU_POOL: [i32; 22] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 20, 21, 22, 23, 24];
const CK_GANONDORF: i32 = 0x19;

struct Fight {
    ncpu: i32,
    level: [i32; 3],
    stocks: [i32; 3],
}
const SCHEDULE: [Fight; FIGHTS] = [
    Fight { ncpu: 1, level: [3, 0, 0], stocks: [1, 0, 0] },
    Fight { ncpu: 1, level: [5, 0, 0], stocks: [2, 0, 0] },
    Fight { ncpu: 2, level: [5, 6, 0], stocks: [1, 1, 0] },
    Fight { ncpu: 3, level: [6, 7, 7], stocks: [1, 1, 1] },
    Fight { ncpu: 1, level: [9, 0, 0], stocks: [3, 0, 0] },
];

// upgrades: stat index (ST_*) and step (x1000), or the stock pool
const ST_ATK: usize = 0;
const ST_DEF: usize = 1;
const ST_WEIGHT: usize = 2;
const ST_SPEED: usize = 3;
const ST_JUMP: usize = 4;
const N_UPGRADES: i32 = 6;
const UP_STOCK: i32 = 2;
/// (stat, step) per upgrade id; id 2 (Extra stock) has no stat.
const UPGRADES: [(usize, i32); 6] = [(ST_ATK, 200), (ST_DEF, -150), (99, 1), (ST_WEIGHT, 200), (ST_SPEED, 120), (ST_JUMP, 120)];
/// gd_envoy_apply `which` per stat.
const APPLY: [i32; 5] = [0, 1, 3, 4, 5];

const RUN_NONE: i32 = 0;
const RUN_LIVE: i32 = 1;
const RUN_CLEAR: i32 = 2;
const RUN_OVER: i32 = 3;

struct Run {
    state: i32,
    runs: u32,
    seed: u32,
    fight: i32,
    pool: i32,
    stats: [i32; 5],
    main: i32,
    partner: i32,
    picks: i32,
}

struct Match {
    seq: i32,
    live: bool,
    frames: i32,
    attrs_done: bool,
    reward_due: bool,
    outcome: i32, // 0 running, 1 won, 2 lost
    tag_cd: i32,
    stage_clock: i32,
    stage_switches: i32,
    tags: i32,
    last_stocks: i32,
}

struct Reward {
    active: bool,
    offers: [i32; 3],
    cursor: i32,
    timer: i32,
    last_pick: i32,
    since_pick: i32,
}

static mut RUN: Run = Run { state: RUN_NONE, runs: 0, seed: 1, fight: 0, pool: START_POOL, stats: [1000; 5], main: -1, partner: -1, picks: 0 };
static mut M: Match = Match { seq: 0, live: false, frames: 0, attrs_done: false, reward_due: false, outcome: 0, tag_cd: 0, stage_clock: 0, stage_switches: 0, tags: 0, last_stocks: -1 };
static mut RW: Reward = Reward { active: false, offers: [0, 1, 2], cursor: 0, timer: 0, last_pick: -1, since_pick: 9999 };
static mut PLAN: [i32; 32] = [0; 32];
static mut VIEW: [i32; 64] = [0; 64];

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

/// The plan of the match the run wants next: the live run's current fight, or a new run's first.
unsafe fn make_plan() {
    let fresh = RUN.state != RUN_LIVE;
    let (fight, pool, stats, seed) = if fresh {
        (gd_envoy_start_fight() as usize, START_POOL, [1000; 5], mix(RUN.runs.wrapping_add(1).wrapping_mul(0x9E37_79B9)))
    } else {
        (RUN.fight as usize, RUN.pool, RUN.stats, RUN.seed)
    };
    let f = &SCHEDULE[fight];
    let p = &mut PLAN;
    *p = [0; 32];
    p[0] = 1;
    p[1] = STAGES[fight];
    p[2] = if fresh { -1 } else { RUN.main };
    p[3] = if fresh { -1 } else { RUN.partner };
    p[4] = pool.max(1);
    p[5] = stats[ST_ATK];
    p[6] = stats[ST_DEF];
    p[7] = f.ncpu;
    for i in 0..3 {
        p[8 + i] = CPU_POOL[(mix(seed ^ ((fight as u32) << 8) ^ i as u32) % CPU_POOL.len() as u32) as usize];
        p[11 + i] = f.level[i].max(1);
        p[14 + i] = f.stocks[i].max(1);
        p[17 + i] = 1000;
        p[20 + i] = 1000;
        p[23 + i] = 1000;
    }
    if fight == FIGHTS - 1 {
        p[8] = CK_GANONDORF;
        p[17] = 1250;
        p[20] = 750;
        p[23] = 1250;
        p[26] = 1; // the boss fight (the engine swaps in Petey when MELEE_ENVOY_BOSS=petey)
    }
    gd_envoy_plan(p.as_ptr(), 32);
}

/// A rewritten match began (gd_envoy_seq moved).
unsafe fn on_new_match(seq: i32) {
    if RUN.state != RUN_LIVE {
        RUN.runs = RUN.runs.wrapping_add(1);
        RUN.state = RUN_LIVE;
        RUN.seed = mix(RUN.runs.wrapping_mul(0x9E37_79B9));
        RUN.fight = gd_envoy_start_fight();
        RUN.pool = START_POOL;
        RUN.stats = [1000; 5];
        RUN.picks = 0;
        RUN.main = gd_envoy_roster(0);
        RUN.partner = gd_envoy_roster(1);
        RW.last_pick = -1;
        log(b"envoy: run start, main\0", RUN.main);
    }
    M = Match { seq, live: false, frames: 0, attrs_done: false, reward_due: RUN.fight > 0, outcome: 0, tag_cd: 0, stage_clock: 0, stage_switches: 0, tags: 0, last_stocks: -1 };
    RW.active = false;
    log(b"envoy: fight\0", RUN.fight + 1);
}

unsafe fn roll_offers() {
    let mut h = mix(RUN.seed ^ 0xA5A5_0000 ^ (RUN.fight as u32) << 4 ^ RUN.picks as u32);
    let mut n = 0;
    while n < 3 {
        let c = (h % N_UPGRADES as u32) as i32;
        h = mix(h.wrapping_add(0x6D2B_79F5));
        if RW.offers[..n].contains(&c) {
            continue;
        }
        RW.offers[n] = c;
        n += 1;
    }
}

/// (before, after) of an upgrade for the compare row.
unsafe fn compare(id: i32) -> (i32, i32) {
    let (st, step) = UPGRADES[id as usize];
    if id == UP_STOCK {
        let s = gd_player_stocks(0);
        return (s, s + step);
    }
    let b = RUN.stats[st];
    let a = if st == ST_DEF { (b + step).max(400) } else { b + step };
    (b, a)
}

unsafe fn pick(id: i32) {
    let (b, a) = compare(id);
    if id == UP_STOCK {
        gd_envoy_apply(0, 2, a);
    } else {
        let st = UPGRADES[id as usize].0;
        RUN.stats[st] = a;
        if st == ST_ATK || st == ST_DEF {
            gd_envoy_apply(0, APPLY[st], a);
        } else {
            gd_envoy_apply(0, APPLY[st], a * 1000 / b.max(1));
        }
    }
    RUN.picks += 1;
    RW.active = false;
    RW.last_pick = id;
    RW.since_pick = 0;
    gd_hold_release();
    log(b"envoy: reward picked\0", id);
}

unsafe fn reward_frame() {
    let pad = gd_pad_pressed(0);
    if pad & (DPAD_LEFT | STICK_LEFT) != 0 {
        RW.cursor = (RW.cursor + 2) % 3;
    }
    if pad & (DPAD_RIGHT | STICK_RIGHT) != 0 {
        RW.cursor = (RW.cursor + 1) % 3;
    }
    RW.timer -= 1;
    if pad & BTN_A != 0 {
        pick(RW.offers[RW.cursor as usize]);
    } else if RW.timer <= 0 {
        pick(RW.offers[0]);
    }
}

unsafe fn tag_frame() {
    let phase = gd_transition_phase();
    if phase == 1 {
        if gd_transition_kind() == KIND_TAG {
            let f = gd_transition_frame();
            if f >= TAG_SKIP_AFTER && gd_pad_pressed(0) & DPAD_DOWN != 0 {
                gd_tag_swap(1);
            } else if f + 1 >= TAG_LEN {
                gd_tag_swap(0);
            }
        }
        return;
    }
    if M.tag_cd > 0 {
        M.tag_cd -= 1;
        return;
    }
    if gd_tag_partner(0) < 0 {
        return;
    }
    let h = gd_fighter_from_slot(0);
    if h == 0 {
        return;
    }
    let mut w = [0u32; 8];
    gd_fighter_get_input(h, w.as_mut_ptr());
    if w[6] & DPAD_DOWN == 0 {
        return;
    }
    let rc = gd_tag_begin_ex(0, TAG_LEN, 1);
    if rc == 0 {
        M.tag_cd = TAG_COOLDOWN;
        M.tags += 1;
        log(b"envoy: tag\0", M.tags);
    }
}

unsafe fn stage_frame() {
    if RUN.fight < 1 || gd_transition_phase() != 0 {
        return;
    }
    M.stage_clock += 1;
    if M.stage_clock < STAGE_PERIOD {
        return;
    }
    let cur = gd_stage_grkind();
    let next = match STAGES.iter().position(|&k| k == cur) {
        Some(i) => STAGES[(i + 1) % STAGES.len()],
        None => STAGES[0],
    };
    let rc = gd_stage_transition(next, STAGE_LEN);
    if rc == 0 {
        M.stage_clock = 0;
        M.stage_switches += 1;
        log(b"envoy: stage transition to\0", next);
    } else if rc != -9 {
        M.stage_clock = 0; // refused for good this time (e.g. unsupported stage): try again a period later
        log(b"envoy: stage transition refused\0", rc);
    }
}

unsafe fn outcome_frame() {
    if M.outcome != 0 {
        return;
    }
    let me = gd_player_stocks(0);
    let mut foes = 0;
    for s in 1..=SCHEDULE[RUN.fight as usize].ncpu {
        foes += gd_player_stocks(s).max(0);
    }
    let key = me * 100 + foes;
    if key != M.last_stocks {
        M.last_stocks = key;
        log(b"envoy: stocks (pool*100 + foes) ", key);
    }
    if foes == 0 && me > 0 {
        M.outcome = 1;
        RUN.pool = me;
        if RUN.fight as usize == FIGHTS - 1 {
            RUN.state = RUN_CLEAR;
            log(b"envoy: RUN CLEAR, pool left\0", me);
        } else {
            RUN.fight += 1;
            log(b"envoy: fight won, next\0", RUN.fight + 1);
        }
        make_plan();
    } else if me <= 0 {
        M.outcome = 2;
        RUN.pool = 0;
        RUN.state = RUN_OVER;
        log(b"envoy: RUN OVER at fight\0", RUN.fight + 1);
        make_plan();
    }
}

unsafe fn publish() {
    let v = &mut VIEW;
    v[0] = 0x454E_5659;
    v[1] = RUN.state;
    v[2] = RUN.fight;
    v[3] = FIGHTS as i32;
    v[4] = if M.live { gd_player_stocks(0) } else { RUN.pool };
    v[5] = M.live as i32;
    v[6] = RUN.main;
    v[7] = RUN.partner;
    v[8] = M.tag_cd;
    v[9] = TAG_COOLDOWN;
    v[10] = RW.active as i32;
    v[11] = RW.cursor;
    v[12] = RW.timer;
    for i in 0..3 {
        let (b, a) = compare(RW.offers[i]);
        v[13 + i] = RW.offers[i];
        v[16 + i] = b;
        v[19 + i] = a;
    }
    for i in 0..5 {
        v[22 + i] = RUN.stats[i];
    }
    v[27] = if RUN.fight >= 1 && M.live { STAGE_PERIOD - M.stage_clock } else { -1 };
    v[28] = SCHEDULE[RUN.fight as usize].ncpu;
    v[29] = (RUN.fight as usize == FIGHTS - 1) as i32;
    v[30] = RW.last_pick;
    v[31] = RW.since_pick;
    v[32] = M.outcome;
    v[33] = REWARD_TIME;
    v[34] = gd_tag_partner(0);
    v[35] = M.stage_switches;
    gd_envoy_publish(v.as_ptr(), 64);
}

extern "C" fn on_frame_end() {
    unsafe {
        let seq = gd_envoy_seq();
        if seq != M.seq {
            on_new_match(seq);
        }
        let present = gd_player_present(0) != 0;
        if present && M.seq != 0 && RUN.state == RUN_LIVE {
            M.live = true;
            M.frames += 1;
            if RW.since_pick < 9999 {
                RW.since_pick += 1;
            }
            if !M.attrs_done {
                M.attrs_done = true;
                for st in [ST_WEIGHT, ST_SPEED, ST_JUMP] {
                    if RUN.stats[st] != 1000 {
                        gd_envoy_apply(0, APPLY[st], RUN.stats[st]);
                    }
                }
            }
            if RW.active {
                reward_frame();
            } else if M.reward_due && M.frames >= REWARD_DELAY && gd_transition_phase() == 0 && M.outcome == 0 {
                if gd_hold_begin(REWARD_TIME + 120) == 0 {
                    M.reward_due = false;
                    RW.active = true;
                    RW.cursor = 0;
                    RW.timer = REWARD_TIME;
                    roll_offers();
                    log(b"envoy: reward screen, fight\0", RUN.fight + 1);
                }
            } else if gd_transition_phase() == 0 || gd_transition_kind() != KIND_HOLD {
                tag_frame();
                stage_frame();
                outcome_frame();
            }
            gd_hash_word(RUN.state as u32 ^ (RUN.fight as u32) << 4 ^ (RUN.stats[0] as u32) << 8 ^ (M.tag_cd as u32) << 20 ^ (RW.cursor as u32) << 30);
        } else if !present {
            M.live = false;
        }
        publish();
    }
}

#[no_mangle]
pub extern "C" fn gd_mod_loaded() {
    unsafe {
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
const POST: u32 = 2;
static HOOKS: [HookDecl; 1] = [
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 40, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 4096, hook_count: 1, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.envoy_sim"), mod_name: text(b"Envoy run"),
};
