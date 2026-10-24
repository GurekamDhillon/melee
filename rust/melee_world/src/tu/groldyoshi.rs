//! Port of src/melee/gr/groldyoshi.c (Yoshi's Story, the 1.0 "old" map: the three clouds that drop
//! and respawn, the shy-guy guest that walks across). Every function of the TU, native: pointers
//! into the game's structs, callees of other TUs through `ext` hooks. Checked against the C (whole
//! TU, shipping contraction, stateful oracle) by tests/tu_groldyoshi.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py groldyoshi`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::inline::*;
use melee_types::*;

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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjGetFlags(a0: *mut HSD_JObj) -> u32 = |a0: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(a0 as _) as _ };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { let _ = crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C97DC(a0: i32, a1: i32, a2: *mut HSD_JObj) = |a0: i32, a1: i32, a2: *mut HSD_JObj| { unsafe { let _ = crate::tu::grlib::grLib_801C97DC(a0 as _, a1 as _, a2 as _); } };
    fn grMaterial_801C8858(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8858(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { let _ = crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { let _ = crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
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

/// gr/inlines.h `Ground_StartMapAnim`.
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// gr/inlines.h `Ground_InitMapCollAndAnim`.
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// gr/inlines.h `Ground_SetupStageCallbacks` (TARGET_PC path: callbacks go through `Mex_GrBind`).
unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).callback3)));
    }
    if (*callbacks).on_init.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).on_init)));
        (f.unwrap())(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(core::mem::transmute::<_, *mut c_void>((*callbacks).gobj_proc)));
        ext::HSD_GObj_SetupProc(gobj, f, 4);
    }
}

/// gr/inlines.h `rand_range`.
unsafe fn rand_range(a: i32, b: i32) -> i32 {
    if a > b {
        let n = a.wrapping_sub(b);
        b.wrapping_add(if n != 0 { ext::HSD_Randi(n) } else { 0 })
    } else if a < b {
        let n = b.wrapping_sub(a);
        a.wrapping_add(if n != 0 { ext::HSD_Randi(n) } else { 0 })
    } else {
        a
    }
}

/// The anonymous struct `yakumono_param` points at (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OyYakumonoParam {
    pub x0: i16,
    pub x2: i16,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: i16,
    pub x12: i16,
    pub x14: i16,
    pub x16: i16,
    pub x18: i16,
}
melee_core::gc_layout!(OyYakumonoParam, "grOldYoshi_yakumono_param", size 0x1C, align 4, [
    x0 @ 0x0: 2 "s16", x2 @ 0x2: 2 "s16", x4 @ 0x4: 4 "float", x8 @ 0x8: 4 "float", xC @ 0xC: 4 "float",
    x10 @ 0x10: 2 "s16", x12 @ 0x12: 2 "s16", x14 @ 0x14: 2 "s16", x16 @ 0x16: 2 "s16", x18 @ 0x18: 2 "s16",
]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOy_803B83F0_t {
    pub x: [i32; 5],
}
melee_core::gc_layout!(grOy_803B83F0_t, "grOy_803B83F0_t", size 0x14, align 4, [x @ 0x0: 20 "int[5]"]);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOy_803B83F0_Data {
    pub ids: grOy_803B83F0_t,
    pub pad: i32,
}
melee_core::gc_layout!(grOy_803B83F0_Data, "grOy_803B83F0_Data", size 0x18, align 4, [
    ids @ 0x0: 20 "grOy_803B83F0_t", pad @ 0x14: 4 "int",
]);

/// `grOy_803B83F0` (3B83F0)
pub static grOy_803B83F0: grOy_803B83F0_Data = grOy_803B83F0_Data { ids: grOy_803B83F0_t { x: [1, 2, 3, 4, 5] }, pad: 0 };

/// `static ... * yakumono_param` (the map's `Ground_GetYakumonoParam()` block)
pub static mut yakumono_param: *mut OyYakumonoParam = null_mut();

/// `grOy_803E6574` (3E6574)
pub static mut grOy_803E6574: [i16; 6] = [0, 1, 1, 5, 2, 9];

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags } }
}

