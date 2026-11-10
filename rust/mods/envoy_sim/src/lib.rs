//! envoy_sim: a single-player Envoy run (docs/rust-port/notes/envoy-1p.md, envoy-crests.md). The sim half: run state,
//! the plan of the next fight, tags, mid-fight stage switches, crests (offers, wearers, moments, the four words) and
//! the run's end. Everything is this module's linear memory (rolled back with the game) or engine statics
//! (melee_modes::tu::{envoy, tag_swap}); the presentation (envoy_ui) only reads the published view.
//!
//! * run: 5 fights against CPUs that ramp up (level and count); fight 5 is the boss stand-in (a level-9 Ganondorf with
//!   damage, defense and size buffs). The player's pair (P1 = main, P2 = partner on the retail CSS) shares one stock
//!   pool across the run; a fight is won when every CPU is out of stocks, the run is over when the pool is empty.
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown, 60-frame frozen cinematic; each fighter keeps its %.
//! * stage switches: from fight 2 on, every 30 s of play, the next of BF / YS / DL / FoD / FD, under a 90-frame
//!   frozen wipe-and-flash.
//! * crests (envoy_data, full-game-plan.md 4.2): at the start of fights 2-5 the sim freezes and offers 3 crests from
//!   3 different families, each naming its wearer (main or partner); `+1 stock` replaces the third when the pool is
//!   at 1 (D-pad / stick left-right, A takes; 20 s, then offer 0). A crest fires at its moment (tag in / out, shift,
//!   stock lost, KO dealt, opener) or holds during its window (always, Fresh, Benched, last stock). Not drawn in
//!   fights (redesign Q7): the moments are logged (`envoy: crest fire <key>`) and counted in the view.
//! * determinism: the whole state is one `#[repr(C)]` struct of i32 words; every frame an FNV-1a over all of it and
//!   the pool digest goes into `gd_hash_word`, so a divergence anywhere (an offer, a window end) is caught on the
//!   frame it happens.
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
    fn gd_envoy_test_crests(out: *mut u8, cap: u32) -> i32;
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
/// Charge: benched this long before the tag-in.
const CHARGE_BENCH: i32 = 600;
/// Rebirth: the window counts from the KO, so it also covers the respawn platform.
const RESPAWN_FRAMES: i32 = 90;
/// Handoff: the partner's boost window.
const HANDOFF_FRAMES: i32 = 300;

const DPAD_LEFT: u32 = 0x0001;
const DPAD_RIGHT: u32 = 0x0002;
const DPAD_DOWN: u32 = 0x0004;
const BTN_A: u32 = 0x0100;
const STICK_LEFT: u32 = 0x0004_0000;
const STICK_RIGHT: u32 = 0x0008_0000;

const KIND_TAG: i32 = 1;
const KIND_STAGE: i32 = 2;
const KIND_HOLD: i32 = 3;

// gd_envoy_apply `which`
const AP_ATTACK: i32 = 0;
const AP_DEFENSE: i32 = 1;
const AP_STOCKS: i32 = 2;
const AP_WEIGHT: i32 = 3;
const AP_SPEED: i32 = 4;
const AP_JUMP: i32 = 5;
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

/// An offer that is not a crest: one more shared stock.
const OFFER_STOCK: i32 = 1000;
const SLOTS: usize = 3;

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
    /// Worn crests: pool index + 1 (0 empty); [0] the main, [1] the partner.
    crests: [[i32; SLOTS]; 2],
    arm_seen: i32,
    // ---- the fight
    seq: i32,
    live: i32,
    frames: i32,
    /// Unfrozen live frames (the clock every window uses: freezes do not eat windows).
    play: i32,
    attrs_done: i32,
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
    /// Fighter kinds of the main and the partner (to tell which one is in after a respawn).
    kind: [i32; 2],
    prev_phase: i32,
    prev_kind: i32,
    my_stocks: i32,
    foe_stocks: i32,
    fresh_until: [i32; 2],
    bench_since: [i32; 2],
    boost_until: i32,
    boost_to: i32,
    boost: i32,
    marked_until: [i32; 3],
    burn_until: [i32; 3],
    burn_next: [i32; 3],
    atk_set: i32,
    def_set: i32,
    foe_def_set: [i32; 3],
    foe_def_base: [i32; 3],
    fires: i32,
    last_fire: i32,
    last_fire_at: i32,
    // ---- the crest offer
    rw_active: i32,
    offers: [i32; 3],
    wearers: [i32; 3],
    cursor: i32,
    timer: i32,
    last_pick: i32,
    last_wearer: i32,
    since_pick: i32,
}

