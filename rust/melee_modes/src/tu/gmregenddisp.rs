//! Port of src/melee/gm/gmregenddisp.c: the "end display" scene of regular-match mode, a trophy
//! gallery stage: fog/light/camera GObjs from the archive, one pedestal per owned fighter trophy
//! scattered on rings, and the fighter model that walks the scene.
//!
//! Callees are `ext` stubs (HSD, Toy, tyDisplay...); the file's statics are `globals!`, and the
//! tyfall TU's statics it reads are registered through `globals_ext!`.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
use core::ffi::c_void;
use core::ptr::null_mut;

use melee_core::fp::fmadds;
/// The game's own sinf/cosf (pc/gameworld/gekko_fp.c; bit-exact port in melee_runtime).
use melee_runtime::gekko::{cosf, sinf};
use melee_types::{
    CKind_Seak, CKind_Zelda, CharacterKind, DynamicModelDesc, GObj_RenderFunc, HSD_AObjDesc, HSD_Archive, HSD_CObj, HSD_CameraAnim,
    HSD_CameraDescPerspective, HSD_Fog, HSD_FogDesc, HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_Joint, HSD_JObj, HSD_LObj,
    HSD_LightAnim, HSD_MatAnimJoint, HSD_AnimJoint, HSD_ShapeAnimJoint, LightList, TyDspEntry,
};

use super::gmregtyfall::{
    gm_804D6798, gm_804D679C, gm_804D67A0, gm_804D67A4, gm_804D67AC, HSD_GObj_CameraKind, HSD_GObj_JObjKind,
    HSD_GObj_LightKind,
};
use super::hsd::{self, get_hsd_obj, get_jobj, hsd_assert, jobj_child};

pub mod ext {
    use super::*;
    crate::oext_fns! {
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, cb: GObj_RenderFunc, gx_link: u8, prio: u32) -> ();
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, cb: GObj_RenderFunc, gx_link: u32) -> ();
        fn HSD_ArchiveGetPublicAddress(archive: *mut HSD_Archive, name: *const i8) -> *mut c_void = crate::tu::adapt::HSD_ArchiveGetPublicAddress__1a942f;
        fn HSD_CObjAddAnim(cobj: *mut HSD_CObj, anim: *mut HSD_CameraAnim) -> () = crate::tu::adapt::HSD_CObjAddAnim__a45160;
        fn HSD_CObjAnim(cobj: *mut HSD_CObj) -> () = crate::tu::adapt::HSD_CObjAnim__1b6417;
        fn HSD_CObjReqAnim(cobj: *mut HSD_CObj, frame: f32) -> () = crate::tu::adapt::HSD_CObjReqAnim__1aeeec;
        fn HSD_FogInterpretAnim(fog: *mut HSD_Fog) -> ();
        fn HSD_FogLoadDesc(desc: *mut HSD_FogDesc) -> *mut HSD_Fog;
        fn HSD_FogReqAnim(fog: *mut HSD_Fog, frame: f32) -> ();
        fn HSD_Fog_8037DE7C(fog: *mut HSD_Fog, aobjdesc: *mut HSD_AObjDesc) -> ();
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObj_803910D8(gobj: *mut HSD_GObj, pass: i32) -> ();
        fn HSD_GObj_FogCallback(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_GObj_FogCallback__2aeaad;
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_GObj_SetupProc(gobj: *mut HSD_GObj, cb: HSD_GObjEvent, priority: u8) -> *mut HSD_GObjProc;
        fn HSD_JObjAddAnimAll(jobj: *mut HSD_JObj, anim: *mut HSD_AnimJoint, mat: *mut HSD_MatAnimJoint, shape: *mut HSD_ShapeAnimJoint) -> ();
        fn HSD_JObjAddChild(parent: *mut HSD_JObj, child: *mut HSD_JObj) -> ();
        fn HSD_JObjAlloc() -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjAlloc__5ecba4;
        fn HSD_JObjAnimAll(jobj: *mut HSD_JObj) -> ();
        fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_JObjReqAnimAll(jobj: *mut HSD_JObj, frame: f32) -> ();
        fn HSD_LObjAddAnimAll(lobj: *mut HSD_LObj, anim: *mut HSD_LightAnim) -> ();
        fn HSD_LObjAnimAll(lobj: *mut HSD_LObj) -> ();
        fn HSD_LObjReqAnimAll(lobj: *mut HSD_LObj, frame: f32) -> ();
        fn HSD_Randf() -> f32;
        fn HSD_Randi(max: i32) -> i32;
        fn Toy_803048C0(trophy: i32) -> i32 = crate::tu::adapt::Toy_803048C0__c4f25d;
        fn Toy_803060BC(trophy: i32, what: i32) -> f32 = crate::tu::adapt::Toy_803060BC__946c5c;
        fn gm_8016895C(jobj: *mut HSD_JObj, model: *mut DynamicModelDesc, a2: i32) -> () = crate::tu::adapt::gm_8016895C__f7ee90;
        fn gm_801A4B60() -> () = crate::tu::adapt::gm_801A4B60__d469d9;
        fn gm_801A4BA8() -> u32 = crate::tu::adapt::gm_801A4BA8__1c9c2c;
        /// gmregtyfall.c (ported: forwarded)
        fn gm_801A659C(arg0: i32) -> i32 = super::super::gmregtyfall::gm_801A659C;
        fn gm_801BEFB0() -> CharacterKind = crate::tu::adapt::gm_801BEFB0__a895fb;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gm_GetCurrentSceneEnterData() -> *mut c_void = crate::tu::adapt::gm_GetCurrentSceneEnterData__3de8a4;
        fn lbAudioAx_800237A8(sfx_id: i32, sfx_vol: i32, sfx_pan: i32) -> i32 = crate::tu::adapt::lbAudioAx_800237A8__95b1fa;
        fn lb_8000C1C0(a: *mut HSD_JObj, b: *mut HSD_JObj) -> () = crate::tu::adapt::lb_8000C1C0__9b8c49;
        fn lb_8000C290(a: *mut HSD_JObj, b: *mut HSD_JObj) -> () = crate::tu::adapt::lb_8000C290__3ed82e;
        fn lb_80011AC4(lights: *mut *mut LightList) -> *mut HSD_LObj = crate::tu::adapt::lb_80011AC4__8bc38c;
        fn lb_80013B14(desc: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = crate::tu::adapt::lb_80013B14__a63cd0;
        fn tyDisplay_8031B9DC(id: i32) -> *mut TyDspEntry = crate::tu::adapt::tyDisplay_8031B9DC__051a66;
        fn tyDisplay_8031BB34(id: i8) -> i32 = crate::tu::adapt::tyDisplay_8031BB34__c9308d;
        fn tyDisplay_8031BB94(id: i8) -> *mut i8 = crate::tu::adapt::tyDisplay_8031BB94__d3b220;
    }
}

