//! src/melee/gm/gmscmemcard.c: the memory-card prompt scene ("There is no save data. Create one?" and its
//! kin): a state machine over `enter_data.decision` (21 prompts), driven by the merged controller triggers, with
//! the notice window of gm_1ADD for the text. All 6 functions (plus the `static inline` helpers) are checked
//! against the C by a state oracle (tests/tu_gmscmemcard.rs over gen/turef/tu_gmscmemcard_ref.c): the game's
//! gmscmemcard.c compiled natively (shipping contraction) with the callees stubbed and logged; the callee log
//! and `enter_data` are compared after every call. The TARGET_PC additions (scene report, scene-launch and m-ex
//! prompt skip, settings auto-create) are part of the port and are ported too.
//!
//! The callees that live in gm_1ADD (ported in this crate) default to the real functions when no hook is set.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

/// `exitData` (gmscmemcard.c), GameCube size 8 (`bool` is `int` in this codebase, MSL/stdbool.h).
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct exitData {
    pub unk0: i32,
    pub mode_id: u8,
}
melee_core::gc_layout!(exitData, "exitData", size 8, align 4, [
    unk0 @ 0x0: 4 "bool",
    mode_id @ 0x4: 1 "u8",
]);

/// `struct enterData_x0_t` (gmscmemcard.c), GameCube size 8.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct enterData_x0_t {
    /// `::LbLanguage`?
    pub unk0: i32,
    /// Memory card channel.
    pub chan: u8,
    pub mode_id: u8,
}
melee_core::gc_layout!(enterData_x0_t, "enterData_x0_t", size 8, align 4, [
    unk0 @ 0x0: 4 "int",
    chan @ 0x4: 1 "u8",
    mode_id @ 0x5: 1 "u8",
]);

/// `enterData` (gmscmemcard.c), GameCube size 0x20.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct enterData {
    pub unk0: enterData_x0_t,
    pub unk8: exitData,
    pub unk10: i32,
    /// `tickDecision`: the prompt (0..=20).
    pub decision: i32,
    pub unk18: i32,
    pub unk1C: u8,
}
melee_core::gc_layout!(enterData, "enterData", size 0x20, align 4, [
    unk0 @ 0x0: 8 "struct enterData_x0_t",
    unk8 @ 0x8: 8 "exitData",
    unk10 @ 0x10: 4 "int",
    decision @ 0x14: 4 "tickDecision",
    unk18 @ 0x18: 4 "bool",
    unk1C @ 0x1C: 1 "u8",
]);

pub const tickDecision_1: i32 = 1;
pub const tickDecision_20: i32 = 20;
/// `LbCardStatus_0` / `LbCardStatus_4` (lb/lbcardgame.h).
pub const LbCardStatus_0: i32 = 0;
pub const LbCardStatus_4: i32 = 4;
/// `DbLKind_DebugRom` (db/db.h).
pub const DbLKind_DebugRom: i32 = 3;
pub const HSD_PAD_R: u32 = 1 << 5;
pub const HSD_PAD_L: u32 = 1 << 6;
pub const HSD_PAD_A: u32 = 1 << 8;
pub const HSD_PAD_B: u32 = 1 << 9;
pub const HSD_PAD_START: u32 = 1 << 12;

pub static mut gm_804D6870: u8 = 0;
pub static mut gm_804D6872: u16 = 0;

/// `static int gm_803DD550_jp[]` / `gm_803DD550_us[]`: per-prompt notice-window row counts.
pub static gm_803DD550_jp: [i32; 23] = [1, 1, 1, 1, 2, 2, 3, 1, 3, 2, 4, 1, 4, 4, 5, 1, 2, 4, 2, 3, 2, 2, 2];
pub static gm_803DD550_us: [i32; 23] = [1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 5, 2, 5, 2, 3, 2, 2, 4, 2, 3, 2, 1, 2];

