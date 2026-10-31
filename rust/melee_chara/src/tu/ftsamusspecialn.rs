//! src/melee/ft/kinds/ftSamus/ftsamusspecialn.c plus the Samus inlines (`inlines.h`). FMA: `fma_sites.py` reports one fused site in `ftSs_SpecialS_8012A074` (`position.x += x34 * facing_dir`).
#![allow(non_snake_case, unused_unsafe)]

use melee_core::fp::fmadds;
use melee_types::*;

type G = *mut HSD_GObj;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    pub use crate::tu::ftcaptainspecialn::ext::{efSync_Spawn, EFSYNC_SPAWN};
    crate::port_hook!(LB_8000B1CC, lb_8000B1CC, melee_runtime::tu::lb_00B0::lb_8000B1CC, (jobj: *mut HSD_JObj, off: *mut Vec3, out: *mut Vec3));
    crate::port_hook!(EFLIB_DESTROYALL, efLib_DestroyAll, melee_world::tu::efn::eflib::efLib_DestroyAll, (gobj: *mut HSD_GObj));
    crate::port_hook!(IT_802B5974, it_802B5974, melee_items::tu::itsamuschargeshot::it_802B5974, (gobj: *mut HSD_GObj));
    crate::port_hook!(IT_802B62D0, it_802B62D0, crate::adapt::it_802B62D0, (gobj: *mut HSD_GObj, pos: *mut Vec3, is_smash: bool, dir: f32) -> *mut HSD_GObj);
}

#[inline(always)]
unsafe fn fpg(g: G) -> *mut Fighter {
    (*g).user_data as *mut Fighter
}

/// `ftCheckThrowB0` (ft/inlines.h): test and clear throw_flags_b0.
pub(crate) unsafe fn ftCheckThrowB0(fp: *mut Fighter) -> bool {
    melee_core::ct!("ftCheckThrowB0");
    if (*fp).anon_2210.anon_0.throw_flags_b0() != 0 {
        (*fp).anon_2210.anon_0.set_throw_flags_b0(0);
        true
    } else {
        false
    }
}

/// `ftSamus_destroyAllEF` (inlines.h).
unsafe fn ftSamus_destroyAllEF(gobj: G) {
    if !gobj.is_null() {
        let fp = fpg(gobj);
        if (*fp).u.ss.x2234 != 0 {
            ext::efLib_DestroyAll(gobj);
            (*fp).u.ss.x2234 = 0;
        }
    }
}

/// `ftSamus_UnkAndDestroyAllEF` (inlines.h).
unsafe fn ftSamus_UnkAndDestroyAllEF(gobj: G) {
    if !gobj.is_null() {
        let fp = fpg(gobj);
        let x222c = (*fp).u.ss.x222C;
        if !x222c.is_null() {
            ext::it_802B5974(x222c);
            (*fp).u.ss.x222C = core::ptr::null_mut();
        }
        ftSamus_destroyAllEF(gobj);
    }
}

/// `ftSs_SpecialN_80129258` (ftsamusspecialn.c).
pub fn ftSs_SpecialN_80129258(gobj: G) {
    melee_core::ct!("ftSs_SpecialN_80129258");
    unsafe {
        if !gobj.is_null() {
            let fp = fpg(gobj);
            ftSamus_UnkAndDestroyAllEF(gobj);
            (*fp).u.ss.x2230 = 0;
        }
    }
}

/// `ftSs_SpecialS_8012A168` (static, ftsamusspecialn.c): the first missile of the move spawns its
/// flash effect.
unsafe fn ftSs_SpecialS_8012A168(gobj: G, spawnlocation: *mut Vec3) {
    let fp = fpg(gobj);
    if (*fp).x2219_b0() == 0 {
        ext::efSync_Spawn(1155, gobj, &[spawnlocation as *mut core::ffi::c_void]);
        (*fp).set_x2219_b0(1);
    }
    crate::tu::ftinl::Fighter_SetEffectHitlagCallbacks(fp);
}

/// `ftSs_SpecialS_8012A074` (ftsamusspecialn.c): spawn the missile at the throw frame.
pub fn ftSs_SpecialS_8012A074(gobj: G) {
    melee_core::ct!("ftSs_SpecialS_8012A074");
    unsafe {
        let fp = fpg(gobj);
        let attr = (*fp).dat_attrs as *const ftSs_DatAttrs;
        if ftCheckThrowB0(fp) {
            let mut position: Vec3 = core::mem::zeroed();
            (*fp).u.ss.x2238 = (*fp).u.ss.x2238.wrapping_add(1);
            ext::lb_8000B1CC((*(*fp).parts.add(FtPart_56 as usize)).joint, core::ptr::null_mut(), &mut position);
            position.x = fmadds((*attr).x34, (*fp).facing_dir, position.x);
            if (*fp).motion_id == ftSs_MS_SpecialS as i32 || (*fp).motion_id == ftSs_MS_SpecialAirS as i32 {
                ext::it_802B62D0(gobj, &mut position, false, (*fp).facing_dir);
            } else {
                ext::it_802B62D0(gobj, &mut position, true, (*fp).facing_dir);
            }
            ftSs_SpecialS_8012A168(gobj, &mut position);
            (*fp).accessory4_cb = None;
        }
    }
}