crate::globals! {
    /// `static HSD_GObj* gm_80480A00[0x34]`: the trophy pedestal GObjs by fighter.
    rw pub gm_80480A00: [*mut HSD_GObj; 0x34] = [null_mut(); 0x34];
    /// `static s32 gm_80480AD0[0x1A]`: per-fighter random sort keys.
    rw pub gm_80480AD0: [i32; 0x1A] = [0; 0x1A];
    rw pub gm_804D67B8: *mut HSD_GObj = null_mut();
    rw pub gm_804D67BC: *mut HSD_GObj = null_mut();
    rw pub gm_804D67C0: *mut HSD_GObj = null_mut();
    rw pub gm_804D67C4: i32 = 0;
}

// baselib/gobjobject.c's real kind (a zeroed stand-in made this TU's fog objects kind 0)
pub use melee_runtime::tu::gobj_native::HSD_GObj_FogKind;

crate::globals_ext! {
    rw gm_804D6798 = super::gmregtyfall::gm_804D6798: *mut HSD_Joint;
    rw gm_804D679C = super::gmregtyfall::gm_804D679C: *mut HSD_Archive;
    rw gm_804D67A0 = super::gmregtyfall::gm_804D67A0: *mut melee_types::SceneDesc;
    rw gm_804D67A4 = super::gmregtyfall::gm_804D67A4: *mut melee_types::SceneDesc;
    rw gm_804D67AC = super::gmregtyfall::gm_804D67AC: *mut melee_types::SceneDesc;
    rw HSD_GObj_CameraKind = super::gmregtyfall::HSD_GObj_CameraKind: u8;
    rw HSD_GObj_JObjKind = super::gmregtyfall::HSD_GObj_JObjKind: u8;
    rw HSD_GObj_LightKind = super::gmregtyfall::HSD_GObj_LightKind: i8;
}

