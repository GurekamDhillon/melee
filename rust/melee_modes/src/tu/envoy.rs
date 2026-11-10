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
pub const VIEW_WORDS: usize = 160;

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
/// Final Destination (Gr_Kind): the Petey arena for now (an original stage is later work).
const GR_FD: i32 = 37;

pub static mut PLAN: [i32; PLAN_WORDS] = [0; PLAN_WORDS];
pub static mut VIEW: [i32; VIEW_WORDS] = [0; VIEW_WORDS];
/// Matches rewritten so far (the mod's "a new fight began" edge).
pub static mut SEQ: i32 = 0;
/// The pair the last rewrite used: main, partner (character kinds).
pub static mut ROSTER: [i32; 2] = [-1, -1];

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
/// The 1-P menu cursor is on the Envoy row (below the retail rows).
pub static mut DOOR_HOVER: i32 = 0;
/// The Envoy row (or the browser's Envoy mode) was chosen: the host opens the camp scene at the next present.
pub static mut DOOR_OPEN: i32 = 0;
/// The camp asked for the VS mode (its Continue / New run on the main menu): the menu scene starts it next frame.
pub static mut LAUNCH_VS: i32 = 0;

/// The run record (envoy_sim publishes it in VIEW[REC_AT..]; the host keeps the last one on disk).
pub const REC_WORDS: usize = 32;
pub const REC_AT: usize = 40;
pub const REC_MAGIC: i32 = 0x454E_5652; // "ENVR"
pub const REC_VERSION: i32 = 2; // 2: crests (milestone 1); a v1 record is dropped
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
    r.len() >= REC_WORDS && r[0] == REC_MAGIC && r[1] == REC_VERSION && r[REC_WORDS - 1] == rec_digest(r)
}

/// The 1-P menu's input with the Envoy row (mnmain `mn_8022D7F4`, after the retail read): what the retail code must
/// do. `at_last`: the retail cursor is on the last unlocked row.
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
    crate::tu::petey::disarm();
    if PLAN[P_ACTIVE] == 0 || armed() == 0 || d.is_null() || crate::tu::gm_1A3F::gm_GetCurrentGameMode() as u32 != GM_VS as u32 {
        return;
    }
    let d = &mut *d;
    crate::tu::tag_swap::reset();
    let css_main = d.players[0].ckind as i32;
    let p2 = &d.players[1];
    let css_partner = if p2.slot_type != Gm_PKind_NA as u8 { p2.ckind as i32 } else { -1 };
    let mut main = if PLAN[P_MAIN] >= 0 { PLAN[P_MAIN] } else { css_main };
    if !taggable(main) {
        main = CK_FOX;
    }
    let mut partner = if PLAN[P_PARTNER] >= 0 { PLAN[P_PARTNER] } else { css_partner };
    if !taggable(partner) || partner == main {
        partner = if main == CK_MARTH { CK_FOX } else { CK_MARTH };
    }
    ROSTER = [main, partner];
    pl::TAG_PARTNER_CK = [partner as i8, -1, -1, -1];

    let me = &mut d.players[0];
    me.ckind = main as i8;
    me.slot_type = 0;
    // test knob (headless win-path checks): MELEE_ENVOY_TEST_P1CPU=1 makes the player a level-9 CPU
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
    let n = PLAN[P_NCPU].clamp(1, 3) as usize;
    for i in 0..3 {
        let p = &mut d.players[1 + i];
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
        p.slot = (i + 2) as u8; // the CSS's own numbering for a CPU row
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
        let c = &mut d.players[1];
        c.cpu_kind = 0;
        c.cpu_level = 1;
        c.stocks = 1;
        c.model_scale = 1.0;
        c.attack_ratio = 1.0;
        c.defense_ratio = 1.0;
        // HP scales with the players on the player's side (P3: 1-2; one today, two once Envoy has a 2nd human)
        let side = d.players[0].team;
        let humans = [0usize, 2, 3].iter().filter(|&&i| d.players[i].slot_type != Gm_PKind_NA as u8 && d.players[i].team == side).count();
        crate::tu::petey::arm(humans.clamp(1, 2) as i32, 1, 0x5E7E_u32 ^ (SEQ as u32).wrapping_mul(0x9E37_79B9));
        melee_fighter::tu::ftdrawcommon::BOSS_DRAW = Some(crate::tu::petey::draw);
    } else {
        crate::tu::petey::disarm();
    }
    SEQ = SEQ.wrapping_add(1);
    CAMP_DUE = 1;
    eprintln!("envoy: match {} rewritten: main {main} partner {partner} stocks {} cpus {n} stage {}", SEQ, me_stocks(d), PLAN[P_STAGE]);
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
