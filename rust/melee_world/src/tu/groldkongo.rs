//! Port of src/melee/gr/groldkongo.c (Kongo Jungle 1.0 "old": the rotating barrel cannon, the bird that
//! flies across, and the stage tables). Every function of the TU, native: pointers into the game's
//! structs, callees of other TUs through `ext` hooks. Checked against the C (whole TU, shipping
//! contraction, stateful oracle) by tests/tu_groldkongo.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py groldkongo`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use melee_types::Ground as Ground_leaves;
#[allow(unused_imports)]
use melee_types::HSD_GObj as HSD_GObj_leaves;
#[allow(unused_imports)]
use crate::gr::leaves::{grkongo_801d8270, groldkongo_802105ac_gp};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::{f2i16, f2i32};
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
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
    fn Camera_LogicBounds(a0: *mut f32, a1: *mut f32, a2: *mut f32) -> i32;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4D70(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32) -> i32 = |a0: *mut HSD_GObj, a1: *mut Vec, a2: f32| unsafe { crate::tu::ground::Ground_801C4D70(a0 as _, a1 as _, a2 as _) as _ };
    fn Ground_801C4DA0(a0: *mut Vec, a1: *mut f32) -> i32 = |a0: *mut Vec, a1: *mut f32| unsafe { crate::tu::ground::Ground_801C4DA0(a0 as _, a1 as _) as _ };
    fn Ground_801C5440(a0: *mut Ground, a1: i32, a2: u32) = |a0: *mut Ground, a1: i32, a2: u32| { unsafe { crate::tu::ground::Ground_801C5440(a0 as _, a1 as _, a2 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn ftCo_8009EC70(a0: *mut HSD_GObj, a1: *mut Vec, a2: *mut c_void, a3: f32);
    fn ftCo_800C0764(a0: *mut HSD_GObj, a1: u32, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u32, a2: *mut c_void| { unsafe { crate::ftdevice::ftCo_800C0764(a0 as _, a1 as _, core::mem::transmute(a2)); } };
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_GetPos(a0 as _, a1 as _); } };
    fn ftLib_StartRumble(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grMaterial_801C94D8(a0: *mut c_void) = |a0: *mut c_void| { unsafe { crate::tu::grmaterial::grMaterial_801C94D8(a0 as _); } };
    fn grMaterial_801C95C4(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C95C4(a0 as _); } };
    fn grMaterial_801C9604(a0: *mut HSD_GObj, a1: *mut c_void, a2: i32) = |a0: *mut HSD_GObj, a1: *mut c_void, a2: i32| unsafe { crate::tu::grmaterial::grMaterial_801C9604(a0 as _, a1 as _, a2 != 0) };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

const M_TAU: f64 = 6.283185307179586;
/// `MTXDegToRad(a)`: `a * 0.01745329252f` (f32 product)
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// gr/inlines.h `Ground_StartMapAnim`.
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// gr/inlines.h `Ground_InitMapCollAndAnim`.
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// gr/inlines.h `Ground_SetupStageCallbacks` (TARGET_PC path: callbacks go through `Mex_GrBind`).
unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).callback3)));
    }
    if (*callbacks).on_init.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).on_init)));
        (f.unwrap())(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).gobj_proc)));
        ext::HSD_GObj_SetupProc(gobj, f, 4);
    }
}

/// gr/inlines.h `rand_range`.
unsafe fn rand_range(a: i32, b: i32) -> i32 {
    if a > b {
        let n = a.wrapping_sub(b);
        b.wrapping_add(if n != 0 { ext::HSD_Randi(n) } else { 0 })
    } else if a < b {
        let n = b.wrapping_sub(a);
        a.wrapping_add(if n != 0 { ext::HSD_Randi(n) } else { 0 })
    } else {
        a
    }
}

/// gr/inlines.h `Ground_ClampSymmetric`.
unsafe fn Ground_ClampSymmetric(value: f32, mut limit: f32, out: *mut f32) {
    if value > limit {
        *out = limit;
    } else {
        limit = -limit;
        if value < limit {
            *out = limit;
        }
    }
}

