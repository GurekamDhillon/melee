//! Leaf helpers from it/ and ty/ (port round 1, night 2): pure math and functions that only
//! touch `Item`/`ItemLink` fields. Each cites the C it mirrors. Float code keeps the C order and
//! precision, and fuses `a*b+c` exactly where the shipping build does
//! (`tools/rust-port/fma_sites.py`): an f32 site is `fmadds` (f64 product + f64 add, one f32
//! rounding), an f64 site is a true `f64::mul_add`. Verified against the C in tests/oracle.rs.
//!
//! Where the C takes an `Item_GObj*` and only uses `GET_ITEM(gobj)`, the port takes `&mut Item`
//! (the GObj layer is group 1's; callers pass the user data).

use crate::layout::{
    ftCommonData, itECB, itSeakChain_Attrs, Collide_CeilingHug, Collide_CeilingPush, GroundOrAir,
    Item, ItemCommonData, ItemLink, Vec as GcVec, GA_Air,
};
use melee_core::fp::fmadds;

/// M_PI (src/MSL/math.h), M_PI_2 = M_PI / 2, M_TAU (src/Runtime/platform.h): double literals.
pub const M_PI: f64 = 3.14159265358979323846;
pub const M_PI_2: f64 = M_PI / 2.0;
pub const M_TAU: f64 = 6.283185307179586;
/// `Collide_CeilingMask` (src/melee/mp/forward.h).
pub const COLLIDE_CEILING_MASK: i32 = Collide_CeilingPush | Collide_CeilingHug;

// ---- vector products -------------------------------------------------------------------

/// `product_xy` (itmaplib.c, static inline). 1 fused site: `fmadds(ax, bx, ay*by)`.
pub fn product_xy(a: &GcVec, b: &GcVec) -> f32 {
    fmadds(a.x, b.x, a.y * b.y)
}

/// `product_xyz` (itzako.c, static inline). 2 fused sites.
pub fn product_xyz(a: &GcVec, b: &GcVec) -> f32 {
    let xy = fmadds(a.x, b.x, a.y * b.y);
    fmadds(a.z, b.z, xy)
}

/// `neg_dot` (itleadead.c, static inline): `-(ax*bx) - (ay*by) - (az*bz)`.
/// Clang folds it into 2 fused sites: `fmadds(-ax, bx, -(ay*by))`, then `fmadds(-az, bz, _)`.
pub fn neg_dot(a: &GcVec, b: &GcVec) -> f32 {
    let t = fmadds(-a.x, b.x, -(a.y * b.y));
    fmadds(-a.z, b.z, t)
}

/// `_tyDisplay_80318CB4_calc_dist_sq` (tydisplay.c, static inline). 1 site: `fmadds(dx, dx, dz*dz)`.
pub fn ty_display_calc_dist_sq(dz: f32, dx: f32) -> f32 {
    fmadds(dx, dx, dz * dz)
}

/// `it_link_lerp` (itlinkhookshot.c, static inline). 1 site: `fmadds(t, a, (1-t)*b)`.
pub fn it_link_lerp(a: f32, b: f32, t: f32) -> f32 {
    fmadds(t, a, (1.0 - t) * b)
}

// ---- angles ----------------------------------------------------------------------------

/// `it_8027B798_CalcAngle` (itzako.c, static inline). 1 site: `fmadds(x148, ratio, 1.0)`.
/// `angle` 0x169 (361) is the Sakurai angle.
pub fn it_8027b798_calc_angle(ground_or_air: GroundOrAir, angle: i32, cd: &ftCommonData,
    knockback: f32) -> f32 {
    if angle != 0x169 {
        0.017453292f32 * angle as f32
    } else if ground_or_air == GA_Air {
        cd.x144_radians
    } else if knockback < cd.x14C {
        0.0
    } else {
        let ratio = (knockback - cd.x14C) / (cd.x150 - cd.x14C);
        let mut temp_angle = 0.017453292f32 * fmadds(cd.x148, ratio, 1.0);
        if temp_angle > 0.017453292f32 * cd.x148 {
            temp_angle = 0.017453292f32 * cd.x148;
        }
        temp_angle
    }
}

/// `clamp_tau` (itlinkboomerang.c, static): compares and adds in f64 (M_TAU is a double).
pub fn clamp_tau(angle: &mut f32) {
    if (*angle as f64) > M_TAU {
        *angle = (*angle as f64 - M_TAU) as f32;
    } else if (*angle as f64) < -M_TAU {
        *angle = (*angle as f64 + M_TAU) as f32;
    }
}

/// `clamp_pi_tau` (itlinkboomerang.c, static).
pub fn clamp_pi_tau(angle: &mut f32) {
    if (*angle as f64) < -M_PI {
        *angle = (*angle as f64 + M_TAU) as f32;
    } else if (*angle as f64) > M_PI {
        *angle = (*angle as f64 - M_TAU) as f32;
    }
}

