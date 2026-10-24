#![allow(non_snake_case)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_Barrel.c (DK barrel cannon: inside, fire out),
//! plus the ftCommon/inlines.h helpers it uses (`ftCo_ReleaseItemAndVictim`,
//! `ftCo_Barrel_ApplyKnockback`). Every function of the TU; calls out of it go through `ext`
//! (unported ones panic until rewired; ported ones default to the port). Checked against the C
//! (whole TU, shipping contraction) by tests/tu_ftCo_Barrel.rs.

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
    type I = *mut Item_GObj;
    ext_fns! {
        ftCommon_8007DB58(g: G) = |g: G| { unsafe { let _ = crate::ftcommon3::ftcommon_8007db58(g as _); } };
        ftCo_8009750C(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Lift::ftCo_8009750C(g as _); } };
        ftCo_800DD168(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_Throw::ftCo_800DD168(g as _); } };
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        it_802960B8(i: I) -> i32 = |i: I| -> i32 { unsafe { melee_items::tu::ittarucann::it_802960B8(i as _) as _ } };
        it_802960CC(i: I) -> i32 = |i: I| -> i32 { unsafe { melee_items::tu::ittarucann::it_802960CC(i as _) as _ } };
        it_80295F38(i: I) = |i: I| { unsafe { let _ = melee_items::tu::ittarucann::it_80295F38(i as _); } };
        it_8026C334(i: I, pos: *mut Vec3) = |i: I, pos: *mut Vec3| { unsafe { let _ = melee_items::tu::it_26B1::it_8026C334(i as _, pos as _); } };
        itGetOwner(i: I) -> G = |i: I| -> G { unsafe { melee_items::tu::it_26B1::itGetOwner(i as _) as _ } };
        ftCommon_8007D5D4(fp: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_8007E2FC(g: G) = |g| crate::ftcommon2::ftcommon_8007e2fc(super::fp(g));
        ftCommon_8007E2F4(fp: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftCommon_8007EFC0(fp: &mut Fighter, v: u32) = crate::ftcommon2::ftcommon_8007efc0;
        ftColl_800787B4(i: I, g: G, v: i32) = |i: I, g: G, v: i32| { unsafe { let _ = crate::tu::ftcoll::ftcoll_800787b4(i as _, g as _, v as _); } };
        ftLib_80086960(g: G) -> bool = crate::ftlib::ftlib_80086960;
        ftColl_8007B8CC(fp: *mut Fighter, owner: G) = |fp: *mut Fighter, owner: G| { unsafe { let _ = crate::ft0df::ftcoll_8007b8cc(&mut *(fp as *mut _), owner as _); } };
        ftLib_800866DC(g: G, v: &mut Vec3) = crate::ftlib::ftlib_800866dc;
        // rp3 mpColl_80043680
        mpColl_80043680(c: *mut CollData, v: *mut Vec3) = |c: *mut CollData, v: *mut Vec3| unsafe { melee_world::tu::mpcoll::mpColl_80043680(c, v as _); };
        // rp3-end
        lbColl_80008D30(hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1) = |hit: *mut HitCapsule, a: *mut lbColl_80008D30_arg1| { unsafe { let _ = melee_runtime::tu::lbcollision::lbColl_80008D30(hit as _, a as _); } };
        ftColl_80079EA8(fp: *mut Fighter, hit: *mut HitCapsule, n: u32) -> f32 = |fp: *mut Fighter, hit: *mut HitCapsule, n: u32| -> f32 { unsafe { crate::tu::ftcoll::ftcoll_80079ea8(&*(fp as *const _), &*(hit as *const _), n as _) } };
        ftCo_8008DCE0(g: G, a: i32, facing: f32) = |g: G, a: i32, facing: f32| { unsafe { let _ = crate::tu::ftCo_Damage::ftCo_8008DCE0(g as _, a as _, facing as _); } };
    }
}

