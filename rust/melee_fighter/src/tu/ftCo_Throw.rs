#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Throw.c (the thrower's states, throw
//! selection, the release hit ftCo_800DDDE4 and the throw-out hit ftCo_800DE2F0), with the
//! TARGET_PC paths (release fast-forward guard, ScriptGame hit rules / mod values,
//! FTKB_IS_KIRBY / FTKB_CANON_KIND m-ex clones, Geno template kinds). Two fused sites, both in
//! ftCo_800DDDE4 (the XRotN position offsets) -> `fmadds`. Calls out of the TU go through `ext`
//! (unported ones panic until rewired; ported ones default to the port). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_Throw.rs.
use crate::types::*;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_types::inline as ji;

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
        ftCo_800DC920(a: G, b: G) = crate::tu::ftCo_CaptureCut::ftCo_800DC920;
        ftCo_800DCFD4(g: G) = crate::tu::ftCo_CaptureCut::ftCo_800DCFD4;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        ftCo_800DF7F4(f: &Fighter) -> bool = crate::ft0df::ftco_800df7f4;
        ftCo_800DF844(f: &Fighter) -> bool = crate::ft0df::ftco_800df844;
        ftCo_800DF878(f: &Fighter) -> bool = crate::ft0df::ftco_800df878;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCo_800DE3FC(g: G, msid: i32, speed: f32) = crate::tu::ftCo_Thrown::ftCo_800DE3FC;
        ftKb_MexKirbyLike(kind: i32) -> i32 = |_| todo!("TODO(port) ftKb_MexKirbyLike (ftkirby.c)");
        ftSs_Init_CreateThrowGrappleBeam(g: G, msid: i32, speed: f32) = |_, _, _| todo!("TODO(port) ftSs_Init_CreateThrowGrappleBeam (ftsamus)");
        ftColl_8007B7A4(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b7a4(g as _, n as _); } };
        Geno_DefineTemplateSkeleton(kind: i32) -> i32 = crate::hooks::geno_define_template_skeleton;
        Geno_DefineBaseKind(kind: i32) -> i32 = |kind: i32| -> i32 { unsafe { crate::platform::Geno_DefineBaseKind(kind) } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        ftCo_800DE974(g: G) = crate::tu::ftCo_Thrown::ftCo_800DE974;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        OSReport(msg: &str) = |_| {};
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
        mpLineGetFlags(line_id: i32) -> u32 = |line_id: i32| -> u32 { unsafe { melee_world::tu::mplib::mpLineGetFlags(line_id as _) } };
        pl_80040614(slot: i32, sub: bool, timer: f32) = crate::plbonus::pl_80040614;
        ftCo_800DE7C0(g: G, victim: G, up: bool) = crate::tu::ftCo_Thrown::ftCo_800DE7C0;
        ftCo_800DE920(g: G, frame: f32) = crate::tu::ftCo_Thrown::ftCo_800DE920;
        ftMt_SpecialN_Shoot(g: G) = |_| todo!("TODO(port) ftMt_SpecialN_Shoot (ftmewtwospecialn.c)");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_8009B56C(g: G) = crate::tu::ftCo_CargoWait::ftCo_8009B56C;
        ft_80085134(g: G) = crate::ft084e::ft_80085134;
        ft_80085004(g: G) = crate::ft084e::ft_80085004;
        ft_80083C00(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80083c00(g as _, ev)); } };
        ft_80083CE4(g: G, line_cb: unsafe extern "C" fn(G, i32) -> bool, cb: HSD_GObjEvent) = |g: G, line_cb: unsafe extern "C" fn(G, i32) -> bool, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_coll_ev(line_cb, cb, |a, b| crate::tu::ft_081b::ft_80083ce4(g as _, a, b)); } };
        ft_80083B68(g: G) = |g: G| { unsafe { crate::tu::ft_081b::ft_80083b68(g as _); } };
        ft_800841B8(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_800841b8(g as _, ev)); } };
        ftFx_Throw_Anim(g: G) = |_| todo!("TODO(port) ftFx_Throw_Anim (ftfoxspecialn.c)");
        ftCamera_800762F4(g: G) = |g: G| { unsafe { crate::tu::ftcamera::ftcamera_800762f4(g as _); } };
        ftCamera_UpdateCameraBox(g: G) = |g: G| { unsafe { crate::tu::ftcamera::ftcamera_update_camera_box(g as _); } };
        ftColl_8007891C(g: G, victim: G, damage: f32) = |g: G, victim: G, damage: f32| { unsafe { crate::tu::ftcoll::ftcoll_8007891c(g as _, victim as _, damage as _); } };
        ftColl_8007B62C(g: G, n: i32) = |g: G, n: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b62c(g as _, n as _); } };
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3) = |j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3| { unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } };
        ftColl_8007B868(g: G) -> i32 = |g: G| -> i32 { unsafe { crate::tu::ftcoll::ftcoll_8007b868(g as _) } };
        ScriptGame_FighterDealtDamage(f: *mut Fighter, damage: f32) -> f32 = |_, _| todo!("TODO(pc script) ScriptGame_FighterDealtDamage");
        ScriptGame_HitRuleContact(hit: *mut HitCapsule, f: *mut Fighter, value: f32, kb: i32) -> f32 = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleContact");
        ftColl_80076640(f: *mut Fighter, damage: &mut f32) -> bool = |f: *mut Fighter, damage: &mut f32| -> bool { unsafe { crate::tu::ftcoll::ftcoll_80076640(&mut *(f as *mut _), damage) } };
        ScriptGame_HitRulePercentQueue(hit: *mut HitCapsule, f: *mut Fighter, damage: f32) = |_, _, _| todo!("TODO(pc script) ScriptGame_HitRulePercentQueue");
        gm_8016B248() -> f32 = || todo!("TODO(port) gm_8016B248 (gm_unsplit.c)");
        Player_GetAttackRatio(slot: i32) -> f32 = |slot: i32| -> f32 { unsafe { crate::tu::player::Player_GetAttackRatio(slot as _) as _ } };
        Player_GetDefenseRatio(slot: i32) -> f32 = |slot: i32| -> f32 { unsafe { crate::tu::player::Player_GetDefenseRatio(slot as _) as _ } };
        ftColl_80079AB0(f: *mut Fighter, hit: *mut HitCapsule, count: u32, a: f32, b: f32, c: f32, d: f32) -> f32 = |f: *mut Fighter, hit: *mut HitCapsule, count: u32, a: f32, b: f32, c: f32, d: f32| -> f32 { unsafe { crate::tu::ftcoll::ftcoll_80079ab0(&*(f as *const _), &*(hit as *const _), count as _, a as _, b as _, c as _, d as _) } };
        ScriptGame_HitRuleWon(hit: *mut HitCapsule, f: *mut Fighter) = |_, _| todo!("TODO(pc script) ScriptGame_HitRuleWon");
        ScriptGame_HitRuleReport(att: *mut Fighter, vic: *mut Fighter, hit: *mut HitCapsule, damage: f32) = |_, _, _, _| todo!("TODO(pc script) ScriptGame_HitRuleReport");
        ftColl_80078710(g: G, victim: G, facing: *mut f32) = |g: G, victim: G, facing: *mut f32| { unsafe { crate::tu::ftcoll::ftcoll_80078710(g as _, victim as _, facing as _); } };
        ftCommon_8007EE0C(f: &mut Fighter, v: i32) = |_, _| todo!("ftCommon_8007EE0C: installed by melee_link");
        ftCo_Damage_CalcKnockback(f: &mut Fighter) = |f: &mut Fighter| { unsafe { crate::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(f as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        lb_8000C390(j: *mut HSD_JObj) = |j: *mut HSD_JObj| { unsafe { melee_runtime::tu::lb_00B0::lb_8000C390(j as _); } };
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (jobj.c)");
        // rp3 mpColl_80043670
        mpColl_80043670(coll: &mut CollData) = |coll: &mut CollData| unsafe { melee_world::tu::mpcoll::mpColl_80043670(coll as _); };
        // rp3-end
        ftCommon_UnlockECB(f: &mut Fighter) = crate::ftcommon2::unlock_ecb;
        mpColl_800471F8(coll: &mut CollData) -> bool = |coll: &mut CollData| -> bool { unsafe { (melee_world::tu::mpcoll::mpColl_800471F8(coll as *mut _ as *mut _) != 0 as _) } };
        lbColl_80008D30(hit: &mut HitCapsule, arg: *const lbColl_80008D30_arg1) = |hit: &mut HitCapsule, arg: *const lbColl_80008D30_arg1| { unsafe { melee_runtime::tu::lbcollision::lbColl_80008D30(hit as _, arg as _); } };
        ftColl_800788D4(g: G) = |g: G| { unsafe { crate::tu::ftcoll::ftcoll_800788d4(g as _); } };
        ScriptGame_ModValue(slot: i32, field: i32, base: f32) -> f32 = |_, _, _| todo!("TODO(pc script) ScriptGame_ModValue");
        Fighter_UnkTakeDamage_8006CC30(f: &mut Fighter, damage: f32) = |f: &mut Fighter, damage: f32| { unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(f as _, damage as _); } };
        ftCo_8008E908(g: G, v: f32) = |g: G, v: f32| { unsafe { crate::tu::ftCo_Damage::ftCo_8008E908(g as _, v as _); } };
    }
}

