//! Item kinds, port round 5 (night 3), hand-ported part: small per-kind functions whose bodies
//! touch only the Item, its `ItemAttr`, its kind's special attributes and item vars, and call
//! only functions already ported. (The one-statement forwarders of this round are generated in
//! `itfwd.rs`.) Each function cites the C it mirrors; all are verified against verbatim C copies
//! in tests/c_oracle/round5_bodies.h (tests/oracle_round5.rs).
//!
//! Conventions as in `itkinds.rs`: the `Item_GObj*` becomes the `Item`; a C `gobj != NULL`
//! (or `ip != NULL`) guard becomes an `Option`; reading `xCC_item_attr` or the special attributes
//! makes a function `unsafe` (precondition: valid); the `it_804D6D28` global is
//! `&ItemCommonData`. Pointer item vars that the C compares with a GObj are compared as
//! `*mut HSD_GObj` (`Item_GObj` and `Fighter_GObj` are both `HSD_GObj`).
//!
//! Fused multiply-adds (`tools/rust-port/fma_sites.py`, shipping contraction): f32 (`fmadds`)
//! in `itUnknown_UnkMotion0_Phys` (1), `it_8028D56C` (1), `it_802F23AC` (2),
//! `it_27CF_Logic114_DmgReceived` (1); f64 (a true fma) in `it_2E5A_UnkMotion1_Anim` (1).
//! Every other function here has none.
#![allow(non_snake_case)]

use crate::helpers::it_80275158;
use crate::item::it_802762bc;
use crate::itkinds::{it_802e657c, it_8028b988, it_capsule_unk_motion4_phys};
use crate::itlib::{it_8026b3a8, it_8026b894, it_802751d8, it_802755c0};
use crate::itlib3::{it_80272860, it_80274658, it_8027a344, it_coll_bounce_off_victim};
use crate::layout::{
    itPikachuthunderAttributes, itRshellAttributes, itTaruCann_DatAttrs,
    itThunderPokemonAttributes, itUnknownAttributes, HSD_GObj, Item, ItLGunAttr,
    ItemCommonData, MaximTomatoSpecialAttr, ScopeBeamAttrs, Vec as GcVec, HitCapsule_Enabled,
};
use melee_core::fp::fmadds;
use std::ptr::null_mut;

/// The kind's special attributes (`ip->xC4_article_data->x4_specialAttributes`) as a `T`.
/// # Safety
/// The article data and its special attributes must be valid as a `T` for `'a`.
unsafe fn sa<'a, T>(ip: &Item) -> &'a T {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const T)
}

/// `it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
unsafe fn fall(ip: &mut Item) {
    let a = &*ip.xCC_item_attr;
    it_80272860(ip, a.x10_fall_speed, a.x14_fall_speed_max);
}

/// The C `ABS(x)` macro, `((x) < 0 ? -(x) : (x))` (keeps the sign of a zero, unlike `abs`).
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

fn gp(p: *mut HSD_GObj) -> *mut HSD_GObj {
    p
}

// ---- getters / setters of one field ------------------------------------------------------

/// `itSeakNeedleThrown_Logic109_Destroyed` (itseakneedlethrown.c): `owner = NULL`.
pub fn it_seak_needle_thrown_logic109_destroyed(ip: &mut Item) {
    ip.owner = null_mut();
}

/// `it_8028ECE0` (itfreeze.c): `freeze.unk_1C = NULL`.
pub fn it_8028ece0(ip: &mut Item) {
    ip.xDD4_itemVar.freeze.unk_1C = null_mut();
}

/// `it_802B1DEC` (itpikachuthunder.c): `pikachuthunder.x4`.
pub fn it_802b1dec(ip: &Item) -> i32 {
    unsafe { ip.xDD4_itemVar.pikachuthunder.x4 }
}

/// `it_802B3544` (itpikachutjoltground.c): `pikachujoltground.xDDC = 0`.
pub fn it_802b3544(ip: &mut Item) {
    ip.xDD4_itemVar.pikachujoltground.xDDC = null_mut();
}

/// `it_802E5EF4` (it_2E5A.c): `it_2E5A.x0`.
pub fn it_802e5ef4(ip: &Item) -> i32 {
    unsafe { ip.xDD4_itemVar.it_2E5A.x0 }
}

