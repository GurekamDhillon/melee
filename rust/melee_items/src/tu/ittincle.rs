//! Port of src/melee/it/kinds/ittincle.c: Tingle (Hyrule Temple, logic table 13). Callees of unported
//! TUs go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction)
//! by tests/kinds_ittincle.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_coll_bounce_off_victim;
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
use crate::helpers::clamp_tau;
#[allow(unused_imports)]
use crate::helpers::it_8027b798_calc_angle;
#[allow(unused_imports)]
use crate::helpers::COLLIDE_CEILING_MASK;
#[allow(unused_imports)]
use crate::helpers::M_PI;
#[allow(unused_imports)]
use crate::helpers::M_PI_2;
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
use crate::layout::itHitodemanAttributes;
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
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::GA_Air;
#[allow(unused_imports)]
use crate::layout::It_Kind_Flipper;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_core::vec::Vec3 as RVec;
#[allow(unused_imports)]
use melee_runtime::gekko::cosf;
#[allow(unused_imports)]
use melee_runtime::gekko::sinf;
#[allow(unused_imports)]
use melee_runtime::lbtrigf::atan2f;
#[allow(unused_imports)]
use melee_runtime::lbvector;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
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
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn Ground_801C0498() -> f32 => unsafe { melee_world::tu::ground::Ground_801C0498() };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjSetFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026A8EC(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_8026A8EC(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026D6F4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D6F4(a0, a1) };
    fn it_8026DA08(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DA08(a0) };
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::it_80272C6C(a0) };
    fn it_8027542C(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8027542c(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756D0(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_802756d0(&mut *((*a0).user_data as *mut Item)) };
    fn it_802756E0(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_802756e0(&mut *((*a0).user_data as *mut Item)) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027B730(a0: *mut HSD_GObj) => unsafe { crate::itlib3::it_8027b730(&mut *((*a0).user_data as *mut Item)) };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) => unsafe { melee_runtime::tu::lb_00B0::lb_8000B1CC(a0, a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

crate::kinds_jobj_inl!();

/// `HSD_JObjSetScaleX` (baselib/jobj.h, static inline)
#[inline]
pub unsafe fn HSD_JObjSetScaleX(jobj: *mut HSD_JObj, x: f32) {
    melee_core::ct!("HSD_JObjSetScaleX");
    hsd_assert(!jobj.is_null(), "(jobj)");
    (*jobj).scale.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

/// `ABS(x)` (Runtime/platform.h) for f32.
#[inline]
fn ABS(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `-1.0f` as an operand of a real multiply (`x *= -1.0f`): a NaN keeps its sign on the console and in
/// the C build; black_box keeps LLVM from turning the multiply into a negation.
#[inline]
fn neg_one() -> f32 {
    core::hint::black_box(-1.0f32)
}

#[inline]
unsafe fn SA(ip: *mut Item) -> *mut itTincleAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itTincleAttributes
}

/// `ItemStateTable it_803F9000[]`
pub static mut it_803F9000: [ItemStateTable; 13] = [
    st_row(-1, pred_b!(itTincle_UnkMotion0_Anim), Some(itTincle_UnkMotion0_Phys), pred_b!(itTincle_UnkMotion0_Coll)),
    st_row(0, pred_b!(itTincle_UnkMotion1_Anim), Some(itTincle_UnkMotion1_Phys), pred_b!(itTincle_UnkMotion1_Coll)),
    st_row(0, pred_b!(itTincle_UnkMotion2_Anim), Some(itTincle_UnkMotion2_Phys), pred_b!(itTincle_UnkMotion2_Coll)),
    st_row(0, pred_b!(itTincle_UnkMotion3_Anim), Some(itTincle_UnkMotion3_Phys), pred_b!(itTincle_UnkMotion3_Coll)),
    st_row(0, pred_b!(itTincle_UnkMotion4_Anim), Some(itTincle_UnkMotion4_Phys), pred_b!(itTincle_UnkMotion4_Coll)),
    st_row(0, pred_b!(itTincle_UnkMotion5_Anim), Some(itTincle_UnkMotion5_Phys), pred_b!(itTincle_UnkMotion5_Coll)),
    st_row(1, pred_b!(itTincle_UnkMotion7_Anim), Some(itTincle_UnkMotion7_Phys), pred_b!(itTincle_UnkMotion7_Coll)),
    st_row(2, pred_b!(itTincle_UnkMotion7_Anim), Some(itTincle_UnkMotion7_Phys), pred_b!(itTincle_UnkMotion7_Coll)),
    st_row(3, pred_b!(itTincle_UnkMotion8_Anim), Some(itTincle_UnkMotion8_Phys), pred_b!(itTincle_UnkMotion8_Coll)),
    st_row(4, pred_b!(itTincle_UnkMotion9_Anim), Some(itTincle_UnkMotion9_Phys), pred_b!(itTincle_UnkMotion9_Coll)),
    st_row(5, pred_b!(itTincle_UnkMotion10_Anim), Some(itTincle_UnkMotion10_Phys), pred_b!(itTincle_UnkMotion10_Coll)),
    st_row(6, pred_b!(itTincle_UnkMotion11_Anim), Some(itTincle_UnkMotion11_Phys), pred_b!(itTincle_UnkMotion11_Coll)),
    st_row(6, pred_b!(itTincle_UnkMotion12_Anim), Some(itTincle_UnkMotion12_Phys), pred_b!(itTincle_UnkMotion12_Coll)),
];

pub unsafe extern "C" fn it_802EB5C8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EB5C8");
    let mut pos: Vec3 = core::mem::zeroed();
    let ip = GET_ITEM(gobj);

    ext::it_8027B730(gobj);
    (*ip).xD5C = 0;
    (*ip).xDCC_flag.set_b3(0);
    (*ip).xDC8_word.flags.set_x15(0);
    ext::it_8027542C(gobj);

    ext::lb_8000B1CC((*(*ip).xBBC_dynamicBoneTable).bones[11], null_mut(), &mut pos);

    (*ip).xDD4_itemVar.tincle.x50 = ABS(pos.y - (*ip).pos.y);

    (*ip).xDD4_itemVar.tincle.x68 = (*ip).xC1C;
    it_802EB6DC(gobj);
}

pub unsafe extern "C" fn itTincle_Logic13_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_Logic13_DmgReceived");
    it_802EC18C(gobj);
    false
}

pub unsafe extern "C" fn itTincle_Logic13_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_Logic13_DmgDealt");
    if (*GET_ITEM(gobj)).msid != 7 {
        it_802EC1F4(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802EB6DC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EB6DC");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);

    let randi_result = ext::HSD_Randi(melee_core::ppc::fctiwz_f32(ABS((*sa).x10 - (*sa).xC)));
    (*ip).pos.x = (*sa).xC + randi_result as f32;
    (*ip).pos.y = (*sa).x14;
    (*ip).pos.z = 2.0;
    ext::it_802762BC(ip);
    itResetVelocity(ip);
    let mut range_i = (*sa).x4.wrapping_sub((*sa).x8);
    if range_i < 0 {
        range_i = range_i.wrapping_neg();
    }
    (*ip).xDD4_itemVar.tincle.x20 = ext::HSD_Randi(range_i);
    (*ip).xDD4_itemVar.tincle.x24 = 0;
    (*ip).xDD4_itemVar.tincle.x28 = 0;
    (*ip).xDD4_itemVar.tincle.x2C = 0;
    (*ip).xDD4_itemVar.tincle.x34 = 0.0;
    (*ip).xDD4_itemVar.tincle.x38 = 0.0;
    (*ip).xDD4_itemVar.tincle.x4C = 0.0;
    ext::it_802756D0(gobj);
    ext::HSD_JObjSetFlagsAll(GET_JOBJ(gobj), JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion0_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tincle.x20 <= 0 {
        it_802EB870(gobj);
        return;
    }
    (*ip).xDD4_itemVar.tincle.x20 -= 1;
}

pub unsafe extern "C" fn itTincle_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion0_Coll");
    ext::it_8026DA08(gobj);
    false
}

pub unsafe extern "C" fn it_802EB870(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EB870");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);

    let mut s: i32 = melee_core::ppc::fctiwz_f32(ABS((*sa).x10 - (*sa).xC) / 2.0);
    (*ip).xDD4_itemVar.tincle.x58 = (*sa).xC + s as f32;
    s = ext::HSD_Randi(melee_core::ppc::fctiwz_f32(ABS((*sa).x1C - (*sa).x20)));
    (*ip).xDD4_itemVar.tincle.x5C = (*sa).x20 + s as f32;
    (*ip).x40_vel.y = -(*sa).x18;
    (*ip).x40_vel.x = 0.0;
    ext::HSD_JObjClearFlagsAll(GET_JOBJ(gobj), JOBJ_HIDDEN as u32);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion1_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).pos.y <= (*ip).xDD4_itemVar.tincle.x5C {
        it_802EBA00(gobj);
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion1_Coll");
    ext::it_8026E414(gobj, Some(it_802EBA00));
    false
}

pub unsafe extern "C" fn it_802EBA00(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EBA00");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);
    let dur_f = (*sa).x28 as f32;
    let height = (*sa).x24;

    itResetVelocity(ip);

    (*ip).xDD4_itemVar.tincle.x20 = (*sa).x2C;
    (*ip).x40_vel.y = 0.0;

    (*ip).xDD4_itemVar.tincle.x38 = (2.0 * height) / (dur_f * dur_f);
    (*ip).xDD4_itemVar.tincle.x38 *= 0.5;

    (*ip).xDD4_itemVar.tincle.x2C = (*sa).x28.wrapping_mul(2);
    (*ip).xDD4_itemVar.tincle.x3C = 2.0;
    (*ip).xDD4_itemVar.tincle.x24 = 0;

    ext::it_802756E0(gobj);

    (*ip).xDC8_word.flags.set_x1A(1);

    ext::HSD_JObjClearFlagsAll(GET_JOBJ(gobj), JOBJ_HIDDEN as u32);

    ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1);

    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion2_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE);
    }
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion2_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let bounce_count = (*ip).xDD4_itemVar.tincle.x3C;
    let sa = SA(ip);

    if bounce_count == 0.0 {
        (*ip).xDD4_itemVar.tincle.x3C = 2.0;
        (*ip).xDD4_itemVar.tincle.x2C = (*sa).x28.wrapping_mul(2);
        (*ip).xDD4_itemVar.tincle.x38 *= neg_one();
    } else {
        let half_cycle = (*ip).xDD4_itemVar.tincle.x2C;
        if half_cycle == 0 {
            (*ip).xDD4_itemVar.tincle.x3C = bounce_count - 1.0;
        } else {
            if half_cycle < (*sa).x28 {
                (*ip).x40_vel.y += (*ip).xDD4_itemVar.tincle.x38;
            } else {
                (*ip).x40_vel.y -= (*ip).xDD4_itemVar.tincle.x38;
            }
            (*ip).xDD4_itemVar.tincle.x2C -= 1;
        }
    }

    {
        let timer = (*ip).xDD4_itemVar.tincle.x20;
        if timer != 0 {
            (*ip).xDD4_itemVar.tincle.x20 = timer - 1;
            return;
        }
    }

    {
        let drift_state = (*ip).xDD4_itemVar.tincle.x24;
        if drift_state == 0 {
            if ext::HSD_Randi((*sa).x30) == 0 {
                it_802EC870(gobj, 0);
                return;
            }
            (*ip).x40_vel.x = 0.0;
            (*ip).xDD4_itemVar.tincle.x20 = (*sa).x2C;
            return;
        }
        {
            let drift_counter = (*ip).xDD4_itemVar.tincle.x28;
            if drift_counter >= drift_state {
                (*ip).x40_vel.x = 0.0;
                (*ip).xDD4_itemVar.tincle.x34 = 0.0;
                (*ip).xDD4_itemVar.tincle.x28 = 0;
                (*ip).xDD4_itemVar.tincle.x24 = 0;
                (*ip).xDD4_itemVar.tincle.x20 = (*sa).x2C;
                return;
            }
            if drift_counter == drift_state / 2 {
                (*ip).xDD4_itemVar.tincle.x34 *= neg_one();
            }
            (*ip).x40_vel.x += (*ip).xDD4_itemVar.tincle.x34;
            (*ip).xDD4_itemVar.tincle.x28 += 1;
        }
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion2_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion2_Coll");
    if ext::it_8026DAA8(gobj) != 0 {
        it_802EC870(gobj, 1);
    }
    false
}

pub unsafe extern "C" fn it_802EBD14(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EBD14");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.tincle.x30 = (*ip).msid as f32;
    (*ip).xDD4_itemVar.tincle.x44 = (*ip).x40_vel.y;
    (*ip).xDD4_itemVar.tincle.x48 = (*ip).xDD4_itemVar.tincle.x38;
    (*ip).xDD4_itemVar.tincle.x40 = (*ip).xDD4_itemVar.tincle.x2C as f32;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = -(*ip).xDD4_itemVar.tincle.x54;
    (*ip).xDD4_itemVar.tincle.x38 = (*ip).xDD4_itemVar.tincle.x54 / 10.0;
    (*ip).xDD4_itemVar.tincle.x20 = 0x13;
    (*ip).xDD4_itemVar.tincle.x4C = (*ip).pos.y;
    ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTincle_UnkMotion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion3_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion3_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion3_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.tincle.x20 == 0 {
        (*ip).pos.y = (*ip).xDD4_itemVar.tincle.x4C;
        (*ip).x40_vel.y = 0.0;
        it_802EBE5C(gobj);
    } else {
        (*ip).x40_vel.y += (*ip).xDD4_itemVar.tincle.x38;
        (*ip).xDD4_itemVar.tincle.x20 -= 1;
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion3_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion3_Coll");
    ext::it_8026E414(gobj, Some(it_802EC18C));
    false
}

pub unsafe extern "C" fn it_802EBE5C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EBE5C");
    let ip = GET_ITEM(gobj);
    let data = SA(ip);

    (*ip).xDD4_itemVar.tincle.x30 = (*ip).msid as f32;
    (*ip).xDD4_itemVar.tincle.x44 = (*ip).x40_vel.y;
    (*ip).xDD4_itemVar.tincle.x48 = (*ip).xDD4_itemVar.tincle.x38;
    (*ip).xDD4_itemVar.tincle.x40 = (*ip).xDD4_itemVar.tincle.x2C as f32;

    itResetVelocity(ip);

    (*ip).xDD4_itemVar.tincle.x20 = (*data).x40;
    (*ip).xDD4_itemVar.tincle.x4C = (*ip).pos.y;

    ext::Item_80268E5C(gobj, 4, 2);
}

pub unsafe extern "C" fn itTincle_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion4_Phys");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);
    if (*ip).xDD4_itemVar.tincle.x20 == 0 {
        (*ip).x40_vel.y = -(*sa).x48;
        let mut dist = (*ip).xDD4_itemVar.tincle.x4C - (*ip).pos.y;
        if dist < 0.0 {
            dist = -dist;
        }
        if dist > (*sa).x44 {
            it_802EC18C(gobj);
        }
    } else {
        (*ip).xDD4_itemVar.tincle.x20 -= 1;
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion4_Coll");
    ext::it_8026E414(gobj, Some(it_802EC18C));
    false
}

pub unsafe extern "C" fn it_802EBFAC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EBFAC");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);
    let dist = (*ip).xDD4_itemVar.tincle.x5C - (*ip).pos.y;
    let max_speed = (*sa).x4C;

    if dist > max_speed {
        (*ip).x40_vel.y = max_speed;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.x = 0.0;
        (*ip).xDD4_itemVar.tincle.x2C = melee_core::ppc::fctiwz_f32(dist);
        {
            let vel_y = (*ip).x40_vel.y;
            (*ip).xDD4_itemVar.tincle.x38 = -(vel_y * vel_y) / (vel_y + ((*ip).xDD4_itemVar.tincle.x2C.wrapping_mul(2)) as f32);
        }
        (*ip).xDD4_itemVar.tincle.x20 = 0;
        ext::Item_80268E5C(gobj, 5, ITEM_UNK_0x1);
        return;
    }
    if (*ip).xDD4_itemVar.tincle.x30 == 2.0 {
        (*ip).x40_vel.y = (*ip).xDD4_itemVar.tincle.x44;
        (*ip).x40_vel.z = 0.0;
        (*ip).x40_vel.x = 0.0;
        (*ip).xDD4_itemVar.tincle.x38 = (*ip).xDD4_itemVar.tincle.x48;
        (*ip).xDD4_itemVar.tincle.x2C = melee_core::ppc::fctiwz_f32((*ip).xDD4_itemVar.tincle.x40);
        (*ip).xDD4_itemVar.tincle.x20 = (*sa).x2C;
        (*ip).xDD4_itemVar.tincle.x24 = 0;
        ext::Item_80268E5C(gobj, 2, ITEM_UNK_0x1);
        return;
    }
    (*ip).x40_vel.y = (*ip).xDD4_itemVar.tincle.x44;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.tincle.x38 = (*ip).xDD4_itemVar.tincle.x48;
    (*ip).xDD4_itemVar.tincle.x2C = melee_core::ppc::fctiwz_f32((*ip).xDD4_itemVar.tincle.x40);
    ext::Item_80268E5C(gobj, 5, ITEM_UNK_0x1);
}

pub unsafe extern "C" fn itTincle_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion5_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion5_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion5_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).x40_vel.y <= 0.0 {
        itResetVelocity(ip);
        (*ip).xDD4_itemVar.tincle.x38 = 0.0;
        (*ip).xDD4_itemVar.tincle.x2C = 0;
        (*ip).xDD4_itemVar.tincle.x20 = 0;
        it_802EBA00(gobj);
    } else {
        (*ip).x40_vel.y += (*ip).xDD4_itemVar.tincle.x38;
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion5_Coll");
    ext::it_8026DAA8(gobj);
    false
}

