//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandcapturewaitcrazyhand.c: `ftCh_GrabUnk1_8015B800`,
//! the release of the grabbed fighter. No float math.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::*;

/// Calls into other TUs (settable hooks; unset they run the port).
pub mod ext {
    use melee_types::*;
    crate::port_hook!(FTCO_CAPTURECUT_ENTER, ftCo_CaptureCut_Enter, melee_fighter::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter, (gobj: *mut HSD_GObj));
}

/// `ftCh_GrabUnk1_8015B800`: show the fighter again, turn it around and cut the capture.
pub fn ftCh_GrabUnk1_8015B800(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).set_invisible(0);
        (*fp).facing_dir = -(*fp).facing_dir;
        ext::ftCo_CaptureCut_Enter(gobj);
    }
}
