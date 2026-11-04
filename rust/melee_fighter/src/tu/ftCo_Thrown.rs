#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Thrown.c (the thrown victim's states, the
//! throw release hit and knockback), with the TARGET_PC paths (held-timing guard `>=`, ScriptGame
//! hit rules, FTKB_IS_KIRBY m-ex clones). Two fused sites, both in ftCo_800DE508 (the XRotN
//! position offsets) -> `fmadds`. Calls out of the TU go through `ext` (unported ones panic until
//! rewired; ported ones default to the port). Checked against the C (whole TU, shipping
//! contraction) by tests/tu_ftCo_Thrown.rs.
use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
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
        ftCo_800DB368(att: &mut Fighter, f: &mut Fighter) = crate::tu::ftCo_CapturePulled::ftCo_800DB368;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftColl_8007B62C(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b62c(g as _, n as _); } };
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        pl_80040614(slot: i32, sub: bool, timer: f32) = crate::plbonus::pl_80040614;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007E358(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e358(a0);
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3) = |j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        ftCommon_8007E3EC(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e3ec(a0);
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = |_, _| todo!("ftCommon_GrabMash: installed by melee_link");
        fn_800DC044(g: G) -> bool = crate::tu::ftCo_CaptureWait::fn_800DC044;
        fn_800DC070(g: G) = crate::tu::ftCo_CaptureJump::fn_800DC070;
        ftCo_CaptureCut_Enter(g: G) = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter;
        ftCamera_800762F4(g: G) = |g: G| { unsafe { crate::tu::ftcamera::ftcamera_800762f4(g as _); } };
        ftCamera_UpdateCameraBox(g: G) = |g: G| { unsafe { crate::tu::ftcamera::ftcamera_update_camera_box(g as _); } };
        ftColl_8007B8CC(f: &mut Fighter, victim: G) = |a0, a1| crate::tu::ftcoll::ftcoll_8007b8cc(unsafe { &mut *(a0 as *mut Fighter) }, a1);
        ftCo_8008DCE0(g: G, angle: i32, facing: f32) = |g: G, angle: i32, facing: f32| { unsafe { crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, angle as _, facing as _); } };
        ftCo_8008E5A4(f: &mut Fighter) = |f: &mut Fighter| { unsafe { crate::tu::ftCo_Damage::ftCo_8008E5A4(f as _); } };
        ftColl_80079C70(f: &mut Fighter, thrower: &mut Fighter, hit: *mut HitCapsule, count: i32) -> f32 = |f: &mut Fighter, thrower: &mut Fighter, hit: *mut HitCapsule, count: i32| -> f32 { unsafe { crate::tu::ftcoll::ftcoll_80079c70(f as *mut _ as *mut _, thrower as *mut _ as *mut _, hit as _, count as _) } };
        ScriptGame_FighterDealtDamage(f: *mut Fighter, damage: f32) -> f32 = |_, _| todo!("TODO(pc script) ScriptGame_FighterDealtDamage");
        ScriptGame_HitRuleContact(hit: *mut HitCapsule, f: *mut Fighter, value: f32, kb: i32) -> f32 = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleContact");
        ScriptGame_HitRuleReport(att: *mut Fighter, vic: *mut Fighter, hit: *mut HitCapsule, damage: f32) = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleReport");
        ScriptGame_HitRulePercentQueue(hit: *mut HitCapsule, f: *mut Fighter, damage: f32) = |_, _, _| todo!("TODO(pc script) ScriptGame_HitRulePercentQueue");
        Fighter_UnkTakeDamage_8006CC30(f: &mut Fighter, damage: f32) = |f: &mut Fighter, damage: f32| { unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(f as _, damage as _); } };
        ftColl_8007891C(g: G, victim: G, damage: f32) = |g: G, victim: G, damage: f32| { unsafe { crate::tu::ftcoll::ftcoll_8007891c(g as _, victim as _, damage as _); } };
    }
}

/// `FTKB_IS_KIRBY(k)` (TARGET_PC: m-ex Kirby clones count).
fn is_kirby(k: u32) -> bool {
    k == Ft_Kind_Kirby || (k >= Ft_Kind_Mex0 && k < Ft_Kind_Max && ext::ftKb_MexKirbyLike(k as i32) != 0)
}

fn part_joint(f: &Fighter, part: Fighter_Part) -> *mut HSD_JObj {
    let i = ext::ftParts_GetBoneIndex(f, part as i32);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// `ftCo_800DE508` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DE508_cb(g: G) {
    melee_core::ct!("ftCo_800DE508_cb");
    ftCo_800DE508(g)
}
/// `fn_800DE798` as the `x21EC` value.
pub unsafe extern "C" fn fn_800DE798_cb(g: G) {
    melee_core::ct!("fn_800DE798_cb");
    fn_800DE798(g)
}

