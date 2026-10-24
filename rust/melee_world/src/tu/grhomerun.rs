//! Port of src/melee/gr/grhomerun.c (Home-Run Contest: the endless stage built from `parts`, the
//! parallax backgrounds, the distance text and the sandbag landing marker). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_grhomerun.rs.
//!
//! Fused sites (tools/rust-port/fma_sites.py), each the gwtool `fmadds` (f64 product + f64 sum, one
//! f32 rounding): `grHomeRun_8021D680` (9: the three text coordinates `a + k * (s * scale)`, the two
//! `x +=/-= 2150.99F * scale` steps and the four background offsets), `grHomeRun_8021DF50/E0D4/E258/E3DC`
//! (3 each, the same text coordinates), `grHomeRun_8021E500` (`160 * (s * scale) + offset`) and
//! `grHomeRun_8021EA30` (`*pos - 70 * (s * scale)`).
//!
//! The overlay views of the union `GroundVars` are used as in the C: `u.unk.xC8/xCC/xD0` are 32-bit ints
//! holding pointers (`(int) ptr`, read back sign-extended), which in the native layout alias the halves
//! of `u.homerun.parts/back`. The oracle runs with every pointer below 2 GiB so these casts round-trip.
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::grhomerun::{
    grHomeRun_8021C824, grHomeRun_8021CB10, grHomeRun_8021D678, grHomeRun_8021DEE0, grHomeRun_8021DF48, grHomeRun_8021E064,
    grHomeRun_8021E0CC, grHomeRun_8021E1E8, grHomeRun_8021E250, grHomeRun_8021E36C, grHomeRun_8021E3D4, grHomeRun_8021E4F0,
    grHomeRun_8021EEB4,
};

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
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
    fn Camera_80030A50() -> *mut HSD_GObj;
    fn Camera_GetTransformInterest(a0: *mut Vec) = |a0: *mut Vec| { unsafe { *a0 = crate::cm::camera::world().camera_get_transform_interest(); } };
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32);
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C2CF4(a0: i32) -> *mut HSD_JObj = |a0: i32| unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ };
    fn Ground_801C38AC(a0: f32) = |a0: f32| { unsafe { let _ = crate::tu::ground::Ground_801C38AC(a0 as _); } };
    fn Ground_801C39B0(a0: f32) = |a0: f32| { unsafe { let _ = crate::tu::ground::Ground_801C39B0(a0 as _); } };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetP1Fighter() -> *mut HSD_GObj = || unsafe { crate::tu::ground::Ground_GetP1Fighter() as _ };
    fn Ground_GetP1Fighter2() -> *mut HSD_GObj = || unsafe { crate::tu::ground::Ground_GetP1Fighter2() as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_ArchiveGetPublicAddress(a0: *mut HSD_Archive, a1: *const i8) -> *mut c_void = |a0: *mut HSD_Archive, a1: *const i8| unsafe { melee_runtime::tu::archive::HSD_ArchiveGetPublicAddress(a0 as _, a1 as _) as _ };
    fn HSD_CObjEndCurrent() = || { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjEndCurrent(); } };
    fn HSD_CObjGetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjGetFar(a0: *mut HSD_CObj) -> f32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjGetFar(a0 as _) as _ };
    fn HSD_CObjGetFlags(a0: *mut HSD_CObj) -> u32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj_world::HSD_CObjGetFlags(a0 as _) as _ };
    fn HSD_CObjGetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjGetNear(a0: *mut HSD_CObj) -> f32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjGetNear(a0 as _) as _ };
    fn HSD_CObjGetPerspective(a0: *mut HSD_CObj, a1: *mut f32, a2: *mut f32) = |a0: *mut HSD_CObj, a1: *mut f32, a2: *mut f32| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetPerspective(a0 as _, a1 as _, a2 as _); } };
    fn HSD_CObjGetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = |a0: *mut HSD_CObj, a1: *mut _Scissor| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetScissor(a0 as _, a1 as _); } };
    fn HSD_CObjGetViewportf(a0: *mut HSD_CObj, a1: *mut _HSD_RectF32) = |a0: *mut HSD_CObj, a1: *mut _HSD_RectF32| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjGetViewportf(a0 as _, a1 as _); } };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetFar(a0 as _, a1 as _); } };
    fn HSD_CObjSetFlags(a0: *mut HSD_CObj, a1: u32) = |a0: *mut HSD_CObj, a1: u32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetFlags(a0 as _, a1 as _); } };
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetNear(a0 as _, a1 as _); } };
    fn HSD_CObjSetPerspective(a0: *mut HSD_CObj, a1: f32, a2: f32) = |a0: *mut HSD_CObj, a1: f32, a2: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetPerspective(a0 as _, a1 as _, a2 as _); } };
    fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = |a0: *mut HSD_CObj, a1: *mut _Scissor| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetScissor(a0 as _, a1 as _); } };
    fn HSD_CObjSetViewportf(a0: *mut HSD_CObj, a1: *mut _HSD_RectF32) = |a0: *mut HSD_CObj, a1: *mut _HSD_RectF32| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetViewportf(a0 as _, a1 as _); } };
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = |a0: *mut HSD_Text| { unsafe { let _ = melee_runtime::tu::sislib::HSD_SisLib_803A5CC4(a0 as _); } };
    fn HSD_SisLib_803A611C(a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32) -> i32 = |a0: i32, a1: *mut HSD_GObj, a2: u16, a3: u8, a4: u8, a5: u8, a6: u8, a7: u32| unsafe { melee_runtime::tu::sislib::HSD_SisLib_803A611C(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _) as _ };
    fn HSD_SisLib_803A6754(a0: i32, a1: i32) -> *mut HSD_Text = |a0: i32, a1: i32| unsafe { melee_runtime::tu::hsd_3A64::HSD_SisLib_803A6754(a0 as _, a1 as _) as _ };
    fn HSD_SisLib_803A6B98(a0: *mut HSD_Text, a1: f32, a2: f32, a3: *const i8, a4: i32) -> i32;
    fn Stage_GetBlastZoneRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneRightOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) = |a0: *mut Vec| { unsafe { let _ = crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } };
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086B74(a0: *mut HSD_GObj) -> *mut CmSubject;
    fn gm_80180AF4() -> *mut HSD_GObj;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct = || unsafe { crate::tu::grdatfiles::grDatFiles_GetArchive() as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_8026BE84(a0: *mut BobOmbRain) -> *mut HSD_GObj;
    fn lbLang_IsSavedLanguageJP() -> i32 = || unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageJP() as _ };
    fn lbLang_IsSavedLanguageUS() -> i32 = || unsafe { melee_runtime::tu::lblanguage::lbLang_IsSavedLanguageUS() as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn lb_80013B14(a0: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = |a0: *mut HSD_CameraDescPerspective| unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(a0 as _) as _ };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { let _ = crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpJointUpdateBounding(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::mplib::mpJointUpdateBounding(a0 as _); } };
    fn mpLib_80056758(a0: i32, a1: f32, a2: f32, a3: f32, a4: f32) = |a0: i32, a1: f32, a2: f32, a3: f32, a4: f32| { unsafe { let _ = crate::tu::mplib::mpLib_80056758(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Ground_801C0498() -> f32 { super::Ground_801C0498() }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObj_CameraKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_SisLib_804D1124: [*mut SIS; 5] = unsafe { core::mem::zeroed() };
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

/// gwtool's `fmadds`: the f32 product and sum fused in f64, one rounding to f32.
#[inline(always)]
fn fmadds(a: f32, b: f32, c: f32) -> f32 {
    ((a as f64) * (b as f64) + (c as f64)) as f32
}

/// `c - a * b` fused (clang emits `fmuladd(-a, b, c)`)
#[inline(always)]
fn fnmadds(a: f32, b: f32, c: f32) -> f32 {
    ((-a as f64) * (b as f64) + (c as f64)) as f32
}

/// `static f32 grHr_804D6AD8`
pub static mut grHr_804D6AD8: f32 = 0.0;
/// `static int grHr_804D6ADC`
pub static mut grHr_804D6ADC: i32 = 0;
/// `static f32 grHr_804D6AE0`
pub static mut grHr_804D6AE0: f32 = 0.0;
/// `static f32 grHr_804D6AE4`
pub static mut grHr_804D6AE4: f32 = 0.0;
/// `static void* yakumono_param`
pub static mut yakumono_param: *mut c_void = null_mut();
/// `static int grHr_804D4998[2]`
pub static mut grHr_804D4998: [i32; 2] = [0xA, 0];
/// `static char grHr_804D49A0[]`
pub static mut grHr_804D49A0: [u8; 6] = *b"/GrHr\0";

/// the function-local `static HSD_WObjDesc camera_eye_desc` of grHomeRun_8021CB20
pub static mut camera_eye_desc: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 1.0 }, robjdesc: null_mut() };
/// the function-local `static HSD_WObjDesc camera_interest_desc`
pub static mut camera_interest_desc: HSD_WObjDesc =
    HSD_WObjDesc { class_name: null_mut(), pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, robjdesc: null_mut() };
/// the function-local `static HSD_CameraDescPerspective cobj_desc`
pub static mut cobj_desc: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: 1,
    viewport: HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 640 },
    scissor: Scissor { left: 0, right: 480, top: 0, bottom: 480 },
    eyepos: unsafe { &raw mut camera_eye_desc },
    interest: unsafe { &raw mut camera_interest_desc },
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 30.0,
    aspect: 1.3636,
};

