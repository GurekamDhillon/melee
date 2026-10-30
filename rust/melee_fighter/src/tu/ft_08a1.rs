//! src/melee/ft/ft_08A1.c: the stick test (`ft_8008A1FC`, `ft_8008A244` + its inline; ported
//! and verified in `crate::ft084e`, re-exported here) and the "back to Wait" transitions
//! `ft_8008A2BC` / `ft_8008A324` / `ft_8008A348`. No float math, no fused sites.
//!
//! `crate::hooks::FT_8008A2BC` (the hook ft084e calls) still defaults to a panic; a later pass
//! can point it at [`ft_8008A2BC`].
#![allow(non_snake_case)]

use crate::hooks as h;
use crate::types::*;

pub use crate::ft084e::{ft_8008a1fc as ft_8008A1FC, ft_8008a244 as ft_8008A244,
                        ft_8008a244_inline as ft_8008A244_inline};

pub type G = *mut HSD_GObj;

/// Callees outside this TU with no port yet: settable, panic while unset.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    macro_rules! ext_fns {
        ($($hook:ident $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported (ft_08A1 ext)")),
                }
            }
        )*};
    }
    ext_fns! {
        FTMH_MS_389_80151018 ftMh_MS_389_80151018(g: G);
        FTCH_GRABUNK1_8015BC88 ftCh_GrabUnk1_8015BC88(g: G);
        FTCO_DOWNSPOT_ENTER ftCo_DownSpot_Enter(g: G);
        FTCO_800C4ED8 ftCo_800C4ED8(g: G);
        IT_802BDB94 it_802BDB94(i: G);
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, frame: f32, rate: f32, blend: f32, other: G);
        FTCO_8008A698 ftCo_8008A698(fp: &mut Fighter) -> bool;
        FTCO_8008A6D8 ftCo_8008A6D8(g: G, msid: i32);
        FTLK_ATTACKAIR_800EB3BC ftLk_AttackAir_800EB3BC(g: G);
        FTCL_INIT_8014919C ftCl_Init_8014919C(g: G);
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ft_8008A2BC`: end an action. Master/Crazy Hand have their own wait; everyone else Waits.
pub fn ft_8008A2BC(g: G) {
    melee_core::ct!("ft_8008A2BC");
    if h::ftlib_get_kind(g) == Ft_Kind_MasterH {
        ext::ftMh_MS_389_80151018(g);
    } else if h::ftlib_get_kind(g) == Ft_Kind_CrezyH {
        ext::ftCh_GrabUnk1_8015BC88(g);
    } else {
        ft_8008A348(g, 0.0);
    }
}

/// `ft_8008A324`: [`ft_8008A348`] with the default blend (-1).
pub fn ft_8008A324(g: G) {
    melee_core::ct!("ft_8008A324");
    ft_8008A348(g, -1.0);
}

/// `ft_8008A348`: into `ftCo_MS_Wait` (or DownSpot / the 800C5240 state), closing Peach's
/// parasol, then the Link / Young Link Wait hooks.
pub fn ft_8008A348(g: G, anim_blend: f32) {
    melee_core::ct!("ft_8008A348");
    let f = fp(g);
    if f.x2224_b2() != 0 {
        ext::ftCo_DownSpot_Enter(g);
        return;
    }
    if h::ftco_800c5240(g) {
        ext::ftCo_800C4ED8(g);
        return;
    }
    if f.ground_or_air == GA_Air {
        crate::ftcommon3::ftcommon_8007d7fc(f);
    }
    if f.kind == Ft_Kind_Peach && !f.item_gobj.is_null() && h::it_get_kind(f.item_gobj) == It_Kind_Peach_Parasol as i32 {
        ext::it_802BDB94(f.item_gobj);
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Wait as i32, Ft_MF_None as u32, 0.0, 1.0, anim_blend, core::ptr::null_mut());
    let f = fp(g);
    if ext::ftCo_8008A698(f) && h::ftdata_80085fd4_x8(f, ftCo_MS_DeadUpFall as i32) != 0 {
        ext::ftCo_8008A6D8(g, ftCo_MS_DeadUpFall as i32);
    }
    let f = fp(g);
    crate::ftcommon2::ftcommon_8007efc0(f, h::common_data().x5F0);
    match f.kind {
        Ft_Kind_Link => ext::ftLk_AttackAir_800EB3BC(g),
        Ft_Kind_CLink => ext::ftCl_Init_8014919C(g),
        _ => {}
    }
}
