//! Port of src/melee/lb/lb_00B0.c: JObj / DObj / RObj / AnimJoint helpers (tree walks, SRT copy and blend,
//! animation rate setters, constraint ROBj helpers, free TEV register search, small table readers).
//! The header inlines of jobj.h / lobj.h this file uses are ported here; callees of unported TUs go
//! through `ext` (unset: panics "TODO"). Every contraction of the shipping build is explicit
//! (`fmadds`). Checked against the C (whole TU, shipping contraction) by tests/lbtu_lb_00B0.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut EulerToQuat: Option<fn(*mut Vec, *mut Quaternion) -> i32> = None;
        pub static mut HSD_AObjReqAnim: Option<fn(*mut HSD_AObj, f32)> = None;
        pub static mut HSD_AObjSetRate: Option<fn(*mut HSD_AObj, f32)> = None;
        pub static mut HSD_JObjAddAnimAll: Option<fn(*mut HSD_JObj, *mut HSD_AnimJoint, *mut HSD_MatAnimJoint, *mut HSD_ShapeAnimJoint)> = None;
        pub static mut HSD_JObjClearFlags: Option<fn(*mut HSD_JObj, u32)> = None;
        pub static mut HSD_JObjDeleteRObj: Option<fn(*mut HSD_JObj, *mut HSD_RObj)> = None;
        pub static mut HSD_JObjGetFlags: Option<fn(*mut HSD_JObj) -> u32> = None;
        pub static mut HSD_JObjPrependRObj: Option<fn(*mut HSD_JObj, *mut HSD_RObj)> = None;
        pub static mut HSD_JObjSetFlags: Option<fn(*mut HSD_JObj, u32)> = None;
        pub static mut HSD_JObjSetMtxDirtySub: Option<fn(*mut HSD_JObj)> = None;
        pub static mut HSD_JObjSetupMatrixSub: Option<fn(*mut HSD_JObj)> = None;
        pub static mut HSD_JObjWalkTree: Option<fn(*mut HSD_JObj, Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut f32, i32)>, *mut *mut f32)> = None;
        pub static mut HSD_LObjGetFlags: Option<fn(*mut HSD_LObj) -> u32> = None;
        pub static mut HSD_QuatLib_8037EF28: Option<fn(*mut Quaternion, *mut Quaternion, *mut Quaternion, f32) -> i32> = None;
        pub static mut HSD_RObjAlloc: Option<fn() -> *mut HSD_RObj> = None;
        pub static mut HSD_RObjGetByType: Option<fn(*mut HSD_RObj, u32, u32) -> *mut HSD_RObj> = None;
        pub static mut HSD_RObjRemove: Option<fn(*mut HSD_RObj)> = None;
        pub static mut HSD_RObjSetConstraintObj: Option<fn(*mut HSD_RObj, *mut c_void)> = None;
        pub static mut HSD_RObjSetFlags: Option<fn(*mut HSD_RObj, u32)> = None;
        pub static mut PSMTXMultVec: Option<fn(*mut [f32; 4], *mut Vec, *mut Vec)> = None;
    }
    pub fn EulerToQuat(a0: *mut Vec, a1: *mut Quaternion) -> i32 {
        match unsafe { hook::EulerToQuat } { Some(f) => f(a0, a1), None => { panic!("TODO: EulerToQuat is not ported") } }
    }
    pub fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32) {
        match unsafe { hook::HSD_AObjReqAnim } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_AObjReqAnim is not ported") } }
    }
    pub fn HSD_AObjSetRate(a0: *mut HSD_AObj, a1: f32) {
        match unsafe { hook::HSD_AObjSetRate } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_AObjSetRate is not ported") } }
    }
    pub fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint) {
        match unsafe { hook::HSD_JObjAddAnimAll } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: HSD_JObjAddAnimAll is not ported") } }
    }
    pub fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32) {
        match unsafe { hook::HSD_JObjClearFlags } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_JObjClearFlags is not ported") } }
    }
    pub fn HSD_JObjDeleteRObj(a0: *mut HSD_JObj, a1: *mut HSD_RObj) {
        match unsafe { hook::HSD_JObjDeleteRObj } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_JObjDeleteRObj is not ported") } }
    }
    pub fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 {
        match unsafe { hook::HSD_JObjGetFlags } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjGetFlags is not ported") } }
    }
    pub fn HSD_JObjPrependRObj(a0: *mut HSD_JObj, a1: *mut HSD_RObj) {
        match unsafe { hook::HSD_JObjPrependRObj } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_JObjPrependRObj is not ported") } }
    }
    pub fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32) {
        match unsafe { hook::HSD_JObjSetFlags } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_JObjSetFlags is not ported") } }
    }
    pub fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) {
        match unsafe { hook::HSD_JObjSetMtxDirtySub } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjSetMtxDirtySub is not ported") } }
    }
    pub fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj) {
        match unsafe { hook::HSD_JObjSetupMatrixSub } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjSetupMatrixSub is not ported") } }
    }
    pub fn HSD_JObjWalkTree(a0: *mut HSD_JObj, a1: Option<unsafe extern "C" fn(*mut HSD_JObj, *mut *mut f32, i32)>, a2: *mut *mut f32) {
        match unsafe { hook::HSD_JObjWalkTree } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_JObjWalkTree is not ported") } }
    }
    pub fn HSD_LObjGetFlags(a0: *mut HSD_LObj) -> u32 {
        match unsafe { hook::HSD_LObjGetFlags } { Some(f) => f(a0), None => { panic!("TODO: HSD_LObjGetFlags is not ported") } }
    }
    pub fn HSD_QuatLib_8037EF28(a0: *mut Quaternion, a1: *mut Quaternion, a2: *mut Quaternion, a3: f32) -> i32 {
        match unsafe { hook::HSD_QuatLib_8037EF28 } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: HSD_QuatLib_8037EF28 is not ported") } }
    }
    pub fn HSD_RObjAlloc() -> *mut HSD_RObj {
        match unsafe { hook::HSD_RObjAlloc } { Some(f) => f(), None => { panic!("TODO: HSD_RObjAlloc is not ported") } }
    }
    pub fn HSD_RObjGetByType(a0: *mut HSD_RObj, a1: u32, a2: u32) -> *mut HSD_RObj {
        match unsafe { hook::HSD_RObjGetByType } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_RObjGetByType is not ported") } }
    }
    pub fn HSD_RObjRemove(a0: *mut HSD_RObj) {
        match unsafe { hook::HSD_RObjRemove } { Some(f) => f(a0), None => { panic!("TODO: HSD_RObjRemove is not ported") } }
    }
    pub fn HSD_RObjSetConstraintObj(a0: *mut HSD_RObj, a1: *mut c_void) {
        match unsafe { hook::HSD_RObjSetConstraintObj } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_RObjSetConstraintObj is not ported") } }
    }
    pub fn HSD_RObjSetFlags(a0: *mut HSD_RObj, a1: u32) {
        match unsafe { hook::HSD_RObjSetFlags } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_RObjSetFlags is not ported") } }
    }
    pub fn PSMTXMultVec(a0: *mut [f32; 4], a1: *mut Vec, a2: *mut Vec) {
        match unsafe { hook::PSMTXMultVec } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: PSMTXMultVec is not ported") } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const JOBJ_MTX_DIRTY: u32 = 1 << 6;
const JOBJ_INSTANCE: u32 = 1 << 12;
const JOBJ_USE_QUATERNION: u32 = 1 << 17;
const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
const JOBJ_MTX_INDEP_SRT: u32 = 1 << 25;
const AOBJ_REWINDED: u32 = 1 << 26;
const AOBJ_NO_ANIM: u32 = 1 << 30;
const POBJ_SHAPEANIM: u16 = 1 << 12;
const REFTYPE_JOBJ: u32 = 0x1000_0000;

/// `HSD_ASSERT(line, cond)`: the C build's `__assert` does not return.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("HSD_ASSERT {what}");
    }
}

