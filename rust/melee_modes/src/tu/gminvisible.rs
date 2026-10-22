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
    hook!(GMVSMELEE_ENTERCSS, gmVsMelee_EnterCss, (s: *mut GameModeState, vs: *mut VsModeData, sel: i32));
    hook!(GMVSMELEE_EXITCSS, gmVsMelee_ExitCss, (s: *mut GameModeState, vs: *mut VsModeData));
    hook!(GMVSMELEE_ENTERSSS, gmVsMelee_EnterSss, (s: *mut GameModeState, vs: *mut VsModeData));
    hook!(GMVSMELEE_EXITSSS, gmVsMelee_ExitSss, (s: *mut GameModeState, vs: *mut VsModeData, next: i32));
    hook!(GMVSMELEE_ENTERVS, gmVsMelee_EnterVs, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_ENTERSUDDENDEATH, gmVsMelee_EnterSuddenDeath, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_EXITVS, gmVsMelee_ExitVs, (s: *mut GameModeState, a: i32, b: i32));
    hook!(GMVSMELEE_EXITSUDDENDEATH, gmVsMelee_ExitSuddenDeath, (s: *mut GameModeState));
    hook!(GMVSMELEE_ENTERRESULTS, gmVsMelee_EnterResults, (s: *mut GameModeState));
    hook!(GMVSMELEE_EXITRESULTS, gmVsMelee_ExitResults, (s: *mut GameModeState, vs: *mut VsModeData, next: i32));
    hook!(GMVSMELEE_RESETKOCOUNTS, gmVsMelee_ResetKOCounts, ());
    hook!(GMFRONTEND_ATLASSELECT, gmFrontend_AtlasSelect, (s: *mut GameModeState, sss: i32, name: *const i8));
    hook!(GM_INITVSMODE, gm_InitVsMode, (vs: *mut VsModeData));
}
