//! src/melee/gm/gm_1BFA.c: the Challenger Approach game mode (the "a new challenger appears" fight):
//! its scene table, the approach / vs / prize scene hooks, and the prize list builder. Every function
//! is checked bit for bit against the C by a state oracle (tests/tu_gm_1BFA.rs over
//! gen/turef/tu_gm_1BFA_ref.c): the same seeded stream drives the callee results, the callees log
//! their arguments, and the log plus the data the functions write are compared.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use crate::lb_r6::memzero;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// `enum { state_approach, state_vs, state_prize }`.
pub const state_approach: u8 = 0;
pub const state_vs: u8 = 1;
pub const state_prize: u8 = 2;

/// `static u8 gm_8049E558[0x170]`: per-id "listed" flags (0x42 CKind-style ids, then per-trophy slots).
pub static mut gm_8049E558: [u8; 0x170] = [0; 0x170];

/// Callees in TUs that are not ported yet (gm, gmmain_lib, toy, lbtime, lbcard, HSD memory) and the
/// globals of other TUs the table points at. Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// `gm_GetGameModeStateEnterData(scene)` / `gm_GetGameModeStateExitData(scene)` (gm/gm_1A3F.c):
    /// the scene's info pointers.
    pub fn gm_GetGameModeStateEnterData(scene: *mut GameModeState) -> *mut c_void {
        unsafe { (*scene).info.enter_data }
    }
    pub fn gm_GetGameModeStateExitData(scene: *mut GameModeState) -> *mut c_void {
        unsafe { (*scene).info.exit_data }
    }
    hook!(GM_GETCHALLENGERDATA, gm_GetChallengerData, () -> *mut ChallengerData);
    hook!(GM_SETUPRULESDEFAULTS, gm_SetupRulesDefaults, (rules: *mut StartMeleeRules));
    hook!(GM_SETUPALLPLAYERDEFAULTS, gm_SetupAllPlayerDefaults, (players: *mut PlayerInitData));
    hook!(GM_GETCHALLENGERSTKIND, gm_GetChallengerStKind, (sel_kind: SelectableCharacterKind) -> StKind);
    hook!(GM_CKINDTOSELKIND, gm_CKindToSelKind, (ckind: u8) -> u8);
    hook!(GM_SETUPHUMANPLAYER, gm_SetupHumanPlayer, (p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8));
    hook!(GM_SETUPCPUPLAYER, gm_SetupCpuPlayer, (p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8));
    hook!(GM_DECIDECHALLENGERCPULEVEL, gm_DecideChallengerCpuLevel, (cpu_ckind: u8, human_nametag: u8) -> u8);
    hook!(GM_LOADRUMBLEENABLED, gm_LoadRumbleEnabled, (start: *mut StartMeleeData));
    hook!(GM_80162968, gm_80162968, (seconds: u32) -> i32);
    hook!(GM_8016247C, gm_8016247C, (a: i32) -> i32);
    hook!(GM_UNLOCKCKIND, gm_UnlockCKind, (ckind: CharacterKind));
    hook!(GMMAINLIB_8015DB2C, gmMainLib_8015DB2C, (a: u8));
    hook!(GM_CKINDTOUNLOCKINDEX, gm_CKindToUnlockIndex, (ckind: CharacterKind) -> i8);
    hook!(GM_80173EEC, gm_80173EEC, ());
    hook!(GM_80172898, gm_80172898, (a: u16) -> i32);
    hook!(GM_801721EC, gm_801721EC, () -> bool);
    hook!(GM_SETPENDINGGAMEMODE, gm_SetPendingGameMode, (pending_mode: u8));
    hook!(GM_SETNEWGAMEMODEPENDING, gm_SetNewGameModePending, ());
    hook!(TOY_803124BC, Toy_803124BC, ());
    hook!(GM_8017219C, gm_8017219C, (a: u32) -> bool);
    hook!(GMMAINLIB_8015D804, gmMainLib_8015D804, (a: i32) -> *mut u32);
    hook!(GMMAINLIB_8015DA1C, gmMainLib_8015DA1C, (a: u32) -> i32);
    hook!(GMMAINLIB_8015DA90, gmMainLib_8015DA90, (a: u32) -> i32);
    hook!(GMMAINLIB_8015D970, gmMainLib_8015D970, (idx: isize) -> *mut u32);
    hook!(GMMAINLIB_8015DA40, gmMainLib_8015DA40, (a: u32));
    hook!(LBTIME_GETTIMEINSECONDS, lbTime_GetTimeInSeconds, () -> u32);
    hook!(TOY_SETUNLOCKSTATE, Toy_SetUnlockState, (id: enum_t, v: bool));
    hook!(TOY_803048C0, Toy_803048C0, (a: i32) -> i32);
    hook!(GM_80172174, gm_80172174, ());
    hook!(GM_80174180, gm_80174180, ());
    hook!(HSD_MEMALLOC, HSD_MemAlloc, (size: isize) -> *mut c_void);
    // lb/inlines.h `lbCardGame_SetupArchive` (static inline): three calls.
    hook!(LBCARDNEW_ALLOCWORKAREA, lbCardNew_AllocWorkArea, ());
    hook!(LBCARDGAME_LOADARCHIVE, lbCardGame_LoadArchive, (a: i32));
    hook!(LBCARDGAME_SAVECHANGES, lbCardGame_SaveChanges, ());
    // Data of other TUs that the scene table points at (their addresses).
    hook!(GMVSMELEE_APPROACHDATA, gmVsMelee_ApproachData, () -> *mut VsApproachData);
    hook!(GMVSMELEE_STARTDATA, gmVsMelee_StartData, () -> *mut StartMeleeData);
    hook!(GMVSMELEE_VSEXITINFO, gmVsMelee_VsExitInfo, () -> *mut MatchExitInfo);
    /// `if_Scene_Prize_EnterData` (if/if_2FD9.c).
    hook!(IF_SCENE_PRIZE_ENTERDATA, if_Scene_Prize_EnterData, () -> *mut un_804A1F48_t);
}

