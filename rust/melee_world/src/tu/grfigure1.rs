//! Port of src/melee/gr/grfigure1.c: the Figure 1 target-bonus stage ("/GrEF1.dat"): three stage
//! gobjs (0: model with an animation, 1: the collision map, 2: collision map plus animation),
//! no yakumono-driven behaviour. Callees of unported TUs go through `ext`; the constant-return
//! callbacks are the earlier round's ports (`gr::trivial`), re-exported. Checked against the C
//! (whole TU, shipping contraction) by tests/tu_grfigure1.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
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
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, isize)>, a2: u8, a3: u32| { unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C39C0() = || { unsafe { crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 = |a0: *mut HSD_GObj| unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: isize) = |a0: *mut HSD_GObj, a1: isize| { unsafe { crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
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
    StageCallbacks { on_init, callback1, gobj_proc, callback3, x10: StageCallbacks_x10 { flags: u32::to_be(flags) } }
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
pub unsafe extern "C" fn grFigure1_OnDemoInit(_unused: i32) { melee_core::ct!("grFigure1_OnDemoInit");}

/// 20DFE0
pub unsafe extern "C" fn grFigure1_OnInit() {
    melee_core::ct!("grFigure1_OnInit");
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
pub unsafe extern "C" fn grFigure1_OnLoad() { melee_core::ct!("grFigure1_OnLoad");}

/// 20E054
pub unsafe extern "C" fn grFigure1_OnStart() {
    melee_core::ct!("grFigure1_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// 20E080: sets up stage gobj `gobj_id` with its row of the callback table.
pub unsafe extern "C" fn grFigure1_8020E080(gobj_id: i32) -> *mut HSD_GObj {
    melee_core::ct!("grFigure1_8020E080");
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
    melee_core::ct!("stageGObj0_OnInit");
    Ground_StartMapAnim(gobj);
}

/// 20E19C
pub unsafe extern "C" fn grFigure1_8020E19C(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure1_8020E19C");}

/// 20E1A0
pub unsafe extern "C" fn grFigure1_8020E1A0(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure1_8020E1A0");}

/// 20E1A4
pub unsafe extern "C" fn grFigure1_8020E1A4(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFigure1_8020E1A4");
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
}

/// 20E1D8
pub unsafe extern "C" fn grFigure1_8020E1D8(gobj: *mut HSD_GObj) {
    melee_core::ct!("grFigure1_8020E1D8");
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}

/// 20E1FC
pub unsafe extern "C" fn grFigure1_8020E1FC(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure1_8020E1FC");}

/// 20E200
pub unsafe extern "C" fn stageGObj2_OnInit(gobj: *mut HSD_GObj) {
    melee_core::ct!("stageGObj2_OnInit");
    Ground_InitMapCollAndAnim(gobj);
}

/// 20E258
pub unsafe extern "C" fn grFigure1_8020E258(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure1_8020E258");}

/// 20E25C
pub unsafe extern "C" fn grFigure1_8020E25C(_gobj: *mut HSD_GObj) { melee_core::ct!("grFigure1_8020E25C");}

/// 20E270 (the non-static one; grfigure2.c stores its address as its demo-init)
pub unsafe extern "C" fn grFigure1_8020E270(_n: i32) { melee_core::ct!("grFigure1_8020E270");}

// ---- folded from grfigure1.rs (consolidation steps 21/22) ----
/// grFigure1_OnStart_fp (gr/grfigure1.c:81)
    pub unsafe extern "C" fn grFigure1_OnStart_fp() { let c = c(); c.grzakogenerator_801cae04(core::ptr::null_mut()); }

/// stageGObj0_OnInit_fp (gr/grfigure1.c:107)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp (gr/grfigure1.c:142)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grfigure2.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2 (gr/grfigure2.c:101)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2 (gr/grfigure2.c:134)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grfigure3.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22 (gr/grfigure3.c:104)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22 (gr/grfigure3.c:137)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grfigureget.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222 (gr/grfigureget.c:139)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grflatzone.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222 (gr/grflatzone.c:176)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grfourside.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222 (gr/grfourside.c:181)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222 (gr/grfourside.c:195)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grgarden.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222 (gr/grgarden.c:126)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grgreatbay.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222 (gr/grgreatbay.c:263)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grinishie1.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222 (gr/grinishie1.c:291)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grinishie2.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222 (gr/grinishie2.c:263)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grkraid.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222 (gr/grkraid.c:111)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grlast.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222 (gr/grlast.c:289)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222 (gr/grlast.c:319)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from groldkongo.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222 (gr/groldkongo.c:189)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from groldpupupu.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222 (gr/groldpupupu.c:211)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from groldyoshi.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222 (gr/groldyoshi.c:115)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grshrine.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222 (gr/grshrine.c:118)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grshrineroute.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222 (gr/grshrineroute.c:247)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtcaptain.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222 (gr/grtcaptain.c:112)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222 (gr/grtcaptain.c:126)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtclink.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp222222 (gr/grtclink.c:108)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtdonkey.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222 (gr/grtdonkey.c:106)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222222 (gr/grtdonkey.c:120)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtdrmario.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222 (gr/grtdrmario.c:110)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222 (gr/grtdrmario.c:124)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtemblem.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222 (gr/grtemblem.c:109)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222 (gr/grtemblem.c:123)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtest.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222 (gr/grtest.c:106)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222222222 (gr/grtest.c:120)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtfalco.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp22222222222 (gr/grtfalco.c:138)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtfox.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222 (gr/grtfox.c:101)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222 (gr/grtfox.c:115)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtgamewatch.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp2222222222222 (gr/grtgamewatch.c:104)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtganon.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp22222222222222 (gr/grtganon.c:114)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtkirby.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222 (gr/grtkirby.c:111)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222222 (gr/grtkirby.c:125)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtkoopa.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222222 (gr/grtkoopa.c:111)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222222222222222 (gr/grtkoopa.c:124)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtlink.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222222 (gr/grtlink.c:111)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222222222222 (gr/grtlink.c:125)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtluigi.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222222 (gr/grtluigi.c:89)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222222222 (gr/grtluigi.c:103)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtmario.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222222222 (gr/grtmario.c:117)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtmars.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222222222 (gr/grtmars.c:104)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222222222222222222 (gr/grtmars.c:118)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtmewtwo.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222222222 (gr/grtmewtwo.c:133)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222222222222222 (gr/grtmewtwo.c:147)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtness.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222222222222 (gr/grtness.c:112)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222222222222 (gr/grtness.c:132)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtpeach.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp2222222222222222222222 (gr/grtpeach.c:134)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtpichu.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222222222222 (gr/grtpichu.c:110)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222222222222222222 (gr/grtpichu.c:124)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtpikachu.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp222222222222222222222222 (gr/grtpikachu.c:103)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtpurin.rs (consolidation steps 21/22) ----
/// stageGObj2_OnInit_fp2222222222222222222222222 (gr/grtpurin.c:111)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtsamus.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222222222222 (gr/grtsamus.c:108)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222222222222222222222 (gr/grtsamus.c:122)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtseak.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222222222222222 (gr/grtseak.c:169)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222222222222222222 (gr/grtseak.c:183)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtyoshi.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222222222222222 (gr/grtyoshi.c:111)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp2222222222222222222222222222 (gr/grtyoshi.c:125)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp2222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grtzelda.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222222222222222 (gr/grtzelda.c:95)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp22222222222222222222222222222 (gr/grtzelda.c:109)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp22222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; let j = (*a0).hsd_obj as *mut HSD_JObj; c.ground_init_map_coll(j, (*gp).map_id as i32); c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grvenom.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp222222222222222222222222222222222222 (gr/grvenom.c:607)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp222222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from gryorster.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp2222222222222222222222222222222222222 (gr/gryorster.c:124)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp2222222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grzebesroute.rs (consolidation steps 21/22) ----
/// stageGObj0_OnInit_fp22222222222222222222222222222222222222 (gr/grzebesroute.c:137)
    pub unsafe extern "C" fn stageGObj0_OnInit_fp22222222222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

/// stageGObj2_OnInit_fp222222222222222222222222222222 (gr/grzebesroute.c:151)
    pub unsafe extern "C" fn stageGObj2_OnInit_fp222222222222222222222222222222(a0: *mut HSD_GObj) { let c = c(); let gp = (*a0).user_data as *mut Ground; c.granime_801c8138(a0, (*gp).map_id as i32, 0); }

// ---- folded from grfigure1.rs (consolidation steps 21/22) ----
/// grFigure1_8020E078 (gr/grfigure1.c:86)
    pub extern "C" fn grFigure1_8020E078() -> i32 { 0 }

/// grFigure1_8020E194 (gr/grfigure1.c:112)
    pub extern "C" fn grFigure1_8020E194(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure1_8020E1D0 (gr/grfigure1.c:129)
    pub extern "C" fn grFigure1_8020E1D0(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure1_8020E250 (gr/grfigure1.c:147)
    pub extern "C" fn grFigure1_8020E250(a0: *mut HSD_GObj) -> i32 { 0 }

/// grFigure1_OnTouchLine (gr/grfigure1.c:156)
    pub extern "C" fn grFigure1_OnTouchLine(a0: i32) -> *mut melee_types::lbColl_80008D30_arg1 { core::ptr::null_mut() }

/// grFigure1_OnCheckShadowRender (gr/grfigure1.c:161)
    pub extern "C" fn grFigure1_OnCheckShadowRender(a0: *mut Vec3, a1: i32, a2: *mut HSD_JObj) -> i32 { 1 }
