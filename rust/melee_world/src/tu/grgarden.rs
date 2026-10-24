//! Port of src/melee/gr/grgarden.c (Kongo Jungle 64 / "Jungle Japes": the stage callbacks table,
//! the water current, Klaptrap and Cranky Kong). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_grgarden.rs. Float arithmetic: `srt->scale.* *= param1`, `dVar1 += iVar1`
//! and the water test, all plain f32 (no fused sites).
//!
//! The constant-return and empty callbacks that `gr/trivial.rs` already holds are re-exported.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::inline::*;
use melee_types::*;

pub use crate::gr::trivial::grgarden::{
    grGarden_80202C70, grGarden_80202D8C, grGarden_80202E40, grGarden_80202F34, grGarden_80202F94, grGarden_80202FF4,
    grGarden_80203090, grGarden_80203248, grGarden_802035C8,
};

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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Ground_801C10B8(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>| { unsafe { let _ = crate::tu::ground::Ground_801C10B8(a0 as _, a1 as _); } };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4E70(a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj) = |a0: *mut HSD_JObj, a1: *mut HSD_JObj, a2: *mut HSD_JObj, a3: *mut HSD_JObj, a4: *mut HSD_JObj, a5: *mut HSD_JObj| { unsafe { let _ = crate::tu::ground::Ground_801C4E70(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn Ground_801C53EC(a0: u32) = |a0: u32| { unsafe { let _ = crate::tu::ground::Ground_801C53EC(a0 as _); } };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_JObjRemoveAnimAll(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn Stage_80225194() -> u32 = || unsafe { crate::tu::stage::Stage_80225194() as _ };
    fn ftCo_800C06E8(a0: *mut HSD_GObj, a1: i32, a2: *mut c_void);
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086684(a0: *mut HSD_GObj, a1: *mut Vec);
    fn ftLib_80086B80(a0: *mut HSD_GObj) -> f32;
    fn gm_8016B238() -> i32;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grAnime_801C84A4(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C84A4(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grLib_801C96F8(a0: i32, a1: i32, a2: *mut Vec) -> *mut HSD_Generator = |a0: i32, a1: i32, a2: *mut Vec| unsafe { crate::tu::grlib::grLib_801C96F8(a0 as _, a1 as _, a2 as _) as _ };
    fn grMaterial_801C8CFC(a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>) -> *mut HSD_GObj = |a0: i32, a1: i32, a2: *mut Ground, a3: *mut HSD_JObj, a4: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground)>, a5: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut Vec, *mut HSD_GObj, f32)>, a6: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>| unsafe { crate::tu::grmaterial::grMaterial_801C8CFC(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _) as _ };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
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

/// `struct grGarden_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grGarden_YakumonoParam {
    pub x0: f32,
    pub x4: f32,
    pub x8: i32,
    pub xC: i32,
    pub x10: i32,
    pub x14: i32,
    pub x18: f32,
    pub x1C: f32,
}

const NOCB: StageCallbacks = unsafe { core::mem::zeroed() };
const fn cbs(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent,
             callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    let mut c = NOCB;
    c.on_init = on_init;
    c.callback1 = callback1;
    c.gobj_proc = gobj_proc;
    c.callback3 = callback3;
    c.anon_10 = StageCallbacks_anon_135 { flags };
    c
}

/// `grGd_StageCallbacks` (3E52xx): cranky kong, main stage, klaptrap, background, birds, water.
pub static mut grGd_StageCallbacks: [StageCallbacks; 7] = [
    cbs(Some(stageGObj0_OnInit), Some(grGarden_80202D8C), Some(grGarden_80202D94), Some(grGarden_80202D98), 0),
    cbs(Some(grGarden_80203004), Some(grGarden_80203090), Some(grGarden_80203098), Some(grGarden_802031A0), 0), // cranky kong
    cbs(Some(grGarden_80202DC8), Some(grGarden_80202E40), Some(grGarden_80202E48), Some(grGarden_80202E68), 0xC0000000), // main stage
    cbs(Some(grGarden_802031A4), Some(grGarden_80203248), Some(grGarden_80203250), Some(grGarden_80203494), 0), // klaptrap
    cbs(Some(grGarden_80202E6C), Some(grGarden_80202F34), Some(grGarden_80202F3C), Some(grGarden_80202F40), 0), // background
    cbs(Some(grGarden_80202F44), Some(grGarden_80202F94), Some(grGarden_80202F9C), Some(grGarden_80202FA0), 0), // birds
    cbs(Some(grGarden_80202FA4), Some(grGarden_80202FF4), Some(grGarden_80202FFC), Some(grGarden_80203000), 0), // water
];

/// `grGd_StageData` (3E52E0)
pub static mut grGd_StageData: StageData = StageData {
    grkind: Gr_Kind_Garden,
    callbacks: unsafe { &raw mut grGd_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrGd.dat\0".as_ptr() as *mut i8,
    on_init: Some(grGarden_80202B70),
    on_demo_init: Some(grGarden_80202B6C),
    on_load: Some(grGarden_80202C24),
    on_start: Some(grGarden_80202C4C),
    callback4: Some(grGarden_80202C70),
    on_touch_line: Some(grGarden_802035C8),
    on_check_shadow_render: Some(grGarden_802035D0),
    flags2: 1,
    joints: null_mut(),
    joint_count: 0,
};

/// `static struct grGarden_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grGarden_YakumonoParam = null_mut();

#[inline(always)]
unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data as *mut Ground
}

/// `static inline Ground_SetupStageCallbacks` (gr/inlines.h, TARGET_PC form: every callback goes
/// through `Mex_GrBind`).
unsafe fn Ground_SetupStageCallbacks(gobj: *mut Ground_GObj, callbacks: *mut StageCallbacks) {
    let gp = GET_GROUND(gobj);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        let f = ext::Mex_GrBind((*callbacks).callback3.map_or(null_mut(), |f| f as *mut c_void));
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(f);
    }
    if (*callbacks).on_init.is_some() {
        let f = ext::Mex_GrBind((*callbacks).on_init.map_or(null_mut(), |f| f as *mut c_void));
        core::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut HSD_GObj)>(f)(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        let f = ext::Mex_GrBind((*callbacks).gobj_proc.map_or(null_mut(), |f| f as *mut c_void));
        ext::HSD_GObj_SetupProc(gobj, core::mem::transmute::<*mut c_void, HSD_GObjEvent>(f), 4);
    }
}

/// `static inline Ground_StartMapAnim` (gr/inlines.h)
unsafe fn Ground_StartMapAnim(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 202B6C
pub unsafe extern "C" fn grGarden_80202B6C(_arg: i32) {}

/// 202B70
pub unsafe extern "C" fn grGarden_80202B70() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grGarden_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grGarden_80202C78(0);
    grGarden_80202C78(4);
    grGarden_80202C78(5);
    grGarden_80202C78(6);
    grGarden_80202C78(1);
    grGarden_80202C78(3);
    grGarden_80202C78(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    if ext::Stage_80225194() != 62 && ext::gm_8016B238() == 0 {
        ext::Ground_801C53EC(0x64194);
    }
}

/// 202C24
pub unsafe extern "C" fn grGarden_80202C24() {
    ext::Ground_801C53EC(0x64193);
}

/// 202C4C
pub unsafe extern "C" fn grGarden_80202C4C() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 202C78
pub unsafe extern "C" fn grGarden_80202C78(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = &raw mut grGd_StageCallbacks[gobj_id as usize];

    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    } else {
        // OSReport("%s:%d: couldn t get gobj(id=%d)\n", __FILE__, 0xE3, gobj_id);
    }

    gobj
}

/// `static stageGObj0_OnInit`
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut Ground_GObj) {
    Ground_StartMapAnim(gobj);
}

/// 202D94
pub unsafe extern "C" fn grGarden_80202D94(_arg: *mut Ground_GObj) {}

/// 202D98
pub unsafe extern "C" fn grGarden_80202D98(_arg: *mut Ground_GObj) {}

/// 202D9C
pub unsafe extern "C" fn grGarden_80202D9C(arg: *mut Ground_GObj) {
    ext::ftCo_800C06E8(arg, 4, grGarden_8020349C as *mut c_void);
}

/// 202DC8
pub unsafe extern "C" fn grGarden_80202DC8(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::Ground_801C10B8(gobj, Some(grGarden_80202D9C));
    (*gp).x11_flags.set_b012(1);
    (*gp).x10_flags.set_b5(1);
}

/// 202E48
pub unsafe extern "C" fn grGarden_80202E48(_arg: *mut Ground_GObj) {
    ext::lb_800115F4();
}

/// 202E68
pub unsafe extern "C" fn grGarden_80202E68(_arg: *mut Ground_GObj) {}

/// 202E6C
pub unsafe extern "C" fn grGarden_80202E6C(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(1);
    let a = ext::Ground_801C3FA4(gobj, 4);
    let b = ext::Ground_801C3FA4(gobj, 3);
    let c = ext::Ground_801C3FA4(gobj, 7);
    let d = ext::Ground_801C3FA4(gobj, 6);
    let e = ext::Ground_801C3FA4(gobj, 10);
    let f = ext::Ground_801C3FA4(gobj, 9);
    ext::Ground_801C4E70(a, b, c, d, e, f);
}

/// 202F3C
pub unsafe extern "C" fn grGarden_80202F3C(_arg: *mut Ground_GObj) {}

/// 202F40
pub unsafe extern "C" fn grGarden_80202F40(_arg: *mut Ground_GObj) {}

/// 202F44
pub unsafe extern "C" fn grGarden_80202F44(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(1);
}

/// 202F9C
pub unsafe extern "C" fn grGarden_80202F9C(_arg: *mut Ground_GObj) {}

/// 202FA0
pub unsafe extern "C" fn grGarden_80202FA0(_arg: *mut Ground_GObj) {}

/// 202FA4
pub unsafe extern "C" fn grGarden_80202FA4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(2);
}

/// 202FFC
pub unsafe extern "C" fn grGarden_80202FFC(_arg: *mut Ground_GObj) {}

/// 203000
pub unsafe extern "C" fn grGarden_80203000(_arg: *mut Ground_GObj) {}

/// 203004
pub unsafe extern "C" fn grGarden_80203004(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    (*gp).u.garden.xc4 = 0;
    let mut iVar1 = (*yakumono_param).x8;
    iVar1 = if iVar1 != 0 { ext::HSD_Randi(iVar1) } else { 0 };
    (*gp).u.garden.xc8 = iVar1.wrapping_add(1);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*gp).u.garden.xc4);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(1);
}

