//! src/melee/gm/gmlightning.c: Lightning VS mode callbacks (game speed 1.25x).
//! Verification: none yet (wrappers over unported gmVsMelee_* callees). The
//! `gm_Mode_LightningVs_States` table is not ported in this pass.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ptr::addr_of_mut;
use melee_types::*;

use crate::tu::gmmain_lib::gmMainLib_804D3EE0;

const gmVsMode_State_Css: i32 = 0;
const gmVsMode_State_Sss: i32 = 1;
const gmVsMode_State_Vs: i32 = 2;
const gmVsMode_State_SuddenDeath: i32 = 3;
const gmVsMode_State_Results: i32 = 4;
const gmVsMode_State_Approach: i32 = 2 << 6;
const gmVsMode_State_ApproachVs: i32 = 129;
const gmVsMode_State_Prize: i32 = 3 << 6;


/// C `gm_801BA704` (CSS enter; TARGET_PC Atlas select for the CSS).
pub unsafe fn gm_801BA704(scene: *mut GameModeState) {
    ext::gmVsMelee_EnterCss(scene, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]), 9);
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// C `gm_801BA730` (CSS exit).
pub unsafe fn gm_801BA730(scene: *mut GameModeState) {
    ext::gmVsMelee_ExitCss(scene, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]));
}

/// C `gm_801BA758` (SSS enter; TARGET_PC Atlas select for the SSS).
pub unsafe fn gm_801BA758(scene: *mut GameModeState) {
    ext::gmVsMelee_EnterSss(scene, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]));
    ext::gmFrontend_AtlasSelect(scene, 1, core::ptr::null());
}

/// C `gm_801BA780` (SSS exit).
pub unsafe fn gm_801BA780(scene: *mut GameModeState) {
    ext::gmVsMelee_ExitSss(
        scene,
        addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]),
        gmVsMode_State_Css as _,
    );
}

/// C `fn_801BA7AC`: sets game speed to 1.25 for Lightning Melee.
pub unsafe fn fn_801BA7AC(start: *mut StartMeleeData, _vs: *mut StartMeleeData) {
    (*start).rules.game_speed = 1.25;
}

/// C `gm_801BA7B8` (VS enter).
pub unsafe fn gm_801BA7B8(scene: *mut GameModeState) {
    let data = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]);
    ext::gmVsMelee_EnterVs(scene, data, Some(fn_801BA7AC), None);
}

/// C `gm_801BA7EC` (VS exit).
pub unsafe fn gm_801BA7EC(scene: *mut GameModeState) {
    ext::gmVsMelee_ExitVs(scene, 4, 3);
}

/// C `gm_801BA814` (sudden death enter).
pub unsafe fn gm_801BA814(scene: *mut GameModeState) {
    let data = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]);
    ext::gmVsMelee_EnterSuddenDeath(scene, data, Some(fn_801BA7AC), None);
}

/// C `gm_801BA848` (sudden death exit).
pub unsafe fn gm_801BA848(scene: *mut GameModeState) {
    ext::gmVsMelee_ExitSuddenDeath(scene);
}

/// C `gm_801BA868` (results enter).
pub unsafe fn gm_801BA868(scene: *mut GameModeState) {
    ext::gmVsMelee_EnterResults(scene);
}

/// C `gm_801BA888` (results exit).
pub unsafe fn gm_801BA888(scene: *mut GameModeState) {
    ext::gmVsMelee_ExitResults(scene, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]), 0);
}

/// C `gm_Mode_LightningVs_OnInit`.
pub unsafe fn gm_Mode_LightningVs_OnInit() {
    ext::gm_InitVsMode(addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Lightning as usize]));
}

/// C `gm_Mode_LightningVs_OnLoad`.
pub unsafe fn gm_Mode_LightningVs_OnLoad() {
    ext::gmVsMelee_ResetKOCounts();
}

