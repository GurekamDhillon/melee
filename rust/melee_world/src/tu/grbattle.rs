//! src/melee/gr/grbattle.c: Battlefield-style "NBa" stage (Gr_Kind_Battle). State is the two statics
//! (`isDemoFight`, `yakumono_param`), the per-gobj `Ground` and `stage_info`; every other callee is a
//! TODO hook in [`ext`]. The constant-return / empty callbacks (`grBattle_80219D7C`, the `*_Callback1`
//! predicates, `grBattle_OnTouchLine`, `grBattle_OnCheckShadowRender`) are the earlier round's ports,
//! re-exported. No float arithmetic here.
//! State-oracle checked against the C (tests/gro_grbattle.rs, gen/oracle/gr_oracle.py).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::missing_safety_doc)]

use super::grinl::{assert_, GET_GROUND, GET_JOBJ, HSD_JObjSetTranslate};
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

pub use crate::gr::trivial::grbattle::{
    grBattle_80219D7C, grBattle_BG_Callback1, grBattle_GObj0_Callback1, grBattle_GObj1_Callback1, grBattle_GObj2_Callback1,
    grBattle_GObj4_Callback1, grBattle_GObj5_Callback1, grBattle_GObj6_Callback1, grBattle_OnCheckShadowRender,
    grBattle_OnTouchLine,
};

