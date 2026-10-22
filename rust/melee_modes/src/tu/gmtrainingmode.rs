#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::all)]
//! Native port of src/melee/gm/gmtrainingmode.c (the whole TU: Training mode's CSS/SSS/match
//! state table, its enter/exit hooks and the mode's OnInit/OnLoad). Calls out of the TU go
//! through `ext` (a port where one exists in this crate, else a TODO panic). Checked against the C
//! (whole TU included verbatim, shipping contraction) by tests/tu_gmtrainingmode.rs.
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
        gm_GetGameModeStateEnterData(state: *mut GameModeState) -> *mut c_void = |_| todo!("TODO(port) gm_GetGameModeStateEnterData (gm_1A3F.c)");
        gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void = |_| todo!("TODO(port) gm_GetGameModeStateExitData (gm_1A3F.c)");
        lbCardNew_AllocWorkArea() = || todo!("TODO(port) lbCardNew_AllocWorkArea (lbcardnew.c)");
        lbCardGame_LoadArchive(arg: i32) = |_| todo!("TODO(port) lbCardGame_LoadArchive (lbcardgame.c)");
        lbCardGame_SaveChanges() = || todo!("TODO(port) lbCardGame_SaveChanges (lbcardgame.c)");
        gm_801B06B0(css: *mut CSSData, match_type: u8, ckind: i8, stocks: i8, color: i8, nametag: u8, cpu_level: u8, slot: u8)
            = |_, _, _, _, _, _, _, _| todo!("TODO(port) gm_801B06B0 (gm_1B03.c)");
        gm_801B07B4(css: *mut CSSData, ckind: i8, stocks: i8, color: i8, nametag: u8, level: u8, slot: u8)
            = |css, ckind, stocks, color, nametag, level, slot| crate::gm_ptr::gm_801B07B4(unsafe { &mut *css }, ckind, stocks, color, nametag, level, slot);
        lbDvd_SetupVsPreloadCache() = || todo!("TODO(port) lbDvd_SetupVsPreloadCache (lbdvd.c)");
        lbTime_8000AF74(a: u32, b: i32) -> u32 = crate::lb::lbTime_8000AF74;
        gmFrontend_TrainingSelect(state: *mut GameModeState, sss: i32) = |_, _| todo!("TODO(port) gmFrontend_TrainingSelect (gmfrontend.c)");
        gm_801B0730(css: *mut CSSData, ckind: *mut i8, stocks: *mut u8, color: *mut u8, nametag: *mut u8, cpu_level: *mut u8)
            = |css, ckind, stocks, color, nametag, level| crate::gm_ptr::gm_801B0730(unsafe { &*css }, unsafe { ckind.as_mut() }, unsafe { stocks.as_mut() }, unsafe { color.as_mut() }, unsafe { nametag.as_mut() }, unsafe { level.as_mut() });
        gm_801B07E8(css: *mut CSSData, ckind: *mut i8, stocks: *mut i8, color: *mut i8, arg4: *mut i8, level: *mut u8)
            = |css, ckind, stocks, color, arg4, level| crate::gm_ptr::gm_801B07E8(unsafe { &*css }, unsafe { ckind.as_mut() }, unsafe { stocks.as_mut() }, unsafe { color.as_mut() }, unsafe { arg4.as_mut() }, unsafe { level.as_mut() });
        gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) = |_| todo!("TODO(port) gm_ChangeGameModeAfterCurrentScene (gm_1A3F.c)");
        gm_SetupAllPlayerDefaults(players: *mut PlayerInitData) = |_| todo!("TODO(port) gm_SetupAllPlayerDefaults (gm_1B03.c)");
        gm_GetNumCostumesForCKind(ckind: u8) -> u8 = |_| todo!("TODO(port) gm_GetNumCostumesForCKind (gm_1B03.c)");
        lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState = || todo!("TODO(port) lbDvd_GetPreloadCacheScene (lbdvd.c)");
        lbDvd_80018254() = || todo!("TODO(port) lbDvd_80018254 (lbdvd.c)");
        lbAudioAx_80026E84(ckind: CharacterKind) -> u64 = |_| todo!("TODO(port) lbAudioAx_80026E84 (lbaudio_ax.c)");
        lbAudioAx_80026F2C(arg: u32) = |_| todo!("TODO(port) lbAudioAx_80026F2C (lbaudio_ax.c)");
        lbAudioAx_8002702C(flags: u32, mask: u64) = |_, _| todo!("TODO(port) lbAudioAx_8002702C (lbaudio_ax.c)");
        lbAudioAx_80027168() = || todo!("TODO(port) lbAudioAx_80027168 (lbaudio_ax.c)");
        gm_SetNextGameModeStateId(id: u8) = |_| todo!("TODO(port) gm_SetNextGameModeStateId (gm_1A3F.c)");
        lbAudioAx_80026EBC(stkind: StKind) -> u64 = |_| todo!("TODO(port) lbAudioAx_80026EBC (lbaudio_ax.c)");
        gm_SetupRulesDefaults(rules: *mut StartMeleeRules) = |_| todo!("TODO(port) gm_SetupRulesDefaults (gm_1B03.c)");
        gm_SetupHumanPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8)
            = |p, ckind, color, stocks, slot| crate::gm_r4::gm_SetupHumanPlayer(unsafe { &mut *p }, ckind, color, stocks, slot);
        gm_SetupCpuPlayer(p: *mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8)
            = |p, ckind, color, stocks, slot| crate::gm_r4::gm_SetupCpuPlayer(unsafe { &mut *p }, ckind, color, stocks, slot);
        gm_LoadRumbleEnabled(data: *mut StartMeleeData) = |_| todo!("TODO(port) gm_LoadRumbleEnabled (gm_1B03.c)");
        gm_80189CDC(data: *mut StartMeleeData) = |_| todo!("TODO(port) gm_80189CDC (gm_1884.c)");
        gm_80162968(seconds: u32) -> i32 = |_| todo!("TODO(port) gm_80162968 (gm_1601.c)");
        gm_8016247C(arg: i32) -> i32 = |_| todo!("TODO(port) gm_8016247C (gm_1601.c)");
        gm_80163298(ckind: i8, arg: u16) -> bool = |_, _| todo!("TODO(port) gm_80163298 (gm_1601.c)");
        gm_80188454(idx: i32) -> i32 = |_| todo!("TODO(port) gm_80188454 (gm_1884.c)");
        gm_80173BC4(ckind: i8) = |_| todo!("TODO(port) gm_80173BC4 (gm_16F1.c)");
        gm_80173EEC() = || todo!("TODO(port) gm_80173EEC (gm_16F1.c)");
        gm_80173754(a: u8, b: u8) -> bool = |_, _| todo!("TODO(port) gm_80173754 (gm_16F1.c)");
        lbAudioAx_80024030(arg: i32) = |_| todo!("TODO(port) lbAudioAx_80024030 (lbaudio_ax.c)");
        gm_InitVsMode(vs: *mut VsModeData) = |_| todo!("TODO(port) gm_InitVsMode (gm_1B03.c)");
        gm_801677F0() -> u8 = || todo!("TODO(port) gm_801677F0 (gm_1601.c)");
        SceneLaunch_SeedVs(vs: *mut VsModeData, dummy_fallback: bool) -> bool = |_, _| todo!("TODO(port) SceneLaunch_SeedVs (gmscenelaunch.h, TARGET_PC)");
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
    ext::gm_801B07E8(css, c_kind, stocks, color, arg4, level);
}

