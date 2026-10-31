//! src/melee/gm/gm_1A33.c: the VS camera scene's glue (`gm_Scene_CameraVs_*`): builds the two side labels and the
//! header text from the camera UI of gmcamera.c, and turns button presses into the scene's result
//! (`*x0` = 0 confirm, 1 cancel, 2 back-out) before ending the scene. All 4 functions are checked against the
//! C by a state oracle (tests/tu_gm_1A33.rs over gen/turef/tu_gm_1A33_ref.c): the game's gm_1A33.c compiled
//! natively (shipping contraction) with the callees stubbed and logged; the log, the text objects and the scene's
//! `CameraVsData` are compared after every call.
//!
//! The callees in gmcamera.c and gm_1A36.c (ported) default to the real functions when no hook is set.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

pub const PAD_MAX_CONTROLLERS: u8 = 4;
pub const HSD_PAD_A: u64 = 1 << 8;
pub const HSD_PAD_B: u64 = 1 << 9;
pub const HSD_PAD_START: u64 = 1 << 12;
pub const PAD_BUTTON_B: u64 = 1 << 9;

/// `static f32 gmCamera_803DA758[12]`: text placements (font size x/y, position x/y) of the two sides and the header.
pub static gmCamera_803DA758: [f32; 12] = [0.7, 0.7, 168.0, 160.0, 0.7, 0.7, 168.0, 192.0, 0.7, 0.7, 0.0, 256.0];

/// `static struct CameraVsData gmCamera_VsCameraTextLayout;`
pub static mut gmCamera_VsCameraTextLayout: CameraVsData = unsafe { core::mem::zeroed() };

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics, except the ones in
/// gmcamera.c / gm_1A36.c, which default to this crate's port.
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
    // gm/gmcamera.c (ported)
    hook_or!(GMCAMERA_801A25C8, gmCamera_801A25C8, crate::tu::gmcamera::gmCamera_801A25C8, ());
    hook_or!(GMCAMERA_801A2334, gmCamera_801A2334, crate::tu::gmcamera::gmCamera_801A2334, (a: i32, x: f32, y: f32, w: f32, h: f32) -> *mut HSD_Text);
    hook_or!(GMCAMERA_801A253C, gmCamera_801A253C, crate::tu::gmcamera::gmCamera_801A253C, (a: *mut i32, b: *mut i32));
    hook_or!(GMCAMERA_801A2640, gmCamera_801A2640, crate::tu::gmcamera::gmCamera_801A2640, () -> i32);
    hook_or!(GMCAMERA_801A2650, gmCamera_801A2650, crate::tu::gmcamera::gmCamera_801A2650, ());
    hook_or!(GMCAMERA_801A2224, gmCamera_801A2224, crate::tu::gmcamera::gmCamera_801A2224, (p: *mut u8, v: u32) -> *mut u8);
    // gm/gm_1A36.c (ported)
    hook_or!(GM_GETBUTTONSTRIGGERED, gm_GetButtonsTriggered, __gbt, (idx: u8) -> u64);
    unsafe fn __gbt(idx: u8) -> u64 {
        crate::tu::gm_1A36::gm_GetButtonsTriggered(idx)
    }
    // sysdolphin/baselib/sislib.c
    pub static mut HSD_SISLIB_803A5ACC: Option<fn(i32, i32, f32, f32, f32, f32, f32) -> *mut HSD_Text> = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A5ACC(a: i32, b: i32, x: f32, y: f32, z: f32, w: f32, h: f32) -> *mut HSD_Text {
                match unsafe { HSD_SISLIB_803A5ACC } {
                    Some(f) => f(a, b, x, y, z, w, h),
                    None => { return unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a as _, b as _, x as _, y as _, z as _, w as _, h as _) as _ }; },
                }
            }
    pub static mut HSD_SISLIB_803A6368: Option<fn(*mut HSD_Text, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6368(text: *mut HSD_Text, v: i32)  {
                match unsafe { HSD_SISLIB_803A6368 } {
                    Some(f) => f(text, v),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A6368(text as _, v as _); } return; },
                }
            }
    pub static mut HSD_SISLIB_803A6530: Option<fn(i32, i32, i32) -> *mut u8> = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A6530(a: i32, b: i32, c: i32) -> *mut u8 {
                match unsafe { HSD_SISLIB_803A6530 } {
                    Some(f) => f(a, b, c),
                    None => { return unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6530(a as _, b as _, c as _) as _ }; },
                }
            }
    pub static mut HSD_SISLIB_803A660C: Option<fn(i32, i32, i32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A660C(a: i32, b: i32, c: i32)  {
                match unsafe { HSD_SISLIB_803A660C } {
                    Some(f) => f(a, b, c),
                    None => { unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A660C(a as _, b as _, c as _); } return; },
                }
            }
    pub static mut HSD_SISLIB_803A5CC4: Option<fn(*mut HSD_Text) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_SisLib_803A5CC4(text: *mut HSD_Text)  {
                match unsafe { HSD_SISLIB_803A5CC4 } {
                    Some(f) => f(text),
                    None => { unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(text as _); } return; },
                }
            }
    // lb/lbsnap.c, mn/mnmain.c, gm/gmscene.c, lb/lbaudio_ax.c (the sfx inlines of mn/inlines.h)
    pub static mut LBSNAP_8001D338: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn lbSnap_8001D338(i: i32) -> i32 {
                match unsafe { LBSNAP_8001D338 } {
                    Some(f) => f(i),
                    None => { return unsafe { melee_runtime::tu::lbsnap::lbSnap_8001D338(i as _) as _ }; },
                }
            }
    pub static mut MN_8022F218: Option<fn() -> bool> = None;
            #[allow(non_snake_case)]
            pub fn mn_8022F218() -> bool {
                match unsafe { MN_8022F218 } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::mn_22EC::mn_8022F218() as _ }; },
                }
            }
    pub static mut MN_8022F268: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn mn_8022F268()  {
                match unsafe { MN_8022F268 } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::mn_22EC::mn_8022F268(); } return; },
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
    pub static mut LBAUDIOAX_80024030: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn lbAudioAx_80024030(id: i32)  {
                match unsafe { LBAUDIOAX_80024030 } {
                    Some(f) => f(id),
                    None => { unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024030(id as _); } return; },
                }
            }
}