const fn cbs(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent,
             callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    let mut c: StageCallbacks = unsafe { core::mem::zeroed() };
    c.on_init = on_init;
    c.callback1 = callback1;
    c.gobj_proc = gobj_proc;
    c.callback3 = callback3;
    c.anon_10 = StageCallbacks_anon_135 { flags };
    c
}

/// `grHr_StageCallbacks`
pub static mut grHr_StageCallbacks: [StageCallbacks; 11] = [
    cbs(Some(grHomeRun_8021C914), Some(grHomeRun_8021CB10), Some(grHomeRun_8021CB18), Some(grHomeRun_8021CB1C), 0),
    cbs(Some(grHomeRun_8021E038), Some(grHomeRun_8021E064), Some(grHomeRun_8021E06C), Some(grHomeRun_8021E070), 0),
    cbs(Some(grHomeRun_8021E074), Some(grHomeRun_8021E0CC), Some(grHomeRun_8021E0D4), Some(grHomeRun_8021E18C), 0),
    cbs(Some(grHomeRun_8021E4C4), Some(grHomeRun_8021E4F0), Some(grHomeRun_8021E4F8), Some(grHomeRun_8021E4FC), 0),
    cbs(Some(grHomeRun_8021DEB4), Some(grHomeRun_8021DEE0), Some(grHomeRun_8021DEE8), Some(grHomeRun_8021DEEC), 0),
    cbs(Some(grHomeRun_8021DEF0), Some(grHomeRun_8021DF48), Some(grHomeRun_8021DF50), Some(grHomeRun_8021E008), 0),
    cbs(Some(grHomeRun_8021E1BC), Some(grHomeRun_8021E1E8), Some(grHomeRun_8021E1F0), Some(grHomeRun_8021E1F4), 0),
    cbs(Some(grHomeRun_8021E1F8), Some(grHomeRun_8021E250), Some(grHomeRun_8021E258), Some(grHomeRun_8021E310), 0),
    cbs(Some(grHomeRun_8021E340), Some(grHomeRun_8021E36C), Some(grHomeRun_8021E374), Some(grHomeRun_8021E378), 0),
    cbs(Some(grHomeRun_8021E37C), Some(grHomeRun_8021E3D4), Some(grHomeRun_8021E3DC), Some(grHomeRun_8021E494), 0),
    cbs(Some(grHomeRun_8021CB20), Some(grHomeRun_8021D678), Some(grHomeRun_8021D680), Some(grHomeRun_8021DEB0), 0xC0000000),
];

