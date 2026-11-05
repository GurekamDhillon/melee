//! Port of src/melee/lb/lbcollision.c (hit/hurt capsule tests, victim lists and the hitbox
//! debug-draw). Float expressions fuse exactly where the shipping build does
//! (`tools/rust-port/fma_sites.py src/melee/lb/lbcollision.c`: 80005C44 9, 80005EBC 9,
//! 80005FC0 5, 80006094 31+3 f64, end 2, 800067F8 23+2 f64, 800077A0 14, sqrDistance 2,
//! sqrtf_store 3 f64, 80006E58 37+3 f64, 80008FC8 2, 800096B4 2); every `fmadds`/`fmsubs` below
//! is one of those sites.
//!
//! Pointers into game structs stay native pointers. The GX/HSD/lb callees of other TUs are
//! `ext` hooks named like the C functions.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, unused_mut)]

use crate::gekko::sqrtf;
use crate::hsd_mtx;
use crate::lbvector;
use crate::mtx::{self, Mtx};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::{fmadds, fmsubs, frsqrte};
use melee_types::*;
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
    fn GXBegin(a0: u32, a1: u32, a2: u16);
    fn GXCallDisplayList(a0: *mut c_void, a1: u32);
    fn GXClearVtxDesc();
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXLoadNrmMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetAlphaUpdate(a0: u8);
    fn GXSetArray(a0: u32, a1: *const c_void, a2: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetChanAmbColor(a0: u32, a1: _GXColor);
    fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn GXSetChanMatColor(a0: u32, a1: _GXColor);
    fn GXSetColorUpdate(a0: u8);
    fn GXSetCullMode(a0: u32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetFog(a0: u32, a1: f32, a2: f32, a3: f32, a4: f32, a5: _GXColor);
    fn GXSetNumChans(a0: u8);
    fn GXSetNumTevStages(a0: u8);
    fn GXSetNumTexGens(a0: u8);
    fn GXSetTevAlphaIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevAlphaOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevClampMode(a0: i32, a1: i32);
    fn GXSetTevColor(a0: u32, a1: _GXColor);
    fn GXSetTevColorIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevColorOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXSetZCompLoc(a0: u8);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = crate::tu::cobj::HSD_CObjGetCurrent();
    fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut c_void) = crate::tu::cobj::HSD_CObjGetViewingMtx(a0 as _, a1 as _);
    fn HSD_ClearVtxDesc() = crate::tu::pobj::HSD_ClearVtxDesc();
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_MtxInverse(a0: *mut c_void, a1: *mut c_void) = { let r = crate::hsd_mtx::inverse(&*(a0 as *const [[f32; 4]; 3])); (a1 as *mut [[f32; 4]; 3]).write(r); };
    fn HSD_StateInitTev() = crate::tu::tev::HSD_StateInitTev(crate::tu::globals::render());
    fn HSD_StateInvalidate(a0: i32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _);
    fn PSMTXIdentity(a0: *mut c_void);
    fn PSMTXMultVec(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec) = { let v = crate::mtx::mult_vec(&*(a0 as *const [[f32; 4]; 3]), &crate::mtx::Vec3 { x: (*a1).x, y: (*a1).y, z: (*a1).z }); *a2 = Vec { x: v.x, y: v.y, z: v.z }; };
    fn PSMTXScale(a0: *mut c_void, a1: f32, a2: f32, a3: f32);
    fn PSMTXTrans(a0: *mut c_void, a1: f32, a2: f32, a3: f32);
    fn PSMTXTranspose(a0: *mut c_void, a1: *mut c_void);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECNormalize(a0: *mut Vec, a1: *mut Vec);
    fn lbAudioAx_80024184(a0: i32, a1: i32, a2: i32, a3: i32) -> i32;
    fn lbVector_AngleXY(a0: *mut Vec, a1: *mut Vec) -> f32;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = { let (jobj, offset, out) = (a0, a1, a2); crate::tu::lb_00B0::lb_8000B1CC(jobj, offset as _, out as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT
/// jobj.h `HSD_JObjSetupMatrix` (inline): rebuild when dirty and not user-defined.
unsafe fn jobj_setup_matrix(j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    let fl = (*j).flags;
    if fl & (1 << 23) == 0 && fl & (1 << 6) != 0 {
        ext::HSD_JObjSetupMatrixSub(j);
    }
}

type CVec3 = melee_core::vec::Vec3;
#[inline]
fn cv(v: &Vec3) -> CVec3 {
    CVec3 { x: v.x, y: v.y, z: v.z }
}
#[inline]
fn uv(v: CVec3) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

#[inline]
fn approximatelyZero(x: f32) -> bool {
    x < 0.00001f32 && x > -0.00001f32
}
#[inline]
fn isSmall(x: f32) -> bool {
    x < 0.00001f32 && x > -0.00001f32
}
/// `z*z + (x*x + y*y)` and `x*x + y*y + z*z` (both contract to the same pair of fusions).
#[inline]
fn sq3(x: f32, y: f32, z: f32) -> f32 {
    fmadds(z, z, fmadds(x, x, y * y))
}
/// `az*bz + (ax*bx + ay*by)` / `ax*bx + ay*by + az*bz`.
#[inline]
fn dot3(ax: f32, ay: f32, az: f32, bx: f32, by: f32, bz: f32) -> f32 {
    fmadds(az, bz, fmadds(ax, bx, ay * by))
}
/// `0.5 * a + b` with a double constant: an f64 fma, rounded to f32.
#[inline]
fn half_fma(a: f32, b: f32) -> f32 {
    0.5f64.mul_add(a as f64, b as f64) as f32
}
#[inline]
fn clamp01(s: f32) -> f32 {
    if s as f64 > 1.0 {
        1.0
    } else if (s as f64) < 0.0 {
        0.0
    } else {
        s
    }
}
/// The AABB broad phase on one axis: does the expanded segment s..e miss both p and q?
#[inline]
fn reject_axis(s: f32, e: f32, p: f32, q: f32, r: f32) -> bool {
    if s > e {
        let b = s + r;
        if b < p && b < q {
            return true;
        }
        let b = e - r;
        if b > p && b > q {
            return true;
        }
    } else {
        let b = s - r;
        if b > p && b > q {
            return true;
        }
        let b = e + r;
        if b < p && b < q {
            return true;
        }
    }
    false
}

#[inline]
unsafe fn jobj_mtx(j: *mut HSD_JObj) -> *const Mtx {
    jobj_setup_matrix(j); // jobj.h HSD_JObjGetMtxPtr
    (&raw const (*j).mtx) as *const Mtx
}

// ---- .sdata colours -------------------------------------------------------------------------
pub static lbColl_SpherePositions: [u8; 0x1A0] = [
    0x29, 0xCF, 0x2D, 0x41, 0x11, 0x51, 0x20, 0x00, 0x2D, 0x41, 0x1F, 0xFF, 0x29, 0xCF, 0xE7, 0x83,
    0x29, 0xCF, 0x00, 0x00, 0xC4, 0xE0, 0x18, 0x7D, 0x11, 0x51, 0x3B, 0x20, 0x11, 0x51, 0x11, 0x51,
    0xD2, 0xBF, 0x29, 0xCF, 0x00, 0x00, 0xD2, 0xBF, 0x2D, 0x41, 0x16, 0xA0, 0x18, 0x7D, 0x36, 0xA0,
    0x00, 0x00, 0x18, 0x7D, 0x3B, 0x20, 0x09, 0x5F, 0xC4, 0xE0, 0x16, 0xA0, 0x00, 0x00, 0x3B, 0x20,
    0x18, 0x7D, 0x09, 0x5F, 0x3B, 0x20, 0x16, 0xA0, 0x11, 0x51, 0x2D, 0x41, 0x29, 0xCF, 0x00, 0x00,
    0x2D, 0x41, 0x2D, 0x41, 0x29, 0xCF, 0x18, 0x7D, 0x29, 0xCF, 0x36, 0xA0, 0x18, 0x7D, 0x16, 0xA0,
    0x2D, 0x41, 0x00, 0x00, 0x2D, 0x41, 0x3B, 0x20, 0x00, 0x00, 0x18, 0x7D, 0x36, 0xA0, 0xE7, 0x83,
    0x16, 0xA0, 0x16, 0xA0, 0x3B, 0x20, 0x09, 0x5F, 0x20, 0x00, 0xD2, 0xBF, 0x1F, 0xFF, 0x29, 0xCF,
    0xD2, 0xBF, 0x11, 0x51, 0x16, 0xA0, 0xC4, 0xE0, 0x09, 0x5F, 0x11, 0x51, 0xC4, 0xE0, 0x11, 0x51,
    0x00, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0x18, 0x7D, 0x00, 0x00, 0x3B, 0x20, 0x16, 0xA0, 0xE7, 0x83,
    0x36, 0xA0, 0x00, 0x00, 0xE7, 0x83, 0x3B, 0x20, 0x00, 0x00, 0x3B, 0x20, 0xE7, 0x83, 0x00, 0x00,
    0x2D, 0x41, 0xD2, 0xBF, 0x00, 0x00, 0x18, 0x7D, 0xC4, 0xE0, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x01,
    0x00, 0x00, 0xE7, 0x83, 0xC4, 0xE0, 0x00, 0x00, 0xD2, 0xBF, 0xD2, 0xBF, 0x00, 0x00, 0xC4, 0xE0,
    0xE7, 0x83, 0x09, 0x5F, 0x3B, 0x20, 0xE9, 0x60, 0x11, 0x51, 0x2D, 0x41, 0xD6, 0x31, 0x16, 0xA0,
    0x18, 0x7D, 0xC9, 0x60, 0x18, 0x7D, 0x00, 0x00, 0xC4, 0xE0, 0x16, 0xA0, 0xE7, 0x83, 0xC9, 0x60,
    0x11, 0x51, 0xD2, 0xBF, 0xD6, 0x31, 0x09, 0x5F, 0xC4, 0xE0, 0xE9, 0x60, 0x11, 0x51, 0x3B, 0x20,
    0xEE, 0xAF, 0x1F, 0xFF, 0x2D, 0x41, 0xE0, 0x00, 0x29, 0xCF, 0x18, 0x7D, 0xD6, 0x31, 0x2D, 0x41,
    0x00, 0x00, 0xD2, 0xBF, 0x29, 0xCF, 0xE7, 0x83, 0xD6, 0x31, 0x1F, 0xFF, 0xD2, 0xBF, 0xE0, 0x00,
    0x11, 0x51, 0xC4, 0xE0, 0xEE, 0xAF, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x18, 0x7D, 0x3B, 0x20,
    0x00, 0x00, 0x16, 0xA0, 0x3B, 0x20, 0xF6, 0xA1, 0x2D, 0x41, 0x2D, 0x41, 0x00, 0x00, 0x29, 0xCF,
    0x2D, 0x41, 0xEE, 0xAF, 0x3B, 0x20, 0x18, 0x7D, 0x00, 0x00, 0x36, 0xA0, 0x18, 0x7D, 0xE9, 0x60,
    0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x20, 0x00, 0x00, 0xE7, 0x83, 0x3B, 0x20, 0xE7, 0x83,
    0x00, 0x00, 0x36, 0xA0, 0xE7, 0x83, 0xE9, 0x60, 0x2D, 0x41, 0xD2, 0xBF, 0x00, 0x00, 0x29, 0xCF,
    0xD2, 0xBF, 0xEE, 0xAF, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x18, 0x7D, 0xC4, 0xE0, 0x00, 0x00,
    0x16, 0xA0, 0xC4, 0xE0, 0xF6, 0xA1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub static lbColl_SphereNormals: [u8; 0x1A0] = [
    0x29, 0xC7, 0x2D, 0x49, 0x11, 0x4E, 0x1F, 0xF9, 0x2D, 0x49, 0x1F, 0xF9, 0x29, 0xCD, 0xE7, 0x7B,
    0x29, 0xCD, 0x00, 0x00, 0xC4, 0xDC, 0x18, 0x75, 0x11, 0x4B, 0x3B, 0x24, 0x11, 0x4B, 0x11, 0x4E,
    0xD2, 0xB7, 0x29, 0xC7, 0x00, 0x00, 0xD2, 0xB7, 0x2D, 0x38, 0x16, 0x9F, 0x18, 0x85, 0x36, 0x9D,
    0x00, 0x00, 0x18, 0x85, 0x3B, 0x1D, 0x09, 0x5C, 0xC4, 0xDC, 0x16, 0x99, 0x00, 0x00, 0x3B, 0x24,
    0x18, 0x75, 0x09, 0x5C, 0x3B, 0x24, 0x16, 0x99, 0x11, 0x4E, 0x2D, 0x49, 0x29, 0xC7, 0x00, 0x00,
    0x2D, 0x49, 0x2D, 0x38, 0x29, 0xCD, 0x18, 0x85, 0x29, 0xCD, 0x36, 0x9D, 0x18, 0x85, 0x16, 0x9F,
    0x2D, 0x41, 0x00, 0x00, 0x2D, 0x41, 0x3B, 0x20, 0x00, 0x00, 0x18, 0x7D, 0x36, 0x9D, 0xE7, 0x7B,
    0x16, 0x9F, 0x16, 0x99, 0x3B, 0x24, 0x09, 0x5C, 0x1F, 0xF9, 0xD2, 0xB7, 0x1F, 0xF9, 0x29, 0xC7,
    0xD2, 0xB7, 0x11, 0x4E, 0x16, 0x99, 0xC4, 0xDC, 0x09, 0x5C, 0x11, 0x4B, 0xC4, 0xDC, 0x11, 0x4B,
    0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x18, 0x7D, 0x00, 0x00, 0x3B, 0x20, 0x16, 0x9F, 0xE7, 0x7B,
    0x36, 0x9D, 0x00, 0x00, 0xE7, 0x7B, 0x3B, 0x1D, 0x00, 0x00, 0x3B, 0x24, 0xE7, 0x8B, 0x00, 0x00,
    0x2D, 0x49, 0xD2, 0xC8, 0x00, 0x00, 0x18, 0x85, 0xC4, 0xE3, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00,
    0x00, 0x00, 0xE7, 0x7B, 0xC4, 0xE3, 0x00, 0x00, 0xD2, 0xB7, 0xD2, 0xC8, 0x00, 0x00, 0xC4, 0xDC,
    0xE7, 0x8B, 0x09, 0x5C, 0x3B, 0x24, 0xE9, 0x67, 0x11, 0x4E, 0x2D, 0x49, 0xD6, 0x39, 0x16, 0x9F,
    0x18, 0x85, 0xC9, 0x63, 0x18, 0x7D, 0x00, 0x00, 0xC4, 0xE0, 0x16, 0x9F, 0xE7, 0x7B, 0xC9, 0x63,
    0x11, 0x4E, 0xD2, 0xB7, 0xD6, 0x39, 0x09, 0x5C, 0xC4, 0xDC, 0xE9, 0x67, 0x11, 0x4B, 0x3B, 0x24,
    0xEE, 0xB5, 0x1F, 0xF9, 0x2D, 0x49, 0xE0, 0x07, 0x29, 0xCD, 0x18, 0x85, 0xD6, 0x33, 0x2D, 0x41,
    0x00, 0x00, 0xD2, 0xBF, 0x29, 0xCD, 0xE7, 0x7B, 0xD6, 0x33, 0x1F, 0xF9, 0xD2, 0xB7, 0xE0, 0x07,
    0x11, 0x4B, 0xC4, 0xDC, 0xEE, 0xB5, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x18, 0x75, 0x3B, 0x24,
    0x00, 0x00, 0x16, 0x99, 0x3B, 0x24, 0xF6, 0xA4, 0x2D, 0x38, 0x2D, 0x49, 0x00, 0x00, 0x29, 0xC7,
    0x2D, 0x49, 0xEE, 0xB2, 0x3B, 0x1D, 0x18, 0x85, 0x00, 0x00, 0x36, 0x9D, 0x18, 0x85, 0xE9, 0x61,
    0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x20, 0x00, 0x00, 0xE7, 0x83, 0x3B, 0x1D, 0xE7, 0x7B,
    0x00, 0x00, 0x36, 0x9D, 0xE7, 0x7B, 0xE9, 0x61, 0x2D, 0x38, 0xD2, 0xB7, 0x00, 0x00, 0x29, 0xC7,
    0xD2, 0xB7, 0xEE, 0xB2, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x18, 0x75, 0xC4, 0xDC, 0x00, 0x00,
    0x16, 0x99, 0xC4, 0xDC, 0xF6, 0xA4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub static lbColl_SphereDisplayList: [u8; 0x120] = [
    0x98, 0x00, 0x75, 0x0B, 0x0B, 0x04, 0x04, 0x0C, 0x0C, 0x01, 0x01, 0x07, 0x07, 0x0E, 0x0E, 0x19,
    0x19, 0x10, 0x10, 0x1A, 0x1A, 0x02, 0x02, 0x05, 0x05, 0x14, 0x14, 0x09, 0x09, 0x14, 0x14, 0x17,
    0x17, 0x15, 0x15, 0x17, 0x17, 0x16, 0x16, 0x17, 0x17, 0x3E, 0x3E, 0x09, 0x09, 0x03, 0x03, 0x05,
    0x05, 0x06, 0x06, 0x1A, 0x1A, 0x1B, 0x1B, 0x19, 0x19, 0x18, 0x18, 0x07, 0x07, 0x08, 0x08, 0x0C,
    0x0C, 0x0D, 0x0D, 0x0B, 0x0B, 0x0A, 0x0A, 0x0B, 0x0B, 0x31, 0x31, 0x04, 0x04, 0x31, 0x31, 0x13,
    0x13, 0x32, 0x32, 0x00, 0x00, 0x34, 0x34, 0x0F, 0x0F, 0x36, 0x36, 0x11, 0x11, 0x38, 0x38, 0x12,
    0x12, 0x3A, 0x3A, 0x15, 0x15, 0x3C, 0x3C, 0x16, 0x16, 0x3F, 0x3F, 0x3E, 0x3E, 0x3F, 0x3F, 0x40,
    0x40, 0x3F, 0x3F, 0x3D, 0x3D, 0x3C, 0x3C, 0x3B, 0x3B, 0x3A, 0x3A, 0x39, 0x39, 0x38, 0x38, 0x37,
    0x37, 0x36, 0x36, 0x35, 0x35, 0x34, 0x34, 0x33, 0x33, 0x32, 0x32, 0x33, 0x33, 0x31, 0x31, 0x33,
    0x33, 0x2A, 0x2A, 0x35, 0x35, 0x2B, 0x2B, 0x37, 0x37, 0x2C, 0x2C, 0x39, 0x39, 0x2D, 0x2D, 0x3B,
    0x3B, 0x2E, 0x2E, 0x3D, 0x3D, 0x2F, 0x2F, 0x40, 0x40, 0x30, 0x30, 0x3E, 0x3E, 0x30, 0x30, 0x29,
    0x29, 0x30, 0x30, 0x28, 0x28, 0x2F, 0x2F, 0x27, 0x27, 0x2E, 0x2E, 0x26, 0x26, 0x2D, 0x2D, 0x25,
    0x25, 0x2C, 0x2C, 0x24, 0x24, 0x2B, 0x2B, 0x23, 0x23, 0x2A, 0x2A, 0x23, 0x23, 0x31, 0x31, 0x23,
    0x23, 0x1C, 0x1C, 0x24, 0x24, 0x1D, 0x1D, 0x25, 0x25, 0x1E, 0x1E, 0x26, 0x26, 0x1F, 0x1F, 0x27,
    0x27, 0x20, 0x20, 0x28, 0x28, 0x21, 0x21, 0x29, 0x29, 0x22, 0x22, 0x3E, 0x3E, 0x98, 0x00, 0x0C,
    0x15, 0x15, 0x14, 0x14, 0x12, 0x12, 0x02, 0x02, 0x11, 0x11, 0x10, 0x10, 0x0F, 0x0F, 0x0E, 0x0E,
    0x00, 0x00, 0x01, 0x01, 0x13, 0x13, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub static lbColl_CylinderPositions: [u8; 0x80] = [
    0xE0, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0xE0, 0x01, 0xE7, 0x83, 0x3B, 0x20, 0xE0, 0x00, 0x00, 0x00,
    0xC0, 0x01, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x01, 0x20, 0x00, 0x00, 0x00, 0xC0, 0x01, 0xE0, 0x01,
    0xC4, 0xE0, 0xE7, 0x83, 0xE0, 0x01, 0xE7, 0x83, 0xC4, 0xE0, 0xE0, 0x01, 0xD2, 0xBF, 0xD2, 0xBF,
    0xE0, 0x00, 0x18, 0x7D, 0xC4, 0xE0, 0x00, 0x00, 0x18, 0x7D, 0xC4, 0xE0, 0x1F, 0xFF, 0x18, 0x7D,
    0xC4, 0xE0, 0x1F, 0xFF, 0x18, 0x7D, 0x3B, 0x20, 0xE0, 0x00, 0x18, 0x7D, 0x3B, 0x20, 0xE0, 0x00,
    0x2D, 0x41, 0x2D, 0x41, 0x00, 0x00, 0x18, 0x7D, 0x3B, 0x20, 0xE0, 0x01, 0xC0, 0x01, 0x00, 0x00,
    0x00, 0x00, 0xE7, 0x83, 0xC4, 0xE0, 0x00, 0x00, 0xD2, 0xBF, 0xD2, 0xBF, 0x20, 0x00, 0xD2, 0xBF,
    0xD2, 0xBF, 0x20, 0x00, 0xE7, 0x83, 0xC4, 0xE0, 0xE0, 0x00, 0x2D, 0x41, 0xD2, 0xBF, 0x00, 0x00,
];

pub static lbColl_UnknownData: [u8; 0xA0] = [
    0x2D, 0x41, 0xD2, 0xBF, 0x1F, 0xFF, 0x2D, 0x41, 0xD2, 0xBF, 0x00, 0x00, 0xC4, 0xE0, 0xE7, 0x83,
    0x00, 0x00, 0xC0, 0x01, 0x00, 0x00, 0x20, 0x00, 0xC0, 0x01, 0x00, 0x00, 0x20, 0x00, 0xC4, 0xE0,
    0xE7, 0x83, 0x00, 0x00, 0x2D, 0x41, 0x2D, 0x41, 0x1F, 0xFF, 0x3B, 0x20, 0x18, 0x7D, 0x1F, 0xFF,
    0x2D, 0x41, 0x2D, 0x41, 0x00, 0x00, 0xE7, 0x83, 0x3B, 0x20, 0x00, 0x00, 0x00, 0x00, 0x3F, 0xFF,
    0x20, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0x20, 0x00, 0xE7, 0x83, 0x3B, 0x20, 0x20, 0x00, 0xD2, 0xBF,
    0x2D, 0x41, 0x20, 0x00, 0xC4, 0xE0, 0x18, 0x7D, 0xE0, 0x00, 0x3B, 0x20, 0xE7, 0x83, 0x00, 0x00,
    0x3B, 0x20, 0xE7, 0x83, 0x1F, 0xFF, 0x3B, 0x20, 0xE7, 0x83, 0x1F, 0xFF, 0x40, 0x00, 0x00, 0x00,
    0xE0, 0x01, 0xC4, 0xE0, 0x18, 0x7D, 0xE0, 0x01, 0xD2, 0xBF, 0x2D, 0x41, 0x00, 0x00, 0xD2, 0xBF,
    0x2D, 0x41, 0x00, 0x00, 0xC4, 0xE0, 0x18, 0x7D, 0xE0, 0x00, 0x3B, 0x20, 0x18, 0x7D, 0xE0, 0x00,
    0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x20, 0x18, 0x7D,
];

pub static lbColl_CylinderNormals: [u8; 0x60] = [
    0x00, 0x00, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x00, 0x00, 0x18, 0xFF, 0xC5, 0x00, 0x00, 0x00, 0x18,
    0x00, 0x3B, 0x00, 0x00, 0xFF, 0xD3, 0xFF, 0xD3, 0x00, 0x00, 0xFF, 0xE8, 0xFF, 0xC5, 0x00, 0x00,
    0x00, 0x2D, 0xFF, 0xD3, 0x00, 0x00, 0xFF, 0xC0, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xC5, 0xFF, 0xE8,
    0x00, 0x00, 0x00, 0x2D, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0xFF, 0xE8,
    0x00, 0x3B, 0x00, 0x00, 0x00, 0x3B, 0xFF, 0xE8, 0x00, 0x00, 0xFF, 0xD3, 0x00, 0x2D, 0x00, 0x00,
    0xFF, 0xC5, 0x00, 0x18, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3B, 0x00, 0x18,
];

pub static lbColl_CylinderDisplayList: [u8; 0xA0] = [
    0x98, 0x00, 0x45, 0x2C, 0x0F, 0x2D, 0x0E, 0x2F, 0x0F, 0x2D, 0x0E, 0x2E, 0x0E, 0x24, 0x0B, 0x25,
    0x0B, 0x14, 0x05, 0x15, 0x05, 0x08, 0x01, 0x09, 0x01, 0x02, 0x00, 0x03, 0x00, 0x06, 0x04, 0x10,
    0x04, 0x07, 0x03, 0x11, 0x03, 0x05, 0x07, 0x17, 0x07, 0x0F, 0x06, 0x18, 0x06, 0x28, 0x0D, 0x2B,
    0x0D, 0x29, 0x0C, 0x2A, 0x0C, 0x01, 0x0A, 0x1E, 0x0A, 0x00, 0x09, 0x1F, 0x09, 0x0C, 0x02, 0x0E,
    0x02, 0x0D, 0x08, 0x1B, 0x08, 0x2C, 0x0F, 0x1B, 0x08, 0x2F, 0x0F, 0x1D, 0x08, 0x2F, 0x0F, 0x1C,
    0x0F, 0x2E, 0x0E, 0x27, 0x0E, 0x25, 0x0B, 0x26, 0x0B, 0x15, 0x05, 0x16, 0x05, 0x09, 0x01, 0x0A,
    0x01, 0x03, 0x00, 0x04, 0x00, 0x10, 0x04, 0x13, 0x04, 0x11, 0x03, 0x12, 0x03, 0x17, 0x07, 0x1A,
    0x07, 0x18, 0x06, 0x19, 0x06, 0x2B, 0x0D, 0x23, 0x0D, 0x2A, 0x0C, 0x22, 0x0C, 0x1E, 0x0A, 0x21,
    0x0A, 0x1F, 0x09, 0x20, 0x09, 0x0E, 0x02, 0x0B, 0x02, 0x1B, 0x08, 0x1D, 0x08, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub static lbColl_803B9880: [i32; 42] = [
    0x00083D60, 0x00083D60, 0x00083D60, 0x0000005B, 0x0000005A, 0x00000059, 0x00000058, 0x00000057, 0x00000056, 0x0000006F, 0x00000070, 0x00000071, 0x00000054, 0x00000054, 0x00000054, 0x0000005A, 0x00000059, 0x000000DF, 0x000000E1, 0x000000E1, 0x000000E1, 0x00000062, 0x00000063, 0x00000064, 0x00000065, 0x00000066, 0x00000067, 0x0004461B, 0x0004461B, 0x0004461B, 0x000000F1, 0x000000F1, 0x000000F1, 0x0000005E, 0x0000005D, 0x0000005C, 0x00035BAF, 0x00035BB2, 0x00035BB5, 0x00083D60, 0x00083D60, 0x0000020D,
];

static mut lbColl_804D36A0: GXColor = GXColor { r: 0xFF, g: 0x00, b: 0x00, a: 0x80 };
static mut lbColl_804D36A4: GXColor = GXColor { r: 0xFF, g: 0x00, b: 0xFF, a: 0x80 };
static mut lbColl_804D36A8: GXColor = GXColor { r: 0x80, g: 0x00, b: 0x00, a: 0x80 };
static mut lbColl_804D36AC: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0x00, a: 0x80 };
static mut lbColl_804D36B0: GXColor = GXColor { r: 0x80, g: 0x80, b: 0x00, a: 0x80 };
static mut lbColl_804D36B4: GXColor = GXColor { r: 0x00, g: 0xFF, b: 0x00, a: 0x80 };
static mut lbColl_804D36B8: GXColor = GXColor { r: 0x00, g: 0x80, b: 0x00, a: 0x80 };
static mut lbColl_804D36BC: GXColor = GXColor { r: 0x00, g: 0x00, b: 0xFF, a: 0x80 };
static mut lbColl_804D36C0: GXColor = GXColor { r: 0x00, g: 0x00, b: 0x80, a: 0x80 };
static mut lbColl_804D36C4: GXColor = GXColor { r: 0x00, g: 0xFF, b: 0xFF, a: 0x80 };
static mut lbColl_804D36C8: GXColor = GXColor { r: 0x00, g: 0x80, b: 0x80, a: 0x80 };
static mut lbColl_804D36CC: GXColor = GXColor { r: 0x00, g: 0xFF, b: 0x80, a: 0x80 };
static mut lbColl_804D36D0: GXColor = GXColor { r: 0x00, g: 0x80, b: 0x40, a: 0x80 };
static mut lbColl_804D36D4: GXColor = GXColor { r: 0x00, g: 0x80, b: 0xFF, a: 0x80 };
static mut lbColl_804D36D8: GXColor = GXColor { r: 0x00, g: 0x40, b: 0x80, a: 0x80 };
static mut lbColl_804D36DC: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0x80 };
static mut lbColl_804D36E0: GXColor = GXColor { r: 0xFF, g: 0x80, b: 0x00, a: 0x80 };
static mut lbColl_804D36E4: GXColor = GXColor { r: 0x80, g: 0x40, b: 0x00, a: 0x80 };
static mut lbColl_804D36E8: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0x80 };
static mut lbColl_804D36EC: GXColor = GXColor { r: 0x80, g: 0x80, b: 0x80, a: 0x80 };
static mut lbColl_804D36F0: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0x00, a: 0x80 };
static mut lbColl_804D36F4: GXColor = GXColor { r: 0x80, g: 0x80, b: 0x00, a: 0x80 };
static mut lbColl_804D36F8: GXColor = GXColor { r: 0xFF, g: 0x00, b: 0x00, a: 0x80 };
static mut lbColl_804D36FC: GXColor = GXColor { r: 0x80, g: 0x00, b: 0x00, a: 0x80 };
pub static mut lbColl_803B9928: [LbCollColorPair; 6] = unsafe {
    [
        LbCollColorPair { pad: &raw mut lbColl_804D36AC, pad_x: &raw mut lbColl_804D36B0 },
        LbCollColorPair { pad: &raw mut lbColl_804D36B4, pad_x: &raw mut lbColl_804D36B8 },
        LbCollColorPair { pad: &raw mut lbColl_804D36BC, pad_x: &raw mut lbColl_804D36C0 },
        core::mem::zeroed(),
        core::mem::zeroed(),
        core::mem::zeroed(),
    ]
};

// ---- .text ----------------------------------------------------------------------------------

pub unsafe extern "C" fn lbColl_80005BB0(arg0: *mut HitCapsule, arg1: i32) -> i32 {
    melee_core::ct!("lbColl_80005BB0");
    let kind = (*arg0).sfx_kind as u32;
    if kind == 0xD {
        let sev = (*arg0).sfx_severity as u32;
        if sev == 2 {
            return ext::lbAudioAx_80024184(lbColl_803B9880[(kind * 3 + sev) as usize], 127, 64, arg1);
        }
    }
    ext::lbAudioAx_80024184(lbColl_803B9880[(kind * 3 + (*arg0).sfx_severity as u32) as usize], 127, 64, -1)
}

pub unsafe extern "C" fn lbColl_80005C44(arg0: *const Vec3, arg1: *const Vec3, arg2: *const Vec3, arg3: *mut Vec3, arg8: f32, arg9: f32) -> bool {
    melee_core::ct!("lbColl_80005C44");
    let temp_f2 = arg8 + arg9;
    let sp38 = *arg0;
    let sp2c = *arg2;

    if sp38.x > (*arg1).x {
        if sp38.x + temp_f2 < sp2c.x {
            return false;
        }
        if (*arg1).x - temp_f2 > sp2c.x {
            return false;
        }
    } else {
        if sp38.x - temp_f2 > sp2c.x {
            return false;
        }
        if (*arg1).x + temp_f2 < sp2c.x {
            return false;
        }
    }
    if sp38.y > (*arg1).y {
        if sp38.y + temp_f2 < sp2c.y {
            return false;
        }
        if (*arg1).y - temp_f2 > sp2c.y {
            return false;
        }
    } else {
        if sp38.y - temp_f2 > sp2c.y {
            return false;
        }
        if (*arg1).y + temp_f2 < sp2c.y {
            return false;
        }
    }
    if sp38.z > (*arg1).z {
        if sp38.z + temp_f2 < sp2c.z {
            return false;
        }
        if (*arg1).z - temp_f2 > sp2c.z {
            return false;
        }
    } else {
        if sp38.z - temp_f2 > sp2c.z {
            return false;
        }
        if (*arg1).z + temp_f2 < sp2c.z {
            return false;
        }
    }

    let d1 = Vec3 { x: (*arg1).x - sp38.x, y: (*arg1).y - sp38.y, z: (*arg1).z - sp38.z };
    let mut d2_x = sp38.x - sp2c.x;
    let mut d2_y = sp38.y - sp2c.y;
    let mut d2_z = sp38.z - sp2c.z;

    let d1_dot_d1 = dot3(d1.x, d1.y, d1.z, d1.x, d1.y, d1.z);
    let d1_dot_d2 = dot3(d1.x, d1.y, d1.z, d2_x, d2_y, d2_z);
    let scale = if approximatelyZero(d1_dot_d1) { 0.0f32 } else { clamp01(-d1_dot_d2 / d1_dot_d1) };
    (*arg3).x = fmadds(d1.x, scale, sp38.x);
    (*arg3).y = fmadds(d1.y, scale, sp38.y);
    (*arg3).z = fmadds(d1.z, scale, sp38.z);

    d2_x = (*arg3).x - sp2c.x;
    d2_y = (*arg3).y - sp2c.y;
    d2_z = (*arg3).z - sp2c.z;

    !(temp_f2 * temp_f2 < sq3(d2_x, d2_y, d2_z))
}

pub unsafe extern "C" fn lbColl_80005EBC(arg0: *const Vec3, arg1: *const Vec3, arg2: *const Vec3, arg3: *mut f32) -> f32 {
    melee_core::ct!("lbColl_80005EBC");
    let sp50 = *arg0;
    let d1 = Vec3 { x: (*arg1).x - (*arg0).x, y: (*arg1).y - (*arg0).y, z: (*arg1).z - (*arg0).z };
    let sp38 = *arg2;
    let d2 = Vec3 { x: sp50.x - sp38.x, y: sp50.y - sp38.y, z: sp50.z - sp38.z };
    let d1_dot_d1 = dot3(d1.x, d1.y, d1.z, d1.x, d1.y, d1.z);
    let d1_dot_d2 = dot3(d1.x, d1.y, d1.z, d2.x, d2.y, d2.z);
    let scale = clamp01(-d1_dot_d2 / d1_dot_d1);
    let x = fmadds(d1.x, scale, sp50.x) - (*arg2).x;
    let y = fmadds(d1.y, scale, sp50.y) - (*arg2).y;
    let z = fmadds(d1.z, scale, sp50.z) - (*arg2).z;
    *arg3 = scale;
    sq3(x, y, z)
}

pub unsafe extern "C" fn lbColl_80005FC0(arg0: *const Vec3, arg1: *const Vec3, arg2: *const Vec3, arg3: *mut f32) -> f32 {
    melee_core::ct!("lbColl_80005FC0");
    let sp50 = *arg0;
    let d1x = (*arg1).x - (*arg0).x;
    let d1y = (*arg1).y - (*arg0).y;
    let sp38 = *arg2;
    let d2x = sp50.x - sp38.x;
    let d2y = sp50.y - sp38.y;
    let d1_dot_d1 = fmadds(d1x, d1x, d1y * d1y);
    let d1_dot_d2 = fmadds(d1x, d2x, d1y * d2y);
    let scale = clamp01(-d1_dot_d2 / d1_dot_d1);
    let x = fmadds(d1x, scale, sp50.x) - (*arg2).x;
    let y = fmadds(d1y, scale, sp50.y) - (*arg2).y;
    *arg3 = scale;
    fmadds(x, x, y * y)
}

/// `end`: is `a` within `unk_sum` of `b`?
unsafe fn end(a: *const Vec3, b: *const Vec3, unk_sum: f32) -> bool {
    let x = (*a).x - (*b).x;
    let y = (*a).y - (*b).y;
    let z = (*a).z - (*b).z;
    !(unk_sum * unk_sum < sq3(x, y, z))
}

pub unsafe extern "C" fn lbColl_80006094(arg0: *const Vec3, arg1: *const Vec3, arg2: *const Vec3, arg3: *const Vec3, arg4: *mut Vec3, arg5: *mut Vec3, arg6: f32, arg7: f32) -> bool {
    melee_core::ct!("lbColl_80006094");
    let unk_sum = arg6 + arg7;
    let vec4 = *arg0;
    let arg4_offset = vec4;
    let arg5_offset = *arg2;

    let arg1_x = (*arg1).x;
    if reject_axis(arg4_offset.x, arg1_x, arg5_offset.x, (*arg3).x, unk_sum) {
        return false;
    }
    if reject_axis(arg4_offset.y, (*arg1).y, arg5_offset.y, (*arg3).y, unk_sum) {
        return false;
    }
    if reject_axis(arg4_offset.z, (*arg1).z, arg5_offset.z, (*arg3).z, unk_sum) {
        return false;
    }

    let d1_y = (*arg1).y - arg4_offset.y;
    let d1_z = (*arg1).z - arg4_offset.z;
    let d2_y = (*arg3).y - arg5_offset.y;
    let offset_delta_y = arg4_offset.y - arg5_offset.y;
    let d1_x = arg1_x - arg4_offset.x;
    let d2_x = (*arg3).x - arg5_offset.x;
    let arg3_z = (*arg3).z;
    let d2_z = arg3_z - arg5_offset.z;
    let offset_delta_x = arg4_offset.x - arg5_offset.x;
    let d1_len_sq = sq3(d1_x, d1_y, d1_z);
    let d2_len_sq = sq3(d2_x, d2_y, d2_z);
    let d1_dot_d2 = dot3(d1_x, d1_y, d1_z, d2_x, d2_y, d2_z);
    let offset_delta_z = arg4_offset.z - arg5_offset.z;
    let d2_dot_offset_delta = dot3(d2_x, d2_y, d2_z, offset_delta_x, offset_delta_y, offset_delta_z);
    let d1_dot_offset_delta = dot3(d1_x, d1_y, d1_z, offset_delta_x, offset_delta_y, offset_delta_z);
    let denom = fmsubs(d1_len_sq, d2_len_sq, d1_dot_d2 * d1_dot_d2);

    let mut arg4_scl: f32;
    let mut arg5_scl: f32;
    if approximatelyZero(d2_len_sq) {
        if approximatelyZero(d1_len_sq) {
            arg4_scl = 0.0;
            arg5_scl = 0.0;
        } else {
            arg5_scl = 0.0;
            arg4_scl = clamp01(-d1_dot_offset_delta / d1_len_sq);
        }
    } else if approximatelyZero(denom) {
        let mid_y = half_fma(d2_y, arg5_offset.y);
        let mid_x = half_fma(d2_x, arg5_offset.x);
        let arg4_mid_y = arg4_offset.y - mid_y;
        let arg1_mid_y = (*arg1).y - mid_y;
        let mid_z = half_fma(d2_z, arg5_offset.z);
        let arg4_mid_x = arg4_offset.x - mid_x;
        let arg1_mid_x = arg1_x - mid_x;
        let arg4_offset_z = arg4_offset.z - mid_z;
        let arg1_mid_z = (*arg1).z - mid_z;

        if sq3(arg4_mid_x, arg4_mid_y, arg4_offset_z) < sq3(arg1_mid_x, arg1_mid_y, arg1_mid_z) {
            let c3 = arg5_offset;
            arg4_scl = 0.0;
            let d1 = Vec3 { x: (*arg3).x - (*arg2).x, y: (*arg3).y - (*arg2).y, z: arg3_z - (*arg2).z };
            let a2 = vec4;
            let dot0 = dot3(d1.x, d1.y, d1.z, c3.x - a2.x, c3.y - a2.y, c3.z - a2.z);
            arg5_scl = clamp01(-dot0 / sq3(d1.x, d1.y, d1.z));
        } else {
            let c2 = arg5_offset;
            arg4_scl = 1.0;
            let d2 = Vec3 { x: (*arg3).x - (*arg2).x, y: (*arg3).y - (*arg2).y, z: arg3_z - (*arg2).z };
            let b0 = *arg1;
            let dot1 = dot3(d2.x, d2.y, d2.z, c2.x - b0.x, c2.y - b0.y, c2.z - b0.z);
            arg5_scl = clamp01(-dot1 / sq3(d2.x, d2.y, d2.z));
        }
    } else {
        arg4_scl = fmsubs(d1_dot_d2, d2_dot_offset_delta, d2_len_sq * d1_dot_offset_delta) / denom;
        arg5_scl = fmsubs(d1_len_sq, d2_dot_offset_delta, d1_dot_d2 * d1_dot_offset_delta) / denom;
        if arg4_scl as f64 > 1.0 || (arg4_scl as f64) < 0.0 || arg5_scl as f64 > 1.0 || (arg5_scl as f64) < 0.0 {
            let mut candidate0_arg5_scl = 0.0f32;
            let mut candidate1_arg4_scl = 0.0f32;
            let candidate0_scale;
            let candidate0_dist_sq;
            let candidate1_dist_sq;
            if (arg4_scl as f64) < 0.0 {
                candidate0_scale = 0.0f32;
                candidate0_dist_sq = lbColl_80005EBC(arg2, arg3, arg0, &mut candidate0_arg5_scl);
            } else {
                candidate0_scale = 1.0f32;
                candidate0_dist_sq = lbColl_80005EBC(arg2, arg3, arg1, &mut candidate0_arg5_scl);
            }
            if (arg5_scl as f64) < 0.0 {
                arg5_scl = 0.0;
                candidate1_dist_sq = lbColl_80005EBC(arg0, arg1, arg2, &mut candidate1_arg4_scl);
            } else {
                arg5_scl = 1.0;
                candidate1_dist_sq = lbColl_80005EBC(arg0, arg1, arg3, &mut candidate1_arg4_scl);
            }
            if candidate0_dist_sq < candidate1_dist_sq {
                arg4_scl = candidate0_scale;
                arg5_scl = candidate0_arg5_scl;
            } else {
                arg4_scl = candidate1_arg4_scl;
            }
        }
    }

    (*arg4).x = fmadds(d1_x, arg4_scl, arg4_offset.x);
    (*arg4).y = fmadds(d1_y, arg4_scl, arg4_offset.y);
    (*arg4).z = fmadds(d1_z, arg4_scl, arg4_offset.z);
    (*arg5).x = fmadds(d2_x, arg5_scl, arg5_offset.x);
    (*arg5).y = fmadds(d2_y, arg5_scl, arg5_offset.y);
    (*arg5).z = fmadds(d2_z, arg5_scl, arg5_offset.z);
    end(arg4, arg5, unk_sum)
}

pub unsafe extern "C" fn lbColl_800067F8(a: *const Vec3, b: *const Vec3, c: *const Vec3, d: *const Vec3, e: *mut Vec3, f: *mut Vec3, p: f32, q: f32) -> bool {
    melee_core::ct!("lbColl_800067F8");
    let sum_pq = p + q;
    let a0 = *a;
    let a1 = a0;
    let c0 = *c;
    let c1 = c0;
    let b_x = (*b).x;
    if reject_axis(a1.x, b_x, c1.x, (*d).x, sum_pq) {
        return false;
    }
    let a1_y = a1.y;
    let b_y = (*b).y;
    if reject_axis(a1_y, b_y, c1.y, (*d).y, sum_pq) {
        return false;
    }

    let c1_y = c1.y;
    let d_y = (*d).y;
    let diff_ac_y = a1.y - c1.y;
    let d_x = (*d).x;

    let diff_ba_x = b_x - a1.x;
    let diff_ba_y = b_y - a1_y;
    let diff_dc_x = d_x - c1.x;
    let diff_dc_y = d_y - c1_y;

    let sqdist2_ba = fmadds(diff_ba_x, diff_ba_x, diff_ba_y * diff_ba_y);
    let sqdist2_dc = fmadds(diff_dc_x, diff_dc_x, diff_dc_y * diff_dc_y);
    let dot2_diff_ba_dc = fmadds(diff_ba_x, diff_dc_x, diff_ba_y * diff_dc_y);
    let diff_ac_x = a1.x - c1.x;
    let dot2_diff_dc_ac = fmadds(diff_dc_x, diff_ac_x, diff_dc_y * diff_ac_y);
    let y_product = diff_ba_y * diff_ac_y;
    let dot2_diff_ba_ac = fmadds(diff_ba_x, diff_ac_x, y_product);
    let dot_squared = dot2_diff_ba_dc * dot2_diff_ba_dc;
    let determinant = fmsubs(sqdist2_ba, sqdist2_dc, dot_squared);

    let mut scl_e: f32;
    let mut scl_f: f32;
    if approximatelyZero(sqdist2_dc) {
        if approximatelyZero(sqdist2_ba) {
            scl_e = 0.0;
            scl_f = 0.0;
        } else {
            scl_f = 0.0;
            scl_e = clamp01(-dot2_diff_ba_ac / sqdist2_ba);
        }
    } else if approximatelyZero(determinant) {
        let mid_y = half_fma(diff_dc_y, c1_y);
        let mid_x = half_fma(diff_dc_x, c1.x);
        let a_mid_y = a1_y - mid_y;
        let b_mid_y = b_y - mid_y;
        let a_mid_x = a1.x - mid_x;
        let b_mid_x = b_x - mid_x;
        if fmadds(a_mid_x, a_mid_x, a_mid_y * a_mid_y) < fmadds(b_mid_x, b_mid_x, b_mid_y * b_mid_y) {
            let c3 = c0;
            scl_e = 0.0;
            let d1 = Vec3 { x: d_x - (*c).x, y: d_y - (*c).y, z: (*d).z - (*c).z };
            let a2 = a0;
            let dot0 = dot3(d1.x, d1.y, d1.z, c3.x - a2.x, c3.y - a2.y, c3.z - a2.z);
            scl_f = clamp01(-dot0 / sq3(d1.x, d1.y, d1.z));
        } else {
            let c2 = c0;
            scl_e = 1.0;
            let d2 = Vec3 { x: d_x - (*c).x, y: d_y - (*c).y, z: (*d).z - (*c).z };
            let b0 = *b;
            let dot1 = dot3(d2.x, d2.y, d2.z, c2.x - b0.x, c2.y - b0.y, c2.z - b0.z);
            scl_f = clamp01(-dot1 / sq3(d2.x, d2.y, d2.z));
        }
    } else {
        scl_e = fmsubs(dot2_diff_ba_dc, dot2_diff_dc_ac, sqdist2_dc * dot2_diff_ba_ac) / determinant;
        scl_f = fmsubs(sqdist2_ba, dot2_diff_dc_ac, dot2_diff_ba_dc * dot2_diff_ba_ac) / determinant;
        if scl_e as f64 > 1.0 || (scl_e as f64) < 0.0 || scl_f as f64 > 1.0 || (scl_f as f64) < 0.0 {
            let mut out0 = 0.0f32;
            let mut out1 = 0.0f32;
            let temp_scl_e;
            let result0;
            if (scl_e as f64) < 0.0 {
                temp_scl_e = 0.0f32;
                result0 = lbColl_80005FC0(c, d, a, &mut out0);
            } else {
                temp_scl_e = 1.0f32;
                result0 = lbColl_80005FC0(c, d, b, &mut out0);
            }
            let result1;
            if (scl_f as f64) < 0.0 {
                scl_f = 0.0;
                result1 = lbColl_80005FC0(a, b, c, &mut out1);
            } else {
                scl_f = 1.0;
                result1 = lbColl_80005FC0(a, b, d, &mut out1);
            }
            if result0 < result1 {
                scl_e = temp_scl_e;
                scl_f = out0;
            } else {
                scl_e = out1;
            }
        }
    }

    (*e).x = fmadds(diff_ba_x, scl_e, a1.x);
    (*e).y = fmadds(diff_ba_y, scl_e, a1_y);
    (*e).z = 0.0;
    (*f).x = fmadds(diff_dc_x, scl_f, c1.x);
    (*f).y = fmadds(diff_dc_y, scl_f, c1_y);
    (*f).z = 0.0;

    let x = (*e).x - (*f).x;
    let y = (*e).y - (*f).y;
    let distance_sq = fmadds(x, x, y * y);
    !(sum_pq * sum_pq < distance_sq)
}

pub unsafe extern "C" fn lbColl_80006E58(
    hit_start: *const Vec3,
    hit_end: *const Vec3,
    hurt_start: *const Vec3,
    hurt_end: *const Vec3,
    hit_closest: *mut Vec3,
    hurt_closest: *mut Vec3,
    hurt_mtx: *const Mtx,
    out_contact_pos: *mut Vec3,
    out_overlap: *mut f32,
    hit_radius: f32,
    hurt_radius: f32,
    broadphase_scale: f32,
) -> bool {
    melee_core::ct!("lbColl_80006E58");
    let broadphase_radius = fmadds(hurt_radius, broadphase_scale, hit_radius);
    let hs = *hit_start;
    let us = *hurt_start;
    let hit_end_x = (*hit_end).x;
    if reject_axis(hs.x, hit_end_x, us.x, (*hurt_end).x, broadphase_radius) {
        return false;
    }
    if reject_axis(hs.y, (*hit_end).y, us.y, (*hurt_end).y, broadphase_radius) {
        return false;
    }
    if reject_axis(hs.z, (*hit_end).z, us.z, (*hurt_end).z, broadphase_radius) {
        return false;
    }

    let hit_delta = Vec3 { x: hit_end_x - hs.x, y: (*hit_end).y - hs.y, z: (*hit_end).z - hs.z };
    let hurt_end_x = (*hurt_end).x;
    let hurt_end_y = (*hurt_end).y;
    let hurt_end_z = (*hurt_end).z;
    let hurt_start_y = us.y;
    let hurt_start_z = us.z;
    let mut axis = Vec3 { x: hurt_end_x - us.x, y: hurt_end_y - hurt_start_y, z: hurt_end_z - hurt_start_z };
    let sep_y = hit_delta.y * hit_delta.y;
    let sep_x = hit_delta.x * hit_delta.x;
    let sep_z = hit_delta.z * hit_delta.z;
    let hit_len_sq = sep_z + (sep_x + sep_y);
    let offset = Vec3 { x: hs.x - us.x, y: hs.y - hurt_start_y, z: hs.z - hurt_start_z };
    let hurt_start_dot = dot3(axis.x, axis.y, axis.z, offset.x, offset.y, offset.z);
    let segment_dot = dot3(hit_delta.x, hit_delta.y, hit_delta.z, axis.x, axis.y, axis.z);
    let hit_start_dot = dot3(hit_delta.x, hit_delta.y, hit_delta.z, offset.x, offset.y, offset.z);
    let mut hurt_len_sq = axis.y * axis.y;
    hurt_len_sq = fmadds(axis.x, axis.x, hurt_len_sq);
    hurt_len_sq = fmadds(axis.z, axis.z, hurt_len_sq);
    let closest_denom = fmsubs(hit_len_sq, hurt_len_sq, segment_dot * segment_dot);

    let mut hit_param: f32;
    let mut hurt_param: f32;
    if approximatelyZero(hurt_len_sq) {
        if approximatelyZero(hit_len_sq) {
            hit_param = 0.0;
            hurt_param = hit_param;
        } else {
            hurt_param = 0.0;
            hit_param = -hit_start_dot / hit_len_sq;
            if hit_param as f64 > 1.0 {
                hit_param = 1.0;
            } else if (hit_param as f64) < 0.0 {
                hit_param = hurt_param;
            }
        }
    } else if approximatelyZero(closest_denom) {
        let mut mid = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        mid.x = half_fma(axis.x, us.x);
        mid.y = half_fma(axis.y, hurt_start_y);
        mid.z = half_fma(axis.z, hurt_start_z);
        let sep_x = hs.x - mid.x;
        mid.x = (*hit_end).x - mid.x;
        let sep_y = hs.y - mid.y;
        mid.y = (*hit_end).y - mid.y;
        let sep_z = hs.z - mid.z;
        mid.z = (*hit_end).z - mid.z;
        if sq3(sep_x, sep_y, sep_z) < sq3(mid.x, mid.y, mid.z) {
            let c3 = *hurt_start;
            hit_param = 0.0;
            let d1 = Vec3 { x: hurt_end_x - (*hurt_start).x, y: hurt_end_y - (*hurt_start).y, z: hurt_end_z - (*hurt_start).z };
            let a2 = *hit_start;
            let dot = dot3(d1.x, d1.y, d1.z, c3.x - a2.x, c3.y - a2.y, c3.z - a2.z);
            let mut p = -dot / sq3(d1.x, d1.y, d1.z);
            if p as f64 > 1.0 {
                p = 1.0;
            } else if (p as f64) < 0.0 {
                p = hit_param;
            }
            hurt_param = p;
        } else {
            let c2 = *hurt_start;
            hit_param = 1.0;
            let d1 = Vec3 { x: hurt_end_x - (*hurt_start).x, y: hurt_end_y - (*hurt_start).y, z: hurt_end_z - (*hurt_start).z };
            let b0 = *hit_end;
            let dot = dot3(d1.x, d1.y, d1.z, c2.x - b0.x, c2.y - b0.y, c2.z - b0.z);
            let mut p = -dot / sq3(d1.x, d1.y, d1.z);
            if p as f64 > 1.0 {
                p = hit_param;
            } else if (p as f64) < 0.0 {
                p = 0.0;
            }
            hurt_param = p;
        }
    } else {
        hit_param = fmsubs(segment_dot, hurt_start_dot, hurt_len_sq * hit_start_dot) / closest_denom;
        hurt_param = fmsubs(hit_len_sq, hurt_start_dot, segment_dot * hit_start_dot) / closest_denom;
        if hit_param as f64 > 1.0 || (hit_param as f64) < 0.0 || hurt_param as f64 > 1.0 || (hurt_param as f64) < 0.0 {
            let mut candidate_hurt_param = 0.0f32;
            let mut candidate_hit_param = 0.0f32;
            let hit_endpoint_param;
            let hit_endpoint_dist_sq;
            let hurt_endpoint_param;
            let hurt_endpoint_dist_sq;
            if (hit_param as f64) < 0.0 {
                hit_endpoint_param = 0.0f32;
                hit_endpoint_dist_sq = lbColl_80005EBC(hurt_start, hurt_end, hit_start, &mut candidate_hurt_param);
            } else {
                hit_endpoint_param = 1.0f32;
                hit_endpoint_dist_sq = lbColl_80005EBC(hurt_start, hurt_end, hit_end, &mut candidate_hurt_param);
            }
            if (hurt_param as f64) < 0.0 {
                hurt_endpoint_param = 0.0f32;
                hurt_endpoint_dist_sq = lbColl_80005EBC(hit_start, hit_end, hurt_start, &mut candidate_hit_param);
            } else {
                hurt_endpoint_param = 1.0f32;
                hurt_endpoint_dist_sq = lbColl_80005EBC(hit_start, hit_end, hurt_end, &mut candidate_hit_param);
            }
            if hit_endpoint_dist_sq < hurt_endpoint_dist_sq {
                hit_param = hit_endpoint_param;
                hurt_param = candidate_hurt_param;
            } else {
                hit_param = candidate_hit_param;
                hurt_param = hurt_endpoint_param;
            }
        }
    }
    (*hit_closest).x = fmadds(hit_delta.x, hit_param, hs.x);
    (*hit_closest).y = fmadds(hit_delta.y, hit_param, hs.y);
    (*hit_closest).z = fmadds(hit_delta.z, hit_param, hs.z);
    (*hurt_closest).x = fmadds(axis.x, hurt_param, us.x);
    (*hurt_closest).y = fmadds(axis.y, hurt_param, hurt_start_y);
    (*hurt_closest).z = fmadds(axis.z, hurt_param, hurt_start_z);
    let sx = (*hit_closest).x - (*hurt_closest).x;
    let sy = (*hit_closest).y - (*hurt_closest).y;
    let sz = (*hit_closest).z - (*hurt_closest).z;
    let closest_dist_sq = sq3(sx, sy, sz);
    axis.x = sqrtf(closest_dist_sq);
    if approximatelyZero(axis.x) {
        *out_overlap = (hit_radius + hurt_radius) - axis.x;
        *out_contact_pos = *hit_closest;
        return true;
    }

    // Measure the hurt radius in the matrix's local space for scaled bones.
    let mut inv_hurt_mtx: Mtx = [[0.0; 4]; 3];
    ext::HSD_MtxInverse(hurt_mtx as *mut c_void, (&mut inv_hurt_mtx as *mut Mtx).cast());
    // the C reuses `hit_start_copy` / `hit_delta` as the outputs (their old values survive a no-op callee)
    let mut lhs = Vec { x: hs.x, y: hs.y, z: hs.z };
    let mut rhs = Vec { x: hit_delta.x, y: hit_delta.y, z: hit_delta.z };
    ext::PSMTXMultVec((&mut inv_hurt_mtx as *mut Mtx).cast(), hit_closest as *mut Vec, &mut lhs);
    ext::PSMTXMultVec((&mut inv_hurt_mtx as *mut Mtx).cast(), hurt_closest as *mut Vec, &mut rhs);
    let lsx = lhs.x - rhs.x;
    let lsy = lhs.y - rhs.y;
    let lsz = lhs.z - rhs.z;
    let local_dist_sq = sqrtf(sq3(lsx, lsy, lsz));
    let scaled_hurt_radius = (hurt_radius * axis.x) / local_dist_sq;
    let contact_lerp = scaled_hurt_radius / axis.x;
    let allowed_distance = hit_radius + scaled_hurt_radius;
    *out_overlap = allowed_distance - axis.x;
    let hurt_closest_x = (*hurt_closest).x;
    (*out_contact_pos).x = fmadds(contact_lerp, (*hit_closest).x - hurt_closest_x, hurt_closest_x);
    let hurt_closest_y = (*hurt_closest).y;
    (*out_contact_pos).y = fmadds(contact_lerp, (*hit_closest).y - hurt_closest_y, hurt_closest_y);
    let hurt_closest_z = (*hurt_closest).z;
    (*out_contact_pos).z = fmadds(contact_lerp, (*hit_closest).z - hurt_closest_z, hurt_closest_z);
    !(allowed_distance < axis.x)
}

/// `sqrtf_store`: MSL `sqrtf` by three Newton steps on `__frsqrte` (the volatile spill is
/// not observable).
fn sqrtf_store(x: f32) -> f32 {
    if x > 0.0f32 {
        let xd = x as f64;
        let mut guess = frsqrte(xd);
        for _ in 0..3 {
            let g2 = guess * guess;
            guess = (0.5 * guess) * (-g2).mul_add(xd, 3.0);
        }
        return (xd * guess) as f32;
    }
    x
}

pub unsafe extern "C" fn lbColl_800077A0(a: *const Vec3, arg1: *const Mtx, b: *const Vec3, c: *const Vec3, d: *mut Vec3, e: *mut Vec3, angle: *mut f32, x: f32, dist_offset: f32) {
    melee_core::ct!("lbColl_800077A0");
    let diff_cb = Vec3 { x: (*c).x - (*b).x, y: (*c).y - (*b).y, z: (*c).z - (*b).z };

    if diff_cb.x != 0.0 || diff_cb.y != 0.0 || diff_cb.z != 0.0 {
        let normal_x = mtx::mult_vec(&*arg1, &CVec3 { x, y: 0.0, z: 0.0 });
        let multi_mtx = mtx::mult_vec(&*arg1, &CVec3 { x: 0.0, y: 0.0, z: 0.0 });

        // sqrDistance(&normal_x, &multi_mtx)
        let sx = normal_x.x - multi_mtx.x;
        let sy = normal_x.y - multi_mtx.y;
        let sz = normal_x.z - multi_mtx.z;
        let dist = sqrtf_store(sq3(sx, sy, sz));

        let offset_dist = dist + dist_offset;

        let diff_ba = Vec3 { x: (*b).x - (*a).x, y: (*b).y - (*a).y, z: (*b).z - (*a).z };

        // diff_cb's address goes to lbVector_AngleXY (not inlined), so MWCC keeps it in memory and does
        // not fuse these squares (fmuls x3, fadds x2; notes/fp-audit.md). ba_dot below stays fused.
        let dot_diff_cb = diff_cb.x * diff_cb.x + diff_cb.y * diff_cb.y + diff_cb.z * diff_cb.z;

        let scl;
        if approximatelyZero(dot_diff_cb) {
            scl = 0.0f32;
        } else {
            let n0 = fmadds(2.0f32 * diff_cb.z, diff_ba.z, fmadds(2.0f32 * diff_cb.x, diff_ba.x, (2.0f32 * diff_cb.y) * diff_ba.y));
            let ba_dot = sq3(diff_ba.x, diff_ba.y, diff_ba.z);
            let mut n1 = fmsubs(n0, n0, (4.0f32 * dot_diff_cb) * -(fmsubs(offset_dist, offset_dist, ba_dot)));
            if n1 < 0.0f32 {
                n1 = 0.0f32;
            }
            // The console takes this square root (DOL 0x800079CC: frsqrte + 3 Newton steps). The C PC build passed
            // `sqrt_tmp - 1` (outside the array) as the spill slot and clang folded the UB store away, returning n1
            // unchanged; both builds copied that until the Dolphin oracle showed Falco's laser bouncing off a shield
            // with a different normal (notes/fp-audit.md, "PC-only UB in a console sqrt").
            scl = (-n0 - sqrtf_store(n1)) / (2.0f32 * dot_diff_cb);
        }

        let normalize_e = CVec3 {
            x: fmadds(scl, diff_cb.x, (*b).x) - (*a).x,
            y: fmadds(scl, diff_cb.y, (*b).y) - (*a).y,
            z: fmadds(scl, diff_cb.z, (*b).z) - (*a).z,
        };

        *e = uv(mtx::vec_normalize(&normalize_e));

        *angle = lbvector::angle_xy(&cv(&*e), &cv(&diff_cb));
        (*d).x = fmadds(dist, (*e).x, (*a).x);
        (*d).y = fmadds(dist, (*e).y, (*a).y);
        (*d).z = fmadds(dist, (*e).z, (*a).z);
    } else {
        *angle = core::f64::consts::PI as f32;
        (*e).z = 0.0;
        (*e).y = 0.0;
        (*e).x = 0.0;
    }
}

pub unsafe extern "C" fn lbColl_80007AFC(a: *mut HitCapsule, b: *mut HitCapsule, x: f32, y: f32) -> bool {
    melee_core::ct!("lbColl_80007AFC");
    let a_val = if (*a).x43_b1() != 0 { (*a).scale } else { (*a).scale * x };
    let b_val = if (*b).x43_b1() != 0 { (*b).scale } else { (*b).scale * y };
    lbColl_80006094(&(*b).x58, &(*b).x4C, &(*a).x58, &(*a).x4C, &raw mut (*b).hurt_coll_pos, &raw mut (*a).hurt_coll_pos, b_val, a_val)
}

pub unsafe extern "C" fn lbColl_80007B78(a: *mut Fighter_x1614_t, b: *mut Fighter_x1614_t, x: f32, y: f32) -> bool {
    melee_core::ct!("lbColl_80007B78");
    lbColl_800067F8(&(*b).x14, &(*b).x8, &(*a).x14, &(*a).x8, &raw mut (*b).x20, &raw mut (*a).x20, (*b).x0 * y, (*a).x0 * x)
}

pub unsafe extern "C" fn lbColl_80007BCC(arg0: *mut HitCapsule, shield_hit: *mut HitResult, arg2: *const Mtx, arg3: i32, arg4: f32, arg5: f32, arg6: f32) -> bool {
    melee_core::ct!("lbColl_80007BCC");
    let mut sp74 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp68 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp38: Mtx = [[0.0; 4]; 3];

    if (*shield_hit).skip_update_pos() == 0 {
        ext::lb_8000B1CC((*shield_hit).bone, &raw mut (*shield_hit).offset, &raw mut (*shield_hit).pos);
        if !arg2.is_null() {
            (*shield_hit).pos.z = arg6;
        }
        (*shield_hit).set_skip_update_pos(1);
    }
    if arg3 != 0 {
        (*arg0).hurt_coll_pos = (*shield_hit).pos;
        (*arg0).coll_distance = 0.0;
        return true;
    }
    if !arg2.is_null() {
        ext::PSMTXConcat(arg2 as *mut c_void, jobj_mtx((*shield_hit).bone) as *mut c_void, (&mut sp38 as *mut Mtx).cast());
    }
    let hurt_mtx: *const Mtx = if !arg2.is_null() { &sp38 } else { jobj_mtx((*shield_hit).bone) };
    let hit_radius = if (*arg0).x43_b1() != 0 { (*arg0).scale } else { (*arg0).scale * arg4 };

    lbColl_80006E58(
        &(*arg0).x58,
        &(*arg0).x4C,
        &(*shield_hit).pos,
        &(*shield_hit).pos,
        &mut sp74,
        &mut sp68,
        hurt_mtx,
        &raw mut (*arg0).hurt_coll_pos,
        &raw mut (*arg0).coll_distance,
        hit_radius,
        (*shield_hit).size,
        20.0f32 * arg5,
    )
}

pub unsafe extern "C" fn lbColl_80007DD8(capsule: *mut HitCapsule, hit: *mut HitResult, hit_transform: *const Mtx, arg3: *mut Vec3, angle: *mut f32, scale: f32) {
    melee_core::ct!("lbColl_80007DD8");
    let mut unused_result = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut transformed_hit: Mtx = [[0.0; 4]; 3];

    if !hit_transform.is_null() {
        transformed_hit = mtx::concat(&*hit_transform, &*jobj_mtx((*hit).bone));
    }
    let dist_offset = if (*capsule).x43_b1() != 0 { (*capsule).scale } else { (*capsule).scale * scale };
    let m: *const Mtx = if !hit_transform.is_null() { &transformed_hit } else { jobj_mtx((*hit).bone) };
    lbColl_800077A0(&(*hit).pos, m, &(*capsule).x58, &(*capsule).x4C, &mut unused_result, arg3, angle, (*hit).size, dist_offset);
}

/// Shared by the hurt-capsule tests: refresh the capsule's world endpoints.
unsafe fn update_hurt_pos(arg1: *mut HurtCapsule, has_mtx: bool, z: f32) {
    if (*arg1).skip_update_pos() == 0 {
        ext::lb_8000B1CC((*arg1).bone, &raw mut (*arg1).a_offset, &raw mut (*arg1).a_pos);
        ext::lb_8000B1CC((*arg1).bone, &raw mut (*arg1).b_offset, &raw mut (*arg1).b_pos);
        if has_mtx {
            (*arg1).b_pos.z = z;
            (*arg1).a_pos.z = z;
        }
        (*arg1).set_skip_update_pos(1);
    }
}

pub unsafe extern "C" fn lbColl_80007ECC(arg0: *mut HitCapsule, arg1: *mut HurtCapsule, arg2: *const Mtx, hit_scl_y: f32, hurt_scl_y: f32, hurt_pos_z: f32) -> bool {
    melee_core::ct!("lbColl_80007ECC");
    let mut sp70 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp64 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp34: Mtx = [[0.0; 4]; 3];

    if (*arg1).state == HurtCapsule_Enabled {
        update_hurt_pos(arg1, !arg2.is_null(), hurt_pos_z);
        if !arg2.is_null() {
            sp34 = mtx::concat(&*arg2, &*jobj_mtx((*arg1).bone));
        }
        let hurt_mtx: *const Mtx = if !arg2.is_null() { &sp34 } else { jobj_mtx((*arg1).bone) };
        let hit_radius = if (*arg0).x43_b1() != 0 { (*arg0).scale } else { (*arg0).scale * hit_scl_y };
        return lbColl_80006E58(
            &(*arg0).x58,
            &(*arg0).x4C,
            &(*arg1).a_pos,
            &(*arg1).b_pos,
            &mut sp70,
            &mut sp64,
            hurt_mtx,
            &raw mut (*arg0).hurt_coll_pos,
            &raw mut (*arg0).coll_distance,
            hit_radius,
            (*arg1).scale,
            3.0f32 * hurt_scl_y,
        );
    }
    false
}

pub unsafe extern "C" fn lbColl_8000805C(arg0: *mut HitCapsule, arg1: *mut HurtCapsule, arg2: *const Mtx, arg3: i32, arg4: f32, arg5: f32, arg6: f32) -> bool {
    melee_core::ct!("lbColl_8000805C");
    let mut sp74 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp68 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp38: Mtx = [[0.0; 4]; 3];

    if (*arg1).state != HurtCapsule_Intangible {
        update_hurt_pos(arg1, !arg2.is_null(), arg6);
        if arg3 != 0 {
            (*arg0).hurt_coll_pos.x = 0.5f32 * ((*arg1).a_pos.x + (*arg1).b_pos.x);
            (*arg0).hurt_coll_pos.y = 0.5f32 * ((*arg1).a_pos.y + (*arg1).b_pos.y);
            (*arg0).hurt_coll_pos.z = 0.5f32 * ((*arg1).a_pos.z + (*arg1).b_pos.z);
            (*arg0).coll_distance = 5.0;
            return true;
        }
        if !arg2.is_null() {
            sp38 = mtx::concat(&*arg2, &*jobj_mtx((*arg1).bone));
        }
        let var_r9: *const Mtx = if !arg2.is_null() { &sp38 } else { jobj_mtx((*arg1).bone) };
        let hit_radius = if (*arg0).x43_b1() != 0 { (*arg0).scale } else { (*arg0).scale * arg4 };
        return lbColl_80006E58(
            &(*arg0).x58,
            &(*arg0).x4C,
            &(*arg1).a_pos,
            &(*arg1).b_pos,
            &mut sp74,
            &mut sp68,
            var_r9,
            &raw mut (*arg0).hurt_coll_pos,
            &raw mut (*arg0).coll_distance,
            hit_radius,
            (*arg1).scale,
            3.0f32 * arg5,
        );
    }
    false
}

pub unsafe extern "C" fn lbColl_80008248(arg0: *mut HitCapsule, arg1: *mut HurtCapsule, arg2: *const Mtx, arg3: f32, arg4: f32, arg5: f32) -> bool {
    melee_core::ct!("lbColl_80008248");
    let mut sp70 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp64 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp34: Mtx = [[0.0; 4]; 3];

    update_hurt_pos(arg1, !arg2.is_null(), arg5);
    if !arg2.is_null() {
        sp34 = mtx::concat(&*arg2, &*jobj_mtx((*arg1).bone));
    }
    let var_r9: *const Mtx = if !arg2.is_null() { &sp34 } else { jobj_mtx((*arg1).bone) };
    let var_f1 = if (*arg0).x43_b1() != 0 { (*arg0).scale } else { (*arg0).scale * arg3 };
    lbColl_80006E58(
        &(*arg0).x58,
        &(*arg0).x4C,
        &(*arg1).a_pos,
        &(*arg1).b_pos,
        &mut sp70,
        &mut sp64,
        var_r9,
        &raw mut (*arg0).hurt_coll_pos,
        &raw mut (*arg0).coll_distance,
        var_f1,
        (*arg1).scale,
        3.0f32 * arg4,
    )
}

pub unsafe extern "C" fn lbColl_800083C4(arg0: *mut HurtCapsule) {
    melee_core::ct!("lbColl_800083C4");
    if (*arg0).skip_update_pos() != 0 {
        return;
    }
    ext::lb_8000B1CC((*arg0).bone, &raw mut (*arg0).a_offset, &raw mut (*arg0).a_pos);
    ext::lb_8000B1CC((*arg0).bone, &raw mut (*arg0).b_offset, &raw mut (*arg0).b_pos);
    (*arg0).set_skip_update_pos(1);
}

pub unsafe extern "C" fn lbColl_80008428(arg0: *mut HitCapsule) {
    melee_core::ct!("lbColl_80008428");
    (*arg0).state = HitCapsule_Disabled;
}

pub unsafe extern "C" fn lbColl_80008434(arg0: *mut HitCapsule) {
    melee_core::ct!("lbColl_80008434");
    (*arg0).state = HitCapsule_Enabled;
}

pub unsafe extern "C" fn lbColl_80008440(hit: *mut HitCapsule) {
    melee_core::ct!("lbColl_80008440");
    for i in 0..12 {
        (*hit).victims_1[i].victim = null_mut();
    }
    (*hit).x44 = 0;
    for i in 0..12 {
        (*hit).victims_2[i].victim = null_mut();
    }
    (*hit).x45 = 0;
}

pub unsafe extern "C" fn lbColl_CopyHitCapsule(src: *const HitCapsule, dst: *mut HitCapsule) {
    melee_core::ct!("lbColl_CopyHitCapsule");
    for i in 0..12 {
        (*dst).victims_1[i] = (*src).victims_1[i];
        (*dst).victims_2[i] = (*src).victims_2[i];
    }
    (*dst).x44 = (*src).x44;
    (*dst).x45 = (*src).x45;
}

pub unsafe extern "C" fn lbColl_80008688(capsule: *mut HitCapsule, ty: i32, victim: *mut c_void) -> bool {
    melee_core::ct!("lbColl_80008688");
    let mut i = 0usize;
    while i < 12 {
        if (*capsule).victims_1[i].victim == victim {
            match ty {
                2 | 4 | 5 | 7 | 8 => (*capsule).victims_1[i].x4 = (*capsule).x40_b4() as u32,
                _ => {}
            }
            return false;
        }
        i += 1;
    }

    i = 0;
    while i < 12 {
        if (*capsule).victims_1[i].victim.is_null() {
            break;
        }
        i += 1;
    }

    let var_r0 = if i == 12 { (*capsule).x44 as usize } else { i };

    (*capsule).victims_1[var_r0].victim = victim;
    match ty {
        2 | 4 | 5 | 7 | 8 => (*capsule).victims_1[var_r0].x4 = (*capsule).x40_b4() as u32,
        _ => (*capsule).victims_1[var_r0].x4 = 0,
    }

    if i == 12 {
        (*capsule).x44 = (*capsule).x44.wrapping_add(1);
        if (*capsule).x44 >= 0xC {
            (*capsule).x44 = 0;
        }
    }
    true
}

pub unsafe extern "C" fn lbColl_80008820(capsule: *mut HitCapsule, ty: i32, victim: *mut c_void) -> bool {
    melee_core::ct!("lbColl_80008820");
    let mut i = 0usize;
    while i < 12 {
        if (*capsule).victims_2[i].victim == victim {
            match ty {
                2 | 5 | 7 | 8 => (*capsule).victims_2[i].x4 = (*capsule).x40_b4() as u32,
                _ => {}
            }
            return false;
        }
        i += 1;
    }

    i = 0;
    while i < 12 {
        if (*capsule).victims_2[i].victim.is_null() {
            break;
        }
        i += 1;
    }

    let var_r0 = if i == 12 { (*capsule).x45 as usize } else { i };

    (*capsule).victims_2[var_r0].victim = victim;
    match ty {
        2 | 5 | 7 | 8 => (*capsule).victims_2[var_r0].x4 = (*capsule).x40_b4() as u32,
        _ => (*capsule).victims_2[var_r0].x4 = 0,
    }

    if i == 12 {
        (*capsule).x45 = (*capsule).x45.wrapping_add(1);
        if (*capsule).x45 >= 12 {
            (*capsule).x45 = 0;
        }
    }
    true
}

pub unsafe extern "C" fn lbColl_800089B8(hit: *mut HitCapsule, arg1: *mut c_void) {
    melee_core::ct!("lbColl_800089B8");
    for i in 0..12 {
        if (*hit).victims_1[i].victim == arg1 {
            (*hit).victims_1[i].victim = null_mut();
        }
    }
}

pub unsafe extern "C" fn lbColl_80008A5C(hit: *mut HitCapsule) {
    melee_core::ct!("lbColl_80008A5C");
    if (*hit).state != HitCapsule_Disabled {
        for i in 0..12 {
            if !(*hit).victims_1[i].victim.is_null() {
                let temp_r4 = (*hit).victims_1[i].x4;
                if temp_r4 != 0 {
                    (*hit).victims_1[i].x4 = temp_r4 - 1;
                    if (*hit).victims_1[i].x4 == 0 {
                        (*hit).victims_1[i].victim = null_mut();
                    }
                }
            }
        }
        for i in 0..12 {
            if !(*hit).victims_2[i].victim.is_null() {
                let temp_r4 = (*hit).victims_2[i].x4;
                if temp_r4 != 0 {
                    (*hit).victims_2[i].x4 = temp_r4 - 1;
                    if (*hit).victims_2[i].x4 == 0 {
                        (*hit).victims_2[i].victim = null_mut();
                    }
                }
            }
        }
    }
}

pub unsafe extern "C" fn lbColl_80008D30(arg0: *mut HitCapsule, arg1: *const lbColl_80008D30_arg1) {
    melee_core::ct!("lbColl_80008D30");
    (*arg0).state = (*arg1).state;
    (*arg0).damage = (*arg1).damage as f32;
    (*arg0).kb_angle = (*arg1).kb_angle;
    (*arg0).x24 = (*arg1).unkC;
    (*arg0).x28 = (*arg1).unk10;
    (*arg0).x2C = (*arg1).unk14;
    (*arg0).element = (*arg1).element;
    (*arg0).sfx_severity = (*arg1).sfx_severity;
    (*arg0).sfx_kind = (*arg1).sfx_kind;
    (*arg0).unk_count = (*arg1).damage;
}

pub static lbColl_804D7A50: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

pub unsafe extern "C" fn lbColl_80008DA4(arg0: *const GXColor, arg1: *const GXColor) {
    melee_core::ct!("lbColl_80008DA4");
    let mut sp10 = lbColl_804D7A50;
    ext::GXSetColorUpdate(1);
    ext::GXSetAlphaUpdate(0);
    ext::GXSetFog(GX_FOG_NONE, 0.0, 0.0, 0.0, 0.0, sp10);
    let var_r3 = if (*arg0).a < 0xFF { GX_BM_BLEND } else { GX_BM_NONE };
    ext::GXSetBlendMode(var_r3, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_NOOP);
    ext::GXSetAlphaCompare(GX_GREATER, 0, GX_AOP_AND, GX_GREATER, 0);
    let var_r5: u8 = if (*arg0).a < 0xFF { 0 } else { 1 };
    ext::GXSetZMode(1, GX_LEQUAL, var_r5);
    ext::GXSetZCompLoc(0);
    ext::GXSetNumTexGens(0);
    ext::GXSetTevClampMode(0, 0);
    ext::GXSetNumTevStages(1);
    ext::GXSetTevOrder(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
    ext::GXSetTevColor(GX_TEVREG0, *arg0);
    ext::GXSetTevColorIn(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_RASC, GX_CC_C0, GX_CC_ZERO);
    ext::GXSetTevAlphaIn(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_KONST, GX_CA_A0, GX_CA_ZERO);
    ext::GXSetTevColorOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
    ext::GXSetTevAlphaOp(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, 1, GX_TEVPREV);
    ext::GXSetNumChans(1);
    sp10 = *arg1;
    ext::GXSetChanAmbColor(GX_COLOR0A0, sp10);
    sp10.r = 0xFF;
    sp10.g = 0xFF;
    sp10.b = 0xFF;
    sp10.a = 0xFF;
    ext::GXSetChanMatColor(GX_COLOR0A0, sp10);
    if (*arg0).a < 0xFF {
        ext::GXSetChanCtrl(GX_COLOR0A0, 0, GX_SRC_REG, GX_SRC_REG, 1, GX_DF_NONE, GX_AF_NONE);
    } else {
        ext::GXSetChanCtrl(GX_COLOR0A0, 1, GX_SRC_REG, GX_SRC_REG, 1, GX_DF_CLAMP, GX_AF_NONE);
    }
}

/// The GXSETARRAY / GXSetVtxDesc / GXSetVtxAttrFmt prologue shared by the capsule draws.
unsafe fn setup_sphere_arrays() {
    ext::GXClearVtxDesc();
    ext::GXSetArray(GX_VA_POS, lbColl_SpherePositions.as_ptr().cast(), 6);
    ext::GXSetArray(GX_VA_NRM, lbColl_SphereNormals.as_ptr().cast(), 6);
    ext::GXSetVtxDesc(GX_VA_POS, GX_INDEX8);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_TEX_ST, GX_RGBA4, 0xE);
    ext::GXSetVtxDesc(GX_VA_NRM, GX_INDEX8);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_NRM, GX_TEX_S, GX_RGBA4, 0xE);
}

unsafe fn setup_cylinder_arrays() {
    ext::GXClearVtxDesc();
    ext::GXSetArray(GX_VA_POS, lbColl_CylinderPositions.as_ptr().cast(), 6);
    ext::GXSetArray(GX_VA_NRM, lbColl_CylinderNormals.as_ptr().cast(), 6);
    ext::GXSetVtxDesc(GX_VA_POS, GX_INDEX8);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_TEX_ST, GX_RGBA4, 0xE);
    ext::GXSetVtxDesc(GX_VA_NRM, GX_INDEX8);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_NRM, GX_TEX_S, GX_RGBA4, 6);
}

