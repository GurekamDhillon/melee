//! Port of src/melee/gm/gmregtyfall.c: the trophy-fall ("toy fall") scene shown after a mode's
//! credits: a fighter model riding a trophy pedestal, two screen-capture cameras that ping-pong
//! EFB copies into SObj textures, and the scene's enter/frame callbacks.
//!
//! Callees live in unported TUs (HSD, Player, lb, Toy...) and are `ext` stubs. All state is the
//! file's globals (`globals!`), driven by the oracle through `oracle_globals()`.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use melee_types::{
    CharacterKind, GObj_RenderFunc, GameModeKind, HSD_AObj, HSD_Archive, HSD_CObj, HSD_CameraAnim, HSD_CameraDescPerspective,
    HSD_GObj, HSD_GObjEvent, HSD_GObjProc, HSD_ImageDesc, HSD_JObj, HSD_Joint, HSD_LObj, HSD_SObj, HSD_SObjDesc, LightList,
    SceneDesc, Vec3, GM_ADVENTURE, GM_ADVENTURE_GOVER, GM_ALLSTAR, GM_CLASSIC, GM_CLASSIC_GOVER, GM_DEBUG_GOVER,
};

use super::hsd::{self, get_jobj, jobj_child, jobj_next};

/// `PAD_BUTTON_START` (dolphin/pad.h).
pub const PAD_BUTTON_START: u64 = 0x1000;

