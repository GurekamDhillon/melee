//! Port of src/melee/gm/gmclassic.c (Classic Mode: the 26-row scene table, the matchup tables, the
//! random matchup order, the opponent picker, the intro / VS / game-over / CSS scene handlers, and
//! the `MELEE_SCENE` one-player direct launch). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_gmclassic.rs.
//!
//! Memory model: on the console `gmClassicIntroDataBuffer` (0x20 bytes) and `gm_804908A0` (0x70
//! bytes) are adjacent and the code reaches the latter through the former (`gmClassicRuntimeData`).
//! Here they are one object, `gmClassicRuntimeBuf`; `gm_804908A0()` is its `state` bytes.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, non_camel_case_types, unused_variables)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use crate::gm_ptr::{gmClassicMatchup as gmClassicMatchup_gm_ptr};
use super::support::*;
use core::ffi::c_void;
use melee_types::*;

pub use crate::gm_ptr::gmClassicMatchup;
/// Already ported and oracle-checked in round 3 (`tests/oracle_ptr.rs`): the slice form of the
/// matchup count. This TU's own callers use `matchup_count` (pointer form, terminator search).

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
    fn Ground_801C1DE4(a0: *mut i32, a1: *mut i32) = crate::tu::adapt::Ground_801C1DE4__3e6526;
    fn Ground_801C5A28() = crate::tu::adapt::Ground_801C5A28__62c4d3;
    fn HSD_Randi(a0: i32) -> i32;
    fn SceneLaunch_OnePDirect(a0: i32) -> i32;
    fn SceneLaunch_OnePLevel() -> i32;
    fn SceneLaunch_OnePStep() -> i32;
    fn SceneLaunch_PlayerCKind(a0: i32) -> i32;
    fn SceneLaunch_PlayerColor(a0: i32) -> i32;
    fn SceneLaunch_PlayerStocks(a0: i32) -> i32;
    fn Script_OnePScripted() -> i32;
    fn Stage_8022519C(a0: u32) -> u32 = crate::tu::adapt::Stage_8022519C__a93aca;
    fn gmFrontend_AtlasSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_AtlasSelect__26b7fc;
    fn gmMainLib_8015CDC8() -> *mut gmm_x0_528_t = crate::tu::adapt::gmMainLib_8015CDC8__ddba5a;
    fn gmMainLib_8015D438(a0: u8) -> *mut u32 = crate::tu::adapt::gmMainLib_8015D438__7caf41;
    fn gmMainLib_8015D450(a0: u8) -> *mut i32 = crate::tu::adapt::gmMainLib_8015D450__dabb77;
    fn gmMainLib_8015D48C(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D48C__ede167;
    fn gmMainLib_8015D4E8(a0: u8, a1: i32) = crate::tu::adapt::gmMainLib_8015D4E8__f34d9f;
    fn gmMainLib_8015ED98() -> *mut gmm_retval_ED98 = crate::tu::adapt::gmMainLib_8015ED98__51617a;
    fn gmMainLib_8015EDB0() -> *mut gmm_retval_EDB0 = crate::tu::adapt::gmMainLib_8015EDB0__52d907;
    fn gmMainLib_8015EDBC() -> *mut gmm_retval_EDBC = crate::tu::adapt::gmMainLib_8015EDBC__bd0e2b;
    fn gmRegSetupEnemyColorTable(a0: i8, a1: u8, a2: *const i8, a3: *mut u8) = crate::tu::adapt::gmRegSetupEnemyColorTable__a8a833;
    fn gm_80164430(a0: u16) -> i32 = crate::tu::adapt::gm_80164430__8808be;
    fn gm_801647F8(a0: u8) -> i32 = crate::tu::adapt::gm_801647F8__46ed88;
    fn gm_801677F0() -> u8 = crate::tu::adapt::gm_801677F0__be41b7;
    fn gm_80168F88() = crate::tu::adapt::gm_80168F88__9d899a;
    fn gm_80172174() = crate::tu::adapt::gm_80172174__0e08c4;
    fn gm_8017BE84(a0: u32) -> i32 = crate::tu::adapt::gm_8017BE84__49c20d;
    fn gm_8017C984(a0: *mut c_void);
    fn gm_8017C9A8(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: u8) = crate::tu::adapt::gm_8017C9A8__94d55d;
    fn gm_8017CA38(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: *mut gmm_x0_528_t, a3: u8) = crate::tu::adapt::gm_8017CA38__442d2c;
    fn gm_8017CBAC(a0: *mut UnkAdventureData, a1: *mut gmm_x0_528_t, a2: u8) = crate::tu::adapt::gm_8017CBAC__33592d;
    fn gm_8017CD94(a0: *mut UnkAdventureData, a1: i32, a2: i32, a3: i32) -> u8 = crate::tu::adapt::gm_8017CD94__09d838;
    fn gm_8017CE34(a0: *mut StartMeleeData, a1: *mut Unk1PData, a2: *mut i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: u32, a8: i32, a9: i32) = crate::tu::adapt::gm_8017CE34__75f422;
    fn gm_8017D7AC(a0: *mut MatchExitInfo, a1: *mut Unk1PData, a2: u8) -> i32 = crate::tu::adapt::gm_8017D7AC__84cd0d;
    fn gm_8017DB58(a0: *mut Unk1PData_x24);
    fn gm_8017DB6C(a0: *mut gm_8017DB6C_arg0_t, a1: i32) -> u8;
    fn gm_8017DB78(a0: *mut gm_8017DB6C_arg0_t, a1: i32) -> u8;
    fn gm_8017DB88(a0: *mut c_void, a1: u8, a2: i32, a3: i32, a4: *mut u8, a5: u8, a6: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a7: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a8: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>, a9: Option<unsafe extern "C" fn(i32, i32) -> f32>, a10: Option<unsafe extern "C" fn(i32, i32) -> f32>) -> i32 = crate::tu::adapt::gm_8017DB88__2129f0;
    fn gm_8017EB3C(a0: u8, a1: u8) -> u8 = crate::tu::adapt::gm_8017EB3C__28a3ff;
    fn gm_8017EB64(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017EB64__39144b;
    fn gm_8017EB98(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017EB98__c0f6a1;
    fn gm_8017EBCC(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017EBCC__415a10;
    fn gm_8017EC00(a0: u8, a1: u8) -> f32 = crate::tu::adapt::gm_8017EC00__0d00a7;
    fn gm_8017EC50(a0: u8, a1: u8) -> f32 = crate::tu::adapt::gm_8017EC50__df88ed;
    fn gm_8017ECA0(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017ECA0__c89fbd;
    fn gm_8017ECD4(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017ECD4__e2693d;
    fn gm_8017ED08(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::adapt::gm_8017ED08__4f54e0;
    fn gm_8017ED3C(a0: u8, a1: u8) -> f32 = crate::tu::adapt::gm_8017ED3C__2915b5;
    fn gm_8017ED8C(a0: u8, a1: u8) -> f32 = crate::tu::adapt::gm_8017ED8C__49f083;
    fn gm_801B06B0(a0: *mut CSSData, a1: u8, a2: i8, a3: i8, a4: i8, a5: u8, a6: u8, a7: u8) = crate::tu::adapt::gm_801B06B0__3a08bf;
    fn gm_801B0730(a0: *mut CSSData, a1: *mut i8, a2: *mut u8, a3: *mut u8, a4: *mut u8, a5: *mut u8);
    fn gm_CKindToSelKind(a0: u8) -> u8 = crate::tu::adapt::gm_CKindToSelKind__a9d153;
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_GetAllStarData() -> *mut UnkAllstarData = crate::tu::adapt::gm_GetAllStarData__0c7e4e;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
    fn gm_IsCKindUnlocked(a0: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
    fn gm_LoadRumbleEnabled(a0: *mut StartMeleeData) = crate::tu::adapt::gm_LoadRumbleEnabled__62a06a;
    fn gm_SetGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetGameModeStateId__147996;
    fn gm_SetNewGameModePending() = crate::tu::adapt::gm_SetNewGameModePending__1e5e48;
    fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
    fn gm_SetPendingGameMode(a0: u8) = crate::tu::adapt::gm_SetPendingGameMode__90d26d;
    fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026E84__5dcc27;
    fn lbAudioAx_80026EBC(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026EBC__431bc0;
    fn lbAudioAx_80026F2C(a0: u32) = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
    fn lbAudioAx_8002702C(a0: u32, a1: u64) = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
    fn lbAudioAx_80027168() = crate::tu::adapt::lbAudioAx_80027168__342616;
    fn lbDvd_80017700(a0: i32) = crate::tu::adapt::lbDvd_80017700__5067b6;
    fn lbDvd_80018254() = crate::tu::adapt::lbDvd_80018254__6be579;
    fn lbDvd_80018C2C(a0: u8) = crate::tu::adapt::lbDvd_80018C2C__30bf87;
    fn lbDvd_80018C6C() = crate::tu::adapt::lbDvd_80018C6C__4e040c;
    fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990;
    fn lbDvd_SetupVsPreloadCache() = crate::tu::adapt::lbDvd_SetupVsPreloadCache__d38cf3;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn gm_8017EB3C(a0: u8, a1: u8) -> u8 { super::gm_8017EB3C(a0, a1) }
        pub unsafe extern "C" fn gm_8017EB64(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017EB64(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017EB98(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017EB98(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017EBCC(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017EBCC(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017EC00(a0: u8, a1: u8) -> f32 { super::gm_8017EC00(a0, a1) }
        pub unsafe extern "C" fn gm_8017EC50(a0: u8, a1: u8) -> f32 { super::gm_8017EC50(a0, a1) }
        pub unsafe extern "C" fn gm_8017ECA0(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017ECA0(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017ECD4(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017ECD4(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017ED08(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8017ED08(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8017ED3C(a0: u8, a1: u8) -> f32 { super::gm_8017ED3C(a0, a1) }
        pub unsafe extern "C" fn gm_8017ED8C(a0: u8, a1: u8) -> f32 { super::gm_8017ED8C(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    // gm_17c0.c defines them (CSSData, DebugGameOverData, MatchExitInfo, StartMeleeData): the 8-byte stand-ins here
    // were overrun by gm_801B06B0's CSSData writes (the scene table points at them as each state's enter/exit data).
    pub use crate::tu::gm_17c0::{gmClassic_80470708, gmClassic_80470850, gmClassic_8047086C, gmClassic_80472AF8};
}
// END GENERATED EXT

/// `gmClassicMatchupData`: a matchup plus 2 bytes of padding (6 -> 8).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmClassicMatchupData {
    pub x00: gmClassicMatchup,
    pub pad_06: [u8; 2],
}

melee_core::gc_layout!(gmClassicMatchupData, "gmClassicMatchupData", size 8, align 2, [
    x00 @ 0x0: 6 "gmClassicMatchup",
    pad_06 @ 0x6: 2 "u8[2]",
]);

/// `gmClassicIntroData` (0x20): the intro scene's enter data.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmClassicIntroData {
    pub x00: i32,
    pub x04: i32,
    pub x08: u8,
    pub x09: u8,
    pub x0A: u8,
    pub x0B: u8,
    pub x0C: u8,
    pub x0D: [u8; 3],
    pub x10: [u8; 3],
    pub x13: [u8; 3],
    pub x16: [u8; 3],
    pub x19: [u8; 3],
    pub x1C: [u8; 3],
    pub x1F: u8,
}

melee_core::gc_layout!(gmClassicIntroData, "gmClassicIntroData", size 0x20, align 4, [
    x00 @ 0x0: 4 "s32",
    x04 @ 0x4: 4 "s32",
    x08 @ 0x8: 1 "u8",
    x09 @ 0x9: 1 "u8",
    x0A @ 0xA: 1 "u8",
    x0B @ 0xB: 1 "u8",
    x0C @ 0xC: 1 "u8",
    x0D @ 0xD: 3 "u8[3]",
    x10 @ 0x10: 3 "u8[3]",
    x13 @ 0x13: 3 "u8[3]",
    x16 @ 0x16: 3 "u8[3]",
    x19 @ 0x19: 3 "u8[3]",
    x1C @ 0x1C: 3 "u8[3]",
    x1F @ 0x1F: 1 "u8",
]);

/// `gmClassic_804908A0Data` (union, 0x70): `bytes[0x70]`, or the five order tables at +0x00 (12),
/// +0x0C (0x28), +0x34 (0x20), +0x54 (0x0C), +0x60 (0x10).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmClassic_804908A0Data {
    pub bytes: [u8; 0x70],
}

melee_core::gc_layout!(gmClassic_804908A0Data, "gmClassic_804908A0Data", size 0x70, align 1, [
    bytes @ 0x0: 0x70 "u8[0x70]",
]);

/// `gmClassicRuntimeData` (0x90): intro data then the matchup-order state.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmClassicRuntimeData {
    pub intro: gmClassicIntroData,
    pub state: gmClassic_804908A0Data,
}

melee_core::gc_layout!(gmClassicRuntimeData, "gmClassicRuntimeData", size 0x90, align 4, [
    intro @ 0x0: 0x20 "gmClassicIntroData",
    state @ 0x20: 0x70 "gmClassic_804908A0Data",
]);

/// `gmClassic_803DDEC8Data` (0x2F0 on the console; the pointer in each `gm_803DDEC8Struct` makes the
/// native record larger, the field order is the same).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gmClassic_803DDEC8Data {
    pub x00: [gm_803DDEC8Struct; 12],
    pub x0C0: [gmClassicMatchup; 2],
    pub x0CC: [gmClassicMatchup; 39],
    pub pad_1B6: [u8; 2],
    pub x1B8: [gmClassicMatchup; 30],
    pub x26C: [gmClassicMatchup; 11],
    pub pad_2AE: [u8; 2],
    pub x2B0: [gmClassicMatchup; 10],
    pub pad_2EC: [u8; 4],
}

melee_core::gc_layout!(gmClassic_803DDEC8Data, "gmClassic_803DDEC8Data", size 0x2F0, align 4, [
    x00 @ 0x0: 0xC0 "gm_803DDEC8Struct[12]",
    x0C0 @ 0xC0: 12 "gmClassicMatchup[2]",
    x0CC @ 0xCC: 0xEA "gmClassicMatchup[39]",
    pad_1B6 @ 0x1B6: 2 "u8[2]",
    x1B8 @ 0x1B8: 0xB4 "gmClassicMatchup[30]",
    x26C @ 0x26C: 0x42 "gmClassicMatchup[11]",
    pad_2AE @ 0x2AE: 2 "u8[2]",
    x2B0 @ 0x2B0: 0x3C "gmClassicMatchup[10]",
    pad_2EC @ 0x2EC: 4 "u8[4]",
]);

const fn mt(x00: u16, x02: [u8; 3], x05: u8) -> gmClassicMatchup {
    gmClassicMatchup { x00, x02, x05 }
}

/// `gmClassicIntroDataBuffer` + `gm_804908A0`, as one object (see the module comment).
pub static mut gmClassicRuntimeBuf: gmClassicRuntimeData = unsafe { core::mem::zeroed() };
/// `UNK_T gmClassic_804D68D0`
pub static mut gmClassic_804D68D0: *mut c_void = core::ptr::null_mut();

/// `gmClassicIntroDataBuffer`
#[inline(always)]
pub unsafe fn gmClassicIntroDataBuffer() -> *mut gmClassicIntroData {
    melee_core::ct!("gmClassicIntroDataBuffer");
    &raw mut gmClassicRuntimeBuf.intro
}

/// `gm_804908A0` (u8[112]): the order-state bytes.
#[inline(always)]
pub unsafe fn gm_804908A0() -> *mut u8 {
    melee_core::ct!("gm_804908A0");
    (&raw mut gmClassicRuntimeBuf.state) as *mut u8
}

/// `gm_804D4318` / `gm_804D4320` / `gm_804D4328`
pub static mut gm_804D4318: gmClassicMatchupData = gmClassicMatchupData { x00: mt(0x148, [0x21, 0x21, 0x21], 0), pad_06: [0; 2] };
pub static mut gm_804D4320: gmClassicMatchupData = gmClassicMatchupData { x00: mt(0x052, [0x21, 0x21, 0x21], 0), pad_06: [0; 2] };
pub static mut gm_804D4328: gmClassicMatchupData = gmClassicMatchupData { x00: mt(0x053, [0x21, 0x21, 0x21], 0), pad_06: [0; 2] };

/// `static gmClassic_803DDEC8Data gmClassic_803DDEC8`: the per-step scene rows and the five
/// opponent pools (terminated by x00 == 0x148).
pub static mut gmClassic_803DDEC8: gmClassic_803DDEC8Data = gmClassic_803DDEC8Data {
        x00: [
            gm_803DDEC8Struct { x0: 0, x1: 0, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 2, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 1, x1: 16, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 2, x1: 128, x2: 1, pad03: 0, x4: 120, x6: 0, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 3, x1: 0, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 2, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 4, x1: 2, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 5, x1: 128, x2: 3, pad03: 0, x4: 120, x6: 0, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 6, x1: 0, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 2, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 7, x1: 8, x2: 0, pad03: 0, x4: 300, x6: 10, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 8, x1: 128, x2: 2, pad03: 0, x4: 180, x6: 0, x8: 4, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 9, x1: 4, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 2, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 10, x1: 32, x2: 0, pad03: 0, x4: 300, x6: 0, x8: 2, pad09: [0; 3], xC: core::ptr::null_mut() },
            gm_803DDEC8Struct { x0: 13, x1: 0, x2: 0, pad03: 0, x4: 0, x6: 0, x8: 0, pad09: [0; 3], xC: core::ptr::null_mut() },
        ],
        x0C0: [
            mt(176, [30, 26, 33], 0),
            mt(328, [0, 0, 0], 0),
        ],
        x0CC: [
            mt(86, [8, 33, 33], 0),
            mt(87, [8, 33, 33], 0),
            mt(88, [1, 33, 33], 0),
            mt(89, [1, 33, 33], 0),
            mt(90, [6, 33, 33], 0),
            mt(91, [6, 33, 33], 0),
            mt(92, [16, 33, 33], 0),
            mt(93, [16, 33, 33], 0),
            mt(94, [17, 33, 33], 0),
            mt(95, [17, 33, 33], 0),
            mt(96, [4, 33, 33], 0),
            mt(97, [4, 33, 33], 0),
            mt(98, [2, 33, 33], 0),
            mt(99, [2, 33, 33], 0),
            mt(100, [13, 33, 33], 0),
            mt(101, [7, 33, 33], 0),
            mt(102, [7, 33, 33], 0),
            mt(103, [0, 33, 33], 0),
            mt(104, [0, 33, 33], 0),
            mt(105, [11, 33, 33], 0),
            mt(106, [11, 33, 33], 0),
            mt(107, [15, 33, 33], 0),
            mt(108, [5, 33, 33], 0),
            mt(109, [5, 33, 33], 0),
            mt(110, [12, 33, 33], 0),
            mt(111, [12, 33, 33], 0),
            mt(112, [18, 33, 33], 0),
            mt(113, [9, 33, 33], 0),
            mt(114, [10, 33, 33], 0),
            mt(115, [10, 33, 33], 0),
            mt(116, [14, 33, 33], 0),
            mt(117, [14, 33, 33], 0),
            mt(118, [22, 33, 33], 0),
            mt(119, [21, 33, 33], 0),
            mt(120, [21, 33, 33], 0),
            mt(121, [20, 33, 33], 0),
            mt(122, [20, 33, 33], 0),
            mt(124, [24, 33, 33], 0),
            mt(328, [0, 0, 0], 0),
        ],
        pad_1B6: [0; 2],
        x1B8: [
            mt(125, [8, 5, 33], 0),
            mt(126, [8, 12, 33], 0),
            mt(127, [1, 2, 33], 0),
            mt(128, [6, 18, 33], 0),
            mt(129, [6, 21, 33], 0),
            mt(130, [6, 7, 33], 0),
            mt(131, [9, 6, 33], 0),
            mt(132, [16, 0, 33], 0),
            mt(133, [16, 2, 33], 0),
            mt(134, [17, 7, 33], 0),
            mt(135, [17, 11, 33], 0),
            mt(136, [4, 13, 33], 0),
            mt(137, [4, 24, 33], 0),
            mt(138, [4, 15, 33], 0),
            mt(139, [4, 14, 33], 0),
            mt(140, [2, 20, 33], 0),
            mt(141, [2, 0, 33], 0),
            mt(142, [13, 24, 33], 0),
            mt(143, [13, 15, 33], 0),
            mt(144, [7, 22, 33], 0),
            mt(145, [11, 12, 33], 0),
            mt(146, [11, 10, 33], 0),
            mt(147, [0, 20, 33], 0),
            mt(148, [5, 10, 33], 0),
            mt(149, [5, 12, 33], 0),
            mt(150, [5, 18, 33], 0),
            mt(151, [12, 18, 33], 0),
            mt(152, [18, 21, 33], 0),
            mt(153, [18, 9, 33], 0),
            mt(328, [0, 0, 0], 0),
        ],
        x26C: [
            mt(155, [8, 33, 33], 0),
            mt(156, [1, 33, 33], 0),
            mt(157, [6, 33, 33], 0),
            mt(158, [17, 33, 33], 0),
            mt(159, [7, 33, 33], 0),
            mt(160, [0, 33, 33], 0),
            mt(161, [15, 33, 33], 0),
            mt(162, [5, 33, 33], 0),
            mt(163, [22, 33, 33], 0),
            mt(164, [21, 33, 33], 0),
            mt(328, [0, 0, 0], 0),
        ],
        pad_2AE: [0; 2],
        x2B0: [
            mt(165, [8, 8, 8], 0),
            mt(166, [1, 1, 1], 0),
            mt(167, [4, 4, 4], 0),
            mt(168, [7, 7, 7], 0),
            mt(169, [11, 11, 11], 0),
            mt(170, [15, 15, 15], 0),
            mt(174, [0, 0, 0], 0),
            mt(172, [24, 24, 24], 0),
            mt(173, [3, 3, 3], 0),
            mt(328, [0, 0, 0], 0),
        ],
        pad_2EC: [0; 4],
};

/// `gm_Mode_Classic_States` (26 rows: the 11 intro / VS pairs, coming soon, game over, CSS, end).
pub static mut gm_Mode_Classic_States: [GameModeState; 26] = unsafe {
    macro_rules! pair {
        ($id:expr) => {
            [
                state($id, lbDvdPreload_3, Some(gmClassic_801B3500), None, GS_INTRO_EASY,
                      &raw mut gmClassicRuntimeBuf.intro as *mut c_void, &raw mut gmClassic_804D68D0 as *mut c_void),
                state($id + 1, lbDvdPreload_3, Some(gmClassic_801B3A34), Some(gmClassic_801B3B40), GS_VS,
                      &raw mut ext::gmClassic_80472AF8 as *mut c_void, &raw mut ext::gmClassic_8047086C as *mut c_void),
            ]
        };
    }
    let p0 = pair!(0);
    let p1 = pair!(8);
    let p2 = pair!(16);
    let p3 = pair!(24);
    let p4 = pair!(32);
    let p5 = pair!(40);
    let p6 = pair!(48);
    let p7 = pair!(56);
    let p8 = pair!(64);
    let p9 = pair!(72);
    let p10 = pair!(80);
    [
        p0[0], p0[1], p1[0], p1[1], p2[0], p2[1], p3[0], p3[1], p4[0], p4[1], p5[0], p5[1], p6[0], p6[1],
        p7[0], p7[1], p8[0], p8[1], p9[0], p9[1], p10[0], p10[1],
        state(104, lbDvdPreload_3, None, Some(gmClassic_801B3F18), GS_COMING_SOON, NULL, NULL),
        state(105, lbDvdPreload_3, Some(gmClassic_801B3D44), Some(gmClassic_801B3D84), GS_GAMEOVER,
              &raw mut ext::gmClassic_80470850 as *mut c_void, &raw mut ext::gmClassic_80470850 as *mut c_void),
        state(112, lbDvdPreload_3, Some(gmClassic_801B3DD8), Some(gmClassic_801B3E44), GS_CSS,
              &raw mut ext::gmClassic_80470708 as *mut c_void, &raw mut ext::gmClassic_80470708 as *mut c_void),
        end(),
    ]
};

/// Number of entries before the 0x148 terminator.
#[inline]
unsafe fn matchup_count(matchups: *const gmClassicMatchup) -> i32 {
    let mut count = 0;
    while (*matchups.add(count as usize)).x00 != 0x148 {
        count += 1;
    }
    count
}

/// `gmClassic_InitMatchupOrder` (static inline): the identity order of `matchups`, shuffled
/// into `runtime[..][order_offset]` by `HSD_Randi`.
pub unsafe fn gmClassic_InitMatchupOrder(
    matchups: *const gmClassicMatchup,
    order: *mut u8,
    runtime: *mut u8,
    order_offset: i32,
) {
    melee_core::ct!("gmClassic_InitMatchupOrder");
    let mut count = 0;
    let mut m = matchups;
    while (*m).x00 != 0x148 {
        m = m.add(1);
        count += 1;
    }

    for i in 0..count {
        *order.add(i as usize) = i as u8;
    }

    for i in 0..count {
        let swap_idx = ext::HSD_Randi(count);
        let swap = runtime.offset(swap_idx as isize);
        let cur = order.add(i as usize);
        let tmp = *cur;
        *cur = *swap.offset(order_offset as isize);
        *swap.offset(order_offset as isize) = tmp;
    }
}

/// 1B2BA4: the first pool entry whose characters are unlocked, are not the player's, and are
/// not already used by a row of `arg2`; failing that, the last entry that shared a stage.
pub unsafe fn gmClassic_801B2BA4(
    arg0: *mut gmClassicMatchup,
    arg1: *const u8,
    arg2: *mut gm_803DDEC8Struct,
) -> *mut gmClassicMatchup {
    melee_core::ct!("gmClassic_801B2BA4");
    let mut result: *mut gmClassicMatchup = core::ptr::null_mut();
    let target_char = (*ext::gmMainLib_8015CDC8()).c_kind as i32;

    let mut outer = 0;
    while outer < matchup_count(arg0) {
        'next: {
            let entry = arg0.add(*arg1.add(outer as usize) as usize);
            for j in 0..3 {
                let cur_char = (*entry).x02[j] as i8 as i32;

                if cur_char == ChKind_None as i32 {
                    continue;
                }
                if ext::gm_80164430((*entry).x00) == 0 {
                    break 'next;
                }
                if ext::gm_IsCKindUnlocked(cur_char as u8) == 0 {
                    break 'next;
                }
                if cur_char == target_char {
                    break 'next;
                }

                let mut temp_idx = 0usize;
                while (*arg2.add(temp_idx)).x0 != 0xD {
                    for k in 0..3 {
                        let xc = (*arg2.add(temp_idx)).xC as *mut gmClassicMatchup;
                        if !xc.is_null() && cur_char == (*xc).x02[k] as i8 as i32 {
                            break 'next;
                        }
                    }
                    temp_idx += 1;
                }

                temp_idx = 0;
                while (*arg2.add(temp_idx)).x0 != 0xD {
                    let xc = (*arg2.add(temp_idx)).xC as *mut gmClassicMatchup;
                    if !xc.is_null() {
                        if ext::Stage_8022519C((*xc).x00 as u32) == ext::Stage_8022519C((*entry).x00 as u32) {
                            result = entry;
                            break 'next;
                        }
                    }
                    temp_idx += 1;
                }
            }
            if !entry.is_null() {
                return entry;
            }
        }
        outer += 1;
    }

    if !result.is_null() {
        return result;
    }
    core::ptr::null_mut()
}

/// 1B2D54: picks each row's opponent matchup; returns the row that ends the table (or the 0x20 row).
pub unsafe fn gmClassic_801B2D54(arg0: *mut gm_803DDEC8Struct) -> *mut gm_803DDEC8Struct {
    melee_core::ct!("gmClassic_801B2D54");
    let o = &raw mut gmClassicRuntimeBuf;
    let order = (&raw mut (*o).state) as *mut u8;
    let mut ptr = arg0;
    let matchups = &raw mut gmClassic_803DDEC8;

    while (*ptr).x0 != 0xD {
        if (*ptr).x1 & 8 != 0 {
            let result = gmClassic_801B2BA4((&raw mut (*matchups).x2B0) as *mut gmClassicMatchup, order.add(0x60), arg0);
            if !result.is_null() {
                (*ptr).xC = result as *mut _;
            } else {
                loop {}
            }
        }
        ptr = ptr.add(1);
    }

    ptr = arg0;
    while (*ptr).x0 != 0xD {
        let flags = (*ptr).x1;
        if (flags & 2 != 0) && (flags & 0x20 == 0) {
            let result = gmClassic_801B2BA4((&raw mut (*matchups).x26C) as *mut gmClassicMatchup, order.add(0x54), arg0);
            if !result.is_null() {
                (*ptr).xC = result as *mut _;
            } else {
                loop {}
            }
        }
        ptr = ptr.add(1);
    }

    ptr = arg0;
    while (*ptr).x0 != 0xD {
        let flags = (*ptr).x1;
        if (flags & 0x10 != 0) && (flags & 0x20 == 0) {
            let result = gmClassic_801B2BA4((&raw mut (*matchups).x1B8) as *mut gmClassicMatchup, order.add(0x34), arg0);
            if !result.is_null() {
                (*ptr).xC = result as *mut _;
            } else {
                loop {}
            }
        }
        ptr = ptr.add(1);
    }

    ptr = arg0;
    while (*ptr).x0 != 0xD {
        let flags = (*ptr).x1;
        if flags == 0 || flags == 4 {
            let result = gmClassic_801B2BA4((&raw mut (*matchups).x0CC) as *mut gmClassicMatchup, order.add(0x0C), arg0);
            if !result.is_null() {
                (*ptr).xC = result as *mut _;
            } else {
                loop {}
            }
        }
        ptr = ptr.add(1);
    }

    ptr = arg0;
    while (*ptr).x0 != 0xD {
        if (*ptr).x1 == 0x80 {
            let x2val = (*ptr).x2;
            match x2val as i32 {
                1 => (*ptr).xC = (&raw mut gm_804D4318.x00) as *mut _,
                2 => (*ptr).xC = (&raw mut gm_804D4320.x00) as *mut _,
                3 => (*ptr).xC = (&raw mut gm_804D4328.x00) as *mut _,
                _ => {}
            }
        }
        ptr = ptr.add(1);
    }

    ptr = arg0;
    while (*ptr).x0 != 0xD {
        if (*ptr).x1 & 0x20 != 0 {
            (*ptr).xC = (&raw mut (*matchups).x0C0) as *mut _;
            return ptr;
        }
        ptr = ptr.add(1);
    }
    ptr
}

/// `gm_OnePSceneDirect` (pc): `MELEE_SCENE="mode=classic;p1=..."` skips the character select.
pub unsafe fn gm_OnePSceneDirect(s: *mut gmm_x0_528_t, d: *mut Unk1PData, game_mode: i32) -> i32 {
    melee_core::ct!("gm_OnePSceneDirect");
    if ext::SceneLaunch_OnePDirect(game_mode) == 0 {
        return 0;
    }
    let color = ext::SceneLaunch_PlayerColor(0);
    let stocks = ext::SceneLaunch_PlayerStocks(0);
    (*s).c_kind = ext::SceneLaunch_PlayerCKind(0) as i8;
    (*s).color = (if color < 0 { 0 } else { color }) as u8;
    (*s).stocks = (if stocks > 0 { stocks } else { 3 }) as u8;
    (*s).cpu_level = ext::SceneLaunch_OnePLevel() as u8;
    (*s).x5 = ext::SceneLaunch_OnePStep() as u8;
    (*d).x0.ckind = (*s).c_kind;
    (*d).x0.color = (*s).color;
    (*d).x0.cpu_level = (*s).cpu_level;
    (*d).x0.stocks = (*s).stocks;
    (*d).x0.nametag = (*s).nametag;
    (*ext::lbDvd_GetPreloadCacheScene()).game_cache.entries[0].char_id = (*s).c_kind as i32;
    (*ext::lbDvd_GetPreloadCacheScene()).game_cache.entries[0].color = (*s).color;
    ext::lbDvd_SetupVsPreloadCache();
    1
}

/// `gm_Mode_Classic_OnLoad`
pub fn gm_Mode_Classic_OnLoad() {
    melee_core::ct!("gm_Mode_Classic_OnLoad");
    unsafe {
        let o = &raw mut gmClassicRuntimeBuf;
        let m = &raw mut gmClassic_803DDEC8;
        let o_bytes = o as *mut u8;

        let mut entry = (&raw mut (*m).x00) as *mut gm_803DDEC8Struct;
        while (*entry).x0 != 0x0D {
            (*entry).xC = core::ptr::null_mut();
            entry = entry.add(1);
        }

        gmClassic_InitMatchupOrder((&raw mut (*m).x2B0) as *const gmClassicMatchup, gm_804908A0().add(0x60), o_bytes, 0x80);
        gmClassic_InitMatchupOrder((&raw mut (*m).x26C) as *const gmClassicMatchup, gm_804908A0().add(0x54), o_bytes, 0x74);
        gmClassic_InitMatchupOrder((&raw mut (*m).x1B8) as *const gmClassicMatchup, gm_804908A0().add(0x34), o_bytes, 0x54);
        gmClassic_InitMatchupOrder((&raw mut (*m).x0CC) as *const gmClassicMatchup, gm_804908A0().add(0x0C), o_bytes, 0x2C);

        let data = ext::gm_GetAllStarData();
        ext::gmMainLib_8015CDC8();
        ext::gm_8017C984(data as *mut c_void);

        {
            let mut p = (&raw mut (*o).state) as *mut u8;
            for _ in 0..12 {
                *p = 0;
                p = p.add(1);
            }
        }

        ext::gm_8017DB58((*data).x0.xC.x24.as_mut_ptr());
        (*data).x0.x0.slot = ext::gm_801677F0();
        (*data).x0.x48 = Some(ext::cb::gm_8017EB3C);
        (*data).x0.x4C = Some(ext::cb::gm_8017EB64);
        (*data).x0.x50 = Some(ext::cb::gm_8017EBCC);
        (*data).x0.x54 = Some(ext::cb::gm_8017EB98);
        (*data).x0.x64 = Some(ext::cb::gm_8017ED3C);
        (*data).x0.x68 = Some(ext::cb::gm_8017ED8C);
        (*data).x0.x58 = Some(ext::cb::gm_8017ECA0);
        (*data).x0.x5C = Some(ext::cb::gm_8017ED08);
        (*data).x0.x60 = Some(ext::cb::gm_8017ECD4);
        (*data).x0.x6C = Some(ext::cb::gm_8017EC00);
        (*data).x0.x70 = Some(ext::cb::gm_8017EC50);

        ext::gm_SetGameModeStateId(0x70);
        ext::gm_80172174();
        ext::Ground_801C5A28();
        if gm_OnePSceneDirect(ext::gmMainLib_8015CDC8(), &raw mut (*data).x0, GM_CLASSIC as i32) != 0 {
            gmClassic_801B2D54((&raw mut (*m).x00) as *mut gm_803DDEC8Struct);
            ext::gm_80168F88();
            ext::gm_SetGameModeStateId(((*ext::gmMainLib_8015CDC8()).x5 << 3) as u8);
        }
    }
}

/// `gm_Mode_Classic_OnInit`
pub fn gm_Mode_Classic_OnInit() {
    melee_core::ct!("gm_Mode_Classic_OnInit");
    unsafe {
        let temp_r3 = ext::gmMainLib_8015CDC8();
        (*temp_r3).c_kind = ChKind_None as i8;
        (*temp_r3).color = 0;
        (*temp_r3).stocks = 3;
        (*temp_r3).cpu_level = 0;
        (*temp_r3).nametag = 0x78;
        (*temp_r3).x5 = 0;
    }
}

/// `gmClassic_GetStKind` (static inline)
pub unsafe fn gmClassic_GetStKind(entry: *mut gm_803DDEC8Struct, ad: *mut UnkAllstarData) -> StKind {
    melee_core::ct!("gmClassic_GetStKind");
    if (*entry).x1 == 0x80 && (*entry).x2 == 1 {
        return ext::gm_801647F8((*ad).x0.x0.ckind as u8) as u16 as StKind;
    }
    (*((*entry).xC as *mut gmClassicMatchup)).x00 as StKind
}

/// 1B3500: the intro scene's enter handler
pub unsafe extern "C" fn gmClassic_801B3500(arg0: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3500");
    let sd = ext::gm_GetGameModeStateEnterData(arg0) as *mut gmClassicIntroData;
    let m = &raw mut gmClassic_803DDEC8;
    let idx = ext::gm_8017BE84((*arg0).id as u32) as u8;
    let entry = ((&raw mut (*m).x00) as *mut gm_803DDEC8Struct).add(idx as usize);
    let ad = ext::gm_GetAllStarData();
    let mut enemy_count: i32 = 0;
    (*ad).x0.x0.mode = (*arg0).id;
    (*sd).x0A = (*entry).x0.wrapping_add(1);
    (*sd).x08 = (*ad).x0.x0.slot;
    let xc = (*entry).xC as *mut gmClassicMatchup;

    if (*entry).x1 & 0x80 != 0 {
        (*sd).x00 = 3;
    } else if (*entry).x1 & 8 != 0 {
        (*sd).x00 = 4;
    } else if (*entry).x1 & 0x10 != 0 {
        (*sd).x00 = 1;
    } else if (*entry).x1 & 2 != 0 {
        (*sd).x00 = 2;
    } else {
        (*sd).x00 = 0;
    }

    if (*sd).x00 == 3 {
        match (*entry).x2 {
            1 => (*sd).x04 = 1,
            2 => (*sd).x04 = 2,
            3 => (*sd).x04 = 3,
            _ => {}
        }
    } else {
        (*sd).x04 = 0;
    }

    for i in 0..3usize {
        (*sd).x10[i] = (*xc).x02[i];
        (*sd).x16[i] = ext::gm_8017CD94(
            ad as *mut UnkAdventureData,
            (*xc).x02[i] as i8 as i32,
            (*entry).x0 as i32,
            i as i32,
        );
        ext::gmRegSetupEnemyColorTable(
            (*ad).x0.x0.ckind,
            (*ad).x0.x0.color,
            (*xc).x02.as_ptr() as *const i8,
            (*sd).x16.as_mut_ptr(),
        );
        if (*entry).x1 & 4 != 0 {
            (*sd).x1C[i] = 1;
        } else {
            (*sd).x1C[i] = 0;
        }
        if (*sd).x10[i] as i32 != ChKind_None as i32 {
            enemy_count += 1;
        }
    }
    (*sd).x0C = enemy_count as u8;

    let mut ally_count: i32 = 1;
    let ckind = (*ad).x0.x0.ckind;
    if ckind as i32 == CKind_Zelda as i32 && (*ad).x0.xC.x12 != 0 {
        (*sd).x0D[0] = CKind_Seak as u8;
    } else {
        (*sd).x0D[0] = ckind as u8;
    }
    (*sd).x13[0] = (*ad).x0.x0.color;

    ext::gm_8017DB88(
        (*ad).x0.xC.x24.as_mut_ptr() as *mut c_void,
        (*entry).x1,
        (*ad).x0.x0.cpu_level as i32,
        ext::gm_8017BE84((*arg0).id as u32) as u8 as i32,
        (*xc).x02.as_mut_ptr(),
        (*sd).x0D[0],
        core::mem::transmute((*ad).x0.x58),
        core::mem::transmute((*ad).x0.x5C),
        core::mem::transmute((*ad).x0.x60),
        core::mem::transmute((*ad).x0.x6C),
        core::mem::transmute((*ad).x0.x70),
    );

    for i in 1..3usize {
        (*sd).x0D[i] = ext::gm_8017DB6C((*ad).x0.xC.x24.as_mut_ptr() as *mut gm_8017DB6C_arg0_t, i as i32 - 1);
        (*sd).x13[i] = ext::gm_8017DB78((*ad).x0.xC.x24.as_mut_ptr() as *mut gm_8017DB6C_arg0_t, i as i32 - 1);
        if (*sd).x0D[i] as i32 != ChKind_None as i32 {
            ally_count += 1;
        }
    }
    (*sd).x0B = ally_count as u8;
    (*sd).x09 = (*ad).x0.x0.nametag;

    let gc = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    ext::lbDvd_80018C6C();
    let mut count = 0usize;
    (*gc).entries[count].char_id = (*sd).x0D[0] as i32;
    (*gc).entries[count].color = (*ad).x0.x0.color;
    count += 1;
    ext::lbDvd_80018254();
    ext::lbDvd_80018C2C(0xC7);
    ext::lbDvd_80017700(4);

    for i in 0..3usize {
        let echar = (*xc).x02[i] as i8;
        if echar as i32 != ChKind_None as i32 {
            (*gc).entries[count].char_id = echar as i32;
            if (*entry).x1 & 8 != 0 {
                (*gc).entries[count].color = 0xFF;
            } else {
                (*gc).entries[count].color = (*sd).x16[i];
            }
            count += 1;
        }
    }

    for i in 0..3usize {
        let achar = (*ad).x0.xC.x24[i].ckind;
        if achar as i32 != ChKind_None as i32 {
            (*gc).entries[count].char_id = achar as i32;
            (*gc).entries[count].color = (*ad).x0.xC.x24[i].color;
            count += 1;
        }
    }

    ext::lbDvd_80018254();

    if (*entry).x1 == 0x80 && (*entry).x2 == 1 {
        (*gc).stkind = ext::gm_801647F8((*ad).x0.x0.ckind as u8) as u16 as StKind;
    } else if (*entry).x1 == 4 {
        (*gc).stkind = 0xAF;
    } else {
        (*gc).stkind = (*xc).x00 as StKind;
    }

    ext::lbDvd_80018254();

    let mut audio = ext::lbAudioAx_80026E84((*ad).x0.x0.ckind as u32);

    for i in 0..3usize {
        let achar = (*ad).x0.xC.x24[i].ckind;
        if achar as i32 != ChKind_None as i32 {
            audio |= ext::lbAudioAx_80026E84(achar as u32);
        }
    }

    for i in 0..3usize {
        if (*xc).x02[i] as i8 as i32 != ChKind_None as i32 {
            audio |= ext::lbAudioAx_80026E84((*xc).x02[i] as i8 as u32);
            if (*xc).x02[i] as i8 as i32 == CKind_Kirby as i32 {
                audio |= (2u64 << 32) | 0x4000;
            }
        }
    }

    audio |= ext::lbAudioAx_80026EBC(gmClassic_GetStKind(entry, ad) as u32);

    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, audio);
    ext::lbAudioAx_80027168();
}

/// 1B3A34: the VS scene's enter handler
pub unsafe extern "C" fn gmClassic_801B3A34(arg0: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3A34");
    let temp_r30 = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let m = &raw mut gmClassic_803DDEC8;
    let temp_r31 = ((&raw mut (*m).x00) as *mut gm_803DDEC8Struct).add(ext::gm_8017BE84((*arg0).id as u32) as u8 as usize);
    let temp_r29 = ext::gm_GetAllStarData();
    let xc = (*temp_r31).xC as *mut gmClassicMatchup;
    let mut var_r27: i32 = (*xc).x00 as i32;
    if (*temp_r31).x1 == 0x80 && (*temp_r31).x2 == 1 {
        var_r27 = ext::gm_801647F8((*temp_r29).x0.x0.ckind as u8) & 0xFFFF;
    }
    let flags = (*temp_r31).x1;
    if flags == 4 {
        var_r27 = 0xAF;
    }
    (*temp_r29).x0.x8 = flags;
    (*temp_r29).x0.x9 = (*temp_r31).x2;
    (*temp_r29).x0.xB = (*temp_r31).x8;
    let idx_val = (ext::gm_8017BE84((*arg0).id as u32) as u16 as i32) - 1;
    let temp_r28 = *gm_804908A0().offset(idx_val as isize);
    let sp8 = ext::gm_8017BE84((*arg0).id as u32) as u16 as i32;
    let spC = temp_r28 as i32;
    ext::gm_8017CE34(
        temp_r30,
        &raw mut (*temp_r29).x0,
        (*xc).x02.as_mut_ptr() as *mut i8,
        (*temp_r31).x6 as u8,
        1,
        0,
        (*temp_r31).x4 as i32,
        var_r27 as u32,
        sp8,
        spC,
    );
    ext::gm_LoadRumbleEnabled(temp_r30);
}

/// 1B3B40: the VS scene's exit handler (records the result, the all-star clear, the time records)
pub unsafe extern "C" fn gmClassic_801B3B40(arg0: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3B40");
    let mei = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    let asd = ext::gm_GetAllStarData();
    let m = &raw mut gmClassic_803DDEC8;
    let entry = ((&raw mut (*m).x00) as *mut gm_803DDEC8Struct).add(ext::gm_8017BE84((*arg0).id as u32) as u8 as usize);
    let exit_result = (*mei).x8;
    let id = (*arg0).id;
    let idx = (ext::gm_8017BE84(id as u32) as u16 as i32) - 1;
    if exit_result != 0 {
        *gm_804908A0().offset(idx as isize) = 2;
    } else {
        *gm_804908A0().offset(idx as isize) = 1;
    }

    if ext::gm_8017D7AC(mei, &raw mut (*asd).x0, 0x69) != 0 && (*entry.add(1)).x0 == 0xD {
        ext::gm_8017CBAC(asd as *mut UnkAdventureData, ext::gmMainLib_8015CDC8(), 0x15);
    }

    if (*entry).x1 == 0x80 && (*entry).x2 == 1 && ext::Script_OnePScripted() == 0 {
        let mut sp18: i32 = 0;
        let mut sp14: i32 = 0;
        let char_id = ext::gm_CKindToSelKind((*asd).x0.x0.ckind as u8);
        let time_ptr = ext::gmMainLib_8015D438(char_id);
        let best_ptr = ext::gmMainLib_8015D450(char_id);
        ext::Ground_801C1DE4(&mut sp18, &mut sp14);

        if sp18 == 0 {
            let mask: i32 = 1i32.wrapping_shl(char_id as u32);
            if mask & (*ext::gmMainLib_8015EDBC()).x8 == 0 {
                *best_ptr = (*mei).match_end.frame_count as i32;
                (*ext::gmMainLib_8015EDBC()).x8 |= mask;
            } else if (*best_ptr as u32) > (*mei).match_end.frame_count {
                *best_ptr = (*mei).match_end.frame_count as i32;
            }
            (*ext::gmMainLib_8015ED98()).x1C |= mask;
            (*ext::gmMainLib_8015EDB0()).x4 |= mask;
        }

        if ext::gmMainLib_8015D48C(char_id) == 0 {
            if sp18 == 0 {
                ext::gmMainLib_8015D4E8(char_id, 1);
                *time_ptr = (*mei).match_end.frame_count;
                return;
            }
            let diff = (sp14 - sp18) as u32;
            if *time_ptr < diff {
                *time_ptr = (sp14 - sp18) as u32;
            }
        } else if sp18 == 0 {
            if *time_ptr > (*mei).match_end.frame_count {
                *time_ptr = (*mei).match_end.frame_count;
            }
        }
    }
}

/// 1B3D44
pub unsafe extern "C" fn gmClassic_801B3D44(scene: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3D44");
    let temp_r31 = ext::gm_GetGameModeStateEnterData(scene) as *mut DebugGameOverData;
    ext::gm_8017C9A8(temp_r31, &raw mut (*ext::gm_GetAllStarData()).x0, 1);
}

/// 1B3D84
pub unsafe extern "C" fn gmClassic_801B3D84(scene: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3D84");
    let temp_r30 = ext::gm_GetGameModeStateExitData(scene) as *mut DebugGameOverData;
    ext::gm_8017CA38(temp_r30, &raw mut (*ext::gm_GetAllStarData()).x0, ext::gmMainLib_8015CDC8(), 1);
}

/// 1B3DD8
pub unsafe extern "C" fn gmClassic_801B3DD8(scene: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3DD8");
    let css = ext::gm_GetGameModeStateEnterData(scene) as *mut CSSData;
    let temp_r31 = ext::gmMainLib_8015CDC8();
    ext::gm_801B06B0(
        css,
        0xB,
        (*temp_r31).c_kind,
        (*temp_r31).stocks as i8,
        (*temp_r31).color as i8,
        (*temp_r31).nametag,
        (*temp_r31).cpu_level,
        (*ext::gm_GetAllStarData()).x0.x0.slot,
    );
    ext::lbDvd_SetupVsPreloadCache();
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// 1B3E44
pub unsafe extern "C" fn gmClassic_801B3E44(scene: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3E44");
    let temp_r30 = ext::gm_GetGameModeStateExitData(scene) as *mut CSSData;
    let temp_r29 = ext::gmMainLib_8015CDC8();
    let temp_r31 = ext::gm_GetAllStarData();
    let r4 = (&raw mut gmClassic_803DDEC8.x00) as *mut gm_803DDEC8Struct;
    if (*temp_r30).pending_scene_change == 2 {
        // This only happens when, instead of pressing start to begin the game,
        // we press back to exit.
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    ext::gm_801B0730(
        temp_r30,
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
    gmClassic_801B2D54(r4);
    ext::gm_SetNextGameModeStateId(((*temp_r29).x5 << 3) as u8);
    ext::gm_80168F88();
}

/// 1B3F18
pub unsafe extern "C" fn gmClassic_801B3F18(_scene: *mut GameModeState) {
    melee_core::ct!("gmClassic_801B3F18");
    ext::gm_SetPendingGameMode(GM_MENU as u8);
    ext::gm_SetNewGameModePending();
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmclassic.c `gmClassic_GetMatchupCount`: entries up to the 0x148 terminator.
pub fn gmClassic_GetMatchupCount(matchups: &[gmClassicMatchup_gm_ptr]) -> i32 {
    melee_core::ct!("gmClassic_GetMatchupCount");
    let mut count = 0usize;
    while matchups[count].x00 != 0x148 {
        count += 1;
    }
    count as i32
}
