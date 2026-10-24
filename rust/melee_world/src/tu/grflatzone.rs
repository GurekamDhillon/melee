//! Port of src/melee/gr/grflatzone.c: Flat Zone (nine stage gobjs, "/GrFz.dat"): the elevated
//! building / crane / machine on the Flat Zone map. Gobj 2 times the next building swap, gobj 3
//! is the crane (a small state machine that drops a dynamic-attribute disc), gobj 5 picks the
//! background machine's animation from a transition table, gobj 8 rains items.
//!
//! Union aliasing: the C names the same `Ground` union storage through `flatzone`, `flatzone2`
//! and `unk`. On the GameCube (4-byte pointers) `flatzone.xD0` / `unk.xD0` / `flatzone2.xD0` are
//! one word at +0xD0 and `flatzone.xD4` is `flatzone2.timer`; with native 8-byte pointers
//! `flatzone2.xCC` pushes `flatzone2.xD0` / `timer` up by 4, so the three views stop aliasing.
//! This port keeps the GameCube aliasing: it uses `flatzone2.xD0` / `flatzone2.timer` for every
//! one of those accesses (the C oracle compiles a copy of the TU with the same three-way
//! substitution, see `gen/mtu/specs.py`). Callees of unported TUs go through `ext`; the
//! constant-return callbacks are the earlier round's ports (`gr::trivial`), re-exported. Checked
//! against the C (whole TU, shipping contraction) by tests/tu_grflatzone.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::srem;
use melee_types::*;

use super::grdisplay::grDisplay_801C5DB0;
pub use crate::gr::trivial::grflatzone::{
    grFlatzone_80216F40, grFlatzone_8021705C, grFlatzone_802171CC, grFlatzone_80217430, grFlatzone_802174E4,
    grFlatzone_802176B4, grFlatzone_80217D28, grFlatzone_80217DFC, grFlatzone_80217E80, grFlatzone_80217EE8,
    grFlatzone_802182B4, grFlatzone_802182BC,
};

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
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
    fn Camera_800311CC(a0: f32) = |a0: f32| crate::cm::camera::world().camera_800311cc(a0);
    fn Camera_800311DC(a0: f32) = |a0: f32| crate::cm::camera::world().camera_800311dc(a0);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { let _ = crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C3990(a0: f32) = |a0: f32| { unsafe { let _ = crate::tu::ground::Ground_801C3990(a0 as _); } };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grDynamicAttr_801CA0F8(a0: i32, a1: *mut Vec, a2: i32, a3: f32, a4: i32) -> *mut grDynamicAttr_UnkStruct = |a0: i32, a1: *mut Vec, a2: i32, a3: f32, a4: i32| unsafe { crate::tu::grdynamicattr::grDynamicAttr_801CA0F8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _) as _ };
    fn grDynamicAttr_801CA1C0(a0: *mut grDynamicAttr_UnkStruct) = |a0: *mut grDynamicAttr_UnkStruct| { unsafe { let _ = crate::tu::grdynamicattr::grDynamicAttr_801CA1C0(a0 as _); } };
    fn grMaterial_801C8858(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8858(a0 as _, a1 as _); } };
    fn it_802EEFA8(a0: *mut Vec, a1: i32, a2: f32) -> *mut HSD_GObj;
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpCheckMultiple(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: u32, a9: i32, a10: i32) -> i32 = |a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut Vec, a5: *mut i32, a6: *mut u32, a7: *mut Vec, a8: u32, a9: i32, a10: i32| unsafe { crate::tu::mplib::mpCheckMultiple(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _, a9 as _, a10 as _) as _ };
    fn mpLib_80058560() = || { unsafe { let _ = crate::tu::mplib::mpLib_80058560(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `struct grFlatzone_YakumonoParam` (the stage's yakumono block)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grFlatzone_YakumonoParam {
    pub unk0: i32,
    pub unk4: i32,
    pub unk8: i32,
    pub unkC: i32,
    pub unk10: i32,
    pub unk14: i32,
    pub unk18: i32,
    pub unk1C: i32,
    pub unk20: i32,
    pub unk24: f32,
    pub unk28: f32,
    pub unk2C: i32,
    pub unk30: i32,
    pub unk34: i32,
    pub unk38: f32,
    pub unk3C: i32,
}

/// `const fn` for a `StageCallbacks` table row: `{ on_init, callback1, gobj_proc, callback3, flags }`.
const fn row(
    on_init: HSD_GObjEvent,
    callback1: HSD_GObjPredicate,
    gobj_proc: HSD_GObjEvent,
    callback3: HSD_GObjEvent,
    flags: u32,
) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags } }
}