pub mod ext {
    use super::*;
    crate::oext_fns! {
        // ---- gm
        fn gm_80160474(ckind: CharacterKind, mode: GameModeKind) -> i32 = crate::tu::adapt::gm_80160474__9c9fb4;
        fn gm_801604DC(ckind: CharacterKind, mode: GameModeKind) -> *mut i8 = crate::tu::adapt::gm_801604DC__f2261c;
        fn gm_80160564(ckind: CharacterKind, mode: GameModeKind) -> *mut i8 = crate::tu::adapt::gm_80160564__4558bf;
        fn gm_801A4B60() -> () = crate::tu::adapt::gm_801A4B60__d469d9;
        fn gm_801A7B00() -> () = crate::tu::adapt::gm_801A7B00__0265f5;
        fn gm_801BEFB0() -> CharacterKind = crate::tu::adapt::gm_801BEFB0__a895fb;
        fn gm_801BEFD0() -> i32 = crate::tu::adapt::gm_801BEFD0__339fe0;
        fn gm_801BF010() -> i32 = crate::tu::adapt::gm_801BF010__72a66e;
        fn gm_801BF050() -> i32 = crate::tu::adapt::gm_801BF050__2b5449;
        fn gm_GetButtonsTriggered(slot: u8) -> u64 = crate::tu::adapt::gm_GetButtonsTriggered__dca604;
        fn gm_GetCurrentGameMode() -> u8 = crate::tu::adapt::gm_GetCurrentGameMode__f56b36;
        fn gm_GetCurrentSceneEnterData() -> *mut c_void = crate::tu::adapt::gm_GetCurrentSceneEnterData__3de8a4;
        fn gm_GetCurrentSceneExitData() -> *mut c_void = crate::tu::adapt::gm_GetCurrentSceneExitData__188bc0;
        // ---- baselib
        fn Camera_800313E0(gobj: *mut HSD_GObj, a1: u64) -> ();
        fn GObj_Create(classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj;
        fn GObj_SetupGXLink(gobj: *mut HSD_GObj, cb: GObj_RenderFunc, gx_link: u8, prio: u32) -> ();
        fn GObj_SetupGXLinkMax(gobj: *mut HSD_GObj, cb: GObj_RenderFunc, gx_link: u32) -> ();
        fn HSD_CObjAddAnim(cobj: *mut HSD_CObj, anim: *mut HSD_CameraAnim) -> () = crate::tu::adapt::HSD_CObjAddAnim__a45160;
        fn HSD_CObjAnim(cobj: *mut HSD_CObj) -> () = crate::tu::adapt::HSD_CObjAnim__1b6417;
        fn HSD_CObjEndCurrent() -> () = crate::tu::adapt::HSD_CObjEndCurrent__d69e71;
        fn HSD_CObjReqAnim(cobj: *mut HSD_CObj, frame: f32) -> () = crate::tu::adapt::HSD_CObjReqAnim__1aeeec;
        fn HSD_CObjSetCurrent(cobj: *mut HSD_CObj) -> i32 = crate::tu::adapt::HSD_CObjSetCurrent__d18716;
        fn HSD_Free(p: *mut c_void) -> () = crate::tu::adapt::HSD_Free__3ee311;
        fn HSD_GObjFree(gobj: *mut HSD_GObj) -> ();
        fn HSD_GObjGXLink_8039084C(gobj: *mut HSD_GObj) -> ();
        fn HSD_GObjObject_80390A70(gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void) -> ();
        fn HSD_GObj_80390ED0(gobj: *mut HSD_GObj, a1: u32) -> ();
        fn HSD_GObj_JObjCallback(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_GObj_JObjCallback__9cd907;
        fn HSD_GObj_LObjCallback(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_GObj_LObjCallback__aaa236;
        fn HSD_GObj_SetupProc(gobj: *mut HSD_GObj, cb: HSD_GObjEvent, priority: u8) -> *mut HSD_GObjProc;
        fn HSD_ImageDescCopyFromEFB(img: *mut HSD_ImageDesc, x: u16, y: u16, clear: u8, mipmap: i32) -> () = crate::tu::adapt::HSD_ImageDescCopyFromEFB__a5dfa4;
        fn HSD_JObjAnimAll(jobj: *mut HSD_JObj) -> ();
        fn HSD_JObjLoadJoint(joint: *mut HSD_Joint) -> *mut HSD_JObj = crate::tu::adapt::HSD_JObjLoadJoint__3efee1;
        fn HSD_SObjLib_803A4740(sobj: *mut HSD_SObj) -> () = crate::tu::adapt::HSD_SObjLib_803A4740__344956;
        fn HSD_SObjLib_803A477C(gobj: *mut HSD_GObj, desc: *mut HSD_SObjDesc, a2: u32, a3: u32, a4: u8, a5: u8) -> *mut HSD_SObj;
        fn HSD_SObjLib_803A49E0(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_SObjLib_803A49E0__00f645;
        fn HSD_SObjLib_803A54EC(gobj: *mut HSD_GObj, pass: i32) -> () = crate::tu::adapt::HSD_SObjLib_803A54EC__11e875;
        fn HSD_SObjLib_803A55DC(gobj: *mut HSD_GObj, w: u16, h: u16, a3: i32) -> () = crate::tu::adapt::HSD_SObjLib_803A55DC__ba9596;
        // ---- item / stage / ef / ft
        fn Item_80266FA8() -> () = crate::tu::adapt::Item_80266FA8__f7386b;
        fn Item_80266FCC() -> () = crate::tu::adapt::Item_80266FCC__dfa49f;
        fn Stage_8022524C() -> () = crate::tu::adapt::Stage_8022524C__8c7585;
        fn efAsync_LoadSync(a0: i32) -> ();
        fn efLib_Init() -> () = crate::tu::adapt::efLib_Init__0977f5;
        fn ftDemo_ObjAllocInit() -> ();
        // ---- player
        fn Player_80031EBC(slot: i32) -> () = crate::tu::adapt::Player_80031EBC__106f49;
        fn Player_80032768(slot: i32, v: *mut Vec3) -> () = crate::tu::adapt::Player_80032768__a41ef7;
        fn Player_80032A04(slot: i32, v: *mut Vec3) -> () = crate::tu::adapt::Player_80032A04__beba94;
        fn Player_80032BB0(slot: i32) -> f32 = crate::tu::adapt::Player_80032BB0__dd6bcc;
        fn Player_80036CF0(slot: i32) -> () = crate::tu::adapt::Player_80036CF0__06a169;
        fn Player_80036E20(ckind: CharacterKind, archive: *mut HSD_Archive, a2: i32) -> () = crate::tu::adapt::Player_80036E20__54207b;
        fn Player_80036EA0(slot: i32) -> *mut HSD_JObj = crate::tu::adapt::Player_80036EA0__cdc15f;
        fn Player_80036F34(slot: i32, a1: i32) -> () = crate::tu::adapt::Player_80036F34__e8645c;
        fn Player_GetEntity(slot: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntity__090538;
        fn Player_GetEntityAtIndex(slot: i32, idx: i32) -> *mut HSD_GObj = crate::tu::adapt::Player_GetEntityAtIndex__85ed4e;
        fn Player_LoadPlayerCoords(slot: i32, out: *mut Vec3) -> () = crate::tu::adapt::Player_LoadPlayerCoords__879cc0;
        fn Player_SetCostumeId(slot: i32, id: i32) -> () = crate::tu::adapt::Player_SetCostumeId__920e80;
        fn Player_SetFacingDirection(slot: i32, dir: f32) -> () = crate::tu::adapt::Player_SetFacingDirection__e0fc0a;
        fn Player_SetPlayerCharacter(slot: i32, ckind: CharacterKind) -> () = crate::tu::adapt::Player_SetPlayerCharacter__d99f81;
        fn Player_SetPlayerId(slot: i32, id: i32) -> () = crate::tu::adapt::Player_SetPlayerId__dba656;
        fn Player_SetScale(slot: i32, scale: f32) -> () = crate::tu::adapt::Player_SetScale__3c042c;
        fn Player_SetSlottype(slot: i32, kind: u32) -> ();
        // ---- lb / mn / ty / sfx
        fn Toy_803060BC(trophy: i32, what: i32) -> f32 = crate::tu::adapt::Toy_803060BC__946c5c;
        fn Toy_803102D0() -> () = crate::tu::adapt::Toy_803102D0__0dcc17;
        fn Toy_803124BC() -> () = crate::tu::adapt::Toy_803124BC__320358;
        fn lbAudioAx_80023694() -> i32 = crate::tu::adapt::lbAudioAx_80023694__e3b7af;
        fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = crate::tu::adapt::lbAudioAx_800237A8__95b1fa;
        fn lbAudioAx_80026F2C(a0: u32) -> () = crate::tu::adapt::lbAudioAx_80026F2C__c80d0b;
        fn lbAudioAx_8002702C(a0: u32, a1: u64) -> () = crate::tu::adapt::lbAudioAx_8002702C__208dc3;
        fn lbAudioAx_80027168() -> () = crate::tu::adapt::lbAudioAx_80027168__342616;
        fn lbAudioAx_80027648() -> () = crate::tu::adapt::lbAudioAx_80027648__4f40c8;
        fn lb_8000BA0C(jobj: *mut HSD_JObj, v: f32) -> () = crate::tu::adapt::lb_8000BA0C__a558e3;
        fn lb_8000C1C0(a: *mut HSD_JObj, b: *mut HSD_JObj) -> () = crate::tu::adapt::lb_8000C1C0__9b8c49;
        fn lb_8000C290(a: *mut HSD_JObj, b: *mut HSD_JObj) -> () = crate::tu::adapt::lb_8000C290__3ed82e;
        fn lb_80011AC4(lights: *mut *mut LightList) -> *mut HSD_LObj = crate::tu::adapt::lb_80011AC4__8bc38c;
        fn lb_800121FC(img: *mut HSD_ImageDesc, w: i32, h: i32, fmt: u32, a4: i16) -> *mut HSD_ImageDesc = crate::tu::adapt::lb_800121FC__e48dc8;
        fn lb_80013B14(desc: *mut HSD_CameraDescPerspective) -> *mut HSD_CObj = crate::tu::adapt::lb_80013B14__a63cd0;
        fn mn_8022F0F0(a0: i32) -> () = crate::tu::adapt::mn_8022F0F0__97c62c;
    }

    /// lbarchive.c, variadic in C: `lbArchive_LoadSymbols(file, out0, name1, out1, ..., NULL)`.
    /// The recorder logs the two fixed arguments only (the symbol names are literals in the caller);
    /// `rest` carries the C varargs.
    pub unsafe fn lbArchive_LoadSymbols(file: *const i8, out0: *mut c_void, _rest: &[*const c_void]) -> *mut HSD_Archive {
        use crate::oracle::{arg, ext_call, OVal, K};
        if let Some(v) = ext_call("lbArchive_LoadSymbols", &[arg(file), arg(out0)], K::P) {
            return <*mut HSD_Archive as OVal>::from_bits(v);
        }
        panic!("TODO(ext) lbArchive_LoadSymbols: its TU is not ported")
    }
}

/// `struct ImageDesc_Array` (file-local): two EFB capture slots of two images, then 0xC bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImageDesc_Array {
    pub x0: [[HSD_ImageDesc; 2]; 2],
    pub x60: [u8; 0xC],
}

const ZERO_IMG: HSD_ImageDesc = unsafe { core::mem::zeroed() };
const ZERO_IMG_ARRAY: ImageDesc_Array = ImageDesc_Array { x0: [[ZERO_IMG; 2]; 2], x60: [0; 0xC] };
const ZERO_SOBJ_DESC: HSD_SObjDesc = HSD_SObjDesc { image: null_mut(), tlut: null_mut() };

crate::globals! {
    // ---- defined by this TU
    rw pub gm_804D67AC: *mut SceneDesc = null_mut();
    rw pub gm_804D67A8: *mut SceneDesc = null_mut();
    rw pub gm_804D67A4: *mut SceneDesc = null_mut();
    rw pub gm_804D67A0: *mut SceneDesc = null_mut();
    rw pub gm_804D679C: *mut HSD_Archive = null_mut();
    rw pub gm_804D6798: *mut HSD_Joint = null_mut();
    rw pub gm_804D6794: i32 = 0;
    rw pub gm_804D6790: i32 = 0;
    rw pub gm_804D6788: HSD_SObjDesc = ZERO_SOBJ_DESC;
    rw pub gm_804D6784: i32 = 0;
    rw pub gm_804D6780: i32 = 0;
    rw pub gm_804D677C: *mut HSD_GObj = null_mut();
    rw pub gm_804D6778: *mut HSD_GObj = null_mut();
    rw pub gm_804D6774: i32 = 0;
    rw pub gm_804D6770: i32 = 0;
    rw pub gm_804D676C: *mut HSD_GObj = null_mut();
    rw pub gm_804D6768: *mut HSD_GObj = null_mut();
    rw pub gm_804D6760: HSD_SObjDesc = ZERO_SOBJ_DESC;
    rw pub gm_804D675C: i32 = 0;
    rw pub gm_804D6758: i32 = 0;
    rw pub gm_804D6754: *mut HSD_GObj = null_mut();
    rw pub gm_804D6750: i32 = 0;
    rw pub gm_804D674C: *mut HSD_GObj = null_mut();
    rw pub gm_804D6748: *mut SceneDesc = null_mut();
    rw pub gm_804D6744: *mut HSD_Archive = null_mut();
    rw pub gm_804D6740: u8 = 0;
    rw pub gm_804809D0: [HSD_ImageDesc; 2] = [ZERO_IMG; 2];
    rw pub gm_804808F8: ImageDesc_Array = ZERO_IMG_ARRAY;
    rw pub gm_80480964: ImageDesc_Array = ZERO_IMG_ARRAY;
    /// `Vec3 gm_803DB2C8 = { 0.0f, 182.0f, 0.0f }`
    ro pub gm_803DB2C8: Vec3 = Vec3 { x: 0.0, y: 182.0, z: 0.0 };
    /// `Vec3 gm_803DB2D4 = { 0.0f, 125, 0.0f }`
    ro pub gm_803DB2D4: Vec3 = Vec3 { x: 0.0, y: 125.0, z: 0.0 };
    /// `Vec3 gm_803DB2E0 = { 0.5f, -7.0f, 0.0f }`
    ro pub gm_803DB2E0: Vec3 = Vec3 { x: 0.5, y: -7.0, z: 0.0 };
    /// per-fighter model scale of the fall scene
    ro pub gm_803DB2EC: [f32; 33] = [
        0.6, 0.5, 0.75, 0.7, 0.7, 0.4, 0.8, 0.75, 0.9, 0.7, 0.6, 0.6, 0.75, 0.7, 0.65, 0.7, 0.6, 0.7, 0.8, 0.7, 0.7, 0.7, 1.0, 0.7,
        0.4, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    /// `static struct { u8 x0, x1, x2, x3; } gm_804D4278 = { 0xAA, 0xAA, 0xFF, 0xFF }`
    ro pub gm_804D4278: [u8; 4] = [0xAA, 0xAA, 0xFF, 0xFF];
    // ---- defined by other TUs (placeholders until they are ported)
    /// baselib/gobjobject.c
    rw pub HSD_GObj_CameraKind: u8 = 0;
    /// baselib/gobjobject.c
    rw pub HSD_GObj_JObjKind: u8 = 0;
    /// baselib/gobjobject.c
    rw pub HSD_GObj_LightKind: i8 = 0;
    /// baselib/sobjlib.c
    rw pub HSD_SObjLib_804D7960: u8 = 0;
}

/// gm_801A659C: the trophy lookup for the current mode.
/// (`bool` is `typedef int bool` in this codebase: the callee's int result is returned as is.)
pub unsafe fn gm_801A659C(arg0: i32) -> i32 {
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC_GOVER => ext::gm_80160474(arg0 as u32, GM_CLASSIC),
        GM_ADVENTURE_GOVER => ext::gm_80160474(arg0 as u32, GM_ADVENTURE),
        GM_DEBUG_GOVER => ext::gm_80160474(arg0 as u32, ext::gm_801BF050() as u32),
        _ => ext::gm_80160474(arg0 as u32, GM_ALLSTAR),
    }
}

/// gm_801A6630: store the exit byte and leave the scene.
pub unsafe fn gm_801A6630(arg0: i32) {
    let tmp = ext::gm_GetCurrentSceneExitData() as *mut u8;
    *tmp = arg0 as u8;
    ext::gm_801A4B60();
}

/// fn_801A6664: the player-flash proc: animate, then ping-pong the capture texture of the
/// foreground SObj (gm_804D6754) and the fading overlay SObj (gm_804D677C).
pub unsafe extern "C" fn fn_801A6664(arg0: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(get_jobj(arg0));
    if gm_804D6758 != 0 {
        ext::HSD_SObjLib_803A4740((*gm_804D6754).hsd_obj.cast());
        gm_804D6760.image = addr_of_mut!(gm_804808F8.x0[gm_804D675C as usize][0]);
        gm_804D6760.tlut = null_mut();
        let temp_r3 = ext::HSD_SObjLib_803A477C(gm_804D6754, addr_of_mut!(gm_804D6760), 0, 0, 0x80, 0);
        (*temp_r3).x10 = 60.0;
        (*temp_r3).x14 = 0.0;
        gm_804D675C = (gm_804D675C == 0) as i32;
        if gm_804D6750 != 0 {
            if gm_804D6750 > 0xFF {
                gm_804D6750 -= 6;
            } else {
                gm_804D6750 -= 3;
            }
            if gm_804D6750 < 0 {
                gm_804D6750 = 0;
            }
        }
        if gm_804D6750 > 0xFF {
            (*temp_r3).anon_3c.anon_0.x3F = 0xFF;
        } else {
            (*temp_r3).anon_3c.anon_0.x3F = gm_804D6750 as u8;
        }
    }
    if gm_804D6780 != 0 {
        if gm_804D6790 != 0 {
            gm_804D6790 -= 1;
            return;
        }
        gm_804D6790 = 7;
        ext::HSD_SObjLib_803A4740((*gm_804D677C).hsd_obj.cast());
        gm_804D6788.image = addr_of_mut!(gm_804809D0[gm_804D6784 as usize]);
        gm_804D6788.tlut = null_mut();
        let temp_r3_2 = ext::HSD_SObjLib_803A477C(gm_804D677C, addr_of_mut!(gm_804D6788), 0, 0, 0x80, 0);
        (*temp_r3_2).x10 = 60.0;
        (*temp_r3_2).x14 = 0.0;
        (*temp_r3_2).anon_3c.anon_0.x3C = gm_804D4278[0];
        (*temp_r3_2).anon_3c.anon_0.x3D = gm_804D4278[1];
        (*temp_r3_2).anon_3c.anon_0.x3E = gm_804D4278[2];
        if gm_804D6794 < 0xBE {
            gm_804D6794 += 0xA;
            if gm_804D6794 > 0xBE {
                gm_804D6794 = 0xBE;
            }
        }
        (*temp_r3_2).anon_3c.anon_0.x3F = gm_804D6794 as u8;
        gm_804D6784 = (gm_804D6784 == 0) as i32;
    }
}

/// fn_801A6844: animate the trophy model.
pub unsafe extern "C" fn fn_801A6844(gobj: *mut HSD_GObj) {
    ext::HSD_JObjAnimAll(get_jobj(gobj));
}

/// fn_801A6868: while the fighter is above the rest height, sink it by 1/300 of the drop.
pub unsafe extern "C" fn fn_801A6868(_unused: *mut HSD_GObj) {
    let mut spC = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::Player_LoadPlayerCoords(0, &mut spC);
    if spC.y >= gm_803DB2D4.y {
        spC.y += (gm_803DB2D4.y - gm_803DB2C8.y) / 300.0f32;
        ext::Player_80032A04(0, &mut spC);
    }
}

/// gm_801A68D8: spawn the demo fighter on a dummy stage.
pub unsafe fn gm_801A68D8() {
    ext::gm_GetCurrentSceneEnterData();
    hsd::Stage_InitScene(melee_types::St_Kind_Dummy, null_mut());
    ext::Item_80266FA8();
    ext::Item_80266FCC();
    ext::Stage_8022524C();
    ext::efLib_Init();
    ext::efAsync_LoadSync(0);
    ext::ftDemo_ObjAllocInit();
    ext::Player_80036E20(ext::gm_801BEFB0(), gm_804D6744, 2);
    ext::Player_80036CF0(0);
    ext::Player_SetPlayerCharacter(0, ext::gm_801BEFB0());
    ext::Player_SetCostumeId(0, ext::gm_801BEFD0());
    ext::Player_SetPlayerId(0, 0);
    ext::Player_SetSlottype(0, melee_types::Gm_PKind_Demo);
    ext::Player_SetFacingDirection(0, 1.0f32);
    ext::Player_80032768(0, addr_of!(gm_803DB2C8) as *mut Vec3);
    ext::Player_80036F34(0, 7);

    let temp_r3 = ext::Player_GetEntity(0);
    gm_804D674C = temp_r3;
    ext::lb_8000BA0C(get_jobj(temp_r3), 0.6f32);
    ext::HSD_GObj_SetupProc(temp_r3, Some(fn_801A6868), 4);

    let temp_r3_2 = ext::Player_GetEntityAtIndex(0, 1);
    if !temp_r3_2.is_null() {
        ext::lb_8000BA0C(get_jobj(temp_r3_2), 0.6f32);
    }
    let temp_f31 = ext::Player_80032BB0(0);
    let tmp = gm_803DB2EC[ext::gm_801BEFB0() as usize];
    let mult = 1.0f32 / temp_f31;
    ext::Player_SetScale(0, mult * tmp);
}

/// fn_801A6A48: capture the screen into gm_804809D0[gm_804D6784].
pub unsafe extern "C" fn fn_801A6A48(gobj: *mut HSD_GObj, arg1: i32) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) != 0 {
        ext::HSD_SObjLib_803A54EC(gobj, arg1);
        gm_804D6780 = 1;
        ext::HSD_ImageDescCopyFromEFB(addr_of_mut!(gm_804809D0[gm_804D6784 as usize]), 0x3C, 0, 0, 1);
        ext::HSD_CObjEndCurrent();
    }
}

/// fn_801A6ACC: capture colour and depth into gm_804808F8.x0[gm_804D675C].
pub unsafe extern "C" fn fn_801A6ACC(gobj: *mut HSD_GObj, _unused: i32) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) != 0 {
        ext::Camera_800313E0(gobj, 1);
        gm_804D6758 = 1;
        ext::HSD_ImageDescCopyFromEFB(addr_of_mut!(gm_804808F8.x0[gm_804D675C as usize][0]), 0x3C, 0, 0, 0);
        ext::HSD_ImageDescCopyFromEFB(addr_of_mut!(gm_804808F8.x0[gm_804D675C as usize][1]), 0x3C, 0, 1, 1);
        ext::HSD_CObjEndCurrent();
    }
}

