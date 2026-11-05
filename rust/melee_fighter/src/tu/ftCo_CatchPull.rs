#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CatchPull.c (grab connects, pull the victim
//! in). Every function of the TU, TARGET_PC path (Geno own pull-in clip); calls out go through
//! `ext`. Checked against the C by tests/tu_ftCo_CatchPull.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[inline(always)]
fn item<'a>(g: *mut Item_GObj) -> &'a mut Item {
    unsafe { &mut *((*g).user_data as *mut Item) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        it_802A7840(g: *mut Item_GObj) = |g: *mut Item_GObj| { unsafe { melee_items::tu::itlinkhookshot::it_802A7840(g as _); } };
        it_802BAA94(g: *mut Item_GObj) = |g: *mut Item_GObj| { unsafe { melee_items::tu::itsamusgrapple::it_802BAA94(g as _); } };
        // TARGET_PC: a Geno fighter's own clip for this motion (row, or < 0).
        Geno_MotionAnimRow(fp: &mut Fighter, motion: i32) -> i32 = |_, _| todo!("TODO(port) Geno_MotionAnimRow (pc/geno)");
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        fn_800DA1D8(g: G) = crate::tu::ftCo_CatchWait::fn_800DA1D8;
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCommon_CalcGroundAccel_Deaccel(fp: &mut Fighter, friction: f32) = crate::ftcommon::calc_ground_accel_deaccel;
        ftCommon_SetSelfMovementFromGroundedMovement(g: G) = |a0| crate::tu::ftcommon::ftCommon_SetSelfMovementFromGroundedMovement_fp(a0);
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ftCo_800DC920(g: G, victim: G) = |g: G, victim: G| { unsafe { crate::tu::ftCo_CaptureCut::ftCo_800DC920(g as _, victim as _); } };
        ftCo_Fall_Enter(g: G) = crate::hooks::ftco_fall_enter;
        lb_8000B1CC(jobj: *mut HSD_JObj, offset: *const Vec3, out: &mut Vec3) = |jobj: *mut HSD_JObj, offset: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(jobj as _, offset as _, out as _); } };
        ftParts_GetBoneIndex(fp: &mut Fighter, part: u32) -> u32 = |_, _| todo!("ftParts_GetBoneIndex: installed by melee_link");
        ftCo_800DA698(g: G, arg1: bool) = crate::tu::ftCo_CatchCut::ftCo_800DA698;
    }
}

/// `fn_800DA190` as the C `HSD_GObjEvent` stored in `accessory1_cb`.
pub unsafe extern "C" fn fn_800DA190_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA190_cb"); fn_800DA190(g) }
/// `fn_800DA004` as the C `HSD_GObjEvent` handed to `ft_800841B8`.
pub unsafe extern "C" fn fn_800DA004_cb(g: *mut HSD_GObj) { melee_core::ct!("fn_800DA004_cb"); fn_800DA004(g) }