/// `static enterData enter_data;`
pub static mut enter_data: enterData =
    enterData { unk0: enterData_x0_t { unk0: 0, chan: 0, mode_id: 0 }, unk8: exitData { unk0: 0, mode_id: 0 }, unk10: 0, decision: 0, unk18: 0, unk1C: 0 };

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics, except the gm_1ADD
/// functions, which default to this crate's port.
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
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// A hook that falls back to the ported function `$real`.
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $real:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => unsafe { $real($($a),*) },
                }
            }
        };
    }
    // gm/gm_1ADD.c (ported)
    hook_or!(GM_801AEDC8, gm_801AEDC8, crate::tu::gm_1ADD::gm_801AEDC8, () -> u32);
    hook_or!(GM_801AE848, gm_801AE848, crate::tu::gm_1ADD::gm_801AE848, (arg0: i32));
    hook_or!(GM_801ADE1C, gm_801ADE1C, crate::tu::gm_1ADD::gm_801ADE1C, (arg0: i32, arg1: i32, arg2: f32, arg3: f32) -> bool);
    hook_or!(GM_801AE44C, gm_801AE44C, crate::tu::gm_1ADD::gm_801AE44C, (arg0: i32, val: f32));
    hook_or!(GM_801AE544, gm_801AE544, crate::tu::gm_1ADD::gm_801AE544, (arg0: i32, val: f32));
    hook_or!(GM_801AE050, gm_801AE050, crate::tu::gm_1ADD::gm_801AE050, (a: i32, b: i32, c: i32, x: f32, y: f32) -> bool);
    hook_or!(GM_801AE640, gm_801AE640, crate::tu::gm_1ADD::gm_801AE640, (arg0: i32, arg1: bool));
    hook_or!(GM_801AE74C, gm_801AE74C, crate::tu::gm_1ADD::gm_801AE74C, (arg0: i32, arg1: bool));
    hook_or!(GM_801ADDD8, gm_801ADDD8, crate::tu::gm_1ADD::gm_801ADDD8, ());
    // lb/lbcardnew.c, lb/lbcardgame.c, lb/lblanguage.c
    pub static mut LBCARDNEW_PROBEEX: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbCardNew_ProbeEx(chan: i32) -> i32 {
                match unsafe { LBCARDNEW_PROBEEX } {
                    Some(f) => f(chan),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lbCardNew_ProbeEx(chan as _) as _ }; },
                }
            }
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
    pub static mut LBCARDGAME_SETCARDSTATUS: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbCardGame_SetCardStatus(status: i32)  {
                match unsafe { LBCARDGAME_SETCARDSTATUS } {
                    Some(f) => f(status),
                    None => { unsafe { melee_runtime::tu::lbcardgame::lbCardGame_SetCardStatus(status as _); } return; },
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
    pub static mut LB_8001C87C: Option<fn() -> u32> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001C87C() -> u32 {
                match unsafe { LB_8001C87C } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardgame::lb_8001C87C() as _ }; },
                }
            }
    pub static mut LB_8001CBBC: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001CBBC() -> i32 {
                match unsafe { LB_8001CBBC } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardgame::lb_8001CBBC() as _ }; },
                }
            }
    pub static mut LB_8001CC4C: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001CC4C() -> i32 {
                match unsafe { LB_8001CC4C } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardgame::lb_8001CC4C() as _ }; },
                }
            }
    pub static mut LB_8001C8BC: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001C8BC() -> i32 {
                match unsafe { LB_8001C8BC } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbcardgame::lb_8001C8BC() as _ }; },
                }
            }
    pub static mut LB_8001B8C8: Option<fn(i32) -> bool> = None;
            #[allow(non_snake_case)]
            pub fn lb_8001B8C8(chan: i32) -> bool {
                match unsafe { LB_8001B8C8 } {
                    Some(f) => f(chan),
                    None => { return unsafe { melee_runtime::tu::lbcardnew::lb_8001B8C8(chan as _) != 0 }; },
                }
            }
    pub static mut LBLANG_ISSAVEDLANGUAGEUS: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn lbLang_IsSavedLanguageUS() -> bool {
                match unsafe { LBLANG_ISSAVEDLANGUAGEUS } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ }; },
                }
            }
    // gm/gmmain_lib.c, gm/gmscene.c
    pub static mut GMMAINLIB_8015FA34: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gmMainLib_8015FA34(v: i32)  {
                match unsafe { GMMAINLIB_8015FA34 } {
                    Some(f) => f(v),
                    None => { unsafe { crate::tu::gmmain_lib::gmMainLib_8015FA34(v as _); } return; },
                }
            }
    pub static mut GM_801A4B60: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A4B60()  {
                match unsafe { GM_801A4B60 } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmscene::gm_801A4B60(); } return; },
                }
            }
    // lb/lbaudio_ax.c (the sfx inlines of mn/inlines.h), db/db.c, sysdolphin/baselib/controller.c
    pub static mut LBAUDIOAX_80024030: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80024030(id: i32)  {
                match unsafe { LBAUDIOAX_80024030 } {
                    Some(f) => f(id),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(id as _); } return; },
                }
            }
    hook!(DBLEVEL, DbLevel, () -> i32);
    /// `HSD_PadCopyStatus[0].button` / `.trigger`.
    hook!(HSD_PADCOPYSTATUS0_BUTTON, hsd_pad_copy_status0_button, () -> u32);
    hook!(HSD_PADCOPYSTATUS0_TRIGGER, hsd_pad_copy_status0_trigger, () -> u32);
    // TARGET_PC: pc/platform (scene launch, scene report, m-ex, settings)
    hook!(SCENEREPORT_MEMCARD, SceneReport_Memcard, (decision: i32, option: i32));
    hook!(SCENEREPORT_CURSOR, SceneReport_Cursor, (what: &str, a: i32, b: i32));
    hook!(SCENELAUNCH_SKIPMEMCARD, SceneLaunch_SkipMemcard, () -> i32);
    hook!(MEX_ENABLED, Mex_Enabled, (name: &str) -> i32);
    hook!(SETTINGS_INT, Settings_Int, (key: &str, dflt: i32) -> i32);
}

