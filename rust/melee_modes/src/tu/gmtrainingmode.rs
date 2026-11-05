#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::all)]
//! Native port of src/melee/gm/gmtrainingmode.c (the whole TU: Training mode's CSS/SSS/match
//! state table, its enter/exit hooks and the mode's OnInit/OnLoad). Calls out of the TU go
//! through `ext` (a port where one exists in this crate, else a TODO panic). Checked against the C
//! (whole TU included verbatim, shipping contraction) by tests/tu_gmtrainingmode.rs.
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gm_ptr::gm_801B07E8 as gm_801B07E8_gm_r4;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::CSSData as CSSData_gm_r4;
use super::globals::{gm_80473814, gmMainLib_804D3EE0};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

/// Calls out of the TU. `ext::hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use crate::tu::ext_fns_c as ext_fns;
    use core::ffi::c_void;
    use melee_types::*;
    ext_fns! {
        gm_GetGameModeStateEnterData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(state as _) as _ } };
        gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = |state: *mut GameModeState| -> *mut c_void { unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(state as _) as _ } };
        lbCardNew_AllocWorkArea() = || { unsafe { melee_runtime::tu::lbcardnew::lbCardNew_AllocWorkArea(); } };
        lbCardGame_LoadArchive(arg: i32) = |arg: i32| { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_LoadArchive(arg as _); } };
        lbCardGame_SaveChanges() = || { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SaveChanges(); } };
        // rp3 gm_801B06B0
        gm_801B06B0(css: *mut CSSData, match_type: u8, ckind: i8, stocks: i8, color: i8, nametag: u8, cpu_level: u8, slot: u8) = |css: *mut CSSData, match_type: u8, ckind: i8, stocks: i8, color: i8, nametag: u8, cpu_level: u8, slot: u8| unsafe { crate::tu::gm_1B03::gm_801B06B0(&mut *(css as *mut _), match_type, ckind, stocks, color, nametag, cpu_level, slot); };
        // rp3-end
        gm_801B07B4(css: *mut CSSData, ckind: i8, stocks: i8, color: i8, nametag: u8, level: u8, slot: u8)
            = |css, ckind, stocks, color, nametag, level, slot| crate::gm_ptr::gm_801B07B4(unsafe { &mut *css }, ckind, stocks, color, nametag, level, slot);
        lbDvd_SetupVsPreloadCache() = || { unsafe { melee_runtime::tu::lbdvd::lbDvd_SetupVsPreloadCache(); } };
        lbTime_8000AF74(a: u32, b: i32) -> u32 = crate::lb::lbTime_8000AF74;
        gmFrontend_TrainingSelect(state: *mut GameModeState, sss: i32) = |state: *mut GameModeState, sss: i32| { unsafe { crate::tu::gmfrontend::gmFrontend_TrainingSelect(state as _, sss as _); } };
        gm_801B0730(css: *mut CSSData, ckind: *mut i8, stocks: *mut u8, color: *mut u8, nametag: *mut u8, cpu_level: *mut u8)
            = |_, _, _, _, _, _| todo!("gm_801B0730: installed by melee_link");
        gm_801B07E8(css: *mut CSSData, ckind: *mut i8, stocks: *mut i8, color: *mut i8, arg4: *mut i8, level: *mut u8)
            = |_, _, _, _, _, _| todo!("gm_801B07E8: installed by melee_link");
        gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) = |pending_mode: i32| { unsafe { crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(pending_mode as _); } };
        gm_SetupAllPlayerDefaults(players: *mut PlayerInitData) = |players: *mut PlayerInitData| { unsafe { crate::tu::gm_1601::gm_SetupAllPlayerDefaults(players as _); } };
        gm_GetNumCostumesForCKind(ckind: u8) -> u8 = |ckind: u8| -> u8 { unsafe { crate::tu::gm_1601::gm_GetNumCostumesForCKind(ckind as _) as _ } };
        lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = || -> *mut PreloadedGameModeState { unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() as _ } };
        lbDvd_80018254() = || { unsafe { melee_runtime::tu::lbdvd::lbDvd_80018254(); } };
        lbAudioAx_80026E84(ckind: CharacterKind) -> u64 = |ckind: CharacterKind| -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026E84(ckind as _) as _ } };
        lbAudioAx_80026F2C(arg: u32) = |arg: u32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(arg as _); } };
        lbAudioAx_8002702C(flags: u32, mask: u64) = |flags: u32, mask: u64| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(flags as _, mask as _); } };
        lbAudioAx_80027168() = || { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168(); } };
        gm_SetNextGameModeStateId(id: u8) = |id: u8| { unsafe { crate::tu::gm_1A3F::gm_SetNextGameModeStateId(id as _); } };
        lbAudioAx_80026EBC(stkind: StKind) -> u64 = |stkind: StKind| -> u64 { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80026EBC(stkind as _) as _ } };
        gm_SetupRulesDefaults(rules: *mut StartMeleeRules) = |rules: *mut StartMeleeRules| { unsafe { crate::tu::gm_1601::gm_SetupRulesDefaults(rules as _); } };
        gm_SetupHumanPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8)
            = |_, _, _, _, _| todo!("gm_SetupHumanPlayer: installed by melee_link");
        gm_SetupCpuPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8)
            = |_, _, _, _, _| todo!("gm_SetupCpuPlayer: installed by melee_link");
        gm_LoadRumbleEnabled(data: *mut StartMeleeData) = |data: *mut StartMeleeData| { unsafe { crate::tu::gmvs::gm_LoadRumbleEnabled(data as _); } };
        gm_80189CDC(data: *mut StartMeleeData) = |data: *mut StartMeleeData| { unsafe { crate::tu::gm_1884::gm_80189CDC(data as _); } };
        gm_80162968(seconds: u32) -> i32 = |seconds: u32| -> i32 { unsafe { crate::tu::gm_1601::gm_80162968(seconds as _) as _ } };
        gm_8016247C(arg: i32) -> i32 = |arg: i32| -> i32 { unsafe { crate::tu::gm_1601::gm_8016247C(arg as _) as _ } };
        gm_80163298(ckind: i8, arg: u16) -> bool = |ckind: i8, arg: u16| -> bool { unsafe { crate::tu::gm_1601::gm_80163298(ckind as _, arg as _) as _ } };
        gm_80188454(idx: i32) -> i32 = |idx: i32| -> i32 { unsafe { crate::tu::gm_1884::gm_80188454(idx as _) as _ } };
        gm_80173BC4(ckind: i8) = |ckind: i8| { unsafe { crate::tu::gm_1736::gm_80173BC4(ckind as _); } };
        gm_80173EEC() = || { unsafe { crate::tu::gm_1736::gm_80173EEC(); } };
        gm_80173754(a: u8, b: u8) -> bool = |a: u8, b: u8| -> bool { unsafe { crate::tu::gm_1736::gm_80173754(a as _, b as _) != 0 } };
        lbAudioAx_80024030(arg: i32) = |arg: i32| { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(arg as _); } };
        gm_InitVsMode(vs: *mut VsModeData) = |vs: *mut VsModeData| { unsafe { crate::tu::gm_1601::gm_InitVsMode(vs as _); } };
        gm_801677F0() -> u8 = || -> u8 { unsafe { crate::tu::gm_1601::gm_801677F0() as _ } };
        SceneLaunch_SeedVs(vs: *mut VsModeData, dummy_fallback: bool) -> bool = |vs, d| unsafe { crate::tu::gmscenelaunch::SceneLaunch_SeedVs(vs, d as i32) != 0 };
    }
}

