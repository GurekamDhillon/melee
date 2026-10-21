//! Port round 12 (night 3, round 4): hand ports of 15 item functions whose callees are all
//! ported (this crate's `it_80272CC0` / `it_80277040`) or native elsewhere in the workspace
//! (MSL `sqrtf` / `sqrtf_accurate` in `melee_runtime::gekko`; `PSVECAdd`, mtx_pc.c's plain
//! component adds, written out). Verified against verbatim C copies
//! (tests/c_oracle/round12_bodies.h, extracted by gen_round12.py) in tests/oracle_round12.rs.
//!
//! Conventions as in round 3 / round 10 / round 11: a followed C pointer makes the function
//! `unsafe` (precondition: valid, as in C); globals are parameters: `it_804D6D28` ->
//! `&ItemCommonData`, `Item_804A0C64` -> `&mut HSD_ObjAllocUnk`, `it_804D6D24` -> the article
//! table pointer, the `it_804D6D08` / `it_804D6D0C` / `it_804D6D18` counters -> `&mut`,
//! `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head (TODO(group1), as itlib3.rs).
//!
//! `it_8026B1D4`'s `my_sqrtf` is MSL `sqrtf` (the PC build has no MUST_MATCH, it_26B1.c:59);
//! itmaplib.c's `sqrtf_accurate_sp18` is the same four Newton steps as MSL `sqrtf_accurate`.
//!
//! Fused sites (shipping-build contraction): `it_8026B1D4` 3 f32 (`vx*vx + vy*vy + vz*vz` ->
//! `fmadds(vz, vz, fmadds(vx, vx, vy*vy))`, `ret += speed * k5` -> `fmadds(speed, k5, ret)`),
//! `it_8026C258` 1, `it_802775F0` 1, `it_80277544` 1 (each `SQ(x) + SQ(y)` -> `fmadds(x, x, y*y)`).
#![allow(non_snake_case)]

use crate::itkinds10::it_80277040;
use crate::itkinds11::it_80272cc0;
use crate::layout::{
    Article, Fighter, HSD_GObj, HSD_GObjEvent, HSD_GObjInteraction, HSD_JObj, HSD_ObjAllocUnk,
    HitCapsule, Item, ItemCommonData, Vec as GcVec, itLGun_ItemVars, It_Kind_L_Gun,
};
use melee_core::fp::fmadds;
use melee_runtime::gekko::{sqrtf, sqrtf_accurate};

#[inline(always)]
unsafe fn item<'a>(gobj: *mut HSD_GObj) -> &'a mut Item {
    &mut *((*gobj).user_data as *mut Item)
}

// ---- it_26B1.c -------------------------------------------------------------------------

/// `it_8026B1D4`: hitbox damage, plus a speed bonus (`k5 * |vel| + k6`, at least 1) when
/// `xDC8_word.flags.x14` is set. `common` is `it_804D6D28`.
/// # Safety
/// `gobj` is a live item GObj.
pub unsafe fn it_8026b1d4(gobj: *mut HSD_GObj, cap: &HitCapsule, common: &ItemCommonData) -> f32 {
    let mut ret = cap.damage;
    let ip = item(gobj);
    if ip.xDC8_word.flags.x14() != 0 {
        let v = &ip.x40_vel;
        let speed = sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)));
        ret = fmadds(speed, common.x80_float[5], ret);
        ret += common.x80_float[6];
        if ret as f64 <= 1.0 {
            return 1.0;
        }
    }
    ret
}

/// `it_8026B7D8`: `it_804D6D0C`.
pub fn it_8026b7d8(it_804d6d0c: &i32) -> i32 {
    *it_804d6d0c
}

/// `it_8026B7E0`: `it_804D6D08`.
pub fn it_8026b7e0(it_804d6d08: &i32) -> i32 {
    *it_804d6d08
}

