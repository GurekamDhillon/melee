//! Port of src/melee/gr/gricemt.c (Icicle Mountain: the endlessly scrolling climb of ice fields, the ice-block
//! platforms with their break animations and the background scroll). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_gricemt.rs.
//!
//! Fused sites (tools/rust-port/fma_sites.py), each the gwtool `fmadds` (f64 product + f64 sum, one f32 rounding;
//! a subtraction fuses as `a * b + (-c)`): `grIceMt_801F993C` (two: `y * x4 + x40` and the outer
//! `y * x8 - (that)`), `grIceMt_801FA0BC` (`-20.0f * scale + frame`), `stageGObj8_GObjProc` (`3500.0f * scale - cur`
//! and `cur + 3500.0f * scale`) and `stageGObj10_GObjProc` (`cur * ratio - ratio * (0.5f * a * ratio)`).
//!
//! The constant-return callbacks that `gr/trivial.rs` already holds are re-exported.
//!
//! `GroundVars` is used through the same views as the C: `icemt` (the shorts: the x100 / x108 animation records
//! of the block maps 2 and 4, `x1A` / `x1C` of the field controller), `icemt1` (a block map), `icemt9` / `icemt10`
//! (the two field controllers) and `icemt_bg`. The C addresses the animation records as `(u8*) gp + 0x100` and
//! `&u.icemt1.x34[2]` (GameCube offsets); they are the `u.icemt.x100` / `x108` shorts here.
//!
//! `grIceMt_801F96E0` falls off the end of a float function in the C (its value is never read: the `y`
//! parameter of `grIceMt_801F9ACC` is unused); it returns 0.0.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::gricemt::{
    grIceMt_801F71E0, grIceMt_801FA8F8, grIceMt_801FA900, stageGObj0_Callback1, stageGObj1_Callback1, stageGObj2_Callback1,
    stageGObj3_Callback1, stageGObj4_Callback1, stageGObj5_Callback1, stageGObj6_Callback1, stageGObj8_Callback1,
    stageGObj9_Callback1,
};

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Ground_801C0498() -> f32;
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn Ground_801C1D6C(a0: u32);
    fn Ground_801C2D0C(a0: i32, a1: *mut HSD_JObj);
    fn Ground_801C3214(a0: i32) -> i32;
    fn Ground_801C3260(a0: i32) -> i32;
    fn Ground_801C32AC(a0: i32);
    fn Ground_801C39C0();
    fn Ground_801C3BB4();
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj;
    fn Ground_801C4A08(a0: *mut HSD_GObj);
    fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj);
    fn Ground_801C53EC(a0: u32);
    fn Ground_801C5740(a0: i32);
    fn Ground_801C5750();
    fn Ground_801C5764() -> i32;
    fn Ground_801C5840() -> i32;
    fn Ground_801C5940() -> i32;
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj;
    fn Ground_GetP1Fighter() -> *mut HSD_GObj;
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj;
    fn Ground_GetYakumonoParam() -> *mut c_void;
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32;
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32;
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjAddChild(a0: *mut HSD_JObj, a1: *mut HSD_JObj);
    fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj;
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randi(a0: i32) -> i32;
    fn Stage_80225194() -> u32;
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086EC0(a0: *mut HSD_GObj) -> i32;
    fn gm_801674C4(a0: i8, a1: u8, a2: i8, a3: i8, a4: Option<unsafe extern "C" fn(i32) -> i32>);
    fn grAnime_801C77FC(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn grAnime_801C78FC(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn grAnime_801C7A04(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32);
    fn grAnime_801C7B24(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32);
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32;
    fn grDatFiles_GetArchive() -> *mut UnkArchiveStruct;
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32);
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator;
    fn grMaterial_801C8CDC(a0: *mut HSD_GObj);
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj;
    fn grMaterial_801C8DE0(a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32);
    fn grMaterial_801C8E08(a0: *mut HSD_GObj);
    fn grMaterial_801C8E28(a0: *mut HSD_GObj);
    fn grMaterial_801C8E68(a0: *mut HSD_GObj, a1: u32);
    fn grZakoGenerator_801CADE0();
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj;
    fn grZakoGenerator_801CAEB0(a0: i32, a1: i32);
    fn grZakoGenerator_801CAF08();
    fn it_8026B294(a0: *mut HSD_GObj, a1: *mut Vec);
    fn memzero(a0: *mut c_void, a1: i32);
    fn mpJointGetCb1(a0: i32, a1: *mut Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>, a2: *mut *mut c_void);
    fn mpJointListAdd(a0: i32);
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>);
    fn mpLib_80057BC0(a0: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj { super::Ground_GetMapGObj(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
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

/// `struct grIceMt_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grIceMt_YakumonoParam {
    pub x0: i16,
    pub x2: i16,
    pub x4: i16,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: i16,
    pub x36: i16,
    pub x38: i16,
    pub x3A: u16,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4C: f32,
    pub x50: f32,
    pub x54: f32,
    pub x58: f32,
    pub x5C: f32,
    pub x60: f32,
    pub x64: f32,
    pub x68: f32,
    pub x6C: f32,
    pub x70: f32,
    pub x74: f32,
    pub x78: f32,
    pub x7C: f32,
    pub x80: f32,
    pub x84: f32,
    pub x88: f32,
    pub x8C: f32,
    pub x90: f32,
    pub x94: f32,
    pub ft_max_y: i16,
    pub x9E: i16,
    pub x9C: f32,
    pub xA0: f32,
    pub xA4: i16,
    pub xA6: i16,
    pub xA8: i16,
    pub field_ixs: *mut i16,
    pub xB0: *mut i16,
    pub xB4: *mut i16,
    pub xB8: i16,
    pub pad: i16,
    pub xBC: grZakoGenerator_SpawnDesc,
    pub xC0: f32,
    pub xC4: f32,
    pub xC8: f32,
    pub xCC: f32,
}

/// `IceMtRowData`: Ice Mountain row data - 12 bytes each.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IceMtRowData {
    pub gobj_id: i32,
    pub x4: f32,
    pub x8: f32,
}

/// `GrIceMtCb`: the next-field / next-delay chooser of `grIceMt_801FA364`
pub type GrIceMtCb = unsafe extern "C" fn(*mut Ground_GObj, *mut i32) -> i32;

pub static mut grIm_804D69E8: *mut HSD_GObj = null_mut();
pub static mut grIm_804D69EC: *mut HSD_GObj = null_mut();
pub static mut grIm_804D69F0: *mut HSD_GObj = null_mut();
/// `static struct grIceMt_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grIceMt_YakumonoParam = null_mut();

/// `grIm_803B8220`
pub static grIm_803B8220: [Vec3; 5] = [
    Vec3 { x: 0.0, y: 0.0, z: 0.0 },
    Vec3 { x: -8.0, y: 15.0, z: 0.0 },
    Vec3 { x: 8.0, y: 15.0, z: 0.0 },
    Vec3 { x: -24.0, y: 15.0, z: 0.0 },
    Vec3 { x: 24.0, y: 15.0, z: 0.0 },
];

const fn row(gobj_id: i32, x4: f32, x8: f32) -> IceMtRowData {
    IceMtRowData { gobj_id, x4, x8 }
}

/// `icemt_field`
pub static mut icemt_field: [IceMtRowData; 6] = [
    row(1, 180.0, -180.0),
    row(2, 190.0, -180.0),
    row(3, 190.0, -195.0),
    row(4, 195.0, -185.0),
    row(5, 190.0, -200.0),
    row(6, 180.0, -190.0),
];
/// `ICEMT_FIELD_MAX`
pub const ICEMT_FIELD_MAX: usize = 6;
/// `BLOCK_COLL_JOBJ_MAX`
pub const BLOCK_COLL_JOBJ_MAX: i32 = 20;

const fn gj(x: i16, y: i16, z: i16) -> GrJoint {
    GrJoint { x, y, z }
}

/// `grIm_803E40B0`
pub static mut grIm_803E40B0: [GrJoint; 195] = [
    gj(0, 1, 0), gj(1, 1, 0), gj(2, 1, 0), gj(3, 1, 0), gj(4, 1, 0), gj(5, 1, 0),
    gj(6, 1, 0), gj(7, 1, 0), gj(10, 1, 0), gj(11, 1, 0), gj(12, 1, 0), gj(13, 1, 0),
    gj(14, 1, 0), gj(15, 1, 0), gj(16, 1, 0), gj(17, 1, 0), gj(18, 1, 0), gj(19, 1, 0),
    gj(20, 1, 0), gj(21, 1, 0), gj(22, 1, 0), gj(23, 1, 0), gj(24, 1, 0), gj(25, 1, 0),
    gj(26, 1, 0), gj(27, 1, 0), gj(28, 1, 0), gj(29, 1, 0), gj(30, 1, 0), gj(31, 1, 0),
    gj(32, 2, 0), gj(33, 2, 0), gj(36, 2, 3), gj(39, 2, 0), gj(40, 2, 0), gj(41, 2, 0),
    gj(42, 2, 0), gj(43, 2, 0), gj(44, 2, 0), gj(45, 2, 0), gj(46, 2, 0), gj(47, 2, 0),
    gj(48, 2, 0), gj(49, 2, 0), gj(50, 2, 0), gj(51, 2, 0), gj(52, 2, 0), gj(53, 2, 0),
    gj(54, 2, 0), gj(55, 2, 0), gj(56, 2, 0), gj(57, 2, 0), gj(58, 2, 0), gj(59, 2, 0),
    gj(60, 2, 0), gj(61, 2, 0), gj(62, 2, 0), gj(63, 2, 0), gj(64, 2, 0), gj(65, 2, 0),
    gj(66, 2, 0), gj(67, 3, 0), gj(68, 3, 0), gj(69, 3, 0), gj(74, 3, 0), gj(75, 3, 0),
    gj(76, 3, 0), gj(77, 3, 0), gj(78, 3, 0), gj(79, 3, 0), gj(80, 3, 0), gj(81, 3, 0),
    gj(82, 3, 0), gj(83, 3, 0), gj(84, 3, 0), gj(85, 3, 0), gj(86, 3, 0), gj(87, 3, 0),
    gj(88, 3, 0), gj(89, 3, 0), gj(90, 3, 0), gj(91, 3, 0), gj(92, 3, 0), gj(93, 3, 0),
    gj(94, 3, 0), gj(95, 3, 0), gj(96, 3, 0), gj(97, 3, 0), gj(98, 3, 0), gj(99, 3, 0),
    gj(100, 3, 0), gj(101, 3, 0), gj(102, 4, 0), gj(103, 4, 0), gj(104, 4, 0), gj(105, 4, 0),
    gj(106, 4, 0), gj(107, 4, 0), gj(108, 4, 0), gj(109, 4, 0), gj(110, 4, 0), gj(111, 4, 0),
    gj(113, 4, 2), gj(114, 4, 3), gj(115, 4, 4), gj(118, 4, 0), gj(119, 4, 0), gj(120, 4, 0),
    gj(121, 4, 0), gj(122, 4, 0), gj(123, 4, 0), gj(124, 4, 0), gj(125, 4, 0), gj(126, 4, 0),
    gj(127, 4, 0), gj(128, 4, 0), gj(129, 4, 0), gj(130, 4, 0), gj(131, 4, 0), gj(132, 4, 0),
    gj(133, 4, 0), gj(134, 4, 0), gj(135, 4, 0), gj(136, 4, 0), gj(137, 4, 0), gj(138, 4, 0),
    gj(139, 4, 0), gj(140, 4, 0), gj(141, 4, 0), gj(142, 4, 0), gj(143, 5, 0), gj(144, 5, 0),
    gj(145, 5, 0), gj(146, 5, 0), gj(152, 5, 0), gj(153, 5, 0), gj(154, 5, 0), gj(155, 5, 0),
    gj(156, 5, 0), gj(157, 5, 0), gj(158, 5, 0), gj(159, 5, 0), gj(160, 5, 0), gj(161, 5, 0),
    gj(162, 5, 0), gj(163, 5, 0), gj(164, 5, 0), gj(165, 5, 0), gj(166, 5, 0), gj(167, 5, 0),
    gj(168, 5, 0), gj(169, 5, 0), gj(170, 5, 0), gj(171, 5, 0), gj(172, 5, 0), gj(173, 5, 0),
    gj(174, 5, 0), gj(175, 5, 0), gj(176, 5, 0), gj(177, 5, 0), gj(178, 5, 0), gj(179, 5, 0),
    gj(180, 6, 0), gj(181, 6, 0), gj(182, 6, 0), gj(183, 6, 0), gj(184, 6, 0), gj(189, 6, 0),
    gj(190, 6, 0), gj(191, 6, 0), gj(192, 6, 0), gj(193, 6, 0), gj(194, 6, 0), gj(195, 6, 0),
    gj(196, 6, 0), gj(197, 6, 0), gj(198, 6, 0), gj(199, 6, 0), gj(200, 6, 0), gj(201, 6, 0),
    gj(202, 6, 0), gj(203, 6, 0), gj(204, 6, 0), gj(205, 6, 0), gj(206, 6, 0), gj(207, 6, 0),
    gj(208, 6, 0), gj(209, 6, 0), gj(210, 6, 0), gj(211, 6, 0), gj(212, 6, 0), gj(213, 6, 0),
    gj(214, 6, 0), gj(215, 6, 0), gj(216, 6, 0),
];

/// `grIm_803E4544`: 0..=216
pub static mut grIm_803E4544: [i16; 217] = {
    let mut a = [0i16; 217];
    let mut i = 0;
    while i < 217 {
        a[i] = i as i16;
        i += 1;
    }
    a
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

/// `stage_callbacks`
pub static mut stage_callbacks: [StageCallbacks; 11] = [
    cbs(Some(stageGObj0_OnInit), Some(stageGObj0_Callback1), Some(stageGObj0_GObjProc), Some(stageGObj0_Callback3), 0),
    cbs(Some(stageGObj1_OnInit), Some(stageGObj1_Callback1), Some(stageGObj1_GObjProc), Some(stageGObj1_Callback3), 0),
    cbs(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cbs(Some(stageGObj3_OnInit), Some(stageGObj3_Callback1), Some(stageGObj3_GObjProc), Some(stageGObj3_Callback3), 0),
    cbs(Some(stageGObj4_OnInit), Some(stageGObj4_Callback1), Some(stageGObj4_GObjProc), Some(stageGObj4_Callback3), 0),
    cbs(Some(stageGObj5_OnInit), Some(stageGObj5_Callback1), Some(stageGObj5_GObjProc), Some(stageGObj5_Callback3), 0),
    cbs(Some(stageGObj6_OnInit), Some(stageGObj6_Callback1), Some(stageGObj6_GObjProc), Some(stageGObj6_Callback3), 0),
    cbs(None, None, None, None, 0),
    cbs(Some(stageGObj8_OnInit), Some(stageGObj8_Callback1), Some(stageGObj8_GObjProc), Some(stageGObj8_Callback3),
        (1 << 30) | (1 << 31)),
    cbs(Some(stageGObj9_OnInit), Some(stageGObj9_Callback1), Some(stageGObj9_GObjProc), Some(stageGObj9_Callback3), 0),
    cbs(Some(stageGObj10_OnInit), Some(stageGObj10_Callback1), Some(stageGObj10_GObjProc), Some(stageGObj10_Callback3), 0),
];

/// `grIm_StageData`
pub static mut grIm_StageData: StageData = StageData {
    grkind: Gr_Kind_Icemt,
    callbacks: unsafe { &raw mut stage_callbacks as *mut StageCallbacks },
    data1: b"/GrIm.dat\0".as_ptr() as *mut i8,
    on_init: Some(grIceMt_801F686C),
    on_demo_init: Some(grIceMt_801F6868),
    on_load: Some(grIceMt_801F7080),
    on_start: Some(grIceMt_801F71DC),
    callback4: Some(grIceMt_801F71E0),
    on_touch_line: Some(grIceMt_801FA8F8),
    on_check_shadow_render: Some(grIceMt_801FA900),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grIm_803E40B0 as *mut GrJoint },
    joint_count: 195,
};

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

const _: () = assert!(core::mem::offset_of!(grIceMt_GroundVars, x100) == 0x3C);
const _: () = assert!(core::mem::offset_of!(grIceMt_GroundVars, x108) == 0x44);

/// the animation records start at `u.icemt.x100` (gp + 0x100 on the GameCube) and run on into `x108`: the pointer
/// keeps the whole view's provenance
#[inline(always)]
unsafe fn x100_ptr(gp: *mut Ground) -> *mut i16 {
    (&raw mut (*gp).u.icemt as *mut i16).add(0x3C / 2)
}

#[inline(always)]
unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `static inline rand_zero`
#[inline(always)]
pub unsafe fn rand_zero(n: i32) -> bool {
    (if n != 0 { ext::HSD_Randi(n) } else { 0 }) == 0
}

/// `static inline grIceMt_GetRandomIndex`
#[inline(always)]
pub unsafe fn grIceMt_GetRandomIndex(max: i32, list: *const i32) -> i32 {
    *list.add((if max != 0 { ext::HSD_Randi(max) } else { 0 }) as usize)
}

/// `static inline grIceMt_GetRandomTimer`
#[inline(always)]
pub unsafe fn grIceMt_GetRandomTimer(out: *mut i32) {
    let mut a = (*yakumono_param).x36 as i32;
    let b = (*yakumono_param).x38 as i32;
    if a > b {
        let d = a - b;
        a = b + if d != 0 { ext::HSD_Randi(d) } else { 0 };
    } else if a < b {
        let d = b - a;
        a = a + if d != 0 { ext::HSD_Randi(d) } else { 0 };
    }
    *out = a;
}

/// 1F6868
pub unsafe extern "C" fn grIceMt_801F6868(_id: i32) {}

/// 1F686C
pub unsafe extern "C" fn grIceMt_801F686C() {
    let mut field30: i32 = 0;
    let mut field29: i32 = 0;
    let mut field28: i32 = 0;

    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grIceMt_YakumonoParam;
    ext::stage_info.unk8C.set_b4(1);
    ext::stage_info.unk8C.set_b5(0);
    setupStageCallbacks(0);
    setupStageCallbacks(8);
    ext::Ground_801C3260(1);
    ext::Ground_801C3260(2);
    ext::Ground_801C3260(3);
    ext::Ground_801C3260(4);
    ext::Ground_801C3260(5);
    ext::Ground_801C3260(6);

    if ext::Stage_80225194() == 76 {
        let fi = (*yakumono_param).field_ixs;

        let mut i = 0usize;
        while i < ICEMT_FIELD_MAX {
            let id = icemt_field[i].gobj_id;
            if *fi as i32 == id || *fi.add(1) as i32 == id {
                i += 1;
                continue;
            }
            field30 = icemt_field[i].gobj_id;
            break;
        }
        assert!(i < ICEMT_FIELD_MAX, "gricemt.c:600: i<ICEMT_FIELD_MAX");

        i = 0;
        while i < ICEMT_FIELD_MAX {
            let id = icemt_field[i].gobj_id;
            if *fi as i32 == id || *fi.add(1) as i32 == id || field30 == id {
                i += 1;
                continue;
            }
            field29 = icemt_field[i].gobj_id;
            break;
        }
        assert!(i < ICEMT_FIELD_MAX, "gricemt.c:609: i<ICEMT_FIELD_MAX");

        i = 0;
        while i < ICEMT_FIELD_MAX {
            let id = icemt_field[i].gobj_id;
            if *fi as i32 == id || *fi.add(1) as i32 == id || field30 == id || field29 == id {
                i += 1;
                continue;
            }
            field28 = icemt_field[i].gobj_id;
            break;
        }
        assert!(i < ICEMT_FIELD_MAX, "gricemt.c:619: i<ICEMT_FIELD_MAX");

        let mut y_pos = ext::Ground_801C0498();
        y_pos = -20.0f32 * y_pos;
        let a = grIceMt_801F993C(icemt_field[*fi as usize].gobj_id, icemt_field[*fi.add(1) as usize].gobj_id);
        y_pos = y_pos + a;
        let a = grIceMt_801F993C(icemt_field[*fi.add(1) as usize].gobj_id, icemt_field[field30 as usize].gobj_id);
        let y_pos2 = y_pos + a;
        let a = grIceMt_801F993C(icemt_field[field30 as usize].gobj_id, icemt_field[field29 as usize].gobj_id);
        let y_pos3 = y_pos2 + a;
        let a = grIceMt_801F993C(icemt_field[field29 as usize].gobj_id, icemt_field[field28 as usize].gobj_id);
        let y_pos4 = y_pos3 + a;

        let mut gobj = setupStageCallbacks(icemt_field[*fi.add(2) as usize].gobj_id);
        assert!(!gobj.is_null(), "gricemt.c:636: gobj");
        let mut jobj = GET_JOBJ(gobj);
        assert!(!jobj.is_null(), "gricemt.c:637: jobj");
        HSD_JObjSetTranslateY(&mut *jobj, y_pos2, &mut Dirty);
        grIm_804D69E8 = gobj;

        gobj = setupStageCallbacks(icemt_field[*fi.add(3) as usize].gobj_id);
        assert!(!gobj.is_null(), "gricemt.c:641: gobj");
        jobj = GET_JOBJ(gobj);
        assert!(!jobj.is_null(), "gricemt.c:642: jobj");
        HSD_JObjSetTranslateY(&mut *jobj, y_pos3, &mut Dirty);
        grIm_804D69EC = gobj;

        gobj = setupStageCallbacks(icemt_field[*fi.add(4) as usize].gobj_id);
        assert!(!gobj.is_null(), "gricemt.c:646: gobj");
        jobj = GET_JOBJ(gobj);
        assert!(!jobj.is_null(), "gricemt.c:647: jobj");
        HSD_JObjSetTranslateY(&mut *jobj, y_pos4, &mut Dirty);
        grIm_804D69F0 = gobj;

        setupStageCallbacks(10);
    } else {
        setupStageCallbacks(9);
        grIm_804D69E8 = null_mut();
        grIm_804D69EC = null_mut();
        grIm_804D69F0 = null_mut();
    }
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// 1F7080
pub unsafe extern "C" fn grIceMt_801F7080() {
    if !grIm_804D69E8.is_null() {
        ext::Ground_801C4A08(grIm_804D69E8);
    }
    if !grIm_804D69EC.is_null() {
        ext::Ground_801C4A08(grIm_804D69EC);
    }
    if !grIm_804D69F0.is_null() {
        ext::Ground_801C4A08(grIm_804D69F0);
    }
    for id in 1..=6 {
        let gobj = ext::Ground_GetMapGObj(id);
        if !gobj.is_null() {
            ext::Ground_UpdateMapColl(gobj);
        }
    }
    if ext::Stage_80225194() == 76 {
        ext::grZakoGenerator_801CAE04(&raw mut (*yakumono_param).xBC);
        if rand_zero((*yakumono_param).xB8 as i32) {
            let a = ext::Ground_801C5840();
            let b = ext::Ground_801C5940();
            ext::grZakoGenerator_801CAEB0(a, b);
        }

        ext::grZakoGenerator_801CADE0();
    } else {
        ext::grZakoGenerator_801CAE04(null_mut());
    }
    let mut x = grIm_803B8220[0];
    ext::grLib_801C96F8(0x7530, 0x1E, &mut x);
}

/// 1F71DC
pub unsafe extern "C" fn grIceMt_801F71DC() {}

/// 1F71E8
pub unsafe extern "C" fn setupStageCallbacks(gobj_id: i32) -> *mut Ground_GObj {
    let callbacks = &raw mut stage_callbacks[gobj_id as usize];
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
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 860, gobj_id);
    }

    gobj
}

/// 1F72D4
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    let mut jobj = ext::Ground_801C3FA4(gobj, 18);
    if !jobj.is_null() {
        let pos = grIm_803B8220[1];
        HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);
    }
    jobj = ext::Ground_801C3FA4(gobj, 19);
    if !jobj.is_null() {
        let pos = grIm_803B8220[2];
        HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);
    }
    jobj = ext::Ground_801C3FA4(gobj, 20);
    if !jobj.is_null() {
        let pos = grIm_803B8220[3];
        HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);
    }
    jobj = ext::Ground_801C3FA4(gobj, 21);
    if !jobj.is_null() {
        let pos = grIm_803B8220[4];
        HSD_JObjSetTranslate(&mut *jobj, &pos, &mut Dirty);
    }
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// 1F75E4
pub unsafe extern "C" fn stageGObj0_GObjProc(_arg0: *mut Ground_GObj) {}

