//! Native port of src/melee/it/kinds/itlugia.c: the Lugia Pokemon item (rise / hover / dive state machine, the
//! aeroblast spawner `it_802D1BBC` with its target search `it_802D208C`) and the three aeroblast items.
//! Callees in other TUs go through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU,
//! shipping contraction) by tests/tu_itlugia.rs.
//!
//! Float math, as the shipping build contracts it: `my_sqrtf` / `my_sqrtf_accurate` are MSL `sqrtf` /
//! `sqrtf_accurate` (`melee_runtime::gekko`, which holds the verified fused Newton step);
//! `-((prod * -disc) - (x18 * x18))` is `fmaf(prod, -disc, -(x18 * x18))` (the left product fuses);
//! `target.x += x3C * cosf(a)` is `fmaf(x3C, cosf(a), target.x)`; the `xE50.y >= 1.0` tests and the
//! `xE50.y - 1.0` updates are in double. `cosf` / `sinf` are the MSL ports of `melee_runtime::gekko`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
#[allow(unused_imports)]
use crate::itkinds9::{sqrt_n};
#[allow(unused_imports)]
use crate::helpers::pika_scale;
#[allow(unused_imports)]
use crate::itkinds11::it_80272cc0;
#[allow(unused_imports)]
use crate::itkinds13::swap_velocity;
#[allow(unused_imports)]
use crate::itkinds5::it_802ac5d8;
#[allow(unused_imports)]
use crate::itkinds5::it_802b3368;
#[allow(unused_imports)]
use crate::itkinds8::get_min_life;
#[allow(unused_imports)]
use crate::itkinds9::my_sqrtf_accurate as my_sqrtf_accurate_itkinds14;
#[allow(unused_imports)]
use crate::itlib::it_80275594;
#[allow(unused_imports)]
use crate::itlib3::it_80272828;
#[allow(unused_imports)]
use crate::layout::Article;
#[allow(unused_imports)]
use crate::layout::CommandInfo;
#[allow(unused_imports)]
use crate::layout::HSD_JObj;
#[allow(unused_imports)]
use crate::layout::HSD_Joint;
#[allow(unused_imports)]
use crate::layout::ItemStateTable;
#[allow(unused_imports)]
use crate::layout::it_8026C47C_arg0_t;
#[allow(unused_imports)]
use crate::layout::itBombHeiAttributes;
#[allow(unused_imports)]
use crate::layout::itClimbersStringAttributes;
#[allow(unused_imports)]
use crate::layout::itLikelikeAttributes;
#[allow(unused_imports)]
use crate::layout::itNessPKFirepillarAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusBombAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::ItemKind;
#[allow(unused_imports)]
use crate::layout::It_Kind_Capsule;
#[allow(unused_imports)]
use crate::layout::It_Kind_RabbitC;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_core::vec::Vec3 as RVec;
#[allow(unused_imports)]
use melee_runtime::lbvector;
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::helpers::it_rshell_clamp_vel;
#[allow(unused_imports)]
use crate::helpers::itRShell_Attrs;
#[allow(unused_imports)]
use crate::item::decelerate_item_x;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_802756d0;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_802756e0;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
#[allow(unused_imports)]
use crate::layout::itClimbersIceAttributes;
#[allow(unused_imports)]
use crate::layout::itCoinAttributes;
#[allow(unused_imports)]
use crate::layout::itDoseiAttributes;
#[allow(unused_imports)]
use crate::layout::itFireAttributes;
#[allow(unused_imports)]
use crate::layout::itFlipper_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itFreezerAttributes;
#[allow(unused_imports)]
use crate::layout::itGamewatchchefAttributes;
#[allow(unused_imports)]
use crate::layout::itHammerheadAttributes;
#[allow(unused_imports)]
use crate::layout::itHitodemanAttributes;
#[allow(unused_imports)]
use crate::layout::itHououAttr;
#[allow(unused_imports)]
use crate::layout::itLeadeadAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkArrowAttributes;
#[allow(unused_imports)]
use crate::layout::itLinkBombAttributes;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itMatadogasAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachToadSporeAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusMissileAttributes;
#[allow(unused_imports)]
use crate::layout::itsonansAttributes;
#[allow(unused_imports)]
use crate::layout::HeartContainerAttr;
#[allow(unused_imports)]
use crate::layout::ItemAttr;
#[allow(unused_imports)]
use crate::layout::KinokoAttrs;
#[allow(unused_imports)]
use crate::layout::MewVars;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use crate::layout::GA_Ground;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast2;
#[allow(unused_imports)]
use crate::layout::It_Kind_Lugia_Aeroblast3;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas1;
#[allow(unused_imports)]
use crate::layout::It_Kind_Matadogas_Gas2;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::helpers::decrease_lifetimer;
#[allow(unused_imports)]
use crate::itkinds::it_802d5884;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_chicorita_phys;
#[allow(unused_imports)]
use crate::itkinds::it_harisen_logic24_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lipstick_logic23_dmg_dealt;
#[allow(unused_imports)]
use crate::itkinds::it_lizardon_unk_motion1_phys;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_anim_inline;
#[allow(unused_imports)]
use crate::itkinds::it_maril_unk_motion2_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_80275870;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
#[allow(unused_imports)]
use crate::itlib3::it_8027b730;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::gekko::{cosf, sinf};
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Rand() -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Rand() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn Stage_GetBlastZoneTopOffset() -> f32 => unsafe { melee_world::tu::stage::Stage_GetBlastZoneTopOffset() };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80273670(a0: *mut HSD_GObj, a1: i32, a2: f32) => unsafe { crate::tu::it_2725::it_80273670(a0, a1, a2) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027A160(a0: *mut HSD_JObj, a1: *mut Item) => unsafe { crate::tu::it_279C::it_8027A160(a0, a1) };
    fn it_8027A344(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027a344(&mut *((*a0).user_data as *mut Item)) };
    fn it_8027ADEC(a0: i32, a1: *mut HSD_GObj, a2: *mut HSD_JObj, a3: f32) => unsafe { crate::tu::it_279C::it_8027ADEC(a0, a1, a2, a3) };
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec => unsafe { melee_runtime::tu::lbvector::lbVector_Diff(a0, a1, a2) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::inl_it_core!();
crate::inl_it_hitlag_cb!();
crate::inl_it_enter_air_hitlag_desc!();
crate::inl_it_tick!();

#[inline(always)]
unsafe fn attrs_of(ip: *mut Item) -> *mut itLugiaAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLugiaAttributes
}

pub static mut it_803F7EE8: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802D1DB4), physics_updated: Some(it_802D1DD8), collided: Some(it_802D1E64) },
    ItemStateTable { anim_id: 1, animated: Some(itLugia_UnkMotion1_Anim), physics_updated: Some(itLugia_UnkMotion1_Phys), collided: Some(itLugia_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 2, animated: Some(itLugia_UnkMotion2_Anim), physics_updated: Some(itLugia_UnkMotion2_Phys), collided: Some(itLugia_UnkMotion2_Coll) },
    ItemStateTable { anim_id: 3, animated: Some(itLugia_UnkMotion3_Anim), physics_updated: Some(itLugia_UnkMotion3_Phys), collided: Some(itLugia_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 4, animated: Some(itLugia_UnkMotion4_Anim), physics_updated: Some(itLugia_UnkMotion4_Phys), collided: Some(itLugia_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 5, animated: Some(itLugia_UnkMotion5_Anim), physics_updated: Some(itLugia_UnkMotion5_Phys), collided: Some(itLugia_UnkMotion5_Coll) },
];

pub static mut it_803F7F48: [ItemStateTable; 1] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802D246C), physics_updated: Some(it_802D24A0), collided: Some(it_802D24FC) },
];

