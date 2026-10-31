//! Port of src/melee/gr/grheal.c (Pokemon Stadium's cousin "Yoshi's Story"? no: the "Heal" stage,
//! i.e. the Fountain-of-style Target/Home-run scoreboard: the next-player icons, bob-omb rain and
//! the coin). Checked against the C (whole TU, shipping contraction) by tests/tu_grheal.rs.
//! Float arithmetic: `2.0F * Stage_GetBlastZoneTopOffset()` (one f32 product, nothing to fuse).
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::grheal::{
    grHeal_8021EFE4, grHeal_8021F830, grHeal_8021F838, stageGObj0_Callback1, stageGObj1_Callback1, stageGObj2_Callback1,
    stageGObj3_Callback1, stageGObj4_Callback1,
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
    fn Ground_801C2D24(a0: i32, a1: *mut Vec) -> i32 = |a0: i32, a1: *mut Vec| unsafe { crate::tu::ground::Ground_801C2D24(a0 as _, a1 as _) as _ };
    fn Ground_801C3980(a0: f32) = |a0: f32| { unsafe { crate::tu::ground::Ground_801C3980(a0 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3D44(a0: *mut c_void, a1: f32, a2: f32) -> i32 = |a0: *mut c_void, a1: f32, a2: f32| unsafe { crate::tu::ground::Ground_801C3D44(a0 as _, a1 as _, a2 as _) as _ };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_801C5840() -> i32 = || unsafe { crate::tu::ground::Ground_801C5840() as _ };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjReparent(a0: *mut HSD_JObj, a1: *mut HSD_JObj) -> *mut HSD_JObj = |a0: *mut HSD_JObj, a1: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjReparent(a0 as _, a1 as _) as _ };
    fn Stage_GetBlastZoneTopOffset() -> f32 = || unsafe { crate::tu::stage::Stage_GetBlastZoneTopOffset() as _ };
    fn Toy_80304A58(a0: i32);
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_8026BE84(a0: *mut BobOmbRain) -> *mut HSD_GObj;
    fn it_802F2094(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: i32) -> *mut HSD_GObj;
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::shared::gm_80473A18;
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

/// `grHeal_UnkData` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grHeal_UnkData {
    pub x0: i32,
    pub x4: i32,
}

/// `grHeal_803B84A8` (3B84A8)
pub static grHeal_803B84A8: Vec3 = Vec3 { x: 0.0, y: 40.0, z: 0.0 };

/// `char_id_count`
pub const char_id_count: usize = 26;

/// `grHeal_803E83B8` (3E83B8)
pub static grHeal_803E83B8: [i16; 26] =
    [29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54];

/// `frame_to_character_id`
pub static frame_to_character_id: [enum_t; 26] =
    [0, 21, 1, 22, 20, 2, 25, 3, 14, 4, 5, 7, 6, 8, 9, 10, 11, 24, 12, 13, 15, 16, 17, 18, 23, -1];

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

/// `stage_callbacks`
pub static mut stage_callbacks: [StageCallbacks; 5] = [
    cbs(Some(stageGObj0_OnInit), Some(stageGObj0_Callback1), Some(stageGObj0_GObjProc), Some(stageGObj0_Callback3), 0),
    cbs(Some(stageGObj1_OnInit), Some(stageGObj1_Callback1), Some(stageGObj1_GObjProc), Some(stageGObj1_Callback3), (1 << 30) | (1 << 31)),
    cbs(Some(stageGObj2_OnInit), Some(stageGObj2_Callback1), Some(stageGObj2_GObjProc), Some(stageGObj2_Callback3), 0),
    cbs(Some(stageGObj3_OnInit), Some(stageGObj3_Callback1), Some(stageGObj3_GObjProc), Some(stageGObj3_Callback3), 0),
    cbs(Some(stageGObj4_OnInit), Some(stageGObj4_Callback1), Some(stageGObj4_GObjProc), Some(stageGObj4_Callback3), 0),
];

/// `grHeal_StageData`
pub static mut grHeal_StageData: StageData = StageData {
    grkind: Gr_Kind_Heal,
    callbacks: unsafe { &raw mut stage_callbacks as *mut StageCallbacks },
    data1: b"/GrHe.dat\0".as_ptr() as *mut i8,
    on_init: Some(grHeal_8021EF3C),
    on_demo_init: Some(grHeal_8021EF38),
    on_load: Some(grHeal_8021EFBC),
    on_start: Some(grHeal_8021EFC0),
    callback4: Some(grHeal_8021EFE4),
    on_touch_line: Some(grHeal_8021F830),
    on_check_shadow_render: Some(grHeal_8021F838),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `grHeal_804D49D8` (4D49D8)
pub static grHeal_804D49D8: [i16; 4] = [7, 8, 9, 0];

/// `static grHeal_UnkData* grHeal_804D6AF0[2]` (4D6AF0)
pub static mut grHeal_804D6AF0: [*mut grHeal_UnkData; 2] = [null_mut(); 2];

// `_94[next_count + 2]` in stageGObj1_OnInit reads on into `x96`
const _: () = assert!(core::mem::offset_of!(UnkAllstarData, x96) == core::mem::offset_of!(UnkAllstarData, _94) + 2);

/// `%` as the shipping build computes it (gwtool's makeArithmeticSafe: x % 0 == x, INT_MIN % -1 == 0)
#[inline(always)]
fn srem(a: i32, b: i32) -> i32 {
    if b == 0 { a } else if a == i32::MIN && b == -1 { 0 } else { a % b }
}

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// 21EF38
pub unsafe extern "C" fn grHeal_8021EF38(_arg0: i32) { melee_core::ct!("grHeal_8021EF38");}

/// 21EF3C
pub unsafe extern "C" fn grHeal_8021EF3C() {
    melee_core::ct!("grHeal_8021EF3C");
    grHeal_804D6AF0[0] = ext::Ground_GetYakumonoParam() as *mut grHeal_UnkData;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);

    grHeal_8021EFEC(0);
    grHeal_8021EFEC(1);
    grHeal_8021EFEC(3);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Ground_801C3980(2.0 * ext::Stage_GetBlastZoneTopOffset());
}

/// 21EFBC
pub unsafe extern "C" fn grHeal_8021EFBC() { melee_core::ct!("grHeal_8021EFBC");}

/// 21EFC0
pub unsafe extern "C" fn grHeal_8021EFC0() {
    melee_core::ct!("grHeal_8021EFC0");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 21EFEC
pub unsafe extern "C" fn grHeal_8021EFEC(gobj_id: u32) -> *mut Ground_GObj {
    melee_core::ct!("grHeal_8021EFEC");
    let callbacks = &raw mut stage_callbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id as i32);

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
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 273, gobj_id);
    }

    gobj
}

