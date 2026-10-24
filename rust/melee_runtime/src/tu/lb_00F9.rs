//! Native port of lb_00F9.c: dynamics chains and aesthetic wind effects.
//! Shipping C contractions are explicit; the state/return/call oracle covers every TU definition.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::{fmadds, fmsubs};
use melee_types::inline as ji;
pub use melee_types::lb_Collider;

// // BEGIN GENERATED EXT (rust/melee_runtime/gen/lbtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code, unreachable_code)]
    use super::*;
    use core::ffi::c_void;
    pub mod hook {
        use super::*;
        pub static mut EulerToQuat: Option<fn(*mut Vec, *mut Quaternion) -> i32> = None;
        pub static mut GXBegin: Option<fn(u32, u32, u16)> = None;
        pub static mut GXColor4u8: Option<fn(u8, u8, u8, u8)> = None;
        pub static mut GXLoadPosMtxImm: Option<fn(*mut [f32; 4], u32)> = None;
        pub static mut GXPosition3f32: Option<fn(f32, f32, f32)> = None;
        pub static mut GXSetLineWidth: Option<fn(u8, u32)> = None;
        pub static mut HSD_CObjGetCurrent: Option<fn() -> *mut HSD_CObj> = None;
        pub static mut HSD_CObjGetViewingMtx: Option<fn(*mut HSD_CObj, *mut [f32; 4])> = None;
        pub static mut HSD_JObjClearFlagsAll: Option<fn(*mut HSD_JObj, u32)> = None;
        pub static mut HSD_JObjGetDObj: Option<fn(*mut HSD_JObj) -> *mut HSD_DObj> = None;
        pub static mut HSD_JObjSetMtxDirtySub: Option<fn(*mut HSD_JObj)> = None;
        pub static mut HSD_JObjSetupMatrixSub: Option<fn(*mut HSD_JObj)> = None;
        pub static mut HSD_MemAlloc: Option<fn(i32) -> *mut c_void> = None;
        pub static mut HSD_QuatLib_8037EB28: Option<fn(*mut [f32; 4], *mut Vec) -> i32> = None;
        pub static mut HSD_QuatLib_8037EC4C: Option<fn(*mut Quaternion, *mut Quaternion, *mut Quaternion) -> i32> = None;
        pub static mut HSD_QuatLib_8037ECE0: Option<fn(*mut Vec, *mut Quaternion, f32) -> i32> = None;
        pub static mut HSD_StateInitDirect: Option<fn(i32, u32)> = None;
        pub static mut PSMTXConcat: Option<fn(*mut [f32; 4], *mut [f32; 4], *mut [f32; 4])> = None;
        pub static mut PSMTXCopy: Option<fn(*mut [f32; 4], *mut [f32; 4])> = None;
        pub static mut PSMTXIdentity: Option<fn(*mut [f32; 4])> = None;
        pub static mut PSMTXMultVec: Option<fn(*mut [f32; 4], *mut Vec, *mut Vec)> = None;
        pub static mut PSMTXQuat: Option<fn(*mut [f32; 4], *mut Quaternion)> = None;
        pub static mut PSMTXScale: Option<fn(*mut [f32; 4], f32, f32, f32)> = None;
        pub static mut PSMTXTrans: Option<fn(*mut [f32; 4], f32, f32, f32)> = None;
        pub static mut PSMTXTranspose: Option<fn(*mut [f32; 4], *mut [f32; 4])> = None;
        pub static mut PSVECCrossProduct: Option<fn(*mut Vec, *mut Vec, *mut Vec)> = None;
        pub static mut __frsqrte: Option<fn(f64) -> f64> = None;
        pub static mut atan2f: Option<fn(f32, f32) -> f32> = None;
        pub static mut cosf: Option<fn(f32) -> f32> = None;
        pub static mut lbColl_80005C44: Option<fn(*const Vec, *const Vec, *const Vec, *mut Vec, f32, f32) -> i32> = None;
        pub static mut lbVector_Angle: Option<fn(*mut Vec, *mut Vec) -> f32> = None;
        pub static mut lbVector_CreateEulerMatrix: Option<fn(*mut [f32; 4], *mut Quaternion)> = None;
        pub static mut lbVector_Normalize: Option<fn(*mut Vec) -> f32> = None;
        pub static mut lbVector_RotateAboutUnitAxis: Option<fn(*mut Vec, *mut Vec, f32)> = None;
        pub static mut lb_80011B74: Option<fn(*mut HSD_DObj, u32)> = None;
        pub static mut lb_80011C18: Option<fn(*mut HSD_JObj, u32)> = None;
        pub static mut mpCheckFloor: Option<fn(f32, f32, f32, f32, f32, *mut Vec, *mut i32, *mut u32, *mut Vec, i32, i32, i32, Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, *mut HSD_GObj) -> i32> = None;
        pub static mut sinf: Option<fn(f32) -> f32> = None;
    }
    pub fn EulerToQuat(a0: *mut Vec, a1: *mut Quaternion) -> i32 {
        match unsafe { hook::EulerToQuat } { Some(f) => f(a0, a1), None => { panic!("TODO: EulerToQuat is not ported") } }
    }
    pub fn GXBegin(a0: u32, a1: u32, a2: u16) {
        match unsafe { hook::GXBegin } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GXBegin is not ported") } }
    }
    pub fn GXColor4u8(a0: u8, a1: u8, a2: u8, a3: u8) {
        match unsafe { hook::GXColor4u8 } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: GXColor4u8 is not ported") } }
    }
    pub fn GXLoadPosMtxImm(a0: *mut [f32; 4], a1: u32) {
        match unsafe { hook::GXLoadPosMtxImm } { Some(f) => f(a0, a1), None => { panic!("TODO: GXLoadPosMtxImm is not ported") } }
    }
    pub fn GXPosition3f32(a0: f32, a1: f32, a2: f32) {
        match unsafe { hook::GXPosition3f32 } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: GXPosition3f32 is not ported") } }
    }
    pub fn GXSetLineWidth(a0: u8, a1: u32) {
        match unsafe { hook::GXSetLineWidth } { Some(f) => f(a0, a1), None => { panic!("TODO: GXSetLineWidth is not ported") } }
    }
    pub fn HSD_CObjGetCurrent() -> *mut HSD_CObj {
        match unsafe { hook::HSD_CObjGetCurrent } { Some(f) => f(), None => { { return unsafe { crate::tu::cobj::HSD_CObjGetCurrent() as _ }; } } }
    }
    pub fn HSD_CObjGetViewingMtx(a0: *mut HSD_CObj, a1: *mut [f32; 4]) {
        match unsafe { hook::HSD_CObjGetViewingMtx } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_CObjGetViewingMtx is not ported") } }
    }
    pub fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) {
        match unsafe { hook::HSD_JObjClearFlagsAll } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_JObjClearFlagsAll is not ported") } }
    }
    pub fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj {
        match unsafe { hook::HSD_JObjGetDObj } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjGetDObj is not ported") } }
    }
    pub fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) {
        match unsafe { hook::HSD_JObjSetMtxDirtySub } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjSetMtxDirtySub is not ported") } }
    }
    pub fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj) {
        match unsafe { hook::HSD_JObjSetupMatrixSub } { Some(f) => f(a0), None => { panic!("TODO: HSD_JObjSetupMatrixSub is not ported") } }
    }
    pub fn HSD_MemAlloc(a0: i32) -> *mut c_void {
        match unsafe { hook::HSD_MemAlloc } { Some(f) => f(a0), None => { { return unsafe { crate::tu::memory::HSD_MemAlloc(a0 as _) as _ }; } } }
    }
    pub fn HSD_QuatLib_8037EB28(a0: *mut [f32; 4], a1: *mut Vec) -> i32 {
        match unsafe { hook::HSD_QuatLib_8037EB28 } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_QuatLib_8037EB28 is not ported") } }
    }
    pub fn HSD_QuatLib_8037EC4C(a0: *mut Quaternion, a1: *mut Quaternion, a2: *mut Quaternion) -> i32 {
        match unsafe { hook::HSD_QuatLib_8037EC4C } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_QuatLib_8037EC4C is not ported") } }
    }
    pub fn HSD_QuatLib_8037ECE0(a0: *mut Vec, a1: *mut Quaternion, a2: f32) -> i32 {
        match unsafe { hook::HSD_QuatLib_8037ECE0 } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: HSD_QuatLib_8037ECE0 is not ported") } }
    }
    pub fn HSD_StateInitDirect(a0: i32, a1: u32) {
        match unsafe { hook::HSD_StateInitDirect } { Some(f) => f(a0, a1), None => { panic!("TODO: HSD_StateInitDirect is not ported") } }
    }
    pub fn PSMTXConcat(a0: *mut [f32; 4], a1: *mut [f32; 4], a2: *mut [f32; 4]) {
        match unsafe { hook::PSMTXConcat } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: PSMTXConcat is not ported") } }
    }
    pub fn PSMTXCopy(a0: *mut [f32; 4], a1: *mut [f32; 4]) {
        match unsafe { hook::PSMTXCopy } { Some(f) => f(a0, a1), None => { panic!("TODO: PSMTXCopy is not ported") } }
    }
    pub fn PSMTXIdentity(a0: *mut [f32; 4]) {
        match unsafe { hook::PSMTXIdentity } { Some(f) => f(a0), None => { panic!("TODO: PSMTXIdentity is not ported") } }
    }
    pub fn PSMTXMultVec(a0: *mut [f32; 4], a1: *mut Vec, a2: *mut Vec) {
        match unsafe { hook::PSMTXMultVec } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: PSMTXMultVec is not ported") } }
    }
    pub fn PSMTXQuat(a0: *mut [f32; 4], a1: *mut Quaternion) {
        match unsafe { hook::PSMTXQuat } { Some(f) => f(a0, a1), None => { panic!("TODO: PSMTXQuat is not ported") } }
    }
    pub fn PSMTXScale(a0: *mut [f32; 4], a1: f32, a2: f32, a3: f32) {
        match unsafe { hook::PSMTXScale } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: PSMTXScale is not ported") } }
    }
    pub fn PSMTXTrans(a0: *mut [f32; 4], a1: f32, a2: f32, a3: f32) {
        match unsafe { hook::PSMTXTrans } { Some(f) => f(a0, a1, a2, a3), None => { panic!("TODO: PSMTXTrans is not ported") } }
    }
    pub fn PSMTXTranspose(a0: *mut [f32; 4], a1: *mut [f32; 4]) {
        match unsafe { hook::PSMTXTranspose } { Some(f) => f(a0, a1), None => { panic!("TODO: PSMTXTranspose is not ported") } }
    }
    pub fn PSVECCrossProduct(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) {
        match unsafe { hook::PSVECCrossProduct } { Some(f) => f(a0, a1, a2), None => { panic!("TODO: PSVECCrossProduct is not ported") } }
    }
    pub fn __frsqrte(a0: f64) -> f64 {
        match unsafe { hook::__frsqrte } { Some(f) => f(a0), None => { panic!("TODO: __frsqrte is not ported") } }
    }
    pub fn atan2f(a0: f32, a1: f32) -> f32 {
        match unsafe { hook::atan2f } { Some(f) => f(a0, a1), None => { { return unsafe { crate::lbtrigf::atan2f(a0 as _, a1 as _) as _ }; } } }
    }
    pub fn cosf(a0: f32) -> f32 {
        match unsafe { hook::cosf } { Some(f) => f(a0), None => { { return unsafe { crate::gekko::cosf(a0 as _) as _ }; } } }
    }
    pub fn lbColl_80005C44(a0: *const Vec, a1: *const Vec, a2: *const Vec, a3: *mut Vec, a4: f32, a5: f32) -> i32 {
        match unsafe { hook::lbColl_80005C44 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { { return unsafe { crate::tu::lbcollision::lbColl_80005C44(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _) as _ }; } } }
    }
    pub fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 {
        match unsafe { hook::lbVector_Angle } { Some(f) => f(a0, a1), None => { { return unsafe { crate::tu::lbvector::lbVector_Angle(a0 as _, a1 as _) as _ }; } } }
    }
    pub fn lbVector_CreateEulerMatrix(a0: *mut [f32; 4], a1: *mut Quaternion) {
        match unsafe { hook::lbVector_CreateEulerMatrix } { Some(f) => f(a0, a1), None => { { unsafe { let _ = crate::tu::lbvector::lbVector_CreateEulerMatrix(a0 as _, a1 as _); } return; } } }
    }
    pub fn lbVector_Normalize(a0: *mut Vec) -> f32 {
        match unsafe { hook::lbVector_Normalize } { Some(f) => f(a0), None => { { return unsafe { crate::tu::lbvector::lbVector_Normalize(a0 as _) as _ }; } } }
    }
    pub fn lbVector_RotateAboutUnitAxis(a0: *mut Vec, a1: *mut Vec, a2: f32) {
        match unsafe { hook::lbVector_RotateAboutUnitAxis } { Some(f) => f(a0, a1, a2), None => { { unsafe { let _ = crate::tu::lbvector::lbVector_RotateAboutUnitAxis(a0 as _, a1 as _, a2 as _); } return; } } }
    }
    pub fn lb_80011B74(a0: *mut HSD_DObj, a1: u32) {
        match unsafe { hook::lb_80011B74 } { Some(f) => f(a0, a1), None => { { unsafe { let _ = crate::tu::lbspdisplay::lb_80011B74(a0 as _, a1 as _); } return; } } }
    }
    pub fn lb_80011C18(a0: *mut HSD_JObj, a1: u32) {
        match unsafe { hook::lb_80011C18 } { Some(f) => f(a0, a1), None => { { unsafe { let _ = crate::tu::lbspdisplay::lb_80011C18(a0 as _, a1 as _); } return; } } }
    }
    pub fn mpCheckFloor(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32, a5: *mut Vec, a6: *mut i32, a7: *mut u32, a8: *mut Vec, a9: i32, a10: i32, a11: i32, a12: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, a13: *mut HSD_GObj) -> i32 {
        match unsafe { hook::mpCheckFloor } { Some(f) => f(a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13), None => { panic!("TODO: mpCheckFloor is not ported") } }
    }
    pub fn sinf(a0: f32) -> f32 {
        match unsafe { hook::sinf } { Some(f) => f(a0), None => { { return unsafe { crate::gekko::sinf(a0 as _) as _ }; } } }
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn atan2f(a0: f32, a1: f32) -> f32 { super::atan2f(a0, a1) }
        pub unsafe extern "C" fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 { super::lbVector_Angle(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub static lb_803B7280: [Vec; 3] = [Vec { x: 0.0, y: -1.0, z: 0.0 }; 3];
pub static mut lb_804D63A0: *mut lb_804D63A0_t = null_mut();
pub static mut cur_data: *mut DynamicsData = null_mut();
pub static mut lb_804D63A8: *mut lb_804D63A8_t = null_mut();
pub static mut lb_804D63AC: *mut lb_80011A50_t = null_mut();
pub static mut lb_804D63B0: *mut lb_80011A50_t = null_mut();
pub static mut lb_804D63B4: i32 = 0;
pub static mut lb_804D63B8: u8 = 0;

struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) { ji::HSD_JObjSetMtxDirtyInline(j, self); }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j); }
}
unsafe fn dirty(j: *mut HSD_JObj) {
    if !j.is_null() { ji::HSD_JObjSetMtxDirtyInline(&mut *j, &mut Dirty); }
}
unsafe fn setup_matrix(j: *mut HSD_JObj) {
    if !j.is_null() && ji::HSD_JObjMtxIsDirty(&*j) { ext::HSD_JObjSetupMatrixSub(j); }
}

