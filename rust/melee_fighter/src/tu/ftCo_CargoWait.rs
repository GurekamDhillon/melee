#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CargoWait.c (DK cargo hold idle). Every
//! function of the TU; calls out go through `ext`. Checked by tests/tu_ftCo_CargoWait.rs.

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
        ft_8008A1FC(g: G) -> bool = crate::ft084e::ft_8008a1fc;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCo_8009C5A4(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_Shouldered::ftCo_8009C5A4(g as _, msid as _); } };
        ftCo_8009C640(g: G, msid: i32) = |g: G, msid: i32| { unsafe { let _ = crate::tu::ftCo_Shouldered::ftCo_8009C640(g as _, msid as _); } };
        ftCo_8009BF3C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoThrow::ftCo_8009BF3C(g as _) as _ } };
        ftCo_8009BB1C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_CargoJump::ftCo_8009BB1C(g as _) as _ } };
        ftCo_8009C4F8(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftco_09c4::ftCo_8009C4F8(g as _) as _ } };
        ftCo_8009B818(g: G) -> bool = crate::tu::ftCo_CargoTurn::ftCo_8009B818;
        ftCo_8009B67C(g: G) -> bool = crate::tu::ftCo_CargoWalk::ftCo_8009B67C;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_8008403C(g: G, cb: HSD_GObjEvent) = |g: G, cb: HSD_GObjEvent| { unsafe { let _ = crate::bridge::with_ev(cb, |ev| crate::tu::ft_081b::ft_8008403c(g as _, ev)); } };
        ftCo_8009BC58(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_CargoFall::ftCo_8009BC58(g as _); } };
    }
}

/// `ftCo_8009BC58` as the C `HSD_GObjEvent` handed to `ft_8008403C`.
pub unsafe extern "C" fn ftCo_8009BC58_cb(g: *mut HSD_GObj) {
    melee_core::ct!("ftCo_8009BC58_cb");
    ext::ftCo_8009BC58(g)
}

pub fn ftCo_8009B4D0(g: G) -> bool {
    melee_core::ct!("ftCo_8009B4D0");
    if ext::ft_8008A1FC(g) {
        ftCo_8009B518(g);
        return true;
    }
    false
}

/// `inlineA0`: re-enter the cargo-hold wait state and put the victim in Shouldered state 266.
fn inline_a0(g: G, cb: fn(G, i32)) {
    let f = fp(g);
    let ms = unsafe { (*f.x2CC).x4_motion_state };
    ext::Fighter_ChangeMotionState(g, ms, Ft_MF_None, 0.0, 1.0, 0.0, core::ptr::null_mut());
    cb(fp(g).victim_gobj, 266);
}

pub fn ftCo_8009B518(g: G) {
    melee_core::ct!("ftCo_8009B518");
    inline_a0(g, ext::ftCo_8009C5A4);
}

pub fn ftCo_8009B56C(g: G) {
    melee_core::ct!("ftCo_8009B56C");
    inline_a0(g, ext::ftCo_8009C640);
}

pub fn ftCo_CargoWait_Anim(_g: G) { melee_core::ct!("ftCo_CargoWait_Anim");}

pub fn ftCo_CargoWait_IASA(g: G) {
    melee_core::ct!("ftCo_CargoWait_IASA");
    if ext::ftCo_8009BF3C(g) { return; }
    if ext::ftCo_8009BB1C(g) { return; }
    if ext::ftCo_8009C4F8(g) { return; }
    if ext::ftCo_8009B818(g) { return; }
    if ext::ftCo_8009B67C(g) { return; }
}

pub fn ftCo_CargoWait_Phys(g: G) {
    melee_core::ct!("ftCo_CargoWait_Phys");
    ext::ft_80084F3C(g);
}

pub fn ftCo_CargoWait_Coll(g: G) {
    melee_core::ct!("ftCo_CargoWait_Coll");
    ext::ft_8008403C(g, Some(ftCo_8009BC58_cb));
}