/// `static StageCallbacks grFz_StageCallbacks[]`
pub static mut grFz_StageCallbacks: [StageCallbacks; 9] = [
    row(Some(stageGObj0_OnInit), Some(grFlatzone_8021705C), Some(grFlatzone_80217064), Some(grFlatzone_80217068), 0),
    row(Some(grFlatzone_802173D4), Some(grFlatzone_80217430), Some(grFlatzone_80217438), Some(grFlatzone_8021743C), 0),
    row(Some(grFlatzone_8021707C), Some(grFlatzone_802171CC), Some(grFlatzone_802171D4), Some(grFlatzone_802173D0), (1 << 30) | (1 << 31)),
    row(Some(grFlatzone_80217648), Some(grFlatzone_802176B4), Some(grFlatzone_802176BC), Some(grFlatzone_80217CD0), 0),
    row(Some(grFlatzone_80217CD4), Some(grFlatzone_80217D28), Some(grFlatzone_80217D30), Some(grFlatzone_80217DA4), 0),
    row(Some(grFlatzone_80217450), Some(grFlatzone_802174E4), Some(grFlatzone_802174EC), Some(grFlatzone_80217644), 0),
    row(Some(grFlatzone_80217DA8), Some(grFlatzone_80217DFC), Some(grFlatzone_80217E04), Some(grFlatzone_80217E78), 0),
    row(Some(grFlatzone_80217E7C), Some(grFlatzone_80217E80), Some(grFlatzone_80217E88), Some(grFlatzone_80217E8C), 0),
    row(Some(grFlatzone_80217E90), Some(grFlatzone_80217EE8), Some(grFlatzone_80217EF0), Some(grFlatzone_8021805C), 0),
];