/// Build the sphere/sphere/cylinder matrices for a capsule from `p0` to `p1`, radius `r`.
/// Returns (spD4, spA4, sp74, var_f31) as the C leaves them before the view matrix is applied.
fn capsule_mtxs(p0: &Vec3, p1: &Vec3, r: f32) -> (Mtx, Mtx, Mtx, f32) {
    let mut spa4 = mtx::scale(r, r, r);
    let mut sp38 = Vec3 { x: p0.x - p1.x, y: p0.y - p1.y, z: p0.z - p1.z };
    // SQ(sp38.x) + ...: sp38 lives in memory (its address is taken), not fused on the console
    let var_f31 = sqrtf(sp38.x * sp38.x + sp38.y * sp38.y + sp38.z * sp38.z);
    let mut spd4: Mtx;
    let mut sp74: Mtx = [[0.0; 4]; 3];
    if isSmall(var_f31) || (isSmall(sp38.x) && isSmall(sp38.y) && isSmall(sp38.z)) {
        let mut sp44 = mtx::identity();
        sp44[0][0] = -1.0;
        sp44[1][0] = 0.0;
        sp44[2][0] = 0.0;
        sp44[0][1] = 0.0;
        sp44[1][1] = 1.0;
        sp44[2][1] = 0.0;
        sp44[0][2] = 0.0;
        sp44[1][2] = 0.0;
        sp44[2][2] = -1.0;
        spd4 = mtx::concat(&sp44, &spa4);
    } else {
        sp74 = mtx::scale(var_f31, r, r);
        let mut sp2c;
        if isSmall(sp38.x) && isSmall(sp38.y) {
            sp2c = Vec3 { x: sp38.z, y: 0.0, z: 0.0 };
        } else {
            sp2c = Vec3 { x: sp38.y, y: -sp38.x, z: 0.0 };
        }
        sp38 = uv(mtx::vec_normalize(&cv(&sp38)));
        sp2c = uv(mtx::vec_normalize(&cv(&sp2c)));
        let sp20 = uv(mtx::vec_cross(&cv(&sp38), &cv(&sp2c)));
        let mut sp44 = mtx::identity();
        sp44[0][0] = sp38.x;
        sp44[1][0] = sp38.y;
        sp44[2][0] = sp38.z;
        sp44[0][1] = sp2c.x;
        sp44[1][1] = sp2c.y;
        sp44[2][1] = sp2c.z;
        sp44[0][2] = sp20.x;
        sp44[1][2] = sp20.y;
        sp44[2][2] = sp20.z;
        spd4 = mtx::concat(&sp44, &spa4);
        sp74 = mtx::concat(&sp44, &sp74);
        sp44[0][0] = -sp38.x;
        sp44[1][0] = -sp38.y;
        sp44[2][0] = -sp38.z;
        sp44[0][1] = sp2c.x;
        sp44[1][1] = sp2c.y;
        sp44[2][1] = sp2c.z;
        sp44[0][2] = -sp20.x;
        sp44[1][2] = -sp20.y;
        sp44[2][2] = -sp20.z;
        spa4 = mtx::concat(&sp44, &spa4);
    }
    let sp44 = mtx::trans(p0.x, p0.y, p0.z);
    spd4 = mtx::concat(&sp44, &spd4);
    let sp44 = mtx::trans(p1.x, p1.y, p1.z);
    spa4 = mtx::concat(&sp44, &spa4);
    let sp44 = mtx::trans(
        (0.5f64 * (p0.x + p1.x) as f64) as f32,
        (0.5f64 * (p0.y + p1.y) as f64) as f32,
        (0.5f64 * (p0.z + p1.z) as f64) as f32,
    );
    sp74 = mtx::concat(&sp44, &sp74);
    (spd4, spa4, sp74, var_f31)
}

