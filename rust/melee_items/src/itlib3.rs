//! Item library, port round 3 (night 3): the next layer of it_2725.c, it_26B1.c, it_279C.c,
//! itanimlist.c, itcoll.c, item.c, ithitbox.c, itmaplib.c, itspawn.c and itzako.c. Every
//! function here either calls nothing or calls only functions already ported in `itlib.rs` /
//! `helpers.rs`. Each cites the C it mirrors; verified against verbatim C copies in
//! tests/c_oracle/round3.c (tests/oracle_round3.rs).
//!
//! Conventions as in `itlib.rs`: an `Item_GObj*` used only for its user data becomes the
//! `Item`; a followed C pointer makes the function `unsafe` (precondition: valid). Globals
//! become parameters: `it_804D6D28` -> `&ItemCommonData`, `it_804D6D40` -> `&it_804D6D40_t`,
//! `Item_804A0E24` -> `&PokemonSelectionState`, `it_804D6D14` -> `&mut u32`, and
//! `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` -> the list head pointer (TODO(group1): take it
//! from the native GObj runtime once that exists).
//!
//! Fused multiply-adds (`tools/rust-port/fma_sites.py`, shipping-build contraction):
//! `itColl_BounceOffVictim` 1, `it_80270CD8` 6 (3 per branch), `it_8026CA4C` 1 (f32, so
//! `fmadds`); every other function here has none.
#![allow(non_upper_case_globals)]

use crate::helpers::COLLIDE_CEILING_MASK;
use crate::itlib::{
    it_8026b3a8, it_8026b724, it_8026b73c, it_8026bb88, it_8026bbcc, it_802714c0, it_8027346c,
    it_80273484, it_8027349c, it_802734b4, it_80275228, it_80275240, it_80275594,
    COLLIDE_FLOOR_MASK,
};
use crate::layout::{
    it_2728_DatAttrs, it_804D6D40_t, itPokemonSpawn_DatAttrs, CommandInfo, Collide_LeftWallMask,
    Collide_RightWallMask, Fighter, HSD_GObj, HitCapsule, Item, ItemCommonData, ItemKind,
    ItemLink, ItemPickTable, ItemStateDesc, PokemonSelectionState, Vec as GcVec,
    lb_80014638_arg1_t, HSD_GOBJ_CLASS_ITEM, It_Kind_L_Gun_Ray, It_PKind_Marumine,
    It_PKind_Start, It_PKind_Terminate,
};
use melee_core::fp::fmadds;
use std::ptr::null_mut;

// ---- it_2725.c -------------------------------------------------------------------------

/// `it_80272818`: the fighter stored in the article's special attributes.
/// # Safety
/// `ip.xC4_article_data` and its `x4_specialAttributes` must be valid.
pub unsafe fn it_80272818(ip: &Item) -> *mut Fighter {
    (*((*ip.xC4_article_data).x4_specialAttributes as *mut it_2728_DatAttrs)).fighter
}

/// `it_80272828`: the last item GObj of `kind` in the item p-link list starting at `head`.
/// # Safety
/// `head` must start a valid `next`-linked GObj list whose user data are Items.
pub unsafe fn it_80272828(head: *mut HSD_GObj, kind: ItemKind) -> *mut HSD_GObj {
    let mut ret = null_mut();
    let mut cur = head;
    while !cur.is_null() {
        if (*((*cur).user_data as *mut Item)).kind == kind {
            ret = cur;
        }
        cur = (*cur).next;
    }
    ret
}

/// `it_80272860`: apply `arg1` to `vel.y` unless moving against it at `|vel.y| >= arg2`.
pub fn it_80272860(ip: &mut Item, arg1: f32, arg2: f32) {
    let r0 = if arg1 < 0.0 { -1 } else { 1 };
    let mut f3 = ip.x40_vel.y;
    let r3 = if f3 < 0.0 { -1 } else { 1 };
    if r3 != r0 {
        if f3 < 0.0 {
            f3 = -f3;
        }
        if f3 < arg2 {
            ip.x40_vel.y -= arg1;
        }
        return;
    }
    ip.x40_vel.y -= arg1;
}