/// `StageData grFz_StageData`
pub static mut grFz_StageData: StageData = StageData {
    grkind: Gr_Kind_Flatzone,
    callbacks: unsafe { &raw mut grFz_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrFz.dat".as_ptr() as *mut i8,
    on_init: Some(grFlatzone_80216E78),
    on_demo_init: Some(grFlatzone_80216E74),
    on_load: Some(grFlatzone_UnkStage0_OnLoad),
    on_start: Some(grFlatzone_UnkStage0_OnStart),
    callback4: Some(grFlatzone_80216F40),
    on_touch_line: Some(grFlatzone_802182B4),
    on_check_shadow_render: Some(grFlatzone_802182BC),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `static struct grFlatzone_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grFlatzone_YakumonoParam = null_mut();
/// `static const Vec3 grFz_803B8430`
pub static grFz_803B8430: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

/// `static s16 grFz_803E7A68[]`: the background machine's animation per (state, step) pair
/// (5 columns per row; -1 = no animation for that pair).
pub static grFz_803E7A68: [i16; 47] = [
    0x0000, 0x0001, 0x0002, -1, -1, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C,
    0x000D, 0x000E, 0x000F, 0x0010, 0x0011, 0x0012, -1, -1, -1, -1, 0x0013, -1, -1, -1, -1, 0x0014, -1, -1, -1, -1,
    0x0015, -1, -1, -1, -1, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

/// `((s16(*)[5]) grFz_803E7A68)[row][col]`
#[inline]
unsafe fn table(row: usize, col: usize) -> i16 {
    *grFz_803E7A68.as_ptr().add(row * 5 + col)
}

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
        panic!("grflatzone.c: HSD_ASSERT {what}");
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

/// `HSD_JObjSetTranslateZ` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetTranslateZ(jobj: *mut HSD_JObj, z: f32) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).translate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjSetTranslate` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjSetTranslate(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!translate.is_null(), "(translate)");
    (*jobj).translate = *translate;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `HSD_JObjGetTranslation` (baselib/jobj.h, static inline)
#[inline]
unsafe fn HSD_JObjGetTranslation(jobj: *mut HSD_JObj, translate: *mut Vec3) {
    hsd_assert(!jobj.is_null(), "(jobj)");
    hsd_assert(!translate.is_null(), "(translate)");
    *translate = (*jobj).translate;
}

/// `rand_int` (static inline): a random value between `a` and `b` (exclusive of the larger end),
/// the arithmetic wrapping like the retargeted build.
#[inline]
pub unsafe fn rand_int(mut a: i32, b: i32) -> i32 {
    if a > b {
        let c = a.wrapping_sub(b);
        a = b.wrapping_add(if c != 0 { ext::HSD_Randi(c) } else { 0 });
    } else if a < b {
        let c = b.wrapping_sub(a);
        a = a.wrapping_add(if c != 0 { ext::HSD_Randi(c) } else { 0 });
    }
    a
}

/// `rand_int_inner` (static inline)
#[inline]
pub unsafe fn rand_int_inner(a: i32) -> i32 {
    if a != 0 {
        ext::HSD_Randi(a)
    } else {
        0
    }
}

/// `SIGN_RANDOM()`
#[inline]
unsafe fn sign_random() -> f32 {
    if ext::HSD_Randi(2) != 0 {
        1.0
    } else {
        -1.0
    }
}

/// 216E74
pub unsafe extern "C" fn grFlatzone_80216E74(_arg0: i32) {}

/// 216E78
pub unsafe extern "C" fn grFlatzone_80216E78() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grFlatzone_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grFlatzone_80216F48(0);
    grFlatzone_80216F48(1);
    grFlatzone_80216F48(2);
    grFlatzone_80216F48(3);
    grFlatzone_80216F48(4);
    grFlatzone_80216F48(5);
    grFlatzone_80216F48(6);
    grFlatzone_80216F48(8);
    grFlatzone_80218060(1);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Camera_800311DC(1.0);
    ext::Camera_800311CC(1000.0);
    ext::Ground_801C3990(ext::Stage_GetBlastZoneBottomOffset() - 50.0);
}

/// grFlatzone_UnkStage0_OnLoad
pub unsafe extern "C" fn grFlatzone_UnkStage0_OnLoad() {}

/// grFlatzone_UnkStage0_OnStart
pub unsafe extern "C" fn grFlatzone_UnkStage0_OnStart() {}

/// `grFlatzone_80216F48_inline` (static inline)
#[inline]
pub unsafe fn grFlatzone_80216F48_inline(arg0: i32) -> *mut HSD_GObj {
    ext::Ground_GetStageGObj(arg0)
}

/// 216F48: sets up stage gobj `gobj_id` with its row of the callback table.
pub unsafe extern "C" fn grFlatzone_80216F48(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = &raw mut grFz_StageCallbacks[gobj_id as usize];
    let gobj = grFlatzone_80216F48_inline(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    // else OSReport("%s:%d: couldn t get gobj(id=%d)\n", "grflatzone.c", 0xE8, ...): diagnostics only
    gobj
}

/// stageGObj0_OnInit
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    Ground_StartMapAnim(gobj);
}

/// 217064
pub unsafe extern "C" fn grFlatzone_80217064(_gobj: *mut HSD_GObj) {}

/// 217068
pub unsafe extern "C" fn grFlatzone_80217068(_gobj: *mut HSD_GObj) {}

/// 21706C
pub unsafe extern "C" fn fn_8021706C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.flatzone.xC4 = 0;
}

/// 21707C: gobj 2 (the building swap timer).
pub unsafe extern "C" fn grFlatzone_8021707C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    HSD_JObjSetTranslateZ(jobj, 1.0);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.flatzone.xC4 = 1;
    (*gp).u.flatzone.xC7 = 1;
    (*gp).u.flatzone.xC6 = 0;
    (*gp).u.flatzone.xC8 = rand_int((*yakumono_param).unk4, (*yakumono_param).unk0) as i16;
    ext::Ground_801C10B8(gobj, Some(fn_8021706C));
}

/// `grFlatzone_802171D4_inline`
#[inline]
pub unsafe fn grFlatzone_802171D4_inline(gobj: *mut HSD_GObj) {
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            (*gp).u.flatzone.xC4 = 1;
        }
    }
}

