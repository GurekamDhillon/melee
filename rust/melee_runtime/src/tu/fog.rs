//! `src/sysdolphin/baselib/fog.c`: HSD fog objects. Set / init / load-desc / anim / update are
//! native ports in `crate::fog` (oracle `tests/oracle_fog.rs`), re-exported under the C names.
//! The class machinery (`hsdFog` / `hsdFogAdj` infos, alloc through `hsdNew`, `FogRelease`) works
//! on the GameCube-layout records and is ported below; class-system callees are TODO stubs.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
pub use crate::fog::{
    fog_set as HSD_FogSet, req_anim as HSD_FogReqAnim, req_anim_by_flags as HSD_FogReqAnimByFlags, HsdFog,
    HsdFogAdj, HsdFogAdjDesc, HsdFogDesc,
};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::ref_DEC;
use melee_types::*;

/// Callees in unported TUs (class.c, object.c, aobj.c).
pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    hook!(HSDNEW, hsdNew, (info: *mut HSD_ClassInfo) -> *mut HSD_Class);
    hook!(HSDINITCLASSINFO, hsdInitClassInfo, (info: *mut HSD_ClassInfo, parent: *mut HSD_ClassInfo, lib: *const i8, name: *const i8, info_size: i32, obj_size: i32));
    hook!(HSDOBJ, hsdObj, () -> *mut HSD_ClassInfo);
    hook!(HSD_AOBJREMOVE, HSD_AObjRemove, (aobj: *mut HSD_AObj));
}

/// `hsdFog` (`{ FogInfoInit }`) and `hsdFogAdj`.
pub static mut hsdFog: HSD_FogInfo = unsafe { core::mem::zeroed() };
pub static mut hsdFogAdj: HSD_ClassInfo = unsafe { core::mem::zeroed() };

/// Stores `FogInfoInit` / `FogAdjInfoInit` as the infos' `info_init` (the static initializers).
pub unsafe fn init_fog_statics() {
    hsdFog.parent.parent.head.info_init = Some(FogInfoInit_c);
    hsdFogAdj.head.info_init = Some(FogAdjInfoInit_c);
}
unsafe extern "C" fn FogInfoInit_c() {
    FogInfoInit()
}
unsafe extern "C" fn FogAdjInfoInit_c() {
    FogAdjInfoInit()
}

/// `HSD_FogAlloc`.
pub unsafe fn HSD_FogAlloc() -> *mut HSD_Fog {
    let fog = ext::hsdNew(&mut hsdFog.parent.parent) as *mut HSD_Fog;
    assert!(!fog.is_null(), "fog.c:0x8C fog");
    fog
}

/// `HSD_FogAdjAlloc`.
pub unsafe fn HSD_FogAdjAlloc() -> *mut HSD_FogAdj {
    let adj = ext::hsdNew(&mut hsdFogAdj) as *mut HSD_FogAdj;
    assert!(!adj.is_null(), "fog.c:0xD6 adj");
    adj
}

/// `HSD_Fog_8037DE7C` (HSD_FogAddAnim): replace the fog's AObj with one loaded from `desc`.
/// `load` is `HSD_AObjLoadDesc` (aobj.c; native port `crate::aobj`).
pub unsafe fn HSD_Fog_8037DE7C(fog: *mut HSD_Fog, desc: *mut HSD_AObjDesc, load: impl FnOnce(*mut HSD_AObjDesc) -> *mut HSD_AObj) {
    if !fog.is_null() {
        if !(*fog).aobj.is_null() {
            ext::HSD_AObjRemove((*fog).aobj);
        }
        (*fog).aobj = load(desc);
    }
}

/// `FogRelease` (class `release`): drop the adj (ref_DEC, then its release + destroy), remove
/// the AObj, then the parent class's release.
pub unsafe extern "C" fn FogRelease(o: *mut HSD_Class) {
    let fog = o as *mut HSD_Fog;
    if !(*fog).fog_adj.is_null() {
        let adj = (*fog).fog_adj;
        if ref_DEC(&mut (*adj).parent) {
            let info = (*(adj as *mut HSD_Class)).class_info;
            if let Some(release) = (*info).release {
                release(adj as *mut HSD_Class);
            }
            if let Some(destroy) = (*info).destroy {
                destroy(adj as *mut HSD_Class);
            }
        }
    }
    ext::HSD_AObjRemove((*fog).aobj);
    let parent = hsdFog.parent.parent.head.parent;
    if let Some(release) = (*parent).release {
        release(o);
    }
}

/// `FogInfoInit`.
pub unsafe fn FogInfoInit() {
    ext::hsdInitClassInfo(
        &mut hsdFog.parent.parent,
        ext::hsdObj(),
        b"sysdolphin_base_library\0".as_ptr() as *const i8,
        b"hsd_fog\0".as_ptr() as *const i8,
        0x3C,
        <HSD_Fog as melee_core::gc::GcLayout>::GC_SIZE as i32,
    );
    hsdFog.parent.parent.release = Some(FogRelease);
}

/// `FogAdjInfoInit`.
pub unsafe fn FogAdjInfoInit() {
    ext::hsdInitClassInfo(
        &mut hsdFogAdj,
        ext::hsdObj(),
        b"sysdolphin_base_library\0".as_ptr() as *const i8,
        b"hsd_fogadj\0".as_ptr() as *const i8,
        0x3C,
        <HSD_FogAdj as melee_core::gc::GcLayout>::GC_SIZE as i32,
    );
}

/// `FogUpdateFunc(obj, type, val)`: native `HsdFog::update_func`.
pub fn FogUpdateFunc<O>(fog: &mut HsdFog<O>, ty: u32, fv: f32) {
    fog.update_func(ty, fv)
}

#[allow(unused)]
fn _unused(_: *mut c_void) -> *mut c_void {
    null_mut()
}
