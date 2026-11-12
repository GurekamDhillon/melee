//! Envoy run support (rust/mods/envoy_sim, envoy_ui): the native half of a single-player Envoy run.
//!
//! Everything here is inert until the Envoy sim mod publishes a plan (`gd_envoy_plan`) and the run is armed
//! (`ARMED`, from the Envoy camp); with no mod loaded PLAN[0] stays 0 and `match_begin` returns at once, so the goldens
//! and replays are unchanged, and with the mods loaded but no run armed VS is retail.
//!
//! * plan: the next VS match the run wants (stage, the player's tag pair, stocks and ratios, the CPU line-up). The
//!   mod keeps it current; `match_begin` (top of `fn_8016E730`, before the players and the stage are created)
//!   rewrites the `StartMeleeData` from it. A plan main / partner of -1 takes the CSS pick of P1 / P2: the run's
//!   pair is picked on the retail CSS.
//! * view: words the sim mod publishes for its presentation mod (`gd_ui_envoy`, read-only).
//! * live upgrades: `apply` (attack / defense ratio, stocks, attribute multipliers on both tag fighters).
//!
//! All statics: the snapshot covers them, so a rollback restores them with the game.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_fighter::tu::player as pl;
use melee_types::*;

pub const PLAN_WORDS: usize = 32;
pub const VIEW_WORDS: usize = 768;

// plan layout (i32 words)
pub const P_ACTIVE: usize = 0;
pub const P_STAGE: usize = 1;
pub const P_MAIN: usize = 2;
pub const P_PARTNER: usize = 3;
pub const P_STOCKS: usize = 4;
pub const P_ATK: usize = 5;
pub const P_DEF: usize = 6;
pub const P_NCPU: usize = 7;
pub const P_CPU_CK: usize = 8;
pub const P_CPU_LV: usize = 11;
pub const P_CPU_STOCKS: usize = 14;
pub const P_CPU_ATK: usize = 17;
pub const P_CPU_DEF: usize = 20;
pub const P_CPU_SCALE: usize = 23;
/// 1: this fight is the run's boss (the Petey boss replaces the stand-in when `MELEE_ENVOY_BOSS=petey`).
pub const P_BOSS: usize = 26;
/// 1: local co-op (envoy-core-v2.md V2): slot 1 is a second human with its own tag pair, the CPUs take slots 2-3.
pub const P_COOP: usize = 27;
/// Co-op: the second player's main / partner (-1: the CSS pick of P2 / P4).
pub const P_MAIN2: usize = 28;
pub const P_PARTNER2: usize = 29;
/// Envoy endless loops: the Petey cages' HP x1000 (0: retail's; envoy_sim raises it 25% a loop).
pub const P_BOSS_HP: usize = 30;
/// Final Destination (Gr_Kind): the Petey arena for now (an original stage is later work).
const GR_FD: i32 = 37;

pub static mut PLAN: [i32; PLAN_WORDS] = [0; PLAN_WORDS];
pub static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];
/// V3 (envoy-synergy-v3.md 12.5): per match slot, event bits the engine sets during the frame at the exact site
/// (bit 0: the fighter L-cancelled an aerial landing), read and cleared by `gd_envoy_events`. Set only while a run
/// is armed, so retail play never writes it.
pub static mut EVENTS: [u32; 4] = [0; 4];
pub const EVENT_LCANCEL: u32 = 1;

/// The L-cancel site (`ftCo_LandingAir_EnterWithLag`, the branch where `x67F < xE4` sets the skill mode): installed
/// in `melee_fighter::hooks::ENVOY_LCANCEL`. A sub-fighter (the second Ice Climber) is ignored.
pub fn on_lcancel(f: &Fighter) {
    if armed() == 0 || f.is_sub_fighter() != 0 {
        return;
    }
    let s = f.player_idx as usize;
    if s < 4 {
        unsafe { EVENTS[s] |= EVENT_LCANCEL };
    }
}

/// Read and clear `slot`'s event bits.
pub unsafe fn events(slot: i32) -> u32 {
    if !(0..4).contains(&slot) {
        return 0;
    }
    let e = EVENTS[slot as usize];
    EVENTS[slot as usize] = 0;
    e
}

/// Grand Finale (V3 12.5): multiply the knockback of the hit `slot`'s fighter in play is taking now (called from the
/// take-damage hook, where `kb_applied` is already set and `ftCo_Damage_CalcKnockback` reads it next).
pub unsafe fn kb_scale(slot: i32, mul_milli: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let m = mul_milli.clamp(1000, 1500) as f32 / 1000.0;
    (*f).dmg.kb_applied *= m;
    0
}

