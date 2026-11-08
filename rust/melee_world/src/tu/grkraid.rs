//! src/melee/gr/grkraid.c: Brinstar's Kraid stage (Gr_Kind_Kraid). State is the file statics
//! (`yakumono_param`, `grKr_804D6A0C`), the per-gobj `Ground` (`u.kraid` / `u.kraid2`) and the
//! callback / stage-data tables; every other callee is a TODO hook in [`ext`]. The constant-return /
//! empty callbacks are the earlier round's ports (`crate::gr::trivial::grkraid`), re-exported.
//! Float code: no fused site (`fma_sites.py`); `(f32) / s32` divides by the converted int.
//! State-oracle checked against the C (tests/gro_grkraid.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::grinl::{GET_GROUND, GET_JOBJ, HSD_JObjAddRotationZ, HSD_JObjSetScale, HSD_JObjSetTranslateX};
use super::grinl2::{Ground_SetupStageCallbacks, Ground_StartMapAnim, Ground_UpdateWindAndMapColl};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;


/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use melee_types::*;
    pub use crate::tu::grinl::ext::{stage_info, HSD_JObjSetMtxDirtySub};
    pub use crate::tu::grinl2::ext::{
        grAnime_801C8138, grDisplay_801C5DB0, GObj_SetupGXLink, Ground_UpdateMapColl, HSD_GObj_SetupProc, HSD_Randi,
        lb_800115F4,
    };
    hook_or!(CAMERA_REQUESTQUAKE, Camera_RequestQuake, (a0: CmQuakeKind, a1: *mut Vec3), |a0: CmQuakeKind, a1: *mut Vec3| unsafe { crate::tu::camera::Camera_RequestQuake(a0 as _, a1 as _) });
    pub static mut GROUND_801C0498: Option<fn() -> f32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C0498() -> f32 {
        match unsafe { GROUND_801C0498 } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_801C0498() as _ }; } }
    }
    pub static mut GROUND_801C39C0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C39C0() {
        match unsafe { GROUND_801C39C0 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C39C0(); } return; } }
    }
    pub static mut GROUND_801C3BB4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3BB4() {
        match unsafe { GROUND_801C3BB4 } { Some(f) => f(), None => { unsafe { crate::tu::ground::Ground_801C3BB4(); } return; } }
    }
    pub static mut GROUND_801C3FA4: Option<fn(*mut HSD_GObj, i32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3FA4(gobj: *mut HSD_GObj, depth: i32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C3FA4 } { Some(f) => f(gobj, depth), None => { return unsafe { crate::tu::ground::Ground_801C3FA4(gobj as _, depth as _) as _ }; } }
    }
    pub static mut GROUND_801C53EC: Option<fn(u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C53EC(a0: u32) {
        match unsafe { GROUND_801C53EC } { Some(f) => f(a0), None => { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } return; } }
    }
    pub static mut GROUND_801C5440: Option<fn(*mut Ground, s32, u32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5440(gp: *mut Ground, i: s32, a2: u32) {
        match unsafe { GROUND_801C5440 } { Some(f) => f(gp, i, a2), None => { unsafe { crate::tu::ground::Ground_801C5440(gp as _, i as _, a2 as _); } return; } }
    }
    pub static mut GROUND_801C5544: Option<fn(*mut Ground, s32)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C5544(a0: *mut Ground, a1: s32) {
        match unsafe { GROUND_801C5544 } { Some(f) => f(a0, a1), None => { unsafe { crate::tu::ground::Ground_801C5544(a0 as _, a1 as _); } return; } }
    }
    pub static mut GROUND_GETMAPGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetMapGObj(map_id: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETMAPGOBJ } { Some(f) => f(map_id), None => { return unsafe { crate::tu::ground::Ground_GetMapGObj(map_id as _) as _ }; } }
    }
    pub static mut GROUND_GETSTAGEGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETSTAGEGOBJ } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ }; } }
    }
    pub static mut GROUND_GETYAKUMONOPARAM: Option<fn() -> *mut core::ffi::c_void> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetYakumonoParam() -> *mut core::ffi::c_void {
        match unsafe { GROUND_GETYAKUMONOPARAM } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ }; } }
    }
    pub static mut GROUND_INITMAPCOLL: Option<fn(*mut HSD_JObj, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_InitMapColl(jobj: *mut HSD_JObj, map_id: s32) -> i32 {
        match unsafe { GROUND_INITMAPCOLL } { Some(f) => f(jobj, map_id), None => { return unsafe { crate::tu::ground::Ground_InitMapColl(jobj as _, map_id as _) as _ }; } }
    }
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    hook_or!(OSREPORT, OSReport, (a0: *mut i8), |a0: *mut i8| unsafe { melee_runtime::tu::os_error::OSReport(a0 as _, &[]) });
    hook!(FTLIB_STARTRUMBLEALL, ftLib_StartRumbleAll, (a0: s32, a1: s32));
    pub static mut GRANIME_801C7FF8: Option<fn(*mut HSD_GObj, i32, i32, i32, f32, f32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) {
        match unsafe { GRANIME_801C7FF8 } { Some(f) => f(a0, a1, a2, a3, a4, a5), None => { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } return; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut LB_8000B1CC: Option<fn(*mut HSD_JObj, *mut Vec3, *mut Vec3)> = None;
    #[allow(non_snake_case)]
    pub fn lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3) {
        match unsafe { LB_8000B1CC } { Some(f) => f(jobj, pos0, pos1), None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, pos0 as _, pos1 as _); } return; } }
    }
    /// `HSD_PadStatus HSD_PadMasterStatus[4]` (baselib/controller.c).
    pub use melee_runtime::tu::controller::HSD_PadMasterStatus;
}