/* 4D68C0 */ pub static mut gm_804D68C0: u8 = 0;
/* 4D68C1 */ pub static mut gm_804D68C1: u8 = 0;
/* 48E230 */ pub static mut css_data: CSSData = unsafe { core::mem::zeroed() };
/* 48E378 */ pub static mut sss_data: SSSData = unsafe { core::mem::zeroed() };
/* 48E4C0 */ pub static mut training_enter_data: StartMeleeData = unsafe { core::mem::zeroed() };
/* 48E5F8 */ pub static mut training_exit_data: [*mut c_void; 0x2288 / 4] = [null_mut(); 0x2288 / 4];

const fn state(
    id: u8,
    on_enter: unsafe extern "C" fn(*mut GameModeState),
    on_exit: unsafe extern "C" fn(*mut GameModeState),
    kind: GameSceneKind,
    enter: *mut c_void,
    exit: *mut c_void,
) -> GameModeState {
    GameModeState {
        id,
        preload: lbDvdPreload_3 as u8,
        flags: 0,
        on_enter: Some(on_enter),
        on_exit: Some(on_exit),
        info: GameSceneInfo { scene_kind: kind as u8, enter_data: enter, exit_data: exit },
    }
}

/// `gm_Mode_Training_States`: CSS 0, SSS 1, the match 2, and the `{ -1 }` terminator.
pub static mut gm_Mode_Training_States: [GameModeState; 4] = [
    state(0, gm_801B1B74, gm_801B1C24, GS_CSS, addr_of_mut!(css_data) as *mut c_void, addr_of_mut!(css_data) as *mut c_void),
    state(1, gm_801B1EB8, gm_801B1EEC, GS_SSS, addr_of_mut!(sss_data) as *mut c_void, addr_of_mut!(sss_data) as *mut c_void),
    state(
        2,
        gm_801B1F70,
        gm_801B2204,
        GS_TRAINING,
        addr_of_mut!(training_enter_data) as *mut c_void,
        addr_of_mut!(training_exit_data) as *mut c_void,
    ),
    GameModeState {
        id: -1i8 as u8,
        preload: 0,
        flags: 0,
        on_enter: None,
        on_exit: None,
        info: GameSceneInfo { scene_kind: 0, enter_data: null_mut(), exit_data: null_mut() },
    },
];

