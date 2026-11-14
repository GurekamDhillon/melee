//! envoy_sim: an Envoy run (docs/rust-port/notes/envoy-1p.md, envoy-drives.md, envoy-v3-rules.md). The sim half:
//! run state, the plan of the next fight, tags, mid-fight stage switches, drives and keystones, Heat (V3), local
//! co-op and the run's end. Everything is this module's linear memory (rolled back with the game) or engine statics
//! (melee_modes::tu::{envoy, tag_swap, petey}); the presentation (envoy_ui) only reads the published view.
//!
//! * run: the Short run, 5 fights: a duel, an elite (act boss 1), a pack, an elite (act boss 2), the final boss (a
//!   level-9 Ganondorf, or Petey with MELEE_ENVOY_BOSS=petey). Each player's pair (main + partner) shares one stock
//!   pool: 4 solo, 5 in co-op (MELEE_ENVOY_COOP=1: a second human in slot 1 with its own pair).
//! * tag: D-pad down (ground or air, actionable), 5 s cooldown (Lock: 10 s), 60-frame frozen cinematic.
//! * stage switches: events, the next of BF / YS / DL / FoD / FD: a boss or elite breaks (every road); on the
//!   Shifting road also a foe KO (not the last); 1 a fight (2 on the Shifting road), 20 s apart, never in the first 8 s.
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
use envoy_data::items::*;
use envoy_data::view::*;
use envoy_data::*;

mod items;

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
    fn gd_fighter_get_facing(h: u32) -> f32;
    fn gd_fighter_read(h: u32, out: *mut u32);
    fn gd_player_present(slot: i32) -> i32;
    fn gd_player_stocks(slot: i32) -> i32;
    fn gd_tag_partner(slot: i32) -> i32;
    fn gd_tag_begin_ex(slot: i32, len: i32, flags: i32) -> i32;
    fn gd_tag_swap(skip: i32) -> i32;
    fn gd_tag_quick(slot: i32, any: i32, pause: i32) -> i32;
    fn gd_tag_assist(slot: i32, dir: i32) -> i32;
    fn gd_tag_assist_frames(slot: i32) -> i32;
    fn gd_envoy_wave_park(slot: i32) -> i32;
    fn gd_envoy_wave_pending() -> i32;
    fn gd_envoy_wave_start() -> i32;
    fn gd_pad_held(port: i32) -> u32;
    fn gd_pad_stick(port: i32) -> i32;
    fn gd_envoy_tag_button() -> i32;
    fn gd_transition_phase() -> i32;
    fn gd_transition_kind() -> i32;
    fn gd_transition_frame() -> i32;
    fn gd_transition_slot() -> i32;
    fn gd_stage_grkind() -> i32;
    fn gd_stage_transition(grkind: i32, len: i32) -> i32;
    fn gd_stage_transition_live(grkind: i32, len: i32, at: i32) -> i32;
    fn gd_stage_live(out: *mut i32) -> i32;
    fn gd_hold_begin(len: i32) -> i32;
    fn gd_envoy_end_hold(on: i32) -> i32;
    fn gd_envoy_end_match() -> i32;
    fn gd_envoy_enemy_spawn(which: i32, x: f32, y: f32, facing: i32, handle: i32) -> i32;
    fn gd_envoy_enemy_i(handle: i32, field: i32) -> i32;
    fn gd_envoy_enemy_status(handle: i32) -> i32;
    fn gd_hold_release() -> i32;
    fn gd_pad_pressed(port: i32) -> u32;
    fn gd_envoy_plan(ptr: *const i32, n: u32) -> i32;
    fn gd_envoy_publish(ptr: *const i32, n: u32) -> i32;
    fn gd_envoy_seq() -> i32;
    fn gd_envoy_start_fight() -> i32;
    fn gd_envoy_salt() -> i32;
    fn gd_envoy_quit() -> i32;
    fn gd_envoy_roster(i: i32) -> i32;
    fn gd_envoy_apply(slot: i32, which: i32, v: i32) -> i32;
    fn gd_envoy_percent(slot: i32, bench: i32, delta_milli: i32) -> i32;
    fn gd_envoy_test_drives(out: *mut u8, cap: u32) -> i32;
    fn gd_envoy_shared_drives() -> i32;
    fn gd_envoy_coop() -> i32;
    fn gd_envoy_speed(slot: i32, active: i32, bench: i32) -> i32;
    fn gd_envoy_absorb(slot: i32) -> i32;
    fn gd_envoy_attacker(slot: i32) -> i32;
    fn gd_envoy_combo_break(slot: i32, frames: i32, freeze: i32) -> i32;
    fn gd_envoy_jumps(slot: i32) -> i32;
    fn gd_envoy_air_jump(slot: i32) -> i32;
    fn gd_envoy_blast(slot: i32, x: f32, y: f32, r: f32, dmg: i32, angle: i32, kbg: i32, bkb: i32) -> i32;
    fn gd_envoy_blast_cap(slot: i32, cap_milli: i32, tag: i32) -> i32;
    fn gd_envoy_landed(slot: i32, out: *mut i32) -> i32;
    fn gd_envoy_boss(out: *mut u32) -> i32;
    fn gd_envoy_boss_dmg(out: *mut u32) -> i32;
    fn gd_envoy_events(slot: i32) -> u32;
    fn gd_envoy_kb_scale(slot: i32, mul_milli: i32) -> i32;
    fn gd_envoy_armed() -> i32;
    fn gd_envoy_record(out: *mut i32, cap: u32) -> i32;
    fn gd_hash_word(w: u32);
    fn gd_log(msg: *const u8, value: i32);
}

const FIGHTS: usize = 4;
const START_POOL: [i32; 2] = [4, 5];
const TAG_LEN: i32 = 60;
const TAG_COOLDOWN: i32 = 300;
/// Quick Tag (envoy-tag-rethink.md direction 1; the default): cooldown 3 s (Lock doubles it), a press remembered
/// 8 frames, the tag-cancel window 20 frames after the player's fighter dealt damage, the newcomer's hitlag-style
/// pause (frames; the outgoing fighter is gone at once and the foe runs on).
const QT_COOLDOWN: i32 = 180;
const QT_BUFFER: i32 = 8;
const QT_WINDOW: i32 = 20;
const QT_PAUSE: i32 = 2;
/// Tap vs hold (Assist Strike): Tag released before QT_HOLD frames is a tap (the partner strikes, ASSIST_COST Heat);
/// held QT_HOLD frames is the swap (free, the cooldown). A press within the after-damage window keeps it (Handoff).
const QT_HOLD: i32 = 15;
/// Items: the Heat meter is gone from the HUD, so the strike costs nothing (it is still one strike at a time).
const ASSIST_COST: i64 = 0;
/// The stick (pad units, about +-80) past which a strike goes up or down instead of forward.
const ASSIST_STICK: i32 = 40;
const TAG_BTN_Y: i32 = 0;
const TAG_BTN_X: i32 = 1;
const TAG_SKIP_AFTER: i32 = 6;
const STAGE_LEN: i32 = 90;
/// Stage switches (decide-6 A): no switch in a fight's first STAGE_GRACE live frames, none within STAGE_GAP of the
/// last one (a request waits for both); the Shifting road's damage step (x1000, all players summed) and its pay.
/// (Retuned after play: "it swapped stages way too much".) At most STAGE_MAX switches a fight (SHIFT_CAP on the
/// Shifting road); the Shifting road switches on foe KOs only (no damage trigger) and pays SHIFT_COINS a switch.
const STAGE_GRACE: i32 = 480;
const STAGE_GAP: i32 = 1200;
const STAGE_MAX: i32 = 1;
const SHIFT_COINS: i32 = 10;
const SHIFT_CAP: i32 = 2;
/// The torture knob's switch spacing (live frames); the window after a switch in which a fighter entering a KO, or
/// a stock lost, is reported as caused by it ("envoy: KO right after a stage switch"); the frame after a switch at
/// which every fighter's place is logged (it has landed by then).
const TORTURE_GAP: i32 = 240;
const SWITCH_KO_WINDOW: i32 = 120;
const SWITCH_CHECK: i32 = 45;
const REWARD_TIME: i32 = 1200;
/// The shop's timer (it leaves on its own at 0).
const SHOP_TIME: i32 = 1800;
/// Frames into a fight before its reward screen opens (after the fighters are placed).
const REWARD_DELAY: i32 = 2;
/// After the last KO, every drop still on the floor flies to its owner over this many frames (then the match ends).
const FLY_TIME: i32 = 30;

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
/// Rooms (slice B): a session is three rooms and the boss: rooms 1-2, the stop, room 3, the boss. Room 1 is one short
/// wave; the others two (room_waves), the elite arriving in room 3's second wave (its first is a plain foe, ELITE_W1).
const SCHEDULE: [Fight; FIGHTS] = [
    Fight { kind: FK_DUEL, ncpu: 1, level: [3, 0, 0], stocks: [1, 0, 0], foe_drives: 0, foe_level: 0, scale: 1000 },
    Fight { kind: FK_PACK, ncpu: 2, level: [5, 6, 0], stocks: [1, 1, 0], foe_drives: 0, foe_level: 0, scale: 1000 },
    Fight { kind: FK_ELITE, ncpu: 1, level: [8, 0, 0], stocks: [2, 0, 0], foe_drives: 2, foe_level: 2, scale: 1200 },
    Fight { kind: FK_BOSS, ncpu: 1, level: [9, 0, 0], stocks: [3, 0, 0], foe_drives: 2, foe_level: 2, scale: 1250 },
];
/// The plan's words (melee_modes::tu::envoy PLAN_WORDS): 44-49 the gauntlet queue (foe rows 0-1, waves 3-5), 50 the
/// elite's wave, 51 the stage a live transition goes to next.
const PLAN_N: usize = 52;
const PL_GAUNT: usize = 44;
const PL_ELITE_WAVE: usize = 50;
const PL_STAGE_NEXT: usize = 51;

/// The room mix (owner, 2026-11-13: "more 1v1s, they can be back to back; different characters are good"; Fights 2,
/// envoy-items-design.md section 10). Per fight of a loop, by loop (1, 2, 3 on): the foes on the field at once and
/// the waves. A 1-foe room with more than one wave is a gauntlet: back-to-back duels, the next foe in right after
/// the KO (no results, no menu), a new character every duel. The one table to tune; co-op adds a foe to a duel or a
/// pack room (room_foes), the boss is always one wave of one. GAUNT_MAX waves at most (the plan's queue).
struct Mix {
    foes: [i32; 3],
    waves: [i32; 3],
}
const ROOM_MIX: [Mix; FIGHTS] = [
    // room 1: the opening gauntlet
    Mix { foes: [1, 1, 1], waves: [2, 3, 3] },
    // room 2: the crowd, 1v3 (crowd knockback); a second crowd wave from loop 3
    Mix { foes: [3, 3, 3], waves: [1, 1, 2] },
    // room 3: the elite gauntlet: plain duels, the elite in the last one
    Mix { foes: [1, 1, 1], waves: [3, 3, 4] },
    // the boss
    Mix { foes: [1, 1, 1], waves: [1, 1, 1] },
];
const GAUNT_MAX: i32 = 5;
/// Recent foe characters remembered across rooms (a new room prefers faces not among them).
const SEEN: usize = 10;

/// Crowd knockback (owner, 2026-11-13): in a fight with CROWD_FROM or more foes, every foe takes CROWD_KB (x1000)
/// knockback, CROWD_KB_STEP more for each foe past CROWD_FROM, at most CROWD_KB_CAP (the engine's own clamp is 1500).
/// Foes only, never a player, Envoy only (the sim's take-damage hook). Four ports and at least one human make 1v3
/// the most today; the Subspace enemies are not fighters and do not count.
const CROWD_FROM: i32 = 3;
const CROWD_KB: i32 = 1350;
const CROWD_KB_STEP: i32 = 100;
const CROWD_KB_CAP: i32 = 1500;

/// The live stage transition (no freeze): its length and the hand-off frame (the rift's core crosses the middle of
/// the screen there), in frames; the presentation draws the rift from the engine's record.
const LIVE_LEN: i32 = 180;
const LIVE_AT: i32 = 96;

/// An elite room's first wave: a plain foe at this level with one stock (the elite comes in the second wave).
const ELITE_W1_LEVEL: i32 = 6;
/// Coins for each wave cleared before a room's last (the room's own clear pays COINS_CLEAR once).
const COINS_WAVE: i32 = 5;
/// The elite's stocks when it arrives in an elite room's second wave (the room ran 150-190 s at 2; target 90-120).
const ELITE_W2_STOCKS: i32 = 1;
/// The most waves a room can have (loops add one each).
const WAVES_MAX: i32 = 4;
/// A shop opens at the start of these fights (after fights 1, 3 and 4; V3 12.9).
#[allow(dead_code)]
const SHOP_AT: [bool; FIGHTS] = [false, false, true, false];
/// Sessions (v3g): the fights that open one. Rooms 1-2 are one session, room 3 and the boss the next, in every loop;
/// inside a session the next fight follows at once (no picks, no shop, no road), the picks are banked for the stop.
const SESSION_START: [bool; FIGHTS] = [true, false, true, false];
/// The drives a foe may carry: the ones that read as an attack on the players.
const FOE_POOL: [&str; 8] = ["ember", "nova", "bolt", "frost", "rush", "bulwark", "hex", "comet"];

const RUN_NONE: i32 = 0;
const RUN_LIVE: i32 = 1;
/// No longer set: a clear starts the next loop (endless NG+, envoy-v3-rules.md "Endless loops").
#[allow(dead_code)]
const RUN_CLEAR: i32 = 2;
/// The stock pool's ceiling for the +1 a clear gives (the HUD's dot row holds it).
const POOL_CAP: i32 = 8;
/// The furthest loop the scaling counts (a guard; nobody gets there).
const LOOP_TOP: i32 = 50;
const RUN_OVER: i32 = 3;
/// `S.outcome`: 0 the fight is on, 1 won, 2 lost (the run is over), 3 left from the pause (the run is kept).
const OUT_LEFT: i32 = 3;

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
const PICK_DX: f32 = 16.0;
const PICK_DY: f32 = 28.0;
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
/// A drop with no room (a full row, or the top level): banked as a pick (view: P_GAIN level 0).
const G_BANKED: i32 = 6;

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