/// `it_8026C1D4`: the L-Gun article's `timer`. `tbl` is `it_804D6D24`.
/// # Safety
/// `tbl[It_Kind_L_Gun]` and its special attributes are valid.
pub unsafe fn it_8026c1d4(tbl: *const *mut Article) -> u32 {
    let a = *tbl.add(It_Kind_L_Gun as usize);
    (*((*a).x4_specialAttributes as *const itLGun_ItemVars)).timer as u32
}

/// `it_8026C258`: the nearest (2D) target / lock-on / enemy / kind-7 item, not grabbed and not
/// owned with `x13` set, on the `facing_dir` side of `pos` (both sides unless +-1).
/// # Safety
/// `head` is the item p-link list (each user data an Item).
pub unsafe fn it_8026c258(head: *mut HSD_GObj, pos: &GcVec, facing_dir: f32) -> *mut HSD_GObj {
    let mut min_sq_dist = f32::MAX;
    let mut result = core::ptr::null_mut();
    let mut cur = head;
    while !cur.is_null() {
        let ip = item(cur);
        let hk = ip.hold_kind;
        if (hk == 4 || hk == 5 || hk == 6 || hk == 7)
            && ip.grab_victim.is_null()
            && (ip.xDC8_word.flags.x13() == 0 || ip.owner.is_null())
            && (facing_dir != -1.0 || !(ip.pos.x > pos.x))
            && (facing_dir != 1.0 || !(ip.pos.x < pos.x))
        {
            let dx = pos.x - ip.pos.x;
            let dy = pos.y - ip.pos.y;
            let sq = fmadds(dx, dx, dy * dy);
            if sq < min_sq_dist {
                min_sq_dist = sq;
                result = cur;
            }
        }
        cur = (*cur).next;
    }
    result
}

// ---- item.c ----------------------------------------------------------------------------

/// `Item_8026784C`: whether the drop class `drop_item` has reached its cap (`Item_804A0C64`);
/// class 9 instead takes one from its pending count.
pub fn item_8026784c(drop_item: i32, _unused: i32, d: &mut HSD_ObjAllocUnk) -> bool {
    match drop_item {
        0 => d.x0 >= d.x4,
        1 => d.x8 >= d.xC,
        2 => d.x10 >= d.x14,
        3 => d.x58 >= d.x5C,
        5 => d.x40 >= d.x44,
        7 => d.x34 >= d.x38,
        9 => {
            if d.x1C > d.x20 {
                d.x1C = d.x1C.wrapping_sub(1);
                true
            } else {
                false
            }
        }
        10 => d.x24 >= d.x28,
        11 => d.x48 >= d.x4C,
        12 => d.x50 >= d.x54,
        _ => false,
    }
}

// ---- itcoll.c --------------------------------------------------------------------------

/// `it_8026F9A0`: `it_804D6D18 = 0`.
pub fn it_8026f9a0(it_804d6d18: &mut u32) {
    *it_804d6d18 = 0;
}

// ---- it_2725.c -------------------------------------------------------------------------

/// `it_80272C90`: the model's bone-attach bone (`it_80272CC0(gobj, x8_bone_attach_id)`).
/// # Safety
/// As `it_80272CC0`.
pub unsafe fn it_80272c90(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let id = (*(*item(gobj).xC4_article_data).x10_modelDesc).x8_bone_attach_id;
    it_80272cc0(gobj, id)
}

/// `it_802746F8`: descend `(xC_bit_field >> 6) & 3` first children from the root JObj
/// (NULL-safe).
/// # Safety
/// `gobj` is a live item GObj; its article data / model desc valid; reached JObjs valid or NULL.
pub unsafe fn it_802746f8(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let mut j = (*gobj).hsd_obj as *mut HSD_JObj;
    let bit_chk = ((*(*item(gobj).xC4_article_data).x10_modelDesc).xC_bit_field as u32 >> 6) & 3;
    let mut n = bit_chk as i32;
    while n > 0 {
        j = if j.is_null() { core::ptr::null_mut() } else { (*j).child };
        n -= 1;
    }
    j
}