pub fn ftCo_800DE3FC(g: G, msid: i32, anim_speed: f32) {
    melee_core::ct!("ftCo_800DE3FC");
    unsafe {
        let f = fp(g);
        let thrower = fp(f.victim_gobj);
        ext::ftCo_800DB368(thrower, fp(g));
        let f = fp(g);
        f.facing_dir = thrower.facing_dir;
        unsafe { f.mv.co.thrown.unk_bool = 0 };
        ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, anim_speed, 0.0, f.victim_gobj);
        if thrower.kind == Ft_Kind_Yoshi {
            ext::ftColl_8007B62C(g, 2);
        }
        if !is_kirby(thrower.kind) || (msid.wrapping_sub(ftCo_MS_ThrownF as i32) as u32) > 1 {
            let f = fp(g);
            ext::pl_80040614(thrower.player_id as i32, thrower.is_sub_fighter() != 0, f.grab_timer);
            fp(g).grab_timer = 0.0;
        }
        fp(g).accessory1_cb = Some(ftCo_800DE508_cb);
        ext::ftCommon_8007E2F4(fp(g), 0x1FF);
        ext::ftCommon_8007E358(g);
        ext::ftAnim_8006EBA4(g);
    }
}

/// Assigned to `Fighter::accessory1_cb`.
pub fn ftCo_800DE508(g: G) {
    melee_core::ct!("ftCo_800DE508");
    let f = fp(g);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(part_joint(f, FtPart_XRotN), core::ptr::null(), &mut pos);
    let f = fp(g);
    pos.x = fmadds(f.facing_dir, f.x1A70.z * f.x34_scale.y, pos.x);
    pos.y = fmadds(f.x1A70.y, f.x34_scale.y, pos.y);
    pos.z = 0.0;
    f.cur_pos = pos;
}

/// static `ftCo_800DE5A4`.
pub fn ftCo_800DE5A4(g: G) {
    melee_core::ct!("ftCo_800DE5A4");
    unsafe {
        let f = fp(g);
        unsafe {
            if f.mv.co.thrown.unk_bool != 0 {
                if f.mv.co.thrown.anim_timer != 0.0 {
                    // TARGET_PC held-timing guard: reaching or passing the hold frame holds.
                    if f.cur_anim_frame >= f.mv.co.thrown.anim_timer {
                        ext::ftAnim_SetAnimRate(g, 0.0);
                        fp(g).mv.co.thrown.anim_timer = 0.0;
                        ext::ftCommon_8007E3EC(g);
                    }
                }
            } else {
                ext::ftCommon_8007E3EC(g);
            }
        }
        let f = fp(g);
        if f.grab_timer != 0.0 {
            f.grab_timer -= cd().grab_timer_decrement;
            ext::ftCommon_GrabMash(f, cd().x3C8);
            let f = fp(g);
            if f.grab_timer <= 0.0 {
                if unsafe { f.mv.co.thrown.xC } != 0 || ext::fn_800DC044(g) {
                    ext::fn_800DC070(g);
                } else {
                    ext::ftCo_CaptureCut_Enter(g);
                }
            }
        }
    }
}

pub fn ftCo_ThrownF_Anim(g: G) {
    melee_core::ct!("ftCo_ThrownF_Anim");
    ftCo_800DE5A4(g);
}
pub fn ftCo_ThrownF_IASA(_g: G) { melee_core::ct!("ftCo_ThrownF_IASA");}
pub fn ftCo_ThrownF_Phys(_g: G) { melee_core::ct!("ftCo_ThrownF_Phys");}
pub fn ftCo_ThrownF_Coll(_g: G) { melee_core::ct!("ftCo_ThrownF_Coll");}

pub fn ftCo_ThrownB_Anim(g: G) {
    melee_core::ct!("ftCo_ThrownB_Anim");
    ftCo_800DE5A4(g);
}
pub fn ftCo_ThrownB_IASA(_g: G) { melee_core::ct!("ftCo_ThrownB_IASA");}
pub fn ftCo_ThrownB_Phys(_g: G) { melee_core::ct!("ftCo_ThrownB_Phys");}
pub fn ftCo_ThrownB_Coll(_g: G) { melee_core::ct!("ftCo_ThrownB_Coll");}

pub fn ftCo_ThrownHi_Anim(g: G) {
    melee_core::ct!("ftCo_ThrownHi_Anim");
    ftCo_800DE5A4(g);
}
pub fn ftCo_ThrownHi_IASA(_g: G) { melee_core::ct!("ftCo_ThrownHi_IASA");}
pub fn ftCo_ThrownHi_Phys(_g: G) { melee_core::ct!("ftCo_ThrownHi_Phys");}
pub fn ftCo_ThrownHi_Coll(_g: G) { melee_core::ct!("ftCo_ThrownHi_Coll");}

