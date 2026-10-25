//! Native-layout port of `src/sysdolphin/baselib/displayfunc.c` (raw pointers over the generated
//! `HSD_JObj` / `HSD_DObj` / `HSD_CObj` records): the billboard matrices, `HSD_JObjMakePositionMtx`,
//! the envelope node matrix, the JObj / DObj display walk, the z-sorted list, `HSD_EraseRect`.
//! `tu::displayfunc` is the descriptor-driven renderer of the platform draw path; this is the
//! C-shaped surface, checked against the C by `tests/tu_displayfunc_native.rs` (gen/mtu).
//!
//! Native-layout note: `_HSD_ZListSort` reaches the list links through byte offsets (0x3C / 0x40 on the
//! console); here they are the offsets of `sort.texedge` / `sort.xlu` in the native record.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, dead_code)]
use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
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
    fn GXClearVtxDesc();
    fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8);
    fn GXInitTexObj(a0: *mut _GXTexObj, a1: *mut c_void, a2: u16, a3: u16, a4: u32, a5: u32, a6: u32, a7: u8);
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXLoadTexObj(a0: *mut _GXTexObj, a1: u32);
    fn GXPosition3f32(a0: f32, a1: f32, a2: f32);
    fn GXSetAlphaCompare(a0: u32, a1: u8, a2: u32, a3: u32, a4: u8);
    fn GXSetAlphaUpdate(a0: u8);
    fn GXSetBlendMode(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn GXSetColorUpdate(a0: u8);
    fn GXSetCullMode(a0: u32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetNumChans(a0: u8);
    fn GXSetNumTevStages(a0: u8);
    fn GXSetNumTexGens(a0: u8);
    fn GXSetTevOp(a0: u32, a1: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetTexCoordGen2(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn GXSetZCompLoc(a0: u8);
    fn GXSetZMode(a0: u8, a1: u32, a2: u8);
    fn GXSetZTexture(a0: u32, a1: u32, a2: u32);
    fn GXTexCoord2u8(a0: u8, a1: u8);
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj;
    fn HSD_ClearVtxDesc();
    fn HSD_DObjSetCurrent(a0: *mut HSD_DObj);
    fn HSD_JObjSetCurrent(a0: *mut HSD_JObj);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_LObjSetupSpecularInit(a0: *mut c_void);
    fn HSD_MtxAlloc() -> *mut c_void;
    fn HSD_MtxFree(a0: *mut c_void);
    fn HSD_MtxInverseConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void;
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32);
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void);
    fn HSD_PObjClearMtxMark(a0: *mut c_void, a1: u32);
    fn HSD_SetupRenderMode(a0: u32);
    fn HSD_StateInitTev();
    fn HSD_StateInvalidate(a0: i32);
    fn MTXRotRad(a0: *mut c_void, a1: i8, a2: f32);
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXInverse(a0: *mut c_void, a1: *mut c_void) -> u32;
    fn PSMTXScale(a0: *mut c_void, a1: f32, a2: f32, a3: f32);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec);
    fn PSVECMag(a0: *mut Vec) -> f32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_identityMtx: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;

const JOBJ_SKELETON: u32 = 1 << 0;
const JOBJ_SKELETON_ROOT: u32 = 1 << 1;
const JOBJ_HIDDEN: u32 = 1 << 4;
const JOBJ_INSTANCE: u32 = 1 << 12;
const JOBJ_SPECULAR: u32 = 1 << 16;
const JOBJ_PTCL: u32 = 1 << 5;
const JOBJ_SPLINE: u32 = 1 << 14;
const JOBJ_BILLBOARD_FIELD: u32 = 0xE00;
const JOBJ_BILLBOARD: u32 = 0x200;
const JOBJ_VBILLBOARD: u32 = 0x400;
const JOBJ_HBILLBOARD: u32 = 0x600;
const JOBJ_RBILLBOARD: u32 = 0x800;
const JOBJ_PBILLBOARD: u32 = 0x2000;
const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
const JOBJ_TRSP_SHIFT: u32 = 18;
const JOBJ_OPA: u32 = 0x40000;
const JOBJ_XLU: u32 = 0x80000;
const JOBJ_TEXEDGE: u32 = 0x100000;
const DOBJ_HIDDEN: u32 = 1;
const DOBJ_TRSP_SHIFT: u32 = 1;
const HSD_TRSP_OPA: u32 = 1;
const HSD_TRSP_XLU: u32 = 2;
const HSD_TRSP_TEXEDGE: u32 = 4;
const RENDER_SHADOW: u32 = 1 << 26;
const JOBJ_PTCL_ACTIVE: u32 = 0x7FFF_FFFF;
const JOBJ_PTCL_OFFSET_MASK: u32 = 0xFF_FFFF;
const JOBJ_PTCL_OFFSET_SHIFT: u32 = 6;
const JOBJ_PTCL_BANK_MASK: u32 = 0x3F;

/// The TU-local `HSD_ZList`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HSD_ZList_sort {
    pub texedge: *mut HSD_ZList,
    pub xlu: *mut HSD_ZList,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HSD_ZList {
    pub pmtx: [[f32; 4]; 3],
    pub vmtx: *mut [f32; 4],
    pub jobj: *mut HSD_JObj,
    pub rendermode: u32,
    pub sort: HSD_ZList_sort,
    pub next: *mut HSD_ZList,
}
const ZL_TEXEDGE: usize = offset_of!(HSD_ZList, sort) + offset_of!(HSD_ZList_sort, texedge);
const ZL_XLU: usize = offset_of!(HSD_ZList, sort) + offset_of!(HSD_ZList_sort, xlu);

pub static mut zlist_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut sptcl_callback: Option<unsafe extern "C" fn(i32, i32, i32, *mut HSD_JObj)> = None;
pub static mut erase_color: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };
pub static mut zsort_listing: i32 = 0;
pub static mut zsort_sorting: i32 = 0;
pub static mut zlist_top: *mut HSD_ZList = null_mut();
pub static mut zlist_bottom: *mut *mut HSD_ZList = unsafe { addr_of_mut!(zlist_top) };
pub static mut zlist_texedge_top: *mut HSD_ZList = null_mut();
pub static mut zlist_texedge_bottom: *mut *mut HSD_ZList = unsafe { addr_of_mut!(zlist_texedge_top) };
pub static mut zlist_texedge_nb: i32 = 0;
pub static mut zlist_xlu_top: *mut HSD_ZList = null_mut();
pub static mut zlist_xlu_bottom: *mut *mut HSD_ZList = unsafe { addr_of_mut!(zlist_xlu_top) };
pub static mut zlist_xlu_nb: i32 = 0;
pub static mut zOne: Vec = Vec { x: 0., y: 0., z: 1. };
pub static mut yOne: Vec = Vec { x: 0., y: 1., z: 0. };
pub static mut zOne2: Vec = Vec { x: 0., y: 0., z: 1. };

