
//! Port round 14 (night 3, round 6): hand ports of 26 item functions whose callees are all
//! ported in this crate (`it_80272CC0`, `it_80272860`, `it_802AC5D8`, `swapVelocity`,
//! `get_min_life`, `it_802B3368`, `it_80275594`, `pika_scale`, `it_8026B3A8`, `it_80275444`,
//! `it_80272828`), native in melee_runtime (`lbVector_NormalizeXY`) or that call nothing.
//! Verified against verbatim C copies (tests/c_oracle/round14_bodies.h, extracted by
//! gen_round14.py) in tests/oracle_round14.rs.
//!
//! Conventions as in rounds 3 / 10-13: a followed C pointer makes the function `unsafe`
//! (precondition: valid, as in C); globals are parameters (`it_803F5AB0`, itparasol.c's state
//! table -> a slice; `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head, TODO(group1) as
//! itlib3.rs). Callbacks that C passes a GObj take the `Item` when they only read
//! `GET_ITEM(gobj)`.
//!
//! Fused sites (shipping-build contraction, `fmadds` / `fmsubs` = gwtool's lowering of
//! `llvm.fmuladd`): `itYoshistar_UnkMotion0_Phys` 1 (`vel.x += accel * facing_dir`),
//! `itSamusbomb_UnkMotion2_Phys` 1 (`VEC2_SQ_LEN` = `SQ(x) + SQ(y)` -> `fmadds(x, x, y*y)`),
//! `itNesspkfirepillar_Logic24_DmgReceived` 1 (`-((xC9C * x4) - life)` -> `-fmsubs(..)`).
#![allow(non_snake_case)]

use crate::helpers::pika_scale;

use crate::itkinds11::it_80272cc0;

use crate::itkinds13::swap_velocity;

use crate::itkinds5::{it_802ac5d8, it_802b3368};

use crate::itkinds8::get_min_life;

use crate::itkinds9::my_sqrtf_accurate;

use crate::itlib::{it_8026b3a8, it_80275444, it_80275594};

use crate::itlib3::{it_80272828, it_80272860};

use crate::layout::{
    Article, CommandInfo, HSD_GObj, HSD_JObj, HSD_Joint, Item, ItemStateTable,
    Vec as GcVec, it_8026C47C_arg0_t, itBombHeiAttributes, itClimbersStringAttributes,
    itLikelikeAttributes, itNessPKFirepillarAttributes, itSamusBombAttributes,
    itSScopeAttributes, ItemKind, It_Kind_Capsule, It_Kind_RabbitC,
};

use melee_core::fp::{fmadds, fmsubs};

use melee_core::vec::Vec3 as RVec;

use melee_runtime::lbvector;

use core::ptr::null_mut;

#[inline(always)]
pub(crate) unsafe fn sa<'a, T>(ip: &Item) -> &'a mut T {
    &mut *((*ip.xC4_article_data).x4_specialAttributes as *mut T)
}

#[inline(always)]
pub(crate) unsafe fn item_of<'a>(gobj: *mut HSD_GObj) -> Option<&'a mut Item> {
    if gobj.is_null() {
        None
    } else {
        ((*gobj).user_data as *mut Item).as_mut()
    }
}

/// itdosei.c `itDosei_GetJObj`: `gobj->hsd_obj`.
/// # Safety
/// `gobj` valid.
pub unsafe fn it_dosei_get_jobj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// itlikelike.c `likelikeVelocity`: within `attrs->x0.x0_f32->y * multiplier` of `x20` on both
/// axes, swap velocity every fifth frame (counter `x48`); else reset the counter.
/// # Safety
/// `attrs.x0.x0_f32` valid; the item's special attributes as `swapVelocity` needs.
pub unsafe fn likelike_velocity(ip: &mut Item, attrs: &itLikelikeAttributes, multiplier: f32) {
    let ll = ip.xDD4_itemVar.likelike;
    let dx = ll.x20.x - ip.pos.x;
    let dy = ll.x20.y - ip.pos.y;
    let lim = (*attrs.x0.x0_f32).y * multiplier;
    if (if dx < 0.0 { -dx } else { dx }) < lim && (if dy < 0.0 { -dy } else { dy }) < (*attrs.x0.x0_f32).y * multiplier {
        if ip.xDD4_itemVar.likelike.x48 >= 4 {
            swap_velocity(ip);
            ip.xDD4_itemVar.likelike.x48 = 0;
        } else {
            ip.xDD4_itemVar.likelike.x48 += 1;
        }
    } else {
        ip.xDD4_itemVar.likelike.x48 = 0;
    }
}

/// itlinkhookshot.c `fn_802A33A0_GetFighter`: `item->owner->user_data`.
/// # Safety
/// `owner` valid.
pub unsafe fn fn_802a33a0_get_fighter(ip: &Item) -> *mut core::ffi::c_void {
    (*ip.owner).user_data as *mut core::ffi::c_void
}

