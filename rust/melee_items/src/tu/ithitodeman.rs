//! Native port of src/melee/it/kinds/ithitodeman.c (Starmie / "Hitodeman" Pokemon and its star
//! item). Callees in other TUs go through `ext`; sqrtf / lbVector_* are the native ports in
//! melee_runtime. Checked against the C (whole TU, shipping contraction) by tests/tu_ithitodeman.rs.
//!
//! Fused sites (`fmadds`): `it_802D472C` 2 (length squared), `itHitodeman_UnkMotion1_Phys` 1
//! (`-((-facing * x4C) - vel.x)` -> `-fmadds(-facing, x4C, -vel.x)`), `it_802D43EC` 2,
//! `it_802D4C74` 1 (`prev_pos.x += facing * x54`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::layout::itPikachuthunderAttributes;
#[allow(unused_imports)]
use crate::layout::itRshellAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruCann_DatAttrs;
#[allow(unused_imports)]
use crate::layout::itThunderPokemonAttributes;
#[allow(unused_imports)]
use crate::layout::itUnknownAttributes;
#[allow(unused_imports)]
use crate::layout::ItLGunAttr;
#[allow(unused_imports)]
use crate::layout::MaximTomatoSpecialAttr;
#[allow(unused_imports)]
use crate::layout::ScopeBeamAttrs;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
#[allow(unused_imports)]
use crate::itkinds5::{c_abs as c_abs_itkinds5};
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
use crate::itkinds9::my_sqrtf_accurate;
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
use crate::helpers::clamp_tau;
#[allow(unused_imports)]
use crate::helpers::it_8027b798_calc_angle;
#[allow(unused_imports)]
use crate::helpers::COLLIDE_CEILING_MASK;
#[allow(unused_imports)]
use crate::helpers::M_PI;
#[allow(unused_imports)]
use crate::helpers::M_TAU;
#[allow(unused_imports)]
use crate::item::norm_xf74;
#[allow(unused_imports)]
use crate::itlib::it_80276408;
#[allow(unused_imports)]
use crate::itlib::COLLIDE_FLOOR_MASK;
#[allow(unused_imports)]
use crate::layout::ftCommonData;
#[allow(unused_imports)]
use crate::layout::itBoxAttributes;
#[allow(unused_imports)]
use crate::layout::itMarilAttributes;
#[allow(unused_imports)]
use crate::layout::itPikachutJoltGroundAttributes;
#[allow(unused_imports)]
use crate::layout::itTincleAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::It_Kind_Flipper;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_runtime::gekko::cosf;
#[allow(unused_imports)]
use melee_runtime::gekko::sinf;
#[allow(unused_imports)]
use melee_runtime::lbtrigf::atan2f;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
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
use std::ptr::null_mut as null_mut_itkinds;
#[allow(unused_imports)]
use crate::itfwd::it_maril_unk_motion2_anim;
#[allow(unused_imports)]
use crate::itfwd::it_zgshell_logic11_reflected;
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
use crate::helpers::M_PI_2;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_core::vec::Vec3 as RVec;
use melee_runtime::gekko::sqrtf;
use melee_runtime::lbvector;
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efLib_PauseAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_PauseAll(a0) };
    fn efLib_ResumeAll(a0: *mut HSD_GObj) => unsafe { melee_world::tu::efn::eflib::efLib_ResumeAll(a0) };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[*const c_void]) -> *mut c_void;
    fn ftLib_FindLowestPercentOpponent(a0: *mut HSD_GObj) -> *mut HSD_GObj;
    fn ftLib_GetPos(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::ftlib::ftLib_GetPos(a0, a1) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026BB88(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB88(a0, a1 as _) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_80273454(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80273454(a0) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80279C48(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279C48(a0) };
    fn it_80279CDC(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_279C::it_80279CDC(a0, a1) };
    fn it_80279FF8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_279C::it_80279FF8(a0) };
    fn it_8027A09C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027A09C(a0) as _ };
    fn it_8027A118(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32 => unsafe { crate::tu::it_279C::it_8027A118(a0, a1) as _ };
    fn it_8027AE34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027AE34(a0) as _ };
    fn it_8027AF50(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027AF50(a0) as _ };
    fn lbAudioAx_800263E8(a0: f32, a1: *mut HSD_GObj, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, a8: i32, a9: i32, a10: i32) -> *mut HSD_GObj;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn efLib_PauseAll(a0: *mut HSD_GObj) { super::efLib_PauseAll(a0) }
        pub unsafe extern "C" fn efLib_ResumeAll(a0: *mut HSD_GObj) { super::efLib_ResumeAll(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut guess: f64 = unsafe { core::mem::zeroed() };
    pub static mut item: *mut Item = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
    pub static mut y: f32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}
#[inline]
unsafe fn attrs_of(ip: *mut Item) -> *mut itHitodemanAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHitodemanAttributes
}
#[inline]
unsafe fn set_effect_hitlag_callbacks(ip: *mut Item) {
    (*ip).entered_hitlag = Some(ext::cb::efLib_PauseAll);
    (*ip).exited_hitlag = Some(ext::cb::efLib_ResumeAll);
}
/// `Item_EnterAirStateWithHitlag` (src/melee/it/kinds/inlines.h)
#[inline]
unsafe fn enter_air_state_with_hitlag(gobj: *mut Item_GObj, msid: i32) {
    let item = ip_of(gobj);
    ext::it_802762BC(item);
    ext::Item_80268E5C(gobj, msid, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(item);
}
/// `HSD_JObjMtxIsDirty` (jobj.h)
#[inline]
unsafe fn jobj_mtx_is_dirty(jobj: *mut HSD_JObj) -> bool {
    assert!(!jobj.is_null(), "jobj");
    (*jobj).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*jobj).flags & JOBJ_MTX_DIRTY as u32 != 0
}
/// `HSD_JObjSetRotationY` (jobj.h)
#[inline]
unsafe fn jobj_set_rotation_y(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "jobj");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 && !jobj.is_null() && !jobj_mtx_is_dirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}
#[inline]
fn c_abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// `ItemStateTable it_803F8128[3]` (3F8128)
pub static it_803F8128: [ItemStateTable; 3] = [
    ItemStateTable { anim_id: 0, animated: Some(it_802D4564), physics_updated: Some(it_802D472C), collided: Some(it_802D48A8) },
    ItemStateTable {
        anim_id: 1,
        animated: Some(itHitodeman_UnkMotion1_Anim),
        physics_updated: Some(itHitodeman_UnkMotion1_Phys),
        collided: Some(itHitodeman_UnkMotion1_Coll),
    },
    ItemStateTable {
        anim_id: -1,
        animated: Some(itHitodeman_UnkMotion2_Anim),
        physics_updated: Some(itHitodeman_UnkMotion2_Phys),
        collided: Some(itHitodeman_UnkMotion2_Coll),
    },
];

