//! Port of src/melee/gm/gmevent.c (Event Match: the CSS / VS scene handlers, the per-event match
//! setup from the `GmEvent.dat` level table, the 51 events' per-frame win / lose procs, and the
//! small event-state accessors). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_gmevent.rs.
//!
//! Two PPC-isms are kept as the C has them: `gm_8016A404((s32) event_info[level]->x4)` passes a
//! function address as a 32-bit int, and the level table's records (`gm_804D6900_t` ...) are
//! native structs here with the GameCube layout in their `GcLayout`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, non_camel_case_types, unused_variables, unused_assignments)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn Camera_SetQuakeScale(a0: f32) = crate::tu::adapt::Camera_SetQuakeScale__cmc;
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn Ground_801C1D84() -> u32 = crate::tu::adapt::Ground_801C1D84__af7526;
    fn HSD_GObjFree(a0: *mut HSD_GObj);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_Randi(a0: i32) -> i32;
    fn Player_80031790(a0: i32) = crate::tu::adapt::Player_80031790__11821c;
    fn Player_80031DA8(a0: i32, a1: i32) = crate::tu::adapt::Player_80031DA8__fbecc7;
    fn Player_800325C8(a0: u32, a1: i32) -> i8 = crate::tu::adapt::Player_800325C8__4ace7f;
    fn Player_80036394(a0: i32) -> u32 = crate::tu::adapt::Player_80036394__f86559;
    fn Player_80036844(a0: i32, a1: i32) = crate::tu::adapt::Player_80036844__2d424f;
    fn Player_GetCoins(a0: i32) -> i32 = crate::tu::adapt::Player_GetCoins__c7d98f;
    fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntity__090538;
    fn Player_GetEntityAtIndex(a0: i32, a1: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntityAtIndex__85ed4e;
    fn Player_GetKOsByPlayerIndex(a0: i32, a1: i32) -> i32 = crate::tu::adapt::Player_GetKOsByPlayerIndex__bd5a79;
    fn Player_GetNametagSlotID(a0: i32) -> u8 = crate::tu::adapt::Player_GetNametagSlotID__655434;
    fn Player_GetP1Stock() -> i32 = crate::tu::adapt::Player_GetP1Stock__f5a98b;
    fn Player_GetPlayerCharacter(a0: i32) -> u32 = crate::tu::adapt::Player_GetPlayerCharacter__fdeb58;
    fn Player_GetPlayerSlotType(a0: i32) -> Gm_PKind = crate::tu::adapt::Player_GetPlayerSlotType__2d05e5;
    fn Player_GetRemainingHP(a0: i32) -> i32 = crate::tu::adapt::Player_GetRemainingHP__d5e70f;
    fn Player_GetStocks(a0: i32) -> i32 = crate::tu::adapt::Player_GetStocks__1c1d51;
    fn Player_SetModelScale(a0: i32, a1: f32) = crate::tu::adapt::Player_SetModelScale__1d6c7a;
    fn Player_SetPlayerAndEntityCpuType(a0: i32, a1: i32) = crate::tu::adapt::Player_SetPlayerAndEntityCpuType__534529;
    fn Player_SetStocks(a0: i32, a1: i32) = crate::tu::adapt::Player_SetStocks__1270f2;
    fn ftBossLib_8015C74C() -> i32 = melee_fighter::tu::ftbosslib::ftBossLib_8015C74C;
    fn ftBossLib_8015C7EC() -> i32 = melee_fighter::tu::ftbosslib::ftBossLib_8015C7EC;
    fn ftBossLib_8015C92C() -> i32 = melee_fighter::tu::ftbosslib::ftBossLib_8015C92C;
    fn ftBossLib_8015CB9C(a0: i32) = crate::tu::adapt::ftBossLib_8015CB9C__c24da3;
    fn ftBossLib_8015CC14() = crate::tu::adapt::ftBossLib_8015CC14__f4f13c;
    fn ftLib_IsSleeping_8008731C(a0: *mut HSD_GObj) -> i32 = crate::tu::adapt::ftLib_IsSleeping_8008731C__becdfc;
    fn ftLib_LoadFighterCostume(a0: i8, a1: u8) = crate::tu::adapt::ftLib_LoadFighterCostume__b45233;
    fn ftLib_GetKind(a0: *mut HSD_GObj) -> u32;
    fn gmFrontend_AtlasSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = crate::tu::adapt::gmFrontend_AtlasSelect__26b7fc;
    fn gmMainLib_8015CEB4(a0: i32) = crate::tu::adapt::gmMainLib_8015CEB4__86b231;
    fn gmMainLib_8015CF5C(a0: i32) -> i32 = crate::tu::adapt::gmMainLib_8015CF5C__8fd145;
    fn gmMainLib_8015CF70(a0: i32, a1: i32) = crate::tu::adapt::gmMainLib_8015CF70__0b6cc3;
    fn gmMainLib_8015CF84() = crate::tu::adapt::gmMainLib_8015CF84__b2804a;
    fn gmVs_GetSceneController() -> *mut VsSceneController = crate::tu::adapt::gmVs_GetSceneController__400cec;
    fn gmVs_GetSceneState() -> *mut VsSceneState = crate::tu::adapt::gmVs_GetSceneState__53d82c;
    fn gm_8016247C(a0: i32) -> i32 = crate::tu::adapt::gm_8016247C__6fb2cc;
    fn gm_80162968(a0: u32) -> i32 = crate::tu::adapt::gm_80162968__8ae6df;
    fn gm_80164504(a0: u16) = crate::tu::adapt::gm_80164504__95f551;
    fn gm_80166378(a0: *mut MatchEnd) = crate::tu::adapt::gm_80166378__34fe0e;
    fn gm_801677F0() -> u8 = crate::tu::adapt::gm_801677F0__be41b7;
    fn gm_80167858(a0: i32, a1: i32, a2: i32, a3: i32) = crate::tu::adapt::gm_80167858__a75f4a;
    fn gm_80168940(a0: *mut MatchEnd) -> i32 = crate::tu::adapt::gm_80168940__6c4a67;
    fn gm_80168F88() = crate::tu::adapt::gm_80168F88__9d899a;
    fn gm_80169384() -> i32 = crate::tu::adapt::gm_80169384__e6b9b1;
    fn gm_8016A164() = crate::tu::adapt::gm_8016A164__b4c98b;
    fn gm_8016A21C(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_8016A21C__1e76b5;
    fn gm_8016A22C(a0: i8, a1: i8, a2: i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: i32, a8: u8, a9: u8, a10: u8, a11: i32, a12: i32, a13: i32, a14: i32, a15: i32, a16: i32, a17: f32, a18: f32) = crate::tu::adapt::gm_8016A22C__e5b47e;
    // the C passes the callback as an s32 (a PPC address); natively it is the 64-bit function pointer
    fn gm_8016A404(a0: crate::tu::gm_16A2::GmEventPlayerInitCallback);
    fn gm_8016A414(a0: f32) = crate::tu::adapt::gm_8016A414__f962ef;
    fn gm_8016A424(a0: i8) = crate::tu::adapt::gm_8016A424__d0ff6a;
    fn gm_8016A434() = crate::tu::adapt::gm_8016A434__76ff7e;
    fn gm_8016A92C(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_8016A92C__fe3fb1;
    fn gm_8016A9E8(a0: u8, a1: i8) -> i32 = crate::tu::adapt::gm_8016A9E8__ea3c0b;
    fn gm_8016AC44(a0: i8, a1: i8) -> i32 = crate::tu::adapt::gm_8016AC44__284fb4;
    fn gm_8016AEEC() -> u32 = crate::tu::adapt::gm_8016AEEC__6820bb;
    fn gm_8016AEFC() -> u16 = crate::tu::adapt::gm_8016AEFC__dd4657;
    fn gm_8016B328() = crate::tu::adapt::gm_8016B328__315877;
    fn gm_8016B33C(a0: i32) = crate::tu::adapt::gm_8016B33C__7b92b1;
    fn gm_8016B350(a0: i32) = crate::tu::adapt::gm_8016B350__3b3894;
    fn gm_8016B364(a0: i32) = crate::tu::adapt::gm_8016B364__eadb9b;
    fn gm_8016B378(a0: i8) = crate::tu::adapt::gm_8016B378__8395d5;
    fn gm_8016B774() -> *mut MatchEnd = crate::tu::adapt::gm_8016B774__ba5c36;
    fn gm_8016EDDC(a0: i32, a1: *mut PlayerInitData) -> i32 = crate::tu::adapt::gm_8016EDDC__aa6fea;
    fn gm_8016F00C(a0: i32) = crate::tu::adapt::gm_8016F00C__e9523d;
    fn gm_80172898(a0: u16) -> i32 = crate::tu::adapt::gm_80172898__ee73a3;
    fn gm_801732D8(a0: u8) -> u8 = crate::tu::adapt::gm_801732D8__45e05d;
    fn gm_8017335C() -> u16 = crate::tu::adapt::gm_8017335C__86aa87;
    fn gm_80173754(a0: u8, a1: u8) -> i32 = crate::tu::adapt::gm_80173754__f83272;
    fn gm_80173D3C(a0: i32) = crate::tu::adapt::gm_80173D3C__7c55ab;
    fn gm_80173EEC() = crate::tu::adapt::gm_80173EEC__58a22d;
    fn gm_801B05F4(a0: *mut PlayerInitData, a1: i32) = crate::tu::adapt::gm_801B05F4__ce3447;
    fn gm_801B06B0(a0: *mut CSSData, a1: u8, a2: i8, a3: i8, a4: i8, a5: u8, a6: u8, a7: u8) = crate::tu::adapt::gm_801B06B0__3a08bf;
    fn gm_801B0730(a0: *mut CSSData, a1: *mut i8, a2: *mut u8, a3: *mut u8, a4: *mut u8, a5: *mut u8);
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_ClearDbPauseFlag(a0: i32) = crate::tu::adapt::gm_ClearDbPauseFlag__57b7e0;
    fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
    fn gm_GetFrameCount() -> u32 = crate::tu::adapt::gm_GetFrameCount__aa8e1a;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
    fn gm_InitChallengerData(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) = crate::tu::adapt::gm_InitChallengerData__c5db86;
    fn gm_LoadRumbleEnabled(a0: *mut StartMeleeData) = crate::tu::adapt::gm_LoadRumbleEnabled__62a06a;
    fn gm_RumbleEnabledForPlayer(a0: i32, a1: i32) -> i32 = crate::tu::adapt::gm_RumbleEnabledForPlayer__aa5085;
    fn gm_SetDbPauseFlag(a0: i32) = crate::tu::adapt::gm_SetDbPauseFlag__fad6e4;
    fn gm_SetGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetGameModeStateId__147996;
    fn gm_SetGameSpeed(a0: f32) = crate::tu::adapt::gm_SetGameSpeed__e4fa3d;
    fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
    fn gm_SetupPlayerDefaults(a0: *mut PlayerInitData) = crate::tu::adapt::gm_SetupPlayerDefaults__e9a9ae;
    fn gm_SetupRulesDefaults(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_SetupRulesDefaults__e9563a;
    fn ifStock_802FA2D0(a0: i32) = crate::tu::adapt::ifStock_802FA2D0__60b4c4;
    fn itEvYoshiEgg_Spawn(a0: *mut Vec) = crate::tu::adapt::itEvYoshiEgg_Spawn__b8d779;
    fn lbArchive_LoadSymbols(a0: *const u8, a1: *mut c_void, a2: *const u8, a3: i32) -> *mut c_void;
    fn lbAudioAx_8002438C(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_8002438C__35c010;
    fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026E84__5dcc27;
    fn lbAudioAx_80026EBC(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026EBC__431bc0;
    fn lbAudioAx_80026F2C(a0: u32) = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
    fn lbAudioAx_8002702C(a0: u32, a1: u64) = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
    fn lbAudioAx_80027168() = crate::tu::adapt::lbAudioAx_80027168__342616;
    fn lbAudioAx_80027648() = crate::tu::adapt::lbAudioAx_80027648__4f40c8;
    fn lbAudioAx_80028B6C() = crate::tu::adapt::lbAudioAx_80028B6C__bb9f49;
    fn lbAudioAx_80028B90() = crate::tu::adapt::lbAudioAx_80028B90__0093c2;
    fn lbBgFlash_8002063C(a0: i32) = crate::tu::adapt::lbBgFlash_8002063C__95303c;
    fn lbBgFlash_80020688(a0: i32) = crate::tu::adapt::lbBgFlash_80020688__4d10b5;
    fn lbBgFlash_SetFlashScale(a0: f32) = crate::tu::adapt::lbBgFlash_SetFlashScale__f5c112;
    fn lbDvd_80017700(a0: i32) = crate::tu::adapt::lbDvd_80017700__5067b6;
    fn lbDvd_80018254() = crate::tu::adapt::lbDvd_80018254__6be579;
    fn lbDvd_80018C2C(a0: u8) = crate::tu::adapt::lbDvd_80018C2C__30bf87;
    fn lbDvd_80018C6C() = crate::tu::adapt::lbDvd_80018C6C__4e040c;
    fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = crate::tu::adapt::lbDvd_GetPreloadCacheScene__1ed990;
    fn pl_8003FBFC(a0: i32) -> i32 = crate::tu::adapt::pl_8003FBFC__27089d;
    fn pl_8003FC20(a0: i32) -> i32 = crate::tu::adapt::pl_8003FC20__860736;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_world::gmmain::gmMainLib_804D3EE0;
}
// END GENERATED EXT

/// `gm_801BAB40_src` (the .c's name for an event's player-init record, 0x1C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_801BAB40_src {
    pub c_kind: i8,
    pub slot_type: u8,
    pub stocks: u8,
    pub color: u8,
    pub x5: u8,
    pub sub_color: u8,
    pub team: u8,
    pub xB: u8,
    pub flags: u8,
    pub xE: u8,
    pub cpu_level: u8,
    pub pad: u8,
    pub x12: u16,
    pub hp: u16,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
}

melee_core::gc_layout!(gm_801BAB40_src, "gm_801BAB40_src", size 0x1C, align 4, [
    c_kind @ 0x0: 1 "s8",
    slot_type @ 0x1: 1 "u8",
    stocks @ 0x2: 1 "u8",
    color @ 0x3: 1 "u8",
    x5 @ 0x4: 1 "u8",
    sub_color @ 0x5: 1 "u8",
    team @ 0x6: 1 "u8",
    xB @ 0x7: 1 "u8",
    flags @ 0x8: 1 "u8",
    xE @ 0x9: 1 "u8",
    cpu_level @ 0xA: 1 "u8",
    pad @ 0xB: 1 "u8",
    x12 @ 0xC: 2 "u16",
    hp @ 0xE: 2 "u16",
    x18 @ 0x10: 4 "f32",
    x1C @ 0x14: 4 "f32",
    x20 @ 0x18: 4 "f32",
]);

/// `gm_event_char_list`: the opponent pool of a random-opponent event (ends at `ChKind_Max`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_event_char_list {
    pub c_kind: [u8; 33],
}

melee_core::gc_layout!(gm_event_char_list, "gm_event_char_list", size 33, align 1, [
    c_kind @ 0x0: 33 "u8[33]",
]);

/// `gm_evinit`: per-level match init data; shares its first two bytes' bitfield layout with
/// `StartMeleeRules` (GameCube bit order, MSB first, as melee_types keeps bitfields).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_evinit {
    /// `x0_0:3 x0_3:3 x0_6:1 x0_7:1 x1_0:1 x1_1:1 x1_2:1 x1_3:1 x1_4:1 x1_5:3`
    pub bf_0: [u8; 2],
    pub is_teams: u8,
    pub item_freq: i8,
    pub sd_penalty: i8,
    pub unk5: u8,
    pub stkind: u16,
    pub time_limit: u32,
    pub padC: [u8; 4],
    pub x10: u64,
    pub x18: i32,
    pub x1C: f32,
    pub game_speed: f32,
    pub unk24: f32,
}

impl gm_evinit {
    pub fn x0_0(&self) -> u32 { bits_get(&self.bf_0, 13, 3) as u32 }
    pub fn set_x0_0(&mut self, v: u32) { bits_set(&mut self.bf_0, 13, 3, (v as u64) & 0x7) }
    pub fn x0_3(&self) -> u32 { bits_get(&self.bf_0, 10, 3) as u32 }
    pub fn set_x0_3(&mut self, v: u32) { bits_set(&mut self.bf_0, 10, 3, (v as u64) & 0x7) }
    pub fn x0_6(&self) -> u32 { bits_get(&self.bf_0, 9, 1) as u32 }
    pub fn set_x0_6(&mut self, v: u32) { bits_set(&mut self.bf_0, 9, 1, (v as u64) & 0x1) }
    pub fn x0_7(&self) -> u32 { bits_get(&self.bf_0, 8, 1) as u32 }
    pub fn set_x0_7(&mut self, v: u32) { bits_set(&mut self.bf_0, 8, 1, (v as u64) & 0x1) }
    pub fn x1_0(&self) -> u32 { bits_get(&self.bf_0, 7, 1) as u32 }
    pub fn set_x1_0(&mut self, v: u32) { bits_set(&mut self.bf_0, 7, 1, (v as u64) & 0x1) }
    pub fn x1_1(&self) -> u32 { bits_get(&self.bf_0, 6, 1) as u32 }
    pub fn set_x1_1(&mut self, v: u32) { bits_set(&mut self.bf_0, 6, 1, (v as u64) & 0x1) }
    pub fn x1_2(&self) -> u32 { bits_get(&self.bf_0, 5, 1) as u32 }
    pub fn set_x1_2(&mut self, v: u32) { bits_set(&mut self.bf_0, 5, 1, (v as u64) & 0x1) }
    pub fn x1_3(&self) -> u32 { bits_get(&self.bf_0, 4, 1) as u32 }
    pub fn set_x1_3(&mut self, v: u32) { bits_set(&mut self.bf_0, 4, 1, (v as u64) & 0x1) }
    pub fn x1_4(&self) -> u32 { bits_get(&self.bf_0, 3, 1) as u32 }
    pub fn set_x1_4(&mut self, v: u32) { bits_set(&mut self.bf_0, 3, 1, (v as u64) & 0x1) }
    pub fn x1_5(&self) -> u32 { bits_get(&self.bf_0, 0, 3) as u32 }
    pub fn set_x1_5(&mut self, v: u32) { bits_set(&mut self.bf_0, 0, 3, (v as u64) & 0x7) }
}

melee_core::gc_layout!(gm_evinit, "gm_evinit", size 0x28, align 8, [
    x0_0 @ 0x0: 4 "u32" bits 0 + 3,
    x0_3 @ 0x0: 4 "u32" bits 3 + 3,
    x0_6 @ 0x0: 4 "u32" bits 6 + 1,
    x0_7 @ 0x0: 4 "u32" bits 7 + 1,
    x1_0 @ 0x0: 4 "u32" bits 8 + 1,
    x1_1 @ 0x0: 4 "u32" bits 9 + 1,
    x1_2 @ 0x0: 4 "u32" bits 10 + 1,
    x1_3 @ 0x0: 4 "u32" bits 11 + 1,
    x1_4 @ 0x0: 4 "u32" bits 12 + 1,
    x1_5 @ 0x0: 4 "u32" bits 13 + 3,
    is_teams @ 0x2: 1 "u8",
    item_freq @ 0x3: 1 "s8",
    sd_penalty @ 0x4: 1 "s8",
    unk5 @ 0x5: 1 "u8",
    stkind @ 0x6: 2 "u16",
    time_limit @ 0x8: 4 "u32",
    padC @ 0xC: 4 "u8[4]",
    x10 @ 0x10: 8 "u64",
    x18 @ 0x18: 4 "s32",
    x1C @ 0x1C: 4 "f32",
    game_speed @ 0x20: 4 "f32",
    unk24 @ 0x24: 4 "f32",
]);

/// `gm_evstage_table`: per-round stages and opponents, for levels with multiple rounds.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_evstage_table {
    pub count: u8,
    pub pad1: u8,
    pub stage: [u16; 7],
    pub entries: [*mut gm_801BAB40_src; 6],
}

melee_core::gc_layout!(gm_evstage_table, "gm_evstage_table", size 0x28, align 4, [
    count @ 0x0: 1 "u8",
    pad1 @ 0x1: 1 "u8",
    stage @ 0x2: 14 "u16[7]",
    entries @ 0x10: 24 "gm_801BAB40_src*[6]",
]);

/// `gm_evbonus`: a bonus-stage level's fixed target and its settings.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_evbonus {
    pub c_kind: i8,
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
    pub x4: u8,
    pub x5: u8,
    pub color: u8,
    pub pad7: u8,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub flags: u8,
    pub x15: u8,
    pub x16: u8,
    pub x17: u8,
}

