//! src/melee/gr/grrcruise.c: the Rainbow Cruise stage (the rotating platforms, the vanishing
//! blocks, the scrolling ship, the cruise camera light).
//!
//! The constant/empty callbacks and the single-call forwarder already ported in `gr::trivial` /
//! `gr::forward` are re-exported; the rest is ported here. Callees outside this TU are `tu_ext`
//! hooks (unset: panic `TODO ... not ported`). State-oracle checked by tests/tu_grrcruise.rs
//! against gen/turef/tu_grrcruise_ref.c.
//!
//! GameCube union overlays that native layouts break (the GroundVars members the C reaches through
//! two names): `u.map.chikuwa` and `u.rcruise.entries` are the same word at +0x6C, and
//! `u.rcruise2.xEC` is `u.scroll.anim_gobj` (+0x28). Where the C writes one and reads the other,
//! this port writes both (see `grRCruise_801FF5B4`, `grRCruise_801FF168`). The entry flag byte
//! `pad_01` is the MSB-first bitfield `b0` = 0x80. The vanish table is allocated for the native
//! `Map_VanishEntry` width (the C sizes it by the 8-byte descriptor table).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_runtime::gekko::{cosf, sinf, sqrtf};
use melee_runtime::lbtrigf::atan2f;
use melee_types::inline as mi;
use melee_types::*;

pub use crate::gr::forward::grrcruise::grRCruise_801FF29C;
pub use crate::gr::trivial::grrcruise::{
    grRCruise_8020014C, grRCruise_801FF2C0, grRCruise_801FF3E0, grRCruise_801FF434,
    grRCruise_801FF6CC, grRCruise_801FF794, grRCruise_801FF8DC, grRCruise_801FFAD4,
    grRCruise_80201C50, grRCruise_80201C58,
};

/// `struct grRCruise_YakumonoParam`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grRCruise_YakumonoParam {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
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
    pub x34: f32,
    pub x38: f32,
    pub x3C: i32,
    pub x40: i32,
    pub x44: i32,
}
melee_core::gc_layout!(grRCruise_YakumonoParam, "grRCruise_YakumonoParam", size 0x48, align 4, [
    x0 @ 0x0: 4 "f32",
    x4 @ 0x4: 4 "f32",
    x8 @ 0x8: 4 "f32",
    xC @ 0xC: 4 "s32",
    x10 @ 0x10: 4 "s32",
    x14 @ 0x14: 4 "s32",
    x18 @ 0x18: 4 "s32",
    x1C @ 0x1C: 4 "s32",
    x20 @ 0x20: 4 "s32",
    x24 @ 0x24: 4 "s32",
    x28 @ 0x28: 4 "s32",
    x2C @ 0x2C: 4 "f32",
    x30 @ 0x30: 4 "f32",
    x34 @ 0x34: 4 "f32",
    x38 @ 0x38: 4 "f32",
    x3C @ 0x3C: 4 "s32",
    x40 @ 0x40: 4 "s32",
    x44 @ 0x44: 4 "s32",
]);

/// `static struct grRCruise_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grRCruise_YakumonoParam = core::ptr::null_mut();

const fn j(x: i16, y: i16, z: i16) -> GrJoint {
    GrJoint { x, y, z }
}
pub static mut grRc_803E4DA8: [GrJoint; 23] = [
    j(0, 1, 1), j(1, 1, 1), j(2, 1, 1), j(3, 1, 1), j(4, 1, 1),
    j(11, 1, 7), j(10, 1, 17), j(6, 1, 7), j(7, 1, 7), j(8, 1, 7),
    j(9, 1, 7), j(12, 1, 19), j(13, 1, 18), j(14, 1, 18), j(15, 1, 18),
    j(16, 1, 18), j(17, 1, 18), j(18, 1, 18), j(19, 1, 18), j(20, 1, 18),
    j(21, 1, 18), j(22, 1, 18), j(23, 1, 18),
];

/// `static s16 grRc_804D4790[]`: the three pop-up block joints.
pub static grRc_804D4790: [i16; 3] = [0x01, 0x1A, 0x33];

/// `s16 grRc_803E4FF0[]`: the 17 rotating-block joints.
pub static grRc_803E4FF0: [i16; 17] = [
    0x20, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
];

const fn v(x0: i16, x2: i16, x4: i32) -> Map_VanishDesc {
    Map_VanishDesc { x0, x2, x4 }
}
/// `struct Map_VanishDesc lbl_803E5014[]`.
pub static lbl_803E5014: [Map_VanishDesc; 20] = [
    v(6, 4, 0), v(8, 5, 0), v(16, 10, 0), v(11, 6, 0), v(12, 7, 0),
    v(13, 8, 0), v(14, 9, 0), v(15, 8, 0), v(19, 12, 1), v(20, 13, 1),
    v(21, 14, 1), v(30, 15, 1), v(22, 16, 1), v(23, 17, 1), v(24, 18, 1),
    v(25, 19, 1), v(26, 20, 1), v(27, 21, 1), v(28, 22, 1), v(29, 23, 1),
];

pub unsafe extern "C" fn grRCruise_801FF164(_arg: i32) {}

