//! `src/sysdolphin/baselib/robj.c` over the game's own `HSD_RObj` / `HSD_Rvalue` records (raw pointers, native
//! layout). The owned-model port of the same file (checked bit for bit against the C) is `crate::robj`; the math
//! helpers it shares (matrix columns, `HSD_MtxInverseConcat`, vector ops) are called from there.
#![allow(non_snake_case)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gekko::sqrtf as sqrtf_robj;
#[allow(unused_imports)]
use crate::jobj::JObjId;
#[allow(unused_imports)]
use crate::jobj::JObjTree;
#[allow(unused_imports)]
use crate::mtx as mtx_robj;
#[allow(unused_imports)]
use crate::mtx::Vec3 as Vec3_robj;
#[allow(unused_imports)]
use crate::robj::{EPS, RObjUpdate, RObjVal};
use melee_types::HSD_ObjAllocData;

/// `robj_alloc_data` / `rvalue_alloc_data` (robj.c): the native pools of the engine path.
pub static mut robj_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut rvalue_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

/// `HSD_RObjInitAllocData()`.
pub unsafe fn HSD_RObjInitAllocData() {
    melee_core::ct!("HSD_RObjInitAllocData");
    crate::tu::objalloc::HSD_ObjAllocInit(&raw mut robj_alloc_data, core::mem::size_of::<melee_types::HSD_RObj>(), 4);
    crate::tu::objalloc::HSD_ObjAllocInit(&raw mut rvalue_alloc_data, core::mem::size_of::<melee_types::HSD_Rvalue>(), 4);
}
/// `HSD_RObjGetAllocData()`.
pub unsafe fn HSD_RObjGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_RObjGetAllocData");
    &raw mut robj_alloc_data
}
/// `HSD_RvalueObjGetAllocData()`.
pub unsafe fn HSD_RvalueObjGetAllocData() -> *mut HSD_ObjAllocData {
    melee_core::ct!("HSD_RvalueObjGetAllocData");
    &raw mut rvalue_alloc_data
}
/// `static float* arg_buf; static u32 arg_buf_size;` (robj.c): the expression argument buffer.
pub static mut arg_buf: *mut f32 = core::ptr::null_mut();
pub static mut arg_buf_size: u32 = 0;

/// `_HSD_RObjForgetMemory(low, high)`.
pub unsafe fn _HSD_RObjForgetMemory(low: *mut core::ffi::c_void, high: *mut core::ffi::c_void) {
    melee_core::ct!("_HSD_RObjForgetMemory");
    let p = arg_buf as *mut core::ffi::c_void;
    if low <= p && p < high {
        arg_buf = core::ptr::null_mut();
        arg_buf_size = 0;
    }
}




use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

const ROBJ_TYPE_MASK_: u32 = 0x7000_0000;
const REFTYPE_EXP_: u32 = 0x0000_0000;
const REFTYPE_JOBJ_: u32 = 0x1000_0000;
const REFTYPE_LIMIT_: u32 = 0x2000_0000;
const REFTYPE_BYTECODE_: u32 = 0x3000_0000;
const REFTYPE_IKHINT_: u32 = 0x4000_0000;
const EPS_: f32 = 1.00000001335e-10f32;
/// `HSD_RObj_80406E74`.
static HSD_RObj_80406E74: [i32; 3] = [0x32, 0x33, 0x34];

type UpdateFunc = unsafe extern "C" fn(*mut c_void, enum_t, *mut HSD_ObjData);

/// `HSD_JObjRefThis` (jobj.h): `iref_INC`.
unsafe fn jobj_ref_this(j: *mut HSD_JObj) {
    if !j.is_null() {
        (*j).object.ref_count_individual = (*j).object.ref_count_individual.wrapping_add(1);
        assert!((*j).object.ref_count_individual != 0, "object.h:158 HSD_OBJ(o)->ref_count_individual != 0");
    }
}

#[inline]
fn v3(v: &Vec) -> crate::mtx::Vec3 {
    crate::mtx::Vec3 { x: v.x, y: v.y, z: v.z }
}
#[inline]
fn nv(v: &crate::mtx::Vec3) -> Vec {
    Vec { x: v.x, y: v.y, z: v.z }
}

/// `HSD_RObjSetFlags`.
pub unsafe fn HSD_RObjSetFlags(robj: *mut HSD_RObj, flags: u32) {
    melee_core::ct!("HSD_RObjSetFlags");
    if !robj.is_null() {
        (*robj).flags |= flags;
    }
}

/// `HSD_RObjGetByType`.
pub unsafe fn HSD_RObjGetByType(robj: *mut HSD_RObj, type_: u32, subtype: u32) -> *mut HSD_RObj {
    melee_core::ct!("HSD_RObjGetByType");
    inlined_HSD_RObjGetByType(robj, type_, subtype)
}

