//! src/melee/gm/gm_1A7A.c: the trophy-congratulations backdrop (`gm_801A7B00`): the light, the camera (whose
//! animation ends the scene), the stage model and the character trophy placed from the toy table. All 4
//! functions plus the 4 `static inline` setup helpers are checked against the C by a state oracle
//! (tests/tu_gm_1A7A.rs over gen/turef/tu_gm_1A7A_ref.c): the game's gm_1A7A.c compiled natively (shipping
//! contraction) with the engine calls stubbed and logged; the callee log (float arguments as bit patterns),
//! the two static gobj pointers and the joint transform are compared.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs, unused_unsafe)]

use core::ffi::c_void;
use melee_types::*;

/// `static HSD_GObj* gm_804D67B0;` (the stage model) / `gm_804D67B4;` (the character).
pub static mut gm_804D67B0: *mut HSD_GObj = core::ptr::null_mut();
pub static mut gm_804D67B4: *mut HSD_GObj = core::ptr::null_mut();

/// Calls into TUs that are not ported yet. Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    // The scene data the regular-trophy TU (gmregtyfall.c) loaded: `gm_804D6798`, `gm_804D67A8`, `gm_804D67AC`.
    hook!(GM_804D6798, gm_804D6798, () -> *mut HSD_Joint);
    hook!(GM_804D67A8, gm_804D67A8, () -> *mut SceneDesc);
    hook!(GM_804D67AC, gm_804D67AC, () -> *mut SceneDesc);
    // sysdolphin/baselib/gobj*.c
    hook!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj);
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void));
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render: usize, gx_link: u8, priority: u32));
    hook!(GOBJ_SETUPGXLINKMAX, GObj_SetupGXLinkMax, (gobj: *mut HSD_GObj, render: usize, priority: u32));
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (gobj: *mut HSD_GObj, proc: usize, prio: u8));
    hook!(HSD_GOBJFREE, HSD_GObjFree, (gobj: *mut HSD_GObj));
    /// `HSD_GObj_LightKind` / `HSD_GObj_JObjKind` / `HSD_GObj_CameraKind` (sysdolphin/baselib/gobj.c).
    hook!(HSD_GOBJ_LIGHTKIND, hsd_gobj_light_kind, () -> i8);
    hook!(HSD_GOBJ_JOBJKIND, hsd_gobj_jobj_kind, () -> u8);
    hook!(HSD_GOBJ_CAMERAKIND, hsd_gobj_camera_kind, () -> u8);
    // sysdolphin/baselib/jobj.c, cobj.c
    pub static mut HSD_JOBJLOADJOINT: Option<fn(*mut HSD_Joint) -> *mut HSD_JObj> = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj {
                match unsafe { HSD_JOBJLOADJOINT } {
                    Some(f) => f(joint),
                    None => { return unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(joint as _) as _ }; },
                }
            }
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (jobj: *mut HSD_JObj, frame: f32));
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    /// The out-of-line `HSD_JObjSetMtxDirty` (the `WithMtxDirty` setters call it, not the macro).
    pub static mut HSD_JOBJSETMTXDIRTY: Option<fn(*mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj)  {
                match unsafe { HSD_JOBJSETMTXDIRTY } {
                    Some(f) => f(jobj),
                    None => { unsafe { melee_fighter::tu::ft_0c31::HSD_JObjSetMtxDirty(jobj as _); } return; },
                }
            }
    pub static mut HSD_COBJADDANIM: Option<fn(*mut HSD_CObj, *mut HSD_CameraAnim) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_CObjAddAnim(cobj: *mut HSD_CObj, anim: *mut HSD_CameraAnim)  {
                match unsafe { HSD_COBJADDANIM } {
                    Some(f) => f(cobj, anim),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjAddAnim(cobj as _, anim as _); } return; },
                }
            }
    pub static mut HSD_COBJREQANIM: Option<fn(*mut HSD_CObj, f32) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_CObjReqAnim(cobj: *mut HSD_CObj, frame: f32)  {
                match unsafe { HSD_COBJREQANIM } {
                    Some(f) => f(cobj, frame),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjReqAnim(cobj as _, frame as _); } return; },
                }
            }
    pub static mut HSD_COBJANIM: Option<fn(*mut HSD_CObj) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_CObjAnim(cobj: *mut HSD_CObj)  {
                match unsafe { HSD_COBJANIM } {
                    Some(f) => f(cobj),
                    None => { unsafe { melee_runtime::tu::cobj::HSD_CObjAnim(cobj as _); } return; },
                }
            }
    // lb/lbspdisplay.c, lb/lb_00B0.c, gm/gm_1601.c, gm/gmevent.c, gm/gmregtyfall.c, gm/gmscene.c, mn/mnmain.c, ty/toy.c
    pub static mut LB_80011AC4: Option<fn(*mut *mut LightList) -> *mut HSD_LObj> = None;
            #[allow(non_snake_case)]
            pub fn lb_80011AC4(lights: *mut *mut LightList) -> *mut HSD_LObj {
                match unsafe { LB_80011AC4 } {
                    Some(f) => f(lights),
                    None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(lights as _) as _ }; },
                }
            }
    pub static mut LB_80013B14: Option<fn(*mut HSD_CameraDescPerspective) -> *mut HSD_CObj> = None;
            #[allow(non_snake_case)]
            pub fn lb_80013B14(desc: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj {
                match unsafe { LB_80013B14 } {
                    Some(f) => f(desc),
                    None => { return unsafe { melee_runtime::tu::lbspdisplay::lb_80013B14(desc as _) as _ }; },
                }
            }
    pub static mut LB_8000C1C0: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C1C0(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj)  {
                match unsafe { LB_8000C1C0 } {
                    Some(f) => f(jobj, constraint),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000C1C0(jobj as _, constraint as _); } return; },
                }
            }
    pub static mut LB_8000C290: Option<fn(*mut HSD_JObj, *mut HSD_JObj) > = None;
            #[allow(non_snake_case)]
            pub fn lb_8000C290(jobj: *mut HSD_JObj, constraint: *mut HSD_JObj)  {
                match unsafe { LB_8000C290 } {
                    Some(f) => f(jobj, constraint),
                    None => { unsafe { melee_runtime::tu::lb_00B0::lb_8000C290(jobj as _, constraint as _); } return; },
                }
            }
    pub static mut GM_8016895C: Option<fn(*mut HSD_JObj, *mut DynamicModelDesc, i32) > = None;
            #[allow(non_snake_case)]
            pub fn gm_8016895C(jobj: *mut HSD_JObj, mdl: *mut DynamicModelDesc, idx: i32)  {
                match unsafe { GM_8016895C } {
                    Some(f) => f(jobj, mdl, idx),
                    None => { unsafe { crate::tu::gm_1601::gm_8016895C(jobj as _, mdl as _, idx as _); } return; },
                }
            }
    pub static mut GM_801BEFB0: Option<fn() -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801BEFB0() -> i32 {
                match unsafe { GM_801BEFB0 } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmgover::gm_801BEFB0() as _ }; },
                }
            }
    pub static mut GM_801A659C: Option<fn(i32) -> i32> = None;
            #[allow(non_snake_case)]
            pub fn gm_801A659C(i: i32) -> i32 {
                match unsafe { GM_801A659C } {
                    Some(f) => f(i),
                    None => { return unsafe { crate::tu::gmregtyfall::gm_801A659C(i as _) as _ }; },
                }
            }
    pub static mut GM_GETCURRENTSCENEENTERDATA: Option<fn() -> *mut c_void> = None;
            #[allow(non_snake_case)]
            pub fn gm_GetCurrentSceneEnterData() -> *mut c_void {
                match unsafe { GM_GETCURRENTSCENEENTERDATA } {
                    Some(f) => f(),
                    None => { return unsafe { crate::tu::gmscene::gm_GetCurrentSceneEnterData() as _ }; },
                }
            }
    pub static mut GM_801A9630: Option<fn() > = None;
            #[allow(non_snake_case)]
            pub fn gm_801A9630()  {
                match unsafe { GM_801A9630 } {
                    Some(f) => f(),
                    None => { unsafe { crate::tu::gmregenddisp::gm_801A9630(); } return; },
                }
            }
    pub static mut MN_8022F0F0: Option<fn(i32) > = None;
            #[allow(non_snake_case)]
            pub fn mn_8022F0F0(v: i32)  {
                match unsafe { MN_8022F0F0 } {
                    Some(f) => f(v),
                    None => { unsafe { crate::tu::mn_22EC::mn_8022F0F0(v as _); } return; },
                }
            }
    pub static mut TOY_803060BC: Option<fn(i32, i32) -> f32> = None;
            #[allow(non_snake_case)]
            pub fn Toy_803060BC(a: i32, b: i32) -> f32 {
                match unsafe { TOY_803060BC } {
                    Some(f) => f(a, b),
                    None => { return unsafe { melee_items::tu::toy::Toy_803060BC(a as _, b as _) as _ }; },
                }
            }

    /// `HSD_GObj_LObjCallback` / `HSD_GObj_803910D8` / `HSD_GObj_JObjCallback` (sysdolphin/baselib/gobj.c): the
    /// GX-link render callbacks; address only.
    pub unsafe extern "C" fn HSD_GObj_LObjCallback(_gobj: *mut HSD_GObj, _unused: i32) {
        { unsafe { melee_runtime::tu::gobj::HSD_GObj_LObjCallback(_gobj as _, _unused as _); } return; };
    }
    pub unsafe extern "C" fn HSD_GObj_803910D8(_gobj: *mut HSD_GObj, _code: i32) {
        panic!("TODO(sysdolphin): HSD_GObj_803910D8 is not ported");
    }
    pub unsafe extern "C" fn HSD_GObj_JObjCallback(_gobj: *mut HSD_GObj, _code: i32) {
        { unsafe { melee_runtime::tu::gobj::HSD_GObj_JObjCallback(_gobj as _, _code as _); } return; };
    }
}

