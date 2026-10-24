#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_HammerWait.c (hammer item: wait state, hammer
//! timers, colanim/headless helpers). Every function of the TU; calls out of it go through `ext`
//! (unported ones panic until rewired; ported ones default to the port). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_HammerWait.rs.

use crate::types::*;

pub type G = *mut HSD_GObj;
type I = *mut Item_GObj;

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
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftColl_8007AFF8(g: G) = |g: G| { unsafe { let _ = crate::tu::ftcoll::ftcoll_8007aff8(g as _); } };
        ft_800892A0(g: G) = |_| todo!("TODO(port) ft_800892A0 (ft_0892.c)");
        ft_80089824(g: G) = crate::ft0892::ft_80089824;
        it_8027B070(i: I, g: G) = |i: I, g: G| { unsafe { let _ = melee_items::tu::it_279C::it_8027B070(i as _, g as _); } };
        ftCo_800C5A50(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerJump::ftCo_800C5A50(g as _) as _ } };
        ftCo_800C60C8(g: G) -> bool = crate::tu::ftco_0c60::ftco_800c60c8;
        ftCo_800C576C(g: G) -> bool = |g: G| -> bool { unsafe { crate::tu::ftCo_HammerTurn::ftCo_800C576C(g as _) as _ } };
        ftCo_800C55CC(g: G) -> bool = crate::tu::ftCo_HammerWalk::ftCo_800C55CC;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_8008403C(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_8008403c;
        ftCo_800C5D34(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_HammerFall::ftCo_800C5D34(g as _); } };
        it_80293E34(i: I) = |i: I| { unsafe { let _ = melee_items::tu::ithammer::it_80293E34(i as _); } };
        ftCommon_8007ECD4(f: &mut Fighter, a: i32) = crate::ftcommon2::ftcommon_8007ecd4;
        Item_8026A8EC(i: I) = |i: I| { unsafe { let _ = melee_items::tu::item::Item_8026A8EC(i as _); } };
        ft_80088110(f: &mut Fighter) = crate::ft0877::ft_80088110;
        ftCo_800C0694(f: &mut Fighter) -> i32 = crate::ft0df::ftco_800c0694;
        ftCommon_8007D92C(g: G) = crate::ftcommon3::ftcommon_8007d92c;
        ftCo_800C0200(f: &mut Fighter, a: i32) = crate::tu::ftcolanim::ftco_800c0200;
        itGetKind(i: I) -> i32 = |i: I| -> i32 { unsafe { melee_items::tu::it_26B1::itGetKind(i as _) as _ } };
        it_80293DCC(i: I) -> i32 = |i: I| -> i32 { unsafe { melee_items::tu::ithammer::it_80293DCC(i as _) as _ } };
        pl_800403C0(slot: i32, sub: i32) = |slot: i32, sub: i32| { unsafe { let _ = crate::tu::plbonuslib::pl_800403C0(slot as _, sub as _); } };
        ft_800880AC(f: &mut Fighter) = crate::ft0877::ft_800880ac;
        ftCo_800BFFD0(f: &mut Fighter, a: i32, b: bool) -> bool = |f: &mut Fighter, a, b: bool| crate::tu::ftcolanim::ftco_800bffd0(f, a, b as i32);
        it_8026C16C(i: I, headless: bool) = |i: I, headless: bool| { unsafe { let _ = melee_items::tu::it_26B1::it_8026C16C(i as _, headless as _); } };
    }
}

pub fn ftCo_800C4E4C(g: G) -> bool {
    if ext::ft_8008A1FC(g) {
        ftCo_800C4ED8(g);
        return true;
    }
    false
}

pub fn ftCo_800C4E94(f: &mut Fighter) {
    ftCo_800C5408(f);
    ftCo_800C5588(f);
    ext::ftCommon_8007EBAC(f, 1, 0);
}

/// `inlineA0`: enter HammerWait, keeping the hammer animation frame when already hammering.
fn inline_a0(g: G) {
    let f = fp(g);
    let flags = ftCo_800C54C4(f);
    let start = ftCo_800C548C(f);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_HammerWait as i32, flags, start, 1.0, 0.0, core::ptr::null_mut());
    ftCo_800C4E94(fp(g));
}

pub fn ftCo_800C4ED8(g: G) {
    inline_a0(g);
}

/// `inlineB0`.
fn inline_b0(g: G) {
    let f = fp(g);
    if f.x2338.x != 0 {
        ext::ftColl_8007AFF8(g);
    } else if f.cur_anim_frame >= 0.0 && f.cur_anim_frame < f.frame_speed_mul {
        ext::ft_800892A0(g);
        ext::ft_80089824(g);
        ext::it_8027B070(fp(g).item_gobj, g);
    }
}