/// 203098
pub unsafe extern "C" fn grGarden_80203098(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 || ext::grAnime_801C84A4(gobj, 0, 7) != 0 {
        if (*gp).u.garden.xc8 == 0 {
            (*gp).u.garden.xc4 = (*gp).u.garden.xc4.wrapping_add(1);
            if 3 < (*gp).u.garden.xc4 {
                (*gp).u.garden.xc4 = 0;
            }
            if (*gp).u.garden.xc4 == 0 {
                let iVar1 = if (*yakumono_param).x8 != 0 { ext::HSD_Randi((*yakumono_param).x8) } else { 0 };
                (*gp).u.garden.xc8 = iVar1.wrapping_add(1);
            } else if (*gp).u.garden.xc4 == 2 {
                let iVar1 = if (*yakumono_param).xC != 0 { ext::HSD_Randi((*yakumono_param).xC) } else { 0 };
                (*gp).u.garden.xc8 = iVar1.wrapping_add(1);
            }
            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, (*gp).u.garden.xc4);
        } else {
            (*gp).u.garden.xc8 = (*gp).u.garden.xc8.wrapping_sub(1);
        }
    }
}

/// 2031A0
pub unsafe extern "C" fn grGarden_802031A0(_arg: *mut Ground_GObj) {}

/// 2031A4
pub unsafe extern "C" fn grGarden_802031A4(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let j = ext::Ground_801C3FA4(gobj, 8);
    (*gp).u.garden2.xc4 = ext::grMaterial_801C8CFC(0, 0, gp, j, Some(grGarden_80203498), None, None);
    let iVar1 = if (*yakumono_param).x14 != 0 { ext::HSD_Randi((*yakumono_param).x14) } else { 0 };
    (*gp).u.garden2.xcc = (*yakumono_param).x10.wrapping_add(iVar1);
    (*gp).u.garden2.xc8 = 0;
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    (*gp).x11_flags.set_b012(1);
}