// ---- header inlines (mtx.h, jobj.h, cobj.h) ----
unsafe fn col_vec(m: *const [f32; 4], col: usize, v: *mut Vec) {
    (*v).x = (*m.add(0))[col];
    (*v).y = (*m.add(1))[col];
    (*v).z = (*m.add(2))[col];
}
unsafe fn set_col_vec(m: *mut [f32; 4], col: usize, v: *const Vec) {
    (*m.add(0))[col] = (*v).x;
    (*m.add(1))[col] = (*v).y;
    (*m.add(2))[col] = (*v).z;
}
unsafe fn col_mag(m: *const [f32; 4], col: usize) -> f32 {
    let (a, b, c) = ((*m.add(0))[col], (*m.add(1))[col], (*m.add(2))[col]);
    crate::gekko::sqrtf(fmadds(c, c, fmadds(a, a, b * b)))
}
/// jobj.h `HSD_JObjSetupMatrix` (inline): rebuild when dirty and not user-defined.
unsafe fn jobj_setup_matrix(j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    let fl = (*j).flags;
    if fl & JOBJ_USER_DEF_MTX == 0 && fl & (JOBJ_MTX_DIRTY_BIT) != 0 {
        ext::HSD_JObjSetupMatrixSub(j);
    }
}
const JOBJ_MTX_DIRTY_BIT: u32 = 1 << 6;
unsafe fn jobj_info(j: *mut HSD_JObj) -> *mut HSD_JObjInfo {
    (*j).object.parent.class_info.cast()
}

