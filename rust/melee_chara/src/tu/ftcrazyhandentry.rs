//! melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `fn_80156F6C` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c).
pub fn fn_80156F6C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80156F6C");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 343 /* ftMh_MS_Entry */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).cur_pos.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftCrazyHand_DatAttrs)).x20;
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_Entry_Phys` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c).
pub fn ftCh_Entry_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Entry_Phys");
    st::ft_80085134(gobj);
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_Entry_IASA` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c).
pub fn ftCh_Entry_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Entry_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0 /* Gm_PKind_Human */) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftCh_Entry_Anim` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c).
pub fn ftCh_Entry_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_Entry_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        crate::tu::ftcrazyhandfingerbeam0::ext::ftCh_GrabUnk1_8015BC88(gobj);
    }
}

/// `ftCh_Entry_Coll` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandentry.c).
pub fn ftCh_Entry_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCh_Entry_Coll"); }
