//! Port of src/melee/ef/efalt.c: `efAlt_Spawn`, the dispatcher for gfx ids 0x479..0x4B9.
//!
//! Callees go through [`EfAltHost`] (the [`EfHost`] of efasync plus the creators only efalt
//! uses); defaults forward to `mod ext` TODO stubs named like the C functions. The only float
//! arithmetic is `HSD_JObjGetTranslationZ(jobj) - 6.0f` (0x49D, plain f32) and the ±M_PI_2
//! narrowing shared with efasync ([`facing_rot`]).
#![allow(non_snake_case, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;

use melee_types::{EF_Effect, Fighter, FtPart_R3rdNa, FtPart_TransN, HSD_GObj, HSD_Generator, HSD_JObj, Vec3};

use super::efasync::jobj::*;
use super::efasync::{drain_anim_queue, facing_rot, EfHost, VaList, EF_LOADKIND_SYNC, M_PI_2};

pub mod ext {
    use super::*;
    pub unsafe fn efLib_Create_Attach_Scale(_id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect { { return unsafe { crate::tu::eflib::efLib_Create_Attach_Scale(_id as _, _g as _, _j as _) as _ }; } }
    pub unsafe fn efLib_Create_AttachChild_Scale(_id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect { { return unsafe { crate::tu::eflib::efLib_Create_AttachChild_Scale(_id as _, _g as _, _j as _) as _ }; } }
    pub unsafe fn efLib_Create_Attach_Scale_FacingDir(_id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect { { return unsafe { crate::tu::eflib::efLib_Create_Attach_Scale_FacingDir(_id as _, _g as _, _j as _) as _ }; } }
    pub unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDir(_id: i32, _vl: VaList<'_>) -> *mut HSD_Generator { panic!("TODO(group2 eflib) efLib_CreateGenerator_AppSRT_SetFacingDir") }
    pub unsafe fn efLib_CreateGenerator_Attach_Scale(_id: i32, _vl: VaList<'_>, _g: *mut HSD_GObj) -> *mut HSD_Generator { panic!("TODO(group2 eflib) efLib_CreateGenerator_Attach_Scale") }
    pub unsafe extern "C" fn efLib_Cb_ftMr_SpecialLw(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_ftMr_SpecialLw(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetScaleRotY_FromFighter(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetScaleRotY_FromFighter(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromFighter(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetRotYZ_FromFighter(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetRotYZ_FromParamZ_FighterDir(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetRotYZ_FromParamZ_FighterDir(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetRotYAndTransition(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetRotYAndTransition(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetRotY_FromFighterDir(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetRotY_FromFighterDir(_e as _); } return; } }
    pub unsafe extern "C" fn efLib_Cb_SetJObjOffsetZ(_e: *mut EF_Effect) { { unsafe { let _ = crate::tu::eflib::efLib_Cb_SetJObjOffsetZ(_e as _); } return; } }
}

/// efalt.c's callees beyond [`EfHost`].
pub unsafe trait EfAltHost: EfHost {
    unsafe fn efLib_Create_Attach_Scale(&mut self, id: u32, g: *mut HSD_GObj, j: *mut HSD_JObj) -> *mut EF_Effect { ext::efLib_Create_Attach_Scale(id, g, j) }
    unsafe fn efLib_Create_AttachChild_Scale(&mut self, id: u32, g: *mut HSD_GObj, j: *mut HSD_JObj) -> *mut EF_Effect { ext::efLib_Create_AttachChild_Scale(id, g, j) }
    unsafe fn efLib_Create_Attach_Scale_FacingDir(&mut self, id: u32, g: *mut HSD_GObj, j: *mut HSD_JObj) -> *mut EF_Effect { ext::efLib_Create_Attach_Scale_FacingDir(id, g, j) }
    unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDir(&mut self, id: i32, vl: VaList<'_>) -> *mut HSD_Generator { ext::efLib_CreateGenerator_AppSRT_SetFacingDir(id, vl) }
    unsafe fn efLib_CreateGenerator_Attach_Scale(&mut self, id: i32, vl: VaList<'_>, g: *mut HSD_GObj) -> *mut HSD_Generator { ext::efLib_CreateGenerator_Attach_Scale(id, vl, g) }
}

/// 0x49D's child offset: `HSD_JObjGetTranslationZ(jobj) - 6.0f`.
#[inline]
pub fn offset_z(z: f32) -> f32 {
    z - 6.0
}

/// efAlt_Spawn.
pub unsafe fn efAlt_Spawn<H: EfAltHost + ?Sized>(h: &mut H, gfx_id: i32, gobj: *mut HSD_GObj, vl: &mut VaList<'_>) -> *mut c_void {
    let mut ret: *mut c_void = null_mut();
    h.ef_data().load_kind = EF_LOADKIND_SYNC;

    macro_rules! gen {
        ($id:expr) => {{ ret = h.efLib_CreateGenerator($id, vl.vec3()).cast(); }};
    }
    macro_rules! efac {
        ($b:expr, $id:expr) => {{ ret = h.hsd_8039EFAC(0, $b, $id, vl.jobj()).cast(); }};
    }
    macro_rules! set {
        ($e:expr, $f:expr) => {{ let e = $e; ret = e.cast(); if !e.is_null() { (*e).update = Some($f); } }};
    }
    // `rotate_y = *va_arg(f32*) < 0 ? -M_PI_2 : M_PI_2` on the new effect.
    macro_rules! facing {
        ($e:expr) => {{
            let e: *mut EF_Effect = $e;
            ret = e.cast();
            if !e.is_null() {
                let r = facing_rot(*vl.f32p());
                HSD_JObjSetRotationY(h, GET_JOBJ((*e).gobj), r);
            }
            e
        }};
    }
    macro_rules! pair_facing_dir {
        ($a:expr, $b:expr) => {{
            let e = h.efLib_Create_Attach_Scale_FacingDir($a, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let j = vl.jobj();
                (*e).next = h.efLib_Create_Attach_Scale_FacingDir($b, gobj, j);
            }
        }};
    }
    macro_rules! pos_rot_y_pi2_z {
        ($id:expr) => {{
            let e = h.efLib_Create_Attach_Pos($id, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                HSD_JObjSetRotationY(h, GET_JOBJ((*e).gobj), M_PI_2 as f32);
                let z = *vl.f32p();
                HSD_JObjSetRotationZ(h, GET_JOBJ((*e).gobj), z);
            }
        }};
    }
    macro_rules! attach_facing_then_efac {
        ($id:expr, $bank:expr, $efac_id:expr) => {{
            let j = vl.jobj();
            let e = h.efLib_Create_Attach($id, gobj, j);
            facing!(e);
            h.hsd_8039EFAC(0, $bank, $efac_id, j);
        }};
    }
    macro_rules! child_copy_scale {
        ($id:expr) => {{
            let e = h.efLib_Create_AttachChild($id, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                let mut scale = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                HSD_JObjGetScale(GET_JOBJ(gobj), &mut scale);
                hsd_assert(!GET_JOBJ((*e).gobj).is_null(), "jobj");
                HSD_JObjSetScale(h, GET_JOBJ((*e).gobj), &scale);
            }
        }};
    }

    match gfx_id {
        0x479 => gen!(0x3F2),
        0x47A => attach_facing_then_efac!(0x3E8, 1, 0x3E9),
        0x47B => gen!(0x3EB),
        0x47C => set!(h.efLib_Create_Attach_Scale(0x3E9, gobj, vl.jobj()), ext::efLib_Cb_ftMr_SpecialLw),
        0x47D => efac!(1, 0x3F0),
        0x47E => efac!(1, 0x3F1),
        0x47F => efac!(2, 0x7D4),
        0x480 => efac!(2, 0x7D2),
        0x481 => efac!(2, 0x7D3),
        0x482 => set!(h.efLib_Create_Attach(0x7D0, gobj, vl.jobj()), ext::efLib_Cb_SetScaleRotY_FromFighter),
        0x483 => gen!(0x7D7),
        0x484 => efac!(2, 0x7DB),
        0x485 => efac!(2, 0x7DE),
        0x486 => { let e = h.efLib_Create_Attach_Pos(0x7D1, gobj, vl.vec3()); facing!(e); }
        0x487 => ret = h.efLib_Create_AttachChild_Scale(0x7D2, gobj, vl.jobj()).cast(),
        0x488 => ret = h.efLib_Create_Attach_Scale(0xBB8, gobj, vl.jobj()).cast(),
        0x489 => ret = h.efLib_Create_Attach_Scale(0xBB9, gobj, vl.jobj()).cast(),
        0x48A => ret = h.efLib_Create_Attach_Scale(0xBBA, gobj, vl.jobj()).cast(),
        0x48B => ret = h.efLib_Create_Attach(0xBBB, gobj, vl.jobj()).cast(),
        0x48C => set!(h.efLib_Create_Attach(0xBBC, gobj, vl.jobj()), ext::efLib_Cb_SetRotYZ_FromFighter),
        0x48D => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDir(0xBC0, *vl).cast(),
        0x48E => pos_rot_y_pi2_z!(0xBBD),
        0x48F => pair_facing_dir!(0xFA0, 0xFA1),
        0x490 => {
            let e = h.efLib_Create_Attach(0xFA2, gobj, vl.jobj());
            ret = e.cast();
            if !e.is_null() {
                (*e).params.z = *vl.f32p();
                (*e).update = Some(ext::efLib_Cb_SetRotYZ_FromParamZ_FighterDir);
            }
        }
        0x491 => ret = h.efLib_Create_Attach(0xFA4, gobj, vl.jobj()).cast(),
        0x492 => { let e = h.efLib_Create_Attach_Scale(0xFA3, gobj, vl.jobj()); facing!(e); }
        0x493 => { let e = h.efLib_Create_Attach_Scale(0xFA5, gobj, vl.jobj()); facing!(e); }
        0x494 => {
            let flags: u16 = 0x41;
            let fp = (*gobj).user_data as *mut Fighter;
            let j1 = (*(*fp).parts.add(FtPart_R3rdNa as usize)).joint;
            let j2 = (*(*fp).parts.add(FtPart_TransN as usize)).joint;
            let e1 = h.efLib_Create_AttachChild(0x1388, gobj, j1);
            ret = e1.cast();
            if !e1.is_null() {
                (*e1).update = Some(ext::efLib_Cb_SetRotYAndTransition);
                (*e1).lifetime = flags;
                (*e1).user_data = null_mut();
                let n = h.efLib_Create_Attach(0x1389, gobj, j1);
                (*e1).next = n;
                if !n.is_null() {
                    let e2 = (*e1).next;
                    (*e2).update = Some(ext::efLib_Cb_SetRotYAndTransition);
                    (*e2).lifetime = flags;
                    (*e2).user_data = fp.cast();
                    let n = h.efLib_Create_Attach(0x138A, gobj, j2);
                    (*e2).next = n;
                    if !n.is_null() {
                        let e3 = (*e2).next;
                        (*e3).update = Some(ext::efLib_Cb_SetRotYAndTransition);
                        (*e3).lifetime = flags;
                        (*e3).user_data = fp.cast();
                    }
                }
            }
        }
        0x495 => { let e = h.efLib_Create_Attach_Scale(0x138B, gobj, vl.jobj()); facing!(e); }
        0x496 => ret = h.efLib_Create_Attach(0x138C, gobj, vl.jobj()).cast(),
        0x497 => set!(h.efLib_Create_Attach(0x138D, gobj, vl.jobj()), ext::efLib_Cb_SetRotY_FromFighterDir),
        0x498 => ret = h.efLib_Create_AttachChild(0x138E, gobj, vl.jobj()).cast(),
        0x499 => ret = h.efLib_Create_AttachChild(0x138F, gobj, vl.jobj()).cast(),
        0x49A => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDir(0x138B, *vl).cast(),
        0x49B => efac!(5, 0x138F),
        0x49C => efac!(5, 0x1395),
        0x49D => {
            let e = h.efLib_Create_Attach_Pos(0x1390, gobj, vl.vec3());
            ret = e.cast();
            if !e.is_null() {
                let gj = GET_JOBJ(gobj);
                let j = GET_JOBJ((*e).gobj);
                HSD_JObjSetScaleX(h, j, HSD_JObjGetScaleY(gj));
                HSD_JObjSetScaleY(h, j, HSD_JObjGetScaleY(gj));
                HSD_JObjSetScaleZ(h, j, HSD_JObjGetScaleY(gj));
                HSD_JObjSetTranslateZ(h, HSD_JObjGetChild(j), offset_z(HSD_JObjGetTranslationZ(j)));
                // The C reads a second f32* for the Z rotation (two va_arg calls).
                if *vl.f32p() < 0.0 {
                    HSD_JObjSetRotationY(h, j, -M_PI_2 as f32);
                    let z = -*vl.f32p();
                    HSD_JObjSetRotationZ(h, j, z);
                } else {
                    HSD_JObjSetRotationY(h, j, M_PI_2 as f32);
                    let z = *vl.f32p();
                    HSD_JObjSetRotationZ(h, j, z);
                }
                (*e).update = Some(ext::efLib_Cb_SetJObjOffsetZ);
            }
        }
        0x49E => gen!(0x206),
        0x49F => attach_facing_then_efac!(0x7D00, 0x20, 0x7D00),
        0x4A0 => gen!(0x7D02),
        0x4A5 => ret = h.efLib_CreateGenerator_Attach_Scale(0xA028, *vl, gobj).cast(),
        0x4A6 => ret = h.efLib_CreateGenerator_Attach_Scale(0xA029, *vl, gobj).cast(),
        0x4A7 => ret = h.efLib_CreateGenerator_Attach_Scale(0xA02A, *vl, gobj).cast(),
        0x4A8 => ret = h.efLib_CreateGenerator_Attach_Scale(0xA02B, *vl, gobj).cast(),
        0x4A1 => efac!(0x22, 0x84D4),
        0x4A2 => efac!(0x22, 0x84D2),
        0x4A3 => efac!(0x22, 0x84D3),
        0x4A4 => { let e = h.efLib_Create_Attach_Pos(0x84D0, gobj, vl.vec3()); facing!(e); }
        0x4A9 | 0x4AA => {
            let e = h.efLib_Create_Attach_Scale(if gfx_id == 0x4A9 { 0x9858 } else { 0x9859 }, gobj, vl.jobj());
            let e = facing!(e);
            if !e.is_null() {
                h.HSD_JObjAnimAll(GET_JOBJ((*e).gobj));
            }
        }
        0x4AB => pair_facing_dir!(0x9471, 0x9470),
        0x4AC => pos_rot_y_pi2_z!(0x80E8),
        0x4AD => efac!(0x24, 0x8CA0),
        0x4AE => efac!(0x2E, 0xB3B0),
        0x4AF => efac!(0x2E, 0xB3B1),
        0x4B0 => ret = h.efLib_CreateGenerator_AppSRT_SetFacingDir(0xB3B6, *vl).cast(),
        0x4B1 => attach_facing_then_efac!(0x9088, 0x25, 0x9088),
        0x4B2 => gen!(0x908A),
        0x4B3 => pair_facing_dir!(0xB799, 0xB798),
        0x4B4 => ret = h.efLib_Create_Attach_Scale_FacingDir(0x4E20, gobj, vl.jobj()).cast(),
        0x4B5 => ret = h.efLib_Create_Attach_Scale_FacingDir(0x4E21, gobj, vl.jobj()).cast(),
        0x4B6 => child_copy_scale!(0x5208),
        0x4B7 => child_copy_scale!(0x5209),
        0x4B8 => ret = h.efLib_Create_Attach_Scale_FacingDir(0xBB80, gobj, vl.jobj()).cast(),
        0x4B9 => ret = h.efLib_Create_Attach_Scale_FacingDir(0xBB81, gobj, vl.jobj()).cast(),
        _ => {}
    }
    drain_anim_queue(h);
    ret
}
