//! Port of src/melee/gm/gmcameramode.c (Camera Mode: the scene table, the DVD preload of the three
//! snapshot buffers, and the CSS / SSS / VS scene handlers over the mode's `VsModeData` row).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_gmcameramode.rs.
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
    fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32;
    fn gmCamera_801A3098() = crate::tu::adapt::gmCamera_801A3098__ec0ef3;
    fn gmCamera_801A30E4() = crate::tu::adapt::gmCamera_801A30E4__21b227;
    fn gmCamera_801A31FC() = crate::tu::adapt::gmCamera_801A31FC__fb65a6;
    fn gmFrontend_AtlasSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_AtlasSelect__26b7fc;
    fn gmVsMelee_GetKOCounts() -> *mut u8 = crate::tu::adapt::gmVsMelee_GetKOCounts__f55015;
    fn gmVsMelee_UpdateKOCounts(a0: *mut u8, a1: *mut MatchEnd);
    fn gm_8016247C(a0: i32) -> i32 = crate::tu::adapt::gm_8016247C__6fb2cc;
    fn gm_80165268(a0: i32) = crate::tu::adapt::gm_80165268__fb784b;
    fn gm_80167BC8(a0: *mut VsModeData) = crate::tu::adapt::gm_80167BC8__ea74bf;
    fn gm_80167FC4(a0: *mut SSSData) = crate::tu::adapt::gm_80167FC4__c413c9;
    fn gm_80168638(a0: *mut MatchEnd) = crate::tu::adapt::gm_80168638__516aa9;
    fn gm_80168710(a0: *mut MatchEnd, a1: *mut VsModeData) = crate::tu::adapt::gm_80168710__2df7b9;
    fn gm_801688AC(a0: *mut MatchEnd) -> i32 = crate::tu::adapt::gm_801688AC__8d2545;
    fn gm_CameraModeVSGetPauser() -> i32 = crate::tu::adapt::gm_CameraModeVSGetPauser__ac5b00;
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
    fn gm_InitVsMode(a0: *mut VsModeData) = crate::tu::adapt::gm_InitVsMode__c4ead2;
    fn gm_LoadAnnouncer() = crate::tu::adapt::gm_LoadAnnouncer__907f2e;
    fn gm_LoadRumbleEnabled(a0: *mut StartMeleeData) = crate::tu::adapt::gm_LoadRumbleEnabled__62a06a;
    fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
    fn gm_SetupSubColors(a0: *mut StartMeleeData) = crate::tu::adapt::gm_SetupSubColors__761278;
    fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026E84__5dcc27;
    fn lbAudioAx_80026EBC(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026EBC__431bc0;
    fn lbAudioAx_80026F2C(a0: u32) = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
    fn lbAudioAx_8002702C(a0: u32, a1: u64) = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
    fn lbAudioAx_80027168() = crate::tu::adapt::lbAudioAx_80027168__342616;
    fn lbCardNew_AllocWorkArea() = crate::tu::adapt::lbCardNew_AllocWorkArea__a2c203;
    fn lbDvd_80017740(a0: i32, a1: i32, a2: i32, a3: i32, a4: u32, a5: i32, a6: i32, a7: u8, a8: i32) -> *mut c_void = crate::tu::adapt::lbDvd_80017740__447a8d;
    fn lbDvd_80018254() = crate::tu::adapt::lbDvd_80018254__6be579;
    fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990;
    fn lbDvd_GetPreloadedArchive(a0: i32) -> *mut c_void = crate::tu::adapt::lbDvd_GetPreloadedArchive__a74d59;
    fn lbDvd_SetupVsPreloadCache() = crate::tu::adapt::lbDvd_SetupVsPreloadCache__d38cf3;
    fn lbSnap_8001E204() -> i32 = crate::tu::adapt::lbSnap_8001E204__c45919;
    fn lbSnap_8001E210() -> i32 = crate::tu::adapt::lbSnap_8001E210__8a0d85;
    fn lbSnap_8001E218(a0: *mut c_void, a1: *mut c_void /* Unk80433380_48 */) = crate::tu::adapt::lbSnap_8001E218__ac2c91;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn gmCamera_801A3098() { super::gmCamera_801A3098() }
        pub unsafe extern "C" fn gmCamera_801A30E4() { super::gmCamera_801A30E4() }
        pub unsafe extern "C" fn gmCamera_801A31FC() { super::gmCamera_801A31FC() }
        pub unsafe extern "C" fn gm_80165268(a0: i32) { super::gm_80165268(a0) }
        pub unsafe extern "C" fn gm_CameraModeVSGetPauser() -> i32 { super::gm_CameraModeVSGetPauser() }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_world::gmmain::gmMainLib_804D3EE0;
    pub use crate::tu::gmvsmelee::gmVsMelee_CssData;
    pub use crate::tu::gmvsmelee::gmVsMelee_SssData;
    pub use crate::tu::gmvsmelee::gmVsMelee_StartData;
    pub use crate::tu::gmvsmelee::gmVsMelee_VsExitInfo;
}
// END GENERATED EXT