/// `ItemStateTable it_803F8180[1]` (3F8180)
pub static it_803F8180: [ItemStateTable; 1] =
    [ItemStateTable { anim_id: 0, animated: Some(it_802D4FFC), physics_updated: Some(it_802D5044), collided: Some(it_802D5048) }];

/// `void it_2725_Logic24_Spawned(Item_GObj*)` (2D42C4)
pub unsafe extern "C" fn it_2725_Logic24_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic24_Spawned");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);

    (*ip).facing_dir = 0.0;
    (*ip).anon_dbc.xDBC_itcmd_var4.flags.set_x0(0);

    ext::it_80279CDC(gobj, (*attrs).x0);
    it_802D43EC(gobj);
    it_802D4494(gobj);
    it_802D4510(gobj);

    (*ip).xDD4_itemVar.hitodeman.x8C = (*attrs).x3C as f32;

    (*ip).xDD4_itemVar.hitodeman.x6C = 0.0;
    (*ip).xDD4_itemVar.hitodeman.x70 = 0.0;
    (*ip).xDD4_itemVar.hitodeman.x74 = 0.0;
    (*ip).xDD4_itemVar.hitodeman.x78 = 0.0;
    (*ip).xDD4_itemVar.hitodeman.x7C = 0.0;
    (*ip).xDD4_itemVar.hitodeman.x80 = 0.0;

    (*ip).xDAC_itcmd_var0 = 0;
    (*ip).xDB0_itcmd_var1 = 0;

    it_802D4B54(gobj);
    ext::Item_8026AE84(ip, 0x2738, 0x7F, 0x40);
}

