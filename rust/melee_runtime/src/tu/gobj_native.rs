//! Native, raw-pointer port of the GObj library: `src/sysdolphin/baselib/gobjinit.c`,
//! `gobjplink.c` (create / free / priority change of the process links), `gobjgxlink.c` (render
//! links), `gobjproc.c` (the per-GObj process queue), `gobjobject.c`, `gobjuserdata.c` and the
//! non-rendering helpers of `gobj.c`. It owns the C globals (`HSD_GObjPLinkHead`, `plinklow_gobjs`,
//! `HSD_GObjGXLinkHead`, `HSD_GObj_804D7820`, `HSD_GObj_ProcList`, `HSD_GObj_GObjProcHead`,
//! `HSD_GObjLibInitData`, the two alloc pools, the delayed-proc record, ...) and works on the
//! GameCube-layout records `HSD_GObj` / `HSD_GObjProc` through raw pointers, as the C does.
//!
//! The model-based [`crate::gobj`] (ids, host trait) stays the verified reference of the logic;
//! this is the C-shaped surface the TUs call. Memory comes from [`super::objalloc`] /
//! [`super::memory`] (host hooks), exactly as in the C. The TARGET_PC script hooks
//! (`ScriptGame_StageSlot*`) are the [`ext`] function pointers (unset: allow creation, no-op).
//! `HSD_ASSERT` (a halt in the C) is a panic. The PC render tracing (`gobj_trace_render`) and the
//! profiler zones are diagnostics and are not ported.
//!
//! Checked against the C compiled natively (shipping contraction, the real gobj*.c): state oracle
//! `tests/gobj_native_vs_c.rs` (`python rust/melee_runtime/gen/pure/run_pure.py gobj_native`).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use super::memory::HSD_MemAlloc;
use super::objalloc::{HSD_ObjAlloc, HSD_ObjAllocInit, HSD_ObjFree};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::{
    GObjFunc, GObjFuncs, GObj_RenderFunc, HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_ObjAllocData,
    _HSD_GObjLibInitDataType,
};

pub const HSD_GOBJ_GXLINK_NONE: u8 = 0xFF;
pub const HSD_GOBJ_OBJ_NONE: u8 = 0xFF;
pub const HSD_GOBJ_USER_DATA_NONE: u8 = 0xFF;
pub const HSD_GOBJ_PLINK_MAX: u8 = 63;
pub const HSD_GOBJ_GX_LINK_MAX: u8 = 63;
pub const HSD_GOBJPROC_PRI_MAX: u8 = 2;

/// `_unk_gobj_struct HSD_GObj_DelayedProcInfo` with the C's native (LSB-first) bit order.
#[repr(C)]
pub struct DelayedProcInfo {
    /// bit 0 `in_delayed_proc`, bit 1 `delay_remove_gobj`, bit 2 `delay_remove_proc`, bit 3 `delay_change_gobj_pri`
    pub flags: u32,
    pub r#type: u32,
    pub p_link: u8,
    pub p_prio: u8,
    pub gobj: *mut HSD_GObj,
}
impl DelayedProcInfo {
    pub const IN_DELAYED_PROC: u32 = 1;
    pub const DELAY_REMOVE_GOBJ: u32 = 2;
    pub const DELAY_REMOVE_PROC: u32 = 4;
    pub const DELAY_CHANGE_GOBJ_PRI: u32 = 8;
}

