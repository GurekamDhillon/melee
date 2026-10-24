//! Native port of src/melee/it/kinds/itkinoko.c (the Super Mushroom). Callees in other TUs go
//! through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_itkinoko.rs.
//! No fused sites.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use crate::itlib::COLLIDE_FLOOR_MASK;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn it_8026B390(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026D78C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E7E0(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80276408(a0: *mut HSD_GObj, a1: *mut CollData, a2: *mut Vec);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub static mut it_804D6D24: *mut *mut Article = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

#[inline]
unsafe fn ip_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ItemStateTable it_803F6110[2]` (3F6110)
pub static it_803F6110: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itKinoko_UnkMotion0_Anim), physics_updated: Some(itKinoko_UnkMotion0_Phys), collided: Some(itKinoko_UnkMotion0_Coll) },
    ItemStateTable { anim_id: 0, animated: Some(itKinoko_UnkMotion1_Anim), physics_updated: Some(itKinoko_UnkMotion1_Phys), collided: Some(itKinoko_UnkMotion1_Coll) },
];

/// `HSD_AnimJoint* it_80293660(int idx)` (293660)
pub unsafe extern "C" fn it_80293660(idx: i32) -> *mut HSD_AnimJoint {
    let attrs = (**ext::it_804D6D24.add(It_Kind_Kinoko as usize)).x4_specialAttributes as *mut KinokoAnim;
    (*attrs.offset(idx as isize + 2)).joint
}

/// `void itKinoko_Logic26_Spawned(Item_GObj*)` (29366C)
pub unsafe extern "C" fn itKinoko_Logic26_Spawned(gobj: *mut Item_GObj) {
    let it = ip_of(gobj);
    if (*it).pos.x > 0.0 {
        (*it).facing_dir = -1.0;
    } else {
        (*it).facing_dir = 1.0;
    }
    (*it).xD5C = 0;
    (*it).xDD4_itemVar.kinoko.x0 = 0.0;
    (*it).xDD4_itemVar.kinoko.x4.x = 0.0;
    (*it).xDD4_itemVar.kinoko.x4.y = 1.0;
    (*it).xDD4_itemVar.kinoko.x4.z = 0.0;
    it_8029385C(gobj);
}

/// `void it_802936E4(Item_GObj*)` (2936E4)
pub unsafe extern "C" fn it_802936E4(gobj: *mut Item_GObj) {
    let it = ip_of(gobj);
    (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0;
    (*it).x40_vel.z = 0.0;
    (*it).x40_vel.y = 0.0;

    ext::it_8026B390(gobj);
    ext::Item_8026AE84(it, 0x107, 0x7F, 0x40);
    ext::Item_80268E5C(gobj, 0, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as u32);
}

/// `bool itKinoko_UnkMotion0_Anim(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion0_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

/// `void itKinoko_UnkMotion0_Phys(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion0_Phys(gobj: *mut Item_GObj) {
    let it = ip_of(gobj);
    let attrs = (*(*it).xC4_article_data).x4_specialAttributes as *mut KinokoAttrs;
    if (*it).ground_or_air != GA_Air {
        (*it).xDD4_itemVar.kinoko.x0 = (*attrs).x0 * (*it).facing_dir;
        (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0 * (*it).xDD4_itemVar.kinoko.x4.y;
    }
}

/// `bool itKinoko_UnkMotion0_Coll(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion0_Coll(gobj: *mut Item_GObj) -> i32 {
    let it = ip_of(gobj);
    let cd = &raw mut (*it).x378_itemColl;

    ext::it_8026D78C(gobj, Some(it_80293884));
    if (*it).ground_or_air != GA_Air {
        if (*cd).env_flags & COLLIDE_FLOOR_MASK != 0 {
            ext::it_80276408(gobj, cd, &raw mut (*it).xDD4_itemVar.kinoko.x4);
        } else {
            (*it).xDD4_itemVar.kinoko.x4.x = 0.0;
        }
        if ((*cd).env_flags & Collide_LeftWallMask != 0) || ((*cd).env_flags & Collide_RightWallMask != 0) {
            (*it).facing_dir = -(*it).facing_dir;
            (*it).xDD4_itemVar.kinoko.x0 = -(*it).xDD4_itemVar.kinoko.x0;
            (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0 * (*it).xDD4_itemVar.kinoko.x4.y;
        }
    }
    0
}

/// `void it_8029385C(Item_GObj*)` (29385C)
pub unsafe extern "C" fn it_8029385C(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE as u32);
}

/// `void it_80293884(Item_GObj*)` (293884)
pub unsafe extern "C" fn it_80293884(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 1, (ITEM_UNK_0x1 | ITEM_HIT_PRESERVE) as u32);
}

/// `bool itKinoko_UnkMotion1_Anim(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion1_Anim(_gobj: *mut Item_GObj) -> i32 {
    0
}

/// `void itKinoko_UnkMotion1_Phys(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let it = ip_of(gobj);
    let attr = (*it).xCC_item_attr;
    let sa = (*(*it).xC4_article_data).x4_specialAttributes as *mut KinokoAttrs;

    ext::it_80272860(gobj, (*attr).x10_fall_speed, (*attr).x14_fall_speed_max);

    let vel = if (*it).x40_vel.x < 0.0 { -(*it).x40_vel.x } else { (*it).x40_vel.x };

    if vel < (*sa).x4 {
        (*it).x40_vel.x = 0.0;
        return;
    } else if (*it).x40_vel.x > 0.0 {
        (*it).x40_vel.x -= (*sa).x4;
    }
    if (*it).x40_vel.x < 0.0 {
        (*it).x40_vel.x += (*sa).x4;
    }
}

/// `bool itKinoko_UnkMotion1_Coll(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_UnkMotion1_Coll(gobj: *mut Item_GObj) -> i32 {
    let it = ip_of(gobj);
    let cd = &raw mut (*it).x378_itemColl;

    ext::it_8026E7E0(gobj, Some(it_802936E4));
    if (*it).ground_or_air != GA_Air && ((*cd).env_flags & COLLIDE_FLOOR_MASK != 0) {
        ext::it_80276408(gobj, cd, &raw mut (*it).xDD4_itemVar.kinoko.x4);
    }
    0
}

/// `bool itKinoko_Logic26_DmgDealt(Item_GObj*)`
pub unsafe extern "C" fn itKinoko_Logic26_DmgDealt(_gobj: *mut Item_GObj) -> i32 {
    1
}

/// `void itKinoko_Logic26_EvtUnk(Item_GObj*, HSD_GObj*)`
pub unsafe extern "C" fn itKinoko_Logic26_EvtUnk(gobj: *mut Item_GObj, gobj_2: *mut HSD_GObj) {
    ext::it_8026B894(gobj, gobj_2);
}
