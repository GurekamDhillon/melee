//! src/melee/gm/gmtoulib.c: tournament-mode library (bracket model, camera and light/fog
//! helpers, display of bracket lines, character/stage helpers, `gm_801905F0` match setup).
//!
//! Callees in other (unported) TUs go through [`ext`], one stub per C function, named like the C
//! function. Every stub forwards to [`hook::HOOK`] when a test installed one and panics with
//! "TODO(ext) name" otherwise. The state oracle (tests/tu_gmtou.rs) installs a hook that logs
//! `(name, args)` and answers from a seeded script, and runs the same hook in the C harness.
//!
//! Fused float sites (`tools/rust-port/fma_sites.py`): `fn_8018B090` case 24 (six `±0.1f * t + x`),
//! `gmTournament_GetBracketSlide{Forward,Reverse}Y` (`0.3f * t + x`) and `fn_8018E85C`
//! (`0.0083f * n + 1.0f`, twice) are `fmadds` below.
//!
//! GameCube-layout raw offsets in the C (`fn_8018E85C`, `fn_8018F508`, `fn_8018FA24`): the
//! BracketEntry stride/offset arithmetic is expressed through the named fields (same bytes on the
//! GC: slot stride 0x2C, entry size 0xDC); the TmData offsets <= 0x4E0 agree between the GC and
//! native layouts, so those stay byte-offset accesses.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::HSD_GObj as HSD_GObj_gm_r9;
#[allow(unused_imports)]
use melee_types::BracketEntry as BracketEntry_gm_r10;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::BracketEntry as BracketEntry_gm_ptr;
#[allow(unused_imports)]
use mt::HSD_GObj as HSD_GObj_gm_ptr;
#[allow(unused_imports)]
use crate::gm::{CKIND_GKOOPS, CKIND_KOOPA, CKIND_SEAK};
use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};

use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_core::fp::{fnmadds, fnmsubs};
use melee_runtime::cabi::{f2i32, sdiv};
use melee_types::inline::{
    HSD_JObjMtxIsDirty, HSD_JObjSetScaleX, HSD_JObjSetScaleY, HSD_JObjSetScaleZ, HSD_JObjSetTranslateX,
    HSD_JObjSetTranslateY, HSD_JObjSetTranslateZ, JObjDirty,
};
use melee_types::*;

// ---------------------------------------------------------------------------------------------
// ext machinery
// ---------------------------------------------------------------------------------------------

/// The call hook behind every `ext` stub.
pub mod hook {
    use melee_types::*;

    /// `(callee name, arguments as 64-bit words (floats as their f32 bits)) -> return word`.
    pub type Hook = unsafe fn(&'static str, &[u64]) -> u64;
    pub static mut HOOK: Option<Hook> = None;

    /// A value that crosses the ext boundary as one 64-bit word.
    pub trait Word: Sized {
        fn to_w(self) -> u64;
        fn from_w(w: u64) -> Self;
    }
    macro_rules! int_word {
        ($($t:ty),*) => {$(
            impl Word for $t {
                #[inline] fn to_w(self) -> u64 { self as i64 as u64 }
                #[inline] fn from_w(w: u64) -> Self { w as $t }
            }
        )*};
    }
    int_word!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);
    impl Word for bool {
        fn to_w(self) -> u64 {
            self as u64
        }
        fn from_w(w: u64) -> Self {
            w != 0
        }
    }
    impl Word for f32 {
        fn to_w(self) -> u64 {
            self.to_bits() as u64
        }
        fn from_w(w: u64) -> Self {
            f32::from_bits(w as u32)
        }
    }
    impl Word for f64 {
        fn to_w(self) -> u64 {
            self.to_bits()
        }
        fn from_w(w: u64) -> Self {
            f64::from_bits(w)
        }
    }
    impl Word for () {
        fn to_w(self) -> u64 {
            0
        }
        fn from_w(_w: u64) -> Self {}
    }
    impl<T> Word for *mut T {
        fn to_w(self) -> u64 {
            self as usize as u64
        }
        fn from_w(w: u64) -> Self {
            w as usize as *mut T
        }
    }
    impl<T> Word for *const T {
        fn to_w(self) -> u64 {
            self as usize as u64
        }
        fn from_w(w: u64) -> Self {
            w as usize as *const T
        }
    }
    impl Word for HSD_GObjEvent {
        fn to_w(self) -> u64 {
            self.map_or(0, |f| f as usize as u64)
        }
        fn from_w(w: u64) -> Self {
            if w == 0 { None } else { Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_GObj)>(w as usize) }) }
        }
    }
    impl Word for GObj_RenderFunc {
        fn to_w(self) -> u64 {
            self.map_or(0, |f| f as usize as u64)
        }
        fn from_w(w: u64) -> Self {
            if w == 0 { None } else { Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_GObj, isize)>(w as usize) }) }
        }
    }

    /// Dispatch one ext call.
    pub unsafe fn call(name: &'static str, args: &[u64]) -> u64 {
        match HOOK {
            Some(h) => h(name, args),
            None => crate::tu::ext_hook::fallback(name, args), // menuguard: the engine by-name table
        }
    }
}

/// Defines ext stubs: `fn name(a: T, ..) -> R;` becomes an `extern "C"` function that sends its
/// arguments through [`hook::call`].
#[macro_export]
macro_rules! gmtou_ext {
    (@ret ; $e:expr) => {{ let _ = $e; }};
    (@ret $r:ty ; $e:expr) => { <$r as $crate::tu::gmtoulib::hook::Word>::from_w($e) };
    ($( fn $name:ident ( $($a:ident : $t:ty),* $(,)? ) $(-> $r:ty)? ; )*) => {$(
        #[allow(improper_ctypes_definitions, unused_variables, non_snake_case)]
        pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
            $crate::gmtou_ext!(@ret $($r)? ; $crate::tu::gmtoulib::hook::call(
                stringify!($name),
                &[$($crate::tu::gmtoulib::hook::Word::to_w($a)),*]))
        }
    )*};
}

/// Callees owned by other (unported) TUs, and the globals they own.
pub mod ext {
    use super::*;

    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
    pub use melee_runtime::tu::controller::HSD_PadMasterStatus;

    crate::gmtou_ext! {
        fn HSD_GObj_803910D8(gobj: *mut HSD_GObj, arg1: isize);
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: isize);
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, unused: isize);
        fn fn_801640B0(item_mask: *mut u64);
        fn gm_80164430(arg0: u16) -> i32;
        fn gm_RumbleEnabledForPlayer(port: i32, nametag: i32) -> i32;
        fn gm_SetupAllPlayerDefaults(arg0: *mut PlayerInitData);
        fn gm_SetupRulesDefaults(arg0: *mut StartMeleeRules);
        fn gm_8016895C(arg0: *mut HSD_JObj, arg1: *mut DynamicModelDesc, idx: i32);
        fn gm_LoadAnnouncer();
        fn gm_GetNumCostumesForCKind(arg0: u8) -> u8;
        fn gm_GetButtonsPressed(arg0: u8) -> u64;
        fn gm_GetButtonsTriggered(arg0: u8) -> u64;
        fn gm_801A36C0(arg0: u8) -> u64;
        fn gmMainLib_GetGameRules() -> *mut GameRules;
        fn gmMainLib_GetGamePrefs() -> *mut GamePrefs;
        fn gmMainLib_8015ED30() -> i32;
        fn lbLang_IsSavedLanguageUS() -> i32;
        fn lb_80011AC4(arg0: *mut *mut LightList) -> *mut HSD_LObj;
        fn mn_8022F0F0(arg0: i32);
        fn mn_8022F138(arg0: u16, arg1: u16);
        fn HSD_JObjReqAnimAll(arg0: *mut HSD_JObj, arg1: f32);
        fn HSD_JObjSetMtxDirtySub(arg0: *mut HSD_JObj);
        fn HSD_JObjLoadJoint(arg0: *mut HSD_Joint) -> *mut HSD_JObj;
        fn HSD_JObjAnimAll(arg0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(arg0: *mut HSD_JObj, flags: u32);
        fn GetNameText(slot: i32) -> *mut c_char;
        fn mnSelStageRandom() -> i32;
        fn Player_SetPlayerCharacter(slot: i32, value: CharacterKind);
        fn Player_SetSlottype(slot: i32, value: Gm_PKind);
        fn Player_SetCostumeId(slot: i32, costume_id: i32);
        fn HSD_CObjSetInterest(cobj: *mut HSD_CObj, arg1: *mut Vec3);
        fn HSD_CObjSetEyePosition(cobj: *mut HSD_CObj, arg1: *mut Vec3);
        fn HSD_CObjSetFov(arg0: *mut HSD_CObj, arg1: f32);
        fn HSD_CObjLoadDesc(desc: *mut HSD_CObjDesc) -> *mut HSD_CObj;
        fn HSD_FogSet(arg0: *mut HSD_Fog);
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(arg0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(arg0: *mut HSD_GObj, arg1: HSD_GObjEvent, arg2: u8) -> *mut HSD_GObjProc;
        fn DrawRectangle(x: f32, y: f32, w: f32, h: f32, color: *mut GXColor);
        fn hsd_80391A04(scale_x: f32, scale_y: f32, line_width: i32);
        fn HSD_Randi(max_val: i32) -> i32;
        fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32;
        fn HSD_SisLib_803A5E70();
    }

    /// `HSD_SisLib_803A6B98(text, x, y, fmt, ...)`; `va` are the variadic words (unused by the
    /// recorded hook: only the fixed arguments are compared).
    pub unsafe extern "C" fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const c_char) -> i32 {
        use hook::Word;
        <i32 as Word>::from_w(hook::call("HSD_SisLib_803A6B98", &[text.to_w(), x.to_w(), y.to_w(), fmt.to_w()]))
    }
    /// `printf(fmt, ...)` with one integer variadic.
    pub unsafe extern "C" fn printf(fmt: *const c_char, v: i32) -> i32 {
        use hook::Word;
        <i32 as Word>::from_w(hook::call("printf", &[fmt.to_w(), v.to_w()]))
    }
    /// `__assert(file, line, expr)`: aborts.
    pub unsafe extern "C" fn __assert(file: *const c_char, line: u32, expr: *const c_char) -> ! {
        use hook::Word;
        hook::call("__assert", &[file.to_w(), line.to_w(), expr.to_w()]);
        panic!("__assert");
    }
}

// ---------------------------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------------------------

pub static mut lbl_804D663C: i32 = 0;
pub static mut lbl_804D6638: *mut HSD_Archive = null_mut();
pub static mut lbl_804D6630: i32 = 0;
pub static mut lbl_804D6634: i32 = 0;

/// `lbl_803B7C80`: slide wobble table.
pub const lbl_803B7C80: [i32; 10] = [30, -20, 15, -12, 10, -8, 6, -4, 2, 1];

/// `lbl_803D9DF4` / `lbl_803D9E08` (eye / interest).
pub static mut lbl_803D9DF4: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 320.0, y: -240.0, z: 415.6922 }, robjdesc: null_mut() };
pub static mut lbl_803D9E08: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 320.0, y: -240.0, z: 0.0 }, robjdesc: null_mut() };

/// `cobj_desc` (rodata at 0x3B7CA8).
pub static mut cobj_desc: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: PROJ_PERSPECTIVE as u16,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: addr_of_mut!(lbl_803D9DF4),
    interest: addr_of_mut!(lbl_803D9E08),
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 30000.0,
    fov: 60.0,
    aspect: 1.3333333,
};

