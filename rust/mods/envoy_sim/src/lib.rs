//! envoy_sim: a single-player Envoy run (docs/rust-port/notes/envoy-1p.md, envoy-drives.md). The sim half: run state,
//! the plan of the next fight, tags, mid-fight stage switches, drives and keystones (envoy-core-v2.md, milestone V1)
//! and the run's end. Everything is this module's linear memory (rolled back with the game) or engine statics
//! (melee_modes::tu::{envoy, tag_swap}); the presentation (envoy_ui) only reads the published view and the fire log.
//!
//! * run: 5 fights against CPUs that ramp up (level and count); fight 5 is the boss (a level-9 Ganondorf, or Petey
//!   with MELEE_ENVOY_BOSS=petey). The pair (P1 = main, P2 = partner on the retail CSS) shares one stock pool.
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown, 60-frame frozen cinematic; each fighter keeps its %.
//! * stage switches: from fight 2 on, every 30 s of play, the next of BF / YS / DL / FoD / FD.
//! * drives (envoy_data): each fighter wears up to 3 (`[2][4]` of id and level 1-3; a second copy levels it up). The
//!   fighter in play charges its drives from hits landed and taken (the take-damage hook), landings (the Landing
//!   enter rows), time and tag-in; a full drive fires on the next free frame, at most one fire per fighter every 20
//!   frames (Relay's volley: 6 frames apart). A fire applies its effect (damage-only bursts, Burning, Chilled, Haste,
//!   an absorbing orb) and appends to the 8-entry fire log the presentation draws and plays from.
//! * keystones: one picked at the run's start (Plague Bearer, Echo Oath, Overclock), each with its price.
//! * earning: the first KO in a fight drops a drive on a floor point (walk over it; still on the floor when the fight
//!   is won, it is swept up), and fights 2-5 open with a pick of three drives and then the fighter that wears it.
//! * MELEE_ENVOY_SHARED_DRIVES=1: one set shared by the pair (Q1's alternative), fixed per run.
//! * determinism: the whole state is one `#[repr(C)]` struct of i32 words; every frame an FNV-1a over all of it, the
//!   plan and the pool digest goes into `gd_hash_word`, so a divergence in any charge shows on the frame it happens.
#![no_std]
#![allow(static_mut_refs)]

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
    fn gd_envoy_percent(slot: i32, bench: i32, delta_milli: i32) -> i32;
    fn gd_envoy_test_drives(out: *mut u8, cap: u32) -> i32;
    fn gd_envoy_shared_drives() -> i32;
    fn gd_envoy_armed() -> i32;
    fn gd_envoy_record(out: *mut i32, cap: u32) -> i32;
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
const BTN_B: u32 = 0x0200;
const STICK_LEFT: u32 = 0x0004_0000;
const STICK_RIGHT: u32 = 0x0008_0000;

const KIND_TAG: i32 = 1;
const KIND_HOLD: i32 = 3;

// gd_envoy_apply `which`
const AP_ATTACK: i32 = 0;
const AP_DEFENSE: i32 = 1;
const AP_STOCKS: i32 = 2;
const AP_SPEED: i32 = 4;
const AP_FIGHTER: [i32; 2] = [0x100, 0x200];

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

const RUN_NONE: i32 = 0;
const RUN_LIVE: i32 = 1;
const RUN_CLEAR: i32 = 2;
const RUN_OVER: i32 = 3;

/// An offer that is not a drive: one more shared stock.
const OFFER_STOCK: i32 = 1000;
/// The reward screens: a pick of three drives, a keystone pick.
const RW_DRIVES: i32 = 1;
const RW_KEYSTONES: i32 = 2;
/// Slot arrays hold 4 (act 2 in the full run); V1 opens 3 per fighter.
const SLOTS: usize = 4;
const OPEN_SLOTS: usize = 3;
const KEYSTONES: usize = 3;
const LOG: usize = 8;
const LOG_W: usize = 8;
const ECHOES: usize = 4;
/// Echo Oath's repeat: frames after the fire.
const ECHO_DELAY: i32 = 12;
/// Effect reach in world units (fighters are about 10-15 tall).
const NOVA_R: f32 = 38.0;
const SHOCK_R: f32 = 32.0;
const SPLASH_R: f32 = 26.0;
/// A floor drop is taken when the fighter in play stands this close (x, and y above the drop).
const PICK_DX: f32 = 11.0;
const PICK_DY: f32 = 22.0;
/// Floor points (x100) a drop lands on: the main floor of every stage in the rotation is at y 0.
const DROP_X: [i32; 4] = [-3500, -1200, 1200, 3500];

