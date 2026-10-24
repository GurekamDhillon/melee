//! `src/sysdolphin/baselib/fobj.c`: keyframe interpreter. The interpreter, state accessors and
//! list walkers are native ports in `crate::fobj` (oracle `tests/oracle_anim.rs`); they are
//! re-exported here. The GameCube-layout lifecycle (allocator pool, Remove / RemoveAll / Alloc /
//! Free / LoadDesc on `melee_types::HSD_FObj`) is ported below with the allocator as a TODO stub.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]
pub use crate::fobj::{
    interpret_anim_all as HSD_FObjInterpretAnimAll, load_desc_list as HSD_FObjLoadDescList,
    req_anim_all as HSD_FObjReqAnimAll, stop_anim_all as HSD_FObjStopAnimAll, HsdFObj,
};
use crate::cabi::f2i16;
use core::ptr::{null_mut, write_bytes};
use melee_types::*;

/// Callees in unported TUs (objalloc.c).
pub mod ext {
    use crate::tu::hookmac::hook;
    use melee_types::*;
    hook!(HSD_OBJALLOCINIT, HSD_ObjAllocInit, (data: *mut HSD_ObjAllocData, size: u32, align: u32));
    hook!(HSD_OBJALLOC, HSD_ObjAlloc, (data: *mut HSD_ObjAllocData) -> *mut core::ffi::c_void);
    hook!(HSD_OBJFREE, HSD_ObjFree, (data: *mut HSD_ObjAllocData, obj: *mut core::ffi::c_void));
}

/// `fobj_alloc_data`.
pub static mut fobj_alloc_data: core::mem::MaybeUninit<HSD_ObjAllocData> = core::mem::MaybeUninit::zeroed();

/// `HSD_FObjGetAllocData`.
pub unsafe fn HSD_FObjGetAllocData() -> *mut HSD_ObjAllocData {
    fobj_alloc_data.as_mut_ptr()
}

/// `HSD_FObjInitAllocData`: pool of `sizeof(HSD_FObj)` (0x30), align 4.
pub unsafe fn HSD_FObjInitAllocData() {
    ext::HSD_ObjAllocInit(HSD_FObjGetAllocData(), 0x30, 4);
}

/// `HSD_FObjSetState`.
pub unsafe fn HSD_FObjSetState(fobj: *mut HSD_FObj, state: u32) -> u32 {
    if !fobj.is_null() {
        (*fobj).flags = (state & 0xF) as u8 | ((*fobj).flags & 0xF0);
    }
    state
}

/// `HSD_FObjGetState`.
pub unsafe fn HSD_FObjGetState(fobj: *mut HSD_FObj) -> u32 {
    if fobj.is_null() {
        return 0;
    }
    ((*fobj).flags & 0xF) as u32
}

/// `HSD_FObjRemove`.
pub unsafe fn HSD_FObjRemove(fobj: *mut HSD_FObj) {
    if fobj.is_null() {
        return;
    }
    HSD_FObjFree(fobj);
}

/// `HSD_FObjRemoveAll`: frees the tail first, as the C recursion does.
pub unsafe fn HSD_FObjRemoveAll(fobj: *mut HSD_FObj) {
    if fobj.is_null() {
        return;
    }
    HSD_FObjRemoveAll((*fobj).next);
    HSD_FObjRemove(fobj);
}

/// `HSD_FObjAlloc`: pool alloc, zeroed.
pub unsafe fn HSD_FObjAlloc() -> *mut HSD_FObj {
    let new = ext::HSD_ObjAlloc(HSD_FObjGetAllocData()) as *mut HSD_FObj;
    assert!(!new.is_null(), "fobj.c:0x2F3 new");
    write_bytes(new, 0, 1);
    new
}

/// `HSD_FObjFree`.
pub unsafe fn HSD_FObjFree(fobj: *mut HSD_FObj) {
    ext::HSD_ObjFree(HSD_FObjGetAllocData(), fobj as *mut core::ffi::c_void);
}

/// `HSD_FObjLoadDesc`: builds the list from a descriptor chain (`next` loaded first, as the C).
pub unsafe fn HSD_FObjLoadDesc(desc: *mut HSD_FObjDesc) -> *mut HSD_FObj {
    if desc.is_null() {
        return null_mut();
    }
    let fobj = HSD_FObjAlloc();
    (*fobj).next = HSD_FObjLoadDesc((*desc).next);
    (*fobj).startframe = f2i16((*desc).startframe as f64);
    (*fobj).obj_type = (*desc).r#type;
    (*fobj).frac_value = (*desc).frac_value;
    (*fobj).frac_slope = (*desc).frac_slope;
    (*fobj).ad_head = (*desc).ad;
    (*fobj).length = (*desc).length;
    (*fobj).flags = 0;
    fobj
}
