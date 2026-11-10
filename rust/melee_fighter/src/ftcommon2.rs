
//! Hand port, round 2: the field logic of src/melee/ft/ftcommon.c (lines 246-256, 541-620,
//! 672-686, 777-827, 993-1046, 1302-1309, 1325-1355, 1379-1382, 1581-1584, 1596-1666,
//! 1808-2024): jumps/ECB, grab mash, velocity setters, rumble triggers, held-item scale, stick
//! activity. Same rules as `ftcommon` (C operation order, `ABS` = `abs_c`, gobj -> fighter).
//! Fused sites follow the shipping build (`tools/rust-port/fma_sites.py src/melee/ft/ftcommon.c`):
//! 8007ED50, 8007EE0C, 8007FDA0, 80080174 have one f32 fmuladd each -> `fmadds`.
//! Calls out of the ported set go through `crate::hooks`.
//! Checked against the C (cut verbatim, built with the shipping contraction) by
//! tests/ftcommon2_ref.rs.

use crate::ftcommon::abs_c;

use crate::hooks as h;

use crate::hooks::{common_data, ground_friction_mul};

use crate::types::{
    Fighter, Ft_Kind_Nana, Ft_Kind_Popo, Ft_Kind_Sandbag, GA_Air, GA_Ground, HSD_GObjEvent, HSD_GObjInteraction,
    MtxPtr, Vec3,
};

use melee_core::fp::fmadds;

/// `CollData_X130_Locked` (mp/forward.h).
pub(crate) const COLL_X130_LOCKED: u32 = 1 << 4;

/// `HSD_PAD_AB | HSD_PAD_XY | HSD_PAD_LR` (baselib/controller.h).
pub(crate) const PAD_ABXYLR: u32 = (1 << 8) | (1 << 9) | (1 << 10) | (1 << 11) | (1 << 31);

#[inline(always)]
pub(crate) fn ids(fp: &Fighter) -> (i32, i32) {
    (fp.player_idx as i32, fp.is_sub_fighter() as i32)
}

/// `ftCommon_SandbagGetKnockbackDeaccelX`: `((float*) fp->dat_attrs)[0]`.
pub fn sandbag_get_knockback_deaccel_x(fp: &Fighter) -> f32 {
    assert!(fp.kind == Ft_Kind_Sandbag, "ftcommon.c:299");
    unsafe { *(fp.dat_attrs as *const f32) }
}

/// `ftCommon_SandbagGetKnockbackDeaccelY`: `((float*) fp->dat_attrs)[1]`.
pub fn sandbag_get_knockback_deaccel_y(fp: &Fighter) -> f32 {
    assert!(fp.kind == Ft_Kind_Sandbag, "ftcommon.c:308");
    unsafe { *(fp.dat_attrs as *const f32).add(1) }
}

pub fn unlock_ecb(fp: &mut Fighter) {
    fp.ecb_lock = 0;
    fp.coll_data.x130_flags &= !COLL_X130_LOCKED;
}

pub fn use_all_jumps(fp: &mut Fighter) {
    fp.x1968_jumpsUsed = fp.co_attrs.max_jumps as u8;
}

/// `ftCommon_InitGrab(fp, bool arg1, timer)`; `bool` is MSL `int`, stored into a `:1`.
pub fn init_grab(fp: &mut Fighter, arg1: i32, timer: f32) {
    fp.grab_timer = timer;
    fp.x1A51 = 0;
    fp.x1A50 = 0;
    fp.set_x2224_b6(arg1 as u8);
    if fp.x2224_b6() != 0 {
        fp.x1A52 = 0;
        fp.x1A53 = h::grab_mash_shake().x4 as u8;
    }
}

/// Stick-direction latch shared by GrabMash and 8007FFD8.
#[inline(always)]
pub(crate) fn mash_dir(x: f32, y: f32, dx: &mut i8, dy: &mut i8) {
    let t = common_data().x308;
    if x < -t {
        *dx = -1;
    }
    if x > t {
        *dx = 1;
    }
    if y < -t {
        *dy = -1;
    }
    if y > t {
        *dy = 1;
    }
}

