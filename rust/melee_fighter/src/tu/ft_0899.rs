//! src/melee/ft/ft_0899.c: foot IK on the ground (`ft_80089B08`, leg chains from
//! `ftData::x58`) + floor-slope root tilt, and its flag setter `ft_8008A1B8`.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0899.c`): ft_80089B08 has 2 f32
//! (`dx*dx + dy*dy`, `facing*atan2f(..) + adj`) -> `fmadds`, and 3 f64 (the inline sqrt's
//! Newton steps) -> `melee_runtime::gekko::sqrtf`, which carries exactly those fmas.
//! Checked against the C (whole TU included verbatim, shipping contraction) by
//! tests/tu_ft_0899.rs.
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::gekko::sqrtf;
use melee_runtime::lbtrigf::atan2f;

pub type G = *mut HSD_GObj;

/// Callees outside this TU with no port yet: settable, panic while unset.
pub mod ext {
    use crate::types::*;
    macro_rules! ext_fns {
        ($($hook:ident $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported (ft_0899 ext)")),
                }
            }
        )*};
    }
    ext_fns! {
        MPLIB_8004DD90_FLOOR mpLib_8004DD90_Floor(line: i32, pos: &Vec3, y: &mut f32, flags: &mut u32, n: &mut Vec3) -> i32;
        MP_FLOOR_GET_LEFT mpFloorGetLeft(line: i32, out: &mut Vec3);
        MP_FLOOR_GET_RIGHT mpFloorGetRight(line: i32, out: &mut Vec3);
        MP_LINE_GET_V1_POS mpLineGetV1Pos(line: i32, out: &mut Vec3);
        MP_LINE_GET_V0_POS mpLineGetV0Pos(line: i32, out: &mut Vec3);
        MP_LINE_GET_KIND mpLineGetKind(line: i32) -> i32;
        MP_LINE_GET_NORMAL mpLineGetNormal(line: i32, out: &mut Vec3);
        LBBGFLASH_80021410 lbBgFlash_80021410(ik: &mut IKState);
        LBBGFLASH_80020E38 lbBgFlash_80020E38(j: *mut HSD_JObj, v: &Vec3, a: f32, b: f32, c: f32);
        FT_PART_SET_ROT_X ftPartSetRotX(fp: &mut Fighter, part: i32, x: f32);
        DB_804D4AF8 db_804D4AF8() -> i32;
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn rv(v: &Vec3) -> melee_core::vec::Vec3 {
    melee_core::vec::Vec3 { x: v.x, y: v.y, z: v.z }
}
#[inline(always)]
fn tv(v: &melee_core::vec::Vec3) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}
#[inline(always)]
fn abs_c(v: f32) -> f32 {
    if v < 0.0 { -v } else { v }
}

/// `fn_8008998C` (static): drop the IK target `ik.pos4` onto the floor under it, the step
/// clamped to slope 0.45. Returns whether it moved.
pub fn fn_8008998C(f: &mut Fighter, ik: &mut IKState, normal: &mut Vec3) -> bool {
    melee_core::ct!("fn_8008998C");
    let jobj_pos = unsafe { (*((*f.gobj).hsd_obj as *mut HSD_JObj)).translate };
    *normal = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let line = f.coll_data.floor.index;
    let (mut floor_y, mut flags) = (0.0f32, 0u32);
    let mut delta_y;
    if ext::mpLib_8004DD90_Floor(line, &ik.pos4, &mut floor_y, &mut flags, normal) != -1 {
        delta_y = (ik.pos4.y + floor_y) - jobj_pos.y;
    } else {
        let mut sp20 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::mpFloorGetLeft(line, &mut sp20);
        if ik.pos4.x > sp20.x {
            ext::mpFloorGetRight(line, &mut sp20);
        }
        delta_y = sp20.y - jobj_pos.y;
    }
    if abs_c(delta_y) < 0.0001 {
        return false;
    }
    let pos_x = ik.pos4.x;
    if pos_x != jobj_pos.x {
        let delta_x = pos_x - jobj_pos.x;
        let slope = delta_y / delta_x;
        if slope > 0.45 {
            delta_y = 0.45 * delta_x;
        } else if slope < -0.45 {
            delta_y = -0.45 * delta_x;
        }
    } else {
        delta_y = 0.0;
    }
    ik.pos4.y += delta_y;
    true
}

