//! src/melee/cm/camera.c, functions not covered by the older `crate::cm::camera` module
//! (which already ports ~75 of the TU's ~190 functions; those are re-exported here).
//!
//! This file adds the CmSubject pool (Camera_80028F5C, Camera_80029020, Camera_80029044,
//! Camera_800290D4). The pool is the `cm_804D6458` free list; live subjects are
//! `CameraWorld::subjects` in the same order the older module uses (index 0 = `cm_804D6468`,
//! the most recently allocated, then `->prev`). A subject is addressed by its index there.
//!
//! TODO: the rest of camera.c (Camera_Init, the mode update functions 8002A4AC..8002F3AC, pause
//! camera, the render pass, the TARGET_PC script camera, projection helpers) is not ported yet.
#![allow(non_snake_case)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types::CmQuakeKind;
#[allow(unused_imports)]
use crate::cm::camera::{CmScriptPose, world};
pub use crate::cm::camera::*;
use melee_types::{CmSubject, CmSubjectState, Vec as Vec3};

/// `cm_804D6458` free list: how many subjects are still in the `HSD_MemAlloc` pool.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CmSubjectPool {
    pub free: usize,
}

/// `Camera_80028F5C`: reset a subject to `state` with the default extents.
pub fn Camera_80028F5C(subject: Option<&mut CmSubject>, state: CmSubjectState) {
    melee_core::ct!("Camera_80028F5C");
    if let Some(s) = subject {
        s.state = state;
        s.pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        s.bone_pos = s.pos;
        s.facing_dir = 0.0;
        s.set_on_ledge(0);
        s.set_force_inactive(0);
        s.set_was_framed(0);
        s.state_timer = 0;
        s.ext.h.x = -1.0;
        s.ext.h.y = 1.0;
        s.ext.v.x = 1.0;
        s.ext.v.y = -1.0;
        s.ext.v.z = 1.0;
        s.target_ext.h = s.ext.h;
        s.target_ext.v = s.ext.v;
        s.x54 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        s.x60 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    }
}

/// `Camera_80029020`: `Camera_80029044(0)`.
pub fn Camera_80029020(w: &mut CameraWorld, pool: &mut CmSubjectPool) -> usize {
    melee_core::ct!("Camera_80029020");
    Camera_80029044(w, pool, 0)
}

/// `Camera_80029044`: take a subject from the pool, append it as the new list tail
/// (`cm_804D6468`, index 0) and reset it to `state`. The C spins forever after
/// `OSReport("couldn't get CmSubject struct.\n")` when the pool is empty; this panics.
pub fn Camera_80029044(w: &mut CameraWorld, pool: &mut CmSubjectPool, state: CmSubjectState) -> usize {
    melee_core::ct!("Camera_80029044");
    if pool.free == 0 {
        panic!("couldn't get CmSubject struct.");
    }
    pool.free -= 1;
    // The reset writes every field of the record (next/prev are list links, kept by the Vec).
    let mut s = CmSubject::zeroed();
    Camera_80028F5C(Some(&mut s), state);
    w.subjects.insert(0, s);
    if w.geno_fly_subject.len() + 1 == w.subjects.len() {
        w.geno_fly_subject.insert(0, false);
    }
    0
}

/// `Camera_800290D4`: unlink subject `i` and return it to the pool.
pub fn Camera_800290D4(w: &mut CameraWorld, pool: &mut CmSubjectPool, i: usize) -> CmSubject {
    melee_core::ct!("Camera_800290D4");
    let s = w.subjects.remove(i);
    if i < w.geno_fly_subject.len() {
        w.geno_fly_subject.remove(i);
    }
    pool.free += 1;
    s
}

// ---- folded from camera.rs (consolidation steps 21/22) ----
/// `Camera_80030A50_fp`: the camera GObj, after an auto-view refresh of its CObj (TARGET_PC).
///
/// # Safety
/// `game_camera.gobj` is NULL or a live camera GObj.
pub unsafe fn Camera_80030A50_fp() -> *mut melee_types::HSD_GObj {
    melee_core::ct!("Camera_80030A50_fp");
    let gobj = world().game_camera.gobj;
    if !gobj.is_null() {
        melee_runtime::tu::cobj::HSD_CObjUpdateView((*gobj).hsd_obj.cast());
    }
    gobj
}

/// `Camera_RequestQuake_fp(kind, pos)` on the process camera world, with the real `grLib_801C9CEC`.
///
/// # Safety
/// `pos` is NULL or points to a `Vec3`.
pub unsafe fn Camera_RequestQuake_fp(kind: CmQuakeKind, pos: *mut Vec3) {
    melee_core::ct!("Camera_RequestQuake_fp");
    world().camera_request_quake_with(kind, if pos.is_null() { None } else { Some(*pos) }, |k| unsafe {
        crate::tu::grlib::grLib_801C9CEC(k)
    });
}

/// `cm_script_float_fp`: the int bit pattern as an f32.
pub fn cm_script_float_fp(bits: i32) -> f32 {
    melee_core::ct!("cm_script_float_fp");
    f32::from_bits(bits as u32)
}

/// `cm_script_bits_fp`: the f32 bit pattern as an int.
pub fn cm_script_bits_fp(value: f32) -> i32 {
    melee_core::ct!("cm_script_bits_fp");
    value.to_bits() as i32
}

/// `cm_script_lerp_fp`: `a + (b - a) * t`, one fused site.
pub fn cm_script_lerp_fp(a: f32, b: f32, t: f32) -> f32 {
    melee_core::ct!("cm_script_lerp_fp");
    fmadds(b - a, t, a)
}

/// `cm_script_blend_fp`: field-wise lerp.
pub fn cm_script_blend_fp(a: &CmScriptPose, b: &CmScriptPose, t: f32) -> CmScriptPose {
    melee_core::ct!("cm_script_blend_fp");
    let l = |x: f32, y: f32| cm_script_lerp_fp(x, y, t);
    CmScriptPose {
        eye: Vec3 { x: l(a.eye.x, b.eye.x), y: l(a.eye.y, b.eye.y), z: l(a.eye.z, b.eye.z) },
        interest: Vec3 {
            x: l(a.interest.x, b.interest.x),
            y: l(a.interest.y, b.interest.y),
            z: l(a.interest.z, b.interest.z),
        },
        fov: l(a.fov, b.fov),
        roll: l(a.roll, b.roll),
    }
}

/// `cm_script_ease_fp`: 1 ease-in, 2 ease-out, 3 ease-in-out (its second branch fuses), else
/// linear.
pub fn cm_script_ease_fp(t: f32, ease: i32) -> f32 {
    melee_core::ct!("cm_script_ease_fp");
    match ease {
        1 => t * t,
        2 => t * (2.0 - t),
        3 => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                fmadds(-(2.0 * (1.0 - t)), 1.0 - t, 1.0)
            }
        }
        _ => t,
    }
}
