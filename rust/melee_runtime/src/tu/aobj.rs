//! Port of src/sysdolphin/baselib/aobj.c on the GameCube records of `melee_types` (raw pointers):
//! every function of the TU, including the `HSD_ForeachAnim` object-graph walkers that the
//! owning-style `crate::aobj` (HsdAObj<O>, verified against the C) leaves out. The frame logic
//! reuses `crate::aobj::{fmodf, AOBJ_*}`; callees of unported TUs (FObj, ObjAlloc, ID table, JObj
//! load/unref, HSD_Panic) are TODO stubs in `ext`, named like the C functions.
//!
//! Differences forced by Rust: `HSD_ForeachAnim` is C-variadic; here the one optional variadic
//! argument is passed as a [`CallbackArg`] (the C reads `f64` for the `F` forms and stores it in
//! `arg.f`, so the caller passes the already-narrowed `f32`). `HSD_ASSERT` panics (the C build
//! does not return from `__assert`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

pub use crate::aobj::{AOBJ_FIRST_PLAY, AOBJ_LOOP, AOBJ_NO_ANIM, AOBJ_NO_UPDATE, AOBJ_REWINDED};

pub mod ext {
    #![allow(non_snake_case, unused_variables, dead_code)]
    use super::*;
    macro_rules! todo_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            $( pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?)
            } )*
        };
    }
    todo_fns! {
        fn HSD_ObjAllocInit(a0: *mut HSD_ObjAllocData, a1: usize, a2: u32) = crate::tu::objalloc::HSD_ObjAllocInit(a0, a1, a2);
        fn HSD_ObjAlloc(a0: *mut HSD_ObjAllocData) -> *mut c_void = crate::tu::objalloc::HSD_ObjAlloc(a0);
        fn HSD_ObjFree(a0: *mut HSD_ObjAllocData, a1: *mut c_void) = crate::tu::objalloc::HSD_ObjFree(a0, a1);
        fn HSD_FObjRemoveAll(a0: *mut HSD_FObj) = crate::tu::fobj::HSD_FObjRemoveAll(a0);
        fn HSD_FObjReqAnimAll(a0: *mut HSD_FObj, a1: f32) = crate::tu::fobj::HSD_FObjReqAnimAll(a0, a1);
        fn HSD_FObjStopAnimAll(a0: *mut HSD_FObj, a1: *mut c_void, a2: HSD_ObjUpdateFunc, a3: f32) = crate::tu::fobj::HSD_FObjStopAnimAll(a0, a1, a2, a3);
        fn HSD_FObjInterpretAnimAll(a0: *mut HSD_FObj, a1: *mut c_void, a2: HSD_ObjUpdateFunc, a3: f32) = crate::tu::fobj::HSD_FObjInterpretAnimAll(a0 as _, a1, a2, a3);
        fn HSD_FObjLoadDesc(a0: *mut _HSD_FObjDesc) -> *mut HSD_FObj = crate::tu::fobj::HSD_FObjLoadDesc(a0 as _);
        fn HSD_IDGetDataFromTable(a0: u32, a1: HSD_IDKey, a2: *mut i32) -> *mut c_void = crate::tu::id::HSD_IDGetDataFromTable(a0 as _, a1, a2);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::jobj::HSD_JObjLoadJoint(a0);
        fn HSD_JObjUnref(a0: *mut HSD_JObj) = crate::tu::jobj::HSD_JObjUnref(a0);
        fn HSD_Panic(a0: *const i8, a1: u32, a2: *const i8) = crate::tu::debug::HSD_Panic(a0 as _, a1, a2 as _);
    }
}

/// `union callbackArg` (aobj.h).
#[derive(Clone, Copy)]
#[repr(C)]
pub union CallbackArg {
    pub f: f32,
    pub d: u32,
    pub v: *mut c_void,
}

pub static mut aobj_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

static mut endcallback_list: *mut HSD_SList = null_mut();

static mut HSD_AObj_804D762C: i32 = 0;
static mut HSD_AObj_804D7630: i32 = 0;

