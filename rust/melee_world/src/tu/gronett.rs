//! Port of src/melee/gr/gronett.c (Onett: the awnings, the building that collapses and the passing
//! cars). Every function of the TU, native: pointers into the game's structs, callees of other TUs
//! through `ext` hooks. Checked against the C (whole TU, shipping contraction, stateful oracle) by
//! tests/tu_gronett.rs; regenerate the C side with `python rust/melee_world/gen/cref/run_gr.py gronett`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
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
    fn Camera_LogicBounds(a0: *mut f32, a1: *mut f32, a2: *mut f32) -> i32;
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj| { unsafe { crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { crate::tu::ground::Ground_801C53EC(a0 as _); } };
    fn Ground_801C5440(a0: *mut Ground, a1: i32, a2: u32) = |a0: *mut Ground, a1: i32, a2: u32| { unsafe { crate::tu::ground::Ground_801C5440(a0 as _, a1 as _, a2 as _); } };
    fn Ground_801C5544(a0: *mut Ground, a1: i32) = |a0: *mut Ground, a1: i32| { unsafe { crate::tu::ground::Ground_801C5544(a0 as _, a1 as _); } };
    fn Ground_801C5784(a0: i32) = |a0: i32| { unsafe { crate::tu::ground::Ground_801C5784(a0 as _); } };
    fn Ground_801C5794() -> i32 = || unsafe { crate::tu::ground::Ground_801C5794() as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32 = || melee_runtime::tu::random::HSD_Randf(melee_runtime::tu::globals::rand());
    fn HSD_Randi(a0: i32) -> i32 = |a0: i32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), a0);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec) = |a0: *mut HSD_GObj, a1: *mut Vec| { unsafe { let _ = crate::ftlib::ftLib_80086644(a0 as _, a1 as _); } };
    fn gm_8016C6C0(a0: *mut HSD_GObj) -> i32;
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grDatFiles_801C6330(a0: i32) -> *mut UnkArchiveStruct = |a0: i32| unsafe { crate::tu::grdatfiles::grDatFiles_801C6330(a0 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut Vec| unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C9808(a0: i32, a1: i32, a2: *mut HSD_JObj) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut HSD_JObj| unsafe { crate::tu::grlib::grLib_801C9808(a0 as _, a1 as _, a2 as _) as _ };
    fn grLib_801C9834(a0: *mut HSD_Generator) = |a0: *mut HSD_Generator| { unsafe { crate::tu::grlib::grLib_801C9834(a0 as _); } };
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn grMaterial_801C8E08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { crate::tu::grmaterial::grMaterial_801C8E08(a0 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn it_80274C60(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::itacc::it_80274C60(a0 as _); } };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn mpJointListAdd(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpJointListAdd(a0 as _); } };
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>) = |a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>| { unsafe { crate::tu::mplib::mpJointSetCb1(a0 as _, a1 as _, a2 as _); } };
    fn mpLib_80057BC0(a0: i32) = |a0: i32| { unsafe { crate::tu::mplib::mpLib_80057BC0(a0 as _); } };
    fn un_802FD604(a0: i32) = |a0: i32| { unsafe { crate::tu::ifhazard::un_802FD604(a0 as _); } };
    fn un_802FD65C() = || { unsafe { crate::tu::ifhazard::un_802FD65C(); } };
    fn un_802FD8A0(a0: *mut DynamicModelDesc) = |a0: *mut DynamicModelDesc| { unsafe { crate::tu::ifhazard::un_802FD8A0(a0 as _); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn HSD_Randi(a0: i32) -> i32 { super::HSD_Randi(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
    pub use crate::tu::ground::stage_info;
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
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

/// gr/inlines.h `Ground_UpdateWindAndMapColl`.
unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
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

/// `ABS(x)`: `((x) < 0 ? -(x) : (x))`
#[inline(always)]
fn abs_macro(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `struct grOnett_StageParam` (file-local in the C): Onett stage yakumono parameters
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOnett_StageParam {
    pub awning_initial: f32,
    pub max_velocity: f32,
    pub vel_threshold: f32,
    pub pos_threshold: f32,
    pub damping: f32,
    pub spring_force: f32,
    pub spring_constant: f32,
    pub max_displacement: f32,
    pub awning_delta: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: f32,
    pub x38: f32,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: f32,
    pub x4C: f32,
    pub x50: f32,
    pub x54: f32,
    pub x58: f32,
    pub x5C: f32,
    pub x60: f32,
    pub x64: f32,
}
melee_core::gc_layout!(grOnett_StageParam, "grOnett_StageParam", size 0x68, align 4, [
    awning_initial @ 0x0: 4 "f32", max_velocity @ 0x4: 4 "f32", vel_threshold @ 0x8: 4 "f32",
    pos_threshold @ 0xC: 4 "f32", damping @ 0x10: 4 "f32", spring_force @ 0x14: 4 "f32",
    spring_constant @ 0x18: 4 "f32", max_displacement @ 0x1C: 4 "f32", awning_delta @ 0x20: 4 "f32",
    x24 @ 0x24: 4 "f32", x28 @ 0x28: 4 "f32", x2C @ 0x2C: 4 "f32", x30 @ 0x30: 4 "f32",
    x34 @ 0x34: 4 "f32", x38 @ 0x38: 4 "f32", x3C @ 0x3C: 4 "f32", x40 @ 0x40: 4 "f32",
    x44 @ 0x44: 4 "f32", x48 @ 0x48: 4 "f32", x4C @ 0x4C: 4 "f32", x50 @ 0x50: 4 "f32",
    x54 @ 0x54: 4 "f32", x58 @ 0x58: 4 "f32", x5C @ 0x5C: 4 "f32", x60 @ 0x60: 4 "f32", x64 @ 0x64: 4 "f32",
]);

/// `static struct grOnett_StageParam* yakumono_param`
pub static mut yakumono_param: *mut grOnett_StageParam = null_mut();

/// `static s32 grOt_804D69C4`
pub static mut grOt_804D69C4: i32 = 0;

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags: u32::to_be(flags) } }
}

/// `grOt_803E27E0` (3E27E0)
pub static mut grOt_803E27E0: [StageCallbacks; 6] = [
    cb(None, None, None, None, 0),
    cb(None, None, None, None, 0),
    cb(Some(grOnett_801E38DC), Some(grOnett_801E3920), Some(grOnett_801E3928), Some(grOnett_801E392C), 0),
    cb(Some(grOnett_801E41C8), Some(grOnett_801E43D8), Some(grOnett_801E43E0), Some(grOnett_801E502C), 0),
    cb(Some(grOnett_801E3CE4), Some(grOnett_801E3D98), Some(grOnett_801E3DA0), Some(grOnett_801E40E0), 0),
    cb(Some(grOnett_801E3A34), Some(grOnett_801E3C58), Some(grOnett_801E3C60), Some(grOnett_801E3CE0), (1 << 30) | (1 << 31)),
];

/// `grOt_StageData` (3E2858)
pub static mut grOt_StageData: StageData = StageData {
    grkind: Gr_Kind_Onett as _,
    callbacks: unsafe { &raw mut grOt_803E27E0 as *mut StageCallbacks },
    data1: b"/GrOt\0".as_ptr() as *mut i8,
    on_init: Some(grOnett_801E3738),
    on_demo_init: Some(grOnett_801E3734),
    on_load: Some(grOnett_801E37C4),
    on_start: Some(grOnett_801E37C8),
    callback4: Some(grOnett_801E37EC),
    on_touch_line: Some(grOnett_801E5760),
    on_check_shadow_render: Some(grOnett_801E5768),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grOnett_801E3734(_arg: i32) { melee_core::ct!("grOnett_801E3734");}

pub unsafe extern "C" fn grOnett_801E3738() {
    melee_core::ct!("grOnett_801E3738");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grOnett_StageParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grOnett_801E37F4(0);
    grOnett_801E37F4(2);
    grOnett_801E37F4(5);
    grOnett_801E37F4(4);
    grOnett_801E37F4(3);
    ext::un_802FD8A0(grOnett_801E56FC());
    ext::un_802FD65C();
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

pub unsafe extern "C" fn grOnett_801E37C4() { melee_core::ct!("grOnett_801E37C4");}

pub unsafe extern "C" fn grOnett_801E37C8() {
    melee_core::ct!("grOnett_801E37C8");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grOnett_801E37EC() -> i32 {
    melee_core::ct!("grOnett_801E37EC");
    0
}

pub unsafe extern "C" fn grOnett_801E37F4(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grOnett_801E37F4");
    let callbacks = (&raw mut grOt_803E27E0 as *mut StageCallbacks).add(gobj_id as usize);
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    gobj
}

pub unsafe extern "C" fn grOnett_801E38DC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E38DC");
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grOnett_801E3920(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grOnett_801E3920");
    0
}

pub unsafe extern "C" fn grOnett_801E3928(_gobj: *mut HSD_GObj) { melee_core::ct!("grOnett_801E3928");}

pub unsafe extern "C" fn grOnett_801E392C(_gobj: *mut HSD_GObj) { melee_core::ct!("grOnett_801E392C");}

pub unsafe extern "C" fn grOnett_801E3930(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E3930");
    let gp = (*gobj).user_data as *mut Ground;

    ext::mpJointSetCb1(0, gp as *mut c_void, Some(grOnett_801E54B4));
    ext::mpJointSetCb1(1, gp as *mut c_void, Some(grOnett_801E54B4));
    (*gp).u.onett.subject = ext::Camera_80029020();
    assert!(!(*gp).u.onett.subject.is_null(), "gp->u.map.subject"); // HSD_ASSERTMSG(331, ...)
    let s = (*gp).u.onett.subject;
    (*s).target_ext.h.x = -40.0;
    (*s).target_ext.h.y = 40.0;
    (*s).target_ext.v.x = 20.0;
    (*s).target_ext.v.y = -20.0;
    (*s).target_ext.v.z = 1.0;
    (*s).pos.x = 0.0;
    (*s).pos.y = 0.0;
    (*s).pos.z = 0.0;
    (*s).bone_pos = (*s).pos;
    (*s).state = CmSubjectState_Inactive;
}

pub unsafe extern "C" fn grOnett_801E3A34(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E3A34");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let yp = yakumono_param;

    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x10_flags.set_b5(1);

    (*gp).u.onett.subject = null_mut();

    (*gp).u.onett.awnings[0].initial = 0.0;
    (*gp).u.onett.awnings[0].jobj = ext::Ground_801C3FA4(gobj, 0xE);
    (*gp).u.onett.awnings[0].initial_y = HSD_JObjGetTranslationY(&*(*gp).u.onett.awnings[0].jobj);
    (*gp).u.onett.awnings[0].accumulator = 0.0;
    (*gp).u.onett.awnings[0].velocity = 0.0;
    (*gp).u.onett.awnings[0].counter_prev = 0;
    (*gp).u.onett.awnings[0].counter = 0;
    (*gp).u.onett.awnings[0].flag = 0;
    (*gp).u.onett.awnings[0].cooldown = 0;

    (*gp).u.onett.awnings[1].initial = 0.0;
    (*gp).u.onett.awnings[1].jobj = ext::Ground_801C3FA4(gobj, 0xF);
    (*gp).u.onett.awnings[1].initial_y = HSD_JObjGetTranslationY(&*(*gp).u.onett.awnings[1].jobj);
    (*gp).u.onett.awnings[1].accumulator = 0.0;
    (*gp).u.onett.awnings[1].velocity = 0.0;
    (*gp).u.onett.awnings[1].counter_prev = 0;
    (*gp).u.onett.awnings[1].counter = 0;
    (*gp).u.onett.awnings[1].flag = 0;
    (*gp).u.onett.awnings[1].cooldown = 0;

    (*gp).u.onett.timer = rand_range(f2i32((*yp).x28 as f64), f2i32((*yp).x24 as f64));
    (*gp).u.onett.gen = null_mut();

    // arguments are evaluated left to right
    let a0 = ext::Ground_801C3FA4(gobj, 16);
    let a1 = ext::Ground_801C3FA4(gobj, 17);
    let a2 = ext::Ground_801C3FA4(gobj, 19);
    let a3 = ext::Ground_801C3FA4(gobj, 20);
    let a4 = ext::Ground_801C3FA4(gobj, 18);
    let a5 = ext::Ground_801C3FA4(gobj, 21);
    ext::Ground_801C4E70(a0, a1, a2, a3, a4, a5);

    ext::Ground_801C10B8(gobj, Some(grOnett_801E3930));
    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grOnett_801E3C58(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grOnett_801E3C58");
    0
}

pub unsafe extern "C" fn grOnett_801E3C60(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E3C60");
    let gp = (*gobj).user_data as *mut Ground;

    grOnett_801E5214(gobj);
    grOnett_801E5538(gobj);
    if !(*gp).u.onett.subject.is_null() {
        if ext::Ground_801C5794() != 0 {
            (*(*gp).u.onett.subject).state = CmSubjectState_Active;
        } else {
            (*(*gp).u.onett.subject).state = CmSubjectState_Inactive;
        }
    }
    Ground_UpdateWindAndMapColl(gobj);
}

pub unsafe extern "C" fn grOnett_801E3CE0(_gobj: *mut HSD_GObj) { melee_core::ct!("grOnett_801E3CE0");}

pub unsafe extern "C" fn grOnett_801E3CE4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E3CE4");
    let gp = (*gobj).user_data as *mut Ground;

    ext::Ground_InitMapColl((*gobj).hsd_obj as *mut HSD_JObj, (*gp).map_id as i32);
    ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 0.0);

    (*gp).u.onett_building.state = -1;
    (*gp).u.onett_building.next_state = 0;
    (*gp).u.onett_building.hit_count = 0;
    (*gp).u.onett_building.frame = 0;

    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x10_flags.set_b5(1);

    ext::mpJointSetCb1(3, gp as *mut c_void, Some(grOnett_801E40E4));
    ext::mpJointSetCb1(4, gp as *mut c_void, Some(grOnett_801E40E4));
}

pub unsafe extern "C" fn grOnett_801E3D98(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grOnett_801E3D98");
    0
}

pub unsafe extern "C" fn grOnett_801E3DA0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E3DA0");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let yp = yakumono_param;
    let b = &mut (*gp).u.onett_building;

    b.frame = b.frame.wrapping_add(1);

    match b.state {
        7 => {
            if b.timer & 2 != 0 {
                ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 5), JOBJ_HIDDEN as u32);
                ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 2), JOBJ_HIDDEN as u32);
                ext::HSD_JObjSetFlagsAll(ext::Ground_801C3FA4(gobj, 4), JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
            let tmp = b.timer as i32;
            b.timer = tmp.wrapping_sub(1) as u32;
            if tmp < 0 {
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
                b.next_state = 0;
            }
        }
        1 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                b.next_state = 0;
            }
        }
        2 => {
            if b.frame == 0x8C {
                ext::Ground_801C5440(gp, 0, 0x5F373);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                b.next_state = 3;
            }
        }
        4 => {
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                b.next_state = 3;
            }
        }
        5 => {
            if b.frame == 0x46 || b.frame == 0x82 {
                ext::Ground_801C5440(gp, 0, 0x5F374);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                let rand = ext::HSD_Randf();
                let lo = (*yp).x3C;
                let range = (*yp).x40 - lo;
                b.timer = f2i32(fmadds(range, rand, lo) as f64) as u32;
                b.next_state = 6;
            }
            if b.frame == 0x14 {
                ext::mpLib_80057BC0(3);
            } else if b.frame == 0x5A {
                ext::mpLib_80057BC0(4);
            }
        }
        6 => {
            let tmp = b.timer as i32;
            b.timer = tmp.wrapping_sub(1) as u32;
            if tmp < 0 {
                b.next_state = 7;
                b.timer = 0x3C;
            }
        }
        _ => {}
    }

    if b.state != b.next_state {
        b.state = b.next_state;
        match b.state {
            7 => {
                ext::grAnime_801C7FF8(gobj, 0, 7, 0, 0.0, 1.0);
                ext::mpJointListAdd(3);
                ext::mpJointListAdd(4);
            }
            1 => ext::grAnime_801C7FF8(gobj, 0, 7, 1, 0.0, 1.0),
            2 => ext::grAnime_801C7FF8(gobj, 0, 7, 3, 0.0, 1.0),
            4 => ext::grAnime_801C7FF8(gobj, 0, 7, 4, 0.0, 1.0),
            5 => ext::grAnime_801C7FF8(gobj, 0, 7, 5, 0.0, 1.0),
            6 => b.hit_count = 0,
            _ => {}
        }
        b.frame = 0;
    }

    ext::Ground_UpdateMapColl(gobj);
}

