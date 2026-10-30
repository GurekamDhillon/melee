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

/// `pl_80038788`.
pub fn pl_80038788(player: i32, kind: i32, arg2: i32) {
    set_point_value(player, kind, arg2 as u32);
}

/// `pl_80038824`.
pub fn pl_80038824(player: i32, kind: i32) {
    let t = ext(player);
    hsd_assert(92, h::gm_decision_get_type(kind) == GM_DEC_TYPE_POINT, "gmDecisionGetType(kind) == Gm_DecType_Point");
    let v = &mut t.x0_staleMoveTable.x904[kind as usize];
    *v = v.wrapping_add(1);
}

/// `pl_80038898`.
pub fn pl_80038898(player: i32, kind: i32) {
    let t = ext(player);
    hsd_assert(103, h::gm_decision_get_type(kind) == GM_DEC_TYPE_POINT, "gmDecisionGetType(kind) == Gm_DecType_Point");
    let cur = t.x0_staleMoveTable.x904[kind as usize];
    if cur == 0 {
        return;
    }
    t.x0_staleMoveTable.x904[kind as usize] = cur - 1;
}

/// `pl_80038914`: the `pl_804D6470` constants pointer.
pub fn pl_80038914() -> *const pl_804D6470_t {
    unsafe { h::PL_804D6470 }
}

/// `pl_80038F10`.
pub fn pl_80038f10(player: i32) {
    let t = ext(player);
    t.xD50 = 0;
    t.xD54 = 0;
    t.xD60 = 0;
    t.xD64 = 6;
    for i in 0..6 {
        let o = ext(i);
        if player == o.xD64 {
            o.xD60 = 0;
            o.xD64 = 6;
        }
        if player as u32 == o.x0_staleMoveTable.xCAC {
            o.x0_staleMoveTable.xCAC = 6;
        }
        if player == o.xD6C {
            o.xD6C = 6;
        }
    }
}

/// `fn_80038FB8`.
pub fn fn_80038fb8(player: i32) {
    let t = ext(player);
    for v in t.x0_staleMoveTable.x904.iter_mut() {
        *v = 0;
    }
}

/// `pl_Verify_gm_8016AEDC` (pl/inlines.h).
#[inline]
fn verify_gm_8016aedc() -> bool {
    h::gm_get_frame_count() != 0 && h::gm_get_frame_count() != (-2i32) as u32
}

/// `pl_8003906C`. `arg2` is used when `kind == -1`.
#[allow(clippy::too_many_arguments)]
pub fn pl_8003906c(player: i32, kind: i32, arg2: &mut u32, arg3: u32, arg4: u32, arg5: u32, arg6: &mut i32) -> i32 {
    if verify_gm_8016aedc() {
        if h::gm_get_frame_count() <= arg3 {
            if arg5 >= arg4 {
                if kind != -1 {
                    set_point_value(player, kind, 1);
                } else {
                    *arg2 = 1;
                }
            }
            if arg3 == h::gm_get_frame_count() {
                *arg6 = arg5 as i32;
                return 1;
            }
        } else if h::gm_get_frame_count() % arg3 == 0 {
            let r3 = if kind != -1 { h::pl_80039418(player, kind) } else { *arg2 };
            if r3 as i32 != 0 && arg5.wrapping_sub(*arg6 as u32) < arg4 {
                if kind != -1 {
                    set_point_value(player, kind, 0);
                } else {
                    *arg2 = 0;
                }
            }
            *arg6 = arg5 as i32;
            return 1;
        }
    }
    0
}

/// `pl_80039238`.
#[allow(clippy::too_many_arguments)]
pub fn pl_80039238(player: i32, kind: i32, arg2: &mut i32, arg3: u32, arg4: &mut f32, arg6: f32, arg7: f32) -> bool {
    let r0 = h::gm_get_frame_count() != 0 && h::gm_get_frame_count() != (-2i32) as u32;
    if r0 {
        if h::gm_get_frame_count() <= arg3 {
            if arg7 >= arg6 {
                if kind != -1 {
                    set_point_value(player, kind, 1);
                } else {
                    *arg2 = 1;
                }
            }
            if arg3 == h::gm_get_frame_count() {
                *arg4 = arg7;
                return true;
            }
        } else if h::gm_get_frame_count() % arg3 == 0 {
            let r3 = if kind != -1 { h::pl_80039418(player, kind) as i32 } else { *arg2 };
            if r3 != 0 && arg7 - *arg4 < arg6 {
                if kind != -1 {
                    set_point_value(player, kind, 0);
                } else {
                    *arg2 = 0;
                }
            }
            *arg4 = arg7;
            return true;
        }
    }
    false
}

/// `pl_80039418`.
pub fn pl_80039418(player: i32, kind: i32) -> u32 {
    ext(player).x0_staleMoveTable.x904[kind as usize]
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