unsafe fn zlist_next(list: *mut HSD_ZList, offset: usize) -> *mut *mut HSD_ZList {
    (list as *mut u8).add(offset).cast()
}

/// `HSD_ZListInitAllocData`.
pub unsafe fn HSD_ZListInitAllocData() {
    ext::HSD_ObjAllocInit(addr_of_mut!(zlist_alloc_data), size_of::<HSD_ZList>() as u32, 4);
}

unsafe fn HSD_ZListAlloc() -> *mut HSD_ZList {
    let list = ext::HSD_ObjAlloc(addr_of_mut!(zlist_alloc_data)).cast::<HSD_ZList>();
    core::ptr::write_bytes(addr_of_mut!((*list).vmtx).cast::<u8>(), 0, size_of::<HSD_ZList>() - size_of::<[[f32; 4]; 3]>());
    list
}

unsafe fn HSD_ZListFree(ptr: *mut HSD_ZList) {
    ext::HSD_ObjFree(addr_of_mut!(zlist_alloc_data), ptr.cast());
}

/// `HSD_StateInitDirect`.
pub unsafe fn HSD_StateInitDirect(vtxfmt: i32, rendermode: u32) {
    ext::HSD_ClearVtxDesc();
    ext::GXSetVtxAttrFmt(vtxfmt as u32, 9, 1, 4, 0);
    ext::GXSetVtxAttrFmt(vtxfmt as u32, 11, 1, 5, 0);
    ext::HSD_StateInitTev();
    ext::HSD_SetupRenderMode(rendermode | 0x2800_0000);
    ext::GXSetVtxDesc(9, 1);
    ext::GXSetVtxDesc(11, 1);
    ext::GXSetCurrentMtx(0);
}

unsafe fn mkVBillBoardMtx(jobj: *mut HSD_JObj, src: *mut [f32; 4], dst: *mut [f32; 4]) {
    let (mut pos, mut ax, mut ay, mut az) = (Vec::zeroed(), Vec::zeroed(), Vec::zeroed(), Vec::zeroed());
    col_vec(src, 3, &mut pos);
    col_vec(src, 1, &mut ay);
    let mut sx = col_mag(src, 0);
    let mut sz = col_mag(src, 2);
    if (*jobj).flags & JOBJ_PBILLBOARD != 0 {
        ext::PSVECCrossProduct(&mut pos, &mut ay, &mut ax);
    } else {
        ext::PSVECCrossProduct(&mut ay, addr_of_mut!(zOne), &mut ax);
    }
    ext::PSVECCrossProduct(&mut ax, &mut ay, &mut az);
    sx /= ext::PSVECMag(&mut ax);
    sz /= ext::PSVECMag(&mut az);
    (*dst.add(0))[0] = sx * ax.x;
    (*dst.add(1))[0] = sx * ax.y;
    (*dst.add(2))[0] = sx * ax.z;
    set_col_vec(dst, 1, &ay);
    (*dst.add(0))[2] = sz * az.x;
    (*dst.add(1))[2] = sz * az.y;
    (*dst.add(2))[2] = sz * az.z;
    set_col_vec(dst, 3, &pos);
}