/// MSL `sqrtf` (three Newton steps; the `volatile` temporary only forces the store).
#[inline]
pub fn my_sqrtf(x: f32) -> f32 {
    melee_core::ct!("my_sqrtf");
    melee_runtime::gekko::sqrtf(x)
}

/// MSL `sqrtf_accurate` (four Newton steps).
#[inline]
pub fn my_sqrtf_accurate(x: f32) -> f32 {
    melee_core::ct!("my_sqrtf_accurate");
    melee_runtime::gekko::sqrtf_accurate(x)
}

pub unsafe fn it_2725_Logic17_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic17_Spawned");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.lugia.x64 = (*ip).pos;
    (*ip).facing_dir = 0.0;
    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDD4_itemVar.lugia.x60 = (*attrs).x14;
    (*ip).xDCC_flag.set_b3(0);
    it_802D1D40(gobj);
    ext::it_80279CDC(gobj, (*attrs).x0);
    (*ip).xDD4_itemVar.lugia.xE50.x = 0.0;
}

pub unsafe extern "C" fn it_802D14D0() { melee_core::ct!("it_802D14D0");}

pub unsafe fn itLugia_Logic17_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_Logic17_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `jobj == NULL ? NULL : jobj->child`
#[inline(always)]
unsafe fn child_of(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() { null_mut() } else { (*jobj).child }
}

