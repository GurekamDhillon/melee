//! Native-layout port of `src/sysdolphin/baselib/wobj.c` (raw-pointer API over the GameCube-layout
//! `HSD_WObj`). The verified safe logic lives in `crate::wobj`; this is the C-shaped surface.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, dead_code)]
pub use crate::wobj::{HsdWObj, HsdWObjAnim, HsdWObjDesc, WObjHost};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
const HSD_OBJ_NOREF: u16 = 0xFFFF;

/// `hsdWObj` (class info; `info_init` = `WObjInfoInit`).
pub static mut hsdWObj: HSD_WObjInfo = {
    let mut info: HSD_WObjInfo = unsafe { core::mem::zeroed() };
    info.parent.parent.head.info_init = Some(WObjInfoInit);
    info
};
pub static mut default_class: *mut HSD_ClassInfo = null_mut();

/// Callees in other TUs. Unported ones panic with a TODO naming the C function.
pub mod ext {
    use super::*;
    pub static mut hsdObj: HSD_ClassInfo = unsafe { core::mem::zeroed() };
    pub static mut HSD_AObjRemove: unsafe fn(*mut HSD_AObj) = |_| todo!("HSD_AObjRemove");
    pub static mut HSD_RObjRemoveAnimAll: unsafe fn(*mut HSD_RObj) = |_| todo!("HSD_RObjRemoveAnimAll");
    pub static mut HSD_RObjRemoveAll: unsafe fn(*mut HSD_RObj) = |_| todo!("HSD_RObjRemoveAll");
    pub static mut HSD_RObjAnimAll: unsafe fn(*mut HSD_RObj) = |_| todo!("HSD_RObjAnimAll");
    pub static mut HSD_AObjReqAnim: unsafe fn(*mut HSD_AObj, f32) = |_, _| todo!("HSD_AObjReqAnim");
    pub static mut HSD_RObjReqAnimAll: unsafe fn(*mut HSD_RObj, f32) = |_, _| todo!("HSD_RObjReqAnimAll");
    pub static mut HSD_RObjAddAnimAll: unsafe fn(*mut HSD_RObj, *mut HSD_RObjAnimJoint) =
        |_, _| todo!("HSD_RObjAddAnimAll");
    pub static mut HSD_RObjResolveRefsAll: unsafe fn(*mut HSD_RObj, *mut HSD_RObjDesc) =
        |_, _| todo!("HSD_RObjResolveRefsAll");
    pub static mut HSD_AObjLoadDesc: unsafe fn(*mut HSD_AObjDesc) -> *mut HSD_AObj = |_| todo!("HSD_AObjLoadDesc");
    pub static mut HSD_RObjLoadDesc: unsafe fn(*mut HSD_RObjDesc) -> *mut HSD_RObj = |_| todo!("HSD_RObjLoadDesc");
    pub static mut HSD_AObjInterpretAnim: unsafe fn(*mut HSD_AObj, *mut c_void, HSD_ObjUpdateFunc) =
        |_, _, _| todo!("HSD_AObjInterpretAnim");
    pub static mut HSD_JObjSetupMatrixSub: unsafe fn(*mut HSD_JObj) = |_| todo!("HSD_JObjSetupMatrixSub");
    pub static mut PSMTXMultVec: unsafe fn(*mut [f32; 4], *mut Vec3, *mut Vec3) = super::native_mtx_mult_vec;
    pub static mut splArcLengthPoint: unsafe fn(*mut Vec3, *mut HSD_Spline, f32) = super::native_spline_point;
    pub static mut hsdIsDescendantOf: unsafe fn(*mut c_void, *mut c_void) -> i32 =
        |_, _| todo!("hsdIsDescendantOf");
    pub static mut hsdSearchClassInfo: unsafe fn(*const i8) -> *mut HSD_ClassInfo = |_| todo!("hsdSearchClassInfo");
    pub static mut hsdNew: unsafe fn(*mut HSD_ClassInfo) -> *mut c_void = |_| todo!("hsdNew");
    pub static mut hsdInitClassInfo: unsafe fn(*mut HSD_ClassInfo, *mut HSD_ClassInfo, *mut i8, *mut i8, i32, i32) =
        |_, _, _, _, _, _| todo!("hsdInitClassInfo");
}

/// `MTXMultVec(m, src, dst)`; reuses the verified fused implementation.
pub unsafe fn native_mtx_mult_vec(m: *mut [f32; 4], src: *mut Vec3, dst: *mut Vec3) {
    let m = core::slice::from_raw_parts(m, 3);
    let v = crate::mtx::mult_vec(m, &melee_core::vec::Vec3 { x: (*src).x, y: (*src).y, z: (*src).z });
    *dst = Vec3 { x: v.x, y: v.y, z: v.z };
}
/// TODO: `splArcLengthPoint` (spline.c) has no native-pointer port yet.
pub unsafe fn native_spline_point(_: *mut Vec3, _: *mut HSD_Spline, _: f32) {
    todo!("splArcLengthPoint (spline.c)")
}

