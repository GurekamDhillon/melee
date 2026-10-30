//! Port of src/melee/gm/gmgiant.c (Giant Melee: the VS scene table and its CSS / SSS / VS /
//! sudden-death / results handlers, which hand the mode's `VsModeData` row to the shared
//! gmVsMelee_* routines, and the 1.8x player setup). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_gmgiant.rs.
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
        pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_ApproachVs_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnExit(a0: *mut GameModeState) { super::gm_ModeState_ApproachVs_OnExit(a0) }
        pub unsafe extern "C" fn gm_ModeState_Approach_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_Approach_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_Prize_OnEnter(a0: *mut GameModeState) { super::gm_ModeState_Prize_OnEnter(a0) }
        pub unsafe extern "C" fn gm_ModeState_Prize_OnExit(a0: *mut GameModeState) { super::gm_ModeState_Prize_OnExit(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut gmMainLib_804D3EE0: *mut gmm_x0 = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_ApproachData: VsApproachData = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_CssData: CSSData = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_ResultsEnterData: ResultsMatchInfo = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_SssData: SSSData = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_StartData: StartMeleeData = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_SuddenDeathExitInfo: MatchExitInfo = unsafe { core::mem::zeroed() };
    pub static mut gmVsMelee_VsExitInfo: MatchExitInfo = unsafe { core::mem::zeroed() };
    pub static mut if_Scene_Prize_EnterData: un_804A1F48_t = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline(always)]
unsafe fn giant_row() -> *mut VsModeData {
    &raw mut (*ext::gmMainLib_804D3EE0).modes.table[GmVsMode_Giant as usize]
}

/// `gm_Mode_GiantVs_States`
pub static mut gm_Mode_GiantVs_States: [GameModeState; 9] = unsafe {
    [
        state(0, 3, Some(gm_801B8FB8), Some(gm_801B8FE4), GS_CSS,
              &raw mut ext::gmVsMelee_CssData as *mut c_void, &raw mut ext::gmVsMelee_CssData as *mut c_void),
        state(1, 3, Some(gm_801B900C), Some(gm_801B9034), GS_SSS,
              &raw mut ext::gmVsMelee_SssData as *mut c_void, &raw mut ext::gmVsMelee_SssData as *mut c_void),
        state(2, 3, Some(gm_801B9084), Some(gm_801B90B8), GS_VS,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_VsExitInfo as *mut c_void),
        state(3, 3, Some(gm_801B90E0), Some(gm_801B9114), GS_SUDDEN_DEATH,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_SuddenDeathExitInfo as *mut c_void),
        state(4, 3, Some(gm_801B9134), Some(gm_801B9154), GS_RESULTS,
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

/// 1B8FB8
pub unsafe extern "C" fn gm_801B8FB8(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B8FB8");
    ext::gmVsMelee_EnterCss(scene, giant_row(), 4);
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// 1B8FE4
pub unsafe extern "C" fn gm_801B8FE4(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B8FE4");
    ext::gmVsMelee_ExitCss(scene, giant_row());
}

/// 1B900C
pub unsafe extern "C" fn gm_801B900C(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B900C");
    ext::gmVsMelee_EnterSss(scene, giant_row());
    ext::gmFrontend_AtlasSelect(scene, 1, core::ptr::null());
}

/// 1B9034
pub unsafe extern "C" fn gm_801B9034(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9034");
    ext::gmVsMelee_ExitSss(scene, giant_row(), 0);
}

/// 1B9060: C-ABI entry of the (round-3 verified) `gm_ptr::fn_801B9060`, the per-player setup
/// callback gmVsMelee_EnterVs / EnterSuddenDeath run.
pub unsafe extern "C" fn fn_801B9060(arg0: *mut PlayerInitData, unused: *mut PlayerInitData) {
    melee_core::ct!("fn_801B9060");
    crate::gm_ptr::fn_801B9060(&mut *arg0, unused)
}

/// 1B9084
pub unsafe extern "C" fn gm_801B9084(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9084");
    let data = giant_row();
    ext::gmVsMelee_EnterVs(scene, data, None, Some(fn_801B9060));
}

/// 1B90B8
pub unsafe extern "C" fn gm_801B90B8(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B90B8");
    ext::gmVsMelee_ExitVs(scene, 4, 3);
}

/// 1B90E0
pub unsafe extern "C" fn gm_801B90E0(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B90E0");
    let data = giant_row();
    ext::gmVsMelee_EnterSuddenDeath(scene, data, None, Some(fn_801B9060));
}

/// 1B9114
pub unsafe extern "C" fn gm_801B9114(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9114");
    ext::gmVsMelee_ExitSuddenDeath(scene);
}

/// 1B9134
pub unsafe extern "C" fn gm_801B9134(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9134");
    ext::gmVsMelee_EnterResults(scene);
}

/// 1B9154
pub unsafe extern "C" fn gm_801B9154(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B9154");
    ext::gmVsMelee_ExitResults(scene, giant_row(), 0);
}

/// 1B9174 (`gm_Mode_GiantVs_OnInit`)
pub fn gm_Mode_GiantVs_OnInit() {
    melee_core::ct!("gm_Mode_GiantVs_OnInit");
    unsafe { ext::gm_InitVsMode(giant_row()) }
}

/// `gm_Mode_GiantVs_OnLoad`
pub fn gm_Mode_GiantVs_OnLoad() {
    melee_core::ct!("gm_Mode_GiantVs_OnLoad");
    ext::gmVsMelee_ResetKOCounts();
}
