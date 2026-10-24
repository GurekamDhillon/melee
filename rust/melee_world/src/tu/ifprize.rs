//! Port of src/melee/if/ifprize.c (built `TARGET_PC`): the trophy-prize ("new trophy!") popup scene.
//! Every call out of the TU goes through `ext`.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_ifprize.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = |a0: *mut HSD_CObjDesc| unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(a0 as _) as _ };
    fn HSD_DObjReqAnimAll(a0: *mut HSD_DObj, a1: f32);
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_803910D8(a0 as _, a1 as _); } };
    fn HSD_GObj_JObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_JObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj::HSD_GObj_LObjCallback(a0 as _, a1 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = |a0: *mut HSD_Joint| unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ };
    fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn HSD_SisLib_803A5ACC(a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) -> *mut HSD_Text = |a0: i32, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A5ACC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn HSD_SisLib_803A5F50(a0: i32) = |a0: i32| { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5F50(a0 as _); } };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = |a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ };
    fn HSD_SisLib_803A62A0(a0: i32, a1: *mut i8, a2: *mut i8) = |a0: i32, a1: *mut i8, a2: *mut i8| { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A62A0(a0 as _, a1 as _, a2 as _); } };
    fn HSD_SisLib_803A6368(a0: *mut HSD_Text, a1: i32) = |a0: *mut HSD_Text, a1: i32| { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A6368(a0 as _, a1 as _); } };
    fn HSD_SisLib_803A6530(a0: i32, a1: i32, a2: i32) -> *mut u8 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6530(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_SisLib_803A660C(a0: i32, a1: i32, a2: i32) = |a0: i32, a1: i32, a2: i32| { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A660C(a0 as _, a1 as _, a2 as _); } };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = |a0: i32, a1: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ };
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const u8) -> i32;
    fn HSD_SisLib_803A7664(a0: *mut HSD_Text) = |a0: *mut HSD_Text| { unsafe { let _ = melee_runtime::tu::hsd_3A64::HSD_SisLib_803A7664(a0 as _); } };
    fn Toy_803063D4(a0: i32, a1: i32, a2: i32) -> i32;
    fn Toy_803124BC();
    fn Ui_RetailHidden(a0: i32) -> i32;
    fn gmMainLib_8015D8B0(a0: u32);
    fn gm_8016895C(a0: *mut HSD_JObj, a1: *mut DynamicModelDesc, a2: i32);
    fn gm_801692E8(a0: u32, a1: *mut datetime);
    fn gm_801A4B60();
    fn gm_GetButtonsTriggered(a0: u8) -> u64;
    fn lbArchive_80016DBC(a0: *const u8, a1: *mut c_void, a2: *const u8, rest: &[(*mut c_void, *const u8)]) -> *mut HSD_Archive;
    fn lbArchive_80016EFC(a0: *mut HSD_Archive) = |a0: *mut HSD_Archive| { unsafe { let _ = melee_runtime::tu::lbarchive::lbArchive_80016EFC(a0 as _); } };
    fn lbAudioAx_800236DC() -> i32 = || unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() as _ };
    fn lbAudioAx_80023F28(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ };
    fn lbLang_IsSavedLanguageUS() -> i32 = || unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ };
    fn lb_80011AC4(a0: *mut *mut LightList) -> *mut HSD_LObj = |a0: *mut *mut LightList| unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(a0 as _) as _ };
    fn lb_80011E24(a0: *mut HSD_JObj, a1: *mut *mut HSD_JObj, ids: &[i32]) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
        pub unsafe extern "C" fn HSD_GObj_LObjCallback(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_LObjCallback(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_LightKind: i8 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `AT_RE_HUD_PRIZE` (pc/platform/gw_ui_retail_ids.h).
const AT_RE_HUD_PRIZE: i32 = 6;
const PAD_MAX_CONTROLLERS: u8 = 4;
const PAD_BUTTON_A: u64 = 1 << 8;
const PAD_BUTTON_START: u64 = 1 << 12;

/// `HSD_GObjGetHSDObj` (baselib/gobj.h, static inline).
#[inline]
unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).hsd_obj
}

/// `HSD_JObjGetChild` (baselib/jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}

/// `struct un_803F9B30`: prize id -> text/frame index.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_803F9B30 {
    pub x0: i32,
    pub x4: u16,
}

melee_core::gc_layout!(un_803F9B30, "un_803F9B30", size 8, align 4, [
    x0 @ 0x0: 4 "s32",
    x4 @ 0x4: 2 "u16",
]);