/// Callees in TUs that are not ported yet. An unset hook panics.
pub mod ext {
    use crate::tu::hookmac::{hook, hook_or};
    use core::ffi::c_void;
    use melee_types::*;
    pub use crate::tu::grinl::ext::HSD_JObjSetMtxDirtySub;
    hook_or!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32), |gobj: *mut HSD_GObj, render_cb: GObj_RenderFunc, gx_link: u8, priority: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(gobj as _, render_cb as _, gx_link as _, priority as _); } });
    pub static mut GROUND_801C2CF4: Option<fn(s32) -> *mut HSD_JObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C2CF4(a0: s32) -> *mut HSD_JObj {
        match unsafe { GROUND_801C2CF4 } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_801C2CF4(a0 as _) as _ }; } }
    }
    pub static mut GROUND_801C39C0: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C39C0() {
        match unsafe { GROUND_801C39C0 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } return; } }
    }
    pub static mut GROUND_801C3BB4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C3BB4() {
        match unsafe { GROUND_801C3BB4 } { Some(f) => f(), None => { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } return; } }
    }
    pub static mut GROUND_801C4A08: Option<fn(*mut HSD_GObj)> = None;
    #[allow(non_snake_case)]
    pub fn Ground_801C4A08(gobj: *mut HSD_GObj) {
        match unsafe { GROUND_801C4A08 } { Some(f) => f(gobj), None => { unsafe { let _ = crate::tu::ground::Ground_801C4A08(gobj as _); } return; } }
    }
    pub static mut GROUND_GETMAPGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetMapGObj(map_id: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETMAPGOBJ } { Some(f) => f(map_id), None => { return unsafe { crate::tu::ground::Ground_GetMapGObj(map_id as _) as _ }; } }
    }
    pub static mut GROUND_GETSTAGEGOBJ: Option<fn(i32) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj {
        match unsafe { GROUND_GETSTAGEGOBJ } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ }; } }
    }
    pub static mut GROUND_GETYAKUMONOPARAM: Option<fn() -> *mut c_void> = None;
    #[allow(non_snake_case)]
    pub fn Ground_GetYakumonoParam() -> *mut c_void {
        match unsafe { GROUND_GETYAKUMONOPARAM } { Some(f) => f(), None => { return unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ }; } }
    }
    pub static mut GROUND_INITMAPCOLL: Option<fn(*mut HSD_JObj, s32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_InitMapColl(jobj: *mut HSD_JObj, map_id: s32) -> i32 {
        match unsafe { GROUND_INITMAPCOLL } { Some(f) => f(jobj, map_id), None => { return unsafe { crate::tu::ground::Ground_InitMapColl(jobj as _, map_id as _) as _ }; } }
    }
    pub static mut GROUND_UPDATEMAPCOLL: Option<fn(*mut HSD_GObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Ground_UpdateMapColl(a0: *mut HSD_GObj) -> i32 {
        match unsafe { GROUND_UPDATEMAPCOLL } { Some(f) => f(a0), None => { return unsafe { crate::tu::ground::Ground_UpdateMapColl(a0 as _) as _ }; } }
    }
    hook_or!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8) -> *mut HSD_GObjProc, |a0: *mut HSD_GObj, a1: HSD_GObjEvent, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ });
    hook_or!(HSD_JOBJCLEARFLAGSALL, HSD_JObjClearFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_JOBJGETFLAGS, HSD_JObjGetFlags, (jobj: *mut HSD_JObj) -> u32, |jobj: *mut HSD_JObj| unsafe { melee_runtime::tu::jobj_world::HSD_JObjGetFlags(jobj as _) as _ });
    hook_or!(HSD_JOBJSETFLAGSALL, HSD_JObjSetFlagsAll, (a0: *mut HSD_JObj, flags: u32), |a0: *mut HSD_JObj, flags: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, flags as _); } });
    hook_or!(HSD_RANDI, HSD_Randi, (max_val: s32) -> s32, |max_val: s32| melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), max_val));
    hook_or!(OSREPORT, OSReport, (a0: *const i8, va: &[usize]), |a0: *const i8, va: &[usize]| unsafe { melee_runtime::tu::os_error::OSReport(a0, va) });
    pub static mut STAGE_80225194: Option<fn() -> StKind> = None;
    #[allow(non_snake_case)]
    pub fn Stage_80225194() -> StKind {
        match unsafe { STAGE_80225194 } { Some(f) => f(), None => { return unsafe { crate::tu::stage::Stage_80225194() as _ }; } }
    }
    hook!(GM_8016B3D8, gm_8016B3D8, () -> i32);
    pub static mut GRANIME_801C8138: Option<fn(*mut HSD_GObj, enum_t, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C8138(a0: *mut HSD_GObj, a1: enum_t, a2: i32) {
        match unsafe { GRANIME_801C8138 } { Some(f) => f(a0, a1, a2), None => { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } return; } }
    }
    pub static mut GRANIME_801C83D0: Option<fn(*mut HSD_GObj, i32, enum_t) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: enum_t) -> i32 {
        match unsafe { GRANIME_801C83D0 } { Some(f) => f(a0, a1, a2), None => { return unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ }; } }
    }
    pub static mut GRLIB_801C96E8: Option<fn(*mut HSD_GObj) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C96E8(a0: *mut HSD_GObj) -> i32 {
        match unsafe { GRLIB_801C96E8 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grlib::grLib_801C96E8(a0 as _) as _ }; } }
    }
    pub static mut GRLIB_801C9A10: Option<fn() -> VecMtxPtr> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9A10() -> VecMtxPtr {
        match unsafe { GRLIB_801C9A10 } { Some(f) => f(), None => { return unsafe { crate::tu::grlib::grLib_801C9A10() as _ }; } }
    }
    pub static mut GRLIB_801C9A70: Option<fn(enum_t, *mut Vec)> = None;
    #[allow(non_snake_case)]
    pub fn grLib_801C9A70(a0: enum_t, a1: *mut Vec) {
        match unsafe { GRLIB_801C9A70 } { Some(f) => f(a0, a1), None => { unsafe { let _ = crate::tu::grlib::grLib_801C9A70(a0 as _, a1 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C94D8: Option<fn(*mut c_void)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C94D8(a0: *mut c_void) {
        match unsafe { GRMATERIAL_801C94D8 } { Some(f) => f(a0), None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C94D8(a0 as _); } return; } }
    }
    pub static mut GRMATERIAL_801C9604: Option<fn(*mut HSD_GObj, *mut c_void, i32)> = None;
    #[allow(non_snake_case)]
    pub fn grMaterial_801C9604(bg: *mut HSD_GObj, a1: *mut c_void, a2: i32) {
        match unsafe { GRMATERIAL_801C9604 } {
            Some(f) => f(bg, a1, a2),
            None => { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C9604(bg as _, a1 as _, (a2 != 0)); } },
        }
    }
    pub static mut GRZAKOGENERATOR_801CAE04: Option<fn(*mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj> = None;
    #[allow(non_snake_case)]
    pub fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj {
        match unsafe { GRZAKOGENERATOR_801CAE04 } { Some(f) => f(a0), None => { return unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ }; } }
    }
    pub static mut LB_800115F4: Option<fn()> = None;
    #[allow(non_snake_case)]
    pub fn lb_800115F4() {
        match unsafe { LB_800115F4 } { Some(f) => f(), None => { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } return; } }
    }
    pub use crate::tu::grinl::ext::stage_info;
    /// `grDisplay_801C5DB0` is only ever passed as a render callback (its address is the argument).
    pub unsafe extern "C" fn grDisplay_801C5DB0(_gobj: *mut HSD_GObj, _pass: i32) {
        { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(_gobj as _, _pass as _); } return; }
    }
}