pub fn ftCo_800C4F64(g: G) {
    inline_b0(g);
}

pub fn ftCo_HammerWait_Anim(g: G) {
    inline_b0(g);
}

pub fn ftCo_HammerWait_IASA(g: G) {
    if ext::ftCo_800C5A50(g) { return; }
    if ext::ftCo_800C60C8(g) { return; }
    if ext::ftCo_800C576C(g) { return; }
    if ext::ftCo_800C55CC(g) { return; }
}

pub fn ftCo_HammerWait_Phys(g: G) {
    ext::ft_80084F3C(g);
}

pub fn ftCo_HammerWait_Coll(g: G) {
    ext::ft_8008403C(g, ext::ftCo_800C5D34);
}

pub fn ftCo_800C511C(g: G) {
    let f = fp(g);
    if f.motion_id != ftCo_MS_LightGet as i32 {
        if f.x2330.x != 0 {
            f.x2330.x -= 1;
        }
        if f.x2330.y != 0 && ftCo_800C53E4(f) {
            f.x2330.y -= 1;
            if f.x2330.y == 0 {
                f.x2338.x = 1;
                ext::it_80293E34(f.item_gobj);
                ext::ftCommon_8007ECD4(fp(g), 1);
            }
        }
        let f = fp(g);
        if f.x2330.x == 0 {
            let mut b = false;
            ext::Item_8026A8EC(f.item_gobj);
            ext::ft_80088110(fp(g));
            if ext::ftCo_800C0694(fp(g)) == 106 {
                b = true;
            }
            if ftCo_800C53E4(fp(g)) {
                ext::ftCommon_8007D92C(g);
            }
            if b {
                ext::ftCo_800C0200(fp(g), 106);
            }
        }
        let f = fp(g);
        if f.x2338.y != 0 {
            f.x2338.y -= 1;
        }
    }
}

pub fn ftCo_800C5240(g: G) -> bool {
    let f = fp(g);
    !f.item_gobj.is_null() && ext::itGetKind(f.item_gobj) == It_Kind_Hammer as i32
}

pub fn ftCo_800C5284(g: G) {
    let f = fp(g);
    f.x2330.x = crate::hooks::common_data().x6AC;
    f.x2330.y = ext::it_80293DCC(f.item_gobj);
    let f = fp(g);
    f.x2338.x = 0;
    f.x2338.y = 0;
    if f.x2330.y != 0 {
        ext::pl_800403C0(f.player_id as i32, f.is_sub_fighter() as i32);
    }
    ext::ft_800880AC(fp(g));
}

pub fn ftCo_800C52F4(g: G) {
    ftCo_800C5284(g);
    if fp(g).ground_or_air == GA_Ground {
        inline_a0(g);
    } else {
        ext::ftCo_800C5D34(g);
    }
}

pub fn ftCo_800C53E4(f: &Fighter) -> bool {
    f.motion_id >= ftCo_MS_HammerWait as i32 && f.motion_id <= ftCo_MS_HammerLanding as i32
}

pub fn ftCo_800C5408(f: &mut Fighter) {
    if ext::ftCo_800C0694(f) != 106 {
        ext::ftCo_800BFFD0(f, 106, false);
    }
}

pub fn ftCo_800C544C(f: &mut Fighter) {
    if ext::ftCo_800C0694(f) == 106 {
        ext::ftCo_800C0200(f, 106);
    }
}

pub fn ftCo_800C548C(f: &Fighter) -> f32 {
    if ftCo_800C53E4(f) { f.cur_anim_frame } else { 0.0 }
}

pub fn ftCo_800C54C4(f: &Fighter) -> u32 {
    if ftCo_800C53E4(f) {
        Ft_MF_SkipHit | Ft_MF_SkipModel | Ft_MF_SkipMatAnim | Ft_MF_SkipRumble | Ft_MF_SkipColAnim
            | Ft_MF_UpdateCmd | Ft_MF_Unk19
    } else {
        Ft_MF_None
    }
}

pub fn ftCo_800C5500(g: G) {
    ftCo_800C544C(fp(g));
    ext::ft_80088110(fp(g));
}

pub fn ftCo_800C554C(f: &mut Fighter) {
    f.x2338.y = crate::hooks::common_data().x6B8;
}

pub fn ftCo_800C555C(g: G) {
    ext::it_8026C16C(fp(g).item_gobj, false);
}

pub fn ftCo_800C5588(f: &mut Fighter) {
    ext::it_8026C16C(f.item_gobj, true);
    f.set_x2219_b4(1);
}