/// `grHr_StageData`
pub static mut grHr_StageData: StageData = StageData {
    grkind: Gr_Kind_Homerun,
    callbacks: unsafe { &raw mut grHr_StageCallbacks as *mut StageCallbacks },
    data1: unsafe { &raw mut grHr_804D49A0 as *mut i8 },
    on_init: Some(grHomeRun_8021C754),
    on_demo_init: Some(grHomeRun_8021C750),
    on_load: Some(grHomeRun_8021C7FC),
    on_start: Some(grHomeRun_8021C800),
    callback4: Some(grHomeRun_8021C824),
    on_touch_line: Some(grHomeRun_8021EEB4),
    on_check_shadow_render: Some(grHomeRun_8021EEBC),
    flags2: 1,
    joints: unsafe { &raw mut grHr_804D4998 as *mut GrJoint },
    joint_count: 1,
};

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// `/` and `%` as the shipping build computes them (gwtool's makeArithmeticSafe: x / 0 == (x < 0 ? -1 : 0),
/// x % 0 == x, INT_MIN / -1 == -1, INT_MIN % -1 == 0)
#[inline(always)]
fn sdiv(a: i32, b: i32) -> i32 {
    if b == 0 || (a == i32::MIN && b == -1) { if a < 0 { -1 } else { 0 } } else { a / b }
}
#[inline(always)]
fn srem(a: i32, b: i32) -> i32 {
    if b == 0 { a } else if a == i32::MIN && b == -1 { 0 } else { a % b }
}

/// `INIT_ADD_PARTS_RANGE`: calls the stage scale every time it is evaluated
#[inline(always)]
unsafe fn init_add_parts_range() -> f32 {
    1.0 + (2400.0 / (160.0 * ext::Ground_801C0498()))
}

/// `static inline Ground_UpdateWindAndMapColl` (gr/inlines.h)
unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut Ground_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// `HSD_JObjSetScale{X,Y,Z}(jobj, grHr_804D6AE4 * HSD_JObjGetScale{X,Y,Z}(jobj))`
unsafe fn scale_by_ae4(jobj: *mut HSD_JObj) {
    let v = grHr_804D6AE4 * HSD_JObjGetScaleX(&*jobj);
    HSD_JObjSetScaleX(&mut *jobj, v, &mut Dirty);
    let v = grHr_804D6AE4 * HSD_JObjGetScaleY(&*jobj);
    HSD_JObjSetScaleY(&mut *jobj, v, &mut Dirty);
    let v = grHr_804D6AE4 * HSD_JObjGetScaleZ(&*jobj);
    HSD_JObjSetScaleZ(&mut *jobj, v, &mut Dirty);
}

/// 21C750
pub unsafe extern "C" fn grHomeRun_8021C750(_arg: i32) {}

/// 21C754
pub unsafe extern "C" fn grHomeRun_8021C754() {
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    yakumono_param = ext::Ground_GetYakumonoParam();
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grHomeRun_8021EDD4();
    grHomeRun_8021C82C(0);
    grHomeRun_8021C82C(0xA);
    grHomeRun_8021ED74();
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
    let a = 3.0 * (ext::Stage_GetCamBoundsRightOffset() - cam_offset.x);
    ext::Ground_801C38AC(a);
    let b = 3.0 * (ext::Stage_GetBlastZoneRightOffset() - cam_offset.x);
    ext::Ground_801C39B0(b);
}

/// 21C7FC
pub unsafe extern "C" fn grHomeRun_8021C7FC() {}

