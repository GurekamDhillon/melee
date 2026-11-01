//! src/sysdolphin/baselib/gobjobject.c: attach / detach / release a GObj's HSD object.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_imports)]
use super::gobj_native::*;
use super::gobj_native::*;
use super::gobj_native::{at, hsd_assert};
use super::memory::HSD_MemAlloc;
use super::objalloc::{HSD_ObjAlloc, HSD_ObjAllocInit, HSD_ObjFree};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::{
    GObjFunc, GObjFuncs, GObj_RenderFunc, HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_ObjAllocData,
    _HSD_GObjLibInitDataType,
};

/// `HSD_GObjObject_80390A3C`: first GObj of link `arg1` with classifier `id`.
pub unsafe fn HSD_GObjObject_80390A3C(id: u16, arg1: u8) -> *mut HSD_GObj {
    melee_core::ct!("HSD_GObjObject_80390A3C");
    let mut cur = *at(HSD_GObjPLinkHead, arg1 as usize);
    while !cur.is_null() {
        if (*cur).classifier == id {
            break;
        }
        cur = (*cur).next;
    }
    cur
}

/// `HSD_GObjObject_80390A70`: attach the object of `kind`.
pub unsafe fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) {
    melee_core::ct!("HSD_GObjObject_80390A70");
    hsd_assert!("gobjobject.c", 42, (*gobj).obj_kind == HSD_GOBJ_OBJ_NONE);
    (*gobj).obj_kind = kind;
    (*gobj).hsd_obj = obj;
}

/// `HSD_GObjObject_80390ADC`: detach and return the object (NULL when none).
pub unsafe fn HSD_GObjObject_80390ADC(gobj: *mut HSD_GObj) -> *mut c_void {
    melee_core::ct!("HSD_GObjObject_80390ADC");
    if (*gobj).obj_kind != HSD_GOBJ_OBJ_NONE {
        (*gobj).obj_kind = HSD_GOBJ_OBJ_NONE;
        let obj = (*gobj).hsd_obj;
        (*gobj).hsd_obj = null_mut();
        obj
    } else {
        null_mut()
    }
}

/// `HSD_GObjObject_80390B0C`: run the kind's release function and detach.
pub unsafe fn HSD_GObjObject_80390B0C(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjObject_80390B0C");
    if (*gobj).obj_kind != HSD_GOBJ_OBJ_NONE {
        if let Some(f) = *HSD_GObj_804D7810.add((*gobj).obj_kind as usize) {
            f((*gobj).hsd_obj.cast());
        }
        (*gobj).obj_kind = HSD_GOBJ_OBJ_NONE;
        (*gobj).hsd_obj = null_mut();
    }
}
