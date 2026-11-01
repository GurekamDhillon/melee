//! src/melee/ft/ft_0DF1.c: already ported and C-verified in `crate::ft0df` (tests/ft0df_ref.rs).
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon::abs_c;
#[allow(unused_imports)]
use crate::ftcommon2 as fc2;
#[allow(unused_imports)]
use crate::ftcommon3::ftco_get_cstick_angle;
#[allow(unused_imports)]
use crate::hooks as h;
#[allow(unused_imports)]
use crate::types::*;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ft0df::{can_use_cstick, cd, cs, fp};
pub use crate::ft0df::*;

// ---- folded from ft0df.rs (consolidation steps 21/22) ----
/// `ftCo_800DF1C8`: horizontal flick past `dash_smash_stick_threshold`.
pub fn ftco_800df1c8(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df1c8");
    let t = cd().dash_smash_stick_threshold;
    abs_c(cs(fp, 1).x) < t && abs_c(cs(fp, 0).x) >= t
}

pub fn ftco_800df21c(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df21c"); can_use_cstick(fp) && ftco_800df1c8(fp) }

/// `ftCo_800DF2D8`: up flick past `xCC`.
pub fn ftco_800df2d8(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df2d8");
    let t = cd().xCC;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}

pub fn ftco_800df30c(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df30c"); can_use_cstick(fp) && ftco_800df2d8(fp) }

/// `ftCo_800DF3A8`: down flick past `xD4`.
pub fn ftco_800df3a8(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df3a8");
    let t = cd().xD4;
    cs(fp, 1).y > t && cs(fp, 0).y <= t
}

pub fn ftco_800df3dc(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df3dc"); can_use_cstick(fp) && ftco_800df3a8(fp) }

/// `ftCo_800DF478`: flick on either axis (`xDC` / `xE0`).
pub fn ftco_800df478(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df478");
    let c = cd();
    (abs_c(cs(fp, 1).x) < c.xDC && abs_c(cs(fp, 0).x) >= c.xDC)
        || (abs_c(cs(fp, 1).y) < c.xE0 && abs_c(cs(fp, 0).y) >= c.xE0)
}

pub fn ftco_800df50c(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df50c"); can_use_cstick(fp) && ftco_800df478(fp) }

/// `ftCo_800DF608`: `|cstick|^2 >= sdi_min_stick_mag^2`.
pub fn ftco_800df608(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df608");
    let v = cs(fp, 0);
    let m = cd().sdi_min_stick_mag;
    fmadds(v.x, v.x, v.y * v.y) >= m * m
}

pub fn ftco_800df644(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df644");
    let t = cd().x7F4;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}

pub fn ftco_800df678(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df678");
    let c = cd();
    abs_c(cs(fp, 1).x) < c.x248 && abs_c(cs(fp, 0).x) >= c.x248 && ftco_get_cstick_angle(fp) < c.x20_radians
}

pub fn ftco_800df6f8(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df6f8");
    let t = cd().x7F8;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}

pub fn ftco_800df72c(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df72c");
    if !h::gm_8016b0fc() {
        let t = cd().x7FC;
        if fp.facing_dir * cs(fp, 1).x < t && fp.facing_dir * cs(fp, 0).x >= t {
            return true;
        }
    }
    false
}

pub fn ftco_800df79c(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df79c");
    let t = cd().x494;
    abs_c(cs(fp, 0).x) >= t || abs_c(cs(fp, 0).y) >= t
}

pub fn ftco_800df7f4(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df7f4");
    let t = cd().x98;
    (cs(fp, 1).x < t && cs(fp, 0).x >= t) || (cs(fp, 1).x > -t && cs(fp, 0).x <= -t)
}

pub fn ftco_800df844(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df844");
    let t = cd().attackhi3_stick_threshold_y;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}

pub fn ftco_800df878(fp: &Fighter) -> bool {
    melee_core::ct!("ftco_800df878");
    let t = cd().xB0;
    cs(fp, 1).y <= t && cs(fp, 0).y <= t
}

pub fn ftco_800df8b0(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df8b0"); abs_c(cs(fp, 0).x) >= cd().x31C }

pub fn ftco_800df8e8(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df8e8"); cs(fp, 0).y <= cd().x314 }

pub fn ftco_800df910(fp: &Fighter) -> bool { melee_core::ct!("ftco_800df910"); cs(fp, 0).y >= cd().tap_jump_threshold }
