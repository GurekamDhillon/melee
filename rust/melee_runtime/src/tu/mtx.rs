//! mtx.c: reuse the C-oracle-verified arithmetic; complete the pool wrappers.
#![allow(non_snake_case, non_upper_case_globals)]
use core::{ffi::c_void, ptr::addr_of_mut};
#[allow(unused_imports)]
use melee_types::*;
pub use crate::hsd_mtx::{calc_val as calcVal, determinant3x4 as HSD_CalcDeterminantMatrix3x4, inverse as HSD_MtxInverse, inverse_in_place as HSD_MtxInverseInPlace, inverse_concat as HSD_MtxInverseConcat, get_rotation as HSD_MtxGetRotation, get_scale as HSD_MtxGetScale, mtx_quat as HSD_MtxQuat, srt as HSD_MtxSRT};
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void = crate::tu::objalloc::HSD_ObjAlloc(a0);
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = crate::tu::objalloc::HSD_ObjAllocInit(a0, a1 as _, a2);
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void) = crate::tu::objalloc::HSD_ObjFree(a0, a1);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXIdentity(a0: *mut c_void);
    fn PSMTXQuat(a0: *mut c_void, a1: *mut Quaternion);
    fn PSMTXScale(a0: *mut c_void, a1: f32, a2: f32, a3: f32);
    fn PSMTXTrans(a0: *mut c_void, a1: f32, a2: f32, a3: f32);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECDotProduct(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn PSVECMag(a0: *mut Vec) -> f32;
    fn PSVECNormalize(a0: *mut Vec, a1: *mut Vec);
    fn PSVECScale(a0: *mut Vec, a1: *mut Vec, a2: f32);
    fn PSVECSubtract(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn asinf(a0: f32) -> f32;
    fn atan2f(a0: f32, a1: f32) -> f32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT
pub static mut HSD_Mtx_804C2310: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut HSD_Mtx_804C233C: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub unsafe fn HSD_VecAlloc() -> *mut c_void {
    melee_core::ct!("HSD_VecAlloc");
    let v = ext::HSD_ObjAlloc(addr_of_mut!(HSD_Mtx_804C2310));
    assert!(!v.is_null(), "HSD_VecAlloc: vec"); v
}
pub unsafe fn HSD_VecFree(v: *mut c_void) { melee_core::ct!("HSD_VecFree"); if !v.is_null() { ext::HSD_ObjFree(addr_of_mut!(HSD_Mtx_804C2310), v); } }
pub unsafe fn HSD_MtxAlloc() -> *mut c_void {
    melee_core::ct!("HSD_MtxAlloc");
    let v = ext::HSD_ObjAlloc(addr_of_mut!(HSD_Mtx_804C233C));
    assert!(!v.is_null(), "HSD_MtxAlloc: mtx"); v
}
pub unsafe fn HSD_MtxFree(v: *mut c_void) { melee_core::ct!("HSD_MtxFree"); if !v.is_null() { ext::HSD_ObjFree(addr_of_mut!(HSD_Mtx_804C233C), v); } }
pub unsafe fn HSD_VecGetAllocData() -> *mut HSD_ObjAllocData { melee_core::ct!("HSD_VecGetAllocData"); addr_of_mut!(HSD_Mtx_804C2310) }
pub unsafe fn HSD_MtxGetAllocData() -> *mut HSD_ObjAllocData { melee_core::ct!("HSD_MtxGetAllocData"); addr_of_mut!(HSD_Mtx_804C233C) }
pub unsafe fn HSD_VecInitAllocData() { melee_core::ct!("HSD_VecInitAllocData"); ext::HSD_ObjAllocInit(HSD_VecGetAllocData(), core::mem::size_of::<Vec3>() as u32, 4); }
pub unsafe fn HSD_MtxInitAllocData() { melee_core::ct!("HSD_MtxInitAllocData"); ext::HSD_ObjAllocInit(HSD_MtxGetAllocData(), core::mem::size_of::<Mtx>() as u32, 4); }

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