pub fn grab_mash(fp: &mut Fighter, arg1: f32) -> bool {
    let mut result = false;
    if fp.input.pressed_buttons & PAD_ABXYLR != 0 {
        fp.grab_timer -= arg1;
        result = true;
    }
    let (r5, r6) = (fp.x1A50, fp.x1A51);
    let (mut dx, mut dy) = (r5, r6);
    mash_dir(fp.input.lstick[0].x, fp.input.lstick[0].y, &mut dx, &mut dy);
    fp.x1A50 = dx;
    fp.x1A51 = dy;
    if r5 != fp.x1A50 || r6 != fp.x1A51 {
        fp.grab_timer -= arg1;
        result = true;
    }
    if result && fp.x2224_b6() != 0 {
        fp.set_x2224_b5(1);
        fp.x1A52 = fp.x1A52.wrapping_add(1);
        if fp.x1A52 >= fp.x1A53 {
            fp.x1A52 = 0;
        }
    } else {
        fp.set_x2224_b5(0);
    }
    let (s, u) = ids(fp);
    h::pl_800402d0(s, u, result as i32);
    result
}

pub fn get_model_scale(fp: &Fighter) -> f32 {
    fp.x34_scale.y * fp.co_attrs.model_scaling
}

/// `x710 * x2024 + x708`: one fmuladd in the shipping build (8007FDA0, 80080174) -> fmadds.
#[inline(always)]
pub(crate) fn held_item_scale_raw(fp: &Fighter) -> f32 {
    let cd = common_data();
    fmadds(cd.x710, fp.x2024 as f32, cd.x708)
}

pub use crate::tu::ftcommon::ftcommon_8007d5d4;
pub use crate::tu::ftcommon::ftcommon_8007d60c;
pub use crate::tu::ftcommon::ftcommon_8007d780;
pub use crate::tu::ftcommon::ftcommon_8007e2a4;
pub use crate::tu::ftcommon::ftcommon_8007e2d0;
pub use crate::tu::ftcommon::ftcommon_8007e2f4;
pub use crate::tu::ftcommon::ftcommon_8007e2fc;
pub use crate::tu::ftcommon::ftcommon_8007ebac;
pub use crate::tu::ftcommon::ftcommon_8007ecd4;
pub use crate::tu::ftcommon::ftcommon_8007ed2c;
pub use crate::tu::ftcommon::ftcommon_8007ed50;
pub use crate::tu::ftcommon::ftcommon_8007ee0c;
pub use crate::tu::ftcommon::ftcommon_8007efc0;
pub use crate::tu::ftcommon::ftcommon_8007f76c;
pub use crate::tu::ftcommon::ftcommon_8007f7b4;
pub use crate::tu::ftcommon::ftcommon_8007f804;
pub use crate::tu::ftcommon::ftcommon_8007f8b4;
pub use crate::tu::ftcommon::ftcommon_8007fda0;
pub use crate::tu::ftcommon::ftcommon_8007ff74;
pub use crate::tu::ftcommon::ftcommon_8007ffd8;
pub use crate::tu::ftcommon::ftcommon_80080144;
pub use crate::tu::ftcommon::ftcommon_80080174;
pub use crate::tu::ftcommon::ftcommon_8008021c;
pub use crate::tu::ftcommon::ftcommon_8008031c;
pub use crate::tu::ftcommon::ftcommon_80080460;
pub use crate::tu::ftcommon::ftcommon_80080474;
pub use crate::tu::ftcommon::ftcommon_80080484;
pub use crate::tu::ftcommon::ftcommon_800804a0;
pub use crate::tu::ftcommon::ftcommon_800804ec;
pub use crate::tu::ftcommon::ftcommon_800804fc;
pub use crate::tu::ftcommon::my_fminf;
