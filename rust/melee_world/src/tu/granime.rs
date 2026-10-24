//! src/melee/gr/granime.c: stage map animation: loading AnimJoint / MatAnimJoint / ShapeAnimJoint trees onto a JObj
//! tree, walking every AObj of a model with a callback (`grAnime_801C752C`), and the `Ground_801C3FA4`-based
//! convenience wrappers. No float arithmetic (every float is stored or passed on).
//!
//! Native adaptations (numerically and observably identical to the PPC build):
//! * **callbacks**: the C stores `void* func` and calls it through one of five signatures chosen by `type`; the
//!   registers line up on the PPC (floats in f1.., ints in r3..) but not on x86-64 (positional), so callbacks are the
//!   uniform [`AnimCallback`] `(aobj, obj, flags, arg_bits)`; `arg_bits` is the 4 bytes at `*param` (a float's bits for
//!   the float types) and 0 for the types that pass nothing. [`ext::HSD_AObjClearFlags_cb`] and friends adapt the
//!   engine functions the C passes as callbacks.
//! * **varargs**: `grAnime_801C752C` takes its single vararg as a raw 64-bit word (`va`, a `f64`'s bits for the
//!   float types).
//! * **setjmp/longjmp**: `grAnime_801C8318` catches an unwind instead; `fn_801C82E8` raises it (`C-unwind`).
//! State-oracle checked against the C (tests/gro_granime.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

/// `grAnime_801C6F50`'s callback: `(aobj, obj, flags, arg_bits)`.
pub type AnimCallback = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, u32, u32);

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::hook;
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::HSD_JObjSetMtxDirtySub;
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, i32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
    }
    pub static mut GROUND_801C498C: Option<fn() -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C498C() -> *mut HSD_GObj {
        match unsafe { GROUND_801C498C } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C498C() as _ }; } }
    }
    pub static mut HSD_AOBJCLEARFLAGS: Option<fn(*mut HSD_AObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjClearFlags(aobj: *mut HSD_AObj, flags: u32) {
        match unsafe { HSD_AOBJCLEARFLAGS } { Some(f) => f(aobj, flags), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjClearFlags(aobj as _, flags as _); } return; } }
    }
    pub static mut HSD_AOBJGETFLAGS: Option<fn(*mut HSD_AObj) -> u32> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjGetFlags(aobj: *mut HSD_AObj) -> u32 {
        match unsafe { HSD_AOBJGETFLAGS } { Some(f) => f(aobj), None => { return unsafe { melee_runtime::tu::aobj::HSD_AObjGetFlags(aobj as _) as _ }; } }
    }
    pub static mut HSD_AOBJLOADDESC: Option<fn(*mut HSD_AObjDesc) -> *mut HSD_AObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjLoadDesc(aobjdesc: *mut HSD_AObjDesc) -> *mut HSD_AObj {
        match unsafe { HSD_AOBJLOADDESC } { Some(f) => f(aobjdesc), None => { return unsafe { melee_runtime::tu::aobj::HSD_AObjLoadDesc(aobjdesc as _) as _ }; } }
    }
    pub static mut HSD_AOBJREMOVE: Option<fn(*mut HSD_AObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjRemove(aobj: *mut HSD_AObj) {
        match unsafe { HSD_AOBJREMOVE } { Some(f) => f(aobj), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjRemove(aobj as _); } return; } }
    }
    pub static mut HSD_AOBJSETENDFRAME: Option<fn(*mut HSD_AObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjSetEndFrame(aobj: *mut HSD_AObj, frame: f32) {
        match unsafe { HSD_AOBJSETENDFRAME } { Some(f) => f(aobj, frame), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetEndFrame(aobj as _, frame as _); } return; } }
    }
    pub static mut HSD_AOBJSETFLAGS: Option<fn(*mut HSD_AObj, u32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32) {
        match unsafe { HSD_AOBJSETFLAGS } { Some(f) => f(aobj, flags), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetFlags(aobj as _, flags as _); } return; } }
    }
    pub static mut HSD_AOBJSETRATE: Option<fn(*mut HSD_AObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32) {
        match unsafe { HSD_AOBJSETRATE } { Some(f) => f(aobj, rate), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetRate(aobj as _, rate as _); } return; } }
    }
    pub static mut HSD_AOBJSETREWINDFRAME: Option<fn(*mut HSD_AObj, f32)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_AObjSetRewindFrame(aobj: *mut HSD_AObj, frame: f32) {
        match unsafe { HSD_AOBJSETREWINDFRAME } { Some(f) => f(aobj, frame), None => { unsafe { let _ = melee_runtime::tu::aobj::HSD_AObjSetRewindFrame(aobj as _, frame as _); } return; } }
    }
    /// `fn_801C82E8` may unwind out of this (the C `longjmp`s); `func` is an `extern "C-unwind" fn(*mut HSD_AObj, *mut c_void)`.
    hook!(HSD_FOREACHANIM, HSD_ForeachAnim, (obj: *mut c_void, r#type: u32, mask: u32, func: *mut c_void, arg_type: u32, va: &[usize]));
    hook!(HSD_JOBJADDANIMALL, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    hook!(HSD_JOBJREMOVEANIMALL, HSD_JObjRemoveAnimAll, (jobj: *mut HSD_JObj));
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32));
    hook!(HSD_JOBJREQANIMALLBYFLAGS, HSD_JObjReqAnimAllByFlags, (jobj: *mut HSD_JObj, flags: u32, frame: f32));
    hook!(HSD_JOBJREQANIMBYFLAGS, HSD_JObjReqAnimByFlags, (jobj: *mut HSD_JObj, flags: u32, frame: f32));
    hook!(HSD_JOBJRESETRST, HSD_JObjResetRST, (jobj: *mut HSD_JObj, joint: *mut HSD_Joint));
    hook!(OSREPORT, OSReport, (a0: *const i8, va: &[usize]));
    pub static mut GRDATFILES_801C6330: Option<fn(s32) -> *mut UnkArchiveStruct> = None;
    #[allow(non_snake_case)]
    pub fn grDatFiles_801C6330(a0: s32) -> *mut UnkArchiveStruct {
        match unsafe { GRDATFILES_801C6330 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C8B28: Option<fn(*mut HSD_JObj) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8B28(a0: *mut HSD_JObj) -> *mut HSD_JObj {
        match unsafe { GRMATERIAL_801C8B28 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grmaterial::grMaterial_801C8B28(a0 as _) as _ }; } }
    }
    pub static mut GRMATERIAL_801C8B68: Option<fn(*mut HSD_JObj, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C8B68(a0: *mut HSD_JObj, a1: i32) {
        match unsafe { GRMATERIAL_801C8B68 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8B68(a0 as _, a1 as _); } return; } }
    }
    pub static mut LBARCHIVE_80016EFC: Option<fn(*mut HSD_Archive)> = None;
    #[allow(non_snake_case)]
    pub fn lbArchive_80016EFC(a0: *mut HSD_Archive) {
        match unsafe { LBARCHIVE_80016EFC } { Some(f) => f(a0), None => { unsafe { let _ = melee_runtime::tu::lbarchive::lbArchive_80016EFC(a0 as _); } return; } }
    }
    pub static mut LBHEAP_80015CA8: Option<fn(i32, *mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn lbHeap_80015CA8(a0: i32, a1: *mut c_void) {
        match unsafe { LBHEAP_80015CA8 } { Some(f) => f(a0, a1), None => { unsafe { let _ = melee_runtime::tu::lbheap::lbHeap_80015CA8(a0 as _, a1 as _); } return; } }
    }
    pub static mut LB_8000B804: Option<fn(*mut HSD_JObj, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B804(a0: *mut HSD_JObj, a1: *mut HSD_Joint) {
        match unsafe { LB_8000B804 } { Some(f) => f(a0, a1), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B804(a0 as _, a1 as _); } return; } }
    }
    pub static mut MEMZERO: Option<fn(*mut c_void, ssize_t)> = None;
    #[allow(non_snake_case)]
    pub fn memzero(mem: *mut c_void, size: ssize_t) {
        match unsafe { MEMZERO } { Some(f) => f(mem, size), None => { unsafe { let _ = melee_runtime::tu::lb_00B0::memzero(mem as _, size as _); } return; } }
    }

    // The engine functions the C passes as callbacks (`void* func`), in the uniform AnimCallback form.
    pub unsafe extern "C" fn HSD_AObjClearFlags_cb(aobj: *mut HSD_AObj, _obj: *mut c_void, _flags: u32, arg: u32) {
        HSD_AObjClearFlags(aobj, arg)
    }
    pub unsafe extern "C" fn HSD_AObjSetFlags_cb(aobj: *mut HSD_AObj, _obj: *mut c_void, _flags: u32, arg: u32) {
        HSD_AObjSetFlags(aobj, arg)
    }
    pub unsafe extern "C" fn HSD_AObjSetRate_cb(aobj: *mut HSD_AObj, _obj: *mut c_void, _flags: u32, arg: u32) {
        HSD_AObjSetRate(aobj, f32::from_bits(arg))
    }
}