/// `inlined_HSD_RObjGetByType` (static inline).
pub unsafe fn inlined_HSD_RObjGetByType(robj: *mut HSD_RObj, type_: u32, subtype: u32) -> *mut HSD_RObj {
    melee_core::ct!("inlined_HSD_RObjGetByType");
    if robj.is_null() {
        return null_mut();
    }
    let mut curr = robj;
    while !curr.is_null() {
        if (*curr).flags & 0x8000_0000 != 0
            && (*curr).flags & ROBJ_TYPE_MASK_ == type_
            && (subtype == 0 || subtype == ((*curr).flags & 0xFFF_FFFF))
        {
            return curr;
        }
        curr = (*curr).next;
    }
    null_mut()
}

/// `RObjUpdateFunc` (static).
pub unsafe extern "C" fn RObjUpdateFunc(obj: *mut c_void, type_: enum_t, val: *mut HSD_ObjData) {
    melee_core::ct!("RObjUpdateFunc");
    if obj.is_null() {
        return;
    }
    if type_ != 1 {
        return;
    }
    let robj = obj as *mut HSD_RObj;
    if (*val).fv >= 0.5 {
        (*robj).flags |= 0x8000_0000;
        return;
    }
    (*robj).flags &= 0x7fff_ffff;
}

/// `HSD_RObjAnim`.
pub unsafe fn HSD_RObjAnim(robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjAnim");
    if robj.is_null() {
        return;
    }
    super::aobj::HSD_AObjInterpretAnim((*robj).aobj, robj as *mut c_void, Some(RObjUpdateFunc));
}

/// `HSD_RObjAnimAll`.
pub unsafe fn HSD_RObjAnimAll(robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjAnimAll");
    let mut curr = robj;
    while !curr.is_null() {
        HSD_RObjAnim(curr);
        curr = (*curr).next;
    }
}

/// `HSD_RObjRemoveAnimByFlags`.
pub unsafe fn HSD_RObjRemoveAnimByFlags(robj: *mut HSD_RObj, flags: u32) {
    melee_core::ct!("HSD_RObjRemoveAnimByFlags");
    if robj.is_null() {
        return;
    }
    if !(*robj).aobj.is_null() && flags & 0x80 != 0 {
        super::aobj::HSD_AObjRemove((*robj).aobj);
        (*robj).aobj = null_mut();
    }
}

/// `HSD_RObjRemoveAnimAllByFlags`.
pub unsafe fn HSD_RObjRemoveAnimAllByFlags(mut robj: *mut HSD_RObj, flags: u32) {
    melee_core::ct!("HSD_RObjRemoveAnimAllByFlags");
    while !robj.is_null() {
        HSD_RObjRemoveAnimByFlags(robj, flags);
        robj = (*robj).next;
    }
}

/// `HSD_RObjRemoveAnimAll`.
pub unsafe fn HSD_RObjRemoveAnimAll(robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjRemoveAnimAll");
    HSD_RObjRemoveAnimAllByFlags(robj, 0x7FF);
}

/// `HSD_RObjReqAnimByFlags`.
pub unsafe fn HSD_RObjReqAnimByFlags(robj: *mut HSD_RObj, startframe: f32, flags: u32) {
    melee_core::ct!("HSD_RObjReqAnimByFlags");
    if robj.is_null() {
        return;
    }
    if !(*robj).aobj.is_null() && flags & 0x80 != 0 {
        super::aobj::HSD_AObjReqAnim((*robj).aobj, startframe);
    }
}

/// `HSD_RObjReqAnimAllByFlags`.
pub unsafe fn HSD_RObjReqAnimAllByFlags(mut robj: *mut HSD_RObj, startframe: f32, flags: u32) {
    melee_core::ct!("HSD_RObjReqAnimAllByFlags");
    while !robj.is_null() {
        HSD_RObjReqAnimByFlags(robj, startframe, flags);
        robj = (*robj).next;
    }
}

/// `HSD_RObjReqAnimAll`.
pub unsafe fn HSD_RObjReqAnimAll(robj: *mut HSD_RObj, startframe: f32) {
    melee_core::ct!("HSD_RObjReqAnimAll");
    HSD_RObjReqAnimAllByFlags(robj, startframe, 0x7FF);
}

/// `HSD_RObjAddAnim`.
pub unsafe fn HSD_RObjAddAnim(robj: *mut HSD_RObj, anim: *mut HSD_RObjAnimJoint) {
    melee_core::ct!("HSD_RObjAddAnim");
    if robj.is_null() || anim.is_null() {
        return;
    }
    if !(*robj).aobj.is_null() {
        super::aobj::HSD_AObjRemove((*robj).aobj);
    }
    (*robj).aobj = super::aobj::HSD_AObjLoadDesc((*anim).aobjdesc);
}

