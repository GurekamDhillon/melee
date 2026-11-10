//! Hand port of src/melee/ft/ft_0819.c (all 4 definitions: ft_80081938, ft_800819A8, the
//! static `inlineA0` item-overlap scan, ft_80081A00). No fused sites. Checked against the C
//! (cut verbatim, shipping contraction) by tests/tu_ft_0819.rs.
#![allow(static_mut_refs)]

use crate::hooks;
use crate::types::*;

pub type G = *mut HSD_GObj;

/// Calls out of this TU: settable fn pointers, panicking with a TODO when reached unset
/// (except where a ported default exists).
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        LB_80014638 lb_80014638(a: &lb_80014638_arg0_t, b: &lb_80014638_arg1_t) -> bool;
        FTCO_800C703C ftCo_800C703C(g: G);
    }
    /// Override for `it_8026BC68`; unset = `melee_items::itlib::it_8026bc68`.
    pub static mut IT_8026BC68: Option<fn(G) -> bool> = None;
    #[allow(non_snake_case)]
    pub fn it_8026BC68(g: G) -> bool {
        match unsafe { IT_8026BC68 } {
            Some(f) => f(g),
            None => melee_items::itlib::it_8026bc68(unsafe { &*((*g).user_data as *const Item) }),
        }
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// The ECB's left/right x around `coll_data.cur_pos` and `cur_pos.y + x17C`.
#[inline(always)]
fn ecb_span(f: &Fighter) -> Vec3 {
    let sp44 = f.coll_data.cur_pos;
    let (lx, rx) = (f.coll_data.ecb.left.x, f.coll_data.ecb.right.x);
    Vec3 { x: lx + sp44.x, y: rx + sp44.x, z: f.cur_pos.y + f.co_attrs.x17C }
}

/// `ft_80081938`: sets `dmg.x1930.xC` and copies it to `x0`.
pub fn ft_80081938(g: G) {
    melee_core::ct!("ft_80081938");
    let f = fp(g);
    f.dmg.x1930.xC = ecb_span(f);
    f.dmg.x1930.x0 = f.dmg.x1930.xC;
}

/// `ft_800819A8`: sets `dmg.x1930.x0`.
pub fn ft_800819a8(g: G) {
    melee_core::ct!("ft_800819a8");
    let f = fp(g);
    f.dmg.x1930.x0 = ecb_span(f);
}

/// `inlineA0` (static inline): marks every flagged item overlapping `dmg.x1930`.
fn inline_a0(g: G) {
    let mut cur = hooks::plink_head(HSD_GOBJ_PLINK_ITEM);
    while !cur.is_null() {
        if ext::it_8026BC68(cur) {
            let ip = unsafe { &mut *((*cur).user_data as *mut Item) };
            let f = fp(g);
            if ext::lb_80014638(&f.dmg.x1930, &ip.xB54) {
                ip.xCD0 = if ip.pos.x > f.cur_pos.x { -1.0 } else { 1.0 };
                ip.xCFC = g;
                ip.xCB0_source_ply = f.player_idx as i32;
                f.dmg.x1950 = 1;
            }
        }
        cur = unsafe { (*cur).next };
    }
}

/// `ft_80081A00`.
pub fn ft_80081a00(g: G) -> bool {
    melee_core::ct!("ft_80081a00");
    let f = fp(g);
    if f.x2219_b1() == 0 && f.x2223_b4() == 0 {
        f.set_x2223_b5(1);
        f.dmg.x1930.xC = ecb_span(f);
        inline_a0(g);
        let f = fp(g);
        if f.dmg.x1950 != 0 {
            ext::ftCo_800C703C(g);
            fp(g).dmg.x1950 = 0;
            return true;
        }
    }
    false
}