/// 1B1C24: the CSS is left; build the training match's players from its choices.
pub unsafe extern "C" fn gm_801B1C24(arg0: *mut GameModeState) {
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
        (*players.add(i)).color = (((*players.add(i - 1)).color as i32 + 1) % n) as u8;
        if (*players.add(i)).color == (*players.add(0)).color {
            let n = ext::gm_GetNumCostumesForCKind((*players.add(j)).ckind as u8) as i32;
            (*players.add(i)).color = (((*players.add(i)).color as i32 + 1) % n) as u8;
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
    let sss = ext::gm_GetGameModeStateEnterData(arg0) as *mut SSSData;
    (*sss).x1 = 0;
    (*sss).force_stage_id = -1;
    (*sss).unk_stage = 0;
    // TARGET_PC: the kit's stage select, when asked for (gmfrontend.c)
    ext::gmFrontend_TrainingSelect(arg0, 1);
}

/// 1B1EEC: the SSS is left; record the chosen stage.
pub unsafe extern "C" fn gm_801B1EEC(arg0: *mut GameModeState) {
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
pub unsafe extern "C" fn fn_801B1F6C(_unused: i32) {}

/// 1B1F70: the match is entered; build its StartMeleeData from the mode's rows.
pub unsafe extern "C" fn gm_801B1F70(arg0: *mut GameModeState) {
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
    gm_804D68C0 = ext::gm_801677F0();
    gm_804D68C1 = 0;
    // TARGET_PC: scene launch seeds this mode's VsModeData (gmscenelaunch.h)
    ext::SceneLaunch_SeedVs(training_row(), true);
}