/// `static float grAnime_804D6958` / `grAnime_804D695C`.
pub static mut grAnime_804D6958: f32 = 0.0;
pub static mut grAnime_804D695C: f32 = 0.0;

/// The raised "longjmp" of `fn_801C82E8` (`grAnime_8049EE40`'s jmp_buf has no counterpart).
struct LongJmp;

const ARG_TYPE_DOBJ: u32 = 3;
const ARG_TYPE_MOBJ: u32 = 8;
const ARG_TYPE_POBJ: u32 = 9;
const CALL_ON_DOBJ: i32 = 1 << (ARG_TYPE_DOBJ - 1);
const CALL_ON_MOBJ: i32 = 1 << (ARG_TYPE_MOBJ - 1);
const CALL_ON_POBJ: i32 = 1 << (ARG_TYPE_POBJ - 1);

#[inline(always)]
unsafe fn pobj_type(o: *mut HSD_PObj) -> u32 {
    ((*o).flags & 0x3000) as u32
}
#[inline(always)]
fn union_type_dobj(j: *mut HSD_JObj) -> bool {
    unsafe { (*j).flags & (JOBJ_PTCL | JOBJ_SPLINE) as u32 == 0 }
}

pub unsafe fn grAnime_801C65B0(arg0: *mut UnkArchiveStruct) {
    if arg0.is_null() {
        return;
    }
    if (*arg0).unk0 as usize != usize::MAX {
        if (*arg0).unk8 == 0 {
            ext::lbArchive_80016EFC((*arg0).unk0);
        } else if (*arg0).unk8 == 1 {
            ext::lbHeap_80015CA8(0, (*arg0).unk0 as *mut c_void);
        }
    }
    ext::memzero(arg0 as *mut c_void, core::mem::size_of::<UnkArchiveStruct>() as ssize_t);
}

