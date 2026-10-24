//! Port of src/melee/gr/grgreens.c (Green Greens: Whispy's wind and apples, the 5x6 grid of
//! breakable star blocks that fall and respawn). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_grgreens.rs.
//!
//! Fused sites (tools/rust-port/fma_sites.py), each the gwtool `fmadds` (f64 product + f64 sum, one
//! f32 rounding): `grGreens_80213C10` (`diff * rand + y0`, the apple height), `grGreens_8021483C`
//! (the four grid coordinates `scale * k + min`) and `grGreens_802159B8` (twice: nothing to fuse
//! beyond `vec.y += 5.0f * scale`, which is `fmadds(5, scale, y)` in both branches).
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
//! `GroundVars` is used through the same views as the C: `greens` (the map's blocks), `greens2`
//! (Whispy's state) and `corneria` (the two `HSD_Free` pointers).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::grgreens::{
    fn_802159B4, fn_80215D50, grGreens_8021351C, grGreens_802138AC, grGreens_80213900, grGreens_8021393C, grGreens_80213978,
    grGreens_802139B4, grGreens_80213AAC, grGreens_80214654, grGreens_80214794, grGreens_80216E64, grGreens_80216E6C,
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
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { let _ = crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C2CF4(a0: i32) -> *mut HSD_JObj = |a0: i32| unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ };
    fn Ground_801C32D4(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { crate::tu::ground::Ground_801C32D4(a0 as _, a1 as _) as _ };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_801C5414(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { let _ = crate::tu::ground::Ground_801C5414(a0 as _, a1 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjClearFlags(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetFlags(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut f32) -> *mut c_void;
    fn ftCo_800C06E8(a0: *mut HSD_GObj, a1: i32, a2: *mut c_void);
    fn ftLib_800864A8(a0: *mut Vec, a1: *mut HSD_GObj) -> f32;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grAnime_801C84A4(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C84A4(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C9A10() -> *mut Vec = || unsafe { crate::tu::grlib::grLib_801C9A10() as _ };
    fn grLib_801C9A70(a0: i32, a1: *mut Vec) = |a0: i32, a1: *mut Vec| { unsafe { let _ = crate::tu::grlib::grLib_801C9A70(a0 as _, a1 as _); } };
    fn grMaterial_801C8CDC(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8CDC(a0 as _); } };
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn grMaterial_801C8D98(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8D98(a0 as _, a1 as _); } };
    fn grMaterial_801C8DE0(a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32) = |a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8DE0(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _); } };
    fn grMaterial_801C8E08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8E08(a0 as _); } };
    fn grMaterial_801C8E28(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8E28(a0 as _); } };
    fn grMaterial_801C8E48(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::grmaterial::grMaterial_801C8E48(a0 as _) as _ };
    fn grMaterial_801C8E68(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8E68(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_80275414(a0: *mut HSD_GObj);
    fn it_802EE200(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32, a3: f32) -> *mut HSD_GObj;
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn lb_80011A50(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) -> *mut lb_80011A50_t = |a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32| unsafe { melee_runtime::tu::lb_00F9::lb_80011A50(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ };
    fn memzero(a0: *mut c_void, a1: i32) = |a0: *mut c_void, a1: i32| { unsafe { let _ = melee_runtime::tu::lb_00B0::memzero(a0 as _, a1 as _); } };
    fn mpJointSetCb2(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { let _ = crate::tu::mplib::mpJointSetCb2(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80055E9C(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::mplib::mpLib_80055E9C(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::mplib::mpLib_80057424(a0 as _); } };
    fn mpLib_800581DC(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { let _ = crate::tu::mplib::mpLib_800581DC(a0 as _, a1 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
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

/// `struct grGreens_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grGreens_YakumonoParam {
    pub x0_blockTimerMin: i32,
    pub x4_blockTimerMax: i32,
    pub x8_blockBombChance: i32,
    pub xC: i32,
    pub x10: i32,
    pub x14: i32,
    pub x18: i32,
    pub x1C: i32,
    pub x20: i32,
    pub x24: i32,
    pub x28: i32,
    pub x2C: f32,
    pub x30: f32,
    pub x34_windTimerMin: i32,
    pub x38_windTimerMax: i32,
    pub x3C_windSpeed: f32,
    pub x40_left: f32,
    pub x44_right: f32,
    pub x48_top: f32,
    pub x4C_bottom: f32,
    pub x50: f32,
    pub x54: f32,
    pub x58: f32,
    pub x5C: i32,
    pub x60: i32,
    pub x64: i32,
    pub x68: i32,
    pub x6C: f32,
    pub x70: f32,
    pub x74: f32,
    pub x78: f32,
}

pub const Gr_Greens_Block_Status_None: u32 = 0;
pub const Gr_Greens_Block_Colum: i32 = 3;
pub const Gr_Greens_Block_Max: usize = 30;

/// `static struct grGreens_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grGreens_YakumonoParam = null_mut();
/// `static u8 grGr_804D6AAC`
pub static mut grGr_804D6AAC: u8 = 0;
/// `static u8 grGr_804D6AAD`
pub static mut grGr_804D6AAD: u8 = 0;
/// `static u8 grGr_8049F9E0[0x20]`
pub static mut grGr_8049F9E0: [u8; 0x20] = [0; 0x20];

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

/// `grGr_callbacks`
pub static mut grGr_callbacks: [StageCallbacks; 7] = [
    cbs(Some(grGreens_8021360C), Some(grGreens_802138AC), Some(grGreens_802138B4), Some(grGreens_802138B8), 0),
    cbs(Some(stageGObj1_OnInit), Some(grGreens_8021393C), Some(grGreens_80213944), Some(grGreens_80213948), 0),
    cbs(Some(grGreens_802138BC), Some(grGreens_80213900), Some(grGreens_80213908), Some(grGreens_8021390C), 0),
    cbs(Some(stageGObj3_OnInit), Some(grGreens_80213978), Some(grGreens_80213980), Some(grGreens_80213984), 0),
    cbs(Some(grGreens_80213988), Some(grGreens_802139B4), Some(grGreens_802139BC), Some(grGreens_802139C0), 0),
    cbs(Some(grGreens_802139C4), Some(grGreens_80213AAC), Some(grGreens_80213C10), Some(grGreens_80214654), 0),
    cbs(Some(grGreens_80214674), Some(grGreens_80214794), Some(grGreens_8021479C), Some(grGreens_80214804), 0xC0000000),
];

/// `grGr_StageData`
pub static mut grGr_StageData: StageData = StageData {
    grkind: Gr_Kind_Greens,
    callbacks: unsafe { &raw mut grGr_callbacks as *mut StageCallbacks },
    data1: b"/GrGr.dat\0".as_ptr() as *mut i8,
    on_init: Some(grGreens_Init),
    on_demo_init: Some(grGreens_80213458),
    on_load: Some(grGreens_802134F4),
    on_start: Some(grGreens_802134F8),
    callback4: Some(grGreens_8021351C),
    on_touch_line: Some(grGreens_80216E64),
    on_check_shadow_render: Some(grGreens_80216E6C),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `grGr_803E7734`
pub static grGr_803E7734: [i32; 10] = [0, 1, 0, 1, 0, 2, 0, 1, 0, 2];
/// `grGr_803E775C`
pub static grGr_803E775C: [i32; 8] = [0, 0, 0, 14, 0, 0, 0, 14];
/// `grGr_803E777C`
pub static grGr_803E777C: [[i32; 2]; 7] = [[1, 8], [15, 16], [2, 9], [3, 10], [4, 11], [5, 12], [6, 13]];
/// `grGr_803E77B4`
pub static grGr_803E77B4: [i32; 8] = [0, 0, 0, 14, 0, 0, 0, 14];
/// `grGr_803E77D4`
pub static grGr_803E77D4: [[i32; 2]; 7] = [[1, 8], [15, 16], [2, 9], [3, 10], [4, 11], [5, 12], [6, 13]];
/// `grGr_803E780C`
pub static mut grGr_803E780C: [Vec3; 2] = [Vec3 { x: -1.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }];

/// `grGr_803E7840`: JObj indices of the 30 breakable blocks (passed to Ground_801C3FA4).
pub static grGr_803E7840: [i16; Gr_Greens_Block_Max] = [
    5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
    0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22,
];

/// `grGr_803E787C`: map joint IDs of the 30 breakable blocks (passed to mpJointSetCb2).
pub static grGr_803E787C: [i16; Gr_Greens_Block_Max] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A,
    0x1B, 0x1C, 0x1D,
];

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// `%` as the shipping build computes it (gwtool's makeArithmeticSafe: x % 0 == x, INT_MIN % -1 == 0)
#[inline(always)]
fn srem(a: i32, b: i32) -> i32 {
    if b == 0 { a } else if a == i32::MIN && b == -1 { 0 } else { a % b }
}

/// `static inline randrange`
pub unsafe fn randrange(min: i32, max: i32) -> i32 {
    let rng: i32;
    if min > max {
        let diff = min.wrapping_sub(max);
        rng = if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
        max.wrapping_add(rng)
    } else if min < max {
        let diff = max.wrapping_sub(min);
        rng = if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
        min.wrapping_add(rng)
    } else {
        min
    }
}

/// `static inline getBlock`
pub unsafe fn getBlock(gp: *mut Ground, i: i32, j: i32) -> *mut grGreens_BlockVars {
    (*(*gp).u.greens.x8_blocks.add(i as usize)).as_mut_ptr().add(j as usize)
}

/// `blocks[i][j]` of the map's block grid
#[inline(always)]
unsafe fn blk(gp: *mut Ground, i: i32, j: i32) -> *mut grGreens_BlockVars {
    getBlock(gp, i, j)
}

/// `(gp->u.greens.x4 + row * 6)[col]`: the grid's rest position of one block
#[inline(always)]
unsafe fn pos_at(gp: *mut Ground, row: i32, col: i32) -> *mut Vec3 {
    (*gp).u.greens.x4.offset((row * 6) as isize).add(col as usize)
}

/// `static inline Ground_StartMapAnim` (gr/inlines.h)
unsafe fn Ground_StartMapAnim(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 213458
pub unsafe extern "C" fn grGreens_80213458(_arg: i32) {
    grGr_804D6AAC = 1;
}

/// 213460
pub unsafe extern "C" fn grGreens_Init() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grGreens_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grGreens_80213524(0);
    grGreens_80213524(2);
    grGreens_80213524(6);
    grGreens_80213524(5);
    grGreens_80213524(4);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::grLib_801C9A10();
    grGr_804D6AAC = 0;
}

/// 2134F4
pub unsafe extern "C" fn grGreens_802134F4() {}

/// 2134F8
pub unsafe extern "C" fn grGreens_802134F8() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 213524
pub unsafe extern "C" fn grGreens_80213524(id: i32) -> *mut Ground_GObj {
    let cbs = &raw mut grGr_callbacks[id as usize];
    let gobj = ext::Ground_GetStageGObj(id);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
        if (*cbs).callback3.is_some() {
            (*gp).x1C_callback = (*cbs).callback3;
        }
        if let Some(f) = (*cbs).on_init {
            f(gobj);
        }
        if (*cbs).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*cbs).gobj_proc, 4);
        }
    } else {
        // OSReport((char*) grGr_callbacks + 0xCC, "grgreens.c", 281, id);
    }
    gobj
}

/// 21360C
pub unsafe extern "C" fn grGreens_8021360C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    if grGr_804D6AAC != 0 {
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::grLib_801C9A70(0, &mut v);
        let j = ext::Ground_801C2CF4(0);
        HSD_JObjSetTranslate(&mut *j, &v, &mut Dirty);
        ext::grLib_801C9A70(1, &mut v);
        let j = ext::Ground_801C2CF4(1);
        HSD_JObjSetTranslate(&mut *j, &v, &mut Dirty);
        ext::grLib_801C9A70(2, &mut v);
        let j = ext::Ground_801C2CF4(2);
        HSD_JObjSetTranslate(&mut *j, &v, &mut Dirty);
        ext::grLib_801C9A70(3, &mut v);
        let j = ext::Ground_801C2CF4(3);
        HSD_JObjSetTranslate(&mut *j, &v, &mut Dirty);
    }
}

/// 2138B4
pub unsafe extern "C" fn grGreens_802138B4(_arg: *mut Ground_GObj) {}

/// 2138B8
pub unsafe extern "C" fn grGreens_802138B8(_arg: *mut Ground_GObj) {}

/// 2138BC
pub unsafe extern "C" fn grGreens_802138BC(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
}

/// 213908
pub unsafe extern "C" fn grGreens_80213908(_arg: *mut Ground_GObj) {}

/// 21390C
pub unsafe extern "C" fn grGreens_8021390C(_arg: *mut Ground_GObj) {}

/// `static stageGObj1_OnInit`
pub unsafe extern "C" fn stageGObj1_OnInit(gobj: *mut Ground_GObj) {
    Ground_StartMapAnim(gobj);
}

/// 213944
pub unsafe extern "C" fn grGreens_80213944(_arg: *mut Ground_GObj) {}

/// 213948
pub unsafe extern "C" fn grGreens_80213948(_arg: *mut Ground_GObj) {}

/// `static stageGObj3_OnInit`
pub unsafe extern "C" fn stageGObj3_OnInit(gobj: *mut Ground_GObj) {
    Ground_StartMapAnim(gobj);
}

/// 213980
pub unsafe extern "C" fn grGreens_80213980(_arg: *mut Ground_GObj) {}

/// 213984
pub unsafe extern "C" fn grGreens_80213984(_arg: *mut Ground_GObj) {}

/// 213988
pub unsafe extern "C" fn grGreens_80213988(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 2139BC
pub unsafe extern "C" fn grGreens_802139BC(_arg: *mut Ground_GObj) {}

/// 2139C0
pub unsafe extern "C" fn grGreens_802139C0(_arg: *mut Ground_GObj) {}

/// 2139C4
pub unsafe extern "C" fn grGreens_802139C4(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::ftCo_800C06E8(gobj, 9, fn_80213B1C as *mut c_void);
    (*gp).u.greens.x0_flags.whole_thing = 0;
    (*gp).u.greens.x4 = null_mut();
    (*gp).u.greens.x8_blocks = null_mut();
    (*gp).u.greens.xC = randrange((*yakumono_param).x38_windTimerMax, (*yakumono_param).x34_windTimerMin);
    (*gp).u.greens.x10 = 1;
    (*gp).u.greens.x1C = 0;
    (*gp).u.greens.x14 = ext::HSD_Randi(2);
    (*gp).u.greens.x18 = 0;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 213AB4
pub unsafe extern "C" fn grGreens_80213AB4(vec: *mut Vec, max_x: f32, mut min_x: f32, max_y: f32, mut min_y: f32) -> i32 {
    let max_y_actual: f32;
    let max_x_actual: f32;
    if max_x > min_x {
        max_x_actual = max_x;
    } else {
        max_x_actual = min_x;
        min_x = max_x;
    }
    if max_y > min_y {
        max_y_actual = max_y;
    } else {
        max_y_actual = min_y;
        min_y = max_y;
    }
    if min_x < (*vec).x && (*vec).x < max_x_actual {
        if min_y < (*vec).y && (*vec).y < max_y_actual {
            return 1;
        }
    }
    0
}

/// 213B1C (`fn_80213B1C`)
pub unsafe extern "C" fn fn_80213B1C(ground_gobj: *mut Ground_GObj, fighter_gobj: *mut Fighter_GObj, vec: *mut Vec) -> i32 {
    let mut vec2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::ftLib_80086644(fighter_gobj, &mut vec2);
    (*vec).y = 0.0;
    (*vec).z = 0.0;
    match (*GET_GROUND(ground_gobj)).u.greens2.x18 {
        1 => {
            if grGreens_80213AB4(&mut vec2, -(*yakumono_param).x40_left, -(*yakumono_param).x44_right,
                                 (*yakumono_param).x48_top, (*yakumono_param).x4C_bottom) == 1
            {
                (*vec).x = -(*yakumono_param).x3C_windSpeed;
                return 1;
            }
        }
        2 => {
            if grGreens_80213AB4(&mut vec2, (*yakumono_param).x44_right, (*yakumono_param).x40_left,
                                 (*yakumono_param).x48_top, (*yakumono_param).x4C_bottom) == 1
            {
                (*vec).x = (*yakumono_param).x3C_windSpeed;
                return 1;
            }
        }
        _ => {}
    }
    (*vec).x = 0.0;
    0
}

/// `static inline get_whispy_dir`
pub unsafe fn get_whispy_dir(gobj: *mut Ground_GObj, pos: *mut Vec3) -> i32 {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    HSD_JObjGetTranslation(&*jobj, &mut *pos);
    if ext::ftLib_800864A8(pos, null_mut()) == 1.0 { 1 } else { 0 }
}

/// `lb_80011A50(&grGr_803E780C[wind_dir], ...)` of the wind particles (both call sites)
unsafe fn wind_particles(wind_dir: i32) {
    if wind_dir == 0 {
        ext::lb_80011A50(&raw mut grGr_803E780C[wind_dir as usize], 0xF, 0.5, 0.0, 0.0, -(*yakumono_param).x44_right,
                         (*yakumono_param).x48_top, -(*yakumono_param).x40_left, (*yakumono_param).x4C_bottom);
    } else {
        ext::lb_80011A50(&raw mut grGr_803E780C[wind_dir as usize], 0xF, 0.5, 0.0, 0.0, (*yakumono_param).x40_left,
                         (*yakumono_param).x48_top, (*yakumono_param).x44_right, (*yakumono_param).x4C_bottom);
    }
}

/// `grAnime_801C8138` of Whispy's pair of gobjs for animation `x8` of state 1
unsafe fn whispy_anim(gobj: *mut Ground_GObj, gp: *mut Ground, bg_gobj: *mut Ground_GObj, bg_gp: *mut Ground) {
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grGr_803E777C[(*gp).u.greens2.x8 as usize][(*gp).u.greens2.x14 as usize]);
    ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32, grGr_803E77D4[(*gp).u.greens2.x8 as usize][(*gp).u.greens2.x14 as usize]);
}

/// advance Whispy to the next entry of the pattern table
unsafe fn next_pattern(gobj: *mut Ground_GObj) {
    let cur_gp = GET_GROUND(gobj);
    (*cur_gp).u.greens2.x0 = ((*cur_gp).u.greens2.x0 + 1) % 10;
    (*cur_gp).u.greens2.x4 = grGr_803E7734[(*cur_gp).u.greens2.x0 as usize];
    (*cur_gp).u.greens2.x10 = 1;
}

/// 213C10
pub unsafe extern "C" fn grGreens_80213C10(gobj: *mut Ground_GObj) {
    let bg_gobj = ext::Ground_GetMapGObj(4);
    let gp = GET_GROUND(gobj);
    let bg_gp = GET_GROUND(bg_gobj);
    let mut apple_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    (*gp).u.greens2.x18 = 0;

    if grGr_804D6AAC != 0 {
        return;
    }
    if grGr_804D6AAD != 0 {
        return;
    }

    (*gp).u.greens2.x1C += 1;

    match (*gp).u.greens2.x4 {
        0 => {
            if (*gp).u.greens2.x10 != 0 {
                (*gp).u.greens2.x10 = 0;
                (*gp).u.greens2.x8 = ext::HSD_Randi(8);
                (*gp).u.greens2.xC = randrange((*yakumono_param).x38_windTimerMax, (*yakumono_param).x34_windTimerMin);
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grGr_803E775C[(*gp).u.greens2.x8 as usize]);
                ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32, grGr_803E77B4[(*gp).u.greens2.x8 as usize]);
                return;
            }

            (*gp).u.greens2.xC -= 1;
            if ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
                if (*gp).u.greens2.xC > 0 {
                    (*gp).u.greens2.x8 = ext::HSD_Randi(8);
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grGr_803E775C[(*gp).u.greens2.x8 as usize]);
                    ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32, grGr_803E77B4[(*gp).u.greens2.x8 as usize]);
                    return;
                }

                next_pattern(gobj);
            }
        }
        1 => {
            if (*gp).u.greens2.x10 != 0 {
                (*gp).u.greens2.x10 = 0;
                (*gp).u.greens2.x8 = 0;
                let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
                HSD_JObjGetTranslation(&*jobj, &mut pos);
                let dir = if ext::ftLib_800864A8(&mut pos, null_mut()) == 1.0 { 1 } else { 0 };
                (*gp).u.greens2.x14 = dir;
                (*gp).u.greens2.xC = 0;
                whispy_anim(gobj, gp, bg_gobj, bg_gp);
                return;
            }

            match (*gp).u.greens2.x8 {
                0 | 5 => {
                    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                        (*gp).u.greens2.x8 += 1;
                        whispy_anim(gobj, gp, bg_gobj, bg_gp);
                        return;
                    }
                }
                1 => {
                    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                        (*gp).u.greens2.x8 += 1;
                        whispy_anim(gobj, gp, bg_gobj, bg_gp);
                        ext::lbAudioAx_800237A8(0x68FB8, 0x7F, 0x40);
                        return;
                    }
                }
                2 => {
                    if ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
                        (*gp).u.greens2.xC += 1;
                        if (*gp).u.greens2.xC as f32 > (*yakumono_param).x50 {
                            (*gp).u.greens2.x8 += 1;
                            (*gp).u.greens2.xC = 0;
                            whispy_anim(gobj, gp, bg_gobj, bg_gp);
                            ext::lbAudioAx_800237A8(if (*gp).u.greens2.x14 == 0 { 0x68FB0 } else { 0x68FB1 }, 0x7F, 0x40);
                            return;
                        }
                    }
                }
                3 => {
                    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                        (*gp).u.greens2.x8 += 1;
                        ext::grAnime_801C8138(gobj, (*gp).map_id as i32,
                                              grGr_803E777C[(*gp).u.greens2.x8 as usize][(*gp).u.greens2.x14 as usize]);
                        let wind_dir = (*GET_GROUND(gobj)).u.greens2.x14;
                        wind_particles(wind_dir);
                        ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32,
                                              grGr_803E77D4[(*gp).u.greens2.x8 as usize][(*gp).u.greens2.x14 as usize]);
                        return;
                    }
                }
                4 => {
                    ext::Camera_RequestQuake(QuakeKind_Loop as u32, null_mut());
                    (*gp).u.greens2.x18 = (*gp).u.greens2.x14 + 1;
                    if ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
                        (*gp).u.greens2.xC += 1;
                        if (*gp).u.greens2.xC as f32 > (*yakumono_param).x54
                            || ((*gp).u.greens2.xC as f32 > (*yakumono_param).x58
                                && (*gp).u.greens2.x14 != get_whispy_dir(gobj, &mut pos2))
                        {
                            (*gp).u.greens2.x8 += 1;
                            (*gp).u.greens2.xC = 0;
                            whispy_anim(gobj, gp, bg_gobj, bg_gp);
                            return;
                        }

                        let wind_dir = (*GET_GROUND(gobj)).u.greens2.x14;
                        wind_particles(wind_dir);
                        return;
                    }
                }
                6 => {
                    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                        next_pattern(gobj);
                        return;
                    }
                }
                _ => {}
            }
        }
        2 => {
            if (*gp).u.greens2.x10 != 0 {
                (*gp).u.greens2.x10 = 0;
                (*gp).u.greens2.x8 = 0;
                (*gp).u.greens2.xC = 0;
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 7);
                ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32, 7);
                ext::lbAudioAx_800237A8(0x68FB2, 0x7F, 0x40);
                (*gp).u.greens2.x20 = randrange((*yakumono_param).x60, (*yakumono_param).x5C - 1);
                (*gp).u.greens2.x24 = ext::HSD_Randi(2);
            } else if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                if (*gp).u.greens2.x20 < 0 {
                    next_pattern(gobj);
                } else {
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 7);
                    ext::grAnime_801C8138(bg_gobj, (*bg_gp).map_id as i32, 7);
                }
            } else if (*gp).u.greens2.x20 >= 0 {
                if (*gp).u.greens2.xC >= (*yakumono_param).x64
                    && srem((*gp).u.greens2.xC.wrapping_sub((*yakumono_param).x64), (*yakumono_param).x68) == 0
                {
                    let mut rand = ext::HSD_Randf();
                    let sign = if (*gp).u.greens2.x24 == 0 { -1.0f32 } else { 1.0f32 };
                    apple_pos.x = sign * (40.0 * rand);
                    rand = ext::HSD_Randf();
                    let diff = (*yakumono_param).x70 - (*yakumono_param).x6C;
                    apple_pos.y = fmadds(diff, rand, (*yakumono_param).x6C);
                    apple_pos.z = -40.0;
                    ext::lbAudioAx_800237A8(0x68FB3, 0x7F, 0x40);
                    ext::it_802EE200(gobj, &mut apple_pos, (*yakumono_param).x74, (*yakumono_param).x78);
                    (*gp).u.greens2.x20 -= 1;
                    (*gp).u.greens2.x24 = ((*gp).u.greens2.x24 + 1) & 1;
                }
            }

            (*gp).u.greens2.xC += 1;
        }
        _ => {}
    }
}