/// `sfxForward()` / `sfxMove()` (mn/inlines.h, static inline).
#[inline(always)]
fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
#[inline(always)]
fn sfxMove() {
    ext::lbAudioAx_80024030(2);
}

/// `get_lang_val(idx)`: the notice window's row count for prompt `idx` in the saved language.
fn get_lang_val(idx: i32) -> i32 {
    let i = (idx - 2) as usize;
    if ext::lbLang_IsSavedLanguageUS() {
        gm_803DD550_us[i]
    } else {
        gm_803DD550_jp[i]
    }
}

/// `gm_801AEE6C(arg0, arg1, arg2)`: rebuild the notice window: kind 0 = text only, 1 = two buttons (Yes/No),
/// 2 = two buttons (other labels); `arg1` is the message id and `arg2` its row count.
pub fn gm_801AEE6C(arg0: i32, arg1: i32, arg2: i32) {
    melee_core::ct!("gm_801AEE6C");
    let scale: f32 = 1.12;
    unsafe {
        match arg0 {
            0 => {
                ext::gm_801AE848(0);
                ext::gm_801ADE1C(0, arg1, 0.0, 0.0);
                ext::gm_801AE44C(0, scale * ((arg2 - 2) as f32));
                ext::gm_801AE544(0, -2.0);
                enter_data.unk10 = 0;
            }
            1 => {
                ext::gm_801AE848(0);
                ext::gm_801ADE1C(0, arg1, 0.0, 0.0);
                ext::gm_801AE44C(0, scale * ((arg2 - 2) as f32));
                ext::gm_801AE050(0, 0, 3, -3.5, -scale * ((arg2 - 2) as f32));
                ext::gm_801AE050(0, 1, 4, 3.5, -scale * ((arg2 - 2) as f32));
                enter_data.unk1C = 0;
                ext::gm_801AE640(0, enter_data.unk1C != 0);
                ext::gm_801AE74C(0, enter_data.unk1C == 0);
                enter_data.unk10 = 1;
            }
            2 => {
                ext::gm_801AE848(0);
                ext::gm_801ADE1C(0, arg1, 0.0, 0.0);
                ext::gm_801AE44C(0, scale * ((arg2 - 2) as f32));
                ext::gm_801AE050(0, 0, 2, -3.5, -scale * ((arg2 - 2) as f32));
                ext::gm_801AE050(0, 1, 5, 3.5, -scale * ((arg2 - 2) as f32));
                enter_data.unk1C = 0;
                ext::gm_801AE640(0, enter_data.unk1C != 0);
                ext::gm_801AE74C(0, enter_data.unk1C == 0);
                enter_data.unk10 = 1;
            }
            _ => {}
        }
    }
}

