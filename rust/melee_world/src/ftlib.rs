
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

pub use crate::tu::ftlib::ftLib_GetPos_fp as ftLib_GetPos;
pub use crate::tu::ftlib::ftLib_GetPrevPos_fp as ftLib_GetPrevPos;
pub use crate::tu::ftlib::ftLib_IsFighter_fp as ftLib_IsFighter;
pub use crate::tu::ftlib::ftLib_GetCollData_fp as ftLib_GetCollData;
pub use crate::tu::ftlib::ftLib_80086A4C_fp as ftLib_80086A4C;
pub use crate::tu::ftlib::ftLib_IsOffscreen_fp as ftLib_IsOffscreen;
pub use crate::tu::ftlib::ftLib_GetCameraSubject_fp as ftLib_GetCameraSubject;
pub use crate::tu::ftlib::ftLib_80086B80_fp as ftLib_80086B80;
pub use crate::tu::ftlib::ftLib_GetCameraSubjectBonePos_fp as ftLib_GetCameraSubjectBonePos;
pub use crate::tu::ftlib::ftLib_GetPlayerIndex_fp as ftLib_GetPlayerIndex;
pub use crate::tu::ftlib::ftLib_GetPosDelta_fp as ftLib_GetPosDelta;
pub use crate::tu::ftlib::ftLib_IsInHitstun_fp as ftLib_IsInHitstun;
pub use crate::tu::ftlib::ftLib_IsSleeping_fp as ftLib_IsSleeping;
pub use crate::tu::ftlib::ftLib_GetShadow_fp as ftLib_GetShadow;
pub use crate::tu::ftlib::ftLib_IsSleeping_8008731C;
pub use crate::tu::ftlib::ftLib_IsDead_fp as ftLib_IsDead;
pub use crate::tu::ftlib::ftLib_IsEntry;
pub use crate::tu::ftlib::ftLib_IsSubFighter_fp as ftLib_IsSubFighter;
