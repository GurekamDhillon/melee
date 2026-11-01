//! melee/ft/kinds/ftCrazyHand/ftcrazyhanddrill.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `fn_80157C50` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddrill.c).
pub fn fn_80157C50(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80157C50");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 354 /* ftMh_MS_Drill */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.ch.unk0.x0 = f32::from_bits(0x00000000) /* 0 */;
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_Drill_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhanddrill.c).
pub fn ftCh_Drill_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Drill_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_id as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