pub static mut HSD_GObj_DelayedProcInfo: DelayedProcInfo =
    DelayedProcInfo { flags: 0, r#type: 0, p_link: 0, p_prio: 0, gobj: null_mut() };
pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = null_mut();
pub static mut plinklow_gobjs: *mut *mut HSD_GObj = null_mut();
pub static mut HSD_GObjGXLinkHead: *mut *mut HSD_GObj = null_mut();
pub static mut HSD_GObj_804D7820: *mut *mut HSD_GObj = null_mut();
pub static mut HSD_GObj_ProcList: *mut *mut HSD_GObjProc = null_mut();
pub static mut HSD_GObj_GObjProcHead: *mut *mut HSD_GObjProc = null_mut();
pub static mut HSD_GObj_804D783C: i32 = 0;
pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = null_mut();
pub static mut HSD_GObj_CurrentInvokedSLink: i32 = 0;
pub static mut HSD_GObj_NextInvokedProc: *mut HSD_GObjProc = null_mut();
pub static mut HSD_GObj_CurrentInvokedProcGObj: *mut HSD_GObj = null_mut();
pub static mut HSD_GObj_804D7818: *mut HSD_GObj = null_mut();
pub static mut HSD_GObj_804D7814: *mut HSD_GObj = null_mut();
pub static mut HSD_GObj_804D7810: *mut GObjFunc = null_mut();
pub static mut HSD_GObj_CameraKind: u8 = 0;
pub static mut HSD_GObj_LightKind: i8 = 0;
pub static mut HSD_GObj_JObjKind: u8 = 0;
pub static mut HSD_GObj_FogKind: i8 = 0;
pub static mut gobj_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut gobjproc_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut HSD_GObjLibInitData: _HSD_GObjLibInitDataType =
    _HSD_GObjLibInitDataType { p_link_max: 0, gx_link_max: 0, gproc_pri_max: 0, funcs: null_mut(), unk_2: null_mut() };

/// Callees outside the library: the TARGET_PC script hooks of the stage-slot host. Unset, a
/// stage slot never vetoes a creation and nothing is notified.
pub mod ext {
    use super::*;
    pub static mut ScriptGame_StageSlotCanCreate: Option<fn() -> i32> = None;
    pub static mut ScriptGame_StageSlotCreated: Option<fn(*mut HSD_GObj)> = None;
    pub static mut ScriptGame_StageSlotDestroyed: Option<fn(*mut HSD_GObj)> = None;
    pub static mut ScriptGame_StageSlotProcBegin: Option<fn(*mut HSD_GObj) -> i32> = None;
    pub static mut ScriptGame_StageSlotProcEnd: Option<fn()> = None;
    /// `ScriptStageHostHidden`
    pub static mut ScriptStageHostHidden: i32 = 0;
}

macro_rules! hsd_assert {
    ($file:literal, $line:expr, $cond:expr) => {
        if !($cond) {
            panic!(concat!($file, ":{} HSD_ASSERT(", stringify!($cond), ")"), $line);
        }
    };
}

/// `plink[i]` / `gxlink[i]` slot helpers.
#[inline]
unsafe fn at<T>(base: *mut *mut T, i: usize) -> *mut *mut T {
    base.add(i)
}

// ---------------------------------------------------------------------------------- gobjinit.c

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

/// `ref_DEC` (object.h): true when the object is not reference counted (`NOREF`) or this was the last reference.
unsafe fn ref_dec(o: *mut melee_types::HSD_Obj) -> bool {
    if (*o).ref_count == 0xFFFF {
        return true;
    }
    let was = (*o).ref_count;
    (*o).ref_count = was.wrapping_sub(1);
    was == 0
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

/// `HSD_GObj_80391260`: register the four built-in kinds (camera, light, jobj, fog) after the
/// caller's own function tables. `lobj_remove` / `jobj_remove` are `HSD_LObjRemoveAll` /
/// `HSD_JObjRemoveAll`, the other two are [`HSD_GObj_80391120`] / [`HSD_GObj_803911C0`].
pub static mut HSD_GObj_80408600: [GObjFunc; 4] = [None, None, None, None];
pub static mut HSD_GObj_80408610: GObjFuncs = GObjFuncs { next: null_mut(), size: 4, funcs: null_mut() };

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

// ---------------------------------------------------------------------------------- gobjplink.c

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
    if let Some(f) = ext::ScriptGame_StageSlotCanCreate {
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
    if let Some(f) = ext::ScriptGame_StageSlotCreated {
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
    if std::env::var_os("MELEE_DBG_FREE").is_some() { eprintln!("GObjFree {:p} class {} p_link {}
{}", gobj, (*gobj).classifier, (*gobj).p_link, std::backtrace::Backtrace::force_capture()); }
    if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::IN_DELAYED_PROC == 0 && gobj == HSD_GObj_CurrentInvokedProcGObj {
        HSD_GObj_DelayedProcInfo.flags |= DelayedProcInfo::DELAY_REMOVE_GOBJ;
        return;
    }
    if let Some(f) = ext::ScriptGame_StageSlotDestroyed {
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

// --------------------------------------------------------------------------------- gobjgxlink.c

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

unsafe fn gx_find_prio_position(gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    let mut i = *at(HSD_GObjGXLinkHead, (*gobj).gx_link as usize);
    while !i.is_null() && (*i).render_priority < (*gobj).render_priority {
        i = (*i).next_gx;
    }
    i
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

// ---------------------------------------------------------------------------------- gobjproc.c

#[inline]
unsafe fn proc_slot(p_link: usize, s_link: usize) -> *mut *mut HSD_GObjProc {
    HSD_GObj_ProcList.add(p_link + s_link * (HSD_GObjLibInitData.p_link_max as usize + 1))
}

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

// -------------------------------------------------------------------------------- gobjobject.c

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

// ------------------------------------------------------------------------------ gobjuserdata.c

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

// ------------------------------------------------------------------------------- gobj.c (procs)

/// `HSD_GObj_80390C5C`: set `flags_1` on every proc of `gobj`.
pub unsafe fn HSD_GObj_80390C5C(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObj_80390C5C");
    let mut p = (*gobj).proc;
    while !p.is_null() {
        (*p).set_flags_1(1);
        p = (*p).child;
    }
}

/// `HSD_GObj_80390C84`: clear `flags_1` on every proc of `gobj`.
pub unsafe fn HSD_GObj_80390C84(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObj_80390C84");
    let mut p = (*gobj).proc;
    while !p.is_null() {
        (*p).set_flags_1(0);
        p = (*p).child;
    }
}

/// `HSD_GObj_80390CAC`: clear `flags_2` on every proc of `gobj`.
pub unsafe fn HSD_GObj_80390CAC(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObj_80390CAC");
    let mut p = (*gobj).proc;
    while !p.is_null() {
        (*p).set_flags_2(0);
        p = (*p).child;
    }
}

/// `HSD_GObj_80390CD4`: stamp `flags_3` of every proc of `gobj` with the frame phase.
pub unsafe fn HSD_GObj_80390CD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("HSD_GObj_80390CD4");
    let mut p = (*gobj).proc;
    while !p.is_null() {
        (*p).set_flags_3(HSD_GObj_804D783C as u8);
        p = (*p).child;
    }
}

/// `HSD_GObj_RunProcs` (the GObj_RunProcs of the frame): run every due proc in link order.
pub unsafe fn HSD_GObj_RunProcs() {
    melee_core::ct!("HSD_GObj_RunProcs");
    let var_r31: u64 = if !HSD_GObjLibInitData.unk_2.is_null() { *HSD_GObjLibInitData.unk_2 } else { 0 };
    HSD_GObj_804D783C += 1;
    if HSD_GObj_804D783C > 2 {
        HSD_GObj_804D783C = 0;
    }
    let mut i = 0i32;
    while i <= HSD_GObjLibInitData.gproc_pri_max as i32 {
        HSD_GObj_CurrentInvokedSLink = i;
        let mut proc = *at(HSD_GObj_GObjProcHead, i as usize);
        while !proc.is_null() {
            HSD_GObj_NextInvokedProc = (*proc).next;
            if (*proc).flags_3() as i32 != HSD_GObj_804D783C {
                (*proc).set_flags_3(HSD_GObj_804D783C as u8);
                let gobj = (*proc).gobj;
                if var_r31 & (1u64 << (*gobj).p_link) == 0 && (*proc).flags_1() == 0 && (*proc).flags_2() == 0 {
                    HSD_GObj_CurrentInvokedProcGObj = gobj;
                    HSD_GObj_CurrentInvokedProc = proc;
                    let owned = match ext::ScriptGame_StageSlotProcBegin {
                        Some(f) => f(gobj),
                        None => 0,
                    };
                    if ext::ScriptStageHostHidden == 0 || (*gobj).classifier != 3 || owned != 0 {
                        if let Some(cb) = (*proc).on_invoke {
                            cb((*proc).gobj);
                        }
                    }
                    if owned != 0 {
                        if let Some(f) = ext::ScriptGame_StageSlotProcEnd {
                            f();
                        }
                    }
                    HSD_GObj_NextInvokedProc = (*proc).next;
                    if HSD_GObj_DelayedProcInfo.flags != 0 {
                        HSD_GObj_DelayedProcInfo.flags |= DelayedProcInfo::IN_DELAYED_PROC;
                        if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::DELAY_REMOVE_GOBJ != 0 {
                            HSD_GObjFree((*proc).gobj);
                        } else {
                            if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::DELAY_CHANGE_GOBJ_PRI != 0 {
                                HSD_GObjPLink_ChangeGObjPri_Unk(
                                    HSD_GObj_DelayedProcInfo.r#type,
                                    (*proc).gobj,
                                    HSD_GObj_DelayedProcInfo.p_link,
                                    HSD_GObj_DelayedProcInfo.p_prio,
                                    HSD_GObj_DelayedProcInfo.gobj,
                                );
                            }
                            if HSD_GObj_DelayedProcInfo.flags & DelayedProcInfo::DELAY_REMOVE_PROC != 0 {
                                HSD_GObjProc_RemoveProc(proc);
                            }
                        }
                        HSD_GObj_DelayedProcInfo.flags = 0;
                    }
                    HSD_GObj_CurrentInvokedProcGObj = null_mut();
                    HSD_GObj_CurrentInvokedProc = null_mut();
                }
            }
            proc = HSD_GObj_NextInvokedProc;
        }
        i += 1;
    }
}

// ------------------------------------------------------------------------ gobj.c (render walk)

/// `HSD_RandRenderBegin` / `HSD_RandRenderEnd` (TARGET_PC render-pass random stream guards).
pub static mut HSD_RandRenderBegin: Option<fn()> = None;
pub static mut HSD_RandRenderEnd: Option<fn()> = None;

/// `HSD_GObj_80390EB8` (`GObj_GetFlagFromArray`).
pub fn HSD_GObj_80390EB8(i: i32) -> u32 {
    melee_core::ct!("HSD_GObj_80390EB8");
    const HSD_GOBJ_804085F0: [u32; 4] = [1, 4, 2, 0];
    HSD_GOBJ_804085F0[i as usize]
}

/// `HSD_GObj_80390ED0` (`GObj_SetTextureCamera`): call the render callbacks of every GX link the
/// camera `gobj` selects (`gxlink_prios`), for each camera index set in `mask`.
pub unsafe fn HSD_GObj_80390ED0(gobj: *mut HSD_GObj, mut mask: u32) {
    melee_core::ct!("HSD_GObj_80390ED0");
    if HEADLESS_RENDER {
        return;
    }
    let mut i = 0i32;
    if let Some(f) = HSD_RandRenderBegin {
        f();
    }
    while mask != 0 {
        if mask & 1 != 0 {
            let mut prios = (*gobj).gxlink_prios;
            let mut j = 0usize;
            while prios != 0 {
                if prios & 1 != 0 {
                    let mut cur = *at(HSD_GObjGXLinkHead, j);
                    while !cur.is_null() {
                        if let Some(cb) = (*cur).render_cb {
                            let saved = HSD_GObj_804D7814;
                            HSD_GObj_804D7814 = cur;
                            cb(cur, i);
                            HSD_GObj_804D7814 = saved;
                        }
                        cur = (*cur).next_gx;
                    }
                }
                j += 1;
                prios >>= 1;
            }
        }
        i += 1;
        mask >>= 1;
    }
    if let Some(f) = HSD_RandRenderEnd {
        f();
    }
}

/// `HSD_GObj_80390FC0` (`GObj_RunGXLinkMaxCallbacks`): the max-link render callbacks.
/// The headless engine draws nothing: the render passes return at once (the renderer step lifts this).
pub static mut HEADLESS_RENDER: bool = false;

pub unsafe fn HSD_GObj_80390FC0() {
    melee_core::ct!("HSD_GObj_80390FC0");
    if HEADLESS_RENDER {
        return;
    }
    if let Some(f) = HSD_RandRenderBegin {
        f();
    }
    let mut cur = *at(HSD_GObjGXLinkHead, HSD_GObjLibInitData.gx_link_max as usize + 1);
    while !cur.is_null() {
        if let Some(cb) = (*cur).render_cb {
            let saved = HSD_GObj_804D7818;
            HSD_GObj_804D7818 = cur;
            cb(cur, 0);
            HSD_GObj_804D7818 = saved;
        }
        cur = (*cur).next_gx;
    }
    if let Some(f) = HSD_RandRenderEnd {
        f();
    }
}

/// `HSD_GObj_803910D8`: render the scene for a camera GObj (set current, walk, end).
pub unsafe fn HSD_GObj_803910D8(gobj: *mut HSD_GObj, _renderpass: i32) {
    melee_core::ct!("HSD_GObj_803910D8");
    if super::cobj::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) {
        HSD_GObj_80390ED0(gobj, 7);
        super::cobj::HSD_CObjEndCurrent();
    }
}