/// `it_802ECC8C` (ittincle.c): `tincle.x20`.
pub fn it_802ecc8c(ip: &Item) -> i32 {
    unsafe { ip.xDD4_itemVar.tincle.x20 }
}

/// `it_802ECC98` (ittincle.c): `tincle.x54 = arg1`.
pub fn it_802ecc98(ip: &mut Item, arg1: f32) {
    ip.xDD4_itemVar.tincle.x54 = arg1;
}

/// `it_802ECD1C` (itkyasarin.c): `*v = pos`.
pub fn it_802ecd1c(ip: &Item, v: &mut GcVec) {
    *v = ip.pos;
}

/// `it_802F2014` (itcoin.c): `x378_itemColl.joint_id_only = arg1`.
pub fn it_802f2014(ip: &mut Item, arg1: i32) {
    ip.x378_itemColl.joint_id_only = arg1;
}

/// `itSamusGrapple_Logic53_Spawned` (itsamusgrapple.c): `samusgrapple.unk_10 = NULL`.
pub fn it_samus_grapple_logic53_spawned(ip: &mut Item) {
    ip.xDD4_itemVar.samusgrapple.unk_10 = None;
}

/// `it_802A2418` (itlinkhookshot.c): `linkhookshot.x10 = NULL`.
pub fn it_802a2418(ip: &mut Item) {
    ip.xDD4_itemVar.linkhookshot.x10 = None;
}

/// `itZeldadinfireexplode_UnkMotion0_Phys` (itzeldadinfireexplode.c):
/// `vel.z = vel.y = vel.z = 0` (x is left alone).
pub fn it_zeldadinfireexplode_unk_motion0_phys(ip: &mut Item) {
    ip.x40_vel.y = 0.0;
    ip.x40_vel.z = 0.0;
}

/// `it_802EAF28` (itgreatfoxlaser.c): the s32 at item var +0x38 (`likelike.x38`).
pub fn it_802eaf28(ip: &Item) -> i32 {
    unsafe { ip.xDD4_itemVar.likelike.x38 }
}

/// `it_8028428C` (ittomato.c): `tomato.heal_amount = sa->heal_amount_1`.
/// # Safety
/// The special attributes must be a valid `MaximTomatoSpecialAttr`.
pub unsafe fn it_8028428c(ip: &mut Item) {
    let a: &MaximTomatoSpecialAttr = sa(ip);
    ip.xDD4_itemVar.tomato.heal_amount = a.heal_amount_1;
}

/// `it_80292EF8` (itfflower.c): `(0, 5.5, 2.5)`.
pub fn it_80292ef8(vec: &mut GcVec) {
    vec.x = 0.0;
    vec.y = 5.5;
    vec.z = 2.5;
}

/// `it_802960B8` (ittarucann.c): `sa->x2C`.
/// # Safety
/// The special attributes must be a valid `itTaruCann_DatAttrs`.
pub unsafe fn it_802960b8(ip: &Item) -> i32 {
    sa::<itTaruCann_DatAttrs>(ip).x2C
}

/// `it_802960CC` (ittarucann.c): `sa->x28`.
/// # Safety
/// The special attributes must be a valid `itTaruCann_DatAttrs`.
pub unsafe fn it_802960cc(ip: &Item) -> i32 {
    sa::<itTaruCann_DatAttrs>(ip).x28
}

/// `it_802E37A4` (itwhitebea.c): `freeze.x20 = NULL` if there is an item.
pub fn it_802e37a4(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.xDD4_itemVar.freeze.x20 = null_mut();
    }
}

/// `it_802F046C` (itmasterhandlaser.c): `masterhandlaser.x0 = true` if there is an item.
pub fn it_802f046c(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.xDD4_itemVar.masterhandlaser.x0 = 1;
    }
}

/// `it_8028E774` (itlgun.c): `*arg1 = sa->pos`.
/// # Safety
/// The special attributes must be a valid `ItLGunAttr`.
pub unsafe fn it_8028e774(ip: &Item, arg1: &mut GcVec) {
    let a: &ItLGunAttr = sa(ip);
    arg1.x = a.pos.x;
    arg1.y = a.pos.y;
    arg1.z = a.pos.z;
}

