
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
pub(crate) const GM_PLAYER_OTHER: i32 = 6;

#[inline]
pub(crate) fn ext(slot: i32) -> &'static mut pl_StaleMoveTableExt_t {
    unsafe { &mut *crate::player::get_stale_move_table_index_ptr2(slot) }
}

#[inline]
pub(crate) fn hsd_assert(line: u32, cond: bool, what: &str) {
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

/// `pl_Verify_gm_8016AEDC` (pl/inlines.h).
#[inline]
pub(crate) fn verify_gm_8016aedc() -> bool {
    h::gm_get_frame_count() != 0 && h::gm_get_frame_count() != (-2i32) as u32
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

pub use crate::tu::plbonus::fn_80038700_fp as fn_80038700;
pub use crate::tu::plbonus::fn_80038fb8;
pub use crate::tu::plbonus::pl_800386d8;
pub use crate::tu::plbonus::pl_800386e8;
pub use crate::tu::plbonus::pl_80038788_fp as pl_80038788;
pub use crate::tu::plbonus::pl_80038824_fp as pl_80038824;
pub use crate::tu::plbonus::pl_80038898_fp as pl_80038898;
pub use crate::tu::plbonus::pl_80038914_fp as pl_80038914;
pub use crate::tu::plbonus::pl_80038f10;
pub use crate::tu::plbonus::pl_8003906c;
pub use crate::tu::plbonus::pl_80039238_fp as pl_80039238;
pub use crate::tu::plbonus::pl_80039418_fp as pl_80039418;