melee_core::gc_layout!(gm_evbonus, "gm_evbonus", size 0x18, align 4, [
    c_kind @ 0x0: 1 "s8",
    x1 @ 0x1: 1 "u8",
    x2 @ 0x2: 1 "u8",
    x3 @ 0x3: 1 "u8",
    x4 @ 0x4: 1 "u8",
    x5 @ 0x5: 1 "u8",
    color @ 0x6: 1 "u8",
    pad7 @ 0x7: 1 "u8",
    x8 @ 0x8: 4 "f32",
    xC @ 0xC: 4 "f32",
    x10 @ 0x10: 4 "f32",
    flags @ 0x14: 1 "u8",
    x15 @ 0x15: 1 "u8",
    x16 @ 0x16: 1 "u8",
    x17 @ 0x17: 1 "u8",
]);

/// `gm_804D6900_x4_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_804D6900_x4_t {
    pub x0: i32,
    pub x4: isize,
}

melee_core::gc_layout!(gm_804D6900_x4_t, "gm_804D6900_x4_t", size 8, align 4, [
    x0 @ 0x0: 4 "int",
    x4 @ 0x4: 4 "intptr_t",
]);

/// `gm_804D6900_t`: one level of `sqEventInitDataLevelTbl`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_804D6900_t {
    pub kind: u8,
    /// top 3 bits: player count
    pub flags: u8,
    pub pad2: [u8; 2],
    pub x4: *mut gm_804D6900_x4_t,
    pub evinit: *mut gm_evinit,
    pub evbonus: *mut gm_evbonus,
    pub evstage_table: *mut gm_evstage_table,
    /// The C declares `player_init[5]`, but onEnterVs's skip-the-NULLs loop reads `player_init[5]` (the word after the
    /// record in GmEvent.dat) when a 6-player level's fifth entry is NULL (event 37); the sixth slot holds that word
    pub player_init: [*mut gm_801BAB40_src; 6],
}

melee_core::gc_layout!(gm_804D6900_t, "gm_804D6900_t", size 0x2C, align 4, [
    kind @ 0x0: 1 "u8",
    flags @ 0x1: 1 "u8",
    pad2 @ 0x2: 2 "u8[2]",
    x4 @ 0x4: 4 "gm_804D6900_x4_t*",
    evinit @ 0x8: 4 "gm_evinit*",
    evbonus @ 0xC: 4 "gm_evbonus*",
    evstage_table @ 0x10: 4 "gm_evstage_table*",
    player_init @ 0x14: 24 "gm_801BAB40_src*[6]",
]);

