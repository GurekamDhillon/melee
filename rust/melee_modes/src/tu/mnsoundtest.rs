//! src/melee/mn/mnsoundtest.c: the Sound Test menu: the sound-effect and music lists, the volume
//! and view toggles, the sample / category animations and the scene builder.
//! Callees in TUs that are not ported yet are `ext` stubs named like the C function.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_mut, unused_variables, unused_assignments, clippy::missing_safety_doc)]

use crate::mn_inl::*;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// `SoundTestModelDesc` (mnsoundtest.c): four untyped archive section pointers.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SoundTestModelDesc {
    pub joint: *mut c_void,
    pub animjoint: *mut c_void,
    pub matanim_joint: *mut c_void,
    pub shapeanim_joint: *mut c_void,
}

melee_core::gc_layout!(SoundTestModelDesc, "SoundTestModelDesc", size 0x10, align 4, [
    joint @ 0x0: 4 "void*",
    animjoint @ 0x4: 4 "void*",
    matanim_joint @ 0x8: 4 "void*",
    shapeanim_joint @ 0xC: 4 "void*",
]);

/// `soundtest_data` (mnsoundtest.c): one sound-test row.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct soundtest_data {
    pub text_id: u16,
    pub idx: u32,
}

melee_core::gc_layout!(soundtest_data, "soundtest_data", size 8, align 4, [
    text_id @ 0x0: 2 "u16",
    idx @ 0x4: 4 "u32",
]);