/// The jobj.h inline dirty marks.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}
/// `HSD_JObjSetTranslate` (jobj.h inline).
fn set_translate(j: *mut HSD_JObj, t: &Vec3) {
    assert!(!j.is_null(), "jobj.h: jobj");
    ji::HSD_JObjSetTranslate(unsafe { &mut *j }, t, &mut Dirty);
}

/// `FTKB_IS_KIRBY(k)` (TARGET_PC: m-ex Kirby clones count).
fn is_kirby(k: u32) -> bool {
    k == Ft_Kind_Kirby || (k >= Ft_Kind_Mex0 && k < Ft_Kind_Max && ext::ftKb_MexKirbyLike(k as i32) != 0)
}

/// `ftGetTemplateKind` (ft/inlines.h, TARGET_PC).
fn template_kind(f: &Fighter) -> i32 {
    if f.kind >= Ft_Kind_Mex0 && ext::Geno_DefineTemplateSkeleton(f.kind as i32) != 0 {
        return ext::Geno_DefineBaseKind(f.kind as i32);
    }
    f.kind as i32
}

fn part_joint(f: &Fighter, part: Fighter_Part) -> *mut HSD_JObj {
    let i = ext::ftParts_GetBoneIndex(f, part as i32);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

/// TU-local callbacks as values.
pub unsafe extern "C" fn fn_800DD568_cb(g: G) {
    melee_core::ct!("fn_800DD568_cb");
    fn_800DD568(g)
}
pub unsafe extern "C" fn fn_800DD5EC_cb(g: G) {
    melee_core::ct!("fn_800DD5EC_cb");
    fn_800DD5EC(g)
}
pub unsafe extern "C" fn fn_800DD684_cb(g: G) {
    melee_core::ct!("fn_800DD684_cb");
    fn_800DD684(g)
}
pub unsafe extern "C" fn fn_800DD6E4_cb(g: G, arg: i32) -> bool {
    melee_core::ct!("fn_800DD6E4_cb");
    fn_800DD6E4(g, arg)
}

pub fn ftCo_800DD100(g: G) {
    melee_core::ct!("ftCo_800DD100");
    let f = fp(g);
    let victim = f.victim_gobj;
    if !victim.is_null() {
        if f.anon_221b.x221B_b5() != 0 {
            ext::ftCo_800DC920(g, victim);
        } else {
            ext::ftCo_800DC920(victim, g);
        }
        ext::ftCommon_8007D92C(victim);
    }
}

pub fn ftCo_800DD168(g: G) {
    melee_core::ct!("ftCo_800DD168");
    let f = fp(g);
    let victim = f.victim_gobj;
    if !victim.is_null() {
        if f.anon_221b.x221B_b5() != 0 {
            ext::ftCo_800DCFD4(victim);
            ext::ftCo_800DC920(g, victim);
        } else {
            ext::ftCo_800DC920(victim, g);
            ext::ftCommon_8007D92C(victim);
        }
    }
}

pub fn ftCo_800DD1E4(g: G) -> bool {
    melee_core::ct!("ftCo_800DD1E4");
    let f = fp(g);
    let mut msid = f.motion_id;
    let i1 = {
        let t = cd().x98;
        (f.input.lstick[1].x < t && f.input.lstick[0].x >= t) || (f.input.lstick[1].x > -t && f.input.lstick[0].x <= -t)
    };
    let i2 = |f: &Fighter| {
        let t = cd().attackhi3_stick_threshold_y;
        f.input.lstick[1].y < t && f.input.lstick[0].y >= t
    };
    let i3 = |f: &Fighter| {
        let t = cd().xB0;
        f.input.lstick[1].y > t && f.input.lstick[0].y <= t
    };
    if i1 {
        msid = if f.input.lstick[0].x * f.facing_dir > 0.0 { 219 } else { 220 };
    } else if ext::ftCo_800DF7F4(f) {
        msid = if f.input.cstick[0].x * f.facing_dir > 0.0 { 219 } else { 220 };
    } else if i2(f) || ext::ftCo_800DF844(f) {
        msid = 221;
    } else if i3(f) || ext::ftCo_800DF878(f) {
        msid = 222;
    }
    if msid != fp(g).motion_id {
        ftCo_800DD4B0(g, msid);
        return true;
    }
    false
}

pub fn ftCo_800DD398(g: G, msid: i32, victim_msid: i32, anim_speed: f32) {
    melee_core::ct!("ftCo_800DD398");
    unsafe {
        let f = fp(g);
        f.cmd_vars[0] = 0;
        f.anon_2210.throw_flags = 0;
        f.mv.co.fighterthrow.x4 = 0;
        f.mv.co.fighterthrow.x8 = 0;
        ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, anim_speed, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006EBA4(g);
        ext::ftCommon_8007E2F4(fp(g), 0x1FF);
        ext::ftCo_800DE3FC(fp(g).victim_gobj, victim_msid, anim_speed);
        let f = fp(g);
        let canon = if is_kirby(f.kind) { Ft_Kind_Kirby } else { f.kind };
        if canon == Ft_Kind_Kirby {
            if msid == 221 {
                let victim = fp(f.victim_gobj);
                victim.set_x2219_b2(1);
                fp(g).set_x2219_b2(1);
            }
        } else if canon == Ft_Kind_Samus {
            ext::ftSs_Init_CreateThrowGrappleBeam(g, msid, anim_speed);
        }
        ext::ftColl_8007B7A4(g, cd().x348);
    }
}

pub fn ftCo_800DD4B0(g: G, msid: i32) {
    melee_core::ct!("ftCo_800DD4B0");
    let f = fp(g);
    let victim = fp(f.victim_gobj);
    let throw_index = msid.wrapping_sub(219);
    let mut victim_msid = throw_index.wrapping_add(239);
    let anim_speed = unsafe {
        if (*(*f.ft_data).x0).weight_independent_throws_mask as i32 & melee_runtime::cabi::shl(1, throw_index as u32) as i32 != 0 {
            1.0
        } else {
            1.0 / ((*(*victim.ft_data).x0).weight * cd().x37C)
        }
    };
    if msid == 222 && (f.kind == Ft_Kind_GKoops || f.kind == Ft_Kind_Koopa) {
        if victim.kind == Ft_Kind_Peach || template_kind(victim) == Ft_Kind_Zelda as i32 {
            victim_msid = 243;
        }
    }
    ftCo_800DD398(g, msid, victim_msid, anim_speed);
}

pub fn fn_800DD568(g: G) {
    melee_core::ct!("fn_800DD568");
    unsafe {
        ext::ftAnim_SetAnimRate(g, 1.0);
        ext::ftAnim_8006EBA4(g);
        if let Some(cb) = fp(g).anim_cb {
            unsafe { cb(g) };
        }
        let f = fp(g);
        f.mv.co.fighterthrow.x4 = 0;
        f.mv.co.fighterthrow.x8 = 1;
        if !f.victim_gobj.is_null() {
            ext::ftCo_800DE974(f.victim_gobj);
        }
        ext::ftCommon_8007D7FC(fp(g));
    }
}

pub fn fn_800DD5EC(g: G) {
    melee_core::ct!("fn_800DD5EC");
    unsafe {
        // TARGET_PC: fast-forward to the release, at most the longest throw (600 frames).
        let mut guard = 0;
        loop {
            if fp(g).cmd_vars[0] != 0 {
                break;
            }
            let gv = guard;
            guard += 1;
            if gv >= 600 {
                break;
            }
            ext::ftAnim_8006EBA4(g);
        }
        if guard >= 600 {
            let f = fp(g);
            ext::OSReport(&format!(
                "throw: P{} kind {} motion {} never set cmd_var 0 (the release); landing without it\n",
                f.player_id as i32 + 1,
                f.kind as i32,
                f.motion_id
            ));
        }
        if let Some(cb) = fp(g).anim_cb {
            unsafe { cb(g) };
        }
        ext::ftAnim_SetAnimRate(g, 1.0);
        let f = fp(g);
        f.mv.co.fighterthrow.x4 = 0;
        f.mv.co.fighterthrow.x8 = 1;
        if !f.victim_gobj.is_null() {
            ext::ftCo_800DE974(f.victim_gobj);
        }
        ext::ftCommon_8007D7FC(fp(g));
    }
}

pub fn fn_800DD684(g: G) {
    melee_core::ct!("fn_800DD684");
    let victim = fp(g).victim_gobj;
    if !victim.is_null() {
        ext::ftCo_800DC920(g, victim);
        ext::ftCo_Fall_Enter(victim);
    }
    ext::ftCo_Fall_Enter(g);
}

pub fn fn_800DD6E4(_g: G, arg: i32) -> bool {
    melee_core::ct!("fn_800DD6E4");
    arg != -1 && ext::mpLineGetFlags(arg) & LINE_FLAG_PLATFORM as u32 == 0
}

pub fn ftCo_800DD724(g: G) {
    melee_core::ct!("ftCo_800DD724");
    unsafe {
        let f = fp(g);
        let b4 = unsafe {
            if f.anon_2210.anon_0.throw_flags_b4() != 0 {
                f.anon_2210.anon_0.set_throw_flags_b4(0);
                true
            } else {
                false
            }
        };
        if b4 {
            f.facing_dir = -f.facing_dir;
        }
        let b3 = unsafe {
            if f.anon_2210.anon_0.throw_flags_b3() != 0 {
                f.anon_2210.anon_0.set_throw_flags_b3(0);
                true
            } else {
                false
            }
        };
        if b3 {
            let victim = f.victim_gobj;
            ext::ftCommon_8007E2F4(f, 0);
            if !victim.is_null() {
                let f = fp(g);
                ext::pl_80040614(f.player_id as i32, f.is_sub_fighter() != 0, fp(victim).grab_timer);
                ftCo_800DE2A8(g, victim);
                let up = fp(g).motion_id == 222;
                ext::ftCo_800DE7C0(victim, g, up);
            }
        }
        let f = fp(g);
        if f.cmd_vars[0] != 0 && f.mv.co.fighterthrow.x4 == 0 {
            f.mv.co.fighterthrow.x4 = 1;
            f.cmd_vars[0] = 0;
            f.mv.co.throw_.xC = f.x6A4_transNOffset;
            // throw_.xC.y / .z are fighterthrow.self_vel_y / self_vel_x at their GC offsets (0x2350 / 0x2354); the native layouts differ
            f.mv.co.fighterthrow.self_vel_y = f.x6A4_transNOffset.y;
            f.mv.co.fighterthrow.self_vel_x = f.x6A4_transNOffset.z;
            ext::ftAnim_SetAnimRate(g, 0.0);
            let f = fp(g);
            if !f.victim_gobj.is_null() {
                ext::ftCo_800DE920(f.victim_gobj, f.cur_anim_frame);
            }
            let f = fp(g);
            if is_kirby(f.kind) && f.motion_id == 221 {
                f.set_x2219_b2(0);
                if !f.victim_gobj.is_null() {
                    fp(f.victim_gobj).set_x2219_b2(0);
                }
            }
        }
    }
}

pub fn ftCo_ThrowF_Anim(g: G) {
    melee_core::ct!("ftCo_ThrowF_Anim");
    ext::ftMt_SpecialN_Shoot(g);
    ftCo_800DD724(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        if fp(g).x2222_b0() != 0 {
            ext::ftCo_8009B56C(g);
        } else {
            ext::ftCommon_8007D92C(g);
        }
    }
}

pub fn ftCo_ThrowF_IASA(_g: G) { melee_core::ct!("ftCo_ThrowF_IASA");}

pub fn ftCo_ThrowF_Phys(g: G) {
    melee_core::ct!("ftCo_ThrowF_Phys");
    unsafe {
        let f = fp(g);
        if f.ground_or_air == GA_Air {
            if f.mv.co.fighterthrow.x4 != 0 {
                f.self_vel.x = f.mv.co.throw_.xC.z * f.facing_dir;
                f.self_vel.y = f.mv.co.throw_.xC.y;
            } else {
                ext::ft_80085134(g);
            }
        } else {
            ext::ft_80085004(g);
        }
    }
}

/// The shared `ftCo_Throw{F,B,Hi,Lw}_Coll` body.
fn throw_coll(g: G) {
    unsafe {
        let f = fp(g);
        if f.ground_or_air == GA_Air {
            if f.mv.co.fighterthrow.x4 != 0 {
                ext::ft_80083C00(g, Some(fn_800DD568_cb));
            } else if f.mv.co.fighterthrow.x8 == 0 {
                ext::ft_80083CE4(g, fn_800DD6E4_cb, Some(fn_800DD5EC_cb));
            } else {
                ext::ft_80083B68(g);
            }
        } else {
            ext::ft_800841B8(g, Some(fn_800DD684_cb));
        }
    }
}

/// The shared `ftCo_Throw{B,Hi,Lw}_Phys` body.
fn throw_phys(g: G) {
    unsafe {
        let f = fp(g);
        if f.ground_or_air == GA_Air {
            if f.mv.co.fighterthrow.x4 != 0 {
                f.self_vel.x = f.mv.co.fighterthrow.self_vel_x * f.facing_dir;
                f.self_vel.y = f.mv.co.fighterthrow.self_vel_y;
            } else {
                ext::ft_80085134(g);
            }
        } else {
            ext::ft_80085004(g);
        }
    }
}

/// The shared `ftCo_Throw{B,Hi,Lw}_Anim` body.
fn throw_anim(g: G) {
    ext::ftFx_Throw_Anim(g);
    ftCo_800DD724(g);
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCommon_8007D92C(g);
    }
}

