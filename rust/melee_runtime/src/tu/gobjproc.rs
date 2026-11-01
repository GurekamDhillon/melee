//! src/sysdolphin/baselib/gobjproc.c: the per-GObj process queue.
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

/// `HSD_GObjProc_QueueProc`.
pub unsafe fn HSD_GObjProc_QueueProc(gproc: *mut HSD_GObjProc) {
    melee_core::ct!("HSD_GObjProc_QueueProc");
    let proc_gobj = (*gproc).gobj;
    let s_link = (*gproc).s_link as usize;
    let mut p_link = (*proc_gobj).p_link as i32;
    let mut dst_proc: *mut HSD_GObjProc = null_mut();
    let mut found = false;
    if !(*proc_slot(p_link as usize, s_link)).is_null() {
        let mut cur_gobj = proc_gobj;
        'outer: while !cur_gobj.is_null() {
            dst_proc = (*cur_gobj).proc;
            while !dst_proc.is_null() {
                if (*dst_proc).s_link as usize == s_link {
                    if *proc_slot(p_link as usize, s_link) == dst_proc {
                        *proc_slot(p_link as usize, s_link) = gproc;
                    }
                    found = true;
                    break 'outer;
                }
                dst_proc = (*dst_proc).child;
            }
            cur_gobj = (*cur_gobj).prev;
        }
    } else {
        *proc_slot(p_link as usize, s_link) = gproc;
    }
    if !found {
        // scan the lower process links for a destination
        loop {
            let was = p_link;
            p_link -= 1;
            if was == 0 {
                break;
            }
            dst_proc = *proc_slot(p_link as usize, s_link);
            if !dst_proc.is_null() {
                found = true;
                break;
            }
        }
    }
    if found {
        (*gproc).next = (*dst_proc).next;
        (*dst_proc).next = gproc;
        (*gproc).prev = dst_proc;
    } else {
        (*gproc).next = *at(HSD_GObj_GObjProcHead, s_link);
        *at(HSD_GObj_GObjProcHead, s_link) = gproc;
        (*gproc).prev = null_mut();
    }
    if !(*gproc).next.is_null() {
        (*(*gproc).next).prev = gproc;
    }
    (*gproc).child = (*proc_gobj).proc;
    (*proc_gobj).proc = gproc;
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC != 0
        && (*gproc).prev == HSD_GObj_CurrentInvokedProc
        && (*gproc).next == HSD_GObj_NextInvokedProc
        && s_link as i32 == HSD_GObj_CurrentInvokedSLink
    {
        HSD_GObj_NextInvokedProc = gproc;
    }
}

/// `HSD_GObjProc_UnqueueProc`.
pub unsafe fn HSD_GObjProc_UnqueueProc(gproc: *mut HSD_GObjProc) {
    melee_core::ct!("HSD_GObjProc_UnqueueProc");
    let p_link = (*(*gproc).gobj).p_link as usize;
    let s_link = (*gproc).s_link as usize;
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC != 0 && gproc == HSD_GObj_NextInvokedProc {
        HSD_GObj_NextInvokedProc = (*gproc).next;
    }
    if gproc == *proc_slot(p_link, s_link) {
        if !(*gproc).prev.is_null() && (*(*(*gproc).prev).gobj).p_link as usize == p_link {
            *proc_slot(p_link, s_link) = (*gproc).prev;
        } else {
            *proc_slot(p_link, s_link) = null_mut();
        }
    }
    if !(*gproc).prev.is_null() {
        (*(*gproc).prev).next = (*gproc).next;
    } else {
        *at(HSD_GObj_GObjProcHead, s_link) = (*gproc).next;
    }
    if !(*gproc).next.is_null() {
        (*(*gproc).next).prev = (*gproc).prev;
    }
}

/// `HSD_GObjProc_UnlinkProcFromGObj`.
pub unsafe fn HSD_GObjProc_UnlinkProcFromGObj(gproc: *mut HSD_GObjProc) {
    melee_core::ct!("HSD_GObjProc_UnlinkProcFromGObj");
    let gobj = (*gproc).gobj;
    HSD_GObjProc_UnqueueProc(gproc);
    if (*gobj).proc == gproc {
        (*gobj).proc = (*gproc).child;
    } else {
        let mut cur = (*gobj).proc;
        while (*cur).child != gproc {
            cur = (*cur).child;
        }
        (*cur).child = (*gproc).child;
    }
}

/// `HSD_GObj_SetupProc`.
pub unsafe fn HSD_GObj_SetupProc(gobj: *mut HSD_GObj, func: HSD_GObjEvent, pri: u8) -> *mut HSD_GObjProc {
    melee_core::ct!("HSD_GObj_SetupProc");
    let gproc: *mut HSD_GObjProc = HSD_ObjAlloc(&mut gobjproc_alloc_data).cast();
    hsd_assert!("gobjproc.c", 31, !gproc.is_null());
    hsd_assert!("gobjproc.c", 216, pri <= HSD_GObjLibInitData.gproc_pri_max);
    (*gproc).s_link = pri;
    (*gproc).set_flags_1(0);
    (*gproc).set_flags_2(0);
    (*gproc).set_flags_3(3);
    (*gproc).gobj = gobj;
    (*gproc).on_invoke = func;
    HSD_GObjProc_QueueProc(gproc);
    gproc
}

/// `HSD_GObjProc_RemoveProc`.
pub unsafe fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc) {
    melee_core::ct!("HSD_GObjProc_RemoveProc");
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC == 0 && gproc == HSD_GObj_CurrentInvokedProc {
        HSD_GObj_DelayedProcInfo.flags |= DelayedProcInfo::DELAY_REMOVE_PROC;
    } else {
        HSD_GObjProc_UnlinkProcFromGObj(gproc);
        HSD_ObjFree(&mut gobjproc_alloc_data, gproc.cast());
    }
}

/// `HSD_GObjProc_RemoveAllProcs`.
pub unsafe fn HSD_GObjProc_RemoveAllProcs(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObjProc_RemoveAllProcs");
    let mut cur = (*gobj).proc;
    while !cur.is_null() {
        let next = (*cur).child;
        HSD_GObjProc_RemoveProc(cur);
        cur = next;
    }
}