/// `void it_802D43AC(Item_GObj*)` (2D43AC)
pub unsafe extern "C" fn it_802D43AC(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D43AC");}

/// `void it_802D43B0(Item_GObj*, Item_GObj*)` (2D43B0)
pub unsafe extern "C" fn it_802D43B0(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D43B0");
    let ip = ip_of(gobj);
    ext::it_8026B894(gobj, ref_gobj);
    (*ip).xDD4_itemVar.hitodeman.x90 = null_mut();
    (*ip).xDAC_itcmd_var0 = 1;
}

/// `void it_802D43EC(Item_GObj*)` (2D43EC)
pub unsafe extern "C" fn it_802D43EC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D43EC");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);

    let owner_gobj = ext::ftLib_FindLowestPercentOpponent((*ip).owner);
    if !owner_gobj.is_null() {
        (*ip).xDD4_itemVar.hitodeman.x90 = owner_gobj;
    } else {
        (*ip).xDD4_itemVar.hitodeman.x90 = (*ip).owner;
    }

    let randf = ext::HSD_Randf();
    let mut diff = (*attrs).x8;
    diff = (*attrs).x4 - diff;
    (*ip).xDD4_itemVar.hitodeman.x60 = fmadds(diff, randf, (*attrs).x8);

    let randf = ext::HSD_Randf();
    let diff = (*attrs).xC - (*attrs).x10;
    (*ip).xDD4_itemVar.hitodeman.x64 = fmadds(diff, randf, (*attrs).x10);

    if ext::HSD_Randi(2) != 0 {
        (*ip).xDD4_itemVar.hitodeman.x60 *= -1.0;
    }
}

/// `void it_802D4494(Item_GObj*)` (2D4494)
pub unsafe extern "C" fn it_802D4494(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4494");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);
    let randi = ext::HSD_Randi(melee_core::ppc::fctiwz_f32((*attrs).x14 - (*attrs).x18));
    (*ip).xDD4_itemVar.hitodeman.x84 = (*attrs).x18 + randi as f32;
}

/// `void it_802D4510(Item_GObj*)` (2D4510)
pub unsafe extern "C" fn it_802D4510(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4510");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);
    (*ip).xDD4_itemVar.hitodeman.x88 =
        (*attrs).x44.wrapping_add(ext::HSD_Randi((*attrs).x40.wrapping_sub((*attrs).x44)));
}

/// `static inline void it_802D4564_anim_done(Item_GObj*)`
#[inline]
pub unsafe fn it_802D4564_anim_done(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4564_anim_done");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
}

/// `static inline Article* it_802D472C_inline(Item*)`
#[inline]
pub unsafe fn it_802D472C_inline(ip: *mut Item) -> *mut Article {
    melee_core::ct!("it_802D472C_inline");
    (*ip).xC4_article_data
}

/// `bool it_802D4564(Item_GObj*)` (2D4564)
pub unsafe extern "C" fn it_802D4564(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4564");
    let ip = ip_of(gobj);

    if (*ip).xDAC_itcmd_var0 != 0 {
        return 1;
    }
    if ext::it_80272C6C(gobj) == 0 {
        it_802D4564_anim_done(gobj);
    }
    (*ip).xDD4_itemVar.hitodeman.x84 -= 1.0;
    if (*ip).xDD4_itemVar.hitodeman.x84 < 0.0 {
        if !(*ip).xDD4_itemVar.hitodeman.x90.is_null() {
            let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
            let mut target_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
            ext::ftLib_GetPos((*ip).xDD4_itemVar.hitodeman.x90, &mut target_pos);
            if target_pos.x < (*ip).pos.x {
                (*ip).facing_dir = -1.0;
            } else {
                (*ip).facing_dir = 1.0;
            }
            jobj_set_rotation_y(jobj, 1.5707964 * (*ip).facing_dir);
        } else {
            ext::it_80279C48(gobj);
        }
        it_802D4990(gobj);
        (*ip).x40_vel.x = 0.0;
        (*ip).x40_vel.y = 0.0;
        (*ip).x40_vel.z = 0.0;
        ext::Item_8026AE84(ip, 0x2738, 127, 64);
    }
    0
}