/// `struct grKraid_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grKraid_YakumonoParam {
    pub map_time_min: u32,
    pub map_time_max: u32,
    pub map_time_acl: s32,
    pub map_rot_spd_min: f32,
    pub map_rot_spd_max: f32,
    pub kraid_wait_time: u32,
    pub kraid_wait_time_add: u32,
    pub kraid_pos_x: [f32; 6],
}

const PAD_BUTTON_A: u32 = 0x0100;

/// `GrJoint grKr_803E4C78[]`.
pub static mut grKr_803E4C78: [GrJoint; 6] = [
    GrJoint { x: 0, y: 3, z: 12 },
    GrJoint { x: 1, y: 3, z: 12 },
    GrJoint { x: 2, y: 3, z: 12 },
    GrJoint { x: 3, y: 3, z: 12 },
    GrJoint { x: 4, y: 3, z: 12 },
    GrJoint { x: 5, y: 3, z: 12 },
];

macro_rules! cbs {
    ($a:expr, $b:expr, $c:expr, $d:expr, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: u32::to_be($flags) },
        }
    };
}

/// `StageCallbacks grKr_StageCallbacks[5]`.
pub static mut grKr_StageCallbacks: [StageCallbacks; 5] = [
    cbs!(stageGObj0_OnInit, grKraid_801FE1DC, grKraid_801FE1E4, grKraid_801FE1E8, 0),
    cbs!(grKraid_801FE1EC, grKraid_801FE2C8, grKraid_801FE2D0, grKraid_801FE35C, 0),
    cbs!(grKraid_801FE360, grKraid_801FE3A4, grKraid_801FE3AC, grKraid_801FE3B0, 0),
    cbs!(grKraid_801FE3B4, grKraid_801FE438, grKraid_801FE440, grKraid_801FE6D4, 0xC000_0000),
    cbs!(grKraid_801FE818, grKraid_801FE9F8, grKraid_801FEA00, grKraid_801FF14C, 0),
];

