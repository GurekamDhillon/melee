//! Port of src/melee/it/kinds/itlgun.c: the ray gun item (ammo, pickup/drop states, firing hook).
//! Callees of unported TUs go through `ext` (unset: panics "TODO"). Checked against the C
//! (whole TU, shipping contraction) by tests/kinds_itlgun.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::it_80275158;
#[allow(unused_imports)]
use crate::item::it_802762bc;
#[allow(unused_imports)]
use crate::itkinds::it_802e657c;
#[allow(unused_imports)]
use crate::itkinds::it_8028b988;
#[allow(unused_imports)]
use crate::itkinds::it_capsule_unk_motion4_phys;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_8026b894;
#[allow(unused_imports)]
use crate::itlib::it_802751d8;
#[allow(unused_imports)]
use crate::itlib::it_802755c0;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_80274658;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
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
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItLGunAttr;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::MaximTomatoSpecialAttr;
#[allow(unused_imports)]
use crate::layout::ScopeBeamAttrs;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::HitCapsule_Enabled;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itkinds5;
#[allow(unused_imports)]
use crate::itkinds5::{sa};
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
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80298168(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32) => unsafe { crate::tu::itlgunray::it_80298168(a0, a1 as _, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `Item_StopAndEnterState` (it/kinds/inlines.h, static inline)
#[inline]
unsafe fn Item_StopAndEnterState(gobj: *mut Item_GObj, ip: *mut Item, msid: i32) {
    itResetVelocity(ip);
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, msid, 2);
}

/// `ItemStateTable it_803F5CC8[]`
pub static mut it_803F5CC8: [ItemStateTable; 6] = [
    st_row(-1, pred_b!(itLgun_UnkMotion0_Anim), Some(itLgun_UnkMotion0_Phys), pred_b!(itLgun_UnkMotion0_Coll)),
    st_row(-1, pred_b!(itLgun_UnkMotion4_Anim), Some(itLgun_UnkMotion1_Phys), Some(itLgun_UnkMotion4_Coll)),
    st_row(-1, pred_b!(itLgun_UnkMotion2_Anim), Some(itLgun_UnkMotion2_Phys), None),
    st_row(0, pred_b!(itLgun_UnkMotion3_Anim), Some(itLgun_UnkMotion3_Phys), None),
    st_row(1, pred_b!(itLgun_UnkMotion4_Anim), Some(itLgun_UnkMotion4_Phys), Some(itLgun_UnkMotion4_Coll)),
    st_row(-1, pred_b!(itLgun_UnkMotion5_Anim), Some(itLgun_UnkMotion5_Phys), pred_b!(itLgun_UnkMotion5_Coll)),
];

pub unsafe extern "C" fn itLGun_Logic16_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLGun_Logic16_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    let item_spec_attr = (*(*ip).xC4_article_data).x4_specialAttributes as *mut ItLGunAttr;
    (*ip).xD4C = (*item_spec_attr).max_ammo;
    (*ip).xDD4_itemVar.lgun.timer = 0;
    it_8028E860(gobj);
}

pub unsafe extern "C" fn it_8028E774(gobj: *mut Item_GObj, arg1: *mut Vec3) {
    melee_core::ct!("it_8028E774");
    let da = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut ItLGunAttr;
    (*arg1).x = (*da).pos.x;
    (*arg1).y = (*da).pos.y;
    (*arg1).z = (*da).pos.z;
}

/// Decrease ammo and fire/spawn lgunray.
pub unsafe extern "C" fn it_8028E79C(gobj: *mut Item_GObj, pos: *mut Vec3, facing_dir: f32) {
    melee_core::ct!("it_8028E79C");
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C > 0 {
        (*ip).xD4C -= 1;
    }
    ext::it_80298168((*ip).owner, pos, facing_dir);
}

pub unsafe extern "C" fn it_8028E7D8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028E7D8");
    let ip = GET_ITEM(gobj);
    Item_StopAndEnterState(gobj, ip, 0);
}