/// `struct grBattle_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grBattle_YakumonoParam {
    pub bg_curr_color_overlay: *mut c_void,
    pub bg_prev_color_overlay: *mut c_void,
}

/// `static GXBool isDemoFight` (4D6AC8).
pub static mut isDemoFight: u8 = 0;
/// `static struct grBattle_YakumonoParam* yakumono_param` (4D6ACC).
pub static mut yakumono_param: *mut grBattle_YakumonoParam = null_mut();

const BG_WAITING: i32 = 0;
const BG_TRANSITIONING: i32 = 1;
const BG_DONE: i32 = 2;

macro_rules! cbs {
    ($a:ident, $b:ident, $c:ident, $d:ident, $flags:expr) => {
        StageCallbacks {
            on_init: Some($a),
            callback1: Some($b),
            gobj_proc: Some($c),
            callback3: Some($d),
            anon_10: StageCallbacks_anon_135 { flags: $flags },
        }
    };
}

/// `static StageCallbacks grNBa_StageCallbacks[]`: one entry per gobj kind (0, 1, 2, BG, 4, 5, 6).
pub static mut grNBa_StageCallbacks: [StageCallbacks; 7] = [
    cbs!(grBattle_GObj0_Callback0, grBattle_GObj0_Callback1, grBattle_GObj0_Callback2, grBattle_GObj0_Callback3, 0),
    cbs!(grBattle_GObj1_Callback0, grBattle_GObj1_Callback1, grBattle_GObj1_Callback2, grBattle_GObj1_Callback3, 0),
    cbs!(grBattle_GObj2_Callback0, grBattle_GObj2_Callback1, grBattle_GObj2_Callback2, grBattle_GObj2_Callback3, 0),
    cbs!(grBattle_BG_Callback0, grBattle_BG_Callback1, grBattle_BG_Callback2, grBattle_BG_Callback3, 0),
    cbs!(grBattle_GObj4_Callback0, grBattle_GObj4_Callback1, grBattle_GObj4_Callback2, grBattle_GObj4_Callback3, 0),
    cbs!(grBattle_GObj5_Callback0, grBattle_GObj5_Callback1, grBattle_GObj5_Callback2, grBattle_GObj5_Callback3, 0),
    cbs!(grBattle_GObj6_Callback0, grBattle_GObj6_Callback1, grBattle_GObj6_Callback2, grBattle_GObj6_Callback3, (1 << 30) | (1 << 31)),
];