/// `s32 lb_803B9FC0[]`
pub static lb_803B9FC0: [i32; 8] = [14, 14, 14, 14, 2, 4, 6, 0];
/// `s32 lb_803B9FE0[]`
pub static lb_803B9FE0: [i32; 4] = [12, 13, 14, 15];
/// `s32 lb_803B9FF0[]`
pub static lb_803B9FF0: [i32; 4] = [7, 4, 5, 6];
/// `s32 lb_803BA000[]`
pub static lb_803BA000: [i32; 8] = [6, 6, 6, 6, 1, 2, 3, 0];
/// `s32 lb_803BA020[]`
pub static lb_803BA020: [i32; 4] = [28, 29, 30, 31];
/// `s32 lb_803BA030[]`
pub static lb_803BA030: [i32; 4] = [7, 4, 5, 6];

// ---- jobj.h / lobj.h static inlines used by the file

#[inline]
unsafe fn HSD_JObjGetParent(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() { null_mut() } else { (*j).parent }
}

#[inline]
unsafe fn HSD_JObjGetRObj(j: *mut HSD_JObj) -> *mut HSD_RObj {
    hsd_assert(!j.is_null(), "(jobj)");
    (*j).robj
}

#[inline]
unsafe fn HSD_JObjMtxIsDirty(j: *mut HSD_JObj) -> bool {
    hsd_assert(!j.is_null(), "(jobj)");
    let mut result = false;
    if (*j).flags & JOBJ_USER_DEF_MTX == 0 && (*j).flags & JOBJ_MTX_DIRTY != 0 {
        result = true;
    }
    result
}