// ---- the combo breaker: a player a foe (a CPU or an enemy) keeps hitting bursts free. Players hit by foes only:
// never the other way round, and never in VS (the sim only acts in an armed run). One tunable block.
/// The foe hit that would make the combo this many hits fires the breaker instead of landing.
const CB_HITS: i32 = 4;
/// ... or the hit that would take the combo's damage to this much (percent x1000).
const CB_DAMAGE: i32 = 35_000;
/// The combo ends after this many frames without a foe hit (or when the player is actionable on the ground).
const CB_GAP: i32 = 60;
/// Intangible frames after a break (the game's white intangibility flash shows them).
const CB_INTANGIBLE: i32 = 45;
/// Frames before the breaker can fire again (3 s).
const CB_COOLDOWN: i32 = 180;
/// Frames every fighter freezes on the break (a hitlag freeze, so the break reads).
const CB_FREEZE: i32 = 6;
/// The push on the foes nearby: a no-damage blast from the player, radius, angle (361: the Sakurai angle), growth, base.
const CB_PUSH_R: f32 = 24.0;
const CB_PUSH_ANGLE: i32 = 361;
const CB_PUSH_KBG: i32 = 0;
const CB_PUSH_BKB: i32 = 50;

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
    /// The loop (1 first; +1 per boss cleared: endless NG+). 0 reads as 1.
    lp: i32,
    /// Sessions: drive picks banked for the next stop; the picks in the stop now and the one being made (1-based).
    bank: i32,
    bank_n: i32,
    bank_i: i32,
    /// The stop runs in this match (after the session's last fight, before the match ends); the stop was had (the next
    /// session's first fight opens with no picks, no shop).
    stop: i32,
    stop_done: i32,
    /// The bank as tokens (RK_*, 4 bits each, the next lowest); `bank` is their count.
    bank_q: i32,
    /// The forked road: the road kind of the session in play (0: none, read as Open), P1's cursor and P2's flag at the
    /// road screen (index + 1; 0 none).
    road: i32,
    road_cur: i32,
    road_flag: i32,
    /// The fire being dealt now: its cap on what lands (x1000), set by fire().
    fx_cap: i32,
    /// The shop's level-up: one per visit, per player.
    shop_lv_done: [i32; NP],
    /// Drive protection: pinned slots per set (bits); a replacement or scrap waiting for its confirm (0 none, 1 open),
    /// its cursor (0 Keep), where (fighter * 8 + slot), the incoming drive (id + 1; 0 = a scrap), its source (1 a pick,
    /// 2 a shop row, 3 a scrap) and the shop row; coins a declined offer pays.
    pins: [i32; SETS],
    confirm: [i32; NP],
    confirm_cur: [i32; NP],
    confirm_at: [i32; NP],
    confirm_new: [i32; NP],
    confirm_src: [i32; NP],
    confirm_row: [i32; NP],
    decline: [i32; NP],
    /// Test knob `protect`: the stop's shop pins, scraps, moves and tries a pinned replace on its own (synctests).
    test_protect: i32,
    /// The last gain that came from a drop: its world position (x100) and owner slot, for the UI's trail.
    gain_from: [[i32; 3]; NP],
    /// Enemy waves (first step, test knob `~K`): the enemies' handles (0: none), their damage taken and hit counts as
    /// last read, spawned (1) for this match.
    en_h: [i32; EN_MAX],
    en_dmg: [i32; EN_MAX],
    en_rcv: [i32; EN_MAX],
    en_spawned: i32,
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
    /// Quick Tag: frames a tag press is still remembered, frames left in the window after dealing damage, Handoff
    /// off (test knob `nohandoff`), the tag button in use this match (0 Y, 1 X, 2 classic D-pad).
    tag_buf: [i32; NP],
    dmg_win: [i32; NP],
    handoff_off: i32,
    tag_auto: i32,
    tag_btn: i32,
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
    drop_all: i32, // test: every KO drops (a fight ends with a drop on the floor)
    /// Test knob (`finaleoff`): Grand Finale's write is computed and logged but not made (the A/B launch probe);
    /// the victim of the last write (or would-be write) and its frame, to log the launch on the next frame.
    finale_off: i32,
    /// Test knob (`aslog`): log player 1's action-state changes; the frame an air dodge was last seen per player.
    as_log: i32,
    /// Test knob (`^N`): every fight starts with the Heat that gives Grand Finale +N/1000 (the A/B probe at a fixed Heat).
    test_heat: i64,
    /// Test knob (`*N`): the player's attack ratio x1000 at match start (a boss break inside a headless test).
    test_atk: i32,
    escape_at: [i32; NP],
    kb_watch: i32,
    kb_watch_at: i32,
    /// Stage switches: the pending request's reason (SH_*; 0 none), the Shifting road's damage toward the next
    /// switch (x1000) and the switches it paid this fight.
    stage_want: i32,
    shift_dmg: i32,
    shift_paid: i32,
    /// Test knob (`noshift`): no stage switch at all (the owner's A/B).
    stage_off: i32,
    /// Rooms: the wave in play (1-based), the room's waves, the foe slots parked for the next wave (bits), test knob
    /// `wave2` (every fight but the boss gets two waves).
    wave: i32,
    waves: i32,
    park: i32,
    wave2: i32,
    /// The wave in play is cleared (paid; the next one not yet in).
    wave_clr: i32,
    /// Tap vs hold: frames Tag has been held since its press (0 none), the after-damage window at the press, the
    /// stick direction at the press (0 forward, 1 up, 2 down), a tap waiting to strike (frames left), the swap's
    /// remembered window, the strike frames seen last frame (the end edge), the test hold for `tagauto`.
    hold_t: [i32; NP],
    hold_win: [i32; NP],
    hold_dir: [i32; NP],
    strike_buf: [i32; NP],
    buf_win: [i32; NP],
    assist_seen: [i32; NP],
    auto_hold: [i32; NP],
    strikes: i32,
    test_buy: i32,
    /// Test knob `torture`: a stage switch every TORTURE_GAP live frames through every stage pair, no caps (the
    /// switch-safety check, tools/rust-port/envoy/switch_torture.sh).
    test_torture: i32,
    /// Each slot's motion at the last switch (+1; 0: not in play) and whether a KO after it was reported.
    sw_as: [i32; 4],
    sw_ko: [i32; 4],
    /// Whether the slot was hit or grabbed since the last switch.
    sw_hit: [i32; 4],
    // ---- envoy items (items.rs; envoy-items-design.md)
    /// The stock limit (heart containers; `pool` is the stocks held now).
    lim: i32,
    /// Copies of each item id per player.
    inv: [[i32; ITEM_IDS]; NP],
    /// The projectile pool: kind, owner player, x100, y100, age, life, a, b (view layout); velocity x100; the slots
    /// each one already struck (bits); its look flags.
    proj: [[i32; PJ_W]; PROJ],
    pv: [[i32; 2]; PROJ],
    pj_hit: [i32; PROJ],
    pf: [i32; PROJ],
    /// One-shot visuals for the UI (frame, item, owner, x100, y100, size x100, flags) and the count written.
    burst: [[i32; BU_W]; BURSTS],
    burst_n: i32,
    /// Per player: this nair / smash already fired its item (once per move); the attack watched for a whiff (state
    /// + 1), whether it connected, where it began (x100, y100, facing); a short hop waiting for its apex and its age;
    /// the action state last frame.
    it_nair: [i32; NP],
    it_smash: [i32; NP],
    it_atk: [i32; NP],
    it_atk_hit: [i32; NP],
    it_atk_at: [[i32; 3]; NP],
    it_hop: [i32; NP],
    it_hop_t: [i32; NP],
    it_prev: [i32; NP],
    /// Moon Drops: the frame each foe slot can be nicked again, per player.
    moon_cd: [[i32; 4]; NP],
    /// Pedestals out (PED_*), frames left before they fly in, opened this match (1).
    ped_kind: i32,
    ped_left: i32,
    ped_done: i32,
    /// The campfire: the play frame it burned (0 none) and the stocks it gave back.
    camp_at: i32,
    camp_gain: i32,
    // ---- Fights 2 (envoy-items-design.md section 10)
    /// The run's recent foe characters (a ring; -1 empty) and the next write index.
    seen: [i32; SEEN],
    seen_n: i32,
    /// The crowd knockback of this fight (x1000; 1000 none) and the foe hits it scaled.
    crowd: i32,
    crowd_hits: i32,
    /// The live stage transition this fight: 0 none, 1 requested (the rift sweeping), 2 handed over; the engine's
    /// finished count seen last.
    lv_on: i32,
    lv_count: i32,
    /// The combo breaker per player: foe hits in the running combo, its damage (x1000), the frame of its last hit,
    /// the cooldown left, a break waiting for the frame end; breaks fired since load.
    cb_hits: [i32; NP],
    cb_dmg: [i32; NP],
    cb_last: [i32; NP],
    cb_cd: [i32; NP],
    cb_due: [i32; NP],
    cb_n: i32,
}

/// The engine's event bits this stepped frame (read once in tech_frame, used by the items the same frame).
static mut EVB: [u32; NP] = [0; NP];

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
static mut PLAN: [i32; PLAN_N] = [0; PLAN_N];
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
            r[R_DRIVES + s * SLOTS + k] = S.drives[s][k] << 4 | S.levels[s][k] | (S.pins[s] >> k & 1) << 12;
        }
    }
    r[R_POOL_DIGEST] = POOL_DIGEST as i32;
    r[R_COINS] = coins();
    r[R_REROLLS] = S.rerolls;
    r[R_ACTIVE] = S.act_id[0] | S.act_id[1] << 8;
    r[R_PIPS] = ((HH.pips[0] / PIP_UNIT) as i32).clamp(0, 255) | ((HH.pips[1] / PIP_UNIT) as i32).clamp(0, 255) << 8;
    r[R_EARNED] = ((HH.earned[0] / COIN_UNIT) as i32).clamp(0, 0xFFFF) | ((HH.earned[1] / COIN_UNIT) as i32).clamp(0, 0x7FFF) << 16;
    // v8: the loop | the stop done << 8 | the road << 9 | the first 5 bank tokens << 12
    r[R_LOOP] = lp().clamp(1, 0xFF) | ((S.stop_done != 0) as i32) << 8 | S.road.clamp(0, 7) << 9 | (S.bank_q & 0xF_FFFF) << 12;
    // v9: the item copies replace the drive words, the stock limit the keystones; the digest is the item table's
    items::rec_put(&mut r);
    r[R_POOL_DIGEST] = (POOL_DIGEST ^ ITEMS_DIGEST) as i32;
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
    let ok = n == REC_WORDS as i32 && r[0] == REC_MAGIC && (5..=REC_VERSION).contains(&r[1]) && r[REC_WORDS - 1] == rec_digest(&r) && (0..FIGHTS as i32).contains(&r[R_FIGHT]);
    if ok && r[1] < 9 {
        // a drive-era run: its drives have no items to become; it is dropped (a new run)
        log(b"envoy: saved run is from the drive rules (record version), dropped\0", r[1]);
        S.state = RUN_NONE;
    } else if ok && r[R_POOL_DIGEST] != (POOL_DIGEST ^ ITEMS_DIGEST) as i32 {
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
        S.lp = rec_loop(r[R_LOOP]).clamp(1, LOOP_TOP);
        S.bank = 0;
        S.bank_q = 0;
        S.road = 0;
        S.stop_done = 0;
        if r[1] >= 8 {
            S.stop_done = r[R_LOOP] >> 8 & 1;
            S.road = r[R_LOOP] >> 9 & 7;
            let q = r[R_LOOP] >> 12 & 0xF_FFFF;
            for i in 0..5 {
                let k = token(q, i);
                if (RK_PICK..=RK_KEYSTONE).contains(&k) {
                    bank_push(k);
                }
            }
        } else if r[1] >= 7 {
            // a version 7 record: its bank count as picks, the Open road
            for _ in 0..(r[R_LOOP] >> 16 & 0xFF).min(BANK_MAX) {
                bank_push(RK_PICK);
            }
            S.stop_done = r[R_LOOP] >> 24 & 1;
        }
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
                let (id, lv) = rec_drive(w, r[1]);
                let ok = is_kind(id, K_DRIVE) && (1..=LEVEL_TOP).contains(&lv);
                S.drives[s][k] = if ok { id } else { 0 };
                S.levels[s][k] = if ok { lv } else { 0 };
                if ok && rec_pin(w, r[1]) {
                    S.pins[s] |= 1 << k;
                } else {
                    S.pins[s] &= !(1 << k);
                }
            }
        }
        HH.purse = r[R_COINS].max(0) as i64 * COIN_UNIT;
        S.rerolls = r[R_REROLLS].max(0);
        S.act_id = [r[R_ACTIVE] & 0xFF, r[R_ACTIVE] >> 8 & 0xFF];
        HH.pips = [(r[R_PIPS] & 0xFF) as i64 * PIP_UNIT, (r[R_PIPS] >> 8 & 0xFF) as i64 * PIP_UNIT];
        HH.earned = [(r[R_EARNED] & 0xFFFF) as i64 * COIN_UNIT, (r[R_EARNED] >> 16 & 0x7FFF) as i64 * COIN_UNIT];
        // items: no drives, keystones or banked picks survive; the item copies and the stock limit come back
        S.drives = [[0; SLOTS]; SETS];
        S.levels = [[0; SLOTS]; SETS];
        S.pins = [0; SETS];
        S.keys = [[0; KEYSTONES]; NP];
        S.bank = 0;
        S.bank_q = 0;
        S.act_id = [0; NP];
        HH.pips = [0; NP];
        items::rec_get(&r);
        S.pool = S.pool.clamp(1, S.lim);
        log(b"envoy: continued with stock limit\0", S.lim);
        log(b"envoy: continued on road ", S.road);
        log(b"envoy: armed, run continued at loop*10 + fight\0", S.lp * 10 + S.fight + 1);
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

// ---- endless loops (NG+; envoy-v3-rules.md "Endless loops"): everything scales from the one loop number

/// The loop (1: the first time through the five fights).
unsafe fn lp() -> i32 {
    S.lp.clamp(1, LOOP_TOP)
}

/// Loops past the first (the scaling's step count).
unsafe fn lx() -> i32 {
    lp() - 1
}

/// The highest drive level reachable now: 3, one more per loop after the first.
unsafe fn lvl_cap() -> i32 {
    (MAX_LEVEL + lx()).min(LEVEL_TOP)
}

/// The fight's key for the seeded rolls: the fight in loop 1 (loop 1's rolls are unchanged), new ones after.
unsafe fn fkey() -> u32 {
    (S.fight + 8 * lx()) as u32
}

/// `base` raised by `per` thousandths per loop step `e`.
fn per_loop(base: i32, per: i32, e: i32) -> i32 {
    (base as i64 * (1000 + per as i64 * e as i64) / 1000) as i32
}

/// The retune (envoy-content-pass.md 5.3): drive and reroll prices +10% per loop (level-ups: no loop markup; a stock
/// 100 to loop 5, then +25 a loop). The Petey cages' HP (so the HP between his four breaks): +25% per loop.
const LOOP_PRICE: i32 = 100;
const LOOP_BOSS_HP: i32 = 250;
/// Foe damage dealt x1000 after `e` loops cleared: +5% at loop 2, +10% a loop for loops 3-5, +15% a loop after.
fn foe_atk(e: i32) -> i32 {
    match e {
        i32::MIN..=0 => 1000,
        1 => 1050,
        2..=4 => 1050 + 100 * (e - 1),
        _ => 1350 + 150 * (e - 4),
    }
}

/// Foe toughness x1000 (damage taken is divided by it): +10% a loop for loops 2-5, +15% a loop after.
fn foe_tough(e: i32) -> i32 {
    if e <= 4 { 1000 + 100 * e.max(0) } else { 1400 + 150 * (e - 4) }
}

/// An ordinary fight's elite modifier (loop 3 on): the first CPU is an elite of this size.
const LOOP_ELITE_SCALE: i32 = 1150;

// ---- balance pass 1 (docs/rust-port/notes/envoy-balance.md): four rules against one-shots
/// Rule 1: a foe's damage bonuses add, never multiply: half the elite / boss size bonus (the size already reaches
/// further) plus the loop bonus, the sum capped.
const FOE_ATK_CAP: i32 = 1500;
/// Rule 2: a foe's drive fires at most at level 3's strength and lands at most this much (x1000), blasts included.
const FOE_FIRE_CAP: i32 = 12_000;
/// Rule 3: a room has at most this many waves (was WAVES_MAX).
const WAVES_CAP: i32 = 3;
/// Rule 4: clearing a wave (not the room's last) heals every player's fighter in play this much (x1000).
const WAVE_HEAL: i32 = 25_000;

/// Test knob `oldscale`: the pre-balance numbers (measurement A/B only). Fixed per process.
static mut OLD_SCALE: i32 = -1;
unsafe fn old_scale() -> bool {
    if OLD_SCALE < 0 {
        OLD_SCALE = test_flag("oldscale") as i32;
    }
    OLD_SCALE == 1
}

/// Test knob `blastover`: a foe's blast is armed at 30% whatever the rules say, so a run shows the engine's cap
/// holding it to FOE_FIRE_CAP on what lands (`kblog blast raw` against the record's `dmg`). Fixed per process.
static mut BLAST_OVER: i32 = -1;
unsafe fn blast_over() -> bool {
    if BLAST_OVER < 0 {
        BLAST_OVER = test_flag("blastover") as i32;
    }
    BLAST_OVER == 1
}

/// A foe's attack ratio x1000: the size bonus (`sc` - 1000) plus the loop bonus, capped (rule 1).
unsafe fn foe_atk_with(sc: i32, e: i32) -> i32 {
    if old_scale() {
        return (sc as i64 * foe_atk(e) as i64 / 1000) as i32;
    }
    (1000 + (sc - 1000).max(0) / 2 + (foe_atk(e) - 1000)).min(FOE_ATK_CAP)
}

/// Test knob `kblog`: everything a foe does to a player's percent, as it was applied. Not sim state (logs only).
/// * a hit (the take-damage hook): `kblog hit` lines, one record per source. `dmg` is the percent the fighter gained
///   (read before and after the hit lands, so after every multiplier and cap). A frame's hits arrive summed, so a
///   frame in which a foe's drive blast landed is split in two records: the blast's part (the engine reports it per
///   capsule: `gd_envoy_landed`), flagged `blast` and followed by `kblog blast <drive>` (owner slot * 1000 + level *
///   100 + capsule * 10 + capsules landed), `kblog blast raw` (before the cap) and `kblog blast launches` (1: the
///   blast's knockback won the frame), and the rest (ordinary moves). The record of the hit that launches comes last.
/// * a foe's percent fire (`kblog fire <drive>`), a foe's Mark extra (`kblog mark extra`) and a Burning tick
///   (`kblog burn tick`): percent only, no knockback.
/// * every player stock lost (`kblog KO`): the percent it was lost at, the last hit record and whether it was a blast.
static mut KB_LOG: i32 = -1;
/// The hit waiting for its launch speed: play frame, percent gained, attacker slot, then `gd_envoy_landed`'s words.
static mut KB_W: [[i32; 12]; 2] = [[-1; 12]; 2];
static mut KB_PRE: [i32; 2] = [-1; 2];
static mut KB_PCT: [i32; 2] = [0; 2];
static mut KB_LAST: [[i32; 4]; 2] = [[0; 4]; 2];
/// The drive (id + 1) and level of the foe fire running now (`fire`), for `kblog fire`.
static mut KB_FIRE: [i32; 2] = [0; 2];

