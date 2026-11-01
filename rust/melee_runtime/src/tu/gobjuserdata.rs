//! src/sysdolphin/baselib/gobjuserdata.c: GObj user data.
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

/// `GObj_InitUserData`.
pub unsafe fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void) {
    melee_core::ct!("GObj_InitUserData");
    hsd_assert!("gobjuserdata.c", 40, (*gobj).user_data_kind == HSD_GOBJ_USER_DATA_NONE);
    (*gobj).user_data_kind = kind;
    (*gobj).user_data = data;
    (*gobj).user_data_remove_func = remove_func;
}

/// `GObj_RemoveUserData`.
pub unsafe fn GObj_RemoveUserData(gobj: *mut HSD_GObj) {
    melee_core::ct!("GObj_RemoveUserData");
    if (*gobj).user_data_kind == HSD_GOBJ_USER_DATA_NONE {
        return;
    }
    hsd_assert!("gobjuserdata.c", 99, (*gobj).user_data_remove_func.is_some());
    ((*gobj).user_data_remove_func.unwrap())((*gobj).user_data);
    (*gobj).user_data_kind = HSD_GOBJ_USER_DATA_NONE;
    (*gobj).user_data = null_mut();
}
