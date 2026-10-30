//! src/melee/ft/kinds/ftMasterHand/ftmasterhandfingergun3.c: Master Hand finger-gun end state.
//! No float arithmetic. Smoke-tested by tests/tu_ftmasterhandfingergun3.rs.
#![allow(non_snake_case, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::*;

/// Already ported (rounds 2/8): re-exported under this TU.
pub use crate::chars::r2::ftMasterHand::ftmasterhandfingergun3::{ftMh_FingerGun3_Phys, ftMh_MS_366_80153820};
pub use crate::chars::r8::ftMasterHand::ftmasterhandfingergun3::ftMh_FingerGun3_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook.
pub mod ext {
    pub use crate::tu::ftmasterhanddamage0::ext::ftMh_MS_389_80151018;
}

pub fn ftMh_FingerGun3_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_FingerGun3_Anim");
    if st::ftAnim_IsFramesRemaining(gobj) == 0 {
        ext::ftMh_MS_389_80151018(gobj);
    }
}

pub fn ftMh_FingerGun3_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_FingerGun3_Coll");}