/// `StageData grKr_StageData`.
pub static mut grKr_StageData: StageData = StageData {
    grkind: Gr_Kind_Kraid,
    callbacks: unsafe { core::ptr::addr_of!(grKr_StageCallbacks) as *mut StageCallbacks },
    data1: b"/GrKr.dat\0".as_ptr() as *mut i8,
    on_init: Some(grKraid_OnInit),
    on_demo_init: Some(grKraid_OnDemoInit),
    on_load: Some(grKraid_OnLoad),
    on_start: Some(grKraid_OnStart),
    callback4: Some(grKraid_801FE0BC),
    on_touch_line: Some(grKraid_OnTouchLine),
    on_check_shadow_render: Some(grKraid_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: unsafe { core::ptr::addr_of!(grKr_803E4C78) as *mut GrJoint },
    joint_count: 6,
};

/// `static struct grKraid_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grKraid_YakumonoParam = null_mut();
/// `static int grKr_804D6A0C`.
pub static mut grKr_804D6A0C: i32 = 0;

pub unsafe extern "C" fn grKraid_OnInit() {
    melee_core::ct!("grKraid_OnInit");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grKraid_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grKraid_801FE0C4(0);
    grKraid_801FE0C4(2);
    grKraid_801FE0C4(3);
    let mut gobj = grKraid_801FE0C4(4);
    ext::HSD_JObjSetFlagsAll(GET_JOBJ(gobj), JOBJ_HIDDEN as u32);
    gobj = grKraid_801FE0C4(1);
    ext::HSD_JObjSetFlagsAll(GET_JOBJ(gobj), JOBJ_HIDDEN as u32);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grKraid_OnLoad() { melee_core::ct!("grKraid_OnLoad");}

pub unsafe extern "C" fn grKraid_OnStart() {
    melee_core::ct!("grKraid_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grKraid_801FE0C4(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grKraid_801FE0C4");
    let callbacks: *mut StageCallbacks = &mut grKr_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 223, gobj_id)
        let mut msg = *b"%s:%d: couldn t get gobj(id=%d)\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
    }

    gobj
}

/// `static void stageGObj0_OnInit(Ground_GObj*)`.
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grKraid_801FE1EC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE1EC");
    let gp = (*gobj).user_data as *mut Ground;
    let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let s = ext::Ground_801C0498();
    scale.z = s;
    scale.y = s;
    scale.x = s;
    HSD_JObjSetScale(GET_JOBJ(gobj), &scale);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grKraid_801FE2D0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE2D0");
    let master = ext::HSD_PadMasterStatus.as_mut_ptr();
    let gp = GET_GROUND(gobj);
    if (*master.add(1)).trigger & PAD_BUTTON_A != 0 {
        grKr_804D6A0C += 1;
        if grKr_804D6A0C > 6 {
            grKr_804D6A0C = 0;
        }
        let mut msg = *b"*** Req Effect Anime %d\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
        ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, grKr_804D6A0C);
    }
}

pub unsafe extern "C" fn grKraid_801FE360(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE360");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grKraid_801FE3B4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE3B4");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    (*gp).u.kraid.x0 = 0;
    (*gp).u.kraid.x4 = 0.0;
    (*gp).u.kraid.x8 = 0.0;
    (*gp).u.kraid.xC = 0.0;
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::grAnime_801C7FF8(gobj, 0xD, 0x7, 0x2, 0.0, 1.0);
}

/// `const Vec grKr_803B8278`.
pub static grKr_803B8278: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

