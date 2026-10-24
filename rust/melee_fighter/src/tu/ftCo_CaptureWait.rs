#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureWait.c (held in a grab: the Hi/Lw
//! wait states, the mash and grab-timer drain, the Yoshi-grab hurtbox swap and the jump-out
//! input). No fused sites. Smoke-tested by tests/tu_ftCo_CaptureWait.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}

/// `HSD_PAD_XY`.
const HSD_PAD_XY: u32 = (1 << 10) | (1 << 11);

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
        ftColl_8007B0C0(g: G, kind: i32) = |g: G, kind: i32| { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007b0c0(g as _, kind as _); } };
        ftParts_GetBoneIndex(f: &Fighter, part: i32) -> i32 = crate::ftdmgfall::ftparts_get_bone_index;
        ftYs_Init_8012BAC0(f: &mut Fighter) -> f32 = |f: &mut Fighter| -> f32 { unsafe { crate::chardata::ftYs_Init_8012BAC0(f) } };
        ftCommon_GetModelScale(f: &Fighter) -> f32 = crate::ftcommon2::get_model_scale;
        ftColl_HurtboxInit(f: &mut Fighter, caps: *mut FighterHurtCapsule, init: &ftHurtboxInit) = |f: &mut Fighter, caps: *mut FighterHurtCapsule, init: &ftHurtboxInit| { unsafe { let _ = crate::tu::ftcoll::ftcoll_hurtbox_init(f, caps as _, init); } };
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007D7FC(f: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_UnlockECB(f: &mut Fighter) = crate::ftcommon2::unlock_ecb;
        ftCommon_GrabMash(f: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        it_802A7840(item: G) = |item: G| { unsafe { let _ = melee_items::tu::itlinkhookshot::it_802A7840(item as _); } };
        it_802BAA94(item: G) = |item: G| { unsafe { let _ = melee_items::tu::itsamusgrapple::it_802BAA94(item as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800DA698(g: G, b: bool) = |g: G, b: bool| { unsafe { let _ = crate::tu::ftCo_CatchCut::ftCo_800DA698(g as _, b as _); } };
        ftAnim_SetAnimRate(g: G, rate: f32) = |g: G, rate: f32| { unsafe { let _ = crate::tu::ftanim::ftAnim_SetAnimRate(g as _, rate as _); } };
        ft_80083C00(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_80083c00(g as _, ev)); } };
        ft_8008403C(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_8008403c(g as _, ev)); } };
    }
}

use super::ftCo_CapturePulled as pulled;

/// The Yoshi-grab block shared by every entry: intangible, follow XRotN, egg-sized hurtbox.
fn yoshi_hurtbox(g: G) {
    let f = fp(g);
    let vf = fp(f.victim_gobj);
    if vf.kind == Ft_Kind_Yoshi {
        f.set_invisible(1);
        f.accessory1_cb = Some(pulled::ftCo_800DB464_cb);
        ext::ftColl_8007B0C0(g, HurtCapsule_Intangible as i32);
        let mut h: ftHurtboxInit = unsafe { core::mem::zeroed() };
        h.bone_idx = ext::ftParts_GetBoneIndex(f, FtPart_XRotN as i32) as _;
        h.height = HurtHeight_Mid as _;
        h.is_grabbable = 0 as _;
        h.a_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        h.b_offset = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let ys = ext::ftYs_Init_8012BAC0(vf);
        h.scale = ys / ext::ftCommon_GetModelScale(f);
        let caps = f.hurt_capsules.as_mut_ptr();
        ext::ftColl_HurtboxInit(f, caps, &h);
    }
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
}

/// `fn_800DB5D8`.
pub fn fn_800DB5D8(g: G) {
    yoshi_hurtbox(g);
}

/// `fn_800DB6C8`: enter CaptureWait (Hi from CapturePulledHi, else Lw); kind-specific cleanup.
pub fn fn_800DB6C8(g: G) {
    let f = fp(g);
    let vf = fp(f.victim_gobj);
    if f.motion_id == ftCo_MS_CapturePulledHi as i32 {
        fn_800DB790(g);
    } else {
        fn_800DBAE4(g);
    }
    let f = fp(g);
    match vf.kind {
        k if k == Ft_Kind_Yoshi => pulled::ftCo_800DB368(vf, f),
        k if k == Ft_Kind_Link || k == Ft_Kind_CLink => unsafe {
            if !vf.u.lk.xC.is_null() {
                ext::it_802A7840(vf.u.lk.xC as G);
            }
        },
        k if k == Ft_Kind_Samus => unsafe {
            if !vf.u.ss.x223C.is_null() {
                ext::it_802BAA94(vf.u.ss.x223C as G);
            }
        },
        _ => {}
    }
    ext::ftCommon_8007EBAC(fp(g), 3, 0);
}

/// `fn_800DB790`.
pub fn fn_800DB790(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    yoshi_hurtbox(g);
}