/// `void it_802D472C(Item_GObj*)` (2D472C)
pub unsafe extern "C" fn it_802D472C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D472C");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);
    let mut target_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
    let mut dir = Vec { x: 0.0, y: 0.0, z: 0.0 };

    if !(*ip).xDD4_itemVar.hitodeman.x90.is_null() {
        ext::ftLib_GetPos((*ip).xDD4_itemVar.hitodeman.x90, &mut target_pos);
        target_pos.x += (*ip).xDD4_itemVar.hitodeman.x60;
        target_pos.y += (*ip).xDD4_itemVar.hitodeman.x64;
        target_pos.z = 0.0;
        let d = lbvector::diff(&*(&target_pos as *const Vec as *const RVec), &*(&(*ip).pos as *const Vec as *const RVec));
        dir = Vec { x: d.x, y: d.y, z: d.z };
        // dir's address leaves the function: squares not fused on the console (notes/fp-audit.md)
        let dist = sqrtf(dir.x * dir.x + dir.y * dir.y + dir.z * dir.z);
        if dist > (*attrs).x38 {
            lbvector::normalize(&mut *(&mut dir as *mut Vec as *mut RVec));
            let speed = dist * (*attrs).x34;
            dir.x *= speed;
            dir.y *= speed;
            dir.z *= speed;
        }
        (*ip).x40_vel.x = it_802D48B0(
            &raw mut (*ip).xDD4_itemVar.hitodeman.x6C,
            dir.x,
            (*attrs).x24,
            (*attrs).x20,
            (*attrs).x1C,
        );
        (*ip).x40_vel.y = it_802D48B0(
            &raw mut (*ip).xDD4_itemVar.hitodeman.x70,
            dir.y,
            (*attrs).x30,
            (*attrs).x2C,
            (*attrs).x28,
        );
    }
}

/// `bool it_802D48A8(Item_GObj*)` (2D48A8)
pub unsafe extern "C" fn it_802D48A8(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D48A8");
    0
}

/// `f32 it_802D48B0(f32* value, f32 target, f32 max_val, f32 accel, f32 decel)` (2D48B0)
pub unsafe extern "C" fn it_802D48B0(value: *mut f32, mut target: f32, max_val: f32, accel: f32, decel: f32) -> f32 {
    melee_core::ct!("it_802D48B0");
    let current = *value;
    let diff = target - current;

    if c_abs(diff) > c_abs(accel) {
        if diff > 0.0 {
            target = current + accel;
        } else {
            target = current - accel;
        }
    } else if c_abs(diff) < c_abs(decel) {
        if diff > 0.0 {
            target = current + decel;
        } else {
            target = current - decel;
        }
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

/// `void it_802D4990(Item_GObj*)` (2D4990)
pub unsafe extern "C" fn it_802D4990(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4990");
    let ip = ip_of(gobj);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    (*ip).on_accessory = Some(it_802D4B50);
}

/// `bool itHitodeman_UnkMotion1_Anim(Item_GObj*)` (2D49E8)
pub unsafe extern "C" fn itHitodeman_UnkMotion1_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHitodeman_UnkMotion1_Anim");
    let mut ip = ip_of(gobj);

    if (*ip).xDAC_itcmd_var0 != 0 {
        return 1;
    }

    let timer = (*ip).xDD4_itemVar.hitodeman.x8C - 1.0;
    (*ip).xDD4_itemVar.hitodeman.x8C = timer;

    if timer < 0.0 {
        (*ip).xDD4_itemVar.hitodeman.x88 = (*ip).xDD4_itemVar.hitodeman.x88.wrapping_sub(1);
        if (*ip).xDD4_itemVar.hitodeman.x88 < 0 {
            return 1;
        }

        (*ip).xDD4_itemVar.hitodeman.x8C = (*attrs_of(ip)).x3C as f32;

        if !(*ip).xDD4_itemVar.hitodeman.x90.is_null() {
            let attrs = attrs_of(ip);
            (*ip).x40_vel.x = -(*ip).facing_dir * (*attrs).x48;
            it_802D4C74(gobj);
            (*ip).xDB0_itcmd_var1 = 1;
        }
    }

    if ext::it_80272C6C(gobj) == 0 {
        ip = ip_of(gobj);
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
        set_effect_hitlag_callbacks(ip);
        (*ip).on_accessory = Some(it_802D4B50);
    }

    0
}

/// `void itHitodeman_UnkMotion1_Phys(Item_GObj*)` (2D4B14)
pub unsafe extern "C" fn itHitodeman_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHitodeman_UnkMotion1_Phys");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);
    if (*ip).xDB0_itcmd_var1 != 0 {
        (*ip).x40_vel.x = -fmadds(-(*ip).facing_dir, (*attrs).x4C, -(*ip).x40_vel.x);
    }
}

