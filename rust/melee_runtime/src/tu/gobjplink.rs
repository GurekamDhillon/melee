//! src/sysdolphin/baselib/gobjplink.c: create / free / priority change of the process links.
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

/// `GObj_PReorder`: link `gobj` after `hiprio_gobj` (NULL: at the head) in its process link.
pub unsafe fn GObj_PReorder(gobj: *mut HSD_GObj, hiprio_gobj: *mut HSD_GObj) {
    melee_core::ct!("GObj_PReorder");
    let link = (*gobj).p_link as usize;
    (*gobj).prev = hiprio_gobj;
    if !hiprio_gobj.is_null() {
        (*gobj).next = (*hiprio_gobj).next;
        (*hiprio_gobj).next = gobj;
    } else {
        (*gobj).next = *at(HSD_GObjPLinkHead, link);
        *at(HSD_GObjPLinkHead, link) = gobj;
    }
    if !(*gobj).next.is_null() {
        (*(*gobj).next).prev = gobj;
    } else {
        *at(plinklow_gobjs, link) = gobj;
    }
}

unsafe fn gobj_first_lower_prio(gobj: *mut HSD_GObj) {
    let mut r4 = *at(plinklow_gobjs, (*gobj).p_link as usize);
    while !r4.is_null() && (*r4).p_priority > (*gobj).p_priority {
        r4 = (*r4).prev;
    }
    GObj_PReorder(gobj, r4);
}

unsafe fn gobj_first_higher_prio(gobj: *mut HSD_GObj) {
    let mut r4 = *at(HSD_GObjPLinkHead, (*gobj).p_link as usize);
    while !r4.is_null() && (*r4).p_priority < (*gobj).p_priority {
        r4 = (*r4).next;
    }
    GObj_PReorder(gobj, if !r4.is_null() { (*r4).prev } else { *at(plinklow_gobjs, (*gobj).p_link as usize) });
}

/// `CreateGObj`.
pub unsafe fn CreateGObj(r#where: i32, classifier: u16, p_link: u8, priority: u8, position: *mut HSD_GObj) -> *mut HSD_GObj {
    melee_core::ct!("CreateGObj");
    if let Some(f) = super::gobj_native::ext::ScriptGame_StageSlotCanCreate {
        if f() == 0 {
            return null_mut();
        }
    }
    hsd_assert!("gobjplink.c", 0xA8, p_link <= HSD_GObjLibInitData.p_link_max);
    let gobj: *mut HSD_GObj = HSD_ObjAlloc(&mut gobj_alloc_data).cast();
    if gobj.is_null() {
        return null_mut();
    }
    (*gobj).classifier = classifier;
    (*gobj).p_link = p_link;
    (*gobj).gx_link = HSD_GOBJ_GXLINK_NONE;
    (*gobj).p_priority = priority;
    (*gobj).render_priority = 0;
    (*gobj).obj_kind = HSD_GOBJ_OBJ_NONE;
    (*gobj).user_data_kind = 0xFF;
    (*gobj).prev_gx = null_mut();
    (*gobj).next_gx = null_mut();
    (*gobj).proc = null_mut();
    (*gobj).render_cb = None;
    (*gobj).gxlink_prios = 0;
    (*gobj).hsd_obj = null_mut();
    (*gobj).user_data = null_mut();
    (*gobj).user_data_remove_func = None;
    match r#where {
        0 => gobj_first_lower_prio(gobj),
        1 => gobj_first_higher_prio(gobj),
        2 => GObj_PReorder(gobj, position),
        3 => GObj_PReorder(gobj, (*position).prev),
        _ => {}
    }
    if let Some(f) = super::gobj_native::ext::ScriptGame_StageSlotCreated {
        f(gobj);
    }
    gobj
}

/// `GObj_Create`.
pub unsafe fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj {
    melee_core::ct!("GObj_Create");
    CreateGObj(0, classifier, p_link, priority, null_mut())
}