/// `clamp_angle_pi` (itlinkboomerang.c, static inline): f32 compare, f64 add.
pub fn clamp_angle_pi(angle: &mut f32) {
    if *angle <= 0.0 {
        *angle = (*angle as f64 + M_PI) as f32;
    } else {
        *angle = (*angle as f64 - M_PI) as f32;
    }
}

/// `norm_xF74_from_angle` (itlinkboomerang.c, static inline).
pub fn norm_xf74_from_angle(ip: &mut Item, angle_in: f32) {
    let angle: f64 = if ip.facing_dir == 1.0 { angle_in as f64 } else { angle_in as f64 - M_PI };
    // The caller owns a Link boomerang (xDD4_itemVar holds its linkboomerang vars).
    ip.xDD4_itemVar.linkboomerang.xF78 = angle as f32;
}

/// `itDosei_FacingAngle` (itdosei.c, static inline): `fd*M_PI_2 + fd*m` in f64, one f64 fused
/// site (`fma(fd, M_PI_2, (double)(fd*m))`, the `fd*m` product in f32), stored as f32.
pub fn it_dosei_facing_angle(ip: &Item, m: f32) -> f32 {
    let fd = ip.facing_dir;
    (fd as f64).mul_add(M_PI_2, (fd * m) as f64) as f32
}

// ---- scalar helpers --------------------------------------------------------------------

/// `itLinkboomerang_UnkMotion3_Phys_add_clamp` (itlinkboomerang.c, static inline).
pub fn link_boomerang_add_clamp(mut x: f32, y: f32, max: f32) -> f32 {
    x += y;
    if x > max {
        x = max;
    }
    x
}

/// `fsign_inline` (itlinkbomb.c, static inline).
pub fn fsign_inline(x: f32) -> f32 {
    if x >= 0.0 { 1.0 } else { -1.0 }
}

/// `float_sign_int_inline` (itlinkbomb.c, static inline).
pub fn float_sign_int_inline(x: f32) -> i32 {
    if x == -1.0 { -1 } else { 1 }
}

/// `inline_fabsf` (ittaru.c, static inline). `+x` keeps a NaN or -0.0 as is.
pub fn inline_fabsf(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `lt_zero` (itnokonoko.c, static inline).
pub fn lt_zero(val: f32) -> f32 {
    if val < 0.0 { -1.0 } else { 1.0 }
}

/// `getX` (itoctarockstone.c, static inline): |x1 - x2| by compare-negate.
pub fn octarock_stone_get_x(x1: f32, x2: f32) -> f32 {
    let res = x1 - x2;
    if res < 0.0 { -res } else { res }
}

/// `pika_scale` (itpikachuthunder.c, static): `(s32)` is gwtool's saturating conversion
/// (NaN -> 0), which is what Rust's `as i32` does.
pub fn pika_scale(a: f32, b: f32) -> f32 {
    let f = 10000.0f32 * a / b;
    ((f + 1.0) as i32) as f32 / 10000.0
}

/// `tyFigupon_StoreDigits` (tyfigupon.c, static inline): writes least significant digit first,
/// at least one; C `%` and `/` truncate toward zero like Rust's. Panics if `digits` is too short
/// (the C would write past the end).
pub fn ty_figupon_store_digits(digits: &mut [i32], mut value: i32) {
    let mut i = 0;
    loop {
        digits[i] = value % 10;
        i += 1;
        value /= 10;
        if value <= 0 {
            break;
        }
    }
}

// ---- collision ---------------------------------------------------------------------------

/// `itColl_chkECBOverlap` (itcoll.c, static).
pub fn it_coll_chk_ecb_overlap(pos_x: f32, pos_y: f32, a: &itECB, b: &itECB, target: &GcVec)
    -> bool {
    let top = pos_y + (a.top + b.top);
    let bottom = pos_y + (a.bottom + b.bottom);
    let right = pos_x + (a.right + b.right);
    let left = pos_x + (a.left + b.left);
    top >= target.y && bottom <= target.y && right <= target.x && left >= target.x
}

/// `it_802763E0` (itmaplib.c): 2 and record the ceiling surface when touching one, else 0.
pub fn it_802763e0(ip: &mut Item) -> i32 {
    let mut result = 0;
    if ip.x378_itemColl.env_flags & COLLIDE_CEILING_MASK != 0 {
        ip.xC30 = ip.x378_itemColl.ceiling.index;
        result = 2;
    }
    result
}

// ---- lifetime ----------------------------------------------------------------------------

/// `decrease_lifetimer` (itpeachturnip.c, static): `--` on an f32 subtracts 1.0f.
pub fn decrease_lifetimer(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer -= 1.0;
    ip.xD44_lifeTimer <= 0.0
}

/// `it_80275158` (it_2725.c). `common` is the C global `it_804D6D28` (ItemCommonData), passed
/// in until the item globals exist natively.
pub fn it_80275158(ip: &mut Item, lifetime: f32, common: &ItemCommonData) {
    ip.xD44_lifeTimer = lifetime;
    ip.xD48_halfLifeTimer = lifetime * common.x4C_float;
}

// ---- Red shell (itrshell.c) -------------------------------------------------------------

/// `itRShell_Attrs`: file-local in itrshell.c, so not in the generated set. Same field order;
/// GameCube offsets in the `gc_layout!` below (ppc32: all 4-byte fields, size 0x58).
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct itRShell_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: GcVec,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub pad30: [u8; 8],
    pub x38: f32,
    pub x3C: f32,
    pub x40: f32,
    pub x44: f32,
    pub x48: GcVec,
    pub x54: i32,
}
impl itRShell_Attrs {
    /// All-zero is a valid value (plain floats).
    pub fn zeroed() -> Self {
        // SAFETY: every field is a float, integer or byte array.
        unsafe { core::mem::zeroed() }
    }
}
melee_core::gc_layout!(itRShell_Attrs, "itRShell_Attrs", size 0x58, align 4, [
    x0 @ 0x0: 4 "float", x4 @ 0x4: 4 "float", x8 @ 0x8: 4 "float", xC @ 0xC: 4 "float",
    x10 @ 0x10: 4 "float", x14 @ 0x14: 12 "Vec3", x20 @ 0x20: 4 "float", x24 @ 0x24: 4 "float",
    x28 @ 0x28: 4 "float", x2C @ 0x2C: 4 "float", pad30 @ 0x30: 8 "char[8]",
    x38 @ 0x38: 4 "float", x3C @ 0x3C: 4 "float", x40 @ 0x40: 4 "float", x44 @ 0x44: 4 "float",
    x48 @ 0x48: 12 "Vec", x54 @ 0x54: 4 "s32",
]);