/// `it_802DFFA0` (itzgshell.c): `zgshell.vel = 0`.
pub fn it_802dffa0(ip: &mut Item) {
    ip.xDD4_itemVar.zgshell.vel.x = 0.0;
    ip.xDD4_itemVar.zgshell.vel.y = 0.0;
    ip.xDD4_itemVar.zgshell.vel.z = 0.0;
}

/// `it_802ECCA4` (ittincle.c): `*pos = pos + (0, tincle.x50, 0)`, `*kind = msid`.
pub fn it_802ecca4(ip: &Item, kind: &mut i32, pos: &mut GcVec) {
    *pos = ip.pos;
    pos.y += unsafe { ip.xDD4_itemVar.tincle.x50 };
    *kind = ip.msid;
}

/// `it_8028ECF0` (itfreeze.c): `*v = pos`, returns the item.
pub fn it_8028ecf0(ip: &mut Item, v: &mut GcVec) -> *mut Item {
    v.x = ip.pos.x;
    v.y = ip.pos.y;
    v.z = ip.pos.z;
    ip
}

/// `it_802B43B0` (itpikachutjoltair.c): `pikachujoltair.xDD8 = 0` if there is an item.
pub fn it_802b43b0(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.xDD4_itemVar.pikachujoltair.xDD8 = null_mut();
    }
}

/// `it_802AC5D8` (itnesspkthundertrail.c): clear `owner` and `xDC8_word.flags.x13`.
pub fn it_802ac5d8(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.owner = null_mut();
        ip.xDC8_word.flags.set_x13(0);
    }
}

/// `it_802B3EFC` (itpikachutjoltair.c): `pikachujoltair.xDD8`, or NULL without an item.
pub fn it_802b3efc(ip: Option<&Item>) -> *mut HSD_GObj {
    match ip {
        Some(ip) => unsafe { ip.xDD4_itemVar.pikachujoltair.xDD8 },
        None => null_mut(),
    }
}

/// `it_802C3D44` (itzeldadinfire.c): clear `owner`, `zeldadinfire.xDE0` and `flags.x13`.
pub fn it_802c3d44(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.owner = null_mut();
        ip.xDD4_itemVar.zeldadinfire.xDE0 = null_mut();
        ip.xDC8_word.flags.set_x13(0);
    }
}

/// `it_802AB568` (itnesspkthunderball.c): `owner`, or NULL without an item.
pub fn it_802ab568(ip: Option<&Item>) -> *mut HSD_GObj {
    match ip {
        Some(ip) => ip.owner,
        None => null_mut(),
    }
}

/// `it_802B3368` (itpikachutjoltground.c): `pikachujoltground.xDDC`, or NULL.
pub fn it_802b3368(ip: Option<&Item>) -> *mut HSD_GObj {
    match ip {
        Some(ip) => unsafe { ip.xDD4_itemVar.pikachujoltground.xDDC },
        None => null_mut(),
    }
}

/// `it_802AAA50` (itnesspkflash.c): clear `owner`, `pkflush.xDE0_PKFlash_Owner`, `flags.x13`.
pub fn it_802aaa50(ip: Option<&mut Item>) {
    if let Some(ip) = ip {
        ip.owner = null_mut();
        ip.xDD4_itemVar.pkflush.xDE0_PKFlash_Owner = null_mut();
        ip.xDC8_word.flags.set_x13(0);
    }
}

/// `it_802A8398` (itlinkarrow.c): `pos = *pos`, `linkarrow.x18 = *pos2` if there is an item.
pub fn it_802a8398(ip: Option<&mut Item>, pos: &GcVec, pos2: &GcVec) {
    if let Some(ip) = ip {
        ip.pos = *pos;
        ip.xDD4_itemVar.linkarrow.x18 = *pos2;
    }
}

/// `it_802AB3F0` (itnesspkthunderball.c): `*out = pkthunder.positions[idx * 2]`, zero without
/// an item; nothing without `out`.
pub fn it_802ab3f0(ip: Option<&Item>, out: Option<&mut GcVec>, idx: i32) {
    match (ip, out) {
        (Some(ip), Some(out)) => *out = unsafe { ip.xDD4_itemVar.pkthunder.positions[(idx * 2) as usize] },
        (None, Some(out)) => {
            out.x = 0.0;
            out.y = 0.0;
            out.z = 0.0;
        }
        (_, None) => {}
    }
}

