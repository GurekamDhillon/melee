//! src/melee/ft/kinds/ftMasterHand/ftmasterhandwait11.c. Both functions were ported in
//! round 9 (`chars::r9`) and are re-exported under this TU. No float math, no fused sites.
#![allow(non_snake_case)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_MS_378_80154CF8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwait11.c).
pub fn ftMh_MS_378_80154CF8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_378_80154CF8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 377 /* ftMh_MS_Squeezing1 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.x;
        (*fp).mv.mh.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x30_pos2.y;
        (*fp).mv.mh.unk0.xC.z = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftMh_MS_375_80154C78` (src/melee/ft/kinds/ftMasterHand/ftmasterhandwait11.c).
pub fn ftMh_MS_375_80154C78(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_375_80154C78");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 376 /* ftMh_MS_Squeezing0 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.xC.x = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x118_pos.x;
        (*fp).mv.mh.unk0.xC.y = (*((*(*fp).ft_data).ext_attr as *const melee_types::ftMasterHand_SpecialAttrs)).x118_pos.y;
        (*fp).mv.mh.unk0.xC.z = f32::from_bits(0x00000000) /* 0 */;
    }
}
