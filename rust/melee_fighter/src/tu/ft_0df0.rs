//! src/melee/ft/ft_0DF0.c: already ported and C-verified in `crate::ft0df` (tests/ft0df_ref.rs).
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
use crate::ft0df::{G, HSD_PAD_A, SMASH_STATE_4, charge_sfx, fp, release, tick};
pub use crate::ft0df::*;

// ---- folded from ft0df.rs (consolidation steps 21/22) ----
// ---- ft_0DF0.c ---------------------------------------------------------------------------

/// `ftCo_800DEE84(gobj, arg1, hold_frame, dmg_mult)`: arm a smash charge.
pub fn ftco_800dee84(g: G, arg1: u32, arg2: f32, dmg_mult: f32) {
    melee_core::ct!("ftco_800dee84");
    let a = &mut fp(g).smash_attrs;
    a.state = SmashState_PreCharge as _;
    a.x2118_frames = 0.0;
    a.x211C_holdFrame = arg2;
    a.x2120_damageMul = dmg_mult;
    a.x2128 = arg1 as i32;
}

/// `ftCo_800DEEA8`.
pub fn ftco_800deea8(g: G) {
    melee_core::ct!("ftco_800deea8");
    fp(g).smash_attrs.state = SmashState_None as _;
}

/// `ftCo_800DEEB8(fp, x)`: scale `x` by the charged damage multiplier once released.
pub fn ftco_800deeb8(fp: &Fighter, arg1: f32) -> f32 {
    melee_core::ct!("ftco_800deeb8");
    let a = &fp.smash_attrs;
    if a.state as i32 != SmashState_Release as i32 {
        return arg1;
    }
    arg1 * fmadds(a.x2120_damageMul - 1.0, a.x2118_frames / a.x211C_holdFrame, 1.0)
}

/// `ftCo_800DEEE8(fp, &shift)`: the charge shake offset (C returns `shift`, or NULL).
pub fn ftco_800deee8(fp: &mut Fighter) -> Option<Vec2> {
    melee_core::ct!("ftco_800deee8");
    let a = &fp.smash_attrs;
    let st = a.state as i32;
    if st == SmashState_Charging as i32 || st == SMASH_STATE_4 {
        let t = h::smash_charge_shake_table();
        let v = unsafe { *t.x0.add(a.x212C as usize) };
        return Some(Vec2 { x: v.x, y: v.y });
    }
    None
}

/// `ftCo_800DEF38`: per-frame charge update.
pub fn ftco_800def38(g: G) {
    melee_core::ct!("ftco_800def38");
    let f = fp(g);
    let st = f.smash_attrs.state as i32;
    if st == SmashState_Charging as i32 {
        tick(f);
        if f.smash_attrs.x2118_frames >= f.smash_attrs.x211C_holdFrame {
            if f.smash_attrs.x2118_frames > f.smash_attrs.x211C_holdFrame {
                f.smash_attrs.x2118_frames = f.smash_attrs.x211C_holdFrame;
            }
            release(g);
        }
        charge_sfx(fp(g));
    } else if st == SMASH_STATE_4 {
        tick(f);
        charge_sfx(f);
    }
}

/// `ftCo_800DF0D0`: A held/released transitions.
pub fn ftco_800df0d0(g: G) {
    melee_core::ct!("ftco_800df0d0");
    let f = fp(g);
    let st = f.smash_attrs.state as i32;
    if st == SmashState_PreCharge as i32 {
        if f.input.held_buttons[0] & HSD_PAD_A != 0 {
            f.smash_attrs.state = SmashState_Charging as _;
            f.smash_attrs.x2124_frameSpeedMul = f.frame_speed_mul;
            f.smash_attrs.x212C = 0;
            f.smash_attrs.x212D = h::smash_charge_shake_table().x4 as u8;
            f.smash_attrs.x2130_sfxBool = 0;
            h::ftanim_set_anim_rate(g, 0.0);
            let k = fp(g).smash_attrs.x2128;
            if k != 0x7B {
                h::ftco_800bffd0(fp(g), k, 0);
            }
            fc2::ftcommon_8007ebac(fp(g), 0x24, 0);
        } else {
            f.smash_attrs.state = SmashState_None as _;
        }
    } else if st == SmashState_Charging as i32 {
        if f.input.held_buttons[0] & HSD_PAD_A == 0 {
            release(g);
        }
    }
}
