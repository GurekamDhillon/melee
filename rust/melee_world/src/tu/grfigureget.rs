//! Port of src/melee/gr/grfigureget.c: the Figure-Get stage ("/GrNFg.dat", the trophy coin-catch
//! bonus): stage gobj 1 drops three trophy-coin items from above at random intervals, the
//! joint callback `grFigureGet_80219B10` collects coins that land on the middle platform, and
//! when all coins have been dropped the result flags are set in `stage_info.flags`. Callees of
//! unported TUs go through `ext`; the constant-return callbacks are the earlier round's ports
//! (`gr::trivial`), re-exported. Checked against the C (whole TU, shipping contraction) by
//! tests/tu_grfigureget.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::grdisplay::grDisplay_801C5DB0;
pub use crate::gr::trivial::grfigureget::{
    grFigureGet_802195C4, grFigureGet_802196E0, grFigureGet_80219890, grFigureGet_OnCheckShadowRender,
    grFigureGet_OnTouchLine,
};

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32);
    fn Ground_801C39C0();
    fn Ground_801C3BB4();
    fn Ground_801C5840() -> i32;
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj;
    fn Ground_GetYakumonoParam() -> *mut c_void;
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc;
    fn HSD_Randi(a0: i32) -> i32;
    fn Mex_GrBind(a0: *mut c_void) -> *mut c_void;
    fn Stage_GetCamBoundsTopOffset() -> f32;
    fn Toy_80305058(a0: i32, a1: i32, a2: i32, a3: f32) -> i32;
    fn ftLib_80086984(a0: *mut HSD_GObj) -> *mut CollData;
    fn gm_8017E280(a0: u16, a1: u32) -> i8;
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32);
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj;
    fn it_802F2014(a0: *mut HSD_GObj, a1: i32);
    fn it_802F2020(a0: *mut HSD_GObj) -> i32;
    fn it_802F202C(a0: *mut HSD_GObj);
    fn it_802F2094(a0: *mut HSD_GObj, a1: *mut Vec, a2: i32, a3: i32) -> *mut HSD_GObj;
    fn lb_800115F4();
    fn memzero(a0: *mut c_void, a1: i32);
    fn mpJointSetCb1(a0: i32, a1: *mut c_void, a2: Option<unsafe extern "C" fn(*mut c_void, i32, *mut CollData, i32, u32, f32)>);
    fn tyDisplay_8031C2CC();
    fn tyDisplay_8031C354(a0: i32, a1: *mut i32, a2: i32, a3: i32) -> i32;
    fn tyDisplay_8031C454(a0: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = unsafe { core::mem::zeroed() };
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

/// `grFigureGet_Params` (the stage's yakumono block)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grFigureGet_Params {
    pub x0: i32,
    pub x4: i32,
    pub x8: i32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
}

/// `static Vec3 const grFigureGet_803B8470`
pub static grFigureGet_803B8470: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

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

/// `static StageCallbacks grFigureGet_StageCallbacks[]`
pub static mut grFigureGet_StageCallbacks: [StageCallbacks; 2] = [
    row(Some(stageGObj0_OnInit), Some(grFigureGet_802196E0), Some(grFigureGet_802196E8), Some(grFigureGet_802196EC), 0),
    row(Some(grFigureGet_802196F0), Some(grFigureGet_80219890), Some(grFigureGet_80219898), Some(grFigureGet_80219B0C), (1 << 30) | (1 << 31)),
];

/// `StageData grFigureGet_StageData`
pub static mut grFigureGet_StageData: StageData = StageData {
    grkind: Gr_Kind_FigureGet,
    callbacks: unsafe { &raw mut grFigureGet_StageCallbacks as *mut StageCallbacks },
    data1: c"/GrNFg.dat".as_ptr() as *mut i8,
    on_init: Some(grFigureGet_OnInit),
    on_demo_init: Some(grFigureGet_OnDemoInit),
    on_load: Some(grFigureGet_OnLoad),
    on_start: Some(grFigureGet_OnStart),
    callback4: Some(grFigureGet_802195C4),
    on_touch_line: Some(grFigureGet_OnTouchLine),
    on_check_shadow_render: Some(grFigureGet_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

/// `static grFigureGet_Params* yakumono_param`
pub static mut yakumono_param: *mut grFigureGet_Params = null_mut();

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

/// grFigureGet_OnDemoInit
pub unsafe extern "C" fn grFigureGet_OnDemoInit(_unused: i32) {}

/// grFigureGet_OnInit
pub unsafe extern "C" fn grFigureGet_OnInit() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grFigureGet_Params;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grFigureGet_802195CC(0);
    grFigureGet_802195CC(1);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
}

/// grFigureGet_OnLoad
pub unsafe extern "C" fn grFigureGet_OnLoad() {}

/// grFigureGet_OnStart
pub unsafe extern "C" fn grFigureGet_OnStart() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

/// grFigureGet_802195CC: sets up stage gobj `gobj_id` with its row of the callback table.
pub unsafe extern "C" fn grFigureGet_802195CC(gobj_id: i32) -> *mut HSD_GObj {
    let callbacks = &raw mut grFigureGet_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);
    if !gobj.is_null() {
        Ground_SetupStageCallbacks(gobj, callbacks);
    }
    // else OSReport("%s:%d: couldn t get gobj(id=%d)\n", ...): diagnostics only
    gobj
}