/// gr/inlines.h `Ground_WrapAngle` (the `M_TAU` double literal: double compare and arithmetic).
unsafe fn Ground_WrapAngle(angle: *mut f32) {
    if (*angle as f64) > M_TAU {
        *angle = ((*angle as f64) - M_TAU) as f32;
    } else if (*angle as f64) < -M_TAU {
        *angle = ((*angle as f64) + M_TAU) as f32;
    }
}

/// `struct grOldKongo_YakumonoParam` (file-local in the C; field names from SSBU param files).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOldKongo_YakumonoParam {
    pub rframe_bird_wait_a: i16,
    pub rframe_bird_wait_b: i16,
    pub rrange_bird_random_offset_y: f32,
    pub rframe_barrel_shoot_a: f32,
    pub rframe_barrel_shoot_b: f32,
    pub rframe_barrel_in: f32,
    pub rframe_barrel_wait_a: f32,
    pub rframe_barrel_wait_b: f32,
    pub rspeed_barrel_rot_accel: f32,
    pub rspeed_barrel_rot_max: f32,
    pub rframe_barrel_roll_a: f32,
    pub rframe_barrel_roll_b: f32,
    pub rrate_barrel_ld: i16,
    pub rrate_barrel_l: i16,
    pub rrate_barrel_lu: i16,
    pub rrate_barrel_u: i16,
    pub rrate_barrel_ru: i16,
    pub rrate_barrel_r: i16,
    pub rrate_barrel_rd: i16,
    pub rrate_barrel_d: i16,
    pub rframe_barrel_interval_a: i32,
    pub rframe_barrel_interval_b: i32,
    pub rspeed_barrel_move_accel: f32,
    pub rspeed_barrel_move_max: f32,
    pub rframe_barrel_stop_a: i32,
    pub rframe_barrel_stop_b: i32,
    pub rpower_barrel_attack: i32,
    pub rvector_barrel_attack: i32,
    pub rreff_barrel_attack: i32,
    pub rrfix_barrel_attack: i32,
    pub rradd_barrel_attack: i32,
    pub x68: i32,
    pub x6C: *mut c_void,
}
melee_core::gc_layout!(grOldKongo_YakumonoParam, "grOldKongo_YakumonoParam", size 0x70, align 4, [
    rframe_bird_wait_a @ 0x0: 2 "s16", rframe_bird_wait_b @ 0x2: 2 "s16",
    rrange_bird_random_offset_y @ 0x4: 4 "f32", rframe_barrel_shoot_a @ 0x8: 4 "f32",
    rframe_barrel_shoot_b @ 0xC: 4 "f32", rframe_barrel_in @ 0x10: 4 "f32",
    rframe_barrel_wait_a @ 0x14: 4 "f32", rframe_barrel_wait_b @ 0x18: 4 "f32",
    rspeed_barrel_rot_accel @ 0x1C: 4 "f32", rspeed_barrel_rot_max @ 0x20: 4 "f32",
    rframe_barrel_roll_a @ 0x24: 4 "f32", rframe_barrel_roll_b @ 0x28: 4 "f32",
    rrate_barrel_ld @ 0x2C: 2 "s16", rrate_barrel_l @ 0x2E: 2 "s16", rrate_barrel_lu @ 0x30: 2 "s16",
    rrate_barrel_u @ 0x32: 2 "s16", rrate_barrel_ru @ 0x34: 2 "s16", rrate_barrel_r @ 0x36: 2 "s16",
    rrate_barrel_rd @ 0x38: 2 "s16", rrate_barrel_d @ 0x3A: 2 "s16",
    rframe_barrel_interval_a @ 0x3C: 4 "s32", rframe_barrel_interval_b @ 0x40: 4 "s32",
    rspeed_barrel_move_accel @ 0x44: 4 "f32", rspeed_barrel_move_max @ 0x48: 4 "f32",
    rframe_barrel_stop_a @ 0x4C: 4 "s32", rframe_barrel_stop_b @ 0x50: 4 "s32",
    rpower_barrel_attack @ 0x54: 4 "s32", rvector_barrel_attack @ 0x58: 4 "s32",
    rreff_barrel_attack @ 0x5C: 4 "s32", rrfix_barrel_attack @ 0x60: 4 "s32",
    rradd_barrel_attack @ 0x64: 4 "s32", x68 @ 0x68: 4 "s32", x6C @ 0x6C: 4 "void *",
]);