/// `struct un_802FEBE0_OnEnter_arg0`: one prize notice (a linked list through `x8`; the native
/// record is larger than the GameCube's 12 bytes because of the pointer).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_802FEBE0_OnEnter_arg0 {
    pub x0: u16,
    pub x2: u16,
    pub x4: i32,
    pub x8: *mut un_802FEBE0_OnEnter_arg0,
}

melee_core::gc_layout!(un_802FEBE0_OnEnter_arg0, "un_802FEBE0_OnEnter_arg0", size 12, align 4, [
    x0 @ 0x0: 2 "u16",
    x2 @ 0x2: 2 "u16",
    x4 @ 0x4: 4 "s32",
    x8 @ 0x8: 4 "un_802FEBE0_OnEnter_arg0*",
]);

/// `struct un_803F9D48`: the popup's state (GameCube size 0x3C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct un_803F9D48 {
    /// `x0a:4` (high nibble, GameCube bit order) and `x0b:4`
    pub bf_0: u8,
    pub x1: u8,
    pub x2: i8,
    pub x3: u8,
    pub x4: u16,
    pub x6: u16,
    pub x8: u16,
    pub xA: u8,
    pub xB: u8,
    pub xC: i32,
    pub x10: *mut HSD_JObj,
    pub x14: *mut HSD_GObj,
    pub x18: *mut HSD_GObj,
    pub x1C: *mut HSD_GObj,
    pub x20: *mut HSD_Text,
    pub x24: *mut HSD_Text,
    pub x28: *mut un_802FEBE0_OnEnter_arg0,
    pub x2C: *mut un_802FEBE0_OnEnter_arg0,
    pub x30: [i32; 3],
}

impl un_803F9D48 {
    #[inline]
    pub fn x0a(&self) -> u8 {
        self.bf_0 >> 4
    }
    #[inline]
    pub fn set_x0a(&mut self, v: u8) {
        self.bf_0 = (self.bf_0 & 0x0F) | ((v & 0xF) << 4);
    }
    #[inline]
    pub fn x0b(&self) -> u8 {
        self.bf_0 & 0xF
    }
    #[inline]
    pub fn set_x0b(&mut self, v: u8) {
        self.bf_0 = (self.bf_0 & 0xF0) | (v & 0xF);
    }
}

melee_core::gc_layout!(un_803F9D48, "un_803F9D48", size 0x3C, align 4, [
    x0a @ 0x0: 1 "u8" bits 0 + 4,
    x0b @ 0x0: 1 "u8" bits 4 + 4,
    x1 @ 0x1: 1 "u8",
    x2 @ 0x2: 1 "s8",
    x3 @ 0x3: 1 "u8",
    x4 @ 0x4: 2 "u16",
    x6 @ 0x6: 2 "u16",
    x8 @ 0x8: 2 "u16",
    xA @ 0xA: 1 "u8",
    xB @ 0xB: 1 "u8",
    xC @ 0xC: 4 "s32",
    x10 @ 0x10: 4 "HSD_JObj*",
    x14 @ 0x14: 4 "HSD_GObj*",
    x18 @ 0x18: 4 "HSD_GObj*",
    x1C @ 0x1C: 4 "HSD_GObj*",
    x20 @ 0x20: 4 "HSD_Text*",
    x24 @ 0x24: 4 "HSD_Text*",
    x28 @ 0x28: 4 "un_802FEBE0_OnEnter_arg0*",
    x2C @ 0x2C: 4 "un_802FEBE0_OnEnter_arg0*",
    x30 @ 0x30: 12 "s32[3]",
]);

/// 3F9A00
pub static mut un_803F9A00: [i32; 76] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, //
    1, 1, 1, 3, 2, 2, 5, 5, 4, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, //
    8, 8, 8, 8, 8, 8, 8, 8, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, //
    5, 5, 5, 6, 6, 5, 7, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5,
];

const fn p(x0: i32, x4: u16) -> un_803F9B30 {
    un_803F9B30 { x0, x4 }
}

/// 3F9B30
pub static mut un_803F9B30: [un_803F9B30; 67] = [
    p(0, 0), p(1, 1), p(2, 2), p(3, 3), p(4, 4), p(5, 5),
    p(6, 6), p(7, 7), p(8, 8), p(9, 9), p(10, 10), p(11, 11),
    p(12, 12), p(13, 13), p(14, 14), p(15, 15), p(16, 16), p(17, 17),
    p(18, 18), p(19, 19), p(20, 20), p(21, 21), p(22, 22), p(23, 23),
    p(24, 24), p(25, 25), p(26, 26), p(27, 27), p(28, 28), p(29, 29),
    p(30, 30), p(31, 31), p(32, 32), p(33, 33), p(34, 34), p(35, 35),
    p(36, 36), p(37, 37), p(38, 38), p(39, 39), p(40, 40), p(41, 41),
    p(42, 42), p(43, 43), p(44, 44), p(45, 45), p(46, 46), p(47, 47),
    p(48, 48), p(49, 49), p(50, 50), p(51, 51), p(52, 52), p(53, 53),
    p(54, 54), p(55, 55), p(56, 56), p(57, 57), p(58, 58), p(59, 59),
    p(60, 60), p(61, 61), p(62, 63), p(63, 67), p(64, 68), p(65, 62),
    p(66, 0),
];

