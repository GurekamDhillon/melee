//! src/melee/gr/grstory.c: Yoshi's Story (Randall the Cloud and the Shy Guys).
//!
//! The constant/empty callbacks and the single-call forwarders already ported in `gr::trivial` /
//! `gr::forward` are re-exported; the rest is ported here. Callees outside this TU are `tu_ext`
//! hooks (unset: panic `TODO ... not ported`). State-oracle checked by tests/tu_grstory.rs against
//! gen/turef/tu_grstory_ref.c.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use super::gr_util::*;
use crate::tu_ext as ext;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::*;

pub use crate::gr::forward::grstory::grStory_UnkStage0_OnStart;
pub use crate::gr::trivial::grstory::{
    grStory_801E30D0, grStory_801E3224, grStory_801E332C, grStory_801E33D8, grStory_801E36D0,
};

/// `struct grStory_YakumonoParam`: the stage's yakumono (parameter) block.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grStory_YakumonoParam {
    pub timer_min: f32,
    pub timer_rand: f32,
    pub spawnmany_rarity: f32,
    pub vpos: [f32; 6],
}
melee_core::gc_layout!(grStory_YakumonoParam, "grStory_YakumonoParam", size 0x24, align 4, [
    timer_min @ 0x0: 4 "float",
    timer_rand @ 0x4: 4 "float",
    spawnmany_rarity @ 0x8: 4 "float",
    vpos @ 0xC: 24 "float[6]",
]);

/// `static struct grStory_YakumonoParam* yakumono_param`.
pub static mut yakumono_param: *mut grStory_YakumonoParam = core::ptr::null_mut();

/// `grStory_StageSlotParamSwap` (TARGET_PC): install another stage-slot parameter block, returning
/// the previous one.
pub unsafe extern "C" fn grStory_StageSlotParamSwap(param: *mut c_void) -> *mut c_void {
    melee_core::ct!("grStory_StageSlotParamSwap");
    let previous = yakumono_param as *mut c_void;
    yakumono_param = param as *mut grStory_YakumonoParam;
    previous
}

pub unsafe extern "C" fn grStory_801E302C(_unused: i32) { melee_core::ct!("grStory_801E302C");}

pub unsafe extern "C" fn grStory_801E3030() {
    melee_core::ct!("grStory_801E3030");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grStory_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grStory_801E30D8(0);
    grStory_801E30D8(1);
    grStory_801E30D8(3);
    grStory_801E30D8(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grStory_UnkStage0_OnLoad() { melee_core::ct!("grStory_UnkStage0_OnLoad");}

pub unsafe extern "C" fn grStory_801E30D8(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grStory_801E30D8");
    let callbacks = (&raw mut grSt_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        report_no_gobj(c"grstory.c".as_ptr(), 220, gobj_id);
    }
    gobj
}

pub unsafe extern "C" fn grStory_801E31C0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E31C0");
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
    ext::grMaterial_801C8858(ext::Ground_801C3FA4(gobj, 1), 0x20000000);
}

pub unsafe extern "C" fn grStory_801E322C(_gobj: *mut HSD_GObj) { melee_core::ct!("grStory_801E322C");}

pub unsafe extern "C" fn grStory_801E3230(_gobj: *mut HSD_GObj) { melee_core::ct!("grStory_801E3230");}

/// `reset_shyguy_timer`: the randomised timer is overwritten by the hardcoded 120 right after.
unsafe fn reset_shyguy_timer(gp: *mut Ground) {
    let yp = yakumono_param;
    (*gp).u.shyguys.timer = ((*yp).timer_min + randi((*yp).timer_rand as i32) as f32) as i32;
    (*gp).u.shyguys.timer = 120;
}

/// `set_shyguy_spawn_count`: 1, or 3 to 5 with a one in `rarity` chance.
unsafe fn set_shyguy_spawn_count(gp: *mut Ground, rarity: i32) {
    if randi(rarity) == 0 {
        (*gp).u.shyguys.count = (randi(3) + 3) as i8;
    } else {
        (*gp).u.shyguys.count = 1;
    }
}

pub unsafe extern "C" fn grStory_801E3234(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E3234");
    let gp = GET_GROUND(gobj);
    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 1.0);
    ext::grAnime_801C7FF8(gobj, 5, 7, 1, 0.0, 1.0);
    reset_shyguy_timer(gp);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grStory_801E3334(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E3334");
    grStory_801E3418(gobj);
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grStory_801E336C(_gobj: *mut HSD_GObj) { melee_core::ct!("grStory_801E336C");}

pub unsafe extern "C" fn grStory_801E3370(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E3370");
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).u.randall.timer = 0;
    (*gp).u.randall.jobj = ext::Ground_801C3FA4(gobj, 1);
}

pub unsafe extern "C" fn grStory_801E33E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E33E0");
    ext::Ground_UpdateMapColl(gobj);
    grStory_801E366C(gobj);
}

