
//! Item kinds, port round 4 (night 3): per-kind state callbacks (`src/melee/it/kinds/*.c`) whose
//! bodies touch only the Item, its `ItemAttr`, its kind's special attributes and item vars, and
//! call only functions already ported (`itlib.rs`, `itlib3.rs`, `item.rs`, `helpers.rs`). Each
//! function cites the C it mirrors; all are verified against verbatim C copies in
//! tests/c_oracle/round4_bodies.h (tests/oracle_round4.rs).
//!
//! Conventions as in `itlib3.rs`: the `Item_GObj*` becomes the `Item`; reading `xCC_item_attr`
//! or the article's special attributes makes a function `unsafe` (precondition: valid); the
//! `it_804D6D28` global is `&ItemCommonData`. The first ~140 functions are generated from five
//! one-statement shapes shared by many kinds (fall, fall while airborne, bounce off the victim,
//! `vel.y -= fall_speed` (airborne), bounce back); the rest are hand ported.
//!
//! Fused multiply-adds (`tools/rust-port/fma_sites.py` on every source TU, shipping
//! contraction): `itHitodeman_UnkMotion1_Phys`, `itKamex_UnkMotion1_Phys`,
//! `itLinkbomb_UnkMotion4_Phys`, `itSamusmissile_UnkMotion1_Phys` (1 f32 site each, `fmadds`);
//! every other function here has none.
#![allow(non_upper_case_globals, non_snake_case)]

use crate::helpers::{it_80275158, it_rshell_clamp_vel, itRShell_Attrs};

use crate::item::{decelerate_item_x, it_802762bc};

use crate::itlib::{
    it_8026b3a8, it_80273030, it_80273130, it_80275444, it_802756d0, it_8027572c,
};

use crate::itlib3::{it_80272860, it_8027a344, it_802756e0, it_coll_bounce_off_victim};

use crate::layout::{
    itClimbersIceAttributes, itCoinAttributes, itDoseiAttributes, itFireAttributes,
    itFlipper_DatAttrs, itFreezerAttributes, itGamewatchchefAttributes, itHammerheadAttributes,
    itHitodemanAttributes, itHououAttr, itLeadeadAttributes, itLinkArrowAttributes,
    itLinkBombAttributes, itLizardonAttributes, itMatadogasAttributes, itPeachToadSporeAttributes,
    itSamusMissileAttributes, itsonansAttributes, HeartContainerAttr, HSD_GObj, Item, ItemAttr,
    ItemCommonData, KinokoAttrs, MewVars, Vec as GcVec, GA_Air, GA_Ground,
    It_Kind_Lugia_Aeroblast, It_Kind_Lugia_Aeroblast2, It_Kind_Lugia_Aeroblast3,
    It_Kind_Matadogas_Gas1, It_Kind_Matadogas_Gas2,
};

use melee_core::fp::fmadds;

use std::ptr::null_mut;

/// The item's `ItemAttr` (`ip->xCC_item_attr`).
/// # Safety
/// `ip.xCC_item_attr` must be valid for `'a`.
pub(crate) unsafe fn attr<'a>(ip: &Item) -> &'a ItemAttr {
    &*ip.xCC_item_attr
}

/// The kind's special attributes (`ip->xC4_article_data->x4_specialAttributes`) as a `T`.
/// # Safety
/// The article data and its special attributes must be valid as a `T` for `'a`.
pub(crate) unsafe fn sa<'a, T>(ip: &Item) -> &'a T {
    &*((*ip.xC4_article_data).x4_specialAttributes as *const T)
}

/// `it_80272860(gobj, attr->x10_fall_speed, attr->x14_fall_speed_max)`, the common fall step.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub(crate) unsafe fn fall(ip: &mut Item) {
    let a = attr(ip);
    it_80272860(ip, a.x10_fall_speed, a.x14_fall_speed_max);
}

// ---- generated: one-statement shapes ----------------------------------------------------

/// `itBombhei_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_bombhei_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itClimbersice_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_climbersice_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itDosei_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_dosei_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itDosei_UnkMotion8_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_dosei_unk_motion8_phys(ip: &mut Item) {
    fall(ip);
}

/// `itDosei_UnkMotion11_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_dosei_unk_motion11_phys(ip: &mut Item) {
    fall(ip);
}

/// `itEvyoshiegg_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_evyoshiegg_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFflower_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_fflower_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFflower_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_fflower_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFflowerflame_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_fflowerflame_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFlipper_Falling_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_flipper_falling_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFreeze_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_freeze_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFushigibana_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_fushigibana_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itGshell_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_gshell_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itGshell_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_gshell_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itGshell_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_gshell_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itGshell_UnkMotion8_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_gshell_unk_motion8_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHarisen_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_harisen_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHeart_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_heart_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHinoarashi_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_hinoarashi_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itKusudama_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kusudama_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLeadead_UnkMotion10_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_leadead_unk_motion10_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLeadead_UnkMotion14_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_leadead_unk_motion14_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLikelike_UnkMotion19_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_likelike_unk_motion19_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLipstick_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lipstick_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLucky_UnkMotion5_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lucky_unk_motion5_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLucky_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lucky_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLucky_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lucky_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMaril_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_maril_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMarumine_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_marumine_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMarumine_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_marumine_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMsbomb_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_msbomb_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOctarock_UnkMotion2_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarock_unk_motion2_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOctarock_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarock_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldkuri_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldkuri_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldkuri_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldkuri_UnkMotion8_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion8_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldottosea_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldottosea_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itPatapata_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_patapata_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itRshell_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rshell_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itRshell_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rshell_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itRshell_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rshell_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itRshell_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rshell_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itSamusbomb_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_samusbomb_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itScball_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_scball_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itSpycloak_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_spycloak_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itSpycloak_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_spycloak_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itStar_UnkMotion0_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_star_unk_motion0_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTaru_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_taru_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTarucann_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_tarucann_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTincle_UnkMotion8_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_tincle_unk_motion8_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTomato_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_tomato_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itTosakinto_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_tosakinto_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itWhispyapple_UnkMotion5_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whispyapple_unk_motion5_phys(ip: &mut Item) {
    fall(ip);
}

/// `itOldottosea_UnkMotion9_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldottosea_unk_motion9_phys(ip: &mut Item) {
    fall(ip);
}

/// `itWhitebea_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whitebea_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itWhitebea_UnkMotion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whitebea_unk_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itWhitebea_UnkMotion9_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whitebea_unk_motion9_phys(ip: &mut Item) {
    fall(ip);
}