/// `HSD_RObjAddAnimAll`.
pub unsafe fn HSD_RObjAddAnimAll(robj: *mut HSD_RObj, anim: *mut HSD_RObjAnimJoint) {
    melee_core::ct!("HSD_RObjAddAnimAll");
    if robj.is_null() || anim.is_null() {
        return;
    }
    let (mut i, mut j) = (robj, anim);
    while !i.is_null() && !j.is_null() {
        HSD_RObjAddAnim(i, j);
        i = (*i).next;
        j = (*j).next;
    }
}

/// `HSD_RObjGetConstraintType` (static).
unsafe fn HSD_RObjGetConstraintType(robj: *mut HSD_RObj) -> u32 {
    melee_core::ct!("HSD_RObjGetConstraintType");
    if robj.is_null() {
        return 0;
    }
    (*robj).flags & 0x0FFF_FFFF
}

/// `HSD_RObjGetGlobalPosition`.
pub unsafe fn HSD_RObjGetGlobalPosition(robj: *mut HSD_RObj, type_: i32, p: *mut Vec) -> i32 {
    melee_core::ct!("HSD_RObjGetGlobalPosition");
    let mut v = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut n = 0i32;
    if robj.is_null() {
        return 0;
    }
    let mut rp = robj;
    while !rp.is_null() {
        if (*rp).flags & ROBJ_TYPE_MASK_ == REFTYPE_JOBJ_
            && (*rp).flags & 0x8000_0000 != 0
            && type_ as u32 == HSD_RObjGetConstraintType(rp)
        {
            let j = (*rp).u.jobj;
            assert!(!j.is_null(), "robj.c:498 rp->u.jobj");
            super::jobj::HSD_JObjSetupMatrix(j);
            n = n.wrapping_add(1);
            v.x += (*j).mtx[0][3];
            v.y += (*j).mtx[1][3];
            v.z += (*j).mtx[2][3];
        }
        rp = (*rp).next;
    }
    if n != 0 {
        let f = 1.0f32 / n as f32;
        (*p).x = f * v.x;
        (*p).y = f * v.y;
        (*p).z = f * v.z;
    }
    n
}

/// `set_dirup_matrix` (static).
unsafe fn set_dirup_matrix(dir: *mut Vec, uv: *mut Vec, scale: *const Vec, obj: *mut c_void, update: UpdateFunc) {
    melee_core::ct!("set_dirup_matrix");
    use crate::gekko::sqrtf;
    use crate::mtx;
    let (d, u) = (v3(&*dir), v3(&*uv));
    let mut z = mtx::vec_cross(&d, &u);
    let kdir = sqrtf(1.0f32 / (EPS_ + mtx::vec_dot(&d, &d)));
    let d = mtx::vec_scale(&d, kdir);
    let kz = sqrtf(1.0f32 / (EPS_ + mtx::vec_dot(&z, &z)));
    z = mtx::vec_scale(&z, kz);
    let u = mtx::vec_cross(&z, &d);
    *dir = nv(&d);
    *uv = nv(&u);
    let mut v = HSD_ObjData { p: Vec { x: d.x * (*scale).x, y: d.y * (*scale).x, z: d.z * (*scale).x } };
    update(obj, 50, &mut v);
    v = HSD_ObjData { p: Vec { x: u.x * (*scale).y, y: u.y * (*scale).y, z: u.z * (*scale).y } };
    update(obj, 51, &mut v);
    v = HSD_ObjData { p: Vec { x: z.x * (*scale).z, y: z.y * (*scale).z, z: z.z * (*scale).z } };
    update(obj, 52, &mut v);
    update(obj, 55, null_mut());
}

/// `resolveCnsDirUp` (static).
unsafe fn resolveCnsDirUp(robj: *mut HSD_RObj, obj: *mut c_void, update: UpdateFunc) {
    melee_core::ct!("resolveCnsDirUp");
    use crate::mtx;
    let mut this_scale = Vec { x: 1.0, y: 1.0, z: 1.0 };
    let mut up = Vec { x: 0.0, y: 1.0, z: 0.0 };
    let mut this_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    if HSD_RObjGetGlobalPosition(robj, 2, &mut this_pos) != 0 {
        let jo = obj as *mut HSD_JObj;
        let dir = Vec { x: (*jo).mtx[0][3], y: (*jo).mtx[1][3], z: (*jo).mtx[2][3] };
        this_pos = nv(&mtx::vec_sub(&v3(&this_pos), &v3(&dir)));
        if HSD_RObjGetGlobalPosition(robj, 3, &mut up) != 0 {
            up = nv(&mtx::vec_sub(&v3(&up), &v3(&dir)));
        } else {
            let k = 1.0f32 - mtx::vec_dot(&v3(&this_pos), &v3(&up));
            if crate::hsd_mtx::fabsf_bitwise(k) < EPS_ {
                up = Vec { x: 0.0, y: 0.0, z: 1.0 };
            }
        }
        if !(*jo).scl.is_null() {
            this_scale = *(*jo).scl;
        }
        set_dirup_matrix(&mut this_pos, &mut up, &this_scale, obj, update);
    }
}