pub unsafe extern "C" fn itLgun_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion0_Anim");
    false
}

pub unsafe extern "C" fn itLgun_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLgun_UnkMotion0_Phys");}

pub unsafe extern "C" fn itLgun_UnkMotion0_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8028E860));
    false
}

pub unsafe extern "C" fn it_8028E860(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028E860");
    ext::Item_80268E5C(gobj, 1, 2);
}

pub unsafe extern "C" fn itLgun_UnkMotion4_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion4_Anim");
    false
}

pub unsafe extern "C" fn itLgun_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLgun_UnkMotion1_Phys");
    let item_comm_attr = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*item_comm_attr).x10_fall_speed, (*item_comm_attr).x14_fall_speed_max);
}

/// Returns the raw `int` of the callee (`bool` is the MSL `int`).
pub unsafe extern "C" fn itLgun_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itLgun_UnkMotion4_Coll");
    if (*GET_ITEM(gobj)).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_8028E7D8));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe extern "C" fn itLGun_Logic16_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itLGun_Logic16_PickedUp");
    ext::Item_80268E5C(gobj, 2, 2);
}

pub unsafe extern "C" fn itLgun_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion2_Anim");
    false
}

pub unsafe extern "C" fn itLgun_UnkMotion2_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itLgun_UnkMotion2_Phys");}

pub unsafe extern "C" fn it_8028E938(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8028E938");
    (*GET_ITEM(gobj)).xDD4_itemVar.lgun.timer = 40;
    ext::Item_80268E5C(gobj, 3, 2);
}

pub unsafe extern "C" fn itLgun_UnkMotion3_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xDD4_itemVar.lgun.timer = (*ip).xDD4_itemVar.lgun.timer.wrapping_sub(1);
    if (*ip).xDD4_itemVar.lgun.timer <= 0 {
        ext::Item_80268E5C(gobj, 2, 2);
    }
    false
}

pub unsafe extern "C" fn itLgun_UnkMotion3_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLgun_UnkMotion3_Phys");}

pub unsafe extern "C" fn itLGun_Logic16_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itLGun_Logic16_Dropped");
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itLGun_Logic16_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itLGun_Logic16_Thrown");
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itLgun_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLgun_UnkMotion4_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itLGun_Logic16_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGun_Logic16_DmgDealt");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itLGun_Logic16_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGun_Logic16_Clanked");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itLGun_Logic16_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLGun_Logic16_HitShield");
    ext::itColl_BounceOffVictim(gobj);
    false
}

pub unsafe extern "C" fn itLGun_Logic16_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLGun_Logic16_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe extern "C" fn itLGun_Logic16_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLGun_Logic16_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

pub unsafe extern "C" fn itLGun_Logic16_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itLGun_Logic16_EnteredAir");
    ext::Item_80268E5C(gobj, 5, 2);
}

pub unsafe extern "C" fn itLgun_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion5_Anim");
    false
}

pub unsafe extern "C" fn itLgun_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLgun_UnkMotion5_Phys");}

pub unsafe extern "C" fn itLgun_UnkMotion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itLgun_UnkMotion5_Coll");
    ext::it_8026E8C4(gobj, Some(it_8028E7D8), Some(it_8028E860));
    false
}

pub unsafe extern "C" fn itLGun_Logic16_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itLGun_Logic16_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itkinds5.rs (consolidation steps 21/22) ----
/// `it_8028E774` (itlgun.c): `*arg1 = sa->pos`.
/// # Safety
/// The special attributes must be a valid `ItLGunAttr`.
pub unsafe fn it_8028e774(ip: &Item, arg1: &mut GcVec) {
    melee_core::ct!("it_8028e774");
    let a: &ItLGunAttr = sa(ip);
    arg1.x = a.pos.x;
    arg1.y = a.pos.y;
    arg1.z = a.pos.z;
}