pub unsafe extern "C" fn grRCruise_801FF168() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grRCruise_YakumonoParam;
    ext::stage_info.unk8C.set_b4(1);
    ext::stage_info.unk8C.set_b5(0);
    let jgobj = grRCruise_801FF2C8(3);
    let gp = GET_GROUND(jgobj);
    (*gp).u.rcruise.xC4.set_b0(1);
    let jobj = GET_JOBJ(jgobj);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    let grgobj = grRCruise_801FF2C8(3);
    let gp2 = GET_GROUND(grgobj);
    (*gp2).u.rcruise2.xEC = jgobj;
    // the same word on the console (u.rcruise2.xEC is u.scroll.anim_gobj)
    (*gp2).u.scroll.anim_gobj = jgobj;
    grRCruise_801FF2C8(0);
    grRCruise_801FF2C8(2);
    grRCruise_801FF2C8(5);
    let gobj1 = grRCruise_801FF2C8(1);
    let gobj6 = grRCruise_801FF2C8(6);
    let gobj4 = grRCruise_801FF2C8(4);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    grRCruise_801FFADC(grgobj);
    ext::Ground_UpdateMapColl(gobj1);
    ext::Ground_801C32AC(1);
    ext::Ground_UpdateMapColl(gobj6);
    ext::Ground_801C32AC(6);
    ext::Ground_UpdateMapColl(gobj4);
    ext::Ground_801C32AC(4);
    ext::mpLib_80057BC0(11);
}

pub unsafe extern "C" fn grRCruise_801FF298() {}

