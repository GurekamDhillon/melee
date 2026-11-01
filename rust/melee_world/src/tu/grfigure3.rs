//! Port of src/melee/gr/grfigure3.c: the Figure 3 target-bonus stage ("/GrEF3.dat"), same shape as
//! grfigure1.c: three stage gobjs (0: model with an animation, 1: the collision map, 2: collision
//! map plus animation), no yakumono-driven behaviour. Callees of unported TUs go through `ext`;
//! the constant-return callbacks are the earlier round's ports (`gr::trivial`), re-exported.
//! Checked against the C (whole TU, shipping contraction) by tests/tu_grfigure3.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::gr::trivial::*;
#[allow(unused_imports)]
use crate::gr::forward::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::grdisplay::grDisplay_801C5DB0;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lb_800115F4() = || { unsafe { melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::ground::stage_info;
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
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags: u32::to_be(flags) } }
}

/// `static StageCallbacks grEF2_StageCallbacks[3]`
pub static mut grEF2_StageCallbacks: [StageCallbacks; 3] = [
    row(Some(stageGObj0_OnInit), Some(grFigure3_8020E6BC), Some(grFigure3_8020E6C4), Some(grFigure3_8020E6C8), 0),
    row(Some(grFigure3_8020E6CC), Some(grFigure3_8020E6F8), Some(grFigure3_8020E700), Some(grFigure3_8020E724), 0xC0000000),
    row(Some(stageGObj2_OnInit), Some(grFigure3_8020E778), Some(grFigure3_8020E780), Some(grFigure3_8020E784), 0),
];

/// `StageData grEF3_StageData`
pub static mut grEF3_StageData: StageData = StageData {
    grkind: Gr_Kind_Figure3,
    callbacks: unsafe { &raw mut grEF2_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrEF3.dat".as_ptr() as *mut i8,
    on_init: Some(grFigure3_8020E508),
    on_demo_init: Some(grFigure3_8020E504),
    on_load: Some(grFigure3_OnLoad),
    on_start: Some(grFigure3_OnStart),
    callback4: Some(grFigure3_8020E5A0),
    on_touch_line: Some(grFigure3_8020E788),
    on_check_shadow_render: Some(grFigure3_8020E790),
    flags2: 0x00000001,
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

/// grFigure3_8020E504
pub unsafe extern "C" fn grFigure3_8020E504(_number: i32) { melee_core::ct!("grFigure3_8020E504");}

/// grFigure3_8020E508
pub unsafe extern "C" fn grFigure3_8020E508() {
    melee_core::ct!("grFigure3_8020E508");
    yakumono_param = ext::Ground_GetYakumonoParam();
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grFigure3_8020E5A8(0);
    grFigure3_8020E5A8(2);
    grFigure3_8020E5A8(1);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// grFigure3_OnLoad
pub unsafe extern "C" fn grFigure3_OnLoad() { melee_core::ct!("grFigure3_OnLoad");}

/// grFigure3_OnStart
pub unsafe extern "C" fn grFigure3_OnStart() {
    melee_core::ct!("grFigure3_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// grFigure3_8020E5A8
pub unsafe extern "C" fn grFigure3_8020E5A8(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grFigure3_8020E5A8");
    let callbacks = &raw mut grEF2_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    // else OSReport("%s:%d: couldn t get gobj(id=%d)\n", ...): diagnostics only
    gobj
}

/// stageGObj0_OnInit
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

/// grFigure3_8020E6C4
pub unsafe extern "C" fn grFigure3_8020E6C4(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure3_8020E6C4");}

/// grFigure3_8020E6C8
pub unsafe extern "C" fn grFigure3_8020E6C8(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure3_8020E6C8");}

/// grFigure3_8020E6CC
pub unsafe extern "C" fn grFigure3_8020E6CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFigure3_8020E6CC");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
}

/// grFigure3_8020E700
pub unsafe extern "C" fn grFigure3_8020E700(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFigure3_8020E700");
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// grFigure3_8020E724
pub unsafe extern "C" fn grFigure3_8020E724(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure3_8020E724");}

/// stageGObj2_OnInit
pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    Ground_InitMapCollAndAnim(gobj);
}

/// grFigure3_8020E780
pub unsafe extern "C" fn grFigure3_8020E780(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure3_8020E780");}

/// grFigure3_8020E784
pub unsafe extern "C" fn grFigure3_8020E784(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure3_8020E784");}

// ---- folded from grfigure3.rs (consolidation steps 21/22) ----
/// grFigure3_OnStart_fp (gr/grfigure3.c:78)
    pub unsafe extern "C" fn grFigure3_OnStart_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

// ---- folded from grfigure3.rs (consolidation steps 21/22) ----
/// grFigure3_8020E5A0 (gr/grfigure3.c:83)
    pub extern "C" fn grFigure3_8020E5A0() -> i32 { 0 }

/// grFigure3_8020E6BC (gr/grfigure3.c:109)
    pub extern "C" fn grFigure3_8020E6BC(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure3_8020E6F8 (gr/grfigure3.c:124)
    pub extern "C" fn grFigure3_8020E6F8(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure3_8020E778 (gr/grfigure3.c:142)
    pub extern "C" fn grFigure3_8020E778(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure3_8020E788 (gr/grfigure3.c:151)
    pub extern "C" fn grFigure3_8020E788(a0: i32) -> *mut DynamicsDesc { core::ptr::null_mut() }

/// grFigure3_8020E790 (gr/grfigure3.c:156)
    pub extern "C" fn grFigure3_8020E790(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