/// Callees in TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<unsafe fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut GMVSMELEE_ENTERCSS: Option<unsafe fn(*mut GameModeState, *mut VsModeData, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_EnterCss(s: *mut GameModeState, vs: *mut VsModeData, sel: i32)  {
                match unsafe { GMVSMELEE_ENTERCSS } {
                    Some(f) => unsafe { f(s, vs, sel) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_EnterCss(s as _, vs as _, sel as _); } return; },
                }
            }
    pub static mut GMVSMELEE_EXITCSS: Option<unsafe fn(*mut GameModeState, *mut VsModeData) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ExitCss(s: *mut GameModeState, vs: *mut VsModeData)  {
                match unsafe { GMVSMELEE_EXITCSS } {
                    Some(f) => unsafe { f(s, vs) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitCss(s as _, vs as _); } return; },
                }
            }
    pub static mut GMVSMELEE_ENTERSSS: Option<unsafe fn(*mut GameModeState, *mut VsModeData) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_EnterSss(s: *mut GameModeState, vs: *mut VsModeData)  {
                match unsafe { GMVSMELEE_ENTERSSS } {
                    Some(f) => unsafe { f(s, vs) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_EnterSss(s as _, vs as _); } return; },
                }
            }
    pub static mut GMVSMELEE_EXITSSS: Option<unsafe fn(*mut GameModeState, *mut VsModeData, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ExitSss(s: *mut GameModeState, vs: *mut VsModeData, next: i32)  {
                match unsafe { GMVSMELEE_EXITSSS } {
                    Some(f) => unsafe { f(s, vs, next) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitSss(s as _, vs as _, next as _); } return; },
                }
            }
    hook!(GMVSMELEE_ENTERVS, gmVsMelee_EnterVs, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    pub static mut GMVSMELEE_EXITVS: Option<unsafe fn(*mut GameModeState, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ExitVs(s: *mut GameModeState, a: i32, b: i32)  {
                match unsafe { GMVSMELEE_EXITVS } {
                    Some(f) => unsafe { f(s, a, b) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitVs(s as _, a as _, b as _); } return; },
                }
            }
    hook!(GMVSMELEE_ENTERSUDDENDEATH, gmVsMelee_EnterSuddenDeath, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    pub static mut GMVSMELEE_EXITSUDDENDEATH: Option<fn(*mut GameModeState)> = None;
    #[allow(non_snake_case)]
    pub fn gmVsMelee_ExitSuddenDeath(s: *mut GameModeState) {
        match unsafe { GMVSMELEE_EXITSUDDENDEATH } {
            Some(f) => f(s),
            None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitSuddenDeath(s as _); } },
        }
    }
    pub static mut GMVSMELEE_ENTERRESULTS: Option<unsafe fn(*mut GameModeState) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_EnterResults(s: *mut GameModeState)  {
                match unsafe { GMVSMELEE_ENTERRESULTS } {
                    Some(f) => unsafe { f(s) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_EnterResults(s as _); } return; },
                }
            }
    pub static mut GMVSMELEE_EXITRESULTS: Option<unsafe fn(*mut GameModeState, *mut VsModeData, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ExitResults(s: *mut GameModeState, vs: *mut VsModeData, next: i32)  {
                match unsafe { GMVSMELEE_EXITRESULTS } {
                    Some(f) => unsafe { f(s, vs, next) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitResults(s as _, vs as _, next as _); } return; },
                }
            }
    pub static mut GMVSMELEE_RESETKOCOUNTS: Option<unsafe fn() > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ResetKOCounts()  {
                match unsafe { GMVSMELEE_RESETKOCOUNTS } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ResetKOCounts(); } return; },
                }
            }
    pub static mut GMFRONTEND_ATLASSELECT: Option<unsafe fn(*mut GameModeState, i32, *const i8) > = None;
            #[allow(non_snake_case)]
            pub fn gmFrontend_AtlasSelect(s: *mut GameModeState, sss: i32, name: *const i8)  {
                match unsafe { GMFRONTEND_ATLASSELECT } {
                    Some(f) => unsafe { f(s, sss, name) },
                    None => { unsafe { let _ = crate::tu::gmfrontend::gmFrontend_AtlasSelect(s as _, sss as _, name as _); } return; },
                }
            }
    pub static mut GM_INITVSMODE: Option<unsafe fn(*mut VsModeData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_InitVsMode(vs: *mut VsModeData)  {
                match unsafe { GM_INITVSMODE } {
                    Some(f) => unsafe { f(vs) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_InitVsMode(vs as _); } return; },
                }
            }
}
