//! src/sysdolphin/baselib/gobjgxlink.c: the GX (render) link lists.
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

/// `GObj_GXReorder`.
pub unsafe fn GObj_GXReorder(gobj: *mut HSD_GObj, hiprio_gobj: *mut HSD_GObj) {
    melee_core::ct!("GObj_GXReorder");
    let link = (*gobj).gx_link as usize;
    (*gobj).prev_gx = hiprio_gobj;
    if !hiprio_gobj.is_null() {
        (*gobj).next_gx = (*hiprio_gobj).next_gx;
        (*hiprio_gobj).next_gx = gobj;
    } else {
        (*gobj).next_gx = *at(HSD_GObjGXLinkHead, link);
        *at(HSD_GObjGXLinkHead, link) = gobj;
    }
    if !(*gobj).next_gx.is_null() {
        (*(*gobj).next_gx).prev_gx = gobj;
    } else {
        *at(HSD_GObj_804D7820, (*gobj).gx_link as usize) = gobj;
    }
}

/// `GObj_SetupGXLink`.
pub unsafe fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32) {
    melee_core::ct!("GObj_SetupGXLink");
    hsd_assert!("gobjgxlink.c", 167, gx_link <= HSD_GObjLibInitData.gx_link_max);
    (*gobj).render_cb = render_cb;
    (*gobj).gx_link = gx_link;
    (*gobj).render_priority = priority as u8;
    let mut i = *at(HSD_GObj_804D7820, (*gobj).gx_link as usize);
    while !i.is_null() && (*i).render_priority > (*gobj).render_priority {
        i = (*i).prev_gx;
    }
    GObj_GXReorder(gobj, i);
}

/// `GObj_SetupGXLinkMax`.
pub unsafe fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32) {
    melee_core::ct!("GObj_SetupGXLinkMax");
    let max_link = HSD_GObjLibInitData.gx_link_max;
    (*gobj).render_cb = render_cb;
    (*gobj).gx_link = max_link.wrapping_add(1);
    (*gobj).render_priority = priority as u8;
    let mut i = *at(HSD_GObj_804D7820, (*gobj).gx_link as usize);
    while !i.is_null() && (*i).render_priority > (*gobj).render_priority {
        i = (*i).prev_gx;
    }
    GObj_GXReorder(gobj, i);
}

/// `GObj_SetupGXLinkMaxSorted`.
pub unsafe fn GObj_SetupGXLinkMaxSorted(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, priority: u32) {
    melee_core::ct!("GObj_SetupGXLinkMaxSorted");
    let max_link = HSD_GObjLibInitData.gx_link_max;
    (*gobj).render_cb = render_cb;
    (*gobj).gx_link = max_link.wrapping_add(1);
    (*gobj).render_priority = priority as u8;
    let mut i = gx_find_prio_position(gobj);
    if !i.is_null() {
        i = (*i).prev_gx;
    } else {
        i = *at(HSD_GObj_804D7820, (*gobj).gx_link as usize);
    }
    GObj_GXReorder(gobj, i);
}

/// `HSD_GObjGXLink_8039084C`: unlink from the render link.
pub unsafe fn HSD_GObjGXLink_8039084C(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjGXLink_8039084C");
    hsd_assert!("gobjgxlink.c", 415, (*gobj).gx_link != HSD_GOBJ_GXLINK_NONE);
    let prev = (*gobj).prev_gx;
    if !prev.is_null() {
        (*prev).next_gx = (*gobj).next_gx;
    } else {
        *at(HSD_GObjGXLinkHead, (*gobj).gx_link as usize) = (*gobj).next_gx;
    }
    let next = (*gobj).next_gx;
    if !next.is_null() {
        (*next).prev_gx = (*gobj).prev_gx;
    } else {
        *at(HSD_GObj_804D7820, (*gobj).gx_link as usize) = (*gobj).prev_gx;
    }
    (*gobj).gx_link = HSD_GOBJ_GXLINK_NONE;
    (*gobj).render_priority = 0;
    (*gobj).prev_gx = null_mut();
    (*gobj).next_gx = null_mut();
}

/// `HSD_GObjGXLink_80390908`: move to `gx_link` at `priority`.
pub unsafe fn HSD_GObjGXLink_80390908(gobj: *mut HSD_GObj, gx_link: u8, priority: u8) {
    melee_core::ct!("HSD_GObjGXLink_80390908");
    hsd_assert!("gobjgxlink.c", 535, gx_link <= HSD_GObjLibInitData.gx_link_max);
    HSD_GObjGXLink_8039084C(gobj);
    (*gobj).gx_link = gx_link;
    (*gobj).render_priority = priority;
    let mut cur = *at(HSD_GObjGXLinkHead, (*gobj).gx_link as usize);
    while !cur.is_null() && (*cur).render_priority < (*gobj).render_priority {
        cur = (*cur).next_gx;
    }
    GObj_GXReorder(gobj, if !cur.is_null() { (*cur).prev_gx } else { *at(HSD_GObj_804D7820, (*gobj).gx_link as usize) });
}

/// `HSD_GObjGXLink_803909D8`: re-link `gobj` right after `other`'s predecessor, in `other`'s link.
pub unsafe fn HSD_GObjGXLink_803909D8(gobj: *mut HSD_GObj, other: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjGXLink_803909D8");
    let prio = (*other).render_priority;
    let link = (*other).gx_link;
    HSD_GObjGXLink_8039084C(gobj);
    (*gobj).gx_link = link;
    (*gobj).render_priority = prio;
    GObj_GXReorder(gobj, (*other).prev_gx);
}
