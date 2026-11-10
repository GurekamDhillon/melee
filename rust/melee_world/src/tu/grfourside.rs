//! Port of src/melee/gr/grfourside.c: Fourside (seven stage gobjs, "/GrFs.dat"): gobj 6 is the
//! sky/ground that keeps the three background billboards facing the camera (`fn_801F3F74`),
//! gobj 1 the construction crane (a seven-state machine moving the iron beam), gobj 5 the UFO
//! that visits one of three buildings and takes over the camera, gobj 3 the helicopter, and the
//! joint callback `grFourside_801F30A0` counts the hits on the UFO. Callees of unported TUs go
//! through `ext`; the constant-return callbacks are the earlier round's ports (`gr::trivial`),
//! re-exported. Checked against the C (whole TU, shipping contraction) by tests/tu_grfourside.rs.
//!
//! The crane's `foursideCrane.x1` byte holds C bitfields; they are read and written through the
//! GameCube-order accessors (b0 = MSB), so the raw byte differs from a native C build while every
//! field value is the same (the oracle compares the bits by value).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::grdisplay::grDisplay_801C5DB0;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_80029020() -> *mut CmSubject;
    fn Camera_800290D4(a0: *mut CmSubject);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj| { unsafe { crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_CObjGetCurrent() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() as _ };
    fn HSD_CObjGetEyeVector(a0: *mut HSD_CObj, a1: *mut Vec) -> i32 = |a0: *mut HSD_CObj, a1: *mut Vec| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetEyeVector(a0 as _, a1 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn atan2f(a0: f32, a1: f32) -> f32 = |a0: f32, a1: f32| unsafe { melee_runtime::lbtrigf::atan2f(a0 as _, a1 as _) as _ };
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C7B24(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7B24(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grAnime_801C84A4(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C84A4(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80055E9C(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E9C(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057424(a0 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

/// `struct grFourside_YakumonoParam` (the stage's yakumono block)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grFourside_YakumonoParam {
    pub heli_wait: i32,
    pub heli_wait_add: i32,
    pub heli_stay_time: i32,
    pub crane_wait: i32,
    pub crane_wait_add: i32,
    pub crane_iron_wait: i32,
    pub crane_iron_wait_add: i32,
    pub crane_iron_up_min: f32,
    pub crane_iron_up_max: f32,
    pub crane_iron_down_min: f32,
    pub crane_iron_down_max: f32,
    pub crane_iron_spd: f32,
    pub crane_iron_stop_acl: f32,
    pub ufo_wait: i32,
    pub ufo_cs_offs: f32,
    pub ufo_stay_time: i32,
    pub ufo_stay_time_add: i32,
    pub ufo_challenge: u16,
    pub x46: u16,
    pub x48: u16,
}

/// `static struct grFourside_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grFourside_YakumonoParam = null_mut();

/// `GrJoint grFs_803E3CE8[]`
pub static mut grFs_803E3CE8: [GrJoint; 3] = [
    GrJoint { x: 3, y: 1, z: 7 },
    GrJoint { x: 4, y: 5, z: 1 },
    GrJoint { x: 0, y: 6, z: 10 },
];

/// `const fn` for a `StageCallbacks` table row: `{ on_init, callback1, gobj_proc, callback3, flags }`.
const fn row(
    on_init: HSD_GObjEvent,
    callback1: HSD_GObjPredicate,
    gobj_proc: HSD_GObjEvent,
    callback3: HSD_GObjEvent,
    flags: u32,
) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, x10: StageCallbacks_x10 { flags: u32::to_be(flags) } }
}

/// `StageCallbacks grFs_StageCallbacks[]`
pub static mut grFs_StageCallbacks: [StageCallbacks; 7] = [
    row(Some(stageGObj0_OnInit), Some(grFourside_801F2EE8), Some(grFourside_801F2EF0), Some(grFourside_801F2EF4), 0),
    row(Some(grFourside_801F3154), Some(grFourside_801F326C), Some(grFourside_801F3274), Some(grFourside_801F37F8), 0),
    row(Some(stageGObj2_OnInit), Some(grFourside_801F2F24), Some(grFourside_801F2F2C), Some(grFourside_801F2F30), 0),
    row(Some(grFourside_801F3C40), Some(grFourside_801F3CC0), Some(grFourside_801F3CC8), Some(grFourside_801F3F0C), 0),
    row(Some(grFourside_801F30F0), Some(grFourside_801F3144), Some(grFourside_801F314C), Some(grFourside_801F3150), 0),
    row(Some(grFourside_801F37FC), Some(grFourside_801F388C), Some(grFourside_801F3894), Some(grFourside_801F3B6C), 0),
    row(Some(grFourside_801F2F34), Some(grFourside_801F3070), Some(grFourside_801F3078), Some(grFourside_801F309C), (1 << 30) | (1 << 31)),
];

/// `StageData grFs_StageData`
pub static mut grFs_StageData: StageData = StageData {
    grkind: Gr_Kind_Fourside,
    callbacks: unsafe { &raw mut grFs_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrFs.dat".as_ptr() as *mut i8,
    on_init: Some(grFourside_801F2D10),
    on_demo_init: Some(grFourside_801F2D0C),
    on_load: Some(grFourside_801F2DA0),
    on_start: Some(grFourside_801F2DA4),
    callback4: Some(grFourside_801F2DC8),
    on_touch_line: Some(grFourside_801F41E0),
    on_check_shadow_render: Some(grFourside_801F41E8),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grFs_803E3CE8 as *mut GrJoint },
    joint_count: 3,
};

/// `Ground_SetupStageCallbacks` (gr/inlines.h, TARGET_PC arm): binds the row's callbacks (the
/// m-ex `Mex_GrBind` passes native pointers through) and runs `on_init`.
#[inline]
unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
            core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).callback3),
        ));
    }
    if (*callbacks).on_init.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
            core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).on_init),
        ));
        f.expect("on_init")(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        ext::HSD_GObj_SetupProc(
            gobj,
            core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
                core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).gobj_proc),
            )),
            4,
        );
    }
}

