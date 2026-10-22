//! src/melee/gm/gmhowto.c: the How To movie scene (MvHowto.mth) and its per-frame skip logic.
//! Verification: none yet. `gm_801ACC94` returns the table address (its contents are the data
//! the C passes to `lbMthp_8001F410`); the scene callbacks drive unported lb/gm code.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::addr_of_mut;
use melee_types::*;

use crate::tu::gmmain_lib as gml;

const PAD_MAX_CONTROLLERS: i32 = 4;

/// C `static u32 gm_803DD2C0[]`: the MvHowto.mth movie's table (74 words, copied from gmhowto.c).
pub static mut gm_803DD2C0: [u32; 74] = [
    1, 19, 856, 1, 1, 85, 279, 1, 1, 59, 17, 1, 1, 59, 19,
    1, 1, 59, 35, 1, 1, 27, 37, 1, 1, 59, 43, 1, 1, 59,
    39, 1, 1, 59, 499, 1, 1, 67, 892, 1, 240, 2, 63, 1, 1,
    77, 67, 1, 1, 67, 23, 1, 1, 71, 51, 1, 1, 59, 25, 1,
    1, 87, 53, 1, 1, 95, 59, 1, 1, 101, 117, 1, 65536, 1,
];

/// C `static HSD_GObj* gm_804D6850;` the movie's player object.
static mut gm_804D6850: *mut HSD_GObj = core::ptr::null_mut();

/// C `gm_801ACC94`: the address of the movie table.
pub unsafe fn gm_801ACC94() -> *mut c_void {
    addr_of_mut!(gm_803DD2C0) as *mut c_void
}

/// C `gm_Scene_HowTo_OnEnter`.
pub unsafe fn gm_Scene_HowTo_OnEnter(_unused: *mut c_void) {
    let mut gobj = ext::GObj_Create(0x13, 0x14, 0);
    ext::gm_801A9DD0(gobj, 0x280, 0x1E0, 8, 0);
    (*gobj).gxlink_prios = 1 << 0xB;

    gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D6850 = gobj;
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_SObjLib_804D7960(), core::ptr::null_mut());
    ext::GObj_SetupGXLink(gobj, ext::lbMthp_8001F67C, 0xB, 0);
    ext::lbMthp_8001F624(gobj, 0x280, 0x1E0);
    ext::lbAudioAx_80023F28(0x24);
    ext::lbAudioAx_80024E50(1);
    ext::lbMthp_8001F410("MvHowto.mth", addr_of_mut!(gm_803DD2C0) as *mut u32, 0, 0, 0);
    ext::lbAudioAx_80024E50(0);
    ext::set_gm_804D6724(ext::fn_801AA0E8);
}

/// C `gm_Scene_HowTo_OnFrame`.
pub unsafe fn gm_Scene_HowTo_OnFrame() {
    let mut var_r31 = false;
    ext::lbMthp_8001F578();
    ext::lbMthp_8001F5C4();
    if gml::gmMainLib_8046B0F0.xC != 0 && ext::lbMthp_8001F604() == 0 {
        var_r31 = true;
    }
    if ext::lbMthp_8001F604() != 0
        || (ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as i32) & (HSD_PAD_START | HSD_PAD_A) as u32) != 0
        || var_r31
    {
        ext::lbAudioAx_800236DC();
        ext::lbAudioAx_80023694();
        if var_r31 {
            ext::gm_801A4B74();
        } else {
            ext::gm_801A4B60();
        }
        if (ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS as i32) & (HSD_PAD_START | HSD_PAD_A) as u32) != 0 {
            ext::sfxForward();
            ext::gm_SetPendingGameMode(GM_TITLE as i32);
            ext::gm_SetNewGameModePending();
        } else if gml::gmMainLib_8015DB00() != 5 {
            gml::gmMainLib_8015DB18();
            ext::gm_SetNextGameModeStateId(0);
        }
    }
}

/// Callees in TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
    use melee_types::*;
    macro_rules! hook {
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
    hook!(GOBJ_CREATE, GObj_Create, (a: u32, b: u32, c: u32) -> *mut HSD_GObj);
    hook!(GM_801A9DD0, gm_801A9DD0, (gobj: *mut HSD_GObj, a: i32, b: i32, c: i32, d: i32));
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, sobj: *mut c_void, p: *mut c_void));
    hook!(HSD_SOBJLIB_804D7960, HSD_SObjLib_804D7960, () -> *mut c_void);
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj), prio: i32, a: i32));
    hook!(LBMTHP_8001F67C, lbMthp_8001F67C, (gobj: *mut HSD_GObj));
    hook!(LBMTHP_8001F624, lbMthp_8001F624, (gobj: *mut HSD_GObj, a: i32, b: i32));
    hook!(LBAUDIOAX_80023F28, lbAudioAx_80023F28, (a: i32));
    hook!(LBAUDIOAX_80024E50, lbAudioAx_80024E50, (a: i32));
    hook!(LBMTHP_8001F410, lbMthp_8001F410, (path: &str, table: *mut u32, a: i32, b: i32, c: i32));
    hook!(LBMTHP_8001F578, lbMthp_8001F578, ());
    hook!(LBMTHP_8001F5C4, lbMthp_8001F5C4, ());
    hook!(LBMTHP_8001F604, lbMthp_8001F604, () -> i32);
    hook!(LBAUDIOAX_800236DC, lbAudioAx_800236DC, ());
    hook!(LBAUDIOAX_80023694, lbAudioAx_80023694, ());
    hook!(GM_801A4B74, gm_801A4B74, ());
    hook!(GM_801A4B60, gm_801A4B60, ());
    hook!(GM_GETBUTTONSTRIGGERED, gm_GetButtonsTriggered, (port: i32) -> u32);
    hook!(SFXFORWARD, sfxForward, ());
    hook!(GM_SETPENDINGGAMEMODE, gm_SetPendingGameMode, (mode: i32));
    hook!(GM_SETNEWGAMEMODEPENDING, gm_SetNewGameModePending, ());
    hook!(GM_SETNEXTGAMEMODESTATEID, gm_SetNextGameModeStateId, (id: i32));
    hook!(FN_801AA0E8, fn_801AA0E8, ());
    hook!(SET_GM_804D6724, set_gm_804D6724, (f: fn()));
}