#[inline(always)]
unsafe fn training_row() -> *mut VsModeData {
    &mut (*gmMainLib_804D3EE0).modes.table[GmVsMode_Training as usize]
}

/// 1B1B74: the CSS is entered.
pub unsafe extern "C" fn gm_801B1B74(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B1B74");
    let vs_data = training_row();
    let css = ext::gm_GetGameModeStateEnterData(arg0) as *mut CSSData;
    if gm_804D68C1 != 0 {
        ext::lbCardNew_AllocWorkArea();
        ext::lbCardGame_LoadArchive(0);
        ext::lbCardGame_SaveChanges();
    }
    let p = &(*vs_data).start.players;
    ext::gm_801B06B0(css, 0x17, p[0].ckind, 1, p[0].color as i8, p[0].nametag, 9, gm_804D68C0);
    ext::gm_801B07B4(css, p[1].ckind, 1, p[1].color as i8, p[1].nametag, 0, gm_804D68C0);
    ext::lbDvd_SetupVsPreloadCache();
    gm_804D68C1 = ext::lbTime_8000AF74(gm_804D68C1 as u32, 1) as u8;
    // TARGET_PC: the kit's character select, when asked for (gmfrontend.c)
    ext::gmFrontend_TrainingSelect(arg0, 0);
}

/// `gm_801B07E8_layer` (static): `gm_801B07E8` through a signature the call sites share.
pub unsafe fn gm_801B07E8_layer(css: *mut CSSData, c_kind: *mut i8, stocks: *mut i8, color: *mut i8, arg4: *mut i8, level: *mut u8) {
    melee_core::ct!("gm_801B07E8_layer");
    ext::gm_801B07E8(css, c_kind, stocks, color, arg4, level);
}

