#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureWaitKirby.c (held in Kirby's mouth).
//! Every function of the TU; calls out of it go through `ext` (unported ones panic until
//! rewired). Checked against the C (whole TU, shipping contraction) by
//! tests/tu_ftCo_CaptureWaitKirby.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        // rp3 ftCo_800DB368
        ftCo_800DB368(victim: *mut Fighter, fp: *mut Fighter) = |victim: *mut Fighter, fp: *mut Fighter| unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB368(&mut *(victim as *mut _), &mut *(fp as *mut _)); };
        // rp3-end
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        Fighter_UpdateModelScale(g: G) = crate::hooks::fighter_update_model_scale;
        ftCo_800DB464(g: G) = |g: G| { unsafe { crate::tu::ftCo_CapturePulled::ftCo_800DB464(g as _); } };
        ftColl_8007B62C(g: G, v: i32) = |g: G, v: i32| { unsafe { crate::tu::ftcoll::ftcoll_8007b62c(g as _, v as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftKb_SpecialN_800F598C(g: G, a: i32) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F598C (ftkirby)");
        ftKb_SpecialN_800F597C(g: G) -> bool = |g: G| -> bool { unsafe { (crate::chardata::ftKb_SpecialN_800F597C(&*(g as *const _)) != 0 as _) } };
        ftKb_SpecialN_800F5954(g: G) = |_| todo!("TODO(port) ftKb_SpecialN_800F5954 (ftkirby)");
        ftKb_SpecialN_800F5A04(g: G, a: f32) = |_, _| todo!("TODO(port) ftKb_SpecialN_800F5A04 (ftkirby)");
        ftKb_SpecialN_800F5A38(g: G, victim: *mut Fighter) -> bool = |g: G, victim: *mut Fighter| -> bool { unsafe { crate::chardata::ftKb_SpecialN_800F5A38(&*(g as *const _), &*(victim as *const _)) } };
        ftKb_SpecialN_800F5B00(g: G) -> f32 = |g: G| -> f32 { unsafe { crate::chardata::ftKb_SpecialN_800F5B00(&*(g as *const _)) } };
        ftKb_SpecialN_800F5B10(g: G) -> f32 = |g: G| -> f32 { unsafe { crate::chardata::ftKb_SpecialN_800F5B10(&*(g as *const _)) } };
        ftKb_SpecialN_800F5A60(g: G) -> f32 = |g: G| -> f32 { unsafe { crate::chardata::ftKb_SpecialN_800F5A60(&*(g as *const _)) } };
        ftCommon_GrabMash(f: &mut Fighter, a: f32) -> bool = crate::ftcommon2::grab_mash;
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |_| todo!("TODO(port) HSD_JObjSetMtxDirtySub (baselib jobj.c)");
        ftCo_CaptureCut_Enter(g: G) = crate::tu::ftCo_CaptureCut::ftCo_CaptureCut_Enter;
        ftCo_800DA698(g: G, b: bool) = |g: G, b: bool| { unsafe { crate::tu::ftCo_CatchCut::ftCo_800DA698(g as _, b as _); } };
    }
}

/// `ftCo_800DB464` as the `accessory1_cb` value.
pub unsafe extern "C" fn ftCo_800DB464_cb(g: G) {
    melee_core::ct!("ftCo_800DB464_cb");
    ext::ftCo_800DB464(g)
}

/// `HSD_JObjSetScale` (jobj.h inline; asserts are debug-only).
fn hsd_jobj_set_scale(j: *mut HSD_JObj, scale: &Vec3) {
    unsafe {
        (*j).scale = *scale;
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            let fl = (*j).flags;
            // HSD_JObjSetMtxDirty: skip when already dirty (and not user-defined).
            if !(fl & JOBJ_USER_DEF_MTX as u32 == 0 && fl & JOBJ_MTX_DIRTY as u32 != 0) {
                ext::HSD_JObjSetMtxDirtySub(j);
            }
        }
    }
}