/// `it_80272D1C`: `gobj` is non-null and of the item class.
/// # Safety
/// `gobj` must be null or valid.
pub unsafe fn it_80272d1c(gobj: *const HSD_GObj) -> bool {
    !gobj.is_null() && (*gobj).classifier as i32 == HSD_GOBJ_CLASS_ITEM
}

/// `itColl_BounceOffVictim`. 1 fused site: `vel.y = fmadds(vel.y, x5C, x60)`.
pub fn it_coll_bounce_off_victim(ip: &mut Item, common: &ItemCommonData) {
    ip.x40_vel.x *= common.x58_float;
    ip.x40_vel.y = fmadds(ip.x40_vel.y, common.x5C_float, common.x60_float);
}

/// `it_80273500`: set the velocity, then the four `it_8027346C`.. resets.
pub fn it_80273500(ip: &mut Item, vel: &GcVec) {
    ip.x40_vel.x = vel.x;
    ip.x40_vel.y = vel.y;
    ip.x40_vel.z = vel.z;
    it_8027346c(ip);
    it_80273484(ip);
    it_8027349c(ip);
    it_802734b4(ip);
}

/// `it_80274658`: spin speed from `spin_spd` and an angle in degrees; returns the sign of vel.x.
/// No fused site (`0.01f * spin * (k * arg1)` is three products).
pub fn it_80274658(ip: &mut Item, arg1: f32) -> i32 {
    let mut s2 = 0.0f32;
    let s1 = ip.spin_spd;
    if s1 != s2 {
        s2 = 0.01f32 * s1 * (0.017453292f32 * arg1);
    }
    ip.xD3C_spinSpeed = s2;
    let d1 = if ip.facing_dir < 0.0 { -1 } else { 1 };
    let d2 = if ip.x40_vel.x < 0.0 { -1 } else { 1 };
    if d2 != d1 {
        ip.xD3C_spinSpeed = -ip.xD3C_spinSpeed;
    }
    if ip.xDC8_word.flags.x19() != 1 {
        ip.xD3C_spinSpeed *= -ip.facing_dir;
    }
    d2
}

/// `it_80274D04`: ECB centre minus the previous ECB centre (x, y).
pub fn it_80274d04(ip: &Item, out: &mut lb_80014638_arg1_t) {
    let mut a = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    let mut b = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    it_8026bb88(ip, &mut a);
    it_8026bbcc(ip, &mut b);
    out.unk_x = a.x - b.x;
    out.unk_y = a.y - b.y;
}

/// `it_80275174`: `xD48_halfLifeTimer = lifetime * x4C`.
pub fn it_80275174(ip: &mut Item, lifetime: f32, common: &ItemCommonData) {
    ip.xD48_halfLifeTimer = lifetime * common.x4C_float;
}

/// `it_80275328`: initialise an `ItemLink` owned by `gobj` (`coll_data` is left untouched).
pub fn it_80275328(gobj: *mut HSD_GObj, link: &mut ItemLink) {
    link.prev = null_mut();
    link.next = null_mut();
    link.vel = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    link.pos = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    link.x20 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    link.set_x2C_b0(0);
    link.set_x2C_b1(0);
    link.set_x2C_b2(0);
    link.x1CC = 0;
    link.gobj = gobj;
    link.jobj = null_mut();
}

/// `it_80275390`: drop the owner, then `it_8026B3A8`.
pub fn it_80275390(ip: &mut Item) {
    ip.owner = null_mut();
    it_8026b3a8(ip);
}

// ---- it_26B1.c -------------------------------------------------------------------------

/// `it_8026B3C0`: the number of items of `kind` in the item list.
/// # Safety
/// As [`it_80272828`].
pub unsafe fn it_8026b3c0(head: *mut HSD_GObj, kind: ItemKind) -> i32 {
    let mut i = 0;
    let mut cur = head;
    while !cur.is_null() {
        if (*((*cur).user_data as *mut Item)).kind == kind {
            i += 1;
        }
        cur = (*cur).next;
    }
    i
}

/// `it_8026C3FC`: `it_8026B724` on every item.
/// # Safety
/// As [`it_80272828`].
pub unsafe fn it_8026c3fc(head: *mut HSD_GObj) {
    let mut cur = head;
    while !cur.is_null() {
        it_8026b724(&mut *((*cur).user_data as *mut Item));
        cur = (*cur).next;
    }
}