/// Load `view * m`, its inverse transpose as the normal matrix, and draw the display list.
unsafe fn draw_list(view: &Mtx, m: &Mtx, list: &[u8]) {
    let mut c = mtx::concat(view, m);
    ext::GXLoadPosMtxImm((&c as *const _ as *mut Mtx).cast(), 0);
    hsd_mtx::inverse_in_place(&mut c);
    c = mtx::transpose(&c);
    ext::GXLoadNrmMtxImm((&c as *const _ as *mut Mtx).cast(), 0);
    ext::GXCallDisplayList(list.as_ptr().cast_mut().cast(), list.len() as u32);
}

pub unsafe extern "C" fn lbColl_80008FC8(arg0: Vec3, arg1: Vec3, arg2: *const GXColor, arg3: *const GXColor, arg4: f32) {
    melee_core::ct!("lbColl_80008FC8");
    let mut sp104: Mtx = [[0.0; 4]; 3];

    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
    lbColl_80008DA4(arg2, arg3);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), (&mut sp104 as *mut Mtx).cast());
    let (spd4, spa4, sp74, var_f31) = capsule_mtxs(&arg0, &arg1, arg4);
    ext::GXSetCullMode(GX_CULL_BACK);
    setup_sphere_arrays();
    draw_list(&sp104, &spd4, &lbColl_SphereDisplayList);
    setup_sphere_arrays();
    draw_list(&sp104, &spa4, &lbColl_SphereDisplayList);
    if !isSmall(var_f31) {
        setup_cylinder_arrays();
        draw_list(&sp104, &sp74, &lbColl_CylinderDisplayList);
    }
    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
}