#[inline]
fn hsd_assert(cond: bool, line: u32) {
    if !cond {
        panic!("HSD_ASSERT aobj.c:{line}");
    }
}

pub unsafe extern "C" fn HSD_AObjInitAllocData() {
    ext::HSD_ObjAllocInit(&raw mut aobj_alloc_data, core::mem::size_of::<HSD_AObj>(), 4);
}

pub unsafe extern "C" fn HSD_AObjGetAllocData() -> *mut HSD_ObjAllocData {
    &raw mut aobj_alloc_data
}

pub unsafe extern "C" fn HSD_AObjGetFlags(aobj: *mut HSD_AObj) -> u32 {
    if !aobj.is_null() {
        (*aobj).flags
    } else {
        0
    }
}

pub unsafe extern "C" fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32) {
    // TARGET_PC: an impossibly small pointer is treated as absent.
    if (aobj as usize) < 0x1000 {
        return;
    }
    if !aobj.is_null() {
        let flags = flags & (AOBJ_LOOP | AOBJ_NO_UPDATE);
        (*aobj).flags |= flags;
    }
}

pub unsafe extern "C" fn HSD_AObjClearFlags(aobj: *mut HSD_AObj, flags: u32) {
    if !aobj.is_null() {
        let flags = flags & (AOBJ_LOOP | AOBJ_NO_UPDATE);
        (*aobj).flags &= !flags;
    }
}

pub unsafe extern "C" fn HSD_AObjSetFObj(aobj: *mut HSD_AObj, fobj: *mut HSD_FObj) {
    if aobj.is_null() {
        return;
    }
    if !(*aobj).fobj.is_null() {
        ext::HSD_FObjRemoveAll((*aobj).fobj);
    }
    (*aobj).fobj = fobj;
}

pub unsafe extern "C" fn HSD_AObjInitEndCallBack() {
    HSD_AObj_804D762C = 0;
    HSD_AObj_804D7630 = 0;
}

pub unsafe extern "C" fn HSD_AObjInvokeCallBacks() {
    if HSD_AObj_804D762C != 0 && HSD_AObj_804D7630 == 0 {
        let mut list = endcallback_list;
        while !list.is_null() {
            let func: unsafe extern "C" fn() = core::mem::transmute((*list).data);
            func();
            list = (*list).next;
        }
    }
}

pub unsafe extern "C" fn HSD_AObjReqAnim(aobj: *mut HSD_AObj, frame: f32) {
    if aobj.is_null() {
        return;
    }
    (*aobj).curr_frame = frame;
    let flags = (*aobj).flags & !AOBJ_NO_ANIM;
    (*aobj).flags = flags | AOBJ_FIRST_PLAY;
    ext::HSD_FObjReqAnimAll((*aobj).fobj, frame);
}

pub unsafe extern "C" fn HSD_AObjStopAnim(aobj: *mut HSD_AObj, obj: *mut c_void, func: HSD_ObjUpdateFunc) {
    if aobj.is_null() {
        return;
    }
    ext::HSD_FObjStopAnimAll((*aobj).fobj, obj, func, (*aobj).framerate);
    (*aobj).flags |= AOBJ_NO_ANIM;
}