pub unsafe fn checkJObjFlags(jobj: *mut HSD_JObj) -> bool {
    (*jobj).flags & ((1 << 5) | (1 << 14)) == 0
}
pub unsafe fn lb_8000F9F8(jobj: *mut HSD_JObj) {
    let child = ji::HSD_JObjGetChild(jobj.as_ref());
    if !child.is_null() { ext::lb_80011C18(child, 1 << 26); }
    if checkJObjFlags(jobj) && !ext::HSD_JObjGetDObj(jobj).is_null() {
        ext::lb_80011B74(ext::HSD_JObjGetDObj(jobj), 1 << 26);
    }
}
pub unsafe fn lb_8000FA94() {
    for i in 0..320 {
        (*lb_804D63A0).entries[i].desc.lb_unk0.jobj = null_mut();
        (*lb_804D63A0).entries[i].next = if i < 319 { &raw mut (*lb_804D63A0).entries[i + 1] } else { null_mut() };
    }
    cur_data = &raw mut (*lb_804D63A0).entries[0];
    for i in 0..8 {
        (*lb_804D63A8).entries[i].x0 = 0;
        (*lb_804D63A8).entries[i].next = if i < 7 { &raw mut (*lb_804D63A8).entries[i + 1] } else { null_mut() };
    }
    lb_804D63AC = &raw mut (*lb_804D63A8).entries[0];
    lb_804D63B0 = null_mut();
    lb_804D63B4 = 0;
}
pub unsafe fn lb_8000FCDC() {
    lb_804D63A0 = ext::HSD_MemAlloc(core::mem::size_of::<lb_804D63A0_t>() as i32).cast();
    lb_804D63A8 = ext::HSD_MemAlloc(core::mem::size_of::<lb_804D63A8_t>() as i32).cast();
    lb_8000FA94();
}
pub unsafe fn lb_8000FD18(desc: *mut DynamicsDesc) {
    let mut cur = (*desc).data;
    while !cur.is_null() {
        let next = (*cur).next;
        (*cur).next = cur_data;
        cur_data = cur;
        cur = next;
    }
    (*desc).data = null_mut();
}
pub unsafe fn popDynamicsData() -> *mut DynamicsData {
    let entry = cur_data;
    if entry.is_null() { return null_mut(); }
    cur_data = (*entry).next;
    (*entry).next = null_mut();
    entry
}
pub unsafe fn lb_8000FD48(mut jobj: *mut HSD_JObj, desc: *mut DynamicsDesc, max_count: usize) {
    if desc.is_null() { return; }
    if jobj.is_null() { (*desc).data = null_mut(); return; }
    (*desc).count = 0;
    let mut prev: *mut DynamicsData = null_mut();
    while ((*desc).count as i32) < max_count as i32 {
        if (*desc).count == 0 {
            prev = popDynamicsData();
            (*desc).data = prev;
        } else {
            (*prev).next = popDynamicsData();
            prev = (*prev).next;
        }
        if prev.is_null() { return; }
        dirty(jobj);
        setup_matrix(jobj);
        let d = &mut (*prev).desc.lb_unk0;
        d.jobj = jobj;
        d.rotate = (*jobj).rotate;
        d.scale = (*jobj).scale;
        d.translate = (*jobj).translate;
        d.unk_2C = Vec { x: (*jobj).mtx[0][3], y: (*jobj).mtx[1][3], z: (*jobj).mtx[2][3] };
        d.unk_58 = d.rotate;
        d.unk_38 = Vec { x: 1.0, y: 0.0, z: 0.0 };
        d.unk_44 = 0.0;
        d.unk_8C = 0.0;
        d.unk_48 = 0.0;
        (*prev).next = null_mut();
        (*prev).unk_94 = 0;
        jobj = (*jobj).child;
        (*desc).count = (*desc).count.wrapping_add(1);
    }
    prev = (*desc).data;
    while !(*prev).next.is_null() {
        let next = (*prev).next;
        let a = &mut (*prev).desc.lb_unk0;
        let b = &(*next).desc.lb_unk0;
        let dx = a.unk_2C.x - b.unk_2C.x;
        let dy = a.unk_2C.y - b.unk_2C.y;
        let dz = a.unk_2C.z - b.unk_2C.z;
        a.unk_48 = crate::gekko::sqrtf(fmadds(dz, dz, fmadds(dx, dx, dy * dy)));
        let tx = absf(b.translate.x);
        let ty = absf(b.translate.y);
        let tz = absf(b.translate.z);
        a.unk_54 = if tz > ty { if tz > tx { -1 } else { 0 } } else if ty > tx { 1 } else { 0 };
        prev = next;
    }
}
pub unsafe fn inlineA0() -> *mut lb_80011A50_t {
    let ret = lb_804D63AC;
    if ret.is_null() { return null_mut(); }
    lb_804D63AC = (*ret).next;
    (*ret).next = lb_804D63B0;
    lb_804D63B0 = ret;
    ret
}
pub unsafe fn inlineA1() -> *mut lb_80011A50_t {
    let mut cur = lb_804D63B0;
    if cur.is_null() { return null_mut(); }
    let mut ret = cur;
    let mut max = (*cur).x1;
    while !cur.is_null() {
        if max < (*cur).x1 { max = (*cur).x1; ret = cur; }
        cur = (*cur).next;
    }
    ret
}
pub unsafe fn lb_800100B0(arg0: *mut lb_80011A50_t, _arg1: f32) -> *mut lb_80011A50_t {
    let mut ret = inlineA0();
    if ret.is_null() {
        ret = inlineA1();
        if ret.is_null() || (*ret).x1 < (*arg0).x1 { return null_mut(); }
    }
    (*ret).x0 = (*arg0).x0;
    (*ret).x1 = (*arg0).x1;
    (*ret).x4 = (*arg0).x4;
    (*ret).unk_scale = (*arg0).unk_scale;
    (*ret).x24 = (*arg0).x24;
    (*ret).unk_count0 = (*arg0).unk_count0;
    (*ret).unk_angle_float = (*arg0).unk_angle_float;
    (*ret).x10 = (*arg0).x10;
    (*ret).x14 = (*arg0).x14;
    (*ret).x18 = (*arg0).x18;
    (*ret).x1C = (*arg0).x1C;
    (*ret).unk_angle_int = 0;
    ret
}
pub unsafe fn lb_800101C8(arg0: *mut Vec, arg1: *mut Vec) -> f32 {
    let mut cur = lb_804D63B0;
    *arg1 = Vec { x: 0.0, y: 0.0, z: 0.0 };
    while !cur.is_null() {
        let d = &*cur;
        let scale0 = (0.5f64 * d.unk_scale as f64 * (1.0 + ext::cosf(d.unk_angle_int as f32 * d.unk_angle_float) as f64)) as f32;
        if d.x0 == 1 {
            if (*arg0).x > d.x10 && (*arg0).x < d.x18 && (*arg0).y < d.x14 && (*arg0).y > d.x1C {
                (*arg1).x = fmadds(d.x4.x, scale0, (*arg1).x);
                (*arg1).y = fmadds(d.x4.y, scale0, (*arg1).y);
                (*arg1).z = fmadds(d.x4.z, scale0, (*arg1).z);
            }
        } else {
            let mut delta = Vec { x: (*arg0).x - d.x4.x, y: (*arg0).y - d.x4.y, z: (*arg0).z - d.x4.z };
            let mut v = (0.05f64 * ext::lbVector_Normalize(&mut delta) as f64) as f32;
            if (v as f64) < 1.0 { v = 1.0; }
            let scale1 = (1.0f64 / (v * v) as f64) as f32;
            (*arg1).x = fmadds(scale1, delta.x * scale0, (*arg1).x);
            (*arg1).y = fmadds(scale1, delta.y * scale0, (*arg1).y);
            (*arg1).z = fmadds(scale1, delta.z * scale0, (*arg1).z);
        }
        cur = d.next;
    }
    ext::lbVector_Normalize(arg1)
}
pub unsafe fn lb_800103B8(a: *mut Vec, b: *mut Vec) -> f32 { lb_800101C8(a, b) }
pub unsafe fn lb_800103D8(vec: *mut Vec, x0: f32, x1: f32, x2: f32, x3: f32, offset: f32) -> bool {
    let dist0 = x1 - offset;
    let dist1 = x3 - offset;
    if dist0 == dist1 { return dist0 > 1e-10f32; }
    if dist0 as f64 > 0.0 && (dist1 as f64) < 0.0 {
        (*vec).x = fmadds(-dist1 / (dist0 - dist1), x0 - x2, x2);
        (*vec).y = offset;
        (*vec).z = 0.0;
        return true;
    }
    false
}
pub fn approximatelyZeroVec3(vec: Vec) -> bool {
    !(vec.x >= 0.00001 || vec.x <= -0.00001 || vec.y >= 0.00001 || vec.y <= -0.00001 || vec.z >= 0.00001 || vec.z <= -0.00001)
}
pub fn absf(x: f32) -> f32 { if x < 0.0 { -x } else { x } }
pub unsafe fn groundHeight(data: *mut DynamicsData, floor_point: *mut Vec) -> f32 {
    absf((*data).desc.lb_unk0.unk_2C.y - (*floor_point).y)
}