/// Matches rewritten so far (the mod's "a new fight began" edge).
pub static mut SEQ: i32 = 0;
/// The pair the last rewrite used: main, partner (character kinds).
pub static mut ROSTER: [i32; 2] = [-1, -1];
/// Co-op: the second player's pair (main, partner); -1 solo.
pub static mut ROSTER2: [i32; 2] = [-1, -1];

// ---- the door (envoy-redesign.md 1.2-1.3): menu-time state, never match state
/// The run is armed: only then does `match_begin` rewrite a VS match (and the sim mod publish a plan). Set by the
/// camp's Continue / New run, cleared by Abandon, by the end of the run and by entering the main menu.
pub static mut ARMED: i32 = 0;

/// Whether the run is armed: the camp's flag, or the headless audit knob `MELEE_ENVOY_ARM=1` (a scene / `--match` start has
/// no menus to arm it from).
pub fn armed() -> i32 {
    static KNOB: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if unsafe { ARMED } != 0 || *KNOB.get_or_init(|| std::env::var_os("MELEE_ENVOY_ARM").is_some()) { 1 } else { 0 }
}
/// A rewritten fight has started since the run was armed: the camp replaces the VS CSS (the results land in it).
pub static mut CAMP_DUE: i32 = 0;
/// The 1-P menu's extra Envoy row exists (a presentation mod registered the camp scene with `gd_ui_envoy_door`).
pub static mut DOOR_ON: i32 = 0;
#[allow(non_snake_case)]
pub fn DOOR_ON_now() -> bool {
    unsafe { DOOR_ON != 0 }
}
/// The 1-P menu cursor is on the Envoy row (below the retail rows).
pub static mut DOOR_HOVER: i32 = 0;
/// Envoy Quick Tag: the tag button (the camp's switch, `gd_ui_envoy_set_tag_button`; MELEE_ENVOY_TAG_BUTTON=y|x|dpad
/// sets the start value): Y (default), X, or D-pad down with the classic cinematic tag.
pub const TAG_BTN_Y: i32 = 0;
pub const TAG_BTN_X: i32 = 1;
pub const TAG_BTN_CLASSIC: i32 = 2;
pub static mut TAG_BUTTON: i32 = -1;

pub fn tag_button() -> i32 {
    unsafe {
        if TAG_BUTTON < 0 {
            TAG_BUTTON = match std::env::var("MELEE_ENVOY_TAG_BUTTON").as_deref() {
                Ok("x") | Ok("X") => TAG_BTN_X,
                Ok("dpad") | Ok("classic") => TAG_BTN_CLASSIC,
                _ => TAG_BTN_Y,
            };
        }
        TAG_BUTTON
    }
}

/// The Envoy row (or the browser's Envoy mode) was chosen: the host opens the camp scene at the next present.
pub static mut DOOR_OPEN: i32 = 0;
/// The camp asked for the VS mode (its Continue / New run on the main menu): the menu scene starts it next frame.
pub static mut LAUNCH_VS: i32 = 0;
/// Envoy V3: player 2 joined at the camp (envoy_ui, `gd_ui_envoy_coop`): a new run starts in co-op (`gd_envoy_coop`).
pub static mut COOP_JOIN: i32 = 0;

/// The run record (envoy_sim publishes it in VIEW[REC_AT..]; the host keeps the last one on disk).
pub const REC_WORDS: usize = 48;
pub const REC_AT: usize = 704;
pub const REC_MAGIC: i32 = 0x454E_5652; // "ENVR"
pub const REC_VERSION: i32 = 8; // 8: the forked road, bank tokens, pins; 7: sessions (the bank in R_LOOP); 6: endless loops; 5 (V3) still read; older dropped
/// The record the host holds (loaded from disk, updated from the view); `SAVED_LEN` 0 = no run saved.
pub static mut SAVED: [i32; REC_WORDS] = [0; REC_WORDS];
pub static mut SAVED_LEN: i32 = 0;
/// The camp asked to forget the saved run (New run / Abandon): the host deletes the file.
pub static mut SAVED_DROP: i32 = 0;

