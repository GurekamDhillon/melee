#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_ItemScopeRapid.c (Super Scope rapid fire).
//! Every function of the TU; calls out go through `ext`. Checked against the C by
//! tests/tu_ftCo_ItemScopeRapid.rs.
//!
//! The IASA functions write GC `fp + 0x2340` (`mv.co.common.x0`) from GC
//! `p_ftCommonData + 0x5BC` (`x5BC`), and the air one reads the fighter through GC
//! `gobj + 0x2C` (`user_data`).

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

const HSD_PAD_A: u32 = 0x100;

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
        fn_800D769C(fp: &Fighter, msid: i32) -> i32 = crate::tu::ftCo_ItemScopeStart::fn_800D769C;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        ftCommon_ClampAirDrift(fp: &mut Fighter) = crate::ftcommon::clamp_air_drift;
        ftCommon_8007D7FC(fp: &mut Fighter) = crate::ftcommon3::ftcommon_8007d7fc;
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        fn_800D8378(g: G) = |_| todo!("TODO(port) fn_800D8378 (ftCo_ItemScopeFire.c)");
        ft_800892A0(g: G) = |_| todo!("TODO(port) ft_800892A0 (ft_0892.c)");
        ft_80089824(g: G) = crate::ft0892::ft_80089824;
        fn_800D86E0(g: G) = |_| todo!("TODO(port) fn_800D86E0 (ftCo_ItemScopeEnd.c)");
        fn_800D874C(g: G) = |_| todo!("TODO(port) fn_800D874C (ftCo_ItemScopeEnd.c)");
        it_8026B594(item: *mut HSD_GObj) -> bool = |i| crate::tu::ftCo_ItemScopeStart::ext::it_8026B594(i);
        ftCo_800CDE54(g: G) = crate::tu::ft_0cdd::ftco_800cde54;
        ftCo_800CDE74(g: G) = |g| crate::tu::ftCo_ItemScopeStart::ext::ftCo_800CDE74(g);
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
        it_80291DAC(item: *mut HSD_GObj, n: i32) -> i32 = |i, n| crate::tu::ftCo_ItemScopeStart::ext::it_80291DAC(i, n);
        fn_800D84D4(g: G, n: i32) = |_, _| todo!("TODO(port) fn_800D84D4 (ftCo_ItemScopeFire.c)");
    }
}

pub unsafe extern "C" fn fn_800D80F4_c(g: G) {
    fn_800D80F4(g)
}
pub unsafe extern "C" fn fn_800D8378_c(g: G) {
    ext::fn_800D8378(g)
}

#[inline(always)]
fn set_cbs(f: &mut Fighter) {
    f.accessory4_cb = Some(fn_800D80F4_c);
    f.take_dmg_cb = Some(fn_800D8378_c);
}

pub fn fn_800D7BDC(g: G) {
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeRapid as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0x02000000, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe { f.mv.co.common.x0 = cd().x5BC };
    set_cbs(f);
}

pub fn fn_800D7C60(g: G) {
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirRapid as i32);
    ext::Fighter_ChangeMotionState(g, msid, 0x02000000, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    let f = fp(g);
    unsafe { f.mv.co.common.x0 = cd().x5BC };
    set_cbs(f);
}

pub fn fn_800D7CEC(g: G) {
    ext::ftCommon_8007D7FC(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeRapid as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    set_cbs(fp(g));
}

pub fn fn_800D7D70(g: G) {
    ext::ftCommon_8007D5D4(fp(g));
    let msid = ext::fn_800D769C(fp(g), ftCo_MS_ItemScopeAirRapid as i32);
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, msid, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    ext::ftCommon_ClampAirDrift(fp(g));
    set_cbs(fp(g));
}

fn rapid_anim(g: G, done: fn(G)) {
    let f = fp(g);
    if f.cur_anim_frame >= 0.0 && f.cur_anim_frame < f.frame_speed_mul {
        ext::ft_800892A0(g);
        ext::ft_80089824(g);
        let f = fp(g);
        unsafe { f.mv.co.common.x0 = f.mv.co.common.x0.wrapping_sub(1) };
        if unsafe { f.mv.co.common.x0 } == 0 {
            done(g);
        } else if !f.item_gobj.is_null() && ext::it_8026B594(f.item_gobj) {
            let f = fp(g);
            let m = if f.ground_or_air == GA_Ground { ftCo_MS_ItemScopeRapidEmpty } else { ftCo_MS_ItemScopeAirRapidEmpty };
            ext::Fighter_ChangeMotionState(g, m as i32, 0x0C4C5080, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
            set_cbs(fp(g));
        }
    }
}

pub fn ftCo_ItemScopeRapid_Anim(g: G) {
    rapid_anim(g, ext::fn_800D86E0);
}

pub fn ftCo_ItemScopeAirRapid_Anim(g: G) {
    rapid_anim(g, ext::fn_800D874C);
}

pub fn ftCo_ItemScopeRapid_IASA(g: G) {
    let f = fp(g);
    if f.input.pressed_buttons & HSD_PAD_A != 0 {
        unsafe { f.mv.co.common.x0 = cd().x5BC };
    }
}

pub fn ftCo_ItemScopeAirRapid_IASA(g: G) {
    ftCo_ItemScopeRapid_IASA(g);
}

pub fn ftCo_ItemScopeRapid_Phys(g: G) {
    ext::ftCo_800CDE54(g);
}

pub fn ftCo_ItemScopeAirRapid_Phys(g: G) {
    ext::ftCo_800CDE74(g);
}

pub fn ftCo_ItemScopeRapid_Coll(g: G) {
    ext::ft_800841B8(g, fn_800D7D70);
}

pub fn ftCo_ItemScopeAirRapid_Coll(g: G) {
    ext::ft_80082C74(g, fn_800D7CEC);
}

pub fn fn_800D80F4(g: G) {
    let f = fp(g);
    if !f.item_gobj.is_null() {
        let v = ext::it_80291DAC(f.item_gobj, 0);
        ext::fn_800D84D4(g, v);
    }
}