/// fn_801A6B6C: when the camera animation finishes, free the scene.
pub unsafe extern "C" fn fn_801A6B6C(gobj: *mut HSD_GObj) {
    let cobj: *mut HSD_CObj = (*gobj).hsd_obj.cast();
    ext::HSD_CObjAnim(cobj);
    let aobj: *mut HSD_AObj = (*cobj).aobj;
    if (*aobj).curr_frame == (*aobj).end_frame {
        ext::Player_80031EBC(0);
        ext::HSD_GObjFree(gm_804D6768);
        ext::HSD_GObjFree(gm_804D6778);
        ext::HSD_GObjFree(gm_804D6754);
        ext::HSD_GObjFree(gm_804D676C);
        ext::HSD_GObjFree(gm_804D677C);
        ext::mn_8022F0F0(3);
        ext::HSD_Free(gm_80480964.x0[0][0].image_ptr);
        ext::HSD_Free(gm_80480964.x0[0][1].image_ptr);
        ext::HSD_Free(gm_80480964.x0[1][0].image_ptr);
        ext::HSD_Free(gm_80480964.x0[1][1].image_ptr);
        ext::gm_801A7B00();
        ext::HSD_GObjFree(gobj);
    }
}

/// fn_801A6C30: advance the camera animation.
pub unsafe extern "C" fn fn_801A6C30(gobj: *mut HSD_GObj) {
    ext::HSD_CObjAnim((*gobj).hsd_obj.cast());
}