/// 1F75E8
pub unsafe extern "C" fn stageGObj0_Callback3(_arg0: *mut Ground_GObj) {}

/// 1F75EC
pub unsafe extern "C" fn fn_801F75EC(gobj: *mut Ground_GObj) {
    (*GET_GROUND(gobj)).u.icemt9.x14 = 0;
}

/// 1F75FC
pub unsafe extern "C" fn stageGObj9_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    ext::memzero(&raw mut (*gp).u.icemt9.x18 as *mut c_void, core::mem::size_of_val(&(*gp).u.icemt9.x18) as i32);
    ext::memzero(&raw mut (*gp).u.icemt9.x30 as *mut c_void, core::mem::size_of_val(&(*gp).u.icemt9.x30) as i32);

    let mut i_var1: u32;
    loop {
        i_var1 = ext::HSD_Randi(6) as u32;
        let val = (*gp).u.icemt9.x30[i_var1 as usize];
        if val == 0 {
            break;
        }
    }
    (*gp).u.icemt9.x30[i_var1 as usize] = (*yakumono_param).x0;
    (*gp).u.icemt9.x0.ids.under = icemt_field[i_var1 as usize].gobj_id as i16;
    loop {
        i_var1 = ext::HSD_Randi(6) as u32;
        let val = (*gp).u.icemt9.x30[i_var1 as usize];
        if val == 0 {
            break;
        }
    }
    (*gp).u.icemt9.x30[i_var1 as usize] = (*yakumono_param).x0;
    (*gp).u.icemt9.x0.ids.upper = icemt_field[i_var1 as usize].gobj_id as i16;
    grIceMt_801FA0BC(&raw mut (*gp).u.icemt9.x0.ids);
    (*gp).u.icemt9.x16 = 0;
    (*gp).u.icemt9.x0.state.phase = 0;
    (*gp).u.icemt9.x0.state.burst_count = 0;
    (*gp).u.icemt9.x0.state.delay = 0;
    (*gp).u.icemt9.x0.state.lerp_count = 0;
    (*gp).u.icemt9.x0.state.idx = -1;
    (*gp).u.icemt9.x0.state.cur = 0.0;
    (*gp).u.icemt9.x14 = 1;
    ext::Ground_801C10B8(gobj, Some(fn_801F75EC));
}