/// 1B1C24: the CSS is left; build the training match's players from its choices.
pub unsafe extern "C" fn gm_801B1C24(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B1C24");
    let vs = training_row();
    let css = ext::gm_GetGameModeStateExitData(arg0) as *mut CSSData;
    if (*css).pending_scene_change == 2 {
        ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32);
        return;
    }
    let players = (*vs).start.players.as_mut_ptr();
    ext::gm_SetupAllPlayerDefaults(players);
    ext::gm_801B0730(css, &mut (*players.add(0)).ckind, null_mut(), &mut (*players.add(0)).color, &mut (*players.add(0)).nametag, null_mut());
    gm_801B07E8_layer(
        css,
        &mut (*players.add(1)).ckind,
        null_mut(),
        &mut (*players.add(1)).color as *mut u8 as *mut i8,
        &mut (*players.add(1)).nametag as *mut u8 as *mut i8,
        null_mut(),
    );
    let mut i: usize = 2;
    let mut j: usize = 2;
    (*players.add(1)).cpu_kind = 0;
    while i < 4 {
        *players.add(i) = *players.add(1);
        let n = ext::gm_GetNumCostumesForCKind((*players.add(j)).ckind as u8) as i32;
        (*players.add(i)).color = (<i32 as melee_core::ppc::Ppc>::prem((*players.add(i - 1)).color as i32 + 1, n)) as u8;
        if (*players.add(i)).color == (*players.add(0)).color {
            let n = ext::gm_GetNumCostumesForCKind((*players.add(j)).ckind as u8) as i32;
            (*players.add(i)).color = (<i32 as melee_core::ppc::Ppc>::prem((*players.add(i)).color as i32 + 1, n)) as u8;
        }
        (*players.add(i)).slot_type = 3;
        i += 1;
        j += 1;
    }
    if gm_804D68C0 == 0 {
        (*players.add(1)).slot = 0;
        (*players.add(2)).slot = 0;
        (*players.add(3)).slot = 0;
    } else {
        let mut k: usize = 1;
        for i in 0..4i32 {
            if gm_804D68C0 as i32 != i {
                (*players.add(k)).slot = (i + 1) as u8;
                k += 1;
            }
        }
    }
    let cache = &mut (*ext::lbDvd_GetPreloadCacheScene()).game_cache;
    cache.entries[2].char_id = (*players.add(2)).ckind as i32;
    cache.entries[2].color = (*players.add(2)).color;
    cache.entries[3].char_id = (*players.add(3)).ckind as i32;
    cache.entries[3].color = (*players.add(3)).color;
    ext::lbDvd_80018254();
    let mut mask: u64 = 0;
    for k in 0..4 {
        mask |= ext::lbAudioAx_80026E84((*players.add(k)).ckind as CharacterKind);
    }
    ext::lbAudioAx_80026F2C(20);
    ext::lbAudioAx_8002702C(4, mask);
    ext::lbAudioAx_80027168();
}

/// 1B1EB8: the SSS is entered.
pub unsafe extern "C" fn gm_801B1EB8(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B1EB8");
    let sss = ext::gm_GetGameModeStateEnterData(arg0) as *mut SSSData;
    (*sss).x1 = 0;
    (*sss).force_stage_id = -1;
    (*sss).unk_stage = 0;
    // TARGET_PC: the kit's stage select, when asked for (gmfrontend.c)
    ext::gmFrontend_TrainingSelect(arg0, 1);
}

/// 1B1EEC: the SSS is left; record the chosen stage.
pub unsafe extern "C" fn gm_801B1EEC(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B1EEC");
    let vs_data = training_row();
    let sss = ext::gm_GetGameModeStateExitData(arg0) as *mut SSSData;
    if (*sss).start_game == 0 {
        ext::gm_SetNextGameModeStateId(0);
        return;
    }
    let stkind = (*sss).vs.start.rules.stkind as i16;
    gm_80473814.stage_id = stkind;
    (*vs_data).start.rules.stkind = stkind as u16;
    ext::lbAudioAx_80026F2C(0x18);
    ext::lbAudioAx_8002702C(8, ext::lbAudioAx_80026EBC((*vs_data).start.rules.stkind as StKind));
    ext::lbAudioAx_80027168();
}

/// 1B1F6C
pub unsafe extern "C" fn fn_801B1F6C(_unused: i32) { melee_core::ct!("fn_801B1F6C");}