/// `gm_801AF0D4_inline()`: a card inserted/removed since the last prompt sends the scene back to decision 1.
unsafe fn gm_801AF0D4_inline() -> bool {
    if ext::lbCardNew_ProbeEx(enter_data.unk0.chan as i32) != 0 {
        if enter_data.unk18 == 1 {
            enter_data.unk18 = 0;
            enter_data.decision = 1;
            return true;
        }
    } else if enter_data.unk18 == 0 {
        enter_data.unk18 = 1;
        enter_data.decision = 1;
        return true;
    }
    false
}

/// `gm_801AF0D4()`: the Yes/No cursor (left/up = 0, right/down = 1); true when the card status changed.
pub fn gm_801AF0D4() -> bool {
    melee_core::ct!("gm_801AF0D4");
    unsafe {
        let saved_unk1C = enter_data.unk1C as i32;

        if gm_801AF0D4_inline() {
            return true;
        }

        if ext::gm_801AEDC8() & 0x40001 != 0 {
            if enter_data.unk1C != 0 {
                if enter_data.unk10 != 0 {
                    sfxMove();
                }
                enter_data.unk1C = 0;
            }
        } else if ext::gm_801AEDC8() & 0x80002 != 0 {
            if enter_data.unk1C < 1 {
                if enter_data.unk10 != 0 {
                    sfxMove();
                }
                enter_data.unk1C = 1;
            }
        }
        if saved_unk1C != enter_data.unk1C as i32 {
            ext::gm_801AE640(0, enter_data.unk1C != 0);
            ext::gm_801AE74C(0, enter_data.unk1C == 0);
        }
        false
    }
}

/// `set_gm_804D6870_inline()`: L+R+A on pad 1 latches the debug flag.
unsafe fn set_gm_804D6870_inline() -> u8 {
    if (ext::hsd_pad_copy_status0_button() & HSD_PAD_L) != 0
        && (ext::hsd_pad_copy_status0_button() & HSD_PAD_R) != 0
        && (ext::hsd_pad_copy_status0_button() & HSD_PAD_A) != 0
    {
        gm_804D6870 = 1;
    }
    gm_804D6870
}

/// `gm_801AEDC8_flag_check()`: START or A on any pad (plays the forward sound).
fn gm_801AEDC8_flag_check() -> bool {
    if ext::gm_801AEDC8() & (HSD_PAD_START | HSD_PAD_A) != 0 {
        sfxForward();
        return true;
    }
    false
}

/// `unk_inline()`: the "disable saving" outcome prompts (18 for the first-boot flow, 19 otherwise).
unsafe fn unk_inline() {
    if enter_data.unk0.unk0 == 1 {
        gm_801AEE6C(2, 23, get_lang_val(23));
        enter_data.decision = 18;
    } else {
        gm_801AEE6C(2, 24, get_lang_val(24));
        enter_data.decision = 19;
    }
}

/// `gm_801AF250()`: show the prompt for the card status `lb_8001C87C()` and set the next decision.
pub fn gm_801AF250() {
    melee_core::ct!("gm_801AF250");
    unsafe {
        let temp_r3 = ext::lb_8001C87C();
        enter_data.unk18 = 1;
        match temp_r3 {
            1 | 2 => {
                if enter_data.unk0.unk0 == 0 {
                    gm_801AEE6C(1, 7, get_lang_val(6));
                } else {
                    gm_801AEE6C(1, 6, get_lang_val(6));
                    enter_data.unk1C = 1;
                    ext::gm_801AE640(0, enter_data.unk1C != 0);
                    ext::gm_801AE74C(0, enter_data.unk1C == 0);
                }
                enter_data.decision = 2;
            }
            3 => {
                gm_801AEE6C(1, 8, get_lang_val(8));
                enter_data.decision = 3;
                enter_data.unk1C = 1;
                ext::gm_801AE640(0, enter_data.unk1C != 0);
                ext::gm_801AE74C(0, enter_data.unk1C == 0);
            }
            4 => {
                gm_801AEE6C(1, 0xA, get_lang_val(0xA));
                enter_data.decision = 5;
            }
            5 => {
                gm_801AEE6C(0, 0xE, get_lang_val(0xE));
                enter_data.decision = 9;
            }
            6 => {
                gm_801AEE6C(0, 0xF, get_lang_val(0xF));
                enter_data.decision = 0xA;
            }
            9 => {
                gm_801AEE6C(1, 0x10, get_lang_val(0x10));
                enter_data.decision = 0xB;
                enter_data.unk1C = 1;
                ext::gm_801AE640(0, enter_data.unk1C != 0);
                ext::gm_801AE74C(0, enter_data.unk1C == 0);
            }
            10 | 11 | 13 => {
                gm_801AEE6C(0, 0x13, get_lang_val(0x13));
                enter_data.decision = 0xE;
            }
            12 => {
                gm_801AEE6C(0, 0x14, get_lang_val(0x14));
                enter_data.decision = 0xF;
            }
            14 => {
                gm_801AEE6C(0, 0x15, get_lang_val(0x15));
                enter_data.decision = 0x10;
            }
            15 => {
                gm_801AEE6C(0, 0x16, get_lang_val(0x16));
                enter_data.unk18 = 0;
                enter_data.decision = 0x11;
            }
            _ => {
                ext::gm_801A4B60();
            }
        }
    }
}