pub unsafe extern "C" fn lbColl_800096B4(arg0: *const Mtx, arg1: Vec3, arg2: Vec3, arg3: *const GXColor, arg4: *const GXColor, arg5: f32) {
    melee_core::ct!("lbColl_800096B4");
    let mut sp108: Mtx = [[0.0; 4]; 3];

    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
    lbColl_80008DA4(arg3, arg4);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), (&mut sp108 as *mut Mtx).cast());
    let (mut spd8, mut spa8, mut sp78, var_f31) = capsule_mtxs(&arg1, &arg2, arg5);
    spd8 = mtx::concat(&*arg0, &spd8);
    spa8 = mtx::concat(&*arg0, &spa8);
    sp78 = mtx::concat(&*arg0, &sp78);
    ext::GXSetCullMode(GX_CULL_BACK);
    setup_sphere_arrays();
    draw_list(&sp108, &spd8, &lbColl_SphereDisplayList);
    setup_sphere_arrays();
    draw_list(&sp108, &spa8, &lbColl_SphereDisplayList);
    if !isSmall(var_f31) {
        setup_cylinder_arrays();
        draw_list(&sp108, &sp78, &lbColl_CylinderDisplayList);
    }
    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
}

pub unsafe extern "C" fn lbColl_80009DD4(v0: *const Vec3, v1: *const Vec3, clr: *const GXColor) {
    melee_core::ct!("lbColl_80009DD4");
    let mut view_mtx: Mtx = [[0.0; 4]; 3];

    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
    lbColl_80008DA4(clr, clr);
    ext::HSD_ClearVtxDesc();
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    ext::GXSetVtxAttrFmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    ext::GXSetVtxDesc(GX_VA_POS, GX_DIRECT);
    ext::GXSetVtxDesc(GX_VA_CLR0, GX_DIRECT);
    ext::GXSetCurrentMtx(0);
    ext::GXSetCullMode(GX_CULL_NONE);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), (&mut view_mtx as *mut Mtx).cast());
    ext::GXLoadPosMtxImm((&view_mtx as *const Mtx as *mut Mtx).cast(), 0);
    ext::GXBegin(GX_QUADS, GX_VTXFMT0, 4);

    let z0 = (*v0).z;
    let y0 = (*v0).y;
    let x0 = (*v0).x;
    ext::GXPosition3f32(x0, y0, z0);
    let a = (*clr).a;
    let b = (*clr).b;
    let g = (*clr).g;
    let r = (*clr).r;
    ext::GXColor4u8(r, g, b, a);

    let x1 = (*v1).x;
    ext::GXPosition3f32(x1, y0, z0);
    ext::GXColor4u8(r, g, b, a);

    let y1 = (*v1).y;
    ext::GXPosition3f32(x1, y1, z0);
    ext::GXColor4u8(r, g, b, a);

    ext::GXPosition3f32(x0, y1, z0);
    ext::GXColor4u8(r, g, b, a);

    ext::HSD_StateInvalidate(-1);
    ext::HSD_StateInitTev();
}