unsafe fn set_rot_axis(j: *mut HSD_JObj, axis: i32, v: f32) {
    match axis { 0 => (*j).rotate.x = v, 1 => (*j).rotate.y = v, _ => (*j).rotate.z = v }
    if (*j).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 { dirty(j); }
}
#[inline(always)]
fn mp(m: &mut Mtx) -> *mut [f32; 4] { m.as_mut_ptr() as *mut [f32; 4] }

pub unsafe fn lb_8001044C(desc: *mut DynamicsDesc, colliders_raw: *mut c_void, num_colliders: i32,
                          pos_y: f32, use_floor_fn: bool, part: Fighter_Part,
                          first_active: i32, ground_check: bool) {
    let mut parent_mtx: Mtx = core::mem::zeroed();
    let mut temp_mtx: Mtx = core::mem::zeroed();
    let mut bone_mtx: Mtx = core::mem::zeroed();
    let mut constrained_mtx: Mtx = core::mem::zeroed();
    let mut trans_mtx: Mtx = core::mem::zeroed();
    let mut scale_mtx: Mtx = core::mem::zeroed();
    let mut natural_dir: Vec = core::mem::zeroed();
    let mut current_dir: Vec = core::mem::zeroed();
    let mut link_dir: Vec = core::mem::zeroed();
    let mut rotation_axis: Vec = core::mem::zeroed();
    let mut local_axis: Vec = core::mem::zeroed();
    let mut euler_angles: Vec = core::mem::zeroed();
    let mut angle_quat: Quaternion = core::mem::zeroed();
    let mut euler_quat: Quaternion = core::mem::zeroed();
    let mut result_quat: Quaternion = core::mem::zeroed();
    let mut on_ground: i32 = 0;

    if lb_804D63B8 != 0 { return; }
    if desc.is_null() { return; }
    let mut cur = (*desc).data;
    if cur.is_null() { return; }
    if part > 0xFF { return; }

    let mut loop_index: i32 = 0;
    let mut i = 0i32;
    while i < part as i32 {
        cur = (*cur).next;
        loop_index += 1;
        i += 1;
    }

    let parent_jobj = (*(*cur).desc.lb_unk0.jobj).parent;
    if !parent_jobj.is_null() {
        dirty(parent_jobj);
        setup_matrix(parent_jobj);
        ext::PSMTXCopy(mp(&mut (*parent_jobj).mtx), mp(&mut parent_mtx));
    } else {
        ext::PSMTXIdentity(mp(&mut parent_mtx));
    }

    let mut jobj = (*cur).desc.lb_unk0.jobj;

    while !(*cur).next.is_null() {
        let angle_diff: f32;
        let mut force_mag: f32;
        let d = &mut (*cur).desc.lb_unk0;

        ext::PSMTXTrans(mp(&mut trans_mtx), (*jobj).translate.x, (*jobj).translate.y, (*jobj).translate.z);
        ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut trans_mtx), mp(&mut bone_mtx));

        ext::lbVector_CreateEulerMatrix(mp(&mut temp_mtx), &mut d.unk_58);
        ext::PSMTXConcat(mp(&mut bone_mtx), mp(&mut temp_mtx), mp(&mut constrained_mtx));

        ext::lbVector_CreateEulerMatrix(mp(&mut temp_mtx), &mut (*jobj).rotate);
        ext::PSMTXConcat(mp(&mut bone_mtx), mp(&mut temp_mtx), mp(&mut bone_mtx));

        ext::PSMTXScale(mp(&mut scale_mtx), (*jobj).scale.x, (*jobj).scale.y, (*jobj).scale.z);
        ext::PSMTXConcat(mp(&mut bone_mtx), mp(&mut scale_mtx), mp(&mut bone_mtx));
        ext::PSMTXConcat(mp(&mut constrained_mtx), mp(&mut scale_mtx), mp(&mut constrained_mtx));

        d.unk_2C.x = bone_mtx[0][3];
        d.unk_2C.y = bone_mtx[1][3];
        d.unk_2C.z = bone_mtx[2][3];

        let child_t: *mut Vec = &mut (*(*jobj).child).translate;
        ext::PSMTXMultVec(mp(&mut constrained_mtx), child_t, &mut natural_dir);
        natural_dir.x -= d.unk_2C.x;
        natural_dir.y -= d.unk_2C.y;
        natural_dir.z -= d.unk_2C.z;

        ext::PSMTXMultVec(mp(&mut bone_mtx), child_t, &mut current_dir);
        current_dir.x -= d.unk_2C.x;
        current_dir.y -= d.unk_2C.y;
        current_dir.z -= d.unk_2C.z;

        let nx = (*(*cur).next).desc.lb_unk0.unk_2C;
        link_dir.x = nx.x - d.unk_2C.x;
        link_dir.y = nx.y - d.unk_2C.y;
        link_dir.z = nx.z - d.unk_2C.z;

        ext::lbVector_Normalize(&mut natural_dir);
        ext::lbVector_Normalize(&mut current_dir);
        ext::lbVector_Normalize(&mut link_dir);

        let mut saved_dir: Vec = link_dir;

        // stiffness blend
        if d.unk_4C * (*desc).pos.x < 1.0 {
            let stiff_angle = absf(ext::lbVector_Angle(&mut link_dir, &mut current_dir));
            if stiff_angle != 0.0 {
                let mut stiffness_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut link_dir, &mut current_dir, &mut stiffness_axis);
                ext::lbVector_Normalize(&mut stiffness_axis);
                ext::lbVector_RotateAboutUnitAxis(
                    &mut link_dir, &mut stiffness_axis,
                    stiff_angle * ((1.0f64 - (d.unk_4C * (*desc).pos.x) as f64) as f32));
            }
        }

        // gravity
        {
            let mut grav_dir: Vec = lb_803B7280[0];
            let grav_angle = ext::lbVector_Angle(&mut link_dir, &mut grav_dir);
            if grav_angle != 0.0 {
                let mut grav_rot = d.unk_8C * ext::sinf(grav_angle);
                if grav_rot < 0.0 { grav_rot = -grav_rot; }
                let mut gravity_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut link_dir, &mut grav_dir, &mut gravity_axis);
                ext::lbVector_Normalize(&mut gravity_axis);
                ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut gravity_axis, grav_rot);
            }
        }

        // interaction force
        force_mag = 0.0;
        if !lb_804D63B0.is_null() && first_active <= loop_index {
            let mut next_pos: Vec = (*(*cur).next).desc.lb_unk0.unk_2C;
            let mut force_dir: Vec = core::mem::zeroed();
            force_mag = lb_800101C8(&mut next_pos, &mut force_dir);
            let force_angle = ext::lbVector_Angle(&mut link_dir, &mut force_dir);
            if force_angle != 0.0 {
                let mut force_rot = (force_mag * ext::sinf(force_angle)) / d.unk_48;
                if force_rot < 0.0 { force_rot = -force_rot; }
                let mut force_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut link_dir, &mut force_dir, &mut force_axis);
                ext::lbVector_Normalize(&mut force_axis);
                ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut force_axis, force_rot);
            }
        }

        // angular velocity rotation
        if 0.0 != d.unk_44 {
            ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut d.unk_38, d.unk_44);
        }

        // max angle constraint
        if force_mag < d.unk_8C * d.unk_48 {
            let mut clamp_dir: Vec = saved_dir;
            if ext::lbVector_Angle(&mut clamp_dir, &mut link_dir) > d.unk_88 {
                let mut clamp_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut clamp_dir, &mut link_dir, &mut clamp_axis);
                ext::lbVector_Normalize(&mut clamp_axis);
                ext::lbVector_RotateAboutUnitAxis(&mut clamp_dir, &mut clamp_axis, d.unk_88);
                link_dir = clamp_dir;
            }
        }

        // convergence limit
        if d.unk_50 > 0.0 {
            if ext::lbVector_Angle(&mut natural_dir, &mut link_dir) < d.unk_50 {
                link_dir = natural_dir;
            } else {
                let mut convergence_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut link_dir, &mut natural_dir, &mut convergence_axis);
                ext::lbVector_Normalize(&mut convergence_axis);
                ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut convergence_axis, d.unk_50);
            }
        }

        // max angle deviation
        {
            let dev_angle = ext::lbVector_Angle(&mut natural_dir, &mut link_dir);
            if dev_angle > d.unk_68 {
                let mut deviation_axis: Vec = core::mem::zeroed();
                ext::PSVECCrossProduct(&mut link_dir, &mut natural_dir, &mut deviation_axis);
                ext::lbVector_Normalize(&mut deviation_axis);
                ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut deviation_axis, dev_angle - d.unk_68);
            }
        }

        // collider avoidance
        if num_colliders != 0 {
            ext::lbVector_Normalize(&mut link_dir);
            let mut collider = colliders_raw as *mut lb_Collider;
            let mut ci = 0;
            while ci < num_colliders {
                let mut next_bone_pos: Vec = core::mem::zeroed();
                next_bone_pos.x = fmadds(link_dir.x, d.unk_48, d.unk_2C.x);
                next_bone_pos.y = fmadds(link_dir.y, d.unk_48, d.unk_2C.y);
                next_bone_pos.z = fmadds(link_dir.z, d.unk_48, d.unk_2C.z);

                let mut coll_dir: Vec = core::mem::zeroed();
                coll_dir.x = (*collider).position.x - d.unk_2C.x;
                coll_dir.y = (*collider).position.y - d.unk_2C.y;
                coll_dir.z = (*collider).position.z - d.unk_2C.z;

                let coll_dist = crate::gekko::sqrtf(fmadds(coll_dir.z, coll_dir.z,
                    fmadds(coll_dir.x, coll_dir.x, coll_dir.y * coll_dir.y)));
                let mut collision_point: Vec = core::mem::zeroed();
                if coll_dist > (*collider).radius
                    && ext::lbColl_80005C44(&d.unk_2C, &next_bone_pos, &(*collider).position,
                                            &mut collision_point, 0.1, (*collider).radius) != 0
                {
                    let coll_angle = ext::lbVector_Angle(&mut coll_dir, &mut link_dir);
                    if 0.0 != coll_angle {
                        let adj_radius = (0.1f64 + (*collider).radius as f64) as f32;
                        let side_sq = crate::gekko::sqrtf(fmsubs(coll_dist, coll_dist, adj_radius * adj_radius));
                        let mut avoidance_angle = ext::atan2f(adj_radius, side_sq);
                        if avoidance_angle < 0.0 { avoidance_angle = -avoidance_angle; }
                        avoidance_angle -= coll_angle;
                        if avoidance_angle > 0.0 {
                            let mut avoidance_axis: Vec = core::mem::zeroed();
                            ext::PSVECCrossProduct(&mut coll_dir, &mut link_dir, &mut avoidance_axis);
                            ext::lbVector_Normalize(&mut avoidance_axis);
                            ext::lbVector_RotateAboutUnitAxis(&mut link_dir, &mut avoidance_axis, avoidance_angle);
                        }
                    }
                }
                collider = collider.add(1);
                ci += 1;
            }
        }

        // ground collision
        if ground_check {
            let mut floor_point: Vec = core::mem::zeroed();
            let mut floor_normal: Vec = core::mem::zeroed();
            let mut line_id: i32 = 0;
            let mut floor_flags: u32 = 0;
            let mut floor_cross: Vec = core::mem::zeroed();
            if on_ground != 0 {
                let mut gnd_norm: Vec = lb_803B7280[1];
                ext::lbVector_Normalize(&mut link_dir);
                let end_x = fmadds(link_dir.x, d.unk_48, d.unk_2C.x);
                let end_y = fmadds(link_dir.y, d.unk_48, d.unk_2C.y);
                let floor_hit: bool = if use_floor_fn {
                    lb_800103D8(&mut floor_point, end_x, (1.0f64 + end_y as f64) as f32, end_x,
                                (end_y as f64 - 1.0) as f32, pos_y)
                } else {
                    ext::mpCheckFloor(end_x, (1.0f64 + end_y as f64) as f32, end_x,
                                      (end_y as f64 - 1.0) as f32, 0.1, &mut floor_point,
                                      &mut line_id, &mut floor_flags, &mut floor_normal,
                                      -1, -1, -1, None, null_mut()) != 0
                };
                ext::PSVECCrossProduct(&mut gnd_norm, &mut link_dir, &mut floor_cross);
                ext::lbVector_Normalize(&mut floor_cross);
                ext::lbVector_RotateAboutUnitAxis(&mut gnd_norm, &mut floor_cross, 1.5707964);
                link_dir = gnd_norm;
                on_ground = if floor_hit { 1 } else { 0 };
            } else {
                let mut gnd_norm2: Vec = lb_803B7280[2];
                ext::lbVector_Normalize(&mut link_dir);
                let floor_hit2: bool = if use_floor_fn {
                    lb_800103D8(&mut floor_point, d.unk_2C.x, (1.0f64 + d.unk_2C.y as f64) as f32,
                                d.unk_2C.x, d.unk_2C.y, pos_y)
                } else {
                    ext::mpCheckFloor(d.unk_2C.x, (1.0f64 + d.unk_2C.y as f64) as f32, d.unk_2C.x,
                                      d.unk_2C.y, 0.1, &mut floor_point, &mut line_id,
                                      &mut floor_flags, &mut floor_normal, -1, -1, -1, None,
                                      null_mut()) != 0
                };
                if floor_hit2 {
                    ext::PSVECCrossProduct(&mut gnd_norm2, &mut link_dir, &mut floor_cross);
                    ext::lbVector_Normalize(&mut floor_cross);
                    ext::lbVector_RotateAboutUnitAxis(&mut gnd_norm2, &mut floor_cross, 1.5707964);
                    on_ground = 1;
                    link_dir = gnd_norm2;
                } else {
                    let end_x2 = fmadds(link_dir.x, d.unk_48, d.unk_2C.x);
                    let end_y2 = fmadds(link_dir.y, d.unk_48, d.unk_2C.y);
                    let floor_hit3: bool = if use_floor_fn {
                        lb_800103D8(&mut floor_point, d.unk_2C.x, d.unk_2C.y, end_x2, end_y2, pos_y)
                    } else {
                        ext::mpCheckFloor(d.unk_2C.x, d.unk_2C.y, end_x2, end_y2, 0.1,
                                          &mut floor_point, &mut line_id, &mut floor_flags,
                                          &mut floor_normal, -1, -1, -1, None, null_mut()) != 0
                    };
                    if floor_hit3 {
                        let height_diff = groundHeight(cur, &mut floor_point);
                        let horiz_dist = crate::gekko::sqrtf(-fmsubs(height_diff, height_diff, d.unk_48 * d.unk_48));
                        let ground_angle = absf(ext::atan2f(horiz_dist, height_diff));
                        ext::PSVECCrossProduct(&mut gnd_norm2, &mut link_dir, &mut floor_cross);
                        ext::lbVector_Normalize(&mut floor_cross);
                        ext::lbVector_RotateAboutUnitAxis(&mut gnd_norm2, &mut floor_cross, ground_angle);
                        on_ground = 1;
                        link_dir = gnd_norm2;
                    } else {
                        on_ground = 0;
                    }
                }
            }
        }

        // angular difference and rotation axis
        angle_diff = ext::lbVector_Angle(&mut current_dir, &mut link_dir);
        ext::PSVECCrossProduct(&mut current_dir, &mut link_dir, &mut rotation_axis);
        ext::lbVector_Normalize(&mut rotation_axis);

        let mut adj_dir: Vec = current_dir;
        ext::lbVector_RotateAboutUnitAxis(&mut adj_dir, &mut rotation_axis, angle_diff);
        link_dir = adj_dir;

        // store angular velocity
        if on_ground != 0 {
            d.unk_44 = 0.0;
        } else {
            d.unk_44 = ext::lbVector_Angle(&mut saved_dir, &mut link_dir);
        }

        {
            let mut ang_vel = d.unk_44;
            if ang_vel < 0.0 { ang_vel = -ang_vel; }
            if ang_vel > 0.0 {
                ext::PSVECCrossProduct(&mut saved_dir, &mut link_dir, &mut d.unk_38);
                ext::lbVector_Normalize(&mut d.unk_38);
            }
        }

        {
            let ang_vel2 = d.unk_44;
            if 0.0 != ang_vel2 {
                let damping = d.unk_84;
                if ang_vel2 > damping {
                    d.unk_44 = ang_vel2 - damping;
                } else if ang_vel2 < -damping {
                    d.unk_44 = ang_vel2 + damping;
                } else {
                    d.unk_44 = 0.0;
                }
            }
        }

        // update JObj rotation via quaternion
        if !(angle_diff < 0.00001 && angle_diff > -0.00001) {
            ext::PSMTXTranspose(mp(&mut parent_mtx), mp(&mut bone_mtx));
            ext::PSMTXMultVec(mp(&mut bone_mtx), &mut rotation_axis, &mut local_axis);
            if !approximatelyZeroVec3(local_axis) {
                let mut rotation: Quaternion = core::mem::zeroed();
                ext::HSD_QuatLib_8037ECE0(&mut local_axis, &mut angle_quat, angle_diff);
                euler_angles.x = (*jobj).rotate.x;
                euler_angles.y = (*jobj).rotate.y;
                euler_angles.z = (*jobj).rotate.z;
                ext::EulerToQuat(&mut euler_angles, &mut euler_quat);
                ext::HSD_QuatLib_8037EC4C(&mut angle_quat, &mut euler_quat, &mut result_quat);
                ext::PSMTXQuat(mp(&mut bone_mtx), &mut result_quat);
                ext::HSD_QuatLib_8037EB28(mp(&mut bone_mtx), &mut rotation as *mut Quaternion as *mut Vec);
                // HSD_JObjSetRotation
                (*jobj).rotate = rotation;
                if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 { dirty(jobj); }
                ext::HSD_JObjClearFlagsAll(jobj, 0x20000);
            }
        }

        // dominant axis damping
        {
            let axis = d.unk_54;
            if axis == -1 {
                let r = (*jobj).rotate.z * 0.9;
                set_rot_axis(jobj, 2, r);
            } else if axis == 0 {
                let r = (*jobj).rotate.x * 0.9;
                set_rot_axis(jobj, 0, r);
            } else {
                let r = (*jobj).rotate.y * 0.9;
                set_rot_axis(jobj, 1, r);
            }
        }

        // update parent matrix for next iteration
        ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut trans_mtx), mp(&mut parent_mtx));
        ext::lbVector_CreateEulerMatrix(mp(&mut temp_mtx), &mut (*jobj).rotate);
        ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut temp_mtx), mp(&mut parent_mtx));
        ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut scale_mtx), mp(&mut parent_mtx));

        jobj = (*jobj).child;
        loop_index += 1;
        cur = (*cur).next;
    }

    // tail node
    ext::PSMTXTrans(mp(&mut temp_mtx), (*jobj).translate.x, (*jobj).translate.y, (*jobj).translate.z);
    ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut temp_mtx), mp(&mut parent_mtx));
    ext::lbVector_CreateEulerMatrix(mp(&mut temp_mtx), &mut (*jobj).rotate);
    ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut temp_mtx), mp(&mut parent_mtx));
    ext::PSMTXScale(mp(&mut temp_mtx), (*jobj).scale.x, (*jobj).scale.y, (*jobj).scale.z);
    ext::PSMTXConcat(mp(&mut parent_mtx), mp(&mut temp_mtx), mp(&mut parent_mtx));
    let d = &mut (*cur).desc.lb_unk0;
    d.unk_2C.x = parent_mtx[0][3];
    d.unk_2C.y = parent_mtx[1][3];
    d.unk_2C.z = parent_mtx[2][3];
}
pub unsafe fn lb_800115F4() {
    let mut total_scale = 0.0f32;
    let mut cur = lb_804D63B0;
    while !cur.is_null() {
        if (*cur).x0 == 1 { total_scale += (*cur).unk_scale; }
        (*cur).unk_scale -= (*cur).x24;
        if ((*cur).unk_scale as f64) < 0.0 { (*cur).unk_scale = 0.0; }
        if (*cur).unk_count0 > 0 { (*cur).unk_count0 -= 1; }
        (*cur).unk_angle_int = (*cur).unk_angle_int.wrapping_add(1);
        if (*cur).unk_count0 == 0 {
            let prev = cur;
            cur = (*cur).next;
            let mut head = lb_804D63B0;
            if prev == head { lb_804D63B0 = cur; }
            else {
                while (*head).next != prev { head = (*head).next; }
                (*head).next = (*prev).next;
            }
            (*prev).next = lb_804D63AC;
            lb_804D63AC = prev;
        } else { cur = (*cur).next; }
    }
    lb_804D63B4 = if total_scale as f64 > 0.1 {
        if lb_804D63B4 > 0 { 2 } else { 1 }
    } else if lb_804D63B4 > 0 { -1 } else { 0 };
}
pub unsafe fn lb_80011710(arg0: *mut DynamicsDesc, arg1: *mut DynamicsDesc) {
    (*arg1).pos = (*arg0).pos;
    let data0 = (&raw const (*(*arg0).data).desc.lb_unk1.array).cast::<lb_00F9_UnkDesc1Inner>();
    let mut data1 = (*arg1).data;
    for i in 0..((*arg0).count as i32) {
        let a = &*data0.add(i as usize);
        let b = &mut (*data1).desc.lb_unk0;
        b.unk_4C = a.unk_0;
        b.unk_50 = a.unk_4;
        b.unk_58 = a.unk_8;
        b.unk_68 = a.unk_18;
        b.unk_6C = a.unk_1C;
        b.unk_78 = a.unk_28;
        b.unk_84 = a.unk_34;
        b.unk_88 = a.unk_38;
        b.unk_8C = if b.unk_48 as f64 != 0.0 { (*arg0).pos.z / b.unk_48 } else { 0.0 };
        data1 = (*data1).next;
    }
}
pub unsafe fn lb_800117F4(arg0: *mut DynamicsDesc, arg1: *mut GXColor, _arg2: *mut GXColor, arg3: i32, arg4: u32) -> bool {
    if (*arg1).a < 255 { if arg4 != 2 { return false; } }
    else if arg4 != 0 { return false; }
    let mut view = [[0.0f32; 4]; 3];
    ext::HSD_StateInitDirect(0, 2);
    ext::HSD_CObjGetViewingMtx(ext::HSD_CObjGetCurrent(), view.as_mut_ptr());
    ext::GXLoadPosMtxImm(view.as_mut_ptr(), 0);
    ext::GXSetLineWidth(12, 5);
    ext::GXBegin(0xB0, 0, (*arg0).count as u16);
    let mut cur = (*arg0).data;
    let mut i = 0i32;
    while !cur.is_null() {
        let j = (*cur).desc.lb_unk0.jobj;
        dirty(j);
        setup_matrix(j);
        ext::GXPosition3f32((*j).mtx[0][3], (*j).mtx[1][3], (*j).mtx[2][3]);
        let c = &*arg1;
        if i < arg3 { ext::GXColor4u8(c.r / 2, c.g / 2, c.b / 2, c.a); }
        else { ext::GXColor4u8(c.r, c.g, c.b, c.a); }
        cur = (*cur).next;
        i = i.wrapping_add(1);
    }
    true
}
pub unsafe fn lb_800119DC(arg0: *mut Vec, arg1: i32, arg2: f32, arg3: f32, arg4: f32) {
    let mut e: lb_80011A50_t = core::mem::zeroed();
    e.x0 = 2; e.x1 = 100; e.x4 = *arg0;
    e.unk_scale = arg2; e.x24 = arg3; e.unk_count0 = arg1; e.unk_angle_float = arg4;
    e.x10 = -10000.0; e.x14 = 10000.0; e.x18 = 10000.0; e.x1C = -10000.0;
    lb_800100B0(&mut e, arg2);
}
pub unsafe fn lb_80011A50(arg0: *mut Vec, arg1: i32, arg2: f32, arg3: f32, arg4: f32, arg5: f32, arg6: f32, arg7: f32, arg8: f32) -> *mut lb_80011A50_t {
    let mut e: lb_80011A50_t = core::mem::zeroed();
    e.x0 = 1; e.x1 = 0; e.x4 = *arg0;
    e.unk_scale = arg2; e.x24 = arg3; e.unk_count0 = arg1; e.unk_angle_float = arg4;
    e.x10 = arg5; e.x14 = arg6; e.x18 = arg7; e.x1C = arg8;
    lb_800100B0(&mut e, arg2)
}
pub unsafe fn lb_80011ABC() -> i32 { lb_804D63B4 }
