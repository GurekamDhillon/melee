//! Port of src/melee/gm/gmhanyucss.c (the Hanyu character-select scene: copies the VS data in on
//! enter, and on exit steps `match_type` through the 24 CSS rounds and stores the VS data back).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_gmhanyucss.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use super::support::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn gmVsMelee_GetVsData() -> *mut VsModeData;
    fn gm_80164A0C(a0: u8);
    fn gm_80164F18();
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `static CSSData gm_8049BEE8` (49BEE8).
pub static mut gm_8049BEE8: CSSData = unsafe { core::mem::zeroed() };

/// `gm_Mode_HanyuCss_States`: one CSS scene, then the `{ -1 }` terminator.
pub static mut gm_Mode_HanyuCss_States: [GameModeState; 2] = [
    state(
        0,
        lbDvdPreload_2,
        Some(gm_801BED3C),
        Some(gm_801BEDA8),
        GS_CSS,
        unsafe { &raw mut gm_8049BEE8 as *mut c_void },
        NULL,
    ),
    end(),
];

/// 1BED3C
pub unsafe extern "C" fn gm_801BED3C(arg0: *mut GameModeState) {
    let temp_r31 = ext::gm_GetGameModeStateEnterData(arg0) as *mut CSSData;
    (*temp_r31).vs = *ext::gmVsMelee_GetVsData();
    ext::gm_80164F18();
    if (*temp_r31).match_type & 1 != 0 {
        ext::gm_80164A0C(7);
    }
}

/// 1BEDA8
pub unsafe extern "C" fn gm_801BEDA8(arg0: *mut GameModeState) {
    let css = ext::gm_GetGameModeStateEnterData(arg0) as *mut CSSData;
    let vs = ext::gmVsMelee_GetVsData();
    if (*css).pending_scene_change == 2 {
        if (*css).match_type != 0 {
            (*css).match_type -= 1;
        } else {
            (*css).match_type = 0x17;
        }
    } else {
        (*css).match_type = (((*css).match_type as i32 + 1) % 24) as u8;
    }
    *vs = (*css).vs;
}
