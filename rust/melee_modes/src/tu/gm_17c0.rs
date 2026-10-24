//! Native port of src/melee/gm/gm_17C0.c: the 1P-mode (Classic / Adventure / All-Star) controller. The Master
//! Hand / Crazy Hand boss-fight state machine (`fn_8017C1A4`), the stage / opponent setup of a 1P match
//! (`gm_8017CE34`), the post-match bookkeeping (`gm_8017D7AC`, `gm_8017CA38`, `gm_8017CBAC`), the
//! per-mode 1P data accessors and the unlock-progress table. Verified against the C by
//! tests/tu_gm_17c0.rs (state oracle).
//!
//! Notes on the port:
//! * This is the `TARGET_PC` build of the C: the m-ex external-id constants, the 1P-script hooks
//!   (`Script_OnePStage` ... `Script_BossHookEnabled`) and the boss hook (`boss_hook`, `BossHook_*`) are in.
//! * `OSReport` is variadic in C; the stub takes the format string only, so the formatted values of those
//!   log lines are not passed on (the log text itself is the same).
//! * `fn_8017C71C` / `fn_8017C7A0` / `fn_8017C7EC` / `fn_8017C1A4` are `extern "C"`: their addresses are stored
//!   in the rules (`on_match_start`, `on_match_end`) and in a GObj proc.
//! * `fn_8017C1A4` has a function-local `static int drive_tick` in the C; it is `DRIVE_TICK` here.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::ext_hook::memzero;
use crate::ext_fns;
use core::ffi::c_void;
use melee_types::*;

mod data;
pub use data::lbl_803B7C08;

/// `lbl_804706D8_t` (GC size 4): one unlock-progress slot.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_804706D8_t {
    pub x0: i16,
    pub x2: u8,
    pub x3: u8,
}

/// `struct lbl_804706C0_t` (GC size 0x18): the 1P boss-fight controller state.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct lbl_804706C0_t {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    /// HP (stamina)
    pub xC: i32,
    pub x10: i32,
    pub x14: i32,
}

/// The anonymous `boss_hook` struct (TARGET_PC): every decision the simulation reads.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct boss_hook_t {
    pub sent_mask: i32,
    pub pending: i32,
    pub held: i32,
    pub restored: i32,
    pub limit: i32,
    pub elapsed: i32,
    pub sleep_wait: i32,
    /// hands already driven into the death by BossHook_DriveDeaths (once only)
    pub driven_mask: i32,
}

/// `RegClearCharEntry` (GC size 0xC).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RegClearCharEntry {
    pub x0: u8,
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
    pub x4: f32,
    pub x8: f32,
}

pub static mut lbl_804706C0: lbl_804706C0_t = lbl_804706C0_t { x0: 0, x4: 0, x8: 0, xC: 0, x10: 0, x14: 0 };
pub static mut boss_hook: boss_hook_t =
    boss_hook_t { sent_mask: 0, pending: 0, held: 0, restored: 0, limit: 0, elapsed: 0, sleep_wait: 0, driven_mask: 0 };
pub static mut lbl_804706D8: [lbl_804706D8_t; 12] = [lbl_804706D8_t { x0: 0, x2: 0, x3: 0 }; 12];
pub static mut gmClassic_80472AF8: StartMeleeData = unsafe { core::mem::zeroed() };
pub static mut gmClassic_8047086C: MatchExitInfo = unsafe { core::mem::zeroed() };
pub static mut gmClassic_80470850: DebugGameOverData = unsafe { core::mem::zeroed() };
pub static mut gmClassic_80470708: CSSData = unsafe { core::mem::zeroed() };

/// `static int drive_tick` of `fn_8017C1A4`.
static mut DRIVE_TICK: i32 = 0;

