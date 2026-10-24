//! Port of src/melee/gm/gmdebugmode.c (the debug game mode: the debug menu scenes, and the enter /
//! exit handlers of the scenes it launches: prize, VS, results, intro, game over, approach,
//! memory card). Checked against the C (whole TU, shipping contraction) by tests/tu_gmdebugmode.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, non_camel_case_types)]

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
    fn HSD_Randi(a0: i32) -> i32;
    fn gm_80166A98(a0: *mut MatchEnd, a1: u8, a2: i8, a3: u8, a4: i8, a5: u8, a6: i8, a7: u8, a8: u8, a9: u8) -> i32 = crate::tu::adapt::gm_80166A98__8c24a8;
    fn gm_80168F88() = crate::tu::adapt::gm_80168F88__9d899a;
    fn gm_801701A0() = crate::tu::adapt::gm_801701A0__12d021;
    fn gm_80177724(a0: *mut ResultsMatchInfo) = crate::tu::adapt::gm_80177724__c19fac;
    fn gm_801A4B60() = crate::tu::adapt::gm_801A4B60__d469d9;
    fn gm_ChangeGameModeAfterCurrentScene(a0: i32) = crate::tu::adapt::gm_ChangeGameModeAfterCurrentScene__2292c1;
    fn gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateEnterData__d31bdf;
    fn gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = crate::tu::adapt::gm_GetGameModeStateExitData__c3c609;
    fn gm_LoadAnnouncer() = crate::tu::adapt::gm_LoadAnnouncer__907f2e;
    fn gm_SetNextGameModeStateId(a0: u8) = crate::tu::adapt::gm_SetNextGameModeStateId__031e98;
    fn lbAudioAx_800236DC() -> i32 = crate::tu::adapt::lbAudioAx_800236DC__5d9f3e;
    fn lbAudioAx_80024030(a0: i32) = crate::tu::adapt::lbAudioAx_80024030__36458a;
    fn lbAudioAx_80026E84(a0: u32) -> u64 = crate::tu::adapt::lbAudioAx_80026E84__5dcc27;
    fn lbAudioAx_80026F2C(a0: u32) = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
    fn lbAudioAx_8002702C(a0: u32, a1: u64) = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
    fn lbAudioAx_80027168() = crate::tu::adapt::lbAudioAx_80027168__342616;
    fn lbAudioAx_80027648() = crate::tu::adapt::lbAudioAx_80027648__4f40c8;
    fn un_802FFEE0(a0: *mut c_void /* UnkSoundTestData0 */) = crate::tu::adapt::un_802FFEE0__17ec63;
    fn un_802FFF2C(a0: *mut StartMeleeData) = crate::tu::adapt::un_802FFF2C__73c528;
    fn un_80301BA8(a0: *mut c_void /* UnkSoundTestData1 */) = crate::tu::adapt::un_80301BA8__89da53;
    fn un_80301C64(a0: *mut un_80301C64_t) = crate::tu::adapt::un_80301C64__b851f6;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
    pub static mut un_803FA258: [i32; 128] = unsafe { core::mem::zeroed() };
    pub static mut un_803FA4E0: [un_80304138_objalloc_t_x8; 1] = unsafe { core::mem::zeroed() };
    pub static mut un_803FA790: [un_80304138_objalloc_t_x8; 4] = unsafe { core::mem::zeroed() };
    pub static mut un_803FC4CC: [un_80304138_objalloc_t_x8; 10] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

const HSD_PAD_L: u32 = 1 << 6;

/// `sfxBack` (mn/inlines.h).
#[inline(always)]
fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}

/// `struct DebugResultsData`: data used by onEnterResults1.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DebugResultsData {
    /// `x0_0:1`, `x0_1:1`, `x0_other:6` (GameCube bit order, MSB first)
    pub bf_0: [u8; 1],
    pub x1: u8,
    pub x2: u8,
    pub x3: u8,
    pub x4: u8,
    pub pad_x5: [u8; 3],
    pub match_end: MatchEnd,
}

impl DebugResultsData {
    pub fn x0_0(&self) -> u8 { bits_get(&self.bf_0, 7, 1) as u8 }
    pub fn set_x0_0(&mut self, v: u8) { bits_set(&mut self.bf_0, 7, 1, (v as u64) & 0x1) }
    pub fn x0_1(&self) -> u8 { bits_get(&self.bf_0, 6, 1) as u8 }
    pub fn set_x0_1(&mut self, v: u8) { bits_set(&mut self.bf_0, 6, 1, (v as u64) & 0x1) }
    pub fn x0_other(&self) -> u8 { bits_get(&self.bf_0, 0, 6) as u8 }
    pub fn set_x0_other(&mut self, v: u8) { bits_set(&mut self.bf_0, 0, 6, (v as u64) & 0x3F) }
}

