//! src/melee/gm/gm_1A3F.c: the game's top-level mode/state loop. `gm_801A4510` (UnclePunch's `Scene_Main`) runs
//! `runGameMode` for the current mode forever; `runGameMode` loads the mode and runs `gm_801A4014` (one scene
//! of one state: preload, state/scene enter, the frame loop, scene/state exit, advance to the next state) until
//! a mode change is pending; the `state_machine` accessors in between are the scene API the whole game uses
//! (`gm_ChangeGameModeAfterCurrentScene`, `gm_SetNextGameModeStateId`...). All 21 functions (`preloadState`,
//! `firstState`/`nextState`/`findState`/`findMode` inlines included) are checked against the C by a state
//! oracle (tests/tu_gm_1A3F.rs over gen/turef/tu_gm_1A3F_ref.c): the game's gm_1A3F.c compiled natively
//! (shipping contraction) with every callee stubbed and logged and a scripted frame loop; the callee log, the
//! whole `state_machine`, the preload cache and the return value are compared after every call.
//!
//! The TARGET_PC additions (scene trace, the Atlas scene policy and its stand-in scenes, the frontend router, the
//! 1P interstage) are part of the port and are ported too, with their callees as hooks.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

pub use crate::gm::gm_Is1PMode;

/// `GM_FRONTEND` (gm/gmfrontend.h): the port's own frontend screens, `GM_COUNT + 1`.
pub const GM_FRONTEND: u8 = GM_COUNT as u8 + 1;
const GM_COUNT_U8: u8 = GM_COUNT as u8;
const U8_MAX: i32 = 255;
pub const HSD_PAD_B: u32 = 1 << 9;