pub mod ext {
    use super::*;
    use crate::ext_fns;
    ext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        extern "C" fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, arg1: i32) = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn lbAudioAx_80023090(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023090__11c684;
        fn lbAudioAx_80023694() -> i32 = crate::tu::adapt::lbAudioAx_80023694__e3b7af;
        fn lbAudioAx_800236DC() -> i32 = crate::tu::adapt::lbAudioAx_800236DC__5d9f3e;
        fn lbAudioAx_80023968(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023968__138b6f;
        fn lbAudioAx_80023A44(a0: i32, a1: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023A44__d67545;
        fn lbAudioAx_80023B24(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023B24__808380;
        fn lbAudioAx_80023F28(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_80023F28__582fc2;
        fn lbAudioAx_800245F4(a0: i32) -> i32 = crate::tu::adapt::lbAudioAx_800245F4__23a8b4;
        fn lbAudioAx_80024614(a0: i32) = crate::tu::adapt::lbAudioAx_80024614__7a0894;
        fn mn_80229894(a0: i32, a1: u16, a2: i32);
        fn mn_8022EA08(buf: *mut c_char, num: u32);
        fn mn_8022EC18(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings, a2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EC18__bee2c1;
        fn mn_8022EE84(arg0: *mut HSD_JObj, arg1: *mut AnimLoopSettings, arg2: HSD_TypeMask) -> f32 = crate::tu::adapt::mn_8022EE84__fdce2b;
        fn mn_8022EFD8(a0: *mut HSD_JObj, a1: *mut AnimLoopSettings) -> f32 = crate::tu::adapt::mn_8022EFD8__c1f371;
        fn mn_8022F298(a0: *mut HSD_JObj) -> f32 = crate::tu::adapt::mn_8022F298__0e9b78;
        fn mn_8022F3D8(a0: *mut HSD_JObj, a1: u8, a2: HSD_TypeMask) = crate::tu::adapt::mn_8022F3D8__9f7fac;
        fn HSD_JObjReqAnimAll(a0: *mut HSD_JObj, a1: f32);
        fn HSD_JObjLoadJoint(a0: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjAddAnimAll(a0: *mut HSD_JObj, a1: *mut HSD_AnimJoint, a2: *mut HSD_MatAnimJoint, a3: *mut HSD_ShapeAnimJoint);
        fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
        fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, flags: u32);
        fn HSD_JObjReqAnim(jobj: *mut HSD_JObj, frame: f32);
        fn HSD_JObjAnim(jobj: *mut HSD_JObj);
        fn HSD_SisLib_803A6754(arg0: i32, arg1: i32) -> *mut HSD_Text = crate::tu::adapt::HSD_SisLib_803A6754__02fb41;
        fn HSD_SisLib_803A5CC4(a0: *mut HSD_Text) = crate::tu::adapt::HSD_SisLib_803A5CC4__6c7e38;
        fn gm_801601C4(a0: i8) -> i32 = crate::gm::gm_801601C4;
        fn gm_80160244(a0: i8) -> i32 = crate::gm::gm_80160244;
        fn gm_801603B0() = crate::tu::gm_1601::gm_801603B0;
        fn gmMainLib_8015ECB0() -> i32 = crate::tu::gmmain_lib::gmMainLib_8015ECB0;
        fn gmMainLib_8015ED68(port: isize) = crate::tu::gmmain_lib::gmMainLib_8015ED68;
        fn gmMainLib_8015ED74() -> u8 = crate::tu::gmmain_lib::gmMainLib_8015ED74;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32);
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void);
        fn HSD_GObjFree(a0: *mut HSD_GObj);
        fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc;
        fn HSD_GObjProc_RemoveProc(gproc: *mut HSD_GObjProc);
        fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, remove_func: Option<unsafe extern "C" fn(*mut c_void) -> ()>, data: *mut c_void, );
        extern "C" fn HSD_Free(ptr: *mut c_void) = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_MemAlloc(size: isize) -> *mut c_void = crate::tu::adapt::HSD_MemAlloc__849f1b;
    }
    pub static mut HSD_GObj_CurrentInvokedProc: *mut HSD_GObjProc = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_804D783C: i32 = unsafe { core::mem::zeroed() };
    pub static mut HSD_GObj_JObjKind: u8 = unsafe { core::mem::zeroed() };
    pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
}
use ext::*;

static mut mnSoundTest_804D6C40: *mut mnSoundTest_GObj = core::ptr::null::<
    mnSoundTest_GObj,
>() as *mut mnSoundTest_GObj;
static mut mnSoundTest_804D6C44: u8 = 0;
static mut mnSoundTest_804D6C48: i32 = 0;
static mut mnSoundTest_804D6C4C: i32 = 0;
static mut mnSoundTest_804A08C8: SoundTestModelDesc = SoundTestModelDesc {
    joint: null_mut(),
    animjoint: null_mut(),
    matanim_joint: null_mut(),
    shapeanim_joint: null_mut(),
};
pub static mut vec_0: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 19 as i32 as f32,
    loop_frame: -0.1f64 as f32,
};
pub static mut vec_1: AnimLoopSettings = AnimLoopSettings {
    start_frame: 20 as i32 as f32,
    end_frame: 39 as i32 as f32,
    loop_frame: -0.1f64 as f32,
};
pub static mut vec_2: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 20 as i32 as f32,
    loop_frame: -0.1f64 as f32,
};
pub static mut vec_3: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 0 as i32 as f32,
    loop_frame: -0.1f64 as f32,
};
pub static mut vec_4: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 200 as i32 as f32,
    loop_frame: 0 as i32 as f32,
};
pub static mut vec_5: AnimLoopSettings = AnimLoopSettings {
    start_frame: 0 as i32 as f32,
    end_frame: 10 as i32 as f32,
    loop_frame: -0.1f64 as f32,
};
pub static mut vec_6: Vec3 = Vec3 {
    x: -2 as f32,
    y: -6.6f64 as f32,
    z: 0 as i32 as f32,
};
pub static mut vec_7: Vec3 = Vec3 {
    x: -1.7f64 as f32,
    y: -4.7f64 as f32,
    z: 0 as i32 as f32,
};
pub static mut vec_8: Vec3 = Vec3 {
    x: -1.7f64 as f32,
    y: 2 as i32 as f32,
    z: 0 as i32 as f32,
};
pub static mut text_ids: [u8; 80] = [
    0x2a as i32 as u8,
    0x2 as i32 as u8,
    0x2d as i32 as u8,
    0x20 as i32 as u8,
    0x15 as i32 as u8,
    0x16 as i32 as u8,
    0x2f as i32 as u8,
    0x34 as i32 as u8,
    0x21 as i32 as u8,
    0x33 as i32 as u8,
    0x32 as i32 as u8,
    0x1f as i32 as u8,
    0x17 as i32 as u8,
    0x4 as i32 as u8,
    0x31 as i32 as u8,
    0x2b as i32 as u8,
    0x2c as i32 as u8,
    0x24 as i32 as u8,
    0x1 as i32 as u8,
    0x28 as i32 as u8,
    0x13 as i32 as u8,
    0x19 as i32 as u8,
    0x1a as i32 as u8,
    0x1b as i32 as u8,
    0x1c as i32 as u8,
    0x18 as i32 as u8,
    0x3a as i32 as u8,
    0x25 as i32 as u8,
    0x27 as i32 as u8,
    0x26 as i32 as u8,
    0x3c as i32 as u8,
    0x2e as i32 as u8,
    0x3b as i32 as u8,
    0 as i32 as u8,
    0x37 as i32 as u8,
    0x29 as i32 as u8,
    0x3d as i32 as u8,
    0x3e as i32 as u8,
    0xe as i32 as u8,
    0x7 as i32 as u8,
    0xd as i32 as u8,
    0x11 as i32 as u8,
    0x12 as i32 as u8,
    0xc as i32 as u8,
    0x9 as i32 as u8,
    0x10 as i32 as u8,
    0xa as i32 as u8,
    0xf as i32 as u8,
    0x8 as i32 as u8,
    0x3f as i32 as u8,
    0xb as i32 as u8,
    0x38 as i32 as u8,
    0x40 as i32 as u8,
    0x41 as i32 as u8,
    0x22 as i32 as u8,
    0x35 as i32 as u8,
    0x4f as i32 as u8,
    0x39 as i32 as u8,
    0x43 as i32 as u8,
    0x44 as i32 as u8,
    0x36 as i32 as u8,
    0x45 as i32 as u8,
    0x46 as i32 as u8,
    0x23 as i32 as u8,
    0x1d as i32 as u8,
    0x1e as i32 as u8,
    0x5 as i32 as u8,
    0x6 as i32 as u8,
    0x3 as i32 as u8,
    0x14 as i32 as u8,
    0x47 as i32 as u8,
    0x48 as i32 as u8,
    0x49 as i32 as u8,
    0x4a as i32 as u8,
    0x4b as i32 as u8,
    0x4c as i32 as u8,
    0x4d as i32 as u8,
    0x4e as i32 as u8,
    0x30 as i32 as u8,
    0x42 as i32 as u8,
];
pub static mut data_2: [soundtest_data; 80] = [
    soundtest_data {
        text_id: 0xe7,
        idx: 0x1,
    },
    soundtest_data {
        text_id: 0xe8,
        idx: 0x3,
    },
    soundtest_data {
        text_id: 0xe9,
        idx: 0x4,
    },
    soundtest_data {
        text_id: 0xea,
        idx: 0x5,
    },
    soundtest_data {
        text_id: 0xeb,
        idx: 0x6,
    },
    soundtest_data {
        text_id: 0xed,
        idx: 0xa,
    },
    soundtest_data {
        text_id: 0xee,
        idx: 0xb,
    },
    soundtest_data {
        text_id: 0xef,
        idx: 0xd,
    },
    soundtest_data {
        text_id: 0xf0,
        idx: 0xe,
    },
    soundtest_data {
        text_id: 0xf1,
        idx: 0x10,
    },
    soundtest_data {
        text_id: 0xf2,
        idx: 0x11,
    },
    soundtest_data {
        text_id: 0xf3,
        idx: 0x13,
    },
    soundtest_data {
        text_id: 0xf4,
        idx: 0x14,
    },
    soundtest_data {
        text_id: 0xf5,
        idx: 0x15,
    },
    soundtest_data {
        text_id: 0xf6,
        idx: 0x16,
    },
    soundtest_data {
        text_id: 0xf7,
        idx: 0x17,
    },
    soundtest_data {
        text_id: 0xf8,
        idx: 0x18,
    },
    soundtest_data {
        text_id: 0xf9,
        idx: 0x19,
    },
    soundtest_data {
        text_id: 0xfa,
        idx: 0x1d,
    },
    soundtest_data {
        text_id: 0xfb,
        idx: 0x1f,
    },
    soundtest_data {
        text_id: 0xfc,
        idx: 0x20,
    },
    soundtest_data {
        text_id: 0xfd,
        idx: 0x32,
    },
    soundtest_data {
        text_id: 0xfe,
        idx: 0x22,
    },
    soundtest_data {
        text_id: 0xff,
        idx: 0x23,
    },
    soundtest_data {
        text_id: 0x100,
        idx: 0x28,
    },
    soundtest_data {
        text_id: 0x101,
        idx: 0x29,
    },
    soundtest_data {
        text_id: 0x102,
        idx: 0x2a,
    },
    soundtest_data {
        text_id: 0x103,
        idx: 0x2b,
    },
    soundtest_data {
        text_id: 0x104,
        idx: 0x2c,
    },
    soundtest_data {
        text_id: 0x105,
        idx: 0x2d,
    },
    soundtest_data {
        text_id: 0x106,
        idx: 0x2e,
    },
    soundtest_data {
        text_id: 0x107,
        idx: 0x31,
    },
    soundtest_data {
        text_id: 0x108,
        idx: 0x21,
    },
    soundtest_data {
        text_id: 0x109,
        idx: 0x33,
    },
    soundtest_data {
        text_id: 0x10a,
        idx: 0x34,
    },
    soundtest_data {
        text_id: 0x10b,
        idx: 0x35,
    },
    soundtest_data {
        text_id: 0x10c,
        idx: 0x38,
    },
    soundtest_data {
        text_id: 0x10d,
        idx: 0x39,
    },
    soundtest_data {
        text_id: 0x10e,
        idx: 0x3a,
    },
    soundtest_data {
        text_id: 0x10f,
        idx: 0x3b,
    },
    soundtest_data {
        text_id: 0x110,
        idx: 0x3c,
    },
    soundtest_data {
        text_id: 0x111,
        idx: 0x3d,
    },
    soundtest_data {
        text_id: 0x112,
        idx: 0x3e,
    },
    soundtest_data {
        text_id: 0x113,
        idx: 0x40,
    },
    soundtest_data {
        text_id: 0x114,
        idx: 0x41,
    },
    soundtest_data {
        text_id: 0x115,
        idx: 0x42,
    },
    soundtest_data {
        text_id: 0x116,
        idx: 0x4a,
    },
    soundtest_data {
        text_id: 0x117,
        idx: 0x4b,
    },
    soundtest_data {
        text_id: 0x118,
        idx: 0x4c,
    },
    soundtest_data {
        text_id: 0x119,
        idx: 0x54,
    },
    soundtest_data {
        text_id: 0x11a,
        idx: 0x5f,
    },
    soundtest_data {
        text_id: 0x11b,
        idx: 0x60,
    },
    soundtest_data {
        text_id: 0x11c,
        idx: 0x61,
    },
    soundtest_data {
        text_id: 0x11d,
        idx: 0x36,
    },
    soundtest_data {
        text_id: 0x11e,
        idx: 0,
    },
    soundtest_data {
        text_id: 0x11f,
        idx: 0x37,
    },
    soundtest_data {
        text_id: 0x120,
        idx: 0x50,
    },
    soundtest_data {
        text_id: 0x121,
        idx: 0x53,
    },
    soundtest_data {
        text_id: 0x122,
        idx: 0x1e,
    },
    soundtest_data {
        text_id: 0x123,
        idx: 0x3f,
    },
    soundtest_data {
        text_id: 0x124,
        idx: 0x4d,
    },
    soundtest_data {
        text_id: 0x125,
        idx: 0x7,
    },
    soundtest_data {
        text_id: 0x126,
        idx: 0x2,
    },
    soundtest_data {
        text_id: 0x127,
        idx: 0xf,
    },
    soundtest_data {
        text_id: 0x128,
        idx: 0x51,
    },
    soundtest_data {
        text_id: 0x129,
        idx: 0x4e,
    },
    soundtest_data {
        text_id: 0x12a,
        idx: 0x8,
    },
    soundtest_data {
        text_id: 0x12b,
        idx: 0x26,
    },
    soundtest_data {
        text_id: 0x12c,
        idx: 0x27,
    },
    soundtest_data {
        text_id: 0x12f,
        idx: 0x5d,
    },
    soundtest_data {
        text_id: 0x130,
        idx: 0x5e,
    },
    soundtest_data {
        text_id: 0x132,
        idx: 0x46,
    },
    soundtest_data {
        text_id: 0x133,
        idx: 0x47,
    },
    soundtest_data {
        text_id: 0x134,
        idx: 0x48,
    },
    soundtest_data {
        text_id: 0x135,
        idx: 0x43,
    },
    soundtest_data {
        text_id: 0x136,
        idx: 0x44,
    },
    soundtest_data {
        text_id: 0x137,
        idx: 0x45,
    },
    soundtest_data {
        text_id: 0x138,
        idx: 0x2f,
    },
    soundtest_data {
        text_id: 0x139,
        idx: 0x30,
    },
    soundtest_data {
        text_id: 0x13d,
        idx: 0x25,
    },
];
pub static mut data_3: [u8; 30] = [
    0 as i32 as u8,
    1 as i32 as u8,
    2 as i32 as u8,
    3 as i32 as u8,
    4 as i32 as u8,
    5 as i32 as u8,
    6 as i32 as u8,
    7 as i32 as u8,
    8 as i32 as u8,
    9 as i32 as u8,
    10 as i32 as u8,
    11 as i32 as u8,
    12 as i32 as u8,
    13 as i32 as u8,
    14 as i32 as u8,
    15 as i32 as u8,
    16 as i32 as u8,
    17 as i32 as u8,
    18 as i32 as u8,
    19 as i32 as u8,
    20 as i32 as u8,
    21 as i32 as u8,
    22 as i32 as u8,
    23 as i32 as u8,
    24 as i32 as u8,
    25 as i32 as u8,
    26 as i32 as u8,
    27 as i32 as u8,
    28 as i32 as u8,
    29 as i32 as u8,
];
pub static mut data_4: [i32; 60] = [
    25,
    8,
    1,
    6,
    16,
    17,
    4,
    2,
    13,
    0,
    11,
    5,
    12,
    14,
    18,
    7,
    15,
    10,
    9,
    3,
    21,
    20,
    24,
    19,
    23,
    22,
    29,
    28,
    26,
    27,
    6,
    8,
    11,
    18,
    14,
    15,
    16,
    17,
    18,
    19,
    20,
    21,
    22,
    24,
    13,
    25,
    26,
    28,
    27,
    10,
    7,
    9,
    31,
    23,
    30,
    4,
    0,
    39,
    3,
    32,
];
static mut mnSoundTest_803EF4F4: [c_char; 22] = unsafe {
    core::mem::transmute::<
        [u8; 22],
        [c_char; 22],
    >(*b"Can't get user_data.\n\0")
};
static mut mnSoundTest_803EF50C: [c_char; 14] = unsafe {
    core::mem::transmute::<[u8; 14], [c_char; 14]>(*b"mnsoundtest.c\0")
};
static mut mnSoundTest_803EF51C: [c_char; 10] = unsafe {
    core::mem::transmute::<[u8; 10], [c_char; 10]>(*b"user_data\0")
};
static mut mnSoundTest_803EF528: [c_char; 23] = unsafe {
    core::mem::transmute::<
        [u8; 23],
        [c_char; 23],
    >(*b"MenMainConTs_Top_joint\0")
};
static mut mnSoundTest_803EF540: [c_char; 27] = unsafe {
    core::mem::transmute::<
        [u8; 27],
        [c_char; 27],
    >(*b"MenMainConTs_Top_animjoint\0")
};
static mut mnSoundTest_803EF55C: [c_char; 31] = unsafe {
    core::mem::transmute::<
        [u8; 31],
        [c_char; 31],
    >(*b"MenMainConTs_Top_matanim_joint\0")
};
static mut mnSoundTest_803EF57C: [c_char; 33] = unsafe {
    core::mem::transmute::<
        [u8; 33],
        [c_char; 33],
    >(*b"MenMainConTs_Top_shapeanim_joint\0")
};
pub unsafe extern "C" fn mnSoundTest_8024A790(mut arg0: *mut mnSoundTest_GObj) {
    let mut temp_f31: f32 = 0.;
    let mut temp_f30: f32 = 0.;
    let mut temp_r31: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut temp_r30: i32 = 0;
    let mut temp_r30_2: i32 = 0;
    let mut temp_r3: i32 = 0;
    let mut temp_r3_2: i32 = 0;
    temp_r31 = (*arg0).user_data as *mut soundtest_user_data;
    temp_f30 = 1.0_f32 as f32;
    temp_f31 = temp_f30
        * (1.0_f32 - (*(&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus)).nml_analogL);
    temp_f30
        *= (1.0_f32 - (*(&raw mut HSD_PadCopyStatus as *mut HSD_PadStatus)).nml_analogR)
            as f32;
    temp_f31
        *= (1.0_f32 - HSD_PadCopyStatus[1 as i32 as usize].nml_analogL)
            as f32;
    temp_f30
        *= (1.0_f32 - HSD_PadCopyStatus[1 as i32 as usize].nml_analogR)
            as f32;
    temp_f31
        *= (1.0_f32 - HSD_PadCopyStatus[2 as i32 as usize].nml_analogL)
            as f32;
    temp_f30
        *= (1.0_f32 - HSD_PadCopyStatus[2 as i32 as usize].nml_analogR)
            as f32;
    temp_f31
        *= (1.0_f32 - HSD_PadCopyStatus[3 as i32 as usize].nml_analogL)
            as f32;
    temp_f30
        *= (1.0_f32 - HSD_PadCopyStatus[3 as i32 as usize].nml_analogR)
            as f32;
    if temp_f31 < 0.9_f32 || (*temp_r31).unk8 < 1.0_f32 {
        temp_r30 = ((*temp_r31).unk8
            * (temp_f31 * gm_801601C4(gmMainLib_8015ED74() as i8) as f32)) as i32;
        lbAudioAx_80024614(temp_r30 as i32);
        mnSoundTest_804D6C48 = temp_r30;
    } else {
        temp_r3 = gm_801601C4(gmMainLib_8015ED74() as i8) as i32;
        if temp_r3 != mnSoundTest_804D6C48 {
            lbAudioAx_80024614(temp_r3 as i32);
            mnSoundTest_804D6C48 = temp_r3;
        }
    }
    if temp_f30 < 0.9_f32 || (*temp_r31).unkC < 1.0_f32 {
        temp_r30_2 = ((*temp_r31).unkC
            * (temp_f30 * gm_80160244(gmMainLib_8015ED74() as i8) as f32)) as i32;
        lbAudioAx_800245F4(temp_r30_2 as i32);
        mnSoundTest_804D6C4C = temp_r30_2;
        return;
    }
    temp_r3_2 = gm_80160244(gmMainLib_8015ED74() as i8) as i32;
    if temp_r3_2 != mnSoundTest_804D6C4C {
        lbAudioAx_800245F4(temp_r3_2 as i32);
        mnSoundTest_804D6C4C = temp_r3_2;
    }
}
pub unsafe extern "C" fn mnSoundTest_8024A958(mut arg0: *mut mnSoundTest_GObj) {
    let mut unused1: f32 = 0.;
    let mut unused2: f32 = 0.;
    let mut new_var: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut unused3: f32 = 0.;
    let mut string: [c_char; 3] = [0; 3];
    let mut temp_f1: f32 = 0.;
    let mut temp_r3: *mut HSD_Text = null_mut();
    let mut temp_r3_2: *mut HSD_Text = null_mut();
    let mut temp_r3_3: *mut HSD_Text = null_mut();
    let mut temp_r3_4: *mut HSD_Text = null_mut();
    let mut temp_r30: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    temp_r30 = (*arg0).user_data as *mut soundtest_user_data;
    new_var = temp_r30;
    temp_r3 = (*new_var).unk14;
    if !temp_r3.is_null() {
        HSD_SisLib_803A5CC4(temp_r3);
    }
    temp_r3_2 = (*new_var).unk18;
    if !temp_r3_2.is_null() {
        HSD_SisLib_803A5CC4((*new_var).unk18);
    }
    temp_r3_3 = HSD_SisLib_803A5ACC(
        0,
        0,
        vec_6.x,
        vec_6.y,
        18.0_f32,
        350.0_f32,
        10.0_f32,
    );
    (*new_var).unk14 = temp_r3_3;
    (*temp_r3_3).font_size.x = 0.035_f32 as f32;
    (*temp_r3_3).font_size.y = 0.035_f32 as f32;
    (*temp_r3_3).default_alignment = 2;
    HSD_SisLib_803A6368(
        temp_r3_3,
        data_2[text_ids[(*new_var).unk1 as usize] as usize].text_id as i32,
    );
    temp_r3_4 = HSD_SisLib_803A6754(0, 0);
    (*new_var).unk18 = temp_r3_4;
    temp_f1 = vec_7.y;
    (*temp_r3_4).pos_x = vec_7.x;
    (*temp_r3_4).pos_y = temp_f1;
    (*temp_r3_4).pos_z = 18.0_f32 as f32;
    (*temp_r3_4).font_size.x = 0.04_f32 as f32;
    (*temp_r3_4).font_size.y = 0.04_f32 as f32;
    mn_8022EA08(&raw mut string as *mut c_char, (*new_var).unk1 as u32);
    (*temp_r3_4).default_alignment = 1;
    HSD_SisLib_803A6B98(
        temp_r3_4,
        0.0_f32,
        0.0_f32,
        &raw mut string as *mut c_char,
        &[],
    );
}
pub unsafe extern "C" fn mnSoundTest_8024AA70(
    mut arg0: *mut mnSoundTest_GObj,
    mut arg1: u8,
) {
    let mut sp38: *mut HSD_JObj = null_mut();
    let mut sp34: *mut HSD_JObj = null_mut();
    let mut pad1: u32 = 0;
    let mut pad2: u32 = 0;
    let mut temp_r31: *mut HSD_JObj = null_mut();
    let mut string: [c_char; 3] = [0; 3];
    let mut temp_r3: *mut HSD_Text = null_mut();
    let mut temp_r3_2: *mut HSD_Text = null_mut();
    let mut temp_r3_4: *mut HSD_Text = null_mut();
    let mut temp_f1: f32 = 0.;
    let mut pad5: u32 = 0;
    let mut new_var: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut temp_r30: *mut c_void = core::ptr::null_mut::<
        c_void,
    >();
    let mut temp_r30_2: *mut c_void = core::ptr::null_mut::<
        c_void,
    >();
    let mut jobj: *mut HSD_JObj = HSD_GObjGetHSDObj(arg0 as *mut HSD_GObj)
        as *mut HSD_JObj;
    let mut user_data: *mut soundtest_user_data = (*arg0).user_data
        as *mut soundtest_user_data;
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp38, &[0x13, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp34, &[0x14, -1]);
    if arg1 as i32 == 2 {
        let mut temp_r30_2_0: *mut soundtest_user_data = (*arg0).user_data
            as *mut soundtest_user_data;
        new_var = temp_r30_2_0;
        temp_r3 = (*temp_r30_2_0).unk1C;
        if !temp_r3.is_null() {
            HSD_SisLib_803A5CC4((*temp_r30_2_0).unk1C);
        }
        temp_r3_2 = HSD_SisLib_803A6754(
            0,
            0,
        );
        temp_r3_4 = temp_r3_2;
        (*new_var).unk1C = temp_r3_4;
        temp_f1 = vec_8.y;
        (*temp_r3_2).pos_x = vec_8.x;
        (*temp_r3_2).pos_y = temp_f1;
        (*temp_r3_2).pos_z = 18.0_f32 as f32;
        (*temp_r3_2).font_size.x = 0.04_f32 as f32;
        (*temp_r3_2).font_size.y = 0.04_f32 as f32;
        mn_8022EA08(
            &raw mut string as *mut c_char,
            (*new_var).unk4 as u32,
        );
        (*temp_r3_2).default_alignment = 1;
        HSD_SisLib_803A6B98(
            temp_r3_2,
            0.0_f32,
            0.0_f32,
            &raw mut string as *mut c_char,
            &[],
        );
        HSD_JObjClearFlagsAll(sp38, JOBJ_HIDDEN as u32);
    } else {
        temp_r3_4 = (*user_data).unk1C;
        if !temp_r3_4.is_null() {
            HSD_SisLib_803A5CC4(temp_r3_4);
            (*user_data).unk1C = null_mut();
        }
        HSD_JObjSetFlagsAll(sp38, JOBJ_HIDDEN as u32);
    }
    HSD_JObjReqAnimAll(
        sp34,
        (arg1 as i32 == 1) as i32
            as f32,
    );
    mn_8022F3D8(sp34 as *mut HSD_JObj, 0xff, 1056 as HSD_TypeMask);
    HSD_JObjAnimAll(sp34);
}
pub unsafe extern "C" fn mnSoundTest_8024ABF8(
    mut arg0: *mut mnSoundTest_GObj,
    mut arg1: u8,
) {
    let mut sp1C: *mut HSD_JObj = null_mut();
    let mut sp18: *mut HSD_JObj = null_mut();
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut temp_r31: *mut HSD_JObj = null_mut();
    let mut temp_f31: f32 = 0.;
    let mut temp_f30: f32 = 0.;
    temp_r31 = HSD_GObjGetHSDObj(arg0 as *mut HSD_GObj) as *mut HSD_JObj;
    lb_80011E24(temp_r31 as *mut HSD_JObj, &raw mut sp18, &[2, -1]);
    lb_80011E24(temp_r31 as *mut HSD_JObj, &raw mut sp1C, &[5, -1]);
    lb_80011E24(temp_r31 as *mut HSD_JObj, &raw mut sp14, &[4, -1]);
    temp_f31 = mn_8022F298(sp18 as *mut HSD_JObj);
    temp_f30 = mn_8022F298(sp14 as *mut HSD_JObj);
    HSD_JObjReqAnimAll(sp18, arg1 as f32);
    mn_8022F3D8(sp18 as *mut HSD_JObj, 0xff, 1056 as HSD_TypeMask);
    HSD_JObjAnimAll(sp18);
    HSD_JObjReqAnim(sp18, temp_f31);
    HSD_JObjAnim(sp18);
    HSD_JObjReqAnimAll(sp14, temp_f30);
    mn_8022F3D8(sp14 as *mut HSD_JObj, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(sp14);
    if arg1 as i32 != 0 {
        HSD_JObjReqAnimAll(sp1C, vec_2.start_frame);
    } else {
        HSD_JObjReqAnimAll(sp1C, vec_3.end_frame);
    }
    mn_8022F3D8(sp1C as *mut HSD_JObj, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(sp1C);
}
pub unsafe extern "C" fn mnSoundTest_8024AD58(
    mut arg0: *mut mnSoundTest_GObj,
    mut arg1: u8,
) {
    let mut sp1C: *mut HSD_JObj = null_mut();
    let mut sp18: *mut HSD_JObj = null_mut();
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut temp_r30: *mut HSD_JObj = null_mut();
    let mut temp_f31: f32 = 0.;
    let mut temp_r0: u8 = 0;
    temp_r30 = HSD_GObjGetHSDObj(arg0 as *mut HSD_GObj) as *mut HSD_JObj;
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp18, &[0xb, -1]);
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp1C, &[0xe, -1]);
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp14, &[0xd, -1]);
    temp_r0 = (arg1 != 0) as i32 as u8;
    temp_f31 = mn_8022F298(sp18 as *mut HSD_JObj);
    HSD_JObjReqAnimAll(sp18, temp_r0 as f32);
    mn_8022F3D8(sp18 as *mut HSD_JObj, 0xff, 1056 as HSD_TypeMask);
    HSD_JObjAnimAll(sp18);
    HSD_JObjReqAnimAll(sp18, temp_f31);
    HSD_JObjAnim(sp18);
    HSD_JObjReqAnimAll(sp14, vec_5.end_frame);
    mn_8022F3D8(sp14 as *mut HSD_JObj, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(sp14);
    if temp_r0 as i32 != 0 {
        HSD_JObjReqAnimAll(sp1C, vec_2.start_frame);
    } else {
        HSD_JObjReqAnimAll(sp1C, vec_3.end_frame);
    }
    mn_8022F3D8(sp1C as *mut HSD_JObj, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(sp1C);
    mnSoundTest_8024AA70(arg0, arg1);
}
unsafe extern "C" fn fn_8024AED0_inline(
    mut arg0: *mut mnSoundTest_GObj,
    mut string: *mut c_char,
) {
    let mut text: *mut HSD_Text = null_mut();
    let mut pos_y: f32 = 0.;
    let mut user_data: *mut soundtest_user_data = (*mnSoundTest_804D6C40).user_data
        as *mut soundtest_user_data;
    let mut old_text: *mut HSD_Text = (*user_data).unk1C;
    if !old_text.is_null() {
        HSD_SisLib_803A5CC4(old_text);
    }
    text = HSD_SisLib_803A6754(0, 0);
    (*user_data).unk1C = text;
    pos_y = vec_8.y;
    (*text).pos_x = vec_8.x;
    (*text).pos_y = pos_y;
    (*text).pos_z = 18.0_f32 as f32;
    (*text).font_size.x = 0.04_f32 as f32;
    (*text).font_size.y = 0.04_f32 as f32;
    mn_8022EA08(string, (*user_data).unk4 as u32);
    (*text).default_alignment = 1;
    HSD_SisLib_803A6B98(text, 0.0_f32, 0.0_f32, string, &[]);
}
unsafe extern "C" fn fn_8024AED0_GetUserData() -> *mut soundtest_user_data {
    return (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
}
pub unsafe extern "C" fn fn_8024AED0(mut arg0: *mut mnSoundTest_GObj) {
    let mut operand_pad: [u8; 24] = [0; 24];
    let mut sp7C: *mut HSD_JObj = null_mut();
    let mut array_pad: [u8; 4] = [0; 4];
    let mut string1: [c_char; 20] = [0; 20];
    let mut string2: [c_char; 20] = [0; 20];
    let mut low_pad: [u8; 4] = [0; 4];
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut sp50: *mut c_void = core::ptr::null_mut::<
        c_void,
    >();
    let mut temp_r3_2: *mut HSD_GObjProc = null_mut();
    let mut temp_r3_7: *mut HSD_Text = null_mut();
    let mut temp_r3_8: *mut HSD_Text = null_mut();
    let mut fade_timer: f32 = 0.;
    let mut preview_timer: f32 = 0.;
    let mut audio_id: i32 = 0;
    let mut selection: u8 = 0;
    let mut sound_idx: u16 = 0;
    let mut sound_kind: i32 = 0;
    let mut temp_r3_3: i32 = 0;
    let mut temp_r5: i32 = 0;
    let mut temp_r4: u16 = 0;
    let mut menu_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut reset_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut temp_r28_5: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut input_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut events: u64 = 0;
    let mut user_data: *mut soundtest_user_data = (*mnSoundTest_804D6C40).user_data
        as *mut soundtest_user_data;
    if mn_804D6BC8.cooldown as i32 != 0 {
        Menu_DecrementAnimTimer();
        return;
    }
    events = Menu_GetAllInputs();
    if events & MenuInput_Back as i32 as u64 != 0 {
        lbAudioAx_80023694();
        (*user_data).unk0 = 1;
        mnSoundTest_8024AA70(mnSoundTest_804D6C40, (*user_data).unk0);
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        temp_r3_2 = HSD_GObj_SetupProc(
            arg0 as *mut HSD_GObj,
            Some(fn_8024B2B0),
            0,
        );
        (*temp_r3_2).set_flags_3(HSD_GObj_804D783C as u8);
        return;
    }
    menu_data = (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
    fade_timer = (*menu_data).unk8;
    if fade_timer < 1.0_f32 {
        (*menu_data).unk8 = fade_timer as f32 - 0.02_f32;
        if (*menu_data).unk8 <= 0.0_f32 {
            (*menu_data).unk8 = 1.0_f32 as f32;
            reset_data = fn_8024AED0_GetUserData();
            lbAudioAx_800236DC();
            (*reset_data).unk2 = 0x50;
        }
    }
    preview_timer = (*menu_data).unkC;
    if preview_timer < 1.0_f32 {
        (*menu_data).unkC = preview_timer as f32 - 0.02_f32;
        if (*menu_data).unkC <= 0.0_f32 {
            (*menu_data).unkC = 1.0_f32 as f32;
            lbAudioAx_80023694();
        }
    }
    if mnSoundTest_804D6C44 as i32 == 0 {
        temp_r3_3 = lbAudioAx_80023968(
            data_3[(*user_data).unk3 as usize] as i32,
        ) as i32;
        temp_r5 = (0 & 0) as i32;
        if events & MenuInput_AButton as i32 as u64 != 0 {
            sound_idx = (*user_data).unk4;
            selection = data_3[(*user_data).unk3 as usize];
            input_data = fn_8024AED0_GetUserData();
            jobj = (*mnSoundTest_804D6C40).hsd_obj as *mut HSD_JObj;
            (*input_data).unkC = 1.0_f32 as f32;
            sound_kind = gm_80160244(gmMainLib_8015ED74() as i8) as i32;
            lbAudioAx_800245F4(sound_kind as i32);
            mnSoundTest_804D6C4C = sound_kind;
            audio_id = lbAudioAx_80023A44(
                selection as i32,
                sound_idx as i32,
            ) as i32;
            lbAudioAx_80023694();
            lbAudioAx_80023B24(audio_id as i32);
            lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp7C, &[0xd, -1]);
            HSD_JObjReqAnimAll(sp7C, vec_5.start_frame);
            mn_8022F3D8(sp7C as *mut HSD_JObj, 0xff, MOBJ_MASK);
            HSD_JObjAnimAll(sp7C);
        } else if events & MenuInput_StartButton as i32 as u64 != 0 {
            if (*user_data).unkC == 1.0_f32 {
                (*user_data).unkC = 0.98_f32 as f32;
            }
        } else if events & MenuInput_Left as i32 as u64 != 0 {
            temp_r4 = (*user_data).unk4;
            if temp_r4 as i32 != 0 {
                (*user_data).unk4 = (temp_r4 as i32
                    - 1) as u16;
            } else {
                (*user_data).unk4 = (temp_r3_3 as i32
                    - 1) as u16;
            }
            fn_8024AED0_inline(arg0, &raw mut string1 as *mut c_char);
        } else if events & MenuInput_Right as i32 as u64 != 0 {
            (*user_data).unk4 = ((*user_data).unk4 as i32
                + 1) as u16;
            if (*user_data).unk4 as i32 >= temp_r3_3 {
                (*user_data).unk4 = 0;
            }
            fn_8024AED0_inline(arg0, &raw mut string2 as *mut c_char);
        }
        mnSoundTest_8024A790(mnSoundTest_804D6C40);
    }
}
unsafe extern "C" fn mnSoundTest_GetInputs() -> u64 {
    return mn_80229624(4) as u64;
}
unsafe extern "C" fn mnSoundTest_GetAudioVolume() -> i32 {
    return gm_801601C4(gmMainLib_8015ED74() as i8) as i32;
}
unsafe extern "C" fn mnSoundTest_ToggleView(mut ud: *mut soundtest_user_data) {
    let mut state: i32 = 0;
    let mut gobj: *mut mnSoundTest_GObj = null_mut();
    state = (if (*ud).unk0 as i32 == 0 {
        1
    } else {
        0
    }) as i32;
    (*ud).unk0 = state as u8;
    ud = (*ud).unk0 as u32 as *mut soundtest_user_data;
    gobj = mnSoundTest_804D6C40;
    mnSoundTest_8024ABF8(
        gobj,
        (ud as u32 == 0) as i32 as u8,
    );
    mnSoundTest_8024AD58(gobj, ud as u32 as u8);
}
unsafe extern "C" fn mnSoundTest_PlaySampleAnim(
    mut jobj: *mut HSD_JObj,
    mut pj: *mut *mut HSD_JObj,
    mut p: *const u8,
) {
    let mut sound_kind: i32 = 0;
    lb_80011E24(jobj as *mut HSD_JObj, pj as *mut *mut HSD_JObj, &[4, -1]);
    HSD_JObjReqAnimAll(*pj, mn_8022F298(*pj));
    mn_8022F3D8(*pj, 0xff, MOBJ_MASK);
    HSD_JObjAnimAll(*pj);
    sound_kind = data_2[*p as usize].idx as i32;
    if lbAudioAx_80023090(sound_kind as i32) != 0 {
        gmMainLib_8015ED68(sound_kind as isize);
    }
}
unsafe extern "C" fn mnSoundTest_UpdateCategoryAnim(
    mut ud: *mut soundtest_user_data,
    mut pj: *mut *mut HSD_JObj,
) {
    let mut selection: u8 = 0;
    (*ud).unk4 = 0;
    selection = data_3[(*ud).unk3 as usize];
    lb_80011E24(HSD_GObjGetHSDObj(mnSoundTest_804D6C40 as *mut HSD_GObj) as *mut HSD_JObj, pj as *mut *mut HSD_JObj, &[0x15, -1]);
    HSD_JObjReqAnimAll(*pj, data_4[selection as usize] as f32);
    mn_8022F3D8(*pj, 0xff, 160 as HSD_TypeMask);
    HSD_JObjAnimAll(*pj);
}
pub unsafe extern "C" fn fn_8024B2B0(mut arg0: *mut mnSoundTest_GObj) {
    let mut high_pad: [u8; 40] = [0; 40];
    let mut sp54: *mut HSD_JObj = null_mut();
    let mut stack_pad1: [u8; 8] = [0; 8];
    let mut sp48: *mut HSD_JObj = null_mut();
    let mut stack_pad2: [u8; 8] = [0; 8];
    let mut sp3C: *mut HSD_JObj = null_mut();
    let mut md2: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut sound_id: i32 = 0;
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut inputs: u64 = 0;
    let mut p: *mut u8 = null_mut();
    let mut user_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut anim_idx: i32 = 0;
    let mut audio_id: i32 = 0;
    let mut sound_kind: i32 = 0;
    let mut reset_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut menu_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    user_data = (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
    if mn_804D6BC8.cooldown as i32 != 0 {
        mn_804D6BC8.cooldown = (mn_804D6BC8.cooldown as i32
            - 1) as u16;
        mn_804D6BC8.x2 = 0;
        mn_804D6BC8.x4 = 0 as i32;
        return;
    }
    inputs = mnSoundTest_GetInputs();
    mn_804A04F0.buttons = inputs;
    if inputs & MenuInput_Back as i32 as u64 != 0 {
        mn_804A04F0.entering_menu = 0;
        mn_80229894(5, 2, 3);
        lbAudioAx_80023694();
        gm_801603B0();
        lbAudioAx_80023F28(gmMainLib_8015ECB0());
        return;
    }
    if inputs
        & (MenuInput_Up as i32 | MenuInput_Down as i32)
            as u64 != 0
    {
        mnSoundTest_ToggleView(user_data);
        return;
    }
    menu_data = (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
    if (*menu_data).unk8 < 1.0_f32 {
        (*menu_data).unk8 -= 0.02_f32;
        if (*menu_data).unk8 <= 0.0_f32 {
            (*menu_data).unk8 = 1.0_f32 as f32;
            reset_data = fn_8024AED0_GetUserData();
            lbAudioAx_800236DC();
            (*reset_data).unk2 = 0x50;
        }
    }
    if (*menu_data).unkC < 1.0_f32 {
        (*menu_data).unkC -= 0.02_f32;
        if (*menu_data).unkC <= 0.0_f32 {
            (*menu_data).unkC = 1.0_f32 as f32;
            lbAudioAx_80023694();
        }
    }
    if mnSoundTest_804D6C44 as i32 != 0 {
        return;
    }
    if (*user_data).unk0 as i32 == 0 {
        if inputs & MenuInput_AButton as i32 as u64 != 0 {
            sound_id = (*user_data).unk1 as i32;
            if (*user_data).unk2 as u32 == sound_id as u32 {
                md2 = (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
                lbAudioAx_800236DC();
                (*md2).unk2 = 0x50;
            } else {
                user_data = fn_8024AED0_GetUserData();
                arg0 = HSD_GObjGetHSDObj(mnSoundTest_804D6C40 as *mut HSD_GObj)
                    as *mut HSD_JObj as *mut mnSoundTest_GObj;
                (*user_data).unk8 = 1.0_f32 as f32;
                audio_id = mnSoundTest_GetAudioVolume() as i32;
                lbAudioAx_80024614(audio_id);
                mnSoundTest_804D6C48 = audio_id as i32;
                p = (&raw mut text_ids as *mut u8).offset(sound_id as isize)
                    as *mut u8;
                lbAudioAx_80023F28(data_2[*p as usize].idx as i32);
                (*user_data).unk2 = sound_id as u8;
                mnSoundTest_PlaySampleAnim(arg0 as *mut HSD_JObj, &raw mut sp54, p);
            }
        } else if inputs & MenuInput_StartButton as i32 as u64 != 0 {
            if (*user_data).unk8 == 1.0_f32 {
                (*user_data).unk8 = 0.98_f32 as f32;
            }
        } else if inputs & MenuInput_Left as i32 as u64 != 0 {
            if (*user_data).unk1 as i32 != 0 {
                (*user_data).unk1 = ((*user_data).unk1 as i32
                    - 1) as u8;
            } else {
                (*user_data).unk1 = 0x4f;
            }
            mnSoundTest_8024A958(mnSoundTest_804D6C40);
        } else if inputs & MenuInput_Right as i32 as u64 != 0 {
            (*user_data).unk1 = ((*user_data).unk1 as i32
                + 1) as u8;
            if (*user_data).unk1 as u32 >= 0x50 {
                (*user_data).unk1 = 0;
            }
            mnSoundTest_8024A958(mnSoundTest_804D6C40);
        }
    } else {
        if inputs & MenuInput_AButton as i32 as u64 != 0 {
            (*user_data).unk0 = 2;
            mnSoundTest_8024AA70(mnSoundTest_804D6C40, (*user_data).unk0);
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            proc_0 = HSD_GObj_SetupProc(
                arg0 as *mut HSD_GObj,
                Some(fn_8024AED0),
                0,
            );
            (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
            return;
        }
        if inputs & MenuInput_Left as i32 as u64 != 0 {
            if (*user_data).unk3 as i32 != 0 {
                (*user_data).unk3 = ((*user_data).unk3 as i32
                    - 1) as u8;
            } else {
                (*user_data).unk3 = 0x1d;
            }
            mnSoundTest_UpdateCategoryAnim(user_data, &raw mut sp48);
        } else if inputs & MenuInput_Right as i32 as u64 != 0 {
            (*user_data).unk3 = ((*user_data).unk3 as i32
                + 1) as u8;
            if (*user_data).unk3 as i32 >= 0x1e {
                (*user_data).unk3 = 0;
            }
            mnSoundTest_UpdateCategoryAnim(user_data, &raw mut sp3C);
        }
    }
    mnSoundTest_8024A790(mnSoundTest_804D6C40);
}
pub unsafe extern "C" fn fn_8024B7E4(mut arg0: *mut mnSoundTest_GObj) {
    let mut pad2: u32 = 0;
    let mut sp20: *mut HSD_JObj = null_mut();
    let mut sp1C: *mut HSD_JObj = null_mut();
    let mut sp18: *mut HSD_JObj = null_mut();
    let mut temp_r31: *mut AnimLoopSettings = core::ptr::null_mut::<
        AnimLoopSettings,
    >();
    let mut temp_r30: *mut HSD_JObj = null_mut();
    let mut temp_f31: f32 = 0.;
    let mut pad1: u32 = 0;
    temp_r30 = HSD_GObjGetHSDObj(arg0 as *mut HSD_GObj) as *mut HSD_JObj;
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp20, &[0xb, -1]);
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp1C, &[2, -1]);
    lb_80011E24(temp_r30 as *mut HSD_JObj, &raw mut sp18, &[1, -1]);
    temp_f31 = mn_8022EFD8(sp20 as *mut HSD_JObj, &raw mut vec_1) as f32;
    mn_8022EFD8(sp1C as *mut HSD_JObj, &raw mut vec_1);
    mn_8022EFD8(sp18 as *mut HSD_JObj, &raw mut vec_1);
    if temp_f31 == vec_1.end_frame {
        HSD_GObjFree(arg0 as *mut HSD_GObj);
    }
}
pub unsafe extern "C" fn fn_8024B8B4(mut arg0: *mut mnSoundTest_GObj, mut farg0: f32) {
    let mut var_f1: f32 = 0.;
    let mut sp20: *mut HSD_JObj = null_mut();
    let mut sp1C: *mut HSD_JObj = null_mut();
    let mut new_var: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut sp18: *mut HSD_JObj = null_mut();
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut sp10: *mut HSD_JObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut text: *mut HSD_Text = null_mut();
    var_f1 = farg0;
    user_data = (*arg0).user_data as *mut soundtest_user_data;
    jobj = (*arg0).hsd_obj as *mut HSD_JObj;
    if mn_804A04F0.cur_menu as i32 != 0x1b {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            arg0 as *mut HSD_GObj,
            Some(fn_8024B7E4),
            0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
        user_data = (*arg0).user_data as *mut soundtest_user_data;
        new_var = user_data;
        text = (*new_var).unk10;
        if !text.is_null() {
            HSD_SisLib_803A5CC4(text);
            (*user_data).unk10 = null_mut();
        }
        text = (*user_data).unk14;
        if !text.is_null() {
            HSD_SisLib_803A5CC4(text);
            (*user_data).unk14 = null_mut();
        }
        text = (*user_data).unk18;
        if !text.is_null() {
            HSD_SisLib_803A5CC4(text);
            (*user_data).unk18 = null_mut();
        }
        text = (*user_data).unk1C;
        if !text.is_null() {
            HSD_SisLib_803A5CC4(text);
            (*user_data).unk1C = null_mut();
        }
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp1C, &[0xb, -1]);
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp18, &[2, -1]);
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp14, &[1, -1]);
        HSD_JObjReqAnim(sp1C, vec_1.start_frame);
        HSD_JObjAnim(sp1C);
        HSD_JObjReqAnim(sp18, vec_1.start_frame);
        HSD_JObjAnim(sp18);
        HSD_JObjReqAnim(sp14, vec_1.start_frame);
        HSD_JObjAnim(sp14);
        return;
    }
    if (*user_data).unk0 as i32 == 0 {
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp20, &[5, -1]);
        var_f1 = mn_8022EC18(sp20 as *mut HSD_JObj, &raw mut vec_2, MOBJ_MASK) as f32;
    }
    if (*user_data).unk0 as i32 != 0 {
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp20, &[0xe, -1]);
        var_f1 = mn_8022EC18(sp20 as *mut HSD_JObj, &raw mut vec_2, MOBJ_MASK) as f32;
    }
    if (*user_data).unk2 as i32 != 0x50 {
        lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp10, &[4, -1]);
        var_f1 = mn_8022EC18(sp10 as *mut HSD_JObj, &raw mut vec_4, MOBJ_MASK) as f32;
    }
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp10, &[0xd, -1]);
    mn_8022EC18(sp10 as *mut HSD_JObj, &raw mut vec_5, MOBJ_MASK);
}
pub unsafe extern "C" fn fn_8024BAF0(mut arg0: *mut mnSoundTest_GObj) {
    let mut pad2: u32 = 0;
    let mut sp1C: *mut HSD_JObj = null_mut();
    let mut sp18: *mut HSD_JObj = null_mut();
    let mut sp14: *mut HSD_JObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut proc2: *mut HSD_GObjProc = null_mut();
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut user_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut temp_f31: f32 = 0.;
    let mut pad1: u32 = 0;
    jobj = HSD_GObjGetHSDObj(arg0 as *mut HSD_GObj) as *mut HSD_JObj;
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp1C, &[0xb, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp18, &[2, -1]);
    lb_80011E24(jobj as *mut HSD_JObj, &raw mut sp14, &[1, -1]);
    if mn_804A04F0.cur_menu as i32 != 0x1b {
        HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
        proc_0 = HSD_GObj_SetupProc(
            arg0 as *mut HSD_GObj,
            Some(fn_8024B7E4),
            0,
        );
        (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
        user_data = (*arg0).user_data as *mut soundtest_user_data;
        if !(*user_data).unk10.is_null() {
            HSD_SisLib_803A5CC4((*user_data).unk10);
            (*user_data).unk10 = null_mut();
        }
        if !(*user_data).unk14.is_null() {
            HSD_SisLib_803A5CC4((*user_data).unk14);
            (*user_data).unk14 = null_mut();
        }
        if !(*user_data).unk18.is_null() {
            HSD_SisLib_803A5CC4((*user_data).unk18);
            (*user_data).unk18 = null_mut();
        }
        if !(*user_data).unk1C.is_null() {
            HSD_SisLib_803A5CC4((*user_data).unk1C);
            (*user_data).unk1C = null_mut();
        }
    } else {
        temp_f31 = mn_8022EE84(sp1C as *mut HSD_JObj, &raw mut vec_0, MOBJ_MASK)
            as f32;
        mn_8022EE84(sp18 as *mut HSD_JObj, &raw mut vec_0, MOBJ_MASK);
        mn_8022EFD8(sp14 as *mut HSD_JObj, &raw mut vec_0);
        if temp_f31 == vec_0.end_frame {
            HSD_GObjProc_RemoveProc(HSD_GObj_CurrentInvokedProc);
            proc2 = HSD_GObj_SetupProc(
                arg0 as *mut HSD_GObj,
                core::mem::transmute::<
                    Event,
                    HSD_GObjEvent,
                >(
                    core::mem::transmute::<
                        Option<unsafe extern "C" fn(*mut mnSoundTest_GObj, f32) -> ()>,
                        Event,
                    >(
                        Some(
                            fn_8024B8B4
                               ,
                        ),
                    ),
                ),
                0,
            );
            (*proc2).set_flags_3(HSD_GObj_804D783C as u8);
            mnSoundTest_8024A958(arg0);
            mnSoundTest_804D6C44 = 0;
        }
    };
}
pub unsafe extern "C" fn mnSoundTest_8024BCA0(mut arg0: i32) {
    let mut jobj: *mut HSD_JObj = null_mut();
    let mut gobj: *mut mnSoundTest_GObj = null_mut();
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut category_jobj: *mut HSD_JObj = null_mut();
    let mut model_desc: *mut SoundTestModelDesc = core::ptr::null_mut::<
        SoundTestModelDesc,
    >();
    let mut user_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut text_user_data: *mut soundtest_user_data = core::ptr::null_mut::<
        soundtest_user_data,
    >();
    let mut pad2: u32 = 0;
    let mut text: *mut HSD_Text = null_mut();
    let mut selection: u8 = 0;
    model_desc = &raw mut mnSoundTest_804A08C8;
    gobj = GObj_Create(6, 7, 0x80) as *mut mnSoundTest_GObj;
    mnSoundTest_804D6C40 = gobj;
    jobj = HSD_JObjLoadJoint((*model_desc).joint as *mut HSD_Joint);
    HSD_GObjObject_80390A70(
        gobj as *mut HSD_GObj,
        HSD_GObj_JObjKind,
        jobj as *mut c_void,
    );
    GObj_SetupGXLink(
        gobj as *mut HSD_GObj,
        Some(
            HSD_GObj_JObjCallback
               ,
        ),
        4,
        0x80,
    );
    HSD_JObjAddAnimAll(
        jobj,
        (*model_desc).animjoint as *mut HSD_AnimJoint,
        (*model_desc).matanim_joint as *mut HSD_MatAnimJoint,
        (*model_desc).shapeanim_joint as *mut HSD_ShapeAnimJoint,
    );
    HSD_JObjReqAnimAll(jobj, 0.0_f32);
    HSD_JObjAnimAll(jobj);
    proc_0 = HSD_GObj_SetupProc(
        gobj as *mut HSD_GObj,
        core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut mnSoundTest_GObj) -> ()>,
            HSD_GObjEvent,
        >(Some(fn_8024BAF0)),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u8);
    user_data = HSD_MemAlloc(core::mem::size_of::<soundtest_user_data>() as isize)
        as *mut soundtest_user_data;
    if user_data.is_null() {
        OSReport(mnSoundTest_803EF4F4.as_ptr());
        __assert("mnsoundtest.c", 0x5f4, "user_data");
    }
    (*user_data).unk1 = 0;
    (*user_data).unk2 = 0x50;
    (*user_data).unk4 = 0;
    (*user_data).unk3 = 0;
    (*user_data).unk0 = 0;
    (*user_data).unk8 = 1.0_f32 as f32;
    (*user_data).unkC = 1.0_f32 as f32;
    (*user_data).unk10 = null_mut();
    (*user_data).unk14 = null_mut();
    (*user_data).unk18 = null_mut();
    (*user_data).unk1C = null_mut();
    GObj_InitUserData(
        gobj as *mut HSD_GObj,
        0,
        Some(HSD_Free),
        user_data as *mut c_void,
    );
    text_user_data = (*mnSoundTest_804D6C40).user_data as *mut soundtest_user_data;
    if !(*text_user_data).unk10.is_null() {
        HSD_SisLib_803A5CC4((*text_user_data).unk10);
    }
    text = HSD_SisLib_803A5ACC(
        0,
        1,
        -9.5_f32,
        9.1_f32,
        17.0_f32,
        364.68332_f32,
        38.38772_f32,
    );
    (*text_user_data).unk10 = text;
    (*text).font_size.x = 0.0521_f32 as f32;
    (*text).font_size.y = 0.0521_f32 as f32;
    HSD_SisLib_803A6368(text, 0xbe);
    selection = (*user_data).unk0;
    mnSoundTest_8024ABF8(
        gobj,
        (selection as i32 == 0)
            as i32 as u8,
    );
    mnSoundTest_8024AD58(gobj, selection);
    selection = data_3[(*user_data).unk3 as usize];
    lb_80011E24(HSD_GObjGetHSDObj(gobj as *mut HSD_GObj) as *mut HSD_JObj, &raw mut category_jobj, &[0x15, -1]);
    HSD_JObjReqAnimAll(category_jobj, data_4[selection as usize] as f32);
    mn_8022F3D8(category_jobj as *mut HSD_JObj, 0xff, 160 as HSD_TypeMask);
    HSD_JObjAnimAll(category_jobj);
}
pub unsafe extern "C" fn mnSoundTest_8024BEE0(mut arg0: i32) -> *mut HSD_GObjProc {
    let mut proc_0: *mut HSD_GObjProc = null_mut();
    let mut archive: *mut HSD_Archive = null_mut();
    let mut temp_ret: i32 = 0;
    mn_804D6BC8.cooldown = 5;
    mn_804A04F0.prev_menu = mn_804A04F0.cur_menu;
    mn_804A04F0.cur_menu = 0x1b as MenuKind8;
    mn_804A04F0.hovered_selection = 0;
    mnSoundTest_804D6C48 = gm_801601C4(gmMainLib_8015ED74() as i8) as i32;
    temp_ret = gm_80160244(gmMainLib_8015ED74() as i8) as i32;
    mnSoundTest_804D6C4C = temp_ret;
    archive = mn_804D6BB8;
    mnSoundTest_804D6C44 = 1;
    lbArchive_LoadSections(archive, &[(&raw mut mnSoundTest_804A08C8.joint as *mut *mut c_void, &raw mut mnSoundTest_803EF528 as *mut c_char), (&raw mut mnSoundTest_804A08C8.animjoint as *mut *mut c_void, &raw mut mnSoundTest_803EF540 as *mut c_char), (&raw mut mnSoundTest_804A08C8.matanim_joint as *mut *mut c_void, &raw mut mnSoundTest_803EF55C as *mut c_char), (&raw mut mnSoundTest_804A08C8.shapeanim_joint as *mut *mut c_void, &raw mut mnSoundTest_803EF57C as *mut c_char)]);
    mnSoundTest_8024BCA0(arg0 as i32);
    proc_0 = HSD_GObj_SetupProc(
        GObj_Create(0, 1, 0x80),
        Some(fn_8024B2B0),
        0,
    );
    (*proc_0).set_flags_3(HSD_GObj_804D783C as u16 as u8);
    return proc_0;
}
pub const NULL: i32 = 0;