pub unsafe extern "C" fn grOnett_801E40E0(_gobj: *mut HSD_GObj) { melee_core::ct!("grOnett_801E40E0");}

/// `mpLib_JointCollisionCallback` of the building
pub unsafe extern "C" fn grOnett_801E40E4(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32, ground_kind: u32, delta_y: f32) {
    melee_core::ct!("grOnett_801E40E4");
    let gp = user_data as *mut Ground;
    let yp = yakumono_param;

    if ground_kind == 1 {
        (*gp).u.onett_building.hit_count = (*gp).u.onett_building.hit_count.wrapping_add(1);
        match (*gp).u.onett_building.state {
            4 | 5 => {}
            0 => {
                if ((*gp).u.onett_building.hit_count as f32) < (*yp).x34 {
                    (*gp).u.onett_building.next_state = 1;
                    return;
                }
                (*gp).u.onett_building.next_state = 2;
                return;
            }
            3 => {
                if ((*gp).u.onett_building.hit_count as f32) < (*yp).x38 {
                    (*gp).u.onett_building.next_state = 4;
                    return;
                }
                (*gp).u.onett_building.next_state = 5;
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grOnett_801E41B0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E41B0");
    (*((*gobj).user_data as *mut Ground)).u.onettcar.set_x0_b0(0);
}

pub unsafe extern "C" fn grOnett_801E41C8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E41C8");
    let gp = (*gobj).user_data as *mut Ground;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;

    let c = &mut (*gp).u.onettcar;
    c.car_jobjs[0] = ext::Ground_801C3FA4(gobj, 0xB);
    c.car_jobjs[1] = ext::Ground_801C3FA4(gobj, 0x1);
    c.car_jobjs[2] = ext::Ground_801C3FA4(gobj, 0x6);
    c.car_jobjs[3] = ext::Ground_801C3FA4(gobj, 0x10);
    c.unk_jobj = ext::Ground_801C3FA4(gobj, 0x10);

    c.car_jobjs2[0] = ext::Ground_801C3FA4(gobj, 0xF);
    c.car_jobjs2[1] = ext::Ground_801C3FA4(gobj, 0x5);
    c.car_jobjs2[2] = ext::Ground_801C3FA4(gobj, 0xA);
    c.car_jobjs2[3] = ext::Ground_801C3FA4(gobj, 0x14);
    c.unk_jobj2 = ext::Ground_801C3FA4(gobj, 0x14);

    for i in 0..4usize {
        HSD_JObjSetTranslateX(&mut *c.car_jobjs[i], 726.0, &mut Dirty);
        ext::HSD_JObjSetFlagsAll(c.car_jobjs[i], JOBJ_HIDDEN as u32);
        c.car_items[i] = ext::grMaterial_801C8CFC(0, 0, gp, c.car_jobjs[i], Some(grOnett_801E5030), None, None);
        ext::grMaterial_801C8E08(c.car_items[i]);
        ext::it_80274C60(c.car_items[i]);
    }

    c.curr_car = ext::HSD_Randi(4) as i8;
    c.next_car = 4;
    c.curr_car = 4;
    c.state_b = 0;
    c.state_a = 0;
    grOt_804D69C4 = 0;

    ext::Ground_801C10B8(gobj, Some(grOnett_801E41B0));
    (*gp).u.onettcar.set_x0_b0(1);
}

pub unsafe extern "C" fn grOnett_801E43D8(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("grOnett_801E43D8");
    0
}

/// `HSD_RandF_noinline` (file-local inline)
pub unsafe fn HSD_RandF_noinline() -> f32 {
    melee_core::ct!("HSD_RandF_noinline");
    ext::HSD_Randf()
}

/// `grOnett_StartNextCar` (file-local inline)
pub unsafe fn grOnett_StartNextCar(gp: *mut Ground) {
    melee_core::ct!("grOnett_StartNextCar");
    let yp = yakumono_param;
    let c = &mut (*gp).u.onettcar;
    let next = c.next_car;

    while c.next_car == next || c.next_car == c.curr_car {
        c.next_car = ext::HSD_Randi(4) as i8;
    }

    let jobj = c.car_jobjs[c.next_car as usize];
    HSD_JObjSetTranslateX(&mut *jobj, -642.0, &mut Dirty);
    HSD_JObjSetTranslateZ(&mut *jobj, 56.0, &mut Dirty);
    HSD_JObjSetRotationY(&mut *(*jobj).child, 3.14159265358979323846f64 as f32, &mut Dirty);
    c.state_b = 1;
    c.timer_b = f2i32((*yp).x44 as f64);
}

/// `grOnett_WaitCar` (file-local inline)
pub unsafe fn grOnett_WaitCar(gp: *mut Ground, saved_car: i8) {
    melee_core::ct!("grOnett_WaitCar");
    let yp = yakumono_param;
    let c = &mut (*gp).u.onettcar;
    let t = c.x108;

    if t != 0 {
        c.x108 = t.wrapping_sub(1);
    } else {
        let car = c.curr_car as i32;
        c.x108 = f2i32((*yp).x48 as f64);
        {
            let rand = ext::HSD_Randf();
            c.car_speed = fmadds((*yp).x58, rand, (*yp).x54);
        }
        if c.car_speed > 0.0 {
            c.car_speed *= -1.0;
        }
        c.state_a = 2;
        ext::Item_80268E5C(c.car_items[car as usize], saved_car as i32 + 2, ITEM_ANIM_UPDATE as _);
    }
}

pub unsafe extern "C" fn grOnett_801E43E0(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E43E0");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let saved_car: i8 = (*gp).u.onettcar.curr_car;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut cam_x: f32 = 0.0;
    let mut cam_y: f32 = 0.0;
    let mut cam_z: f32 = 0.0;

    if (*gp).u.onettcar.x0_b0() == 0 {
        ext::Camera_LogicBounds(&mut cam_x, &mut cam_y, &mut cam_z);

        let mut car_jobj = (*gp).u.onettcar.car_jobjs[saved_car as usize];
        let car_jobj2 = (*gp).u.onettcar.car_jobjs2[saved_car as usize];

        match (*gp).u.onettcar.state_a as i8 {
            0 => {
                let old_car = (*gp).u.onettcar.curr_car as i32;

                while (*gp).u.onettcar.curr_car as i32 == old_car || (*gp).u.onettcar.curr_car == (*gp).u.onettcar.next_car {
                    (*gp).u.onettcar.curr_car = ext::HSD_Randi(4) as i8;
                }

                car_jobj = (*gp).u.onettcar.car_jobjs[(*gp).u.onettcar.curr_car as usize];
                HSD_JObjSetTranslateX(&mut *car_jobj, 726.0, &mut Dirty);
                HSD_JObjSetTranslateZ(&mut *car_jobj, 30.0, &mut Dirty);
                HSD_JObjSetRotationY(&mut *(*car_jobj).child, 0.0, &mut Dirty);
                (*gp).u.onettcar.state_a = 1;
                (*gp).u.onettcar.x108 = f2i32((*yp).x44 as f64);
                ext::Ground_801C5784(0);
            }
            1 => grOnett_WaitCar(gp, saved_car),
            2 => {
                if (*gp).u.onettcar.x108 != 0 {
                    let mut fighter_count = 0;
                    let mut iter = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
                    while !iter.is_null() {
                        iter = (*iter).next;
                        fighter_count += 1;
                    }
                    (*gp).u.onettcar.x108 = (*gp).u.onettcar.x108.wrapping_sub(1);
                    iter = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
                    while !iter.is_null() {
                        ext::ftLib_80086644(iter, &mut pos);
                        if pos.y <= 1.0 {
                            let rank = ext::gm_8016C6C0(iter);
                            if fighter_count == 1 || rank == 0 {
                                (*gp).u.onettcar.x108 = 0;
                                ext::HSD_JObjClearFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                                (*gp).u.onettcar.state_a = 5;
                            } else if fighter_count == rank + 1 {
                                (*gp).u.onettcar.x108 = f2i32((*yp).x50 as f64);
                                (*gp).u.onettcar.state_a = 3;
                            } else {
                                (*gp).u.onettcar.x108 = f2i32((*yp).x4C as f64);
                                (*gp).u.onettcar.state_a = 3;
                            }
                            (*gp).u.onettcar.x110 = 0;
                        }
                        iter = (*iter).next;
                    }
                } else {
                    (*gp).u.onettcar.x108 = 0;
                    (*gp).u.onettcar.x110 = 0;
                    ext::HSD_JObjClearFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                    (*gp).u.onettcar.state_a = 5;
                    ext::Ground_801C5784(1);
                }
            }
            3 => {
                let t = (*gp).u.onettcar.x108;
                if t != 0 {
                    (*gp).u.onettcar.x108 = t.wrapping_sub(1);
                } else {
                    (*gp).u.onettcar.x108 = 0;
                    (*gp).u.onettcar.x110 = 0;
                    ext::HSD_JObjClearFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                    (*gp).u.onettcar.state_a = 5;
                    ext::Ground_801C5784(1);
                }
            }
            4 => {
                HSD_JObjAddRotationY(&mut *(*car_jobj).child, 0.017453292f32 * (*yp).x5C, &mut Dirty);
                ext::grLib_801C9808(0x2C, 0, car_jobj2);
                HSD_JObjAddTranslationX(&mut *car_jobj, (*gp).u.onettcar.car_speed, &mut Dirty);
                if HSD_JObjGetTranslationX(&*car_jobj) <= -642.0 {
                    ext::HSD_JObjSetFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                    (*gp).u.onettcar.state_a = 0;
                    ext::Item_80268E5C((*gp).u.onettcar.car_items[saved_car as usize], 0, ITEM_ANIM_UPDATE as _);
                    return;
                }
            }
            5 | 6 => {
                HSD_JObjAddTranslationX(&mut *car_jobj, (*gp).u.onettcar.car_speed, &mut Dirty);
                let mut run_case2 = false;
                match (*gp).u.onettcar.x110 {
                    0 => {
                        if HSD_JObjGetTranslationX(&*car_jobj) <= 580.0 {
                            grOnett_801E5194(gp, (*gp).u.onettcar.curr_car as i32, 5);
                            ext::un_802FD604(f2i32((*yp).x60 as f64));
                            (*gp).u.onettcar.x110 = 1;
                        }
                    }
                    1 => {
                        if HSD_JObjGetTranslationX(&*car_jobj) <= cam_z + (*yp).x64 {
                            let mut iter = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
                            while !iter.is_null() {
                                ext::ftLib_80086644(iter, &mut pos);
                                if pos.y <= 1.0 {
                                    let car = saved_car;
                                    ext::HSD_Randi(3);
                                    grOnett_801E5140(car as i32);
                                    (*gp).u.onettcar.x110 = 2;
                                    ext::Ground_801C5784(2);
                                    break;
                                }
                                iter = (*iter).next;
                            }
                        }
                        run_case2 = true; // fallthrough
                    }
                    2 => run_case2 = true,
                    _ => {}
                }
                if run_case2 && HSD_JObjGetTranslationX(&*car_jobj) <= 100.0 + cam_y {
                    (*gp).u.onettcar.x110 = 4;
                }
                if HSD_JObjGetTranslationX(&*car_jobj) <= -642.0 {
                    ext::HSD_JObjSetFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                    (*gp).u.onettcar.state_a = 0;
                    ext::Item_80268E5C((*gp).u.onettcar.car_items[saved_car as usize], 0, ITEM_ANIM_UPDATE as _);
                    return;
                }
            }
            _ => {}
        }

        // State machine B: left-to-right car
        {
            let state_b = (*gp).u.onettcar.state_b as i8;
            let next = (*gp).u.onettcar.next_car;
            car_jobj = (*gp).u.onettcar.car_jobjs[next as usize];

            match state_b {
                0 => {
                    grOnett_StartNextCar(gp);
                    return;
                }
                1 => {
                    let t = (*gp).u.onettcar.timer_b;
                    if t != 0 {
                        (*gp).u.onettcar.timer_b = t.wrapping_sub(1);
                        return;
                    }
                    (*gp).u.onettcar.timer_b = f2i32((*yp).x48 as f64);
                    (*gp).u.onettcar.speed_b = fmadds((*yp).x58, HSD_RandF_noinline(), (*yp).x54);
                    (*gp).u.onettcar.state_b = 2;
                    return;
                }
                2 => {
                    let t = (*gp).u.onettcar.timer_b;
                    if t != 0 {
                        (*gp).u.onettcar.timer_b = t.wrapping_sub(1);
                        return;
                    }
                    (*gp).u.onettcar.timer_b = 0;
                    ext::HSD_JObjClearFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                    (*gp).u.onettcar.sub_state_b = 0;
                    (*gp).u.onettcar.state_b = 5;
                    ext::Item_80268E5C((*gp).u.onettcar.car_items[next as usize], 0, ITEM_ANIM_UPDATE as _);
                    return;
                }
                5 => {
                    HSD_JObjAddTranslationX(&mut *car_jobj, (*gp).u.onettcar.speed_b, &mut Dirty);
                    match (*gp).u.onettcar.sub_state_b {
                        0 => {
                            if HSD_JObjGetTranslationX(&*car_jobj) >= -513.0 {
                                grOnett_801E5194(gp, (*gp).u.onettcar.next_car as i32, 4);
                                (*gp).u.onettcar.sub_state_b = 4;
                            }
                        }
                        _ => {}
                    }
                    if HSD_JObjGetTranslationX(&*car_jobj) >= 726.0 {
                        ext::HSD_JObjSetFlagsAll(car_jobj, JOBJ_HIDDEN as u32);
                        (*gp).u.onettcar.state_b = 0;
                    }
                }
                _ => {}
            }
        }
    }
}

pub unsafe extern "C" fn grOnett_801E502C(_gobj: *mut HSD_GObj) { melee_core::ct!("grOnett_801E502C");}

pub unsafe extern "C" fn grOnett_801E5030(_igobj: *mut HSD_GObj, gp: *mut Ground) {
    melee_core::ct!("grOnett_801E5030");
    if (*gp).u.onettcar.state_a as i32 == 4 {
        let s: i32 = if ext::HSD_Randi(2) != 0 { 0x5F37C } else { 0x5F37D };
        ext::Ground_801C53EC(s as u32);
    } else if ext::HSD_Randi(2) != 0 {
        let rand = ext::HSD_Randf();
        (*gp).u.onettcar.car_speed += 0.5 + rand;
        if abs_macro((*gp).u.onettcar.car_speed) < 0.5 {
            (*gp).u.onettcar.car_speed = 0.5;
        }
        (*gp).u.onettcar.state_a = 4;
        (*gp).u.onettcar.x110 = 4;
        (*gp).u.onettcar.x108 = 1;
        let sfx: u32 = if ext::HSD_Randi(2) != 0 { 0x5F37C } else { 0x5F37D };
        ext::Ground_801C5544(gp, 5);
        ext::Ground_801C53EC(sfx);
    } else {
        (*gp).u.onettcar.state_a = 6;
    }
}

pub unsafe extern "C" fn grOnett_801E5140(idx: i32) {
    melee_core::ct!("grOnett_801E5140");
    let table: [u32; 4] = [0x5F370, 0x5F370, 0x5F371, 0x5F372];
    ext::Ground_801C53EC(table[idx as usize]);
}

/// `grOnett_SfxLookup` (file-local inline)
pub unsafe fn grOnett_SfxLookup(table: *mut c_void, idx: i32, ty: i32) -> i32 {
    melee_core::ct!("grOnett_SfxLookup");
    (*(table as *const [i32; 2]).add(idx as usize))[(ty - 4) as usize]
}

pub unsafe extern "C" fn grOnett_801E5194(gp: *mut Ground, car_idx: i32, sound_type: i32) {
    melee_core::ct!("grOnett_801E5194");
    let sfx_table: [[i32; 2]; 4] = [
        [0x5F377, 0x5F376],
        [0x5F377, 0x5F376],
        [0x5F379, 0x5F378],
        [0x5F37B, 0x5F37A],
    ];
    let tmp = gp;
    ext::Ground_801C5440(tmp, sound_type, grOnett_SfxLookup(sfx_table.as_ptr() as *mut c_void, car_idx, sound_type) as u32);
}

pub unsafe extern "C" fn grOnett_801E5214(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E5214");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    for i in 0..2usize {
        let a = &mut (*gp).u.onett.awnings[i];

        if a.accumulator < -(*yp).max_displacement {
            a.accumulator = -(*yp).max_displacement;
        }

        let disp = a.accumulator;
        let error = disp - a.velocity;
        let mut force = a.initial;
        let spring = (*yp).spring_constant;
        force += spring;

        let mut abs_ratio = abs_macro(error);
        abs_ratio /= (*yp).max_displacement;

        if error < 0.0 {
            // -(spring_force * abs_ratio - force): fused as fmuladd(spring_force, abs_ratio, -force)
            force = -fmadds((*yp).spring_force, abs_ratio, -force);
        } else {
            force = fmadds((*yp).spring_force, abs_ratio, force);
        }

        force -= spring;
        force *= 1.0 - (*yp).damping;

        if force > (*yp).max_velocity {
            force = (*yp).max_velocity;
        } else if force < -(*yp).max_velocity {
            force = -(*yp).max_velocity;
        }

        let vel = a.velocity;
        if abs_macro(vel - disp) < (*yp).pos_threshold && abs_macro(force) < (*yp).vel_threshold {
            a.velocity = disp;
            a.initial = 0.0;
        } else {
            a.initial = force;
            a.velocity += force;
            let lim = (*yp).max_displacement;
            if a.velocity > lim {
                a.velocity = lim;
            } else if a.velocity < -lim {
                a.velocity = -lim;
            }
        }

        let jobj = a.jobj;
        HSD_JObjSetTranslateY(&mut *jobj, a.velocity + (a.initial_y + a.accumulator), &mut Dirty);

        if a.cooldown == 0 && (a.counter_prev != a.counter || a.flag != 0) {
            ext::grLib_801C9808(0x7533, 0x1E, a.jobj);
            ext::Ground_801C5440(gp, 0, 0x5F375);
            a.cooldown = 0xA;
        }

        a.accumulator = 0.0;
        a.counter_prev = a.counter;
        a.counter = 0;
        a.flag = 0;

        if a.cooldown != 0 {
            a.cooldown = a.cooldown.wrapping_sub(1);
        }
    }
}

/// `mpLib_JointCollisionCallback` of the awnings
pub unsafe extern "C" fn grOnett_801E54B4(user_data: *mut c_void, joint_id: i32, coll: *mut CollData, coll_x50: i32, ground_kind: u32, delta_y: f32) {
    melee_core::ct!("grOnett_801E54B4");
    let gp = user_data as *mut Ground;
    let yp = yakumono_param;
    let temp = (*coll).x34_flags.b1234() as i32;

    if temp != 1 && temp != 2 && temp != 3 {
        return;
    }

    let idx = if joint_id == 0 { 0usize } else { 1usize };
    if ground_kind == 1 {
        (*gp).u.onett.awnings[idx].flag = 1;
        (*gp).u.onett.awnings[idx].initial = (*yp).awning_initial;
    }

    (*gp).u.onett.awnings[idx].accumulator += (*yp).awning_delta;
    (*gp).u.onett.awnings[idx].counter = (*gp).u.onett.awnings[idx].counter.wrapping_add(1);
}

pub unsafe extern "C" fn grOnett_801E5538(gobj: *mut HSD_GObj) {
    melee_core::ct!("grOnett_801E5538");
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let new_var2 = (*gp).u.onett.timer;
    (*gp).u.onett.timer = new_var2.wrapping_sub(1);
    let new_var = new_var2;
    if new_var < 0 {
        if !(*gp).u.onett.gen.is_null() {
            (*gp).u.onett.timer = rand_range(f2i32((*yp).x28 as f64), f2i32((*yp).x24 as f64));
            ext::grLib_801C9834((*gp).u.onett.gen);
            (*gp).u.onett.gen = null_mut();
            return;
        }
        (*gp).u.onett.timer = rand_range(f2i32((*yp).x30 as f64), f2i32((*yp).x2C as f64));
        {
            let jobj = ext::Ground_801C3FA4(gobj, 6);
            if !jobj.is_null() {
                let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                let mut rot = Vec3 { x: 15.75, y: 40.0, z: 28.25 };
                ext::lb_8000B1CC(jobj, &mut rot, &mut pos);
                (*gp).u.onett.gen = ext::grLib_801C96F8(0x7530, 0x1E, &mut pos);
            }
        }
    }
}

pub unsafe extern "C" fn grOnett_801E56FC() -> *mut DynamicModelDesc {
    melee_core::ct!("grOnett_801E56FC");
    let archive = ext::grDatFiles_801C6330(0x1);
    assert!(!archive.is_null(), "HSD_ASSERT(1319, archive)");
    let dat = (*archive).unk4;
    if !dat.is_null() {
        return (*dat).unk8.add(1) as *mut DynamicModelDesc;
    }
    null_mut()
}

pub unsafe extern "C" fn grOnett_801E5760(_arg: i32) -> *mut DynamicsDesc {
    melee_core::ct!("grOnett_801E5760");
    null_mut()
}

pub unsafe extern "C" fn grOnett_801E5768(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    melee_core::ct!("grOnett_801E5768");
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}