/// `lbl_803D9D20` (gmtoulib.static.h): per-count tables.
pub static mut lbl_803D9D20: lbl_803D9D20_t = lbl_803D9D20_t {
    x0: [
        0x04, 0x06, 0x08, 0x0C, 0x10, 0x18, 0x20, 0x30, 0x40, 0x06, 0x09, 0x0C, 0x12, 0x1B, 0x08, 0x0C, 0x10, 0x10, 0x14,
        0x18, 0x1C, 0x20, 0x20, 0x24, 0x28, 0x2C, 0x30, 0x34, 0x38, 0x3C, 0x40, 0x40,
    ],
    x20: [
        0x04, 0x07, 0x08, 0x0E, 0x10, 0x19, 0x20, 0x31, 0x3F, 0x04, 0x05, 0x08, 0x0A, 0x0E, 0x04, 0x05, 0x06, 0x08, 0x09,
        0x0A, 0x0B, 0x0C, 0x10, 0x0D, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x17, 0x06, 0x05, 0x04, 0x04, 0x06, 0x04,
        0x05, 0x04, 0x05, 0x04, 0x04, 0x04, 0x05, 0x04, 0x04, 0x04, 0x05, 0x06, 0x05, 0x04, 0x05, 0x05, 0x04, 0x04, 0x04,
    ],
    x59: [
        0x00, 0x01, 0x02, 0x04, 0x05, 0x06, 0x08, 0x0B, 0x0D, 0x0E, 0x12, 0x11, 0x10, 0x0C, 0x03, 0x16, 0x07, 0x0F, 0x0A,
        0x09, 0x19, 0x15, 0x14, 0x18, 0x17,
    ],
    x72: [0; 26],
};

/// `lbl_803D9DAC`: bracket camera (current / target / step).
pub static mut lbl_803D9DAC: lbl_803D9DAC_t = lbl_803D9DAC_t {
    current: Vec { x: 320.0, y: -240.0, z: 415.6922 },
    target: Vec { x: 320.0, y: -240.0, z: 415.6922 },
    step: Vec { x: 0.0, y: 0.0, z: 0.0 },
};

/// `lbl_803D9DD0`: the camera object (cobj at +0, fov at +0x20).
pub static mut lbl_803D9DD0: lbl_803D9DD0_t = unsafe { core::mem::zeroed() };

/// `lbl_803D9E1C`: per-entrant-count (x, y) of the bracket-champion slot.
pub static lbl_803D9E1C: [[i16; 2]; 32] = [
    [0x140, 0x8C], [0x15E, 0x82], [0x140, 0x96], [0x140, 0x78], [0x140, 0xA0], [0x172, 0xA0], [0x140, 0x8C], [0x168, 0xEB],
    [0x140, 0xF0], [0x140, 0xC8], [0x140, 0xBE], [0x140, 0x96], [0x140, 0xAA], [0x140, 0x8C], [0x140, 0xB4], [0x140, 0xB4],
    [0x140, 0xA0], [0x140, 0x8C], [0x14E, 0x8C], [0x140, 0x8C], [0x140, 0xA0], [0x140, 0xAA], [0x140, 0x96], [0x140, 0xF0],
    [0x154, 0xF0], [0x13F, 0xF0], [0x13F, 0xF0], [0x13F, 0xF0], [0x147, 0xF0], [0x140, 0xF0], [0x140, 0xF0], [0x140, 0xF0],
];

/// `lbl_80473AB8` (.bss): the 64 bracket entries. `lbl_804771B8`: the three region source tables.
pub static mut lbl_80473AB8: [BracketEntry; 64] = unsafe { core::mem::zeroed() };
pub static mut lbl_804771B8: TmBoxArrays = unsafe { core::mem::zeroed() };
/// `gm_80477738` / `gm_804771C4` (must not be static in the C).
pub static mut gm_80477738: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut gm_804771C4: TmData = unsafe { core::mem::zeroed() };

/// `lbl_803D9F0C`: the last three random characters.
pub static mut lbl_803D9F0C: lbl_803D9F0C_t = lbl_803D9F0C_t { x0: -1, x4: -1, x8: -1 };

/// The three name templates (Shift-JIS "ｸﾗｽ 00", "ＧＮＴ 00", "ＣＰＵ 00"); 12 bytes each as on the GC
/// (the C writes the terminator at index 9, one past the 9 initialised bytes).
pub static mut lbl_803D9EE8: [u8; 12] = [131, 81, 131, 88, 131, 103, 32, 48, 48, 0, 0, 0];
pub static mut lbl_803D9EF4: [u8; 12] = [130, 103, 130, 108, 130, 109, 32, 48, 48, 0, 0, 0];
pub static mut lbl_803D9F00: [u8; 12] = [130, 98, 130, 111, 130, 116, 32, 48, 48, 0, 0, 0];
pub static mut lbl_804DA6B4: *mut u8 = addr_of_mut!(lbl_803D9EE8) as *mut u8;
pub static mut lbl_804DA6B8: *mut u8 = addr_of_mut!(lbl_803D9EF4) as *mut u8;
pub static mut lbl_804DA6BC: *mut u8 = addr_of_mut!(lbl_803D9F00) as *mut u8;
pub static mut lbl_804DA6C0: *mut u8 = addr_of_mut!(lbl_803D9F00) as *mut u8;
pub static mut lbl_804DA6C4: *mut u8 = addr_of_mut!(lbl_803D9EE8) as *mut u8;
pub static mut lbl_804DA6C8: *mut u8 = addr_of_mut!(lbl_803D9EF4) as *mut u8;
pub static mut lbl_804DA6CC: *mut u8 = addr_of_mut!(lbl_803D9F00) as *mut u8;
pub static mut lbl_804DA6D0: *mut u8 = addr_of_mut!(lbl_803D9F00) as *mut u8;

const lbl_804DA67C: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0, a: 0xFF };
const lbl_804DA684: GXColor = GXColor { r: 255, g: 255, b: 0, a: 255 };
const col: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0x00, a: 0xFF };
const lbl_804DA69C: GXColor = GXColor { r: 255, g: 255, b: 0, a: 255 };

/// Already ported (rounds 1 to 10): re-exported under this TU.

// ---------------------------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------------------------

/// dolphin/pad.h button masks used by the tournament menus.
pub const PAD_BUTTON_LEFT: u32 = 1 << 0;
pub const PAD_BUTTON_RIGHT: u32 = 1 << 1;
pub const PAD_BUTTON_DOWN: u32 = 1 << 2;
pub const PAD_BUTTON_UP: u32 = 1 << 3;
pub const PAD_BUTTON_A: u32 = 1 << 8;
pub const PAD_BUTTON_B: u32 = 1 << 9;
pub const PAD_BUTTON_X: u32 = 1 << 10;
pub const PAD_BUTTON_Y: u32 = 1 << 11;
pub const PAD_BUTTON_START: u32 = 1 << 12;
pub const PAD_STICK_UP: u32 = 1 << 16;
pub const PAD_STICK_DOWN: u32 = 1 << 17;
pub const PAD_STICK_LEFT: u32 = 1 << 18;
pub const PAD_STICK_RIGHT: u32 = 1 << 19;

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(j) {
            unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        unsafe { ext::HSD_JObjSetMtxDirtySub(j) };
    }
}

/// jobj.h `HSD_JObjSetTranslate{X,Y,Z}` / `SetScale{X,Y,Z}` (HSD_ASSERT(jobj) aborts on null).
#[inline]
pub unsafe fn set_tx(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_tx");
    HSD_JObjSetTranslateX(&mut *j, v, &mut Dirty)
}
#[inline]
pub unsafe fn set_ty(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_ty");
    HSD_JObjSetTranslateY(&mut *j, v, &mut Dirty)
}
#[inline]
pub unsafe fn set_tz(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_tz");
    HSD_JObjSetTranslateZ(&mut *j, v, &mut Dirty)
}
#[inline]
pub unsafe fn set_sx(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_sx");
    HSD_JObjSetScaleX(&mut *j, v, &mut Dirty)
}
#[inline]
pub unsafe fn set_sy(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_sy");
    HSD_JObjSetScaleY(&mut *j, v, &mut Dirty)
}
#[inline]
pub unsafe fn set_sz(j: *mut HSD_JObj, v: f32) {
    melee_core::ct!("set_sz");
    HSD_JObjSetScaleZ(&mut *j, v, &mut Dirty)
}

/// `DrawRectangle(x, y, w, h, &copy_of_color)`.
macro_rules! dr {
    ($x:expr, $y:expr, $w:expr, $h:expr, $c:expr) => {{
        let mut color: GXColor = $c;
        ext::DrawRectangle($x, $y, $w, $h, &mut color);
    }};
}

#[inline]
unsafe fn ent(i: i32) -> &'static mut BracketEntry {
    &mut (*addr_of_mut!(lbl_80473AB8))[i as usize]
}

// ---------------------------------------------------------------------------------------------
// functions
// ---------------------------------------------------------------------------------------------

/// `gmTournament_SetBracketByes` (re-uses the round-3 port).
#[inline]
unsafe fn set_byes(count: i32) {
    crate::gm_ptr::gmTournament_SetBracketByes(&mut *addr_of_mut!(lbl_80473AB8), count);
}

pub unsafe extern "C" fn fn_8018A514(count: i32, val: f32) {
    melee_core::ct!("fn_8018A514");
    let region: i32 = if count < 9 { 0 } else if count >= 14 { 2 } else { 1 };
    let srcs = addr_of_mut!(lbl_804771B8) as *mut *mut BracketSrcEntry;
    let mut src: *mut BracketSrcEntry = *srcs.add(region as usize);
    let x20 = &(*addr_of_mut!(lbl_803D9D20)).x20;

    if count < 9 {
        let mut i = 0;
        while i < count {
            src = src.offset(x20[i as usize] as isize);
            i += 1;
        }
    } else if count < 14 {
        let mut i = 9;
        while i < count {
            src = src.offset(x20[i as usize] as isize);
            i += 1;
        }
    } else {
        let mut i = 14;
        while i < count {
            src = src.offset(x20[i as usize] as isize);
            i += 1;
        }
    }

    // ((lbl_803D9D20_u*) &lbl_803D9D20)->bytes[count + 0x20]
    let n = *(addr_of_mut!(lbl_803D9D20) as *const u8).offset((count + 0x20) as isize) as i32;

    let mut i = 0;
    while i < n {
        let e = ent(i);
        let s = &*src;
        e.x0 = s.x0;
        e.x1 = s.x1;
        e.x2 = s.x2;
        e.x3 = s.x3;
        e.x4 = s.x4;
        e.x5 = s.x5;
        e.x6 = s.x6;
        e.x1C = val;
        e.xC = s.x8;
        e.x14 = s.xC;
        e.x10 = s.x10;
        e.x18 = s.x14;
        e.x20.r = s.x18;
        e.x20.g = s.x1A;
        e.x20.b = s.x1C;
        e.x20.a = s.x1E;
        e.x24 = s.x19;
        e.x25 = s.x1B;
        e.x26 = s.x1D;
        e.x27 = s.x1F;
        e.x28 = s.x20;
        e.slots[0].x52 = 9;
        e.slots[0].x32 = 0;
        e.slots[1].x52 = 9;
        e.slots[1].x32 = 0;
        e.slots[2].x52 = 9;
        e.slots[2].x32 = 0;
        e.slots[3].x52 = 9;
        e.slots[3].x32 = 0;
        e.slots[0].x30 = s.x21;
        e.slots[1].x30 = s.x22;
        e.slots[2].x30 = s.x23;
        e.slots[3].x30 = s.x24;
        src = src.add(1);
        i += 1;
    }

    if region == 0 {
        set_byes(count);
    }
}