pub unsafe extern "C" fn it_802EC18C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC18C");
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    ext::it_802756D0(gobj);
    (*ip).xDD4_itemVar.tincle.x20 = 3;
    ext::Item_80268E5C(gobj, 6, 0x12);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn it_802EC1F4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC1F4");
    let ip = GET_ITEM(gobj);
    itResetVelocity(ip);
    ext::it_802756D0(gobj);
    (*ip).xDD4_itemVar.tincle.x20 = 0;
    ext::Item_80268E5C(gobj, 7, 0x12);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion7_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion7_Anim");
    let ip = GET_ITEM(gobj);

    if (*ip).xDD4_itemVar.tincle.x20 == 0 {
        (*ip).xDD4_itemVar.tincle.x20 = -1;
        if (*ip).msid == 6 {
            ext::Item_80268E5C(gobj, 7, 1);
        }
    } else if (*ip).xDD4_itemVar.tincle.x20 > 0 {
        (*ip).xDD4_itemVar.tincle.x20 -= 1;
    }

    if ext::it_80272C6C(gobj) == 0 {
        it_802EC35C(gobj);
    }

    false
}

pub unsafe extern "C" fn itTincle_UnkMotion7_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTincle_UnkMotion7_Phys");}

pub unsafe extern "C" fn itTincle_UnkMotion7_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion7_Coll");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air == GA_Air {
        ext::it_8026E414(gobj, Some(it_802EC3F4));
    } else {
        if (*ip).xDD4_itemVar.tincle.x20 < 0 {
            it_802EC3F4(gobj);
        }
        ext::it_8026D6F4(gobj, Some(it_802EC35C));
    }
    false
}