/// `itWstar_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_wstar_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itYoshieggthrow_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_yoshieggthrow_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itZrshell_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_zrshell_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itZrshell_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_zrshell_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itZrshell_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_zrshell_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itZrshell_UnkMotion8_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_zrshell_unk_motion8_phys(ip: &mut Item) {
    fall(ip);
}

/// `itBat_Logic11_Clanked`: `itColl_BounceOffVictim`.
pub fn it_bat_logic11_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itBat_Logic11_HitShield`: `itColl_BounceOffVictim`.
pub fn it_bat_logic11_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itDosei_Logic7_Clanked`: `itColl_BounceOffVictim`.
pub fn it_dosei_logic7_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itDosei_Logic7_HitShield`: `itColl_BounceOffVictim`.
pub fn it_dosei_logic7_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itFFlower_Logic25_DmgDealt`: `itColl_BounceOffVictim`.
pub fn it_fflower_logic25_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itFFlower_Logic25_Clanked`: `itColl_BounceOffVictim`.
pub fn it_fflower_logic25_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itFFlower_Logic25_HitShield`: `itColl_BounceOffVictim`.
pub fn it_fflower_logic25_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itLGun_Logic16_DmgDealt`: `itColl_BounceOffVictim`.
pub fn it_lgun_logic16_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itLGun_Logic16_Clanked`: `itColl_BounceOffVictim`.
pub fn it_lgun_logic16_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itLGun_Logic16_HitShield`: `itColl_BounceOffVictim`.
pub fn it_lgun_logic16_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itParasol_Logic13_DmgDealt`: `itColl_BounceOffVictim`.
pub fn it_parasol_logic13_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itParasol_Logic13_Clanked`: `itColl_BounceOffVictim`.
pub fn it_parasol_logic13_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itParasol_Logic13_HitShield`: `itColl_BounceOffVictim`.
pub fn it_parasol_logic13_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itPeachTurnip_Logic56_Clanked`: `itColl_BounceOffVictim`.
pub fn it_peach_turnip_logic56_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itPeachTurnip_Logic56_HitShield`: `itColl_BounceOffVictim`.
pub fn it_peach_turnip_logic56_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itRShell_Logic15_Clanked`: `itColl_BounceOffVictim`.
pub fn it_rshell_logic15_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itScball_Logic30_HitShield`: `itColl_BounceOffVictim`.
pub fn it_scball_logic30_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itSScope_Logic21_DmgDealt`: `itColl_BounceOffVictim`.
pub fn it_sscope_logic21_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itSScope_Logic21_Clanked`: `itColl_BounceOffVictim`.
pub fn it_sscope_logic21_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itSScope_Logic21_HitShield`: `itColl_BounceOffVictim`.
pub fn it_sscope_logic21_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itSword_Logic12_Clanked`: `itColl_BounceOffVictim`.
pub fn it_sword_logic12_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itSword_Logic12_HitShield`: `itColl_BounceOffVictim`.
pub fn it_sword_logic12_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

/// `itKyasarin_UnkMotion9_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kyasarin_unk_motion9_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itLeadead_UnkMotion16_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_leadead_unk_motion16_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itLeadead_UnkMotion17_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_leadead_unk_motion17_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itLikelike_UnkMotion21_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_likelike_unk_motion21_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itOctarock_UnkMotion8_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarock_unk_motion8_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itOctarockstone_UnkMotion1_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarockstone_unk_motion1_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itOldkuri_UnkMotion11_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion11_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itOldottosea_UnkMotion11_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldottosea_unk_motion11_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itWhitebea_UnkMotion11_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whitebea_unk_motion11_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itZrshell_UnkMotion10_Phys`: `vel.y -= fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_zrshell_unk_motion10_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
}

/// `itEntei_UnkMotion0_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_entei_unk_motion0_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itKamex_UnkMotion2_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kamex_unk_motion2_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itPippi_UnkMotion5_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_pippi_unk_motion5_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itRaikou_UnkMotion0_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_raikou_unk_motion0_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itSuikun_UnkMotion0_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_suikun_unk_motion0_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itLeadead_UnkMotion12_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_leadead_unk_motion12_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itNokonoko_UnkMotion5_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nokonoko_unk_motion5_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itOctarock_UnkMotion3_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarock_unk_motion3_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itOldkuri_UnkMotion9_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldkuri_unk_motion9_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itOldottosea_UnkMotion5_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_oldottosea_unk_motion5_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itWhitebea_UnkMotion4_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_whitebea_unk_motion4_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `it_2ADA_UnkMotion0_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_2ada_unk_motion0_phys(ip: &mut Item) {
    fall(ip);
}

/// `it_2E5A_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_2e5a_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itCoin_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_coin_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itEgg_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_egg_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLgun_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lgun_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFoods_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_foods_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHammer_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_hammer_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHammer_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_hammer_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itHammerHead_Logic40_Reflected`: `it_80273030` (bounce back), result dropped.
pub fn it_hammer_head_logic40_reflected(ip: &mut Item) -> bool {
    it_80273030(ip);
    false
}

/// `itOctarockstone_Logic4_Reflected`: `it_80273030` (bounce back), result dropped.
pub fn it_octarockstone_logic4_reflected(ip: &mut Item) -> bool {
    it_80273030(ip);
    false
}

/// `itPeachTurnip_Logic56_Reflected`: `it_80273030` (bounce back), result dropped.
pub fn it_peach_turnip_logic56_reflected(ip: &mut Item) -> bool {
    it_80273030(ip);
    false
}

/// `itLinkbomb_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_linkbomb_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itPeachturnip_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_peachturnip_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMball_Motion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_mball_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMball_Motion6_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_mball_motion6_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMetalb_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_metalb_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itMetalb_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_metalb_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itNokonoko_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nokonoko_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itNokonoko_UnkMotion10_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nokonoko_unk_motion10_phys(ip: &mut Item) {
    fall(ip);
}

/// `itPippi_UnkMotion1_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_pippi_unk_motion1_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itTogepy_UnkMotion1_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_togepy_unk_motion1_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itRabbitc_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rabbitc_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itRabbitc_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_rabbitc_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itCapsule_UnkMotion4_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_capsule_unk_motion4_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFflowerflame_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_fflowerflame_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itFreeze_UnkMotion3_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_freeze_unk_motion3_phys(ip: &mut Item) {
    fall(ip);
}

/// `itSscope_UnkMotion1_Phys`: fall (`it_80272860` with the item's fall speeds).
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_sscope_unk_motion1_phys(ip: &mut Item) {
    fall(ip);
}

/// `itLizardon_UnkMotion1_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_lizardon_unk_motion1_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itMarumine_UnkMotion5_Phys`: fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_marumine_unk_motion5_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itLinkBomb_Logic16_HitShield`: `itColl_BounceOffVictim`.
pub fn it_link_bomb_logic16_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_coll_bounce_off_victim(ip, common);
    false
}

