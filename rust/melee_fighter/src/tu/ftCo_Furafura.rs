#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Furafura.c (shield-break dizzy). Every function
//! of the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_Furafura.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_InitGrab(fp: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ft_800885A8(fp: &mut Fighter, s: i32, v: u8, p: u8) = crate::ft0881::ft_800885a8;
        ftCommon_GrabMash(fp: &mut Fighter, v: f32) -> bool = crate::ftcommon2::grab_mash;
        ft_8008A2BC(g: G) = crate::hooks::ft_8008a2bc;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_80083F88(g: G) = crate::tu::ft_081b::ft_80083f88;
    }
}

pub fn ftCo_80099010(g: G) {
    melee_core::ct!("ftCo_80099010");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Furafura as i32, (Ft_MF_SkipModel | Ft_MF_SkipMatAnim) as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.shield_health = cd().x280_unkShieldHealth;
    let mut percent = cd().x2F8 - f.dmg.x1830_percent;
    if percent < 0.0 {
        percent = 0.0;
    }
    ext::ftCommon_InitGrab(f, 0, percent + cd().x2FC);
    ext::ftCommon_8007EBAC(fp(g), 25, 0);
    ext::ft_800885A8(fp(g), 95, 127, 64);
}

pub fn ftCo_Furafura_Anim(g: G) {
    melee_core::ct!("ftCo_Furafura_Anim");
    let f = fp(g);
    f.shield_health = cd().x280_unkShieldHealth;
    f.grab_timer -= cd().x300;
    ext::ftCommon_GrabMash(f, cd().x304);
    if fp(g).grab_timer <= 0.0 {
        ext::ft_8008A2BC(g);
    }
}

pub fn ftCo_Furafura_IASA(_g: G) { melee_core::ct!("ftCo_Furafura_IASA");}

pub fn ftCo_Furafura_Phys(g: G) {
    melee_core::ct!("ftCo_Furafura_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_Furafura_Coll(g: G) {
    melee_core::ct!("ftCo_Furafura_Coll");
    ext::ft_80083F88(g);
}