pub unsafe fn grAnime_801C6620(arg0: *mut HSD_PObj, arg1: *mut HSD_ShapeAnim) {
    if arg0.is_null() || arg1.is_null() {
        return;
    }
    let mut pobj = arg0;
    let mut shape_anim = arg1;
    while !pobj.is_null() {
        if !pobj.is_null() {
            assert_(pobj_type(pobj) == POBJ_SHAPEANIM as u32 && !(*pobj).u.shape_set.is_null(), 38, "pobj_type(pobj) == POBJ_SHAPEANIM && pobj->u.shape_set");
            if !shape_anim.is_null() {
                let shapeset = (*pobj).u.shape_set;
                if !(*shape_anim).aobjdesc.is_null() {
                    if !(*shapeset).aobj.is_null() {
                        ext::HSD_AObjRemove((*shapeset).aobj);
                    }
                    (*shapeset).aobj = ext::HSD_AObjLoadDesc((*shape_anim).aobjdesc);
                }
            }
        }
        pobj = (*pobj).next;
        shape_anim = if !shape_anim.is_null() { (*shape_anim).next } else { null_mut() };
    }
}

/// `static inline HSD_TexAnimFindById`.
pub unsafe fn HSD_TexAnimFindById(cur: *mut HSD_TexAnim, id: i32) -> *mut HSD_TexAnim {
    let mut cur = cur;
    while !cur.is_null() {
        if (*cur).id as i32 == id {
            return cur;
        }
        cur = (*cur).next;
    }
    null_mut()
}

pub unsafe fn grAnime_801C6710(tobj: *mut HSD_TObj, texanim: *mut HSD_TexAnim) {
    if tobj.is_null() {
        return;
    }
    let texanim = HSD_TexAnimFindById(texanim, (*tobj).id as i32);
    if !texanim.is_null() && !(*texanim).aobjdesc.is_null() {
        if !(*tobj).aobj.is_null() {
            ext::HSD_AObjRemove((*tobj).aobj);
        }
        (*tobj).aobj = ext::HSD_AObjLoadDesc((*texanim).aobjdesc);
        (*tobj).imagetbl = (*texanim).imagetbl;
    }
}

/// `static inline grAnime_801C6710_all`.
pub unsafe fn grAnime_801C6710_all(tobj: *mut HSD_TObj, texanim: *mut HSD_TexAnim) {
    let mut tobj = tobj;
    if !tobj.is_null() {
        while !tobj.is_null() {
            grAnime_801C6710(tobj, texanim);
            tobj = (*tobj).next;
        }
    }
}

pub unsafe fn grAnime_801C67A8(mobj: *mut HSD_MObj, matanim: *mut HSD_MatAnim) {
    if !mobj.is_null() && !matanim.is_null() {
        if !(*matanim).aobjdesc.is_null() {
            if !(*mobj).aobj.is_null() {
                ext::HSD_AObjRemove((*mobj).aobj);
            }
            (*mobj).aobj = ext::HSD_AObjLoadDesc((*matanim).aobjdesc);
        }
        grAnime_801C6710_all((*mobj).tobj, (*matanim).texanim);
    }
}

pub unsafe fn grAnime_801C67A8_inner(mobj: *mut HSD_MObj, matanim: *mut HSD_MatAnim) {
    grAnime_801C67A8(mobj, matanim);
}

pub unsafe fn grAnime_801C67A8_noinline(mobj: *mut HSD_MObj, matanim: *mut HSD_MatAnim) {
    grAnime_801C67A8_inner(mobj, matanim);
}

pub unsafe fn grAnime_801C683C(arg0: *mut HSD_DObj, arg1: *mut HSD_MatAnim, arg2: *mut HSD_ShapeAnimDObj) {
    if arg0.is_null() {
        return;
    }
    let mut dobj = arg0;
    let mut matanim = arg1;
    let mut sa_dobj = arg2;
    while !dobj.is_null() {
        if !dobj.is_null() {
            grAnime_801C6620((*dobj).pobj, if !sa_dobj.is_null() { (*sa_dobj).shapeanim } else { null_mut() });
            grAnime_801C67A8_noinline((*dobj).mobj, matanim);
        }
        dobj = (*dobj).next;
        matanim = if !matanim.is_null() { (*matanim).next } else { null_mut() };
        sa_dobj = if !sa_dobj.is_null() { (*sa_dobj).next } else { null_mut() };
    }
}

pub unsafe fn grAnime_801C68F4(robj: *mut HSD_RObj, robjanimjoint: *mut HSD_RObjAnimJoint) {
    if robj.is_null() || robjanimjoint.is_null() {
        return;
    }
    if !(*robjanimjoint).aobjdesc.is_null() {
        if !(*robj).aobj.is_null() {
            ext::HSD_AObjRemove((*robj).aobj);
        }
        (*robj).aobj = ext::HSD_AObjLoadDesc((*robjanimjoint).aobjdesc);
    }
}

pub unsafe fn grAnime_801C6960(robj: *mut HSD_RObj, arg1: *mut HSD_RObjAnimJoint) {
    if robj.is_null() || arg1.is_null() {
        return;
    }
    let mut phi_r31 = robj;
    let mut phi_r30 = arg1;
    while !phi_r31.is_null() && !phi_r30.is_null() {
        grAnime_801C68F4(phi_r31, phi_r30);
        phi_r31 = (*phi_r31).next;
        phi_r30 = (*phi_r30).next;
    }
}

pub unsafe fn grAnime_801C69FC(aobj: *mut HSD_AObj) -> *mut HSD_AObj {
    if aobj.is_null() || (*aobj).fobj.is_null() {
        return aobj;
    }

    let mut cur: *mut *mut HSD_FObj = &mut (*aobj).fobj;
    while !(*cur).is_null() {
        if (**cur).obj_type == 0xC {
            let tmp = *cur;
            *cur = (*tmp).next;
            (*tmp).next = (*aobj).fobj;
            (*aobj).fobj = tmp;
            break;
        }
        cur = &mut (**cur).next;
    }
    aobj
}

