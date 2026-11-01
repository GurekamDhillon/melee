
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
pub(crate) unsafe fn sa<'a, T>(ip: &Item) -> &'a T {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const T)
}

/// `it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub(crate) unsafe fn fall(ip: &mut Item) {
    let a = &*ip.xCC_item_attr;
    it_80272860(ip, a.x10_fall_speed, a.x14_fall_speed_max);
}

/// The C `ABS(x)` macro, `((x) < 0 ? -(x) : (x))` (keeps the sign of a zero, unlike `abs`).
pub(crate) fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

pub(crate) fn gp(p: *mut HSD_GObj) -> *mut HSD_GObj {
    p
}

// ---- getters / setters of one field ------------------------------------------------------

/// `itSeakNeedleThrown_Logic109_Destroyed` (itseakneedlethrown.c): `owner = NULL`.
pub fn it_seak_needle_thrown_logic109_destroyed(ip: &mut Item) {
    ip.owner = null_mut();
}

/// `itSamusGrapple_Logic53_Spawned` (itsamusgrapple.c): `samusgrapple.unk_10 = NULL`.
pub fn it_samus_grapple_logic53_spawned(ip: &mut Item) {
    ip.xDD4_itemVar.samusgrapple.unk_10 = None;
}

/// `itZeldadinfireexplode_UnkMotion0_Phys` (itzeldadinfireexplode.c):
/// `vel.z = vel.y = vel.z = 0` (x is left alone).
pub fn it_zeldadinfireexplode_unk_motion0_phys(ip: &mut Item) {
    ip.x40_vel.y = 0.0;
    ip.x40_vel.z = 0.0;
}

// ---- reference cleanup (EvtUnk) ------------------------------------------------------------

/// `itSamusBomb_Logic50_EvtUnk` (itsamusbomb.c): `it_8026B894`, then `samusbomb.owner = NULL`.
pub fn it_samus_bomb_logic50_evt_unk(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    it_8026b894(ip, ref_gobj);
    ip.xDD4_itemVar.samusbomb.owner = null_mut();
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

/// One clamped integer step (`*v += step`, stop at `target`) of `it_80285084` and friends.
pub(crate) fn step_i(v: &mut i32, target: i32, step: &mut i32) {
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
pub(crate) fn step_f(v: &mut f32, target: f32, step: &mut f32) {
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

pub use crate::tu::itmewtwoshadowball::fn_802c5e18;
pub use crate::tu::ittomato::it_8028428c;
pub use crate::tu::itsword::it_80284fc4;
pub use crate::tu::itsword::it_80285024_fp as it_80285024;
pub use crate::tu::itsword::it_80285084_fp as it_80285084;
pub use crate::tu::itsword::it_80285140_fp as it_80285140;
pub use crate::tu::itsword::it_802851fc;
pub use crate::tu::itsword::it_802852b8;
pub use crate::tu::itrshell::it_8028d56c;
pub use crate::tu::itlgun::it_8028e774;
pub use crate::tu::itfreeze::it_8028ec98;
pub use crate::tu::itfreeze::it_8028ece0;
pub use crate::tu::itfreeze::it_8028ecf0;
pub use crate::tu::itfflower::it_80292ef8;
pub use crate::tu::ittarucann::it_802960b8;
pub use crate::tu::ittarucann::it_802960cc;
pub use crate::tu::itlinkbomb::it_8029fd84;
pub use crate::tu::itlinkboomerang::it_802a23cc;
pub use crate::tu::itlinkhookshot::it_802a2418;
pub use crate::tu::itlinkhookshot::it_802a7d40;
pub use crate::tu::itlinkarrow::it_802a8398;
pub use crate::tu::itnesspkflash::it_802aaa50;
pub use crate::tu::itnesspkthunderball::it_802ab3f0;
pub use crate::tu::itnesspkthunderball::it_802ab468;
pub use crate::tu::itnesspkthunderball::it_802ab568;
pub use crate::tu::itnesspkthundertrail::it_802ac5d8;
pub use crate::tu::itpikachuthunder::it_802b1dec;
pub use crate::tu::itpikachuthunder::it_802b1fe8;
pub use crate::tu::itpikachutjoltground::it_802b3368;
pub use crate::tu::itpikachutjoltground::it_802b3544;
pub use crate::tu::itpikachutjoltair::it_802b3efc;
pub use crate::tu::itpikachutjoltair::it_802b43b0;
pub use crate::tu::itzeldadinfire::it_802c3d44;
pub use crate::tu::ithitodeman::it_802d43b0;
pub use crate::tu::ithitodeman::it_802d48b0;
pub use crate::tu::itzgshell::it_802dffa0;
pub use crate::tu::itwhitebea::it_802e37a4;
pub use crate::tu::it_2E5A::it_802e5ef4;
pub use crate::tu::itgreatfoxlaser::it_802eaf28;
pub use crate::tu::ittincle::it_802ecc8c;
pub use crate::tu::ittincle::it_802ecc98;
pub use crate::tu::ittincle::it_802ecca4;
pub use crate::tu::itkyasarin::it_802ecd1c;
pub use crate::tu::itmasterhandlaser::it_802f046c;
pub use crate::tu::itcoin::it_802f2014;
pub use crate::tu::itkirby_2F23::it_802f23ac;
