//! Native port of src/melee/it/kinds/ithammerhead.c: the Hammer Head (Hammer item's thrown head):
//! spawn, state callbacks, logic callbacks. Callees in other TUs go through `ext`. Checked against
//! the C (the whole TU, shipping contraction, item/jobj state hashed field by field) by
//! tests/tu_ithammerhead.rs.
//!
//! One fused site: `velocity->y * initial_velocity + 0.5f` (fmadds).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn db_80225DD8(a0: *mut HSD_GObj, a1: *mut HSD_GObj) => unsafe { crate::tu::dbitem::db_80225DD8(a0, a1) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::tu::it_26B1::it_8026B390(a0) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_26B1::it_8026B894(a0, a1) as _ };
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec) => unsafe { crate::tu::it_26B1::it_8026BB68(a0, a1 as _) };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_802728C8(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802728C8(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80275158(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::it_2725::it_80275158(a0, a1) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::tu::itmaplib::it_802762BC(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

crate::jobj_inlines_it!();
crate::inl_Item_InitSpawnCommonFields!();
crate::inl_Item_InitSpawnPositionFromParent!();

/// `ItemStateTable it_803F6640[4]`
pub static it_803F6640: [ItemStateTable; 4] = [
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammerhead_UnkMotion2_Anim), physics_updated: Some(itHammerhead_UnkMotion2_Phys), collided: Some(itHammerhead_UnkMotion2_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammerhead_UnkMotion1_Anim), physics_updated: Some(itHammerhead_UnkMotion1_Phys), collided: None },
    ItemStateTable { anim_id: 0, animated: Some(itHammerhead_UnkMotion2_Anim), physics_updated: Some(itHammerhead_UnkMotion2_Phys), collided: Some(itHammerhead_UnkMotion2_Coll) },
    ItemStateTable { anim_id: -1i32 as _, animated: Some(itHammerhead_UnkMotion3_Anim), physics_updated: Some(itHammerhead_UnkMotion3_Phys), collided: Some(itHammerhead_UnkMotion3_Coll) },
];

/// `void it_80299C48(Item_GObj* parent, Vec3* pos, Vec3* velocity, f32 facing_dir)`: throw a head.
pub unsafe extern "C" fn it_80299C48(parent_gobj: *mut HSD_GObj, pos: *mut Vec3, velocity: *mut Vec3, facing_dir: f32) {
    melee_core::ct!("it_80299C48");
    let mut spawn: SpawnItem = core::mem::zeroed();
    spawn.kind = It_Kind_Hammer_Head as _;
    Item_InitSpawnPositionFromParent(&raw mut spawn, parent_gobj, pos);
    Item_InitSpawnCommonFields(&raw mut spawn, parent_gobj, facing_dir, true);
    let gobj = ext::Item_80268B18(&raw mut spawn);
    if !gobj.is_null() {
        let it = GET_ITEM(gobj);
        let attrs = (*(*it).xC4_article_data).x4_specialAttributes as *mut itHammerheadAttributes;
        it_80299D7C(gobj);
        (*it).facing_dir = facing_dir;
        (*it).x40_vel.x = (*velocity).x * (*attrs).initial_velocity * (*it).facing_dir;
        (*it).x40_vel.y = fmadds((*velocity).y, (*attrs).initial_velocity, 0.5);
        ext::db_80225DD8(gobj, parent_gobj);
    }
}

pub unsafe extern "C" fn itHammerHead_Logic40_Spawned(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerHead_Logic40_Spawned");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHammerheadAttributes;
    ext::it_80275158(gobj, (*attrs).x4);
}

pub unsafe extern "C" fn it_80299D7C(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80299D7C");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air != GA_Air as _ {
        ext::it_802762BC(ip);
    }
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 0, (ITEM_UNK_0x1 | ITEM_DROP_UPDATE) as u32);
}

pub unsafe extern "C" fn itHammerHead_Logic40_PickedUp(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerHead_Logic40_PickedUp");
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itHammerheadAttributes;
    ext::it_80275158(gobj, (*attrs).x4);
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itHammerhead_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerhead_UnkMotion1_Anim");
    0
}

pub unsafe extern "C" fn itHammerhead_UnkMotion1_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHammerhead_UnkMotion1_Phys");}

pub unsafe extern "C" fn itHammerHead_Logic40_Dropped(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerHead_Logic40_Dropped");
    itHammerHead_Logic40_Thrown(gobj);
}

pub unsafe extern "C" fn itHammerHead_Logic40_Thrown(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerHead_Logic40_Thrown");
    let ip = GET_ITEM(gobj);
    if (*ip).ground_or_air != GA_Air as _ {
        ext::it_802762BC(ip);
    }
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 2, (ITEM_ANIM_UPDATE | ITEM_DROP_UPDATE) as u32);
}

pub unsafe extern "C" fn itHammerhead_UnkMotion2_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerhead_UnkMotion2_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
        ext::it_802728C8(gobj);
    }
    0
}

pub unsafe extern "C" fn itHammerhead_UnkMotion2_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerhead_UnkMotion2_Phys");
    let ip = GET_ITEM(gobj);
    let attr = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itHammerhead_UnkMotion2_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerhead_UnkMotion2_Coll");
    ext::it_8026E15C(gobj, Some(it_80299F94));
    0
}

pub unsafe extern "C" fn it_80299F94(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80299F94");
    it_80299FB4(gobj);
}

/// `static inline Item* it_80299FB4_get(Item_GObj*)`
pub unsafe extern "C" fn it_80299FB4_get(gobj: *mut HSD_GObj) -> *mut Item {
    melee_core::ct!("it_80299FB4_get");
    GET_ITEM(gobj)
}

pub unsafe extern "C" fn it_80299FB4(gobj: *mut HSD_GObj) {
    melee_core::ct!("it_80299FB4");
    let ip = it_80299FB4_get(gobj);
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_8026B390(gobj);
    ext::Item_80268E5C(gobj, 3, 1);
}

pub unsafe extern "C" fn itHammerhead_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerhead_UnkMotion3_Anim");
    let ip = GET_ITEM(gobj);
    (*ip).xD44_lifeTimer -= 1.0;
    if (*ip).xD44_lifeTimer <= 0.0 {
        return 1;
    }
    if (*ip).xD44_lifeTimer <= (*ext::it_804D6D28).x34 as f32 {
        ext::it_802728C8(gobj);
    }
    0
}

pub unsafe extern "C" fn itHammerhead_UnkMotion3_Phys(_gobj: *mut HSD_GObj) { melee_core::ct!("itHammerhead_UnkMotion3_Phys");}

pub unsafe extern "C" fn itHammerhead_UnkMotion3_Coll(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerhead_UnkMotion3_Coll");
    ext::it_8026D62C(gobj, Some(itHammerHead_Logic40_Thrown));
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_DmgDealt(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerHead_Logic40_DmgDealt");
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_Clanked(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerHead_Logic40_Clanked");
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_HitShield(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerHead_Logic40_HitShield");
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_Reflected(gobj: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerHead_Logic40_Reflected");
    ext::it_80273030(gobj);
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_DmgReceived(_arg0: *mut HSD_GObj) -> i32 {
    melee_core::ct!("itHammerHead_Logic40_DmgReceived");
    0
}

pub unsafe extern "C" fn itHammerHead_Logic40_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("itHammerHead_Logic40_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
