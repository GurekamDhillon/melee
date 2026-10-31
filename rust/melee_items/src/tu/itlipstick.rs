//! Native port of src/melee/it/kinds/itlipstick.c: the Lipstick item (spawn timer from the article's special
//! attributes, the six states, the spore hand-off, bounce/clank/reflect logic callbacks). Callees in other TUs
//! go through `ext` (hooks; unset panics "TODO"). Checked against the C (the whole TU, shipping contraction)
//! by tests/tu_itlipstick.rs. No float arithmetic.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, clippy::missing_safety_doc)]

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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
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
    fn it_8026DAA8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DAA8(a0) };
    fn it_8026DBC8(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DBC8(a0) };
    fn it_8026DF34(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itgroundcoll::it_8026DF34(a0) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80276FC4(a0: *mut HSD_GObj, a1: i32) => unsafe { crate::tu::itmaplib::it_80276FC4(a0, a1) };
    fn it_8029A114(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32, a3: i32) => unsafe { crate::tu::itlipstickspore::it_8029A114(a0 as _, a1 as _, a2, a3) };
    fn it_8029A218(a0: *mut HSD_GObj, a1: *mut Vec, a2: f32, a3: i32) => unsafe { crate::tu::itlipstickspore::it_8029A218(a0, a1 as _, a2, a3) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

crate::inl_it_core!();
crate::inl_it_fall!();

#[inline(always)]
unsafe fn GET_ATTRS(ip: *mut Item) -> *mut itLipstickAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itLipstickAttributes
}

pub static mut it_803F6310: [ItemStateTable; 6] = [
    ItemStateTable { anim_id: 0, animated: Some(itLipstick_UnkMotion0_Anim), physics_updated: Some(itLipstick_UnkMotion0_Phys), collided: Some(itLipstick_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itLipstick_UnkMotion4_Anim), physics_updated: Some(itLipstick_UnkMotion1_Phys), collided: Some(itLipstick_UnkMotion1_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itLipstick_UnkMotion2_Anim), physics_updated: Some(itLipstick_UnkMotion2_Phys), collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itLipstick_UnkMotion4_Anim), physics_updated: Some(itLipstick_UnkMotion4_Phys), collided: Some(itLipstick_UnkMotion3_Coll) },
    ItemStateTable { anim_id: 1, animated: Some(itLipstick_UnkMotion4_Anim), physics_updated: Some(itLipstick_UnkMotion4_Phys), collided: Some(itLipstick_UnkMotion4_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itLipstick_UnkMotion5_Anim), physics_updated: Some(itLipstick_UnkMotion5_Phys), collided: Some(itLipstick_UnkMotion5_Coll) },
];

pub unsafe fn itLipstick_Logic23_Spawned(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_Spawned");
    let ip = (*gobj).user_data as *mut Item;
    (*ip).xD4C = (*GET_ATTRS(ip)).x0;
    (*ip).xDAC_itcmd_var0 = 0;
    it_8029583C(gobj);
}

pub unsafe fn it_80295748(gobj: *mut Item_GObj, pos: *mut Vec) {
    melee_core::ct!("it_80295748");
    let ip = GET_ITEM(gobj);
    *pos = (*GET_ATTRS(ip)).x4;
}

pub unsafe fn it_80295770(gobj: *mut Item_GObj, arg1: *mut Vec, arg2: f32, arg3: i32) {
    melee_core::ct!("it_80295770");
    let ip = GET_ITEM(gobj);
    if (*ip).xD4C > 0 {
        (*ip).xD4C -= 1;
    }
    ext::it_8029A114((*ip).owner, arg1, arg2, arg3);
}

pub unsafe extern "C" fn it_802957AC(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802957AC");
    let ip = GET_ITEM(gobj);
    ext::it_8026B390(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLipstick_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion0_Anim");
    0
}

pub unsafe extern "C" fn itLipstick_UnkMotion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLipstick_UnkMotion0_Phys");}

pub unsafe extern "C" fn itLipstick_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion0_Coll");
    ext::it_8026D62C(gobj, Some(it_8029583C));
    0
}

