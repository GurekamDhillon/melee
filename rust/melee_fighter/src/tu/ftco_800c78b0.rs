//! src/melee/ft/ftCo_800C78B0.c: CaptureLikelike (swallowed by a Like Like): entry
//! `ftCo_800C78B0`, its callbacks, the spit-out hit `ftCo_800C7B0C` and `ftCo_800C7C60`.
//!
//! Fused sites (`tools/rust-port/fma_sites.py`, ftCo_800C78B0: 2): `x744 * (x748 - handicap) +
//! x740` and `percent * x754 + tmp2`, both -> `fmadds`. `dmg.x1848_kb_angle` is an int: the
//! float angle converts saturating (gwtool), as Rust `as`. Checked against the C (whole TU
//! included, shipping contraction) by tests/tu_ftco_800c78b0.rs.

use super::ftco_800c7070::{ext as e70, fp, G};
use super::ftco_800c7590::{ext as e90, fn_800c7890, ftco_release_item_and_victim};
use crate::types::*;
use melee_core::fp::fmadds;

/// `ftCo_800C78B0(gobj, owner)`: enter CaptureLikelike inside item `owner`.
pub fn ftco_800c78b0(g: G, owner: *mut HSD_GObj) {
    melee_core::ct!("ftco_800c78b0");
    ftco_release_item_and_victim(g);
    e70::Fighter_ChangeMotionState(g, ftCo_MS_CaptureLikelike, 0x1080, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ext::ftAnim_8006EBA4(g);
    let f = fp(g);
    f.accessory1_cb = Some(fn_800c7ae0);
    f.death2_cb = Some(fn_800c7890);
    f.take_dmg_cb = Some(ftco_800c7a30_cb);
    f.set_x221D_b5(1);
    f.mv.co.capturelikelike.x0 = owner;
    f.mv.co.capturelikelike.x4 = 0;
    e90::ftCommon_8007E2FC(fp(g));
    let cd = crate::hooks::common_data();
    let f = fp(g);
    let pid = f.player_idx as i32;
    let tmp1 = cd.x74C * (cd.x750 - (e90::Player_80033BB8(pid) + 1) as f32);
    let mut tmp2 = fmadds(cd.x744, cd.x748 - e90::Player_GetHandicap(pid) as f32, cd.x740);
    tmp2 += tmp1;
    e90::ftCommon_InitGrab(f, 0, fmadds(f.dmg.x1830_percent, cd.x754, tmp2));
    let f = fp(g);
    f.x1988 = 2;
    f.set_invisible(1);
    e90::ftCommon_8007E2F4(f, 0x1FF);
}

/// `ftCo_800C7A30(gobj)`: `it_802DBA68(capturelikelike.x0)`.
pub fn ftco_800c7a30(g: G) {
    melee_core::ct!("ftco_800c7a30");
    ext::it_802DBA68(unsafe { fp(g).mv.co.capturelikelike.x0 })
}

unsafe extern "C" fn ftco_800c7a30_cb(g: G) {
    ftco_800c7a30(g)
}

/// `ftCo_CaptureLikelike_Anim(gobj)`: mash; once the timer runs out, tell the Like Like (once,
/// flag in the int at mv +4).
pub fn ftco_capture_likelike_anim(g: G) {
    melee_core::ct!("ftco_capture_likelike_anim");
    let cd = crate::hooks::common_data();
    let f = fp(g);
    f.grab_timer -= cd.x758;
    e90::ftCommon_GrabMash(f, cd.x75C);
    let f = fp(g);
    // `*(int*) &fp->mv.co.mushroom.x4`: the same word as capturelikelike.x4.
    if unsafe { f.mv.co.capturelikelike.x4 } == 0 && f.grab_timer <= 0.0 {
        ext::it_802DB9F4(unsafe { f.mv.co.capturelikelike.x0 });
        fp(g).mv.co.capturelikelike.x4 = 1;
    }
}

/// `ftCo_CaptureLikelike_IASA(gobj)`: empty.
pub fn ftco_capture_likelike_iasa(_g: G) { melee_core::ct!("ftco_capture_likelike_iasa");}
/// `ftCo_CaptureLikelike_Phys(gobj)`: empty.
pub fn ftco_capture_likelike_phys(_g: G) { melee_core::ct!("ftco_capture_likelike_phys");}
/// `ftCo_CaptureLikelike_Coll(gobj)`: empty.
pub fn ftco_capture_likelike_coll(_g: G) { melee_core::ct!("ftco_capture_likelike_coll");}

/// `fn_800C7AE0(gobj)`: accessory1_cb, follow the item's position.
pub unsafe extern "C" fn fn_800c7ae0(g: G) {
    melee_core::ct!("fn_800c7ae0");
    let f = fp(g);
    let item = unsafe { f.mv.co.captureleadead.x0 };
    ext::it_8026B294(item, &mut f.cur_pos);
}

/// `ftCo_800C7B0C(gobj, pos, last_pos, hit_desc, angle)`: spat out: place and take the hit.
pub fn ftco_800c7b0c(g: G, arg1: &Vec3, arg2: &Vec3, arg3: *mut lbColl_80008D30_arg1, arg4: f32) {
    melee_core::ct!("ftco_800c7b0c");
    let mut hit = HitCapsule::zeroed();
    let f = fp(g);
    f.coll_data.last_pos = *arg2;
    ext::mpColl_80043670(&mut fp(g).coll_data);
    let mut angle = arg4;
    fp(g).coll_data.cur_pos = *arg1;
    ext::mpColl_800471F8(&mut fp(g).coll_data);
    let f = fp(g);
    f.cur_pos = f.coll_data.cur_pos;
    ext::lbColl_80008D30(&mut hit, arg3);
    ext::ftColl_800788D4(g);
    if angle < 0.0 {
        angle += 360.0;
    }
    let f = fp(g);
    angle = if f.facing_dir < 0.0 { angle } else { 180.0 - angle };
    let n = hit.unk_count;
    let kb = ext::ftColl_80079EA8(f, &mut hit, n);
    let f = fp(g);
    f.dmg.kb_applied = kb;
    f.dmg.x1848_kb_angle = melee_core::ppc::fctiwz_f32(angle);
    f.dmg.facing_dir_1 = f.facing_dir;
    f.dmg.x184c_damaged_hurtbox = 0;
    f.dmg.x1854_collpos = f.cur_pos;
    f.dmg.x1860_element = hit.element;
    ext::Fighter_UnkTakeDamage_8006CC30(fp(g), hit.damage);
    ext::ftCo_Damage_CalcKnockback(fp(g));
    ext::ftCo_8008E908(g, 0.0);
}

/// `ftCo_800C7C60(gobj, damage)`: `Fighter_TakeDamage_8006CC7C(fp, damage)`.
pub fn ftco_800c7c60(g: G, damage_amount: i32) {
    melee_core::ct!("ftco_800c7c60");
    ext::Fighter_TakeDamage_8006CC7C(fp(g), damage_amount as f32)
}

pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    use super::super::ftco_800c7590::ext::hooks;
    use crate::types::*;
    type G = *mut HSD_GObj;
    hooks! {
        FTANIM_8006EBA4 ftAnim_8006EBA4(g: G) => unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g) };
        IT_802DBA68 it_802DBA68(i: *mut Item_GObj) => unsafe { melee_items::tu::itlikelike::it_802DBA68(i as _) };
        IT_802DB9F4 it_802DB9F4(i: *mut Item_GObj) => unsafe { melee_items::tu::itlikelike::it_802DB9F4(i as _) };
        IT_8026B294 it_8026B294(i: *mut Item_GObj, pos: &mut Vec3) => melee_items::itlib::it_8026b294(unsafe { &*((*i).user_data as *const Item) }, pos);
        MPCOLL_80043670 mpColl_80043670(c: &mut CollData) => unsafe { melee_world::tu::mpcoll::mpColl_80043670(c as *mut CollData as _) };
        MPCOLL_800471F8 mpColl_800471F8(c: &mut CollData) -> bool => unsafe { melee_world::tu::mpcoll::mpColl_800471F8(c as *mut CollData as _) != 0 };
        LBCOLL_80008D30 lbColl_80008D30(h: &mut HitCapsule, a: *mut lbColl_80008D30_arg1) => unsafe { melee_runtime::tu::lbcollision::lbColl_80008D30(h as *mut HitCapsule as _, a as _) };
        FTCOLL_800788D4 ftColl_800788D4(g: G) => crate::tu::ftcoll::ftcoll_800788d4(g);
        FTCOLL_80079EA8 ftColl_80079EA8(fp: &mut Fighter, h: &mut HitCapsule, n: u32) -> f32 => crate::tu::ftcoll::ftcoll_80079ea8(fp, h, n);
        FIGHTER_UNK_TAKE_DAMAGE_8006CC30 Fighter_UnkTakeDamage_8006CC30(fp: &mut Fighter, d: f32) => unsafe { crate::tu::fighter::Fighter_UnkTakeDamage_8006CC30(fp, d) };
        FTCO_DAMAGE_CALC_KNOCKBACK ftCo_Damage_CalcKnockback(fp: &mut Fighter) => crate::tu::ftCo_Damage::ftCo_Damage_CalcKnockback(fp);
        FTCO_8008E908 ftCo_8008E908(g: G, f: f32) => crate::tu::ftCo_Damage::ftCo_8008E908(g as _, f);
        FIGHTER_TAKE_DAMAGE_8006CC7C Fighter_TakeDamage_8006CC7C(fp: &mut Fighter, d: f32) => unsafe { crate::tu::fighter::Fighter_TakeDamage_8006CC7C(fp, d) };
    }
}
