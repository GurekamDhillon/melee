//! envoy_sim: an Envoy run (docs/rust-port/notes/envoy-1p.md, envoy-drives.md, envoy-v3-rules.md). The sim half:
//! run state, the plan of the next fight, tags, mid-fight stage switches, drives and keystones, Heat (V3), local
//! co-op and the run's end. Everything is this module's linear memory (rolled back with the game) or engine statics
//! (melee_modes::tu::{envoy, tag_swap, petey}); the presentation (envoy_ui) only reads the published view.
//!
//! * run: the Short run, 5 fights: a duel, an elite (act boss 1), a pack, an elite (act boss 2), the final boss (a
//!   level-9 Ganondorf, or Petey with MELEE_ENVOY_BOSS=petey). Each player's pair (main + partner) shares one stock
//!   pool: 4 solo, 5 in co-op (MELEE_ENVOY_COOP=1: a second human in slot 1 with its own pair).
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown (Lock: 10 s), 60-frame frozen cinematic.
//! * stage switches: from fight 2 on, every 30 s of play, the next of BF / YS / DL / FoD / FD.
//! * V3 Heat (envoy-synergy-v3.md 3, 12): one meter per player, +1 per 1% dealt (never by hit count), -1 per 1%
//!   taken, Cold / Warm / Hot / Blazing scale every fire x1 / x1.25 / x1.5 / x2; a KO cashes out (coins into the one
//!   purse, pips for the active, a flare at the other foes) and keeps half; a lost stock halves it.
//! * drives (envoy_data): up to 3 / 4 / 5 per fighter in slot order. Triggers charge by damage (remainders carried),
//!   time, a tag, a shift or a cash-out, and fire as chains: a fire's damage charges the triggers to its right by its
//!   coefficient, at most 4 links 6 frames apart, 20 frames between chains. Echo makes its right neighbour fire twice.
//!   +Heat and xGain drives act on the Heat step; Insulate shapes the loss. Volley (D-pad right, 3 pips) fires the
//!   whole row. The Inferno set (three RED drives on one fighter): Burning to 6 stacks, its ticks feed Heat at 0.5.
//! * technique: the engine's L-cancel bit (`gd_envoy_events`), wavedash and tech polled, offstage damage by position.
//! * coins: cash-outs, a fight cleared, edgeguard KOs, interest; a shop after fights 1, 3 and 4.
//! * determinism: the whole state is two `#[repr(C)]` structs (i32 words; i64 words); every frame an FNV-1a over
//!   both, the plan and the pool digest goes into `gd_hash_word`. Integers only past the take-damage hook.
#![no_std]
#![allow(static_mut_refs)]

use envoy_data::heat;
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
    fn gd_fighter_get_action_state(h: u32) -> i32;
    fn gd_fighter_read(h: u32, out: *mut u32);
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
    fn gd_envoy_boss_dmg(out: *mut u32) -> i32;
    fn gd_envoy_events(slot: i32) -> u32;
    fn gd_envoy_kb_scale(slot: i32, mul_milli: i32) -> i32;
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
/// The shop's timer (it leaves on its own at 0).
const SHOP_TIME: i32 = 1800;
/// Frames into a fight before its reward screen opens (after the fighters are placed).
const REWARD_DELAY: i32 = 2;
/// After the last KO, the fight waits up to this long while a drop is still on the floor (the collect hold), then
/// the drops are swept up.
const COLLECT_HOLD: i32 = 600;

const DPAD_LEFT: u32 = 0x0001;
const DPAD_RIGHT: u32 = 0x0002;
const DPAD_DOWN: u32 = 0x0004;
const DPAD_UP: u32 = 0x0008;
const BTN_A: u32 = 0x0100;
const BTN_B: u32 = 0x0200;
const BTN_X: u32 = 0x0400;
const BTN_Y: u32 = 0x0800;
const STICK_UP: u32 = 0x0001_0000;
const STICK_DOWN: u32 = 0x0002_0000;
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
/// The ledge x of each (offstage damage, V3 2.3) [K: the retail stage edges, not read from the stage files].
const LEDGE_X: [f32; 5] = [68.4, 56.0, 77.3, 63.4, 85.6];
/// CPU characters (CharacterKind, the CSS ids): the retail cast minus Ice Climbers / Zelda / Sheik / Ganondorf (boss).
const CPU_POOL: [i32; 22] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 20, 21, 22, 23, 24];
const CK_GANONDORF: i32 = 0x19;

/// Action states: JumpF..JumpAerialB; AttackAirN..AttackAirLw; the smash attacks (AttackS4Hi..AttackLw4); the attack
/// states a move can be (Attack11 and up); LandingFallSpecial, EscapeAir, the Passive (tech) states.
const AS_JUMP: (i32, i32) = (25, 28);
const AS_AERIAL: (i32, i32) = (65, 69);
const AS_SMASH: (i32, i32) = (58, 64);
const AS_ATTACK0: i32 = 44;
const AS_LANDING_SPECIAL: i32 = 43;
const AS_ESCAPE_AIR: i32 = 236;
const AS_TECH: (i32, i32) = (199, 204);
const AS_LANDING_AIR: (i32, i32) = (70, 74);
/// Landing, LandingFallSpecial, LandingAirN..LandingAirLw (a landing).
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
/// A shop opens at the start of these fights (after fights 1, 3 and 4; V3 12.9).
const SHOP_AT: [bool; FIGHTS] = [false, true, false, true, true];
/// The drives a foe may carry: the ones that read as an attack on the players.
const FOE_POOL: [&str; 8] = ["ember", "nova", "bolt", "frost", "rush", "bulwark", "hex", "comet"];

const RUN_NONE: i32 = 0;
const RUN_LIVE: i32 = 1;
const RUN_CLEAR: i32 = 2;
const RUN_OVER: i32 = 3;

const NP: usize = 2;
const SETS: usize = 4;
const OWNERS: usize = 8;
const PEND: usize = 12;
const RECS: usize = 24;
/// Echo Oath's repeat: frames after the fire; Feather Storm's hits, frames apart.
const ECHO_DELAY: i32 = 12;
const FEATHER_GAP: i32 = 8;
/// Effect reach in world units (fighters are about 10-15 tall).
const BOLT_R: f32 = 90.0;
const SPLASH_R: f32 = 26.0;
const NOVA_R: f32 = 38.0;
const SHOCK_R: f32 = 32.0;
/// Frost's Chilled, Hex's and Thorns' Marked (frames); Winter Oath's Chilled.
const CHILL_LEN: i32 = 240;
const MARK_LEN: i32 = 300;
/// Glass Core's price: damage taken x1.6.
const GLASS_TAKE: i32 = 1600;
/// A floor drop is taken when its owner's fighter in play stands this close (x, and y above the drop).
const PICK_DX: f32 = 11.0;
const PICK_DY: f32 = 22.0;
/// Floor points (x100) a drop lands on: the main floor of every stage in the rotation is at y 0.
const DROP_X: [i32; 4] = [-3500, -1200, 1200, 3500];
/// An edgeguard KO: the foe took offstage damage this recently.
const EDGEGUARD_WINDOW: i32 = 120;

// pending kinds
const PK_ECHO: i32 = 1;
const PK_PULSE: i32 = 2;
// a gain's source
const G_DROP: i32 = 1;
const G_PICK: i32 = 2;
const G_KEYSTONE: i32 = 3;
const G_SWEPT: i32 = 4;
const G_SHOP: i32 = 5;

// a damage record's tags
const T_AERIAL: i32 = 1;
const T_SMASH: i32 = 2;
const T_OFFSTAGE: i32 = 4;
const T_FIRE: i32 = 8;
const T_WORD: i32 = 16;
// how a record charges: a move (every matching source), a fire (only to the right of its slot), word damage
// (nothing; under Plague Bearer the damage sources except Burning's own drives), nothing
const CM_MOVE: i32 = 0;
const CM_FIRE: i32 = 1;
const CM_WORD: i32 = 2;
const CM_NONE: i32 = 3;
// the fire context hurt() reads (fx_mode)
const FX_FIRE: i32 = 0;
const FX_FLARE: i32 = 1;
const FX_FOE: i32 = 2;

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
    /// Worn drives per set (player * 2 + fighter; shared: fighter 0 only), in slot order: pool index + 1, level 1-3.
    drives: [[i32; SLOTS]; SETS],
    levels: [[i32; SLOTS]; SETS],
    /// Keystones per player: pool index + 1 (0 empty).
    keys: [[i32; KEYSTONES]; NP],
    arm_seen: i32,
    rerolls: i32,
    act_id: [i32; NP],
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
    won_pool: i32,
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
    boss_dmg: [i32; 4],
    boss_breaks: i32,
    boss_hp: i32,
    boss_hp_max: i32,
    /// The slot each drive's charge last struck (its target).
    target: [[i32; SLOTS]; SETS],
    /// Per owner (sets 0-3, foe slots 4-7): the last fire frame (foes: one fire per 20 frames).
    last_fire_at: [i32; OWNERS],
    /// Chains per set: running, the slot of the queued link, link number, parent coefficient, frame of the next link,
    /// frame the last chain ended, Volley (no link limit; fires uncharged triggers too), the last link's log index.
    ch_on: [i32; SETS],
    ch_k: [i32; SETS],
    ch_link: [i32; SETS],
    ch_coef: [i32; SETS],
    ch_next: [i32; SETS],
    ch_end: [i32; SETS],
    ch_volley: [i32; SETS],
    ch_log: [i32; SETS],
    chain_n: i32,
    /// Pending repeats and pulses: (frame, kind, id + 1, owner, level, frac, target, flags, coef, slot).
    pend: [[i32; 10]; PEND],
    /// Bulwark's orb per owner; Haste per owner (until); Updraft's banked jump per player.
    shield: [i32; OWNERS],
    haste_until: [i32; OWNERS],
    banked_jump: [i32; NP],
    // statuses per match slot
    burn_until: [i32; 4],
    burn_next: [i32; 4],
    burn_stacks: [i32; 4],
    burn_acc: [i32; 4],
    burn_owner: [i32; 4],
    chill_until: [i32; 4],
    mark_until: [i32; 4],
    /// The sim's own percent per slot this stock (x1000): the 150% line.
    foe_m: [i32; 4],
    /// The last offstage damage on a slot (frame), the last player to hit it with a smash (+1), the last hitter (+1).
    last_off: [i32; 4],
    last_smash: [i32; 4],
    /// Foe drives per match slot (elites and bosses): id + 1, level, target.
    fdrives: [[i32; 2]; 4],
    flevels: [[i32; 2]; 4],
    ftarget: [[i32; 2]; 4],
    atk_set: [i32; NP],
    def_set: [i32; NP],
    fires: i32,
    hits: i32,
    taken: i32,
    blasts: i32,
    // ---- V3 per player
    hstate: [i32; NP],
    free: [i32; NP],
    tally: [[i32; 4]; NP],
    marks: [[i32; 3]; NP],
    streak: [i32; NP],
    aerial_dealt: [i32; NP],
    hist: [[i32; 3]; NP],
    cur_move: [i32; NP],
    peak: [i32; NP],
    cashouts: [i32; NP],
    loss_acc: [i32; NP],
    stock_lost: i32,
    /// The fire context hurt() reads: mode, the firing set, its slot, the record's coefficient, the fire's damage
    /// total dealt so far, the first target struck.
    fx_mode: i32,
    fx_set: i32,
    fx_slot: i32,
    fx_coef: i32,
    fx_dealt: i32,
    fx_first: i32,
    /// The frame's damage records: (attacker slot or -1, victim slot, dmg x1000, tags, attacker action state).
    rec: [[i32; 5]; RECS],
    rec_n: i32,
    /// The event ring (12.8).
    ev: [[i32; EV_W]; EV],
    ev_n: i32,
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
    scursor: [i32; NP],
    last_pick: [i32; NP],
    last_wearer: [i32; NP],
    since_pick: i32,
    // ---- the shop
    shop_due: i32,
    shop_off: [[i32; 3]; NP],
    shop_cursor: [i32; NP],
    shop_step: [i32; NP],
    shop_fighter: [i32; NP],
    shop_msg: [i32; NP],
    shop_pending: [i32; NP],
    shop_rerolls: i32,
    /// Test knobs (MELEE_ENVOY_TEST_DRIVES): slots opened for a test row, no shop.
    slots_bonus: i32,
    no_shop: i32,
    /// Test knob (`finaleoff`): Grand Finale's write is computed and logged but not made (the A/B launch probe);
    /// the victim of the last write (or would-be write) and its frame, to log the launch on the next frame.
    finale_off: i32,
    /// Test knob (`aslog`): log player 1's action-state changes; the frame an air dodge was last seen per player.
    as_log: i32,
    /// Test knob (`^N`): every fight starts with the Heat that gives Grand Finale +N/1000 (the A/B probe at a fixed Heat).
    test_heat: i64,
    escape_at: [i32; NP],
    kb_watch: i32,
    kb_watch_at: i32,
}

