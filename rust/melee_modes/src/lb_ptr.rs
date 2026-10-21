//! Round 3: `src/melee/lb/` leaf functions that take pointers to game structs, from the lb TUs
//! group 1 did not port. TODO(group1): move into melee_runtime when group 1 ports these TUs.
//! Conventions and verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
//!
//! FMA: `tools/rust-port/fma_sites.py` lists the shipping build's fused sites; every
//! `fmadds` below is one of them and every other `a*b+c` is left unfused.
#![allow(non_snake_case)]

use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types as mt;
use mt::{CommandInfo, HSD_DObj, HSD_JObj, HitCapsule, Vec as Vec3};

/// `FigaTree` (lb/lbanim.h). melee_types only has it opaque. TODO(group0): generate it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FigaTree {
    pub r#type: i32,
    pub flags: u32,
    pub frames: f32,
    pub nodes: *mut i8,
    pub tracks: *mut c_void,
}
melee_core::gc_layout!(FigaTree, "FigaTree", size 0x14, align 4, [
    r#type @ 0x0: 4 "int",
    flags @ 0x4: 4 "u32",
    frames @ 0x8: 4 "f32",
    nodes @ 0xC: 4 "s8*",
    tracks @ 0x10: 4 "FigaTrack*",
]);

/// lbanim.c `lbAnim_8001E8F8`.
pub fn lbAnim_8001E8F8(tree: Option<&FigaTree>) -> f32 {
    tree.map_or(0.0, |t| t.frames)
}

/// lbcardnew.c `readCardFileSize` (a `volatile` read).
pub fn readCardFileSize(file_size: &i32) -> i32 {
    // SAFETY: a reference is valid for reads.
    unsafe { core::ptr::read_volatile(file_size) }
}