// fire log flags
const FL_ECHO: i32 = 1;
const FL_VOLLEY: i32 = 2;
const FL_LINK: i32 = 4;
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
    main: i32,
    partner: i32,
    picks: i32,
    shared: i32,
    /// Worn drives: pool index + 1 (0 empty) and level 1-3; [0] the main's set, [1] the partner's (shared: [0] only).
    drives: [[i32; SLOTS]; 2],
    levels: [[i32; SLOTS]; 2],
    /// Keystones: pool index + 1 (0 empty), the player's.
    keys: [i32; KEYSTONES],
    arm_seen: i32,
    // ---- the fight
    seq: i32,
    live: i32,
    frames: i32,
    /// Unfrozen live frames (the clock every window and charge uses: freezes do not eat them).
    play: i32,
    attrs_done: i32,
    /// The reward screen this fight opens with (0 none, RW_DRIVES, RW_KEYSTONES).
    reward_due: i32,
    outcome: i32, // 0 running, 1 won, 2 lost
    tag_cd: i32,
    stage_clock: i32,
    stage_switches: i32,
    tags: i32,
    last_stocks: i32,
    opened: i32,
    /// The fighter in play: 0 main, 1 partner.
    active: i32,
    kind: [i32; 2],
    prev_phase: i32,
    prev_kind: i32,
    my_stocks: i32,
    foe_stocks: i32,
    foe_each: [i32; 3],
    // charges, per set and slot (time: frames; events: counts), and the foe the last charging hit struck
    charge: [[i32; SLOTS]; 2],
    target: [[i32; SLOTS]; 2],
    last_fire_at: [i32; 2],
    relay_mask: [i32; 2],
    relay_next: [i32; 2],
    /// Echo Oath's repeats: (frame, id + 1, set, level).
    echo: [[i32; 4]; ECHOES],
    /// Bulwark's orb armed (per set); Haste on the fighter (main, partner): until and applied.
    shield: [i32; 2],
    haste_until: [i32; 2],
    haste_on: [i32; 2],
    // foe statuses (slots 1-3)
    burn_until: [i32; 3],
    burn_next: [i32; 3],
    burn_stacks: [i32; 3],
    chill_until: [i32; 3],
    chill_on: [i32; 3],
    atk_set: i32,
    def_set: i32,
    fires: i32,
    hits: i32,
    taken: i32,
    landings: i32,
    // the floor drop: 0 none, 1 on the floor, 2 taken; drive id + 1; floor point x100
    drop_state: i32,
    drop_id: i32,
    drop_x: i32,
    drop_y: i32,
    drops: i32,
    /// The last gain (presentation keys its sound on `gain_n`): frame, id + 1, fighter, level, source.
    gain: [i32; 5],
    gain_n: i32,
    /// The fire log ring: LOG entries of (frame, id + 1, set, target slot, x100, y100, level, flags).
    log: [[i32; LOG_W]; LOG],
    log_n: i32,
    // ---- the reward screen
    rw_active: i32,
    offers: [i32; 3],
    cursor: i32,
    timer: i32,
    /// 0 choosing the card, 1 choosing the fighter.
    step: i32,
    fcursor: i32,
    last_pick: i32,
    last_wearer: i32,
    since_pick: i32,
}

static mut S: St = unsafe { core::mem::zeroed() };
static mut PLAN: [i32; 32] = [0; 32];
const VIEW_WORDS: usize = 256;
static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];

// the run record (melee_modes::tu::envoy REC_*): published in VIEW[REC_AT..], saved by the host, handed back at arm
const REC_WORDS: usize = 32;
const REC_AT: usize = 40;
const REC_MAGIC: i32 = 0x454E_5652;
const REC_VERSION: i32 = 3;
const REC_KEYS: usize = 7;
const REC_DRIVES: usize = 14;
const REC_POOL: usize = 22;

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
    r[2] = S.seed;
    r[3] = S.runs;
    r[4] = S.fight;
    r[5] = S.pool;
    r[6] = S.shared;
    for k in 0..KEYSTONES {
        r[REC_KEYS + k] = S.keys[k];
    }
    r[11] = S.main;
    r[12] = S.partner;
    r[13] = S.picks;
    for f in 0..2 {
        for k in 0..SLOTS {
            r[REC_DRIVES + f * SLOTS + k] = S.drives[f][k] * 4 + S.levels[f][k];
        }
    }
    r[REC_POOL] = POOL_DIGEST as i32;
    r[REC_WORDS - 1] = rec_digest(&r);
    r
}

