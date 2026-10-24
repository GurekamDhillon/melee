#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
//! Native port of src/melee/ft/kinds/ftCommon/ftpickupitem.c (item pickup: the grab-box tests,
//! the nearest-item scan, the instant-use items and LightGet/HeavyGet). Every function of the TU
//! incl. the TARGET_PC additions (geno_consumable_collect, Geno "press" items, the m-ex pickup
//! dispatch); calls out go through `ext`. Fused sites (tools/rust-port/fma_sites.py): the grab
//! box x `facing_dir * off.x + cur_pos.x` (both scans) and the scan's squared distance; checked
//! by tests/tu_ftpickupitem.rs against the C (whole TU, shipping contraction).

use crate::types::*;
use melee_core::fp::fmadds;

pub type G = *mut HSD_GObj;
pub type I = *mut Item_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    use core::ffi::c_void;
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
    #[inline(always)]
    fn ip<'a>(i: I) -> &'a mut Item {
        unsafe { &mut *((*i).user_data as *mut Item) }
    }
    ext_fns! {
        Item_IsGrabbable(i: I) -> bool = |i| unsafe { melee_items::itlib3::item_is_grabbable(ip(i)) };
        it_8026B47C(i: I) -> i32 = |i| melee_items::itlib::it_8026b47c(ip(i));
        it_8026B4F0(i: I) -> bool = |i| melee_items::itlib::it_8026b4f0(ip(i));
        itIsHeavy(i: I) -> bool = |i| unsafe { melee_items::itlib::it_is_heavy(ip(i)) };
        it_8026B344(i: I, pos: &mut Vec3) = |i: I, pos: &mut Vec3| { unsafe { let _ = melee_items::tu::it_26B1::it_8026B344(i as _, pos as _); } };
        itGetGrabRangeX(i: I) -> f32 = |i| melee_items::itlib::it_get_grab_range_x(ip(i));
        itGetGrabRangeY(i: I) -> f32 = |i| melee_items::itlib::it_get_grab_range_y(ip(i));
        ftCo_IsCpuControlled(f: &mut Fighter) -> bool = |_| todo!("TODO(port) ftCo_IsCpuControlled (ftCo_0A01.c)");
        // `HSD_GObjPLinkHead[link]`.
        HSD_GObjPLinkHead(link: i32) -> G = crate::hooks::plink_head;
        Geno_ItemPressDist(i: I, f: &mut Fighter) -> f32 = |_, _| todo!("TODO(pc/geno) Geno_ItemPressDist (geno_game_items.inc)");
        it_8026B30C(i: I) -> i32 = crate::hooks::it_8026b30c;
        Geno_ItemFamilyEvent(i: I, a: i32, port: i32) = |_, _, _| todo!("TODO(pc/geno) Geno_ItemFamilyEvent");
        itGetKind(i: I) -> i32 = |i| melee_items::itlib::it_get_kind(ip(i)) as i32;
        Fighter_8006CF5C(f: &mut Fighter, v: i32) = |f: &mut Fighter, v: i32| { unsafe { let _ = crate::tu::fighter::Fighter_8006CF5C(f as _, v as _); } };
        Item_8026A8EC(i: I) = crate::hooks::item_8026a8ec;
        ftCo_800C4724(g: G) = |g: G| { unsafe { let _ = crate::tu::ftCo_WarpStar::ftCo_800C4724(g as _); } };
        ftCo_800C52F4(g: G) = crate::tu::ftCo_HammerWait::ftCo_800C52F4;
        ftCommon_8007FA58(g: G, i: I) = crate::ftcommon3::ftcommon_8007fa58;
        ftLib_800871A8(g: G, i: I) = crate::tu::ftlib::ftlib_800871a8;
        ftCo_800C88D4(g: G, v: i32, b: i32) = |g: G, v: i32, b: i32| { unsafe { let _ = crate::tu::ft_0c88::ftco_800c88d4(g as _, v as _, (b != 0 as _)); } };
        ftCo_CalcYScaledKnockback(a: f32, b: f32, c: f32) -> f32 = crate::tu::ftchangeparam::ftco_calc_y_scaled_knockback;
        // `Fighter_804D6524->x8` (PlCo size-scaling exponent).
        Fighter_804D6524_x8() -> f32 = || crate::tu::ftchangeparam::ext::d6524().x8;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { let _ = crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { let _ = crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCo_800974C4(g: G) = crate::tu::ftCo_Lift::ftCo_800974C4;
        pl_8003E17C(slot: i32, sub: i32, i: I) = |slot: i32, sub: i32, i: I| { unsafe { let _ = crate::tu::plbonuslib::pl_8003E17C(slot as _, sub as _, i as _); } };
        // `ftData_OnItemPickupExt[kind]`.
        ftData_OnItemPickupExt(kind: i32) -> Option<unsafe extern "C" fn(G, i32)> = |kind: i32| -> Option<unsafe extern "C" fn(G, i32)> { unsafe { crate::tu::ftdata::ftData_OnItemPickupExt[kind as usize] } };
        Mex_OnItemPickupDispatch(kind: i32, g: *mut c_void, i: *mut c_void, vanilla: *mut c_void) = |kind: i32, g: *mut c_void, i: *mut c_void, vanilla: *mut c_void| { unsafe { let _ = crate::platform::mex_dispatch2(crate::platform::MEX_EVENT_ON_ITEM_PICKUP, kind, g as _, i, vanilla); } };
        pl_8003E854(slot: i32, sub: i32, i: I) = |slot: i32, sub: i32, i: I| { unsafe { let _ = crate::tu::plbonuslib::pl_8003E854(slot as _, sub as _, i as _); } };
        Item_8026AB54(i: I, owner: G, part: i32) = crate::hooks::item_8026ab54;
        ft_800881D8(f: &mut Fighter, id: i32, vol: u8, pan: u8) = crate::ft0881::ft_800881d8;
        Geno_ItemPressCollect(i: I, port: i32) = |_, _| todo!("TODO(pc/geno) Geno_ItemPressCollect");
        ftAnim_IsFramesRemaining(g: G) -> bool = crate::hooks::ftanim_is_frames_remaining;
        ftDk_MS_341_800DF980(g: G) = |_| todo!("TODO(port) ftDk_MS_341_800DF980 (ftdonkeyheavywait0.c, melee_chara)");
        ftCo_80096D9C(g: G) = crate::tu::ftCo_Lift::ftCo_80096D9C;
        ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        ft_80084F3C(g: G) = crate::ft084e::ft_80084f3c;
        ft_800841B8(g: G, cb: fn(G)) = crate::tu::ft_081b::ft_800841b8;
        Item_8026ABD8(i: I, pos: &Vec3, f: f32) = crate::hooks::item_8026abd8;
        ftCo_800C5284(g: G) = crate::tu::ftCo_HammerWait::ftCo_800C5284;
        ftCo_8009750C(g: G) = crate::tu::ftCo_Lift::ftCo_8009750C;
        ftCo_Fall_Enter(g: G) = crate::tu::ftCo_Fall::ftCo_Fall_Enter;
    }
}

/// C callback-pointer forms (`HSD_GObjEvent`) of the functions stored in Fighter.
pub unsafe extern "C" fn ftCo_800974C4_cb(g: G) { ext::ftCo_800974C4(g) }
pub unsafe extern "C" fn ftpickupitem_80094DF8_cb(g: G) { ftpickupitem_80094DF8(g) }

/// `ftCheckThrowB3` (ft/inlines.h): test and clear throw_flags_b3.
fn check_throw_b3(f: &mut Fighter) -> bool {
    unsafe {
        if f.anon_2210.anon_0.throw_flags_b3() != 0 {
            f.anon_2210.anon_0.set_throw_flags_b3(0);
            true
        } else {
            false
        }
    }
}

/// The grab box `off` (x/y centre, z/w half-extents) around the fighter, widened by the item's
/// grab range, contains `pos`? Returns the box centre for the distance.
#[inline(always)]
fn in_box(f: &Fighter, off: &Quaternion, x_range: f32, y_range: f32, pos: &Vec3) -> Option<(f32, f32)> {
    let x1 = off.z;
    let x0 = fmadds(f.facing_dir, off.x, f.cur_pos.x);
    let y0 = f.cur_pos.y + off.y;
    let y1 = off.w;
    if x0 - x1 - x_range < pos.x
        && x_range + (x0 + x1) > pos.x
        && y0 - y1 - y_range < pos.y
        && y_range + (y0 + y1) > pos.y
    {
        Some((x0, y0))
    } else {
        None
    }
}

/// `geno_consumable_collect` (TARGET_PC): the Geno item-family event for consumables.
pub fn geno_consumable_collect(item: I, port: i32) {
    let k = ext::itGetKind(item);
    if k == It_Kind_Heart as i32
        || k == It_Kind_Tomato as i32
        || k == It_Kind_Foods as i32
        || k == It_Kind_Lucky_Egg as i32
        || k == It_Kind_WhispyHealApple as i32
        || k == It_Kind_WStar as i32
        || k == It_Kind_Hammer as i32
        || k == It_Kind_RabbitC as i32
        || k == It_Kind_MetalB as i32
        || k == It_Kind_Spycloak as i32
        || k == It_Kind_Coin as i32
    {
        ext::Geno_ItemFamilyEvent(item, 0, port);
    }
}

/// `ftpickupitem_80094150`: is `item_gobj` in this fighter's pickup box?
pub fn ftpickupitem_80094150(g: G, item_gobj: I) -> bool {
    if ext::Item_IsGrabbable(item_gobj)
        && (fp(g).x2222_b4() == 0 || ext::it_8026B47C(item_gobj) == 0)
        && (fp(g).item_gobj.is_null() || ext::it_8026B4F0(item_gobj))
    {
        let heavy = ext::itIsHeavy(item_gobj);
        let mut it_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::it_8026B344(item_gobj, &mut it_pos);
        let x_range = ext::itGetGrabRangeX(item_gobj);
        let y_range = ext::itGetGrabRangeY(item_gobj);
        let f = fp(g);
        let pickup = &f.x294_itPickup;
        let off = if heavy {
            pickup.gr_heavy_offset
        } else if f.ground_or_air == GA_Ground {
            pickup.gr_light_offset
        } else {
            pickup.air_light_offset
        };
        if in_box(f, &off, x_range, y_range, &it_pos).is_some() {
            return true;
        }
    }
    false
}

/// `ftpickupitem_800942A0`: the nearest pickable item (`flags` bit 0 light, bit 1 heavy).
pub fn ftpickupitem_800942A0(g: G, flags: u32) -> I {
    let f = fp(g);
    if ext::ftCo_IsCpuControlled(f) && fp(g).cpu.kind as i32 == 28 {
        return core::ptr::null_mut();
    }
    let mut min_dist_sq: f32 = 30000.0;
    let mut cur = ext::HSD_GObjPLinkHead(HSD_GOBJ_PLINK_ITEM);
    let mut result: I = core::ptr::null_mut();
    while !cur.is_null() {
        // TARGET_PC: Geno "press" script items, found by the same box (geno_game_items.inc).
        let press_dist = if flags & 1 != 0 { ext::Geno_ItemPressDist(cur, fp(g)) } else { -1.0 };
        if press_dist >= 0.0 {
            if press_dist < min_dist_sq {
                result = cur;
                min_dist_sq = press_dist;
            }
            cur = unsafe { (*cur).next };
            continue;
        }
        if ext::Item_IsGrabbable(cur) {
            let unk_enum = ext::itIsHeavy(cur) as i32;
            if (fp(g).x2222_b4() == 0 || ext::it_8026B47C(cur) == 0)
                && (fp(g).item_gobj.is_null() || ext::it_8026B4F0(cur))
                && ((unk_enum == 0 && flags & 1 != 0) || (unk_enum == 1 && flags & 2 != 0))
            {
                let mut it_pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::it_8026B344(cur, &mut it_pos);
                let x_range = ext::itGetGrabRangeX(cur);
                let y_range = ext::itGetGrabRangeY(cur);
                let f = fp(g);
                let pickup = &f.x294_itPickup;
                let vec = if unk_enum == 0 {
                    if f.ground_or_air == GA_Ground { pickup.gr_light_offset } else { pickup.air_light_offset }
                } else {
                    pickup.gr_heavy_offset
                };
                if let Some((x0, y0)) = in_box(f, &vec, x_range, y_range, &it_pos) {
                    let y_diff = it_pos.y - y0;
                    let x_diff = it_pos.x - x0;
                    let dist_sq = fmadds(x_diff, x_diff, y_diff * y_diff);
                    if dist_sq < min_dist_sq {
                        result = cur;
                        min_dist_sq = dist_sq;
                    }
                }
            }
        }
        cur = unsafe { (*cur).next };
    }
    result
}

/// The instant-use switch shared by `ftpickupitem_8009447C` and `ftpickupitem_80094B6C`;
/// `true` = the C returned from inside the switch (WStar / Hammer), which differ per caller.
fn use_item(g: G, item_gobj: I, star: fn(G, I), hammer: fn(G)) -> bool {
    let k = ext::itGetKind(item_gobj);
    let heal = |g: G, i: I| {
        let v = ext::it_8026B47C(i);
        ext::Fighter_8006CF5C(fp(g), v);
        ext::Item_8026A8EC(i);
    };
    if k == It_Kind_Heart as i32
        || k == It_Kind_Tomato as i32
        || k == It_Kind_Foods as i32
        || k == It_Kind_Lucky_Egg as i32
        || k == It_Kind_WhispyHealApple as i32
    {
        heal(g, item_gobj);
    } else if k == It_Kind_WStar as i32 {
        star(g, item_gobj);
        return true;
    } else if k == It_Kind_Hammer as i32 {
        hammer(g);
        return true;
    } else if k == It_Kind_RabbitC as i32 {
        ext::ftCommon_8007FA58(g, item_gobj);
    } else if k == It_Kind_MetalB as i32 {
        ext::ftLib_800871A8(g, item_gobj);
        ext::Item_8026A8EC(item_gobj);
    } else if k == It_Kind_Spycloak as i32 {
        let v = crate::hooks::common_data().x7CC;
        ext::ftCo_800C88D4(g, v, 1);
        ext::Item_8026A8EC(item_gobj);
    } else if k == It_Kind_Coin as i32 {
        ext::Item_8026A8EC(item_gobj);
    }
    false
}

/// `ftpickupitem_8009447C`: use an instant item on the grab frame; true = state changed.
pub fn ftpickupitem_8009447C(g: G, item_gobj: I) -> bool {
    assert!(!item_gobj.is_null(), "ftGetImmItem item_gobj is NULL!!");
    if ext::it_8026B30C(item_gobj) == 5 {
        geno_consumable_collect(item_gobj, fp(g).player_id as i32);
        return use_item(g, item_gobj, |g, _| ext::ftCo_800C4724(g), ext::ftCo_800C52F4);
    }
    false
}

/// `ftpickupitem_80094694`: enter LightGet / HeavyGet.
pub fn ftpickupitem_80094694(g: G, msid: i32, looping: bool) {
    let anim_spd = if msid == ftCo_MS_HeavyGet as i32 {
        let sy = fp(g).x34_scale.y;
        let e = ext::Fighter_804D6524_x8();
        ext::ftCo_CalcYScaledKnockback(1.0, sy, e)
    } else {
        1.0
    };
    fp(g).anon_2210.throw_flags = 0;
    ext::Fighter_ChangeMotionState(g, msid, Ft_MF_None as u32, 0.0, anim_spd, 0.0, core::ptr::null_mut());
    if looping {
        while !check_throw_b3(fp(g)) {
            ext::ftAnim_8006EBA4(g);
        }
    } else {
        ext::ftAnim_8006EBA4(g);
    }
    let f = fp(g);
    unsafe {
        if msid == ftCo_MS_HeavyGet as i32 {
            f.mv.co.itemget.x0 = 1;
            f.take_dmg_cb = Some(ftCo_800974C4_cb);
        } else {
            f.mv.co.itemget.x0 = 0;
            f.take_dmg_cb = Some(ftpickupitem_80094DF8_cb);
        }
    }
}

/// `ftpickupitem_80094790`: start a pickup if an item is in reach.
pub fn ftpickupitem_80094790(g: G) -> bool {
    if fp(g).x1978.is_null() {
        let it = ftpickupitem_800942A0(g, 3);
        if !it.is_null() {
            if !ext::itIsHeavy(it) {
                ftpickupitem_80094694(g, ftCo_MS_LightGet as i32, false);
            } else {
                ftpickupitem_80094694(g, ftCo_MS_HeavyGet as i32, false);
            }
            return true;
        }
    }
    false
}

/// `ftpickupitem_80094818`: per-kind pickup callback and the pickup stat.
pub fn ftpickupitem_80094818(g: G, arg1: bool) {
    let f = fp(g);
    if !f.x1978.is_null() {
        ext::pl_8003E17C(f.player_id as i32, f.is_sub_fighter() as i32, f.x1978);
    } else if !f.item_gobj.is_null() {
        if let Some(cb) = ext::ftData_OnItemPickupExt(f.kind as i32) {
            unsafe { cb(g, arg1 as i32) };
        }
        let f = fp(g);
        ext::pl_8003E17C(f.player_id as i32, f.is_sub_fighter() as i32, f.item_gobj);
    }
}

/// `ftpickupitem_800948A8`: take `item_gobj` into the hand.
pub fn ftpickupitem_800948A8(g: G, item_gobj: I) {
    let f = fp(g);
    if !f.item_gobj.is_null() {
        f.x1978 = item_gobj;
    } else {
        f.item_gobj = item_gobj;
    }
    ftpickupitem_80094818(g, true);
    ext::Mex_OnItemPickupDispatch(fp(g).kind as i32, g as *mut _, item_gobj as *mut _, core::ptr::null_mut());
    let ret_part: i32 = if !ext::itIsHeavy(item_gobj) {
        if ext::itGetKind(item_gobj) == It_Kind_WStar as i32 {
            FtPart_TopN as i32
        } else {
            unsafe { (*(*fp(g).ft_data).x8).x10 as i32 }
        }
    } else {
        unsafe { (*(*fp(g).ft_data).x8).x11 as i32 }
    };
    let f = fp(g);
    ext::pl_8003E854(f.player_id as i32, f.is_sub_fighter() as i32, item_gobj);
    ext::Item_8026AB54(item_gobj, g, ret_part);
    if ext::itIsHeavy(item_gobj) {
        let f = fp(g);
        let sfx = unsafe { (*(*f.ft_data).x4C_sfx).x2C };
        ext::ft_800881D8(f, sfx, 127, 64);
    }
}

/// `ftpickupitem_Anim`.
pub fn ftpickupitem_Anim(g: G) {
    if check_throw_b3(fp(g)) {
        let x0 = unsafe { fp(g).mv.co.itemget.x0 };
        let item_gobj = ftpickupitem_800942A0(g, if x0 != 0 { 2 } else { 1 });
        if !item_gobj.is_null() {
            // TARGET_PC: a "press" script item is collected on the grab frame, not held.
            if ext::Geno_ItemPressDist(item_gobj, fp(g)) >= 0.0 {
                ext::Geno_ItemPressCollect(item_gobj, fp(g).player_id as i32);
            } else {
                ftpickupitem_800948A8(g, item_gobj);
            }
        }
    }
    if !ext::ftAnim_IsFramesRemaining(g) {
        let f = fp(g);
        let item_gobj = if !f.x1978.is_null() { f.x1978 } else { f.item_gobj };
        if !item_gobj.is_null() {
            if f.motion_id == ftCo_MS_LightGet as _ {
                if ftpickupitem_8009447C(g, item_gobj) {
                    return;
                }
            } else {
                if f.x2222_b0() != 0 {
                    ext::ftDk_MS_341_800DF980(g);
                } else {
                    ext::ftCo_80096D9C(g);
                }
                return;
            }
        }
        ext::ft_8008A2BC(g);
    }
}

/// `ftpickupitem_IASA`.
pub fn ftpickupitem_IASA(_g: G) {}

/// `ftpickupitem_Phys`.
pub fn ftpickupitem_Phys(g: G) {
    ext::ft_80084F3C(g);
}

/// `ftpickupitem_Coll`.
pub fn ftpickupitem_Coll(g: G) {
    ext::ft_800841B8(g, ftpickupitem_80094D90);
}

/// `ftpickupitem_80094B6C`: use an instant item that was picked up mid-state.
pub fn ftpickupitem_80094B6C(g: G, item_gobj: I) {
    assert!(!item_gobj.is_null(), "ftGetImmItem item_gobj is NULL!!");
    if ext::it_8026B30C(item_gobj) == 5 {
        geno_consumable_collect(item_gobj, fp(g).player_id as i32);
        use_item(
            g,
            item_gobj,
            |_, i| {
                let vec = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                ext::Item_8026ABD8(i, &vec, 1.0);
            },
            ext::ftCo_800C5284,
        );
    }
}

/// `ftpickupitem_80094D90`: landed mid-pickup.
pub fn ftpickupitem_80094D90(g: G) {
    ext::ftCo_8009750C(g);
    if !fp(g).item_gobj.is_null() {
        ftpickupitem_80094B6C(g, fp(g).item_gobj);
    }
    if !fp(g).x1978.is_null() {
        ftpickupitem_80094B6C(g, fp(g).x1978);
    }
    ext::ftCo_Fall_Enter(g);
}

/// `ftpickupitem_80094DF8`: take_dmg callback (light pickup).
pub fn ftpickupitem_80094DF8(g: G) {
    if !fp(g).item_gobj.is_null() {
        ftpickupitem_80094B6C(g, fp(g).item_gobj);
    }
    if !fp(g).x1978.is_null() {
        ftpickupitem_80094B6C(g, fp(g).x1978);
    }
}
