//! src/melee/gm/gmhomerun.c: Home Run Contest mode (scene callbacks, the VS record it owns).
//! Verification: none yet (state oracle not written; every callback drives unported gm/lb code).
//! The `gm_Mode_Homerun_States` table is not ported in this pass (it names the unported
//! gmVsMelee CSS/SSS data and GameModeState arrays).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ptr::addr_of_mut;
use melee_types as mt;
use mt::*;

use crate::tu::gmmain_lib::gmMainLib_804D3EE0;
use crate::tu::gmmain_lib as gml;

/// C: `VsModeData gmHomeRun_VsModeData;` (also read by gmmain_lib's name-tag adjust).
pub static mut gmHomeRun_VsModeData: VsModeData = unsafe { core::mem::zeroed() };
static mut gm_804D68F8: u8 = 0;
static mut gm_804D68F9: u8 = 0;

/// C `gm_80181998` as a C-ABI match-start callback (`Event` is `Option<unsafe extern "C" fn()>`).
unsafe extern "C" fn gm_80181998_c() {
    ext::gm_80181998()
}

/// C `gm_801B98E8` (CSS enter).
pub unsafe fn gm_801B98E8(scene: *mut GameModeState) {
    let vs = addr_of_mut!(gmHomeRun_VsModeData);
    let css = ext::gm_GetGameModeStateEnterData(scene) as *mut CSSData;
    if gm_804D68F9 != 0 {
        ext::lbCardNew_AllocWorkArea();
        ext::lbCardGame_LoadArchive(0);
        ext::lbCardGame_SaveChanges();
    }
    let p0 = &(*vs).start.players[0];
    ext::gm_801B06B0(css, 0x10, p0.ckind as i32, 1, p0.color as i32, p0.nametag as i32, 0, gm_804D68F8 as i32);
    let game_cache = addr_of_mut!((*ext::lbDvd_GetPreloadCacheScene()).game_cache);
    ext::lbDvd_SetupVsPreloadCache();
    (*game_cache).entries[1].char_id = ChKind_Sandbag as _;
    (*game_cache).entries[1].color = 0;
    (*game_cache).stkind = 0x54;
    ext::lbDvd_80018254();
    gm_804D68F9 = crate::lb::lbTime_8000AF74(gm_804D68F9 as u32, 1) as u8;
    // TARGET_PC: the Atlas select, when it can run; else the retail screen.
    ext::gmFrontend_AtlasSelect(scene, 0, core::ptr::null());
}

/// C `gm_801B999C` (CSS exit).
pub unsafe fn gm_801B999C(scene: *mut GameModeState) {
    let vs = addr_of_mut!(gmHomeRun_VsModeData);
    let temp = ext::gm_GetGameModeStateExitData(scene) as *mut CSSData;
    if (*temp).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as _);
        return;
    }
    ext::gm_SetupAllPlayerDefaults((*vs).start.players.as_mut_ptr());
    gm_801B0730_call(temp, vs);
    let p1 = &mut (*vs).start.players[1];
    p1.ckind = ChKind_Sandbag as _;
    p1.cpu_kind = 0xF;
    p1.defense_ratio = 1.0;
    p1.slot_type = Gm_PKind_Cpu as _;
    p1.stocks = 1;
    p1.team = 1;
}

/// The `gm_801B0730(temp_r3, &ckind, NULL, &color, &nametag, NULL)` call from the exit callback.
unsafe fn gm_801B0730_call(temp: *mut CSSData, vs: *mut VsModeData) {
    let p0 = addr_of_mut!((*vs).start.players[0]);
    crate::gm_ptr::gm_801B0730(
        &*temp,
        Some(&mut (*p0).ckind),
        None,
        Some(&mut (*p0).color),
        Some(&mut (*p0).nametag),
        None,
    );
}