pub unsafe extern "C" fn it_802EC35C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC35C");
    let ip = GET_ITEM(gobj);
    (*ip).pos.z = -10.0;
    ext::Item_80268E5C(gobj, 8, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTincle_UnkMotion8_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion8_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion8_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion8_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itTincle_UnkMotion8_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion8_Coll");
    ext::it_8026E414(gobj, Some(it_802EC3F4));
    false
}

pub unsafe extern "C" fn it_802EC3F4(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC3F4");
    let ip = (*gobj).user_data as *mut Item;
    itResetVelocity(ip);
    (*ip).xDC8_word.flags.set_x1A(0);
    (*ip).x70_nudge.z = 0.0;
    (*ip).x70_nudge.y = 0.0;
    (*ip).x70_nudge.x = 0.0;
    ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion9_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion9_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802EC4D0(gobj);
    }
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion9_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTincle_UnkMotion9_Phys");}

pub unsafe extern "C" fn itTincle_UnkMotion9_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion9_Coll");
    ext::it_8026D62C(gobj, Some(it_802EC35C));
    false
}

pub unsafe extern "C" fn it_802EC4D0(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC4D0");
    let ip = GET_ITEM(gobj);
    let data = SA(ip);

    let mut range = (*data).x55 as i32 - (*data).x54 as i32;
    if range < 0 {
        range = -range;
    }

    (*ip).xDD4_itemVar.tincle.x20 = ext::HSD_Randi(range);
    (*ip).xDD4_itemVar.tincle.x20 += (*data).x54 as i32;

    ext::Item_80268E5C(gobj, 10, 2);
}