/// 214658 (`fn_80214658`)
pub unsafe extern "C" fn fn_80214658(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    (*gp).u.greens.x0_flags.anon_0.set_b0(0);
    grGr_804D6AAD = 0;
}

/// 214674
pub unsafe extern "C" fn grGreens_80214674(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let new_var = gobj;
    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    (*gp).u.greens.x4 = ext::HSD_MemAlloc((5 * 6 * core::mem::size_of::<Vec3>()) as i32) as *mut Vec3;
    ext::memzero((*gp).u.greens.x4 as *mut c_void, (5 * 6 * core::mem::size_of::<Vec3>()) as i32);
    (*gp).u.greens.x8_blocks = ext::HSD_MemAlloc((5 * core::mem::size_of::<[grGreens_BlockVars; 6]>()) as i32) as *mut [grGreens_BlockVars; 6];
    ext::memzero((*gp).u.greens.x8_blocks as *mut c_void, (5 * core::mem::size_of::<[grGreens_BlockVars; 6]>()) as i32);
    grGreens_8021483C(new_var);
    if grGr_804D6AAC == 0 {
        grGreens_80214B58(new_var);
    }
    grGr_804D6AAC = 0;
    grGreens_80214FA8(new_var);
    ext::Ground_801C10B8(new_var, Some(fn_80214658));
    (*gp).u.greens.xC = randrange((*yakumono_param).x0_blockTimerMin, (*yakumono_param).x4_blockTimerMax);
    (*gp).u.greens.x0_flags.anon_0.set_b0(1);
    grGr_804D6AAD = 1;
}