/// 2171D4
pub unsafe extern "C" fn grFlatzone_802171D4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.flatzone.xC4 == 0 && (*gp).u.flatzone.xC7 != 0 {
        if (*gp).u.flatzone.xC8 != 0 {
            (*gp).u.flatzone.xC8 = (*gp).u.flatzone.xC8.wrapping_sub(1);
            if (*gp).u.flatzone.xC8 == 0 {
                (*gp).u.flatzone.xC7 = 0;
                (*gp).u.flatzone.xC6 = (*gp).u.flatzone.xC5;
                loop {
                    let vals: [i16; 2] = [3, 4];
                    (*gp).u.flatzone.xC5 = vals[ext::HSD_Randi(2) as usize] as u8;
                    if (*gp).u.flatzone.xC6 != (*gp).u.flatzone.xC5 {
                        break;
                    }
                }
                match (*gp).u.flatzone.xC5 {
                    1 => {
                        grFlatzone_802171D4_inline(ext::Ground_GetMapGObj(4));
                        grFlatzone_80218260();
                    }
                    2 => {
                        grFlatzone_802171D4_inline(ext::Ground_GetMapGObj(6));
                        grFlatzone_80218260();
                    }
                    3 => {
                        grFlatzone_802171D4_inline(ext::Ground_GetMapGObj(8));
                        grFlatzone_802181B4();
                    }
                    4 => {
                        grFlatzone_802171D4_inline(ext::Ground_GetMapGObj(3));
                        grFlatzone_802181B4();
                    }
                    _ => {}
                }
            }
        } else {
            grFlatzone_80218060(0);
            (*gp).u.flatzone.xC8 = rand_int((*yakumono_param).unk4, (*yakumono_param).unk0) as i16;
        }
    }
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// 2173D0
pub unsafe extern "C" fn grFlatzone_802173D0(_gobj: *mut HSD_GObj) {}

/// 2173D4
pub unsafe extern "C" fn grFlatzone_802173D4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grMaterial_801C8858(jobj, 0x28000000);
    (*gp).x11_flags.set_b012(2);
}

/// 217438
pub unsafe extern "C" fn grFlatzone_80217438(_gobj: *mut HSD_GObj) {}

/// 21743C
pub unsafe extern "C" fn grFlatzone_8021743C(_gobj: *mut HSD_GObj) {}

/// 217440
pub unsafe extern "C" fn fn_80217440(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.flatzone.xC4 = 0;
}

/// 217450: gobj 5 (the background machine) init.
pub unsafe extern "C" fn grFlatzone_80217450(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.flatzone3.xC7 = 8;
    (*gp).u.flatzone3.xC8 = 8;
    (*gp).u.flatzone3.xC9 = 0;
    (*gp).u.flatzone3.xCA = 0;
    (*gp).u.flatzone.xCC = 0;
    (*gp).u.flatzone.xC6 = 0xFF;
    (*gp).u.flatzone.xC4 = 1;
    ext::Ground_801C10B8(gobj, Some(fn_80217440));
}

/// 2174EC: gobj 5 per frame: picks the next animation of the background machine.
pub unsafe extern "C" fn grFlatzone_802174EC(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.flatzone3.xC4 == 0 && (*gp).u.flatzone3.xC7 != 8 && (*gp).u.flatzone3.xC9 != 0 {
        let cc = (*gp).u.flatzone.xCC;
        (*gp).u.flatzone.xCC = cc.wrapping_sub(1);
        if cc < 0 {
            (*gp).u.flatzone3.xCA = (*gp).u.flatzone3.xC9;
            let row_entry: i16;
            loop {
                let mut next_anim = ext::HSD_Randi(4);
                next_anim += 1;
                (*gp).u.flatzone3.xC9 = next_anim as u8;
                if (*gp).u.flatzone3.xCA == (*gp).u.flatzone3.xC9 {
                    continue;
                }
                let e = table((*gp).u.flatzone3.xC7 as usize, (*gp).u.flatzone3.xC9 as usize);
                if e != -1 {
                    row_entry = e;
                    break;
                }
            }
            (*gp).u.flatzone3.xC5 = row_entry as u8;
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*gp).u.flatzone3.xC5 as i32);
            (*gp).u.flatzone.xCC = rand_int((*yakumono_param).unkC, (*yakumono_param).unk8) as i16;
        }
    }
    ext::Ground_UpdateMapColl(gobj);
    if (*gp).u.flatzone3.xC6 != (*gp).u.flatzone3.xC5 {
        (*gp).u.flatzone3.xC6 = (*gp).u.flatzone3.xC5;
        ext::mpLib_80058560();
    }
}

