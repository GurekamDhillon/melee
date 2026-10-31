//! Port of src/melee/gr/grgreatbay.c (Great Bay: the turtle, the floating platform, the falling
//! moon and giants, Tingle's balloon). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_grgreatbay.rs.
//!
//! Fused sites (tools/rust-port/fma_sites.py), each the gwtool `fmadds` (f64 product + f64 sum, one
//! f32 rounding): `grGreatBay_801F5E28` (the three `x += delta * factor` easing steps) and
//! `grGreatBay_801F60C4` (`coll_x50 * mul + add` for the slant, slide and down amounts).
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::grgreatbay::{
    grGreatBay_801F423C, grGreatBay_801F42F8, grGreatBay_801F4430, grGreatBay_801F4438, grGreatBay_801F443C,
    grGreatBay_801F44A0, grGreatBay_801F44A8, grGreatBay_801F44AC, grGreatBay_801F4510, grGreatBay_801F4518,
    grGreatBay_801F451C, grGreatBay_801F4650, grGreatBay_801F4690, grGreatBay_801F4994, grGreatBay_801F545C,
    grGreatBay_801F55F8, grGreatBay_801F5988, grGreatBay_801F59F0, grGreatBay_801F59F8, grGreatBay_801F59FC,
    grGreatBay_801F5AF0, grGreatBay_801F5D48, grGreatBay_801F680C,
};

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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3F20(a0: *mut HSD_JObj) -> f32 = |a0: *mut HSD_JObj| unsafe { crate::tu::ground::Ground_801C3F20(a0 as _) as _ };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj| { unsafe { crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } };
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
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn ftCo_800C07F8(a0: *mut HSD_GObj, a1: u32, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u32, a2: *mut c_void| { unsafe { crate::ftdevice::ftCo_800C07F8(a0 as _, a1 as _, core::mem::transmute(a2)); } };
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } };
    fn ftLib_80086684(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086684(a0 as _, a1 as _); } };
    fn ftLib_80086B80(a0: *mut HSD_GObj) -> f32 = |a0: *mut HSD_GObj| unsafe { crate::ftlib::ftLib_80086B80(a0 as _) as _ };
    fn ft_80087838(a0: *mut HSD_GObj) -> i32;
    fn grAnime_801C77FC(a0: *mut HSD_GObj, a1: i32, a2: u32) = |a0: *mut HSD_GObj, a1: i32, a2: u32| { unsafe { crate::tu::granime::grAnime_801C77FC(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32) = |a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32| { unsafe { crate::tu::granime::grAnime_801C7A04(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grAnime_801C84A4(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C84A4(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut Vec| unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C97DC(a0: i32, a1: i32, a2: *mut HSD_JObj) = |a0: i32, a1: i32, a2: *mut HSD_JObj| { unsafe { crate::tu::grlib::grLib_801C97DC(a0 as _, a1 as _, a2 as _); } };
    fn grLib_801C9874(a0: *mut HSD_Generator) = |a0: *mut HSD_Generator| { unsafe { crate::tu::grlib::grLib_801C9874(a0 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_8026D324(a0: i32) -> i32;
    fn it_802EBD14(a0: *mut HSD_GObj);
    fn it_802EBFAC(a0: *mut HSD_GObj);
    fn it_802EC830(a0: *mut HSD_GObj);
    fn it_802ECA70(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn it_802ECC8C(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::itacc::it_802ECC8C(a0 as _) as _ };
    fn it_802ECC98(a0: *mut HSD_GObj, a1: f32) = |a0: *mut HSD_GObj, a1: f32| { unsafe { let _ = crate::itacc::it_802ECC98(a0 as _, a1 as _); } };
    fn it_802ECCA4(a0: *mut HSD_GObj, a1: *mut i32, a2: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut i32, a2: *mut Vec| { unsafe { let _ = crate::itacc::it_802ECCA4(a0 as _, a1 as _, a2 as _); } };
    fn lb_8000B134(a0: *mut HSD_JObj) -> i32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::lb_00B0::lb_8000B134(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80055E24(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E24(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057424(a0 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    fn mpLib_80057FDC(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057FDC(a0 as _); } };
    fn mpLib_80058044(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80058044(a0 as _); } };
    fn mpVtxSetPos(a0: i32, a1: f32, a2: f32) = |a0: i32, a1: f32, a2: f32| { unsafe { crate::tu::mplib::mpVtxSetPos(a0 as _, a1 as _, a2 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
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

/// gwtool's `fmadds`: the f32 product and sum fused in f64, one rounding to f32.
#[inline(always)]
fn fmadds(a: f32, b: f32, c: f32) -> f32 {
    ((a as f64) * (b as f64) + (c as f64)) as f32
}

/// One `{ kind, weight }` entry of `grGb_StageAttr::items` (anonymous struct in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grGb_StageAttr_item {
    pub kind: i16,
    pub weight: i16,
}

/// `grGb_StageAttr`: the stage's Yakumono parameter block.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grGb_StageAttr {
    pub moon_fall_wait_a: i16,
    pub moon_fall_wait_b: i16,
    pub floatfloor_landing_rate: f32,
    pub floatfloor_slant_mul: f32,
    pub floatfloor_slant_add: f32,
    pub floatfloor_slant_limit: f32,
    pub floatfloor_slant_rate: f32,
    pub floatfloor_slant_reb_rate: f32,
    pub floatfloor_slide_mul: f32,
    pub floatfloor_slide_add: f32,
    pub floatfloor_slide_limit: f32,
    pub floatfloor_slide_rate: f32,
    pub floatfloor_slide_reb_rate: f32,
    pub floatfloor_down_mul: f32,
    pub floatfloor_down_add: f32,
    pub floatfloor_down_limit: f32,
    pub floatfloor_down_up_rate: f32,
    pub floatfloor_down_down_rate: f32,
    pub kame_wait_frame_a: i16,
    pub kame_wait_frame_b: i16,
    pub kame_rebirth_frame_a: i16,
    pub kame_rebirth_frame_b: i16,
    pub kame_x: f32,
    pub kame_y: f32,
    pub kame_x_offset_init: f32,
    pub kame_x_lr_offset_a: f32,
    pub kame_x_lr_offset_b: f32,
    pub kame_x_fb_offset_a: f32,
    pub kame_x_fb_offset_b: f32,
    pub kame_scale: f32,
    pub kame_ud_scale: f32,
    pub kame_dir_prob: [i16; 4],
    pub kame_item_prob: f32,
    pub items: [grGb_StageAttr_item; 10],
}

/// `grGb_804D69E0_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grGb_804D69E0_t {
    pub x0: *mut grGb_StageAttr,
}

/// `grGb_804D69E0`
pub static mut grGb_804D69E0: grGb_804D69E0_t = grGb_804D69E0_t { x0: null_mut() };

/// `grGb_803E3E60`
pub static mut grGb_803E3E60: [GrJoint; 6] = [
    GrJoint { x: 0, y: 2, z: 49 },
    GrJoint { x: 1, y: 1, z: 2 },
    GrJoint { x: 2, y: 1, z: 3 },
    GrJoint { x: 4, y: 1, z: 34 },
    GrJoint { x: 3, y: 1, z: 38 },
    GrJoint { x: 5, y: 10, z: 0 },
];

const fn cbs(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent,
             callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    let mut c: StageCallbacks = unsafe { core::mem::zeroed() };
    c.on_init = on_init;
    c.callback1 = callback1;
    c.gobj_proc = gobj_proc;
    c.callback3 = callback3;
    c.anon_10 = StageCallbacks_anon_135 { flags: u32::to_be(flags) };
    c
}

/// `grGb_StageCallbacks`
pub static mut grGb_StageCallbacks: [StageCallbacks; 11] = [
    cbs(Some(stageGObj0_OnInit), Some(grGreatBay_801F4430), Some(grGreatBay_801F4438), Some(grGreatBay_801F443C), 0),
    // turtle
    cbs(Some(grGreatBay_801F4694), Some(grGreatBay_801F4994), Some(grGreatBay_801F499C), Some(grGreatBay_801F545C), 0),
    // main stage
    cbs(Some(grGreatBay_801F454C), Some(grGreatBay_801F4650), Some(grGreatBay_801F4658), Some(grGreatBay_801F4690), 0xC0000000),
    // background
    cbs(Some(grGreatBay_801F4440), Some(grGreatBay_801F44A0), Some(grGreatBay_801F44A8), Some(grGreatBay_801F44AC), 0),
    // foreground water
    cbs(Some(grGreatBay_801F44B0), Some(grGreatBay_801F4510), Some(grGreatBay_801F4518), Some(grGreatBay_801F451C), 0),
    // giant 1..4
    cbs(Some(grGreatBay_801F598C), Some(grGreatBay_801F59F0), Some(grGreatBay_801F59F8), Some(grGreatBay_801F59FC), 0),
    cbs(Some(grGreatBay_801F598C), Some(grGreatBay_801F59F0), Some(grGreatBay_801F59F8), Some(grGreatBay_801F59FC), 0),
    cbs(Some(grGreatBay_801F598C), Some(grGreatBay_801F59F0), Some(grGreatBay_801F59F8), Some(grGreatBay_801F59FC), 0),
    cbs(Some(grGreatBay_801F598C), Some(grGreatBay_801F59F0), Some(grGreatBay_801F59F8), Some(grGreatBay_801F59FC), 0),
    // moon
    cbs(Some(grGreatBay_801F5A00), Some(grGreatBay_801F5AF0), Some(grGreatBay_801F5AF8), Some(grGreatBay_801F5D48), 0),
    // tingle
    cbs(Some(grGreatBay_801F5460), Some(grGreatBay_801F55F8), Some(grGreatBay_801F5600), Some(grGreatBay_801F5988), 0),
];

/// `grGb_StageData`
pub static mut grGb_StageData: StageData = StageData {
    grkind: Gr_Kind_GreatBay,
    callbacks: unsafe { &raw mut grGb_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrGb.dat\0".as_ptr() as *mut i8,
    on_init: Some(grGreatBay_801F4240),
    on_demo_init: Some(grGreatBay_801F423C),
    on_load: Some(grGreatBay_801F42D0),
    on_start: Some(grGreatBay_801F42D4),
    callback4: Some(grGreatBay_801F42F8),
    on_touch_line: Some(grGreatBay_801F680C),
    on_check_shadow_render: Some(grGreatBay_801F6814),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grGb_803E3E60 as *mut GrJoint },
    joint_count: 6,
};

/// `grGb_803B81C8`
pub static grGb_803B81C8: Vec3 = Vec3 { x: 0.99, y: 35.72, z: 0.0 };

/// `grGb_803B81D4`: the item template the turtle's drops start from
pub static mut grGb_803B81D4: SpawnItem = SpawnItem {
    x0_parent_gobj: null_mut(),
    x4_parent_gobj2: null_mut(),
    kind: -1i32 as ItemKind,
    hold_kind: -1,
    x10: 1,
    pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    prev_pos: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    facing_dir: -1.0,
    x3C_damage: 0,
    x3E: 0,
    x40: 0,
    x44_flag: UnkFlagStruct { byte: 0x80 },
    x45_flag: UnkFlagStruct { byte: 0 },
    x46_flag: UnkFlagStruct { byte: 0 },
    x47_flag: UnkFlagStruct { byte: 0 },
    x48_ground_or_air: 0 as GroundOrAir,
};

/// `grGb_803E4048`
pub static grGb_803E4048: [[i16; 2]; 8] = [[32, 20], [23, 18], [22, 15], [29, 16], [28, 17], [27, 19], [39, 21], [0, 0]];

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

#[inline(always)]
unsafe fn attr() -> *mut grGb_StageAttr {
    grGb_804D69E0.x0
}

/// `ZRANDI(n)`
#[inline(always)]
unsafe fn zrandi(n: i32) -> i32 {
    if n != 0 { ext::HSD_Randi(n) } else { 0 }
}

/// `static inline rand_range` (gr/inlines.h)
unsafe fn rand_range(a: i32, b: i32) -> i32 {
    if a > b {
        b.wrapping_add(zrandi(a.wrapping_sub(b)))
    } else if a < b {
        a.wrapping_add(zrandi(b.wrapping_sub(a)))
    } else {
        a
    }
}

/// `static inline Ground_StartMapAnim` (gr/inlines.h)
unsafe fn Ground_StartMapAnim(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `static inline Ground_InitMapCollAndAnim` (gr/inlines.h)
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 1F4240
pub unsafe extern "C" fn grGreatBay_801F4240() {
    melee_core::ct!("grGreatBay_801F4240");
    grGb_804D69E0.x0 = ext::Ground_GetYakumonoParam() as *mut grGb_StageAttr;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grGreatBay_801F4300(0);
    grGreatBay_801F4300(3);
    grGreatBay_801F4300(4);
    grGreatBay_801F4300(2);
    grGreatBay_801F4300(1);
    grGreatBay_801F4300(9);
    grGreatBay_801F4300(10);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// 1F42D0
pub unsafe extern "C" fn grGreatBay_801F42D0() { melee_core::ct!("grGreatBay_801F42D0");}

/// 1F42D4
pub unsafe extern "C" fn grGreatBay_801F42D4() {
    melee_core::ct!("grGreatBay_801F42D4");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 1F4300
pub unsafe extern "C" fn grGreatBay_801F4300(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grGreatBay_801F4300");
    let callbacks = &raw mut grGb_StageCallbacks[gobj_id as usize];

    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
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
            if gobj_id == 10 {
                ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 5);
            } else {
                ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
            }
        }
    } else {
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 247, gobj_id);
    }

    gobj
}

/// `static stageGObj0_OnInit`
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut Ground_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

/// 1F4440
pub unsafe extern "C" fn grGreatBay_801F4440(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F4440");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    (*gp).x11_flags.set_b012(1);
}

/// 1F44B0
pub unsafe extern "C" fn grGreatBay_801F44B0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F44B0");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    (*gp).x11_flags.set_b012(3);
}

/// 1F4520 (`fn_801F4520`)
pub unsafe extern "C" fn fn_801F4520(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801F4520");
    ext::ftCo_800C07F8(gobj, 6, grGreatBay_801F6708 as *mut c_void);
}

/// 1F454C
pub unsafe extern "C" fn grGreatBay_801F454C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F454C");
    let gp = GET_GROUND(gobj);

    Ground_InitMapCollAndAnim(gobj);
    (*gp).x8_callback = None;
    (*gp).x11_flags.set_b012(1);
    (*gp).xC_callback = None;
    (*gp).x10_flags.set_b5(1);

    let a = ext::Ground_801C3FA4(gobj, 4);
    let b = ext::Ground_801C3FA4(gobj, 1);
    let c = ext::Ground_801C3FA4(gobj, 3);
    let d = ext::Ground_801C3FA4(gobj, 2);
    let e = ext::Ground_801C3FA4(gobj, 6);
    let f = ext::Ground_801C3FA4(gobj, 5);
    ext::Ground_801C4E70(a, b, c, d, e, f);
    ext::mpLib_80057BC0(1);
    grGreatBay_801F5D4C(gobj);
    ext::Ground_801C10B8(gobj, Some(fn_801F4520));
}

/// 1F4658
pub unsafe extern "C" fn grGreatBay_801F4658(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F4658");
    grGreatBay_801F5E28(gobj);
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// 1F4694
pub unsafe extern "C" fn grGreatBay_801F4694(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F4694");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);

    HSD_JObjGetScale(&*jobj, &mut scale);
    let factor = (*attr()).kame_scale;
    scale.x *= factor;
    scale.y *= factor;
    scale.z *= factor;
    HSD_JObjSetScale(&mut *jobj, &scale, &mut Dirty);

    HSD_JObjSetTranslateX(&mut *jobj, (*attr()).kame_x + (*attr()).kame_x_offset_init, &mut Dirty);
    HSD_JObjSetTranslateY(&mut *jobj, (*attr()).kame_y, &mut Dirty);

    let j2 = ext::Ground_801C3FA4(gobj, 2);
    (*gp).u.greatbay.xCC = HSD_JObjGetTranslationY(&*j2);

    (*gp).u.greatbay.xC4 = 0;
    (*gp).u.greatbay.xC5.set_b0123456(0);

    let max_val = (*attr()).kame_wait_frame_b;
    let min_val = (*attr()).kame_wait_frame_a;
    (*gp).u.greatbay.xC6 = rand_range(max_val as i32, min_val as i32) as i16;

    (*gp).u.greatbay.xC5.set_b7(1);
    (*gp).u.greatbay.xC8 = null_mut();
    (*gp).x10_flags.set_b5(1);
    (*gp).x11_flags.set_b012(1);
}

/// 1F499C
pub unsafe extern "C" fn grGreatBay_801F499C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F499C");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let trans_jobj = ext::Ground_801C3FA4(gobj, 2);
    let jobj3 = ext::Ground_801C3FA4(gobj, 3);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    assert!(!jobj.is_null(), "jobj");
    assert!(!trans_jobj.is_null(), "trans_jobj");

    match (*gp).u.greatbay.xC4 {
        0 => {
            if (*gp).u.greatbay.xC6 < 0 {
                if ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
                    (*gp).u.greatbay.xC4 = 1;
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 1);
                    (*gp).u.greatbay.xC6 = 0;
                    ext::mpLib_80058044(2);
                    ext::mpLib_80058044(3);
                    ext::mpLib_80058044(4);
                    ext::mpLib_80058044(1);
                }
            } else {
                (*gp).u.greatbay.xC6 -= 1;
            }
        }
        1 => {
            let timer = (*gp).u.greatbay.xC6;
            (*gp).u.greatbay.xC6 = timer.wrapping_add(1);
            if timer == 90 {
                ext::Ground_801C53EC(0x57E42);
                ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
                (*gp).u.greatbay.xC8 = ext::grLib_801C96F8(0x7531, 0x1E, &mut pos);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                (*gp).u.greatbay.xC4 = 3;
                (*gp).u.greatbay.xC6 =
                    rand_range((*attr()).kame_rebirth_frame_b as i32, (*attr()).kame_rebirth_frame_a as i32) as i16;
                ext::grAnime_801C7A04(gobj, 0, 7, 0.0);
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                if !(*gp).u.greatbay.xC8.is_null() {
                    ext::grLib_801C9874((*gp).u.greatbay.xC8);
                    (*gp).u.greatbay.xC8 = null_mut();
                }
            }
        }
        2 => {
            let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            (*gp).u.greatbay.xC6 = (*gp).u.greatbay.xC6.wrapping_add(1);
            if (*gp).u.greatbay.xC6 == 60 {
                ext::Ground_801C53EC(0x57E41);
                ext::lb_8000B1CC(jobj, null_mut(), &mut pos2);
                (*gp).u.greatbay.xC8 = ext::grLib_801C96F8(0x7531, 0x1E, &mut pos2);
            }
            {
                if (*gp).u.greatbay.xC6 == 60 || (*gp).u.greatbay.xC6 == 120 {
                    let j = ext::Ground_801C3FA4(gobj, 34);
                    if !j.is_null() {
                        ext::grLib_801C97DC(0x7533, 0x1E, j);
                    }
                }
            }
            {
                if (*gp).u.greatbay.xC6 == 90 || (*gp).u.greatbay.xC6 == 150 {
                    let j = ext::Ground_801C3FA4(gobj, 38);
                    if !j.is_null() {
                        ext::grLib_801C97DC(0x7533, 0x1E, j);
                    }
                }
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                (*gp).u.greatbay.xC4 = 0;
                (*gp).u.greatbay.xC6 =
                    rand_range((*attr()).kame_wait_frame_b as i32, (*attr()).kame_wait_frame_a as i32) as i16;
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                ext::mpLib_80057FDC(2);
                ext::mpLib_80057FDC(3);
                ext::mpLib_80057FDC(4);
                ext::mpLib_80057FDC(1);
                if !(*gp).u.greatbay.xC8.is_null() {
                    ext::grLib_801C9874((*gp).u.greatbay.xC8);
                    (*gp).u.greatbay.xC8 = null_mut();
                }
            }
        }
        3 => {
            let timer = (*gp).u.greatbay.xC6;
            (*gp).u.greatbay.xC6 = timer.wrapping_sub(1);
            if timer < 0 {
                (*gp).u.greatbay.xC4 = 2;
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 2);
                (*gp).u.greatbay.xC6 = 0;
                (*gp).u.greatbay.xC5.set_b7(0);
                let cur = (*gp).u.greatbay.xC5.b0123456() as i32;
                (*gp).u.greatbay.xC5.set_b0123456(grGreatBay_801F62F8(cur) as u8);
                match (*gp).u.greatbay.xC5.b0123456() {
                    1 => {
                        let o = rand_range((*attr()).kame_x_fb_offset_b as i32, (*attr()).kame_x_fb_offset_a as i32);
                        HSD_JObjSetTranslateX(&mut *jobj, (*attr()).kame_x + o as f32, &mut Dirty);
                        ext::mpLib_80057BC0(2);
                        ext::mpLib_80057BC0(3);
                        ext::mpLib_80057BC0(4);
                        ext::mpJointListAdd(1);
                    }
                    2 => {
                        panic!("grgreatbay.c:652: assertion failed (0)");
                    }
                    0 => {
                        let o = rand_range((*attr()).kame_x_lr_offset_b as i32, (*attr()).kame_x_lr_offset_a as i32);
                        HSD_JObjSetTranslateX(&mut *jobj, (*attr()).kame_x + o as f32, &mut Dirty);
                        ext::mpJointListAdd(2);
                        ext::mpJointListAdd(3);
                        ext::mpJointListAdd(4);
                        ext::mpLib_80057BC0(1);
                    }
                    3 => {
                        let o = rand_range((*attr()).kame_x_fb_offset_b as i32, (*attr()).kame_x_fb_offset_a as i32);
                        HSD_JObjSetTranslateX(&mut *jobj, (*attr()).kame_x + o as f32, &mut Dirty);
                        ext::mpLib_80057BC0(2);
                        ext::mpLib_80057BC0(3);
                        ext::mpLib_80057BC0(4);
                        ext::mpJointListAdd(1);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    {
        let mut y = HSD_JObjGetTranslationY(&*trans_jobj);
        if (*gp).u.greatbay.xCC != y {
            y *= (*attr()).kame_ud_scale;
            HSD_JObjSetTranslateY(&mut *trans_jobj, y, &mut Dirty);
            (*gp).u.greatbay.xCC = y;
        }
    }

    match (*gp).u.greatbay.xC5.b0123456() {
        1 => HSD_JObjSetRotationY(&mut *jobj3, 1.5707964, &mut Dirty),
        2 => panic!("grgreatbay.c:692: assertion failed (0)"),
        0 => HSD_JObjSetRotationY(&mut *trans_jobj, 0.0, &mut Dirty),
        3 => HSD_JObjSetRotationY(&mut *jobj3, -1.5707964, &mut Dirty),
        _ => {}
    }

    ext::Ground_UpdateMapColl(gobj);
    grGreatBay_801F660C(gobj);
    ext::mpLib_80055E24(2);

    if (*gp).u.greatbay.xC4 == 2 && (*gp).u.greatbay.xC5.b7() == 0 {
        (*gp).u.greatbay.xC5.set_b7(grGreatBay_801F63F4(gobj) as u8);
    }
}

/// `static inline grGreatBay_801F5460_inline`
pub unsafe fn grGreatBay_801F5460_inline(gobj: *mut Ground_GObj, jobj: *mut HSD_JObj) {
    melee_core::ct!("grGreatBay_801F5460_inline");
    let gp = GET_GROUND(gobj);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    (*gp).xC_callback = None;
    ext::mpJointSetCb1(5, gp as *mut c_void, Some(grGreatBay_801F5914));
    ext::mpJointListAdd(5);
}

/// 1F5460
pub unsafe extern "C" fn grGreatBay_801F5460(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F5460");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let gp = (*gobj).user_data as *mut Ground;

    grGreatBay_801F5460_inline(gobj, jobj);
    HSD_JObjSetTranslateX(&mut *jobj, 1000.0, &mut Dirty);
    HSD_JObjSetTranslateY(&mut *jobj, 1000.0, &mut Dirty);
    ext::Ground_UpdateMapColl(gobj);
    ext::mpLib_80057424(5);
    (*gp).u.greatbay4.xF0 = ext::it_802ECA70(gobj);
    (*gp).u.greatbay4.xC4 = 0;
    (*gp).u.greatbay4.xC8 = 0;
    (*gp).u.greatbay4.xD0 = 0;
    (*gp).u.greatbay4.xCC = 0;
    (*gp).u.greatbay4.xD8 = 0;
    (*gp).u.greatbay4.xDC = 0;
    (*gp).u.greatbay4.xE0 = 0.0;
}

/// 1F5600
pub unsafe extern "C" fn grGreatBay_801F5600(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F5600");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let mut pos;

    ext::it_802ECCA4((*gp).u.greatbay4.xF0, &raw mut (*gp).u.greatbay4.xC4, &raw mut (*gp).u.greatbay4.xE4);
    if (*gp).u.greatbay4.xE4.y < -50.0 {
        ext::it_802EC830((*gp).u.greatbay4.xF0);
        ext::mpLib_80057424(5);
        (*gp).u.greatbay4.xF0 = ext::it_802ECA70(gobj);
        (*gp).u.greatbay4.xC4 = 0;
        (*gp).u.greatbay4.xC8 = 0;
        (*gp).u.greatbay4.xD0 = 0;
        (*gp).u.greatbay4.xCC = 0;
        (*gp).u.greatbay4.xD8 = 0;
        (*gp).u.greatbay4.xDC = 0;
        (*gp).u.greatbay4.xE0 = 0.0;
        return;
    }
    ext::it_802ECC98((*gp).u.greatbay4.xF0, (*gp).u.greatbay4.xE0);
    match (*gp).u.greatbay4.xC4 {
        1 => {
            ext::mpJointListAdd(5);
            ext::Ground_UpdateMapColl(gobj);
        }
        2 | 5 => {
            if (*gp).u.greatbay4.xD8 != 0 && (*gp).u.greatbay4.xD4 == 0 {
                ext::it_802EBD14((*gp).u.greatbay4.xF0);
            }
        }
        4 => {
            if (*gp).u.greatbay4.xD8 == 0 {
                ext::it_802EBFAC((*gp).u.greatbay4.xF0);
            } else if (*gp).u.greatbay4.xDC != (*gp).u.greatbay4.xD8 {
                ext::it_802EBD14((*gp).u.greatbay4.xF0);
            }
        }
        6 | 7 => {
            if (*gp).u.greatbay4.xCC == 0 && ext::it_802ECC8C((*gp).u.greatbay4.xF0) == 0 {
                pos = (*gp).u.greatbay4.xE4;
                pos.y += 10.0;
                ext::grLib_801C96F8(0x7534, 0x1E, &mut pos);
                ext::Ground_801C53EC(0x57E4A);
                (*gp).u.greatbay4.xCC = 1;
            }
            ext::mpLib_80057BC0(5);
        }
        8 => {
            if (*gp).u.greatbay4.xD0 == 0 && (*gp).u.greatbay4.xE4.y <= 0.0 {
                (*gp).u.greatbay4.xD0 = 1;
                ext::grLib_801C96F8(0x7537, 0x1E, &raw mut (*gp).u.greatbay4.xE4);
                ext::Ground_801C53EC(0x57E48);
            }
        }
        9 => {
            if (*gp).u.greatbay4.xC8 != (*gp).u.greatbay4.xC4 {
                ext::Ground_801C53EC(0x57E47);
            }
        }
        12 => {
            if (*gp).u.greatbay4.xC8 != (*gp).u.greatbay4.xC4 {
                (*gp).u.greatbay4.xCC = 0;
                ext::mpJointListAdd(5);
            }
            if (*gp).u.greatbay4.xD8 != 0 {
                ext::it_802EBD14((*gp).u.greatbay4.xF0);
            }
        }
        _ => {}
    }
    (*gp).u.greatbay4.xC8 = (*gp).u.greatbay4.xC4;
    HSD_JObjSetTranslate(&mut *jobj, &(*gp).u.greatbay4.xE4, &mut Dirty);
    ext::Ground_UpdateMapColl(gobj);
    (*gp).u.greatbay4.xDC = (*gp).u.greatbay4.xD8;
    (*gp).u.greatbay4.xD8 = 0;
    (*gp).u.greatbay4.xE0 = 0.0;
    ext::Ground_UpdateMapColl(gobj);
}

/// 1F5914 (`grGreatBay_801F5914`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn grGreatBay_801F5914(user_data: *mut c_void, _joint_id: i32, coll: *mut CollData, coll_x50: i32,
                                             _ground_kind: u32, _delta_y: f32) {
    melee_core::ct!("grGreatBay_801F5914");
    let gp = user_data as *mut Ground;

    let temp_r0 = (*coll).x34_flags.b1234() as i32;
    if temp_r0 == 1 || temp_r0 == 2 || temp_r0 == 3 {
        (*gp).u.greatbay.xD4 = ((*coll).env_flags as u32 & (Collide_LeftLedgeGrab | Collide_RightLedgeGrab) as u32) as i32;
        (*gp).u.greatbay.xD8 = (*gp).u.greatbay.xD8.wrapping_add(1);
        (*gp).u.greatbay.xE0 += coll_x50 as f32 / 100.0;
    }
}

/// 1F598C
pub unsafe extern "C" fn grGreatBay_801F598C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F598C");
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C7A04(gobj, 0, 7, 0.0);
    (*gp).x11_flags.set_b012(2);
}

/// 1F5A00
pub unsafe extern "C" fn grGreatBay_801F5A00(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F5A00");
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C7A04(gobj, 0, 7, 0.0);
    (*gp).u.greatbay2.gobjs[0] = grGreatBay_801F4300(5);
    (*gp).u.greatbay2.gobjs[1] = grGreatBay_801F4300(6);
    (*gp).u.greatbay2.gobjs[2] = grGreatBay_801F4300(7);
    (*gp).u.greatbay2.gobjs[3] = grGreatBay_801F4300(8);

    let max_val = (*attr()).moon_fall_wait_b;
    let min_val = (*attr()).moon_fall_wait_a;
    (*gp).u.greatbay2.x10 = rand_range(max_val as i32, min_val as i32) as i16;

    (*gp).x11_flags.set_b012(2);
}

/// 1F5AF8
pub unsafe extern "C" fn grGreatBay_801F5AF8(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F5AF8");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    if (*gp).u.greatbay2.x10 > 0 {
        (*gp).u.greatbay2.x10 = ((*gp).u.greatbay2.x10 as i32 - 1) as i16;
        if (*gp).u.greatbay2.x10 == 0 {
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
            ext::grAnime_801C7A04(gobj, 0, 7, 1.0);
            let mut i = 0usize;
            while i < 4 {
                let g = (*gp).u.greatbay2.gobjs[i];
                if !g.is_null() {
                    let child_gp = GET_GROUND(g);
                    if !child_gp.is_null() {
                        ext::grAnime_801C8138((*gp).u.greatbay2.gobjs[i], (*child_gp).map_id as i32, 0);
                        ext::grAnime_801C7A04((*gp).u.greatbay2.gobjs[i], 0, 7, 1.0);
                    }
                }
                i += 1;
            }
            (*gp).u.greatbay2.x12.set_b0(0);
            (*gp).u.greatbay2.x12.set_b1(0);
            (*gp).u.greatbay2.x12.set_b2(0);
            (*gp).u.greatbay2.x12.set_b3(0);
        }
    } else {
        let frame = ext::Ground_801C3F20(jobj);
        if (*gp).u.greatbay2.x12.b0() == 0 && frame > 0.0 {
            (*gp).u.greatbay2.x12.set_b0(1);
        }
        if (*gp).u.greatbay2.x12.b1() == 0 && frame > 0.0 {
            (*gp).u.greatbay2.x12.set_b1(1);
        }
        if (*gp).u.greatbay2.x12.b2() == 0 && frame > 0.0 {
            (*gp).u.greatbay2.x12.set_b2(1);
        }
        if (*gp).u.greatbay2.x12.b3() == 0 && frame > 0.0 {
            (*gp).u.greatbay2.x12.set_b3(1);
        }
        if ext::lb_8000B134(jobj) != 0 {
            ext::grAnime_801C7A04(gobj, 0, 7, 0.0);
            let mut i = 0usize;
            while i < 4 {
                if !(*gp).u.greatbay2.gobjs[i].is_null() {
                    ext::grAnime_801C7A04((*gp).u.greatbay2.gobjs[i], 0, 7, 0.0);
                }
                i += 1;
            }
            (*gp).u.greatbay2.x10 = rand_range((*attr()).moon_fall_wait_b as i32, (*attr()).moon_fall_wait_a as i32) as i16;
        }
    }
}

/// 1F5D4C
pub unsafe extern "C" fn grGreatBay_801F5D4C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F5D4C");
    let gp = GET_GROUND(gobj);

    ext::mpJointSetCb1(0, gp as *mut c_void, Some(grGreatBay_801F60C4));
    let jobj = ext::Ground_801C3FA4(gobj, 49);
    (*gp).u.greatbay3.jobj = jobj;
    if !jobj.is_null() {
        (*gp).u.greatbay3.translation.x = HSD_JObjGetTranslationX(&*(*gp).u.greatbay3.jobj);
        (*gp).u.greatbay3.translation.y = HSD_JObjGetTranslationY(&*(*gp).u.greatbay3.jobj);
    } else {
        panic!("grgreatbay.c:1132: assertion failed (0)");
    }
    (*gp).u.greatbay3.translation.z = 0.0;
    (*gp).u.greatbay3.xD0 = 0.0;
    (*gp).u.greatbay3.xD4 = 0.0;
    (*gp).u.greatbay3.xD8 = 0.0;
    (*gp).u.greatbay3.xDC = 0.0;
    (*gp).u.greatbay3.xE0 = 0.0;
}

/// 1F5E28
pub unsafe extern "C" fn grGreatBay_801F5E28(gobj: *mut HSD_GObj) {
    melee_core::ct!("grGreatBay_801F5E28");
    let gp = GET_GROUND(gobj);
    let jobj = (*gp).u.greatbay3.jobj;

    if !jobj.is_null() {
        let mut delta: f32;
        let mut factor: f32;

        delta = (*gp).u.greatbay3.translation.z - (*gp).u.greatbay3.xD8;
        if -(*gp).u.greatbay3.xD8 * delta < 0.0 {
            factor = (*attr()).floatfloor_slant_rate;
        } else {
            factor = (*attr()).floatfloor_slant_reb_rate;
        }
        (*gp).u.greatbay3.xD8 = fmadds(delta, factor, (*gp).u.greatbay3.xD8);

        delta = (*gp).u.greatbay3.xD0 - (*gp).u.greatbay3.xDC;
        if -(*gp).u.greatbay3.xDC * delta < 0.0 {
            factor = (*attr()).floatfloor_slide_rate;
        } else {
            factor = (*attr()).floatfloor_slide_reb_rate;
        }
        (*gp).u.greatbay3.xDC = fmadds(delta, factor, (*gp).u.greatbay3.xDC);

        delta = (*gp).u.greatbay3.xD4 - (*gp).u.greatbay3.xE0;
        if delta < 0.0 {
            factor = (*attr()).floatfloor_down_down_rate;
        } else {
            factor = (*attr()).floatfloor_down_up_rate;
        }
        (*gp).u.greatbay3.xE0 = fmadds(delta, factor, (*gp).u.greatbay3.xE0);

        HSD_JObjSetRotationZ(&mut *jobj, (*gp).u.greatbay3.xD8, &mut Dirty);
        HSD_JObjSetTranslateX(&mut *jobj, (*gp).u.greatbay3.translation.x + (*gp).u.greatbay3.xDC, &mut Dirty);
        HSD_JObjSetTranslateY(&mut *jobj, (*gp).u.greatbay3.translation.y + (*gp).u.greatbay3.xE0, &mut Dirty);

        (*gp).u.greatbay3.translation.z = 0.0;
        (*gp).u.greatbay3.xD0 = 0.0;
        (*gp).u.greatbay3.xD4 = 0.0;
    }
}

/// 1F60C4 (`grGreatBay_801F60C4`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn grGreatBay_801F60C4(user_data: *mut c_void, _joint_id: i32, coll: *mut CollData, mut coll_x50: i32,
                                             ground_kind: u32, _delta_y: f32) {
    melee_core::ct!("grGreatBay_801F60C4");
    let gp = user_data as *mut Ground;

    let temp_r0 = (*coll).x34_flags.b1234() as i32;
    if temp_r0 == 1 || temp_r0 == 3 {
        if ground_kind == 1 {
            coll_x50 = (coll_x50 as f32 * (*attr()).floatfloor_landing_rate) as i32;
        }
        let jobj = (*gp).u.greatbay3.jobj;
        if !jobj.is_null() {
            let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
            let dx = (*coll).cur_pos.x - pos.x;
            let mut t = (if dx < 0.0 { -dx } else { dx }) / 15.0;
            if t > 1.0 {
                t = 1.0;
            }
            let rot_amount = t * (0.017453292f32 * fmadds(coll_x50 as f32, (*attr()).floatfloor_slant_mul, (*attr()).floatfloor_slant_add));
            let disp_amount = t * fmadds(coll_x50 as f32, (*attr()).floatfloor_slide_mul, (*attr()).floatfloor_slide_add);
            let y_amount = fmadds(coll_x50 as f32, (*attr()).floatfloor_down_mul, (*attr()).floatfloor_down_add);
            if dx < 0.0 {
                (*gp).u.greatbay3.translation.z += rot_amount;
                {
                    let max = (*attr()).floatfloor_slant_limit;
                    if (*gp).u.greatbay3.translation.z > max {
                        (*gp).u.greatbay3.translation.z = max;
                    }
                }
                (*gp).u.greatbay3.xD0 += disp_amount;
                {
                    let max = (*attr()).floatfloor_slide_limit;
                    if (*gp).u.greatbay3.xD0 > max {
                        (*gp).u.greatbay3.xD0 = max;
                    }
                }
            } else {
                (*gp).u.greatbay3.translation.z -= rot_amount;
                if (*gp).u.greatbay3.translation.z < -(*attr()).floatfloor_slant_limit {
                    (*gp).u.greatbay3.translation.z = -(*attr()).floatfloor_slant_limit;
                }
                (*gp).u.greatbay3.xD0 -= disp_amount;
                if (*gp).u.greatbay3.xD0 < -(*attr()).floatfloor_slide_limit {
                    (*gp).u.greatbay3.xD0 = -(*attr()).floatfloor_slide_limit;
                }
            }
            (*gp).u.greatbay3.xD4 -= y_amount;
            if (*gp).u.greatbay3.xD4 < -(*attr()).floatfloor_down_limit {
                (*gp).u.greatbay3.xD4 = -(*attr()).floatfloor_down_limit;
            }
        }
    }
}

/// 1F62F8
pub unsafe extern "C" fn grGreatBay_801F62F8(mut current: i32) -> i32 {
    melee_core::ct!("grGreatBay_801F62F8");
    let mut result: i32;
    let mut i: i32 = 0;

    loop {
        current += 0;
        let a = grGb_804D69E0.x0;
        let total = (*a).kame_dir_prob[0] as i32 + (*a).kame_dir_prob[1] as i32 + (*a).kame_dir_prob[3] as i32 + (*a).kame_dir_prob[2] as i32;
        let mut rand = zrandi(total);

        let a = grGb_804D69E0.x0;
        rand -= (*a).kame_dir_prob[2] as i32;
        if rand < 0 {
            result = 1;
        } else {
            rand -= (*a).kame_dir_prob[1] as i32;
            if 0 > rand {
                result = 2;
            } else {
                rand -= (*a).kame_dir_prob[0] as i32;
                if rand < 0 {
                    result = 0;
                } else {
                    rand -= (*a).kame_dir_prob[3] as i32;
                    if rand < 0 {
                        result = 3;
                    } else {
                        panic!("grgreatbay.c:1277: assertion failed (0)");
                    }
                }
            }
        }

        if current != result {
            return result;
        }
        let cont = i <= 1000;
        i += 1;
        if !cont {
            break;
        }
    }
    result = 0;
    result
}

/// 1F63F4
pub unsafe extern "C" fn grGreatBay_801F63F4(gobj: *mut Ground_GObj) -> i32 {
    melee_core::ct!("grGreatBay_801F63F4");
    let mut rot = grGb_803B81C8;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let jobj = ext::Ground_801C3FA4(gobj, 2);
    if jobj.is_null() {
        return 0;
    }
    ext::lb_8000B1CC(jobj, &mut rot, &mut pos);
    pos.z = 0.0;
    if pos.y <= ext::Stage_GetBlastZoneBottomOffset() {
        return 0;
    }
    if ext::HSD_Randf() > (*attr()).kame_item_prob {
        return 1;
    }

    let mut total: i32 = 0;
    let mut i = 0usize;
    while i < 10 {
        let item_id = (*attr()).items[i].kind;
        if item_id != -1 && ext::it_8026D324(item_id as i32) != 0 {
            total += (*attr()).items[i].weight as i32;
        }
        i += 1;
    }

    if total == 0 {
        return 1;
    }

    let mut selected: i32 = -1;
    let mut rand = zrandi(total);

    i = 0;
    while i < 10 {
        let item_id = (*attr()).items[i].kind;
        if item_id != -1 && ext::it_8026D324(item_id as i32) != 0 {
            rand -= (*attr()).items[i].weight as i32;
            if rand < 0 {
                selected = (*attr()).items[i].kind as i32;
                break;
            }
        }
        i += 1;
    }
    if selected == -1 {
        return 1;
    }

    let mut spawn: SpawnItem = grGb_803B81D4;
    spawn.kind = selected as ItemKind;
    spawn.prev_pos = pos;
    spawn.pos = pos;
    ext::Item_80268B18(&mut spawn);
    1
}

/// 1F660C
pub unsafe extern "C" fn grGreatBay_801F660C(gobj: *mut Ground_GObj) {
    melee_core::ct!("grGreatBay_801F660C");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let mut i: u32 = 0;
    while i < 7 {
        let jobj = ext::Ground_801C3FA4(gobj, grGb_803E4048[i as usize][1] as i32);
        assert!(!jobj.is_null(), "jobj");
        ext::lb_8000B1CC(jobj, null_mut(), &mut pos);
        ext::mpVtxSetPos(grGb_803E4048[i as usize][0] as i32, pos.x, pos.y);
        i += 1;
    }
}

/// 1F66A4
pub unsafe extern "C" fn grGreatBay_801F66A4() -> i32 {
    melee_core::ct!("grGreatBay_801F66A4");
    if ext::stage_info.grkind == Gr_Kind_GreatBay {
        let gobj = ext::Ground_GetMapGObj(1);
        if !gobj.is_null() {
            let gp = GET_GROUND(gobj);
            if !gp.is_null() {
                if (*gp).u.greatbay.xC4 != 0 {
                    return 1;
                }
            }
        }
    }
    0
}

/// 1F6708
pub unsafe extern "C" fn grGreatBay_801F6708(_unk: u32, gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grGreatBay_801F6708");
    let mut current = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut previous = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::ftLib_80086644(gobj, &mut current);
    ext::ftLib_80086684(gobj, &mut previous);
    if ext::ft_80087838(gobj) == 0 {
        if current.y < -5.0 {
            if previous.y > -5.0 {
                let arg = ext::ftLib_80086B80(gobj) / 10.0;
                current.y = -5.0;
                grGreatBay_801F67A4(&mut current, arg);
                ext::Ground_801C53EC(0x57E40);
            }
        }
    }

    0
}

/// 1F67A4
pub unsafe extern "C" fn grGreatBay_801F67A4(vec: *mut Vec3, arg8: f32) {
    melee_core::ct!("grGreatBay_801F67A4");
    let temp_r3 = ext::grLib_801C96F8(0x7537, 0x1E, vec);
    if !temp_r3.is_null() {
        let temp_r3_2 = (*temp_r3).appsrt;
        (*temp_r3_2).scale.x *= arg8;
        (*temp_r3_2).scale.y *= arg8;
        (*temp_r3_2).scale.z *= arg8;
    }
}

/// 1F6814
pub unsafe extern "C" fn grGreatBay_801F6814(a: *mut Vec3, _b: i32, joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grGreatBay_801F6814");
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint, null_mut(), &mut b);

    if (*a).y > b.y {
        1
    } else {
        0
    }
}