/// `bool itHitodeman_UnkMotion1_Coll(Item_GObj*)` (2D4B48)
pub unsafe extern "C" fn itHitodeman_UnkMotion1_Coll(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHitodeman_UnkMotion1_Coll");
    0
}

/// `void it_802D4B50(Item_GObj*)` (2D4B50)
pub unsafe extern "C" fn it_802D4B50(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D4B50");}

/// `void it_802D4B54(Item_GObj*)` (2D4B54)
pub unsafe extern "C" fn it_802D4B54(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4B54");
    enter_air_state_with_hitlag(gobj, 2);
}

/// `bool itHitodeman_UnkMotion2_Anim(Item_GObj*)` (2D4BB8)
pub unsafe extern "C" fn itHitodeman_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHitodeman_UnkMotion2_Anim");
    ext::it_80279FF8(gobj);
    0
}

/// `void itHitodeman_UnkMotion2_Phys(Item_GObj*)` (2D4BDC)
pub unsafe extern "C" fn itHitodeman_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itHitodeman_UnkMotion2_Phys");
    if ext::it_8027A09C(gobj) != 0 {
        ext::it_80273454(gobj);
        let ip = ip_of(gobj);
        ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
        set_effect_hitlag_callbacks(ip);
    }
}

/// `bool itHitodeman_UnkMotion2_Coll(Item_GObj*)` (2D4C4C)
pub unsafe extern "C" fn itHitodeman_UnkMotion2_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHitodeman_UnkMotion2_Coll");
    ext::it_8027A118(gobj, Some(it_802D43AC))
}

/// `void it_802D4C74(Item_GObj*)` (2D4C74): throw a star.
pub unsafe extern "C" fn it_802D4C74(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4C74");
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.prev_pos = (*ip).pos;
    spawn.prev_pos.x = fmadds((*ip).facing_dir, (*attrs).x54, spawn.prev_pos.x);
    spawn.prev_pos.y += (*attrs).x58;
    ext::it_8026BB88(gobj, &mut spawn.pos);
    spawn.facing_dir = (*ip).facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.x = spawn.facing_dir * (*attrs).x50;
    spawn.vel.y = 0.0;
    spawn.vel.z = 0.0;
    spawn.kind = It_Kind_Hitodeman_Star;
    spawn.x0_parent_gobj = (*ip).owner;
    spawn.x4_parent_gobj2 = gobj;
    spawn.x44_flag.anon_0.set_b0(0);
    spawn.x40 = 0;
    let item_gobj = ext::Item_80268B18(&mut spawn);
    if !item_gobj.is_null() {
        if (*ip).facing_dir == 1.0 {
            ext::lbAudioAx_800263E8(3.0, gobj, 1, 0x2739, 127, 127, 0, 127, 60, 0, -1);
        } else {
            ext::lbAudioAx_800263E8(3.0, gobj, 1, 0x273A, 127, 127, 0, 127, 60, 0, -1);
        }
    }
}