pub unsafe extern "C" fn grRCruise_801FF2C8(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = (&raw mut grRc_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grrcruise.c".as_ptr(), 290, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn grRCruise_801FF3B4(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn grRCruise_801FF3E8(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF3EC(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF3F0(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grRCruise_801FF43C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF440(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF444(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let g = gobj as *mut c_void;
    ext::mpJointSetCb1(5, g, Some(grRCruise_80200578));
    for joint in [27, 36, 37, 38, 39, 40, 41, 42, 43, 28, 29, 30, 31, 32, 33, 34, 35] {
        ext::mpJointSetCb1(joint, g, Some(grRCruise_802010A4));
    }
    (*gp).u.rcruise.x10 = 0;
}

pub unsafe extern "C" fn grRCruise_801FF5B4(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.rcruise.x10 = 1;
    (*gp).u.map.chikuwa = ext::HSD_MemAlloc((17 * core::mem::size_of::<grRCruise_Entry>()) as i32) as *mut Map_Chikuwa;
    if (*gp).u.map.chikuwa.is_null() {
        panic!("assert: grrcruise.c:410 gp->u.map.chikuwa");
    }
    // the same word on the console (u.map.chikuwa is u.rcruise.entries)
    (*gp).u.rcruise.entries = (*gp).u.map.chikuwa as *mut grRCruise_Entry;
    grRCruise_80201410(gobj);
    ext::Ground_801C10B8(gobj, Some(grRCruise_801FF444));
    grRCruise_80200540(gobj);
    grRCruise_80200B48(gobj);
    ext::Ground_UpdateMapColl(gobj);
    ext::mpLib_80058560();
    {
        let mut pos = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
        (*gp).u.rcruise.x4 = ext::lb_80011A50(
            &mut pos,
            -1,
            0.4,
            0.0,
            60.0 * 0.01745329252f32,
            -10000.0,
            10000.0,
            10000.0,
            -10000.0,
        );
        (*gp).u.rcruise.x8 = 1.0 * 0.01745329252f32;
        (*gp).u.rcruise.xC = 1;
    }
}

pub unsafe extern "C" fn grRCruise_801FF6D4(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    if (*gp).u.rcruise.x10 == 0 {
        grRCruise_80201588(gobj);
        grRCruise_8020071C(gobj);
        grRCruise_80200C04(gobj);
    }
    ext::lb_800115F4();
    grRCruise_80201110(gobj);
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grRCruise_801FF738(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF73C(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(gobj, 31), JOBJ_HIDDEN as u32);
}

pub unsafe extern "C" fn grRCruise_801FF79C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF7A0(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF7A4(gobj: *mut HSD_GObj) {
    let stage_gobj = gobj;
    let gp = GET_GROUND(stage_gobj);
    let mut jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(stage_gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C752C(jobj, 1, 30628, ext::HSD_AObjSetFlags as *mut c_void, 3, AOBJ_LOOP as u32);
    let archive = ext::grDatFiles_GetArchive();
    let mut data: *mut DynamicsDesc = core::ptr::null_mut();
    let mut have = false;
    if !archive.is_null() {
        data = ext::HSD_ArchiveGetPublicAddress((*archive).unk0, c"dynamicsdata_shipflag".as_ptr())
            as *mut DynamicsDesc;
        have = !data.is_null();
    }
    if have {
        ext::grLib_801C9B20(ext::Ground_801C3FA4(stage_gobj, 23), data, &mut (*gp).u.rcruise2.xC4);
    } else {
        (*gp).u.rcruise2.xC4.data = core::ptr::null_mut();
    }

    jobj = ext::Ground_801C3FA4(stage_gobj, 10);
    if !jobj.is_null() {
        ext::Ground_801C2D0C(0, jobj);
    }
    jobj = ext::Ground_801C3FA4(stage_gobj, 11);
    if !jobj.is_null() {
        ext::Ground_801C2D0C(1, jobj);
    }
    jobj = ext::Ground_801C3FA4(stage_gobj, 12);
    if !jobj.is_null() {
        ext::Ground_801C2D0C(2, jobj);
    }
    jobj = ext::Ground_801C3FA4(stage_gobj, 13);
    if !jobj.is_null() {
        ext::Ground_801C2D0C(3, jobj);
    }
}

pub unsafe extern "C" fn grRCruise_801FF8E4(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grLib_801C9B8C((&raw mut (*gp).u) as *mut c_void);
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grRCruise_801FF920(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_801FF924(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    (*gp).u.scroll.x38[0] = ext::Ground_801C3FA4(gobj, 4);
    (*gp).u.scroll.x38[1] = ext::Ground_801C3FA4(gobj, 5);
    (*gp).u.scroll.x38[2] = ext::Ground_801C3FA4(gobj, 6);
    (*gp).u.scroll.x44 = ext::Ground_801C3FA4(gobj, 7);
    (*gp).u.scroll.int_jobj = ext::Ground_801C3FA4(gobj, 3);
    if (*gp).u.scroll.int_jobj.is_null() {
        panic!("assert: grrcruise.c:0x2B0 gp->u.scroll.int_jobj");
    }
    (*gp).u.scroll.cam_jobj = ext::Ground_801C3FA4(gobj, 2);
    if (*gp).u.scroll.cam_jobj.is_null() {
        panic!("assert: grrcruise.c:0x2B2 gp->u.scroll.cam_jobj");
    }
    (*gp).u.scroll.ctr_jobj = ext::Ground_801C3FA4(gobj, 1);
    if (*gp).u.scroll.ctr_jobj.is_null() {
        panic!("assert: grrcruise.c:0x2B4 gp->u.scroll.ctr_jobj");
    }

    mi::HSD_JObjGetTranslation(&*(*gp).u.scroll.ctr_jobj, &mut (*gp).u.scroll.x4);
    (*gp).u.scroll.x10.z = 0.0;
    (*gp).u.scroll.x10.y = 0.0;
    (*gp).u.scroll.x10.x = 0.0;
    (*gp).u.scroll.x1C.z = 0.0;
    (*gp).u.scroll.x1C.y = 0.0;
    (*gp).u.scroll.x1C.x = 0.0;
    (*gp).u.scroll.set_x0(0);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C752C(jobj, 1, 30628, ext::HSD_AObjSetFlags as *mut c_void, 3, 0x20000000);
    (*gobj).render_cb = Some(fn_80201BE0);
}

/// The static inline `grRCruise_801FFADC_inline`: where the ship's interest point lands on the
/// camera plane.
unsafe fn grRCruise_801FFADC_inline(gobj: *mut HSD_GObj, sp64: *mut Vec3) {
    let mut sp18 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp24 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut diff = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_offset2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let anim_gp = GET_GROUND(gobj);
    let int_jobj = (*anim_gp).u.scroll.int_jobj;
    let cam_jobj = (*anim_gp).u.scroll.cam_jobj;
    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset2);
    ext::lb_8000B1CC(cam_jobj, core::ptr::null_mut(), &mut sp18);
    ext::lb_8000B1CC(int_jobj, core::ptr::null_mut(), &mut sp24);
    ext::lbVector_Diff(&mut sp24, &mut sp18, &mut diff);
    // -(a * b - c): the multiply-subtract is fused
    (*sp64).x = -fmadds(diff.x, sp18.z / diff.z, -sp18.x);
    (*sp64).y = -fmadds(diff.y, sp18.z / diff.z, -sp18.y);
    (*sp64).z = 0.0;
    ext::lbVector_Sub(sp64, &mut cam_offset2);
    (*sp64).y += 10.0;
}

/// Dirty-flag helper of the JObj inline setters (the macro form calls `HSD_JObjSetMtxDirtySub`).
unsafe fn set_translate(jobj: *mut HSD_JObj, t: &Vec3) {
    mi::HSD_JObjSetTranslate(&mut *jobj, t, &mut Dirty);
}

pub unsafe extern "C" fn grRCruise_801FFADC(arg0: *mut HSD_GObj) {
    let mut sp64 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    let gp = GET_GROUND(arg0);
    if (*gp).u.scroll.x0() == 0 {
        ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
        if (*gp).u.scroll.anim_gobj.is_null() {
            panic!("assert: grrcruise.c:0x2E0 gp->u.scroll.anim_gobj");
        }
        grRCruise_801FFADC_inline((*gp).u.scroll.anim_gobj, &mut sp64);
        ext::lbVector_Diff(&mut (*gp).u.scroll.x10, &mut sp64, &mut (*gp).u.scroll.x1C);
        (*gp).u.scroll.x10 = sp64;
        sp64.x *= -1.0;
        sp64.y *= -1.0;
        sp64.z *= -1.0;
        ext::lbVector_Add(&mut sp64, &mut cam_offset);
        let temp = (*arg0).hsd_obj as *mut HSD_JObj;
        if !temp.is_null() {
            set_translate(temp, &sp64);
            mi::HSD_JObjAddTranslationZ(&mut *temp, -350.0 * ext::Ground_801C0498(), &mut Dirty);
        }
        for id in [2, 1, 5, 6, 4] {
            let gobj = ext::Ground_GetMapGObj(id);
            if !gobj.is_null() {
                let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
                if !jobj.is_null() {
                    set_translate(jobj, &sp64);
                }
            }
        }
    }
}

pub unsafe extern "C" fn grRCruise_80200070(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grRCruise_80200074(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    for i in 0..3 {
        ext::grAnime_801C8098(gobj, grRc_804D4790[i] as i32, 7, 1, 0.0, 0.0);
        (*gp).u.rcruise.x3C[i].x2 = ext::Ground_801C32D4((*gp).map_id as i32, grRc_804D4790[i] as i32) as i16;
        ext::mpJointSetCb1((*gp).u.rcruise.x3C[i].x2 as i32, gobj as *mut c_void, Some(fn_80200460));
        (*gp).u.rcruise.x3C[i].xC = ext::Ground_801C3FA4(gobj, grRc_804D4790[i] as i32);
        (*gp).u.rcruise.x3C[i].x8 = 0;
        (*gp).u.rcruise.x3C[i].x4 = 0;
        (*gp).u.rcruise.x3C[i].x0 = 0;
    }
}

pub unsafe extern "C" fn grRCruise_80200154(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let gp2 = gp;
    let yp = yakumono_param;

    for i in 0..3usize {
        let e = &mut (*gp).u.rcruise.x3C[i];
        match e.x0 {
            0 => {
                ext::mpJointListAdd((*gp2).u.rcruise.x3C[i].x2 as i32);
                grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 1);
                (*gp).u.rcruise.x3C[i].x4 = 0;
                (*gp).u.rcruise.x3C[i].x0 = 1;
            }
            2 => {
                if (*gp).u.rcruise.x3C[i].x8 == 0 {
                    (*gp).u.rcruise.x3C[i].x4 = 0;
                    ext::grAnime_801C7A94(gobj, grRc_804D4790[i] as i32, 1, 0.0);
                    (*gp).u.rcruise.x3C[i].x0 = 3;
                } else if ext::grAnime_801C83D0(gobj, grRc_804D4790[i] as i32, 7) != 0 {
                    (*gp).u.rcruise.x3C[i].x4 = 0;
                    (*gp).u.rcruise.x3C[i].x0 = 5;
                }
            }
            3 => {
                if (*gp).u.rcruise.x3C[i].x4 >= (*yp).xC {
                    (*gp).u.rcruise.x3C[i].x4 = 0;
                    (*gp).u.rcruise.x3C[i].set_x1_b0(0);
                    (*gp).u.rcruise.x3C[i].x0 = 4;
                }
                (*gp).u.rcruise.x3C[i].x4 += 1;
            }
            4 => {
                if (*gp).u.rcruise.x3C[i].x4 % (*yp).x14 == 0 {
                    let b = (*gp).u.rcruise.x3C[i].x1_b0() ^ 1;
                    (*gp).u.rcruise.x3C[i].set_x1_b0(b);
                    if (*gp).u.rcruise.x3C[i].x1_b0() != 0 {
                        grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 0);
                    } else {
                        grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 1);
                    }
                }
                if (*gp).u.rcruise.x3C[i].x4 >= (*yp).x10 {
                    ext::mpLib_80057BC0((*gp2).u.rcruise.x3C[i].x2 as i32);
                    grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 0);
                    ext::grAnime_801C7BA0(gobj, grRc_804D4790[i] as i32, 1, 0.0);
                    ext::grAnime_801C7A94(gobj, grRc_804D4790[i] as i32, 1, 0.0);
                    ext::mpLib_80055E9C((*gp2).u.rcruise.x3C[i].x2 as i32);
                    ext::mpLib_80057424((*gp2).u.rcruise.x3C[i].x2 as i32);
                    (*gp).u.rcruise.x3C[i].x0 = 0;
                }
                (*gp).u.rcruise.x3C[i].x4 += 1;
            }
            5 => {
                if (*gp).u.rcruise.x3C[i].x4 % (*yp).x1C == 0 {
                    grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 0);
                } else {
                    grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 1);
                }
                if (*gp).u.rcruise.x3C[i].x4 >= (*yp).x18 {
                    ext::mpLib_80057BC0((*gp2).u.rcruise.x3C[i].x2 as i32);
                    grRCruise_80201B60((*(*gp2).u.rcruise.x3C[i].xC).child, 0);
                    ext::grAnime_801C7BA0(gobj, grRc_804D4790[i] as i32, 1, 0.0);
                    ext::grAnime_801C7A94(gobj, grRc_804D4790[i] as i32, 1, 0.0);
                    ext::mpLib_80055E9C((*gp2).u.rcruise.x3C[i].x2 as i32);
                    ext::mpLib_80057424((*gp2).u.rcruise.x3C[i].x2 as i32);
                    (*gp).u.rcruise.x3C[i].x0 = 0;
                }
                (*gp).u.rcruise.x3C[i].x4 += 1;
            }
            _ => {}
        }
        (*gp).u.rcruise.x3C[i].x8 = 0;
    }
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grRCruise_8020045C(_arg: *mut HSD_GObj) {}

/// The joint contact callback of the three pop-up blocks (`mpLib_JointCollisionCallback`).
pub unsafe extern "C" fn fn_80200460(
    user_data: *mut c_void,
    joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    _ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let gobj = user_data as *mut HSD_GObj;
    let gp1 = (*gobj).user_data as *mut Ground;

    if (*coll).x34_flags.b1234() as i32 == 1 {
        for i in 0..3usize {
            if (*gp1).u.rcruise.x3C[i].x2 as i32 == joint_id {
                let state = (*gp1).u.rcruise.x3C[i].x0;
                if state == 1 || state == 3 || state == 4 {
                    (*gp1).u.rcruise.x3C[i].x4 = 0;
                    grRCruise_80201B60((*(*gp1).u.rcruise.x3C[i].xC).child, 1);
                    ext::grAnime_801C7A94(gobj, grRc_804D4790[i] as i32, 1, 1.0);
                    (*gp1).u.rcruise.x3C[i].x0 = 2;
                }
                (*gp1).u.rcruise.x3C[i].x8 += 1;
                break;
            }
        }
    }
}

pub unsafe extern "C" fn grRCruise_80200540(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.rcruise.x18 = 0.0;
    (*gp).u.rcruise.x14 = 0.0;
    (*gp).u.rcruise.x1C = 0.0;
    (*gp).u.rcruise.x20 = 0.0;
    (*gp).u.rcruise.x34 = 0;
    (*gp).u.rcruise.x30 = 0;
    (*gp).u.rcruise.x28 = 0.0;
    (*gp).u.rcruise.x24 = 0.0;
    (*gp).u.rcruise.x38 = 0;
    (*gp).u.rcruise.x2C = 0;
}

/// The joint contact callback of the rotating hub (`mpLib_JointCollisionCallback`).
pub unsafe extern "C" fn grRCruise_80200578(
    user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    mut coll_x50: i32,
    _ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gobj = user_data as *mut HSD_GObj;
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = ext::Ground_801C3FA4(gobj, 8);

    let b = (*coll).x34_flags.b1234() as i32;
    if b != 1 && b != 2 && b != 3 {
        return;
    }
    if coll_x50 as f32 > 1000.0 {
        coll_x50 = 1000;
    }
    ext::lb_8000B1CC(jobj, core::ptr::null_mut(), &mut pos);
    let dx = pos.x - (*coll).cur_pos.x;
    let dy = pos.y - (*coll).cur_pos.y;
    let dist = sqrtf(fmadds(dy, dy, dx * dx));
    if dist > 4.0 {
        if pos.x < (*coll).cur_pos.x {
            (*gp).u.rcruise.x24 = fmadds(dist, coll_x50 as f32 / 1000.0, (*gp).u.rcruise.x24);
        } else {
            (*gp).u.rcruise.x28 = fmadds(dist, coll_x50 as f32 / 1000.0, (*gp).u.rcruise.x28);
        }
    }
    (*gp).u.rcruise.x34 += 1;
}

pub unsafe extern "C" fn grRCruise_8020071C(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let yp = yakumono_param;
    let jobj = ext::Ground_801C3FA4(gobj, 8);
    let m5 = ext::Ground_GetMapGObj(5);
    let jobj5 = ext::Ground_801C3FA4(m5, 8);
    let x18 = (*gp).u.rcruise.x18;
    let abs_rot = if x18 < 0.0 { -x18 } else { x18 };
    // abs_rot - 360.0f * (s32) (abs_rot / 360.0f): the multiply-subtract is fused
    let wrapped = fmadds(-360.0, (abs_rot / 360.0) as i32 as f32, abs_rot);

    match (*gp).u.rcruise.x2C {
        0 => {
            if (*gp).u.rcruise.x34 == 0 {
                (*gp).u.rcruise.x1C = if (*gp).u.rcruise.x18 < 0.0 { 1.0 } else { -1.0 };
                (*gp).u.rcruise.x20 = (*yp).x8 * (*gp).u.rcruise.x1C;
                if wrapped <= 0.2 {
                    (*gp).u.rcruise.x20 = 0.0;
                    (*gp).u.rcruise.x2C = 0;
                }
            } else if (*gp).u.rcruise.x24 < (*gp).u.rcruise.x28 {
                (*gp).u.rcruise.x1C = 1.0;
                (*gp).u.rcruise.x20 = (*yp).x0 * ((*gp).u.rcruise.x28 - (*gp).u.rcruise.x24);
                if (*gp).u.rcruise.x20 >= (*yp).x4 {
                    (*gp).u.rcruise.x20 = (*yp).x4;
                }
            } else {
                (*gp).u.rcruise.x1C = -1.0;
                (*gp).u.rcruise.x20 = -(*yp).x0 * ((*gp).u.rcruise.x24 - (*gp).u.rcruise.x28);
                if (*gp).u.rcruise.x20 <= -(*yp).x4 {
                    (*gp).u.rcruise.x20 = -(*yp).x4;
                }
            }
        }
        1 => {
            if (*gp).u.rcruise.x38 == 0 {
                if (*gp).u.rcruise.x34 == 0 {
                    (*gp).u.rcruise.x1C = if (*gp).u.rcruise.x18 < 0.0 { 1.0 } else { -1.0 };
                    (*gp).u.rcruise.x20 = (*yp).x8 * (*gp).u.rcruise.x1C;
                    if wrapped <= 0.2 {
                        (*gp).u.rcruise.x20 = 0.0;
                        (*gp).u.rcruise.x2C = 0;
                    }
                } else {
                    (*gp).u.rcruise.x2C = 0;
                }
            } else {
                (*gp).u.rcruise.x38 -= 1;
                (*gp).u.rcruise.x20 = fmadds(0.008, -(*gp).u.rcruise.x1C, (*gp).u.rcruise.x20);
                if (if (*gp).u.rcruise.x20 < 0.0 { -(*gp).u.rcruise.x20 } else { (*gp).u.rcruise.x20 }) <= 0.008 {
                    (*gp).u.rcruise.x20 = 0.0;
                }
            }
        }
        _ => {}
    }
    (*gp).u.rcruise.x18 += (*gp).u.rcruise.x20;
    mi::HSD_JObjSetRotationZ(&mut *jobj, 0.017453292 * (*gp).u.rcruise.x18, &mut Dirty);
    mi::HSD_JObjSetRotationZ(&mut *jobj5, 0.017453292 * (*gp).u.rcruise.x18, &mut Dirty);
    (*gp).u.rcruise.x30 = (*gp).u.rcruise.x34;
    (*gp).u.rcruise.x34 = 0;
    (*gp).u.rcruise.x28 = 0.0;
    (*gp).u.rcruise.x24 = 0.0;
}

pub unsafe extern "C" fn grRCruise_80200B48(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    for i in 0..grRc_803E4FF0.len() {
        let entry = (*gp).u.rcruise.entries.add(i);
        (*entry).x2 = ext::Ground_801C32D4((*gp).map_id as i32, grRc_803E4FF0[i] as i32) as i16;
        (*entry).x14 = ext::Ground_801C3FA4(gobj, grRc_803E4FF0[i] as i32);
        (*entry).x8 = 0;
        (*entry).x4 = 0;
        let jobj = (*entry).x14;
        (*entry).xC = mi::HSD_JObjGetTranslationY(&*jobj);
        (*entry).x0 = 0;
    }
}

pub unsafe extern "C" fn grRCruise_80200C04(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    for i in 0..grRc_803E4FF0.len() {
        let entry = (*gp).u.rcruise.entries.add(i);
        match (*entry).x0 {
            1 => {
                if (*entry).x8 == 0 {
                    (*entry).x4 = 0;
                    (*entry).x0 = 0;
                } else if (*entry).x4 >= (*yp).x20 {
                    (*entry).x0 = 2;
                } else {
                    (*entry).x4 += 1;
                }
            }
            2 => {
                if (*entry).x4 % (*yp).x28 == 0 {
                    let range = 100.0 * ((*yp).x30 - (*yp).x2C);
                    let randi = if range as i32 != 0 { ext::HSD_Randi(range as i32) } else { 0 };
                    mi::HSD_JObjSetTranslateY(
                        &mut *(*entry).x14,
                        (*entry).xC + ((*yp).x2C + (randi as f32 / 100.0)),
                        &mut Dirty,
                    );
                }
                if (*entry).x4 >= (*yp).x24 {
                    (*entry).x10 = 0.0;
                    (*entry).x0 = 3;
                }
                (*entry).x4 += 1;
            }
            3 => {
                (*entry).x10 -= (*yp).x34;
                if (if (*entry).x10 < 0.0 { -(*entry).x10 } else { (*entry).x10 }) > (*yp).x38 {
                    (*entry).x10 = -(*yp).x38;
                }
                mi::HSD_JObjAddTranslationY(&mut *(*entry).x14, (*entry).x10, &mut Dirty);
                if mi::HSD_JObjGetTranslationY(&*(*entry).x14) <= ext::Stage_GetCamBoundsBottomOffset() {
                    (*entry).x10 = 0.0;
                    grRCruise_80201B60((*entry).x14, 0);
                    ext::mpLib_80057BC0((*entry).x2 as i32);
                    (*entry).x4 = 0;
                    (*entry).x0 = 4;
                }
            }
            4 => {
                if (*entry).x4 >= (*yp).x3C {
                    mi::HSD_JObjSetTranslateY(&mut *(*entry).x14, (*entry).xC, &mut Dirty);
                    ext::mpLib_80055E9C((*entry).x2 as i32);
                    ext::mpLib_80057424((*entry).x2 as i32);
                    grRCruise_80201B60((*entry).x14, 1);
                    ext::mpJointListAdd((*entry).x2 as i32);
                    (*entry).x4 = 0;
                    // `((struct grRCruise_EntryFlags*) &entry->pad_01)->b0 = 0` (MSB-first b0 = 0x80)
                    (*entry).pad_01 &= !0x80;
                    (*entry).x0 = 5;
                }
                (*entry).x4 += 1;
            }
            5 => {
                if (*entry).x4 % (*yp).x44 == 0 {
                    (*entry).pad_01 = ((*entry).pad_01 & !0x80) | ((((*entry).pad_01 >> 7) ^ 1) << 7);
                    if (*entry).pad_01 & 0x80 != 0 {
                        grRCruise_80201B60((*entry).x14, 0);
                    } else {
                        grRCruise_80201B60((*entry).x14, 1);
                    }
                }
                if (*entry).x4 >= (*yp).x40 {
                    grRCruise_80201B60((*entry).x14, 1);
                    (*entry).x4 = 0;
                    (*entry).x0 = 0;
                }
                (*entry).x4 += 1;
            }
            _ => {}
        }
        (*entry).x8 = 0;
    }
}

/// The joint contact callback of the 17 rotating blocks (`mpLib_JointCollisionCallback`).
pub unsafe extern "C" fn grRCruise_802010A4(
    user_data: *mut c_void,
    joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    _ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let gp = (*(user_data as *mut HSD_GObj)).user_data as *mut Ground;

    if (*coll).x34_flags.b1234() as i32 != 1 {
        return;
    }
    for i in 0..17usize {
        let entry = (*gp).u.rcruise.entries.add(i);
        if (*entry).x2 as i32 == joint_id {
            if (*entry).x0 == 0 {
                (*entry).x4 = 0;
                (*entry).x0 = 1;
            }
            (*entry).x8 += 1;
            break;
        }
    }
}

pub unsafe extern "C" fn grRCruise_80201110(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let lb = (*gp).u.rcruise.x4;
    let mut angle = atan2f((*lb).x4.z, (*lb).x4.x);
    angle += (*gp).u.rcruise.x8;

    if angle as f64 > 1.0471975430846214 {
        angle = 1.0471976;
    }
    if (angle as f64) < -1.0471975430846214 {
        angle = -1.0471976;
    }
    (*lb).x4.x = cosf(angle);
    (*lb).x4.z = sinf(angle);
    if (*lb).unk_angle_int % 30 == 0 && ext::HSD_Randf() as f64 > 0.5 {
        (*gp).u.rcruise.x8 = -(*gp).u.rcruise.x8;
    }
    if (*lb).unk_angle_int % 300 == 0 {
        if (*gp).u.rcruise.xC != 0 {
            (*gp).u.rcruise.xC = 0;
        } else if ext::HSD_Randf() as f64 > 0.5 {
            (*gp).u.rcruise.xC = 1;
        }
    }
    if (*gp).u.rcruise.xC != 0 {
        // 0.3 * HSD_Randf() + 0.2 in double: one fused multiply-add
        (*lb).unk_scale = 0.3f64.mul_add(ext::HSD_Randf() as f64, 0.2) as f32;
    } else {
        (*lb).unk_scale = 0.2f64.mul_add(ext::HSD_Randf() as f64, 0.1) as f32;
    }
}

/// `grRCruise_80201288_inline`: the jobj's first DObj.
#[inline]
unsafe fn dobj_of(jobj: *mut HSD_JObj) -> *mut HSD_DObj {
    ext::HSD_JObjGetDObj(jobj)
}

/// Apply `callback(dobj, flags)` to the DObj of `jobj`, its child and its sibling, one level of
/// recursion each (the exact traversal the C unrolls).
pub unsafe extern "C" fn grRCruise_80201288(
    jobj: *mut HSD_JObj,
    callback: unsafe extern "C" fn(*mut HSD_DObj, u32),
    flags: u32,
) {
    if !jobj.is_null() {
        let temp_r3 = dobj_of(jobj);
        if !temp_r3.is_null() {
            callback(temp_r3, flags);
        }
        let var_r29 = if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child };
        let tmp = var_r29;
        if !tmp.is_null() {
            let temp_r3_2 = dobj_of(tmp);
            if !temp_r3_2.is_null() {
                callback(temp_r3_2, flags);
            }
            let var_r3 = if var_r29.is_null() { core::ptr::null_mut() } else { (*tmp).child };
            if !var_r3.is_null() {
                grRCruise_80201288(var_r3, callback, flags);
            }
            let var_r3_2 = if tmp.is_null() { core::ptr::null_mut() } else { (*tmp).next };
            if !var_r3_2.is_null() {
                grRCruise_80201288(var_r3_2, callback, flags);
            }
        }
        let var_r29_2 = if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).next };
        if !var_r29_2.is_null() {
            let temp_r3_3 = dobj_of(var_r29_2);
            if !temp_r3_3.is_null() {
                callback(temp_r3_3, flags);
            }
            let var_r3_3 = if var_r29_2.is_null() { core::ptr::null_mut() } else { (*var_r29_2).child };
            if !var_r3_3.is_null() {
                grRCruise_80201288(var_r3_3, callback, flags);
            }
            let var_r3_4 = if var_r29_2.is_null() { core::ptr::null_mut() } else { (*var_r29_2).next };
            if !var_r3_4.is_null() {
                grRCruise_80201288(var_r3_4, callback, flags);
            }
        }
    }
}