// ---- hand-ported bodies -----------------------------------------------------------------

/// `itStarRod_Logic22_DmgDealt`: bounce off the victim in motion states 3 and 4.
pub fn it_star_rod_logic22_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    if ip.msid == 3 || ip.msid == 4 {
        it_coll_bounce_off_victim(ip, common);
    }
    false
}

/// `itStarRod_Logic22_Clanked`: same body as `itStarRod_Logic22_DmgDealt`.
pub fn it_star_rod_logic22_clanked(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_star_rod_logic22_dmg_dealt(ip, common)
}

/// `itStarRod_Logic22_HitShield`: same body as `itStarRod_Logic22_DmgDealt`.
pub fn it_star_rod_logic22_hit_shield(ip: &mut Item, common: &ItemCommonData) -> bool {
    it_star_rod_logic22_dmg_dealt(ip, common)
}

/// `itBat_Logic11_DmgDealt`: bounce off the victim in motion state 3.
pub fn it_bat_logic11_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    if ip.msid == 3 {
        it_coll_bounce_off_victim(ip, common);
    }
    false
}

/// `itHarisen_Logic24_DmgDealt`: bounce off the victim in motion states 7 and 8.
pub fn it_harisen_logic24_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    if ip.msid == 7 || ip.msid == 8 {
        it_coll_bounce_off_victim(ip, common);
    }
    false
}

/// `itLipstick_Logic23_DmgDealt`: bounce off the victim in motion states 3 and 4.
pub fn it_lipstick_logic23_dmg_dealt(ip: &mut Item, common: &ItemCommonData) -> bool {
    if ip.msid == 3 || ip.msid == 4 {
        it_coll_bounce_off_victim(ip, common);
    }
    false
}

/// `itKinoko_UnkMotion0_Phys`: on the ground, walk at `attrs.x0` in the facing direction.
/// # Safety
/// `ip.xC4_article_data->x4_specialAttributes` must be a valid `KinokoAttrs`.
pub unsafe fn it_kinoko_unk_motion0_phys(ip: &mut Item) {
    let attrs = sa::<KinokoAttrs>(ip);
    if ip.ground_or_air != GA_Air {
        ip.xDD4_itemVar.kinoko.x0 = attrs.x0 * ip.facing_dir;
        ip.x40_vel.x = ip.xDD4_itemVar.kinoko.x0 * ip.xDD4_itemVar.kinoko.x4.y;
    }
}

/// `itDkinoko_UnkMotion0_Phys`: same body as `itKinoko_UnkMotion0_Phys`.
/// # Safety
/// As `it_kinoko_unk_motion0_phys`.
pub unsafe fn it_dkinoko_unk_motion0_phys(ip: &mut Item) {
    it_kinoko_unk_motion0_phys(ip)
}

/// `itKinoko_UnkMotion1_Phys`: fall, then decelerate `vel.x` by `attrs.x4` to a stop.
/// # Safety
/// `ip.xCC_item_attr` and the special attributes (`KinokoAttrs`) must be valid.
pub unsafe fn it_kinoko_unk_motion1_phys(ip: &mut Item) {
    let sa = sa::<KinokoAttrs>(ip);
    fall(ip);
    let vel = if ip.x40_vel.x < 0.0 { -ip.x40_vel.x } else { ip.x40_vel.x };
    if vel < sa.x4 {
        ip.x40_vel.x = 0.0;
        return;
    } else if ip.x40_vel.x > 0.0 {
        ip.x40_vel.x -= sa.x4;
    }
    if ip.x40_vel.x < 0.0 {
        ip.x40_vel.x += sa.x4;
    }
}

/// `itDkinoko_UnkMotion1_Phys`: same body as `itKinoko_UnkMotion1_Phys`.
/// # Safety
/// As `it_kinoko_unk_motion1_phys`.
pub unsafe fn it_dkinoko_unk_motion1_phys(ip: &mut Item) {
    it_kinoko_unk_motion1_phys(ip)
}

/// `itDosei_UnkMotion6_Anim`: `dosei.xDE4 = pos`.
pub fn it_dosei_unk_motion6_anim(ip: &mut Item) -> bool {
    ip.xDD4_itemVar.dosei.xDE4 = ip.pos;
    false
}

/// `itDosei_UnkMotion8_Anim`: same body as `itDosei_UnkMotion6_Anim`.
pub fn it_dosei_unk_motion8_anim(ip: &mut Item) -> bool {
    it_dosei_unk_motion6_anim(ip)
}

/// Life timer `-= 1`, true when `<= 0` (`it_802CAE60`, `it_802CF120`, `it_802D5884`, ... below).
pub(crate) fn lifetimer_tick(ip: &mut Item) -> bool {
    ip.xD44_lifeTimer -= 1.0;
    ip.xD44_lifeTimer <= 0.0
}

