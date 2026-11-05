//! src/melee/gr/grshrineroute.c: the Underground Maze ("Shrine Route") stage: the six route symbols
//! and the spot events, the swaying chandelier, the moving stage lights (the same light picker as
//! grpushon), the player-detection joint callback and the TARGET_PC script query.
//!
//! The constant/empty callbacks already ported in `gr::trivial` are re-exported; the rest is ported
//! here. Callees outside this TU are `tu_ext` hooks (unset: panic `TODO ... not ported`).
//! State-oracle checked by tests/tu_grshrineroute.rs against gen/turef/tu_grshrineroute_ref.c.
//!
//! `Ground.u.shrineroute.xD4` is a C `u32` that holds a gobj pointer (`(u32) temp`, read back as
//! `(HSD_GObj*) gp->...xD4`); the port keeps the C field and its truncating casts, which hold for
//! the 32-bit-addressable arena the oracle and the layout tests use.
//!
//! `grShrineRoute_8020AA40`'s `OSReport("%s:%d: oioi...\n")` passes no arguments for its two
//! conversions (the C reads garbage varargs); the port reports it with an empty argument list and
//! the oracle never reaches it (the light chain always has 15 or more objects).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use super::gr_util::*;
use crate::tu_ext as ext;
use core::ffi::c_void;
use melee_core::fp::{fmadds, fmsubs};
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_types::inline as mi;
use melee_types::*;


/// `struct grShrineRoute_YakumonoParam`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grShrineRoute_YakumonoParam {
    pub x0: *mut c_void,
    pub x4: *mut c_void,
    pub x8: *mut c_void,
    pub xC: *mut c_void,
    pub x10: i32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
    pub x24: i32,
    pub spawn_desc: grZakoGenerator_SpawnDesc,
}
melee_core::gc_layout!(grShrineRoute_YakumonoParam, "grShrineRoute_YakumonoParam", size 0x2C, align 4, [
    x0 @ 0x0: 4 "void*",
    x4 @ 0x4: 4 "void*",
    x8 @ 0x8: 4 "void*",
    xC @ 0xC: 4 "void*",
    x10 @ 0x10: 4 "int",
    x14 @ 0x14: 4 "f32",
    x18 @ 0x18: 4 "f32",
    x1C @ 0x1C: 4 "f32",
    x20 @ 0x20: 4 "f32",
    x24 @ 0x24: 4 "int",
    spawn_desc @ 0x28: 4 "grZakoGenerator_SpawnDesc",
]);

/// `struct grSh_Route_LightConfig`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grSh_Route_LightConfig {
    pub color: GXColor,
    pub pos: Vec3,
    pub interest: Vec3,
    pub r#type: i32,
    pub cutoff: f32,
    pub point_func: i32,
    pub ref_br: f32,
    pub ref_dist: f32,
    pub dist_func: i32,
}
melee_core::gc_layout!(grSh_Route_LightConfig, "grSh_Route_LightConfig", size 0x34, align 4, [
    color @ 0x0: 4 "GXColor",
    pos @ 0x4: 12 "Vec3",
    interest @ 0x10: 12 "Vec3",
    r#type @ 0x1C: 4 "s32",
    cutoff @ 0x20: 4 "f32",
    point_func @ 0x24: 4 "GXSpotFn",
    ref_br @ 0x28: 4 "f32",
    ref_dist @ 0x2C: 4 "f32",
    dist_func @ 0x30: 4 "GXDistAttnFn",
]);

/// `static s16 grSh_Route_803E58E0[]`.
pub static grSh_Route_803E58E0: [i16; 8] = [51, 79, 101, 102, 115, 116, 131, 0];

const fn cb(
    on_init: unsafe extern "C" fn(*mut HSD_GObj),
    callback1: unsafe extern "C" fn(*mut HSD_GObj) -> i32,
    gobj_proc: unsafe extern "C" fn(*mut HSD_GObj),
    callback3: unsafe extern "C" fn(*mut HSD_GObj),
    flags: u32,
) -> StageCallbacks {
    StageCallbacks {
        on_init: Some(on_init),
        callback1: Some(callback1),
        gobj_proc: Some(gobj_proc),
        callback3: Some(callback3),
        anon_10: StageCallbacks_anon_135 { flags: u32::to_be(flags) },
    }
}

/// `static StageCallbacks stage_callbacks[]`.
pub static mut stage_callbacks: [StageCallbacks; 7] = [
    cb(stageGObj0_OnInit, grShrineRoute_802089D8, grShrineRoute_802089E0, grShrineRoute_802089E4, 0),
    cb(stageGObj1_3_OnInit, grShrineRoute_8020A894, grShrineRoute_8020A89C, grShrineRoute_8020A8A0, 0),
    cb(grShrineRoute_802089E8, grShrineRoute_80208A28, grShrineRoute_80208A30, grShrineRoute_80208A34, 0),
    cb(stageGObj1_3_OnInit, grShrineRoute_8020A894, grShrineRoute_8020A89C, grShrineRoute_8020A8A0, 0),
    cb(grShrineRoute_80208D14, grShrineRoute_80208F0C, grShrineRoute_80208F70, grShrineRoute_80209AEC, 0xC0000000),
    cb(grShrineRoute_80209AF0, grShrineRoute_80209BE4, grShrineRoute_80209BEC, grShrineRoute_8020A100, 0),
    cb(grShrineRoute_8020A104, grShrineRoute_8020A214, grShrineRoute_8020A21C, grShrineRoute_8020A864, 0),
];

