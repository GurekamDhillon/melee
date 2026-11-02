//! Native shadow objects; math delegates to existing oracle-verified ports.
#![allow(non_snake_case, non_upper_case_globals)]
use core::{
    ffi::c_void,
    ptr::{addr_of_mut, null_mut},
};
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
    fn GXCopyTex(a0: *mut c_void, a1: u8);
    fn GXGetTexBufferSize(a0: u16, a1: u16, a2: u32, a3: u8, a4: u8) -> u32;
    fn GXInvalidateTexAll();
    fn GXLoadPosMtxImm(a0: *mut c_void, a1: u32);
    fn GXPixModeSync();
    fn GXPosition2f32(a0: f32, a1: f32);
    fn GXSetCurrentMtx(a0: u32);
    fn GXSetScissor(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetTexCopyDst(a0: u16, a1: u16, a2: u32, a3: u8);
    fn GXSetTexCopySrc(a0: u16, a1: u16, a2: u16, a3: u16);
    fn GXSetVtxAttrFmt(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8);
    fn GXSetVtxDesc(a0: u32, a1: u32);
    fn HSD_CObjAlloc() -> *mut HSD_CObj;
    fn HSD_CObjEndCurrent();
    fn HSD_CObjGetBottom(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetEyeDistance(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetLeft(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetNear(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetProjectionType(a0: *mut HSD_CObj) -> i32;
    fn HSD_CObjGetRight(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjGetTop(a0: *mut HSD_CObj) -> f32;
    fn HSD_CObjSetAspect(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32;
    fn HSD_CObjSetFov(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetFrustum(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32);
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32);
    fn HSD_CObjSetScissorx4(a0: *mut HSD_CObj, a1: u16, a2: u16, a3: u16, a4: u16);
    fn HSD_CObjSetViewportfx4(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32);
    fn HSD_ClearVtxDesc();
    fn HSD_Free(a0: *mut c_void);
    fn HSD_ImageDescAlloc() -> *mut HSD_ImageDesc;
    fn HSD_ImageDescFree(a0: *mut HSD_ImageDesc);
    fn HSD_JObjDispAll(a0: *mut HSD_JObj, a1: *mut c_void, a2: u32, a3: u32);
    fn HSD_JObjUnref(a0: *mut HSD_JObj);
    fn HSD_MObjAddShadowTexture(a0: *mut HSD_TObj);
    fn HSD_MObjDeleteShadowTexture(a0: *mut HSD_TObj);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void;
    fn HSD_ObjAlloc(a0: *mut _HSD_ObjAllocData) -> *mut c_void;
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32);
    fn HSD_ObjFree(a0: *mut _HSD_ObjAllocData, a1: *mut c_void);
    fn HSD_SListAllocAndPrepend(a0: *mut _HSD_SList, a1: *mut c_void) -> *mut _HSD_SList;
    fn HSD_SListRemove(a0: *mut _HSD_SList) -> *mut _HSD_SList;
    fn HSD_SetupChannelAll(a0: *mut HSD_Chan);
    fn HSD_SetupPEMode(a0: u32, a1: *mut HSD_PEDesc);
    fn HSD_SetupTevStageAll(a0: *mut _HSD_TevDesc);
    fn HSD_StateInitTev();
    fn HSD_StateSetCullMode(a0: i32);
    fn HSD_StateSetNumTexGens();
    fn HSD_TObjAlloc() -> *mut HSD_TObj;
    fn HSD_TObjFree(a0: *mut HSD_TObj);
    fn MTXLightFrustum(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32, a9: f32) = { let r = crate::mtx::light_frustum(a1, a2, a3, a4, a5, a6, a7, a8, a9); (a0 as *mut [[f32; 4]; 3]).write(r); };
    fn MTXLightOrtho(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) = { let r = crate::mtx::light_ortho(a1, a2, a3, a4, a5, a6, a7, a8); (a0 as *mut [[f32; 4]; 3]).write(r); };
    fn MTXLightPerspective(a0: *mut c_void, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32) = { let r = crate::mtx::light_perspective(a1, a2, a3, a4, a5, a6); (a0 as *mut [[f32; 4]; 3]).write(r); };
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::mtx_pc::raw::PSMTXConcat(a0 as _, a1 as _, a2 as _);
    fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = { let r = crate::mtx::vec_cross(&crate::mtx::Vec3 { x: (*a0).x, y: (*a0).y, z: (*a0).z }, &crate::mtx::Vec3 { x: (*a1).x, y: (*a1).y, z: (*a1).z }); *a2 = Vec { x: r.x, y: r.y, z: r.z }; };
    fn PSVECDotProduct(a0: *mut Vec, a1: *mut Vec) -> f32 = crate::mtx::vec_dot(&crate::mtx::Vec3 { x: (*a0).x, y: (*a0).y, z: (*a0).z }, &crate::mtx::Vec3 { x: (*a1).x, y: (*a1).y, z: (*a1).z });
    fn PSVECMag(a0: *mut Vec) -> f32 = crate::mtx::vec_mag(&crate::mtx::Vec3 { x: (*a0).x, y: (*a0).y, z: (*a0).z });
    fn PSVECNormalize(a0: *mut Vec, a1: *mut Vec) = { let r = crate::mtx::vec_normalize(&crate::mtx::Vec3 { x: (*a0).x, y: (*a0).y, z: (*a0).z }); *a1 = Vec { x: r.x, y: r.y, z: r.z }; };
    fn PSVECSubtract(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) = { let r = crate::mtx::vec_sub(&crate::mtx::Vec3 { x: (*a0).x, y: (*a0).y, z: (*a0).z }, &crate::mtx::Vec3 { x: (*a1).x, y: (*a1).y, z: (*a1).z }); *a2 = Vec { x: r.x, y: r.y, z: r.z }; };
    fn atan2f(a0: f32, a1: f32) -> f32 = crate::lbtrigf::atan2f(a0, a1);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::perf::HSD_PerfCurrentStat;
    pub static mut HSD_identityMtx: [[f32; 4]; 3] = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;
/// util.c `HSD_identityMtx` (read-only here).
pub static mut HSD_identityMtx_local: [[f32; 4]; 3] = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
unsafe fn identity_mtx() -> *mut c_void {
    if use_ext_data { addr_of_mut!(ext::HSD_identityMtx).cast() } else { addr_of_mut!(HSD_identityMtx_local).cast() }
}
/// perf.h `HSD_PerfCountMtxLoad` (static inline).
unsafe fn perf_count_mtx_load() {
    let st = if use_ext_data { addr_of_mut!(ext::HSD_PerfCurrentStat) } else { addr_of_mut!(crate::tu::perf::HSD_PerfCurrentStat) };
    (*st).nb_mtx_load = (*st).nb_mtx_load.wrapping_add(1);
}
/// class.h `hsdDelete` (static inline).
unsafe fn hsd_delete(o: *mut c_void) {
    if o.is_null() {
        return;
    }
    let i = (*o.cast::<HSD_Class>()).class_info;
    ((*i).release.expect("hsdDelete: release"))(o.cast());
    ((*i).destroy.expect("hsdDelete: destroy"))(o.cast());
}
/// jobj.h `HSD_JObjRef` (inline `ref_INC`).
unsafe fn jobj_ref(j: *mut HSD_JObj) {
    if !j.is_null() {
        (*j).object.ref_count = (*j).object.ref_count.wrapping_add(1);
        assert_ne!((*j).object.ref_count, 65535);
    }
}
/// cobj.h `HSD_CObjGetViewingMtxPtrDirect` (inline).
unsafe fn cobj_view_mtx(c: *mut HSD_CObj) -> *mut Mtx {
    (*c).view_mtx.as_mut_ptr().cast()
}
pub static mut shadow_alloc_data: HSD_ObjAllocData =
    unsafe { core::mem::zeroed() };
pub unsafe fn HSD_ShadowGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_ShadowGetAllocData");
    addr_of_mut!(shadow_alloc_data)
}
pub unsafe fn HSD_ShadowInitAllocData() {
    melee_core::ct!("HSD_ShadowInitAllocData");
    ext::HSD_ObjAllocInit(
        HSD_ShadowGetAllocData(),
        core::mem::size_of::<HSD_Shadow>() as u32,
        4,
    );
}
pub unsafe fn makeShadowTObj() -> *mut HSD_TObj {
    melee_core::ct!("makeShadowTObj");
    let t = ext::HSD_TObjAlloc();
    (*t).src = 0;
    (*t).wrap_s = 0;
    (*t).wrap_t = 0;
    (*t).flags = 0x540103;
    (*t).imagedesc = ext::HSD_ImageDescAlloc();
    t
}
pub unsafe fn HSD_ShadowAlloc() -> *mut HSD_Shadow {
    melee_core::ct!("HSD_ShadowAlloc");
    let s = ext::HSD_ObjAlloc(HSD_ShadowGetAllocData()).cast::<HSD_Shadow>();
    s.write(HSD_Shadow::zeroed());
    (*s).camera = ext::HSD_CObjAlloc();
    (*s).texture = makeShadowTObj();
    (*s).scaleS = 0.5;
    (*s).scaleT = -0.5;
    (*s).transS = 0.5;
    (*s).transT = 0.5;
    let d = (*(*s).texture).imagedesc;
    (*d).format = 0;
    (*d).width = 256;
    (*d).height = 256;
    ext::HSD_CObjSetViewportfx4((*s).camera, 0.0, 256.0, 0.0, 256.0);
    ext::HSD_CObjSetScissorx4((*s).camera, 0, 256, 0, 256);
    s
}
pub unsafe fn HSD_ShadowRemove(s: *mut HSD_Shadow) {
    melee_core::ct!("HSD_ShadowRemove");
    if s.is_null() {
        return;
    }
    let c = (*s).camera;
    if !c.is_null() {
        let r = (*c).parent.ref_count;
        let dec = r == u16::MAX || r == 0;
        if r != u16::MAX {
            (*c).parent.ref_count = r.wrapping_sub(1);
        }
        if dec {
            hsd_delete(c.cast());
        }
    }
    HSD_ShadowDeleteObject(s, null_mut());
    let t = (*s).texture;
    if (*s).active != 0 {
        ext::HSD_MObjDeleteShadowTexture(t);
    }
    let d = (*t).imagedesc;
    if !(*d).image_ptr.is_null() {
        ext::HSD_Free((*d).image_ptr);
    }
    ext::HSD_ImageDescFree(d);
    ext::HSD_TObjFree(t);
    ext::HSD_ObjFree(HSD_ShadowGetAllocData(), s.cast());
}
pub unsafe fn HSD_ShadowInit(s: *mut HSD_Shadow) {
    melee_core::ct!("HSD_ShadowInit");
    assert!(!s.is_null());
    let d = &*(*(*s).texture).imagedesc;
    ext::GXSetTexCopySrc(0, 0, d.width, d.height);
    ext::GXSetTexCopyDst(d.width, d.height, 0x20, 0);
}
pub unsafe fn HSD_ShadowSetSize(s: *mut HSD_Shadow, w: u16, h: u16) {
    melee_core::ct!("HSD_ShadowSetSize");
    assert!(!s.is_null() && w > 0 && h > 0);
    let d = (*(*s).texture).imagedesc;
    if (*d).image_ptr.is_null() || (*d).width != w || (*d).height != h {
        if !(*d).image_ptr.is_null() {
            ext::HSD_Free((*d).image_ptr);
        }
        let size = ext::GXGetTexBufferSize(w, h, 0, 0, 0);
        assert!(size > 0);
        (*d).image_ptr = ext::HSD_MemAlloc(size as i32);
        (*d).width = w;
        (*d).height = h;
        ext::HSD_CObjSetViewportfx4((*s).camera, 0.0, w as f32, 0.0, h as f32);
        ext::HSD_CObjSetScissorx4((*s).camera, 0, w, 0, h);
    }
}
pub unsafe fn drawBackgroundRect(s: *mut HSD_Shadow) {
    melee_core::ct!("drawBackgroundRect");
    let c = (*s).camera;
    ext::GXLoadPosMtxImm(identity_mtx(), 0);
    perf_count_mtx_load();
    ext::GXSetCurrentMtx(0);
    ext::HSD_ClearVtxDesc();
    ext::GXSetVtxDesc(9, 1);
    ext::GXSetVtxAttrFmt(0, 9, 1, 4, 0);
    ext::HSD_StateSetCullMode(2);
    let t = ext::HSD_CObjGetTop(c) * 1.2;
    let b = ext::HSD_CObjGetBottom(c) * 1.2;
    let l = ext::HSD_CObjGetLeft(c) * 1.2;
    let r = ext::HSD_CObjGetRight(c) * 1.2;
    let n = ext::HSD_CObjGetNear(c) * -1.1;
    ext::GXBegin(0x80, 0, 4);
    for (a, b) in [(l, t), (n, r), (t, n), (r, b), (n, l), (b, n)] {
        ext::GXPosition2f32(a, b);
    }
    
}
pub unsafe fn HSD_ShadowStartRender(s: *mut HSD_Shadow) {
    melee_core::ct!("HSD_ShadowStartRender");
    assert!(!s.is_null());
    assert!(!(*s).camera.is_null() && !(*s).texture.is_null());
    let d = (*(*s).texture).imagedesc;
    assert!(!d.is_null());
    if (*s).objects.is_null() {
        return;
    }
    ext::HSD_CObjSetCurrent((*s).camera);
    let mut chan = HSD_Chan::zeroed();
    chan.chan = 4;
    chan.mat_color.a = 255;
    chan.diff_fn = 2;
    chan.attn_fn = 2;
    let mut tev = _HSD_TevDesc::zeroed();
    tev.flags = 1;
    tev.coord = 255;
    tev.map = 255;
    tev.color = 4;
    let conf = &mut tev.u.tevconf;
    conf.clr_a = 15;
    conf.clr_b = 15;
    conf.clr_c = 15;
    conf.clr_d = 10;
    conf.clr_clamp = 1;
    conf.alpha_a = 7;
    conf.alpha_b = 7;
    conf.alpha_c = 7;
    conf.alpha_d = 5;
    conf.alpha_clamp = 1;
    let mut pe = HSD_PEDesc::zeroed();
    pe.flags = 9;
    pe.src_factor = 4;
    pe.dst_factor = 5;
    pe.logic_op = 15;
    pe.z_comp = 7;
    pe.alpha_comp0 = 7;
    pe.alpha_comp1 = 7;
    ext::HSD_StateInitTev();
    ext::HSD_SetupTevStageAll(&mut tev);
    ext::HSD_StateSetNumTexGens();
    ext::HSD_SetupPEMode(0, &mut pe);
    chan.mat_color.r = 255;
    chan.mat_color.g = 255;
    chan.mat_color.b = 255;
    ext::HSD_SetupChannelAll(&mut chan);
    ext::GXSetScissor(0, 0, (*d).width as u32, (*d).height as u32);
    drawBackgroundRect(s);
    chan.mat_color.r = (*s).intensity;
    chan.mat_color.g = (*s).intensity;
    chan.mat_color.b = (*s).intensity;
    ext::HSD_SetupChannelAll(&mut chan);
    ext::GXSetScissor(
        2,
        2,
        ((*d).width as u32).wrapping_sub(4),
        ((*d).height as u32).wrapping_sub(4),
    );
    let mut list = (*s).objects;
    while !list.is_null() {
        ext::HSD_JObjDispAll((*list).data.cast(), null_mut(), 5, 1 << 26);
        list = (*list).next;
    }
    ext::HSD_CObjEndCurrent();
}
pub unsafe fn HSD_ShadowEndRender(s: *mut HSD_Shadow) {
    melee_core::ct!("HSD_ShadowEndRender");
    assert!(!s.is_null());
    let d = (*(*s).texture).imagedesc;
    if (*d).image_ptr.is_null() {
        HSD_ShadowSetSize(s, (*d).width, (*d).height);
    }
    ext::GXCopyTex((*d).image_ptr, 1);
    ext::GXPixModeSync();
    ext::GXInvalidateTexAll();
    makeMatrix(s);
}
pub unsafe fn HSD_ShadowSetActive(s: *mut HSD_Shadow, active: i32) {
    melee_core::ct!("HSD_ShadowSetActive");
    assert!(!s.is_null());
    if ((*s).active != 0) == (active != 0) {
        return;
    }
    (*s).active = active;
    if active != 0 {
        let d = (*(*s).texture).imagedesc;
        if (*d).image_ptr.is_null() {
            HSD_ShadowSetSize(s, (*d).width, (*d).height);
        }
        ext::HSD_MObjAddShadowTexture((*s).texture);
    } else {
        ext::HSD_MObjDeleteShadowTexture((*s).texture);
    }
}
pub unsafe fn HSD_ShadowAddObject(s: *mut HSD_Shadow, j: *mut HSD_JObj) {
    melee_core::ct!("HSD_ShadowAddObject");
    if s.is_null() || j.is_null() {
        return;
    }
    let mut l = (*s).objects;
    while !l.is_null() {
        if (*l).data == j.cast() {
            return;
        }
        l = (*l).next;
    }
    (*s).objects = ext::HSD_SListAllocAndPrepend((*s).objects, j.cast());
    jobj_ref(j);
}
pub unsafe fn HSD_ShadowDeleteObject(s: *mut HSD_Shadow, j: *mut HSD_JObj) {
    melee_core::ct!("HSD_ShadowDeleteObject");
    if s.is_null() {
        return;
    }
    if !j.is_null() {
        let mut l = addr_of_mut!((*s).objects);
        while !(*l).is_null() {
            if (**l).data == j.cast() {
                ext::HSD_JObjUnref(j);
                *l = ext::HSD_SListRemove(*l);
                return;
            }
            l = addr_of_mut!((**l).next);
        }
    } else {
        while !(*s).objects.is_null() {
            ext::HSD_JObjUnref((*(*s).objects).data.cast());
            (*s).objects = ext::HSD_SListRemove((*s).objects);
        }
    }
}
pub unsafe fn makeMatrix(s: *mut HSD_Shadow) {
    melee_core::ct!("makeMatrix");
    let c = (*s).camera;
    let p = &(*c).projection_param;
    let mut mprj: Mtx = [[0.0; 4]; 3];
    let mp: *mut c_void = (&mut mprj as *mut Mtx).cast();
    match ext::HSD_CObjGetProjectionType(c) {
        1 => ext::MTXLightPerspective(mp, p.perspective.fov, p.perspective.aspect, (*s).scaleS, (*s).scaleT, (*s).transS, (*s).transT),
        2 => ext::MTXLightFrustum(mp, p.frustum.top, p.frustum.bottom, p.frustum.left, p.frustum.right, (*c).near, (*s).scaleS, (*s).scaleT, (*s).transS, (*s).transT),
        3 => ext::MTXLightOrtho(mp, p.ortho.top, p.ortho.bottom, p.ortho.left, p.ortho.right, (*s).scaleS, (*s).scaleT, (*s).transS, (*s).transT),
        _ => panic!("shadow.c:698"),
    };
    ext::PSMTXConcat(mp, cobj_view_mtx(c).cast(), (&raw mut (*(*s).texture).mtx).cast());
}
pub unsafe fn HSD_ShadowSetViewingRect(
    s: *mut HSD_Shadow,
    t: f32,
    b: f32,
    l: f32,
    r: f32,
) {
    melee_core::ct!("HSD_ShadowSetViewingRect");
    assert!(!s.is_null());
    let c = (*s).camera;
    let distance = ext::HSD_CObjGetEyeDistance(c);
    assert!(distance > 0.0);
    match ext::HSD_CObjGetProjectionType(c) {
        1 => {
            let width = if t.abs() > b.abs() { t.abs() } else { b.abs() };
            let height = if l.abs() > r.abs() { l.abs() } else { r.abs() };
            ext::HSD_CObjSetAspect(c, height / width);
            ext::HSD_CObjSetFov(c, ext::atan2f(height, distance));
        }
        3 => ext::HSD_CObjSetOrtho(c, t, b, l, r),
        2 => {
            let scale = ext::HSD_CObjGetNear(c) / distance;
            assert!(scale > 0.0);
            ext::HSD_CObjSetFrustum(c, scale * t, scale * b, scale * l, scale * r);
        }
        _ => panic!("shadow.c:762"),
    }
}
pub unsafe fn HSD_ViewingRectInit(
    r: *mut HSD_ViewingRect,
    p: *mut Vec3,
    i: *mut Vec3,
    u: *mut Vec3,
    perspective: i32,
) {
    melee_core::ct!("HSD_ViewingRectInit");
    assert!(!r.is_null());
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*r).origin = *p;
    ext::PSVECSubtract(i, p, &mut (*r).eye_v);
    ext::PSVECNormalize(&mut (*r).eye_v, &mut (*r).eye_vn);
    ext::PSVECNormalize(u, &mut v);
    ext::PSVECCrossProduct(&mut (*r).eye_vn, &mut v, &mut (*r).right_v);
    ext::PSVECCrossProduct(&mut (*r).right_v, &mut (*r).eye_vn, &mut (*r).up_v);
    (*r).distance = ext::PSVECMag(&mut (*r).eye_v);
    (*r).top = -f32::MAX;
    (*r).right = -f32::MAX;
    (*r).bottom = f32::MAX;
    (*r).left = f32::MAX;
    (*r).perspective = perspective;
}
pub unsafe fn HSD_ViewingRectCheck(r: *mut HSD_ViewingRect) -> i32 {
    melee_core::ct!("HSD_ViewingRectCheck");
    assert!(!r.is_null());
    ((*r).top > (*r).bottom && (*r).right > (*r).left) as i32
}
pub unsafe fn HSD_ViewingRectAddRect(
    r: *mut HSD_ViewingRect,
    p: *mut Vec3,
    mut t: f32,
    mut b: f32,
    mut l: f32,
    mut right: f32,
) {
    melee_core::ct!("HSD_ViewingRectAddRect");
    assert!(!r.is_null() && !p.is_null());
    let mut o2p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut e2p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let x;
    let y;
    ext::PSVECSubtract(p, &mut (*r).origin, &mut o2p);
    let dot = ext::PSVECDotProduct(&mut o2p, &mut (*r).eye_vn);
    if (*r).perspective != 0 {
        if dot <= 0.0 {
            return;
        }
        let scale = (*r).distance / dot;
        crate::tu::mtx_pc::raw::PSVECScale(&mut o2p, &mut o2p, scale);
        ext::PSVECSubtract(&mut o2p, &mut (*r).eye_v, &mut e2p);
        x = ext::PSVECDotProduct(&mut (*r).right_v, &mut e2p);
        y = ext::PSVECDotProduct(&mut (*r).up_v, &mut e2p);
        t *= scale;
        b *= scale;
        l *= scale;
        right *= scale;
    } else {
        let mut tmp = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        crate::tu::mtx_pc::raw::PSVECScale(&mut (*r).eye_vn, &mut tmp, dot);
        ext::PSVECSubtract(&mut o2p, &mut tmp, &mut e2p);
        x = ext::PSVECDotProduct(&mut (*r).right_v, &mut e2p);
        y = ext::PSVECDotProduct(&mut (*r).up_v, &mut e2p);
    }
    if x + right > (*r).right {
        (*r).right = x + right;
    }
    if x + l < (*r).left {
        (*r).left = x + l;
    }
    if y + t > (*r).top {
        (*r).top = y + t;
    }
    if y + b < (*r).bottom {
        (*r).bottom = y + b;
    }
}

fn cv(v: &Vec3) -> melee_core::vec::Vec3 {
    melee_core::vec::Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}
fn tv(v: melee_core::vec::Vec3) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}
fn vec_sub(a: &Vec3, b: &Vec3) -> Vec3 {
    tv(crate::mtx::vec_sub(&cv(a), &cv(b)))
}
fn vec_cross(a: &Vec3, b: &Vec3) -> Vec3 {
    tv(crate::mtx::vec_cross(&cv(a), &cv(b)))
}
fn vec_dot(a: &Vec3, b: &Vec3) -> f32 {
    crate::mtx::vec_dot(&cv(a), &cv(b))
}
fn vec_normalize(a: &Vec3) -> Vec3 {
    tv(crate::mtx::vec_normalize(&cv(a)))
}
fn vec_scale(a: &Vec3, b: f32) -> Vec3 {
    tv(crate::mtx::vec_scale(&cv(a), b))
}
fn vec_mag(a: &Vec3) -> f32 {
    crate::mtx::vec_mag(&cv(a))
}
