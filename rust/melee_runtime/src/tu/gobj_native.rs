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
pub(crate) unsafe fn at<T>(base: *mut *mut T, i: usize) -> *mut *mut T {
    base.add(i)
}



// ---------------------------------------------------------------------------------- gobjinit.c












/// `HSD_GObj_80391260`: register the four built-in kinds (camera, light, jobj, fog) after the
/// caller's own function tables. `lobj_remove` / `jobj_remove` are `HSD_LObjRemoveAll` /
/// `HSD_JObjRemoveAll`, the other two are [`HSD_GObj_80391120`] / [`HSD_GObj_803911C0`].
pub static mut HSD_GObj_80408600: [GObjFunc; 4] = [None, None, None, None];

pub static mut HSD_GObj_80408610: GObjFuncs = GObjFuncs { next: null_mut(), size: 4, funcs: null_mut() };







// ---------------------------------------------------------------------------------- gobjplink.c
















// --------------------------------------------------------------------------------- gobjgxlink.c








pub(crate) unsafe fn gx_find_prio_position(gobj: *mut HSD_GObj) -> *mut HSD_GObj {
    let mut i = *at(HSD_GObjGXLinkHead, (*gobj).gx_link as usize);
    while !i.is_null() && (*i).render_priority < (*gobj).render_priority {
        i = (*i).next_gx;
    }
    i
}











// ---------------------------------------------------------------------------------- gobjproc.c


#[inline]
pub(crate) unsafe fn proc_slot(p_link: usize, s_link: usize) -> *mut *mut HSD_GObjProc {
    HSD_GObj_ProcList.add(p_link + s_link * (HSD_GObjLibInitData.p_link_max as usize + 1))
}















// -------------------------------------------------------------------------------- gobjobject.c










// ------------------------------------------------------------------------------ gobjuserdata.c






// ------------------------------------------------------------------------------- gobj.c (procs)














// ------------------------------------------------------------------------ gobj.c (render walk)


/// `HSD_RandRenderBegin` / `HSD_RandRenderEnd` (TARGET_PC render-pass random stream guards).
pub static mut HSD_RandRenderBegin: Option<fn()> = None;

pub static mut HSD_RandRenderEnd: Option<fn()> = None;







/// `HSD_GObj_80390FC0` (`GObj_RunGXLinkMaxCallbacks`): the max-link render callbacks.
/// Headless engine (`MELEE_ENGINE_HEADLESS`): no GX bridge and no presented frame, but the render callbacks still run, in
/// render-stub mode (display calls answer nothing), like the C build's turbo frames (`Gx_SuppressDraws`, the callbacks run
/// without display lists). They must: the draw pass has simulation-visible effects. `HSD_JObjDispAll` sets up the world
/// matrices and clears `JOBJ_MTX_DIRTY`, and `HSD_JObjSetMtxDirty` only propagates to the children (`HSD_JObjSetMtxDirtySub`)
/// from a clean parent; skipped, the fighters' child bones kept stale matrices and the bone dynamics drifted
/// (cpu_fox_falco_dl frame 142). The main camera's `fn_800301D0` also copies the camera pose into the CObj the offscreen
/// tests read (`Camera_8002A4AC`, `HSD_CObjSetCurrent`).
pub static mut HEADLESS_RENDER: bool = false;







pub(crate) use hsd_assert;

pub use super::gobjinit::{HSD_GObjSetInitDefaults, HSD_GObjInit};
pub use super::gobj::{HSD_GObj_803912A8, HSD_GObj_80391120, HSD_GObj_803911C0, HSD_GObj_80391260, HSD_GObj_80390C5C, HSD_GObj_80390C84, HSD_GObj_80390CAC, HSD_GObj_80390CD4, HSD_GObj_RunProcs, HSD_GObj_80390EB8, HSD_GObj_80390ED0, HSD_GObj_80390FC0, HSD_GObj_803910D8};
pub use super::gobjplink::{GObj_PReorder, CreateGObj, GObj_Create, HSD_GObjFree, HSD_GObjPLink_ChangeGObjPri_Unk};
pub use super::gobjgxlink::{GObj_GXReorder, GObj_SetupGXLink, GObj_SetupGXLinkMax, GObj_SetupGXLinkMaxSorted, HSD_GObjGXLink_8039084C, HSD_GObjGXLink_80390908, HSD_GObjGXLink_803909D8};
pub use super::gobjproc::{HSD_GObjProc_QueueProc, HSD_GObjProc_UnqueueProc, HSD_GObjProc_UnlinkProcFromGObj, HSD_GObj_SetupProc, HSD_GObjProc_RemoveProc, HSD_GObjProc_RemoveAllProcs};
pub use super::gobjobject::{HSD_GObjObject_80390A3C, HSD_GObjObject_80390A70, HSD_GObjObject_80390ADC, HSD_GObjObject_80390B0C};
pub use super::gobjuserdata::{GObj_InitUserData, GObj_RemoveUserData};
