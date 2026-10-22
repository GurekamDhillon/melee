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
    hook!(GMVSMELEE_ENTERCSS, gmVsMelee_EnterCss, (s: *mut GameModeState, vs: *mut VsModeData, sel: i32));
    hook!(GMVSMELEE_EXITCSS, gmVsMelee_ExitCss, (s: *mut GameModeState, vs: *mut VsModeData));
    hook!(GMVSMELEE_ENTERSSS, gmVsMelee_EnterSss, (s: *mut GameModeState, vs: *mut VsModeData));
    hook!(GMVSMELEE_EXITSSS, gmVsMelee_ExitSss, (s: *mut GameModeState, vs: *mut VsModeData, next: i32));
    hook!(GMVSMELEE_ENTERVS, gmVsMelee_EnterVs, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_EXITVS, gmVsMelee_ExitVs, (s: *mut GameModeState, a: i32, b: i32));
    hook!(GMVSMELEE_ENTERSUDDENDEATH, gmVsMelee_EnterSuddenDeath, (s: *mut GameModeState, vs: *mut VsModeData, pre: Option<unsafe fn(*mut StartMeleeData, *mut StartMeleeData)>, per: Option<unsafe fn(*mut PlayerInitData, *mut PlayerInitData)>));
    hook!(GMVSMELEE_EXITSUDDENDEATH, gmVsMelee_ExitSuddenDeath, (s: *mut GameModeState));
    hook!(GMVSMELEE_ENTERRESULTS, gmVsMelee_EnterResults, (s: *mut GameModeState));
    hook!(GMVSMELEE_EXITRESULTS, gmVsMelee_ExitResults, (s: *mut GameModeState, vs: *mut VsModeData, next: i32));
    hook!(GMVSMELEE_RESETKOCOUNTS, gmVsMelee_ResetKOCounts, ());
    hook!(GMFRONTEND_ATLASSELECT, gmFrontend_AtlasSelect, (s: *mut GameModeState, sss: i32, name: *const i8));
    hook!(GM_INITVSMODE, gm_InitVsMode, (vs: *mut VsModeData));
}
