//! Pointer-native ports of the `src/sysdolphin/baselib/jobj.c` entry points the character code
//! calls on raw `HSD_JObj*` (the arena ports in `tu::jobj` / `jobj_rt` work on `JObjTree` ids and
//! cannot be called with game pointers): matrix-dirty propagation, the flag/DObj accessors, the
//! `JObjUpdateFunc` animation sink and the `Req/Add/Anim` tree walkers, plus the `jobj.h` inlines
//! (`HSD_JObjSetTranslate`, `HSD_JObjSetRotationZ`, `HSD_JObjGetMtxPtr`).
//!
//! Cross-TU callees that have no pointer-native port yet (robj.c's animation walkers, the
//! spline point) go through `ext` hooks that are only reached when the JObj really has an RObj /
//! a spline path.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

const MTX_DIRTY: u32 = JOBJ_MTX_DIRTY as u32;
const USER_DEF_MTX: u32 = JOBJ_USER_DEF_MTX as u32;
const INDEP_PARENT: u32 = JOBJ_MTX_INDEP_PARENT as u32;
const INDEP_SRT: u32 = JOBJ_MTX_INDEP_SRT as u32;
const INSTANCE: u32 = JOBJ_INSTANCE as u32;
const HIDDEN: u32 = JOBJ_HIDDEN as u32;
const CLASSICAL_SCALE: u32 = JOBJ_CLASSICAL_SCALE as u32;
const PTCL: u32 = JOBJ_PTCL as u32;
const SPLINE: u32 = JOBJ_SPLINE as u32;

/// Callees without a pointer-native port. Unset, they panic with the C name.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($name:ident, $slot:ident, ($($a:ident:$ty:ty),*) $(-> $ret:ty)?) => {
            pub static mut $slot: Option<unsafe fn($($ty),*) $(-> $ret)?> = None;
            pub unsafe fn $name($($a:$ty),*) $(-> $ret)? {
                ($slot.expect(concat!("TODO cross-TU: ", stringify!($name))))($($a),*)
            }
        };
    }
    hook!(splArcLengthPoint, SPL_ARC, (out: *mut Vec, spline: *mut HSD_Spline, s: f32));
    hook!(HSD_RObjAnimAll, ROBJ_ANIM_ALL, (r: *mut HSD_RObj));
    hook!(HSD_RObjAddAnimAll, ROBJ_ADD_ANIM_ALL, (r: *mut HSD_RObj, a: *mut HSD_RObjAnimJoint));
    hook!(HSD_RObjReqAnimAllByFlags, ROBJ_REQ, (r: *mut HSD_RObj, frame: f32, flags: u32));
    hook!(HSD_RObjRemoveAnimAllByFlags, ROBJ_REMOVE_ANIM, (r: *mut HSD_RObj, flags: u32));
    hook!(HSD_RObjGetByType, ROBJ_BY_TYPE, (r: *mut HSD_RObj, ty: u32, x: u32) -> *mut HSD_RObj);
}

/// `ufc_callback`: (jobj, type, value).
pub type UfcCallback = unsafe extern "C" fn(*mut HSD_JObj, u32, f32);
pub static mut ufc_callbacks_ptr: *mut _HSD_SList = null_mut();
pub static mut dptcl_callback: Option<unsafe extern "C" fn(i32, i32, i32, *mut HSD_JObj)> = None;
pub static mut jsound_callback: Option<unsafe extern "C" fn(i32)> = None;
pub static mut ptcltgt_callback: Option<unsafe extern "C" fn(*mut HSD_JObj, i32)> = None;

#[inline]
unsafe fn union_type_dobj(j: *mut HSD_JObj) -> bool {
    (*j).flags & (PTCL | SPLINE) == 0
}

/// `HSD_JObjMtxIsDirty` (jobj.h).
#[inline]
pub unsafe fn HSD_JObjMtxIsDirty(j: *mut HSD_JObj) -> bool {
    melee_core::ct!("HSD_JObjMtxIsDirty");
    (*j).flags & USER_DEF_MTX == 0 && (*j).flags & MTX_DIRTY != 0
}