/// The 64-bit half of the state (Heat and charge in ticks, the purse, the pips): i64 words only.
#[repr(C)]
struct H {
    heat: [i64; NP],
    chg: [[i64; SLOTS]; SETS],
    fchg: [[i64; 2]; 4],
    purse: i64,
    earned: [i64; NP],
    pips: [i64; NP],
}

static mut S: St = unsafe { core::mem::zeroed() };
static mut HH: H = unsafe { core::mem::zeroed() };
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

unsafe fn coins() -> i32 {
    (HH.purse / COIN_UNIT) as i32
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
    r[R_COINS] = coins();
    r[R_REROLLS] = S.rerolls;
    r[R_ACTIVE] = S.act_id[0] | S.act_id[1] << 8;
    r[R_PIPS] = ((HH.pips[0] / PIP_UNIT) as i32).clamp(0, 255) | ((HH.pips[1] / PIP_UNIT) as i32).clamp(0, 255) << 8;
    r[R_EARNED] = ((HH.earned[0] / COIN_UNIT) as i32).clamp(0, 0xFFFF) | ((HH.earned[1] / COIN_UNIT) as i32).clamp(0, 0x7FFF) << 16;
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
        HH.purse = r[R_COINS].max(0) as i64 * COIN_UNIT;
        S.rerolls = r[R_REROLLS].max(0);
        S.act_id = [r[R_ACTIVE] & 0xFF, r[R_ACTIVE] >> 8 & 0xFF];
        HH.pips = [(r[R_PIPS] & 0xFF) as i64 * PIP_UNIT, (r[R_PIPS] >> 8 & 0xFF) as i64 * PIP_UNIT];
        HH.earned = [(r[R_EARNED] & 0xFFFF) as i64 * COIN_UNIT, (r[R_EARNED] >> 16 & 0x7FFF) as i64 * COIN_UNIT];
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

// ---- the event ring

unsafe fn ev(kind: i32, who: i32, amount: i32) {
    S.ev[S.ev_n as usize % EV] = [S.play, kind, who, amount];
    S.ev_n += 1;
}

// ---- drives: who wears what

/// The set fighter `f` of player `p` uses (the pair's one set when drives are shared).
unsafe fn set_of(p: usize, f: usize) -> usize {
    p * 2 + if S.shared != 0 { 0 } else { f }
}

/// Drive slots each fighter has open: 3, then 4 after the first boss, 5 after the second (V3 5.3).
unsafe fn open_slots() -> usize {
    (3 + S.bosses.clamp(0, 2) + S.slots_bonus).min(SLOTS as i32) as usize
}

unsafe fn slot_of(set: usize, id: usize) -> Option<usize> {
    S.drives[set].iter().position(|&d| d == id as i32 + 1)
}

/// Whether fighter `f` of player `p` can take drive `id` without replacing: a level-up below 3, or a free slot.
unsafe fn can_take(p: usize, f: usize, id: usize) -> bool {
    let set = set_of(p, f);
    match slot_of(set, id) {
        Some(k) => S.levels[set][k] < MAX_LEVEL,
        None => S.drives[set][..open_slots()].contains(&0),
    }
}

/// Whether fighter `f` could take drive `id` by replacing one (a full row, not worn).
unsafe fn can_replace(p: usize, f: usize, id: usize) -> bool {
    let set = set_of(p, f);
    slot_of(set, id).is_none() && !S.drives[set][..open_slots()].contains(&0)
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
            HH.chg[set][k] = 0;
            1
        }
    };
    note_gain(p, f, id, lv, how);
    lv
}

unsafe fn note_gain(p: usize, f: usize, id: usize, lv: i32, how: i32) {
    let n = S.gain[p][5] + 1;
    S.gain[p] = [S.play, id as i32 + 1, f as i32, lv, how, n];
    log_drive(if lv > 1 { b"drive level up" } else { b"drive gained" }, id, p as i32 * 100 + f as i32 * 10 + lv);
    if has_inferno(set_of(p, f)) {
        ev(EV_INFERNO, set_of(p, f) as i32, 1);
    }
}