/// 21479C
pub unsafe extern "C" fn grGreens_8021479C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.greens.x0_flags.anon_0.b0() == 0 {
        grGreens_802166C4(gobj);
    }
    ext::Ground_UpdateMapColl(gobj);
    if (*gp).u.greens.x0_flags.anon_0.b0() == 0 {
        grGreens_80216C20(gobj);
    }
    ext::lb_800115F4();
}

/// 214804
pub unsafe extern "C" fn grGreens_80214804(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    ext::HSD_Free((*gp).u.corneria.xC8 as *mut c_void);
    ext::HSD_Free((*gp).u.corneria.xCC as *mut c_void);
}

/// 21483C
pub unsafe extern "C" fn grGreens_8021483C(gobj: *mut Ground_GObj) {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut left_max: f32 = -3.4028235e38;
    let mut left_min: f32 = 3.4028235e38;
    let mut left_top: f32 = -3.4028235e38;
    let mut left_bottom: f32 = 3.4028235e38;
    let mut right_max: f32 = -3.4028235e38;
    let mut right_top: f32 = -3.4028235e38;
    let mut right_min: f32 = 3.4028235e38;
    let mut right_bottom: f32 = 3.4028235e38;
    let gp = GET_GROUND(gobj);

    let mut i = 0usize;
    while i < 30 {
        let jobj = ext::Ground_801C3FA4(gobj, grGr_803E7840[i] as i32);

        assert!(!jobj.is_null(), "jobj");
        ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
        if vec.x < 0.0 {
            if left_max < vec.x {
                left_max = vec.x;
            }
            if left_min > vec.x {
                left_min = vec.x;
            }
            if left_top < vec.y {
                left_top = vec.y;
            }
            if left_bottom > vec.y {
                left_bottom = vec.y;
            }
        } else {
            if right_max < vec.x {
                right_max = vec.x;
            }
            if right_min > vec.x {
                right_min = vec.x;
            }
            if right_top < vec.y {
                right_top = vec.y;
            }
            if right_bottom > vec.y {
                right_bottom = vec.y;
            }
        }
        i += 1;
    }
    let x4 = (*gp).u.greens.x4 as *mut [Vec3; 6];
    let mut x_scale = (left_max - left_min) / 2.0;
    let mut y_scale = (left_top - left_bottom) / 4.0;
    for i in 0..5usize {
        for j in 0..3usize {
            (*x4.add(i))[j].x = fmadds(x_scale, j as i32 as f32, left_min);
            (*x4.add(i))[j].y = fmadds(y_scale, i as i32 as f32, left_bottom);
            (*x4.add(i))[j].z = 0.0;
        }
    }
    x_scale = (right_max - right_min) / 2.0;
    y_scale = (right_top - right_bottom) / 4.0;
    for i in 0..5usize {
        for j in 3..6usize {
            (*x4.add(i))[j].x = fmadds(x_scale, (j as i32 - 3) as f32, right_min);
            (*x4.add(i))[j].y = fmadds(y_scale, i as i32 as f32, right_bottom);
            (*x4.add(i))[j].z = 0.0;
        }
    }
}

