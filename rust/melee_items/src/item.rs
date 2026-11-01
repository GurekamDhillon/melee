
//! Ported item functions that only touch `Item` fields (no GObj, HSD or fighter calls yet).
//! Each cites the C function it mirrors; float expressions keep the C operation order and
//! precision (f32 unless the C promotes to f64).

use crate::layout::{itSamusMissileAttributes, GroundOrAir, HSD_GObj, Item};

/// GroundOrAir (src/melee/ft/types.h): GA_Ground = 0, GA_Air = 1.
pub const GA_GROUND: GroundOrAir = melee_types::GA_Ground;

pub const GA_AIR: GroundOrAir = melee_types::GA_Air;

/// `checkHitLag` (item.c, static): raise hitlag to at least `min_value`, set xDC8 flag xA.
pub fn check_hit_lag(min_value: f32, ip: &mut Item) {
    if ip.xCBC_hitlagFrames < min_value {
        ip.xCBC_hitlagFrames = min_value;
    }
    ip.xDC8_word.flags.set_xA(1);
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

pub use crate::tu::it_2725::it_80274f28;
pub use crate::tu::itmaplib::it_802762b0;
pub use crate::tu::itmaplib::it_802762bc;
pub use crate::tu::item::item_80269ca0;
pub use crate::tu::itlinkboomerang::norm_xf74;