/// itnesspkfirepillar.c `itNesspkfirepillar_Logic24_DmgReceived`: life -= damage * x4
/// (`-fmsubs`), floored to `get_min_life()` at `<= 0`. Returns false.
/// # Safety
/// The special attributes are an `itNessPKFirepillarAttributes`.
pub unsafe fn it_nesspkfirepillar_logic24_dmg_received(ip: &mut Item) -> bool {
    let a = sa::<itNessPKFirepillarAttributes>(ip);
    ip.xD44_lifeTimer = -fmsubs(ip.xC9C as f32, a.x4, ip.xD44_lifeTimer);
    if ip.xD44_lifeTimer <= 0.0 {
        ip.xD44_lifeTimer = get_min_life();
    }
    false
}

/// itpikachuthunder.c `itPikachuthunder_UnkMotion2_ScaleCall`: hitbox 0 scale *=
/// `pika_scale(x10, xC)`.
pub fn it_pikachuthunder_unk_motion2_scale_call(ip: &mut Item) {
    let v = unsafe { ip.xDD4_itemVar.pikachuthunder };
    it_80275594(ip, 0, pika_scale(v.x10, v.xC));
}

/// itpikachutjoltair.c `itPikachutjoltair_UnkMotion0_Anim_inline`: true unless the linked
/// ground jolt links back to `gobj`.
/// # Safety
/// `xDD8` NULL or a GObj whose user_data is NULL or an Item.
pub unsafe fn it_pikachutjoltair_unk_motion0_anim_inline(gobj: *mut HSD_GObj, ip: &Item) -> bool {
    let linked = ip.xDD4_itemVar.pikachujoltair.xDD8;
    if linked.is_null() {
        return true;
    }
    if it_802b3368(item_of(linked).map(|r| &*r)) != gobj {
        return true;
    }
    false
}

/// itsamusbomb.c `itSamusbomb_UnkMotion2_Phys`: clamp the xy length of `x7C` to `attrs->xC`
/// (4-step `my_sqrtf`, `lbVector_NormalizeXY`).
/// # Safety
/// The special attributes are an `itSamusBombAttributes`.
pub unsafe fn it_samusbomb_unk_motion2_phys(ip: &mut Item) {
    let a = sa::<itSamusBombAttributes>(ip);
    let v = ip.x7C;
    if my_sqrtf_accurate(fmadds(v.x, v.x, v.y * v.y)) > a.xC {
        let mut r = RVec { x: v.x, y: v.y, z: v.z };
        lbvector::normalize_xy(&mut r);
        ip.x7C = GcVec { x: r.x, y: r.y, z: r.z };
        let mult = a.xC;
        ip.x7C.x *= mult;
        ip.x7C.y *= mult;
    }
}

/// ityoshistar.c `itYoshistar_UnkMotion0_Phys`: fall (`it_80272860`), then
/// `vel.x += star->accel * facing_dir` (fused).
/// # Safety
/// `xCC_item_attr` valid; the special attributes are a `StarAttrs` (`{ speed, accel }`).
pub unsafe fn it_yoshistar_unk_motion0_phys(ip: &mut Item) {
    let attr = &*ip.xCC_item_attr;
    let star = (*ip.xC4_article_data).x4_specialAttributes as *const f32;
    let accel = *star.add(1);
    it_80272860(ip, attr.x10_fall_speed, attr.x14_fall_speed_max);
    ip.x40_vel.x = fmadds(accel, ip.facing_dir, ip.x40_vel.x);
}

/// itzgshell.c `get_attrs`: the special attributes pointer (an `itGShell_Attrs`, file-local in C).
/// # Safety
/// `xC4_article_data` valid.
pub unsafe fn zgshell_get_attrs(ip: &Item) -> *mut core::ffi::c_void {
    (*ip.xC4_article_data).x4_specialAttributes as *mut core::ffi::c_void
}

pub use crate::tu::itbombhei::fn_80280974_inline_fp as fn_80280974_inline;
pub use crate::tu::it_2725::get_bone_by_id_fp as get_bone_by_id;
pub use crate::tu::itspawn::it_8026c47c;
pub use crate::tu::itanimlist::it_80279720_fp as it_80279720;
pub use crate::tu::itanimlist::it_80279744_fp as it_80279744;
pub use crate::tu::itanimlist::it_80279768_fp as it_80279768;
pub use crate::tu::itsword::it_80285314_fp as it_80285314;
pub use crate::tu::itparasol::it_8028b08c;
pub use crate::tu::itsscope::it_80291d38;
pub use crate::tu::itnesspkthunderball::it_802ab9c0;
pub use crate::tu::itpikachuthunder::it_802b22b8_inline;
pub use crate::tu::itclimbersstring::it_802c248c_joint24;
pub use crate::tu::itclimbersstring::it_802c248c_joint28;
pub use crate::tu::itlugia::it_802d14d0;
pub use crate::tu::ithitodeman::it_802d472c_inline;
pub use crate::tu::itcrazyhandbomb::it_802f1340;
pub use crate::tu::itparasol::jobj_child_fp as jobj_child;