/// 217644
pub unsafe extern "C" fn grFlatzone_80217644(_gobj: *mut HSD_GObj) {}

/// 217648: gobj 3 (the crane) init.
pub unsafe extern "C" fn grFlatzone_80217648(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.flatzone.xC4 = 0;
    (*gp).u.flatzone2.xC8 = 1.0;
    (*gp).u.flatzone2.xCC = null_mut();
    (*gp).u.flatzone2.xD0 = -1;
    (*gp).u.flatzone2.timer = 0;
}

/// `Ground_GetMapGObj(2)` followed by `grAnime_801C8138(gobj, gp->map_id, anim)` (inlined 3x)
#[inline]
unsafe fn map2_anim(anim: i32) {
    let gobj = ext::Ground_GetMapGObj(2);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, anim);
        }
    }
}

/// 2176BC: gobj 3 per frame: the crane's drop sequence (see the file comment for the aliasing).
pub unsafe extern "C" fn grFlatzone_802176BC(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut trigger_machine = 0;
    let mut pos: Vec3 = grFz_803B8430;
    if (*gp).u.flatzone.xC4 != 0 {
        if (*gp).u.flatzone2.xD0 == 3 {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            if (*gp).u.flatzone2.xCC.is_null() {
                ext::grDynamicAttr_801CA1C0((*gp).u.flatzone2.xCC);
                (*gp).u.flatzone2.xCC = null_mut();
            }
        }
        (*gp).u.flatzone.xC4 = 0;
        (*gp).u.flatzone2.xC8 = sign_random();
        (*gp).u.flatzone2.xD0 = 0;
        (*gp).u.flatzone2.timer = (*yakumono_param).unk20;
        if (*gp).u.flatzone2.xC8 == 1.0 {
            pos.x = (*yakumono_param).unk24;
            (*gp).u.flatzone2.timer = ((*gp).u.flatzone2.timer as f32 / 2.0) as i32;
        } else {
            pos.x = (*yakumono_param).unk28;
            map2_anim(1);
        }
        HSD_JObjSetTranslate(jobj, &mut pos);
    }
    match (*gp).u.flatzone2.xD0 {
        0 => {
            if (*gp).u.flatzone2.timer == (*yakumono_param).unk20 / 2 {
                if (*gp).u.flatzone2.xC8 == 1.0 {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                } else {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 2);
                }
                ext::HSD_JObjRemoveAnimAll(jobj);
            }
            (*gp).u.flatzone2.timer = (*gp).u.flatzone2.timer.wrapping_sub(1);
            if (*gp).u.flatzone2.timer <= 0 {
                (*gp).u.flatzone2.xD0 = 1;
                let inner = rand_int_inner((*yakumono_param).unk34.wrapping_sub((*yakumono_param).unk30));
                (*gp).u.flatzone2.timer = (*yakumono_param).unk2C.wrapping_mul((*yakumono_param).unk30.wrapping_add(inner));
                if (*gp).u.flatzone2.xC8 == 1.0 {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                } else {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 2);
                }
            }
        }
        1 => {
            if srem((*gp).u.flatzone2.timer, (*yakumono_param).unk2C) == 1 {
                HSD_JObjGetTranslation(jobj, &mut pos);
                pos.x = fmadds((*yakumono_param).unk38, (*gp).u.flatzone2.xC8, pos.x);
                HSD_JObjSetTranslate(jobj, &mut pos);
            }
            (*gp).u.flatzone2.timer = (*gp).u.flatzone2.timer.wrapping_sub(1);
            if (*gp).u.flatzone2.timer <= 0 {
                (*gp).u.flatzone2.xD0 = 2;
                (*gp).u.flatzone2.timer = 0;
                if (*gp).u.flatzone2.xC8 == 1.0 {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 1);
                } else {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 3);
                }
            }
        }
        2 => {
            if ext::grAnime_801C83D0(gobj, 0, 1) != 0 {
                let mut pos_2: Vec3 = core::mem::zeroed();
                let mut ret: i32 = 0;
                (*gp).u.flatzone2.xD0 = 3;
                (*gp).u.flatzone2.timer = (*yakumono_param).unk3C;
                HSD_JObjGetTranslation(jobj, &mut pos);
                pos.x = fmadds(36.0, (*gp).u.flatzone2.xC8, pos.x);
                pos.y -= 27.0;
                let other_x = pos.x;
                let other_z = 5.0 + pos.y;
                pos.y -= 5.0;
                let line_id = if ext::mpCheckMultiple(other_x, other_z, pos.x, pos.y, &mut pos_2, &mut ret, null_mut(), null_mut(), 1, -1, 0x32) != 0 {
                    ret
                } else {
                    -1
                };
                let checked_line_id = line_id;
                if line_id != -1 {
                    (*gp).u.flatzone2.xCC =
                        ext::grDynamicAttr_801CA0F8(0x11, &mut pos_2, checked_line_id, 22.0, (*yakumono_param).unk3C);
                }
                trigger_machine = 1;
            }
        }
        3 => {
            (*gp).u.flatzone2.timer = (*gp).u.flatzone2.timer.wrapping_sub(1);
            if (*gp).u.flatzone2.timer <= 1 {
                (*gp).u.flatzone2.xD0 = 4;
                (*gp).u.flatzone2.timer = 0;
            }
        }
        4 => {
            ext::HSD_JObjRemoveAnimAll(jobj);
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            (*gp).u.flatzone2.xCC = null_mut();
            (*gp).u.flatzone2.xD0 = -1;
        }
        _ => {}
    }
    if trigger_machine != 0 {
        {
            let gobj2 = ext::Ground_GetMapGObj(2);
            if !gobj2.is_null() {
                let gp2 = (*gobj2).user_data as *mut Ground;
                if !gp2.is_null() {
                    (*gp2).u.flatzone.xC7 = 1;
                }
            }
        }
        if (*gp).u.flatzone2.xC8 != 1.0 {
            map2_anim(0);
        }
    }
    ext::Ground_UpdateMapColl(gobj);
}