/// 1F7728
pub unsafe extern "C" fn stageGObj9_GObjProc(gobj: *mut Ground_GObj) {
    let mut y: f32 = 0.0;
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.icemt9.x14 == 0 {
        grIceMt_801FA364(&raw mut (*gp).u.icemt9.x0.state, &mut y, fn_801F8E58, gobj);
        let ids = &raw mut (*gp).u.icemt9.x0.ids;
        let v = grIceMt_801F96E0(ids, -y);
        grIceMt_801F9ACC(ids, v, Some(fn_801F9038), gobj);
        grIceMt_801F9668(y);
    }
}

/// 1F77AC
pub unsafe extern "C" fn stageGObj9_Callback3(_gobj: *mut Ground_GObj) {}

/// 1F77B0
pub unsafe extern "C" fn stageGObj10_UnkGroundCallback(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let mut cb: mpLib_JointCollisionCallback = None;
    let mut user_data_out: *mut c_void = null_mut();

    (*gp).u.icemt10.set_x14_b0(0);

    for i in 0..grIm_803E4544.len() {
        ext::mpJointGetCb1(grIm_803E4544[i] as i32, &mut cb, &mut user_data_out);
        if cb.is_none() {
            ext::mpJointSetCb1(grIm_803E4544[i] as i32, gp as *mut c_void, Some(onJointCollision));
        }
    }
}

