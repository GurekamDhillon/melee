//! melee/ft/kinds/ftPurin/ftpurinspecials.c: ports folded from the hand-written character modules (consolidation step 20).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialS_Anim` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ft_8008A2BC(gobj);
    }
}

/// `ftPr_SpecialAirS_Anim` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialAirS_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        st::ftCo_Fall_Enter(gobj);
    }
}

/// `ftPr_SpecialS_Phys` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Phys");
    st::ft_80084FA8(gobj);
}

/// `ftPr_SpecialS_8013D590` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_8013D590(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_8013D590");
    let fp = unsafe { &mut *((*gobj).user_data as *mut Fighter) };
    st::ftCommon_GroundToAirStateChange(gobj, fp, 364 /* ftPr_MS_SpecialAirS */, 0xC4C508Au32 /* ftPr_MF_SpecialS_Coll */);
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialS_Coll` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_Coll");
    unsafe {
        if st::ft_800827A0(gobj) == 0 /* GA_Ground */ {
            crate::chars::r2::ftPurin::ftpurinspecials::ftPr_SpecialS_8013D590(gobj);
        }
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftPr_SpecialAirS_Coll` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialAirS_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialAirS_Coll");
    unsafe {
        if (st::ft_80081D0C(gobj) != 0 /* GA_Ground */) {
            crate::chars::r6::ftPurin::ftpurinspecials::ftPr_SpecialS_8013D5F0(gobj);
        }
    }
}

/// `ftPr_SpecialS_8013D5F0` (src/melee/ft/kinds/ftPurin/ftpurinspecials.c).
pub fn ftPr_SpecialS_8013D5F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPr_SpecialS_8013D5F0");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::ftCommon_AirToGroundStateChange(gobj, &mut *fp, 363 /* ftPr_MS_SpecialS */, 0xC4C508Au32 /* ftPr_MF_SpecialS_Coll */);
        st::ftCommon_ClampAirDrift(&mut *fp);
    }
}
