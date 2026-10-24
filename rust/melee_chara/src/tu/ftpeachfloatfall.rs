//! src/melee/ft/kinds/ftPeach/ftpeachfloatfall.c: Peach's float-fall (forward / backward) states.
//! No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested through
//! recording hooks by tests/tu_ftpeachfloatfall.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use crate::tu::ftpeachfloat::{cb_ftCo_80096CC8, cb_ft_80082B1C, ext as fext};
use melee_types::*;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `ftCo_FallAerial_Enter(gobj)` (ftCommon/ftCo_FallAerial.c).
    pub static mut FTCO_FALLAERIAL_ENTER: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn ftCo_FallAerial_Enter(gobj: *mut HSD_GObj) {
        match unsafe { FTCO_FALLAERIAL_ENTER } {
            Some(f) => f(gobj),
            None => { unsafe { let _ = melee_fighter::tu::ftCo_FallAerial::ftCo_FallAerial_Enter(gobj as _); } return; },
        }
    }
}

/// Already ported (chars::ftPeach): re-exported under this TU.
pub use crate::chars::ftPeach::ftpeachfloatfall::getFloatDir;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPe_UpdateFloatDir(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftPe_DatAttrs;
        let msid = getFloatDir(&*fp);
        let anim_start = if msid == ftPe_MS_FloatFallF {
            (*da).floatfallf_anim_start
        } else {
            (*da).floatfallb_anim_start
        };
        st::Fighter_ChangeMotionState(
            gobj,
            msid as i32,
            Ft_MF_KeepGfx,
            anim_start - (*da).floatfall_anim_start_offset,
            1.0,
            0.0,
            core::ptr::null_mut(),
        );
    }
}

pub fn ftPe_FloatFall_Anim(gobj: *mut HSD_GObj) {
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftCo_FallAerial_Enter(gobj);
    }
}

pub fn ftPe_FloatFall_IASA(_gobj: *mut HSD_GObj) {}

pub fn ftPe_FloatFall_Phys(gobj: *mut HSD_GObj) {
    fext::ftCo_JumpAerial_Phys_Cb(gobj);
}

pub fn ftPe_FloatFall_Coll(gobj: *mut HSD_GObj) {
    fext::ft_800831CC(gobj, Some(cb_ftCo_80096CC8), Some(cb_ft_80082B1C));
}