/// The record's digest word (the last one): FNV-1a over the others. A stale or edited file fails it.
pub fn rec_digest(r: &[i32]) -> i32 {
    let mut h: u32 = 0x811C_9DC5;
    for &w in &r[..REC_WORDS - 1] {
        for b in w.to_le_bytes() {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h as i32
}

pub fn rec_valid(r: &[i32]) -> bool {
    r.len() >= REC_WORDS && r[0] == REC_MAGIC && (5..=REC_VERSION).contains(&r[1]) && r[REC_WORDS - 1] == rec_digest(r)
}

/// Envoy endless loops: the furthest loop any run on this save reached (host file `best.bin`, envoy_store; the camp
/// shows it). Menu-time state, never read by the sim.
pub static mut BEST_LOOP: i32 = 0;

/// Sessions (v3g): the armed run's fight was won and the next fight is in the same session (envoy_data view.rs
/// `V_SESS_NEXT` 760): the VS mode skips the results screen, straight back to the character select the camp replaces.


/// Envoy enemy waves (first step): the script layer's enemy seams (script_game.rs `Script_EnemyHit` / `_Defeated` /
/// `_Removed`, no-ops before). Logged only: envoy_sim polls the registry (`gd_envoy_enemy_i`), which is game state,
/// so nothing here has to be rolled back.
pub unsafe extern "C" fn enemy_hit(handle: i32, player: i32, dmg: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} hit by player {player} for {dmg}");
    }
}
pub unsafe extern "C" fn enemy_defeated(kind: i32, handle: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} (kind {kind}) defeated");
    }
}
pub unsafe extern "C" fn enemy_removed(kind: i32, handle: i32, reason: i32) {
    if std::env::var_os("MELEE_ENVOY_ENEMY_LOG").is_some() {
        eprintln!("envoy: enemy {handle} (kind {kind}) removed, reason {reason}");
    }
}

pub unsafe fn session_continue() -> bool {
    armed() != 0 && VIEW[0] == 0x454E_5633 && VIEW[1] == 1 && (VIEW[760] == 1 || VIEW[766] == 1)
}

/// The 1-P menu's input with the Envoy row (mnmain `mn_8022D7F4`, after the retail read): what the retail code must
/// do. `at_last`: the retail cursor is on the last unlocked row.
/// The main menu's Envoy row was chosen with the Brawl files missing: presents left of the "needs your Brawl disc"
/// message the mod draws (the camp never loads).
pub static mut DOOR_NO_DISC: i32 = 0;
/// The run ended and the player chose "Back to camp" on the road: 1 = the VS character select backs out to the main
/// menu (as a held B does), 2 = the main menu opens the camp. Never leaves the player on the character select.
pub static mut BACK_TO_CAMP: i32 = 0;
/// Fights 2+: the road's "Start the fight" goes straight to the match with the party the run has and the stage the
/// run chose (the match rewrite sets both): 1 = the VS character select confirms itself as soon as it is ready
/// (as Start would), 2 = confirmed, the stage select is skipped (gmvsmode `onExitCss`). `MELEE_ENVOY_SELECT=1`
/// keeps both screens.
pub static mut SKIP_SELECT: i32 = 0;
pub fn skip_select_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !*V.get_or_init(|| std::env::var("MELEE_ENVOY_SELECT").is_ok_and(|v| v == "1"))
}
/// A on the main menu's Envoy row (mnmain `mn_8022DB10`): open the camp, or refuse when the Brawl files are missing.
pub unsafe fn door_main_open() -> bool {
    if !crate::tu::petey_model::brawl_present() {
        DOOR_NO_DISC = 240;
        return false;
    }
    DOOR_OPEN = 1;
    true
}
pub const DOOR_RETAIL: i32 = 0;
pub const DOOR_EAT: i32 = 1;
pub const DOOR_MOVE: i32 = 2;
pub const DOOR_OPENED: i32 = 3;
pub const DOOR_TO_TOP: i32 = 4;
pub unsafe fn door_1p(confirm: bool, back: bool, up: bool, down: bool, at_last: bool) -> i32 {
    if DOOR_ON == 0 {
        DOOR_HOVER = 0;
        return DOOR_RETAIL;
    }
    if DOOR_HOVER != 0 {
        if confirm {
            DOOR_OPEN = 1;
            return DOOR_OPENED;
        }
        if back {
            DOOR_HOVER = 0;
            return DOOR_RETAIL;
        }
        if up {
            DOOR_HOVER = 0;
            return DOOR_MOVE;
        }
        if down {
            DOOR_HOVER = 0;
            return DOOR_TO_TOP;
        }
        return DOOR_EAT;
    }
    if down && at_last && !confirm && !back {
        DOOR_HOVER = 1;
        return DOOR_MOVE;
    }
    DOOR_RETAIL
}

const CK_ICS: i32 = 0x0E;
const CK_ZELDA: i32 = 0x12;
const CK_SHEIK: i32 = 0x13;
const CK_FOX: i32 = 0x02;
const CK_MARTH: i32 = 0x09;