unsafe fn mkHBillBoardMtx(jobj: *mut HSD_JObj, src: *mut [f32; 4], dst: *mut [f32; 4]) {
    let (mut pos, mut ax, mut ay, mut az, mut uy) = (Vec::zeroed(), Vec::zeroed(), Vec::zeroed(), Vec::zeroed(), Vec::zeroed());
    col_vec(src, 3, &mut pos);
    col_vec(src, 0, &mut ax);
    let mut sy = col_mag(src, 1);
    let mut sz = col_mag(src, 2);
    if (*jobj).flags & JOBJ_PBILLBOARD != 0 {
        uy.y = crate::gekko::sqrtf(fmadds(pos.x, pos.x, pos.z * pos.z));
        uy.x = -pos.y / uy.y * pos.x;
        uy.z = -pos.y / uy.y * pos.z;
        ext::PSVECCrossProduct(&mut ax, &mut uy, &mut az);
    } else {
        ext::PSVECCrossProduct(&mut ax, addr_of_mut!(yOne), &mut az);
    }
    ext::PSVECCrossProduct(&mut az, &mut ax, &mut ay);
    sy /= ext::PSVECMag(&mut ay);
    sz /= ext::PSVECMag(&mut az);
    set_col_vec(dst, 0, &ax);
    (*dst.add(0))[1] = sy * ay.x;
    (*dst.add(1))[1] = sy * ay.y;
    (*dst.add(2))[1] = sy * ay.z;
    (*dst.add(0))[2] = sz * az.x;
    (*dst.add(1))[2] = sz * az.y;
    (*dst.add(2))[2] = sz * az.z;
    set_col_vec(dst, 3, &pos);
}

unsafe fn mkBillBoardMtx(jobj: *mut HSD_JObj, src: *mut [f32; 4], dst: *mut [f32; 4]) {
    let (mut ax, mut ay, mut pos) = (Vec::zeroed(), Vec::zeroed(), Vec::zeroed());
    let az: *mut Vec;
    let mut sx = col_mag(src, 0);
    let mut sz = col_mag(src, 2);
    col_vec(src, 1, &mut ay);
    let mut sy = ext::PSVECMag(&mut ay);
    col_vec(src, 3, &mut pos);
    if (*jobj).flags & JOBJ_PBILLBOARD != 0 {
        ext::PSVECCrossProduct(&mut pos, &mut ay, &mut ax);
        ext::PSVECCrossProduct(&mut ax, &mut pos, &mut ay);
        az = &mut pos;
        sz /= -ext::PSVECMag(az);
    } else {
        ext::PSVECCrossProduct(&mut ay, addr_of_mut!(zOne2), &mut ax);
        ext::PSVECCrossProduct(addr_of_mut!(zOne2), &mut ax, &mut ay);
        az = addr_of_mut!(zOne2);
        sz /= ext::PSVECMag(addr_of_mut!(zOne2));
    }
    sx /= ext::PSVECMag(&mut ax);
    sy /= ext::PSVECMag(&mut ay);
    (*dst.add(0))[0] = sx * ax.x;
    (*dst.add(1))[0] = sx * ax.y;
    (*dst.add(2))[0] = sx * ax.z;
    (*dst.add(0))[1] = sy * ay.x;
    (*dst.add(1))[1] = sy * ay.y;
    (*dst.add(2))[1] = sy * ay.z;
    (*dst.add(0))[2] = sz * (*az).x;
    (*dst.add(1))[2] = sz * (*az).y;
    (*dst.add(2))[2] = sz * (*az).z;
    set_col_vec(dst, 3, &pos);
}

unsafe fn mkRBillBoardMtx(jobj: *mut HSD_JObj, src: *mut [f32; 4], dst: *mut [f32; 4]) {
    let mut rot = [[0.0f32; 4]; 3];
    let mut scl = [[0.0f32; 4]; 3];
    let sx = col_mag(src, 0);
    let sy = col_mag(src, 1);
    let sz = col_mag(src, 2);
    ext::PSMTXScale(scl.as_mut_ptr().cast(), sx, sy, sz);
    ext::MTXRotRad(rot.as_mut_ptr().cast(), b'z' as i8, (*jobj).rotate.z);
    rot[0][3] = (*src.add(0))[3];
    rot[1][3] = (*src.add(1))[3];
    rot[2][3] = (*src.add(2))[3];
    ext::PSMTXConcat(rot.as_mut_ptr().cast(), scl.as_mut_ptr().cast(), dst.cast());
}

