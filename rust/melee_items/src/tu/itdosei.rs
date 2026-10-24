//! Native port of src/melee/it/kinds/itdosei.c (the Dosei: Dodongo-style rolling item): the spawn,
//! the walking/turning motions (`it_803F55D0`), the thrown/dropped/picked states and the hit
//! responses. Callees in other TUs go through `ext`; callees ported in this crate are called
//! directly (`it_dosei_facing_angle` is the C `itDosei_FacingAngle`). `*_p` wrappers return `i32`
//! for the state table (`HSD_GObjPredicate`). Checked by tests/tu_itdosei.rs.
//!
//! Fused sites (shipping contraction): `0.5F * (n * f) + 1.0F` is `fmadds(0.5, n*f, 1.0)`; the
//! facing angle is in `it_dosei_facing_angle` (f64 fused).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_core::fp::fmadds;

use crate::helpers::it_dosei_facing_angle;

/// `ITEM_ANIM_UPDATE` (src/melee/it/forward.h): `1 << 1`.
pub const ITEM_ANIM_UPDATE: u32 = 1 << 1;
/// `JOBJ_HIDDEN` (sysdolphin/baselib/jobj.h).
pub const JOBJ_HIDDEN: u32 = 1 << 4;
/// `M_PI_2` (src/MSL/math.h): a double literal.
pub const M_PI_2: f64 = 1.5707963267948966;
/// `M_PI` (src/MSL/math.h): a double literal.
pub const M_PI: f64 = 3.14159265358979323846;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32);
    fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj);
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_Randi(a0: i32) -> i32;
    fn Item_80268DD4(a0: *mut HSD_GObj, a1: f32);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32);
    fn Item_8026AF0C(a0: *mut Item, a1: i32, a2: u8, a3: u8);
    fn itColl_BounceOffShield(a0: *mut HSD_GObj) -> i32;
    fn itColl_BounceOffVictim(a0: *mut HSD_GObj);
    fn it_8026B390(a0: *mut HSD_GObj);
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32;
    fn it_8026D62C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026D8A4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>) -> i32;
    fn it_8026E15C(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_8026E8C4(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: Option<unsafe extern "C" fn(*mut HSD_GObj)>);
    fn it_802725D4(a0: *mut HSD_GObj);
    fn it_80272860(a0: *mut HSD_GObj, a1: f32, a2: f32);
    fn it_80272C6C(a0: *mut HSD_GObj) -> i32;
    fn it_80273030(a0: *mut HSD_GObj) -> i32;
    fn it_80273130(a0: *mut HSD_GObj) -> i32;
    fn it_80274484(a0: *mut HSD_GObj, a1: *mut HSD_JObj, a2: f32);
    fn it_80274658(a0: *mut HSD_GObj, a1: f32) -> i32;
    fn it_802762B0(a0: *mut Item);
    fn it_802762BC(a0: *mut Item);
    fn it_80276308(a0: *mut HSD_GObj) -> i32;
    fn it_80276CB8(a0: *mut HSD_GObj);
    fn lb_8000BA0C(a0: *mut HSD_JObj, a1: f32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirty(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirty(a0) }
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut attrs: *mut ItemAttr = unsafe { core::mem::zeroed() };
    pub static mut it_804D6D28: *mut ItemCommonData = unsafe { core::mem::zeroed() };
    pub static mut result: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

jobj_inl!();

#[allow(dead_code)]
unsafe fn HSD_JObjSetRotationXWithMtxDirty(jobj: *mut HSD_JObj, x: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(639, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(640, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.x = x;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}
#[allow(dead_code)]
unsafe fn HSD_JObjSetRotationYWithMtxDirty(jobj: *mut HSD_JObj, y: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(660, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(661, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.y = y;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}
#[allow(dead_code)]
unsafe fn HSD_JObjSetRotationZWithMtxDirty(jobj: *mut HSD_JObj, z: f32) {
    assert!(!jobj.is_null(), "HSD_ASSERT(681, jobj)");
    assert!((*jobj).flags & JOBJ_USE_QUATERNION as u32 == 0, "HSD_ASSERT(682, !(jobj->flags & JOBJ_USE_QUATERNION))");
    (*jobj).rotate.z = z;
    if (*jobj).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
        ext::HSD_JObjSetMtxDirty(jobj);
    }
}

macro_rules! pred_i32 {
    ($($name:ident => $f:ident;)*) => { $( pub unsafe extern "C" fn $name(g: *mut HSD_GObj) -> i32 { $f(g) as i32 } )* };
}

#[inline]
unsafe fn item_of(g: *mut HSD_GObj) -> *mut Item {
    (*g).user_data as *mut Item
}

/// `ip->xC4_article_data->x4_specialAttributes` as `itDoseiAttributes`.
#[inline]
unsafe fn attrs_of<'a>(ip: *mut Item) -> &'a itDoseiAttributes {
    &*((*(*ip).xC4_article_data).x4_specialAttributes as *const itDoseiAttributes)
}

/// `GET_JOBJ(gobj)` / `gobj->hsd_obj`.
#[inline]
unsafe fn jobj_of(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `itResetVelocity` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn itResetVelocity(ip: *mut Item) {
    (*ip).x40_vel.x = 0.0;
    (*ip).x40_vel.y = 0.0;
    (*ip).x40_vel.z = 0.0;
}

/// `Item_ApplyFallingPhysics` (src/melee/it/inlines.h, static inline).
#[inline]
unsafe fn Item_ApplyFallingPhysics(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
    ext::it_80274658(gobj, (*ext::it_804D6D28).x68_float);
}

/// `itDosei_SetSpeed` (static inline): the animation frame speed and the joint's speed.
pub unsafe fn itDosei_SetSpeed(gobj: *mut HSD_GObj, ip: *mut Item, speed: f32) {
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = speed;
    ext::lb_8000BA0C(jobj, speed);
}

/// `HSD_JObjSetRotationZero` (static inline).
pub unsafe fn HSD_JObjSetRotationZero(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    HSD_JObjSetRotationX(jobj, 0.0);
    HSD_JObjSetRotationY(jobj, 0.0);
    HSD_JObjSetRotationZ(jobj, 0.0);
}

/// `HSD_JObjSetRotationZeroWithMtxDirty` (static inline).
pub unsafe fn HSD_JObjSetRotationZeroWithMtxDirty(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    HSD_JObjSetRotationXWithMtxDirty(jobj, 0.0);
    HSD_JObjSetRotationYWithMtxDirty(jobj, 0.0);
    HSD_JObjSetRotationZWithMtxDirty(jobj, 0.0);
}

/// `itDosei_FacingAngle` (static inline): `it_dosei_facing_angle`.
pub unsafe fn itDosei_FacingAngle(gobj: *mut HSD_GObj, m: f32) -> f32 {
    it_dosei_facing_angle(&*item_of(gobj), m)
}

/// `it_803F56B8[]` (itdosei.c): the hit-sound ids.
pub static it_803F56B8: [i32; 4] = [0x109, 0x10A, 0x10B, 0];

/// `itDosei_Logic7_Spawned`: reset the velocity and the state variables, then motion 8.
pub unsafe extern "C" fn itDosei_Logic7_Spawned(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let article = (*ip).xC4_article_data;
    let attr = &*((*article).x4_specialAttributes as *const itDoseiAttributes);

    itResetVelocity(ip);
    (*ip).xDD4_itemVar.dosei.xDD4 = attr.unk4;
    (*ip).xDD4_itemVar.dosei.xDF0 = 0;
    (*ip).xDD4_itemVar.dosei.xDE0 = 1.0;
    ext::it_8026B390(gobj);
    itDosei_80282BFC(gobj);
    HSD_JObjSetRotationZero(gobj);
}

/// `itDosei_80281390`: come to rest (motion 0, owner dropped).
pub unsafe extern "C" fn itDosei_80281390(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    itResetVelocity(ip);
    ext::Item_80268E5C(gobj, 0, 3);
    itDosei_SetSpeed(gobj, ip, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion0_Anim`: track the position; after the countdown, start walking.
pub unsafe extern "C" fn itDosei_UnkMotion0_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);

    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    HSD_JObjSetRotationZero(gobj);

    HSD_JObjSetRotationY((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));
    (*ip).xDD4_itemVar.dosei.xDF0 -= 1;
    if (*ip).xDD4_itemVar.dosei.xDF0 <= 0 {
        (*ip).xDD4_itemVar.dosei.xDF0 = 0;
        itDosei_802817A0(gobj);
    }
    false
}

/// `itDosei_UnkMotion0_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion0_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_UnkMotion0_Coll`: land into `itDosei_80282074`.
pub unsafe extern "C" fn itDosei_UnkMotion0_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(itDosei_80282074));
    ext::it_80276CB8(gobj);
    false
}

/// `itDosei_80281734`: choose the turn by the slope under the facing direction.
pub unsafe extern "C" fn itDosei_80281734(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    if (*ip).facing_dir == 1.0 {
        if (*ip).xDD4_itemVar.dosei.xDF8.x < 0.0 {
            itDosei_80281C6C(gobj);
            return;
        }
        itDosei_802817A0(gobj);
        return;
    }
    if (*ip).xDD4_itemVar.dosei.xDF8.x > 0.0 {
        itDosei_80281C6C(gobj);
        return;
    }
    itDosei_802817A0(gobj);
}

/// `itDosei_802817A0`: start walking (motion 1) at the attribute speed.
pub unsafe extern "C" fn itDosei_802817A0(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);
    let attr = attrs_of(ip);

    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x17(1);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).owner = null_mut();
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 1, 0xB);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);
    HSD_JObjSetRotationZero(gobj);

    HSD_JObjSetRotationY((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));
    (*ip).x40_vel.x = (*ip).facing_dir * attr.unk8;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
}

/// `itDosei_UnkMotion1_Anim`: walk; the frame speed follows the floor slope.
pub unsafe extern "C" fn itDosei_UnkMotion1_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    let ip2 = item_of(gobj);
    let n = (*ip2).x378_itemColl.floor.normal.x * (*ip2).facing_dir;
    let frame_speed: f32 = fmadds(0.5, n, 1.0);
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = frame_speed;
    ext::lb_8000BA0C(jobj, frame_speed);
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
    }
    false
}

/// `itDosei_UnkMotion1_Phys`: walk at `unk8` scaled by the frame speed, in the facing direction.
pub unsafe extern "C" fn itDosei_UnkMotion1_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attr = attrs_of(ip);
    let v = attr.unk8 * (*ip).x5D0_animFrameSpeed;
    (*ip).x40_vel.x = (*ip).facing_dir * if v < 0.0 { -v } else { v };
}

/// `itDosei_UnkMotion1_Coll`: stay walking on a flat floor, turn at a wall or a slope.
pub unsafe extern "C" fn itDosei_UnkMotion1_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);

    if ext::it_8026D8A4(gobj, Some(itDosei_80281C6C)) != 0 {
        if ext::it_80276308(gobj) != 0 {
            itDosei_80281C6C(gobj);
            return false;
        }

        {
            let ip2 = item_of(gobj);
            let normal_x = (*ip2).x378_itemColl.floor.normal.x;
            let angle = if normal_x < 0.0 { -normal_x } else { normal_x };
            if angle >= (M_PI as f32) / 4.0 {
                (*ip2).xD5C = 1;
                (*ip2).xDC8_word.flags.set_x1F(1);
            } else {
                (*ip2).xD5C = 0;
                (*ip2).xDC8_word.flags.set_x1F(0);
            }
        }

        if (*ip).xD5C == 1 {
            itDosei_Logic7_EnteredAir(gobj);
        } else {
            ext::it_80276CB8(gobj);
        }
    } else {
        itDosei_80282074(gobj);
    }

    false
}

/// `itDosei_80281C6C`: return to the last position (motion 2).
pub unsafe extern "C" fn itDosei_80281C6C(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDD8 = 0;
    (*ip).pos = (*ip).xDD4_itemVar.dosei.xDE4;
    (*ip).xDD4_itemVar.dosei.xDDC = 0.0;
    (*ip).x40_vel.x = 0.0;
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 2, 3);
    itDosei_SetSpeed(gobj, ip, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion2_Anim`: reverse direction when the animation ends.
pub unsafe extern "C" fn itDosei_UnkMotion2_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 1, ITEM_ANIM_UPDATE);
        (*ip).facing_dir = -(*ip).facing_dir;

        HSD_JObjSetRotationY((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));
    }
    false
}

/// `itDosei_UnkMotion2_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion2_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_SetupWalk_FC` (static inline): walk again after the turn.
pub unsafe fn itDosei_SetupWalk_FC(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);
    let attr = attrs_of(ip);

    (*ip).xD5C = 0;
    (*ip).xDC8_word.flags.set_x17(1);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).owner = null_mut();
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 1, 0xB);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);

    HSD_JObjSetRotationZeroWithMtxDirty(gobj);
    HSD_JObjSetRotationYWithMtxDirty((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));

    (*ip).x40_vel.x = (*ip).facing_dir * attr.unk8;
    (*ip).x40_vel.z = 0.0;
    (*ip).x40_vel.y = 0.0;
}

/// `itDosei_UnkMotion2_Coll`: land into `itDosei_80282074`; restart the walk once counted.
pub unsafe extern "C" fn itDosei_UnkMotion2_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    ext::it_8026D62C(gobj, Some(itDosei_80282074));
    if (*ip).xDD4_itemVar.dosei.xDD8 == 2 {
        itDosei_SetupWalk_FC(gobj);
    }
    ext::it_80276CB8(gobj);
    false
}