static mut S: St = unsafe { core::mem::zeroed() };
static mut PLAN: [i32; 32] = [0; 32];
const VIEW_WORDS: usize = 160;
static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];

// the run record (melee_modes::tu::envoy REC_*): published in VIEW[REC_AT..], saved by the host, handed back at arm
const REC_WORDS: usize = 32;
const REC_AT: usize = 40;
const REC_MAGIC: i32 = 0x454E_5652;
const REC_VERSION: i32 = 2;
const REC_CRESTS: usize = 14;
const REC_POOL: usize = 20;

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
    r[11] = S.main;
    r[12] = S.partner;
    r[13] = S.picks;
    for f in 0..2 {
        for k in 0..SLOTS {
            r[REC_CRESTS + f * SLOTS + k] = S.crests[f][k];
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
        // a record from another crest pool: its ids mean other crests
        log(b"envoy: saved run is from another crest pool, dropped\0", r[REC_POOL]);
        S.state = RUN_NONE;
    } else if ok {
        S.state = RUN_LIVE;
        S.seed = r[2];
        S.runs = r[3];
        S.fight = r[4];
        S.pool = r[5].max(1);
        S.main = r[11];
        S.partner = r[12];
        S.picks = r[13];
        for f in 0..2 {
            for k in 0..SLOTS {
                let c = r[REC_CRESTS + f * SLOTS + k];
                S.crests[f][k] = if (1..=POOL_LEN as i32).contains(&c) { c } else { 0 };
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

/// `envoy: <what> <crest key>` (value: the wearer, 0 main / 1 partner).
fn log_crest(what: &[u8], id: usize, v: i32) {
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

// ---- crests: who wears what

unsafe fn wears(f: usize, id: usize) -> bool {
    S.crests[f].contains(&(id as i32 + 1))
}

unsafe fn worn(id: usize) -> bool {
    wears(0, id) || wears(1, id)
}

unsafe fn count(f: usize) -> usize {
    S.crests[f].iter().filter(|&&c| c != 0).count()
}

/// Calls `each(fighter, id)` for every worn crest, main first.
unsafe fn each_worn(mut each: impl FnMut(usize, usize)) {
    for f in 0..2 {
        for k in 0..SLOTS {
            let c = S.crests[f][k];
            if c > 0 {
                each(f, c as usize - 1);
            }
        }
    }
}

fn secs(milli: i32) -> i32 {
    milli * 60 / 1000
}

unsafe fn fire(id: usize, f: usize) {
    S.fires += 1;
    S.last_fire = id as i32;
    S.last_fire_at = S.play;
    log_crest(b"crest fire", id, f as i32);
}

/// Wear crest `id` on fighter `f` (the pick, the test knob). ALWAYS attribute crests apply at once.
unsafe fn wear(f: usize, id: usize) -> bool {
    if worn(id) {
        return false;
    }
    let Some(k) = S.crests[f].iter().position(|&c| c == 0) else { return false };
    S.crests[f][k] = id as i32 + 1;
    if S.attrs_done != 0 {
        apply_attr(f, id);
    }
    true
}

/// An ALWAYS weight / speed / jump crest on its wearer's fighter object (the multiplier is constant: a fighter wears
/// a crest once).
unsafe fn apply_attr(f: usize, id: usize) {
    let c = &POOL[id];
    if c.moment != M_ALWAYS {
        return;
    }
    let which = match c.effect {
        E_WEIGHT => AP_WEIGHT,
        E_SPEED => AP_SPEED,
        E_JUMP => AP_JUMP,
        _ => return,
    };
    gd_envoy_apply(0, which | AP_FIGHTER[f], 1000 + c.number);
}

/// MELEE_ENVOY_TEST_CRESTS="key,key/key" (main / partner): crests worn from the run's start (tests, demos).
unsafe fn test_crests() {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_crests(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    for (f, side) in s.split('/').take(2).enumerate() {
        for key in side.split(',') {
            if let Some(id) = by_key(key.trim()) {
                if wear(f, id) {
                    log_crest(b"test crest", id, f as i32);
                }
            }
        }
    }
}

/// The plan of the match the run wants next: the live run's current fight, or a new run's first.
unsafe fn make_plan() {
    if S.arm_seen == 0 {
        // not armed: no plan, so a VS match with the mods loaded is retail
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
    p[5] = 1000; // the ratios are the crests' (set every frame for the fighter in play)
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
        S.crests = [[0; SLOTS]; 2];
        S.picks = 0;
        S.main = gd_envoy_roster(0);
        S.partner = gd_envoy_roster(1);
        S.last_pick = -1;
        S.attrs_done = 0;
        log(b"envoy: run start, main\0", S.main);
        test_crests();
    }
    S.seq = seq;
    S.live = 0;
    S.frames = 0;
    S.play = 0;
    S.attrs_done = 0;
    S.reward_due = (S.fight > 0) as i32;
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
    S.fresh_until = [0; 2];
    S.bench_since = [0; 2];
    S.boost_until = 0;
    S.boost_to = 0;
    S.boost = 0;
    S.marked_until = [0; 3];
    S.burn_until = [0; 3];
    S.burn_next = [0; 3];
    S.atk_set = -1;
    S.def_set = -1;
    S.foe_def_set = [-1; 3];
    S.rw_active = 0;
    log(b"envoy: fight\0", S.fight + 1);
}

// ---- the crest offer

unsafe fn roll_offers() {
    let mut h = mix(S.seed as u32 ^ 0xA5A5_0000 ^ (S.fight as u32) << 4 ^ S.picks as u32);
    let mut fams = [false; 8];
    let mut n = 0;
    let mut tries = 0;
    S.offers = [OFFER_STOCK; 3];
    while n < 3 && tries < 400 {
        tries += 1;
        let c = (h % POOL_LEN as u32) as usize;
        h = mix(h.wrapping_add(0x6D2B_79F5));
        let fam = POOL[c].family as usize;
        if worn(c) || fams[fam] || S.offers[..n].contains(&(c as i32)) {
            continue;
        }
        S.offers[n] = c as i32;
        fams[fam] = true;
        n += 1;
    }
    // wearers: the fighter with fewer crests first, then alternating (ties: the roll), never a full fighter
    let (c0, c1) = (count(0), count(1));
    let base = if c0 < c1 { 0 } else if c1 < c0 { 1 } else { (h >> 7) as usize & 1 };
    for i in 0..3 {
        let mut w = (base + i) % 2;
        if count(w) >= SLOTS {
            w = 1 - w;
        }
        S.wearers[i] = w as i32;
    }
    if gd_player_stocks(0) <= 1 {
        S.offers[2] = OFFER_STOCK;
        S.wearers[2] = -1;
    }
}

unsafe fn pick(i: usize) {
    let id = S.offers[i];
    if id == OFFER_STOCK {
        gd_envoy_apply(0, AP_STOCKS, gd_player_stocks(0) + 1);
        log(b"envoy: took +1 stock\0", gd_player_stocks(0));
    } else if (0..POOL_LEN as i32).contains(&id) {
        let w = S.wearers[i].clamp(0, 1) as usize;
        if wear(w, id as usize) {
            log_crest(b"crest taken", id as usize, w as i32);
        }
    }
    S.picks += 1;
    S.rw_active = 0;
    S.last_pick = id;
    S.last_wearer = S.wearers[i];
    S.since_pick = 0;
    gd_hold_release();
}

unsafe fn reward_frame() {
    let pad = gd_pad_pressed(0);
    if pad & (DPAD_LEFT | STICK_LEFT) != 0 {
        S.cursor = (S.cursor + 2) % 3;
    }
    if pad & (DPAD_RIGHT | STICK_RIGHT) != 0 {
        S.cursor = (S.cursor + 1) % 3;
    }
    S.timer -= 1;
    if pad & BTN_A != 0 {
        pick(S.cursor as usize);
    } else if S.timer <= 0 {
        pick(0);
    }
}

// ---- moments

/// The foe slot (1..=3) nearest the fighter in play, or 0.
unsafe fn nearest_foe() -> i32 {
    let me = gd_fighter_from_slot(0);
    if me == 0 {
        return 0;
    }
    let (x, y) = (gd_fighter_get_pos_x(me), gd_fighter_get_pos_y(me));
    let mut best = 0;
    let mut bd = f32::MAX;
    for s in 1..=ncpu() {
        let h = gd_fighter_from_slot(s);
        if h == 0 || gd_player_stocks(s) <= 0 {
            continue;
        }
        let (dx, dy) = (gd_fighter_get_pos_x(h) - x, gd_fighter_get_pos_y(h) - y);
        let d = dx * dx + dy * dy;
        if d < bd {
            bd = d;
            best = s;
        }
    }
    best
}

fn ncpu() -> i32 {
    unsafe { SCHEDULE[S.fight.clamp(0, FIGHTS as i32 - 1) as usize].ncpu }
}

unsafe fn mark(s: i32, frames: i32) {
    if s >= 1 {
        let i = s as usize - 1;
        S.marked_until[i] = S.marked_until[i].max(S.play + frames);
    }
}

unsafe fn burn(s: i32, frames: i32) {
    if s >= 1 {
        let i = s as usize - 1;
        if S.burn_until[i] <= S.play {
            S.burn_next[i] = S.play + BURN_TICK;
        }
        S.burn_until[i] = S.burn_until[i].max(S.play + frames);
    }
}

unsafe fn heal(f: usize, milli: i32) {
    gd_envoy_percent(0, (f as i32 != S.active) as i32, -milli);
}

/// An edge moment's effect for crest `id` worn by `f`.
unsafe fn effect(id: usize, f: usize) -> bool {
    let c = &POOL[id];
    let n = c.number;
    match c.effect {
        E_HEAL => heal(f, n),
        E_HEAL_BOTH => {
            heal(0, n);
            heal(1, n);
        }
        E_MARK => mark(nearest_foe(), secs(n)),
        E_MARK_ALL => (1..=ncpu()).for_each(|s| mark(s, secs(n))),
        E_BURN => burn(nearest_foe(), secs(n)),
        E_BURN_ALL => (1..=ncpu()).for_each(|s| burn(s, secs(n))),
        E_FRESH => {
            let t = if c.moment == M_SHIFT { S.active as usize } else { f };
            let extra = if c.moment == M_STOCK_LOST { RESPAWN_FRAMES } else { 0 };
            S.fresh_until[t] = S.fresh_until[t].max(S.play + secs(n) + extra);
        }
        E_PARTNER_ATTACK => {
            S.boost_to = 1 - f as i32;
            S.boost = n;
            S.boost_until = S.play + HANDOFF_FRAMES;
        }
        E_FOE_PERCENT => {
            for s in 1..=ncpu() {
                gd_envoy_percent(s, 0, n);
            }
        }
        E_FIRST_SHIFT => {
            if S.fight < 1 {
                return false; // no shifts in fight 1
            }
            S.stage_clock = S.stage_clock.max(STAGE_PERIOD - secs(n));
        }
        _ => return false,
    }
    true
}

/// An edge moment: every crest keyed on it whose wearer qualifies (`who`: one fighter, or -1 for the party).
unsafe fn moment(m: u8, who: i32) {
    each_worn(|f, id| {
        let c = &POOL[id];
        if c.moment != m || (who >= 0 && f as i32 != who) || matches!(c.effect, E_FRESH_LEN | E_TAG_COOLDOWN) {
            return;
        }
        // Rebirth: only the fighter who lost the stock respawns
        if m == M_STOCK_LOST && c.effect == E_FRESH && f as i32 != S.active {
            return;
        }
        if effect(id, f) {
            fire(id, f);
        }
    });
}

unsafe fn on_tag() {
    let out = S.active as usize;
    let inn = 1 - out;
    S.active = inn as i32;
    S.bench_since[out] = S.play;
    // Fresh for the one coming in: 3 s, or longer by Linger / Charge
    let mut len = FRESH_FRAMES;
    each_worn(|f, id| {
        let c = &POOL[id];
        if f != inn || c.effect != E_FRESH_LEN {
            return;
        }
        let benched = S.play - S.bench_since[inn];
        if c.reads == W_BENCHED && benched < CHARGE_BENCH {
            return;
        }
        len = len.max(secs(c.number));
        fire(id, f);
    });
    S.fresh_until[inn] = S.play + len;
    log(b"envoy: moment tag in, fighter\0", inn as i32);
    moment(M_TAG_OUT, out as i32);
    moment(M_TAG_IN, inn as i32);
}

unsafe fn on_shift() {
    log(b"envoy: moment shift\0", S.stage_switches);
    moment(M_SHIFT, -1);
}

/// Per unfrozen live frame: the opener, the stock edges, the windows that tick.
unsafe fn moments_frame() {
    if S.opened == 0 {
        S.opened = 1;
        log(b"envoy: moment opener, fight\0", S.fight + 1);
        moment(M_OPENER, -1);
        if S.fight as usize == FIGHTS - 1 {
            log(b"envoy: moment boss\0", 0);
            moment(M_BOSS, -1);
        }
    }
    let me = gd_player_stocks(0);
    let mut foes = 0;
    for s in 1..=ncpu() {
        foes += gd_player_stocks(s).max(0);
    }
    if me < S.my_stocks && me > 0 {
        log(b"envoy: moment stock lost, pool\0", me);
        moment(M_STOCK_LOST, -1);
    }
    if foes < S.foe_stocks {
        log(b"envoy: moment KO dealt, foes left\0", foes);
        moment(M_KO_DEALT, -1);
    }
    S.my_stocks = me;
    S.foe_stocks = foes;
    // Benched: Rest heals the bench once a second
    let b = 1 - S.active as usize;
    if gd_tag_partner(0) >= 0 && (S.play - S.bench_since[b]) % 60 == 59 {
        each_worn(|f, id| {
            let c = &POOL[id];
            if f == b && c.moment == M_BENCHED && c.effect == E_HEAL {
                heal(f, c.number);
                S.fires += 1;
            }
        });
    }
    // Burning: 1% a second on the foe
    for i in 0..3 {
        if S.play < S.burn_until[i] && S.play >= S.burn_next[i] {
            S.burn_next[i] += BURN_TICK;
            if gd_player_present(i as i32 + 1) != 0 {
                gd_envoy_percent(i as i32 + 1, 0, 1000);
            }
        }
    }
}

/// The ratios of the fighter in play (its ALWAYS / Fresh / last-stock crests, Handoff) and of the foes (Marked);
/// written only when they change.
unsafe fn ratios_frame() {
    let a = S.active as usize;
    let mut atk = 1000;
    let mut def = 1000;
    let fresh = S.play < S.fresh_until[a];
    let last = gd_player_stocks(0) == 1;
    each_worn(|f, id| {
        let c = &POOL[id];
        if f != a {
            return;
        }
        let on = match c.moment {
            M_ALWAYS => true,
            M_FRESH => fresh,
            M_LAST_STOCK => last,
            _ => false,
        };
        if on {
            match c.effect {
                E_ATTACK => atk += c.number,
                E_DEFENSE => def -= c.number,
                _ => {}
            }
        }
    });
    if S.play < S.boost_until && S.boost_to == a as i32 {
        atk += S.boost;
    }
    let def = def.max(400);
    if atk != S.atk_set {
        S.atk_set = atk;
        gd_envoy_apply(0, AP_ATTACK, atk);
    }
    if def != S.def_set {
        S.def_set = def;
        gd_envoy_apply(0, AP_DEFENSE, def);
    }
    for i in 0..ncpu() as usize {
        let base = S.foe_def_base[i];
        let d = if S.play < S.marked_until[i] { base * MARKED_DEFENSE / 1000 } else { base };
        if d != S.foe_def_set[i] {
            S.foe_def_set[i] = d;
            gd_envoy_apply(i as i32 + 1, AP_DEFENSE, d);
        }
    }
}

/// The first live frame: the pair's kinds, attribute crests on each fighter object, the stock baselines.
unsafe fn fight_setup() {
    S.attrs_done = 1;
    let h = gd_fighter_from_slot(0);
    S.kind = [if h != 0 { gd_fighter_kind(h) } else { -1 }, gd_tag_partner(0)];
    each_worn(|f, id| apply_attr(f, id));
    for i in 0..3 {
        S.foe_def_base[i] = if PLAN[20 + i] > 0 { PLAN[20 + i] } else { 1000 };
    }
    S.my_stocks = gd_player_stocks(0);
    S.foe_stocks = (1..=ncpu()).map(|s| gd_player_stocks(s).max(0)).sum();
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

unsafe fn tag_cooldown() -> i32 {
    let mut cd = TAG_COOLDOWN;
    each_worn(|_, id| {
        if POOL[id].effect == E_TAG_COOLDOWN {
            cd = cd.min(secs(POOL[id].number));
        }
    });
    cd
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
        S.tag_cd = tag_cooldown();
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
        S.stage_clock = 0; // refused for good this time (e.g. unsupported stage): try again a period later
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

// view layout (envoy_ui): 0 magic, 1 state, 2 fight, 3 fights, 4 pool, 5 live, 6 main, 7 partner, 8 tag cd,
// 9 tag cd max, 10 offer open, 11 cursor, 12 timer, 13-15 offers (pool index or OFFER_STOCK), 16-18 their wearers,
// 19 stocks now, 20 -, 21 -, 22-27 worn crests (index + 1; main 22-24, partner 25-27), 28 cpus, 29 boss, 30 last
// pick, 31 frames since, 32 outcome, 33 offer time, 34 partner kind, 35 shifts, 36 seq, 37 armed, 38 fighter in
// play, 39 last pick's wearer, 40-71 the record, 72 crest fires, 73 last fire, 74 frames since it, 75 pool digest,
// 76 stage countdown, 77-78 Fresh frames left (main, partner).
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
    v[9] = tag_cooldown();
    v[10] = S.rw_active;
    v[11] = S.cursor;
    v[12] = S.timer;
    for i in 0..3 {
        v[13 + i] = S.offers[i];
        v[16 + i] = S.wearers[i];
    }
    v[19] = gd_player_stocks(0);
    for f in 0..2 {
        for k in 0..SLOTS {
            v[22 + f * SLOTS + k] = S.crests[f][k];
        }
    }
    v[28] = ncpu();
    let stage = if S.fight >= 1 && S.live != 0 { STAGE_PERIOD - S.stage_clock } else { -1 };
    v[29] = (S.fight as usize == FIGHTS - 1) as i32;
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
    v[73] = S.last_fire;
    v[74] = S.play - S.last_fire_at;
    v[75] = POOL_DIGEST as i32;
    v[76] = stage;
    v[77] = (S.fresh_until[0] - S.play).max(0);
    v[78] = (S.fresh_until[1] - S.play).max(0);
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
            if S.prev_phase == 1 && phase == 0 {
                if S.prev_kind == KIND_TAG {
                    on_tag();
                } else if S.prev_kind == KIND_STAGE {
                    on_shift();
                }
            }
            S.prev_phase = phase;
            S.prev_kind = kind;
            track_active();
            if S.rw_active != 0 {
                reward_frame();
            } else if S.reward_due != 0 && S.frames >= REWARD_DELAY && phase == 0 && S.outcome == 0 {
                if gd_hold_begin(REWARD_TIME + 120) == 0 {
                    S.reward_due = 0;
                    S.rw_active = 1;
                    S.cursor = 0;
                    S.timer = REWARD_TIME;
                    roll_offers();
                    log(b"envoy: crest offer, fight\0", S.fight + 1);
                }
            } else if phase == 0 || kind != KIND_HOLD {
                tag_frame();
                stage_frame();
                if gd_transition_phase() == 0 && S.reward_due == 0 && S.outcome == 0 {
                    S.play += 1;
                    moments_frame();
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
        S.last_fire = -1;
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
const POST: u32 = 2;
static HOOKS: [HookDecl; 1] = [
    HookDecl { address: 0x8039_0CFC, mode: POST, priority: 40, flags: 0, f: on_frame_end as *const () },
];

#[no_mangle]
pub static gdmod_abi_version: u32 = ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0002_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 4096, hook_count: 1, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.envoy_sim"), mod_name: text(b"Envoy run"),
};
