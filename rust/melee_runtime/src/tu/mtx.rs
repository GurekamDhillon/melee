//! mtx.c: reuse the C-oracle-verified arithmetic; complete the pool wrappers.
#![allow(non_snake_case, non_upper_case_globals)]
use core::{ffi::c_void, ptr::addr_of_mut};
use melee_types::{HSD_ObjAllocData, Vec3, Mtx};
use super::objalloc::{HSD_ObjAlloc, HSD_ObjFree, HSD_ObjAllocInit};
pub use crate::hsd_mtx::{calc_val as calcVal, determinant3x4 as HSD_CalcDeterminantMatrix3x4, inverse as HSD_MtxInverse, inverse_in_place as HSD_MtxInverseInPlace, inverse_concat as HSD_MtxInverseConcat, get_rotation as HSD_MtxGetRotation, get_scale as HSD_MtxGetScale, mtx_quat as HSD_MtxQuat, srt as HSD_MtxSRT};
pub static mut HSD_Mtx_804C2310: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut HSD_Mtx_804C233C: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub unsafe fn HSD_VecAlloc() -> *mut c_void {
    let v = HSD_ObjAlloc(addr_of_mut!(HSD_Mtx_804C2310));
    assert!(!v.is_null(), "HSD_VecAlloc: vec"); v
}
pub unsafe fn HSD_VecFree(v: *mut c_void) { if !v.is_null() { HSD_ObjFree(addr_of_mut!(HSD_Mtx_804C2310), v); } }
pub unsafe fn HSD_MtxAlloc() -> *mut c_void {
    let v = HSD_ObjAlloc(addr_of_mut!(HSD_Mtx_804C233C));
    assert!(!v.is_null(), "HSD_MtxAlloc: mtx"); v
}
pub unsafe fn HSD_MtxFree(v: *mut c_void) { if !v.is_null() { HSD_ObjFree(addr_of_mut!(HSD_Mtx_804C233C), v); } }
pub unsafe fn HSD_VecGetAllocData() -> *mut HSD_ObjAllocData { addr_of_mut!(HSD_Mtx_804C2310) }
pub unsafe fn HSD_MtxGetAllocData() -> *mut HSD_ObjAllocData { addr_of_mut!(HSD_Mtx_804C233C) }
pub unsafe fn HSD_VecInitAllocData() { HSD_ObjAllocInit(HSD_VecGetAllocData(), core::mem::size_of::<Vec3>(), 4); }
pub unsafe fn HSD_MtxInitAllocData() { HSD_ObjAllocInit(HSD_MtxGetAllocData(), core::mem::size_of::<Mtx>(), 4); }

#[allow(non_snake_case, unused_imports, clippy::missing_safety_doc)]
pub mod cnames {
    use crate::mtx::Quaternion;
    use melee_core::vec::Vec3;
    #[allow(unused_imports)]
    use crate::hsd_mtx::*;
    use super::*;
    /// `HSD_MkRotationMtx`: the C entry point over `crate::hsd_mtx::mk_rotation_mtx` (the owned object model).
    #[inline]
    pub fn HSD_MkRotationMtx(r: &Vec3) -> Mtx {
        crate::hsd_mtx::mk_rotation_mtx(r)
    }
    /// `HSD_MtxInverseTranspose`: the C entry point over `crate::hsd_mtx::inverse_transpose` (the owned object model).
    #[inline]
    pub fn HSD_MtxInverseTranspose(src: &Mtx) -> Mtx {
        crate::hsd_mtx::inverse_transpose(src)
    }
    /// `HSD_MtxGetTranslate`: the C entry point over `crate::hsd_mtx::get_translate` (the owned object model).
    #[inline]
    pub fn HSD_MtxGetTranslate(m: &Mtx) -> Vec3 {
        crate::hsd_mtx::get_translate(m)
    }
    /// `HSD_MtxScaledAdd`: the C entry point over `crate::hsd_mtx::scaled_add` (the owned object model).
    #[inline]
    pub fn HSD_MtxScaledAdd(a: &Mtx, b: &Mtx, s: f32) -> Mtx {
        crate::hsd_mtx::scaled_add(a, b, s)
    }
    /// `HSD_MtxSRTQuat`: the C entry point over `crate::hsd_mtx::srt_quat` (the owned object model).
    #[inline]
    pub fn HSD_MtxSRTQuat(s: &Vec3, q: &Quaternion, t: &Vec3, ps: Option<&Vec3>) -> Mtx {
        crate::hsd_mtx::srt_quat(s, q, t, ps)
    }
}
pub use cnames::*;
