//! Native port of src/melee/it/kinds/itdkinoko.c (Dr. Mario-less Kinoko "D-Kinoko" mushroom): the
//! spawn, the sliding/bouncing motion 0 and the decelerating motion 1 (`it_803F6130`). Callees in
//! other TUs go through `ext`; callees ported in this crate are called directly. `*_p` wrappers
//! return `i32` for the state table (`HSD_GObjPredicate`). Checked by tests/tu_itdkinoko.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use crate::itlib::{it_8026b390, it_8026b894, it_8026bd24, COLLIDE_FLOOR_MASK as Collide_FloorMask};
use crate::itlib3::it_80272860;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;

// BEGIN GENERATED EXT (callees of the C TU that are not ported in this crate yet)
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
    fn Item_80268E5C(gobj: *mut HSD_GObj, msid: i32, flags: u32);
    fn Item_8026AE84(ip: *mut Item, a1: i32, a2: u8, a3: u8);
    fn it_8026D78C(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    fn it_8026E414(gobj: *mut HSD_GObj, cb: HSD_GObjEvent);
    }
}
// END GENERATED EXT

macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ip->xC4_article_data->x4_specialAttributes` as a `T`.
#[inline]
unsafe fn sa<'a, T>(ip: *mut Item) -> &'a T {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const T)
}

/// `itDKinoko_Logic27_Spawned`: face away from the origin, reset the slide, enter motion 1.
pub unsafe extern "C" fn itDKinoko_Logic27_Spawned(gobj: *mut HSD_GObj) {
    let it = item_of(gobj);
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
    it_80293BE8(gobj);
}

/// `it_80293A70`: launch along the slide speed and play the landing effect.
pub unsafe extern "C" fn it_80293A70(gobj: *mut HSD_GObj) {
    let it = item_of(gobj);
    (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0;
    (*it).x40_vel.y = 0.0;
    (*it).x40_vel.z = 0.0;

    it_8026b390(&mut *it);
    ext::Item_8026AE84(it, 0x107, 0x7F, 0x40);
    ext::Item_80268E5C(gobj, 0, (ITEM_UNK_0x1 as u32) | (ITEM_HIT_PRESERVE as u32));
}

/// `itDkinoko_UnkMotion0_Anim` (returns false).
pub unsafe extern "C" fn itDkinoko_UnkMotion0_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itDkinoko_UnkMotion0_Phys`: slide at `attrs->x0` in the facing direction while grounded.
pub unsafe extern "C" fn itDkinoko_UnkMotion0_Phys(gobj: *mut HSD_GObj) {
    let it = item_of(gobj);
    let attrs = sa::<KinokoAttrs>(it);
    if (*it).ground_or_air != GA_Air {
        (*it).xDD4_itemVar.kinoko.x0 = attrs.x0 * (*it).facing_dir;
        (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0 * (*it).xDD4_itemVar.kinoko.x4.y;
    }
}

/// `itDkinoko_UnkMotion0_Coll`: bounce off walls, track the floor normal.
pub unsafe extern "C" fn itDkinoko_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    let it = item_of(gobj);
    let cd: *mut CollData = &mut (*it).x378_itemColl;

    ext::it_8026D78C(gobj, Some(it_80293C10));
    if (*it).ground_or_air != GA_Air {
        if (*cd).env_flags & Collide_FloorMask != 0 {
            crate::itlib::it_80276408(&*cd, &mut (*it).xDD4_itemVar.kinoko.x4);
        } else {
            (*it).xDD4_itemVar.kinoko.x4.x = 0.0;
        }
        if (*cd).env_flags & Collide_LeftWallMask != 0 || (*cd).env_flags & Collide_RightWallMask != 0 {
            (*it).facing_dir = -(*it).facing_dir;
            (*it).xDD4_itemVar.kinoko.x0 = -(*it).xDD4_itemVar.kinoko.x0;
            (*it).x40_vel.x = (*it).xDD4_itemVar.kinoko.x0 * (*it).xDD4_itemVar.kinoko.x4.y;
        }
    }
    false
}

/// `it_80293BE8`: enter motion 1.
pub unsafe extern "C" fn it_80293BE8(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
}

/// `it_80293C10`: enter motion 1 keeping the hitboxes.
pub unsafe extern "C" fn it_80293C10(gobj: *mut HSD_GObj) {
    ext::Item_80268E5C(gobj, 1, (ITEM_UNK_0x1 as u32) | (ITEM_HIT_PRESERVE as u32));
}

/// `itDkinoko_UnkMotion1_Anim` (returns false).
pub unsafe extern "C" fn itDkinoko_UnkMotion1_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itDkinoko_UnkMotion1_Phys`: fall, then decelerate the slide by `sa->x4` (snapping to zero).
pub unsafe extern "C" fn itDkinoko_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    let it = item_of(gobj);
    let attr = &*(*it).xCC_item_attr;
    let sa = sa::<KinokoAttrs>(it);

    crate::itlib3::it_80272860(&mut *it, attr.x10_fall_speed, attr.x14_fall_speed_max);

    let vel: f32 = if (*it).x40_vel.x < 0.0 { -(*it).x40_vel.x } else { (*it).x40_vel.x };

    if vel < sa.x4 {
        (*it).x40_vel.x = 0.0;
        return;
    } else if (*it).x40_vel.x > 0.0 {
        (*it).x40_vel.x -= sa.x4;
    }
    if (*it).x40_vel.x < 0.0 {
        (*it).x40_vel.x += sa.x4;
    }
}

/// `itDkinoko_UnkMotion1_Coll`: floor tracking while grounded.
pub unsafe extern "C" fn itDkinoko_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> bool {
    let it = item_of(gobj);
    let cd: *mut CollData = &mut (*it).x378_itemColl;

    ext::it_8026E414(gobj, Some(it_80293A70));
    if (*it).ground_or_air != GA_Air && (*cd).env_flags & Collide_FloorMask != 0 {
        crate::itlib::it_80276408(&*cd, &mut (*it).xDD4_itemVar.kinoko.x4);
    }
    false
}

/// `itDKinoko_Logic27_DmgDealt`: always true.
pub unsafe extern "C" fn itDKinoko_Logic27_DmgDealt(_gobj: *mut HSD_GObj) -> bool {
    true
}

/// `itDKinoko_Logic27_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itDKinoko_Logic27_EvtUnk(gobj: *mut HSD_GObj, gobj_2: *mut HSD_GObj) {
    it_8026b894(&mut *item_of(gobj), gobj_2);
}

pred_i32! {
    itDkinoko_UnkMotion0_Anim_p => itDkinoko_UnkMotion0_Anim;
    itDkinoko_UnkMotion0_Coll_p => itDkinoko_UnkMotion0_Coll;
    itDkinoko_UnkMotion1_Anim_p => itDkinoko_UnkMotion1_Anim;
    itDkinoko_UnkMotion1_Coll_p => itDkinoko_UnkMotion1_Coll;
}

/// `it_803F6130[]` (itdkinoko.c).
pub static it_803F6130: [ItemStateTable; 2] = [
    ItemStateTable { anim_id: 0, animated: Some(itDkinoko_UnkMotion0_Anim_p), physics_updated: Some(itDkinoko_UnkMotion0_Phys), collided: Some(itDkinoko_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: 0, animated: Some(itDkinoko_UnkMotion1_Anim_p), physics_updated: Some(itDkinoko_UnkMotion1_Phys), collided: Some(itDkinoko_UnkMotion1_Coll_p) },
];