pub static mut grSh_Route_StageData: StageData = StageData {
    grkind: Gr_Kind_ShrineRoute,
    callbacks: unsafe { &raw mut stage_callbacks as *mut StageCallbacks },
    data1: c"/GrNSr.dat".as_ptr() as *mut i8,
    on_init: Some(grShrineRoute_OnInit),
    on_demo_init: Some(grShrineRoute_OnDemoInit),
    on_load: Some(grShrineRoute_OnLoad),
    on_start: Some(grShrineRoute_OnStart),
    callback4: Some(grShrineRoute_802088B8),
    on_touch_line: Some(grShrineRoute_OnTouchLine),
    on_check_shadow_render: Some(grShrineRoute_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};

/// `static struct grShrineRoute_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grShrineRoute_YakumonoParam = core::ptr::null_mut();

pub unsafe extern "C" fn grShrineRoute_OnDemoInit(_arg: i32) { melee_core::ct!("grShrineRoute_OnDemoInit");}

pub unsafe extern "C" fn grShrineRoute_OnInit() {
    melee_core::ct!("grShrineRoute_OnInit");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grShrineRoute_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(0);
    grShrineRoute_802088C0(0);
    grShrineRoute_802088C0(4);
    grShrineRoute_802088C0(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    grShrineRoute_8020B0AC();
}

pub unsafe extern "C" fn grShrineRoute_OnLoad() {
    melee_core::ct!("grShrineRoute_OnLoad");
    grShrineRoute_802088C0(6);
}

pub unsafe extern "C" fn grShrineRoute_OnStart() {
    melee_core::ct!("grShrineRoute_OnStart");
    ext::grZakoGenerator_801CAE04(&raw mut (*yakumono_param).spawn_desc);
    let mut val = (*yakumono_param).x24;
    if val != 0 {
        val = ext::HSD_Randi((*yakumono_param).x24);
    } else {
        val = 0;
    }
    if val == 0 {
        // PPC evaluates call arguments left to right
        let a = ext::Ground_801C5840();
        let b = ext::Ground_801C5940();
        ext::grZakoGenerator_801CAEB0(a, b);
    }
}

pub unsafe extern "C" fn grShrineRoute_802088C0(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grShrineRoute_802088C0");
    let callbacks = (&raw mut stage_callbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);
        if (*callbacks).callback3.is_some() {
            (*gp).x1C_callback = (*callbacks).callback3;
        }
        if let Some(on_init) = (*callbacks).on_init {
            on_init(gobj);
        }
        if (*callbacks).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    } else {
        report_no_gobj(c"grshrineroute.c".as_ptr(), 271, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grShrineRoute_802089E0(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_802089E0");}

pub unsafe extern "C" fn grShrineRoute_802089E4(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_802089E4");}

pub unsafe extern "C" fn grShrineRoute_802089E8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_802089E8");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
}

pub unsafe extern "C" fn grShrineRoute_80208A30(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_80208A30");}

pub unsafe extern "C" fn grShrineRoute_80208A34(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_80208A34");}

pub unsafe extern "C" fn fn_80208A38(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_80208A38");
    let mut flags = [0u8; 6];
    let gp = (*gobj).user_data as *mut Ground;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    {
        let idx = ext::HSD_Randi(6);
        flags[idx as usize] = 1;
    }

    for i in 0..6usize {
        let jobj = ext::Ground_801C2CF4(i as i32 + 0xBD);
        if !jobj.is_null() {
            if flags[i] != 0 {
                (*gp).u.map.symbol[i] = grShrineRoute_802088C0(3);
            } else {
                (*gp).u.map.symbol[i] = grShrineRoute_802088C0(1);
            }
            ext::lb_8000B1CC(jobj, core::ptr::null_mut(), &mut pos);
            let effect = (*gp).u.map.symbol[i];
            if !effect.is_null() {
                let ejobj = GET_JOBJ(effect);
                if !ejobj.is_null() {
                    mi::HSD_JObjSetTranslate(&mut *ejobj, &pos, &mut Dirty);
                    mi::HSD_JObjSetScaleX(&mut *ejobj, (*yakumono_param).x14, &mut Dirty);
                    mi::HSD_JObjSetScaleY(&mut *ejobj, (*yakumono_param).x14, &mut Dirty);
                    mi::HSD_JObjSetScaleZ(&mut *ejobj, (*yakumono_param).x14, &mut Dirty);
                }
            }
        }
    }
}

pub unsafe extern "C" fn grShrineRoute_80208D14(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_80208D14");
    let gp = (*gobj).user_data as *mut Ground;
    let mut center = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    ext::mpLib_80058560();
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 1) as *mut c_void);
    ext::grMaterial_801C94D8(ext::Ground_801C3FA4(gobj, 1) as *mut c_void);

    (*gp).u.shrineroute.xC4 = 0;
    (*gp).u.shrineroute.xC6 = 0;
    (*gp).u.shrineroute.xC8 = 0;
    (*gp).u.shrineroute.xCE = 0;
    (*gp).u.shrineroute.xD4 = 0;
    (*gp).u.shrineroute.xCA = 0;
    (*gp).u.shrineroute.xCC = 0;

    (*gp).x11_flags.set_b012(2);
    ext::stage_info.x6DC = 0;

    for i in 0..6usize {
        (*gp).u.map.symbol[i] = core::ptr::null_mut();
    }

    ext::ftCo_800C07F8(gobj, 3, grShrineRoute_8020AE08 as *mut c_void);
    let g = gp as *mut c_void;
    for joint in 8..=13 {
        ext::mpJointSetCb1(joint, g, Some(onJointCollision));
    }

    if ext::Ground_801C2D24(0x94, &mut center) != 0 {
        for i in 0..3usize {
            let jobj = ext::Ground_801C2CF4(i as i32 + 0x7F);
            (*gp).u.shrineroute.platforms[i].jobj = jobj;
            if !jobj.is_null() {
                mi::HSD_JObjGetTranslation(
                    &*(*gp).u.shrineroute.platforms[i].jobj,
                    &mut (*gp).u.shrineroute.platforms[i].offset,
                );
                ext::lbVector_Sub(&mut (*gp).u.shrineroute.platforms[i].offset, &mut center);
            }
        }
    } else {
        (*gp).u.shrineroute.platforms[0].jobj = core::ptr::null_mut();
        (*gp).u.shrineroute.platforms[1].jobj = core::ptr::null_mut();
        (*gp).u.shrineroute.platforms[2].jobj = core::ptr::null_mut();
    }

    ext::Ground_801C10B8(gobj, Some(fn_80208A38));
}

pub unsafe extern "C" fn grShrineRoute_80208F14() -> i32 {
    melee_core::ct!("grShrineRoute_80208F14");
    let gobj = ext::Ground_GetMapGObj(4);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        if !gp.is_null() && (*gp).u.shrineroute.xCA != 0 && (*gp).u.shrineroute.xCC != 0 {
            return 1;
        }
    }
    0
}

/// The `if (result != id) { hide joint j; mpLib_80057BC0(m) }` arms of case 0.
unsafe fn hide_other(gobj: *mut HSD_GObj, result: i32, id: i32, jidx: i32, m: i32) {
    if result != id {
        let j = ext::Ground_801C3FA4(gobj, jidx);
        if !j.is_null() {
            ext::HSD_JObjSetFlagsAll(j, JOBJ_HIDDEN as u32);
        }
        ext::mpLib_80057BC0(m);
    }
}

/// The `if (pid != id) { show joint j; mpJointListAdd(m) }` arms of case 5.
unsafe fn show_other(gobj: *mut HSD_GObj, pid: i32, id: i32, jidx: i32, m: i32) {
    if pid != id {
        let j = ext::Ground_801C3FA4(gobj, jidx);
        if !j.is_null() {
            ext::HSD_JObjClearFlagsAll(j, JOBJ_HIDDEN as u32);
        }
        ext::mpJointListAdd(m);
    }
}

pub unsafe extern "C" fn grShrineRoute_80208F70(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_80208F70");
    let mut sp88 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp7C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp68 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp5C = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp50 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp44 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = (*gobj).user_data as *mut Ground;
    let mut track_plat: i32 = 0;
    let yp = yakumono_param;

    let player = ext::Ground_GetP1Fighter();
    if !player.is_null() {
        ext::ftLib_80086644(player, &mut sp7C);
    } else {
        sp7C.z = 0.0;
        sp7C.y = 0.0;
        sp7C.x = 0.0;
    }

    match (*gp).u.shrineroute.xC4 {
        0 => {
            track_plat = 0;
            // PPC evaluates call arguments left to right
            let radius = (*yp).x20 * ext::Ground_801C0498();
            let a1 = (*yp).x1C * ext::Ground_801C0498();
            let result = ext::Ground_801C3DB4(grShrineRoute_80208F14 as *mut c_void, a1, radius);
            if result != -1 {
                let ix = result - 0xBD;
                if (*gp).u.shrineroute.xC6 as i32 & (<i32 as melee_core::ppc::Ppc>::pshl(1, (ix) as i64)) == 0 {
                    (*gp).u.shrineroute.xC8 = result as u16;
                    if (*gp).u.map.symbol[ix as usize].is_null() {
                        panic!("assert: grshrineroute.c:0x213 gp->u.map.symbol[ix]");
                    }
                    let mid = (*((*(*gp).u.map.symbol[ix as u32 as usize]).user_data as *mut Ground)).map_id as i32;
                    if !player.is_null() {
                        ext::Player_80031790(ext::ftLib_80086BE0(player) as i32);
                    }
                    if mid == 3 {
                        (*gp).u.shrineroute.xC4 = 6;
                        (*gp).u.shrineroute.xD0 = 0x3C;
                    } else {
                        (*gp).u.shrineroute.xC4 = 1;
                        (*gp).u.shrineroute.xCE = 0x3C;
                        for m in 0..8 {
                            ext::mpLib_80057BC0(m);
                        }
                        let temp = grShrineRoute_802088C0(5);
                        (*gp).u.shrineroute.xD4 = temp as usize as u32;
                        if !temp.is_null() {
                            let ejobj = (*((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj)).hsd_obj as *mut HSD_JObj;
                            ext::HSD_JObjSetFlagsAll(ejobj, JOBJ_HIDDEN as u32);
                            ext::Ground_801C2D24(result, &mut sp88);
                            mi::HSD_JObjSetTranslate(&mut *ejobj, &sp88, &mut Dirty);
                        }
                        ext::grMaterial_801C9604(gobj, (*yp).x0, 0);
                        let temp = ext::Ground_GetMapGObj(2);
                        if !temp.is_null() {
                            ext::grMaterial_801C9604(temp, (*yp).x0, 0);
                        }
                        hide_other(gobj, result, 0xBD, 0x13, 8);
                        hide_other(gobj, result, 0xBE, 0x14, 9);
                        hide_other(gobj, result, 0xBF, 0x12, 0xA);
                        hide_other(gobj, result, 0xC0, 0x11, 0xB);
                        hide_other(gobj, result, 0xC1, 0x10, 0xC);
                        hide_other(gobj, result, 0xC2, 0xF, 0xD);
                        ext::stage_info.x6DC = 1;
                        {
                            let dst = ext::Ground_801C2CF4(5);
                            let src = ext::Ground_801C2CF4(result - 0xA);
                            if !dst.is_null() && !src.is_null() {
                                mi::HSD_JObjGetTranslation(&*src, &mut sp68);
                                mi::HSD_JObjSetTranslate(&mut *dst, &sp68, &mut Dirty);
                            }
                        }
                        ext::grZakoGenerator_801CAF08();
                        grShrineRoute_8020B020(gobj, (*gp).u.shrineroute.xC8 as i32, 1);
                    }
                }
            }
        }
        1 => {
            track_plat = 1;
            if ext::grLib_801C96E8(gobj) != 0 {
                (*gp).u.shrineroute.xC4 = 2;
                ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 1), JOBJ_HIDDEN as u32);
                ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 1), JOBJ_HIDDEN as u32);
                {
                    let gr2 = ext::Ground_GetMapGObj(2);
                    if !gr2.is_null() {
                        ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gr2, 0), JOBJ_HIDDEN as u32);
                    }
                }
                ext::grAnime_801C7A04(gobj, 0, 7, 0.0);
                ext::grLib_801C9908((*gobj).hsd_obj as *mut HSD_JObj);
                if (*gp).u.shrineroute.xD4 != 0 {
                    ext::grMaterial_801C9604((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj, (*yp).x4, 0);
                }
            }
        }
        2 => {
            track_plat = 1;
            if (*gp).u.shrineroute.xD4 != 0 {
                let dg = (*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj;
                let ejobj = (*dg).hsd_obj as *mut HSD_JObj;
                if ext::HSD_JObjGetFlags(ejobj) & 0x10 != 0 {
                    let anim = (*dg).user_data;
                    ext::HSD_JObjClearFlagsAll(ejobj, JOBJ_HIDDEN as u32);
                    ext::grAnime_801C8138(dg, (*(anim as *mut Ground)).map_id as i32, 0);
                }
            }
            if ((*gp).u.shrineroute.xD4 != 0
                && ext::grLib_801C96E8((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj) != 0)
                || (*gp).u.shrineroute.xD4 == 0
            {
                (*gp).u.shrineroute.xC4 = 3;
                ext::Ground_801C5750();
                ext::gm_801674C4(6, 1, 1, ((*gp).u.shrineroute.xC8 as i32 - 0xA) as i8, Some(grShrineRoute_8020AD24));
                ext::Player_80031900();
                grShrineRoute_8020AF38(gobj, (*gp).u.shrineroute.xC8 as i32);
            }
        }
        3 => {
            track_plat = 1;
            if ext::Ground_801C5764() == 1 && (*gp).u.shrineroute.xCA != 0 && (*gp).u.shrineroute.xCC != 0 {
                (*gp).u.shrineroute.xC6 =
                    ((*gp).u.shrineroute.xC6 as i32 | (<i32 as melee_core::ppc::Ppc>::pshl(1, (((*gp).u.shrineroute.xC8 as i32 - 0xBD)) as i64))) as u16;
                ext::grZakoGenerator_801CAEF0(1);
                grShrineRoute_8020B020(gobj, -1, 0);
                (*gp).u.shrineroute.xCE = 0x3C;
                for m in 0..8 {
                    ext::mpJointListAdd(m);
                }
                grShrineRoute_8020B0AC();
                if (*gp).u.shrineroute.xD4 != 0 {
                    ext::grMaterial_801C9604((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj, (*yp).x8, 0);
                }
                ext::stage_info.x6DC = 0;
                (*gp).u.shrineroute.xC4 = 4;
            }
        }
        4 => {
            track_plat = 0;
            if ((*gp).u.shrineroute.xD4 != 0
                && ext::grLib_801C96E8((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj) != 0)
                || (*gp).u.shrineroute.xD4 == 0
            {
                (*gp).u.shrineroute.xC4 = 5;
                ext::grLib_801C9908((*((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj)).hsd_obj as *mut HSD_JObj);
                if (*gp).u.shrineroute.xD4 != 0 {
                    ext::Ground_801C4A08((*gp).u.shrineroute.xD4 as usize as *mut HSD_GObj);
                    (*gp).u.shrineroute.xD4 = 0;
                }
                ext::grMaterial_801C9604(gobj, (*yp).xC, 0);
                {
                    let gr2 = ext::Ground_GetMapGObj(2);
                    if !gr2.is_null() {
                        ext::grMaterial_801C9604(gr2, (*yp).xC, 0);
                    }
                }
            }
        }
        5 => {
            track_plat = 0;
            let j19 = ext::Ground_801C3FA4(gobj, 1);
            if ext::HSD_JObjGetFlags(j19) & 0x10 != 0 {
                ext::HSD_JObjClearFlagsAll(j19, JOBJ_HIDDEN as u32);
                ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(gobj, 1), JOBJ_HIDDEN as u32);
                let gr2 = ext::Ground_GetMapGObj(2);
                if !gr2.is_null() {
                    ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(gr2, 0), JOBJ_HIDDEN as u32);
                }
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                let pid = (*gp).u.shrineroute.xC8 as i32;
                show_other(gobj, pid, 0xBD, 0x13, 8);
                show_other(gobj, pid, 0xBE, 0x14, 9);
                show_other(gobj, pid, 0xBF, 0x12, 0xA);
                show_other(gobj, pid, 0xC0, 0x11, 0xB);
                show_other(gobj, pid, 0xC1, 0x10, 0xC);
                show_other(gobj, pid, 0xC2, 0xF, 0xD);
                ext::mpLib_80058560();
            }
            if ext::grLib_801C96E8(gobj) != 0 {
                (*gp).u.shrineroute.xC4 = 0;
            }
        }
        6 => {
            ext::stage_info.flags |= 0x10;
        }
        _ => {}
    }

    if track_plat == 0 {
        if !player.is_null() {
            if ext::ftLib_80086EC0(player) != 0 {
                ext::Stage_UnkSetVec3TCam_Offset(&mut sp5C);
                (*gp).u.shrineroute.xCE = 0x3C;
            } else if ext::ftLib_8008732C(player) != 0 {
                ext::Stage_UnkSetVec3TCam_Offset(&mut sp5C);
                (*gp).u.shrineroute.xCE = 0;
            } else {
                sp5C = sp7C;
            }
        } else {
            ext::Stage_UnkSetVec3TCam_Offset(&mut sp5C);
        }
        ext::Camera_80030AE0(1);
    } else {
        ext::Ground_801C2D24((*gp).u.shrineroute.xC8 as i32, &mut sp5C);
        sp5C.y += 30.0;
        ext::Camera_80030AE0(0);
    }

    {
        let timer = (*gp).u.shrineroute.xCE;
        if timer == 0 {
            ext::Ground_801C38BC(sp5C.x, sp5C.y);
        } else {
            let t = 1.0 / timer as f32;
            ext::Stage_UnkSetVec3TCam_Offset(&mut sp50);
            ext::lbVector_Sub(&mut sp5C, &mut sp50);
            sp5C.x *= t;
            sp5C.y *= t;
            sp5C.z *= t;
            ext::lbVector_Add(&mut sp50, &mut sp5C);
            ext::Ground_801C38BC(sp50.x, sp50.y);
            (*gp).u.shrineroute.xCE = (*gp).u.shrineroute.xCE.wrapping_sub(1);
        }
    }

    ext::Stage_UnkSetVec3TCam_Offset(&mut sp44);
    for k in 0..3usize {
        let jobj = (*gp).u.shrineroute.platforms[k].jobj;
        if !jobj.is_null() {
            let tx = (*gp).u.shrineroute.platforms[k].offset.x + sp44.x;
            mi::HSD_JObjSetTranslateX(&mut *jobj, tx, &mut Dirty);
            let jobj = (*gp).u.shrineroute.platforms[k].jobj;
            let ty = (*gp).u.shrineroute.platforms[k].offset.y + sp44.y;
            mi::HSD_JObjSetTranslateY(&mut *jobj, ty, &mut Dirty);
        }
    }

    (*gp).u.shrineroute.xCA = 0;
    (*gp).u.shrineroute.xCC = 0;
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grShrineRoute_80209AEC(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_80209AEC");}

pub unsafe extern "C" fn grShrineRoute_80209AF0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_80209AF0");
    let jobj = GET_JOBJ(gobj);
    let gp = GET_GROUND(gobj);

    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(2);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
    (*gp).u.shrineroute3.xC4 = ext::Ground_801C3FA4(gobj, 10);
    grShrineRoute_8020A8A4(gobj);
    (*gp).u.shrineroute3.xCC = 0.0;
    (*gp).u.shrineroute3.xC8 = 0.0;
    // 0.00006981317F * HSD_Randf() + 0.000017453292F: one fused multiply-add
    (*gp).u.shrineroute3.xD0 = fmadds(0.00006981317, ext::HSD_Randf(), 0.000017453292);
    (*gp).u.shrineroute3.xD0 *= if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
    (*gp).u.shrineroute3.xD4 = fmadds(0.00006981317, ext::HSD_Randf(), 0.000017453292);
    (*gp).u.shrineroute3.xD4 *= if ext::HSD_Randi(2) != 0 { 1.0 } else { -1.0 };
}

#[inline(always)]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub unsafe extern "C" fn grShrineRoute_80209BEC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_80209BEC");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    (*gp).u.shrineroute3.xC8 += (*gp).u.shrineroute3.xD0;
    (*gp).u.shrineroute3.xC8 *= 0.99;
    if (*gp).u.shrineroute3.xC8 > 0.0008726646 {
        (*gp).u.shrineroute3.xC8 = 0.0008726646;
    } else if (*gp).u.shrineroute3.xC8 < -0.0008726646 {
        (*gp).u.shrineroute3.xC8 = -0.0008726646;
    }

    (*gp).u.shrineroute3.xCC += (*gp).u.shrineroute3.xD4;
    (*gp).u.shrineroute3.xCC *= 0.99;
    if (*gp).u.shrineroute3.xCC > 0.0008726646 {
        (*gp).u.shrineroute3.xCC = 0.0008726646;
    } else if (*gp).u.shrineroute3.xCC < -0.0008726646 {
        (*gp).u.shrineroute3.xCC = -0.0008726646;
    }

    let mut rot = mi::HSD_JObjGetRotationX(&*jobj) + (*gp).u.shrineroute3.xC8;
    if rot > 0.2617994 {
        rot = 0.2617994;
        (*gp).u.shrineroute3.xC8 = -abs_f((*gp).u.shrineroute3.xC8);
        let tmp = ext::HSD_Randf();
        // 0.00006981317f * -tmp - 0.000017453292f: one fused multiply-subtract
        (*gp).u.shrineroute3.xD0 = fmsubs(0.00006981317, -tmp, 0.000017453292);
    } else if rot < -0.2617994 {
        rot = -0.2617994;
        (*gp).u.shrineroute3.xC8 = abs_f((*gp).u.shrineroute3.xC8);
        (*gp).u.shrineroute3.xD0 = fmadds(0.00006981317, ext::HSD_Randf(), 0.000017453292);
    }
    mi::HSD_JObjSetRotationX(&mut *jobj, rot, &mut Dirty);

    {
        let mut rot = mi::HSD_JObjGetRotationY(&*jobj) + (*gp).u.shrineroute3.xCC;
        if rot > 0.17453292 {
            rot = 0.17453292;
            (*gp).u.shrineroute3.xCC = -abs_f((*gp).u.shrineroute3.xCC);
            let tmp = ext::HSD_Randf();
            (*gp).u.shrineroute3.xD4 = fmsubs(0.00006981317, -tmp, 0.000017453292);
        } else if rot < -0.17453292 {
            rot = -0.17453292;
            (*gp).u.shrineroute3.xCC = abs_f((*gp).u.shrineroute3.xCC);
            (*gp).u.shrineroute3.xD4 = fmadds(0.00006981317, ext::HSD_Randf(), 0.000017453292);
        }
        mi::HSD_JObjSetRotationY(&mut *jobj, rot, &mut Dirty);
    }

    if !(*gp).u.shrineroute3.xC4.is_null() {
        let c4 = (*gp).u.shrineroute3.xC4;
        let mut rot = (*gp).u.shrineroute3.xD8 + mi::HSD_JObjGetRotationX(&*c4);
        if rot as f64 > 6.283185307179586 {
            rot = (rot as f64 - 6.283185307179586) as f32;
        } else if (rot as f64) < -6.283185307179586 {
            rot = (rot as f64 + 6.283185307179586) as f32;
        }
        mi::HSD_JObjSetRotationX(&mut *c4, rot, &mut Dirty);

        rot = (*gp).u.shrineroute3.xDC + mi::HSD_JObjGetRotationY(&*c4);
        if rot as f64 > 6.283185307179586 {
            rot = (rot as f64 - 6.283185307179586) as f32;
        } else if (rot as f64) < -6.283185307179586 {
            rot = (rot as f64 + 6.283185307179586) as f32;
        }
        mi::HSD_JObjSetRotationY(&mut *c4, rot, &mut Dirty);
    }

    if ext::lb_8000B134(jobj) != 0 {
        grShrineRoute_8020A8A4(gobj);
    }
}

pub unsafe extern "C" fn grShrineRoute_8020A100(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_8020A100");}

pub unsafe extern "C" fn grShrineRoute_8020A104(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_8020A104");
    let gp = GET_GROUND(gobj);
    let mut lgobj = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_LIGHT as usize);

    while !lgobj.is_null() {
        if (*lgobj).classifier != 0xC {
            lgobj = (*lgobj).next;
        } else {
            break;
        }
    }
    if lgobj.is_null() {
        panic!("assert: grshrineroute.c:1106 lgobj");
    }
    (*gp).u.shrineroute2.xC4 = lgobj;
    grShrineRoute_8020AA40((*gp).u.shrineroute2.xC4);

    let mut lobj = (*(*gp).u.shrineroute2.xC4).hsd_obj as *mut HSD_LObj;
    (*gp).u.shrineroute2.x168 = 0;
    while !lobj.is_null() {
        let n = (*gp).u.shrineroute2.x168 as usize;
        (*gp).u.shrineroute2.xC8[n] = lobj;
        (*gp).u.shrineroute2.x118[n] = ext::HSD_LObjGetFlags(lobj);

        lobj = (*lobj).next;
        (*gp).u.shrineroute2.x168 += 1;
    }

    (*gp).u.shrineroute2.x16C = grShrineRoute_8020AB58((*gp).u.shrineroute2.xC4);
    (*gp).u.shrineroute2.x170 = grShrineRoute_8020AC44((*gp).u.shrineroute2.xC4);
    grShrineRoute_8020A21C(gobj);
}

pub unsafe extern "C" fn grShrineRoute_8020A21C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_8020A21C");
    let mut player_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut light_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sorted = [0i32; 20];
    let mut distances = [0.0f32; 20];
    let mut pos1 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut pos2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dir: Vec3;
    let mut color = GXColor { r: 0, g: 0, b: 0, a: 0 };
    let mut temp_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let gp = GET_GROUND(gobj);
    let player = ext::Ground_GetP1Fighter();
    if !player.is_null() {
        ext::ftLib_80086644(player, &mut player_pos);
    } else {
        player_pos.z = 0.0;
        player_pos.y = 0.0;
        player_pos.x = 0.0;
    }

    if !(*gp).u.shrineroute2.xC4.is_null() {
        let mut i: i32 = 0;
        while i < (*gp).u.shrineroute2.x168 as i32 {
            let lobj = (*gp).u.shrineroute2.xC8[i as usize];
            let ty = ((*lobj).flags & 3) as i32;

            if ty == 0 {
                distances[i as usize] = -100.0;
            } else if ty == 1 || ty == 2 {
                ext::HSD_LObjGetPosition(lobj, &mut pos1);
                let dx = player_pos.x - pos1.x;
                let dy = player_pos.y - pos1.y;
                {
                    let dx2 = dx * dx;
                    let dy2 = dy * dy;
                    distances[i as usize] = sqrtf(dx2 + dy2);
                }
            } else if ty == 3 {
                ext::HSD_LObjGetPosition(lobj, &mut pos2);
                distances[i as usize] = abs_f(player_pos.x - pos2.x);
            } else {
                panic!("assert: grshrineroute.c:0x4A1 0");
            }

            if distances[i as usize] < 10.0 {
                distances[i as usize] = 10.0;
            }
            sorted[i as usize] = i;
            i += 1;
        }

        // Selection sort by distance
        i = 0;
        while i < (*gp).u.shrineroute2.x168 as i32 {
            let count = (*gp).u.shrineroute2.x168 as i32;
            let mut min_dist = 3.4028235e38f32;
            let mut best = 0i32;
            let mut j = i;
            while j < count {
                if min_dist > distances[sorted[j as usize] as usize] {
                    best = j;
                    min_dist = distances[sorted[j as usize] as usize];
                }
                j += 1;
            }
            let tmp = sorted[i as usize];
            sorted[i as usize] = sorted[best as usize];
            sorted[best as usize] = tmp;
            i += 1;
        }

        // Enable first 6 lights, disable rest
        i = 0;
        while i < (*gp).u.shrineroute2.x168 as i32 {
            if i < 6 {
                ext::HSD_LObjClearFlags((*gp).u.shrineroute2.xC8[sorted[i as usize] as usize], 0x20);
            } else {
                ext::HSD_LObjSetFlags((*gp).u.shrineroute2.xC8[sorted[i as usize] as usize], 0x20);
            }
            i += 1;
        }

        // Compute weighted direction
        dir = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let ref_dist = distances[sorted[1] as usize];

        i = 1;
        while i < (*gp).u.shrineroute2.x168 as i32 {
            if ext::HSD_LObjGetPosition((*gp).u.shrineroute2.xC8[sorted[i as usize] as usize], &mut light_pos) != 0 {
                let mut weight = ref_dist / distances[sorted[i as usize] as usize];
                ext::lbVector_Sub(&mut light_pos, &mut player_pos);
                if light_pos.y < 15.0 {
                    if light_pos.y < 0.0 {
                        weight = 0.0;
                    } else {
                        weight *= light_pos.y / 15.0;
                    }
                }
                ext::lbVector_Normalize(&mut light_pos);
                light_pos.x *= weight;
                light_pos.y *= weight;
                light_pos.z *= weight;
                ext::lbVector_Add(&mut dir, &mut light_pos);
            }
            i += 1;
        }

        // Compute magnitude and normalize direction
        // dir's address leaves the function (lbVector_Add/Normalize): squares not fused on the console
        let dist_sq = sqrtf(dir.z * dir.z + (dir.x * dir.x + dir.y * dir.y));
        if dist_sq < 0.01 {
            dir.y = 100.0;
        }
        ext::lbVector_Normalize(&mut dir);
        dir.x *= 20.0;
        dir.y *= 20.0;
        dir.z *= 20.0;
        ext::lbVector_Add(&mut dir, &mut player_pos);

        // Set spotlight
        ext::HSD_LObjSetPosition((*gp).u.shrineroute2.x16C, &mut dir);
        ext::HSD_LObjSetInterest((*gp).u.shrineroute2.x16C, &mut player_pos);
        ext::HSD_LObjSetFlags((*gp).u.shrineroute2.x16C, 0x400);

        // Color from 2nd nearest
        {
            let dist_diff = distances[sorted[2] as usize] - distances[sorted[1] as usize];
            ext::HSD_LObjGetColor((*gp).u.shrineroute2.xC8[sorted[1] as usize], &mut color);
            if distances[sorted[2] as usize] - distances[sorted[1] as usize] < 50.0 {
                let ratio = dist_diff / 50.0;
                color.r = melee_core::ppc::fctiwz_f32(color.r as f32 * ratio) as u8;
                color.g = melee_core::ppc::fctiwz_f32(color.g as f32 * ratio) as u8;
                color.b = melee_core::ppc::fctiwz_f32(color.b as f32 * ratio) as u8;
            }
        }
        ext::HSD_LObjSetColor((*gp).u.shrineroute2.x170, color);

        // Copy position from 2nd nearest
        if ext::HSD_LObjGetPosition((*gp).u.shrineroute2.xC8[sorted[1] as usize], &mut temp_pos) == 0 {
            temp_pos.z = 0.0;
            temp_pos.y = 0.0;
            temp_pos.x = 0.0;
        }
        ext::HSD_LObjSetPosition((*gp).u.shrineroute2.x170, &mut temp_pos);

        // Copy interest from 2nd nearest
        if ext::HSD_LObjGetInterest((*gp).u.shrineroute2.xC8[sorted[1] as usize], &mut temp_pos) == 0 {
            temp_pos.z = 0.0;
            temp_pos.y = 0.0;
            temp_pos.x = 0.0;
        }
        ext::HSD_LObjSetInterest((*gp).u.shrineroute2.x170, &mut temp_pos);

        // Copy and modify flags
        (*(*gp).u.shrineroute2.x170).flags = (*(*gp).u.shrineroute2.xC8[sorted[1] as usize]).flags;
        (*(*gp).u.shrineroute2.x170).flags &= !4;
        (*(*gp).u.shrineroute2.x170).flags |= 8;
    }
}

pub unsafe extern "C" fn grShrineRoute_8020A864(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_8020A864");}

pub unsafe extern "C" fn stageGObj1_3_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj1_3_OnInit");
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grShrineRoute_8020A89C(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_8020A89C");}

pub unsafe extern "C" fn grShrineRoute_8020A8A0(_arg: *mut HSD_GObj) { melee_core::ct!("grShrineRoute_8020A8A0");}

pub unsafe extern "C" fn grShrineRoute_8020A8A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_8020A8A4");
    let gp = (*gobj).user_data as *mut Ground;

    if !(*gp).u.shrineroute3.xC4.is_null() {
        let c4 = (*gp).u.shrineroute3.xC4;
        let angle = (6.283185307179586 * ext::HSD_Randf() as f64) as f32;
        mi::HSD_JObjSetTranslateX(&mut *c4, 300.0 * cosf(angle), &mut Dirty);
        mi::HSD_JObjSetTranslateY(&mut *c4, 300.0 * sinf(angle), &mut Dirty);
        // 0.04363323f * ((2.0f * rand) - 1.0f): the inner multiply-subtract is fused
        (*gp).u.shrineroute3.xD8 = 0.04363323 * fmsubs(2.0, ext::HSD_Randf(), 1.0);
        (*gp).u.shrineroute3.xDC = 0.04363323 * fmsubs(2.0, ext::HSD_Randf(), 1.0);
    }
}

const fn lc(
    color: [u8; 4],
    pos: [f32; 3],
    interest: [f32; 3],
    ty: i32,
    cutoff: f32,
    point_func: i32,
    ref_br: f32,
    ref_dist: f32,
) -> grSh_Route_LightConfig {
    grSh_Route_LightConfig {
        color: GXColor { r: color[0], g: color[1], b: color[2], a: color[3] },
        pos: Vec3 { x: pos[0], y: pos[1], z: pos[2] },
        interest: Vec3 { x: interest[0], y: interest[1], z: interest[2] },
        r#type: ty,
        cutoff,
        point_func,
        ref_br,
        ref_dist,
        dist_func: GX_DA_STEEP as i32,
    }
}
const Z3: [f32; 3] = [0.0, 0.0, 0.0];
const B1: i32 = LOBJ_FLAGS_B1;
const B1I: i32 = LOBJ_FLAGS_B1 | LOBJ_INFINITE;
const OFF: i32 = GX_SP_OFF as i32;
const COS2: i32 = GX_SP_COS2 as i32;

/// `static struct grSh_Route_LightConfig light_config[]`.
pub static light_config: [grSh_Route_LightConfig; 15] = [
    lc([0x80, 0x66, 0x4D, 0xFF], [790.0, -320.0, 70.0], Z3, B1, 0.0, OFF, 0.01, 800.0),
    lc([0x80, 0x80, 0x80, 0xFF], [510.0, -10.0, 70.0], Z3, B1, 0.0, OFF, 0.01, 500.0),
    lc([0xB3, 0x99, 0x66, 0xFF], [840.0, 210.0, 40.0], Z3, B1, 0.0, OFF, 0.1, 260.0),
    lc([0xFF, 0xCC, 0x66, 0xFF], [600.0, 120.0, 30.0], Z3, B1, 0.0, OFF, 0.01, 500.0),
    lc([0x80, 0x80, 0x80, 0xFF], [800.0, -130.0, 28.0], Z3, B1, 0.0, OFF, 0.01, 400.0),
    lc([0xCC, 0x99, 0xA5, 0xFF], [710.0, -50.0, 20.0], Z3, B1, 0.0, OFF, 0.01, 500.0),
    lc([0x90, 0xC2, 0xFF, 0xFF], [10.0, 18.0, 9.0], Z3, B1, 0.0, OFF, 0.03, 810.0),
    lc([0xFF, 0x82, 0x82, 0xFF], [-260.0, -220.0, 20.0], Z3, B1, 0.0, OFF, 0.03, 300.0),
    lc([0x78, 0xB4, 0xFF, 0xFF], [244.0, -288.0, 22.0], Z3, B1, 0.0, OFF, 0.03, 300.0),
    lc([0xF0, 0xF0, 0xF0, 0xFF], [-66.0, 140.0, 27.0], Z3, B1, 0.0, OFF, 0.03, 400.0),
    lc([0x80, 0x99, 0x66, 0xFF], [500.0, -150.0, 60.0], [500.0, -330.0, -20.0], B1I, 30.0, COS2, 0.99, 200.0),
    lc([0xC8, 0xF0, 0xFF, 0xFF], [560.0, 310.0, 160.0], [560.0, 290.0, -10.0], B1I, 45.0, COS2, 0.99, 24.0),
    lc([0xA4, 0xD8, 0xFF, 0xFF], [260.0, 444.0, 88.0], [242.0, -166.0, -33.0], B1I, 25.0, COS2, 0.99, 450.0),
    lc([0xFF, 0x4D, 0x4D, 0xFF], [-162.0, -300.0, 23.0], [-314.0, 111.0, -5.0], B1I, 31.0, COS2, 0.99, 370.0),
    lc([0x64, 0x80, 0x4D, 0xFF], [-88.0, -131.0, 66.0], [-91.0, -227.0, -25.0], B1I, 45.0, COS2, 0.99, 125.0),
];

pub unsafe extern "C" fn grShrineRoute_8020AA40(gobj: *mut HSD_GObj) {
    melee_core::ct!("grShrineRoute_8020AA40");
    if !gobj.is_null() {
        let mut lobj = (*((*gobj).hsd_obj as *mut HSD_LObj)).next;
        for i in 0..light_config.len() {
            let config = &light_config[i];
            if !lobj.is_null() {
                ext::HSD_LObjSetColor(lobj, config.color);
                ext::HSD_LObjSetPosition(lobj, &raw const config.pos as *mut Vec3);
                ext::HSD_LObjSetDistAttn(lobj, config.ref_dist, config.ref_br, config.dist_func);
                if config.r#type == LOBJ_FLAGS_B1 {
                    (*lobj).flags = (LOBJ_FLAGS_B1 | LOBJ_DIFFUSE) as u16;
                } else if config.r#type == (LOBJ_INFINITE | LOBJ_FLAGS_B1) {
                    (*lobj).flags = (LOBJ_INFINITE | LOBJ_FLAGS_B1 | LOBJ_DIFFUSE) as u16;
                    ext::HSD_LObjSetInterest(lobj, &raw const config.interest as *mut Vec3);
                    ext::HSD_LObjSetSpot(lobj, config.cutoff, config.point_func);
                }
            } else {
                ext::OSReport(c"%s:%d: oioi...\n".as_ptr(), &[]);
            }
            lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
        }
    }
}

pub static mut lobj0_shininess: f32 = 16.0;

pub static mut lobj0: HSD_LightDesc = HSD_LightDesc {
    class_name: core::ptr::null_mut(),
    next: core::ptr::null_mut(),
    flags: (LOBJ_FLAGS_B1 | LOBJ_DIFFUSE) as u16,
    attnflags: 0,
    color: GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF },
    position: core::ptr::null_mut(),
    interest: core::ptr::null_mut(),
    u: HSD_LightDesc_u { p: unsafe { &raw mut lobj0_shininess as *mut c_void } },
};