/// gm_801A6C54: the first capture camera (colour+depth into gm_804808F8).
pub unsafe fn gm_801A6C54() {
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14((*(*gm_804D6748).cameras).desc.cast());
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_801A6ACC), 8);
    (*gobj).gxlink_prios = 0x61;
    ext::HSD_CObjAddAnim(cobj, *(*(*gm_804D6748).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0f32);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A6C30), 0);

    for i in 0..2usize {
        gm_804808F8.x0[i][0].image_ptr = null_mut();
        gm_804808F8.x0[i][1].image_ptr = null_mut();
        ext::lb_800121FC(addr_of_mut!(gm_804808F8.x0[i][0]), 0x1EA, 0x1E0, 5, 0);
        ext::lb_800121FC(addr_of_mut!(gm_804808F8.x0[i][1]), 0x1EA, 0x1E0, 0x16, 0);
    }
}

/// fn_801A6D78: render pass of the second capture camera.
pub unsafe extern "C" fn fn_801A6D78(gobj: *mut HSD_GObj, _unused: i32) {
    if ext::HSD_CObjSetCurrent((*gobj).hsd_obj.cast()) != 0 {
        ext::HSD_GObj_80390ED0(gobj, 7);
        ext::HSD_CObjEndCurrent();
    }
}

/// gm_801A6DC0: the second capture camera (gm_80480964), whose proc ends the scene.
pub unsafe fn gm_801A6DC0() {
    let gobj = ext::GObj_Create(0x13, 0x14, 0);
    let cobj = ext::lb_80013B14((*(*gm_804D6748).cameras).desc.cast());
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_CameraKind, cobj.cast());
    ext::GObj_SetupGXLinkMax(gobj, Some(fn_801A6D78), 0xB);
    (*gobj).gxlink_prios = 0x801;
    ext::HSD_CObjAddAnim(cobj, *(*(*gm_804D6748).cameras).anims);
    ext::HSD_CObjReqAnim(cobj, 0.0f32);
    ext::HSD_CObjAnim(cobj);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A6B6C), 0);

    for i in 0..2usize {
        gm_80480964.x0[i][0].image_ptr = null_mut();
        gm_80480964.x0[i][1].image_ptr = null_mut();
        ext::lb_800121FC(addr_of_mut!(gm_80480964.x0[i][0]), 0x1EA, 0x1E0, 5, 0);
        ext::lb_800121FC(addr_of_mut!(gm_80480964.x0[i][1]), 0x1EA, 0x1E0, 0x16, 0);
    }
}