/// `void it_2725_Logic43_Spawned(Item_GObj*)` (2D4DE0)
pub unsafe extern "C" fn it_2725_Logic43_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("it_2725_Logic43_Spawned");
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let ip = ip_of(gobj);
    let attrs = attrs_of(ip);

    ext::it_80275158(gobj, (*attrs).x0);
    (*ip).xDB4_itcmd_var2 = 0;
    ext::it_80274740(gobj);
    it_802D4F78(gobj);
    ext::it_8026B3A8(gobj);
    jobj_set_rotation_y(jobj, M_PI_2 as f32 * (*ip).facing_dir);
}

/// `bool it_802D4EF4(Item_GObj*)` (2D4EF4)
pub unsafe extern "C" fn it_802D4EF4(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4EF4");
    (*ip_of(gobj)).xDB4_itcmd_var2 = 1;
    0
}

/// `bool it_802D4F08(Item_GObj*)` (2D4F08)
pub unsafe extern "C" fn it_802D4F08(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4F08");
    ext::it_8027AF50(gobj)
}

/// `bool it_802D4F28(Item_GObj*)` (2D4F28)
pub unsafe extern "C" fn it_802D4F28(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4F28");
    ext::it_8027AE34(gobj)
}

/// `bool itHitodeman_Logic43_Absorbed(Item_GObj*)` (2D4F48)
pub unsafe extern "C" fn itHitodeman_Logic43_Absorbed(_arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("itHitodeman_Logic43_Absorbed");
    1
}

/// `bool it_802D4F50(Item_GObj*)` (2D4F50)
pub unsafe extern "C" fn it_802D4F50(_arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4F50");
    1
}

/// `void it_802D4F58(Item_GObj*, Item_GObj*)` (2D4F58)
pub unsafe extern "C" fn it_802D4F58(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4F58");
    ext::it_8026B894(gobj, ref_gobj);
}

