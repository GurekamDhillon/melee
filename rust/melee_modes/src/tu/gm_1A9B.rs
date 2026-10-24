//! src/melee/gm/gm_1A9B.c: the ending ("Congratulations") THP playback scene of the 1P modes: `OnEnter` picks the
//! movie by game-over mode and character, `OnFrame` waits out the input delay and ends the scene on A/Start.
//! Both functions are checked against the C by a state oracle (tests/tu_gm_1A9B.rs over
//! gen/turef/tu_gm_1A9B_ref.c): the game's gm_1A9B.c compiled natively (shipping contraction) with the
//! callees stubbed and logged; the three movie tables are compared through the file name logged for every
//! (mode, character) pair.
//!
//! `gm_GetButtonsTriggered` (gm_1A36.c, ported) defaults to the real function when no hook is set.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

pub const PAD_BUTTON_A: u64 = 1 << 8;
pub const PAD_BUTTON_START: u64 = 1 << 12;

/// `static u8 gmRegend_InputDelayTimer;` / `static u8 gmRegend_ExitTimer;`
pub static mut gmRegend_InputDelayTimer: u8 = 0;
pub static mut gmRegend_ExitTimer: u8 = 0;

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics, except
/// `gm_GetButtonsTriggered`, which defaults to this crate's port.
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
    pub static mut GM_GETBUTTONSTRIGGERED: Option<fn(u8) -> u64> = None;
    /// `gm_GetButtonsTriggered(idx)` (gm/gm_1A36.c, ported).
    pub fn gm_GetButtonsTriggered(idx: u8) -> u64 {
        match unsafe { GM_GETBUTTONSTRIGGERED } {
            Some(f) => f(idx),
            None => crate::tu::gm_1A36::gm_GetButtonsTriggered(idx),
        }
    }
    // sysdolphin/baselib/gobj.c, gobjgxlink.c, gobjobject.c, sobjlib.c
    hook!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj);
    pub static mut HSD_SOBJLIB_803A55DC: Option<fn(*mut HSD_GObj, u16, u16, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SObjLib_803A55DC(g: *mut HSD_GObj, w: u16, h: u16, n: i32)  {
                match unsafe { HSD_SOBJLIB_803A55DC } {
                    Some(f) => f(g, w, h, n),
                    None => { unsafe { let _ = melee_runtime::tu::sobjlib::HSD_SObjLib_803A55DC(g as _, w as _, h as _, n as _); } return; },
                }
            }
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void));
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render: usize, gx_link: u8, priority: u32));
    /// `HSD_SObjLib_804D7960` (sysdolphin/baselib/sobjlib.c): the SObj object kind.
    hook!(HSD_SOBJLIB_804D7960, hsd_sobjlib_804d7960, () -> u8);
    // lb/lbaudio_ax.c, lb/lbbgflash.c, lb/lbmthp.c
    pub static mut LBAUDIOAX_80026F2C: Option<fn(u32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80026F2C(v: u32)  {
                match unsafe { LBAUDIOAX_80026F2C } {
                    Some(f) => f(v),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80026F2C(v as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_8002702C: Option<fn(u32, u64) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_8002702C(flags: u32, mask: u64)  {
                match unsafe { LBAUDIOAX_8002702C } {
                    Some(f) => f(flags, mask),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_8002702C(flags as _, mask as _); } return; },
                }
            }
    pub static mut LBAUDIOAX_80027168: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80027168()  {
                match unsafe { LBAUDIOAX_80027168 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80027168(); } return; },
                }
            }
    pub static mut LBAUDIOAX_80027648: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80027648()  {
                match unsafe { LBAUDIOAX_80027648 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80027648(); } return; },
                }
            }
    pub static mut LBAUDIOAX_80023694: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80023694() -> i32 {
                match unsafe { LBAUDIOAX_80023694 } {
                    Some(f) => f(),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694() as _ }; },
                }
            }
    pub static mut LBAUDIOAX_800237A8: Option<fn(i32, i32, i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_800237A8(sfx_id: i32, vol: i32, pan: i32) -> i32 {
                match unsafe { LBAUDIOAX_800237A8 } {
                    Some(f) => f(sfx_id, vol, pan),
                    None => { return unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(sfx_id as _, vol as _, pan as _) as _ }; },
                }
            }
    pub static mut LBAUDIOAX_80024030: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80024030(id: i32)  {
                match unsafe { LBAUDIOAX_80024030 } {
                    Some(f) => f(id),
                    None => { unsafe { let _ = melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(id as _); } return; },
                }
            }
    pub static mut LBBGFLASH_800209F4: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn lbBgFlash_800209F4()  {
                match unsafe { LBBGFLASH_800209F4 } {
                    Some(f) => f(),
                    None => { unsafe { let _ = melee_runtime::tu::lbbgflash::lbBgFlash_800209F4(); } return; },
                }
            }
    pub static mut LBBGFLASH_8002063C: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbBgFlash_8002063C(count: i32)  {
                match unsafe { LBBGFLASH_8002063C } {
                    Some(f) => f(count),
                    None => { unsafe { let _ = melee_runtime::tu::lbbgflash::lbBgFlash_8002063C(count as _); } return; },
                }
            }
    pub static mut LBMTHP_8001F890: Option<fn(*mut HSD_GObj) -> *mut HSD_SObj> = None;
            #[allow(non_snake_case)]
            pub fn lbMthp8001F890(g: *mut HSD_GObj) -> *mut HSD_SObj {
                match unsafe { LBMTHP_8001F890 } {
                    Some(f) => f(g),
                    None => { return unsafe { crate::tu::lb_01F8::lbMthp8001F890(g as _) as _ }; },
                }
            }
    hook!(LBMTHP_8001FAA0, lbMthp8001FAA0, (filename: &str, w: i32, h: i32));
    // gm/gmevent.c, gm/gm_1A3F.c, gm/gmregclear.c, gm/gmregtyfall.c, gm/gm_1601.c (TARGET_PC)
    pub static mut GM_801BEFB0: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801BEFB0() -> i32 {
                match unsafe { GM_801BEFB0 } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmgover::gm_801BEFB0() as _ }; },
                }
            }
    pub static mut GM_801BF010: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF010() -> i32 {
                match unsafe { GM_801BF010 } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmgover::gm_801BF010() as _ }; },
                }
            }
    pub static mut GM_801BF050: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801BF050() -> i32 {
                match unsafe { GM_801BF050 } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmgover::gm_801BF050() as _ }; },
                }
            }
    pub static mut GM_GETCURRENTGAMEMODE: Option<fn() -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetCurrentGameMode() -> u8 {
                match unsafe { GM_GETCURRENTGAMEMODE } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gm_1A3F::gm_GetCurrentGameMode() as _ }; },
                }
            }
    pub static mut GM_8017DFF4: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_8017DFF4(v: i32) -> i32 {
                match unsafe { GM_8017DFF4 } {
                    Some(f) => f(v),
                    None => { return unsafe { crate::tu::gm_17c0::gm_8017DFF4(v as _) as _ }; },
                }
            }
    pub static mut GM_801A6630: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A6630(v: i32)  {
                match unsafe { GM_801A6630 } {
                    Some(f) => f(v),
                    None => { unsafe { let _ = crate::tu::gmregtyfall::gm_801A6630(v as _); } return; },
                }
            }
    pub static mut GM_MEXVANILLAKIND: Option<fn(i32, i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_MexVanillaKind(kind: i32, character_kind: i32) -> i32 {
                match unsafe { GM_MEXVANILLAKIND } {
                    Some(f) => f(kind, character_kind),
                    None => { return unsafe { crate::tu::gm_1601::gm_MexVanillaKind(kind as _, character_kind as _) as _ }; },
                }
            }

    /// `lbMthp8001F928` (lb/lbmthp.c): the GX-link render callback of the movie; address only.
    pub unsafe extern "C" fn lbMthp8001F928(_gobj: *mut HSD_GObj, _code: i32) {
        { unsafe { let _ = crate::tu::lb_01F8::lbMthp8001F928(_gobj as _, _code as _); } return; };
    }
}