/// `static inline grAnime_801C69FC_for_jobj` (same effect, different register allocation in the C).
pub unsafe fn grAnime_801C69FC_for_jobj(aobj: *mut HSD_AObj) -> *mut HSD_AObj {
    if aobj.is_null() || (*aobj).fobj.is_null() {
        return aobj;
    }
    let mut cur: *mut *mut HSD_FObj = &mut (*aobj).fobj;
    while !(*cur).is_null() {
        let fobj = *cur;
        if (*fobj).obj_type == 0xC {
            *cur = (*fobj).next;
            (*fobj).next = (*aobj).fobj;
            (*aobj).fobj = fobj;
            break;
        }
        cur = &mut (*fobj).next;
    }
    aobj
}

pub unsafe fn grAnime_801C69FC_inner(aobj: *mut HSD_AObj) -> *mut HSD_AObj {
    grAnime_801C69FC(aobj)
}

pub unsafe fn grAnime_801C69FC_noinline(aobj: *mut HSD_AObj) -> *mut HSD_AObj {
    grAnime_801C69FC_inner(aobj)
}

pub unsafe fn grAnime_GetAObj(jobj: *mut HSD_JObj) -> *mut HSD_AObj {
    (*jobj).aobj
}

pub unsafe fn grAnime_801C6A54(jobj: *mut HSD_JObj, animjoint: *mut HSD_AnimJoint, matanimjoint: *mut HSD_MatAnimJoint, shapeanimjoint: *mut HSD_ShapeAnimJoint) {
    if jobj.is_null() {
        return;
    }
    if !animjoint.is_null() {
        if !(*animjoint).aobjdesc.is_null() {
            if !(*jobj).aobj.is_null() {
                ext::HSD_AObjRemove((*jobj).aobj);
            }
            (*jobj).aobj = ext::HSD_AObjLoadDesc((*animjoint).aobjdesc);
            grAnime_801C69FC_for_jobj(grAnime_GetAObj(jobj));
        }
        grAnime_801C6960((*jobj).robj, (*animjoint).robj_anim);
    }
    if union_type_dobj(jobj) {
        grAnime_801C683C(
            (*jobj).u.dobj,
            if !matanimjoint.is_null() { (*matanimjoint).matanim } else { null_mut() },
            if !shapeanimjoint.is_null() { (*shapeanimjoint).shapeanimdobj } else { null_mut() },
        );
    }
}

pub unsafe fn grAnime_801C6A54_inner(jobj: *mut HSD_JObj, animjoint: *mut HSD_AnimJoint, matanimjoint: *mut HSD_MatAnimJoint, shapeanimjoint: *mut HSD_ShapeAnimJoint) {
    if jobj.is_null() {
        return;
    }
    if !animjoint.is_null() {
        if !(*animjoint).aobjdesc.is_null() {
            if !(*jobj).aobj.is_null() {
                ext::HSD_AObjRemove((*jobj).aobj);
            }
            (*jobj).aobj = ext::HSD_AObjLoadDesc((*animjoint).aobjdesc);
            grAnime_801C69FC_noinline((*jobj).aobj);
        }
        grAnime_801C6960((*jobj).robj, (*animjoint).robj_anim);
    }
    if union_type_dobj(jobj) {
        grAnime_801C683C(
            (*jobj).u.dobj,
            if !matanimjoint.is_null() { (*matanimjoint).matanim } else { null_mut() },
            if !shapeanimjoint.is_null() { (*shapeanimjoint).shapeanimdobj } else { null_mut() },
        );
    }
}

pub unsafe fn grAnime_801C6A54_noinline(jobj: *mut HSD_JObj, animjoint: *mut HSD_AnimJoint, matanimjoint: *mut HSD_MatAnimJoint, shapeanimjoint: *mut HSD_ShapeAnimJoint) {
    grAnime_801C6A54(jobj, animjoint, matanimjoint, shapeanimjoint);
}

/// The next sibling of each of the three joint cursors (`x != NULL ? x->next : NULL`).
macro_rules! nx {
    ($p:expr) => {
        if !$p.is_null() { (*$p).next } else { null_mut() }
    };
}
macro_rules! ch {
    ($p:expr) => {
        if !$p.is_null() { (*$p).child } else { null_mut() }
    };
}

/// `static inline grAnime_801C6C0C_inner`: one recursion level of `grAnime_801C6C0C`.
pub unsafe fn grAnime_801C6C0C_inner(jobj: *mut HSD_JObj, animjoint: *mut HSD_AnimJoint, matanimjoint: *mut HSD_MatAnimJoint, shapeanimjoint: *mut HSD_ShapeAnimJoint) {
    if !jobj.is_null() {
        grAnime_801C6A54_noinline(jobj, animjoint, matanimjoint, shapeanimjoint);
        if (*jobj).flags & 0x1000 == 0 {
            let mut child = (*jobj).child;
            let mut caj = ch!(animjoint);
            let mut cmj = ch!(matanimjoint);
            let mut csj = ch!(shapeanimjoint);
            while !child.is_null() {
                grAnime_801C6C0C(child, caj, cmj, csj);
                child = (*child).next;
                caj = nx!(caj);
                cmj = nx!(cmj);
                csj = nx!(csj);
            }
        }
    }
}