/// `lb/inlines.h` `lbCardGame_SetupArchive`.
fn lbCardGame_SetupArchive() {
    ext::lbCardNew_AllocWorkArea();
    ext::lbCardGame_LoadArchive(0);
    ext::lbCardGame_SaveChanges();
}

/// `GameModeState gm_Mode_ChallengerApproach_States[]`: the scene table (approach, vs, prize, end).
/// Built on demand because the entries hold the addresses of data in other TUs.
pub fn gm_Mode_ChallengerApproach_States() -> [GameModeState; 4] {
    let mut t = [GameModeState::zeroed(); 4];
    t[0].id = state_approach;
    t[0].preload = lbDvdPreload_2 as u8;
    t[0].flags = 0;
    t[0].on_enter = Some(gm_ModeState_Approach_OnEnter);
    t[0].on_exit = None;
    t[0].info.scene_kind = GS_APPROACH as u8;
    t[0].info.enter_data = ext::gmVsMelee_ApproachData() as *mut c_void;
    t[0].info.exit_data = ext::gmVsMelee_ApproachData() as *mut c_void;

    t[1].id = state_vs;
    t[1].preload = lbDvdPreload_2 as u8;
    t[1].flags = 0;
    t[1].on_enter = Some(gm_ModeState_ApproachVs_OnEnter);
    t[1].on_exit = Some(onExitVs);
    t[1].info.scene_kind = GS_VS as u8;
    t[1].info.enter_data = ext::gmVsMelee_StartData() as *mut c_void;
    t[1].info.exit_data = ext::gmVsMelee_VsExitInfo() as *mut c_void;

    t[2].id = state_prize;
    t[2].preload = lbDvdPreload_2 as u8;
    t[2].flags = 0;
    t[2].on_enter = Some(gm_ModeState_Prize_OnEnter);
    t[2].on_exit = Some(onExitPrize);
    t[2].info.scene_kind = GS_PRIZE_INTERFACE as u8;
    t[2].info.enter_data = ext::if_Scene_Prize_EnterData() as *mut c_void;
    t[2].info.exit_data = null_mut();

    t[3].id = GM_GAMEMODESTATE_TERMINATE as u8;
    t
}

