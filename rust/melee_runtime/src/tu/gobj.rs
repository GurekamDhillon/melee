//! `src/sysdolphin/baselib/gobj.c`: the per-frame proc walk, the camera / max-link render walks,
//! the obj-kind callbacks and the library's kind-table setup.
//!
//! Native ports on [`GObjSystem`] (`crate::gobj`, oracle `tests/oracle_gobj.rs`): the C names for
//! them are exposed here. The obj-kind callbacks below operate on the GameCube-layout record
//! (`melee_types::HSD_GObj`), since they are what the obj-kind table holds for camera / light /
//! jobj / fog objects. Callees in unported TUs are TODO stubs in [`ext`].
//!
//! Not ported (observability only): `gobj_trace_render`, the profiler zones.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
use melee_types::inline::ref_DEC;
use melee_types::*;

/// Callees outside gobj.c.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    // rp3 HSD_LObj_803668EC
    hook_or!(HSD_LOBJ_803668EC, HSD_LObj_803668EC, (lobj: *mut HSD_LObj), |lobj: *mut HSD_LObj| unsafe { crate::tu::lobj::HSD_LObj_803668EC(lobj); });
    // rp3-end

    // rp3 HSD_LObjSetupInit
    hook_or!(HSD_LOBJSETUPINIT, HSD_LObjSetupInit, (cobj: *mut HSD_CObj), |cobj: *mut HSD_CObj| unsafe { crate::tu::lobj::HSD_LObjSetupInit(cobj); });
    // rp3-end

    pub static mut HSD_COBJGETCURRENT: Option<fn() -> *mut HSD_CObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjGetCurrent() -> *mut HSD_CObj {
        match unsafe { HSD_COBJGETCURRENT } { Some(f) => f(), None => { return unsafe { crate::tu::cobj::HSD_CObjGetCurrent() as _ }; } }
    }
    pub static mut HSD_COBJSETCURRENT: Option<fn(*mut HSD_CObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjSetCurrent(cobj: *mut HSD_CObj) -> i32 {
        match unsafe { HSD_COBJSETCURRENT } { Some(f) => f(cobj), None => { return unsafe { crate::tu::cobj::HSD_CObjSetCurrent(cobj as _) as _ }; } }
    }
    pub static mut HSD_COBJENDCURRENT: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn HSD_CObjEndCurrent() {
        match unsafe { HSD_COBJENDCURRENT } { Some(f) => f(), None => { unsafe { crate::tu::cobj::HSD_CObjEndCurrent(); } return; } }
    }
    pub static mut HSD_JOBJDISPALL: Option<fn(*mut HSD_JObj, *mut [f32; 4], u32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjDispAll(jobj: *mut HSD_JObj, vmtx: *mut [f32; 4], flags: u32, rendermode: u32) {
        match unsafe { HSD_JOBJDISPALL } {
            Some(f) => f(jobj, vmtx, flags, rendermode),
            None => { unsafe { crate::tu::jobj::HSD_JObjDispAll(jobj as _, vmtx as *const _, flags as _, rendermode as _); } },
        }
    }
    hook!(HSD_FOGSET, HSD_FogSet, (fog: *mut HSD_Fog));
    hook!(HSDDELETE, hsdDelete, (obj: *mut core::ffi::c_void));
}

use super::gobj_native::{self as gn, *};
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

/// `ref_DEC` (object.h): true when the object is not reference counted (`NOREF`) or this was the last reference.
pub(crate) unsafe fn ref_dec(o: *mut melee_types::HSD_Obj) -> bool {
    if (*o).ref_count == 0xFFFF {
        return true;
    }
    let was = (*o).ref_count;
    (*o).ref_count = was.wrapping_sub(1);
    was == 0
}

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
/// Debug aid (`MELEE_WATCH_PROC=<hex addr>`): reports the u32 there whenever it changed across a gobj proc (priority and
/// proc address), to find which proc overwrites a variable. Complements `MELEE_WATCH` (function entries, calltrace builds).
static mut WATCH_PROC: Option<(usize, u32)> = None;

static mut WATCH_PROC_INIT: bool = false;

#[allow(static_mut_refs)]
unsafe fn watch_proc(pri: i32, cb: usize, after: bool) {
    if !WATCH_PROC_INIT {
        WATCH_PROC_INIT = true;
        if let Some(a) = std::env::var("MELEE_WATCH_PROC").ok().and_then(|v| usize::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
            WATCH_PROC = Some((a, core::ptr::read_volatile(a as *const u32)));
        }
    }
    if let Some((a, old)) = WATCH_PROC.as_mut() {
        let now = core::ptr::read_volatile(*a as *const u32);
        if now != *old {
            eprintln!("watch-proc: {:#x} {:#010x} -> {:#010x} {} proc pri {pri} cb {cb:#x}", *a, *old, now, if after { "after" } else { "before" });
            *old = now;
        }
    }
}

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
                    let owned = match gn::ext::ScriptGame_StageSlotProcBegin {
                        Some(f) => f(gobj),
                        None => 0,
                    };
                    if gn::ext::ScriptStageHostHidden == 0 || (*gobj).classifier != 3 || owned != 0 {
                        if let Some(cb) = (*proc).on_invoke {
                            watch_proc(i, cb as usize, false);
                            cb((*proc).gobj);
                            watch_proc(i, cb as usize, true);
                        }
                    }
                    if owned != 0 {
                        if let Some(f) = gn::ext::ScriptGame_StageSlotProcEnd {
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

pub unsafe fn HSD_GObj_80390FC0() {
    melee_core::ct!("HSD_GObj_80390FC0");
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

// Split into src/sysdolphin/baselib/hsd_3910.c upstream (doldecomp 598987cf3); re-exported for existing callers.
pub use super::hsd_3910::{HSD_GObj_803910D8, HSD_GObj_80391120, HSD_GObj_803911C0, HSD_GObj_80391260, HSD_GObj_803912A8, HSD_GObj_FogCallback, HSD_GObj_JObjCallback, HSD_GObj_LObjCallback};