#[inline]
unsafe fn jobj_parent(j: *mut HSD_JObj) -> *mut HSD_JObj {
    if j.is_null() {
        null_mut()
    } else {
        (*j).parent
    }
}

/// `resolveCnsOrientation` (static).
unsafe fn resolveCnsOrientation(robj: *mut HSD_RObj, obj: *mut c_void, update: UpdateFunc) {
    melee_core::ct!("resolveCnsOrientation");
    use crate::mtx::{self, Mtx};
    use crate::robj::{mtx_col_mag, mtx_col_vec};
    assert!(!obj.is_null(), "robj.c:0x276 obj");
    let robj = inlined_HSD_RObjGetByType(robj, 0x1000_0000, 4);
    if robj.is_null() {
        return;
    }
    let target = (*robj).u.jobj;
    let objj = obj as *mut HSD_JObj;
    if super::jobj_world::HSD_JObjGetFlags(target) & 8 == 0 || jobj_parent(target).is_null() {
        let mtx0: Mtx = *(super::jobj_ptr::HSD_JObjGetMtxPtr(target) as *mut Mtx);
        for i in 0..3usize {
            let mut v = mtx_col_vec(&mtx0, i);
            let mut sval = mtx::vec_mag(&v);
            if sval > 1e-10f32 {
                sval = 1.0f32 / sval;
            }
            sval *= mtx_col_mag(&(*objj).mtx, i);
            v.x *= sval;
            v.y *= sval;
            v.z *= sval;
            let mut d = HSD_ObjData { p: nv(&v) };
            update(obj, HSD_RObj_80406E74[i], &mut d);
        }
    } else {
        let p = jobj_parent(target);
        let (pm, tm) = (
            super::jobj_ptr::HSD_JObjGetMtxPtr(p) as *mut Mtx,
            super::jobj_ptr::HSD_JObjGetMtxPtr(target) as *mut Mtx,
        );
        let mut mtx1: Mtx = crate::hsd_mtx::inverse_concat(&*pm, &*tm);
        for i in 0..3usize {
            let mut v = mtx_col_vec(&mtx1, i);
            let mut sval = mtx::vec_mag(&v);
            if sval > 1e-10f32 {
                sval = 1.0f32 / sval;
            }
            sval *= mtx_col_mag(&(*objj).mtx, i);
            v.x *= sval;
            v.y *= sval;
            v.z *= sval;
            mtx1[0][i] = v.x;
            mtx1[1][i] = v.y;
            mtx1[2][i] = v.z;
        }
        let mut j = jobj_parent(target);
        while !j.is_null() {
            let mut mtx0: Mtx = if !jobj_parent(j).is_null() {
                let (a, b) = (
                    super::jobj_ptr::HSD_JObjGetMtxPtr(jobj_parent(j)) as *mut Mtx,
                    super::jobj_ptr::HSD_JObjGetMtxPtr(j) as *mut Mtx,
                );
                crate::hsd_mtx::inverse_concat(&*a, &*b)
            } else {
                *(super::jobj_ptr::HSD_JObjGetMtxPtr(j) as *mut Mtx)
            };
            for i in 0..3usize {
                let mut v = mtx_col_vec(&mtx0, i);
                let mut sval = mtx::vec_mag(&v);
                if sval > 1e-10f32 {
                    sval = 1.0f32 / sval;
                }
                v.x *= sval;
                v.y *= sval;
                v.z *= sval;
                mtx0[0][i] = v.x;
                mtx0[1][i] = v.y;
                mtx0[2][i] = v.z;
            }
            mtx1 = mtx::concat(&mtx0, &mtx1);
            j = jobj_parent(j);
        }
        for i in 0..3usize {
            let v = mtx_col_vec(&mtx1, i);
            let mut d = HSD_ObjData { p: nv(&v) };
            update(obj, HSD_RObj_80406E74[i], &mut d);
        }
    }
    update(obj, 0x37, null_mut());
}