pub unsafe extern "C" fn grStory_801E3414(_gobj: *mut HSD_GObj) { melee_core::ct!("grStory_801E3414");}

/// `frand_amp1`: a float random number centred at 0 with an amplitude of 1.
#[inline]
unsafe fn frand_amp1() -> f32 {
    2.0 * (ext::HSD_Randf() - 0.5)
}

pub unsafe extern "C" fn grStory_801E3418(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E3418");
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut spawn_pattern: i32;
    let gp = GET_GROUND(gobj);

    if ext::it_8026B3C0(It_Kind_Heiho) != 0 {
        return;
    }
    if (*gp).u.shyguys.timer != 0 {
        (*gp).u.shyguys.timer -= 1;
        return;
    }
    reset_shyguy_timer(gp);
    loop {
        spawn_pattern = randi(6);
        if (*gp).u.shyguys.pattern as i32 != spawn_pattern {
            break;
        }
    }
    (*gp).u.shyguys.pattern = spawn_pattern as i8;
    if spawn_pattern < 3 {
        pos.x = -292.0;
    } else {
        pos.x = 304.0;
    }
    pos.y = (*yakumono_param).vpos[spawn_pattern as usize];
    pos.z = 2.0;
    {
        let temp_r29 = randi(3);
        set_shyguy_spawn_count(gp, (*yakumono_param).spawnmany_rarity as i32);
        set_shyguy_spawn_count(gp, 2);
        let mut i = 0;
        while i < (*gp).u.shyguys.count as i32 {
            ext::it_802D8618(i, &mut pos, temp_r29, (25.0 * i as f32) as i32);
            pos.y = fmadds(3.0, frand_amp1(), (*yakumono_param).vpos[spawn_pattern as usize]);
            i += 1;
        }
    }
}

pub unsafe extern "C" fn grStory_801E366C(gobj: *mut HSD_GObj) {
    melee_core::ct!("grStory_801E366C");
    let gp = GET_GROUND(gobj);
    let t = (*gp).u.randall.timer;
    (*gp).u.randall.timer = t.wrapping_sub(1);
    if t >= 0 {
        return;
    }
    if !(*gp).u.randall.jobj.is_null() {
        ext::grLib_801C97DC(0x2C, 0, (*gp).u.randall.jobj);
        (*gp).u.randall.timer = (ext::HSD_Randi(20) + 10) as i16;
    }
}

pub unsafe extern "C" fn grStory_801E36D8(a: *mut Vec3, _unused: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grStory_801E36D8");
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, core::ptr::null_mut(), &mut b);
    if (*a).y + 1.0 > b.y { 1 } else { 0 }
}

pub static mut grSt_StageCallbacks: [StageCallbacks; 4] = [
    StageCallbacks { on_init: None, callback1: None, gobj_proc: None, callback3: None, anon_10: StageCallbacks_anon_135 { flags: 0 } },
    StageCallbacks {
        on_init: Some(grStory_801E31C0),
        callback1: Some(grStory_801E3224),
        gobj_proc: Some(grStory_801E322C),
        callback3: Some(grStory_801E3230),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grStory_801E3370),
        callback1: Some(grStory_801E33D8),
        gobj_proc: Some(grStory_801E33E0),
        callback3: Some(grStory_801E3414),
        anon_10: StageCallbacks_anon_135 { flags: 0 },
    },
    StageCallbacks {
        on_init: Some(grStory_801E3234),
        callback1: Some(grStory_801E332C),
        gobj_proc: Some(grStory_801E3334),
        callback3: Some(grStory_801E336C),
        anon_10: StageCallbacks_anon_135 { flags: (1 << 30) | (1 << 31) },
    },
];

pub static mut grSt_StageData: StageData = StageData {
    grkind: Gr_Kind_Story,
    callbacks: unsafe { &raw mut grSt_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrSt.dat".as_ptr() as *mut i8,
    on_init: Some(grStory_801E3030),
    on_demo_init: Some(grStory_801E302C),
    on_load: Some(grStory_UnkStage0_OnLoad),
    on_start: Some(grStory_UnkStage0_OnStart),
    callback4: Some(grStory_801E30D0),
    on_touch_line: Some(grStory_801E36D0),
    on_check_shadow_render: Some(grStory_801E36D8),
    flags2: 1 << 0,
    joints: core::ptr::null_mut(),
    joint_count: 0,
};