/// `itMaril_UnkMotion2_Anim_inline` (static): as `it_802CAE60`.
pub fn it_maril_unk_motion2_anim_inline(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itLuigifireball_UnkMotion0_Anim`: as `it_802CAE60`.
pub fn it_luigifireball_unk_motion0_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itMariofireball_UnkMotion0_Anim`: as `it_802CAE60`.
pub fn it_mariofireball_unk_motion0_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itNesspkfire_UnkMotion0_Anim`: as `it_802CAE60`.
pub fn it_nesspkfire_unk_motion0_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itPikachuthunder_UnkMotion1_Anim`: as `it_802CAE60`.
pub fn it_pikachuthunder_unk_motion1_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `it_2ADA_UnkMotion0_Anim`: as `it_802CAE60`.
pub fn it_2ada_unk_motion0_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `it_2F28_UnkMotion0_Anim`: as `it_802CAE60`.
pub fn it_2f28_unk_motion0_anim(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itFushigibana_UnkMotion1_Anim_inline2` (static): as `it_802CAE60`.
pub fn it_fushigibana_unk_motion1_anim_inline2(ip: &mut Item) -> bool {
    lifetimer_tick(ip)
}

/// `itZeldaDinFire_GetOwner`: the owner.
pub fn it_zelda_din_fire_get_owner(ip: &Item) -> *mut HSD_GObj {
    ip.owner
}

/// `itRshell_UnkMotion3_Anim`: count `rshell.xDD8` down; at 0, `it_80275444` unless
/// `xDCD_flag.b5`.
pub fn it_rshell_unk_motion3_anim(ip: &mut Item) -> bool {
    unsafe {
        if ip.xDD4_itemVar.rshell.xDD8 <= 0.0 {
            if ip.xDCD_flag.x0.b5() == 0 {
                it_80275444(ip);
            }
        } else {
            ip.xDD4_itemVar.rshell.xDD8 -= 1.0;
        }
    }
    false
}

/// `itRshell_UnkMotion4_Anim`: same body as `itRshell_UnkMotion3_Anim`.
pub fn it_rshell_unk_motion4_anim(ip: &mut Item) -> bool {
    it_rshell_unk_motion3_anim(ip)
}

/// `itSeakneedlethrown_UnkMotion1_Phys`: `vel.x = xDD8`, `vel.y += xDE0` clamped below at `xDDC`.
pub fn it_seakneedlethrown_unk_motion1_phys(ip: &mut Item) {
    unsafe {
        let v = &ip.xDD4_itemVar.seakneedlethrown;
        let (dd8, de0, ddc) = (v.xDD8, v.xDE0, v.xDDC);
        ip.x40_vel.x = dd8;
        ip.x40_vel.y += de0;
        if ip.x40_vel.y < ddc {
            ip.x40_vel.y = ddc;
        }
    }
}

/// `itSeakneedlethrown_UnkMotion4_Phys`: same body as `itSeakneedlethrown_UnkMotion1_Phys`.
pub fn it_seakneedlethrown_unk_motion4_phys(ip: &mut Item) {
    it_seakneedlethrown_unk_motion1_phys(ip)
}

/// `itOctarockstone_UnkMotion0_Anim`: fall; never ends the state.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_octarockstone_unk_motion0_anim(ip: &mut Item) -> bool {
    fall(ip);
    false
}

/// `itKabigon_UnkMotion0_Phys`: fall while `xDB0_itcmd_var1 == 0` and airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kabigon_unk_motion0_phys(ip: &mut Item) {
    if ip.xDB0_itcmd_var1 == 0 && ip.ground_or_air == 1 {
        fall(ip);
    }
}

/// `itNesspkfirepillar_UnkMotion0_Phys`. The C condition is an assignment
/// (`ground_or_air = GA_Air`), so it always sets the item airborne and falls.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nesspkfirepillar_unk_motion0_phys(ip: &mut Item) {
    ip.ground_or_air = GA_Air;
    fall(ip);
}

/// `itNokonoko_UnkMotion7_Phys`: `vel.y -= fall_speed` while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nokonoko_unk_motion7_phys(ip: &mut Item) {
    if ip.ground_or_air == 1 {
        ip.x40_vel.y -= attr(ip).x10_fall_speed;
    }
}

/// `itLikelike_UnkMotion3_Phys`: `vel.y += fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_likelike_unk_motion3_phys(ip: &mut Item) {
    ip.x40_vel.y += attr(ip).x10_fall_speed;
}

/// `itMewtwoDisable_Logic67_Reflected`: `it_80273030`.
pub fn it_mewtwo_disable_logic67_reflected(ip: &mut Item) -> bool {
    it_80273030(ip)
}

/// `itDrMarioPill_Reflected`: `it_8026B3A8`, then `it_80273030`.
pub fn it_dr_mario_pill_reflected(ip: &mut Item) -> bool {
    it_8026b3a8(ip);
    it_80273030(ip)
}

/// `itMball_Reflected`: same body as `itDrMarioPill_Reflected`.
pub fn it_mball_reflected(ip: &mut Item) -> bool {
    it_8026b3a8(ip);
    it_80273030(ip)
}

/// `itGShell_Logic14_Reflected`: bounce back; `gshell.xDD4 = xD48_halfLifeTimer`.
pub fn it_gshell_logic14_reflected(ip: &mut Item) -> bool {
    it_80273030(ip);
    ip.xDD4_itemVar.gshell.xDD4 = ip.xD48_halfLifeTimer;
    false
}

/// `itRShell_Logic15_Reflected`: bounce back; `rshell.xDD4 = xD48_halfLifeTimer`.
pub fn it_rshell_logic15_reflected(ip: &mut Item) -> bool {
    it_80273030(ip);
    ip.xDD4_itemVar.rshell.xDD4 = ip.xD48_halfLifeTimer;
    false
}

/// `itPeachToadSpore_Logic68_Reflected`: bounce back keeping the life timer.
pub fn it_peach_toad_spore_logic68_reflected(ip: &mut Item) -> bool {
    let life = ip.xD44_lifeTimer;
    it_80273030(ip);
    ip.xD44_lifeTimer = life;
    false
}

/// `itChicoritaleaf_UnkMotion0_Anim`: true once the life timer is out, else count it down.
pub fn it_chicoritaleaf_unk_motion0_anim(ip: &mut Item) -> bool {
    if ip.xD44_lifeTimer <= 0.0 {
        return true;
    }
    ip.xD44_lifeTimer -= 1.0;
    false
}

/// `itDrMarioPill_UnkMotion0_Anim` (static): `it_8026B3A8`, then count the life timer down.
pub fn it_dr_mario_pill_unk_motion0_anim(ip: &mut Item) -> bool {
    it_8026b3a8(ip);
    lifetimer_tick(ip)
}

/// `itPeachtoadspore_UnkMotion0_Anim`: done when `xDAC_itcmd_var0` is set or the life timer
/// runs out (then clamped to 0).
pub fn it_peachtoadspore_unk_motion0_anim(ip: &mut Item) -> bool {
    if ip.xDAC_itcmd_var0 != 0 {
        return true;
    }
    ip.xD44_lifeTimer -= 1.0;
    if ip.xD44_lifeTimer <= 0.0 {
        ip.xD44_lifeTimer = 0.0;
        return true;
    }
    false
}

/// `itSonans_UnkMotion1_Anim`: count the life timer down; true only when it is exactly 0.
pub fn it_sonans_unk_motion1_anim(ip: &mut Item) -> bool {
    let lifetime = ip.xD44_lifeTimer - 1.0;
    ip.xD44_lifeTimer = lifetime;
    lifetime == 0.0
}

/// `itBox_UnkMotion7_Anim`: count `box.despawn_timer` down; true at `<= 0`.
pub fn it_box_unk_motion7_anim(ip: &mut Item) -> bool {
    unsafe {
        let b = &mut ip.xDD4_itemVar.r#box;
        b.despawn_timer = b.despawn_timer.wrapping_sub(1);
        b.despawn_timer <= 0
    }
}

/// `itEgg_UnkMotion6_Anim`: count `egg.rand_max` down; true at `<= 0`.
pub fn it_egg_unk_motion6_anim(ip: &mut Item) -> bool {
    unsafe {
        let e = &mut ip.xDD4_itemVar.egg;
        e.rand_max = e.rand_max.wrapping_sub(1);
        e.rand_max <= 0
    }
}

/// `itFoxlaser_UnkMotion1_Phys`: `foxlaser.pos = pos`.
pub fn it_foxlaser_unk_motion1_phys(ip: &mut Item) {
    ip.xDD4_itemVar.foxlaser.pos = ip.pos;
}

/// `itLgunray_UnkMotion0_Phys`: `lgunray.pos = pos`.
pub fn it_lgunray_unk_motion0_phys(ip: &mut Item) {
    ip.xDD4_itemVar.lgunray.pos = ip.pos;
}

/// `itCerebi_UnkMotion2_Phys`: `vel.y += ((f32*) special attributes)[3]`.
/// # Safety
/// The special attributes must hold at least 4 floats.
pub unsafe fn it_cerebi_unk_motion2_phys(ip: &mut Item) {
    let a = (*ip.xC4_article_data).x4_specialAttributes as *const f32;
    ip.x40_vel.y += *a.add(3);
}

/// `itFire_UnkMotion2_Phys`: `vel.y += attrs.xC`.
/// # Safety
/// The special attributes must be a valid `itFireAttributes`.
pub unsafe fn it_fire_unk_motion2_phys(ip: &mut Item) {
    ip.x40_vel.y += sa::<itFireAttributes>(ip).xC;
}

/// `itMew_UnkMotion2_Phys`: `vel.y += attrs.xC`.
/// # Safety
/// The special attributes must be a valid `MewVars`.
pub unsafe fn it_mew_unk_motion2_phys(ip: &mut Item) {
    ip.x40_vel.y += sa::<MewVars>(ip).xC;
}

/// `itClimbersIce_sub_x4` (static inline): `xD44_lifeTimer -= attrs.x4`.
/// # Safety
/// The special attributes must be a valid `itClimbersIceAttributes`.
pub unsafe fn it_climbers_ice_sub_x4(ip: &mut Item) {
    ip.xD44_lifeTimer -= sa::<itClimbersIceAttributes>(ip).x4;
}

/// `itCoin_UnkMotion0_Phys`: fall with the item's or the coin's own speeds.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_coin_unk_motion0_phys(ip: &mut Item) {
    if ip.xDD4_itemVar.coin.x4 == 0 {
        fall(ip);
        return;
    }
    let (a, b) = (ip.xDD4_itemVar.coin.x8, ip.xDD4_itemVar.coin.xC);
    it_80272860(ip, a, b);
}

/// `itCoin_UnkMotion3_Phys`: `vel.x *= attrs.x34`, then fall with the coin's speeds.
/// # Safety
/// The special attributes must be a valid `itCoinAttributes`.
pub unsafe fn it_coin_unk_motion3_phys(ip: &mut Item) {
    ip.x40_vel.x *= sa::<itCoinAttributes>(ip).x34;
    let (a, b) = (ip.xDD4_itemVar.coin.x8, ip.xDD4_itemVar.coin.xC);
    it_80272860(ip, a, b);
}

/// `itDosei_UnkMotion1_Phys`: `vel.x = facing_dir * |attrs.unk8 * x5D0_animFrameSpeed|`.
/// # Safety
/// The special attributes must be a valid `itDoseiAttributes`.
pub unsafe fn it_dosei_unk_motion1_phys(ip: &mut Item) {
    let p = sa::<itDoseiAttributes>(ip).unk8 * ip.x5D0_animFrameSpeed;
    ip.x40_vel.x = ip.facing_dir * if p < 0.0 { -p } else { p };
}

/// `itDrMarioPill_Motion2_Phys` (static): `it_802756D0`, `it_8026B3A8`.
pub fn it_dr_mario_pill_motion2_phys(ip: &mut Item) {
    it_802756d0(ip);
    it_8026b3a8(ip);
}

/// `itDrMariopill_Motion2_Coll` (static): `it_802756D0`, `it_8026B3A8`.
pub fn it_dr_mariopill_motion2_coll(ip: &mut Item) -> bool {
    it_802756d0(ip);
    it_8026b3a8(ip);
    false
}

/// `itDrMarioPill_Motion6_Coll` (static): `it_8026B3A8`.
pub fn it_dr_mario_pill_motion6_coll(ip: &mut Item) -> bool {
    it_8026b3a8(ip);
    false
}

/// `itFoxIllusion_Logic14_DmgDealt`: `xCA8 = 0`.
pub fn it_fox_illusion_logic14_dmg_dealt(ip: &mut Item) -> bool {
    ip.xCA8 = 0;
    false
}

/// `itLipstick_UnkMotion2_Anim`: `xDAC_itcmd_var0 = 1`.
pub fn it_lipstick_unk_motion2_anim(ip: &mut Item) -> bool {
    ip.xDAC_itcmd_var0 = 1;
    false
}

/// `itSonans_Logic9_DmgDealt`: `sonans.x60 *= -1`.
pub fn it_sonans_logic9_dmg_dealt(ip: &mut Item) -> bool {
    unsafe { ip.xDD4_itemVar.sonans.x60 *= -1.0 };
    false
}

/// `itHitodeman_UnkMotion1_Phys`: while `xDB0_itcmd_var1`, accelerate along the facing
/// direction. 1 fused site: `vel.x = -fmadds(-facing_dir, attrs.x4C, -vel.x)`.
/// # Safety
/// The special attributes must be a valid `itHitodemanAttributes`.
pub unsafe fn it_hitodeman_unk_motion1_phys(ip: &mut Item) {
    let attrs = sa::<itHitodemanAttributes>(ip);
    if ip.xDB0_itcmd_var1 != 0 {
        ip.x40_vel.x = -fmadds(-ip.facing_dir, attrs.x4C, -ip.x40_vel.x);
    }
}

/// `itLeadead_UnkMotion3_Anim_inline` (static inline): `leadead.x48 = attrs.x18 * 3`.
/// # Safety
/// The special attributes must be a valid `itLeadeadAttributes`.
pub unsafe fn it_leadead_unk_motion3_anim_inline(ip: &mut Item) {
    ip.xDD4_itemVar.leadead.x48 = sa::<itLeadeadAttributes>(ip).x18 as i32 * 3;
}

/// `itLeadead_UnkMotion4_Phys`: walk at `attrs.x14` after frame 0x24, stop after 0x71.
/// # Safety
/// The special attributes must be a valid `itLeadeadAttributes`.
pub unsafe fn it_leadead_unk_motion4_phys(ip: &mut Item) {
    let attr = sa::<itLeadeadAttributes>(ip);
    if ip.xDD4_itemVar.leadead.x48 > 0x24 {
        ip.x40_vel.x = attr.x14 * ip.facing_dir;
    }
    if ip.xDD4_itemVar.leadead.x48 > 0x71 {
        ip.x40_vel.x = 0.0;
    }
    ip.xDD4_itemVar.leadead.x48 = ip.xDD4_itemVar.leadead.x48.wrapping_add(1);
}

/// `itPeachtoadspore_UnkMotion0_Phys`: scale `vel.x`, `vel.y` by `attrs.x8_speed_decay_rate`.
/// # Safety
/// The special attributes must be a valid `itPeachToadSporeAttributes`.
pub unsafe fn it_peachtoadspore_unk_motion0_phys(ip: &mut Item) {
    let k = sa::<itPeachToadSporeAttributes>(ip).x8_speed_decay_rate;
    ip.x40_vel.x *= k;
    ip.x40_vel.y *= k;
}

/// `itParasol_UnkMotion1_Phys`: fall, then `decelerateItemX(attrs[0])`.
/// # Safety
/// `ip.xCC_item_attr` and the special attributes (at least 2 floats) must be valid.
pub unsafe fn it_parasol_unk_motion1_phys(ip: &mut Item) {
    let a = (*ip.xC4_article_data).x4_specialAttributes as *const f32;
    fall(ip);
    decelerate_item_x(ip, *a);
}

/// `itParasol_UnkMotion2_Phys`: fall, then `decelerateItemX(attrs[1])`.
/// # Safety
/// As `it_parasol_unk_motion1_phys`.
pub unsafe fn it_parasol_unk_motion2_phys(ip: &mut Item) {
    let a = (*ip.xC4_article_data).x4_specialAttributes as *const f32;
    fall(ip);
    decelerate_item_x(ip, *a.add(1));
}

/// `itSeakneedlethrown_UnkMotion0_Anim`: `xDE4 = pos`, then `it_80273130`.
pub fn it_seakneedlethrown_unk_motion0_anim(ip: &mut Item) -> bool {
    ip.xDD4_itemVar.seakneedlethrown.xDE4 = ip.pos;
    it_80273130(ip)
}

/// `itPikachutjoltground_UnkMotion0_Anim`: `xDE8 = pos`, then `it_80273130`.
pub fn it_pikachutjoltground_unk_motion0_anim(ip: &mut Item) -> bool {
    ip.xDD4_itemVar.pikachujoltground.xDE8 = ip.pos;
    it_80273130(ip)
}

/// `itArwinglaser_UnkMotion3_Anim` (static): `xE24 = xE18`, `xE30 = 1`, `it_80273130` (result
/// dropped).
pub fn it_arwinglaser_unk_motion3_anim(ip: &mut Item) {
    unsafe {
        let a = &mut ip.xDD4_itemVar.arwinglaser;
        a.xE24 = a.xE18;
        a.xE30 = 1;
    }
    it_80273130(ip);
}

/// `itNessPKFlashExplode_UnkMotion0_Phys`: `vel.z = vel.y = vel.z = 0` (x untouched).
pub fn it_ness_pk_flash_explode_unk_motion0_phys(ip: &mut Item) {
    ip.x40_vel.z = 0.0;
    ip.x40_vel.y = 0.0;
}

/// `itLipstickspore_UnkMotion0_Phys`: decelerate `vel.x` by `xDD8` to a stop.
pub fn it_lipstickspore_unk_motion0_phys(ip: &mut Item) {
    let d = unsafe { ip.xDD4_itemVar.lipstickspore.xDD8 };
    if ip.x40_vel.x == 0.0 {
        return;
    }
    if d == 0.0 {
        return;
    }
    if ip.x40_vel.x > 0.0 {
        ip.x40_vel.x -= d;
    } else {
        ip.x40_vel.x += d;
    }
    let mut abs_vel = ip.x40_vel.x;
    if abs_vel < 0.0 {
        abs_vel = -abs_vel;
    }
    if abs_vel < d {
        ip.x40_vel.x = 0.0;
    }
}

/// `itOctarock_UnkMotion1_Phys`: leave the ground when moving up.
pub fn it_octarock_unk_motion1_phys(ip: &mut Item) {
    if ip.ground_or_air == GA_Ground && ip.x40_vel.y > 0.0 {
        it_802762bc(ip);
    }
}

/// `itOctarock_UnkMotion4_Phys`: leave the ground once (`octarock.x2C`) when moving up.
pub fn it_octarock_unk_motion4_phys(ip: &mut Item) {
    unsafe {
        if ip.xDD4_itemVar.octarock.x2C == 0 && ip.x40_vel.y > 0.0 {
            it_802762bc(ip);
            ip.xDD4_itemVar.octarock.x2C = 1;
        }
    }
}

/// `itNokonoko_UnkMotion8_Phys`: `vel.y -= fall_speed`; count `nokonoko.x28` down, at 0
/// `it_802756E0`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_nokonoko_unk_motion8_phys(ip: &mut Item) {
    ip.x40_vel.y -= attr(ip).x10_fall_speed;
    if ip.xDD4_itemVar.nokonoko.x28 == 0 {
        it_802756e0(ip);
        return;
    }
    ip.xDD4_itemVar.nokonoko.x28 = ip.xDD4_itemVar.nokonoko.x28.wrapping_sub(1);
}