/// `gm_ModeState_Approach_OnEnter`: hand the challenger's CPU kind and the human slot to the vs scene.
pub unsafe extern "C" fn gm_ModeState_Approach_OnEnter(arg0: *mut GameModeState) {
    let vs = ext::gm_GetGameModeStateEnterData(arg0) as *mut VsApproachData;
    let challenger = ext::gm_GetChallengerData();
    (*vs).cpu_ckind = (*challenger).cpu_ckind;
    (*vs).human_slot = (*challenger).human_slot;
    lbCardGame_SetupArchive();
}

/// `gm_ModeState_ApproachVs_OnEnter`: the stock match against the challenger (human in slot 0, CPU in 1).
pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnEnter(state: *mut GameModeState) {
    let start = ext::gm_GetGameModeStateEnterData(state) as *mut StartMeleeData;
    let challenger = ext::gm_GetChallengerData();
    ext::gm_SetupRulesDefaults(addr_of_mut!((*start).rules));
    ext::gm_SetupAllPlayerDefaults((*start).players.as_mut_ptr());
    (*start).rules.set_timer_enabled(0);
    (*start).rules.set_match_kind(MatchKind_Stock as u32);
    (*start).rules.set_x0_3(2);
    (*start).rules.stkind = ext::gm_GetChallengerStKind(ext::gm_CKindToSelKind((*challenger).cpu_ckind) as SelectableCharacterKind) as u16;
    ext::gm_SetupHumanPlayer(
        addr_of_mut!((*start).players[0]),
        (*challenger).human_ckind,
        (*challenger).human_color,
        1,
        (*challenger).human_slot,
    );
    ext::gm_SetupCpuPlayer(addr_of_mut!((*start).players[1]), (*challenger).cpu_ckind, 0, 1, 1);
    {
        let human_nametag = (*challenger).human_nametag;
        (*start).players[0].nametag = human_nametag;
        (*start).players[1].nametag = GM_NAMETAG_COUNT as u8;
        (*start).players[1].cpu_kind = CpuKind_4 as u8;
        (*start).players[1].cpu_level = ext::gm_DecideChallengerCpuLevel((*challenger).cpu_ckind, human_nametag);
    }
    ext::gm_LoadRumbleEnabled(start);
}

/// `onExitVs` (static): record the match, unlock or re-lock the challenger, then go back or to the prize scene.
pub unsafe extern "C" fn onExitVs(state: *mut GameModeState) {
    let mei = ext::gm_GetGameModeStateExitData(state) as *mut MatchExitInfo;
    let challenger = ext::gm_GetChallengerData();
    ext::gm_80162968((*mei).match_end.frame_count / GM_FPS as u32);
    ext::gm_8016247C((*mei).match_end.player_standings[0].xE as i32);
    if (*mei).match_end.outcome as u32 != OUTCOME_NO_CONTEST
        && (*mei).match_end.outcome as u32 != OUTCOME_RETRY
        && (*mei).match_end.player_standings[0].stocks != 0
    {
        ext::gm_UnlockCKind((*challenger).cpu_ckind as CharacterKind);
    } else {
        ext::gmMainLib_8015DB2C(ext::gm_CKindToUnlockIndex((*challenger).cpu_ckind as CharacterKind) as u8);
    }
    ext::gm_80173EEC();
    ext::gm_80172898(0x100);
    if !ext::gm_801721EC() {
        ext::gm_SetPendingGameMode((*challenger).curr_mode);
        ext::gm_SetNewGameModePending();
    }
}

/// `gm_801BFC60` (static, `dont_inline`): fill the first prize entry in place, later ones in a fresh
/// allocation linked from `arg4`; returns the slot where the next entry's address goes.
pub unsafe fn gm_801BFC60(arg0: u32, arg1: i32, arg2: u32, arg3: u32, arg4: *mut *mut c_void) -> *mut *mut c_void {
    if arg1 == 0 {
        let e = ext::if_Scene_Prize_EnterData();
        (*e).x0 = arg0 as u16;
        (*e).x4 = arg3 as i32;
        (*e).x2 = arg2 as u16;
        return addr_of_mut!((*e).x8) as *mut *mut c_void;
    }
    let temp_r3 = ext::HSD_MemAlloc(core::mem::size_of::<un_804A1F48_t>() as isize) as *mut un_804A1F48_t;
    if !temp_r3.is_null() {
        (*temp_r3).x0 = arg0 as u16;
        (*temp_r3).x4 = arg3 as i32;
        (*temp_r3).x2 = arg2 as u16;
        *arg4 = temp_r3 as *mut c_void;
        return addr_of_mut!((*temp_r3).x8) as *mut *mut c_void;
    }
    arg4
}