/// gm_801A6EE4: load the scene's archives for the current mode.
pub unsafe fn gm_801A6EE4() {
    let var_r29: GameModeKind;

    ext::gm_GetCurrentSceneEnterData();
    ext::Toy_803124BC();
    ext::Toy_803102D0();
    match ext::gm_GetCurrentGameMode() as u32 {
        GM_CLASSIC_GOVER => var_r29 = GM_CLASSIC,
        GM_ADVENTURE_GOVER => var_r29 = GM_ADVENTURE,
        GM_DEBUG_GOVER => var_r29 = ext::gm_801BF050() as u32,
        _ => var_r29 = GM_ALLSTAR,
    }
    let f = ext::gm_801604DC(ext::gm_801BEFB0(), var_r29);
    let n = ext::gm_80160564(ext::gm_801BEFB0(), var_r29);
    ext::lbArchive_LoadSymbols(f, addr_of_mut!(gm_804D6798).cast(), &[n.cast(), null_mut::<c_void>() as *const c_void]);
    gm_804D6744 = ext::lbArchive_LoadSymbols(
        c"GmRegEnd".as_ptr(),
        addr_of_mut!(gm_804D6748).cast(),
        &[
            c"cut1CanimScene".as_ptr().cast(),
            addr_of_mut!(gm_804D67A8).cast(),
            c"cut2CanimScene".as_ptr().cast(),
            addr_of_mut!(gm_804D67A4).cast(),
            c"cut3CanimScene".as_ptr().cast(),
            addr_of_mut!(gm_804D67A0).cast(),
            c"cut3BgScene".as_ptr().cast(),
            null_mut::<c_void>() as *const c_void,
        ],
    );
    ext::lbArchive_LoadSymbols(
        c"GmRgStnd.dat".as_ptr(),
        addr_of_mut!(gm_804D67AC).cast(),
        &[c"standScene".as_ptr().cast(), null_mut::<c_void>() as *const c_void],
    );
    match var_r29 {
        GM_CLASSIC_GOVER | GM_CLASSIC => {
            gm_804D679C = ext::lbArchive_LoadSymbols(c"TyMcCmDs.dat".as_ptr(), null_mut(), &[]);
        }
        GM_ADVENTURE_GOVER | GM_ADVENTURE => {
            gm_804D679C = ext::lbArchive_LoadSymbols(c"TyMcR1Ds.dat".as_ptr(), null_mut(), &[]);
        }
        _ => {
            gm_804D679C = ext::lbArchive_LoadSymbols(c"TyMcR2Ds.dat".as_ptr(), null_mut(), &[]);
        }
    }
}