/// `static inline BOMB`
pub unsafe fn BOMB(idx: i32) {
    grGr_8049F9E0[idx as usize] =
        if (if (*yakumono_param).x20 != 0 { ext::HSD_Randi((*yakumono_param).x20) } else { 0 }) != 0 { 1 } else { 2 };
}

/// 214B58
pub unsafe extern "C" fn grGreens_80214B58(_gobj: *mut Ground_GObj) {
    let mut i = 0usize;
    while i < 24 {
        grGr_8049F9E0[i] = 0;
        i += 1;
    }

    grGr_8049F9E0[0x18] = 0;
    grGr_8049F9E0[0x19] = 0;
    grGr_8049F9E0[0x1A] = 0;
    grGr_8049F9E0[0x1B] = 0;
    grGr_8049F9E0[0x1C] = 0;
    grGr_8049F9E0[0x1D] = 0;

    BOMB(0x12);
    BOMB(0x0C);
    BOMB(0x0D);
    BOMB(0x06);
    BOMB(0x07);
    BOMB(0x08);
    BOMB(0x00);
    BOMB(0x01);
    BOMB(0x02);
    BOMB(0x17);
    BOMB(0x10);
    BOMB(0x11);
    BOMB(0x09);
    BOMB(0x0A);
    BOMB(0x0B);
    BOMB(0x03);
    BOMB(0x04);
    BOMB(0x05);
}

