//! Port of src/melee/it/kinds/ityoshieggthrow.c: the egg Yoshi throws (logic table 43). Callees of unported TUs
//! go through `ext` (unset: panics "TODO"). Checked against the C (whole TU, shipping contraction) by
//! tests/kinds_ityoshieggthrow.rs.
//!
//! `efAsync_Spawn` is variadic in the C; its only extra argument (the effect scale, spawn kind 4) is the sixth
//! parameter of the `ext` hook and is null for the five-argument calls.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj;
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026A8EC(a0: *mut HSD_GObj);
    fn Item_8026AB54(a0: *mut HSD_GObj, a1: *mut HSD_GObj, a2: u32);
    fn Item_8026AE84(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut f32);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn it_8026B3A8(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026BB44(a0: *mut HSD_GObj);
    fn it_8026BB68(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_8026BD24(a0: *mut HSD_GObj);
    fn it_8026E5A0(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_80273454(a0: *mut HSD_GObj);
    fn it_80273598(a0: *mut HSD_GObj, a1: i32, a2: i32);
    fn it_8027429C(a0: *mut HSD_GObj, a1: *mut Vec);
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    fn it_80275158(a0: *mut HSD_GObj, a1: f32);
    fn it_8027518C(a0: *mut HSD_GObj);
    fn it_802751D8(a0: *mut HSD_GObj) -> i32;
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

/// `ItemStateTable it_803F7118[]`
pub static mut it_803F7118: [ItemStateTable; 3] = [
    st_row(-1, None, None, None),
    st_row(
        0,
        pred_b(itYoshieggthrow_UnkMotion1_Anim),
        Some(itYoshieggthrow_UnkMotion1_Phys),
        pred_b(itYoshieggthrow_UnkMotion1_Coll),
    ),
    st_row(1, pred_b(itYoshieggthrow_UnkMotion2_Anim), None, None),
];

pub unsafe extern "C" fn it_802B2890(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    (*ip).owner = null_mut();
    (*ip).xDC8_word.flags.set_x13(0);
    ext::Item_8026A8EC(gobj);
}

pub unsafe extern "C" fn it_802B28C8(gobj: *mut Item_GObj, pos: *mut Vec3, offset: *mut Vec3, facing_dir: f32, _unused: f32) {
    let ip = (*gobj).user_data as *mut Item;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;

    it_802B2B08(gobj);
    ext::it_8027429C(gobj, pos);
    (*ip).xDC8_word.flags.set_x14(0);
    ext::it_8026B3A8(gobj);
    (*ip).pos.x += (*offset).x;
    (*ip).pos.y += (*offset).y;
    (*ip).pos.z += (*offset).z;
    HSD_JObjSetTranslate(jobj, &mut (*ip).pos);
    ext::it_80274658(gobj, facing_dir);
}

pub unsafe extern "C" fn it_802B2A10(parent: *mut Fighter_GObj, pos: *mut Vec3, part: i32, facing_dir: f32) -> *mut Item_GObj {
    let mut spawn: SpawnItem = core::mem::zeroed();

    spawn.kind = It_Kind_Yoshi_EggThrow;
    spawn.prev_pos = *pos;
    ext::it_8026BB68(parent, &mut spawn.pos);
    spawn.facing_dir = facing_dir;
    spawn.x3C_damage = 0;
    spawn.vel.z = 0.0;
    spawn.vel.y = 0.0;
    spawn.vel.x = 0.0;
    spawn.x0_parent_gobj = parent;
    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
    spawn.x44_flag.anon_0.set_b0(1);
    spawn.x40 = 0;
    let gobj = ext::Item_80268B18(&mut spawn);
    if !gobj.is_null() {
        ext::Item_8026AB54(gobj, parent, part as u32);
    }
    gobj
}

pub unsafe extern "C" fn itYoshiEggThrow_Logic43_PickedUp(gobj: *mut Item_GObj) {
    ext::Item_80268E5C(gobj, 0, ITEM_ANIM_UPDATE);
}

pub unsafe extern "C" fn it_802B2B08(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itYoshiEggThrowAttributes;
    ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    ext::it_80275158(gobj, (*attrs).x0);
}

pub unsafe extern "C" fn itYoshieggthrow_UnkMotion1_Anim(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);

    if (*ip).xD44_lifeTimer <= 0.0 {
        it_802B2C38(gobj);
    } else {
        (*ip).xD44_lifeTimer -= 1.0;
    }

    false
}

pub unsafe extern "C" fn itYoshieggthrow_UnkMotion1_Phys(gobj: *mut Item_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*ip).xCC_item_attr;
    ext::it_80272860(gobj, (*attrs).x10_fall_speed, (*attrs).x14_fall_speed_max);
}

pub unsafe extern "C" fn itYoshieggthrow_UnkMotion1_Coll(gobj: *mut Item_GObj) -> bool {
    ext::it_8026E5A0(gobj, Some(it_802B2C38));
    false
}

pub unsafe extern "C" fn it_802B2C04(gobj: *mut Item_GObj) -> bool {
    if (*((*gobj).user_data as *mut Item)).msid != 2 {
        it_802B2C38(gobj);
    }
    false
}

/// `spawn1` (static inline)
#[inline]
unsafe fn spawn1(gobj: *mut HSD_GObj, jobj: *mut HSD_JObj) {
    let ip = GET_ITEM(gobj);
    ext::efAsync_Spawn(gobj, &raw mut (*ip).xBC0 as *mut c_void, 1, 0x4CE, jobj, null_mut());
}

/// `spawn2` (static inline)
#[inline]
unsafe fn spawn2(gobj: *mut HSD_GObj, jobj: *mut HSD_JObj) {
    let ip = GET_ITEM(gobj);
    let mut scale: f32 = 1.0;
    ext::efAsync_Spawn(gobj, &raw mut (*GET_ITEM(gobj)).xBC0 as *mut c_void, 4, 0x4CF, jobj, &mut scale);
}

pub unsafe extern "C" fn it_802B2C38(gobj: *mut HSD_GObj) {
    let ip = GET_ITEM(gobj);
    let attrs = (*(*ip).xC4_article_data).x4_specialAttributes as *mut itYoshiEggThrowAttributes;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Item_80268E5C(gobj, 2, 0x12);
    ext::it_8026BB44(gobj);
    ext::it_80273598(gobj, 0xC, 0xA);
    ext::it_8026BD24(gobj);
    ext::it_8027518C(gobj);
    ext::it_80273454(gobj);
    (*ip).xD44_lifeTimer = (*attrs).x4;
    spawn1(gobj, jobj);
    spawn2(gobj, jobj);
    ext::Item_8026AE84(ip, 0x44618, 0x7F, 0x40);
}

pub unsafe extern "C" fn itYoshieggthrow_UnkMotion2_Anim(gobj: *mut Item_GObj) -> bool {
    ext::it_802751D8(gobj) != 0
}

pub unsafe extern "C" fn it_2725_Logic43_Clanked(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 2 {
        it_802B2C38(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802B2E5C(gobj: *mut Item_GObj) -> bool {
    ext::it_80273030(gobj) != 0
}

pub unsafe extern "C" fn it_802B2E7C(gobj: *mut Item_GObj) -> bool {
    let ip = GET_ITEM(gobj);
    if (*ip).msid != 2 {
        it_802B2C38(gobj);
    }
    false
}

pub unsafe extern "C" fn it_802B2F88(gobj: *mut Item_GObj) -> bool {
    ext::itColl_BounceOffShield(gobj) != 0
}

pub unsafe extern "C" fn it_802B2FA8(gobj: *mut Item_GObj, ref_gobj: *mut Item_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}