pub fn ftCo_ThrownHi_Cam(g: G) {
    melee_core::ct!("ftCo_ThrownHi_Cam");
    let victim = fp(fp(g).victim_gobj);
    if is_kirby(victim.kind) && victim.motion_id == ftCo_MS_ThrowHi as i32 {
        ext::ftCamera_800762F4(g);
    } else {
        ext::ftCamera_UpdateCameraBox(g);
    }
}

pub fn ftCo_ThrownLw_Anim(g: G) {
    melee_core::ct!("ftCo_ThrownLw_Anim");
    ftCo_800DE5A4(g);
}
pub fn ftCo_ThrownLw_IASA(_g: G) { melee_core::ct!("ftCo_ThrownLw_IASA");}
pub fn ftCo_ThrownLw_Phys(_g: G) { melee_core::ct!("ftCo_ThrownLw_Phys");}
pub fn ftCo_ThrownLw_Coll(_g: G) { melee_core::ct!("ftCo_ThrownLw_Coll");}

pub fn fn_800DE798(g: G) {
    melee_core::ct!("fn_800DE798");
    unsafe {
        let f = fp(g);
        let v = unsafe { f.mv.co.fighterthrow.victim };
        ext::ftColl_8007B8CC(f, v);
    }
}

pub fn ftCo_800DE7C0(g: G, victim: G, is_upward: bool) {
    melee_core::ct!("ftCo_800DE7C0");
    unsafe {
        let f = fp(g);
        if !victim.is_null() {
            f.x21EC = Some(fn_800DE798_cb);
            f.mv.co.fighterthrow.victim = victim;
        }
        // static inline calcKnockbackAngle / calcFacingDir
        let kb_angle = if is_upward { 90 } else { -1 };
        let a = f.dmg.x1848_kb_angle;
        let facing = if a > 90 && a < 270 { -f.dmg.facing_dir_1 } else { 0.0 };
        ext::ftCo_8008DCE0(g, kb_angle, facing);
        ext::ftCo_8008E5A4(fp(g));
    }
}

pub fn ftCo_800DE854(g: G) {
    melee_core::ct!("ftCo_800DE854");
    let f = fp(g);
    let thrower_gobj = f.victim_gobj;
    let thrower: *mut Fighter = fp(thrower_gobj);
    let hit: *mut HitCapsule = unsafe { &mut (*thrower).xDF4[1] };
    let mut collpos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(part_joint(f, FtPart_XRotN), core::ptr::null(), &mut collpos);
    let fpp: *mut Fighter = fp(g);
    unsafe {
        let f = &mut *fpp;
        f.dmg.kb_applied = ext::ftColl_80079C70(f, &mut *thrower, hit, (*hit).unk_count as i32);
        f.dmg.x1848_kb_angle = (*hit).kb_angle;
        f.dmg.facing_dir_1 = -(*thrower).facing_dir;
        f.dmg.x184c_damaged_hurtbox = 1;
        f.dmg.x1854_collpos = collpos;
        f.dmg.x1860_element = (*hit).element;
        let c = |h: *mut HitCapsule| {
            let d = ext::ScriptGame_FighterDealtDamage(thrower, (*h).damage);
            ext::ScriptGame_HitRuleContact(h, fpp, d, 0)
        };
        let v = c(hit);
        ext::ScriptGame_HitRuleReport(thrower, fpp, hit, v);
        let v = c(hit);
        ext::ScriptGame_HitRulePercentQueue(hit, fpp, v);
        let v = c(hit);
        ext::Fighter_UnkTakeDamage_8006CC30(&mut *fpp, v);
        ext::ftColl_8007891C(thrower_gobj, g, (*hit).damage);
    }
}

pub fn ftCo_800DE920(g: G, anim_timer: f32) {
    melee_core::ct!("ftCo_800DE920");
    unsafe {
        let f = fp(g);
        f.mv.co.thrown.unk_bool = 1;
        // TARGET_PC held-timing guard (see ftCo_800DE5A4).
        if f.cur_anim_frame >= anim_timer {
            ext::ftAnim_SetAnimRate(g, 0.0);
            fp(g).mv.co.thrown.anim_timer = 0.0;
        } else {
            f.mv.co.thrown.anim_timer = anim_timer;
        }
    }
}

pub fn ftCo_800DE974(g: G) {
    melee_core::ct!("ftCo_800DE974");
    unsafe {
        fp(g).mv.co.thrown.unk_bool = 0;
        ext::ftAnim_SetAnimRate(g, 1.0);
        ext::ftAnim_8006EBA4(g);
    }
}
