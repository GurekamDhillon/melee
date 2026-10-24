//! src/melee/ft/kinds/ftSamus/ftsamusspeciallw1.c: `ftSs_SpecialLw_8012ADF0`. FMA: `fma_sites.py` reports one fused site (`vec.x += attr.x74_vec.x * facing_dir`).
#![allow(non_snake_case, unused_unsafe)]

use melee_core::fp::fmadds;
use melee_types::*;

type G = *mut HSD_GObj;

#[inline(always)]
unsafe fn fpg(g: G) -> *mut Fighter {
    (*g).user_data as *mut Fighter
}

/// `ftSs_SpecialLw_8012ADF0` (ftsamusspeciallw1.c): spawn the bomb at the throw frame.
pub fn ftSs_SpecialLw_8012ADF0(gobj: G) {
    unsafe {
        let fp = fpg(gobj);
        let attr = (*fp).dat_attrs as *const ftSs_DatAttrs;
        if crate::tu::ftsamusspecialn::ftCheckThrowB0(fp) {
            let mut vec: Vec3 = core::mem::zeroed();
            melee_runtime::tu::lb_00B0::lb_8000B1CC((*(*fp).parts.add(FtPart_TopN as usize)).joint, core::ptr::null_mut(), &mut vec);
            vec.x = fmadds((*attr).x74_vec.x, (*fp).facing_dir, vec.x);
            vec.y += (*attr).x74_vec.y;
            vec.z += (*attr).x74_vec.z;
            melee_items::tu::itsamusbomb::it_802B4AC8(gobj, &mut vec, (*fp).facing_dir);
            (*fp).accessory4_cb = None;
        }
    }
}