/// C `gm_801B9A3C` (VS enter: builds the start data for the Home Run match).
pub unsafe fn gm_801B9A3C(arg0: *mut GameModeState) {
    let vs = addr_of_mut!(gmHomeRun_VsModeData);
    let start = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    ext::gm_SetupRulesDefaults(addr_of_mut!((*start).rules) as *mut StartMeleeRules);

    (*start).rules = (*vs).start.rules;

    let r = addr_of_mut!((*start).rules);
    (*r).stkind = St_Kind_Unk84 as _;
    (*r).set_match_kind(1);
    (*r).set_timer_enabled(1);
    (*r).set_x1_0(1);
    (*r).set_x4_2(0);
    (*r).set_x4_4(0);
    (*r).is_teams = 0;
    (*r).item_freq = -1;
    (*r).time_limit = 10;
    (*r).game_speed = 1.0;
    (*r).x30 = 1.0;
    (*r).set_x5_0(1);
    (*r).set_x1_3(1);
    (*r).on_match_start = Some(gm_80181998_c);
    (*r).set_x3_3(1);
    (*r).set_x3_2(1);
    (*r).x7 = 9;

    for i in 0..GM_MAX_PLAYERS as usize {
        (*start).players[i] = (*vs).start.players[i];
        (*start).players[i].set_xC_b1(0);
        (*start).players[i].set_xD_b3(1);
    }

    crate::gm_r4::gm_SetupHumanPlayer(
        &mut (*start).players[0],
        (*vs).start.players[0].ckind as u8,
        (*vs).start.players[0].color,
        1,
        gm_804D68F8,
    );
    (*start).players[0].set_xD_b2(1);
    ext::gm_LoadRumbleEnabled(start);
    ext::gm_80181A00((*start).players[0].ckind as i32, (*start).players[0].nametag as i32);
    ext::gm_80180B18();
}

/// C `gm_801B9DD8` (VS exit: records the Home Run result and offers a retry or the menu).
pub unsafe fn gm_801B9DD8(arg0: *mut GameModeState) {
    let temp = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    ext::gm_80162968(((*temp).match_end.frame_count / 60) as u32);
    ext::gm_8016247C((*temp).match_end.player_standings[0].xE as i32);
    ext::gm_80180BA0();
    if (*temp).match_end.outcome == OUTCOME_RETRY as _ {
        ext::gm_SetNextGameModeStateId(1);
        return;
    }
    let selkind = crate::gm::gm_CKindToSelKind((*temp).match_end.player_standings[0].ckind as u8);
    let temp_r31 = ext::gm_80180AE4() as u32;
    let tmp = ext::gm_801734D0(temp_r31) as u16;
    if tmp != 0x148 {
        ext::gm_80164504(tmp as u32);
    }
    let temp_r3_2 = gml::gmMainLib_8015D084(selkind);
    if temp_r31 as i32 > *temp_r3_2 {
        *temp_r3_2 = temp_r31 as i32;
    }
    ext::gm_80173B30(temp_r31);
    ext::gm_80173EEC();
    ext::gm_80172898(8);
    if ext::gm_80173754(0x20, gm_804D68F8 as i32) == 0 {
        ext::gm_SetNextGameModeStateId(0);
    }
}

/// C `gm_Mode_Homerun_OnInit`.
pub unsafe fn gm_Mode_Homerun_OnInit() {
    ext::gm_InitVsMode(addr_of_mut!(gmHomeRun_VsModeData));
}

/// C `gm_Mode_Homerun_OnLoad`.
pub unsafe fn gm_Mode_Homerun_OnLoad() {
    gm_804D68F8 = ext::gm_801677F0();
    gm_804D68F9 = 0;
}

