//! Native port of src/melee/it/kinds/itheart.c: the Heart Container item (spawn, state callbacks,
//! logic callbacks). Callees in other TUs go through `ext`. Checked against the C (the whole TU,
//! shipping contraction, item/jobj state hashed field by field) by tests/tu_itheart.rs.
//!
//! No float arithmetic of its own (copies and `HSD_JObjAddRotationY`'s `+=`: no fused sites).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
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
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_80273030;
#[allow(unused_imports)]
use crate::itlib::it_80273130;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_802756d0;
#[allow(unused_imports)]
use crate::itlib::it_8027572c;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::itlib3::it_8027a344;
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
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemAttr;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
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
use crate::itkinds::{sa};
use core::ffi::c_void;
use core::ptr::null_mut;
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) => unsafe { crate::tu::jobj::HSD_JObjClearFlagsAll(a0, a1) };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) => unsafe { crate::tu::jobj::HSD_JObjSetMtxDirtySub(a0) };
    fn Item_80267454(a0: *mut HSD_GObj) => unsafe { crate::tu::item::Item_80267454(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B390(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::shared::gm_80473A18;
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_jobj_set!();
crate::inl_itResetVelocity!();
crate::inl_Item_InitSpawnPosition!();
crate::inl_Item_InitSpawnCommonFields!();

/// `ItemStateTable it_803F56C8[5]`
pub static it_803F56C8: [ItemStateTable; 5] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeart_UnkMotion0_Anim), physics_updated: Some(itHeart_UnkMotion0_Phys), collided: Some(itHeart_UnkMotion0_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeart_UnkMotion3_Anim), physics_updated: Some(itHeart_UnkMotion3_Phys), collided: Some(itHeart_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeart_UnkMotion2_Anim), physics_updated: Some(itHeart_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeart_UnkMotion3_Anim), physics_updated: Some(itHeart_UnkMotion3_Phys), collided: Some(itHeart_UnkMotion3_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHeart_UnkMotion4_Anim), physics_updated: Some(itHeart_UnkMotion4_Phys), collided: Some(itHeart_UnkMotion4_Coll) },
];

/// `Item_GObj* it_80283AE4(Item_GObj*, Vec3*, s32)`: spawn a heart at `pos`.
pub unsafe extern "C" fn it_80283AE4(gobj: *mut HSD_GObj, pos: *mut Vec3, arg2: i32) -> *mut HSD_GObj {
    melee_core::ct!("it_80283AE4");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut item_gobj: *mut HSD_GObj = null_mut();
    if !gobj.is_null() {
        spawn.kind = It_Kind_Heart as _;
        Item_InitSpawnPosition(&raw mut spawn, pos, true);
        Item_InitSpawnCommonFields(&raw mut spawn, null_mut(), -1.0, true);
        item_gobj = ext::Item_80268B18(&raw mut spawn);
    }
    if !item_gobj.is_null() {
        let ip = GET_ITEM(item_gobj);
        (*ip).xDD4_itemVar.heart.xDD8.anon_0.set_b0(1);
        (*ip).xDD4_itemVar.heart.xDDC = arg2;
        ext::Item_80267454(item_gobj);
    }
    item_gobj
}

/// `void it_80283BD4(Item_GObj*)`
pub unsafe extern "C" fn it_80283BD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80283BD4");
    let ip = GET_ITEM(gobj);
    let vars = (*(*ip).xC4_article_data).x4_specialAttributes as *mut HeartContainerAttr;
    (*ip).xDD4_itemVar.heart.xDD4_heal = (*vars).x4.flags as i32;
}

pub unsafe extern "C" fn itHeart_Logic8_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_Spawned");
    let ip = GET_ITEM(gobj);
    let vars = (*(*ip).xC4_article_data).x4_specialAttributes as *mut HeartContainerAttr;
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = (*vars).x14;
    (*ip).x40_vel.z = 0.0;
    (*ip).xDD4_itemVar.heart.xDD4_heal = (*vars).x0_heal;
    (*ip).xDD4_itemVar.heart.xDD8.anon_0.set_b0(0);
    (*ip).xDD4_itemVar.heart.xDDC = 0;
    it_80283DD4(gobj);
}

pub unsafe extern "C" fn itHeart_Logic8_Destroyed(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_Destroyed");
    let ip = GET_ITEM(gobj);
    if (*ip).xDD4_itemVar.heart.xDD8.anon_0.b0() != 0 {
        ext::gm_80473A18.x90[(*ip).xDD4_itemVar.heart.xDDC as usize] = 0;
    }
}

pub unsafe extern "C" fn it_80283C7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80283C7C");
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHeart_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion0_Anim");
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    let attrs = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut HeartContainerAttr;
    HSD_JObjAddRotationY(child, (*attrs).x18);
    0
}

pub unsafe extern "C" fn itHeart_UnkMotion0_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHeart_UnkMotion0_Phys");}

pub unsafe extern "C" fn itHeart_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_80283DD4));
    0
}

pub unsafe extern "C" fn it_80283DD4(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80283DD4");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHeart_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion3_Anim");
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    let attrs = (*(*GET_ITEM(gobj)).xC4_article_data).x4_specialAttributes as *mut HeartContainerAttr;
    HSD_JObjAddRotationY(child, (*attrs).x18);
    0
}

pub unsafe extern "C" fn itHeart_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_UnkMotion3_Phys");
    let item = GET_ITEM(gobj);
    let attr = (*item).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itHeart_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion3_Coll");
    ext::it_8026E15C(gobj, Some(it_80283C7C));
    0
}

pub unsafe extern "C" fn itHeart_Logic8_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_PickedUp");
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    HSD_JObjSetRotationY(child, 0.0);
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHeart_UnkMotion2_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion2_Anim");
    0
}

pub unsafe extern "C" fn itHeart_UnkMotion2_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHeart_UnkMotion2_Phys");}

pub unsafe extern "C" fn itHeart_Logic8_Dropped(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_Dropped");
    let child = HSD_JObjGetChild(GET_JOBJ(gobj));
    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(child, JOBJ_HIDDEN as u32);
    HSD_JObjSetRotationY(child, 0.0);
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itHeart_Logic8_EnteredAir(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_EnteredAir");
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHeart_UnkMotion4_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion4_Anim");
    0
}

pub unsafe extern "C" fn itHeart_UnkMotion4_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHeart_UnkMotion4_Phys");}

pub unsafe extern "C" fn itHeart_UnkMotion4_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHeart_UnkMotion4_Coll");
    ext::it_8026E8C4(gobj, Some(it_80283C7C), Some(it_80283DD4));
    0
}

pub unsafe extern "C" fn itHeart_Logic8_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itHeart_Logic8_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}

// ---- folded from itkinds.rs (consolidation steps 21/22) ----
/// `it_80283BD4` (itheart.c): `heart.xDD4_heal = attrs.x4.flags`.
/// # Safety
/// The special attributes must be a valid `HeartContainerAttr`.
pub unsafe fn it_80283bd4(ip: &mut Item) {
    melee_core::ct!("it_80283bd4");
    ip.xDD4_itemVar.heart.xDD4_heal = sa::<HeartContainerAttr>(ip).x4.flags as _;
}