/// 21F0D8
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut Ground_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);

    let mut i: i32 = 0;
    while i < (*grHeal_804D6AF0[0]).x0 {
        if ext::gm_80473A18.x90[i as usize] as i32 != 0 {
            grHeal_8021F79C(8, grHeal_804D49D8[i as usize] as i32, i);
        }
        i += 1;
    }
}

/// 21F178
pub unsafe extern "C" fn stageGObj0_GObjProc(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj0_GObjProc");}

/// 21F17C
pub unsafe extern "C" fn stageGObj0_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj0_Callback3");}

/// 21F180
pub unsafe extern "C" fn stageGObj1_OnInit(gobj: *mut Ground_GObj) {
    melee_core::ct!("stageGObj1_OnInit");
    let mut coin_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut next_players = [0u8; 4];
    let mut next_jobjs: [*mut HSD_JObj; 3] = [null_mut(); 3];
    let mut player_jobjs: [*mut HSD_JObj; char_id_count] = [null_mut(); char_id_count];

    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);

    if srem(ext::gm_80473A18._94[0] as i32, (*grHeal_804D6AF0[0]).x4) == 0 {
        coin_pos = grHeal_803B84A8;
        ext::Ground_801C2D24(0xDC, &mut coin_pos);
        let line_id = ext::Ground_801C5840();
        if line_id != -1 {
            ext::it_802F2094(null_mut(), &mut coin_pos, line_id, 0);
            ext::Toy_80304A58(line_id);
        }
    }

    let mut next_count: i32 = 0;
    while next_count < ext::gm_80473A18._94[1] as i32 {
        // _94[next_count + 2] runs on into x96
        next_players[next_count as usize] = ext::gm_80473A18.x96[next_count as usize];
        next_count += 1;
    }

    next_jobjs[0] = ext::Ground_801C3FA4(gobj, 58);
    next_jobjs[1] = ext::Ground_801C3FA4(gobj, 59);
    next_jobjs[2] = ext::Ground_801C3FA4(gobj, 60);

    let mut next_idx: i32 = 0;
    while next_idx < next_count {
        let a = grHeal_8021F70C(next_players[next_idx as usize] as enum_t);
        grHeal_8021F4E8(a, next_jobjs[next_idx as usize]);
        next_idx += 1;
    }

    for i in 0..grHeal_803E83B8.len() {
        player_jobjs[i] = ext::Ground_801C3FA4(gobj, grHeal_803E83B8[i] as i32);
    }

    for i in 0..grHeal_803E83B8.len() {
        let character_id = ext::gm_80473A18.x76[i] as enum_t;
        if character_id != 33 {
            let a = grHeal_8021F70C(character_id);
            grHeal_8021F628(a, player_jobjs[i]);
        }
    }

    ext::mpJointSetCb1(0, gp as *mut c_void, Some(onJointCollision));
    (*gp).u.flatzone2.xC4 = 0;
}

/// 21F424
pub unsafe extern "C" fn fn_8021F424() -> i32 {
    melee_core::ct!("fn_8021F424");
    let ground = ext::Ground_GetMapGObj(1);
    if !ground.is_null() {
        let gp = (*ground).user_data as *mut Ground;
        if !gp.is_null() && (*gp).u.unk.xC4 != 0 {
            return 1;
        }
    }
    0
}

