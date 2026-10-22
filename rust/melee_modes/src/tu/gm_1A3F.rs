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

/// `struct routingInfo` (gm_1A3F.c), GameCube size 6.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct routingInfo {
    /// `::GameModeKind`
    pub curr_mode: u8,
    /// `::GameModeKind`
    pub pending_mode: u8,
    /// `::GameModeKind`
    pub prev_mode: u8,
    /// from `::GameModeState::id`
    pub curr_state_id: u8,
    pub prev_state_id: u8,
    pub next_state_id: u8,
}
melee_core::gc_layout!(routingInfo, "routingInfo", size 6, align 1, [
    curr_mode @ 0x0: 1 "u8",
    pending_mode @ 0x1: 1 "u8",
    prev_mode @ 0x2: 1 "u8",
    curr_state_id @ 0x3: 1 "u8",
    prev_state_id @ 0x4: 1 "u8",
    next_state_id @ 0x5: 1 "u8",
]);

/// `struct stateMachine` (gm_1A3F.c), GameCube size 0x14.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct stateMachine {
    pub routing: routingInfo,
    pub backup_routing: routingInfo,
    /// `::bool`
    pub pending_mode_change: u8,
    pub get_override: Option<unsafe extern "C" fn() -> u8>,
}
melee_core::gc_layout!(stateMachine, "stateMachine", size 0x14, align 4, [
    routing @ 0x0: 6 "struct routingInfo",
    backup_routing @ 0x6: 6 "struct routingInfo",
    pending_mode_change @ 0xC: 1 "u8",
    get_override @ 0x10: 4 "u8 (*)(void)",
]);

/// `static struct stateMachine state_machine;`
pub static mut state_machine: stateMachine = stateMachine {
    routing: routingInfo { curr_mode: 0, pending_mode: 0, prev_mode: 0, curr_state_id: 0, prev_state_id: 0, next_state_id: 0 },
    backup_routing: routingInfo { curr_mode: 0, pending_mode: 0, prev_mode: 0, curr_state_id: 0, prev_state_id: 0, next_state_id: 0 },
    pending_mode_change: 0,
    get_override: None,
};

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
    hook!(LBDVD_80018CF4, lbDvd_80018CF4, (preload: i32));
    hook!(LBDVD_GETPRELOADCACHESCENE, lbDvd_GetPreloadCacheScene, () -> *mut PreloadedGameModeState);
    hook!(LBDVD_80018254, lbDvd_80018254, ());
    hook!(LBDVD_80018F58, lbDvd_80018F58, (value: bool));
    hook!(LBHEAP_80015BB8, lbHeap_80015BB8, (idx: i32) -> i32);
    hook!(LBCARDNEW_FORGETMEMORY, lbCardNew_ForgetMemory, ());
    hook!(LBCARDGAME_RESET, lbCardGame_Reset, ());
    hook!(LBSNAP_8001E27C, lbSnap_8001E27C, ());
    hook!(LB_8001CDB4, lb_8001CDB4, ());
    hook!(LBCARDNEW_COMPLETEALLTASKS, lbCardNew_CompleteAllTasks, (result: i32) -> i32);
    hook!(LBCARDNEW_COMPLETENEXTTASK, lbCardNew_CompleteNextTask, () -> i32);
    hook!(LBMTHP_8001F800, lbMthp_8001F800, ());
    hook!(LBAUDIOAX_80027DBC, lbAudioAx_80027DBC, ());
    // ty/toy.c, ty/tydisplay.c
    hook!(TOY_803127D4, Toy_803127D4, ());
    hook!(TYDISPLAY_8031C8B8, tyDisplay_8031C8B8, ());
    // sysdolphin/baselib: sislib.c, controller.c, devcom.c, video.c
    hook!(HSD_SISLIB_803A6048, HSD_SisLib_803A6048, (size: usize));
    hook!(HSD_PADRESET, HSD_PadReset, ());
    hook!(HSD_DEVCOMISBUSY, HSD_DevComIsBusy, (idx: i32) -> bool);
    hook!(HSD_VISETBLACK, HSD_VISetBlack, (v: bool));
    // dolphin: dvd.c, os.c, vi.c
    hook!(DVDCHECKDISK, DVDCheckDisk, () -> i32);
    hook!(OSRESETSYSTEM, OSResetSystem, (a: i32, b: u32, c: i32));
    hook!(VIGETDTVSTATUS, VIGetDTVStatus, () -> u32);
    hook!(OSGETPROGRESSIVEMODE, OSGetProgressiveMode, () -> u32);
    // gm/gmscene.c, gm/gmscdata.c, gm/gmmain_lib.c
    hook!(GM_FINDGAMESCENEHANDLER, gm_FindGameSceneHandler, (kind: u8) -> *mut GameScene);
    hook!(GM_801A4BD4, gm_801A4BD4, ());
    hook!(GM_801A4B88, gm_801A4B88, (info: *mut GameSceneInfo));
    hook!(GM_801A4D34, gm_801A4D34, (on_frame: SceneFn, info: *mut GameSceneInfo));
    hook!(GM_GETALLGAMEMODES, gm_GetAllGameModes, () -> *mut GameMode);
    hook!(GMMAINLIB_8015FBA4, gmMainLib_8015FBA4, ());
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
    hook!(GM_ONEPINTERSTAGE, gm_OnePInterstage, ());
    hook!(GMFRONTEND_ROUTE, gmFrontend_Route, (from: u8, to: u8) -> u8);
    hook!(GMFRONTEND_REPORTEDMODE, gmFrontend_ReportedMode, () -> u8);
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
pub fn gm_GetCurrentGameMode() -> u8 {
    unsafe { state_machine.routing.curr_mode }
}

/// `gm_GetPreviousGameMode()`.
pub fn gm_GetPreviousGameMode() -> u8 {
    unsafe { state_machine.routing.prev_mode }
}

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