/// `HSD_JObjMakePositionMtx`.
pub unsafe fn HSD_JObjMakePositionMtx(jobj: *mut HSD_JObj, vmtx: *mut [f32; 4], pmtx: *mut [f32; 4]) {
    let mut mtx = [[0.0f32; 4]; 3];
    if (*jobj).flags & JOBJ_BILLBOARD_FIELD != 0 {
        ext::PSMTXConcat(vmtx.cast(), (*jobj).mtx.as_mut_ptr().cast(), mtx.as_mut_ptr().cast());
        match (*jobj).flags & JOBJ_BILLBOARD_FIELD {
            JOBJ_BILLBOARD => mkBillBoardMtx(jobj, mtx.as_mut_ptr(), pmtx),
            JOBJ_VBILLBOARD => mkVBillBoardMtx(jobj, mtx.as_mut_ptr(), pmtx),
            JOBJ_HBILLBOARD => mkHBillBoardMtx(jobj, mtx.as_mut_ptr(), pmtx),
            JOBJ_RBILLBOARD => mkRBillBoardMtx(jobj, mtx.as_mut_ptr(), pmtx),
            _ => panic!("displayfunc.c:322 unkown type of billboard."),
        }
    } else {
        ext::PSMTXConcat(vmtx.cast(), (*jobj).mtx.as_mut_ptr().cast(), pmtx.cast());
    }
}

/// `HSD_JObjFindSkeleton`.
pub unsafe fn HSD_JObjFindSkeleton(mut jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    assert!(!jobj.is_null(), "displayfunc.c:342 jobj");
    while !jobj.is_null() {
        if (*jobj).flags & (JOBJ_SKELETON | JOBJ_SKELETON_ROOT) != 0 {
            return jobj;
        }
        jobj = (*jobj).parent;
    }
    null_mut()
}

/// `_HSD_mkEnvelopeModelNodeMtx`.
pub unsafe fn _HSD_mkEnvelopeModelNodeMtx(m: *mut HSD_JObj, mtx: *mut [f32; 4]) -> *mut [f32; 4] {
    if (*m).flags & JOBJ_SKELETON_ROOT != 0 {
        null_mut()
    } else {
        let x = HSD_JObjFindSkeleton(m);
        assert!(!x.is_null(), "displayfunc.c:422 x");
        if x == m {
            ext::PSMTXInverse((*x).envelopemtx.cast(), mtx.cast());
        } else if (*x).flags & JOBJ_SKELETON_ROOT != 0 {
            ext::HSD_MtxInverseConcat((*x).mtx.as_mut_ptr().cast(), (*m).mtx.as_mut_ptr().cast(), mtx.cast());
        } else {
            let mut n = [[0.0f32; 4]; 3];
            ext::PSMTXConcat((*x).mtx.as_mut_ptr().cast(), (*x).envelopemtx.cast(), n.as_mut_ptr().cast());
            ext::HSD_MtxInverseConcat(n.as_mut_ptr().cast(), (*m).mtx.as_mut_ptr().cast(), mtx.cast());
        }
        mtx
    }
}

/// `HSD_JObjDispSub`.
pub unsafe extern "C" fn HSD_JObjDispSub(jobj: *mut HSD_JObj, vmtx: *mut [f32; 4], pmtx: *mut [f32; 4], trsp_mask: u32, rendermode: u32) {
    ext::HSD_JObjSetCurrent(jobj);
    let dobj_trsp = trsp_mask << DOBJ_TRSP_SHIFT;
    if rendermode & RENDER_SHADOW == 0 && (*jobj).flags & JOBJ_SPECULAR != 0 {
        ext::HSD_LObjSetupSpecularInit(pmtx.cast());
    }
    ext::HSD_PObjClearMtxMark(null_mut(), 0);
    let mut dobj = (*jobj).u.dobj;
    while !dobj.is_null() {
        if (*dobj).flags & DOBJ_HIDDEN == 0 && (*dobj).flags & dobj_trsp != 0 {
            ext::HSD_DObjSetCurrent(dobj);
            let info = (*dobj).parent.class_info.cast::<HSD_DObjInfo>();
            ((*info).disp.expect("dobj disp"))(dobj, vmtx, pmtx, rendermode);
        }
        dobj = (*dobj).next;
    }
    ext::HSD_DObjSetCurrent(null_mut());
    ext::HSD_JObjSetCurrent(null_mut());
}