/// Replace slot `k` of fighter `f` with drive `id` (a full row, V3 5.4): the old drive is gone.
unsafe fn replace(p: usize, f: usize, k: usize, id: usize, how: i32) {
    let set = set_of(p, f);
    let old = S.drives[set][k];
    if old > 0 {
        log_drive(b"drive replaced", old as usize - 1, p as i32 * 100 + f as i32 * 10 + k as i32);
    }
    S.drives[set][k] = id as i32 + 1;
    S.levels[set][k] = 1;
    HH.chg[set][k] = 0;
    note_gain(p, f, id, 1, how);
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

/// The Inferno set: three different RED drives on one fighter's row (V3 6.2).
unsafe fn has_inferno(set: usize) -> bool {
    S.drives[set][..open_slots()].iter().filter(|&&d| d > 0 && POOL[d as usize - 1].family == F_RED).count() >= 3
}

/// MELEE_ENVOY_TEST_DRIVES="key[:level],key/key|keystone,keystone" (main / partner | keystones), and in co-op a
/// second player's after `;`: worn from the run's start (tests, demos). Keystones given here skip the keystone pick.
/// V3: an item `$N` puts N coins in the purse; `noshop` skips the shops (tests); a row longer than the slots open
/// opens more for the run (tests).
unsafe fn test_drives() {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    S.slots_bonus = 2; // a test row may fill all five slots
    for (p, player) in s.split(';').take(NP).enumerate() {
        let (drv, ks) = player.split_once('|').unwrap_or((player, ""));
        for (f, side) in drv.split('/').take(2).enumerate() {
            for item in side.split(',') {
                let item = item.trim();
                if item == "aslog" {
                    S.as_log = 1;
                    continue;
                }
                if item == "finaleall" {
                    S.finale_off |= 2;
                    continue;
                }
                if item == "finaleoff" {
                    S.finale_off |= 1;
                    continue;
                }
                if item == "noshop" {
                    S.no_shop = 1;
                    continue;
                }
                if let Some(c) = item.strip_prefix('^') {
                    S.test_heat = c.parse::<i64>().unwrap_or(0).clamp(0, 500) * FINALE_TICKS;
                    continue;
                }
                if let Some(c) = item.strip_prefix('$') {
                    HH.purse += c.parse::<i64>().unwrap_or(0).max(0) * COIN_UNIT;
                    continue;
                }
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
    // keep only the slots the test row uses
    let used = (0..SETS).map(|s| S.drives[s].iter().rposition(|&d| d > 0).map_or(0, |k| k + 1)).max().unwrap_or(0) as i32;
    S.slots_bonus = (used - 3 - S.bosses).clamp(0, 2);
    S.gain = [[0; 6]; NP];
    S.ev_n = 0;
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
        S.rerolls = 0;
        S.slots_bonus = 0;
        S.no_shop = 0;
        S.finale_off = 0;
        S.as_log = 0;
        S.test_heat = 0;
        S.escape_at = [-99; NP];
        S.kb_watch = -1;
        S.shared = (gd_envoy_shared_drives() != 0) as i32;
        S.pair = [[gd_envoy_roster(0), gd_envoy_roster(1)], [gd_envoy_roster(2), gd_envoy_roster(3)]];
        S.last_pick = [-1; NP];
        S.attrs_done = 0;
        HH = core::mem::zeroed();
        // Volley is given at the run's start with its pips full (V3 12.6)
        for p in 0..nh() as usize {
            S.act_id[p] = ACTIVE_VOLLEY;
            HH.pips[p] = VOLLEY_PIPS * PIP_UNIT;
        }
        log(b"envoy: run start, players\0", S.nh);
        if S.shared != 0 {
            log(b"envoy: drives shared by the pair\0", 1);
        }
        test_drives();
    } else if S.fight > 0 {
        // interest: +1 per 10 coins held, to +5
        let i = (coins() / 10).min(INTEREST_MAX);
        if i > 0 {
            HH.purse += i as i64 * COIN_UNIT;
            log(b"envoy: interest, coins\0", i);
        }
    }
    S.seq = seq;
    S.live = 0;
    S.frames = 0;
    S.play = 0;
    S.attrs_done = 0;
    let keys_short = (0..nh() as usize).any(|p| key_count(p) < keys_earned());
    S.shop_due = (SHOP_AT[S.fight.clamp(0, FIGHTS as i32 - 1) as usize] && S.no_shop == 0) as i32;
    // V3 12.9: the drive pick first, then the keystone pick (after a boss); the run's first keystone opens fight 1
    S.reward_due = if S.fight > 0 { RW_DRIVES } else if keys_short { RW_KEYS } else { 0 };
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
    S.boss_dmg = [0; 4];
    S.boss_breaks = 0;
    S.boss_hp = 0;
    S.boss_hp_max = 0;
    HH.chg = [[0; SLOTS]; SETS];
    HH.fchg = [[0; 2]; 4];
    HH.heat = [S.test_heat; NP];
    S.target = [[-1; SLOTS]; SETS];
    S.last_fire_at = [-CHAIN_GAP; OWNERS];
    S.ch_on = [0; SETS];
    S.ch_k = [0; SETS];
    S.ch_link = [0; SETS];
    S.ch_coef = [1000; SETS];
    S.ch_next = [0; SETS];
    S.ch_end = [-CHAIN_GAP; SETS];
    S.ch_volley = [0; SETS];
    S.ch_log = [-1; SETS];
    S.chain_n = 0;
    S.pend = [[0; 10]; PEND];
    S.shield = [0; OWNERS];
    S.haste_until = [0; OWNERS];
    S.banked_jump = [0; NP];
    S.burn_until = [0; 4];
    S.burn_next = [0; 4];
    S.burn_stacks = [0; 4];
    S.burn_acc = [0; 4];
    S.burn_owner = [-1; 4];
    S.chill_until = [0; 4];
    S.mark_until = [0; 4];
    S.foe_m = [0; 4];
    S.last_off = [-9999; 4];
    S.last_smash = [0; 4];
    S.fdrives = [[0; 2]; 4];
    S.flevels = [[0; 2]; 4];
    S.ftarget = [[-1; 2]; 4];
    S.atk_set = [-1; NP];
    S.def_set = [-1; NP];
    S.hstate = [0; NP];
    S.free = [INSULATE_FREE; NP];
    S.tally = [[0; 4]; NP];
    S.marks = [[0; 3]; NP];
    S.streak = [0; NP];
    S.aerial_dealt = [0; NP];
    S.hist = [[-1; 3]; NP];
    S.cur_move = [-1; NP];
    S.peak = [0; NP];
    S.cashouts = [0; NP];
    S.loss_acc = [0; NP];
    S.stock_lost = 0;
    S.rec_n = 0;
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
    for p in 0..nh() as usize {
        for f in 0..2 {
            if has_inferno(set_of(p, f)) {
                ev(EV_INFERNO, set_of(p, f) as i32, 1);
            }
        }
    }
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

/// A card player `p` may be offered: a keystone not held; a drive either fighter can take, or put in place of one
/// (a full row; V3 5.4). A drive at level 3 on every fighter is never offered.
unsafe fn offer_ok(p: usize, c: usize, keys: bool) -> bool {
    if keys {
        POOL[c].kind == K_KEYSTONE && !S.keys[p].contains(&(c as i32 + 1))
    } else {
        let fits = |f: usize| can_take(p, f, c) || can_replace(p, f, c);
        POOL[c].kind == K_DRIVE && (fits(0) || (S.shared == 0 && fits(1)))
    }
}

unsafe fn connects(c: i32, words: u32) -> bool {
    c != OFFER_STOCK && words & ((1 << POOL[c as usize].gives) | (1 << POOL[c as usize].reads)) & !1 != 0
}

unsafe fn roll_offers(p: usize) {
    let mut h = mix(S.seed as u32 ^ 0xA5A5_0000 ^ (S.fight as u32) << 4 ^ (S.picks as u32) << 12 ^ (p as u32) << 24);
    S.offers[p] = [OFFER_STOCK; 3];
    let mut n = 0;
    let keys = S.rw_active == RW_KEYS;
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
    S.timer = if kind == RW_SHOP { SHOP_TIME } else { REWARD_TIME };
    for p in 0..NP {
        S.rw_done[p] = (p as i32 >= nh() || (kind == RW_KEYS && key_count(p) >= keys_earned())) as i32;
        S.cursor[p] = 0;
        S.step[p] = 0;
        S.fcursor[p] = 0;
        S.scursor[p] = 0;
        if S.rw_done[p] == 0 {
            if kind == RW_SHOP {
                open_shop(p);
            } else {
                roll_offers(p);
            }
        }
    }
    log(
        match kind {
            RW_KEYS => b"envoy: keystone offer, fight\0",
            RW_SHOP => b"envoy: shop open, fight\0",
            _ => b"envoy: drive offer, fight\0",
        },
        S.fight + 1,
    );
}

/// The default fighter for a drive card: the one in play when it can take it, else the other.
unsafe fn default_fighter(p: usize, id: i32) -> i32 {
    if id == OFFER_STOCK || S.shared != 0 || !(0..POOL_LEN as i32).contains(&id) {
        return 0;
    }
    let a = S.active[p] as usize;
    if can_take(p, a, id as usize) || !can_take(p, 1 - a, id as usize) { a as i32 } else { 1 - a as i32 }
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
    done_pick(p, id, f);
}

unsafe fn done_pick(p: usize, id: i32, f: i32) {
    log(b"envoy: card taken, player\0", p as i32);
    S.picks += 1;
    S.last_pick[p] = id;
    S.last_wearer[p] = f;
    S.since_pick = 0;
    S.rw_done[p] = 1;
}

/// Every player has picked (or the timer ran out): the next screen (keystones -> drives -> the shop), or the fight.
unsafe fn reward_close() {
    if S.rw_active == RW_DRIVES && (0..nh() as usize).any(|p| key_count(p) < keys_earned()) {
        open_reward(RW_KEYS);
        return;
    }
    if S.rw_active != RW_SHOP && S.shop_due != 0 {
        S.shop_due = 0;
        open_reward(RW_SHOP);
        return;
    }
    S.rw_active = 0;
    gd_hold_release();
}

/// A drive card chosen for fighter `f`: take it, or (a full row) go to the slot step.
unsafe fn choose(p: usize, id: i32, f: i32) {
    if id != OFFER_STOCK && (0..POOL_LEN as i32).contains(&id) && POOL[id as usize].kind == K_DRIVE && !can_take(p, f as usize, id as usize) {
        if can_replace(p, f as usize, id as usize) {
            S.step[p] = 2;
            S.fcursor[p] = f;
            S.scursor[p] = 0;
        }
        return;
    }
    finish_pick(p, id, f);
}

unsafe fn reward_frame() {
    S.timer -= 1;
    if S.rw_active == RW_SHOP {
        shop_frame();
    } else {
        for p in 0..nh() as usize {
            if S.rw_done[p] != 0 {
                continue;
            }
            pick_frame(p);
        }
    }
    if (0..nh() as usize).all(|p| S.rw_done[p] != 0) {
        reward_close();
    }
}

unsafe fn pick_frame(p: usize) {
    let pad = gd_pad_pressed(p as i32);
    let left = pad & (DPAD_LEFT | STICK_LEFT) != 0;
    let right = pad & (DPAD_RIGHT | STICK_RIGHT) != 0;
    let id = S.offers[p][S.cursor[p].clamp(0, 2) as usize];
    match S.step[p] {
        0 => {
            if left {
                S.cursor[p] = (S.cursor[p] + 2) % 3;
            }
            if right {
                S.cursor[p] = (S.cursor[p] + 1) % 3;
            }
            let id = S.offers[p][S.cursor[p].clamp(0, 2) as usize];
            let one_fighter = id == OFFER_STOCK || S.rw_active == RW_KEYS || S.shared != 0 || gd_tag_partner(p as i32) < 0;
            if pad & BTN_A != 0 {
                if one_fighter {
                    choose(p, id, default_fighter(p, id));
                } else {
                    S.step[p] = 1;
                    S.fcursor[p] = default_fighter(p, id);
                }
            } else if S.timer <= 0 {
                timeout_pick(p);
            }
        }
        1 => {
            if left || right {
                S.fcursor[p] = 1 - S.fcursor[p];
            }
            if pad & BTN_B != 0 {
                S.step[p] = 0;
            } else if pad & BTN_A != 0 {
                choose(p, id, S.fcursor[p]);
            } else if S.timer <= 0 {
                timeout_pick(p);
            }
        }
        _ => {
            // step 2: which drive the new one replaces (a full row)
            let n = open_slots() as i32;
            if left {
                S.scursor[p] = (S.scursor[p] + n - 1) % n;
            }
            if right {
                S.scursor[p] = (S.scursor[p] + 1) % n;
            }
            if pad & BTN_B != 0 {
                S.step[p] = 0;
            } else if pad & BTN_A != 0 {
                let f = S.fcursor[p];
                replace(p, f as usize, S.scursor[p].clamp(0, n - 1) as usize, id as usize, G_PICK);
                done_pick(p, id, f);
            } else if S.timer <= 0 {
                log(b"envoy: pick skipped (full row, time), player\0", p as i32);
                done_pick(p, -1, 0);
            }
        }
    }
}

/// The timer ran out on a pick: card 0 for its default fighter when it fits; a full row skips it.
unsafe fn timeout_pick(p: usize) {
    let id = S.offers[p][0];
    let f = default_fighter(p, id);
    if id == OFFER_STOCK || POOL[id as usize].kind == K_KEYSTONE || can_take(p, f as usize, id as usize) {
        finish_pick(p, id, f);
    } else {
        done_pick(p, -1, f);
    }
}

// ---- the shop (V3 7.3, 12.9): a screen at the start of fights 2, 4 and 5

unsafe fn price_of(id: i32) -> i32 {
    if (0..POOL_LEN as i32).contains(&id) { PRICE_TIER[POOL[id as usize].tier.min(2) as usize] } else { 0 }
}

unsafe fn roll_shop(p: usize) {
    let mut h = mix(S.seed as u32 ^ 0x5A0B_0000 ^ (S.fight as u32) << 4 ^ (S.rerolls as u32) << 12 ^ (p as u32) << 24);
    S.shop_off[p] = [-1; 3];
    let mut n = 0;
    let mut tries = 0;
    while n < 3 && tries < 400 {
        tries += 1;
        let c = (h % POOL_LEN as u32) as usize;
        h = mix(h.wrapping_add(0x6D2B_79F5));
        if POOL[c].kind != K_DRIVE || S.shop_off[p][..n].contains(&(c as i32)) || !offer_ok(p, c, false) {
            continue;
        }
        S.shop_off[p][n] = c as i32;
        n += 1;
    }
}

unsafe fn open_shop(p: usize) {
    S.shop_cursor[p] = 0;
    S.shop_step[p] = 0;
    S.shop_fighter[p] = S.active[p];
    S.shop_msg[p] = SM_NONE;
    S.shop_pending[p] = -1;
    S.shop_rerolls = 0;
    roll_shop(p);
}

unsafe fn reroll_cost() -> i32 {
    PRICE_REROLL + PRICE_REROLL * S.shop_rerolls
}

unsafe fn spend(c: i32) -> bool {
    if coins() < c {
        return false;
    }
    HH.purse -= c as i64 * COIN_UNIT;
    true
}

unsafe fn level_price(set: usize, k: usize) -> i32 {
    let lv = S.levels[set][k];
    if S.drives[set][k] > 0 && (1..MAX_LEVEL).contains(&lv) { PRICE_LEVEL[lv as usize - 1] } else { 0 }
}

unsafe fn shop_frame() {
    for p in 0..nh() as usize {
        if S.rw_done[p] != 0 {
            continue;
        }
        if S.timer <= 0 {
            S.rw_done[p] = 1;
            continue;
        }
        let pad = gd_pad_pressed(p as i32);
        let up = pad & (DPAD_UP | STICK_UP) != 0;
        let down = pad & (DPAD_DOWN | STICK_DOWN) != 0;
        let left = pad & (DPAD_LEFT | STICK_LEFT) != 0;
        let right = pad & (DPAD_RIGHT | STICK_RIGHT) != 0;
        let f = if S.shared != 0 { 0 } else { S.shop_fighter[p].clamp(0, 1) as usize };
        let set = set_of(p, f);
        let n = open_slots() as i32;
        match S.shop_step[p] {
            0 => {
                if up {
                    S.shop_cursor[p] = (S.shop_cursor[p] + SHOP_ROWS - 1) % SHOP_ROWS;
                }
                if down {
                    S.shop_cursor[p] = (S.shop_cursor[p] + 1) % SHOP_ROWS;
                }
                if pad & (BTN_X | BTN_Y) != 0 && S.shared == 0 && gd_tag_partner(p as i32) >= 0 {
                    S.shop_fighter[p] = 1 - S.shop_fighter[p];
                }
                if pad & BTN_A == 0 {
                    continue;
                }
                let row = S.shop_cursor[p];
                S.shop_msg[p] = match row {
                    0..=2 => {
                        let id = S.shop_off[p][row as usize];
                        if id < 0 {
                            SM_CANT
                        } else if coins() < price_of(id) {
                            SM_POOR
                        } else if can_take(p, f, id as usize) {
                            spend(price_of(id));
                            gain(p, f, id as usize, G_SHOP);
                            S.shop_off[p][row as usize] = -1;
                            log_drive(b"shop bought", id as usize, p as i32);
                            SM_BOUGHT
                        } else if can_replace(p, f, id as usize) {
                            S.shop_step[p] = 1;
                            S.shop_pending[p] = row;
                            S.scursor[p] = 0;
                            SM_FULL
                        } else {
                            SM_CANT
                        }
                    }
                    SHOP_LEVEL => {
                        S.shop_step[p] = 1;
                        S.shop_pending[p] = SHOP_LEVEL;
                        S.scursor[p] = 0;
                        SM_NONE
                    }
                    SHOP_STOCK => {
                        if spend(PRICE_STOCK) {
                            S.pool += 1;
                            for h in 0..nh() {
                                gd_envoy_apply(h, AP_STOCKS, gd_player_stocks(h) + 1);
                            }
                            log(b"envoy: shop stock, pool\0", S.pool);
                            SM_STOCK
                        } else {
                            SM_POOR
                        }
                    }
                    SHOP_REROLL => {
                        if spend(reroll_cost()) {
                            S.shop_rerolls += 1;
                            S.rerolls += 1;
                            roll_shop(p);
                            log(b"envoy: shop reroll, player\0", p as i32);
                            SM_REROLLED
                        } else {
                            SM_POOR
                        }
                    }
                    SHOP_REORDER => {
                        S.shop_step[p] = 2;
                        S.scursor[p] = 0;
                        SM_NONE
                    }
                    _ => {
                        S.rw_done[p] = 1;
                        log(b"envoy: shop left, coins\0", coins());
                        SM_NONE
                    }
                };
            }
            1 => {
                // a slot: to level it, or to put a bought drive in place of it
                if left {
                    S.scursor[p] = (S.scursor[p] + n - 1) % n;
                }
                if right {
                    S.scursor[p] = (S.scursor[p] + 1) % n;
                }
                let k = S.scursor[p].clamp(0, n - 1) as usize;
                if pad & BTN_B != 0 {
                    S.shop_step[p] = 0;
                    S.shop_msg[p] = SM_NONE;
                } else if pad & BTN_A != 0 {
                    let row = S.shop_pending[p];
                    if row == SHOP_LEVEL {
                        let c = level_price(set, k);
                        S.shop_msg[p] = if c == 0 {
                            SM_CANT
                        } else if spend(c) {
                            S.levels[set][k] += 1;
                            note_gain(p, f, S.drives[set][k] as usize - 1, S.levels[set][k], G_SHOP);
                            SM_LEVELED
                        } else {
                            SM_POOR
                        };
                    } else if (0..3).contains(&row) {
                        let id = S.shop_off[p][row as usize];
                        S.shop_msg[p] = if id >= 0 && spend(price_of(id)) {
                            replace(p, f, k, id as usize, G_SHOP);
                            S.shop_off[p][row as usize] = -1;
                            SM_REPLACED
                        } else {
                            SM_POOR
                        };
                    }
                    S.shop_step[p] = 0;
                }
            }
            2 => {
                // reorder: pick the drive to move
                if left {
                    S.scursor[p] = (S.scursor[p] + n - 1) % n;
                }
                if right {
                    S.scursor[p] = (S.scursor[p] + 1) % n;
                }
                if pad & BTN_B != 0 {
                    S.shop_step[p] = 0;
                } else if pad & BTN_A != 0 && S.drives[set][S.scursor[p].clamp(0, n - 1) as usize] > 0 {
                    S.shop_step[p] = 3;
                }
            }
            _ => {
                // reorder: move the lifted drive one slot at a time (a swap with its neighbour)
                let k = S.scursor[p].clamp(0, n - 1) as usize;
                let to = if left && k > 0 { k - 1 } else if right && (k as i32) < n - 1 { k + 1 } else { k };
                if to != k {
                    S.drives[set].swap(k, to);
                    S.levels[set].swap(k, to);
                    HH.chg[set].swap(k, to);
                    S.target[set].swap(k, to);
                    S.scursor[p] = to as i32;
                    S.shop_msg[p] = SM_MOVED;
                    log(b"envoy: shop reorder, set*10 + slot\0", set as i32 * 10 + to as i32);
                }
                if pad & (BTN_A | BTN_B) != 0 {
                    S.shop_step[p] = 2;
                }
            }
        }
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

/// Is slot `s` offstage now: past the ledge of the current stage, or below the floor (V3 2.3).
unsafe fn offstage(s: i32) -> bool {
    let Some(p) = pos(s) else { return false };
    let g = gd_stage_grkind();
    // only the five stages of the rotation have a ledge table; any other stage never counts as offstage
    let Some(i) = STAGES.iter().position(|&k| k == g) else { return false };
    let ledge = LEDGE_X[i];
    p.x.abs() > ledge + 1.0 || p.y < -10.0
}

// ---- statuses (every match slot; on a boss, Chilled and Marked act at half strength)

unsafe fn status_len(s: i32, frames: i32) -> i32 {
    // Vulnerable (a price): statuses on that player last twice as long
    if (0..nh()).contains(&s) && has_price(s as usize, P_VULNERABLE) { frames * 2 } else { frames }
}

unsafe fn burn(s: i32, stacks_to: i32, add: bool, by: i32) {
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
    if by >= 0 {
        S.burn_owner[i] = by;
    }
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
    S.burn_acc[s] = 0;
    S.chill_until[s] = 0;
    S.mark_until[s] = 0;
}

/// Haste on an owner (a player's set, or a foe slot + OWNER_FOE).
unsafe fn hasted(owner: usize) -> bool {
    S.haste_until[owner] > S.play
}

// ---- Heat (V3 3)

/// The set of player `p`'s fighter in play.
unsafe fn in_set(p: usize) -> usize {
    set_of(p, S.active[p] as usize & 1)
}

unsafe fn state_of(p: usize) -> usize {
    heat::state_of(HH.heat[p])
}

unsafe fn add_heat(p: usize, ticks: i64) {
    HH.heat[p] = (HH.heat[p] + ticks).clamp(0, heat::CAP_TICKS);
    let m = heat::milli(HH.heat[p]);
    if m > S.peak[p] {
        S.peak[p] = m;
    }
}

/// The level-scaled multiplier of an xGain drive: the part above x1 scales (x1.5 / x1.8 / x2.2).
fn gain_mul(number: i32, level: i32) -> i32 {
    1000 + scaled(number - 1000, level)
}

/// Whether the xGain drive `c` holds for a record with `tags` by player `p` (from set `set`).
unsafe fn gain_holds(p: usize, set: usize, c: &Drive, tags: i32) -> bool {
    match c.when {
        WH_HASTE => hasted(set),
        WH_LCANCEL_STREAK => S.streak[p] >= 3,
        WH_OFFSTAGE => tags & T_OFFSTAGE != 0,
        WH_AERIAL => tags & T_AERIAL != 0,
        _ => false,
    }
}

/// The gain multiplier for a record (12.2): x1, then each live xGain drive whose condition holds, left to right
/// (the fighter in play's row, then the bench's with Twin Soul), then Glass Core; capped at x4.
unsafe fn gain_now(p: usize, tags: i32) -> (i32, i32) {
    let mut g = 1000;
    let mut src = 0;
    let live = live_sets(p);
    for f in 0..2 {
        if live >> f & 1 == 0 {
            continue;
        }
        let set = set_of(p, f);
        for k in 0..open_slots() {
            let d = S.drives[set][k];
            if d <= 0 {
                continue;
            }
            let c = &POOL[d as usize - 1];
            if c.role == R_GAIN_MUL && gain_holds(p, set, c, tags) {
                g = heat::gain_step(g, gain_mul(c.number, S.levels[set][k]));
                src |= 1 << (f * SLOTS + k);
            }
        }
    }
    if has_key(p, E_GLASS) {
        let n = by_key("glass_core").map_or(2000, |i| POOL[i].number);
        g = heat::gain_step(g, n);
        src |= 1 << 15;
    }
    (g, src)
}

/// Does a damage-charged trigger `c` fill from a record (`tags` on victim `v`)?
unsafe fn source_matches(c: &Drive, v: i32, tags: i32) -> bool {
    match c.charge {
        C_DEALT => true,
        C_AERIAL_DEALT => tags & T_AERIAL != 0,
        C_CHILLED_DEALT => chilled(v),
        C_OFFSTAGE_DEALT => tags & T_OFFSTAGE != 0,
        _ => false,
    }
}

/// The charge rate (x1000): Overclock doubles it.
unsafe fn rate(p: usize) -> i32 {
    if has_key(p, E_OVERCLOCK) { 2000 } else { 1000 }
}

/// The Heat step (V3 3.3, steps 3-9) for one piece of damage player `p` dealt foe `v`: eligible damage at the 150%
/// line, the gain multiplier, Heat, the +Heat drives, charge (`cm`), the tally. `fset` / `fk`: the firing drive.
unsafe fn deal(p: usize, v: i32, dmg: i32, coef: i32, tags: i32, astate: i32, cm: i32, fset: usize, fk: usize) {
    if dmg <= 0 || !(0..4).contains(&v) {
        return;
    }
    let vi = v as usize;
    let no_line = v == S.boss_slot;
    let (g, _) = gain_now(p, tags);
    add_heat(p, heat::heat_gain(dmg, S.foe_m[vi], coef, g, no_line));
    S.foe_m[vi] = S.foe_m[vi].saturating_add(dmg);
    S.tally[p][vi] = S.tally[p][vi].saturating_add(dmg);
    // the +Heat drives on a move: Metronome (a move not among the last three, once per move)
    if cm == CM_MOVE && astate >= AS_ATTACK0 && astate != S.cur_move[p] {
        let fresh = !S.hist[p].contains(&astate);
        S.cur_move[p] = astate;
        S.hist[p] = [astate, S.hist[p][0], S.hist[p][1]];
        if fresh {
            heat_add_drives(p, WH_NEW_MOVE);
        }
    }
    if cm == CM_MOVE && tags & T_AERIAL != 0 {
        S.aerial_dealt[p] = 1;
    }
    // charge
    if cm != CM_NONE {
        let r = rate(p);
        let live = live_sets(p);
        for f in 0..2 {
            if live >> f & 1 == 0 {
                continue;
            }
            let set = set_of(p, f);
            if cm == CM_FIRE && set != fset {
                continue;
            }
            for k in 0..open_slots() {
                if cm == CM_FIRE && k <= fk {
                    continue;
                }
                let d = S.drives[set][k];
                if d <= 0 {
                    continue;
                }
                let c = &POOL[d as usize - 1];
                if c.role != R_TRIGGER || !source_matches(c, v, tags) || (cm == CM_FIRE && c.charge == C_AERIAL_DEALT) {
                    continue;
                }
                // word damage charges only under Plague Bearer, and never Burning's own drives
                if cm == CM_WORD && (c.gives == W_BURNING || c.reads == W_BURNING) {
                    continue;
                }
                HH.chg[set][k] += heat::charge_gain(dmg, coef, r);
                S.target[set][k] = v;
            }
        }
    }
    // keystones on damage: Plague Bearer (a Burning stack per 8% dealt), Winter Oath (a move Chills)
    if cm == CM_MOVE || cm == CM_FIRE {
        if has_key(p, E_PLAGUE) {
            S.burn_acc[vi] += dmg;
            let cap = by_key("plague_bearer").map_or(BURN_STACKS, |i| POOL[i].number).max(if has_inferno(in_set(p)) { INFERNO_STACKS } else { 0 });
            while S.burn_acc[vi] >= 8000 {
                S.burn_acc[vi] -= 8000;
                burn(v, cap, true, p as i32);
            }
        }
        if cm == CM_MOVE && has_key(p, E_WINTER) {
            let ms = by_key("winter_oath").map_or(2000, |i| POOL[i].number);
            chill(v, ms * 60 / 1000);
        }
    }
    if tags & T_OFFSTAGE != 0 {
        if S.play - S.last_off[vi] > 30 {
            ev(EV_SPARK, p as i32, MARK_OFFSTAGE);
        }
        S.last_off[vi] = S.play;
    }
    if cm == CM_MOVE {
        S.last_smash[vi] = if tags & T_SMASH != 0 { p as i32 + 1 } else { 0 };
    }
}

/// Every +Heat drive of player `p`'s live rows whose event is `when` pays, left to right.
unsafe fn heat_add_drives(p: usize, when: u8) {
    let live = live_sets(p);
    for f in 0..2 {
        if live >> f & 1 == 0 {
            continue;
        }
        let set = set_of(p, f);
        for k in 0..open_slots() {
            let d = S.drives[set][k];
            if d <= 0 {
                continue;
            }
            let c = &POOL[d as usize - 1];
            if c.role == R_HEAT_ADD && c.when == when {
                let amt = scaled(c.number, S.levels[set][k]);
                add_heat(p, heat::heat_add(amt));
                ev(EV_HEAT_ADD, p as i32, (k as i32) << 24 | amt);
                log_drive(b"heat add", d as usize - 1, p as i32 * 1000 + amt / 1000);
            }
        }
    }
}

/// Damage player `p` took (V3 3.3's last paragraph): Insulate's allowance, then 1 Heat per 1%; damage-taken
/// triggers charge (`coef`: a foe's fire passes its own).
unsafe fn take(p: usize, dmg: i32, coef: i32, attacker: i32) {
    if dmg <= 0 {
        return;
    }
    let ps = p as i32;
    S.foe_m[p] = S.foe_m[p].saturating_add(dmg);
    let mut paid = dmg;
    if wears(p, E_INSULATE) {
        let free = S.free[p].min(paid).max(0);
        S.free[p] -= free;
        paid -= free;
    }
    let before = HH.heat[p];
    HH.heat[p] = (HH.heat[p] - heat::loss(paid, 1000)).max(0);
    S.loss_acc[p] += heat::milli(before) - heat::milli(HH.heat[p]);
    let live = live_sets(p);
    let r = rate(p);
    for f in 0..2 {
        if live >> f & 1 == 0 {
            continue;
        }
        let set = set_of(p, f);
        for k in 0..open_slots() {
            let d = S.drives[set][k];
            if d > 0 && POOL[d as usize - 1].role == R_TRIGGER && POOL[d as usize - 1].charge == C_TAKEN {
                HH.chg[set][k] += heat::charge_gain(dmg, coef, r);
                if attacker >= 0 && attacker != ps {
                    S.target[set][k] = attacker;
                }
            }
        }
    }
    S.taken += 1;
}

/// Does player `p`'s fighter in play wear a drive with `effect`?
unsafe fn wears(p: usize, effect: u8) -> bool {
    let set = in_set(p);
    S.drives[set][..open_slots()].iter().any(|&d| d > 0 && POOL[d as usize - 1].effect == effect)
}

/// A foe dealt damage (to a player): its damage-charged drives fill.
unsafe fn foe_charge(s: i32, dmg: i32) {
    if !(0..4).contains(&s) {
        return;
    }
    let i = s as usize;
    for k in 0..2 {
        let d = S.fdrives[i][k];
        if d > 0 && POOL[d as usize - 1].charge == C_DEALT {
            HH.fchg[i][k] += heat::charge_gain(dmg, 1000, 1000);
        }
    }
}

// ---- firing

/// Percent on a slot (`milli` x1000; Petey's HP through `damage(amount)` in the engine), as the current fire context
/// says: a player's fire is a record (Heat, charge to its right, the tally); a flare is only percent; a foe's fire
/// costs the player Heat. Returns what was dealt.
unsafe fn hurt(s: i32, milli: i32) -> i32 {
    if milli <= 0 || !alive(s) {
        return 0;
    }
    gd_envoy_percent(s, 0, milli);
    match S.fx_mode {
        FX_FIRE => {
            let set = S.fx_set as usize;
            if set < SETS {
                let tags = T_FIRE | if offstage(s) { T_OFFSTAGE } else { 0 };
                deal(set / 2, s, milli, S.fx_coef, tags, -1, CM_FIRE, set, S.fx_slot.max(0) as usize);
            }
        }
        FX_FOE => {
            if (0..nh()).contains(&s) {
                take(s as usize, milli, S.fx_coef, S.fx_set - OWNER_FOE);
            }
        }
        _ => {
            // the flare: no Heat, no charge, no tally; the sim's own percent still moves
            S.foe_m[s as usize] = S.foe_m[s as usize].saturating_add(milli);
        }
    }
    S.fx_dealt += milli;
    if S.fx_first < 0 {
        S.fx_first = s;
    }
    milli
}

/// The blast of a launching drive (Q6): radius, knockback angle, growth, base, all bigger with Heat. False: no
/// capsule free.
unsafe fn launch(owner: usize, at: P, effect: u8, milli: i32, level: i32, state: usize) -> bool {
    let wide = (if level >= MAX_LEVEL { 1.25 } else { 1.0 }) * (1.0 + 0.2 * state as f32);
    let kb = state as i32 * 6;
    let (r, angle, kbg, bkb) = match effect {
        E_NOVA => (16.0 * wide, 361, 70 + kb, 35 + kb),
        E_SHOCKWAVE => (13.0 * wide, 80, 55 + kb, 30 + kb),
        _ => (9.0 * wide, 90, 75 + kb, 40 + kb), // Spire: straight up
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

/// Every enemy of `owner` within `r` of `at` takes `milli`. The first one struck, or -1.
unsafe fn hurt_around(owner: usize, at: P, r: f32, milli: i32, skip: i32) -> i32 {
    let mut first = -1;
    for s in 0..4 {
        if s != skip && enemy_of(owner, s) && pos(s).is_some_and(|p| d2(p, at) < r * r) {
            hurt(s, milli);
            if first < 0 {
                first = s;
            }
        }
    }
    first
}

/// Fire drive `id` of `owner` at `level` (`frac` x1000 of its number; `target`: the slot its charge last struck).
/// `coef`: the record's coefficient (V3 4.1); `slot`: the drive's slot (charge flows to its right). Returns the log
/// index written.
unsafe fn fire(owner: usize, id: usize, level: i32, target: i32, frac: i32, flags: i32, coef: i32, slot: i32) -> i32 {
    let c = &POOL[id];
    let player = owner < SETS;
    let st = if player { state_of(owner / 2) } else { 0 };
    let inferno = player && c.family == F_RED && has_inferno(owner);
    let n = (heat::fire_damage(c.number, level, st, c.launch != 0) as i64 * frac as i64 / 1000) as i32;
    let me_slot = owner_slot(owner);
    let me = pos(me_slot).unwrap_or(P { x: 0.0, y: 0.0 });
    let mut at = me;
    let mut fl = flags | (st as i32) << FL_STATE_SHIFT | if inferno { FL_INFERNO } else { 0 };
    S.fx_mode = if player { FX_FIRE } else { FX_FOE };
    S.fx_set = owner as i32;
    S.fx_slot = slot;
    S.fx_coef = coef;
    S.fx_dealt = 0;
    S.fx_first = -1;
    let stacks = if inferno { INFERNO_STACKS } else { BURN_STACKS };
    let near = |s: i32| if enemy_of(owner, s) { s } else { nearest(owner, me, -1, 1.0e6) };
    let mut tgt = -1;
    match c.effect {
        E_BURST => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            burn(tgt, stacks, inferno, if player { (owner / 2) as i32 } else { -1 });
            // level 3 or Blazing: the burst splashes (wider when Blazing)
            if level >= MAX_LEVEL || st >= 3 || inferno {
                let r = SPLASH_R * (1.0 + 0.25 * st as f32);
                for s in 0..4 {
                    if s != tgt && enemy_of(owner, s) && pos(s).is_some_and(|p| d2(p, at) < r * r) {
                        hurt(s, n);
                        burn(s, stacks, inferno, if player { (owner / 2) as i32 } else { -1 });
                    }
                }
            }
        }
        E_NOVA | E_SHOCKWAVE => {
            let r = (if c.effect == E_NOVA { NOVA_R } else { SHOCK_R }) * (1.0 + 0.2 * st as f32);
            tgt = nearest(owner, me, -1, r);
            let centre = if c.effect == E_NOVA { P { x: me.x, y: me.y + 8.0 } } else { P { x: me.x, y: me.y + 3.0 } };
            // the blast's damage goes through the take-damage hook (a move record at x1)
            if tgt >= 0 && launch(owner, centre, c.effect, n, level, st) {
                fl |= FL_LAUNCH;
            } else {
                tgt = hurt_around(owner, me, r, n, -1);
            }
        }
        E_SPIRE => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
                if launch(owner, P { x: p.x, y: p.y + 6.0 }, E_SPIRE, n, level, st) {
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
            // it chains: level 3 or Hot to a second foe, Blazing to a third
            let jumps = (level >= MAX_LEVEL) as i32 + (st >= 2) as i32 + (st >= 3) as i32;
            let (mut from, mut prev) = (at, tgt);
            let mut hit = [tgt, -1, -1, -1];
            for j in 0..jumps.min(3) as usize {
                let mut best = -1;
                let mut bd = (BOLT_R * 2.0) * (BOLT_R * 2.0);
                for s in 0..4 {
                    if hit.contains(&s) || !enemy_of(owner, s) {
                        continue;
                    }
                    if let Some(q) = pos(s) {
                        if d2(q, from) < bd {
                            bd = d2(q, from);
                            best = s;
                        }
                    }
                }
                if best < 0 || best == prev {
                    break;
                }
                hurt(best, n);
                hit[j + 1] = best;
                prev = best;
                from = pos(best).unwrap_or(from);
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
            hurt(tgt, n);
            chill(tgt, CHILL_LEN + 60 * (level - 1) + 60 * st as i32);
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
            // Blazing: the shards hit every other Chilled foe too
            if st >= 3 {
                for s in 0..4 {
                    if s != tgt && enemy_of(owner, s) && chilled(s) {
                        hurt(s, n);
                        S.chill_until[s as usize] = 0;
                    }
                }
            }
            fl |= FL_LINK;
        }
        E_MARK => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            mark(tgt, MARK_LEN);
        }
        E_THORNS => {
            // the attacker that charged it (its last hit), else the nearest enemy
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            hurt(tgt, n);
            mark(tgt, MARK_LEN);
            fl |= FL_LINK;
        }
        E_HEAL => {
            gd_envoy_percent(me_slot, 0, -n);
        }
        E_AIRJUMP => {
            if player && gd_envoy_air_jump(me_slot) != 0 {
                S.banked_jump[owner / 2] = 1; // no jump used yet: kept for when one is
            }
        }
        E_FEATHERS => {
            // three feathers, four when Hot, five when Blazing; each at a different foe when there are enough
            let count = 3 + (st >= 2) as i32 + (st >= 3) as i32;
            tgt = nearest(owner, me, -1, BOLT_R * 4.0);
            if let Some(p) = pos(tgt) {
                at = p;
            }
            let mut foes = [-1i32; 4];
            let mut nf = 0;
            for s in 0..4 {
                if enemy_of(owner, s) {
                    foes[nf] = s;
                    nf += 1;
                }
            }
            hurt(tgt, n);
            for k in 1..count {
                let t = if nf > 1 { foes[(k as usize) % nf] } else { tgt };
                pend(PK_PULSE, S.play + k * FEATHER_GAP, id, owner, level, n, t, 0, coef, slot);
            }
        }
        E_WILDFIRE => {
            for s in 0..4 {
                if s != target && enemy_of(owner, s) {
                    hurt(s, n);
                    burn(s, stacks, inferno, if player { (owner / 2) as i32 } else { -1 });
                    if tgt < 0 {
                        tgt = s;
                    }
                }
            }
            fl |= FL_LINK;
        }
        E_SURGE => {
            if player {
                for k in 0..open_slots() {
                    let d = S.drives[owner][k];
                    if d > 0 && POOL[d as usize - 1].role == R_TRIGGER && POOL[d as usize - 1].effect != E_SURGE {
                        HH.chg[owner][k] = HH.chg[owner][k].max(need(&POOL[d as usize - 1]));
                    }
                }
            }
        }
        E_RELAY => {
            // every other trigger of this row fires, left to right, 6 frames apart (a chain with no link limit)
            if player {
                start_chain(owner, 0, true);
            }
        }
        _ => {}
    }
    S.fx_mode = FX_FLARE;
    S.fx_set = -1;
    S.fires += 1;
    let li = S.log_n as usize % LOG;
    let dealt = if fl & FL_LAUNCH != 0 { n } else { S.fx_dealt };
    S.log[li] = [S.play, id as i32 + 1, owner as i32, if tgt >= 0 { tgt } else { S.fx_first }, x100(at.x), x100(at.y), level | dealt.max(0) << 4, fl];
    S.log_n += 1;
    log_drive(if fl & FL_ECHO != 0 { b"drive echo" } else { b"drive fire" }, id, owner as i32 * 10 + level);
    if player && fl & (FL_ECHO | FL_PASSIVE) == 0 {
        let p = owner / 2;
        if has_key(p, E_ECHO) && c.effect != E_RELAY {
            let half = by_key("echo_oath").map_or(500, |i| POOL[i].number);
            pend(PK_ECHO, S.play + ECHO_DELAY, id, owner, level, half, tgt, FL_ECHO, coef, slot);
        }
        if has_price(p, P_BLEED) {
            // Bleed: each fire costs the fighter in play 1%
            gd_envoy_percent(p as i32, 0, 1000);
        }
    }
    li as i32
}

unsafe fn pend(kind: i32, at: i32, id: usize, owner: usize, level: i32, frac: i32, target: i32, flags: i32, coef: i32, slot: i32) {
    if let Some(q) = S.pend.iter_mut().find(|q| q[1] == 0) {
        *q = [at, kind, id as i32 + 1, owner as i32, level, frac, target, flags, coef, slot];
    }
}

/// What a full charge is, in the drive's own units: time in frames, damage in charge units, an event 1.
fn need(c: &Drive) -> i64 {
    match c.charge {
        C_TIME => (c.need as i64 * 60 / 1000).max(1),
        C_TAG_IN | C_SHIFT | C_CASHOUT => 1,
        _ => c.need.max(1) as i64 * CHARGE_UNIT,
    }
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

unsafe fn is_trigger(set: usize, k: usize) -> bool {
    let d = S.drives[set][k];
    d > 0 && POOL[d as usize - 1].role == R_TRIGGER
}

unsafe fn full(set: usize, k: usize) -> bool {
    is_trigger(set, k) && HH.chg[set][k] >= need(&POOL[S.drives[set][k] as usize - 1])
}

/// Charge an event trigger (tag-in, a shift, a cash-out) of player `p`'s live rows: it is full now.
unsafe fn charge_event(p: usize, src: u8, target: i32, burning_foe: bool) {
    let live = live_sets(p);
    for f in 0..2 {
        if live >> f & 1 == 0 {
            continue;
        }
        let set = set_of(p, f);
        for k in 0..open_slots() {
            let d = S.drives[set][k];
            if d <= 0 {
                continue;
            }
            let c = &POOL[d as usize - 1];
            if c.role == R_TRIGGER && c.charge == src && (c.when != WH_BURNING_FOE || burning_foe) {
                HH.chg[set][k] = need(c);
                if target >= 0 {
                    S.target[set][k] = target;
                }
            }
        }
    }
}

/// Start a chain on `set` at the first trigger at or after slot `from` that is full (or any trigger: Volley).
unsafe fn start_chain(set: usize, from: usize, volley: bool) {
    let n = open_slots();
    let Some(k) = (from..n).find(|&k| if volley { is_trigger(set, k) && !matches!(POOL[S.drives[set][k] as usize - 1].effect, E_RELAY) } else { full(set, k) }) else {
        return;
    };
    S.ch_on[set] = 1;
    S.ch_k[set] = k as i32;
    S.ch_link[set] = 1;
    S.ch_coef[set] = 1000;
    S.ch_next[set] = S.play;
    S.ch_volley[set] = volley as i32;
    S.ch_log[set] = -1;
}

/// One set's chain this frame (V3 4.1): start at the leftmost full trigger when idle and 20 frames have passed; fire
/// a due link; queue the next full trigger to its right, 6 frames on, up to 4 links (Volley: every trigger, no limit).
unsafe fn chain_frame(set: usize) {
    if S.ch_on[set] == 0 {
        if S.play - S.ch_end[set] >= CHAIN_GAP {
            start_chain(set, 0, false);
        }
        if S.ch_on[set] == 0 {
            return;
        }
    }
    if S.play < S.ch_next[set] {
        return;
    }
    let k = S.ch_k[set].clamp(0, SLOTS as i32 - 1) as usize;
    let volley = S.ch_volley[set] != 0;
    if is_trigger(set, k) {
        let d = S.drives[set][k] as usize - 1;
        let c = &POOL[d];
        let nd = need(c);
        if HH.chg[set][k] >= nd {
            HH.chg[set][k] -= nd; // the remainder stays (12.2)
            if matches!(c.charge, C_TAG_IN | C_SHIFT | C_CASHOUT) {
                HH.chg[set][k] = 0;
            }
        }
        let coef = heat::link_coef(c.coef, S.ch_coef[set]);
        let link = S.ch_link[set].min(15);
        let parent = if S.ch_log[set] >= 0 { S.ch_log[set] + 1 } else { 0 };
        let fl = (link << FL_LINK_SHIFT) | (parent.min(15) << FL_PARENT_SHIFT) | if volley { FL_VOLLEY } else { 0 };
        let li = fire(set, d, S.levels[set][k], S.target[set][k], 1000, fl, coef, k as i32);
        S.chain_n += 1;
        log(b"envoy: chain link, set*100 + link ", set as i32 * 100 + link);
        if !volley && S.ch_volley[set] != 0 {
            return; // Relay started a volley on this row: it runs from here
        }
        S.ch_log[set] = li;
        S.ch_coef[set] = coef.max(1);
        // Echo on its left: this drive fires again 6 frames later at Echo's strength (not a link)
        if k > 0 {
            let e = S.drives[set][k - 1];
            if e > 0 && POOL[e as usize - 1].role == R_NEIGHBOUR && c.effect != E_RELAY {
                let frac = scaled(POOL[e as usize - 1].number, S.levels[set][k - 1]);
                pend(PK_ECHO, S.play + LINK_GAP, d, set, S.levels[set][k], frac, S.target[set][k], FL_ECHO, coef, k as i32);
            }
        }
        S.last_fire_at[set] = S.play;
    }
    // the next link: the leftmost trigger to the right that is full (Volley: any)
    let n = open_slots();
    let next = (k + 1..n).find(|&j| if volley { is_trigger(set, j) && POOL[S.drives[set][j] as usize - 1].effect != E_RELAY } else { full(set, j) });
    match next {
        Some(j) if volley || S.ch_link[set] < CHAIN_LINKS => {
            S.ch_k[set] = j as i32;
            S.ch_link[set] += 1;
            S.ch_next[set] = S.play + LINK_GAP;
        }
        _ => {
            S.ch_on[set] = 0;
            S.ch_volley[set] = 0;
            S.ch_end[set] = S.play;
        }
    }
}

/// Per unfrozen live frame: time charges, each live row's chain, the foes' fires (one per 20 frames), the pending
/// repeats and pulses, Updraft's banked jump.
unsafe fn drives_frame() {
    for p in 0..nh() as usize {
        if !alive(p as i32) {
            continue;
        }
        let live = live_sets(p);
        for f in 0..2 {
            if live >> f & 1 == 0 || (S.shared != 0 && f == 1) {
                continue;
            }
            let set = set_of(p, f);
            for k in 0..open_slots() {
                let d = S.drives[set][k];
                if d > 0 && POOL[d as usize - 1].role == R_TRIGGER && POOL[d as usize - 1].charge == C_TIME {
                    HH.chg[set][k] = (HH.chg[set][k] + 1).min(need(&POOL[d as usize - 1]) * 2);
                }
            }
            chain_frame(set);
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
        for k in 0..2 {
            let d = S.fdrives[i][k];
            if d > 0 && POOL[d as usize - 1].charge == C_TIME {
                HH.fchg[i][k] += 1;
            }
        }
        let owner = OWNER_FOE as usize + i;
        if S.play - S.last_fire_at[owner] >= CHAIN_GAP {
            for k in 0..2 {
                let d = S.fdrives[i][k];
                if d > 0 && HH.fchg[i][k] >= need(&POOL[d as usize - 1]) {
                    S.last_fire_at[owner] = S.play;
                    HH.fchg[i][k] = 0;
                    let c = &POOL[d as usize - 1];
                    fire(owner, d as usize - 1, S.flevels[i][k], S.ftarget[i][k], 1000, 0, c.coef.max(300), -1);
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
        S.pend[q] = [0; 10];
        let owner = e[3] as usize;
        if e[1] == PK_ECHO {
            fire(owner, e[2] as usize - 1, e[4], e[6], e[5], e[7], e[8], e[9]);
        } else if enemy_of(owner, e[6]) {
            // a feather: a piece of the fire's damage, a record of its own
            S.fx_mode = if owner < SETS { FX_FIRE } else { FX_FOE };
            S.fx_set = owner as i32;
            S.fx_slot = e[9];
            S.fx_coef = e[8];
            hurt(e[6], e[5]);
            S.fx_mode = FX_FLARE;
            S.fx_set = -1;
        }
    }
}

// ---- the take-damage hooks (run inside the sim's frame: they re-run in a resimulation)

unsafe fn in_play() -> bool {
    S.live != 0 && S.state == RUN_LIVE && S.rw_active == 0 && S.outcome == 0 && S.attrs_done != 0
}

/// The owner index of the fighter in play at `slot` (a player's set, or a foe).
unsafe fn owner_at(slot: i32) -> usize {
    if (0..nh()).contains(&slot) { in_set(slot as usize) } else { OWNER_FOE as usize + slot as usize }
}

/// damage.fighter_take_damage pre: Bulwark's orb absorbs a hit on the fighter that armed it: the percent is skipped
/// and the knockback goes too.
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

unsafe fn push_rec(a: i32, v: i32, dmg: i32, tags: i32, astate: i32) {
    if S.rec_n < RECS as i32 {
        S.rec[S.rec_n as usize] = [a, v, dmg, tags, astate];
        S.rec_n += 1;
    } else if let Some(r) = S.rec[..RECS].iter_mut().rev().find(|r| r[0] == a && r[1] == v) {
        // the buffer is full: the totals are kept on the last record of the same pair (only its tags are lost)
        r[2] = r[2].saturating_add(dmg);
    }
}

/// damage.fighter_take_damage post (V3 12.4): one damage record (the attacker, the victim, the damage x1000 rounded
/// once here, the tags that only exist now). Marked adds its extra as word damage. Grand Finale's knockback write.
extern "C" fn on_damage(h: u32, d: f32) {
    unsafe {
        if !in_play() || d <= 0.0 {
            return;
        }
        let v = gd_fighter_slot(h);
        if !(0..4).contains(&v) {
            return;
        }
        let dmg = (d * 1000.0 + 0.5) as i32;
        let mut a = gd_envoy_attacker(v);
        if a < 0 || a == v {
            a = -1; // no fighter source: a hazard (or the victim itself)
        }
        let mut tags = 0;
        let mut ast = -1;
        if a >= 0 {
            let ha = gd_fighter_from_slot(a);
            if ha != 0 {
                ast = gd_fighter_get_action_state(ha);
                if (AS_AERIAL.0..=AS_AERIAL.1).contains(&ast) {
                    tags |= T_AERIAL;
                }
                if (AS_SMASH.0..=AS_SMASH.1).contains(&ast) {
                    tags |= T_SMASH;
                }
            }
        }
        let player_a = (0..nh()).contains(&a);
        if S.as_log != 0 && player_a {
            log(b"envoy: hit, victim*1000 + attacker state ", v * 1000 + ast);
        }
        if player_a && is_foe(v) && offstage(v) {
            tags |= T_OFFSTAGE;
        }
        // Grand Finale: a player's smash launches harder with Heat (+1% per 4.5 Heat, to +50%)
        // (test knob `finaleall`: every player hit counts as a smash, so the probe needs no scripted smash to land)
        if player_a && (tags & T_SMASH != 0 || S.finale_off & 2 != 0) && is_foe(v) && has_key(a as usize, E_FINALE) {
            let bonus = (HH.heat[a as usize] / FINALE_TICKS).min(500) as i32;
            if bonus > 0 {
                if S.finale_off & 1 == 0 {
                    gd_envoy_kb_scale(v, 1000 + bonus);
                }
                S.kb_watch = v;
                S.kb_watch_at = S.play;
                log(b"envoy: finale knockback x1000 ", 1000 + bonus);
            }
        }
        push_rec(a, v, dmg, tags, ast);
        // Marked: the hit deals more (half on a boss): word damage for the attacker
        if marked(v) {
            let more = if is_boss(v) { BOSS_MARK_MORE } else { MARK_MORE };
            let extra = (dmg as i64 * more as i64 / 1000) as i32;
            if extra > 0 {
                gd_envoy_percent(v, 0, extra);
                push_rec(a, v, extra, T_WORD, -1);
            }
        }
    }
}

/// The frame's records, in buffer order (V3 12.4 step 3).
unsafe fn records_frame() {
    for i in 0..(S.rec_n as usize).min(RECS) {
        let [a, v, dmg, tags, ast] = S.rec[i];
        let human_v = (0..nh()).contains(&v);
        if human_v {
            if (0..nh()).contains(&a) {
                continue; // friendly: nothing (9.6)
            }
            take(v as usize, dmg, 1000, a);
            if is_foe(a) {
                foe_charge(a, dmg);
                for k in 0..2 {
                    S.ftarget[a as usize][k] = v;
                }
            }
            continue;
        }
        if (0..nh()).contains(&a) && is_foe(v) {
            S.hits += 1;
            let p = a as usize;
            if tags & T_WORD != 0 {
                deal(p, v, dmg, WORD_COEF[0], tags, -1, CM_NONE, 0, 0);
            } else {
                deal(p, v, dmg, 1000, tags, ast, CM_MOVE, 0, 0);
            }
        } else {
            // self, friendly, a hazard on a foe: only the sim's own percent moves
            S.foe_m[v as usize] = S.foe_m[v as usize].saturating_add(dmg);
        }
    }
    S.rec_n = 0;
}

/// A frozen frame: the records' damage is already on the fighters; only the sim's own percent follows.
unsafe fn records_drop() {
    for i in 0..(S.rec_n as usize).min(RECS) {
        let v = S.rec[i][1];
        if (0..4).contains(&v) {
            S.foe_m[v as usize] = S.foe_m[v as usize].saturating_add(S.rec[i][2]);
        }
    }
    S.rec_n = 0;
}

/// The launch after a Grand Finale hit (or the hit it would have scaled, `finaleoff`): the victim's knockback speed
/// x100 on the next frame, logged (the A/B probe of the knockback write).
unsafe fn finale_probe() {
    if S.kb_watch < 0 || S.play <= S.kb_watch_at {
        return;
    }
    let h = gd_fighter_from_slot(S.kb_watch);
    if h != 0 {
        let mut w = [0u32; 20];
        gd_fighter_read(h, w.as_mut_ptr());
        let (x, y) = (f32::from_bits(w[6]), f32::from_bits(w[7]));
        log(if S.finale_off & 1 != 0 { b"envoy: finale probe (write off), kb speed^2 x10000 " } else { b"envoy: finale probe (write on), kb speed^2 x10000 " }, ((x * x + y * y) * 10000.0) as i32);
    }
    S.kb_watch = -1;
}

// ---- technique (V3 2.3): the L-cancel bit, wavedash and tech polled; marks count down

unsafe fn tech_frame() {
    for p in 0..nh() as usize {
        let ev_bits = gd_envoy_events(p as i32);
        for m in S.marks[p].iter_mut() {
            if *m > 0 {
                *m -= 1;
            }
        }
        let h = gd_fighter_from_slot(p as i32);
        let st = if h != 0 { gd_fighter_get_action_state(h) } else { -1 };
        let prev = S.prev_as[p];
        if st != prev {
            S.cur_move[p] = -1;
        }
        if ev_bits & 1 != 0 {
            S.marks[p][0] = MARK_FRAMES[0];
            ev(EV_SPARK, p as i32, MARK_LCANCEL);
            log(b"envoy: technique l-cancel, player\0", p as i32);
            if S.aerial_dealt[p] != 0 {
                heat_add_drives(p, WH_LCANCEL);
            }
        }
        if st == AS_ESCAPE_AIR {
            S.escape_at[p] = S.play;
        }
        if st != prev && S.as_log != 0 && p == 0 {
            log(b"envoy: as ", prev * 1000 + st);
        }
        if st != prev {
            if (AS_LANDING_AIR.0..=AS_LANDING_AIR.1).contains(&st) {
                S.streak[p] = if ev_bits & 1 != 0 { (S.streak[p] + 1).min(3) } else { 0 };
            }
            // a wavedash / waveland: LandingFallSpecial entered within a few frames of an air dodge
            if st == AS_LANDING_SPECIAL && (prev == AS_ESCAPE_AIR || S.play - S.escape_at[p] <= 3) {
                S.marks[p][1] = MARK_FRAMES[1];
                ev(EV_SPARK, p as i32, MARK_WAVEDASH);
                log(b"envoy: technique wavedash, player\0", p as i32);
            }
            if (AS_TECH.0..=AS_TECH.1).contains(&st) {
                S.marks[p][2] = MARK_FRAMES[2];
                ev(EV_SPARK, p as i32, MARK_TECH);
                log(b"envoy: technique tech, player\0", p as i32);
                heat_add_drives(p, WH_TECH);
            }
            if AS_LANDING.iter().any(|r| (r.0..=r.1).contains(&st)) {
                S.aerial_dealt[p] = 0;
            }
        }
    }
}

// ---- moments

unsafe fn on_tag(p: usize) {
    let out = S.active[p] as usize;
    let inn = 1 - out;
    S.active[p] = inn as i32;
    log(b"envoy: moment tag in, player*10 + fighter\0", p as i32 * 10 + inn as i32);
    charge_event(p, C_TAG_IN, -1, false);
}

/// What a foe pays on a cash-out (x1000; V3 3.5).
unsafe fn pay_of(s: i32, last: bool) -> i32 {
    match sched().kind {
        FK_PACK => PAY_MINION,
        FK_ELITE => PAY_ELITE,
        FK_BOSS => {
            if last { PAY_LAST } else { PAY_BREAK }
        }
        _ => {
            let _ = s;
            PAY_FIGHTER
        }
    }
}

/// Player `p` cashes out on foe `s` (V3 3.5, 12.4 step 8): coins into the purse, pips, the flare on every other
/// foe; Wildfire; then half kept (Grand Finale's smash KO: none, and it pays half again).
unsafe fn cash_out(p: usize, s: i32, pay: i32, finale: bool) {
    let h = HH.heat[p];
    let pay = if finale { pay * 3 / 2 } else { pay };
    let c = heat::cash(h, pay);
    let coins_before = coins();
    let pips_before = HH.pips[p];
    HH.purse += c;
    HH.earned[p] += c;
    HH.pips[p] += c;
    let paid = coins() - coins_before;
    ev(EV_CASHOUT, p as i32, heat::milli(h));
    ev(EV_PAY, p as i32, paid);
    ev(EV_PIP, p as i32, ((HH.pips[p] - pips_before) / (PIP_UNIT / 1000)) as i32);
    S.cashouts[p] += 1;
    log(b"envoy: cash-out, heat\0", heat::milli(h) / 1000);
    log(b"envoy: cash-out, coins paid\0", paid);
    // the flare: Heat / 15 percent to every other foe in play, no launch; it feeds nothing
    let fl = heat::flare(h, pay);
    if fl > 0 {
        S.fx_mode = FX_FLARE;
        for t in 0..4 {
            if t != s && is_foe(t) && alive(t) {
                hurt(t, fl);
                ev(EV_FLARE, t, fl);
                log(b"envoy: flare, slot*100000 + percent x1000 ", t * 100_000 + fl);
                let at = pos(t).unwrap_or(P { x: 0.0, y: 0.0 });
                let li = S.log_n as usize % LOG;
                let first = S.drives[in_set(p)][0];
                S.log[li] = [S.play, first.max(0), in_set(p) as i32, t, x100(at.x), x100(at.y), 1 | fl << 4, FL_FLARE | (state_of(p) as i32) << FL_STATE_SHIFT];
                S.log_n += 1;
            }
        }
    }
    // Wildfire and every cash-out trigger: full now (Wildfire only when the foe was Burning)
    charge_event(p, C_CASHOUT, s, burning(s));
    HH.heat[p] = if finale { 0 } else { HH.heat[p] / 2 };
    S.free[p] = INSULATE_FREE;
}

/// A foe lost a stock (or a boss broke): every player with 10% on it this stock cashes out, player 1 first.
unsafe fn foe_ko(s: i32, last: bool) {
    let i = s as usize;
    let pay = pay_of(s, last);
    for p in 0..nh() as usize {
        if S.tally[p][i] < CASHOUT_MIN {
            continue;
        }
        let finale = S.last_smash[i] == p as i32 + 1 && has_key(p, E_FINALE);
        cash_out(p, s, pay, finale);
        if S.play - S.last_off[i] <= EDGEGUARD_WINDOW {
            HH.purse += COINS_EDGEGUARD as i64 * COIN_UNIT;
            HH.earned[p] += COINS_EDGEGUARD as i64 * COIN_UNIT;
            ev(EV_COINS, p as i32, COINS_EDGEGUARD);
            log(b"envoy: edgeguard KO, coins\0", COINS_EDGEGUARD);
        }
    }
    for p in 0..NP {
        S.tally[p][i] = 0;
    }
    S.foe_m[i] = 0;
    S.last_off[i] = -9999;
    S.last_smash[i] = 0;
    if S.drops_made < S.drops_due {
        S.drops_made += 1;
        for q in 0..nh() as usize {
            roll_drop(q);
        }
    }
}

/// Per unfrozen live frame: the stock edges (KOs dealt and cash-outs, stocks lost, the shared pool), the boss's
/// damage and breaks, the drops, Burning, Volley.
unsafe fn moments_frame() {
    let mut foes = 0;
    for s in 0..4 {
        let n = if (0..nh() + ncpu()).contains(&s) { gd_player_stocks(s).max(0) } else { 0 };
        let i = s as usize;
        if n < S.stocks_each[i] {
            if is_foe(s) && s != S.boss_slot {
                foe_ko(s, n == 0);
            } else if (0..nh()).contains(&s) {
                // a lost stock halves Heat
                HH.heat[i] /= 2;
                S.stock_lost = 1;
                ev(EV_HALVED, s, heat::milli(HH.heat[i]));
                S.foe_m[i] = 0;
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
    // the boss with HP (Petey): each player's damage is a record; each quarter of HP is a break (a cash-out)
    if S.boss_slot >= 0 {
        let mut w = [0u32; 6];
        gd_envoy_boss_dmg(w.as_mut_ptr());
        let b = S.boss_slot;
        for p in 0..nh() as usize {
            let now = w[p] as i32;
            let d = now.wrapping_sub(S.boss_dmg[p]).clamp(0, 200_000);
            S.boss_dmg[p] = now;
            if d > 0 {
                S.hits += 1;
                deal(p, b, d, 1000, 0, -1, CM_MOVE, 0, 0);
            }
        }
        S.boss_hp = w[4] as i32;
        S.boss_hp_max = w[5] as i32;
        if S.boss_hp_max > 0 {
            let q = ((S.boss_hp_max - S.boss_hp.max(0)) as i64 * 4 / S.boss_hp_max as i64).clamp(0, 4) as i32;
            while S.boss_breaks < q {
                S.boss_breaks += 1;
                ev(EV_BREAK, b, S.boss_breaks);
                log(b"envoy: boss break\0", S.boss_breaks);
                let i = b as usize;
                let pay = if S.boss_breaks >= 4 { PAY_LAST } else { PAY_BREAK };
                for p in 0..nh() as usize {
                    if S.tally[p][i] >= CASHOUT_MIN {
                        cash_out(p, b, pay, false);
                    }
                    S.tally[p][i] = 0;
                }
            }
        }
    }
    drop_frame();
    // Burning: 1.5% a second per stack; word damage for the player who added the last stack (V3 12.4 step 5)
    for i in 0..4 {
        if S.burn_stacks[i] > 0 && S.play >= S.burn_until[i] {
            S.burn_stacks[i] = 0;
        }
        if S.burn_stacks[i] > 0 && S.play >= S.burn_next[i] {
            S.burn_next[i] += BURN_TICK;
            let s = i as i32;
            if alive(s) {
                let d = BURN_TICK_PCT * S.burn_stacks[i];
                gd_envoy_percent(s, 0, d);
                let p = S.burn_owner[i];
                if (0..nh()).contains(&p) && is_foe(s) {
                    let pu = p as usize;
                    let inf = has_inferno(in_set(pu));
                    let cm = if has_key(pu, E_PLAGUE) { CM_WORD } else { CM_NONE };
                    deal(pu, s, d, WORD_COEF[inf as usize], T_WORD, -1, cm, 0, 0);
                } else if (0..nh()).contains(&s) {
                    take(i, d, WORD_COEF[0], -1);
                } else {
                    S.foe_m[i] = S.foe_m[i].saturating_add(d);
                }
            }
        }
    }
    // Volley: D-pad right, 3 pips: every trigger of the fighter in play fires now, left to right (12.6)
    for p in 0..nh() as usize {
        if S.act_id[p] == ACTIVE_VOLLEY && HH.pips[p] >= VOLLEY_PIPS * PIP_UNIT && alive(p as i32) && gd_pad_pressed(p as i32) & DPAD_RIGHT != 0 {
            let set = in_set(p);
            if (0..open_slots()).any(|k| is_trigger(set, k)) {
                HH.pips[p] -= VOLLEY_PIPS * PIP_UNIT;
                start_chain(set, 0, true);
                ev(EV_ACTIVE, p as i32, ACTIVE_VOLLEY);
                log(b"envoy: volley, player\0", p as i32);
            }
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
                def = def * GLASS_TAKE / 1000;
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
    S.boss_slot = gd_envoy_boss_dmg(w.as_mut_ptr());
    for p in 0..4 {
        S.boss_dmg[p] = w[p] as i32;
    }
    S.boss_hp = w[4] as i32;
    S.boss_hp_max = w[5] as i32;
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
            charge_event(p, C_SHIFT, -1, false);
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

/// The fight is won (the last foe fell; `S.won_pool` is the pool left): sweep the drops, pay the clear, advance.
unsafe fn fight_won() {
    let me = S.won_pool.max(1);
    for p in 0..nh() as usize {
        for k in 0..DROPS {
            if S.drop[p][k][0] == 1 {
                take_drop(p, k, G_SWEPT);
            }
        }
    }
    // coins: a fight cleared, and more with no stock lost (V3 7.3)
    let c = COINS_CLEAR + if S.stock_lost == 0 { COINS_CLEAN } else { 0 };
    HH.purse += c as i64 * COIN_UNIT;
    for p in 0..nh() as usize {
        HH.earned[p] += c as i64 * COIN_UNIT / nh() as i64;
        ev(EV_COINS, p as i32, c);
    }
    log(b"envoy: fight cleared, coins\0", c);
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

/// Per live unfrozen frame, the Heat states (events on a change) and the frame's loss event.
unsafe fn heat_events() {
    for p in 0..nh() as usize {
        let s = state_of(p) as i32;
        if s != S.hstate[p] {
            ev(if s > S.hstate[p] { EV_STATE_UP } else { EV_STATE_DOWN }, p as i32, s);
            log(b"envoy: heat state, player*10 + state\0", p as i32 * 10 + s);
            S.hstate[p] = s;
        }
        if S.loss_acc[p] > 0 {
            ev(EV_LOSS, p as i32, S.loss_acc[p]);
            S.loss_acc[p] = 0;
        }
    }
}

/// FNV-1a over the whole state, the 64-bit half and the pool digest: this frame's word for the compared netplay /
/// synctest hash.
unsafe fn digest() -> u32 {
    let b = core::slice::from_raw_parts(core::ptr::addr_of!(S) as *const u8, core::mem::size_of::<St>());
    let h = core::slice::from_raw_parts(core::ptr::addr_of!(HH) as *const u8, core::mem::size_of::<H>());
    let p = core::slice::from_raw_parts(PLAN.as_ptr() as *const u8, PLAN.len() * 4);
    fnv(fnv(fnv(fnv(0x811C_9DC5, &POOL_DIGEST.to_le_bytes()), b), h), p)
}

fn fbits(v: f32) -> i32 {
    v.to_bits() as i32
}

fn left(until: i32, now: i32) -> i32 {
    (until - now).max(0)
}

/// The view (envoy_data::view; docs/rust-port/notes/envoy-v3-view.md).
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
    v[V_RW_TIME] = if S.rw_active == RW_SHOP { SHOP_TIME } else { REWARD_TIME };
    let frozen = (gd_transition_phase() != 0 || S.rw_active != 0 || S.collect > 0) as i32;
    v[V_FROZEN] = frozen;
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
    v[V_COINS] = coins();
    v[V_EV_N] = S.ev_n;
    v[V_REROLL_COST] = reroll_cost();
    v[V_CHAIN_N] = S.chain_n;
    v[V_HEAT_ON] = 1;
    v[V_SHOP_DUE] = S.shop_due;
    let n_open = open_slots();
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
                v[b + P_CHARGE + f * SLOTS + k] = if d > 0 {
                    let c = &POOL[d as usize - 1];
                    if c.role == R_TRIGGER {
                        (HH.chg[set][k].min(need(c)) * 1000 / need(c).max(1)) as i32
                    } else if c.effect == E_INSULATE {
                        S.free[p] * 1000 / INSULATE_FREE
                    } else {
                        0
                    }
                } else {
                    0
                };
            }
            v[b + P_HASTE + f] = left(S.haste_until[set], S.play);
            v[b + P_SHIELD + f] = S.shield[set];
            v[b + P_SET + f] = if has_inferno(set) { SET_INFERNO } else { 0 };
        }
        v[b + P_LIVE_SETS] = live_sets(p) as i32;
        v[b + P_RW_DONE] = S.rw_done[p];
        v[b + P_CURSOR] = S.cursor[p];
        v[b + P_STEP] = S.step[p];
        v[b + P_FCURSOR] = S.fcursor[p];
        v[b + P_SCURSOR] = S.scursor[p];
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
        // Heat
        v[b + P_HEAT] = heat::milli(HH.heat[p]);
        v[b + P_HEAT_CAP] = (HEAT_CAP * 1000) as i32;
        for m in 0..3 {
            v[b + P_HEAT_MARKS + m] = (HEAT_MARKS[m] * 1000) as i32;
        }
        let st = state_of(p);
        v[b + P_HEAT_STATE] = st as i32;
        v[b + P_HEAT_FULL] = (HH.heat[p] >= heat::CAP_TICKS) as i32;
        v[b + P_HEAT_FROZEN] = frozen;
        let (g, src) = gain_now(p, 0);
        v[b + P_GAIN_MUL] = g;
        v[b + P_GAIN_SRC] = src;
        let ins = wears(p, E_INSULATE);
        v[b + P_FREE] = if ins { S.free[p] } else { 0 };
        v[b + P_FREE_MAX] = if ins { INSULATE_FREE } else { 0 };
        v[b + P_ACT_ID] = S.act_id[p];
        v[b + P_PIPS] = (HH.pips[p] / PIP_UNIT) as i32;
        v[b + P_PIPS_NEED] = VOLLEY_PIPS as i32;
        v[b + P_PIP_PART] = (HH.pips[p] % PIP_UNIT / (PIP_UNIT / 1000)) as i32;
        for m in 0..3 {
            v[b + P_MARKS + m] = S.marks[p][m];
        }
        v[b + P_EARNED] = (HH.earned[p] / COIN_UNIT) as i32;
        v[b + P_HEAT_SCALE] = STATE_SCALE[st];
        v[b + P_LCANCEL_STREAK] = S.streak[p];
        for s in 0..4 {
            v[b + P_TALLY + s] = S.tally[p][s];
        }
        v[b + P_ACT_READY] = (S.act_id[p] != 0 && HH.pips[p] >= VOLLEY_PIPS * PIP_UNIT) as i32;
        let set = in_set(p);
        v[b + P_CHAIN] = if S.ch_on[set] != 0 { S.ch_link[set] } else { 0 };
        v[b + P_HEAT_PEAK] = S.peak[p];
        v[b + P_CASHOUTS] = S.cashouts[p];
        // the shop
        let sf = if S.shared != 0 { 0 } else { S.shop_fighter[p].clamp(0, 1) as usize };
        let sset = set_of(p, sf);
        for i in 0..3 {
            let id = S.shop_off[p][i];
            v[b + P_SHOP_OFFERS + i] = id;
            v[b + P_SHOP_PRICE + i] = if id >= 0 { price_of(id) } else { 0 };
        }
        v[b + P_SHOP_CURSOR] = S.shop_cursor[p];
        v[b + P_SHOP_STEP] = S.shop_step[p];
        v[b + P_SHOP_FIGHTER] = sf as i32;
        v[b + P_SHOP_DONE] = if S.rw_active == RW_SHOP { S.rw_done[p] } else { 0 };
        v[b + P_SHOP_MSG] = S.shop_msg[p];
        for k in 0..SLOTS {
            v[b + P_SHOP_LEVEL_PRICE + k] = if k < n_open { level_price(sset, k) } else { 0 };
        }
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
            v[b + S_CHARGE + k] = if d > 0 { (HH.fchg[i][k].min(need(&POOL[d as usize - 1])) * 1000 / need(&POOL[d as usize - 1]).max(1)) as i32 } else { 0 };
        }
        v[b + S_HASTE] = left(S.haste_until[OWNER_FOE as usize + i], S.play);
        v[b + S_SHIELD] = S.shield[OWNER_FOE as usize + i];
        v[b + S_PCT] = S.foe_m[i];
        if s == S.boss_slot {
            v[b + S_BREAKS] = S.boss_breaks;
            v[b + S_HP] = S.boss_hp;
            v[b + S_HP_MAX] = S.boss_hp_max;
        }
    }
    for e in 0..LOG {
        v[V_LOG + e * LOG_W..V_LOG + (e + 1) * LOG_W].copy_from_slice(&S.log[e]);
    }
    v[V_LOG_N] = S.log_n;
    for e in 0..EV {
        v[V_EV + e * EV_W..V_EV + (e + 1) * EV_W].copy_from_slice(&S.ev[e]);
    }
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
        let mut stepped = false;
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
                if gd_hold_begin(REWARD_TIME * 4 + SHOP_TIME + 120) == 0 {
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
                    stepped = true;
                    // V3 12.4: technique, the records, the stock edges and cash-outs, the boss, Burning, Volley,
                    // then time charge and the chains, then the states
                    finale_probe();
                    tech_frame();
                    records_frame();
                    moments_frame();
                    drives_frame();
                    heat_events();
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
        if !stepped {
            // a frozen frame steps nothing; the L-cancel bits are cleared so a stale one cannot pay later
            for p in 0..nh() {
                gd_envoy_events(p);
            }
            records_drop();
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
        S.escape_at = [-99; NP];
        S.boss_slot = -1;
        S.offers = [[0, 1, 2]; NP];
        S.shop_off = [[-1; 3]; NP];
        S.fx_mode = FX_FLARE;
        S.fx_set = -1;
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
    magic: 0x444D_4447, struct_size: 124, abi_min: ABI, abi_max: ABI, hook_table_version: 1, mod_version: 0x0005_0000,
    required_features: 0, optional_features: 1 << 3, state_bytes: 16384, hook_count: 3, hooks: HOOKS.as_ptr(),
    mod_id: text(b"gd.envoy_sim"), mod_name: text(b"Envoy run"),
};