/// Callees (and data) in TUs that are not ported yet.
pub mod ext {
    use super::*;
    pub static mut DbLevel: DbLKind = 0;
    /// gmregcommon.c: the table of internal character ids the 1P opponents are picked from (24 ids, ends with
    /// `ChKind_None`).
    pub static mut lbl_803D79F0: [u8; 24] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 20, 21, 22, 24, 25, 33,
    ];
    /// gm_18A1.c
    pub static mut gm_80473A18: UnkAllstarData = unsafe { core::mem::zeroed() };
    ext_fns! {
        fn Camera_RequestQuake(a0: CmQuakeKind, a1: *mut Vec);
        fn Camera_SetQuakeScale(a0: f32) = crate::tu::adapt::Camera_SetQuakeScale__cmc;
        fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
        fn Ground_801C1DAC() -> u32 = crate::tu::adapt::Ground_801C1DAC__5d253e;
        fn Ground_801C5A60() = crate::tu::adapt::Ground_801C5A60__d6325d;
        fn Ground_801C5AD0(a0: i32) -> u32 = crate::tu::adapt::Ground_801C5AD0__e73f00;
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_Randi(a0: i32) -> i32;
        fn OSReport(a0: *mut i8);
        fn Player_80031790(a0: i32) = crate::tu::adapt::Player_80031790__11821c;
        fn Player_80031848(a0: i32) = crate::tu::adapt::Player_80031848__c31dd9;
        fn Player_80036844(a0: i32, a1: i32) = crate::tu::adapt::Player_80036844__2d424f;
        fn Player_GetAttackRatio(a0: i32) -> f32 = crate::tu::adapt::Player_GetAttackRatio__61b7a9;
        fn Player_GetCpuLevel(a0: i32) -> i32 = crate::tu::adapt::Player_GetCpuLevel__8735b7;
        fn Player_GetCpuType(a0: i32) -> i32 = crate::tu::adapt::Player_GetCpuType__ec7fec;
        fn Player_GetEntity(a0: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntity__090538;
        fn Player_GetNametagSlotID(a0: i32) -> u8 = crate::tu::adapt::Player_GetNametagSlotID__655434;
        fn Player_GetPlayerId(a0: i32) -> i32 = crate::tu::adapt::Player_GetPlayerId__aaadbd;
        fn Player_GetRemainingHP(a0: i32) -> i32 = crate::tu::adapt::Player_GetRemainingHP__d5e70f;
        fn Player_GetUnk50(a0: i32) -> f32 = crate::tu::adapt::Player_GetUnk50__869319;
        fn Player_SetMoreFlagsBit3(a0: i32, a1: u8) = crate::tu::adapt::Player_SetMoreFlagsBit3__e7d064;
        fn Player_SetMoreFlagsBit4(a0: i32, a1: u8) = crate::tu::adapt::Player_SetMoreFlagsBit4__b3407d;
        fn Player_SetOtherStamina(a0: i32, a1: i32) = crate::tu::adapt::Player_SetOtherStamina__ccc0cd;
        fn Script_BossHookEnabled() -> i32;
        fn Script_GameEvent(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32);
        fn Script_OnePClear();
        fn Script_OnePComplete() -> i32;
        fn Script_OnePGameOver();
        fn Script_OnePScripted() -> i32;
        fn Script_OnePStage(a0: i32, a1: i32, a2: i32, a3: i32);
        fn Stage_80225074(a0: i32) -> i32 = crate::tu::adapt::Stage_80225074__bd6d40;
        fn Stage_8022519C(a0: StKind) -> GrKind = crate::tu::adapt::Stage_8022519C__9b8f69;
        fn Toy_SetUnlockState(a0: enum_t, a1: i32) = crate::tu::adapt::Toy_SetUnlockState__c2d7ba;
        fn fn_80162BFC(a0: i8, a1: i32) -> i32 = crate::tu::adapt::fn_80162BFC__e70f6e;
        fn fn_80162DF8(a0: u8, a1: u32) -> i32 = crate::tu::adapt::fn_80162DF8__3b29ac;
        fn fn_80162FF4(a0: u8, a1: u32) -> i32 = crate::tu::adapt::fn_80162FF4__5bf35f;
        fn fn_8016A450() = crate::tu::adapt::fn_8016A450__cff474;
        fn fn_8016A46C() = crate::tu::adapt::fn_8016A46C__67847f;
        fn fn_8016B4BC() -> i32 = crate::tu::adapt::fn_8016B4BC__213ddc;
        fn fn_8016B510() -> i32 = crate::tu::adapt::fn_8016B510__59b051;
        fn fn_8016F030(a0: *mut StartMeleeData) = crate::tu::adapt::fn_8016F030__9a74e8;
        fn fn_8017EDDC() -> i32 = crate::tu::gm_17eb::fn_8017EDDC;
        fn ftBossLib_8015C2A8() -> i32 = crate::tu::adapt::ftBossLib_8015C2A8__1b0da4;
        fn ftBossLib_8015C530(a0: i32) -> i32 = crate::tu::adapt::ftBossLib_8015C530__eca1c0;
        fn ftBossLib_8015C74C() -> i32;
        fn ftBossLib_8015C7EC() -> i32;
        fn ftBossLib_8015C88C() -> i32;
        fn ftBossLib_8015C9CC() -> i32;
        fn ftBossLib_8015CA6C(a0: i32) = crate::tu::adapt::ftBossLib_8015CA6C__e73ae5;
        fn ftBossLib_8015CB7C() = crate::tu::adapt::ftBossLib_8015CB7C__08c5b8;
        fn ftBossLib_8015CB9C(a0: i32) = crate::tu::adapt::ftBossLib_8015CB9C__c24da3;
        fn ftBossLib_8015CC14() = crate::tu::adapt::ftBossLib_8015CC14__f4f13c;
        fn ftBossLib_IsMasterHandEntry() -> i32 = crate::tu::adapt::ftBossLib_IsMasterHandEntry__b6143e;
        fn ftCo_800C8C84(a0: *mut HSD_GObj) -> i32 = crate::tu::adapt::ftCo_800C8C84__d9cbd1;
        fn ftLib_80086824() = crate::tu::adapt::ftLib_80086824__82b58f;
        fn ftLib_800868A4() = crate::tu::adapt::ftLib_800868A4__7fb267;
        fn ftLib_80087508(a0: i8, a1: u8) = crate::tu::adapt::ftLib_80087508__b45233;
        fn ftLib_80087574(a0: i8) = crate::tu::adapt::ftLib_80087574__3ccbbb;
        fn ftLib_80087610(a0: u8) = crate::tu::adapt::ftLib_80087610__f2cb3c;
        fn ft_80087C58(a0: *mut Fighter_GObj) -> i32 = crate::tu::adapt::ft_80087C58__344dc5;
        fn ft_80087C64(a0: *mut Fighter_GObj, a1: i32) = crate::tu::adapt::ft_80087C64__aaf698;
        fn gmMainLib_8015D134(a0: u8) = crate::tu::adapt::gmMainLib_8015D134__50170c;
        fn gmMainLib_8015D194(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D194__9e9b6e;
        fn gmMainLib_8015D1AC(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D1AC__400144;
        fn gmMainLib_8015D1C8(a0: u8, a1: i32) = crate::tu::adapt::gmMainLib_8015D1C8__160fa4;
        fn gmMainLib_8015D25C(a0: u8) = crate::tu::adapt::gmMainLib_8015D25C__8fef81;
        fn gmMainLib_8015D2BC(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D2BC__88a357;
        fn gmMainLib_8015D2D4(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D2D4__48be73;
        fn gmMainLib_8015D2F0(a0: u8, a1: i32) = crate::tu::adapt::gmMainLib_8015D2F0__a48088;
        fn gmMainLib_8015D384(a0: u8) = crate::tu::adapt::gmMainLib_8015D384__d67253;
        fn gmMainLib_8015D3E4(a0: u8) -> *mut u8 = crate::tu::adapt::gmMainLib_8015D3E4__d26187;
        fn gmMainLib_8015D3FC(a0: u8) -> i32 = crate::tu::adapt::gmMainLib_8015D3FC__fe331f;
        fn gmMainLib_8015D418(a0: u8, a1: i32) = crate::tu::adapt::gmMainLib_8015D418__778d54;
        fn gmRegSetupEnemyColorTable(a0: i8, a1: u8, a2: *const i8, a3: *mut u8) = crate::tu::adapt::gmRegSetupEnemyColorTable__a8a833;
        fn gmVs_GetSceneController() -> *mut VsSceneController = crate::tu::adapt::gmVs_GetSceneController__400cec;
        fn gmVs_GetSceneState() -> *mut VsSceneState = crate::tu::adapt::gmVs_GetSceneState__53d82c;
        fn gm_801623D8() -> i32 = crate::tu::adapt::gm_801623D8__6fdb31;
        fn gm_801623FC(a0: i32) = crate::tu::adapt::gm_801623FC__a1a1bf;
        fn gm_8016247C(a0: i32) -> i32 = crate::tu::adapt::gm_8016247C__6fb2cc;
        fn gm_80162968(a0: u32) -> i32 = crate::tu::adapt::gm_80162968__8ae6df;
        fn gm_80164430(a0: u16) -> i32 = crate::tu::adapt::gm_80164430__8808be;
        fn gm_80164504(a0: u16) = crate::tu::adapt::gm_80164504__95f551;
        extern "C" fn gm_80165290(a0: i32) = crate::tu::adapt::gm_80165290__e2462d;
        fn gm_80167858(a0: i32, a1: i32, a2: i32, a3: i32) = crate::tu::adapt::gm_80167858__a75f4a;
        fn gm_8016A21C(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_8016A21C__1e76b5;
        fn gm_8016A22C(a0: i8, a1: i8, a2: i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: i32, a8: u8, a9: u8, a10: u8, a11: i32, a12: i32, a13: i32, a14: i32, a15: i32, a16: i32, a17: f32, a18: f32) = crate::tu::adapt::gm_8016A22C__e5b47e;
        fn gm_8016B328() = crate::tu::adapt::gm_8016B328__315877;
        fn gm_8016B33C(a0: i32) = crate::tu::adapt::gm_8016B33C__7b92b1;
        fn gm_8016B350(a0: i32) = crate::tu::adapt::gm_8016B350__3b3894;
        fn gm_8016EDDC(a0: i32, a1: *mut PlayerInitData) -> i32 = crate::tu::adapt::gm_8016EDDC__aa6fea;
        fn gm_80172898(a0: u16) -> i32 = crate::tu::gm_16f1::gm_80172898;
        fn gm_80173224(a0: i32, a1: i32) -> u8 = crate::tu::gm_16f1::gm_80173224;
        fn gm_80173754(a0: u8, a1: u8) -> i32 = crate::tu::gm_1736::gm_80173754;
        fn gm_8017390C(a0: i32, a1: i32) = crate::tu::gm_1736::gm_8017390C;
        fn gm_80173EEC() = crate::tu::gm_1736::gm_80173EEC;
        fn gm_CKindToSelKind(a0: u8) -> u8 = crate::tu::adapt::gm_CKindToSelKind__a9d153;
        fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
        fn gm_ClearDbPauseFlag(a0: i32) = crate::tu::adapt::gm_ClearDbPauseFlag__57b7e0;
        fn gm_GetAdventureData() -> *mut UnkAdventureData = crate::tu::gm_17e4::gm_GetAdventureData;
        fn gm_GetAllStarData() -> *mut UnkAllstarData = crate::tu::gm_17eb::gm_GetAllStarData;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gm_GetCurrentSceneIndex() -> u8 = crate::tu::adapt::gm_GetCurrentSceneIndex__5f49a4;
        fn gm_GetNumCostumesForCKind(a0: u8) -> u8 = crate::tu::adapt::gm_GetNumCostumesForCKind__a594a2;
        fn gm_InitChallengerData(a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) = crate::tu::gm_1736::gm_InitChallengerData;
        fn gm_IsCKindUnlocked(a0: u8) -> i32 = crate::tu::adapt::gm_IsCKindUnlocked__b6c64b;
        fn gm_SetDbPauseFlag(a0: i32) = crate::tu::adapt::gm_SetDbPauseFlag__fad6e4;
        fn gm_SetGameSpeed(a0: f32) = crate::tu::adapt::gm_SetGameSpeed__e4fa3d;
        fn gm_SetNewGameModePending() = crate::tu::adapt::gm_SetNewGameModePending__1e5e48;
        fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
        fn gm_SetPendingGameMode(a0: u8) = crate::tu::adapt::gm_SetPendingGameMode__90d26d;
        fn gm_SetupHumanPlayer(a0: *mut PlayerInitData, a1: u8, a2: u8, a3: u8, a4: u8) = crate::tu::adapt::gm_SetupHumanPlayer__13678a;
        fn gm_SetupPlayerDefaults(a0: *mut PlayerInitData) = crate::tu::adapt::gm_SetupPlayerDefaults__e9a9ae;
        fn grPushOn_80219230(a0: i32) -> i32 = crate::tu::adapt::grPushOn_80219230__68fe3c;
        fn lbAudioAx_800236DC() -> i32 = crate::tu::adapt::lbAudioAx_800236DC__5d9f3e;
        fn lbAudioAx_800237A8(a0: enum_t, a1: i32, a2: i32) -> i32 = crate::tu::adapt::lbAudioAx_800237A8__04460b;
        fn lbAudioAx_8002438C(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_8002438C__35c010;
        fn lbAudioAx_80028B6C() = crate::tu::adapt::lbAudioAx_80028B6C__bb9f49;
        fn lbAudioAx_80028B90() = crate::tu::adapt::lbAudioAx_80028B90__0093c2;
        fn lbBgFlash_800205F0(a0: i32) = crate::tu::adapt::lbBgFlash_800205F0__8f9470;
        fn lbBgFlash_8002063C(a0: i32) = crate::tu::adapt::lbBgFlash_8002063C__95303c;
        fn lbBgFlash_80020688(a0: i32) = crate::tu::adapt::lbBgFlash_80020688__4d10b5;
        fn lbBgFlash_800206D4(a0: *mut _GXColor, a1: *mut _GXColor, a2: i32) = crate::tu::adapt::lbBgFlash_800206D4__8952dd;
        fn lbCardGame_LoadArchive(a0: i32) = crate::tu::adapt::lbCardGame_LoadArchive__e9acb0;
        fn lbCardGame_SaveChanges() = crate::tu::adapt::lbCardGame_SaveChanges__0fc9e6;
        fn lbCardNew_AllocWorkArea() = crate::tu::adapt::lbCardNew_AllocWorkArea__a2c203;
        fn lbTime_8000AEC8(a0: u32, a1: u32) -> u32 = crate::tu::adapt::lbTime_8000AEC8__f87b66;
        fn lbTime_8000AF74(a0: u32, a1: i32) -> u32 = crate::tu::adapt::lbTime_8000AF74__3857e8;
        fn memzero(a0: *mut c_void, a1: ssize_t) = super::memzero_ssize;
    }
}

#[inline]
unsafe fn memzero_ssize(p: *mut c_void, n: ssize_t) {
    memzero(p, n)
}

/// `LAB_EV_BOSS_DEFEATED` (pc/gameworld/script_lab.h).
const LAB_EV_BOSS_DEFEATED: i32 = 7;
/// `BOSS_END_WAIT_MAX`: controller ticks the boss-end wait must not outlast.
const BOSS_END_WAIT_MAX: i32 = 300;

/// `GET_FIGHTER(gobj)`
#[inline]
unsafe fn fighter_of(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline]
fn osreport(s: &'static [u8]) {
    unsafe { ext::OSReport(s.as_ptr() as *mut i8) }
}

/// Extend this whitelist when another HP boss uses the 1P controller.
pub unsafe fn BossHook_Kind(kind: i32) -> i32 {
    (kind == Ft_Kind_MasterH as i32 || kind == Ft_Kind_CrezyH as i32) as i32
}

pub unsafe fn BossHook_AllSleeping() -> i32 {
    for port in 1..6 {
        let gobj = ext::Player_GetEntity(port);
        if !gobj.is_null() {
            let fp = fighter_of(gobj);
            if BossHook_Kind((*fp).kind as i32) != 0
                && ext::Player_GetRemainingHP(port) <= 0
                && (*fp).motion_id != ftCo_MS_Sleep as _
            {
                return 0;
            }
        }
    }
    1
}

/// A boss hand whose HP is 0 but that never ran the retail death is driven through the retail stamina KO
/// (`ftCo_800C8C84`), once per hand. Returns the number of hands driven.
pub unsafe fn BossHook_DriveDeaths() -> i32 {
    let mut n = 0;
    for port in 1..3 {
        let gobj = ext::Player_GetEntity(port);
        if !gobj.is_null() {
            let fp = fighter_of(gobj);
            if BossHook_Kind((*fp).kind as i32) != 0
                && ext::Player_GetRemainingHP(port) <= 0
                && (*fp).motion_id != ftCo_MS_Sleep as _
                && (*fp).x2224_b2() == 0
                && boss_hook.driven_mask & (1 << port) == 0
            {
                if ext::ftCo_800C8C84(gobj) != 0 {
                    boss_hook.driven_mask |= 1 << port;
                    osreport(b"boss_hook: hand port=%d kind=%d was at 0 HP without a retail death: driven through ftCo_800C8C84\n\0");
                    n += 1;
                }
            }
        }
    }
    n
}

pub unsafe fn BossHook_Poll() {
    for port in 1..6 {
        let bit = 1 << port;
        let gobj = ext::Player_GetEntity(port);
        if !gobj.is_null() && boss_hook.sent_mask & bit == 0 && ext::Player_GetRemainingHP(port) <= 0 {
            let fp = fighter_of(gobj);
            if BossHook_Kind((*fp).kind as i32) != 0 && (*fp).motion_id == ftCo_MS_Sleep as _ {
                let x = (*fp).cur_pos.x.to_bits() as i32;
                let y = (*fp).cur_pos.y.to_bits() as i32;
                boss_hook.sent_mask |= bit;
                boss_hook.pending = 1;
                ext::Script_GameEvent(LAB_EV_BOSS_DEFEATED, (*fp).kind as i32, port, x, y);
                osreport(b"boss_hook: defeated kind=%d port=%d\n\0");
            }
        }
    }
}

pub unsafe fn BossHook_Hold(frames: i32) -> i32 {
    if boss_hook.sent_mask == 0 || lbl_804706C0.x0 < 3 || lbl_804706C0.x0 > 8 {
        return 0;
    }
    boss_hook.held = 1;
    boss_hook.limit = frames;
    boss_hook.elapsed = 0;
    osreport(b"boss_hook: hold %d frames\n\0");
    1
}

pub unsafe fn BossHook_Release() -> i32 {
    if boss_hook.held == 0 {
        return 0;
    }
    boss_hook.held = 0;
    osreport(b"boss_hook: released\n\0");
    1
}

pub unsafe fn fn_8017C0C8() {
    let mut sp8: PlayerInitData = core::mem::zeroed();
    ext::gm_SetupPlayerDefaults(&mut sp8);
    sp8.slot_type = Gm_PKind_Cpu as u8;
    sp8.stocks = 1;
    sp8.cpu_kind = 4;
    sp8.cpu_level = ext::Player_GetCpuLevel(2) as u8;
    sp8.cpu_kind = ext::Player_GetCpuType(2) as u8;
    sp8.attack_ratio = ext::Player_GetAttackRatio(2);
    sp8.defense_ratio = ext::Player_GetUnk50(2);
    sp8.color = 0;
    sp8.ckind = CKind_CrezyH as i8;
    sp8.team = 4;
    sp8.set_xC_b7(1);
    sp8.hp = lbl_804706C0.xC as u16;
    sp8.set_xD_b2(1);
    sp8.set_xD_b0(1);
    sp8.set_xD_b2(1);
    sp8.spawn_dir = 1;
    ext::gm_8016EDDC(1, &mut sp8);
}

pub unsafe extern "C" fn fn_8017C1A4(_unused: *mut HSD_GObj) {
    let tmp = core::ptr::addr_of_mut!(lbl_804706C0);

    let temp_r31 = ext::ftBossLib_8015C74C();
    let temp_r29 = ext::ftBossLib_8015C7EC();
    let temp_r27 = ext::ftBossLib_8015C88C();
    let temp_r3 = ext::ftBossLib_8015C9CC();
    // Events are queued in the GObj pass and delivered by Script_FramePost.
    boss_hook.pending = 0;
    if boss_hook.held != 0 {
        boss_hook.elapsed += 1;
        if boss_hook.elapsed >= boss_hook.limit {
            boss_hook.held = 0;
            osreport(b"boss_hook: hold timed out\n\0");
        }
    }
    if (*tmp).x0 >= 3 && (*tmp).x0 <= 8 && ext::Script_BossHookEnabled() != 0 {
        BossHook_Poll();
    }
    if (*tmp).x0 >= 3 && (*tmp).x0 <= 8 {
        DRIVE_TICK += 1;
        if DRIVE_TICK % 10 == 0 {
            BossHook_DriveDeaths();
        }
    }
    match (*tmp).x0 {
        0 => {
            if (*tmp).x8 == 0 {
                let c = ext::gmVs_GetSceneController();
                (*c).start.set_disable_pausing(0);
                (*c).state.hud_enabled = 0;
                ext::lbAudioAx_800237A8(0x81650, 0x7F, 0x40);
                ext::ftLib_80086824();
                (*tmp).x8 = 1;
            } else if ext::ftBossLib_IsMasterHandEntry() == 0 {
                (*tmp).x0 = 2;
            }
        }
        1 => {
            if (*tmp).x8 == 0 {
                ext::gmVs_GetSceneController();
                ext::lbAudioAx_800236DC();
                ext::ftBossLib_8015CA6C(0x1D9);
                ext::Player_SetMoreFlagsBit4(0, 1);
                ext::Player_SetMoreFlagsBit3(0, 1);
                (*tmp).x10 = ext::ft_80087C58(ext::Player_GetEntity(0) as *mut Fighter_GObj);
                ext::ft_80087C64(ext::Player_GetEntity(0) as *mut Fighter_GObj, 2);
                (*tmp).x14 = ext::ft_80087C58(ext::Player_GetEntity(2) as *mut Fighter_GObj);
                ext::ft_80087C64(ext::Player_GetEntity(2) as *mut Fighter_GObj, 2);
                (*tmp).x8 = 1;
            } else if (*tmp).x8 == 1 {
                ext::lbAudioAx_800237A8(0x81651, 0x7F, 0x40);
                fn_8017C0C8();
                (*tmp).x8 = 2;
            } else if ext::ftBossLib_8015C2A8() == 0 {
                ext::gmVs_GetSceneController();
                ext::Stage_80225074(0);
                ext::ftBossLib_8015CB7C();
                ext::Player_SetMoreFlagsBit4(0, 0);
                ext::Player_SetMoreFlagsBit3(0, 0);
                ext::ft_80087C64(ext::Player_GetEntity(0) as *mut Fighter_GObj, (*tmp).x10);
                ext::ft_80087C64(ext::Player_GetEntity(2) as *mut Fighter_GObj, (*tmp).x14);
                ext::ftBossLib_8015CC14();
                (*tmp).x0 = 4;
            }
        }
        2 => {
            let c = ext::gmVs_GetSceneController();
            (*c).state.hud_enabled = 1;
            ext::Stage_80225074(0);
            ext::ftLib_800868A4();
            ext::ftBossLib_8015CC14();
            (*tmp).x0 = 3;
        }
        3 => {
            if ext::Player_GetRemainingHP(2) <= 0 {
                (*tmp).x0 = 7;
                (*tmp).x8 = 0;
            } else if (ext::Player_GetRemainingHP(2) as f32) <= 0.5f32 * ((*tmp).xC as f32) && ext::fn_8017EDDC() != 0 {
                (*tmp).x0 = 1;
                (*tmp).x8 = 0;
            }
        }
        4 => {
            if ext::Player_GetRemainingHP(2) <= 0 && ext::Player_GetRemainingHP(1) <= 0 {
                (*tmp).x0 = 7;
                (*tmp).x8 = 0;
            } else if ext::Player_GetRemainingHP(2) <= 0 || ext::Player_GetRemainingHP(1) <= 0 {
                (*tmp).x0 = 6;
                (*tmp).x8 = 0;
            }
        }
        5 => {
            if ext::Player_GetRemainingHP(2) <= 0 && ext::Player_GetRemainingHP(1) <= 0 {
                (*tmp).x0 = 7;
                (*tmp).x8 = 0;
            }
        }
        6 => {
            if ext::Player_GetRemainingHP(2) <= 0 {
                (*tmp).x4 = 2;
            } else {
                (*tmp).x4 = 1;
            }
            let temp_r27_2 = temp_r31 + temp_r29;
            if (*tmp).x8 == 0 {
                ext::ftBossLib_8015CB9C((*tmp).x4);
                ext::gm_SetDbPauseFlag(6);
            } else if (*tmp).x8 == temp_r31 {
                ext::lbAudioAx_8002438C(0x4E200);
                ext::ftBossLib_8015CC14();
                ext::gm_ClearDbPauseFlag(6);
                ext::gm_SetGameSpeed(0.5);
            } else if (*tmp).x8 >= temp_r27_2 {
                ext::gm_SetGameSpeed(1.0);
                (*tmp).x0 = 5;
            }
            if ext::Player_GetRemainingHP(2) <= 0 && ext::Player_GetRemainingHP(1) <= 0 {
                (*tmp).x0 = 7;
                (*tmp).x8 = 0;
            } else if (*tmp).x8 <= temp_r27_2 {
                (*tmp).x8 += 1;
            }
        }
        7 => {
            let temp_r28 = temp_r31 + temp_r27;
            let temp_r29_2 = temp_r3 + temp_r28;
            if (*tmp).x8 == 0 {
                let c = ext::gmVs_GetSceneController();
                (*c).state.hud_enabled = 0;
                if (*tmp).x4 == 2 {
                    (*tmp).x4 = 1;
                } else {
                    (*tmp).x4 = 2;
                }
                ext::ftBossLib_8015CB9C((*tmp).x4);
                ext::gm_SetDbPauseFlag(6);
                ext::Player_80031790(0);
                ext::Player_80036844(0, 1);
                let c = ext::gmVs_GetSceneController();
                (*c).start.set_timer_enabled(0);
            } else if (*tmp).x8 == temp_r31 {
                ext::lbAudioAx_8002438C(0x4E200);
                ext::ftBossLib_8015CC14();
                ext::gm_ClearDbPauseFlag(6);
                ext::gm_SetGameSpeed(0.5);
                ext::lbBgFlash_80020688(temp_r27);
            } else if (*tmp).x8 == temp_r28 {
                ext::lbBgFlash_800205F0(temp_r3);
            } else if (*tmp).x8 >= temp_r29_2 {
                // With no boss hook, take the original finish branch. A same-frame death event gets
                // FramePost to call Lua before the termination branch can run.
                if boss_hook.held != 0 || (ext::Script_BossHookEnabled() != 0 && (BossHook_AllSleeping() == 0 || boss_hook.pending != 0)) {
                    (*tmp).x0 = 8;
                    boss_hook.sleep_wait = 0;
                    // the variadic arguments are evaluated (BossHook_AllSleeping calls out), left to right
                    let _ = (BossHook_AllSleeping(), boss_hook.pending, boss_hook.held);
                    osreport(b"scene: boss-end wait sleep=%d pending=%d held=%d\n\0");
                    return;
                }
                ext::lbAudioAx_80028B90();
                ext::gm_SetGameSpeed(1.0);
                // The vanilla branch remains in state 7 until the VS scene exits.
                if (*ext::gmVs_GetSceneController()).state.terminate_match == 0 {
                    let _ = (ext::Player_GetRemainingHP(2), ext::Player_GetRemainingHP(1));
                    osreport(b"scene: boss-end request controller=7 hp_mh=%d hp_ch=%d\n\0");
                }
                ext::gm_8016B33C(8);
                ext::gm_8016B328();
                return;
            }
            if (*tmp).x8 >= temp_r31 {
                ext::lbAudioAx_80028B6C();
            }
            if (*tmp).x8 <= temp_r28 && (*tmp).x8 % 30 == 0 {
                ext::gm_80167858(ext::Player_GetPlayerId(0), ext::Player_GetNametagSlotID(0) as i32, 2, 0x1E);
                ext::Camera_RequestQuake(QuakeKind_Medium, core::ptr::null_mut());
            }
            if (*tmp).x8 <= temp_r29_2 {
                (*tmp).x8 += 1;
            }
        }
        8 => {
            // Wait for the death cleanup in ftCo_800BFD04, then for FramePost. A bad death animation cannot
            // strand the match: the same safety limit applies to that wait.
            if BossHook_AllSleeping() == 0 {
                boss_hook.sleep_wait += 1;
                if boss_hook.sleep_wait < BOSS_END_WAIT_MAX {
                    return;
                }
            }
            if boss_hook.pending != 0 {
                return;
            }
            if boss_hook.held != 0 {
                if boss_hook.restored == 0 {
                    let c = ext::gmVs_GetSceneController();
                    (*c).state.hud_enabled = 1;
                    // Keep the encounter clock stopped so targets cannot force a time-out.
                    ext::ftBossLib_8015CC14();
                    ext::gm_ClearDbPauseFlag(6);
                    ext::gm_SetGameSpeed(1.0);
                    ext::Player_80031848(0);
                    ext::Player_80036844(0, 0);
                    // State 7 ended on lbBgFlash's white-to-black fade, which covers the stage while the hook
                    // holds the match; fade it back out.
                    let mut black = _GXColor { r: 0, g: 0, b: 0, a: 255 };
                    let mut clear = _GXColor { r: 0, g: 0, b: 0, a: 0 };
                    ext::lbBgFlash_800206D4(&mut black, &mut clear, 20);
                    boss_hook.restored = 1;
                    osreport(b"boss_hook: gameplay resumed during hold\n\0");
                }
                return;
            }
            if boss_hook.sleep_wait >= BOSS_END_WAIT_MAX {
                osreport(b"boss_hook: death wait timed out\n\0");
            }
            boss_hook.held = 0;
            if boss_hook.restored != 0 {
                (*ext::gmVs_GetSceneController()).state.hud_enabled = 0;
                ext::Player_80031790(0);
                ext::Player_80036844(0, 1);
                ext::lbBgFlash_8002063C(20); // back to black, as the vanilla finish leaves it
            }
            ext::lbAudioAx_80028B90();
            ext::gm_SetGameSpeed(1.0);
            ext::gm_8016B33C(8);
            ext::gm_8016B328();
            let _ = (
                ext::Player_GetRemainingHP(2),
                ext::Player_GetRemainingHP(1),
                (*ext::gmVs_GetSceneController()).state.terminate_match,
            );
            osreport(b"scene: boss-end terminate controller=8 hp_mh=%d hp_ch=%d flag=%d\n\0");
            (*tmp).x0 = 9;
        }
        _ => {}
    }
}

pub unsafe extern "C" fn fn_8017C71C() {
    let tmp = core::ptr::addr_of_mut!(lbl_804706C0);
    ext::HSD_GObj_SetupProc(ext::GObj_Create(0xF, 0x11, 0), Some(fn_8017C1A4), 0x15);
    (*tmp).x0 = 0;
    (*tmp).x4 = 1;
    (*tmp).x8 = 0;
    boss_hook.sent_mask = 0;
    boss_hook.pending = 0;
    boss_hook.held = 0;
    boss_hook.restored = 0;
    boss_hook.limit = 0;
    boss_hook.elapsed = 0;
    boss_hook.sleep_wait = 0;
    boss_hook.driven_mask = 0;
    (*tmp).xC = ext::ftBossLib_8015C530(gm_8017E068());
    ext::Player_SetOtherStamina(2, (*tmp).xC);
    ext::ftLib_80087508(Ft_Kind_CrezyH as i8, 0);
}

pub unsafe extern "C" fn fn_8017C7A0() {
    ext::lbBgFlash_8002063C(0x30);
    if ext::Ground_801C1DAC() != 0 {
        ext::gm_8016B350(0x7C85E);
        return;
    }
    ext::gm_8016B350(0x9C48);
}

pub unsafe extern "C" fn fn_8017C7EC() {
    let temp_r30 = fn_8017DF28();
    let scene_state = ext::gmVs_GetSceneState();
    (*scene_state).timer_seconds = ext::grPushOn_80219230((*temp_r30).x0.ckind as i32) as _;
}

pub unsafe fn gm_8017C838() {
    let mut sp10: [i8; 3] = [Ft_Kind_None as i8; 3];

    let temp_r30 = ext::gm_GetAdventureData();
    let temp_r3 = ext::gmVs_GetSceneController();
    match (*temp_r3).start.stkind {
        0x3B => sp10[0] = Ft_Kind_Yoshi as i8,
        0x3F => sp10[0] = Ft_Kind_Link as i8,
        0x44 => {
            sp10[0] = Ft_Kind_Kirby as i8;
            let var_r3: u8 = if (*temp_r30).x0.x0.ckind as i32 == CKind_Kirby as i32 && (*temp_r30).x0.x0.color == 0 { 1 } else { 0 };
            ext::ftLib_80087610(var_r3);
        }
        0x48 => {
            // m-ex: the internal Pikachu id literal 0xC is Ft_Kind_Pikachu
            sp10[0] = Ft_Kind_Pikachu as i8;
            if ext::gm_IsCKindUnlocked(CKind_Pichu as u8) != 0 {
                sp10[1] = Ft_Kind_Pichu as i8;
            }
            if ext::gm_IsCKindUnlocked(CKind_Purin as u8) != 0 {
                sp10[2] = Ft_Kind_Purin as i8;
            }
        }
        0x4C => {
            sp10[0] = Ft_Kind_Popo as i8;
            sp10[1] = Ft_Kind_Nana as i8;
        }
        0x4E => {
            sp10[0] = Ft_Kind_Boy as i8;
            sp10[1] = Ft_Kind_Girl as i8;
        }
        _ => {}
    }

    for i in 0..3 {
        if sp10[i] as i32 != Ft_Kind_Max as i32 {
            ext::ftLib_80087574(sp10[i]);
        }
    }
    ext::Camera_SetQuakeScale(0.2);
}

pub unsafe fn gm_8017C984(arg0: *mut c_void) {
    ext::memzero(arg0, 0x74);
}

pub unsafe fn gm_8017C9A8(arg0: *mut DebugGameOverData, arg1: *mut Unk1PData, arg2: u8) {
    (*arg0).x0 = (*arg1).xC.x18 as _;
    (*arg0).x8 = arg2 as _;
    (*arg0).ckind = (*arg1).x0.ckind as _;
    (*arg0).slot = (*arg1).x0.slot as _;
    (*arg0).x15 = (*arg1).x0.nametag as _;
    (*arg0).x18 = ext::gm_801623D8() as _;
    (*arg0).x16 = lbl_803B7C08[arg2 as usize][(*arg1).x0.cpu_level as usize] as _;
}

pub unsafe fn gm_8017CA38(arg0: *mut DebugGameOverData, arg1: *mut Unk1PData, arg2: *mut gmm_x0_528_t, arg3: u8) {
    (*arg1).xC.x14 = (*arg0).x18 as _;
    if ext::Script_OnePScripted() == 0 {
        ext::gm_801623FC((*arg1).xC.x14 as i32);
    }
    if (*arg0).xC == 0 {
        ext::Script_OnePGameOver();
        if ext::Script_OnePScripted() != 0 {
            ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            return;
        }
        let temp_r31 = ext::gm_80173224(arg3 as i32, 0);
        match ext::gm_GetCurrentGameMode() as u32 {
            GM_CLASSIC => {
                ext::fn_80162BFC((*arg1).x0.ckind as i8, (*arg0).x4 as i32);
            }
            GM_ADVENTURE => {
                ext::fn_80162DF8((*arg1).x0.ckind as u8, (*arg0).x4 as u32);
            }
            GM_ALLSTAR => {
                ext::fn_80162FF4((*arg1).x0.ckind as u8, (*arg0).x4 as u32);
            }
            _ => {}
        }
        ext::Ground_801C5A60();
        ext::gm_8017390C(fn_8017DF90() as i32, 0);
        ext::gm_80173EEC();
        ext::gm_80172898(0x40);
        if temp_r31 as u32 == ChKind_None as u32 {
            if ext::gm_80173754(1, (*arg1).x0.slot) == 0 {
                ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
            }
        } else {
            ext::gm_InitChallengerData((*arg1).x0.ckind as u8, (*arg1).x0.color, (*arg1).x0.slot, (*arg1).x0.nametag, temp_r31, 1);
            ext::gm_ChangeGameModeAfterCurrentScene(GM_CHALLENGER_APPROACH as i32);
        }
    } else {
        (*arg1).xC.x18 = ext::lbTime_8000AEC8((*arg0).x4 as u32, 1) as _;
        (*arg1).x0.stocks = (*arg2).stocks as _;
        (*arg1).xC.xD = ext::lbTime_8000AF74((*arg1).xC.xD as u32, 1) as _;
        ext::gm_SetNextGameModeStateId((*arg1).x0.mode);
    }
}

pub unsafe fn gm_8017CBAC(arg0: *mut UnkAdventureData, arg1: *mut gmm_x0_528_t, arg2: u8) {
    if (ext::gm_GetCurrentGameMode() as u32 == GM_CLASSIC || ext::gm_GetCurrentGameMode() as u32 == GM_ADVENTURE)
        && ext::Script_OnePComplete() != 0
    {
        ext::Ground_801C5A60();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return; // No trophy/credits/card records for a scripted run.
    }
    let temp_r31 = ext::gm_CKindToSelKind((*arg0).x0.x0.ckind as u8);
    ext::Ground_801C5A60();
    match arg2 {
        21 => {
            ext::gmMainLib_8015D134(temp_r31);
            ext::fn_80162BFC((*arg0).x0.x0.ckind as i8, (*arg0).x0.xC.x18 as i32);
            let temp_r3 = ext::gmMainLib_8015D194(temp_r31);
            if *temp_r3 < (*arg0).x0.x0.cpu_level {
                *temp_r3 = (*arg0).x0.x0.cpu_level;
            }
            let temp_r3_2 = ext::gmMainLib_8015D1AC(temp_r31);
            if (*arg0).x0.x0.cpu_level == 4 && (temp_r3_2 == 0 || ((*arg1).stocks as i32) < temp_r3_2) {
                ext::gmMainLib_8015D1C8(temp_r31, (*arg1).stocks as i32);
            }
        }
        22 => {
            ext::gmMainLib_8015D25C(temp_r31);
            ext::fn_80162DF8((*arg0).x0.x0.ckind as u8, (*arg0).x0.xC.x18 as u32);
            let temp_r3 = ext::gmMainLib_8015D2BC(temp_r31);
            if *temp_r3 < (*arg0).x0.x0.cpu_level {
                *temp_r3 = (*arg0).x0.x0.cpu_level;
            }
            let temp_r3_2 = ext::gmMainLib_8015D2D4(temp_r31);
            if (*arg0).x0.x0.cpu_level == 4 && (temp_r3_2 == 0 || ((*arg1).stocks as i32) < temp_r3_2) {
                ext::gmMainLib_8015D2F0(temp_r31, (*arg1).stocks as i32);
            }
        }
        23 => {
            ext::gmMainLib_8015D384(temp_r31);
            ext::fn_80162FF4((*arg0).x0.x0.ckind as u8, (*arg0).x0.xC.x18 as u32);
            if ext::gm_80164430(0x1F) == 0 {
                ext::gm_80164504(0x1F);
            }
            let temp_r3 = ext::gmMainLib_8015D3E4(temp_r31);
            if *temp_r3 < (*arg0).x0.x0.cpu_level {
                *temp_r3 = (*arg0).x0.x0.cpu_level;
            }
            let temp_r3_2 = ext::gmMainLib_8015D3FC(temp_r31);
            if (*arg0).x0.x0.cpu_level == 4 && (temp_r3_2 == 0 || ((*arg1).stocks as i32) < temp_r3_2) {
                ext::gmMainLib_8015D418(temp_r31, (*arg1).stocks as i32);
            }
        }
        _ => {}
    }
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    ext::lbCardGame_SaveChanges();
    ext::gm_SetPendingGameMode(arg2);
    ext::gm_SetNewGameModePending();
}

pub unsafe fn gm_8017CD94(arg0: *mut UnkAdventureData, arg1: i32, arg2: i32, arg3: i32) -> u8 {
    let num_colors = ext::gm_GetNumCostumesForCKind(arg1 as u8);
    if let Some(f) = (*arg0).x0.x54 {
        let result = f(arg2 as u8, (*arg0).x0.x0.cpu_level, arg3 as u8);
        if num_colors != 0 {
            return result % num_colors;
        }
        return 0;
    }
    0
}

pub unsafe fn gm_8017CE34_CountEnemies(arg0: *const i8) -> i32 {
    let mut count = 0;
    for i in 0..3 {
        // m-ex: the external-id null literal 0x21 is ChKind_None
        if *arg0.add(i) as i32 != ChKind_None as i32 {
            count += 1;
        }
    }
    count
}

pub unsafe fn gm_8017CE34_SetupColors(arg1: *mut Unk1PData, count: i32, arg2: *mut i8, colors: *mut u8) {
    for color_idx in 0..3usize {
        let num_colors = ext::gm_GetNumCostumesForCKind(*arg2.add(color_idx) as u8);
        let result;
        if let Some(f) = (*arg1).x54 {
            let r = f(count as u8, (*arg1).x0.cpu_level, color_idx as u8);
            result = if num_colors != 0 { r % num_colors } else { 0 };
        } else {
            result = 0;
        }
        *colors.add(color_idx) = result;
    }
}

pub unsafe fn gm_8017CE34_GetCpuLevel(arg1: *mut Unk1PData) -> u8 {
    (*arg1).x0.cpu_level
}

pub unsafe fn gm_8017CE34(
    arg0: *mut StartMeleeData,
    arg1: *mut Unk1PData,
    arg2: *mut i8,
    arg3: u8,
    arg4: u8,
    arg5: u8,
    arg6: i32,
    arg7: StKind,
    count: i32,
    arg9: i32,
) {
    let mut colors = [0u8; 16];
    let mut enemy_level: u8 = 0;
    let mut boss_count: i32 = 0;
    let mut enemy_cpu_type: u8 = 0;
    let player_idx: i32;
    let mut player_idx_m: i32;

    ext::Script_OnePStage(ext::gm_GetCurrentGameMode() as i32, count, (*arg1).x8 as i32, 0);
    (*arg1).xC.xC = 1;
    fn_8017E21C();
    ext::fn_8016F030(arg0);
    (*arg0).rules.x30 = 1.0;
    (*arg0).rules.set_x4_4(1);

    match arg9 {
        0 => (*arg0).rules.xA = 0,
        1 => (*arg0).rules.xA = 3,
        2 => (*arg0).rules.xA = 2,
        _ => {}
    }

    (*arg0).rules.set_x1_1(1);
    (*arg0).rules.set_x4_2(1);
    (*arg0).rules.set_x4_3(1);
    (*arg0).rules.set_x2_5(0);
    (*arg0).rules.set_x0_3((*arg1).xB as _);

    if arg6 != 0 {
        (*arg0).rules.time_limit = arg6 as u32;
        (*arg0).rules.set_timer_enabled(1);
    } else {
        (*arg0).rules.set_timer_enabled(0);
    }

    (*arg0).rules.x18 = (*arg1).xC.x18 as u32;
    (*arg0).rules.stkind = arg7 as u16;
    (*arg0).rules.item_freq = ((*arg1).x48.unwrap())(count as u8, (*arg1).x0.cpu_level) as i8;

    (*arg0).rules.x20 = (-1i32) as i64 as u64;

    let enemy_count = gm_8017CE34_CountEnemies(arg2);
    if enemy_count == 0 {
        (*arg0).rules.set_x5_1(1);
    }
    if (*arg1).x8 & 1 != 0 {
        (*arg0).rules.set_x2_7(1);
    }
    if (*arg1).x8 & 0x20 != 0 {
        (*arg0).rules.set_x4_5(1);
    }
    if (*arg1).x8 & 0x80 != 0 {
        (*arg1).xC.xC = 5;
        match (*arg1).x9 as i32 {
            1 => {
                (*arg0).rules.on_pause_override = Some(ext::gm_80165290);
                (*arg0).rules.x9 = 1;
                (*arg0).rules.set_x3_1(0);
                (*arg0).players[0].set_xC_b1(0);
            }
            2 => {
                (*arg0).rules.x9 = 3;
                (*arg0).rules.on_match_start = Some(fn_8017C7EC);
            }
            3 => {
                (*arg0).rules.x9 = 2;
                (*arg0).rules.x7 = 9;
                (*arg0).rules.set_x4_4(0);
                (*arg0).rules.on_match_end =
                    Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(u8)>(fn_8017C7A0));
                (*arg0).rules.xD = 0x30;
            }
            _ => {}
        }
    }

    let player_stocks: u8 = if (*arg1).x8 & 0x80 != 0 { 1 } else { (*arg1).x0.stocks };

    let player_ckind: u8;
    if (*arg1).x0.ckind as u32 == CKind_Zelda && (*arg1).xC.x12 != 0 {
        player_ckind = CKind_Seak as u8;
    } else if (*arg1).x8 & 0x80 != 0 && (*arg1).x9 == 1 && (*arg1).x0.ckind as u32 == CKind_PopoNana {
        player_ckind = ChKind_Popo as u8;
    } else {
        player_ckind = (*arg1).x0.ckind as u8;
    }

    ext::gm_SetupHumanPlayer((*arg0).players.as_mut_ptr(), player_ckind, (*arg1).x0.color, player_stocks, (*arg1).x0.slot);
    (*arg0).players[0].nametag = (*arg1).x0.nametag;
    (*arg0).players[0].spawn_dir = (*arg1).xA as i8;

    {
        let team_color: u8 = match (*arg1).x58 {
            Some(f) => f(count as u8, gm_8017CE34_GetCpuLevel(arg1), 0),
            None => 0,
        };
        player_idx = fn_8017DD7C((*arg0).players.as_mut_ptr(), (*arg1).xC.x24.as_mut_ptr(), team_color) + 1;
        (*arg0).rules.is_teams = 1;
    }
    player_idx_m = player_idx;

    let attack_ratio = ((*arg1).x64.unwrap())(count as u8, (*arg1).x0.cpu_level);
    let defense_ratio = ((*arg1).x68.unwrap())(count as u8, (*arg1).x0.cpu_level);

    let mut color_iter: usize = 0;
    gm_8017CE34_SetupColors(arg1, count, arg2, colors.as_mut_ptr());

    {
        let temp_r3_4 = (*arg1).x8 & 8;
        if temp_r3_4 != 0 && (*arg1).xC.x11 == 0 {
            let base_enemy_count = gm_8017CE34_CountEnemies(arg2);
            (*arg1).xC.xC = 3;
            let mut event_enemy_count = base_enemy_count;
            let mut special_stage = 0;
            let mut special_enemy_mode = 0;
            let sp8: u8 = 0;
            if let Some(f) = (*arg1).x4C {
                enemy_level = f(count as u8, (*arg1).x0.cpu_level, 0);
            }

            let first_enemy = *arg2 as u8;
            // m-ex: the special-enemy classification in terms of the external-id enums (Kirby, wireframes
            // Boy/Girl, Pikachu)
            if first_enemy as i8 as i32 != CKind_Kirby as i32 {
                if first_enemy.wrapping_sub(CKind_Boy as u8) <= (CKind_Girl as u8).wrapping_sub(CKind_Boy as u8) {
                    (*arg0).rules.set_x0_3(6);
                    event_enemy_count = 5;
                    colors[0] = 0;
                    special_stage = 1;
                    special_enemy_mode = 1;
                } else if first_enemy as i8 as i32 == CKind_Pikachu as i32
                    && (*arg2.add(1) as i32 != CKind_Pikachu as i32 || *arg2.add(2) as i32 != CKind_Pikachu as i32)
                {
                    special_enemy_mode = 2;
                }
            }

            let stage_flags = ext::Ground_801C5AD0(ext::Stage_8022519C(arg7) as i32);

            ext::gm_8016A22C(
                *arg2,
                *arg2.add(1),
                *arg2.add(2),
                colors[0],
                colors[1],
                colors[2],
                special_stage,
                special_enemy_mode,
                sp8,
                (*arg1).x0.ckind as u8,
                (*arg1).x0.color,
                enemy_level as i32,
                arg3 as i32,
                event_enemy_count,
                stage_flags as i32,
                arg5 as i32,
                arg4 as i32,
                attack_ratio,
                defense_ratio,
            );
            ext::gm_8016A21C(&mut (*arg0).rules);
            (*arg1).xC.x11 = 0;
            if (*arg1).x8 & 4 != 0 {
                ext::fn_8016A450();
            }
            if special_enemy_mode as u8 == 1 {
                ext::fn_8016A46C();
                (*arg0).players[0].set_xC_b5(1);
            }
        } else if temp_r3_4 != 0 {
            ext::gm_8016A21C(&mut (*arg0).rules);
            (*arg1).xC.x11 = 0;
        }
    }

    ext::gmRegSetupEnemyColorTable((*arg1).x0.ckind as u8 as i8, (*arg1).x0.color, arg2, colors.as_mut_ptr());

    let mut enemy_idx: i32 = 0;
    loop {
        let enemy_kind = arg2.add(enemy_idx as usize);
        // m-ex: the external-id null literal 0x21 is ChKind_None
        if *enemy_kind as u8 as i32 != ChKind_None as i32 {
            let p = &mut (*arg0).players[player_idx_m as usize] as *mut PlayerInitData;
            if (*arg1).x8 & 8 != 0 {
                if let Some(f) = (*arg1).x4C {
                    enemy_level = f(count as u8, (*arg1).x0.cpu_level, 0);
                }
                if let Some(f) = (*arg1).x50 {
                    enemy_cpu_type = f(count as u8, (*arg1).x0.cpu_level, 0);
                }
            } else {
                if let Some(f) = (*arg1).x4C {
                    let selected_enemy_level = f(count as u8, (*arg1).x0.cpu_level, enemy_idx as u8);
                    enemy_level = selected_enemy_level;
                }
                if let Some(f) = (*arg1).x50 {
                    enemy_cpu_type = f(count as u8, (*arg1).x0.cpu_level, enemy_idx as u8);
                }
            }
            ext::gm_SetupPlayerDefaults(p);
            (*p).slot_type = 1;
            (*p).ckind = *enemy_kind as u8 as i8;
            (*p).stocks = 1;
            (*p).cpu_level = enemy_level;
            (*p).cpu_kind = enemy_cpu_type;
            (*p).attack_ratio = attack_ratio;
            (*p).defense_ratio = defense_ratio;
            (*p).color = colors[color_iter];
            if (*arg1).x8 & 2 != 0 {
                (*p).model_scale = 2.0;
                (*p).xB = 2;
            } else {
                (*p).model_scale = 1.0;
                (*p).xB = 0;
            }
            if (*arg1).x8 & 4 != 0 {
                (*p).set_vs_metal(1);
                (*p).cpu_kind = 0x1B;
            }
            if (*p).ckind as i32 == CKind_GKoops as i32 {
                (*p).set_xC_b1(0);
            }
            let enemy_ckind = (*p).ckind as u8;
            if enemy_ckind as i8 as i32 == CKind_MasterH as i32 || enemy_ckind as i8 as i32 == CKind_CrezyH as i32 {
                (*p).set_xC_b7(1);
                (*p).hp = 0x12C;
                (*p).set_xD_b2(1);
                (*p).set_xD_b0(1);
                (*p).set_xD_b2(1);
                (*p).spawn_dir = -1;
                if (*p).ckind as i32 == CKind_CrezyH as i32 {
                    (*p).slot_type = 3;
                }
                boss_count += 1;
            }
            if (*arg0).rules.is_teams == 1 {
                (*p).team = 4;
            }
            player_idx_m += 1;
            if player_idx_m >= 6 {
                break;
            }
        } else {
            // m-ex: the external Master Hand id literal 0x1A is CKind_MasterH
            if enemy_idx == 0 && *enemy_kind.add(1) as i32 == CKind_MasterH as i32 {
                (*arg0).players[player_idx_m as usize].slot_type = 3;
                player_idx_m += 1;
            }
        }
        enemy_idx += 1;
        color_iter += 1;
        if enemy_idx >= 3 {
            break;
        }
    }

    {
        let mut i = player_idx_m;
        while i < 6 {
            (*arg0).players[i as usize].slot_type = 3;
            i += 1;
        }
    }

    let flags = (*arg1).x8 as i32;
    if flags & 0x40 != 0 {
        (*arg1).xC.xC = 7;
    } else if flags & 1 != 0 {
        if flags & 8 != 0 && arg3 > 1 {
            (*arg1).xC.xC = 4;
        } else {
            (*arg1).xC.xC = 2;
        }
    }
    if boss_count != 0 {
        (*arg0).rules.set_x1_2(1);
        (*arg0).rules.set_x1_3(1);
        (*arg0).rules.set_x1_4(1);
        (*arg0).rules.set_x0_3(3);
        (*arg0).rules.set_disable_pausing(1);
        (*arg0).rules.x7 = 0;
        (*arg0).rules.on_match_start = Some(fn_8017C71C);
        (*arg1).xC.xC = 6;
    }
    if arg7 == 0x49 {
        (*arg1).xC.xC = 8;
    }
}

pub unsafe fn gm_8017D7AC(arg0: *mut MatchExitInfo, arg1: *mut Unk1PData, arg2: u8) -> i32 {
    (*arg1).xC.x14 = ext::lbTime_8000AEC8((*arg1).xC.x14 as u32, (*arg0).match_end.player_standings[0].xE as u32) as _;
    if ext::Script_OnePScripted() == 0 {
        ext::gm_8016247C((*arg0).match_end.player_standings[0].xE as i32);
    }
    (*arg1).xC.x1C = ext::lbTime_8000AEC8((*arg1).xC.x1C as u32, (*arg0).match_end.player_standings[0].x44 as u32) as _;
    if ext::Script_OnePScripted() == 0 {
        ext::gm_80162968((*arg0).match_end.frame_count / 60);
    }
    if (*arg1).x8 & 0x80 == 0 {
        (*arg1).xC.x20 = ext::lbTime_8000AEC8((*arg1).xC.x20 as u32, (*arg0).match_end.frame_count) as _;
    }
    if (*arg0).match_end.player_standings[0].ftkind == 7 {
        (*arg1).xC.x12 = 1;
    } else {
        (*arg1).xC.x12 = 0;
    }
    if ext::fn_8016B510() != 0 {
        (*arg1).xC.xF = 1;
    }
    if ext::fn_8016B4BC() != 0 {
        (*arg1).xC.xE = 1;
    }
    let temp_r0 = (*arg0).match_end.outcome;
    if (temp_r0 as u32 == OUTCOME_NO_CONTEST as u32 || temp_r0 as u32 == OUTCOME_RETRY as u32)
        && ext::DbLevel <= DbLKind_DebugDevelop as DbLKind
    {
        ext::Script_OnePGameOver();
        if ext::Script_OnePScripted() == 0 {
            match ext::gm_GetCurrentGameMode() as u32 {
                GM_CLASSIC => {
                    ext::fn_80162BFC((*arg1).x0.ckind as i8, (*arg1).xC.x18 as i32);
                }
                GM_ADVENTURE => {
                    ext::fn_80162DF8((*arg1).x0.ckind as u8, (*arg1).xC.x18 as u32);
                }
                GM_ALLSTAR => {
                    ext::fn_80162FF4((*arg1).x0.ckind as u8, (*arg1).xC.x18 as u32);
                }
                _ => {}
            }
        }
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return 0;
    }
    if ext::Script_OnePScripted() == 0 {
        fn_8017E3C8();
    }
    if (*arg1).x8 & 0x80 == 0 {
        (*arg1).x0.stocks = (*arg0).match_end.player_standings[0].stocks as _;
        if (*arg1).x0.stocks != 0 {
            if (*arg0).match_end.outcome as u32 == OUTCOME_TIMEOUT as u32 {
                (*arg1).x0.stocks = (*arg1).x0.stocks.wrapping_sub(1);
                if (*arg1).x0.stocks == 0 {
                    ext::gm_SetNextGameModeStateId(arg2);
                    return 0;
                }
                if (*arg1).x8 & 0x40 == 0 {
                    (*arg1).xC.x10 = (*arg1).xC.x10.wrapping_add(1);
                    ext::gm_SetNextGameModeStateId(ext::gm_GetCurrentSceneIndex());
                    return 0;
                }
            }
        } else {
            (*arg1).xC.x10 = 0;
            ext::gm_SetNextGameModeStateId(arg2);
            return 0;
        }
    }
    (*arg1).xC.x18 = (*arg0).x0 as _;
    (*arg1).xC.x10 = 0;
    ext::Script_OnePClear();
    1
}

pub unsafe fn pick_random_ckind(arr: *mut u8, used_ckinds: *const u8, preset_ckinds: *const u8) -> i32 {
    let base = arr;
    let mut cursor = base;
    let mut count: i32 = 0;
    while *cursor as i32 != ChKind_None as i32 {
        cursor = cursor.add(1);
        count += 1;
    }

    let mut w = base;
    for _ in 0..count {
        let temp = *w;
        let swap_idx = ext::HSD_Randi(count);
        *w = *base.add(swap_idx as usize);
        w = w.add(1);
        *base.add(swap_idx as usize) = temp;
    }

    let mut w2 = base;
    for i in 0..count {
        if ext::gm_IsCKindUnlocked(*w2) != 0 {
            let ckind = *w2;
            let mut used = false;
            for used_idx in 0..4 {
                if ckind as i8 == *used_ckinds.add(used_idx) as i8 {
                    used = true;
                    break;
                }
            }
            if !used {
                let mut preset = false;
                for preset_idx in 0..3 {
                    if ckind as i8 == *preset_ckinds.add(preset_idx) as i8 {
                        preset = true;
                        break;
                    }
                }
                if !preset {
                    return *base.add(i as usize) as i32;
                }
            }
        }
        w2 = w2.add(1);
    }

    ChKind_None as i32
}

pub unsafe fn fn_8017D9C0(used_ckinds: *const u8, preset_ckinds: *const u8) -> i32 {
    pick_random_ckind(ext::lbl_803D79F0.as_mut_ptr(), used_ckinds, preset_ckinds)
}

pub unsafe fn gm_8017DB58(arg0: *mut Unk1PData_x24) {
    for i in 0..3 {
        (*arg0.add(i)).ckind = ChKind_None as _;
    }
}

pub unsafe fn gm_8017DB6C(arg0: *mut gm_8017DB6C_arg0_t, index: i32) -> u8 {
    (*arg0.offset(index as isize)).x0
}

pub unsafe fn gm_8017DB78(arg0: *mut gm_8017DB6C_arg0_t, index: i32) -> u8 {
    (*arg0.offset(index as isize)).x1
}

pub unsafe fn gm_8017DB88(
    arg0: *mut c_void,
    arg1: u8,
    arg2: i32,
    arg3: i32,
    arg4: *mut u8,
    arg5: u8,
    arg6: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>,
    arg7: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>,
    arg8: Option<unsafe extern "C" fn(i32, i32, u8) -> u8>,
    arg9: Option<unsafe extern "C" fn(i32, i32) -> f32>,
    arg10: Option<unsafe extern "C" fn(i32, i32) -> f32>,
) -> i32 {
    let mut chars = [0u8; 4];

    let count = fn_8017DE54(arg1, arg4);
    // m-ex: the external-id null literal 0x21 is ChKind_None
    chars[1] = ChKind_None as u8;
    let mut out = arg0 as *mut RegClearCharEntry;
    chars[2] = ChKind_None as u8;
    let mut pi: usize = 0;
    let mut i = 0;
    chars[3] = ChKind_None as u8;
    chars[0] = arg5;
    while i < count {
        chars[pi + 1] = fn_8017D9C0(chars.as_ptr(), arg4) as u8;
        (*out).x0 = chars[pi + 1];
        let val = match arg8 {
            Some(f) => f(arg3, arg2, i as u8) as i32,
            None => 0,
        };
        (*out).x1 = val as u8;
        let val = match arg6 {
            Some(f) => f(arg3, arg2, i as u8) as i32,
            None => 0,
        };
        (*out).x2 = val as u8;
        let val = match arg7 {
            Some(f) => f(arg3, arg2, i as u8) as i32,
            None => 0,
        };
        (*out).x3 = val as u8;
        let fval = match arg9 {
            Some(f) => f(arg3, arg2),
            None => 0.0,
        };
        (*out).x4 = fval;
        let fval = match arg10 {
            Some(f) => f(arg3, arg2),
            None => 0.0,
        };
        (*out).x8 = fval;
        pi += 1;
        out = out.add(1);
        i += 1;
    }
    while i < 3 {
        // m-ex: the external-id null literal 0x21 is ChKind_None
        (*(arg0 as *mut RegClearCharEntry).add(i as usize)).x0 = ChKind_None as u8;
        i += 1;
    }
    count
}

pub unsafe fn fn_8017DD7C(arg0: *mut PlayerInitData, arg1: *mut Unk1PData_x24, _arg2: u8) -> i32 {
    let mut index: usize = 1;
    for i in 0..3 {
        if (*arg1.add(i)).ckind as i32 != ChKind_None as i32 {
            ext::gm_SetupPlayerDefaults(arg0.add(index));
            let p = arg0.add(index);
            (*p).ckind = (*arg1.add(i)).ckind as _;
            (*p).slot_type = 1;
            (*p).stocks = 1;
            (*p).team = (*arg0).team;
            (*p).color = (*arg1.add(i)).color;
            (*p).cpu_kind = (*arg1.add(i)).cpu_kind;
            (*p).cpu_level = (*arg1.add(i)).cpu_level;
            (*p).attack_ratio = (*arg1.add(i)).attack_ratio;
            (*p).defense_ratio = (*arg1.add(i)).defense_ratio;
            (*p).set_xD_b1(1);
            if (*p).ckind as i32 == CKind_GKoops as i32 {
                (*p).set_xC_b1(0);
            }
            index += 1;
        }
    }
    (index - 1) as i32
}

pub unsafe fn fn_8017DE54(arg0: u8, arg1: *mut u8) -> i32 {
    if arg0 & 0x20 != 0 {
        return 0;
    }
    if arg0 & 0x10 != 0 {
        let mut count = 0;
        // m-ex: the external-id null literal 0x21 is ChKind_None
        if *arg1 as i32 != ChKind_None as i32 {
            count = 1;
        }
        let p = arg1.add(1);
        if *p as i32 != ChKind_None as i32 {
            count += 1;
        }
        if *p.add(1) as i32 != ChKind_None as i32 {
            count += 1;
        }
        return 3 - count;
    }
    if arg0 & 2 != 0 {
        return 2;
    }
    0
}

pub unsafe fn fn_8017DEC8(arg0: i32) -> *mut Unk1PData {
    match arg0 {
        0 => core::ptr::addr_of_mut!((*ext::gm_GetAdventureData()).x0),
        1 => core::ptr::addr_of_mut!((*ext::gm_GetAllStarData()).x0),
        2 => core::ptr::addr_of_mut!(ext::gm_80473A18.x0),
        _ => core::ptr::null_mut(),
    }
}

pub unsafe fn fn_8017DF28() -> *mut Unk1PData {
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC => core::ptr::addr_of_mut!((*ext::gm_GetAllStarData()).x0),
        GM_ADVENTURE => core::ptr::addr_of_mut!((*ext::gm_GetAdventureData()).x0),
        GM_ALLSTAR => core::ptr::addr_of_mut!(ext::gm_80473A18.x0),
        _ => core::ptr::null_mut(),
    }
}

pub unsafe fn fn_8017DF90() -> u8 {
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC => 1,
        GM_ADVENTURE => 0,
        GM_ALLSTAR => 2,
        _ => 3,
    }
}

pub unsafe fn gm_8017DFF4(arg0: i32) -> i32 {
    let var_r3 = fn_8017DEC8(arg0);
    if !var_r3.is_null() {
        return (*var_r3).x0.cpu_level as i32;
    }
    -1
}

pub unsafe fn gm_8017E068() -> i32 {
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        return (*var_r3).x0.cpu_level as i32;
    }
    -1
}

pub unsafe fn fn_8017E0E4() -> i32 {
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        return (*var_r3).xC.xC as i32;
    }
    -1
}

pub unsafe fn fn_8017E160() -> i32 {
    let var_r3 = fn_8017DF28();
    if !var_r3.is_null() {
        for i in 0..3 {
            if (*var_r3).xC.x24[i].ckind as i32 != ChKind_None as i32 {
                return 1;
            }
        }
        return 0;
    }
    0
}

pub unsafe fn fn_8017E21C() {
    for i in 0..12 {
        lbl_804706D8[i].x0 = -1;
    }
}

pub unsafe fn gm_8017E280(arg0: u16, arg1: u32) -> i8 {
    let var_r7 = lbl_804706D8.as_mut_ptr();
    for index in 0..12 {
        if arg0 as i32 == (*var_r7.add(index)).x0 as i32 {
            (*var_r7.add(index)).x2 = (*var_r7.add(index)).x2.wrapping_add(arg1 as u8);
            return (*var_r7.add(index)).x2 as i8;
        }
    }
    for index in 0..12 {
        if (*var_r7.add(index)).x0 == -1 {
            (*var_r7.add(index)).x0 = arg0 as i16;
            (*var_r7.add(index)).x2 = arg1 as u8;
            return arg1 as u8 as i8;
        }
    }
    -1
}

pub unsafe fn fn_8017E318() -> i32 {
    let mut sum: i32 = 0;
    for i in 0..12 {
        if lbl_804706D8[i].x0 != -1 {
            sum += lbl_804706D8[i].x2 as i32;
        }
    }
    sum
}

pub unsafe fn fn_8017E3C8() {
    for i in 0..12 {
        if lbl_804706D8[i].x0 != -1 {
            ext::Toy_SetUnlockState(lbl_804706D8[i].x0 as enum_t, lbl_804706D8[i].x2 as i32);
        }
    }
}