/// `initImages` (static inline): allocate the two overlay capture images.
pub unsafe fn initImages() {
    for i in 0..2usize {
        gm_804809D0[i].image_ptr = null_mut();
        ext::lb_800121FC(addr_of_mut!(gm_804809D0[i]), 0x1EA, 0x1E0, 5, 0);
    }
}

/// `gm_801A7070_SetupMain` (static inline): the fighter model's pedestal joint tree.
pub unsafe fn gm_801A7070_SetupMain(priority: u8) {
    let gobj = ext::GObj_Create(0xE, 0xF, priority);
    gm_804D6768 = gobj;
    let jobj = ext::HSD_JObjLoadJoint(gm_804D6798);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);
    let scale = gm_803DB2EC[ext::gm_801BEFB0() as usize];
    hsd::HSD_JObjSetScaleX(jobj, scale);
    hsd::HSD_JObjSetScaleY(jobj, scale);
    hsd::HSD_JObjSetScaleZ(jobj, scale);

    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A6664), 0x17);
    let player_jobj = ext::Player_80036EA0(0);
    ext::lb_8000C1C0(jobj_child(jobj_child(jobj_child(jobj_child(jobj)))), player_jobj);
    ext::lb_8000C290(jobj_child(jobj_child(jobj_child(jobj_child(jobj)))), player_jobj);
}