pub unsafe extern "C" fn grKraid_801FE440(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE440");
    let gp = GET_GROUND(gobj);
    let jobj = ext::Ground_801C3FA4(gobj, 0);
    let mut pos = grKr_803B8278;
    match (*gp).u.kraid.x0 {
        0 => {
            if (*gp).u.kraid.x4 != 0.0 {
                (*gp).u.kraid.x10 = super::grinl2::rand_range((*yakumono_param).map_time_max as i32, (*yakumono_param).map_time_min as i32);
                (*gp).u.kraid.x0 = 1;
                ext::grAnime_801C7FF8(gobj, 18, 7, 1, 0.0, 1.0);
                ext::Ground_801C53EC(420007);
                (*gp).u.kraid.x1 = 1;
            }
        }
        1 => {
            if (*gp).u.kraid.x10 == 0 {
                (*gp).u.kraid.x8 -= (*gp).u.kraid.xC;
            } else {
                (*gp).u.kraid.x10 += -1;
            }
            let f_var2 = if (*gp).u.kraid.xC < 0.0 { -(*gp).u.kraid.xC } else { (*gp).u.kraid.xC };
            let mut f_var3 = (*gp).u.kraid.x8;
            let f_var4 = if f_var3 < 0.0 { -f_var3 } else { f_var3 };
            if f_var4 < f_var2 {
                (*gp).u.kraid.xC = 0.0;
                (*gp).u.kraid.x8 = 0.0;
                (*gp).u.kraid.x4 = 0.0;
                (*gp).u.kraid.x0 = 0;
            } else {
                f_var3 *= 3.14159265_f32 / 180.0_f32;
                HSD_JObjAddRotationZ(jobj, f_var3);
                ext::Camera_RequestQuake(QuakeKind_Loop, &mut pos);
            }
        }
        _ => {}
    }
    if (*gp).u.kraid.x1 != 0 && ext::grAnime_801C83D0(gobj, 18, 7) != 0 {
        ext::grAnime_801C7FF8(gobj, 18, 7, 0, 0.0, 1.0);
        (*gp).u.kraid.x1 = 0;
    }
    Ground_UpdateWindAndMapColl(gobj);
    grKraid_801FF150(gobj);
}

pub unsafe fn grKraid_801FE6D8(hand: *mut HSD_JObj, param2: f32) {
    melee_core::ct!("grKraid_801FE6D8");
    let map = (*ext::Ground_GetMapGObj(3)).user_data as *mut Ground;
    let mut handpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if (*map).u.kraid.x4 == 0.0 {
        (*map).u.kraid.x4 = param2;
        ext::lb_8000B1CC(hand, null_mut(), &mut handpos);
        let mut msg = *b"Kraid Hand Pos = %f\n\0";
        ext::OSReport(msg.as_mut_ptr() as *mut i8);
        let max = (*yakumono_param).map_rot_spd_max;
        let min = (*yakumono_param).map_rot_spd_min;
        let mut rot = (max / min) / 132.0_f32;
        rot *= min;
        (*yakumono_param).map_rot_spd_min = (*yakumono_param).map_rot_spd_min;
        (*map).u.kraid.x8 = rot * (if handpos.x < 0.0 { -handpos.x } else { handpos.x });
        if (*map).u.kraid.x8 < (*yakumono_param).map_rot_spd_min {
            (*map).u.kraid.x8 = (*yakumono_param).map_rot_spd_min;
        }
        if (*map).u.kraid.x8 > (*yakumono_param).map_rot_spd_max {
            (*map).u.kraid.x8 = (*yakumono_param).map_rot_spd_max;
        }
        (*map).u.kraid.x8 = (*map).u.kraid.x8 * (*map).u.kraid.x4;
        (*map).u.kraid.xC = (*map).u.kraid.x8 / ((*yakumono_param).map_time_acl as f32);
        ext::Ground_801C53EC(420005);
        ext::Camera_RequestQuake(QuakeKind_Medium, &mut handpos);
    }
}