/// 214FA8
pub unsafe extern "C" fn grGreens_80214FA8(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    let mut i: u32 = 0;
    while i < 30 {
        ext::mpJointSetCb2(grGr_803E787C[i as usize] as i32, gp as *mut c_void, Some(fn_80216DE4));
        i += 1;
    }
    let mut j: i32 = 0;
    while j < 30 {
        let jobj = ext::Ground_801C3FA4(gobj, grGr_803E7840[j as usize] as i32);
        if !jobj.is_null() {
            ext::HSD_JObjSetFlags(jobj, JOBJ_HIDDEN as u32);
        } else {
            panic!("grgreens.c:1203: assertion failed (0)");
        }
        j += 1;
    }
    j = 0;
    while j < 5 {
        let mut col: i32 = 0;
        while col < 6 {
            match grGr_8049F9E0[(j * 6 + col) as usize] {
                1 | 2 => {
                    grGreens_80215358(gobj, col, j, grGr_8049F9E0[(j * 6 + col) as usize] as i32, 3);
                }
                _ => {}
            }
            col += 1;
        }
        j += 1;
    }
}

/// 2150C4
pub unsafe extern "C" fn grGreens_802150C4(gobj: *mut Ground_GObj, arg1: i32, arg2: i32) {
    let gp = GET_GROUND(gobj);

    if arg2 > 0 && arg1 > 0 && (*blk(gp, arg2 - 1, arg1 - 1)).status() == 3 {
        if (*blk(gp, arg2 - 1, arg1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 - 1, arg1 - 1)).x18, (*blk(gp, arg2 - 1, arg1)).x18);
        } else if (*blk(gp, arg2, arg1 - 1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 - 1, arg1 - 1)).x18, (*blk(gp, arg2, arg1 - 1)).x18);
        }
    }
    if arg2 < 4 && arg1 > 0 && (*blk(gp, arg2 + 1, arg1 - 1)).status() == 3 {
        if (*blk(gp, arg2 + 1, arg1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 + 1, arg1 - 1)).x18, (*blk(gp, arg2 + 1, arg1)).x18);
        } else if (*blk(gp, arg2, arg1 - 1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 + 1, arg1 - 1)).x18, (*blk(gp, arg2, arg1 - 1)).x18);
        }
    }
    if arg2 > 0 && arg1 < 5 && (*blk(gp, arg2 - 1, arg1 + 1)).status() == 3 {
        if (*blk(gp, arg2 - 1, arg1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 - 1, arg1 + 1)).x18, (*blk(gp, arg2 - 1, arg1)).x18);
        } else if (*blk(gp, arg2, arg1 + 1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 - 1, arg1 + 1)).x18, (*blk(gp, arg2, arg1 + 1)).x18);
        }
    }
    if arg2 < 4 && arg1 < 5 && (*blk(gp, arg2 + 1, arg1 + 1)).status() == 3 {
        if (*blk(gp, arg2 + 1, arg1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 + 1, arg1 + 1)).x18, (*blk(gp, arg2 + 1, arg1)).x18);
            return;
        }
        if (*blk(gp, arg2, arg1 - 1)).status() == 3 {
            ext::mpLib_800581DC((*blk(gp, arg2 + 1, arg1 + 1)).x18, (*blk(gp, arg2, arg1 + 1)).x18);
        }
    }
}