/// `fn_800DB8A4`.
pub fn fn_800DB8A4(g: G) {
    let f = fp(g);
    unsafe { f.mv.co.capturewait.x0 = (f.mv.co.capturewait.x0 as f64 + 1.0) as f32 };
    f.grab_timer -= cd().grab_timer_decrement;
    let m = ext::ftCommon_GrabMash(f, cd().x3A8);
    f.mv.co.capturewait.x8 = m as i32;
}

/// `ftCo_CaptureWaitHi_Anim`.
pub fn ftCo_CaptureWaitHi_Anim(g: G) {
    fn_800DB8A4(g);
    let f = fp(g);
    if f.grab_timer <= 0.0 {
        ext::ftCo_800DA698(f.victim_gobj, false);
        if unsafe { fp(g).mv.co.capturewait.xC } != 0 || fn_800DC044(g) {
            super::ftCo_CaptureJump::fn_800DC070(g);
            return;
        }
        super::ftCo_CaptureCut::ftCo_CaptureCut_Enter(g);
        return;
    }
    unsafe {
        let w = &mut f.mv.co.capturewait;
        if w.x4 != 0.0 {
            w.x4 -= 1.0;
            if w.x4 <= 0.0 && w.x8 == 0 {
                ext::ftAnim_SetAnimRate(g, 1.0);
                fp(g).mv.co.capturewait.x4 = 0.0;
            }
        }
        let f = fp(g);
        if f.mv.co.capturewait.x4 <= 0.0 && f.mv.co.capturewait.x8 != 0 {
            f.mv.co.capturewait.x4 = cd().x3B0;
            ext::ftAnim_SetAnimRate(g, cd().shouldered_anim_rate);
        }
    }
}

/// `ftCo_CaptureWaitHi_IASA`.
pub fn ftCo_CaptureWaitHi_IASA(g: G) {
    fn_800DC014(g);
}

/// `ftCo_CaptureWaitHi_Phys`.
pub fn ftCo_CaptureWaitHi_Phys(g: G) {
    pulled::fn_800DAD18(g);
}

/// `ftCo_CaptureWaitHi_Coll`.
pub fn ftCo_CaptureWaitHi_Coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800DBAC4));
    }
}

/// `fn_800DBAC4`.
pub unsafe extern "C" fn fn_800DBAC4(g: G) {
    fn_800DBBF8(g);
}

/// `fn_800DBAE4`.
pub fn fn_800DBAE4(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    yoshi_hurtbox(g);
}

/// `fn_800DBBF8`: landed while held -> CaptureWaitLw.
pub fn fn_800DBBF8(g: G) {
    let before = fp(g);
    ext::ftCommon_8007D7FC(before);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitLw as i32, Ft_MF_UpdateCmd as u32, before.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    yoshi_hurtbox(g);
    pulled::fn_800DAA40(g, fp(g).victim_gobj);
}

/// `ftCo_CaptureWaitLw_Anim`.
pub fn ftCo_CaptureWaitLw_Anim(g: G) {
    ftCo_CaptureWaitHi_Anim(g);
}

/// `ftCo_CaptureWaitLw_IASA`.
pub fn ftCo_CaptureWaitLw_IASA(g: G) {
    fn_800DC014(g);
}

/// Left the ground while held -> CaptureWaitHi (`ftCo_CaptureWaitLw_Phys_inline` /
/// `fn_800DBED4_inline`).
fn to_air(g: G) {
    let f = fp(g);
    ext::ftCommon_8007D5D4(f);
    ext::ftCommon_UnlockECB(f);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitHi as i32, Ft_MF_UpdateCmd as u32, f.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    fn_800DB5D8(g);
    pulled::fn_800DAA40(g, fp(g).victim_gobj);
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800DBAC4));
    }
    let p = fp(g).cur_pos;
    super::ftCo_CaptureCut::set_translate(unsafe { (*g).hsd_obj as *mut HSD_JObj }, &p);
}

/// `ftCo_CaptureWaitLw_Phys`.
pub fn ftCo_CaptureWaitLw_Phys(g: G) {
    if pulled::fn_800DAD18(g) {
        to_air(g);
    }
}

/// `ftCo_CaptureWaitLw_Coll`.
pub fn ftCo_CaptureWaitLw_Coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_8008403C(g, Some(fn_800DBED4));
    }
}

/// `fn_800DBED4`.
pub unsafe extern "C" fn fn_800DBED4(g: G) {
    to_air(g);
}

/// `fn_800DC014`: X/Y early in the hold buffers a jump-out.
pub fn fn_800DC014(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.capturewait.x0 < cd().x3AC && f.input.pressed_buttons as u32 & HSD_PAD_XY != 0 {
            f.mv.co.capturewait.xC = 1;
        }
    }
}

/// `fn_800DC044`: stick up past the tap-jump threshold.
pub fn fn_800DC044(g: G) -> bool {
    fp(g).input.lstick[0].y >= cd().tap_jump_threshold
}