/// `sfxForward()` (mn/inlines.h, static inline).
#[inline(always)]
fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}

/// `gm_Scene_Congrats_OnEnter(unused)`: start the ending movie for the current mode and character.
pub fn gm_Scene_Congrats_OnEnter(_unused: *mut c_void) {
    unsafe {
        gmRegend_InputDelayTimer = 0x1E;
        gmRegend_ExitTimer = 0;
        let sobj_gobj = ext::GObj_Create(0x13, 0x14, 0);
        ext::HSD_SObjLib_803A55DC(sobj_gobj, 0x280, 0x1E0, 8);
        (*sobj_gobj).gxlink_prios = 0x800;
        ext::lbAudioAx_80026F2C(0x12);
        ext::lbAudioAx_8002702C(2, 0x10);
        ext::lbAudioAx_80027168();
        ext::lbAudioAx_80027648();
        ext::lbBgFlash_800209F4();
        let gobj = ext::GObj_Create(0xE, 0xF, 0);
        ext::HSD_GObjObject_80390A70(gobj, ext::hsd_sobjlib_804d7960(), core::ptr::null_mut());
        ext::GObj_SetupGXLink(gobj, ext::lbMthp8001F928 as usize, 0xB, 0);
        let mut ckind = ext::gm_801BEFB0();
        // TARGET_PC: the three ending THP filename tables contain retail character rows.
        ckind = ext::gm_MexVanillaKind(ckind, 1);
        let mut gover_kind = ext::gm_GetCurrentGameMode() as i32;
        if gover_kind == GM_DEBUG_GOVER as i32 {
            gover_kind = ext::gm_801BF050();
        }
        let thpfile = if gover_kind == GM_CLASSIC_GOVER as i32 {
            gmRegend_SimpleThpFiles[ckind as usize]
        } else if gover_kind == GM_ADVENTURE_GOVER as i32 {
            gmRegend_AdventureThpFiles[ckind as usize]
        } else {
            gmRegend_AllstarThpFiles[ckind as usize]
        };
        ext::lbMthp8001FAA0(thpfile, 0x230, 0x1A0);
        let thp_disp = ext::lbMthp8001F890(gobj);
        (*thp_disp).x10 = 320.0;
        (*thp_disp).x14 = 240.0;
        (*thp_disp).x40 |= 2;
        let mode = ext::gm_GetCurrentGameMode() as i32;
        let level = if mode == GM_CLASSIC_GOVER as i32 {
            ext::gm_8017DFF4(1)
        } else if mode == GM_ADVENTURE_GOVER as i32 {
            ext::gm_8017DFF4(0)
        } else {
            ext::gm_8017DFF4(2)
        };
        if level == 4 {
            ext::lbAudioAx_800237A8(0x9C45, 0x7F, 0x40);
        } else {
            ext::lbAudioAx_800237A8(0x9C41, 0x7F, 0x40);
        }
    }
}

