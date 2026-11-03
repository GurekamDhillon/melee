//! Port of src/melee/gm/gmallstar.c (All-Star Mode: the 30-row scene table of fights and rest areas, the
//! opponent table `gm_803DEBE8` shuffled per run, the round table `gm_803DEC4C`, the fight / rest area /
//! game-over / CSS scene handlers, and the `MELEE_SCENE` one-player direct launch).
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
    fn gmMainLib_8015CDE0() -> *mut gmm_x0_528_t = crate::tu_ext::gmMainLib_8015CDE0;
    fn gmRegSetupEnemyColorTable(a0: i8, a1: u8, a2: *const i8, a3: *mut u8) = crate::tu::adapt::gmRegSetupEnemyColorTable__a8a833;
    fn gm_801677F0() -> u8 = crate::tu::adapt::gm_801677F0__be41b7;
    fn gm_80168F88() = crate::tu::adapt::gm_80168F88__9d899a;
    fn gm_8016A21C(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_8016A21C__1e76b5;
    fn gm_8016A22C(a0: i8, a1: i8, a2: i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: i32, a8: u8, a9: u8, a10: u8, a11: i32, a12: i32, a13: i32, a14: i32, a15: i32, a16: i32, a17: f32, a18: f32) = crate::tu::adapt::gm_8016A22C__e5b47e;
    fn gm_8016A92C(a0: *mut StartMeleeRules) = crate::tu::adapt::gm_8016A92C__fe3fb1;
    fn gm_8016A998(a0: i8, a1: i8) -> i32 = crate::tu::gm_16A2::gm_8016A998;
    fn gm_80172174() = crate::tu::adapt::gm_80172174__0e08c4;
    fn gm_8017BE84(a0: u32) -> i32 = crate::tu::adapt::gm_8017BE84__49c20d;
    fn gm_8017C984(a0: *mut c_void) = crate::tu::gm_17c0::gm_8017C984;
    fn gm_8017C9A8(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: u8) = crate::tu::adapt::gm_8017C9A8__94d55d;
    fn gm_8017CA38(a0: *mut DebugGameOverData, a1: *mut Unk1PData, a2: *mut gmm_x0_528_t, a3: u8) = crate::tu::adapt::gm_8017CA38__442d2c;
    fn gm_8017CBAC(a0: *mut UnkAdventureData, a1: *mut gmm_x0_528_t, a2: u8) = crate::tu::adapt::gm_8017CBAC__33592d;
    fn gm_8017CE34(a0: *mut StartMeleeData, a1: *mut Unk1PData, a2: *mut i8, a3: u8, a4: u8, a5: u8, a6: i32, a7: u32, a8: i32, a9: i32) = crate::tu::adapt::gm_8017CE34__75f422;
    fn gm_8017D7AC(a0: *mut MatchExitInfo, a1: *mut Unk1PData, a2: u8) -> i32 = crate::tu::adapt::gm_8017D7AC__84cd0d;
    fn gm_8017DB58(a0: *mut Unk1PData_x24) = crate::tu::gm_17c0::gm_8017DB58;
    fn gm_8018A160(a0: u8, a1: u8) -> u8 = crate::tu::gm_18A1::gm_8018A160;
    fn gm_8018A188(a0: u8, a1: u8) -> f32 = crate::tu::gm_18A1::gm_8018A188;
    fn gm_8018A1D8(a0: u8, a1: u8) -> f32 = crate::tu::gm_18A1::gm_8018A1D8;
    fn gm_8018A228(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_18A1::gm_8018A228;
    fn gm_8018A25C(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_18A1::gm_8018A25C;
    fn gm_8018A290(a0: u8, a1: u8, a2: u8) -> u8 = crate::tu::gm_18A1::gm_8018A290;
    fn gm_8018A2C4(a0: u8, a1: u8) -> f32 = crate::tu::gm_18A1::gm_8018A2C4;
    fn gm_8018A314(a0: u8, a1: u8) -> f32 = crate::tu::gm_18A1::gm_8018A314;
    fn gm_801B06B0(a0: *mut CSSData, a1: u8, a2: i8, a3: i8, a4: i8, a5: u8, a6: u8, a7: u8) = crate::tu::adapt::gm_801B06B0__3a08bf;
    fn gm_801B0730(a0: *mut CSSData, a1: *mut i8, a2: *mut u8, a3: *mut u8, a4: *mut u8, a5: *mut u8) = crate::tu::gmclassic::ext::gm_801B0730;
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
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
    fn lbBgFlash_8002063C(a0: i32) = crate::tu::adapt::lbBgFlash_8002063C__95303c;
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
        pub unsafe extern "C" fn gm_8018A160(a0: u8, a1: u8) -> u8 { super::gm_8018A160(a0, a1) }
        pub unsafe extern "C" fn gm_8018A228(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8018A228(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8018A25C(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8018A25C(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8018A290(a0: u8, a1: u8, a2: u8) -> u8 { super::gm_8018A290(a0, a1, a2) }
        pub unsafe extern "C" fn gm_8018A2C4(a0: u8, a1: u8) -> f32 { super::gm_8018A2C4(a0, a1) }
        pub unsafe extern "C" fn gm_8018A314(a0: u8, a1: u8) -> f32 { super::gm_8018A314(a0, a1) }
    }
    // Data defined in other TUs.
    pub use crate::tu::gm_17c0::{gmClassic_80470708, gmClassic_80470850, gmClassic_8047086C, gmClassic_80472AF8};
    pub use crate::tu::gm_18A1::gm_80473A18;
}
// END GENERATED EXT

/// `AllStarOpponent` (file-local): the two candidate stages, the stage picked for this run, the character.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AllStarOpponent {
    pub stages: [u8; 2],
    pub stage: u8,
    pub character: u8,
}

/// `AllstarRoundInfo` (file-local): a round's first opponent and opponent count.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AllstarRoundInfo {
    pub start: i32,
    pub count: i32,
}

const fn fight(id: u8) -> GameModeState {
    state(id, 3, Some(gm_801B5624), Some(gm_801B59AC), GS_VS,
          unsafe { &raw mut ext::gmClassic_80472AF8 as *mut c_void }, unsafe { &raw mut ext::gmClassic_8047086C as *mut c_void })
}

const fn rest(id: u8) -> GameModeState {
    state(id, 3, Some(gm_801B5ACC), Some(gm_801B5E7C), GS_VS,
          unsafe { &raw mut ext::gmClassic_80472AF8 as *mut c_void }, unsafe { &raw mut ext::gmClassic_8047086C as *mut c_void })
}

/// `gm_Mode_AllStar_States` (29 rows: twelve fight / rest area pairs, the last fight (Game & Watch), coming soon,
/// game over, CSS, end).
pub static mut gm_Mode_AllStar_States: [GameModeState; 29] = [
    fight(0x00), rest(0x01), fight(0x08), rest(0x09), fight(0x10), rest(0x11), fight(0x18), rest(0x19),
    fight(0x20), rest(0x21), fight(0x28), rest(0x29), fight(0x30), rest(0x31), fight(0x38), rest(0x39),
    fight(0x40), rest(0x41), fight(0x48), rest(0x49), fight(0x50), rest(0x51), fight(0x58), rest(0x59),
    fight(0x60),
    state(0x68, 3, None, Some(gm_801B607C), GS_COMING_SOON, NULL, NULL),
    state(0x69, 3, Some(gm_801B5EB4), Some(gm_801B5EE4), GS_GAMEOVER,
          unsafe { &raw mut ext::gmClassic_80470850 as *mut c_void }, unsafe { &raw mut ext::gmClassic_80470850 as *mut c_void }),
    state(0x70, 3, Some(gm_801B5F50), Some(gm_801B5FB4), GS_CSS,
          unsafe { &raw mut ext::gmClassic_80470708 as *mut c_void }, unsafe { &raw mut ext::gmClassic_80470708 as *mut c_void }),
    end(),
];

const fn opp(s: u8, c: u8) -> AllStarOpponent {
    AllStarOpponent { stages: [s, s], stage: 0, character: c }
}

/// `gm_803DEBE8`: the 25 opponents (`CKind_Playable_Count - 1`), shuffled by `gm_Mode_AllStar_OnLoad`.
pub static mut gm_803DEBE8: [AllStarOpponent; 25] = [
    opp(0xB1, 8), opp(0xB2, 1), opp(0xB3, 6), opp(0xB4, 0x10), opp(0xB5, 0x11), opp(0xB6, 4), opp(0xB7, 2),
    opp(0xB8, 0xD), opp(0xB9, 7), opp(0xBA, 0), opp(0xBB, 0xB), opp(0xBC, 0xF), opp(0xBD, 0xE), opp(0xBE, 0xC),
    opp(0xBF, 0x12), opp(0xC0, 9), opp(0xC1, 0xA), opp(0xC2, 5), opp(0xC3, 0x16), opp(0xC4, 0x15), opp(0xC5, 0x14),
    opp(0xC6, 0x18), opp(0xC7, 0x17), opp(0xC9, 0x19), opp(0xC8, 3),
];

/// `gm_803DEC4C`: per round, its first opponent and how many fight at once.
pub static mut gm_803DEC4C: [AllstarRoundInfo; 13] = {
    const fn r(start: i32, count: i32) -> AllstarRoundInfo { AllstarRoundInfo { start, count } }
    [r(0, 1), r(1, 1), r(2, 1), r(3, 1), r(4, 2), r(6, 2), r(8, 2), r(10, 2), r(12, 3), r(15, 3), r(18, 3), r(21, 3), r(24, 1)]
};

/// `gm_80490940`: per opponent, 0 (not fought), 1 (fought) or 2 (fought, the exit flag set).
pub static mut gm_80490940: [u8; 25] = [0; 25];

unsafe fn allstar() -> *mut UnkAllstarData {
    &raw mut ext::gm_80473A18
}

/// The opponents of `round` (`gm_801B5324_inline` / `gm_801B5624_inline`): the round's characters, the rest none.
unsafe fn round_chars(char_ids: &mut [i8; 3], opp_data: *const AllStarOpponent, round: usize) {
    for c in char_ids.iter_mut() {
        *c = ChKind_None as i8;
    }
    for i in 0..gm_803DEC4C[round].count as usize {
        char_ids[i] = (*opp_data.add(i)).character as i8;
    }
}

/// The index in `gm_803DEBE8` of `round`'s first opponent.
unsafe fn round_start(round: usize) -> usize {
    gm_803DEC4C[round].start as usize
}

/// 1B5324: the next round's preload (the player, the opponents, the stage) and its sound banks.
pub unsafe fn gm_801B5324(arg0: *mut UnkAllstarData, arg1: i32) {
    melee_core::ct!("gm_801B5324");
    let mut chars: [i8; 3] = [0; 3];
    let mut colors: [u8; 3] = [0; 3];
    let mut is_last_round = 0;
    let opp_data = (&raw mut gm_803DEBE8 as *mut AllStarOpponent).add(round_start(arg1 as usize));
    round_chars(&mut chars, opp_data, arg1 as usize);
    for i in 0..3 {
        colors[i] = (*arg0).x0.x54.unwrap()(arg1 as u8, (*arg0).x0.x0.cpu_level, i as u8);
    }
    ext::gmRegSetupEnemyColorTable((*arg0).x0.x0.ckind, (*arg0).x0.x0.color, chars.as_ptr(), colors.as_mut_ptr());
    if arg1 == 0xC {
        chars = [CKind_GameWatch as i8; 3];
        colors = [0; 3];
        is_last_round = 1;
    }
    let gc = &raw mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    ext::lbDvd_80018C6C();
    let mut slot_idx = 0usize;
    (*gc).entries[slot_idx].char_id = (*arg0).x0.x0.ckind as i32;
    (*gc).entries[slot_idx].color = (*arg0).x0.x0.color;
    slot_idx += 1;
    ext::lbDvd_80018254();
    ext::lbDvd_80018C2C(0xC7);
    ext::lbDvd_80017700(4);
    for i in 0..3 {
        if chars[i] as i32 != ChKind_None as i32 {
            (*gc).entries[slot_idx].char_id = chars[i] as i32;
            (*gc).entries[slot_idx].color = if is_last_round != 0 { 0xFF } else { colors[i] };
            slot_idx += 1;
        }
    }
    (*gc).stkind = (*opp_data).stage as _;
    ext::lbDvd_80018254();
    let mut audio = ext::lbAudioAx_80026E84((*arg0).x0.x0.ckind as i32 as u32);
    for j in 0..3 {
        audio |= ext::lbAudioAx_80026E84(chars[j] as i32 as u32);
    }
    audio |= ext::lbAudioAx_80026EBC((*opp_data).stage as u32);
    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0xC, audio);
    ext::lbAudioAx_80027168();
}

/// 1B5624: a fight (the round's opponents on their stage; the run's timer counts up; the last fight is the
/// Game & Watch team).
pub unsafe extern "C" fn gm_801B5624(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5624");
    let mut chars: [i8; 3] = [0; 3];
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let allstar = allstar();
    let mut round = ext::gm_8017BE84((*arg0).id as u32) as u16;
    let opp_data = (&raw mut gm_803DEBE8 as *mut AllStarOpponent).add(round_start(round as usize));
    round_chars(&mut chars, opp_data, round as usize);
    (*allstar).x0.xB = 4;
    (*allstar).x0.x8 = 0;
    round = ext::gm_8017BE84((*arg0).id as u32) as u16;
    let color = gm_80490940[round_start(round as usize)];
    round = ext::gm_8017BE84((*arg0).id as u32) as u16;
    ext::gm_8017CE34(data, &raw mut (*allstar).x0, chars.as_mut_ptr(), 0, 0, 0, 0, (*opp_data).stage as u32, round as i32, color as i32);
    let r = &mut (*data).rules;
    r.set_timer_enabled(1);
    r.set_timer_counts_up(1);
    r.set_x1_0(1);
    r.time_limit = ((*allstar).x9C as i32 / GM_FPS) as u32;
    r.x14 = (((*allstar).x9C as i32 % GM_FPS) + 1) as u8;
    r.x20 &= 0xFFFF_FFFF_FFFB_FCFF;
    (*data).players[0].set_xC_b1(if (*arg0).id == 0 { 1 } else { 0 });
    if (*arg0).id == 0x60 {
        let cpu_level = (*allstar).x0.x0.cpu_level;
        let f30 = ext::gm_8018A1D8(0xC, cpu_level);
        let f31 = ext::gm_8018A188(0xC, cpu_level);
        let opp_count = ext::gm_8018A228(0xC, cpu_level, 0);
        ext::gm_8016A22C(
            CKind_GameWatch as i8, ChKind_None as i8, ChKind_None as i8, 0, 0, 0, 1, 0, 0,
            (*data).players[0].ckind as u8, (*data).players[0].color,
            opp_count as i32, 0x19, 5, 1, 0, 1, f31, f30,
        );
        (*data).rules.set_x4_5(1);
        (*data).rules.set_x0_3(6);
        ext::gm_8016A21C(&raw mut (*data).rules);
    }
    (*data).players[0].damage = (*allstar).x74;
    ext::gm_LoadRumbleEnabled(data);
    (*allstar).x0.x0.mode = (*arg0).id;
}

/// 1B59AC: a fight's exit (the damage carried over, the run's time; the mode's end after the last fight).
pub unsafe extern "C" fn gm_801B59AC(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B59AC");
    let exit = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    let idx = (*arg0).id;
    let result = (*exit).x8;
    let data = allstar();
    let round = ext::gm_8017BE84(idx as u32) as u16;
    let i = round_start(round as usize);
    gm_80490940[i] = if result != 0 { 2 } else { 1 };
    (*data).x74 = (*exit).match_end.player_standings[0].percent as u16;
    (*data).x9C = (*data).x9C.wrapping_add((*exit).match_end.frame_count);
    if ext::gm_8017D7AC(exit, &raw mut (*data).x0, 0x69) != 0 && (*arg0).id == 0x60 {
        ext::gm_8017CBAC(data as *mut UnkAdventureData, ext::gmMainLib_8015CDE0(), 0x17);
    }
}

/// The rest area's match-end callback: the background flash.
pub unsafe extern "C" fn fn_801B5AA8(_arg0: u8) {
    melee_core::ct!("fn_801B5AA8");
    ext::lbBgFlash_8002063C(0x78);
}

/// 1B5ACC: the rest area (stage 85; the heart containers by the remaining opponents, the next round preloaded).
pub unsafe extern "C" fn gm_801B5ACC(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5ACC");
    let mut chars: [i8; 3] = [ChKind_None as i8; 3];
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    let a = allstar();
    (*a).x0.x8 |= 1 << 7;
    let round = ext::gm_8017BE84((*arg0).id as u32) as u16;
    let color = gm_80490940[round_start(round as usize)];
    ext::gm_8017CE34(data, &raw mut (*a).x0, chars.as_mut_ptr(), 0, 0, 0, 0, 85, 0, color as i32);
    let r = &mut (*data).rules;
    r.set_timer_enabled(0);
    r.set_timer_counts_up(1);
    r.set_x1_0(1);
    r.time_limit = ((*a).x9C as i32 / GM_FPS) as u32;
    r.x14 = ((*a).x9C as i32 % GM_FPS) as u8;
    r.xD = GM_NAMETAG_COUNT as u8;
    (*data).players[0].damage = (*a).x74;
    (*data).players[0].set_xD_b2(1);
    (*data).rules.x7 = 9;
    let rest_round = ext::gm_8017BE84((*arg0).id as u32) as u16 as usize;
    let ri = (&raw mut gm_803DEC4C as *mut AllstarRoundInfo).add(rest_round);
    // the defeated opponents' trophies stand in random free slots of x76
    for i in 0..(*ri).count as usize {
        let mut slot;
        loop {
            slot = ext::HSD_Randi(CKind_Playable_Count as i32) as usize;
            if (*a).x76[slot] as u32 != ChKind_None as u32 {
                continue;
            }
            break;
        }
        (*a).x76[slot] = gm_803DEBE8[i + (*ri).start as usize].character;
    }
    let next = &*ri.add(1);
    let mut i = 0;
    while i < next.count as usize {
        (*a).x96[i] = gm_803DEBE8[next.start as usize + i].character;
        i += 1;
    }
    (*a)._94[1] = i as u8;
    (*a)._94[0] = (rest_round + 1) as u8;
    (*data).players[0].set_xC_b1(0);
    (*data).rules.set_x1_2(1);
    (*data).rules.set_x1_3(1);
    (*data).rules.set_x4_4(0);
    ext::gm_LoadRumbleEnabled(data);
    ext::gm_8016A92C(&raw mut (*data).rules);
    // gm_801B5ACC_inline1: the fighters of the rounds still to come
    let mut end_idx = next.start;
    while end_idx < 25 {
        ext::gm_8016A998(gm_803DEBE8[end_idx as usize].character as i8, 0);
        end_idx += 1;
    }
    gm_801B5324(a, rest_round as i32 + 1);
    (*data).rules.on_match_end = Some(fn_801B5AA8);
}

/// 1B5E7C: the rest area's exit.
pub unsafe extern "C" fn gm_801B5E7C(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5E7C");
    let exit = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    let a = allstar();
    (*a).x74 = (*exit).match_end.player_standings[0].percent as u16;
    ext::gm_8017D7AC(exit, &raw mut (*a).x0, 0x69);
}

/// 1B5EB4
pub unsafe extern "C" fn gm_801B5EB4(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5EB4");
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut DebugGameOverData;
    ext::gm_8017C9A8(data, &raw mut (*allstar()).x0, 2);
}

/// 1B5EE4: the game over's exit (a continue starts the damage over).
pub unsafe extern "C" fn gm_801B5EE4(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5EE4");
    let data = ext::gm_GetGameModeStateExitData(arg0) as *mut DebugGameOverData;
    let a = allstar();
    ext::gm_8017CA38(data, &raw mut (*a).x0, ext::gmMainLib_8015CDE0(), 2);
    if (*data).xC != 0 {
        (*a).x74 = 0;
    }
}

/// 1B5F50: the CSS's enter data.
pub unsafe extern "C" fn gm_801B5F50(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5F50");
    let css = ext::gm_GetGameModeStateEnterData(arg0) as *mut CSSData;
    let settings = ext::gmMainLib_8015CDE0();
    ext::gm_801B06B0(
        css,
        0xD,
        (*settings).c_kind,
        (*settings).stocks as i8,
        (*settings).color as i8,
        (*settings).nametag,
        (*settings).cpu_level,
        (*allstar()).x0.x0.slot,
    );
    ext::lbDvd_SetupVsPreloadCache();
    ext::gmFrontend_AtlasSelect(arg0, 0, core::ptr::null());
}

/// 1B5FB4: the CSS's exit (back to the menu, or into the first round).
pub unsafe extern "C" fn gm_801B5FB4(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B5FB4");
    let css = ext::gm_GetGameModeStateExitData(arg0) as *mut CSSData;
    let settings = ext::gmMainLib_8015CDE0();
    let a = allstar();
    if (*css).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    ext::gm_801B0730(
        css,
        &raw mut (*settings).c_kind,
        &raw mut (*settings).stocks,
        &raw mut (*settings).color,
        &raw mut (*settings).nametag,
        &raw mut (*settings).cpu_level,
    );
    (*a).x0.x0.ckind = (*settings).c_kind;
    (*a).x0.x0.color = (*settings).color;
    (*a).x0.x0.cpu_level = (*settings).cpu_level;
    (*a).x0.x0.stocks = (*settings).stocks;
    (*a).x0.x0.nametag = (*settings).nametag;
    ext::gm_SetNextGameModeStateId(((*settings).x5 as u32 * 8) as u8 & 0xF8);
    ext::gm_80168F88();
    gm_801B5324(a, (*settings).x5 as i32);
}

/// 1B607C: the coming-soon scene's exit, back to the menu.
pub unsafe extern "C" fn gm_801B607C(_unused: *mut GameModeState) {
    melee_core::ct!("gm_801B607C");
    ext::gm_SetPendingGameMode(GM_MENU as u8);
    ext::gm_SetNewGameModePending();
}

/// `gm_Mode_AllStar_OnLoad`: the run's record, the opponents' stages and order.
pub fn gm_Mode_AllStar_OnLoad() {
    melee_core::ct!("gm_Mode_AllStar_OnLoad");
    unsafe {
        let data = allstar();
        ext::gmMainLib_8015CDE0();
        ext::gm_8017C984(data as *mut c_void);
        for i in 0..gm_80490940.len() {
            gm_80490940[i] = 0;
        }
        ext::gm_8017DB58((*data).x0.xC.x24.as_mut_ptr());
        (*data).x0.x0.slot = ext::gm_801677F0();
        (*data).x0.x48 = Some(ext::cb::gm_8018A160);
        (*data).x0.x4C = Some(ext::cb::gm_8018A228);
        (*data).x0.x50 = Some(ext::cb::gm_8018A290);
        (*data).x0.x54 = Some(ext::cb::gm_8018A25C);
        (*data).x0.x58 = None;
        (*data).x0.x64 = Some(ext::cb::gm_8018A2C4);
        (*data).x0.x68 = Some(ext::cb::gm_8018A314);
        for index in 0..gm_803DEBE8.len() {
            let o = &mut gm_803DEBE8[index];
            o.stage = o.stages[ext::HSD_Randi(2) as usize];
        }
        for index in 0..0x17usize {
            let rand_offset = ext::HSD_Randi(0x18 - index as i32) as usize;
            let tmp = gm_803DEBE8[index];
            gm_803DEBE8[index] = gm_803DEBE8[index + rand_offset];
            gm_803DEBE8[index + rand_offset] = tmp;
        }
        (*data).x74 = 0;
        (*data).x9C = 0;
        for i in 0..CKind_Playable_Count as usize {
            (*data).x76[i] = ChKind_None as u8;
        }
        (*data).x90 = [1, 1, 1, 1];
        ext::gm_SetGameModeStateId(0x70);
        ext::gm_80172174();
        ext::Ground_801C5A28();
        if crate::tu::gmclassic::gm_OnePSceneDirect(ext::gmMainLib_8015CDE0(), &raw mut (*data).x0, GM_ALLSTAR as i32) != 0 {
            ext::gm_80168F88();
            ext::gm_SetGameModeStateId(((*ext::gmMainLib_8015CDE0()).x5 as u32 * 8) as u8 & 0xF8);
            gm_801B5324(data, (*ext::gmMainLib_8015CDE0()).x5 as i32);
        }
    }
}