pub unsafe extern "C" fn grKraid_801FE818(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FE818");
    let gp = GET_GROUND(gobj);
    (*gp).u.kraid2.x10 = ext::Ground_801C3FA4(gobj, 31);
    (*gp).u.kraid2.x14 = ext::Ground_801C3FA4(gobj, 47);
    let mut f_var3: f32 = 0.0;
    let mut f_var1: f32 = f_var3;
    (*gp).u.kraid2.x4 = 0;
    (*gp).u.kraid2.x3 = 0;

    // Slot 0 is compared against the zeros; slots 1..5 against the running min / max.
    let mut f_var2 = (*yakumono_param).kraid_pos_x[0];
    if f_var2 < 0.0 {
        (*gp).u.kraid2.x3 = 0;
        f_var3 = f_var2;
    } else if f_var2 > f_var1 {
        f_var1 = f_var2;
        (*gp).u.kraid2.x4 = 0;
    }
    for i in 1..6usize {
        f_var2 = (*yakumono_param).kraid_pos_x[i];
        if f_var2 < f_var3 {
            (*gp).u.kraid2.x3 = i as i8;
            f_var3 = f_var2;
        } else if f_var2 > f_var1 {
            f_var1 = f_var2;
            (*gp).u.kraid2.x4 = i as i8;
        }
    }
    (*gp).u.kraid2.x0 = 0;
    let add = (*yakumono_param).kraid_wait_time_add;
    (*gp).u.kraid2.xC = (*yakumono_param).kraid_wait_time.wrapping_add(if add as i32 != 0 { ext::HSD_Randi(add as i32) as u32 } else { 0 }) as i32;
    (*gp).u.kraid2.x8 = 0.0;
    let _ = f_var1;
    let _ = f_var3;
    let i_var: i8 = 0;
    (*gp).u.kraid2.x2 = i_var;
    (*gp).u.kraid2.x1 = i_var;
    (*gp).u.kraid2.x5 = 1;
}

pub unsafe extern "C" fn grKraid_801FEA00(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FEA00");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    match (*gp).u.kraid2.x0 {
        0 => {
            if (*gp).u.kraid2.xC <= 0 {
                let map = (*ext::Ground_GetMapGObj(3)).user_data as *mut Ground;
                if melee_core::ppc::fctiwz_f32((*map).u.kraid.x4) == 0 {
                    grKraid_801FEE54(gobj);
                    ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                    ext::HSD_JObjClearFlagsAll((*ext::Ground_GetMapGObj(1)).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
                    grKraid_801FF068(gobj, 3);
                    ext::Ground_801C53EC(420001);
                    ext::Ground_801C53EC(420008);
                    (*gp).u.kraid2.x0 = 1;
                }
            } else {
                (*gp).u.kraid2.xC += -1;
            }
        }
        1 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                grKraid_801FF068(gobj, ext::HSD_Randi(3));
                (*gp).u.kraid2.xC = 0;
                (*gp).u.kraid2.x0 = 2;
            }
        }
        2 => {
            let mut c_var3 = (*gp).u.kraid2.x1;
            if c_var3 == 0 && (*gp).u.kraid2.xC == 80 {
                ext::Ground_801C53EC(420000);
            } else if c_var3 == 1 && (*gp).u.kraid2.xC == 45 {
                ext::Ground_801C53EC(420000);
            } else if c_var3 == 2 && (*gp).u.kraid2.xC == 55 {
                ext::Ground_801C5440(gp, 0, 420000);
            }
            c_var3 = (*gp).u.kraid2.x1;
            if c_var3 == 2 && (*gp).u.kraid2.xC == 161 {
                ext::Ground_801C5544(gp, 0);
                ext::Ground_801C53EC(420004);
            } else if c_var3 == 2 && (*gp).u.kraid2.xC == 246 {
                ext::Ground_801C53EC(420004);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                // grKraid_801FF068(gobj, gp->u.kraid2.x8 + 5.0f): float -> int argument
                grKraid_801FF068(gobj, melee_core::ppc::fctiwz_f32((*gp).u.kraid2.x8 + 5.0_f32));
                (*gp).u.kraid2.xC = 0;
                (*gp).u.kraid2.x0 = 3;
            }
            (*gp).u.kraid2.xC += 1;
        }
        3 => {
            if (*gp).u.kraid2.x1 == 5 {
                let map = (*ext::Ground_GetMapGObj(3)).user_data as *mut Ground;
                if melee_core::ppc::fctiwz_f32((*map).u.kraid.x4) == 0 && (*gp).u.kraid2.xC >= 147 && (*gp).u.kraid2.xC <= 169 {
                    if (*gp).u.kraid2.xC == 147 {
                        ext::Ground_801C53EC(420003);
                    }
                    if grKraid_801FF0E0(gobj, 1) {
                        (*gp).u.kraid2.x5 = 1;
                        (*gp).u.kraid2.x8 = 1.0;
                        grKraid_801FE6D8((*gp).u.kraid2.x14, -1.0);
                    }
                }
                (*gp).u.kraid2.xC += 1;
            } else {
                let map = (*ext::Ground_GetMapGObj(3)).user_data as *mut Ground;
                if melee_core::ppc::fctiwz_f32((*map).u.kraid.x4) == 0 && (*gp).u.kraid2.xC >= 123 && (*gp).u.kraid2.xC <= 138 {
                    if (*gp).u.kraid2.xC == 123 {
                        ext::Ground_801C53EC(420003);
                    }
                    if grKraid_801FF0E0(gobj, 0) {
                        (*gp).u.kraid2.x5 = 1;
                        (*gp).u.kraid2.x8 = 1.0;
                        grKraid_801FE6D8((*gp).u.kraid2.x10, 1.0);
                    }
                }
                (*gp).u.kraid2.xC += 1;
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                grKraid_801FF068(gobj, 4);
                ext::Ground_801C53EC(420002);
                ext::Ground_801C53EC(420008);
                (*gp).u.kraid2.x0 = 4;
            }
        }
        4 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                (*gp).u.kraid2.xC = (*yakumono_param).kraid_wait_time.wrapping_add((*yakumono_param).kraid_wait_time_add) as i32;
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                ext::HSD_JObjSetFlagsAll((*ext::Ground_GetMapGObj(1)).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
                let add = (*yakumono_param).kraid_wait_time_add;
                (*gp).u.kraid2.xC = (*yakumono_param).kraid_wait_time.wrapping_add(if add as i32 != 0 { ext::HSD_Randi(add as i32) as u32 } else { 0 }) as i32;
                (*gp).u.kraid2.x0 = 0;
            }
        }
        _ => {}
    }
}