/// `resolveLimits` (static).
unsafe fn resolveLimits(robj: *mut HSD_RObj, obj: *mut c_void, _update: UpdateFunc) {
    melee_core::ct!("resolveLimits");
    let jobj = obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "robj.c:0x2E1 jobj");
    let is_limit = |rp: *mut HSD_RObj| (*rp).flags & ROBJ_TYPE_MASK_ == REFTYPE_LIMIT_;
    let mut rp = robj;
    while !rp.is_null() {
        if is_limit(rp) {
            break;
        }
        rp = (*rp).next;
    }
    if rp.is_null() {
        return;
    }
    let mut update_mtx = false;
    rp = robj;
    while !rp.is_null() {
        if is_limit(rp) {
            let l = (*rp).u.limit;
            // (field, is_lower_bound); cases 11/12 clamp translate.y again, as in the C.
            let (f, lower): (*mut f32, bool) = match (*rp).flags & 0xFFF_FFFF {
                1 => (&mut (*jobj).rotate.x, true),
                2 => (&mut (*jobj).rotate.x, false),
                3 => (&mut (*jobj).rotate.y, true),
                4 => (&mut (*jobj).rotate.y, false),
                5 => (&mut (*jobj).rotate.z, true),
                6 => (&mut (*jobj).rotate.z, false),
                7 => (&mut (*jobj).translate.x, true),
                8 => (&mut (*jobj).translate.x, false),
                9 | 11 => (&mut (*jobj).translate.y, true),
                10 | 12 => (&mut (*jobj).translate.y, false),
                _ => {
                    rp = (*rp).next;
                    continue;
                }
            };
            if lower {
                if *f < l {
                    *f = l;
                }
            } else if *f > l {
                *f = l;
            }
            update_mtx = true;
        }
        rp = (*rp).next;
    }
    if update_mtx {
        super::jobj_ptr::HSD_JObjMakeMatrix(jobj);
    }
}

/// `HSD_RObjUpdateAll`.
pub unsafe fn HSD_RObjUpdateAll(robj: *mut HSD_RObj, obj: *mut c_void, update_func: HSD_ObjUpdateFunc) {
    melee_core::ct!("HSD_RObjUpdateAll");
    if robj.is_null() {
        return;
    }
    let update = update_func.expect("HSD_RObjUpdateAll: update_func");
    let mut vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
    if HSD_RObjGetGlobalPosition(robj, 1, &mut vec) != 0 {
        let mut d = HSD_ObjData { p: vec };
        update(obj, 0x35, &mut d);
        update(obj, 0x38, null_mut());
    }
    resolveCnsDirUp(robj, obj, update);
    resolveCnsOrientation(robj, obj, update);
    resolveLimits(robj, obj, update);
    let mut rp = robj;
    while !rp.is_null() {
        if (*rp).flags & ROBJ_TYPE_MASK_ == 0 && (*rp).flags & 0x8000_0000 != 0 {
            expEvaluate(&mut (*rp).u.exp, (*rp).flags & 0xFFF_FFFF, obj, update);
        }
        rp = (*rp).next;
    }
}

/// `HSD_RObjResolveRefs`.
pub unsafe fn HSD_RObjResolveRefs(robj: *mut HSD_RObj, desc: *mut HSD_RObjDesc) {
    melee_core::ct!("HSD_RObjResolveRefs");
    if robj.is_null() || desc.is_null() {
        return;
    }
    match (*robj).flags & ROBJ_TYPE_MASK_ {
        REFTYPE_JOBJ_ => {
            super::jobj::HSD_JObjUnrefThis((*robj).u.jobj);
            // `(HSD_IDKey) desc->u.joint`: the key is the decoded pointer itself
            (*robj).u.jobj =
                super::id::HSD_IDGetDataFromTable(null_mut(), (*desc).u.joint as usize, null_mut()) as *mut HSD_JObj;
            assert!(!(*robj).u.jobj.is_null(), "robj.c:883 robj->u.jobj");
            jobj_ref_this((*robj).u.jobj);
        }
        0 => HSD_RvalueResolveRefsAll((*robj).u.exp.rvalue, (*(*desc).u.exp).rvalue),
        _ => {}
    }
}

/// `HSD_RObjResolveRefsAll`.
pub unsafe fn HSD_RObjResolveRefsAll(mut robj: *mut HSD_RObj, mut desc: *mut HSD_RObjDesc) {
    melee_core::ct!("HSD_RObjResolveRefsAll");
    while !robj.is_null() && !desc.is_null() {
        HSD_RObjResolveRefs(robj, desc);
        robj = (*robj).next;
        desc = (*desc).next;
    }
}