/// `HSD_GObjFree`.
pub unsafe fn HSD_GObjFree(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjFree");
    hsd_assert!("gobjplink.c", 0x171, !gobj.is_null());
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC == 0 && gobj == HSD_GObj_CurrentInvokedProcGObj {
        HSD_GObj_DelayedProcInfo.flags |= DelayedProcInfo::DELAY_REMOVE_GOBJ;
        return;
    }
    if let Some(f) = super::gobj_native::ext::ScriptGame_StageSlotDestroyed {
        f(gobj);
    }
    GObj_RemoveUserData(gobj);
    HSD_GObjObject_80390B0C(gobj);
    HSD_GObjProc_RemoveAllProcs(gobj);
    if (*gobj).gx_link != HSD_GOBJ_GXLINK_NONE {
        HSD_GObjGXLink_8039084C(gobj);
    }
    if !(*gobj).prev.is_null() {
        (*(*gobj).prev).next = (*gobj).next;
    } else {
        *at(HSD_GObjPLinkHead, (*gobj).p_link as usize) = (*gobj).next;
    }
    if !(*gobj).next.is_null() {
        (*(*gobj).next).prev = (*gobj).prev;
    } else {
        *at(plinklow_gobjs, (*gobj).p_link as usize) = (*gobj).prev;
    }
    HSD_ObjFree(&mut gobj_alloc_data, gobj.cast());
}

/// `HSD_GObjPLink_ChangeGObjPri_Unk`.
pub unsafe fn HSD_GObjPLink_ChangeGObjPri_Unk(arg0: u32, gobj: *mut HSD_GObj, p_link: u8, priority: u8, position: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjPLink_ChangeGObjPri_Unk");
    hsd_assert!("gobjplink.c", 0x1A3, p_link <= HSD_GObjLibInitData.p_link_max);
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC == 0 && gobj == HSD_GObj_CurrentInvokedProcGObj {
        HSD_GObj_DelayedProcInfo.flags |= DelayedProcInfo::DELAY_CHANGE_GOBJ_PRI;
        HSD_GObj_DelayedProcInfo.r#type = arg0;
        HSD_GObj_DelayedProcInfo.p_link = p_link;
        HSD_GObj_DelayedProcInfo.p_prio = priority;
        HSD_GObj_DelayedProcInfo.gobj = position;
        return;
    }
    let mut cur = (*gobj).proc;
    let mut proc_cur: *mut HSD_GObjProc = null_mut();
    while !cur.is_null() {
        HSD_GObjProc_UnqueueProc(cur);
        let child = (*cur).child;
        (*cur).child = proc_cur;
        proc_cur = cur;
        cur = child;
    }
    (*gobj).proc = null_mut();
    if !(*gobj).prev.is_null() {
        (*(*gobj).prev).next = (*gobj).next;
    } else {
        *at(HSD_GObjPLinkHead, (*gobj).p_link as usize) = (*gobj).next;
    }
    if !(*gobj).next.is_null() {
        (*(*gobj).next).prev = (*gobj).prev;
    } else {
        *at(plinklow_gobjs, (*gobj).p_link as usize) = (*gobj).prev;
    }
    (*gobj).p_link = p_link;
    (*gobj).p_priority = priority;
    match arg0 {
        0 => gobj_first_lower_prio(gobj),
        1 => gobj_first_higher_prio(gobj),
        2 => GObj_PReorder(gobj, position),
        3 => GObj_PReorder(gobj, (*position).prev),
        _ => {}
    }
    let flags_new = if HSD_GObj_804D783C == 0 { 2 } else { HSD_GObj_804D783C - 1 };
    let flags_cur = if flags_new == 0 { 2 } else { flags_new - 1 };
    cur = proc_cur;
    while !cur.is_null() {
        let child = (*cur).child;
        HSD_GObjProc_QueueProc(cur);
        if (*cur).flags_3() as i32 == flags_cur {
            (*cur).set_flags_3(flags_new as u8);
        }
        cur = child;
    }
}
