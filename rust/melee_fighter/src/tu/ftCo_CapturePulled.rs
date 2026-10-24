#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CapturePulled.c (being pulled in by a grab:
//! the grab-timer formula, the entries, the follow-the-grabber physics and the XRotN constraint).
//! Fused sites (tools/rust-port/fma_sites.py): ftCo_800DA824 and fn_800DA8E4 (the handicap and
//! percent terms) and ftCo_800DB464 (the XRotN position offsets), two f32 fmuladd each, lowered
//! to fmadds. ftCo_800DA824, ftCo_800DB464, fn_800DAD18, fn_800DAA40 and
//! ftCo_CapturePulledHi_Phys are checked bit-exact against the C by
//! tests/tu_ftCo_CapturePulled.rs (gen/cref/tu_shard121.py); the rest is smoke-tested.

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
        Player_80033BB8(slot: i32) -> i32 = |slot: i32| -> i32 { unsafe { crate::tu::player::Player_80033BB8(slot as _) as _ } };
        Player_GetHandicap(slot: i32) -> i32 = |slot: i32| -> i32 { unsafe { crate::tu::player::Player_GetHandicap(slot as _) as _ } };
        ftCo_ReleaseItemAndVictim(g: G) = crate::tu::ftco_800c7590::ftco_release_item_and_victim;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_UnlockECB(f: &mut Fighter) = crate::ftcommon2::unlock_ecb;
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        lb_8000B1CC(j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3) = |j: *mut HSD_JObj, off: *const Vec3, out: &mut Vec3| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(j as _, off as _, out as _); } };
        lb_8000C1C0(j: *mut HSD_JObj, constraint: *mut HSD_JObj) = |j: *mut HSD_JObj, constraint: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000C1C0(j as _, constraint as _); } };
        ftParts_JObjSetRotation(j: *mut HSD_JObj, q: &Quaternion) = |j: *mut HSD_JObj, q: &Quaternion| { unsafe { let _ = crate::tu::ftparts::ftParts_JObjSetRotation(j as _, q as _); } };
        ft_80083C00(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80083c00(g as _, ev)); } };
        ft_8008403C(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_8008403c(g as _, ev)); } };
        ftAnim_8006F368(f: &mut Fighter, idx: i32) -> bool = |f: &mut Fighter, idx: i32| -> bool { unsafe { crate::tu::ftanim::ftAnim_8006F368(f as _, idx as _) as _ } };
        ftAnim_8006DF0C(f: &mut Fighter) = |f: &mut Fighter| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006DF0C(f as _); } };
    }
}