/// `sfxForward()` / `sfxBack()` (mn/inlines.h, static inline).
#[inline(always)]
fn sfxForward() {
    ext::lbAudioAx_80024030(1);
}
#[inline(always)]
fn sfxBack() {
    ext::lbAudioAx_80024030(0);
}

/// `gmCamera_801A33BC()`: rebuild the two side labels and the header text with its message.
pub fn gmCamera_801A33BC() {
    melee_core::ct!("gmCamera_801A33BC");
    unsafe {
        let mut sp10: i32 = 0;
        let mut spC: i32 = 0;
        let layout = &raw mut gmCamera_VsCameraTextLayout;

        ext::gmCamera_801A25C8();
        (*layout).anon_4.anon_0.slot_a = ext::gmCamera_801A2334(
            0,
            gmCamera_803DA758[2],
            gmCamera_803DA758[3],
            gmCamera_803DA758[0],
            gmCamera_803DA758[1],
        );
        (*layout).anon_4.anon_0.slot_b = ext::gmCamera_801A2334(
            1,
            gmCamera_803DA758[6],
            gmCamera_803DA758[7],
            gmCamera_803DA758[4],
            gmCamera_803DA758[5],
        );
        ext::gmCamera_801A253C(&mut sp10, &mut spC);
        let text = ext::HSD_SisLib_803A5ACC(3, ext::gmCamera_801A2640(), gmCamera_803DA758[10], gmCamera_803DA758[11], 0.0, 914.2857, 64.0);
        (*layout).anon_4.anon_0.bottom_text = text;
        (*text).default_alignment = 1;
        (*text).default_kerning = 1;
        (*text).default_fitting = 1;
        {
            let fy = gmCamera_803DA758[9];
            let fx = gmCamera_803DA758[8];
            (*text).font_size.x = fx;
            (*text).font_size.y = fy;
        }
        ext::HSD_SisLib_803A6368(text, 0x15);
        match sp10 {
            0 | 1 => {
                ext::gmCamera_801A2224(ext::HSD_SisLib_803A6530(3, 0x15, 0x11), spC as u32);
                ext::HSD_SisLib_803A660C(3, 0x15, 0x12);
            }
            2 => {
                ext::HSD_SisLib_803A6530(3, 0x15, 0x13);
            }
            _ => {
                ext::HSD_SisLib_803A6530(3, 0x15, 0x14);
            }
        }
    }
}

/// `gm_Scene_CameraVs_OnFrame()`: rebuild the text after a snapshot; otherwise A/Start confirm (`*x0 = 0`),
/// B cancels (`*x0 = 1`) and the menu back-out sets `*x0 = 2`; each ends the scene.
pub fn gm_Scene_CameraVs_OnFrame() {
    melee_core::ct!("gm_Scene_CameraVs_OnFrame");
    unsafe {
        let data = &raw mut gmCamera_VsCameraTextLayout;
        if ext::lbSnap_8001D338(0) != 0 || ext::lbSnap_8001D338(1) != 0 {
            for i in 0..3usize {
                if !(*data).anon_4.text[i].is_null() {
                    ext::HSD_SisLib_803A5CC4((*data).anon_4.text[i]);
                    (*data).anon_4.text[i] = core::ptr::null_mut();
                }
            }
            gmCamera_801A33BC();
        } else if ext::mn_8022F218() {
            sfxBack();
            ext::mn_8022F268();
            *(*data).x0 = 2;
            ext::gm_801A4B60();
        } else if ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS) & (HSD_PAD_START | HSD_PAD_A) != 0 {
            sfxForward();
            *(*data).x0 = 0;
            ext::gm_801A4B60();
        } else if ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS) & PAD_BUTTON_B != 0 {
            sfxBack();
            *(*data).x0 = 1;
            ext::gm_801A4B60();
        }
    }
}

/// `gm_Scene_CameraVs_OnEnter(arg0)`: remember the result word, reset the camera UI and build the texts.
pub fn gm_Scene_CameraVs_OnEnter(arg0: *mut c_void) {
    melee_core::ct!("gm_Scene_CameraVs_OnEnter");
    unsafe {
        let layout = &raw mut gmCamera_VsCameraTextLayout;
        (*layout).x0 = arg0 as *mut u32;
        ext::gmCamera_801A2650();
        (*layout).anon_4.anon_0.slot_a = core::ptr::null_mut();
        (*layout).anon_4.anon_0.slot_b = core::ptr::null_mut();
        (*layout).anon_4.anon_0.bottom_text = core::ptr::null_mut();
        gmCamera_801A33BC();
    }
}

/// `gm_Scene_CameraVs_OnExit(unused)`: nothing.
pub fn gm_Scene_CameraVs_OnExit(_unused: *mut c_void) { melee_core::ct!("gm_Scene_CameraVs_OnExit");}