/// `it_802AB468` (itnesspkthunderball.c): `*out = pkthunder.xEEC`, zero without an item.
pub fn it_802ab468(ip: Option<&Item>, out: Option<&mut f32>) {
    if let Some(out) = out {
        *out = match ip {
            Some(ip) => unsafe { ip.xDD4_itemVar.pkthunder.xEEC },
            None => 0.0,
        };
    }
}

// ---- reference cleanup (EvtUnk) ------------------------------------------------------------

/// `itSamusBomb_Logic50_EvtUnk` (itsamusbomb.c): `it_8026B894`, then `samusbomb.owner = NULL`.
pub fn it_samus_bomb_logic50_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    ip.xDD4_itemVar.samusbomb.owner = null_mut();
}

/// `it_802D43B0` (ithitodeman.c): `it_8026B894`, `hitodeman.x90 = NULL`, `xDAC = 1`.
pub fn it_802d43b0(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    ip.xDD4_itemVar.hitodeman.x90 = null_mut();
    ip.xDAC_itcmd_var0 = 1;
}

/// `itClimbersIce_Logic90_EvtUnk` (itclimbersice.c): forget `climbersice.x0` if it is
/// `ref_gobj`, then `it_8026B894`.
pub fn it_climbers_ice_logic90_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    unsafe {
        if gp(ip.xDD4_itemVar.climbersice.x0) == ref_gobj {
            ip.xDD4_itemVar.climbersice.x0 = null_mut();
        }
    }
    it_8026b894(ip, ref_gobj);
}

/// `itPikachuThunder_Logic39_EvtUnk` (itpikachuthunder.c): forget `pikachuthunder.x38`, then
/// `it_8026B894`.
pub fn it_pikachu_thunder_logic39_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    unsafe {
        if gp(ip.xDD4_itemVar.pikachuthunder.x38) == ref_gobj {
            ip.xDD4_itemVar.pikachuthunder.x38 = null_mut();
        }
    }
    it_8026b894(ip, ref_gobj);
}

/// `itSamusGrapple_Logic53_EvtUnk` (itsamusgrapple.c): `it_8026B894`, then forget
/// `samusgrapple.x8`.
pub fn it_samus_grapple_logic53_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    unsafe {
        if gp(ip.xDD4_itemVar.samusgrapple.x8) == ref_gobj {
            ip.xDD4_itemVar.samusgrapple.x8 = null_mut();
        }
    }
}

/// `itSeakChain_Logic54_EvtUnk` (itseakchain.c): `it_8026B894`, then forget
/// `seakchain.parent_gobj`.
pub fn it_seak_chain_logic54_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    unsafe {
        if gp(ip.xDD4_itemVar.seakchain.parent_gobj) == ref_gobj {
            ip.xDD4_itemVar.seakchain.parent_gobj = null_mut();
        }
    }
}

/// `it_2725_Logic52_EvtUnk` (itsamusmissile.c): `it_8026B894`, then forget
/// `samusmissile.owner`.
pub fn it_2725_logic52_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    unsafe {
        if gp(ip.xDD4_itemVar.samusmissile.owner) == ref_gobj {
            ip.xDD4_itemVar.samusmissile.owner = null_mut();
        }
    }
}

/// `it_802A7D40` (itlinkhookshot.c): `it_8026B894`, then forget `linkhookshot.x8`.
pub fn it_802a7d40(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    unsafe {
        if gp(ip.xDD4_itemVar.linkhookshot.x8) == ref_gobj {
            ip.xDD4_itemVar.linkhookshot.x8 = null_mut();
        }
    }
}

/// `it_8029FD84` (itlinkbomb.c): forget `linkbomb.x10`, then `it_8026B894`.
pub fn it_8029fd84(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    unsafe {
        if gp(ip.xDD4_itemVar.linkbomb.x10) == ref_gobj {
            ip.xDD4_itemVar.linkbomb.x10 = null_mut();
        }
    }
    it_8026b894(ip, ref_gobj);
}