#[inline]
unsafe fn HSD_JObjSetupMatrix(j: *mut HSD_JObj) {
    if j.is_null() || !HSD_JObjMtxIsDirty(j) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(j);
}

/// The `HSD_JObjSetMtxDirty` macro.
#[inline]
unsafe fn HSD_JObjSetMtxDirty(j: *mut HSD_JObj) {
    if !j.is_null() && !HSD_JObjMtxIsDirty(j) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline]
unsafe fn HSD_JObjGetRotation(j: *mut HSD_JObj, rotate: *mut Quaternion) {
    hsd_assert(!j.is_null(), "(jobj)");
    hsd_assert(!rotate.is_null(), "(rotate)");
    *rotate = (*j).rotate;
}

#[inline]
unsafe fn HSD_JObjGetScale(j: *mut HSD_JObj, scale: *mut Vec) {
    hsd_assert(!j.is_null(), "(jobj)");
    hsd_assert(!scale.is_null(), "(scale)");
    *scale = (*j).scale;
}

#[inline]
unsafe fn HSD_JObjGetTranslation(j: *mut HSD_JObj, translate: *mut Vec) {
    hsd_assert(!j.is_null(), "(jobj)");
    hsd_assert(!translate.is_null(), "(translate)");
    *translate = (*j).translate;
}

#[inline]
unsafe fn HSD_LObjGetNext(l: *mut HSD_LObj) -> *mut HSD_LObj {
    if l.is_null() { null_mut() } else { (*l).next }
}

pub unsafe fn lb_8000B074(jobj: *mut HSD_JObj) -> bool {
    let aobj = (*jobj).aobj;
    !aobj.is_null() && (*aobj).flags & AOBJ_NO_ANIM == 0
}

/// The depth-first walk lb_8000B09C / lb_8000B134 share: true at the first joint whose AObj satisfies `hit`.
unsafe fn tree_any(mut jobj: *mut HSD_JObj, hit: impl Fn(u32) -> bool) -> bool {
    while !jobj.is_null() {
        if !(*jobj).aobj.is_null() && hit((*(*jobj).aobj).flags) {
            return true;
        }
        if ((*jobj).child.is_null() && (*jobj).next.is_null()) || (*jobj).flags & JOBJ_INSTANCE != 0 {
            loop {
                if (*jobj).parent.is_null() {
                    jobj = null_mut();
                    break;
                }
                if !(*(*jobj).parent).next.is_null() {
                    jobj = (*(*jobj).parent).next;
                    break;
                }
                jobj = (*jobj).parent;
            }
        } else if !(*jobj).child.is_null() {
            jobj = (*jobj).child;
        } else {
            jobj = (*jobj).next;
        }
    }
    false
}

pub unsafe fn lb_8000B09C(jobj: *mut HSD_JObj) -> bool {
    tree_any(jobj, |f| f & AOBJ_NO_ANIM == 0)
}

pub unsafe fn lb_8000B134(jobj: *mut HSD_JObj) -> bool {
    tree_any(jobj, |f| f & AOBJ_REWINDED != 0)
}

pub unsafe fn lb_8000B1CC(arg0: *mut HSD_JObj, pos0: *mut Vec, pos1: *mut Vec) {
    let mut r: Quaternion = core::mem::zeroed();
    let mut s: Vec = core::mem::zeroed();
    if arg0.is_null() {
        *pos1 = *pos0;
        return;
    }
    if !HSD_JObjGetParent(arg0).is_null() {
        HSD_JObjSetupMatrix(arg0);
        if pos0.is_null() || (!((*pos0).x != 0.0) && !((*pos0).y != 0.0) && !((*pos0).z != 0.0)) {
            (*pos1).x = (*arg0).mtx[0][3];
            (*pos1).y = (*arg0).mtx[1][3];
            (*pos1).z = (*arg0).mtx[2][3];
        } else {
            ext::PSMTXMultVec((*arg0).mtx.as_mut_ptr(), pos0, pos1);
        }
        return;
    }
    if pos0.is_null() || (!((*pos0).x != 0.0) && !((*pos0).y != 0.0) && !((*pos0).z != 0.0)) {
        HSD_JObjGetTranslation(arg0, pos1);
        return;
    }
    HSD_JObjGetRotation(arg0, &mut r);
    HSD_JObjGetScale(arg0, &mut s);
    if !(r.x != 0.0) && !(r.y != 0.0) && !(r.z != 0.0) && s.x == 1.0 && s.y == 1.0 && s.z == 1.0 {
        HSD_JObjGetTranslation(arg0, pos1);
        (*pos1).x += (*pos0).x;
        (*pos1).y += (*pos0).y;
        (*pos1).z += (*pos0).z;
        return;
    }
    HSD_JObjSetupMatrix(arg0);
    ext::PSMTXMultVec((*arg0).mtx.as_mut_ptr(), pos0, pos1);
}