melee_core::gc_layout!(DebugResultsData, "DebugResultsData", size 0x2284, align 4, [
    x0_0 @ 0x0: 1 "u8" bits 0 + 1,
    x0_1 @ 0x0: 1 "u8" bits 1 + 1,
    x0_other @ 0x0: 1 "u8" bits 2 + 6,
    x1 @ 0x1: 1 "u8",
    x2 @ 0x2: 1 "u8",
    x3 @ 0x3: 1 "u8",
    x4 @ 0x4: 1 "u8",
    pad_x5 @ 0x5: 3 "u8[3]",
    match_end @ 0x8: 0x227C "MatchEnd",
]);

/// `struct DebugMemcardData`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DebugMemcardData {
    pub x0: u32,
    pub x4: u8,
}

melee_core::gc_layout!(DebugMemcardData, "DebugMemcardData", size 0x8, align 4, [
    x0 @ 0x0: 4 "u32",
    x4 @ 0x4: 1 "u8",
]);

/// `struct enterMenu0_Data` (two `UNK_T` slots)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct enterMenu0_Data {
    pub x0: *mut c_void,
    pub x4: *mut c_void,
}

melee_core::gc_layout!(enterMenu0_Data, "enterMenu0_Data", size 0x8, align 4, [
    x0 @ 0x0: 4 "UNK_T",
    x4 @ 0x4: 4 "UNK_T",
]);

/// `struct exitIntroData`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct exitIntroData {
    pub x0: *mut c_void,
}

melee_core::gc_layout!(exitIntroData, "exitIntroData", size 0x4, align 4, [
    x0 @ 0x0: 4 "UNK_T",
]);

/// 480DC8 `struct UnkSoundTestData1` (0x20 bytes; the TU only passes its address on)
pub static mut intro_easy_enter_data: [u8; 0x20] = [0; 0x20];
/// 480DE8
pub static mut vs_enter_data: StartMeleeData = unsafe { core::mem::zeroed() };
/// 480F20
pub static mut vs_exit_data: MatchExitInfo = unsafe { core::mem::zeroed() };
/// 4831A8
pub static mut results0_data: ResultsMatchInfo = unsafe { core::mem::zeroed() };
/// 48542C `struct UnkSoundTestData0` (0xC bytes)
pub static mut prize_enter_data: [u8; 0xC] = [0; 0xC];
/// 485438
pub static mut game_over_data: DebugGameOverData = unsafe { core::mem::zeroed() };
/// 485454
pub static mut results1_enter_data: DebugResultsData = unsafe { core::mem::zeroed() };
/// 4D6880
pub static mut intro_easy_exit_data: exitIntroData = unsafe { core::mem::zeroed() };
/// 4D6884
pub static mut intro_allstar_enter_data: un_80301C64_t = unsafe { core::mem::zeroed() };
/// 4D688C
pub static mut intro_allstar_exit_data: exitIntroData = unsafe { core::mem::zeroed() };
/// 4D6890
pub static mut menu_enter_data: enterMenu0_Data = unsafe { core::mem::zeroed() };
/// 4D6898
pub static mut approach_data: exitIntroData = unsafe { core::mem::zeroed() };
/// 4D689C
pub static mut memcard_enter_data: DebugMemcardData = unsafe { core::mem::zeroed() };
/// 4D68A4
pub static mut memcard_exit_data: DebugMemcardData = unsafe { core::mem::zeroed() };