/// A character that can be one half of a tag pair (a retail fighter that owns no sub-fighter).
pub fn taggable(ck: i32) -> bool {
    (0..=0x19).contains(&ck) && ck != CK_ICS && ck != CK_ZELDA && ck != CK_SHEIK
}

fn ratio(v: i32) -> f32 {
    if v <= 0 { 1.0 } else { v as f32 / 1000.0 }
}

/// Rewrite the VS match about to start from the plan (no-op without one, outside VS, and for a replay).
pub unsafe fn match_begin(d: *mut StartMeleeData) {
    melee_core::frameprof::note(melee_core::frameprof::Kind::Envoy, || melee_core::frameprof::Text::new().s("match setup"));
    crate::tu::petey::disarm();
    // Envoy sessions: the match-end hold is the sim's, per match (gd_envoy_end_hold); never left on for a retail match
    crate::tu::gmvs::gmVs_SetEndHold(0);
    melee_fighter::hooks::ENVOY_TAG_MASK = [0; 4];
    if PLAN[P_ACTIVE] == 0 || armed() == 0 || d.is_null() || crate::tu::gm_1A3F::gm_GetCurrentGameMode() as u32 != GM_VS as u32 {
        return;
    }
    let d = &mut *d;
    crate::tu::tag_swap::reset();
    EVENTS = [0; 4];
    melee_fighter::hooks::ENVOY_LCANCEL = Some(on_lcancel);
    let coop = PLAN[P_COOP] == 1;
    let css = |i: usize| -> i32 { let p = &d.players[i]; if p.slot_type != Gm_PKind_NA as u8 { p.ckind as i32 } else { -1 } };
    // solo: P1's pick is the main, P2's the partner; co-op: P1 + P3 are player 1's pair, P2 + P4 player 2's
    let (css_main, css_partner) = (css(0), if coop { css(2) } else { css(1) });
    let pair = |want_main: i32, want_partner: i32, cm: i32, cp: i32| -> (i32, i32) {
        let mut main = if want_main >= 0 { want_main } else { cm };
        if !taggable(main) {
            main = CK_FOX;
        }
        let mut partner = if want_partner >= 0 { want_partner } else { cp };
        if !taggable(partner) || partner == main {
            partner = if main == CK_MARTH { CK_FOX } else { CK_MARTH };
        }
        (main, partner)
    };
    let (main, partner) = pair(PLAN[P_MAIN], PLAN[P_PARTNER], css_main, css_partner);
    ROSTER = [main, partner];
    pl::TAG_PARTNER_CK = [partner as i8, -1, -1, -1];
    ROSTER2 = [-1, -1];
    let humans = if coop { 2 } else { 1 };
    if coop {
        let (m2, p2) = pair(PLAN[P_MAIN2], PLAN[P_PARTNER2], css(1), css(3));
        ROSTER2 = [m2, p2];
        pl::TAG_PARTNER_CK[1] = p2 as i8;
    }

    for h in 0..humans {
        let me = &mut d.players[h];
        me.ckind = (if h == 0 { main } else { ROSTER2[0] }) as i8;
        me.slot_type = 0;
        // test knob (headless win-path checks): MELEE_ENVOY_TEST_P1CPU=1 makes the players level-9 CPUs
        if std::env::var_os("MELEE_ENVOY_TEST_P1CPU").is_some() {
            me.slot_type = 1;
            me.cpu_level = 9;
            me.cpu_kind = 4;
        }
        me.stocks = PLAN[P_STOCKS].clamp(1, 99) as i8;
        me.team = 0;
        me.attack_ratio = ratio(PLAN[P_ATK]);
        me.defense_ratio = ratio(PLAN[P_DEF]);
        me.model_scale = 1.0;
        if h == 1 {
            me.color = 0;
            me.sub_color = 0;
            me.slot = 2; // port 2 (1-based; getPort): never port 1, or player 2 would mirror player 1's controller
            me.spawn_pos = -1;
            me.handicap = 9;
        }
    }
    // Quick Tag: the tag button no longer jumps for the run's humans (the classic D-pad tag masks nothing)
    let mask = match tag_button() {
        TAG_BTN_Y => 0x0800,
        TAG_BTN_X => 0x0400,
        _ => 0,
    };
    for h in 0..humans {
        melee_fighter::hooks::ENVOY_TAG_MASK[h] = mask;
    }
    // 0: an enemy-wave fight (no foe fighters; the sim spawns the enemies, the match end is held)
    let n = PLAN[P_NCPU].clamp(0, 4 - humans as i32) as usize;
    if n == 0 {
        crate::tu::gmvs::gmVs_SetEndHold(1); // a wave fight: no foe to eliminate; the sim ends it (gd_envoy_end_match)
    }
    for i in 0..(4 - humans) {
        let p = &mut d.players[humans + i];
        if i >= n {
            p.slot_type = Gm_PKind_NA as u8;
            continue;
        }
        p.ckind = PLAN[P_CPU_CK + i].clamp(0, 0x19) as i8;
        p.slot_type = 1;
        p.cpu_kind = 4; // the VS AI (0 is the training dummy)
        p.cpu_level = PLAN[P_CPU_LV + i].clamp(1, 9) as u8;
        p.stocks = PLAN[P_CPU_STOCKS + i].clamp(1, 99) as i8;
        p.color = 0;
        p.sub_color = 0;
        p.team = 1;
        p.slot = (humans + i + 1) as u8; // the CSS's own numbering for a CPU row
        p.spawn_pos = -1;
        p.handicap = 9; // the CSS default (0 is not a neutral handicap)
        p.attack_ratio = ratio(PLAN[P_CPU_ATK + i]);
        p.defense_ratio = ratio(PLAN[P_CPU_DEF + i]);
        p.model_scale = ratio(PLAN[P_CPU_SCALE + i]);
    }
    // 4-player rows past the first four are never used by VS
    let r = &mut d.rules;
    r.is_teams = 1;
    r.set_friendly_fire(0);
    r.set_is_stock(1);
    r.set_match_kind(MatchKind_Stock as u32);
    r.set_timer_enabled(0);
    r.time_limit = 0;
    r.item_freq = -1;
    let petey = PLAN[P_BOSS] == 1 && crate::tu::petey::selected();
    let stage = if petey { GR_FD } else { PLAN[P_STAGE] };
    if stage > 0 && crate::tu::stage_hop::supported(stage) {
        r.stkind = melee_world::tu::stage::Stage_StageSlotSelect(stage) as u16;
    }
    if petey {
        // the one CPU is Petey's carrier: stands still (cpu_kind 0), plain size and ratios, one stock
        let c = &mut d.players[humans];
        c.cpu_kind = 0;
        c.cpu_level = 1;
        c.stocks = 1;
        c.model_scale = 1.0;
        c.attack_ratio = 1.0;
        c.defense_ratio = 1.0;
        // HP scales with the players on the player's side (P3: 1-2)
        crate::tu::petey::arm(humans as i32, humans as i32, 0x5E7E_u32 ^ (SEQ as u32).wrapping_mul(0x9E37_79B9));
        melee_fighter::tu::ftdrawcommon::BOSS_DRAW = Some(crate::tu::petey::draw);
    } else {
        crate::tu::petey::disarm();
    }
    BASE = [[[0.0; 4]; 2]; 4];
    BLASTS = [Blast::OFF; 4];
    SEQ = SEQ.wrapping_add(1);
    CAMP_DUE = 1;
    eprintln!("envoy: match {} rewritten: main {main} partner {partner} co-op {} stocks {} cpus {n} stage {}", SEQ, coop as i32, me_stocks(d), PLAN[P_STAGE]);
}