#[inline]
fn pass_for(c: &GXColor) -> u32 {
    if c.a == 0xFF { 0 } else { 2 }
}

pub unsafe extern "C" fn lbColl_80009F54(hit: *mut HitCapsule, arg1: u32, arg8: f32) -> bool {
    melee_core::ct!("lbColl_80009F54");
    if (*hit).state != HitCapsule_Disabled && (*hit).x43_b2() == 0 {
        let var_r5: *mut GXColor = match (*hit).element {
            HitElement_Catch => &raw mut lbColl_804D36A4,
            HitElement_Inert => &raw mut lbColl_804D36DC,
            _ => &raw mut lbColl_804D36A0,
        };
        if pass_for(&*var_r5) == arg1 {
            let var_f1 = if (*hit).x43_b1() != 0 { (*hit).scale } else { (*hit).scale * arg8 };
            lbColl_80008FC8((*hit).x58, (*hit).x4C, var_r5, &raw mut lbColl_804D36A8, var_f1);
            return true;
        }
    }
    false
}

/// `lbColl_DrawHitResult`: the capsule drawn in the local space of `mtx`.
unsafe fn lbColl_DrawHitResult(m: *const Mtx, a: &Vec3, b: &Vec3, c0: *const GXColor, c1: *const GXColor, size: f32) {
    let sp3c = hsd_mtx::inverse(&*m);
    let sp30 = uv(mtx::mult_vec(&sp3c, &cv(a)));
    let sp24 = uv(mtx::mult_vec(&sp3c, &cv(b)));
    lbColl_800096B4(m, sp30, sp24, c0, c1, size);
}