unsafe fn set_hidden_a(i: usize) {
    let joint = lbl_803E5014[i].x0 as i32;
    let gobj5 = ext::Ground_GetMapGObj(5);
    if !gobj5.is_null() {
        let gobj1 = ext::Ground_GetMapGObj(1);
        if !gobj1.is_null() {
            let mut jobj = ext::Ground_801C3FA4(gobj5, joint);
            if !jobj.is_null() {
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
            jobj = ext::Ground_801C3FA4(gobj1, joint);
            if !jobj.is_null() {
                grRCruise_80201288(jobj, ext::HSD_DObjSetFlags, DOBJ_HIDDEN as u32);
            }
        }
    }
}

unsafe fn set_hidden_b(i: usize) {
    let joint = lbl_803E5014[i].x0 as i32;
    let gobj5 = ext::Ground_GetMapGObj(5);
    if !gobj5.is_null() {
        let gobj1 = ext::Ground_GetMapGObj(1);
        if !gobj1.is_null() {
            let mut jobj = ext::Ground_801C3FA4(gobj5, joint);
            if !jobj.is_null() {
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
            jobj = ext::Ground_801C3FA4(gobj1, joint);
            if !jobj.is_null() {
                grRCruise_80201288(jobj, ext::HSD_DObjClearFlags, DOBJ_HIDDEN as u32);
            }
        }
    }
}

pub unsafe extern "C" fn grRCruise_80201410(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);

    // native `Map_VanishEntry` is wider than the 8-byte descriptor the C sizes the table by
    (*gp).u.map.vanish = ext::HSD_MemAlloc((lbl_803E5014.len() * core::mem::size_of::<Map_VanishEntry>()) as i32)
        as *mut Map_VanishEntry;
    if (*gp).u.map.vanish.is_null() {
        panic!("assert: grrcruise.c:1453 gp->u.map.vanish");
    }
    for i in 0..lbl_803E5014.len() {
        let ve = (*gp).u.map.vanish.add(i);
        (*ve).jobj = ext::Ground_801C3FA4(gobj, lbl_803E5014[i].x0 as i32);
        if (*ve).jobj.is_null() {
            panic!("assert: grrcruise.c:1459 gp->u.map.vanish[i].jobj");
        }
        if lbl_803E5014[i].x4 != 0 {
            (*ve).x0 = 2;
            set_hidden_a(i);
            ext::mpJointListAdd(lbl_803E5014[i].x2 as i32);
        } else {
            (*ve).x0 = 0;
            ext::grAnime_801C7FF8(gobj, lbl_803E5014[i].x0 as i32, 2, 1, 0.0, 1.0);
            ext::mpLib_80057BC0(lbl_803E5014[i].x2 as i32);
        }
    }
}

pub unsafe extern "C" fn grRCruise_80201588(gobj: *mut HSD_GObj) {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    if (*gp).u.map.vanish.is_null() {
        panic!("assert: grrcruise.c:0x5D6 gp->u.map.vanish");
    }
    for i in 0..lbl_803E5014.len() {
        if lbl_803E5014[i].x4 != 0 {
            continue;
        }
        let ve = (*gp).u.map.vanish.add(i);
        match (*ve).x0 {
            0 => {
                ext::lb_8000B1CC((*ve).jobj, core::ptr::null_mut(), &mut pos);
                if ext::Camera_8003118C(&mut pos, -20.0) != 0 {
                    (*ve).x0 = 1;
                    ext::grAnime_801C7FF8(gobj, lbl_803E5014[i].x0 as i32, 2, 2, 0.0, 1.0);
                    ext::mpJointListAdd(lbl_803E5014[i].x2 as i32);
                    ext::mpLib_80055E9C(lbl_803E5014[i].x2 as i32);
                    ext::mpLib_80057424(lbl_803E5014[i].x2 as i32);
                }
            }
            1 => {
                if ext::grAnime_801C83D0(gobj, lbl_803E5014[i].x0 as i32, 2) != 0 {
                    (*ve).x0 = 2;
                    set_hidden_a(i);
                }
            }
            2 => {
                ext::lb_8000B1CC((*ve).jobj, core::ptr::null_mut(), &mut pos);
                if ext::Camera_8003118C(&mut pos, -20.0) == 0 {
                    (*ve).x0 = 3;
                    ext::grAnime_801C7FF8(gobj, lbl_803E5014[i].x0 as i32, 2, 3, 0.0, 1.0);
                    set_hidden_b(i);
                }
            }
            3 => {
                if ext::grAnime_801C83D0(gobj, lbl_803E5014[i].x0 as i32, 2) != 0 {
                    (*ve).x0 = 0;
                    ext::grAnime_801C7FF8(gobj, lbl_803E5014[i].x0 as i32, 2, 1, 0.0, 1.0);
                    let mut jj = 0usize;
                    while jj < lbl_803E5014.len() {
                        if lbl_803E5014[jj].x2 == lbl_803E5014[i].x2 && (*(*gp).u.map.vanish.add(jj)).x0 != 0 {
                            break;
                        }
                        jj += 1;
                    }
                    if jj == lbl_803E5014.len() {
                        ext::mpLib_80057BC0(lbl_803E5014[i].x2 as i32);
                    }
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grRCruise_80201918(vec: *mut Vec3) {
    let gobj = ext::Ground_GetMapGObj(3);
    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        if !gp.is_null() {
            *vec = (*gp).u.scroll.x1C;
            return;
        }
    }
    (*vec).z = 0.0;
    (*vec).y = 0.0;
    (*vec).x = 0.0;
}

pub unsafe extern "C" fn grRCruise_80201988(line_id: i32) -> i32 {
    if ext::stage_info.grkind == Gr_Kind_RCruise && line_id != -1 {
        let joint = ext::mpJointFromLine(line_id);
        let result = joint == 0x1B
            || joint == 0x24
            || joint == 0x25
            || joint == 0x26
            || joint == 0x27
            || joint == 0x28
            || joint == 0x29
            || joint == 0x2A
            || joint == 0x2B
            || joint == 0x1C
            || joint == 0x1D
            || joint == 0x1E
            || joint == 0x1F
            || joint == 0x20
            || joint == 0x21
            || joint == 0x22
            || joint == 0x23;
        if result {
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn grRCruise_80201B60(jobj: *mut HSD_JObj, clear: i32) {
    let mut dobj = ext::HSD_JObjGetDObj(jobj);
    while !dobj.is_null() {
        if clear != 0 {
            ext::HSD_DObjClearFlags(dobj, DOBJ_HIDDEN as u32);
        } else {
            ext::HSD_DObjSetFlags(dobj, DOBJ_HIDDEN as u32);
        }
        dobj = if dobj.is_null() { core::ptr::null_mut() } else { (*dobj).next };
    }
}

pub unsafe extern "C" fn fn_80201BE0(gobj: *mut HSD_GObj, pass: i32) {
    if ext::gm_GetDbPauseFlag(1) != 0 || ext::gm_GetDbPauseFlag(2) != 0 || ext::Camera_8003010C() != 0 {
        return;
    }
    ext::grDisplay_801C5DB0(gobj, pass);
}

pub static mut grRc_StageCallbacks: [StageCallbacks; 7] = [
    StageCallbacks {
        on_init: Some(grRCruise_801FF3B4),
        callback1: Some(grRCruise_801FF3E0),
        gobj_proc: Some(grRCruise_801FF3E8),
        callback3: Some(grRCruise_801FF3EC),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_801FF5B4),
        callback1: Some(grRCruise_801FF6CC),
        gobj_proc: Some(grRCruise_801FF6D4),
        callback3: Some(grRCruise_801FF738),
        anon_10: StageCallbacks_anon_135 { flags: 0xC0000000 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_801FF3F0),
        callback1: Some(grRCruise_801FF434),
        gobj_proc: Some(grRCruise_801FF43C),
        callback3: Some(grRCruise_801FF440),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_801FF924),
        callback1: Some(grRCruise_801FFAD4),
        gobj_proc: Some(grRCruise_801FFADC),
        callback3: Some(grRCruise_80200070),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_80200074),
        callback1: Some(grRCruise_8020014C),
        gobj_proc: Some(grRCruise_80200154),
        callback3: Some(grRCruise_8020045C),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_801FF73C),
        callback1: Some(grRCruise_801FF794),
        gobj_proc: Some(grRCruise_801FF79C),
        callback3: Some(grRCruise_801FF7A0),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grRCruise_801FF7A4),
        callback1: Some(grRCruise_801FF8DC),
        gobj_proc: Some(grRCruise_801FF8E4),
        callback3: Some(grRCruise_801FF920),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
];

pub static mut grRc_StageData: StageData = StageData {
    grkind: Gr_Kind_RCruise,
    callbacks: unsafe { &raw mut grRc_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrRc.dat".as_ptr() as *mut i8,
    on_init: Some(grRCruise_801FF168),
    on_demo_init: Some(grRCruise_801FF164),
    on_load: Some(grRCruise_801FF298),
    on_start: Some(grRCruise_801FF29C),
    callback4: Some(grRCruise_801FF2C0),
    on_touch_line: Some(grRCruise_80201C50),
    on_check_shadow_render: Some(grRCruise_80201C58),
    flags2: 1 << 2,
    joints: unsafe { &raw mut grRc_803E4DA8 as *mut GrJoint },
    joint_count: 23,
};