/// `it_8026C42C`: the `it_8026B73C` flag update on every item (the C inlines the same body).
/// # Safety
/// As [`it_80272828`].
pub unsafe fn it_8026c42c(head: *mut HSD_GObj) {
    let mut cur = head;
    while !cur.is_null() {
        it_8026b73c(&mut *((*cur).user_data as *mut Item));
        cur = (*cur).next;
    }
}

// ---- it_279C.c -------------------------------------------------------------------------

/// `it_8027A13C`: zero the pokemon-spawn x4/x10 vectors.
pub fn it_8027a13c(ip: &mut Item) {
    unsafe {
        let v = &mut ip.xDD4_itemVar.pokemon_spawn;
        v.x10 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
        v.x4 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    }
}

/// `it_8027A344`.
pub fn it_8027a344(ip: &mut Item) {
    unsafe {
        ip.x40_vel.x = ip.xDD4_itemVar.pokemon_spawn.x4.z * ip.facing_dir;
        ip.x40_vel.y = ip.xDD4_itemVar.pokemon_spawn.x4.y;
    }
}

/// `it_8027A364`: 1 + the spawn weights of every pokemon other than the last two spawned.
/// # Safety
/// `ip.xC4_article_data->x4_specialAttributes` must be a valid `itPokemonSpawn_DatAttrs`.
pub unsafe fn it_8027a364(ip: &Item, sel: &PokemonSelectionState) -> i32 {
    let attr = &*((*ip.xC4_article_data).x4_specialAttributes as *const itPokemonSpawn_DatAttrs);
    let mut ret = 1i32;
    for i in It_PKind_Start..It_PKind_Terminate {
        if sel.last_kind != i && sel.previous_kind != i {
            ret = ret.wrapping_add(attr.pokemon_spawn_weights[(i - It_PKind_Start) as usize]);
        }
    }
    ret
}

/// `it_8027AAA0`: copy the spawner's (`ip1`) attributes into the spawned item's vars.
/// # Safety
/// `ip1`'s article special attributes must be a valid `itPokemonSpawn_DatAttrs`;
/// `item2.xCC_item_attr` must be valid; `attr.x38` must be at most 6 (the C overruns `x2C`).
pub unsafe fn it_8027aaa0(ip1: &Item, item2: &mut Item, arg2: i32) {
    let attr = &*((*ip1.xC4_article_data).x4_specialAttributes as *const itPokemonSpawn_DatAttrs);
    let scale = (*item2.xCC_item_attr).x60_scale;
    let v = &mut item2.xDD4_itemVar.pokemon_spawn;
    if arg2 == It_PKind_Marumine {
        v.x1C = 0.0;
        v.x20 = 0.0;
    } else {
        v.x1C = attr.xC;
        v.x20 = attr.x10;
    }
    v.x24 = attr.x14;
    v.x28 = attr.x18;
    v.x48 = (scale - v.x4C) / v.x28;
    v.x44 = attr.x34;
    v.x54 = attr.x38;
    v.x5C = 0;
    v.x58 = 0;
    v.x50 = 0.0;
    for i in 0..attr.x38.max(0) as usize {
        // the C re-reads x60_scale each iteration; nothing writes it in between
        v.x2C[i] = attr.x1C[i] * scale;
    }
}

// ---- itanimlist.c ----------------------------------------------------------------------

/// `it_802795EC`: item script "set hitbox scale".
/// # Safety
/// `cmd.anon_8.u` must point to a valid command followed by the next one.
pub unsafe fn it_802795ec(ip: &mut Item, cmd: &mut CommandInfo) {
    let c = &(*cmd.anon_8.u).set_hitbox_scale;
    let idx = c.idx() as i32;
    let value = c.value();
    let hit = &mut ip.x5D4_hitboxes[idx as usize].hit;
    hit.scale = 0.003906f32 * value as f32;
    ip.x3C = hit.scale;
    let m = 1.0f32 / ip.scl;
    it_80275594(ip, idx, m);
    cmd.anon_8.u = cmd.anon_8.u.add(1);
}

