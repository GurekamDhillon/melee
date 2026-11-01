//! lbcollision.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types::HitCapsule as HitCapsule_lb_r8;
#[allow(unused_imports)]
use core::ffi::c_void;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::HitCapsule;
#[allow(unused_imports)]
use mt::Vec as Vec3;
// ---- folded from lb.rs (consolidation steps 21/22) ----
/// lbcollision.c `isSmall_fp`.
pub fn isSmall_fp(x: f32) -> bool {
    melee_core::ct!("isSmall_fp");
    x < 0.00001 && x > -0.00001
}

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lbcollision.c `lbColl_80005EBC_fp`: squared distance from `arg2` to segment arg0..arg1 (3D);
/// `*arg3` gets the clamped segment parameter. Nine fused sites.
pub fn lbColl_80005EBC_fp(arg0: &Vec3, arg1: &Vec3, arg2: &Vec3, arg3: &mut f32) -> f32 {
    melee_core::ct!("lbColl_80005EBC_fp");
    let sp50 = *arg0;
    let d1 = Vec3 { x: arg1.x - arg0.x, y: arg1.y - arg0.y, z: arg1.z - arg0.z };
    let sp38 = *arg2;
    let d2 = Vec3 { x: sp50.x - sp38.x, y: sp50.y - sp38.y, z: sp50.z - sp38.z };
    let d1_dot_d1 = fmadds(d1.z, d1.z, fmadds(d1.x, d1.x, d1.y * d1.y));
    let d1_dot_d2 = fmadds(d1.z, d2.z, fmadds(d1.x, d2.x, d1.y * d2.y));
    let mut scale = -d1_dot_d2 / d1_dot_d1;
    if scale as f64 > 1.0 {
        scale = 1.0;
    } else if (scale as f64) < 0.0 {
        scale = 0.0;
    }
    let x = fmadds(d1.x, scale, sp50.x) - arg2.x;
    let y = fmadds(d1.y, scale, sp50.y) - arg2.y;
    let z = fmadds(d1.z, scale, sp50.z) - arg2.z;
    *arg3 = scale;
    fmadds(z, z, fmadds(x, x, y * y))
}

/// lbcollision.c `lbColl_80005FC0_fp`: the same in the XY plane. Five fused sites.
pub fn lbColl_80005FC0_fp(arg0: &Vec3, arg1: &Vec3, arg2: &Vec3, arg3: &mut f32) -> f32 {
    melee_core::ct!("lbColl_80005FC0_fp");
    let sp50 = *arg0;
    let d1x = arg1.x - arg0.x;
    let d1y = arg1.y - arg0.y;
    let sp38 = *arg2;
    let d2x = sp50.x - sp38.x;
    let d2y = sp50.y - sp38.y;
    let d1_dot_d1 = fmadds(d1x, d1x, d1y * d1y);
    let d1_dot_d2 = fmadds(d1x, d2x, d1y * d2y);
    let mut scale = -d1_dot_d2 / d1_dot_d1;
    if scale as f64 > 1.0 {
        scale = 1.0;
    } else if (scale as f64) < 0.0 {
        scale = 0.0;
    }
    let x = fmadds(d1x, scale, sp50.x) - arg2.x;
    let y = fmadds(d1y, scale, sp50.y) - arg2.y;
    *arg3 = scale;
    fmadds(x, x, y * y)
}

/// lbcollision.c `end_fp`: `a` within `unk_sum` of `b` (`z*z + (x*x + y*y)`: both fused).
pub fn end_fp(a: &Vec3, b: &Vec3, unk_sum: f32) -> bool {
    melee_core::ct!("end_fp");
    let x = a.x - b.x;
    let y = a.y - b.y;
    let z = a.z - b.z;
    !(unk_sum * unk_sum < fmadds(z, z, fmadds(x, x, y * y)))
}