/// `HSD_JObjDispDObj`.
pub unsafe fn HSD_JObjDispDObj(jobj: *mut HSD_JObj, mut vmtx: *mut [f32; 4], trsp_mask: u32, rendermode: u32) {
    let mut mtx = [[0.0f32; 4]; 3];
    if (*jobj).flags & JOBJ_HIDDEN == 0 {
        let xlu_bits = (*jobj).flags & (trsp_mask << JOBJ_TRSP_SHIFT);
        if xlu_bits != 0 {
            jobj_setup_matrix(jobj);
            if vmtx.is_null() {
                let cobj = ext::HSD_CObjGetCurrent();
                vmtx = (*cobj).view_mtx.as_mut_ptr();
            }
            let info = jobj_info(jobj);
            ((*info).make_pmtx.expect("make_pmtx"))(jobj, vmtx, mtx.as_mut_ptr());
            if xlu_bits & JOBJ_OPA != 0 {
                ((*info).disp.expect("jobj disp"))(jobj, vmtx, mtx.as_mut_ptr(), HSD_TRSP_OPA, rendermode);
            }
            if zsort_listing == 0 {
                if xlu_bits & JOBJ_TEXEDGE != 0 {
                    ((*info).disp.expect("jobj disp"))(jobj, vmtx, mtx.as_mut_ptr(), HSD_TRSP_TEXEDGE, rendermode);
                }
                if xlu_bits & JOBJ_XLU != 0 {
                    ((*info).disp.expect("jobj disp"))(jobj, vmtx, mtx.as_mut_ptr(), HSD_TRSP_XLU, rendermode);
                }
            } else if xlu_bits & (JOBJ_TEXEDGE | JOBJ_XLU) != 0 {
                let zlist = HSD_ZListAlloc();
                ext::PSMTXCopy(mtx.as_mut_ptr().cast(), (*zlist).pmtx.as_mut_ptr().cast());
                if !vmtx.is_null() {
                    (*zlist).vmtx = ext::HSD_MtxAlloc().cast();
                    ext::PSMTXCopy(vmtx.cast(), (*zlist).vmtx.cast());
                }
                (*zlist).jobj = jobj;
                (*zlist).rendermode = rendermode;
                *zlist_bottom = zlist;
                zlist_bottom = addr_of_mut!((*zlist).next);
                if xlu_bits & JOBJ_TEXEDGE != 0 {
                    *zlist_texedge_bottom = zlist;
                    zlist_texedge_bottom = addr_of_mut!((*zlist).sort.texedge);
                    zlist_texedge_nb += 1;
                }
                if xlu_bits & JOBJ_XLU != 0 {
                    *zlist_xlu_bottom = zlist;
                    zlist_xlu_bottom = addr_of_mut!((*zlist).sort.xlu);
                    zlist_xlu_nb += 1;
                }
            }
        }
    }
}

unsafe fn zlist_sort(mut list: *mut HSD_ZList, nb: i32, offset: usize) -> *mut HSD_ZList {
    if nb <= 1 {
        if !list.is_null() {
            *zlist_next(list, offset) = null_mut();
        }
        return list;
    }
    let nb_fore = nb / 2;
    let nb_hind = nb - nb_fore;
    let mut hind = list;
    for _ in 0..nb_fore {
        hind = *zlist_next(hind, offset);
    }
    let mut fore = zlist_sort(list, nb_fore, offset);
    hind = zlist_sort(hind, nb_hind, offset);
    list = null_mut();
    let mut ptr: *mut *mut HSD_ZList = &mut list;
    while !fore.is_null() && !hind.is_null() {
        if (*fore).pmtx[2][3] <= (*hind).pmtx[2][3] {
            *ptr = fore;
            fore = *zlist_next(fore, offset);
        } else {
            *ptr = hind;
            hind = *zlist_next(hind, offset);
        }
        ptr = zlist_next(*ptr, offset);
    }
    if !fore.is_null() {
        *ptr = fore;
    } else if !hind.is_null() {
        *ptr = hind;
    }
    list
}

