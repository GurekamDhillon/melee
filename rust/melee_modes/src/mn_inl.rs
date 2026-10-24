//! Shared by the menu (`src/melee/mn/`) TU ports: the menu globals owned by TUs that are not
//! ported yet (mnmain.c), the `mn/inlines.h` helpers and the `jobj.h` / `gobj.h` / `lobj.h`
//! static inlines the menus use, and the `__assert` the baselib asserts expand to.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::ext_fns;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_types::*;

/// `mnmain.c` `/* 4A04F0 */ MenuFlow mn_804A04F0`.
pub static mut mn_804A04F0: MenuFlow = unsafe { core::mem::zeroed() };
/// `mnmain.c` `/* 4D6BC8 */ MenuInputState mn_804D6BC8`.
pub static mut mn_804D6BC8: MenuInputState = unsafe { core::mem::zeroed() };
/// `mnmain.c` `/* 4D6BB8 */ HSD_Archive* mn_804D6BB8` (the menu model archive).
pub static mut mn_804D6BB8: *mut HSD_Archive = null_mut();
/// `mnmain.c` `/* 4D6BB4 */ u8 mn_804D6BB4`.
pub static mut mn_804D6BB4: u8 = 0;
/// `mnmain.c` `/* 4D6BB5 */ u8 mn_804D6BB5`.
pub static mut mn_804D6BB5: u8 = 0;

ext_fns! {
    fn lbAudioAx_80024030(sfx: i32);
    fn mn_80229624(port: u32) -> u32;
    fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj);
    fn HSD_SisLib_803A5ACC(a: i32, b: i32, x: f32, y: f32, z: f32, w: f32, h: f32) -> *mut HSD_Text;
    fn HSD_SisLib_803A6368(text: *mut HSD_Text, id: i32);
    // C varargs, fixed here to the arity every mn caller uses.
    fn OSReport(fmt: *const c_char);
}

/// `HSD_SisLib_803A6B98(text, x, y, fmt, ...)` (sislib, not ported): the printf arguments are
/// passed as integer / pointer bits.
pub unsafe fn HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const c_char, args: &[usize]) -> i32 {
    match crate::tu::ext_hook::EXT_HOOK {
        Some(h) => {
            let mut a: std::vec::Vec<u64> = std::vec![text as usize as u64, x.to_bits() as u64, y.to_bits() as u64, fmt as usize as u64];
            a.extend(args.iter().map(|v| *v as u64));
            h("HSD_SisLib_803A6B98", &a) as i32
        }
        None => panic!("TODO ext: HSD_SisLib_803A6B98 is in a TU that is not ported yet"),
    }
}

/// `lb_80011E24(jobj, &out, idx0, idx1, ..., -1)` (lb/lb_00B0.c, not ported): finds the joint at the
/// child-index path; `path` includes the `-1` terminator.
pub unsafe fn lb_80011E24(jobj: *mut HSD_JObj, out: *mut *mut HSD_JObj, path: &[i32]) -> i32 {
    match crate::tu::ext_hook::EXT_HOOK {
        Some(h) => {
            let mut a: std::vec::Vec<u64> = std::vec![jobj as usize as u64, out as usize as u64];
            a.extend(path.iter().map(|v| *v as i64 as u64));
            h("lb_80011E24", &a) as i32
        }
        None => panic!("TODO ext: lb_80011E24 is in a TU that is not ported yet"),
    }
}

/// `lbArchive_LoadSections(archive, &sym0, "name0", &sym1, "name1", ..., 0)` (lb/lbarchive.c, not
/// ported): the (symbol slot, section name) pairs.
pub unsafe fn lbArchive_LoadSections(archive: *mut HSD_Archive, sections: &[(*mut *mut c_void, *const c_char)]) {
    match crate::tu::ext_hook::EXT_HOOK {
        Some(h) => {
            let mut a: std::vec::Vec<u64> = std::vec![archive as usize as u64];
            for (slot, name) in sections {
                a.push(*slot as usize as u64);
                a.push(*name as usize as u64);
            }
            h("lbArchive_LoadSections", &a);
        }
        None => panic!("TODO ext: lbArchive_LoadSections is in a TU that is not ported yet"),
    }
}

/// `__assert(file, line, expr)`: the baselib assert failure (aborts).
pub fn __assert(file: &str, line: u32, expr: &str) -> ! {
    panic!("{file}:{line}: assertion failed: {expr}");
}

/// `HSD_ASSERT(line, cond)` of the baselib headers.
macro_rules! hsd_assert {
    ($file:literal, $line:literal, $cond:expr, $expr:literal) => {
        if !$cond {
            __assert($file, $line, $expr);
        }
    };
}
pub(crate) use hsd_assert;

pub unsafe fn HSD_GObjGetUserData(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).user_data
}
pub unsafe fn HSD_GObjGetHSDObj(gobj: *mut HSD_GObj) -> *mut c_void {
    (*gobj).hsd_obj
}
/// `GET_MENU(gobj)`.
pub unsafe fn GET_MENU(gobj: *mut HSD_GObj) -> *mut Menu {
    HSD_GObjGetUserData(gobj) as *mut Menu
}
/// `GET_JOBJ(gobj)`.
pub unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj
}

