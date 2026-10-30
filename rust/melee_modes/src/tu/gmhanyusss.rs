//! src/melee/gm/gmhanyusss.c: the Hanyu SSS mode's enter callback. Its state table
//! (`gm_Mode_HanyuSss_States`) is not ported in this pass.
//! Verification: none yet (state oracle not written; the callee reads the gmMainLib record).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use melee_types::*;

use crate::tu::gmmain_lib::gmMainLib_804D3EE0;

/// C `gm_801BEE58`: hands the Melee VS record to the SSS enter data.
pub unsafe fn gm_801BEE58(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801BEE58");
    let var_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut SSSData;
    (*var_r3).vs = (*gmMainLib_804D3EE0).modes.table[GmVsMode_Melee as usize];
}

/// Callees in TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut GM_GETGAMEMODESTATEENTERDATA: Option<fn(*mut melee_types::GameModeState) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetGameModeStateEnterData(s: *mut melee_types::GameModeState) -> *mut c_void {
                match unsafe { GM_GETGAMEMODESTATEENTERDATA } {
                    Some(f) => unsafe { f(s) },
                    None => { return unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(s as _) as _ }; },
                }
            }
}