/// `static struct grOldKongo_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grOldKongo_YakumonoParam = null_mut();

/// `grOk_803E6580` (3E6580)
pub static mut grOk_803E6580: [GrJoint; 2] = [GrJoint { x: 0, y: 3, z: 1 }, GrJoint { x: 1, y: 3, z: 2 }];

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, x10: StageCallbacks_x10 { flags: u32::to_be(flags) } }
}

/// `stage_callbacks` (static in the C)
pub static mut stage_callbacks: [StageCallbacks; 4] = [
    cb(Some(stageGObj0_OnInit), Some(stageGObj0_Callback1), Some(stageGObj0_GObjProc), Some(stageGObj0_Callback3), 0),
    cb(Some(stageGObj1_OnInit), Some(stageGObj1_Callback1), Some(stageGObj1_GObjProc), Some(stageGObj1_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj3_OnInit), Some(stageGObj3_Callback1), Some(stageGObj3_GObjProc), Some(stageGObj3_Callback3), (1 << 30) | (1 << 31)),
];

/// `grOk_StageData` (3E65A0)
pub static mut grOk_StageData: StageData = StageData {
    grkind: Gr_Kind_OldKongo as _,
    callbacks: unsafe { &raw mut stage_callbacks as *mut StageCallbacks },
    data1: b"/GrOk.dat\0".as_ptr() as *mut i8,
    on_init: Some(grOldKongo_8020F46C),
    on_demo_init: Some(grOldKongo_8020F468),
    on_load: Some(grOldKongo_8020F4E4),
    on_start: Some(grOldKongo_8020F4E8),
    callback4: Some(grOldKongo_8020F524),
    on_touch_line: Some(grOldKongo_80210780),
    on_check_shadow_render: Some(grOldKongo_80210788),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grOk_803E6580 as *mut GrJoint },
    joint_count: 2,
};

pub unsafe extern "C" fn grOldKongo_8020F468(_arg: i32) { melee_core::ct!("grOldKongo_8020F468");}