/// lbcollision.c `lbColl_80005EBC`: squared distance from `arg2` to segment arg0..arg1 (3D);
/// `*arg3` gets the clamped segment parameter. Nine fused sites.
pub fn lbColl_80005EBC(arg0: &Vec3, arg1: &Vec3, arg2: &Vec3, arg3: &mut f32) -> f32 {
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

/// lbcollision.c `lbColl_80005FC0`: the same in the XY plane. Five fused sites.
pub fn lbColl_80005FC0(arg0: &Vec3, arg1: &Vec3, arg2: &Vec3, arg3: &mut f32) -> f32 {
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

/// lbcollision.c `end`: `a` within `unk_sum` of `b` (`z*z + (x*x + y*y)`: both fused).
pub fn end(a: &Vec3, b: &Vec3, unk_sum: f32) -> bool {
    let x = a.x - b.x;
    let y = a.y - b.y;
    let z = a.z - b.z;
    !(unk_sum * unk_sum < fmadds(z, z, fmadds(x, x, y * y)))
}

/// lbcollision.c `lbColl_GetY`.
pub fn lbColl_GetY(v: &Vec3) -> f32 {
    v.y
}

/// lbcollision.c `lbColl_DifferenceY`.
pub fn lbColl_DifferenceY(a: &Vec3, b: &Vec3) -> f32 {
    a.y - b.y
}

/// lbcollision.c `sqrDistance` (two fused sites).
pub fn sqrDistance(a: &Vec3, b: &Vec3) -> f32 {
    let x = a.x - b.x;
    let y = a.y - b.y;
    let z = a.z - b.z;
    fmadds(z, z, fmadds(x, x, y * y))
}

/// lbcollision.c `lbColl_80008428`.
pub fn lbColl_80008428(arg0: &mut HitCapsule) {
    arg0.state = mt::HitCapsule_Disabled;
}

/// lbcollision.c `lbColl_80008434`.
pub fn lbColl_80008434(arg0: &mut HitCapsule) {
    arg0.state = mt::HitCapsule_Enabled;
}

/// lbcollision.c `lbColl_80008440`: forget every victim.
pub fn lbColl_80008440(hit: &mut HitCapsule) {
    for v in &mut hit.victims_1 {
        v.victim = core::ptr::null_mut();
    }
    hit.x44 = 0;
    for v in &mut hit.victims_2 {
        v.victim = core::ptr::null_mut();
    }
    hit.x45 = 0;
}

/// lbcollision.c `lbColl_800089B8`: forget one victim.
pub fn lbColl_800089B8(hit: &mut HitCapsule, arg1: *mut c_void) {
    for v in &mut hit.victims_1 {
        if v.victim == arg1 {
            v.victim = core::ptr::null_mut();
        }
    }
}

/// lbcollision.c `lbColl_80008A5C`: count down the victims' rehit timers.
pub fn lbColl_80008A5C(hit: &mut HitCapsule) {
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

/// lbcollision.c `lbColl_80008D30`: copy a hitbox description (damage u32 -> f32).
pub fn lbColl_80008D30(arg0: &mut HitCapsule, arg1: &mt::lbColl_80008D30_arg1) {
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

/// lbcommand.c `Command_00`: end of script.
pub fn Command_00(info: &mut CommandInfo) {
    info.anon_8.u = core::ptr::null_mut();
}

/// lbcommand.c `Command_06`: return from a subroutine.
pub fn Command_06(info: &mut CommandInfo) {
    info.loop_count = info.loop_count.wrapping_sub(1);
    info.anon_8.u = info.event_return[info.loop_count as usize];
}

/// lbcommand.c `Command_08`: next command, then wait forever (F32_MAX).
pub fn Command_08(info: &mut CommandInfo) {
    // SAFETY: reading the union's pointer member; the C reads the same bytes.
    let u = unsafe { info.anon_8.u };
    info.anon_8.u = u.wrapping_add(1);
    info.timer = 3.4028235e38f32;
}

/// lbshadow.c `splGetCardinalTangent` (14 fused sites).
pub fn splGetCardinalTangent(p: &mut Vec3, cp: &[Vec3], tension: f32, u: f32) {
    let u2 = u * u;
    let car0 = tension * (fmadds(-3.0, u2, 4.0 * u) - 1.0);
    let car1 = fmadds(3.0 * (2.0 - tension), u2, 2.0 * (tension - 3.0) * u);
    let car2 = tension + fmadds(3.0 * (tension - 2.0), u2, 2.0 * -fmadds(2.0, tension, -3.0) * u);
    let car3 = tension * fmadds(3.0, u2, -(2.0 * u));
    let c = [car0, car1, car2, car3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBSplineTangent` (11 fused sites).
pub fn splGetBSplineTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    let u2 = u * u;
    let u_1 = 1.0 - u;
    let half = 0.5f32;
    let b0 = u_1 * (-half * u_1);
    let b1 = half * fmadds(3.0, u2, -(4.0 * u));
    let b2 = half * (1.0 + fmadds(-3.0, u2, 2.0 * u));
    let b3 = half * u2;
    let c = [b0, b1, b2, b3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbshadow.c `splGetBezierTangent` (12 fused sites).
pub fn splGetBezierTangent(p: &mut Vec3, cp: &[Vec3], u: f32) {
    let u_1 = u - 1.0;
    let u2 = u * u;
    let bez0 = -3.0 * u_1 * u_1;
    let bez1 = 3.0 * fmadds(3.0, u2, fmadds(-4.0, u, 1.0));
    let bez2 = 3.0 * fmadds(2.0, u, -(3.0 * u2));
    let bez3 = 3.0 * u2;
    let c = [bez0, bez1, bez2, bez3];
    let comb = |f: fn(&Vec3) -> f32| {
        fmadds(f(&cp[3]), c[3], fmadds(f(&cp[2]), c[2], fmadds(f(&cp[0]), c[0], f(&cp[1]) * c[1])))
    };
    p.x = comb(|v| v.x);
    p.y = comb(|v| v.y);
    p.z = comb(|v| v.z);
}

/// lbsnap.c `lbSnap_ClearText`: 0x21 NULs.
pub fn lbSnap_ClearText(text: &mut [i8]) {
    text[..0x21].fill(0);
}

/// lbspdisplay.c and lb_00F9.c `checkJObjFlags` (identical static copies): neither a
/// particle nor a spline joint.
pub fn checkJObjFlags(jobj: &HSD_JObj) -> bool {
    jobj.flags & (mt::JOBJ_PTCL | mt::JOBJ_SPLINE) as u32 == 0
}

/// lbvector.c `lbVector_Mirror`: reflect `a` in the XY plane about a unit axis (3 fused sites).
pub fn lbVector_Mirror(a: &mut Vec3, unit_mirror_axis: &Vec3) {
    let m = unit_mirror_axis;
    let f = fmadds(m.x, a.x, m.y * a.y) * -2.0;
    a.x = fmadds(m.x, f, a.x);
    a.y = fmadds(m.y, f, a.y);
}

/// lb_00B0.c `lb_8000CE30`.
pub fn lb_8000CE30(dobj: Option<&mut HSD_DObj>, next: *mut HSD_DObj) {
    if let Some(d) = dobj {
        d.next = next;
    }
}

/// lb_00B0.c `lb_8000CE40`.
pub fn lb_8000CE40(jobj: Option<&mut HSD_JObj>, dobj: *mut HSD_DObj) {
    if let Some(j) = jobj {
        j.u.dobj = dobj;
    }
}

/// lb_00F9.c `lb_800103D8`: where the segment (x0,x1)-(x2,x3) crosses height `offset`
/// (one fused site).
pub fn lb_800103D8(vec: &mut Vec3, x0: f32, x1: f32, x2: f32, x3: f32, offset: f32) -> bool {
    let dist0 = x1 - offset;
    let dist1 = x3 - offset;
    if dist0 == dist1 {
        return dist0 > 1e-10f32;
    }
    if dist0 as f64 > 0.0 && (dist1 as f64) < 0.0 {
        vec.x = fmadds(-dist1 / (dist0 - dist1), x0 - x2, x2);
        vec.y = offset;
        vec.z = 0.0;
        return true;
    }
    false
}

/// lb_013B.c `lb_80013BB0`.
pub fn lb_80013BB0(_arg: *mut mt::ColorOverlay) -> bool {
    true
}

/// lb_0146.c `lb_80014638` (two fused sites).
pub fn lb_80014638(arg0: &mt::lb_80014638_arg0_t, arg1: &mt::lb_80014638_arg1_t) -> bool {
    let mut sp30 = arg0.x0;
    let sp24 = arg0.xC;
    let sp18 = arg1.x8;
    sp30.x = sp30.x + arg1.unk_x;
    sp30.y += arg1.unk_x;
    sp30.z += arg1.unk_y;
    if sp30.z < sp24.z {
        return false;
    }
    if sp30.z < sp18.z || sp24.z > sp18.z {
        return false;
    }
    let d = sp24.z - sp30.z;
    let z = if (if d < 0.0 { -d } else { d }) < 0.01f32 { 1.0 } else { (sp18.z - sp30.z) / d };
    if fmadds(z, sp24.x - sp30.x, sp30.x) > sp18.y {
        return false;
    }
    if fmadds(z, sp24.y - sp30.y, sp30.y) < sp18.x {
        return false;
    }
    true
}