pub unsafe fn grAnime_801C6C0C(jobj: *mut HSD_JObj, animjoint: *mut HSD_AnimJoint, matanimjoint: *mut HSD_MatAnimJoint, shapeanimjoint: *mut HSD_ShapeAnimJoint) {
    if !jobj.is_null() {
        grAnime_801C6A54_inner(jobj, animjoint, matanimjoint, shapeanimjoint);
        if (*jobj).flags & 0x1000 == 0 {
            let mut jp = (*jobj).child;
            let mut aj = ch!(animjoint);
            let mut mj = ch!(matanimjoint);
            let mut sj = ch!(shapeanimjoint);
            while !jp.is_null() {
                grAnime_801C6C0C_inner(jp, aj, mj, sj);
                jp = (*jp).next;
                aj = nx!(aj);
                mj = nx!(mj);
                sj = nx!(sj);
            }
        }
    }
}

pub unsafe extern "C" fn fn_801C6EE4(aobj: *mut HSD_AObj, _obj: *mut c_void, _flags: u32, _arg: u32) {
    ext::HSD_AObjSetFlags(aobj, AOBJ_LOOP as u32);
    ext::HSD_AObjSetRewindFrame(aobj, grAnime_804D6958);
    ext::HSD_AObjSetEndFrame(aobj, grAnime_804D695C);
}

pub unsafe extern "C" fn fn_801C6F2C(aobj: *mut HSD_AObj, _obj: *mut c_void, _flags: u32, _arg: u32) {
    ext::HSD_AObjSetFlags(aobj, AOBJ_LOOP as u32);
}

/// `grAnime_801C6F50`: call the callback with the argument shape `type` selects. `func` is an [`AnimCallback`].
pub unsafe fn grAnime_801C6F50(aobj: *mut HSD_AObj, obj: *mut c_void, flags: u32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    let f = core::mem::transmute::<*mut c_void, AnimCallback>(func);
    match r#type {
        0 | 4 | 8 => f(aobj, obj, flags, 0),
        1 | 5 | 9 => f(aobj, obj, flags, *(param as *const u32)),
        2 | 3 => f(aobj, obj, flags, *(param as *const u32)),
        6 | 7 => f(aobj, obj, flags, *(param as *const u32)),
        10 | 11 => f(aobj, obj, flags, *(param as *const u32)),
        _ => {}
    }
}