fn me_stocks(d: &StartMeleeData) -> i32 {
    d.players[0].stocks as i32
}

unsafe fn fighters(slot: i32) -> [*mut Fighter; 2] {
    let p = &*pl::Player_GetPtrForSlot(slot);
    let f = |g: *mut HSD_GObj| if g.is_null() { core::ptr::null_mut() } else { (*g).user_data as *mut Fighter };
    [f(p.player_entity[0]), f(p.player_entity[1])]
}

pub const UP_ATTACK: i32 = 0;
pub const UP_DEFENSE: i32 = 1;
pub const UP_STOCKS: i32 = 2;
pub const UP_WEIGHT: i32 = 3;
pub const UP_SPEED: i32 = 4;
pub const UP_JUMP: i32 = 5;

/// `apply`'s fighter selector (`which | APPLY_MAIN`): the attribute multipliers touch only the main (entity 0) or the
/// partner (entity 1) of the tag pair; without one, both. Ratios and stocks are per slot.
pub const APPLY_MAIN: i32 = 0x100;
pub const APPLY_PARTNER: i32 = 0x200;

/// A live upgrade on `slot` (both tag fighters, or one with `APPLY_MAIN` / `APPLY_PARTNER`). `v` is x1000 for
/// ratios / multipliers, a count for stocks.
pub unsafe fn apply(slot: i32, which: i32, v: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let sel = (which >> 8) & 3;
    let which = which & 0xFF;
    let m = v as f32 / 1000.0;
    match which {
        UP_ATTACK => pl::Player_SetAttackRatio(slot, m),
        UP_DEFENSE => pl::Player_SetDefenseRatio(slot, m),
        UP_STOCKS => pl::Player_SetStocks(slot, v.clamp(0, 99)),
        UP_WEIGHT | UP_SPEED | UP_JUMP => {
            if !(0.25..=4.0).contains(&m) {
                return -2;
            }
            for (i, f) in fighters(slot).into_iter().enumerate() {
                if f.is_null() || (sel != 0 && sel != i as i32 + 1) {
                    continue;
                }
                let a = &mut (*f).co_attrs;
                match which {
                    UP_WEIGHT => a.weight *= m,
                    UP_SPEED => {
                        a.walk_max_vel *= m;
                        a.dash_initial_velocity *= m;
                        a.dash_max_velocity *= m;
                    }
                    _ => a.jump_v_initial_velocity *= m,
                }
            }
        }
        _ => return -2,
    }
    0
}