pub unsafe extern "C" fn HSD_AObjInterpretAnim(aobj: *mut HSD_AObj, obj: *mut c_void, update_func: HSD_ObjUpdateFunc) {
    let mut rate: f32;

    if aobj.is_null() || (*aobj).flags & AOBJ_NO_ANIM != 0 {
        return;
    }

    if (*aobj).flags & AOBJ_FIRST_PLAY != 0 {
        (*aobj).flags &= 0xF7FFFFFF;
        rate = 0.0;
    } else {
        rate = (*aobj).framerate;
        (*aobj).curr_frame += (*aobj).framerate;
    }

    if (*aobj).flags & AOBJ_LOOP != 0 && (*aobj).end_frame <= (*aobj).curr_frame {
        if (*aobj).rewind_frame < (*aobj).end_frame {
            ext::HSD_FObjStopAnimAll((*aobj).fobj, obj, update_func, rate);
            let y = (*aobj).end_frame - (*aobj).rewind_frame;
            let x = (*aobj).curr_frame - (*aobj).rewind_frame;
            (*aobj).curr_frame = crate::aobj::fmodf(x, y) + (*aobj).rewind_frame;
            ext::HSD_FObjReqAnimAll((*aobj).fobj, (*aobj).curr_frame);
        } else {
            (*aobj).curr_frame = (*aobj).end_frame;
        }
        rate = 0.0;
        (*aobj).flags |= AOBJ_REWINDED;
    } else {
        (*aobj).flags &= 0xFBFFFFFF;
    }

    if (*aobj).flags & AOBJ_NO_UPDATE != 0 {
        ext::HSD_FObjInterpretAnimAll((*aobj).fobj, obj, None, rate);
    } else {
        ext::HSD_FObjInterpretAnimAll((*aobj).fobj, obj, update_func, rate);
    }

    if (*aobj).flags & AOBJ_LOOP == 0 && (*aobj).end_frame <= (*aobj).curr_frame {
        ext::HSD_FObjStopAnimAll((*aobj).fobj, obj, update_func, (*aobj).framerate);
        (*aobj).flags |= AOBJ_NO_ANIM;
    }

    if (*aobj).flags & AOBJ_NO_ANIM != 0 {
        HSD_AObj_804D762C += 1;
    } else {
        HSD_AObj_804D7630 += 1;
    }
}

pub unsafe extern "C" fn HSD_AObjLoadDesc(aobjdesc: *mut HSD_AObjDesc) -> *mut HSD_AObj {
    if aobjdesc.is_null() {
        return null_mut();
    }
    let aobj = HSD_AObjAlloc();
    HSD_AObjSetFlags(aobj, (*aobjdesc).flags);
    HSD_AObjSetRewindFrame(aobj, 0.0);
    HSD_AObjSetEndFrame(aobj, (*aobjdesc).end_frame);
    let fobj = ext::HSD_FObjLoadDesc((*aobjdesc).fobjdesc);
    HSD_AObjSetFObj(aobj, fobj);
    let id = (*aobjdesc).obj_id;
    if id as usize != 0 {
        let hsd_obj = ext::HSD_IDGetDataFromTable(0, id, null_mut()) as *mut HSD_Obj;
        let mut phi_r30 = hsd_obj;
        if !hsd_obj.is_null() {
            // ref_INC
            (*hsd_obj).ref_count = (*hsd_obj).ref_count.wrapping_add(1);
            hsd_assert((*hsd_obj).ref_count != u16::MAX, 93);
        } else {
            phi_r30 = ext::HSD_JObjLoadJoint((*aobjdesc).obj_id as usize as *mut HSD_Joint) as *mut HSD_Obj;
        }
        if !aobj.is_null() {
            if !(*aobj).hsd_obj.is_null() {
                ext::HSD_JObjUnref((*aobj).hsd_obj as *mut HSD_JObj);
            }
            (*aobj).hsd_obj = phi_r30;
        }
    }
    aobj
}

pub unsafe extern "C" fn HSD_AObjRemove(aobj: *mut HSD_AObj) {
    if aobj.is_null() {
        return;
    }
    if !(*aobj).fobj.is_null() {
        ext::HSD_FObjRemoveAll((*aobj).fobj);
    }
    (*aobj).fobj = null_mut();

    if !(*aobj).hsd_obj.is_null() {
        ext::HSD_JObjUnref((*aobj).hsd_obj as *mut HSD_JObj);
    }
    (*aobj).hsd_obj = null_mut();
    HSD_AObjFree(aobj);
}

pub unsafe extern "C" fn HSD_AObjAlloc() -> *mut HSD_AObj {
    let new = ext::HSD_ObjAlloc(HSD_AObjGetAllocData()) as *mut HSD_AObj;
    hsd_assert(!new.is_null(), 489);

    core::ptr::write_bytes(new as *mut u8, 0, core::mem::size_of::<HSD_AObj>());
    (*new).flags = AOBJ_NO_ANIM;
    (*new).framerate = 1.0;
    new
}