pub unsafe fn grKraid_801FEE54(gobj: *mut HSD_GObj) {
    melee_core::ct!("grKraid_801FEE54");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    let mut i_var3: i32;
    if (*gp).u.kraid2.x5 != 0 {
        loop {
            i_var3 = ext::HSD_Randi(6);
            if (*gp).u.kraid2.x2 as i32 != i_var3 {
                break;
            }
        }
    } else {
        i_var3 = if ext::HSD_Randi(2) != 0 { (*gp).u.kraid2.x3 as i32 } else { (*gp).u.kraid2.x4 as i32 };
    }
    (*gp).u.kraid2.x5 = 0;
    (*gp).u.kraid2.x2 = i_var3 as i8;
    HSD_JObjSetTranslateX(jobj, (*yakumono_param).kraid_pos_x[i_var3 as usize]);
    HSD_JObjSetTranslateX((*ext::Ground_GetMapGObj(1)).hsd_obj as *mut HSD_JObj, (*yakumono_param).kraid_pos_x[i_var3 as usize]);
    if (*yakumono_param).kraid_pos_x[i_var3 as usize] == 0.0 {
        i_var3 = if ext::HSD_Randi(2) != 0 { 1 } else { 0 };
        (*gp).u.kraid2.x8 = i_var3 as f32;
    } else if (*yakumono_param).kraid_pos_x[i_var3 as usize] < 0.0 {
        (*gp).u.kraid2.x8 = 0.0;
    } else {
        (*gp).u.kraid2.x8 = 1.0;
    }
}