/// `HSD_RObjLoadDesc`.
pub unsafe fn HSD_RObjLoadDesc(robjdesc: *mut HSD_RObjDesc) -> *mut HSD_RObj {
    melee_core::ct!("HSD_RObjLoadDesc");
    if robjdesc.is_null() {
        return null_mut();
    }
    let robj = HSD_RObjAlloc();
    (*robj).next = HSD_RObjLoadDesc((*robjdesc).next);
    (*robj).flags = (*robjdesc).flags;
    match (*robj).flags & ROBJ_TYPE_MASK_ {
        REFTYPE_JOBJ_ => {}
        REFTYPE_LIMIT_ => match (*robj).flags & 0xFFF_FFFF {
            1..=6 => (*robj).u.limit = 0.017453292f32 * (*robjdesc).u.limit,
            _ => (*robj).u.limit = (*robjdesc).u.limit,
        },
        REFTYPE_EXP_ => expLoadDesc(&mut (*robj).u.exp, (*robjdesc).u.exp),
        REFTYPE_BYTECODE_ => {
            bcexpLoadDesc(&mut (*robj).u.exp, (*robjdesc).u.bcexp);
            (*robj).flags &= !ROBJ_TYPE_MASK_;
        }
        REFTYPE_IKHINT_ => {
            (*robj).u.ik_hint.bone_length = (*(*robjdesc).u.ik_hint).bone_length;
            (*robj).u.ik_hint.rotate_x = (*(*robjdesc).u.ik_hint).rotate_x;
        }
        _ => {
            super::debug::HSD_Panic(c"robj.c".as_ptr(), 0x3C0, c"unexpected type of robj.\n".as_ptr());
        }
    }
    robj
}

/// `HSD_RObjRemove`.
pub unsafe fn HSD_RObjRemove(robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjRemove");
    if robj.is_null() {
        return;
    }
    match (*robj).flags & ROBJ_TYPE_MASK_ {
        REFTYPE_JOBJ_ => super::jobj::HSD_JObjUnrefThis((*robj).u.jobj),
        0 => HSD_RvalueRemoveAll((*robj).u.exp.rvalue),
        _ => {}
    }
    super::aobj::HSD_AObjRemove((*robj).aobj);
    HSD_RObjFree(robj);
}

/// `HSD_RObjRemoveAll`.
pub unsafe fn HSD_RObjRemoveAll(mut robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjRemoveAll");
    while !robj.is_null() {
        let next = (*robj).next;
        HSD_RObjRemove(robj);
        robj = next;
    }
}

/// `HSD_RObjAlloc`.
pub unsafe fn HSD_RObjAlloc() -> *mut HSD_RObj {
    melee_core::ct!("HSD_RObjAlloc");
    let new = super::objalloc::HSD_ObjAlloc(HSD_RObjGetAllocData()) as *mut HSD_RObj;
    assert!(!new.is_null(), "robj.c:1032 new");
    core::ptr::write_bytes(new, 0, 1);
    new
}

/// `HSD_RObjFree`.
pub unsafe fn HSD_RObjFree(robj: *mut HSD_RObj) {
    melee_core::ct!("HSD_RObjFree");
    super::objalloc::HSD_ObjFree(HSD_RObjGetAllocData(), robj as *mut c_void);
}

