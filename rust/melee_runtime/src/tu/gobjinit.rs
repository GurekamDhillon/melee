//! src/sysdolphin/baselib/gobjinit.c: library init defaults and HSD_GObjInit.
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

/// `HSD_GObjSetInitDefaults`.
pub unsafe fn HSD_GObjSetInitDefaults(arg0: *mut _HSD_GObjLibInitDataType) {
    melee_core::ct!("HSD_GObjSetInitDefaults");
    *arg0 = _HSD_GObjLibInitDataType {
        p_link_max: HSD_GOBJ_PLINK_MAX,
        gx_link_max: HSD_GOBJ_GX_LINK_MAX,
        gproc_pri_max: HSD_GOBJPROC_PRI_MAX,
        funcs: null_mut(),
        unk_2: null_mut(),
    };
}

/// `HSD_GObjInit`.
pub unsafe fn HSD_GObjInit(arg0: *mut _HSD_GObjLibInitDataType) {
    melee_core::ct!("HSD_GObjInit");
    HSD_GObj_80391260(arg0);
    HSD_GObjLibInitData = core::ptr::read(arg0);
    let plinks = (*arg0).p_link_max as usize + 1;
    HSD_GObjPLinkHead = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * plinks) as i32).cast();
    plinklow_gobjs = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * plinks) as i32).cast();
    for i in 0..plinks {
        *at(HSD_GObjPLinkHead, i) = null_mut();
        *at(plinklow_gobjs, i) = null_mut();
    }
    let gxlinks = (*arg0).gx_link_max as usize + 2;
    HSD_GObjGXLinkHead = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * gxlinks) as i32).cast();
    HSD_GObj_804D7820 = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * gxlinks) as i32).cast();
    for i in 0..gxlinks {
        *at(HSD_GObjGXLinkHead, i) = null_mut();
        *at(HSD_GObj_804D7820, i) = null_mut();
    }
    let pris = (*arg0).gproc_pri_max as usize + 1;
    HSD_GObj_GObjProcHead = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObjProc>() * pris) as i32).cast();
    for i in 0..pris {
        *at(HSD_GObj_GObjProcHead, i) = null_mut();
    }
    HSD_GObj_ProcList = HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObjProc>() * pris * plinks) as i32).cast();
    for i in 0..pris * plinks {
        *at(HSD_GObj_ProcList, i) = null_mut();
    }
    HSD_ObjAllocInit(&mut gobj_alloc_data, core::mem::size_of::<HSD_GObj>(), 4);
    HSD_ObjAllocInit(&mut gobjproc_alloc_data, core::mem::size_of::<HSD_GObjProc>(), 4);

    let mut nfuncs = 0usize;
    let mut f = (*arg0).funcs;
    while !f.is_null() {
        nfuncs += (*f).size as usize;
        f = (*f).next;
    }
    if nfuncs != 0 {
        HSD_GObj_804D7810 = HSD_MemAlloc((core::mem::size_of::<GObjFunc>() * nfuncs) as i32).cast();
        let mut k = 0usize;
        let mut cur = (*arg0).funcs;
        while !cur.is_null() {
            for i in 0..(*cur).size as usize {
                *HSD_GObj_804D7810.add(k) = *(*cur).funcs.add(i);
                k += 1;
            }
            cur = (*cur).next;
        }
    } else {
        HSD_GObj_804D7810 = null_mut();
    }
    HSD_GObj_804D783C = 0;
    HSD_GObj_CurrentInvokedProcGObj = null_mut();
    HSD_GObj_CurrentInvokedProc = null_mut();
    HSD_GObj_DelayedProcInfo.flags = 0;
    HSD_GObj_804D7818 = null_mut();
    HSD_GObj_804D7814 = null_mut();
}