/// 1F785C
pub unsafe extern "C" fn stageGObj10_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let fi = (*yakumono_param).field_ixs;

    (*gp).u.icemt10.x0.ids.under = icemt_field[*fi.add(1) as usize].gobj_id as i16;
    (*gp).u.icemt10.x0.ids.upper = icemt_field[*fi as usize].gobj_id as i16;

    grIceMt_801FA0BC(&raw mut (*gp).u.icemt10.x0.ids);

    (*gp).u.icemt10.x18 = 0;
    (*gp).u.icemt10.x1C = 2;
    (*gp).u.icemt10.x1A = 0;

    (*gp).u.icemt10.set_x14_b1(0);
    (*gp).u.icemt10.set_x14_b2(0);
    (*gp).u.icemt10.set_x14_b3(0);
    (*gp).u.icemt10.set_x14_b4(0);

    (*gp).u.icemt10.x16 = 0;
    (*gp).u.icemt10.x20 = 0.0;

    (*gp).u.icemt10.x0.state.phase = 0;
    (*gp).u.icemt10.x0.state.burst_count = 0;
    (*gp).u.icemt10.x0.state.delay = 0;
    (*gp).u.icemt10.x0.state.lerp_count = 0;
    (*gp).u.icemt10.x0.state.idx = -1;
    (*gp).u.icemt10.x0.state.cur = 0.0;

    (*gp).u.icemt10.set_x14_b0(1);

    ext::Ground_801C10B8(gobj, Some(stageGObj10_UnkGroundCallback));
}

/// 1F796C
pub unsafe extern "C" fn stageGObj10_Callback1(gobj: *mut Ground_GObj) -> i32 {
    let gp = (*gobj).user_data as *mut Ground;

    if (*gp).u.icemt10.x0.ids.under != -1 {
        let mgobj = ext::Ground_GetMapGObj((*gp).u.icemt10.x0.ids.under as i32);
        assert!(!mgobj.is_null(), "gricemt.c:1141: mgobj");
        ext::Ground_801C4A08(mgobj);
    }
    if (*gp).u.icemt10.x0.ids.upper != -1 {
        let mgobj = ext::Ground_GetMapGObj((*gp).u.icemt10.x0.ids.upper as i32);
        assert!(!mgobj.is_null(), "gricemt.c:1146: mgobj");
        ext::Ground_801C4A08(mgobj);
    }
    stageGObj10_OnInit(gobj);
    (*gp).u.icemt10.set_x14_b0(0);
    0
}

/// 1F7A2C
pub unsafe extern "C" fn stageGObj10_GObjProc(arg0: *mut Ground_GObj) {
    let gp = (*arg0).user_data as *mut Ground;
    let mut sp30: f32 = 0.0;
    let mut sp24: Vec3 = core::mem::zeroed();
    let mut var_r30: i32 = 0;

    if (*gp).u.icemt10.x14_b0() != 0 {
        return;
    }
    if (*gp).u.icemt10.x14_b2() == 0 {
        var_r30 = grIceMt_801FA364(&raw mut (*gp).u.icemt9.x0.state, &mut sp30, fn_801F9150, arg0) as i32;
        if (*gp).u.icemt10.x14_b4() != 0 {
            let fighter_gobj = ext::Ground_GetP1Fighter();
            if !fighter_gobj.is_null() {
                ext::ftLib_80086644(fighter_gobj, &mut sp24);
                if (*gp).u.icemt10.x14_b1() == 0
                    && ext::ftLib_80086EC0(fighter_gobj) == 0
                    && sp24.y > (*yakumono_param).ft_max_y as f32
                {
                    (*gp).u.icemt10.set_x14_b1(1);
                    (*gp).u.icemt10.x16 = (*yakumono_param).xA4;
                }
            }
        }
        if (*gp).u.icemt10.x14_b1() != 0 {
            (*gp).u.icemt10.x20 += (*yakumono_param).xA0;
            if (*gp).u.icemt10.x20 > (*yakumono_param).x9C {
                (*gp).u.icemt10.x20 = (*yakumono_param).x9C;
            }
            (*gp).u.icemt10.x16 = (*gp).u.icemt10.x16.wrapping_sub(1);
            if (*gp).u.icemt10.x16 == 0 {
                (*gp).u.icemt10.set_x14_b1(0);
            }
        } else {
            (*gp).u.icemt10.x20 -= (*yakumono_param).xA0;
            if (*gp).u.icemt10.x20 < 0.0 {
                (*gp).u.icemt10.x20 = 0.0;
            }
        }
        sp30 += (*gp).u.icemt10.x20;
    } else {
        let mgobj = ext::Ground_GetMapGObj((*gp).u.icemt10.x0.ids.upper as i32);
        assert!(!mgobj.is_null(), "gricemt.c:1219: mgobj");
        let jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
        assert!(!jobj.is_null(), "gricemt.c:1221: jobj");
        let dist = HSD_JObjGetTranslationY(&*jobj) - (*yakumono_param).xA8 as f32;
        if dist < 0.0 {
            (*gp).u.icemt10.x0.state.cur = 0.0;
        } else {
            let ratio = (*gp).u.icemt10.x0.state.cur / (*yakumono_param).xA0;
            let sub = ratio * (0.5f32 * (*yakumono_param).xA0 * ratio);
            if fmadds((*gp).u.icemt10.x0.state.cur, ratio, -sub) > dist {
                (*gp).u.icemt10.x0.state.cur -= (*yakumono_param).xA0;
                if (*gp).u.icemt10.x0.state.cur < (*yakumono_param).xA0 {
                    (*gp).u.icemt10.x0.state.cur = (*yakumono_param).xA0;
                }
            }
        }
        sp30 = (*gp).u.icemt10.x0.state.cur;
    }
    let ids = &raw mut (*gp).u.icemt10.x0.ids;
    let v = grIceMt_801F96E0(ids, -sp30);
    grIceMt_801F9ACC(ids, v, Some(fn_801F91A8), arg0);
    grIceMt_801F9668(sp30);
    if (*gp).u.icemt10.x0.ids.under == -1 {
        (*gp).u.icemt10.set_x14_b2(1);
    }
    if (*gp).u.icemt10.x14_b3() == 0 {
        if var_r30 != 0 && (*gp).u.icemt.x1A == (*yakumono_param).xA6 {
            (*gp).u.icemt10.set_x14_b3(1);
            ext::Ground_801C5750();
            ext::gm_801674C4(14, 2, 2, 0, Some(fn_801FA4CC));
            ext::grZakoGenerator_801CAF08();
        }
    } else {
        let r = ext::Ground_801C5764();
        match r {
            2 => {}
            1 => ext::Ground_801C1D6C(16),
            _ => {}
        }
    }
    (*gp).u.icemt10.set_x14_b4(0);
}

/// 1F7D90
pub unsafe extern "C" fn stageGObj10_Callback3(_gobj: *mut Ground_GObj) {}

/// 1F7D94
pub unsafe extern "C" fn stageGObj1_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let mut sp14: [i16; 2] = [1, 2];
    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(gobj, 3);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(gobj, 4);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(gobj, 5);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(gobj, 6);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(gobj, 18);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(gobj, 11);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(gobj, 12);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(gobj, 13);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(gobj, 14);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(gobj, 15);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(gobj, 16);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(gobj, 17);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(gobj, sp14.as_mut_ptr(), 2, (*gp).u.icemt1.x34.as_mut_ptr());
}

