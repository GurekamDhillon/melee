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
    hook!(GOBJ_CREATE, GObj_Create, (a: u32, b: u32, c: u32) -> *mut HSD_GObj);
    pub static mut GM_801A9DD0: Option<fn(*mut HSD_GObj, i32, i32, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A9DD0(gobj: *mut HSD_GObj, a: i32, b: i32, c: i32, d: i32)  {
                match unsafe { GM_801A9DD0 } {
                    Some(f) => unsafe { f(gobj, a, b, c, d) },
                    None => { unsafe { let _ = crate::tu::gmopening::gm_801A9DD0(gobj as _, a as _, b as _, c as _, d as _); } return; },
                }
            }
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, sobj: *mut c_void, p: *mut c_void));
    hook!(HSD_SOBJLIB_804D7960, HSD_SObjLib_804D7960, () -> *mut c_void);
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, cb: fn(*mut HSD_GObj), prio: i32, a: i32));
    hook!(LBMTHP_8001F67C, lbMthp_8001F67C, (gobj: *mut HSD_GObj));
    pub static mut LBMTHP_8001F624: Option<fn(*mut HSD_GObj, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbMthp_8001F624(gobj: *mut HSD_GObj, a: i32, b: i32)  {
                match unsafe { LBMTHP_8001F624 } {
                    Some(f) => unsafe { f(gobj, a, b) },
                    None => { unsafe { let _ = melee_runtime::tu::lbmthp::lbMthp_8001F624(gobj as _, a as _, b as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_80023F28: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80023F28(a: i32)  {
                match unsafe { LBAUDIOAX_80023F28 } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_80024E50: Option<fn(i32)> = None;
    #[allow(non_snake_case)]
    pub fn lbAudioAx_80024E50(a: i32) {
        match unsafe { LBAUDIOAX_80024E50 } {
            Some(f) => f(a),
            None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80024E50((a != 0)); } },
        }
    }
    hook!(LBMTHP_8001F410, lbMthp_8001F410, (path: &str, table: *mut u32, a: i32, b: i32, c: i32));
    pub static mut LBMTHP_8001F578: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lbMthp_8001F578() {
        match unsafe { LBMTHP_8001F578 } {
            Some(f) => f(),
            None => { unsafe { let _ = melee_runtime::tu::lbmthp::lbMthp_8001F578(); } },
        }
    }
    pub static mut LBMTHP_8001F5C4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbMthp_8001F5C4()  {
                match unsafe { LBMTHP_8001F5C4 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbmthp::lbMthp_8001F5C4(); } return; },
                }
            }
    pub static mut LBMTHP_8001F604: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbMthp_8001F604() -> i32 {
                match unsafe { LBMTHP_8001F604 } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { melee_runtime::tu::lbmthp::lbMthp_8001F604() as _ }; },
                }
            }
    pub static mut LBAUDIOAX_800236DC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_800236DC()  {
                match unsafe { LBAUDIOAX_800236DC } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC(); } return; },
                }
            }
    pub static mut LBAUDIOAX_80023694: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80023694()  {
                match unsafe { LBAUDIOAX_80023694 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694(); } return; },
                }
            }
    pub static mut GM_801A4B74: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4B74()  {
                match unsafe { GM_801A4B74 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gmscene::gm_801A4B74(); } return; },
                }
            }
    pub static mut GM_801A4B60: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4B60()  {
                match unsafe { GM_801A4B60 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gmscene::gm_801A4B60(); } return; },
                }
            }
    pub static mut GM_GETBUTTONSTRIGGERED: Option<fn(i32) -> u32> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetButtonsTriggered(port: i32) -> u32 {
                match unsafe { GM_GETBUTTONSTRIGGERED } {
                    Some(f) => unsafe { f(port) },
                    None => { return unsafe { crate::tu::gm_1A36::gm_GetButtonsTriggered(port as _) as _ }; },
                }
            }
    pub static mut SFXFORWARD: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn sfxForward()  {
                match unsafe { SFXFORWARD } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::hsd::sfxForward(); } return; },
                }
            }
    pub static mut GM_SETPENDINGGAMEMODE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetPendingGameMode(mode: i32)  {
                match unsafe { GM_SETPENDINGGAMEMODE } {
                    Some(f) => unsafe { f(mode) },
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_SetPendingGameMode(mode as _); } return; },
                }
            }
    pub static mut GM_SETNEWGAMEMODEPENDING: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetNewGameModePending()  {
                match unsafe { GM_SETNEWGAMEMODEPENDING } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_SetNewGameModePending(); } return; },
                }
            }
    pub static mut GM_SETNEXTGAMEMODESTATEID: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetNextGameModeStateId(id: i32)  {
                match unsafe { GM_SETNEXTGAMEMODESTATEID } {
                    Some(f) => unsafe { f(id) },
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_SetNextGameModeStateId(id as _); } return; },
                }
            }
    pub static mut FN_801AA0E8: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn fn_801AA0E8()  {
                match unsafe { FN_801AA0E8 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gmopening::fn_801AA0E8(); } return; },
                }
            }
    hook!(SET_GM_804D6724, set_gm_804D6724, (f: fn()));
}
