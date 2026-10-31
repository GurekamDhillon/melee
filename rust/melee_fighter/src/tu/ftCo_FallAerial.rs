#![allow(non_snake_case, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_FallAerial.c. Every function of the TU;
//! calls out of it go through `ext`. Checked against the C by tests/tu_ftCo_FallAerial.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    #[allow(unused_imports)]
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
        ftCo_Fall_Anim_Inner(g: G, mv_x4: *mut f32, n: i32, f: i32, b: i32) = crate::tu::ftCo_Fall::ftCo_Fall_Anim_Inner;
        ftCo_800CC988(g: G, a: f32) = crate::tu::ftCo_Fall::ftCo_800CC988;
        ftCo_Fall_IASA_Inner(g: G) -> bool = crate::tu::ftCo_Fall::ftCo_Fall_IASA_Inner;
        ft_80084DB0(g: G) = crate::ft084e::ft_80084db0;
    }
}

pub fn ftCo_FallAerial_Enter(g: G) {
    melee_core::ct!("ftCo_FallAerial_Enter");
    ext::Fighter_ChangeMotionState(g, ftCo_MS_FallAerial, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    unsafe {
        f.mv.co.fallaerial.smid = ftCo_SM_FallAerial;
        f.mv.co.fallaerial.x4 = 0.0;
    }
}

pub fn ftCo_FallAerial_Anim(g: G) {
    melee_core::ct!("ftCo_FallAerial_Anim");
    let f = fp(g);
    let p = unsafe { core::ptr::addr_of_mut!(f.mv.co.fallaerial.x4) };
    ext::ftCo_Fall_Anim_Inner(g, p, ftCo_SM_FallAerial, ftCo_SM_FallAerialF, ftCo_SM_FallAerialB);
    ext::ftCo_800CC988(g, unsafe { fp(g).mv.co.fallaerial.x4 });
}

pub fn ftCo_FallAerial_IASA(g: G) {
    melee_core::ct!("ftCo_FallAerial_IASA");
    ext::ftCo_Fall_IASA_Inner(g);
}

pub fn ftCo_FallAerial_Phys(g: G) {
    melee_core::ct!("ftCo_FallAerial_Phys");
    ext::ft_80084DB0(g);
}
