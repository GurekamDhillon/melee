//! src/melee/gr/grlib.c: stage helpers (generator wrappers, the 4 stage-quake cameras gobjs, the
//! "pointer inside a fighter/item ECB" test). Native-signature port of every function; the older
//! `crate::gr::grlib` module is the hook-object formulation of the same code.
//! State-oracle checked against the C (tests/gro_grlib.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::stage::StageWorld;
#[allow(unused_imports)]
use crate::vec::Vec3 as Vec3_grlib;
#[allow(unused_imports)]
use crate::gr::grlib::{GrLibHooks, kill_generators_on, point_inside_coll};
use super::grinl::{assert_, ABS, GET_JOBJ, HSD_JObjGetChild, HSD_JObjGetNext, HSD_JObjGetTranslationX, HSD_JObjGetTranslationY};
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_core::fp::fmadds;
use melee_types::*;

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::stage_info;
    hook_or!(CAMERA_SETQUAKEOFFSET, Camera_SetQuakeOffset, (x: f32, y: f32), |x: f32, y: f32| unsafe { crate::tu::camera::Camera_SetQuakeOffset(x, y) });
    hook_or!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj, |classifier: u16, p_link: u8, priority: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(classifier as _, p_link as _, priority as _) as _ });
    pub static mut GROUND_801C0498: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0498() -> f32 {
        match unsafe { GROUND_801C0498 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C0498() as _ }; } }
    }
    pub static mut GROUND_801C2D24: Option<fn(enum_t, *mut Vec3) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2D24(a0: enum_t, a1: *mut Vec3) -> i32 {
        match unsafe { GROUND_801C2D24 } { Some(f) => f(a0, a1), None => { return unsafe { crate::tu::ground::Ground_801C2D24(a0 as _, a1 as _) as _ }; } }
    }
    pub static mut HSD_AOBJSETFLAGS: Option<fn(*mut HSD_AObj, u32)> = None;
    #[allow(non_snake_case)]
    #[inline(never)] // its address is handed to HSD_ForeachAnim and compared: keep one copy
    pub fn HSD_AObjSetFlags(aobj: *mut HSD_AObj, flags: u32) {
        match unsafe { HSD_AOBJSETFLAGS } { Some(f) => f(aobj, flags), None => { unsafe { melee_runtime::tu::aobj::HSD_AObjSetFlags(aobj as _, flags as _); } return; } }
    }
    hook!(HSD_FOREACHANIM, HSD_ForeachAnim, (obj: *mut c_void, ty: HSD_Type, mask: HSD_TypeMask, func: *mut c_void, arg_type: AObj_Arg_Type));
    hook_or!(HSD_GOBJFREE, HSD_GObjFree, (a0: *mut HSD_GObj), |a0: *mut HSD_GObj| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } });
    hook_or!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void), |gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void| { unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(gobj as _, kind as _, obj as _); } });
    hook_or!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc, |a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ });
    hook!(HSD_JOBJADDANIMALL, HSD_JObjAddAnimAll, (a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint));
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (a0: *mut HSD_JObj));
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj {
        match unsafe { HSD_JOBJLOADJOINT } { Some(f) => f(a0), None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(a0 as _) as _ }; } }
    }
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (a0: *mut HSD_JObj, a1: f32));
    pub static mut GRBIGBLUE_801EF7D8: Option<fn(*mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn grBigBlue_801EF7D8(a0: *mut Vec3) {
        match unsafe { GRBIGBLUE_801EF7D8 } { Some(f) => f(a0), None => { unsafe { crate::tu::grbigblue::grBigBlue_801EF7D8(a0 as _); } return; } }
    }
    pub static mut GRICEMT_801FA728: Option<fn(*mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn grIceMt_801FA728(a0: *mut Vec3) {
        match unsafe { GRICEMT_801FA728 } { Some(f) => f(a0), None => { unsafe { crate::tu::gricemt::grIceMt_801FA728(a0 as _); } return; } }
    }
    pub static mut GRRCRUISE_80201918: Option<fn(*mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn grRCruise_80201918(a0: *mut Vec3) {
        match unsafe { GRRCRUISE_80201918 } { Some(f) => f(a0), None => { unsafe { crate::tu::grrcruise::grRCruise_80201918(a0 as _); } return; } }
    }
    hook!(HSD_8039D4DC, hsd_8039D4DC, (a0: *mut HSD_Generator));
    hook!(HSD_8039D5DC, hsd_8039D5DC, (a0: *mut HSD_JObj));
    hook!(HSD_8039EFAC, hsd_8039EFAC, (linkNo: s32, bank: s32, gfx_id: s32, a3: *mut HSD_JObj) -> *mut HSD_Generator);
    hook!(HSD_8039F05C, hsd_8039F05C, (linkNo: s32, bank: s32, idx: s32) -> *mut HSD_Generator);
    hook!(HSD_8039F6CC, hsd_8039F6CC, (linkNo: s32, bank: s32, gfx_id: s32, a3: *mut HSD_JObj) -> *mut HSD_Generator);
    hook_or!(ITGETKIND, itGetKind, (gobj: *mut HSD_GObj) -> ItemKind, |gobj: *mut HSD_GObj| unsafe { crate::itacc::itGetKind(gobj as _) as _ });
    pub static mut LB_8000FD18: Option<fn(*mut DynamicsDesc)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000FD18(a0: *mut DynamicsDesc) {
        match unsafe { LB_8000FD18 } { Some(f) => f(a0), None => { unsafe { melee_runtime::tu::lb_00F9::lb_8000FD18(a0 as _); } return; } }
    }
    pub static mut LB_8000FD48: Option<fn(*mut HSD_JObj, *mut DynamicsDesc, u32)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000FD48(a0: *mut HSD_JObj, a1: *mut DynamicsDesc, a2: u32) {
        match unsafe { LB_8000FD48 } { Some(f) => f(a0, a1, a2), None => { unsafe { melee_runtime::tu::lb_00F9::lb_8000FD48(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut LB_8001044C: Option<fn(*mut DynamicsDesc, *mut c_void, i32, f32, i32, Fighter_Part, i32, i32)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8001044C(a0: *mut DynamicsDesc, a1: *mut c_void, a2: i32, pos_y: f32, a4: i32, part: Fighter_Part, a6: i32, a7: i32) {
        match unsafe { LB_8001044C } {
            Some(f) => f(a0, a1, a2, pos_y, a4, part, a6, a7),
            None => { unsafe { melee_runtime::tu::lb_00F9::lb_8001044C(a0 as _, a1 as _, a2 as _, pos_y as _, (a4 != 0), part, a6 as _, (a7 != 0)); } },
        }
    }
    pub static mut LB_80011710: Option<fn(*mut DynamicsDesc, *mut DynamicsDesc)> = None;
    #[allow(non_snake_case)]
    pub fn lb_80011710(a0: *mut DynamicsDesc, a1: *mut DynamicsDesc) {
        match unsafe { LB_80011710 } { Some(f) => f(a0, a1), None => { unsafe { melee_runtime::tu::lb_00F9::lb_80011710(a0 as _, a1 as _); } return; } }
    }
    hook!(PSADDGENERATORAPPSRT_BEGIN, psAddGeneratorAppSRT_begin, (a0: *mut HSD_Generator, a1: s32) -> *mut HSD_psAppSRT);
    // Externs that are data, not code: `HSD_GObj** HSD_GObjPLinkHead` (the C global is a pointer to
    // the array of per-p_link list heads) and `HSD_Generator* hsd_804D78FC` (generator list head).
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use melee_runtime::tu::generator::hsd_804D78FC;
    pub use melee_runtime::tu::gobj_native::HSD_GObj_JObjKind;
}

/// `static VecMtx grLib_8049EF58` (49EF58).
pub static mut grLib_8049EF58: [Vec3; 4] = [Vec3 { x: 0.0, y: 0.0, z: 0.0 }; 4];

pub unsafe extern "C" fn grLib_801C96E8(arg0: *mut HSD_GObj) -> bool {
    melee_core::ct!("grLib_801C96E8");
    let gp = (*arg0).user_data as *mut Ground;
    (*gp).x10_flags.b4() != 0
}

pub unsafe fn grLib_801C96F8(arg0: s32, arg1: s32, arg2: *mut Vec3) -> *mut HSD_Generator {
    melee_core::ct!("grLib_801C96F8");
    let temp_r3 = ext::hsd_8039F05C(0, arg1, arg0);
    if !temp_r3.is_null() {
        let mut phi_r30 = (*temp_r3).appsrt;
        if phi_r30.is_null() {
            phi_r30 = ext::psAddGeneratorAppSRT_begin(temp_r3, 1);
        } else {
            (*phi_r30).xA2 = 0;
        }
        if phi_r30.is_null() {
            ext::hsd_8039D4DC(temp_r3);
            return null_mut();
        }
        (*phi_r30).translate = *arg2;
        let scale = ext::Ground_801C0498();
        (*phi_r30).scale.x *= scale;
        (*phi_r30).scale.y *= scale;
        (*phi_r30).scale.z *= scale;
        (*temp_r3).pos.z = 0.0;
        (*temp_r3).pos.y = 0.0;
        (*temp_r3).pos.x = 0.0;
    }
    temp_r3
}

pub unsafe fn grLib_801C97DC(arg0: s32, arg1: s32, arg2: *mut HSD_JObj) {
    melee_core::ct!("grLib_801C97DC");
    ext::hsd_8039F6CC(0, arg1, arg0, arg2);
}

pub unsafe fn grLib_801C9808(arg0: s32, arg1: s32, arg2: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("grLib_801C9808");
    ext::hsd_8039EFAC(0, arg1, arg0, arg2)
}

pub unsafe fn grLib_801C9834(arg0: *mut HSD_Generator) {
    melee_core::ct!("grLib_801C9834");
    ext::hsd_8039D4DC(arg0);
}

pub unsafe fn grLib_801C9854(jobj: *mut HSD_JObj) {
    melee_core::ct!("grLib_801C9854");
    ext::hsd_8039D5DC(jobj);
}

pub unsafe fn grLib_801C9874(arg0: *mut HSD_Generator) {
    melee_core::ct!("grLib_801C9874");
    (*arg0).r#type |= 0x80;
    ext::hsd_8039D4DC(arg0);
}

pub unsafe fn grLib_801C98A0(jobj: *mut HSD_JObj) {
    melee_core::ct!("grLib_801C98A0");
    if jobj.is_null() {
        return;
    }
    let mut cur = ext::hsd_804D78FC;
    while !cur.is_null() {
        let next = (*cur).next;
        if (*cur).jobj == jobj {
            (*cur).r#type |= 0x80;
            ext::hsd_8039D4DC(cur);
        }
        cur = next;
    }
}

pub unsafe fn grLib_801C9908(jobj: *mut HSD_JObj) {
    melee_core::ct!("grLib_801C9908");
    let mut jobj = jobj;
    if jobj.is_null() {
        return;
    }
    let mut cur = ext::hsd_804D78FC;
    while !cur.is_null() {
        let next = (*cur).next;
        if (*cur).jobj == jobj {
            (*cur).r#type |= 0x80;
            ext::hsd_8039D4DC(cur);
        }
        cur = next;
    }

    if (*jobj).flags & JOBJ_INSTANCE as u32 != 0 {
        return;
    }

    jobj = HSD_JObjGetChild(jobj);
    while !jobj.is_null() {
        grLib_801C9908(jobj);
        jobj = HSD_JObjGetNext(jobj);
    }
}

pub unsafe fn grLib_801C99C0(bank: s32, gfx_id: s32, jobj: *mut HSD_JObj, flag: s32) {
    melee_core::ct!("grLib_801C99C0");
    if flag != 0 {
        ext::hsd_8039EFAC(0, bank, gfx_id, jobj);
    } else {
        ext::hsd_8039F6CC(0, bank, gfx_id, jobj);
    }
}

pub unsafe fn grLib_801C9A10() -> *mut Vec3 {
    melee_core::ct!("grLib_801C9A10");
    ext::Ground_801C2D24(0, &mut grLib_8049EF58[0]);
    ext::Ground_801C2D24(1, &mut grLib_8049EF58[1]);
    ext::Ground_801C2D24(2, &mut grLib_8049EF58[2]);
    ext::Ground_801C2D24(3, &mut grLib_8049EF58[3]);
    addr_of_mut!(grLib_8049EF58) as *mut Vec3
}

pub unsafe fn grLib_801C9A70(arg0: enum_t, v: *mut Vec3) {
    melee_core::ct!("grLib_801C9A70");
    let i = match arg0 {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        _ => {
            assert_(false, 290, "0");
            0
        }
    };
    *v = grLib_8049EF58[i];
}

pub unsafe fn grLib_801C9B20(arg0: *mut HSD_JObj, arg1: *mut DynamicsDesc, arg2: *mut DynamicsDesc) {
    melee_core::ct!("grLib_801C9B20");
    ext::lb_8000FD48(arg0, arg2, (*arg1).count as u32);
    ext::lb_80011710(arg1, arg2);
}

pub unsafe fn grLib_801C9B6C(arg0: *mut DynamicsDesc) {
    melee_core::ct!("grLib_801C9B6C");
    ext::lb_8000FD18(arg0);
}

pub unsafe fn grLib_801C9B8C(arg0: *mut DynamicsDesc) {
    melee_core::ct!("grLib_801C9B8C");
    ext::lb_8001044C(arg0, null_mut(), 0, 0.0, 0, 0 as Fighter_Part, 0, 0);
}

/// `static void grLib_801C9BC8(HSD_GObj*)`.
#[inline(never)] // compared by address: keep one copy
pub unsafe extern "C" fn grLib_801C9BC8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLib_801C9BC8");
    let jobj = GET_JOBJ(gobj);
    ext::HSD_JObjAnimAll(jobj);
    ext::Camera_SetQuakeOffset(HSD_JObjGetTranslationX(jobj), HSD_JObjGetTranslationY(jobj));
}

/// `static void grLib_801C9C40(HSD_GObj*)`.
#[inline(never)] // compared by address: keep one copy
pub unsafe extern "C" fn grLib_801C9C40(gobj: *mut HSD_GObj) {
    melee_core::ct!("grLib_801C9C40");
    let jobj = GET_JOBJ(gobj);
    let aobj = (*jobj).aobj;

    ext::HSD_JObjAnimAll(jobj);
    ext::Camera_SetQuakeOffset(HSD_JObjGetTranslationX(jobj), HSD_JObjGetTranslationY(jobj));

    if aobj.is_null() || (*aobj).flags & 0x4000_0000 != 0 {
        ext::HSD_GObjFree(gobj);
    }
}

pub unsafe fn grLib_801C9CEC(kind: CmQuakeKind) -> *mut HSD_GObj {
    melee_core::ct!("grLib_801C9CEC");
    let quake_idx: usize = if kind == QuakeKind_Loop {
        0
    } else if kind == QuakeKind_Small {
        1
    } else if kind == QuakeKind_Medium {
        2
    } else if kind == QuakeKind_Large {
        3
    } else {
        return null_mut();
    };

    let si = ext::stage_info();
    if (*si).quake_model_set.is_null() {
        return null_mut();
    }
    let tmp = kind as u8;
    let gobj = ext::GObj_Create(HSD_GOBJ_CLASS_STAGE as u16, 18, tmp);
    let jobj = ext::HSD_JObjLoadJoint((*(*si).quake_model_set).joint);

    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_JObjKind, jobj as *mut c_void);
    if kind == QuakeKind_Loop {
        ext::HSD_GObj_SetupProc(gobj, Some(grLib_801C9BC8), 1);
    } else {
        ext::HSD_GObj_SetupProc(gobj, Some(grLib_801C9C40), 1);
    }

    ext::HSD_JObjAddAnimAll(jobj, *(*(*si).quake_model_set).anims.add(quake_idx), null_mut(), null_mut());
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    if kind == QuakeKind_Loop {
        // HSD_ForeachAnim(jobj, 6, 0x20, HSD_AObjSetFlags, AOBJ_ARG_AU, AOBJ_LOOP): the variadic
        // argument (AOBJ_LOOP) is the constant 0x20000000.
        ext::HSD_ForeachAnim(jobj as *mut c_void, 6, 0x20, ext::HSD_AObjSetFlags as *mut c_void, AOBJ_ARG_AU);
    }
    gobj
}

pub unsafe fn grLib_801C9E40() -> s32 {
    melee_core::ct!("grLib_801C9E40");
    (*ext::stage_info()).x708 as s32
}

pub unsafe fn grLib_801C9E50(val: s16) {
    melee_core::ct!("grLib_801C9E50");
    (*ext::stage_info()).x708 = val;
}

pub unsafe fn grLib_801C9E60(v: *mut Vec3) -> bool {
    melee_core::ct!("grLib_801C9E60");
    let id = (*ext::stage_info()).grkind;

    if id == Gr_Kind_RCruise {
        ext::grRCruise_80201918(v);
        return true;
    }

    if id == Gr_Kind_BigBlue {
        ext::grBigBlue_801EF7D8(v);
        return true;
    }

    if id == Gr_Kind_Icemt {
        ext::grIceMt_801FA728(v);
        return true;
    }

    (*v).z = 0.0;
    (*v).y = 0.0;
    (*v).x = 0.0;

    false
}

/// `static inline bool PointInsideColl(CollData*, Vec3*, float)` (two fused `0.5f * a + b`
/// per axis, shipping contraction).
unsafe fn PointInsideColl(cd: *mut CollData, point: *mut Vec3, offset: f32) -> bool {
    let top = (*cd).ecb.top.y;
    let bottom = (*cd).ecb.bottom.y;
    let mut topn = (*cd).cur_pos.y;
    let height = top - bottom;
    let mut comp = fmadds(0.5, height, offset);
    let y = fmadds(0.5, top + bottom, topn) - (*point).y;
    if ABS(y) > comp {
        return false;
    }

    let right = (*cd).ecb.right.x;
    let left = (*cd).ecb.left.x;
    topn = (*cd).cur_pos.x;
    let width = right - left;
    comp = fmadds(0.5, width, offset);
    let x = fmadds(0.5, left + right, topn) - (*point).x;
    if ABS(x) > comp {
        return false;
    }

    true
}

pub unsafe fn grLib_801C9EE8(point: *mut Vec3, offset: f32) -> bool {
    melee_core::ct!("grLib_801C9EE8");
    let fighters = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    let mut cur_fighter = fighters;
    while !cur_fighter.is_null() {
        let fp = (*cur_fighter).user_data as *mut Fighter;
        if PointInsideColl(&mut (*fp).coll_data, point, offset) {
            return true;
        }
        cur_fighter = (*cur_fighter).next;
    }

    let items = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
    let mut cur_item = items;
    while !cur_item.is_null() {
        if ext::itGetKind(cur_item) != It_PKind_Random {
            let ip = (*cur_item).user_data as *mut Item;
            if PointInsideColl(&mut (*ip).x378_itemColl, point, offset) {
                return true;
            }
        }
        cur_item = (*cur_item).next;
    }
    false
}

// ---- folded from grlib.rs (consolidation steps 21/22) ----
/// grLib_801C96E8
pub fn grlib_801c96e8(gp: &Ground) -> bool {
    melee_core::ct!("grlib_801c96e8");
    gp.x10_flags.b4() != 0
}

/// grLib_801C97DC
pub fn grlib_801c97dc(h: &mut dyn GrLibHooks, arg0: i32, arg1: i32, j: *mut HSD_JObj) {
    melee_core::ct!("grlib_801c97dc");
    h.hsd_8039f6cc(0, arg1, arg0, j);
}

/// grLib_801C9808
pub fn grlib_801c9808(h: &mut dyn GrLibHooks, arg0: i32, arg1: i32, j: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("grlib_801c9808");
    h.hsd_8039efac(0, arg1, arg0, j)
}

/// grLib_801C9834
pub fn grlib_801c9834(h: &mut dyn GrLibHooks, g: *mut HSD_Generator) {
    melee_core::ct!("grlib_801c9834");
    h.hsd_8039d4dc(g);
}

/// grLib_801C9854
pub fn grlib_801c9854(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    melee_core::ct!("grlib_801c9854");
    h.hsd_8039d5dc(j);
}

/// grLib_801C9874
pub fn grlib_801c9874(h: &mut dyn GrLibHooks, g: *mut HSD_Generator) {
    melee_core::ct!("grlib_801c9874");
    unsafe { (*g).r#type |= 0x80 };
    h.hsd_8039d4dc(g);
}

/// grLib_801C98A0
pub fn grlib_801c98a0(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    melee_core::ct!("grlib_801c98a0");
    if j.is_null() {
        return;
    }
    kill_generators_on(h, j);
}

/// grLib_801C9908 (recursive over the joint tree, stops at instances)
pub fn grlib_801c9908(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    melee_core::ct!("grlib_801c9908");
    if j.is_null() {
        return;
    }
    kill_generators_on(h, j);
    if unsafe { (*j).flags } & JOBJ_INSTANCE as u32 != 0 {
        return;
    }
    let mut c = unsafe { (*j).child };
    while !c.is_null() {
        grlib_801c9908(h, c);
        c = unsafe { (*c).next };
    }
}

/// grLib_801C99C0
pub fn grlib_801c99c0(h: &mut dyn GrLibHooks, bank: i32, gfx_id: i32, j: *mut HSD_JObj, flag: i32) {
    melee_core::ct!("grlib_801c99c0");
    if flag != 0 {
        h.hsd_8039efac(0, bank, gfx_id, j);
    } else {
        h.hsd_8039f6cc(0, bank, gfx_id, j);
    }
}

/// grLib_801C9B20
pub fn grlib_801c9b20(h: &mut dyn GrLibHooks, j: *mut HSD_JObj, a1: *mut DynamicsDesc, a2: *mut DynamicsDesc) {
    melee_core::ct!("grlib_801c9b20");
    let count = unsafe { (*a1).count };
    h.lb_8000fd48(j, a2, count as i32);
    h.lb_80011710(a1, a2);
}

/// grLib_801C9B6C
pub fn grlib_801c9b6c(h: &mut dyn GrLibHooks, a: *mut core::ffi::c_void) {
    melee_core::ct!("grlib_801c9b6c");
    h.lb_8000fd18(a);
}

/// grLib_801C9B8C
pub fn grlib_801c9b8c(h: &mut dyn GrLibHooks, a: *mut core::ffi::c_void) {
    melee_core::ct!("grlib_801c9b8c");
    h.lb_8001044c_zero(a);
}

/// grLib_801C9BC8 (static proc, loop quake)
pub fn grlib_801c9bc8(h: &mut dyn GrLibHooks, gobj: *mut HSD_GObj) {
    melee_core::ct!("grlib_801c9bc8");
    let j = unsafe { (*gobj).hsd_obj } as *mut HSD_JObj;
    h.jobj_anim_all(j);
    let (x, y) = h.jobj_translation_xy(j);
    h.camera_set_quake_offset(x, y);
}

/// grLib_801C9C40 (static proc, one-shot quake)
pub fn grlib_801c9c40(h: &mut dyn GrLibHooks, gobj: *mut HSD_GObj) {
    melee_core::ct!("grlib_801c9c40");
    let j = unsafe { (*gobj).hsd_obj } as *mut HSD_JObj;
    let aobj = unsafe { (*j).aobj };
    h.jobj_anim_all(j);
    let (x, y) = h.jobj_translation_xy(j);
    h.camera_set_quake_offset(x, y);
    if aobj.is_null() || unsafe { (*aobj).flags } & 0x4000_0000 != 0 {
        h.gobj_free(gobj);
    }
}

/// grLib_801C9EE8
pub fn grlib_801c9ee8(h: &mut dyn GrLibHooks, p: &Vec3_grlib, offset: f32) -> bool {
    melee_core::ct!("grlib_801c9ee8");
    for cd in h.fighter_colls() {
        if point_inside_coll(unsafe { &*cd }, p, offset) {
            return true;
        }
    }
    for (kind, cd) in h.item_colls() {
        if kind != It_PKind_Random && point_inside_coll(unsafe { &*cd }, p, offset) {
            return true;
        }
    }
    false
}
