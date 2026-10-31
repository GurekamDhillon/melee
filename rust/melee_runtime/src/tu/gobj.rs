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
use crate::gobj::{hsd_gobj_803912a8, hsd_gobj_80390eb8, GObjHost, GObjId, GObjSystem, HsdGObjLibInitData, KindFunc};
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
            None => { unsafe { crate::tu::jobj::HSD_JObjDispAll(jobj as _, &*(vmtx as *const _), flags as _, rendermode as _); } },
        }
    }
    hook!(HSD_FOGSET, HSD_FogSet, (fog: *mut HSD_Fog));
    hook!(HSDDELETE, hsdDelete, (obj: *mut core::ffi::c_void));
}

/// `HSD_GObj_80391120` (camera kind entry of the obj-kind table): HSD_Obj release. NULL is a
/// no-op; otherwise `ref_DEC(obj)` and, when it reports the last reference, `hsdDelete(obj)`.
pub unsafe fn HSD_GObj_80391120(obj: *mut HSD_Obj) {
    melee_core::ct!("HSD_GObj_80391120");
    if !obj.is_null() && ref_DEC(&mut *obj) {
        ext::hsdDelete(obj as *mut core::ffi::c_void);
    }
}

/// `HSD_GObj_803911C0`: the same release, as the fog kind's own table entry.
pub unsafe fn HSD_GObj_803911C0(obj: *mut HSD_Obj) {
    melee_core::ct!("HSD_GObj_803911C0");
    HSD_GObj_80391120(obj)
}

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
    ext::HSD_JObjDispAll(jobj, core::ptr::null_mut(), hsd_gobj_80390eb8(arg1), 0);
}

/// `HSD_GObj_FogCallback`.
pub unsafe fn HSD_GObj_FogCallback(gobj: *mut HSD_GObj, _unused: i32) {
    melee_core::ct!("HSD_GObj_FogCallback");
    ext::HSD_FogSet((*gobj).hsd_obj as *mut HSD_Fog);
}

/// `HSD_GObj_803910D8`: render callback of a camera GObj; `walk` is
/// `HSD_GObj_80390ED0(gobj, 7)` (the camera walk over the GObj system, which the caller owns).
pub unsafe fn HSD_GObj_803910D8(gobj: *mut HSD_GObj, _renderpass: i32, walk: impl FnOnce()) {
    melee_core::ct!("HSD_GObj_803910D8");
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj as *mut HSD_CObj) != 0 {
        walk();
        ext::HSD_CObjEndCurrent();
    }
}

/// `HSD_GObj_80390C5C`..`80390CD4`: proc-pause helpers.
pub fn HSD_GObj_80390C5C<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) {
    melee_core::ct!("HSD_GObj_80390C5C");
    sys.hsd_gobj_80390c5c(gobj)
}
pub fn HSD_GObj_80390C84<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) {
    melee_core::ct!("HSD_GObj_80390C84");
    sys.hsd_gobj_80390c84(gobj)
}
pub fn HSD_GObj_80390CAC<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) {
    melee_core::ct!("HSD_GObj_80390CAC");
    sys.hsd_gobj_80390cac(gobj)
}
pub fn HSD_GObj_80390CD4<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) {
    melee_core::ct!("HSD_GObj_80390CD4");
    sys.hsd_gobj_80390cd4(gobj)
}

/// `HSD_GObj_RunProcs`.
pub fn HSD_GObj_RunProcs<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, host: &mut impl GObjHost<F, O>) {
    melee_core::ct!("HSD_GObj_RunProcs");
    sys.run_procs(host)
}

/// `HSD_GObj_80390EB8` (GObj_GetFlagFromArray).
pub fn HSD_GObj_80390EB8(i: i32) -> u32 {
    melee_core::ct!("HSD_GObj_80390EB8");
    hsd_gobj_80390eb8(i)
}

/// `HSD_GObj_80390ED0` (GObj_SetTextureCamera).
pub fn HSD_GObj_80390ED0<F: Copy, O: Copy>(
    sys: &mut GObjSystem<F, O>,
    host: &mut impl GObjHost<F, O>,
    gobj: GObjId,
    mask: u32,
) {
    melee_core::ct!("HSD_GObj_80390ED0");
    sys.hsd_gobj_80390ed0(host, gobj, mask)
}

/// `HSD_GObj_80390FC0` (GObj_RunGXLinkMaxCallbacks).
pub fn HSD_GObj_80390FC0<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, host: &mut impl GObjHost<F, O>) {
    melee_core::ct!("HSD_GObj_80390FC0");
    sys.hsd_gobj_80390fc0(host)
}

/// `HSD_GObj_80391260`: append the built-in kind table and number the camera / light / jobj /
/// fog kinds.
pub fn HSD_GObj_80391260<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, arg0: &mut HsdGObjLibInitData<F>) {
    melee_core::ct!("HSD_GObj_80391260");
    sys.hsd_gobj_80391260(arg0)
}

/// `HSD_GObj_803912A8`: append `arg1` to the funcs chain; returns the previous function count.
pub fn HSD_GObj_803912A8<F>(arg0: &mut HsdGObjLibInitData<F>, arg1: std::vec::Vec<KindFunc<F>>) -> u8 {
    melee_core::ct!("HSD_GObj_803912A8");
    hsd_gobj_803912a8(arg0, arg1)
}