/// Foe slot `s`'s attack ratio x1000 as the plan gave it (an elite room's second wave: the elite's).
unsafe fn foe_atk_now(s: i32) -> i32 {
    let row = (s - nh()).clamp(0, 2) as usize;
    let ew = if PLAN[PL_ELITE_WAVE] > 0 { PLAN[PL_ELITE_WAVE] } else { 2 };
    if row == 0 && S.wave >= ew && PLAN[38] > 0 { PLAN[38] } else { PLAN[17 + row] }
}
unsafe fn kb_log() -> bool {
    if KB_LOG < 0 {
        KB_LOG = test_flag("kblog") as i32;
    }
    KB_LOG == 1
}
/// A fighter's percent x1000 by handle (-1: none).
unsafe fn kb_percent(h: u32) -> i32 {
    if h == 0 {
        return -1;
    }
    let mut w = [0u32; 20];
    gd_fighter_read(h, w.as_mut_ptr());
    (f32::from_bits(w[10]) * 1000.0 + 0.5) as i32
}
unsafe fn kb_kind(s: i32) -> i32 {
    let h = if s >= 0 { gd_fighter_from_slot(s) } else { 0 };
    if h != 0 { gd_fighter_kind(h) } else { -1 }
}
/// One `kblog hit` record: fight*10 + loop | dmg x1000 | percent after x1000 | speed^2 x10000 | blast*100 + attacker
/// kind | the knockback that won the frame x1000.
unsafe fn kb_rec(p: usize, dmg: i32, pct: i32, sp: i32, blast: i32, a: i32, kb: i32) {
    log(b"envoy: kblog hit fight*10+loop\0", S.fight * 10 + lp());
    log(b"envoy: kblog hit dmg\0", dmg);
    log(b"envoy: kblog hit pct\0", pct);
    log(b"envoy: kblog hit speed\0", sp);
    log(b"envoy: kblog hit blast*100+ck\0", blast * 100 + kb_kind(a));
    log(b"envoy: kblog hit kb\0", kb);
    KB_LAST[p] = [dmg, pct, sp, blast];
}
unsafe fn kb_frame() {
    if !kb_log() {
        return;
    }
    for p in 0..nh() as usize {
        let h = gd_fighter_from_slot(p as i32);
        if h == 0 {
            continue;
        }
        let mut w = [0u32; 20];
        gd_fighter_read(h, w.as_mut_ptr());
        let pct = (f32::from_bits(w[10]) * 1000.0) as i32;
        let wt = KB_W[p];
        if wt[0] >= 0 && S.play > wt[0] {
            let (x, y) = (f32::from_bits(w[6]), f32::from_bits(w[7]));
            let sp = ((x * x + y * y) * 10000.0) as i32; // speed^2 x10000 (no_std: no sqrt)
            // wt[3..12]: capsules landed, landed x1000, before the cap, owner slot, capsule, tag, group, launches, kb
            let (dmg, a, kb) = (wt[1], wt[2], wt[11]);
            let part = if wt[3] > 0 && is_foe(wt[6]) { wt[4].min(dmg) } else { 0 };
            let rest = dmg - part;
            if part > 0 && rest > 0 && wt[10] != 0 {
                kb_rec(p, rest, pct, sp, 0, a, kb); // the blast launches: its record comes last
            }
            if part > 0 {
                kb_rec(p, part, pct, sp, 1, wt[6], kb);
                let id = (wt[8] & 0xFF) as usize;
                if id >= 1 && id <= POOL.len() {
                    log_drive(b"kblog blast", id - 1, wt[6] * 1000 + (wt[8] >> 8 & 0xF) * 100 + wt[7].clamp(0, 9) * 10 + wt[3].min(9));
                }
                log(b"envoy: kblog blast raw\0", wt[5]);
                log(b"envoy: kblog blast launches\0", wt[10]);
            }
            if rest > 0 && (part == 0 || wt[10] == 0) {
                kb_rec(p, rest, pct, sp, 0, a, kb);
            }
            KB_W[p] = [-1; 12];
        }
        if pct > 0 {
            KB_PCT[p] = pct;
        }
    }
}
unsafe fn kb_ko(p: usize) {
    if kb_log() && p < 2 {
        log(b"envoy: kblog KO fight*10+loop\0", S.fight * 10 + lp());
        log(b"envoy: kblog KO pct\0", KB_PCT[p]);
        log(b"envoy: kblog KO last dmg\0", KB_LAST[p][0]);
        log(b"envoy: kblog KO last speed\0", KB_LAST[p][2]);
        log(b"envoy: kblog KO last blast\0", KB_LAST[p][3]);
        KB_PCT[p] = 0;
        KB_LAST[p] = [0; 4];
    }
}

/// A MELEE_ENVOY_TEST_DRIVES item given by name (read from the string, so a fresh run's first plan sees it too).
unsafe fn test_flag(name: &str) -> bool {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return false;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    s.split(|c| c == ',' || c == '/' || c == '|' || c == ';').any(|t| t.trim() == name)
}

/// MELEE_ENVOY_TEST_DRIVES item `@N`: a new run starts in loop N (tests; fixed per process).
unsafe fn test_loop() -> i32 {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return 1;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    s.split(|c| c == ',' || c == '/' || c == '|' || c == ';')
        .find_map(|t| t.trim().strip_prefix('@').and_then(|x| x.parse::<i32>().ok()))
        .unwrap_or(1)
        .clamp(1, LOOP_TOP)
}

/// MELEE_ENVOY_TEST_DRIVES item `#N`: a new run's seed as if N more runs came before (tests: seed sweeps).
unsafe fn test_seed() -> i32 {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return 0;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    s.split(|c| c == ',' || c == '/' || c == '|' || c == ';')
        .find_map(|t| t.trim().strip_prefix('#').and_then(|x| x.parse::<i32>().ok()))
        .unwrap_or(0)
        .clamp(0, 1_000_000)
}

/// The seed of run number `runs` (1 the first of this save): the old fixed sequence when the host's salt is 0 (tests,
/// scripted runs, `MELEE_ENVOY_ARM`), else salted by the camp's New run (`gd_envoy_salt`: fixed from the arm on, so
/// the plan made before the match and the run's start agree; the seed is sim state from then on).
unsafe fn run_seed(runs: i32) -> u32 {
    mix((runs as u32).wrapping_add(test_seed() as u32).wrapping_add((gd_envoy_salt() as u32).wrapping_mul(0x85EB_CA6B)).wrapping_mul(0x9E37_79B9))
}

fn nh() -> i32 {
    unsafe { S.nh.clamp(1, 2) }
}

/// CPUs this fight: the schedule's, one more in co-op for a duel or a pack (the CPUs have slots 2-3 then).
unsafe fn ncpu() -> i32 {
    let f = sched();
    if test_waves() >= 0 {
        return 0;
    }
    if old_scale() {
        let extra = (nh() == 2 && (f.kind == FK_DUEL || f.kind == FK_PACK)) as i32;
        // loop 2 on: a solo pack fight has one more foe
        let more = (nh() == 1 && f.kind == FK_PACK && lx() >= 1) as i32;
        return (f.ncpu + extra + more).min(4 - nh());
    }
    room_foes(S.fight as usize, lx(), nh() == 2).min(4 - nh())
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
        Some(k) => S.levels[set][k] < lvl_cap(),
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
            if S.levels[set][k] >= lvl_cap() {
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
    S.pins[set] &= !(1 << k);
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
                if item.starts_with('~') {
                    continue; // an enemy-wave test fight (test_waves)
                }
                if item.starts_with('#') {
                    continue; // the seed offset (test_seed)
                }
                if item.starts_with('@') {
                    continue; // the start loop (test_loop)
                }
                if item == "noshop" {
                    S.no_shop = 1;
                    continue;
                }
                if item == "tagauto" {
                    S.tag_auto = 1;
                    continue;
                }
                if item == "nohandoff" {
                    S.handoff_off = 1;
                    continue;
                }
                if item == "protect" {
                    S.test_protect = 1;
                    continue;
                }
                if item == "torture" {
                    S.test_torture = 1;
                    continue;
                }
                if item == "buystock" {
                    S.test_buy = 1;
                    continue;
                }
                if item == "wave2" {
                    S.wave2 = 1;
                    continue;
                }
                if item == "noshift" {
                    S.stage_off = 1;
                    continue;
                }
                if item == "shift" {
                    S.road_cur = 2; // the forked road: take the Shifting road (tests)
                    S.road = ROAD_SHIFT;
                    continue;
                }
                if item == "steep" {
                    S.road_cur = 1; // the forked road: take the Steep road (tests; a run begun past fight 1 is on it)
                    S.road = ROAD_STEEP;
                    continue;
                }
                if item == "dropall" {
                    S.drop_all = 1;
                    continue;
                }
                if let Some(c) = item.strip_prefix('*') {
                    S.test_atk = c.parse::<i32>().unwrap_or(0).clamp(0, 20_000);
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
                let n = lv.trim().parse::<i32>().unwrap_or(1);
                // items (`key[:copies]`), the stock limit (`lim:N`) and the stocks held (`pool:N`); drive keys are
                // no longer worn by the players (the drives are cut)
                match key.trim() {
                    "lim" => {
                        S.lim = n.clamp(1, items::LIM_CAP);
                        S.pool = S.pool.min(S.lim);
                    }
                    "pool" => S.pool = n.clamp(1, S.lim.max(1)),
                    k => {
                        let _ = f;
                        items::test_item(p, k, n);
                    }
                }
            }
        }
        let _ = ks; // keystones are cut with the drives
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
        PLAN = [0; PLAN_N];
        gd_envoy_plan(PLAN.as_ptr(), PLAN_N as u32);
        return;
    }
    let fresh = S.state != RUN_LIVE;
    let coop = if fresh { gd_envoy_coop() != 0 } else { S.nh == 2 };
    let (fight, pool, seed) = if fresh {
        ((gd_envoy_start_fight() as usize).min(FIGHTS - 1), START_POOL[coop as usize], run_seed(S.runs.wrapping_add(1)))
    } else {
        (S.fight as usize, S.pool, S.seed as u32)
    };
    let f = &SCHEDULE[fight];
    let e = if fresh { test_loop() - 1 } else { lx() };
    let waves = test_waves() >= 0;
    let n = if old_scale() {
        let more = (!coop && f.kind == FK_PACK && e >= 1) as i32;
        (f.ncpu + (coop && (f.kind == FK_DUEL || f.kind == FK_PACK)) as i32 + more).min(if coop { 2 } else { 3 })
    } else {
        room_foes(fight, e, coop)
    };
    let p = &mut PLAN;
    *p = [0; PLAN_N];
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
        let used = [p[8], p[9], p[10]];
        p[8 + i] = if old_scale() {
            CPU_POOL[(mix(seed ^ ((fight as u32 + 8 * e as u32) << 8) ^ i as u32) % CPU_POOL.len() as u32) as usize]
        } else {
            pick_face(mix(seed ^ ((fight as u32 + 8 * e as u32) << 8) ^ i as u32), &used[..i])
        };
        // loop 2 on: every CPU at level 9; elites and bosses one more stock every second loop
        // loop 2: loop 1's levels +3 (to 9); loop 3 on: 9
        p[11 + i] = if e >= 2 { 9 } else if e == 1 { (f.level[j].max(1) + 3).min(9) } else { f.level[j].max(1) };
        p[14 + i] = f.stocks[j].max(1) + (coop && f.kind >= FK_ELITE) as i32 + (f.kind >= FK_ELITE) as i32 * (e / 2);
        p[17 + i] = foe_atk_with(1000, e);
        p[20 + i] = 1_000_000 / foe_tough(e);
        p[23 + i] = f.scale;
    }
    if f.kind >= FK_ELITE || e >= 2 {
        // an elite or a boss (and loop 3 on, every fight's first foe): bigger, hits harder, takes less
        let sc = if f.kind >= FK_ELITE { f.scale } else { LOOP_ELITE_SCALE };
        p[17] = foe_atk_with(sc, e);
        p[20] = (2000 - sc) * 1000 / foe_tough(e);
        p[23] = sc;
    }
    p[30] = per_loop(1000, LOOP_BOSS_HP, e); // the Petey cages' HP x1000 (melee_modes::tu::envoy P_BOSS_HP)
    // rooms: each foe row's next-wave character (another pick from the pool) and stocks; -1 for a one-wave fight
    let nw = room_waves(fight, f.kind, e);
    if f.kind == FK_ELITE && nw >= 2 {
        // the elite arrives in the second wave: the first is a plain foe; the elite's numbers ride in 38-41
        p[38] = p[17];
        p[39] = p[20];
        p[40] = p[23];
        p[41] = p[11];
        p[11] = if e >= 2 { 9 } else { (ELITE_W1_LEVEL + 3 * e).min(9) };
        p[14] = 1;
        p[17] = foe_atk_with(if e >= 2 { LOOP_ELITE_SCALE } else { 1000 }, e);
        p[20] = 1_000_000 / foe_tough(e);
        p[23] = if e >= 2 { LOOP_ELITE_SCALE } else { 1000 };
    }
    for i in 0..3 {
        if nw >= 2 && f.kind != FK_BOSS {
            // a new face: never one of this room's first-wave characters (any row) nor another row's second-wave one
            // (it used to avoid only its own row's, so wave two often brought back a face from wave one)
            let rows = (p[7].clamp(0, 3) as usize).max(i + 1);
            let used = |c: i32, p: &[i32; PLAN_N]| (0..rows).any(|r| p[8 + r] == c) || (0..i).any(|r| p[32 + r] == c);
            let start = (mix(seed ^ ((fight as u32 + 8 * e as u32) << 8) ^ (i as u32 + 0x40)) % CPU_POOL.len() as u32) as usize;
            let mut ck = CPU_POOL[start];
            if old_scale() {
                for k in 0..CPU_POOL.len() {
                    let c = CPU_POOL[(start + k * 7) % CPU_POOL.len()];
                    if !used(c, p) {
                        ck = c;
                        break;
                    }
                }
            } else {
                let mut taken = [-1i32; 6];
                for r in 0..rows.min(3) {
                    taken[r] = p[8 + r];
                }
                for r in 0..i {
                    taken[3 + r] = p[32 + r];
                }
                ck = pick_face(start as u32, &taken);
            }
            p[32 + i] = ck;
            // the elite's stocks for an elite room's second wave, else the row's own
            p[35 + i] = if f.kind == FK_ELITE && i == 0 { ELITE_W2_STOCKS + (coop as i32) + e / 2 } else { p[14 + i] };
        } else {
            p[32 + i] = -1;
            p[35 + i] = 0;
        }
    }
    if nw >= 3 && f.kind != FK_BOSS && !old_scale() {
        // gauntlets: waves 3.. of foe rows 0-1 bring a new face each (never one this room has had; the run's recent
        // faces avoided while the cast allows)
        let rows = p[7].clamp(0, 2) as usize;
        let mut taken = [-1i32; 16];
        let mut nt = 0;
        for r in 0..(p[7].clamp(0, 3) as usize) {
            taken[nt] = p[8 + r];
            taken[nt + 1] = p[32 + r];
            nt += 2;
        }
        for r in 0..2 {
            for k in 0..3 {
                let w = 3 + k as i32;
                if r >= rows || w > nw {
                    p[PL_GAUNT + r * 3 + k] = -1;
                    continue;
                }
                let c = pick_face(mix(seed ^ ((fight as u32 + 8 * e as u32) << 8) ^ (0x80 + r as u32 * 8 + k as u32)), &taken[..nt]);
                p[PL_GAUNT + r * 3 + k] = c;
                if nt < taken.len() {
                    taken[nt] = c;
                    nt += 1;
                }
            }
        }
        let face = |r: usize| (0..3).fold(0, |a, k| a * 100 + (p[PL_GAUNT + r * 3 + k] + 1));
        log(b"envoy: gauntlet faces, waves 3-5 ", face(0));
    } else {
        for k in 0..6 {
            p[PL_GAUNT + k] = -1;
        }
    }
    p[PL_ELITE_WAVE] = if f.kind == FK_ELITE && nw >= 2 { nw } else { 0 };
    // the stage a live transition goes to first (the next in the rotation): hinted to the asset worker now
    p[PL_STAGE_NEXT] = match STAGES.iter().position(|&k| k == p[1]) {
        Some(i) => STAGES[(i + 1) % STAGES.len()],
        None => STAGES[0],
    };
    if nw >= 2 && f.kind != FK_BOSS {
        // the room's faces for the checks (wave_faces.sh): per row (ckind + 1), 0 for a row not in play, two digits each
        let face = |w: usize| (0..3).fold(0, |a, r| a * 100 + if (r as i32) < p[7] { p[w + r] + 1 } else { 0 });
        log(b"envoy: room faces, wave 1 ", face(8));
        log(b"envoy: room faces, wave 2 ", face(32));
    }
    if waves {
        // an enemy-wave test fight: no foe fighters, Final Destination
        p[7] = 0;
        p[1] = 37;
        p[26] = 0;
    }
    if f.kind == FK_BOSS {
        p[8] = CK_GANONDORF;
        p[26] = 1; // the boss fight (the engine swaps in Petey when MELEE_ENVOY_BOSS=petey)
    }
    p[27] = coop as i32;
    p[28] = if fresh || !coop { -1 } else { S.pair[1][0] };
    p[29] = if fresh || !coop { -1 } else { S.pair[1][1] };
    gd_envoy_plan(p.as_ptr(), PLAN_N as u32);
}

