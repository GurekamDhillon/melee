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
unsafe fn sa<'a, T>(ip: &Item) -> &'a mut T {
    &mut *((*ip.xC4_article_data).x4_specialAttributes as *mut T)
}
#[inline(always)]
unsafe fn item_of<'a>(gobj: *mut HSD_GObj) -> Option<&'a mut Item> {
    if gobj.is_null() {
        None
    } else {
        ((*gobj).user_data as *mut Item).as_mut()
    }
}

// ---- it_2725.c --------------------------------------------------------------------------

pub use crate::tu::it_2725::get_bone_by_id;

// ---- itanimlist.c -----------------------------------------------------------------------

/// `it_80279768`: `xDBC_itcmd_var4.flags.x0 = true`, next command.
/// # Safety
/// `cmd.u` points into a command array (one past is formed, as in C).
pub unsafe fn it_80279768(ip: &mut Item, cmd: &mut CommandInfo) {
    ip.anon_dbc.xDBC_itcmd_var4.flags.set_x0(1);
    cmd.anon_8.u = cmd.anon_8.u.add(1);
}
/// `it_80279720`: `xDB0_itcmd_var1 = cmd->u->set_throw_flags.hit_idx`, next command.
/// # Safety
/// As `it_80279768`; `cmd.u` valid to read.
pub unsafe fn it_80279720(ip: &mut Item, cmd: &mut CommandInfo) {
    ip.xDB0_itcmd_var1 = (*cmd.anon_8.u).set_throw_flags.hit_idx();
    cmd.anon_8.u = cmd.anon_8.u.add(1);
}
/// `it_80279744`: `xDB4_itcmd_var2 = cmd->u->set_throw_flags.hit_idx`, next command.
/// # Safety
/// As `it_80279720`.
pub unsafe fn it_80279744(ip: &mut Item, cmd: &mut CommandInfo) {
    ip.xDB4_itcmd_var2 = (*cmd.anon_8.u).set_throw_flags.hit_idx();
    cmd.anon_8.u = cmd.anon_8.u.add(1);
}

// ---- itspawn.c --------------------------------------------------------------------------

/// `it_8026C47C`: one bit per item kind `It_Kind_Capsule..238` that has a live item on the
/// p-link list (`it_80272828`), 32 kinds per word (`!(kind & It_Kind_RabbitC)` starts a word).
/// # Safety
/// As `it_80272828` (`head` the item p-link list head).
pub unsafe fn it_8026c47c(head: *mut HSD_GObj, arg: &mut it_8026C47C_arg0_t) {
    let mut it_kind = It_Kind_Capsule as u32;
    let mut bit_idx: i32 = 0;
    *arg = it_8026C47C_arg0_t::zeroed();
    // SAFETY: eight consecutive i32 fields (unk0..unk1C), repr(C).
    let words = core::slice::from_raw_parts_mut(arg as *mut it_8026C47C_arg0_t as *mut i32, 8);
    let mut w = 0usize;
    while it_kind < 238 {
        if !it_80272828(head, it_kind as ItemKind).is_null() {
            words[w] |= 1i32.wrapping_shl(bit_idx as u32);
        }
        it_kind += 1;
        bit_idx += 1;
        if it_kind & It_Kind_RabbitC as u32 == 0 {
            bit_idx = 0;
            w += 1;
        }
    }
}

// ---- kinds ------------------------------------------------------------------------------

/// `itBombHei` `fn_80280974_inline`: flags.x15 off, `vel.x = attrs->xC * facing_dir`,
/// `xD5C = 0`, then `it_80275444`.
/// # Safety
/// The special attributes are an `itBombHeiAttributes`.
pub unsafe fn fn_80280974_inline(ip: &mut Item) {
    let a = sa::<itBombHeiAttributes>(ip);
    it_8026b3a8(ip);
    ip.x40_vel.x = a.xC * ip.facing_dir;
    ip.xD5C = 0;
    it_80275444(ip);
}

pub use crate::tu::itclimbersstring::it_802C248C_joint24 as it_802c248c_joint24;
pub use crate::tu::itclimbersstring::it_802C248C_joint28 as it_802c248c_joint28;

/// itcrazyhandbomb.c `it_802F1340`: empty.
pub fn it_802f1340() {}
/// itlugia.c `it_802D14D0`: empty.
pub fn it_802d14d0() {}

/// itdosei.c `itDosei_GetJObj`: `gobj->hsd_obj`.
/// # Safety
/// `gobj` valid.
pub unsafe fn it_dosei_get_jobj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

pub use crate::tu::ithitodeman::it_802D472C_inline as it_802d472c_inline;

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

/// itnesspkthunderball.c `it_802AB9C0`: with `xF00 == 0`, clear owner / flags.x13 and run
/// `it_802AC5D8` on the six linked trail GObjs.
/// # Safety
/// `gobj` NULL or a live item GObj; the six links NULL or GObjs (user_data NULL or an Item).
pub unsafe fn it_802ab9c0(gobj: *mut HSD_GObj) {
    if let Some(ip) = item_of(gobj) {
        if ip.xDD4_itemVar.pkthunder.xF00 == 0 {
            ip.owner = null_mut();
            ip.xDC8_word.flags.set_x13(0);
            for i in 0..6 {
                let l = ip.xDD4_itemVar.pkthunder.xDD4[i];
                if !l.is_null() {
                    it_802ac5d8(item_of(l));
                }
            }
        }
    }
}

/// itparasol.c `it_8028B08C`: `{20, 16, 45, 2, 20, 30, 40, 16}[st[statenum].anim_id]`. Panics
/// on an anim id outside 0..8 (the C reads past its table; the real table has -1 entries).
pub fn it_8028b08c(st: &[ItemStateTable], statenum: i32) -> i32 {
    const T: [i32; 8] = [20, 16, 45, 2, 20, 30, 40, 16];
    T[st[statenum as usize].anim_id as usize]
}

/// itpikachuthunder.c `it_802B22B8_inline`: copy `pos` to the child's `x28`, set its `x4`.
/// # Safety
/// `x34` NULL or a GObj whose user_data is NULL or an Item.
pub unsafe fn it_802b22b8_inline(ip: &mut Item) {
    let c = ip.xDD4_itemVar.pikachuthunder.x34;
    if !c.is_null() {
        if let Some(ch) = item_of(c) {
            ch.xDD4_itemVar.pikachuthunder.x28 = ip.pos;
            ch.xDD4_itemVar.pikachuthunder.x4 = 1;
        }
    }
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

/// itsscope.c `it_80291D38`: `attrs->xC[charge_level]` as s32 (charge 0..9; the C has no
/// return for other values). `as i32` is gwtool's saturating conversion.
/// # Safety
/// The special attributes are an `itSScopeAttributes`.
pub unsafe fn it_80291d38(ip: &Item, charge_level: i32) -> i32 {
    let a = sa::<itSScopeAttributes>(ip);
    a.xC[charge_level as usize] as i32
}

/// itsword.c `it_80285314`: `it_80272CC0(gobj, 5)`.
/// # Safety
/// As `it_80272CC0`.
pub unsafe fn it_80285314(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    it_80272cc0(gobj, 5)
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

pub use crate::tu::itparasol::jobj_child;