/// `routingInfo`, `stateMachine`, the `state_machine` global and the two mode accessors: owned by
/// `melee_world::gm` (the world TUs read them too).
pub use melee_world::gm::{gm_GetCurrentGameMode, gm_GetPreviousGameMode, routingInfo, stateMachine, state_machine};

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics, except
/// `gm_801A3EF4` (gm_1A36.c, ported), which defaults to this crate's port.
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
    pub type SceneFn = Option<unsafe extern "C" fn()>;
    // lb/lbdvd.c, lb/lbheap.c, lb/lbcardnew.c, lb/lbcardgame.c, lb/lbsnap.c, lb/lbmthp.c, lb/lbaudio_ax.c, lb/lbcardgame.c
    pub static mut LBDVD_80018CF4: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_80018CF4(preload: i32)  {
                match unsafe { LBDVD_80018CF4 } {
                    Some(f) => f(preload),
                    None => { unsafe { let _ = melee_runtime::tu::lbdvd::lbDvd_80018CF4(preload as _); } return; },
                }
            }
    pub static mut LBDVD_GETPRELOADCACHESCENE: Option<fn() -> *mut PreloadedGameModeState> = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_GetPreloadCacheScene() -> *mut PreloadedGameModeState {
                match unsafe { LBDVD_GETPRELOADCACHESCENE } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbdvd::lbDvd_GetPreloadCacheScene() as _ }; },
                }
            }
    pub static mut LBDVD_80018254: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_80018254()  {
                match unsafe { LBDVD_80018254 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbdvd::lbDvd_80018254(); } return; },
                }
            }
    pub static mut LBDVD_80018F58: Option<fn(bool) > = None;
            #[allow(non_snake_case)]
            pub fn lbDvd_80018F58(value: bool)  {
                match unsafe { LBDVD_80018F58 } {
                    Some(f) => f(value),
                    None => { unsafe { let _ = melee_runtime::tu::lbdvd::lbDvd_80018F58(value as _); } return; },
                }
            }
    pub static mut LBHEAP_80015BB8: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbHeap_80015BB8(idx: i32) -> i32 {
                match unsafe { LBHEAP_80015BB8 } {
                    Some(f) => f(idx),
                    None => { return unsafe { melee_runtime::tu::lbheap::lbHeap_80015BB8(idx as _) as _ }; },
                }
            }
    pub static mut LBCARDNEW_FORGETMEMORY: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_ForgetMemory()  {
                match unsafe { LBCARDNEW_FORGETMEMORY } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbcardnew::lbCardNew_ForgetMemory(); } return; },
                }
            }
    pub static mut LBCARDGAME_RESET: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_Reset()  {
                match unsafe { LBCARDGAME_RESET } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbcardgame::lbCardGame_Reset(); } return; },
                }
            }
    pub static mut LBSNAP_8001E27C: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001E27C()  {
                match unsafe { LBSNAP_8001E27C } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbsnap::lbSnap_8001E27C(); } return; },
                }
            }
    pub static mut LB_8001CDB4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lb_8001CDB4()  {
                match unsafe { LB_8001CDB4 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbcardgame::lb_8001CDB4(); } return; },
                }
            }
    pub static mut LBCARDNEW_COMPLETEALLTASKS: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_CompleteAllTasks(result: i32) -> i32 {
                match unsafe { LBCARDNEW_COMPLETEALLTASKS } {
                    Some(f) => f(result),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteAllTasks(result as _) as _ }; },
                }
            }
    pub static mut LBCARDNEW_COMPLETENEXTTASK: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_CompleteNextTask() -> i32 {
                match unsafe { LBCARDNEW_COMPLETENEXTTASK } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lbCardNew_CompleteNextTask() as _ }; },
                }
            }
    pub static mut LBMTHP_8001F800: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbMthp_8001F800()  {
                match unsafe { LBMTHP_8001F800 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbmthp::lbMthp_8001F800(); } return; },
                }
            }
    pub static mut LBAUDIOAX_80027DBC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80027DBC()  {
                match unsafe { LBAUDIOAX_80027DBC } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80027DBC(); } return; },
                }
            }
    // ty/toy.c, ty/tydisplay.c
    pub static mut TOY_803127D4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn Toy_803127D4()  {
                match unsafe { TOY_803127D4 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_803127D4(); } return; },
                }
            }
    pub static mut TYDISPLAY_8031C8B8: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn tyDisplay_8031C8B8()  {
                match unsafe { TYDISPLAY_8031C8B8 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_items::tu::tydisplay::tyDisplay_8031C8B8(); } return; },
                }
            }
    // sysdolphin/baselib: sislib.c, controller.c, devcom.c, video.c
    pub static mut HSD_SISLIB_803A6048: Option<fn(usize) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6048(size: usize)  {
                match unsafe { HSD_SISLIB_803A6048 } {
                    Some(f) => f(size),
                    None => { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6048(size as _); } return; },
                }
            }
    pub static mut HSD_PADRESET: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn HSD_PadReset()  {
                match unsafe { HSD_PADRESET } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::controller::HSD_PadReset(); } return; },
                }
            }
    pub static mut HSD_DEVCOMISBUSY: Option<fn(i32) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn HSD_DevComIsBusy(idx: i32) -> bool {
                match unsafe { HSD_DEVCOMISBUSY } {
                    Some(f) => f(idx),
                    None => { return unsafe { melee_runtime::tu::devcom::HSD_DevComIsBusy(idx as _) as _ }; },
                }
            }
    pub static mut HSD_VISETBLACK: Option<fn(bool) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_VISetBlack(v: bool)  {
                match unsafe { HSD_VISETBLACK } {
                    Some(f) => f(v),
                    None => { unsafe { let _ = melee_runtime::tu::video::HSD_VISetBlack(v as _); } return; },
                }
            }
    // dolphin: dvd.c, os.c, vi.c
    hook!(DVDCHECKDISK, DVDCheckDisk, () -> i32);
    hook!(OSRESETSYSTEM, OSResetSystem, (a: i32, b: u32, c: i32));
    hook!(VIGETDTVSTATUS, VIGetDTVStatus, () -> u32);
    hook!(OSGETPROGRESSIVEMODE, OSGetProgressiveMode, () -> u32);
    // gm/gmscene.c, gm/gmscdata.c, gm/gmmain_lib.c
    pub static mut GM_FINDGAMESCENEHANDLER: Option<fn(u8) -> *mut GameScene> = None;
            #[allow(non_snake_case)]
            pub fn gm_FindGameSceneHandler(kind: u8) -> *mut GameScene {
                match unsafe { GM_FINDGAMESCENEHANDLER } {
                    Some(f) => f(kind),
                    None => { return unsafe { crate::tu::gmscene::gm_FindGameSceneHandler(kind as _) as _ }; },
                }
            }
    pub static mut GM_801A4BD4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4BD4()  {
                match unsafe { GM_801A4BD4 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = crate::tu::gmscene::gm_801A4BD4(); } return; },
                }
            }
    pub static mut GM_801A4B88: Option<fn(*mut GameSceneInfo) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4B88(info: *mut GameSceneInfo)  {
                match unsafe { GM_801A4B88 } {
                    Some(f) => f(info),
                    None => { unsafe { let _ = crate::tu::gmscene::gm_801A4B88(info as _); } return; },
                }
            }
    pub static mut GM_801A4D34: Option<fn(SceneFn, *mut GameSceneInfo) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4D34(on_frame: SceneFn, info: *mut GameSceneInfo)  {
                match unsafe { GM_801A4D34 } {
                    Some(f) => f(on_frame, info),
                    None => { unsafe { let _ = crate::tu::gmscene::gm_801A4D34(on_frame as _, info as _); } return; },
                }
            }
    pub static mut GM_GETALLGAMEMODES: Option<fn() -> *mut GameMode> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetAllGameModes() -> *mut GameMode {
                match unsafe { GM_GETALLGAMEMODES } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmscdata::gm_GetAllGameModes() as _ }; },
                }
            }
    pub static mut GMMAINLIB_8015FBA4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015FBA4()  {
                match unsafe { GMMAINLIB_8015FBA4 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = crate::tu::gmmain_lib::gmMainLib_8015FBA4(); } return; },
                }
            }
    /// `gmMainLib_8046B0F0.resetting` (gm/gmmain_lib.c): read, and cleared by `gm_801A4510`.
    hook!(GMMAINLIB_8046B0F0_RESETTING, gmMainLib_8046B0F0_resetting, () -> bool);
    hook!(SET_GMMAINLIB_8046B0F0_RESETTING, set_gmMainLib_8046B0F0_resetting, (v: bool));
    hook!(SET_GMMAINLIB_8046B0F0_SKIP_INTRO, set_gmMainLib_8046B0F0_skip_intro, (v: bool));
    /// `db_gameLaunchButtonState` (db/db.c).
    hook!(DB_GAMELAUNCHBUTTONSTATE, db_gameLaunchButtonState, () -> u16);
    // TARGET_PC: pc/platform (scene report, Atlas scene policy, frontend, fighter controls, 1P barrier)
    hook!(SCENEREPORT_STATE, SceneReport_State, (phase: i32, mode: i32, state_id: i32, scene_kind: i32));
    hook!(UI_SCENEPOLICY, Ui_ScenePolicy, (kind: i32) -> i32);
    hook!(GMFRONTEND_ATLASSTANDIN, gmFrontend_AtlasStandIn, (kind: u8) -> *mut GameScene);
    hook!(UI_SCENEPOLICYMASK, Ui_ScenePolicyMask, (scene_kind: i32, window_known: i32));
    hook!(FAD_TOYWINDOWKNOWN, Fad_ToyWindowKnown, (kind: u8) -> i32);
    hook!(FIGHTERCONTROLS_MATCHCLEAR, FighterControls_MatchClear, ());
    hook!(UI_TOYPROBE, Ui_ToyProbe, () -> i32);
    hook!(FAD_OVERLAYWRAP, Fad_OverlayWrap, (kind: u8, inner: SceneFn) -> SceneFn);
    hook!(SCRIPT_ONEPBARRIERPENDING, Script_OnePBarrierPending, () -> i32);
    pub static mut GM_ONEPINTERSTAGE: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_OnePInterstage()  {
                match unsafe { GM_ONEPINTERSTAGE } {
                    Some(f) => f(),
                    None => { unsafe { let _ = crate::tu::gmscene::gm_OnePInterstage(); } return; },
                }
            }
    pub static mut GMFRONTEND_ROUTE: Option<fn(u8, u8) -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gmFrontend_Route(from: u8, to: u8) -> u8 {
                match unsafe { GMFRONTEND_ROUTE } {
                    Some(f) => f(from, to),
                    None => { return unsafe { crate::tu::gmfrontend::gmFrontend_Route(from as _, to as _) as _ }; },
                }
            }
    pub static mut GMFRONTEND_REPORTEDMODE: Option<fn() -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gmFrontend_ReportedMode() -> u8 {
                match unsafe { GMFRONTEND_REPORTEDMODE } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmfrontend::gmFrontend_ReportedMode() as _ }; },
                }
            }
    /// `gm_801A3EF4()` (gm/gm_1A36.c, ported).
    pub static mut GM_801A3EF4: Option<fn()> = None;
    pub fn gm_801A3EF4() {
        match unsafe { GM_801A3EF4 } {
            Some(f) => f(),
            None => crate::tu::gm_1A36::gm_801A3EF4(),
        }
    }
}