pub unsafe extern "C" fn fn_8018A970(arg0: i32) {
    melee_core::ct!("fn_8018A970");
    let mut i = 0;
    while i < 0x40 {
        if ent(i).x0 != 0 {
            let gobj = ext::GObj_Create(0xE, 0x1B, 0);
            (*gobj).user_data = ent(i) as *mut BracketEntry as *mut c_void;
            if i == 0 {
                ext::HSD_GObj_SetupProc(gobj, Some(fn_8018B090), 0);
            }
            ext::GObj_SetupGXLink(gobj, Some(fn_8018E46C), 4, 1);
        }
        i += 1;
    }

    if arg0 < 9 {
        set_byes(arg0);
    }
}

/// `gmTournament_SetTripleRightCoords` (round-10 port is on `BracketEntry` + out-slots).
unsafe fn gmTournament_SetTripleRightCoords(entry: &BracketEntry, s: &mut BracketEntrySlot) {
    let val1 = entry.xC + entry.x14;
    s.x3C = val1;
    s.x44 = val1;
    s.x34 = val1;
    let val = entry.x10 + entry.x18;
    s.x40 = val;
    s.x48 = val;
    s.x38 = val;
    s.x40 = entry.x10 + entry.x18 - sdiv(entry.x18, 3);
}

unsafe fn gmTournament_SetRegularCoords(entry_idx: i32, slot_idx: i32, x3: u8, s: &mut BracketEntrySlot) {
    let e = ent(entry_idx);
    let val1 = e.xC + slot_idx * sdiv(e.x14, x3 as i32);
    s.x3C = val1;
    s.x44 = val1;
    s.x34 = val1;
    let val2 = e.x10 + e.x18 - e.x18 * e.x2 as i32;
    s.x40 = val2;
    s.x48 = val2;
    s.x38 = val2;
    s.x40 = e.x10 + e.x18 * e.x2 as i32;
}

pub unsafe extern "C" fn fn_8018AA74(jobj: *mut HSD_JObj, entry_idx: i32, slot_idx: i32) {
    melee_core::ct!("fn_8018AA74");
    let tm = gm_GetTournamentData();
    let entry: *mut BracketEntry = ent(entry_idx);
    let p: *mut BracketEntrySlot = &mut (*entry).slots[slot_idx as usize];

    if (*entry).x1 != 0 {
        let x3 = (*entry).x3;
        if x3 == 0 && (*entry).x4 != 0 {
            match (*entry).x4 {
                1 => {
                    let xc = (*entry).xC;
                    (*p).x3C = xc;
                    (*p).x44 = xc;
                    (*p).x34 = xc;
                    let x18 = (*entry).x18;
                    let x10 = (*entry).x10;
                    let val = x10 + x18 - slot_idx * x18;
                    (*p).x40 = val;
                    (*p).x48 = val;
                    (*p).x38 = val;
                    (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 2);
                }
                2 => match slot_idx {
                    0 => {
                        let val1 = (*entry).xC + 0x2B;
                        (*p).x3C = val1;
                        (*p).x44 = val1;
                        (*p).x34 = val1;
                        let val2 = (*entry).x10 + (*entry).x18;
                        (*p).x40 = val2;
                        (*p).x48 = val2;
                        (*p).x38 = val2;
                        (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 2);
                    }
                    1 => {
                        let val1 = (*entry).xC + (*entry).x14;
                        (*p).x3C = val1;
                        (*p).x44 = val1;
                        (*p).x34 = val1;
                        let val2 = (*entry).x10 + (*entry).x18;
                        (*p).x40 = val2;
                        (*p).x48 = val2;
                        (*p).x38 = val2;
                        (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 2);
                    }
                    _ => {
                        let val1 = (*entry).xC + sdiv((*entry).x14, 2);
                        (*p).x3C = val1;
                        (*p).x44 = val1;
                        (*p).x34 = val1;
                        let x10 = (*entry).x10;
                        (*p).x40 = x10;
                        (*p).x48 = x10;
                        (*p).x38 = x10;
                        (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 2);
                    }
                },
                3 => match slot_idx {
                    0 => {
                        let xc = (*entry).xC;
                        (*p).x3C = xc;
                        (*p).x44 = xc;
                        (*p).x34 = xc;
                        let val = (*entry).x10 + (*entry).x18;
                        (*p).x40 = val;
                        (*p).x48 = val;
                        (*p).x38 = val;
                        (*p).x40 = (*entry).x10 + (*entry).x18 - sdiv((*entry).x18, 3);
                    }
                    1 => gmTournament_SetTripleRightCoords(&*entry, &mut *p),
                    2 => {
                        let xc = (*entry).xC;
                        (*p).x3C = xc;
                        (*p).x44 = xc;
                        (*p).x34 = xc;
                        let x10 = (*entry).x10;
                        (*p).x40 = x10;
                        (*p).x48 = x10;
                        (*p).x38 = x10;
                        (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 3);
                    }
                    _ => {
                        let val1 = (*entry).xC + (*entry).x14;
                        (*p).x3C = val1;
                        (*p).x44 = val1;
                        (*p).x34 = val1;
                        let x10 = (*entry).x10;
                        (*p).x40 = x10;
                        (*p).x48 = x10;
                        (*p).x38 = x10;
                        (*p).x40 = (*entry).x10 + sdiv((*entry).x18, 3);
                    }
                },
                _ => {}
            }
        } else {
            gmTournament_SetRegularCoords(entry_idx, slot_idx, x3, &mut *p);
            if (*entry).x3 == 1 {
                let tm_x2e = (*tm).x2E;
                if tm_x2e == 6 {
                    if entry_idx == 5 && slot_idx == 1 {
                        let tmp = (*p).x48 + 0x46;
                        (*p).x48 = tmp;
                        (*p).x38 = tmp;
                    }
                } else if tm_x2e == 0xC {
                    if (entry_idx == 0xA && slot_idx == 0) || (entry_idx == 0xB && slot_idx == 1) {
                        let tmp = (*p).x48 + 0x3C;
                        (*p).x48 = tmp;
                        (*p).x38 = tmp;
                    }
                } else if tm_x2e == 0x18 {
                    if entry_idx == 0x17 && slot_idx == 1 {
                        let tmp = (*p).x48 + 0x28;
                        (*p).x48 = tmp;
                        (*p).x38 = tmp;
                    }
                } else if tm_x2e == 0x30 {
                    if entry_idx == 0x2E && slot_idx == 1 {
                        let tmp = (*p).x48 + 0x1E;
                        (*p).x48 = tmp;
                        (*p).x38 = tmp;
                    } else if entry_idx == 0x2F && slot_idx == 1 {
                        let tmp = (*p).x48 - 0x1E;
                        (*p).x48 = tmp;
                        (*p).x38 = tmp;
                    }
                }
            }
        }
    }

    set_tx(jobj, ent(entry_idx).slots[slot_idx as usize].x44 as f32);
    set_ty(jobj, -(ent(entry_idx).slots[slot_idx as usize].x48 as f32));
}

/// `gmTournament_GetBracketSlideYForward`: `-(0.3f * t + x48)`, fused.
#[inline]
unsafe fn gmTournament_GetBracketSlideYForward(slot: &BracketEntrySlot) -> f32 {
    -fmadds(0.3, lbl_804D6630 as f32, slot.x48 as f32)
}

/// `gmTournament_GetBracketSlideYReverse`: `-(x48 - 0.3f * t)`, fused as `fmadds(-0.3, t, x48)`.
#[inline]
unsafe fn gmTournament_GetBracketSlideYReverse(slot: &BracketEntrySlot) -> f32 {
    -fmadds(-0.3, lbl_804D6630 as f32, slot.x48 as f32)
}

unsafe fn fn_8018B090_inline3(entry: &BracketEntry) {
    fn_80190520((entry.xC + sdiv(entry.x14, 2)) as f32, -((entry.x10 + sdiv(entry.x18, 2)) as f32), -150.0);
}

unsafe fn fn_8018B090_inline5(entries: *mut BracketEntry, src: *mut BracketEntry, slot_idx: i32) {
    (*src).x1 = 0;
    let ne = (*src).x5 as usize;
    let ns = (*src).x6 as usize;
    let d = &mut (*entries.add(ne)).slots[ns];
    d.x30 = 1;
    d.x50 = (*src).slots[slot_idx as usize].x50;
    d.x51 = (*src).slots[slot_idx as usize].x51;
    d.x52 = (*src).slots[slot_idx as usize].x52;
    d.x4D = (*src).slots[slot_idx as usize].x4D;
    d.x4E = (*src).slots[slot_idx as usize].x4E;
    d.x4F = (*src).slots[slot_idx as usize].x4F;
    (*src).slots[slot_idx as usize].x30 = 0;
    (*src).slots[slot_idx as usize].x4E = 3;
}

unsafe fn fn_8018B090_inline8(camera: *mut lbl_803D9DAC_t) {
    crate::mn_ptr::mn_8022F410(&mut (*camera).current.x, &(*camera).target.x, (*camera).step.x);
    crate::mn_ptr::mn_8022F410(&mut (*camera).current.y, &(*camera).target.y, (*camera).step.y);
    crate::mn_ptr::mn_8022F410(&mut (*camera).current.z, &(*camera).target.z, (*camera).step.z);
    fn_80190520((*camera).current.x, (*camera).current.y, (*camera).current.z);
}

#[inline]
unsafe fn slot_jobj(entries: *mut BracketEntry, entry: i32, slot: i32) -> *mut HSD_JObj {
    (*(*entries.add(entry as usize)).slots[slot as usize].x2C).hsd_obj as *mut HSD_JObj
}

