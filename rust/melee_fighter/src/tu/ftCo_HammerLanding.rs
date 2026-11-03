#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerLanding.c. Every function of the TU; calls out go
//! through `ext`. Checked against the C by tests/tu_ftCo_HammerLanding.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[allow(dead_code)]
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
        ftCo_800C54C4(fp: &mut Fighter) -> u32 = |fp: &mut Fighter| -> u32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C54C4(fp as _) as _ } };
        ftCo_800C548C(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C548C(fp as _) as _ } };
        ftCo_800C4E94(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4E94(fp as _); } };
        ftCo_800C4F64(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4F64(g as _); } };
        ftCo_Landing_Enter(g: G, msid: i32, f1: bool, f2: u32, a: f32, b: f32) = |g: G, msid: i32, f1: bool, f2: u32, a: f32, b: f32| { unsafe { crate::tu::ftCo_Landing::ftCo_Landing_Enter(g as _, msid as _, f1 as _, f2 as _, a as _, b as _); } };
        ftCo_800C4ED8(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4ED8(g as _); } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_HammerWait_Coll(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_HammerWait_Coll(g as _); } };
    }
}

pub fn ftCo_HammerLanding_Enter(g: G) {
    melee_core::ct!("ftCo_HammerLanding_Enter");
    // mod host site (hooks-v1 0x800C5F88, melee_core::modhook): one None check when no mod hooks it
    melee_core::mod_hook!(0x800C5F88, [melee_core::modhook::Arg::Fighter(g as _)], ftCo_HammerLanding_Enter__orig(g));
    #[allow(unused_unsafe)] unsafe { ftCo_HammerLanding_Enter__orig(g) }
}

fn ftCo_HammerLanding_Enter__orig(g: G) {
    unsafe { fp(g).mv.co.hammerlanding.x4 = cd().x6B4 as f32 };
    let flags = ext::ftCo_800C54C4(fp(g));
    let start = ext::ftCo_800C548C(fp(g));
    ext::ftCo_Landing_Enter(g, ftCo_MS_HammerLanding as i32, Ft_MF_KeepFastFall != 0, flags, start, 1.0);
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerLanding_Anim(g: G) {
    melee_core::ct!("ftCo_HammerLanding_Anim");
    ext::ftCo_800C4F64(g);
    if unsafe { fp(g).mv.co.hammerlanding.x4 } <= 0.0 {
        ext::ftCo_800C4ED8(g);
    }
    unsafe { fp(g).mv.co.hammerlanding.x4 -= 1.0 };
}

pub fn ftCo_HammerLanding_IASA(_g: G) { melee_core::ct!("ftCo_HammerLanding_IASA");}

pub fn ftCo_HammerLanding_Phys(g: G) {
    melee_core::ct!("ftCo_HammerLanding_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_HammerLanding_Coll(g: G) {
    melee_core::ct!("ftCo_HammerLanding_Coll");
    ext::ftCo_HammerWait_Coll(g);
}