pub unsafe extern "C" fn itLugia_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion1_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = child_of(jobj);

    ext::it_8027A160(child, ip);

    if ext::it_80272C6C(gobj) == 0 {
        it_802D1580(gobj);
    }

    0
}

pub unsafe extern "C" fn itLugia_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_UnkMotion1_Phys");
    ext::it_8027A344(gobj);
}

pub unsafe extern "C" fn itLugia_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion1_Coll");
    0
}

pub unsafe fn it_802D1580(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1580");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.lugia.xE50.x = -(*attrs).x4;
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
}

pub unsafe extern "C" fn itLugia_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion2_Anim");
    let item = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let bone = child_of(jobj);

    ext::it_8027A160(bone, item);
    0
}

pub unsafe extern "C" fn itLugia_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8027A344(gobj);
    if ext::it_80272C6C(gobj) != 0 {
        (*ip).xDD4_itemVar.lugia.xE50.x += (*attrs).x8;
    } else {
        (*ip).xDD4_itemVar.lugia.xE50.x += (*attrs).xC;
    }
    (*ip).x40_vel.y += (*ip).xDD4_itemVar.lugia.xE50.x;
    if (*ip).pos.y > ext::Stage_GetBlastZoneTopOffset() {
        (*ip).x40_vel.y = 0.0;
        it_802D16D4(gobj);
    }
}

pub unsafe extern "C" fn itLugia_UnkMotion2_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion2_Coll");
    0
}

pub unsafe fn it_802D16D4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D16D4");
    let item = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE as u32);
    (*item).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*item).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}

pub unsafe extern "C" fn itLugia_UnkMotion3_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion3_Anim");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);
    let child = child_of(jobj);

    ext::it_8027A160(child, ip);

    if ext::it_80272C6C(gobj) == 0 {
        it_802D16D4(gobj);
    }

    0
}

pub unsafe extern "C" fn itLugia_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8027A344(gobj);
    let old = (*ip).xDD4_itemVar.lugia.x60;
    (*ip).xDD4_itemVar.lugia.x60 = old.wrapping_sub(1);
    if old != 0 {
        (*ip).x40_vel.z = (*attrs).x10;
    } else {
        (*ip).x40_vel.z = 0.0;
        it_802D1830(gobj);
    }
}

pub unsafe extern "C" fn itLugia_UnkMotion3_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion3_Coll");
    0
}

pub unsafe fn it_802D1830(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1830");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);

    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    let disc = (*ip).pos.y - (*ip).xDD4_itemVar.lugia.x64.y;
    let x18 = (*attrs).x18;
    let prod = 8.0_f32 * x18;
    // `(prod * -disc) - (x18 * x18)`: the left product contracts into the subtraction
    let sqrtval = my_sqrtf_accurate(-melee_core::fp::fmadds(prod, -disc, -(x18 * x18)));
    (*ip).xDD4_itemVar.lugia.xE50.x = -((-x18 + sqrtval) / 2.0);
}