/// `it_802796FC`: item script "set throw flags" (`xDAC_itcmd_var0 = hit_idx`).
/// # Safety
/// As [`it_802795ec`].
pub unsafe fn it_802796fc(ip: &mut Item, cmd: &mut CommandInfo) {
    ip.xDAC_itcmd_var0 = (*cmd.anon_8.u).set_throw_flags.hit_idx();
    cmd.anon_8.u = cmd.anon_8.u.add(1);
}

// ---- itcoll.c --------------------------------------------------------------------------

/// `it_80270CD8`: knockback of `hit` against this item, capped at `x80_float[7]`.
/// 6 fused sites (3 per branch), all `fmadds(a, b, c)` with `c` evaluated first, as clang
/// contracts `a*b + c*d` into `fmuladd(a, b, c*d)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_80270cd8(ip: &Item, hit: &HitCapsule, common: &ItemCommonData) -> f32 {
    let dm = (*ip.xCC_item_attr).x1C_damage_mul;
    let f = &common.x80_float;
    let k = 0.01f32 * hit.x24 as f32;
    let inner = if hit.x28 != 0 {
        fmadds(f[10], f[8], f[9] * (f[10] * hit.x28 as f32))
    } else {
        let s = ip.xC9C as f32 + ip.xCA0 as f32;
        fmadds(f[8], s, f[9] * (hit.damage * s))
    };
    let t = fmadds(f[11], dm * inner, f[12]);
    let mut f1 = fmadds(k, t, hit.x2C as f32);
    if f1 >= f[7] {
        f1 = f[7];
    }
    f1
}

// ---- item.c ----------------------------------------------------------------------------

/// `Item_80268E40`: start the state's command script.
pub fn item_80268e40(ip: &mut Item, desc: &ItemStateDesc) {
    ip.x524_cmd.anon_8.u = desc.xC_script as *mut _;
    ip.x524_cmd.loop_count = 0;
    ip.x524_cmd.timer = 0.0;
}

/// `Item_80269CC4` (static): reset the damage / interaction state.
pub fn item_80269cc4(ip: &mut Item) {
    ip.xCA0 = 0;
    ip.xCA4 = 0;
    ip.xC94 = 0;
    ip.xCC8_knockback = 0.0;
    ip.xC34_damageDealt = 0;
    ip.xC48 = 0;
    ip.xC4C = 0;
    ip.xC50 = 0;
    ip.xCCC_incDamageDirection = 0.0;
    ip.xCB8_outDamageDirection = 0.0;
    ip.xC68 = 0.0;
    ip.xCD0 = 0.0;
    ip.xCC0 = 0.0;
    unsafe { ip.xDCF_flag.anon_0.set_b6(0) };
    ip.xCC4 = 0;
    ip.xCEC_fighterGObj = null_mut();
    ip.xCF0_itemGObj = null_mut();
    ip.xCF4_fighterGObjUnk = null_mut();
    ip.toucher = null_mut();
    ip.xC64_reflectGObj = null_mut();
    ip.xC90_absorbGObj = null_mut();
    ip.xCFC = null_mut();
    ip.xCB4 = -1;
    ip.xC38 = -1;
    ip.xCE0 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    ip.xCD4 = GcVec { x: 0.0, y: 0.0, z: 0.0 };
    unsafe {
        ip.xDCE_flag.anon_0.set_b5(0);
        ip.xDCE_flag.anon_0.set_b4(0);
    }
    ip.xDCC_flag.set_b1(0);
    ip.xC70 = 1.0;
    ip.xC6C = 1.0;
    ip.xDCC_flag.set_b2(0);
    ip.xDC8_word.flags.set_xB(0);
    ip.xCA8 = 0;
    ip.xDC8_word.flags.set_x3(0);
    ip.xDC8_word.flags.set_x4(0);
    unsafe { ip.xDCE_flag.anon_0.set_b6(0) };
}

/// `Item_8026AE60`: next nonzero spawn id from the `it_804D6D14` counter.
pub fn item_8026ae60(counter: &mut u32) -> u32 {
    let result = *counter;
    *counter = counter.wrapping_add(1);
    if *counter == 0 {
        *counter = counter.wrapping_add(1);
    }
    result
}