/// `ftCo_ReleaseItemAndVictim` (ftCommon/inlines.h).
pub fn ftCo_ReleaseItemAndVictim(g: G) {
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

/// `ftCo_Barrel_ApplyKnockback` (ftCommon/inlines.h).
pub fn ftCo_Barrel_ApplyKnockback(f: &mut Fighter, mut kb_angle: f32, hit: *mut HitCapsule) {
    if kb_angle < 0.0 {
        kb_angle += 360.0;
    }
    kb_angle = if f.facing_dir < 0.0 { kb_angle } else { 180.0 - kb_angle };
    let n = unsafe { (*hit).unk_count };
    f.dmg.kb_applied = ext::ftColl_80079EA8(f, hit, n);
    f.dmg.x1848_kb_angle = melee_runtime::cabi::f2i32(kb_angle as f64);
    f.dmg.facing_dir_1 = f.facing_dir;
    f.dmg.x184c_damaged_hurtbox = 0;
    f.dmg.x1854_collpos = f.cur_pos;
    f.dmg.x1860_element = unsafe { (*hit).element };
}

pub unsafe extern "C" fn ftCo_Barrel_Death2_Cb(g: G) {
    ftCo_Barrel_Take_Dmg_Cb(g);
}

pub fn ftCo_Barrel_Enter(g: G, item: *mut Item_GObj) {
    ftCo_ReleaseItemAndVictim(g);
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Barrel as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.death2_cb = Some(ftCo_Barrel_Death2_Cb);
    f.accessory1_cb = Some(ftCo_Barrel_Accessory1_Cb);
    f.take_dmg_cb = Some(ftCo_Barrel_Take_Dmg_Cb);
    f.set_x221D_b5(1);
    let x0 = ext::it_802960B8(item);
    fp(g).mv.co.barrel.x0 = x0;
    let x4 = ext::it_802960CC(item);
    fp(g).mv.co.barrel.x4 = x4;
    fp(g).mv.co.barrel.x8 = item;
    ext::ftCommon_8007D5D4(fp(g));
    ext::ftCommon_8007E2FC(g);
    let f = fp(g);
    f.x1988 = 2;
    f.set_invisible(1);
    ext::ftCommon_8007E2F4(fp(g), 0x1FF);
    ext::ftCommon_8007EFC0(fp(g), 1);
    ext::ftColl_800787B4(unsafe { fp(g).mv.co.barrel.x8 }, g, 0);
}

pub unsafe extern "C" fn ftCo_Barrel_Take_Dmg_Cb(g: G) {
    ext::it_80295F38(fp(g).mv.co.barrel.x8);
}

pub fn ftCo_Barrel_Anim(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.barrel.x0 != 0 {
            f.mv.co.barrel.x0 = f.mv.co.barrel.x0.wrapping_sub(1);
            if f.mv.co.barrel.x0 == 0 {
                ext::it_80295F38(f.mv.co.barrel.x8);
            }
        }
    }
}

pub fn ftCo_Barrel_IASA(g: G) {
    let f = fp(g);
    unsafe {
        if f.mv.co.barrel.x4 != 0 {
            f.mv.co.barrel.x4 = f.mv.co.barrel.x4.wrapping_sub(1);
        }
        if f.mv.co.barrel.x4 == 0 && f.input.pressed_buttons & (HSD_PAD_A | HSD_PAD_B) as u32 != 0 {
            ext::it_80295F38(f.mv.co.barrel.x8);
        }
    }
}

pub fn ftCo_Barrel_Phys(_g: G) {}

pub fn ftCo_Barrel_Coll(_g: G) {}

pub unsafe extern "C" fn ftCo_Barrel_Accessory1_Cb(g: G) {
    let f = fp(g);
    ext::it_8026C334(f.mv.co.barrel.x8, &mut f.cur_pos);
}

/// static `fn_800C9290` (installed as `x21EC`).
pub unsafe extern "C" fn fn_800C9290(g: G) {
    let owner = ext::itGetOwner(fp(g).mv.co.barrel.x8);
    if ext::ftLib_80086960(owner) {
        ext::ftColl_8007B8CC(fp(g), owner);
    }
}

pub fn ftCo_800C92E4(g: G, arg1: &Vec3, arg2: *mut Vec3, arg3: *mut lbColl_80008D30_arg1, kb_angle: f32) {
    let mut hit = SmallerHitCapsule::zeroed();
    {
        let jobj = unsafe { (*g).hsd_obj as *mut HSD_JObj };
        let v1 = unsafe { (*jobj).translate };
        let mut v2 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::ftLib_800866DC(g, &mut v2);
        let f = fp(g);
        f.cur_pos.x = (arg1.x + v1.x) - v2.x;
        f.cur_pos.y = (arg1.y + v1.y) - v2.y;
        f.cur_pos.z = 0.0;
    }
    ext::mpColl_80043680(&mut fp(g).coll_data, arg2);
    let hp = &mut hit as *mut SmallerHitCapsule as *mut HitCapsule;
    ext::lbColl_80008D30(hp, arg3);
    ftCo_Barrel_ApplyKnockback(fp(g), kb_angle, hp);
    ext::ftColl_800787B4(unsafe { fp(g).mv.co.barrel.x8 }, g, 0);
    fp(g).x21EC = Some(fn_800C9290);
    ext::ftCo_8008DCE0(g, 0x5B, 0.0);
}