pub unsafe extern "C" fn itTincle_UnkMotion10_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion10_Anim");
    let ip = GET_ITEM(gobj);

    if ext::it_80272C6C(gobj) == 0 {
        (*ip).xDD4_itemVar.tincle.x20 -= 1;
        if (*ip).xDD4_itemVar.tincle.x20 != 0 {
            ext::Item_80268E5C(gobj, 10, 2);
        } else {
            it_802EC604(gobj);
        }
    }
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion10_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTincle_UnkMotion10_Phys");}

pub unsafe extern "C" fn itTincle_UnkMotion10_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion10_Coll");
    ext::it_8026D62C(gobj, Some(it_802EC35C));
    false
}

pub unsafe extern "C" fn it_802EC604(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802EC604");
    ext::Item_80268E5C(gobj, 0xB, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itTincle_UnkMotion11_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion11_Anim");
    if ext::it_80272C6C(gobj) == 0 {
        it_802EC69C(gobj);
    }
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion11_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itTincle_UnkMotion11_Phys");}

pub unsafe extern "C" fn itTincle_UnkMotion11_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion11_Coll");
    ext::it_8026D62C(gobj, Some(it_802EC35C));
    false
}

pub unsafe extern "C" fn it_802EC69C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC69C");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);

    (*ip).pos.z = 0.0;
    let mut dist = ABS((*ip).xDD4_itemVar.tincle.x5C - (*ip).pos.y);
    if dist < (*sa).x44 {
        (*ip).xDD4_itemVar.tincle.x5C += 20.0;
        dist += 20.0;
    }
    (*ip).x40_vel.y = (*sa).x50;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDD4_itemVar.tincle.x2C = melee_core::ppc::fctiwz_f32(dist);
    (*ip).xDD4_itemVar.tincle.x38 =
        -((*ip).x40_vel.y * (*ip).x40_vel.y) / ((*ip).x40_vel.y + ((*ip).xDD4_itemVar.tincle.x2C.wrapping_mul(2)) as f32);
    (*ip).xDD4_itemVar.tincle.x20 = 0;
    ext::it_802762BC(ip);
    ext::it_802756E0(gobj);
    ext::Item_80268E5C(gobj, 0xC, ITEM_UNK_0x1);
    it_802EC9E8(gobj);
}