/// `itMewtwoDisable_Logic67_EvtUnk` (itmewtwodisable.c): forget `mdisable.owner`, then
/// `it_8026B894`.
pub fn it_mewtwo_disable_logic67_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    unsafe {
        if gp(ip.xDD4_itemVar.mdisable.owner) == ref_gobj {
            ip.xDD4_itemVar.mdisable.owner = null_mut();
        }
    }
    it_8026b894(ip, ref_gobj);
}

/// `itPeachTurnip_Logic56_EvtUnk` (itpeachturnip.c): forget `peachturnip.xDE4_owner`, then
/// `it_8026B894`.
pub fn it_peach_turnip_logic56_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    unsafe {
        if gp(ip.xDD4_itemVar.peachturnip.xDE4_owner) == ref_gobj {
            ip.xDD4_itemVar.peachturnip.xDE4_owner = null_mut();
        }
    }
    it_8026b894(ip, ref_gobj);
}

/// `it_802A23CC` (itlinkboomerang.c): `it_8026B894`, then forget `linkboomerang.xF98`.
pub fn it_802a23cc(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    unsafe {
        if gp(ip.xDD4_itemVar.linkboomerang.xF98) == ref_gobj {
            ip.xDD4_itemVar.linkboomerang.xF98 = null_mut();
        }
    }
}

// ---- physics / animation callbacks -------------------------------------------------------

/// `itCapsule_UnkMotion3_Phys` (itcapsule.c): `itCapsule_UnkMotion4_Phys`, then
/// `it_80274658(gobj, it_804D6D28->x68_float)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_capsule_unk_motion3_phys(ip: &mut Item, common: &ItemCommonData) {
    it_capsule_unk_motion4_phys(ip);
    it_80274658(ip, common.x68_float);
}

/// `itGshell_UnkMotion4_Anim` (itgshell.c): `it_8028B988`, false.
/// # Safety
/// As `it_8028b988`.
pub unsafe fn it_gshell_unk_motion4_anim(ip: &mut Item) -> bool {
    it_8028b988(ip);
    false
}

/// `itStarrod_UnkMotion1_Phys` (itstarrod.c): fall.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_starrod_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTosakinto_UnkMotion2_Phys` (ittosakinto.c): `it_8027A344`, `vel.x += tosakinto.x64_vel_x`.
pub fn it_tosakinto_unk_motion2_phys(ip: &mut Item) {
    it_8027a344(ip);
    ip.x40_vel.x += unsafe { ip.xDD4_itemVar.tosakinto.x64_vel_x };
}

/// `it_2E5A_UnkMotion1_Anim` (it_2E5A.c): `animFrameSpeed = 0.1 * ABS(vel.y) + 1.0` (in f64,
/// one fused f64 site), then `it_802E657C`.
pub fn it_2e5a_unk_motion1_anim(ip: &mut Item) -> bool {
    ip.x5D0_animFrameSpeed = 0.1f64.mul_add(c_abs(ip.x40_vel.y) as f64, 1.0) as f32;
    it_802e657c(ip)
}

/// `it_8028EC98` (itfreeze.c): `freeze.x10 = vel`, lifetime `it_804D6D28->x30_lifetime`.
pub fn it_8028ec98(ip: &mut Item, vel: f32, common: &ItemCommonData) {
    ip.xDD4_itemVar.freeze.x10 = vel;
    it_80275158(ip, common.x30_lifetime as f32, common);
}

/// `it_802F23AC` (itkirby_2F23.c): squared distance from `pos` (two `fmadds`), as f64.
pub fn it_802f23ac(ip: &Item, pos: &GcVec) -> f64 {
    let dx = ip.pos.x - pos.x;
    let dy = ip.pos.y - pos.y;
    let dz = ip.pos.z - pos.z;
    fmadds(dz, dz, fmadds(dx, dx, dy * dy)) as f64
}

/// `itBat_UnkMotion3_Phys` (itbat.c): fall, then `it_80274658(gobj, it_804D6D28->x68_float)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_bat_unk_motion3_phys(ip: &mut Item, common: &ItemCommonData) {
    fall(ip);
    it_80274658(ip, common.x68_float);
}