/// `HSD_LObjSetNext` (lobj.h inline): asserts the lobj is non-NULL.
unsafe fn hsd_lobj_set_next(lobj: *mut HSD_LObj, next: *mut HSD_LObj) {
    if lobj.is_null() {
        panic!("assert: lobj.h:0x136 lobj");
    }
    (*lobj).next = next;
}

pub unsafe extern "C" fn grShrineRoute_8020AB58(gobj: *mut HSD_GObj) -> *mut HSD_LObj {
    melee_core::ct!("grShrineRoute_8020AB58");
    if gobj.is_null() {
        return core::ptr::null_mut();
    }
    let mut lobj = (*gobj).hsd_obj as *mut HSD_LObj;
    if lobj.is_null() {
        return core::ptr::null_mut();
    }
    while !(if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next }).is_null() {
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
    }
    let new_lobj = ext::HSD_LObjLoadDesc(&raw mut lobj0);
    if !new_lobj.is_null() {
        let mut pos = Vec3 { x: 0.0, y: 100.0, z: 0.0 };
        hsd_lobj_set_next(lobj, new_lobj);
        ext::HSD_LObjSetPosition(new_lobj, &mut pos);
        ext::HSD_LObjSetFlags(new_lobj, 0x20);
    }
    new_lobj
}