/// 1F7EE8
pub unsafe extern "C" fn stageGObj1_GObjProc(arg0: *mut Ground_GObj) {
    grIceMt_801F98A8(arg0);
    ext::Ground_UpdateMapColl(arg0);
}

/// the Callback3 shared by the block maps: destroys the first `n` material items
#[inline(always)]
unsafe fn destroy_items(gobj: *mut Ground_GObj, n: usize) {
    let gp = (*gobj).user_data as *mut Ground;
    for i in 0..n {
        if !(*gp).u.icemt1.x34[i].is_null() {
            ext::grMaterial_801C8CDC((*gp).u.icemt1.x34[i]);
        }
    }
}

/// 1F7F1C
pub unsafe extern "C" fn stageGObj1_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 2);
}

/// 1F7F70
pub unsafe extern "C" fn stageGObj2_OnInit(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let mut sp14: [i16; 2] = [1, 2];
    ext::Ground_InitMapColl((*arg0).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(arg0, 0, 7);
    let jobj = ext::Ground_801C3FA4(arg0, 4);
    let jobj2 = ext::Ground_801C3FA4(arg0, 5);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(arg0, 6);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(arg0, 7);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(arg0, 8);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(arg0, 9);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(arg0, 21);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(arg0, 14);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(arg0, 15);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(arg0, 16);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(arg0, 17);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(arg0, 18);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(arg0, 19);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(arg0, 20);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(arg0, sp14.as_mut_ptr(), 2, (*gp).u.icemt1.x34.as_mut_ptr());
    let a = grIceMt_801FA500(arg0, jobj);
    grIceMt_801F91EC(arg0, x100_ptr(gp), a, -1, 0x25, 0x109, 0x27E, Some(fn_801F9338));
    let a = grIceMt_801FA500(arg0, jobj2);
    grIceMt_801F91EC(arg0, x100_ptr(gp).add(4 + 3), a, -1, 38, 265, 638, Some(fn_801F9448));
}

/// 1F815C
pub unsafe extern "C" fn stageGObj2_GObjProc(param1: *mut Ground_GObj) {
    let gp = GET_GROUND(param1);
    grIceMt_801F929C(param1, x100_ptr(gp) as *mut c_void);
    grIceMt_801F929C(param1, x100_ptr(gp).add(4 + 3) as *mut c_void);
    grIceMt_801F98A8(param1);
    ext::Ground_UpdateMapColl(param1);
}

/// 1F81B4
pub unsafe extern "C" fn stageGObj2_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 2);
}

/// 1F8208
pub unsafe extern "C" fn stageGObj3_OnInit(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let mut sp14: [i16; 4] = [1, 2, 3, 4];
    ext::Ground_InitMapColl((*arg0).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(arg0, 0, 7);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(arg0, 5);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(arg0, 6);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(arg0, 7);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(arg0, 8);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(arg0, 21);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(arg0, 14);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(arg0, 15);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(arg0, 16);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(arg0, 17);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(arg0, 18);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(arg0, 19);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(arg0, 20);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(arg0, sp14.as_mut_ptr(), 4, (*gp).u.icemt1.x34.as_mut_ptr());
}

/// 1F8364
pub unsafe extern "C" fn stageGObj3_GObjProc(arg0: *mut Ground_GObj) {
    grIceMt_801F98A8(arg0);
    ext::Ground_UpdateMapColl(arg0);
}

/// 1F8398
pub unsafe extern "C" fn stageGObj3_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 4);
}

/// 1F83EC
pub unsafe extern "C" fn stageGObj4_OnInit(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let mut sp14: [i16; 4] = [1, 2, 3, 4];
    ext::Ground_801C0498();
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(arg0, 0, 7);
    let jobj2 = ext::Ground_801C3FA4(arg0, 7);
    let jobj3 = ext::Ground_801C3FA4(arg0, 5);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(arg0, 8);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(arg0, 9);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(arg0, 10);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(arg0, 11);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(arg0, 23);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(arg0, 16);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(arg0, 17);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(arg0, 18);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(arg0, 19);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(arg0, 20);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(arg0, 21);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(arg0, 22);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(arg0, sp14.as_mut_ptr(), 4, (*gp).u.icemt1.x34.as_mut_ptr());
    let r = grIceMt_801FA500(arg0, jobj3);
    let a = grIceMt_801FA500(arg0, jobj2);
    grIceMt_801F91EC(arg0, x100_ptr(gp).add(4), a, r, 117, 265, 638, Some(fn_801F9558));
}

/// 1F85C4
pub unsafe extern "C" fn stageGObj4_GObjProc(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    grIceMt_801F929C(gobj, x100_ptr(gp).add(4) as *mut c_void);
    grIceMt_801F98A8(gobj);
    ext::Ground_UpdateMapColl(gobj);
}

/// 1F8608
pub unsafe extern "C" fn stageGObj4_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 4);
}

/// 1F865C
pub unsafe extern "C" fn stageGObj5_OnInit(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let mut sp14: [i16; 5] = [1, 2, 3, 4, 5];
    ext::Ground_InitMapColl((*arg0).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(arg0, 0, 7);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(arg0, 6);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(arg0, 7);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(arg0, 8);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(arg0, 9);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(arg0, 22);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(arg0, 15);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(arg0, 16);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(arg0, 17);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(arg0, 18);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(arg0, 19);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(arg0, 20);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(arg0, 21);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(arg0, sp14.as_mut_ptr(), 5, (*gp).u.icemt1.x34.as_mut_ptr());
}

/// 1F87C8
pub unsafe extern "C" fn stageGObj5_GObjProc(param1: *mut Ground_GObj) {
    grIceMt_801F98A8(param1);
    ext::Ground_UpdateMapColl(param1);
}

/// 1F87FC: destroys the HSD_GObj* stored in the first five block slots
pub unsafe extern "C" fn stageGObj5_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 5);
}

/// 1F8850
pub unsafe extern "C" fn stageGObj6_OnInit(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let mut sp14: [i16; 4] = [1, 2, 3, 4];
    ext::Ground_InitMapColl((*arg0).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(arg0, 0, 7);
    (*gp).u.icemt1.x4 = ext::Ground_801C3FA4(arg0, 5);
    (*gp).u.icemt1.x8 = ext::Ground_801C3FA4(arg0, 6);
    (*gp).u.icemt1.xC = ext::Ground_801C3FA4(arg0, 7);
    (*gp).u.icemt1.x10 = ext::Ground_801C3FA4(arg0, 8);
    (*gp).u.icemt1.x14 = ext::Ground_801C3FA4(arg0, 21);
    (*gp).u.icemt1.x18 = ext::Ground_801C3FA4(arg0, 14);
    (*gp).u.icemt1.x1C = ext::Ground_801C3FA4(arg0, 15);
    (*gp).u.icemt1.x20 = ext::Ground_801C3FA4(arg0, 16);
    (*gp).u.icemt1.x24 = ext::Ground_801C3FA4(arg0, 17);
    (*gp).u.icemt1.x28 = ext::Ground_801C3FA4(arg0, 18);
    (*gp).u.icemt1.x2C = ext::Ground_801C3FA4(arg0, 19);
    (*gp).u.icemt1.x30 = ext::Ground_801C3FA4(arg0, 20);
    (*gp).u.icemt1.set_x0_b0(0);
    grIceMt_801F8CDC(arg0, sp14.as_mut_ptr(), 4, (*gp).u.icemt1.x34.as_mut_ptr());
}

/// 1F89AC
pub unsafe extern "C" fn stageGObj6_GObjProc(param1: *mut Ground_GObj) {
    grIceMt_801F98A8(param1);
    ext::Ground_UpdateMapColl(param1);
}

/// 1F89E0
pub unsafe extern "C" fn stageGObj6_Callback3(gobj: *mut Ground_GObj) {
    destroy_items(gobj, 4);
}

/// 1F8A34
pub unsafe extern "C" fn stageGObj8_OnInit(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C77FC(gobj, 0, 7);
    let j1 = ext::Ground_801C3FA4(gobj, 1);
    let j2 = ext::Ground_801C3FA4(gobj, 2);
    let j3 = ext::Ground_801C3FA4(gobj, 3);
    let j4 = ext::Ground_801C3FA4(gobj, 4);
    let j5 = ext::Ground_801C3FA4(gobj, 5);
    let j6 = ext::Ground_801C3FA4(gobj, 6);
    ext::Ground_801C4E70(j1, j2, j3, j4, j5, j6);
    (*gp).u.icemt_bg.x0 = 0.0;
    (*gp).x11_flags.set_b012(2);
}

/// 1F8B10
pub unsafe extern "C" fn stageGObj8_GObjProc(arg0: *mut Ground_GObj) {
    let gp = GET_GROUND(arg0);
    let mul = 0.3f32 * (*gp).u.icemt_bg.x0;
    let jobj = ext::Ground_801C3FA4(arg0, 8);
    assert!(!jobj.is_null(), "gricemt.c:1935: jobj");
    let mut cur = HSD_JObjGetTranslationY(&*jobj);
    cur = cur - mul;
    if cur > 0.5f32 * (3500.0f32 * ext::Ground_801C0498()) {
        let g = ext::Ground_801C0498();
        cur = -fmadds(3500.0, g, -cur);
    } else {
        let g = ext::Ground_801C0498();
        if cur < 0.5f32 * -(3500.0f32 * g) {
            let g = ext::Ground_801C0498();
            cur = fmadds(3500.0, g, cur);
        }
    }
    HSD_JObjSetTranslateY(&mut *jobj, cur, &mut Dirty);
}

/// 1F8C60
pub unsafe extern "C" fn stageGObj8_Callback3(_arg0: *mut Ground_GObj) {}

/// 1F8C64
pub unsafe extern "C" fn fn_801F8C64(gobj: *mut Item_GObj, _u1: *mut Ground, _u2: *mut Vec3, _u3: *mut HSD_GObj, _u4: f32) {
    let mut pos: Vec3 = core::mem::zeroed();
    let it = (*gobj).user_data as *mut Item;
    ext::grMaterial_801C8E28(gobj);

    ext::HSD_JObjSetFlagsAll((*it).xDD4_itemVar.mato.x4, JOBJ_HIDDEN as u32);
    ext::it_8026B294(gobj, &mut pos);
    ext::efSync_Spawn(0x445, gobj, &mut pos);
    ext::Ground_801C53EC(310);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut pos);
}