/// `HSD_JObjSetMtxDirty` (jobj.h macro).
#[inline]
pub unsafe fn HSD_JObjSetMtxDirty(j: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetMtxDirty");
    if !j.is_null() && !HSD_JObjMtxIsDirty(j) {
        HSD_JObjSetMtxDirtySub(j);
    }
}

pub unsafe fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetMtxDirtySub");
    (*jobj).flags |= 0x40;
    if (*jobj).flags & INSTANCE == 0 {
        let mut child = (*jobj).child;
        while !child.is_null() {
            if (*child).flags & INDEP_PARENT == 0 && !HSD_JObjMtxIsDirty(child) {
                HSD_JObjSetMtxDirtySub(child);
            }
            child = (*child).next;
        }
    }
}

pub unsafe fn HSD_JObjCheckDepend(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjCheckDepend");
    if jobj.is_null() {
        return;
    }
    if !HSD_JObjMtxIsDirty(jobj) {
        let f = (*jobj).flags;
        let parent = (*jobj).parent;
        let eff = f & (JOBJ_EFFECTOR as u32);
        if f & USER_DEF_MTX != 0 {
            if f & INDEP_PARENT == 0 && !parent.is_null() && HSD_JObjMtxIsDirty(parent) {
                (*jobj).flags |= MTX_DIRTY;
            }
        } else if (!parent.is_null() && (*parent).flags & MTX_DIRTY != 0)
            || eff == JOBJ_JOINT1 as u32
            || eff == JOBJ_JOINT2 as u32
            || eff == JOBJ_EFFECTOR as u32
            || !(*jobj).robj.is_null()
        {
            (*jobj).flags |= MTX_DIRTY;
        }
    }
}

pub unsafe fn HSD_JObjSetFlags(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjSetFlags");
    if !jobj.is_null() {
        if ((*jobj).flags ^ flags) & CLASSICAL_SCALE != 0 {
            HSD_JObjSetMtxDirty(jobj);
        }
        (*jobj).flags |= flags;
    }
}

pub unsafe fn HSD_JObjSetFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjSetFlagsAll");
    if !jobj.is_null() {
        HSD_JObjSetFlags(jobj, flags);
        if (*jobj).flags & INSTANCE == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjSetFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

pub unsafe fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjClearFlags");
    if !jobj.is_null() {
        if ((*jobj).flags ^ flags) & CLASSICAL_SCALE != 0 {
            HSD_JObjSetMtxDirty(jobj);
        }
        (*jobj).flags &= !flags;
    }
}

pub unsafe fn HSD_JObjClearFlagsAll(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjClearFlagsAll");
    if !jobj.is_null() {
        HSD_JObjClearFlags(jobj, flags);
        if (*jobj).flags & INSTANCE == 0 {
            let mut i = (*jobj).child;
            while !i.is_null() {
                HSD_JObjClearFlagsAll(i, flags);
                i = (*i).next;
            }
        }
    }
}

pub unsafe fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    melee_core::ct!("HSD_JObjGetDObj");
    if jobj.is_null() || !union_type_dobj(jobj) {
        return null_mut();
    }
    (*jobj).u.dobj
}

pub unsafe fn HSD_JObjAddDObj(jobj: *mut HSD_JObj, dobj: *mut HSD_DObj) {
    melee_core::ct!("HSD_JObjAddDObj");
    if jobj.is_null() || dobj.is_null() || !union_type_dobj(jobj) {
        return;
    }
    (*dobj).next = (*jobj).u.dobj;
    (*jobj).u.dobj = dobj;
}

// ---- jobj.h inlines -------------------------------------------------------------------------