/// `static inline get_block_material_params`
pub unsafe fn get_block_material_params(block_type: i32, duration: *mut f32, scale: *mut f32) {
    let value: i32 = if block_type == 2 { (*yakumono_param).x28 } else { (*yakumono_param).x24 };
    *duration = value as f32 * ext::Ground_801C0498();
    *scale = 5.0 * ext::Ground_801C0498();
}

/// 215358
pub unsafe extern "C" fn grGreens_80215358(gobj: *mut Ground_GObj, col: i32, row: i32, arg3: i32, arg4: i32) {
    let gp = GET_GROUND(gobj);
    let block = blk(gp, row, col);
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut arr = [0u8; 30];
    let type_: i32;
    let mut mat_param_a: f32 = 0.0;
    let mut mat_param_b: f32 = 0.0;
    assert!((*block).status() == Gr_Greens_Block_Status_None, "block->status==Gr_Greens_Block_Status_None");
    let mut n = 0;
    while n < 5 {
        let mut m = 0;
        while m < 6 {
            if (*blk(gp, n, m)).status() != Gr_Greens_Block_Status_None {
                arr[(*blk(gp, n, m)).index() as usize] = 1;
            }
            m += 1;
        }
        n += 1;
    }
    let mut num: i32 = 0;
    while num < 30 {
        if arr[num as usize] == 0 {
            break;
        }
        num += 1;
    }
    assert!(num < Gr_Greens_Block_Max as i32, "num<Gr_Greens_Block_Max");
    if arg3 == 1 {
        type_ = 1;
    } else if arg3 == 2 {
        type_ = 3;
    } else {
        panic!("grgreens.c:1324: assertion failed (0)");
    }
    let block_gobj = grGreens_80213524(type_);
    assert!(!block_gobj.is_null(), "block_gobj");
    let jobj = ext::Ground_801C3FA4(gobj, grGr_803E7840[num as usize] as i32);
    ext::HSD_JObjClearFlags(jobj, JOBJ_HIDDEN as u32);
    let item_gobj = ext::grMaterial_801C8CFC(8, 0, gp, jobj, Some(fn_802159B4), Some(fn_80215B84), Some(fn_80215D50));
    get_block_material_params(arg3, &mut mat_param_b, &mut mat_param_a);
    ext::grMaterial_801C8DE0(item_gobj, 0.0, mat_param_a, 0.0, 0.0, mat_param_a, 0.0, mat_param_b);
    ext::grMaterial_801C8E08(item_gobj);
    ext::grMaterial_801C8E68(item_gobj, GA_Ground as u32);
    (*block).set_status(arg4 as u32);
    (*block).set_index(num as u32);
    (*block).set_x1_1(if arg3 == 2 { 1 } else { 0 });
    (*block).set_x1_2(0);
    (*block).set_x1_3(0);
    (*block).x8 = if arg4 == 1 { ext::Stage_GetBlastZoneTopOffset() } else { (*(*gp).u.greens.x4.offset((row * 6 + col) as isize)).y };
    (*block).x4 = 0.0;
    (*block).xC = block_gobj;
    (*block).x10 = item_gobj;
    (*block).x14 = jobj;
    (*block).x18 = ext::Ground_801C32D4(6, grGr_803E7840[num as usize] as i32);
    (*block).x1C = null_mut();
    (*block).set_x1_4(0);
    (*block).set_x1_7(0);
    vec.x = (*pos_at(gp, row, col)).x;
    vec.y = (*block).x8;
    vec.z = 0.0;
    if arg4 == 3 {
        (*block).set_x1_5(1);
    } else {
        (*block).set_x1_5(0);
    }
    HSD_JObjSetTranslate(&mut *((*block_gobj).hsd_obj as *mut HSD_JObj), &vec, &mut Dirty);
    let f = 1.0 / ext::Ground_801C0498();
    vec.x *= f;
    vec.y *= f;
    vec.z *= f;
    HSD_JObjSetTranslate(&mut *(*block).x14, &vec, &mut Dirty);
    ext::mpLib_80055E9C((*block).x18);
    ext::mpLib_80057424((*block).x18);
}

/// 2159B8
pub unsafe extern "C" fn grGreens_802159B8(gp: *mut Ground, i: i32, j: i32, gobj: *mut HSD_GObj) {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let item_gobj = (*blk(gp, j, i)).x10;
    if !item_gobj.is_null() && (*blk(gp, j, i)).x1_7() == 0 {
        (*blk(gp, j, i)).set_x1_7(1);
        ext::grMaterial_801C8E28(item_gobj);
        (*blk(gp, j, i)).x1C = gobj;

        if (*blk(gp, j, i)).x1_1() != 0 {
            let jobj = (*(*blk(gp, j, i)).xC).hsd_obj as *mut HSD_JObj;

            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            ext::grMaterial_801C8D98(item_gobj, 1);
            ext::it_80275414(item_gobj);
            (*blk(gp, j, i)).set_x1_2(1);
            HSD_JObjGetTranslation(&*((*item_gobj).hsd_obj as *mut HSD_JObj), &mut vec);
            vec.y = fmadds(5.0, ext::Ground_801C0498(), vec.y);
            ext::efSync_Spawn(1039, item_gobj, &mut vec, null_mut());
        } else {
            let mut f: f32 = 0.0;
            (*blk(gp, j, i)).set_x1_3(1);
            ext::Camera_RequestQuake(QuakeKind_Small as u32, null_mut());
            HSD_JObjGetTranslation(&*((*item_gobj).hsd_obj as *mut HSD_JObj), &mut vec);
            vec.y = fmadds(5.0, ext::Ground_801C0498(), vec.y);
            ext::efSync_Spawn(1032, item_gobj, &mut vec, &mut f);
            ext::Ground_801C5414(430007, 186);
        }
    }
}

/// `static inline find_block`
pub unsafe fn find_block(gp: *mut Ground, item_gobj: *mut Item_GObj, row: *mut i32, col: *mut i32) -> bool {
    let mut i = 0;
    while i < 5 {
        let mut j = 0;
        while j < 6 {
            let block = blk(gp, i, j);

            if (*block).status() != Gr_Greens_Block_Status_None && (*block).x10 == item_gobj {
                *row = i;
                *col = j;
                return true;
            }
            j += 1;
        }
        i += 1;
    }
    false
}

/// 215B84 (`fn_80215B84`)
pub unsafe extern "C" fn fn_80215B84(item_gobj: *mut Item_GObj, gp: *mut Ground, _arg2: *mut Vec, gobj: *mut HSD_GObj, _arg4: f32) {
    let ground = gp;
    let mut col: i32 = -1;
    let mut row: i32 = 0;
    let hit = gobj;

    if !find_block(ground, item_gobj, &mut row, &mut col) {
        panic!("grgreens.c:1465: assertion failed (0)");
    }
    grGreens_802159B8(ground, col, row, hit);
}

