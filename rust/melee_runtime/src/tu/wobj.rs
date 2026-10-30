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
    fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData)>);
    fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj;
    fn HSD_AObjRemove(a0: *mut HSD_AObj);
    fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_RObjAddAnimAll(a0: *mut HSD_RObj, a1: *mut HSD_RObjAnimJoint);
    fn HSD_RObjAnimAll(a0: *mut HSD_RObj);
    fn HSD_RObjLoadDesc(a0: *mut HSD_RObjDesc) -> *mut HSD_RObj;
    fn HSD_RObjRemoveAll(a0: *mut HSD_RObj);
    fn HSD_RObjRemoveAnimAll(a0: *mut HSD_RObj);
    fn HSD_RObjReqAnimAll(a0: *mut HSD_RObj, a1: f32);
    fn HSD_RObjResolveRefsAll(a0: *mut HSD_RObj, a1: *mut HSD_RObjDesc);
    fn PSMTXMultVec(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec) = super::native_mtx_mult_vec(a0 as _, a1, a2);
    fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *mut i8, a3: *mut i8, a4: i32, a5: i32);
    fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32;
    fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void;
    fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo;
    fn splArcLengthPoint(a0: *mut Vec, a1: *mut HSD_Spline, a2: f32) = super::native_spline_point(a0, a1, a2);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32 { super::hsdIsDescendantOf(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut hsdObj: HSD_ClassInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `MTXMultVec(m, src, dst)`; reuses the verified fused implementation.
pub unsafe fn native_mtx_mult_vec(m: *mut [f32; 4], src: *mut Vec3, dst: *mut Vec3) {
    melee_core::ct!("native_mtx_mult_vec");
    let m = core::slice::from_raw_parts(m, 3);
    let v = crate::mtx::mult_vec(m, &melee_core::vec::Vec3 { x: (*src).x, y: (*src).y, z: (*src).z });
    *dst = Vec3 { x: v.x, y: v.y, z: v.z };
}
/// TODO: `splArcLengthPoint` (spline.c) has no native-pointer port yet.
pub unsafe fn native_spline_point(_: *mut Vec3, _: *mut HSD_Spline, _: f32) {
    melee_core::ct!("native_spline_point");
    todo!("splArcLengthPoint (spline.c)")
}

/// jobj.h `HSD_JObjSetupMatrix` (inline): rebuild when dirty and not user-defined.
unsafe fn jobj_setup_matrix(j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    let fl = (*j).flags;
    if fl & JOBJ_USER_DEF_MTX == 0 && fl & (JOBJ_MTX_DIRTY as u32) != 0 {
        ext::HSD_JObjSetupMatrixSub(j);
    }
}

/// The `if (flags & 1) { ... }` prologue shared by the position setters/getter.
unsafe fn resolve_world_pos(w: *mut HSD_WObj) {
    if (*w).flags & 1 != 0 {
        if !(*w).aobj.is_null() && !(*(*w).aobj).hsd_obj.is_null() {
            let jp = (*(*w).aobj).hsd_obj as *mut HSD_JObj;
            jobj_setup_matrix(jp);
            let pos = addr_of_mut!((*w).pos);
            ext::PSMTXMultVec((*jp).mtx.as_mut_ptr().cast(), pos, pos);
        }
        (*w).flags &= 0xFFFF_FFFE;
    }
}

pub unsafe fn HSD_WObjRemoveAnim(wobj: *mut HSD_WObj) {
    melee_core::ct!("HSD_WObjRemoveAnim");
    if !wobj.is_null() {
        ext::HSD_AObjRemove((*wobj).aobj);
        (*wobj).aobj = null_mut();
        ext::HSD_RObjRemoveAnimAll((*wobj).robj);
    }
}

pub unsafe fn HSD_WObjReqAnim(wobj: *mut HSD_WObj, frame: f32) {
    melee_core::ct!("HSD_WObjReqAnim");
    if !wobj.is_null() {
        ext::HSD_AObjReqAnim((*wobj).aobj, frame);
        ext::HSD_RObjReqAnimAll((*wobj).robj, frame);
    }
}

pub unsafe fn HSD_WObjAddAnim(wobj: *mut HSD_WObj, anim: *mut HSD_WObjAnim) {
    melee_core::ct!("HSD_WObjAddAnim");
    if !wobj.is_null() && !anim.is_null() {
        if !(*wobj).aobj.is_null() {
            ext::HSD_AObjRemove((*wobj).aobj);
        }
        (*wobj).aobj = ext::HSD_AObjLoadDesc((*anim).aobjdesc);
        ext::HSD_RObjAddAnimAll((*wobj).robj, (*anim).robjanim);
    }
}

pub unsafe extern "C" fn WObjUpdateFunc(obj: *mut c_void, ty: i32, val: *mut HSD_ObjData) {
    melee_core::ct!("WObjUpdateFunc");
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
            ext::splArcLengthPoint(&mut p, (*jp).u.spline, (*val).fv);
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
    melee_core::ct!("HSD_WObjInterpretAnim");
    if !wobj.is_null() {
        ext::HSD_AObjInterpretAnim((*wobj).aobj, wobj.cast(), Some(WObjUpdateFunc));
        ext::HSD_RObjAnimAll((*wobj).robj);
    }
}

pub unsafe extern "C" fn WObjLoad(wobj: *mut HSD_WObj, desc: *mut HSD_WObjDesc) -> i32 {
    melee_core::ct!("WObjLoad");
    HSD_WObjSetPosition(wobj, addr_of_mut!((*desc).pos));
    if !(*wobj).robj.is_null() {
        ext::HSD_RObjRemoveAll((*wobj).robj);
    }
    (*wobj).robj = ext::HSD_RObjLoadDesc((*desc).robjdesc);
    ext::HSD_RObjResolveRefsAll((*wobj).robj, (*desc).robjdesc);
    0
}

pub unsafe fn HSD_WObjInit(wobj: *mut HSD_WObj, desc: *mut HSD_WObjDesc) {
    melee_core::ct!("HSD_WObjInit");
    if wobj.is_null() || desc.is_null() {
        return;
    }
    HSD_WObjSetPosition(wobj, addr_of_mut!((*desc).pos));
    if !(*wobj).robj.is_null() {
        ext::HSD_RObjRemoveAll((*wobj).robj);
    }
    (*wobj).robj = ext::HSD_RObjLoadDesc((*desc).robjdesc);
    ext::HSD_RObjResolveRefsAll((*wobj).robj, (*desc).robjdesc);
}

pub unsafe fn HSD_WObjSetDefaultClass(info: *mut HSD_ClassInfo) {
    melee_core::ct!("HSD_WObjSetDefaultClass");
    if !info.is_null() {
        assert!(
            ext::hsdIsDescendantOf(info.cast(), addr_of_mut!(hsdWObj).cast()) != 0,
            "wobj.c:221 hsdIsDescendantOf(info, &hsdWObj)"
        );
    }
    default_class = info;
}

pub unsafe fn HSD_WObjLoadDesc(desc: *mut HSD_WObjDesc) -> *mut HSD_WObj {
    melee_core::ct!("HSD_WObjLoadDesc");
    if desc.is_null() {
        return null_mut();
    }
    let mut info: *mut HSD_ClassInfo = null_mut();
    let use_alloc = (*desc).class_name.is_null() || {
        info = ext::hsdSearchClassInfo((*desc).class_name);
        info.is_null()
    };
    let wobj: *mut HSD_WObj = if use_alloc {
        HSD_WObjAlloc()
    } else {
        let w = ext::hsdNew(info) as *mut HSD_WObj;
        assert!(!w.is_null(), "wobj.c:252 wobj");
        w
    };
    let m = (*wobj).parent.parent.class_info as *mut HSD_WObjInfo;
    ((*m).load.expect("wobj load"))(wobj, desc);
    wobj
}

pub unsafe fn HSD_WObjSetPosition(wobj: *mut HSD_WObj, pos: *mut Vec3) {
    melee_core::ct!("HSD_WObjSetPosition");
    if wobj.is_null() || pos.is_null() {
        return;
    }
    (*wobj).pos = *pos;
    (*wobj).flags |= 0x2;
    (*wobj).flags &= 0xFFFF_FFFE;
}

pub unsafe fn HSD_WObjSetPositionX(wobj: *mut HSD_WObj, val: f32) {
    melee_core::ct!("HSD_WObjSetPositionX");
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.x = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjSetPositionY(wobj: *mut HSD_WObj, val: f32) {
    melee_core::ct!("HSD_WObjSetPositionY");
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.y = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjSetPositionZ(wobj: *mut HSD_WObj, val: f32) {
    melee_core::ct!("HSD_WObjSetPositionZ");
    if !wobj.is_null() {
        resolve_world_pos(wobj);
        (*wobj).pos.z = val;
        (*wobj).flags |= 0x2;
    }
}

pub unsafe fn HSD_WObjGetPosition(wobj: *mut HSD_WObj, vec: *mut Vec3) {
    melee_core::ct!("HSD_WObjGetPosition");
    if wobj.is_null() || vec.is_null() {
        return;
    }
    resolve_world_pos(wobj);
    *vec = (*wobj).pos;
}

pub unsafe fn HSD_WObjAlloc() -> *mut HSD_WObj {
    melee_core::ct!("HSD_WObjAlloc");
    let info = if default_class.is_null() { addr_of_mut!(hsdWObj).cast::<HSD_ClassInfo>() } else { default_class };
    let wobj = ext::hsdNew(info) as *mut HSD_WObj;
    assert!(!wobj.is_null(), "wobj.c:591 wobj");
    wobj
}

pub unsafe extern "C" fn WObjRelease(o: *mut HSD_Class) {
    melee_core::ct!("WObjRelease");
    let wobj = o as *mut HSD_WObj;
    ext::HSD_RObjRemoveAll((*wobj).robj);
    ext::HSD_AObjRemove((*wobj).aobj);
    let parent = hsdWObj.parent.parent.head.parent;
    ((*parent).release.expect("parent release"))(o);
}

pub unsafe extern "C" fn WObjAmnesia(info: *mut HSD_ClassInfo) {
    melee_core::ct!("WObjAmnesia");
    if info == default_class {
        default_class = null_mut();
    }
    let parent = hsdWObj.parent.parent.head.parent;
    ((*parent).amnesia.expect("parent amnesia"))(info);
}

pub unsafe extern "C" fn WObjInfoInit() {
    melee_core::ct!("WObjInfoInit");
    ext::hsdInitClassInfo(
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
    melee_core::ct!("HSD_WObjUnref");
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
    melee_core::ct!("HSD_WObjClearFlags");
    (*wobj).flags &= !flags;
}