pub unsafe extern "C" fn lbColl_8000A044(hit: *mut HitCapsule, arg1: u32, arg8: f32) -> bool {
    melee_core::ct!("lbColl_8000A044");
    if (*hit).state != HitCapsule_Disabled && (*hit).x43_b2() == 0 {
        if pass_for(&lbColl_804D36E8) == arg1 {
            let var_f1 = if (*hit).x43_b1() != 0 { (*hit).scale } else { (*hit).scale * arg8 };
            lbColl_80008FC8((*hit).x58, (*hit).x4C, &raw const lbColl_804D36E8, &raw const lbColl_804D36EC, var_f1);
            return true;
        }
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A10C(arg0: *mut lbColl_8000A10C_arg0_t, arg1: u32, arg2: f32) -> bool {
    melee_core::ct!("lbColl_8000A10C");
    let c = &raw mut lbColl_804D36F8;
    if pass_for(&*c) == arg1 {
        lbColl_80008FC8((*arg0).x14, (*arg0).x8, c, &raw const lbColl_804D36FC, (*arg0).x0 * arg2);
        return true;
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A1A8(arg0: *mut Fighter_x1614_t, arg1: i32, scale_y: f32) -> bool {
    melee_core::ct!("lbColl_8000A1A8");
    if pass_for(&lbColl_804D36F0) == arg1 as u32 {
        lbColl_80008FC8((*arg0).x14, (*arg0).x8, &raw const lbColl_804D36F0, &raw const lbColl_804D36F4, (*arg0).x0 * scale_y);
        return true;
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A244(hurt: *mut HurtCapsule, arg1: u32, arg2: *const Mtx, arg3: f32) -> bool {
    melee_core::ct!("lbColl_8000A244");
    let mut sp9c: Mtx = [[0.0; 4]; 3];
    let temp_r3 = lbColl_803B9928[(*hurt).state as usize].pad;
    if pass_for(&*temp_r3) == arg1 {
        update_hurt_pos(hurt, !arg2.is_null(), arg3);
        if !arg2.is_null() {
            sp9c = mtx::concat(&*arg2, &*jobj_mtx((*hurt).bone));
        }
        let temp_f31 = (*hurt).scale;
        let temp_r31_2 = lbColl_803B9928[(*hurt).state as usize].pad_x;
        let sp84 = (*hurt).b_pos;
        let sp90 = (*hurt).a_pos;
        let var_r28: *const Mtx = if !arg2.is_null() { &sp9c } else { jobj_mtx((*hurt).bone) };
        lbColl_DrawHitResult(var_r28, &sp90, &sp84, temp_r3, temp_r31_2, temp_f31);
        return true;
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A460(hurt: *mut Fighter_x1670_t, arg1: u32) -> bool {
    melee_core::ct!("lbColl_8000A460");
    let r31 = &raw mut lbColl_804D36E0;
    let r30 = &raw mut lbColl_804D36E4;
    if pass_for(&*r31) == arg1 {
        jobj_setup_matrix((*hurt).jobj);
        lbColl_800096B4(jobj_mtx((*hurt).jobj), (*hurt).v1, (*hurt).v1, r31, r30, (*hurt).v2);
        return true;
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A584(hurt: *mut HurtCapsule, arg1: u32, arg2: u32, arg3: *const Mtx, arg8: f32) -> bool {
    melee_core::ct!("lbColl_8000A584");
    let mut spa0: Mtx = [[0.0; 4]; 3];
    let mut var_r4 = arg1;
    if (*hurt).state == HurtCapsule_Intangible {
        var_r4 = 2;
    }
    let temp_r3 = lbColl_803B9928[var_r4 as usize].pad;
    if pass_for(&*temp_r3) == arg2 {
        update_hurt_pos(hurt, !arg3.is_null(), arg8);
        if !arg3.is_null() {
            spa0 = mtx::concat(&*arg3, &*jobj_mtx((*hurt).bone));
        }
        let temp_f31 = (*hurt).scale;
        let temp_r31_2 = lbColl_803B9928[var_r4 as usize].pad_x;
        let sp88 = (*hurt).b_pos;
        let sp94 = (*hurt).a_pos;
        let m: *const Mtx = if !arg3.is_null() { &spa0 } else { jobj_mtx((*hurt).bone) };
        lbColl_DrawHitResult(m, &sp94, &sp88, temp_r3, temp_r31_2, temp_f31);
        return true;
    }
    false
}

unsafe fn lbColl_DrawHit(hit: *mut HitResult, arg1: u32, arg2: *const Mtx, pos_z: f32, color: *const GXColor, secondary_color: *const GXColor) -> bool {
    let mut hit_mtx: Mtx = [[0.0; 4]; 3];
    if pass_for(&*color) == arg1 {
        if (*hit).skip_update_pos() == 0 {
            ext::lb_8000B1CC((*hit).bone, &raw mut (*hit).offset, &raw mut (*hit).pos);
            if !arg2.is_null() {
                (*hit).pos.z = pos_z;
            }
            (*hit).set_skip_update_pos(1);
        }
        if !arg2.is_null() {
            hit_mtx = mtx::concat(&*arg2, &*jobj_mtx((*hit).bone));
        }
        let temp_f31 = (*hit).size;
        let pos_b = (*hit).pos;
        let pos_a = (*hit).pos;
        let var_r31: *const Mtx = if !arg2.is_null() { &hit_mtx } else { jobj_mtx((*hit).bone) };
        lbColl_DrawHitResult(var_r31, &pos_a, &pos_b, color, secondary_color, temp_f31);
        return true;
    }
    false
}

pub unsafe extern "C" fn lbColl_8000A78C(hit: *mut HitResult, arg1: u32, arg2: *const Mtx, pos_z: f32) -> bool {
    melee_core::ct!("lbColl_8000A78C");
    lbColl_DrawHit(hit, arg1, arg2, pos_z, &raw const lbColl_804D36C4, &raw const lbColl_804D36C8)
}

pub unsafe extern "C" fn lbColl_8000A95C(hit: *mut HitResult, arg1: u32, arg2: *const Mtx, pos_z: f32) -> bool {
    melee_core::ct!("lbColl_8000A95C");
    lbColl_DrawHit(hit, arg1, arg2, pos_z, &raw const lbColl_804D36CC, &raw const lbColl_804D36D0)
}

pub unsafe extern "C" fn lbColl_8000AB2C(hit: *mut HitResult, arg1: u32, arg2: *const Mtx, pos_z: f32) -> bool {
    melee_core::ct!("lbColl_8000AB2C");
    lbColl_DrawHit(hit, arg1, arg2, pos_z, &raw const lbColl_804D36D4, &raw const lbColl_804D36D8)
}

pub unsafe extern "C" fn lbColl_8000ACFC(victim: *mut c_void, hitbox: *const HitCapsule) -> bool {
    melee_core::ct!("lbColl_8000ACFC");
    let mut i = 0usize;
    while i < 12 {
        if (*hitbox).victims_1[i].victim == victim {
            break;
        }
        i += 1;
    }
    i != 12
}
