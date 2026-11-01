//! Samus (src/melee/ft/kinds/ftSamus). The four entry points below take the fighter (not the gobj) for the
//! `chars_dispatch` oracle tests; the bodies are the TU ports (`tu::ftsamusspeciallw0/1`), called through a stand-in gobj.

use melee_types::{Fighter, HSD_GObj};

/// A gobj whose `user_data` is `fp` (what `GET_FIGHTER` reads); NULL for a missing fighter.
pub(crate) fn gobj_for(fp: *const Fighter) -> HSD_GObj {
    let mut g: HSD_GObj = unsafe { core::mem::zeroed() };
    g.user_data = fp as *mut core::ffi::c_void;
    g
}

pub mod ftsamusspeciallw0 {
pub use crate::tu::ftsamusspeciallw0::ftSs_SpecialAirLw_Phys_fp as ftSs_SpecialAirLw_Phys;
pub use crate::tu::ftsamusspeciallw0::ftSs_SpecialLw_80129158_fp as ftSs_SpecialLw_80129158;
pub use crate::tu::ftsamusspeciallw0::ftSs_SpecialN_801291A8_fp as ftSs_SpecialN_801291A8;
pub use crate::tu::ftsamusspeciallw0::opt_gobj;

use super::gobj_for;

use melee_types::Fighter;

}

pub mod ftsamusspeciallw1 {
pub use crate::tu::ftsamusspeciallw1::ftSs_SpecialAirLwBomb_Phys_fp as ftSs_SpecialAirLwBomb_Phys;

use super::gobj_for;

use melee_types::Fighter;

}
