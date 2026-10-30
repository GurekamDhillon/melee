//! src/melee/ft/kinds/ftPeach/ftpeachattacks4.c: Peach's forward smash (club / pan / racket).
//! No fused sites (tools/rust-port/fma_sites.py: 0 in every function). Smoke-tested through
//! recording hooks by tests/tu_ftpeachattacks4.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/5): re-exported under this TU.
pub use crate::chars::r2::ftPeach::ftpeachattacks4::{ftPe_AttackS4_Anim, ftPe_AttackS4_Phys};
pub use crate::chars::r5::ftPeach::ftpeachattacks4::ftPe_AttackS4_Coll;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    /// `HSD_Randi(max)` (baselib/random.c); unset, group 3's `hooks::hsd_randi`.
    pub static mut HSD_RANDI: Option<fn(i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_Randi(max: i32) -> i32 {
        match unsafe { HSD_RANDI } {
            Some(f) => f(max),
            None => melee_fighter::hooks::hsd_randi(max),
        }
    }
}

pub fn ftPe_AttackS4_Enter(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_AttackS4_Enter");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).set_allow_interrupt(0);
        (*fp).cmd_vars[0] = 0;
        let min = ftPe_MS_AttackS4Club as i32;
        let max = ftPe_MS_AttackS4Racket as i32;
        let mut msid;
        loop {
            msid = ext::HSD_Randi(max - min + 1) + min;
            if msid != (*fp).u.pe.attacks4_motion_id {
                break;
            }
        }
        (*fp).u.pe.attacks4_motion_id = msid;
        st::Fighter_ChangeMotionState(gobj, msid, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
    }
}

pub fn ftPe_AttackS4_IASA(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftPe_AttackS4_IASA");
    unsafe {
        if (*((*gobj).user_data as *mut Fighter)).allow_interrupt() != 0 {
            st::ftCo_Wait_IASA(gobj);
        }
    }
}