/// 3F9D48: the popup state.
pub static mut un_803F9D48: un_803F9D48 = un_803F9D48 {
    bf_0: 0,
    x1: 0,
    x2: 0,
    x3: 0,
    x4: 0,
    x6: 0,
    x8: 0,
    xA: 0,
    xB: 0,
    xC: 0,
    x10: null_mut(),
    x14: null_mut(),
    x18: null_mut(),
    x1C: null_mut(),
    x20: null_mut(),
    x24: null_mut(),
    x28: null_mut(),
    x2C: null_mut(),
    x30: [0x43, 0x44, 0x45],
};

/// 4D6D98 .sbss
pub static mut un_804D6D98: *mut HSD_Archive = null_mut();
/// 4D6D9C .sbss
pub static mut un_804D6D9C: *mut SceneDesc = null_mut();

pub unsafe extern "C" fn un_802FE3F8(a: i32, b: i32, c: *mut i16, d: *mut i16) {
    let mut x: *mut un_803F9B30 = &raw mut un_803F9B30[0];
    while (*x).x0 != 66 {
        if (*x).x0 == a {
            if a == 0x3E {
                if !c.is_null() {
                    *c = (b + (*x).x4 as i32) as i16;
                }
                if !d.is_null() {
                    *d = (b + (*x).x4 as i32 + 1) as i16;
                }
            } else if !c.is_null() {
                *c = (b + (*x).x4 as i32) as i16;
            }
        }
        x = x.add(1);
    }
}

#[inline]
pub unsafe extern "C" fn un_802FE3F8_inner(a: i32, b: i32, c: *mut i16, d: *mut i16) {
    un_802FE3F8(a, b, c, d);
}

/// ifPrize_Scene_OnEnter will inline un_802FE3F8 otherwise
#[inline]
pub unsafe extern "C" fn un_802FE3F8_noinline(a: i32, b: i32, c: *mut i16, d: *mut i16) {
    un_802FE3F8_inner(a, b, c, d);
}

/// The prize counter: the Atlas retail mask hides this element's render only (never its proc);
/// empty by default.
pub unsafe extern "C" fn ifPrize_RenderGuarded(gobj: *mut HSD_GObj, pass: i32) {
    if ext::Ui_RetailHidden(AT_RE_HUD_PRIZE) != 0 {
        return;
    }
    ext::HSD_GObj_JObjCallback(gobj, pass);
}

pub unsafe extern "C" fn fn_802FE470(gobj: *mut HSD_GObj) {
    let st = &raw mut un_803F9D48;
    if (*st).x1 != 4 {
        let jobj = HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj;
        match (*st).x1 {
            0 => {
                if ((*st).x2 as i32) < 5 {
                    (*st).x2 = (*st).x2.wrapping_add(1);
                } else {
                    un_802FE918((*(*st).x28).x0 as i32, (*(*st).x28).x2 as i32, (*(*st).x28).x4);
                    (*st).x1 = 1;
                }
            }
            1 => {
                (*st).x2 = 0xA;
                if ext::gm_GetButtonsTriggered(PAD_MAX_CONTROLLERS) & (PAD_BUTTON_A | PAD_BUTTON_START) != 0 {
                    if (*st).x2C.is_null() {
                        un_802FE8CC();
                    } else {
                        un_802FE918((*(*st).x2C).x0 as i32, (*(*st).x2C).x2 as i32, (*(*st).x2C).x4);
                        (*st).x2C = (*(*st).x2C).x8;
                    }
                }
            }
            2 => {
                if ((*st).x2 as i32) < 0xF {
                    (*st).x2 = (*st).x2.wrapping_add(1);
                } else {
                    (*st).x1 = 3;
                }
            }
            3 => {
                ext::HSD_GObjFree((*st).x18);
                ext::HSD_GObjFree((*st).x1C);
                ext::HSD_GObjFree((*st).x14);
                ext::lbArchive_80016EFC(un_804D6D98);
                (*st).x1 = 4;
                (*st).set_x0a(0);
                (*st).set_x0b(0);
                ext::gm_801A4B60();
                return;
            }
            _ => {}
        }
        ext::HSD_JObjReqAnimAll(jobj, (*st).x2 as f32);
        ext::HSD_DObjReqAnimAll((*(*st).x10).u.dobj, (*st).x4 as f32);
        ext::HSD_DObjReqAnimAll((*HSD_JObjGetChild((*st).x10)).u.dobj, (*st).x4 as f32);
        ext::HSD_JObjAnimAll(jobj);
    }
}