/// `fn_800D9CE8`: the grab connected; enter CatchPull (0xD5) or CatchDashPull (0xD7).
pub fn fn_800D9CE8(g: G) {
    melee_core::ct!("fn_800D9CE8");
    let f = fp(g);
    f.gr_vel = 0.0;
    let frame = f.cur_anim_frame;
    let mut start_frame = frame;
    let next_motion;
    if f.motion_id == 0xD4 {
        next_motion = 0xD5;
        if f.kind == Ft_Kind_Yoshi {
            let ya = unsafe { &*(f.dat_attrs as *const ftYoshiAttributes) };
            if frame >= ya.x124 && frame < ya.x128 {
                let rate = frame - ya.x124;
                ext::ftAnim_SetAnimRate(g, rate);
                ext::ftAnim_8006EBA4(g);
                start_frame = ya.x12C[melee_core::ppc::fctiwz_f32(rate) as usize] as f32;
            }
        }
    } else {
        next_motion = 0xD7;
    }

    let f = fp(g);
    match f.kind {
        Ft_Kind_CLink | Ft_Kind_Link => {
            let it = unsafe { f.u.lk.xC };
            ext::it_802A7840(it);
            let j = unsafe { item(it).xDD4_itemVar.linkhookshot.xC };
            fp(g).mv.co.capturedamage.x18 = j;
        }
        Ft_Kind_Samus => {
            let it = unsafe { f.u.ss.x223C };
            ext::it_802BAA94(it);
            let j = unsafe { item(it).xDD4_itemVar.samusgrapple.xC };
            fp(g).mv.co.capturedamage.x18 = j;
        }
        _ => unsafe {
            let j = (*f.parts.offset((*(*f.ft_data).x8).x11 as isize)).joint;
            f.mv.co.capturedamage.x18 = j;
        },
    }

    let f = fp(g);
    f.anon_2210.throw_flags = 0;
    // TARGET_PC: a fighter with a pull-in clip of its own plays it from its first frame.
    if ext::Geno_MotionAnimRow(f, next_motion) >= 0 {
        ext::Fighter_ChangeMotionState(g, next_motion, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        let f = fp(g);
        f.accessory1_cb = Some(fn_800DA190_cb);
        f.anon_221b.set_x221B_b7(0);
        return;
    }
    ext::Fighter_ChangeMotionState(g, next_motion, 0x4000, start_frame, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.accessory1_cb = Some(fn_800DA190_cb);
    f.anon_221b.set_x221B_b7(0);
}

pub fn ftCo_CatchPull_Anim(g: G) {
    melee_core::ct!("ftCo_CatchPull_Anim");
    let f = fp(g);
    let mut should_transition = false;
    match f.kind {
        Ft_Kind_Link | Ft_Kind_CLink => {
            let it = unsafe { f.u.lk.xC };
            if it.is_null() || unsafe { item(it).xDD4_itemVar.linkhookshot.x14 } != 0 {
                should_transition = true;
            }
        }
        Ft_Kind_Samus => {
            let it = unsafe { f.u.ss.x223C };
            if it.is_null() || unsafe { item(it).xDD4_itemVar.samusgrapple.x14 } != 0 {
                should_transition = true;
            }
        }
        _ => {
            let mut go = true;
            if ext::ftAnim_IsFramesRemaining(g) {
                let f = fp(g);
                let result = if unsafe { f.anon_2210.anon_0.throw_flags_b3() } != 0 {
                    unsafe { f.anon_2210.anon_0.set_throw_flags_b3(0) };
                    true
                } else {
                    false
                };
                if !result {
                    go = false;
                }
            }
            if go {
                should_transition = true;
            }
        }
    }
    if should_transition {
        ext::fn_800DA1D8(g);
    }
}

pub fn ftCo_CatchPull_IASA(_g: G) { melee_core::ct!("ftCo_CatchPull_IASA");}

pub fn ftCo_CatchPull_Phys(g: G) {
    melee_core::ct!("ftCo_CatchPull_Phys");
    let f = fp(g);
    let fr = crate::hooks::common_data().x64 * f.co_attrs.ground_friction;
    ext::ftCommon_CalcGroundAccel_Deaccel(f, fr);
    ext::ftCommon_SetSelfMovementFromGroundedMovement(g);
}

pub fn ftCo_CatchPull_Coll(g: G) {
    melee_core::ct!("ftCo_CatchPull_Coll");
    ext::ft_800841B8(g, Some(fn_800DA004_cb));
}

pub fn fn_800DA004(g: G) {
    melee_core::ct!("fn_800DA004");
    let victim = fp(g).victim_gobj;
    ext::ftCo_800DC920(g, victim);
    ext::ftCo_Fall_Enter(g);
    ext::ftCo_Fall_Enter(victim);
}

/// `fn_800DA054`: while holding, release when the victim drifts out of reach, else walk toward it.
pub fn fn_800DA054(g: G) {
    melee_core::ct!("fn_800DA054");
    let f = fp(g);
    let victim = unsafe { &mut *((*f.victim_gobj).user_data as *mut Fighter) };
    if victim.x2226_b2() != 0 {
        return;
    }
    let mut self_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut victim_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let j = unsafe { f.mv.co.capturedamage.x18 };
    ext::lb_8000B1CC(j, core::ptr::null(), &mut self_pos);
    let bone = ext::ftParts_GetBoneIndex(victim, FtPart_XRotN);
    let vj = unsafe { (*victim.parts.offset(bone as isize)).joint };
    ext::lb_8000B1CC(vj, core::ptr::null(), &mut victim_pos);
    let f = fp(g);
    let cd = crate::hooks::common_data();
    let facing = f.facing_dir;
    let ddy = victim_pos.y - self_pos.y;
    let dx = victim_pos.x - self_pos.x;
    let mut dy = ddy + f.x2170;
    let release = if dx * facing > cd.x34C {
        true
    } else {
        if dy < 0.0 {
            dy = -dy;
        }
        dy > cd.x350
    };
    if release {
        ext::ftCo_800DA698(g, true);
    } else if dx * facing < 0.0 {
        let mut v = if dx < 0.0 { -dx } else { dx };
        let spd = f.co_attrs.walk_max_vel;
        if v > spd {
            v = spd;
        }
        f.gr_vel = if dx > 0.0 { v } else { -v };
    }
}

pub fn fn_800DA190(g: G) {
    melee_core::ct!("fn_800DA190");
    match fp(g).kind {
        6 | 13 | 20 => {}
        _ => fn_800DA054(g),
    }
}
