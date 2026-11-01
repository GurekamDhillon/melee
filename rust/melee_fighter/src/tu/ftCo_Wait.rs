#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Wait.c (every function). Calls out of
//! the TU go through `ext` (unported ones panic until rewired). Checked against the C (whole TU,
//! shipping contraction) by tests/tu_ftCo_Wait.rs.
use crate::types::*;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

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

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    ext_fns! {
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_Catch_CheckInput(g: G) -> bool = crate::tu::ftCo_Catch::ftCo_Catch_CheckInput;
        ftCo_AttackS4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS4::ftCo_AttackS4_CheckInput;
        ftCo_AttackHi4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi4::ftCo_AttackHi4_CheckInput;
        ftCo_AttackLw4_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw4::ftCo_AttackLw4_CheckInput;
        ftCo_AttackS3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackS3::ftCo_AttackS3_CheckInput;
        ftCo_AttackHi3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackHi3::ftCo_AttackHi3_CheckInput;
        ftCo_AttackLw3_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackLw3::ftCo_AttackLw3_CheckInput;
        ftCo_Attack1_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack1::ftCo_Attack1_CheckInput;
        ftCo_80099794(g: G) -> bool = crate::tu::ftCo_Escape::ftCo_80099794;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftFx_AppealS_CheckInput(g: G) -> bool = |_| todo!("TODO(port) ftFx_AppealS_CheckInput (ftfoxappeals.c)");
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        ftCo_Jump_CheckInput(g: G) -> bool = crate::tu::ftCo_Jump::ftCo_Jump_CheckInput;
        ftCo_Dash_CheckInput(g: G) -> bool = crate::tu::ftCo_Dash::ftCo_Dash_CheckInput;
        ftCo_800D5FB0(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Squat::ftCo_800D5FB0(g as _) as _ } };
        ftCo_Turn_CheckInput(g: G) -> bool = crate::tu::ftCo_Turn::ftCo_Turn_CheckInput;
        ftCo_Walk_CheckInput(g: G) -> bool = crate::tu::ftCo_Walk::ftCo_Walk_CheckInput;
        ftCo_DownSpot_Enter(g: G) = crate::tu::ftCo_DownSpot::ftCo_DownSpot_Enter;
        ftCo_8008A7A8(g: G, ws: *mut core::ffi::c_void) = |g, ws| crate::tu::ftwaitanim::ftCo_8008A7A8(g, ws as *const _);
        ft_80084F3C(g: G) = |_| todo!("ft_80084F3C: installed by melee_link");
        ftColl_8007AEE0(g: G) = |_| todo!("ftColl_8007AEE0: installed by melee_link");
        ft_80084280(g: G) = crate::tu::ft_081b::ft_80084280;
    }
}

pub fn ftCo_Wait_Anim(g: G) {
    melee_core::ct!("ftCo_Wait_Anim");
    let f = fp(g);
    if f.x2224_b2() != 0 {
        ext::ftCo_DownSpot_Enter(g);
    } else {
        let ws = unsafe { (*f.ft_data).x24 };
        ext::ftCo_8008A7A8(g, ws as *mut core::ffi::c_void);
    }
}

pub fn ftCo_Wait_IASA(g: G) {
    melee_core::ct!("ftCo_Wait_IASA");
    if ext::ftCo_SpecialS_CheckInput(g) {
        return;
    }
    if ext::ftCo_Attack100_CheckInput(g) {
        return;
    }
    if ext::ftCo_800D6824(g) {
        return;
    }
    if ext::ftCo_800D68C0(g) {
        return;
    }
    if ext::ftCo_Catch_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackS4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackHi4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackLw4_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackS3_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackHi3_CheckInput(g) {
        return;
    }
    if ext::ftCo_AttackLw3_CheckInput(g) {
        return;
    }
    if ext::ftCo_Attack1_CheckInput(g) {
        return;
    }
    if ext::ftCo_80099794(g) {
        return;
    }
    if ext::ftCo_80091A4C(g) {
        return;
    }
    if ext::ftFx_AppealS_CheckInput(g) {
        return;
    }
    if ext::ftCo_800DE9D8(g) {
        return;
    }
    if ext::ftCo_Jump_CheckInput(g) {
        return;
    }
    if ext::ftCo_Dash_CheckInput(g) {
        return;
    }
    if ext::ftCo_800D5FB0(g) {
        return;
    }
    if ext::ftCo_Turn_CheckInput(g) {
        return;
    }
    if ext::ftCo_Walk_CheckInput(g) {
        return;
    }
}

pub fn ftCo_Wait_Phys(g: G) {
    melee_core::ct!("ftCo_Wait_Phys");
    ext::ft_80084F3C(g);
    ext::ftColl_8007AEE0(g);
}

pub fn ftCo_Wait_Coll(g: G) {
    melee_core::ct!("ftCo_Wait_Coll");
    ext::ft_80084280(g);
}