/// `GET_JOBJ(gobj)` / `GET_COBJ(gobj)` (gobj.h): `gobj->hsd_obj`.
#[inline(always)]
unsafe fn GET_JOBJ(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}
#[inline(always)]
unsafe fn GET_COBJ(gobj: *mut HSD_GObj) -> *mut HSD_CObj {
    (*gobj).hsd_obj as *mut HSD_CObj
}

/// `HSD_JObjGetChild(jobj)` (jobj.h): NULL-safe `jobj->child`.
#[inline(always)]
unsafe fn HSD_JObjGetChild(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        core::ptr::null_mut()
    } else {
        (*jobj).child
    }
}

macro_rules! with_mtx_dirty {
    ($(#[$m:meta])* $name:ident, $($field:tt)+) => {
        $(#[$m])*
        #[inline(always)]
        unsafe fn $name(jobj: *mut HSD_JObj, v: f32) {
            (*jobj).$($field)+ = v;
            if (*jobj).flags & (JOBJ_MTX_INDEP_SRT as u32) == 0 {
                ext::HSD_JObjSetMtxDirty(jobj);
            }
        }
    };
}
with_mtx_dirty!(/** `HSD_JObjSetTranslateXWithMtxDirty` (jobj.h). */ HSD_JObjSetTranslateXWithMtxDirty, translate.x);
with_mtx_dirty!(/** `HSD_JObjSetTranslateYWithMtxDirty` (jobj.h). */ HSD_JObjSetTranslateYWithMtxDirty, translate.y);
with_mtx_dirty!(/** `HSD_JObjSetTranslateZWithMtxDirty` (jobj.h). */ HSD_JObjSetTranslateZWithMtxDirty, translate.z);
with_mtx_dirty!(/** `HSD_JObjSetScaleXWithMtxDirty` (jobj.h). */ HSD_JObjSetScaleXWithMtxDirty, scale.x);
with_mtx_dirty!(/** `HSD_JObjSetScaleYWithMtxDirty` (jobj.h). */ HSD_JObjSetScaleYWithMtxDirty, scale.y);
with_mtx_dirty!(/** `HSD_JObjSetScaleZWithMtxDirty` (jobj.h). */ HSD_JObjSetScaleZWithMtxDirty, scale.z);
with_mtx_dirty!(/** `HSD_JObjSetRotationYWithMtxDirty` (jobj.h; the quaternion flag is asserted off). */ HSD_JObjSetRotationYWithMtxDirty, rotate.y);

/// `fn_801A7A44(gobj)`: the stage model's proc.
pub unsafe fn fn_801A7A44(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A7A44");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

/// `fn_801A7A68(gobj)`: the character's proc.
pub unsafe fn fn_801A7A68(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A7A68");
    ext::HSD_JObjAnimAll(GET_JOBJ(gobj));
}

/// `fn_801A7A8C(gobj)`: the camera's proc; when its animation reaches the end, free the two models, end the
/// scene (`mn_8022F0F0(3)`, `gm_801A9630()`) and free itself.
pub unsafe fn fn_801A7A8C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A7A8C");
    let cobj = GET_COBJ(gobj);
    ext::HSD_CObjAnim(cobj);
    if (*(*cobj).aobj).curr_frame == (*(*cobj).aobj).end_frame {
        ext::HSD_GObjFree(gm_804D67B0);
        ext::HSD_GObjFree(gm_804D67B4);
        ext::mn_8022F0F0(3);
        ext::gm_801A9630();
        ext::HSD_GObjFree(gobj);
    }
}

unsafe fn setupLight() {
    let gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*ext::gm_804D67A8()).lights);
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_light_kind() as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_LObjCallback as usize, 0, 0);
}

