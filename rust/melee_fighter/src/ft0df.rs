//! Hand port, round 13: all of src/melee/ft/ft_0DF0.c (smash-attack charge state machine:
//! `smash_attrs`) and all of ft_0DF1.c (C-stick flick / hold tests against PlCo thresholds),
//! plus field leaves of ftcoll.c (`skip_update_pos` resets, ftColl_8007B8A8/B8CC), ftdevice.c
//! (`ftCo_800C0658` colour-overlay select) and ftparts.c (`x5F4_arr` model-part visibility).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): ftCo_800DEEB8 `(mul - 1) * (frames / hold) + 1`
//! and ftCo_800DF608 `SQ(x) + SQ(y)` (left product fused, the right one rounded first); both ->
//! `fmadds`. Checked against the C (cut verbatim, shipping contraction) by tests/ft0df_ref.rs.

use crate::ftcommon::abs_c;
use crate::ftcommon2 as fc2;
use crate::ftcommon3::ftco_get_cstick_angle;
use crate::hooks as h;
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

const HSD_PAD_A: u32 = 0x100;
const SMASH_STATE_4: i32 = 4;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    unsafe { &*h::P_FT_COMMON_DATA }
}

// ---- ft_0DF0.c ---------------------------------------------------------------------------

/// `ftCo_800DEE84(gobj, arg1, hold_frame, dmg_mult)`: arm a smash charge.
pub fn ftco_800dee84(g: G, arg1: u32, arg2: f32, dmg_mult: f32) {
    let a = &mut fp(g).smash_attrs;
    a.state = SmashState_PreCharge as _;
    a.x2118_frames = 0.0;
    a.x211C_holdFrame = arg2;
    a.x2120_damageMul = dmg_mult;
    a.x2128 = arg1 as i32;
}

/// `ftCo_800DEEA8`.
pub fn ftco_800deea8(g: G) {
    fp(g).smash_attrs.state = SmashState_None as _;
}

/// `ftCo_800DEEB8(fp, x)`: scale `x` by the charged damage multiplier once released.
pub fn ftco_800deeb8(fp: &Fighter, arg1: f32) -> f32 {
    let a = &fp.smash_attrs;
    if a.state as i32 != SmashState_Release as i32 {
        return arg1;
    }
    arg1 * fmadds(a.x2120_damageMul - 1.0, a.x2118_frames / a.x211C_holdFrame, 1.0)
}

/// `ftCo_800DEEE8(fp, &shift)`: the charge shake offset (C returns `shift`, or NULL).
pub fn ftco_800deee8(fp: &mut Fighter) -> Option<Vec2> {
    let a = &fp.smash_attrs;
    let st = a.state as i32;
    if st == SmashState_Charging as i32 || st == SMASH_STATE_4 {
        let t = h::smash_charge_shake_table();
        let v = unsafe { *t.x0.add(a.x212C as usize) };
        return Some(Vec2 { x: v.x, y: v.y });
    }
    None
}

fn tick(fp: &mut Fighter) {
    let a = &mut fp.smash_attrs;
    a.x2118_frames += 1.0;
    a.x212C = a.x212C.wrapping_add(1);
    if a.x212C >= a.x212D {
        a.x212C = 0;
    }
}

fn charge_sfx(fp: &mut Fighter) {
    if fp.smash_attrs.x2130_sfxBool == 0 && fp.smash_attrs.x2118_frames >= cd().x7C8 {
        h::play_sfx(fp, 0x7B, 0x7F, 0x40);
        fp.smash_attrs.x2130_sfxBool = 1;
    }
}

fn release(g: G) {
    let f = fp(g);
    f.smash_attrs.state = SmashState_Release as _;
    let rate = f.smash_attrs.x2124_frameSpeedMul;
    h::ftanim_set_anim_rate(g, rate);
    let k = fp(g).smash_attrs.x2128;
    h::ftco_800c0200(fp(g), k);
    fc2::ftcommon_8007ecd4(fp(g), 0x24);
}