pub fn ftCo_800BD620(g: G) {
    melee_core::ct!("ftCo_800BD620");
    let f = fp(g);
    let victim = unsafe { (*f.victim_gobj).user_data as *mut Fighter };
    f.set_x2226_b2(0);
    ext::ftCo_800DB368(victim, f as *mut Fighter);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureWaitKirby as i32, Ft_MF_SkipModel | Ft_MF_SkipMatAnim, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::Fighter_UpdateModelScale(g);
    let f = fp(g);
    f.accessory1_cb = Some(ftCo_800DB464_cb);
    f.set_invisible(1);
    ext::ftColl_8007B62C(g, 2);
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007E2FC(g);
}

fn inline_a0(f: &mut Fighter, arg1: i32) {
    f.active_timer.lstick.y = 254;
    ext::ftKb_SpecialN_800F598C(f.victim_gobj, arg1);
}

pub fn ftCo_CaptureWaitKirby_Anim(_g: G) { melee_core::ct!("ftCo_CaptureWaitKirby_Anim");}

pub fn ftCo_800BD6EC_noinline(g: G) {
    melee_core::ct!("ftCo_800BD6EC_noinline");
    ftCo_800BD6EC(g);
}

pub fn ftCo_800BD6EC(g: G) {
    melee_core::ct!("ftCo_800BD6EC");
    let f = fp(g);
    let cd = crate::hooks::common_data();
    if f.input.lstick[0].y >= cd.tap_jump_threshold && (f.active_timer.lstick.y as i32) < cd.tap_jump_window {
        inline_a0(f, 1);
        return;
    }
    if f.input.lstick[0].y <= cd.x464
        && (f.active_timer.lstick.y as f32) < cd.x468
        && ext::ftKb_SpecialN_800F597C(f.victim_gobj)
    {
        let f = fp(g);
        inline_a0(f, -1);
        ext::ftKb_SpecialN_800F5954(fp(g).victim_gobj);
    }
}

fn inline_b0(g: G) {
    let f = fp(g);
    let cd = crate::hooks::common_data();
    let mut lstick_x = f.input.lstick[0].x;
    if lstick_x < 0.0 {
        lstick_x = -lstick_x;
    }
    if lstick_x >= cd.dash_smash_stick_threshold
        && (f.active_timer.lstick.x as f32) < cd.dash_smash_window as f32 + cd.x44
    {
        f.active_timer.lstick.x = 254;
        let value: i32 = if f.input.lstick[0].x < 0.0 { -1 } else { 1 };
        ext::ftKb_SpecialN_800F5A04(f.victim_gobj, value as f32);
    }
}

pub fn ftCo_CaptureWaitKirby_IASA(g: G) {
    melee_core::ct!("ftCo_CaptureWaitKirby_IASA");
    let jobj = unsafe { (*g).hsd_obj as *mut HSD_JObj };
    let user = unsafe { (*g).user_data as *mut Fighter };
    if ext::ftKb_SpecialN_800F5A38(fp(g).victim_gobj, user) {
        ftCo_800BD6EC_noinline(g);
        inline_b0(g);
    }
    let d = ext::ftKb_SpecialN_800F5B00(fp(g).victim_gobj);
    fp(g).grab_timer -= d;
    let a = ext::ftKb_SpecialN_800F5B10(fp(g).victim_gobj);
    let r = ext::ftCommon_GrabMash(fp(g), a);
    fp(g).mv.co.capturekirby.x18 = r as i32;
    if fp(g).grab_timer <= 0.0 {
        let victim_gobj = fp(g).victim_gobj;
        fp(g).facing_dir = ext::ftKb_SpecialN_800F5A60(victim_gobj);
        let scale = unsafe { fp(g).mv.co.capturekirby.scale };
        hsd_jobj_set_scale(jobj, &scale);
        ext::ftCo_CaptureCut_Enter(g);
        ext::ftCo_800DA698(victim_gobj, false);
    }
}

pub fn ftCo_CaptureWaitKirby_Phys(_g: G) { melee_core::ct!("ftCo_CaptureWaitKirby_Phys");}

pub fn ftCo_CaptureWaitKirby_Coll(_g: G) { melee_core::ct!("ftCo_CaptureWaitKirby_Coll");}