/// 497758
pub static mut css_data: CSSData = unsafe { core::mem::zeroed() };
/// 4978A0
pub static mut vs_enter_data: StartMeleeData = unsafe { core::mem::zeroed() };
/// 4979D8
pub static mut vs_exit_data: [MatchExitInfo; 2] = unsafe { core::mem::zeroed() };
/// 4D6900: `GmEvent.dat`'s `sqEventInitDataLevelTbl` (`[0]` is the table of level pointers)
pub static mut gm_804D6900: [*mut *mut gm_804D6900_t; 2] = [core::ptr::null_mut(); 2];

/// `static u8 gm_803DF918[]` (803DF918): event index -> level table index.
pub static gm_803DF918: [u8; 51] = [
    0, 17, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 36, 15, 16,
    1, 18, 19, 20, 21, 26, 23, 24, 25, 22, 27, 28, 29, 30, 31, 32, 33,
    47, 35, 14, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 34, 48, 49, 50,
];

static mut gm_804D4330: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4338: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4340: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4348: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4350: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC9E8), x4: None };
static mut gm_804D4358: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4360: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4368: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4370: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BCAF0), x4: None };
static mut gm_804D4378: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BCF20), x4: None };
static mut gm_804D4380: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BCF40), x4: None };
static mut gm_804D4388: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4390: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD028), x4: None };
static mut gm_804D4398: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D43A0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D43A8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: Some(gm_801BEA10) };
static mut gm_804D43B0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD164), x4: None };
static mut gm_804D43B8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D43C0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD30C), x4: None };
static mut gm_804D43C8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD44C), x4: None };
static mut gm_804D43D0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD46C), x4: None };
static mut gm_804D43D8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD658), x4: Some(gm_801BEA4C) };
static mut gm_804D43E0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D43E8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: unsafe { core::mem::transmute::<unsafe extern "C" fn(i32, i32), Option<unsafe extern "C" fn(i32)>>(gm_801BEA88) } };
static mut gm_804D43F0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD7FC), x4: None };
static mut gm_804D43F8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4400: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4408: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: Some(gm_801BEAF0) };
static mut gm_804D4410: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BD93C), x4: None };
static mut gm_804D4418: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BDAD4), x4: None };
static mut gm_804D4420: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4428: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BDAF4), x4: None };
static mut gm_804D4430: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BDC08), x4: None };
static mut gm_804D4438: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4440: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BDD44), x4: None };
static mut gm_804D4448: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BDE94), x4: None };
static mut gm_804D4450: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4458: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4460: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4468: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BE37C), x4: None };
static mut gm_804D4470: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4478: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4480: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D4488: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BE39C), x4: None };
static mut gm_804D4490: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: Some(gm_801BEB2C) };
static mut gm_804D4498: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D44A0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D44A8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };
static mut gm_804D44B0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BE618), x4: None };
static mut gm_804D44B8: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BE638), x4: None };
static mut gm_804D44C0: gm_803DF94C_t = gm_803DF94C_t { x0: Some(gm_801BC754), x4: None };

/// `gm_803DF94C_t* gm_803DF94C[]` (803DF94C): per-event {frame proc, player-setup callback}.
pub static mut gm_803DF94C: [*mut gm_803DF94C_t; 51] = unsafe {
    [
        &raw mut gm_804D4330, &raw mut gm_804D43B8, &raw mut gm_804D4340, &raw mut gm_804D4348,
        &raw mut gm_804D4350, &raw mut gm_804D4358, &raw mut gm_804D4360, &raw mut gm_804D4368,
        &raw mut gm_804D4370, &raw mut gm_804D4378, &raw mut gm_804D4380, &raw mut gm_804D4388,
        &raw mut gm_804D4390, &raw mut gm_804D4398, &raw mut gm_804D4450, &raw mut gm_804D43A8,
        &raw mut gm_804D43B0, &raw mut gm_804D4338, &raw mut gm_804D43C0, &raw mut gm_804D43C8,
        &raw mut gm_804D43D0, &raw mut gm_804D43D8, &raw mut gm_804D4400, &raw mut gm_804D43E8,
        &raw mut gm_804D43F0, &raw mut gm_804D43F8, &raw mut gm_804D43E0, &raw mut gm_804D4408,
        &raw mut gm_804D4410, &raw mut gm_804D4418, &raw mut gm_804D4420, &raw mut gm_804D4428,
        &raw mut gm_804D4430, &raw mut gm_804D4438, &raw mut gm_804D44A8, &raw mut gm_804D4448,
        &raw mut gm_804D43A0, &raw mut gm_804D4458, &raw mut gm_804D4460, &raw mut gm_804D4468,
        &raw mut gm_804D4470, &raw mut gm_804D4478, &raw mut gm_804D4480, &raw mut gm_804D4488,
        &raw mut gm_804D4490, &raw mut gm_804D4498, &raw mut gm_804D44A0, &raw mut gm_804D4440,
        &raw mut gm_804D44B0, &raw mut gm_804D44B8, &raw mut gm_804D44C0,
    ]
};

/// `gm_Mode_Event_States`
pub static mut gm_Mode_Event_States: [GameModeState; 3] = unsafe {
    [
        state(0, lbDvdPreload_3, Some(onEnterCss), Some(onExitCss), GS_CSS,
              &raw mut css_data as *mut c_void, &raw mut css_data as *mut c_void),
        state(1, lbDvdPreload_3, Some(onEnterVs), Some(onExitVs), GS_VS,
              &raw mut vs_enter_data as *mut c_void, &raw mut vs_exit_data as *mut c_void),
        end(),
    ]
};

/// `gmMainLib_804D3EE0->vs.unk_530`
#[inline(always)]
unsafe fn ev_ptr() -> *mut EventData {
    &raw mut (*ext::gmMainLib_804D3EE0).vs.unk_530
}

#[inline(always)]
unsafe fn levels() -> *mut *mut gm_804D6900_t {
    gm_804D6900[0]
}

/// `gm_GetEventData` (static inline): the event data block inside gmMainLib_804D3EE0.
pub unsafe fn gm_GetEventData() -> *mut EventData {
    melee_core::ct!("gm_GetEventData");
    ev_ptr()
}

/// `gm_GetNextColor` (static inline)
pub fn gm_GetNextColor(color: u8) -> u8 {
    melee_core::ct!("gm_GetNextColor");
    let mut color = color;
    if color <= 2 {
        color += 1;
    } else {
        color = 0;
    }
    color
}

/// 1BA8FC
pub fn gm_801BA8FC() {
    melee_core::ct!("gm_801BA8FC");
    unsafe {
        ext::lbArchive_LoadSymbols(
            c"GmEvent.dat".as_ptr() as *const u8,
            &raw mut gm_804D6900 as *mut c_void,
            c"sqEventInitDataLevelTbl".as_ptr() as *const u8,
            0,
        );
    }
}

/// 1BA938
pub unsafe fn gm_801BA938(arg0: *mut EventData, lo: i32, hi: i32, arg3: i32) {
    melee_core::ct!("gm_801BA938");
    let cache = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    let mut mask: u64;
    let mut char_id: i8;
    let mut i: i32;

    i = lo;
    while i < hi {
        char_id = (*arg0).x4C[i as usize];
        if char_id as i32 != ChKind_None as i32 {
            (*cache).entries[i as usize].char_id = char_id as i32;
            (*cache).entries[i as usize].color = (*arg0).x50[i as usize];
            if (*arg0).x45 != 0 && i > 0 && (*arg0).x4C[i as usize] == 4 {
                (*cache).entries[i as usize].x5 = 0;
            }
        }
        i += 1;
    }

    if arg3 != 0 {
        (*cache).stkind = (*arg0).x48;
    }
    ext::lbDvd_80018254();
    mask = 0;
    i = lo;
    while i < hi {
        if (*arg0).x4C[i as usize] as i32 != ChKind_None as i32 {
            mask |= ext::lbAudioAx_80026E84((*arg0).x4C[i as usize] as u32);
        }
        i += 1;
    }
    mask |= ext::lbAudioAx_80026EBC((*arg0).x48);
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, mask);
    ext::lbAudioAx_80027168();
}

/// 1BAA60
pub unsafe extern "C" fn onEnterCss(arg0: *mut GameModeState) {
    melee_core::ct!("onEnterCss");
    let temp_r31 = ev_ptr();
    let css = ext::gm_GetGameModeStateEnterData(arg0) as *mut CSSData;

    ext::gm_801B06B0(css, 0xE, (*temp_r31).x2, 0, (*temp_r31).x3 as i8, (*temp_r31).nametag, 0, (*temp_r31).x6);
    if (*temp_r31).x44 == 1 {
        gm_801BA938(temp_r31, 1, 4, 1);
    }
    ext::gmFrontend_AtlasSelect(arg0, 0, core::ptr::null());
}

/// 1BAAD0
pub unsafe extern "C" fn onExitCss(arg0: *mut GameModeState) {
    melee_core::ct!("onExitCss");
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gm_GetGameModeStateExitData(arg0) as *mut CSSData;
    if (*temp_r3).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    ext::gm_801B0730(
        temp_r3,
        &raw mut (*temp_r31).x2,
        core::ptr::null_mut(),
        &raw mut (*temp_r31).x3,
        &raw mut (*temp_r31).nametag,
        core::ptr::null_mut(),
    );
    (*temp_r31).x8 = -1;
    (*temp_r31).x9 = -1;
    (*temp_r31).xA = -1;
}

/// 1BAB40
pub unsafe fn gm_801BAB40(arg0: *mut PlayerInitData, src: *mut gm_801BAB40_src) {
    melee_core::ct!("gm_801BAB40");
    ext::gm_SetupPlayerDefaults(arg0);
    (*arg0).ckind = (*src).c_kind;
    (*arg0).slot_type = (*src).slot_type;
    (*arg0).stocks = (*src).stocks as i8;
    (*arg0).color = (*src).color;
    (*arg0).slot = 0;
    (*arg0).spawn_pos = (*src).x5 as i8;
    (*arg0).sub_color = (*src).sub_color;
    (*arg0).handicap = 9;
    (*arg0).team = (*src).team;
    (*arg0).nametag = 0x78;
    (*arg0).xB = (*src).xB;
    (*arg0).set_rumble_enabled(0);
    (*arg0).set_xC_b1(((*src).flags & 0x80) >> 7);
    (*arg0).set_vs_metal(((*src).flags & 0x40) >> 6);
    (*arg0).set_xC_b3(((*src).flags & 0x20) >> 5);
    (*arg0).set_vs_invisible(((*src).flags & 0x10) >> 4);
    (*arg0).set_xC_b6(((*src).flags & 0x08) >> 3);
    (*arg0).set_xC_b7(((*src).flags & 0x04) >> 2);
    (*arg0).set_xD_b1(0);
    (*arg0).set_xD_b2(0);
    (*arg0).set_xD_b4(0);
    (*arg0).cpu_kind = (*src).xE;
    (*arg0).cpu_level = (*src).cpu_level;
    (*arg0).damage = 0;
    (*arg0).damage1 = (*src).x12;
    (*arg0).hp = (*src).hp;
    (*arg0).attack_ratio = (*src).x18;
    (*arg0).defense_ratio = (*src).x1C;
    (*arg0).model_scale = (*src).x20;
}

/// 1BAC9C: a random opponent for slot `arg1`, not among the first `arg1` players' characters
pub unsafe fn gm_801BAC9C(arg0: *mut GameModeState, arg1: i32) -> i32 {
    melee_core::ct!("gm_801BAC9C");
    let mut chars = [0u8; ChKind_Max as usize];
    let ev = ev_ptr();
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let mut count: i32 = 0;
    let src = (*(*levels().add((*ev).unk_535 as usize))).x4 as *mut gm_event_char_list;

    let mut i = 0;
    while i < ChKind_Max as i32 {
        let ptr = &raw mut (*src).c_kind[i as usize];
        let c = *ptr;
        if c as i32 == ChKind_Max as i32 {
            break;
        }
        let mut found = 0;
        let mut k = 0;
        while k < arg1 {
            if (*data).players[k as usize].ckind == *ptr as i8 {
                found += 1;
            }
            k += 1;
        }
        if found == 0 {
            chars[count as usize] = c;
            count += 1;
        }
        i += 1;
    }
    chars[ext::HSD_Randi(count) as usize] as i32
}