/// `StageData grNBa_StageData`.
pub static mut grNBa_StageData: StageData = StageData {
    grkind: Gr_Kind_Battle,
    callbacks: core::ptr::addr_of!(grNBa_StageCallbacks) as *mut StageCallbacks,
    data1: b"/GrNBa.dat\0".as_ptr() as *mut i8,
    on_init: Some(grBattle_OnInit),
    on_demo_init: Some(grBattle_OnDemoInit),
    on_load: Some(grBattle_OnLoad),
    on_start: Some(grBattle_OnStart),
    callback4: Some(grBattle_80219D7C),
    on_touch_line: Some(grBattle_OnTouchLine),
    on_check_shadow_render: Some(grBattle_OnCheckShadowRender),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grBattle_OnDemoInit(_arg0: i32) {
    melee_core::ct!("grBattle_OnDemoInit");
    isDemoFight = 1;
}

pub unsafe extern "C" fn grBattle_OnInit() {
    melee_core::ct!("grBattle_OnInit");
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grBattle_YakumonoParam;
    let si = ext::stage_info();
    (*si).unk8C.set_b4(1);
    (*si).unk8C.set_b5(1);

    grBattle_80219D84(0);
    if ext::gm_8016B3D8() != 0 || ext::Stage_80225194() as i32 == 273 || ext::Stage_80225194() as i32 == 245 {
        grBattle_80219D84(5);
    } else {
        grBattle_80219D84(3);
        grBattle_80219D84(1);
    }
    grBattle_80219D84(6);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::grLib_801C9A10();
    isDemoFight = 0;
}

pub unsafe extern "C" fn grBattle_OnLoad() { melee_core::ct!("grBattle_OnLoad");}

pub unsafe extern "C" fn grBattle_OnStart() {
    melee_core::ct!("grBattle_OnStart");
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe fn grBattle_80219D84(gobj_id: i32) -> *mut Ground_GObj {
    melee_core::ct!("grBattle_80219D84");
    let callbacks: *mut StageCallbacks = &mut grNBa_StageCallbacks[gobj_id as usize];
    let gobj = ext::Ground_GetStageGObj(gobj_id);

    if !gobj.is_null() {
        let gp = GET_GROUND(gobj);
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::grDisplay_801C5DB0), 3, 0);

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
        ext::OSReport(
            b"%s:%d: couldn t get gobj(id=%d)\n\0".as_ptr() as *const i8,
            &[b"src/melee/gr/grbattle.c\0".as_ptr() as usize, 232, gobj_id as u32 as usize],
        );
    }
    gobj
}

pub unsafe extern "C" fn grBattle_GObj0_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj0_Callback0");
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let gp = GET_GROUND(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);

    if isDemoFight != 0 {
        for i in 0..4 {
            ext::grLib_801C9A70(i, &mut v);
            HSD_JObjSetTranslate(ext::Ground_801C2CF4(i as s32), &v);
        }
    }
}

