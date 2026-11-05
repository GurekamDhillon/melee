//! Port of src/sysdolphin/baselib/hsd_3910.c, split from gobj.c upstream (doldecomp 598987cf3): HSD_GObj_803910D8, HSD_GObj_80391120, HSD_GObj_803911C0, HSD_GObj_80391260, HSD_GObj_803912A8, HSD_GObj_FogCallback, HSD_GObj_JObjCallback, HSD_GObj_LObjCallback.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use melee_types::inline::ref_DEC;
use melee_types::*;
use super::gobj_native::{self as gn, *};
use super::gobj_native::*;
use super::gobj_native::{at, hsd_assert};
use super::memory::HSD_MemAlloc;
use super::objalloc::{HSD_ObjAlloc, HSD_ObjAllocInit, HSD_ObjFree};
use core::ffi::c_void;
use core::ptr::null_mut;
use super::gobj::*;
use super::gobj::ext;

/// `HSD_GObj_LObjCallback`: render callback of a light GObj.
pub unsafe fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("HSD_GObj_LObjCallback");
    ext::HSD_LObj_803668EC((*gobj).hsd_obj as *mut HSD_LObj);
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
}

/// `HSD_GObj_JObjCallback`: render callback of a jobj GObj (flags from `HSD_GObj_80390EB8`).
pub unsafe fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("HSD_GObj_JObjCallback");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::HSD_JObjDispAll(jobj, core::ptr::null_mut(), HSD_GObj_80390EB8(arg1), 0);
}

/// `HSD_GObj_FogCallback`.
pub unsafe fn HSD_GObj_FogCallback(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("HSD_GObj_FogCallback");
    ext::HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}

/// `HSD_GObj_803912A8`: append `arg1` to the function-table chain, return the count before it.
pub unsafe fn HSD_GObj_803912A8(arg0: *mut _HSD_GObjLibInitDataType, arg1: *mut GObjFuncs) -> u8 {
    melee_core::ct!("HSD_GObj_803912A8");
    let mut pcur: *mut *mut GObjFuncs = &mut (*arg0).funcs;
    let mut n: u8 = 0;
    while !(*pcur).is_null() {
        let cur = *pcur;
        pcur = &mut (*cur).next;
        n = n.wrapping_add((*cur).size);
    }
    *pcur = arg1;
    (**pcur).next = null_mut();
    n
}

/// `HSD_GObj_80391120`: drop one reference of an HSD object and delete it (`hsdDelete`: the
/// class's `release` then `destroy`) when it was the last.
pub unsafe extern "C" fn HSD_GObj_80391120(obj: *mut melee_types::HSD_Obj) {
    melee_core::ct!("HSD_GObj_80391120");
    if !obj.is_null() && ref_dec(obj) {
        let info = (*obj).parent.class_info;
        ((*info).release.unwrap())(obj.cast());
        ((*info).destroy.unwrap())(obj.cast());
    }
}

pub unsafe extern "C" fn HSD_GObj_803911C0(obj: *mut melee_types::HSD_Obj) {
    melee_core::ct!("HSD_GObj_803911C0");
    HSD_GObj_80391120(obj);
}

pub unsafe fn HSD_GObj_80391260(arg0: *mut _HSD_GObjLibInitDataType) {
    melee_core::ct!("HSD_GObj_80391260");
    if HSD_GObj_80408600[0].is_none() {
        HSD_GObj_80408600[0] = Some(HSD_GObj_80391120);
        HSD_GObj_80408600[3] = Some(HSD_GObj_803911C0);
    }
    HSD_GObj_80408610.funcs = HSD_GObj_80408600.as_mut_ptr();
    let mut count = HSD_GObj_803912A8(arg0, &mut HSD_GObj_80408610);
    HSD_GObj_CameraKind = count;
    count = count.wrapping_add(1);
    HSD_GObj_LightKind = count as i8;
    count = count.wrapping_add(1);
    HSD_GObj_JObjKind = count;
    count = count.wrapping_add(1);
    HSD_GObj_FogKind = count as i8;
}

/// `HSD_GObj_803910D8`: render the scene for a camera GObj (set current, walk, end).
pub unsafe fn HSD_GObj_803910D8(gobj: *mut HSD_GObj, _renderpass: i32) {
    melee_core::ct!("HSD_GObj_803910D8");
    if super::cobj::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) {
        HSD_GObj_80390ED0(gobj, 7);
        super::cobj::HSD_CObjEndCurrent();
    }
}