/// 21C800
pub unsafe extern "C" fn grHomeRun_8021C800() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 21C82C (spells out `Ground_SetupStageCallbacks`)
pub unsafe extern "C" fn grHomeRun_8021C82C(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = &raw mut grHr_StageCallbacks[gobj_id as usize];

    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
        if (*callbacks).callback3.is_some() {
            (*gp).x1C_callback = (*callbacks).callback3;
        }
        if let Some(f) = (*callbacks).on_init {
            f(gobj);
        }
        if (*callbacks).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    } else {
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 305, gobj_id);
    }

    gobj
}

/// 21C914
pub unsafe extern "C" fn grHomeRun_8021C914(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);

    scale_by_ae4(jobj);
}

/// 21CB18
pub unsafe extern "C" fn grHomeRun_8021CB18(_arg: *mut Ground_GObj) {}

/// 21CB1C
pub unsafe extern "C" fn grHomeRun_8021CB1C(_arg: *mut Ground_GObj) {}

/// 21CB20
pub unsafe extern "C" fn grHomeRun_8021CB20(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let mut jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let jobj2 = jobj;
    ext::Ground_InitMapColl(jobj2, (*gp).map_id as i32);

    (*gp).u.homerun.parts = ext::HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * Gr_Homerun_Parts_Max as usize) as i32) as *mut *mut HSD_GObj;
    assert!(!(*gp).u.homerun.parts.is_null(), "gp->u.map.parts");

    (*gp).u.homerun.back = ext::HSD_MemAlloc((core::mem::size_of::<*mut HSD_GObj>() * Gr_Homerun_Parts_Back as usize) as i32) as *mut *mut HSD_GObj;
    assert!(!(*gp).u.homerun.back.is_null(), "gp->u.map.back");

    scale_by_ae4(jobj2);

    (*gp).x10_flags.set_b5(1);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::mpJointSetCb1(0, gp as *mut c_void, Some(fn_8021E994));

    let archive = ext::grDatFiles_GetArchive();
    (*gp).u.unk.text_gobj = ext::GObj_Create(HSD_GOBJ_CLASS_TEXT as u16, 19, 0);
    let cobj = ext::lb_80013B14(&raw mut cobj_desc);
    ext::HSD_CObjSetPerspective(cobj, 30.0, 1.4);
    {
        let kind = ext::HSD_GObj_CameraKind;
        ext::HSD_GObjObject_80390A70((*gp).u.unk.text_gobj, kind, cobj as *mut c_void);
    }
    ext::GObj_SetupGXLinkMax((*gp).u.unk.text_gobj, Some(fn_8021EB10), 7);
    (*(*gp).u.unk.text_gobj).gxlink_prios = 2;
    {
        let text_gobj = (*gp).u.unk.text_gobj;
        ext::HSD_SisLib_803A611C(1, text_gobj, HSD_GOBJ_CLASS_SISLIB_UNK as u16, 13, 0, 1, 0, 7);
    }
    ext::HSD_SisLib_804D1124[1] =
        ext::HSD_ArchiveGetPublicAddress((*archive).unk0, b"SIS_GrHomerunData\0".as_ptr() as *const i8) as *mut SIS;
    assert!(init_add_parts_range() * 2.0 < Gr_Homerun_Parts_Max as f32, "INIT_ADD_PARTS_RANGE*2<Gr_Homerun_Parts_Max");

    let mut i: i32 = 0;
    while (i as f32) < init_add_parts_range() {
        *(*gp).u.homerun.parts.add(i as usize) = grHomeRun_8021E500(i);
        i += 1;
    }
    while i < Gr_Homerun_Parts_Max as i32 {
        *(*gp).u.homerun.parts.add(i as usize) = null_mut();
        i += 1;
    }

    i = 0;
    while i < Gr_Homerun_Parts_Back as i32 {
        *(*gp).u.homerun.back.add(i as usize) = grHomeRun_8021C82C(4);
        assert!(!(*(*gp).u.homerun.back.add(i as usize)).is_null(), "gp->u.map.back[i]");

        jobj = (**(*gp).u.homerun.back.add(i as usize)).hsd_obj as *mut HSD_JObj;
        assert!(!jobj.is_null(), "jobj");

        scale_by_ae4(jobj);

        let child = HSD_JObjGetChild(jobj.as_ref());
        HSD_JObjSetTranslateX(&mut *child, 0.0, &mut Dirty);
        let t = grHr_804D6AE4 * ((4.0 + i as f32) * -(160.0 * ext::Ground_801C0498()));
        HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);
        i += 1;
    }

    (*gp).u.homerun.bg_gobj[0] = grHomeRun_8021C82C(3);
    assert!(!(*gp).u.homerun.bg_gobj[0].is_null(), "gp->u.map.bg_gobj[0]");
    jobj = (*(*gp).u.homerun.bg_gobj[0]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = 1.5 * -(2150.99 * ext::Ground_801C0498());
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    (*gp).u.homerun.bg_gobj[1] = grHomeRun_8021C82C(3);
    assert!(!(*gp).u.homerun.bg_gobj[1].is_null(), "gp->u.map.bg_gobj[1]");
    jobj = (*(*gp).u.homerun.bg_gobj[1]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = 0.5 * -(2150.99 * ext::Ground_801C0498());
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    (*gp).u.homerun.bg_gobj[2] = grHomeRun_8021C82C(3);
    assert!(!(*gp).u.homerun.bg_gobj[2].is_null(), "gp->u.map.bg_gobj[2]");
    jobj = (*(*gp).u.homerun.bg_gobj[2]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = 0.5 * (2150.99 * ext::Ground_801C0498());
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    (*gp).u.homerun.bg_gobj[3] = grHomeRun_8021C82C(3);
    assert!(!(*gp).u.homerun.bg_gobj[3].is_null(), "gp->u.map.bg_gobj[3]");
    jobj = (*(*gp).u.homerun.bg_gobj[3]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = 1.5 * (2150.99 * ext::Ground_801C0498());
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    (*gp).u.homerun.xCC = null_mut();
    (*gp).u.homerun.xD0 = ext::Ground_801C3FA4(gobj, 1);
    (*gp).u.homerun.xE8_flags.set_b0(0);
}

/// The text coordinate block shared by grHomeRun_8021D680 and the four `*_Callback2` of the text
/// stages: `pos` is the joint position, `text` the freshly created text.
unsafe fn set_text_pos(text: *mut HSD_Text, pos: &Vec3) {
    let mut scale = ext::Ground_801C0498();
    let z = fmadds(0.0, grHr_804D6AE4 * scale, pos.z);

    scale = ext::Ground_801C0498();
    let y = fmadds(0.0, grHr_804D6AE4 * scale, -pos.y);

    scale = ext::Ground_801C0498();
    (*text).pos_x = fmadds(-1.0, grHr_804D6AE4 * scale, pos.x);
    (*text).pos_y = y;
    (*text).pos_z = z;
}

/// 21D680
pub unsafe extern "C" fn grHomeRun_8021D680(gobj: *mut Ground_GObj) {
    let mut pos1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_interest = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let gp = (*gobj).user_data as *mut Ground;
    let mut fighter = ext::Ground_GetP1Fighter();
    if !fighter.is_null() {
        ext::ftLib_80086644(fighter, &mut pos0);
        let mut subject = ext::ftLib_80086B74(fighter);
        if !subject.is_null() {
            (*subject).state = CmSubjectState_Inactive;
            (*subject).set_force_inactive(1);
        }
        fighter = ext::Ground_GetP1Fighter2();
        if !fighter.is_null() {
            subject = ext::ftLib_80086B74(fighter);
            if !subject.is_null() {
                (*subject).state = CmSubjectState_Inactive;
                (*subject).set_force_inactive(1);
            }
        }
    } else {
        return;
    }
    {
        let sandbag_gobj = ext::gm_80180AF4();
        if !sandbag_gobj.is_null() {
            ext::ftLib_80086644(sandbag_gobj, &mut pos1);
            grHomeRun_8021EA30(&pos1.x);
            if pos1.y < 4.0 && (*gp).u.homerun.xE8_flags.b0() != 0 {
                grHomeRun_8021EAF8();
            }
        }
    }
    (*gp).u.homerun.xE8_flags.set_b0(0);
    if (*gp).u.unk.xD0 as u32 != 0 && (*gp).u.unk.xCC as u32 == 0 {
        (*gp).u.unk.xCC = grHomeRun_8021EC58(0) as isize as i32;
        let jobj2 = (*gp).u.unk.xD0 as isize as *mut HSD_JObj;
        ext::lb_8000B1CC(jobj2, null_mut(), &mut pos2);

        let text = (*gp).u.unk.xCC as isize as *mut HSD_Text;
        set_text_pos(text, &pos2);
    }

    ext::Camera_GetTransformInterest(&mut cam_interest);
    let mut x0: i32 = (((cam_interest.x - 2400.0) / (160.0 * ext::Ground_801C0498())) - 1.0) as i32;
    if x0 < 0 {
        x0 = 0;
    }

    let x1: i32 = (1.0 + ((2400.0 + cam_interest.x) / (160.0 * ext::Ground_801C0498()))) as i32;
    let mut gobj2 = *(*gp).u.homerun.parts;
    let mut gp2 = (*gobj2).user_data as *mut Ground;
    if ((*gp2).u.homerun2.xC4 as i32) < x0 {
        ext::Ground_801C4A08(gobj2);
        let mut i = 0usize;
        while i < 63 {
            *(*gp).u.homerun.parts.add(i) = *(*gp).u.homerun.parts.add(i + 1);
            i += 1;
        }
        *(*gp).u.homerun.parts.add(63) = null_mut();
    } else {
        gp2 = (*gobj2).user_data as *mut Ground;
        if (*gp2).u.homerun2.xC4 as i32 > x0 {
            gobj2 = grHomeRun_8021E500((*gp2).u.homerun2.xC4 as i32 - 1);
            let mut i = 63usize;
            while i > 0 {
                let vars = &raw mut (*gp).u.homerun;
                *(*vars).parts.add(i) = *(*vars).parts.add(i - 1);
                i -= 1;
            }
            *(*gp).u.homerun.parts = gobj2;
        }
    }

    let mut i: i32 = 63;
    while i > 0 {
        gobj2 = *(*gp).u.homerun.parts.add(i as usize);
        if !gobj2.is_null() {
            break;
        }
        i -= 1;
    }
    assert!(!gobj2.is_null(), "gobj2"); // HSD_ASSERT(0x239, gobj2)
    gp2 = (*gobj2).user_data as *mut Ground;
    if (*gp2).u.homerun2.xC4 as i32 > x1 {
        *(*gp).u.homerun.parts.add(i as usize) = null_mut();
        ext::Ground_801C4A08(gobj2);
    } else if (((*gp2).u.homerun2.xC4 as i32) < x1) {
        *(*gp).u.homerun.parts.add(i as usize + 1) = grHomeRun_8021E500((*gp2).u.homerun2.xC4 as i32 + 1);
    }

    let mut x: f32 = 0.85 * cam_interest.x;
    loop {
        let diff = x - cam_interest.x;
        let lo = 0.5 * -(2150.99 * ext::Ground_801C0498());
        if diff < lo {
            x = fmadds(2150.99, ext::Ground_801C0498(), x);
        } else {
            let hi = 0.5 * (2150.99 * ext::Ground_801C0498());
            if diff > hi {
                x = fnmadds(2150.99, ext::Ground_801C0498(), x);
            } else {
                break;
            }
        }
    }

    let mut jobj = (*(*gp).u.homerun.bg_gobj[0]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = fnmadds(1.5, 2150.99 * ext::Ground_801C0498(), x);
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    jobj = (*(*gp).u.homerun.bg_gobj[1]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = fnmadds(0.5, 2150.99 * ext::Ground_801C0498(), x);
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    jobj = (*(*gp).u.homerun.bg_gobj[2]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = fmadds(0.5, 2150.99 * ext::Ground_801C0498(), x);
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    jobj = (*(*gp).u.homerun.bg_gobj[3]).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let t = fmadds(1.5, 2150.99 * ext::Ground_801C0498(), x);
    HSD_JObjSetTranslateX(&mut *jobj, t, &mut Dirty);

    Ground_UpdateWindAndMapColl(gobj);
    ext::mpLib_80056758(3, 0.0, 0.0, 20000.0, 0.0);
    ext::mpJointUpdateBounding(0);
}

/// 21DEB0
pub unsafe extern "C" fn grHomeRun_8021DEB0(_arg: *mut Ground_GObj) {}

/// 21DEB4
pub unsafe extern "C" fn grHomeRun_8021DEB4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 21DEE8
pub unsafe extern "C" fn grHomeRun_8021DEE8(_arg: *mut Ground_GObj) {}

/// 21DEEC
pub unsafe extern "C" fn grHomeRun_8021DEEC(_arg: *mut Ground_GObj) {}

/// the on_init shared by the text stages (21DEF0, 21E074, 21E1F8, 21E37C)
unsafe fn text_stage_init(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.unk.xC8 = 0;
    (*gp).u.unk.xCC = ext::Ground_801C3FA4(gobj, 1) as isize as i32;
}

/// the gobj_proc shared by the text stages (21DF50, 21E0D4, 21E258, 21E3DC)
unsafe fn text_stage_proc(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.unk.xCC as u32 != 0 {
        if (*gp).u.unk.xC8 as u32 == 0 {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

            (*gp).u.unk.xC8 = grHomeRun_8021EC58((*gp).u.homerun2.xC6 as i32) as isize as i32;
            ext::lb_8000B1CC((*gp).u.unk.xCC as isize as *mut HSD_JObj, null_mut(), &mut pos);

            let text = (*gp).u.unk.xC8 as isize as *mut HSD_Text;
            set_text_pos(text, &pos);
        }
    }
}

/// the callback3 shared by the text stages (21E008, 21E18C, 21E310, 21E494)
unsafe fn text_stage_free(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.unk.xC8 as u32 != 0 {
        ext::HSD_SisLib_803A5CC4((*gp).u.unk.xC8 as isize as *mut HSD_Text);
    }
}

/// 21DEF0
pub unsafe extern "C" fn grHomeRun_8021DEF0(gobj: *mut Ground_GObj) { text_stage_init(gobj) }
/// 21DF50
pub unsafe extern "C" fn grHomeRun_8021DF50(gobj: *mut Ground_GObj) { text_stage_proc(gobj) }
/// 21E008
pub unsafe extern "C" fn grHomeRun_8021E008(gobj: *mut Ground_GObj) { text_stage_free(gobj) }

/// 21E038
pub unsafe extern "C" fn grHomeRun_8021E038(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 21E06C
pub unsafe extern "C" fn grHomeRun_8021E06C(_arg: *mut Ground_GObj) {}

/// 21E070
pub unsafe extern "C" fn grHomeRun_8021E070(_arg: *mut Ground_GObj) {}

/// 21E074
pub unsafe extern "C" fn grHomeRun_8021E074(gobj: *mut Ground_GObj) { text_stage_init(gobj) }
/// 21E0D4
pub unsafe extern "C" fn grHomeRun_8021E0D4(gobj: *mut Ground_GObj) { text_stage_proc(gobj) }
/// 21E18C
pub unsafe extern "C" fn grHomeRun_8021E18C(gobj: *mut Ground_GObj) { text_stage_free(gobj) }

/// 21E1BC
pub unsafe extern "C" fn grHomeRun_8021E1BC(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 21E1F0
pub unsafe extern "C" fn grHomeRun_8021E1F0(_arg: *mut Ground_GObj) {}

/// 21E1F4
pub unsafe extern "C" fn grHomeRun_8021E1F4(_arg: *mut Ground_GObj) {}

/// 21E1F8
pub unsafe extern "C" fn grHomeRun_8021E1F8(gobj: *mut Ground_GObj) { text_stage_init(gobj) }
/// 21E258
pub unsafe extern "C" fn grHomeRun_8021E258(gobj: *mut Ground_GObj) { text_stage_proc(gobj) }
/// 21E310
pub unsafe extern "C" fn grHomeRun_8021E310(gobj: *mut Ground_GObj) { text_stage_free(gobj) }

/// 21E340
pub unsafe extern "C" fn grHomeRun_8021E340(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 21E374
pub unsafe extern "C" fn grHomeRun_8021E374(_arg: *mut Ground_GObj) {}

/// 21E378
pub unsafe extern "C" fn grHomeRun_8021E378(_arg: *mut Ground_GObj) {}

/// 21E37C
pub unsafe extern "C" fn grHomeRun_8021E37C(gobj: *mut Ground_GObj) { text_stage_init(gobj) }
/// 21E3DC
pub unsafe extern "C" fn grHomeRun_8021E3DC(gobj: *mut Ground_GObj) { text_stage_proc(gobj) }
/// 21E494
pub unsafe extern "C" fn grHomeRun_8021E494(gobj: *mut Ground_GObj) { text_stage_free(gobj) }

/// 21E4C4
pub unsafe extern "C" fn grHomeRun_8021E4C4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 21E4F8
pub unsafe extern "C" fn grHomeRun_8021E4F8(_arg: *mut Ground_GObj) {}

/// 21E4FC
pub unsafe extern "C" fn grHomeRun_8021E4FC(_arg: *mut Ground_GObj) {}

/// 21E500
pub unsafe extern "C" fn grHomeRun_8021E500(arg0: i32) -> *mut HSD_GObj {
    let gobj_id: i32;

    let quot = sdiv(arg0, grHr_804D6ADC);
    let idx = quot % 4;
    if idx == 0 {
        gobj_id = if grHr_804D6ADC - 1 != srem(arg0, grHr_804D6ADC) { 4 } else { 5 };
    } else if idx == 1 {
        gobj_id = if grHr_804D6ADC - 1 != srem(arg0, grHr_804D6ADC) { 1 } else { 2 };
    } else if idx == 2 {
        gobj_id = if grHr_804D6ADC - 1 != srem(arg0, grHr_804D6ADC) { 6 } else { 7 };
    } else if idx == 3 {
        gobj_id = if grHr_804D6ADC - 1 != srem(arg0, grHr_804D6ADC) { 8 } else { 9 };
    } else {
        panic!("grhomerun.c:0x3D2: assertion failed (0)");
    }

    let gobj = grHomeRun_8021C82C(gobj_id);
    assert!(!gobj.is_null(), "gobj");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "jobj");
    let gp = (*gobj).user_data as *mut Ground;
    assert!(!gp.is_null(), "gp");

    scale_by_ae4(jobj);

    let child = HSD_JObjGetChild(jobj.as_ref());
    HSD_JObjSetTranslateX(&mut *child, 0.0, &mut Dirty);

    let offset = grHr_804D6AE4 * (arg0 as f32 * (160.0 * ext::Ground_801C0498()));
    (*gp).u.homerun2.xD0 = fmadds(160.0, grHr_804D6AE4 * ext::Ground_801C0498(), offset);
    HSD_JObjSetTranslateX(&mut *jobj, (*gp).u.homerun2.xD0, &mut Dirty);

    (*gp).u.homerun2.xC6 = ((grHr_804D6AE0 as i32).wrapping_mul(sdiv(arg0 + 1, grHr_804D6ADC))) as u16;
    (*gp).u.homerun2.xC4 = arg0 as u16;
    gobj
}

/// 21E994 (`fn_8021E994`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn fn_8021E994(_user_data: *mut c_void, _joint_id: i32, coll: *mut CollData, _coll_x50: i32,
                                     ground_kind: u32, _delta_y: f32) {
    let coll_kind = (*coll).x34_flags.b1234() as i32;
    let mut gobj = ext::Ground_GetMapGObj(0xA);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        if gp.is_null() {
            (*coll).x34_flags = (*coll).x34_flags;
            return;
        }
        if coll_kind == 1 {
            gobj = ext::gm_80180AF4();
            if (*coll).x0_gobj == gobj && ground_kind == 1 {
                (*gp).u.homerun.xE8_flags.set_b0(1);
            }
        }
    }
}

/// 21EA30
pub unsafe extern "C" fn grHomeRun_8021EA30(pos: *const f32) {
    let mut time = ext::Ground_801C0498();
    let mut result = fnmadds(70.0, grHr_804D6AE4 * time, *pos);

    time = ext::Ground_801C0498();
    let mut new_var = 160.0 * time;
    new_var = (grHr_804D6ADC as f32) * new_var;
    result /= grHr_804D6AE4 * new_var;
    result *= grHr_804D6AE0;

    if result < 0.0 {
        result = 0.0;
    }

    if ext::lbLang_IsSavedLanguageUS() != 0 {
        result = (result as f64 * 0.304788) as f32;
    }

    ext::stage_info.x6E0 = 100.0 * result;
}

/// 21EAF8
pub unsafe extern "C" fn grHomeRun_8021EAF8() {
    ext::stage_info.flags |= 1 << 7;
}

/// 21EB10 (`fn_8021EB10`): copies the main camera into a target camera
pub unsafe extern "C" fn fn_8021EB10(target_cam_gobj: *mut HSD_GObj, _code: i32) {
    let mut fov: f32 = 0.0;
    let mut aspect: f32 = 0.0;
    let mut viewport: HSD_RectF32 = core::mem::zeroed();
    let mut scissor: Scissor = core::mem::zeroed();
    let mut position = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let target_cam_cobj = (*target_cam_gobj).hsd_obj as *mut HSD_CObj;
    let main_cam_gobj = ext::Camera_80030A50();
    if !main_cam_gobj.is_null() {
        let main_cam_cobj = (*main_cam_gobj).hsd_obj as *mut HSD_CObj;
        if !main_cam_cobj.is_null() {
            let fl = ext::HSD_CObjGetFlags(main_cam_cobj);
            ext::HSD_CObjSetFlags(target_cam_cobj, fl);
            ext::HSD_CObjGetViewportf(main_cam_cobj, &mut viewport);
            ext::HSD_CObjSetViewportf(target_cam_cobj, &mut viewport);
            ext::HSD_CObjGetScissor(main_cam_cobj, &mut scissor);
            ext::HSD_CObjSetScissor(target_cam_cobj, &mut scissor);
            (*target_cam_cobj).u = (*main_cam_cobj).u;
            let n = ext::HSD_CObjGetNear(main_cam_cobj);
            ext::HSD_CObjSetNear(target_cam_cobj, n);
            let f = ext::HSD_CObjGetFar(main_cam_cobj);
            ext::HSD_CObjSetFar(target_cam_cobj, f);

            ext::HSD_CObjGetPerspective(main_cam_cobj, &mut fov, &mut aspect);
            ext::HSD_CObjSetPerspective(target_cam_cobj, fov, aspect);
            ext::HSD_CObjGetEyePosition(main_cam_cobj, &mut position);
            ext::HSD_CObjSetEyePosition(target_cam_cobj, &mut position);
            ext::HSD_CObjGetInterest(main_cam_cobj, &mut position);
            ext::HSD_CObjSetInterest(target_cam_cobj, &mut position);
        }
    }
    if ext::HSD_CObjSetCurrent(target_cam_cobj) != 0 {
        ext::HSD_FogSet(null_mut());
        ext::HSD_GObj_80390ED0(target_cam_gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

/// 21EC58
pub unsafe extern "C" fn grHomeRun_8021EC58(arg: i32) -> *mut HSD_Text {
    let color = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

    let text = ext::HSD_SisLib_803A6754(1, 0);
    (*text).pos_z = 0.0;

    let mut temp = grHr_804D6AE4 * ext::Ground_801C0498() * 8.0;
    (*text).box_size_x = grHr_804D6AE4 * ext::Ground_801C0498() * 13.0;
    (*text).box_size_y = temp;
    (*text).default_kerning = 0;

    temp = grHr_804D6AE4 * ext::Ground_801C0498() * 0.2;
    (*text).x34.x = grHr_804D6AE4 * ext::Ground_801C0498() * 0.2;
    (*text).x34.y = temp;
    (*text).text_color = color;
    (*text).default_alignment = 1;
    (*text).default_fitting = 1;
    (*text).x4C = 1;

    ext::HSD_SisLib_803A6B98(text, 0.0, -29.0, b"%d\0".as_ptr() as *const i8, arg as f32 as i32);

    text
}

/// 21ED74
pub unsafe extern "C" fn grHomeRun_8021ED74() {
    let mut bobomb_rain: BobOmbRain = core::mem::zeroed();
    let jobj = ext::Ground_801C2CF4(0x7F);
    bobomb_rain.x0 = null_mut();
    bobomb_rain.x4 = null_mut();
    bobomb_rain.x14 = 0xB;
    bobomb_rain.x18 = 0;
    bobomb_rain.x1C.anon_0.set_b0(1);
    ext::lb_8000B1CC(jobj, null_mut(), &mut bobomb_rain.x8_vec);
    ext::it_8026BE84(&mut bobomb_rain);
}

/// 21EDD4
pub unsafe extern "C" fn grHomeRun_8021EDD4() {
    let scale: f32;
    let max_dist: f32;

    if ext::lbLang_IsSavedLanguageJP() != 0 {
        scale = 0.1;
        max_dist = 50.0;
    } else {
        scale = 0.3280969;
        max_dist = 100.0;
    }

    let ratio = max_dist / scale;
    let mut num_ticks: f32 = (0.5 + ratio / (160.0 * ext::Ground_801C0498())) as i32 as f32;

    if num_ticks < 1.0 {
        num_ticks = 1.0;
    }

    let dist = 160.0 * ext::Ground_801C0498();
    grHr_804D6AD8 = scale;
    grHr_804D6AE0 = max_dist;
    grHr_804D6ADC = num_ticks as i32;
    grHr_804D6AE4 = ratio / (num_ticks * dist);
}

/// 21EEBC
pub unsafe extern "C" fn grHomeRun_8021EEBC(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}

/// 21EF10
pub unsafe extern "C" fn grHomeRun_8021EF10() -> f32 {
    5.0 * ext::Ground_801C0498()
}