pub unsafe fn lb_8000B4FC(jobj: *mut HSD_JObj, joint: *mut HSD_Joint) {
    if jobj.is_null() || joint.is_null() {
        return;
    }
    (*jobj).rotate.x = (*joint).rotation.x;
    (*jobj).rotate.y = (*joint).rotation.y;
    (*jobj).rotate.z = (*joint).rotation.z;
    (*jobj).scale = (*joint).scale;
    (*jobj).translate = (*joint).position;
    ext::HSD_JObjClearFlags(jobj, JOBJ_USE_QUATERNION);
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn lb_8000B5DC(jobj: *mut HSD_JObj, joint: *mut HSD_Joint) {
    if jobj.is_null() || joint.is_null() {
        return;
    }
    (*jobj).rotate.x = (*joint).rotation.x;
    (*jobj).rotate.y = (*joint).rotation.y;
    (*jobj).rotate.z = (*joint).rotation.z;
    (*jobj).translate = (*joint).position;
    ext::HSD_JObjClearFlags(jobj, JOBJ_USE_QUATERNION);
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn lb_8000B6A4(jobj: *mut HSD_JObj, joint: *mut HSD_Joint) {
    if jobj.is_null() || joint.is_null() {
        return;
    }
    (*jobj).scale = (*joint).scale;
    (*jobj).translate = (*joint).position;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn lb_8000B760(jobj: *mut HSD_JObj, joint: *mut HSD_Joint) {
    if jobj.is_null() || joint.is_null() {
        return;
    }
    (*jobj).translate = (*joint).position;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

pub unsafe fn lb_8000B804(jobj: *mut HSD_JObj, joint: *mut HSD_Joint) {
    if jobj.is_null() || joint.is_null() {
        return;
    }
    (*jobj).rotate.x = (*joint).rotation.x;
    (*jobj).rotate.y = (*joint).rotation.y;
    (*jobj).rotate.z = (*joint).rotation.z;
    (*jobj).scale = (*joint).scale;
    (*jobj).translate = (*joint).position;
    ext::HSD_JObjClearFlags(jobj, JOBJ_USE_QUATERNION);
    ext::HSD_JObjSetFlags(jobj, JOBJ_MTX_DIRTY);

    lb_8000B804((*jobj).next, (*joint).next);
    lb_8000B804((*jobj).child, (*joint).child);
}

/// `HSD_JObjWalkTree` callback (static): sets the AObj rate when the walk says so.
pub unsafe extern "C" fn lb_8000B9D8(jobj: *mut HSD_JObj, arg1: *mut *mut f32, arg2: i32) {
    let tmp = *arg1;
    if arg2 != 0 {
        ext::HSD_AObjSetRate((*jobj).aobj, *tmp);
    }
}

pub unsafe fn lb_8000BA0C(jobj: *mut HSD_JObj, mut arg8: f32) {
    let mut sp10: *mut f32 = &mut arg8;
    ext::HSD_JObjWalkTree(jobj, Some(lb_8000B9D8), &mut sp10);
}

pub unsafe fn lbDObjSetRateAll(dobj: *mut HSD_DObj, val: f32) {
    if dobj.is_null() {
        return;
    }
    if !(*dobj).pobj.is_null() {
        let mut cur = (*dobj).pobj;
        while !cur.is_null() {
            if (*cur).flags & 0x3000 == POBJ_SHAPEANIM && !(*(*cur).u.shape_set).aobj.is_null() {
                ext::HSD_AObjSetRate((*(*cur).u.shape_set).aobj, val);
            }
            cur = (*cur).next;
        }
    }
    if !(*dobj).mobj.is_null() {
        if !(*(*dobj).mobj).aobj.is_null() {
            ext::HSD_AObjSetRate((*(*dobj).mobj).aobj, val);
        }
        if !(*(*dobj).mobj).tobj.is_null() {
            let mut cur = (*(*dobj).mobj).tobj;
            while !cur.is_null() {
                if !(*cur).aobj.is_null() {
                    ext::HSD_AObjSetRate((*cur).aobj, val);
                }
                cur = (*cur).next;
            }
        }
    }
}

/// (exact copy of the above, but with HSD_AObjReqAnim)
pub unsafe fn lbDObjReqAnimAll(dobj: *mut HSD_DObj, val: f32) {
    if dobj.is_null() {
        return;
    }
    if !(*dobj).pobj.is_null() {
        let mut cur = (*dobj).pobj;
        while !cur.is_null() {
            if (*cur).flags & 0x3000 == POBJ_SHAPEANIM && !(*(*cur).u.shape_set).aobj.is_null() {
                ext::HSD_AObjReqAnim((*(*cur).u.shape_set).aobj, val);
            }
            cur = (*cur).next;
        }
    }
    if !(*dobj).mobj.is_null() {
        if !(*(*dobj).mobj).aobj.is_null() {
            ext::HSD_AObjReqAnim((*(*dobj).mobj).aobj, val);
        }
        if !(*(*dobj).mobj).tobj.is_null() {
            let mut cur = (*(*dobj).mobj).tobj;
            while !cur.is_null() {
                if !(*cur).aobj.is_null() {
                    ext::HSD_AObjReqAnim((*cur).aobj, val);
                }
                cur = (*cur).next;
            }
        }
    }
}

pub unsafe fn lbFindJObjWithAObj(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        return null_mut();
    }
    if !(*jobj).aobj.is_null() {
        return jobj;
    }
    let tmp = lbFindJObjWithAObj((*jobj).child);
    if !tmp.is_null() {
        return tmp;
    }
    let tmp = lbFindJObjWithAObj((*jobj).next);
    if !tmp.is_null() {
        return tmp;
    }
    null_mut()
}

pub unsafe fn lbGetJObjFramerate(jobj: *mut HSD_JObj) -> f32 {
    let jobj = lbFindJObjWithAObj(jobj);
    if !jobj.is_null() {
        return (*(*jobj).aobj).framerate;
    }
    0.0
}

pub unsafe fn lbGetJObjCurrFrame(jobj: *mut HSD_JObj) -> f32 {
    let jobj = lbFindJObjWithAObj(jobj);
    if !jobj.is_null() {
        return (*(*jobj).aobj).curr_frame;
    }
    0.0
}

pub unsafe fn lbGetJObjEndFrame(jobj: *mut HSD_JObj) -> f32 {
    let jobj = lbFindJObjWithAObj(jobj);
    if !jobj.is_null() {
        return (*(*jobj).aobj).end_frame;
    }
    0.0
}

pub unsafe fn lb_8000BECC(animjoint: *mut HSD_AnimJoint) -> *mut HSD_AnimJoint {
    if animjoint.is_null() {
        return null_mut();
    }
    if !(*animjoint).aobjdesc.is_null() {
        return animjoint;
    }
    let tmp = lb_8000BECC((*animjoint).child);
    if !tmp.is_null() {
        return tmp;
    }
    let tmp = lb_8000BECC((*animjoint).next);
    if !tmp.is_null() {
        return tmp;
    }
    null_mut()
}

pub unsafe fn lb_8000BFF0(animjoint: *mut HSD_AnimJoint) -> f32 {
    let animjoint = lb_8000BECC(animjoint);
    if !animjoint.is_null() {
        return (*(*animjoint).aobjdesc).end_frame;
    }
    0.0
}

pub unsafe fn lb_8000C07C(jobj: *mut HSD_JObj, i: i32, arg3: *mut *mut HSD_AnimJoint, arg4: *mut *mut HSD_MatAnimJoint, arg5: *mut *mut HSD_ShapeAnimJoint) {
    let phi_r4 = if !arg3.is_null() { *arg3.offset(i as isize) } else { null_mut() };
    let phi_r5 = if !arg4.is_null() { *arg4.offset(i as isize) } else { null_mut() };
    let phi_r6 = if !arg5.is_null() { *arg5.offset(i as isize) } else { null_mut() };
    ext::HSD_JObjAddAnimAll(jobj, phi_r4, phi_r5, phi_r6);
}

pub unsafe fn lb_8000C0E8(jobj: *mut HSD_JObj, i: i32, arg2: *mut DynamicModelDesc) {
    lb_8000C07C(jobj, i, (*arg2).anims, (*arg2).matanims, (*arg2).shapeanims);
}

pub unsafe fn memzero(mem: *mut c_void, mut size: i32) {
    let mut bytes = mem as *mut u8;
    while size != 0 {
        size -= 1;
        *bytes = 0;
        bytes = bytes.add(1);
    }
}

pub unsafe fn lb_8000C1C0(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj) {
    let robj = ext::HSD_RObjAlloc();
    ext::HSD_RObjSetFlags(robj, 0x9000_0001);
    ext::HSD_RObjSetConstraintObj(robj, constraint as *mut c_void);
    ext::HSD_JObjPrependRObj(jobj, robj);
}

pub unsafe fn lb_8000C228(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj) {
    let robj = ext::HSD_RObjAlloc();
    ext::HSD_RObjSetFlags(robj, 0x9000_0002);
    ext::HSD_RObjSetConstraintObj(robj, constraint as *mut c_void);
    ext::HSD_JObjPrependRObj(jobj, robj);
}

pub unsafe fn lb_8000C290(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj) {
    let robj = ext::HSD_RObjAlloc();
    ext::HSD_RObjSetFlags(robj, 0x9000_0004);
    ext::HSD_RObjSetConstraintObj(robj, constraint as *mut c_void);
    ext::HSD_JObjPrependRObj(jobj, robj);
}

pub unsafe fn lb_8000C2F8(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj) {
    lb_8000C1C0(jobj, constraint);
    lb_8000C290(jobj, constraint);
}

pub unsafe fn robj_next(robj: *mut HSD_RObj) -> *mut HSD_RObj {
    if !robj.is_null() {
        return (*robj).next;
    }
    null_mut()
}

pub unsafe fn lb_8000C390(jobj: *mut HSD_JObj) {
    let mut cur = HSD_JObjGetRObj(jobj);
    loop {
        cur = ext::HSD_RObjGetByType(cur, REFTYPE_JOBJ, 0);
        if cur.is_null() {
            break;
        }
        let next = robj_next(cur);
        ext::HSD_JObjDeleteRObj(jobj, cur);
        ext::HSD_RObjRemove(cur);
        cur = next;
    }
}

pub unsafe fn lb_8000C420(jobj: *mut HSD_JObj, flags: u32, limit: f32) {
    let robj = ext::HSD_RObjAlloc();
    ext::HSD_RObjSetFlags(robj, flags | 0xA000_0000);
    if !robj.is_null() {
        (*robj).u.limit = limit;
    }
    ext::HSD_JObjPrependRObj(jobj, robj);
}

pub unsafe fn lb_8000C490(jobj1: *mut HSD_JObj, jobj2: *mut HSD_JObj, arg2: *mut HSD_JObj, arg8: f32, arg9: f32) {
    let (j1, j2, a2) = (&mut *jobj1, &mut *jobj2, &mut *arg2);
    let mut sp60: Vec = core::mem::zeroed();
    let mut sp54: Vec = core::mem::zeroed();
    let mut quat1: Quaternion = core::mem::zeroed();
    let mut quat2: Quaternion = core::mem::zeroed();

    a2.translate.x = fmadds(j1.translate.x, arg8, j2.translate.x * arg9);
    a2.translate.y = fmadds(j1.translate.y, arg8, j2.translate.y * arg9);
    a2.translate.z = fmadds(j1.translate.z, arg8, j2.translate.z * arg9);
    a2.scale.x = fmadds(j1.scale.x, arg8, j2.scale.x * arg9);
    a2.scale.y = fmadds(j1.scale.y, arg8, j2.scale.y * arg9);
    a2.scale.z = fmadds(j1.scale.z, arg8, j2.scale.z * arg9);
    let is_quat_1 = ext::HSD_JObjGetFlags(jobj1) & 0x20000;
    let is_quat_2 = ext::HSD_JObjGetFlags(jobj2) & 0x20000;
    if is_quat_1 == 0 && is_quat_2 == 0 {
        let dx = j1.rotate.x - j2.rotate.x;
        let dy = j1.rotate.y - j2.rotate.y;
        let dz = j1.rotate.z - j2.rotate.z;
        let abs = |v: f32| if v < 0.0 { -v } else { v };
        if abs(dx) <= 0.0001 && abs(dy) <= 0.0001 && abs(dz) <= 0.0001 {
            a2.rotate = j1.rotate;
            ext::HSD_JObjClearFlags(arg2, 0x20000);
            ext::HSD_JObjSetFlags(arg2, 0x40);
            return;
        }
    }
    if is_quat_1 != 0 {
        quat1 = j1.rotate;
    } else {
        sp60.x = j1.rotate.x;
        sp60.y = j1.rotate.y;
        sp60.z = j1.rotate.z;
        ext::EulerToQuat(&mut sp60, &mut quat1);
    }
    if is_quat_2 != 0 {
        quat2 = j2.rotate;
    } else {
        sp54.x = j2.rotate.x;
        sp54.y = j2.rotate.y;
        sp54.z = j2.rotate.z;
        ext::EulerToQuat(&mut sp54, &mut quat2);
    }

    let sq = |v: f32| v * v;
    let sx = sq(quat1.x + quat2.x);
    let sy = sq(quat1.y + quat2.y);
    let sz = sq(quat1.z + quat2.z);
    let sw = sq(quat1.w + quat2.w);

    let dx = sq(quat1.x - quat2.x);
    let dy = sq(quat1.y - quat2.y);
    let dz = sq(quat1.z - quat2.z);
    let dw = sq(quat1.w - quat2.w);

    let sum_square_sums = sx + sy + sz + sw;
    let sum_square_diffs = dx + dy + dz + dw;

    if sum_square_diffs > sum_square_sums {
        quat2.x = -quat2.x;
        quat2.y = -quat2.y;
        quat2.z = -quat2.z;
        quat2.w = -quat2.w;
    }
    ext::HSD_QuatLib_8037EF28(&mut quat1, &mut quat2, &mut a2.rotate, arg9);
    ext::HSD_JObjSetFlags(arg2, 0x20000);
    ext::HSD_JObjSetFlags(arg2, 0x40);
}

pub unsafe fn lbCopyJObjSRT(src: *mut HSD_JObj, dst: *mut HSD_JObj) {
    (*dst).rotate = (*src).rotate;
    (*dst).scale = (*src).scale;
    (*dst).translate = (*src).translate;
    if ext::HSD_JObjGetFlags(src) & JOBJ_USE_QUATERNION != 0 {
        ext::HSD_JObjSetFlags(dst, JOBJ_USE_QUATERNION);
    } else {
        ext::HSD_JObjClearFlags(dst, JOBJ_USE_QUATERNION);
    }
    ext::HSD_JObjSetFlags(dst, JOBJ_MTX_DIRTY);
}

pub unsafe fn lb_8000C868(arg0: *mut HSD_Joint, arg1: *mut HSD_JObj, arg2: *mut HSD_JObj, arg8: f32, arg9: f32) {
    let (j0, j1, j2) = (&mut *arg0, &mut *arg1, &mut *arg2);
    let mut sp_c0: Vec = core::mem::zeroed();
    let mut sp_b4: Vec = core::mem::zeroed();
    let mut sp_a4: Quaternion = core::mem::zeroed();
    let mut sp94: Quaternion = core::mem::zeroed();

    j2.translate.x = fmadds(j0.position.x, arg8, j1.translate.x * arg9);
    j2.translate.y = fmadds(j0.position.y, arg8, j1.translate.y * arg9);
    j2.translate.z = fmadds(j0.position.z, arg8, j1.translate.z * arg9);
    j2.scale.x = fmadds(j0.scale.x, arg8, j1.scale.x * arg9);
    j2.scale.y = fmadds(j0.scale.y, arg8, j1.scale.y * arg9);
    j2.scale.z = fmadds(j0.scale.z, arg8, j1.scale.z * arg9);
    let temp_r31 = ext::HSD_JObjGetFlags(arg1) & 0x20000;
    if temp_r31 == 0 {
        let temp_f5 = j0.rotation.x - j1.rotate.x;
        let temp_f3 = j0.rotation.y - j1.rotate.y;
        let temp_f2 = j0.rotation.z - j1.rotate.z;
        let phi_f1 = if temp_f5 < 0.0 { -temp_f5 } else { temp_f5 };
        if phi_f1 <= 1e-4 {
            let phi_f1_2 = if temp_f3 < 0.0 { -temp_f3 } else { temp_f3 };
            if phi_f1_2 <= 1e-4 {
                let phi_f1_3 = if temp_f2 < 0.0 { -temp_f2 } else { temp_f2 };
                if phi_f1_3 <= 1e-4 {
                    j2.rotate.x = j0.rotation.x;
                    j2.rotate.y = j0.rotation.y;
                    j2.rotate.z = j0.rotation.z;
                    ext::HSD_JObjClearFlags(arg2, 0x20000);
                    ext::HSD_JObjSetFlags(arg2, 0x40);
                    return;
                }
            }
        }
    }
    sp_c0.x = j0.rotation.x;
    sp_c0.y = j0.rotation.y;
    sp_c0.z = j0.rotation.z;
    ext::EulerToQuat(&mut sp_c0, &mut sp_a4);
    if temp_r31 != 0 {
        sp94 = j1.rotate;
    } else {
        sp_b4.x = j1.rotate.x;
        sp_b4.y = j1.rotate.y;
        sp_b4.z = j1.rotate.z;
        ext::EulerToQuat(&mut sp_b4, &mut sp94);
    }

    let sq = |v: f32| v * v;
    let sum = Quaternion { x: sq(sp_a4.x + sp94.x), y: sq(sp_a4.y + sp94.y), z: sq(sp_a4.z + sp94.z), w: sq(sp_a4.w + sp94.w) };
    let dif = Quaternion { x: sq(sp_a4.x - sp94.x), y: sq(sp_a4.y - sp94.y), z: sq(sp_a4.z - sp94.z), w: sq(sp_a4.w - sp94.w) };

    if dif.x + dif.y + dif.z + dif.w > sum.x + sum.y + sum.z + sum.w {
        sp94.x = -sp94.x;
        sp94.y = -sp94.y;
        sp94.z = -sp94.z;
        sp94.w = -sp94.w;
    }
    ext::HSD_QuatLib_8037EF28(&mut sp_a4, &mut sp94, &mut j2.rotate, arg9);
    ext::HSD_JObjSetFlags(arg2, 0x20000);
    ext::HSD_JObjSetFlags(arg2, 0x40);
}

pub unsafe fn lbGetFreeColorRegImpl(i0: i32, mut tevdesc: *mut HSD_TevDesc, mut texp1: *mut HSD_TExp, mut texp2: *mut HSD_TExp) -> i32 {
    let mut register_used = [false; 8];
    while !texp1.is_null() {
        if (*texp1).cnst.reg < 8 {
            register_used[(*texp1).cnst.reg as usize] = true;
        }
        texp1 = (*texp1).cnst.next;
    }
    while !tevdesc.is_null() {
        let idx = lb_803B9FF0[(*tevdesc).u.tevconf.clr_out_reg as usize];
        register_used[idx as usize] = true;
        tevdesc = (*tevdesc).next;
    }
    while !texp2.is_null() {
        if (*texp2).cnst.reg < 8 {
            register_used[(*texp2).cnst.reg as usize] = true;
        }
        texp2 = (*texp2).cnst.next;
    }
    let mut i = i0;
    while i < 7 {
        if !register_used[i as usize] {
            return i;
        }
        i += 1;
    }
    -1
}

pub unsafe fn lbGetFreeColorRegister(i0: i32, mobj: *mut HSD_MObj, texp: *mut HSD_TExp) -> i32 {
    lbGetFreeColorRegImpl(i0, &mut (*(*mobj).tevdesc).desc, (*mobj).texp, texp)
}

pub fn lb_8000CC8C(i: i32) -> i32 {
    lb_803B9FC0[i as usize]
}

pub fn lb_8000CCA4(i: i32) -> i32 {
    lb_803B9FE0[i as usize]
}

pub unsafe fn lbGetFreeAlphaRegImpl(i0: i32, mut cur: *mut HSD_TevDesc, arg2: *mut HSD_TExp, arg3: *mut HSD_TExp) -> i32 {
    let mut register_used = [false; 8];
    while !cur.is_null() {
        let idx = lb_803BA030[(*cur).u.tevconf.alpha_out_reg as usize];
        register_used[idx as usize] = true;
        cur = (*cur).next;
    }
    let mut i = i0;
    while i < 7 {
        if !register_used[i as usize] {
            return i;
        }
        i += 1;
    }
    -1
}

pub unsafe fn lbGetFreeAlphaRegister(i0: i32, mobj: *mut HSD_MObj, texp: *mut HSD_TExp) -> i32 {
    lbGetFreeAlphaRegImpl(i0, &mut (*(*mobj).tevdesc).desc, (*mobj).texp, texp)
}

pub fn lb_8000CD90(i: i32) -> i32 {
    lb_803BA000[i as usize]
}

pub fn lb_8000CDA8(i: i32) -> i32 {
    lb_803BA020[i as usize]
}

/// The C has no `return` after the loop (the PPC leaves r3 = NULL); the port returns NULL.
pub unsafe fn lb_8000CDC0(mut cur: *mut HSD_LObj) -> *mut HSD_LObj {
    while !cur.is_null() {
        if (*cur).flags & ((1 << 0) | (1 << 1)) == 0 && ext::HSD_LObjGetFlags(cur) & (1 << 5) == 0 {
            return cur;
        }
        cur = HSD_LObjGetNext(cur);
    }
    null_mut()
}

pub unsafe fn lb_8000CE30(dobj: *mut HSD_DObj, next: *mut HSD_DObj) {
    if !dobj.is_null() {
        (*dobj).next = next;
    }
}

pub unsafe fn lb_8000CE40(jobj: *mut HSD_JObj, dobj: *mut HSD_DObj) {
    if !jobj.is_null() {
        (*jobj).u.dobj = dobj;
    }
}