/// 215D54
pub unsafe extern "C" fn grGreens_80215D54(gobj: *mut Ground_GObj, arg1: i32, _arg2: i32) -> i32 {
    let gp = GET_GROUND(gobj);

    let mut row: i32 = 1;
    while row < 5 {
        if ((*blk(gp, row, arg1)).status() == 1 || (*blk(gp, row, arg1)).status() == 2) && (*blk(gp, row - 1, arg1)).status() == 0 {
            let mut r = row;
            while r < 5 {
                let temp = *blk(gp, r, arg1);
                *blk(gp, r, arg1) = *blk(gp, r - 1, arg1);
                *blk(gp, r - 1, arg1) = temp;
                r += 1;
            }
            if (*blk(gp, row - 1, arg1)).status() == 2 {
                (*blk(gp, row - 1, arg1)).set_status(1);
            }
            row = 0;
        }
        row += 1;
    }
    gobj as isize as i32
}

/// 215ED8
pub unsafe extern "C" fn grGreens_80215ED8(gobj: *mut Ground_GObj, col: i32, row: i32) {
    let gp = GET_GROUND(gobj);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut scale: f32;

    if (*blk(gp, row, col)).status() == Gr_Greens_Block_Status_None {
        return;
    }
    if (*blk(gp, row, col)).x1_3() != 0 {
        return;
    }

    if (*blk(gp, row, col)).x1_2() != 0 {
        if ext::grMaterial_801C8E48((*blk(gp, row, col)).x10) != 0 {
            (*blk(gp, row, col)).set_x1_3(1);
        }
        return;
    }

    match (*blk(gp, row, col)).status() {
        1 => {
            if (*blk(gp, row, col)).x1_4() != 0 {
                (*blk(gp, row, col)).x4 = 0.0;
                (*blk(gp, row, col)).set_x1_4(0);
                grGreens_802159B8(gp, col, row, null_mut());
            } else {
                (*blk(gp, row, col)).x4 += (*yakumono_param).x30;
                if (*blk(gp, row, col)).x4 > (*yakumono_param).x2C {
                    (*blk(gp, row, col)).x4 = (*yakumono_param).x2C;
                }
                (*blk(gp, row, col)).x8 -= (*blk(gp, row, col)).x4;
                if row > 0 && ((*blk(gp, row - 1, col)).status() == 1 || (*blk(gp, row - 1, col)).status() == 2) {
                    let spacing: f32 = (*(*gp).u.greens.x4.offset((row * 6) as isize).add(col as usize)).y
                        - (*(*gp).u.greens.x4.offset((row * 6 - 6) as isize).add(col as usize)).y;
                    if (*blk(gp, row, col)).x8 - (*blk(gp, row - 1, col)).x8 < spacing {
                        (*blk(gp, row, col)).set_status(2);
                        (*blk(gp, row, col)).x8 = spacing + (*blk(gp, row - 1, col)).x8;
                    }
                }
            }

            if (*blk(gp, row, col)).x8 < (*pos_at(gp, row, col)).y {
                (*blk(gp, row, col)).x8 = (*pos_at(gp, row, col)).y;
                (*blk(gp, row, col)).set_status(3);
                (*blk(gp, row, col)).set_x1_5(1);
                let mut next_row = row + 1;
                while next_row < 5 {
                    if (*blk(gp, next_row, col)).status() != 2 {
                        break;
                    }
                    (*blk(gp, next_row, col)).x8 = (*pos_at(gp, next_row, col)).y;
                    (*blk(gp, next_row, col)).set_status(3);
                    (*blk(gp, next_row, col)).set_x1_5(1);
                    pos.x = (*pos_at(gp, next_row, col)).x;
                    pos.y = (*blk(gp, next_row, col)).x8;
                    pos.z = 0.0;
                    HSD_JObjSetTranslate(&mut *((*(*blk(gp, next_row, col)).xC).hsd_obj as *mut HSD_JObj), &pos, &mut Dirty);
                    scale = 1.0 / ext::Ground_801C0498();
                    pos.x *= scale;
                    pos.y *= scale;
                    pos.z *= scale;
                    HSD_JObjSetTranslate(&mut *(*blk(gp, next_row, col)).x14, &pos, &mut Dirty);
                    next_row += 1;
                }
            } else {
                let mut next_row = row + 1;
                while next_row < 5 {
                    if (*blk(gp, next_row, col)).status() != 2 {
                        break;
                    }
                    pos.x = (*pos_at(gp, next_row, col)).x;
                    scale = (*pos_at(gp, next_row, col)).y - (*(*gp).u.greens.x4.offset((next_row * 6 - 6) as isize).add(col as usize)).y;
                    pos.y = scale + (*blk(gp, next_row - 1, col)).x8;
                    pos.z = 0.0;
                    (*blk(gp, next_row, col)).x8 = pos.y;
                    HSD_JObjSetTranslate(&mut *((*(*blk(gp, next_row, col)).xC).hsd_obj as *mut HSD_JObj), &pos, &mut Dirty);
                    scale = 1.0 / ext::Ground_801C0498();
                    pos.x *= scale;
                    pos.y *= scale;
                    pos.z *= scale;
                    HSD_JObjSetTranslate(&mut *(*blk(gp, next_row, col)).x14, &pos, &mut Dirty);
                    next_row += 1;
                }
            }
        }
        2 => return,
        3 => return,
        _ => {}
    }

    pos.x = (*pos_at(gp, row, col)).x;
    pos.y = (*blk(gp, row, col)).x8;
    pos.z = 0.0;
    HSD_JObjSetTranslate(&mut *((*(*blk(gp, row, col)).xC).hsd_obj as *mut HSD_JObj), &pos, &mut Dirty);
    scale = 1.0 / ext::Ground_801C0498();
    pos.x *= scale;
    pos.y *= scale;
    pos.z *= scale;
    HSD_JObjSetTranslate(&mut *(*blk(gp, row, col)).x14, &pos, &mut Dirty);
}