fn set_translate(j: *mut HSD_JObj, t: &Vec3) {
    super::ftCo_CaptureCut::set_translate(j, t)
}
fn joint(f: &Fighter, part: i32) -> *mut HSD_JObj {
    let i = ext::ftParts_GetBoneIndex(f, part);
    unsafe { (*f.parts.offset(i as isize)).joint }
}
/// The grabber's hold joint (`victim->mv.co.capturedamage.x18`) minus this fighter's XRotN.
fn hold_delta(f: &Fighter) -> Vec3 {
    let mut a = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = a;
    let hold = unsafe { fp(f.victim_gobj).mv.co.capturedamage.x18 };
    ext::lb_8000B1CC(hold, core::ptr::null(), &mut a);
    ext::lb_8000B1CC(joint(f, FtPart_XRotN as i32), core::ptr::null(), &mut b);
    Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

/// The grab timer: `x358*(x35C-handicap)+x354 + x360*(x364-(slot+1)) + percent*x368`.
fn grab_timer(f: &Fighter) -> f32 {
    let co = cd();
    let slot = ext::Player_80033BB8(f.player_id as i32) + 1;
    let value = co.x360 * (co.x364 - slot as f32);
    let h = ext::Player_GetHandicap(f.player_id as i32);
    let mut t = fmadds(co.x358, co.x35C - h as f32, co.x354);
    t += value;
    fmadds(f.dmg.x1830_percent, co.x368, t)
}

/// `ftCo_800DA824`.
pub fn ftCo_800DA824(f: &mut Fighter) -> f32 {
    grab_timer(f)
}

/// `fn_800DA8E4`: enter a CapturePulled state `msid`.
pub fn fn_800DA8E4(g: G, vg: G, msid: i32) {
    let f = fp(g);
    let v = fp(vg);
    ext::ftCo_ReleaseItemAndVictim(g);
    f.x1A5C = vg;
    f.victim_gobj = vg;
    f.anon_221b.set_x221B_b5(0);
    f.anon_221b.set_x221B_b7(0);
    f.facing_dir = -v.facing_dir;
    let t = grab_timer(f);
    ext::ftCommon_InitGrab(f, 0, t);
    unsafe {
        f.mv.ca.specials.grav = 0.0;
        f.mv._x0[0xC] = 0;
        f.mv.ca.speciallw.friction = 0.0;
        f.mv.ca.speciallw.x4 = 0 as _;
    }
    ext::Fighter_ChangeMotionState(g, msid, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2FC(g);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
}

/// The shared tail of `fn_800DAA40` and its inlined copies.
fn follow(a0: G, a1: G) {
    let mut d = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    fn_800DAC78(a0, &mut d);
    let r31 = fp(a0);
    let r30 = fp(a1);
    if r31.ground_or_air == GA_Ground {
        r30.x2170 = d.y + r31.cur_pos.y - r30.cur_pos.y;
    } else {
        r30.x2170 = 0.0;
        r31.cur_pos.x += d.x;
        r31.cur_pos.y += d.y;
        r31.cur_pos.z += d.z;
    }
}

/// `fn_800DAA40`.
pub fn fn_800DAA40(a0: G, a1: G) {
    follow(a0, a1)
}

/// `fn_800DAADC`: enter CapturePulledLw / Hi by ground state and snap to the grabber.
pub fn fn_800DAADC(a0: G, a1: G) {
    let msid = if fp(a0).ground_or_air == GA_Ground { ftCo_MS_CapturePulledLw } else { ftCo_MS_CapturePulledHi };
    fn_800DA8E4(a0, a1, msid as i32);
    follow(a0, a1);
    let f = fp(a0);
    if let Some(cb) = f.coll_cb {
        unsafe { cb(a0) };
    }
    let f = fp(a0);
    let p = f.cur_pos;
    set_translate(unsafe { (*a0).hsd_obj as *mut HSD_JObj }, &p);
}

/// `ftCo_CapturePulledHi_Anim`.
pub fn ftCo_CapturePulledHi_Anim(_g: G) {}
/// `ftCo_CapturePulledHi_IASA`.
pub fn ftCo_CapturePulledHi_IASA(_g: G) {}

/// `fn_800DAC78`: the grabber's hold joint minus this fighter's XRotN.
pub fn fn_800DAC78(g: G, out: &mut Vec3) {
    *out = hold_delta(fp(g));
}

/// `fn_800DAD18`: follow the hold joint; true when it is above x3C4 * scale.
pub fn fn_800DAD18(g: G) -> bool {
    let f = fp(g);
    let d = hold_delta(f);
    let r = d.y > cd().x3C4 * f.x34_scale.y;
    f.cur_pos.x += d.x;
    f.cur_pos.y += d.y;
    f.cur_pos.z += d.z;
    r
}

/// `ftCo_CapturePulledHi_Phys`.
pub fn ftCo_CapturePulledHi_Phys(g: G) {
    let f = fp(g);
    let d = hold_delta(f);
    f.cur_pos.x += d.x;
    f.cur_pos.y += d.y;
    f.cur_pos.z += d.z;
}

/// `ftCo_CapturePulledHi_Coll`.
pub fn ftCo_CapturePulledHi_Coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800DAECC));
    }
}

/// `fn_800DAECC`.
pub unsafe extern "C" fn fn_800DAECC(g: G) {
    fn_800DAEEC(g);
}

