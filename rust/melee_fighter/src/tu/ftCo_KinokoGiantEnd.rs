#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_KinokoGiantEnd.c (Super Mushroom wears off).
//! Every function of the TU, the static inline included; calls out go through `ext`. Checked
//! against the C by tests/tu_ftCo_KinokoGiantEnd.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
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
        fn_800D2818(fp: &mut Fighter) -> bool = |fp: &mut Fighter| -> bool { unsafe { crate::tu::ft_0d27::fn_800d2818(fp as _) as _ } };
        fn_800D2BF8(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2bf8(g as _); } };
        ftCo_800D2770(g: G, s: f32) = |g: G, s: f32| { unsafe { let _ = crate::tu::ft_0d27::ftco_800d2770(g as _, s as _); } };
        ftCommon_8007E2FC(g: G) = |g: G| { unsafe { let _ = crate::ftcommon2::ftcommon_8007e2fc(&mut *(g as *mut _)); } };
        ft_8008A324(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_08a1::ft_8008A324(g as _); } };
        fn_800D2B30(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2b30(g as _); } };
        fn_800D2890(g: G, id: i32) = |g: G, id: i32| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2890(g as _, id as _); } };
        fn_800D299C(g: G, a: i32, b: f32, c: f32) = |g: G, a: i32, b: f32, c: f32| { unsafe { let _ = crate::tu::ft_0d27::fn_800d299c(g as _, a as _, b as _, c as _); } };
        fn_800D2B04(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2b04(fp as _); } };
        fn_800D290C(g: G) = |g: G| { unsafe { let _ = crate::tu::ft_0d27::fn_800d290c(g as _); } };
        fn_800D2CA0(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2ca0(fp as _); } };
        fn_800D2CD4(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { let _ = crate::tu::ft_0d27::fn_800d2cd4(fp as _); } };
        fn_800D2A3C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ft_0d27::fn_800d2a3c(g as _) as _ } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ft_8008403C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008403c;
        ft_80082C74(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_80082c74;
    }
}

pub unsafe extern "C" fn ftCo_800D1B04_c(g: G) {
    melee_core::ct!("ftCo_800D1B04_c");
    ftCo_800D1B04(g)
}

pub fn Fighter_SuperMushroomEnd(g: G) -> i32 {
    melee_core::ct!("Fighter_SuperMushroomEnd");
    if !ext::fn_800D2818(fp(g)) {
        return 0;
    }
    if fp(g).ground_or_air == GA_Ground {
        ftCo_800D1BA0(g, 0);
    } else {
        ftCo_800D1D10(g, 0);
    }
    1
}

pub fn ftCo_800D1B04(g: G) {
    melee_core::ct!("ftCo_800D1B04");
    ext::fn_800D2BF8(g);
    let f = fp(g);
    f.set_x2220_b5(0);
    ext::ftCo_800D2770(g, fp(g).x34_scale.x);
    let f = fp(g);
    if unsafe { f.mv.co.mushroom.x8 } != 0 {
        ext::ftCommon_8007E2FC(g);
        unsafe { fp(g).mv.co.mushroom.x8 = 0 };
    }
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_8008A324(g);
        return;
    }
    if let Some(cb) = unsafe { fp(g).mv.co.mushroom.x0 } {
        unsafe { cb(g) };
    }
}

/// static inline `ftCo_800D1BA0_inline`.
#[inline(always)]
pub fn ftCo_800D1BA0_inline(g: G, id: i32, arg1: i32) {
    melee_core::ct!("ftCo_800D1BA0_inline");
    ext::fn_800D2B30(g);
    ext::fn_800D2890(g, id);
    let f = fp(g);
    unsafe { f.mv.co.mushroom.x8 = arg1 };
    let (y, x) = (f.x34_scale.y, f.x34_scale.x);
    ext::fn_800D299C(g, 1, y, x);
    ext::fn_800D2B04(fp(g));
    unsafe { fp(g).mv.co.mushroom.x4 = Some(ftCo_800D1B04_c) };
}

pub fn ftCo_800D1BA0(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D1BA0");
    let id = fp(g).motion_id;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantEnd as i32, 0x91, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D1BA0_inline(g, id, arg1);
    fp(g).take_dmg_cb = Some(ftCo_800D1B04_c);
}

pub fn fn_800D1C40(g: G) {
    melee_core::ct!("fn_800D1C40");
    ext::fn_800D2CA0(fp(g));
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantEnd as i32, 0xC4C5081, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D1B04_c);
}

pub fn ftCo_KinokoGiantEnd_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoGiantEnd_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D1B04(g);
    }
}

pub fn ftCo_KinokoGiantEnd_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoGiantEnd_Coll");
    ext::ft_8008403C(g, fn_800D1DB0);
}

pub fn ftCo_800D1D10(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D1D10");
    let id = fp(g).motion_id;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantEndAir as i32, 0x91, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D1BA0_inline(g, id, arg1);
    fp(g).take_dmg_cb = Some(ftCo_800D1B04_c);
}

pub fn fn_800D1DB0(g: G) {
    melee_core::ct!("fn_800D1DB0");
    ext::fn_800D2CD4(fp(g));
    let f = fp(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantEndAir as i32, 0xC4C5081, f.cur_anim_frame, f.frame_speed_mul, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D1B04_c);
}

pub fn ftCo_KinokoGiantEndAir_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoGiantEndAir_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D1B04(g);
    }
}

pub fn ftCo_KinokoGiantEndAir_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoGiantEndAir_Coll");
    ext::ft_80082C74(g, fn_800D1C40);
}