pub unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}
pub unsafe fn HSD_JObjGetNext(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).next }
}
pub unsafe fn HSD_LObjSetNext(lobj: *mut HSD_LObj, next: *mut HSD_LObj) {
    hsd_assert!("src/sysdolphin/baselib/lobj.h", 194, !lobj.is_null(), "lobj");
    (*lobj).next = next;
}

/// `HSD_JObjMtxIsDirty`.
pub unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 228, !jobj.is_null(), "jobj");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// The tail every `HSD_JObjSet*` inline shares: mark the matrix dirty unless it is independent.
#[inline]
unsafe fn mtx_dirty_sub(jobj: *mut HSD_JObj) {
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        HSD_JObjSetMtxDirtySub(jobj);
    }
}

macro_rules! jobj_setter {
    ($name:ident, $line:literal, $($f:ident).+) => {
        pub unsafe fn $name(jobj: *mut HSD_JObj, v: f32) {
            hsd_assert!("src/sysdolphin/baselib/jobj.h", $line, !jobj.is_null(), "jobj");
            (*jobj).$($f).+ = v;
            mtx_dirty_sub(jobj);
        }
    };
}
jobj_setter!(HSD_JObjSetScaleX, 402, scale.x);
jobj_setter!(HSD_JObjSetScaleY, 420, scale.y);
jobj_setter!(HSD_JObjSetTranslateX, 513, translate.x);
jobj_setter!(HSD_JObjSetTranslateY, 531, translate.y);
jobj_setter!(HSD_JObjSetTranslateZ, 549, translate.z);

/// `HSD_JObjAddTranslationY`.
pub unsafe fn HSD_JObjAddTranslationY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 672, !jobj.is_null(), "jobj");
    (*jobj).translate.y += y;
    mtx_dirty_sub(jobj);
}

pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 481, !jobj.is_null(), "jobj");
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 482, !translate.is_null(), "translate");
    (*jobj).translate = *translate;
    mtx_dirty_sub(jobj);
}

pub unsafe fn HSD_JObjSetTranslateWithMtxDirty(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 492, !jobj.is_null(), "jobj");
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 493, !translate.is_null(), "translate");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 567, !jobj.is_null(), "jobj");
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 568, !translate.is_null(), "translate");
    *translate = (*jobj).translate;
}
pub unsafe fn HSD_JObjGetTranslationX(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 582, !jobj.is_null(), "jobj");
    (*jobj).translate.x
}
pub unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 588, !jobj.is_null(), "jobj");
    (*jobj).translate.y
}
pub unsafe fn HSD_JObjGetTranslationZ(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert!("src/sysdolphin/baselib/jobj.h", 594, !jobj.is_null(), "jobj");
    (*jobj).translate.z
}

/// `mn/inlines.h` `Menu_DecrementAnimTimer`.
pub unsafe fn Menu_DecrementAnimTimer() {
    mn_804D6BC8.cooldown = mn_804D6BC8.cooldown.wrapping_sub(1);
    mn_804D6BC8.x2 = 0;
    mn_804D6BC8.x4 = 0;
}
/// `mn/inlines.h` `Menu_GetAllInputs`.
pub unsafe fn Menu_GetAllInputs() -> u64 {
    mn_804A04F0.buttons = mn_80229624(4) as u64;
    mn_804A04F0.buttons
}
pub unsafe fn sfxBack() {
    lbAudioAx_80024030(0);
}
pub unsafe fn sfxForward() {
    lbAudioAx_80024030(1);
}
pub unsafe fn sfxMove() {
    lbAudioAx_80024030(2);
}
/// `mn/inlines.h` `Menu_InitCenterText`.
pub unsafe fn Menu_InitCenterText(menu: *mut Menu, val: i32) {
    let text = HSD_SisLib_803A5ACC(0, 1, -9.5, 9.1, 17.0, 364.68332, 38.38772);
    (*menu).text = text;
    (*text).font_size.x = 0.0521;
    (*text).font_size.y = 0.0521;
    HSD_SisLib_803A6368(text, val);
}

/// NUL-terminated C string literal pointer helper for `OSReport`-style callees.
pub const fn cstr(s: &'static str) -> *const c_char {
    s.as_ptr() as *const c_char
}

// dolphin/pad.h
pub const PAD_MAX_CONTROLLERS: i32 = 4;
pub const PAD_CONFIRM: u64 = 1 << 32;
pub const PAD_CANCEL: u64 = 1 << 33;
pub const PAD_ANY_UP: u64 = 1 << 36;
pub const PAD_ANY_DOWN: u64 = 1 << 37;
pub const PAD_ANY_LEFT: u64 = 1 << 38;
pub const PAD_ANY_RIGHT: u64 = 1 << 39;