/// `itHammerhead_UnkMotion2_Phys` (ithammerhead.c): as `itBat_UnkMotion3_Phys`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_hammerhead_unk_motion2_phys(ip: &mut Item, common: &ItemCommonData) {
    fall(ip);
    it_80274658(ip, common.x68_float);
}

/// `itUnknown_UnkMotion0_Phys` (itunknown.c): fall with the kind's speeds, then
/// `vel.x += sa->x4 * unknown.x60` (fused).
/// # Safety
/// The special attributes must be a valid `itUnknownAttributes`.
pub unsafe fn it_unknown_unk_motion0_phys(ip: &mut Item) {
    let a: &itUnknownAttributes = sa(ip);
    it_80272860(ip, a.xC, a.x10);
    ip.x40_vel.x = fmadds(a.x4, ip.xDD4_itemVar.unknown.x60, ip.x40_vel.x);
}

/// `itZrshell_UnkMotion11_Phys` (itzgshell.c): `it_802762BC` while rising.
pub fn it_zrshell_unk_motion11_phys(ip: &mut Item) {
    if ip.x40_vel.y > 0.0 {
        it_802762bc(ip);
    }
}

/// `it_27CF_UnkMotion1_Phys` (ityoshiegglay.c): fall.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_27cf_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `it_8028D56C` (itrshell.c): `((f1 * -x14.x) + (-x14.y * f2)) * x14.z` (first sum fused).
/// # Safety
/// The special attributes must be a valid `itRshellAttributes`.
pub unsafe fn it_8028d56c(ip: &Item, f1: f32, f2: f32) -> f32 {
    let a: &itRshellAttributes = sa(ip);
    let f = fmadds(f1, -a.x14.x, -a.x14.y * f2);
    f * a.x14.z
}

/// `itSword_Logic12_DmgDealt` (itsword.c): bounce off the victim in motion state 3, false.
pub fn it_sword_logic12_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    if ip.msid == 3 {
        it_coll_bounce_off_victim(ip, common);
    }
    false
}

/// `itTomato_UnkMotion0_Phys` (ittomato.c): fall.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_tomato_unk_motion0_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFoods_UnkMotion0_Phys` (itfoods.c): fall.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_foods_unk_motion0_phys(ip: &mut Item) {
    fall(ip);
}

/// `it_27CF_Logic114_DmgReceived` (ityoshiegglay.c): `lifeTimer -= xCA0 * yoshiegglay.x0`
/// (fused), false.
pub fn it_27cf_logic114_dmg_received(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer = fmadds(-(ip.xCA0 as f32), unsafe { ip.xDD4_itemVar.yoshiegglay.x0 }, ip.xD44_lifeTimer);
    false
}

/// `itTaru_UnkMotion7_Anim` (ittaru.c): count `taru.xDD8` down; true when it reaches 0.
pub fn it_taru_unk_motion7_anim(ip: &mut Item) -> bool {
    unsafe {
        let t = &mut ip.xDD4_itemVar.taru;
        t.xDD8 = t.xDD8.wrapping_sub(1);
        t.xDD8 <= 0
    }
}

/// `itTarucann_UnkMotion2_Anim` (ittarucann.c): `xDCC_flag.b3 = (tarucann.x20 == NULL)`, false.
pub fn it_tarucann_unk_motion2_anim(ip: &mut Item) -> bool {
    let none = unsafe { ip.xDD4_itemVar.tarucann.x20.is_null() };
    ip.xDCC_flag.set_b3(none as u8);
    false
}

/// `itWstar_UnkMotion5_Anim` (itwstar.c): on command var 0, rescale the hitboxes by
/// `wstar.xDD8` and clear it; then the life-timer tick.
pub fn it_wstar_unk_motion5_anim(ip: &mut Item) -> bool {
    if ip.xDAC_itcmd_var0 != 0 {
        let s = unsafe { ip.xDD4_itemVar.wstar.xDD8 };
        it_802755c0(ip, s);
        ip.xDAC_itcmd_var0 = 0;
    }
    it_802751d8(ip)
}