/// `HSD_JObjSetTranslate` (jobj.h): the asserts are debug-only.
pub unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *const Vec) {
    melee_core::ct!("HSD_JObjSetTranslate");
    (*jobj).translate = *translate;
    if (*jobj).flags & INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn HSD_JObjSetRotationX(jobj: *mut HSD_JObj, x: f32) {
    melee_core::ct!("HSD_JObjSetRotationX");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "jobj.h: !(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.x = x;
    if (*jobj).flags & INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}
pub unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    melee_core::ct!("HSD_JObjSetRotationY");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "jobj.h: !(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}
pub unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    melee_core::ct!("HSD_JObjSetRotationZ");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "jobj.h: !(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.z = z;
    if (*jobj).flags & INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetMtxPtr` (jobj.h inline).
pub unsafe fn HSD_JObjGetMtxPtr(jobj: *mut HSD_JObj) -> *mut c_void {
    melee_core::ct!("HSD_JObjGetMtxPtr");
    assert!(!jobj.is_null(), "jobj.h:1144 jobj");
    super::jobj::HSD_JObjSetupMatrix(jobj);
    (&raw mut (*jobj).mtx) as *mut c_void
}

/// `HSD_JObjCopyMtx` (jobj.h inline): `PSMTXCopy(mtx, jobj->mtx)`.
pub unsafe fn HSD_JObjCopyMtx(jobj: *mut HSD_JObj, mtx: *const c_void) {
    melee_core::ct!("HSD_JObjCopyMtx");
    assert!(!jobj.is_null(), "jobj.h jobj");
    assert!(!mtx.is_null(), "jobj.h mtx");
    (*jobj).mtx = *(mtx as *const Mtx);
}

// ---- animation --------------------------------------------------------------------------------

#[inline]
unsafe fn set_trans(j: *mut HSD_JObj, axis: u8, v: f32) {
    match axis {
        0 => (*j).translate.x = v,
        1 => (*j).translate.y = v,
        _ => (*j).translate.z = v,
    }
    if (*j).flags & INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(j);
    }
}

/// `JObjUpdateFunc`.
pub unsafe extern "C" fn JObjUpdateFunc(obj: *mut c_void, ty: i32, val: *mut HSD_ObjData) {
    melee_core::ct!("JObjUpdateFunc");
    use crate::hsd_mtx::{fabsf_bitwise, get_rotation, get_scale, get_translate, inverse_concat};
    let jobj = obj as *mut HSD_JObj;
    if jobj.is_null() {
        return;
    }
    match ty {
        4 => {
            // HSD_A_J_PATH
            if ((*val).fv as f64) < 0.0 {
                (*val).fv = 0.0;
            }
            if 1.0 < (*val).fv as f64 {
                (*val).fv = 1.0;
            }
            assert!(!(*jobj).aobj.is_null(), "jobj.c:0x24B jobj->aobj");
            let jp = (*(*jobj).aobj).hsd_obj as *mut HSD_JObj;
            assert!(!jp.is_null(), "jobj.c:0x24D jp");
            assert!(!(*jp).u.spline.is_null(), "jobj.c:0x24E jp->u.spline");
            let mut p: Vec = core::mem::zeroed();
            ext::splArcLengthPoint(&mut p, (*jp).u.spline, (*val).fv);
            set_trans(jobj, 0, p.x);
            set_trans(jobj, 1, p.y);
            set_trans(jobj, 2, p.z);
        }
        1 => {
            // HSD_A_J_ROTX
            if (*jobj).flags & JOBJ_JOINT1 as u32 != 0 {
                let robj = ext::HSD_RObjGetByType((*jobj).robj, 0x4000_0000, 0);
                if !robj.is_null() {
                    (*robj).u.ik_hint.rotate_x = (*val).fv;
                }
            }
            HSD_JObjSetRotationX(jobj, (*val).fv);
        }
        2 => HSD_JObjSetRotationY(jobj, (*val).fv),
        3 => HSD_JObjSetRotationZ(jobj, (*val).fv),
        5 => set_trans(jobj, 0, (*val).fv),
        6 => set_trans(jobj, 1, (*val).fv),
        7 => set_trans(jobj, 2, (*val).fv),
        8 | 9 | 10 => {
            if fabsf_bitwise((*val).fv) < 1e-3 {
                (*val).fv = 1e-3;
            }
            match ty {
                8 => (*jobj).scale.x = (*val).fv,
                9 => (*jobj).scale.y = (*val).fv,
                _ => (*jobj).scale.z = (*val).fv,
            }
            if (*jobj).flags & INDEP_SRT == 0 {
                HSD_JObjSetMtxDirty(jobj);
            }
        }
        12 => {
            if (*val).fv as f64 > 0.5 {
                HSD_JObjClearFlagsAll(jobj, HIDDEN);
            } else {
                HSD_JObjSetFlagsAll(jobj, HIDDEN);
            }
        }
        11 => {
            if (*val).fv as f64 > 0.5 {
                HSD_JObjClearFlags(jobj, HIDDEN);
            } else {
                HSD_JObjSetFlags(jobj, HIDDEN);
            }
        }
        20..=29 | 30..=39 => {
            let arg = if ty < 30 { (*val).iv as f32 } else { (*val).fv };
            let mut node = ufc_callbacks_ptr;
            while !node.is_null() {
                let cb: UfcCallback = core::mem::transmute((*node).data);
                cb(jobj, ty as u32, arg);
                node = (*node).next;
            }
        }
        0x28 => {
            let iv = (*val).iv;
            let lo = iv & 0x3F;
            let hi = (iv >> 6) & 0xFF_FFFF;
            if let Some(cb) = dptcl_callback {
                cb(0, lo, hi, jobj);
            }
        }
        0x29 => {
            if let Some(cb) = jsound_callback {
                cb((*val).iv);
            }
        }
        0x2A => {
            if let Some(cb) = ptcltgt_callback {
                cb(jobj, (*val).iv);
            }
        }
        0x32..=0x35 => {
            let col = (ty - 0x32) as usize;
            let p = (*val).p;
            (*jobj).mtx[0][col] = p.x;
            (*jobj).mtx[1][col] = p.y;
            (*jobj).mtx[2][col] = p.z;
        }
        0x36..=0x39 => {
            let m = if !(*jobj).parent.is_null() {
                inverse_concat(&(*(*jobj).parent).mtx, &(*jobj).mtx)
            } else {
                (*jobj).mtx
            };
            if ty == 0x36 || ty == 0x38 {
                let t = get_translate(&m);
                (*jobj).translate = Vec { x: t.x, y: t.y, z: t.z };
            }
            if ty == 0x36 || ty == 0x37 {
                let r = get_rotation(&m);
                (*jobj).rotate.x = r.x;
                (*jobj).rotate.y = r.y;
                (*jobj).rotate.z = r.z;
            }
            if ty == 0x36 || ty == 0x39 {
                let s = get_scale(&m);
                (*jobj).scale = Vec { x: s.x, y: s.y, z: s.z };
            }
        }
        _ => {}
    }
}

pub unsafe fn HSD_JObjAnim(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjAnim");
    if jobj.is_null() {
        return;
    }
    HSD_JObjCheckDepend(jobj);
    super::aobj::HSD_AObjInterpretAnim((*jobj).aobj, jobj as *mut c_void, Some(JObjUpdateFunc));
    if !(*jobj).robj.is_null() {
        ext::HSD_RObjAnimAll((*jobj).robj);
    }
    if union_type_dobj(jobj) {
        super::dobj::hsd::HSD_DObjAnimAll((*jobj).u.dobj);
    }
}

unsafe fn JObjAnimAll(jobj: *mut HSD_JObj) {
    if jobj.is_null() {
        return;
    }
    HSD_JObjAnim(jobj);
    if (*jobj).flags & INSTANCE == 0 {
        let mut c = (*jobj).child;
        while !c.is_null() {
            JObjAnimAll(c);
            c = (*c).next;
        }
    }
}

pub unsafe fn HSD_JObjAnimAll(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjAnimAll");
    if !jobj.is_null() {
        super::aobj::HSD_AObjInitEndCallBack();
        JObjAnimAll(jobj);
        super::aobj::HSD_AObjInvokeCallBacks();
    }
}

pub unsafe fn HSD_JObjReqAnimByFlags(jobj: *mut HSD_JObj, flags: u32, frame: f32) {
    melee_core::ct!("HSD_JObjReqAnimByFlags");
    if jobj.is_null() {
        return;
    }
    if flags & 1 != 0 {
        super::aobj::HSD_AObjReqAnim((*jobj).aobj, frame);
    }
    if union_type_dobj(jobj) {
        super::dobj::hsd::HSD_DObjReqAnimAllByFlags((*jobj).u.dobj, frame, flags);
    }
    if !(*jobj).robj.is_null() {
        ext::HSD_RObjReqAnimAllByFlags((*jobj).robj, frame, flags);
    }
}

pub unsafe fn HSD_JObjReqAnimAllByFlags(jobj: *mut HSD_JObj, flags: u32, frame: f32) {
    melee_core::ct!("HSD_JObjReqAnimAllByFlags");
    if jobj.is_null() {
        return;
    }
    HSD_JObjReqAnimByFlags(jobj, flags, frame);
    if (*jobj).flags & INSTANCE == 0 {
        let mut c = (*jobj).child;
        while !c.is_null() {
            HSD_JObjReqAnimAllByFlags(c, flags, frame);
            c = (*c).next;
        }
    }
}

pub unsafe fn HSD_JObjReqAnimAll(jobj: *mut HSD_JObj, frame: f32) {
    melee_core::ct!("HSD_JObjReqAnimAll");
    HSD_JObjReqAnimAllByFlags(jobj, 0x7FF, frame);
}

pub unsafe fn HSD_JObjRemoveAnimByFlags(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjRemoveAnimByFlags");
    if jobj.is_null() {
        return;
    }
    if flags & 1 != 0 {
        super::aobj::HSD_AObjRemove((*jobj).aobj);
        (*jobj).aobj = null_mut();
    }
    if union_type_dobj(jobj) {
        super::dobj::hsd::HSD_DObjRemoveAnimAllByFlags((*jobj).u.dobj, flags);
    }
    if !(*jobj).robj.is_null() {
        ext::HSD_RObjRemoveAnimAllByFlags((*jobj).robj, flags);
    }
}

pub unsafe fn HSD_JObjRemoveAnimAllByFlags(jobj: *mut HSD_JObj, flags: u32) {
    melee_core::ct!("HSD_JObjRemoveAnimAllByFlags");
    if jobj.is_null() {
        return;
    }
    HSD_JObjRemoveAnimByFlags(jobj, flags);
    if (*jobj).flags & INSTANCE == 0 {
        let mut c = (*jobj).child;
        while !c.is_null() {
            HSD_JObjRemoveAnimAllByFlags(c, flags);
            c = (*c).next;
        }
    }
}

pub unsafe fn HSD_JObjRemoveAnim(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjRemoveAnim");
    HSD_JObjRemoveAnimByFlags(jobj, 0x7FF);
}

pub unsafe fn HSD_JObjRemoveAnimAll(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjRemoveAnimAll");
    HSD_JObjRemoveAnimAllByFlags(jobj, 0x7FF);
}

/// `HSD_JObjSetupMatrixSub`: rebuild this joint's matrix (and its dirty ancestors') over the native tree bridge.
pub unsafe fn HSD_JObjSetupMatrixSub(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjSetupMatrixSub");
    super::jobj::native_setup_sub(jobj);
}

/// `HSD_JObjMakeMatrix`: the default class's matrix build.
pub unsafe fn HSD_JObjMakeMatrix(jobj: *mut HSD_JObj) {
    melee_core::ct!("HSD_JObjMakeMatrix");
    super::jobj::native_make_matrix_pub(jobj);
}

/// `JObjSortAnim`: the first JOBJ-type track moves to the front.
pub unsafe fn JObjSortAnim(aobj: *mut HSD_AObj) {
    melee_core::ct!("JObjSortAnim");
    if aobj.is_null() || (*aobj).fobj.is_null() {
        return;
    }
    let mut fobj_ptr: *mut *mut HSD_FObj = &mut (*aobj).fobj;
    while !(*fobj_ptr).is_null() {
        let fobj = *fobj_ptr;
        if (*fobj).obj_type == 12 {
            *fobj_ptr = (*fobj).next;
            (*fobj).next = (*aobj).fobj;
            (*aobj).fobj = fobj;
            break;
        }
        fobj_ptr = &mut (*fobj).next;
    }
}

pub unsafe fn HSD_JObjAddAnim(
    jobj: *mut HSD_JObj,
    an_joint: *mut HSD_AnimJoint,
    mat_joint: *mut HSD_MatAnimJoint,
    sh_joint: *mut HSD_ShapeAnimJoint,
) {
    melee_core::ct!("HSD_JObjAddAnim");
    if jobj.is_null() {
        return;
    }
    if !an_joint.is_null() {
        if !(*jobj).aobj.is_null() {
            super::aobj::HSD_AObjRemove((*jobj).aobj);
        }
        (*jobj).aobj = super::aobj::HSD_AObjLoadDesc((*an_joint).aobjdesc);
        JObjSortAnim((*jobj).aobj);
        if !(*jobj).robj.is_null() {
            ext::HSD_RObjAddAnimAll((*jobj).robj, (*an_joint).robj_anim);
        }
        if (*an_joint).flags & 1 != 0 {
            HSD_JObjSetFlags(jobj, CLASSICAL_SCALE);
        } else {
            HSD_JObjClearFlags(jobj, CLASSICAL_SCALE);
        }
    }
    if union_type_dobj(jobj) {
        super::dobj::hsd::HSD_DObjAddAnimAll(
            (*jobj).u.dobj,
            if mat_joint.is_null() { null_mut() } else { (*mat_joint).matanim as *mut HSD_MatAnim },
            if sh_joint.is_null() { null_mut() } else { (*sh_joint).shapeanimdobj },
        );
    }
}

pub unsafe fn HSD_JObjAddAnimAll(
    jobj: *mut HSD_JObj,
    ajoint: *mut HSD_AnimJoint,
    mjoint: *mut HSD_MatAnimJoint,
    sjoint: *mut HSD_ShapeAnimJoint,
) {
    melee_core::ct!("HSD_JObjAddAnimAll");
    if jobj.is_null() {
        return;
    }
    HSD_JObjAddAnim(jobj, ajoint, mjoint, sjoint);
    if (*jobj).flags & INSTANCE == 0 {
        let mut jp = (*jobj).child;
        let mut aj = if ajoint.is_null() { null_mut() } else { (*ajoint).child };
        let mut mj = if mjoint.is_null() { null_mut() } else { (*mjoint).child };
        let mut sj = if sjoint.is_null() { null_mut() } else { (*sjoint).child };
        while !jp.is_null() {
            HSD_JObjAddAnimAll(jp, aj, mj, sj);
            jp = (*jp).next;
            aj = if aj.is_null() { null_mut() } else { (*aj).next };
            mj = if mj.is_null() { null_mut() } else { (*mj).next };
            sj = if sj.is_null() { null_mut() } else { (*sj).next };
        }
    }
}

/// `HSD_JObjWalkTreeCallback`.
pub type JObjWalkTreeCallback = Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut f32, i32)>;

/// `HSD_JObjWalkTree0` (baselib/jobj.c): `cb(jobj, args, 1 or 2)` for `jobj` (1 = first child of its parent), then its subtree.
pub unsafe fn HSD_JObjWalkTree0_ptr(jobj: *mut HSD_JObj, cb: JObjWalkTreeCallback, cb_args: *mut *mut f32) {
    melee_core::ct!("HSD_JObjWalkTree0_ptr");
    if jobj.is_null() {
        return;
    }
    assert!(!(*jobj).parent.is_null(), "jobj.c:174: jobj->parent"); // HSD_ASSERT(0xAE, jobj->parent)
    let ty = if (*(*jobj).parent).child == jobj { 1 } else { 2 };
    if let Some(f) = cb {
        f(jobj, cb_args, ty);
    }
    if (*jobj).flags & INSTANCE == 0 {
        let mut child = (*jobj).child;
        while !child.is_null() {
            HSD_JObjWalkTree0_ptr(child, cb, cb_args);
            child = (*child).next;
        }
    }
}

/// `HSD_JObjWalkTree` (baselib/jobj.c): `cb(jobj, args, 0)` for the root, then the subtrees of its children.
pub unsafe fn HSD_JObjWalkTree_ptr(jobj: *mut HSD_JObj, cb: JObjWalkTreeCallback, cb_args: *mut *mut f32) {
    melee_core::ct!("HSD_JObjWalkTree_ptr");
    if jobj.is_null() {
        return;
    }
    if let Some(f) = cb {
        f(jobj, cb_args, 0);
    }
    if (*jobj).flags & INSTANCE == 0 {
        let mut child = (*jobj).child;
        while !child.is_null() {
            HSD_JObjWalkTree0_ptr(child, cb, cb_args);
            child = (*child).next;
        }
    }
}