pub fn ftCo_ThrowF_Coll(g: G) {
    melee_core::ct!("ftCo_ThrowF_Coll");
    throw_coll(g);
}

pub fn ftCo_ThrowB_Anim(g: G) {
    melee_core::ct!("ftCo_ThrowB_Anim");
    throw_anim(g);
}
pub fn ftCo_ThrowB_IASA(_g: G) { melee_core::ct!("ftCo_ThrowB_IASA");}
pub fn ftCo_ThrowB_Phys(g: G) {
    melee_core::ct!("ftCo_ThrowB_Phys");
    throw_phys(g);
}
pub fn ftCo_ThrowB_Coll(g: G) {
    melee_core::ct!("ftCo_ThrowB_Coll");
    throw_coll(g);
}

pub fn ftCo_ThrowHi_Anim(g: G) {
    melee_core::ct!("ftCo_ThrowHi_Anim");
    throw_anim(g);
}
pub fn ftCo_ThrowHi_IASA(_g: G) { melee_core::ct!("ftCo_ThrowHi_IASA");}
pub fn ftCo_ThrowHi_Phys(g: G) {
    melee_core::ct!("ftCo_ThrowHi_Phys");
    throw_phys(g);
}
pub fn ftCo_ThrowHi_Coll(g: G) {
    melee_core::ct!("ftCo_ThrowHi_Coll");
    throw_coll(g);
}