/// 1F8CDC: creates the material items and attaches them to the platform JObjs of the map
pub unsafe extern "C" fn grIceMt_801F8CDC(gobj: *mut Ground_GObj, joint_indices: *mut i16, block_num: i32,
                                          output_array: *mut *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let mut parent_jobjs: [*mut HSD_JObj; 20] = [null_mut(); 20];

    let archive = ext::grDatFiles_GetArchive();
    let jobj_desc = (*(*(*archive).unk4).unk8.add(7)).unk0;

    assert!(block_num <= BLOCK_COLL_JOBJ_MAX, "gricemt.c:2004: block_num<=BLOCK_COLL_JOBJ_MAX");

    let mut i = 0;
    while i < block_num {
        parent_jobjs[i as usize] = ext::Ground_801C3FA4(gobj, *joint_indices.add(i as usize) as i32);
        i += 1;
    }

    i = 0;
    while i < block_num {
        let coll_jobj = parent_jobjs[i as usize];
        assert!(!coll_jobj.is_null(), "gricemt.c:0x7E3: coll_jobj");

        let block_jobj = ext::HSD_JObjLoadJoint(jobj_desc);
        assert!(!block_jobj.is_null(), "gricemt.c:0x7E6: block_jobj");

        ext::HSD_JObjAddChild(coll_jobj, block_jobj);

        let item = ext::grMaterial_801C8CFC(8, 0, gp, coll_jobj, None, Some(fn_801F8C64), None);
        if !item.is_null() {
            ext::grMaterial_801C8DE0(item, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 6.0);
            ext::grMaterial_801C8E08(item);
            ext::grMaterial_801C8E68(item, 0);
        }
        *output_array.add(i as usize) = item;
        i += 1;
    }
}

/// 1F8E58
pub unsafe extern "C" fn fn_801F8E58(arg0: *mut Ground_GObj, out: *mut i32) -> i32 {
    let gp = (*arg0).user_data as *mut Ground;
    let mut list: [i32; 12] = [0; 12];
    let mut max: i32 = 0;

    for i in 0..12i32 {
        if (*gp).u.icemt9.x18[i as usize] == 0 && (ext::Stage_80225194() != 212 || i >= 4) {
            list[max as usize] = i;
            max += 1;
        }
    }

    assert!(max != 0, "gricemt.c:2077: max");
    let chosen = grIceMt_GetRandomIndex(max, list.as_ptr());

    for i in 0..12usize {
        if (*gp).u.icemt9.x18[i] > 0 {
            (*gp).u.icemt9.x18[i] -= 1;
        }
    }

    (*gp).u.icemt9.x18[chosen as usize] = (*yakumono_param).x2;
    grIceMt_GetRandomTimer(out);
    chosen
}

/// 1F9038
pub unsafe extern "C" fn fn_801F9038(gobj: *mut Ground_GObj) -> i32 {
    let gp = (*gobj).user_data as *mut Ground;
    let mut idx: usize;

    loop {
        idx = ext::HSD_Randi(6) as usize;
        if !((*gp).u.icemt9.x30[idx] != 0
            || (*gp).u.icemt9.x0.ids.under as i32 == icemt_field[idx].gobj_id
            || (*gp).u.icemt9.x0.ids.upper as i32 == icemt_field[idx].gobj_id)
        {
            break;
        }
    }

    for i in 0..6usize {
        if (*gp).u.icemt9.x30[i] > 0 {
            (*gp).u.icemt9.x30[i] -= 1;
        }
    }

    (*gp).u.icemt9.x30[idx] = (*yakumono_param).x0;

    icemt_field[idx].gobj_id
}

/// 1F9150
pub unsafe extern "C" fn fn_801F9150(arg0: *mut Ground_GObj, out: *mut i32) -> i32 {
    let gp = (*arg0).user_data as *mut Ground;
    let next: i16 = 0;

    loop {
        let a = *(*yakumono_param).xB0.offset((*gp).u.icemt.x1A as isize) as i32;
        *out = *(*yakumono_param).xB4.offset((*gp).u.icemt.x1A as isize) as i32;
        if a != -1 && *out != -1 {
            (*gp).u.icemt.x1A = (*gp).u.icemt.x1A.wrapping_add(1);
            return a;
        }
        (*gp).u.icemt.x1A = next;
    }
}

/// 1F91A8
pub unsafe extern "C" fn fn_801F91A8(gobj: *mut Ground_GObj) -> i32 {
    let gp = (*gobj).user_data as *mut Ground;
    let xac = (*yakumono_param).field_ixs;
    let index = *xac.offset((*gp).u.icemt.x1C as isize) as i32;
    if index == -1 {
        return index;
    }
    let result = icemt_field[index as usize].gobj_id;
    (*gp).u.icemt.x1C = (*gp).u.icemt.x1C.wrapping_add(1);
    result
}

/// 1F91EC
pub unsafe extern "C" fn grIceMt_801F91EC(param_1: *mut HSD_GObj, param_2: *mut i16, param_3: i32, param_4: i32,
                                          param_5: i32, param_6: i32, param_7: i32, param_8: mpLib_JointCollisionCallback) {
    let gp = GET_GROUND(param_1);
    *param_2 = 0;
    *param_2.add(1) = 0;
    *param_2.add(2) = param_3 as i16;
    *param_2.add(3) = param_4 as i16;
    *param_2.add(4) = param_5 as i16;
    *param_2.add(5) = param_6 as i16;
    *param_2.add(6) = param_7 as i16;
    ext::grAnime_801C7A04(param_1, param_3, 7, 0.0);
    if param_4 != -1 {
        ext::grAnime_801C7A04(param_1, param_4, 7, 0.0);
    }
    ext::mpJointSetCb1(param_5, gp as *mut c_void, param_8);
}

/// 1F929C
pub unsafe extern "C" fn grIceMt_801F929C(arg0: *mut HSD_GObj, arg1: *mut c_void) {
    let p = arg1 as *mut i16;

    if *p != 0 {
        *p.add(1) = (*p.add(1)).wrapping_add(1);
        if *p.add(1) == *p.add(5) {
            ext::mpLib_80057BC0(*p.add(4) as i32);
        } else if *p.add(1) == *p.add(6) {
            ext::mpJointListAdd(*p.add(4) as i32);
        }
        if ext::grAnime_801C83D0(arg0, *p.add(2) as i32, 7) != 0 {
            *p = 0;
        }
    }
}

/// the three break-animation starts: the record `s` of the map gobj `map` is armed on the first collision
#[inline(always)]
unsafe fn break_start(coll: *mut CollData, s: *mut i16, map: i32) {
    if (*coll).x34_flags.b1234() as i32 == 1 && *s == 0 {
        let gobj = ext::Ground_GetMapGObj(map);
        *s = 1;
        *s.add(1) = 0;
        ext::grAnime_801C7A04(gobj, *s.add(2) as i32, 7, 1.0);
        ext::grAnime_801C7B24(gobj, *s.add(2) as i32, 7, 0.0);
        ext::grAnime_801C78FC(gobj, *s.add(2) as i32, 7);
        if *s.add(3) != -1 {
            ext::grAnime_801C7A04(gobj, *s.add(3) as i32, 7, 1.0);
            ext::grAnime_801C7B24(gobj, *s.add(3) as i32, 7, 0.0);
            ext::grAnime_801C78FC(gobj, *s.add(3) as i32, 7);
        }
    }
}