/// `_HSD_ZListSort`.
pub unsafe fn _HSD_ZListSort() {
    if zsort_sorting != 0 {
        zlist_texedge_top = zlist_sort(zlist_texedge_top, zlist_texedge_nb, ZL_TEXEDGE);
        zlist_xlu_top = zlist_sort(zlist_xlu_top, zlist_xlu_nb, ZL_XLU);
    }
}

/// `_HSD_ZListDisp`.
pub unsafe fn _HSD_ZListDisp() {
    let cobj = ext::HSD_CObjGetCurrent();
    let vmtx = (*cobj).view_mtx.as_mut_ptr();
    let mut list = zlist_texedge_top;
    while !list.is_null() {
        let info = jobj_info((*list).jobj);
        ((*info).disp.expect("jobj disp"))((*list).jobj, if !(*list).vmtx.is_null() { (*list).vmtx } else { vmtx }, (*list).pmtx.as_mut_ptr(), HSD_TRSP_TEXEDGE, (*list).rendermode);
        list = (*list).sort.texedge;
    }
    list = zlist_xlu_top;
    while !list.is_null() {
        let info = jobj_info((*list).jobj);
        ((*info).disp.expect("jobj disp"))((*list).jobj, if !(*list).vmtx.is_null() { (*list).vmtx } else { vmtx }, (*list).pmtx.as_mut_ptr(), HSD_TRSP_XLU, (*list).rendermode);
        list = (*list).sort.xlu;
    }
    _HSD_ZListClear();
}

/// `_HSD_ZListClear`.
pub unsafe fn _HSD_ZListClear() {
    let mut list = zlist_top;
    while !list.is_null() {
        let next = (*list).next;
        if !(*list).vmtx.is_null() {
            ext::HSD_MtxFree((*list).vmtx.cast());
        }
        HSD_ZListFree(list);
        list = next;
    }
    zlist_top = null_mut();
    zlist_bottom = addr_of_mut!(zlist_top);
    zlist_texedge_top = null_mut();
    zlist_texedge_bottom = addr_of_mut!(zlist_texedge_top);
    zlist_texedge_nb = 0;
    zlist_xlu_top = null_mut();
    zlist_xlu_bottom = addr_of_mut!(zlist_xlu_top);
    zlist_xlu_nb = 0;
}

/// `HSD_JObjDisp`.
pub unsafe fn HSD_JObjDisp(jobj: *mut HSD_JObj, vmtx: *mut [f32; 4], trsp_mask: u32, rendermode: u32) {
    if !jobj.is_null() {
        if (*jobj).flags & (JOBJ_PTCL | JOBJ_SPLINE) == 0 {
            HSD_JObjDispDObj(jobj, vmtx, trsp_mask, rendermode);
        } else if (*jobj).flags & JOBJ_PTCL != 0 && sptcl_callback.is_some() {
            let mut sp = (*jobj).u.ptcl;
            while !sp.is_null() {
                let data = (*sp).data as usize as u32;
                if data & 0x8000_0000 != 0 {
                    let bank = JOBJ_PTCL_BANK_MASK & data;
                    let offset = (data >> JOBJ_PTCL_OFFSET_SHIFT) & JOBJ_PTCL_OFFSET_MASK;
                    (sptcl_callback.unwrap())(0, bank as i32, offset as i32, jobj);
                }
                (*sp).data = (data & JOBJ_PTCL_ACTIVE) as usize as *mut c_void;
                sp = (*sp).next;
            }
        }
    }
}

/// `HSD_JObjSetSPtclCallback`.
pub unsafe fn HSD_JObjSetSPtclCallback(func: Option<unsafe extern "C" fn(i32, i32, i32, *mut HSD_JObj)>) {
    sptcl_callback = func;
}

/// `HSD_SetEraseColor`.
pub unsafe fn HSD_SetEraseColor(r: u8, g: u8, b: u8, a: u8) {
    erase_color.r = r;
    erase_color.g = g;
    erase_color.b = b;
    erase_color.a = a;
}

#[repr(C, align(32))]
struct DepthImage([u8; 107]);
static mut depth_image: DepthImage = DepthImage([255; 107]);