/// 203250
pub unsafe extern "C" fn grGarden_80203250(gobj: *mut Ground_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = ext::Ground_801C3FA4(gobj, 0);

    match (*gp).u.garden2.xc8 {
        0 => {
            if (*gp).u.garden2.xcc == 0 {
                ext::Item_80268E5C((*gp).u.garden2.xc4, 2, 2);
                ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
                let mut dVar1: f32 = (*yakumono_param).x18;
                let d = (*yakumono_param).x1C - (*yakumono_param).x18;
                let mut iVar1: i32 = (if d < 0.0 { -((*yakumono_param).x1C - (*yakumono_param).x18) } else { (*yakumono_param).x1C - (*yakumono_param).x18 }) as i32;

                iVar1 = if iVar1 != 0 { ext::HSD_Randi(iVar1) } else { 0 };
                dVar1 += iVar1 as f32;
                HSD_JObjSetTranslateX(&mut *jobj, dVar1, &mut Dirty);
                (*gp).u.garden2.xcc = 0;
                (*gp).u.garden2.xc8 = 1;
            } else {
                (*gp).u.garden2.xcc = (*gp).u.garden2.xcc.wrapping_sub(1);
            }
        }
        1 => {
            let iVar4 = (*gp).u.garden2.xcc;
            if iVar4 == 38 || iVar4 == 148 {
                ext::Ground_801C53EC(410001);
            } else if iVar4 == 110 || iVar4 == 210 {
                ext::Ground_801C53EC(410002);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                ext::Item_80268E5C((*gp).u.garden2.xc4, 0, 2);
                ext::HSD_JObjRemoveAnimAll(jobj);
                let iVar1 = if (*yakumono_param).x14 != 0 { ext::HSD_Randi((*yakumono_param).x14) } else { 0 };
                (*gp).u.garden2.xcc = (*yakumono_param).x10.wrapping_add(iVar1);
                (*gp).u.garden2.xc8 = 0;
            }
            (*gp).u.garden2.xcc = (*gp).u.garden2.xcc.wrapping_add(1);
        }
        _ => {}
    }
}