/// `Ground_StartMapAnim` (gr/inlines.h)
#[inline]
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `HSD_ASSERT(line, cond)`: `__assert` does not return.
#[inline]
fn hsd_assert(cond: bool, what: &str) {
    if !cond {
        panic!("grfourside.c: HSD_ASSERT {what}");
    }
}

/// `HSD_JObjMtxIsDirty` (baselib/jobj.h, static inline).
#[inline]
unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}

/// `HSD_JObjSetMtxDirty` (macro): marks the joint's matrix dirty unless it already is.
#[inline]
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

/// `HSD_JObjSetTranslateY` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetTranslateY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetTranslationY` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjGetTranslationY(jobj: *mut HSD_JObj) -> f32 {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.y
}

/// `HSD_JObjSetRotationY` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetRotationY(jobj: *mut HSD_JObj, y: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `ZRANDI`-style `n != 0 ? HSD_Randi(n) : 0`
#[inline]
unsafe fn zrandi(n: i32) -> i32 {
    if n != 0 {
        ext::HSD_Randi(n)
    } else {
        0
    }
}

/// 1F2D0C
pub unsafe extern "C" fn grFourside_801F2D0C(_arg: i32) { melee_core::ct!("grFourside_801F2D0C");}

/// 1F2D10
pub unsafe extern "C" fn grFourside_801F2D10() {
    melee_core::ct!("grFourside_801F2D10");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grFourside_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grFourside_801F2DD0(0);
    grFourside_801F2DD0(4);
    grFourside_801F2DD0(6);
    grFourside_801F2DD0(2);
    grFourside_801F2DD0(1);
    grFourside_801F2DD0(3);
    grFourside_801F2DD0(5);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// 1F2DA0
pub unsafe extern "C" fn grFourside_801F2DA0() { melee_core::ct!("grFourside_801F2DA0");}

/// 1F2DA4
pub unsafe extern "C" fn grFourside_801F2DA4() {
    melee_core::ct!("grFourside_801F2DA4");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 1F2DD0: sets up stage gobj `gobj_id` with its row of the callback table.
pub unsafe extern "C" fn grFourside_801F2DD0(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grFourside_801F2DD0");
    let callbacks = &raw mut grFs_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    // else OSReport("%s:%d: couldn t get gobj(id=%d)\n", "grfourside.c", 215, ...): diagnostics only
    gobj
}

/// stageGObj0_OnInit
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

/// 1F2EF0
pub unsafe extern "C" fn grFourside_801F2EF0(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F2EF0");}

/// 1F2EF4
pub unsafe extern "C" fn grFourside_801F2EF4(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F2EF4");}

/// stageGObj2_OnInit
pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    Ground_StartMapAnim(gobj);
}

/// 1F2F2C
pub unsafe extern "C" fn grFourside_801F2F2C(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F2F2C");}

/// 1F2F30
pub unsafe extern "C" fn grFourside_801F2F30(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F2F30");}

/// 1F2F34: gobj 6 (sky): the three billboards, the render callback and the hit-count callback.
pub unsafe extern "C" fn grFourside_801F2F34(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F2F34");
    let gp = (*gobj).user_data as *mut Ground;
    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    let a0 = ext::Ground_801C3FA4(gobj, 7);
    let a1 = ext::Ground_801C3FA4(gobj, 4);
    let a2 = ext::Ground_801C3FA4(gobj, 6);
    let a3 = ext::Ground_801C3FA4(gobj, 5);
    let a4 = ext::Ground_801C3FA4(gobj, 9);
    let a5 = ext::Ground_801C3FA4(gobj, 8);
    ext::Ground_801C4E70(a0, a1, a2, a3, a4, a5);
    (*gobj).render_cb = Some(fn_801F3F74);
    (*gp).u.fourside.x0 = ext::Ground_801C3FA4(gobj, 2);
    (*gp).u.fourside.x4 = ext::Ground_801C3FA4(gobj, 3);
    let p = ext::Ground_GetMapGObj(4);
    if !p.is_null() {
        (*gp).u.fourside.x8 = (*p).hsd_obj as *mut HSD_JObj;
    } else {
        (*gp).u.fourside.x8 = null_mut();
    }
    ext::mpJointSetCb1(0, gobj.cast(), Some(grFourside_801F30A0));
    (*gp).x10_flags.set_b5(1);
    (*gp).x11_flags.set_b012(1);
}

/// 1F3078
pub unsafe extern "C" fn grFourside_801F3078(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3078");
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// 1F309C
pub unsafe extern "C" fn grFourside_801F309C(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F309C");}

/// 1F30A0 (mpLib_JointCollisionCallback): counts the hits on the UFO (map gobj 5).
pub unsafe extern "C" fn grFourside_801F30A0(
    _user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    _ground_kind: u32,
    _delta_y: f32,
) {
    melee_core::ct!("grFourside_801F30A0");
    let gobj = ext::Ground_GetMapGObj(5);
    let mut val = ((*coll).x34_flags.b1234() as i32) << 3;
    let gp1 = (*gobj).user_data as *mut Ground;
    val = (val >> 3) & 0xF;
    if val == 1 {
        (*gp1).u.fourside2.x8 += 1;
    }
}

/// 1F30F0
pub unsafe extern "C" fn grFourside_801F30F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F30F0");
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).x10_flags.set_b2(0);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
}

/// 1F314C
pub unsafe extern "C" fn grFourside_801F314C(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F314C");}

/// 1F3150
pub unsafe extern "C" fn grFourside_801F3150(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F3150");}

/// 1F3154: gobj 1 (crane) init.
pub unsafe extern "C" fn grFourside_801F3154(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3154");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let crane_iron = ext::Ground_801C3FA4(gobj, 4);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 0.0);
    (*gp).u.foursideCrane.x1.set_b0(0);
    (*gp).u.foursideCrane.x4 = (*yakumono_param).crane_wait.wrapping_add(zrandi((*yakumono_param).crane_wait_add));
    (*gp).u.foursideCrane.x8 = HSD_JObjGetTranslationY(crane_iron);
    (*gp).u.foursideCrane.x8 = (*gp).u.foursideCrane.x8 - 10.0;
    (*gp).u.foursideCrane.x10 = (*gp).u.foursideCrane.x8;
    (*gp).u.foursideCrane.xC = (*gp).u.foursideCrane.x10;
    (*gp).u.foursideCrane.x0 = 0;
    (*gp).x10_flags.set_b5(1);
    (*gp).x11_flags.set_b012(1);
}

/// `static inline void grFourside_UpdateCrane(Ground*, HSD_GObj*)`: one step of the crane's machine.
pub unsafe fn grFourside_UpdateCrane(gp: *mut Ground, hsd_gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_UpdateCrane");
    let c = &raw mut (*gp).u.foursideCrane;
    let yp = yakumono_param;
    match (*c).x0 {
        0 => {
            if (*c).x4 <= 0 {
                if (*c).x1.b0() == 0 {
                    ext::grAnime_801C7B24(hsd_gobj, 0, 7, 0.0);
                    ext::grAnime_801C7A04(hsd_gobj, 0, 7, 1.0);
                    (*c).x4 = 0;
                    (*c).x0 = 1;
                } else {
                    ext::grAnime_801C7B24(hsd_gobj, 0, 7, 400.0);
                    ext::grAnime_801C7A04(hsd_gobj, 0, 7, 1.0);
                    (*c).x4 = 0;
                    (*c).x0 = 2;
                }
            } else {
                (*c).x4 -= 1;
            }
        }
        1 => {
            if (*c).x4 >= 300 {
                ext::grAnime_801C7A04(hsd_gobj, 0, 7, 0.0);
                (*c).x1.set_b0(1);
                (*c).x0 = 3;
            } else {
                (*c).x4 += 1;
            }
        }
        2 => {
            if (*c).x4 >= 300 {
                ext::grAnime_801C7A04(hsd_gobj, 0, 7, 0.0);
                (*c).x1.set_b0(0);
                (*c).x0 = 3;
            } else {
                (*c).x4 += 1;
            }
        }
        3 => {
            (*c).x4 = (*yp).crane_iron_wait.wrapping_add(zrandi((*yp).crane_iron_wait_add));
            (*c).x0 = 4;
        }
        4 => {
            if (*c).x4 <= 0 {
                (*c).x10 = (*c).x8;
                if (*c).xC > (*c).x8 {
                    let mut f = (*yp).crane_iron_down_max - (*yp).crane_iron_down_min;
                    if f < 0.0 {
                        f = -f;
                    }
                    let rand_max2 = melee_core::ppc::fctiwz_f32(f);
                    let rand_max = melee_core::ppc::fctiwz_f32(f);
                    f = if rand_max != 0 { ext::HSD_Randi(rand_max2) as f32 } else { 0.0 };
                    let mut t = (*yp).crane_iron_down_min;
                    if t < 0.0 {
                        t = -t;
                    }
                    (*c).x10 = (*c).x10 - (f + t);
                    (*c).x1.set_b1(1);
                    // OSReport("pos = %f : tgrpos = %f\n", ...): diagnostics only
                    (*c).x0 = 5;
                } else {
                    let mut f = (*yp).crane_iron_up_max - (*yp).crane_iron_up_min;
                    if f < 0.0 {
                        f = -f;
                    }
                    let rand_max2 = melee_core::ppc::fctiwz_f32(f);
                    let rand_max = melee_core::ppc::fctiwz_f32(f);
                    f = if rand_max != 0 { ext::HSD_Randi(rand_max2) as f32 } else { 0.0 };
                    let mut t = (*yp).crane_iron_up_min;
                    if t < 0.0 {
                        t = -t;
                    }
                    (*c).x10 = (*c).x10 + (f + t);
                    (*c).x1.set_b1(0);
                    // OSReport("pos = %f : tgrpos = %f\n", ...): diagnostics only
                    (*c).x0 = 6;
                }
            } else {
                (*c).x4 -= 1;
            }
        }
        5 => {
            (*c).xC -= (*yp).crane_iron_spd;
            if (*c).xC <= (*c).x10 {
                (*c).xC = (*c).x10;
                (*c).x18 = (*yp).crane_iron_spd;
                (*c).x14 = (*c).x18;
                (*c).x1C = (*yp).crane_iron_stop_acl;
                (*c).x0 = 7;
            }
        }
        6 => {
            (*c).xC += (*yp).crane_iron_spd;
            if (*c).xC >= (*c).x10 {
                (*c).x18 = (*yp).crane_iron_spd;
                (*c).x14 = (*c).x18;
                (*c).x1C = (*yp).crane_iron_stop_acl;
                (*c).x0 = 7;
            }
        }
        7 => {
            let t = (*c).x1C;
            let f = if t < 0.0 { -t } else { t };
            if f > (*c).x14 {
                (*c).xC = (*c).x10;
                (*c).x4 = (*yp).crane_wait.wrapping_add(zrandi((*yp).crane_wait_add));
                (*c).x0 = 0;
            } else {
                (*c).x18 += t;
                (*c).xC += (*c).x18;
                if (*c).x1C < 0.0 && (*c).xC < (*c).x10 {
                    (*c).x1C *= -2.0;
                }
                if (*c).x1C > 0.0 && (*c).xC > (*c).x10 {
                    (*c).x1C *= -2.0;
                }
            }
        }
        _ => {}
    }
}

/// `static inline float grFourside_GetCraneY(Ground*)`
pub unsafe fn grFourside_GetCraneY(gp: *mut Ground) -> f32 {
    melee_core::ct!("grFourside_GetCraneY");
    (*gp).u.foursideCrane.xC
}

/// `static inline HSD_JObj* grFourside_GetCraneIron(HSD_GObj*)`
pub unsafe fn grFourside_GetCraneIron(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    melee_core::ct!("grFourside_GetCraneIron");
    ext::Ground_801C3FA4(gobj, 4)
}

/// 1F3274: the crane per frame.
pub unsafe extern "C" fn grFourside_801F3274(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3274");
    let gp = (*gobj).user_data as *mut Ground;
    let crane_iron = grFourside_GetCraneIron(gobj);
    grFourside_UpdateCrane(gp, gobj);
    HSD_JObjSetTranslateY(crane_iron, grFourside_GetCraneY(gp));
    ext::Ground_UpdateMapColl(gobj);
}

/// 1F37F8
pub unsafe extern "C" fn grFourside_801F37F8(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F37F8");}

/// 1F37FC: gobj 5 (UFO) init.
pub unsafe extern "C" fn grFourside_801F37FC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F37FC");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    let new_var = (*yakumono_param).ufo_wait;
    (*gp).u.foursideCrane.x4 = new_var;
    (*gp).u.foursideUfo.x0 = 0;
    (*gp).u.foursideUfo.x1 = 0xff;
    (*gp).u.foursideUfo.x8 = 0;
    (*gp).u.foursideUfo.x2 = 0;
    (*gp).u.foursideUfo.xC = null_mut();
    (*gp).u.foursideUfo.x3 = 0;
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    ext::mpLib_80057BC0(4);
    (*gp).x10_flags.set_b5(1);
}

/// `static inline void grFourside_801F3894_inline(...)`: one step of the UFO's machine.
pub unsafe fn grFourside_801F3894_inline(gobj: *mut HSD_GObj, gp: *mut Ground, jobj: *mut HSD_JObj, state: u8, prev_building: u8) {
    melee_core::ct!("grFourside_801F3894_inline");
    let u = &raw mut (*gp).u.foursideUfo;
    let yp = yakumono_param;
    match state {
        0 => {
            let timer = (*u).x4;
            if timer <= 0 {
                let mut prob: i32 = 0;
                if (*u).x8 != 0 {
                    prob = (*yp).x46 as i16 as i32;
                }
                if (*u).x2 as i32 >= (*yp).ufo_challenge as i16 as i32 {
                    prob = (*yp).x48 as i16 as i32;
                }
                if prob != 0 && ext::HSD_Randi(prob) == 0 {
                    let mut building: i32;
                    (*u).x2 = 0;
                    loop {
                        building = ext::HSD_Randi(3);
                        if prev_building as i32 != building {
                            break;
                        }
                    }
                    if building == 2 || grFourside_801F3F10() == 0 {
                        (*u).x1 = building as u8;
                        ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*u).x1 as i32 * 4);
                        ext::mpLib_80055E9C(4);
                        ext::mpLib_80057424(4);
                        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                        ext::mpJointListAdd(4);
                        (*u).xC = ext::Camera_80029020();
                        (*u).x3 = 0;
                        (*u).x0 = 1;
                    }
                } else {
                    (*u).x4 = (*yp).ufo_wait;
                    (*u).x2 = (*u).x2.wrapping_add(1);
                }
            } else {
                (*u).x4 = timer.wrapping_sub(1);
            }
        }
        1 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                let rand_add = (*yp).ufo_stay_time_add;
                let var_r6 = if rand_add != 0 { ext::HSD_Randi(rand_add) } else { 0 };
                (*u).x4 = (*yp).ufo_stay_time.wrapping_add(var_r6);
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (prev_building as i32 * 4) + 1);
                ext::Camera_800290D4((*u).xC);
                (*u).xC = null_mut();
                (*u).x0 = 2;
            }
            grFourside_801F3B70(gobj);
            if (*u).x3 == 1 {
                let building = (*u).x1;
                if building == 0 {
                    ext::Ground_801C53EC(0x704E4);
                } else if building == 1 {
                    ext::Ground_801C53EC(0x704E0);
                } else {
                    ext::Ground_801C53EC(0x704E3);
                }
                (*u).x3 = (*u).x3.wrapping_add(1);
            }
        }
        2 => {
            let timer = (*u).x4;
            if timer <= 0 {
                if ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (prev_building as i32 * 4) + 2);
                    (*u).x0 = 4;
                }
            } else {
                (*u).x4 = timer.wrapping_sub(1);
            }
        }
        4 => {
            let timer = (*u).x4;
            if timer >= 0x3C {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                ext::mpLib_80057BC0(4);
                (*u).x4 = (*yp).ufo_wait;
                (*u).x0 = 0;
            } else {
                (*u).x4 = timer.wrapping_add(1);
            }
        }
        _ => {}
    }
    ext::Ground_UpdateMapColl(gobj);
    (*u).x8 = 0;
}