pub unsafe extern "C" fn fn_8018B090(arg0: *mut HSD_GObj) {
    melee_core::ct!("fn_8018B090");
    let entries: *mut BracketEntry = addr_of_mut!(lbl_80473AB8) as *mut BracketEntry;
    let tm = gm_GetTournamentData();
    let mut var_r24: i32 = 0;
    let idx = fn_8018F74C();
    let e = entries.add(idx as usize);
    let mut i: i32;
    let mut k: i32;
    let mut fall = false;

    match (*tm).cur_option {
        20 => {
            lbl_804D6630 = 0;
            return;
        }
        21 => {
            if (*e).x18 != 0 {
                i = 0;
                while i < 4 {
                    if (*e).slots[i as usize].x30 != 0 {
                        let jobj2 = slot_jobj(entries, idx, i);
                        let s = &(*e).slots[i as usize];
                        if (*e).x2 != 0 {
                            set_ty(jobj2, gmTournament_GetBracketSlideYReverse(s));
                        } else {
                            match (*e).x4 {
                                1 => {
                                    if i == 0 {
                                        set_ty(jobj2, gmTournament_GetBracketSlideYForward(s));
                                    } else {
                                        set_ty(jobj2, gmTournament_GetBracketSlideYReverse(s));
                                    }
                                }
                                2 | 3 => {
                                    if i <= 1 {
                                        set_ty(jobj2, gmTournament_GetBracketSlideYForward(s));
                                    } else {
                                        set_ty(jobj2, gmTournament_GetBracketSlideYReverse(s));
                                    }
                                }
                                _ => set_ty(jobj2, gmTournament_GetBracketSlideYForward(s)),
                            }
                        }
                    }
                    i += 1;
                }
            }
            lbl_804D6630 += 1;
            return;
        }
        22 => {
            let h = (*e).x18;
            if h != 0 {
                let c = addr_of_mut!(lbl_803D9DAC);
                (*c).current.x = 320.0;
                (*c).current.y = -240.0;
                (*c).current.z = 0.0;
                (*c).target.x = ((*e).xC + sdiv((*e).x14, 2)) as f32;
                (*c).target.y = -(((*e).x10 + sdiv(h, 2)) as f32);
                (*c).target.z = -150.0;
                let mut d = ((*c).target.x - (*c).current.x) * 0.033333;
                // ABS(x): (x < 0 ? -x : x), the product recomputed
                d = if d < 0.0 { -d } else { d };
                (*c).step.x = 1.0 + d;
                d = 0.033333 * ((*c).target.y - (*c).current.y);
                if d < 0.0 {
                    d = -d;
                }
                (*c).step.y = 1.0 + d;
                d = 0.033333 * ((*c).target.z - (*c).current.z);
                if d < 0.0 {
                    d = -d;
                }
                (*c).step.z = 1.0 + d;
            }
            lbl_804D6630 = 0;
            fall = true;
        }
        23 => fall = true,
        24 => {
            if (*e).x18 != 0 {
                let arr = lbl_803B7C80;
                lbl_804D6630 += 1;
                k = 0;
                while k < 4 {
                    if (*e).slots[k as usize].x30 != 0 {
                        let jobj = (*(*e).slots[k as usize].x2C).hsd_obj as *mut HSD_JObj;
                        let w = arr[(lbl_804D6630 % 10) as usize] as f32;
                        let x40 = (*e).slots[k as usize].x40 as f32;
                        // -(0.1f * w + x40)
                        let plus = |_: ()| -fmadds(0.1, w, x40);
                        // -(x40 - 0.1f * w) == -fmadds(-0.1, w, x40)
                        let minus = |_: ()| -fmadds(-0.1, w, x40);
                        if (*e).x2 != 0 {
                            set_ty(jobj, minus(()));
                        } else {
                            match (*e).x4 {
                                1 => {
                                    if k == 0 {
                                        set_ty(jobj, plus(()));
                                    } else {
                                        set_ty(jobj, minus(()));
                                    }
                                }
                                2 | 3 => {
                                    if k <= 1 {
                                        set_ty(jobj, plus(()));
                                    } else {
                                        set_ty(jobj, minus(()));
                                    }
                                }
                                _ => set_ty(jobj, plus(())),
                            }
                        }
                    }
                    k += 1;
                }
            }
            return;
        }
        25 => {
            i = 0;
            while i < 4 {
                if (*e).slots[i as usize].x30 != 0 {
                    (*e).slots[i as usize].x48 = (*e).slots[i as usize].x40;
                }
                i += 1;
            }
            return;
        }
        32 => {
            let h = (*e).x18;
            if h != 0 {
                fn_8018B090_inline3(&*e);
                i = 0;
                while i < 4 {
                    if (*e).slots[i as usize].x4C == 0 {
                        (*e).slots[i as usize].x3C = (*e).xC + sdiv((*e).x14, 2);
                    } else {
                        let t = (*e).slots[i as usize].x40;
                        (*e).slots[i as usize].x40 = (*e).slots[i as usize].x38;
                        (*e).slots[i as usize].x38 = t;
                    }
                    i += 1;
                }
            } else {
                fn_801904D0();
            }
            (*tm).cur_option = 0x21;
            return;
        }
        33 => {
            if (*e).x18 != 0 {
                i = 0;
                while i < 4 {
                    let sp: *mut BracketEntrySlot = &mut (*e).slots[i as usize];
                    if (*sp).x30 == 0 {
                        var_r24 += 1;
                    } else {
                        let t5 = (*sp).x4C;
                        if t5 == 0 {
                            lbl_804D6634 = i;
                            if (*e).x4 != 1 {
                                crate::mn_ptr::mn_8022F470(&mut (*sp).x44, &(*sp).x3C, 2);
                                if (*sp).x44 == (*sp).x3C {
                                    var_r24 += 1;
                                }
                                set_tx((*(*sp).x2C).hsd_obj as *mut HSD_JObj, (*sp).x44 as f32);
                            } else {
                                var_r24 += 1;
                            }
                        } else {
                            let t5i = t5 as i32;
                            crate::mn_ptr::mn_8022F470(&mut (*sp).x48, &(*sp).x40, t5i + sdiv(t5i + 1, 2));
                            if (*sp).x48 == (*sp).x40 {
                                var_r24 += 1;
                            }
                            set_ty((*(*sp).x2C).hsd_obj as *mut HSD_JObj, -((*sp).x48 as f32));
                        }
                    }
                    i += 1;
                }
                if var_r24 == 4 {
                    let tbl = &lbl_803D9E1C[(*tm).entrants as usize];
                    ent(idx).slots[lbl_804D6634 as usize].x3C = tbl[0] as i32;
                    ent(idx).slots[lbl_804D6634 as usize].x40 = tbl[1] as i32;
                    (*tm).cur_option = 0x22;
                }
            } else {
                (*tm).cur_option = 0x22;
            }
            return;
        }
        34 => {
            if (*e).x18 != 0 {
                if (*tm).x33 == 5 {
                    let sp: *mut BracketEntrySlot = &mut (*e).slots[lbl_804D6634 as usize];
                    crate::mn_ptr::mn_8022F470(&mut (*sp).x44, &(*sp).x3C, 2);
                    set_tx((*(*sp).x2C).hsd_obj as *mut HSD_JObj, (*sp).x44 as f32);
                    if (*sp).x44 == (*sp).x3C {
                        (*tm).cur_option = 0x23;
                    }
                } else {
                    lbl_804D6630 = 0;
                    (*tm).cur_option = 0x24;
                }
            } else {
                lbl_804D6630 = 0;
                (*tm).cur_option = 0x24;
            }
            return;
        }
        35 => {
            if (*e).x18 != 0 {
                if (*tm).x33 == 5 {
                    let sp: *mut BracketEntrySlot = &mut (*e).slots[lbl_804D6634 as usize];
                    crate::mn_ptr::mn_8022F470(&mut (*sp).x48, &(*sp).x40, 2);
                    set_ty((*(*sp).x2C).hsd_obj as *mut HSD_JObj, -((*sp).x48 as f32));
                    if (*sp).x48 == (*sp).x40 {
                        (*tm).x33 = 6;
                    }
                } else {
                    lbl_804D6630 = 0;
                    (*tm).cur_option = 0x24;
                }
            } else {
                lbl_804D6630 = 0;
                (*tm).cur_option = 0x24;
            }
            return;
        }
        36 => {
            let h = (*e).x18;
            if h != 0 {
                let c = addr_of_mut!(lbl_803D9DAC);
                (*c).current.x = ((*e).xC + sdiv((*e).x14, 2)) as f32;
                (*c).current.y = -(((*e).x10 + sdiv(h, 2)) as f32);
                (*c).current.z = -150.0;
                (*c).target.x = 320.0;
                (*c).target.y = -240.0;
                (*c).target.z = 0.0;
                let mut d = 0.04 * ((*c).target.x - (*c).current.x);
                if d < 0.0 {
                    d = -d;
                }
                (*c).step.x = d;
                d = 0.04 * ((*c).target.y - (*c).current.y);
                if d < 0.0 {
                    d = -d;
                }
                (*c).step.y = d;
                d = 0.04 * ((*c).target.z - (*c).current.z);
                if d < 0.0 {
                    d = -d;
                }
                (*c).step.z = d;
            }
            (*tm).cur_option = 0x25;
            return;
        }
        37 => {
            if (*e).x18 == 0 {
                lbl_804D6630 = 0x78;
            }
            if lbl_804D6630 < 0x78 {
                lbl_804D6630 += 1;
                fn_8018B090_inline8(addr_of_mut!(lbl_803D9DAC));
                return;
            }
            lbl_804D6630 = 0;
            fn_801904D0();
            (*tm).cur_option = 0x26;
            return;
        }
        38 => {
            if (*e).x18 == 0 {
                lbl_804D6630 = 0x78;
            }
            if lbl_804D6630 < 0x78 {
                lbl_804D6630 += 1;
                return;
            }
            let slot_idx = crate::gm_ptr::fn_8018B090_inline4(&*e);
            let src = entries.add(fn_8018F74C() as usize);
            fn_8018B090_inline5(entries, src, slot_idx);
            (*tm).cur_option = 0x27;
            return;
        }
        _ => return,
    }

    // case 22 falls through into case 23
    if fall {
        if (*e).x18 != 0 {
            let amount = sdiv(lbl_804D6630, 6);
            lbl_804D6630 += 1;
            let c = addr_of_mut!(lbl_803D9DAC);
            if crate::mn_ptr::mn_8022F410(&mut (*c).current.x, &(*c).target.x, (*c).step.x) < 0 {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.x, &(*c).target.x, (*c).step.x);
            } else {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.x, &(*c).target.x, (*c).step.x);
            }
            if crate::mn_ptr::mn_8022F410(&mut (*c).current.y, &(*c).target.y, (*c).step.y) < 0 {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.y, &(*c).target.y, (*c).step.y);
            } else {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.y, &(*c).target.y, (*c).step.y);
            }
            if crate::mn_ptr::mn_8022F410(&mut (*c).current.z, &(*c).target.z, (*c).step.z) < 0 {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.z, &(*c).target.z, (*c).step.z);
            } else {
                crate::mn_ptr::mn_8022F410(&mut (*c).current.z, &(*c).target.z, (*c).step.z);
            }
            fn_80190520((*c).current.x, (*c).current.y, (*c).current.z);
            if (*e).x4 != 1 {
                k = 0;
                while k < 4 {
                    if (*e).slots[k as usize].x30 != 0 {
                        crate::mn_ptr::mn_8022F470(&mut (*e).slots[k as usize].x48, &(*e).slots[k as usize].x40, amount);
                        let y = -((*e).slots[k as usize].x48 as f32);
                        let jobj2 = (*(*e).slots[k as usize].x2C).hsd_obj as *mut HSD_JObj;
                        set_ty(jobj2, y);
                    }
                    k += 1;
                }
                return;
            }
            i = 0;
            while i < 4 {
                if (*e).slots[i as usize].x30 != 0 {
                    (*e).slots[i as usize].x40 = (*e).slots[i as usize].x48;
                }
                i += 1;
            }
        }
    }
}