/// 1F9338
pub unsafe extern "C" fn fn_801F9338(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32,
                                     ground_kind: u32, delta_y: f32) {
    let gp = user_data as *mut Ground;
    break_start(coll, x100_ptr(gp), 2);
    onJointCollision(user_data, joint_id, coll, coll_x50, ground_kind, delta_y);
}

/// 1F9448
pub unsafe extern "C" fn fn_801F9448(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32,
                                     ground_kind: u32, delta_y: f32) {
    let gp = user_data as *mut Ground;
    break_start(coll, x100_ptr(gp).add(4 + 3), 2);
    onJointCollision(user_data, joint_id, coll, coll_x50, ground_kind, delta_y);
}

/// 1F9558
pub unsafe extern "C" fn fn_801F9558(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32,
                                     ground_kind: u32, delta_y: f32) {
    let gp = user_data as *mut Ground;
    break_start(coll, x100_ptr(gp).add(4), 4);
    onJointCollision(user_data, joint_id, coll, coll_x50, ground_kind, delta_y);
}

/// 1F9668
pub unsafe extern "C" fn grIceMt_801F9668(arg0: f32) {
    let bg_gobj = ext::Ground_GetMapGObj(8);
    assert!(!bg_gobj.is_null(), "gricemt.c:0xA37: bg_gobj");
    let bg_gp = GET_GROUND(bg_gobj);
    assert!(!bg_gp.is_null(), "gricemt.c:0xA38: bg_gp");

    (*bg_gp).u.icemt_bg.x0 = arg0;
}

/// 1F96E0
pub unsafe extern "C" fn grIceMt_801F96E0(arg0: *mut grIceMt_GObj9_GObj10_UnderUpperIdPair, y: f32) -> f32 {
    if (*arg0).under != -1 {
        let mgobj = ext::Ground_GetMapGObj((*arg0).under as i32);
        assert!(!mgobj.is_null(), "gricemt.c:2629: mgobj");
        let jobj = GET_JOBJ(mgobj);
        assert!(!jobj.is_null(), "gricemt.c:2630: jobj");
        HSD_JObjAddTranslationY(&mut *jobj, y, &mut Dirty);
    }
    if (*arg0).upper != -1 {
        let mgobj = ext::Ground_GetMapGObj((*arg0).upper as i32);
        assert!(!mgobj.is_null(), "gricemt.c:2635: mgobj");
        let jobj = GET_JOBJ(mgobj);
        assert!(!jobj.is_null(), "gricemt.c:2636: jobj");
        HSD_JObjAddTranslationY(&mut *jobj, y, &mut Dirty);
    }
    0.0
}

/// `static inline grIceMt_FA0BC_jobjs`: the first of the map's four refreshed joints
#[inline(always)]
pub unsafe fn grIceMt_FA0BC_jobjs(g: *mut Ground) -> *mut *mut HSD_JObj {
    &raw mut (*g).u.icemt1.x4
}

/// the "block map changed" refresh: if the flag is set, clears it and refreshes the four joints
#[inline(always)]
unsafe fn refresh_joints(gp: *mut Ground) {
    let ptrs = grIceMt_FA0BC_jobjs(gp);
    if (*gp).u.icemt1.x0_b1() != 0 {
        (*gp).u.icemt1.set_x0_b1(0);
        for k in 0..4 {
            if !(*ptrs.add(k)).is_null() {
                ext::Ground_801C2D0C(k as i32, *ptrs.add(k));
            }
        }
    }
}

/// 1F98A8
pub unsafe extern "C" fn grIceMt_801F98A8(param1: *mut Ground_GObj) {
    let gp = (*param1).user_data as *mut Ground;
    refresh_joints(gp);
}

/// 1F993C
pub unsafe extern "C" fn grIceMt_801F993C(under_id: i32, upper_id: i32) -> f32 {
    let y = ext::Ground_801C0498();

    let mut upper_ix = 0usize;
    while upper_ix < ICEMT_FIELD_MAX {
        if under_id == icemt_field[upper_ix].gobj_id {
            break;
        }
        upper_ix += 1;
    }
    assert!(upper_ix < ICEMT_FIELD_MAX, "gricemt.c:2717: upper_ix<ICEMT_FIELD_MAX");

    let mut under_ix = 0usize;
    while under_ix < ICEMT_FIELD_MAX {
        if upper_id == icemt_field[under_ix].gobj_id {
            break;
        }
        under_ix += 1;
    }
    assert!(under_ix < ICEMT_FIELD_MAX, "gricemt.c:2722: under_ix<ICEMT_FIELD_MAX");

    let inner = fmadds(y, icemt_field[under_ix].x4, (*yakumono_param).x40);
    -fmadds(y, icemt_field[upper_ix].x8, -inner)
}

/// 1F9ACC
pub unsafe extern "C" fn grIceMt_801F9ACC(ids: *mut grIceMt_GObj9_GObj10_UnderUpperIdPair, _y: f32,
                                          ev: GrIceMtSegmentLookup, arg3: *mut Ground_GObj) -> bool {
    let mut result = false;

    if (*ids).under == -1 || (*ids).upper == -1 {
        return false;
    }

    let y0 = grIceMt_801F993C((*ids).under as i32, (*ids).upper as i32);
    let mut mgobj = ext::Ground_GetMapGObj((*ids).upper as i32);
    assert!(!mgobj.is_null(), "gricemt.c:2745: mgobj");
    let mut jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "gricemt.c:2746: jobj");
    let cur = HSD_JObjGetTranslationY(&*jobj);
    let abs = |x: f32| if x < 0.0 { -x } else { x };
    if abs(cur) < 10.0 {
        let gp = GET_GROUND(ext::Ground_GetMapGObj((*ids).upper as i32));
        (*gp).u.icemt1.set_x0_b1(1);
    } else if abs(cur + y0) < 10.0 {
        let gp = GET_GROUND(ext::Ground_GetMapGObj((*ids).under as i32));
        (*gp).u.icemt1.set_x0_b1(1);
    }
    if cur < 0.5f32 * -y0 {
        let id = (*ids).under as i32;
        if id != -1 {
            let gp = GET_GROUND(ext::Ground_GetMapGObj(id));
            if (*gp).u.icemt1.x0_b0() == 0 {
                (*gp).u.icemt1.set_x0_b0(1);
                (*gp).u.icemt1.set_x0_b1(1);
            }
        }
        let id = (*ids).upper as i32;
        if id != -1 {
            let gp = GET_GROUND(ext::Ground_GetMapGObj(id));
            (*gp).u.icemt1.set_x0_b0(0);
        }
    } else {
        let id = (*ids).upper as i32;
        if id != -1 {
            let gp = GET_GROUND(ext::Ground_GetMapGObj(id));
            if (*gp).u.icemt1.x0_b0() == 0 {
                (*gp).u.icemt1.set_x0_b0(1);
                (*gp).u.icemt1.set_x0_b1(1);
            }
        }
        let id = (*ids).under as i32;
        if id != -1 {
            let gp = GET_GROUND(ext::Ground_GetMapGObj(id));
            (*gp).u.icemt1.set_x0_b0(0);
        }
    }
    let ev = ev.unwrap();
    if cur > 0.0 {
        let id = (*ids).under as i32;
        if id != -1 {
            mgobj = ext::Ground_GetMapGObj(id);
            assert!(!mgobj.is_null(), "gricemt.c:2803: mgobj");
            ext::Ground_801C4A08(mgobj);
        }
        (*ids).under = (*ids).upper;
        (*ids).upper = ev(arg3) as i16;
        if (*ids).upper != -1 {
            let y1 = grIceMt_801F993C((*ids).under as i32, (*ids).upper as i32);
            mgobj = setupStageCallbacks((*ids).upper as i32);
            assert!(!mgobj.is_null(), "gricemt.c:2814: mgobj");
            jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
            assert!(!jobj.is_null(), "gricemt.c:2815: jobj");
            HSD_JObjSetTranslateY(&mut *jobj, cur - y1, &mut Dirty);
            ext::Ground_801C32AC((*ids).upper as i32);
            ext::Ground_UpdateMapColl(mgobj);
            ext::Ground_801C3214((*ids).upper as i32);
        }
        mgobj = ext::Ground_GetMapGObj((*ids).under as i32);
        if !mgobj.is_null() {
            let gp = GET_GROUND(mgobj);
            if !gp.is_null() {
                (*gp).u.icemt1.set_x0_b1(1);
                refresh_joints(gp);
            }
        }
        result = true;
    } else if cur < -y0 {
        let id = (*ids).upper as i32;
        if id != -1 {
            mgobj = ext::Ground_GetMapGObj(id);
            assert!(!mgobj.is_null(), "gricemt.c:2835: mgobj");
            ext::Ground_801C4A08(mgobj);
        }
        (*ids).upper = (*ids).under;
        (*ids).under = ev(arg3) as i16;
        if (*ids).under != -1 {
            let y1 = grIceMt_801F993C((*ids).under as i32, (*ids).upper as i32);
            mgobj = setupStageCallbacks((*ids).under as i32);
            assert!(!mgobj.is_null(), "gricemt.c:2846: mgobj");
            jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
            assert!(!jobj.is_null(), "gricemt.c:2847: jobj");
            HSD_JObjSetTranslateY(&mut *jobj, y1 + (cur + y0), &mut Dirty);
            ext::Ground_801C32AC((*ids).under as i32);
            ext::Ground_UpdateMapColl(mgobj);
            ext::Ground_801C3214((*ids).under as i32);
        }
        mgobj = ext::Ground_GetMapGObj((*ids).upper as i32);
        if !mgobj.is_null() {
            let gp = GET_GROUND(mgobj);
            if !gp.is_null() {
                (*gp).u.icemt1.set_x0_b1(1);
                refresh_joints(gp);
            }
        }
        result = true;
    }
    if result {
        grIceMt_801FA854();
    }
    result
}

