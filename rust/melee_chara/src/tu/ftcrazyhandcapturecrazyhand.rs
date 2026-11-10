//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturecrazyhand.c: the victim's CaptureCrazyHand
//! state (entered through `fn_8015B548`, the finger-beam `grabbed_cb`). No float arithmetic, no
//! fused sites (fma_sites.py).
#![allow(non_snake_case, unused_unsafe)]
use crate::stubs as st;
use melee_types::*;

/// `fn_8015B548(gobj, gobj2)`: Crazy Hand (`gobj2`) grabbed `gobj`.
pub fn fn_8015B548(gobj: *mut HSD_GObj, gobj2: *mut HSD_GObj) {
    melee_core::ct!("fn_8015B548");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let fp2 = (*gobj2).user_data as *mut Fighter;
        melee_fighter::tu::ftCo_Barrel::ftCo_ReleaseItemAndVictim(gobj);
        (*fp).x1A5C = gobj2;
        (*fp).victim_gobj = gobj2;
        (*fp).x221B.set_x221B_b5(0);
        (*fp).x221B.set_x221B_b7(0);
        (*fp).facing_dir = (*fp2).facing_dir;
        let t = melee_fighter::tu::ftCo_CapturePulled::ftCo_800DA824(&mut *fp2);
        melee_fighter::ftcommon2::init_grab(&mut *fp, 0, t);
        melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *fp2, &mut *fp);
        (*fp).accessory1_cb = Some(melee_fighter::tu::ftCo_CapturePulled::ftCo_800DB464_cb);
        st::ftCommon_8007D5D4(&mut *fp);
        st::Fighter_ChangeMotionState(gobj, 0x150, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).set_invisible(1);
        st::ftCommon_8007E2F4(&mut *fp, 0x1FF);
        (*fp).set_x2220_b3(1);
        st::ftAnim_8006EBA4(gobj);
        st::ftCommon_8007E2FC(gobj);
    }
}

/// `fn_8015B548` as the `grabbed_cb` (`HSD_GObjInteraction`) value.
pub unsafe extern "C" fn cb_fn_8015B548(gobj: *mut HSD_GObj, gobj2: *mut HSD_GObj) {
    fn_8015B548(gobj, gobj2)
}

pub fn ftCo_CaptureCrazyHand_Anim(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureCrazyHand_Anim"); }
pub fn ftCo_CaptureCrazyHand_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureCrazyHand_IASA"); }
pub fn ftCo_CaptureCrazyHand_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureCrazyHand_Phys"); }
pub fn ftCo_CaptureCrazyHand_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftCo_CaptureCrazyHand_Coll"); }