pub unsafe extern "C" fn un_802FE6A8() {
    let st = &raw mut un_803F9D48;
    let gobj_camera = ext::GObj_Create(HSD_GOBJ_CLASS_CAMERA as u16, 20, 0);
    ext::HSD_GObjObject_80390A70(
        gobj_camera,
        ext::HSD_GObj_CameraKind,
        ext::HSD_CObjLoadDesc((*(*un_804D6D9C).cameras).desc).cast(),
    );
    ext::GObj_SetupGXLinkMax(gobj_camera, Some(ext::cb::HSD_GObj_803910D8), 8);
    (*gobj_camera).gxlink_prios = 0xC00;
    (*st).x18 = gobj_camera;
    let gobj_light = ext::GObj_Create(HSD_GOBJ_CLASS_LIGHT as u16, 3, 0);
    ext::HSD_GObjObject_80390A70(gobj_light, ext::HSD_GObj_LightKind as u8, ext::lb_80011AC4((*un_804D6D9C).lights).cast());
    ext::GObj_SetupGXLink(gobj_light, Some(ext::cb::HSD_GObj_LObjCallback), 10, 0);
    (*st).x1C = gobj_light;
    let gobj_ui = ext::GObj_Create(HSD_GOBJ_CLASS_UI as u16, 15, 0);
    let jobj_ui = ext::HSD_JObjLoadJoint((**(*un_804D6D9C).models).joint);
    ext::HSD_GObjObject_80390A70(gobj_ui, ext::HSD_GObj_JObjKind, jobj_ui.cast());
    ext::GObj_SetupGXLink(gobj_ui, Some(ifPrize_RenderGuarded), 11, 0);
    ext::HSD_GObj_SetupProc(gobj_ui, Some(fn_802FE470), 17);
    ext::gm_8016895C(jobj_ui, *(*un_804D6D9C).models, 0);
    ext::HSD_JObjReqAnimAll(jobj_ui, 0.0);
    ext::HSD_JObjAnimAll(jobj_ui);
    ext::lb_80011E24(jobj_ui, &raw mut (*st).x10, &[2]);
    (*st).x14 = gobj_ui;
    {
        let color = _GXColor { r: 0x5A, g: 0x5A, b: 0x5A, a: 0xFF };

        ext::HSD_SisLib_803A611C(2, null_mut(), 9, 20, 0, 14, 0, 18);
        (*st).x20 = ext::HSD_SisLib_803A5ACC(2, 0, 105.0, 202.0, 0.0, 435.0, 75.0);
        (*(*st).x20).default_fitting = 1;
        (*(*st).x20).default_alignment = 1;
        (*st).x24 = ext::HSD_SisLib_803A6754(2, 0);
        (*(*st).x24).default_alignment = 1;
        (*(*st).x24).default_kerning = 1;
        (*(*st).x24).text_color = color;
    }
}

pub unsafe extern "C" fn un_802FE8CC() {
    let st = &raw mut un_803F9D48;
    if (*st).x0a() == 0 || (*st).x0b() == 0 {
        return;
    }
    (*st).x1 = 2;
    ext::HSD_SisLib_803A5F50(2);
}

/// Needed for `.data` ordering
pub unsafe extern "C" fn setArchive() {
    un_804D6D98 = ext::lbArchive_80016DBC(
        c"IfPrize".as_ptr().cast(),
        (&raw mut un_804D6D9C).cast(),
        c"ScInfPrize_scene_data".as_ptr().cast(),
        &[],
    );
    if ext::lbLang_IsSavedLanguageUS() != 0 {
        ext::HSD_SisLib_803A62A0(2, c"SdPrize.usd".as_ptr() as *mut i8, c"SIS_PrizeData".as_ptr() as *mut i8);
    } else {
        ext::HSD_SisLib_803A62A0(2, c"SdPrize.dat".as_ptr() as *mut i8, c"SIS_PrizeData".as_ptr() as *mut i8);
    }
}