pub fn ftCo_ThrowHi_Cam(g: G) {
    melee_core::ct!("ftCo_ThrowHi_Cam");
    let f = fp(g);
    if is_kirby(f.kind) && f.motion_id == 221 {
        ext::ftCamera_800762F4(g);
    } else {
        ext::ftCamera_UpdateCameraBox(g);
    }
}

pub fn ftCo_ThrowLw_Anim(g: G) {
    melee_core::ct!("ftCo_ThrowLw_Anim");
    throw_anim(g);
}
pub fn ftCo_ThrowLw_IASA(_g: G) { melee_core::ct!("ftCo_ThrowLw_IASA");}
pub fn ftCo_ThrowLw_Phys(g: G) {
    melee_core::ct!("ftCo_ThrowLw_Phys");
    throw_phys(g);
}
pub fn ftCo_ThrowLw_Coll(g: G) {
    melee_core::ct!("ftCo_ThrowLw_Coll");
    throw_coll(g);
}

pub fn ftCo_800DDDE4(g: G, g2: G, arg: bool) {
    melee_core::ct!("ftCo_800DDDE4");
    let fpp: *mut Fighter = fp(g);
    let fp2p: *mut Fighter = fp(g2);
    unsafe {
        let f = &mut *fpp;
        let hit: *mut HitCapsule = f.xDF4.as_mut_ptr();
        let (fp3, fp4): (*mut Fighter, *mut Fighter) =
            if f.anon_221b.x221B_b7() != 0 { (fp2p, fpp) } else { (fpp, fp2p) };
        f.x1A5C = core::ptr::null_mut();
        f.victim_gobj = core::ptr::null_mut();
        f.anon_221b.set_x221B_b5(0);
        f.anon_221b.set_x221B_b7(0);
        ext::ftColl_8007891C(g, g2, (*hit).damage);
        if (*fp2p).x1988 != 0 {
            ext::ftColl_8007B62C(g2, 0);
        }
        let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::lb_8000B1CC(part_joint(&*fp3, FtPart_TransN2), core::ptr::null(), &mut vec);
        let mut damage = if ext::ftColl_8007B868(g2) == 0 {
            let d = ext::ScriptGame_FighterDealtDamage(fpp, (*hit).damage);
            ext::ScriptGame_HitRuleContact(hit, fp2p, d, 0)
        } else {
            0.0
        };
        if ext::ftColl_80076640(fp2p, &mut damage) {
            ext::ScriptGame_HitRulePercentQueue(hit, fp2p, damage);
        }
        // Argument order of the oracle build (x86-64 clang evaluates call args right to left);
        // the three getters are pure, so the order is not observable in the game.
        let c = ext::Player_GetDefenseRatio((*fp2p).player_id as i32);
        let b = ext::Player_GetAttackRatio((*fpp).player_id as i32);
        let a = ext::gm_8016B248();
        (*fp2p).dmg.kb_applied = ext::ftColl_80079AB0(fp2p, hit, (*hit).unk_count, a, b, c, cd().x10C);
        (*fp2p).dmg.kb_applied = ext::ScriptGame_HitRuleContact(hit, fp2p, (*fp2p).dmg.kb_applied, 1);
        ext::ScriptGame_HitRuleWon(hit, fp2p);
        ext::ScriptGame_HitRuleReport(fpp, fp2p, hit, damage);
        let f2 = &mut *fp2p;
        f2.dmg.x1848_kb_angle = (*hit).kb_angle;
        f2.dmg.facing_dir_1 = -(*fpp).facing_dir;
        f2.dmg.x184c_damaged_hurtbox = 1;
        f2.dmg.x1854_collpos = vec;
        f2.dmg.x1860_element = (*hit).element;
        ext::ftColl_80078710(g, g2, &mut (*fp2p).dmg.facing_dir_1);
        ext::ftCommon_8007EE0C(&mut *fpp, f2i32((*hit).damage as f64));
        ext::ftCo_Damage_CalcKnockback(&mut *fp2p);
        ext::ftCommon_8007D5D4(&mut *fp4);
        if (*fp4).x2226_b2() != 0 {
            let jobj2 = part_joint(&*fp4, FtPart_XRotN);
            ext::lb_8000C390(jobj2);
            (*fp4).set_x2226_b2(0);
            let t = (*fp4).x2174;
            set_translate(jobj2, &t);
            if arg {
                let f4 = &*fp4;
                vec.x = fmadds(f4.facing_dir, f4.x1A70.z * f4.x34_scale.y, vec.x);
                vec.y = fmadds(f4.x1A70.y, f4.x34_scale.y, vec.y);
                vec.z = 0.0;
            }
            let jobj = (*(*fp4).gobj).hsd_obj as *mut HSD_JObj;
            let f3 = &*fp3;
            let vec2 = Vec3 {
                x: f3.cur_pos.x + 0.0,
                y: f3.cur_pos.y + 0.5 * (f3.coll_data.ecb.top.y + f3.coll_data.ecb.bottom.y),
                z: f3.cur_pos.z + 0.0,
            };
            (*fp4).coll_data.last_pos = vec2;
            ext::mpColl_80043670(&mut (*fp4).coll_data);
            (*fp4).coll_data.cur_pos = vec;
            set_translate(jobj, &vec);
            ext::ftCommon_UnlockECB(&mut *fp4);
            ext::mpColl_800471F8(&mut (*fp4).coll_data);
            (*fp4).cur_pos = (*fp4).coll_data.cur_pos;
            let p = (*fp4).cur_pos;
            set_translate(jobj, &p);
        }
        let f2 = &mut *fp2p;
        f2.x1A5C = core::ptr::null_mut();
        f2.victim_gobj = core::ptr::null_mut();
        f2.anon_221b.set_x221B_b7(0);
    }
}