/// jobj.h `HSD_JObjSetupMatrix` (inline): rebuild when dirty and not user-defined.
unsafe fn jobj_setup_matrix(j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    let fl = (*j).flags;
    if fl & JOBJ_USER_DEF_MTX == 0 && fl & (JOBJ_MTX_DIRTY as u32) != 0 {
        (ext::HSD_JObjSetupMatrixSub)(j);
    }
}

/// The `if (flags & 1) { ... }` prologue shared by the position setters/getter.
unsafe fn resolve_world_pos(w: *mut HSD_WObj) {
    if (*w).flags & 1 != 0 {
        if !(*w).aobj.is_null() && !(*(*w).aobj).hsd_obj.is_null() {
            let jp = (*(*w).aobj).hsd_obj as *mut HSD_JObj;
            jobj_setup_matrix(jp);
            let pos = addr_of_mut!((*w).pos);
            (ext::PSMTXMultVec)((*jp).mtx.as_mut_ptr().cast(), pos, pos);
        }
        (*w).flags &= 0xFFFF_FFFE;
    }
}

pub unsafe fn HSD_WObjRemoveAnim(wobj: *mut HSD_WObj) {
    if !wobj.is_null() {
        (ext::HSD_AObjRemove)((*wobj).aobj);
        (*wobj).aobj = null_mut();
        (ext::HSD_RObjRemoveAnimAll)((*wobj).robj);
    }
}

pub unsafe fn HSD_WObjReqAnim(wobj: *mut HSD_WObj, frame: f32) {
    if !wobj.is_null() {
        (ext::HSD_AObjReqAnim)((*wobj).aobj, frame);
        (ext::HSD_RObjReqAnimAll)((*wobj).robj, frame);
    }
}

pub unsafe fn HSD_WObjAddAnim(wobj: *mut HSD_WObj, anim: *mut HSD_WObjAnim) {
    if !wobj.is_null() && !anim.is_null() {
        if !(*wobj).aobj.is_null() {
            (ext::HSD_AObjRemove)((*wobj).aobj);
        }
        (*wobj).aobj = (ext::HSD_AObjLoadDesc)((*anim).aobjdesc);
        (ext::HSD_RObjAddAnimAll)((*wobj).robj, (*anim).robjanim);
    }
}

pub unsafe extern "C" fn WObjUpdateFunc(obj: *mut c_void, ty: i32, val: *mut HSD_ObjData) {
    let wobj = obj as *mut HSD_WObj;
    if wobj.is_null() {
        return;
    }
    match ty {
        4 => {
            if ((*val).fv as f64) < 0.0 {
                (*val).fv = 0.0;
            }
            if 1.0 < ((*val).fv as f64) {
                (*val).fv = 1.0;
            }
            assert!(!(*wobj).aobj.is_null(), "wobj.c:148 wobj->aobj");
            let jp = (*(*wobj).aobj).hsd_obj as *mut HSD_JObj;
            assert!(!jp.is_null(), "wobj.c:150 jp");
            assert!(!(*jp).u.spline.is_null(), "wobj.c:151 jp->u.spline");
            let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            (ext::splArcLengthPoint)(&mut p, (*jp).u.spline, (*val).fv);
            HSD_WObjSetPosition(wobj, &mut p);
            (*wobj).flags |= 1;
        }
        5 => HSD_WObjSetPositionX(wobj, (*val).fv),
        6 => HSD_WObjSetPositionY(wobj, (*val).fv),
        7 => HSD_WObjSetPositionZ(wobj, (*val).fv),
        _ => {}
    }
}

pub unsafe fn HSD_WObjInterpretAnim(wobj: *mut HSD_WObj) {
    if !wobj.is_null() {
        (ext::HSD_AObjInterpretAnim)((*wobj).aobj, wobj.cast(), Some(WObjUpdateFunc));
        (ext::HSD_RObjAnimAll)((*wobj).robj);
    }
}

pub unsafe extern "C" fn WObjLoad(wobj: *mut HSD_WObj, desc: *mut HSD_WObjDesc) -> i32 {
    HSD_WObjSetPosition(wobj, addr_of_mut!((*desc).pos));
    if !(*wobj).robj.is_null() {
        (ext::HSD_RObjRemoveAll)((*wobj).robj);
    }
    (*wobj).robj = (ext::HSD_RObjLoadDesc)((*desc).robjdesc);
    (ext::HSD_RObjResolveRefsAll)((*wobj).robj, (*desc).robjdesc);
    0
}

pub unsafe fn HSD_WObjInit(wobj: *mut HSD_WObj, desc: *mut HSD_WObjDesc) {
    if wobj.is_null() || desc.is_null() {
        return;
    }
    HSD_WObjSetPosition(wobj, addr_of_mut!((*desc).pos));
    if !(*wobj).robj.is_null() {
        (ext::HSD_RObjRemoveAll)((*wobj).robj);
    }
    (*wobj).robj = (ext::HSD_RObjLoadDesc)((*desc).robjdesc);
    (ext::HSD_RObjResolveRefsAll)((*wobj).robj, (*desc).robjdesc);
}