unsafe fn setupCamera() {
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let scene = ext::gm_804D67A8();
    let cobj = ext::lb_80013B14((*(*scene).cameras).desc as *mut HSD_CameraDescPerspective);
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_camera_kind(), cobj as *mut c_void);
    ext::GObj_SetupGXLinkMax(gobj, ext::HSD_GObj_803910D8 as usize, 8);
    (*gobj).gxlink_prios = 0x801;
    ext::HSD_CObjAddAnim(cobj, *(*(*scene).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, fn_801A7A8C as usize, 0);
}

unsafe fn setupMain() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D67B0 = gobj;
    let jobj = ext::HSD_JObjLoadJoint(ext::gm_804D6798());
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_jobj_kind(), jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_JObjCallback as usize, 0xB, 0);
    ext::gm_8016895C(jobj, *(*ext::gm_804D67A8()).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_GObj_SetupProc(gobj, fn_801A7A44 as usize, 0x17);
}

unsafe fn setupCharacter() {
    let gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D67B4 = gobj;
    let jobj = ext::HSD_JObjLoadJoint((**(*ext::gm_804D67AC()).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_jobj_kind(), jobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_JObjCallback as usize, 0xB, 0);

    let char_idx = ext::gm_801A659C(ext::gm_801BEFB0());
    let child = if jobj.is_null() { core::ptr::null_mut() } else { (*jobj).child };

    let mut val = -ext::Toy_803060BC(char_idx, 0);
    HSD_JObjSetTranslateXWithMtxDirty(child, val);
    val = -ext::Toy_803060BC(char_idx, 1);
    HSD_JObjSetTranslateYWithMtxDirty(child, val);
    val = -ext::Toy_803060BC(char_idx, 2);
    HSD_JObjSetTranslateZWithMtxDirty(child, val);

    val = -(0.017453292f32 * ext::Toy_803060BC(char_idx, 5));
    HSD_JObjSetRotationYWithMtxDirty(child, val);

    let mut scale = 1.0f32 / ext::Toy_803060BC(char_idx, 3);
    val = ext::Toy_803060BC(char_idx, 4);
    scale = val * scale;
    HSD_JObjSetScaleXWithMtxDirty(child, scale);
    HSD_JObjSetScaleYWithMtxDirty(child, scale);
    HSD_JObjSetScaleZWithMtxDirty(child, scale);

    ext::HSD_GObj_SetupProc(gobj, fn_801A7A68 as usize, 0x17);

    let mut target = HSD_JObjGetChild(GET_JOBJ(gm_804D67B0));
    target = HSD_JObjGetChild(target);
    target = HSD_JObjGetChild(target);
    target = HSD_JObjGetChild(target);
    target = HSD_JObjGetChild(target);
    if target.is_null() {
        target = core::ptr::null_mut();
    } else {
        target = (*target).next;
    }

    ext::lb_8000C1C0((*gobj).hsd_obj as *mut HSD_JObj, target);
    ext::lb_8000C290((*gobj).hsd_obj as *mut HSD_JObj, target);
}

/// `gm_801A7B00()`: build the backdrop: light, camera, stage model, then the character trophy.
pub unsafe fn gm_801A7B00() {
    melee_core::ct!("gm_801A7B00");
    setupLight();
    setupCamera();
    setupMain();

    ext::gm_GetCurrentSceneEnterData();

    setupCharacter();
}