/// 3DD6D0 `gm_Mode_Debug_States`
pub static mut gm_Mode_Debug_States: [GameModeState; 16] = unsafe {
    [
        state(0, lbDvdPreload_2, Some(onEnterMenu0), None, GS_DEBUG_MENU, &raw mut menu_enter_data as *mut c_void, NULL),
        state(1, lbDvdPreload_2, Some(onEnterMenu1), None, GS_DEBUG_MENU, &raw mut menu_enter_data as *mut c_void, NULL),
        state(2, lbDvdPreload_2, Some(onEnterMenu2), None, GS_DEBUG_MENU, &raw mut menu_enter_data as *mut c_void, NULL),
        state(3, lbDvdPreload_2, Some(onEnterPrize), Some(onExitPrize), GS_PRIZE_INTERFACE,
              &raw mut prize_enter_data as *mut c_void, NULL),
        state(4, lbDvdPreload_2, Some(onEnterVs), None, GS_VS,
              &raw mut vs_enter_data as *mut c_void, &raw mut vs_exit_data as *mut c_void),
        state(5, lbDvdPreload_2, Some(onEnterResults0), Some(onExitResults0), GS_RESULTS,
              &raw mut results0_data as *mut c_void, &raw mut results0_data as *mut c_void),
        state(6, lbDvdPreload_2, Some(onEnterIntroEasy), Some(onExitIntro), GS_INTRO_EASY,
              &raw mut intro_easy_enter_data as *mut c_void, &raw mut intro_easy_exit_data as *mut c_void),
        state(7, lbDvdPreload_2, Some(onEnterIntroAllstar), Some(onExitIntro), GS_INTRO_ALLSTAR,
              &raw mut intro_allstar_enter_data as *mut c_void, &raw mut intro_allstar_exit_data as *mut c_void),
        state(8, lbDvdPreload_2, None, Some(onExitIntro), GS_COMING_SOON, NULL, NULL),
        state(9, lbDvdPreload_2, Some(onEnterGameOver), Some(onExitIntro), GS_GAMEOVER,
              &raw mut game_over_data as *mut c_void, &raw mut game_over_data as *mut c_void),
        state(10, lbDvdPreload_2, Some(onEnterApproach), Some(onExitIntro), GS_APPROACH,
              &raw mut approach_data as *mut c_void, &raw mut approach_data as *mut c_void),
        state(11, lbDvdPreload_2, Some(onEnterResults1), Some(onExitResults1), GS_RESULTS,
              &raw mut results1_enter_data as *mut c_void, NULL),
        state(12, lbDvdPreload_2, None, Some(onExitIntro), GS_MOVIE_OPENING, NULL, NULL),
        state(13, lbDvdPreload_2, Some(onEnterMemCard), Some(onExitMemCard), GS_MEMCARD,
              &raw mut memcard_enter_data as *mut c_void, &raw mut memcard_exit_data as *mut c_void),
        state(14, lbDvdPreload_2, None, None, GS_STAFFROLL, NULL, NULL),
        end(),
    ]
};

/// 1B099C
pub unsafe extern "C" fn onExitIntro(_unused: *mut GameModeState) {
    ext::gm_SetNextGameModeStateId(0); // onEnterMenu0
}

/// 1B09C0
pub unsafe extern "C" fn onEnterMenu0(arg0: *mut GameModeState) {
    let temp_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut enterMenu0_Data;
    (*temp_r3).x0 = &raw mut ext::un_803FA4E0 as *mut c_void;
    (*temp_r3).x4 = fn_801B09F8 as *const () as *mut c_void;
}

/// 1B09F8
pub unsafe extern "C" fn fn_801B09F8(arg0: i32) -> i32 {
    if arg0 == 0 {
        sfxBack();
        ext::gm_ChangeGameModeAfterCurrentScene(GM_TITLE as i32);
        ext::gm_801A4B60();
    }
    0
}

