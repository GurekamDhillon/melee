//! Port of src/melee/gr/grpura.c (Pokemon Stadium 2 / "Pura": the star-field background, the model
//! copies that follow the map joints, the camera subjects and the toon-texture compile pass). Every
//! function of the TU, native: pointers into the game's structs, callees of other TUs through `ext`
//! hooks. Checked against the C (whole TU, shipping contraction, stateful oracle) by
//! tests/tu_grpura.rs; regenerate the C side with `python rust/melee_world/gen/cref/run_gr.py grpura`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i8;
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
    fn Camera_80029020() -> *mut CmSubject;
    fn Camera_SetBackgroundColor(a0: u8, a1: u8, a2: u8) = |a0: u8, a1: u8, a2: u8| unsafe { crate::tu::camera::Camera_SetBackgroundColor(a0, a1, a2) };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C205C(a0: *mut _GXColor) = |a0: *mut _GXColor| { unsafe { crate::tu::ground::Ground_801C205C(a0 as _); } };
    fn Ground_801C33C0(a0: i32, a1: i32) -> i32 = |a0: i32, a1: i32| unsafe { crate::tu::ground::Ground_801C33C0(a0 as _, a1 as _) as _ };
    fn Ground_801C3880(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3880(a0 as _); } };
    fn Ground_801C3890(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3890(a0 as _); } };
    fn Ground_801C38A0(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C38A0(a0 as _); } };
    fn Ground_801C38AC(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C38AC(a0 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(a0 as _) as _ };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_MObjCompileTev(a0: *mut HSD_MObj) = |a0: *mut HSD_MObj| { unsafe { melee_runtime::tu::mobj::HSD_MObjCompileTev(a0 as _); } };
    fn HSD_MObjSetToonTextureImage(a0: *mut HSD_ImageDesc) = |a0: *mut HSD_ImageDesc| { unsafe { melee_runtime::tu::mobj::HSD_MObjSetToonTextureImage(a0 as _); } };
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Stage_GetCamBoundsBottomOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsBottomOffset() as _ };
    fn Stage_GetCamBoundsLeftOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsLeftOffset() as _ };
    fn Stage_GetCamBoundsRightOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsRightOffset() as _ };
    fn Stage_GetCamBoundsTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetCamBoundsTopOffset() as _ };
    fn Stage_UnkSetVec3TCam_Offset(a0: *mut Vec) = |a0: *mut Vec| { unsafe { crate::tu::stage::Stage_UnkSetVec3TCam_Offset(a0 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn lb_80011C18(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::lbspdisplay::lb_80011C18(a0 as _, a1 as _); } };
    fn mpJointUpdateBounding(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointUpdateBounding(a0 as _); } };
    fn mpLib_80055E24(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80055E24(a0 as _); } };
    fn mpVtxSetPos(a0: i32, a1: f32, a2: f32) = |a0: i32, a1: f32, a2: f32| { unsafe { crate::tu::mplib::mpVtxSetPos(a0 as _, a1 as _, a2 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
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

/// gr/inlines.h `Ground_InitMapCollAndAnim`.
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `union_type_dobj(o)`: `(o)->flags & (JOBJ_PTCL | JOBJ_SPLINE) ? false : true`
#[inline(always)]
unsafe fn union_type_dobj(o: *mut HSD_JObj) -> bool {
    (*o).flags & (JOBJ_PTCL | JOBJ_SPLINE) as u32 == 0
}

/// `struct GrPuModelDesc`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrPuModelDesc {
    pub x0: i32,
    pub x4: f32,
    pub x8: i32,
}
melee_core::gc_layout!(GrPuModelDesc, "GrPuModelDesc", size 0xC, align 4, [x0 @ 0x0: 4 "s32", x4 @ 0x4: 4 "f32", x8 @ 0x8: 4 "s32"]);

/// `struct GrPuVtxMapEntry`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GrPuVtxMapEntry {
    pub joint: GrJoint,
    pub x06: i16,
    pub jobj: *mut HSD_JObj,
}
melee_core::gc_layout!(GrPuVtxMapEntry, "GrPuVtxMapEntry", size 0xC, align 4, [
    joint @ 0x0: 6 "GrJoint", x06 @ 0x6: 2 "s16", jobj @ 0x8: 4 "HSD_JObj *",
]);

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags: u32::to_be(flags) } }
}

