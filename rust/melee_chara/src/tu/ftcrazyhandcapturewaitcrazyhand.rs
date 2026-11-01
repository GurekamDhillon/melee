//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturewaitcrazyhand.c: `ftCh_GrabUnk1_8015B800`,
//! the release of the grabbed fighter. No float math.
#![allow(non_snake_case, unused_unsafe)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::stubs as st;
#[allow(unused_imports)]
use melee_types::{Fighter, HSD_GObj};
use melee_types::*;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTCO_CAPTURECUT_ENTER, ftCo_CaptureCut_Enter, melee_fighter::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter, (gobj: *mut HSD_GObj));
}

/// `ftCh_GrabUnk1_8015B800`: show the fighter again, turn it around and cut the capture.
pub fn ftCh_GrabUnk1_8015B800(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015B800");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).set_invisible(0);
        (*fp).facing_dir = -(*fp).facing_dir;
        ext::ftCo_CaptureCut_Enter(gobj);
    }
}

// ---- folded from chars blocks (consolidation step 20) ----
/// `ftCh_GrabUnk1_8015B778` (src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturewaitcrazyhand.c).
pub fn ftCh_GrabUnk1_8015B778(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCh_GrabUnk1_8015B778");
    unsafe {
        let fp: *mut Fighter = (*gobj).user_data as *mut Fighter;
        st::Fighter_ChangeMotionState(gobj, 338 /* 0x152 */, 0u32, 0.0_f32, 1.0_f32, 0.0_f32, core::ptr::null_mut());
        (*fp).set_invisible(1);
        st::ftCommon_8007E2F4(&mut *fp, (511 /* (s16) (511) */ as i16));
        (*fp).set_x2220_b3(1);
        st::ftAnim_8006EBA4(gobj);
    }
}