/// 1B0A34
pub unsafe extern "C" fn onEnterMenu1(arg0: *mut GameModeState) {
    let temp_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut enterMenu0_Data;
    (*temp_r3).x0 = &raw mut ext::un_803FA790 as *mut c_void;
    (*temp_r3).x4 = fn_801B0A8C as *const () as *mut c_void;
    ext::lbAudioAx_80026F2C(0x12);
    ext::lbAudioAx_8002702C(2, 0x20);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

/// 1B0A8C
pub unsafe extern "C" fn fn_801B0A8C(arg0: i32) -> i32 {
    if arg0 == 0 {
        sfxBack();
        ext::gm_SetNextGameModeStateId(0);
        ext::gm_801A4B60();
    }
    0
}

/// 1B0AC8
pub unsafe extern "C" fn onEnterMenu2(arg0: *mut GameModeState) {
    let temp_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut enterMenu0_Data;
    (*temp_r3).x0 = &raw mut ext::un_803FC4CC as *mut c_void;
    (*temp_r3).x4 = fn_801B0A8C as *const () as *mut c_void;
}

/// 1B0B00
pub unsafe extern "C" fn onEnterPrize(arg0: *mut GameModeState) {
    ext::un_802FFEE0(ext::gm_GetGameModeStateEnterData(arg0));
}

/// 1B0B24
pub unsafe extern "C" fn onExitPrize(_arg0: *mut GameModeState) {
    ext::gm_SetNextGameModeStateId(2);
}

/// 1B0B48
pub unsafe extern "C" fn onEnterVs(arg0: *mut GameModeState) {
    ext::un_802FFF2C(ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData);
    ext::lbAudioAx_80026F2C(0x12);
    ext::lbAudioAx_8002702C(2, 0x20);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
}

/// 1B0B8C
pub unsafe extern "C" fn onEnterResults0(arg0: *mut GameModeState) {
    let temp_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut ResultsMatchInfo;
    ext::gm_80177724(temp_r3);
    (*temp_r3).match_end = vs_exit_data.match_end;
}

/// 1B0BF0
pub unsafe extern "C" fn onExitResults0(arg0: *mut GameModeState) {
    ext::gm_GetGameModeStateExitData(arg0);
    ext::gm_SetNextGameModeStateId(0);
}

/// 1B0C18
pub unsafe extern "C" fn onEnterIntroEasy(arg0: *mut GameModeState) {
    let temp_r31 = ext::gm_GetGameModeStateEnterData(arg0);
    ext::gm_80168F88();
    ext::un_80301BA8(temp_r31);
}

/// 1B0C50
pub unsafe extern "C" fn onEnterIntroAllstar(arg0: *mut GameModeState) {
    ext::un_80301C64(ext::gm_GetGameModeStateEnterData(arg0) as *mut un_80301C64_t);
}

/// 1B0C74
pub unsafe extern "C" fn onEnterGameOver(arg0: *mut GameModeState) {
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut DebugGameOverData;
    (*data).x0 = 0x1869F;
    (*data).x8 = 1;
    (*data).ckind = ext::un_803FA258[0x4D] as u32;
    (*data).slot = 0;
    (*data).x15 = 0x78;
    (*data).x18 = ext::HSD_Randi(0x3E8) as u16;
    (*data).x16 = (ext::HSD_Randi(0xA) + 1) as u16;
}

/// 1B0CF0
pub unsafe extern "C" fn onEnterApproach(arg0: *mut GameModeState) {
    let temp_r3 = ext::gm_GetGameModeStateEnterData(arg0) as *mut u8;
    *temp_r3.add(1) = 0;

    *temp_r3 = match ext::un_803FA258[0x50] {
        1 => 3,
        2 => 7,
        3 => 9,
        4 => 0xA,
        5 => 0xF,
        6 => 0x14,
        7 => 0x15,
        8 => 0x16,
        9 => 0x17,
        10 => 0x18,
        11 => 0x19,
        _ => 0x19, // default and 12
    };
}

/// 1B0DD0
pub unsafe extern "C" fn onEnterResults1(arg0: *mut GameModeState) {
    let mut sfx_result: u64 = 0;
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut DebugResultsData;
    let match_end = &raw mut (*data).match_end;
    let r = &raw const ext::un_803FA258;

    (*data).set_x0_0((*r)[0x5E] as u8);
    (*data).set_x0_1((*r)[0x5F] as u8);
    (*data).x1 = (*r)[0x5A] as u8;
    (*data).x2 = (*r)[0x5B] as u8;
    (*data).x3 = (*r)[0x5C] as u8;
    (*data).x4 = (*r)[0x5D] as u8;

    ext::gm_80166A98(
        match_end,
        ((*r)[0x60] & 0xFF) as u8,
        (*r)[0x51] as i8,
        ((*r)[0x56] - 1) as u8,
        (*r)[0x52] as i8,
        ((*r)[0x57] - 1) as u8,
        (*r)[0x53] as i8,
        ((*r)[0x58] - 1) as u8,
        (*r)[0x54] as i8 as u8,
        ((*r)[0x59] - 1) as u8,
    );

    for i in 0..4 {
        if (*match_end).player_standings[i].pkind != 3 && (*match_end).player_standings[i].is_big_loser == 0 {
            sfx_result |= ext::lbAudioAx_80026E84((*match_end).player_standings[i].ckind as u32);
        }
    }

    ext::lbAudioAx_80026F2C(0x14);
    ext::lbAudioAx_8002702C(0x4, sfx_result);
    ext::gm_LoadAnnouncer();
    ext::gm_801701A0();
}

/// 1B0F1C
pub unsafe extern "C" fn onExitResults1(_arg0: *mut GameModeState) {
    ext::lbAudioAx_800236DC();
    if ext::HSD_PadCopyStatus[0].button & HSD_PAD_L != 0 {
        ext::gm_SetNextGameModeStateId(11); // onEnterResults1
    } else {
        ext::gm_SetNextGameModeStateId(0); // onEnterMenu0
    }
}

/// 1B0F60
pub unsafe extern "C" fn onEnterMemCard(arg0: *mut GameModeState) {
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut DebugMemcardData;
    (*data).x0 = 1;
    (*data).x4 = 0;
}

/// 1B0F90
pub unsafe extern "C" fn onExitMemCard(arg0: *mut GameModeState) {
    ext::gm_GetGameModeStateExitData(arg0);
    ext::gm_SetNextGameModeStateId(0);
}
