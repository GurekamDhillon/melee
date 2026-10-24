//! Native shadow objects; math delegates to existing oracle-verified ports.
#![allow(non_snake_case, non_upper_case_globals)]
use core::{
    ffi::c_void,
    ptr::{addr_of_mut, null_mut},
};
use melee_types::*;
pub static mut shadow_alloc_data: HSD_ObjAllocData =
    unsafe { core::mem::zeroed() };
macro_rules! hooks { ($($n:ident($($a:ident:$t:ty),*) $(->$r:ty)?;)* )=>{pub mod ext {use super::*;$(pub static mut $n:Option<unsafe fn($($t),*) $(->$r)?>=None;)*} $(unsafe fn $n($($a:$t),*) $(->$r)? {(ext::$n.expect(concat!("TODO ",stringify!($n))))($($a),*)})*};}
hooks! {
HSD_ObjAllocInit(a:*mut HSD_ObjAllocData,b:u32,c:u32); HSD_ObjAlloc(a:*mut HSD_ObjAllocData)->*mut c_void; HSD_ObjFree(a:*mut HSD_ObjAllocData,b:*mut c_void);
HSD_TObjAlloc()->*mut HSD_TObj; HSD_ImageDescAlloc()->*mut HSD_ImageDesc; HSD_CObjAlloc()->*mut HSD_CObj;
HSD_CObjSetViewportfx4(a:*mut HSD_CObj,b:f32,c:f32,d:f32,e:f32); HSD_CObjSetScissorx4(a:*mut HSD_CObj,b:u16,c:u16,d:u16,e:u16);
hsdDelete(a:*mut c_void); HSD_MObjDeleteShadowTexture(a:*mut HSD_TObj);HSD_Free(a:*mut c_void);HSD_ImageDescFree(a:*mut HSD_ImageDesc);HSD_TObjFree(a:*mut HSD_TObj);
GXSetTexCopySrc(a:u16,b:u16,c:u16,d:u16);GXSetTexCopyDst(a:u16,b:u16,c:u32,d:u8);GXGetTexBufferSize(a:u16,b:u16,c:u32,d:u8,e:u8)->u32;HSD_MemAlloc(a:u32)->*mut c_void;
GXLoadPosMtxImm(a:*const Mtx,b:u32);HSD_PerfCountMtxLoad();GXSetCurrentMtx(a:u32);HSD_ClearVtxDesc();GXSetVtxDesc(a:u32,b:u32);GXSetVtxAttrFmt(a:u32,b:u32,c:u32,d:u32,e:u8);HSD_StateSetCullMode(a:u32);
HSD_CObjGetTop(a:*mut HSD_CObj)->f32;HSD_CObjGetBottom(a:*mut HSD_CObj)->f32;HSD_CObjGetLeft(a:*mut HSD_CObj)->f32;HSD_CObjGetRight(a:*mut HSD_CObj)->f32;HSD_CObjGetNear(a:*mut HSD_CObj)->f32;
GXBegin(a:u32,b:u32,c:u16);GXPosition2f32(a:f32,b:f32);GXEnd();HSD_CObjSetCurrent(a:*mut HSD_CObj)->i32;HSD_StateInitTev();HSD_SetupTevStageAll(a:*mut _HSD_TevDesc);HSD_StateSetNumTexGens();HSD_SetupPEMode(a:u32,b:*mut HSD_PEDesc);HSD_SetupChannelAll(a:*mut HSD_Chan);GXSetScissor(a:u32,b:u32,c:u32,d:u32);HSD_JObjDispAll(a:*mut HSD_JObj,b:*mut Mtx,c:u32,d:u32);HSD_CObjEndCurrent();
GXCopyTex(a:*mut c_void,b:u8);GXPixModeSync();GXInvalidateTexAll();HSD_MObjAddShadowTexture(a:*mut HSD_TObj);
HSD_SListAllocAndPrepend(a:*mut _HSD_SList,b:*mut c_void)->*mut _HSD_SList;HSD_JObjRef(a:*mut HSD_JObj);HSD_JObjUnref(a:*mut HSD_JObj);HSD_SListRemove(a:*mut _HSD_SList)->*mut _HSD_SList;
HSD_CObjGetProjectionType(a:*mut HSD_CObj)->u32;HSD_CObjGetViewingMtxPtrDirect(a:*mut HSD_CObj)->*mut Mtx;HSD_CObjGetEyeDistance(a:*mut HSD_CObj)->f32;HSD_CObjSetAspect(a:*mut HSD_CObj,b:f32);HSD_CObjSetFov(a:*mut HSD_CObj,b:f32);HSD_CObjSetOrtho(a:*mut HSD_CObj,b:f32,c:f32,d:f32,e:f32);HSD_CObjSetFrustum(a:*mut HSD_CObj,b:f32,c:f32,d:f32,e:f32);
}
pub unsafe fn HSD_ShadowGetAllocData() -> *mut HSD_ObjAllocData {
    addr_of_mut!(shadow_alloc_data)
}
pub unsafe fn HSD_ShadowInitAllocData() {
    HSD_ObjAllocInit(
        HSD_ShadowGetAllocData(),
        core::mem::size_of::<HSD_Shadow>() as u32,
        4,
    );
}
pub unsafe fn makeShadowTObj() -> *mut HSD_TObj {
    let t = HSD_TObjAlloc();
    (*t).src = 0;
    (*t).wrap_s = 0;
    (*t).wrap_t = 0;
    (*t).flags = 0x540103;
    (*t).imagedesc = HSD_ImageDescAlloc();
    t
}
pub unsafe fn HSD_ShadowAlloc() -> *mut HSD_Shadow {
    let s = HSD_ObjAlloc(HSD_ShadowGetAllocData()).cast::<HSD_Shadow>();
    s.write(HSD_Shadow::zeroed());
    (*s).camera = HSD_CObjAlloc();
    (*s).texture = makeShadowTObj();
    (*s).scaleS = 0.5;
    (*s).scaleT = -0.5;
    (*s).transS = 0.5;
    (*s).transT = 0.5;
    let d = (*(*s).texture).imagedesc;
    (*d).format = 0;
    (*d).width = 256;
    (*d).height = 256;
    HSD_CObjSetViewportfx4((*s).camera, 0.0, 256.0, 0.0, 256.0);
    HSD_CObjSetScissorx4((*s).camera, 0, 256, 0, 256);
    s
}
pub unsafe fn HSD_ShadowRemove(s: *mut HSD_Shadow) {
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
            hsdDelete(c.cast());
        }
    }
    HSD_ShadowDeleteObject(s, null_mut());
    let t = (*s).texture;
    if (*s).active != 0 {
        HSD_MObjDeleteShadowTexture(t);
    }
    let d = (*t).imagedesc;
    if !(*d).image_ptr.is_null() {
        HSD_Free((*d).image_ptr);
    }
    HSD_ImageDescFree(d);
    HSD_TObjFree(t);
    HSD_ObjFree(HSD_ShadowGetAllocData(), s.cast());
}
pub unsafe fn HSD_ShadowInit(s: *mut HSD_Shadow) {
    assert!(!s.is_null());
    let d = &*(*(*s).texture).imagedesc;
    GXSetTexCopySrc(0, 0, d.width, d.height);
    GXSetTexCopyDst(d.width, d.height, 0x20, 0);
}
pub unsafe fn HSD_ShadowSetSize(s: *mut HSD_Shadow, w: u16, h: u16) {
    assert!(!s.is_null() && w > 0 && h > 0);
    let d = (*(*s).texture).imagedesc;
    if (*d).image_ptr.is_null() || (*d).width != w || (*d).height != h {
        if !(*d).image_ptr.is_null() {
            HSD_Free((*d).image_ptr);
        }
        let size = GXGetTexBufferSize(w, h, 0, 0, 0);
        assert!(size > 0);
        (*d).image_ptr = HSD_MemAlloc(size);
        (*d).width = w;
        (*d).height = h;
        HSD_CObjSetViewportfx4((*s).camera, 0.0, w as f32, 0.0, h as f32);
        HSD_CObjSetScissorx4((*s).camera, 0, w, 0, h);
    }
}
pub unsafe fn drawBackgroundRect(s: *mut HSD_Shadow) {
    let c = (*s).camera;
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    GXLoadPosMtxImm(&identity, 0);
    HSD_PerfCountMtxLoad();
    GXSetCurrentMtx(0);
    HSD_ClearVtxDesc();
    GXSetVtxDesc(9, 1);
    GXSetVtxAttrFmt(0, 9, 1, 4, 0);
    HSD_StateSetCullMode(2);
    let t = HSD_CObjGetTop(c) * 1.2;
    let b = HSD_CObjGetBottom(c) * 1.2;
    let l = HSD_CObjGetLeft(c) * 1.2;
    let r = HSD_CObjGetRight(c) * 1.2;
    let n = HSD_CObjGetNear(c) * -1.1;
    GXBegin(0x80, 0, 4);
    for (a, b) in [(l, t), (n, r), (t, n), (r, b), (n, l), (b, n)] {
        GXPosition2f32(a, b);
    }
    GXEnd();
}
pub unsafe fn HSD_ShadowStartRender(s: *mut HSD_Shadow) {
    assert!(!s.is_null());
    assert!(!(*s).camera.is_null() && !(*s).texture.is_null());
    let d = (*(*s).texture).imagedesc;
    assert!(!d.is_null());
    if (*s).objects.is_null() {
        return;
    }
    HSD_CObjSetCurrent((*s).camera);
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
    HSD_StateInitTev();
    HSD_SetupTevStageAll(&mut tev);
    HSD_StateSetNumTexGens();
    HSD_SetupPEMode(0, &mut pe);
    chan.mat_color.r = 255;
    chan.mat_color.g = 255;
    chan.mat_color.b = 255;
    HSD_SetupChannelAll(&mut chan);
    GXSetScissor(0, 0, (*d).width as u32, (*d).height as u32);
    drawBackgroundRect(s);
    chan.mat_color.r = (*s).intensity;
    chan.mat_color.g = (*s).intensity;
    chan.mat_color.b = (*s).intensity;
    HSD_SetupChannelAll(&mut chan);
    GXSetScissor(
        2,
        2,
        ((*d).width as u32).wrapping_sub(4),
        ((*d).height as u32).wrapping_sub(4),
    );
    let mut list = (*s).objects;
    while !list.is_null() {
        HSD_JObjDispAll((*list).data.cast(), null_mut(), 5, 1 << 26);
        list = (*list).next;
    }
    HSD_CObjEndCurrent();
}
pub unsafe fn HSD_ShadowEndRender(s: *mut HSD_Shadow) {
    assert!(!s.is_null());
    let d = (*(*s).texture).imagedesc;
    if (*d).image_ptr.is_null() {
        HSD_ShadowSetSize(s, (*d).width, (*d).height);
    }
    GXCopyTex((*d).image_ptr, 1);
    GXPixModeSync();
    GXInvalidateTexAll();
    makeMatrix(s);
}
pub unsafe fn HSD_ShadowSetActive(s: *mut HSD_Shadow, active: i32) {
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
        HSD_MObjAddShadowTexture((*s).texture);
    } else {
        HSD_MObjDeleteShadowTexture((*s).texture);
    }
}
pub unsafe fn HSD_ShadowAddObject(s: *mut HSD_Shadow, j: *mut HSD_JObj) {
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
    (*s).objects = HSD_SListAllocAndPrepend((*s).objects, j.cast());
    HSD_JObjRef(j);
}
pub unsafe fn HSD_ShadowDeleteObject(s: *mut HSD_Shadow, j: *mut HSD_JObj) {
    if s.is_null() {
        return;
    }
    if !j.is_null() {
        let mut l = addr_of_mut!((*s).objects);
        while !(*l).is_null() {
            if (**l).data == j.cast() {
                HSD_JObjUnref(j);
                *l = HSD_SListRemove(*l);
                return;
            }
            l = addr_of_mut!((**l).next);
        }
    } else {
        while !(*s).objects.is_null() {
            HSD_JObjUnref((*(*s).objects).data.cast());
            (*s).objects = HSD_SListRemove((*s).objects);
        }
    }
}
pub unsafe fn makeMatrix(s: *mut HSD_Shadow) {
    let c = (*s).camera;
    let p = &(*c).projection_param;
    let m = match HSD_CObjGetProjectionType(c) {
        1 => crate::mtx::light_perspective(
            p.perspective.fov,
            p.perspective.aspect,
            (*s).scaleS,
            (*s).scaleT,
            (*s).transS,
            (*s).transT,
        ),
        2 => crate::mtx::light_frustum(
            p.frustum.top,
            p.frustum.bottom,
            p.frustum.left,
            p.frustum.right,
            (*c).near,
            (*s).scaleS,
            (*s).scaleT,
            (*s).transS,
            (*s).transT,
        ),
        3 => crate::mtx::light_ortho(
            p.ortho.top,
            p.ortho.bottom,
            p.ortho.left,
            p.ortho.right,
            (*s).scaleS,
            (*s).scaleT,
            (*s).transS,
            (*s).transT,
        ),
        _ => panic!("shadow.c:698"),
    };
    (*(*s).texture).mtx =
        crate::mtx::concat(&m, &*HSD_CObjGetViewingMtxPtrDirect(c));
}
pub unsafe fn HSD_ShadowSetViewingRect(
    s: *mut HSD_Shadow,
    t: f32,
    b: f32,
    l: f32,
    r: f32,
) {
    assert!(!s.is_null());
    let c = (*s).camera;
    let distance = HSD_CObjGetEyeDistance(c);
    assert!(distance > 0.0);
    match HSD_CObjGetProjectionType(c) {
        1 => {
            let width = if t.abs() > b.abs() { t.abs() } else { b.abs() };
            let height = if l.abs() > r.abs() { l.abs() } else { r.abs() };
            HSD_CObjSetAspect(c, height / width);
            HSD_CObjSetFov(c, crate::lbtrigf::atan2f(height, distance));
        }
        3 => HSD_CObjSetOrtho(c, t, b, l, r),
        2 => {
            let scale = HSD_CObjGetNear(c) / distance;
            assert!(scale > 0.0);
            HSD_CObjSetFrustum(c, scale * t, scale * b, scale * l, scale * r);
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
    assert!(!r.is_null());
    (*r).origin = *p;
    (*r).eye_v = vec_sub(&*i, &*p);
    (*r).eye_vn = vec_normalize(&(*r).eye_v);
    let v = vec_normalize(&*u);
    (*r).right_v = vec_cross(&(*r).eye_vn, &v);
    (*r).up_v = vec_cross(&(*r).right_v, &(*r).eye_vn);
    (*r).distance = vec_mag(&(*r).eye_v);
    (*r).top = -f32::MAX;
    (*r).right = -f32::MAX;
    (*r).bottom = f32::MAX;
    (*r).left = f32::MAX;
    (*r).perspective = perspective;
}
pub unsafe fn HSD_ViewingRectCheck(r: *mut HSD_ViewingRect) -> i32 {
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
    assert!(!r.is_null() && !p.is_null());
    let mut o = vec_sub(&*p, &(*r).origin);
    let dot = vec_dot(&o, &(*r).eye_vn);
    let e;
    if (*r).perspective != 0 {
        if dot <= 0.0 {
            return;
        }
        let scale = (*r).distance / dot;
        o = vec_scale(&o, scale);
        e = vec_sub(&o, &(*r).eye_v);
        t *= scale;
        b *= scale;
        l *= scale;
        right *= scale;
    } else {
        let tmp = vec_scale(&(*r).eye_vn, dot);
        e = vec_sub(&o, &tmp);
    }
    let x = vec_dot(&(*r).right_v, &e);
    let y = vec_dot(&(*r).up_v, &e);
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
