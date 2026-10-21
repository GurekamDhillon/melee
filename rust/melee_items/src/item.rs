//! Ported item functions that only touch `Item` fields (no GObj, HSD or fighter calls yet).
//! Each cites the C function it mirrors; float expressions keep the C operation order and
//! precision (f32 unless the C promotes to f64).

use crate::layout::{itSamusMissileAttributes, GroundOrAir, HSD_GObj, Item};

/// GroundOrAir (src/melee/ft/types.h): GA_Ground = 0, GA_Air = 1.
pub const GA_GROUND: GroundOrAir = melee_types::GA_Ground;
pub const GA_AIR: GroundOrAir = melee_types::GA_Air;

/// `Item_80269CA0` (item.c): add damage, clamp at 999.
pub fn item_80269ca0(ip: &mut Item, damage: i32) {
    ip.xC9C = ip.xC9C.wrapping_add(damage);
    if ip.xC9C > 999 {
        ip.xC9C = 999;
    }
}

/// `checkHitLag` (item.c, static): raise hitlag to at least `min_value`, set xDC8 flag xA.
pub fn check_hit_lag(min_value: f32, ip: &mut Item) {
    if ip.xCBC_hitlagFrames < min_value {
        ip.xCBC_hitlagFrames = min_value;
    }
    ip.xDC8_word.flags.set_xA(1);
}

/// `it_802762B0` (itmaplib.c).
pub fn it_802762b0(ip: &mut Item) {
    ip.ground_or_air = GA_GROUND;
}

/// `it_802762BC` (itmaplib.c).
pub fn it_802762bc(ip: &mut Item) {
    ip.ground_or_air = GA_AIR;
    ip.xDC8_word.flags.set_x1F(0);
}

/// `it_80274F28` (it_2725.c).
pub fn it_80274f28(ip: &mut Item, arg1: i8, arg2: Option<unsafe extern "C" fn(*mut HSD_GObj)>,
    arg3: Option<unsafe extern "C" fn(*mut HSD_GObj, *mut HSD_GObj)>) {
    // SAFETY: UnkFlagStruct is a union of a byte and its bit view; both are plain bits.
    unsafe { ip.xDD0_flag.anon_0.set_b5(1) };
    ip.xD08 = arg1 as u8;
    ip.grab_dealt = arg2;
    ip.grabbed_for_victim = arg3;
}

/// `decelerateItemX` (itparasol.c, static inline). `ABS` is a float compare-negate.
pub fn decelerate_item_x(ip: &mut Item, decel_x: f32) {
    let vx = ip.x40_vel.x;
    let abs = if vx < 0.0 { -vx } else { vx };
    if abs > decel_x {
        if ip.x40_vel.x > decel_x {
            ip.x40_vel.x -= decel_x;
        }
        if ip.x40_vel.x < -decel_x {
            ip.x40_vel.x += decel_x;
        }
    } else {
        ip.x40_vel.x = 0.0;
    }
}

/// `itSamusMissile_ClampTurn` (itsamusmissile.c, static inline).
pub fn its_samus_missile_clamp_turn(ip: &mut Item, sa: &itSamusMissileAttributes) {
    // SAFETY: the caller owns a Samus missile; xDD4_itemVar holds its samusmissile vars.
    unsafe {
        let turn = ip.xDD4_itemVar.samusmissile.x8;
        let abs_turn = if turn < 0.0 { -turn } else { turn };
        let max_turn = sa.x1C;
        if abs_turn > max_turn {
            ip.xDD4_itemVar.samusmissile.x8 = if turn > 0.0 { max_turn } else { -max_turn };
        }
    }
}

/// M_PI from src/MSL/math.h (a double literal).
pub const M_PI: f64 = 3.14159265358979323846;

/// `norm_xF74` (itlinkboomerang.c, static inline): angle in f64, stored as f32.
pub fn norm_xf74(ip: &mut Item) {
    // SAFETY: the caller owns a Link boomerang; xDD4_itemVar holds its linkboomerang vars.
    unsafe {
        let v = &mut ip.xDD4_itemVar.linkboomerang;
        let angle: f64 = if ip.facing_dir == 1.0 {
            v.xF74 as f64
        } else {
            v.xF74 as f64 - M_PI
        };
        v.xF78 = angle as f32;
    }
}

/// `itNesspkthunderball_ShiftAngles` (itnesspkthunderball.c, static inline).
pub fn pkthunderball_shift_angles(ip: &mut Item) {
    // SAFETY: the caller owns a PK Thunder ball; xDD4_itemVar holds its pkthunder vars.
    unsafe {
        let a = &mut ip.xDD4_itemVar.pkthunder.angles;
        let mut i = 14i32;
        while i >= 0 {
            a[(i + 1) as usize] = a[i as usize];
            i -= 1;
        }
    }
}