/// `fn_801A7FB4_inline` / `fn_801A7FB4_inline2` (gm/inlines.h, identical bodies): how many of the
/// 26 trophies are owned.
unsafe fn owned_trophies() -> i32 {
    let mut count = 0;
    let mut i = count;
    while i < 0x1A {
        if ext::Toy_803048C0(ext::gm_801A659C(i)) != 0 {
            count += 1;
        }
        i += 1;
    }
    count
}

/// fn_801A7FB4: fog proc; the fog animation plays to a frame that depends on how many trophies
/// are shown (185 up to 5, 200 up to 13, else to the end).
pub unsafe extern "C" fn fn_801A7FB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A7FB4");
    let fog = get_hsd_obj(gobj) as *mut HSD_Fog;

    ext::gm_GetCurrentGameMode();
    let count = owned_trophies();

    if count <= 5 {
        if (*(*fog).aobj).curr_frame < 185.0f32 {
            ext::HSD_FogInterpretAnim(fog);
        }
        return;
    }

    ext::gm_GetCurrentGameMode();
    let count = owned_trophies();

    if count <= 0xD {
        if (*(*fog).aobj).curr_frame < 200.0f32 {
            ext::HSD_FogInterpretAnim(fog);
        }
    } else {
        ext::HSD_FogInterpretAnim(fog);
    }
}

/// fn_801A80CC: light proc.
pub unsafe extern "C" fn fn_801A80CC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A80CC");
    ext::HSD_LObjAnimAll(get_hsd_obj(gobj) as *mut HSD_LObj);
}

/// fn_801A80F0: model proc.
pub unsafe extern "C" fn fn_801A80F0(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A80F0");
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// gm_801A8114: place the fighter model's child joint from the trophy's table.
pub unsafe fn gm_801A8114(arg0: *mut HSD_JObj, arg1: i32) {
    melee_core::ct!("gm_801A8114");
    let transJobj = if arg0.is_null() { null_mut() } else { (*arg0).child };

    hsd::HSD_JObjSetTranslateX(transJobj, ext::Toy_803060BC(arg1, 0));
    hsd::HSD_JObjSetTranslateY(transJobj, ext::Toy_803060BC(arg1, 1));
    hsd::HSD_JObjSetTranslateZ(transJobj, ext::Toy_803060BC(arg1, 2));

    hsd::HSD_JObjSetRotationY(transJobj, 0.017453292f32 * ext::Toy_803060BC(arg1, 5));

    let a = ext::Toy_803060BC(arg1, 3);
    let b = ext::Toy_803060BC(arg1, 4);
    let scale = a * (1.0f32 / b);
    hsd_assert(!transJobj.is_null(), 92, c"transJobj");

    hsd::HSD_JObjSetScaleX(transJobj, scale);
    hsd::HSD_JObjSetScaleY(transJobj, scale);
    hsd::HSD_JObjSetScaleZ(transJobj, scale);
}

/// fn_801A851C: backdrop proc.
pub unsafe extern "C" fn fn_801A851C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A851C");
    ext::HSD_JObjAnimAll(get_jobj(gobj));
    hsd::HSD_JObjSetTranslateZ(get_jobj(gobj), 2.0f32);
    if ext::gm_801A4BA8() == 0x174 {
        ext::lbAudioAx_800237A8(0x7EF41, 0x7F, 0x40);
    }
}