/// `Item_IsGrabbable`: grabbable flag set and a `picked_up` callback present.
/// # Safety
/// `ip.xB8_itemLogicTable` must be valid when `xDC8_word.flags.x15` is set.
pub unsafe fn item_is_grabbable(ip: &Item) -> bool {
    ip.xDC8_word.flags.x15() != 0 && (*ip.xB8_itemLogicTable).picked_up.is_some()
}

// ---- ithitbox.c ------------------------------------------------------------------------

/// `it_802756E0`: `xD0C = 0`, then `it_802714C0`.
pub fn it_802756e0(ip: &mut Item) {
    ip.xD0C = 0;
    it_802714c0(ip);
}

// ---- itmaplib.c ------------------------------------------------------------------------

/// `it_80276CEC`: store the contact normal and surface index, floor winning over ceiling over
/// right wall over left wall. With no contact the C reads an uninitialised `coll_index`; the
/// shipping build compiles with `-ftrivial-auto-var-init=zero`, so it is 0 here.
pub fn it_80276cec(ip: &mut Item) {
    let coll = &ip.x378_itemColl;
    let mut idx: u32 = 0;
    let mut n = ip.xAC_unk;
    if coll.env_flags & Collide_LeftWallMask != 0 {
        n = coll.left_facing_wall.normal;
        idx = coll.left_facing_wall.index as u32;
    }
    if coll.env_flags & Collide_RightWallMask != 0 {
        n = coll.right_facing_wall.normal;
        idx = coll.right_facing_wall.index as u32;
    }
    if coll.env_flags & COLLIDE_CEILING_MASK != 0 {
        n = coll.ceiling.normal;
        idx = coll.ceiling.index as u32;
    }
    if coll.env_flags & COLLIDE_FLOOR_MASK != 0 {
        n = coll.floor.normal;
        idx = coll.floor.index as u32;
    }
    ip.xAC_unk = n;
    ip.xC2C = idx as i32;
}

// ---- itspawn.c -------------------------------------------------------------------------

/// `it_8026CA4C`: sum of the scaled, rounded-up weights of the kinds selected by `mask`
/// (bit 0 = kind `start`), for kinds below `It_Kind_L_Gun_Ray`. `sum += f` is
/// `sum = (s32) ((f32) sum + f)`; 1 fused site: `fmadds(arg4, (f32) w, 0.99)`. The result is
/// stored to the u16 `x8`.
pub fn it_8026ca4c(alloc: &mut ItemPickTable, weights: &[i32], mask: u64, start: i32, arg4: f32) {
    let mut mask = mask;
    let mut sum: i32 = 0;
    let mut i = start;
    while i < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 {
            let f = fmadds(arg4, weights[i as usize] as f32, 0.99f32);
            sum = c_f32_to_s32(sum as f32 + f);
        }
        i += 1;
        mask >>= 1;
    }
    alloc.x8 = sum as u16;
}

/// C `(s32) f` as the shipping build runs it: gwtool lowers `fptosi` to the saturating
/// `llvm.fptosi.sat.i32` (NaN -> 0), which is exactly Rust's `as i32`.
#[inline]
fn c_f32_to_s32(f: f32) -> i32 {
    f as i32
}

// ---- itzako.c --------------------------------------------------------------------------

/// `it_8027B730`: reset the zako vars, then `it_80275228` / `it_80275240`.
pub fn it_8027b730(ip: &mut Item) {
    unsafe {
        let z = &mut ip.xDD4_itemVar.zako;
        z.jobj = null_mut();
        z.idx = -1;
        z.x8.z = 0.0;
        z.x8.x = 0.0;
        z.x8.y = 1.0;
        z.x14.x = 0;
        z.x14.y = 0;
        z.x14.z = 0;
    }
    it_80275228(ip);
    it_80275240(ip);
}

/// `it_8027CBFC`: a per-state float from `it_804D6D40`, through an `s32` (truncated).
pub fn it_8027cbfc(ip: &Item, d40: &it_804D6D40_t) -> f32 {
    let r: i32 = match ip.xCC4 {
        6 | 7 => c_f32_to_s32(d40.xC),
        9 => c_f32_to_s32(d40.x10),
        _ => 0,
    };
    r as f32
}