/// `getTrophyTranslateZ` (static inline).
pub unsafe fn getTrophyTranslateZ(trophy: i32) -> f32 {
    ext::Toy_803060BC(trophy, 2)
}

/// `gm_801A7070_SetupTrophy` (static inline): the trophy model, parented under the pedestal.
pub unsafe fn gm_801A7070_SetupTrophy(priority: u8) {
    let gobj = ext::GObj_Create(0xE, 0xF, priority);
    gm_804D6778 = gobj;
    let jobj = ext::HSD_JObjLoadJoint((**(*gm_804D67AC).models).joint);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_JObjCallback), 0xB, 0);

    let trophy = gm_801A659C(ext::gm_801BEFB0() as i32);
    let child = if jobj.is_null() { null_mut() } else { (*jobj).child };
    let mut val = -ext::Toy_803060BC(trophy, 0);
    hsd::HSD_JObjSetTranslateXWithMtxDirty(child, val);
    val = -ext::Toy_803060BC(trophy, 1);
    hsd::HSD_JObjSetTranslateYWithMtxDirty(child, val);
    val = -getTrophyTranslateZ(trophy);
    hsd::HSD_JObjSetTranslateZWithMtxDirty(child, val);

    val = -(0.017453292f32 * ext::Toy_803060BC(trophy, 5));
    hsd::HSD_JObjSetRotationYWithMtxDirty(child, val);
    let mut scale = 1.0f32 / ext::Toy_803060BC(trophy, 3);
    val = ext::Toy_803060BC(trophy, 4);
    scale = val * scale;
    hsd::HSD_JObjSetScaleXWithMtxDirty(child, scale);
    hsd::HSD_JObjSetScaleYWithMtxDirty(child, scale);
    hsd::HSD_JObjSetScaleZWithMtxDirty(child, scale);
    val = gm_803DB2EC[ext::gm_801BEFB0() as usize];
    hsd::HSD_JObjSetScaleX(jobj, val);
    hsd::HSD_JObjSetScaleY(jobj, val);
    hsd::HSD_JObjSetScaleZ(jobj, val);

    ext::HSD_GObj_SetupProc(gobj, Some(fn_801A6844), 0x17);
    let mut target: *mut HSD_JObj = (*gm_804D6768).hsd_obj.cast();
    target = jobj_next(jobj_child(jobj_child(jobj_child(jobj_child(jobj_child(target))))));
    ext::lb_8000C1C0((*gobj).hsd_obj.cast(), target);
    ext::lb_8000C290((*gobj).hsd_obj.cast(), target);
}