/// Add `delta_milli` / 1000 percent to `slot`'s active fighter (`bench` 0) or its benched tag partner (`bench` 1),
/// clamped to 0..=999 (a heal is a negative delta). Returns the new percent x1000, or -1 (no such fighter). The
/// bench is the sleeping sub-fighter (its own percent, carried in by the swap), or in the in-place prototype the
/// benched percent the swap restores.
pub unsafe fn percent(slot: i32, bench: i32, delta_milli: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    if bench == 0 && slot == crate::tu::petey::carrier() {
        // the boss's HP is separate from percent: damage(amount) (heals do nothing)
        crate::tu::petey::damage(delta_milli as f32 / 1000.0);
        return 0;
    }
    let add = |p: f32| {
        let v = p + delta_milli as f32 / 1000.0;
        if v < 0.0 { 0.0 } else if v > 999.0 { 999.0 } else { v }
    };
    if bench != 0 && pl::tag_inplace() {
        if pl::TAG_SLOT[slot as usize] == 0 {
            return -1;
        }
        let v = add(pl::TAG_BENCH_PCT[slot as usize]);
        pl::TAG_BENCH_PCT[slot as usize] = v;
        return melee_core::ppc::fctiwz_f32(v * 1000.0);
    }
    let p = &*pl::Player_GetPtrForSlot(slot);
    let g = p.player_entity[p.transformed[(bench != 0) as usize] as usize];
    if g.is_null() || (*g).user_data.is_null() {
        return -1;
    }
    let f = &mut *((*g).user_data as *mut Fighter);
    let v = add(f.dmg.x1830_percent);
    f.dmg.x1830_percent = v;
    pl::Player_SetHPByIndex(slot, f.is_sub_fighter() as i32, melee_core::ppc::fctiwz_f32(v));
    melee_core::ppc::fctiwz_f32(v * 1000.0)
}

// ---- V2 (envoy-core-v2.md 2.4): speed from a stored base, the orb's absorb, hit attribution, air jumps, blasts

/// The walk / dash attributes each (slot, entity) had before Envoy's first speed write, and the character they
/// belong to (+1; 0 none): `speed` writes `base * m`, so nothing drifts (V1 multiplied and divided back) and a
/// respawned or swapped-in fighter (in-place tag, a new stock) is re-based from its own fresh attributes.
pub static mut BASE: [[[f32; 4]; 2]; 4] = [[[0.0; 4]; 2]; 4];