/// `ftCo_800DEF38`: per-frame charge update.
pub fn ftco_800def38(g: G) {
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

// ---- ft_0DF1.c ---------------------------------------------------------------------------

/// `canUseCstick` (static inline): single-button mode off, or the held item allows the C-stick.
pub fn can_use_cstick(fp: &Fighter) -> bool {
    !h::gm_8016b0fc() || h::it_8026b30c(fp.item_gobj) == 0
}

fn cs(fp: &Fighter, i: usize) -> Vec2 {
    let v = &fp.input.cstick[i];
    Vec2 { x: v.x, y: v.y }
}

/// `ftCo_800DF1C8`: horizontal flick past `dash_smash_stick_threshold`.
pub fn ftco_800df1c8(fp: &Fighter) -> bool {
    let t = cd().dash_smash_stick_threshold;
    abs_c(cs(fp, 1).x) < t && abs_c(cs(fp, 0).x) >= t
}
pub fn ftco_800df21c(fp: &Fighter) -> bool { can_use_cstick(fp) && ftco_800df1c8(fp) }
/// `ftCo_800DF2D8`: up flick past `xCC`.
pub fn ftco_800df2d8(fp: &Fighter) -> bool {
    let t = cd().xCC;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}
pub fn ftco_800df30c(fp: &Fighter) -> bool { can_use_cstick(fp) && ftco_800df2d8(fp) }
/// `ftCo_800DF3A8`: down flick past `xD4`.
pub fn ftco_800df3a8(fp: &Fighter) -> bool {
    let t = cd().xD4;
    cs(fp, 1).y > t && cs(fp, 0).y <= t
}
pub fn ftco_800df3dc(fp: &Fighter) -> bool { can_use_cstick(fp) && ftco_800df3a8(fp) }
/// `ftCo_800DF478`: flick on either axis (`xDC` / `xE0`).
pub fn ftco_800df478(fp: &Fighter) -> bool {
    let c = cd();
    (abs_c(cs(fp, 1).x) < c.xDC && abs_c(cs(fp, 0).x) >= c.xDC)
        || (abs_c(cs(fp, 1).y) < c.xE0 && abs_c(cs(fp, 0).y) >= c.xE0)
}
pub fn ftco_800df50c(fp: &Fighter) -> bool { can_use_cstick(fp) && ftco_800df478(fp) }
/// `ftCo_800DF608`: `|cstick|^2 >= sdi_min_stick_mag^2`.
pub fn ftco_800df608(fp: &Fighter) -> bool {
    let v = cs(fp, 0);
    let m = cd().sdi_min_stick_mag;
    fmadds(v.x, v.x, v.y * v.y) >= m * m
}
pub fn ftco_800df644(fp: &Fighter) -> bool {
    let t = cd().x7F4;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}
pub fn ftco_800df678(fp: &Fighter) -> bool {
    let c = cd();
    abs_c(cs(fp, 1).x) < c.x248 && abs_c(cs(fp, 0).x) >= c.x248 && ftco_get_cstick_angle(fp) < c.x20_radians
}
pub fn ftco_800df6f8(fp: &Fighter) -> bool {
    let t = cd().x7F8;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}
pub fn ftco_800df72c(fp: &Fighter) -> bool {
    if !h::gm_8016b0fc() {
        let t = cd().x7FC;
        if fp.facing_dir * cs(fp, 1).x < t && fp.facing_dir * cs(fp, 0).x >= t {
            return true;
        }
    }
    false
}
pub fn ftco_800df79c(fp: &Fighter) -> bool {
    let t = cd().x494;
    abs_c(cs(fp, 0).x) >= t || abs_c(cs(fp, 0).y) >= t
}
pub fn ftco_800df7f4(fp: &Fighter) -> bool {
    let t = cd().x98;
    (cs(fp, 1).x < t && cs(fp, 0).x >= t) || (cs(fp, 1).x > -t && cs(fp, 0).x <= -t)
}
pub fn ftco_800df844(fp: &Fighter) -> bool {
    let t = cd().attackhi3_stick_threshold_y;
    cs(fp, 1).y < t && cs(fp, 0).y >= t
}
pub fn ftco_800df878(fp: &Fighter) -> bool {
    let t = cd().xB0;
    cs(fp, 1).y <= t && cs(fp, 0).y <= t
}
pub fn ftco_800df8b0(fp: &Fighter) -> bool { abs_c(cs(fp, 0).x) >= cd().x31C }
pub fn ftco_800df8e8(fp: &Fighter) -> bool { cs(fp, 0).y <= cd().x314 }
pub fn ftco_800df910(fp: &Fighter) -> bool { cs(fp, 0).y >= cd().tap_jump_threshold }

// ---- ftcoll.c leaves ---------------------------------------------------------------------

pub fn ftcoll_8007aee0(g: G) { fp(g).shield_hit.set_skip_update_pos(0) }
pub fn ftcoll_8007aef8(g: G) { fp(g).reflect_hit.set_skip_update_pos(0) }
pub fn ftcoll_8007af10(g: G) { fp(g).absorb_hit.set_skip_update_pos(0) }
/// `ftColl_8007AF28`: clear `skip_update_pos` on every hurt capsule.
pub fn ftcoll_8007af28(g: G) {
    let f = fp(g);
    let mut i = 0usize;
    while i < f.hurt_capsules_len as usize {
        f.hurt_capsules[i].capsule.set_skip_update_pos(0);
        i += 1;
    }
}
/// `ftColl_8007B8A8(hit, &offset)`.
pub fn ftcoll_8007b8a8(hit: &mut HitCapsule, v: &Vec3) {
    hit.jobj = core::ptr::null_mut();
    hit.b_offset = *v;
}
/// `ftColl_8007B8CC(fp, grabber)`: the thrown hitbox's owner/team come from the grabber.
pub fn ftcoll_8007b8cc(f: &mut Fighter, grabber: G) {
    let gf = fp(grabber);
    let (team, pid) = (gf.team, gf.player_id);
    f.x1064_thrownHitbox.anon_134.owner = grabber;
    f.x119C_teamUnk = team;
    f.grabber_unk1 = pid;
}

// ---- ftdevice.c leaves -------------------------------------------------------------------

/// `ftCo_800C0658`: `x408` if its colanim is set (C: `(int) ptr != 0`), else `x488`.
pub fn ftco_800c0658(f: &mut Fighter) -> *mut ColorOverlay {
    if unsafe { f.x408.x28_colanim.ptr } as usize as u32 != 0 { &mut f.x408 } else { &mut f.x488 }
}
pub fn ftco_800c0674(g: G) -> *mut ColorOverlay { ftco_800c0658(fp(g)) }
pub fn ftco_800c0694(f: &mut Fighter) -> i32 { unsafe { (*ftco_800c0658(f)).x28_colanim.i } }

// ---- ftparts.c leaves --------------------------------------------------------------------

pub fn ftparts_80074a4c(g: G, model_idx: i32, val: i32) {
    let f = fp(g);
    f.x5F4_arr[model_idx as usize].prev = val as i8;
    f.set_x221D_b2(1);
}
pub fn ftparts_80074a74(g: G, model_idx: i32) -> i32 { fp(g).x5F4_arr[model_idx as usize].prev as i32 }
pub fn ftparts_80074a8c(g: G) {
    let f = fp(g);
    let mut i = 0u32;
    while i < f.x5AC.model_num {
        f.x5F4_arr[i as usize].idx = f.x5F4_arr[i as usize].prev;
        i += 1;
    }
    f.set_x221D_b2(0);
}
pub fn ftparts_80074acc(g: G) {
    let f = fp(g);
    let mut i = 0u32;
    while i < f.x5AC.model_num {
        f.x5F4_arr[i as usize].idx = -1;
        i += 1;
    }
    f.set_x221D_b2(0);
}