/// `stage_callbacks` (static in the C; 28 entries)
pub static mut stage_callbacks: [StageCallbacks; 28] = [
    cb(Some(stageGObj0_OnInit), Some(stageGObj0_Callback1), Some(stageGObj0_GObjProc), Some(stageGObj0_Callback3), 0),
    cb(Some(stageGObj1_OnInit), Some(stageGObj1_Callback1), Some(stageGObj1_GObjProc), Some(stageGObj1_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj4_OnInit), Some(stageGObj4_Callback1), Some(stageGObj4_GObjProc), Some(stageGObj4_Callback3), (1 << 30)),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cb(Some(stageGObj27_OnInit), Some(stageGObj27_Callback1), Some(stageGObj27_GObjProc), Some(stageGObj27_Callback3), (1 << 31)),
];

/// `grPu_StageData` (3E6A20-ish)
pub static mut grPu_StageData: StageData = StageData {
    grkind: Gr_Kind_Pura as _,
    callbacks: unsafe { &raw mut stage_callbacks as *mut StageCallbacks },
    data1: b"/GrPu.dat\0".as_ptr() as *mut i8,
    on_init: Some(grPura_80211D00),
    on_demo_init: Some(grPura_80211CFC),
    on_load: Some(grPura_80211DD8),
    on_start: Some(grPura_80211DDC),
    callback4: Some(grPura_80211E00),
    on_touch_line: Some(grPura_802130C0),
    on_check_shadow_render: Some(grPura_802130C8),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grPura_80211CFC(_num: i32) { melee_core::ct!("grPura_80211CFC");}

/// `static void* yakumono_param` (4D6AA0)
pub static mut yakumono_param: *mut c_void = null_mut();

pub unsafe extern "C" fn grPura_80211D00() {
    melee_core::ct!("grPura_80211D00");
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    yakumono_param = ext::Ground_GetYakumonoParam();
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    setupStageCallbacks(0);
    setupStageCallbacks(1);
    setupStageCallbacks(4);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
    ext::Ground_801C3880(0.8f32 * (ext::Stage_GetCamBoundsTopOffset() - cam_offset.y));
    ext::Ground_801C3890(0.8f32 * (ext::Stage_GetCamBoundsBottomOffset() - cam_offset.y));
    ext::Ground_801C38A0(0.8f32 * (ext::Stage_GetCamBoundsLeftOffset() - cam_offset.x));
    ext::Ground_801C38AC(0.8f32 * (ext::Stage_GetCamBoundsRightOffset() - cam_offset.x));
}

pub unsafe extern "C" fn grPura_80211DD8() { melee_core::ct!("grPura_80211DD8");}

pub unsafe extern "C" fn grPura_80211DDC() {
    melee_core::ct!("grPura_80211DDC");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grPura_80211E00() -> i32 {
    melee_core::ct!("grPura_80211E00");
    0
}

pub unsafe extern "C" fn grPura_80211E08_noinline(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grPura_80211E08_noinline");
    setupStageCallbacks(gobj_id)
}

pub unsafe extern "C" fn grPura_80211E08_noinline2(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grPura_80211E08_noinline2");
    grPura_80211E08_noinline(gobj_id)
}

/// Spells out `Ground_SetupStageCallbacks` (no `Mex_GrBind`).
pub unsafe extern "C" fn setupStageCallbacks(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("setupStageCallbacks");
    let callbacks = (&raw mut stage_callbacks as *mut StageCallbacks).add(gobj_id as usize);
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
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn stageGObj0_Callback1(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj0_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj0_GObjProc(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj0_GObjProc");}

pub unsafe extern "C" fn stageGObj0_Callback3(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj0_Callback3");}

pub unsafe extern "C" fn stageGObj27_OnInit(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj27_OnInit");
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn stageGObj27_Callback1(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj27_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj27_GObjProc(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj27_GObjProc");}

pub unsafe extern "C" fn stageGObj27_Callback3(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj27_Callback3");}

pub unsafe extern "C" fn stageGObj4_OnInit(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj4_OnInit");
    Ground_InitMapCollAndAnim(arg0);
    grPura_80212CD4(arg0);
    grPura_802125F0(arg0);
    grPura_80212FC0(arg0);
}

pub unsafe extern "C" fn stageGObj4_Callback1(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj4_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj4_GObjProc(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj4_GObjProc");
    grPura_80212EF4(arg0);
    ext::Ground_UpdateMapColl(arg0);
    grPura_80213030(arg0);
    ext::mpLib_80055E24(0x18);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn stageGObj4_Callback3(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj4_Callback3");}

/// `grPu_803E6AA0` (3E6AA0)
pub static mut grPu_803E6AA0: [_GXColor; 4] = [
    _GXColor { r: 0x00, g: 0x00, b: 0x00, a: 0xFF },
    _GXColor { r: 0x00, g: 0x00, b: 0x50, a: 0xFF },
    _GXColor { r: 0x1E, g: 0x46, b: 0x00, a: 0xFF },
    _GXColor { r: 0x87, g: 0x87, b: 0x00, a: 0xFF },
];

pub unsafe extern "C" fn stageGObj1_OnInit(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj1_OnInit");
    let gp = (*arg0).user_data as *mut Ground;
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
    (*gp).u.pura.xC4 = ext::HSD_Randi(4) as i16;
    loop {
        let u_var1 = ext::HSD_Randi(4) as u32;
        (*gp).u.pura.xC6 = u_var1 as i16;
        if (*gp).u.pura.xC4 != (*gp).u.pura.xC6 {
            break;
        }
    }
    let c = &mut grPu_803E6AA0[(*gp).u.pura.xC4 as usize] as *mut _GXColor;
    ext::Ground_801C205C(c);
    ext::Camera_SetBackgroundColor((*c).r, (*c).g, (*c).b);
    (*gp).u.pura.xC8 = 0;
}

pub unsafe extern "C" fn stageGObj1_Callback1(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj1_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj1_GObjProc(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj1_GObjProc");
    let gp = (*arg0).user_data as *mut Ground;

    if ((*gp).u.pura.xC8 as i32) < 0xE10 {
        let spil_c = grPu_803E6AA0[(*gp).u.pura.xC6 as usize];
        let mut sp18 = grPu_803E6AA0[(*gp).u.pura.xC4 as usize];
        let cur = (*gp).u.pura.xC8;
        let t = (cur as f32) / 3600.0f32;
        (*gp).u.pura.xC8 = cur.wrapping_add(1);
        // t * (f32)(spilC.x - sp18.x) + (f32) sp18.x: fused, then (s8) and stored into a u8
        sp18.r = f2i8(fmadds(t, (spil_c.r as i32 - sp18.r as i32) as f32, sp18.r as f32) as f64) as u8;
        sp18.g = f2i8(fmadds(t, (spil_c.g as i32 - sp18.g as i32) as f32, sp18.g as f32) as f64) as u8;
        sp18.b = f2i8(fmadds(t, (spil_c.b as i32 - sp18.b as i32) as f32, sp18.b as f32) as f64) as u8;
        ext::Ground_801C205C(&mut sp18);
        ext::Camera_SetBackgroundColor(sp18.r, sp18.g, sp18.b);
        return;
    }
    (*gp).u.pura.xC4 = (*gp).u.pura.xC6;
    loop {
        let u_var1 = ext::HSD_Randi(4) as u32;
        (*gp).u.pura.xC6 = u_var1 as i16;
        if (*gp).u.pura.xC4 != (*gp).u.pura.xC6 {
            break;
        }
    }
    (*gp).u.pura.xC8 = 0;
}

pub unsafe extern "C" fn stageGObj1_Callback3(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj1_Callback3");}

/// `grPu_803E6AB0` (3E6AB0)
pub static mut grPu_803E6AB0: [GrPuModelDesc; 27] = [
    GrPuModelDesc { x0: 0, x4: 1.0, x8: 2 }, GrPuModelDesc { x0: 1, x4: 1.0, x8: 3 }, GrPuModelDesc { x0: 2, x4: 1.0, x8: 5 },
    GrPuModelDesc { x0: 3, x4: 1.0, x8: 6 }, GrPuModelDesc { x0: 4, x4: 1.0, x8: 7 }, GrPuModelDesc { x0: 5, x4: 1.0, x8: 8 },
    GrPuModelDesc { x0: 6, x4: 1.0, x8: 9 }, GrPuModelDesc { x0: 7, x4: 1.0, x8: 10 }, GrPuModelDesc { x0: 8, x4: 1.0, x8: 11 },
    GrPuModelDesc { x0: 9, x4: 1.0, x8: 12 }, GrPuModelDesc { x0: 10, x4: 1.0, x8: 13 }, GrPuModelDesc { x0: 11, x4: 1.0, x8: 13 },
    GrPuModelDesc { x0: 12, x4: 1.0, x8: 13 }, GrPuModelDesc { x0: 13, x4: 1.0, x8: 14 }, GrPuModelDesc { x0: 14, x4: 1.0, x8: 15 },
    GrPuModelDesc { x0: 15, x4: 1.0, x8: 16 }, GrPuModelDesc { x0: 16, x4: 1.0, x8: 17 }, GrPuModelDesc { x0: 17, x4: 1.0, x8: 18 },
    GrPuModelDesc { x0: 18, x4: 1.0, x8: 19 }, GrPuModelDesc { x0: 19, x4: 1.0, x8: 20 }, GrPuModelDesc { x0: 20, x4: 1.0, x8: 21 },
    GrPuModelDesc { x0: 21, x4: 1.0, x8: 22 }, GrPuModelDesc { x0: 22, x4: 1.0, x8: 23 }, GrPuModelDesc { x0: 23, x4: 1.0, x8: 24 },
    GrPuModelDesc { x0: 24, x4: 1.0, x8: 25 }, GrPuModelDesc { x0: 25, x4: 1.0, x8: 26 }, GrPuModelDesc { x0: -1, x4: 1.0, x8: -1 },
];

const fn vm(x: i16, y: i16, z: i16) -> GrPuVtxMapEntry {
    GrPuVtxMapEntry { joint: GrJoint { x, y, z }, x06: 0, jobj: null_mut() }
}

/// `grPu_803E6C0C` (3E6C0C): the map joint to vertex table; `jobj` is filled by `grPura_80212FC0`
pub static mut grPu_803E6C0C: [GrPuVtxMapEntry; 44] = [
    vm(20, 2, 21), vm(21, 2, 22), vm(22, 2, 23), vm(23, 2, 24), vm(24, 2, 25), vm(25, 2, 26), vm(26, 2, 27), vm(39, 2, 28),
    vm(284, 12, 21), vm(285, 12, 23), vm(267, 12, 24), vm(261, 12, 25), vm(262, 12, 26), vm(275, 12, 27), vm(276, 12, 28),
    vm(277, 12, 29), vm(269, 12, 30), vm(270, 12, 31), vm(272, 12, 32), vm(286, 12, 33),
    vm(180, 8, 23), vm(181, 8, 51), vm(182, 8, 27), vm(183, 8, 30), vm(184, 8, 31),
    vm(513, 25, 10), vm(514, 25, 13), vm(520, 25, 16), vm(521, 25, 19), vm(522, 25, 22), vm(496, 25, 25), vm(497, 25, 28),
    vm(500, 25, 31), vm(523, 25, 32), vm(524, 25, 29), vm(527, 25, 26), vm(528, 25, 23), vm(525, 25, 20), vm(526, 25, 17),
    vm(494, 25, 14), vm(495, 25, 11),
    vm(-1, 0, 0), vm(0, 0, 0), vm(0, 0, 0),
];

/// `grPu_803E6E20` (3E6E20): the 32x32 toon texture (GX image data, 32-byte aligned)
#[repr(C, align(32))]
pub struct GrPuTex(pub [u16; 1024]);
pub static mut grPu_803E6E20: GrPuTex = GrPuTex([
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E,
    0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E,
    0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
    0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E,
    0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E, 0x738E,
    0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410, 0x8410,
    0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534, 0x8410, 0x8410, 0xA534, 0xA534,
    0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534, 0xA534,
    0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79, 0xCE79,
    0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF, 0xCE79, 0xCE79, 0xFFFF, 0xFFFF,
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF,
]);

/// `grPu_803E7620` (3E7620)
pub static mut grPu_803E7620: HSD_ImageDesc = HSD_ImageDesc {
    image_ptr: unsafe { &raw mut grPu_803E6E20 as *mut c_void },
    width: 32,
    height: 32,
    format: 4,
    mipmap: 0,
    minLOD: 0.0,
    maxLOD: 0.0,
};

pub unsafe extern "C" fn stageGObj2_OnInit(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    (*arg0).render_cb = Some(fn_802130D0);
    ext::HSD_MObjSetToonTextureImage(&raw mut grPu_803E7620);
    ext::lb_80011C18(jobj, 0x1000);
    grPura_80213250(jobj);
    ext::HSD_MObjSetToonTextureImage(null_mut());
    ext::grAnime_801C8138(arg0, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn stageGObj2_Callback1(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("stageGObj2_Callback1");
    0
}

pub unsafe extern "C" fn stageGObj2_GObjProc(arg0: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_GObjProc");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut vec2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut quat = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    let gp = (*arg0).user_data as *mut Ground;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;

    HSD_JObjGetTranslation2(&*(*gp).u.pura2.xC8, &mut vec);
    HSD_JObjSetTranslate(&mut *jobj, &vec, &mut Dirty);
    HSD_JObjGetRotation(&*(*gp).u.pura2.xC8, &mut quat);
    HSD_JObjSetRotation(&mut *jobj, &quat, &mut Dirty);
    HSD_JObjGetScale(&*(*gp).u.pura2.xC8, &mut vec2);
    HSD_JObjSetScale(&mut *jobj, &vec2, &mut Dirty);

    if (ext::HSD_JObjGetFlags((*gp).u.pura2.xC8) & 0x10 != 0) && (ext::HSD_JObjGetFlags(jobj) & 0x10 == 0) {
        ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    } else if (ext::HSD_JObjGetFlags((*gp).u.pura2.xC8) & 0x10 == 0) && (ext::HSD_JObjGetFlags(jobj) & 0x10 != 0) {
        ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
    }
}

pub unsafe extern "C" fn stageGObj2_Callback3(_arg0: *mut HSD_GObj) { melee_core::ct!("stageGObj2_Callback3");}

pub unsafe extern "C" fn grPura_802125F0(arg0: *mut HSD_GObj) {
    melee_core::ct!("grPura_802125F0");
    let mut desc = (&raw mut grPu_803E6AB0) as *mut GrPuModelDesc;

    let mut i = 0u32;
    while i < 27 {
        if (*desc).x8 != -1 {
            let gobj = setupStageCallbacks((*desc).x8);
            assert!(!gobj.is_null(), "HSD_ASSERT(0x291, gobj) grpura.c");
            let gp = (*gobj).user_data as *mut Ground;
            assert!(!gp.is_null(), "HSD_ASSERT(0x292, gp) grpura.c");

            (*gp).u.pura.xC4 = (*desc).x0 as i16;
            let joint = ext::Ground_801C33C0(4, (*gp).u.pura.xC4 as i32);
            (*gp).u.pura2.xC8 = ext::Ground_801C3FA4(arg0, joint);

            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            HSD_JObjSetTranslateX(&mut *jobj, HSD_JObjGetTranslationX(&*(*gp).u.pura2.xC8), &mut Dirty);
            HSD_JObjSetTranslateY(&mut *jobj, HSD_JObjGetTranslationY(&*(*gp).u.pura2.xC8), &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *jobj, HSD_JObjGetTranslationZ(&*(*gp).u.pura2.xC8), &mut Dirty);

            if ext::HSD_JObjGetFlags((*gp).u.pura2.xC8) & 0x10 != 0 {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }

            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            let child = if jobj.is_null() { null_mut() } else { (*jobj).child };
            HSD_JObjSetTranslateX(&mut *child, 0.0, &mut Dirty);
            HSD_JObjSetTranslateY(&mut *child, 0.0, &mut Dirty);
            HSD_JObjSetTranslateZ(&mut *child, 0.0, &mut Dirty);

            let jobj2 = (*gp).u.pura2.xC8;
            let mut scale = HSD_JObjGetScaleX(&*jobj2);
            if scale < 2.0 {
                scale *= (*desc).x4;
            }
            HSD_JObjSetScaleX(&mut *jobj2, scale, &mut Dirty);
            HSD_JObjSetScaleY(&mut *jobj2, scale, &mut Dirty);
            HSD_JObjSetScaleZ(&mut *jobj2, scale, &mut Dirty);

            let jobj3 = (*gobj).hsd_obj as *mut HSD_JObj;
            HSD_JObjSetScaleX(&mut *jobj3, scale, &mut Dirty);
            HSD_JObjSetScaleY(&mut *jobj3, scale, &mut Dirty);
            HSD_JObjSetScaleZ(&mut *jobj3, scale, &mut Dirty);
        }
        i += 1;
        desc = desc.add(1);
    }
}

pub unsafe extern "C" fn grPura_80212CD4(arg0: *mut HSD_GObj) {
    melee_core::ct!("grPura_80212CD4");
    let gp = (*arg0).user_data as *mut Ground;
    let gp2 = gp;
    let jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    let mut subject_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    for i in 0..25usize {
        (*gp).u.pura3.xC4[i] = null_mut();
        (*gp).u.pura3.x128[i] = null_mut();
    }
    let mut node = HSD_JObjGetChild(if jobj.is_null() { None } else { Some(&*jobj) });

    if !node.is_null() {
        node = HSD_JObjGetChild(Some(&*node));
        let mut i = 0usize;
        while i < 25 && !node.is_null() {
            (*gp).u.pura3.x128[i] = ext::Camera_80029020();
            if !(*gp).u.pura3.x128[i].is_null() {
                (*gp).u.pura3.xC4[i] = node;
                ext::lb_8000B1CC((*gp2).u.pura3.xC4[i], null_mut(), &mut subject_pos);
                let s = (*gp).u.pura3.x128[i];
                (*s).pos = subject_pos;
                if ext::HSD_JObjGetFlags(node) & 0x10 != 0 {
                    (*s).state = CmSubjectState_Inactive;
                } else {
                    (*s).state = CmSubjectState_Active;
                }
                (*s).target_ext.v.x = 30.0;
                (*s).target_ext.v.y = -30.0;
                (*s).target_ext.h.x = -30.0;
                (*s).target_ext.h.y = 30.0;
                {
                    let subject = (*gp).u.pura3.x128[i];
                    (*subject).ext.h = (*subject).target_ext.h;
                    (*subject).ext.v = (*subject).target_ext.v;
                }
            }
            i += 1;
            node = HSD_JObjGetNext(Some(&*node));
        }
    }
}

pub unsafe extern "C" fn grPura_80212EF4(arg0: *mut HSD_GObj) {
    melee_core::ct!("grPura_80212EF4");
    let gp = (*arg0).user_data as *mut Ground;
    let gp2 = gp;
    let mut sp_c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    for i in 0..25usize {
        if !(*gp).u.pura3.xC4[i].is_null() && !(*gp).u.pura3.x128[i].is_null() {
            ext::lb_8000B1CC((*gp2).u.pura3.xC4[i], null_mut(), &mut sp_c);
            (*(*gp).u.pura3.x128[i]).pos = sp_c;
            if ext::HSD_JObjGetFlags((*gp2).u.pura3.xC4[i]) & 0x10 != 0 {
                (*(*gp).u.pura3.x128[i]).state = CmSubjectState_Inactive;
            } else {
                (*(*gp).u.pura3.x128[i]).state = CmSubjectState_Active;
            }
        }
    }
}

pub unsafe extern "C" fn grPura_80212FC0(_arg0: *mut HSD_GObj) {
    melee_core::ct!("grPura_80212FC0");
    let mut var_r30 = 0usize;
    while var_r30 < 0x2A {
        if grPu_803E6C0C[var_r30].joint.x != -1 {
            let temp_r3 = ext::Ground_GetMapGObj(grPu_803E6C0C[var_r30].joint.y as i32);
            if !temp_r3.is_null() {
                grPu_803E6C0C[var_r30].jobj = ext::Ground_801C3FA4(temp_r3, grPu_803E6C0C[var_r30].joint.z as i32);
            }
        }
        var_r30 += 1;
    }
}

pub unsafe extern "C" fn grPura_80213030(_arg0: *mut HSD_GObj) {
    melee_core::ct!("grPura_80213030");
    let mut sp_c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut var_r30 = 0usize;
    while var_r30 < 0x2A {
        if !grPu_803E6C0C[var_r30].jobj.is_null() {
            ext::lb_8000B1CC(grPu_803E6C0C[var_r30].jobj, null_mut(), &mut sp_c);
            ext::mpVtxSetPos(grPu_803E6C0C[var_r30].joint.x as i32, sp_c.x, sp_c.y);
        }
        var_r30 += 1;
    }
    ext::mpJointUpdateBounding(0);
    ext::mpJointUpdateBounding(9);
    ext::mpJointUpdateBounding(0x18);
    ext::mpJointUpdateBounding(5);
}

pub unsafe extern "C" fn grPura_802130C0(_arg0: i32) -> *mut DynamicsDesc {
    melee_core::ct!("grPura_802130C0");
    null_mut()
}

pub unsafe extern "C" fn grPura_802130C8(_a: *mut Vec3, _num: i32, _joint: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grPura_802130C8");
    1
}

pub unsafe extern "C" fn fn_802130D0(arg0: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("fn_802130D0");
    ext::HSD_MObjSetToonTextureImage(&raw mut grPu_803E7620);
    ext::grDisplay_801C5DB0(arg0, arg1);
    ext::HSD_MObjSetToonTextureImage(null_mut());
}

pub unsafe extern "C" fn grPura_80213128(dobj: *mut HSD_DObj) {
    melee_core::ct!("grPura_80213128");
    let next = (*dobj).next;
    if !next.is_null() {
        let next2 = (*next).next;
        if !next2.is_null() {
            if !(*next2).next.is_null() {
                grPura_80213128((*next2).next);
            }
            let mut iter = next2;
            while !iter.is_null() {
                grPura_80213224(iter);
                iter = (*iter).next;
            }
            if !(*next2).mobj.is_null() {
                ext::HSD_MObjCompileTev((*next2).mobj);
            }
        }
        let mut iter = next;
        while !iter.is_null() {
            ext::HSD_MObjCompileTev((*iter).mobj);
            iter = (*iter).next;
        }
        if !(*next).mobj.is_null() {
            ext::HSD_MObjCompileTev((*next).mobj);
        }
    }
    let mut iter = dobj;
    while !iter.is_null() {
        ext::HSD_MObjCompileTev((*iter).mobj);
        iter = (*iter).next;
    }
    if !(*dobj).mobj.is_null() {
        ext::HSD_MObjCompileTev((*dobj).mobj);
    }
}

pub unsafe extern "C" fn grPura_80213224(dobj: *mut HSD_DObj) {
    melee_core::ct!("grPura_80213224");
    if !dobj.is_null() {
        ext::HSD_MObjCompileTev((*dobj).mobj);
    }
}

pub unsafe extern "C" fn grPura_80213250(jobj: *mut HSD_JObj) {
    melee_core::ct!("grPura_80213250");
    let child = (*jobj).child;

    if !child.is_null() {
        if !(*child).child.is_null() {
            grPura_80213250((*child).child);
        }
        if !(*child).next.is_null() {
            grPura_80213250((*child).next);
        }
        if union_type_dobj(child) {
            let dobj = (*child).u.dobj;
            if !dobj.is_null() {
                if !(*dobj).next.is_null() {
                    grPura_80213128((*dobj).next);
                }
                let mut iter = dobj;
                while !iter.is_null() {
                    grPura_80213224(iter);
                    iter = (*iter).next;
                }
                if !(*dobj).mobj.is_null() {
                    ext::HSD_MObjCompileTev((*dobj).mobj);
                }
            }
        }
    }

    {
        let sibling = (*jobj).next;
        if !sibling.is_null() {
            if !(*sibling).child.is_null() {
                grPura_80213250((*sibling).child);
            }
            if !(*sibling).next.is_null() {
                grPura_80213250((*sibling).next);
            }
            if union_type_dobj(sibling) {
                let sibling_dobj = (*sibling).u.dobj;
                if !sibling_dobj.is_null() {
                    if !(*sibling_dobj).next.is_null() {
                        grPura_80213128((*sibling_dobj).next);
                    }
                    let mut sibling_iter = sibling_dobj;
                    while !sibling_iter.is_null() {
                        grPura_80213224(sibling_iter);
                        sibling_iter = (*sibling_iter).next;
                    }
                    if !(*sibling_dobj).mobj.is_null() {
                        ext::HSD_MObjCompileTev((*sibling_dobj).mobj);
                    }
                }
            }
        }
    }

    if union_type_dobj(jobj) {
        let self_dobj = (*jobj).u.dobj;
        if !self_dobj.is_null() {
            let self_next = (*self_dobj).next;
            if !self_next.is_null() {
                if !(*self_next).next.is_null() {
                    grPura_80213128((*self_next).next);
                }
                let mut self_iter = self_next;
                while !self_iter.is_null() {
                    grPura_80213224(self_iter);
                    self_iter = (*self_iter).next;
                }
                if !(*self_next).mobj.is_null() {
                    ext::HSD_MObjCompileTev((*self_next).mobj);
                }
            }
            let mut self_iter = self_dobj;
            while !self_iter.is_null() {
                ext::HSD_MObjCompileTev((*self_iter).mobj);
                self_iter = (*self_iter).next;
            }
            if !(*self_dobj).mobj.is_null() {
                ext::HSD_MObjCompileTev((*self_dobj).mobj);
            }
        }
    }
}

// ---- folded from grpura.rs (consolidation steps 21/22) ----
/// grPura_80211DDC_fp (gr/grpura.c:321)
    pub unsafe extern "C" fn grPura_80211DDC_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grpura.rs (consolidation steps 21/22) ----
/// grPura_80211E00_fp (gr/grpura.c:326)
    pub extern "C" fn grPura_80211E00_fp() -> i32 { 0 }

/// stageGObj27_Callback1_fp (gr/grpura.c:391)
    pub extern "C" fn stageGObj27_Callback1_fp(a0: *mut HSD_GObj) -> i32 { 0 }

/// grPura_802130C0_fp (gr/grpura.c:875)
    pub extern "C" fn grPura_802130C0_fp(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }

/// grPura_802130C8_fp (gr/grpura.c:880)
    pub extern "C" fn grPura_802130C8_fp(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
