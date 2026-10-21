//! Hand port, round 9: leaf functions of ft/ftanim.c (x8B0 part-anim slots, part joint pick,
//! the inverse-scale helper). Checked against the C (cut verbatim) by tests/ftleaf_ref.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `scale_inline` (static): `v *= 1 / scale` per component (no fused sites).
pub fn scale_inline(scale: f32, v: &mut Vec3) {
    let f1 = 1.0f32 / scale;
    v.x *= f1;
    v.y *= f1;
    v.z *= f1;
}

/// `get_part_joint` (static).
/// # Safety
/// `f.parts` must point to at least `part + 1` bones.
pub unsafe fn get_part_joint(f: &Fighter, part: i32, do_blending: bool) -> *mut HSD_JObj {
    let b = &*f.parts.offset(part as isize);
    if do_blending { b.x4_jobj2 } else { b.joint }
}

/// `ftAnim_8007077C`: every x8B0 slot's x11 and x10 to -1.
pub fn ftanim_8007077c(g: G) {
    for s in fp(g).x8B0.iter_mut() {
        s.x11 = -1;
        s.x10 = -1;
    }
}

/// `ftAnim_80070FB4`: `x8B0[arg1].x10 = arg2` (s8 truncation).
pub fn ftanim_80070fb4(g: G, arg1: i32, arg2: i32) {
    fp(g).x8B0[arg1 as usize].x10 = arg2 as i8;
}