unsafe fn active_fighter(slot: i32) -> *mut Fighter {
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

/// The walk and dash multipliers (x1000) of `slot`'s fighter in play (`active`) and its benched partner (`bench`;
/// ignored when it has no fighter object). Absolute: each call sets `base * m`.
pub unsafe fn speed(slot: i32, active: i32, bench: i32) -> i32 {
    if !(0..4).contains(&slot) || crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let p = &*pl::Player_GetPtrForSlot(slot);
    let act = p.transformed[0] as usize & 1;
    for (e, f) in fighters(slot).into_iter().enumerate() {
        if f.is_null() {
            continue;
        }
        let m = (if e == act { active } else { bench }).clamp(250, 4000) as f32 / 1000.0;
        let a = &mut (*f).co_attrs;
        let b = &mut BASE[slot as usize][e];
        if b[3] != (*f).kind as f32 + 1.0 {
            *b = [a.walk_max_vel, a.dash_initial_velocity, a.dash_max_velocity, (*f).kind as f32 + 1.0];
        }
        a.walk_max_vel = b[0] * m;
        a.dash_initial_velocity = b[1] * m;
        a.dash_max_velocity = b[2] * m;
    }
    0
}

/// Bulwark's orb (called from the take-damage pre hook that skips the percent): the hit's knockback goes too, so
/// the hit leaves no launch and no flinch (`ftCo_8008EC90` takes a zero knockback as no reaction).
pub unsafe fn absorb(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    (*f).dmg.kb_applied = 0.0;
    0
}

/// The slot whose hit `slot`'s fighter in play took last (its damage source: a fighter, or an item's owner), or -1.
pub unsafe fn attacker(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let mut g = (*f).dmg.x1868_source;
    if !g.is_null() && (*g).classifier as i32 == HSD_GOBJ_CLASS_ITEM && !(*g).user_data.is_null() {
        g = (*((*g).user_data as *mut Item)).owner;
    }
    if g.is_null() || (*g).classifier as i32 != HSD_GOBJ_CLASS_FIGHTER || (*g).user_data.is_null() {
        return -1;
    }
    let s = (*((*g).user_data as *mut Fighter)).player_idx as i32;
    if (0..4).contains(&s) { s } else { -1 }
}

/// `slot`'s fighter in play: jumps used | max jumps << 8 (Updraft), or -1.
pub unsafe fn jumps(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    (*f).x1968_jumpsUsed as i32 | ((*f).co_attrs.max_jumps as i32) << 8
}

/// Updraft: give back one used air jump (0 done, -1 none used).
pub unsafe fn air_jump(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() || (*f).x1968_jumpsUsed <= 1 {
        return -1;
    }
    (*f).x1968_jumpsUsed -= 1;
    0
}

/// A drive blast (Nova, Comet, Spire): a hitbox on the owner's fighter in play, on the engine's hit path as Petey's
/// attacks are (`petey::hitboxes`), so it launches with the game's own knockback, hits Petey's body and cages, and
/// its damage goes through the take-damage hook. It lives `BLAST_LIFE` logic frames on a capsule slot the owner's
/// move is not using; teammates are pre-registered as victims, so it never hits a teammate whatever the friendly-fire
/// rule says.
#[derive(Clone, Copy)]
pub struct Blast {
    pub slot: i32,
    pub cap: i32,
    pub left: i32,
    pub x: f32,
    pub y: f32,
    pub hit: HitCapsule,
}
impl Blast {
    pub const OFF: Blast = Blast { slot: -1, cap: -1, left: 0, x: 0.0, y: 0.0, hit: unsafe { core::mem::zeroed() } };
}
pub static mut BLASTS: [Blast; 4] = [Blast::OFF; 4];
static mut BLAST_GRP: u32 = 0;
const BLAST_LIFE: i32 = 3;

unsafe fn blast_offset(f: &Fighter, x: f32, y: f32) -> Option<Vec3> {
    let root = (*f.parts).joint;
    if root.is_null() {
        return None;
    }
    let m = melee_runtime::hsd_mtx::inverse(&(*root).mtx);
    Some(Vec3 { x: m[0][0] * x + m[0][1] * y + m[0][3], y: m[1][0] * x + m[1][1] * y + m[1][3], z: m[2][0] * x + m[2][1] * y + m[2][3] })
}

/// A blast from `slot`'s fighter in play at world (x, y), radius `r`, `dmg` percent, knockback `angle` (361: the
/// Sakurai angle), growth `kbg`, base `bkb`. 0 armed; -1 no fighter / not in gameplay; -2 no free capsule.
pub unsafe fn blast(slot: i32, x: f32, y: f32, r: f32, dmg: i32, angle: i32, kbg: i32, bkb: i32) -> i32 {
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return -1;
    }
    let f = active_fighter(slot);
    if f.is_null() {
        return -1;
    }
    let Some(k) = BLASTS.iter().position(|b| b.left <= 0) else { return -2 };
    let busy = |i: usize| BLASTS.iter().any(|b| b.left > 0 && b.slot == slot && b.cap == i as i32);
    let Some(ci) = (0..4).rev().find(|&i| (*f).x914[i].state == HitCapsule_Disabled && !busy(i)) else { return -2 };
    let Some(off) = blast_offset(&*f, x, y) else { return -1 };
    let h = &mut *core::ptr::addr_of_mut!((*f).x914[ci]);
    *h = core::mem::zeroed();
    BLAST_GRP = BLAST_GRP.wrapping_add(1);
    h.x4 = 0xB1A5_0000 | (BLAST_GRP & 0xFFFF);
    h.state = HitCapsule_Enabled;
    h.jobj = (*(*f).parts).joint;
    h.damage = dmg.clamp(0, 99) as f32;
    h.unk_count = dmg.clamp(0, 99) as u32;
    h.kb_angle = angle;
    h.x24 = kbg.max(0) as u32;
    h.x28 = 0;
    h.x2C = bkb.max(0) as u32;
    h.element = 0;
    h.x34 = 0;
    h.sfx_severity = 1;
    h.set_x40_b2(1);
    h.set_x40_b3(1);
    h.set_x42_b5(1);
    h.set_x42_b7(1);
    h.set_x42_b1(1);
    h.b_offset = off;
    h.scale = r;
    h.set_x43_b1(1);
    // the owner's teammates (both tag entities) are victims already: the blast passes through them
    let team = d_team(slot);
    let mut v = 0;
    for s in 0..4 {
        if s == slot || team < 0 || d_team(s) != team {
            continue;
        }
        for t in fighters(s) {
            if !t.is_null() && v < 12 {
                h.victims_1[v].victim = t as *mut core::ffi::c_void;
                v += 1;
            }
        }
    }
    (*f).set_x2219_b3(1);
    BLASTS[k] = Blast { slot, cap: ci as i32, left: BLAST_LIFE, x, y, hit: *h };
    0
}