/// Draws tournament bracket lines for different bracket types (0-3).
pub unsafe extern "C" fn fn_8018C8D4(data: *mut BracketEntry, arg1: i32, arg2: i32, arg3: i32, arg4: i32, _arg5: i32) {
    melee_core::ct!("fn_8018C8D4");
    let data = &*data;
    let c0 = lbl_804DA67C;
    let thickness = data.x1C;

    match data.x4 as i32 {
        0 => {
            dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, c0);
            if data.x20.g == 0 {
                dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
            }
        }
        1 => {
            dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, c0);
            if data.x20.g == 0 {
                if data.slots[0].x4C == 0 {
                    let half_h = sdiv(arg4, 2);
                    dr!(arg1 as f32, (arg2 + half_h) as f32, thickness, half_h as f32, data.x20);
                    return;
                }
                dr!(arg1 as f32, arg2 as f32, thickness, sdiv(arg4, 2) as f32, data.x20);
            }
        }
        2 => {
            let half = sdiv(arg3, 2);
            let center = arg1 + half;
            let half_h = sdiv(arg4, 2);
            dr!(center as f32, arg2 as f32, thickness, half_h as f32, c0);

            let neg_thickness = -thickness;
            let mid_y = arg2 + half_h;
            dr!((arg1 + 0x2B) as f32, mid_y as f32, (arg3 as f32 + thickness) - 43.0, neg_thickness, c0);

            dr!((arg1 + 0x2B) as f32, mid_y as f32, thickness, half_h as f32, c0);

            let right = arg1 + arg3;
            dr!(right as f32, mid_y as f32, thickness, half_h as f32, c0);

            if data.x20.g == 0 {
                if data.slots[0].x4C == 0 {
                    dr!((arg1 + 0x2B) as f32, mid_y as f32, thickness, half_h as f32, data.x20);
                    dr!((arg1 + 0x2B) as f32, mid_y as f32, (half - 0x2B) as f32, neg_thickness, data.x20);
                    return;
                }
                if data.slots[1].x4C == 0 {
                    dr!(right as f32, mid_y as f32, thickness, half_h as f32, data.x20);
                    dr!(center as f32, mid_y as f32, half as f32 + thickness, neg_thickness, data.x20);
                    return;
                }
                dr!(center as f32, arg2 as f32, thickness, half_h as f32, data.x20);
            }
        }
        3 => {
            let half = sdiv(arg3, 2);
            let third_h = sdiv(arg4, 3);

            dr!((arg1 + half) as f32, (arg2 + third_h) as f32, thickness, (third_h - 2) as f32, c0);
            dr!(arg1 as f32, arg2 as f32, thickness, third_h as f32, c0);

            let right = arg1 + arg3;
            dr!(right as f32, arg2 as f32, thickness, third_h as f32, c0);

            let bot_y = (arg2 + arg4) - third_h;
            dr!(arg1 as f32, bot_y as f32, thickness, third_h as f32, c0);
            dr!(right as f32, bot_y as f32, thickness, third_h as f32, c0);

            let neg_thickness = -thickness;
            dr!(arg1 as f32, (arg2 + third_h) as f32, arg3 as f32 + thickness, neg_thickness, c0);
            dr!(arg1 as f32, bot_y as f32, arg3 as f32 + thickness, neg_thickness, c0);

            if data.x20.g == 0 {
                if data.slots[0].x4C == 0 {
                    let two_third_y = arg2 + sdiv(arg4 * 2, 3);
                    dr!(arg1 as f32, two_third_y as f32, thickness, third_h as f32, data.x20);
                    dr!(arg1 as f32, two_third_y as f32, half as f32, neg_thickness, data.x20);
                    dr!((arg1 + half) as f32, (arg2 + sdiv(arg4, 2)) as f32, thickness, (sdiv(arg4, 6) - 1) as f32, data.x20);
                    return;
                }
                if data.slots[1].x4C == 0 {
                    let two_third_y = arg2 + sdiv(arg4 * 2, 3);
                    dr!(right as f32, two_third_y as f32, thickness, third_h as f32, data.x20);
                    dr!((arg1 + half) as f32, two_third_y as f32, half as f32, neg_thickness, data.x20);
                    dr!((arg1 + half) as f32, (arg2 + sdiv(arg4, 2)) as f32, thickness, (sdiv(arg4, 6) - 1) as f32, data.x20);
                    return;
                }
                if data.slots[2].x4C == 0 {
                    dr!(arg1 as f32, arg2 as f32, thickness, third_h as f32, data.x20);
                    dr!(arg1 as f32, (arg2 + third_h) as f32, half as f32, neg_thickness, data.x20);
                    dr!((arg1 + half) as f32, (arg2 + third_h) as f32, thickness, (sdiv(arg4, 6) - 1) as f32, data.x20);
                    return;
                }
                dr!(right as f32, arg2 as f32, thickness, third_h as f32, data.x20);
                dr!((arg1 + half) as f32, (arg2 + third_h) as f32, half as f32, neg_thickness, data.x20);
                dr!((arg1 + half) as f32, (arg2 + third_h) as f32, thickness, (sdiv(arg4, 6) - 1) as f32, data.x20);
            }
        }
        _ => {}
    }
}

/// Draws tournament bracket connector lines with optional tail segments.
pub unsafe extern "C" fn fn_8018D50C(data: *mut BracketEntry, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    melee_core::ct!("fn_8018D50C");
    let data = &*data;
    let tm = gm_GetTournamentData();
    let c0 = lbl_804DA684;
    let thickness = data.x1C;
    dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, c0);

    let right = arg1 + arg3;
    dr!(right as f32, arg2 as f32, thickness, arg4 as f32, c0);

    let neg_thickness = -thickness;
    dr!(arg1 as f32, arg5 as f32, arg3 as f32 + thickness, neg_thickness, c0);

    if data.x20.g == 0 {
        if data.slots[0].x4C == 0 {
            dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
            dr!(arg1 as f32, arg5 as f32, sdiv(arg3, 2) as f32 + thickness, neg_thickness, data.x20);
        } else {
            dr!(right as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
            let half2 = sdiv(arg3, 2);
            dr!((arg1 + half2) as f32, arg5 as f32, half2 as f32 + thickness, neg_thickness, data.x20);
        }
    }

    let entrants = (*tm).entrants;
    if entrants == 1 {
        if data.slots[1].x32 != 0 {
            let bottom = arg2 + arg4;
            dr!(right as f32, bottom as f32, thickness, -70.0, c0);
            if data.x20.g == 0 && data.slots[0].x4C != 0 {
                dr!(right as f32, bottom as f32, thickness, -70.0, data.x20);
            }
        }
    } else if entrants == 3 {
        if data.slots[0].x32 != 0 {
            let bottom = arg2 + arg4;
            dr!(arg1 as f32, bottom as f32, thickness, -60.0, c0);
            if data.x20.g == 0 && data.slots[0].x4C == 0 {
                dr!(arg1 as f32, bottom as f32, thickness, -60.0, data.x20);
            }
        } else if data.slots[1].x32 != 0 {
            let bottom = arg2 + arg4;
            dr!(right as f32, bottom as f32, thickness, -60.0, c0);
            if data.x20.g == 0 && data.slots[0].x4C != 0 {
                dr!(right as f32, bottom as f32, thickness, -60.0, data.x20);
            }
        }
    } else if entrants == 5 {
        if data.slots[1].x32 != 0 {
            let bottom = arg2 + arg4;
            dr!(right as f32, bottom as f32, thickness, -40.0, c0);
            if data.x20.g == 0 && data.slots[0].x4C != 0 {
                dr!(right as f32, bottom as f32, thickness, -40.0, data.x20);
            }
        }
    } else if entrants == 7 {
        if data.slots[1].x32 != 0 {
            if data.x2 == 0 {
                let bottom = arg2 + arg4;
                dr!(right as f32, bottom as f32, thickness, -30.0, c0);
                if data.x20.g == 0 && data.slots[0].x4C != 0 {
                    dr!(right as f32, bottom as f32, thickness, -30.0, data.x20);
                }
            } else {
                dr!(right as f32, (arg2 + 0x1E) as f32, thickness, -30.0, c0);
                if data.x20.g == 0 && data.slots[0].x4C != 0 {
                    dr!(right as f32, (arg2 + 0x1E) as f32, thickness, -30.0, data.x20);
                }
            }
        }
    }
}

pub unsafe extern "C" fn fn_8018DC18(data: *mut BracketEntry, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    melee_core::ct!("fn_8018DC18");
    let data = &*data;
    let c0 = col;
    let thickness = data.x1C;
    dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, c0);

    let right = arg1 + arg3;
    dr!(right as f32, arg2 as f32, thickness, arg4 as f32, c0);

    let half = sdiv(arg3, 2);
    let center = arg1 + half;
    dr!(center as f32, arg2 as f32, thickness, arg4 as f32, c0);

    let neg_thickness = -thickness;
    dr!(arg1 as f32, arg5 as f32, arg3 as f32 + thickness, neg_thickness, c0);

    if data.x20.g == 0 {
        if data.slots[0].x4C == 0 {
            dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
            dr!(arg1 as f32, arg5 as f32, half as f32 + thickness, neg_thickness, data.x20);
            return;
        }
        if data.slots[1].x4C == 0 {
            dr!(center as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
            return;
        }
        dr!(right as f32, arg2 as f32, thickness, arg4 as f32, data.x20);
        dr!(center as f32, arg5 as f32, half as f32 + thickness, neg_thickness, data.x20);
    }
}

pub unsafe extern "C" fn fn_8018DF68(arg0: *mut BracketEntry, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32) {
    melee_core::ct!("fn_8018DF68");
    let arg0 = &*arg0;
    let line_color = lbl_804DA69C;
    let thickness = arg0.x1C;
    dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, line_color);

    let right = arg1 + arg3;
    dr!(right as f32, arg2 as f32, thickness, arg4 as f32, line_color);

    let third = sdiv(arg3, 3);
    {
        let x = (arg1 + third) as f32;
        dr!(x, arg2 as f32, thickness, arg4 as f32, line_color);
    }

    let right_third = right - third;
    dr!(right_third as f32, arg2 as f32, thickness, arg4 as f32, line_color);

    let neg_thickness = -thickness;
    dr!(arg1 as f32, arg5 as f32, arg3 as f32 + thickness, neg_thickness, line_color);

    if arg0.x20.g == 0 {
        if arg0.slots[0].x4C == 0 {
            dr!(arg1 as f32, arg2 as f32, thickness, arg4 as f32, arg0.x20);
            dr!(arg1 as f32, arg5 as f32, sdiv(arg3, 2) as f32 + thickness, neg_thickness, arg0.x20);
            return;
        }
        if arg0.slots[1].x4C == 0 {
            dr!((arg1 + third) as f32, arg2 as f32, thickness, arg4 as f32, arg0.x20);
            dr!(
                (arg1 + third) as f32,
                arg5 as f32,
                (sdiv(arg3, 2) as f32 + thickness) - third as f32,
                neg_thickness,
                arg0.x20
            );
            return;
        }
        if arg0.slots[2].x4C == 0 {
            dr!(right_third as f32, arg2 as f32, thickness, arg4 as f32, arg0.x20);
            let half = sdiv(arg3, 2);
            dr!((arg1 + half) as f32, arg5 as f32, (half as f32 + thickness) - third as f32, neg_thickness, arg0.x20);
            return;
        }
        dr!(right as f32, arg2 as f32, thickness, arg4 as f32, arg0.x20);
        let half = sdiv(arg3, 2);
        dr!((arg1 + half) as f32, arg5 as f32, half as f32 + thickness, neg_thickness, arg0.x20);
    }
}

pub unsafe extern "C" fn fn_8018E46C(gobj: *mut HSD_GObj, _unused: isize) {
    melee_core::ct!("fn_8018E46C");
    let data = (*gobj).user_data as *mut BracketEntry;
    (*data).x3 = (*data).x3;

    let temp = if (*data).x2 != 0 { -((*data).x10 + (*data).x18) } else { -(*data).x10 };
    let r30 = temp;
    ext::HSD_FogSet(null_mut());
    ext::hsd_80391A04(1.0, 1.0, 1);
    let x = (*data).xC - f2i32((0.5f32 * (*data).x1C) as f64);
    match (*data).x3 {
        0 => fn_8018C8D4(data, x, -(*data).x10, (*data).x14, -(*data).x18, r30),
        1 => fn_8018D50C(data, x, -(*data).x10, (*data).x14, -(*data).x18, r30),
        2 => fn_8018DC18(data, x, -(*data).x10, (*data).x14, -(*data).x18, r30),
        3 => fn_8018DF68(data, x, -(*data).x10, (*data).x14, -(*data).x18, r30),
        _ => {}
    }
}

unsafe fn gmTournament_InitBracket(entrant_count: i32, anim_frame: f32, reset: bool) {
    if reset {
        fn_8018A514(entrant_count, anim_frame);
    }
    fn_8018A970(entrant_count);
}