/// stageGObj0_OnInit
pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    Ground_StartMapAnim(gobj);
}

/// grFigureGet_802196E8
pub unsafe extern "C" fn grFigureGet_802196E8(_gobj: *mut HSD_GObj) {}

/// grFigureGet_802196EC
pub unsafe extern "C" fn grFigureGet_802196EC(_gobj: *mut HSD_GObj) {}

/// grFigureGet_802196F0: picks the three trophies, resets the drop state and installs the
/// joint callback.
pub unsafe extern "C" fn grFigureGet_802196F0(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let mut sp14: i32 = 0;

    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    ext::tyDisplay_8031C2CC();
    let i0 = ext::HSD_Randi(3);
    (*gp).u.figureget.x10[i0 as usize] = ext::Toy_80305058(0, 0x63, 0, 65.0);
    let mut i1 = i0 + 1;
    if i1 >= 3 {
        i1 = 0;
    }
    (*gp).u.figureget.x10[i1 as usize] = ext::Ground_801C5840();
    if ext::tyDisplay_8031C354((*gp).u.figureget.x10[i1 as usize], &mut sp14, 1, 2) == 0 {
        loop {
            let temp_r3 = ext::Toy_80305058(2, 0x63, 1, 75.0);
            sp14 = temp_r3;
            if (*gp).u.figureget.x10[i1 as usize] != temp_r3 {
                break;
            }
        }
    }
    i1 += 1;
    if i1 >= 3 {
        i1 = 0;
    }
    (*gp).u.figureget.x10[i1 as usize] = sp14;
    for n in 0..3 {
        ext::tyDisplay_8031C454((*gp).u.figureget.x10[n]);
    }
    let temp_r3_2 = (*yakumono_param).x4;
    let var_r6 = if temp_r3_2 != 0 { ext::HSD_Randi(temp_r3_2) } else { 0 };
    (*gp).u.figureget.x0 = (*yakumono_param).x0 + var_r6;
    (*gp).u.figureget.x4 = 0;
    (*gp).u.figureget.x8 = 0;
    (*gp).u.figureget.xC = 0;
    ext::memzero((*gp).u.figureget.x1C.as_mut_ptr().cast(), core::mem::size_of_val(&(*gp).u.figureget.x1C) as i32);
    ext::memzero((*gp).u.figureget.x28.as_mut_ptr().cast(), core::mem::size_of_val(&(*gp).u.figureget.x28) as i32);
    ext::memzero((*gp).u.figureget.x34.as_mut_ptr().cast(), core::mem::size_of_val(&(*gp).u.figureget.x34) as i32);
    ext::mpJointSetCb1(0, gp.cast(), Some(grFigureGet_80219B10));
}