/// `gm_Scene_MemCard_OnFrame()`: one tick of the prompt state machine.
pub fn gm_Scene_MemCard_OnFrame() {
    melee_core::ct!("gm_Scene_MemCard_OnFrame");
    unsafe {
        if ext::DbLevel() >= DbLKind_DebugRom && set_gm_804D6870_inline() != 0 {
            if ext::hsd_pad_copy_status0_trigger() & HSD_PAD_L != 0 {
                if gm_804D6872 > 6 {
                    gm_804D6872 -= 1;
                    gm_801AEE6C(0, gm_804D6872 as i32, get_lang_val(gm_804D6872 as i32));
                }
            } else if ext::hsd_pad_copy_status0_trigger() & HSD_PAD_R != 0 {
                if gm_804D6872 < 0x18 {
                    gm_804D6872 += 1;
                    gm_801AEE6C(0, gm_804D6872 as i32, get_lang_val(gm_804D6872 as i32));
                }
            }
            if (ext::hsd_pad_copy_status0_button() & HSD_PAD_L) != 0
                && (ext::hsd_pad_copy_status0_button() & HSD_PAD_R) != 0
                && (ext::hsd_pad_copy_status0_button() & HSD_PAD_B) != 0
            {
                ext::gm_801A4B60();
            }
            return;
        }

        // TARGET_PC
        ext::SceneReport_Memcard(enter_data.decision, enter_data.unk1C as i32);

        match enter_data.decision {
            0 => {
                let temp_r29 = ext::lb_8001CBBC();
                ext::gmMainLib_8015FA34(temp_r29);
                // TARGET_PC: the boot blocker (scene launch / m-ex skip_memcard_prompt).
                if ext::SceneLaunch_SkipMemcard() != 0
                    || (ext::Mex_Enabled("skip_memcard_prompt") != 0 && (temp_r29 == 0xF || temp_r29 == 0xD))
                {
                    enter_data.unk8.unk0 = 0;
                    enter_data.decision = tickDecision_20;
                    ext::lbCardGame_SetCardStatus(4);
                    ext::SceneReport_Cursor("memcard-skipped", temp_r29, 0);
                    return;
                }
                if temp_r29 == 0 || temp_r29 == 2 {
                    enter_data.unk8.unk0 = 1;
                    enter_data.decision = 0x14;
                } else {
                    enter_data.decision = 1;
                }
            }
            1 => gm_801AF250(),
            2 => {
                // TARGET_PC: the port's card is a folder it made itself; take the prompt's own Yes.
                if enter_data.unk0.unk0 != 0 && ext::Settings_Int("auto_create_save", 1) != 0 {
                    ext::SceneReport_Cursor("memcard-autocreate", enter_data.decision, 0);
                    enter_data.unk8.unk0 = 1;
                    enter_data.decision = 20;
                    ext::lbCardGame_SetCardStatus(LbCardStatus_0);
                    ext::lbCardGame_SaveChanges();
                    return;
                }
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    if enter_data.unk1C == 0 {
                        if enter_data.unk0.unk0 == 0 {
                            enter_data.decision = 0;
                            ext::lbCardGame_SetCardStatus(LbCardStatus_0);
                        } else {
                            enter_data.unk8.unk0 = 1;
                            enter_data.decision = 20;
                            ext::lbCardGame_SetCardStatus(LbCardStatus_0);
                            ext::lbCardGame_SaveChanges();
                        }
                    } else {
                        unk_inline();
                    }
                }
            }
            3 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    if enter_data.unk1C == 0 {
                        gm_801AEE6C(0, 9, get_lang_val(9));
                        enter_data.decision = 4;
                    } else {
                        unk_inline();
                    }
                }
            }
            4 => {
                if !gm_801AF0D4() {
                    if ext::lb_8001CC4C() == 0 {
                        gm_801AEE6C(0, 0xB, get_lang_val(0xB));
                        enter_data.decision = 6;
                    } else {
                        gm_801AEE6C(0, 0xD, get_lang_val(0xD));
                        enter_data.decision = 8;
                    }
                }
            }
            5 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    if enter_data.unk1C == 0 {
                        gm_801AEE6C(0, 0xB, get_lang_val(0xB));
                        enter_data.decision = 6;
                    } else {
                        unk_inline();
                    }
                }
            }
            6 => {
                if !gm_801AF0D4() {
                    if ext::lb_8001C8BC() == 0 {
                        gm_801AEE6C(0, 0xC, get_lang_val(0xC));
                        enter_data.decision = 7;
                    } else {
                        gm_801AEE6C(0, 0xD, get_lang_val(0xD));
                        enter_data.decision = 8;
                    }
                }
            }
            7 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    enter_data.unk8.unk0 = 1;
                    enter_data.decision = 0x14;
                    ext::lbCardGame_SetCardStatus(0);
                }
            }
            8 | 9 | 10 | 13 | 14 | 15 | 16 | 17 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    unk_inline();
                }
            }
            11 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    if enter_data.unk1C == 0 {
                        if !ext::lb_8001B8C8(enter_data.unk0.chan as i32) {
                            gm_801AEE6C(0, 0x11, get_lang_val(0x11));
                            enter_data.decision = 0xC;
                        } else {
                            gm_801AEE6C(0, 0x12, get_lang_val(0x12));
                            enter_data.decision = 0xD;
                        }
                    } else {
                        unk_inline();
                    }
                }
            }
            12 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    enter_data.decision = 1;
                }
            }
            18 | 19 => {
                if !gm_801AF0D4() && gm_801AEDC8_flag_check() {
                    if enter_data.unk1C == 0 {
                        enter_data.unk8.unk0 = 0;
                        enter_data.decision = tickDecision_20;
                        ext::lbCardGame_SetCardStatus(LbCardStatus_4);
                    } else {
                        enter_data.decision = tickDecision_1;
                    }
                }
            }
            _ => {
                // 20 and anything else
                ext::gm_801A4B60();
            }
        }
    }
}