/// Initializes the tournament bracket camera and optionally resets bracket data.
pub unsafe extern "C" fn fn_8018E618(arg0: i32, farg0: f32, arg1: i32) {
    melee_core::ct!("fn_8018E618");
    let mut cam: HSD_CameraDescPerspective = cobj_desc;

    loop {
        let cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_27 as usize);
        if cur.is_null() {
            break;
        }
        ext::HSD_GObjFree(cur);
    }
    loop {
        let cur = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_20 as usize);
        if cur.is_null() {
            break;
        }
        ext::HSD_GObjFree(cur);
    }

    let mut i = 0;
    while i < 0x40 {
        if arg1 != 0 {
            let e = ent(i);
            e.x0 = 0;
            e.slots[0].x4E = 3;
            e.slots[1].x4E = 3;
            e.slots[2].x4E = 3;
            e.slots[3].x4E = 3;
        }
        i += 1;
    }

    let gobj = ext::GObj_Create(9, 20, 1);
    {
        let cobj = ext::HSD_CObjLoadDesc(&mut cam as *mut HSD_CameraDescPerspective as *mut HSD_CObjDesc);
        lbl_803D9DD0.cobj = cobj;
        let cobj_ptr: *mut *mut HSD_CObj = addr_of_mut!(lbl_803D9DD0.cobj);
        let kind_ptr: *mut u8 = addr_of_mut!(ext::HSD_GObj_CameraKind);
        ext::HSD_GObjObject_80390A70(gobj, *kind_ptr, *cobj_ptr as *mut c_void);
    }
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 1);
    // ((u32*) &gobj->gxlink_prios)[1] = 0x10; [0] = 0. On the GC (big-endian) [1] is the low word;
    // the native little-endian C build stores it in the HIGH word, and the port matches the C build.
    (*gobj).gxlink_prios = 0x10u64 << 32;

    gmTournament_InitBracket(arg0, farg0, arg1 != 0);
}

pub unsafe extern "C" fn fn_8018E85C(model: *mut DynamicModelDesc, flag: i32) {
    melee_core::ct!("fn_8018E85C");
    let td = gm_GetTournamentData();
    let tdb = td as *mut u8;
    let mut bracket_idx: i32 = 0;

    let mut outer_idx = 0;
    while outer_idx < 0x40 {
        if ent(outer_idx).x0 == 0 {
            outer_idx += 1;
            continue;
        }
        let mut inner_idx = 0;
        while inner_idx < 4 {
            let sub: *mut BracketEntrySlot = &mut ent(outer_idx).slots[inner_idx as usize];
            if (*sub).x30 == 0 {
                inner_idx += 1;
                continue;
            }

            if flag != 0 {
                let mut j: i32 = 0;
                while j < 64 {
                    if *tdb.add((0x44 + j * 0x12) as usize) as i32 == bracket_idx {
                        break;
                    }
                    j += 1;
                }
                (*sub).x50 = j as u8;
                let ptr = tdb.add((j * 0x12) as usize);
                (*sub).x4D = *ptr.add(0x3A);
                (*sub).x4E = *ptr.add(0x37);
                (*sub).x4F = *ptr.add(0x3E);
                (*sub).x51 = *ptr.add(0x38);
                (*sub).x52 = *ptr.add(0x39);
                (*sub).x54 = core::ptr::read_unaligned(ptr.add(0x40) as *const u16);
                bracket_idx += 1;
            }

            let gobj = ext::GObj_Create(0xE, 0x1B, 0);
            (*sub).x2C = gobj;
            let jobj = ext::HSD_JObjLoadJoint((*model).joint);
            ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
            ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 4, 2);
            ext::gm_8016895C(jobj, model, 0);

            let anim_frame = (*sub).x4D as i32 + (*sub).x4F as i32 * 0x1E;
            ext::HSD_JObjReqAnimAll(jobj, anim_frame as f32);
            ext::HSD_JObjAnimAll(jobj);

            let pos_multiplier: f32 = if outer_idx == fn_8018F74C() { 10.0 } else { 7.0 };

            let mut pos = fmadds(0.0083, (0x40 - (*td).x2E as i32) as f32, 1.0);
            let mut final_pos = pos_multiplier * pos;
            set_sx(jobj, final_pos);

            pos = fmadds(0.0083, (0x40 - (*td).x2E as i32) as f32, 1.0);
            final_pos = pos_multiplier * pos;
            set_sy(jobj, final_pos);

            if (*td).cur_option < 0x1F {
                fn_8018AA74(jobj, outer_idx, inner_idx);
            } else {
                fn_8018FDC4(jobj, (*sub).x44 as f32, -((*sub).x48 as f32), 666.0);
            }
            inner_idx += 1;
        }
        outer_idx += 1;
    }
}

pub unsafe extern "C" fn fn_8018EC48() {
    melee_core::ct!("fn_8018EC48");
    ext::mn_8022F138(0x19, 0x1C);
    ext::mn_8022F138(0x12, 0x15);
}

pub unsafe extern "C" fn fn_8018EC7C() {
    melee_core::ct!("fn_8018EC7C");
    ext::mn_8022F0F0(0x1B);
    ext::mn_8022F0F0(0x14);
}

#[inline]
unsafe fn us() -> usize {
    (ext::lbLang_IsSavedLanguageUS() != 0) as usize
}

pub unsafe extern "C" fn fn_8018ECA8(char_id: i32, name_type: i32, jobj_idx1: i32, pos_x: f32, pos_y: f32, jobj_idx2: i32) {
    melee_core::ct!("fn_8018ECA8");
    let tm = gm_GetTournamentData();
    let mut hmn_texts: [*mut u8; 2] = [null_mut(); 2];
    let mut cpu_texts: [*mut u8; 2] = [null_mut(); 2];

    hmn_texts[0] = lbl_804DA6B4;
    cpu_texts[0] = lbl_804DA6BC;
    hmn_texts[1] = lbl_804DA6B8;
    cpu_texts[1] = lbl_804DA6C0;

    if name_type == 0xFF {
        ext::HSD_SisLib_803A6B98(*(*tm).x518.as_ptr().add(jobj_idx1 as usize), pos_x, pos_y, ext::GetNameText(char_id as u8 as i32));
        return;
    }

    if char_id >= 0x320 {
        if char_id < 0x384 {
            let num = char_id - 0x320;
            if num >= 10 {
                *hmn_texts[us()].add(7) = (num / 10 + 0x30) as i8 as u8;
                *hmn_texts[us()].add(8) = (num % 10 + 0x30) as i8 as u8;
                *hmn_texts[us()].add(9) = 0;
            } else {
                *hmn_texts[us()].add(7) = (char_id - 0x2F0) as i8 as u8;
                *hmn_texts[us()].add(8) = 0;
            }
            ext::HSD_SisLib_803A6B98(*(*tm).x518.as_ptr().add(jobj_idx1 as usize), pos_x, pos_y, hmn_texts[us()] as *const c_char);
            return;
        }
        if char_id >= 0x3E7 {
            return;
        }
        let num = char_id - 0x384;
        if num >= 10 {
            *cpu_texts[us()].add(7) = (num / 10 + 0x30) as i8 as u8;
            *cpu_texts[us()].add(8) = (num % 10 + 0x30) as i8 as u8;
            *cpu_texts[us()].add(9) = 0;
        } else {
            *cpu_texts[us()].add(7) = (char_id - 0x354) as i8 as u8;
            *cpu_texts[us()].add(8) = 0;
        }
        ext::HSD_SisLib_803A6B98(*(*tm).x518.as_ptr().add(jobj_idx2 as usize), pos_x, pos_y, cpu_texts[us()] as *const c_char);
        return;
    }

    if name_type == 0 {
        ext::HSD_SisLib_803A6B98(*(*tm).x518.as_ptr().add(jobj_idx1 as usize), pos_x, pos_y, ext::GetNameText(char_id as u8 as i32));
    } else if name_type == 1 {
        ext::HSD_SisLib_803A6B98(*(*tm).x518.as_ptr().add(jobj_idx2 as usize), pos_x, pos_y, ext::GetNameText(char_id as u8 as i32));
    }
}

#[inline]
unsafe fn c_strcpy(dst: *mut u8, mut src: *const u8) {
    let mut d = dst;
    loop {
        *d = *src;
        if *src == 0 {
            break;
        }
        d = d.add(1);
        src = src.add(1);
    }
}

/// Formats a tournament slot display name into a destination buffer.
pub unsafe extern "C" fn fn_8018F00C(dest: *mut c_char, slot_id: i32) {
    melee_core::ct!("fn_8018F00C");
    let templates_800: [*mut u8; 2] = [lbl_804DA6C4, lbl_804DA6C8];
    let templates_900: [*mut u8; 2] = [lbl_804DA6CC, lbl_804DA6D0];

    let tmpl_800 = templates_800[us()];
    let tmpl_900 = templates_900[us()];

    if slot_id >= 0x320 {
        if slot_id < 0x384 {
            let diff = slot_id - 0x320;
            if diff < 10 {
                *tmpl_800.add(7) = (slot_id - 0x2F0) as i8 as u8;
                *tmpl_800.add(8) = 0;
            } else {
                *tmpl_800.add(7) = (diff / 10 + 0x30) as i8 as u8;
                *tmpl_800.add(8) = (diff % 10 + 0x30) as i8 as u8;
                *tmpl_800.add(9) = 0;
            }
            c_strcpy(dest as *mut u8, tmpl_800);
        } else if slot_id != 0x3E7 {
            let diff = slot_id - 0x384;
            if diff < 10 {
                *tmpl_900.add(7) = (slot_id - 0x354) as i8 as u8;
                *tmpl_900.add(8) = 0;
            } else {
                *tmpl_900.add(7) = (diff / 10 + 0x30) as i8 as u8;
                *tmpl_900.add(8) = (diff % 10 + 0x30) as i8 as u8;
                *tmpl_900.add(9) = 0;
            }
            c_strcpy(dest as *mut u8, tmpl_900);
        }
    } else {
        c_strcpy(dest as *mut u8, ext::GetNameText(slot_id as u8 as i32) as *const u8);
    }
}

pub unsafe extern "C" fn gm_8018F1B0(me: *mut MatchEnd) -> i32 {
    melee_core::ct!("gm_8018F1B0");
    let tm = gm_GetTournamentData();

    match gm_804771C4.match_type {
        0 => return 1,
        1 => match (*tm).hmn_cpu_count {
            1 => return 1,
            2 => {
                if (*tm).x30 == 3 {
                    if (*me).n_winners == 3 {
                        return 1;
                    }
                } else if (*tm).x30 == 4 {
                    if (*me).n_winners >= 3 {
                        return 1;
                    }
                }
            }
            3 => {
                if (*me).n_winners == 4 {
                    return 1;
                }
            }
            _ => {}
        },
        2 => match (*tm).hmn_cpu_count {
            1 => {
                if (*tm).x30 == (*me).n_winners {
                    return 1;
                }
            }
            2 => {
                if (*tm).x30 == 3 {
                    return 1;
                } else if (*tm).x30 == 4 {
                    if (*me).n_winners >= 3 {
                        return 1;
                    }
                }
            }
            3 => return 1,
            _ => {}
        },
        _ => {}
    }
    0
}

pub unsafe extern "C" fn fn_8018F310(arg0: i32) -> i32 {
    melee_core::ct!("fn_8018F310");
    let t = &(*addr_of_mut!(lbl_803D9D20)).x59;
    let mut i = 0;
    while i < t.len() {
        if t[i] as i32 == arg0 {
            return i as i32;
        }
        i += 1;
    }
    -1
}

