//! Group 3 round 11: the bonus-value primitives at the top of src/melee/pl/plbonus.c (15 C
//! definitions incl. the statics `setFlag`, `setPointValue` (inline) and `resetBonuses` (inline)):
//! field logic over `pl_StaleMoveTableExt_t::x0_staleMoveTable.x904` (the 215 bonus values).
//! Calls out: `gmDecisionGetType` (group 6, gm_16F1.c, only for the HSD_ASSERTs) and
//! `gm_GetFrameCount` through `crate::hooks`. No fused site (fma_sites.py: no a*b+c here).
//! Checked against the C (cut verbatim) by tests/plbonusc_ref.rs (gen/cref/plbonusc_ref.c).
//!
//! HSD_ASSERT is live in the C build (debug.h, `__assert`); the port panics where it would assert.

use crate::hooks as h;
use crate::types::*;

/// `Gm_DecType_Flag` (gm_16F1.h).
pub const GM_DEC_TYPE_FLAG: i32 = 0;
/// `Gm_DecType_Point`.
pub const GM_DEC_TYPE_POINT: i32 = 1;
/// `Gm_Player_Other` (gm/forward.h).
const GM_PLAYER_OTHER: i32 = 6;

#[inline]
fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

#[inline]
fn hsd_assert(line: u32, cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT plbonus.c:{line} {what}");
    }
}

/// `setFlag` (static).
pub fn set_flag(player: i32, kind: i32) {
    let table = ext(player);
    hsd_assert(0x38, player != GM_PLAYER_OTHER, "player != Gm_Player_Other");
    hsd_assert(0x39, h::gm_decision_get_type(kind) == GM_DEC_TYPE_FLAG, "gmDecisionGetType(kind) == Gm_DecType_Flag");
    table.x0_staleMoveTable.x904[kind as usize] = 1;
}

/// `setPointValue` (static inline).
pub fn set_point_value(player: i32, kind: i32, val: u32) {
    let table = ext(player);
    hsd_assert(0x50, player != GM_PLAYER_OTHER, "player != Gm_Player_Other");
    hsd_assert(0x51, h::gm_decision_get_type(kind) == GM_DEC_TYPE_POINT, "gmDecisionGetType(kind) == Gm_DecType_Point");
    table.x0_staleMoveTable.x904[kind as usize] = val;
}

/// `pl_800386D8`.
pub fn pl_800386d8(arg0: &plActionStats, arg1: i32) -> u32 {
    unsafe { arg0.anon_358.anon_0.by_attack_hi[arg1 as usize] }
}

/// `pl_800386E8`.
pub fn pl_800386e8(arg0: &pl_800386E8_arg0_t) -> i32 {
    let temp = arg0.unk_190.wrapping_add(arg0.unk_5AC);
    temp.wrapping_add(arg0.unk_5B0)
}

/// `fn_80038700`: sum of `arg0[i + 1]` for i in arg1..=arg2.
pub fn fn_80038700(arg0: &[i32], arg1: i32, arg2: i32) -> i32 {
    let mut sum: i32 = 0;
    let mut i = arg1;
    while i <= arg2 {
        sum = sum.wrapping_add(arg0[(i + 1) as usize]);
        i += 1;
    }
    sum
}

pub fn pl_80038788(player: i32, kind: i32, arg2: i32) {
    unsafe { crate::tu::plbonus::pl_80038788(player, kind, arg2) }
}

pub fn pl_80038824(player: i32, kind: i32) {
    unsafe { crate::tu::plbonus::pl_80038824(player, kind) }
}

pub fn pl_80038898(player: i32, kind: i32) {
    unsafe { crate::tu::plbonus::pl_80038898(player, kind) }
}

pub fn pl_80038914() -> *const pl_804D6470_t {
    unsafe { crate::tu::plbonus::pl_80038914() as _ }
}

pub use crate::tu::plbonus::pl_80038F10 as pl_80038f10;

pub use crate::tu::plbonus::fn_80038FB8 as fn_80038fb8;

/// `pl_Verify_gm_8016AEDC` (pl/inlines.h).
#[inline]
fn verify_gm_8016aedc() -> bool {
    h::gm_get_frame_count() != 0 && h::gm_get_frame_count() != (-2i32) as u32
}

pub use crate::tu::plbonus::pl_8003906C as pl_8003906c;

pub use crate::tu::plbonus::pl_80039238;

pub fn pl_80039418(player: i32, kind: i32) -> u32 {
    unsafe { crate::tu::plbonus::pl_80039418(player, kind) }
}

/// `resetBonuses` (static inline): clears every flag-type bonus.
pub fn reset_bonuses(player: i32) {
    let t = ext(player);
    for i in 0..215 {
        if h::gm_decision_get_type(i) == GM_DEC_TYPE_FLAG {
            t.x0_staleMoveTable.x904[i as usize] = 0;
        }
    }
}