/// `HSD_EraseRect`.
pub unsafe fn HSD_EraseRect(top: f32, bottom: f32, left: f32, right: f32, z: f32, enable_color: i32, enable_alpha: i32, enable_depth: i32) {
    let mut texobj: GXTexObj = core::mem::zeroed();
    if enable_color == 0 && enable_alpha == 0 && enable_depth == 0 {
        return;
    }
    if enable_depth != 0 {
        ext::GXInitTexObj(&mut texobj, addr_of_mut!(depth_image).cast(), 4, 4, 0x11, 1, 1, 0);
        ext::GXLoadTexObj(&mut texobj, 0);
        ext::GXSetNumTexGens(1);
        ext::GXSetTexCoordGen2(0, 1, 4, 60, 0, 125);
        ext::GXSetNumTevStages(1);
        ext::GXSetTevOrder(0, 0, 0, 4);
        ext::GXSetTevOp(0, 4);
        ext::GXSetZTexture(2, 0x11, 0);
    } else {
        ext::GXSetNumTexGens(0);
        ext::GXSetNumTevStages(1);
        ext::GXSetTevOrder(0, 0xFF, 0xFF, 4);
        ext::GXSetTevOp(0, 4);
    }
    ext::GXSetCullMode(0);
    ext::GXSetAlphaCompare(7, 0, 1, 7, 0);
    ext::GXSetZCompLoc(1);
    ext::GXSetZMode(1, 7, if enable_depth != 0 { 1 } else { 0 });
    ext::GXSetBlendMode(2, 1, 0, 3);
    ext::GXSetColorUpdate(if enable_color != 0 { 1 } else { 0 });
    ext::GXSetAlphaUpdate(if enable_alpha != 0 { 1 } else { 0 });
    ext::GXSetNumChans(1);
    ext::GXSetChanCtrl(4, 0, 0, 1, 0, 0, 2);
    ext::GXClearVtxDesc();
    ext::GXSetVtxAttrFmt(0, 9, 1, 4, 0);
    ext::GXSetVtxAttrFmt(0, 11, 1, 5, 0);
    ext::GXSetVtxAttrFmt(0, 13, 1, 0, 0);
    ext::GXLoadPosMtxImm(identity_mtx().cast(), 0);
    ext::GXSetCurrentMtx(0);
    ext::GXSetVtxDesc(9, 1);
    ext::GXSetVtxDesc(11, 1);
    ext::GXSetVtxDesc(13, 1);
    let color = erase_color;
    ext::GXBegin(0x80, 0, 4);
    ext::GXPosition3f32(left, top, z);
    ext::GXColor4u8(color.r, color.g, color.b, color.a);
    ext::GXTexCoord2u8(0, 0);
    ext::GXPosition3f32(right, top, z);
    ext::GXColor4u8(color.r, color.g, color.b, color.a);
    ext::GXTexCoord2u8(1, 0);
    ext::GXPosition3f32(right, bottom, z);
    ext::GXColor4u8(color.r, color.g, color.b, color.a);
    ext::GXTexCoord2u8(1, 1);
    ext::GXPosition3f32(left, bottom, z);
    ext::GXColor4u8(color.r, color.g, color.b, color.a);
    ext::GXTexCoord2u8(0, 1);
    ext::GXSetZTexture(0, 0x11, 0);
    ext::HSD_StateInvalidate(-1);
}
/// util.c `HSD_identityMtx` (read-only here).
pub static mut HSD_identityMtx_local: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
unsafe fn identity_mtx() -> *mut [f32; 4] {
    if use_ext_data { addr_of_mut!(ext::HSD_identityMtx).cast() } else { addr_of_mut!(HSD_identityMtx_local).cast() }
}

/// `_HSD_DispForgetMemory`.
pub unsafe fn _HSD_DispForgetMemory(_lo: *mut c_void, _hi: *mut c_void) {
    zlist_top = null_mut();
    zlist_bottom = addr_of_mut!(zlist_top);
    zlist_texedge_top = null_mut();
    zlist_texedge_bottom = addr_of_mut!(zlist_texedge_top);
    zlist_texedge_nb = 0;
    zlist_xlu_top = null_mut();
    zlist_xlu_bottom = addr_of_mut!(zlist_xlu_top);
    zlist_xlu_nb = 0;
}
