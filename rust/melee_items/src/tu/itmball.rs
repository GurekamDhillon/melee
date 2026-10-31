//! Port of src/melee/it/kinds/itmball.c: the Monster Ball (logic table 2). `mball.b0` is "already opened
//! once", `b1` "the Pokemon was released". Callees of unported TUs go through `ext` (unset: panics
//! "TODO"); `efAsync_Spawn` is a C varargs function called with its five fixed arguments only. Checked
//! against the C (whole TU, shipping contraction) by tests/kinds_itmball.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

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
                    None => { $( return $fb; )? panic!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8) => unsafe { crate::tu::item::Item_8026AE84(a0, a1, a2, a3) };
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_2725::itColl_BounceOffShield(a0) as _ };
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::itColl_BounceOffVictim(a0) };
    fn it_8026B390(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b390(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B3A8(a0: *mut HSD_GObj) => unsafe { crate::itlib::it_8026b3a8(&mut *((*a0).user_data as *mut Item)) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026D62C(a0, a1) };
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E15C(a0, a1) };
    fn it_8026E414(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E414(a0, a1) };
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>) => unsafe { crate::tu::itgroundcoll::it_8026E8C4(a0, a1, a2) };
    fn it_802725D4(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_802725D4(a0) };
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32) => unsafe { crate::itlib3::it_80272860(&mut *((*a0).user_data as *mut Item), a1, a2) };
    fn it_80272980(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80272980(a0) };
    fn it_80273030(a0: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_80273030(&mut *((*a0).user_data as *mut Item)) as _ };
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32 => unsafe { crate::itlib3::it_80274658(&mut *((*a0).user_data as *mut Item), a1) };
    fn it_80274740(a0: *mut HSD_GObj) => unsafe { crate::tu::it_2725::it_80274740(a0) };
    fn it_80275D5C(a0: *mut HSD_GObj, a1: *mut _itECB) => unsafe { crate::tu::itmaplib::it_80275D5C(a0, a1 as _) };
    fn it_8027AB64(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::it_279C::it_8027AB64(a0) as _ };
    fn mpCollSetFacingDir(a0: *mut CollData, a1: i32) => unsafe { melee_world::tu::mpcoll::mpCollSetFacingDir(a0, a1) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::item::Item_804A0C64;
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `ItemStateTable it_803F6488[]`
pub static mut it_803F6488: [ItemStateTable; 7] = [
    st_row(-1, pred_b(itMball_Motion0_Anim), Some(itMball_Motion0_Phys), pred_b(itMball_Motion0_Coll)),
    st_row(-1, pred_b(itMball_Motion3_Anim), Some(itMball_Motion1_Phys), pred_b(itMball_Motion1_Coll)),
    st_row(-1, pred_b(itMball_Motion2_Anim), Some(itMball_Motion2_Phys), None),
    st_row(0, pred_b(itMball_Motion3_Anim), Some(itMball_Motion3_Phys), pred_b(itMball_Motion3_Coll)),
    st_row(-1, pred_b(itMball_Motion4_Anim), Some(itMball_Motion4_Phys), pred_b(itMball_Motion4_Coll)),
    st_row(1, pred_b(itMball_Motion5_Anim), Some(itMball_Motion5_Phys), pred_b(itMball_Motion5_Coll)),
    st_row(1, pred_b(itMball_Motion6_Anim), Some(itMball_Motion6_Phys), pred_b(itMball_Motion6_Coll)),
];

/// `Item_TickLifetime` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_TickLifetime(ip: *mut Item) -> bool {
    if (*ip).xD44_lifeTimer <= 0.0 {
        return true;
    }
    (*ip).xD44_lifeTimer -= 1.0;
    false
}