/// `preloadState(state)`: preload the state's files, reserve the SIS buffer for its scene kind, mark the
/// persistent heaps and reset the card/snap/toy/trophy-display state.
pub unsafe fn preloadState(state: *mut GameModeState) {
    ext::lbDvd_80018CF4((*state).preload as i32);
    let k = (*state).info.scene_kind as i32;
    if k == GS_STAFFROLL as i32 || k == GS_RESULTS as i32 {
        ext::HSD_SisLib_803A6048(0xC000);
    } else if k == GS_CSS as i32 {
        ext::HSD_SisLib_803A6048(0x2400);
    } else {
        ext::HSD_SisLib_803A6048(0x4800);
    }
    let preloaded_state = ext::lbDvd_GetPreloadCacheScene();
    if ext::lbHeap_80015BB8(2) == 0 {
        (*preloaded_state).is_heap_persistent[0] = 1;
    }
    if ext::lbHeap_80015BB8(3) == 0 {
        (*preloaded_state).is_heap_persistent[1] = 1;
    }
    ext::lbDvd_80018254();
    ext::lbCardNew_ForgetMemory();
    ext::lbCardGame_Reset();
    ext::lbSnap_8001E27C();
    ext::Toy_803127D4();
    ext::tyDisplay_8031C8B8();
}