/// `checkUnk0()`: 0 when `unk0.unk0` is 0, else 1.
unsafe fn checkUnk0() -> i32 {
    if enter_data.unk0.unk0 == 0 {
        return 0;
    }
    1
}

/// `gm_Scene_MemCard_OnEnter(user_data)`: take the caller's enter data, pick the first prompt, load the card
/// work area and the notice window.
pub fn gm_Scene_MemCard_OnEnter(user_data: *mut c_void) {
    melee_core::ct!("gm_Scene_MemCard_OnEnter");
    unsafe {
        let data = user_data as *mut enterData;

        enter_data = enterData::default();
        if !data.is_null() {
            enter_data.unk0 = (*data).unk0;
        }
        enter_data.decision = checkUnk0();
        enter_data.unk8.mode_id = enter_data.unk0.mode_id;
        ext::lbCardNew_AllocWorkArea();
        ext::lbCardGame_LoadArchive(0);
        ext::gm_801ADDD8();
        gm_804D6870 = 0;
        gm_804D6872 = 6;
    }
}

/// `gm_Scene_MemCard_OnExit(user_data)`: hand the result back to the caller and tear down the window.
pub fn gm_Scene_MemCard_OnExit(user_data: *mut c_void) {
    melee_core::ct!("gm_Scene_MemCard_OnExit");
    unsafe {
        let data = user_data as *mut exitData;
        if !data.is_null() {
            *data = enter_data.unk8;
        }
        ext::gm_801AE848(0);
    }
}