/// `Item_ApplyFallingPhysics` (it/inlines.h, static inline)
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut Item_GObj) {
    let attrs = (*GET_ITEM(gobj)).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

pub unsafe extern "C" fn itMball_Destroyed(item_gobj: *mut Item_GObj) {
    melee_core::ct!("itMball_Destroyed");
    let it = GET_ITEM(item_gobj);
    if (*it).xDD4_itemVar.mball.b1() == 0 {
        ext::Item_804A0C64.x1C = ext::Item_804A0C64.x1C.wrapping_sub(1);
    }
}

pub unsafe extern "C" fn itMball_Spawned(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Spawned");
    let item = GET_ITEM(arg0);

    (*item).xDD4_itemVar.mball.set_b0(0);
    (*item).xDD4_itemVar.mball.set_b1(0);
    itMball_802979D4(arg0);
}

pub unsafe extern "C" fn itMball_80297944(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_80297944");
    let item = GET_ITEM(arg0);
    let jobj = GET_JOBJ(arg0);
    ext::it_8026B390(arg0);
    itResetVelocity(item);
    ext::Item_80268E5C(arg0, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMball_Motion0_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion0_Anim");
    false
}

pub unsafe extern "C" fn itMball_Motion0_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMball_Motion0_Phys");}

pub unsafe extern "C" fn itMball_Motion0_Coll(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion0_Coll");
    ext::it_8026D62C(arg0, Some(itMball_802979D4));
    false
}

pub unsafe extern "C" fn itMball_802979D4(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_802979D4");
    let item = GET_ITEM(arg0);

    if (*item).xDD4_itemVar.mball.b0() == 0 {
        ext::Item_80268E5C(arg0, 1, ITEM_UNK_0x1);
        return;
    }
    ext::Item_80268E5C(arg0, 1, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMball_Motion3_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion3_Anim");
    false
}

pub unsafe extern "C" fn itMball_Motion1_Phys(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Motion1_Phys");
    let item = GET_ITEM(arg0);
    ext::it_80272860(arg0, (*(*item).xCC_item_attr).x10_fall_speed, (*(*item).xCC_item_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMball_Motion1_Coll(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion1_Coll");
    let item = GET_ITEM(arg0);

    let previous_vel_x = (*item).x40_vel.x;
    ext::it_8026E15C(arg0, Some(itMball_80297944));
    if (*item).x40_vel.x != previous_vel_x {
        ext::it_80272980(arg0);
    }
    false
}

pub unsafe extern "C" fn itMball_PickedUp(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_PickedUp");
    ext::Item_80268E5C(arg0, 2, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn itMball_Motion2_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion2_Anim");
    false
}

pub unsafe extern "C" fn itMball_Motion2_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMball_Motion2_Phys");}

pub unsafe extern "C" fn itMball_Dropped(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Dropped");
    ext::Item_80268E5C(arg0, 3, 6);
}

pub unsafe extern "C" fn itMball_Thrown(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Thrown");
    ext::Item_80268E5C(arg0, 3, 6);
}

pub unsafe extern "C" fn itMball_Motion3_Phys(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Motion3_Phys");
    Item_ApplyFallingPhysics(arg0);
}

pub unsafe extern "C" fn itMball_Motion3_Coll(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion3_Coll");
    ext::it_8026E414(arg0, Some(itMball_80297CC4));
    false
}

pub unsafe extern "C" fn itMball_DmgDealt(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_DmgDealt");
    ext::it_8026B3A8(arg0);
    ext::it_802725D4(arg0);
    ext::itColl_BounceOffVictim(arg0);
    false
}

pub unsafe extern "C" fn itMball_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itMball_EnteredAir");
    let ip = GET_ITEM(gobj);
    ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    (*ip).facing_dir = if (*ip).x40_vel.x + (*ip).x7C.x + (*ip).x88.x >= 0.0 { 1.0 } else { -1.0 };
    ext::mpCollSetFacingDir(&mut (*ip).x378_itemColl, if -1.0 == (*ip).facing_dir { -1 } else { 1 });
}

pub unsafe extern "C" fn itMball_Motion4_Anim(_gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion4_Anim");
    false
}

pub unsafe extern "C" fn itMball_Motion4_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMball_Motion4_Phys");}

pub unsafe extern "C" fn itMball_Motion4_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion4_Coll");
    ext::it_8026E8C4(gobj, Some(itMball_80297944), Some(itMball_802979D4));
    false
}

pub unsafe extern "C" fn itMball_80297CC4(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_80297CC4");
    let zero: f32 = 0.0;
    let item = GET_ITEM(arg0);
    let spec_attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itMBallAttributes;
    let hsd_obj = (*arg0).hsd_obj as *mut HSD_JObj;

    ext::it_8026B3A8(arg0);
    ext::it_80275D5C(arg0, &mut (*item).xC0C);
    (*item).xDC8_word.flags.set_x1A(0);
    (*item).xD5C = 0;
    (*item).x40_vel.z = zero;
    (*item).x40_vel.y = zero;
    (*item).x40_vel.x = zero;
    ext::it_80274740(arg0);
    if (*item).xDD4_itemVar.mball.b0() == 0 {
        (*item).xD44_lifeTimer = (*spec_attrs).x0;
        (*item).xDD4_itemVar.mball.set_b0(1);
        ext::efAsync_Spawn(arg0, &mut (*GET_ITEM(arg0)).xBC0 as *mut _ as *mut c_void, 0, 0x449, hsd_obj);
        ext::Item_8026AE84(item, 0x10C, 0x7F, 0x40);
        ext::Item_80268E5C(arg0, 5, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(arg0, 5, ITEM_UNK_0x1);
    }
    (*item).on_accessory = Some(itMball_OnAccessory);
}

pub unsafe extern "C" fn itMball_OnAccessory(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_OnAccessory");
    let item = GET_ITEM(arg0);
    let spec_attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itMBallAttributes;

    if (*item).xD44_lifeTimer == (*spec_attrs).x4 {
        (*item).xDD4_itemVar.mball.set_b1(ext::it_8027AB64(arg0) as u8);
    }
}

pub unsafe extern "C" fn itMball_Motion5_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion5_Anim");
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip)
}

pub unsafe extern "C" fn itMball_Motion5_Phys(_gobj: *mut Item_GObj) { melee_core::ct!("itMball_Motion5_Phys");}

pub unsafe extern "C" fn itMball_Motion5_Coll(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion5_Coll");
    ext::it_8026D62C(gobj, Some(itMball_80297E8C));
    false
}

pub unsafe extern "C" fn itMball_80297E8C(gobj: *mut Item_GObj) {
    melee_core::ct!("itMball_80297E8C");
    let item = GET_ITEM(gobj);
    let spec_attrs = (*(*item).xC4_article_data).x4_specialAttributes as *mut itMBallAttributes;
    let jobj = GET_JOBJ(gobj);

    ext::it_8026B3A8(gobj);
    (*item).xDC8_word.flags.set_x1A(0);
    (*item).xD5C = 0;
    itResetVelocity(item);
    ext::it_80274740(gobj);
    if (*item).xDD4_itemVar.mball.b0() == 0 {
        (*item).xD44_lifeTimer = (*spec_attrs).x0;
        (*item).xDD4_itemVar.mball.set_b0(1);
        ext::efAsync_Spawn(gobj, &mut (*GET_ITEM(gobj)).xBC0 as *mut _ as *mut c_void, 0, 1097, jobj);
        ext::Item_8026AE84(item, 0x10C, 0x7F, 0x40);
        ext::Item_80268E5C(gobj, 6, ITEM_ANIM_UPDATE);
    } else {
        ext::Item_80268E5C(gobj, 6, ITEM_UNK_0x1);
    }
    (*item).on_accessory = Some(itMball_OnAccessory);
}

pub unsafe extern "C" fn itMball_Motion6_Anim(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion6_Anim");
    let ip = GET_ITEM(gobj);
    Item_TickLifetime(ip)
}

pub unsafe extern "C" fn itMball_Motion6_Phys(arg0: *mut Item_GObj) {
    melee_core::ct!("itMball_Motion6_Phys");
    let item = GET_ITEM(arg0);

    ext::it_80272860(arg0, (*(*item).xCC_item_attr).x10_fall_speed, (*(*item).xCC_item_attr).x14_fall_speed_max);
}

pub unsafe extern "C" fn itMball_Motion6_Coll(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Motion6_Coll");
    let item = GET_ITEM(arg0);

    let previous_vel_x = (*item).x40_vel.x;
    ext::it_8026E15C(arg0, Some(itMball_80297CC4));
    if (*item).x40_vel.x != previous_vel_x {
        ext::it_80272980(arg0);
    }
    false
}

pub unsafe extern "C" fn itMball_Clanked(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Clanked");
    ext::it_8026B3A8(arg0);
    ext::it_802725D4(arg0);
    ext::itColl_BounceOffVictim(arg0);
    false
}

pub unsafe extern "C" fn itMball_Reflected(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_Reflected");
    ext::it_8026B3A8(arg0);
    ext::it_80273030(arg0) != 0
}

pub unsafe extern "C" fn itMball_HitShield(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_HitShield");
    ext::it_8026B3A8(arg0);
    ext::it_802725D4(arg0);
    ext::itColl_BounceOffVictim(arg0);
    false
}

pub unsafe extern "C" fn itMball_ShieldBounced(arg0: *mut Item_GObj) -> bool {
    melee_core::ct!("itMball_ShieldBounced");
    ext::it_8026B3A8(arg0);
    ext::itColl_BounceOffShield(arg0) != 0
}

pub unsafe extern "C" fn itMball_EvtUnk(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    melee_core::ct!("itMball_EvtUnk");
    ext::it_8026B894(gobj, ref_gobj);
}