/// `grOy_StageCallbacks` (3E64A8)
pub static mut grOy_StageCallbacks: [StageCallbacks; 6] = [
    cb(Some(stageGObj0_OnInit), Some(grOldYoshi_8020E968), Some(grOldYoshi_8020E970), Some(grOldYoshi_8020E974), 0),
    cb(Some(grOldYoshi_8020E978), Some(grOldYoshi_8020E9E0), Some(grOldYoshi_8020E9E8), Some(grOldYoshi_8020E9EC), 0),
    cb(Some(grOldYoshi_8020EAFC), Some(grOldYoshi_8020EC08), Some(grOldYoshi_8020EC10), Some(grOldYoshi_8020EFC8), 0),
    cb(Some(grOldYoshi_8020EFCC), Some(grOldYoshi_8020F080), Some(grOldYoshi_8020F088), Some(grOldYoshi_8020F2A4), 0),
    cb(Some(grOldYoshi_8020E9F0), Some(grOldYoshi_8020EA5C), Some(grOldYoshi_8020EA64), Some(grOldYoshi_8020EA88), 0xC0000000),
    cb(Some(grOldYoshi_8020EA8C), Some(grOldYoshi_8020EAEC), Some(grOldYoshi_8020EAF4), Some(grOldYoshi_8020EAF8), 0),
];