pub unsafe extern "C" fn HSD_AObjFree(aobj: *mut HSD_AObj) {
    if aobj.is_null() {
        return;
    }
    ext::HSD_ObjFree(HSD_AObjGetAllocData(), aobj as *mut c_void);
}

unsafe fn callbackForeachFunc(aobj: *mut HSD_AObj, obj: *mut c_void, type_: HSD_Type, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    type Fa = unsafe extern "C" fn(*mut HSD_AObj);
    type Faf = unsafe extern "C" fn(*mut HSD_AObj, f32);
    type Fav = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void);
    type Fau = unsafe extern "C" fn(*mut HSD_AObj, u32);
    type Fao = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void);
    type Faot = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, HSD_Type);
    type Faof = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, f32);
    type Faov = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, *mut c_void);
    type Faou = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, u32);
    type Faotf = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, HSD_Type, f32);
    type Faotv = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, HSD_Type, *mut c_void);
    type Faotu = unsafe extern "C" fn(*mut HSD_AObj, *mut c_void, HSD_Type, u32);
    match arg_type {
        AOBJ_ARG_A => core::mem::transmute::<*mut c_void, Fa>(func)(aobj),
        AOBJ_ARG_AF => core::mem::transmute::<*mut c_void, Faf>(func)(aobj, (*arg).f),
        AOBJ_ARG_AV => core::mem::transmute::<*mut c_void, Fav>(func)(aobj, (*arg).v),
        AOBJ_ARG_AU => core::mem::transmute::<*mut c_void, Fau>(func)(aobj, (*arg).d),
        AOBJ_ARG_AO => core::mem::transmute::<*mut c_void, Fao>(func)(aobj, obj),
        AOBJ_ARG_AOT => core::mem::transmute::<*mut c_void, Faot>(func)(aobj, obj, type_),
        AOBJ_ARG_AOF => core::mem::transmute::<*mut c_void, Faof>(func)(aobj, obj, (*arg).f),
        AOBJ_ARG_AOV => core::mem::transmute::<*mut c_void, Faov>(func)(aobj, obj, (*arg).v),
        AOBJ_ARG_AOU => core::mem::transmute::<*mut c_void, Faou>(func)(aobj, obj, (*arg).d),
        AOBJ_ARG_AOTF => core::mem::transmute::<*mut c_void, Faotf>(func)(aobj, obj, type_, (*arg).f),
        AOBJ_ARG_AOTV => core::mem::transmute::<*mut c_void, Faotv>(func)(aobj, obj, type_, (*arg).v),
        AOBJ_ARG_AOTU => core::mem::transmute::<*mut c_void, Faotu>(func)(aobj, obj, type_, (*arg).d),
        _ => {}
    }
}

unsafe fn FogForeachAnim(fog: *mut HSD_Fog, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    if mask & FOG_MASK != 0 && !fog.is_null() && !(*fog).aobj.is_null() {
        callbackForeachFunc((*fog).aobj, fog as *mut c_void, FOG_TYPE, func, arg_type, arg);
    }
}

unsafe fn TObjForeachAnim(mut tobj: *mut HSD_TObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    while !tobj.is_null() {
        if mask & TOBJ_MASK != 0 && !(*tobj).aobj.is_null() {
            callbackForeachFunc((*tobj).aobj, tobj as *mut c_void, TOBJ_TYPE, func, arg_type, arg);
        }
        tobj = (*tobj).next;
    }
}

unsafe fn RObjForeachAnim(mut robj: *mut HSD_RObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    while !robj.is_null() {
        if mask & ROBJ_MASK != 0 && !(*robj).aobj.is_null() {
            callbackForeachFunc((*robj).aobj, robj as *mut c_void, ROBJ_TYPE, func, arg_type, arg);
        }
        robj = (*robj).next;
    }
}

unsafe fn WObjForeachAnim(wobj: *mut HSD_WObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    if wobj.is_null() {
        return;
    }
    if mask & WOBJ_MASK != 0 && !(*wobj).aobj.is_null() {
        callbackForeachFunc((*wobj).aobj, wobj as *mut c_void, WOBJ_TYPE, func, arg_type, arg);
    }
    RObjForeachAnim((*wobj).robj, mask, func, arg_type, arg);
}