#[inline(always)]
unsafe fn row() -> *mut VsModeData {
    &raw mut (*ext::gmMainLib_804D3EE0).modes.table[GmVsMode_Camera as usize]
}

/// `static s64 gm_804D68C8` (4D68C8): the scene table's dummy enter / exit data.
pub static mut gm_804D68C8: i64 = 0;

/// `gm_Mode_Camera_States`
pub static mut gm_Mode_Camera_States: [GameModeState; 5] = unsafe {
    [
        state(0, lbDvdPreload_3, Some(gm_801B24B4), Some(gm_801B2510), GS_CAMERA_VS,
              &raw mut gm_804D68C8 as *mut c_void, &raw mut gm_804D68C8 as *mut c_void),
        state(1, lbDvdPreload_3, Some(gm_801B254C), Some(gm_801B25D4), GS_CSS,
              &raw mut ext::gmVsMelee_CssData as *mut c_void, &raw mut ext::gmVsMelee_CssData as *mut c_void),
        state(2, lbDvdPreload_3, Some(gm_801B26AC), Some(gm_801B2704), GS_SSS,
              &raw mut ext::gmVsMelee_SssData as *mut c_void, &raw mut ext::gmVsMelee_SssData as *mut c_void),
        state(3, lbDvdPreload_3, Some(gm_PrepCameraModeVSScene), Some(gm_801B2AF8), GS_VS,
              &raw mut ext::gmVsMelee_StartData as *mut c_void, &raw mut ext::gmVsMelee_VsExitInfo as *mut c_void),
        end(),
    ]
};

/// 1B23F0
pub fn gm_801B23F0() {
    melee_core::ct!("gm_801B23F0");
    let size = ext::GXGetTexBufferSize(640, 480, 4, 0, 0);
    ext::lbDvd_80017740(0, 2006, 4, 4, (size.wrapping_add(31)) & !31u32, 0, 1, 32, 0);
    ext::lbDvd_80017740(0, 2007, 4, 4, ext::lbSnap_8001E204() as u32, 0, 1, 32, 0);
    ext::lbDvd_80017740(0, 2008, 4, 4, ext::lbSnap_8001E210() as u32, 0, 1, 32, 0);
}

/// 1B24B4
pub unsafe extern "C" fn gm_801B24B4(_arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B24B4");
    let temp_r31 = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    ext::lbDvd_SetupVsPreloadCache();
    (*temp_r31).mode_kind = GM_CAMERA_MODE as u8;
    ext::lbDvd_80018254();
    ext::lbCardNew_AllocWorkArea();
    let temp_r31_2 = ext::lbDvd_GetPreloadedArchive(0x7D8);
    ext::lbSnap_8001E218(ext::lbDvd_GetPreloadedArchive(0x7D7), temp_r31_2);
}

/// 1B2510
pub unsafe extern "C" fn gm_801B2510(state: *mut GameModeState) {
    melee_core::ct!("gm_801B2510");
    let data = ext::gm_GetGameModeStateExitData(state) as *mut i32;
    let temp_r0 = *data;
    if temp_r0 == 1 || temp_r0 == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
    }
}

/// 1B254C
pub unsafe extern "C" fn gm_801B254C(state: *mut GameModeState) {
    melee_core::ct!("gm_801B254C");
    let temp_r31 = row();
    let temp_r30 = ext::gm_GetGameModeStateEnterData(state) as *mut CSSData;
    let temp_r3 = ext::gmVsMelee_GetKOCounts();

    (*temp_r30).match_type = 1;
    (*temp_r30).ko_counts = temp_r3;

    (*temp_r30).vs = *temp_r31;

    let temp_r30_2 = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    ext::lbDvd_SetupVsPreloadCache();
    (*temp_r30_2).mode_kind = GM_CAMERA_MODE as u8;
    ext::lbDvd_80018254();
    ext::gmFrontend_AtlasSelect(state, 0, core::ptr::null());
}