/// 1BAD70
pub unsafe extern "C" fn onEnterVs(arg0: *mut GameModeState) {
    melee_core::ct!("onEnterVs");
    let ev = gm_GetEventData();
    let md = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let level = (*ev).unk_535;
    let event_info = &raw mut gm_803DF94C as *mut *mut gm_803DF94C_t;

    ext::lbArchive_LoadSymbols(
        c"GmEvent.dat".as_ptr() as *const u8,
        &raw mut gm_804D6900 as *mut c_void,
        c"sqEventInitDataLevelTbl".as_ptr() as *const u8,
        0,
    );
    let levels = gm_804D6900[0];
    let lvl = *levels.add(level as usize);
    ext::gm_SetupRulesDefaults(&raw mut (*md).rules);
    (*md).rules.set_match_kind((*(*lvl).evinit).x0_0());
    (*md).rules.set_x0_3((*(*lvl).evinit).x0_3());
    (*md).rules.set_timer_enabled((*(*lvl).evinit).x0_6());
    (*md).rules.set_timer_counts_up((*(*lvl).evinit).x0_7());
    (*md).rules.set_x1_0(1);
    (*md).rules.set_x1_1(0);
    (*md).rules.set_x1_2(0);
    (*md).rules.set_x1_3(0);
    (*md).rules.set_timer_shows_hours(0);
    (*md).rules.set_friendly_fire((*(*lvl).evinit).x1_1());
    (*md).rules.set_x2_2(0);
    (*md).rules.set_single_button(0);
    (*md).rules.set_disable_pausing(0);
    (*md).rules.set_x2_5((*(*lvl).evinit).x1_2());
    (*md).rules.set_x3_1(1);
    (*md).rules.set_x3_2(1);
    (*md).rules.set_x3_3(1);
    (*md).rules.set_x3_4(1);
    (*md).rules.set_x3_5(1);
    (*md).rules.set_x3_7(0);
    (*md).rules.set_x4_2(0);
    (*md).rules.set_x4_3(1);
    (*md).rules.set_x4_4(0);
    (*md).rules.set_x4_6(0);
    (*md).rules.set_x4_7(0);
    (*md).rules.set_x5_0(1);
    (*md).rules.set_x5_1(0);
    (*md).rules.set_x5_2((*(*lvl).evinit).x1_3());
    (*md).rules.set_x5_3((*(*lvl).evinit).x1_4());
    (*md).rules.x7 = 0;
    (*md).rules.is_teams = (*(*lvl).evinit).is_teams;
    (*md).rules.x9 = 0;
    (*md).rules.xA = 0;
    (*md).rules.item_freq = (*(*lvl).evinit).item_freq;
    (*md).rules.sd_penalty = (*(*lvl).evinit).sd_penalty;
    (*md).rules.xD = 0x6E;
    (*md).rules.stkind = (*(*lvl).evinit).stkind;
    (*md).rules.time_limit = (*(*lvl).evinit).time_limit;
    (*md).rules.x14 = 0;
    (*md).rules.x18 = 0;
    (*md).rules.x20 = (*(*lvl).evinit).x10;
    (*md).rules.x28 = (*(*lvl).evinit).x18;
    (*md).rules.x30 = (*(*lvl).evinit).x1C;
    (*md).rules.game_speed = (*(*lvl).evinit).game_speed;
    (*md).rules.on_match_start = Some(fn_801BBFE8);
    if (*md).rules.timer_counts_up() & 1 != 0 {
        (*ev).set_xB_0(1);
    }
    if (*(*lvl).evinit).x1_0() != 0 {
        (*ev).set_xB_6(1);
    }
    if (*(*lvl).evinit).unk24 != 1.0f32 {
        (*ev).x1C = (*(*lvl).evinit).unk24;
    }
    if (*lvl).kind == 2 {
        (*ev).set_xB_4(1);
        let stage: u16 = (*(*lvl).evstage_table).stage[(*ev).x20 as usize];
        (*md).rules.stkind = stage;
        (*ev).x48 = stage as StKind;
        if (*ev).x20 > 0 {
            (*md).rules.set_x1_2(1);
            (*md).rules.set_x1_3(1);
            (*md).rules.time_limit = (*ev).x2C as u32;
        }
        ext::gm_8016A92C(&raw mut (*md).rules);
    }

    let mut i = 0;
    while i < 6 {
        (*md).players[i].slot_type = Gm_PKind_NA as u8;
        i += 1;
    }

    let mut player_idx: i32 = 0;
    while player_idx < (((*lvl).flags >> 5) & 7) as i32 {
        while (*lvl).player_init[player_idx as usize].is_null() {
            player_idx += 1;
        }
        gm_801BAB40(&raw mut (*md).players[player_idx as usize], (*lvl).player_init[player_idx as usize]);
        if player_idx == 0 {
            ext::gm_801B05F4((*md).players.as_mut_ptr(), (*ev).x6 as i32);
            (*ev).x7 = (*md).players[0].team;
            if (*md).players[0].ckind as i32 == ChKind_None as i32 {
                (*md).players[0].ckind = (*ev).x2;
                (*md).players[0].color = (*ev).x3;
                (*md).players[0].nametag = (*ev).nametag;
            }
            let mut c: u8 = (*md).players[0].ckind as u8;
            (*ev).x0 = c;
            (*ev).x4C[0] = c as i8;
            c = (*md).players[0].color;
            (*ev).x1 = c;
            (*ev).x50[0] = c;
            let rumble = ext::gm_RumbleEnabledForPlayer((*ev).x6 as i32, (*md).players[0].nametag as i32);
            (*md).players[0].set_rumble_enabled(rumble as u8);
        } else {
            if (*(*lvl).player_init[player_idx as usize]).team == 0 {
                (*md).players[player_idx as usize].team = (*md).players[0].team;
                (*md).players[player_idx as usize].set_xD_b1(1);
            }
            if (*(*lvl).player_init[player_idx as usize]).c_kind as i32 == ChKind_None as i32 {
                let t = (&raw mut (*ev).x8).offset(player_idx as isize - 1);
                let v = *t;
                if v == -1 {
                    let nv = gm_801BAC9C(arg0, player_idx) as u8;
                    (*md).players[player_idx as usize].ckind = nv as i8;
                    *t = (*md).players[player_idx as usize].ckind;
                } else {
                    (*md).players[player_idx as usize].ckind = v;
                }
            }
            let c_kind = (*md).players[player_idx as usize].ckind;
            if c_kind == (*md).players[0].ckind {
                let c = (*md).players[player_idx as usize].color;
                if c == (*md).players[0].color {
                    (*md).players[player_idx as usize].color = gm_GetNextColor(c);
                }
            }
            if (*md).players[0].ckind as i32 == CKind_Seak as i32 && (*md).players[player_idx as usize].ckind as i32 == CKind_Zelda as i32 {
                let c = (*md).players[player_idx as usize].color;
                if c == (*md).players[0].color {
                    (*md).players[player_idx as usize].color = gm_GetNextColor(c);
                }
            }
            if player_idx < 4 {
                (*ev).x4C[player_idx as usize] = (*md).players[player_idx as usize].ckind;
                (*ev).x50[player_idx as usize] = (*md).players[player_idx as usize].color;
            }
        }
        player_idx += 1;
    }

    if (*lvl).kind == 2 {
        if (*ev).x20 > 0 {
            (*md).players[0].stocks = (*ev).x24 as i8;
            (*md).players[0].damage = (*ev).x28 as u16;
            (*md).players[0].set_xC_b1(0);
            let c = (*ev).x38 as i8;
            if c as i32 != ChKind_None as i32 {
                (*md).players[0].ckind = c;
                (*ev).x0 = c as u8;
                (*ev).x4C[0] = c;
            }
        }
        gm_801BAB40(&raw mut (*md).players[1], (*(*lvl).evstage_table).entries[(*ev).x20 as usize]);
        if (*md).players[1].ckind == (*md).players[0].ckind {
            let c = (*md).players[1].color;
            if c == (*md).players[0].color {
                (*md).players[1].color = gm_GetNextColor(c);
            }
        }
        if (*md).players[0].ckind as i32 == CKind_Seak as i32 && (*md).players[1].ckind as i32 == CKind_Zelda as i32 {
            let c = (*md).players[1].color;
            if c == (*md).players[0].color {
                (*md).players[1].color = gm_GetNextColor(c);
            }
        }
        (*ev).x4C[1] = (*md).players[1].ckind;
        (*ev).x50[1] = (*md).players[1].color;
    }
    if level == 0x2B {
        let c2: i32 = (*ev).x4C[2] as i32;
        let c = (*ev).x50[2];
        let _ = c2;
        if (*ev).x4C[0] as i32 == c2 && (*ev).x50[0] == c {
            (*ev).x50[2] = gm_GetNextColor(c);
        }
    }
    if level == 0x31 {
        (*md).players[1].spawn_dir = 1;
        (*md).players[2].spawn_dir = -1;
        (*md).players[1].set_xD_b0(1);
        (*md).players[2].set_xD_b0(1);
        (*md).players[1].set_xD_b2(1);
        (*md).players[2].set_xD_b2(1);
    }
    if (*lvl).kind == 1 {
        (*ev).set_xB_3(1);
        let mut bonus = (*lvl).evbonus;
        let x5_flag: i32;
        if (*bonus).x5 == 1 {
            x5_flag = 1;
            if (*ev).x0 as i8 == (*bonus).c_kind {
                let c = (*bonus).color;
                if (*ev).x1 == c {
                    (*ev).x50[1] = gm_GetNextColor(c);
                }
            }
        } else {
            x5_flag = 0;
        }
        bonus = (*lvl).evbonus;
        let k: i8 = (*bonus).c_kind;
        let color2: u8 = if k == 4 { (*bonus).x17 } else { 0 };
        ext::gm_8016A22C(
            k,
            ChKind_None as i8,
            ChKind_None as i8,
            (*ev).x50[1],
            0,
            0,
            x5_flag,
            0,
            color2,
            (*ev).x0,
            (*ev).x1,
            (*bonus).x1 as i32,
            (*bonus).x2 as i32,
            (*bonus).x3 as i32,
            (*bonus).x4 as i32,
            0,
            1,
            (*bonus).x8,
            (*bonus).xC,
        );
        ext::gm_8016A414((*(*lvl).evbonus).x10);
        ext::gm_8016A21C(&raw mut (*md).rules);
        if (((*(*lvl).evbonus).flags >> 7) & 1) != 0 {
            ext::gm_8016A434();
        }
        if (*(*event_info.add(level as usize))).x4.is_some() {
            let f = (*(*event_info.add(level as usize))).x4;
            ext::gm_8016A404(core::mem::transmute(f));
        }
        if (*(*lvl).evbonus).x15 != 0 {
            ext::gm_8016A424((*(*lvl).evbonus).x15 as i8);
        }
    }
    let cache = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    match (*ev).x44 {
        0 => gm_801BA938(ev, 0, 4, 1),
        1 => gm_801BA938(ev, 0, 4, 1),
        2 => {
            (*cache).entries[0].char_id = (*ev).x4C[0] as i32;
            (*cache).entries[0].color = (*ev).x50[0];
            ext::lbDvd_80018254();
            ext::lbDvd_80018C2C(0xC7);
            gm_801BA938(ev, 1, 4, 1);
        }
        3 => {
            ext::lbDvd_80018C2C(0xC7);
            gm_801BA938(ev, 1, 4, 1);
        }
        4 => {
            ext::lbDvd_80018C6C();
            (*cache).entries[0].char_id = (*ev).x4C[0] as i32;
            (*cache).entries[0].color = (*ev).x50[0];
            ext::lbDvd_80018254();
            ext::lbDvd_80017700(4);
            gm_801BA938(ev, 1, 4, 1);
        }
        _ => {}
    }
    ext::gm_LoadRumbleEnabled(md);
    ext::gm_80168F88();
}