/// 203494
pub unsafe extern "C" fn grGarden_80203494(_arg: *mut Ground_GObj) {}

/// 203498
pub unsafe extern "C" fn grGarden_80203498(_item_gobj: *mut Item_GObj, _gp: *mut Ground) {}

/// 20349C
pub unsafe extern "C" fn grGarden_8020349C(_unk: u32, player: *mut HSD_GObj, water: *mut Vec3) -> i32 {
    let mut current = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut previous = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dVar3: f32 = (*yakumono_param).x0;
    ext::ftLib_80086644(player, &mut current);
    ext::ftLib_80086684(player, &mut previous);
    let uVar1;
    if current.y < dVar3 {
        if previous.y > dVar3 {
            dVar3 = ext::ftLib_80086B80(player) / 10.0;
            current.y = -30.0;
            grGarden_80203560(dVar3, &mut current);
            ext::Ground_801C53EC(410000);
        }
        uVar1 = true;
        (*water).x = (*yakumono_param).x4;
        (*water).y = 0.0;
        (*water).z = 0.0;
    } else {
        uVar1 = false;
    }
    uVar1 as i32
}

/// 203560
pub unsafe extern "C" fn grGarden_80203560(param1: f32, vec: *mut Vec3) {
    let gen = ext::grLib_801C96F8(0x753A, 0x1E, vec);
    if !gen.is_null() {
        let srt = (*gen).appsrt;
        (*srt).scale.x *= param1;
        (*srt).scale.y *= param1;
        (*srt).scale.z *= param1;
    }
}

/// 2035D0
pub unsafe extern "C" fn grGarden_802035D0(a: *mut Vec3, _b: i32, jobj: *mut HSD_JObj) -> i32 {
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(jobj, null_mut(), &mut vec);
    if (*a).y > vec.y {
        1
    } else {
        0
    }
}

/// 203624
pub unsafe extern "C" fn grGarden_80203624() -> f32 {
    -20.0
}
