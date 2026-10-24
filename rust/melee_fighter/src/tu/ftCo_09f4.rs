//! src/melee/ft/kinds/ftCommon/ftCo_09F4.c, the part the fighter material code calls:
//! `ftCo_8009F75C` (switch the fighter light overlay on / off for the next draw) and the file
//! statics `lobj0` / `lobj1` it reads. (`ftCo_8009F4A4`, which creates those lights, is not
//! ported.) No fused sites (fma_sites.py).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ptr::null_mut;

/// `lobj0`: the stage light with the fighter-overlay setup.
pub static mut lobj0: *mut HSD_LObj = null_mut();
/// `lobj1`: the copy lit without the overlay.
pub static mut lobj1: *mut HSD_LObj = null_mut();

/// `ftCo_8009F75C(fp, set_alpha)`.
pub unsafe fn ftCo_8009F75C(fp: *mut Fighter, set_alpha: bool) {
    let data = crate::ft0df::ftco_800c0658(&mut *fp);
    if (*data).x7C_flag2() != 0 && (*data).x7C_light_enable() != 0 {
        if set_alpha {
            melee_runtime::tu::lobj::HSD_LObjSetFlags((*fp).x588, LOBJ_ALPHA as u32);
        } else {
            melee_runtime::tu::lobj::HSD_LObjClearFlags((*fp).x588, LOBJ_ALPHA as u32);
        }
        if set_alpha {
            melee_runtime::tu::lobj::HSD_LObj_803668EC(lobj0);
        } else {
            melee_runtime::tu::lobj::HSD_LObjSetCurrentAll(lobj1);
        }
        melee_runtime::tu::lobj::HSD_LObjAddCurrent((*fp).x588);
        melee_runtime::tu::lobj::HSD_LObjSetupInit(melee_runtime::tu::cobj::HSD_CObjGetCurrent());
    }
}
