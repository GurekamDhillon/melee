#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_RunDirect.c (TARGET_PC build). Every function of the TU;
//! calls out of it go through `ext` (unported ones panic until rewired). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_RunDirect.rs.
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
        ftCo_Run_Anim(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Run::ftCo_Run_Anim(g as _); } };
        ftCo_Run_Phys(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Run::ftCo_Run_Phys(g as _); } };
        ftCo_Run_Coll(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Run::ftCo_Run_Coll(g as _); } };
        ftCo_SpecialS_CheckInput(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_SpecialS::ftCo_SpecialS_CheckInput(g as _) as _ } };
        ftCo_Attack100_CheckInput(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_Attack100_CheckInput;
        ftCo_800D6824(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D6824;
        ftCo_800D68C0(g: G) -> bool = crate::tu::ftCo_Attack100::ftCo_800D68C0;
        ftCo_800D8A38(g: G) -> bool = crate::tu::ftCo_Catch::ftCo_800D8A38;
        ftCo_AttackDash_CheckInput(g: G) -> bool = crate::tu::ftCo_AttackDash::ftCo_AttackDash_CheckInput;
        ftCo_AttackDash_SetMv0(g: G) = crate::tu::ftCo_AttackDash::ftCo_AttackDash_SetMv0;
        ftCo_80091A4C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Guard::ftCo_80091A4C(g as _) as _ } };
        ftCo_80091B90(g: G, n: i32) = |g: G, n: i32| { unsafe { let _ = crate::tu::ftCo_Guard::ftCo_80091B90(g as _, n as _); } };
        ftCo_80091B9C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Guard::ftCo_80091B9C(g as _); } };
        ftCo_800DE9D8(g: G) -> bool = crate::tu::ftCo_AppealS::ftCo_800DE9D8;
        fn_800CAF78(g: G) -> bool = crate::tu::ftCo_Jump::fn_800CAF78;
        fn_800CA698(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_Run::fn_800CA698(g as _) as _ } };
        ft_8008A244(g: G) -> bool = crate::ft084e::ft_8008a244;
    }
}

pub fn ftCo_RunDirect_Anim(g: G) {
    ext::ftCo_Run_Anim(g);
}

pub fn ftCo_RunDirect_IASA(g: G) {
    if ext::ftCo_SpecialS_CheckInput(g) { return; }
    if ext::ftCo_Attack100_CheckInput(g) { return; }
    if ext::ftCo_800D6824(g) { return; }
    if ext::ftCo_800D68C0(g) { return; }
    if ext::ftCo_800D8A38(g) { return; }
    if ext::ftCo_AttackDash_CheckInput(g) {
        ext::ftCo_AttackDash_SetMv0(g);
        return;
    }
    if ext::ftCo_80091A4C(g) {
        ext::ftCo_80091B90(g, cd().x410);
        ext::ftCo_80091B9C(g);
        return;
    }
    if ext::ftCo_800DE9D8(g) { return; }
    if ext::fn_800CAF78(g) { return; }
    if unsafe { fp(g).mv.ca.specials.grav } <= 0.0 && ext::fn_800CA698(g) { return; }
    if ext::ft_8008A244(g) { return; }
}

pub fn ftCo_RunDirect_Phys(g: G) {
    ext::ftCo_Run_Phys(g);
}

pub fn ftCo_RunDirect_Coll(g: G) {
    ext::ftCo_Run_Coll(g);
}