/// `void it_802D4F78(Item_GObj*)` (2D4F78)
pub unsafe extern "C" fn it_802D4F78(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802D4F78");
    let mut f: f32 = 1.0;
    let ip = ip_of(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
    set_effect_hitlag_callbacks(ip);
    ext::efSync_Spawn(0x46C, gobj, &[jobj as *const c_void, &raw const f as *const c_void]);
}

/// `bool it_802D4FFC(Item_GObj*)` (2D4FFC)
pub unsafe extern "C" fn it_802D4FFC(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D4FFC");
    let ip = ip_of(gobj);
    let timer = (*ip).xD44_lifeTimer - 1.0;
    (*ip).xD44_lifeTimer = timer;
    if timer <= 0.0 {
        return 1;
    }
    if (*ip).xDB4_itcmd_var2 != 0 {
        return 1;
    }
    0
}

/// `void it_802D5044(Item_GObj*)` (2D5044)
pub unsafe extern "C" fn it_802D5044(_gobj: *mut Item_GObj) { melee_core::ct!("it_802D5044");}

/// `bool it_802D5048(Item_GObj*)` (2D5048)
pub unsafe extern "C" fn it_802D5048(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802D5048");
    0
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802D4F58` (src/melee/it/kinds/ithitodeman.c): `it_8026B894(..)`.
pub fn it_802d4f58(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802d4f58");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itfwd6.rs (consolidation steps 21/22) ----
/// `it_802D4F50` (src/melee/it/kinds/ithitodeman.c): `return true;`.
pub fn it_802d4f50(_ip: &mut Item) -> bool {
    melee_core::ct!("it_802d4f50");
    true
}

// ---- folded from itfwd8.rs (consolidation steps 21/22) ----
/// `it_802D43AC` (src/melee/it/kinds/ithitodeman.c): `empty body / return;`.
pub fn it_802d43ac(_ip: &mut Item) { melee_core::ct!("it_802d43ac");}

/// `it_802D48A8` (src/melee/it/kinds/ithitodeman.c): `return false;`.
pub fn it_802d48a8(_ip: &mut Item) -> bool {
    melee_core::ct!("it_802d48a8");
    false
}

/// `it_802D4B50` (src/melee/it/kinds/ithitodeman.c): `empty body / return;`.
pub fn it_802d4b50(_ip: &mut Item) { melee_core::ct!("it_802d4b50");}

/// `it_802D5044` (src/melee/it/kinds/ithitodeman.c): `empty body / return;`.
pub fn it_802d5044(_ip: &mut Item) { melee_core::ct!("it_802d5044");}

/// `it_802D5048` (src/melee/it/kinds/ithitodeman.c): `return false;`.
pub fn it_802d5048(_ip: &mut Item) -> bool {
    melee_core::ct!("it_802d5048");
    false
}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_802D4FFC` (ithitodeman.c): life timer down; true when out or `xDB4_itcmd_var2` is set.
pub fn it_802d4ffc(ip: &mut Item) -> bool {
    melee_core::ct!("it_802d4ffc");
    let timer = ip.xD44_lifeTimer - 1.0;
    ip.xD44_lifeTimer = timer;
    if timer <= 0.0 {
        return true;
    }
    ip.xDB4_itcmd_var2 != 0
}

/// `it_802D4EF4` (ithitodeman.c): `xDB4_itcmd_var2 = 1`.
pub fn it_802d4ef4(ip: &mut Item) -> bool {
    melee_core::ct!("it_802d4ef4");
    ip.xDB4_itcmd_var2 = 1;
    false
}

// ---- folded from itkinds10.rs (consolidation steps 21/22) ----
// ---- ithitodeman.c ---------------------------------------------------------------------

/// `it_802D4494`: `x84 = x18 + randi((s32)(x14 - x18))` (the float-to-int conversion saturates,
/// NaN -> 0, as the build's lowering does).
/// # Safety
/// The item's special attributes are an `itHitodemanAttributes`.
pub unsafe fn it_802d4494(rand: &mut HsdRand, ip: &mut Item) {
    melee_core::ct!("it_802d4494");
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itHitodemanAttributes);
    let r = rand.randi(melee_core::ppc::fctiwz_f32(a.x14 - a.x18));
    ip.xDD4_itemVar.hitodeman.x84 = a.x18 + r as f32;
}

/// `it_802D4510`: `x88 = x44 + randi(x40 - x44)`.
/// # Safety
/// The item's special attributes are an `itHitodemanAttributes`.
pub unsafe fn it_802d4510(rand: &mut HsdRand, ip: &mut Item) {
    melee_core::ct!("it_802d4510");
    let a = &*((*ip.xC4_article_data).x4_specialAttributes as *const itHitodemanAttributes);
    ip.xDD4_itemVar.hitodeman.x88 = a.x44.wrapping_add(rand.randi(a.x40.wrapping_sub(a.x44)));
}

// ---- folded from itkinds14.rs (consolidation steps 21/22) ----
/// ithitodeman.c `it_802D472C_inline`: `ip->xC4_article_data`.
pub fn it_802d472c_inline(ip: &Item) -> *mut Article {
    melee_core::ct!("it_802d472c_inline");
    ip.xC4_article_data
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `it_802D43B0` (ithitodeman.c): `it_8026B894`, `hitodeman.x90 = NULL`, `xDAC = 1`.
pub fn it_802d43b0(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802d43b0");
    it_8026b894(ip, ref_gobj);
    ip.xDD4_itemVar.hitodeman.x90 = null_mut_itkinds5();
    ip.xDAC_itcmd_var0 = 1;
}

// ---- pure ------------------------------------------------------------------------------

/// `it_802D48B0` (ithitodeman.c): move `*value` towards `target` by at most `accel` (or at
/// least `decel`), clamp to `+-max_val`, store and return it.
pub fn it_802d48b0(value: &mut f32, mut target: f32, max_val: f32, accel: f32, decel: f32) -> f32 {
    melee_core::ct!("it_802d48b0");
    let current = *value;
    let diff = target - current;
    if c_abs_itkinds5(diff) > c_abs_itkinds5(accel) {
        target = if diff > 0.0 { current + accel } else { current - accel };
    } else if c_abs_itkinds5(diff) < c_abs_itkinds5(decel) {
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