/// Callees in TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    pub static mut GM_GETGAMEMODESTATEENTERDATA: Option<fn(*mut GameModeState) -> *mut core::ffi::c_void> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetGameModeStateEnterData(s: *mut GameModeState) -> *mut core::ffi::c_void {
                match unsafe { GM_GETGAMEMODESTATEENTERDATA } {
                    Some(f) => unsafe { f(s) },
                    None => { return unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(s as _) as _ }; },
                }
            }
    pub static mut GM_GETGAMEMODESTATEEXITDATA: Option<fn(*mut GameModeState) -> *mut core::ffi::c_void> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetGameModeStateExitData(s: *mut GameModeState) -> *mut core::ffi::c_void {
                match unsafe { GM_GETGAMEMODESTATEEXITDATA } {
                    Some(f) => unsafe { f(s) },
                    None => { return unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(s as _) as _ }; },
                }
            }
    pub static mut LBCARDNEW_ALLOCWORKAREA: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_AllocWorkArea()  {
                match unsafe { LBCARDNEW_ALLOCWORKAREA } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } return; },
                }
            }
    pub static mut LBCARDGAME_LOADARCHIVE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_LoadArchive(a: i32)  {
                match unsafe { LBCARDGAME_LOADARCHIVE } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(a as _); } return; },
                }
            }
    pub static mut LBCARDGAME_SAVECHANGES: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_SaveChanges()  {
                match unsafe { LBCARDGAME_SAVECHANGES } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } return; },
                }
            }
    hook!(GM_801B06B0, gm_801B06B0, (css: *mut CSSData, a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32), crate::tu::adapt::gm_801B06B0__7adbb8);

    pub static mut LBDVD_GETPRELOADCACHESCENE: Option<fn() -> *mut PreloadedGameModeState> = None;
    #[allow(non_snake_case)]
    pub fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState {
        match unsafe { LBDVD_GETPRELOADCACHESCENE } {
            Some(f) => f(),
            None => { unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() as _ } },
        }
    }
    pub static mut LBDVD_SETUPVSPRELOADCACHE: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_SetupVsPreloadCache()  {
                match unsafe { LBDVD_SETUPVSPRELOADCACHE } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache(); } return; },
                }
            }
    pub static mut LBDVD_80018254: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_80018254()  {
                match unsafe { LBDVD_80018254 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = melee_runtime::tu::lbdvd::lbDvd_80018254(); } return; },
                }
            }
    pub static mut GMFRONTEND_ATLASSELECT: Option<fn(*mut GameModeState, i32, *const i8) > = None;
            #[allow(non_snake_case)]
            pub fn gmFrontend_AtlasSelect(s: *mut GameModeState, sss: i32, name: *const i8)  {
                match unsafe { GMFRONTEND_ATLASSELECT } {
                    Some(f) => unsafe { f(s, sss, name) },
                    None => { unsafe { let _ = crate::tu::gmfrontend::gmFrontend_AtlasSelect(s as _, sss as _, name as _); } return; },
                }
            }
    pub static mut GM_CHANGEGAMEMODEAFTERCURRENTSCENE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_ChangeGameModeAfterCurrentScene(m: i32)  {
                match unsafe { GM_CHANGEGAMEMODEAFTERCURRENTSCENE } {
                    Some(f) => unsafe { f(m) },
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(m as _); } return; },
                }
            }
    pub static mut GM_SETUPALLPLAYERDEFAULTS: Option<fn(*mut PlayerInitData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetupAllPlayerDefaults(p: *mut PlayerInitData)  {
                match unsafe { GM_SETUPALLPLAYERDEFAULTS } {
                    Some(f) => unsafe { f(p) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_SetupAllPlayerDefaults(p as _); } return; },
                }
            }
    pub static mut GM_SETUPRULESDEFAULTS: Option<fn(*mut StartMeleeRules) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetupRulesDefaults(r: *mut StartMeleeRules)  {
                match unsafe { GM_SETUPRULESDEFAULTS } {
                    Some(f) => unsafe { f(r) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_SetupRulesDefaults(r as _); } return; },
                }
            }
    pub static mut GM_80181998: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80181998()  {
                match unsafe { GM_80181998 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_180A::gm_80181998(); } return; },
                }
            }
    pub static mut GM_LOADRUMBLEENABLED: Option<fn(*mut StartMeleeData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_LoadRumbleEnabled(s: *mut StartMeleeData)  {
                match unsafe { GM_LOADRUMBLEENABLED } {
                    Some(f) => unsafe { f(s) },
                    None => { unsafe { let _ = crate::tu::gmvs::gm_LoadRumbleEnabled(s as _); } return; },
                }
            }
    pub static mut GM_80181A00: Option<fn(i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80181A00(ckind: i32, nametag: i32)  {
                match unsafe { GM_80181A00 } {
                    Some(f) => unsafe { f(ckind, nametag) },
                    None => { unsafe { let _ = crate::tu::gm_180A::gm_80181A00(ckind as _, nametag as _); } return; },
                }
            }
    pub static mut GM_80180B18: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80180B18()  {
                match unsafe { GM_80180B18 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_180A::gm_80180B18(); } return; },
                }
            }
    pub static mut GM_801677F0: Option<fn() -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_801677F0() -> u8 {
                match unsafe { GM_801677F0 } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_1601::gm_801677F0() as _ }; },
                }
            }
    pub static mut GM_80164504: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80164504(a: u32)  {
                match unsafe { GM_80164504 } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_80164504(a as _); } return; },
                }
            }
    pub static mut GM_80162968: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80162968(frames: u32)  {
                match unsafe { GM_80162968 } {
                    Some(f) => unsafe { f(frames) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_80162968(frames as _); } return; },
                }
            }
    pub static mut GM_8016247C: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_8016247C(a: i32)  {
                match unsafe { GM_8016247C } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_8016247C(a as _); } return; },
                }
            }
    pub static mut GM_80180BA0: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80180BA0()  {
                match unsafe { GM_80180BA0 } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_180A::gm_80180BA0(); } return; },
                }
            }
    pub static mut GM_80180AE4: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_80180AE4() -> i32 {
                match unsafe { GM_80180AE4 } {
                    Some(f) => unsafe { f() },
                    None => { return unsafe { crate::tu::gm_180A::gm_80180AE4() as _ }; },
                }
            }
    pub static mut GM_801734D0: Option<fn(u32) -> i16> = None;
            #[allow(non_snake_case)]
            pub fn gm_801734D0(a: u32) -> i16 {
                match unsafe { GM_801734D0 } {
                    Some(f) => unsafe { f(a) },
                    None => { return unsafe { crate::tu::gm_16f1::gm_801734D0(a as _) as _ }; },
                }
            }
    pub static mut GM_80173B30: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80173B30(a: u32)  {
                match unsafe { GM_80173B30 } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_1736::gm_80173B30(a as _); } return; },
                }
            }
    pub static mut GM_80173EEC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_80173EEC()  {
                match unsafe { GM_80173EEC } {
                    Some(f) => unsafe { f() },
                    None => { unsafe { let _ = crate::tu::gm_1736::gm_80173EEC(); } return; },
                }
            }
    pub static mut GM_80172898: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_80172898(a: i32)  {
                match unsafe { GM_80172898 } {
                    Some(f) => unsafe { f(a) },
                    None => { unsafe { let _ = crate::tu::gm_16f1::gm_80172898(a as _); } return; },
                }
            }
    pub static mut GM_80173754: Option<fn(i32, i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_80173754(a: i32, b: i32) -> i32 {
                match unsafe { GM_80173754 } {
                    Some(f) => unsafe { f(a, b) },
                    None => { return unsafe { crate::tu::gm_1736::gm_80173754(a as _, b as _) as _ }; },
                }
            }
    pub static mut GM_SETNEXTGAMEMODESTATEID: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_SetNextGameModeStateId(id: i32)  {
                match unsafe { GM_SETNEXTGAMEMODESTATEID } {
                    Some(f) => unsafe { f(id) },
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_SetNextGameModeStateId(id as _); } return; },
                }
            }
    pub static mut GM_INITVSMODE: Option<fn(*mut VsModeData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_InitVsMode(vs: *mut VsModeData)  {
                match unsafe { GM_INITVSMODE } {
                    Some(f) => unsafe { f(vs) },
                    None => { unsafe { let _ = crate::tu::gm_1601::gm_InitVsMode(vs as _); } return; },
                }
            }
}