pub unsafe extern "C" fn fn_8018F3BC(arg0: i32) -> i32 {
    melee_core::ct!("fn_8018F3BC");
    (*(addr_of_mut!(lbl_803D9D20) as *const u8).offset(0x59 + arg0 as isize)) as i32
}

pub unsafe extern "C" fn fn_8018F410() -> i32 {
    melee_core::ct!("fn_8018F410");
    let mut char_id: i32;
    loop {
        char_id = ext::HSD_Randi(0x19);
        let a = (*addr_of_mut!(lbl_803D9D20)).x72[char_id as usize] == 0;
        if !(a || lbl_803D9F0C.x0 == char_id || lbl_803D9F0C.x4 == char_id || lbl_803D9F0C.x8 == char_id) {
            break;
        }
    }
    lbl_803D9F0C.x8 = lbl_803D9F0C.x4;
    lbl_803D9F0C.x4 = lbl_803D9F0C.x0;
    lbl_803D9F0C.x0 = char_id;
    char_id
}

pub unsafe extern "C" fn fn_8018F4A0() -> i32 {
    melee_core::ct!("fn_8018F4A0");
    let temp_r3 = ext::mnSelStageRandom();
    if ext::gm_80164430(temp_r3 as u16) == 0 {
        ext::printf(b"This is impossible stage num from mnSelStageRandom() -> stage %d \n\0".as_ptr() as *const c_char, temp_r3);
        ext::__assert(b"src/melee/gm/gmtoulib.c\0".as_ptr() as *const c_char, 2085, b"0\0".as_ptr() as *const c_char);
    }
    temp_r3
}

/// Counts available tournament slots and returns the last found index.
pub unsafe extern "C" fn fn_8018F508(out_index: *mut i32) -> i32 {
    melee_core::ct!("fn_8018F508");
    let mut count: i32 = 0;
    let e = ent(fn_8018F74C());

    if e.x0 == 0 {
        return -1;
    }

    let mut i = 0;
    while i < 4 {
        if e.slots[i as usize].x4E != 3 {
            if !out_index.is_null() {
                *out_index = i;
            }
            count += 1;
        }
        i += 1;
    }
    count
}

pub unsafe extern "C" fn fn_8018F5F0() -> *mut c_char {
    melee_core::ct!("fn_8018F5F0");
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        b"SdTou.usd\0".as_ptr() as *mut c_char
    } else {
        b"SdTou.dat\0".as_ptr() as *mut c_char
    }
}

pub unsafe extern "C" fn gm_GetTournamentData() -> *mut TmData {
    melee_core::ct!("gm_GetTournamentData");
    addr_of_mut!(gm_804771C4)
}

pub unsafe extern "C" fn fn_8018F640(arg0: i32) -> u32 {
    melee_core::ct!("fn_8018F640");
    let a = if arg0 >= 4 { 4 } else { arg0 };
    ext::gm_GetButtonsTriggered(a as u8) as u32
}

pub unsafe extern "C" fn fn_8018F674(arg0: i32) -> u32 {
    melee_core::ct!("fn_8018F674");
    let a = if arg0 >= 4 { 4 } else { arg0 };
    ext::gm_801A36C0(a as u8) as u32
}

pub unsafe extern "C" fn fn_8018F6A8(arg0: i32) -> u32 {
    melee_core::ct!("fn_8018F6A8");
    let a = if arg0 >= 4 { 4 } else { arg0 };
    ext::gm_GetButtonsPressed(a as u8) as u32
}

pub unsafe extern "C" fn fn_8018F74C() -> i32 {
    melee_core::ct!("fn_8018F74C");
    let mut i = 0;
    while i < 0x40 {
        if ent(i).x1 != 0 {
            break;
        }
        i += 1;
    }
    i
}

pub unsafe extern "C" fn fn_8018F808() -> i32 {
    melee_core::ct!("fn_8018F808");
    let mut noerrcount = 0;
    let mut i = 0;
    while i < 4 {
        if ext::HSD_PadMasterStatus[i as u8 as usize].err == 0 {
            noerrcount += 1;
        }
        i += 1;
    }
    noerrcount
}

unsafe fn fn_8018F888_inline0() -> i32 {
    fn_8018F74C()
}

pub unsafe extern "C" fn fn_8018F888() {
    melee_core::ct!("fn_8018F888");
    let mut i = fn_8018F888_inline0();

    // TARGET_PC: skip the write when no entry has x1 set (index 0x40 is past the array)
    if i < 0x40 {
        ent(i).x20.g = 0;
    }

    if gm_804771C4.x33 != 5 {
        return;
    }

    i = fn_8018F888_inline0();

    if i < 0x3F {
        ent(i + 1).x20.g = 0;
    }
}

#[inline]
fn fn_8018FA24_inline0(char_kind: i32) -> i32 {
    if char_kind < CKind_Seak as i32 {
        return char_kind;
    }
    if char_kind == CKind_GKoops as i32 {
        return 5;
    }
    char_kind + 1
}

pub unsafe extern "C" fn fn_8018FA24() {
    melee_core::ct!("fn_8018FA24");
    let tmdata = addr_of_mut!(gm_804771C4) as *mut u8;

    let mut i = 0;
    while i < 64 {
        if ent(i).x1 != 0 {
            break;
        }
        i += 1;
    }

    let entry = ent(i);
    let mut dst = tmdata;
    let mut player_count: i32 = 0;

    let mut player_idx = 0;
    while player_idx < 4 {
        let s = &entry.slots[player_idx as usize];
        *dst.add(0x4B6) = s.x30;
        *dst.add(0x4B7) = s.x50;
        core::ptr::write_unaligned(
            dst.add(0x4BE) as *mut u16,
            core::ptr::read_unaligned(tmdata.add(0x40 + s.x50 as usize * 0x12) as *const u16),
        );
        *dst.add(0x4BC) = s.x51;
        *dst.add(0x4BD) = s.x52;
        *dst.add(0x4B9) = s.x4D;
        ext::Player_SetPlayerCharacter(player_idx, fn_8018FA24_inline0(*dst.add(0x4B9) as i32) as CharacterKind);
        *dst.add(0x4B8) = s.x4E;
        if *dst.add(0x4B8) != 3 {
            player_count += 1;
        }
        ext::Player_SetSlottype(player_idx, *dst.add(0x4B8) as Gm_PKind);
        *dst.add(0x4BB) = s.x4F;
        ext::Player_SetCostumeId(player_idx, *dst.add(0x4BB) as i32);
        *dst.add(0x4BC) = s.x51;
        dst = dst.add(0xA);
        player_idx += 1;
    }

    *tmdata.add(0x30) = player_count as u8;
}

pub unsafe extern "C" fn fn_8018FBE0(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32, arg6: i32) {
    melee_core::ct!("fn_8018FBE0");
    let t = addr_of_mut!(gm_804771C4);
    (*t).cur_option = arg0;
    (*t).x1C = arg1;
    (*t).x20 = arg2;

    let mut i: i32 = 0;
    while 64 > i {
        let m = addr_of_mut!((*t).x37[i as usize]) as *mut u8;
        // TmUnkMenuData is packed (0x12 bytes): x2, x1, xD, x9 (u16), x0
        *m.add(2) = arg3 as u8;
        *m.add(1) = arg4 as u8;
        *m.add(0xD) = i as u8;
        core::ptr::write_unaligned(m.add(9) as *mut u16, arg5 as u16);
        *m = arg6 as u8;
        i += 1;
    }
}

pub unsafe extern "C" fn fn_8018FDC4(jobj: *mut HSD_JObj, x: f32, y: f32, z: f32) {
    melee_core::ct!("fn_8018FDC4");
    if f2i32(x as f64) != 666 {
        set_tx(jobj, x);
    }
    if f2i32(y as f64) != 666 {
        set_ty(jobj, y);
    }
    if f2i32(z as f64) != 666 {
        set_tz(jobj, z);
    }
}

pub unsafe extern "C" fn fn_8018FF9C(jobj: *mut HSD_JObj, x: f32, y: f32, z: f32) {
    melee_core::ct!("fn_8018FF9C");
    if f2i32(x as f64) != 666 {
        set_sx(jobj, x);
    }
    if f2i32(y as f64) != 666 {
        set_sy(jobj, y);
    }
    if f2i32(z as f64) != 666 {
        set_sz(jobj, z);
    }
}

pub unsafe extern "C" fn fn_80190174(cobjdesc: *mut HSD_CObjDesc) -> *mut HSD_GObj {
    melee_core::ct!("fn_80190174");
    let gobj = ext::GObj_Create(0x13, 0x12, 0);
    let cobj = ext::HSD_CObjLoadDesc(cobjdesc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 0);
    (*gobj).gxlink_prios = 7;
    gobj
}

pub unsafe extern "C" fn fn_801901F8(cobjdesc: *mut HSD_CObjDesc) -> *mut HSD_GObj {
    melee_core::ct!("fn_801901F8");
    let gobj = ext::GObj_Create(0x13, 0x15, 2);
    let cobj = ext::HSD_CObjLoadDesc(cobjdesc);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 2);
    (*gobj).gxlink_prios = 0xA;
    gobj
}

pub unsafe extern "C" fn fn_8019027C(lights: *mut c_void) {
    melee_core::ct!("fn_8019027C");
    let gobj = ext::GObj_Create(0xB, 0x1A, 0);
    let lobj = ext::lb_80011AC4(lights as *mut *mut LightList);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 1, 0);
}

/// Initializes SIS library text rendering for tournament mode.
pub unsafe extern "C" fn fn_801902F0(sis_param: i32) {
    melee_core::ct!("fn_801902F0");
    let mut value = sis_param;
    ext::HSD_SisLib_803A5E70();
    if gm_804771C4.x1C == 0 {
        value = 0;
    }
    lbl_804D663C = ext::HSD_SisLib_803A611C(0, value as usize as *mut HSD_GObj, 9, 0x12, 0, 3, 0, 0x13);
}

pub unsafe extern "C" fn fn_8019035C(
    arg0: i32,
    model: *mut DynamicModelDesc,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    arg6: HSD_GObjEvent,
    arg8: f32,
) -> *mut HSD_GObj {
    melee_core::ct!("fn_8019035C");
    let gobj = ext::GObj_Create(0xE, arg3 as u8, 0);
    let jobj = ext::HSD_JObjLoadJoint((*model).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), arg4 as u8, 0);
    if arg6.is_some() {
        ext::HSD_GObj_SetupProc(gobj, arg6, 0);
    }
    if arg5 != 0 {
        ext::gm_8016895C(jobj, model, arg2);
        ext::HSD_JObjReqAnimAll(jobj, arg8);
        ext::HSD_JObjAnimAll(jobj);
        if arg0 != 0 {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
    }
    gobj
}

pub unsafe extern "C" fn fn_8019044C(jobj: *mut HSD_JObj, arg1: f32) {
    melee_core::ct!("fn_8019044C");
    ext::HSD_JObjReqAnimAll(jobj, arg1);
    ext::HSD_JObjAnimAll(jobj);
}

pub unsafe extern "C" fn fn_80190480(arg8: f32) {
    melee_core::ct!("fn_80190480");
    if f2i32(arg8 as f64) == 0 {
        ext::HSD_CObjSetFov(lbl_803D9DD0.cobj, lbl_803D9DD0.xD0);
        return;
    }
    ext::HSD_CObjSetFov(lbl_803D9DD0.cobj, arg8);
}

pub unsafe extern "C" fn fn_801904D0() {
    melee_core::ct!("fn_801904D0");
    let tmp = addr_of_mut!(lbl_803D9DD0);
    ext::HSD_CObjSetInterest((*tmp).cobj, &mut (*addr_of_mut!(lbl_803D9E08)).pos);
    ext::HSD_CObjSetEyePosition((*tmp).cobj, &mut (*addr_of_mut!(lbl_803D9DF4)).pos);
}

