#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_KinokoGiantStart.c (Super Mushroom growth).
//! Every function of the TU (and its static inline `ftCo_800D170C_inline`); calls out go through
//! `ext` (the ft_0D27.c helpers default to `tu::ft_0d27`). No fused site. Checked against the C
//! (whole TU, shipping contraction, saturating float->int) by tests/tu_ftCo_KinokoGiantStart.rs.

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
        ftCo_800D2490(g: G, arg1: i32) = crate::tu::ftCo_KinokoSmallEnd::ftCo_800D2490;
        ftCo_800D2600(g: G, arg1: i32) = crate::tu::ftCo_KinokoSmallEnd::ftCo_800D2600;
        fn_800D2BF8(g: G) = k::fn_800d2bf8;
        ftCo_800D2770(g: G, scale: f32) = k::ftco_800d2770;
        ftCommon_8007E2FC(g: G) = |_| todo!("ftCommon_8007E2FC: installed by melee_link");
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

/// `ftCo_800D15D0` as the C `HSD_GObjEvent` stored in `mv.co.mushroom.x4` / `take_dmg_cb`.
pub unsafe extern "C" fn ftCo_800D15D0_cb(g: *mut HSD_GObj) { melee_core::ct!("ftCo_800D15D0_cb"); ftCo_800D15D0(g) }

pub fn Fighter_SuperMushroomApply(g: G) -> bool {
    melee_core::ct!("Fighter_SuperMushroomApply");
    if !ext::ftCo_800D27C4(fp(g)) {
        return false;
    }
    let ms = fp(g).motion_id as i32;
    if ms >= ftCo_MS_KinokoGiantStart as i32 && ms <= ftCo_MS_KinokoSmallEndAir as i32 {
        call_ev(unsafe { fp(g).mv.co.mushroom.x4 }, g);
    }
    let f = fp(g);
    if f.x2220_b5() != 0 {
        f.x2008 = cd().x688;
        return false;
    }
    if f.x2220_b6() != 0 {
        if f.ground_or_air == GA_Ground {
            ext::ftCo_800D2490(g, 1);
        } else {
            ext::ftCo_800D2600(g, 1);
        }
        return true;
    }
    if f.ground_or_air == GA_Ground {
        ftCo_800D170C(g);
    } else {
        ftCo_800D18CC(g);
    }
    true
}

pub fn ftCo_800D15D0(g: G) {
    melee_core::ct!("ftCo_800D15D0");
    ext::fn_800D2BF8(g);
    let f = fp(g);
    f.set_x2220_b5(1);
    f.x2008 = cd().x688;
    let mut percent = f.dmg.x1830_percent;
    if percent > cd().x68C as f32 {
        percent -= cd().x68C as f32;
        if percent >= cd().x690 as f32 {
            percent = cd().x690 as f32;
        }
        f.x2008 = (f.x2008 as f32 + percent) as i32;
    }
    let s = unsafe { fp(g).mv.co.walk.fast_anim_rate };
    ext::ftCo_800D2770(g, s);
    ext::ftCommon_8007E2FC(g);
    if fp(g).ground_or_air == GA_Ground {
        ext::ft_8008A324(g);
        return;
    }
    call_ev(unsafe { fp(g).mv.co.mushroom.x0 }, g);
}

/// static inline `ftCo_800D170C_inline`.
fn ftCo_800D170C_inline(g: G, id: i32) {
    ext::fn_800D2B30(g);
    ext::fn_800D2890(g, id);
    let f = fp(g);
    let v = if f.x34_scale.x > 1.0 { f.x34_scale.y * cd().x67C } else { f.x34_scale.y * cd().x678 };
    let y = fp(g).x34_scale.y;
    ext::fn_800D299C(g, 0, y, v);
    ext::fn_800D2AD8(fp(g));
    ext::ftCommon_8007EBAC(fp(g), 12, 0);
    unsafe { fp(g).mv.co.mushroom.x4 = Some(ftCo_800D15D0_cb) };
}

pub fn ftCo_800D170C(g: G) {
    melee_core::ct!("ftCo_800D170C");
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantStart as i32, 0x90, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D170C_inline(g, id);
    fp(g).take_dmg_cb = Some(ftCo_800D15D0_cb);
}

/// `fn_800D17FC` (static in the C).
pub fn fn_800D17FC(g: G) {
    melee_core::ct!("fn_800D17FC");
    ext::fn_800D2CA0(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantStart as i32, 0xC4C5080, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D15D0_cb);
}

pub fn ftCo_KinokoGiantStart_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoGiantStart_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D15D0(g);
    }
}

pub fn ftCo_KinokoGiantStart_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoGiantStart_Coll");
    ext::ft_8008403C(g, fn_800D19BC);
}

pub fn ftCo_800D18CC(g: G) {
    melee_core::ct!("ftCo_800D18CC");
    let id = fp(g).motion_id as i32;
    ext::fn_800D290C(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantStartAir as i32, 0x90, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800D170C_inline(g, id);
    fp(g).take_dmg_cb = Some(ftCo_800D15D0_cb);
}

pub fn fn_800D19BC(g: G) {
    melee_core::ct!("fn_800D19BC");
    ext::fn_800D2CD4(fp(g));
    let (a, s) = (fp(g).cur_anim_frame, fp(g).frame_speed_mul);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_KinokoGiantStartAir as i32, 0xC4C5080, a, s, 0.0, core::ptr::null_mut());
    fp(g).take_dmg_cb = Some(ftCo_800D15D0_cb);
}

pub fn ftCo_KinokoGiantStartAir_Anim(g: G) {
    melee_core::ct!("ftCo_KinokoGiantStartAir_Anim");
    if !ext::fn_800D2A3C(g) {
        ftCo_800D15D0(g);
    }
}

pub fn ftCo_KinokoGiantStartAir_Coll(g: G) {
    melee_core::ct!("ftCo_KinokoGiantStartAir_Coll");
    ext::ft_80082C74(g, fn_800D17FC);
}