unsafe fn CObjForeachAnim(cobj: *mut HSD_CObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    if cobj.is_null() {
        return;
    }
    if mask & COBJ_MASK != 0 && !(*cobj).aobj.is_null() {
        callbackForeachFunc((*cobj).aobj, cobj as *mut c_void, COBJ_TYPE, func, arg_type, arg);
    }
    WObjForeachAnim((*cobj).eyepos, mask, func, arg_type, arg);
    WObjForeachAnim((*cobj).interest, mask, func, arg_type, arg);
}

unsafe fn LObjForeachAnim(mut lobj: *mut HSD_LObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    while !lobj.is_null() {
        if mask & LOBJ_MASK != 0 && !(*lobj).aobj.is_null() {
            callbackForeachFunc((*lobj).aobj, lobj as *mut c_void, LOBJ_TYPE, func, arg_type, arg);
        }
        WObjForeachAnim((*lobj).position, mask, func, arg_type, arg);
        WObjForeachAnim((*lobj).interest, mask, func, arg_type, arg);
        lobj = (*lobj).next;
    }
}

unsafe fn PObjForeachAnim(pobj: *mut HSD_PObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    if mask & POBJ_MASK != 0
        && !pobj.is_null()
        && ((*pobj).flags & 0x3000) as i32 == POBJ_SHAPEANIM
        && !(*pobj).u.shape_set.is_null()
        && !(*(*pobj).u.shape_set).aobj.is_null()
    {
        callbackForeachFunc((*(*pobj).u.shape_set).aobj, pobj as *mut c_void, POBJ_TYPE, func, arg_type, arg);
    }
}

unsafe fn MObjForeachAnim(mobj: *mut HSD_MObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    if mobj.is_null() {
        return;
    }
    if mask & MOBJ_MASK != 0 && !(*mobj).aobj.is_null() {
        callbackForeachFunc((*mobj).aobj, mobj as *mut c_void, MOBJ_TYPE, func, arg_type, arg);
    }
    TObjForeachAnim((*mobj).tobj, mask, func, arg_type, arg);
}

unsafe fn DObjForeachAnim(mut dobj: *mut HSD_DObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    while !dobj.is_null() {
        if mask & DOBJ_MASK != 0 && !(*dobj).aobj.is_null() {
            callbackForeachFunc((*dobj).aobj, dobj as *mut c_void, DOBJ_TYPE, func, arg_type, arg);
        }
        MObjForeachAnim((*dobj).mobj, mask, func, arg_type, arg);
        PObjForeachAnim((*dobj).pobj, mask, func, arg_type, arg);
        dobj = (*dobj).next;
    }
}

unsafe fn JObjForeachAnim(mut obj: *mut HSD_JObj, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, arg: *mut CallbackArg) {
    hsd_assert(!obj.is_null(), 715);

    if mask & JOBJ_MASK != 0 && !(*obj).aobj.is_null() {
        callbackForeachFunc((*obj).aobj, obj as *mut c_void, JOBJ_TYPE, func, arg_type, arg);
    }
    // union_type_dobj: neither JOBJ_PTCL nor JOBJ_SPLINE
    if (*obj).flags & ((JOBJ_PTCL | JOBJ_SPLINE) as u32) == 0 {
        DObjForeachAnim((*obj).u.dobj, mask, func, arg_type, arg);
    }
    RObjForeachAnim((*obj).robj, mask, func, arg_type, arg);
    if (*obj).flags & (JOBJ_INSTANCE as u32) == 0 {
        obj = (*obj).child;
        while !obj.is_null() {
            JObjForeachAnim(obj, mask, func, arg_type, arg);
            obj = (*obj).next;
        }
    }
}