/// `itRshell_ClampVel` (itrshell.c, static inline). `attrs` = the C's
/// `ip->xC4_article_data->x4_specialAttributes`.
pub fn it_rshell_clamp_vel(ip: &mut Item, attrs: &itRShell_Attrs) {
    let vel = ip.x40_vel.x;
    let abs_vel = if vel < 0.0 { -vel } else { vel };
    if abs_vel > attrs.xC {
        let sign: i32 = if vel < 0.0 { -1 } else { 1 };
        ip.x40_vel.x = sign as f32;
        ip.x40_vel.x *= attrs.xC;
    }
}

/// `itRshell_UM5_Accel` (itrshell.c, static inline).
pub fn it_rshell_um5_accel(ip: &mut Item, attrs: &itRShell_Attrs, target: &GcVec) {
    let sign: i32 = if target.x - ip.pos.x < 0.0 { -1 } else { 1 };
    // The caller owns a red shell (xDD4_itemVar holds its rshell vars).
    ip.xDD4_itemVar.rshell.xDE0 = attrs.x8 * sign as f32;
}

/// `itRshell_UM5_AddVelAndCheck` (itrshell.c, static inline).
pub fn it_rshell_um5_add_vel_and_check(ip: &mut Item) -> bool {
    // SAFETY: as above.
    let accel = unsafe { ip.xDD4_itemVar.rshell.xDE0 };
    ip.x40_vel.x += accel;
    let sign_accel = if accel < 0.0 { -1 } else { 1 };
    let sign_vx = if ip.x40_vel.x < 0.0 { -1 } else { 1 };
    sign_vx == sign_accel
}

/// `itRshell_UM5C_Reverse` (itrshell.c, static inline).
pub fn it_rshell_um5c_reverse(ip: &mut Item) {
    ip.x40_vel.x = -ip.x40_vel.x;
    // SAFETY: as above.
    unsafe { ip.xDD4_itemVar.rshell.xDE0 = -ip.xDD4_itemVar.rshell.xDE0 };
}

// ---- Sheik chain (itseakchain.c) ---------------------------------------------------------

fn seak_chain_clamp(link: &mut ItemLink, limit: f32) {
    let value = link.vel.x;
    link.vel.x = if value > limit {
        value - limit
    } else if value < -limit {
        value + limit
    } else {
        0.0
    };
}

/// `itSeakChain_clamp_x10` (itseakchain.c, static inline).
pub fn it_seak_chain_clamp_x10(link: &mut ItemLink, attrs: &itSeakChain_Attrs) {
    seak_chain_clamp(link, attrs.x10);
}

/// `itSeakChain_clamp_x14` (itseakchain.c, static inline).
pub fn it_seak_chain_clamp_x14(link: &mut ItemLink, attrs: &itSeakChain_Attrs) {
    seak_chain_clamp(link, attrs.x14);
}