/// `expEvaluate` (static).
unsafe fn expEvaluate(exp: *mut HSD_Exp, type_: u32, obj: *mut c_void, update: UpdateFunc) {
    melee_core::ct!("expEvaluate");
    if (*exp).nb_args == u32::MAX {
        let mut nb_args = 0u32;
        let mut rv = (*exp).rvalue;
        while !rv.is_null() {
            nb_args = nb_args.wrapping_add(crate::hsd_util::get_nb_bits((*rv).flags));
            rv = (*rv).next;
        }
        (*exp).nb_args = nb_args;
    }
    if arg_buf.is_null() {
        if arg_buf_size == 0 {
            arg_buf_size = 100;
        }
        arg_buf = super::memory::HSD_MemAlloc((arg_buf_size as usize * core::mem::size_of::<f32>()) as i32) as *mut f32;
    }
    if arg_buf_size < (*exp).nb_args {
        panic!(
            "Number of argment of expression exceeds the argument buffer\nsize. (requested num of arg {}, allocated {})",
            (*exp).nb_args,
            arg_buf_size
        );
    }
    let mut cur = arg_buf;
    let k = 57.29578f32;
    let mut rvalue = (*exp).rvalue;
    while !rvalue.is_null() {
        let jobj = (*rvalue).jobj;
        assert!(!jobj.is_null(), "robj.c:0x467 jobj");
        super::jobj::HSD_JObjSetupMatrix((*rvalue).jobj);
        let mut bit: u32 = 1;
        while bit != 0 && bit <= (*rvalue).flags {
            let val: Option<f32> = match (*rvalue).flags & bit {
                0x1 => Some(k * (*jobj).rotate.x),
                0x2 => Some(k * (*jobj).rotate.y),
                0x4 => Some(k * (*jobj).rotate.z),
                0x10 => Some((*jobj).translate.x),
                0x20 => Some((*jobj).translate.y),
                0x40 => Some((*jobj).translate.z),
                0x80 => Some((*jobj).scale.x),
                0x100 => Some((*jobj).scale.y),
                0x200 => Some((*jobj).scale.z),
                0x10000 => Some(k * crate::hsd_mtx::get_rotation(&(*jobj).mtx).x),
                0x20000 => Some(k * crate::hsd_mtx::get_rotation(&(*jobj).mtx).y),
                0x40000 => Some(k * crate::hsd_mtx::get_rotation(&(*jobj).mtx).z),
                0x100000 => Some((*jobj).mtx[0][3]),
                0x200000 => Some((*jobj).mtx[1][3]),
                0x400000 => Some((*jobj).mtx[2][3]),
                0x800000 => Some(crate::hsd_mtx::get_scale(&(*jobj).mtx).x),
                0x1000000 => Some(crate::hsd_mtx::get_scale(&(*jobj).mtx).y),
                0x2000000 => Some(crate::hsd_mtx::get_scale(&(*jobj).mtx).z),
                _ => None,
            };
            if let Some(v) = val {
                *cur = v;
                cur = cur.add(1);
            }
            bit <<= 1;
        }
        rvalue = (*rvalue).next;
    }
    let mut fv = if (*exp).is_bytecode != 0 {
        let code = (*exp).expr.bytecode;
        // the bytecode ends itself; the slice only has to cover what the evaluator reads
        let code = if code.is_null() { None } else { Some(core::slice::from_raw_parts(code, 4096)) };
        super::bytecode::HSD_ByteCodeEval(
            code,
            core::slice::from_raw_parts(arg_buf, (*exp).nb_args as usize),
            (*exp).nb_args as i32,
            super::random::global::state(),
        )
    } else {
        ((*exp).expr.func.expect("expEvaluate: NULL expression function"))(arg_buf as *mut c_void)
    };
    if type_.wrapping_sub(1) <= 2 {
        fv *= 0.017453292f32;
    }
    let mut d = HSD_ObjData { fv };
    update(obj, type_ as enum_t, &mut d);
}

/// `dummy_func` (static): the function of an expression whose descriptor has none.
unsafe extern "C" fn dummy_func(_unused: *mut c_void) -> f32 {
    melee_core::ct!("dummy_func");
    0.0
}

/// `HSD_RvalueAlloc`.
pub unsafe fn HSD_RvalueAlloc() -> *mut HSD_Rvalue {
    melee_core::ct!("HSD_RvalueAlloc");
    let rvalue = super::objalloc::HSD_ObjAlloc(HSD_RvalueObjGetAllocData()) as *mut HSD_Rvalue;
    assert!(!rvalue.is_null(), "robj.c:1224 rvalue");
    core::ptr::write_bytes(rvalue, 0, 1);
    rvalue
}

/// `HSD_RvalueRemove`.
pub unsafe fn HSD_RvalueRemove(rvalue: *mut HSD_Rvalue) {
    melee_core::ct!("HSD_RvalueRemove");
    if !rvalue.is_null() {
        super::jobj::HSD_JObjUnrefThis((*rvalue).jobj);
        super::objalloc::HSD_ObjFree(HSD_RvalueObjGetAllocData(), rvalue as *mut c_void);
    }
}

/// `HSD_RvalueRemoveAll`.
pub unsafe fn HSD_RvalueRemoveAll(mut rvalue: *mut HSD_Rvalue) {
    melee_core::ct!("HSD_RvalueRemoveAll");
    while !rvalue.is_null() {
        let next = (*rvalue).next;
        HSD_RvalueRemove(rvalue);
        rvalue = next;
    }
}

/// `loadRvalue` (static): one `HSD_Rvalue` per list entry up to the entry with a NULL joint.
unsafe fn loadRvalue(mut list: *mut HSD_RvalueList) -> *mut HSD_Rvalue {
    melee_core::ct!("loadRvalue");
    if list.is_null() {
        return null_mut();
    }
    let mut head: *mut HSD_Rvalue = null_mut();
    let mut tail: *mut HSD_Rvalue = null_mut();
    while !(*list).joint.is_null() {
        let n = HSD_RvalueAlloc();
        (*n).flags = (*list).flags;
        if tail.is_null() {
            head = n;
        } else {
            (*tail).next = n;
        }
        tail = n;
        list = list.add(1);
    }
    head
}

/// `expLoadDesc` (static).
unsafe fn expLoadDesc(exp: *mut HSD_Exp, desc: *mut HSD_ExpDesc) {
    melee_core::ct!("expLoadDesc");
    core::ptr::write_bytes(exp, 0, 1);
    if !desc.is_null() {
        (*exp).expr.func = Some(match (*desc).func {
            Some(f) => f,
            None => dummy_func,
        });
        (*exp).rvalue = loadRvalue((*desc).rvalue);
        (*exp).nb_args = u32::MAX;
    }
}