/// 1BB758
pub unsafe extern "C" fn onExitVs(arg0: *mut GameModeState) {
    melee_core::ct!("onExitVs");
    let ev = ev_ptr();
    let exit = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    let stage = (*ev).unk_535;
    let kind: u8;

    ext::gm_8016A164();
    if (*exit).match_end.outcome as u32 == OUTCOME_RETRY as u32 {
        let mut do_save: i32 = 0;
        if (*ev).x20 != 0 {
            do_save = 1;
        }
        if stage == 0x31 {
            do_save = 0;
        }
        (*ev).set_xB_5(0);
        (*ev).x20 = 0;
        (*ev).x24 = 0;
        (*ev).x28 = 0;
        (*ev).x2C = 0;
        (*ev).x30 = 0;
        (*ev).x34 = 0;
        (*ev).x38 = ChKind_None as u8;
        (*ev).x3C = 0;
        (*ev).x40 = 0;
        gm_801BBB64();
        if do_save != 0 {
            let gc = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
            ext::lbDvd_80018C6C();
            (*gc).entries[0].char_id = (*ev).x0 as i8 as i32;
            (*gc).entries[0].color = (*ev).x1;
            ext::lbDvd_80018254();
            ext::lbDvd_80017700(4);
        }
        ext::gm_SetNextGameModeStateId(1);
        return;
    }
    if (*exit).match_end.outcome as u32 == OUTCOME_NO_CONTEST as u32 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    (*ev).x3C += ext::gm_80168940(&raw mut vs_exit_data[0].match_end);
    (*ev).x40 += (*exit).match_end.frame_count as i32;
    if (*ev).xB_4() != 0 && (*ev).xB_2() != 0 {
        (*ev).x24 = vs_exit_data[0].match_end.player_standings[0].stocks as i32;
        (*ev).x28 = vs_exit_data[0].match_end.player_standings[0].percent as i32;
        (*ev).set_xB_2(0);
        (*ev).set_xB_5(0);
        let t = (*ev).x20;
        (*ev).x20 = t + 1;
        gm_801BBB64();
        ext::gm_SetNextGameModeStateId(1);
        return;
    }
    if (*ev).xB_1() != 0 {
        let cur: u32 = (*ev).xC as u32;
        let mut best: u32 = ext::gmMainLib_8015CF5C(stage as i32) as u32;
        let mut upd: i32 = 0;
        if (*ev).xB_6() != 0 {
            if best == 0 || cur < best {
                best = cur;
                upd = 1;
            }
        } else if cur > best {
            best = cur;
            upd = 1;
        }
        if upd != 0 {
            ext::gmMainLib_8015CF70(stage as i32, best as i32);
        }
        ext::gmMainLib_8015CEB4(stage as i32);
    }
    ext::gm_8016247C((*ev).x3C);
    ext::gm_80162968(((*ev).x40 as u32) / 60);
    if (*ev).xB_1() != 0 {
        kind = ext::gm_801732D8((*ev).unk_535);
    } else {
        kind = ChKind_Max as u8;
    }
    if (*ev).unk_535 == gm_801BEBC0(0x32) && (*exit).match_end.player_standings[0].stocks == 3 {
        ext::gmMainLib_8015CF84();
    }
    if (*ev).xB_1() != 0 {
        let sid = ext::gm_8017335C();
        if sid != 0x148 {
            ext::gm_80164504(sid);
        }
    }
    if (*ev).xB_1() != 0 {
        ext::gm_80173D3C((*ev).unk_535 as i32);
    }
    ext::gm_80173EEC();
    ext::gm_80172898(0x10);
    if kind != ChKind_Max as u8 {
        ext::gm_InitChallengerData((*ev).x0, (*ev).x1, (*ev).x6, (*ev).nametag, kind, GM_MENU as u8);
        ext::gm_ChangeGameModeAfterCurrentScene(GM_CHALLENGER_APPROACH as i32);
        return;
    }
    if ext::gm_80173754(1, (*ev).x6) == 0 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
    }
}

/// 1BB... `gm_Mode_Event_OnInit`
pub fn gm_Mode_Event_OnInit() {
    melee_core::ct!("gm_Mode_Event_OnInit");
    unsafe {
        let temp_r6 = ev_ptr();

        (*temp_r6).x0 = 0;
        (*temp_r6).x1 = 0;
        (*temp_r6).x2 = ChKind_None as i8;
        (*temp_r6).x3 = 0;
        (*temp_r6).nametag = 0x78;
        (*temp_r6).unk_535 = 0;
        (*temp_r6).x6 = 0;
        (*temp_r6).x7 = 0;
        (*temp_r6).x8 = -1;
        (*temp_r6).x9 = -1;
        (*temp_r6).xA = -1;
        (*temp_r6).set_xB_0(0);
        (*temp_r6).set_xB_1(0);
        (*temp_r6).set_xB_2(0);
        (*temp_r6).set_xB_3(0);
        (*temp_r6).set_xB_4(0);
        (*temp_r6).set_xB_5(0);
        (*temp_r6).set_xB_6(0);
        (*temp_r6).xC = 0;
        (*temp_r6).x10 = 0;
        (*temp_r6).x14 = 0;
        (*temp_r6).x18 = 0;
        (*temp_r6).x1C = 1.0;
        (*temp_r6).x20 = 0;
        (*temp_r6).x24 = 0;
        (*temp_r6).x28 = 0;
        (*temp_r6).x2C = 0;
        (*temp_r6).x30 = 0;
        (*temp_r6).x34 = 0;
        (*temp_r6).x38 = ChKind_None as u8;
        (*temp_r6).x3C = 0;
        (*temp_r6).x40 = 0;
        (*temp_r6).x44 = -1;
        (*temp_r6).x45 = 0;
        (*temp_r6).x48 = 0;

        let mut i = 0;
        while i < 4 {
            (*temp_r6).x4C[i] = ChKind_None as i8;
            (*temp_r6).x50[i] = 0;
            i += 1;
        }
    }
}

/// `gm_801BBB64_inline`
pub unsafe fn gm_801BBB64_inline(ev: *mut EventData) {
    melee_core::ct!("gm_801BBB64_inline");
    let tbl = gm_804D6900[0];
    let idx = (*ev).unk_535;
    if (*(*(*tbl.add(idx as usize))).player_init[0]).c_kind as i32 != ChKind_None as i32 {
        (*ev).x44 = 0;
    } else {
        (*ev).x44 = 1;
    }
}

/// 1BBB64
pub fn gm_801BBB64() {
    melee_core::ct!("gm_801BBB64");
    unsafe {
        let ev = ev_ptr();
        let tbl = gm_804D6900[0];
        let idx = (*ev).unk_535;

        gm_801BBB64_inline(ev);

        let pp = tbl.add(idx as usize);
        (*ev).x48 = (*(*(*pp)).evinit).stkind as StKind;

        let mut i = 0;
        while i < 4 {
            let player_init = (*(*pp)).player_init[i];
            if !player_init.is_null() {
                (*ev).x4C[i] = (*player_init).c_kind;
                (*ev).x50[i] = (*(*(*pp)).player_init[i]).color;
            } else {
                (*ev).x4C[i] = ChKind_None as i8;
                (*ev).x50[i] = 0;
            }
            i += 1;
        }

        if (*(*pp)).kind == 1 {
            (*ev).x4C[1] = (*(*(*pp)).evbonus).c_kind;
            if (*(*(*pp)).evbonus).x5 == 1 {
                (*ev).x50[1] = (*(*(*pp)).evbonus).color;
            } else {
                (*ev).x50[1] = 0xFF;
            }
            if (*(*(*pp)).evbonus).x17 as i32 == 1 {
                (*ev).x45 = 1;
            } else {
                (*ev).x45 = 0;
            }
        } else {
            (*ev).x45 = 0;
        }

        match idx {
            9 | 19 | 29 | 39 | 48 => {
                if (*ev).x20 == 0 {
                    (*ev).x44 = 3;
                } else {
                    (*ev).x44 = 4;
                }
                let mut event_entry = (*(*(*pp)).evstage_table).entries[(*ev).x20 as usize];
                (*ev).x4C[1] = (*event_entry).c_kind;
                event_entry = (*(*(*pp)).evstage_table).entries[(*ev).x20 as usize];
                (*ev).x50[1] = (*event_entry).color;
            }
            35 => {
                if (*ev).x20 == 0 {
                    (*ev).x44 = 2;
                    let mut event_entry = (*(*(*pp)).evstage_table).entries[0];
                    (*ev).x4C[1] = (*event_entry).c_kind;
                    event_entry = (*(*(*pp)).evstage_table).entries[0];
                    (*ev).x50[1] = (*event_entry).color;
                    event_entry = (*(*(*pp)).evstage_table).entries[2];
                    (*ev).x4C[2] = (*event_entry).c_kind;
                    event_entry = (*(*(*pp)).evstage_table).entries[2];
                    (*ev).x50[2] = (*event_entry).color;
                    event_entry = (*(*(*pp)).evstage_table).entries[3];
                    (*ev).x4C[3] = (*event_entry).c_kind;
                    event_entry = (*(*(*pp)).evstage_table).entries[3];
                    (*ev).x50[3] = (*event_entry).color;
                    return;
                }
                if (*ev).x20 == 1 {
                    (*ev).x44 = 4;
                    let mut event_entry = (*(*(*pp)).evstage_table).entries[1];
                    (*ev).x4C[1] = (*event_entry).c_kind;
                    event_entry = (*(*(*pp)).evstage_table).entries[1];
                    (*ev).x50[1] = (*event_entry).color;
                    event_entry = (*(*(*pp)).evstage_table).entries[4];
                    (*ev).x4C[2] = (*event_entry).c_kind;
                    event_entry = (*(*(*pp)).evstage_table).entries[4];
                    (*ev).x50[2] = (*event_entry).color;
                    return;
                }
            }
            43 => {
                let x4 = (*(*pp)).x4;
                let mut event_entry = (*x4).x4 as *mut gm_801BAB40_src;
                (*ev).x4C[2] = (*event_entry).c_kind;
                event_entry = (*x4).x4 as *mut gm_801BAB40_src;
                (*ev).x50[2] = (*event_entry).color;
            }
            _ => {}
        }
    }
}

/// `gm_Mode_Event_OnLoad`
pub fn gm_Mode_Event_OnLoad() {
    melee_core::ct!("gm_Mode_Event_OnLoad");
    unsafe {
        let temp_r30 = ev_ptr();
        gm_801BA8FC();

        let temp_r29 = gm_804D6900[0];
        let temp_r28 = (*temp_r30).unk_535;
        (*temp_r30).x0 = 0;
        (*temp_r30).x1 = 0;
        (*temp_r30).x6 = ext::gm_801677F0();
        (*temp_r30).x7 = 0;
        (*temp_r30).set_xB_0(0);
        (*temp_r30).set_xB_1(0);
        (*temp_r30).set_xB_2(0);
        (*temp_r30).set_xB_3(0);
        (*temp_r30).set_xB_4(0);
        (*temp_r30).set_xB_5(0);
        (*temp_r30).set_xB_6(0);
        (*temp_r30).xC = 0;
        (*temp_r30).x10 = 0;
        (*temp_r30).x14 = 0;
        (*temp_r30).x18 = 0;
        (*temp_r30).x1C = 1.0;
        (*temp_r30).x20 = 0;
        (*temp_r30).x24 = 0;
        (*temp_r30).x28 = 0;
        (*temp_r30).x2C = 0;
        (*temp_r30).x30 = 0;
        (*temp_r30).x34 = 0;
        (*temp_r30).x38 = ChKind_None as u8;
        (*temp_r30).x3C = 0;
        (*temp_r30).x40 = 0;
        gm_801BBB64();
        if (*(*(*temp_r29.add(temp_r28 as usize))).player_init[0]).c_kind as i32 != ChKind_None as i32 {
            ext::gm_SetGameModeStateId(1);
        }
    }
}