/// `grOy_StageData` (3E650C)
pub static mut grOy_StageData: StageData = StageData {
    grkind: Gr_Kind_OldYoshi as _,
    callbacks: unsafe { &raw mut grOy_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrOy.dat\0".as_ptr() as *mut i8,
    on_init: Some(grOldYoshi_8020E79C),
    on_demo_init: Some(grOldYoshi_8020E798),
    on_load: Some(grOldYoshi_8020E824),
    on_start: Some(grOldYoshi_8020E828),
    callback4: Some(grOldYoshi_8020E84C),
    on_touch_line: Some(grOldYoshi_8020F404),
    on_check_shadow_render: Some(grOldYoshi_8020F40C),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grOldYoshi_8020E798(_arg: i32) {}

pub unsafe extern "C" fn grOldYoshi_8020E79C() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut OyYakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grOldYoshi_8020E854(0);
    grOldYoshi_8020E854(1);
    grOldYoshi_8020E854(4);
    grOldYoshi_8020E854(5);
    grOldYoshi_8020E854(2);
    grOldYoshi_8020E854(3);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grOldYoshi_8020E824() {}

pub unsafe extern "C" fn grOldYoshi_8020E828() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grOldYoshi_8020E84C() -> i32 {
    0
}

pub unsafe extern "C" fn grOldYoshi_8020E854(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = (&raw mut grOy_StageCallbacks as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grOldYoshi_8020E968(_arg: *mut HSD_GObj) -> i32 {
    0
}
pub unsafe extern "C" fn grOldYoshi_8020E970(_arg: *mut HSD_GObj) {}
pub unsafe extern "C" fn grOldYoshi_8020E974(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldYoshi_8020E978(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    ext::grMaterial_801C8858(jobj, JOBJ_UNK_B27 as u32);
    ext::grMaterial_801C8858(jobj, JOBJ_ROOT_XLU as u32);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grOldYoshi_8020E9E0(_arg: *mut HSD_GObj) -> i32 {
    0
}
pub unsafe extern "C" fn grOldYoshi_8020E9E8(_arg: *mut HSD_GObj) {}
pub unsafe extern "C" fn grOldYoshi_8020E9EC(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldYoshi_8020E9F0(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    Ground_InitMapCollAndAnim(gobj);
    (*gp).x11_flags.set_b012(1);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grOldYoshi_8020EA5C(_arg: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldYoshi_8020EA64(arg0: *mut HSD_GObj) {
    ext::Ground_UpdateMapColl(arg0);
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grOldYoshi_8020EA88(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldYoshi_8020EA8C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    Ground_InitMapCollAndAnim(gobj);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grOldYoshi_8020EAEC(_arg: *mut HSD_GObj) -> i32 {
    0
}
pub unsafe extern "C" fn grOldYoshi_8020EAF4(_arg: *mut HSD_GObj) {}
pub unsafe extern "C" fn grOldYoshi_8020EAF8(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldYoshi_8020EAFC(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    for i in 0..3usize {
        let cloud = &mut (*gp).u.oldyoshicloud.cloud[i];
        cloud.set_xC4_0123(0);
        cloud.set_xC4_4(0);
        cloud.set_xC4_567(0);
        cloud.xD4 = 0.0;
        cloud.xD0 = 0.0;
        cloud.xC8 = ext::Ground_801C3FA4(gobj, grOy_803E6574[i * 2 + 1] as i32);
        cloud.xCC = HSD_JObjGetTranslationY(&*cloud.xC8);
        ext::mpJointSetCb1(grOy_803E6574[i * 2] as i32, gp as *mut c_void, Some(fn_8020F2A8));
    }
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grOldYoshi_8020EC08(_arg: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldYoshi_8020EC10(arg: *mut HSD_GObj) {
    let gp = (*arg).user_data as *mut Ground;
    let yp = yakumono_param;
    for i in 0..3usize {
        let c = &mut (*gp).u.oldyoshicloud.cloud[i];
        match c.xC4_0123() {
            0 => {
                if c.xC4_4() != 0 {
                    c.xD4 = grOldYoshi_8020F31C(c.xD4, (*yp).x8, c.xD0, (*yp).xC, (*yp).x4, -(*yp).x4);
                    c.xD0 += c.xD4;
                    if c.xD0 > (*yp).xC {
                        c.xD0 = (*yp).xC;
                    }
                    let u_var3 = c.xC4_567();
                    c.set_xC4_567(u_var3.wrapping_add(1));
                    if u_var3 as i32 > (*yp).x0 as i32 {
                        c.set_xC4_0123(1);
                        ext::grAnime_801C7FF8(arg, grOy_803E6574[i * 2 + 1] as i32, 2, 1, 0.0, 1.0);
                        ext::grLib_801C97DC(30000, 30, c.xC8);
                        c.set_xC4_567(0);
                    }
                } else {
                    c.xD4 = grOldYoshi_8020F31C(c.xD4, (*yp).x8, c.xD0, 0.0, (*yp).x4, -(*yp).x4);
                    c.xD0 += c.xD4;
                    if c.xD0 < 0.0 {
                        c.xD0 = 0.0;
                    }
                    c.set_xC4_567(0);
                }
            }
            1 => {
                if c.xC4_567() as i32 == (*yp).x10 as i32 {
                    ext::mpLib_80057BC0(grOy_803E6574[i * 2] as i32);
                }
                c.set_xC4_567(c.xC4_567().wrapping_add(1));
                if ext::grAnime_801C83D0(arg, grOy_803E6574[i * 2 + 1] as i32, 2) != 0 {
                    c.set_xC4_0123(2);
                    ext::mpLib_80057BC0(grOy_803E6574[i * 2] as i32);
                    ext::HSD_JObjSetFlagsAll(c.xC8, JOBJ_HIDDEN as u32);
                    c.set_xC4_567(0);
                }
            }
            2 => {
                let u_var3 = c.xC4_567();
                c.set_xC4_567(u_var3.wrapping_add(1));
                if u_var3 as i32 > (*yp).x2 as i32 {
                    c.set_xC4_0123(3);
                    ext::grAnime_801C7FF8(arg, grOy_803E6574[i * 2 + 1] as i32, 2, 2, 0.0, 1.0);
                    c.set_xC4_567(0);
                }
            }
            3 => {
                if c.xC4_567() == 0 {
                    ext::HSD_JObjClearFlagsAll(c.xC8, JOBJ_HIDDEN as u32);
                }
                if c.xC4_567() as i32 == (*yp).x12 as i32 {
                    ext::mpJointListAdd(grOy_803E6574[i * 2] as i32);
                }
                c.set_xC4_567(c.xC4_567().wrapping_add(1));
                if ext::grAnime_801C83D0(arg, grOy_803E6574[i * 2 + 1] as i32, 2) != 0 {
                    c.set_xC4_0123(0);
                    c.set_xC4_567(0);
                }
            }
            _ => {}
        }
        HSD_JObjSetTranslateY(&mut *c.xC8, c.xCC - c.xD0, &mut Dirty);
        c.set_xC4_4(0);
    }
    ext::Ground_UpdateMapColl(arg);
}

pub unsafe extern "C" fn grOldYoshi_8020EFC8(_arg: *mut HSD_GObj) {}

/// `grOldYoshi_8020EFCC_inline`: the user_data read the C kept out of GET_GROUND
pub unsafe fn grOldYoshi_8020EFCC_inline(arg0: *mut HSD_GObj) -> *mut Ground {
    (*arg0).user_data as *mut Ground
}

pub unsafe extern "C" fn grOldYoshi_8020EFCC(arg: *mut HSD_GObj) {
    let jobj = (*arg).hsd_obj as *mut HSD_JObj;
    let gp = grOldYoshi_8020EFCC_inline(arg);
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
    (*gp).u.oldyoshiguest.xC4 = rand_range((*yakumono_param).x16 as i32, (*yakumono_param).x14 as i32) as i16;
    (*gp).u.oldyoshiguest.xC6 = -1;
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grOldYoshi_8020F080(_arg: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldYoshi_8020F088(arg: *mut HSD_GObj) {
    let gp = (*arg).user_data as *mut Ground;
    if (*gp).u.oldyoshiguest.xC6 == -1 {
        let s_var5 = (*gp).u.oldyoshiguest.xC4;
        (*gp).u.oldyoshiguest.xC4 = s_var5.wrapping_sub(1);
        if s_var5 < 0 {
            let local34 = grOy_803B83F0.ids;
            (*gp).u.oldyoshiguest.xC6 = local34.x[ext::HSD_Randi(5) as usize] as i16;
        }
        ext::grAnime_801C8138(arg, (*gp).map_id as i32, 0);
        let d_var10 = ext::HSD_Randf();
        let d_var9 = ((*yakumono_param).x18 as f32) * fmadds(d_var10, 2.0, -1.0);
        HSD_JObjSetTranslateY(&mut *((*arg).hsd_obj as *mut HSD_JObj), d_var9, &mut Dirty);
    } else {
        let jobj = ext::Ground_801C3FA4(arg, (*gp).u.oldyoshiguest.xC6 as i32);
        if jobj.is_null() {
            return;
        }
        if ext::HSD_JObjGetFlags(jobj) & 0x10 != 0 {
            ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
        }
        if ext::grAnime_801C83D0(arg, 0, 7) != 0 {
            ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
            (*gp).u.oldyoshiguest.xC4 = rand_range((*yakumono_param).x16 as i32, (*yakumono_param).x14 as i32) as i16;
            (*gp).u.oldyoshiguest.xC6 = -1;
        }
    }
}

pub unsafe extern "C" fn grOldYoshi_8020F2A4(_arg: *mut HSD_GObj) {}

/// 20F2A8 `mpLib_JointCollisionCallback`
pub unsafe extern "C" fn fn_8020F2A8(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, _coll_x50: i32, _ground_kind: u32, _delta_y: f32) {
    let gp = user_data as *mut Ground;
    if (*coll).x34_flags.b1234() as i32 != 1 {
        return;
    }
    let mut i = 0usize;
    while i < 3 {
        if joint_id == grOy_803E6574[i * 2] as i32 {
            break;
        }
        i += 1;
    }
    if i == 3 {
        return;
    }
    (*gp).u.oldyoshicloud.cloud[i].set_xC4_4(1);
}

pub unsafe fn grOldYoshi_8020F31C_inline(mut param1: f32, param2: f32, param3: f32, param4: f32, distance: f32) -> f32 {
    let mut f_var1 = param4 - param3;
    if f_var1 < 0.0 {
        f_var1 = -f_var1;
    }
    if f_var1 < distance && ((param4 > param3 && param1 > 0.0) || (param4 < param3 && param1 < 0.0)) {
        if param1 > 0.0 {
            param1 = param1 - param2;
        } else if param1 < 0.0 {
            param1 = param1 + param2;
        }
    } else if param4 > param3 {
        param1 = param1 + param2;
    } else if param4 < param3 {
        param1 = param1 - param2;
    }
    param1
}

pub unsafe extern "C" fn grOldYoshi_8020F31C(param1: f32, param2: f32, param3: f32, param4: f32, param5: f32, param6: f32) -> f32 {
    let f_var2 = if param1 < 0.0 { -param1 } else { param1 };
    let f_var3 = f_var2 / param2;
    let f_var2 = if param1 < 0.0 { -param1 } else { param1 };
    // fVar3 * (-param2 * 0.5f * fVar3) + (fVar3 * fVar2): clang fuses the LHS multiply
    let f_var4 = fmadds(f_var3, -param2 * 0.5 * f_var3, f_var3 * f_var2);
    let param1 = grOldYoshi_8020F31C_inline(param1, param2, param3, param4, f_var4);
    if param1 > param5 {
        return param5;
    }
    if param1 < param6 {
        return param6;
    }
    param1
}

pub unsafe extern "C" fn grOldYoshi_8020F404(_arg: i32) -> *mut DynamicsDesc {
    null_mut()
}

pub unsafe extern "C" fn grOldYoshi_8020F40C(a: *mut Vec3, _arg: i32, joint: *mut HSD_JObj) -> i32 {
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint, null_mut(), &mut b);
    if (*a).y > b.y + -5.0 {
        1
    } else {
        0
    }
}