/// `bcexpLoadDesc` (static).
unsafe fn bcexpLoadDesc(exp: *mut HSD_Exp, desc: *mut HSD_ByteCodeExpDesc) {
    melee_core::ct!("bcexpLoadDesc");
    core::ptr::write_bytes(exp, 0, 1);
    if !desc.is_null() {
        (*exp).expr.bytecode = (*desc).bytecode;
        (*exp).rvalue = loadRvalue((*desc).rvalue);
        (*exp).nb_args = u32::MAX;
        (*exp).is_bytecode = 1;
    }
}

/// `HSD_RvalueResolveRefs`.
pub unsafe fn HSD_RvalueResolveRefs(rvalue: *mut HSD_Rvalue, list: *mut HSD_RvalueList) {
    melee_core::ct!("HSD_RvalueResolveRefs");
    if !rvalue.is_null() && !list.is_null() {
        super::jobj::HSD_JObjUnrefThis((*rvalue).jobj);
        (*rvalue).jobj =
            super::id::HSD_IDGetDataFromTable(null_mut(), (*list).joint as usize, null_mut()) as *mut HSD_JObj;
        assert!(!(*rvalue).jobj.is_null(), "robj.c:1333 rvalue->jobj");
        jobj_ref_this((*rvalue).jobj);
    }
}

/// `HSD_RvalueResolveRefsAll`.
pub unsafe fn HSD_RvalueResolveRefsAll(mut rvalue: *mut HSD_Rvalue, mut list: *mut HSD_RvalueList) {
    melee_core::ct!("HSD_RvalueResolveRefsAll");
    if list.is_null() {
        return;
    }
    while !rvalue.is_null() && !(*list).joint.is_null() {
        HSD_RvalueResolveRefs(rvalue, list);
        rvalue = (*rvalue).next;
        list = list.add(1);
    }
}

/// `HSD_RObjSetConstraintObj`.
pub unsafe fn HSD_RObjSetConstraintObj(robj: *mut HSD_RObj, o: *mut c_void) {
    melee_core::ct!("HSD_RObjSetConstraintObj");
    if robj.is_null() {
        return;
    }
    if !(*robj).u.jobj.is_null() {
        super::jobj::HSD_JObjUnrefThis((*robj).u.jobj);
        (*robj).u.jobj = null_mut();
    }
    let jo = o as *mut HSD_JObj;
    if super::class::hsdObjIsDescendantOf(&mut (*jo).object, core::ptr::addr_of_mut!(super::jobj::hsdJObj) as *mut HSD_ClassInfo) {
        (*robj).u.jobj = jo;
        jobj_ref_this(jo);
    } else {
        super::os_error::OSReport(c"constraint only support jobj target.\n".as_ptr(), &[]);
        panic!("robj.c:1376 HSD_ASSERT(1376, 0)");
    }
}

// ---- folded from robj.rs (consolidation steps 21/22) ----
/// `set_dirup_matrix_fp` (static). `dir` / `uv` are written back as in the C.
pub fn set_dirup_matrix_fp<D, R>(
    dir: &mut Vec3_robj,
    uv: &mut Vec3_robj,
    scale: &Vec3_robj,
    tree: &mut JObjTree<D, R>,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) {
    melee_core::ct!("set_dirup_matrix_fp");
    let mut z = mtx_robj::vec_cross(dir, uv);
    let kdir = sqrtf_robj(1.0f32 / (EPS + mtx_robj::vec_dot(dir, dir)));
    *dir = mtx_robj::vec_scale(dir, kdir);
    let kz = sqrtf_robj(1.0f32 / (EPS + mtx_robj::vec_dot(&z, &z)));
    z = mtx_robj::vec_scale(&z, kz);
    *uv = mtx_robj::vec_cross(&z, dir);
    let v = Vec3_robj { x: dir.x * scale.x, y: dir.y * scale.x, z: dir.z * scale.x };
    update(tree, obj, 50, Some(RObjVal::Vec(v)));
    let v = Vec3_robj { x: uv.x * scale.y, y: uv.y * scale.y, z: uv.z * scale.y };
    update(tree, obj, 51, Some(RObjVal::Vec(v)));
    let v = Vec3_robj { x: z.x * scale.z, y: z.y * scale.z, z: z.z * scale.z };
    update(tree, obj, 52, Some(RObjVal::Vec(v)));
    update(tree, obj, 55, None);
}

/// `dummy_func_fp` (static).
pub fn dummy_func_fp(_args: &[f32]) -> f32 {
    melee_core::ct!("dummy_func_fp");
    0.0
}