pub fn ftCo_800DE2A8(g: G, other: G) {
    melee_core::ct!("ftCo_800DE2A8");
    ftCo_800DDDE4(g, other, true);
}

pub fn ftCo_800DE2CC(g: G, other: G) {
    melee_core::ct!("ftCo_800DE2CC");
    ftCo_800DDDE4(g, other, false);
}

pub fn ftCo_800DE2F0(g: G) {
    melee_core::ct!("ftCo_800DE2F0");
    let mut hit = HitCapsule::zeroed();
    ext::lbColl_80008D30(&mut hit, &cd().x380);
    ext::ftColl_800788D4(g);
    let mut vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(part_joint(fp(g), FtPart_TransN2), core::ptr::null(), &mut vec);
    let f: *mut Fighter = fp(g);
    unsafe {
        // Oracle argument order (right to left); the getters are pure.
        let d = ext::ScriptGame_ModValue((*f).player_id as i32, 9, (*f).co_attrs.weight);
        let c = ext::Player_GetDefenseRatio((*f).player_id as i32);
        let a = ext::gm_8016B248();
        (*f).dmg.kb_applied = ext::ftColl_80079AB0(f, &mut hit, hit.unk_count, a, 1.0, c, d);
        let f = &mut *f;
        f.dmg.x1848_kb_angle = hit.kb_angle;
        f.dmg.facing_dir_1 = f.facing_dir;
        f.dmg.x184c_damaged_hurtbox = 1;
        f.dmg.x1854_collpos = vec;
        f.dmg.x1860_element = hit.element;
        ext::Fighter_UnkTakeDamage_8006CC30(f, hit.damage);
    }
    ext::ftCo_Damage_CalcKnockback(fp(g));
    ext::ftCo_8008E908(g, 0.0);
}