/// `itDosei_80282074`: settle (motion 3, owner dropped).
pub unsafe extern "C" fn itDosei_80282074(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::Item_80268E5C(gobj, 3, 3);
    itDosei_SetSpeed(gobj, ip, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion3_Anim`: hold the position while the animation plays.
pub unsafe extern "C" fn itDosei_UnkMotion3_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 3, ITEM_ANIM_UPDATE);
    }
    false
}

/// `itDosei_UnkMotion3_Phys`: fall.
pub unsafe extern "C" fn itDosei_UnkMotion3_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
}

/// `itDosei_UnkMotion5_Coll`: land into `itDosei_802817A0`.
pub unsafe extern "C" fn itDosei_UnkMotion5_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(itDosei_802817A0));
    false
}

/// `itDosei_Logic7_PickedUp`: show the body and set the speed; re-face unless already held.
pub unsafe extern "C" fn itDosei_Logic7_PickedUp(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    let ip = item_of(gobj);
    let attr = attrs_of(ip);

    if (*ip).msid != 4 {
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
        ext::HSD_JObjClearFlagsAll(HSD_JObjGetChild(jobj), JOBJ_HIDDEN);
        (*ip).x5D0_animFrameSpeed = attr.unk0;
        ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
        (*ip).xDC8_word.flags.set_x17(1);
        (*ip).xDC8_word.flags.set_x19(1);
    } else {
        ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
    }
    if (*ip).msid != 4 {
        HSD_JObjSetRotationZero(gobj);

        HSD_JObjSetRotationY(jobj_of(gobj), itDosei_FacingAngle(gobj, 0.0));
    }
}

/// `itDosei_UnkMotion4_Anim_inline` (static inline): reset the rotation and face the item.
pub unsafe fn itDosei_UnkMotion4_Anim_inline(gobj: *mut HSD_GObj, jobj: *mut HSD_JObj) {
    HSD_JObjSetRotationXWithMtxDirty(jobj, 0.0);
    HSD_JObjSetRotationYWithMtxDirty(jobj, 0.0);
    HSD_JObjSetRotationZWithMtxDirty(jobj, 0.0);
    let angle = itDosei_FacingAngle(gobj, 0.0);

    HSD_JObjSetRotationYWithMtxDirty(jobj_of(gobj), angle);
}

/// `itDosei_UnkMotion4_GetAnimSpeed` (static inline): `attr->unk0`.
pub unsafe fn itDosei_UnkMotion4_GetAnimSpeed(attr: *mut itDoseiAttributes) -> f32 {
    (*attr).unk0
}

/// `itDosei_UnkMotion4_GetAttrs` (static inline): the special attributes.
pub unsafe fn itDosei_UnkMotion4_GetAttrs(ip: *mut Item) -> *mut itDoseiAttributes {
    (*(*ip).xC4_article_data).x4_specialAttributes as *mut itDoseiAttributes
}

/// `itDosei_UnkMotion4_Anim`: when not yet picked up, show the body and set the animation speed.
pub unsafe extern "C" fn itDosei_UnkMotion4_Anim(gobj: *mut HSD_GObj) -> bool {
    if ext::it_80272C6C(gobj) == 0 {
        let ip = item_of(gobj);
        let jobj = jobj_of(gobj);
        let attr = itDosei_UnkMotion4_GetAttrs(ip);

        if (*ip).msid != 4 {
            ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
            ext::HSD_JObjClearFlagsAll(HSD_JObjGetChild(jobj), JOBJ_HIDDEN);
            (*ip).x5D0_animFrameSpeed = itDosei_UnkMotion4_GetAnimSpeed(attr);
            ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
            (*ip).xDC8_word.flags.set_x17(1);
            (*ip).xDC8_word.flags.set_x19(1);
        } else {
            ext::Item_80268E5C(gobj, 4, ITEM_ANIM_UPDATE);
        }
        if (*ip).msid != 4 {
            itDosei_UnkMotion4_Anim_inline(gobj, jobj_of(gobj));
        }
    }
    false
}

/// `itDosei_UnkMotion4_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion4_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_GetJObj` (static inline): `gobj->hsd_obj`.
pub unsafe fn itDosei_GetJObj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

/// `itDosei_Logic7_Dropped`: the body reappears at the dropped speed (motion 5).
pub unsafe extern "C" fn itDosei_Logic7_Dropped(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = itDosei_GetJObj(gobj);
    let attr = attrs_of(ip);

    ext::it_8026B390(gobj);
    ext::HSD_JObjClearFlagsAll(HSD_JObjGetChild(jobj), JOBJ_HIDDEN);
    (*ip).xDC8_word.flags.set_x17(1);
    (*ip).xDC8_word.flags.set_x19(1);
    ext::Item_80268E5C(gobj, 5, 6);
    (*ip).x5D0_animFrameSpeed = attr.unk0;
    ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
    HSD_JObjSetRotationZero(gobj);

    HSD_JObjSetRotationY((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));
}

/// `itDosei_Logic7_Thrown`: launch (motion 5) with the thrown speed `xDE0`.
pub unsafe extern "C" fn itDosei_Logic7_Thrown(gobj: *mut HSD_GObj) {
    let jobj = jobj_of(gobj);
    let ip = item_of(gobj);
    let attr = attrs_of(ip);

    if (*ip).msid != 5 {
        ext::it_8026B390(gobj);
        ext::HSD_JObjClearFlagsAll(HSD_JObjGetChild(jobj), JOBJ_HIDDEN);
        ext::Item_80268E5C(gobj, 5, 6);
        (*ip).x5D0_animFrameSpeed = attr.unk0;
        ext::lb_8000BA0C(jobj, (*ip).x5D0_animFrameSpeed);
    } else {
        ext::Item_80268E5C(gobj, 5, 6);
    }
    ext::it_80274484(gobj, jobj, (*ip).xDD4_itemVar.dosei.xDE0);
}

/// `itDosei_UnkMotion5_Anim` (returns false).
pub unsafe extern "C" fn itDosei_UnkMotion5_Anim(_gobj: *mut HSD_GObj) -> bool {
    false
}

/// `itDosei_UnkMotion5_Phys`: fall.
pub unsafe extern "C" fn itDosei_UnkMotion5_Phys(gobj: *mut HSD_GObj) {
    Item_ApplyFallingPhysics(gobj);
}

/// `itDosei_Logic7_EnteredAir`: pick the airborne motion (10, or 6 when the item was dropped).
pub unsafe extern "C" fn itDosei_Logic7_EnteredAir(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let motion_id: i32 = if (*ip).xDC8_word.flags.x19() == 0 { 0xA } else { 6 };
    ext::Item_80268E5C(gobj, motion_id, 3);
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion6_Anim`: track the position.
pub unsafe extern "C" fn itDosei_UnkMotion6_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    false
}

/// `itDosei_UnkMotion6_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion6_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_UnkMotion6_Coll`: keep the floor normal, then choose the turn.
pub unsafe extern "C" fn itDosei_UnkMotion6_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDF8 = (*ip).x378_itemColl.floor.normal;
    ext::it_8026E8C4(gobj, Some(itDosei_80281734), Some(itDosei_80282074));
    false
}

