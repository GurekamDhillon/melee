//! src/melee/ft/kinds/ftMasterHand/ftmasterhandpapercrush.c: Master Hand rock-crush-down /
//! paper-crush states. No fused sites (fma_sites.py); the countdown compares an f32 against s32
//! attributes as floats. Smoke-tested by tests/tu_ftmasterhandpapercrush.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/3/8): re-exported under this TU.




/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddrill::ext::{ftBossLib_8015C010, ftBossLib_8015C190};
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
}

pub fn ftMh_RockCrushDown_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushDown_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_RockCrushDown_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_RockCrushDown_Coll");}

pub fn ftMh_PaperCrush_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_PaperCrush_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_PaperCrush_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_PaperCrush_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        st::ft_80085134(gobj);
        (*fp).mv.mh.unk0.x0 += 1.0;
        let f = (*fp).mv.mh.unk0.x0;
        if f > (*da).xB0 as f32 && (*fp).mv.mh.unk0.x0 < (*da).xB4 as f32 {
            ext::ftBossLib_8015C010(gobj, (*da).xB8);
        } else {
            (*fp).self_vel.x = 0.0;
        }
        ext::ftBossLib_8015C190(gobj);
    }
}

// ---- folded from chars (consolidation step 20/21) ----
/// `ftMh_RockCrushDown_Phys` (src/melee/ft/kinds/ftMasterHand/ftmasterhandpapercrush.c).
pub fn ftMh_RockCrushDown_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushDown_Phys");
    st::ft_80085134(gobj);
}

/// `ftMh_MS_357_801526D8` (src/melee/ft/kinds/ftMasterHand/ftmasterhandpapercrush.c).
pub fn ftMh_MS_357_801526D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_357_801526D8");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 358 /* ftMh_MS_PaperCrush */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).mv.mh.unk0.x0 = f32::from_bits(0x00000000) /* 0 */;
    }
}

/// `ftMh_RockCrushDown_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandpapercrush.c).
pub fn ftMh_RockCrushDown_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_RockCrushDown_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}

/// `ftMh_PaperCrush_IASA` (src/melee/ft/kinds/ftMasterHand/ftmasterhandpapercrush.c).
pub fn ftMh_PaperCrush_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_PaperCrush_IASA");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        if (st::Player_GetPlayerSlotType(((*fp).player_idx as i32)) == 0) {
            st::ftBossLib_8015BD20(gobj);
        }
    }
}