/// grFigureGet_80219898: the per-frame drop timer; once every coin is out, scores the catch.
pub unsafe extern "C" fn grFigureGet_80219898(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;

    let mut var_r28 = *ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !var_r28.is_null() {
        (*ext::ftLib_80086984(var_r28)).joint_id_skip = 0;
        var_r28 = (*var_r28).next;
    }
    if (*gp).u.figureget.x4 < (*yakumono_param).x8 {
        let temp_r3 = (*gp).u.figureget.x0;
        if temp_r3 == 0 {
            let mut pos: Vec3 = grFigureGet_803B8470;
            (*gp).u.figureget.x4 += 1;
            let mut var_f2: f32 = (*yakumono_param).x10;
            var_f2 -= (*yakumono_param).xC;
            if var_f2 < 0.0 {
                var_f2 = -var_f2;
            }
            let mut temp_f1: f32 = 0.5;
            temp_f1 = var_f2 * temp_f1;
            temp_f1 -= 20.0;
            let mut var_r3 = if temp_f1 as i32 != 0 { ext::HSD_Randi(temp_f1 as i32) } else { 0 };
            pos.x = 20.0 + var_r3 as f32;
            if ext::HSD_Randi(2) == 0 {
                pos.x *= -1.0;
            }
            pos.y = (*yakumono_param).x14 + ext::Stage_GetCamBoundsTopOffset();
            let temp_r6 = (*gp).u.figureget.x4;
            let temp_r3_2 = ext::it_802F2094(gobj, &mut pos, (*gp).u.figureget.x10[(temp_r6 - 1) as usize], temp_r6);
            (*gp).u.figureget.x34[((*gp).u.figureget.x4 - 1) as usize] = temp_r3_2;
            ext::it_802F2014(temp_r3_2, 0);
            let temp_r3 = (*yakumono_param).x4;
            var_r3 = if temp_r3 != 0 { ext::HSD_Randi(temp_r3) } else { 0 };
            (*gp).u.figureget.x0 = (*yakumono_param).x0 + var_r3;
        } else {
            (*gp).u.figureget.x0 = temp_r3 - 1;
        }
    } else if grFigureGet_80219C50(gobj) != 0 {
        let mut var_r30 = 0;
        while var_r30 < (*gp).u.figureget.x8 {
            ext::gm_8017E280((*gp).u.figureget.x1C[var_r30 as usize] as u16, 1);
            var_r30 += 1;
        }
        if (*gp).u.figureget.x8 != 0 {
            if (*gp).u.figureget.x8 == 3 {
                ext::stage_info.flags |= 0x20;
            }
            ext::stage_info.flags |= 0x100;
        } else {
            ext::stage_info.flags |= 0x200;
        }
    }
    ext::lb_800115F4();
}

/// grFigureGet_80219B0C
pub unsafe extern "C" fn grFigureGet_80219B0C(_gobj: *mut HSD_GObj) {}

/// grFigureGet_80219B10 (mpLib_JointCollisionCallback): collects a coin that touches the platform.
pub unsafe extern "C" fn grFigureGet_80219B10(
    user_data: *mut c_void,
    _joint_id: i32,
    coll: *mut CollData,
    _coll_x50: i32,
    _ground_kind: u32,
    _delta_y: f32,
) {
    let gp = user_data as *mut Ground;
    let item_gobj = (*coll).x0_gobj;
    if (*coll).x34_flags.b1234() as i32 == 5
        && !item_gobj.is_null()
        && (*item_gobj).classifier == HSD_GOBJ_CLASS_ITEM as u16
    {
        if (*((*(*coll).x0_gobj).user_data as *mut Item)).kind == It_Kind_Coin && (*gp).u.figureget.x8 < 3 {
            let item_gobj = (*coll).x0_gobj;
            let value = ext::it_802F2020(item_gobj);
            let mut count = 0;
            let mut i = 0;
            while i < (*gp).u.figureget.x8 {
                if (*gp).u.figureget.x28[i as usize] == (*coll).x0_gobj {
                    count += 1;
                }
                i += 1;
            }
            if count == 0 && (*coll).cur_pos.x <= 4.5 && (*coll).cur_pos.x >= -4.5 {
                ext::it_802F202C((*coll).x0_gobj);
                (*gp).u.figureget.x28[(*gp).u.figureget.x8 as usize] = (*coll).x0_gobj;
                (*gp).u.figureget.x1C[(*gp).u.figureget.x8 as usize] = value;
                (*gp).u.figureget.x8 += 1;
            }
        }
    }
}

/// grFigureGet_80219C34: one more coin that will never be caught.
pub unsafe extern "C" fn grFigureGet_80219C34(gobj: *mut HSD_GObj) {
    if gobj.is_null() {
        return;
    }
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.figureget.xC += 1;
}

/// grFigureGet_80219C50: all three coins are either caught or lost.
pub unsafe extern "C" fn grFigureGet_80219C50(gobj: *mut HSD_GObj) -> i32 {
    if gobj.is_null() {
        return 0;
    }
    let gp = (*gobj).user_data as *mut Ground;
    if (*gp).u.figureget.x8 + (*gp).u.figureget.xC >= 3 {
        return 1;
    }
    0
}