pub unsafe extern "C" fn grOldKongo_8020F46C() {
    melee_core::ct!("grOldKongo_8020F46C");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grOldKongo_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    setupStageCallbacks(0);
    setupStageCallbacks(3);
    setupStageCallbacks(1);
    setupStageCallbacks(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grOldKongo_8020F4E4() { melee_core::ct!("grOldKongo_8020F4E4");}

pub unsafe extern "C" fn grOldKongo_8020F4E8() {
    melee_core::ct!("grOldKongo_8020F4E8");
    let gobj = ext::Ground_GetMapGObj(1);
    ext::ftCo_800C0764(gobj, 2, grOldKongo_80210454 as usize as *mut c_void);
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grOldKongo_8020F524() -> i32 {
    melee_core::ct!("grOldKongo_8020F524");
    0
}

pub unsafe extern "C" fn setupStageCallbacks(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("setupStageCallbacks");
    let callbacks = (&raw mut stage_callbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn stageGObj0_Callback1(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj0_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj0_GObjProc(_arg: *mut HSD_GObj) { melee_core::ct!("stageGObj0_GObjProc");}

pub unsafe extern "C" fn stageGObj0_Callback3(_arg: *mut HSD_GObj) { melee_core::ct!("stageGObj0_Callback3");}

pub unsafe extern "C" fn stageGObj3_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj3_OnInit");
    let gp = (*gobj).user_data as *mut Ground;
    Ground_InitMapCollAndAnim(gobj);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn stageGObj3_Callback1(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj3_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj3_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj3_GObjProc");
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn stageGObj3_Callback3(_arg: *mut HSD_GObj) { melee_core::ct!("stageGObj3_Callback3");}

pub unsafe extern "C" fn stageGObj1_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj1_OnInit");
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let yp = yakumono_param;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
    (*gp).u.taru.xE8 = 1.0;
    (*gp).u.taru.xEC = 1.0;
    (*gp).u.taru.xD8 = 0.0;
    (*gp).u.taru.xDC = 0.0;
    (*gp).u.taru.xC4 = 0;
    (*gp).u.taru.xC6 = 0;
    (*gp).u.taru.xC8 = 2;
    (*gp).u.taru.keep = null_mut();
    (*gp).u.taru.xE0 = 0.0;
    (*gp).u.taru.xE4 = 0.0;
    (*gp).u.taru.xEC = (*yp).rspeed_barrel_move_max;
    (*gp).u.taru.xCC = rand_range(f2i32((*yp).rframe_barrel_wait_b as f64), f2i32((*yp).rframe_barrel_wait_a as f64)) as i16;
    (*gp).u.taru.xCE = rand_range((*yp).rframe_barrel_stop_b, (*yp).rframe_barrel_stop_a) as i16;
    ext::lb_8000B1CC(ext::Ground_801C3FA4(gobj, 1), null_mut(), &mut sp14);
    ext::Ground_801C4D70(gobj, &mut sp14, (*gp).u.taru.xDC);
}

pub unsafe extern "C" fn stageGObj1_Callback1(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj1_Callback1");
    0
}

pub unsafe fn grOldKongo_8020F888_inline(gp: *mut Ground) {
    melee_core::ct!("grOldKongo_8020F888_inline");
    let vel = (*gp).u.taru.xE4;
    let yp = yakumono_param;
    if vel > 0.0 {
        if vel < deg_to_rad((*yp).rspeed_barrel_rot_accel) {
            (*gp).u.taru.xE4 = 0.0;
            (*gp).u.taru.xDC = (*gp).u.taru.xD8;
        } else {
            // vel - a * k: fused as fmuladd(-a, k, vel)
            (*gp).u.taru.xE4 = fmadds(-(*yp).rspeed_barrel_rot_accel, 0.01745329252f32, vel);
        }
    } else if vel < 0.0 {
        if vel > -deg_to_rad((*yp).rspeed_barrel_rot_accel) {
            (*gp).u.taru.xE4 = 0.0;
            (*gp).u.taru.xDC = (*gp).u.taru.xD8;
        } else {
            (*gp).u.taru.xE4 = fmadds((*yp).rspeed_barrel_rot_accel, 0.01745329252f32, vel);
        }
    }
}

#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn stageGObj1_GObjProc(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj1_GObjProc");
    let mut sp3C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let yp = yakumono_param;
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = ext::Ground_801C3FA4(gobj, 1);
    let t = &mut (*gp).u.taru;
    match t.xC4 {
        2 | 3 => {
            let angle_limit = 0.5f32 * (t.xE4 * (t.xE4 / deg_to_rad((*yp).rspeed_barrel_rot_accel)));
            let mut angle_delta: f32 = 0.0;
            if t.xE4 > 0.0 {
                angle_delta = t.xD8 - t.xDC;
            } else if t.xE4 < 0.0 {
                angle_delta = t.xDC - t.xD8;
            } else {
                panic!("HSD_ASSERT(0x18C, 0) groldkongo.c");
            }
            if angle_delta < 0.0 {
                angle_delta = ((angle_delta as f64) + M_TAU) as f32;
            }
            let abs_e4 = if t.xE4 < 0.0 { -t.xE4 } else { t.xE4 };
            if angle_delta < angle_limit || angle_delta < abs_e4 {
                if t.xC4 == 3 {
                    t.xC4 = 0;
                }
            } else if t.xC4 == 2 {
                t.xC4 = 3;
            }
        }
        0 => {
            grOldKongo_8020F888_inline(gp);
            t.xCC = t.xCC.wrapping_sub(1);
            if t.xCC < 0 {
                t.xC4 = 1;
                let x_speed = if ext::HSD_Randi(2) != 0 {
                    deg_to_rad((*yp).rspeed_barrel_rot_accel)
                } else {
                    -deg_to_rad((*yp).rspeed_barrel_rot_accel)
                };
                t.xE0 = x_speed;
                t.xCC = rand_range(f2i32((*yp).rframe_barrel_roll_b as f64), f2i32((*yp).rframe_barrel_roll_a as f64)) as i16;
            }
        }
        1 => {
            t.xE4 += t.xE0;
            Ground_ClampSymmetric(t.xE4, deg_to_rad((*yp).rspeed_barrel_rot_max), &mut t.xE4);
            let old = t.xCC;
            t.xCC = old.wrapping_sub(1);
            if old < 0 {
                t.xC4 = 2;
                t.xCC = rand_range(f2i32((*yp).rframe_barrel_wait_b as f64), f2i32((*yp).rframe_barrel_wait_a as f64)) as i16;
                t.xD8 = grOldKongo_80210650();
            }
        }
        _ => {}
    }

    t.xDC += t.xE4;
    Ground_WrapAngle(&mut t.xDC);
    HSD_JObjSetRotationZ(&mut *jobj, t.xDC, &mut Dirty);
    ext::lb_8000B1CC(jobj, null_mut(), &mut sp3C);
    ext::Ground_801C4D70(gobj, &mut sp3C, t.xDC);

    match t.xC8 {
        0 => {
            let old = t.xCE;
            t.xCE = old.wrapping_sub(1);
            if old < 0 {
                t.xC8 = 1;
            }
        }
        1 => {
            t.xEC += (*yp).rspeed_barrel_move_accel;
            let xec_max = (*yp).rspeed_barrel_move_max;
            if t.xEC > xec_max {
                t.xEC = xec_max;
                t.xCE = rand_range((*yp).rframe_barrel_stop_b, (*yp).rframe_barrel_stop_a) as i16;
                t.xC8 = 2;
            }
        }
        2 => {
            let old = t.xCE;
            t.xCE = old.wrapping_sub(1);
            if old < 0 {
                t.xC8 = 3;
            }
        }
        3 => {
            t.xEC -= (*yp).rspeed_barrel_move_accel;
            if t.xEC < 0.0 {
                t.xEC = 0.0;
                t.xCE = rand_range((*yp).rframe_barrel_interval_b, (*yp).rframe_barrel_interval_a) as i16;
                t.xC8 = 0;
            }
        }
        _ => {}
    }
    ext::grAnime_801C7A04(gobj, 0, 7, t.xEC);

    // switch (xC6): case 1 falls into case 2 falls into case 3
    let mut stage = match t.xC6 {
        1 => {
            if t.keep.is_null() {
                t.xC6 = 0;
            } else {
                let old = t.xCA;
                t.xCA = old.wrapping_sub(1);
                if old >= 0 {
                    return;
                }
                t.xC6 = 2;
            }
            2
        }
        2 => 2,
        3 => 3,
        4 => 4,
        _ => 0,
    };
    if stage == 2 {
        ext::grAnime_801C7FF8(gobj, 2, 7, 2, 0.0, 1.0);
        ext::grMaterial_801C95C4(gobj);
        t.hit_timer = 0;
        t.xC6 = 3;
        stage = 3;
    }
    if stage == 3 {
        let old = t.hit_timer;
        t.hit_timer = old.wrapping_add(1);
        if old > 0xA {
            let mut hit = lbColl_80008D30_arg1::zeroed();
            hit.state = 1;
            hit.damage = 1;
            hit.kb_angle = 361;
            hit.unkC = 0;
            hit.unk10 = 0;
            hit.unk14 = 180;
            hit.state = HitCapsule_Enabled;
            hit.damage = (*yp).rpower_barrel_attack as u32;
            hit.kb_angle = (*yp).rvector_barrel_attack;
            hit.unkC = (*yp).rreff_barrel_attack as u32;
            hit.unk10 = (*yp).rrfix_barrel_attack as u32;
            hit.unk14 = (*yp).rradd_barrel_attack as u32;
            hit.element = (*yp).x68 as u32;
            let mut hit_angle: f32 = (1.5707963267948966f64 + t.xDC as f64) as f32;
            if hit_angle < 0.0 {
                hit_angle = ((hit_angle as f64) + M_TAU) as f32;
            } else if (hit_angle as f64) > M_TAU {
                hit_angle = ((hit_angle as f64) - M_TAU) as f32;
            }
            hit_angle = 57.29578f32 * hit_angle;
            {
                let fighter_gobj = t.keep;
                if (*fighter_gobj).p_link == 8 {
                    ext::ftCo_8009EC70(fighter_gobj, &mut sp3C, &mut hit as *mut _ as *mut c_void, hit_angle);
                }
            }
            t.xC6 = 4;
            ext::Ground_801C5440(gp, 0, 0x12A);
        }
    } else if stage == 4 {
        t.xC6 = 0;
    }
}

pub unsafe extern "C" fn stageGObj1_Callback3(_arg: *mut HSD_GObj) { melee_core::ct!("stageGObj1_Callback3");}

pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let yp = yakumono_param;
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.unk.xC4 = rand_range((*yp).rframe_bird_wait_b as i32, (*yp).rframe_bird_wait_a as i32);
}

pub unsafe extern "C" fn stageGObj2_Callback1(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj2_Callback1");
    0
}

#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe extern "C" fn stageGObj2_GObjProc(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_GObjProc");
    let mut left: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut center: f32 = 0.0;
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let yp = yakumono_param;
    let timer = (*gp).u.unk.xC4;

    if timer > 0 {
        (*gp).u.unk.xC4 = timer.wrapping_sub(1);
        if (*gp).u.unk.xC4 == 0 {
            ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
            // rrange * ((2.0f * Randf) - 1.0f) + 70.0f: both sites fused
            let r = ext::HSD_Randf();
            let y = fmadds((*yp).rrange_bird_random_offset_y, fmadds(2.0, r, -1.0), 70.0);
            HSD_JObjSetTranslateY(&mut *jobj, y, &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *jobj, -200.0, &mut Dirty);
            ext::Camera_LogicBounds(&mut left, &mut center, &mut right);
            if ext::HSD_Randi(2) != 0 {
                let x = if 200.0 < right { right } else { 200.0 };
                HSD_JObjSetTranslateX(&mut *jobj, x, &mut Dirty);
                ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(arg0, 3), JOBJ_HIDDEN as u32);
                return;
            }
            let x = if -200.0 > left { left } else { -200.0 };
            HSD_JObjSetTranslateX(&mut *jobj, x, &mut Dirty);
            ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(arg0, 1), JOBJ_HIDDEN as u32);
        }
    } else if ext::grAnime_801C83D0(arg0, 0, 7) != 0 {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
        let mut min = (*yp).rframe_bird_wait_b as i32;
        let max = (*yp).rframe_bird_wait_a as i32;
        if min > max {
            let range = min.wrapping_sub(max);
            min = max.wrapping_add(if range != 0 { ext::HSD_Randi(range) } else { 0 });
        } else if min < max {
            let range = max.wrapping_sub(min);
            min = min.wrapping_add(if range != 0 { ext::HSD_Randi(range) } else { 0 });
        }
        (*gp).u.unk.xC4 = min;
    }
}

pub unsafe extern "C" fn stageGObj2_Callback3(_arg: *mut HSD_GObj) { melee_core::ct!("stageGObj2_Callback3");}

pub unsafe extern "C" fn grOldKongo_80210454(ground_gobj: *mut HSD_GObj, keep: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grOldKongo_80210454");
    let mut pos_gnd = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos_ft = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut unk: f32 = 0.0;
    let yp = yakumono_param;
    let gp = (*ground_gobj).user_data as *mut Ground;

    if (*gp).u.taru.xC6 != 0 {
        return 0;
    }

    ext::Ground_801C4DA0(&mut pos_gnd, &mut unk);
    ext::ftLib_GetPos(keep, &mut pos_ft);

    // (pos_gnd.x - pos_ft.x) * (pos_gnd.x - pos_ft.x) + ...: squares of computed expressions, not
    // fused on the console (notes/fp-audit.md)
    let dx = pos_gnd.x - pos_ft.x;
    let dy = pos_gnd.y - pos_ft.y;
    let dz = pos_gnd.z - pos_ft.z;
    let sum = dx * dx + dy * dy + dz * dz;
    if !(sum < (*yp).rframe_barrel_in * (*yp).rframe_barrel_in) {
        return 0;
    }

    let rand_val = ext::HSD_Randf();
    let diff = (*yp).rframe_barrel_shoot_b - (*yp).rframe_barrel_shoot_a;
    (*gp).u.taru.xCA = f2i16(fmadds(diff, rand_val, (*yp).rframe_barrel_shoot_a) as f64);
    (*gp).u.taru.keep = keep;
    (*gp).u.taru.xC6 = 1;
    ext::Ground_801C5440(gp, 0, 0x129);
    ext::grAnime_801C7FF8(ground_gobj, 2, 7, 1, 0.0, 1.0);
    ext::grMaterial_801C9604(ground_gobj, (*yp).x6C, 0);
    ext::efSync_Spawn(0x405, ground_gobj, &mut pos_ft);
    ext::ftLib_StartRumble(keep, 0xD, 0x1E);
    1
}

pub unsafe extern "C" fn grOldKongo_802105AC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOldKongo_802105AC");
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.taru.xC6 == 1 {
        (*gp).u.taru.xC6 = 2;
    }
}