pub unsafe extern "C" fn fn_80190520(x: f32, y: f32, z: f32) {
    melee_core::ct!("fn_80190520");
    if f2i32(x as f64) == 0 && f2i32(y as f64) == 0 && f2i32(z as f64) == 0 {
        let tmp = addr_of_mut!(lbl_803D9DD0);
        ext::HSD_CObjSetInterest((*tmp).cobj, &mut (*addr_of_mut!(lbl_803D9E08)).pos);
        ext::HSD_CObjSetEyePosition((*tmp).cobj, &mut (*addr_of_mut!(lbl_803D9DF4)).pos);
        return;
    }
    let tmp = addr_of_mut!(lbl_803D9DD0);
    let mut sp14 = Vec { x, y, z };
    ext::HSD_CObjSetInterest((*tmp).cobj, &mut sp14);
    sp14.z = 415.6922f32 + z;
    ext::HSD_CObjSetEyePosition((*tmp).cobj, &mut sp14);
}

#[inline]
fn gm_801905F0_inline0(c_kind: i32) -> i32 {
    if c_kind < CKind_Seak as i32 {
        return c_kind;
    }
    if c_kind == CKind_GKoops as i32 {
        return CKind_Koopa as i32;
    }
    c_kind + 1
}

pub unsafe extern "C" fn gm_801905F0(arg0: *mut StartMeleeData) {
    melee_core::ct!("gm_801905F0");
    let tm = addr_of_mut!(gm_804771C4);
    let rules = ext::gmMainLib_GetGameRules();
    let mut sp18: TmVsData = core::mem::zeroed();

    ext::gm_LoadAnnouncer();
    ext::gm_SetupRulesDefaults(&mut (*arg0).rules);
    (*arg0).rules.is_teams = 0;
    (*arg0).rules.stkind = (*tm).x28 as u16;
    ext::fn_801640B0(&mut (*arg0).rules.x20);
    (*arg0).rules.set_match_kind((*rules).mode as u32);
    if (*rules).mode != 1 {
        (*arg0).rules.set_timer_enabled(1);
    } else if (*rules).stock_time_limit != 0 {
        (*arg0).rules.set_timer_enabled(1);
    } else {
        (*arg0).rules.set_timer_enabled(0);
    }
    if (*arg0).rules.timer_enabled() != 0 {
        if (*rules).mode != 1 {
            if (*rules).time_limit == 0 {
                (*arg0).rules.time_limit = 99 * 60;
            } else {
                (*arg0).rules.time_limit = (*rules).time_limit as u32 * 60;
            }
        } else if (*rules).stock_time_limit == 0 {
            (*arg0).rules.time_limit = 99 * 60;
        } else {
            (*arg0).rules.time_limit = (*rules).stock_time_limit as u32 * 60;
        }
    }
    if (*arg0).rules.match_kind() == MatchKind_Stock as u32 {
        (*arg0).rules.set_is_stock(1);
    }
    (*arg0).rules.set_timer_counts_up(0);
    (*arg0).rules.set_x4_2(0);
    (*arg0).rules.set_x4_4(0);
    (*arg0).rules.item_freq = (*ext::gmMainLib_GetGamePrefs()).item_freq as i8;
    (*arg0).rules.set_x2_2(0);
    (*arg0).rules.x18 = 0;
    (*arg0).rules.game_speed = 1.0;
    (*arg0).rules.x30 = crate::gm::fn_801653E8((*rules).damage_ratio);
    (*arg0).rules.set_x3_4(0);
    (*arg0).rules.set_x3_5(0);
    (*arg0).rules.set_x3_2(0);
    (*arg0).rules.set_x3_3(0);
    match ext::gmMainLib_8015ED30() {
        1 => (*arg0).rules.sd_penalty = 0,
        0 => (*arg0).rules.sd_penalty = -1,
        2 => (*arg0).rules.sd_penalty = -2,
        _ => {}
    }
    if (*rules).pause != 0 {
        (*arg0).rules.set_disable_pausing(0);
    } else {
        (*arg0).rules.set_disable_pausing(1);
    }
    if (*rules).score_display != 0 && (*arg0).rules.match_kind() == 0 {
        (*arg0).rules.set_x3_0(1);
    } else {
        (*arg0).rules.set_x3_0(0);
    }
    ext::gm_SetupAllPlayerDefaults((*arg0).players.as_mut_ptr());

    let mut i: usize = 0;
    while i < 4 {
        let p = &mut (*arg0).players[i];
        if (i as u8) < (*tm).x30 {
            p.model_scale = 1.0;
            p.nametag = core::cmp::min((*tm).x4B8[i].x6 as i32, 0x78) as u8;
            if (*tm).x4B8[i].x2 != 0 {
                p.ckind = gm_801905F0_inline0(fn_8018F410()) as i8;
                p.color = ext::HSD_Randi(ext::gm_GetNumCostumesForCKind(p.ckind as u8) as i32) as u8;
            } else {
                p.ckind = gm_801905F0_inline0((*tm).x4B8[i].x1 as i32) as i8;
                p.color = (*tm).x4B8[i].x3;
            }
            p.slot_type = (*tm).x4B8[i].x0;
            p.stocks = (*rules).stock_count as i8;
            p.sub_color = 0;
            p.team = 0xFF;
            p.set_rumble_enabled(ext::gm_RumbleEnabledForPlayer(i as i32, p.nametag as i32) as u8);
            if (*tm).x4B8[i].x0 == 1 {
                p.set_rumble_enabled(0);
            }
            p.cpu_kind = 4;
            p.cpu_level = (*tm).x4B8[i].x4;
            p.damage1 = 0;
            if (*ext::gmMainLib_GetGameRules()).handicap != 0 {
                p.attack_ratio = crate::gm::fn_8016419C((*tm).x4B8[i].x5);
                p.defense_ratio = crate::gm::fn_801641B4((*tm).x4B8[i].x5);
            } else {
                p.attack_ratio = 1.0;
                p.defense_ratio = 1.0;
            }
        } else {
            p.slot_type = Gm_PKind_NA as u8;
        }
        i += 1;
    }

    sp18.stkind = (*arg0).rules.stkind as StKind;
    let mut i = 0;
    while i < 4 {
        sp18.slot_type[i] = (*arg0).players[i].slot_type as Gm_PKind;
        sp18.char_id[i] = (*arg0).players[i].ckind as i32 as u32;
        sp18.color[i] = (*arg0).players[i].color as u32;
        i += 1;
    }

    fn_8019EF08(&mut sp18);
}

use super::gmtou_2::fn_8019EF08;

// ---- folded from gm.rs (consolidation steps 21/22) ----
/// gmtoulib.c `fn_8018F3D0`.
pub fn fn_8018F3D0(arg0: i32) -> i32 {
    melee_core::ct!("fn_8018F3D0");
    if arg0 == 0xE || (0x10..=0x13).contains(&arg0) || arg0 == 0xA {
        return 1;
    }
    if arg0 <= 0xD {
        return 0;
    }
    2
}

/// gmtoulib.c `fn_8018F6DC`.
pub fn fn_8018F6DC(arg0: i32) -> i32 {
    melee_core::ct!("fn_8018F6DC");
    if arg0 >= 0x13 {
        if arg0 == 0x1D {
            return 5;
        }
        return arg0.wrapping_sub(1);
    }
    arg0
}

/// gmtoulib.c `fn_8018F6FC` (CSSIconHud -> CharacterKind).
pub fn fn_8018F6FC(arg0: i32) -> i32 {
    melee_core::ct!("fn_8018F6FC");
    if arg0 >= 0x13 {
        if arg0 == 0x1D {
            return 5;
        }
        return arg0.wrapping_add(1);
    }
    arg0
}

/// gmtoulib.c `fn_8018F71C`: int arithmetic (-fwrapv), then one int->float conversion.
pub fn fn_8018F71C(arg0: i32, arg1: i32) -> f32 {
    melee_core::ct!("fn_8018F71C");
    arg0.wrapping_add(arg1.wrapping_mul(0x1E)) as f32
}

/// gmtoulib.c `fn_8018FA24_inline0_fp`.
pub fn fn_8018FA24_inline0_fp(char_kind: i32) -> i32 {
    melee_core::ct!("fn_8018FA24_inline0_fp");
    if char_kind < CKIND_SEAK {
        return char_kind;
    }
    if char_kind == CKIND_GKOOPS {
        return 5;
    }
    char_kind.wrapping_add(1)
}

/// gmtoulib.c `gm_801905F0_inline0_fp`.
pub fn gm_801905F0_inline0_fp(c_kind: i32) -> i32 {
    melee_core::ct!("gm_801905F0_inline0_fp");
    if c_kind < CKIND_SEAK {
        return c_kind;
    }
    if c_kind == CKIND_GKOOPS {
        return CKIND_KOOPA;
    }
    c_kind.wrapping_add(1)
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmtoulib.c `gmTournament_SetBracketByes`.
pub fn gmTournament_SetBracketByes(entries: &mut [BracketEntry_gm_ptr], entrant_count: i32) {
    melee_core::ct!("gmTournament_SetBracketByes");
    match entrant_count {
        1 => entries[5].slots[1].x32 = 1,
        3 => {
            entries[10].slots[0].x32 = 1;
            entries[11].slots[1].x32 = 1;
        }
        5 => entries[23].slots[1].x32 = 1,
        7 => {
            entries[46].slots[1].x32 = 1;
            entries[47].slots[1].x32 = 1;
        }
        _ => {}
    }
}

/// gmtoulib.c `fn_8018B090_inline4`: first slot whose x4C is 0 (4 if none).
pub fn fn_8018B090_inline4(entry: &BracketEntry_gm_ptr) -> i32 {
    melee_core::ct!("fn_8018B090_inline4");
    entry.slots.iter().position(|s| s.x4C == 0).unwrap_or(4) as i32
}

/// gmtoulib.c `fn_8018F62C`: `(u32) gobj->user_data`.
pub fn fn_8018F62C(gobj: &HSD_GObj_gm_ptr) -> u32 {
    melee_core::ct!("fn_8018F62C");
    gobj.user_data as usize as u32
}

// ---- folded from gm_r10.rs (consolidation steps 21/22) ----
/// gmtoulib.c `gmTournament_SetTripleRightCoords_fp` (static inline).
pub fn gmTournament_SetTripleRightCoords_fp(
    entry: &BracketEntry_gm_r10,
    p3C: &mut i32,
    p44: &mut i32,
    p34: &mut i32,
    p40: &mut i32,
    p48: &mut i32,
    p38: &mut i32,
) {
    melee_core::ct!("gmTournament_SetTripleRightCoords_fp");
    let val1 = entry.xC.wrapping_add(entry.x14);
    *p3C = val1;
    *p44 = val1;
    *p34 = val1;
    let val = entry.x10.wrapping_add(entry.x18);
    *p48 = val;
    *p38 = val;
    *p40 = val.wrapping_sub(entry.x18 / 3);
}

// ---- folded from gm_r9.rs (consolidation steps 21/22) ----
/// gmtoulib.c `fn_8018FBD8`: `gobj->user_data = (void*) arg1` (the s32 sign-extends, as the
/// native C's int-to-pointer cast does).
pub fn fn_8018FBD8(gobj: &mut HSD_GObj_gm_r9, arg1: i32) {
    melee_core::ct!("fn_8018FBD8");
    gobj.user_data = arg1 as isize as *mut c_void;
}