/// `gm_Mode_Event_OnUnload`
pub fn gm_Mode_Event_OnUnload() { melee_core::ct!("gm_Mode_Event_OnUnload");}

/// 1BBFE8
pub extern "C" fn fn_801BBFE8() {
    melee_core::ct!("fn_801BBFE8");
    ext::gm_GetCurrentGameMode();
    gm_801BC00C();
}

/// `gm_801BC00C_inline`
pub unsafe fn gm_801BC00C_inline(event_entry: *mut gm_801BAB40_src) {
    melee_core::ct!("gm_801BC00C_inline");
    let ckind: u8 = (*event_entry).c_kind as u8;
    let mut costume: u8 = (*event_entry).color;
    let ev2 = ev_ptr();

    if (*ev2).x0 as i8 == ckind as i8 && (*ev2).x1 == costume {
        if costume <= 2 {
            costume += 1;
        } else {
            costume = 0;
        }
    }
    ext::gm_8016A9E8(ckind, costume as i8);
}

/// `gm_801BC00C_GetCharacter`
pub unsafe fn gm_801BC00C_GetCharacter(event_entry: *mut gm_801BAB40_src) -> i8 {
    melee_core::ct!("gm_801BC00C_GetCharacter");
    ext::Player_800325C8((*event_entry).c_kind as i32 as u32, 0)
}

/// `gm_801BC00C_GetCharacterKind`
pub unsafe fn gm_801BC00C_GetCharacterKind(event_entry: *mut gm_801BAB40_src) -> CharacterKind {
    melee_core::ct!("gm_801BC00C_GetCharacterKind");
    (*event_entry).c_kind as i32 as CharacterKind
}

/// 1BC00C
pub fn gm_801BC00C() {
    melee_core::ct!("gm_801BC00C");
    unsafe {
        let mut event_levels: *mut *mut gm_804D6900_t = core::ptr::null_mut();
        let ev = gm_GetEventData();
        let idx = (*ev).unk_535;
        let event_info = &raw mut gm_803DF94C as *mut *mut gm_803DF94C_t;
        let mut event_entry: *mut gm_801BAB40_src;

        (*ev).set_xB_1(0);
        (*ev).x10 = 0;
        (*ev).x14 = 0;
        (*ev).x18 = 0;

        match idx {
            4 | 9 | 12 | 15 | 19 | 21 | 23 | 27 | 29 | 35 | 39 | 43 | 44 | 48 => {
                ext::lbArchive_LoadSymbols(
                    c"GmEvent.dat".as_ptr() as *const u8,
                    &raw mut gm_804D6900 as *mut c_void,
                    c"sqEventInitDataLevelTbl".as_ptr() as *const u8,
                    0,
                );
                event_levels = gm_804D6900[0];
            }
            _ => {}
        }

        match idx {
            35 => {
                if (*ev).x20 == 0 {
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[2];
                    ext::ftLib_LoadFighterCostume(
                        ext::Player_800325C8((*event_entry).c_kind as i32 as u32, 0),
                        (*event_entry).color,
                    );
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[3];
                    ext::ftLib_LoadFighterCostume(
                        ext::Player_800325C8((*event_entry).c_kind as i32 as u32, 0),
                        (*event_entry).color,
                    );
                } else {
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[4];
                    ext::ftLib_LoadFighterCostume(
                        ext::Player_800325C8(gm_801BC00C_GetCharacterKind(event_entry), 0),
                        (*event_entry).color,
                    );
                }
            }
            43 => {
                let chr = gm_801BC00C_GetCharacter((*(*(*event_levels.add(idx as usize))).x4).x4 as *mut gm_801BAB40_src);
                ext::ftLib_LoadFighterCostume(chr, (*ev).x50[2]);
                if (*ev).x0 as i8 == 4 {
                    ext::Player_80031DA8(chr as i32, (*ev).x1 as i32);
                }
            }
            _ => {}
        }

        match idx {
            9 | 19 | 29 | 35 | 39 | 48 => {
                if (*ev).x20 > 0 {
                    let mi = ext::gmVs_GetSceneState();
                    (*mi).timer_seconds = (*ev).x2C as u32;
                    (*mi).unk_2C = (*ev).x30 as u16;
                }
            }
            _ => {}
        }

        match idx {
            9 | 19 | 29 | 39 | 48 => {
                let mut i = (*ev).x20;
                while i < (*(*(*event_levels.add(idx as usize))).evstage_table).count as i32 {
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[i as usize];
                    gm_801BC00C_inline(event_entry);
                    i += 1;
                }
            }
            35 => {
                if (*ev).x20 == 0 {
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[0];
                    gm_801BC00C_inline(event_entry);
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[2];
                    gm_801BC00C_inline(event_entry);
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[3];
                    gm_801BC00C_inline(event_entry);
                }
                if (*ev).x20 <= 1 {
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[1];
                    gm_801BC00C_inline(event_entry);
                    event_entry = (*(*(*event_levels.add(idx as usize))).evstage_table).entries[4];
                    gm_801BC00C_inline(event_entry);
                }
            }
            _ => {}
        }

        if idx == 0x24 {
            ext::lbBgFlash_SetFlashScale(0.2f32);
        }
        ext::Camera_SetQuakeScale((*ev).x1C);
        ext::HSD_GObj_SetupProc(ext::GObj_Create(0xF, 0x11, 0), (*(*event_info.add(idx as usize))).x0, 0x15);
    }
}

/// 1BC488
pub fn gm_801BC488() -> i32 {
    melee_core::ct!("gm_801BC488");
    unsafe {
        let tmp = ext::gm_8016B774();
        let mut sp_c: MatchEnd = *tmp;

        ext::gm_80166378(&mut sp_c);
        if sp_c.n_team_winners == 1 {
            return sp_c.team_winners[0] as i32;
        }
        4
    }
}