pub unsafe extern "C" fn grOldKongo_802105C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOldKongo_802105C8");
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.taru.xC6 != 1 {
        return;
    }

    assert!(!(*gp).u.taru.keep.is_null(), "HSD_ASSERT(751, gp->u.taru.keep)");

    if *((*gp).u.taru.keep as *const u8).add(2) == 8 {
        (*gp).u.taru.xC6 = 0;
        (*gp).u.taru.keep = null_mut();
        ext::grMaterial_801C95C4(gobj);
    }
}

pub unsafe extern "C" fn grOldKongo_80210650() -> f32 {
    melee_core::ct!("grOldKongo_80210650");
    let yp = &*yakumono_param;
    let total: i32 = yp.rrate_barrel_ld as i32 + yp.rrate_barrel_l as i32 + yp.rrate_barrel_lu as i32
        + yp.rrate_barrel_u as i32 + yp.rrate_barrel_ru as i32 + yp.rrate_barrel_r as i32
        + yp.rrate_barrel_rd as i32 + yp.rrate_barrel_d as i32;
    let mut r = if total != 0 { ext::HSD_Randi(total) } else { 0 };

    r = r.wrapping_sub(yp.rrate_barrel_ld as i32);
    if r < 0 {
        return 2.3561945f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_l as i32);
    if r < 0 {
        return 1.5707964f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_lu as i32);
    if r < 0 {
        return 0.7853982f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_u as i32);
    if r < 0 {
        return 0.0;
    }
    r = r.wrapping_sub(yp.rrate_barrel_ru as i32);
    if r < 0 {
        return -0.7853982f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_r as i32);
    if r < 0 {
        return -1.5707964f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_rd as i32);
    if r < 0 {
        return -2.3561945f64 as f32;
    }
    r = r.wrapping_sub(yp.rrate_barrel_d as i32);
    if r < 0 {
        return -3.1415927f64 as f32;
    }
    panic!("HSD_ASSERT(786, 0) groldkongo.c");
}

