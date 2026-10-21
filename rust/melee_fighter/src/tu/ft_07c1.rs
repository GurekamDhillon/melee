//! Hand port of src/melee/ft/ft_07C1.c (all 5 functions: ft_8007C114 hammer release,
//! ft_8007C17C / ft_8007C224 thrown-fighter hitbox, ft_8007C2E0 thrown-fighter push,
//! ft_8007C4BC thrown-fighter vs fighter scan).
//!
//! Fused sites (`tools/rust-port/fma_sites.py`): 8, all in ft_8007C2E0 (each
//! `x98_atk_shield_kb.{x,y} += a * weight_ratio` -> `fmadds`). Checked against the C (cut
//! verbatim, shipping contraction) by tests/tu_ft_07c1.rs.
#![allow(static_mut_refs)]

use crate::hooks;
use crate::types::*;
use melee_core::fp::fmadds;

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
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    ext! {
        FTCO_800C555C ftCo_800C555C(g: G);
        LBCOLL_80008440 lbColl_80008440(hit: *mut HitCapsule);
        LB_8000B1CC lb_8000B1CC(jobj: *mut HSD_JObj, pos0: *mut Vec3, pos1: *mut Vec3);
        LBCOLL_8000ACFC lbColl_8000ACFC(victim: *mut Fighter, hit: *mut HitCapsule) -> bool;
        LBCOLL_80008688 lbColl_80008688(hit: *mut HitCapsule, kind: i32, victim: *mut Fighter) -> bool;
        FTCOLL_8007B868 ftColl_8007B868(g: G) -> i32;
        LBCOLL_80007AFC lbColl_80007AFC(a: *mut HitCapsule, b: *mut HitCapsule, sa: f32, sb: f32) -> bool;
        LBCOLL_800089B8 lbColl_800089B8(hit: *mut HitCapsule, victim: *mut Fighter);
    }
    /// `itGetKind`: the shared hook (default `melee_items::itlib::it_get_kind`).
    #[allow(non_snake_case)]
    pub fn itGetKind(g: *mut Item_GObj) -> i32 {
        crate::hooks::it_get_kind(g)
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// `ft_8007C114`: drop the hammer state if holding one, then clear `x2219_b4`.
pub fn ft_8007c114(g: G) {
    let f = fp(g);
    if !f.item_gobj.is_null() && ext::itGetKind(f.item_gobj) == It_Kind_Hammer as i32 {
        ext::ftCo_800C555C(g);
    }
    fp(g).set_x2219_b4(0);
}

/// `ft_8007C17C`: arm the thrown-fighter hitbox on the part named by `ft_data->x34`.
pub fn ft_8007c17c(g: G) {
    let f = fp(g);
    let x34 = unsafe { &*(*f.ft_data).x34 };
    f.x1064_thrownHitbox.x4 = 0;
    f.x1064_thrownHitbox.state = HitCapsule_Enabled;
    ext::lbColl_80008440(&mut f.x1064_thrownHitbox);
    let f = fp(g);
    let hit = &mut f.x1064_thrownHitbox;
    hit.jobj = unsafe { (*f.parts.add(x34.x0 as usize)).joint };
    hit.scale = x34.scale;
    hit.b_offset.z = 0.0;
    hit.b_offset.y = 0.0;
    hit.b_offset.x = 0.0;
    hit.set_x43_b1(0);
    hit.set_x43_b2(0);
    f.set_x2227_b2(0);
    ft_8007c224(g);
}

/// `ft_8007C224`: advance the thrown-fighter hitbox position.
pub fn ft_8007c224(g: G) {
    let hit: *mut HitCapsule = &mut fp(g).x1064_thrownHitbox;
    unsafe {
        match (*hit).state {
            HitCapsule_Enabled => {
                ext::lb_8000B1CC((*hit).jobj, &mut (*hit).b_offset, &mut (*hit).x4C);
                (*hit).x58 = (*hit).x4C;
                (*hit).state = HitCapsule_Unk2;
            }
            HitCapsule_Unk2 | HitCapsule_Unk3 => {
                if (*hit).state == HitCapsule_Unk2 {
                    (*hit).state = HitCapsule_Unk3;
                }
                (*hit).x58 = (*hit).x4C;
                ext::lb_8000B1CC((*hit).jobj, &mut (*hit).b_offset, &mut (*hit).x4C);
            }
            _ => {}
        }
    }
}

/// One half of ft_8007C2E0 (the C writes the same inline twice): `to` gets pushed by `from`.
/// `kb_axis` is the pair (from, to) of pos_delta; the sign test always uses (fp0, fp1).
fn push(from: &Fighter, to: &mut Fighter, same_x: bool, same_y: bool) {
    let mut w = from.co_attrs.weight / to.co_attrs.weight;
    if w > 1.0 {
        w = 1.0;
    }
    w *= hooks::common_data().hit_weight_mul;
    to.x98_atk_shield_kb.x = if same_x {
        fmadds(from.pos_delta.x - to.pos_delta.x, w, to.x98_atk_shield_kb.x)
    } else {
        fmadds(from.pos_delta.x, w, to.x98_atk_shield_kb.x)
    };
    to.x98_atk_shield_kb.y = if same_y {
        fmadds(from.pos_delta.y - to.pos_delta.y, w, to.x98_atk_shield_kb.y)
    } else {
        fmadds(from.pos_delta.y, w, to.x98_atk_shield_kb.y)
    };
}

/// `ft_8007C2E0(fp0, hit0, fp1, hit1)`. `fp0` and `fp1` are distinct fighters.
pub fn ft_8007c2e0(fp0: *mut Fighter, hit0: *mut HitCapsule, fp1: *mut Fighter, hit1: *mut HitCapsule) {
    if ext::lbColl_8000ACFC(fp1, hit0) {
        return;
    }
    ext::lbColl_80008688(hit1, 3, fp0);
    unsafe {
        if (*fp1).ground_or_air == GA_Air {
            let sx = (*fp0).pos_delta.x * (*fp1).pos_delta.x >= 0.0;
            let sy = (*fp0).pos_delta.y * (*fp1).pos_delta.y >= 0.0;
            push(&*fp0, &mut *fp1, sx, sy);
        }
    }
    ext::lbColl_80008688(hit0, 3, fp1);
    unsafe {
        if (*fp0).ground_or_air == 1 {
            let sx = (*fp0).pos_delta.x * (*fp1).pos_delta.x >= 0.0;
            let sy = (*fp0).pos_delta.y * (*fp1).pos_delta.y >= 0.0;
            push(&*fp1, &mut *fp0, sx, sy);
        }
    }
}

/// `ft_8007C4BC`: thrown fighter `g` against every later fighter in the list.
pub fn ft_8007c4bc(g: G) {
    let f = fp(g);
    if f.x2227_b2() == 0 {
        return;
    }
    if ext::ftColl_8007B868(g) == 2 {
        return;
    }
    let f = fp(g);
    if f.is_sub_fighter() != 0 {
        return;
    }
    let mut seen = false;
    let grab_attacker: G = if !f.victim_gobj.is_null() {
        f.victim_gobj
    } else if f.x221C_b6() != 0 {
        f.dmg.x1868_source
    } else {
        core::ptr::null_mut()
    };
    let fpp: *mut Fighter = f;
    let throw_hitbox: *mut HitCapsule = &mut f.x1064_thrownHitbox;
    let mut cur = hooks::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if g == cur {
            seen = true;
        } else if cur != grab_attacker {
            let fp1p = unsafe { (*cur).user_data as *mut Fighter };
            let fp1 = unsafe { &mut *fp1p };
            let skip = fp1.x2219_b1() != 0
                || fp1.x2227_b2() == 0
                || ext::ftColl_8007B868(cur) == 2
                || fp1.is_sub_fighter() != 0
                || (fp1.x221C_b6() != 0 && fp1.dmg.x1868_source == g);
            if !skip {
                let hit: *mut HitCapsule = &mut fp1.x1064_thrownHitbox;
                if fp1.x1064_thrownHitbox.state != HitCapsule_Disabled && seen {
                    let (s1, s0) = (fp1.x34_scale.y, unsafe { (*fpp).x34_scale.y });
                    if ext::lbColl_80007AFC(hit, throw_hitbox, s1, s0) {
                        ft_8007c2e0(fp1p, hit, fpp, throw_hitbox);
                    } else {
                        ext::lbColl_800089B8(throw_hitbox, fp1p);
                        ext::lbColl_800089B8(hit, fpp);
                    }
                }
            }
        }
        cur = unsafe { (*cur).next };
    }
}