/// 1F3894: the UFO per frame.
pub unsafe extern "C" fn grFourside_801F3894(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3894");
    let gp = (*gobj).user_data as *mut Ground;
    let state = (*gp).u.foursideUfo.x0;
    let prev_building = (*gp).u.foursideUfo.x1;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    grFourside_801F3894_inline(gobj, gp, jobj, state, prev_building);
}

/// 1F3B6C
pub unsafe extern "C" fn grFourside_801F3B6C(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F3B6C");}

/// 1F3B70: while the UFO is above the camera limit, pins the camera subject to its position.
pub unsafe extern "C" fn grFourside_801F3B70(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3B70");
    let mut local18: Vec = core::mem::zeroed();
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = ext::Ground_801C3FA4(gobj, 1);
    ext::lb_8000B1CC(jobj, null_mut(), &mut local18);
    if !(*gp).u.foursideUfo.xC.is_null() {
        if local18.y <= ext::Stage_GetCamBoundsTopOffset() + (*yakumono_param).ufo_cs_offs {
            if (*gp).u.foursideUfo.x3 == 0 {
                (*gp).u.foursideUfo.x3 = 1;
            }
            let cam = (*gp).u.foursideUfo.xC;
            (*cam).pos = local18;
            (*cam).bone_pos = local18;
            (*cam).target_ext.h.x = -50.0;
            (*cam).target_ext.h.y = 50.0;
            (*cam).target_ext.v.x = 15.0;
            (*cam).target_ext.v.y = -15.0;
        }
    }
}

/// 1F3C40: gobj 3 (helicopter) init.
pub unsafe extern "C" fn grFourside_801F3C40(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3C40");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    (*gp).x11_flags.set_b012(1);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.fourside2.x0 = 0;
    (*gp).u.fourside2.x4 = (*yakumono_param).heli_wait.wrapping_add(zrandi((*yakumono_param).heli_wait_add));
    (*gp).u.fourside2.x1 = 0;
}

/// 1F3CC8: the helicopter per frame.
pub unsafe extern "C" fn grFourside_801F3CC8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFourside_801F3CC8");
    let mut pos: Vec3 = core::mem::zeroed();
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let heli_jobj = ext::Ground_801C3FA4(gobj, 2);

    match (*gp).u.fourside2.x0 {
        0 => {
            if (*gp).u.fourside2.x4 <= 0 {
                let other = ext::Ground_GetMapGObj(5);
                let other_gp = (*other).user_data as *mut Ground;
                let other_active = !((*other_gp).u.fourside2.x1 == 2 || (*other_gp).u.fourside2.x0 == 0);
                if !other_active {
                    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                    (*gp).u.fourside2.x1 = 0;
                    (*gp).u.fourside2.x0 = 1;
                }
            } else {
                (*gp).u.fourside2.x4 -= 1;
            }
        }
        1 => {
            if (*gp).u.fourside2.x1 == 0 {
                ext::lb_8000B1CC(heli_jobj, null_mut(), &mut pos);
                if pos.y <= 50.0 + ext::Stage_GetCamBoundsTopOffset() {
                    (*gp).u.fourside2.x1 = 1;
                    ext::Ground_801C53EC(0x704E1);
                }
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                (*gp).u.fourside2.x0 = 2;
            }
        }
        2 => {
            if (*gp).u.fourside2.x4 <= 0 {
                let other = ext::Ground_GetMapGObj(5);
                let other_gp = (*other).user_data as *mut Ground;
                let other_active = !((*other_gp).u.fourside2.x1 == 2 || (*other_gp).u.fourside2.x0 == 0);
                if !other_active {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 2);
                    (*gp).u.fourside2.x0 = 3;
                    ext::Ground_801C53EC(0x704E2);
                    (*gp).u.fourside2.x4 = 0;
                }
            } else {
                (*gp).u.fourside2.x4 -= 1;
            }
        }
        3 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                (*gp).u.fourside2.x0 = 0;
                (*gp).u.fourside2.x4 = (*yakumono_param).heli_wait.wrapping_add(zrandi((*yakumono_param).heli_wait_add));
            }
            (*gp).u.fourside2.x4 += 1;
        }
        _ => {}
    }
}