/// 217CD0
pub unsafe extern "C" fn grFlatzone_80217CD0(_gobj: *mut HSD_GObj) {}

/// 217CD4
pub unsafe extern "C" fn grFlatzone_80217CD4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.flatzone.xC4 = 0;
}

/// 217D30
pub unsafe extern "C" fn grFlatzone_80217D30(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.flatzone.xC4 != 0 {
        {
            let gobj2 = ext::Ground_GetMapGObj(2);
            if !gobj2.is_null() {
                let gp2 = (*gobj2).user_data as *mut Ground;
                if !gp2.is_null() {
                    (*gp2).u.flatzone.xC7 = 1;
                }
            }
        }
        (*gp).u.flatzone.xC4 = 0;
    }
    ext::Ground_UpdateMapColl(gobj);
}

/// 217DA4
pub unsafe extern "C" fn grFlatzone_80217DA4(_gobj: *mut HSD_GObj) {}

/// 217DA8
pub unsafe extern "C" fn grFlatzone_80217DA8(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.flatzone.xC4 = 0;
}

/// 217E04
pub unsafe extern "C" fn grFlatzone_80217E04(gobj: *mut HSD_GObj) {
    grFlatzone_80217D30(gobj);
}

/// 217E78
pub unsafe extern "C" fn grFlatzone_80217E78(_gobj: *mut HSD_GObj) {}

/// 217E7C
pub unsafe extern "C" fn grFlatzone_80217E7C(_gobj: *mut HSD_GObj) {}

/// 217E88
pub unsafe extern "C" fn grFlatzone_80217E88(_gobj: *mut HSD_GObj) {}

/// 217E8C
pub unsafe extern "C" fn grFlatzone_80217E8C(_gobj: *mut HSD_GObj) {}

/// 217E90: gobj 8 (the item rain) init.
pub unsafe extern "C" fn grFlatzone_80217E90(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.flatzone.xC4 = 0;
    *(&raw mut (*gp).u.flatzone.xC8 as *mut i32) = 0;
}