/// gm_Scene_ToyFall_OnEnter.
pub unsafe extern "C" fn gm_Scene_ToyFall_OnEnter(_unused: *mut c_void) {
    gm_804D6740 = 0x1E;
    gm_804D6750 = 0x320;
    gm_804D6754 = null_mut();
    gm_804D6758 = 0;
    gm_804D675C = 0;
    gm_804D676C = null_mut();
    gm_804D6770 = 0;
    gm_804D6774 = 0;
    gm_804D677C = null_mut();
    gm_804D6780 = 0;
    gm_804D6784 = 0;
    gm_804D6794 = 0;
    gm_804D6790 = 7;

    ext::lbAudioAx_80026F2C(0x1C);
    ext::lbAudioAx_8002702C(0x1C, 0x10000000000000);
    ext::lbAudioAx_80027168();
    ext::lbAudioAx_80027648();
    // archive loads
    gm_801A6EE4();
    // create lights
    let mut gobj = ext::GObj_Create(0xB, 3, 0);
    let lobj = ext::lb_80011AC4((*gm_804D6748).lights);
    ext::HSD_GObjObject_80390A70(gobj, HSD_GObj_LightKind as u8, lobj.cast());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_GObj_LObjCallback), 0, 0);
    gm_801A6C54();
    gm_801A6DC0();
    {
        let gobj = ext::GObj_Create(0x13, 0x14, 0);
        ext::HSD_SObjLib_803A55DC(gobj, 0x280, 0x1E0, 0xC);
        ext::HSD_GObjGXLink_8039084C(gobj);
        ext::GObj_SetupGXLinkMax(gobj, Some(fn_801A6A48), 0xC);
        (*gobj).gxlink_prios = 0x40000;
    }
    {
        let gobj = ext::GObj_Create(0x13, 0x14, 0);
        ext::HSD_SObjLib_803A55DC(gobj, 0x280, 0x1E0, 9);
        ext::HSD_GObjGXLink_8039084C(gobj);
        ext::GObj_SetupGXLinkMax(gobj, Some(ext::HSD_SObjLib_803A54EC), 9);
        (*gobj).gxlink_prios = 0x200;
    }

    initImages();

    gobj = ext::GObj_Create(0xE, 0xD, 0);
    gm_804D677C = gobj;
    ext::HSD_GObjObject_80390A70(gobj, HSD_SObjLib_804D7960, null_mut());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_SObjLib_803A49E0), 9, 0);
    gobj = ext::GObj_Create(0xE, 0x18, 0);
    gm_804D676C = gobj;
    ext::HSD_GObjObject_80390A70(gobj, HSD_SObjLib_804D7960, null_mut());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_SObjLib_803A49E0), 0x12, 0);
    gobj = ext::GObj_Create(0xE, 0x18, 0);
    gm_804D6754 = gobj;
    ext::HSD_GObjObject_80390A70(gobj, HSD_SObjLib_804D7960, null_mut());
    ext::GObj_SetupGXLink(gobj, Some(ext::HSD_SObjLib_803A49E0), 0x12, 0);
    gm_801A68D8();
    ext::gm_GetCurrentSceneEnterData();

    gm_801A7070_SetupMain(0);
    ext::gm_GetCurrentSceneEnterData();
    gm_801A7070_SetupTrophy(0);
    ext::lbAudioAx_800237A8(0x7EF40, 0x7F, 0x40);
}

/// gm_Scene_ToyFall_OnFrame: count down, then START leaves.
pub unsafe extern "C" fn gm_Scene_ToyFall_OnFrame() {
    if gm_804D6740 != 0 {
        gm_804D6740 -= 1;
    } else if ext::gm_GetButtonsTriggered(ext::gm_801BF010() as u8) & PAD_BUTTON_START != 0 {
        ext::lbAudioAx_80023694();
        hsd::sfxForward();
        ext::gm_801A4B60();
    }
}

#[allow(dead_code)]
fn _unused() {
    let _ = addr_of!(gm_804D4278);
}