pub unsafe fn HSD_WObjSetDefaultClass(info: *mut HSD_ClassInfo) {
    if !info.is_null() {
        assert!(
            (ext::hsdIsDescendantOf)(info.cast(), addr_of_mut!(hsdWObj).cast()) != 0,
            "wobj.c:221 hsdIsDescendantOf(info, &hsdWObj)"
        );
    }
    default_class = info;
}

pub unsafe fn HSD_WObjLoadDesc(desc: *mut HSD_WObjDesc) -> *mut HSD_WObj {
    if desc.is_null() {
        return null_mut();
    }
    let mut info: *mut HSD_ClassInfo = null_mut();
    let use_alloc = (*desc).class_name.is_null() || {
        info = (ext::hsdSearchClassInfo)((*desc).class_name);
        info.is_null()
    };
    let wobj: *mut HSD_WObj = if use_alloc {
        HSD_WObjAlloc()
    } else {
        let w = (ext::hsdNew)(info) as *mut HSD_WObj;
        assert!(!w.is_null(), "wobj.c:252 wobj");
        w
    };
    let m = (*wobj).parent.parent.class_info as *mut HSD_WObjInfo;
    ((*m).load.expect("wobj load"))(wobj, desc);
    wobj
}

pub unsafe fn HSD_WObjSetPosition(wobj: *mut HSD_WObj, pos: *mut Vec3) {
    if wobj.is_null() || pos.is_null() {
        return;
    }
    (*wobj).pos = *pos;
    (*wobj).flags |= 0x2;
    (*wobj).flags &= 0xFFFF_FFFE;
}

pub unsafe fn HSD_WObjSetPositionX(wobj: *mut HSD_WObj, val: f32) {
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.x = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjSetPositionY(wobj: *mut HSD_WObj, val: f32) {
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.y = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjSetPositionZ(wobj: *mut HSD_WObj, val: f32) {
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.z = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjGetPosition(wobj: *mut HSD_WObj, vec: *mut Vec3) {
    if wobj.is_null() || vec.is_null() {
        return;
    }
    resolve_world_pos(wobj);
    *vec = (*wobj).pos;
}

pub unsafe fn HSD_WObjAlloc() -> *mut HSD_WObj {
    let info = if default_class.is_null() { addr_of_mut!(hsdWObj).cast::<HSD_ClassInfo>() } else { default_class };
    let wobj = (ext::hsdNew)(info) as *mut HSD_WObj;
    assert!(!wobj.is_null(), "wobj.c:591 wobj");
    wobj
}

pub unsafe extern "C" fn WObjRelease(o: *mut HSD_Class) {
    let wobj = o as *mut HSD_WObj;
    (ext::HSD_RObjRemoveAll)((*wobj).robj);
    (ext::HSD_AObjRemove)((*wobj).aobj);
    let parent = hsdWObj.parent.parent.head.parent;
    ((*parent).release.expect("parent release"))(o);
}

pub unsafe extern "C" fn WObjAmnesia(info: *mut HSD_ClassInfo) {
    if info == default_class {
        default_class = null_mut();
    }
    let parent = hsdWObj.parent.parent.head.parent;
    ((*parent).amnesia.expect("parent amnesia"))(info);
}

pub unsafe extern "C" fn WObjInfoInit() {
    (ext::hsdInitClassInfo)(
        addr_of_mut!(hsdWObj).cast(),
        addr_of_mut!(ext::hsdObj),
        c"sysdolphin_base_library".as_ptr().cast_mut(),
        c"had_wobj".as_ptr().cast_mut(),
        core::mem::size_of::<HSD_WObjInfo>() as i32,
        core::mem::size_of::<HSD_WObj>() as i32,
    );
    hsdWObj.parent.parent.release = Some(WObjRelease);
    hsdWObj.parent.parent.amnesia = Some(WObjAmnesia);
    hsdWObj.load = Some(WObjLoad);
}

/// wobj.h `HSD_WObjUnref` (inline).
pub unsafe fn HSD_WObjUnref(wobj: *mut HSD_WObj) {
    if wobj.is_null() {
        return;
    }
    let dec = if (*wobj).parent.ref_count == HSD_OBJ_NOREF {
        true
    } else {
        let z = (*wobj).parent.ref_count == 0;
        (*wobj).parent.ref_count = (*wobj).parent.ref_count.wrapping_sub(1);
        z
    };
    if dec {
        let ci = (*wobj).parent.parent.class_info;
        ((*ci).release.expect("release"))(wobj.cast());
        ((*ci).destroy.expect("destroy"))(wobj.cast());
    }
}

/// wobj.h `HSD_WObjClearFlags` (inline).
pub unsafe fn HSD_WObjClearFlags(wobj: *mut HSD_WObj, flags: u32) {
    (*wobj).flags &= !flags;
}