/// A rewritten match began (gd_envoy_seq moved).
unsafe fn on_new_match(seq: i32) {
    let started = S.state != RUN_LIVE;
    if started {
        S.runs = S.runs.wrapping_add(1);
        S.state = RUN_LIVE;
        S.seed = run_seed(S.runs) as i32;
        S.fight = gd_envoy_start_fight().clamp(0, FIGHTS as i32 - 1);
        S.nh = if gd_envoy_coop() != 0 { 2 } else { 1 };
        S.pool = START_POOL[(S.nh == 2) as usize];
        // items: the stock limit starts where the stocks do; no item held
        S.lim = items::LIM_START[(S.nh == 2) as usize];
        S.seen = [-1; SEEN];
        S.seen_n = 0;
        S.inv = [[0; ITEM_IDS]; NP];
        S.burst = [[0; BU_W]; BURSTS];
        S.burst_n = 0;
        S.bosses = 0;
        S.lp = test_loop();
        if S.lp > 1 {
            S.bosses = 2; // a test run begun in a later loop: both bosses' slots and keystones
        }
        // a test run begun past fight 1 has the pick of the fight before it banked
        S.bank = 0;
        S.bank_q = 0;
        S.bank_n = 0;
        S.bank_i = 0;
        S.drives = [[0; SLOTS]; SETS];
        S.levels = [[0; SLOTS]; SETS];
        S.keys = [[0; KEYSTONES]; NP];
        S.picks = 0;
        S.rerolls = 0;
        S.slots_bonus = 0;
        S.no_shop = 0;
        S.drop_all = 0;
        S.test_protect = 0;
        S.handoff_off = 0;
        S.tag_auto = 0;
        S.pins = [0; SETS];
        S.confirm = [0; NP];
        S.finale_off = 0;
        S.as_log = 0;
        S.test_heat = 0;
        S.test_atk = 0;
        S.escape_at = [-99; NP];
        S.kb_watch = -1;
        S.shared = (gd_envoy_shared_drives() != 0) as i32;
        S.pair = [[gd_envoy_roster(0), gd_envoy_roster(1)], [gd_envoy_roster(2), gd_envoy_roster(3)]];
        S.last_pick = [-1; NP];
        S.attrs_done = 0;
        HH = core::mem::zeroed();
        // items: Volley is cut with the drives (no active, no pips)
        S.act_id = [0; NP];
        log(b"envoy: run start, players\0", S.nh);
        if S.shared != 0 {
            log(b"envoy: drives shared by the pair\0", 1);
        }
        test_drives();
    } else if S.fight > 0 || S.lp > 1 {
        // interest: +1 per 10 coins held, to +5
        let i = (coins() / 10).min(INTEREST_MAX);
        if i > 0 {
            HH.purse += i as i64 * COIN_UNIT;
            log(b"envoy: interest, coins\0", i);
        }
    }
    S.seq = seq;
    // the forked road: a three-fight session's first fight takes the road P1's cursor is on
    if S.fight == 0 {
        let n = roads_offered_at(0);
        S.road = if n > 0 { road_kind(S.road_cur.clamp(0, n - 1)) } else { ROAD_OPEN };
        S.road_flag = 0;
        log(b"envoy: road taken ", S.road);
    } else if SESSION_START[S.fight.clamp(0, FIGHTS as i32 - 1) as usize] {
        S.road = ROAD_OPEN;
    }
    S.live = 0;
    S.frames = 0;
    S.play = 0;
    S.attrs_done = 0;
    // items: no keystone picks (cut with the drives)
    let keys_short = false;
    // sessions: the stop (banked picks, a keystone if due, the shop) opens only a session's first fight
    let sstart = SESSION_START[S.fight.clamp(0, FIGHTS as i32 - 1) as usize];
    // the stop already ran at the end of the last session (in that match): no picks, no shop here
    let stopped = S.stop_done != 0 && sstart;
    S.stop_done = 0;
    S.stop = 0;
    let sstart = sstart && !stopped;
    S.shop_due = (sstart && (S.fight > 0 || S.lp > 1) && S.no_shop == 0) as i32;
    // V3 12.9: the drive pick first, then the keystone pick (after a boss); the run's first keystone opens fight 1
    S.reward_due = if sstart && S.bank > 0 {
        RW_DRIVES
    } else if sstart && keys_short {
        RW_KEYS
    } else {
        0
    };
    let _ = started;
    S.bank_n = if S.reward_due == RW_DRIVES { S.bank } else { 0 };
    S.bank_i = (S.bank_n > 0) as i32;
    // the session's last fight holds the retail match end: the stop runs in it after the win (outcome_frame)
    // every fight holds the retail match end: the drops fly in after the last KO, then the sim ends the match
    // (a session's last fight runs its stop in the held match first)
    gd_envoy_end_hold(1);
    S.outcome = 0;
    S.stage_clock = 0;
    S.stage_switches = 0;
    S.stage_want = 0;
    S.wave = 1;
    S.waves = room_waves(S.fight as usize, sched().kind, lx());
    S.park = 0;
    S.wave_clr = 0;
    S.shift_dmg = 0;
    S.shift_paid = 0;
    S.tags = 0;
    S.strikes = 0;
    S.hold_t = [0; NP];
    S.strike_buf = [0; NP];
    S.buf_win = [0; NP];
    S.assist_seen = [0; NP];
    S.auto_hold = [0; NP];
    S.last_stocks = -1;
    S.collect = 0;
    S.won_pool = 0;
    S.tag_cd = [0; NP];
    S.tag_buf = [0; NP];
    S.cb_hits = [0; NP];
    S.cb_dmg = [0; NP];
    S.cb_cd = [0; NP];
    S.cb_due = [0; NP];
    S.dmg_win = [0; NP];
    S.tag_btn = gd_envoy_tag_button().clamp(0, 2);
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
    // items: no drops on KOs (items come only from the item room, the shop and the boss)
    let _ = f;
    S.drops_due = 0;
    S.drops_made = 0;
    items::reset_match();
    S.en_h = [0; EN_MAX];
    S.en_dmg = [0; EN_MAX];
    S.en_rcv = [0; EN_MAX];
    S.en_spawned = 0;
    if test_waves() >= 0 {
        gd_envoy_end_hold(1);
    }
    S.rw_active = 0;
    foe_drives();
    for p in 0..nh() as usize {
        for f in 0..2 {
            if has_inferno(set_of(p, f)) {
                ev(EV_INFERNO, set_of(p, f) as i32, 1);
            }
        }
    }
    // Fights 2: this room's faces join the run's recent ones (the next rooms prefer others)
    let foes = PLAN[7].clamp(0, 3) as usize;
    for r in 0..foes {
        let mut faces = [PLAN[8 + r], if S.waves >= 2 { PLAN[32 + r] } else { -1 }, -1, -1, -1];
        if r < 2 {
            for k in 0..3 {
                if 3 + k as i32 <= S.waves {
                    faces[2 + k] = PLAN[PL_GAUNT + r * 3 + k];
                }
            }
        }
        for c in faces {
            if c >= 0 && !S.seen.contains(&c) {
                S.seen[S.seen_n.rem_euclid(SEEN as i32) as usize] = c;
                S.seen_n = S.seen_n.wrapping_add(1);
            }
        }
    }
    S.crowd = if test_waves() >= 0 || old_scale() { 1000 } else { crowd_kb(ncpu()) };
    S.crowd_hits = 0;
    S.lv_on = 0;
    if S.crowd > 1000 {
        log(b"envoy: crowd fight, foes*10000 + knockback x1000 ", ncpu() * 10000 + S.crowd);
    }
    if S.waves >= 2 && ncpu() == 1 {
        log(b"envoy: gauntlet, duels ", S.waves);
    }
    log(b"envoy: fight\0", S.fight + 1);
}

/// Elites and bosses carry drives (envoy-core-v2.md Q7), rolled from the run seed: the first CPU only.
/// Loops: from loop 2 an ordinary fight's first CPU carries one; from loop 3 it carries two and the others one;
/// elites and bosses one more per loop (to 2); foe levels +1 per loop (to the level cap).
unsafe fn foe_drives() {
    let f = sched();
    let e = lx();
    let first = nh() as usize;
    for s in first..(first + ncpu().max(0) as usize).min(4) {
        let lead = s == first;
        // ordinary fights from loop 2: the first foe carries one (the retune: no two-drive elite treatment);
        // the Steep road: every foe one more
        let want = if f.kind == FK_ELITE && S.waves >= 2 {
            (e >= 1 && lead) as i32
        } else if f.kind >= FK_ELITE {
            if lead { f.foe_drives + e } else { 0 }
        } else {
            (e >= 1 && lead) as i32
        } + steep_twist() as i32;
        foe_roll(s, want.clamp(0, 2) as usize, (f.foe_level.max(1) + e).clamp(1, lvl_cap()), (s - first) as u32);
    }
}

