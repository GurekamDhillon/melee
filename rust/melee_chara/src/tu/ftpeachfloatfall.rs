//! src/melee/ft/kinds/ftPeach/ftpeachfloatfall.c: Peach's float-fall (forward / backward) states.
//! No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested through
//! recording hooks by tests/tu_ftpeachfloatfall.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_fighter::hooks::common_data;
#[allow(unused_imports)]
use melee_types::{ftPe_MS_FloatFallB, ftPe_MS_FloatFallF, ftPeach_MotionState, Fighter};
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
            None => { unsafe { melee_fighter::tu::ftCo_FallAerial::ftCo_FallAerial_Enter(gobj as _); } return; },
        }
    }
}

/// Already ported (chars::ftPeach): re-exported under this TU.


#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

pub fn ftPe_UpdateFloatDir(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_UpdateFloatDir");
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
    melee_core::ct!("ftPe_FloatFall_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftCo_FallAerial_Enter(gobj);
    }
}

pub fn ftPe_FloatFall_IASA(_gobj: *mut HSD_GObj) { melee_core::ct!("ftPe_FloatFall_IASA");}

pub fn ftPe_FloatFall_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_FloatFall_Phys");
    fext::ftCo_JumpAerial_Phys_Cb(gobj);
}

pub fn ftPe_FloatFall_Coll(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_FloatFall_Coll");
    fext::ft_800831CC(gobj, Some(cb_ftCo_80096CC8), Some(cb_ft_80082B1C));
}

// ---- folded from chars (consolidation step 20/21) ----
/// `getFloatDir` (static): stick x times facing past `-p_ftCommonData->x78` -> forward.
pub fn getFloatDir(fp: &Fighter) -> ftPeach_MotionState {
    melee_core::ct!("getFloatDir");
    if fp.input.lstick[0].x * fp.facing_dir > -common_data().x78 {
        ftPe_MS_FloatFallF
    } else {
        ftPe_MS_FloatFallB
    }
}
