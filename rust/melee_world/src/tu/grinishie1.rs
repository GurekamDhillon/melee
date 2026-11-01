//! Port of src/melee/gr/grinishie1.c (Mushroom Kingdom: the 19 hatena/brick blocks, the two
//! balance platforms and the 20-second warning). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_grinishie1.rs.
//!
//! Fused sites (tools/rust-port/fma_sites.py): `fn_801FC9AC` (two: `xE0 += (f32) coll_x50 * unk2C[1].z`
//! and the `xE4` twin) and `grInishie1_801FC110` (`accel * unk24 + xF4`); each is the gwtool
//! `fmadds` (f64 product + f64 sum, one f32 rounding), see [`fmadds`].
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
//! The `while (true) {}` after an "oioi.." OSReport (corrupted block table) is a panic here.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn Camera_800311CC(a0: f32) = |a0: f32| crate::cm::camera::world().camera_800311cc(a0);
    fn Camera_800311DC(a0: f32) = |a0: f32| crate::cm::camera::world().camera_800311dc(a0);
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec) = |a0: u32, a1: *mut Vec| unsafe { crate::cm::camera::Camera_RequestQuake(a0 as _, a1 as _) };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GetMatchTimer(a0: *mut i32) -> i32 = |a0: *mut i32| unsafe { crate::gm::GetMatchTimer(a0 as _) as _ };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_801C5414(a0: i32, a1: i32) = |a0: i32, a1: i32| { unsafe { crate::tu::ground::Ground_801C5414(a0 as _, a1 as _); } };
    fn Ground_801C5A94() -> i32 = || unsafe { crate::tu::ground::Ground_801C5A94() as _ };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_DObjClearFlags(a0: *mut HSD_DObj, a1: u32);
    fn HSD_DObjSetFlags(a0: *mut HSD_DObj, a1: u32);
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetDObj(a0 as _) as _ };
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(a0 as _) as _ };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Stage_GetBlastZoneBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneBottomOffset() as _ };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec) -> *mut c_void;
    fn gm_8016AE80() -> i32 = || unsafe { crate::gm::gm_8016AE80() as _ };
    fn gm_8016B238() -> i32 = || unsafe { crate::gm::gm_8016B238() as _ };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C9EE8(a0: *mut Vec, a1: f32) -> i32 = |a0: *mut Vec, a1: f32| unsafe { crate::tu::grlib::grLib_801C9EE8(a0 as _, a1 as _) as _ };
    fn grMaterial_801C8858(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { crate::tu::grmaterial::grMaterial_801C8858(a0 as _, a1 as _); } };
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn grMaterial_801C8DE0(a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32) = |a0: *mut HSD_GObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32| { unsafe { crate::tu::grmaterial::grMaterial_801C8DE0(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _); } };
    fn grMaterial_801C8E08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C8E08(a0 as _); } };
    fn grMaterial_801C8E28(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C8E28(a0 as _); } };
    fn grMaterial_801C8E68(a0: *mut HSD_GObj, a1: u32) = |a0: *mut HSD_GObj, a1: u32| { unsafe { crate::tu::grmaterial::grMaterial_801C8E68(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_8026B294(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::itacc::it_8026B294(a0 as _, a1 as _); } };
    fn it_8026D324(a0: i32) -> i32;
    fn it_8026F7C8(a0: *mut Vec, a1: *mut Vec, a2: i32) -> *mut HSD_GObj;
    fn lbAudioAx_80023F28(a0: i32) -> i32 = |a0: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023F28(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointFromLine(a0: i32) -> i32 = |a0: i32| unsafe { crate::tu::mplib::mpJointFromLine(a0 as _) as _ };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpJointSetCb2(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb2(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80055E9C(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E9C(a0 as _); } };
    fn mpLib_80057424(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057424(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_DObjClearFlags(a0: *mut HSD_DObj, a1: u32) { super::HSD_DObjClearFlags(a0, a1) }
        pub unsafe extern "C" fn HSD_DObjSetFlags(a0: *mut HSD_DObj, a1: u32) { super::HSD_DObjSetFlags(a0, a1) }
        pub unsafe extern "C" fn HSD_JObjGetDObj(a0: *mut HSD_JObj) -> *mut HSD_DObj { super::HSD_JObjGetDObj(a0) }
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

/// `struct grInishie1_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grInishie1_YakumonoParam {
    pub unk0: f32,
    pub unk4: f32,
    pub unk8: f32,
    pub unkC: f32,
    pub unk10: f32,
    pub unk14: i16,
    pub unk16: i16,
    pub unk18: u16,
    pub unk1A: i16,
    pub unk1C: i16,
    pub unk1E: i16,
    pub unk20: f32,
    pub unk24: f32,
    pub unk28: f32,
    pub unk2C: [Vec3; 2],
    pub unk44: f32,
    pub unk48: f32,
    pub unk4C: f32,
    pub unk50: f32,
}

/// `struct block_table_struct` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct block_table_struct {
    pub idx: i16,
    pub jobj_idx: i16,
}

/// `BLOCK_COUNT`: the 3 - 13 - 3 block pattern on the stage
pub const BLOCK_COUNT: usize = 19;

/// `static struct grInishie1_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grInishie1_YakumonoParam = null_mut();

/// `grI1_803B8268`: initial velocity for items spawned from hatena blocks
pub static grI1_803B8268: Vec3 = Vec3 { x: 0.0, y: 5.0, z: 0.0 };

/// `grI1_803E48C8`
pub static mut grI1_803E48C8: [GrJoint; 7] = [
    GrJoint { x: 13, y: 3, z: 3 },
    GrJoint { x: 14, y: 3, z: 14 },
    GrJoint { x: 15, y: 3, z: 15 },
    GrJoint { x: 16, y: 3, z: 16 },
    GrJoint { x: 17, y: 3, z: 17 },
    GrJoint { x: 18, y: 3, z: 18 },
    GrJoint { x: 19, y: 3, z: 19 },
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

/// `grI1_StageCallbacks`
pub static mut grI1_StageCallbacks: [StageCallbacks; 4] = [
    cbs(Some(stageGObj0_OnInit), Some(grInishie1_801FAACC), Some(grInishie1_801FAAD4), Some(grInishie1_801FAAD8), 0),
    cbs(Some(grInishie1_801FABB4), Some(grInishie1_801FABF4), Some(grInishie1_801FABFC), Some(grInishie1_801FAC00), 0),
    cbs(Some(grInishie1_801FAC04), Some(grInishie1_801FAC50), Some(grInishie1_801FAC58), Some(grInishie1_801FAD80), 0),
    cbs(Some(grInishie1_801FAADC), Some(grInishie1_801FAB60), Some(grInishie1_801FAB68), Some(grInishie1_801FABB0),
        (1 << 30) | (1 << 31)),
];

/// `grI1_StageData`
pub static mut grI1_StageData: StageData = StageData {
    grkind: Gr_Kind_Inishie1,
    callbacks: unsafe { &raw mut grI1_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrI1.dat\0".as_ptr() as *mut i8,
    on_init: Some(grInishie1_801FA90C),
    on_demo_init: Some(grInishie1_801FA908),
    on_load: Some(grInishie1_801FA984),
    on_start: Some(grInishie1_801FA988),
    callback4: Some(grInishie1_801FA9AC),
    on_touch_line: Some(grInishie1_801FCBB0),
    on_check_shadow_render: Some(grInishie1_801FCBB8),
    flags2: 1 << 0,
    joints: unsafe { &raw mut grI1_803E48C8 as *mut GrJoint },
    joint_count: 7,
};

const fn bt(idx: i16, jobj_idx: i16) -> block_table_struct {
    block_table_struct { idx, jobj_idx }
}

/// `grI1_803E49B8`
pub static grI1_803E49B8: [block_table_struct; BLOCK_COUNT] = [
    bt(13, 3), bt(14, 14), bt(15, 15), bt(16, 16), bt(17, 17), bt(18, 18), bt(19, 19), bt(11, 20), bt(12, 21), bt(1, 4),
    bt(2, 5), bt(3, 6), bt(4, 7), bt(5, 8), bt(6, 9), bt(7, 10), bt(8, 11), bt(9, 12), bt(10, 13),
];

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// `static inline fabsf_inline`
#[inline(always)]
pub fn fabsf_inline(x: f32) -> f32 {
    melee_core::ct!("fabsf_inline");
    if x < 0.0 { -x } else { x }
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

/// `static inline grInishie1_SetRespawnTimer`
pub unsafe fn grInishie1_SetRespawnTimer(gp: *mut Ground, index: u32, min: i32, mut max: i32) {
    melee_core::ct!("grInishie1_SetRespawnTimer");
    if max > min {
        let difference = max.wrapping_sub(min);
        max = min.wrapping_add(if difference != 0 { ext::HSD_Randi(difference) } else { 0 });
    } else if max < min {
        let difference = min.wrapping_sub(max);
        max = max.wrapping_add(if difference != 0 { ext::HSD_Randi(difference) } else { 0 });
    }
    (*(*gp).u.inishie1.block.add(index as usize)).x20 = max as i16;
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

/// `static inline Ground_UpdateWindAndMapColl` (gr/inlines.h)
unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut Ground_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// `HSD_JObjGetDObj`-walk of the DOBJ_LOOP macro: `HSD_DObjSetFlags(dobj, 1U)` on every DObj
unsafe fn dobj_set_all(jobj: *mut HSD_JObj) {
    let mut dobj = ext::HSD_JObjGetDObj(jobj);
    while !dobj.is_null() {
        ext::HSD_DObjSetFlags(dobj, 1);
        dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
    }
}

/// DOBJ_CLEAR_LOOP
unsafe fn dobj_clear_all(jobj: *mut HSD_JObj) {
    let mut dobj = ext::HSD_JObjGetDObj(jobj);
    while !dobj.is_null() {
        ext::HSD_DObjClearFlags(dobj, 1);
        dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
    }
}

/// `static inline grInishie1_ShowBlock`
pub unsafe fn grInishie1_ShowBlock(jobj: *mut HSD_JObj) {
    melee_core::ct!("grInishie1_ShowBlock");
    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
}

/// `static inline grInishie1_801FB3F0_UpdateY`
pub unsafe fn grInishie1_801FB3F0_UpdateY(block: *mut grInishie1_Block) {
    melee_core::ct!("grInishie1_801FB3F0_UpdateY");
    let y = (*block).x8 + (*block).xC;
    HSD_JObjSetTranslateY(&mut *(*block).jobj2, y, &mut Dirty);
}

/// `static inline grInishie1_801FB3F0_IsFar`
pub unsafe fn grInishie1_801FB3F0_IsFar(jobj: *mut HSD_JObj, pos: *mut Vec3) -> i32 {
    melee_core::ct!("grInishie1_801FB3F0_IsFar");
    ext::lb_8000B1CC(jobj, null_mut(), pos);
    ext::grLib_801C9EE8(pos, 15.0)
}

/// `static inline grInishie1_UpdateDObjFlags`
pub unsafe fn grInishie1_UpdateDObjFlags(jobj: *mut HSD_JObj, walker: *mut *mut HSD_DObj, hidden: bool) {
    melee_core::ct!("grInishie1_UpdateDObjFlags");
    *walker = ext::HSD_JObjGetDObj(jobj);
    while !(*walker).is_null() {
        if hidden {
            ext::HSD_DObjSetFlags(*walker, DOBJ_HIDDEN as u32);
        } else {
            ext::HSD_DObjClearFlags(*walker, DOBJ_HIDDEN as u32);
        }
        let next = if !(*walker).is_null() { (**walker).next } else { null_mut() };
        *walker = next;
    }
}

/// `static inline GET_ITEM2`
pub unsafe fn GET_ITEM2(arg0: *mut Item_GObj) -> *mut Item {
    melee_core::ct!("GET_ITEM2");
    (*arg0).user_data as *mut Item
}

/// 1FA908
pub unsafe extern "C" fn grInishie1_801FA908(_arg: i32) { melee_core::ct!("grInishie1_801FA908");}

/// 1FA90C
pub unsafe extern "C" fn grInishie1_801FA90C() {
    melee_core::ct!("grInishie1_801FA90C");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grInishie1_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    setupStageCallbacks(0);
    setupStageCallbacks(3);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Camera_800311DC(1.0);
    ext::Camera_800311CC(4000.0);
}

/// 1FA984
pub unsafe extern "C" fn grInishie1_801FA984() { melee_core::ct!("grInishie1_801FA984");}

/// 1FA988
pub unsafe extern "C" fn grInishie1_801FA988() {
    melee_core::ct!("grInishie1_801FA988");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 1FA9B4
pub unsafe extern "C" fn setupStageCallbacks(gobj_id: i32) -> *mut Ground_GObj {
    melee_core::ct!("setupStageCallbacks");
    let callbacks = &raw mut grI1_StageCallbacks[gobj_id as usize];
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
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 231, gobj_id);
    }

    gobj
}

/// `static stageGObj0_OnInit`
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut Ground_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

/// 1FAAD4
pub unsafe extern "C" fn grInishie1_801FAAD4(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FAAD4");}

/// 1FAAD8
pub unsafe extern "C" fn grInishie1_801FAAD8(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FAAD8");}

/// 1FAADC
pub unsafe extern "C" fn grInishie1_801FAADC(gobj: *mut Ground_GObj) {
    melee_core::ct!("grInishie1_801FAADC");
    let gp = GET_GROUND(gobj);

    Ground_InitMapCollAndAnim(gobj);
    grInishie1_801FAD84(gobj);
    grInishie1_801FC018(gobj);
    (*gp).u.map.set_xC4_b0(0);
    let j = ext::Ground_801C3FA4(gobj, 0x2D);
    ext::grMaterial_801C8858(j, 0x20000000);
}

/// 1FAB68
pub unsafe extern "C" fn grInishie1_801FAB68(gobj: *mut Ground_GObj) {
    melee_core::ct!("grInishie1_801FAB68");
    grInishie1_801FB3F0(gobj);
    grInishie1_801FC664(gobj);
    grInishie1_801FCB10(gobj);
    Ground_UpdateWindAndMapColl(gobj);
}

/// 1FABB0
pub unsafe extern "C" fn grInishie1_801FABB0(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FABB0");}

/// 1FABB4
pub unsafe extern "C" fn grInishie1_801FABB4(gobj: *mut Ground_GObj) {
    melee_core::ct!("grInishie1_801FABB4");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
}

/// 1FABFC
pub unsafe extern "C" fn grInishie1_801FABFC(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FABFC");}

/// 1FAC00
pub unsafe extern "C" fn grInishie1_801FAC00(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FAC00");}

/// 1FAC04
pub unsafe extern "C" fn grInishie1_801FAC04(gobj: *mut Ground_GObj) {
    melee_core::ct!("grInishie1_801FAC04");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.inishie12.xC4 = null_mut();
    (*gp).u.inishie12.xC8 = (*yakumono_param).unk16 as i32;
    (*gp).u.inishie12.xCC = 0;
}

/// 1FAC58
pub unsafe extern "C" fn grInishie1_801FAC58(gobj: *mut Ground_GObj) {
    melee_core::ct!("grInishie1_801FAC58");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let jobj2 = jobj;
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if !gp.is_null() && !jobj2.is_null() {
        if (*gp).u.inishie12.xCC == 0 {
            if (*gp).u.inishie12.xC8 > 0 {
                if (*gp).u.inishie12.xC8 & 1 != 0 {
                    ext::HSD_JObjClearFlagsAll(jobj2, JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjSetFlagsAll(jobj2, JOBJ_HIDDEN as u32);
                }
                (*gp).u.inishie12.xC8 -= 1;
            }
        }

        if !(*gp).u.inishie12.xC4.is_null() {
            ext::lb_8000B1CC((*gp).u.inishie12.xC4, null_mut(), &mut vec);
            HSD_JObjSetTranslate(&mut *jobj2, &vec, &mut Dirty);
        }
    }
}

/// 1FAD80
pub unsafe extern "C" fn grInishie1_801FAD80(_gobj: *mut Ground_GObj) { melee_core::ct!("grInishie1_801FAD80");}

/// 1FAD84
pub unsafe extern "C" fn grInishie1_801FAD84(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FAD84");
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.inishie1.block =
        ext::HSD_MemAlloc((core::mem::size_of::<grInishie1_Block>() * BLOCK_COUNT) as i32) as *mut grInishie1_Block;
    let block = (*gp).u.inishie1.block;

    let block_table = &grI1_803E49B8;

    let mut i: i32 = 0;
    while (i as u32) < BLOCK_COUNT as u32 {
        if i < 0 || i as u32 >= BLOCK_COUNT as u32 {
            panic!("grinishie1.c:0x1F9: oioi..");
        }
        let check = block_table[i as usize].idx;
        let value = check as i32;

        let mut j: u32 = 0;
        while j < BLOCK_COUNT as u32 {
            if check == block_table[j as usize].idx {
                break;
            }
            j += 1;
        }

        if j == BLOCK_COUNT as u32 {
            panic!("grinishie1.c:0x217: oioi..");
        }

        let jobj = ext::Ground_801C3FA4(gobj, grI1_803E49B8[j as usize].jobj_idx as i32);
        if jobj.is_null() {
            panic!("grinishie1.c:0x21D: oioi..");
        }

        let b = block.add(i as usize);
        (*b).x2 = 2;
        (*b).x4 = 0;
        (*b).jobj2 = jobj;
        (*b).hatena_gobj = null_mut();
        (*b).x8 = HSD_JObjGetTranslationY(&*jobj);
        (*b).xC = 0.0;
        (*b).x10 = 0.0;
        ext::mpJointSetCb2(value, gp as *mut c_void, Some(fn_801FBEB8));
        (*b).status = 0;
        i += 1;
    }

    i = 0;
    while (i as u32) < BLOCK_COUNT as u32 {
        let b = block.add(i as usize);
        let new_gobj = ext::grMaterial_801C8CFC(8, 0, gp, (*b).jobj2, None, Some(fn_801FBF6C), None);

        if !new_gobj.is_null() {
            ext::grMaterial_801C8DE0(new_gobj, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 6.0);
            ext::grMaterial_801C8E08(new_gobj);
            ext::grMaterial_801C8E68(new_gobj, 0);
        }

        (*b).item_gobj = new_gobj;
        i += 1;
    }

    if ext::gm_8016AE80() != -1 && ext::gm_8016B238() == 0 {
        let mut item_kind: i32 = It_Kind_Capsule as i32;
        while item_kind < It_Kind_L_Gun_Ray as i32 {
            if ext::it_8026D324(item_kind) != 0 {
                break;
            }
            item_kind += 1;
        }

        if item_kind != It_Kind_L_Gun_Ray as i32 {
            let index1 = ext::HSD_Randi(BLOCK_COUNT as i32);
            let mut index2;
            let index1_copy = index1;
            let mut index2_copy;

            loop {
                index2 = ext::HSD_Randi(BLOCK_COUNT as i32);
                index2_copy = index2;
                if index1 != index2_copy {
                    break;
                }
            }

            (*block.add(index1 as usize)).status = 1;
            grInishie1_801FBAA0(gobj, index1_copy);
            (*GET_GROUND((*block.add(index1 as usize)).hatena_gobj)).u.inishie12.xCC = 1;

            (*block.add(index2 as usize)).status = 2;
            grInishie1_801FBAA0(gobj, index2_copy);
            (*GET_GROUND((*block.add(index2 as usize)).hatena_gobj)).u.inishie12.xCC = 1;
        }
    }

    (*gp).u.inishie1.xC6 = 0;
    (*gp).u.inishie1.xC8 = 0;
    (*gp).u.inishie1.xCA = -1;
    (*gp).u.inishie1.xCC = -1;
    (*gp).u.inishie1.xD8 = 0;
}

/// 1FB0AC
pub unsafe extern "C" fn grInishie1_801FB0AC(gobj: *mut HSD_GObj, ix: u32) {
    melee_core::ct!("grInishie1_801FB0AC");
    let gp = GET_GROUND(gobj);
    let b = (*gp).u.inishie1.block.add(ix as usize);

    if (*b).status == 1 {
        grInishie1_801FBA34(gobj, (*b).jobj2);
        (*b).status = 0;
        grInishie1_801FBC4C(gobj, ix);

        (*gp).u.inishie1.xC6 = rand_range((*yakumono_param).unk4 as i32, (*yakumono_param).unk0 as i32) as i16;
    } else if (*b).status == 2 {
        grInishie1_801FBA34(gobj, (*b).jobj2);
        (*b).status = 0;
        grInishie1_801FBC4C(gobj, ix);

        (*gp).u.inishie1.xC8 = rand_range((*yakumono_param).unkC as i32, (*yakumono_param).unk8 as i32) as i16;
    } else if (*b).status == 3 {
        grInishie1_801FBA34(gobj, (*b).jobj2);
        (*b).status = 0;
        grInishie1_801FBC4C(gobj, ix);

        {
            let mut i: u32 = 0;
            while i < 19 {
                if (*(*gp).u.inishie1.block.add(i as usize)).status == 3 {
                    break;
                }
                i += 1;
            }
            if i == 19 {
                (*gp).u.inishie1.xC6 = rand_range((*yakumono_param).unk4 as i32, (*yakumono_param).unk0 as i32) as i16;
                (*gp).u.inishie1.xC8 = rand_range((*yakumono_param).unkC as i32, (*yakumono_param).unk8 as i32) as i16;
            }
        }

        (*gp).u.inishie1.xD8 = (*yakumono_param).unk10 as i32;
    } else if !(*b).hatena_gobj.is_null() {
        panic!("grinishie1.c:0x29D: assertion failed");
    }
}

/// `HATENA_APPEAR_CHECKLOOP` (macro): one of the two respawn countdowns
unsafe fn hatena_appear_checkloop(gobj: *mut HSD_GObj, vars: *mut Ground, countdown: *mut i16, last_index: *mut i16,
                                  status_val: i16, index: &mut i32) {
    if *countdown > 0 {
        *countdown -= 1;
        if *countdown == 0 {
            let mut i: i32 = 0;

            loop {
                let result = ext::HSD_Randi(0x13);
                *index = result;
                if result != *last_index as i32 && (*(*vars).u.inishie1.block.add(result as usize)).status == 0 {
                    break;
                }
                i += 1;
                if i >= 0x14 {
                    break;
                }
            }

            assert!(i != 0x14, "i!=HATENA_APPEAR_CHECKLOOP");

            *last_index = *index as i16;
            (*(*vars).u.inishie1.block.add(*index as usize)).status = status_val;
            grInishie1_801FBAA0(gobj, *index);
            (*(*vars).u.inishie1.block.add(*index as usize)).x20 = 0;
        }
    }
}

/// 1FB3F0
pub unsafe extern "C" fn grInishie1_801FB3F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FB3F0");
    let vars = (*gobj).user_data as *mut Ground;
    let mut index: i32;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if (*vars).u.inishie1.xD8 > 0 {
        (*vars).u.inishie1.xD8 -= 1;
    }

    if (*vars).u.inishie1.xD8 == 0
        && (((*vars).u.inishie1.xC6 == 1 && (*vars).u.inishie1.xC8 > 0 && (*vars).u.inishie1.xC8 < (*yakumono_param).unk14)
            || ((*vars).u.inishie1.xC8 == 1 && (*vars).u.inishie1.xC6 > 0 && (*vars).u.inishie1.xC6 < (*yakumono_param).unk14))
    {
        // this is likely the rare case mentioned on smashwiki
        // where every block will become a hatena block
        (*vars).u.inishie1.xC8 = 0;
        (*vars).u.inishie1.xC6 = 0;

        index = 0;
        while (index as u32) < 0x13 {
            (*(*vars).u.inishie1.block.add(index as usize)).status = 3;
            grInishie1_801FBAA0(gobj, index);
            (*(*vars).u.inishie1.block.add(index as usize)).x20 = 0;
            index += 1;
        }
    } else {
        index = 0;
        hatena_appear_checkloop(gobj, vars, &raw mut (*vars).u.inishie1.xC6, &raw mut (*vars).u.inishie1.xCA, 1, &mut index);
        hatena_appear_checkloop(gobj, vars, &raw mut (*vars).u.inishie1.xC8, &raw mut (*vars).u.inishie1.xCC, 2, &mut index);
    }

    index = 0;
    while (index as u32) < 0x13 {
        let blk = (*vars).u.inishie1.block.add(index as usize);
        let mut fall = false;
        match (*blk).x2 {
            2 => {
                if (*blk).x4 != 0 {
                    (*blk).x4 -= 1;
                }
            }
            3 => {
                (*blk).x10 = 1.2;
                grInishie1_801FB0AC(gobj, index as u32);
                (*blk).x2 = 4;
                fall = true;
            }
            4 => fall = true,
            0 => {
                if (*blk).x20 != 0 {
                    (*blk).x20 -= 1;
                } else {
                    let far = grInishie1_801FB3F0_IsFar((*blk).jobj2, &mut pos) != 0;
                    if !far {
                        let mut dobj = ext::HSD_JObjGetDObj((*blk).jobj2);
                        while !dobj.is_null() {
                            ext::HSD_DObjClearFlags(dobj, 1);
                            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
                        }
                        grInishie1_ShowBlock((*blk).jobj2);
                        ext::grMaterial_801C8E08((*blk).item_gobj);
                        (*blk).x2 = 1;
                        (*blk).x22 = (*yakumono_param).unk18 as i16;
                    }
                }

                if !(*blk).hatena_gobj.is_null() {
                    let sub_jobj = (*(*blk).hatena_gobj).hsd_obj as *mut HSD_JObj;
                    if ext::HSD_JObjGetFlags(sub_jobj) & 0x10 == 0 {
                        ext::HSD_JObjSetFlagsAll(sub_jobj, JOBJ_HIDDEN as u32);
                    }
                }
            }
            1 => {
                // probably sets the flickering for blocks when they spawn
                if (*blk).x22 > 0 {
                    ext::HSD_JObjGetDObj((*blk).jobj2);
                    if !(*blk).hatena_gobj.is_null() {
                        let target = (*(*blk).hatena_gobj).hsd_obj as *mut HSD_JObj;

                        if (*blk).x22 & 1 != 0 {
                            ext::HSD_JObjClearFlagsAll(target, JOBJ_HIDDEN as u32);
                        } else {
                            ext::HSD_JObjSetFlagsAll(target, JOBJ_HIDDEN as u32);
                        }

                        let mut dobj = ext::HSD_JObjGetDObj((*blk).jobj2);
                        while !dobj.is_null() {
                            ext::HSD_DObjClearFlags(dobj, 1);
                            dobj = if !dobj.is_null() { (*dobj).next } else { null_mut() };
                        }
                    } else if (*blk).x22 & 1 != 0 {
                        let mut dobj: *mut HSD_DObj = null_mut();
                        grInishie1_UpdateDObjFlags((*blk).jobj2, &mut dobj, false);
                    } else {
                        let mut dobj: *mut HSD_DObj = null_mut();
                        grInishie1_UpdateDObjFlags((*blk).jobj2, &mut dobj, true);
                    }

                    (*blk).x22 -= 1;
                } else {
                    (*blk).x2 = 2;
                    (*blk).x4 = 0;
                }
            }
            _ => {}
        }
        if fall {
            // case 4 (also reached by falling through from case 3)
            (*blk).xC += (*blk).x10;

            if (*blk).xC as f64 > 5.0 {
                (*blk).x10 = -1.2;
            } else if ((*blk).xC as f64) < 0.0 {
                (*blk).xC = 0.0;
                (*blk).x2 = 2;
                (*blk).x4 = 10;
            }

            // the blocks appear to not move at all, so not sure what this is
            grInishie1_801FB3F0_UpdateY(blk);
        }
        index += 1;
    }
}

/// 1FBA34: gives item from hatena block upwards velocity, maybe handles spawning it as well
pub unsafe extern "C" fn grInishie1_801FBA34(_gobj: *mut HSD_GObj, jobj: *mut HSD_JObj) {
    melee_core::ct!("grInishie1_801FBA34");
    let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut sp1c);
    let mut vec = grI1_803B8268;
    sp1c.y += 5.0;
    ext::it_8026F7C8(&mut sp1c, &mut vec, 0);
}

/// 1FBAA0
pub unsafe extern "C" fn grInishie1_801FBAA0(gobj: *mut HSD_GObj, ix: i32) {
    melee_core::ct!("grInishie1_801FBAA0");
    let mut position = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mapgp = (*gobj).user_data as *mut Ground;
    let hatena = setupStageCallbacks(2);

    if !hatena.is_null() {
        let hatena_gp = (*hatena).user_data as *mut Ground;
        let b = (*mapgp).u.inishie1.block.add(ix as usize);
        assert!((*b).hatena_gobj.is_null(), "!mapgp->u.map.block[ix].hatena_gobj");

        (*b).hatena_gobj = hatena;

        let hatena_jobj = (*hatena).hsd_obj as *mut HSD_JObj;
        ext::lb_8000B1CC((*b).jobj2, null_mut(), &mut position);
        HSD_JObjSetTranslate(&mut *hatena_jobj, &position, &mut Dirty);

        (*hatena_gp).u.inishie13.xC4 = (*b).jobj2;

        if (*b).x2 == 0 || (*b).x2 == 1 {
            (*hatena_gp).u.inishie13.xCC = 1;
        }

        dobj_set_all((*b).jobj2);
    }
}

/// 1FBC4C
pub unsafe extern "C" fn grInishie1_801FBC4C(gobj: *mut HSD_GObj, index: u32) {
    melee_core::ct!("grInishie1_801FBC4C");
    let gp = GET_GROUND(gobj);
    let b = (*gp).u.inishie1.block.add(index as usize);
    let jobj2 = (*b).jobj2;
    dobj_clear_all(jobj2);
    ext::Ground_801C4A08((*b).hatena_gobj);
    (*b).hatena_gobj = null_mut();
}

/// 1FBCEC
pub unsafe extern "C" fn grInishie1_801FBCEC(gobj: *mut HSD_GObj, index: u32) {
    melee_core::ct!("grInishie1_801FBCEC");
    let gp = (*gobj).user_data as *mut Ground;
    let mut effect_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let b = (*gp).u.inishie1.block.add(index as usize);

    (*b).x2 = 0;
    grInishie1_SetRespawnTimer(gp, index, (*yakumono_param).unk1A as i32, (*yakumono_param).unk1C as i32);

    let d = ext::HSD_JObjGetDObj((*b).jobj2);
    ext::HSD_DObjSetFlags(d, 1);

    ext::grMaterial_801C8E28((*b).item_gobj);

    {
        let status = (*b).status;
        if status == 1 || (status as i32 - 2) as u16 <= 1 {
            let mut item_vel;
            let mut item_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

            ext::Ground_801C4A08((*b).hatena_gobj);
            (*b).hatena_gobj = null_mut();

            ext::lb_8000B1CC((*b).jobj2, null_mut(), &mut item_pos);

            item_vel = grI1_803B8268;
            item_pos.y += 5.0;

            ext::it_8026F7C8(&mut item_pos, &mut item_vel, 0);
        }
    }

    ext::HSD_JObjSetFlagsAll((*b).jobj2, JOBJ_HIDDEN as u32);

    ext::it_8026B294((*b).item_gobj, &mut effect_pos);

    ext::efSync_Spawn(0x442, (*b).item_gobj, &mut effect_pos);

    ext::Ground_801C5414(0x136, 0xBA);
    ext::Camera_RequestQuake(QuakeKind_Small as u32, &mut effect_pos);
}

/// `static inline get_block_id`
pub unsafe fn get_block_id(block_id: i32) -> i32 {
    melee_core::ct!("get_block_id");
    let mut i = BLOCK_COUNT as i32;
    let mut idx: i32 = 0;
    while i != 0 {
        if block_id == grI1_803E49B8[idx as usize].idx as i32 {
            return idx;
        }
        idx += 1;
        i -= 1;
    }
    panic!("grinishie1.c:0x1F0: oioi..");
}

/// 1FBEB8 (`fn_801FBEB8`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn fn_801FBEB8(_user_data: *mut c_void, joint_id: i32, _coll: *mut CollData, _coll_x50: i32,
                                     _ground_kind: u32, delta_y: f32) {
    melee_core::ct!("fn_801FBEB8");
    let id = get_block_id(joint_id);
    if fabsf_inline(delta_y) as f64 > 0.7 {
        let gobj = ext::Ground_GetMapGObj(3);
        grInishie1_801FB0AC(gobj, id as u32);
        grInishie1_801FBCEC(gobj, id as u32);
    }
}

/// 1FBF6C (`fn_801FBF6C`)
pub unsafe extern "C" fn fn_801FBF6C(item_gobj: *mut Item_GObj, gp: *mut Ground, _pos: *mut Vec3, _arg3: *mut HSD_GObj,
                                     _arg4: f32) {
    melee_core::ct!("fn_801FBF6C");
    let ip = GET_ITEM2(item_gobj);

    let mut i: i32 = 0;
    while i < 19 {
        if (*(*gp).u.inishie1.block.add(i as usize)).jobj2 == (*ip).xDD4_itemVar.it_266F.x4 as *mut HSD_JObj {
            break;
        }
        i += 1;
    }

    if i as u32 == 19 {
        return;
    }

    let map_gobj = ext::Ground_GetMapGObj(3);
    assert!(!map_gobj.is_null(), "map_gobj");

    let new_var = map_gobj;
    grInishie1_801FB0AC(map_gobj, i as u32);
    grInishie1_801FBCEC(new_var, i as u32);
}

/// 1FC018
pub unsafe extern "C" fn grInishie1_801FC018(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FC018");
    let gp = (*gobj).user_data as *mut Ground;

    (*gp).u.inishie1.xEE = 0;
    (*gp).u.inishie1.xE0 = 0.0;
    (*gp).u.inishie1.xE4 = 0.0;
    (*gp).u.inishie1.xE8 = 0;
    (*gp).u.inishie1.xEA = 0;
    (*gp).u.inishie1.xF0 = 0.0;
    (*gp).u.inishie1.xF4 = 0.0;

    (*gp).u.inishie1.x108 = ext::Ground_801C3FA4(gobj, 0x1a);
    (*gp).u.inishie1.x10C = ext::Ground_801C3FA4(gobj, 0x1c);

    (*gp).u.inishie1.xFC = HSD_JObjGetTranslationY(&*(*gp).u.inishie1.x108);

    (*gp).u.inishie1.x100 = if (*gp).u.inishie1.x108.is_null() { null_mut() } else { (*(*gp).u.inishie1.x108).child };
    (*gp).u.inishie1.x104 = if (*gp).u.inishie1.x10C.is_null() { null_mut() } else { (*(*gp).u.inishie1.x10C).child };

    ext::mpJointSetCb1(0x15, gp as *mut c_void, Some(fn_801FC9AC));
    ext::mpJointSetCb1(0x14, gp as *mut c_void, Some(fn_801FC9AC));
}

/// 1FC110
pub unsafe extern "C" fn grInishie1_801FC110(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FC110");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;

    if (*gp).u.inishie1.xE0 > 100000.0 || (*gp).u.inishie1.xE4 > 100000.0 {
        (*gp).u.inishie1.xE4 = 100000.0;
        (*gp).u.inishie1.xE0 = 100000.0;
    }

    let mut dist_lo = (*gp).u.inishie1.xE0 - (*yp).unk2C[1].y;
    if dist_lo < 0.0 {
        dist_lo = 0.0;
    }
    let mut dist_hi = (*gp).u.inishie1.xE4 - (*yp).unk2C[1].y;
    if dist_hi < 0.0 {
        dist_hi = 0.0;
    }

    if dist_lo == 0.0 && dist_hi == 0.0 {
        if (if (*gp).u.inishie1.xF0 < 0.0 { -(*gp).u.inishie1.xF0 } else { (*gp).u.inishie1.xF0 }) > 0.0 {
            if (*gp).u.inishie1.xF0 < 0.0 {
                (*gp).u.inishie1.xF0 += (*yp).unk2C[1].x;
                if (*gp).u.inishie1.xF0 > 0.0 {
                    (*gp).u.inishie1.xF0 = 0.0;
                }
            } else {
                (*gp).u.inishie1.xF0 -= (*yp).unk2C[1].x;
                if (*gp).u.inishie1.xF0 < 0.0 {
                    (*gp).u.inishie1.xF0 = 0.0;
                }
            }
        }

        (*gp).u.inishie1.xF4 = 0.0;
    } else {
        let accel = dist_hi - dist_lo;
        (*gp).u.inishie1.xF4 = fmadds(accel, (*yp).unk24, (*gp).u.inishie1.xF4);

        if (*gp).u.inishie1.xF4 > (*yp).unk28 {
            (*gp).u.inishie1.xF4 = (*yp).unk28;
        } else if (*gp).u.inishie1.xF4 < -(*yp).unk28 {
            (*gp).u.inishie1.xF4 = -(*yp).unk28;
        }

        if dist_lo != 0.0 && dist_hi != 0.0 && (*gp).u.inishie1.xF4 != 0.0 {
            (*gp).u.inishie1.xF4 *= (*yp).unk2C[0].x;
        } else {
            (*gp).u.inishie1.xF4 *= (*yp).unk2C[0].y;

            if (*gp).u.inishie1.xF4 > 0.0 {
                (*gp).u.inishie1.xF4 -= (*yp).unk2C[0].z;
                if (*gp).u.inishie1.xF4 < 0.0 {
                    (*gp).u.inishie1.xF4 = 0.0;
                }
            } else if (*gp).u.inishie1.xF4 < 0.0 {
                (*gp).u.inishie1.xF4 += (*yp).unk2C[0].z;
                if (*gp).u.inishie1.xF4 > 0.0 {
                    (*gp).u.inishie1.xF4 = 0.0;
                }
            }
        }

        (*gp).u.inishie1.xF0 += (*gp).u.inishie1.xF4;
    }

    {
        if fabsf_inline((*gp).u.inishie1.xF0) > (*yp).unk20 {
            if (*gp).u.inishie1.xF0 < 0.0 {
                (*gp).u.inishie1.xF0 = -(*yp).unk20;
            } else {
                (*gp).u.inishie1.xF0 = (*yp).unk20;
            }
            (*gp).u.inishie1.xEE = 1;
            (*gp).u.inishie1.xF4 = 0.0;
        }

        HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x108, (*gp).u.inishie1.xFC + (*gp).u.inishie1.xF0, &mut Dirty);
        HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x10C, (*gp).u.inishie1.xFC - (*gp).u.inishie1.xF0, &mut Dirty);

        (*gp).u.inishie1.xE8 = 0;
        (*gp).u.inishie1.xEA = 0;
        (*gp).u.inishie1.xE0 = 0.0;
        (*gp).u.inishie1.xE4 = 0.0;
    }
}

/// 1FC4A0
pub unsafe extern "C" fn grInishie1_801FC4A0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FC4A0");
    let gp = (*gobj).user_data as *mut Ground;
    let mut reached: i32 = 0;
    let zero: f32 = 0.0;

    if (*gp).u.inishie1.xF0 != zero {
        if (*gp).u.inishie1.xF0 < zero {
            (*gp).u.inishie1.xF0 += (*yakumono_param).unk50;
            if (*gp).u.inishie1.xF0 >= zero {
                reached = 1;
            }
        } else {
            (*gp).u.inishie1.xF0 -= (*yakumono_param).unk50;
            if (*gp).u.inishie1.xF0 <= zero {
                reached = 1;
            }
        }
    }
    if reached != 0 {
        (*gp).u.inishie1.xF0 = 0.0;
        (*gp).u.inishie1.xF4 = 0.0;
        (*gp).u.inishie1.xE0 = 0.0;
        (*gp).u.inishie1.xE4 = 0.0;
        (*gp).u.inishie1.xEE = 0;
    }
    HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x108, (*gp).u.inishie1.xFC + (*gp).u.inishie1.xF0, &mut Dirty);
    HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x10C, (*gp).u.inishie1.xFC - (*gp).u.inishie1.xF0, &mut Dirty);
}

/// 1FC664
pub unsafe extern "C" fn grInishie1_801FC664(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FC664");
    let gp = (*gobj).user_data as *mut Ground;
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    match (*gp).u.inishie1.xEE {
        0 => {
            grInishie1_801FC110(gobj);
        }
        1 => {
            (*gp).u.inishie1.xF4 += (*yakumono_param).unk44;
            let t = (*yakumono_param).unk48;
            if (*gp).u.inishie1.xF4 > t {
                (*gp).u.inishie1.xF4 = t;
            }
            HSD_JObjAddTranslationY(&mut *(*gp).u.inishie1.x100, -(*gp).u.inishie1.xF4, &mut Dirty);
            HSD_JObjAddTranslationY(&mut *(*gp).u.inishie1.x104, -(*gp).u.inishie1.xF4, &mut Dirty);
            let bottom = -30.0f32 + ext::Stage_GetBlastZoneBottomOffset();
            ext::lb_8000B1CC((*gp).u.inishie1.x100, null_mut(), &mut sp18);
            ext::lb_8000B1CC((*gp).u.inishie1.x104, null_mut(), &mut sp24);
            if sp18.y < bottom && sp24.y < bottom {
                (*gp).u.inishie1.xEE = 2;
                (*gp).u.inishie1.xF4 = 0.0;
                (*gp).u.inishie1.xEC = (*yakumono_param).unk4C as i32 as i16;
            }
        }
        2 => {
            (*gp).u.inishie1.xEC = (*gp).u.inishie1.xEC.wrapping_sub(1);
            if (*gp).u.inishie1.xEC == 0 {
                (*gp).u.inishie1.xEE = 3;
                HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x100, 0.0, &mut Dirty);
                HSD_JObjSetTranslateY(&mut *(*gp).u.inishie1.x104, 0.0, &mut Dirty);
                ext::mpLib_80055E9C(0x14);
                ext::mpLib_80055E9C(0x15);
                ext::mpLib_80057424(0x14);
                ext::mpLib_80057424(0x15);
            }
        }
        3 => {
            grInishie1_801FC4A0(gobj);
        }
        _ => {}
    }
}

/// 1FC9AC (`fn_801FC9AC`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn fn_801FC9AC(user_data: *mut c_void, joint_id: i32, _coll: *mut CollData, coll_x50: i32,
                                     ground_kind: u32, _delta_y: f32) {
    melee_core::ct!("fn_801FC9AC");
    let gp = user_data as *mut Ground;
    if joint_id == 0x14 {
        (*gp).u.inishie1.xE8 = (*gp).u.inishie1.xE8.wrapping_add(1);
        if ground_kind == 1 {
            (*gp).u.inishie1.xE0 = fmadds(coll_x50 as f32, (*yakumono_param).unk2C[1].z, (*gp).u.inishie1.xE0);
        } else {
            (*gp).u.inishie1.xE0 += coll_x50 as f32;
        }
    } else if joint_id == 0x15 {
        (*gp).u.inishie1.xEA = (*gp).u.inishie1.xEA.wrapping_add(1);
        if ground_kind == 1 {
            (*gp).u.inishie1.xE4 = fmadds(coll_x50 as f32, (*yakumono_param).unk2C[1].z, (*gp).u.inishie1.xE4);
        } else {
            (*gp).u.inishie1.xE4 += coll_x50 as f32;
        }
    }
}

/// 1FCAAC
pub unsafe extern "C" fn grInishie1_801FCAAC(line_id: i32) -> i32 {
    melee_core::ct!("grInishie1_801FCAAC");
    if ext::stage_info.grkind == Gr_Kind_Inishie1 && line_id != -1 {
        let joint = (ext::mpJointFromLine(line_id) as u32).wrapping_sub(20);
        let mut result = 1;
        if joint > 1 {
            result = 0;
        }
        if result != 0 {
            return 1;
        }
    }
    0
}

/// 1FCB10
pub unsafe extern "C" fn grInishie1_801FCB10(gobj: *mut HSD_GObj) {
    melee_core::ct!("grInishie1_801FCB10");
    let mut time_remaining: u32 = 0;

    let gp = GET_GROUND(gobj);
    if (*gp).u.inishie2.xC4_flags.b0() == 0 {
        if ext::GetMatchTimer(&mut time_remaining as *mut u32 as *mut i32) != 0 {
            if time_remaining < 20 {
                let temp_r3 = ext::Ground_801C5A94();
                let var_r3 = if temp_r3 == 0x29 {
                    0x2A
                } else if temp_r3 == 0x2B {
                    0x2C
                } else {
                    -1
                };
                ext::lbAudioAx_80023F28(var_r3);
                (*gp).u.inishie2.xC4_flags.set_b0(1);
            }
        } else {
            (*gp).u.inishie2.xC4_flags.set_b0(1);
        }
    }
}

// ---- folded from grinishie1.rs (consolidation steps 21/22) ----
/// grInishie1_801FA988_fp (gr/grinishie1.c:246)
    pub unsafe extern "C" fn grInishie1_801FA988_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grinishie1.rs (consolidation steps 21/22) ----
/// grInishie1_801FA9AC (gr/grinishie1.c:251)
    pub extern "C" fn grInishie1_801FA9AC() -> i32 { 0 }

/// grInishie1_801FAACC (gr/grinishie1.c:296)
    pub extern "C" fn grInishie1_801FAACC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grInishie1_801FAB60 (gr/grinishie1.c:316)
    pub extern "C" fn grInishie1_801FAB60(a0: *mut HSD_GObj) -> i32 { 0 }

/// grInishie1_801FABF4 (gr/grinishie1.c:339)
    pub extern "C" fn grInishie1_801FABF4(a0: *mut HSD_GObj) -> i32 { 0 }

/// grInishie1_801FAC50 (gr/grinishie1.c:357)
    pub extern "C" fn grInishie1_801FAC50(a0: *mut HSD_GObj) -> i32 { 0 }

/// grInishie1_801FCBB0 (gr/grinishie1.c:1183)
    pub extern "C" fn grInishie1_801FCBB0(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }

/// grInishie1_801FCBB8 (gr/grinishie1.c:1188)
    pub extern "C" fn grInishie1_801FCBB8(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
