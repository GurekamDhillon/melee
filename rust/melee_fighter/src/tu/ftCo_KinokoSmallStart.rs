#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_KinokoSmallStart.c (Poison Mushroom shrink).
//! Every function of the TU (and its static inline helper); calls out go through
//! `ext` (the ft_0D27.c helpers default to `tu::ft_0d27`). No fused site. Checked against the C
//! (whole TU, shipping contraction, saturating float->int) by tests/tu_ftCo_KinokoSmallStart.rs.

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    use crate::tu::ft_0d27 as k;
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
        ftCo_800D27C4(f: &Fighter) -> bool = k::ftco_800d27c4;
        ftCo_800D1BA0(g: G, arg1: i32) = |_, _| todo!("TODO(port) ftCo_800D1BA0 (ftCo_KinokoGiantEnd.c)");
        ftCo_800D1D10(g: G, arg1: i32) = |_, _| todo!("TODO(port) ftCo_800D1D10 (ftCo_KinokoGiantEnd.c)");
        fn_800D2B04(f: &mut Fighter) = k::fn_800d2b04;
        fn_800D2BF8(g: G) = k::fn_800d2bf8;
        ftCo_800D2770(g: G, scale: f32) = k::ftco_800d2770;
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ft_8008A324(g: G) = crate::tu::ft_08a1::ft_8008A324;
        fn_800D2B30(g: G) = k::fn_800d2b30;
        fn_800D2890(g: G, msid: i32) = k::fn_800d2890;
        fn_800D299C(g: G, idx: i32, a: f32, b: f32) = k::fn_800d299c;
        fn_800D2AD8(f: &mut Fighter) = k::fn_800d2ad8;
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        fn_800D290C(g: G) = k::fn_800d290c;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |_, _, _, _, _, _, _| todo!("TODO(port) Fighter_ChangeMotionState (fighter.c)");
        fn_800D2CA0(f: &mut Fighter) = k::fn_800d2ca0;
        fn_800D2CD4(f: &mut Fighter) = k::fn_800d2cd4;
        fn_800D2A3C(g: G) -> bool = k::fn_800d2a3c;
        ft_8008403C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008403c;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

/// Call a stored `HSD_GObjEvent` (`fp->mv.co.mushroom.x0/x4(gobj)`).
#[inline(always)]
fn call_ev(e: HSD_GObjEvent, g: G) {
    unsafe { (e.expect("mushroom callback is NULL"))(g) }
}

/// `ftCo_800D1F6C` as the C `HSD_GObjEvent` stored in `mv.co.mushroom.x4` / `take_dmg_cb`.
pub unsafe extern "C" fn ftCo_800D1F6C_cb(g: *mut HSD_GObj) { ftCo_800D1F6C(g) }

pub fn Fighter_PoisonMushroomApply(g: G) -> bool {
    if !ext::ftCo_800D27C4(fp(g)) {
        return false;
    }
    let ms = fp(g).motion_id as i32;
    if ms >= ftCo_MS_KinokoGiantStart as i32 && ms <= ftCo_MS_KinokoSmallEndAir as i32 {
        call_ev(unsafe { fp(g).mv.co.mushroom.x4 }, g);
    }
    let f = fp(g);
    if f.x2220_b6() != 0 {
        f.x2008 = cd().x688;
        return false;
    }
    if f.x2220_b5() != 0 {
        if f.ground_or_air == GA_Ground {
            ext::ftCo_800D1BA0(g, 1);
        } else {
            ext::ftCo_800D1D10(g, 1);
        }
        return true;
    }
    if f.ground_or_air == GA_Ground {
        ftCo_800D1FFC(g);
    } else {
        ftCo_800D21BC(g);
    }
    true
}

pub fn ftCo_800D1F6C(g: G) {
    ext::fn_800D2BF8(g);
    let f = fp(g);
    f.set_x2220_b6(1);
    f.x2008 = cd().x688;
    let s = unsafe { fp(g).mv.co.walk.fast_anim_frame };
    ext::ftCo_800D2770(g, s);
    ext::ftCommon_8007E2FC(g);
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_8008A324(g);
        return;
    }
    call_ev(unsafe { fp(g).mv.co.mushroom.x0 }, g);
}

/// static inline `ftCo_800D1FFC_inline`.
fn ftCo_800D1FFC_inline(g: G, id: i32) {
    ext::fn_800D2B30(g);
    ext::fn_800D2890(g, id);
    let f = fp(g);
    let v = if f.x34_scale.x < 1.0 { f.x34_scale.y * cd().x684 } else { f.x34_scale.y * cd().x680 };
    let y = fp(g).x34_scale.y;
    ext::fn_800D299C(g, 0, y, v);
    ext::fn_800D2B04(fp(g));
    ext::ftCommon_8007EBAC(fp(g), 12, 0);
    unsafe { fp(g).mv.co.mushroom.x4 = Some(ftCo_800D1F6C_cb) };
}

pub fn ftCo_800D1FFC(g: G) {
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallStart as i32, 0x90, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D1FFC_inline(g, id);
    fp(g).take_dmg_cb = Some(ftCo_800D1F6C_cb);
}

/// `fn_800D20EC` (static in the C).
pub fn fn_800D20EC(g: G) {
    ext::fn_800D2CA0(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallStart as i32, 0xC4C5080, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D1F6C_cb);
}

pub fn ftCo_KinokoSmallStart_Anim(g: G) {
    if !ext::fn_800D2A3C(g) {
        ftCo_800D1F6C(g);
    }
}

pub fn ftCo_KinokoSmallStart_Coll(g: G) {
    ext::ft_8008403C(g, fn_800D22AC);
}

pub fn ftCo_800D21BC(g: G) {
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallStartAir as i32, 0x90, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D1FFC_inline(g, id);
    fp(g).take_dmg_cb = Some(ftCo_800D1F6C_cb);
}

pub fn fn_800D22AC(g: G) {
    ext::fn_800D2CD4(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallStartAir as i32, 0xC4C5080, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D1F6C_cb);
}

pub fn ftCo_KinokoSmallStartAir_Anim(g: G) {
    if !ext::fn_800D2A3C(g) {
        ftCo_800D1F6C(g);
    }
}

pub fn ftCo_KinokoSmallStartAir_Coll(g: G) {
    ext::ft_80082C74(g, fn_800D20EC);
}