/// lbcollision.c `lbColl_GetY`.
pub fn lbColl_GetY(v: &Vec3) -> f32 {
    melee_core::ct!("lbColl_GetY");
    v.y
}

/// lbcollision.c `lbColl_DifferenceY`.
pub fn lbColl_DifferenceY(a: &Vec3, b: &Vec3) -> f32 {
    melee_core::ct!("lbColl_DifferenceY");
    a.y - b.y
}

/// lbcollision.c `sqrDistance` (two fused sites).
pub fn sqrDistance(a: &Vec3, b: &Vec3) -> f32 {
    melee_core::ct!("sqrDistance");
    let x = a.x - b.x;
    let y = a.y - b.y;
    let z = a.z - b.z;
    fmadds(z, z, fmadds(x, x, y * y))
}

/// lbcollision.c `lbColl_800089B8_fp`: forget one victim.
pub fn lbColl_800089B8_fp(hit: &mut HitCapsule, arg1: *mut c_void) {
    melee_core::ct!("lbColl_800089B8_fp");
    for v in &mut hit.victims_1 {
        if v.victim == arg1 {
            v.victim = core::ptr::null_mut();
        }
    }
}

/// lbcollision.c `lbColl_80008A5C_fp`: count down the victims' rehit timers.
pub fn lbColl_80008A5C_fp(hit: &mut HitCapsule) {
    melee_core::ct!("lbColl_80008A5C_fp");
    if hit.state == mt::HitCapsule_Disabled {
        return;
    }
    for v in hit.victims_1.iter_mut().chain(hit.victims_2.iter_mut()) {
        if !v.victim.is_null() && v.x4 != 0 {
            v.x4 -= 1;
            if v.x4 == 0 {
                v.victim = core::ptr::null_mut();
            }
        }
    }
}

/// lbcollision.c `lbColl_80008D30_fp`: copy a hitbox description (damage u32 -> f32).
pub fn lbColl_80008D30_fp(arg0: &mut HitCapsule, arg1: &mt::lbColl_80008D30_arg1) {
    melee_core::ct!("lbColl_80008D30_fp");
    arg0.state = arg1.state;
    arg0.damage = arg1.damage as f32;
    arg0.kb_angle = arg1.kb_angle;
    arg0.x24 = arg1.unkC;
    arg0.x28 = arg1.unk10;
    arg0.x2C = arg1.unk14;
    arg0.element = arg1.element;
    arg0.sfx_severity = arg1.sfx_severity;
    arg0.sfx_kind = arg1.sfx_kind;
    arg0.unk_count = arg1.damage;
}

// ---- folded from lb_r8.rs (consolidation steps 21/22) ----
/// lbcollision.c `lbColl_CopyHitCapsule_fp`: copy both victim arrays and x44/x45.
/// The C struct assignment copies each HitVictim's tail padding too (native: 4 bytes after x4),
/// so the copy is bytewise to keep the record bit-identical.
pub fn lbColl_CopyHitCapsule_fp(src: &HitCapsule_lb_r8, dst: &mut HitCapsule_lb_r8) {
    melee_core::ct!("lbColl_CopyHitCapsule_fp");
    for i in 0..src.victims_1.len() {
        // SAFETY: distinct `&`/`&mut` records, so the elements do not overlap.
        unsafe {
            core::ptr::copy_nonoverlapping(&src.victims_1[i], &mut dst.victims_1[i], 1);
            core::ptr::copy_nonoverlapping(&src.victims_2[i], &mut dst.victims_2[i], 1);
        }
    }
    dst.x44 = src.x44;
    dst.x45 = src.x45;
}

/// lbcollision.c `lbColl_8000ACFC_fp`: `victim` is in `hitbox.victims_1` (compared by address).
pub fn lbColl_8000ACFC_fp(victim: *mut c_void, hitbox: &HitCapsule_lb_r8) -> bool {
    melee_core::ct!("lbColl_8000ACFC_fp");
    hitbox.victims_1.iter().any(|v| v.victim == victim)
}