/// 1F3F0C
pub unsafe extern "C" fn grFourside_801F3F0C(_arg: *mut HSD_GObj) { melee_core::ct!("grFourside_801F3F0C");}

/// 1F3F10: is the helicopter in the way of the UFO's building 0/1 sequence?
pub unsafe extern "C" fn grFourside_801F3F10() -> i32 {
    melee_core::ct!("grFourside_801F3F10");
    let gobj = ext::Ground_GetMapGObj(3);
    let gp = (*gobj).user_data as *mut Ground;
    if ((*gp).u.fourside2.x0 == 1 && (*gp).u.fourside2.x1 == 0) || ((*gp).u.fourside2.x0 == 3 && (*gp).u.fourside2.x4 <= 700) {
        return 1;
    }
    0
}

/// fn_801F3F74: the sky's render callback: turns the three billboards towards the camera, then
/// draws the stage.
pub unsafe extern "C" fn fn_801F3F74(gobj: *mut HSD_GObj, renderpass: isize) {
    melee_core::ct!("fn_801F3F74");
    let mut eye: Vec3 = core::mem::zeroed();
    if renderpass as u32 == 1 {
        let gp = (*gobj).user_data as *mut Ground;
        ext::HSD_CObjGetEyeVector(ext::HSD_CObjGetCurrent(), &mut eye);
        let mut abs_y = eye.y;
        if abs_y < 0.0 {
            abs_y = -abs_y;
        }
        if abs_y < 0.99 {
            let angle = ext::atan2f(-eye.x, -eye.z);
            if !(*gp).u.fourside.x0.is_null() {
                HSD_JObjSetRotationY((*gp).u.fourside.x0, 0.2 * angle);
            }
            if !(*gp).u.fourside.x4.is_null() {
                HSD_JObjSetRotationY((*gp).u.fourside.x4, 0.4 * angle);
            }
            if !(*gp).u.fourside.x8.is_null() {
                HSD_JObjSetRotationY((*gp).u.fourside.x8, 0.7 * angle);
            }
        }
    }
    ext::grDisplay_801C5DB0(gobj, renderpass as isize);
}