/// 217EF0: gobj 8 per frame: drops an item every `unk14` frames while the counter runs.
pub unsafe extern "C" fn grFlatzone_80217EF0(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.flatzone3.xC4 != 0 {
        (*gp).u.unk.xC8 = (*yakumono_param).unk10;
        (*gp).u.flatzone3.xC4 = 0;
    }
    if (*gp).u.unk.xC8 > 0 {
        if srem((*gp).u.unk.xC8, (*yakumono_param).unk14) == 0 {
            let mut pos: Vec3 = core::mem::zeroed();
            {
                let mut spawn_x = ext::HSD_Randf();
                let spawn_left = (*yakumono_param).unk1C;
                spawn_x *= (*yakumono_param).unk18.wrapping_sub(spawn_left) as f32;
                spawn_x -= (spawn_left.wrapping_neg()) as f32;
                pos.x = spawn_x;
            }
            pos.y = ext::Stage_GetBlastZoneTopOffset() - 5.0;
            pos.z = 0.0;
            let item_kind = ext::HSD_Randi(5);
            let sign = sign_random();
            let item_gobj = ext::it_802EEFA8(&mut pos, item_kind, sign);
            if !item_gobj.is_null() && !(*item_gobj).user_data.is_null() {
                (*((*item_gobj).user_data as *mut Item)).x378_itemColl.joint_id_only = 0x32;
            }
        }
        (*gp).u.unk.xC8 -= 1;
        if (*gp).u.unk.xC8 <= 0 {
            let gobj2 = ext::Ground_GetMapGObj(2);
            if !gobj2.is_null() {
                let gp2 = (*gobj2).user_data as *mut Ground;
                if !gp2.is_null() {
                    (*gp2).u.flatzone.xC7 = 1;
                }
            }
        }
    }
}

/// 21805C
pub unsafe extern "C" fn grFlatzone_8021805C(_gobj: *mut HSD_GObj) {}

/// 218060: picks the background machine's next (state, step) and starts its animation.
pub unsafe extern "C" fn grFlatzone_80218060(arg0: i32) {
    let gobj = ext::Ground_GetMapGObj(5);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            if arg0 != 0 {
                (*gp).u.flatzone.xC7 = 2;
                (*gp).u.flatzone3.xC9 = 1;
            } else {
                let mut v: u8;
                loop {
                    loop {
                        (*gp).u.flatzone.xC7 = ext::HSD_Randi(8) as u8;
                        v = (*gp).u.flatzone.xC7;
                        if v == (*gp).u.flatzone3.xC8 {
                            continue;
                        }
                        break;
                    }
                    if table(v as usize, 1) != -1 {
                        break;
                    }
                }
                loop {
                    let randi = ext::HSD_Randi(4);
                    v = (randi + 1) as u8;
                    (*gp).u.flatzone3.xC9 = v;
                    if table((*gp).u.flatzone.xC7 as usize, v as usize) != -1 {
                        break;
                    }
                }
            }
            (*gp).u.flatzone3.xCA = 0;
            (*gp).u.flatzone.xCC = rand_int((*yakumono_param).unkC, (*yakumono_param).unk8) as i16;
            (*gp).u.flatzone.xC5 = table((*gp).u.flatzone.xC7 as usize, (*gp).u.flatzone3.xC9 as usize) as u8;
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*gp).u.flatzone.xC5 as i32);
        }
    }
}

/// `grFlatzone_802181B4_inline` (static inline)
#[inline]
pub unsafe fn grFlatzone_802181B4_inline() {
    let gobj = ext::Ground_GetMapGObj(5);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            loop {
                (*gp).u.flatzone.xC7 = ext::HSD_Randi(8) as u8;
                if (*gp).u.flatzone.xC7 != (*gp).u.flatzone3.xC8 {
                    break;
                }
            }
            (*gp).u.flatzone3.xC9 = 0;
            (*gp).u.flatzone3.xCA = 0;
            (*gp).u.flatzone.xCC = 0;
            (*gp).u.flatzone.xC5 = table((*gp).u.flatzone.xC7 as usize, (*gp).u.flatzone3.xC9 as usize) as u8;
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*gp).u.flatzone.xC5 as i32);
        }
    }
}

/// 2181B4
pub unsafe extern "C" fn grFlatzone_802181B4() {
    grFlatzone_802181B4_inline();
}

/// 218260
pub unsafe extern "C" fn grFlatzone_80218260() {
    let gobj = ext::Ground_GetMapGObj(5);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() {
            (*gp).u.flatzone.xC7 = 8;
            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            if !jobj.is_null() {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
        }
    }
}
