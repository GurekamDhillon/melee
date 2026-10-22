//! Game globals that several gm TUs share (defined in TUs not ported yet). Native pointers/values,
//! set by whoever owns the simulation (the oracle tests set them to synthetic state).
#![allow(non_upper_case_globals, static_mut_refs)]
use core::ptr::null_mut;
use melee_types::*;

/// gmmain_lib.c `/* 4D3EE0 */ struct gmm_x0* gmMainLib_804D3EE0` (the save data).
pub static mut gmMainLib_804D3EE0: *mut gmm_x0 = null_mut();
/// gm_1884.c `/* 473814 */ CssSubStruct gm_80473814`.
pub static mut gm_80473814: CssSubStruct = unsafe { core::mem::zeroed() };