pub unsafe extern "C" fn itLugia_UnkMotion4_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion4_Anim");
    let ip = GET_ITEM(gobj);
    let jobj = GET_JOBJ(gobj);
    let child = child_of(jobj);

    ext::it_8027A160(child, ip);

    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
        Item_SetEffectHitlagCallbacks(ip);
    }

    0
}

pub unsafe extern "C" fn itLugia_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    ext::it_8027A344(gobj);
    (*ip).xDD4_itemVar.lugia.xE50.x += (*attrs).x18;
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.lugia.xE50.x;
    let new_var = (*ip).x40_vel.y;
    if new_var > 0.0 {
        (*ip).x40_vel.y = 0.0;
        it_802D1A44(gobj);
    }
}

pub unsafe extern "C" fn itLugia_UnkMotion4_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion4_Coll");
    0
}

pub unsafe fn it_802D1A44(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1A44");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    let mut pos: Vec3 = core::mem::zeroed();

    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    (*ip).xDD4_itemVar.lugia.xE50.y = 1.0;
    (*ip).xDD4_itemVar.lugia.xE50.z = 0.7;
    (*ip).xDD4_itemVar.lugia.x88 = 0.4;
    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[25], null_mut(), &mut pos);
    (*ip).xDD4_itemVar.lugia.x8C = (*ip).xDD4_itemVar.lugia.x64;
    (*ip).xDD4_itemVar.lugia.x8C.y += (*attrs).x40;
    (*ip).xDD4_itemVar.lugia.x70 = pos;
    (*ip).xDD4_itemVar.lugia.xA4 = 0;
    (*ip).on_accessory = Some(it_802D1BBC);
    (*ip).xDCC_flag.set_b3(1);
}

pub unsafe extern "C" fn itLugia_UnkMotion5_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion5_Anim");
    let jobj = GET_JOBJ(gobj);
    let ip = GET_ITEM(gobj);

    let jobj = child_of(jobj);
    ext::it_8027A160(jobj, ip);
    if ext::it_80272C6C(gobj) == 0 {
        return 1;
    }
    0
}

pub unsafe extern "C" fn itLugia_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_UnkMotion5_Phys");
    ext::it_8027A344(gobj);
}

pub unsafe extern "C" fn itLugia_UnkMotion5_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLugia_UnkMotion5_Coll");
    0
}

pub unsafe extern "C" fn it_802D1BBC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1BBC");
    let ip = GET_ITEM(gobj);

    if (*ip).xDAC_itcmd_var0 != 0 {
        let attrs = attrs_of(ip);
        (*ip).xDD4_itemVar.lugia.xE50.y += (*attrs).x20;
        (*ip).xDD4_itemVar.lugia.xE50.z += (*attrs).x28;
        (*ip).xDD4_itemVar.lugia.x88 += (*attrs).x30;

        if (*ip).xDD4_itemVar.lugia.xE50.y as f64 >= 1.0 {
            (*ip).xDD4_itemVar.lugia.xE50.y = ((*ip).xDD4_itemVar.lugia.xE50.y as f64 - 1.0) as f32;
            it_802D1E8C(gobj, It_Kind_Lugia_Aeroblast as _, (*attrs).x1C);
            match ext::HSD_Randi(3) {
                0 => {
                    ext::Item_8026AE84(ip, 0x274D, 0x7F, 0x40);
                }
                1 => {
                    ext::Item_8026AE84(ip, 0x274E, 0x7F, 0x40);
                }
                2 => {
                    ext::Item_8026AE84(ip, 0x274F, 0x7F, 0x40);
                }
                _ => {}
            }
        }

        if (*ip).xDD4_itemVar.lugia.xE50.z as f64 >= 1.0 {
            (*ip).xDD4_itemVar.lugia.xE50.z = ((*ip).xDD4_itemVar.lugia.xE50.z as f64 - 1.0) as f32;
            it_802D1E8C(gobj, It_Kind_Lugia_Aeroblast2 as _, (*attrs).x24);
        }

        if (*ip).xDD4_itemVar.lugia.x88 as f64 >= 1.0 {
            (*ip).xDD4_itemVar.lugia.x88 = ((*ip).xDD4_itemVar.lugia.x88 as f64 - 1.0) as f32;
            it_802D1E8C(gobj, It_Kind_Lugia_Aeroblast3 as _, (*attrs).x2C);
        }
    }
}

