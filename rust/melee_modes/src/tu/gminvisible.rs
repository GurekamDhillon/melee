//! src/melee/gm/gminvisible.c: Invisible VS mode callbacks (CSS, SSS, VS, sudden death, results).
//! Verification: none yet (thin wrappers over unported gmVsMelee_* callees; the callee-call
//! trace oracle is not written). The `gm_Mode_InvisibleVs_States` table is not ported in this
//! pass (it names the unported gmVsMelee data and GameModeState arrays).
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


/// C `onEnterCss`.
pub unsafe fn onEnterCss(state: *mut GameModeState) {
    let vs = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]);
    ext::gmVsMelee_EnterCss(state, vs, VS_INVISIBLE as _);
    // TARGET_PC: the Atlas select, when it can run; else the retail screen.
    ext::gmFrontend_AtlasSelect(state, 0, core::ptr::null());
}

/// C `onExitCss`.
pub unsafe fn onExitCss(state: *mut GameModeState) {
    ext::gmVsMelee_ExitCss(state, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]));
}

/// C `onEnterSss`.
pub unsafe fn onEnterSss(state: *mut GameModeState) {
    ext::gmVsMelee_EnterSss(state, addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]));
    ext::gmFrontend_AtlasSelect(state, 1, core::ptr::null());
}

/// C `onExitSss`.
pub unsafe fn onExitSss(state: *mut GameModeState) {
    ext::gmVsMelee_ExitSss(
        state,
        addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]),
        gmVsMode_State_Css as _,
    );
}

/// C `initVsPlayer`: the invisible-mode flag on every player start (the `vs` argument is unused).
pub unsafe fn initVsPlayer(start: *mut PlayerInitData, _unused: *mut PlayerInitData) {
    (*start).set_vs_invisible(1);
}

/// C `onEnterVs`.
pub unsafe fn onEnterVs(state: *mut GameModeState) {
    let vs = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]);
    ext::gmVsMelee_EnterVs(state, vs, None, Some(initVsPlayer));
}

/// C `onExitVs`.
pub unsafe fn onExitVs(state: *mut GameModeState) {
    ext::gmVsMelee_ExitVs(state, gmVsMode_State_Results as _, gmVsMode_State_SuddenDeath as _);
}

/// C `onEnterSuddenDeath`.
pub unsafe fn onEnterSuddenDeath(state: *mut GameModeState) {
    let vs = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]);
    ext::gmVsMelee_EnterSuddenDeath(state, vs, None, Some(initVsPlayer));
}

/// C `onExitSuddenDeath`.
pub unsafe fn onExitSuddenDeath(state: *mut GameModeState) {
    ext::gmVsMelee_ExitSuddenDeath(state);
}

/// C `onEnterResults`.
pub unsafe fn onEnterResults(state: *mut GameModeState) {
    ext::gmVsMelee_EnterResults(state);
}

/// C `onExitResults`.
pub unsafe fn onExitResults(state: *mut GameModeState) {
    let vs = addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]);
    ext::gmVsMelee_ExitResults(state, vs, gmVsMode_State_Css as _);
}

/// C `gm_Mode_InvisibleVs_OnInit`.
pub unsafe fn gm_Mode_InvisibleVs_OnInit() {
    ext::gm_InitVsMode(addr_of_mut!((*gmMainLib_804D3EE0).modes.table[GmVsMode_Invisible as usize]));
}

/// C `gm_Mode_InvisibleVs_OnLoad`.
pub unsafe fn gm_Mode_InvisibleVs_OnLoad() {
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
    hook!(GMVSMELEE_ENTERVS, gmVsMelee_EnterVs, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_ENTERSUDDENDEATH, gmVsMelee_EnterSuddenDeath, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_EXITVS, gmVsMelee_ExitVs, (s: *mut GameModeState, a: i32, b: i32));
    pub static mut GMVSMELEE_EXITSUDDENDEATH: Option<unsafe fn(*mut GameModeState) > = None;
            #[allow(non_snake_case)]
            pub fn gmVsMelee_ExitSuddenDeath(s: *mut GameModeState)  {
                match unsafe { GMVSMELEE_EXITSUDDENDEATH } {
                    Some(f) => unsafe { f(s) },
                    None => { unsafe { let _ = crate::tu::gmvsmelee::gmVsMelee_ExitSuddenDeath(s as _); } return; },
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