/// gm_801A85E4: put pedestal `arg1` of 0..0x18 on one of three rings (or a fixed spot), with a
/// random angular jitter. (`arg2` is only scratch in the C.) Each `a*b + c` is a fused
/// multiply-add in the shipping build (`fmuladd` -> `fmadds`); the left multiply is the fused one.
pub unsafe fn gm_801A85E4(jobj: *mut HSD_JObj, arg1: i32, _arg2: i32) {
    melee_core::ct!("gm_801A85E4");
    if arg1 <= 4 {
        let mut arg2 = 4 - arg1;
        let mut idx = arg2 + 2;
        let temp_idx = idx;
        if temp_idx > 4 {
            arg2 = 4 - idx;
            idx = arg2 + 2;
        }
        let _ = arg2;
        let r = 2.0f32 * ext::HSD_Randf();
        let angle = 0.017453292f32 * (fmadds(45.0f32, idx as f32, r) - 1.0f32);
        hsd::HSD_JObjSetTranslateX(jobj, 25.0f32 * cosf(angle));
        let s = -sinf(angle);
        hsd::HSD_JObjSetTranslateZ(jobj, 0.9f32 * fmadds(25.0f32, s, -18.0f32));
    } else if arg1 <= 0xC {
        let mut arg2 = 0xC - arg1;
        let temp_idx = arg2 + 8;
        let mut idx = temp_idx;
        if temp_idx > 0xC {
            arg2 = 0xC - idx;
            idx = arg2 + 8;
        }
        let _ = arg2;
        let r = 4.0f32 * ext::HSD_Randf();
        let angle = 0.017453292f32 * (fmadds(25.714285f32, (idx - 5) as f32, r) - 2.0f32);
        hsd::HSD_JObjSetTranslateX(jobj, 50.0f32 * cosf(angle));
        let s = -sinf(angle);
        hsd::HSD_JObjSetTranslateZ(jobj, 0.9f32 * fmadds(50.0f32, s, -18.0f32));
    } else if arg1 <= 0x15 {
        let mut arg2 = 0x15 - arg1;
        let temp_idx = arg2 + 0x11;
        let mut idx = temp_idx;
        if temp_idx > 0x15 {
            arg2 = 0x15 - idx;
            idx = arg2 + 0x11;
        }
        let _ = arg2;
        let r = ext::HSD_Randf();
        let angle = 0.017453292f32 * (fmadds(22.5f32, (idx - 0xD) as f32, r) - 0.5f32);
        hsd::HSD_JObjSetTranslateX(jobj, 75.0f32 * cosf(angle));
        let s = -sinf(angle);
        hsd::HSD_JObjSetTranslateZ(jobj, 0.8f32 * fmadds(75.0f32, s, -18.0f32));
    } else if arg1 == 0x16 {
        hsd::HSD_JObjSetTranslateX(jobj, -80.0f32);
        hsd::HSD_JObjSetTranslateZ(jobj, -70.0f32);
    } else if arg1 == 0x17 {
        hsd::HSD_JObjSetTranslateX(jobj, 75.0f32);
        hsd::HSD_JObjSetTranslateZ(jobj, -75.0f32);
    } else {
        hsd::HSD_JObjSetTranslateX(jobj, 90.0f32);
        hsd::HSD_JObjSetTranslateZ(jobj, -55.0f32);
    }
}

/// gm_801A8D54: the owned trophies' fighter ids (other than the shown fighter), sorted by
/// descending random key, 0x1A-padded.
pub unsafe fn gm_801A8D54(arg0: *mut i32) {
    melee_core::ct!("gm_801A8D54");
    let mut sp84 = [0i32; 0x1A];

    for i in 0..0x1A {
        *arg0.add(i) = 0x1A;
        sp84[i] = 0x1A;
    }

    let mut count = 0usize;
    for i in 0..0x1Ai32 {
        if (i.wrapping_sub(0x12) as u32) <= 1 {
            ext::gm_GetCurrentGameMode();
            if ext::Toy_803048C0(ext::gm_801A659C(i)) != 0 && ext::gm_801BEFB0() != CKind_Zelda && ext::gm_801BEFB0() != CKind_Seak {
                sp84[count] = i;
                count += 1;
            }
        } else {
            ext::gm_GetCurrentGameMode();
            if ext::Toy_803048C0(ext::gm_801A659C(i)) != 0 && i as u32 != ext::gm_801BEFB0() {
                sp84[count] = i;
                count += 1;
            }
        }
    }

    for i in 0..0x1Ausize {
        if sp84[i] != 0x1A {
            for j in (i + 1)..0x1A {
                if 0x1A != sp84[j] {
                    let lt = gm_80480AD0[sp84[i] as usize] < gm_80480AD0[sp84[j] as usize];
                    if (if lt { 1 } else { 2 }) == 2 {
                        let temp_i = sp84[i];
                        sp84[i] = sp84[j];
                        sp84[j] = temp_i;
                    }
                }
            }
        }
    }

    for i in 0..0x1A {
        *arg0.add(i) = sp84[i];
    }
}


/// `gm_801A9094_get_bg` (static inline): the pedestal stand's joint.
pub unsafe fn gm_801A9094_get_bg() -> *mut HSD_Joint {
    melee_core::ct!("gm_801A9094_get_bg");
    ext::HSD_ArchiveGetPublicAddress(gm_804D679C, c"ToyDspStand_Top_joint".as_ptr()) as *mut HSD_Joint
}

/// `gm_801A9094_create_gobj` (static inline).
pub unsafe fn gm_801A9094_create_gobj() -> *mut HSD_GObj {
    melee_core::ct!("gm_801A9094_create_gobj");
    ext::GObj_Create(0xE, 0xF, 0)
}