/// `firstState(state, next_id)`: the C's loop returns on its first pass (see `gm_ptr::firstState`).
pub unsafe fn firstState(state: *mut GameModeState, next_id: u8) -> u8 {
    crate::gm_ptr::firstState(core::slice::from_raw_parts(state, 1), next_id)
}

/// `nextState(states)`: the id of the first state whose id is above the current one, else the first state's.
pub unsafe fn nextState(states: *mut GameModeState) -> u8 {
    let mut next = states;
    let curr_id = state_machine.routing.curr_state_id;
    let mut cur = states;
    let mut i: usize = 0;
    let mut next_id: u8;

    loop {
        next_id = (*next).id;
        if next_id == 0xFF {
            break;
        }
        if (*cur).id > curr_id {
            return (*states.add(i)).id;
        }
        cur = cur.add(1);
        next = next.add(1);
        i += 1;
    }

    firstState(states, next_id)
}

/// `findState(state)`: the first state (scanning ids from the current one up) with a matching id, or NULL.
pub unsafe fn findState(state: *mut GameModeState) -> *mut GameModeState {
    let mut i = state_machine.routing.curr_state_id as i32;
    while i < U8_MAX {
        let mut j = 0usize;
        while (*state.add(j)).id != 0xFF {
            if i == (*state.add(j)).id as i32 {
                return state.add(j);
            }
            j += 1;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

/// `findMode(kind)`: the mode table entry of that kind, or NULL.
pub unsafe fn findMode(kind: u8) -> *mut GameMode {
    let mut cur = ext::gm_GetAllGameModes();
    while (*cur).kind != GM_COUNT_U8 {
        if (*cur).kind == kind {
            return cur;
        }
        cur = cur.add(1);
    }
    core::ptr::null_mut()
}

/// `gm_801A4014(mode)`: run one scene of the current state of `mode` and advance the state.
pub unsafe fn gm_801A4014(mode: *mut GameMode) {
    let state = findState((*mode).states);
    state_machine.routing.curr_state_id = (*state).id;

    // TARGET_PC: scene trace.
    ext::SceneReport_State(0, (*mode).kind as i32, (*state).id as i32, (*state).info.scene_kind as i32);
    preloadState(state);
    if let Some(f) = (*state).on_enter {
        f(state);
    }
    let info: *mut GameSceneInfo = &mut (*state).info;
    let kind = (*info).scene_kind;
    let mut scene = ext::gm_FindGameSceneHandler(kind);
    // TARGET_PC: the Atlas scene policy (REPLACE swaps in the stand-in's on_enter/on_frame pair).
    if ext::Ui_ScenePolicy(kind as i32) == 2 {
        let stand_in = ext::gmFrontend_AtlasStandIn(kind);
        if !stand_in.is_null() {
            scene = stand_in;
        }
    }
    ext::Ui_ScenePolicyMask(kind as i32, ext::Fad_ToyWindowKnown(kind));
    ext::gm_801A4BD4();
    ext::gm_801A4B88(info);
    // TARGET_PC: clear the previous match's fighter-control mapping.
    ext::FighterControls_MatchClear();
    if let Some(f) = (*scene).on_enter {
        f((*info).enter_data);
    }
    // TARGET_PC: an OVERLAY scene (or the toy probe) gets a per-frame wrapper.
    let mut frame_fn = (*scene).on_frame;
    if ext::Ui_ScenePolicy(kind as i32) == 1 || ext::Ui_ToyProbe() != 0 {
        frame_fn = ext::Fad_OverlayWrap(kind, (*scene).on_frame);
    }
    ext::gm_801A4D34(frame_fn, info);
    ext::SceneReport_State(1, (*mode).kind as i32, (*state).id as i32, (*info).scene_kind as i32);
    if !ext::gmMainLib_8046B0F0_resetting() {
        if let Some(f) = (*scene).on_exit {
            f((*info).exit_data);
        }
    }
    if !ext::gmMainLib_8046B0F0_resetting() {
        if let Some(f) = (*state).on_exit {
            f(state);
        }

        // TARGET_PC
        if ext::Script_OnePBarrierPending() != 0 {
            ext::gm_OnePInterstage();
        }
        state_machine.routing.prev_state_id = state_machine.routing.curr_state_id;

        if state_machine.routing.next_state_id != 0 {
            state_machine.routing.curr_state_id = state_machine.routing.next_state_id - 1;
            state_machine.routing.next_state_id = 0;
        } else {
            state_machine.routing.curr_state_id = nextState((*mode).states);
        }
    }
    ext::lb_8001CDB4();
    ext::lbCardNew_CompleteAllTasks(11);
    ext::lbMthp_8001F800();
    if ext::gmMainLib_8046B0F0_resetting() {
        ext::lbAudioAx_80027DBC();
        ext::HSD_PadReset();
        while ext::lbCardNew_CompleteNextTask() == 11 {}
        if ext::DVDCheckDisk() == 0 {
            ext::OSResetSystem(1, 0, 0);
        }
        ext::lbMthp_8001F800();
        while ext::HSD_DevComIsBusy(1) {}
        ext::gmMainLib_8015FBA4();
        ext::gm_GetAllGameModes();
        state_machine = core::mem::zeroed();
        ext::gm_801A3EF4();
        ext::set_gmMainLib_8046B0F0_skip_intro(true);
        gm_ChangeGameModeAfterCurrentScene(GM_BOOT as i32);
        ext::HSD_VISetBlack(false);
    }
}

/// `gm_GetGameModeStateEnterData(scene)`: the scene's enter data.
pub unsafe fn gm_GetGameModeStateEnterData(scene: *mut GameModeState) -> *mut c_void {
    (*scene).info.enter_data
}

/// `gm_GetGameModeStateExitData(state)`: the scene's exit data.
pub unsafe fn gm_GetGameModeStateExitData(state: *mut GameModeState) -> *mut c_void {
    (*state).info.exit_data
}

/// `gm_SetGameModeStateId(id)`: jump to a state (current and previous).
pub fn gm_SetGameModeStateId(id: u8) {
    unsafe {
        state_machine.routing.curr_state_id = id;
        state_machine.routing.prev_state_id = id;
    }
}

/// `gm_SetNextGameModeStateId(curr_id)`: actually sets the next scene to the scene following the input.
pub fn gm_SetNextGameModeStateId(curr_id: u8) {
    unsafe {
        state_machine.routing.next_state_id = curr_id.wrapping_add(1);
    }
}

/// `gm_GetPreviousSceneIndex()`.
pub fn gm_GetPreviousSceneIndex() -> u8 {
    unsafe { state_machine.routing.prev_state_id }
}

/// `gm_GetCurrentSceneIndex()`.
pub fn gm_GetCurrentSceneIndex() -> u8 {
    unsafe { state_machine.routing.curr_state_id }
}

/// `gm_SetNewGameModePending()`.
pub fn gm_SetNewGameModePending() {
    unsafe {
        state_machine.pending_mode_change = 1;
    }
}

/// `gm_SetPendingGameMode(pending_mode)`.
pub fn gm_SetPendingGameMode(pending_mode: u8) {
    unsafe {
        state_machine.routing.pending_mode = pending_mode;
    }
}

/// `gm_ChangeGameModeAfterCurrentScene(pending_mode)`.
pub fn gm_ChangeGameModeAfterCurrentScene(pending_mode: i32) {
    unsafe {
        state_machine.routing.pending_mode = pending_mode as u8;
        state_machine.pending_mode_change = 1;
    }
}

/// `gm_GetCurrentGameMode()`.

/// `gm_GetPreviousGameMode()`.

/// `gm_SetGameModeOverride(mode)`: the callback that may name a mode to run a single scene of.
pub fn gm_SetGameModeOverride(mode: Option<unsafe extern "C" fn() -> u8>) {
    unsafe {
        state_machine.get_override = mode;
    }
}

/// `runGameMode(mode_kind)`: load the mode, run its scenes until a mode change is pending, unload it, and
/// return the pending mode.
pub unsafe fn runGameMode(mode_kind: u8) -> u8 {
    let mode = findMode(mode_kind);

    state_machine.pending_mode_change = 0;
    state_machine.routing.curr_state_id = 0;
    state_machine.routing.prev_state_id = 0;
    state_machine.routing.next_state_id = 0;
    ext::lbDvd_80018F58((*mode).preloaded != 0);
    if let Some(f) = (*mode).on_load {
        f();
    }
    while state_machine.pending_mode_change == 0 {
        let mut over = GM_COUNT_U8;
        let has = match state_machine.get_override {
            Some(f) => {
                over = f();
                over != GM_COUNT_U8
            }
            None => false,
        };
        if has {
            state_machine.backup_routing = state_machine.routing;
            state_machine.pending_mode_change = 0;
            state_machine.routing.curr_state_id = 0;
            state_machine.routing.prev_state_id = 0;
            state_machine.routing.next_state_id = 0;

            gm_801A4014(findMode(over));
            if !ext::gmMainLib_8046B0F0_resetting() {
                state_machine.routing = state_machine.backup_routing;
            }
        } else {
            gm_801A4014(mode);
        }
    }
    if !ext::gmMainLib_8046B0F0_resetting() {
        if let Some(f) = (*mode).on_unload {
            f();
        }
    }
    state_machine.routing.pending_mode
}

/// `gm_801A4510()` (UnclePunch: `Scene_Main`): initialise the modes and run them forever (it never returns).
pub unsafe fn gm_801A4510() {
    ext::gm_GetAllGameModes();
    state_machine = core::mem::zeroed();
    let modes = ext::gm_GetAllGameModes();
    let mut i = 0usize;
    while (*modes.add(i)).kind != GM_COUNT_U8 {
        if let Some(f) = (*modes.add(i)).on_init {
            f();
        }
        i += 1;
    }
    if ext::VIGetDTVStatus() != 0
        && ((ext::db_gameLaunchButtonState() as u32 & HSD_PAD_B) != 0 || ext::OSGetProgressiveMode() == 1)
    {
        state_machine.routing.curr_mode = GM_PROGRESSIVE_SCAN as u8;
    } else {
        state_machine.routing.curr_mode = GM_BOOT as u8;
    }
    state_machine.routing.prev_mode = GM_COUNT_U8;

    loop {
        let mut next_mode = runGameMode(state_machine.routing.curr_mode);
        if ext::gmMainLib_8046B0F0_resetting() {
            ext::set_gmMainLib_8046B0F0_resetting(false);
        } else {
            // TARGET_PC: the port's frontend routes a screen between this mode and the next.
            next_mode = ext::gmFrontend_Route(state_machine.routing.curr_mode, next_mode);
        }
        state_machine.routing.prev_mode = state_machine.routing.curr_mode;
        if state_machine.routing.curr_mode == GM_FRONTEND {
            state_machine.routing.prev_mode = ext::gmFrontend_ReportedMode();
        }
        state_machine.routing.curr_mode = next_mode;
    }
}