/// `gm_Scene_Congrats_OnFrame()`: count down the input delay; A/Start starts the exit timer (and the flash),
/// whose expiry ends the scene (to 6 in the debug game-over, 1 otherwise).
pub fn gm_Scene_Congrats_OnFrame() {
    unsafe {
        if gmRegend_InputDelayTimer != 0 {
            gmRegend_InputDelayTimer -= 1;
        } else if gmRegend_ExitTimer != 0 {
            gmRegend_ExitTimer -= 1;
            if gmRegend_ExitTimer == 0 {
                if ext::gm_GetCurrentGameMode() == GM_DEBUG_GOVER as u8 {
                    ext::gm_801A6630(6);
                } else {
                    ext::gm_801A6630(1);
                }
            }
        } else if ext::gm_GetButtonsTriggered(ext::gm_801BF010() as u8) & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
            ext::lbBgFlash_8002063C(0x3C);
            gmRegend_ExitTimer = 0x3C;
            ext::lbAudioAx_80023694();
            sfxForward();
        }
    }
}

/// `static char* gmRegend_SimpleThpFiles[26]`: the ending movie per character kind.
pub static gmRegend_SimpleThpFiles: [&str; 26] = [
    "GmRegendSimpleCaptain.thp",
    "GmRegendSimpleDonkey.thp",
    "GmRegendSimpleFox.thp",
    "GmRegendSimpleGamewatch.thp",
    "GmRegendSimpleKirby.thp",
    "GmRegendSimpleKoopa.thp",
    "GmRegendSimpleLink.thp",
    "GmRegendSimpleLuigi.thp",
    "GmRegendSimpleMario.thp",
    "GmRegendSimpleMarth.thp",
    "GmRegendSimpleMewtwo.thp",
    "GmRegendSimpleNess.thp",
    "GmRegendSimplePeach.thp",
    "GmRegendSimplePikachu.thp",
    "GmRegendSimplePoponana.thp",
    "GmRegendSimplePurin.thp",
    "GmRegendSimpleSamus.thp",
    "GmRegendSimpleYoshi.thp",
    "GmRegendSimpleZeldaseak.thp",
    "GmRegendSimpleZeldaseak.thp",
    "GmRegendSimpleFalco.thp",
    "GmRegendSimpleClink.thp",
    "GmRegendSimpleDrmario.thp",
    "GmRegendSimpleRoy.thp",
    "GmRegendSimplePichu.thp",
    "GmRegendSimpleGanon.thp",
];