/// 1B25D4
pub unsafe extern "C" fn gm_801B25D4(state: *mut GameModeState) {
    melee_core::ct!("gm_801B25D4");
    let temp_r31 = row();
    let temp_r3 = ext::gm_GetGameModeStateExitData(state) as *mut CSSData;
    if (*temp_r3).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }

    *temp_r31 = (*temp_r3).vs;

    let mut temp_ret: u64 = 0;
    for i in 0..6 {
        temp_ret |= ext::lbAudioAx_80026E84((*temp_r3).vs.start.players[i].ckind as u32);
    }

    ext::lbAudioAx_80026F2C(0x14);
    ext::lbAudioAx_8002702C(4, temp_ret);
    ext::lbAudioAx_80027168();
}

/// 1B26AC
pub unsafe extern "C" fn gm_801B26AC(state: *mut GameModeState) {
    melee_core::ct!("gm_801B26AC");
    let vs = row();
    let sss = ext::gm_GetGameModeStateEnterData(state) as *mut SSSData;
    (*sss).vs = *vs;
    ext::gm_80167FC4(sss);
    ext::gmFrontend_AtlasSelect(state, 1, core::ptr::null());
}

/// 1B2704
pub unsafe extern "C" fn gm_801B2704(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B2704");
    let temp_r31 = row();
    let var_r3 = ext::gm_GetGameModeStateExitData(arg0) as *mut SSSData;
    if (*var_r3).start_game != 0 {
        *temp_r31 = (*var_r3).vs;
        ext::lbAudioAx_80026F2C(0x18);
        ext::lbAudioAx_8002702C(8, ext::lbAudioAx_80026EBC((*temp_r31).start.rules.stkind as u32));
        ext::lbAudioAx_80027168();
        return;
    }
    ext::gm_SetNextGameModeStateId(1);
}

/// 1B2790
pub unsafe extern "C" fn gm_PrepCameraModeVSScene(state: *mut GameModeState) {
    melee_core::ct!("gm_PrepCameraModeVSScene");
    let vs = row();
    let start = ext::gm_GetGameModeStateEnterData(state) as *mut StartMeleeData;
    ext::gm_80167BC8(vs);

    (*start).rules = (*vs).start.rules;
    (*start).rules.set_match_kind(0);

    (*vs).start.rules.set_x4_4(0);
    (*start).rules.set_timer_enabled(0);

    (*start).rules.set_x5_0(1);
    (*start).rules.set_x1_2(1);
    (*start).rules.set_x1_3(1);
    (*start).rules.set_x2_5(0);
    (*start).rules.set_x2_6(0);
    (*start).rules.set_x3_1(0);
    (*start).rules.set_x4_0(0);

    (*start).rules.on_unpause_override = Some(ext::cb::gm_80165268);
    (*start).rules.on_pause_override = Some(ext::cb::gm_80165268);
    (*start).rules.check_for_pauser_override = Some(ext::cb::gm_CameraModeVSGetPauser);
    (*start).rules.on_match_start = Some(ext::cb::gmCamera_801A31FC);
    (*start).rules.on_frame_start = Some(ext::cb::gmCamera_801A3098);
    (*start).rules.on_frame_end = Some(ext::cb::gmCamera_801A30E4);

    (*start).rules.xD = 1;
    (*start).rules.set_disable_pausing(0);

    for i in 0..GM_MAX_PLAYERS as usize {
        (*start).players[i] = (*vs).start.players[i];
        (*start).players[i].set_xD_b3(1);
    }

    ext::gm_SetupSubColors(start);
    ext::gm_LoadRumbleEnabled(start);
    ext::gm_LoadAnnouncer();
    ext::lbCardNew_AllocWorkArea();
    ext::lbSnap_8001E218(ext::lbDvd_GetPreloadedArchive(2007), ext::lbDvd_GetPreloadedArchive(2008));
}

/// 1B2AF8
pub unsafe extern "C" fn gm_801B2AF8(_arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B2AF8");
    let vs = row();
    let ko_counts = ext::gmVsMelee_GetKOCounts();
    ext::gm_80168638(&raw mut ext::gmVsMelee_VsExitInfo.match_end);
    ext::gm_80168710(&raw mut ext::gmVsMelee_VsExitInfo.match_end, vs);
    ext::gm_8016247C(ext::gm_801688AC(&raw mut ext::gmVsMelee_VsExitInfo.match_end));
    ext::gmVsMelee_UpdateKOCounts(ko_counts, &raw mut ext::gmVsMelee_VsExitInfo.match_end);
    ext::gm_SetNextGameModeStateId(1);
}

/// `gm_Mode_Camera_OnInit`
pub fn gm_Mode_Camera_OnInit() {
    melee_core::ct!("gm_Mode_Camera_OnInit");
    unsafe { ext::gm_InitVsMode(row()) }
}
