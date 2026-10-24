//! src/melee/gm/gmboot.c: the boot scene (memory-card prompt, skip-intro check, Pikmin unlock, scene
//! launch). `bootOnLoad`, `bootOnLeave`, `memcardOnLoad` are checked bit for bit against the C by a
//! state oracle (tests/tu_gmboot.rs over gen/turef/tu_gmboot_ref.c). The state tables
//! (`gm_Mode_Boot_States`, `gm_Mode_MemCard_States`) are data consumed by the mode registry and are
//! not ported here.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::{GameModeState, GM_OPENING_MV, GM_TITLE};

/// `enum { TROPHY_PIKMIN = 275 }` (gmboot.c, "move to toy header").
pub const TROPHY_PIKMIN: i32 = 275;
/// `LANG_JP` (lb/lblanguage.h): the first member of the language enum.
pub const LANG_JP: i32 = 0;

/// `struct loadData` (gmboot.c): the boot scene's enter data.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct loadData {
    pub x0: u32,
    pub x4: u8,
    /// Copied to `exitData::mode_id` to set the next mode.
    pub mode_id: u8,
}

/// `struct exitData` (gmboot.c): the boot scene's exit data.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct exitData {
    pub x0: u32,
    pub mode_id: u8,
}

/// `static struct loadData load_data;` / `static struct exitData leave_data;`
pub static mut load_data: loadData = loadData { x0: 0, x4: 0, mode_id: 0 };
pub static mut leave_data: exitData = exitData { x0: 0, mode_id: 0 };

/// Calls into TUs that are not ported yet (gm_1A3F, gmevent, toy, lb, PC shims). Each is a settable
/// hook; an unset hook panics. The two scene accessors fall back to the field they read.
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
    /// `gmMainLib_8046B0F0.skip_intro` (gm/gmmain_lib.c).
    hook!(GM_MAIN_LIB_SKIP_INTRO, gm_main_lib_skip_intro, () -> bool);
    /// `gm_801BF708(s8)` (gm/gmevent.c).
    pub static mut GM_801BF708: Option<fn(i8) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF708(arg: i8)  {
                match unsafe { GM_801BF708 } {
                    Some(f) => f(arg),
                    None => { unsafe { let _ = crate::tu::gmopeningmode::gm_801BF708(arg as _); } return; },
                }
            }
    /// `Toy_803048C0(id)` (ty/toy.c; s32 in the C, bool-valued).
    pub static mut TOY_803048C0: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn Toy_803048C0(id: i32) -> i32 {
                match unsafe { TOY_803048C0 } {
                    Some(f) => f(id),
                    None => { return unsafe { melee_items::tu::toy::Toy_803048C0(id as _) as _ }; },
                }
            }
    /// `Toy_803124BC()` (ty/toy.c).
    pub static mut TOY_803124BC: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn Toy_803124BC()  {
                match unsafe { TOY_803124BC } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_803124BC(); } return; },
                }
            }
    /// `Toy_SetUnlockState(id, value)` (ty/toy.c).
    pub static mut TOY_SETUNLOCKSTATE: Option<fn(i32, bool) > = None;
            #[allow(non_snake_case)]
            pub fn Toy_SetUnlockState(id: i32, value: bool)  {
                match unsafe { TOY_SETUNLOCKSTATE } {
                    Some(f) => f(id, value),
                    None => { unsafe { let _ = melee_items::tu::toy::Toy_SetUnlockState(id as _, value as _); } return; },
                }
            }
    /// `lb_8001C2D8(chan, company, game_name, filename)` (lb/lbcardnew.c; 0 on success).
    hook!(LB_8001C2D8, lb_8001C2D8, (chan: i32, company: &str, game_name: &str, filename: &str) -> i32);
    /// `lbLang_GetLanguageSetting()` (lb/lblanguage.c).
    pub static mut LBLANG_GETLANGUAGESETTING: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbLang_GetLanguageSetting() -> i32 {
                match unsafe { LBLANG_GETLANGUAGESETTING } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_GetLanguageSetting() as _ }; },
                }
            }
    /// `gm_ChangeGameModeAfterCurrentScene(mode)` (gm/gm_1A3F.c).
    pub static mut GM_CHANGEGAMEMODEAFTERCURRENTSCENE: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_ChangeGameModeAfterCurrentScene(mode: i32)  {
                match unsafe { GM_CHANGEGAMEMODEAFTERCURRENTSCENE } {
                    Some(f) => f(mode),
                    None => { unsafe { let _ = crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(mode as _); } return; },
                }
            }
    /// `gm_SetGameModeOverride(fn)` (gm/gm_1A3F.c).
    hook!(GM_SETGAMEMODEOVERRIDE, gm_SetGameModeOverride, (f: fn() -> u8));
    /// `gm_SetPendingGameMode(mode)` (gm/gm_1A3F.c).
    pub static mut GM_SETPENDINGGAMEMODE: Option<fn(u8)> = None;
    #[allow(non_snake_case)]
    pub fn gm_SetPendingGameMode(mode: u8) {
        match unsafe { GM_SETPENDINGGAMEMODE } {
            Some(f) => f(mode),
            None => { unsafe { let _ = crate::tu::gm_1A3F::gm_SetPendingGameMode(mode as _); } },
        }
    }
    /// `SceneLaunch_BootGameMode()` (gm/gmscenelaunch.c, TARGET_PC): -1 when unset.
    hook!(SCENELAUNCH_BOOTGAMEMODE, SceneLaunch_BootGameMode, () -> i32);
    /// `ContentProbeName()` (pc, TARGET_PC): the probe file name, or NULL.
    hook!(CONTENTPROBENAME, ContentProbeName, () -> Option<&'static str>);
    /// `ContentProbeResult(name, archive)` (pc, TARGET_PC).
    hook!(CONTENTPROBERESULT, ContentProbeResult, (name: &str, archive: *mut c_void));
    /// `lbArchive_LoadArchive(filename)` (lb/lbarchive, TARGET_PC).
    hook!(LBARCHIVE_LOADARCHIVE, lbArchive_LoadArchive, (filename: &str) -> *mut c_void);

    /// `lbCardGame_DecideGameMode` (lb/lbcardgame.c): the override's identity; not ported yet.
    pub fn lbCardGame_DecideGameMode() -> u8 {
        { return unsafe { melee_runtime::tu::lbcardgame::lbCardGame_DecideGameMode() as _ }; }
    }
}