pub unsafe fn grAnime_801C706C(tobj: *mut HSD_TObj, flags: s32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    let mut tobj = tobj;
    while !tobj.is_null() {
        if flags & 0x400 != 0 && !(*tobj).aobj.is_null() {
            grAnime_801C6F50((*tobj).aobj, tobj as *mut c_void, 0xB, func, r#type, param);
        }
        tobj = (*tobj).next;
    }
}

/// `static inline grAnime_MObjForeachAnim`.
pub unsafe fn grAnime_MObjForeachAnim(mobj: *mut HSD_MObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    if mobj.is_null() {
        return;
    }
    if (flags & CALL_ON_MOBJ) != 0 && !(*mobj).aobj.is_null() {
        grAnime_801C6F50((*mobj).aobj, mobj as *mut c_void, ARG_TYPE_MOBJ, func, r#type, param);
    }
    grAnime_801C706C((*mobj).tobj, flags, func, r#type, param);
}

/// `static inline grAnime_PObjForeachAnim`.
pub unsafe fn grAnime_PObjForeachAnim(pobj: *mut HSD_PObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    if (flags & CALL_ON_POBJ) != 0
        && !pobj.is_null()
        && pobj_type(pobj) == POBJ_SHAPEANIM as u32
        && !(*pobj).u.shape_set.is_null()
        && !(*(*pobj).u.shape_set).aobj.is_null()
    {
        grAnime_801C6F50((*(*pobj).u.shape_set).aobj, pobj as *mut c_void, ARG_TYPE_POBJ, func, r#type, param);
    }
}

pub unsafe fn grAnime_801C70E0(dobj: *mut HSD_DObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    let mut dobj = dobj;
    while !dobj.is_null() {
        if (flags & CALL_ON_DOBJ) != 0 && !(*dobj).aobj.is_null() {
            grAnime_801C6F50((*dobj).aobj, dobj as *mut c_void, ARG_TYPE_DOBJ, func, r#type, param);
        }
        grAnime_MObjForeachAnim((*dobj).mobj, flags, func, r#type, param);
        grAnime_PObjForeachAnim((*dobj).pobj, flags, func, r#type, param);
        dobj = (*dobj).next;
    }
}

/// `static inline grAnime_801C6F50_wrapped`.
pub unsafe fn grAnime_801C6F50_wrapped(obj: *mut HSD_JObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void) -> bool {
    assert_(!obj.is_null(), 0x33A, "obj");
    if flags & 0x20 != 0 {
        if !(*obj).aobj.is_null() {
            grAnime_801C6F50((*obj).aobj, obj as *mut c_void, 6, func, r#type, param);
        }
    }
    (*obj).flags & 0x4020 == 0
}

/// `static inline grAnime_RObjForeachAnim`.
pub unsafe fn grAnime_RObjForeachAnim(robj: *mut HSD_RObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void) {
    let mut robj = robj;
    while !robj.is_null() {
        if (flags & 0x200) != 0 && !(*robj).aobj.is_null() {
            grAnime_801C6F50((*robj).aobj, robj as *mut c_void, 0xA, func, r#type, param);
        }
        robj = (*robj).next;
    }
}

pub unsafe fn grAnime_801C7228(obj: *mut HSD_JObj, flags: i32, func: *mut c_void, r#type: u32, param: *mut c_void, arg5: i32) {
    if grAnime_801C6F50_wrapped(obj, flags, func, r#type, param) {
        grAnime_801C70E0((*obj).u.dobj, flags, func, r#type, param);
    }
    grAnime_RObjForeachAnim((*obj).robj, flags, func, r#type, param);
    if arg5 != 0 && (*obj).flags & 0x1000 == 0 {
        let mut o = (*obj).child;
        while !o.is_null() {
            grAnime_801C7228(o, flags, func, r#type, param, arg5);
            o = (*o).next;
        }
    }
}

pub unsafe fn grAnime_801C70E0_inner(arg0: *mut HSD_DObj, arg1: i32, arg2: *mut c_void, arg3: u32, arg4: *mut c_void) {
    grAnime_801C70E0(arg0, arg1, arg2, arg3, arg4);
}

pub unsafe fn grAnime_801C70E0_noinline(arg0: *mut HSD_DObj, arg1: i32, arg2: *mut c_void, arg3: u32, arg4: *mut c_void) {
    grAnime_801C70E0_inner(arg0, arg1, arg2, arg3, arg4);
}

pub unsafe fn grAnime_801C7228_inner(arg0: *mut HSD_JObj, arg1: i32, arg2: *mut c_void, arg3: u32, arg4: *mut c_void, arg5: i32) {
    grAnime_801C7228(arg0, arg1, arg2, arg3, arg4, arg5);
}

pub unsafe fn grAnime_801C7228_noinline(arg0: *mut HSD_JObj, arg1: i32, arg2: *mut c_void, arg3: u32, arg4: *mut c_void, arg5: i32) {
    grAnime_801C7228_inner(arg0, arg1, arg2, arg3, arg4, arg5);
}

/// `grAnime_801C752C(obj, arg1, flags, func, type, ...)`: `va` is the single vararg's raw 64 bits (a `f64`'s bits for
/// the float types, a pointer or an integer otherwise). `func` is an [`AnimCallback`].
pub unsafe fn grAnime_801C752C(obj: *mut HSD_JObj, arg1: s32, flags: s32, func: *mut c_void, r#type: u32, va: &[u64]) {
    // `callbackArg`: 8 bytes natively; the float case stores the narrowed double in the low 4 bytes
    let mut arg: u64 = 0;

    if obj.is_null() {
        return;
    }
    let obj_tmp = obj;
    match r#type {
        0 | 4 | 8 => {}
        1 | 5 | 9 => arg = (f64::from_bits(va[0]) as f32).to_bits() as u64,
        2 | 6 | 10 => arg = va[0],
        3 | 7 | 11 => arg = va[0] as u32 as u64,
        _ => {
            ext::OSReport(b"unexpected argument format.\n\0".as_ptr() as *const i8, &[]);
            assert_(false, 0x36F, "0");
        }
    }
    let param = &mut arg as *mut u64 as *mut c_void;
    if grAnime_801C6F50_wrapped(obj_tmp, flags, func, r#type, param) {
        grAnime_801C70E0_noinline((*obj).u.dobj, flags, func, r#type, param);
    }
    let flags2 = flags & 0x200;
    let mut robj = (*obj).robj;
    while !robj.is_null() {
        if flags2 != 0 && !(*robj).aobj.is_null() {
            grAnime_801C6F50((*robj).aobj, robj as *mut c_void, 0xA, func, r#type, param);
        }
        robj = (*robj).next;
    }
    if arg1 != 0 && (*obj).flags & 0x1000 == 0 {
        let mut child = (*obj).child;
        while !child.is_null() {
            grAnime_801C7228_noinline(child, flags, func, r#type, param, arg1);
            child = (*child).next;
        }
    }
}

/// The bit sets `arg2` selects (shared by 775C..7A94): `0x220`, `0x7484`, `0x100`.
#[inline(always)]
fn anim_flags(arg2: u32) -> i32 {
    let mut v = 0;
    if arg2 & 1 != 0 {
        v |= 0x220;
    }
    if arg2 & 2 != 0 {
        v |= 0x7484;
    }
    if arg2 & 4 != 0 {
        v |= 0x100;
    }
    v
}

pub unsafe fn grAnime_801C775C(gobj: *mut HSD_GObj, arg1: i32, arg2: u32, arg8: f32, arg9: f32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    let var_r31 = anim_flags(arg2);
    grAnime_804D6958 = arg8;
    grAnime_804D695C = arg9;
    grAnime_801C752C(jobj, 1, var_r31, fn_801C6EE4 as *mut c_void, 0, &[]);
}

pub unsafe fn grAnime_801C77FC(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 1, anim_flags(arg2), fn_801C6F2C as *mut c_void, 0, &[]);
}

pub unsafe fn grAnime_801C787C(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 0, anim_flags(arg2), fn_801C6F2C as *mut c_void, 0, &[]);
}

pub unsafe fn grAnime_801C78FC(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 1, anim_flags(arg2), ext::HSD_AObjClearFlags_cb as *mut c_void, 3, &[AOBJ_LOOP as u64]);
}

pub unsafe fn grAnime_801C7980(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 0, anim_flags(arg2), ext::HSD_AObjClearFlags_cb as *mut c_void, 3, &[AOBJ_LOOP as u64]);
}

pub unsafe fn grAnime_801C7A04(gobj: *mut HSD_GObj, arg1: i32, arg2: u32, val: f32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 1, anim_flags(arg2), ext::HSD_AObjSetRate_cb as *mut c_void, 1, &[(val as f64).to_bits()]);
}

pub unsafe fn grAnime_801C7A94(gobj: *mut HSD_GObj, arg1: i32, arg2: u32, val: f32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    grAnime_801C752C(jobj, 0, anim_flags(arg2), ext::HSD_AObjSetRate_cb as *mut c_void, 1, &[(val as f64).to_bits()]);
}

/// The request-flag sets `arg2` selects for `ReqAnim*ByFlags` (7B24, 7BA0): `0x81`, `0x416`, `0x8`.
#[inline(always)]
fn req_flags(arg2: u32) -> u32 {
    let mut v = 0;
    if arg2 & 1 != 0 {
        v |= 0x81;
    }
    if arg2 & 2 != 0 {
        v |= 0x416;
    }
    if arg2 & 4 != 0 {
        v |= 8;
    }
    v
}

pub unsafe fn grAnime_801C7B24(gobj: *mut HSD_GObj, arg1: i32, arg2: u32, arg8: f32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    ext::HSD_JObjReqAnimAllByFlags(jobj, req_flags(arg2), arg8);
}

pub unsafe fn grAnime_801C7BA0(gobj: *mut HSD_GObj, arg1: i32, arg2: u32, arg8: f32) {
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return;
    }
    ext::HSD_JObjReqAnimByFlags(jobj, req_flags(arg2), arg8);
}

#[inline(always)]
unsafe fn x8_of(archive: *mut UnkArchiveStruct, map_id: s32) -> *mut UnkStageDat_x8_t {
    (*(*archive).unk4).unk8.offset(map_id as isize)
}

pub unsafe fn grAnime_801C7C1C(jobj: *mut HSD_JObj, map_id: s32, arg2: s32, arg3: s32, arg4: s32, arg5: i32, farg0: f32, farg1: f32) {
    let mut req_flags: u32 = 0;
    let mut anim_flags: s32 = 0;
    let aj: *mut HSD_AnimJoint;
    let mj: *mut HSD_MatAnimJoint;
    let sj: *mut HSD_ShapeAnimJoint;

    if jobj.is_null() {
        return;
    }
    let mut archive = ext::grDatFiles_801C6330(map_id);
    assert_(!archive.is_null(), 0x4DE, "archive");
    let ajp = (*x8_of(archive, map_id)).unk4;
    if (arg3 & 1) != 0 && !ajp.is_null() && !(*ajp.offset(arg4 as isize)).is_null() {
        aj = (*ajp.offset(arg4 as isize)).offset(arg2 as isize);
        req_flags |= 0x81;
        anim_flags |= 0x220;
    } else {
        aj = null_mut();
    }
    let mjp = (*x8_of(archive, map_id)).unk8;
    if (arg3 & 2) != 0 && !mjp.is_null() && !(*mjp.offset(arg4 as isize)).is_null() {
        mj = (*mjp.offset(arg4 as isize)).offset(arg2 as isize);
        req_flags |= 0x416;
        anim_flags |= 0x7484;
    } else {
        mj = null_mut();
    }
    let sjp = (*x8_of(archive, map_id)).unkC;
    if (arg3 & 4) != 0 && !sjp.is_null() && !(*sjp.offset(arg4 as isize)).is_null() {
        sj = (*sjp.offset(arg4 as isize)).offset(arg2 as isize);
        req_flags |= 8;
        anim_flags |= 0x100;
    } else {
        sj = null_mut();
    }
    if arg5 != 0 {
        grAnime_801C6C0C_inner(jobj, aj, mj, sj);
        ext::HSD_JObjReqAnimAllByFlags(jobj, req_flags, farg0);
    } else {
        grAnime_801C6A54_inner(jobj, aj, mj, sj);
        ext::HSD_JObjReqAnimByFlags(jobj, req_flags, farg0);
    }
    grAnime_801C752C(jobj, arg5, anim_flags, ext::HSD_AObjSetRate_cb as *mut c_void, 1, &[(farg1 as f64).to_bits()]);
    archive = ext::grDatFiles_801C6330(map_id);
    assert_(!archive.is_null(), 0x148, "archive");
    let eflags = (*x8_of(archive, map_id)).x28 as *mut u8;
    let flag: s32 = if !eflags.is_null() { *eflags.offset(arg4 as isize) as s32 } else { 0 };
    if flag != 0 {
        grAnime_801C752C(jobj, arg5, anim_flags, ext::HSD_AObjSetFlags_cb as *mut c_void, 3, &[0x20000000]);
    }
}

pub unsafe fn grAnime_801C7FF8(gobj: *mut Ground_GObj, arg1: i32, arg2: i32, arg3: i32, arg4: f32, arg5: f32) {
    let gr = GET_GROUND(gobj);
    if !gr.is_null() && arg3 >= 0 {
        let temp_r3 = ext::Ground_801C3FA4(gobj, arg1);
        if !temp_r3.is_null() {
            let jobj = ext::grMaterial_801C8B28(temp_r3);
            ext::grMaterial_801C8B68(jobj, 0);
            grAnime_801C7C1C(temp_r3, (*gr).map_id as s32, arg1, arg2, arg3, 1, arg4, arg5);
        }
    }
}

pub unsafe fn grAnime_801C8098(gobj: *mut Ground_GObj, arg1: i32, arg2: i32, arg3: i32, arg4: f32, arg5: f32) {
    let gr = GET_GROUND(gobj);
    if !gr.is_null() && arg3 >= 0 {
        let temp_r3 = ext::Ground_801C3FA4(gobj, arg1);
        if !temp_r3.is_null() {
            let jobj = ext::grMaterial_801C8B28(temp_r3);
            ext::grMaterial_801C8B68(jobj, 0);
            grAnime_801C7C1C(temp_r3, (*gr).map_id as s32, arg1, arg2, arg3, 0, arg4, arg5);
        }
    }
}

pub unsafe fn grAnime_801C8138(gobj: *mut HSD_GObj, arg1: enum_t, arg2: i32) {
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if arg2 < 0 {
        return;
    }
    let mut archive = ext::grDatFiles_801C6330(arg1);
    assert_(!archive.is_null(), 0x556, "archive");
    let joint = (*x8_of(archive, arg1)).unk0;
    if !HSD_JObjGetChild(jobj).is_null() {
        jobj = HSD_JObjGetChild(jobj);
    }
    if !joint.is_null() {
        ext::lb_8000B804(jobj, joint);
    }
    ext::grMaterial_801C8B68(jobj, 0);
    ext::HSD_JObjRemoveAnimAll(jobj);
    let sap = (*x8_of(archive, arg1)).unkC;
    let ajp = (*x8_of(archive, arg1)).unk4;
    let mjp = (*x8_of(archive, arg1)).unk8;
    let sa = if !sap.is_null() { *sap.offset(arg2 as isize) } else { null_mut() };
    let ma = if !mjp.is_null() { *mjp.offset(arg2 as isize) } else { null_mut() };
    let aj = if !ajp.is_null() { *ajp.offset(arg2 as isize) } else { null_mut() };
    ext::HSD_JObjAddAnimAll(jobj, aj, ma, sa);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    archive = ext::grDatFiles_801C6330(arg1);
    assert_(!archive.is_null(), 0x148, "archive");
    let flags = (*x8_of(archive, arg1)).x28 as *mut u8;
    let flag: s32 = if !flags.is_null() { *flags.offset(arg2 as isize) as s32 } else { 0 };
    if flag != 0 {
        grAnime_801C752C(jobj, 1, 0x77A4, ext::HSD_AObjSetFlags_cb as *mut c_void, 3, &[0x20000000]);
    }
    ext::HSD_JObjAnimAll(jobj);
}

/// `fn_801C82E8`: the `longjmp` back to `grAnime_801C8318` (an unwind here). `HSD_ForeachAnim` calls it as
/// `func(aobj, arg)` (the engine's own convention, not [`AnimCallback`]).
pub unsafe extern "C-unwind" fn fn_801C82E8(arg0: *mut HSD_AObj, arg1: *mut *mut HSD_AObj) {
    *arg1 = arg0;
    std::panic::resume_unwind(Box::new(LongJmp));
}

pub unsafe fn grAnime_801C8318(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) -> *mut HSD_AObj {
    let mut var_r30: u32 = 0;
    let mut sp14: *mut HSD_AObj = null_mut();
    let jobj = ext::Ground_801C3FA4(gobj, arg1);
    if jobj.is_null() {
        return null_mut();
    }
    if arg2 & 1 != 0 {
        var_r30 |= 0x220;
    }
    if arg2 & 2 != 0 {
        var_r30 |= 0x7484;
    }
    if arg2 & 4 != 0 {
        var_r30 |= 0x100;
    }
    // if (setjmp(grAnime_8049EE40) == 0) { HSD_ForeachAnim(...) }
    let sp14_ptr = &mut sp14 as *mut *mut HSD_AObj;
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ext::HSD_ForeachAnim(
            jobj as *mut c_void,
            JOBJ_TYPE as u32,
            var_r30,
            fn_801C82E8 as *mut c_void,
            AOBJ_ARG_AV as u32,
            &[sp14_ptr as usize],
        );
    }));
    if let Err(e) = r {
        if !e.is::<LongJmp>() {
            std::panic::resume_unwind(e);
        }
    }
    core::ptr::read_volatile(&sp14)
}

pub unsafe fn grAnime_801C83D0(gobj: *mut HSD_GObj, arg1: i32, arg2: u32) -> bool {
    let aobj = grAnime_801C8318(gobj, arg1, arg2);
    ext::HSD_AObjGetFlags(aobj) & 0x40000000 != 0
}

pub unsafe fn grAnime_801C84A4(gobj: *mut HSD_GObj, arg1: s32, arg2: s32) -> bool {
    let aobj = grAnime_801C8318(gobj, arg1, arg2 as u32);
    ext::HSD_AObjGetFlags(aobj) & 0x04000000 != 0
}

pub unsafe fn grAnime_801C8578(joint: *mut HSD_Joint, counter: *mut s32) -> *mut HSD_Joint {
    let mut joint = joint;
    if *counter == 0 {
        return joint;
    }
    if !(*joint).child.is_null() {
        *counter -= 1;
        joint = grAnime_801C8578((*joint).child, counter);
        if *counter == 0 {
            return joint;
        }
    }
    if !(*joint).next.is_null() {
        *counter -= 1;
        joint = grAnime_801C8578((*joint).next, counter);
        if *counter == 0 {
            return joint;
        }
    }
    joint
}

pub unsafe fn grAnime_801C8578_noinline(joint: *mut HSD_Joint, counter: *mut s32) -> *mut HSD_Joint {
    grAnime_801C8578(joint, counter)
}

pub unsafe fn grAnime_801C86D4(arg0: s32, arg1: *mut HSD_GObj, arg2: s32) {
    ext::Ground_801C3FA4(arg1, arg2);
    let archive = ext::grDatFiles_801C6330(arg0);
    assert_(!archive.is_null(), 1538, "archive");
    let root = (*x8_of(archive, arg0)).unk0;
    let mut sp: s32 = arg2;
    let joint = grAnime_801C8578_noinline(root, &mut sp);
    ext::HSD_JObjResetRST(ext::Ground_801C3FA4(arg1, arg2), joint);
}

pub unsafe fn grAnime_801C8780(_gobj: *mut HSD_GObj, arg1: u32, _arg2: u32, _arg3: f32, _arg4: f32) {
    ext::Ground_801C498C();
    let archive = ext::grDatFiles_801C6330(arg1 as s32);
    assert_(!archive.is_null(), 0x617, "archive");
}
