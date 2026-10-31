#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerKneeBend.c. Every function of the TU; calls out go
//! through `ext`. Checked against the C by tests/tu_ftCo_HammerKneeBend.rs.

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
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_800C54C4(fp: &mut Fighter) -> u32 = |fp: &mut Fighter| -> u32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C54C4(fp as _) as _ } };
        ftCo_800C548C(fp: &mut Fighter) -> f32 = |fp: &mut Fighter| -> f32 { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C548C(fp as _) as _ } };
        ftCo_800C4E94(fp: &mut Fighter) = |fp: &mut Fighter| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4E94(fp as _); } };
        ftCo_800C4F64(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_800C4F64(g as _); } };
        ftCo_800C5A98(g: G) = crate::tu::ftCo_HammerJump::ftCo_800C5A98;
        ftCo_KneeBend_Check_ShortHop(g: G) = |g: G| { unsafe { crate::tu::ftCo_KneeBend::ftCo_KneeBend_Check_ShortHop(g as _); } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_HammerWait_Coll(g: G) = |g: G| { unsafe { crate::tu::ftCo_HammerWait::ftCo_HammerWait_Coll(g as _); } };
    }
}

pub fn ftCo_800C5B88(g: G, arg1: i32) {
    melee_core::ct!("ftCo_800C5B88");
    let f = fp(g);
    unsafe {
        f.mv.co.hammerkneebend.x4 = arg1;
        f.mv.co.hammerkneebend.x0 = 0;
        f.mv.co.cargokneebend.x8 = f.co_attrs.jump_startup_time;
    }
    let flags = ext::ftCo_800C54C4(fp(g));
    let start = ext::ftCo_800C548C(fp(g));
    ext::Fighter_ChangeMotionState(g, ftCo_MS_HammerKneeBend as i32, flags, start, 1.0, 0.0, core::ptr::null_mut());
    ext::ftCo_800C4E94(fp(g));
}

pub fn ftCo_HammerKneeBend_Anim(g: G) {
    melee_core::ct!("ftCo_HammerKneeBend_Anim");
    ext::ftCo_800C4F64(g);
    if unsafe { fp(g).mv.co.hammerkneebend.x8 } <= 0.0 {
        ext::ftCo_800C5A98(g);
    }
    unsafe { fp(g).mv.co.hammerkneebend.x8 -= 1.0 };
}

pub fn ftCo_HammerKneeBend_IASA(g: G) {
    melee_core::ct!("ftCo_HammerKneeBend_IASA");
    ext::ftCo_KneeBend_Check_ShortHop(g);
}

pub fn ftCo_HammerKneeBend_Phys(g: G) {
    melee_core::ct!("ftCo_HammerKneeBend_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_HammerKneeBend_Coll(g: G) {
    melee_core::ct!("ftCo_HammerKneeBend_Coll");
    ext::ftCo_HammerWait_Coll(g);
}