/// `static char* gmRegend_AdventureThpFiles[26]`: the ending movie per character kind.
pub static gmRegend_AdventureThpFiles: [&str; 26] = [
    "GmRegendAdventureCaptain.thp",
    "GmRegendAdventureDonkey.thp",
    "GmRegendAdventureFox.thp",
    "GmRegendAdventureGamewatch.thp",
    "GmRegendAdventureKirby.thp",
    "GmRegendAdventureKoopa.thp",
    "GmRegendAdventureLink.thp",
    "GmRegendAdventureLuigi.thp",
    "GmRegendAdventureMario.thp",
    "GmRegendAdventureMarth.thp",
    "GmRegendAdventureMewtwo.thp",
    "GmRegendAdventureNess.thp",
    "GmRegendAdventurePeach.thp",
    "GmRegendAdventurePikachu.thp",
    "GmRegendAdventurePoponana.thp",
    "GmRegendAdventurePurin.thp",
    "GmRegendAdventureSamus.thp",
    "GmRegendAdventureYoshi.thp",
    "GmRegendAdventureZeldaseak.thp",
    "GmRegendAdventureZeldaseak.thp",
    "GmRegendAdventureFalco.thp",
    "GmRegendAdventureClink.thp",
    "GmRegendAdventureDrmario.thp",
    "GmRegendAdventureRoy.thp",
    "GmRegendAdventurePichu.thp",
    "GmRegendAdventureGanon.thp",
];

/// `static char* gmRegend_AllstarThpFiles[26]`: the ending movie per character kind.
pub static gmRegend_AllstarThpFiles: [&str; 26] = [
    "GmRegendAllstarCaptain.thp",
    "GmRegendAllstarDonkey.thp",
    "GmRegendAllstarFox.thp",
    "GmRegendAllstarGamewatch.thp",
    "GmRegendAllstarKirby.thp",
    "GmRegendAllstarKoopa.thp",
    "GmRegendAllstarLink.thp",
    "GmRegendAllstarLuigi.thp",
    "GmRegendAllstarMario.thp",
    "GmRegendAllstarMarth.thp",
    "GmRegendAllstarMewtwo.thp",
    "GmRegendAllstarNess.thp",
    "GmRegendAllstarPeach.thp",
    "GmRegendAllstarPikachu.thp",
    "GmRegendAllstarPoponana.thp",
    "GmRegendAllstarPurin.thp",
    "GmRegendAllstarSamus.thp",
    "GmRegendAllstarYoshi.thp",
    "GmRegendAllstarZeldaseak.thp",
    "GmRegendAllstarZeldaseak.thp",
    "GmRegendAllstarFalco.thp",
    "GmRegendAllstarClink.thp",
    "GmRegendAllstarDrmario.thp",
    "GmRegendAllstarRoy.thp",
    "GmRegendAllstarPichu.thp",
    "GmRegendAllstarGanon.thp",
];