/// `itPatapata_UnkMotion4_Coll`: count `patapata.x20` down; at 0 set `xDCC_flag.b3`, true.
pub fn it_patapata_unk_motion4_coll(ip: &mut Item) -> bool {
    unsafe {
        if ip.xDD4_itemVar.patapata.x20 == 0 {
            ip.xDCC_flag.set_b3(1);
            return true;
        }
        ip.xDD4_itemVar.patapata.x20 = ip.xDD4_itemVar.patapata.x20.wrapping_sub(1);
    }
    false
}

/// `itGamewatchchef_UnkMotion0_Phys`: fall with the speeds of attribute entry
/// `gamewatchchef.x4`.
/// # Safety
/// The special attributes must be a valid `itGamewatchchefAttributes` with an entry at that
/// index (the C array is declared `[1]` and indexed past it).
pub unsafe fn it_gamewatchchef_unk_motion0_phys(ip: &mut Item) {
    let attrs = sa::<itGamewatchchefAttributes>(ip);
    let index = ip.xDD4_itemVar.gamewatchchef.x4;
    let e = &*attrs.entries.as_ptr().offset(index as isize);
    it_80272860(ip, e.x8, e.xC);
}

/// `itHammerHead_Logic40_Spawned`: `it_80275158(attrs.x4)` (life timer).
/// # Safety
/// The special attributes must be a valid `itHammerheadAttributes`.
pub unsafe fn it_hammer_head_logic40_spawned(ip: &mut Item, common: &ItemCommonData) {
    let l = sa::<itHammerheadAttributes>(ip).x4;
    it_80275158(ip, l, common);
}