/// gm_801A9094: one pedestal (trophy model + stand) per entry, highest slot first.
pub unsafe fn gm_801A9094() {
    melee_core::ct!("gm_801A9094");
    let mut sp8C = [0i32; 0x1A];

    gm_801A8D54(sp8C.as_mut_ptr());
    let mut i: i32 = 0x19;
    loop {
        if sp8C[i as usize] != 0x1A {
            let dsp = ext::tyDisplay_8031B9DC(ext::gm_801A659C(gm_801A9094_get_entry(&sp8C, i)));
            let joint = ext::HSD_ArchiveGetPublicAddress(gm_804D679C, ext::tyDisplay_8031BB34((*dsp).x04 as i8) as isize as *const i8) as *mut HSD_Joint;
            let matanim = ext::HSD_ArchiveGetPublicAddress(gm_804D679C, ext::tyDisplay_8031BB94((*dsp).x04 as i8)) as *mut HSD_MatAnimJoint;
            let bg_joint = gm_801A9094_get_bg();
            let gobj = gm_801A9094_create_gobj();
            gm_80480A00[sp8C[i as usize] as usize] = gobj;
            let root = ext::HSD_JObjAlloc();
            ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, root.cast());
            ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
            hsd::HSD_JObjSetScaleX(root, 1.8f32);
            hsd::HSD_JObjSetScaleY(root, 1.8f32);
            hsd::HSD_JObjSetScaleZ(root, 1.8f32);
            hsd_assert(!joint.is_null(), 311, c"joint");
            let jobj = ext::HSD_JObjLoadJoint(joint);
            hsd_assert(!jobj.is_null(), 313, c"jobj");
            ext::HSD_JObjAddChild(root, jobj);
            ext::HSD_JObjAddAnimAll(jobj, null_mut(), matanim, null_mut());
            ext::HSD_JObjReqAnimAll(jobj, (*dsp).x05 as f32);
            ext::HSD_JObjAnimAll(jobj);
            hsd::HSD_JObjSetTranslateX(jobj, (*dsp).x08);
            hsd::HSD_JObjSetTranslateZ(jobj, (*dsp).x0C);
            ext::HSD_JObjAddChild(root, ext::HSD_JObjLoadJoint(bg_joint));
            gm_801A85E4(root, i, sp8C[i as usize]);
        }
        i -= 1;
        if i < 0 {
            break;
        }
    }
}

/// fn_801A9498: model proc.
pub unsafe extern "C" fn fn_801A9498(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A9498");
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// fn_801A94BC: camera proc; plays the camera animation to a frame that depends on the trophy
/// count (160 / 190 / end), then counts down gm_804D67C4 and leaves.
pub unsafe extern "C" fn fn_801A94BC(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_801A94BC");
    let cobj = get_hsd_obj(gobj) as *mut HSD_CObj;
    ext::gm_GetCurrentGameMode();
    let var_r31 = owned_trophies();
    if var_r31 <= 5 {
        if (*(*cobj).aobj).curr_frame < 160.0f32 {
            ext::HSD_CObjAnim(cobj);
        } else {
            gm_804D67C4 -= 1;
            if gm_804D67C4 == 0 {
                ext::gm_801A4B60();
            }
        }
        return;
    }

    ext::gm_GetCurrentGameMode();
    let var_r31 = owned_trophies();
    if var_r31 <= 0xD {
        if (*(*cobj).aobj).curr_frame < 190.0f32 {
            ext::HSD_CObjAnim(cobj);
        } else {
            gm_804D67C4 -= 1;
            if gm_804D67C4 == 0 {
                ext::gm_801A4B60();
            }
        }
    } else if (*(*cobj).aobj).curr_frame < (*(*cobj).aobj).end_frame {
        ext::HSD_CObjAnim(cobj);
    } else {
        gm_804D67C4 -= 1;
        if gm_804D67C4 == 0 {
            ext::gm_801A4B60();
        }
    }
}

/// `gm_801A8114_inline` (static): a call to gm_801A8114.
pub unsafe fn gm_801A8114_inline(arg0: *mut HSD_JObj, arg1: i32) {
    melee_core::ct!("gm_801A8114_inline");
    gm_801A8114(arg0, arg1);
}

/// `gm_801A9630_init` (static inline).
pub unsafe fn gm_801A9630_init(randoms: *mut *mut i32) {
    melee_core::ct!("gm_801A9630_init");
    for i in 0..0x1Ausize {
        gm_80480A00[i] = null_mut();
    }

    gm_804D67C4 = 0xB4;
    *randoms = core::ptr::addr_of_mut!(gm_80480AD0).cast::<i32>();

    let mut i = 0;
    while i < 0x1A {
        ext::gm_GetCurrentGameMode();
        if ext::Toy_803048C0(ext::gm_801A659C(i)) != 0 {
            **randoms = ext::HSD_Randi(0x2710);
        } else {
            **randoms = 0;
        }
        i += 1;
        *randoms = (*randoms).add(1);
    }
}

/// `gm_801A9630_fog` (static inline).
pub unsafe fn gm_801A9630_fog() {
    melee_core::ct!("gm_801A9630_fog");
    let gobj = ext::GObj_Create(0xE, 3, 0);
    let fog = ext::HSD_FogLoadDesc((*(*gm_804D67A4).fogs).desc);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_FogKind as u8, fog.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_FogCallback), 0, 0);
    ext::HSD_Fog_8037DE7C(fog, (**(*(*gm_804D67A4).fogs).anims).aobjdesc);
    ext::HSD_FogReqAnim(fog, 0.0f32);
    ext::HSD_FogInterpretAnim(fog);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A7FB4), 0x17);
}