/// 2166C4
pub unsafe extern "C" fn grGreens_802166C4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let mut weights = [0u8; 6];

    let old = (*gp).u.greens.xC;
    (*gp).u.greens.xC = old.wrapping_sub(1);
    if old < 0 {
        let mut ix: i32 = -1;
        let mut left_has_zero = 0;
        let mut right_has_zero = 0;

        let mut col: i32 = 0;
        while col < 6 {
            weights[col as usize] = (*yakumono_param).x1C as u8;
            let mut row: i32 = 4;
            while row >= 0 {
                if (*getBlock(gp, row, col)).status() != Gr_Greens_Block_Status_None {
                    match row {
                        0 => weights[col as usize] = (*yakumono_param).x18 as u8,
                        1 => weights[col as usize] = (*yakumono_param).x14 as u8,
                        2 => weights[col as usize] = (*yakumono_param).x10 as u8,
                        3 => weights[col as usize] = (*yakumono_param).xC as u8,
                        4 => weights[col as usize] = (*yakumono_param).x8_blockBombChance as u8,
                        _ => {}
                    }
                    break;
                }
                row -= 1;
            }
            col += 1;
        }

        let mut k = 0usize;
        while k < 3 {
            if weights[k] == 0 {
                left_has_zero = 1;
            }
            if weights[k + 3] == 0 {
                right_has_zero = 1;
            }
            k += 1;
        }

        let mut w = 0usize; // wp - weights_base
        if left_has_zero == 0 {
            if right_has_zero == 0 {
                let mut total = weights[0] as i32 + weights[1] as i32;
                total += weights[2] as i32;
                total += weights[3] as i32;
                total += weights[4] as i32;
                total += weights[5] as i32;
                let mut pick = if total != 0 { ext::HSD_Randi(total) } else { 0 };
                ix = 0;
                pick -= weights[w] as i32;
                if pick >= 0 {
                    ix = 1;
                    w += 1;
                    pick -= weights[w] as i32;
                    if pick >= 0 {
                        ix = 2;
                        w += 1;
                        pick -= weights[w] as i32;
                        if pick >= 0 {
                            ix = 3;
                            w += 1;
                            pick -= weights[w] as i32;
                            if pick >= 0 {
                                ix = 4;
                                w += 1;
                                pick -= weights[w] as i32;
                                if pick >= 0 {
                                    ix = 5;
                                    w += 1;
                                    pick -= weights[w] as i32;
                                    if pick >= 0 {
                                        ix = 6;
                                    }
                                }
                            }
                        }
                    }
                }
                assert!(ix < Gr_Greens_Block_Colum * 2, "ix<Gr_Greens_Block_Colum*2");
            } else {
                let mut total = weights[0] as i32 + weights[1] as i32;
                total += weights[2] as i32;
                let mut pick = if total != 0 { ext::HSD_Randi(total) } else { 0 };
                ix = 0;
                pick -= weights[w] as i32;
                if pick >= 0 {
                    ix = 1;
                    w += 1;
                    pick -= weights[w] as i32;
                    if pick >= 0 {
                        ix = 2;
                        w += 1;
                        pick -= weights[w] as i32;
                        if pick >= 0 {
                            ix = 3;
                        }
                    }
                }
                assert!(ix < Gr_Greens_Block_Colum, "ix<Gr_Greens_Block_Colum");
            }
        } else if right_has_zero == 0 {
            let mut total = weights[3] as i32 + weights[4] as i32;
            total += weights[5] as i32;
            let mut pick = if total != 0 { ext::HSD_Randi(total) } else { 0 };
            ix = 3;
            pick -= weights[3] as i32;
            if pick >= 0 {
                let mut wb = 3usize; // wp = weights_base (= &weights[3])
                ix = 4;
                wb += 1;
                pick -= weights[wb] as i32;
                if pick >= 0 {
                    ix = 5;
                    pick -= weights[wb + 1] as i32;
                    if pick >= 0 {
                        ix = 6;
                    }
                }
            }
            assert!(ix < Gr_Greens_Block_Colum * 2, "ix<Gr_Greens_Block_Colum*2");
        }

        if ix != -1 {
            let mut spawn_row: i32 = 4;

            while spawn_row > 0 {
                if (*getBlock(gp, spawn_row - 1, ix)).status() != Gr_Greens_Block_Status_None {
                    break;
                }
                spawn_row -= 1;
            }
            let type_roll = if (*yakumono_param).x20 != 0 { ext::HSD_Randi((*yakumono_param).x20) } else { 0 };
            let type_ = if type_roll != 0 { 1 } else { 2 };
            grGreens_80215358(gobj, ix, spawn_row, type_, 1);
        }

        (*gp).u.greens.xC = randrange((*yakumono_param).x0_blockTimerMin, (*yakumono_param).x4_blockTimerMax);
    }

    let mut col: i32 = 0;
    while col < 5 {
        let mut row: i32 = 0;
        while row < 6 {
            grGreens_80215ED8(gobj, row, col);
            row += 1;
        }
        col += 1;
    }

    let mut cleanup_row: i32 = 0;
    while cleanup_row < 5 {
        let mut cleanup_col: i32 = 0;
        while cleanup_col < 6 {
            let b = getBlock(gp, cleanup_row, cleanup_col);
            if (*b).x1_3() != 0 {
                (*b).set_x1_3(0);
                (*b).set_x1_6(1);
                (*b).set_status(Gr_Greens_Block_Status_None);
                ext::Ground_801C4A08((*b).xC);
                ext::grMaterial_801C8CDC((*b).x10);
                ext::HSD_JObjSetFlags((*b).x14, JOBJ_HIDDEN as u32);
                grGreens_80215D54(gobj, cleanup_col, cleanup_row);
                cleanup_row = -1;
                cleanup_col = 6;
            }
            cleanup_col += 1;
        }
        cleanup_row += 1;
    }
}

/// 216C20
pub unsafe extern "C" fn grGreens_80216C20(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);

    let mut i: i32 = 0;
    while i < 5 {
        let mut j: i32 = 0;
        while j < 6 {
            if (*blk(gp, i, j)).x1_5() != 0 {
                let gp2 = GET_GROUND(gobj);
                let x18 = (*blk(gp2, i, j)).x18;

                if i > 0 && (*blk(gp2, i - 1, j)).status() == 3 {
                    ext::mpLib_800581DC(x18, (*blk(gp2, i - 1, j)).x18);
                }
                if j > 0 && (*blk(gp2, i, j - 1)).status() == 3 {
                    ext::mpLib_800581DC(x18, (*blk(gp2, i, j - 1)).x18);
                }
                if i < 4 && (*blk(gp2, i + 1, j)).status() == 3 {
                    ext::mpLib_800581DC(x18, (*blk(gp2, i + 1, j)).x18);
                }
                if j < 5 && (*blk(gp2, i, j + 1)).status() == 3 {
                    ext::mpLib_800581DC(x18, (*blk(gp2, i, j + 1)).x18);
                }
                (*blk(gp, i, j)).set_x1_5(0);
            } else if (*blk(gp, i, j)).x1_6() != 0 {
                grGreens_802150C4(gobj, j, i);
                (*blk(gp, i, j)).set_x1_6(0);
            }
            j += 1;
        }
        i += 1;
    }
}

/// 216DE4 (`fn_80216DE4`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn fn_80216DE4(user_data: *mut c_void, joint_id: i32, _coll: *mut CollData, _coll_x50: i32,
                                     _ground_kind: u32, _delta_y: f32) {
    let gp = user_data as *mut Ground;
    let mut i: i32 = 0;
    while i < 5 {
        let mut j: i32 = 0;
        while j < 6 {
            if (*getBlock(gp, i, j)).status() != Gr_Greens_Block_Status_None && joint_id == (*getBlock(gp, i, j)).x18 {
                (*getBlock(gp, i, j)).set_x1_4(1);
                i = 5;
                break;
            }
            j += 1;
        }
        i += 1;
    }
}