/// `gm_ModeState_Prize_OnEnter`: list the unlocked fighters, then the owned trophies (newly seen ones
/// first stamped with the time), as a linked list starting at `if_Scene_Prize_EnterData`.
pub unsafe extern "C" fn gm_ModeState_Prize_OnEnter(arg0: *mut GameModeState) {
    ext::gm_GetGameModeStateEnterData(arg0);
    let mut var_r31: *mut *mut c_void = null_mut();
    ext::gm_GetChallengerData();
    let mut var_r30: i32 = 0;
    memzero(&mut gm_8049E558[..], 0x42);
    memzero(&mut gm_8049E558[0x44..], TY_TROPHY_COUNT as isize);
    ext::Toy_803124BC();
    {
        for var_r25 in 0..0x42usize {
            if ext::gm_8017219C(var_r25 as u32) {
                let temp_r3 = gm_801BFC60(var_r25 as u32, var_r30, 0, *ext::gmMainLib_8015D804(var_r25 as i32), var_r31);
                gm_8049E558[var_r25] = 1;
                var_r31 = temp_r3;
                var_r30 += 1;
            }
        }
    }
    for var_r25_2 in 0..TY_TROPHY_COUNT as usize {
        if ext::gmMainLib_8015DA1C(var_r25_2 as u32) != 0 {
            if ext::gmMainLib_8015DA90(var_r25_2 as u32) == 0 {
                let kp = ext::gmMainLib_8015D970(var_r25_2 as isize);
                *kp = ext::lbTime_GetTimeInSeconds();
                ext::gmMainLib_8015DA40(var_r25_2 as u32);
                ext::Toy_SetUnlockState(var_r25_2 as i16 as enum_t, true);
            }
            gm_8049E558[0x44 + var_r25_2] = ext::Toy_803048C0(var_r25_2 as i32) as u8;
            var_r31 = gm_801BFC60(0x3E, var_r30, var_r25_2 as u32, *ext::gmMainLib_8015D970(var_r25_2 as isize), var_r31);
            var_r30 += 1;
        }
    }

    ext::gm_80173EEC();
    ext::gm_80172898(0xFFFF);
    for var_r28_3 in 0..0x42usize {
        if gm_8049E558[var_r28_3] == 0 && ext::gm_8017219C(var_r28_3 as u32) {
            var_r31 = gm_801BFC60(var_r28_3 as u32, var_r30, 0, *ext::gmMainLib_8015D804(var_r28_3 as i32), var_r31);
            var_r30 += 1;
        }
    }

    for var_r27_2 in 0..TY_TROPHY_COUNT as usize {
        let slot = gm_8049E558[0x44 + var_r27_2];
        if ((slot as i32) < ext::Toy_803048C0(var_r27_2 as i32) || ext::Toy_803048C0(var_r27_2 as i32) == 0)
            && ext::gmMainLib_8015DA1C(var_r27_2 as u32) != 0
        {
            if ext::gmMainLib_8015DA90(var_r27_2 as u32) == 0 {
                let temp_r29_2 = ext::gmMainLib_8015D970(var_r27_2 as isize);
                *temp_r29_2 = ext::lbTime_GetTimeInSeconds();
                ext::gmMainLib_8015DA40(var_r27_2 as u32);
                ext::Toy_SetUnlockState(var_r27_2 as i16 as i32 as enum_t, true);
            }
            var_r31 = gm_801BFC60(0x3E, var_r30, var_r27_2 as u32, *ext::gmMainLib_8015D970(var_r27_2 as isize), var_r31);
            var_r30 += 1;
        }
    }

    ext::gm_80172174();
    ext::gm_80174180();
    if !var_r31.is_null() {
        *var_r31 = null_mut();
    }
    lbCardGame_SetupArchive();
}

/// `onExitPrize` (static): back to the mode the challenge interrupted.
pub unsafe extern "C" fn onExitPrize(_state: *mut GameModeState) {
    ext::gm_SetPendingGameMode((*ext::gm_GetChallengerData()).curr_mode);
    ext::gm_SetNewGameModePending();
}