/// 1FA0BC
pub unsafe extern "C" fn grIceMt_801FA0BC(ids: *mut grIceMt_GObj9_GObj10_UnderUpperIdPair) {
    let frame = grIceMt_801F993C((*ids).under as i32, (*ids).upper as i32);

    if (*ids).under != -1 {
        let mgobj = setupStageCallbacks((*ids).under as i32);
        assert!(!mgobj.is_null(), "gricemt.c:2884: mgobj");
        let jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
        assert!(!jobj.is_null(), "gricemt.c:2886: jobj");
        let g = ext::Ground_801C0498();
        HSD_JObjSetTranslateY(&mut *jobj, fmadds(-20.0, g, frame), &mut Dirty);
        ext::Ground_801C3214((*ids).under as i32);
        ext::Ground_UpdateMapColl(mgobj);
        ext::Ground_801C32AC((*ids).under as i32);
    }

    if (*ids).upper != -1 {
        let mgobj = setupStageCallbacks((*ids).upper as i32);
        assert!(!mgobj.is_null(), "gricemt.c:2896: mgobj");
        let jobj = (*mgobj).hsd_obj as *mut HSD_JObj;
        assert!(!jobj.is_null(), "gricemt.c:2898: jobj");
        let g = ext::Ground_801C0498();
        HSD_JObjSetTranslateY(&mut *jobj, -20.0f32 * g, &mut Dirty);
        ext::Ground_801C3214((*ids).upper as i32);
        ext::Ground_UpdateMapColl(mgobj);
        ext::Ground_801C32AC((*ids).upper as i32);

        let gp = (*mgobj).user_data as *mut Ground;
        (*gp).u.icemt1.set_x0_b0(1);
        (*gp).u.icemt1.set_x0_b1(1);
        refresh_joints(gp);
    }
    grIceMt_801FA854();
}

/// 1FA364
pub unsafe extern "C" fn grIceMt_801FA364(state: *mut grIceMt_FA364_State, out: *mut f32, cb: GrIceMtCb,
                                          gobj: *mut Ground_GObj) -> bool {
    let mut ret = true;
    let mut next_delay: i32 = 0;
    let table = (yakumono_param as *const u8).add(4) as *const f32;

    match (*state).phase {
        0 => {
            let tmp = (*state).delay;
            (*state).delay = tmp.wrapping_sub(1);
            if tmp < 0 {
                (*state).idx = cb(gobj, &mut next_delay) as i16;
                (*state).delay = next_delay as i16;
                (*state).phase = 1;
                (*state).lerp_count = (*yakumono_param).x34;
            }
        }
        1 => {
            (*state).lerp_count = (*state).lerp_count.wrapping_sub(1);
            let tmp = (*state).lerp_count;
            if tmp != 0 {
                (*state).cur += (*table.offset((*state).idx as isize) - (*state).cur) / (tmp as f32);
                ret = false;
            } else {
                (*state).phase = 0;
                (*state).cur = *table.offset((*state).idx as isize);
            }
        }
        _ => {}
    }

    let mut result = (*state).cur;
    if (*state).burst_count != 0 {
        result = (*yakumono_param).x3C * ext::Ground_801C0498();
        (*state).burst_count = (*state).burst_count.wrapping_sub(1);
    }
    *out = result;
    ret
}

/// 1FA4CC
pub unsafe extern "C" fn fn_801FA4CC(num: i32) -> i32 {
    if num == 1 {
        ext::Ground_801C5740(num);
        1
    } else {
        0
    }
}

/// 1FA500: the position of `arg1` in the depth-first walk of the map's joints, or -1
pub unsafe extern "C" fn grIceMt_801FA500(arg0: *mut HSD_GObj, arg1: *mut HSD_JObj) -> i32 {
    let mut count = 0;

    let mut jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "gricemt.c:0xBB1: jobj");
    jobj = (*jobj).child;
    assert!(!jobj.is_null(), "gricemt.c:0xBB2: jobj");

    while !jobj.is_null() {
        if jobj == arg1 {
            break;
        }
        if (*jobj).flags & 0x1000 == 0 && !(*jobj).child.is_null() {
            jobj = (*jobj).child;
        } else if !(*jobj).next.is_null() {
            jobj = (*jobj).next;
        } else {
            loop {
                if (*jobj).parent.is_null() {
                    jobj = null_mut();
                    break;
                }
                if !(*(*jobj).parent).next.is_null() {
                    jobj = (*(*jobj).parent).next;
                    break;
                }
                jobj = (*jobj).parent;
            }
        }
        count += 1;
    }
    if !jobj.is_null() {
        return count;
    }
    -1
}

/// 1FA6D8
pub unsafe extern "C" fn grIceMt_801FA6D8() {
    if ext::Stage_80225194() == 0x4C {
        let gobj = ext::Ground_GetMapGObj(0xA);
        if !gobj.is_null() {
            let gp = (*gobj).user_data as *mut Ground;
            if !gp.is_null() {
                (*gp).u.icemt.xA = (*yakumono_param).x3A as i16;
            }
        }
    }
}

/// 1FA728
pub unsafe extern "C" fn grIceMt_801FA728(arg0: *mut Vec3) {
    if ext::Stage_80225194() == 0x4C {
        let temp_r3 = ext::Ground_GetMapGObj(10);
        if !temp_r3.is_null() {
            let temp_r3_2 = (*temp_r3).user_data as *mut Ground;
            if !temp_r3_2.is_null() {
                (*arg0).x = 0.0;
                (*arg0).y = -(*temp_r3_2).u.icemt.x10;
                (*arg0).z = 0.0;
                return;
            }
        }
        (*arg0).z = 0.0;
        (*arg0).y = 0.0;
        (*arg0).x = 0.0;
        return;
    }
    let temp_r3_3 = ext::Ground_GetMapGObj(9);
    if !temp_r3_3.is_null() {
        let temp_r3_4 = (*temp_r3_3).user_data as *mut Ground;
        if !temp_r3_4.is_null() {
            (*arg0).x = 0.0;
            (*arg0).y = -(*temp_r3_4).u.icemt.x10;
            (*arg0).z = 0.0;
            return;
        }
    }
    (*arg0).z = 0.0;
    (*arg0).y = 0.0;
    (*arg0).x = 0.0;
}

/// 1FA7F0: the first fighter touching the field controller's ice flags it
pub unsafe extern "C" fn onJointCollision(_user_data: *mut c_void, _joint_id: i32, coll: *mut CollData, _coll_x50: i32,
                                          _ground_kind: u32, _delta_y: f32) {
    if ext::Ground_GetP1Fighter() == (*coll).x0_gobj {
        let gobj = ext::Ground_GetMapGObj(0xA);
        if !gobj.is_null() {
            let gp2 = (*gobj).user_data as *mut Ground;
            if !gp2.is_null() {
                (*gp2).u.icemt10.set_x14_b4(1);
            }
        }
    }
}

/// 1FA854
pub unsafe extern "C" fn grIceMt_801FA854() {
    let gobj = ext::Ground_GetMapGObj(0);

    if !gobj.is_null() {
        let mut jobj = ext::Ground_801C3FA4(gobj, 0x12);
        if !jobj.is_null() {
            ext::Ground_801C2D0C(4, jobj);
        }
        jobj = ext::Ground_801C3FA4(gobj, 0x13);
        if !jobj.is_null() {
            ext::Ground_801C2D0C(5, jobj);
        }
        jobj = ext::Ground_801C3FA4(gobj, 0x14);
        if !jobj.is_null() {
            ext::Ground_801C2D0C(6, jobj);
        }
        jobj = ext::Ground_801C3FA4(gobj, 0x15);
        if !jobj.is_null() {
            ext::Ground_801C2D0C(7, jobj);
        }
    }
}