pub unsafe extern "C" fn it_8029583C(gobj: *mut Item_GObj) {
    melee_core::ct!("it_8029583C");
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLipstick_UnkMotion4_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion4_Anim");
    0
}

pub unsafe extern "C" fn itLipstick_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_UnkMotion1_Phys");
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itLipstick_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion1_Coll");
    ext::it_8026E15C(gobj, Some(it_802957AC));
    0
}

pub unsafe fn itLipstick_Logic23_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_PickedUp");
    ext::Item_80268E5C(gobj, 2, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLipstick_UnkMotion2_Anim(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion2_Anim");
    (*GET_ITEM(gobj)).xDAC_itcmd_var0 = 1;
    0
}

pub unsafe extern "C" fn itLipstick_UnkMotion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLipstick_UnkMotion2_Phys");}

pub unsafe fn itLipstick_Logic23_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_Dropped");
    ext::Item_80268E5C(gobj, 4, 6);
}

pub unsafe extern "C" fn itLipstick_UnkMotion4_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion4_Coll");
    let ip = GET_ITEM(gobj);

    if (*ip).xD4C != 0 {
        ext::it_8026E15C(gobj, Some(it_802957AC));
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe fn itLipstick_Logic23_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_Thrown");
    ext::Item_80268E5C(gobj, 3, 6);
}

pub unsafe extern "C" fn itLipstick_UnkMotion4_Phys(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_UnkMotion4_Phys");
    Item_ApplyFallingPhysics(gobj);
}

pub unsafe extern "C" fn itLipstick_UnkMotion3_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion3_Coll");
    let mut ip = GET_ITEM(gobj);

    if (*ip).xD4C != 0 {
        let result = ext::it_8026DAA8(gobj);
        if (result & 0xF) != 0 {
            if (*ip).xDAC_itcmd_var0 != 0 {
                ext::it_8029A218((*ip).owner, &raw mut (*ip).pos, (*ip).facing_dir, 0);
                (*ip).xDAC_itcmd_var0 = 0;
            }

            ext::it_80276FC4(gobj, result);

            if (result & 1) != 0 {
                if ext::it_8026DBC8(gobj) != 0 {
                    ip = GET_ITEM(gobj);
                    ext::it_8026B390(gobj);
                    itResetVelocity(ip);
                    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE as u32);
                }
            }
        }
        return 0;
    }
    ext::it_8026DF34(gobj)
}

pub unsafe fn itLipstick_Logic23_DmgDealt(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_Logic23_DmgDealt");
    let ip = GET_ITEM(gobj);
    if (*ip).msid == 3 || (*ip).msid == 4 {
        ext::itColl_BounceOffVictim(gobj);
    }
    0
}

pub unsafe fn itLipstick_Logic23_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_EnteredAir");
    ext::Item_80268E5C(gobj, 5, ITEM_ANIM_UPDATE as u32);
}

pub unsafe extern "C" fn itLipstick_UnkMotion5_Anim(_gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion5_Anim");
    0
}

pub unsafe extern "C" fn itLipstick_UnkMotion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itLipstick_UnkMotion5_Phys");}

pub unsafe extern "C" fn itLipstick_UnkMotion5_Coll(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_UnkMotion5_Coll");
    ext::it_8026E8C4(gobj, Some(it_802957AC), Some(it_8029583C));
    0
}

pub unsafe fn itLipstick_Logic23_Clanked(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_Logic23_Clanked");
    itLipstick_Logic23_DmgDealt(gobj)
}

pub unsafe fn itLipstick_Logic23_Reflected(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_Logic23_Reflected");
    ext::it_80273030(gobj)
}

pub unsafe fn itLipstick_Logic23_HitShield(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_Logic23_HitShield");
    itLipstick_Logic23_DmgDealt(gobj)
}

pub unsafe fn itLipstick_Logic23_ShieldBounced(gobj: *mut Item_GObj) -> i32 {
    melee_core::ct!("itLipstick_Logic23_ShieldBounced");
    ext::itColl_BounceOffShield(gobj)
}

pub unsafe fn itLipstick_Logic23_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itLipstick_Logic23_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
