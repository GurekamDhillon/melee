//! src/melee/gr/grpushon.c: Fourside-style night city "Pushon" stage (Mushroom-less lights): the
//! stage-wide point lights the fighter moves under, the yakumono parameter block and the
//! collision callbacks.
//!
//! The constant/empty callbacks already ported in `gr::trivial` are re-exported; the rest is ported
//! here. Callees outside this TU are `tu_ext` hooks (unset: panic `TODO ... not ported`).
//! State-oracle checked by tests/tu_grpushon.rs against gen/turef/tu_grpushon_ref.c.
//!
//! One TARGET_PC divergence is kept from the C (it is the PC form): `grPushOn_802187A8` takes the
//! classifier-12 light object from GX link 4 instead of its head, and `grPushOn_80219230` maps the
//! character kind through `gm_MexVanillaKind` first. The retail head read
//! `((HSD_GObj*) HSD_GObjGXLinkHead)->next_gx` is `HSD_GObjGXLinkHead[4]` (next_gx is at +0x10 in the
//! 32-bit layout); the oracle compiles the C with that spelled out.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use core::ffi::c_void;
use melee_core::fp::{fmadds, fmsubs};
use melee_runtime::gekko::sqrtf;
use melee_types::*;

pub use crate::gr::trivial::grpushon::{
    grPushOn_802183DC, grPushOn_80218590, grPushOn_80218670, grPushOn_80218880, grPushOn_80219528,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grPushOn_Entry {
    pub x0: i32,
    pub x4: i16,
    pub x6: i16,
}
melee_core::gc_layout!(grPushOn_Entry, "grPushOn_Entry", size 0x8, align 4, [
    x0 @ 0x0: 4 "s32",
    x4 @ 0x4: 2 "s16",
    x6 @ 0x6: 2 "s16",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grPushOn_Lookup {
    pub key: i32,
    pub value: i32,
}
melee_core::gc_layout!(grPushOn_Lookup, "grPushOn_Lookup", size 0x8, align 4, [
    key @ 0x0: 4 "s32",
    value @ 0x4: 4 "s32",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grPushOn_LightConfig {
    pub color: GXColor,
    pub pos: Vec3,
    pub ref_br: f32,
    pub ref_dist: f32,
    pub dist_func: i32,
}
melee_core::gc_layout!(grPushOn_LightConfig, "grPushOn_LightConfig", size 0x1C, align 4, [
    color @ 0x0: 4 "GXColor",
    pos @ 0x4: 12 "Vec3",
    ref_br @ 0x10: 4 "f32",
    ref_dist @ 0x14: 4 "f32",
    dist_func @ 0x18: 4 "s32",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grPushon_YakumonoParam {
    pub x0: i32,
    pub x4: *mut DynamicsDesc,
    pub x8: *mut DynamicsDesc,
    pub xC: *mut DynamicsDesc,
    pub x10: *mut DynamicsDesc,
    pub x14: *mut DynamicsDesc,
    pub x18: i32,
    pub x1c: [grPushOn_Entry; 0x1E],
    pub x10c: [grPushOn_Lookup; 0x21],
}
melee_core::gc_layout!(grPushon_YakumonoParam, "grPushon_YakumonoParam", size 0x214, align 4, [
    x0 @ 0x0: 4 "s32",
    x4 @ 0x4: 4 "DynamicsDesc*",
    x8 @ 0x8: 4 "DynamicsDesc*",
    xC @ 0xC: 4 "DynamicsDesc*",
    x10 @ 0x10: 4 "DynamicsDesc*",
    x14 @ 0x14: 4 "DynamicsDesc*",
    x18 @ 0x18: 4 "bool",
    x1c @ 0x1C: 240 "grPushOn_Entry[30]",
    x10c @ 0x10C: 264 "grPushOn_Lookup[33]",
]);

/// `static struct grPushon_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grPushon_YakumonoParam = core::ptr::null_mut();

pub static grPushOn_803B8440: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static grPushOn_803B844C: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
pub static grPushOn_803B8458: Vec3 = Vec3 { x: 0.0, y: 100.0, z: 0.0 };
pub static grPushOn_803B8464: Vec3 = Vec3 { x: 0.0, y: 100.0, z: 0.0 };
pub static mut grPushOn_804D4934: f32 = 16.0;
pub static mut grPushOn_804D4948: f32 = 16.0;

const fn color(r: u8, g: u8, b: u8, a: u8) -> GXColor {
    GXColor { r, g, b, a }
}
const fn cfg(c: GXColor, x: f32, y: f32, z: f32, ref_br: f32, ref_dist: f32) -> grPushOn_LightConfig {
    grPushOn_LightConfig { color: c, pos: Vec3 { x, y, z }, ref_br, ref_dist, dist_func: 3 }
}
/// `static struct grPushOn_LightConfig light_configs[9]`.
pub static light_configs: [grPushOn_LightConfig; 9] = [
    cfg(color(0xBE, 0xE6, 0xE6, 0xFF), -12.0, 735.0, -20.0, 0.1, 800.0),
    cfg(color(0xF0, 0xF0, 0xF0, 0xFF), 250.0, 708.0, -30.0, 0.4, 500.0),
    cfg(color(0xC3, 0x56, 0xCD, 0xFF), 550.0, 655.0, -50.0, 0.2, 600.0),
    cfg(color(0xF0, 0xF0, 0xF0, 0xFF), 860.0, 550.0, -30.0, 0.1, 700.0),
    cfg(color(0x7C, 0xE7, 0xFF, 0xFF), 1150.0, 675.0, -60.0, 0.2, 800.0),
    cfg(color(0xFF, 0x4B, 0x4B, 0xFF), 1400.0, 370.0, -60.0, 0.01, 500.0),
    cfg(color(0xE6, 0xE6, 0xE6, 0xFF), 1580.0, 20.0, -30.0, 0.1, 500.0),
    cfg(color(0xFF, 0x6E, 0x6E, 0xFF), 2093.0, 120.0, -20.0, 0.3, 600.0),
    cfg(color(0xBE, 0xE6, 0xFF, 0xFF), 1700.0, 630.0, -20.0, 0.2, 800.0),
];

/// `f32 grPushOn_803E7CCC[13]`: x-min, x-max, y-top of the four "inside" rectangles.
pub static grPushOn_803E7CCC: [f32; 13] = [
    1441.428955078125,
    1645.62890625,
    340.57470703125,
    1306.4332275390625,
    1361.6331787109375,
    361.5741882324219,
    1193.4345703125,
    1268.634521484375,
    367.57391357421875,
    1121.16015625,
    1157.6820068359375,
    354.6068115234375,
    0.0,
];

pub static mut grPushOn_803E7B74: HSD_LightDesc = HSD_LightDesc {
    class_name: core::ptr::null_mut(),
    next: core::ptr::null_mut(),
    flags: 6,
    attnflags: 0,
    color: color(0xFF, 0xFF, 0xFF, 0xFF),
    position: core::ptr::null_mut(),
    interest: core::ptr::null_mut(),
    u: HSD_LightDesc_u { p: unsafe { &raw mut grPushOn_804D4934 as *mut c_void } },
};

pub static mut grPushOn_803E7B90: HSD_LightDesc = HSD_LightDesc {
    class_name: core::ptr::null_mut(),
    next: core::ptr::null_mut(),
    flags: 0xA,
    attnflags: 0,
    color: color(0xFF, 0xFF, 0xFF, 0xFF),
    position: core::ptr::null_mut(),
    interest: core::ptr::null_mut(),
    u: HSD_LightDesc_u { p: unsafe { &raw mut grPushOn_804D4948 as *mut c_void } },
};

pub unsafe extern "C" fn grPushOn_802182C4(_arg: i32) {}

pub unsafe extern "C" fn grPushOn_802182C8() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grPushon_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grPushOn_802183E4(0);
    grPushOn_802183E4(1);
    ext::mpLib_80057BC0(0);
}

pub unsafe extern "C" fn grPushOn_80218330() {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    grPushOn_802183E4(2);
    let gobj = ext::Ground_GetP1Fighter();
    if !gobj.is_null() {
        ext::ftLib_80086644(gobj, &mut vec);
        ext::Ground_801C38BC(vec.x, vec.y);
        ext::Camera_8002F3AC();
    }
}

pub unsafe extern "C" fn grPushOn_80218378() {
    ext::grZakoGenerator_801CAE04(core::ptr::null_mut());
    // `bool` is `int` in the game's headers
    let mut val = (*yakumono_param).x18;
    if val != 0 {
        val = ext::HSD_Randi((*yakumono_param).x18);
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

pub unsafe extern "C" fn grPushOn_802183E4(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = (&raw mut grPushOn_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grpushon.c".as_ptr(), 222, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn grPushOn_802184CC(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let mut cam_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Stage_UnkSetVec3TCam_Offset(&mut cam_offset);
    // x * (a + b) - c fuses to a single multiply-subtract (clang -ffp-contract=on)
    let a = ext::Stage_GetCamBoundsTopOffset();
    let b = ext::Stage_GetBlastZoneTopOffset();
    ext::Ground_801C3880(fmadds(0.5, a + b, -cam_offset.y));
    let a = ext::Stage_GetCamBoundsBottomOffset();
    let b = ext::Stage_GetBlastZoneBottomOffset();
    ext::Ground_801C3890(fmadds(0.5, a + b, -cam_offset.y));
    let a = ext::Stage_GetCamBoundsLeftOffset();
    let b = ext::Stage_GetBlastZoneLeftOffset();
    ext::Ground_801C38A0(fmadds(0.5, a + b, -cam_offset.x));
    let a = ext::Stage_GetCamBoundsRightOffset();
    let b = ext::Stage_GetBlastZoneRightOffset();
    ext::Ground_801C38AC(fmadds(0.5, a + b, -cam_offset.x));
}

pub unsafe extern "C" fn grPushOn_80218598(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grPushOn_8021859C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grPushOn_802185A0(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::ftCo_800C07F8(gobj, 3, fn_802192A4 as *mut c_void);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grAnime_801C752C(jobj, 1, 0x77A4, ext::HSD_AObjSetFlags as *mut c_void, 3, 0x20000000);
    (*gp).u.map.set_xC4_b0(0);
    ext::mpJointSetCb1(3, gp as *mut c_void, Some(fn_802190A0));
    ext::mpJointSetCb1(4, gp as *mut c_void, Some(fn_802190A0));
    ext::mpJointSetCb1(5, gp as *mut c_void, Some(fn_802190A0));
}

pub unsafe extern "C" fn fn_80218678() -> i32 {
    let gobj = ext::Ground_GetMapGObj(1);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        if !gp.is_null() && (*gp).u.map.xC4_b0() != 0 {
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn grPushOn_802186C8(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let mut vec = grPushOn_803B8440;
    ext::Ground_801C3D44(fn_80218678 as *mut c_void, 25.0, 20.0);
    {
        let gobj2 = ext::Ground_GetP1Fighter();
        if !gobj2.is_null() {
            ext::ftLib_80086644(gobj2, &mut vec);
            ext::Ground_801C0498();
            if vec.y < 0.0 {
                ext::Ground_801C0498();
                vec.y = 0.0;
            } else if vec.y > 800.0 * ext::Ground_801C0498() {
                vec.y = 800.0 * ext::Ground_801C0498();
            }
            ext::Ground_801C38BC(vec.x, vec.y);
        }
    }
    ext::lb_800115F4();
    (*gp).u.map.set_xC4_b0(0);
}

pub unsafe extern "C" fn grPushOn_802187A4(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grPushOn_802187A8(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    (*gp).u.pushon.gobj = *ext::HSD_GObjGXLinkHead.add(4) as *mut c_void;
    // TARGET_PC: take the fighters' light object (classifier 12) from the link instead of its head
    {
        let mut h = *ext::HSD_GObjGXLinkHead.add(4);
        while !h.is_null() {
            if (*h).classifier == 12 && (*h).obj_kind as i32 == ext::HSD_GObj_LightKind as i32 && !(*h).hsd_obj.is_null() {
                (*gp).u.pushon.gobj = h as *mut c_void;
                break;
            }
            h = (*h).next_gx;
        }
    }
    grPushOn_802190D0((*gp).u.pushon.gobj as *mut HSD_GObj);
    let mut lobj = (*((*gp).u.pushon.gobj as *mut HSD_GObj)).hsd_obj as *mut HSD_LObj;
    (*gp).u.pushon.count = 0;
    while !lobj.is_null() {
        let n = (*gp).u.pushon.count as usize;
        (*gp).u.pushon.lobjs[n] = lobj;
        (*gp).u.pushon.lobj_flags[n] = ext::HSD_LObjGetFlags(lobj);
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
        (*gp).u.pushon.count += 1;
    }
    (*gp).u.pushon.point_light = grPushOn_80218ED4((*gp).u.pushon.gobj as *mut HSD_GObj);
    (*gp).u.pushon.spot_light = grPushOn_80218FC0((*gp).u.pushon.gobj as *mut HSD_GObj);
    grPushOn_80218888(gobj);
}

pub unsafe extern "C" fn grPushOn_80218888(gobj: *mut HSD_GObj) {
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

    if !(*gp).u.pushon.gobj.is_null() {
        let mut i: i32 = 0;
        while i < (*gp).u.pushon.count {
            let lobj = (*gp).u.pushon.lobjs[i as usize];
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
                let d = player_pos.x - pos2.x;
                distances[i as usize] = if d < 0.0 { -d } else { d };
            } else {
                panic!("assert: grpushon.c:0x1C5 0");
            }
            if distances[i as usize] < 10.0 {
                distances[i as usize] = 10.0;
            }
            sorted[i as usize] = i;
            i += 1;
        }

        // Selection sort by distance
        i = 0;
        while i < (*gp).u.pushon.count {
            let count = (*gp).u.pushon.count;
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

        // Enable first 5 lights, disable rest
        i = 0;
        while i < (*gp).u.pushon.count {
            if i < 5 {
                ext::HSD_LObjClearFlags((*gp).u.pushon.lobjs[sorted[i as usize] as usize], 0x20);
            } else {
                ext::HSD_LObjSetFlags((*gp).u.pushon.lobjs[sorted[i as usize] as usize], 0x20);
            }
            i += 1;
        }

        // Compute weighted direction
        dir = grPushOn_803B844C;
        let ref_dist = distances[sorted[1] as usize];

        i = 1;
        while i < (*gp).u.pushon.count {
            if ext::HSD_LObjGetPosition((*gp).u.pushon.lobjs[sorted[i as usize] as usize], &mut light_pos) != 0 {
                let mut weight = ref_dist / distances[sorted[i as usize] as usize];
                ext::lbVector_Sub(&mut light_pos, &mut player_pos);
                if light_pos.y < 100.0 {
                    if light_pos.y < 0.0 {
                        weight = 0.0;
                    } else {
                        weight *= light_pos.y / 100.0;
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
        let dist_sq = sqrtf(fmadds(dir.z, dir.z, fmadds(dir.x, dir.x, dir.y * dir.y)));
        if dist_sq < 0.01 {
            dir.y = 100.0;
        }
        ext::lbVector_Normalize(&mut dir);
        dir.x *= 20.0;
        dir.y *= 20.0;
        dir.z *= 20.0;
        ext::lbVector_Add(&mut dir, &mut player_pos);

        // Set spotlight
        ext::HSD_LObjSetPosition((*gp).u.pushon.point_light, &mut dir);
        ext::HSD_LObjSetInterest((*gp).u.pushon.point_light, &mut player_pos);
        ext::HSD_LObjSetFlags((*gp).u.pushon.point_light, 0x400);

        // Color from 2nd nearest
        {
            let dist_diff = distances[sorted[2] as usize] - distances[sorted[1] as usize];
            ext::HSD_LObjGetColor((*gp).u.pushon.lobjs[sorted[1] as usize], &mut color);
            if distances[sorted[2] as usize] - distances[sorted[1] as usize] < 50.0 {
                let ratio = dist_diff / 50.0;
                color.r = (color.r as f32 * ratio) as u8;
                color.g = (color.g as f32 * ratio) as u8;
                color.b = (color.b as f32 * ratio) as u8;
            }
        }
        ext::HSD_LObjSetColor((*gp).u.pushon.spot_light, color);

        // Copy position from 2nd nearest
        if ext::HSD_LObjGetPosition((*gp).u.pushon.lobjs[sorted[1] as usize], &mut temp_pos) == 0 {
            temp_pos.z = 0.0;
            temp_pos.y = 0.0;
            temp_pos.x = 0.0;
        }
        ext::HSD_LObjSetPosition((*gp).u.pushon.spot_light, &mut temp_pos);

        // Copy interest from 2nd nearest
        if ext::HSD_LObjGetInterest((*gp).u.pushon.lobjs[sorted[1] as usize], &mut temp_pos) == 0 {
            temp_pos.z = 0.0;
            temp_pos.y = 0.0;
            temp_pos.x = 0.0;
        }
        ext::HSD_LObjSetInterest((*gp).u.pushon.spot_light, &mut temp_pos);

        // Copy and modify flags
        (*(*gp).u.pushon.spot_light).flags = (*(*gp).u.pushon.lobjs[sorted[1] as usize]).flags;
        (*(*gp).u.pushon.spot_light).flags &= !4;
        (*(*gp).u.pushon.spot_light).flags |= 8;
    }
}

pub unsafe extern "C" fn grPushOn_80218ED0(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grPushOn_80218ED4(gobj: *mut HSD_GObj) -> *mut HSD_LObj {
    let mut lobj: *mut HSD_LObj;
    if gobj.is_null() {
        return core::ptr::null_mut();
    }
    lobj = (*gobj).hsd_obj as *mut HSD_LObj;
    if lobj.is_null() {
        return core::ptr::null_mut();
    }
    while !(if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next }).is_null() {
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
    }
    let new_lobj = ext::HSD_LObjLoadDesc(&raw mut grPushOn_803E7B74);
    if !new_lobj.is_null() {
        let mut pos = grPushOn_803B8458;
        hsd_lobj_set_next(lobj, new_lobj);
        ext::HSD_LObjSetPosition(new_lobj, &mut pos);
        ext::HSD_LObjSetFlags(new_lobj, 0x20);
    }
    new_lobj
}

/// `HSD_LObjSetNext` (lobj.h inline): asserts the lobj is non-NULL.
unsafe fn hsd_lobj_set_next(lobj: *mut HSD_LObj, next: *mut HSD_LObj) {
    if lobj.is_null() {
        panic!("assert: lobj.h:0x136 lobj");
    }
    (*lobj).next = next;
}

pub unsafe extern "C" fn grPushOn_80218FC0(gobj: *mut HSD_GObj) -> *mut HSD_LObj {
    let mut lobj: *mut HSD_LObj;
    if gobj.is_null() {
        return core::ptr::null_mut();
    }
    lobj = (*gobj).hsd_obj as *mut HSD_LObj;
    if lobj.is_null() {
        return core::ptr::null_mut();
    }
    while !(if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next }).is_null() {
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
    }
    let new_lobj = ext::HSD_LObjLoadDesc(&raw mut grPushOn_803E7B90);
    if !new_lobj.is_null() {
        let mut pos = grPushOn_803B8464;
        hsd_lobj_set_next(lobj, new_lobj);
        ext::HSD_LObjSetPosition(new_lobj, &mut pos);
    }
    new_lobj
}

/// Ground collision callback for pushon stage elements: activates the push behaviour when the
/// collision flags indicate contact (b1234 == 1) on a floor/ceiling.
pub unsafe extern "C" fn fn_802190A0(
    user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    ground_kind: mpLib_GroundEnum,
    _delta_y: f32,
) {
    let gp = user_data as *mut Ground;
    let flags = *((&raw const (*coll).x34_flags) as *const u8);
    if ((flags >> 3) & 0xF) == 1 && ground_kind.wrapping_sub(1) <= 1 {
        (*gp).u.map.set_xC4_b0(1);
    }
}

pub unsafe extern "C" fn grPushOn_802190D0(gobj: *mut HSD_GObj) {
    let cur = (*gobj).hsd_obj as *mut HSD_LObj;
    let scale = ext::Ground_801C0498();
    let mut pos: Vec3;
    let mut lobj: *mut HSD_LObj = if cur.is_null() { core::ptr::null_mut() } else { (*cur).next };
    let mut entry = light_configs.as_ptr();
    let mut i: u32 = 0;

    while i < 9 && !lobj.is_null() {
        if ((*lobj).flags as i32 & (LOBJ_INFINITE | LOBJ_FLAGS_B1)) as u32 != LOBJ_POINT as u32 {
            panic!("assert: grpushon.c:698 HSD_LObjGetType(lobj)==LOBJ_POINT");
        }
        (*lobj).flags = (LOBJ_POINT | LOBJ_DIFFUSE) as u16;
        ext::HSD_LObjSetColor(lobj, (*entry).color);
        pos = (*entry).pos;
        pos.x *= scale;
        pos.y *= scale;
        pos.z *= scale;
        ext::HSD_LObjSetPosition(lobj, &mut pos);
        ext::HSD_LObjSetDistAttn(lobj, scale * (*entry).ref_dist, (*entry).ref_br, (*entry).dist_func);
        entry = entry.add(1);
        i += 1;
        lobj = if lobj.is_null() { core::ptr::null_mut() } else { (*lobj).next };
    }
}

pub unsafe extern "C" fn grPushOn_80219204(arg0: i32, out1: *mut i32, out2: *mut i32) {
    let idx = (arg0 - 0x99) as usize;
    *out1 = (*yakumono_param).x1c[idx].x0;
    *out2 = (*yakumono_param).x1c[idx].x4 as i32;
}

pub unsafe extern "C" fn grPushOn_80219230(mut arg0: i32) -> i32 {
    let mut i: i32 = 0;
    // TARGET_PC: Race to the Finish's x10c is a retail CharacterKind lookup.
    {
        let vanilla = ext::gm_MexVanillaKind(arg0, 1);
        if vanilla != arg0 {
            ext::OSReport(
                c"grPushOn_80219230: ckind %d uses retail ckind %d\n".as_ptr(),
                &[ext::OsArg::I(arg0 as i64), ext::OsArg::I(vanilla as i64)],
            );
            arg0 = vanilla;
        }
    }
    while i < 0x21 {
        let key = (*yakumono_param).x10c[i as usize].key;
        if key == -1 {
            break;
        }
        if key == arg0 {
            return (*yakumono_param).x10c[i as usize].value;
        }
        i += 1;
    }
    panic!("assert: grpushon.c:861 0");
}

pub unsafe extern "C" fn fn_802192A4(_arg0: *mut c_void, gobj: *mut HSD_GObj, result: *mut i32) -> i32 {
    let mut sp14 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let scale = ext::Ground_801C0498();
    ext::ftLib_80086644(gobj, &mut sp14);
    let mut i = 0usize;
    while i < 4 {
        let t = &grPushOn_803E7CCC;
        if scale * t[i * 3] < sp14.x
            && scale * t[i * 3 + 1] > sp14.x
            && scale * (-50.0 + t[i * 3 + 2]) < sp14.y
            && scale * t[i * 3 + 2] > sp14.y
        {
            *result = (*yakumono_param).x0;
            return 1;
        }
        i += 1;
    }
    0
}

pub unsafe extern "C" fn grPushOn_80219458(arg0: i32) -> *mut DynamicsDesc {
    if arg0 != -1 {
        let joint = ext::mpJointFromLine(arg0);
        if joint != -1 {
            if joint == 1 {
                ext::mpLineGetKind(arg0);
                return (*yakumono_param).x4;
            }
            if joint == 2 {
                let kind = ext::mpLineGetKind(arg0);
                if kind == CollLine_Floor as i32 {
                    return (*yakumono_param).x8;
                }
                if kind == CollLine_Ceiling as i32 {
                    return (*yakumono_param).xC;
                }
                if kind == CollLine_RightWall as i32 {
                    return (*yakumono_param).x10;
                }
                if kind == CollLine_LeftWall as i32 {
                    return (*yakumono_param).x14;
                }
                return core::ptr::null_mut();
            }
            return core::ptr::null_mut();
        }
    }
    core::ptr::null_mut()
}

pub static mut grPushOn_StageCallbacks: [StageCallbacks; 3] = [
    StageCallbacks {
        on_init: Some(grPushOn_802184CC),
        callback1: Some(grPushOn_80218590),
        gobj_proc: Some(grPushOn_80218598),
        callback3: Some(grPushOn_8021859C),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grPushOn_802185A0),
        callback1: Some(grPushOn_80218670),
        gobj_proc: Some(grPushOn_802186C8),
        callback3: Some(grPushOn_802187A4),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
    StageCallbacks {
        on_init: Some(grPushOn_802187A8),
        callback1: Some(grPushOn_80218880),
        gobj_proc: Some(grPushOn_80218888),
        callback3: Some(grPushOn_80218ED0),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
];

pub static mut grPushOn_StageData: StageData = StageData {
    grkind: Gr_Kind_Pushon,
    callbacks: unsafe { &raw mut grPushOn_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrNPo.dat".as_ptr() as *mut i8,
    on_init: Some(grPushOn_802182C8),
    on_demo_init: Some(grPushOn_802182C4),
    on_load: Some(grPushOn_80218330),
    on_start: Some(grPushOn_80218378),
    callback4: Some(grPushOn_802183DC),
    on_touch_line: Some(grPushOn_80219458),
    on_check_shadow_render: Some(grPushOn_80219528),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};