pub unsafe fn it_802D1D40(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1D40");
    Item_EnterAirStateWithHitlagAndStateDesc(gobj);
}

pub unsafe extern "C" fn it_802D1DB4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D1DB4");
    ext::it_80279FF8(gobj);
    0
}

pub unsafe extern "C" fn it_802D1DD8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D1DD8");
    if ext::it_8027A09C(gobj) != 0 {
        let old_ip = GET_ITEM(gobj);
        ext::it_80273454(gobj);
        let ip = GET_ITEM(gobj);
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        Item_SetEffectHitlagCallbacks(ip);
        (*old_ip).xDD1_flag.x0.set_b1(1);
    }
}

pub unsafe extern "C" fn it_802D1E64(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D1E64");
    // the C casts `void (*)(void)` to the callback type
    ext::it_8027A118(gobj, Some(core::mem::transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(*mut HSD_GObj)>(it_802D14D0)))
}

pub unsafe fn it_802D1E8C(gobj: *mut Item_GObj, kind: ItemKind, param: f32) -> *mut Item_GObj {
    melee_core::ct!("it_802D1E8C");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let ip = GET_ITEM(gobj);

    spawn.prev_pos = (*ip).xDD4_itemVar.lugia.x70;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = (*ip).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel = it_802D1F64(gobj, param);
    spawn.kind = kind;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.x0.set_b0(1);
    spawn.x40 = 0;
    ext::Item_80268B18(&mut spawn)
}

#[inline]
pub fn it_802D208C_inline(x: f32, y: f32, z: f32) -> f32 {
    melee_core::ct!("it_802D208C_inline");
    my_sqrtf(x + y + z)
}

pub unsafe fn it_802D1F64(gobj: *mut Item_GObj, param: f32) -> Vec3 {
    melee_core::ct!("it_802D1F64");
    let ip = GET_ITEM(gobj);
    let mut diff: Vec3 = core::mem::zeroed();

    it_802D208C(gobj);
    ext::lbVector_Diff(&mut (*ip).xDD4_itemVar.lugia.x8C, &mut (*ip).xDD4_itemVar.lugia.x70, &mut diff);
    let dx = (*ip).xDD4_itemVar.lugia.x8C.x - (*ip).xDD4_itemVar.lugia.x70.x;
    let dy = (*ip).xDD4_itemVar.lugia.x8C.y - (*ip).xDD4_itemVar.lugia.x70.y;
    let dz = (*ip).xDD4_itemVar.lugia.x8C.z - (*ip).xDD4_itemVar.lugia.x70.z;
    let dx2 = dx * dx;
    let dy2 = dy * dy;
    let dz2 = dz * dz;
    let scale = it_802D208C_inline(dx2, dy2, dz2) / param;
    diff.x /= scale;
    diff.y /= scale;
    diff.z /= scale;
    diff
}

#[cfg_attr(fptrace, melee_fptrace::trace)]
pub unsafe fn it_802D208C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D208C");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    let mut target: Vec3;

    loop {
        let random_offset: i32 = ext::HSD_Randi(melee_core::ppc::fctiwz_f32(((*attrs).x34 - (*attrs).x38) / 2.0));
        let new_angle: i32;

        if ext::HSD_Rand() % 2 != 0 {
            new_angle = melee_core::ppc::fctiwz_f32(random_offset as f32 + ((*ip).xDD4_itemVar.lugia.xA4 as f32 - (*attrs).x34 / 2.0));
        } else {
            new_angle = melee_core::ppc::fctiwz_f32(random_offset as f32 + ((*ip).xDD4_itemVar.lugia.xA4 as f32 + (*attrs).x38 / 2.0));
        }

        target = (*ip).xDD4_itemVar.lugia.x8C;
        let angle: f32 = new_angle as f32 * 0.01745329252_f32;
        // `target.x += x3C * cosf(angle)` contracts to one fma
        target.x = melee_core::fp::fmadds((*attrs).x3C, cosf(angle), target.x);
        target.y = melee_core::fp::fmadds((*attrs).x3C, sinf(angle), target.y);

        let dx = (*ip).xDD4_itemVar.lugia.x64.x - target.x;
        let dy = (*ip).xDD4_itemVar.lugia.x64.y - target.y;
        let dz = (*ip).xDD4_itemVar.lugia.x64.z - target.z;
        let dx2 = dx * dx;
        let dy2 = dy * dy;
        let dz2 = dz * dz;
        if it_802D208C_inline(dx2, dy2, dz2) > (*attrs).x40 {
            (*ip).xDD4_itemVar.lugia.xA4 = (*ip).xDD4_itemVar.lugia.xA4.wrapping_add(180);
        } else {
            break;
        }
    }
    (*ip).xDD4_itemVar.lugia.x8C = target;
}

