//! Port of src/melee/gm/gmadventure.c (Adventure Mode: the 49-row scene table, the per-scene
//! records `gm_803DE650`, the intro / fight / cutscene / game-over / CSS scene handlers, the
//! special routes (Luigi, the Giant Kirby skip, Falco, Metal Luigi, Escape from Brinstar) and the
//! `MELEE_SCENE` one-player direct launch).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, non_camel_case_types, unused_variables)]

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
                    #[allow(unused_unsafe)] None => unsafe { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Ground_801C5A28() = crate::tu::adapt::Ground_801C5A28__62c4d3;
    fn HSD_Randi(a0: i32) -> i32 = crate::tu::gmclassic::ext::HSD_Randi;
    fn SceneLaunch_OnePDirect(a0: i32) -> i32 = crate::tu::gmclassic::ext::SceneLaunch_OnePDirect;
    fn SceneLaunch_OnePLevel() -> i32 = crate::tu::gmclassic::ext::SceneLaunch_OnePLevel;
    fn SceneLaunch_OnePStep() -> i32 = crate::tu::gmclassic::ext::SceneLaunch_OnePStep;
    fn SceneLaunch_PlayerCKind(a0: i32) -> i32 = crate::tu::gmclassic::ext::SceneLaunch_PlayerCKind;
    fn SceneLaunch_PlayerColor(a0: i32) -> i32 = crate::tu::gmclassic::ext::SceneLaunch_PlayerColor;
    fn SceneLaunch_PlayerStocks(a0: i32) -> i32 = crate::tu::gmclassic::ext::SceneLaunch_PlayerStocks;
    fn gmFrontend_AtlasSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_AtlasSelect__26b7fc;
    fn gmMainLib_8015CDD4() -> *mut gmm_x0_528_t = crate::tu::adapt::gmMainLib_8015CDD4__51396a;
    fn gmRegSetupEnemyColorTable(a0: i8, a1: u8, a2: *const i8, a3: *mut u8) = crate::tu::adapt::gmRegSetupEnemyColorTable__a8a833;
    fn gm_801677F0() -> u8 = crate::tu::adapt::gm_801677F0__be41b7;
    fn gm_80168F88() = crate::tu::adapt::gm_80168F88__9d899a;
    fn gm_80169520() -> *mut c_void = crate::tu::gm_16A2::gm_80169520;
    fn gm_80169540() -> *mut c_void = crate::tu::gm_16A2::gm_80169540;
    fn gm_8016A22C(a0: i8, a1: i8, a2: i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: i32, a8: u8, a9: u8, a10: u8, a11: i32, a12: i32, a13: i32, a14: i32, a15: i32, a16: i32, a17: f32, a18: f32) = crate::tu::adapt::gm_8016A22C__e5b47e;
    fn gm_80172174() = crate::tu::adapt::gm_80172174__0e08c4;
    fn gm_8017BE84(a0: u32) -> i32 = crate::tu::adapt::gm_8017BE84__49c20d;
    fn gm_8017BE8C(a0: *const i8) -> i32 = crate::tu::gmregcommon::gm_8017BE8C;
    fn gm_8017C838() = crate::tu::gm_17c0::gm_8017C838;
    fn gm_8017C984(a0: *mut c_void) = crate::tu::gm_17c0::gm_8017C984;
    fn gm_8017C9A8(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: u8) = crate::tu::adapt::gm_8017C9A8__94d55d;
    fn gm_8017CA38(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: *mut gmm_x0_528_t, a3: u8) = crate::tu::adapt::gm_8017CA38__442d2c;
    fn gm_8017CBAC(a0: *mut UnkAdventureData, a1: *mut gmm_x0_528_t, a2: u8) = crate::tu::adapt::gm_8017CBAC__33592d;
    fn gm_8017CE34(a0: *mut StartMeleeData, a1: *mut Unk1PData, a2: *mut i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: u32, a8: i32, a9: i32) = crate::tu::adapt::gm_8017CE34__75f422;
    fn gm_8017D7AC(a0: *mut MatchExitInfo, a1: *mut Unk1PData, a2: u8) -> i32 = crate::tu::adapt::gm_8017D7AC__84cd0d;
    fn gm_8017DB58(a0: *mut Unk1PData_x24) = crate::tu::gm_17c0::gm_8017DB58;
    fn gm_8017E48C(a0: *mut GameModeState) -> u8 = crate::tu::gm_17e4::gm_8017E48C;
    fn gm_8017E4C4(a0: u8) -> *mut gm_803DE650_t = crate::tu::gm_17e4::gm_8017E4C4;
    fn gm_8017E500(a0: u8, a1: u8) -> u8 = crate::tu::gm_17e4::gm_8017E500;
    fn gm_8017E528(a0: u8, a1: u8) -> f32 = crate::tu::gm_17e4::gm_8017E528;
    fn gm_8017E578(a0: u8, a1: u8) -> f32 = crate::tu::gm_17e4::gm_8017E578;
    fn gm_8017E5C8(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E5C8;
    fn gm_8017E5FC(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E5FC;
    fn gm_8017E630(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E630;
    fn gm_8017E664(a0: u8, a1: u8) -> f32 = crate::tu::gm_17e4::gm_8017E664;
    fn gm_8017E6B4(a0: u8, a1: u8) -> f32 = crate::tu::gm_17e4::gm_8017E6B4;
    fn gm_8017E704(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E704;
    fn gm_8017E738(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E738;
    fn gm_8017E76C(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_17e4::gm_8017E76C;
    fn gm_8017E7A0(a0: u8) = crate::tu::gm_17e4::gm_8017E7A0;
    fn gm_8017E7FC(a0: u8) = crate::tu::gm_17e4::gm_8017E7FC;
    fn gm_801B06B0(a0: *mut CSSData, a1: u8, a2: i8, a3: i8, a4: i8, a5: u8, a6: u8, a7: u8) = crate::tu::adapt::gm_801B06B0__3a08bf;
    fn gm_801B0730(a0: *mut CSSData, a1: *mut i8, a2: *mut u8, a3: *mut u8, a4: *mut u8, a5: *mut u8) = crate::tu::gmclassic::ext::gm_801B0730;
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_GetAdventureData() -> *mut UnkAdventureData = crate::tu::gm_17e4::gm_GetAdventureData;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
    fn gm_GetPreviousSceneIndex() -> u8 = crate::tu::gm_1A3F::gm_GetPreviousSceneIndex;
    fn gm_IsCKindUnlocked(a0: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
    fn gm_LoadRumbleEnabled(a0: *mut StartMeleeData) = crate::tu::adapt::gm_LoadRumbleEnabled__62a06a;
    fn gm_SetGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetGameModeStateId__147996;
    fn gm_SetNewGameModePending() = crate::tu::adapt::gm_SetNewGameModePending__1e5e48;
    fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
    fn gm_SetPendingGameMode(a0: u8) = crate::tu::adapt::gm_SetPendingGameMode__90d26d;
    fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::gmclassic::ext::lbAudioAx_80026E84;
    fn lbAudioAx_80026EBC(a0: u32) -> u64 = crate::tu::gmclassic::ext::lbAudioAx_80026EBC;
    fn lbAudioAx_80026F2C(a0: u32) = crate::tu::gmclassic::ext::lbAudioAx_80026F2C;
    fn lbAudioAx_8002702C(a0: u32, a1: u64) = crate::tu::gmclassic::ext::lbAudioAx_8002702C;
    fn lbAudioAx_80027168() = crate::tu::gmclassic::ext::lbAudioAx_80027168;
    fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990;
    fn lbDvd_SetupVsPreloadCache() = crate::tu::adapt::lbDvd_SetupVsPreloadCache__d38cf3;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn gm_8017C838() { super::gm_8017C838() }
        pub unsafe extern "C" fn gm_8017E500(a0: u8, a1: u8) -> u8 { super::gm_8017E500(a0, a1) }
        pub unsafe extern "C" fn gm_8017E528(a0: u8, a1: u8) -> f32 { super::gm_8017E528(a0, a1) }
        pub unsafe extern "C" fn gm_8017E578(a0: u8, a1: u8) -> f32 { super::gm_8017E578(a0, a1) }
        pub unsafe extern "C" fn gm_8017E5C8(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E5C8(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E5FC(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E5FC(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E630(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E630(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E664(a0: u8, a1: u8) -> f32 { super::gm_8017E664(a0, a1) }
        pub unsafe extern "C" fn gm_8017E6B4(a0: u8, a1: u8) -> f32 { super::gm_8017E6B4(a0, a1) }
        pub unsafe extern "C" fn gm_8017E704(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E704(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E738(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E738(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E76C(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017E76C(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017E7A0(a0: u8) { super::gm_8017E7A0(a0) }
        pub unsafe extern "C" fn gm_8017E7FC(a0: u8) { super::gm_8017E7FC(a0) }
    }
    // Data defined in other TUs.
    pub use crate::tu::gm_17c0::{gmClassic_80470708, gmClassic_80470850, gmClassic_8047086C, gmClassic_80472AF8};
}
// END GENERATED EXT

/// `CutsceneData` (file-local): the cutscene scenes' enter data.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CutsceneData {
    pub x0: u8,
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
    pub x4: u8,
    pub x5: u8,
    pub x6: u8,
}

/// `IntroData` (file-local): the stage intro scenes' enter data.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IntroData {
    pub x0: u8,
    pub x1: u8,
    pub x2: u16,
}

pub static mut gm_804D68D8: CutsceneData = CutsceneData { x0: 0, x1: 0, x2: 0, x3: 0, x4: 0, x5: 0, x6: 0 };
pub static mut gm_804D68E0: IntroData = IntroData { x0: 0, x1: 0, x2: 0 };

const ADVENTURE_LUIGI_CUTSCENE: u8 = 0x02;
const ADVENTURE_MARIO_PEACH_FIGHT: u8 = 0x03;
const ADVENTURE_KIRBY_INTRO: u8 = 0x20;
const ADVENTURE_GIANTKIRBY_FIGHT: u8 = 0x25;
const ADVENTURE_GAME_OVER: u8 = 0x69;
const ADVENTURE_BACK_TO_CSS: u8 = 0x70;

/// `NEXT_SCENEBLOCK_AFTER`
const fn next_sceneblock_after(x: u8) -> u8 {
    (x + 8) & !(8 - 1)
}

#[allow(clippy::too_many_arguments)]
const fn ent(x0: u8, x1: u8, x2: u16, x4: u16, x6: u16, x8: u8, x9: u8, xa: [i8; 3], xd: [u8; 3]) -> gm_803DE650_t {
    gm_803DE650_t { x0, x1, x2, x4, x6, x8, x9, xA: xa, xD: xd[0], xE: xd[1], xF: xd[2] }
}

/// The scene table (49 rows: the twelve stages' intro / fight / cutscene blocks, coming soon,
/// game over, CSS, end) and, below it, the per-scene records `gm_803DE650` (`x0 == 0xFF` ends them).
pub static mut gm_Mode_Adventure_States: [GameModeState; 49] = unsafe { [
    state(0x00, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x01, 2, Some(gm_801B4064), Some(gm_801B44A0), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x02, 2, Some(gm_801B45A4), None, GS_CUTSCENE_LUIGI, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x03, 2, Some(gm_801B461C), Some(gm_801B4684), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x08, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x09, 2, Some(gm_801B4768), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x0a, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x10, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x11, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x12, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x18, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x19, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x1a, 2, Some(gm_801B4430), None, GS_CUTSCENE_BRINSTAR, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x1b, 2, Some(gm_801B47FC), Some(gm_801B4860), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x1c, 2, None, None, GS_CUTSCENE_EXPLOSION, NULL, NULL),
    state(0x20, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x21, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x22, 2, Some(gm_801B4974), None, GS_CUTSCENE_3KIRBYS, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x23, 2, Some(gm_801B4B28), Some(gm_801B4C5C), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x24, 2, Some(gm_801B4430), None, GS_CUTSCENE_GIANTKIRBY, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x25, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x28, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x29, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x2a, 2, Some(gm_801B4D34), None, GS_CUTSCENE_STARFOX, NULL, NULL),
    state(0x2b, 2, Some(gm_801B4DAC), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x30, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x31, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x38, 2, None, None, GS_CUTSCENE_FZERO, NULL, NULL),
    state(0x39, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x3a, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x3b, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x40, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x41, 2, Some(gm_801B4E58), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x48, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x49, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x50, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x51, 2, Some(gm_801B4064), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x52, 2, Some(gm_801B4430), None, GS_CUTSCENE_METAL, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x53, 2, Some(gm_801B4EB8), Some(gm_801B4170), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x58, 2, Some(gm_801B3F40), None, GS_INTRO_NORMAL, &raw mut gm_804D68E0 as *mut c_void, NULL),
    state(0x59, 2, Some(gm_801B4F44), Some(gm_801B4FCC), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x5a, 2, Some(gm_801B4430), Some(gm_801B5078), GS_CUTSCENE_BOWSERTOY, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x5b, 2, Some(gm_801B4430), None, GS_CUTSCENE_GIGATRANSFORM, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x5c, 2, Some(gm_801B4064), Some(gm_801B50C4), GS_VS, &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
    state(0x5d, 2, Some(gm_801B4430), Some(gm_801B518C), GS_CUTSCENE_GIGADEFEATED, &raw mut gm_804D68D8 as *mut c_void, NULL),
    state(0x68, 2, None, Some(gm_801B4408), GS_COMING_SOON, NULL, NULL),
    state(0x69, 2, Some(gm_801B4254), Some(gm_801B4294), GS_GAMEOVER, &raw mut ext::gmClassic_80470850 as *mut c_void, &raw mut ext::gmClassic_80470850 as *mut c_void),
    state(0x70, 2, Some(gm_801B42E8), Some(gm_801B4350), GS_CSS, &raw mut ext::gmClassic_80470708 as *mut c_void, &raw mut ext::gmClassic_80470708 as *mut c_void),
    end(),
] };
pub static mut gm_803DE650: [gm_803DE650_t; 46] = [
    ent(0x00, 0x00, 0x0000, 0x0000, 0x003B, 0x04, 0x00, [CKind_Yoshi as i8, CKind_Yoshi as i8, CKind_Yoshi as i8], [0x00, 0x00, 0x00]),
    ent(0x01, 0x09, 0x01A4, 0x0000, 0x003B, 0x04, 0x00, [CKind_Yoshi as i8, CKind_Yoshi as i8, CKind_Yoshi as i8], [0x00, 0x00, 0x00]),
    ent(0x02, 0x00, 0x0000, 0x0000, 0x003B, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x03, 0x00, 0x00F0, 0x0000, 0x003C, 0x03, 0x00, [CKind_Mario as i8, CKind_Peach as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x08, 0x00, 0x0000, 0x0000, 0x003D, 0x04, 0x00, [CKind_Donkey as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x09, 0x00, 0x00F0, 0x0000, 0x003D, 0x03, 0x00, [CKind_Donkey as i8, CKind_Donkey as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x0a, 0x02, 0x00F0, 0x0000, 0x003E, 0x02, 0x00, [CKind_Donkey as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x10, 0x00, 0x0000, 0x0000, 0x003F, 0x04, 0x00, [CKind_Link as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x11, 0x09, 0x01A4, 0x0000, 0x003F, 0x04, 0x00, [CKind_Link as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x12, 0x00, 0x00F0, 0x0000, 0x0040, 0x02, 0x00, [CKind_Zelda as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x18, 0x00, 0x0000, 0x0000, 0x0041, 0x04, 0x00, [CKind_Samus as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x19, 0x00, 0x00F0, 0x0000, 0x0041, 0x02, 0x00, [CKind_Samus as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x1a, 0x00, 0x0000, 0x0000, 0x0042, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x1b, 0x41, 0x0028, 0x0000, 0x0042, 0x04, 0xFF, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x1c, 0x00, 0x0000, 0x0000, 0x0042, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x20, 0x00, 0x0000, 0x0000, 0x0043, 0x04, 0x00, [CKind_Kirby as i8, CKind_Kirby as i8, CKind_Kirby as i8], [0x00, 0x00, 0x00]),
    ent(0x21, 0x00, 0x00F0, 0x0000, 0x0043, 0x02, 0x00, [CKind_Kirby as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x22, 0x00, 0x0000, 0x000F, 0x0044, 0x04, 0x00, [CKind_Kirby as i8, CKind_Kirby as i8, CKind_Kirby as i8], [0x00, 0x00, 0x00]),
    ent(0x23, 0x08, 0x00F0, 0x000F, 0x0044, 0x04, 0x00, [CKind_Kirby as i8, CKind_Kirby as i8, CKind_Kirby as i8], [0x00, 0x00, 0x00]),
    ent(0x24, 0x00, 0x0000, 0x0000, 0x0045, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x25, 0x02, 0x00F0, 0x0000, 0x0045, 0x02, 0x00, [CKind_Kirby as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x28, 0x00, 0x0000, 0x0000, 0x0046, 0x04, 0x00, [CKind_Fox as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x29, 0x00, 0x00F0, 0x0000, 0x0046, 0x02, 0x00, [CKind_Fox as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x2a, 0x00, 0x0000, 0x0000, 0x0047, 0x04, 0x00, [CKind_Fox as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x2b, 0x00, 0x00F0, 0x0000, 0x0047, 0x02, 0x00, [CKind_Fox as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x30, 0x00, 0x0000, 0x0000, 0x0048, 0x04, 0x00, [CKind_Pikachu as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x31, 0x08, 0x00F0, 0x000C, 0x0048, 0x04, 0x00, [CKind_Pikachu as i8, CKind_Pichu as i8, CKind_Purin as i8], [0x00, 0x00, 0x00]),
    ent(0x38, 0x00, 0x0000, 0x0000, 0x0049, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x39, 0x00, 0x0000, 0x0000, 0x0049, 0x04, 0x00, [CKind_Captain as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x3a, 0x41, 0x00F0, 0x0000, 0x0049, 0x04, 0x00, [ChKind_None as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x3b, 0x00, 0x00F0, 0x0000, 0x004A, 0x02, 0x00, [CKind_Captain as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x40, 0x00, 0x0000, 0x0000, 0x004B, 0x04, 0x00, [CKind_Ness as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x41, 0x00, 0x00F0, 0x0000, 0x004B, 0x04, 0x00, [11, 11, 11], [0x00, 0x00, 0x00]),
    ent(0x48, 0x00, 0x0000, 0x0000, 0x004C, 0x04, 0x00, [CKind_PopoNana as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x49, 0x09, 0x00F0, 0x0000, 0x004C, 0x04, 0x00, [CKind_PopoNana as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x50, 0x00, 0x0000, 0x0000, 0x004E, 0x04, 0x00, [CKind_Boy as i8, CKind_Girl as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x51, 0x08, 0x00F0, 0x000F, 0x004E, 0x06, 0x00, [CKind_Boy as i8, CKind_Girl as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x52, 0x00, 0x0000, 0x0000, 0x004F, 0x04, 0x00, [CKind_Mario as i8, CKind_Luigi as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x53, 0x04, 0x00F0, 0x0000, 0x004F, 0x03, 0x00, [CKind_Mario as i8, CKind_Luigi as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x58, 0x00, 0x0000, 0x0000, 0x0050, 0x04, 0x00, [CKind_Koopa as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x59, 0x02, 0x00F0, 0x0000, 0x0050, 0x02, 0x00, [CKind_Koopa as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x5a, 0x00, 0x0000, 0x0000, 0x0050, 0x04, 0x00, [CKind_Koopa as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x5b, 0x00, 0x0000, 0x0000, 0x0051, 0x04, 0x00, [CKind_GKoops as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x5c, 0x20, 0x00F0, 0x0000, 0x0051, 0x02, 0x00, [CKind_GKoops as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0x5d, 0x00, 0x00F0, 0x0000, 0x0051, 0x02, 0x00, [CKind_GKoops as i8, ChKind_None as i8, ChKind_None as i8], [0x00, 0x00, 0x00]),
    ent(0xFF, 0, 0, 0, 0, 0, 0, [0, 0, 0], [0, 0, 0]),
];

/// `gm_80490910`: per scene record, 0 (not played), 1 (played) or 2 (played, the exit flag set).
pub static mut gm_80490910: [u8; 46] = [0; 46];

/// `getIndex` (static inline): the `gm_803DE650` index of `scene_id`'s record.
unsafe fn get_index(scene_id: u8) -> usize {
    <usize as melee_core::ppc::Ppc>::pdiv(ext::gm_8017E4C4(scene_id) as usize - (&raw mut gm_803DE650) as usize, core::mem::size_of::<gm_803DE650_t>())
}

/// `setValUnk` (static inline)
unsafe fn set_val_unk(scene_id: u8, temp_r30: i32) {
    let i = get_index(scene_id);
    gm_80490910[i] = if temp_r30 != 0 { 2 } else { 1 };
}

unsafe fn adv() -> *mut UnkAdventureData {
    ext::gm_GetAdventureData()
}

/// 1B3F40: a stage intro (the stage number, the stage, the slot; the sound banks).
pub unsafe extern "C" fn gm_801B3F40(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B3F40");
    let mut temp_ret: u64 = 0;
    let temp_r30 = ext::gm_GetGameModeStateEnterData(arg0) as *mut IntroData;
    let temp_r31 = ext::gm_8017E4C4((*arg0).id);
    let temp_r3 = adv();
    (*temp_r3).x0.x0.mode = (*arg0).id;
    (*temp_r30).x1 = ext::gm_8017BE84((*arg0).id as u32) as u8;
    (*temp_r30).x2 = (*temp_r31).x6;
    (*temp_r30).x0 = (*temp_r3).x0.x0.slot;
    temp_ret |= ext::lbAudioAx_80026E84((*temp_r3).x0.x0.ckind as i32 as u32);
    for i in 0..3 {
        let ckind = (*temp_r3).x0.xC.x24[i].ckind;
        if ckind as i32 != ChKind_None as i32 {
            temp_ret |= ext::lbAudioAx_80026E84(ckind as i32 as u32);
        }
    }
    for i in 0..3 {
        if (*temp_r31).xA[i] as i32 != ChKind_None as i32 {
            temp_ret |= ext::lbAudioAx_80026E84((*temp_r31).xA[i] as i32 as u32);
            if (*temp_r31).xA[i] as i32 == CKind_Kirby as i32 {
                temp_ret |= 0x2_0000_4000;
            }
        }
    }
    temp_ret |= ext::lbAudioAx_80026EBC((*temp_r31).x6 as u32);
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, temp_ret);
    ext::lbAudioAx_80027168();
}

/// 1B4064: a fight's StartMeleeData from the scene record.
pub unsafe extern "C" fn gm_801B4064(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B4064");
    let temp_r28 = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let temp_r31 = ext::gm_8017E4C4((*arg0).id);
    let temp_r27 = adv();
    let (var_r30, var_r29) = if (*temp_r31).x4 != 0 { (1u8, 0u8) } else { (0u8, 1u8) };
    (*temp_r27).x0.x8 = (*temp_r31).x1;
    (*temp_r27).x0.x9 = 0;
    (*temp_r27).x0.xB = (*temp_r31).x8;
    (*temp_r27).x0.xA = (*temp_r31).x9;
    let tmp = gm_80490910[get_index((*arg0).id)];
    ext::gm_8017CE34(
        temp_r28,
        &raw mut (*temp_r27).x0,
        (*temp_r31).xA.as_mut_ptr(),
        (*temp_r31).x4 as u8,
        var_r30,
        var_r29,
        (*temp_r31).x2 as i32,
        (*temp_r31).x6 as u32,
        ext::gm_8017E48C(arg0) as i32,
        tmp as i32,
    );
    ext::gm_LoadRumbleEnabled(temp_r28);
    if (*temp_r31).x1 & 8 != 0 {
        (*temp_r28).rules.on_match_start = Some(ext::cb::gm_8017C838);
    }
}

/// 1B4170: a fight's exit (game over, or the mode's end after the last record).
pub unsafe extern "C" fn gm_801B4170(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B4170");
    let temp_r28 = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    let temp_r31 = adv();
    set_val_unk((*arg0).id, (*temp_r28).x8);
    if ext::gm_8017D7AC(temp_r28, &raw mut (*temp_r31).x0, ADVENTURE_GAME_OVER) != 0
        && (*ext::gm_8017E4C4((*arg0).id).add(1)).x0 == 0xFF
    {
        ext::gm_8017CBAC(temp_r31, ext::gmMainLib_8015CDD4(), 0x16);
    }
}

/// 1B4254
pub unsafe extern "C" fn gm_801B4254(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4254");
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut DebugGameOverData;
    ext::gm_8017C9A8(data, &raw mut (*adv()).x0, 0);
}

/// 1B4294
pub unsafe extern "C" fn gm_801B4294(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4294");
    let data = ext::gm_GetGameModeStateExitData(scene) as *mut DebugGameOverData;
    ext::gm_8017CA38(data, &raw mut (*adv()).x0, ext::gmMainLib_8015CDD4(), 0);
}

/// 1B42E8: the CSS's enter data.
pub unsafe extern "C" fn gm_801B42E8(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B42E8");
    let css = ext::gm_GetGameModeStateEnterData(scene) as *mut CSSData;
    let temp_r31 = ext::gmMainLib_8015CDD4();
    ext::gm_801B06B0(
        css,
        0xC,
        (*temp_r31).c_kind,
        (*temp_r31).stocks as i8,
        (*temp_r31).color as i8,
        (*temp_r31).nametag,
        (*temp_r31).cpu_level,
        (*adv()).x0.x0.slot,
    );
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// 1B4350: the CSS's exit (back to the menu, or into the first stage).
pub unsafe extern "C" fn gm_801B4350(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4350");
    let css = ext::gm_GetGameModeStateExitData(scene) as *mut CSSData;
    let temp_r29 = ext::gmMainLib_8015CDD4();
    let temp_r31 = adv();
    if (*css).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    ext::gm_801B0730(
        css,
        &raw mut (*temp_r29).c_kind,
        &raw mut (*temp_r29).stocks,
        &raw mut (*temp_r29).color,
        &raw mut (*temp_r29).nametag,
        &raw mut (*temp_r29).cpu_level,
    );
    (*temp_r31).x0.x0.ckind = (*temp_r29).c_kind;
    (*temp_r31).x0.x0.color = (*temp_r29).color;
    (*temp_r31).x0.x0.cpu_level = (*temp_r29).cpu_level;
    (*temp_r31).x0.x0.stocks = (*temp_r29).stocks;
    (*temp_r31).x0.x0.nametag = (*temp_r29).nametag;
    ext::gm_SetNextGameModeStateId((*temp_r29).x5 << 3);
    ext::gm_80168F88();
}

/// 1B4408: the coming-soon scene's exit, back to the menu.
pub unsafe extern "C" fn gm_801B4408(_scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4408");
    ext::gm_SetPendingGameMode(GM_MENU as u8);
    ext::gm_SetNewGameModePending();
}

/// The player's character for a cutscene (Sheik when Zelda transformed).
unsafe fn cutscene_ckind(d: *mut UnkAdventureData) -> u8 {
    if (*d).x0.x0.ckind as i32 == CKind_Zelda as i32 && (*d).x0.xC.x12 != 0 {
        CKind_Seak as u8
    } else {
        (*d).x0.x0.ckind as u8
    }
}

/// 1B4430: a cutscene's player character and color.
pub unsafe extern "C" fn gm_801B4430(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4430");
    let temp_r30 = ext::gm_GetGameModeStateEnterData(scene) as *mut CutsceneData;
    let temp_r31 = ext::gmMainLib_8015CDD4();
    let temp_r3 = adv();
    (*temp_r30).x0 = cutscene_ckind(temp_r3);
    (*temp_r30).x1 = (*temp_r31).color;
}

/// 1B44A0: Mushroom Kingdom's exit (the Luigi route: a 2 in the timer's seconds digit).
pub unsafe extern "C" fn gm_801B44A0(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B44A0");
    let temp_r30 = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let temp_r29 = adv();
    set_val_unk((*scene).id, (*temp_r30).x8);
    if ext::gm_8017D7AC(temp_r30, &raw mut (*temp_r29).x0, ADVENTURE_GAME_OVER) != 0 {
        let seconds = (*temp_r30).x4 % 60;
        if seconds % 10 == 2 {
            ext::gm_SetNextGameModeStateId(ADVENTURE_LUIGI_CUTSCENE);
        } else {
            ext::gm_SetNextGameModeStateId(ADVENTURE_MARIO_PEACH_FIGHT);
        }
    }
}

/// 1B45A4: the Luigi cutscene's colors.
pub unsafe extern "C" fn gm_801B45A4(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B45A4");
    let temp_r31 = ext::gm_GetGameModeStateEnterData(scene) as *mut CutsceneData;
    let temp_r7 = adv();
    let mut colors: [u8; 3] = [0; 3];
    let ckinds: [i8; 3] = [CKind_Mario as i8, CKind_Luigi as i8, ChKind_None as i8];
    ext::gmRegSetupEnemyColorTable((*temp_r7).x0.x0.ckind, (*temp_r7).x0.x0.color, ckinds.as_ptr(), colors.as_mut_ptr());
    (*temp_r31).x1 = colors[0];
    (*temp_r31).x3 = colors[1];
}

/// 1B461C: Peach's Castle (Luigi instead of Mario after the Luigi cutscene).
pub unsafe extern "C" fn gm_801B461C(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B461C");
    gm_801B4064(scene);
    if ext::gm_GetPreviousSceneIndex() == ADVENTURE_LUIGI_CUTSCENE {
        let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
        let adventure = adv();
        (*data).players[1].ckind = CKind_Luigi as i8;
        (*data).players[1].color = gm_804D68D8.x3;
        (*adventure).x74 = 1;
    }
}

/// 1B4684: Peach's Castle's exit (the Luigi unlock: under a minute).
pub unsafe extern "C" fn gm_801B4684(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4684");
    let temp_r29 = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let temp_r31 = adv();
    set_val_unk((*scene).id, (*temp_r29).x8);
    if ext::gm_8017D7AC(temp_r29, &raw mut (*temp_r31).x0, ADVENTURE_GAME_OVER) != 0 {
        (*temp_r31).x75 = if (*temp_r29).match_end.frame_count / 60 <= 60 { 1 } else { 0 };
    }
}

/// 1B4768: the tiny Donkey Kongs (half scale).
pub unsafe extern "C" fn gm_801B4768(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4768");
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    gm_801B4064(scene);
    for i in 0..3 {
        if (*data).players[i + 1].slot_type as u32 == Gm_PKind_Cpu as u32 {
            (*data).players[i + 1].model_scale *= 0.5f32;
        }
    }
}

/// 1B47FC: Escape from Brinstar (its match-end callback; a continue resumes at Green Greens).
pub unsafe extern "C" fn gm_801B47FC(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B47FC");
    let temp_r30 = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    let temp_r31 = adv();
    gm_801B4064(scene);
    (*temp_r30).rules.on_match_end = Some(ext::cb::gm_8017E7A0);
    (*temp_r31).x0.x0.mode = ADVENTURE_KIRBY_INTRO;
}

/// 1B4860: Escape from Brinstar's exit (a timeout costs no stock).
pub unsafe extern "C" fn gm_801B4860(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4860");
    // gm_801B4860_inline0
    let exit_info = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    if (*exit_info).match_end.outcome as u32 != OUTCOME_TIMEOUT as u32 {
        (*adv()).x76 = 1;
    } else {
        let s = &raw mut (*exit_info).match_end.player_standings[0].stocks;
        *s = (*s).wrapping_add(1);
    }
    // gm_801B4860_inline1
    let exit_info = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let adv_data = adv();
    set_val_unk((*scene).id, (*exit_info).x8);
    if ext::gm_8017D7AC(exit_info, &raw mut (*adv_data).x0, ADVENTURE_GAME_OVER) != 0
        && (*ext::gm_8017E4C4((*scene).id).add(1)).x0 == 0xFF
    {
        ext::gm_8017CBAC(adv_data, ext::gmMainLib_8015CDD4(), 0x16);
    }
}

/// `gm_8016A22C_inline` (static): the three Kirbys' fight setup.
unsafe fn gm_8016A22C_inline(arg0: *mut GameModeState, temp_r29: *mut UnkAdventureData) {
    let count = ext::gm_8017E48C(arg0);
    let r = ext::gm_8017E4C4((*arg0).id);
    let lvl = (*temp_r29).x0.x0.cpu_level;
    let x4c = (*temp_r29).x0.x4C.unwrap()(count, lvl, 0);
    let ord = ext::gm_8017BE8C((*r).xA.as_ptr());
    let x64 = (*temp_r29).x0.x64.unwrap()(count, lvl);
    let x68 = (*temp_r29).x0.x68.unwrap()(count, lvl);
    ext::gm_8016A22C(
        (*r).xA[0], (*r).xA[1], (*r).xA[2], (*r).xD, (*r).xE, (*r).xF, 1, 0, 1,
        (*temp_r29).x0.x0.ckind as u8, (*temp_r29).x0.x0.color,
        x4c as i32, (*r).x4 as i32, ord, 1, 0, 1, x64, x68,
    );
    (*temp_r29).x0.xC.x11 = 1;
}

/// 1B4974: the Team Kirby cutscene.
pub unsafe extern "C" fn gm_801B4974(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B4974");
    let temp_r28 = ext::gm_GetGameModeStateEnterData(arg0) as *mut CutsceneData;
    let temp_r29 = adv();
    let temp_r25 = ext::gm_GetGameModeStateEnterData(arg0) as *mut CutsceneData;
    let temp_r24 = ext::gmMainLib_8015CDD4();
    let temp_r3 = adv();
    (*temp_r25).x0 = cutscene_ckind(temp_r3);
    (*temp_r25).x1 = (*temp_r24).color;
    let temp_r30 = ext::gm_8017E4C4((*arg0).id);
    let temp_r24_2 = ext::gm_80169520() as *mut u8;
    let temp_r31 = ext::gm_80169540() as *mut u8;
    gm_8016A22C_inline(arg0, temp_r29);
    let x4 = (*temp_r30).x4 as isize;
    (*temp_r28).x3 = *temp_r24_2;
    (*temp_r28).x4 = *temp_r31.offset(x4 - 2);
    (*temp_r28).x5 = *temp_r31.offset(x4 - 1);
    (*temp_r28).x6 = *temp_r31.offset(x4 - 3);
}

/// 1B4B28: the Team Kirby fight.
pub unsafe extern "C" fn gm_801B4B28(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4B28");
    let adv_data = adv();
    if (*adv_data).x0.xC.x11 != 1 {
        let _unused1 = ext::gm_80169520();
        let _unused2 = ext::gm_80169540();
        gm_8016A22C_inline(scene, adv_data);
    }
    gm_801B4064(scene);
}

/// 1B4C5C: Team Kirby's exit (over 30 seconds skips the Giant Kirby battle).
pub unsafe extern "C" fn gm_801B4C5C(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4C5C");
    let temp_r30 = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let temp_r29 = adv();
    set_val_unk((*scene).id, (*temp_r30).x8);
    if ext::gm_8017D7AC(temp_r30, &raw mut (*temp_r29).x0, ADVENTURE_GAME_OVER) != 0
        && (*temp_r30).match_end.frame_count / 60 > 30
    {
        ext::gm_SetNextGameModeStateId(next_sceneblock_after(ADVENTURE_GIANTKIRBY_FIGHT));
    }
}

/// 1B4D34: the Star Fox cutscene (Falco about a third of the time once unlocked).
pub unsafe extern "C" fn gm_801B4D34(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4D34");
    let temp_r30 = adv();
    let temp_r31 = ext::gm_8017E4C4((*scene).id);
    if ext::gm_IsCKindUnlocked(CKind_Falco as u8) != 0 && ext::HSD_Randi(100) <= 33 {
        (*temp_r30).x7C = CKind_Falco as u8;
    } else {
        (*temp_r30).x7C = (*temp_r31).xA[0] as u8;
    }
}

/// 1B4DAC: Corneria's second fight (Fox or Falco).
pub unsafe extern "C" fn gm_801B4DAC(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4DAC");
    let temp_r30 = adv();
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    gm_801B4064(scene);
    if (*temp_r30).x7C as u32 == CKind_Falco as u32 {
        let ckinds: [i8; 3] = [CKind_Falco as i8, ChKind_None as i8, ChKind_None as i8];
        let mut colors: [u8; 3] = [0; 3];
        colors[0] = (*data).players[1].color;
        ext::gmRegSetupEnemyColorTable((*temp_r30).x0.x0.ckind, (*temp_r30).x0.x0.color, ckinds.as_ptr(), colors.as_mut_ptr());
        (*data).players[1].ckind = CKind_Falco as i8;
        (*data).players[1].color = colors[0];
    }
}

/// 1B4E58: Onett (an item on, frequency 4).
pub unsafe extern "C" fn gm_801B4E58(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4E58");
    adv();
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    gm_801B4064(scene);
    (*data).rules.x20 = 1 << 7;
    (*data).rules.item_freq = 4;
}

/// 1B4EB8: Metal Mario (Metal Luigi only once Luigi is unlocked).
pub unsafe extern "C" fn gm_801B4EB8(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4EB8");
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    gm_801B4064(scene);
    if ext::gm_IsCKindUnlocked(CKind_Luigi as u8) == 0 {
        for i in 0..3 {
            if (*data).players[i + 1].ckind as i32 == CKind_Luigi as i32 {
                (*data).players[i + 1].slot_type = Gm_PKind_NA as u8;
            }
        }
    }
}

/// 1B4F44: Bowser (1.4 scale, its match-end callback).
pub unsafe extern "C" fn gm_801B4F44(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4F44");
    let data = ext::gm_GetGameModeStateEnterData(scene) as *mut StartMeleeData;
    gm_801B4064(scene);
    for i in 0..3 {
        if (*data).players[i + 1].slot_type as u32 == Gm_PKind_Cpu as u32 {
            (*data).players[i + 1].model_scale = 1.4f32;
        }
    }
    (*data).rules.on_match_end = Some(ext::cb::gm_8017E7FC);
}

/// 1B4FCC: Bowser's exit.
pub unsafe extern "C" fn gm_801B4FCC(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B4FCC");
    let temp_r30 = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let temp_r29 = adv();
    set_val_unk((*scene).id, (*temp_r30).x8);
    ext::gm_8017D7AC(temp_r30, &raw mut (*temp_r29).x0, ADVENTURE_GAME_OVER);
}

/// 1B5078: the Bowser trophy cutscene's exit (the mode ends unless Giga Bowser follows).
pub unsafe extern "C" fn gm_801B5078(_scene: *mut GameModeState) {
    melee_core::ct!("gm_801B5078");
    let temp_r3 = adv();
    if (*temp_r3).x77 == 0 {
        ext::gm_8017CBAC(temp_r3, ext::gmMainLib_8015CDD4(), 0x16);
    }
}

/// 1B50C4: Giga Bowser's exit.
pub unsafe extern "C" fn gm_801B50C4(scene: *mut GameModeState) {
    melee_core::ct!("gm_801B50C4");
    let temp_r31 = ext::gm_GetGameModeStateExitData(scene) as *mut MatchExitInfo;
    let temp_r30 = adv();
    let temp_r3 = adv();
    if (*temp_r3).x0.x0.stocks as i32 == (*temp_r31).match_end.player_standings[0].stocks as i32 {
        (*temp_r3).x78 = 1;
    }
    set_val_unk((*scene).id, (*temp_r31).x8);
    ext::gm_8017D7AC(temp_r31, &raw mut (*temp_r30).x0, ADVENTURE_GAME_OVER);
}

/// 1B518C: the Giga Bowser defeated cutscene's exit (the mode's end).
pub unsafe extern "C" fn gm_801B518C(_scene: *mut GameModeState) {
    melee_core::ct!("gm_801B518C");
    ext::gm_8017CBAC(adv(), ext::gmMainLib_8015CDD4(), 0x16);
}

/// `gm_Mode_Adventure_OnLoad`
pub fn gm_Mode_Adventure_OnLoad() {
    melee_core::ct!("gm_Mode_Adventure_OnLoad");
    unsafe {
        let data = adv();
        ext::gmMainLib_8015CDD4();
        ext::gm_8017C984(data as *mut c_void);
        core::ptr::write_bytes(data as *mut u8, 0, core::mem::size_of::<UnkAdventureData>());
        (*data).x74 = 0;
        for i in 0..gm_80490910.len() {
            gm_80490910[i] = 0;
        }
        ext::gm_8017DB58((*data).x0.xC.x24.as_mut_ptr());
        (*data).x0.x0.slot = ext::gm_801677F0();
        (*data).x0.x48 = Some(ext::cb::gm_8017E500);
        (*data).x0.x4C = Some(ext::cb::gm_8017E5C8);
        (*data).x0.x50 = Some(ext::cb::gm_8017E630);
        (*data).x0.x54 = Some(ext::cb::gm_8017E5FC);
        (*data).x0.x64 = Some(ext::cb::gm_8017E528);
        (*data).x0.x68 = Some(ext::cb::gm_8017E578);
        (*data).x0.x58 = Some(ext::cb::gm_8017E704);
        (*data).x0.x5C = Some(ext::cb::gm_8017E76C);
        (*data).x0.x60 = Some(ext::cb::gm_8017E738);
        (*data).x0.x6C = Some(ext::cb::gm_8017E664);
        (*data).x0.x70 = Some(ext::cb::gm_8017E6B4);
        ext::gm_SetGameModeStateId(ADVENTURE_BACK_TO_CSS);
        ext::gm_80172174();
        ext::Ground_801C5A28();
        if crate::tu::gmclassic::gm_OnePSceneDirect(ext::gmMainLib_8015CDD4(), &raw mut (*data).x0, GM_ADVENTURE as i32) != 0 {
            ext::gm_80168F88();
            ext::gm_SetGameModeStateId((*ext::gmMainLib_8015CDD4()).x5 << 3);
        }
    }
}