pub unsafe fn grKraid_801FF068(gobj: *mut HSD_GObj, val: i32) {
    melee_core::ct!("grKraid_801FF068");
    let mut gp = GET_GROUND(gobj);

    (*gp).u.kraid.x1 = val as i8;
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.kraid.x1 as i32);
    let temp = (*gp).u.kraid.x1 as i32;
    let gobj = ext::Ground_GetMapGObj(1);
    gp = GET_GROUND(gobj);
    (*gp).u.kraid.x1 = temp as i8;
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, (*gp).u.kraid.x1 as i32);
    (*gp).u.kraid.x0 = 1;
}

pub unsafe fn grKraid_801FF0E0(gobj: *mut HSD_GObj, flag: i32) -> bool {
    melee_core::ct!("grKraid_801FF0E0");
    let gp = GET_GROUND(gobj);
    let val = if flag == 1 { 0x9e } else { 0x82 };
    let comp = (*gp).u.kraid2.xC >= val;
    if comp {
        ext::ftLib_StartRumbleAll(0xC, 0x0);
    }
    comp
}

pub unsafe fn grKraid_801FF150(_gobj: *mut HSD_GObj) { melee_core::ct!("grKraid_801FF150");}

#[allow(dead_code)]
type _Unused = c_void;

// ---- folded from grkraid.rs (consolidation steps 21/22) ----
/// grKraid_OnStart_fp (gr/grkraid.c:85)
    pub unsafe extern "C" fn grKraid_OnStart_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grkraid.rs (consolidation steps 21/22) ----
/// grKraid_OnDemoInit (gr/grkraid.c:60)
    pub extern "C" fn grKraid_OnDemoInit(a0: i32) {  }

/// grKraid_801FE0BC (gr/grkraid.c:90)
    pub extern "C" fn grKraid_801FE0BC() -> i32 { 0 }

/// grKraid_801FE1DC (gr/grkraid.c:116)
    pub extern "C" fn grKraid_801FE1DC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grKraid_801FE1E4 (gr/grkraid.c:121)
    pub extern "C" fn grKraid_801FE1E4(a0: *mut HSD_GObj) {  }

/// grKraid_801FE1E8 (gr/grkraid.c:126)
    pub extern "C" fn grKraid_801FE1E8(a0: *mut HSD_GObj) {  }

/// grKraid_801FE2C8 (gr/grkraid.c:141)
    pub extern "C" fn grKraid_801FE2C8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grKraid_801FE35C (gr/grkraid.c:160)
    pub extern "C" fn grKraid_801FE35C(a0: *mut HSD_GObj) {  }

/// grKraid_801FE3A4 (gr/grkraid.c:172)
    pub extern "C" fn grKraid_801FE3A4(a0: *mut HSD_GObj) -> i32 { 0 }

/// grKraid_801FE3AC (gr/grkraid.c:177)
    pub extern "C" fn grKraid_801FE3AC(a0: *mut HSD_GObj) {  }

/// grKraid_801FE3B0 (gr/grkraid.c:182)
    pub extern "C" fn grKraid_801FE3B0(a0: *mut HSD_GObj) {  }

/// grKraid_801FE438 (gr/grkraid.c:201)
    pub extern "C" fn grKraid_801FE438(a0: *mut HSD_GObj) -> i32 { 0 }

/// grKraid_801FE6D4 (gr/grkraid.c:260)
    pub extern "C" fn grKraid_801FE6D4(a0: *mut HSD_GObj) {  }

/// grKraid_801FE9F8 (gr/grkraid.c:382)
    pub extern "C" fn grKraid_801FE9F8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grKraid_801FF14C (gr/grkraid.c:563)
    pub extern "C" fn grKraid_801FF14C(a0: *mut HSD_GObj) {  }

/// grKraid_OnTouchLine (gr/grkraid.c:570)
    pub extern "C" fn grKraid_OnTouchLine(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }

/// grKraid_OnCheckShadowRender (gr/grkraid.c:575)
    pub extern "C" fn grKraid_OnCheckShadowRender(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