/// The team of `slot` in the match (-1: no player there).
unsafe fn d_team(slot: i32) -> i32 {
    let f = active_fighter(slot);
    if f.is_null() { -1 } else { (*f).team as i32 }
}

/// Per logic frame (after the post chain, next to `petey::tick`): keep each blast on its capsule for its life (put
/// back if the owner's own state change cleared it and its move left the slot free), then switch it off.
pub unsafe fn tick() {
    if ARMED != 0 && hud_off() {
        hide_retail_hud();
    }
    for b in BLASTS.iter_mut() {
        if b.left <= 0 {
            continue;
        }
        b.left -= 1;
        let f = active_fighter(b.slot);
        if f.is_null() {
            b.left = 0;
            continue;
        }
        let h = &mut *core::ptr::addr_of_mut!((*f).x914[b.cap as usize & 3]);
        let ours = h.x4 == b.hit.x4;
        if b.left <= 0 {
            if ours {
                h.state = HitCapsule_Disabled;
            }
            continue;
        }
        if ours && h.state != HitCapsule_Disabled {
            if let Some(off) = blast_offset(&*f, b.x, b.y) {
                h.b_offset = off;
            }
            b.hit = *h;
        } else if h.state == HitCapsule_Disabled {
            *h = b.hit;
            if let Some(off) = blast_offset(&*f, b.x, b.y) {
                h.b_offset = off;
            }
        }
    }
}

/// Envoy draws its own percent, stocks and timer (envoy_ui's HUD): while a run is armed the retail damage plates,
/// their digits and stock icons are hidden (JObj flags only: no game state; with no run armed the HUD is retail).
/// `MELEE_ENVOY_RETAIL_HUD=1` keeps the retail HUD.
pub fn hud_off() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    !*V.get_or_init(|| std::env::var("MELEE_ENVOY_RETAIL_HUD").is_ok_and(|v| v == "1"))
}
pub unsafe fn hide_retail_hud() {
    use melee_world::tu::ifstatus as ifs;
    // only in a match (the plates are freed at its end and the pointers are not cleared everywhere)
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return;
    }
    let h = ifs::ifStatus_GetHUDInfo();
    if h.is_null() {
        return;
    }
    for i in 0..6usize {
        let p = &mut (*h).players[i];
        p.flags.set_hide_all_digits(1);
        for g in [p.HUD_parent_entity, p.next] {
            if !g.is_null() && !(*g).hsd_obj.is_null() {
                melee_runtime::tu::jobj_ptr::HSD_JObjSetFlagsAll((*g).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
            }
        }
    }
}

/// Read-only words for the Envoy HUD (gd_ui_fight_hud): per match slot 0-3 [present, percent, stocks]. Outside a
/// match all zero.
pub unsafe fn hud_words() -> [i32; 16] {
    let mut o = [0i32; 16];
    if crate::tu::stage_hop::check(-1) == crate::tu::stage_hop::HOP_NOT_GAMEPLAY {
        return o;
    }
    for s in 0..4i32 {
        if melee_fighter::player::get_entity(s).is_null() {
            continue;
        }
        o[s as usize * 3] = 1;
        o[s as usize * 3 + 1] = melee_fighter::player::get_damage(s);
        o[s as usize * 3 + 2] = melee_fighter::stale::stocks(s);
    }
    // word 12: the retail pause is up (its overlay owns the top of the screen)
    o[12] = (crate::tu::gmscene::gm_GetDbPauseFlag(1) != 0) as i32;
    o
}
