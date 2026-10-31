//! Port of src/melee/gm/gmfixedcamera.c (Fixed Camera VS: the scene table and its CSS / SSS / VS /
//! sudden-death / results handlers over the mode's `VsModeData` row). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_gmfixedcamera.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use super::support::*;
use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn gmFrontend_AtlasSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_AtlasSelect__26b7fc;
    fn gmVsMelee_EnterCss(a0: *mut GameModeState, a1: *mut VsModeData, a2: u32) = crate::tu::adapt::gmVsMelee_EnterCss__44afc7;
    fn gmVsMelee_EnterResults(a0: *mut GameModeState) = crate::tu::adapt::gmVsMelee_EnterResults__e8d931;
    fn gmVsMelee_EnterSss(a0: *mut GameModeState, a1: *mut VsModeData) = crate::tu::adapt::gmVsMelee_EnterSss__1e7fb2;
    fn gmVsMelee_EnterSuddenDeath(a0: *mut GameModeState, a1: *mut VsModeData, a2: Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>, a3: Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>) = crate::tu::adapt::gmVsMelee_EnterSuddenDeath__883e46;
    fn gmVsMelee_EnterVs(a0: *mut GameModeState, a1: *mut VsModeData, a2: Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>, a3: Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>) = crate::tu::adapt::gmVsMelee_EnterVs__41232b;
    fn gmVsMelee_ExitCss(a0: *mut GameModeState, a1: *mut VsModeData) = crate::tu::adapt::gmVsMelee_ExitCss__4d2768;
    fn gmVsMelee_ExitResults(a0: *mut GameModeState, a1: *mut VsModeData, a2: u8) = crate::tu::adapt::gmVsMelee_ExitResults__c5c250;
    fn gmVsMelee_ExitSss(a0: *mut GameModeState, a1: *mut VsModeData, a2: u8) = crate::tu::adapt::gmVsMelee_ExitSss__2855c9;
    fn gmVsMelee_ExitSuddenDeath(a0: *mut GameModeState) = crate::tu::adapt::gmVsMelee_ExitSuddenDeath__a6d8ba;
    fn gmVsMelee_ExitVs(a0: *mut GameModeState, a1: u8, a2: u8) = crate::tu::adapt::gmVsMelee_ExitVs__083a0e;
    fn gmVsMelee_ResetKOCounts() = crate::tu::adapt::gmVsMelee_ResetKOCounts__75dda2;
    fn gm_80165290(a0: i32) = crate::tu::adapt::gm_80165290__e2462d;
    fn gm_InitVsMode(a0: *mut VsModeData) = crate::tu::adapt::gm_InitVsMode__c4ead2;
    fn gm_ModeState_ApproachVs_OnEnter(a0: *mut GameModeState) = crate::tu::adapt::gm_ModeState_ApproachVs_OnEnter__73c56c;
    fn gm_ModeState_ApproachVs_OnExit(a0: *mut GameModeState) = crate::tu::adapt::gm_ModeState_ApproachVs_OnExit__ca1d07;
    fn gm_ModeState_Approach_OnEnter(a0: *mut GameModeState) = crate::tu::adapt::gm_ModeState_Approach_OnEnter__044765;
    fn gm_ModeState_Prize_OnEnter(a0: *mut GameModeState) = crate::tu::adapt::gm_ModeState_Prize_OnEnter__8623b9;
    fn gm_ModeState_Prize_OnExit(a0: *mut GameModeState) = crate::tu::adapt::gm_ModeState_Prize_OnExit__253ddf;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn gm_80165290(a0: i32) { super::gm_80165290(a0) }
        pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_ApproachVs_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnExit(a0: *mut GameModeState) { super::gm_ModeState_ApproachVs_OnExit(a0) }
        pub unsafe extern "C" fn gm_ModeState_Approach_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_Approach_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_Prize_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_Prize_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_Prize_OnExit(a0: *mut GameModeState) { super::gm_ModeState_Prize_OnExit(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_world::gmmain::gmMainLib_804D3EE0;
    pub static mut gmVsMelee_ApproachData: VsApproachData = unsafe { core::mem::zeroed() };
    pub use crate::tu::gmvsmelee::gmVsMelee_CssData;
    pub use crate::tu::gmvsmelee::gmVsMelee_ResultsEnterData;
    pub use crate::tu::gmvsmelee::gmVsMelee_SssData;
    pub use crate::tu::gmvsmelee::gmVsMelee_StartData;
    pub use crate::tu::gmvsmelee::gmVsMelee_SuddenDeathExitInfo;
    pub use crate::tu::gmvsmelee::gmVsMelee_VsExitInfo;
    pub use melee_world::tu::if_2FD9::if_Scene_Prize_EnterData;
}
// END GENERATED EXT

#[inline(always)]
unsafe fn row() -> *mut VsModeData {
    &raw mut (*ext::gmMainLib_804D3EE0).modes.table[GmVsMode_FixedCamera as usize]
}

/// `gm_Mode_CameraVs_States`
pub static mut gm_Mode_CameraVs_States: [GameModeState; 9] = unsafe {
    [
        state(0, 3, Some(gm_801B9F10), Some(gm_801B9F3C), GS_CSS,
              &raw mut ext::gmVsMelee_CssData as *mut c_void, &raw mut ext::gmVsMelee_CssData as *mut c_void),
        state(1, 3, Some(gm_801B9F64), Some(gm_801B9F8C), GS_SSS,
              &raw mut ext::gmVsMelee_SssData as *mut c_void, &raw mut ext::gmVsMelee_SssData as *mut c_void),
        state(2, 3, Some(gm_801B9FC8), Some(gm_801B9FFC), GS_VS,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_VsExitInfo as *mut c_void),
        state(3, 3, Some(gm_801BA024), Some(gm_801BA058), GS_SUDDEN_DEATH,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_SuddenDeathExitInfo as *mut c_void),
        state(4, 3, Some(gm_801BA078), Some(gm_801BA098), GS_RESULTS,
              &raw mut ext::gmVsMelee_ResultsEnterData as *mut c_void, NULL),
        state(0x80, 2, Some(ext::cb::gm_ModeState_Approach_OnEnter), None, GS_APPROACH,
              &raw mut ext::gmVsMelee_ApproachData as *mut c_void, &raw mut ext::gmVsMelee_ApproachData as *mut c_void),
        state(0x81, 2, Some(ext::cb::gm_ModeState_ApproachVs_OnEnter), Some(ext::cb::gm_ModeState_ApproachVs_OnExit), GS_VS,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_VsExitInfo as *mut c_void),
        state(0xC0, 2, Some(ext::cb::gm_ModeState_Prize_OnEnter), Some(ext::cb::gm_ModeState_Prize_OnExit), GS_PRIZE_INTERFACE,
              &raw mut ext::if_Scene_Prize_EnterData as *mut c_void, NULL),
        end(),
    ]
};

/// 1B9F10
pub unsafe extern "C" fn gm_801B9F10(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9F10");
    ext::gmVsMelee_EnterCss(scene, row(), 7);
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// 1B9F3C
pub unsafe extern "C" fn gm_801B9F3C(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9F3C");
    ext::gmVsMelee_ExitCss(scene, row());
}

/// 1B9F64
pub unsafe extern "C" fn gm_801B9F64(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9F64");
    ext::gmVsMelee_EnterSss(scene, row());
    ext::gmFrontend_AtlasSelect(scene, 1, core::ptr::null());
}

/// 1B9F8C
pub unsafe extern "C" fn gm_801B9F8C(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9F8C");
    ext::gmVsMelee_ExitSss(scene, row(), 0);
}

/// 1B9FB8: the per-match setup callback (`StartMeleeData*, StartMeleeData*`).
pub unsafe extern "C" fn fn_801B9FB8(start: *mut StartMeleeData, _vs: *mut StartMeleeData) {
    melee_core::ct!("fn_801B9FB8");
    (*start).rules.on_unpause_override = Some(ext::cb::gm_80165290);
}

/// 1B9FC8
pub unsafe extern "C" fn gm_801B9FC8(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9FC8");
    let data = row();
    ext::gmVsMelee_EnterVs(scene, data, Some(fn_801B9FB8), None);
}

/// 1B9FFC
pub unsafe extern "C" fn gm_801B9FFC(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9FFC");
    ext::gmVsMelee_ExitVs(scene, 4, 3);
}

/// 1BA024
pub unsafe extern "C" fn gm_801BA024(scene: *mut GameModeState) {
    melee_core::ct!("gm_801BA024");
    let data = row();
    ext::gmVsMelee_EnterSuddenDeath(scene, data, Some(fn_801B9FB8), None);
}

/// 1BA058
pub unsafe extern "C" fn gm_801BA058(scene: *mut GameModeState) {
    melee_core::ct!("gm_801BA058");
    ext::gmVsMelee_ExitSuddenDeath(scene);
}

/// 1BA078
pub unsafe extern "C" fn gm_801BA078(scene: *mut GameModeState) {
    melee_core::ct!("gm_801BA078");
    ext::gmVsMelee_EnterResults(scene);
}

/// 1BA098
pub unsafe extern "C" fn gm_801BA098(scene: *mut GameModeState) {
    melee_core::ct!("gm_801BA098");
    ext::gmVsMelee_ExitResults(scene, row(), 0);
}

/// `gm_Mode_CameraVs_OnInit`
pub fn gm_Mode_CameraVs_OnInit() {
    melee_core::ct!("gm_Mode_CameraVs_OnInit");
    unsafe { ext::gm_InitVsMode(row()) }
}

/// `gm_Mode_CameraVs_OnLoad`
pub fn gm_Mode_CameraVs_OnLoad() {
    melee_core::ct!("gm_Mode_CameraVs_OnLoad");
    ext::gmVsMelee_ResetKOCounts();
}