pub unsafe extern "C" fn itTincle_UnkMotion12_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion12_Anim");
    false
}

pub unsafe extern "C" fn itTincle_UnkMotion12_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itTincle_UnkMotion12_Phys");
    let ip = GET_ITEM(gobj);
    if (*ip).x40_vel.y <= 0.0 {
        itResetVelocity(ip);
        (*ip).xDD4_itemVar.tincle.x38 = 0.0;
        (*ip).xDD4_itemVar.tincle.x2C = 0;
        (*ip).xDD4_itemVar.tincle.x20 = 0;
        it_802EBA00(gobj);
    } else {
        (*ip).x40_vel.y += (*ip).xDD4_itemVar.tincle.x38;
    }
}

pub unsafe extern "C" fn itTincle_UnkMotion12_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itTincle_UnkMotion12_Coll");
    ext::it_8026DAA8(gobj);
    false
}

pub unsafe extern "C" fn it_802EC830(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC830");
    ext::Item_8026A8EC(gobj);
}

pub unsafe extern "C" fn it_802EC850(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC850");
    ext::it_8026B894(gobj, ref_gobj);
}

pub unsafe extern "C" fn it_802EC870(gobj: *mut Item_GObj, arg1: i32) {
    melee_core::ct!("it_802EC870");
    let ip = GET_ITEM(gobj);
    let sa = SA(ip);

    if arg1 != 0 {
        (*ip).facing_dir = -(*ip).facing_dir;
        (*ip).x40_vel.x *= neg_one();
        (*ip).xDD4_itemVar.tincle.x34 *= neg_one();
        return;
    }

    (*ip).facing_dir = if (*ip).pos.x < (*ip).xDD4_itemVar.tincle.x58 { 1.0 } else { -1.0 };
    if ext::HSD_Randi(3) == 0 {
        (*ip).facing_dir *= neg_one();
    }
    if (*ip).pos.x < (*sa).xC {
        (*ip).facing_dir = 1.0;
    }
    if (*ip).pos.x > (*sa).x10 {
        (*ip).facing_dir = -1.0;
    }

    {
        let abs = ABS((*sa).x38 - (*sa).x34);
        (*ip).x40_vel.x = abs * ext::HSD_Randf();
    }
    (*ip).xDD4_itemVar.tincle.x24 = (*sa).x3C;
    (*ip).xDD4_itemVar.tincle.x34 = ((*sa).x38 - (*ip).x40_vel.x) / ((*ip).xDD4_itemVar.tincle.x24 / 2) as f32;
    (*ip).x40_vel.x = (*ip).x40_vel.x * (*ip).facing_dir;
    (*ip).xDD4_itemVar.tincle.x34 = (*ip).xDD4_itemVar.tincle.x34 * (*ip).facing_dir;
    (*ip).xDD4_itemVar.tincle.x28 = 0;
}