/// One leg: joints from x58 indices (`j0`,`j1`,`j2`), solve, restore the rotations.
fn leg(f: &mut Fighter, ik: &mut IKState, j: [u8; 3], dir: f32) {
    unsafe {
        ik.jobj0 = (*f.parts.add(j[0] as usize)).joint;
        ik.jobj1 = (*f.parts.add(j[1] as usize)).joint;
        ik.jobj2 = (*f.parts.add(j[2] as usize)).joint;
    }
    ik.angle_max = core::f32::consts::FRAC_PI_2; // M_PI_F / 2.0F
    ik.angle_limit = 1.2217305;
    h::lb_8000b1cc(ik.jobj0, &Vec3 { x: 0.0, y: 0.0, z: 0.0 }, &mut ik.pos0);
    h::lb_8000b1cc(ik.jobj1, &Vec3 { x: 0.0, y: 0.0, z: 0.0 }, &mut ik.pos1);
    h::lb_8000b1cc(ik.jobj2, &Vec3 { x: 0.0, y: 0.0, z: 0.0 }, &mut ik.pos2);
    let mut d = melee_runtime::lbvector::diff(&rv(&ik.pos2), &rv(&ik.pos1));
    let len = ik.len1;
    melee_runtime::lbvector::normalize(&mut d);
    d.x *= len;
    d.y *= len;
    d.z *= len;
    melee_runtime::lbvector::add(&mut d, &rv(&ik.pos1));
    ik.pos3 = tv(&d);
    ik.pos4 = ik.pos3;
    let (r0, r1, r2) = unsafe { ((*ik.jobj0).rotate, (*ik.jobj1).rotate, (*ik.jobj2).rotate) };
    let mut spa4 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if fn_8008998C(f, ik, &mut spa4) {
        ext::lbBgFlash_80021410(ik);
    }
    ext::lbBgFlash_80020E38(ik.jobj2, &spa4, 0.34906584, 0.34906584, dir);
    unsafe {
        (*ik.jobj0).rotate = r0;
        (*ik.jobj1).rotate = r1;
        (*ik.jobj2).rotate = r2;
    }
}

/// `ft_80089B08`: grounded foot IK (x221C bits 1/2, debug-gated by `db_804D4AF8`) and the
/// root's slope tilt (bit 4).
///
/// C quirk kept: the C passes `NULL` as lb_8000B1CC's offset; the hook takes a zero offset.
pub fn ft_80089B08(g: G) {
    melee_core::ct!("ft_80089B08");
    let f = fp(g);
    if f.x2219_b5() != 0 || f.ground_or_air != GA_Ground {
        return;
    }
    let bits = f.x221C_u16_y();
    if ext::db_804D4AF8() != 0 {
        let x58 = unsafe { &*(*f.ft_data).x58 };
        let scale_y = f.x34_scale.y;
        let mut ik = IKState::zeroed();
        ik.len0 = x58.x4 * scale_y;
        ik.len1 = scale_y * (x58.xC + x58.x18);
        if bits & 2 != 0 {
            leg(f, &mut ik, [x58.x0, x58.x8, x58.x10], -1.0);
        }
        if bits & 1 != 0 {
            leg(f, &mut ik, [x58.x1, x58.x9, x58.x11], 1.0);
        }
    }
    if bits & 4 != 0 {
        let line_id = f.coll_data.floor.index;
        let n = f.coll_data.floor.normal;
        let mut angle = f.facing_dir * atan2f(n.x, n.y);
        let mut sp38 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut sp2c = sp38;
        ext::mpLineGetV1Pos(line_id, &mut sp38);
        ext::mpLineGetV0Pos(line_id, &mut sp2c);
        let dy = sp38.y - sp2c.y;
        let dx = sp38.x - sp2c.x;
        let line_len = sqrtf(fmadds(dx, dx, dy * dy));
        if line_len < 5.0 {
            let mut adj = 0.0f32;
            let mut sp1c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let next = h::mp_line_get_next(line_id);
            if next != -1 && ext::mpLineGetKind(next) & CollLine_Floor as i32 != 0 {
                ext::mpLineGetNormal(next, &mut sp1c);
                adj = f.facing_dir * atan2f(sp1c.x, sp1c.y);
            }
            let prev = h::mp_line_get_prev(line_id);
            if prev != -1 && ext::mpLineGetKind(prev) & CollLine_Floor as i32 != 0 {
                ext::mpLineGetNormal(prev, &mut sp1c);
                adj = 0.5 * fmadds(f.facing_dir, atan2f(sp1c.x, sp1c.y), adj);
            }
            if abs_c(adj - angle) > 0.17453292 {
                angle = adj;
            }
        }
        let max_angle = 0.017453292f32 * h::common_data().x804;
        if angle > max_angle {
            angle = max_angle;
        } else if angle < -max_angle {
            angle = -max_angle;
        }
        ext::ftPartSetRotX(f, 0, angle);
    }
}

/// `ft_8008A1B8`: set the IK/tilt flags; without the tilt bit the root's X rotation is reset.
pub fn ft_8008A1B8(g: G, flags: u32) {
    melee_core::ct!("ft_8008A1B8");
    let f = fp(g);
    f.set_x221C_u16_y(flags as u16);
    if flags & (1 << 2) == 0 {
        ext::ftPartSetRotX(f, 0, 0.0);
    }
}