unsafe fn foe_roll(s: usize, want: usize, level: i32, k: u32) {
    if want == 0 {
        return;
    }
    let mut h = mix(S.seed as u32 ^ 0xF0E5_0000 ^ fkey() << 4 ^ k << 20);
    let mut n = 0;
    let mut tries = 0;
    while n < want && tries < 32 {
        tries += 1;
        let key = FOE_POOL[(h % FOE_POOL.len() as u32) as usize];
        h = mix(h.wrapping_add(0x6D2B_79F5));
        let Some(id) = by_key(key) else { continue };
        if S.fdrives[s][..n].contains(&(id as i32 + 1)) {
            continue;
        }
        S.fdrives[s][n] = id as i32 + 1;
        S.flevels[s][n] = level;
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
    let mut h = mix(S.seed as u32 ^ 0xA5A5_0000 ^ fkey() << 4 ^ (S.picks as u32) << 12 ^ (p as u32) << 24);
    if S.rw_active == RW_DRIVES && S.bank_n > 0 && bank_head() == RK_LEVEL {
        // a level-up token: the cards are drives this player wears below the level cap (the fighter in play first);
        // taking one raises it a level. None to raise: a drive pick instead
        S.offers[p] = [OFFER_STOCK; 3];
        let mut n = 0;
        let a = S.active[p] as usize;
        for f in [a, 1 - a] {
            let set = set_of(p, f);
            for k in 0..open_slots() {
                let d = S.drives[set][k];
                if n < 3 && d > 0 && S.levels[set][k] < lvl_cap() && !S.offers[p][..n].contains(&(d - 1)) {
                    S.offers[p][n] = d - 1;
                    n += 1;
                }
            }
        }
        if n > 0 {
            for k in n..3 {
                S.offers[p][k] = S.offers[p][0];
            }
            return;
        }
    }
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

/// The shop's extra stock: coins for one more stock in the shared pool (and on the players in the held match).
unsafe fn buy_stock() -> bool {
    // items: a refill, up to the stock limit (never past it)
    if S.pool >= S.lim || !spend(stock_price()) {
        return false;
    }
    S.pool += 1;
    for h in 0..nh() {
        gd_envoy_apply(h, AP_STOCKS, gd_player_stocks(h) + 1);
    }
    log(b"envoy: shop stock, pool\0", S.pool);
    // the next match takes its stocks from the plan, which fight_won already sent: send it again with this stock
    make_plan();
    log(b"envoy: shop stock, plan stocks now\0", PLAN[4]);
    true
}

unsafe fn open_reward(kind: i32) {
    // test knob `buystock`: the shop buys the extra stock at once when it opens (coins given if short)
    if kind == RW_SHOP && S.test_buy != 0 {
        if HH.purse < stock_price() as i64 * COIN_UNIT {
            HH.purse = stock_price() as i64 * COIN_UNIT;
        }
        buy_stock();
    }
    S.rw_active = kind;
    S.timer = if kind == RW_SHOP { SHOP_TIME } else { REWARD_TIME };
    for p in 0..NP {
        S.rw_done[p] = (p as i32 >= nh() || (kind == RW_KEYS && key_count(p) >= keys_earned())) as i32;
        S.cursor[p] = 0;
        S.step[p] = 0;
        S.confirm[p] = 0;
        // "Leave it" (Skip) pays this for the offer (the plumbing for Uniques; nothing for an ordinary pick yet)
        S.decline[p] = 0;
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
    // test knob `buystock` also buys each player's first shop item (coins given if short): the scripted item check
    if kind == RW_SHOP && S.test_buy != 0 {
        for p in 0..nh() as usize {
            let id = S.shop_off[p][0];
            if id > 0 {
                let c = price_of(id) as i64 * COIN_UNIT;
                if HH.purse < c {
                    HH.purse = c;
                }
                spend(price_of(id));
                items::give(p, id, G_SHOP);
                items::log_item(b"shop bought", id, p as i32);
                S.shop_off[p][0] = -1;
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
        make_plan(); // the next match takes its stocks from the plan
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
    // sessions: the banked picks one after another ("Pick 1 of 3")
    if S.rw_active == RW_DRIVES && S.bank > 0 {
        bank_pop();
        if S.bank > 0 {
            S.bank_i += 1;
            open_reward(RW_DRIVES);
            return;
        }
    }
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
    if S.stop != 0 {
        // the stop is over: the held match ends, the road shows the next session
        S.stop = 0;
        S.bank_n = 0;
        make_plan(); // the stop may have changed the pool (a bought or picked stock): the next match reads the plan
        gd_envoy_end_match();
        log(b"envoy: the stop is over, fight\0", S.fight + 1);
    }
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
    if S.confirm[p] != 0 {
        let id = S.confirm_new[p] - 1;
        match confirm_frame(p, pad) {
            3 => {
                let (f, k) = ((S.confirm_at[p] >> 3) as usize, (S.confirm_at[p] & 7) as usize);
                replace(p, f, k, id as usize, G_PICK);
                done_pick(p, id, f as i32);
            }
            2 => S.step[p] = 2,
            _ => {
                if S.timer <= 0 {
                    S.confirm[p] = 0;
                    log(b"envoy: pick skipped (confirm, time), player\0", p as i32);
                    done_pick(p, -1, 0);
                }
            }
        }
        return;
    }
    let left = pad & (DPAD_LEFT | STICK_LEFT) != 0;
    let right = pad & (DPAD_RIGHT | STICK_RIGHT) != 0;
    let id = S.offers[p][S.cursor[p].clamp(0, 2) as usize];
    match S.step[p] {
        0 => {
            // the UI's list (R2 + D2): up / down walk the three rows and Skip (cursor 3, drives only);
            // left / right still cycle the three offers
            let up = pad & (DPAD_UP | STICK_UP) != 0;
            let down = pad & (DPAD_DOWN | STICK_DOWN) != 0;
            let rows = if S.rw_active == RW_KEYS { 3 } else { 4 };
            if left {
                S.cursor[p] = (S.cursor[p].min(2) + 2) % 3;
            }
            if right {
                S.cursor[p] = (S.cursor[p].min(2) + 1) % 3;
            }
            if up {
                S.cursor[p] = (S.cursor[p] + rows - 1) % rows;
            }
            if down {
                S.cursor[p] = (S.cursor[p] + 1) % rows;
            }
            if S.cursor[p] == 3 {
                if pad & BTN_A != 0 {
                    log(b"envoy: pick skipped, player ", p as i32);
                    if S.decline[p] > 0 {
                        HH.purse += S.decline[p] as i64 * COIN_UNIT;
                        log(b"envoy: offer declined, coins ", S.decline[p]);
                    }
                    done_pick(p, -1, 0);
                } else if S.timer <= 0 {
                    timeout_pick(p);
                }
                return;
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
                // never silent: the replacement waits for its confirm (a pinned drive cannot be chosen)
                let f = S.fcursor[p];
                let k = S.scursor[p].clamp(0, n - 1) as usize;
                if !pinned(p, f as usize, k) {
                    open_confirm(p, f as usize, k, id + 1, 1, -1);
                }
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

/// An item's shop price (items: by quality, +10% a loop).
unsafe fn price_of(id: i32) -> i32 {
    if id > 0 { per_loop(item_price(id), LOOP_PRICE, lx()) } else { 0 }
}

/// The shop's three items for player `p`: the shop pool first, then the item pool (the slice's shop pool is small);
/// rerolls give new ones.
unsafe fn roll_shop(p: usize) {
    S.shop_off[p] = [-1; 3];
    let salt = 0x5A0 + S.rerolls as u32 * 7 + S.picks as u32 * 131;
    for n in 0..3 {
        let taken = S.shop_off[p];
        let mut id = items::roll(p, PL_SHOP, salt + n as u32, &taken);
        if id == 0 {
            id = items::roll(p, PL_ITEM, salt + 0x40 + n as u32, &taken);
        }
        S.shop_off[p][n] = if id > 0 { id } else { -1 };
    }
}

unsafe fn open_shop(p: usize) {
    S.shop_cursor[p] = 0;
    S.shop_step[p] = 0;
    S.shop_fighter[p] = S.active[p];
    S.shop_msg[p] = SM_NONE;
    S.shop_pending[p] = -1;
    S.shop_rerolls = 0;
    S.shop_lv_done[p] = 0;
    roll_shop(p);
}

unsafe fn reroll_cost() -> i32 {
    per_loop(PRICE_REROLL + PRICE_REROLL * S.shop_rerolls, LOOP_PRICE, lx())
}

/// Restoring a stock: the base price to loop 5, then +25 a loop.
unsafe fn stock_price() -> i32 {
    let _ = PRICE_STOCK;
    items::STOCK_REFILL + 25 * (lp() - 5).max(0)
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
    if S.drives[set][k] > 0 && (1..lvl_cap()).contains(&lv) {
        // to 2, to 3, then +60 a level past 3; +25% per loop
        // the step only, no loop markup (the retune)
        if lv <= 2 { PRICE_LEVEL[lv as usize - 1] } else { PRICE_LEVEL[1] + 60 * (lv - 2) }
    } else {
        0
    }
}

/// The item shop (items slice): up / down walk the three items, a stock refill, a reroll and Leave; A buys. Items
/// are bought outright (no slots, no level-ups, no reorder); the purse is the party's.
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
        if pad & (DPAD_UP | STICK_UP) != 0 {
            S.shop_cursor[p] = (S.shop_cursor[p] + SHOP_ROWS - 1) % SHOP_ROWS;
        }
        if pad & (DPAD_DOWN | STICK_DOWN) != 0 {
            S.shop_cursor[p] = (S.shop_cursor[p] + 1) % SHOP_ROWS;
        }
        // test knob `buystock`: the first item row is bought too when the purse allows (the scripted shop check)
        if pad & BTN_A == 0 {
            continue;
        }
        let row = S.shop_cursor[p];
        S.shop_msg[p] = match row {
            0..=2 => {
                let id = S.shop_off[p][row as usize];
                if id <= 0 {
                    SM_CANT
                } else if spend(price_of(id)) {
                    items::give(p, id, G_SHOP);
                    items::log_item(b"shop bought", id, p as i32);
                    S.shop_off[p][row as usize] = -1;
                    SM_BOUGHT
                } else {
                    SM_POOR
                }
            }
            SHOP_STOCK => {
                if S.pool >= S.lim {
                    SM_CANT
                } else if buy_stock() {
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
            _ => {
                S.rw_done[p] = 1;
                log(b"envoy: shop left, coins\0", coins());
                SM_NONE
            }
        };
    }
}

#[allow(dead_code)]
unsafe fn shop_frame_drives() {
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
        if S.test_protect != 0 && S.shop_step[p] == 0 && S.confirm[p] == 0 {
            test_protect(p);
        }
        if S.confirm[p] != 0 {
            let (cf, ck) = ((S.confirm_at[p] >> 3) as usize, (S.confirm_at[p] & 7) as usize);
            let cset = set_of(p, cf);
            match confirm_frame(p, pad) {
                3 if S.confirm_src[p] == 3 => {
                    let c = scrap_price(cset, ck);
                    if c > 0 {
                        HH.purse += c as i64 * COIN_UNIT;
                        log_drive(b"scrapped", S.drives[cset][ck] as usize - 1, c);
                        S.drives[cset][ck] = 0;
                        S.levels[cset][ck] = 0;
                        HH.chg[cset][ck] = 0;
                        S.pins[cset] &= !(1 << ck);
                        S.shop_msg[p] = SM_SCRAPPED;
                    }
                }
                3 => {
                    let row = S.confirm_row[p].clamp(0, 2) as usize;
                    let id = S.shop_off[p][row];
                    S.shop_msg[p] = if id >= 0 && id == S.confirm_new[p] - 1 && spend(price_of(id)) {
                        replace(p, cf, ck, id as usize, G_SHOP);
                        S.shop_off[p][row] = -1;
                        SM_REPLACED
                    } else {
                        SM_POOR
                    };
                    S.shop_step[p] = 0;
                }
                2 => {
                    S.shop_msg[p] = SM_KEPT;
                    if S.confirm_src[p] == 2 {
                        S.shop_step[p] = 0;
                    }
                }
                _ => {}
            }
            if S.timer <= 0 {
                S.confirm[p] = 0;
            }
            continue;
        }
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
                    SHOP_LEVEL if S.shop_lv_done[p] != 0 => SM_CANT,
                    SHOP_LEVEL => {
                        S.shop_step[p] = 1;
                        S.shop_pending[p] = SHOP_LEVEL;
                        S.scursor[p] = 0;
                        SM_NONE
                    }
                    SHOP_STOCK => {
                        if buy_stock() {
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
                            // one level-up a visit: back to the rows
                            S.shop_lv_done[p] = 1;
                            S.shop_step[p] = 0;
                            SM_LEVELED
                        } else {
                            SM_POOR
                        };
                    } else if (0..3).contains(&row) {
                        let id = S.shop_off[p][row as usize];
                        if id < 0 || coins() < price_of(id) {
                            S.shop_msg[p] = SM_POOR;
                        } else if pinned(p, f, k) {
                            S.shop_msg[p] = SM_PINNED;
                            continue;
                        } else {
                            // never silent: "Replace X with Y for good?" (the coins are taken on Replace)
                            open_confirm(p, f, k, id + 1, 2, row);
                            continue;
                        }
                    }
                    S.shop_step[p] = 0;
                }
            }
            2 => {
                // reorder: pick the drive to move; the screen spans both of the player's rows (up / down);
                // Y pins / unpins, X scraps (through the confirm)
                if left {
                    S.scursor[p] = (S.scursor[p] + n - 1) % n;
                }
                if right {
                    S.scursor[p] = (S.scursor[p] + 1) % n;
                }
                if (up || down) && two_rows(p) {
                    S.shop_fighter[p] = 1 - S.shop_fighter[p].clamp(0, 1);
                }
                let k = S.scursor[p].clamp(0, n - 1) as usize;
                if pad & BTN_B != 0 {
                    S.shop_step[p] = 0;
                } else if pad & BTN_Y != 0 && S.drives[set][k] > 0 {
                    S.pins[set] ^= 1 << k;
                    S.shop_msg[p] = SM_PINNED;
                    log(b"envoy: pin, set*10 + slot (+100 pinned)\0", set as i32 * 10 + k as i32 + 100 * (S.pins[set] >> k & 1));
                } else if pad & BTN_X != 0 && S.drives[set][k] > 0 {
                    if pinned(p, f, k) {
                        S.shop_msg[p] = SM_PINNED;
                    } else {
                        open_confirm(p, f, k, 0, 3, -1);
                    }
                } else if pad & BTN_A != 0 && S.drives[set][k] > 0 {
                    S.shop_step[p] = 3;
                }
            }
            _ => {
                // reorder: move the lifted drive one slot at a time (a swap with its neighbour)
                let k = S.scursor[p].clamp(0, n - 1) as usize;
                if (up || down) && two_rows(p) {
                    // to the same slot of the player's other fighter (a swap when it is filled)
                    S.shop_msg[p] = if move_across(p, f, k) { SM_MOVED } else { SM_CANT };
                    continue;
                }
                let to = if left && k > 0 { k - 1 } else if right && (k as i32) < n - 1 { k + 1 } else { k };
                if to != k {
                    S.drives[set].swap(k, to);
                    S.levels[set].swap(k, to);
                    HH.chg[set].swap(k, to);
                    S.target[set].swap(k, to);
                    let (bk, bt) = (S.pins[set] >> k & 1, S.pins[set] >> to & 1);
                    S.pins[set] = S.pins[set] & !(1 << k | 1 << to) | bk << to | bt << k;
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
    // Quick Tag: the tag-cancel window opens on any damage the fighter itself deals (a move, a throw, a projectile)
    if cm == CM_MOVE {
        S.dmg_win[p] = QT_WINDOW;
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
    // a fire lands through the foe's toughness and is capped on what lands (the retune); a foe's fire on a player
    // keeps the plain cap
    let milli = match S.fx_mode {
        FX_FIRE if s >= nh() => ((milli as i64 * foe_def(s) as i64 / 1000) as i32).min(S.fx_cap),
        FX_FIRE => milli.min(S.fx_cap),
        // rule 2 at the point the percent is applied, whoever set the fire context (a fire, a queued piece of one)
        FX_FOE => milli.min(if old_scale() { S.fx_cap } else { S.fx_cap.min(FOE_FIRE_CAP) }),
        _ => milli,
    };
    if milli <= 0 {
        return 0;
    }
    gd_envoy_percent(s, 0, milli);
    if S.fx_mode == FX_FOE && kb_log() && (0..nh()).contains(&s) && KB_FIRE[0] > 0 {
        log_drive(b"kblog fire", KB_FIRE[0] as usize - 1, milli);
    }
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
unsafe fn launch(owner: usize, id: usize, at: P, effect: u8, milli: i32, level: i32, state: usize) -> bool {
    let wide = (if level >= MAX_LEVEL { 1.25 } else { 1.0 }) * (1.0 + 0.2 * state as f32);
    let kb = state as i32 * 6;
    let (r, angle, kbg, bkb) = match effect {
        E_NOVA => (16.0 * wide, 361, 70 + kb, 35 + kb),
        E_SHOCKWAVE => (13.0 * wide, 80, 55 + kb, 30 + kb),
        _ => (9.0 * wide, 90, 75 + kb, 40 + kb), // Spire: straight up
    };
    // rule 2: a foe's blast is armed at cap / its attack ratio, rounded down. The engine scales a hit's knockback
    // (not its damage) by the attacker's attack ratio, so this keeps a stronger foe's blast from launching much
    // harder; what lands is capped by the engine itself, below
    let pct = if owner >= SETS && !old_scale() {
        let atk = foe_atk_now(owner as i32 - OWNER_FOE).max(1000) as i64;
        if blast_over() { 30 } else { (milli.min(FOE_FIRE_CAP) as i64 * 1000 / atk / 1000) as i32 }
    } else {
        (milli + 500) / 1000
    };
    let rc = gd_envoy_blast(owner_slot(owner), at.x, at.y, r, pct, angle, kbg, bkb);
    if rc == 0 {
        if owner >= SETS {
            // rule 2 on what lands: the engine caps this capsule's damage on each victim at the last point before
            // it is applied (after staling, the victim's own multipliers and anything added later), and tags it
            // for the logs
            gd_envoy_blast_cap(owner_slot(owner), if old_scale() { 0 } else { FOE_FIRE_CAP }, (id as i32 + 1) | (level.clamp(0, 15) << 8));
        }
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
    // the cap is on what lands (after the foe's toughness, in hurt), not on the number before it
    let foe_new = !player && !old_scale();
    let lv = if foe_new { level.min(MAX_LEVEL) } else { level };
    let mut n = (heat::fire_damage_raw(c.number, lv, st) as i64 * frac as i64 / 1000) as i32;
    S.fx_cap = if c.launch != 0 { LAUNCH_CAP } else { FIRE_CAP };
    if foe_new {
        // rule 2: a foe's fire, blasts too (whose damage the engine then scales by the foe's attack ratio)
        n = n.min(FOE_FIRE_CAP);
        S.fx_cap = FOE_FIRE_CAP;
    }
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
    KB_FIRE = if player { [0, 0] } else { [id as i32 + 1, level] };
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
            if tgt >= 0 && launch(owner, id, centre, c.effect, n, level, st) {
                fl |= FL_LAUNCH;
            } else {
                tgt = hurt_around(owner, me, r, n, -1);
            }
        }
        E_SPIRE => {
            tgt = near(target);
            if let Some(p) = pos(tgt) {
                at = p;
                if launch(owner, id, P { x: p.x, y: p.y + 6.0 }, E_SPIRE, n, level, st) {
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
    KB_FIRE = [0, 0];
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
        if kb_log() && (0..nh()).contains(&s) {
            KB_PRE[s as usize] = kb_percent(h);
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
        // the combo breaker: the hit that would make it CB_HITS (or take the combo to CB_DAMAGE) does not land
        if (0..nh()).contains(&v) && cb_foe_hit(a) && cb_hit(v as usize, dmg) {
            gd_envoy_absorb(v);
            gd_envoy_percent(v, 0, -dmg);
            return;
        }
        // crowd knockback: in a fight with 3+ foes every foe launches farther (never a player; the boss never)
        if S.crowd > 1000 && is_foe(v) && !is_boss(v) {
            gd_envoy_kb_scale(v, S.crowd);
            S.crowd_hits += 1;
            log(b"envoy: crowd knockback, slot*10000 + x1000 ", v * 10000 + S.crowd);
        }
        // what foe drive blasts put into this hit (the engine's per-capsule record of the frame, after its cap)
        let human_v = (0..nh()).contains(&v);
        let mut land = [0i32; 9];
        let foe_blast = if human_v && gd_envoy_landed(v, land.as_mut_ptr()) > 0 && is_foe(land[3]) { land[1].min(dmg) } else { 0 };
        if kb_log() && human_v && (is_foe(a) || foe_blast > 0) {
            // the percent the fighter gained: read before (the pre hook) and after the hit was applied
            let (pre, post) = (KB_PRE[v as usize], kb_percent(h));
            let got = if pre >= 0 && post >= pre { post - pre } else { dmg };
            KB_W[v as usize] = [S.play, got, a, land[0], land[1], land[2], land[3], land[4], land[5], land[6], land[7], land[8]];
        }
        if human_v {
            KB_PRE[v as usize] = -1;
        }
        push_rec(a, v, dmg, tags, ast);
        // Marked: the hit deals more (half on a boss): word damage for the attacker
        if marked(v) {
            let more = if is_boss(v) { BOSS_MARK_MORE } else { MARK_MORE };
            let mut extra = (dmg as i64 * more as i64 / 1000) as i32;
            if human_v && !old_scale() {
                // rule 2: a foe's Mark on a player is a foe drive's damage too: with the blast part of this hit it
                // adds at most FOE_FIRE_CAP
                extra = extra.min((FOE_FIRE_CAP - foe_blast).max(0));
                if kb_log() && extra > 0 {
                    log(b"envoy: kblog mark extra\0", extra);
                }
            }
            if extra > 0 {
                gd_envoy_percent(v, 0, extra);
                push_rec(a, v, extra, T_WORD, -1);
            }
        }
    }
}

// ---- the combo breaker (CB_*)

/// A hit from attacker slot `a` counts toward a combo: a foe CPU, or no fighter while an enemy wave is out.
unsafe fn cb_foe_hit(a: i32) -> bool {
    is_foe(a) || (a < 0 && S.en_h.iter().any(|&h| h > 0))
}

/// Player `p` takes a foe hit of `dmg` (x1000): count it; true when this hit fires the breaker instead of landing.
unsafe fn cb_hit(p: usize, dmg: i32) -> bool {
    if S.cb_due[p] != 0 {
        return true; // a break is already on its way this frame: the hit is swallowed too
    }
    if S.cb_hits[p] > 0 && S.play - S.cb_last[p] >= CB_GAP {
        S.cb_hits[p] = 0;
        S.cb_dmg[p] = 0;
    }
    S.cb_last[p] = S.play;
    if S.cb_cd[p] <= 0 && (S.cb_hits[p] + 1 >= CB_HITS || S.cb_dmg[p] + dmg >= CB_DAMAGE) {
        S.cb_due[p] = 1;
        return true;
    }
    S.cb_hits[p] += 1;
    S.cb_dmg[p] = S.cb_dmg[p].saturating_add(dmg);
    false
}

/// Per live unfrozen frame: a due break bursts the player free (the host: no launch, out of hitstun, intangible
/// with its flash; a no-damage blast pushes the foes near back; the event rings and sounds in envoy_ui); the cooldown
/// steps; a combo ends when the player is actionable on the ground or after CB_GAP frames without a foe hit.
unsafe fn combo_frame() {
    for p in 0..nh() as usize {
        let ps = p as i32;
        if S.cb_cd[p] > 0 {
            S.cb_cd[p] -= 1;
        }
        if S.cb_due[p] != 0 {
            S.cb_due[p] = 0;
            let hits = S.cb_hits[p] + 1;
            let pct = S.cb_dmg[p] / 1000;
            S.cb_hits[p] = 0;
            S.cb_dmg[p] = 0;
            S.cb_cd[p] = CB_COOLDOWN;
            S.cb_n += 1;
            let r = gd_envoy_combo_break(ps, CB_INTANGIBLE, CB_FREEZE);
            if let Some(at) = pos(ps) {
                gd_envoy_blast(ps, at.x, at.y + 8.0, CB_PUSH_R, 0, CB_PUSH_ANGLE, CB_PUSH_KBG, CB_PUSH_BKB);
            }
            ev(EV_COMBOBREAK, ps, hits << 16 | pct.clamp(0, 0xFFFF));
            log(b"envoy: combo break, player*10000 + hits*1000 + combo percent ", ps * 10000 + hits * 1000 + pct.clamp(0, 999));
            log(b"envoy: combo break at live frame ", S.play);
            log(b"envoy: combo breaks so far (+1000000: the host refused) ", S.cb_n + if r == 0 { 0 } else { 1_000_000 });
            continue;
        }
        if S.cb_hits[p] == 0 {
            continue;
        }
        let h = gd_fighter_from_slot(ps);
        let m = if h != 0 { gd_fighter_get_action_state(h) } else { 0 };
        // actionable on the ground: Wait, walk, dash, run, turn, jump squat; squat; or dead (a stock lost)
        let free = (14..=24).contains(&m) || (39..=41).contains(&m) || (0..=10).contains(&m);
        if free || S.play - S.cb_last[p] >= CB_GAP {
            S.cb_hits[p] = 0;
            S.cb_dmg[p] = 0;
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
                items::on_hit(p, v, tags, ast);
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
        EVB[p] = ev_bits;
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
                items::on_wavedash(p);
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
    // items: a KO pays flat coins (pay / 100: a fighter 10, a minion 5, an elite 15), no longer scaled by Heat
    let c = pay as i64 / 100 * COIN_UNIT;
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
    // items: the cash-out flare (Heat / 15 percent on the other foes) is cut with the Heat rewards
    let fl = 0;
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
    // items: no KO drops; Coin Magnet pays
    items::on_ko(s);
}

/// Per unfrozen live frame: the stock edges (KOs dealt and cash-outs, stocks lost, the shared pool), the boss's
/// damage and breaks, the drops, Burning, Volley.
unsafe fn moments_frame() {
    // the boss carrier can arm after the first live frame (fight_setup ran before Petey's fight began): pick it up
    // here, with the damage baselines, or no break is ever counted and its KO reads as a plain foe KO
    if S.boss_slot < 0 {
        let mut w = [0u32; 6];
        let b = gd_envoy_boss_dmg(w.as_mut_ptr());
        if b >= 0 {
            S.boss_slot = b;
            for p in 0..4 {
                S.boss_dmg[p] = w[p] as i32;
            }
            S.boss_hp = w[4] as i32;
            S.boss_hp_max = w[5] as i32;
            log(b"envoy: boss carrier, slot ", b);
        }
    }
    let mut foes = 0;
    for s in 0..4 {
        let n = if (0..nh() + ncpu()).contains(&s) { gd_player_stocks(s).max(0) } else { 0 };
        let i = s as usize;
        if n < S.stocks_each[i] {
            if S.stage_switches > 0 && S.lv_on != 1 && S.stage_clock < SWITCH_KO_WINDOW && S.sw_hit[i] == 0 {
                log(b"envoy: stock lost soon after a stage switch (see the KO lines), slot*10000 + frames since\0", s * 10000 + S.stage_clock);
            }
            if is_foe(s) && s != S.boss_slot {
                foe_ko(s, n == 0);
                // rooms: a foe out of stocks while the room has another wave is parked for it (stocks set back now,
                // before its KO countdown ends and the retail "player out" path could run)
                if n == 0 && S.wave < S.waves && PLAN[35 + (s - nh()).clamp(0, 2) as usize] > 0 {
                    let row = (s - nh()).clamp(0, 2) as usize;
                    // an elite room's plain duels before the elite's come back with the plain foe's stocks
                    let back = if row == 0 && elite_wave() > 0 && S.wave + 1 < elite_wave() { PLAN[14].max(1) } else { PLAN[35 + row] };
                    gd_envoy_apply(s, AP_STOCKS, back);
                    gd_envoy_wave_park(s);
                    S.park |= 1 << s;
                    S.stocks_each[i] = back;
                    log(b"envoy: foe parked for the next wave, slot\0", s);
                    if is_foe(s) {
                        foes += back;
                    }
                    clear_status(i);
                    continue;
                }
                // a stock-based boss or elite that can spare the stock breaks; on the Shifting road any foe KO
                // switches the stage (never the fight's last: the win's collect owns that moment)
                if sched().kind >= FK_ELITE && n > 0 {
                    stage_request(SH_BREAK);
                }
            } else if (0..nh()).contains(&s) {
                kb_ko(i);
                // a lost stock costs a quarter of Heat (rooms; was half)
                HH.heat[i] -= HH.heat[i] / 4;
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
                // no switch: Petey's arena stays put (tag_swap::stage_begin refuses while the boss is active)
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
                if kb_log() && (0..nh()).contains(&s) {
                    log(b"envoy: kblog burn tick\0", d);
                }
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
    let mut h = mix(S.seed as u32 ^ 0xD20F_0000 ^ fkey() << 8 ^ (S.drops_made as u32) << 16 ^ p as u32);
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
    // items: the floor points hold pedestals now (an item id, not a drive)
    items::ped_take(p, k, how);
}

#[allow(dead_code)]
unsafe fn take_drop_drive(p: usize, k: usize, how: i32) {
    let id = S.drop[p][k][1] as usize - 1;
    S.gain_from[p] = [S.drop[p][k][2], S.drop[p][k][3], p as i32];
    let a = S.active[p] as usize;
    let f = if can_take(p, a, id) { a } else { 1 - a };
    S.drop[p][k][0] = 2;
    if gain(p, f, id, how) == 0 {
        // sessions: no room (a full row, or the top level) banks it as one more pick at the stop
        bank_push(RK_PICK);
        let n = S.gain[p][5] + 1;
        S.gain[p] = [S.play, id as i32 + 1, f as i32, 0, G_BANKED, n];
        log_drive(b"drop banked (no room)", id, p as i32);
    }
}

// ---- drive protection: a confirm before any replacement; pin, scrap, move between the player's two fighters

unsafe fn pinned(p: usize, f: usize, k: usize) -> bool {
    S.pins[set_of(p, f)] >> k & 1 != 0
}

/// The player has two rows (not shared drives, a partner in the party).
unsafe fn two_rows(p: usize) -> bool {
    S.shared == 0 && gd_tag_partner(p as i32) >= 0
}

/// Scrapping slot `k` of `set` pays a third of its shop value: the tier price plus its level steps (0: empty or
/// pinned). Never more than buying it cost.
unsafe fn scrap_price(set: usize, k: usize) -> i32 {
    let d = S.drives[set][k];
    if d <= 0 || S.pins[set] >> k & 1 != 0 {
        return 0;
    }
    let lv = S.levels[set][k].max(1);
    let mut v = PRICE_TIER[POOL[d as usize - 1].tier.min(2) as usize];
    for l in 1..lv {
        v += if l <= 2 { PRICE_LEVEL[l as usize - 1] } else { PRICE_LEVEL[1] + 60 * (l - 2) };
    }
    (v / 3).max(1)
}

unsafe fn open_confirm(p: usize, f: usize, k: usize, new_id1: i32, src: i32, row: i32) {
    S.confirm[p] = 1;
    S.confirm_cur[p] = 0; // Keep
    S.confirm_at[p] = (f as i32) << 3 | k as i32;
    S.confirm_new[p] = new_id1;
    S.confirm_src[p] = src;
    S.confirm_row[p] = row;
}

/// The confirm's input: 1 still open, 2 kept (B, or A on Keep), 3 accepted (A on Replace / Scrap).
unsafe fn confirm_frame(p: usize, pad: u32) -> i32 {
    if pad & (DPAD_LEFT | STICK_LEFT | DPAD_RIGHT | STICK_RIGHT | DPAD_UP | STICK_UP | DPAD_DOWN | STICK_DOWN) != 0 {
        S.confirm_cur[p] = 1 - S.confirm_cur[p];
    }
    let r = if pad & BTN_B != 0 || (pad & BTN_A != 0 && S.confirm_cur[p] == 0) {
        2
    } else if pad & BTN_A != 0 {
        3
    } else {
        return 1;
    };
    S.confirm[p] = 0;
    log(if r == 3 { b"envoy: confirm, replaced (src)\0" } else { b"envoy: confirm, kept (src)\0" }, S.confirm_src[p]);
    r
}

/// Move slot `k` of fighter `f` to the same slot of the player's other fighter (a swap when it is filled). Refused
/// when either fighter would then wear the same drive twice.
unsafe fn move_across(p: usize, f: usize, k: usize) -> bool {
    let (a, o) = (set_of(p, f), set_of(p, 1 - f));
    if a == o || k >= open_slots() {
        return false;
    }
    let (da, dob) = (S.drives[a][k], S.drives[o][k]);
    let dup = |s: usize, d: i32| d > 0 && (0..SLOTS).any(|j| j != k && S.drives[s][j] == d);
    if da <= 0 || dup(o, da) || dup(a, dob) {
        return false;
    }
    S.drives[a][k] = dob;
    S.drives[o][k] = da;
    let lv = S.levels[a][k];
    S.levels[a][k] = S.levels[o][k];
    S.levels[o][k] = lv;
    let c = HH.chg[a][k];
    HH.chg[a][k] = HH.chg[o][k];
    HH.chg[o][k] = c;
    let t = S.target[a][k];
    S.target[a][k] = S.target[o][k];
    S.target[o][k] = t;
    let (pa, po) = (S.pins[a] >> k & 1, S.pins[o] >> k & 1);
    S.pins[a] = S.pins[a] & !(1 << k) | po << k;
    S.pins[o] = S.pins[o] & !(1 << k) | pa << k;
    S.shop_fighter[p] = 1 - f as i32;
    log(b"envoy: drive moved across, set*10 + slot\0", o as i32 * 10 + k as i32);
    true
}

/// Test knob `protect` (synctests): at a shop, pin slot 0, try to scrap it (refused), scrap slot 1, move slot 2 to
/// the partner, and send a pinned-slot replace through the confirm path (refused); once per shop visit.
unsafe fn test_protect(p: usize) {
    if S.shop_msg[p] != SM_NONE {
        return;
    }
    let f = S.shop_fighter[p].clamp(0, 1) as usize;
    let set = set_of(p, f);
    if S.drives[set][0] > 0 {
        S.pins[set] |= 1;
        log(b"envoy: pin, set*10 + slot (+100 pinned)\0", set as i32 * 10 + 100);
        log(b"envoy: test scrap pinned refused\0", (scrap_price(set, 0) == 0) as i32);
    }
    if S.drives[set][1] > 0 {
        let c = scrap_price(set, 1);
        HH.purse += c as i64 * COIN_UNIT;
        log_drive(b"scrapped", S.drives[set][1] as usize - 1, c);
        S.drives[set][1] = 0;
        S.levels[set][1] = 0;
        HH.chg[set][1] = 0;
    }
    if two_rows(p) && S.drives[set][2] > 0 {
        let _ = move_across(p, f, 2);
        S.shop_fighter[p] = f as i32;
    }
    log(b"envoy: test replace pinned refused\0", pinned(p, f, 0) as i32);
    S.shop_msg[p] = SM_SCRAPPED;
    S.test_protect = 0; // once a run
}

// ---- the forked road and the bank (envoy-macro-loop.md M1, first slice)

/// Push a reward token on the bank (dropped past BANK_MAX).
unsafe fn bank_push(k: i32) {
    if S.bank < BANK_MAX && k > 0 {
        S.bank_q |= (k & 15) << (4 * S.bank);
        S.bank += 1;
    }
}

/// The next token to open (RK_*; 0 when empty).
unsafe fn bank_head() -> i32 {
    if S.bank > 0 { S.bank_q & 15 } else { 0 }
}

unsafe fn bank_pop() {
    if S.bank > 0 {
        S.bank_q = (S.bank_q as u32 >> 4) as i32;
        S.bank -= 1;
    }
}

/// Roads on offer at the road before fight `fight` (0-based): two before a loop's first session, else none.
fn roads_offered_at(fight: i32) -> i32 {
    if fight == 0 { 3 } else { 0 }
}

fn road_kind(i: i32) -> i32 {
    match i {
        1 => ROAD_STEEP,
        2 => ROAD_SHIFT,
        _ => ROAD_OPEN,
    }
}

fn road_twists(kind: i32) -> i32 {
    match kind {
        ROAD_STEEP => TW_FOE_DRIVE,
        ROAD_SHIFT => TW_FAST_SHIFT,
        _ => 0,
    }
}

/// The Shifting road's session (fights 1-3): the stage also switches on a foe KO (not the fight's last).
unsafe fn shifting() -> bool {
    road_twists(S.road) & TW_FAST_SHIFT != 0 && S.fight < 2
}

/// Ask for a stage switch (stage_frame fires it when the guards allow). A break outranks the Shifting road's reasons.
unsafe fn stage_request(why: i32) {
    if S.stage_want == 0 || why == SH_BREAK {
        S.stage_want = why;
    }
}

/// The token road `kind` banks for its session's fight `fight` (0-based); outside a forked session, a drive pick.
fn road_prize_at(kind: i32, fight: i32) -> i32 {
    if kind == ROAD_STEEP && fight == 1 { RK_LEVEL } else { RK_PICK }
}

/// Every foe carries one more drive: the Steep road's twist, in its session (fights 1-3).
unsafe fn steep_twist() -> bool {
    road_twists(S.road) & TW_FOE_DRIVE != 0 && S.fight < 2
}

/// A foe slot's toughness as the plan set it (damage taken x1000).
unsafe fn foe_def(s: i32) -> i32 {
    let i = (s - nh()).clamp(0, 2) as usize;
    if PLAN[20 + i] > 0 { PLAN[20 + i] } else { 1000 }
}

/// The road screen (out of a match, a fork on offer): P1's left / right moves the cursor, P2's plants the flag.
unsafe fn road_frame() {
    let n = roads_offered_at(if S.state == RUN_LIVE { S.fight } else { gd_envoy_start_fight() });
    if n == 0 {
        return;
    }
    for p in 0..(1 + (gd_envoy_coop() != 0 || S.nh == 2) as i32) {
        let pad = gd_pad_pressed(p);
        let d = (pad & (DPAD_RIGHT | STICK_RIGHT) != 0) as i32 - (pad & (DPAD_LEFT | STICK_LEFT) != 0) as i32;
        if d == 0 {
            continue;
        }
        if p == 0 {
            S.road_cur = (S.road_cur + d).clamp(0, n - 1);
            log(b"envoy: road cursor ", S.road_cur);
        } else {
            S.road_flag = ((S.road_flag - 1).max(0) + d).clamp(0, n - 1) + 1;
        }
    }
}

/// One frame of the fly-in (S.collect 1..=FLY_TIME): each drop on the floor moves toward its owner's fighter, and
/// is taken (worn, a level-up, or banked) on the last frame. A player not on the stage takes it at once.
unsafe fn fly_frame() {
    let left = (FLY_TIME - S.collect + 1).max(1);
    for p in 0..nh() as usize {
        for k in 0..DROPS {
            if S.drop[p][k][0] != 1 {
                continue;
            }
            let Some(me) = pos(p as i32) else {
                take_drop(p, k, G_SWEPT);
                continue;
            };
            if S.collect == 1 {
                items::log_item(b"pedestal flies in", S.drop[p][k][1], p as i32);
            }
            let (tx, ty) = ((me.x * 100.0) as i32, (me.y * 100.0) as i32 + 800);
            S.drop[p][k][2] += (tx - S.drop[p][k][2]) / left;
            S.drop[p][k][3] += (ty - S.drop[p][k][3]) / left;
            if left == 1 {
                take_drop(p, k, G_SWEPT);
            }
        }
    }
}

unsafe fn drops_on_floor() -> bool {
    S.drop.iter().any(|q| q.iter().any(|d| d[0] == 1))
}

/// Per live frame: the ratios (keystone prices, Glass Core) and the speed of every fighter, written from a stored
/// base by the engine (`gd_envoy_speed`): nothing drifts, and a swapped-in or respawned fighter gets its own at once.
unsafe fn attrs_frame() {
    for p in 0..nh() as usize {
        let mut atk = if S.test_atk > 0 { S.test_atk } else { 1000 };
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
        // items: the run's stocks are the truth (a test knob or a continued run may differ from the plan's)
        if S.pool > 0 && gd_player_stocks(p as i32) != S.pool && gd_player_stocks(p as i32) > 0 {
            gd_envoy_apply(p as i32, AP_STOCKS, S.pool);
        }
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

/// The tag cooldown after a tag (Lock doubles it).
unsafe fn tag_cooldown(p: usize) -> i32 {
    let base = if S.tag_btn == 2 { TAG_COOLDOWN } else { QT_COOLDOWN };
    if has_price(p, P_LOCK) { base * 2 } else { base }
}

/// Quick Tag: Y (or X) tags at once, no freeze; legal when free to act, or in any state but the refused ones for
/// QT_WINDOW frames after the player dealt damage. A press waits QT_BUFFER frames for a legal frame.
unsafe fn quick_tag_frame() {
    let btn = if S.tag_btn == TAG_BTN_X { BTN_X } else { BTN_Y };
    for p in 0..nh() as usize {
        // test knob `tagauto`: each player presses Tag every 150 frames (player 2 offset by 40), for synctests;
        // every other press is held to a swap, the rest are taps (strikes)
        let auto = S.tag_auto != 0 && (S.play + 40 * p as i32) % 150 == 0;
        if auto {
            S.auto_hold[p] = if ((S.play + 40 * p as i32) / 150) % 2 == 1 { QT_HOLD + 2 } else { 1 };
        }
        let held = gd_pad_held(p as i32) & btn != 0 || S.auto_hold[p] > 1;
        if S.auto_hold[p] > 0 {
            S.auto_hold[p] -= 1;
        }
        let win = S.dmg_win[p] > 0;
        if S.dmg_win[p] > 0 {
            S.dmg_win[p] -= 1;
        }
        if S.tag_cd[p] > 0 {
            S.tag_cd[p] -= 1;
        }
        // the strike's end edge
        let fr = gd_tag_assist_frames(p as i32);
        if fr == 0 && S.assist_seen[p] > 0 {
            ev(EV_ASSIST_END, p as i32, S.assist_seen[p]);
        }
        S.assist_seen[p] = fr;
        // tap or hold: decided at the release or at QT_HOLD frames
        if auto || gd_pad_pressed(p as i32) & btn != 0 {
            S.hold_t[p] = 1;
            S.hold_win[p] = win as i32;
            let st = gd_pad_stick(p as i32);
            let (x, y) = ((st & 0xFF) as u8 as i8 as i32, (st >> 8 & 0xFF) as u8 as i8 as i32);
            S.hold_dir[p] = if y >= ASSIST_STICK && y >= x.abs() {
                1
            } else if y <= -ASSIST_STICK && -y >= x.abs() {
                2
            } else {
                0
            };
        } else if S.hold_t[p] > 0 {
            if held {
                S.hold_t[p] += 1;
                if S.hold_t[p] >= QT_HOLD {
                    S.hold_t[p] = 0;
                    S.tag_buf[p] = QT_BUFFER;
                    S.buf_win[p] = S.hold_win[p];
                }
            } else {
                S.hold_t[p] = 0;
                S.strike_buf[p] = QT_BUFFER;
            }
        }
        if S.strike_buf[p] > 0 {
            S.strike_buf[p] -= 1;
            assist_try(p);
        }
        if S.tag_buf[p] <= 0 {
            continue;
        }
        S.tag_buf[p] -= 1;
        if S.tag_cd[p] > 0 || gd_tag_partner(p as i32) < 0 {
            continue;
        }
        let win = win || S.buf_win[p] != 0;
        let rc = gd_tag_quick(p as i32, win as i32, QT_PAUSE);
        if rc != 0 {
            if S.tag_buf[p] == 0 {
                let h = gd_fighter_from_slot(p as i32);
                log(b"envoy: tag refused, rc*1000 - action state\0", rc * 1000 - if h != 0 { gd_fighter_get_action_state(h) } else { 0 });
            }
            continue;
        }
        S.tag_buf[p] = 0;
        S.buf_win[p] = 0;
        S.tag_cd[p] = tag_cooldown(p);
        S.tags += 1;
        log(b"envoy: quick tag, player*100 + count\0", p as i32 * 100 + S.tags);
        on_tag(p);
        ev(EV_TAG, p as i32, S.active[p]);
        if win && S.handoff_off == 0 {
            handoff(p);
        }
    }
}

/// Assist Strike (a tap of Tag): with ASSIST_COST Heat the partner jumps in and strikes (the move by the stick at the
/// press); under it, nothing but the bar's shake. A refusal (a strike out, the player not actionable) stays buffered.
unsafe fn assist_try(p: usize) {
    if HH.heat[p] < ASSIST_COST * HEAT_TICK {
        S.strike_buf[p] = 0;
        ev(EV_HEAT_SHORT, p as i32, heat::milli(HH.heat[p]));
        log(b"envoy: strike refused, heat x1000\0", heat::milli(HH.heat[p]));
        return;
    }
    if gd_tag_partner(p as i32) < 0 {
        S.strike_buf[p] = 0;
        return;
    }
    let mv = gd_tag_assist(p as i32, S.hold_dir[p]);
    if mv < 0 {
        if S.strike_buf[p] == 0 {
            log(b"envoy: strike refused, rc\0", mv);
        }
        return;
    }
    S.strike_buf[p] = 0;
    HH.heat[p] -= ASSIST_COST * HEAT_TICK;
    S.strikes += 1;
    ev(EV_ASSIST, p as i32, mv);
    log(b"envoy: assist strike, player*100 + move\0", p as i32 * 100 + mv);
}

/// Handoff: a tag inside the window after dealing damage fires the newcomer's first trigger at once (a normal fire at
/// the player's Heat state, link 1 of a chain from it); with Relay in the row, the whole row (a volley). No Heat cost.
unsafe fn handoff(p: usize) {
    let set = in_set(p);
    let n = open_slots();
    let relay = (0..n).any(|k| S.drives[set][k] > 0 && POOL[S.drives[set][k] as usize - 1].effect == E_RELAY);
    let Some(k) = (0..n).find(|&k| is_trigger(set, k) && POOL[S.drives[set][k] as usize - 1].effect != E_RELAY) else {
        return;
    };
    if relay {
        start_chain(set, 0, true);
    } else {
        HH.chg[set][k] = HH.chg[set][k].max(need(&POOL[S.drives[set][k] as usize - 1]));
        start_chain(set, k, false);
    }
    ev(EV_HANDOFF, p as i32, S.drives[set][k] | (relay as i32) << 8);
    log_drive(b"handoff", S.drives[set][k] as usize - 1, p as i32 * 10 + relay as i32);
}

unsafe fn tag_frame() {
    if S.tag_btn != 2 {
        quick_tag_frame();
        return;
    }
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
            S.tag_cd[p] = tag_cooldown(p);
            S.tags += 1;
            log(b"envoy: tag, player*100 + count\0", p as i32 * 100 + S.tags);
            return; // one transition at a time
        }
    }
}

/// Foe stocks left in play right after slot `s` went down to `n`.
#[allow(dead_code)]
unsafe fn foes_left_after(s: i32, n: i32) -> i32 {
    (0..4).filter(|&t| is_foe(t)).map(|t| if t == s { n.max(0) } else { gd_player_stocks(t).max(0) }).sum()
}

/// Stage switches are events (decide-6 A): a request (stage_request) fires here once the fight is in live play
/// (no freeze: not a tag, a pick, the stop or another switch), past its first STAGE_GRACE frames and STAGE_GAP after
/// the last switch. A request waits for those; the fight's end (the collect or an outcome) drops it.
/// The hook for enemy waves: `stage_request(SH_WAVE)` when a wave's last foe falls and the next one arrives.
unsafe fn stage_frame() {
    if gd_transition_phase() != 0 {
        return;
    }
    S.stage_clock = S.stage_clock.saturating_add(1);
    live_frame();
    if S.stage_switches > 0 && S.lv_on != 1 && (0..=SWITCH_KO_WINDOW).contains(&S.stage_clock) {
        switch_watch();
    }
    if S.test_torture != 0 && S.stage_want == 0 && S.play >= TORTURE_GAP && S.stage_clock >= TORTURE_GAP {
        S.stage_want = SH_WAVE;
    }
    if S.stage_want == 0 {
        return;
    }
    let cap = if S.test_torture != 0 { i32::MAX } else if shifting() && S.stage_want == SH_WAVE { WAVES_MAX } else { STAGE_MAX };
    if S.outcome != 0 || S.collect != 0 || S.stage_off != 0 || S.stage_switches >= cap {
        S.stage_want = 0;
        return;
    }
    if S.reward_due != 0 || S.rw_active != 0 || S.test_torture == 0 && S.play < STAGE_GRACE || S.stage_want != SH_WAVE && S.stage_switches > 0 && S.stage_clock < STAGE_GAP {
        return;
    }
    let cur = gd_stage_grkind();
    // the torture knob walks every ordered stage pair (a step of 1, 2, 3, 4 along the list, five switches each)
    let step = if S.test_torture != 0 { 1 + (S.stage_switches / STAGES.len() as i32) as usize % (STAGES.len() - 1) } else { 1 };
    let next = match STAGES.iter().position(|&k| k == cur) {
        Some(i) => STAGES[(i + step) % STAGES.len()],
        None => STAGES[0],
    };
    let live = !test_flag("freezeshift");
    if !live {
        switch_snapshot(b"torture at switch\0");
    }
    let rc = if live { gd_stage_transition_live(next, LIVE_LEN, LIVE_AT) } else { gd_stage_transition(next, STAGE_LEN) };
    if rc == -9 {
        return; // busy: try again next frame
    }
    let why = S.stage_want;
    S.stage_want = 0;
    if rc != 0 {
        log(b"envoy: stage transition refused ", rc);
        return;
    }
    S.stage_clock = 0;
    if live {
        // the hand-off is LIVE_AT frames on: the switch's KO window counts from there (live_frame resets the clock)
        S.lv_on = 1;
        log(b"envoy: live stage transition begins, to ", next);
    }
    S.stage_switches += 1;
    ev(EV_SHIFT, why, next);
    log(b"envoy: stage transition, reason*100 + to ", why * 100 + next);
    for p in 0..nh() as usize {
        charge_event(p, C_SHIFT, -1, false);
    }
    // the Shifting road pays each switch (to SHIFT_CAP a fight), into the one purse
    if shifting() && S.shift_paid < SHIFT_CAP {
        S.shift_paid += 1;
        HH.purse += SHIFT_COINS as i64 * COIN_UNIT;
        for p in 0..nh() as usize {
            HH.earned[p] += SHIFT_COINS as i64 * COIN_UNIT / nh() as i64;
            ev(EV_COINS, p as i32, SHIFT_COINS);
        }
        log(b"envoy: shifting road pays, coins ", SHIFT_COINS);
    }
}

/// The live stage transition (Fights 2): the hand-off frame starts the switch's watch window (every fighter's place
/// is recorded then, as at a freeze switch); a refused hand-off is logged; the rift's end clears the state.
unsafe fn live_frame() {
    if S.lv_on == 0 {
        return;
    }
    let mut l = [0i32; 8];
    gd_stage_live(l.as_mut_ptr());
    if S.lv_on == 1 && l[1] == 1 {
        S.lv_on = 2;
        S.stage_clock = 0;
        switch_snapshot(b"torture at switch\0");
        log(b"envoy: live stage hand-off, frame*1000 + to ", l[4] * 1000 + l[3]);
    } else if S.lv_on == 1 && l[1] == 2 {
        S.lv_on = 0;
        log(b"envoy: live stage hand-off refused, to ", l[3]);
    }
    if l[0] == 0 && S.lv_on == 2 && S.stage_clock > SWITCH_KO_WINDOW {
        S.lv_on = 0;
    }
}

/// `envoy: <what> s<slot> <field>` (value: as given): the switch-safety check's per-fighter lines.
fn log_slot(what: &[u8], slot: i32, field: &[u8], v: i32) {
    let mut b = [0u8; 80];
    let mut n = 0;
    let digit = [b's', b'0' + slot as u8, b' '];
    for &x in b"envoy: ".iter().chain(what.iter().filter(|&&c| c != 0)).chain(b" ".iter()).chain(digit.iter()).chain(field.iter()) {
        if n < b.len() - 1 {
            b[n] = x;
            n += 1;
        }
    }
    unsafe { gd_log(b.as_ptr(), v) }
}

/// Every fighter in play: motion, x and y (x10). At a switch it also records the motions for switch_watch.
unsafe fn switch_snapshot(what: &[u8]) {
    let at_switch = what.starts_with(b"torture at");
    for s in 0..4 {
        let h = if gd_player_present(s) != 0 { gd_fighter_from_slot(s) } else { 0 };
        if at_switch {
            S.sw_as[s as usize] = if h != 0 { gd_fighter_get_action_state(h) + 1 } else { 0 };
            S.sw_ko[s as usize] = 0;
            S.sw_hit[s as usize] = 0;
        }
        if h == 0 || S.test_torture == 0 {
            continue;
        }
        log_slot(what, s, b"motion", gd_fighter_get_action_state(h));
        log_slot(what, s, b"x10", (gd_fighter_get_pos_x(h) * 10.0) as i32);
        log_slot(what, s, b"y10", (gd_fighter_get_pos_y(h) * 10.0) as i32);
    }
}

/// The frames after a switch: a fighter that was not already in a KO at the switch entering one (motion 0-10) is a
/// KO the switch caused; the torture knob also logs everyone's place at SWITCH_CHECK.
unsafe fn switch_watch() {
    for s in 0..4 {
        let i = s as usize;
        let h = if gd_player_present(s) != 0 { gd_fighter_from_slot(s) } else { 0 };
        if h == 0 || S.sw_ko[i] != 0 {
            continue;
        }
        let m = gd_fighter_get_action_state(h);
        let was = S.sw_as[i] - 1;
        // hitstun, tumble, grabbed / thrown: a KO after that is the fight's, not the switch's
        if (75..=91).contains(&m) || (156..=157).contains(&m) || (223..=251).contains(&m) {
            S.sw_hit[i] = 1;
        }
        if (0..=10).contains(&m) && was > 13 {
            S.sw_ko[i] = 1;
            if S.sw_hit[i] != 0 {
                log(b"envoy: combat KO after a stage switch (hit first), slot*10000 + frames since\0", s * 10000 + S.stage_clock);
            } else {
                log(b"envoy: KO right after a stage switch, slot*10000 + frames since\0", s * 10000 + S.stage_clock);
            }
        }
    }
    if S.stage_clock == SWITCH_CHECK {
        switch_snapshot(b"torture after switch\0");
    }
}

unsafe fn outcome_frame() {
    if S.outcome != 0 {
        return;
    }
    // the fight was left from the pause (the retail No Contest): not a loss. The run stays live at this fight, which
    // starts over with the pool it began with (the same as closing the game mid-fight and continuing)
    if S.collect == 0 && gd_envoy_quit() != 0 {
        S.outcome = OUT_LEFT;
        log(b"envoy: fight left from the pause, run kept at fight ", S.fight + 1);
        make_plan();
        return;
    }
    let me = (0..nh()).map(|p| gd_player_stocks(p)).min().unwrap_or(0);
    let mut foes = 0;
    for s in 0..4 {
        if is_foe(s) {
            foes += gd_player_stocks(s).max(0);
        }
    }
    if test_waves() >= 0 {
        // an enemy-wave fight: the enemies left (all of them before they spawn)
        foes = if S.en_spawned == 0 { EN_TEST as i32 } else { S.en_h.iter().filter(|&&h| h > 0).count() as i32 };
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
        // items: the item room / the boss drop: pedestals on the floor of the held match; the owners walk in
        if S.ped_done == 0 && S.outcome == 0 {
            S.ped_done = 1;
            let k = items::ped_due();
            if k != 0 {
                items::ped_open(k);
            }
        }
        if items::ped_wait() {
            return;
        }
        // the fly-in: every drop still on the floor flies to its owner's fighter in play, then is worn (the
        // match end is held for every fight: nothing a fight dropped is lost, nobody chases a drop)
        if drops_on_floor() && S.collect <= FLY_TIME {
            fly_frame();
            S.collect += 1;
            return;
        }
        if S.outcome == 0 {
            let last = session_last(S.fight);
            fight_won();
            if last && S.state == RUN_LIVE {
                begin_stop();
                if S.stop != 0 {
                    return;
                }
            }
            gd_envoy_end_match();
        }
    } else if me <= 0 {
        S.outcome = 2;
        S.pool = 0;
        S.state = RUN_OVER;
        log(b"envoy: RUN OVER at fight\0", S.fight + 1);
        make_plan();
    }
}

// ---- enemy waves (first step; test knob only, envoy-v3-rules.md "Enemy waves")

const EN_MAX: usize = 6;
const EN_TEST: usize = 3;
/// Damage dealt to an enemy goes into Envoy's records against this match slot (empty in a wave fight).
const EN_SLOT: i32 = 3;

/// MELEE_ENVOY_TEST_DRIVES item `~K`: an enemy-wave test fight with three of script kind K (0 Goomba, 2 ReDead,
/// 4 Octorok); -1 none. Fixed per process.
unsafe fn test_waves() -> i32 {
    static mut CACHE: i32 = -2;
    if CACHE != -2 {
        return CACHE;
    }
    CACHE = test_waves_read();
    CACHE
}

unsafe fn test_waves_read() -> i32 {
    let mut b = [0u8; 256];
    let n = gd_envoy_test_drives(b.as_mut_ptr(), b.len() as u32);
    if n <= 0 {
        return -1;
    }
    let s = core::str::from_utf8(&b[..(n as usize).min(b.len())]).unwrap_or("");
    s.split(|c| c == ',' || c == '/' || c == '|' || c == ';')
        .find_map(|t| t.trim().strip_prefix('~').and_then(|x| x.parse::<i32>().ok()))
        .unwrap_or(-1)
        .clamp(-1, 6)
}

/// Per live frame of a wave fight: spawn the three once the fight is on, then read each enemy's damage taken (its
/// last attacker gets the record: Heat, charge) and its death.
/// Waves in room `fight` (kind `kind`) in loop index `e` (0 the first loop): the boss one; room 1 one short wave
/// (two with the test knob `wave2`); the others two; every loop past the first adds one (to WAVES_MAX).
unsafe fn room_waves(fight: usize, kind: i32, e: i32) -> i32 {
    if kind == FK_BOSS {
        1
    } else if old_scale() {
        if fight == 0 && S.wave2 == 0 && !test_flag("wave2") { 1 + e.clamp(0, 1) } else { (2 + e.max(0)).min(WAVES_MAX) }
    } else {
        let w = ROOM_MIX[fight.min(FIGHTS - 1)].waves[e.clamp(0, 2) as usize];
        let w = if S.wave2 != 0 || test_flag("wave2") { w.max(2) } else { w };
        w.clamp(1, GAUNT_MAX)
    }
}

/// Foes on the field at once in fight `fight` of loop index `e` (ROOM_MIX; co-op one more in a duel or a pack room).
fn room_foes(fight: usize, e: i32, coop: bool) -> i32 {
    let f = &SCHEDULE[fight.min(FIGHTS - 1)];
    if f.kind == FK_BOSS {
        return 1;
    }
    let base = ROOM_MIX[fight.min(FIGHTS - 1)].foes[e.clamp(0, 2) as usize];
    let extra = (coop && (f.kind == FK_DUEL || f.kind == FK_PACK)) as i32;
    (base + extra).clamp(1, if coop { 2 } else { 3 })
}

/// The crowd knockback (x1000) for a fight with `foes` foes (1000: none).
fn crowd_kb(foes: i32) -> i32 {
    if foes < CROWD_FROM { 1000 } else { (CROWD_KB + CROWD_KB_STEP * (foes - CROWD_FROM)).min(CROWD_KB_CAP) }
}

/// A foe character for a room: from `h`'s place in the cast, never one of `used` (this room's faces so far),
/// preferring one not among the run's recent faces (S.seen).
unsafe fn pick_face(h: u32, used: &[i32]) -> i32 {
    let n = CPU_POOL.len();
    let start = (h % n as u32) as usize;
    for pass in 0..2 {
        for k in 0..n {
            let c = CPU_POOL[(start + k * 7) % n];
            if used.contains(&c) || pass == 0 && S.seen.contains(&c) {
                continue;
            }
            return c;
        }
    }
    CPU_POOL[start]
}

/// The wave foe row 0's elite arrives in (an elite room's last wave; 0 none).
unsafe fn elite_wave() -> i32 {
    if sched().kind == FK_ELITE && S.waves >= 2 { S.waves } else { 0 }
}

/// Rooms: when every foe of the wave has fallen (all parked) and each parked foe's KO countdown has ended, the next
/// wave comes back on the same slots as new characters (EV_WAVE; a stage switch request, SH_WAVE).
unsafe fn room_frame() {
    if S.park == 0 {
        return;
    }
    let alive = (0..4).any(|s| is_foe(s) && S.park & (1 << s) == 0 && gd_player_stocks(s) > 0);
    if alive {
        return;
    }
    if S.wave_clr == 0 {
        // the wave is cleared (its last KO cashed out like any KO): coins, and one more drop for each player
        S.wave_clr = 1;
        HH.purse += COINS_WAVE as i64 * COIN_UNIT;
        for p in 0..nh() as usize {
            HH.earned[p] += COINS_WAVE as i64 * COIN_UNIT / nh() as i64;
            ev(EV_COINS, p as i32, COINS_WAVE);
            if !old_scale() {
                // rule 4: a cleared wave heals (never below 0%)
                let h = gd_fighter_from_slot(p as i32);
                if h != 0 {
                    let mut w = [0u32; 20];
                    gd_fighter_read(h, w.as_mut_ptr());
                    let heal = WAVE_HEAL.min(((f32::from_bits(w[10]) * 1000.0) as i32).max(0));
                    if heal > 0 {
                        gd_envoy_percent(p as i32, 0, -heal);
                        log(b"envoy: wave heal x1000\0", heal);
                    }
                }
            }
        }
        log(b"envoy: wave cleared, coins\0", COINS_WAVE);
    }
    if gd_envoy_wave_pending() & S.park != S.park {
        return;
    }
    let n = gd_envoy_wave_start();
    S.park = 0;
    S.wave_clr = 0;
    S.wave += 1;
    ev(EV_WAVE, S.wave, S.waves);
    log(b"envoy: wave starts, wave*100 + foes back\0", S.wave * 100 + n);
    if ncpu() == 1 && !old_scale() {
        let face = if S.wave == 2 { PLAN[32] } else { PLAN[PL_GAUNT + (S.wave - 3).clamp(0, 2) as usize] };
        log(b"envoy: gauntlet duel, wave*100 + character ", S.wave * 100 + face);
    }
    let f = sched();
    if f.kind == FK_ELITE && S.wave == elite_wave() {
        // the elite's drives arrive with it
        let first = nh() as usize;
        foe_roll(first, (f.foe_drives + lx()).clamp(0, 2) as usize, (f.foe_level.max(1) + lx()).clamp(1, lvl_cap()), 8);
        log(b"envoy: the elite arrives, slot\0", first as i32);
    }
    stage_request(SH_WAVE);
}

unsafe fn waves_frame() {
    let k = test_waves();
    if k < 0 {
        return;
    }
    if S.en_spawned == 0 {
        if S.play < 30 {
            return;
        }
        S.en_spawned = 1;
        const XS: [f32; EN_TEST] = [-50.0, 0.0, 50.0];
        for i in 0..EN_TEST {
            let h = (S.seq & 0xFFFF) << 8 | (i as i32 + 1);
            let r = gd_envoy_enemy_spawn(k, XS[i], 30.0, if i % 2 == 0 { 1 } else { -1 }, h);
            S.en_h[i] = if r == h { h } else { 0 };
            log(b"envoy: enemy spawned, handle (-1: refused)\0", r);
        }
        return;
    }
    for i in 0..EN_MAX {
        let h = S.en_h[i];
        if h <= 0 {
            continue;
        }
        let st = gd_envoy_enemy_status(h);
        // the damage first: a one-hit kill is read in the frame it is defeated (the registry keeps it until removed)
        let rcv = gd_envoy_enemy_i(h, 6);
        {
            if rcv >= 0 && rcv != S.en_rcv[i] {
                let dmg = gd_envoy_enemy_i(h, 2).max(0);
                let d = (dmg - S.en_dmg[i]).max(0) * 1000;
                let who = gd_envoy_enemy_i(h, 7) - 1;
                S.en_rcv[i] = rcv;
                S.en_dmg[i] = dmg;
                if (0..nh()).contains(&who) && d > 0 {
                    S.hits += 1;
                    deal(who as usize, EN_SLOT, d, 1000, 0, -1, CM_MOVE, 0, 0);
                    log(b"envoy: enemy hit, player*1000 + damage\0", who * 1000 + d / 1000);
                }
            }
        }
        if st != 1 {
            log(b"envoy: enemy gone, status (2 defeated)\0", st);
            S.en_h[i] = 0;
        }
    }
}

/// Whether fight `f` (0-based) ends its session (the next one opens a session, or the loop ends).
fn session_last(f: i32) -> bool {
    let n = f.clamp(0, FIGHTS as i32 - 1) as usize + 1;
    n >= FIGHTS || SESSION_START[n]
}

/// The stop after a session (the match is held open past the win): the banked picks ("Reward 1 of N"), a keystone if
/// due, the shop; then the match ends (reward_close) and the road shows the next session. `S.fight` is already the
/// next session's first fight. If the freeze cannot start, the stop opens the next fight instead (the old way).
unsafe fn begin_stop() {
    // items: the stop is the shop only (no banked picks, no keystones)
    let due = 0;
    S.shop_due = (S.no_shop == 0) as i32;
    S.stop_done = 1;
    if due == 0 && S.shop_due == 0 {
        return;
    }
    if gd_hold_begin(REWARD_TIME * (S.bank.max(1) + 3) + SHOP_TIME + 120) != 0 {
        S.stop_done = 0;
        S.shop_due = 0;
        return;
    }
    S.stop = 1;
    S.bank_n = if due == RW_DRIVES { S.bank } else { 0 };
    S.bank_i = (S.bank_n > 0) as i32;
    log(b"envoy: the stop, picks banked\0", S.bank);
    let first = if due != 0 {
        due
    } else {
        S.shop_due = 0;
        RW_SHOP
    };
    open_reward(first);
    if (0..nh() as usize).all(|p| S.rw_done[p] != 0) {
        reward_close();
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
    log(b"envoy: fight length, live frames\0", S.play);
    S.outcome = 1;
    S.pool = me;
    // room length (loop*10 + fight, then the live frames it took): loop_run.sh's room-length table
    log(b"envoy: room cleared, loop*10 + fight ", lp() * 10 + S.fight + 1);
    log(b"envoy: room time, live frames ", S.play);
    if sched().kind >= FK_ELITE && S.bosses < 2 && (S.fight as usize) < FIGHTS - 1 {
        S.bosses += 1;
        log(b"envoy: boss beaten, keystone due; bosses ", S.bosses);
    }
    if S.fight as usize == FIGHTS - 1 {
        // endless loops: the run goes on at fight 1 of the next loop with everything kept, one more stock in the pool
        // items: no banked pick, no free stock past the limit; the campfire after the boss refills to the limit
        S.lp = (lp() + 1).min(LOOP_TOP);
        S.fight = 0;
        let _ = POOL_CAP;
        items::campfire();
        log(b"envoy: loop now\0", S.lp);
        log(b"envoy: RUN CLEAR, pool left ", me);
    } else {
        // the road's prize for this fight of its session (the Open road and the elite session: a drive pick)
        // items: a won fight banks nothing (items come from the item room, the shop and the boss)
        let _ = road_prize_at;
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
    v[V_LOOP] = lp();
    let fi = S.fight.clamp(0, FIGHTS as i32 - 1) as usize;
    // the fight won (or every foe out and the drops being collected: the match may end before fight_won runs)
    // and the next fight in the same session
    let won_now = S.collect > 0 && S.outcome == 0 && fi + 1 < FIGHTS && !SESSION_START[fi + 1];
    v[V_SESS_NEXT] = (S.state == RUN_LIVE && ((S.outcome == 1 && !SESSION_START[fi]) || won_now)) as i32;
    v[V_BANK] = S.bank;
    v[V_BANK_N] = if S.rw_active == RW_DRIVES { S.bank_n } else { 0 };
    v[V_BANK_I] = S.bank_i;
    v[V_BANK_Q] = S.bank_q;
    for p in 0..nh() as usize {
        let b = V_P0 + p * V_PW;
        v[b + P_PINS] = S.pins[set_of(p, 0)] | if S.shared == 0 { S.pins[set_of(p, 1)] << 8 } else { 0 };
        for f in 0..2 {
            for k in 0..SLOTS {
                v[b + P_SCRAP_PRICE + f * SLOTS + k] = if f == 1 && S.shared != 0 { 0 } else { scrap_price(set_of(p, f), k) };
            }
        }
        v[b + P_CONFIRM] = S.confirm[p];
        v[b + P_CONFIRM_CUR] = S.confirm_cur[p];
        let (cf, ck) = ((S.confirm_at[p] >> 3) as usize & 1, (S.confirm_at[p] & 7) as usize % SLOTS);
        v[b + P_CONFIRM_OLD] = if S.confirm[p] != 0 { S.drives[set_of(p, cf)][ck] } else { 0 };
        v[b + P_CONFIRM_NEW] = if S.confirm[p] != 0 { S.confirm_new[p] } else { 0 };
        v[b + P_CONFIRM_AT] = S.confirm_at[p];
        v[b + P_REORDER_ROW] = S.shop_fighter[p];
        v[b + P_REORDER_LIFT] = (S.shop_step[p] == 3) as i32;
        v[b + P_DECLINE] = S.decline[p];
    }
    v[V_BANK_KIND] = if S.rw_active == RW_DRIVES && S.bank_n > 0 { bank_head() } else { 0 };
    let fork_at = if S.state == RUN_LIVE { S.fight } else { gd_envoy_start_fight() };
    let nr = if S.arm_seen != 0 && S.live == 0 && S.state != RUN_OVER { roads_offered_at(fork_at) } else { 0 };
    v[V_ROAD_N] = nr;
    v[V_ROAD_CUR] = S.road_cur.clamp(0, (nr - 1).max(0));
    v[V_ROAD_FLAG] = S.road_flag;
    v[V_ROAD_TAKEN] = S.road;
    for r in 0..nr.min(ROADS_MAX as i32) {
        let b = V_ROAD0 + r as usize * V_ROADW;
        let kind = road_kind(r);
        v[b + RD_KIND] = kind;
        v[b + RD_DANGER] = match kind {
            ROAD_STEEP => DANGER_STEEP,
            ROAD_SHIFT => DANGER_ROUGH,
            _ => DANGER_CALM,
        };
        if kind == ROAD_SHIFT {
            v[b + RD_SHIFT_COINS] = SHIFT_COINS;
            v[b + RD_SHIFT_CAP] = SHIFT_CAP;
            v[b + RD_SHIFT_DMG] = 0;
        }
        v[b + RD_NF] = 3;
        let tw = road_twists(kind);
        v[b + RD_TWISTS] = tw;
        let mut prize = 0;
        let mut fd = 0;
        let e = lp() - 1;
        for i in 0..3usize {
            prize |= road_prize_at(kind, i as i32) << (4 * i);
            let f = &SCHEDULE[i];
            let fb = b + RD_FIGHTS + i * RD_FW;
            v[fb] = f.kind + 1;
            v[fb + 1] = CPU_POOL[(mix(S.seed as u32 ^ ((i as u32 + 8 * e as u32) << 8)) % CPU_POOL.len() as u32) as usize];
            v[fb + 2] = STAGES[i];
            v[fb + 3] = f.ncpu;
            let base = if f.kind >= FK_ELITE { f.foe_drives + e } else { (e >= 1) as i32 };
            fd |= (base + (tw & TW_FOE_DRIVE != 0) as i32).clamp(0, 2) << (4 * i);
        }
        v[b + RD_PRIZE] = prize;
        v[b + RD_PRIZE_N] = 3;
        v[b + RD_FOE_DRIVES] = fd;
    }
    let mut kinds = 0;
    let mut k = fi;
    let mut sh = 0;
    loop {
        kinds |= (SCHEDULE[k].kind + 1) << sh;
        sh += 4;
        k += 1;
        if k >= FIGHTS || SESSION_START[k] {
            break;
        }
    }
    v[V_SESS_KINDS] = kinds;
    let mut s0 = fi;
    while !SESSION_START[s0] {
        s0 -= 1;
    }
    v[V_SESS_FIGHT] = (fi - s0 + 1) as i32;
    v[V_STOP_DONE] = (S.state == RUN_LIVE && S.outcome == 1 && S.stop_done != 0 && S.stop == 0) as i32;
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
    v[V_STAGE_IN] = -1;
    v[V_WAVE] = S.wave.max(1);
    v[V_WAVES] = S.waves.max(1);
    v[V_FOES] = if test_waves() >= 0 { 0 } else { ncpu() };
    v[V_CROWD] = S.crowd;
    v[V_ROOM] = if sched().kind == FK_BOSS { 0 } else { S.fight.clamp(0, FIGHTS as i32 - 1) + 1 };
    v[V_ROOMS] = FIGHTS as i32 - 1;
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
        v[b + P_BREAK_CD] = S.cb_cd[p];
        v[b + P_BREAK_MAX] = CB_COOLDOWN;
        v[b + P_TAG_MAX] = tag_cooldown(p);
        v[b + P_TAG_BUF] = S.tag_buf[p];
        v[b + P_TAG_WINDOW] = S.dmg_win[p];
        v[b + P_ASSIST] = S.assist_seen[p];
        v[b + P_TAG_HOLD] = S.hold_t[p];
        v[b + P_ASSIST_COST] = ASSIST_COST as i32;
        v[b + P_TAG_HOLD_AT] = QT_HOLD;
        v[b + P_TAG_BUTTON] = S.tag_btn;
        v[b + P_HANDOFF_ON] = (S.handoff_off == 0) as i32;
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
        v[b + P_GAIN_FROM..b + P_GAIN_FROM + 3].copy_from_slice(&S.gain_from[p]);
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
        v[b + P_SHOP_STOCK_PRICE] = stock_price();
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
    items::publish(v);
    v[V_OPEN] = 0; // items: no drive slots
    gd_envoy_publish(v.as_ptr(), VIEW_WORDS as u32);
}

/// The pause overlay's Abandon run (envoy_ui pause.rs): the UI holds ABANDON_BIT on the player's pad for a few logic
/// inputs (`gd_ui_sim_signal`), so the choice is synced input and every peer and every resimulated frame sees it
/// at the same frame. The run ends as a lost run does (RUN OVER, the saved run dropped) and the match ends now.
const ABANDON_BIT: u32 = 0x2000;
unsafe fn abandon_check() {
    for p in 0..nh() {
        if gd_pad_held(p) & ABANDON_BIT != 0 {
            S.outcome = 2;
            S.pool = 0;
            S.state = RUN_OVER;
            log(b"envoy: run abandoned from the pause overlay, player ", p + 1);
            make_plan();
            gd_envoy_end_match();
            return;
        }
    }
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
            abandon_check();
        }
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
                if gd_hold_begin(REWARD_TIME * (S.bank.max(1) + 3) + SHOP_TIME + 120) == 0 {
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
                    kb_frame();
                    tech_frame();
                    records_frame();
                    combo_frame();
                    moments_frame();
                    items::frame(EVB);
                    room_frame();
                    waves_frame();
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
            if S.arm_seen != 0 && S.state != RUN_OVER {
                road_frame();
            }
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
        S.lim = items::LIM_START[0];
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
