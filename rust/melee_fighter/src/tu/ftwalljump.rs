//! Hand port of src/melee/ft/ftwalljump.c (its one function, `ftWallJump_8008169C`, the
//! walljump interrupt). No fused sites. Checked against the C (cut verbatim, shipping
//! contraction) by tests/tu_ftwalljump.rs.
#![allow(static_mut_refs)]

use crate::hooks;
use crate::types::*;

pub type G = *mut HSD_GObj;

/// Calls out of this TU whose owners are not ported (or not wired) yet: settable fn pointers,
/// panicking with a TODO when reached unset.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        // group 2 mp/mplib.c
        MP_GET_SPEED mpGetSpeed(line_id: i32, pos: &mut Vec3, speed: &mut Vec3) -> bool;
        // ftCo_PassiveWall.c
        FTCO_800C1E64 ftCo_800C1E64(g: G, msid: i32, timer: i32, vel_y_exponent: i32, facing_dir: f32);
    }
}

const MAX_INPUT_FRAMES: u8 = 254;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ftWallJump_8008169C` (UnclePunch: Interrupt_Walljump): `true` if it started a walljump.
pub fn ftwalljump_8008169c(g: G) -> bool {
    melee_core::ct!("ftwalljump_8008169c");
    let f = fp(g);
    if f.can_walljump() == 0 {
        return false;
    }
    let env = f.coll_data.env_flags;
    if env & Collide_RightWallHug != 0 || env & Collide_LeftWallHug != 0 {
        let right = env & Collide_RightWallHug;
        let wall_dir: f32 = if right != 0 { -1.0 } else { 1.0 };
        if f.wall_jump_input_timer < MAX_INPUT_FRAMES && wall_dir == f.x2110_walljumpWallSide {
            f.wall_jump_input_timer += 1;
        } else {
            let mut wall_pos = Vec3::zeroed();
            let mut ecb = Vec3::zeroed();
            let (v, line) = if right != 0 {
                (f.coll_data.ecb.left, f.coll_data.right_facing_wall.index)
            } else {
                (f.coll_data.ecb.right, f.coll_data.left_facing_wall.index)
            };
            ecb.x = v.x;
            ecb.y = v.y;
            ecb.z = 0.0;
            ecb.x += f.cur_pos.x;
            ecb.y += f.cur_pos.y;
            ecb.z += f.cur_pos.z;
            if !ext::mpGetSpeed(line as i32, &mut ecb, &mut wall_pos) {
                wall_pos.x = 0.0;
            }
            let f = fp(g);
            let mut rel = f.pos_delta.x - wall_pos.x;
            rel = if rel < 0.0 { -rel } else { rel };
            if rel > f.co_attrs.wall_jump_min_approach_speed {
                f.x2110_walljumpWallSide = wall_dir;
                f.wall_jump_input_timer = 0;
            }
        }
        let f = fp(g);
        let cd = hooks::common_data();
        let side = f.x2110_walljumpWallSide;
        let lx = f.input.lstick[0].x;
        if (f.wall_jump_input_timer as f32) < cd.x768
            && ((side == -1.0 && lx >= cd.x76C) || (side == 1.0 && lx <= -cd.x76C))
            && (f.active_timer.lstick.x as f32) < cd.x770
        {
            let used = f.x1969_walljumpUsed as i32;
            ext::ftCo_800C1E64(g, ftCo_MS_PassiveWallJump, cd.x774, used, side);
            let f = fp(g);
            f.wall_jump_input_timer = MAX_INPUT_FRAMES;
            if f.x1969_walljumpUsed < 255 {
                f.x1969_walljumpUsed += 1;
            }
            return true;
        }
    } else {
        f.wall_jump_input_timer = MAX_INPUT_FRAMES;
    }
    false
}