pub unsafe extern "C" fn grBattle_GObj0_Callback2(_arg0: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj0_Callback2");}
pub unsafe extern "C" fn grBattle_GObj0_Callback3(_arg0: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj0_Callback3");}

pub unsafe extern "C" fn grBattle_GObj6_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj6_Callback0");
    init_map_coll_and_anim(gobj);
}

pub unsafe extern "C" fn grBattle_GObj6_Callback2(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj6_Callback2");
    ext::Ground_UpdateMapColl(gobj);
    ext::lb_800115F4();
}
pub unsafe extern "C" fn grBattle_GObj6_Callback3(_arg0: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj6_Callback3");}

pub unsafe extern "C" fn grBattle_GObj5_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj5_Callback0");
    let gp = GET_GROUND(gobj);
    init_map_coll_and_anim(gobj);
    (*gp).x11_flags.set_b012(2);
}
pub unsafe extern "C" fn grBattle_GObj5_Callback2(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj5_Callback2");}
pub unsafe extern "C" fn grBattle_GObj5_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj5_Callback3");}

pub unsafe extern "C" fn grBattle_GObj1_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj1_Callback0");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);

    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}
pub unsafe extern "C" fn grBattle_GObj1_Callback2(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj1_Callback2");}
pub unsafe extern "C" fn grBattle_GObj1_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj1_Callback3");}

/// `static void setup_unk_gobj`.
pub unsafe fn setup_unk_gobj(gobj: *mut Ground_GObj) {
    melee_core::ct!("setup_unk_gobj");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
    ext::grMaterial_801C94D8(jobj as *mut c_void);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grBattle_GObj2_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj2_Callback0");
    setup_unk_gobj(gobj);
}
pub unsafe extern "C" fn grBattle_GObj2_Callback2(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj2_Callback2");}
pub unsafe extern "C" fn grBattle_GObj2_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj2_Callback3");}

pub unsafe extern "C" fn grBattle_GObj4_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_GObj4_Callback0");
    setup_unk_gobj(gobj);
}
pub unsafe extern "C" fn grBattle_GObj4_Callback2(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj4_Callback2");}
pub unsafe extern "C" fn grBattle_GObj4_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_GObj4_Callback3");}

/// gr/inlines.h `Ground_InitMapCollAndAnim`.
unsafe fn init_map_coll_and_anim(gobj: *mut Ground_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    ext::Ground_InitMapColl(jobj, (*gp).map_id as s32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
}

/// `static inline reset_bg_timer`.
pub unsafe fn reset_bg_timer(gp: *mut Ground) {
    melee_core::ct!("reset_bg_timer");
    (*gp).u.battle_bg.timer = ext::HSD_Randi(1200) + 2400;
}

pub unsafe extern "C" fn grBattle_BG_Callback0(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_BG_Callback0");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    (*gp).x11_flags.set_b012(2);
    (*gp).u.battle_bg.state = BG_WAITING;
    reset_bg_timer(gp);
    (*gp).u.battle_bg.curr = -1;
    ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
}

pub unsafe extern "C" fn grBattle_BG_Callback2(gobj: *mut Ground_GObj) {
    melee_core::ct!("grBattle_BG_Callback2");
    let gp = GET_GROUND(gobj);
    let jobj = GET_JOBJ(gobj);
    let indices: [i16; 3] = [1, 2, 4];
    let battle_bg_max = indices.len();

    match (*gp).u.battle_bg.state {
        BG_WAITING => {
            let t = (*gp).u.battle_bg.timer;
            (*gp).u.battle_bg.timer = t.wrapping_sub(1);
            if t < 0 {
                (*gp).u.battle_bg.state = BG_TRANSITIONING;
                ext::grAnime_801C8138(gobj, (*gp).map_id as enum_t, 0);
            }
        }
        BG_TRANSITIONING => {
            if ext::HSD_JObjGetFlags(jobj) & JOBJ_HIDDEN as u32 != 0 {
                ext::HSD_JObjClearFlagsAll(jobj, JOBJ_HIDDEN as u32);
            }
            if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                if (*gp).u.battle_bg.curr == -1 {
                    let mut i = 0;
                    while i < battle_bg_max {
                        if !ext::Ground_GetMapGObj(indices[i] as i32).is_null() {
                            (*gp).u.battle_bg.curr = indices[i] as i32;
                            break;
                        }
                        i += 1;
                    }
                    assert_(i < battle_bg_max, 527, "i<BATTLE_BG_MAX");
                }

                // Pick a random different background to transition to
                (*gp).u.battle_bg.prev = (*gp).u.battle_bg.curr;
                loop {
                    let bg_idx = indices[ext::HSD_Randi(battle_bg_max as s32) as usize] as i32;
                    (*gp).u.battle_bg.curr = bg_idx;
                    if bg_idx != (*gp).u.battle_bg.prev {
                        break;
                    }
                }

                let mut bg_gobj = ext::Ground_GetMapGObj((*gp).u.battle_bg.prev);
                assert_(!bg_gobj.is_null(), 535, "bg_gobj");
                ext::grMaterial_801C9604(bg_gobj, (*yakumono_param).bg_prev_color_overlay, 0);

                bg_gobj = grBattle_80219D84((*gp).u.battle_bg.curr);
                assert_(!bg_gobj.is_null(), 539, "bg_gobj");
                ext::grMaterial_801C9604(bg_gobj, (*yakumono_param).bg_curr_color_overlay, 0);

                (*gp).u.battle_bg.state = BG_DONE;
            }
        }
        BG_DONE => {
            let bg_gobj = ext::Ground_GetMapGObj((*gp).u.battle_bg.prev);
            assert_(!bg_gobj.is_null(), 546, "bg_gobj");
            if ext::grLib_801C96E8(bg_gobj) != 0 {
                ext::Ground_801C4A08(bg_gobj);
                ext::HSD_JObjSetFlagsAll(jobj, JOBJ_HIDDEN as u32);
                (*gp).u.battle_bg.state = BG_WAITING;
                reset_bg_timer(gp);
            }
        }
        _ => {}
    }
}

pub unsafe extern "C" fn grBattle_BG_Callback3(_gobj: *mut Ground_GObj) { melee_core::ct!("grBattle_BG_Callback3");}