/// `bootOnLoad` (gmboot.c): clear the load data; skip the intro to the title, or play it.
pub fn bootOnLoad(scene: *mut GameModeState) {
    unsafe {
        let scene_data = ext::gm_GetGameModeStateEnterData(scene) as *mut loadData;
        (*scene_data).x4 = 0;
        (*scene_data).x0 = 0;
        if ext::gm_main_lib_skip_intro() {
            (*scene_data).mode_id = GM_TITLE as u8;
        } else {
            ext::gm_801BF708(0);
            (*scene_data).mode_id = GM_OPENING_MV as u8;
        }
    }
}

/// `bootOnLeave` (gmboot.c): the Pikmin trophy check, the mode override and the next mode, then the
/// TARGET_PC scene launch and content probe.
pub fn bootOnLeave(data: *mut GameModeState) {
    unsafe {
        let scene_data = ext::gm_GetGameModeStateExitData(data) as *mut exitData;

        if ext::Toy_803048C0(TROPHY_PIKMIN) == 0 {
            let game = if ext::lbLang_GetLanguageSetting() == LANG_JP { "GPIJ" } else { "GPIE" };
            if ext::lb_8001C2D8(0, "01", game, "Pikmin dataFile") == 0 {
                ext::Toy_803124BC();
                ext::Toy_SetUnlockState(TROPHY_PIKMIN, true);
            }
        }

        ext::gm_SetGameModeOverride(ext::lbCardGame_DecideGameMode);

        ext::gm_ChangeGameModeAfterCurrentScene((*scene_data).mode_id as i32);

        // TARGET_PC: scene launch (a set env var names the mode; -1 when none).
        let mode = ext::SceneLaunch_BootGameMode();
        if mode >= 0 {
            ext::gm_SetPendingGameMode(mode as u8);
        }
        // m-ex content proof of life: load the named file and report whether it parsed.
        if let Some(probe) = ext::ContentProbeName() {
            ext::ContentProbeResult(probe, ext::lbArchive_LoadArchive(probe));
        }
    }
}

/// `memcardOnLoad` (gmboot.c, the memory-card mode): record that the prompt is the next step.
pub fn memcardOnLoad(scene: *mut GameModeState) {
    unsafe {
        let temp_r3 = ext::gm_GetGameModeStateEnterData(scene) as *mut loadData;
        (*temp_r3).x4 = 0;
        (*temp_r3).x0 = 1;
    }
}

/// `GM_OPENING_MV` / `GM_TITLE` as the bytes the boot scene stores.
pub const BOOT_MODE_TITLE: u8 = GM_TITLE as u8;
pub const BOOT_MODE_OPENING_MV: u8 = GM_OPENING_MV as u8;