/// 1BC4F4
pub unsafe fn gm_801BC4F4(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BC4F4");
    let temp_r30: u32 = (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535 as u32;
    let temp_r28 = ev_ptr();
    (*temp_r28).set_xB_1(1);
    let temp_r29: u32 = ext::gmMainLib_8015CF5C(temp_r30 as i32) as u32;
    let mut var_r25 = false;
    let mut var_r4: u32;
    let mut var_r27: u32;
    if (*temp_r28).xB_6() != 0 {
        var_r4 = ext::gm_GetFrameCount();
        var_r4 = var_r4.wrapping_add((*temp_r28).x34 as u32);
        if temp_r30 == 0x31 {
            var_r4 = (*temp_r28).x34 as u32;
        }
        if var_r4 > 0x34BBF {
            var_r4 = 0x34BBF;
        }
        (*temp_r28).xC = var_r4 as i32;
        if temp_r29 == 0 || var_r4 < temp_r29 {
            var_r25 = true;
        }
    } else {
        var_r27 = 0;
        let mut i = 1;
        while i < 6 {
            if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA {
                var_r27 = var_r27.wrapping_add(ext::Player_GetKOsByPlayerIndex(0, i) as u32);
            }
            i += 1;
        }
        if temp_r30 == 0x1F {
            var_r27 = (ext::Player_GetKOsByPlayerIndex(0, 1) - ext::pl_8003FBFC(0)) as u32;
        }
        // `var_r27 > -1` compares the u32 with 0xFFFFFFFF: never true, so no clamp
        if var_r27 > (-1i32) as u32 {
            var_r27 = (-1i32) as u32;
        }
        (*temp_r28).xC = var_r27 as i32;
        if var_r27 > temp_r29 {
            var_r25 = true;
        }
    }
    ext::Player_80036844(0, 1);
    ext::lbAudioAx_80028B90();
    ext::gm_SetGameSpeed(1.0f32);
    if var_r25 {
        ext::gm_8016B33C(2);
        ext::gm_8016B350(0x9C40);
        ext::gm_8016B364(0x144);
    } else {
        ext::gm_8016B33C(2);
        ext::gm_8016B364(0x145);
    }
    ext::gm_8016B328();
    ext::HSD_GObjFree(gobj);
}

/// 1BC670
pub unsafe fn gm_801BC670(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BC670");
    let temp_r31 = ev_ptr();
    let temp_r30 = (*(*gm_804D6900[0]).add(0)).x4;
    let _ = arg0;

    (*temp_r31).set_xB_2(1);
    (*temp_r31).x10 = 0x78 - (*temp_r30).x4 as i32;
    (*temp_r31).x2C = ext::gm_8016AEEC() as i32;
    (*temp_r31).x30 = ext::gm_8016AEFC() as i32;
    if (*temp_r31).x2C < (*temp_r30).x0 {
        (*temp_r31).x2C = (*temp_r30).x0;
        (*temp_r31).x30 = 0;
    }
    if ext::Player_80036394(0) == Ft_Kind_Seak {
        (*temp_r31).x38 = CKind_Seak as u8;
    } else {
        (*temp_r31).x38 = ChKind_None as u8;
    }
    (*temp_r31).x34 += ext::gm_GetFrameCount() as i32;
    ext::Player_80036844(0, 1);
    ext::lbAudioAx_80028B90();
    ext::gm_SetGameSpeed(1.0f32);
    ext::gm_8016B33C(8);
    ext::gm_8016B328();
}

/// `failEvent` (static inline)
pub unsafe fn failEvent(gobj: *mut HSD_GObj) {
    melee_core::ct!("failEvent");
    (*ev_ptr()).set_xB_1(0);
    ext::lbAudioAx_80028B90();
    ext::gm_SetGameSpeed(1.0f32);
    ext::gm_8016B33C(6);
    ext::gm_8016B364(0x148);
    ext::gm_8016B378(0x28);
    ext::gm_8016B328();
    ext::HSD_GObjFree(gobj);
}

/// The "time ran out" test the procs share: no timer-up flag, timer on, 0:00 and 59 frames.
#[inline(always)]
unsafe fn timer_expired(ev: *mut EventData, ctl: *mut VsSceneController) -> bool {
    if (*ev).xB_0() != 0 {
        false
    } else {
        (*ctl).start.timer_enabled() != 0 && ext::gm_8016AEEC() == 0 && ext::gm_8016AEFC() == 0x3B
    }
}

/// 1BC754
pub unsafe extern "C" fn gm_801BC754(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BC754");
    let temp_r29 = ev_ptr();
    match (*ext::gmVs_GetSceneController()).start.match_kind() {
        1 => {
            let mut count = 0;
            let temp_r28 = ev_ptr();
            let mut i = 1;
            while i < 6 {
                if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA && ext::Player_GetStocks(i) != 0 {
                    count += 1;
                }
                i += 1;
            }
            if (*temp_r28).xB_3() != 0 {
                count += ext::gm_80169384();
            }
            if count == 0 {
                gm_801BC4F4(gobj);
                return;
            }
            if ext::Player_GetP1Stock() <= 0 {
                failEvent(gobj);
                return;
            }
            let temp_r28_2 = ev_ptr();
            let temp_r3 = ext::gmVs_GetSceneController();
            let var_r0 = timer_expired(temp_r28_2, temp_r3);
            if var_r0 {
                failEvent(gobj);
            }
        }
        0 | 2 => {
            let temp_r28_3 = ev_ptr();
            let temp_r3_2 = ext::gmVs_GetSceneController();
            let var_r0_2 = timer_expired(temp_r28_3, temp_r3_2);
            if var_r0_2 {
                if (*temp_r29).x7 as i32 == gm_801BC488() {
                    gm_801BC4F4(gobj);
                    return;
                }
                failEvent(gobj);
            }
        }
        _ => {}
    }
}

/// 1BC9E8
pub unsafe extern "C" fn gm_801BC9E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BC9E8");
    let temp_r30 = (*(*gm_804D6900[0].add((*ev_ptr()).unk_535 as usize))).x4;
    let coins = ext::Player_GetCoins(0) as u32;

    if coins >= (*temp_r30).x0 as u32 {
        gm_801BC4F4(gobj);
        return;
    }
    let temp_r30_2 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r30_2, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BCAF0
pub unsafe extern "C" fn gm_801BCAF0(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BCAF0");
    let mut count = 0;
    let mut i = 1;
    while i < 3 {
        let var_r0: i32;
        let mut go: *mut HSD_GObj = core::ptr::null_mut();
        let cond = ext::Player_GetStocks(i) <= 0 && {
            go = ext::Player_GetEntity(i);
            !go.is_null()
        };
        if cond {
            if ext::ftLib_GetKind(go) == Ft_Kind_Seak {
                var_r0 = 1;
            } else {
                ext::Player_SetStocks(i, 1);
                ext::gm_8016F00C(i);
                var_r0 = 0;
            }
        } else {
            var_r0 = 0;
        }
        if var_r0 != 0 {
            count += 1;
        }
        i += 1;
    }
    if count == 2 {
        gm_801BC4F4(gobj);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r30 = ev_ptr();
    let temp_r3_2 = ext::gmVs_GetSceneController();
    let var_r0_2 = timer_expired(temp_r30, temp_r3_2);
    if var_r0_2 {
        failEvent(gobj);
    }
}

/// 1BCC9C
pub unsafe extern "C" fn gm_801BCC9C(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BCC9C");
    let temp_r29 = gm_804D6900[0];
    let ev = ev_ptr();
    let idx = (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535;
    let x4 = (*(*temp_r29)).x4;

    if (*ext::gmMainLib_804D3EE0).vs.unk_530.xB_2() != 0 {
        (*ev).x10 -= 1;
        if (*ev).x10 < 0 {
            ext::lbBgFlash_8002063C((*x4).x4 as i32);
            ext::HSD_GObjFree(arg0);
        }
        return;
    }
    if ext::Player_GetStocks(1) <= 0 {
        let entry = temp_r29.add(idx as usize);
        let inner = (*(*entry)).evstage_table;
        let cd = (*inner).entries[(*ev).x20 as usize];
        let ev2 = ev_ptr();
        let mut costume = (*cd).color;
        if (*ev2).x0 as i8 == (*cd).c_kind && (*ev2).x1 == costume {
            if costume <= 2 {
                costume += 1;
            } else {
                costume = 0;
            }
        }
        ext::gm_8016AC44((*cd).c_kind, costume as i8);
        if (*ev).x20 >= ((*(*(*entry)).evstage_table).count as i32 - 1) {
            gm_801BC4F4(arg0);
            return;
        }
        if ext::Player_GetP1Stock() <= 0 {
            failEvent(arg0);
            return;
        }
        gm_801BC670(arg0);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(arg0);
        return;
    }
    let var_r0;
    {
        let ev3 = gm_GetEventData();
        let mi = ext::gmVs_GetSceneController();
        var_r0 = timer_expired(ev3, mi);
    }
    if var_r0 {
        failEvent(arg0);
    }
}

/// 1BCF20
pub unsafe extern "C" fn gm_801BCF20(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BCF20");
    gm_801BCC9C(gobj);
}

/// 1BCF40
pub unsafe extern "C" fn gm_801BCF40(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BCF40");
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        gm_801BC4F4(gobj);
    }
}

/// 1BD028
pub unsafe extern "C" fn gm_801BD028(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD028");
    let mut pos: Vec3 = core::mem::zeroed();
    let mut ev = ev_ptr();
    let levels = gm_804D6900[0];
    let level = (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535;

    if (*ext::gmMainLib_804D3EE0).vs.unk_530.x10 == 0 {
        let src = (*(*levels.add(level as usize))).x4 as *mut Vec3;
        pos.x = (*src).x;
        pos.y = (*src).y;
        pos.z = (*src).z;
        ext::itEvYoshiEgg_Spawn(&mut pos);
        (*ev).x10 += 1;
    }
    if (*ev).x18 != 0 {
        failEvent(arg0);
        return;
    }
    ev = ev_ptr();
    let rules = ext::gmVs_GetSceneController();
    let cond = timer_expired(ev, rules);
    if cond {
        gm_801BC4F4(arg0);
    }
}

/// 1BD164
pub unsafe extern "C" fn gm_801BD164(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD164");
    if ext::Player_GetStocks(2) <= 0 {
        let temp_r30 = ext::Player_GetKOsByPlayerIndex(1, 2);
        if ext::Player_GetKOsByPlayerIndex(0, 2) > temp_r30 {
            gm_801BC4F4(gobj);
        } else {
            failEvent(gobj);
        }
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r30_2 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r30_2, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BD30C
pub unsafe extern "C" fn gm_801BD30C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD30C");
    if ext::Player_GetStocks(1) <= 0 {
        failEvent(gobj);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        gm_801BC4F4(gobj);
    }
}

/// 1BD44C
pub unsafe extern "C" fn gm_801BD44C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD44C");
    gm_801BCC9C(gobj);
}

/// 1BD46C
pub unsafe extern "C" fn gm_801BD46C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD46C");
    let mut count = 0;
    let mut var_r30: i32 = 0;
    let mut i = 1;
    while i < 3 {
        let p = ext::Player_GetEntityAtIndex(i, 1);
        if !p.is_null() {
            var_r30 = ext::ftLib_IsSleeping_8008731C(p);
        }
        if var_r30 != 0 {
            count += 1;
        }
        i += 1;
    }
    if count == 2 {
        gm_801BC4F4(gobj);
        return;
    }
    i = 1;
    while i < 3 {
        if ext::Player_GetStocks(i) <= 0 {
            failEvent(gobj);
            return;
        }
        i += 1;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BD658
pub unsafe extern "C" fn gm_801BD658(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD658");
    let mut var_r30: i32 = 0;
    let temp_r28 = ev_ptr();
    let mut i = 1;
    while i < 6 {
        if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA && ext::Player_GetStocks(i) != 0 {
            var_r30 += 1;
        }
        i += 1;
    }
    if (*temp_r28).xB_3() != 0 {
        var_r30 += ext::gm_80169384();
    }
    if var_r30 == 0 {
        gm_801BC4F4(gobj);
        ext::ifStock_802FA2D0(0);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r28_2 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r28_2, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BD7FC
pub unsafe extern "C" fn gm_801BD7FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD7FC");
    if ext::Player_GetStocks(1) <= 0 {
        gm_801BC4F4(gobj);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BD93C
pub unsafe extern "C" fn gm_801BD93C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BD93C");
    if ext::Player_GetStocks(2) <= 0 {
        gm_801BC4F4(gobj);
        return;
    }
    if ext::Player_GetStocks(1) <= 0 {
        failEvent(gobj);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BDAD4
pub unsafe extern "C" fn gm_801BDAD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BDAD4");
    gm_801BCC9C(gobj);
}

/// 1BDAF4
pub unsafe extern "C" fn gm_801BDAF4(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BDAF4");
    let temp_r30 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r30, temp_r3);
    if var_r0 {
        let temp_r30_2 = ext::pl_8003FC20(1);
        let temp_r31 = ext::Player_GetKOsByPlayerIndex(0, 1);
        if temp_r31 - ext::pl_8003FBFC(0) > temp_r30_2 {
            gm_801BC4F4(arg0);
            return;
        }
        failEvent(arg0);
    }
}

/// 1BDC08
pub unsafe extern "C" fn gm_801BDC08(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BDC08");
    if ext::Ground_801C1D84() != 0 {
        gm_801BC4F4(arg0);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(arg0);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        failEvent(arg0);
    }
}

/// 1BDD44
pub unsafe extern "C" fn gm_801BDD44(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BDD44");
    if ext::Player_GetStocks(2) <= 0 && ext::Player_GetStocks(3) <= 0 {
        gm_801BC4F4(arg0);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(arg0);
        return;
    }
    let temp_r31 = ev_ptr();
    let temp_r3 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r31, temp_r3);
    if var_r0 {
        failEvent(arg0);
    }
}

/// 1BDE94
pub unsafe extern "C" fn gm_801BDE94(arg0: *mut HSD_GObj) {
    melee_core::ct!("gm_801BDE94");
    let mut sp50: PlayerInitData = core::mem::zeroed();
    let tbl = gm_804D6900[0];
    let ev = gm_GetEventData();
    let level = (*ev).unk_535;
    let mut mask: u64;
    let x4 = (*(*tbl)).x4;
    let lvl = *tbl.add(level as usize);

    if (*ev).xB_5() == 0 {
        (*ev).set_xB_5(1);
        if (*ev).x20 == 0 {
            mask = ext::lbAudioAx_80026E84((*(*lvl).player_init[0]).c_kind as u8 as i8 as i32 as u32);
            mask |= ext::lbAudioAx_80026E84((*(*(*lvl).evstage_table).entries[0]).c_kind as i32 as u32);
            mask |= ext::lbAudioAx_80026E84((*(*(*lvl).evstage_table).entries[2]).c_kind as i32 as u32);
            mask |= ext::lbAudioAx_80026E84((*(*(*lvl).evstage_table).entries[3]).c_kind as i32 as u32);
        } else {
            mask = ext::lbAudioAx_80026E84((*(*lvl).player_init[0]).c_kind as u8 as i8 as i32 as u32);
            mask |= ext::lbAudioAx_80026E84((*(*(*lvl).evstage_table).entries[1]).c_kind as i32 as u32);
            mask |= ext::lbAudioAx_80026E84((*(*(*lvl).evstage_table).entries[4]).c_kind as i32 as u32);
        }
        ext::lbAudioAx_80026F2C(0x14);
        ext::lbAudioAx_8002702C(4, mask);
        ext::lbAudioAx_80027168();
        ext::lbAudioAx_80027648();
    }

    if (*ev).xB_2() != 0 {
        (*ev).x10 -= 1;
        if (*ev).x10 < 0 {
            ext::lbBgFlash_8002063C((*x4).x4 as i32);
            ext::HSD_GObjFree(arg0);
        }
        return;
    } else if (*ev).x20 == 0 {
        match (*ev).x18 {
            0 => {
                if ext::Player_GetStocks(1) <= 0 {
                    (*ev).x18 = 1;
                    gm_801BAB40(&mut sp50, (*(*lvl).evstage_table).entries[2]);
                    ext::gm_8016EDDC(2, &mut sp50);
                }
            }
            1 => {
                if ext::Player_GetStocks(2) <= 0 {
                    (*ev).x18 = 2;
                    gm_801BAB40(&mut sp50, (*(*lvl).evstage_table).entries[3]);
                    ext::gm_8016EDDC(3, &mut sp50);
                }
            }
            2 => {
                if ext::Player_GetStocks(3) <= 0 {
                    let sp = (*(*lvl).evstage_table).entries[3];
                    let ev2 = ev_ptr();
                    let mut color = (*sp).color;
                    if (*ev2).x0 as i8 == (*sp).c_kind && (*ev2).x1 == color {
                        if color <= 2 {
                            color += 1;
                        } else {
                            color = 0;
                        }
                    }
                    ext::gm_8016AC44((*sp).c_kind, color as i8);
                    if ext::Player_GetP1Stock() <= 0 {
                        failEvent(arg0);
                        return;
                    }
                    gm_801BC670(arg0);
                    return;
                }
            }
            _ => {}
        }
    } else {
        match (*ev).x18 {
            0 => {
                if ext::Player_GetStocks(1) <= 0 {
                    (*ev).x18 = 1;
                    gm_801BAB40(&mut sp50, (*(*lvl).evstage_table).entries[4]);
                    ext::gm_8016EDDC(2, &mut sp50);
                }
            }
            1 => {
                if ext::Player_GetStocks(2) <= 0 {
                    let sp = (*(*lvl).evstage_table).entries[4];
                    let ev2 = ev_ptr();
                    let mut color = (*sp).color;
                    if (*ev2).x0 as i8 == (*sp).c_kind && (*ev2).x1 == color {
                        if color <= 2 {
                            color += 1;
                        } else {
                            color = 0;
                        }
                    }
                    ext::gm_8016AC44((*sp).c_kind, color as i8);
                    gm_801BC4F4(arg0);
                    return;
                }
            }
            _ => {}
        }
    }

    if ext::Player_GetP1Stock() <= 0 {
        failEvent(arg0);
        return;
    }
    {
        let ev2 = gm_GetEventData();
        let info = ext::gmVs_GetSceneController();
        let do_end = timer_expired(ev2, info);
        if do_end {
            failEvent(arg0);
        }
    }
}

/// 1BE37C
pub unsafe extern "C" fn gm_801BE37C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BE37C");
    gm_801BCC9C(gobj);
}

/// 1BE39C
pub unsafe extern "C" fn gm_801BE39C(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BE39C");
    let mut sp40: PlayerInitData = core::mem::zeroed();
    let temp_r28 = gm_804D6900[0];
    let temp_r0 = (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535;
    let temp_r31 = ev_ptr();
    let temp_r30 = (*(*temp_r28.add(temp_r0 as usize))).x4;

    if (*temp_r31).xB_5() == 0 {
        (*temp_r31).set_xB_5(1);
        let mut temp_ret = ext::lbAudioAx_80026E84(ext::Player_GetPlayerCharacter(0));
        temp_ret |= ext::lbAudioAx_80026E84((*(*(*temp_r28.add(temp_r0 as usize))).player_init[1]).c_kind as i32 as u32);
        temp_ret |= ext::lbAudioAx_80026E84((*((*temp_r30).x4 as *mut gm_801BAB40_src)).c_kind as i32 as u32);
        ext::lbAudioAx_80026F2C(0x14);
        ext::lbAudioAx_8002702C(4, temp_ret);
        ext::lbAudioAx_80027168();
        ext::lbAudioAx_80027648();
    }
    if (*temp_r31).x10 != 0 && ext::Player_GetStocks(2) <= 0 {
        gm_801BC4F4(gobj);
        return;
    }
    if ext::Player_GetStocks(1) <= 0 {
        failEvent(gobj);
        return;
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    if (*temp_r31).x10 == 0 && ext::gm_8016AEEC() >= (*temp_r30).x0 as u32 {
        (*temp_r31).x10 = 1;
        gm_801BAB40(&mut sp40, (*temp_r30).x4 as *mut gm_801BAB40_src);
        sp40.color = (*temp_r31).x50[2];
        ext::gm_8016EDDC(2, &mut sp40);
    }
    let temp_r27_5 = ev_ptr();
    let temp_r3_2 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r27_5, temp_r3_2);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BE618
pub unsafe extern "C" fn gm_801BE618(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BE618");
    gm_801BCC9C(gobj);
}

/// 1BE638
pub unsafe extern "C" fn gm_801BE638(gobj: *mut HSD_GObj) {
    melee_core::ct!("gm_801BE638");
    let temp_r30 = ev_ptr();
    let temp_r31 = ext::ftBossLib_8015C74C();
    let temp_r28 = ext::ftBossLib_8015C7EC();
    let temp_r29 = ext::ftBossLib_8015C92C();
    match (*temp_r30).x20 {
        0 => {
            if ext::Player_GetRemainingHP(1) <= 0 || ext::Player_GetRemainingHP(2) <= 0 {
                (*temp_r30).x20 = 1;
                (*temp_r30).x10 = 0;
            }
        }
        1 => {
            if ext::Player_GetRemainingHP(1) <= 0 && ext::Player_GetRemainingHP(2) <= 0 {
                (*temp_r30).x20 = 2;
                (*temp_r30).x10 = 0;
            }
        }
        2 => {}
        _ => {}
    }
    if ext::Player_GetRemainingHP(1) <= 0 && ext::Player_GetRemainingHP(2) <= 0 {
        let temp_r28_2 = temp_r31 + temp_r29;
        if (*temp_r30).x10 == 0 {
            let tmp = ext::gmVs_GetSceneController();
            (*tmp).state.hud_enabled = 0;
            if (*temp_r30).x18 == 1 {
                (*temp_r30).x18 = 2;
            } else {
                (*temp_r30).x18 = 1;
            }
            ext::ftBossLib_8015CB9C((*temp_r30).x18);
            ext::gm_SetDbPauseFlag(6);
            ext::Player_80031790(0);
            ext::Player_80036844(0, 1);
            (*temp_r30).x34 = ext::gm_GetFrameCount() as i32;
            let temp_r3 = ext::gmVs_GetSceneController();
            (*temp_r3).start.set_timer_enabled(0);
        }
        if (*temp_r30).x10 == temp_r31 {
            ext::lbAudioAx_8002438C(0x4E200);
            ext::ftBossLib_8015CC14();
            ext::gm_ClearDbPauseFlag(6);
            ext::gm_SetGameSpeed(0.5f32);
            ext::lbBgFlash_80020688(temp_r29);
        }
        let temp_r0_2 = (*temp_r30).x10;
        if temp_r0_2 >= temp_r28_2 {
            gm_801BC4F4(gobj);
            return;
        }
        if temp_r0_2 >= temp_r31 {
            ext::lbAudioAx_80028B6C();
        }
        if ((*temp_r30).x10 % 30) == 0 {
            ext::gm_80167858((*temp_r30).x6 as i32, ext::Player_GetNametagSlotID(0) as i32, 2, 0x1E);
            ext::Camera_RequestQuake(QuakeKind_Medium, core::ptr::null_mut());
        }
        let temp_r3_2 = (*temp_r30).x10;
        if temp_r3_2 <= temp_r28_2 {
            (*temp_r30).x10 = temp_r3_2 + 1;
        }
    } else if ext::Player_GetRemainingHP(1) <= 0 || ext::Player_GetRemainingHP(2) <= 0 {
        if ext::Player_GetRemainingHP(1) <= 0 {
            (*temp_r30).x18 = 1;
        } else {
            (*temp_r30).x18 = 2;
        }
        let temp_r28_3 = temp_r31 + temp_r28;
        if (*temp_r30).x10 == 0 {
            ext::ftBossLib_8015CB9C((*temp_r30).x18);
            ext::gm_SetDbPauseFlag(6);
        }
        if (*temp_r30).x10 == temp_r31 {
            ext::lbAudioAx_8002438C(0x4E200);
            ext::ftBossLib_8015CC14();
            ext::gm_ClearDbPauseFlag(6);
            ext::gm_SetGameSpeed(0.5f32);
        }
        if (*temp_r30).x10 == temp_r28_3 {
            ext::gm_SetGameSpeed(1.0f32);
        }
        let temp_r3_3 = (*temp_r30).x10;
        if temp_r3_3 <= temp_r28_3 {
            (*temp_r30).x10 = temp_r3_3 + 1;
        }
    }
    if ext::Player_GetP1Stock() <= 0 {
        failEvent(gobj);
        return;
    }
    let temp_r28_4 = ev_ptr();
    let temp_r3_4 = ext::gmVs_GetSceneController();
    let var_r0 = timer_expired(temp_r28_4, temp_r3_4);
    if var_r0 {
        failEvent(gobj);
    }
}

/// 1BEA10: the event's player-CPU type
pub unsafe extern "C" fn gm_801BEA10(arg0: i32) {
    melee_core::ct!("gm_801BEA10");
    let lvl = *gm_804D6900[0].add((*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535 as usize);
    ext::Player_SetPlayerAndEntityCpuType(arg0, (*(*lvl).evbonus).x16 as i32);
}

/// 1BEA4C
pub unsafe extern "C" fn gm_801BEA4C(arg0: i32) {
    melee_core::ct!("gm_801BEA4C");
    gm_801BEA10(arg0);
}

/// 1BEA88
pub unsafe extern "C" fn gm_801BEA88(arg0: i32, arg1: i32) {
    melee_core::ct!("gm_801BEA88");
    gm_801BEA10(arg0);
    if arg1 == 0 {
        ext::Player_SetModelScale(arg0, 2.0f32);
    }
}

/// 1BEAF0
pub unsafe extern "C" fn gm_801BEAF0(arg0: i32) {
    melee_core::ct!("gm_801BEAF0");
    gm_801BEA10(arg0);
}

/// 1BEB2C
pub unsafe extern "C" fn gm_801BEB2C(arg0: i32) {
    melee_core::ct!("gm_801BEB2C");
    gm_801BEA10(arg0);
}

/// 1BEB68
pub fn gm_801BEB68(arg0: i32) {
    melee_core::ct!("gm_801BEB68");
    unsafe { (*ext::gmMainLib_804D3EE0).vs.unk_530.x18 = arg0 }
}

/// 1BEB74
pub fn gm_801BEB74(arg0: u8) {
    melee_core::ct!("gm_801BEB74");
    unsafe { (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535 = arg0 }
}

/// 1BEB80
pub fn gm_801BEB80() -> u8 {
    melee_core::ct!("gm_801BEB80");
    unsafe { (*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535 }
}

/// 1BEB8C
pub fn gm_801BEB8C(arg0: u8) -> bool {
    melee_core::ct!("gm_801BEB8C");
    unsafe { (*(*(*gm_804D6900[0].add(arg0 as usize))).evinit).x1_0() != 0 }
}

/// 1BEBA8
pub fn gm_801BEBA8(arg0: u8) -> u8 {
    melee_core::ct!("gm_801BEBA8");
    gm_803DF918[arg0 as usize]
}

/// 1BEBC0
pub fn gm_801BEBC0(arg0: u8) -> u8 {
    melee_core::ct!("gm_801BEBC0");
    let tmp = &gm_803DF918;
    let mut i: u8 = 0;
    while i < 0x33 {
        if arg0 == tmp[i as usize] {
            break;
        }
        i += 1;
    }
    i
}

/// 1BEBF8
pub fn gm_801BEBF8(arg0: u8) -> u8 {
    melee_core::ct!("gm_801BEBF8");
    unsafe {
        let table = &gm_803DF918;
        let mut i: u8 = 0;
        let array = gm_804D6900[0];

        while i < 0x33 {
            if arg0 == table[i as usize] {
                break;
            }
            i += 1;
        }

        let entry = *array.add(i as usize);
        if entry.is_null() {
            return ChKind_None as u8;
        }

        (*(*entry).player_init[0]).c_kind as u8
    }
}

/// 1BEC54
pub fn gm_801BEC54() -> *mut c_void {
    melee_core::ct!("gm_801BEC54");
    unsafe {
        let temp_r3 = *gm_804D6900[0].add((*ext::gmMainLib_804D3EE0).vs.unk_530.unk_535 as usize);
        if temp_r3.is_null() {
            return core::ptr::null_mut();
        }
        (*temp_r3).x4 as *mut c_void
    }
}

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gmevent.c `gm_GetNextColor_fp`.
pub fn gm_GetNextColor_fp(color: u8) -> u8 {
    melee_core::ct!("gm_GetNextColor_fp");
    if color <= 2 {
        color + 1
    } else {
        0
    }
}