pub static mut lobj1_shininess: f32 = 16.0;

pub static mut lobj1: HSD_LightDesc = HSD_LightDesc {
    class_name: core::ptr::null_mut(),
    next: core::ptr::null_mut(),
    flags: (LOBJ_FLAGS_B1 | LOBJ_SPECULAR) as u16,
    attnflags: 0,
    color: GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF },
    position: core::ptr::null_mut(),
    interest: core::ptr::null_mut(),
    u: HSD_LightDesc_u { p: unsafe { &raw mut lobj1_shininess as *mut c_void } },
};

pub unsafe extern "C" fn grShrineRoute_8020AC44(gobj: *mut HSD_GObj) -> *mut HSD_LObj {
    melee_core::ct!("grShrineRoute_8020AC44");
    if gobj.is_null() {
        return core::ptr::null_mut();
    }
    let mut lobj = (*gobj).hsd_obj as *mut HSD_LObj;
    if lobj.is_null() {
        return core::ptr::null_mut();
    }
    while !(if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next }).is_null() {
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
    }
    let new_lobj = ext::HSD_LObjLoadDesc(&raw mut lobj1);
    if !new_lobj.is_null() {
        let mut pos = Vec3 { x: 0.0, y: 100.0, z: 0.0 };
        hsd_lobj_set_next(lobj, new_lobj);
        ext::HSD_LObjSetPosition(new_lobj, &mut pos);
    }
    new_lobj
}