/// `HSD_ForeachAnim(obj, type, mask, func, arg_type, ...)`: `arg` is the optional variadic argument.
pub unsafe extern "C" fn HSD_ForeachAnim(obj: *mut c_void, type_: HSD_Type, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type, mut arg: CallbackArg) {
    if obj.is_null() {
        return;
    }

    match arg_type {
        AOBJ_ARG_A | AOBJ_ARG_AO | AOBJ_ARG_AOT => {
            // the C leaves `arg` uninitialised for these forms
        }
        AOBJ_ARG_AF | AOBJ_ARG_AOF | AOBJ_ARG_AOTF => {}
        AOBJ_ARG_AV | AOBJ_ARG_AOV | AOBJ_ARG_AOTV => {}
        AOBJ_ARG_AU | AOBJ_ARG_AOU | AOBJ_ARG_AOTU => {}
        _ => {
            ext::HSD_Panic(c"aobj.c".as_ptr(), 826, c"unexpected argument format.\n".as_ptr());
        }
    }

    let argp = &mut arg as *mut CallbackArg;
    match type_ {
        JOBJ_TYPE => JObjForeachAnim(obj as *mut HSD_JObj, mask, func, arg_type, argp),
        DOBJ_TYPE => DObjForeachAnim(obj as *mut HSD_DObj, mask, func, arg_type, argp),
        MOBJ_TYPE => MObjForeachAnim(obj as *mut HSD_MObj, mask, func, arg_type, argp),
        POBJ_TYPE => PObjForeachAnim(obj as *mut HSD_PObj, mask, func, arg_type, argp),
        TOBJ_TYPE => TObjForeachAnim(obj as *mut HSD_TObj, mask, func, arg_type, argp),
        LOBJ_TYPE => LObjForeachAnim(obj as *mut HSD_LObj, mask, func, arg_type, argp),
        COBJ_TYPE => CObjForeachAnim(obj as *mut HSD_CObj, mask, func, arg_type, argp),
        ROBJ_TYPE => RObjForeachAnim(obj as *mut HSD_RObj, mask, func, arg_type, argp),
        WOBJ_TYPE => WObjForeachAnim(obj as *mut HSD_WObj, mask, func, arg_type, argp),
        FOG_TYPE => FogForeachAnim(obj as *mut HSD_Fog, mask, func, arg_type, argp),
        _ => {
            ext::HSD_Panic(c"aobj.c".as_ptr(), 862, c"unexpected type of object.\n".as_ptr());
        }
    }
}

pub unsafe extern "C" fn HSD_AObjSetRate(aobj: *mut HSD_AObj, rate: f32) {
    if aobj.is_null() {
        return;
    }
    (*aobj).framerate = rate;
}

pub unsafe extern "C" fn HSD_AObjSetRewindFrame(aobj: *mut HSD_AObj, frame: f32) {
    if (aobj as usize) < 0x1000 {
        return;
    }
    if aobj.is_null() {
        return;
    }
    (*aobj).rewind_frame = frame;
}

pub unsafe extern "C" fn HSD_AObjSetEndFrame(aobj: *mut HSD_AObj, frame: f32) {
    if (aobj as usize) < 0x1000 {
        return;
    }
    if aobj.is_null() {
        return;
    }
    (*aobj).end_frame = frame;
}

pub unsafe extern "C" fn HSD_AObjSetCurrentFrame(aobj: *mut HSD_AObj, frame: f32) {
    if aobj.is_null() {
        return;
    }
    if (*aobj).flags & AOBJ_NO_ANIM == 0 {
        (*aobj).curr_frame = frame;
        (*aobj).flags = ((*aobj).flags & 0xBFFFFFFF) | AOBJ_FIRST_PLAY;
        ext::HSD_FObjReqAnimAll((*aobj).fobj, frame);
    }
}

pub unsafe extern "C" fn _HSD_AObjForgetMemory(low: *mut c_void, high: *mut c_void) {
    endcallback_list = null_mut();
}

/// `HSD_AObjGetEndFrame` (aobj.h inline): the animation's last frame.
pub unsafe fn HSD_AObjGetEndFrame(aobj: *mut HSD_AObj) -> f32 {
    assert!(!aobj.is_null(), "aobj.h:0xAA aobj");
    (*aobj).end_frame
}
