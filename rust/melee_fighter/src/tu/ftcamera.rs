//! src/melee/ft/ftcamera.c: the fighter's camera subject box (CmSubject) updates.
//! No fused sites (`tools/rust-port/fma_sites.py`). Stage getters (group 2, melee_world, which
//! this crate cannot depend on) are `ext` hooks that panic while unset.
#![allow(non_snake_case, static_mut_refs)]

use crate::types::{CmSubject, CmSubjectState_Active, Fighter, HSD_GObj, UnkFloat6_Camera, Vec3};

pub mod ext {
    use crate::types::Vec3;
    /// TODO(group2 gr/stage.c) `Stage_GetCamFixedZoom`.
    pub static mut Stage_GetCamFixedZoom: Option<fn() -> f32> = None;
    /// TODO(group2 gr/stage.c) `Stage_UnkSetVec3TCam_Offset`.
    pub static mut Stage_UnkSetVec3TCam_Offset: Option<fn(&mut Vec3)> = None;
    /// TODO(group2 gr/stage.c) `Stage_GetBlastZoneTopOffset`.
    pub static mut Stage_GetBlastZoneTopOffset: Option<fn() -> f32> = None;
    /// TODO(group2 gr/stage.c) `Stage_GetCamBoundsTopOffset`.
    pub static mut Stage_GetCamBoundsTopOffset: Option<fn() -> f32> = None;

    pub fn cam_fixed_zoom() -> f32 {
        (unsafe { Stage_GetCamFixedZoom }.expect("TODO(group2) Stage_GetCamFixedZoom"))()
    }
    pub fn tcam_offset(v: &mut Vec3) {
        (unsafe { Stage_UnkSetVec3TCam_Offset }.expect("TODO(group2) Stage_UnkSetVec3TCam_Offset"))(v)
    }
    pub fn blast_top() -> f32 {
        (unsafe { Stage_GetBlastZoneTopOffset }.expect("TODO(group2) Stage_GetBlastZoneTopOffset"))()
    }
    pub fn cam_bounds_top() -> f32 {
        (unsafe { Stage_GetCamBoundsTopOffset }.expect("TODO(group2) Stage_GetCamBoundsTopOffset"))()
    }
}

#[inline(always)]
fn fp<'a>(g: *mut HSD_GObj) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftCamera_80076018`: scale both vectors by `mul`.
pub fn ftcamera_80076018(inp: &UnkFloat6_Camera, out: &mut UnkFloat6_Camera, mul: f32) {
    melee_core::ct!("ftcamera_80076018");
    out.x0.x = inp.x0.x * mul;
    out.x0.y = inp.x0.y * mul;
    out.x0.z = inp.x0.z * mul;
    out.xC.x = inp.xC.x * mul;
    out.xC.y = inp.xC.y * mul;
    out.xC.z = inp.xC.z * mul;
}

/// Shared horizontal-extent block of 80076064 / UpdateCameraBox.
fn set_h(cb: &mut CmSubject, s: &UnkFloat6_Camera, facing: f32) {
    if facing == 1.0 {
        cb.target_ext.h.x = s.x0.z;
        cb.target_ext.h.y = s.x0.y * ext::cam_fixed_zoom();
        cb.facing_dir = 1.0;
    } else {
        cb.target_ext.h.x = -s.x0.y * ext::cam_fixed_zoom();
        cb.target_ext.h.y = -s.x0.z;
        cb.facing_dir = -1.0;
    }
}

/// `ftCamera_80076064`: initialise the camera box from the fighter's position (snap).
pub fn ftcamera_80076064(fp: &mut Fighter) {
    melee_core::ct!("ftcamera_80076064");
    let cb = unsafe { &mut *fp.x890_cameraBox };
    let mut sp: UnkFloat6_Camera = unsafe { core::mem::zeroed() };
    ftcamera_80076018(unsafe { &*(*fp.ft_data).x3C }, &mut sp, fp.x34_scale.y);
    cb.state = CmSubjectState_Active;
    set_h(cb, &sp, fp.facing_dir);
    cb.target_ext.v.x = sp.xC.x;
    cb.target_ext.v.y = sp.xC.y;
    cb.target_ext.v.z = sp.xC.z;
    cb.ext.h = cb.target_ext.h;
    cb.ext.v = cb.target_ext.v;
    cb.pos.x = fp.cur_pos.x;
    cb.pos.y = fp.cur_pos.y + sp.x0.x;
    cb.pos.z = fp.cur_pos.z;
    cb.bone_pos = cb.pos;
}

/// `ftCamera_UpdateCameraBox`.
pub fn ftcamera_update_camera_box(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftcamera_update_camera_box");
    let f = fp(gobj);
    let cb = unsafe { &mut *f.x890_cameraBox };
    let mut s: UnkFloat6_Camera = unsafe { core::mem::zeroed() };
    ftcamera_80076018(unsafe { &*(*f.ft_data).x3C }, &mut s, f.x34_scale.y);
    set_h(cb, &s, f.facing_dir);
    cb.pos.x = f.cur_pos.x;
    cb.pos.y = f.cur_pos.y + s.x0.x;
    cb.pos.z = f.cur_pos.z;
    cb.set_on_ledge(0);
    crate::ftlib::ftlib_800866dc(gobj, &mut cb.bone_pos);
}

/// `ftCamera_800762F4`: refresh only the bone position.
pub fn ftcamera_800762f4(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftcamera_800762f4");
    let f = fp(gobj);
    crate::ftlib::ftlib_800866dc(gobj, unsafe { &mut (*f.x890_cameraBox).bone_pos });
}

/// `ftCamera_80076320`: camera box pinned to the blast-zone top (Star KO / Screen KO).
pub fn ftcamera_80076320(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftcamera_80076320");
    let cb = unsafe { &mut *fp(gobj).x890_cameraBox };
    ftcamera_update_camera_box(gobj);
    let mut center = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::tcam_offset(&mut center);
    assert!(ext::blast_top() - center.y != 0.0, "stGetPlyDeadUp() - center_pos.y != 0.0F");
    let t31 = ext::blast_top() - center.y;
    let t1 = ext::cam_bounds_top() - center.y;
    cb.pos.x = (cb.pos.x * t1) / t31;
    cb.pos.y = ext::blast_top();
}

// ---- C names (the bodies above are the oracle-checked ports; tests/tu_ftcamera.rs calls these)

/// `ftCamera_80076018`.
pub fn ftCamera_80076018(inp: &UnkFloat6_Camera, out: &mut UnkFloat6_Camera, mul: f32) { melee_core::ct!("ftCamera_80076018"); ftcamera_80076018(inp, out, mul) }
/// `ftCamera_80076064`.
pub fn ftCamera_80076064(fp: &mut Fighter) { melee_core::ct!("ftCamera_80076064"); ftcamera_80076064(fp) }
/// `ftCamera_UpdateCameraBox` (Fighter_UpdateCameraBox).
pub fn ftCamera_UpdateCameraBox(gobj: *mut HSD_GObj) { melee_core::ct!("ftCamera_UpdateCameraBox"); ftcamera_update_camera_box(gobj) }
/// `ftCamera_800762F4`.
pub fn ftCamera_800762F4(gobj: *mut HSD_GObj) { melee_core::ct!("ftCamera_800762F4"); ftcamera_800762f4(gobj) }
/// `ftCamera_80076320`.
pub fn ftCamera_80076320(gobj: *mut HSD_GObj) { melee_core::ct!("ftCamera_80076320"); ftcamera_80076320(gobj) }