pub unsafe extern "C" fn grOldKongo_80210780(_gobj: i32) -> *mut melee_types::lbColl_80008D30_arg1 {
    melee_core::ct!("grOldKongo_80210780");
    null_mut()
}

pub unsafe extern "C" fn grOldKongo_80210788(a: *mut Vec3, _arg: i32, joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grOldKongo_80210788");
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint, null_mut(), &mut b);
    if (*a).y > b.y {
        1
    } else {
        0
    }
}

// ---- folded from leaves.rs (consolidation steps 21/22) ----
/// grOldKongo_802105AC(gobj).
///
/// # Safety
/// As [`grkongo_801d8270`].
pub unsafe fn groldkongo_802105ac(gobj: *mut HSD_GObj_leaves) {
    melee_core::ct!("groldkongo_802105ac");
    groldkongo_802105ac_gp(&mut *((*gobj).user_data as *mut Ground_leaves))
}

// ---- folded from groldkongo.rs (consolidation steps 21/22) ----
/// grOldKongo_8020F524_fp (gr/groldkongo.c:169)
    pub extern "C" fn grOldKongo_8020F524_fp() -> i32 { 0 }

/// grOldKongo_80210780_fp (gr/groldkongo.c:634)
    pub extern "C" fn grOldKongo_80210780_fp(a0: i32) -> *mut melee_types::lbColl_80008D30_arg1 { core::ptr::null_mut() }