/// `itDosei_80282BFC`: the idle state (motion 8).
pub unsafe extern "C" fn itDosei_80282BFC(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    ext::Item_80268E5C(gobj, 8, 3);
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion8_Anim`: track the position.
pub unsafe extern "C" fn itDosei_UnkMotion8_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    false
}

/// `itDosei_UnkMotion8_Phys`: fall.
pub unsafe extern "C" fn itDosei_UnkMotion8_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
}

/// `itDosei_UnkMotion8_Coll`: land into `itDosei_80282CD4`.
pub unsafe extern "C" fn itDosei_UnkMotion8_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(itDosei_80282CD4));
    false
}

/// `itDosei_80282CD4`: stop and enter motion 7.
pub unsafe extern "C" fn itDosei_80282CD4(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    itResetVelocity(ip);
    ext::it_802762B0(ip);
    ext::Item_80268E5C(gobj, 7, 3);
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);
    (*ip).owner = null_mut();
}

/// `itDosei_UnkMotion7_Anim`: count down, then roll (`itDosei_80282DE4`).
pub unsafe extern "C" fn itDosei_UnkMotion7_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;

    if (*ip).xDD4_itemVar.dosei.xDD4 > 0 {
        (*ip).xDD4_itemVar.dosei.xDD4 -= 1;
    } else {
        itDosei_80282DE4(gobj);
    }

    false
}

/// `itDosei_UnkMotion7_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion7_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_UnkMotion7_Coll`: land into `itDosei_80282BFC`.
pub unsafe extern "C" fn itDosei_UnkMotion7_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026D62C(gobj, Some(itDosei_80282BFC));
    ext::it_80276CB8(gobj);
    false
}

/// `itDosei_80282DE4` (static): start the half-turn of the roll (motion 9).
pub unsafe fn itDosei_80282DE4(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let jobj = jobj_of(gobj);

    (*ip).xDD4_itemVar.dosei.xDD8 = 1;
    (*ip).xDD4_itemVar.dosei.xDDC = M_PI_2 as f32;
    (*ip).x40_vel.x = 0.0;
    (*ip).xDC8_word.flags.set_x17(1);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).owner = null_mut();
    ext::Item_80268E5C(gobj, 9, 3);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);

    HSD_JObjSetRotationZero(gobj);

    HSD_JObjSetRotationY(
        (*gobj).hsd_obj as *mut HSD_JObj,
        itDosei_FacingAngle(gobj, -(*ip).xDD4_itemVar.dosei.xDDC),
    );
}

/// `itDosei_UnkMotion9_Anim`: advance the roll angle by `PI / unkC` per frame; start walking
/// when it reaches zero.
pub unsafe extern "C" fn itDosei_UnkMotion9_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    let attr = attrs_of(ip);

    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    (*ip).xDD4_itemVar.dosei.xDDC =
        ((*ip).xDD4_itemVar.dosei.xDDC as f64 - (M_PI / attr.unkC as f64)) as f32;
    if (*ip).xDD4_itemVar.dosei.xDD8 == 1 {
        HSD_JObjSetRotationY(
            (*gobj).hsd_obj as *mut HSD_JObj,
            itDosei_FacingAngle(gobj, -(*ip).xDD4_itemVar.dosei.xDDC),
        );
        if (*ip).xDD4_itemVar.dosei.xDDC <= 0.0 {
            (*ip).xDD4_itemVar.dosei.xDD8 = 2;
            (*ip).x40_vel.x = (*ip).facing_dir * attr.unk8;
        }
    }
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 9, ITEM_ANIM_UPDATE);
    }
    false
}

/// `itDosei_UnkMotion9_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion9_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_UnkMotion9_Coll`: land; once the roll is done, walk again at the captured frame.
pub unsafe extern "C" fn itDosei_UnkMotion9_Coll(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    ext::it_8026D62C(gobj, Some(itDosei_80282074));
    if (*ip).xDD4_itemVar.dosei.xDD8 == 2 {
        let frame = (*ip).x5CC_currentAnimFrame;
        itDosei_SetupWalk_FC(gobj);
        ext::Item_80268DD4(gobj, frame);
    }
    ext::it_80276CB8(gobj);
    false
}

/// `itDosei_UnkMotion10_Anim`: track the position at unit speed.
pub unsafe extern "C" fn itDosei_UnkMotion10_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    let jobj = jobj_of(gobj);
    (*ip).x5D0_animFrameSpeed = 1.0;
    ext::lb_8000BA0C(jobj, 1.0);
    false
}

/// `itDosei_UnkMotion10_Phys` (empty).
pub unsafe extern "C" fn itDosei_UnkMotion10_Phys(_gobj: *mut HSD_GObj) {}

/// `itDosei_UnkMotion10_Coll`: land into `itDosei_80282CD4`, or back to `itDosei_80282BFC`.
pub unsafe extern "C" fn itDosei_UnkMotion10_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E8C4(gobj, Some(itDosei_80282CD4), Some(itDosei_80282BFC));
    false
}

/// `itDosei_Logic7_DmgReceived`: launch from the hit (motion 0xB), scaled by the attributes.
pub unsafe extern "C" fn itDosei_Logic7_DmgReceived(gobj: *mut HSD_GObj) -> i32 {
    let ip = item_of(gobj);
    let attr = attrs_of(ip);

    {
        let jobj = itDosei_GetJObj(gobj);
        ext::Item_80268E5C(gobj, 0xB, 3);
        ext::it_802762BC(ip);
        (*ip).x5D0_animFrameSpeed = 1.0;
        ext::lb_8000BA0C(jobj, 1.0);
    }

    (*ip).x40_vel.x = (attr.unk10 as f32) * (*ip).facing_dir;
    let unk14_abs = if attr.unk14 < 0 { -attr.unk14 } else { attr.unk14 };
    (*ip).x40_vel.y = unk14_abs as f32;
    (*ip).x40_vel.z = 0.0;
    (*ip).xDC8_word.flags.set_x17(1);
    (*ip).xDC8_word.flags.set_x19(1);
    (*ip).xDD4_itemVar.dosei.xDF0 = 20;
    HSD_JObjSetRotationZero(gobj);

    HSD_JObjSetRotationY((*gobj).hsd_obj as *mut HSD_JObj, itDosei_FacingAngle(gobj, 0.0));
    (*ip).owner = null_mut();
    (*ip).xD44_lifeTimer -= 60.0;
    ext::it_80273130(gobj)
}

/// `itDosei_UnkMotion11_Anim`: track the position; hold motion 11.
pub unsafe extern "C" fn itDosei_UnkMotion11_Anim(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    (*ip).xDD4_itemVar.dosei.xDE4 = (*ip).pos;
    if ext::it_80272C6C(gobj) == 0 {
        ext::Item_80268E5C(gobj, 11, ITEM_ANIM_UPDATE);
    }
    false
}

/// `itDosei_UnkMotion11_Phys`: fall.
pub unsafe extern "C" fn itDosei_UnkMotion11_Phys(gobj: *mut HSD_GObj) {
    let ip = item_of(gobj);
    let attrs = &*(*ip).xCC_item_attr;
    ext::it_80272860(gobj, attrs.x10_fall_speed, attrs.x14_fall_speed_max);
}

/// `itDosei_UnkMotion11_Coll`: land into `itDosei_80281390`.
pub unsafe extern "C" fn itDosei_UnkMotion11_Coll(gobj: *mut HSD_GObj) -> bool {
    ext::it_8026E15C(gobj, Some(itDosei_80281390));
    false
}

/// `itDosei_Logic7_DmgDealt`: the hit sound; a thrown Dosei bounces off its victim.
pub unsafe extern "C" fn itDosei_Logic7_DmgDealt(gobj: *mut HSD_GObj) -> bool {
    let ip = item_of(gobj);
    ext::Item_8026AF0C(ip, it_803F56B8[ext::HSD_Randi(3) as usize], 127, 64);
    if (*ip).msid == 5 {
        ext::itColl_BounceOffVictim(gobj);
        ext::it_802725D4(gobj);
    }
    false
}

/// `itDosei_Logic7_Reflected`: `it_80273030`.
pub unsafe extern "C" fn itDosei_Logic7_Reflected(gobj: *mut HSD_GObj) -> i32 {
    ext::it_80273030(gobj)
}

/// `itDosei_Logic7_Clanked`: bounce off the victim; false.
pub unsafe extern "C" fn itDosei_Logic7_Clanked(gobj: *mut HSD_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

/// `itDosei_Logic7_HitShield`: bounce off the victim; false.
pub unsafe extern "C" fn itDosei_Logic7_HitShield(gobj: *mut HSD_GObj) -> bool {
    ext::itColl_BounceOffVictim(gobj);
    false
}

/// `itDosei_Logic7_ShieldBounced`: `itColl_BounceOffShield`.
pub unsafe extern "C" fn itDosei_Logic7_ShieldBounced(gobj: *mut HSD_GObj) -> i32 {
    ext::itColl_BounceOffShield(gobj)
}

/// `itDosei_Logic7_EvtUnk`: `it_8026B894`.
pub unsafe extern "C" fn itDosei_Logic7_EvtUnk(gobj: *mut HSD_GObj, ref_gobj: *mut HSD_GObj) {
    ext::it_8026B894(gobj, ref_gobj);
}

pred_i32! {
    itDosei_UnkMotion0_Anim_p => itDosei_UnkMotion0_Anim;
    itDosei_UnkMotion0_Coll_p => itDosei_UnkMotion0_Coll;
    itDosei_UnkMotion1_Anim_p => itDosei_UnkMotion1_Anim;
    itDosei_UnkMotion1_Coll_p => itDosei_UnkMotion1_Coll;
    itDosei_UnkMotion2_Anim_p => itDosei_UnkMotion2_Anim;
    itDosei_UnkMotion2_Coll_p => itDosei_UnkMotion2_Coll;
    itDosei_UnkMotion3_Anim_p => itDosei_UnkMotion3_Anim;
    itDosei_UnkMotion5_Coll_p => itDosei_UnkMotion5_Coll;
    itDosei_UnkMotion4_Anim_p => itDosei_UnkMotion4_Anim;
    itDosei_UnkMotion5_Anim_p => itDosei_UnkMotion5_Anim;
    itDosei_UnkMotion6_Anim_p => itDosei_UnkMotion6_Anim;
    itDosei_UnkMotion6_Coll_p => itDosei_UnkMotion6_Coll;
    itDosei_UnkMotion7_Anim_p => itDosei_UnkMotion7_Anim;
    itDosei_UnkMotion7_Coll_p => itDosei_UnkMotion7_Coll;
    itDosei_UnkMotion8_Anim_p => itDosei_UnkMotion8_Anim;
    itDosei_UnkMotion8_Coll_p => itDosei_UnkMotion8_Coll;
    itDosei_UnkMotion9_Anim_p => itDosei_UnkMotion9_Anim;
    itDosei_UnkMotion9_Coll_p => itDosei_UnkMotion9_Coll;
    itDosei_UnkMotion10_Anim_p => itDosei_UnkMotion10_Anim;
    itDosei_UnkMotion10_Coll_p => itDosei_UnkMotion10_Coll;
    itDosei_UnkMotion11_Anim_p => itDosei_UnkMotion11_Anim;
    itDosei_UnkMotion11_Coll_p => itDosei_UnkMotion11_Coll;
}

/// `it_803F55D0[]` (itdosei.c).
pub static it_803F55D0: [ItemStateTable; 12] = [
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion0_Anim_p), physics_updated: Some(itDosei_UnkMotion0_Phys), collided: Some(itDosei_UnkMotion0_Coll_p) },
    ItemStateTable { anim_id: 0, animated: Some(itDosei_UnkMotion1_Anim_p), physics_updated: Some(itDosei_UnkMotion1_Phys), collided: Some(itDosei_UnkMotion1_Coll_p) },
    ItemStateTable { anim_id: 2, animated: Some(itDosei_UnkMotion2_Anim_p), physics_updated: Some(itDosei_UnkMotion2_Phys), collided: Some(itDosei_UnkMotion2_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion3_Anim_p), physics_updated: Some(itDosei_UnkMotion3_Phys), collided: Some(itDosei_UnkMotion5_Coll_p) },
    ItemStateTable { anim_id: 3, animated: Some(itDosei_UnkMotion4_Anim_p), physics_updated: Some(itDosei_UnkMotion4_Phys), collided: None },
    ItemStateTable { anim_id: 1, animated: Some(itDosei_UnkMotion5_Anim_p), physics_updated: Some(itDosei_UnkMotion5_Phys), collided: Some(itDosei_UnkMotion5_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion6_Anim_p), physics_updated: Some(itDosei_UnkMotion6_Phys), collided: Some(itDosei_UnkMotion6_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion7_Anim_p), physics_updated: Some(itDosei_UnkMotion7_Phys), collided: Some(itDosei_UnkMotion7_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion8_Anim_p), physics_updated: Some(itDosei_UnkMotion8_Phys), collided: Some(itDosei_UnkMotion8_Coll_p) },
    ItemStateTable { anim_id: 0, animated: Some(itDosei_UnkMotion9_Anim_p), physics_updated: Some(itDosei_UnkMotion9_Phys), collided: Some(itDosei_UnkMotion9_Coll_p) },
    ItemStateTable { anim_id: -1, animated: Some(itDosei_UnkMotion10_Anim_p), physics_updated: Some(itDosei_UnkMotion10_Phys), collided: Some(itDosei_UnkMotion10_Coll_p) },
    ItemStateTable { anim_id: 3, animated: Some(itDosei_UnkMotion11_Anim_p), physics_updated: Some(itDosei_UnkMotion11_Phys), collided: Some(itDosei_UnkMotion11_Coll_p) },
];