pub unsafe extern "C" fn grShrineRoute_8020AD24(arg: i32) -> i32 {
    melee_core::ct!("grShrineRoute_8020AD24");
    if arg == 1 {
        ext::Ground_801C5740(arg);
        return 1;
    }
    0
}

/// The joint contact callback of the six route spots (`mpLib_JointCollisionCallback`).
pub unsafe extern "C" fn onJointCollision(
    user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    melee_core::ct!("onJointCollision");
    let gp = user_data as *mut Ground;

    if (*coll).x34_flags.b1234() as i32 == 1 && ground_kind == 2 {
        let slot = ext::ftLib_80086BE0((*coll).x0_gobj) as i32;
        if (*coll).x0_gobj == ext::Player_GetEntity(slot) {
            (*gp).u.shrineroute.xCA = 1;
            let pgobj = ext::Player_GetEntityAtIndex(slot, 1);
            if !pgobj.is_null() && ext::ftLib_8008731C(pgobj) == 0 {
                return;
            }
            (*gp).u.shrineroute.xCC = 1;
            return;
        }
        (*gp).u.shrineroute.xCC = 1;
    }
}

pub unsafe extern "C" fn grShrineRoute_8020AE08(gobj: *mut HSD_GObj, player_gobj: *mut HSD_GObj, out: *mut i32) -> i32 {
    melee_core::ct!("grShrineRoute_8020AE08");
    let gp = (*gobj).user_data as *mut Ground;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if (*gp).u.shrineroute.xC4 == 1 || (*gp).u.shrineroute.xC4 == 3 {
        return 0;
    }

    let mut lo = Vec3 { x: -285.93f64 as f32, y: -226.1, z: 0.0 };
    let mut hi = Vec3 { x: -161.56f64 as f32, y: -226.1, z: 0.0 };
    let scale = ext::Ground_801C0498();
    lo.x *= scale;
    lo.y *= scale;
    lo.z *= scale;
    hi.x *= scale;
    hi.y *= scale;
    hi.z *= scale;

    ext::ftLib_80086644(player_gobj, &mut pos);

    if pos.y < lo.y && lo.x < pos.x && pos.x < hi.x {
        *out = (*yakumono_param).x10;
        return 1;
    }
    0
}