/// `itWhispyapple_UnkMotion0_Phys` (itwhispyapple.c): `it_8026B3A8` when `whispyapple.xDD8 == 1`,
/// then fall.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whispyapple_unk_motion0_phys(ip: &mut Item) {
    if ip.xDD4_itemVar.whispyapple.xDD8 == 1 {
        it_8026b3a8(ip);
    }
    fall(ip);
}

/// `it_27CF_UnkMotion2_Anim` (ityoshiegglay.c): life-timer tick.
pub fn it_27cf_unk_motion2_anim(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer -= 1.0;
    ip.xD44_lifeTimer <= 0.0
}

/// `it_802B1FE8` (itpikachuthunder.c): `*pos = pos - (0, sa->x8 * pikachuthunder.x18, 0)`.
/// # Safety
/// The special attributes must be a valid `itPikachuthunderAttributes`.
pub unsafe fn it_802b1fe8(ip: &Item, pos: &mut GcVec) {
    let a: &itPikachuthunderAttributes = sa(ip);
    let scale = ip.xDD4_itemVar.pikachuthunder.x18;
    let mut offset = -a.x8;
    offset *= scale;
    *pos = ip.pos;
    pos.y += offset;
}

/// `fn_802C5E18` (itmewtwoshadowball.c): `mewtwoshadowball.x20 = (x20 + 1) % 3`.
pub fn fn_802c5e18(ip: &mut Item) {
    unsafe {
        let v = &mut ip.xDD4_itemVar.mewtwoshadowball;
        v.x20 = v.x20.wrapping_add(1);
        v.x20 %= 3;
    }
}

/// `itFlipper_Inflight_Phys` (itflipper.c): fall, damp the velocity by `t / (1 + t)` over the
/// last 10 flight frames, then `it_80274658(gobj, it_804D6D28->x68_float)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_flipper_inflight_phys(ip: &mut Item, common: &ItemCommonData) {
    fall(ip);
    let t = ip.xDD4_itemVar.flipper.xDD4_flightTimer;
    if t <= 0xA {
        ip.x40_vel.x *= t as f32 / (1.0 + t as f32);
        ip.x40_vel.y *= t as f32 / (1.0 + t as f32);
    }
    it_80274658(ip, common.x68_float);
}

/// `itThunder_UnkMotion2_Phys` (itthunder.c): pokemon drift, then `thunder.x68` (vertical
/// speed) starts at `sa->x4` on command var 0 and grows by `sa->x8` while command var 1 is set.
/// # Safety
/// The special attributes must be a valid `itThunderPokemonAttributes`.
pub unsafe fn it_thunder_unk_motion2_phys(ip: &mut Item) {
    it_8027a344(ip);
    if ip.xDAC_itcmd_var0 != 0 {
        let a: &itThunderPokemonAttributes = sa(ip);
        ip.xDD4_itemVar.thunder.x68 = a.x4;
        ip.xDAC_itcmd_var0 = 0;
        ip.xDB0_itcmd_var1 = 1;
    }
    if ip.xDB0_itcmd_var1 != 0 {
        let a: &itThunderPokemonAttributes = sa(ip);
        ip.xDD4_itemVar.thunder.x68 += a.x8;
    }
    ip.x40_vel.y = ip.xDD4_itemVar.thunder.x68;
}

/// `itSscopebeam_UnkMotion9_Anim` (itsscopebeam.c): at the beam level's start frame scale
/// hitbox 0; then tick the life timer (clamped at 0, true when it runs out).
/// # Safety
/// The special attributes must be a valid `ScopeBeamAttrs`; `scopebeam.x0` must index `floats`.
pub unsafe fn it_sscopebeam_unk_motion9_anim(ip: &mut Item) -> bool {
    let index = ip.xDD4_itemVar.scopebeam.x0;
    let attrs: &ScopeBeamAttrs = sa(ip);
    let data = &attrs.floats[index as usize];
    if ip.xD44_lifeTimer == data.lifetime && ip.x5D4_hitboxes[0].hit.state >= HitCapsule_Enabled {
        ip.x5D4_hitboxes[0].hit.scale *= data.scale;
    }
    ip.xD44_lifeTimer -= 1.0;
    if ip.xD44_lifeTimer <= 0.0 {
        ip.xD44_lifeTimer = 0.0;
        return true;
    }
    false
}

// ---- sword (beam sword) blend state --------------------------------------------------------