/// `gm_801A9630_light` (static inline).
pub unsafe fn gm_801A9630_light() {
    melee_core::ct!("gm_801A9630_light");
    let gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*gm_804D67A4).lights);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_LightKind as u8, lobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);
    ext::HSD_LObjAddAnimAll(lobj, *(**(*gm_804D67A4).lights).anims);
    ext::HSD_LObjReqAnimAll(lobj, 0.0f32);
    ext::HSD_LObjAnimAll(lobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A80CC), 0x17);
}

/// `gm_801A9630_camera` (static inline).
pub unsafe fn gm_801A9630_camera() {
    melee_core::ct!("gm_801A9630_camera");
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14((*(*gm_804D67A4).cameras).desc.cast());
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_GObj_803910D8), 8);
    (*gobj).gxlink_prios = 0x801;
    ext::HSD_CObjAddAnim(cobj, *(*(*gm_804D67A4).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0f32);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A94BC), 0);
}

/// gm_801A9630: build the whole scene.
pub unsafe fn gm_801A9630() {
    melee_core::ct!("gm_801A9630");
    let mut randoms: *mut i32 = null_mut();

    gm_801A9630_init(&mut randoms);

    // Fog GObj
    gm_801A9630_fog();

    // Light GObj
    gm_801A9630_light();

    // Camera GObj
    gm_801A9630_camera();

    // Background JObj GObj 1
    let mut gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D67C0 = gobj;
    let mut jobj = ext::HSD_JObjLoadJoint((**(*gm_804D67A0).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::gm_8016895C(jobj, *(*gm_804D67A0).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A9498), 0x17);

    // Background JObj GObj 2
    gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D67BC = gobj;
    jobj = ext::HSD_JObjLoadJoint((**(*gm_804D67AC).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    ext::gm_8016895C(jobj, *(*gm_804D67A4).models, 0);
    ext::HSD_JObjReqAnimAll(jobj, 0.0f32);
    ext::HSD_JObjAnimAll(jobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A851C), 0x17);

    // Character display setup
    gm_801A9094();
    ext::gm_GetCurrentSceneEnterData();

    // Character JObj GObj 3
    gobj = ext::GObj_Create(0xE, 0xF, 0);
    gm_804D67B8 = gobj;
    jobj = ext::HSD_JObjLoadJoint(gm_804D6798);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    gm_801A8114_inline(jobj, ext::gm_801A659C(ext::gm_801BEFB0() as i32));

    // Walk JObj tree to find constraint target (3 levels deep)
    let child = jobj_child(get_jobj(gm_804D67BC));
    let mut target = jobj_child(child);
    if target.is_null() {
        target = null_mut();
    } else {
        target = (*target).child;
    }

    ext::lb_8000C1C0(jobj, target);
    ext::lb_8000C290(jobj, target);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A80F0), 0x17);
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gmregenddisp.c `gm_801A9094_get_entry`.
pub fn gm_801A9094_get_entry(entries: &[i32], i: i32) -> i32 {
    melee_core::ct!("gm_801A9094_get_entry");
    entries[i as usize]
}