/// The run on arm: the host's saved record (Continue) or a fresh one (New run: the host forgot the record).
unsafe fn take_arm() {
    let mut r = [0i32; REC_WORDS];
    let n = gd_envoy_record(r.as_mut_ptr(), REC_WORDS as u32);
    let ok = n == REC_WORDS as i32 && r[0] == REC_MAGIC && r[1] == REC_VERSION && r[REC_WORDS - 1] == rec_digest(&r) && (0..FIGHTS as i32).contains(&r[4]);
    if ok && r[REC_POOL] != POOL_DIGEST as i32 {
        log(b"envoy: saved run is from another drive pool, dropped\0", r[REC_POOL]);
        S.state = RUN_NONE;
    } else if ok {
        S.state = RUN_LIVE;
        S.seed = r[2];
        S.runs = r[3];
        S.fight = r[4];
        S.pool = r[5].max(1);
        S.shared = (r[6] != 0) as i32;
        for k in 0..KEYSTONES {
            let id = r[REC_KEYS + k];
            S.keys[k] = if (1..=POOL_LEN as i32).contains(&id) && POOL[id as usize - 1].kind == K_KEYSTONE { id } else { 0 };
        }
        S.main = r[11];
        S.partner = r[12];
        S.picks = r[13];
        for f in 0..2 {
            for k in 0..SLOTS {
                let w = r[REC_DRIVES + f * SLOTS + k];
                let (id, lv) = (w >> 2, w & 3);
                let ok = (1..=POOL_LEN as i32).contains(&id) && POOL[id as usize - 1].kind == K_DRIVE && (1..=MAX_LEVEL).contains(&lv);
                S.drives[f][k] = if ok { id } else { 0 };
                S.levels[f][k] = if ok { lv } else { 0 };
            }
        }
        log(b"envoy: armed, run continued at fight\0", S.fight + 1);
    } else {
        S.state = RUN_NONE;
        log(b"envoy: armed, new run\0", 0);
    }
    S.outcome = 0;
    S.rw_active = 0;
    S.last_pick = -1;
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

fn ncpu() -> i32 {
    unsafe { SCHEDULE[S.fight.clamp(0, FIGHTS as i32 - 1) as usize].ncpu }
}

// ---- drives: who wears what

/// The drive set fighter `f` uses (the pair's one set when drives are shared).
unsafe fn set_of(f: usize) -> usize {
    if S.shared != 0 { 0 } else { f }
}

unsafe fn slot_of(set: usize, id: usize) -> Option<usize> {
    S.drives[set].iter().position(|&d| d == id as i32 + 1)
}

/// Whether fighter `f` can take drive `id`: a level-up below 3, or a free slot.
unsafe fn can_take(f: usize, id: usize) -> bool {
    let set = set_of(f);
    match slot_of(set, id) {
        Some(k) => S.levels[set][k] < MAX_LEVEL,
        None => S.drives[set][..OPEN_SLOTS].contains(&0),
    }
}

/// Fighter `f` gains drive `id`: a level-up, or a new drive in the first free slot. Returns the new level (0: no).
unsafe fn gain(f: usize, id: usize, how: i32) -> i32 {
    let set = set_of(f);
    let lv = match slot_of(set, id) {
        Some(k) => {
            if S.levels[set][k] >= MAX_LEVEL {
                return 0;
            }
            S.levels[set][k] += 1;
            S.levels[set][k]
        }
        None => {
            let Some(k) = S.drives[set][..OPEN_SLOTS].iter().position(|&d| d == 0) else { return 0 };
            S.drives[set][k] = id as i32 + 1;
            S.levels[set][k] = 1;
            S.charge[set][k] = 0;
            1
        }
    };
    S.gain = [S.play, id as i32 + 1, f as i32, lv, how];
    S.gain_n += 1;
    log_drive(if lv > 1 { b"drive level up" } else { b"drive gained" }, id, f as i32 * 10 + lv);
    lv
}

unsafe fn has_key(effect: u8) -> bool {
    S.keys.iter().any(|&k| k > 0 && POOL[k as usize - 1].effect == effect)
}

unsafe fn take_keystone(id: usize) {
    if let Some(k) = S.keys.iter().position(|&k| k == 0) {
        S.keys[k] = id as i32 + 1;
        S.gain = [S.play, id as i32 + 1, S.active, 1, G_KEYSTONE];
        S.gain_n += 1;
        log_drive(b"keystone taken", id, k as i32);
        S.atk_set = -1;
        S.def_set = -1;
    }
}

/// MELEE_ENVOY_TEST_DRIVES="key[:level],key/key|keystone,keystone" (main / partner | keystones): worn from the run's
/// start (tests, demos). A keystone given here skips the run-start keystone pick.
unsafe fn test_drives() {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    let (drv, ks) = s.split_once('|').unwrap_or((s, ""));
    for (f, side) in drv.split('/').take(2).enumerate() {
        for item in side.split(',') {
            let (key, lv) = item.split_once(':').unwrap_or((item, "1"));
            let lv = lv.trim().parse::<i32>().unwrap_or(1).clamp(1, MAX_LEVEL);
            if let Some(id) = by_key(key.trim()).filter(|&i| POOL[i].kind == K_DRIVE) {
                for _ in 0..lv {
                    gain(f, id, G_PICK);
                }
            }
        }
    }
    for key in ks.split(',') {
        if let Some(id) = by_key(key.trim()).filter(|&i| POOL[i].kind == K_KEYSTONE) {
            take_keystone(id);
        }
    }
    S.gain_n = 0;
}

/// The plan of the match the run wants next: the live run's current fight, or a new run's first.
unsafe fn make_plan() {
    if S.arm_seen == 0 {
        PLAN = [0; 32];
        gd_envoy_plan(PLAN.as_ptr(), 32);
        return;
    }
    let fresh = S.state != RUN_LIVE;
    let (fight, pool, seed) = if fresh {
        (gd_envoy_start_fight() as usize, START_POOL, mix((S.runs as u32).wrapping_add(1).wrapping_mul(0x9E37_79B9)))
    } else {
        (S.fight as usize, S.pool, S.seed as u32)
    };
    let f = &SCHEDULE[fight];
    let p = &mut PLAN;
    *p = [0; 32];
    p[0] = 1;
    p[1] = STAGES[fight];
    p[2] = if fresh { -1 } else { S.main };
    p[3] = if fresh { -1 } else { S.partner };
    p[4] = pool.max(1);
    p[5] = 1000; // the ratios are the keystones' (set every frame for the fighter in play)
    p[6] = 1000;
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
    if S.state != RUN_LIVE {
        S.runs = S.runs.wrapping_add(1);
        S.state = RUN_LIVE;
        S.seed = mix((S.runs as u32).wrapping_mul(0x9E37_79B9)) as i32;
        S.fight = gd_envoy_start_fight();
        S.pool = START_POOL;
        S.drives = [[0; SLOTS]; 2];
        S.levels = [[0; SLOTS]; 2];
        S.keys = [0; KEYSTONES];
        S.picks = 0;
        S.shared = (gd_envoy_shared_drives() != 0) as i32;
        S.main = gd_envoy_roster(0);
        S.partner = gd_envoy_roster(1);
        S.last_pick = -1;
        S.attrs_done = 0;
        log(b"envoy: run start, main\0", S.main);
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
    S.reward_due = if S.keys[0] == 0 { RW_KEYSTONES } else if S.fight > 0 { RW_DRIVES } else { 0 };
    S.outcome = 0;
    S.tag_cd = 0;
    S.stage_clock = 0;
    S.stage_switches = 0;
    S.tags = 0;
    S.last_stocks = -1;
    S.opened = 0;
    S.active = 0;
    S.kind = [-1, -1];
    S.prev_phase = 0;
    S.prev_kind = 0;
    S.charge = [[0; SLOTS]; 2];
    S.target = [[0; SLOTS]; 2];
    S.last_fire_at = [-FIRE_GAP; 2];
    S.relay_mask = [0; 2];
    S.relay_next = [0; 2];
    S.echo = [[0; 4]; ECHOES];
    S.shield = [0; 2];
    S.haste_until = [0; 2];
    S.haste_on = [0; 2];
    S.burn_until = [0; 3];
    S.burn_next = [0; 3];
    S.burn_stacks = [0; 3];
    S.chill_until = [0; 3];
    S.chill_on = [0; 3];
    S.atk_set = -1;
    S.def_set = -1;
    S.drop_state = 0;
    S.drop_id = 0;
    S.drops = 0;
    S.rw_active = 0;
    log(b"envoy: fight\0", S.fight + 1);
}

// ---- the reward screens

/// Words the pair already gives or reads (bit per word), for the "one card connects" rule.
unsafe fn pair_words() -> u32 {
    let mut m = 0u32;
    for f in 0..2 {
        for &d in &S.drives[f] {
            if d > 0 {
                let c = &POOL[d as usize - 1];
                m |= (1 << c.gives) | (1 << c.reads);
            }
        }
    }
    for &k in &S.keys {
        if k > 0 {
            let c = &POOL[k as usize - 1];
            m |= (1 << c.gives) | (1 << c.reads);
        }
    }
    m & !1
}

unsafe fn offer_ok(c: usize, keys: bool) -> bool {
    if keys {
        POOL[c].kind == K_KEYSTONE && !S.keys.contains(&(c as i32 + 1))
    } else {
        POOL[c].kind == K_DRIVE && (can_take(0, c) || (S.shared == 0 && can_take(1, c)))
    }
}

unsafe fn connects(c: i32, words: u32) -> bool {
    c != OFFER_STOCK && words & ((1 << POOL[c as usize].gives) | (1 << POOL[c as usize].reads)) & !1 != 0
}

unsafe fn roll_offers() {
    let mut h = mix(S.seed as u32 ^ 0xA5A5_0000 ^ (S.fight as u32) << 4 ^ S.picks as u32);
    S.offers = [OFFER_STOCK; 3];
    let mut n = 0;
    let keys = S.rw_active == RW_KEYSTONES;
    let mut tries = 0;
    while n < 3 && tries < 400 {
        tries += 1;
        let c = (h % POOL_LEN as u32) as usize;
        h = mix(h.wrapping_add(0x6D2B_79F5));
        if !offer_ok(c, keys) || S.offers[..n].contains(&(c as i32)) {
            continue;
        }
        S.offers[n] = c as i32;
        n += 1;
    }
    if !keys {
        // at least one card connects to a word the pair already gives or reads (when one can)
        let words = pair_words();
        if words != 0 && n == 3 && !S.offers.iter().any(|&c| connects(c, words)) {
            for k in 0..POOL_LEN {
                let c = (k + (h as usize % POOL_LEN)) % POOL_LEN;
                if offer_ok(c, false) && !S.offers.contains(&(c as i32)) && connects(c as i32, words) {
                    S.offers[2] = c as i32;
                    break;
                }
            }
        }
        if gd_player_stocks(0) <= 1 {
            S.offers[2] = OFFER_STOCK;
        }
    }
}

/// The default fighter for a drive card: the one in play when it can take it, else the other.
unsafe fn default_fighter(id: i32) -> i32 {
    if id == OFFER_STOCK || S.shared != 0 || !(0..POOL_LEN as i32).contains(&id) {
        return 0;
    }
    let a = S.active as usize;
    if can_take(a, id as usize) { a as i32 } else { 1 - a as i32 }
}

unsafe fn finish_pick(id: i32, f: i32) {
    if id == OFFER_STOCK {
        gd_envoy_apply(0, AP_STOCKS, gd_player_stocks(0) + 1);
        log(b"envoy: took +1 stock\0", gd_player_stocks(0));
    } else if (0..POOL_LEN as i32).contains(&id) {
        if POOL[id as usize].kind == K_KEYSTONE {
            take_keystone(id as usize);
        } else {
            gain(f.clamp(0, 1) as usize, id as usize, G_PICK);
        }
    }
    S.picks += 1;
    S.last_pick = id;
    S.last_wearer = f;
    S.since_pick = 0;
    // a keystone pick at a later fight's start is followed by that fight's drive pick
    if S.rw_active == RW_KEYSTONES && S.fight > 0 {
        S.rw_active = RW_DRIVES;
        S.cursor = 0;
        S.step = 0;
        S.timer = REWARD_TIME;
        roll_offers();
        log(b"envoy: drive offer, fight\0", S.fight + 1);
        return;
    }
    S.rw_active = 0;
    gd_hold_release();
}

unsafe fn reward_frame() {
    let pad = gd_pad_pressed(0);
    let left = pad & (DPAD_LEFT | STICK_LEFT) != 0;
    let right = pad & (DPAD_RIGHT | STICK_RIGHT) != 0;
    S.timer -= 1;
    if S.step == 0 {
        if left {
            S.cursor = (S.cursor + 2) % 3;
        }
        if right {
            S.cursor = (S.cursor + 1) % 3;
        }
        let id = S.offers[S.cursor.clamp(0, 2) as usize];
        let one_fighter = id == OFFER_STOCK || S.rw_active == RW_KEYSTONES || S.shared != 0 || gd_tag_partner(0) < 0;
        if pad & BTN_A != 0 {
            if one_fighter {
                finish_pick(id, default_fighter(id));
            } else {
                S.step = 1;
                S.fcursor = default_fighter(id);
            }
        } else if S.timer <= 0 {
            let id = S.offers[0];
            finish_pick(id, default_fighter(id));
        }
        return;
    }
    // step 1: which fighter wears it
    let id = S.offers[S.cursor.clamp(0, 2) as usize];
    if left || right {
        S.fcursor = 1 - S.fcursor;
    }
    if pad & BTN_B != 0 {
        S.step = 0;
    } else if pad & BTN_A != 0 && can_take(S.fcursor as usize, id as usize) {
        finish_pick(id, S.fcursor);
    } else if S.timer <= 0 {
        finish_pick(id, default_fighter(id));
    }
}

// ---- positions

#[derive(Clone, Copy)]
struct P {
    x: f32,
    y: f32,
}

unsafe fn pos(slot: i32) -> Option<P> {
    let h = gd_fighter_from_slot(slot);
    if h == 0 {
        return None;
    }
    Some(P { x: gd_fighter_get_pos_x(h), y: gd_fighter_get_pos_y(h) })
}

unsafe fn foe_alive(s: i32) -> bool {
    gd_player_present(s) != 0 && gd_player_stocks(s) > 0
}

fn d2(a: P, b: P) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    dx * dx + dy * dy
}

/// The live foe slot nearest `at`, other than `skip` (0: none).
unsafe fn nearest(at: P, skip: i32) -> i32 {
    let mut best = 0;
    let mut bd = f32::MAX;
    for s in 1..=ncpu() {
        if s == skip || !foe_alive(s) {
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

// ---- statuses

unsafe fn burn(s: i32, stacks_to: i32, add: bool) {
    if s < 1 {
        return;
    }
    let i = s as usize - 1;
    if S.burn_until[i] <= S.play {
        S.burn_next[i] = S.play + BURN_TICK;
        S.burn_stacks[i] = 0;
    }
    S.burn_stacks[i] = if add { (S.burn_stacks[i] + 1).min(stacks_to) } else { S.burn_stacks[i].max(1) };
    S.burn_until[i] = S.burn_until[i].max(S.play + BURN_FRAMES);
}

unsafe fn chill(s: i32, frames: i32) {
    if s < 1 {
        return;
    }
    let i = s as usize - 1;
    if S.chill_on[i] == 0 {
        gd_envoy_apply(s, AP_SPEED, CHILL_SPEED);
        S.chill_on[i] = 1;
    }
    S.chill_until[i] = S.chill_until[i].max(S.play + frames);
}

unsafe fn unchill(i: usize) {
    if S.chill_on[i] != 0 {
        gd_envoy_apply(i as i32 + 1, AP_SPEED, 1_000_000 / CHILL_SPEED);
        S.chill_on[i] = 0;
    }
    S.chill_until[i] = 0;
}

unsafe fn chilled(s: i32) -> bool {
    s >= 1 && S.chill_on[s as usize - 1] != 0
}

unsafe fn haste(f: usize, frames: i32) {
    if S.haste_on[f] == 0 {
        gd_envoy_apply(0, AP_SPEED | AP_FIGHTER[f], HASTE_SPEED);
        S.haste_on[f] = 1;
    }
    S.haste_until[f] = S.haste_until[f].max(S.play + frames);
}

unsafe fn unhaste(f: usize) {
    if S.haste_on[f] != 0 {
        gd_envoy_apply(0, AP_SPEED | AP_FIGHTER[f], 1_000_000 / HASTE_SPEED);
        S.haste_on[f] = 0;
    }
    S.haste_until[f] = 0;
}

unsafe fn hurt(s: i32, milli: i32) {
    if s >= 1 && milli > 0 && foe_alive(s) {
        gd_envoy_percent(s, 0, milli);
    }
}

// ---- firing

/// Fire drive `id` of `set` at `level` (`frac` x1000 of its number; `target`: the foe its charge last struck).
unsafe fn fire(set: usize, id: usize, level: i32, target: i32, frac: i32, flags: i32) {
    let c = &POOL[id];
    let n = scaled(c.number, level) * frac / 1000;
    let me = pos(0).unwrap_or(P { x: 0.0, y: 0.0 });
    let mut at = me;
    let mut tgt = 0;
    let mut fl = flags;
    let near = |s: i32| if s >= 1 && foe_alive(s) { s } else { nearest(me, 0) };
    match c.effect {
        E_BURST => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            burn(tgt, BURN_STACKS, false);
            if level >= MAX_LEVEL {
                for s in 1..=ncpu() {
                    if s != tgt && foe_alive(s) && pos(s).is_some_and(|p| d2(p, at) < SPLASH_R * SPLASH_R) {
                        hurt(s, n);
                        burn(s, BURN_STACKS, false);
                    }
                }
            }
        }
        E_NOVA | E_SHOCKWAVE => {
            let r = if c.effect == E_NOVA { NOVA_R } else { SHOCK_R };
            for s in 1..=ncpu() {
                if foe_alive(s) && pos(s).is_some_and(|p| d2(p, me) < r * r) {
                    hurt(s, n);
                    if tgt == 0 {
                        tgt = s;
                    }
                }
            }
        }
        E_BOLT => {
            tgt = nearest(me, 0);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            if level >= MAX_LEVEL {
                hurt(nearest(at, tgt), n);
            }
        }
        E_HASTE => {
            let f = if S.shared != 0 { S.active as usize } else { set };
            haste(f, n * 60 / 1000);
            tgt = -1;
        }
        E_SHIELD => {
            S.shield[set] = 1;
            tgt = -1;
        }
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
            if tgt >= 1 {
                unchill(tgt as usize - 1);
            }
            fl |= FL_LINK;
        }
        E_RELAY => {
            // every other drive of this set fires, 6 frames apart (exempt from the 20-frame rule)
            let mut m = 0;
            for k in 0..SLOTS {
                let d = S.drives[set][k];
                if d > 0 && POOL[d as usize - 1].effect != E_RELAY {
                    m |= 1 << k;
                }
            }
            S.relay_mask[set] = m;
            S.relay_next[set] = S.play + RELAY_GAP;
            tgt = -1;
        }
        _ => {}
    }
    S.fires += 1;
    let e = &mut S.log[S.log_n as usize % LOG];
    *e = [S.play, id as i32 + 1, set as i32, tgt, x100(at.x), x100(at.y), level, fl];
    S.log_n += 1;
    log_drive(if fl & FL_ECHO != 0 { b"drive echo" } else { b"drive fire" }, id, set as i32 * 10 + level);
    if has_key(E_ECHO) && fl & FL_ECHO == 0 && c.effect != E_RELAY {
        if let Some(q) = S.echo.iter_mut().find(|q| q[1] == 0) {
            *q = [S.play + ECHO_DELAY, id as i32 + 1, set as i32, level];
        }
    }
    if has_key(E_OVERCLOCK) {
        // Bleed: each fire costs the fighter in play 1%
        gd_envoy_percent(0, 0, 1000);
    }
}

unsafe fn fire_slot(set: usize, k: usize, flags: i32) {
    let d = S.drives[set][k];
    if d <= 0 {
        return;
    }
    S.charge[set][k] = 0;
    fire(set, d as usize - 1, S.levels[set][k], S.target[set][k], 1000, flags);
}

/// What a full charge is, in the drive's own units (time: frames).
fn need(c: &Drive) -> i32 {
    if c.charge == C_TIME { c.need * 60 / 1000 } else { c.need }
}

unsafe fn charge_mult() -> i32 {
    if has_key(E_OVERCLOCK) { 2 } else { 1 }
}

/// Charge every drive of the fighter in play keyed on `src` (target: the foe slot the event struck).
unsafe fn charge(src: u8, target: i32) {
    let set = set_of(S.active as usize);
    let m = charge_mult();
    for k in 0..SLOTS {
        let d = S.drives[set][k];
        if d <= 0 {
            continue;
        }
        let c = &POOL[d as usize - 1];
        if c.charge != src || (c.reads == W_CHILLED && !chilled(target)) {
            continue;
        }
        S.charge[set][k] = (S.charge[set][k] + m).min(need(c));
        if target >= 1 {
            S.target[set][k] = target;
        }
    }
}

/// Per unfrozen live frame: time charges, the fire queue, Relay's volley, the echoes.
unsafe fn drives_frame() {
    let set = set_of(S.active as usize);
    charge(C_TIME, 0);
    // the queue: one fire per fighter every FIRE_GAP frames, slot order
    if S.play - S.last_fire_at[set] >= FIRE_GAP {
        for k in 0..SLOTS {
            let d = S.drives[set][k];
            if d > 0 && S.charge[set][k] >= need(&POOL[d as usize - 1]) {
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
    for q in 0..ECHOES {
        let e = S.echo[q];
        if e[1] > 0 && S.play >= e[0] {
            S.echo[q] = [0; 4];
            let half = by_key("echo_oath").map_or(500, |i| POOL[i].number);
            fire(e[2] as usize, e[1] as usize - 1, e[3], 0, half, FL_ECHO);
        }
    }
}

// ---- the take-damage and landing hooks (run inside the sim's frame: they re-run in a resimulation)

unsafe fn in_play() -> bool {
    S.live != 0 && S.state == RUN_LIVE && S.rw_active == 0 && S.outcome == 0 && S.attrs_done != 0
}

/// damage.fighter_take_damage pre: Bulwark's orb absorbs the hit's damage on the fighter in play.
extern "C" fn on_damage_pre(h: u32, d: f32) -> i32 {
    unsafe {
        if !in_play() || d <= 0.0 || gd_fighter_slot(h) != 0 {
            return 0;
        }
        let set = set_of(S.active as usize);
        if S.shield[set] != 0 {
            S.shield[set] = 0;
            log(b"envoy: orb absorbed a hit (x100)\0", x100(d));
            return 1; // GD_SKIP: no percent this hit
        }
    }
    0
}

/// damage.fighter_take_damage post: a hit landed on a foe or taken by the fighter in play.
extern "C" fn on_damage(h: u32, d: f32) {
    unsafe {
        if !in_play() || d <= 0.0 {
            return;
        }
        let s = gd_fighter_slot(h);
        if s == 0 {
            S.taken += 1;
            charge(C_TAKEN, 0);
        } else if (1..=ncpu()).contains(&s) {
            S.hits += 1;
            if has_key(E_PLAGUE) {
                burn(s, BURN_STACKS, true);
            }
            charge(C_HIT, s);
        }
    }
}

unsafe fn landed(h: u32) {
    if in_play() && gd_fighter_slot(h) == 0 {
        S.landings += 1;
        charge(C_LANDING, 0);
    }
}

extern "C" fn on_landing(h: u32, _msid: i32, _interrupt: i32, _flags: i32, _a: f32, _b: f32) -> i32 {
    unsafe { landed(h) };
    0
}

extern "C" fn on_landing_special(h: u32, _a: i32, _b: f32) -> i32 {
    unsafe { landed(h) };
    0
}

// ---- moments

unsafe fn on_tag() {
    let out = S.active as usize;
    let inn = 1 - out;
    S.active = inn as i32;
    unhaste(out);
    log(b"envoy: moment tag in, fighter\0", inn as i32);
    charge(C_TAG_IN, 0);
}

/// Per unfrozen live frame: the opener, the stock edges, the drop, the statuses.
unsafe fn moments_frame() {
    if S.opened == 0 {
        S.opened = 1;
        log(b"envoy: moment opener, fight\0", S.fight + 1);
    }
    let me = gd_player_stocks(0);
    let mut foes = 0;
    for s in 1..=ncpu() {
        let n = gd_player_stocks(s).max(0);
        foes += n;
        let i = s as usize - 1;
        if n < S.foe_each[i] {
            // a KO: statuses end (the respawn starts clean)
            unchill(i);
            S.burn_until[i] = 0;
            S.burn_stacks[i] = 0;
        }
        S.foe_each[i] = n;
    }
    if me < S.my_stocks && me > 0 {
        log(b"envoy: moment stock lost, pool\0", me);
        unhaste(S.active as usize);
    }
    if foes < S.foe_stocks {
        log(b"envoy: moment KO dealt, foes left\0", foes);
        if S.drops == 0 && S.drop_state == 0 {
            roll_drop();
        }
    }
    S.my_stocks = me;
    S.foe_stocks = foes;
    drop_frame();
    // Burning: 2% a second per stack; Chilled and Haste end
    for i in 0..3 {
        if S.burn_stacks[i] > 0 && S.play >= S.burn_until[i] {
            S.burn_stacks[i] = 0;
        }
        if S.burn_stacks[i] > 0 && S.play >= S.burn_next[i] {
            S.burn_next[i] += BURN_TICK;
            hurt(i as i32 + 1, BURN_TICK_PCT * S.burn_stacks[i]);
        }
        if S.chill_on[i] != 0 && S.play >= S.chill_until[i] {
            unchill(i);
        }
    }
    for f in 0..2 {
        if S.haste_on[f] != 0 && S.play >= S.haste_until[f] {
            unhaste(f);
        }
    }
}

/// The first KO's drop: a drive the fighter in play can take (else its partner), on a floor point.
unsafe fn roll_drop() {
    let mut h = mix(S.seed as u32 ^ 0xD20F_0000 ^ (S.fight as u32) << 8 ^ S.drops as u32);
    let a = S.active as usize;
    for pass in 0..2 {
        let f = if pass == 0 { a } else { 1 - a };
        for _ in 0..64 {
            let c = (h % POOL_LEN as u32) as usize;
            h = mix(h.wrapping_add(0x6D2B_79F5));
            if POOL[c].kind == K_DRIVE && can_take(f, c) {
                S.drop_state = 1;
                S.drop_id = c as i32 + 1;
                S.drop_x = DROP_X[(h >> 5) as usize % DROP_X.len()];
                S.drop_y = 0;
                S.drops += 1;
                log_drive(b"drop", c, S.drop_x / 100);
                return;
            }
        }
    }
}

unsafe fn drop_frame() {
    if S.drop_state != 1 {
        return;
    }
    let Some(p) = pos(0) else { return };
    let (dx, dy) = (p.x - S.drop_x as f32 / 100.0, p.y - S.drop_y as f32 / 100.0);
    if dx.abs() < PICK_DX && dy > -6.0 && dy < PICK_DY {
        take_drop(G_DROP);
    }
}

unsafe fn take_drop(how: i32) {
    let id = S.drop_id as usize - 1;
    let a = S.active as usize;
    let f = if can_take(a, id) { a } else { 1 - a };
    S.drop_state = 2;
    if gain(f, id, how) == 0 {
        log_drive(b"drop lost (no room)", id, 0);
    }
}

/// The ratios of the fighter in play: keystone prices (Fragile, Weak); written only when they change.
unsafe fn ratios_frame() {
    let mut atk = 1000;
    let mut def = 1000;
    for &k in &S.keys {
        if k <= 0 {
            continue;
        }
        match POOL[k as usize - 1].price {
            P_FRAGILE => def = def * 1250 / 1000,
            P_WEAK => atk = atk * 800 / 1000,
            _ => {}
        }
    }
    if atk != S.atk_set {
        S.atk_set = atk;
        gd_envoy_apply(0, AP_ATTACK, atk);
    }
    if def != S.def_set {
        S.def_set = def;
        gd_envoy_apply(0, AP_DEFENSE, def);
    }
}

/// The first live frame: the pair's kinds and the stock baselines.
unsafe fn fight_setup() {
    S.attrs_done = 1;
    let h = gd_fighter_from_slot(0);
    S.kind = [if h != 0 { gd_fighter_kind(h) } else { -1 }, gd_tag_partner(0)];
    S.my_stocks = gd_player_stocks(0);
    S.foe_stocks = 0;
    for s in 1..=ncpu() {
        S.foe_each[s as usize - 1] = gd_player_stocks(s).max(0);
        S.foe_stocks += S.foe_each[s as usize - 1];
    }
}

/// Which fighter is in: by kind when the pair differ (a respawn, a refused swap), else the tag count's parity.
unsafe fn track_active() {
    if S.kind[0] < 0 || S.kind[1] < 0 || S.kind[0] == S.kind[1] {
        return;
    }
    let h = gd_fighter_from_slot(0);
    if h == 0 {
        return;
    }
    let k = gd_fighter_kind(h);
    if k == S.kind[0] {
        S.active = 0;
    } else if k == S.kind[1] {
        S.active = 1;
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
    if S.tag_cd > 0 {
        S.tag_cd -= 1;
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
        S.tag_cd = TAG_COOLDOWN;
        S.tags += 1;
        log(b"envoy: tag\0", S.tags);
    }
}

unsafe fn stage_frame() {
    if S.fight < 1 || gd_transition_phase() != 0 {
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
    } else if rc != -9 {
        S.stage_clock = 0;
        log(b"envoy: stage transition refused\0", rc);
    }
}

unsafe fn outcome_frame() {
    if S.outcome != 0 {
        return;
    }
    let me = gd_player_stocks(0);
    let mut foes = 0;
    for s in 1..=ncpu() {
        foes += gd_player_stocks(s).max(0);
    }
    let key = me * 100 + foes;
    if key != S.last_stocks {
        S.last_stocks = key;
        log(b"envoy: stocks (pool*100 + foes)\0", key);
    }
    if foes == 0 && me > 0 {
        if S.drop_state == 1 {
            // the fight is over with the drop still on the floor: it is swept up, never lost
            take_drop(G_SWEPT);
        }
        S.outcome = 1;
        S.pool = me;
        if S.fight as usize == FIGHTS - 1 {
            S.state = RUN_CLEAR;
            log(b"envoy: RUN CLEAR, pool left\0", me);
        } else {
            S.fight += 1;
            log(b"envoy: fight won, next\0", S.fight + 1);
        }
        make_plan();
    } else if me <= 0 {
        S.outcome = 2;
        S.pool = 0;
        S.state = RUN_OVER;
        log(b"envoy: RUN OVER at fight\0", S.fight + 1);
        make_plan();
    }
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

// View layout (envoy_ui; docs/rust-port/notes/envoy-drives.md): 0 magic, 1 state, 2 fight, 3 fights, 4 pool, 5 live,
// 6 main, 7 partner, 8 tag cd, 9 tag cd max, 10 reward screen (1 drives, 2 keystones), 11 cursor, 12 timer, 13-15
// offers (pool index or OFFER_STOCK), 16 step (0 card, 1 fighter), 17 fighter cursor, 18 shared, 19 stocks now, 20
// open slots, 22-29 drives (id + 1; main set 22-25, partner 26-29), 30 last pick, 31 frames since, 32 outcome, 33
// offer time, 34 partner kind, 35 shifts, 36 seq, 37 armed, 38 fighter in play, 39 last pick's fighter, 40-71 the
// record, 72 fires, 73 play frame, 74 frozen, 75 pool digest, 76 stage countdown, 77-84 levels, 85-92 charge
// permille, 93-95 keystones (id + 1), 96-97 fighter in play x / y (f32 bits), 98 its present, 99-104 foes x / y (f32
// bits), 105 cpus, 106-108 Burning stacks, 109-111 Chilled frames left, 112-113 Haste frames left (main, partner),
// 114-115 orb armed (per set), 116 drop state, 117 drop id + 1, 118-119 drop x / y (x100), 120-124 the last gain
// (frame, id + 1, fighter, level, source), 125 gains, 126 fire log count, 127 can-take bits (offer i, fighter f: bit
// i * 2 + f), 128-191 the fire log (8 x frame, id + 1, set, target, x100, y100, level, flags), 192-194 foes alive.
unsafe fn publish() {
    let v = &mut VIEW;
    *v = [0; VIEW_WORDS];
    v[0] = 0x454E_5659;
    v[1] = S.state;
    v[2] = S.fight;
    v[3] = FIGHTS as i32;
    v[4] = if S.live != 0 { gd_player_stocks(0) } else { S.pool };
    v[5] = S.live;
    v[6] = S.main;
    v[7] = S.partner;
    v[8] = S.tag_cd;
    v[9] = TAG_COOLDOWN;
    v[10] = S.rw_active;
    v[11] = S.cursor;
    v[12] = S.timer;
    let mut bits = 0;
    for i in 0..3 {
        v[13 + i] = S.offers[i];
        let id = S.offers[i];
        if (0..POOL_LEN as i32).contains(&id) && POOL[id as usize].kind == K_DRIVE {
            for f in 0..2 {
                if can_take(f, id as usize) {
                    bits |= 1 << (i * 2 + f);
                }
            }
        }
    }
    v[127] = bits;
    v[16] = S.step;
    v[17] = S.fcursor;
    v[18] = S.shared;
    v[19] = gd_player_stocks(0);
    v[20] = OPEN_SLOTS as i32;
    for f in 0..2 {
        for k in 0..SLOTS {
            v[22 + f * SLOTS + k] = S.drives[f][k];
            v[77 + f * SLOTS + k] = S.levels[f][k];
            let d = S.drives[f][k];
            v[85 + f * SLOTS + k] = if d > 0 { S.charge[f][k] * 1000 / need(&POOL[d as usize - 1]).max(1) } else { 0 };
        }
    }
    v[30] = S.last_pick;
    v[31] = S.since_pick;
    v[32] = S.outcome;
    v[33] = REWARD_TIME;
    v[34] = gd_tag_partner(0);
    v[35] = S.stage_switches;
    v[36] = gd_envoy_seq();
    v[37] = S.arm_seen;
    v[38] = S.active;
    v[39] = S.last_wearer;
    let r = record();
    v[REC_AT..REC_AT + REC_WORDS].copy_from_slice(&r);
    v[72] = S.fires;
    v[73] = S.play;
    v[74] = (gd_transition_phase() != 0 || S.rw_active != 0) as i32;
    v[75] = POOL_DIGEST as i32;
    v[76] = if S.fight >= 1 && S.live != 0 { STAGE_PERIOD - S.stage_clock } else { -1 };
    for k in 0..KEYSTONES {
        v[93 + k] = S.keys[k];
    }
    if let Some(p) = pos(0) {
        v[96] = fbits(p.x);
        v[97] = fbits(p.y);
        v[98] = 1;
    }
    for s in 1..=3 {
        let i = s as usize - 1;
        if let Some(p) = pos(s) {
            v[99 + i * 2] = fbits(p.x);
            v[100 + i * 2] = fbits(p.y);
        }
        v[106 + i] = if S.burn_until[i] > S.play { S.burn_stacks[i] } else { 0 };
        v[109 + i] = if S.chill_on[i] != 0 { (S.chill_until[i] - S.play).max(0) } else { 0 };
        v[192 + i] = (s <= ncpu() && foe_alive(s)) as i32;
    }
    v[105] = ncpu();
    for f in 0..2 {
        v[112 + f] = if S.haste_on[f] != 0 { (S.haste_until[f] - S.play).max(0) } else { 0 };
        v[114 + f] = S.shield[f];
    }
    v[116] = S.drop_state;
    v[117] = S.drop_id;
    v[118] = S.drop_x;
    v[119] = S.drop_y;
    v[120..125].copy_from_slice(&S.gain);
    v[125] = S.gain_n;
    v[126] = S.log_n;
    for e in 0..LOG {
        v[128 + e * LOG_W..128 + (e + 1) * LOG_W].copy_from_slice(&S.log[e]);
    }
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
            if S.prev_phase == 1 && phase == 0 && S.prev_kind == KIND_TAG {
                on_tag();
            }
            S.prev_phase = phase;
            S.prev_kind = kind;
            track_active();
            if S.rw_active != 0 {
                reward_frame();
            } else if S.reward_due != 0 && S.frames >= REWARD_DELAY && phase == 0 && S.outcome == 0 {
                if gd_hold_begin(REWARD_TIME * 2 + 120) == 0 {
                    S.rw_active = S.reward_due;
                    S.reward_due = 0;
                    S.cursor = 0;
                    S.step = 0;
                    S.timer = REWARD_TIME;
                    roll_offers();
                    log(if S.rw_active == RW_KEYSTONES { b"envoy: keystone offer, fight\0" } else { b"envoy: drive offer, fight\0" }, S.fight + 1);
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
                ratios_frame();
            }
        } else if !present {
            S.live = 0;
        }
        gd_hash_word(digest());
        publish();
    }
}

#[no_mangle]
pub extern "C" fn gd_mod_loaded() {
    unsafe {
        S.seed = 1;
        S.pool = START_POOL;
        S.main = -1;
        S.partner = -1;
        S.last_pick = -1;
        S.last_wearer = -1;
        S.since_pick = 9999;
        S.last_stocks = -1;
        S.offers = [0, 1, 2];
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
static HOOKS: [HookDecl; 5] = [
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 40, flags: 0, f: on_frame_end as *const () },
    HookDecl { address: 0x8006_CC7C, mode: PRE, priority: 40, flags: 0, f: on_damage_pre as *const () },
    HookDecl { address: 0x8006_CC7C, mode: POST, priority: 40, flags: 0, f: on_damage as *const () },
    HookDecl { address: 0x800D_5AEC, mode: PRE, priority: 40, flags: 0, f: on_landing as *const () },
    HookDecl { address: 0x800D_5CB0, mode: PRE, priority: 40, flags: 0, f: on_landing_special as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0003_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 4096, hook_count: 5, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.envoy_sim"), mod_name: text(b"Envoy run"),
};