/// `fn_800DAEEC`: landed while pulled -> CapturePulledLw.
pub fn fn_800DAEEC(g: G) {
    let f = fp(g);
    // ftCommon_AirToGroundStateChange (ftCommon/inlines.h)
    ext::ftCommon_8007D7FC(f);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CapturePulledLw as i32, Ft_MF_UpdateCmd as u32, f.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    let vf = fp(f.victim_gobj);
    let d = hold_delta(f);
    if f.ground_or_air == GA_Ground {
        vf.x2170 = d.y + f.cur_pos.y - vf.cur_pos.y;
    } else {
        vf.x2170 = 0.0;
        f.cur_pos.x += d.x;
        f.cur_pos.y += d.y;
        f.cur_pos.z += d.z;
    }
}

/// `ftCo_CapturePulledLw_Anim`.
pub fn ftCo_CapturePulledLw_Anim(_g: G) {}
/// `ftCo_CapturePulledLw_IASA`.
pub fn ftCo_CapturePulledLw_IASA(_g: G) {}

/// `fn_800DB230_inline`: left the ground while pulled -> CapturePulledHi.
fn to_air(g: G) {
    let f = fp(g);
    ext::ftCommon_8007D5D4(f);
    ext::ftCommon_UnlockECB(f);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CapturePulledHi as i32, Ft_MF_UpdateCmd as u32, f.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    fn_800DAA40(g, fp(g).victim_gobj);
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800DAECC));
    }
    let p = fp(g).cur_pos;
    set_translate(unsafe { (*g).hsd_obj as *mut HSD_JObj }, &p);
}

/// `ftCo_CapturePulledLw_Phys`.
pub fn ftCo_CapturePulledLw_Phys(g: G) {
    if fn_800DAD18(g) {
        to_air(g);
    }
}

/// `ftCo_CapturePulledLw_Coll`.
pub fn ftCo_CapturePulledLw_Coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_8008403C(g, Some(fn_800DB230));
    }
}

/// `fn_800DB230`.
pub unsafe extern "C" fn fn_800DB230(g: G) {
    to_air(g);
}

/// `ftCo_800DB368`: constrain this fighter's XRotN to the attacker's TransN2.
pub fn ftCo_800DB368(att: &mut Fighter, f: &mut Fighter) {
    let constraint = joint(att, FtPart_TransN2 as i32);
    if f.x2226_b2() == 0 {
        let j = joint(f, FtPart_XRotN as i32);
        let q = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
        ext::ftParts_JObjSetRotation(j, &q);
        f.x2174 = unsafe { (*j).translate };
        ext::lb_8000C1C0(j, constraint);
        f.set_x2226_b2(1);
    }
}

/// `ftCo_800DB464`: place the fighter at its XRotN (accessory1 callback).
pub fn ftCo_800DB464(g: G) {
    let f = fp(g);
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint(f, FtPart_XRotN as i32), core::ptr::null(), &mut pos);
    pos.x = fmadds(f.facing_dir, f.x1A70.z * f.x34_scale.y, pos.x);
    pos.y = fmadds(f.x1A70.y, f.x34_scale.y, pos.y);
    pos.z = 0.0;
    f.cur_pos = pos;
}
/// `ftCo_800DB464` as an `HSD_GObjEvent`.
pub unsafe extern "C" fn ftCo_800DB464_cb(g: G) {
    ftCo_800DB464(g)
}

/// `ftCo_800DB500`.
pub fn ftCo_800DB500(g: G) {
    let f = fp(g);
    if f.x2226_b2() != 0 {
        let i = ext::ftParts_GetBoneIndex(f, FtPart_XRotN as i32);
        if ext::ftAnim_8006F368(f, i) {
            let j = joint(f, FtPart_XRotN as i32);
            f.x2174 = unsafe { (*j).translate };
        }
    }
    if f.x2226_b2() != 0 {
        ext::ftAnim_8006DF0C(f);
    }
}