/// `itHassam_UnkMotion0_Phys`: `it_8027A344`, then fall while airborne.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_hassam_unk_motion0_phys(ip: &mut Item) {
    it_8027a344(ip);
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itHassam_UnkMotion2_Phys`: `it_8027A344`, then `pos += hassam.x5C`.
pub fn it_hassam_unk_motion2_phys(ip: &mut Item) {
    it_8027a344(ip);
    unsafe {
        let d = ip.xDD4_itemVar.hassam.x5C;
        ip.pos.x += d.x;
        ip.pos.y += d.y;
        ip.pos.z += d.z;
    }
}

/// `itMaril_UnkMotion1_Phys`: `it_8027A344`, then `vel.x = maril.x64`.
pub fn it_maril_unk_motion1_phys(ip: &mut Item) {
    it_8027a344(ip);
    ip.x40_vel.x = unsafe { ip.xDD4_itemVar.maril.x64 };
}

/// `itLinkarrow_UnkMotion1_Phys`: `linkarrow.x18 = pos`; `vel.y -= |attrs.x1C|`.
/// # Safety
/// The special attributes must be a valid `itLinkArrowAttributes`.
pub unsafe fn it_linkarrow_unk_motion1_phys(ip: &mut Item) {
    let g = sa::<itLinkArrowAttributes>(ip).x1C;
    ip.xDD4_itemVar.linkarrow.x18 = ip.pos;
    ip.x40_vel.y -= if g < 0.0 { -g } else { g };
}

/// `itFreeze_UnkMotion4_Phys`: take a pending `freeze.x10` velocity, then decelerate `vel.x`
/// by `|freeze.x14|` to a stop.
pub fn it_freeze_unk_motion4_phys(ip: &mut Item) {
    unsafe {
        let x10 = ip.xDD4_itemVar.freeze.x10;
        if x10 != 0.0 {
            ip.x40_vel.x = x10;
            ip.xDD4_itemVar.freeze.x10 = 0.0;
        }
        let mut decel = ip.xDD4_itemVar.freeze.x14;
        let abs_decel = if decel < 0.0 { -decel } else { decel };
        let vel = ip.x40_vel.x;
        let abs_vel = if vel < 0.0 { -vel } else { vel };
        if abs_vel < abs_decel {
            ip.x40_vel.x = 0.0;
            return;
        }
        if vel > 0.0 {
            if ip.xDD4_itemVar.freeze.x14 < 0.0 {
                decel = -decel;
            }
            ip.x40_vel.x -= decel;
        } else {
            if decel < 0.0 {
                decel = -decel;
            }
            ip.x40_vel.x += decel;
        }
    }
}

/// `itFreezer_UnkMotion2_Phys`: on `xDAC_itcmd_var0` launch at `attrs.x4` and start
/// accelerating (`xDB0_itcmd_var1`) by `attrs.x8`.
/// # Safety
/// The special attributes must be a valid `itFreezerAttributes`.
pub unsafe fn it_freezer_unk_motion2_phys(ip: &mut Item) {
    if ip.xDAC_itcmd_var0 != 0 {
        ip.x40_vel.y = sa::<itFreezerAttributes>(ip).x4;
        ip.xDAC_itcmd_var0 = 0;
        ip.xDB0_itcmd_var1 = 1;
    }
    if ip.xDB0_itcmd_var1 != 0 {
        ip.x40_vel.y += sa::<itFreezerAttributes>(ip).x8;
    }
}

/// `itHouou_UnkMotion4_Phys`: `it_8027A344`; accelerate upward by `attrs.x1C`; stop 10 below
/// the start height.
/// # Safety
/// The special attributes must be a valid `itHououAttr`.
pub unsafe fn it_houou_unk_motion4_phys(ip: &mut Item) {
    let attr = sa::<itHououAttr>(ip);
    it_8027a344(ip);
    ip.xDD4_itemVar.houou.vel_accum += attr.x1C;
    ip.x40_vel.y += ip.xDD4_itemVar.houou.vel_accum;
    if ip.pos.y < ip.xDD4_itemVar.houou.start_pos.y - 10.0 {
        ip.x40_vel.y = 0.0;
    }
}

/// `itKireihana_UnkMotion4_Phys`: `it_8027A344`; `kireihana.fall_speed += fall_speed` capped at
/// `fall_speed_max`; `vel.y = -fall_speed`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kireihana_unk_motion4_phys(ip: &mut Item) {
    it_8027a344(ip);
    let a = attr(ip);
    ip.xDD4_itemVar.kireihana.fall_speed += a.x10_fall_speed;
    if ip.xDD4_itemVar.kireihana.fall_speed > a.x14_fall_speed_max {
        ip.xDD4_itemVar.kireihana.fall_speed = a.x14_fall_speed_max;
    }
    ip.x40_vel.y = -ip.xDD4_itemVar.kireihana.fall_speed;
}

/// `itKamex_UnkMotion1_Phys`: brake against the facing direction by `pokemon.x6C`; fall while
/// airborne. 1 fused site: `vel.x = fmadds(pokemon.x6C, facing_dir, vel.x)`.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_kamex_unk_motion1_phys(ip: &mut Item) {
    if ip.x40_vel.x != 0.0 {
        if (ip.x40_vel.x > 0.0 && ip.facing_dir <= 0.0)
            || (ip.x40_vel.x <= 0.0 && ip.facing_dir > 0.0)
        {
            ip.x40_vel.x = fmadds(ip.xDD4_itemVar.pokemon.x6C, ip.facing_dir, ip.x40_vel.x);
        }
    }
    if ip.ground_or_air == GA_Air {
        fall(ip);
    }
}

/// `itLinkbomb_UnkMotion4_Phys`: while `linkbomb.x0.b1`, `vel.x += attrs.x2C * linkbomb.x4`
/// (1 fused site, `fmadds`) and stop below `attrs.x30`.
/// # Safety
/// The special attributes must be a valid `itLinkBombAttributes`.
pub unsafe fn it_linkbomb_unk_motion4_phys(ip: &mut Item) {
    let sa = sa::<itLinkBombAttributes>(ip);
    let zero = 0.0f32;
    if ip.xDD4_itemVar.linkbomb.x0.x0.b1() != 0 {
        let temp_f2 = ip.x40_vel.x;
        if temp_f2 != zero {
            ip.x40_vel.x = fmadds(sa.x2C, ip.xDD4_itemVar.linkbomb.x4, temp_f2);
        }
        let v = ip.x40_vel.x;
        if (if v < 0.0 { -v } else { v }) < sa.x30 {
            ip.x40_vel.x = zero;
        }
    }
}

/// `itSamusmissile_UnkMotion1_Phys`: while `xDB0_itcmd_var1`, accelerate by `attrs.x30` along
/// the facing direction (1 fused site, `fmadds`), capped at `attrs.x34`.
/// # Safety
/// The special attributes must be a valid `itSamusMissileAttributes`.
pub unsafe fn it_samusmissile_unk_motion1_phys(ip: &mut Item) {
    let attrs = sa::<itSamusMissileAttributes>(ip);
    if ip.xDB0_itcmd_var1 != 0 {
        ip.x40_vel.x = fmadds(attrs.x30, ip.facing_dir, ip.x40_vel.x);
        let v = ip.x40_vel.x;
        let xvel = if v < 0.0 { -v } else { v };
        let max_speed = attrs.x34;
        if xvel > max_speed {
            ip.x40_vel.x = if ip.facing_dir == 1.0 { max_speed } else { -max_speed };
        }
    }
}

/// `itGShell_Attrs`: file-local in itgshell.c, so not in the generated set (ppc32: all
/// 4-byte fields, size 0x40, GC offsets in the `gc_layout!` below).
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct itGShell_Attrs {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xC: f32,
    pub x10: f32,
    pub x14: f32,
    pub pad18: [u8; 4],
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
    pub x34: GcVec,
}

impl itGShell_Attrs {
    /// All-zero is a valid value (plain floats).
    pub fn zeroed() -> Self {
        // SAFETY: every field is a float or byte array.
        unsafe { core::mem::zeroed() }
    }
}

melee_core::gc_layout!(itGShell_Attrs, "itGShell_Attrs", size 0x40, align 4, [
    x0 @ 0x0: 4 "float", x4 @ 0x4: 4 "float", x8 @ 0x8: 4 "float", xC @ 0xC: 4 "float",
    x10 @ 0x10: 4 "float", x14 @ 0x14: 4 "float", pad18 @ 0x18: 4 "char[4]",
    x1C @ 0x1C: 4 "float", x20 @ 0x20: 4 "float", x24 @ 0x24: 4 "float", x28 @ 0x28: 4 "float",
    x2C @ 0x2C: 4 "float", x30 @ 0x30: 4 "float", x34 @ 0x34: 12 "Vec",
]);

/// `itGshell_UnkMotion3_Anim`: while `gshell.xDEC_b1`, count `xDE0` down; at 0 clear the bit,
/// reload `attrs.x28`, re-enable hitbox 0, and `it_802756E0` if `xDEC_b2`.
/// # Safety
/// The special attributes must be a valid `itGShell_Attrs`.
pub unsafe fn it_gshell_unk_motion3_anim(ip: &mut Item) -> bool {
    it_8028b988(ip);
    false
}

/// `itFlipper_AddSpinImpulse`: add `vel + attrs.x1C_baseSpinOnHit` to the spin velocity, signed
/// by which quadrant `pos` hit, clamped to `+-attrs.x24_maxSpinVel`.
/// # Safety
/// The special attributes must be a valid `itFlipper_DatAttrs`.
pub unsafe fn it_flipper_add_spin_impulse(ip: &mut Item, pos: &GcVec, vel: f32) {
    let attrs = sa::<itFlipper_DatAttrs>(ip);
    let speed = vel + attrs.x1C_baseSpinOnHit;
    let f = &mut ip.xDD4_itemVar.flipper;
    if pos.x > ip.pos.x {
        if pos.y > ip.pos.y {
            f.xDE4_spinVel += speed;
        } else {
            f.xDE4_spinVel -= speed;
        }
    } else if pos.y > ip.pos.y {
        f.xDE4_spinVel -= speed;
    } else {
        f.xDE4_spinVel += speed;
    }
    if f.xDE4_spinVel > attrs.x24_maxSpinVel {
        f.xDE4_spinVel = attrs.x24_maxSpinVel;
    }
    if f.xDE4_spinVel < -attrs.x24_maxSpinVel {
        f.xDE4_spinVel = -attrs.x24_maxSpinVel;
    }
}

/// `itChicorita_Phys` (static inline): `it_8027A344`; airborne, apply `fall_speed` to
/// `chicorita.x64` (as `it_80272860` does to `vel.y`) and copy it to `vel.y`; grounded, zero it.
/// # Safety
/// `ip.xCC_item_attr` must be valid.
pub unsafe fn it_chicorita_phys(ip: &mut Item) {
    it_8027a344(ip);
    let a = attr(ip);
    if ip.ground_or_air == GA_Air {
        let fall_speed_dir = if a.x10_fall_speed < 0.0 { -1 } else { 1 };
        let mut c = ip.xDD4_itemVar.chicorita.x64;
        let vel_dir = if c < 0.0 { -1 } else { 1 };
        if vel_dir != fall_speed_dir {
            if c < 0.0 {
                c = -c;
            }
            if c < a.x14_fall_speed_max {
                ip.xDD4_itemVar.chicorita.x64 -= a.x10_fall_speed;
            }
        } else {
            ip.xDD4_itemVar.chicorita.x64 -= a.x10_fall_speed;
        }
        ip.x40_vel.y = ip.xDD4_itemVar.chicorita.x64;
        return;
    }
    ip.xDD4_itemVar.chicorita.x64 = 0.0;
}

pub use crate::tu::itsamusbomb::it_2725_logic50_reflected;
pub use crate::tu::itheart::it_80283bd4;
pub use crate::tu::itgshell::it_8028b988;
pub use crate::tu::itrshell::it_8028d390;
pub use crate::tu::itrshell::it_8028d4e4;
pub use crate::tu::itlinkboomerang::it_8029fdbc;
pub use crate::tu::itnesspkflash::it_802aa7e4;
pub use crate::tu::itnesspkflash::it_802aa7f0;
pub use crate::tu::itpeachparasol::it_802bdc38;
pub use crate::tu::itkamex::it_802ca618;
pub use crate::tu::itkamex::it_802cae60;
pub use crate::tu::itmatadogas::it_802cb844;
pub use crate::tu::itlizardon::it_802cc684;
pub use crate::tu::itsonans::it_802cd7d4;
pub use crate::tu::itunknown::it_802cf120;
pub use crate::tu::itlugia::it_802d24a0;
pub use crate::tu::ithitodeman::it_802d4ef4;
pub use crate::tu::ithitodeman::it_802d4ffc;
pub use crate::tu::itlucky::it_802d5884;
pub use crate::tu::itheiho::it_802d98ac;
pub use crate::tu::itoctarock::it_802e5944;
pub use crate::tu::it_2E5A::it_802e657c;
pub use crate::tu::itcoin::it_802f2020;
