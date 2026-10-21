//! src/melee/ft/kinds/ftCommon/ftCo_0DC2.c: CaptureDamageHi / CaptureDamageLw (the grabbed
//! victim's hit reactions). No fused sites. Every callee outside the ported set is an `ext`
//! stub; the HSD_GObjEvent callbacks this TU hands out are `extern "C"` so they fit the C slots.
//! Smoke-tested by tests/tu_ftco_0dc2.rs.
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;

pub type G = *mut Fighter_GObj;

pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    pub fn Fighter_ChangeMotionState(_g: G, _msid: i32, _flags: u32, _start: f32, _speed: f32, _blend: f32, _acc: G) {
        panic!("TODO(group3 fighter.c) Fighter_ChangeMotionState")
    }
    pub fn fn_800DB5D8(_g: G) { panic!("TODO(ftCo_Attack100 / CaptureDamage) fn_800DB5D8") }
    pub fn fn_800DB8A4(_g: G) { panic!("TODO(ftCo_Attack100) fn_800DB8A4") }
    pub fn fn_800DB790(_g: G) { panic!("TODO(ftCo_Attack100) fn_800DB790") }
    pub fn fn_800DBAE4(_g: G) { panic!("TODO(ftCo_Attack100) fn_800DBAE4") }
    pub fn fn_800DAD18(_g: G) -> bool { panic!("TODO(ftCo_Attack100) fn_800DAD18") }
    pub fn fn_800DAA40(_g: G, _victim: G) { panic!("TODO(ftCo_Attack100) fn_800DAA40") }
    pub fn ft_80083C00(_g: G, _cb: HSD_GObjEvent) { panic!("TODO(group3 ft_081B) ft_80083C00") }
    pub fn ft_8008403C(_g: G, _cb: HSD_GObjEvent) { panic!("TODO(group3 ft_081B) ft_8008403C") }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj(g: G) -> *mut HSD_JObj {
    unsafe { (*g).hsd_obj as *mut HSD_JObj }
}

/// `ftCo_800DC284`: enter CaptureDamageHi.
pub fn ftco_800dc284(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureDamageHi as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).mv.co.capturedamage.x4 = 0.0;
    ext::fn_800DB5D8(g);
}

pub fn ftco_capturedamagehi_anim(g: G) {
    ext::fn_800DB8A4(g);
    if !h::ftanim_is_frames_remaining(g) {
        ext::fn_800DB790(g);
    }
}

pub fn ftco_capturedamagehi_iasa(_g: G) {}

pub fn ftco_capturedamagehi_phys(g: G) {
    ext::fn_800DAD18(g);
}

pub fn ftco_capturedamagehi_coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800dc384));
    }
}

/// `fn_800DC384` (HSD_GObjEvent).
pub unsafe extern "C" fn fn_800dc384(g: *mut HSD_GObj) {
    fn_800dc404(g);
}

/// `ftCo_800DC3A4`: enter CaptureDamageLw.
pub fn ftco_800dc3a4(g: G) {
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureDamageLw as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).mv.co.capturedamage.x4 = 0.0;
    ext::fn_800DB5D8(g);
}

/// `fn_800DC404`: switch to CaptureDamageLw keeping the frame.
pub fn fn_800dc404(g: G) {
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureDamageLw as i32, Ft_MF_UpdateCmd, f.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    ext::fn_800DB5D8(g);
    ext::fn_800DAA40(g, f.victim_gobj);
}

pub fn ftco_capturedamagelw_anim(g: G) {
    ext::fn_800DB8A4(g);
    if !h::ftanim_is_frames_remaining(g) {
        ext::fn_800DBAE4(g);
    }
}

pub fn ftco_capturedamagelw_iasa(_g: G) {}

/// Shared body of `ftCo_CaptureDamageLw_Phys_inline` / `fn_800DC624_inline`: back to Hi.
fn to_hi(g: G) {
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureDamageHi as i32, Ft_MF_UpdateCmd, f.cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
    ext::fn_800DB5D8(g);
    ext::fn_800DAA40(g, f.victim_gobj);
    if fp(g).x2226_b2() == 0 {
        ext::ft_80083C00(g, Some(fn_800dc384));
    }
    let pos = f.cur_pos;
    h::hsd_jobj_set_translate(jobj(g), &pos);
}

pub fn ftco_capturedamagelw_phys(g: G) {
    if ext::fn_800DAD18(g) {
        to_hi(g);
    }
}

pub fn ftco_capturedamagelw_coll(g: G) {
    if fp(g).x2226_b2() == 0 {
        ext::ft_8008403C(g, Some(fn_800dc624));
    }
}

/// `fn_800DC624` (HSD_GObjEvent).
pub unsafe extern "C" fn fn_800dc624(g: *mut HSD_GObj) {
    to_hi(g);
}