/// 1B1F70: the match is entered; build its StartMeleeData from the mode's rows.
pub unsafe extern "C" fn gm_801B1F70(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B1F70");
    let vs = training_row();
    let data = ext::gm_GetGameModeStateEnterData(arg0) as *mut StartMeleeData;
    ext::gm_SetupRulesDefaults(&mut (*data).rules);

    (*data).rules = (*vs).start.rules;
    (*data).rules.on_pause_override = Some(fn_801B1F6C);

    (*data).rules.set_x3_6(1);
    (*data).rules.set_x2_5(0);
    (*data).rules.set_x2_1(1);
    ext::gm_SetupAllPlayerDefaults((*data).players.as_mut_ptr());

    for i in 0..4 {
        (*data).players[i] = (*vs).start.players[i];
        (*data).players[i].set_xC_b1(0);
        (*data).players[i].set_xD_b2(1);
    }

    ext::gm_SetupHumanPlayer(
        &mut (*data).players[0],
        (*vs).start.players[0].ckind as u8,
        (*vs).start.players[0].color,
        1,
        gm_804D68C0,
    );

    for i in 1..4usize {
        ext::gm_SetupCpuPlayer(
            &mut (*data).players[i],
            (*vs).start.players[i].ckind as u8,
            (*vs).start.players[i].color,
            1,
            (*vs).start.players[i].slot.wrapping_sub(1),
        );
        if i - 1 != 0 {
            (*data).players[i].slot_type = Gm_PKind_NA as u8;
        }
    }

    ext::gm_LoadRumbleEnabled(data);
    ext::gm_80189CDC(data);
}

/// 1B2204: the match is left; record the results.
pub unsafe extern "C" fn gm_801B2204(arg0: *mut GameModeState) {
    melee_core::ct!("gm_801B2204");
    let temp_r3 = ext::gm_GetGameModeStateExitData(arg0) as *mut MatchExitInfo;
    ext::gm_80162968((*temp_r3).match_end.frame_count / 60);
    ext::gm_8016247C((*temp_r3).match_end.player_standings[0].xE as i32);
    let ck = (*temp_r3).match_end.player_standings[0].ckind;
    ext::gm_80163298(ck, ext::gm_80188454(ck as i32) as u16);
    ext::gm_80173BC4(ck);
    ext::gm_80173EEC();
    if !ext::gm_80173754(28, gm_804D68C0) {
        ext::gm_SetNextGameModeStateId(0);
    }
    ext::lbAudioAx_80024030(1); // sfxForward()
}

/// `gm_Mode_Training_OnInit`
pub unsafe fn gm_Mode_Training_OnInit() {
    melee_core::ct!("gm_Mode_Training_OnInit");
    let temp_r31 = training_row();
    ext::gm_InitVsMode(temp_r31);
    for i in 0..4usize {
        (*temp_r31).start.players[i].color = i as u8;
        (*temp_r31).start.players[i].cpu_kind = 0;
        if i != 0 {
            (*temp_r31).start.players[1].ckind = ChKind_None as i8;
        }
        gm_80473814.saved_players[i] = (*temp_r31).start.players[i];
    }
}

/// `gm_Mode_Training_OnLoad`
pub unsafe fn gm_Mode_Training_OnLoad() {
    melee_core::ct!("gm_Mode_Training_OnLoad");
    gm_804D68C0 = ext::gm_801677F0();
    gm_804D68C1 = 0;
    // TARGET_PC: scene launch seeds this mode's VsModeData (gmscenelaunch.h)
    ext::SceneLaunch_SeedVs(training_row(), true);
}

// ---- folded from gm_r4.rs (consolidation steps 21/22) ----
/// gmtrainingmode.c `gm_801B07E8_layer_fp` (static): forwards to gm_1B03's `gm_801B07E8_gm_r4`.
pub fn gm_801B07E8_layer_fp(css_data_p: &CSSData_gm_r4, c_kind: Option<&mut i8>, stocks: Option<&mut i8>, color: Option<&mut i8>,
                         arg4: Option<&mut i8>, level: Option<&mut u8>) {
    melee_core::ct!("gm_801B07E8_layer_fp");
    gm_801B07E8_gm_r4(css_data_p, c_kind, stocks, color, arg4, level);
}
