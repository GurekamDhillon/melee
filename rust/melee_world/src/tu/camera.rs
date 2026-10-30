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