pub unsafe extern "C" fn it_802EC9E8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802EC9E8");
    let ip = GET_ITEM(gobj);
    let mut ecb: _itECB = (*ip).xDD4_itemVar.tincle.x68;

    match (*ip).msid {
        6 | 7 | 8 | 9 | 10 | 11 => {
            ecb.top *= 0.5;
            ecb.right *= 0.5;
            ecb.left *= 0.5;
        }
        _ => {}
    }

    ext::it_80275D5C(gobj, &mut ecb);
}

pub unsafe extern "C" fn it_802ECA70(gobj: *mut HSD_GObj) -> *mut Item_GObj {
    melee_core::ct!("it_802ECA70");
    let mut pos = Vec3 { x: 0.0, y: 200.0, z: 0.0 };
    let new_gobj = ext::it_8027B5B0(It_Kind_Tincle, &mut pos, null_mut(), null_mut(), 1);

    if !new_gobj.is_null() {
        let ip = GET_ITEM(new_gobj);

        (*ip).scl = ext::Ground_801C0498();
        HSD_JObjSetScaleX((*new_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        HSD_JObjSetScaleY((*new_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        HSD_JObjSetScaleZ((*new_gobj).hsd_obj as *mut HSD_JObj, (*ip).scl);
        (*ip).facing_dir = 0.0;
        (*ip).xDD4_itemVar.tincle.x64 = gobj;
        (*ip).x378_itemColl.joint_id_skip = 5;
        it_802EB6DC(new_gobj);
    }
    new_gobj
}

pub unsafe extern "C" fn it_802ECC8C(arg0: *mut Item_GObj) -> i32 {
    melee_core::ct!("it_802ECC8C");
    (*GET_ITEM(arg0)).xDD4_itemVar.tincle.x20
}

pub unsafe extern "C" fn it_802ECC98(arg0: *mut Item_GObj, arg1: f32) {
    melee_core::ct!("it_802ECC98");
    (*GET_ITEM(arg0)).xDD4_itemVar.tincle.x54 = arg1;
}

pub unsafe extern "C" fn it_802ECCA4(gobj: *mut Item_GObj, kind: *mut i32, pos: *mut Vec3) {
    melee_core::ct!("it_802ECCA4");
    let ip = GET_ITEM(gobj);
    *pos = (*ip).pos;
    (*pos).y += (*ip).xDD4_itemVar.tincle.x50;
    *kind = (*ip).msid;
}

// ---- folded from itfwd.rs (consolidation steps 21/22) ----
/// `it_802EC850` (src/melee/it/kinds/ittincle.c): `it_8026B894(..)`.
pub fn it_802ec850(ip: &mut Item, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802ec850");
    it_8026b894(ip, ref_gobj);
}

// ---- folded from itkinds10.rs (consolidation steps 21/22) ----
// ---- ittincle.c ------------------------------------------------------------------------

/// `it_802EC870`: turn around (`arg1 != 0`), or pick a direction (1 in 3 reversed, clamped to
/// the area `[xC, x10]`) and a random speed. `x24 / 2` is an integer division.
/// # Safety
/// The item's special attributes are an `itTincleAttributes`.
pub unsafe fn it_802ec870(rand: &mut HsdRand, ip: &mut Item, arg1: i32) {
    melee_core::ct!("it_802ec870");
    let sa = &*((*ip.xC4_article_data).x4_specialAttributes as *const itTincleAttributes);
    if arg1 != 0 {
        ip.facing_dir = -ip.facing_dir;
        ip.x40_vel.x *= -1.0;
        ip.xDD4_itemVar.tincle.x34 *= -1.0;
        return;
    }
    ip.facing_dir = if ip.pos.x < ip.xDD4_itemVar.tincle.x58 { 1.0 } else { -1.0 };
    if rand.randi(3) == 0 {
        ip.facing_dir *= -1.0;
    }
    if ip.pos.x < sa.xC {
        ip.facing_dir = 1.0;
    }
    if ip.pos.x > sa.x10 {
        ip.facing_dir = -1.0;
    }
    let d = sa.x38 - sa.x34;
    let abs = if d < 0.0 { -(sa.x38 - sa.x34) } else { sa.x38 - sa.x34 };
    ip.x40_vel.x = abs * rand.randf();
    let t = &mut ip.xDD4_itemVar.tincle;
    t.x24 = sa.x3C;
    t.x34 = (sa.x38 - ip.x40_vel.x) / (t.x24.wrapping_div(2) as f32);
    ip.x40_vel.x *= ip.facing_dir;
    t.x34 *= ip.facing_dir;
    t.x28 = 0;
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `it_802ECC8C` (ittincle.c): `tincle.x20`.
pub fn it_802ecc8c(ip: &Item) -> i32 {
    melee_core::ct!("it_802ecc8c");
    unsafe { ip.xDD4_itemVar.tincle.x20 }
}

/// `it_802ECC98` (ittincle.c): `tincle.x54 = arg1`.
pub fn it_802ecc98(ip: &mut Item, arg1: f32) {
    melee_core::ct!("it_802ecc98");
    ip.xDD4_itemVar.tincle.x54 = arg1;
}

/// `it_802ECCA4` (ittincle.c): `*pos = pos + (0, tincle.x50, 0)`, `*kind = msid`.
pub fn it_802ecca4(ip: &Item, kind: &mut i32, pos: &mut GcVec) {
    melee_core::ct!("it_802ecca4");
    *pos = ip.pos;
    pos.y += unsafe { ip.xDD4_itemVar.tincle.x50 };
    *kind = ip.msid;
}