/// 21F474
pub unsafe extern "C" fn stageGObj1_GObjProc(ground: *mut Ground_GObj) {
    melee_core::ct!("stageGObj1_GObjProc");
    let gp = (*ground).user_data as *mut Ground;
    ext::Ground_801C3D44(fn_8021F424 as *mut c_void, 10.0, 20.0);
    ext::lb_800115F4();
    (*gp).u.flatzone2.xC4 = 0;
}

/// 21F4BC
pub unsafe extern "C" fn stageGObj1_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj1_Callback3");}

/// 21F4C0 (`onJointCollision`, an `mpLib_JointCollisionCallback`)
pub unsafe extern "C" fn onJointCollision(user_data: *mut c_void, _joint_id: i32, coll: *mut CollData,
                                          _coll_x50: i32, ground_kind: u32, _delta_y: f32) {
    melee_core::ct!("onJointCollision");
    let gp = user_data as *mut Ground;
    if (*coll).x34_flags.b1234() as i32 == 1 && ground_kind.wrapping_sub(1) <= 1 {
        (*gp).u.unk.xC4 = 1;
    }
}

/// 21F4E8
pub unsafe extern "C" fn grHeal_8021F4E8(arg0: i32, parent_jobj: *mut HSD_JObj) {
    melee_core::ct!("grHeal_8021F4E8");
    let gobj = grHeal_8021EFEC(4);
    assert!(!gobj.is_null(), "gobj"); // HSD_ASSERT(440, gobj)
    ext::grAnime_801C8138(gobj, 4, 0);
    ext::grAnime_801C7FF8(gobj, 0, 7, 0, arg0 as f32, 0.0);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child = HSD_JObjGetChild(jobj.as_ref());
    ext::HSD_JObjReparent(child, parent_jobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    ext::Ground_801C4A08(gobj);
}

/// 21F5C8
pub unsafe extern "C" fn stageGObj2_OnInit(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj2_OnInit");}

/// 21F5D4
pub unsafe extern "C" fn stageGObj2_GObjProc(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj2_GObjProc");}

/// 21F5D8
pub unsafe extern "C" fn stageGObj2_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj2_Callback3");}

/// 21F5DC
pub unsafe extern "C" fn stageGObj3_OnInit(ground: *mut Ground_GObj) {
    melee_core::ct!("stageGObj3_OnInit");
    let gp = (*ground).user_data as *mut Ground;
    (*gp).x10_flags.set_b2(0);
    ext::grAnime_801C8138(ground, (*gp).map_id as i32, 0);
}

/// 21F620
pub unsafe extern "C" fn stageGObj3_GObjProc(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj3_GObjProc");}

/// 21F624
pub unsafe extern "C" fn stageGObj3_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj3_Callback3");}

/// 21F628
pub unsafe extern "C" fn grHeal_8021F628(arg0: i32, jobj_parent: *mut HSD_JObj) {
    melee_core::ct!("grHeal_8021F628");
    let gobj = grHeal_8021EFEC(2);
    assert!(!gobj.is_null(), "gobj"); // HSD_ASSERT(513, gobj)
    ext::grAnime_801C8138(gobj, 2, 0);
    ext::grAnime_801C7FF8(gobj, 0, 7, 0, arg0 as f32, 0.0);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let child = HSD_JObjGetChild(jobj.as_ref());
    ext::HSD_JObjReparent(child, jobj_parent);
    ext::Ground_801C4A08(gobj);
}

/// 21F6F8
pub unsafe extern "C" fn stageGObj4_OnInit(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj4_OnInit");}

/// 21F704
pub unsafe extern "C" fn stageGObj4_GObjProc(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj4_GObjProc");}

/// 21F708
pub unsafe extern "C" fn stageGObj4_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("stageGObj4_Callback3");}

/// 21F70C
pub unsafe extern "C" fn grHeal_8021F70C(mut character_id: enum_t) -> i32 {
    melee_core::ct!("grHeal_8021F70C");
    let mut frame: i32 = 0;

    if character_id == 19 {
        character_id = 18;
    }
    while frame_to_character_id[frame as usize] != -1 {
        if character_id == frame_to_character_id[frame as usize] {
            break;
        }
        frame += 1;
    }
    if frame_to_character_id[frame as usize] == -1 {
        // OSReport("*** Not found Next Player!(%d)\n", character_id);
        frame = 0;
    }
    frame
}

/// 21F79C
pub unsafe extern "C" fn grHeal_8021F79C(arg0: i32, idx: i32, arg2: i32) {
    melee_core::ct!("grHeal_8021F79C");
    let mut bobomb_rain: BobOmbRain = core::mem::zeroed();

    let gp = ext::Ground_GetMapGObj(0);
    let jobj = ext::Ground_801C3FA4(gp, idx);
    bobomb_rain.x0 = gp;
    bobomb_rain.x4 = null_mut();
    bobomb_rain.x14 = arg0;
    bobomb_rain.x18 = arg2;
    bobomb_rain.x1C.anon_0.set_b0(1);
    ext::lb_8000B1CC(jobj, null_mut(), &mut bobomb_rain.x8_vec);
    ext::it_8026BE84(&mut bobomb_rain);
}
