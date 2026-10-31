#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoTurn.c (DK cargo turn). Every function
//! of the TU; calls out go through `ext`. Checked against the C by tests/tu_ftCo_CargoTurn.rs.

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
        ftCo_800C97A8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Turn::ftCo_800C97A8(g as _) as _ } };
        ftCo_Turn_Enter(g: G, msid: i32, flags: u32, a: f32, b: f32, c: f32) = |g: G, msid: i32, flags: u32, a: f32, b: f32, c: f32| { unsafe { crate::tu::ftCo_Turn::ftCo_Turn_Enter(g as _, msid as _, flags as _, a as _, b as _, c as _); } };
        ftCo_8009C5A4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { crate::tu::ftCo_Shouldered::ftCo_8009C5A4(g as _, msid as _); } };
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftCo_8009B518(g: G) = crate::tu::ftCo_CargoWait::ftCo_8009B518;
        ftCo_8009BF3C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoThrow::ftCo_8009BF3C(g as _) as _ } };
        ftCo_8009BB1C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoJump::ftCo_8009BB1C(g as _) as _ } };
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ftCo_CargoWait_Coll(g: G) = crate::tu::ftCo_CargoWait::ftCo_CargoWait_Coll;
    }
}

pub fn ftCo_8009B818(g: G) -> bool {
    melee_core::ct!("ftCo_8009B818");
    if ext::ftCo_800C97A8(g) {
        ftCo_8009B860(g);
        return true;
    }
    false
}

pub fn ftCo_8009B860(g: G) {
    melee_core::ct!("ftCo_8009B860");
    let f = fp(g);
    let a = unsafe { &*f.x2CC };
    ext::ftCo_Turn_Enter(g, a.x4_motion_state + 4, 0, 0.0, a.cargo_hold.x20_TURN_SPEED, 0.0);
    ext::ftCo_8009C5A4(fp(g).victim_gobj, ftCo_MS_ShoulderedTurn as i32);
}

pub fn ftCo_CargoTurn_Anim(g: G) {
    melee_core::ct!("ftCo_CargoTurn_Anim");
    let f = fp(g);
    unsafe {
        if f.mv.co.walk.fast_anim_frame > 0.0 {
            f.mv.co.walk.fast_anim_frame -= 1.0;
        } else if f.mv.co.cargoturn.x0 == 0 {
            let other = &mut *((*f.victim_gobj).user_data as *mut Fighter);
            f.mv.co.cargoturn.x0 = 1;
            f.facing_dir = -f.facing_dir;
            other.facing_dir = f.facing_dir;
        }
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        ext::ftCo_8009B518(g);
    }
}

pub fn ftCo_CargoTurn_IASA(g: G) {
    melee_core::ct!("ftCo_CargoTurn_IASA");
    if ext::ftCo_8009BF3C(g) {
        return;
    }
    if ext::ftCo_8009BB1C(g) {
        return;
    }
}

pub fn ftCo_CargoTurn_Phys(g: G) {
    melee_core::ct!("ftCo_CargoTurn_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_CargoTurn_Coll(g: G) {
    melee_core::ct!("ftCo_CargoTurn_Coll");
    ext::ftCo_CargoWait_Coll(g);
}