#[inline]
pub unsafe extern "C" fn un_802FE918_update_x3(x3_ptr: *mut u8, r: *mut i32) {
    let old_x3 = *x3_ptr as i32;
    // the C leaves `new_x3` uninitialised when no iteration breaks (zero in the build)
    let mut new_x3: i32 = 0;
    let mut k = 0;
    while k < 3 {
        if k == old_x3 {
            *r += 1;
        } else if *r == k {
            new_x3 = *r;
            break;
        }
        k += 1;
    }
    *x3_ptr = new_x3 as u8;
}

pub unsafe extern "C" fn un_802FE918(a: i32, b: i32, c: i32) {
    let st = &raw mut un_803F9D48;
    let i: i32;
    let mut sp14 = datetime { year: 0, month: 0, day: 0, hour: 0, minute: 0, second: 0 };

    ext::lbAudioAx_800236DC();
    let x3_ptr: *mut u8 = &raw mut (*st).x3;
    ext::lbAudioAx_80023F28((*st).x30[*x3_ptr as usize]);
    let mut r = ext::HSD_Randi(2);
    un_802FE918_update_x3(x3_ptr, &mut r);
    ext::gmMainLib_8015D8B0(a as u32);
    let mut found = 0;
    let mut x: *mut un_803F9B30 = &raw mut un_803F9B30[0];
    while (*x).x0 != 66 {
        if (*x).x0 == a {
            found = un_803F9A00[(*x).x4 as usize];
            break;
        }
        x = x.add(1);
    }
    i = found;
    (*st).x4 = i as u16;
    if a == 0x3E {
        un_802FE3F8(a, 2, &raw mut (*st).x6 as *mut i16, &raw mut (*st).x8 as *mut i16);
        let v_x6: u16 = (*st).x6;
        r = (*st).x8 as i32;
        ext::HSD_SisLib_803A6530(2, 0x4A, v_x6 as i32);
        ext::HSD_SisLib_803A660C(2, 0x4A, ext::Toy_803063D4(b, 0x4E, 0x174));
        ext::HSD_SisLib_803A660C(2, 0x4A, r);
        ext::HSD_SisLib_803A6368((*st).x20, 0x4A);
    } else {
        un_802FE3F8(a, 2, &raw mut (*st).x6 as *mut i16, null_mut());
        ext::HSD_SisLib_803A6368((*st).x20, (*st).x6 as i32);
    }
    ext::gm_801692E8(c as u32, &mut sp14);
    let text = if ext::lbLang_IsSavedLanguageUS() != 0 {
        format!(
            "{:2}.{:2}.{:04}  {:02}:{:02}:{:02}",
            sp14.month as i32, sp14.day as i32, sp14.year as i32, sp14.hour as i32, sp14.minute as i32, sp14.second as i32
        )
    } else {
        format!(
            "{:04}.{:2}.{:2}  {:02}:{:02}:{:02}",
            sp14.year as i32, sp14.month as i32, sp14.day as i32, sp14.hour as i32, sp14.minute as i32, sp14.second as i32
        )
    };
    let mut sp1c = [0u8; 0x104];
    sp1c[..text.len()].copy_from_slice(text.as_bytes());
    let text_p: *mut *mut HSD_Text = &raw mut (*st).x24;
    ext::HSD_SisLib_803A7664(*text_p);
    ext::HSD_SisLib_803A6B98(*text_p, 320.0, 316.0, sp1c.as_ptr());
}

pub unsafe extern "C" fn ifPrize_Scene_OnEnter(arg0_: *mut c_void) {
    let st = &raw mut un_803F9D48;
    let arg0 = arg0_ as *mut un_802FEBE0_OnEnter_arg0;

    ext::Toy_803124BC();
    (*st).x28 = arg0;
    (*st).x2C = (*arg0).x8;
    (*st).x3 = ext::HSD_Randi(3) as u8;
    let arg0x4 = (*arg0).x4;
    let arg0x0 = (*arg0).x0 as i32;
    (*st).set_x0b(1);
    let mut i = 0;
    let mut x: *mut un_803F9B30 = &raw mut un_803F9B30[0];
    while (*x).x0 != 66 {
        if (*x).x0 == arg0x0 {
            i = un_803F9A00[(*x).x4 as usize];
            break;
        }
        x = x.add(1);
    }
    (*st).x4 = i as u16;
    un_802FE3F8_noinline(arg0x0, 2, &raw mut (*st).x6 as *mut i16, null_mut());
    (*st).x2 = -1;
    (*st).x1 = 0;
    (*st).xC = arg0x4;
    (*st).set_x0a(1);
    setArchive();
    un_802FE6A8();
}

pub unsafe extern "C" fn ifPrize_Scene_OnExit(_unused: *mut c_void) {}