/// 1F41E8
pub unsafe extern "C" fn grFourside_801F41E8(a: *mut Vec3, _arg: i32, joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grFourside_801F41E8");
    let mut b: Vec3 = core::mem::zeroed();
    ext::lb_8000B1CC(joint, null_mut(), &mut b);
    if (*a).y > b.y {
        1
    } else {
        0
    }
}

// ---- folded from grfourside.rs (consolidation steps 21/22) ----
/// grFourside_801F2DA4_fp (gr/grfourside.c:154)
    pub unsafe extern "C" fn grFourside_801F2DA4_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grfourside.rs (consolidation steps 21/22) ----
/// grFourside_801F2DC8 (gr/grfourside.c:159)
    pub extern "C" fn grFourside_801F2DC8() -> i32 { 0 }

/// grFourside_801F2EE8 (gr/grfourside.c:186)
    pub extern "C" fn grFourside_801F2EE8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F2F24 (gr/grfourside.c:200)
    pub extern "C" fn grFourside_801F2F24(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F3070 (gr/grfourside.c:233)
    pub extern "C" fn grFourside_801F3070(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F3144 (gr/grfourside.c:275)
    pub extern "C" fn grFourside_801F3144(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F326C (gr/grfourside.c:305)
    pub extern "C" fn grFourside_801F326C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F388C (gr/grfourside.c:509)
    pub extern "C" fn grFourside_801F388C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F3CC0 (gr/grfourside.c:667)
    pub extern "C" fn grFourside_801F3CC0(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFourside_801F41E0 (gr/grfourside.c:796)
    pub extern "C" fn grFourside_801F41E0(a0: i32) -> *mut melee_types::lbColl_80008D30_arg1 { core::ptr::null_mut() }
