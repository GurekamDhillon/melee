//! Port of src/melee/gr/grfigure1.c: the Figure 1 target-bonus stage ("/GrEF1.dat"): three stage
//! gobjs (0: model with an animation, 1: the collision map, 2: collision map plus animation),
//! no yakumono-driven behaviour. Callees of unported TUs go through `ext`; the constant-return
//! callbacks are the earlier round's ports (`gr::trivial`), re-exported. Checked against the C
//! (whole TU, shipping contraction) by tests/tu_grfigure1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::grdisplay::grDisplay_801C5DB0;
pub use crate::gr::trivial::grfigure1::{
    grFigure1_8020E078, grFigure1_8020E194, grFigure1_8020E1D0, grFigure1_8020E250, grFigure1_OnCheckShadowRender,
    grFigure1_OnTouchLine,
};

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `const fn` for a `StageCallbacks` table row: `{ on_init, callback1, gobj_proc, callback3, flags }`.
const fn row(
    on_init: HSD_GObjEvent,
    callback1: HSD_GObjPredicate,
    gobj_proc: HSD_GObjEvent,
    callback3: HSD_GObjEvent,
    flags: u32,
) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags } }
}

/// `static StageCallbacks grEF1_StageCallbacks[3]`
pub static mut grEF1_StageCallbacks: [StageCallbacks; 3] = [
    row(Some(stageGObj0_OnInit), Some(grFigure1_8020E194), Some(grFigure1_8020E19C), Some(grFigure1_8020E1A0), 0),
    row(Some(grFigure1_8020E1A4), Some(grFigure1_8020E1D0), Some(grFigure1_8020E1D8), Some(grFigure1_8020E1FC), (1 << 30) | (1 << 31)),
    row(Some(stageGObj2_OnInit), Some(grFigure1_8020E250), Some(grFigure1_8020E258), Some(grFigure1_8020E25C), 0),
];

/// `StageData grEF1_StageData` (3E62C0)
pub static mut grEF1_StageData: StageData = StageData {
    grkind: Gr_Kind_Figure1,
    callbacks: unsafe { &raw mut grEF1_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrEF1.dat".as_ptr() as *mut i8,
    on_init: Some(grFigure1_OnInit),
    on_demo_init: Some(grFigure1_OnDemoInit),
    on_load: Some(grFigure1_OnLoad),
    on_start: Some(grFigure1_OnStart),
    callback4: Some(grFigure1_8020E078),
    on_touch_line: Some(grFigure1_OnTouchLine),
    on_check_shadow_render: Some(grFigure1_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `static void* yakumono_param`
pub static mut yakumono_param: *mut c_void = null_mut();

/// `Ground_SetupStageCallbacks` (gr/inlines.h, TARGET_PC arm): binds the row's callbacks (the
/// m-ex `Mex_GrBind` passes native pointers through) and runs `on_init`.
#[inline]
unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::GObj_SetupGXLink(gobj, Some(grDisplay_801C5DB0), 3, 0);
    if (*callbacks).callback3.is_some() {
        (*gp).x1C_callback = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
            core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).callback3),
        ));
    }
    if (*callbacks).on_init.is_some() {
        let f = core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
            core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).on_init),
        ));
        f.expect("on_init")(gobj);
    }
    if (*callbacks).gobj_proc.is_some() {
        ext::HSD_GObj_SetupProc(
            gobj,
            core::mem::transmute::<*mut c_void, HSD_GObjEvent>(ext::Mex_GrBind(
                core::mem::transmute::<HSD_GObjEvent, *mut c_void>((*callbacks).gobj_proc),
            )),
            4,
        );
    }
}

/// `Ground_StartMapAnim` (gr/inlines.h)
#[inline]
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `Ground_InitMapCollAndAnim` (gr/inlines.h)
#[inline]
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// 20DFDC
pub unsafe extern "C" fn grFigure1_OnDemoInit(_unused: i32) {}

/// 20DFE0
pub unsafe extern "C" fn grFigure1_OnInit() {
    yakumono_param = ext::Ground_GetYakumonoParam();
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grFigure1_8020E080(0);
    grFigure1_8020E080(2);
    grFigure1_8020E080(1);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// 20E050
pub unsafe extern "C" fn grFigure1_OnLoad() {}

/// 20E054
pub unsafe extern "C" fn grFigure1_OnStart() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 20E080: sets up stage gobj `gobj_id` with its row of the callback table.
pub unsafe extern "C" fn grFigure1_8020E080(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = &raw mut grEF1_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    // else OSReport("%s:%d: couldn t get gobj(id=%d)\n", ...): diagnostics only
    gobj
}

/// 20E168
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    Ground_StartMapAnim(gobj);
}

/// 20E19C
pub unsafe extern "C" fn grFigure1_8020E19C(_gobj: *mut HSD_GObj) {}

/// 20E1A0
pub unsafe extern "C" fn grFigure1_8020E1A0(_gobj: *mut HSD_GObj) {}

/// 20E1A4
pub unsafe extern "C" fn grFigure1_8020E1A4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
}

/// 20E1D8
pub unsafe extern "C" fn grFigure1_8020E1D8(gobj: *mut HSD_GObj) {
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// 20E1FC
pub unsafe extern "C" fn grFigure1_8020E1FC(_gobj: *mut HSD_GObj) {}

/// 20E200
pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    Ground_InitMapCollAndAnim(gobj);
}

/// 20E258
pub unsafe extern "C" fn grFigure1_8020E258(_gobj: *mut HSD_GObj) {}

/// 20E25C
pub unsafe extern "C" fn grFigure1_8020E25C(_gobj: *mut HSD_GObj) {}

/// 20E270 (the non-static one; grfigure2.c stores its address as its demo-init)
pub unsafe extern "C" fn grFigure1_8020E270(_n: i32) {}