pub unsafe extern "C" fn grShrineRoute_8020AF38(gobj: *mut HSD_GObj, arg1: i32) {
    melee_core::ct!("grShrineRoute_8020AF38");
    let ix = (arg1 - 0xBD) as usize;
    let gp = GET_GROUND(gobj);

    let pgobj = ext::Ground_GetP1Fighter();

    if !(*gp).u.map.symbol[ix].is_null() {
        let symbolp = &raw mut (*gp).u.map.symbol[ix];
        let map_id = (*((**symbolp).user_data as *mut Ground)).map_id as i32;
        let mut scale: f32 = 0.7;
        if map_id == 1 {
            scale *= (*yakumono_param).x14;
        } else {
            scale *= (*yakumono_param).x18;
        }
        let jobj = ext::Ground_801C2CF4(arg1);
        ext::efSync_Spawn(0x428, gobj, jobj, &mut scale);
        ext::Ground_801C4A08((*gp).u.map.symbol[ix]);
        *symbolp = core::ptr::null_mut();
        if !pgobj.is_null() {
            ext::ftLib_80086C18(pgobj, 0xB, 0x1E);
        }
    }
}

pub unsafe extern "C" fn grShrineRoute_8020B020(gobj: *mut HSD_GObj, r4: i32, r5: i32) {
    melee_core::ct!("grShrineRoute_8020B020");
    let gp = (*gobj).user_data as *mut Ground;
    let comp = r4 - 189;
    for i in 0..6usize {
        if r4 == -1 || i as i32 != comp {
            let s = (*gp).u.map.symbol[i];
            if !s.is_null() && !GET_JOBJ(s).is_null() {
                if r5 != 0 {
                    ext::HSD_JObjSetFlagsAll((*s).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
                } else {
                    ext::HSD_JObjClearFlagsAll((*s).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
                }
            }
        }
    }
}

pub unsafe extern "C" fn grShrineRoute_8020B0AC() {
    melee_core::ct!("grShrineRoute_8020B0AC");
    for i in 0..7usize {
        ext::mpLib_800575B0(grSh_Route_803E58E0[i] as i32);
    }
}

pub unsafe extern "C" fn grShrineRoute_OnCheckShadowRender(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grShrineRoute_OnCheckShadowRender");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, core::ptr::null_mut(), &mut vec);
    if (*a).y > vec.y { 1 } else { 0 }
}

/// TARGET_PC: the read-only view of the Underground Maze's route state for `gd.stage_objectives()`.
pub unsafe extern "C" fn grShrineRoute_ScriptQuery(field: i32, idx: i32) -> i32 {
    melee_core::ct!("grShrineRoute_ScriptQuery");
    let gobj = ext::Ground_GetMapGObj(4);
    if gobj.is_null() {
        return -1;
    }
    let gp = (*gobj).user_data as *mut Ground;
    if gp.is_null() {
        return -1;
    }
    match field {
        0 => (*gp).u.shrineroute.xC4 as i32,
        1 => (*gp).u.shrineroute.xC6 as i32,
        2 => (*gp).u.shrineroute.xC8 as i32,
        3 => {
            if !(0..=5).contains(&idx)
                || (*gp).u.map.symbol[idx as usize].is_null()
                || (*(*gp).u.map.symbol[idx as usize]).user_data.is_null()
            {
                return -1;
            }
            (*((*(*gp).u.map.symbol[idx as usize]).user_data as *mut Ground)).map_id as i32
        }
        _ => -1,
    }
}

// ---- folded from grshrineroute.rs (consolidation steps 21/22) ----
/// stageGObj1_3_OnInit_fp (gr/grshrineroute.c:1115)
    pub unsafe extern "C" fn stageGObj1_3_OnInit_fp(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grshrineroute.rs (consolidation steps 21/22) ----
/// grShrineRoute_802088B8 (gr/grshrineroute.c:214)
    pub extern "C" fn grShrineRoute_802088B8() -> i32 { 0 }

/// grShrineRoute_802089D8 (gr/grshrineroute.c:252)
    pub extern "C" fn grShrineRoute_802089D8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_80208A28 (gr/grshrineroute.c:269)
    pub extern "C" fn grShrineRoute_80208A28(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_80208F0C (gr/grshrineroute.c:381)
    pub extern "C" fn grShrineRoute_80208F0C(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_80209BE4 (gr/grshrineroute.c:798)
    pub extern "C" fn grShrineRoute_80209BE4(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_8020A214 (gr/grshrineroute.c:925)
    pub extern "C" fn grShrineRoute_8020A214(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_8020A894 (gr/grshrineroute.c:1120)
    pub extern "C" fn grShrineRoute_8020A894(a0: *mut HSD_GObj) -> i32 { 0 }

/// grShrineRoute_OnTouchLine (gr/grshrineroute.c:1564)
    pub extern "C" fn grShrineRoute_OnTouchLine(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }
