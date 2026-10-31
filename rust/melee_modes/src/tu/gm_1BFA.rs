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
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
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
    pub static mut GM_GETCHALLENGERDATA: Option<fn() -> *mut ChallengerData> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetChallengerData() -> *mut ChallengerData {
                match unsafe { GM_GETCHALLENGERDATA } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gm_1736::gm_GetChallengerData() as _ }; },
                }
            }
    pub static mut GM_SETUPRULESDEFAULTS: Option<fn(*mut StartMeleeRules) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetupRulesDefaults(rules: *mut StartMeleeRules)  {
                match unsafe { GM_SETUPRULESDEFAULTS } {
                    Some(f) => f(rules),
                    None => { unsafe { crate::tu::gm_1601::gm_SetupRulesDefaults(rules as _); } return; },
                }
            }
    pub static mut GM_SETUPALLPLAYERDEFAULTS: Option<fn(*mut PlayerInitData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetupAllPlayerDefaults(players: *mut PlayerInitData)  {
                match unsafe { GM_SETUPALLPLAYERDEFAULTS } {
                    Some(f) => f(players),
                    None => { unsafe { crate::tu::gm_1601::gm_SetupAllPlayerDefaults(players as _); } return; },
                }
            }
    pub static mut GM_GETCHALLENGERSTKIND: Option<fn(SelectableCharacterKind) -> StKind> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetChallengerStKind(sel_kind: SelectableCharacterKind) -> StKind {
                match unsafe { GM_GETCHALLENGERSTKIND } {
                    Some(f) => f(sel_kind),
                    None => { return unsafe { crate::gm::gm_GetChallengerStKind(sel_kind as _) as _ }; },
                }
            }
    pub static mut GM_CKINDTOSELKIND: Option<fn(u8) -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_CKindToSelKind(ckind: u8) -> u8 {
                match unsafe { GM_CKINDTOSELKIND } {
                    Some(f) => f(ckind),
                    None => { return unsafe { crate::gm::gm_CKindToSelKind(ckind as _) as _ }; },
                }
            }
    pub static mut GM_SETUPHUMANPLAYER: Option<fn(*mut PlayerInitData, u8, u8, u8, u8)> = None;
    #[allow(non_snake_case)]
    pub fn gm_SetupHumanPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
        match unsafe { GM_SETUPHUMANPLAYER } {
            Some(f) => f(p, ckind, color, stocks, slot),
            None => { unsafe { let _ = crate::gm_r4::gm_SetupHumanPlayer(&mut *(p as *mut _), ckind as _, color as _, stocks as _, slot as _); } },
        }
    }
    pub static mut GM_SETUPCPUPLAYER: Option<fn(*mut PlayerInitData, u8, u8, u8, u8)> = None;
    #[allow(non_snake_case)]
    pub fn gm_SetupCpuPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
        match unsafe { GM_SETUPCPUPLAYER } {
            Some(f) => f(p, ckind, color, stocks, slot),
            None => { unsafe { let _ = crate::gm_r4::gm_SetupCpuPlayer(&mut *(p as *mut _), ckind as _, color as _, stocks as _, slot as _); } },
        }
    }
    pub static mut GM_DECIDECHALLENGERCPULEVEL: Option<fn(u8, u8) -> u8> = None;
    #[allow(non_snake_case)]
    pub fn gm_DecideChallengerCpuLevel(cpu_ckind: u8, human_nametag: u8) -> u8 {
        match unsafe { GM_DECIDECHALLENGERCPULEVEL } {
            Some(f) => f(cpu_ckind, human_nametag),
            None => { unsafe { crate::tu::gm_16f1::gm_DecideChallengerCpuLevel(cpu_ckind as _, human_nametag as _) as _ } },
        }
    }
    pub static mut GM_LOADRUMBLEENABLED: Option<fn(*mut StartMeleeData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_LoadRumbleEnabled(start: *mut StartMeleeData)  {
                match unsafe { GM_LOADRUMBLEENABLED } {
                    Some(f) => f(start),
                    None => { unsafe { crate::tu::gmvs::gm_LoadRumbleEnabled(start as _); } return; },
                }
            }
    pub static mut GM_80162968: Option<fn(u32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_80162968(seconds: u32) -> i32 {
                match unsafe { GM_80162968 } {
                    Some(f) => f(seconds),
                    None => { return unsafe { crate::tu::gm_1601::gm_80162968(seconds as _) as _ }; },
                }
            }
    pub static mut GM_8016247C: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_8016247C(a: i32) -> i32 {
                match unsafe { GM_8016247C } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gm_1601::gm_8016247C(a as _) as _ }; },
                }
            }
    pub static mut GM_UNLOCKCKIND: Option<fn(CharacterKind) > = None;
            #[allow(non_snake_case)]
            pub fn gm_UnlockCKind(ckind: CharacterKind)  {
                match unsafe { GM_UNLOCKCKIND } {
                    Some(f) => f(ckind),
                    None => { unsafe { crate::tu::gm_1601::gm_UnlockCKind(ckind as _); } return; },
                }
            }
    pub static mut GMMAINLIB_8015DB2C: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015DB2C(a: u8)  {
                match unsafe { GMMAINLIB_8015DB2C } {
                    Some(f) => f(a),
                    None => { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DB2C(a as _); } return; },
                }
            }
    pub static mut GM_CKINDTOUNLOCKINDEX: Option<fn(CharacterKind) -> i8> = None;
            #[allow(non_snake_case)]
            pub fn gm_CKindToUnlockIndex(ckind: CharacterKind) -> i8 {
                match unsafe { GM_CKINDTOUNLOCKINDEX } {
                    Some(f) => f(ckind),
                    None => { return unsafe { crate::gm::gm_CKindToUnlockIndex(ckind as _) as _ }; },
                }
            }
    pub static mut GM_80173EEC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80173EEC()  {
                match unsafe { GM_80173EEC } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gm_1736::gm_80173EEC(); } return; },
                }
            }
    pub static mut GM_80172898: Option<fn(u16) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_80172898(a: u16) -> i32 {
                match unsafe { GM_80172898 } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gm_16f1::gm_80172898(a as _) as _ }; },
                }
            }
    pub static mut GM_801721EC: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn gm_801721EC() -> bool {
                match unsafe { GM_801721EC } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gm_16f1::gm_801721EC() != 0 }; },
                }
            }
    pub static mut GM_SETPENDINGGAMEMODE: Option<fn(u8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetPendingGameMode(pending_mode: u8)  {
                match unsafe { GM_SETPENDINGGAMEMODE } {
                    Some(f) => f(pending_mode),
                    None => { unsafe { crate::tu::gm_1A3F::gm_SetPendingGameMode(pending_mode as _); } return; },
                }
            }
    pub static mut GM_SETNEWGAMEMODEPENDING: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetNewGameModePending()  {
                match unsafe { GM_SETNEWGAMEMODEPENDING } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gm_1A3F::gm_SetNewGameModePending(); } return; },
                }
            }
    pub static mut TOY_803124BC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn Toy_803124BC()  {
                match unsafe { TOY_803124BC } {
                    Some(f) => f(),
                    None => { unsafe { melee_items::tu::toy::Toy_803124BC(); } return; },
                }
            }
    pub static mut GM_8017219C: Option<fn(u32) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn gm_8017219C(a: u32) -> bool {
                match unsafe { GM_8017219C } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gm_16f1::gm_8017219C(a as _) != 0 }; },
                }
            }
    pub static mut GMMAINLIB_8015D804: Option<fn(i32) -> *mut u32> = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015D804(a: i32) -> *mut u32 {
                match unsafe { GMMAINLIB_8015D804 } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015D804(a as _) as _ }; },
                }
            }
    pub static mut GMMAINLIB_8015DA1C: Option<fn(u32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015DA1C(a: u32) -> i32 {
                match unsafe { GMMAINLIB_8015DA1C } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA1C(a as _) as _ }; },
                }
            }
    pub static mut GMMAINLIB_8015DA90: Option<fn(u32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015DA90(a: u32) -> i32 {
                match unsafe { GMMAINLIB_8015DA90 } {
                    Some(f) => f(a),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA90(a as _) as _ }; },
                }
            }
    pub static mut GMMAINLIB_8015D970: Option<fn(isize) -> *mut u32> = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015D970(idx: isize) -> *mut u32 {
                match unsafe { GMMAINLIB_8015D970 } {
                    Some(f) => f(idx),
                    None => { return unsafe { crate::tu::gmmain_lib::gmMainLib_8015D970(idx as _) as _ }; },
                }
            }
    pub static mut GMMAINLIB_8015DA40: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015DA40(a: u32)  {
                match unsafe { GMMAINLIB_8015DA40 } {
                    Some(f) => f(a),
                    None => { unsafe { crate::tu::gmmain_lib::gmMainLib_8015DA40(a as _); } return; },
                }
            }
    pub static mut LBTIME_GETTIMEINSECONDS: Option<fn() -> u32> = None;
            #[allow(non_snake_case)]
            pub fn lbTime_GetTimeInSeconds() -> u32 {
                match unsafe { LBTIME_GETTIMEINSECONDS } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbtime::lbTime_GetTimeInSeconds() as _ }; },
                }
            }
    pub static mut TOY_SETUNLOCKSTATE: Option<fn(enum_t, bool) > = None;
            #[allow(non_snake_case)]
            pub fn Toy_SetUnlockState(id: enum_t, v: bool)  {
                match unsafe { TOY_SETUNLOCKSTATE } {
                    Some(f) => f(id, v),
                    None => { unsafe { melee_items::tu::toy::Toy_SetUnlockState(id as _, v as _); } return; },
                }
            }
    pub static mut TOY_803048C0: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Toy_803048C0(a: i32) -> i32 {
                match unsafe { TOY_803048C0 } {
                    Some(f) => f(a),
                    None => { return unsafe { melee_items::tu::toy::Toy_803048C0(a as _) as _ }; },
                }
            }
    pub static mut GM_80172174: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80172174()  {
                match unsafe { GM_80172174 } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gm_16f1::gm_80172174(); } return; },
                }
            }
    pub static mut GM_80174180: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80174180()  {
                match unsafe { GM_80174180 } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gm_1736::gm_80174180(); } return; },
                }
            }
    pub static mut HSD_MEMALLOC: Option<fn(isize) -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn HSD_MemAlloc(size: isize) -> *mut c_void {
                match unsafe { HSD_MEMALLOC } {
                    Some(f) => f(size),
                    None => { return unsafe { melee_runtime::tu::memory::HSD_MemAlloc(size as _) as _ }; },
                }
            }
    // lb/inlines.h `lbCardGame_SetupArchive` (static inline): three calls.
    pub static mut LBCARDNEW_ALLOCWORKAREA: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_AllocWorkArea()  {
                match unsafe { LBCARDNEW_ALLOCWORKAREA } {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } return; },
                }
            }
    pub static mut LBCARDGAME_LOADARCHIVE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_LoadArchive(a: i32)  {
                match unsafe { LBCARDGAME_LOADARCHIVE } {
                    Some(f) => f(a),
                    None => { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(a as _); } return; },
                }
            }
    pub static mut LBCARDGAME_SAVECHANGES: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_SaveChanges()  {
                match unsafe { LBCARDGAME_SAVECHANGES } {
                    Some(f) => f(),
                    None => { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } return; },
                }
            }
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
    melee_core::ct!("gm_ModeState_Approach_OnEnter");
    let vs = ext::gm_GetGameModeStateEnterData(arg0) as *mut VsApproachData;
    let challenger = ext::gm_GetChallengerData();
    (*vs).cpu_ckind = (*challenger).cpu_ckind;
    (*vs).human_slot = (*challenger).human_slot;
    lbCardGame_SetupArchive();
}

/// `gm_ModeState_ApproachVs_OnEnter`: the stock match against the challenger (human in slot 0, CPU in 1).
pub unsafe extern "C" fn gm_ModeState_ApproachVs_OnEnter(state: *mut GameModeState) {
    melee_core::ct!("gm_ModeState_ApproachVs_OnEnter");
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
    melee_core::ct!("onExitVs");
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
    melee_core::ct!("gm_801BFC60");
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
    melee_core::ct!("gm_ModeState_Prize_OnEnter");
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
    melee_core::ct!("onExitPrize");
    ext::gm_SetPendingGameMode((*ext::gm_GetChallengerData()).curr_mode);
    ext::gm_SetNewGameModePending();
}
