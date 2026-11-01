
//! The plain `Fighter` field accessors of `src/melee/ft/ftlib.c` that the stage, HUD and effect
//! TUs call with a fighter `HSD_GObj*` (`gobj->user_data` is the `Fighter`), in the C shape (raw
//! pointers, C names). They live here, in the lowest crate that needs them; `melee_fighter::ftlib`
//! (which depends on this crate and ports the other ~90 functions of the file) forwards its same-named
//! accessors to these, so there is a single port of each. The C dereferences without checks and so
//! do these: pass a live fighter gobj. No float arithmetic.
//! Checked by the fighter's `tests/ftlib_ref.rs` (gen/cref, ftlib.c verbatim), which runs on them.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_types::*;

#[inline(always)]
pub(crate) unsafe fn fp<'a>(gobj: *mut HSD_GObj) -> &'a mut Fighter {
    &mut *((*gobj).user_data as *mut Fighter)
}

#[inline]
pub(crate) unsafe fn msid_in(gobj: *mut HSD_GObj, lo: i32, hi: i32) -> bool {
    let m = fp(gobj).motion_id as i32;
    m >= lo && m <= hi
}

pub use crate::tu::ftlib::ftLib_80086644_fp as ftLib_80086644;
pub use crate::tu::ftlib::ftLib_80086684_fp as ftLib_80086684;
pub use crate::tu::ftlib::ftLib_80086960_fp as ftLib_80086960;
pub use crate::tu::ftlib::ftLib_80086984_fp as ftLib_80086984;
pub use crate::tu::ftlib::ftLib_80086A4C_fp as ftLib_80086A4C;
pub use crate::tu::ftlib::ftLib_80086B64_fp as ftLib_80086B64;
pub use crate::tu::ftlib::ftLib_80086B74_fp as ftLib_80086B74;
pub use crate::tu::ftlib::ftLib_80086B80_fp as ftLib_80086B80;
pub use crate::tu::ftlib::ftLib_80086B90_fp as ftLib_80086B90;
pub use crate::tu::ftlib::ftLib_80086BE0_fp as ftLib_80086BE0;
pub use crate::tu::ftlib::ftLib_80086BEC_fp as ftLib_80086BEC;
pub use crate::tu::ftlib::ftLib_80086EC0_fp as ftLib_80086EC0;
pub use crate::tu::ftlib::ftLib_8008701C_fp as ftLib_8008701C;
pub use crate::tu::ftlib::ftLib_800872B0_fp as ftLib_800872B0;
pub use crate::tu::ftlib::ftLib_8008731C;
pub use crate::tu::ftlib::ftLib_8008732C_fp as ftLib_8008732C;
pub use crate::tu::ftlib::ftLib_800873F4;
pub use crate::tu::ftlib::ftLib_800874BC_fp as ftLib_800874BC;