pub unsafe fn itLugia_Logic39_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_Logic39_Spawned");
    ext::it_8027ADEC(0x465, gobj, HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.8);
    it_802D23F4(gobj);
}

pub unsafe fn itLugia_Logic40_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_Logic40_Spawned");
    ext::it_8027ADEC(0x466, gobj, HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.8);
    it_802D23F4(gobj);
}

pub unsafe fn itLugia_Logic41_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLugia_Logic41_Spawned");
    ext::it_8027ADEC(0x467, gobj, HSD_GObjGetHSDObj(gobj) as *mut HSD_JObj, 0.8);
    it_802D23F4(gobj);
}

pub unsafe fn it_802D23D4(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D23D4");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe fn it_802D23F4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D23F4");
    let ip = GET_ITEM(gobj);
    let attrs = attrs_of(ip);
    (*ip).xD44_lifeTimer = (*attrs).x0;
    ext::it_80274740(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    Item_SetEffectHitlagCallbacks(ip);
    ext::it_8026B3A8(gobj);
}

pub unsafe extern "C" fn it_802D246C(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D246C");
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip) as i32
}

pub unsafe extern "C" fn it_802D24A0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D24A0");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut f32;
    let mut multiplier: f32 = 0.0;

    let kind = (*ip).kind as i64;
    if kind == It_Kind_Lugia_Aeroblast as i64 {
        multiplier = *attrs.add(1);
    } else if kind == It_Kind_Lugia_Aeroblast2 as i64 {
        multiplier = *attrs.add(2);
    } else if kind == It_Kind_Lugia_Aeroblast3 as i64 {
        multiplier = *attrs.add(3);
    }

    (*ip).x40_vel.z *= multiplier;
}

pub unsafe extern "C" fn it_802D24FC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D24FC");
    ext::it_8026DA08(gobj)
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802D23D4` (src/melee/it/kinds/itlugia.c): `it_8026B894(..)`.
pub fn it_802d23d4(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802d23d4");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_802D24A0` (itlugia.c): `vel.z *= attrs[1..=3]` by aeroblast kind (0 otherwise).
/// # Safety
/// The special attributes must hold at least 4 floats.
pub unsafe fn it_802d24a0(ip: &mut Item) {
    melee_core::ct!("it_802d24a0");
    let a = (*ip.xC4_article_data).x4_specialAttributes as *const f32;
    let multiplier = match ip.kind {
        It_Kind_Lugia_Aeroblast => *a.add(1),
        It_Kind_Lugia_Aeroblast2 => *a.add(2),
        It_Kind_Lugia_Aeroblast3 => *a.add(3),
        _ => 0.0,
    };
    ip.x40_vel.z *= multiplier;
}

// ---- folded from itkinds14.rs (consolidation steps 21/22) ----
/// itlugia.c `it_802D14D0`: empty.
pub fn it_802d14d0() { melee_core::ct!("it_802d14d0");}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `my_sqrtf_accurate_fp` (itlugia.c): 4 steps.
pub fn my_sqrtf_accurate_fp(x: f32) -> f32 {
    melee_core::ct!("my_sqrtf_accurate_fp");
    sqrt_n(x, 4)
}
