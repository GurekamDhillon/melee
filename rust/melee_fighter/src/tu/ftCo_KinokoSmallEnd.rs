#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_KinokoSmallEnd.c (Poison Mushroom wearing off).
//! Every function of the TU (and its static inline helper); calls out go through
//! `ext` (the ft_0D27.c helpers default to `tu::ft_0d27`). No fused site. Checked against the C
//! (whole TU, shipping contraction, saturating float->int) by tests/tu_ftCo_KinokoSmallEnd.rs.

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
        fn_800D2818(f: &Fighter) -> bool = k::fn_800d2818;
        fn_800D2BF8(g: G) = k::fn_800d2bf8;
        ftCo_800D2770(g: G, scale: f32) = k::ftco_800d2770;
        ftCommon_8007E2FC(g: G) = |a0| crate::ftcommon2::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        ft_8008A324(g: G) = crate::tu::ft_08a1::ft_8008A324;
        fn_800D2B30(g: G) = k::fn_800d2b30;
        fn_800D2890(g: G, msid: i32) = k::fn_800d2890;
        fn_800D299C(g: G, idx: i32, a: f32, b: f32) = k::fn_800d299c;
        fn_800D2AD8(f: &mut Fighter) = k::fn_800d2ad8;
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        fn_800D290C(g: G) = k::fn_800d290c;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
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

/// `ftCo_800D23F4` as the C `HSD_GObjEvent` stored in `mv.co.mushroom.x4` / `take_dmg_cb`.
pub unsafe extern "C" fn ftCo_800D23F4_cb(g: *mut HSD_GObj) { melee_core::ct!("ftCo_800D23F4_cb"); ftCo_800D23F4(g) }

pub fn Fighter_PoisonMushroomEnd(g: G) -> bool {
    melee_core::ct!("Fighter_PoisonMushroomEnd");
    if !ext::fn_800D2818(fp(g)) {
        return false;
    }
    if fp(g).ground_or_air == GA_Ground {
        ftCo_800D2490(g, 0);
    } else {
        ftCo_800D2600(g, 0);
    }
    true
}

pub fn ftCo_800D23F4(g: G) {
    melee_core::ct!("ftCo_800D23F4");
    ext::fn_800D2BF8(g);
    fp(g).set_x2220_b6(0);
    let s = fp(g).x34_scale.x;
    ext::ftCo_800D2770(g, s);
    if unsafe { fp(g).mv.co.mushroom.x8 } != 0 {
        ext::ftCommon_8007E2FC(g);
        unsafe { fp(g).mv.co.mushroom.x8 = 0 };
    }
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_8008A324(g);
        return;
    }
    call_ev(unsafe { fp(g).mv.co.mushroom.x0 }, g);
}

/// static inline `ftCo_800D2490_inline`.
fn ftCo_800D2490_inline(g: G, id: i32, arg1: i32) {
    ext::fn_800D2B30(g);
    ext::fn_800D2890(g, id);
    unsafe { fp(g).mv.co.mushroom.x8 = arg1 };
    let (y, x) = (fp(g).x34_scale.y, fp(g).x34_scale.x);
    ext::fn_800D299C(g, 0, y, x);
    ext::fn_800D2AD8(fp(g));
    unsafe { fp(g).mv.co.mushroom.x4 = Some(ftCo_800D23F4_cb) };
}

pub fn ftCo_800D2490(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D2490");
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallEnd as i32, 0x91, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D2490_inline(g, id, arg1);
    fp(g).take_dmg_cb = Some(ftCo_800D23F4_cb);
}

/// `fn_800D2530` (static in the C).
pub fn fn_800D2530(g: G) {
    melee_core::ct!("fn_800D2530");
    ext::fn_800D2CA0(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallEnd as i32, 0xC4C5081, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D23F4_cb);
}

pub fn ftCo_KinokoSmallEnd_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoSmallEnd_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D23F4(g);
    }
}

pub fn ftCo_KinokoSmallEnd_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoSmallEnd_Coll");
    ext::ft_8008403C(g, fn_800D26A0);
}

pub fn ftCo_800D2600(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800D2600");
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallEndAir as i32, 0x91, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D2490_inline(g, id, arg1);
    fp(g).take_dmg_cb = Some(ftCo_800D23F4_cb);
}

pub fn fn_800D26A0(g: G) {
    melee_core::ct!("fn_800D26A0");
    ext::fn_800D2CD4(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoSmallEndAir as i32, 0xC4C5081, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D23F4_cb);
}

pub fn ftCo_KinokoSmallEndAir_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoSmallEndAir_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D23F4(g);
    }
}

pub fn ftCo_KinokoSmallEndAir_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoSmallEndAir_Coll");
    ext::ft_80082C74(g, fn_800D2530);
}