/// `it_80284FC4` (itsword.c): start blend A towards `arg1` steps / value `arg2`.
pub fn it_80284fc4(ip: &mut Item, arg1: i32, arg2: f32) {
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        s.x4 = 0;
        s.x8 = arg1;
        s.xC = 1;
        s.x10 = 1.0;
        s.x14 = arg2;
        s.x18 = (s.x14 - s.x10) / arg1 as f32;
    }
}

/// `it_80285024` (itsword.c): blend A back down over `arg1` steps to 1.0.
pub fn it_80285024(ip: &mut Item, arg1: i32) {
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        s.x4 = arg1;
        s.x8 = 0;
        s.xC = -1;
        s.x14 = 1.0;
        s.x18 = (s.x14 - s.x10) / s.x4 as f32;
    }
}

/// One clamped integer step (`*v += step`, stop at `target`) of `it_80285084` and friends.
fn step_i(v: &mut i32, target: i32, step: &mut i32) {
    *v = v.wrapping_add(*step);
    if *step > 0 {
        if *v > target {
            *v = target;
            *step = 0;
        }
    } else if *step < 0 && *v < target {
        *v = target;
        *step = 0;
    }
}

/// One clamped float step (`*v += step`, stop at `target`).
fn step_f(v: &mut f32, target: f32, step: &mut f32) {
    *v += *step;
    if *step > 0.0 {
        if *v > target {
            *v = target;
            *step = 0.0;
        }
    } else if *step < 0.0 && *v < target {
        *v = target;
        *step = 0.0;
    }
}

/// `it_80285084` (itsword.c): step blend A (`x4`->`x8` by `xC`, `x10`->`x14` by `x18`).
pub fn it_80285084(ip: &mut Item) {
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        step_i(&mut s.x4, s.x8, &mut s.xC);
        step_f(&mut s.x10, s.x14, &mut s.x18);
    }
}

/// `it_80285140` (itsword.c): step blend B (`x1C`->`x20` by `x24`, `x28`->`x2C` by `x30`).
pub fn it_80285140(ip: &mut Item) {
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        step_i(&mut s.x1C, s.x20, &mut s.x24);
        step_f(&mut s.x28, s.x2C, &mut s.x30);
    }
}

/// `it_802851FC` (itsword.c): step blend C (`x34`->`x38` by `x3C`, `x40`->`x44` by `x48`).
pub fn it_802851fc(ip: &mut Item) {
    unsafe {
        let s = &mut ip.xDD4_itemVar.sword;
        step_i(&mut s.x34, s.x38, &mut s.x3C);
        step_f(&mut s.x40, s.x44, &mut s.x48);
    }
}

/// `it_802852B8` (itsword.c): `*arg1 = 0`, `*arg2 = 10.03 * (sword.x4C * star.xvel *
/// sword.x28 * 0.3)` (`star.xvel` is the same word as `sword.x10`).
pub fn it_802852b8(ip: &Item, arg1: Option<&mut f32>, arg2: Option<&mut f32>) {
    let f1 = unsafe {
        ip.xDD4_itemVar.sword.x4C * ip.xDD4_itemVar.star.xvel * ip.xDD4_itemVar.sword.x28 * 0.3
    };
    if let Some(a) = arg1 {
        *a = 0.0;
    }
    if let Some(b) = arg2 {
        *b = 10.03 * f1;
    }
}

// ---- pure ------------------------------------------------------------------------------

/// `it_802D48B0` (ithitodeman.c): move `*value` towards `target` by at most `accel` (or at
/// least `decel`), clamp to `+-max_val`, store and return it.
pub fn it_802d48b0(value: &mut f32, mut target: f32, max_val: f32, accel: f32, decel: f32) -> f32 {
    let current = *value;
    let diff = target - current;
    if c_abs(diff) > c_abs(accel) {
        target = if diff > 0.0 { current + accel } else { current - accel };
    } else if c_abs(diff) < c_abs(decel) {
        target = if diff > 0.0 { current + decel } else { current - decel };
    }
    if target > 0.0 {
        if target > max_val {
            target = max_val;
        }
    } else if target < -max_val {
        target = -max_val;
    }
    *value = target;
    target
}