/// `it_80274ED8`: `it_804D6D0C++`.
pub fn it_80274ed8(it_804d6d0c: &mut i32) {
    *it_804d6d0c = it_804d6d0c.wrapping_add(1);
}

/// `it_80274EE8`: `it_804D6D08++`.
pub fn it_80274ee8(it_804d6d08: &mut i32) {
    *it_804d6d08 = it_804d6d08.wrapping_add(1);
}

/// `it_80274F28`: set the grab callbacks (`xDD0_flag.b5`, `xD08`, `grab_dealt`,
/// `grabbed_for_victim`).
pub fn it_80274f28(ip: &mut Item, arg1: i8, arg2: HSD_GObjEvent, arg3: HSD_GObjInteraction) {
    // SAFETY: the flag union is a plain byte.
    unsafe { ip.xDD0_flag.anon_0.set_b5(1) };
    ip.xD08 = arg1 as u8;
    ip.grab_dealt = arg2;
    ip.grabbed_for_victim = arg3;
}

// ---- itmaplib.c ------------------------------------------------------------------------

/// `it_80277544`: after a steep-slope slide (`it_80277040`), stop sliding when fast enough
/// (`|vel + nudge|^2 >= xCC`) or when `xD5C == 2`.
/// # Safety
/// As `it_80277040`.
pub unsafe fn it_80277544(ip: &mut Item, common: &ItemCommonData) -> bool {
    let mut ret_val = false;
    if ip.xD5C != 0 && it_80277040(ip, common) && {
        // PSVECAdd(&vel, &nudge, &sp10)
        let sx = ip.x40_vel.x + ip.x70_nudge.x;
        let sy = ip.x40_vel.y + ip.x70_nudge.y;
        fmadds(sx, sx, sy * sy) >= common.xCC || ip.xD5C == 2
    } {
        ip.xDC8_word.flags.set_x1F(1);
        ret_val = true;
        ip.xD5C = 1;
    }
    ret_val
}

/// `it_802775F0`: spin speed from the 2D speed `arg1` over the ECB bottom, signed by the
/// motion direction vs facing, flipped by facing unless `x19 == 1`. Only when `spin_spd != 0`.
pub fn it_802775f0(ip: &mut Item, arg1: &GcVec) {
    let mut t = 0.0f32;
    if ip.spin_spd != t {
        t = sqrtf_accurate(fmadds(arg1.x, arg1.x, arg1.y * arg1.y));
        ip.xD3C_spinSpeed = 0.85f32 * (t / ip.xC1C.bottom);
        let sa = if arg1.x < 0.0 { -1 } else { 1 };
        let sf = if ip.facing_dir < 0.0 { -1 } else { 1 };
        if sa != sf {
            ip.xD3C_spinSpeed = -ip.xD3C_spinSpeed;
        }
        if ip.xDC8_word.flags.x19() != 1 {
            ip.xD3C_spinSpeed *= -ip.facing_dir;
        }
    }
}

// ---- it_279C.c -------------------------------------------------------------------------

/// `it_8027B070`: copy the owner fighter's attack id / instance and staling fields.
/// # Safety
/// `owner_gobj`'s user data is a Fighter.
pub unsafe fn it_8027b070(ip: &mut Item, owner_gobj: *mut HSD_GObj) {
    let fp = &*((*owner_gobj).user_data as *const Fighter);
    ip.xD88_attackID = fp.x2068_attackID;
    ip.xD8C_attack_instance = fp.x206C_attack_instance;
    ip.xD90 = fp.x2070;
    ip.xD94 = fp.x2074.x2074_vec;
    ip.xD9C = fp.x2074.x207C;
    ip.xDA4_word = fp.x2074.anon_10.x2084;
    ip.xDA8_short = fp.x2074.x2088;
}
