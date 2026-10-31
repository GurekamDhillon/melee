//! Samus (src/melee/ft/kinds/ftSamus). The four entry points below take the fighter (not the gobj) for the
//! `chars_dispatch` oracle tests; the bodies are the TU ports (`tu::ftsamusspeciallw0/1`), called through a stand-in gobj.

use melee_types::{Fighter, HSD_GObj};

/// A gobj whose `user_data` is `fp` (what `GET_FIGHTER` reads); NULL for a missing fighter.
fn gobj_for(fp: *const Fighter) -> HSD_GObj {
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = fp as *mut core::ffi::c_void;
    g
}

pub mod ftsamusspeciallw0 {
    use super::gobj_for;
    use melee_types::Fighter;

    pub fn ftSs_SpecialAirLw_Phys(fp: &mut Fighter) {
        unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialAirLw_Phys(&mut gobj_for(fp)) }
    }

    /// The gobj may be NULL (then 1).
    pub fn ftSs_SpecialLw_80129158(fp: Option<&Fighter>) -> i32 {
        unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialLw_80129158(opt_gobj(fp)) }
    }

    /// The gobj may be NULL (then 1).
    pub fn ftSs_SpecialN_801291A8(fp: Option<&Fighter>) -> i32 {
        unsafe { crate::tu::ftsamusspeciallw0::ftSs_SpecialN_801291A8(opt_gobj(fp)) }
    }

    fn opt_gobj(fp: Option<&Fighter>) -> *mut melee_types::HSD_GObj {
        match fp {
            Some(f) => Box::leak(Box::new(gobj_for(f))),
            None => core::ptr::null_mut(),
        }
    }
}

pub mod ftsamusspeciallw1 {
    use super::gobj_for;
    use melee_types::Fighter;

    pub fn ftSs_SpecialAirLwBomb_Phys(fp: &mut Fighter) {
        unsafe { crate::tu::ftsamusspeciallw1::ftSs_SpecialAirLwBomb_Phys(&mut gobj_for(fp)) }
    }
}
